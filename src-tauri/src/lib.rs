//! Local application boundary: synthetic examples and an app-owned Trace cache.
pub mod local;
pub mod personal;
mod presentation;
pub mod store;
pub mod today;
pub mod trace;

// Reuse the exact Gate 1 inputs internally, without copying or inventing data.
#[path = "../../crates/temporal-core/tests/support/bases.rs"]
mod bases;
#[allow(dead_code)]
#[path = "../../crates/temporal-core/tests/support/builder.rs"]
mod builder;

pub use presentation::Snapshot;
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct Scenario {
    pub id: String,
    pub title: String,
    pub description: String,
}

pub fn catalog() -> Vec<Scenario> {
    [
        (
            "S01",
            "Almost empty week",
            "A quiet week, one appointment, and a small flexible task.",
        ),
        (
            "S02",
            "Class-heavy week",
            "Fixed classes and the different kinds of work that fit between them.",
        ),
        (
            "S03",
            "A large assignment",
            "Advance a day to see pressure change while the estimate stays the same.",
        ),
        (
            "S04",
            "Many small deadlines",
            "Individual fit does not promise that everything fits together.",
        ),
        (
            "S05",
            "Conflicting anchors",
            "Two distinct seminars overlap; both sources remain visible.",
        ),
        (
            "S06",
            "Too little opportunity",
            "Four hours of work and only two declared hours before its deadline.",
        ),
        (
            "S07",
            "Soft intentions",
            "Preferences stay optional. An empty calendar does not establish availability.",
        ),
        (
            "S08",
            "Imported and local",
            "An imported anchor, a local deadline, and an optional intention.",
        ),
        (
            "S09",
            "A real overdue deadline",
            "A real cutoff has passed; current opportunity cannot move it.",
        ),
        (
            "S10",
            "An expired suggestion",
            "Advance one day: the suggestion expires and the task stays flexible.",
        ),
        (
            "S12",
            "Source freshness",
            "Advance an hour to see last-known source data become qualified.",
        ),
        (
            "S14",
            "A flexible routine",
            "Weekly preferences roll forward without creating missed-work debt.",
        ),
    ]
    .into_iter()
    .map(|(id, title, description)| Scenario {
        id: id.into(),
        title: title.into(),
        description: description.into(),
    })
    .collect()
}

pub fn snapshot(id: &str, offset_minutes: u32) -> Result<Snapshot, String> {
    let scenario = catalog()
        .into_iter()
        .find(|s| s.id == id)
        .ok_or("Unknown synthetic scenario")?;
    let mut input = bases::all()
        .into_iter()
        .find(|case| case.name == format!("{id}/base"))
        .ok_or("Synthetic input unavailable")?
        .input;
    let now = input
        .captured_now
        .checked_add_ms(u64::from(offset_minutes) * 60_000)
        .map_err(|e| e.to_string())?;
    if now >= input.evaluation.evaluation_end {
        return Err("The virtual clock must stay within this snapshot's 14-day range".into());
    }
    input.evaluation.now = now;
    let output = temporal_core::evaluate(&input).map_err(|e| e.to_string())?;
    presentation::project(scenario, &input, &output, offset_minutes).map_err(|e| e.to_string())
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn scenario_catalog() -> Vec<Scenario> {
    catalog()
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn evaluate_scenario(scenario_id: String, offset_minutes: u32) -> Result<Snapshot, String> {
    snapshot(&scenario_id, offset_minutes)
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn personal_snapshot(
    app: tauri::AppHandle,
    state: tauri::State<'_, personal::AppStore>,
) -> Result<personal::PersonalView, String> {
    let now = personal::system_instant()?;
    state.with(&app, |store| personal::view(store, now))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn import_trace_json(
    app: tauri::AppHandle,
    state: tauri::State<'_, personal::AppStore>,
    json: String,
) -> Result<personal::ImportResult, String> {
    let now = personal::system_instant()?;
    state.with(&app, |store| personal::import(store, &json, now))
}

#[cfg(feature = "desktop")]
#[tauri::command]
fn mutate_local(
    app: tauri::AppHandle,
    state: tauri::State<'_, personal::AppStore>,
    mutation: local::LocalMutation,
) -> Result<personal::PersonalView, String> {
    let now = personal::system_instant()?;
    state.with(&app, |store| {
        store.mutate_local(&mutation, now)?;
        personal::view(store, now)
    })
}

#[cfg(feature = "desktop")]
pub fn run() {
    tauri::Builder::default()
        .manage(personal::AppStore::default())
        .invoke_handler(tauri::generate_handler![
            scenario_catalog,
            evaluate_scenario,
            personal_snapshot,
            import_trace_json,
            mutate_local
        ])
        .run(tauri::generate_context!())
        .expect("unable to start Temporal Engine");
}
