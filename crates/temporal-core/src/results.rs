//! Derived state and inferred advice are separate from stored temporal objects.

use crate::domain::*;
use crate::reasons::{Reason, ReasonCode};
use crate::time::{Cutoff, DateSpan, Instant, LocalDate, TemporalSpan, TimedSpan, ZoneId};
use std::{
    collections::BTreeSet,
    num::{NonZeroU32, NonZeroU64},
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum PolicyVersion {
    ProofV1,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluationRequest {
    pub now: Instant,
    pub evaluation_end: Instant,
    pub display_zone: ZoneId,
    pub timezone_rules_version: String,
    pub policy_version: PolicyVersion,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EvaluationKey {
    pub snapshot_revision: NonZeroU64,
    pub now: Instant,
    pub evaluation_end: Instant,
    pub display_zone: ZoneId,
    pub timezone_rules_version: String,
    pub policy_version: PolicyVersion,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EvaluationBasis {
    pub snapshot_revision: NonZeroU64,
    pub evaluation_end: Instant,
    pub display_zone: ZoneId,
    pub timezone_rules_version: String,
    pub policy_version: PolicyVersion,
}

impl EvaluationRequest {
    pub fn key(&self, snapshot_revision: NonZeroU64) -> EvaluationKey {
        EvaluationKey {
            snapshot_revision,
            now: self.now,
            evaluation_end: self.evaluation_end,
            display_zone: self.display_zone,
            timezone_rules_version: self.timezone_rules_version.clone(),
            policy_version: self.policy_version,
        }
    }
}
impl EvaluationKey {
    pub fn basis(&self) -> EvaluationBasis {
        EvaluationBasis {
            snapshot_revision: self.snapshot_revision,
            evaluation_end: self.evaluation_end,
            display_zone: self.display_zone,
            timezone_rules_version: self.timezone_rules_version.clone(),
            policy_version: self.policy_version,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct OccurrenceKey {
    pub routine_id: RoutineId,
    pub date: LocalDate,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct WindowKey {
    pub evaluation_key: EvaluationKey,
    pub availability_id: AvailabilityId,
    pub start: Instant,
    pub end: Instant,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PressureKey {
    pub evaluation_key: EvaluationKey,
    pub deadline_id: DeadlineId,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum WorkTarget {
    Deadline(DeadlineId),
    Intention(IntentionId),
    RoutineOccurrence(OccurrenceKey),
    Task(TaskRefId),
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Reference {
    Anchor(AnchorId),
    Availability(AvailabilityId),
    Deadline(DeadlineId),
    Intention(IntentionId),
    Pressure(PressureKey),
    Routine(RoutineId),
    RoutineOccurrence(OccurrenceKey),
    Source(SourceId),
    Task(TaskRefId),
    Window(WindowKey),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnchorPhase {
    Upcoming,
    Ongoing,
    Passed,
    Inactive,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeadlinePhase {
    Upcoming,
    DueNow,
    DueToday,
    Overdue,
    Inactive,
    Unknown,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Resolution {
    Unresolved,
    Satisfied,
    Cancelled,
    Unknown,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum IntentionPhase {
    Inactive,
    NoPreference,
    PreferenceUpcoming,
    PreferredNow,
    PreferencePassed,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OccurrencePhase {
    Inactive,
    Done,
    Skipped,
    Future,
    Current,
    PastUnrecorded,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Health {
    Incompatible,
    Unavailable,
    Partial,
    NeverLoaded,
    Stale,
    Healthy,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceHealthRow {
    pub source_id: SourceId,
    pub health: Health,
    pub reasons: Vec<Reason>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnchorStateRow {
    pub id: AnchorId,
    pub phase: AnchorPhase,
    pub reasons: Vec<Reason>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeadlineStateRow {
    pub id: DeadlineId,
    pub presence: Presence,
    pub resolution: Resolution,
    pub phase: DeadlinePhase,
    pub cutoff: Cutoff,
    pub endpoint: Instant,
    pub reasons: Vec<Reason>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct IntentionStateRow {
    pub id: IntentionId,
    pub phase: IntentionPhase,
    pub reasons: Vec<Reason>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoutineOccurrence {
    pub key: OccurrenceKey,
    pub span: DateSpan,
    pub phase: OccurrencePhase,
    pub reasons: Vec<Reason>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SourceQualification {
    pub source_id: SourceId,
    pub role: SourceRole,
    pub health: Health,
    pub covered: bool,
    pub reasons: Vec<Reason>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Window {
    pub key: WindowKey,
    pub span: TimedSpan,
    pub declaration_ref: AvailabilityId,
    pub blocking_anchor_refs: Vec<AnchorId>,
    pub contexts: DeclaredContexts,
    pub energy_capacity: EnergyCapacity,
    pub source_qualifications: Vec<SourceQualification>,
    pub reasons: Vec<Reason>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnchorConflict {
    pub anchor_ids: [AnchorId; 2],
    pub intersection: TimedSpan,
    pub reasons: Vec<Reason>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FitStatus {
    Fits,
    DoesNotFit,
    Unknown,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FitResult {
    pub target: WorkTarget,
    pub window_key: WindowKey,
    pub span: Option<TimedSpan>,
    pub status: FitStatus,
    pub reasons: Vec<Reason>,
}

/// Effective work may differ from the stored annotation, for example when its
/// linked source task is Done. That derivation must retain an explicit reason.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Workload {
    Unknown,
    Estimate(u32),
    AtLeast(NonZeroU32),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Opportunity {
    Known {
        milliseconds: u64,
        known_qualifying_ms: u64,
        window_keys: Vec<WindowKey>,
    },
    Unknown {
        known_qualifying_ms: u64,
        window_keys: Vec<WindowKey>,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PressureRatio {
    pub work_ms: u64,
    pub opportunity_ms: NonZeroU64,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Risk {
    NotApplicable,
    Unknown,
    Overdue,
    NoKnownWork,
    Insufficient,
    Room,
    Tight,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Qualification {
    DeclaredInputs,
    Conditional,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PressureResult {
    pub evaluation_key: EvaluationKey,
    pub deadline_id: DeadlineId,
    pub work_target: Option<WorkTarget>,
    pub cutoff: Cutoff,
    pub endpoint: Instant,
    pub resolution: Resolution,
    pub presence: Presence,
    pub phase: DeadlinePhase,
    pub workload: Option<Workload>,
    pub opportunity: Option<Opportunity>,
    pub ratio: Option<PressureRatio>,
    pub risk: Risk,
    pub qualification: Qualification,
    pub source_qualifications: Vec<SourceQualification>,
    pub reasons: Vec<Reason>,
    pub limitations: BTreeSet<ReasonCode>,
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum SuggestionKind {
    ConsiderWork,
    RiskNotice,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuggestionKey {
    pub evaluation_key: EvaluationKey,
    pub kind: SuggestionKind,
    pub target: WorkTarget,
    pub proposed_span: Option<TemporalSpan>,
}

/// Inference has neither completion nor overdue state.
/// ```compile_fail
/// use temporal_core::results::Suggestion;
/// fn mark_completed(suggestion: &mut Suggestion) { suggestion.completed = true; }
/// ```
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Suggestion {
    pub key: SuggestionKey,
    pub kind: SuggestionKind,
    pub target: WorkTarget,
    pub proposed_span: Option<TemporalSpan>,
    pub created_at: Instant,
    pub valid_until: Instant,
    pub reasons: Vec<Reason>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SuggestionStatus {
    Current,
    Expired,
    Invalidated,
    Ineligible,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuggestionValidity {
    pub key: SuggestionKey,
    pub status: SuggestionStatus,
    pub reasons: Vec<Reason>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EvaluationOutput {
    pub evaluation_key: EvaluationKey,
    pub source_health: Vec<SourceHealthRow>,
    pub anchor_states: Vec<AnchorStateRow>,
    pub deadline_states: Vec<DeadlineStateRow>,
    pub intention_states: Vec<IntentionStateRow>,
    pub routine_occurrences: Vec<RoutineOccurrence>,
    pub windows: Vec<Window>,
    pub conflicts: Vec<AnchorConflict>,
    pub fits: Vec<FitResult>,
    pub pressures: Vec<PressureResult>,
    pub suggestion_validity: Vec<SuggestionValidity>,
}
