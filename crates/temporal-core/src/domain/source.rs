use super::SourceId;
use crate::time::{Instant, TimedSpan};
use std::num::NonZeroU64;

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    Local,
    Trace,
    Quercus,
    Google,
    Outlook,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub id: SourceId,
    pub kind: SourceKind,
    pub label: String,
}

#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum Projection {
    Anchor,
    Deadline,
    Task,
    TaskDue,
}

#[derive(
    Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct ImportedIdentity {
    pub source_id: SourceId,
    pub external_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_key: Option<String>,
    pub projection: Projection,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalAssertion {
    pub source_id: SourceId,
    pub asserted_at: Instant,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImportedProvenance {
    pub source_id: SourceId,
    pub external_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub occurrence_key: Option<String>,
    pub projection: Projection,
    pub observed_at: Instant,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_revision: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_created_at: Option<Instant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_updated_at: Option<Instant>,
}

impl ImportedProvenance {
    pub fn identity(&self) -> ImportedIdentity {
        ImportedIdentity {
            source_id: self.source_id,
            external_id: self.external_id.clone(),
            occurrence_key: self.occurrence_key.clone(),
            projection: self.projection,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum Provenance {
    LocalUser(LocalAssertion),
    Imported(ImportedProvenance),
}
/// Local soft state cannot carry source-owned provenance.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum UserProvenance {
    LocalUser(LocalAssertion),
}
/// A Trace task cannot be introduced as a local task-capture record.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum TaskProvenance {
    Imported(ImportedProvenance),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum FactWriter {
    LocalUser,
    Source(SourceId),
}

impl Provenance {
    pub fn source_id(&self) -> SourceId {
        match self {
            Self::LocalUser(value) => value.source_id,
            Self::Imported(value) => value.source_id,
        }
    }
    pub fn writer(&self) -> FactWriter {
        match self {
            Self::LocalUser(_) => FactWriter::LocalUser,
            Self::Imported(value) => FactWriter::Source(value.source_id),
        }
    }
}
impl UserProvenance {
    pub fn assertion(&self) -> &LocalAssertion {
        match self {
            Self::LocalUser(value) => value,
        }
    }
    pub fn source_id(&self) -> SourceId {
        self.assertion().source_id
    }
}
impl TaskProvenance {
    pub fn imported(&self) -> &ImportedProvenance {
        match self {
            Self::Imported(value) => value,
        }
    }
    pub fn source_id(&self) -> SourceId {
        self.imported().source_id
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AttemptOutcome {
    Never,
    Complete,
    Partial,
    Failed,
    Incompatible,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceState {
    pub source_id: SourceId,
    pub last_attempt_outcome: AttemptOutcome,
    pub fresh_for_ms: NonZeroU64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_attempt_at: Option<Instant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_success_at: Option<Instant>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_coverage: Option<TimedSpan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deadline_coverage: Option<TimedSpan>,
    pub tasks_complete: bool,
}

#[derive(
    Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum SourceRole {
    Anchors,
    Deadlines,
    Tasks,
}

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(deny_unknown_fields)]
pub struct RequiredSource {
    pub source_id: SourceId,
    pub role: SourceRole,
}
