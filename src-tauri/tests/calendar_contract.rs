//! Gate 4: settings, the bounded evaluation slice, repeating local events,
//! and usual availability. All inputs are synthetic.
use temporal_app::{
    local::{BlockInput, LocalAction, LocalKind, LocalMutation},
    series::SeriesFrequency,
    settings::{SettingsPatch, WeekStart},
    store::Store,
};
use temporal_core::domain::*;
use temporal_core::time::{Instant, TemporalSpan, TimedSpan};

fn now() -> Instant {
    // Wednesday 2026-09-09, 08:00 in Toronto.
    "2026-09-09T12:00:00.000Z".parse().unwrap()
}

fn read(store: &Store, at: Instant) -> EvaluationInput {
    store
        .read(at, "America/Toronto".parse().unwrap())
        .unwrap()
        .input
}

fn anchor(title: &str, start: &str, end: &str) -> LocalMutation {
    let mut mutation = LocalMutation::upsert(LocalKind::Anchor);
    mutation.title = Some(title.into());
    mutation.start = Some(start.into());
    mutation.end = Some(end.into());
    mutation.occupancy = Some(Occupancy::Busy);
    mutation
}

fn weekly_series(title: &str, start: &str, end: &str, days: &[Weekday]) -> LocalMutation {
    let mut mutation = LocalMutation::upsert(LocalKind::AnchorSeries);
    mutation.title = Some(title.into());
    mutation.start = Some(start.into());
    mutation.end = Some(end.into());
    mutation.occupancy = Some(Occupancy::Busy);
    mutation.frequency = Some(SeriesFrequency::Weekly);
    mutation.weekdays = days.to_vec();
    mutation
}

fn timed_start(anchor: &Anchor) -> String {
    match &anchor.span {
        TemporalSpan::Timed(span) => span.start().to_string(),
        TemporalSpan::Dates(span) => span.start_date().to_string(),
    }
}

#[test]
fn settings_persist_and_choose_the_display_zone() {
    let path = std::env::temp_dir().join(format!(
        "temporal-settings-{}.sqlite3",
        uuid::Uuid::new_v4()
    ));
    let mut store = Store::open(&path).unwrap();
    assert_eq!(store.settings().unwrap().display_zone, None);
    let updated = store
        .update_settings(&SettingsPatch {
            display_zone: Some("Europe/Paris".into()),
            week_starts_on: Some(WeekStart::Sun),
            reminder_minutes: Some(15),
            ..SettingsPatch::default()
        })
        .unwrap();
    assert_eq!(updated.zone().name(), "Europe/Paris");
    assert!(
        store
            .update_settings(&SettingsPatch {
                display_zone: Some("Not/AZone".into()),
                ..SettingsPatch::default()
            })
            .is_err()
    );
    drop(store);
    let reopened = Store::open(&path).unwrap();
    let settings = reopened.settings().unwrap();
    assert_eq!(settings.display_zone.as_deref(), Some("Europe/Paris"));
    assert_eq!(settings.week_starts_on, WeekStart::Sun);
    assert_eq!(settings.reminder_minutes, Some(15));
    let view = temporal_app::personal::view(&reopened, now()).unwrap();
    assert_eq!(
        serde_json::to_value(&view.view).unwrap()["zone"],
        "Europe/Paris"
    );
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn the_slice_keeps_relevant_facts_without_deleting_older_ones() {
    let mut store = Store::memory().unwrap();
    let earlier = "2026-09-01T12:00:00.000Z".parse::<Instant>().unwrap();
    let mut old = anchor(
        "Last week's seminar",
        "2026-09-02T09:00",
        "2026-09-02T10:00",
    );
    old.milestone = Some(true);
    store.mutate_local(&old, earlier).unwrap();
    store
        .mutate_local(
            &anchor("Today's lab", "2026-09-09T07:00", "2026-09-09T07:30"),
            earlier,
        )
        .unwrap();
    store
        .mutate_local(
            &anchor("Far future", "2026-10-30T09:00", "2026-10-30T10:00"),
            earlier,
        )
        .unwrap();
    let mut overdue = LocalMutation::upsert(LocalKind::Deadline);
    overdue.title = Some("Overdue form".into());
    overdue.due = Some("2026-09-03T17:00".into());
    store.mutate_local(&overdue, earlier).unwrap();
    let mut settled = LocalMutation::upsert(LocalKind::Deadline);
    settled.title = Some("Settled long ago".into());
    settled.due = Some("2026-08-01T17:00".into());
    store.mutate_local(&settled, earlier).unwrap();
    let settled_id = store
        .read(earlier, "America/Toronto".parse().unwrap())
        .unwrap()
        .local
        .deadlines
        .iter()
        .find(|d| d.title == "Settled long ago")
        .unwrap()
        .meta
        .id
        .to_string();
    let mut satisfy = LocalMutation::upsert(LocalKind::Deadline);
    satisfy.id = Some(settled_id);
    satisfy.completion = Some(temporal_app::local::LocalCompletion::Satisfied);
    store.mutate_local(&satisfy, earlier).unwrap();

    let stored = store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap();
    let titles: Vec<&str> = stored
        .input
        .anchors
        .iter()
        .map(|a| a.title.as_str())
        .collect();
    // Today's earlier lab stays (it falls on today); last week and beyond the extent do not.
    assert_eq!(titles, ["Today's lab"]);
    assert!(stored.input.anchor_annotations.is_empty());
    let deadlines: Vec<&str> = stored
        .input
        .deadlines
        .iter()
        .map(|d| d.title.as_str())
        .collect();
    assert_eq!(deadlines, ["Overdue form"]);
    // Nothing was deleted from storage.
    assert_eq!(stored.local.anchors.len(), 3);
    assert_eq!(stored.local.deadlines.len(), 2);
    assert_eq!(stored.local.anchor_annotations.len(), 1);
    temporal_core::evaluate(&stored.input).unwrap();
}

#[test]
fn a_weekly_series_expands_into_stable_individual_anchors() {
    let mut store = Store::memory().unwrap();
    let mut series = weekly_series(
        "Synthetic lecture",
        "2026-09-08T09:00",
        "2026-09-08T10:30",
        &[Weekday::Tue, Weekday::Thu],
    );
    series.place = Some("Room 101".into());
    store.mutate_local(&series, now()).unwrap();
    let first = store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap();
    // 14-day extent from Wed Sep 9: Thu 10, Tue 15, Thu 17, Tue 22 (Tue 8 is before today).
    let starts: Vec<String> = first.input.anchors.iter().map(timed_start).collect();
    assert_eq!(
        starts,
        [
            "2026-09-10T13:00:00.000Z",
            "2026-09-15T13:00:00.000Z",
            "2026-09-17T13:00:00.000Z",
            "2026-09-22T13:00:00.000Z"
        ]
    );
    assert_eq!(first.series_occurrences.len(), 4);
    assert_eq!(
        first.local.anchor_series[0].details.place.as_deref(),
        Some("Room 101")
    );
    let again = read(&store, now());
    assert_eq!(
        first
            .input
            .anchors
            .iter()
            .map(|a| a.meta.id)
            .collect::<Vec<_>>(),
        again.anchors.iter().map(|a| a.meta.id).collect::<Vec<_>>()
    );

    // Skipping one occurrence removes only that date and keeps the others' identity.
    let series_id = first.local.anchor_series[0].meta.id.to_string();
    let mut skip = LocalMutation::upsert(LocalKind::SeriesSkip);
    skip.id = Some(series_id.clone());
    skip.occurrence_date = Some("2026-09-15".into());
    store.mutate_local(&skip, now()).unwrap();
    let skipped = read(&store, now());
    assert_eq!(skipped.anchors.len(), 3);
    assert!(
        skipped
            .anchors
            .iter()
            .all(|a| first.input.anchors.iter().any(|b| b.meta.id == a.meta.id))
    );
    assert!(store.mutate_local(&skip, now()).is_err());

    // An end date ends the series; editing keeps the series identity.
    let mut edit = LocalMutation::upsert(LocalKind::AnchorSeries);
    edit.id = Some(series_id.clone());
    edit.frequency = Some(SeriesFrequency::Weekly);
    edit.weekdays = vec![Weekday::Tue, Weekday::Thu];
    edit.until_date_exclusive = Some("2026-09-16".into());
    store.mutate_local(&edit, now()).unwrap();
    let ended = store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap();
    assert_eq!(ended.input.anchors.len(), 1);
    assert_eq!(ended.local.anchor_series[0].meta.id.to_string(), series_id);
    assert_eq!(ended.local.anchor_series[0].title, "Synthetic lecture");

    let mut remove = LocalMutation::upsert(LocalKind::AnchorSeries);
    remove.action = LocalAction::Remove;
    remove.id = Some(series_id);
    store.mutate_local(&remove, now()).unwrap();
    assert!(read(&store, now()).anchors.is_empty());
}

#[test]
fn a_batch_is_atomic() {
    let mut store = Store::memory().unwrap();
    store
        .mutate_local(
            &weekly_series(
                "Synthetic studio",
                "2026-09-10T14:00",
                "2026-09-10T16:00",
                &[Weekday::Thu],
            ),
            now(),
        )
        .unwrap();
    let series_id = read_local_series(&store);
    // "Edit this occurrence only": skip it and add a standalone replacement.
    let mut skip = LocalMutation::upsert(LocalKind::SeriesSkip);
    skip.id = Some(series_id.clone());
    skip.occurrence_date = Some("2026-09-10".into());
    let moved = anchor("Synthetic studio", "2026-09-10T15:00", "2026-09-10T17:00");
    store
        .mutate_local_batch(&[skip.clone(), moved], now())
        .unwrap();
    let input = read(&store, now());
    assert_eq!(input.anchors.len(), 2);
    assert_eq!(timed_start(&input.anchors[0]), "2026-09-10T19:00:00.000Z");

    // A failing member rolls back the whole batch.
    let mut unskip = skip.clone();
    unskip.action = LocalAction::Remove;
    let invalid = anchor("Broken", "2026-09-10T17:00", "2026-09-10T16:00");
    assert!(store.mutate_local_batch(&[unskip, invalid], now()).is_err());
    assert_eq!(read(&store, now()), input);
    assert!(store.mutate_local_batch(&[], now()).is_err());
}

fn read_local_series(store: &Store) -> String {
    store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap()
        .local
        .anchor_series[0]
        .meta
        .id
        .to_string()
}

#[test]
fn usual_availability_expands_and_yields_to_explicit_declarations() {
    let mut store = Store::memory().unwrap();
    let mut usual = LocalMutation::upsert(LocalKind::UsualAvailability);
    usual.blocks = [
        Weekday::Mon,
        Weekday::Tue,
        Weekday::Wed,
        Weekday::Thu,
        Weekday::Fri,
    ]
    .into_iter()
    .map(|weekday| BlockInput {
        weekday,
        start: "09:00".into(),
        end: "17:00".into(),
    })
    .collect();
    usual.energy_capacity = Some(EnergyCapacity::Normal);
    store.mutate_local(&usual, now()).unwrap();
    let input = read(&store, now());
    // Wed 9 through Tue 22: ten weekdays. The extent ends at 08:00 on Wed 23,
    // before that day's block begins.
    assert_eq!(input.availability.len(), 10);
    temporal_core::evaluate(&input).unwrap();

    let mut explicit = LocalMutation::upsert(LocalKind::Availability);
    explicit.start = Some("2026-09-10T12:00".into());
    explicit.end = Some("2026-09-10T20:00".into());
    store.mutate_local(&explicit, now()).unwrap();
    let input = read(&store, now());
    let thursday = TimedSpan::new(
        "2026-09-10T04:00:00.000Z".parse().unwrap(),
        "2026-09-11T04:00:00.000Z".parse().unwrap(),
    )
    .unwrap();
    let mut spans: Vec<(String, String)> = input
        .availability
        .iter()
        .filter(|d| d.span.overlaps(&thursday))
        .map(|d| (d.span.start().to_string(), d.span.end().to_string()))
        .collect();
    spans.sort();
    assert_eq!(
        spans,
        [
            (
                "2026-09-10T13:00:00.000Z".into(),
                "2026-09-10T16:00:00.000Z".into()
            ),
            (
                "2026-09-10T16:00:00.000Z".into(),
                "2026-09-11T00:00:00.000Z".into()
            )
        ]
    );
    temporal_core::evaluate(&input).unwrap();

    let mut overlapping = LocalMutation::upsert(LocalKind::UsualAvailability);
    overlapping.blocks = vec![
        BlockInput {
            weekday: Weekday::Mon,
            start: "09:00".into(),
            end: "12:00".into(),
        },
        BlockInput {
            weekday: Weekday::Mon,
            start: "11:00".into(),
            end: "13:00".into(),
        },
    ];
    assert!(store.mutate_local(&overlapping, now()).is_err());

    let mut clear = LocalMutation::upsert(LocalKind::UsualAvailability);
    clear.action = LocalAction::Remove;
    store.mutate_local(&clear, now()).unwrap();
    assert_eq!(read(&store, now()).availability.len(), 1);
}

#[test]
fn a_version_two_store_migrates_to_settings() {
    let path = std::env::temp_dir().join(format!(
        "temporal-migrate-v2-{}.sqlite3",
        uuid::Uuid::new_v4()
    ));
    let mut store = Store::open(&path).unwrap();
    store
        .mutate_local(
            &anchor("Kept", "2026-09-10T09:00", "2026-09-10T10:00"),
            now(),
        )
        .unwrap();
    let before = read(&store, now());
    drop(store);
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch(
            "DROP TABLE app_settings; DROP TABLE calendar_sources; PRAGMA user_version=2;",
        )
        .unwrap();
    drop(connection);
    let reopened = Store::open(&path).unwrap();
    assert_eq!(read(&reopened, now()), before);
    assert_eq!(
        reopened.settings().unwrap(),
        temporal_app::settings::Settings::default()
    );
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn trace_health_qualifies_task_work_but_not_windows_or_unrelated_deadlines() {
    let mut store = Store::memory().unwrap();
    let mut usual = LocalMutation::upsert(LocalKind::UsualAvailability);
    usual.blocks = vec![
        BlockInput {
            weekday: Weekday::Wed,
            start: "09:00".into(),
            end: "17:00".into(),
        },
        BlockInput {
            weekday: Weekday::Fri,
            start: "09:00".into(),
            end: "17:00".into(),
        },
    ];
    store.mutate_local(&usual, now()).unwrap();
    let mut deadline = LocalMutation::upsert(LocalKind::Deadline);
    deadline.title = Some("Synthetic form".into());
    deadline.due = Some("2026-09-15T17:00".into());
    deadline.effort_minutes = Some(60);
    deadline.minimum_chunk_minutes = Some(30);
    store.mutate_local(&deadline, now()).unwrap();
    let conditional = |store: &Store, at: Instant, species: &str| -> Vec<bool> {
        let view = temporal_app::personal::view(store, at).unwrap();
        serde_json::to_value(&view.view).unwrap()["items"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|i| i["species"] == species)
            .map(|i| i["conditional"].as_bool().unwrap())
            .collect()
    };
    // Trace was never imported: nothing about time depends on it.
    assert!(conditional(&store, now(), "window").iter().all(|c| !c));
    assert_eq!(conditional(&store, now(), "deadline"), [false]);

    let export = serde_json::json!({
        "version": "1.0",
        "exported_at": now().to_string(),
        "tasks": [{
            "id": "trace-synthetic", "text": "Synthetic reading", "raw_input": null, "link": null,
            "status": "later", "context": "desk", "priority": 2,
            "due_at": null, "created_at": "2026-09-09T10:00:00.000Z",
            "updated_at": "2026-09-09T11:00:00.000Z", "completed_at": null, "sort_order": 1.5
        }]
    });
    store.import_json(&export.to_string(), now()).unwrap();
    let mut work = LocalMutation::upsert(LocalKind::TaskAnnotation);
    work.trace_external_id = Some("trace-synthetic".into());
    work.effort_minutes = Some(60);
    work.minimum_chunk_minutes = Some(30);
    store.mutate_local(&work, now()).unwrap();
    // Two days later the manual snapshot is stale: only task work is qualified.
    let later = now().checked_add_ms(2 * 86_400_000).unwrap();
    assert_eq!(conditional(&store, later, "task"), [true]);
    assert!(conditional(&store, later, "window").iter().all(|c| !c));
    assert_eq!(conditional(&store, later, "deadline"), [false]);
}
