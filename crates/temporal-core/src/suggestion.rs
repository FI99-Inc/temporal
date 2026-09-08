//! Revalidation of retained advice. This module never generates new advice or
//! writes completion, due dates, or any other factual/user state.

use crate::{
    domain::{DeadlineWork, EvaluationInput, Presence, SourceRole},
    fit, fulfillment, opportunity,
    reasons::{
        self, BoundaryPayload, InvalidatedPayload, Reason, UnavailableField, UnavailablePayload,
    },
    results::*,
    source_health::{self, CoverageNeed, DependencyNeed},
    time::{TimeError, TimedSpan, TimezoneRules},
};

pub(crate) fn derive(
    input: &EvaluationInput,
    windows: &[Window],
    pressures: &[PressureResult],
) -> Result<Vec<SuggestionValidity>, TimeError> {
    let mut rows = input
        .prior_suggestions
        .iter()
        .map(|suggestion| validity(input, suggestion, windows, pressures))
        .collect::<Result<Vec<_>, _>>()?;
    rows.sort_by(|left, right| crate::codec::compare_suggestion_keys(&left.key, &right.key));
    Ok(rows)
}

fn validity(
    input: &EvaluationInput,
    suggestion: &Suggestion,
    windows: &[Window],
    pressures: &[PressureResult],
) -> Result<SuggestionValidity, TimeError> {
    let previous_basis = suggestion.key.evaluation_key.basis();
    let current_basis = input.evaluation.key(input.snapshot_revision).basis();
    let reference = Reference::from(suggestion.target);
    let (status, mut reasons) = if previous_basis != current_basis {
        (
            SuggestionStatus::Invalidated,
            vec![Reason::SuggestionInvalidated {
                references: vec![reference],
                payload: InvalidatedPayload {
                    previous_basis,
                    current_basis,
                },
            }],
        )
    } else if input.evaluation.now >= suggestion.valid_until {
        (
            SuggestionStatus::Expired,
            vec![Reason::SuggestionExpired {
                references: vec![reference],
                payload: BoundaryPayload {
                    boundary: suggestion.valid_until,
                    now: input.evaluation.now,
                },
            }],
        )
    } else {
        match suggestion.kind {
            SuggestionKind::RiskNotice => {
                let row = pressures
                    .iter()
                    .find(|row| WorkTarget::Deadline(row.deadline_id) == suggestion.target);
                if let Some(row) = row {
                    let current =
                        matches!(row.resolution, Resolution::Unresolved | Resolution::Unknown)
                            && matches!(
                                row.risk,
                                Risk::Tight | Risk::Insufficient | Risk::Overdue | Risk::Unknown
                            );
                    (
                        if current {
                            SuggestionStatus::Current
                        } else {
                            SuggestionStatus::Ineligible
                        },
                        row.reasons.clone(),
                    )
                } else {
                    (SuggestionStatus::Ineligible, unavailable(suggestion.target))
                }
            }
            SuggestionKind::ConsiderWork => consider_work(input, suggestion, windows)?,
        }
    };
    reasons::normalize(&mut reasons);
    Ok(SuggestionValidity {
        key: suggestion.key.clone(),
        status,
        reasons,
    })
}

fn consider_work(
    input: &EvaluationInput,
    suggestion: &Suggestion,
    windows: &[Window],
) -> Result<(SuggestionStatus, Vec<Reason>), TimeError> {
    let proposed = suggestion
        .proposed_span
        .as_ref()
        .map(|span| span.resolve(&TimezoneRules::bundled()))
        .transpose()?;
    let Some(rows) = fit::for_suggestion(input, windows, suggestion.target, proposed.as_ref())?
    else {
        return Ok((SuggestionStatus::Ineligible, unavailable(suggestion.target)));
    };
    let current = rows.iter().any(|row| row.status == FitStatus::Fits);
    let mut reasons = Vec::new();
    for row in rows
        .iter()
        .filter(|row| !current || row.status == FitStatus::Fits)
    {
        let window = windows
            .iter()
            .find(|window| window.key == row.window_key)
            .expect("fit row retains an existing Window key");
        reasons.extend(window.reasons.clone());
        reasons.extend(row.reasons.clone());
        for qualification in &window.source_qualifications {
            reasons.extend(qualification.reasons.clone());
        }
        reasons.extend(source_reasons(
            input,
            suggestion.target,
            row.span.as_ref().unwrap_or(&window.span),
        )?);
    }
    if rows.is_empty() {
        let interval = TimedSpan::new(input.evaluation.now, input.evaluation.evaluation_end)?;
        reasons.extend(source_reasons(input, suggestion.target, &interval)?);
        reasons.extend(opportunity::reasons_for_interval(input, &interval)?);
        if input.availability.is_empty() {
            reasons.push(Reason::AvailabilityUnknown {
                references: vec![Reference::from(suggestion.target)],
                payload: UnavailablePayload {
                    field: UnavailableField::Availability,
                },
            });
        }
    }
    Ok((
        if current {
            SuggestionStatus::Current
        } else {
            SuggestionStatus::Ineligible
        },
        reasons,
    ))
}

fn unavailable(target: WorkTarget) -> Vec<Reason> {
    vec![Reason::WorkUnavailable {
        references: vec![Reference::from(target)],
        payload: UnavailablePayload {
            field: UnavailableField::Work,
        },
    }]
}

fn source_reasons(
    input: &EvaluationInput,
    target: WorkTarget,
    interval: &TimedSpan,
) -> Result<Vec<Reason>, TimeError> {
    let mut needs = source_health::dependencies_for_interval(input, interval);
    let deadline = input.deadlines.iter().find(|deadline| {
        deadline.presence == Presence::Present && match target {
            WorkTarget::Deadline(id) => deadline.meta.id == id,
            WorkTarget::Task(id) => matches!(fulfillment::resolve(deadline, &input.task_refs).resolution, Resolution::Unresolved | Resolution::Unknown)
                && matches!(fulfillment::deadline_work(input, deadline), DeadlineWork::Task(linked) if linked == id),
            _ => false,
        }
    });
    if let Some(deadline) = deadline {
        needs.extend(source_health::dependencies_for_deadline(
            input,
            deadline,
            deadline.cutoff.endpoint(&TimezoneRules::bundled())?,
            Some(interval),
        )?);
    }
    if let WorkTarget::Task(id) = target {
        let task = input
            .task_refs
            .iter()
            .find(|task| task.meta.id == id)
            .expect("validated Task reference");
        needs.push(DependencyNeed {
            source_id: task.provenance.source_id(),
            role: SourceRole::Tasks,
            coverage: CoverageNeed::TasksCatalog,
        });
    }
    let mut reasons = Vec::new();
    for need in needs {
        reasons.extend(source_health::qualify(input, &need)?.reasons);
    }
    Ok(reasons)
}
