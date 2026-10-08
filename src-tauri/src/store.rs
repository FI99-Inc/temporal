//! The application's own SQLite cache. There is no Trace database connection.
use crate::{
    calendar::{self, AnchorRow, CalendarEvent, DeadlineRow},
    calendars::{self, CalendarMode, CalendarOrigin, CalendarSource, Imported},
    ics,
    local::{self, LocalMutation, LocalState},
    series,
    settings::{Settings, SettingsPatch},
    trace::{self, Export, ImportError, TraceTask},
};
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::{cell::RefCell, collections::BTreeMap, num::NonZeroU64, path::Path, sync::Arc};
use temporal_core::{domain::*, results::*, time::*};

pub struct Store {
    connection: Connection,
    /// Parsed calendars by source and revision; text is re-read only on change.
    parsed: RefCell<BTreeMap<SourceId, (NonZeroU64, Arc<ics::IcsCalendar>)>>,
}

type Parsed = BTreeMap<SourceId, Arc<ics::IcsCalendar>>;

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
    settings: Settings,
    calendars: Vec<CalendarSource>,
}

/// How far back the evaluation keeps resolved deadlines in view.
const RESOLVED_DEADLINE_LOOKBACK_MS: u64 = 7 * 86_400_000;
/// The Horizon's evaluated extent.
pub const EVALUATION_MS: u64 = 14 * 86_400_000;

pub struct StoredView {
    pub input: EvaluationInput,
    pub exported_at: Option<Instant>,
    pub last_import_at: Option<Instant>,
    pub local: LocalState,
    pub settings: Settings,
    /// Which evaluated Anchors are occurrences of a local series.
    pub series_occurrences: Vec<SeriesLink>,
    /// Imported calendars and their last-known parsed content.
    pub calendars: Vec<(CalendarSource, Arc<ics::IcsCalendar>)>,
}

impl StoredView {
    /// Imported calendar rows touching `[from, to)`, for the Calendar utility.
    pub fn imported_events(
        &self,
        now: Instant,
        from: Instant,
        to: Instant,
        risks: &BTreeMap<DeadlineId, (Risk, String)>,
    ) -> Result<Vec<CalendarEvent>, String> {
        let zone = self.settings.zone();
        let mut events = Vec::new();
        for (source, parsed) in self.calendars.iter().filter(|(s, _)| !s.hidden) {
            let (rows, _) = calendars::normalize(source, parsed, from, to, zone);
            for row in rows {
                events.push(match &row {
                    Imported::Anchor {
                        anchor,
                        place,
                        zero_length,
                    } => {
                        let mut event = calendar::anchor_event(
                            AnchorRow {
                                id: anchor.meta.id.to_string(),
                                anchor,
                                source: &source.source,
                                color: Some(source.color.clone()),
                                editable: false,
                                series: None,
                                details: None,
                                place: place.clone(),
                            },
                            now,
                        )
                        .map_err(|e| e.to_string())?;
                        if *zero_length {
                            event.end = event.start;
                        }
                        event
                    }
                    Imported::Deadline { deadline, place } => {
                        let handled = self
                            .local
                            .handled_imports
                            .contains_key(&deadline.meta.id.to_string());
                        let mut event = calendar::deadline_event(
                            DeadlineRow {
                                deadline,
                                source: &source.source,
                                color: Some(source.color.clone()),
                                editable: false,
                                details: None,
                                resolved: None,
                            },
                            now,
                            risks,
                        )
                        .map_err(|e| e.to_string())?;
                        event.location = place.clone();
                        if handled {
                            event.state = "handled".into();
                            event.risk = None;
                        }
                        event
                    }
                });
            }
        }
        Ok(events)
    }
}

/// An evaluated Anchor, the local series it came from, and its date.
pub type SeriesLink = (AnchorId, AnchorId, LocalDate);

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
                tx.execute_batch(include_str!("../migrations/003_settings.sql"))
                    .map_err(db_error)?;
                save_settings(&tx, &Settings::default())?;
                tx.execute_batch(include_str!("../migrations/004_calendars.sql"))
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
                tx.execute_batch(include_str!("../migrations/003_settings.sql"))
                    .map_err(db_error)?;
                save_settings(&tx, &Settings::default())?;
                tx.execute_batch(include_str!("../migrations/004_calendars.sql"))
                    .map_err(db_error)?;
                load_cache(&tx)?;
            }
            2 => {
                tx.execute_batch(include_str!("../migrations/003_settings.sql"))
                    .map_err(db_error)?;
                save_settings(&tx, &Settings::default())?;
                tx.execute_batch(include_str!("../migrations/004_calendars.sql"))
                    .map_err(db_error)?;
                load_cache(&tx)?;
            }
            3 => {
                tx.execute_batch(include_str!("../migrations/004_calendars.sql"))
                    .map_err(db_error)?;
                load_cache(&tx)?;
            }
            4 => {
                load_cache(&tx)?;
            }
            _ => return Err(
                "Unsupported Temporal Engine store version; no reset or migration was attempted."
                    .into(),
            ),
        }
        tx.commit().map_err(db_error)?;
        Ok(Self {
            connection,
            parsed: RefCell::new(BTreeMap::new()),
        })
    }

    pub fn read(&self, now: Instant, zone: ZoneId) -> Result<StoredView, String> {
        let tx = self.connection.unchecked_transaction().map_err(db_error)?;
        let cache = load_cache(&tx)?;
        check_clock(&cache, now)?;
        let parsed = parsed_calendars(&self.parsed, &tx, &cache.calendars, zone)?;
        let (input, series_occurrences) = build_slice(&cache, &parsed, now, zone)?;
        temporal_core::validation::validate(&input)
            .map_err(|_| "Cached temporal data failed validation.".to_string())?;
        tx.commit().map_err(db_error)?;
        let calendars = cache
            .calendars
            .iter()
            .filter_map(|c| parsed.get(&c.source.id).map(|p| (c.clone(), p.clone())))
            .collect();
        Ok(StoredView {
            input,
            exported_at: cache.metadata.exported_at,
            last_import_at: cache.metadata.state.last_success_at,
            local: cache.local,
            settings: cache.settings,
            series_occurrences,
            calendars,
        })
    }

    pub fn calendars(&self) -> Result<Vec<CalendarSource>, String> {
        load_calendars(&self.connection)
    }

    /// Add a calendar from text the person chose or a feed returned.
    /// Nothing is stored unless the text parses and the result validates.
    #[allow(clippy::too_many_arguments)]
    pub fn add_calendar(
        &mut self,
        label: &str,
        kind: SourceKind,
        mode: CalendarMode,
        color: &str,
        origin: CalendarOrigin,
        text: &str,
        now: Instant,
    ) -> Result<SourceId, String> {
        if matches!(kind, SourceKind::Local | SourceKind::Trace) {
            return Err("Choose Google, Outlook, Quercus, or another calendar.".into());
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let cache = load_cache(&tx)?;
        check_clock(&cache, now)?;
        let zone = cache.settings.zone();
        let fallback: chrono_tz::Tz = zone.name().parse().unwrap_or(chrono_tz::UTC);
        let parsed = ics::parse(text, fallback).map_err(|e| e.to_string())?;
        let id: SourceId = uuid::Uuid::new_v4()
            .to_string()
            .parse()
            .expect("UUIDv4 generator");
        let mut source = calendars::new_source(
            id,
            kind,
            calendars::clean_label(label)?,
            mode,
            calendars::clean_color(color)?,
            origin,
        );
        source.record_success(&parsed, now)?;
        let mut next = cache.clone();
        next.calendars.push(source.clone());
        let mut map = parsed_calendars(&self.parsed, &tx, &cache.calendars, zone)?;
        map.insert(id, Arc::new(parsed));
        let candidate = build_input_with(&next, &map, now, zone)?;
        temporal_core::validation::validate(&candidate).map_err(|_| {
            "This calendar could not be normalized safely; nothing was added.".to_string()
        })?;
        save_calendar(&tx, &source, Some(text))?;
        tx.commit().map_err(db_error)?;
        Ok(id)
    }

    /// Record a refresh: new text replaces the last-known calendar only if it
    /// parses and validates; any failure keeps last-known events.
    pub fn refresh_calendar(
        &mut self,
        id: SourceId,
        fetched: Result<String, (AttemptOutcome, String)>,
        now: Instant,
    ) -> Result<(), String> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let cache = load_cache(&tx)?;
        check_clock(&cache, now)?;
        let zone = cache.settings.zone();
        let mut source = cache
            .calendars
            .iter()
            .find(|c| c.source.id == id)
            .cloned()
            .ok_or("That calendar is no longer connected.")?;
        let fallback: chrono_tz::Tz = zone.name().parse().unwrap_or(chrono_tz::UTC);
        let outcome = fetched.and_then(|text| {
            let parsed = ics::parse(&text, fallback)
                .map_err(|e| (AttemptOutcome::Incompatible, e.to_string()))?;
            let mut next = cache.clone();
            let mut candidate_source = source.clone();
            candidate_source
                .record_success(&parsed, now)
                .map_err(|e| (AttemptOutcome::Failed, e))?;
            for slot in &mut next.calendars {
                if slot.source.id == id {
                    *slot = candidate_source.clone();
                }
            }
            let mut map = parsed_calendars(&self.parsed, &tx, &cache.calendars, zone)
                .map_err(|e| (AttemptOutcome::Failed, e))?;
            map.insert(id, Arc::new(parsed));
            let candidate = build_input_with(&next, &map, now, zone)
                .map_err(|e| (AttemptOutcome::Failed, e))?;
            temporal_core::validation::validate(&candidate).map_err(|_| {
                (
                    AttemptOutcome::Incompatible,
                    "The refreshed calendar could not be normalized; last-known events kept."
                        .to_string(),
                )
            })?;
            Ok((candidate_source, text))
        });
        let result = match outcome {
            Ok((next, text)) => {
                save_calendar(&tx, &next, Some(&text))?;
                Ok(())
            }
            Err((attempt, message)) => {
                source.record_failure(attempt, message.clone(), now)?;
                save_calendar(&tx, &source, None)?;
                Err(message)
            }
        };
        tx.commit().map_err(db_error)?;
        result
    }

    pub fn update_calendar(
        &mut self,
        id: SourceId,
        label: Option<&str>,
        mode: Option<CalendarMode>,
        color: Option<&str>,
        hidden: Option<bool>,
    ) -> Result<(), String> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let mut source = load_calendars(&tx)?
            .into_iter()
            .find(|c| c.source.id == id)
            .ok_or("That calendar is no longer connected.")?;
        if let Some(label) = label {
            source.source.label = calendars::clean_label(label)?;
        }
        if let Some(color) = color {
            source.color = calendars::clean_color(color)?;
        }
        if let Some(mode) = mode {
            source.mode = mode;
        }
        if let Some(hidden) = hidden {
            source.hidden = hidden;
        }
        source.revision = series::revision_after(source.revision)?;
        save_calendar(&tx, &source, None)?;
        tx.commit().map_err(db_error)
    }

    /// Disconnect a calendar and forget its last-known events. Local
    /// acknowledgements for its deadlines are removed with it.
    pub fn remove_calendar(&mut self, id: SourceId) -> Result<CalendarSource, String> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let mut cache = load_cache(&tx)?;
        let source = cache
            .calendars
            .iter()
            .find(|c| c.source.id == id)
            .cloned()
            .ok_or("That calendar is no longer connected.")?;
        tx.execute("DELETE FROM calendar_sources WHERE id=?1", [id.to_string()])
            .map_err(db_error)?;
        let prefix = id.to_string();
        let before = cache.local.handled_imports.len();
        cache
            .local
            .handled_imports
            .retain(|_, handled| handled.source_id != prefix);
        if cache.local.handled_imports.len() != before {
            save_local(&tx, &cache.local)?;
        }
        tx.commit().map_err(db_error)?;
        self.parsed.borrow_mut().remove(&id);
        Ok(source)
    }

    pub fn settings(&self) -> Result<Settings, String> {
        load_settings(&self.connection)
    }

    pub fn update_settings(&mut self, patch: &SettingsPatch) -> Result<Settings, String> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let mut settings = load_settings(&tx)?;
        settings.apply(patch)?;
        save_settings(&tx, &settings)?;
        tx.commit().map_err(db_error)?;
        Ok(settings)
    }

    /// Remember the Trace export file the user chose, for later re-reads.
    pub fn remember_trace_path(&mut self, path: Option<String>) -> Result<(), String> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let mut settings = load_settings(&tx)?;
        settings.trace_export_path = path;
        save_settings(&tx, &settings)?;
        tx.commit().map_err(db_error)
    }

    pub fn mutate_local(&mut self, mutation: &LocalMutation, now: Instant) -> Result<(), String> {
        self.mutate_local_batch(std::slice::from_ref(mutation), now)
    }

    /// Apply several local changes as one transaction: all of them or none.
    pub fn mutate_local_batch(
        &mut self,
        mutations: &[LocalMutation],
        now: Instant,
    ) -> Result<(), String> {
        if mutations.is_empty() || mutations.len() > 16 {
            return Err("Send between one and sixteen local changes at once.".into());
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(db_error)?;
        let previous = load_cache(&tx)?;
        check_clock(&previous, now)?;
        let zone = previous.settings.zone();
        let parsed = parsed_calendars(&self.parsed, &tx, &previous.calendars, zone)?;
        let previous_input = build_input_with(&previous, &parsed, now, zone)?;
        temporal_core::validation::validate(&previous_input).map_err(|_| {
            "Cached temporal data failed validation; no local change was applied.".to_string()
        })?;
        let mut next = previous.clone();
        let trace_tasks: Vec<_> = next
            .tasks
            .iter()
            .map(|(external, cached)| (external.clone(), cached.task.meta.id))
            .collect();
        let imported = imported_deadline_ids(&previous, &parsed, now, zone)?;
        for mutation in mutations {
            local::apply(&mut next.local, mutation, &trace_tasks, now)?;
        }
        if next.local.handled_imports.keys().any(|id| {
            !previous.local.handled_imports.contains_key(id) && !imported.contains_key(id)
        }) {
            return Err(
                "Only an imported deadline currently in view can be marked handled.".into(),
            );
        }
        for (id, handled) in &mut next.local.handled_imports {
            if let Some(source) = imported.get(id) {
                handled.source_id = source.clone();
            }
        }
        next.local.updated_at = Some(now);
        next.local.revision = next
            .local
            .revision
            .get()
            .checked_add(1)
            .and_then(NonZeroU64::new)
            .ok_or_else(|| "local snapshot revision limit reached".to_string())?;
        next.metadata.revision = next_revision(next.metadata.revision)?;
        let candidate = build_input_with(&next, &parsed, now, zone)?;
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
        let parsed = parsed_calendars(
            &self.parsed,
            &tx,
            &previous.calendars,
            previous.settings.zone(),
        )?;
        let applied = trace::decode_json(json, now)
            .and_then(|export| reconcile(&previous, export, now))
            .and_then(|next| {
                let candidate = build_input_with(&next, &parsed, now, next.settings.zone())
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
        || cache
            .local
            .anchor_series
            .iter()
            .any(|record| record.meta.created_at > now || record.meta.updated_at > now)
        || cache
            .local
            .usual_availability
            .as_ref()
            .is_some_and(|record| record.meta.created_at > now || record.meta.updated_at > now)
        || cache
            .calendars
            .iter()
            .any(|c| c.state.last_attempt_at.is_some_and(|at| at > now))
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
        settings: load_settings(connection)?,
        calendars: load_calendars(connection)?,
    })
}

/// Parse (or reuse) every stored calendar's last-known text.
fn parsed_calendars(
    store: &RefCell<BTreeMap<SourceId, (NonZeroU64, Arc<ics::IcsCalendar>)>>,
    connection: &Connection,
    calendars: &[CalendarSource],
    zone: ZoneId,
) -> Result<Parsed, String> {
    let mut cache = store.borrow_mut();
    cache.retain(|id, _| calendars.iter().any(|c| c.source.id == *id));
    let fallback: chrono_tz::Tz = zone.name().parse().unwrap_or(chrono_tz::UTC);
    let mut parsed = BTreeMap::new();
    for calendar in calendars {
        let id = calendar.source.id;
        let hit = cache
            .get(&id)
            .filter(|(revision, _)| *revision == calendar.revision)
            .map(|(_, value)| value.clone());
        let value = match hit {
            Some(value) => value,
            None => {
                let text: String = connection
                    .query_row(
                        "SELECT ics FROM calendar_sources WHERE id=?1",
                        [id.to_string()],
                        |r| r.get(0),
                    )
                    .map_err(db_error)?;
                let value = Arc::new(if text.is_empty() {
                    empty_calendar()
                } else {
                    ics::parse(&text, fallback).map_err(|_| {
                        "A stored calendar could not be read; it was left unchanged.".to_string()
                    })?
                });
                cache.insert(id, (calendar.revision, value.clone()));
                value
            }
        };
        parsed.insert(id, value);
    }
    Ok(parsed)
}

fn load_calendars(connection: &Connection) -> Result<Vec<CalendarSource>, String> {
    let mut statement = connection
        .prepare("SELECT id,payload FROM calendar_sources ORDER BY id")
        .map_err(db_error)?;
    let rows = statement
        .query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, String>(1)?)))
        .map_err(db_error)?;
    let mut calendars = Vec::new();
    for row in rows {
        let (id, payload) = row.map_err(db_error)?;
        let source: CalendarSource = serde_json::from_str(&payload)
            .map_err(|_| "A stored calendar is invalid; no reset was attempted.".to_string())?;
        if source.source.id.to_string() != id || source.state.source_id != source.source.id {
            return Err(
                "A stored calendar identity is inconsistent; no reset was attempted.".into(),
            );
        }
        calendars.push(source);
    }
    Ok(calendars)
}

/// Write a calendar's metadata, and its text when a new version arrived.
fn save_calendar(
    connection: &Connection,
    source: &CalendarSource,
    text: Option<&str>,
) -> Result<(), String> {
    let id = source.source.id.to_string();
    let payload = encode(source)?;
    match text {
        Some(text) => connection
            .execute(
                "INSERT INTO calendar_sources(id,payload,ics) VALUES(?1,?2,?3) ON CONFLICT(id) DO UPDATE SET payload=excluded.payload, ics=excluded.ics",
                params![id, payload, text],
            )
            .map_err(db_error)?,
        None => connection
            .execute(
                "UPDATE calendar_sources SET payload=?2 WHERE id=?1",
                params![id, payload],
            )
            .map_err(db_error)?,
    };
    Ok(())
}

fn empty_calendar() -> ics::IcsCalendar {
    ics::IcsCalendar {
        name: None,
        prodid: None,
        default_zone: None,
        events: Vec::new(),
        skipped: 0,
    }
}

/// Imported deadline identities currently inside the evaluated slice, with
/// the source each belongs to.
fn imported_deadline_ids(
    cache: &Cache,
    parsed: &Parsed,
    now: Instant,
    zone: ZoneId,
) -> Result<BTreeMap<String, String>, String> {
    let rules = TimezoneRules::bundled();
    let from = rules
        .date_at(now, zone)
        .and_then(|date| rules.start_of_date(date, zone))
        .map_err(|e| e.to_string())?
        .checked_sub_ms(366 * 86_400_000)
        .map_err(|e| e.to_string())?;
    let to = now
        .checked_add_ms(EVALUATION_MS)
        .map_err(|e| e.to_string())?;
    let mut ids = BTreeMap::new();
    for source in &cache.calendars {
        let Some(calendar) = parsed.get(&source.source.id) else {
            continue;
        };
        for row in calendars::normalize(source, calendar, from, to, zone).0 {
            if let Imported::Deadline { deadline, .. } = row {
                ids.insert(deadline.meta.id.to_string(), source.source.id.to_string());
            }
        }
    }
    Ok(ids)
}

fn load_settings(connection: &Connection) -> Result<Settings, String> {
    let payload: String = connection
        .query_row(
            "SELECT payload FROM app_settings WHERE singleton=1",
            [],
            |r| r.get(0),
        )
        .map_err(db_error)?;
    serde_json::from_str(&payload)
        .map_err(|_| "Stored settings are invalid; no reset was attempted.".to_string())
}

fn save_settings(connection: &Connection, settings: &Settings) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO app_settings(singleton,payload) VALUES(1,?1) ON CONFLICT(singleton) DO UPDATE SET payload=excluded.payload",
            [encode(settings)?],
        )
        .map_err(db_error)?;
    Ok(())
}
fn build_input_with(
    cache: &Cache,
    parsed: &Parsed,
    now: Instant,
    zone: ZoneId,
) -> Result<EvaluationInput, String> {
    build_slice(cache, parsed, now, zone).map(|(input, _)| input)
}

/// The Horizon's evaluation input: facts relevant to today through the end of
/// the evaluated extent. Leaving older facts out of one evaluation never
/// deletes or rewrites them; they remain stored and visible in the Calendar.
fn build_slice(
    cache: &Cache,
    parsed: &Parsed,
    now: Instant,
    zone: ZoneId,
) -> Result<(EvaluationInput, Vec<SeriesLink>), String> {
    let rules = TimezoneRules::bundled();
    let end = now
        .checked_add_ms(EVALUATION_MS)
        .map_err(|e| e.to_string())?;
    let slice_start = rules
        .date_at(now, zone)
        .and_then(|date| rules.start_of_date(date, zone))
        .or_else(|_| now.checked_sub_ms(86_400_000))
        .map_err(|e| e.to_string())?;
    let range = TimedSpan::new(slice_start, end).map_err(|e| e.to_string())?;
    let local = &cache.local;

    // Tombstones within the range stay in the input, so removal remains explicit.
    let mut anchors: Vec<Anchor> = local
        .anchors
        .iter()
        .filter(|anchor| {
            anchor
                .span
                .resolve(&rules)
                .is_ok_and(|span| span.overlaps(&range))
        })
        .cloned()
        .collect();
    let mut occurrences = Vec::new();
    for record in &local.anchor_series {
        for occurrence in series::expand_series(record, local.source.id, slice_start, end)
            .map_err(|e| e.to_string())?
        {
            occurrences.push((
                occurrence.anchor.meta.id,
                occurrence.series_id,
                occurrence.date,
            ));
            anchors.push(occurrence.anchor);
        }
    }

    // Imported calendars: bounded occurrences with source-stable identity.
    // Unresolved past coursework stays in view (overdue) until the person
    // marks it handled; that acknowledgement never rewrites the source fact.
    let mut imported_deadlines = Vec::new();
    let mut sources = vec![cache.metadata.source.clone(), local.source.clone()];
    let mut source_states = vec![cache.metadata.state.clone()];
    let mut required_sources = vec![RequiredSource {
        source_id: cache.metadata.source.id,
        role: SourceRole::Tasks,
    }];
    let lookback = slice_start
        .checked_sub_ms(366 * 86_400_000)
        .map_err(|e| e.to_string())?;
    for source in cache.calendars.iter().filter(|c| !c.hidden) {
        let Some(calendar) = parsed.get(&source.source.id) else {
            continue;
        };
        sources.push(source.source.clone());
        source_states.push(source.state.clone());
        required_sources.extend(source.required());
        let (rows, _) = calendars::normalize(source, calendar, lookback, end, zone);
        for row in rows {
            match row {
                Imported::Anchor { anchor, .. } => {
                    if anchor
                        .span
                        .resolve(&rules)
                        .is_ok_and(|span| span.overlaps(&range))
                    {
                        anchors.push(anchor);
                    }
                }
                Imported::Deadline { deadline, .. } => {
                    if local
                        .handled_imports
                        .contains_key(&deadline.meta.id.to_string())
                    {
                        continue;
                    }
                    imported_deadlines.push(deadline);
                }
            }
        }
    }

    let task_done = |id: TaskRefId| {
        cache
            .tasks
            .values()
            .any(|t| t.task.meta.id == id && t.task.status == TaskStatus::Done)
    };
    let linked: std::collections::BTreeSet<DeadlineId> = cache
        .tasks
        .values()
        .filter_map(|t| match t.task.due {
            TaskDue::Deadline(id) => Some(id),
            _ => None,
        })
        .collect();
    let recent = slice_start
        .checked_sub_ms(RESOLVED_DEADLINE_LOOKBACK_MS)
        .map_err(|e| e.to_string())?;
    let deadlines: Vec<Deadline> = local
        .deadlines
        .iter()
        .filter(|deadline| {
            if linked.contains(&deadline.meta.id) {
                return true;
            }
            let unresolved = deadline.presence == Presence::Present
                && match &deadline.fulfillment {
                    Fulfillment::Recorded(RecordedResolution::Unresolved) => true,
                    Fulfillment::Recorded(_) => false,
                    Fulfillment::TraceTask(id) => !task_done(*id),
                };
            unresolved
                || deadline
                    .cutoff
                    .endpoint(&rules)
                    .is_ok_and(|at| at >= recent)
        })
        .cloned()
        .chain(imported_deadlines)
        .collect();

    let anchor_ids: std::collections::BTreeSet<AnchorId> =
        anchors.iter().map(|a| a.meta.id).collect();
    let deadline_ids: std::collections::BTreeSet<DeadlineId> =
        deadlines.iter().map(|d| d.meta.id).collect();

    let mut availability: Vec<AvailabilityDeclaration> = local
        .availability
        .iter()
        .filter(|declaration| declaration.span.overlaps(&range))
        .cloned()
        .collect();
    if let Some(usual) = &local.usual_availability {
        availability.extend(
            series::expand_usual(usual, &local.availability, slice_start, end)
                .map_err(|e| e.to_string())?,
        );
    }

    let input = EvaluationInput {
        schema_version: 1,
        snapshot_revision: cache.metadata.revision,
        captured_now: now,
        sources,
        source_states,
        required_sources,
        task_refs: cache.tasks.values().map(|r| r.task.clone()).collect(),
        anchors,
        deadlines,
        intentions: local.intentions.clone(),
        routines: local.routines.clone(),
        anchor_annotations: local
            .anchor_annotations
            .iter()
            .filter(|a| anchor_ids.contains(&a.target_id))
            .cloned()
            .collect(),
        deadline_annotations: local
            .deadline_annotations
            .iter()
            .filter(|a| deadline_ids.contains(&a.target_id))
            .cloned()
            .collect(),
        task_annotations: local.task_annotations.clone(),
        routine_outcomes: local.routine_outcomes.clone(),
        availability,
        prior_suggestions: vec![],
        evaluation: EvaluationRequest {
            now,
            evaluation_end: end,
            display_zone: zone,
            timezone_rules_version: TimezoneRules::bundled().version().into(),
            policy_version: PolicyVersion::ProofV1,
        },
    };
    Ok((input, occurrences))
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
