use super::*;
use crate::{
    reasons::{Reason, UnavailableField},
    results::*,
};

pub(super) fn validate(context: &mut Context<'_>) {
    let input = context.input;
    let mut keys = Vec::new();
    for (index, suggestion) in input.prior_suggestions.iter().enumerate() {
        let path = format!("prior_suggestions[{index}]");
        if keys.contains(&&suggestion.key) {
            context.issue(IssueCode::DuplicateIdentity, format!("{path}.key"));
        }
        keys.push(&suggestion.key);
        if suggestion.kind != suggestion.key.kind
            || suggestion.target != suggestion.key.target
            || suggestion.proposed_span != suggestion.key.proposed_span
        {
            context.issue(IssueCode::InvalidRelationship, format!("{path}.key"));
        }
        key(
            context,
            &suggestion.key.evaluation_key,
            &format!("{path}.key.evaluation_key"),
        );
        if suggestion.created_at != suggestion.key.evaluation_key.now
            || suggestion.created_at >= suggestion.valid_until
        {
            context.issue(IssueCode::InvalidTime, &path);
        }
        if let Some(span) = &suggestion.proposed_span {
            match span.resolve(&context.rules) {
                Ok(span)
                    if span.end() > suggestion.created_at
                        && suggestion.valid_until <= span.end() => {}
                _ => context.issue(IssueCode::InvalidTime, format!("{path}.proposed_span")),
            }
        }
        target(context, suggestion.target, &format!("{path}.target"));
        if suggestion.kind == SuggestionKind::RiskNotice
            && !matches!(suggestion.target, WorkTarget::Deadline(_))
        {
            context.issue(IssueCode::InvalidRelationship, format!("{path}.target"));
        }
        if suggestion.reasons.is_empty() {
            context.issue(IssueCode::InvalidValue, format!("{path}.reasons"));
        }
        let mut pressure_reference = false;
        for (reason_index, reason) in suggestion.reasons.iter().enumerate() {
            let reason_path = format!("{path}.reasons[{reason_index}]");
            if reason.references().is_empty() {
                context.issue(IssueCode::InvalidValue, format!("{reason_path}.references"));
            }
            for (ref_index, reference) in reason.references().iter().enumerate() {
                let ref_path = format!("{reason_path}.references[{ref_index}]");
                match reference {
                    Reference::Anchor(id) => {
                        context.reference(RecordId::Anchor(*id), ref_path);
                    }
                    Reference::Availability(id) => {
                        context.reference(RecordId::Availability(*id), ref_path);
                    }
                    Reference::Deadline(id) => {
                        context.reference(RecordId::Deadline(*id), ref_path);
                    }
                    Reference::Intention(id) => {
                        context.reference(RecordId::Intention(*id), ref_path);
                    }
                    Reference::Routine(id) => {
                        context.reference(RecordId::Routine(*id), ref_path);
                    }
                    Reference::RoutineOccurrence(occurrence) => {
                        context.reference(RecordId::Routine(occurrence.routine_id), ref_path);
                    }
                    Reference::Source(id) => {
                        context.reference(RecordId::Source(*id), ref_path);
                    }
                    Reference::Task(id) => {
                        context.reference(RecordId::TaskRef(*id), ref_path);
                    }
                    Reference::Window(window) => {
                        key(context, &window.evaluation_key, &ref_path);
                        if window.start >= window.end
                            || window.start < window.evaluation_key.now
                            || window.end > window.evaluation_key.evaluation_end
                        {
                            context.issue(IssueCode::InvalidTime, &ref_path);
                        }
                        // The producing Window need not exist in today's derived output.
                        if window.evaluation_key != suggestion.key.evaluation_key {
                            context.issue(IssueCode::InvalidRelationship, &ref_path);
                        }
                    }
                    Reference::Pressure(pressure) => {
                        context.reference(RecordId::Deadline(pressure.deadline_id), &ref_path);
                        key(context, &pressure.evaluation_key, &ref_path);
                        if pressure.evaluation_key != suggestion.key.evaluation_key {
                            context.issue(IssueCode::InvalidRelationship, &ref_path);
                        }
                        if suggestion.target == WorkTarget::Deadline(pressure.deadline_id) {
                            pressure_reference = true;
                        }
                    }
                }
            }
            payload(context, reason, &reason_path);
        }
        if suggestion.kind == SuggestionKind::RiskNotice && !pressure_reference {
            context.issue(IssueCode::InvalidRelationship, format!("{path}.reasons"));
        }
        // Completion/removal/current fit are intentionally not checked here.
        // A prior record remains inspectable until Task 1.9 assesses validity.
    }
}

fn key(context: &mut Context<'_>, key: &EvaluationKey, path: &str) {
    if key.now >= key.evaluation_end {
        context.issue(IssueCode::InvalidTime, path);
    }
    if key.timezone_rules_version.is_empty() {
        context.issue(IssueCode::InvalidValue, path);
    }
}

fn target(context: &mut Context<'_>, target: WorkTarget, path: &str) {
    let id = match target {
        WorkTarget::Deadline(id) => RecordId::Deadline(id),
        WorkTarget::Intention(id) => RecordId::Intention(id),
        WorkTarget::RoutineOccurrence(key) => RecordId::Routine(key.routine_id),
        WorkTarget::Task(id) => RecordId::TaskRef(id),
    };
    context.reference(id, path);
}

fn payload(context: &mut Context<'_>, reason: &Reason, path: &str) {
    let valid = match reason {
        Reason::ResolutionRecorded { payload, .. } => {
            matches!(
                payload.resolution,
                Resolution::Satisfied | Resolution::Cancelled
            ) == payload.evidence.is_some()
        }
        Reason::AnchorBlocked { payload, .. } => {
            payload.blocked_ms == payload.intersection.duration_ms()
                && payload.intersection.start() >= payload.anchor_span.start()
                && payload.intersection.end() <= payload.anchor_span.end()
        }
        Reason::DeclaredAvailability { payload, .. } => {
            payload.clipped_span.as_ref().is_none_or(|clipped| {
                clipped.start() >= payload.span.start() && clipped.end() <= payload.span.end()
            })
        }
        Reason::ChunkTooShort { payload, .. } => {
            payload.chunk_ms > 0 && payload.available_ms < payload.chunk_ms
        }
        Reason::EffortLowerBound { payload, .. } => payload.minutes > 0,
        Reason::PressureRatio { payload, .. } => {
            payload.work_ms > 0
                && payload.opportunity_ms > 0
                && matches!(payload.risk, Risk::Room | Risk::Tight | Risk::Insufficient)
        }
        Reason::EffortUnknown { payload, .. } => payload.field == UnavailableField::Effort,
        Reason::ChunkUnknown { payload, .. } => {
            payload.field == UnavailableField::MinimumChunkMinutes
        }
        Reason::AvailabilityUnknown { payload, .. } => {
            payload.field == UnavailableField::Availability
        }
        Reason::WorkUnavailable { payload, .. } => matches!(
            payload.field,
            UnavailableField::Work | UnavailableField::Status
        ),
        Reason::FulfillmentUnknown { payload, .. } => matches!(
            payload.field,
            UnavailableField::Fulfillment | UnavailableField::Status
        ),
        Reason::SourceCoverage { payload, .. } => match payload.role {
            SourceRole::Anchors => {
                payload.required_endpoint.is_none() && payload.tasks_complete.is_none()
            }
            SourceRole::Deadlines => {
                payload.required_endpoint.is_some()
                    && payload.required_interval.is_none()
                    && payload.tasks_complete.is_none()
            }
            SourceRole::Tasks => {
                payload.tasks_complete.is_some()
                    && payload.required_endpoint.is_none()
                    && payload.required_interval.is_none()
                    && payload.known_interval.is_none()
            }
        },
        Reason::OutsideEvaluationRange { payload, .. } => payload.endpoint > payload.evaluation_end,
        Reason::SuggestionInvalidated { payload, .. } => {
            payload.previous_basis != payload.current_basis
        }
        Reason::SoftPreferencePassed { payload, .. }
        | Reason::RoutinePastUnrecorded { payload, .. }
        | Reason::SuggestionExpired { payload, .. } => payload.now >= payload.boundary,
        _ => true,
    };
    if !valid {
        context.issue(IssueCode::InvalidValue, format!("{path}.payload"));
    }
}
