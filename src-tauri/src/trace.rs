//! The verified Trace JSON 1.0 boundary. No database or network access.
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;
use temporal_core::{domain::*, time::Instant};

pub const MAX_BYTES: usize = 10 * 1024 * 1024;
pub const MAX_AGE_MS: u64 = 24 * 3_600_000;

#[derive(Debug)]
pub struct ImportError {
    pub outcome: AttemptOutcome,
    pub message: &'static str,
}
impl ImportError {
    pub fn partial(message: &'static str) -> Self {
        Self {
            outcome: AttemptOutcome::Partial,
            message,
        }
    }
}

// deserialize_with makes the key required even when its value can be null.
fn nullable<'de, D: Deserializer<'de>>(d: D) -> Result<Option<String>, D::Error> {
    Option::<String>::deserialize(d)
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TraceTask {
    pub id: String,
    pub text: String,
    #[serde(deserialize_with = "nullable")]
    pub raw_input: Option<String>,
    pub link: Option<String>,
    pub status: String,
    #[serde(deserialize_with = "nullable")]
    pub context: Option<String>,
    pub priority: i32,
    #[serde(deserialize_with = "nullable")]
    pub due_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    #[serde(deserialize_with = "nullable")]
    pub completed_at: Option<String>,
    pub sort_order: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct WireExport {
    version: String,
    exported_at: String,
    tasks: Vec<TraceTask>,
}

#[derive(Deserialize)]
struct WireVersion {
    version: String,
}

pub struct Export {
    pub exported_at: Instant,
    pub tasks: Vec<TraceTask>,
    pub remaining_fresh_ms: u64,
}

pub fn decode_json(json: &str, now: Instant) -> Result<Export, ImportError> {
    if json.len() > MAX_BYTES {
        return Err(ImportError::partial(
            "Trace export exceeds 10 MiB; previous snapshot kept.",
        ));
    }
    let wire: WireExport = serde_json::from_str(json).map_err(|error| ImportError {
        outcome: if error.to_string().starts_with("unknown field")
            || serde_json::from_str::<WireVersion>(json).is_ok_and(|v| v.version != "1.0")
        {
            AttemptOutcome::Incompatible
        } else {
            AttemptOutcome::Partial
        },
        message: "Trace export is incomplete or has an unsupported shape; previous snapshot kept.",
    })?;
    if wire.version != "1.0" {
        return Err(ImportError {
            outcome: AttemptOutcome::Incompatible,
            message: "Unsupported Trace export version; previous snapshot kept.",
        });
    }
    let exported_at = instant(&wire.exported_at)?;
    let age = now.duration_since(exported_at).map_err(|_| {
        ImportError::partial(
            "Trace export is from the future; check the clock. Previous snapshot kept.",
        )
    })?;
    if age >= MAX_AGE_MS {
        return Err(ImportError::partial(
            "Trace export is at least 24 hours old. Export again from Trace; previous snapshot kept.",
        ));
    }
    let mut ids = BTreeSet::new();
    for task in &wire.tasks {
        if task.id.trim().is_empty()
            || task.text.trim().is_empty()
            || task.status.trim().is_empty()
            || !(0..=5).contains(&task.priority)
            || !task.sort_order.is_finite()
            || task.due_at.as_ref().is_some_and(|d| d.trim().is_empty())
        {
            return Err(ImportError::partial(
                "Trace export contains invalid task fields; previous snapshot kept.",
            ));
        }
        if !ids.insert(&task.id) {
            return Err(ImportError::partial(
                "Trace export contains duplicate task IDs; previous snapshot kept.",
            ));
        }
        let created = instant(&task.created_at)?;
        let updated = instant(&task.updated_at)?;
        if created > updated || updated > exported_at {
            return Err(ImportError::partial(
                "Trace task timestamps are inconsistent; previous snapshot kept.",
            ));
        }
        if let Some(value) = &task.completed_at {
            let completed = instant(value)?;
            if completed < created
                || completed > updated
                || matches!(task.status.as_str(), "now" | "later" | "someday")
            {
                return Err(ImportError::partial(
                    "Trace completion conflicts with its task state; previous snapshot kept.",
                ));
            }
        }
    }
    Ok(Export {
        exported_at,
        tasks: wire.tasks,
        remaining_fresh_ms: MAX_AGE_MS - age,
    })
}

fn instant(value: &str) -> Result<Instant, ImportError> {
    value.parse().map_err(|_| {
        ImportError::partial("Trace export contains an invalid timestamp; previous snapshot kept.")
    })
}

impl TraceTask {
    pub fn normalize(
        &self,
        source_id: SourceId,
        meta: RecordMeta<TaskRefId>,
        ordinal: i64,
    ) -> TaskRef {
        let status = match self.status.as_str() {
            "now" => TaskStatus::Now,
            "later" => TaskStatus::Later,
            "someday" => TaskStatus::Someday,
            "done" => TaskStatus::Done,
            _ => TaskStatus::Unknown,
        };
        TaskRef {
            title: self.text.clone(),
            presence: Presence::Present,
            provenance: TaskProvenance::Imported(ImportedProvenance {
                source_id,
                external_id: self.id.clone(),
                occurrence_key: None,
                projection: Projection::Task,
                observed_at: meta.updated_at,
                source_revision: None,
                source_created_at: Some(self.created_at.parse().expect("validated timestamp")),
                source_updated_at: Some(self.updated_at.parse().expect("validated timestamp")),
            }),
            meta,
            status,
            due: self
                .due_at
                .as_ref()
                .map_or(TaskDue::None, |value| TaskDue::Unresolved {
                    value: value.clone(),
                    reason:
                        "Trace's export does not preserve the original due precision or timezone."
                            .into(),
                }),
            raw_input: self.raw_input.clone(),
            unknown_status_label: (status == TaskStatus::Unknown).then(|| self.status.clone()),
            source_priority: Some(self.priority.to_string()),
            source_context: self.context.clone(),
            source_sort_order: Some(ordinal),
            source_link: self.link.clone(),
            source_completed_at: self
                .completed_at
                .as_ref()
                .map(|v| v.parse().expect("validated timestamp")),
        }
    }
}
