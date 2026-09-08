use super::*;
use crate::{fulfillment, results::Resolution};

pub(super) fn validate(context: &mut Context<'_>) {
    let input = context.input;
    let mut annotation_ids = BTreeSet::new();
    for (index, annotation) in input.anchor_annotations.iter().enumerate() {
        let path = format!("anchor_annotations[{index}]");
        context.reference(
            RecordId::Anchor(annotation.target_id),
            format!("{path}.target_id"),
        );
        if !annotation_ids.insert(annotation.target_id) {
            context.issue(IssueCode::DuplicateIdentity, &path);
        }
        context.audit(annotation.created_at, annotation.updated_at, &path);
        if let Some(confirmation) = &annotation.confirmation {
            context.recorded(
                confirmation.confirmed_at,
                format!("{path}.confirmation.confirmed_at"),
            );
        }
    }
    let mut annotation_ids = BTreeSet::new();
    for (index, annotation) in input.deadline_annotations.iter().enumerate() {
        let path = format!("deadline_annotations[{index}]");
        context.reference(
            RecordId::Deadline(annotation.target_id),
            format!("{path}.target_id"),
        );
        if !annotation_ids.insert(annotation.target_id) {
            context.issue(IssueCode::DuplicateIdentity, &path);
        }
        context.audit(annotation.created_at, annotation.updated_at, &path);
        if let Some(confirmation) = &annotation.confirmation {
            context.recorded(
                confirmation.confirmed_at,
                format!("{path}.confirmation.confirmed_at"),
            );
        }
        match &annotation.work {
            DeadlineWork::Unspecified => {}
            DeadlineWork::Standalone(work) => context.work(work, &format!("{path}.work.value")),
            DeadlineWork::Task(id) => {
                context.reference(RecordId::TaskRef(*id), format!("{path}.work.value"));
            }
        }
        if let Some(deadline) = input
            .deadlines
            .iter()
            .find(|deadline| deadline.meta.id == annotation.target_id)
            && let Fulfillment::TraceTask(id) = deadline.fulfillment
            && annotation.work != DeadlineWork::Task(id)
        {
            context.issue(IssueCode::InvalidRelationship, format!("{path}.work"));
        }
    }
    let mut annotation_ids = BTreeSet::new();
    for (index, annotation) in input.task_annotations.iter().enumerate() {
        let path = format!("task_annotations[{index}]");
        context.reference(
            RecordId::TaskRef(annotation.target_id),
            format!("{path}.target_id"),
        );
        if !annotation_ids.insert(annotation.target_id) {
            context.issue(IssueCode::DuplicateIdentity, &path);
        }
        context.audit(annotation.created_at, annotation.updated_at, &path);
        context.work(&annotation.work, &format!("{path}.work"));
    }
    let mut outcomes = BTreeSet::new();
    for (index, outcome) in input.routine_outcomes.iter().enumerate() {
        let path = format!("routine_outcomes[{index}]");
        context.reference(
            RecordId::Routine(outcome.routine_id),
            format!("{path}.routine_id"),
        );
        if !outcomes.insert(outcome.key()) {
            context.issue(IssueCode::DuplicateIdentity, &path);
        }
        context.recorded(outcome.recorded_at, format!("{path}.recorded_at"));
        if let Some(routine) = input
            .routines
            .iter()
            .find(|routine| routine.meta.id == outcome.routine_id)
        {
            let RoutineRule::Weekly(rule) = &routine.rule;
            let resolved = outcome
                .date
                .next_day()
                .and_then(|end| crate::time::DateSpan::new(outcome.date, end, rule.zone))
                .and_then(|span| span.resolve(&context.rules));
            if resolved.is_err() {
                context.issue(IssueCode::InvalidTime, format!("{path}.date"));
            }
            // A historical outcome need not match a rule edited after it was recorded.
        }
    }

    let mut owners = BTreeMap::new();
    for (index, deadline) in input.deadlines.iter().enumerate() {
        let path = format!("deadlines[{index}]");
        match &deadline.fulfillment {
            Fulfillment::Recorded(recorded) => {
                if let Provenance::Imported(imported) = &deadline.provenance
                    && imported.projection == Projection::TaskDue
                {
                    context.issue(
                        IssueCode::InvalidRelationship,
                        format!("{path}.fulfillment"),
                    );
                }
                if let RecordedResolution::Satisfied(evidence)
                | RecordedResolution::Cancelled(evidence) = recorded
                {
                    let expected = match deadline.provenance {
                        Provenance::LocalUser(_) => EvidenceAuthority::LocalUser,
                        Provenance::Imported(_) => EvidenceAuthority::Source,
                    };
                    if evidence.authority != expected {
                        context.issue(
                            IssueCode::InvalidRelationship,
                            format!("{path}.fulfillment.value.value.authority"),
                        );
                    }
                    context.recorded(
                        evidence.recorded_at,
                        format!("{path}.fulfillment.value.value.recorded_at"),
                    );
                }
            }
            Fulfillment::TraceTask(id) => {
                context.reference(RecordId::TaskRef(*id), format!("{path}.fulfillment.value"));
                if let Some(task) = input.task_refs.iter().find(|task| task.meta.id == *id) {
                    if !matching_projection(deadline, task) {
                        context.issue(IssueCode::InvalidRelationship, format!("{path}.provenance"));
                    }
                    if deadline.presence == Presence::Present
                        && (task.presence == Presence::Removed
                            || task.due != TaskDue::Deadline(deadline.meta.id))
                    {
                        context.issue(
                            IssueCode::InvalidRelationship,
                            format!("{path}.fulfillment"),
                        );
                    }
                }
            }
        }
        if deadline.presence == Presence::Present
            && matches!(
                fulfillment::resolve(deadline, &input.task_refs).resolution,
                Resolution::Unresolved | Resolution::Unknown
            )
            && let DeadlineWork::Task(task_id) = fulfillment::deadline_work(input, deadline)
            && let Some(previous) = owners.insert(task_id, deadline.meta.id)
        {
            let mut issue = ValidationIssue::new(IssueCode::InvalidRelationship, &path);
            issue.record_id = Some(deadline.meta.id.to_string());
            issue.related_ids = vec![previous.to_string(), task_id.to_string()];
            context.issues.push(issue);
        }
    }
    for (index, task) in input.task_refs.iter().enumerate() {
        if let TaskDue::Deadline(id) = task.due {
            let path = format!("task_refs[{index}].due.value");
            context.reference(RecordId::Deadline(id), &path);
            if let Some(deadline) = input
                .deadlines
                .iter()
                .find(|deadline| deadline.meta.id == id)
                && (deadline.fulfillment != Fulfillment::TraceTask(task.meta.id)
                    || !matching_projection(deadline, task)
                    || (task.presence == Presence::Present
                        && deadline.presence == Presence::Removed))
            {
                context.issue(IssueCode::InvalidRelationship, &path);
            }
        }
    }
}

fn matching_projection(deadline: &Deadline, task: &TaskRef) -> bool {
    let Provenance::Imported(due) = &deadline.provenance else {
        return false;
    };
    let task = task.provenance.imported();
    due.projection == Projection::TaskDue
        && due.source_id == task.source_id
        && due.external_id == task.external_id
        && due.occurrence_key == task.occurrence_key
}
