//! Deterministic temporal phases over an already-normalized snapshot.
//!
//! This module reads facts and the explicit evaluation time. It does not
//! mutate the snapshot, infer attendance, or turn a soft preference into an
//! obligation.

use crate::{
    domain::{Deadline, EvaluationInput, Intention, TaskRef},
    fulfillment,
    reasons::{Reason, TaskCompletionPayload, UnavailableField},
    recurrence,
    results::{
        AnchorPhase, AnchorStateRow, DeadlinePhase, DeadlineStateRow, IntentionPhase,
        IntentionStateRow, LifecycleOutput, Reference, Resolution,
    },
    time::{CutoffPosition, Instant, TimeError, TimezoneRules},
};

/// Derive all lifecycle rows without evaluating health, opportunity, fit, or
/// pressure. The input is expected to have passed Task 1.4 validation.
pub fn evaluate_lifecycle(input: &EvaluationInput) -> Result<LifecycleOutput, TimeError> {
    let rules = TimezoneRules::bundled();
    let mut anchor_states = input
        .anchors
        .iter()
        .map(|anchor| anchor_state(anchor, input.evaluation.now, &rules))
        .collect::<Result<Vec<_>, _>>()?;
    anchor_states.sort_by_key(|row| row.id);

    let mut deadline_states = input
        .deadlines
        .iter()
        .map(|deadline| deadline_state(deadline, &input.task_refs, input.evaluation.now, &rules))
        .collect::<Result<Vec<_>, _>>()?;
    deadline_states.sort_by_key(|row| row.id);

    let mut intention_states = input
        .intentions
        .iter()
        .map(|intention| intention_state(intention, input.evaluation.now, &rules))
        .collect::<Result<Vec<_>, _>>()?;
    intention_states.sort_by_key(|row| row.id);

    let routine_occurrences = recurrence::expand_routines_with_rules(input, &rules)?;

    Ok(LifecycleOutput {
        anchor_states,
        deadline_states,
        intention_states,
        routine_occurrences,
    })
}

pub fn anchor_state(
    anchor: &crate::domain::Anchor,
    now: Instant,
    rules: &TimezoneRules,
) -> Result<AnchorStateRow, TimeError> {
    let phase = if anchor.presence == crate::domain::Presence::Removed {
        AnchorPhase::Inactive
    } else {
        let span = anchor.span.resolve(rules)?;
        if now < span.start() {
            AnchorPhase::Upcoming
        } else if now < span.end() {
            AnchorPhase::Ongoing
        } else {
            AnchorPhase::Passed
        }
    };
    Ok(AnchorStateRow {
        id: anchor.meta.id,
        phase,
        reasons: Vec::new(),
    })
}

pub fn deadline_state(
    deadline: &Deadline,
    tasks: &[TaskRef],
    now: Instant,
    rules: &TimezoneRules,
) -> Result<DeadlineStateRow, TimeError> {
    let endpoint = deadline.cutoff.endpoint(rules)?;
    let resolved = fulfillment::resolve(deadline, tasks);
    let phase = match (deadline.presence, resolved.resolution) {
        (crate::domain::Presence::Removed, _) => DeadlinePhase::Inactive,
        (_, Resolution::Satisfied | Resolution::Cancelled) => DeadlinePhase::Inactive,
        (_, Resolution::Unknown) => DeadlinePhase::Unknown,
        (_, Resolution::Unresolved) => match deadline.cutoff.position(now, rules)? {
            CutoffPosition::Upcoming => DeadlinePhase::Upcoming,
            CutoffPosition::DueNow => DeadlinePhase::DueNow,
            CutoffPosition::DueToday => DeadlinePhase::DueToday,
            CutoffPosition::Overdue => DeadlinePhase::Overdue,
        },
    };

    let mut reasons = vec![Reason::DeadlinePhase {
        references: vec![Reference::Deadline(deadline.meta.id)],
        payload: crate::reasons::DeadlinePhasePayload {
            cutoff: deadline.cutoff.clone(),
            endpoint,
            phase,
            now,
        },
    }];
    match resolved.resolution {
        Resolution::Satisfied | Resolution::Cancelled => {
            if resolved.evidence.is_some() {
                reasons.push(Reason::ResolutionRecorded {
                    references: vec![Reference::Deadline(deadline.meta.id)],
                    payload: crate::reasons::ResolutionPayload {
                        resolution: resolved.resolution,
                        evidence: resolved.evidence,
                    },
                });
            } else if let Some(task_id) = resolved.trace_task {
                completion_reason(&mut reasons, deadline, task_id, tasks);
            }
        }
        Resolution::Unknown => {
            let (field, references) = match resolved.trace_task {
                Some(task_id) if tasks.iter().any(|task| task.meta.id == task_id) => (
                    UnavailableField::Status,
                    vec![
                        Reference::Deadline(deadline.meta.id),
                        Reference::Task(task_id),
                    ],
                ),
                Some(task_id) => (
                    UnavailableField::Fulfillment,
                    vec![
                        Reference::Deadline(deadline.meta.id),
                        Reference::Task(task_id),
                    ],
                ),
                None => (
                    UnavailableField::Fulfillment,
                    vec![Reference::Deadline(deadline.meta.id)],
                ),
            };
            reasons.push(Reason::FulfillmentUnknown {
                references,
                payload: crate::reasons::UnavailablePayload { field },
            });
        }
        Resolution::Unresolved => {
            if let Some(task_id) = resolved.trace_task {
                completion_reason(&mut reasons, deadline, task_id, tasks);
            }
        }
    }
    sort_reasons(&mut reasons);

    Ok(DeadlineStateRow {
        id: deadline.meta.id,
        presence: deadline.presence,
        resolution: resolved.resolution,
        phase,
        cutoff: deadline.cutoff.clone(),
        endpoint,
        reasons,
    })
}

fn completion_reason(
    reasons: &mut Vec<Reason>,
    deadline: &Deadline,
    task_id: crate::domain::TaskRefId,
    tasks: &[TaskRef],
) {
    if let Some(task) = tasks.iter().find(|task| task.meta.id == task_id) {
        reasons.push(Reason::TaskCompletionBasis {
            references: vec![
                Reference::Deadline(deadline.meta.id),
                Reference::Task(task.meta.id),
            ],
            payload: TaskCompletionPayload {
                task_status: task.status,
                source_completed_at: task.source_completed_at,
            },
        });
    } else {
        reasons.push(Reason::FulfillmentUnknown {
            references: vec![
                Reference::Deadline(deadline.meta.id),
                Reference::Task(task_id),
            ],
            payload: crate::reasons::UnavailablePayload {
                field: UnavailableField::Fulfillment,
            },
        });
    }
}

pub fn intention_state(
    intention: &Intention,
    now: Instant,
    rules: &TimezoneRules,
) -> Result<IntentionStateRow, TimeError> {
    let mut reasons = Vec::new();
    let phase = if intention.presence == crate::domain::Presence::Removed
        || !matches!(intention.state, crate::domain::IntentionState::Active)
    {
        IntentionPhase::Inactive
    } else {
        match intention.preferred_span.as_ref() {
            None => IntentionPhase::NoPreference,
            Some(preferred) => {
                let span = preferred.resolve(rules)?;
                if now < span.start() {
                    IntentionPhase::PreferenceUpcoming
                } else if now < span.end() {
                    IntentionPhase::PreferredNow
                } else {
                    reasons.push(Reason::SoftPreferencePassed {
                        references: vec![Reference::Intention(intention.meta.id)],
                        payload: crate::reasons::BoundaryPayload {
                            boundary: span.end(),
                            now,
                        },
                    });
                    IntentionPhase::PreferencePassed
                }
            }
        }
    };
    sort_reasons(&mut reasons);
    Ok(IntentionStateRow {
        id: intention.meta.id,
        phase,
        reasons,
    })
}

fn sort_reasons(reasons: &mut [Reason]) {
    reasons.sort_by_key(|reason| reason.code().as_str());
}
