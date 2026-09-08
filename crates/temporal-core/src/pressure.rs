//! Proof-v1 individual pressure over deterministic opportunity.
//!
//! Pressure is an assessment of one Deadline against the declared inputs. It
//! never allocates shared Windows, changes effort, completes work, or invents
//! a cutoff from an undated task.

use crate::{
    domain::{Deadline, DeadlineWork, Effort, EvaluationInput, Presence, TaskStatus, WorkMetadata},
    fit, fulfillment, lifecycle, opportunity,
    reasons::{
        RatioPayload, Reason, ReasonCode, TaskCompletionPayload, UnavailableField,
        UnavailablePayload, ZeroOpportunityPayload, ZeroWorkBasis, ZeroWorkPayload,
    },
    results::{
        DeadlinePhase, FitResult, Health, Opportunity, PressureKey, PressureRatio, PressureResult,
        Qualification, Reference, Resolution, Risk, SourceQualification, WorkTarget, Workload,
    },
    source_health::{self, DependencyNeed},
    time::{TimeError, TimedSpan, TimezoneRules},
};
use std::{collections::BTreeSet, num::NonZeroU64};

#[derive(Clone)]
struct WorkInfo {
    target: Option<WorkTarget>,
    metadata: Option<WorkMetadata>,
    workload: Option<Workload>,
    work_ms: Option<u64>,
    reasons: Vec<Reason>,
}

/// Derive one proof-v1 pressure result per present Deadline.
pub fn derive(input: &EvaluationInput) -> Result<Vec<PressureResult>, TimeError> {
    let primitive = opportunity::derive(input)?;
    let fits = fit::derive_from_opportunity(input, &primitive)?;
    let opportunities = fit::opportunities(input, &primitive.windows)?;
    derive_with_rows(input, &primitive.windows, &fits, &opportunities)
}

/// Descriptive alias for callers assembling the core proof directly.
pub fn evaluate(input: &EvaluationInput) -> Result<Vec<PressureResult>, TimeError> {
    derive(input)
}

fn derive_with_rows(
    input: &EvaluationInput,
    windows: &[crate::results::Window],
    fits: &[FitResult],
    opportunities: &std::collections::BTreeMap<WorkTarget, Opportunity>,
) -> Result<Vec<PressureResult>, TimeError> {
    let rules = TimezoneRules::bundled();
    let mut deadlines: Vec<&Deadline> = input
        .deadlines
        .iter()
        .filter(|deadline| deadline.presence == Presence::Present)
        .collect();
    deadlines.sort_by_key(|deadline| {
        (
            deadline
                .cutoff
                .endpoint(&rules)
                .unwrap_or(input.evaluation.evaluation_end),
            deadline.meta.id,
        )
    });

    let mut output = Vec::with_capacity(deadlines.len());
    for deadline in deadlines {
        output.push(pressure_for_deadline(
            input,
            deadline,
            windows,
            fits,
            opportunities,
            &rules,
        )?);
    }
    Ok(output)
}

fn pressure_for_deadline(
    input: &EvaluationInput,
    deadline: &Deadline,
    windows: &[crate::results::Window],
    fits: &[FitResult],
    opportunities: &std::collections::BTreeMap<WorkTarget, Opportunity>,
    rules: &TimezoneRules,
) -> Result<PressureResult, TimeError> {
    let state = lifecycle::deadline_state(deadline, &input.task_refs, input.evaluation.now, rules)?;
    let endpoint = state.endpoint;
    let work_target = target_for_deadline(input, deadline);
    let preliminary_work = if state.resolution == Resolution::Unresolved
        && state.phase != DeadlinePhase::Overdue
        && endpoint <= input.evaluation.evaluation_end
    {
        Some(work_info(input, deadline)?)
    } else {
        None
    };
    let anchor_interval = capacity_interval(input, &state, preliminary_work.as_ref());
    let needs = source_health::dependencies_for_deadline(
        input,
        deadline,
        endpoint,
        anchor_interval.as_ref(),
    )?;
    let source_qualifications = qualify_all(input, needs)?;
    let qualification = qualify_result(&source_qualifications);

    let mut reasons = state.reasons.clone();
    append_source_reasons(&mut reasons, &source_qualifications);
    let limitations = BTreeSet::new();

    let mut result = PressureResult {
        evaluation_key: input.evaluation.key(input.snapshot_revision),
        deadline_id: deadline.meta.id,
        work_target,
        cutoff: deadline.cutoff.clone(),
        endpoint,
        resolution: state.resolution,
        presence: deadline.presence,
        phase: state.phase,
        workload: None,
        opportunity: None,
        ratio: None,
        risk: Risk::Unknown,
        qualification,
        source_qualifications,
        reasons,
        limitations,
    };

    match state.resolution {
        Resolution::Satisfied | Resolution::Cancelled => {
            result.risk = Risk::NotApplicable;
            finish_reasons(&mut result.reasons);
            return Ok(result);
        }
        Resolution::Unknown => {
            result.risk = Risk::Unknown;
            finish_reasons(&mut result.reasons);
            return Ok(result);
        }
        Resolution::Unresolved => {}
    }

    if state.phase == DeadlinePhase::Overdue {
        result.risk = Risk::Overdue;
        finish_reasons(&mut result.reasons);
        return Ok(result);
    }
    if endpoint > input.evaluation.evaluation_end {
        result.risk = Risk::Unknown;
        result.reasons.push(Reason::OutsideEvaluationRange {
            references: vec![Reference::Deadline(deadline.meta.id)],
            payload: crate::reasons::OutsideRangePayload {
                endpoint,
                evaluation_end: input.evaluation.evaluation_end,
            },
        });
        finish_reasons(&mut result.reasons);
        return Ok(result);
    }

    let work = preliminary_work.expect("in-range unresolved deadlines have work information");
    result.workload = work.workload;
    result.reasons.extend(work.reasons);
    let Some(work_ms) = work.work_ms else {
        result.risk = Risk::Unknown;
        finish_reasons(&mut result.reasons);
        return Ok(result);
    };
    if work_ms == 0 {
        result.risk = Risk::NoKnownWork;
        finish_reasons(&mut result.reasons);
        return Ok(result);
    }

    let Some(target) = work.target else {
        result.risk = Risk::Unknown;
        result.reasons.push(Reason::WorkUnavailable {
            references: vec![Reference::Deadline(deadline.meta.id)],
            payload: UnavailablePayload {
                field: UnavailableField::Work,
            },
        });
        finish_reasons(&mut result.reasons);
        return Ok(result);
    };
    let math_zero = work
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.earliest_start)
        .map_or(input.evaluation.now, |start| {
            input.evaluation.now.max(start)
        })
        >= endpoint;

    let opportunity = if math_zero {
        result.reasons.push(Reason::ZeroOpportunity {
            references: work_references(deadline.meta.id, Some(target)),
            payload: ZeroOpportunityPayload {
                now: input.evaluation.now,
                endpoint,
                earliest_start: work
                    .metadata
                    .as_ref()
                    .and_then(|metadata| metadata.earliest_start),
            },
        });
        Opportunity::Known {
            milliseconds: 0,
            known_qualifying_ms: 0,
            window_keys: Vec::new(),
        }
    } else {
        opportunities.get(&target).cloned().unwrap_or_else(|| {
            if windows.is_empty() {
                Opportunity::Unknown {
                    known_qualifying_ms: 0,
                    window_keys: Vec::new(),
                }
            } else {
                Opportunity::Known {
                    milliseconds: 0,
                    known_qualifying_ms: 0,
                    window_keys: Vec::new(),
                }
            }
        })
    };
    result.opportunity = Some(opportunity.clone());
    result
        .limitations
        .insert(ReasonCode::IndividualCapacityOnly);
    result.reasons.push(Reason::IndividualCapacityOnly {
        references: work_references(deadline.meta.id, Some(target)),
        payload: crate::reasons::EmptyPayload {},
    });
    append_fit_reasons(&mut result.reasons, fits, target);

    match opportunity {
        Opportunity::Unknown {
            known_qualifying_ms,
            ..
        } => {
            if windows.is_empty() {
                result.reasons.push(Reason::AvailabilityUnknown {
                    references: work_references(deadline.meta.id, Some(target)),
                    payload: UnavailablePayload {
                        field: UnavailableField::Availability,
                    },
                });
            }
            let _ = known_qualifying_ms;
            result.risk = Risk::Unknown;
        }
        Opportunity::Known {
            milliseconds,
            known_qualifying_ms,
            ..
        } => {
            debug_assert_eq!(milliseconds, known_qualifying_ms);
            if milliseconds == 0 {
                result.reasons.push(Reason::ZeroOpportunity {
                    references: work_references(deadline.meta.id, Some(target)),
                    payload: ZeroOpportunityPayload {
                        now: input.evaluation.now,
                        endpoint,
                        earliest_start: work
                            .metadata
                            .as_ref()
                            .and_then(|metadata| metadata.earliest_start),
                    },
                });
                result.risk = Risk::Insufficient;
            } else {
                let opportunity_ms = NonZeroU64::new(milliseconds)
                    .expect("positive known opportunity checked before ratio");
                let risk = ratio_risk(work_ms, opportunity_ms.get());
                result.ratio = Some(PressureRatio {
                    work_ms,
                    opportunity_ms,
                });
                result.reasons.push(Reason::PressureRatio {
                    references: vec![
                        Reference::Deadline(deadline.meta.id),
                        Reference::Pressure(PressureKey {
                            evaluation_key: result.evaluation_key.clone(),
                            deadline_id: deadline.meta.id,
                        }),
                    ],
                    payload: RatioPayload {
                        work_ms,
                        opportunity_ms: opportunity_ms.get(),
                        risk,
                    },
                });
                result.risk = risk;
            }
        }
    }
    finish_reasons(&mut result.reasons);
    Ok(result)
}

fn target_for_deadline(input: &EvaluationInput, deadline: &Deadline) -> Option<WorkTarget> {
    match fulfillment::deadline_work(input, deadline) {
        DeadlineWork::Unspecified => None,
        DeadlineWork::Standalone(_) => Some(WorkTarget::Deadline(deadline.meta.id)),
        DeadlineWork::Task(task_id) => Some(WorkTarget::Task(task_id)),
    }
}

fn work_info(input: &EvaluationInput, deadline: &Deadline) -> Result<WorkInfo, TimeError> {
    let target = target_for_deadline(input, deadline);
    let mut reasons = Vec::new();
    let (metadata, task_status, task_id) = match fulfillment::deadline_work(input, deadline) {
        DeadlineWork::Unspecified => {
            reasons.push(Reason::WorkUnavailable {
                references: vec![Reference::Deadline(deadline.meta.id)],
                payload: UnavailablePayload {
                    field: UnavailableField::Work,
                },
            });
            return Ok(WorkInfo {
                target,
                metadata: None,
                workload: Some(Workload::Unknown),
                work_ms: None,
                reasons,
            });
        }
        DeadlineWork::Standalone(work) => (Some(work), None, None),
        DeadlineWork::Task(task_id) => {
            let task = input.task_refs.iter().find(|task| task.meta.id == task_id);
            let metadata = input
                .task_annotations
                .iter()
                .find(|annotation| annotation.target_id == task_id)
                .map(|annotation| annotation.work.clone())
                .unwrap_or_default();
            (
                Some(metadata),
                task.map(|task| (task.presence, task.status)),
                Some(task_id),
            )
        }
    };

    let metadata = metadata.expect("all non-unspecified work variants carry metadata");
    if let Some((presence, status)) = task_status {
        let task_id = task_id.expect("task work carries an ID");
        if presence == Presence::Removed {
            reasons.push(Reason::WorkUnavailable {
                references: vec![
                    Reference::Deadline(deadline.meta.id),
                    Reference::Task(task_id),
                ],
                payload: UnavailablePayload {
                    field: UnavailableField::Work,
                },
            });
            return Ok(WorkInfo {
                target,
                metadata: Some(metadata),
                workload: Some(Workload::Unknown),
                work_ms: None,
                reasons,
            });
        }
        if status == TaskStatus::Done {
            let task = input.task_refs.iter().find(|task| task.meta.id == task_id);
            reasons.push(Reason::TaskCompletionBasis {
                references: vec![
                    Reference::Deadline(deadline.meta.id),
                    Reference::Task(task_id),
                ],
                payload: TaskCompletionPayload {
                    task_status: status,
                    source_completed_at: task.and_then(|task| task.source_completed_at),
                },
            });
            reasons.push(Reason::ZeroWorkUnresolved {
                references: vec![
                    Reference::Deadline(deadline.meta.id),
                    Reference::Task(task_id),
                ],
                payload: ZeroWorkPayload {
                    basis: ZeroWorkBasis::TaskCompletion,
                },
            });
            return Ok(WorkInfo {
                target,
                metadata: Some(metadata),
                workload: Some(Workload::Estimate(0)),
                work_ms: Some(0),
                reasons,
            });
        }
        if status == TaskStatus::Unknown {
            reasons.push(Reason::WorkUnavailable {
                references: vec![
                    Reference::Deadline(deadline.meta.id),
                    Reference::Task(task_id),
                ],
                payload: UnavailablePayload {
                    field: UnavailableField::Status,
                },
            });
            return Ok(WorkInfo {
                target,
                metadata: Some(metadata),
                workload: Some(Workload::Unknown),
                work_ms: None,
                reasons,
            });
        }
    }

    let (workload, work_ms) = match metadata.effort {
        Effort::Unknown => {
            reasons.push(Reason::EffortUnknown {
                references: work_references(deadline.meta.id, target),
                payload: UnavailablePayload {
                    field: UnavailableField::Effort,
                },
            });
            (Workload::Unknown, None)
        }
        Effort::Estimate(minutes) => {
            let work_ms = minutes
                .checked_mul(60_000)
                .map(u64::from)
                .ok_or(TimeError::ArithmeticOverflow)?;
            let zero_basis = (minutes == 0).then_some(ZeroWorkBasis::UserEstimate);
            if let Some(basis) = zero_basis {
                reasons.push(Reason::ZeroWorkUnresolved {
                    references: work_references(deadline.meta.id, target),
                    payload: ZeroWorkPayload { basis },
                });
            }
            (Workload::Estimate(minutes), Some(work_ms))
        }
        Effort::AtLeast(minutes) => {
            reasons.push(Reason::EffortLowerBound {
                references: work_references(deadline.meta.id, target),
                payload: crate::reasons::LowerBoundPayload {
                    minutes: minutes.get(),
                },
            });
            let work_ms = u64::from(minutes.get())
                .checked_mul(60_000)
                .ok_or(TimeError::ArithmeticOverflow)?;
            (Workload::AtLeast(minutes), Some(work_ms))
        }
    };
    Ok(WorkInfo {
        target,
        metadata: Some(metadata),
        workload: Some(workload),
        work_ms,
        reasons,
    })
}

fn capacity_interval(
    input: &EvaluationInput,
    state: &crate::results::DeadlineStateRow,
    work: Option<&WorkInfo>,
) -> Option<TimedSpan> {
    if state.resolution != Resolution::Unresolved
        || state.phase == DeadlinePhase::Overdue
        || state.endpoint <= input.evaluation.now
        || state.endpoint > input.evaluation.evaluation_end
    {
        return None;
    }
    let work = work?;
    if !work.work_ms.is_some_and(|work_ms| work_ms > 0) {
        return None;
    }
    if work
        .metadata
        .as_ref()
        .and_then(|metadata| metadata.earliest_start)
        .map_or(input.evaluation.now, |start| {
            input.evaluation.now.max(start)
        })
        >= state.endpoint
    {
        return None;
    }
    TimedSpan::new(input.evaluation.now, state.endpoint).ok()
}

fn qualify_all(
    input: &EvaluationInput,
    needs: Vec<DependencyNeed>,
) -> Result<Vec<SourceQualification>, TimeError> {
    let mut qualifications = needs
        .iter()
        .map(|need| source_health::qualify(input, need))
        .collect::<Result<Vec<_>, _>>()?;
    qualifications.sort_by_key(|qualification| (qualification.source_id, qualification.role));
    Ok(qualifications)
}

fn qualify_result(qualifications: &[SourceQualification]) -> Qualification {
    if qualifications
        .iter()
        .all(|qualification| qualification.health == Health::Healthy && qualification.covered)
    {
        Qualification::DeclaredInputs
    } else {
        Qualification::Conditional
    }
}

fn append_source_reasons(reasons: &mut Vec<Reason>, qualifications: &[SourceQualification]) {
    for qualification in qualifications {
        reasons.extend(qualification.reasons.clone());
    }
}

fn append_fit_reasons(reasons: &mut Vec<Reason>, fits: &[FitResult], target: WorkTarget) {
    for fit in fits.iter().filter(|fit| fit.target == target) {
        reasons.extend(fit.reasons.clone());
    }
}

fn target_reference(target: WorkTarget) -> Reference {
    match target {
        WorkTarget::Deadline(id) => Reference::Deadline(id),
        WorkTarget::Intention(id) => Reference::Intention(id),
        WorkTarget::RoutineOccurrence(key) => Reference::RoutineOccurrence(key),
        WorkTarget::Task(id) => Reference::Task(id),
    }
}

fn work_references(
    deadline_id: crate::domain::DeadlineId,
    target: Option<WorkTarget>,
) -> Vec<Reference> {
    match target {
        Some(WorkTarget::Deadline(id)) if id == deadline_id => {
            vec![Reference::Deadline(deadline_id)]
        }
        Some(target) => vec![Reference::Deadline(deadline_id), target_reference(target)],
        None => vec![Reference::Deadline(deadline_id)],
    }
}

fn ratio_risk(work_ms: u64, opportunity_ms: u64) -> Risk {
    let work = u128::from(work_ms);
    let opportunity = u128::from(opportunity_ms);
    if work * 2 < opportunity {
        Risk::Room
    } else if work <= opportunity {
        Risk::Tight
    } else {
        Risk::Insufficient
    }
}

fn finish_reasons(reasons: &mut Vec<Reason>) {
    reasons.sort_by_key(|reason| reason.code().as_str());
    reasons.dedup();
}
