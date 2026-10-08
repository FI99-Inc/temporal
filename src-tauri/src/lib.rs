//! Local application boundary: synthetic examples and an app-owned Trace cache.
pub mod calendar;
pub mod calendars;
pub mod credentials;
#[cfg(feature = "desktop")]
mod fetch;
pub mod ics;
pub mod local;
pub mod personal;
mod presentation;
pub mod reminders;
pub mod series;
pub mod settings;
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
mod desktop {
    use crate::{
        calendars::{CalendarMode, CalendarOrigin},
        credentials, fetch, personal, reminders,
    };
    use serde::Deserialize;
    use std::{
        collections::BTreeSet,
        sync::Mutex,
        time::{Duration, SystemTime},
    };
    use tauri::{Emitter, Manager};
    use temporal_core::domain::SourceKind;

    /// Subscriptions refresh at most this often in the background.
    const FEED_REFRESH: Duration = Duration::from_secs(3 * 3600);

    /// Whether the notification-area icon exists, so hiding is reversible.
    struct TrayReady(bool);

    pub struct Runtime {
        vault: Box<dyn credentials::Vault>,
        shown: Mutex<BTreeSet<String>>,
        trace_seen: Mutex<Option<(String, SystemTime)>>,
    }

    impl Default for Runtime {
        fn default() -> Self {
            Self {
                vault: credentials::system(),
                shown: Mutex::new(BTreeSet::new()),
                trace_seen: Mutex::new(None),
            }
        }
    }

    #[tauri::command]
    fn scenario_catalog() -> Vec<crate::Scenario> {
        crate::catalog()
    }

    #[tauri::command]
    fn evaluate_scenario(
        scenario_id: String,
        offset_minutes: u32,
    ) -> Result<crate::Snapshot, String> {
        crate::snapshot(&scenario_id, offset_minutes)
    }

    /// The shared personal command surface (see `personal::call`).
    #[tauri::command(async)]
    fn app_call(
        app: tauri::AppHandle,
        state: tauri::State<'_, personal::AppStore>,
        runtime: tauri::State<'_, Runtime>,
        command: String,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let now = personal::system_instant()?;
        state.with(&app, |store| {
            let credential = if command == "remove_calendar" {
                let id: personal::IdArgs = serde_json::from_value(args.clone())
                    .map_err(|e| format!("Invalid request: {e}"))?;
                store
                    .calendars()?
                    .into_iter()
                    .find(|c| c.source.id.to_string() == id.id)
                    .and_then(|c| match c.origin {
                        CalendarOrigin::Subscription { credential, .. } => Some(credential),
                        CalendarOrigin::File { .. } => None,
                    })
            } else {
                None
            };
            let result = personal::call(store, now, &command, args)?;
            if let Some(key) = credential {
                runtime.vault.remove(&key)?;
            }
            if command == "update_settings" {
                apply_autostart(&app, store.settings()?.open_at_login);
            }
            Ok(result)
        })
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SubscriptionArgs {
        label: String,
        kind: SourceKind,
        mode: CalendarMode,
        color: String,
        url: String,
    }

    /// Add a subscription: fetch it, store the link in the OS credential
    /// store, and keep only the host for display.
    #[tauri::command(async)]
    fn add_calendar_url(
        app: tauri::AppHandle,
        state: tauri::State<'_, personal::AppStore>,
        runtime: tauri::State<'_, Runtime>,
        input: SubscriptionArgs,
    ) -> Result<personal::PersonalView, String> {
        let (url, host) = credentials::normalize_feed_url(&input.url)?;
        let text = fetch::fetch_feed(&url).map_err(|(_, message)| message)?;
        let key = uuid::Uuid::new_v4().to_string();
        runtime.vault.store(&key, &url)?;
        let now = personal::system_instant()?;
        let added = state.with(&app, |store| {
            store.add_calendar(
                &input.label,
                input.kind,
                input.mode,
                &input.color,
                CalendarOrigin::Subscription {
                    credential: key.clone(),
                    host,
                },
                &text,
                now,
            )?;
            personal::view(store, now)
        });
        if added.is_err() {
            let _ = runtime.vault.remove(&key);
        }
        added
    }

    /// Refresh one subscription now (or every subscription when `id` is None).
    #[tauri::command(async)]
    fn refresh_calendars(
        app: tauri::AppHandle,
        state: tauri::State<'_, personal::AppStore>,
        runtime: tauri::State<'_, Runtime>,
        id: Option<String>,
    ) -> Result<personal::ImportResult, String> {
        let error = refresh_feeds(&app, &state, &runtime, id.as_deref());
        let now = personal::system_instant()?;
        state.with(&app, |store| {
            Ok(personal::ImportResult {
                personal: personal::view(store, now)?,
                error: error.clone(),
            })
        })
    }

    /// Fetch outside the store lock; apply each result inside it.
    fn refresh_feeds(
        app: &tauri::AppHandle,
        state: &personal::AppStore,
        runtime: &Runtime,
        only: Option<&str>,
    ) -> Option<String> {
        let calendars = match state.with(app, |store| store.calendars()) {
            Ok(calendars) => calendars,
            Err(error) => return Some(error),
        };
        let mut last_error = None;
        for calendar in calendars {
            if only.is_some_and(|id| id != calendar.source.id.to_string()) {
                continue;
            }
            let CalendarOrigin::Subscription { credential, .. } = &calendar.origin else {
                continue;
            };
            let fetched = match runtime.vault.load(credential) {
                Ok(Some(url)) => fetch::fetch_feed(&url),
                Ok(None) => Err((
                    temporal_core::domain::AttemptOutcome::Failed,
                    "The saved link is missing from Windows Credential Manager. Remove and re-add this calendar."
                        .to_string(),
                )),
                Err(message) => Err((temporal_core::domain::AttemptOutcome::Failed, message)),
            };
            let Ok(now) = personal::system_instant() else {
                continue;
            };
            let applied = state.with(app, |store| {
                store.refresh_calendar(calendar.source.id, fetched, now)
            });
            if let Err(error) = applied {
                last_error = Some(format!("{}: {error}", calendar.source.label));
            }
        }
        last_error
    }

    /// Choose a Trace export file; it is re-read whenever it changes.
    #[tauri::command]
    async fn pick_trace_export(
        app: tauri::AppHandle,
        state: tauri::State<'_, personal::AppStore>,
        runtime: tauri::State<'_, Runtime>,
    ) -> Result<Option<personal::ImportResult>, String> {
        use tauri_plugin_dialog::DialogExt;
        let picker = app.clone();
        let chosen = tauri::async_runtime::spawn_blocking(move || {
            picker
                .dialog()
                .file()
                .set_title("Choose your Trace JSON export")
                .add_filter("Trace export", &["json"])
                .blocking_pick_file()
        })
        .await
        .map_err(|_| "The file dialog closed unexpectedly.".to_string())?;
        let Some(path) = chosen.and_then(|p| p.into_path().ok()) else {
            return Ok(None);
        };
        let path = path.to_string_lossy().to_string();
        state.with(&app, |store| store.remember_trace_path(Some(path.clone())))?;
        *runtime.trace_seen.lock().map_err(|_| "busy")? = None;
        read_trace(&app, &state, &runtime, true)
    }

    /// Re-read the chosen Trace export if it changed since the last read.
    #[tauri::command(async)]
    fn reimport_trace(
        app: tauri::AppHandle,
        state: tauri::State<'_, personal::AppStore>,
        runtime: tauri::State<'_, Runtime>,
    ) -> Result<Option<personal::ImportResult>, String> {
        read_trace(&app, &state, &runtime, false)
    }

    fn read_trace(
        app: &tauri::AppHandle,
        state: &personal::AppStore,
        runtime: &Runtime,
        force: bool,
    ) -> Result<Option<personal::ImportResult>, String> {
        let Some(path) = state.with(app, |store| Ok(store.settings()?.trace_export_path))? else {
            return Ok(None);
        };
        let modified = std::fs::metadata(&path)
            .and_then(|m| m.modified())
            .map_err(|_| "The chosen Trace export is no longer readable.".to_string())?;
        {
            let seen = runtime.trace_seen.lock().map_err(|_| "busy")?;
            if !force && seen.as_ref() == Some(&(path.clone(), modified)) {
                return Ok(None);
            }
        }
        let bytes = std::fs::read(&path)
            .map_err(|_| "The chosen Trace export could not be read.".to_string())?;
        if bytes.len() > crate::trace::MAX_BYTES {
            return Err("Trace export exceeds 10 MiB. Previous snapshot kept.".into());
        }
        let json = String::from_utf8(bytes)
            .map_err(|_| "The Trace export is not UTF-8 text.".to_string())?;
        let now = personal::system_instant()?;
        let result = state.with(app, |store| personal::import(store, &json, now))?;
        *runtime.trace_seen.lock().map_err(|_| "busy")? = Some((path, modified));
        Ok(Some(result))
    }

    #[tauri::command]
    fn forget_trace_export(
        app: tauri::AppHandle,
        state: tauri::State<'_, personal::AppStore>,
    ) -> Result<(), String> {
        state.with(&app, |store| store.remember_trace_path(None))
    }

    /// Reminders and subscription refresh. Runs while the app runs.
    fn background(app: tauri::AppHandle) {
        std::thread::spawn(move || {
            let mut last_refresh: Option<std::time::Instant> = None;
            loop {
                let state = app.state::<personal::AppStore>();
                let runtime = app.state::<Runtime>();
                if last_refresh.is_none_or(|at| at.elapsed() >= FEED_REFRESH) {
                    last_refresh = Some(std::time::Instant::now());
                    let has_feeds = state
                        .with(&app, |store| {
                            Ok(store
                                .calendars()?
                                .iter()
                                .any(|c| matches!(c.origin, CalendarOrigin::Subscription { .. })))
                        })
                        .unwrap_or(false);
                    if has_feeds {
                        refresh_feeds(&app, &state, &runtime, None);
                        let _ = app.emit("temporal://changed", ());
                    }
                }
                notify_due(&app, &state, &runtime);
                std::thread::sleep(Duration::from_secs(20));
            }
        });
    }

    fn notify_due(app: &tauri::AppHandle, state: &personal::AppStore, runtime: &Runtime) {
        use tauri_plugin_notification::NotificationExt;
        let Ok(now) = personal::system_instant() else {
            return;
        };
        let Ok(mut shown) = runtime.shown.lock() else {
            return;
        };
        let due = state.with(app, |store| {
            let settings = store.settings()?;
            let Some(lead) = settings.reminder_minutes else {
                return Ok(Vec::new());
            };
            let zone = settings.zone();
            let stored = store.read(now, zone)?;
            Ok(reminders::due(&stored.input, now, lead, &shown, |at| {
                crate::presentation::clock(at, zone)
            }))
        });
        for reminder in due.unwrap_or_default() {
            let _ = app
                .notification()
                .builder()
                .title(&reminder.title)
                .body(&reminder.body)
                .show();
            shown.insert(reminder.key);
        }
        if shown.len() > 5000 {
            shown.clear();
        }
    }

    /// Keep the OS sign-in entry in step with the setting. Failure leaves the
    /// app usable; the setting simply has no effect on this machine.
    fn apply_autostart(app: &tauri::AppHandle, enabled: bool) {
        use tauri_plugin_autostart::ManagerExt;
        let manager = app.autolaunch();
        if manager.is_enabled().unwrap_or(false) != enabled {
            let _ = if enabled {
                manager.enable()
            } else {
                manager.disable()
            };
        }
    }

    fn show_main(app: &tauri::AppHandle) {
        if let Some(window) = app.get_webview_window("main") {
            let _ = window.show();
            let _ = window.unminimize();
            let _ = window.set_focus();
        }
    }

    pub fn run() {
        tauri::Builder::default()
            // A second launch focuses the running app instead of starting
            // another store owner, tray icon, and reminder loop.
            .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
                show_main(app);
            }))
            .plugin(tauri_plugin_autostart::init(
                tauri_plugin_autostart::MacosLauncher::LaunchAgent,
                Some(vec!["--background"]),
            ))
            .plugin(tauri_plugin_dialog::init())
            .plugin(tauri_plugin_notification::init())
            .manage(personal::AppStore::default())
            .manage(Runtime::default())
            .invoke_handler(tauri::generate_handler![
                scenario_catalog,
                evaluate_scenario,
                app_call,
                add_calendar_url,
                refresh_calendars,
                pick_trace_export,
                reimport_trace,
                forget_trace_export
            ])
            .setup(|app| {
                use tauri::{
                    menu::{Menu, MenuItem},
                    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
                };
                let open =
                    MenuItem::with_id(app, "open", "Open Temporal Engine", true, None::<&str>)?;
                let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;
                let menu = Menu::with_items(app, &[&open, &quit])?;
                let mut tray = TrayIconBuilder::with_id("main")
                    .tooltip("Temporal Engine")
                    .menu(&menu)
                    .show_menu_on_left_click(false)
                    .on_menu_event(|app, event| match event.id().as_ref() {
                        "open" => show_main(app),
                        "quit" => app.exit(0),
                        _ => {}
                    })
                    .on_tray_icon_event(|tray, event| {
                        if let TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        } = event
                        {
                            show_main(tray.app_handle());
                        }
                    });
                if let Some(icon) = app.default_window_icon() {
                    tray = tray.icon(icon.clone());
                }
                // Without a tray the app still works; the window then simply
                // closes instead of hiding (see the close handler).
                let tray_ready = tray.build(app).is_ok();
                app.manage(TrayReady(tray_ready));
                let handle = app.handle().clone();
                let login = handle
                    .state::<personal::AppStore>()
                    .with(&handle, |store| Ok(store.settings()?.open_at_login))
                    .unwrap_or(false);
                apply_autostart(&handle, login);
                // Started at sign-in: stay in the notification area until opened.
                if tray_ready
                    && std::env::args().any(|arg| arg == "--background")
                    && let Some(window) = app.get_webview_window("main")
                {
                    let _ = window.hide();
                }
                background(app.handle().clone());
                Ok(())
            })
            .on_window_event(|window, event| {
                if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                    let app = window.app_handle();
                    let keep = app.state::<TrayReady>().0
                        && app
                            .state::<personal::AppStore>()
                            .with(app, |store| Ok(store.settings()?.keep_running_in_tray))
                            .unwrap_or(false);
                    if keep {
                        api.prevent_close();
                        let _ = window.hide();
                    }
                }
            })
            .run(tauri::generate_context!())
            .expect("unable to start Temporal Engine");
    }
}

#[cfg(feature = "desktop")]
pub fn run() {
    desktop::run();
}
