//! App boundary for the one local Trace cache. Time is captured outside the core.
use crate::{
    Scenario, Snapshot, calendar,
    calendars::{self, CalendarMode, CalendarOrigin, Imported},
    local::{LocalMutation, LocalState},
    presentation,
    settings::{Settings, SettingsPatch},
    store::Store,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use temporal_core::{domain::*, results::Health, time::*};

#[derive(Serialize)]
pub struct TraceInfo {
    exported_at: Option<String>,
    imported_at: Option<String>,
    present_tasks: usize,
    completed_tasks: usize,
    unresolved_dates: usize,
    last_attempt: AttemptOutcome,
    tasks: Vec<TraceChoice>,
    /// The export file last chosen in the desktop app, if any.
    export_path: Option<String>,
}
#[derive(Serialize)]
pub struct TraceChoice {
    id: TaskRefId,
    external_id: String,
    title: String,
    status: TaskStatus,
    presence: Presence,
}
#[derive(Serialize)]
pub struct PersonalView {
    pub view: Snapshot,
    trace: TraceInfo,
    pub local: LocalState,
    pub settings: Settings,
    pub calendars: Vec<CalendarSummary>,
    /// Editing context for Horizon items that are local or imported facts.
    pub links: BTreeMap<String, ItemLink>,
}
#[derive(Serialize)]
pub struct ImportResult {
    pub personal: PersonalView,
    pub error: Option<String>,
}

/// What the person can do with one Horizon item, without changing the item.
#[derive(Clone, Debug, Default, Serialize)]
pub struct ItemLink {
    pub editable: bool,
    pub series_id: Option<String>,
    pub occurrence_date: Option<String>,
    pub place: Option<String>,
    pub notes: Option<String>,
    pub imported: bool,
    pub source_id: Option<String>,
    pub color: Option<String>,
}

/// One imported calendar as the Sources page shows it. The subscription URL
/// is never included.
#[derive(Serialize)]
pub struct CalendarSummary {
    pub id: String,
    pub label: String,
    pub kind: &'static str,
    pub mode: CalendarMode,
    pub color: String,
    pub origin: &'static str,
    pub origin_label: String,
    pub health: Health,
    pub last_success: Option<String>,
    pub last_attempt: AttemptOutcome,
    pub last_error: Option<String>,
    pub calendar_name: Option<String>,
    pub event_count: usize,
    pub skipped: usize,
    pub hidden: bool,
    /// Past imported deadlines still unresolved and not marked handled.
    pub past_unhandled: Vec<String>,
}

pub fn view(store: &Store, now: Instant) -> Result<PersonalView, String> {
    let zone: ZoneId = store.settings()?.zone();
    let stored = store.read(now, zone)?;
    let input = &stored.input;
    let output = temporal_core::evaluate(input)
        .map_err(|_| "Cached temporal data could not be evaluated.".to_string())?;
    let current: Vec<_> = input
        .task_refs
        .iter()
        .filter(|t| t.presence == Presence::Present)
        .collect();
    let trace = TraceInfo {
        exported_at: stored.exported_at.map(|at| presentation::at(at, zone)),
        imported_at: stored.last_import_at.map(|at| presentation::at(at, zone)),
        present_tasks: current.len(),
        completed_tasks: current
            .iter()
            .filter(|t| t.status == TaskStatus::Done)
            .count(),
        unresolved_dates: current
            .iter()
            .filter(|t| matches!(t.due, TaskDue::Unresolved { .. }))
            .count(),
        last_attempt: input.source_states[0].last_attempt_outcome,
        tasks: input
            .task_refs
            .iter()
            .map(|task| TraceChoice {
                id: task.meta.id,
                external_id: task.provenance.imported().external_id.clone(),
                title: task.title.clone(),
                status: task.status,
                presence: task.presence,
            })
            .collect(),
        export_path: stored.settings.trace_export_path.clone(),
    };
    let scenario = Scenario {
        id: "personal".into(),
        title: "My time".into(),
        description: "Your local view. Trace keeps ownership of task text, dates, and completion."
            .into(),
    };
    let snapshot = presentation::project(scenario, input, &output, 0).map_err(|e| e.to_string())?;

    let mut links = BTreeMap::new();
    for anchor in &stored.local.anchors {
        let id = anchor.meta.id.to_string();
        let details = stored.local.event_details.get(&id);
        links.insert(
            id,
            ItemLink {
                editable: true,
                place: details.and_then(|d| d.place.clone()),
                notes: details.and_then(|d| d.notes.clone()),
                ..ItemLink::default()
            },
        );
    }
    for deadline in &stored.local.deadlines {
        let id = deadline.meta.id.to_string();
        let details = stored.local.event_details.get(&id);
        links.insert(
            id,
            ItemLink {
                editable: true,
                place: details.and_then(|d| d.place.clone()),
                notes: details.and_then(|d| d.notes.clone()),
                ..ItemLink::default()
            },
        );
    }
    for (anchor_id, series_id, date) in &stored.series_occurrences {
        let series = stored
            .local
            .anchor_series
            .iter()
            .find(|s| s.meta.id == *series_id);
        links.insert(
            anchor_id.to_string(),
            ItemLink {
                editable: true,
                series_id: Some(series_id.to_string()),
                occurrence_date: Some(date.to_string()),
                place: series.and_then(|s| s.details.place.clone()),
                notes: series.and_then(|s| s.details.notes.clone()),
                ..ItemLink::default()
            },
        );
    }
    let rules = TimezoneRules::bundled();
    let lookback = now
        .checked_sub_ms(366 * 86_400_000)
        .map_err(|e| e.to_string())?;
    let horizon_end = input.evaluation.evaluation_end;
    let mut calendars = Vec::new();
    for (source, parsed) in &stored.calendars {
        let (rows, _) = calendars::normalize(source, parsed, lookback, horizon_end, zone);
        let mut past_unhandled = Vec::new();
        for row in &rows {
            match row {
                Imported::Anchor { anchor, place, .. } => {
                    links.insert(
                        anchor.meta.id.to_string(),
                        ItemLink {
                            imported: true,
                            place: place.clone(),
                            source_id: Some(source.source.id.to_string()),
                            color: Some(source.color.clone()),
                            ..ItemLink::default()
                        },
                    );
                }
                Imported::Deadline { deadline, place } => {
                    let id = deadline.meta.id.to_string();
                    let handled = stored.local.handled_imports.contains_key(&id);
                    if !handled
                        && deadline
                            .cutoff
                            .endpoint(&rules)
                            .is_ok_and(|endpoint| endpoint <= now)
                    {
                        past_unhandled.push(id.clone());
                    }
                    links.insert(
                        id,
                        ItemLink {
                            imported: true,
                            place: place.clone(),
                            source_id: Some(source.source.id.to_string()),
                            color: Some(source.color.clone()),
                            ..ItemLink::default()
                        },
                    );
                }
            }
        }
        let health = output
            .source_health
            .iter()
            .find(|row| row.source_id == source.source.id)
            .map_or(Health::NeverLoaded, |row| row.health);
        let (origin, origin_label) = match &source.origin {
            CalendarOrigin::File { name } => ("file", name.clone()),
            CalendarOrigin::Subscription { host, .. } => ("subscription", host.clone()),
        };
        calendars.push(CalendarSummary {
            id: source.source.id.to_string(),
            label: source.source.label.clone(),
            kind: calendar::source_kind(source.source.kind),
            mode: source.mode,
            color: source.color.clone(),
            origin,
            origin_label,
            health: if source.hidden {
                Health::Healthy
            } else {
                health
            },
            last_success: source
                .state
                .last_success_at
                .map(|at| presentation::at(at, zone)),
            last_attempt: source.state.last_attempt_outcome,
            last_error: source.last_error.clone(),
            calendar_name: source.calendar_name.clone(),
            event_count: source.event_count,
            skipped: source.skipped,
            hidden: source.hidden,
            past_unhandled,
        });
    }
    Ok(PersonalView {
        view: snapshot,
        trace,
        local: stored.local,
        settings: stored.settings,
        calendars,
        links,
    })
}

pub fn import(store: &mut Store, json: &str, now: Instant) -> Result<ImportResult, String> {
    let error = store.import_json(json, now).err();
    Ok(ImportResult {
        personal: view(store, now)?,
        error,
    })
}

#[cfg(feature = "desktop")]
pub fn system_instant() -> Result<Instant, String> {
    let elapsed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| "System clock is before the supported application epoch.".to_string())?;
    let milliseconds = i64::try_from(elapsed.as_millis())
        .map_err(|_| "System clock is out of range.".to_string())?;
    Instant::from_epoch_ms(milliseconds).map_err(|e| e.to_string())
}

#[cfg(feature = "desktop")]
#[derive(Default)]
pub struct AppStore(std::sync::Mutex<Option<Store>>);

#[cfg(feature = "desktop")]
impl AppStore {
    pub fn with<T>(
        &self,
        app: &tauri::AppHandle,
        action: impl FnOnce(&mut Store) -> Result<T, String>,
    ) -> Result<T, String> {
        use tauri::Manager;
        let mut guard = self
            .0
            .lock()
            .map_err(|_| "Local cache is unavailable.".to_string())?;
        if guard.is_none() {
            let directory = app
                .path()
                .app_data_dir()
                .map_err(|_| "Local app-data directory is unavailable.".to_string())?;
            std::fs::create_dir_all(&directory).map_err(|_| {
                "Could not create Temporal Engine's own data directory.".to_string()
            })?;
            *guard = Some(Store::open(&directory.join("temporal-engine.sqlite3"))?);
        }
        action(guard.as_mut().expect("opened store"))
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RangeArgs {
    from: i64,
    to: i64,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MutationArgs {
    mutation: LocalMutation,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct BatchArgs {
    mutations: Vec<LocalMutation>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SettingsArgs {
    patch: SettingsPatch,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TraceArgs {
    json: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalendarTextArgs {
    pub label: String,
    pub kind: SourceKind,
    pub mode: CalendarMode,
    pub color: String,
    pub file_name: String,
    pub text: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RefreshTextArgs {
    id: String,
    text: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalendarPatchArgs {
    pub id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub mode: Option<CalendarMode>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub hidden: Option<bool>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IdArgs {
    pub id: String,
}

fn args<T: serde::de::DeserializeOwned>(value: serde_json::Value) -> Result<T, String> {
    serde_json::from_value(value).map_err(|e| format!("Invalid request: {e}"))
}
fn json(value: impl Serialize) -> Result<serde_json::Value, String> {
    serde_json::to_value(value).map_err(|_| "Could not encode the response.".to_string())
}
pub fn parse_source_id(value: &str) -> Result<SourceId, String> {
    value
        .parse()
        .map_err(|_| "Unknown calendar identity.".to_string())
}

/// The one personal command surface, shared by the desktop app and the
/// development preview. Network, dialogs, and credentials stay outside it.
pub fn call(
    store: &mut Store,
    now: Instant,
    command: &str,
    value: serde_json::Value,
) -> Result<serde_json::Value, String> {
    match command {
        "personal_snapshot" => json(view(store, now)?),
        "calendar_range" => {
            let range: RangeArgs = args(value)?;
            let from = Instant::from_epoch_ms(range.from).map_err(|e| e.to_string())?;
            let to = Instant::from_epoch_ms(range.to).map_err(|e| e.to_string())?;
            let zone = store.settings()?.zone();
            let stored = store.read(now, zone)?;
            json(calendar::range(&stored, now, from, to)?)
        }
        "mutate_local" => {
            let input: MutationArgs = args(value)?;
            store.mutate_local(&input.mutation, now)?;
            json(view(store, now)?)
        }
        "mutate_local_batch" => {
            let input: BatchArgs = args(value)?;
            store.mutate_local_batch(&input.mutations, now)?;
            json(view(store, now)?)
        }
        "update_settings" => {
            let input: SettingsArgs = args(value)?;
            store.update_settings(&input.patch)?;
            json(view(store, now)?)
        }
        "import_trace_json" => {
            let input: TraceArgs = args(value)?;
            json(import(store, &input.json, now)?)
        }
        "add_calendar_text" => {
            let input: CalendarTextArgs = args(value)?;
            store.add_calendar(
                &input.label,
                input.kind,
                input.mode,
                &input.color,
                CalendarOrigin::File {
                    name: input.file_name.chars().take(120).collect(),
                },
                &input.text,
                now,
            )?;
            json(view(store, now)?)
        }
        "refresh_calendar_text" => {
            let input: RefreshTextArgs = args(value)?;
            let id = parse_source_id(&input.id)?;
            let error = store.refresh_calendar(id, Ok(input.text), now).err();
            json(ImportResult {
                personal: view(store, now)?,
                error,
            })
        }
        "update_calendar" => {
            let input: CalendarPatchArgs = args(value)?;
            store.update_calendar(
                parse_source_id(&input.id)?,
                input.label.as_deref(),
                input.mode,
                input.color.as_deref(),
                input.hidden,
            )?;
            json(view(store, now)?)
        }
        "remove_calendar" => {
            let input: IdArgs = args(value)?;
            store.remove_calendar(parse_source_id(&input.id)?)?;
            json(view(store, now)?)
        }
        _ => Err(format!("Unknown command {command}.")),
    }
}
