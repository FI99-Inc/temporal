//! Deterministic work-target × Window compatibility evaluation.

use crate::{
    domain::{
        DeadlineWork, DeclaredContexts, Effort, EnergyCapacity, EnergyRequirement, EvaluationInput,
        IntentionState, Presence, TaskStatus, WorkMetadata,
    },
    fulfillment,
    reasons::{
        ChunkPayload, ContextPayload, EarliestStartPayload, Reason, UnavailableField,
        UnavailablePayload, ZeroOpportunityPayload,
    },
    recurrence,
    results::{
        FitResult, FitStatus, Opportunity, OpportunityOutput, Reference, Window, WorkTarget,
    },
    time::{Instant, TimeError, TimedSpan, TimezoneRules},
};
use std::collections::BTreeMap;

#[derive(Clone)]
struct TargetSpec {
    target: WorkTarget,
    work: WorkMetadata,
    endpoint: Instant,
    preferred_span: Option<TimedSpan>,
}

/// Derive every eligible target × Window fit row. A row is retained even when
/// its permissible intersection is empty, making exclusion explainable.
pub fn derive(input: &EvaluationInput, windows: &[Window]) -> Result<Vec<FitResult>, TimeError> {
    let targets = target_specs(input)?;
    let mut rows = Vec::new();
    let mut sorted_windows = windows.to_vec();
    sorted_windows.sort_by_key(|window| window.key.clone());
    for target in targets {
        for window in &sorted_windows {
            rows.push(fit_one(input, &target, window)?);
        }
    }
    rows.sort_by_key(|row| (row.target, row.window_key.clone()));
    Ok(rows)
}

/// Convenience entry point for the complete primitive opportunity result.
pub fn derive_from_opportunity(
    input: &EvaluationInput,
    opportunity: &OpportunityOutput,
) -> Result<Vec<FitResult>, TimeError> {
    derive(input, &opportunity.windows)
}

pub fn fit(input: &EvaluationInput, windows: &[Window]) -> Result<Vec<FitResult>, TimeError> {
    derive(input, windows)
}

/// Aggregate the fit matrix into one individual-capacity result per eligible
/// target. An unknown row never contributes to the known subtotal, but it
/// keeps the target's capacity unknown and remains referenced by its Window
/// key. This is not a shared-capacity allocation.
pub fn opportunities(
    input: &EvaluationInput,
    windows: &[Window],
) -> Result<BTreeMap<WorkTarget, Opportunity>, TimeError> {
    let rows = derive(input, windows)?;
    let targets = work_targets(input)?;
    let mut output = BTreeMap::new();
    for target in targets {
        let target_rows: Vec<&FitResult> = rows.iter().filter(|row| row.target == target).collect();
        let mut known_qualifying_ms = 0_u64;
        let mut unknown = windows.is_empty();
        let mut window_keys = Vec::new();
        for row in target_rows {
            match row.status {
                FitStatus::Fits => {
                    known_qualifying_ms = known_qualifying_ms
                        .checked_add(row.span.as_ref().map_or(0, TimedSpan::duration_ms))
                        .ok_or(TimeError::ArithmeticOverflow)?;
                    window_keys.push(row.window_key.clone());
                }
                FitStatus::Unknown => {
                    unknown = true;
                    window_keys.push(row.window_key.clone());
                }
                FitStatus::DoesNotFit => {}
            }
        }
        window_keys.sort();
        window_keys.dedup();
        let opportunity = if unknown {
            Opportunity::Unknown {
                known_qualifying_ms,
                window_keys,
            }
        } else {
            Opportunity::Known {
                milliseconds: known_qualifying_ms,
                known_qualifying_ms,
                window_keys,
            }
        };
        output.insert(target, opportunity);
    }
    Ok(output)
}

/// Return the canonical eligible target inventory without deriving fit rows.
pub fn work_targets(input: &EvaluationInput) -> Result<Vec<WorkTarget>, TimeError> {
    Ok(target_specs(input)?
        .into_iter()
        .map(|target| target.target)
        .collect())
}

fn target_specs(input: &EvaluationInput) -> Result<Vec<TargetSpec>, TimeError> {
    let rules = TimezoneRules::bundled();
    let mut targets = Vec::new();

    let mut tasks = input.task_refs.iter().collect::<Vec<_>>();
    tasks.sort_by_key(|task| task.meta.id);
    for task in tasks {
        if task.presence != Presence::Present
            || !matches!(
                task.status,
                TaskStatus::Now | TaskStatus::Later | TaskStatus::Someday
            )
        {
            continue;
        }
        let work = input
            .task_annotations
            .iter()
            .find(|annotation| annotation.target_id == task.meta.id)
            .map(|annotation| annotation.work.clone())
            .unwrap_or_default();
        if !eligible_effort(work.effort) {
            continue;
        }
        targets.push(TargetSpec {
            target: WorkTarget::Task(task.meta.id),
            work,
            endpoint: task_endpoint(input, task.meta.id, &rules)?,
            preferred_span: None,
        });
    }

    let mut deadlines = input.deadlines.iter().collect::<Vec<_>>();
    deadlines.sort_by_key(|deadline| deadline.meta.id);
    for deadline in deadlines {
        if deadline.presence != Presence::Present {
            continue;
        }
        let fulfillment = fulfillment::resolve(deadline, &input.task_refs);
        if fulfillment.resolution != crate::results::Resolution::Unresolved {
            continue;
        }
        let DeadlineWork::Standalone(work) = fulfillment::deadline_work(input, deadline) else {
            continue;
        };
        if !eligible_effort(work.effort) {
            continue;
        }
        targets.push(TargetSpec {
            target: WorkTarget::Deadline(deadline.meta.id),
            work,
            endpoint: deadline.cutoff.endpoint(&rules)?,
            preferred_span: None,
        });
    }

    let mut intentions = input.intentions.iter().collect::<Vec<_>>();
    intentions.sort_by_key(|intention| intention.meta.id);
    for intention in intentions {
        if intention.presence != Presence::Present || intention.state != IntentionState::Active {
            continue;
        }
        let work = intention.work.clone().unwrap_or_default();
        if !eligible_effort(work.effort) {
            continue;
        }
        targets.push(TargetSpec {
            target: WorkTarget::Intention(intention.meta.id),
            work,
            endpoint: input.evaluation.evaluation_end,
            // A preferred Intention span is a soft signal. It never clips
            // usable opportunity; only Routine occurrence dates are hard
            // bounded here.
            preferred_span: None,
        });
    }

    let mut routines = input.routines.iter().collect::<Vec<_>>();
    routines.sort_by_key(|routine| routine.meta.id);
    let occurrences = recurrence::eligible_occurrences(input)?;
    for occurrence in occurrences {
        let Some(routine) = routines
            .iter()
            .find(|routine| routine.meta.id == occurrence.key.routine_id)
        else {
            continue;
        };
        if !eligible_effort(routine.work.effort) {
            continue;
        }
        let span = occurrence.span.resolve(&rules)?;
        targets.push(TargetSpec {
            target: WorkTarget::RoutineOccurrence(occurrence.key),
            work: routine.work.clone(),
            endpoint: input.evaluation.evaluation_end.min(span.end()),
            preferred_span: Some(span),
        });
    }

    targets.sort_by_key(|target| target.target);
    Ok(targets)
}

fn eligible_effort(effort: Effort) -> bool {
    !matches!(effort, Effort::Estimate(0))
}

fn task_endpoint(
    input: &EvaluationInput,
    task_id: crate::domain::TaskRefId,
    rules: &TimezoneRules,
) -> Result<Instant, TimeError> {
    let mut endpoint = input.evaluation.evaluation_end;
    for deadline in input.deadlines.iter().filter(|deadline| {
        deadline.presence == Presence::Present
            && matches!(
                fulfillment::resolve(deadline, &input.task_refs).resolution,
                crate::results::Resolution::Unresolved | crate::results::Resolution::Unknown
            )
            && matches!(
                fulfillment::deadline_work(input, deadline),
                DeadlineWork::Task(id) if id == task_id
            )
    }) {
        endpoint = endpoint.min(deadline.cutoff.endpoint(rules)?);
    }
    Ok(endpoint)
}

fn fit_one(
    input: &EvaluationInput,
    target: &TargetSpec,
    window: &Window,
) -> Result<FitResult, TimeError> {
    let mut reasons = Vec::new();
    let target_reference = target_reference(target.target);
    let window_reference = Reference::Window(window.key.clone());
    let earliest = target.work.earliest_start;
    let mut lower = window.span.start().max(input.evaluation.now);
    if let Some(start) = earliest {
        if start > window.span.start() {
            reasons.push(Reason::BeforeEarliestStart {
                references: vec![target_reference.clone(), window_reference.clone()],
                payload: EarliestStartPayload {
                    earliest_start: start,
                    window_span: window.span.clone(),
                },
            });
        }
        lower = lower.max(start);
    }
    if let Some(preferred) = &target.preferred_span {
        lower = lower.max(preferred.start());
    }
    let upper = target.endpoint.min(window.span.end()).min(
        target
            .preferred_span
            .as_ref()
            .map_or(target.endpoint, TimedSpan::end),
    );

    if lower >= target.endpoint || lower >= upper {
        reasons.push(Reason::ZeroOpportunity {
            references: vec![target_reference.clone(), window_reference.clone()],
            payload: ZeroOpportunityPayload {
                now: input.evaluation.now,
                endpoint: target.endpoint,
                earliest_start: earliest,
            },
        });
        reasons.sort_by_key(|reason| reason.code().as_str());
        return Ok(FitResult {
            target: target.target,
            window_key: window.key.clone(),
            span: None,
            status: FitStatus::DoesNotFit,
            reasons,
        });
    }

    let span = TimedSpan::new(lower, upper)?;
    let mut definite = false;
    let mut uncertain = false;

    if !target.work.required_contexts.is_empty() {
        match &window.contexts {
            DeclaredContexts::Known(declared)
                if !target.work.required_contexts.is_subset(declared) =>
            {
                definite = true;
                reasons.push(Reason::ContextMismatch {
                    references: vec![target_reference.clone(), window_reference.clone()],
                    payload: ContextPayload {
                        required_contexts: target.work.required_contexts.clone(),
                        declared_contexts: window.contexts.clone(),
                    },
                });
            }
            DeclaredContexts::Unknown => {
                uncertain = true;
                reasons.push(Reason::ContextUnknown {
                    references: vec![target_reference.clone(), window_reference.clone()],
                    payload: ContextPayload {
                        required_contexts: target.work.required_contexts.clone(),
                        declared_contexts: window.contexts.clone(),
                    },
                });
            }
            DeclaredContexts::Known(_) => {}
        }
    }

    match energy_fit(target.work.energy_requirement, window.energy_capacity) {
        EnergyFit::Mismatch => {
            definite = true;
            reasons.push(Reason::EnergyMismatch {
                references: vec![target_reference.clone(), window_reference.clone()],
                payload: crate::reasons::EnergyPayload {
                    energy_requirement: target.work.energy_requirement,
                    energy_capacity: window.energy_capacity,
                },
            });
        }
        EnergyFit::Unknown => {
            uncertain = true;
            reasons.push(Reason::EnergyUnknown {
                references: vec![target_reference.clone(), window_reference.clone()],
                payload: crate::reasons::EnergyPayload {
                    energy_requirement: target.work.energy_requirement,
                    energy_capacity: window.energy_capacity,
                },
            });
        }
        EnergyFit::Fits => {}
    }

    match target.work.minimum_chunk_minutes {
        Some(minutes) => {
            let chunk_ms = u64::from(minutes.get())
                .checked_mul(60_000)
                .ok_or(TimeError::ArithmeticOverflow)?;
            if span.duration_ms() < chunk_ms {
                definite = true;
                reasons.push(Reason::ChunkTooShort {
                    references: vec![target_reference.clone(), window_reference.clone()],
                    payload: ChunkPayload {
                        available_ms: span.duration_ms(),
                        chunk_ms,
                    },
                });
            }
        }
        None => {
            uncertain = true;
            reasons.push(Reason::ChunkUnknown {
                references: vec![target_reference.clone(), window_reference.clone()],
                payload: UnavailablePayload {
                    field: UnavailableField::MinimumChunkMinutes,
                },
            });
        }
    }

    let status = if definite {
        FitStatus::DoesNotFit
    } else if uncertain {
        FitStatus::Unknown
    } else {
        FitStatus::Fits
    };
    reasons.sort_by_key(|reason| reason.code().as_str());
    Ok(FitResult {
        target: target.target,
        window_key: window.key.clone(),
        span: Some(span),
        status,
        reasons,
    })
}

#[derive(Clone, Copy)]
enum EnergyFit {
    Fits,
    Mismatch,
    Unknown,
}

fn energy_fit(requirement: EnergyRequirement, capacity: EnergyCapacity) -> EnergyFit {
    let required = match requirement {
        EnergyRequirement::Unrestricted => return EnergyFit::Fits,
        EnergyRequirement::Light => 1,
        EnergyRequirement::Normal => 2,
        EnergyRequirement::Deep => 3,
    };
    let available = match capacity {
        EnergyCapacity::Unknown => return EnergyFit::Unknown,
        EnergyCapacity::Light => 1,
        EnergyCapacity::Normal => 2,
        EnergyCapacity::Deep => 3,
    };
    if required <= available {
        EnergyFit::Fits
    } else {
        EnergyFit::Mismatch
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
