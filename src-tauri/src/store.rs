//! The application's own SQLite cache. There is no Trace database connection.
use crate::{
    local::{self, LocalMutation, LocalState},
    trace::{self, Export, ImportError, TraceTask},
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, num::NonZeroU64, path::Path};
use temporal_core::{domain::*, results::*, time::*};

pub struct Store {
    connection: Connection,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Metadata {
    source: Source,
    state: SourceState,
    revision: NonZeroU64,
    exported_at: Option<Instant>,
}

#[derive(Clone)]
struct CachedTask {
    raw: TraceTask,
    task: TaskRef,
}
#[derive(Clone)]
struct Cache {
    metadata: Metadata,
    tasks: BTreeMap<String, CachedTask>,
    local: LocalState,
}

pub struct StoredView {
    pub input: EvaluationInput,
    pub exported_at: Option<Instant>,
    pub last_import_at: Option<Instant>,
    pub local: LocalState,
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, String> {
        Self::initialize(Connection::open(path).map_err(db_error)?)
    }
    pub fn memory() -> Result<Self, String> {
        Self::initialize(Connection::open_in_memory().map_err(db_error)?)
    }
    fn initialize(mut connection: Connection) -> Result<Self, String> {
        connection
            .busy_timeout(std::time::Duration::from_secs(3))
            .map_err(db_error)?;
        let tx = connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let version: u32 = tx
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(db_error)?;
        match version {
            0 => {
                let tables: u32 = tx.query_row("SELECT count(*) FROM sqlite_schema WHERE type='table' AND name NOT LIKE 'sqlite_%'",[],|r|r.get(0)).map_err(db_error)?;
                if tables != 0 {
                    return Err(
                        "This is not an empty Temporal Engine store; it was left unchanged.".into(),
                    );
                }
                tx.execute_batch(include_str!("../migrations/001_trace_cache.sql"))
                    .map_err(db_error)?;
                tx.execute_batch(include_str!("../migrations/002_local_state.sql"))
                    .map_err(db_error)?;
                let id = uuid::Uuid::new_v4()
                    .to_string()
                    .parse()
                    .expect("UUIDv4 generator");
                let metadata = Metadata {
                    source: Source {
                        id,
                        kind: SourceKind::Trace,
                        label: "Trace snapshot".into(),
                    },
                    state: SourceState {
                        source_id: id,
                        last_attempt_outcome: AttemptOutcome::Never,
                        fresh_for_ms: NonZeroU64::new(trace::MAX_AGE_MS).unwrap(),
                        last_attempt_at: None,
                        last_success_at: None,
                        anchor_coverage: None,
                        deadline_coverage: None,
                        tasks_complete: false,
                    },
                    revision: NonZeroU64::MIN,
                    exported_at: None,
                };
                save_metadata(&tx, &metadata)?;
                let local_id = uuid::Uuid::new_v4()
                    .to_string()
                    .parse()
                    .expect("UUIDv4 generator");
                save_local(&tx, &LocalState::empty(local_id))?;
            }
            1 => {
                tx.execute_batch(include_str!("../migrations/002_local_state.sql"))
                    .map_err(db_error)?;
                ensure_local_state(&tx)?;
                load_cache(&tx)?;
            }
            2 => {
                load_cache(&tx)?;
            }
            _ => return Err(
                "Unsupported Temporal Engine store version; no reset or migration was attempted."
                    .into(),
            ),
        }
        tx.commit().map_err(db_error)?;
        Ok(Self { connection })
    }

    pub fn read(&self, now: Instant, zone: ZoneId) -> Result<StoredView, String> {
        let tx = self.connection.unchecked_transaction().map_err(db_error)?;
        let cache = load_cache(&tx)?;
        check_clock(&cache, now)?;
        let input = build_input(&cache, now, zone)?;
        temporal_core::validation::validate(&input)
            .map_err(|_| "Cached temporal data failed validation.".to_string())?;
        tx.commit().map_err(db_error)?;
        Ok(StoredView {
            input,
            exported_at: cache.metadata.exported_at,
            last_import_at: cache.metadata.state.last_success_at,
            local: cache.local,
        })
    }

    pub fn mutate_local(&mut self, mutation: &LocalMutation, now: Instant) -> Result<(), String> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let previous = load_cache(&tx)?;
        check_clock(&previous, now)?;
        let previous_input = build_input(&previous, now, "America/Toronto".parse().unwrap())?;
        temporal_core::validation::validate(&previous_input).map_err(|_| {
            "Cached temporal data failed validation; no local change was applied.".to_string()
        })?;
        let mut next = previous.clone();
        let trace_tasks: Vec<_> = next
            .tasks
            .iter()
            .map(|(external, cached)| (external.clone(), cached.task.meta.id))
            .collect();
        local::apply(&mut next.local, mutation, &trace_tasks, now)?;
        next.local.updated_at = Some(now);
        next.local.revision = next
            .local
            .revision
            .get()
            .checked_add(1)
            .and_then(NonZeroU64::new)
            .ok_or_else(|| "local snapshot revision limit reached".to_string())?;
        next.metadata.revision = next_revision(next.metadata.revision)?;
        let candidate = build_input(&next, now, "America/Toronto".parse().unwrap())?;
        temporal_core::validation::validate(&candidate).map_err(|_| {
            "Local temporal change failed validation; previous state kept.".to_string()
        })?;
        save_metadata(&tx, &next.metadata)?;
        save_local(&tx, &next.local)?;
        tx.commit().map_err(db_error)
    }

    pub fn import_json(&mut self, json: &str, now: Instant) -> Result<(), String> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let previous = load_cache(&tx)?;
        check_clock(&previous, now)?;
        let applied = trace::decode_json(json, now)
            .and_then(|export| reconcile(&previous, export, now))
            .and_then(|next| {
                let candidate = build_input(&next, now, "America/Toronto".parse().unwrap())
                    .map_err(|_| {
                        ImportError::partial(
                            "Trace evaluation time is out of range; previous snapshot kept.",
                        )
                    })?;
                temporal_core::validation::validate(&candidate).map_err(|_| {
                    ImportError::partial(
                        "Trace normalization failed validation; previous snapshot kept.",
                    )
                })?;
                Ok(next)
            });
        match applied {
            Ok(next) => {
                // A failed statement rolls back the entire candidate before recording
                // the failed attempt in its own transaction, if the store is writable.
                let persisted =
                    persist_cache(&tx, &next).and_then(|()| tx.commit().map_err(db_error));
                if persisted.is_err() {
                    let _ = self.record_write_failure(now);
                }
                persisted
            }
            Err(error) => {
                let mut metadata = previous.metadata;
                metadata.revision = next_revision(metadata.revision)?;
                metadata.state.last_attempt_at = Some(now);
                metadata.state.last_attempt_outcome = error.outcome;
                save_metadata(&tx, &metadata)?;
                tx.commit().map_err(db_error)?;
                Err(error.message.into())
            }
        }
    }

    fn record_write_failure(&mut self, now: Instant) -> Result<(), String> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let mut metadata = load_cache(&tx)?.metadata;
        metadata.revision = next_revision(metadata.revision)?;
        metadata.state.last_attempt_at = Some(now);
        metadata.state.last_attempt_outcome = AttemptOutcome::Failed;
        save_metadata(&tx, &metadata)?;
        tx.commit().map_err(db_error)
    }
}

fn persist_cache(connection: &Connection, next: &Cache) -> Result<(), String> {
    for (external_id, row) in &next.tasks {
        connection.execute("INSERT INTO trace_tasks (external_id,canonical_id,source_row,normalized) VALUES (?1,?2,?3,?4) ON CONFLICT(external_id) DO UPDATE SET source_row=excluded.source_row,normalized=excluded.normalized",
            params![external_id,row.task.meta.id.to_string(),encode(&row.raw)?,encode(&row.task)?]).map_err(db_error)?;
    }
    save_metadata(connection, &next.metadata)?;
    save_local(connection, &next.local)
}

fn reconcile(previous: &Cache, mut export: Export, now: Instant) -> Result<Cache, ImportError> {
    export.tasks.sort_by(|a, b| a.id.cmp(&b.id));
    if let Some(prior_time) = previous.metadata.exported_at {
        if export.exported_at < prior_time {
            return Err(ImportError::partial(
                "Older Trace export would rewind the cache; previous snapshot kept.",
            ));
        }
        let old_rows: Vec<_> = previous
            .tasks
            .values()
            .filter(|r| r.task.presence == Presence::Present)
            .map(|r| &r.raw)
            .collect();
        if export.exported_at == prior_time && old_rows != export.tasks.iter().collect::<Vec<_>>() {
            return Err(ImportError::partial(
                "Conflicting Trace exports have the same timestamp; previous snapshot kept.",
            ));
        }
    }
    let mut ordered: Vec<_> = export.tasks.iter().collect();
    ordered.sort_by(|a, b| {
        a.status
            .cmp(&b.status)
            .then_with(|| a.sort_order.total_cmp(&b.sort_order))
            .then_with(|| a.id.cmp(&b.id))
    });
    let mut ordinals = BTreeMap::new();
    let mut previous_status = "";
    let mut ordinal = 0i64;
    for row in ordered {
        if row.status != previous_status {
            ordinal = 0;
            previous_status = &row.status;
        }
        ordinals.insert(row.id.as_str(), ordinal);
        ordinal += 1;
    }
    let mut next = previous.clone();
    for raw in &export.tasks {
        let ordinal = ordinals[raw.id.as_str()];
        let old = next.tasks.get(&raw.id);
        if old.is_some_and(|o| {
            o.raw == *raw
                && o.task.presence == Presence::Present
                && o.task.source_sort_order == Some(ordinal)
        }) {
            continue;
        }
        let meta = match old {
            Some(old) => RecordMeta {
                revision: increment(old.task.meta.revision)?,
                updated_at: now,
                ..old.task.meta.clone()
            },
            None => RecordMeta {
                id: uuid::Uuid::new_v4()
                    .to_string()
                    .parse()
                    .expect("UUIDv4 generator"),
                revision: NonZeroU64::MIN,
                created_at: now,
                updated_at: now,
            },
        };
        let task = raw.normalize(next.metadata.source.id, meta, ordinal);
        next.tasks.insert(
            raw.id.clone(),
            CachedTask {
                raw: raw.clone(),
                task,
            },
        );
    }
    for (id, row) in &mut next.tasks {
        if row.task.presence == Presence::Present && !ordinals.contains_key(id.as_str()) {
            row.task.presence = Presence::Removed;
            row.task.meta.revision = increment(row.task.meta.revision)?;
            row.task.meta.updated_at = now;
            let TaskProvenance::Imported(provenance) = &mut row.task.provenance;
            provenance.observed_at = now;
        }
    }
    next.metadata.revision = increment(next.metadata.revision)?;
    next.metadata.exported_at = Some(export.exported_at);
    next.metadata.state.last_attempt_at = Some(now);
    next.metadata.state.last_success_at = Some(now);
    next.metadata.state.last_attempt_outcome = AttemptOutcome::Complete;
    next.metadata.state.fresh_for_ms =
        NonZeroU64::new(export.remaining_fresh_ms).expect("positive accepted age");
    next.metadata.state.tasks_complete = true;
    Ok(next)
}

fn increment(value: NonZeroU64) -> Result<NonZeroU64, ImportError> {
    value
        .get()
        .checked_add(1)
        .and_then(NonZeroU64::new)
        .ok_or(ImportError::partial(
            "Cache revision limit reached; previous snapshot kept.",
        ))
}
fn next_revision(value: NonZeroU64) -> Result<NonZeroU64, String> {
    increment(value).map_err(|e| e.message.into())
}
fn check_clock(cache: &Cache, now: Instant) -> Result<(), String> {
    if cache
        .metadata
        .state
        .last_attempt_at
        .is_some_and(|at| at > now)
        || cache.tasks.values().any(|r| r.task.meta.updated_at > now)
        || cache.local.updated_at.is_some_and(|at| at > now)
        || cache
            .local
            .anchors
            .iter()
            .any(|record| record.meta.created_at > now || record.meta.updated_at > now)
        || cache
            .local
            .deadlines
            .iter()
            .any(|record| record.meta.created_at > now || record.meta.updated_at > now)
        || cache
            .local
            .intentions
            .iter()
            .any(|record| record.meta.created_at > now || record.meta.updated_at > now)
        || cache
            .local
            .routines
            .iter()
            .any(|record| record.meta.created_at > now || record.meta.updated_at > now)
        || cache
            .local
            .availability
            .iter()
            .any(|record| record.meta.created_at > now || record.meta.updated_at > now)
        || cache
            .local
            .anchor_annotations
            .iter()
            .any(|record| record.created_at > now || record.updated_at > now)
        || cache
            .local
            .deadline_annotations
            .iter()
            .any(|record| record.created_at > now || record.updated_at > now)
        || cache
            .local
            .task_annotations
            .iter()
            .any(|record| record.created_at > now || record.updated_at > now)
        || cache
            .local
            .routine_outcomes
            .iter()
            .any(|record| record.recorded_at > now)
    {
        Err("System time is earlier than cached records. Check the clock; recorded timestamps were kept.".into())
    } else {
        Ok(())
    }
}
fn encode(value: &impl Serialize) -> Result<String, String> {
    serde_json::to_string(value).map_err(|_| "Could not encode the local cache.".into())
}
fn db_error(_: rusqlite::Error) -> String {
    "Temporal Engine could not read or commit its local cache. Previous data was not reset.".into()
}
fn save_metadata(connection: &Connection, metadata: &Metadata) -> Result<(), String> {
    connection.execute("INSERT INTO trace_source_state(singleton,payload) VALUES(1,?1) ON CONFLICT(singleton) DO UPDATE SET payload=excluded.payload",[encode(metadata)?]).map_err(db_error)?;
    Ok(())
}
fn load_cache(connection: &Connection) -> Result<Cache, String> {
    let payload: String = connection
        .query_row(
            "SELECT payload FROM trace_source_state WHERE singleton=1",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    let metadata: Metadata = serde_json::from_str(&payload)
        .map_err(|_| "Local source state is invalid; no reset was attempted.".to_string())?;
    let mut statement=connection.prepare("SELECT external_id,canonical_id,source_row,normalized FROM trace_tasks ORDER BY external_id").map_err(db_error)?;
    let rows = statement
        .query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
                r.get::<_, String>(3)?,
            ))
        })
        .map_err(db_error)?;
    let mut tasks = BTreeMap::new();
    for row in rows {
        let (id, canonical, raw, normalized) = row.map_err(db_error)?;
        let raw: TraceTask =
            serde_json::from_str(&raw).map_err(|_| "Invalid cached Trace data.".to_string())?;
        let task: TaskRef = serde_json::from_str(&normalized)
            .map_err(|_| "Invalid cached temporal data.".to_string())?;
        if id != raw.id
            || task.meta.id.to_string() != canonical
            || task.provenance.imported().external_id != id
            || task.provenance.source_id() != metadata.source.id
        {
            return Err("Cached identity mapping is inconsistent; no reset was attempted.".into());
        }
        tasks.insert(id, CachedTask { raw, task });
    }
    let local_payload: String = connection
        .query_row(
            "SELECT payload FROM local_temporal_state WHERE singleton=1",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    let local: LocalState = serde_json::from_str(&local_payload)
        .map_err(|_| "Invalid cached local temporal data.".to_string())?;
    if local.source.kind != SourceKind::Local {
        return Err("Cached local source has an invalid kind; no reset was attempted.".into());
    }
    Ok(Cache {
        metadata,
        tasks,
        local,
    })
}
fn build_input(cache: &Cache, now: Instant, zone: ZoneId) -> Result<EvaluationInput, String> {
    Ok(EvaluationInput {
        schema_version: 1,
        snapshot_revision: cache.metadata.revision,
        captured_now: now,
        sources: vec![cache.metadata.source.clone(), cache.local.source.clone()],
        source_states: vec![cache.metadata.state.clone()],
        required_sources: vec![RequiredSource {
            source_id: cache.metadata.source.id,
            role: SourceRole::Tasks,
        }],
        task_refs: cache.tasks.values().map(|r| r.task.clone()).collect(),
        anchors: cache.local.anchors.clone(),
        deadlines: cache.local.deadlines.clone(),
        intentions: cache.local.intentions.clone(),
        routines: cache.local.routines.clone(),
        anchor_annotations: cache.local.anchor_annotations.clone(),
        deadline_annotations: cache.local.deadline_annotations.clone(),
        task_annotations: cache.local.task_annotations.clone(),
        routine_outcomes: cache.local.routine_outcomes.clone(),
        availability: cache.local.availability.clone(),
        prior_suggestions: vec![],
        evaluation: EvaluationRequest {
            now,
            evaluation_end: now
                .checked_add_ms(14 * 86_400_000)
                .map_err(|e| e.to_string())?,
            display_zone: zone,
            timezone_rules_version: TimezoneRules::bundled().version().into(),
            policy_version: PolicyVersion::ProofV1,
        },
    })
}

fn ensure_local_state(connection: &Connection) -> Result<(), String> {
    let exists: Option<String> = connection
        .query_row(
            "SELECT payload FROM local_temporal_state WHERE singleton=1",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(db_error)?;
    if let Some(payload) = exists {
        let local: LocalState = serde_json::from_str(&payload)
            .map_err(|_| "Invalid cached local temporal data.".to_string())?;
        if local.source.kind != SourceKind::Local {
            return Err("Cached local source has an invalid kind; no reset was attempted.".into());
        }
    } else {
        let id = uuid::Uuid::new_v4()
            .to_string()
            .parse()
            .expect("UUIDv4 generator");
        save_local(connection, &LocalState::empty(id))?;
    }
    Ok(())
}

fn save_local(connection: &Connection, local: &LocalState) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO local_temporal_state(singleton,payload) VALUES(1,?1) ON CONFLICT(singleton) DO UPDATE SET payload=excluded.payload",
            [encode(local)?],
        )
        .map_err(db_error)?;
    Ok(())
}
