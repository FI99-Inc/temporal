//! Clock-independent authoritative fulfillment, shared by validation/lifecycle.

use crate::domain::{
    Deadline, Fulfillment, RecordedResolution, ResolutionEvidence, TaskRef, TaskRefId, TaskStatus,
};
use crate::results::Resolution;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedFulfillment {
    pub resolution: Resolution,
    pub evidence: Option<ResolutionEvidence>,
    pub trace_task: Option<TaskRefId>,
}

pub fn resolve(deadline: &Deadline, tasks: &[TaskRef]) -> ResolvedFulfillment {
    match &deadline.fulfillment {
        Fulfillment::Recorded(recorded) => {
            let (resolution, evidence) = match recorded {
                RecordedResolution::Unresolved => (Resolution::Unresolved, None),
                RecordedResolution::Satisfied(evidence) => {
                    (Resolution::Satisfied, Some(evidence.clone()))
                }
                RecordedResolution::Cancelled(evidence) => {
                    (Resolution::Cancelled, Some(evidence.clone()))
                }
            };
            ResolvedFulfillment {
                resolution,
                evidence,
                trace_task: None,
            }
        }
        Fulfillment::TraceTask(id) => {
            let resolution = match tasks
                .iter()
                .find(|task| task.meta.id == *id)
                .map(|task| task.status)
            {
                Some(TaskStatus::Done) => Resolution::Satisfied,
                Some(TaskStatus::Now | TaskStatus::Later | TaskStatus::Someday) => {
                    Resolution::Unresolved
                }
                Some(TaskStatus::Unknown) | None => Resolution::Unknown,
            };
            // Missing source completion time remains missing; no user evidence is invented.
            ResolvedFulfillment {
                resolution,
                evidence: None,
                trace_task: Some(*id),
            }
        }
    }
}

/// A due projection's work link is intrinsic; annotations cannot replace it.
pub fn deadline_work(
    input: &crate::domain::EvaluationInput,
    deadline: &Deadline,
) -> crate::domain::DeadlineWork {
    use crate::domain::DeadlineWork;
    match deadline.fulfillment {
        Fulfillment::TraceTask(id) => DeadlineWork::Task(id),
        Fulfillment::Recorded(_) => input
            .deadline_annotations
            .iter()
            .find(|annotation| annotation.target_id == deadline.meta.id)
            .map(|annotation| annotation.work.clone())
            .unwrap_or_default(),
    }
}
