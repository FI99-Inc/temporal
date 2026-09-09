//! App boundary for the one local Trace cache. Time is captured outside the core.
use crate::{Scenario, Snapshot, local::LocalState, presentation, store::Store};
use serde::Serialize;
use temporal_core::{domain::*, time::*};

#[derive(Serialize)]
pub struct TraceInfo {
    exported_at: Option<String>,
    imported_at: Option<String>,
    present_tasks: usize,
    completed_tasks: usize,
    unresolved_dates: usize,
    last_attempt: AttemptOutcome,
    tasks: Vec<TraceChoice>,
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
}
#[derive(Serialize)]
pub struct ImportResult {
    pub personal: PersonalView,
    pub error: Option<String>,
}

pub fn view(store: &Store, now: Instant) -> Result<PersonalView, String> {
    let zone: ZoneId = "America/Toronto".parse().unwrap();
    let stored = store.read(now, zone)?;
    let input = stored.input;
    let output = temporal_core::evaluate(&input)
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
    };
    let scenario = Scenario {
        id: "personal".into(),
        title: "My time".into(),
        description: "Your local view. Trace keeps ownership of task text, dates, and completion."
            .into(),
    };
    let view = presentation::project(scenario, &input, &output, 0).map_err(|e| e.to_string())?;
    Ok(PersonalView {
        view,
        trace,
        local: stored.local,
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
