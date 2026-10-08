//! Gate 4.3: read-only iCalendar sources in the app store. Synthetic only.
use temporal_app::{
    calendar,
    calendars::{CalendarMode, CalendarOrigin},
    local::{LocalAction, LocalKind, LocalMutation},
    personal,
    store::Store,
};
use temporal_core::domain::*;
use temporal_core::results::Health;
use temporal_core::time::Instant;

fn now() -> Instant {
    // Wednesday 2026-09-09, 08:00 in Toronto.
    "2026-09-09T12:00:00.000Z".parse().unwrap()
}

const SEMINARS: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Synthetic//EN\r\nX-WR-CALNAME:Synthetic seminars\r\nBEGIN:VEVENT\r\nUID:seminar-1@example.invalid\r\nDTSTART;TZID=America/Toronto:20260910T100000\r\nDTEND;TZID=America/Toronto:20260910T113000\r\nRRULE:FREQ=WEEKLY;COUNT=4\r\nSUMMARY:Synthetic seminar\r\nLOCATION:Room 200\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nUID:holiday@example.invalid\r\nDTSTART;VALUE=DATE:20260912\r\nDTEND;VALUE=DATE:20260913\r\nSUMMARY:Synthetic holiday\r\nTRANSP:TRANSPARENT\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

const COURSEWORK: &str = "BEGIN:VCALENDAR\r\nVERSION:2.0\r\nPRODID:-//Synthetic LMS//EN\r\nBEGIN:VEVENT\r\nUID:event-assignment-101\r\nDTSTART:20260905T035900Z\r\nDTEND:20260905T035900Z\r\nSUMMARY:Synthetic problem set 1\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nUID:event-assignment-102\r\nDTSTART:20260912T035900Z\r\nDTEND:20260912T035900Z\r\nSUMMARY:Synthetic problem set 2\r\nEND:VEVENT\r\nBEGIN:VEVENT\r\nUID:event-calendar-event-7\r\nDTSTART:20260911T180000Z\r\nDTEND:20260911T190000Z\r\nSUMMARY:Synthetic office hour\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";

fn add(store: &mut Store, text: &str, mode: CalendarMode, at: Instant) -> SourceId {
    store
        .add_calendar(
            "Synthetic calendar",
            SourceKind::Calendar,
            mode,
            "#286580",
            CalendarOrigin::File {
                name: "synthetic.ics".into(),
            },
            text,
            at,
        )
        .unwrap()
}

fn read(store: &Store, at: Instant) -> EvaluationInput {
    store
        .read(at, "America/Toronto".parse().unwrap())
        .unwrap()
        .input
}

#[test]
fn an_events_calendar_becomes_imported_anchors_with_stable_identity() {
    let mut store = Store::memory().unwrap();
    let id = add(&mut store, SEMINARS, CalendarMode::Events, now());
    let input = read(&store, now());
    let seminars: Vec<&Anchor> = input
        .anchors
        .iter()
        .filter(|a| a.title == "Synthetic seminar")
        .collect();
    // Weekly from Thu Sep 10, four times: 10, 17 (in range), 24 and Oct 1 (beyond 14 days).
    assert_eq!(seminars.len(), 2);
    for anchor in &seminars {
        let Provenance::Imported(provenance) = &anchor.provenance else {
            panic!("imported provenance expected");
        };
        assert_eq!(provenance.source_id, id);
        assert_eq!(provenance.external_id, "seminar-1@example.invalid");
        assert!(provenance.occurrence_key.is_some());
        assert_eq!(anchor.occupancy, Occupancy::Busy);
    }
    let holiday = input
        .anchors
        .iter()
        .find(|a| a.title == "Synthetic holiday")
        .unwrap();
    assert_eq!(holiday.occupancy, Occupancy::Transparent);
    assert!(input.deadlines.is_empty());
    assert!(
        input
            .sources
            .iter()
            .any(|s| s.id == id && s.kind == SourceKind::Calendar)
    );
    assert!(
        input
            .required_sources
            .iter()
            .any(|r| r.source_id == id && r.role == SourceRole::Anchors)
    );
    let again = read(&store, now());
    assert_eq!(input.anchors, again.anchors);
    temporal_core::evaluate(&input).unwrap();
}

#[test]
fn coursework_assignments_are_real_deadlines_that_can_be_marked_handled() {
    let mut store = Store::memory().unwrap();
    let id = add(&mut store, COURSEWORK, CalendarMode::Coursework, now());
    let input = read(&store, now());
    let mut titles: Vec<&str> = input.deadlines.iter().map(|d| d.title.as_str()).collect();
    titles.sort();
    assert_eq!(
        titles,
        ["Synthetic problem set 1", "Synthetic problem set 2"]
    );
    assert!(
        input
            .anchors
            .iter()
            .any(|a| a.title == "Synthetic office hour")
    );
    let output = temporal_core::evaluate(&input).unwrap();
    let past = input
        .deadlines
        .iter()
        .find(|d| d.title == "Synthetic problem set 1")
        .unwrap();
    let state = output
        .deadline_states
        .iter()
        .find(|s| s.id == past.meta.id)
        .unwrap();
    assert_eq!(
        serde_json::to_value(state.phase).unwrap(),
        serde_json::json!("overdue")
    );

    // The person marks the past assignment handled: it leaves pressure and
    // the daily edit, while the source fact stays visible in the Calendar.
    let view = personal::view(&store, now()).unwrap();
    let summary = view
        .calendars
        .iter()
        .find(|c| c.id == id.to_string())
        .unwrap();
    assert_eq!(summary.past_unhandled, [past.meta.id.to_string()]);
    let mut handled = LocalMutation::upsert(LocalKind::ImportedHandled);
    handled.id = Some(past.meta.id.to_string());
    store.mutate_local(&handled, now()).unwrap();
    let input = read(&store, now());
    assert_eq!(input.deadlines.len(), 1);
    let stored = store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap();
    let range = calendar::range(
        &stored,
        now(),
        "2026-09-01T00:00:00.000Z".parse().unwrap(),
        "2026-09-20T00:00:00.000Z".parse().unwrap(),
    )
    .unwrap();
    let row = range
        .events
        .iter()
        .find(|e| e.title == "Synthetic problem set 1")
        .unwrap();
    assert_eq!(row.state, "handled");
    assert!(!row.editable);
    assert_eq!(row.kind, "deadline");

    // Only an imported deadline in view can be marked; undo is explicit.
    let mut bogus = LocalMutation::upsert(LocalKind::ImportedHandled);
    bogus.id = Some("00000000-0000-4000-8000-00000000beef".into());
    assert!(store.mutate_local(&bogus, now()).is_err());
    handled.action = LocalAction::Remove;
    store.mutate_local(&handled, now()).unwrap();
    assert_eq!(read(&store, now()).deadlines.len(), 2);
}

#[test]
fn a_failed_refresh_keeps_last_known_events_and_exposes_health() {
    let mut store = Store::memory().unwrap();
    let id = add(&mut store, SEMINARS, CalendarMode::Events, now());
    let before = read(&store, now()).anchors;
    let later = now().checked_add_ms(3_600_000).unwrap();
    assert!(
        store
            .refresh_calendar(
                id,
                Err((
                    AttemptOutcome::Failed,
                    "Could not reach the calendar.".into()
                )),
                later
            )
            .is_err()
    );
    assert!(
        store
            .refresh_calendar(id, Ok("not a calendar".into()), later)
            .is_err()
    );
    let input = read(&store, later);
    assert_eq!(input.anchors.len(), before.len());
    let output = temporal_core::evaluate(&input).unwrap();
    let health = output
        .source_health
        .iter()
        .find(|h| h.source_id == id)
        .unwrap();
    assert_eq!(health.health, Health::Incompatible);
    let summary = personal::view(&store, later).unwrap();
    let calendar = summary
        .calendars
        .iter()
        .find(|c| c.id == id.to_string())
        .unwrap();
    assert!(calendar.last_error.is_some());

    // A successful refresh replaces the snapshot; removed entries disappear.
    let trimmed = SEMINARS.replace("RRULE:FREQ=WEEKLY;COUNT=4\r\n", "");
    store.refresh_calendar(id, Ok(trimmed), later).unwrap();
    let refreshed = read(&store, later);
    assert_eq!(
        refreshed
            .anchors
            .iter()
            .filter(|a| a.title == "Synthetic seminar")
            .count(),
        1
    );
    let output = temporal_core::evaluate(&refreshed).unwrap();
    assert_eq!(
        output
            .source_health
            .iter()
            .find(|h| h.source_id == id)
            .unwrap()
            .health,
        Health::Healthy
    );
}

#[test]
fn a_file_snapshot_becomes_stale_and_qualifies_its_rows() {
    let mut store = Store::memory().unwrap();
    let id = add(&mut store, SEMINARS, CalendarMode::Events, now());
    let week_later = now().checked_add_ms(8 * 86_400_000).unwrap();
    let input = read(&store, week_later);
    let output = temporal_core::evaluate(&input).unwrap();
    assert_eq!(
        output
            .source_health
            .iter()
            .find(|h| h.source_id == id)
            .unwrap()
            .health,
        Health::Stale
    );
}

#[test]
fn hidden_and_removed_calendars_leave_evaluation_without_touching_local_time() {
    let mut store = Store::memory().unwrap();
    let id = add(&mut store, SEMINARS, CalendarMode::Events, now());
    let mut local = LocalMutation::upsert(LocalKind::Anchor);
    local.title = Some("Local dentist".into());
    local.start = Some("2026-09-10T15:00".into());
    local.end = Some("2026-09-10T16:00".into());
    local.occupancy = Some(Occupancy::Busy);
    store.mutate_local(&local, now()).unwrap();
    store
        .update_calendar(id, Some("Renamed"), None, Some("#AA3366"), Some(true))
        .unwrap();
    let input = read(&store, now());
    assert_eq!(input.anchors.len(), 1);
    assert!(
        store
            .update_calendar(id, None, None, Some("red"), None)
            .is_err()
    );
    store
        .update_calendar(id, None, None, None, Some(false))
        .unwrap();
    assert_eq!(store.calendars().unwrap()[0].source.label, "Renamed");
    assert_eq!(store.calendars().unwrap()[0].color, "#aa3366");
    assert!(read(&store, now()).anchors.len() > 1);
    store.remove_calendar(id).unwrap();
    let input = read(&store, now());
    assert_eq!(input.anchors.len(), 1);
    assert!(store.calendars().unwrap().is_empty());
    assert!(store.remove_calendar(id).is_err());
}

#[test]
fn bad_input_adds_nothing() {
    let mut store = Store::memory().unwrap();
    for text in ["", "hello", "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:x\r\n"] {
        assert!(
            store
                .add_calendar(
                    "Bad",
                    SourceKind::Google,
                    CalendarMode::Events,
                    "#286580",
                    CalendarOrigin::File {
                        name: "x.ics".into()
                    },
                    text,
                    now(),
                )
                .is_err()
        );
    }
    assert!(
        store
            .add_calendar(
                "Not allowed",
                SourceKind::Trace,
                CalendarMode::Events,
                "#286580",
                CalendarOrigin::File {
                    name: "x.ics".into()
                },
                SEMINARS,
                now(),
            )
            .is_err()
    );
    assert!(store.calendars().unwrap().is_empty());
}

#[test]
fn the_calendar_range_merges_local_series_and_imported_rows() {
    let mut store = Store::memory().unwrap();
    add(&mut store, SEMINARS, CalendarMode::Events, now());
    let mut series = LocalMutation::upsert(LocalKind::AnchorSeries);
    series.title = Some("Synthetic lab".into());
    series.start = Some("2026-09-08T14:00".into());
    series.end = Some("2026-09-08T16:00".into());
    series.occupancy = Some(Occupancy::Busy);
    series.frequency = Some(temporal_app::series::SeriesFrequency::Weekly);
    series.weekdays = vec![Weekday::Tue];
    store.mutate_local(&series, now()).unwrap();
    let stored = store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap();
    // A range well outside the 14-day Horizon still shows stored facts.
    let range = calendar::range(
        &stored,
        now(),
        "2026-09-28T04:00:00.000Z".parse().unwrap(),
        "2026-10-05T04:00:00.000Z".parse().unwrap(),
    )
    .unwrap();
    let titles: Vec<(&str, bool)> = range
        .events
        .iter()
        .map(|e| (e.title.as_str(), e.editable))
        .collect();
    assert_eq!(
        titles,
        [("Synthetic lab", true), ("Synthetic seminar", false)]
    );
    let lab = &range.events[0];
    assert!(lab.series_id.is_some());
    assert_eq!(lab.occurrence_date.as_deref(), Some("2026-09-29"));
    let seminar = &range.events[1];
    assert_eq!(seminar.location.as_deref(), Some("Room 200"));
    assert_eq!(seminar.color.as_deref(), Some("#286580"));
    assert!(calendar::range(&stored, now(), now(), now()).is_err());
}
