//! Typed explanation payloads; prose is presentation, never an assertion source.

use crate::domain::*;
use crate::results::*;
use crate::time::{Cutoff, Instant, TimedSpan};

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeadlinePhasePayload {
    pub cutoff: Cutoff,
    pub endpoint: Instant,
    pub phase: DeadlinePhase,
    pub now: Instant,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResolutionPayload {
    pub resolution: Resolution,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<ResolutionEvidence>,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskCompletionPayload {
    pub task_status: TaskStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_completed_at: Option<Instant>,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AvailabilityPayload {
    pub span: TimedSpan,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clipped_span: Option<TimedSpan>,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnchorBlockedPayload {
    pub anchor_span: TimedSpan,
    pub intersection: TimedSpan,
    pub blocked_ms: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IntersectionPayload {
    pub intersection: TimedSpan,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConservativeAnchorPayload {
    pub reported_certainty: ReportedCertainty,
    pub occupancy: Occupancy,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EarliestStartPayload {
    pub earliest_start: Instant,
    pub window_span: TimedSpan,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ChunkPayload {
    pub available_ms: u64,
    pub chunk_ms: u64,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContextPayload {
    pub required_contexts: ContextTags,
    pub declared_contexts: DeclaredContexts,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyPayload {
    pub energy_requirement: EnergyRequirement,
    pub energy_capacity: EnergyCapacity,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnavailableField {
    Effort,
    MinimumChunkMinutes,
    Availability,
    Work,
    Status,
    Fulfillment,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UnavailablePayload {
    pub field: UnavailableField,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZeroOpportunityPayload {
    pub now: Instant,
    pub endpoint: Instant,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub earliest_start: Option<Instant>,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZeroWorkBasis {
    UserEstimate,
    TaskCompletion,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ZeroWorkPayload {
    pub basis: ZeroWorkBasis,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OutsideRangePayload {
    pub endpoint: Instant,
    pub evaluation_end: Instant,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RatioPayload {
    pub work_ms: u64,
    pub opportunity_ms: u64,
    pub risk: Risk,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LowerBoundPayload {
    pub minutes: u32,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceHealthPayload {
    pub health: Health,
    pub last_attempt_outcome: AttemptOutcome,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_attempt_at: Option<Instant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_success_at: Option<Instant>,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceCoveragePayload {
    pub role: SourceRole,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_interval: Option<TimedSpan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required_endpoint: Option<Instant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub known_interval: Option<TimedSpan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks_complete: Option<bool>,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmptyPayload {}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BoundaryPayload {
    pub boundary: Instant,
    pub now: Instant,
}
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvalidatedPayload {
    pub previous_basis: EvaluationBasis,
    pub current_basis: EvaluationBasis,
}

// Each variant fixes its payload at compile time. Serialization in Task 1.4
// represents each as exactly {code, references, payload}.
macro_rules! reasons {
    ($($variant:ident => ($code:literal, $payload:ty)),+ $(,)?) => {
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum ReasonCode { $($variant),+ }
        impl ReasonCode {
            pub fn as_str(self) -> &'static str { match self { $(Self::$variant => $code),+ } }
        }
        #[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
        #[serde(tag = "code", rename_all = "snake_case", deny_unknown_fields)]
        pub enum Reason { $($variant { references: Vec<Reference>, payload: $payload }),+ }
        impl Reason {
            pub fn code(&self) -> ReasonCode { match self { $(Self::$variant { .. } => ReasonCode::$variant),+ } }
            pub fn references(&self) -> &[Reference] { match self { $(Self::$variant { references, .. } => references),+ } }
            pub fn references_mut(&mut self) -> &mut Vec<Reference> { match self { $(Self::$variant { references, .. } => references),+ } }
        }
    }
}

reasons! {
    AnchorBlocked => ("anchor_blocked", AnchorBlockedPayload),
    AnchorConflict => ("anchor_conflict", IntersectionPayload),
    AvailabilityUnknown => ("availability_unknown", UnavailablePayload),
    BeforeEarliestStart => ("before_earliest_start", EarliestStartPayload),
    ChunkTooShort => ("chunk_too_short", ChunkPayload),
    ChunkUnknown => ("chunk_unknown", UnavailablePayload),
    ConservativeAnchor => ("conservative_anchor", ConservativeAnchorPayload),
    ContextMismatch => ("context_mismatch", ContextPayload),
    ContextUnknown => ("context_unknown", ContextPayload),
    DeadlinePhase => ("deadline_phase", DeadlinePhasePayload),
    DeclaredAvailability => ("declared_availability", AvailabilityPayload),
    EffortLowerBound => ("effort_lower_bound", LowerBoundPayload),
    EffortUnknown => ("effort_unknown", UnavailablePayload),
    EnergyMismatch => ("energy_mismatch", EnergyPayload),
    EnergyUnknown => ("energy_unknown", EnergyPayload),
    FulfillmentUnknown => ("fulfillment_unknown", UnavailablePayload),
    IndividualCapacityOnly => ("individual_capacity_only", EmptyPayload),
    OutsideEvaluationRange => ("outside_evaluation_range", OutsideRangePayload),
    PressureRatio => ("pressure_ratio", RatioPayload),
    ResolutionRecorded => ("resolution_recorded", ResolutionPayload),
    RoutinePastUnrecorded => ("routine_past_unrecorded", BoundaryPayload),
    SoftPreferencePassed => ("soft_preference_passed", BoundaryPayload),
    SourceCoverage => ("source_coverage", SourceCoveragePayload),
    SourceHealth => ("source_health", SourceHealthPayload),
    SuggestionExpired => ("suggestion_expired", BoundaryPayload),
    SuggestionInvalidated => ("suggestion_invalidated", InvalidatedPayload),
    TaskCompletionBasis => ("task_completion_basis", TaskCompletionPayload),
    WorkUnavailable => ("work_unavailable", UnavailablePayload),
    ZeroOpportunity => ("zero_opportunity", ZeroOpportunityPayload),
    ZeroWorkUnresolved => ("zero_work_unresolved", ZeroWorkPayload),
}
