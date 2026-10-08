use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Timelike, Utc};
use chrono_tz::{America, Europe, Tz};
use temporal_app::ics::{self, IcsCalendar, IcsError, IcsTime, Occurrence, OccurrenceSpan};

fn ndt(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_opt(hour, minute, 0)
        .unwrap()
}

fn utc(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> i64 {
    ndt(year, month, day, hour, minute)
        .and_utc()
        .timestamp_millis()
}

fn toronto(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> i64 {
    America::Toronto
        .from_local_datetime(&ndt(year, month, day, hour, minute))
        .earliest()
        .unwrap()
        .timestamp_millis()
}

fn parse(text: &str) -> IcsCalendar {
    ics::parse(text, America::Toronto).expect("synthetic calendar parses")
}

fn expand(text: &str, from: i64, to: i64) -> (Vec<Occurrence>, usize) {
    ics::expand(&parse(text), from, to, America::Toronto)
}

fn year_2026() -> (i64, i64) {
    (utc(2026, 1, 1, 0, 0), utc(2027, 1, 1, 0, 0))
}

fn september() -> (i64, i64) {
    (toronto(2026, 9, 1, 0, 0), toronto(2026, 10, 1, 0, 0))
}

fn timed(occurrence: &Occurrence) -> (i64, i64, Option<Tz>) {
    match &occurrence.span {
        OccurrenceSpan::Timed {
            start_ms,
            end_ms,
            zone,
        } => (*start_ms, *end_ms, *zone),
        other => panic!("expected a timed span, got {other:?}"),
    }
}

fn starts(occurrences: &[Occurrence]) -> Vec<i64> {
    occurrences.iter().map(|o| timed(o).0).collect()
}

fn keys(occurrences: &[Occurrence]) -> Vec<Option<&str>> {
    occurrences
        .iter()
        .map(|o| o.occurrence_key.as_deref())
        .collect()
}

fn summaries(occurrences: &[Occurrence]) -> Vec<&str> {
    occurrences.iter().map(|o| o.summary.as_str()).collect()
}

fn one_event_with_rule(rule: &str) -> String {
    format!(
        "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:synthetic-edge-1@example.invalid\n\
         DTSTART;TZID=America/Toronto:20260915T090000\n\
         DTEND;TZID=America/Toronto:20260915T100000\nRRULE:{rule}\n\
         SUMMARY:Synthetic Edge\nEND:VEVENT\nEND:VCALENDAR\n"
    )
}

#[test]
fn simple_utc_event_keeps_its_fields_and_expands_once() {
    let text = include_str!("fixtures/ics/basic-utc.ics");
    let cal = parse(text);
    assert_eq!(
        cal.prodid.as_deref(),
        Some("-//Synthetic Calendar Co//Fixtures//EN")
    );
    assert_eq!(cal.skipped, 0);
    assert_eq!(cal.events.len(), 1);
    let event = &cal.events[0];
    assert_eq!(event.uid, "synthetic-utc-1@example.invalid");
    assert_eq!(event.summary, "Synthetic Seminar");
    assert_eq!(event.location.as_deref(), Some("Room Alpha"));
    assert_eq!(
        event.url.as_deref(),
        Some("https://example.invalid/synthetic-seminar")
    );
    assert_eq!(event.start, IcsTime::Utc(ndt(2026, 9, 15, 14, 0)));
    assert_eq!(event.end, Some(IcsTime::Utc(ndt(2026, 9, 15, 15, 0))));
    assert!(!event.zone_guessed);

    let (occurrences, unexpanded) = expand(text, september().0, september().1);
    assert_eq!(unexpanded, 0);
    assert_eq!(occurrences.len(), 1);
    assert_eq!(occurrences[0].occurrence_key, None);
    assert_eq!(
        timed(&occurrences[0]),
        (utc(2026, 9, 15, 14, 0), utc(2026, 9, 15, 15, 0), None)
    );
}

#[test]
fn iana_tzid_keeps_the_local_wall_time_in_its_zone() {
    let text = include_str!("fixtures/ics/tzid-iana.ics");
    let cal = parse(text);
    assert_eq!(
        cal.events[0].start,
        IcsTime::Zoned(ndt(2026, 9, 15, 9, 0), America::Toronto)
    );
    let (occurrences, _) = expand(text, september().0, september().1);
    // September is EDT (UTC-4), so 09:00 local is 13:00Z.
    assert_eq!(
        timed(&occurrences[0]),
        (
            utc(2026, 9, 15, 13, 0),
            utc(2026, 9, 15, 14, 0),
            Some(America::Toronto)
        )
    );
    assert!(!occurrences[0].zone_guessed);
}

#[test]
fn windows_tzid_maps_to_its_iana_zone() {
    let text = include_str!("fixtures/ics/tzid-windows.ics");
    let cal = parse(text);
    assert_eq!(
        cal.events[0].start,
        IcsTime::Zoned(ndt(2026, 9, 15, 9, 0), America::New_York)
    );
    assert!(!cal.events[0].zone_guessed);
    let (occurrences, _) = expand(text, september().0, september().1);
    assert_eq!(
        timed(&occurrences[0]),
        (
            utc(2026, 9, 15, 13, 0),
            utc(2026, 9, 15, 14, 0),
            Some(America::New_York)
        )
    );
}

#[test]
fn path_prefixed_tzids_map_to_the_trailing_iana_zone() {
    let text = include_str!("fixtures/ics/tzid-path.ics");
    let cal = parse(text);
    assert_eq!(
        cal.events[0].start,
        IcsTime::Zoned(ndt(2026, 9, 15, 9, 0), America::New_York)
    );
    assert_eq!(
        cal.events[1].start,
        IcsTime::Zoned(ndt(2026, 9, 15, 11, 0), America::Toronto)
    );
    assert!(cal.events.iter().all(|event| !event.zone_guessed));
    let (occurrences, _) = expand(text, september().0, september().1);
    assert_eq!(occurrences.len(), 2);
    assert_eq!(timed(&occurrences[0]).0, utc(2026, 9, 15, 13, 0));
    assert_eq!(timed(&occurrences[1]).0, utc(2026, 9, 15, 15, 0));
}

#[test]
fn unmappable_tzid_is_guessed_and_uses_the_fallback_zone() {
    let text = include_str!("fixtures/ics/unmappable-tzid.ics");
    let cal = ics::parse(text, Europe::London).expect("parses");
    assert!(cal.events[0].zone_guessed);
    assert_eq!(
        cal.events[0].start,
        IcsTime::Zoned(ndt(2026, 9, 15, 9, 0), Europe::London)
    );
    let (occurrences, _) = ics::expand(&cal, september().0, september().1, America::Toronto);
    assert!(occurrences[0].zone_guessed);
    // London is on BST (UTC+1) in September.
    assert_eq!(timed(&occurrences[0]).0, utc(2026, 9, 15, 8, 0));
    assert_eq!(timed(&occurrences[0]).2, Some(Europe::London));
}

#[test]
fn calendar_default_zone_wins_over_the_fallback_for_unmappable_tzid() {
    let text = "BEGIN:VCALENDAR\nX-WR-TIMEZONE:America/Vancouver\nBEGIN:VEVENT\n\
        UID:synthetic-default-1\nDTSTART;TZID=Synthetic/Unknown:20260915T090000\n\
        END:VEVENT\nEND:VCALENDAR\n";
    let cal = parse(text);
    assert_eq!(
        cal.events[0].start,
        IcsTime::Zoned(ndt(2026, 9, 15, 9, 0), America::Vancouver)
    );
    assert!(cal.events[0].zone_guessed);
}

#[test]
fn all_day_single_and_multi_day_spans_are_date_based() {
    let text = include_str!("fixtures/ics/all-day.ics");
    let cal = parse(text);
    assert_eq!(
        cal.events[0].start,
        IcsTime::Date(NaiveDate::from_ymd_opt(2026, 9, 20).unwrap())
    );
    assert_eq!(
        cal.events[0].end,
        Some(IcsTime::Date(NaiveDate::from_ymd_opt(2026, 9, 21).unwrap()))
    );

    let (occurrences, _) = expand(text, september().0, september().1);
    assert_eq!(occurrences.len(), 2);
    assert_eq!(
        occurrences[0].span,
        OccurrenceSpan::Dates {
            start: NaiveDate::from_ymd_opt(2026, 9, 20).unwrap(),
            end_exclusive: NaiveDate::from_ymd_opt(2026, 9, 21).unwrap(),
        }
    );
    assert_eq!(
        occurrences[1].span,
        OccurrenceSpan::Dates {
            start: NaiveDate::from_ymd_opt(2026, 9, 25).unwrap(),
            end_exclusive: NaiveDate::from_ymd_opt(2026, 9, 28).unwrap(),
        }
    );

    // The multi-day block intersects the 27th; a window starting at its exclusive end does not.
    let (on_27th, _) = expand(text, toronto(2026, 9, 27, 0, 0), toronto(2026, 9, 28, 0, 0));
    assert_eq!(summaries(&on_27th), ["Synthetic Conference Days"]);
    let (after, _) = expand(text, toronto(2026, 9, 28, 0, 0), toronto(2026, 9, 29, 0, 0));
    assert!(after.is_empty());
}

#[test]
fn folded_summary_unescapes_text_and_keeps_escaped_separators() {
    let cal = parse(include_str!("fixtures/ics/folded-escaped.ics"));
    let event = &cal.events[0];
    assert_eq!(
        event.summary,
        "Synthetic Seminar, Room 4\nSecond line of a very long title that continues on the folded line"
    );
    assert_eq!(event.location.as_deref(), Some("Building B; Floor 2"));
}

#[test]
fn crlf_folding_removes_exactly_one_whitespace_character() {
    let text = "BEGIN:VCALENDAR\r\nBEGIN:VEVENT\r\nUID:synthetic-crlf-1\r\n\
        DTSTART:20260922T150000Z\r\nSUMMARY:Synthetic\r\n  folded title\r\n\
        DESCRIPTION:ignored\r\nSUMMARY:Tab\r\n\tfold\r\nEND:VEVENT\r\nEND:VCALENDAR\r\n";
    let cal = parse(text);
    assert_eq!(cal.events.len(), 1);
    // The first SUMMARY wins; one leading space of the fold is removed, the second space stays.
    assert_eq!(cal.events[0].summary, "Synthetic folded title");
}

#[test]
fn duration_fills_the_end_and_a_zero_length_assignment_is_a_point() {
    let text = include_str!("fixtures/ics/duration-zero.ics");
    let cal = parse(text);
    let by_uid = |uid: &str| {
        cal.events
            .iter()
            .find(|event| event.uid == uid)
            .expect("fixture event present")
    };

    let timed_event = by_uid("synthetic-duration-1@example.invalid");
    assert_eq!(
        timed_event.end,
        Some(IcsTime::Utc(ndt(2026, 9, 16, 11, 30)))
    );

    let block = by_uid("synthetic-allday-duration@example.invalid");
    assert_eq!(
        block.end,
        Some(IcsTime::Date(NaiveDate::from_ymd_opt(2026, 10, 2).unwrap()))
    );

    let assignment = by_uid("event-assignment-123");
    assert_eq!(assignment.start, assignment.end.clone().unwrap());

    // The zero-length assignment is inside [23:58, 23:59) only if its start is before 23:59.
    let (none, _) = expand(text, utc(2026, 9, 17, 23, 58), utc(2026, 9, 17, 23, 59));
    assert!(
        summaries(&none)
            .iter()
            .all(|s| *s != "Synthetic Assignment Due")
    );
    let (hit, _) = expand(text, utc(2026, 9, 17, 23, 59), utc(2026, 9, 18, 0, 0));
    assert_eq!(summaries(&hit), ["Synthetic Assignment Due"]);
    assert_eq!(
        timed(&hit[0]),
        (utc(2026, 9, 17, 23, 59), utc(2026, 9, 17, 23, 59), None)
    );

    // The two-day block covers 1 October even though it starts on 30 September.
    let (block_hits, _) = expand(text, toronto(2026, 10, 1, 0, 0), toronto(2026, 10, 2, 0, 0));
    assert_eq!(summaries(&block_hits), ["Synthetic Two Day Block"]);
}

#[test]
fn calendar_name_default_zone_and_floating_times_resolve_in_the_default_zone() {
    let text = include_str!("fixtures/ics/calendar-floating.ics");
    let cal = parse(text);
    assert_eq!(cal.name.as_deref(), Some("Synthetic Floating Calendar"));
    assert_eq!(cal.default_zone, Some(America::Toronto));
    assert_eq!(
        cal.events[0].start,
        IcsTime::Floating(ndt(2026, 3, 5, 9, 0))
    );

    // The caller passes UTC, but X-WR-TIMEZONE wins. March 5 is EST (UTC-5).
    let (occurrences, _) = ics::expand(
        &cal,
        utc(2026, 3, 1, 0, 0),
        utc(2026, 3, 10, 0, 0),
        chrono_tz::UTC,
    );
    assert_eq!(
        timed(&occurrences[0]),
        (
            utc(2026, 3, 5, 14, 0),
            utc(2026, 3, 5, 15, 0),
            Some(America::Toronto)
        )
    );
}

#[test]
fn floating_times_without_a_calendar_zone_use_the_caller_zone() {
    let text = include_str!("fixtures/ics/calendar-floating.ics")
        .replace("X-WR-TIMEZONE:America/Toronto\n", "");
    let cal = parse(&text);
    assert_eq!(cal.default_zone, None);
    let (occurrences, _) = ics::expand(
        &cal,
        utc(2026, 3, 1, 0, 0),
        utc(2026, 3, 10, 0, 0),
        America::Vancouver,
    );
    // Vancouver is PST (UTC-8) on 5 March.
    assert_eq!(
        timed(&occurrences[0]),
        (
            utc(2026, 3, 5, 17, 0),
            utc(2026, 3, 5, 18, 0),
            Some(America::Vancouver)
        )
    );
}

#[test]
fn weekly_byday_with_until_stops_at_the_until_instant() {
    let text = include_str!("fixtures/ics/weekly-until.ics");
    let (occurrences, unexpanded) = expand(text, september().0, september().1);
    assert_eq!(unexpanded, 0);
    assert_eq!(
        starts(&occurrences),
        [
            toronto(2026, 9, 7, 9, 0),
            toronto(2026, 9, 9, 9, 0),
            toronto(2026, 9, 14, 9, 0),
            toronto(2026, 9, 16, 9, 0),
            toronto(2026, 9, 21, 9, 0),
            toronto(2026, 9, 23, 9, 0),
        ]
    );
    assert_eq!(keys(&occurrences)[0], Some("20260907T130000Z"));
    assert!(
        occurrences
            .iter()
            .all(|o| timed(o).1 - timed(o).0 == 30 * 60_000)
    );
}

#[test]
fn until_without_z_is_read_in_the_series_zone_not_the_host_zone() {
    let date_only = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:until-date\n\
        DTSTART;TZID=America/Toronto:20260907T090000\n\
        DTEND;TZID=America/Toronto:20260907T093000\n\
        RRULE:FREQ=WEEKLY;BYDAY=MO,WE;UNTIL=20260923\nEND:VEVENT\nEND:VCALENDAR\n";
    let (with_23rd, unexpanded) = expand(date_only, september().0, september().1);
    assert_eq!(unexpanded, 0);
    assert_eq!(with_23rd.len(), 6, "UNTIL date includes the whole 23rd");

    let floating_before = date_only.replace("UNTIL=20260923", "UNTIL=20260922T235959");
    let (before, _) = expand(&floating_before, september().0, september().1);
    assert_eq!(before.len(), 5);
    assert_eq!(
        keys(&before).last().copied().flatten(),
        Some("20260921T130000Z")
    );
}

#[test]
fn count_limits_the_series_and_the_window_selects_from_it() {
    let text = include_str!("fixtures/ics/count-daily.ics");
    let (occurrences, unexpanded) = expand(text, september().0, september().1);
    assert_eq!(unexpanded, 0);
    assert_eq!(
        keys(&occurrences),
        [
            Some("20260901T120000Z"),
            Some("20260902T120000Z"),
            Some("20260903T120000Z"),
        ]
    );

    let (later, _) = expand(text, toronto(2026, 9, 2, 0, 0), september().1);
    assert_eq!(
        keys(&later),
        [Some("20260902T120000Z"), Some("20260903T120000Z")]
    );
}

#[test]
fn huge_count_is_bounded_by_the_range_and_the_per_event_cap() {
    let text = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:synthetic-huge-1\n\
        DTSTART:20260101T000000Z\nDTEND:20260101T003000Z\n\
        RRULE:FREQ=DAILY;COUNT=4000000\nSUMMARY:Synthetic Long Series\n\
        END:VEVENT\nEND:VCALENDAR\n";
    let (month, unexpanded) = expand(text, utc(2026, 1, 1, 0, 0), utc(2026, 2, 1, 0, 0));
    assert_eq!(unexpanded, 0);
    assert_eq!(month.len(), 31);

    let (capped, unexpanded) = expand(text, utc(2026, 1, 1, 0, 0), utc(2040, 1, 1, 0, 0));
    assert_eq!(unexpanded, 0);
    assert_eq!(capped.len(), ics::MAX_OCCURRENCES_PER_EVENT);
    assert!(
        capped
            .windows(2)
            .all(|pair| timed(&pair[0]).0 < timed(&pair[1]).0)
    );
}

#[test]
fn exdate_removes_matching_instances_including_repeated_lists() {
    let text = include_str!("fixtures/ics/exdate-weekly.ics");
    let (occurrences, _) = expand(text, september().0, toronto(2026, 11, 1, 0, 0));
    assert_eq!(
        keys(&occurrences),
        [Some("20260907T130000Z"), Some("20261005T130000Z")]
    );
}

#[test]
fn exdate_matches_by_instant_across_zone_spellings() {
    let text = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:synthetic-exdate-instant\n\
        DTSTART:20260907T130000Z\nRRULE:FREQ=WEEKLY;COUNT=3\n\
        EXDATE:20260914T130000Z\n\
        EXDATE;TZID=America/New_York:20260921T090000\n\
        SUMMARY:Synthetic Instant Match\nEND:VEVENT\nEND:VCALENDAR\n";
    let (occurrences, _) = expand(text, year_2026().0, year_2026().1);
    assert_eq!(keys(&occurrences), [Some("20260907T130000Z")]);
}

#[test]
fn recurrence_id_replaces_one_instance_and_keeps_the_original_key() {
    let text = include_str!("fixtures/ics/recurrence-moved.ics");
    let (occurrences, unexpanded) = expand(text, year_2026().0, year_2026().1);
    assert_eq!(unexpanded, 0);
    assert_eq!(
        summaries(&occurrences),
        [
            "Synthetic Series",
            "Synthetic Series Moved Session",
            "Synthetic Series"
        ]
    );
    assert_eq!(
        keys(&occurrences),
        [
            Some("20260907T130000Z"),
            Some("20260914T130000Z"),
            Some("20260921T130000Z"),
        ]
    );
    assert_eq!(timed(&occurrences[1]).0, toronto(2026, 9, 15, 11, 0));
}

#[test]
fn cancelled_override_removes_its_instance() {
    let text = include_str!("fixtures/ics/recurrence-cancelled.ics");
    let (occurrences, _) = expand(text, year_2026().0, year_2026().1);
    assert_eq!(
        keys(&occurrences),
        [Some("20260907T130000Z"), Some("20260914T130000Z")]
    );
}

#[test]
fn cancelled_master_produces_nothing() {
    let text = include_str!("fixtures/ics/master-cancelled.ics");
    let (occurrences, unexpanded) = expand(text, year_2026().0, year_2026().1);
    assert!(occurrences.is_empty());
    assert_eq!(unexpanded, 0);
}

#[test]
fn override_shows_when_its_own_span_is_in_the_window_even_if_its_original_is_not() {
    let text = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:synthetic-moved-in-1\n\
        DTSTART;TZID=America/Toronto:20260907T090000\n\
        DTEND;TZID=America/Toronto:20260907T100000\nRRULE:FREQ=WEEKLY;COUNT=3\n\
        SUMMARY:Synthetic Moved In\nEND:VEVENT\n\
        BEGIN:VEVENT\nUID:synthetic-moved-in-1\n\
        RECURRENCE-ID;TZID=America/Toronto:20260907T090000\n\
        DTSTART;TZID=America/Toronto:20260912T100000\n\
        DTEND;TZID=America/Toronto:20260912T110000\n\
        SUMMARY:Synthetic Pulled Forward\nEND:VEVENT\nEND:VCALENDAR\n";
    let (occurrences, _) = expand(text, toronto(2026, 9, 10, 0, 0), toronto(2026, 9, 20, 0, 0));
    assert_eq!(
        summaries(&occurrences),
        ["Synthetic Pulled Forward", "Synthetic Moved In"]
    );
    assert_eq!(
        keys(&occurrences),
        [Some("20260907T130000Z"), Some("20260914T130000Z")]
    );
}

#[test]
fn orphan_override_without_a_master_still_yields() {
    let text = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:synthetic-orphan-1\n\
        RECURRENCE-ID:20260915T140000Z\nDTSTART:20260915T150000Z\n\
        DTEND:20260915T160000Z\nSUMMARY:Synthetic Orphan Override\n\
        END:VEVENT\nEND:VCALENDAR\n";
    let (occurrences, unexpanded) = expand(text, year_2026().0, year_2026().1);
    assert_eq!(unexpanded, 0);
    assert_eq!(occurrences.len(), 1);
    assert_eq!(
        occurrences[0].occurrence_key.as_deref(),
        Some("20260915T140000Z")
    );
    assert_eq!(timed(&occurrences[0]).0, utc(2026, 9, 15, 15, 0));
}

#[test]
fn valarm_text_does_not_leak_into_the_event() {
    let cal = parse(include_str!("fixtures/ics/valarm.ics"));
    assert_eq!(cal.events.len(), 1);
    assert_eq!(cal.events[0].summary, "Synthetic Review");
    assert_eq!(cal.skipped, 0);
}

#[test]
fn weekly_nine_oclock_stays_local_across_the_fall_back_change() {
    let text = include_str!("fixtures/ics/dst-weekly.ics");
    let (occurrences, unexpanded) = expand(
        text,
        toronto(2026, 10, 1, 0, 0),
        toronto(2026, 11, 30, 0, 0),
    );
    assert_eq!(unexpanded, 0);
    assert_eq!(occurrences.len(), 3);
    let utc_hours: Vec<u32> = starts(&occurrences)
        .into_iter()
        .map(|ms| {
            let local = DateTime::<Utc>::from_timestamp_millis(ms)
                .unwrap()
                .with_timezone(&America::Toronto);
            assert_eq!(local.hour(), 9, "stays 09:00 local");
            DateTime::<Utc>::from_timestamp_millis(ms).unwrap().hour()
        })
        .collect();
    assert_eq!(utc_hours, [13, 14, 14]);
}

#[test]
fn duplicate_uid_keeps_the_higher_sequence_and_the_later_tie() {
    let cal = parse(include_str!("fixtures/ics/duplicate-uid.ics"));
    assert_eq!(cal.events.len(), 2);
    assert_eq!(cal.skipped, 0);
    let find = |uid: &str| {
        cal.events
            .iter()
            .find(|event| event.uid == uid)
            .expect("kept")
    };
    assert_eq!(
        find("synthetic-dup-1@example.invalid").summary,
        "Synthetic Duplicate Version Three"
    );
    assert_eq!(find("synthetic-dup-1@example.invalid").sequence, 3);
    assert_eq!(
        find("synthetic-tie-1@example.invalid").summary,
        "Synthetic Tie Later"
    );
}

#[test]
fn malformed_rule_is_counted_and_dtstart_is_still_shown() {
    let text = include_str!("fixtures/ics/malformed-rrule.ics");
    let cal = parse(text);
    assert_eq!(cal.events.len(), 1);
    assert_eq!(
        cal.skipped, 2,
        "missing UID and missing DTSTART are skipped"
    );

    let (occurrences, unexpanded) = expand(text, september().0, september().1);
    assert_eq!(unexpanded, 1);
    assert_eq!(summaries(&occurrences), ["Synthetic Bad Rule"]);
    assert_eq!(keys(&occurrences), [Some("20260915T130000Z")]);
}

#[test]
fn malformed_and_edge_rules_never_panic_and_stay_bounded() {
    let failing = [
        ("FREQ=FORTNIGHTLY", "unknown frequency"),
        ("FREQ=WEEKLY;BYDAY=XX", "bad weekday"),
        ("FREQ=MONTHLY;BYMONTHDAY=0", "zero month day"),
        ("FREQ=DAILY;INTERVAL=0", "zero interval"),
        ("FREQ=YEARLY;BYHOUR=25", "hour out of range"),
        ("FREQ=YEARLY;BYYEARDAY=0", "zero year day"),
        ("FREQ=DAILY;COUNT=99999999999", "count overflows"),
        ("FREQ=DAILY;UNTIL=garbage", "unparsable until"),
        ("FREQ=DAILY;UNTIL=20200101", "until before start"),
    ];
    for (rule, why) in failing {
        let (occurrences, unexpanded) =
            expand(&one_event_with_rule(rule), september().0, september().1);
        assert_eq!(unexpanded, 1, "{why}: {rule}");
        assert_eq!(occurrences.len(), 1, "{why}: DTSTART is still shown");
    }

    for rule in [
        "FREQ=YEARLY;BYSETPOS=2",
        "FREQ=YEARLY",
        "FREQ=SECONDLY;COUNT=1",
    ] {
        let (occurrences, _) = expand(&one_event_with_rule(rule), september().0, september().1);
        assert!(
            occurrences.len() <= ics::MAX_OCCURRENCES_PER_EVENT,
            "{rule}"
        );
    }

    let extremes = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:synthetic-extreme-1\n\
        DTSTART:99991231T235959Z\nRRULE:FREQ=DAILY\nSUMMARY:Synthetic Far Future\n\
        END:VEVENT\nBEGIN:VEVENT\nUID:synthetic-extreme-2\nDTSTART:00010101T000000Z\n\
        RRULE:FREQ=SECONDLY;COUNT=3\nSUMMARY:Synthetic Far Past\nEND:VEVENT\nEND:VCALENDAR\n";
    let (far, _) = expand(
        extremes,
        utc(9999, 12, 31, 0, 0),
        utc(9999, 12, 31, 23, 59) + 60_000,
    );
    assert_eq!(far.len(), 1);
    let (past, _) = expand(extremes, utc(1, 1, 1, 0, 0), utc(1, 1, 1, 0, 1));
    assert_eq!(
        past.len(),
        3,
        "COUNT=3 secondly instances at 00:00:00 to 00:00:02"
    );
}

#[test]
fn rule_that_needs_too_much_scanning_is_counted_not_looped() {
    let text = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:synthetic-scan-1\n\
        DTSTART:20200101T000000Z\nRRULE:FREQ=SECONDLY\nSUMMARY:Synthetic Scan\n\
        END:VEVENT\nEND:VCALENDAR\n";
    let (occurrences, unexpanded) = expand(text, utc(2026, 6, 1, 0, 0), utc(2026, 6, 1, 0, 1));
    assert!(occurrences.is_empty());
    assert_eq!(unexpanded, 1);
}

#[test]
fn a_small_secondly_window_expands_within_budget() {
    let text = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:synthetic-seconds-1\n\
        DTSTART:20260601T000000Z\nRRULE:FREQ=SECONDLY\nSUMMARY:Synthetic Seconds\n\
        END:VEVENT\nEND:VCALENDAR\n";
    // A ten-second window: exactly ten one-second instances, and the scan stops at its end.
    let (occurrences, unexpanded) =
        expand(text, utc(2026, 6, 1, 0, 0), utc(2026, 6, 1, 0, 0) + 10_000);
    assert_eq!(unexpanded, 0);
    assert_eq!(occurrences.len(), 10);
}

#[test]
fn dtstart_is_an_instance_even_when_the_rule_does_not_produce_it() {
    let text = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:synthetic-offrule-1\n\
        DTSTART;TZID=America/Toronto:20260907T090000\n\
        DTEND;TZID=America/Toronto:20260907T100000\n\
        RRULE:FREQ=WEEKLY;BYDAY=TU;UNTIL=20260922T235959Z\n\
        SUMMARY:Synthetic Off Rule\nEND:VEVENT\nEND:VCALENDAR\n";
    let (occurrences, unexpanded) = expand(text, september().0, september().1);
    assert_eq!(unexpanded, 0);
    assert_eq!(
        starts(&occurrences),
        [
            toronto(2026, 9, 7, 9, 0),
            toronto(2026, 9, 8, 9, 0),
            toronto(2026, 9, 15, 9, 0),
            toronto(2026, 9, 22, 9, 0),
        ]
    );
}

#[test]
fn count_includes_dtstart_when_the_rule_does_not_produce_it() {
    let text = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:synthetic-offrule-count\n\
        DTSTART;TZID=America/Toronto:20260907T090000\n\
        DTEND;TZID=America/Toronto:20260907T100000\n\
        RRULE:FREQ=WEEKLY;BYDAY=TU;COUNT=2\n\
        SUMMARY:Synthetic Off Rule Count\nEND:VEVENT\nEND:VCALENDAR\n";
    let (occurrences, _) = expand(text, september().0, september().1);
    assert_eq!(
        starts(&occurrences),
        [toronto(2026, 9, 7, 9, 0), toronto(2026, 9, 8, 9, 0)]
    );
}

#[test]
fn rdate_periods_contribute_their_start_and_duplicates_collapse() {
    let text = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:synthetic-rdate-1\n\
        DTSTART:20260901T090000Z\nDTEND:20260901T093000Z\n\
        RDATE;VALUE=PERIOD:20261201T090000Z/PT1H,20261215T090000Z/20261215T100000Z\n\
        RDATE:20260901T090000Z\nSUMMARY:Synthetic Extra Dates\nEND:VEVENT\nEND:VCALENDAR\n";
    let (occurrences, unexpanded) = expand(text, year_2026().0, year_2026().1);
    assert_eq!(unexpanded, 0);
    assert_eq!(
        keys(&occurrences),
        [
            Some("20260901T090000Z"),
            Some("20261201T090000Z"),
            Some("20261215T090000Z"),
        ]
    );
}

#[test]
fn all_day_series_applies_until_and_exdate_by_date() {
    let text = include_str!("fixtures/ics/date-series.ics");
    let (occurrences, unexpanded) =
        expand(text, toronto(2026, 8, 31, 0, 0), toronto(2026, 9, 5, 0, 0));
    assert_eq!(unexpanded, 0);
    assert_eq!(keys(&occurrences), [Some("20260901"), Some("20260903")]);
    assert_eq!(
        occurrences[1].span,
        OccurrenceSpan::Dates {
            start: NaiveDate::from_ymd_opt(2026, 9, 3).unwrap(),
            end_exclusive: NaiveDate::from_ymd_opt(2026, 9, 4).unwrap(),
        }
    );
}

#[test]
fn window_is_half_open_and_zero_length_events_need_a_start_inside_it() {
    let text = include_str!("fixtures/ics/count-daily.ics");
    // The first instance spans 12:00 to 12:30Z on 1 September.
    let (touching_end, _) = expand(text, utc(2026, 9, 1, 12, 30), utc(2026, 9, 1, 13, 0));
    assert!(touching_end.is_empty(), "span ending at `from` is excluded");
    let (touching_start, _) = expand(text, utc(2026, 9, 1, 11, 0), utc(2026, 9, 1, 12, 0));
    assert!(
        touching_start.is_empty(),
        "span starting at `to` is excluded"
    );
    let (inside, _) = expand(text, utc(2026, 9, 1, 12, 29), utc(2026, 9, 1, 12, 30));
    assert_eq!(inside.len(), 1, "partial overlap counts");

    let zero = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:synthetic-zero-1\n\
        DTSTART:20260917T120000Z\nDTEND:20260917T120000Z\nSUMMARY:Synthetic Point\n\
        END:VEVENT\nEND:VCALENDAR\n";
    let (at_from, _) = expand(zero, utc(2026, 9, 17, 12, 0), utc(2026, 9, 17, 13, 0));
    assert_eq!(at_from.len(), 1, "a point at `from` is included");
    let (at_to, _) = expand(zero, utc(2026, 9, 17, 11, 0), utc(2026, 9, 17, 12, 0));
    assert!(at_to.is_empty(), "a point at `to` is excluded");
}

#[test]
fn occurrences_sort_by_start_then_uid_then_key() {
    let text = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:synthetic-b\nDTSTART:20260915T140000Z\n\
        DTEND:20260915T150000Z\nSUMMARY:B\nEND:VEVENT\nBEGIN:VEVENT\nUID:synthetic-a\n\
        DTSTART:20260915T140000Z\nDTEND:20260915T150000Z\nSUMMARY:A\nEND:VEVENT\n\
        BEGIN:VEVENT\nUID:synthetic-early\nDTSTART:20260915T100000Z\n\
        DTEND:20260915T110000Z\nSUMMARY:Early\nEND:VEVENT\nEND:VCALENDAR\n";
    let (occurrences, _) = expand(text, year_2026().0, year_2026().1);
    assert_eq!(summaries(&occurrences), ["Early", "A", "B"]);
}

#[test]
fn vtodo_vjournal_vfreebusy_and_vtimezone_are_ignored() {
    let text = include_str!("fixtures/ics/vtodo-ignored.ics");
    let cal = parse(text);
    assert_eq!(cal.events.len(), 1);
    assert_eq!(cal.events[0].uid, "synthetic-vevent-1@example.invalid");
    assert_eq!(cal.events[0].summary, "Synthetic Real Event");
    assert_eq!(cal.skipped, 0);
}

#[test]
fn a_leading_byte_order_mark_is_accepted() {
    let text = "\u{feff}BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:synthetic-bom-1\n\
        DTSTART:20260915T140000Z\nEND:VEVENT\nEND:VCALENDAR\n";
    assert_eq!(parse(text).events.len(), 1);
}

#[test]
fn non_calendar_text_is_rejected() {
    let error = ics::parse("Synthetic note: nothing to see here.", chrono_tz::UTC).unwrap_err();
    assert_eq!(error, IcsError::NotCalendar);
    assert_eq!(
        ics::parse("", chrono_tz::UTC).unwrap_err(),
        IcsError::NotCalendar
    );
}

#[test]
fn oversized_input_is_rejected_before_parsing() {
    let oversized = "x".repeat(ics::MAX_BYTES + 1);
    assert_eq!(
        ics::parse(&oversized, chrono_tz::UTC).unwrap_err(),
        IcsError::TooLarge
    );
}

#[test]
fn unterminated_calendar_is_malformed_and_reports_a_plain_reason() {
    let text = "BEGIN:VCALENDAR\nBEGIN:VEVENT\nUID:synthetic-cut-1\n\
        DTSTART:20260915T140000Z\n";
    let error = ics::parse(text, chrono_tz::UTC).unwrap_err();
    assert!(matches!(error, IcsError::Malformed(_)));
    let message = error.to_string();
    assert!(message.contains("END:VCALENDAR"), "{message}");
    assert!(!message.contains("synthetic-cut-1"), "never echoes input");
}

#[test]
fn map_tzid_handles_iana_windows_and_utc_names() {
    assert_eq!(ics::map_tzid("America/Toronto"), Some(America::Toronto));
    assert_eq!(
        ics::map_tzid("\"Eastern Standard Time\""),
        Some(America::New_York)
    );
    assert_eq!(
        ics::map_tzid("Coordinated Universal Time"),
        Some(chrono_tz::UTC)
    );
    assert_eq!(ics::map_tzid("GMT Standard Time"), Some(Europe::London));
    assert_eq!(
        ics::map_tzid("/citadel.org/20190103_1/America/Toronto"),
        Some(America::Toronto)
    );
    assert_eq!(ics::map_tzid("Synthetic/Unknown_Zone"), None);
    assert_eq!(ics::map_tzid("   "), None);
}

#[test]
fn an_empty_window_returns_nothing() {
    let text = include_str!("fixtures/ics/basic-utc.ics");
    let (occurrences, unexpanded) = expand(text, utc(2026, 9, 15, 14, 0), utc(2026, 9, 15, 14, 0));
    assert!(occurrences.is_empty());
    assert_eq!(unexpanded, 0);
}
