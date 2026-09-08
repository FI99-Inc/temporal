use super::*;
use crate::results::{EvaluationRequest, OccurrenceKey, Suggestion};
use crate::time::{Cutoff, Instant, LocalDate, TemporalSpan, TimedSpan, ZoneId};
use std::{collections::BTreeSet, num::NonZeroU64};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecordMeta<I> {
    pub id: I,
    pub revision: NonZeroU64,
    pub created_at: Instant,
    pub updated_at: Instant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Presence {
    Present,
    Removed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnchorRigidity {
    Fixed,
    Constrained,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReportedCertainty {
    Confirmed,
    Tentative,
    Unspecified,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Occupancy {
    Busy,
    Transparent,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Anchor {
    pub meta: RecordMeta<AnchorId>,
    pub title: String,
    pub presence: Presence,
    pub provenance: Provenance,
    pub span: TemporalSpan,
    pub rigidity: AnchorRigidity,
    pub reported_certainty: ReportedCertainty,
    pub occupancy: Occupancy,
    pub location: Option<ContextTag>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvidenceAuthority {
    LocalUser,
    Source,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolutionEvidence {
    pub authority: EvidenceAuthority,
    pub recorded_at: Instant,
    pub effective_at: Option<Instant>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserStateEvidence {
    pub recorded_at: Instant,
    pub effective_at: Option<Instant>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecordedResolution {
    Unresolved,
    Satisfied(ResolutionEvidence),
    Cancelled(ResolutionEvidence),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Fulfillment {
    Recorded(RecordedResolution),
    TraceTask(TaskRefId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Deadline {
    pub meta: RecordMeta<DeadlineId>,
    pub title: String,
    pub presence: Presence,
    pub provenance: Provenance,
    pub cutoff: Cutoff,
    pub fulfillment: Fulfillment,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub enum DeadlineWork {
    #[default]
    Unspecified,
    Standalone(WorkMetadata),
    Task(TaskRefId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TaskStatus {
    Now,
    Later,
    Someday,
    Done,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TaskDue {
    None,
    Unresolved { value: String, reason: String },
    Deadline(DeadlineId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskRef {
    pub meta: RecordMeta<TaskRefId>,
    pub title: String,
    pub presence: Presence,
    pub provenance: TaskProvenance,
    pub status: TaskStatus,
    pub due: TaskDue,
    pub raw_input: Option<String>,
    pub unknown_status_label: Option<String>,
    pub source_priority: Option<String>,
    pub source_context: Option<String>,
    pub source_sort_order: Option<i64>,
    pub source_link: Option<String>,
    pub source_completed_at: Option<Instant>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntentionState {
    Active,
    Done,
    Dismissed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Intention {
    pub meta: RecordMeta<IntentionId>,
    pub title: String,
    pub presence: Presence,
    pub provenance: UserProvenance,
    pub preferred_span: Option<TemporalSpan>,
    pub work: Option<WorkMetadata>,
    pub state: IntentionState,
    pub state_evidence: Option<UserStateEvidence>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RoutineState {
    Active,
    Paused,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Weekday {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}
impl Weekday {
    pub fn number(self) -> u32 {
        match self {
            Self::Mon => 1,
            Self::Tue => 2,
            Self::Wed => 3,
            Self::Thu => 4,
            Self::Fri => 5,
            Self::Sat => 6,
            Self::Sun => 7,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeeklyRule {
    pub weekdays: BTreeSet<Weekday>,
    pub start_date: LocalDate,
    pub until_date_exclusive: Option<LocalDate>,
    pub zone: ZoneId,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RoutineRule {
    Weekly(WeeklyRule),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Routine {
    pub meta: RecordMeta<RoutineId>,
    pub title: String,
    pub presence: Presence,
    pub provenance: UserProvenance,
    pub state: RoutineState,
    pub work: WorkMetadata,
    pub rule: RoutineRule,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Outcome {
    Done,
    Skipped,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutineOutcome {
    pub routine_id: RoutineId,
    pub date: LocalDate,
    pub outcome: Outcome,
    pub revision: NonZeroU64,
    pub recorded_at: Instant,
}
impl RoutineOutcome {
    pub fn key(&self) -> OccurrenceKey {
        OccurrenceKey {
            routine_id: self.routine_id,
            date: self.date,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Confirmation {
    pub confirmed_at: Instant,
    pub record_revision: NonZeroU64,
}
impl Confirmation {
    pub fn is_current(&self, fact_revision: NonZeroU64) -> bool {
        self.record_revision == fact_revision
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnchorAnnotation {
    pub target_id: AnchorId,
    pub revision: NonZeroU64,
    pub created_at: Instant,
    pub updated_at: Instant,
    pub importance: Importance,
    pub milestone: bool,
    pub confirmation: Option<Confirmation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeadlineAnnotation {
    pub target_id: DeadlineId,
    pub revision: NonZeroU64,
    pub created_at: Instant,
    pub updated_at: Instant,
    pub importance: Importance,
    pub milestone: bool,
    pub work: DeadlineWork,
    pub confirmation: Option<Confirmation>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TaskAnnotation {
    pub target_id: TaskRefId,
    pub revision: NonZeroU64,
    pub created_at: Instant,
    pub updated_at: Instant,
    pub importance: Importance,
    pub work: WorkMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AvailabilityDeclaration {
    pub meta: RecordMeta<AvailabilityId>,
    pub span: TimedSpan,
    pub contexts: DeclaredContexts,
    pub energy_capacity: EnergyCapacity,
}

/// Normalized facts and user state, with evaluation settings and separately
/// retained inference. This is not a database model or a source wire schema.
/// ```compile_fail
/// use temporal_core::{domain::EvaluationInput, results::Suggestion};
/// fn promote(input: &mut EvaluationInput, advice: Suggestion) {
///     input.deadlines.push(advice);
/// }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluationInput {
    pub schema_version: u32,
    pub snapshot_revision: NonZeroU64,
    pub captured_now: Instant,
    pub sources: Vec<Source>,
    pub source_states: Vec<SourceState>,
    pub required_sources: Vec<RequiredSource>,
    pub anchors: Vec<Anchor>,
    pub deadlines: Vec<Deadline>,
    pub task_refs: Vec<TaskRef>,
    pub intentions: Vec<Intention>,
    pub routines: Vec<Routine>,
    pub anchor_annotations: Vec<AnchorAnnotation>,
    pub deadline_annotations: Vec<DeadlineAnnotation>,
    pub task_annotations: Vec<TaskAnnotation>,
    pub routine_outcomes: Vec<RoutineOutcome>,
    pub availability: Vec<AvailabilityDeclaration>,
    pub evaluation: EvaluationRequest,
    pub prior_suggestions: Vec<Suggestion>,
}
