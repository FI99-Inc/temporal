use std::cell::Cell;
use temporal_core::clock::{Clock, EvaluationTime, FrozenClock};
use temporal_core::time::{
    Cutoff, CutoffPosition, DateSpan, Instant, LocalDate, LocalDateTime, LocalResolution,
    TemporalSpan, TimeError, TimedSpan, TimezoneRules, ZoneId,
};

fn instant(value: &str) -> Instant {
    value.parse().unwrap()
}
fn date(value: &str) -> LocalDate {
    value.parse().unwrap()
}
fn local(value: &str) -> LocalDateTime {
    value.parse().unwrap()
}
fn toronto() -> ZoneId {
    "America/Toronto".parse().unwrap()
}

#[test]
fn normalized_instants_are_exact_utc_milliseconds() {
    for text in [
        "0001-01-01T00:00:00.000Z",
        "2026-09-07T13:00:00.123Z",
        "9999-12-31T23:59:59.999Z",
    ] {
        assert_eq!(instant(text).to_string(), text);
    }
    for text in [
        "2026-09-07T13:00:00Z",
        "2026-09-07T13:00:00.1234Z",
        "2026-09-07T09:00:00.000-04:00",
        "2026-02-30T00:00:00.000Z",
        "0000-01-01T00:00:00.000Z",
        "2016-12-31T23:59:60.000Z",
    ] {
        assert!(text.parse::<Instant>().is_err(), "{text}");
    }
}

#[test]
fn dates_zones_and_local_precision_are_explicit() {
    assert_eq!(date("2024-02-29").next_day().unwrap(), date("2024-03-01"));
    for text in ["2026-02-29", "2026-9-7", "0000-01-01"] {
        assert!(text.parse::<LocalDate>().is_err());
    }
    assert!("America/Invented".parse::<ZoneId>().is_err());
    assert!("-04:00".parse::<ZoneId>().is_err());
    assert!("2026-09-07T09:00:00".parse::<LocalDateTime>().is_err());
    assert!("2016-12-31T23:59:60.000".parse::<LocalDateTime>().is_err());
}

#[test]
fn arithmetic_never_clamps_or_wraps() {
    let epoch = instant("1970-01-01T00:00:00.000Z");
    assert_eq!(epoch.epoch_ms(), 0);
    assert_eq!(epoch.checked_sub_ms(1).unwrap().epoch_ms(), -1);
    assert_eq!(
        epoch.checked_add_ms(u64::MAX),
        Err(TimeError::ArithmeticOverflow)
    );
    assert!(
        instant("9999-12-31T23:59:59.999Z")
            .checked_add_ms(1)
            .is_err()
    );
    assert!(
        instant("0001-01-01T00:00:00.000Z")
            .checked_sub_ms(1)
            .is_err()
    );
    assert!(date("9999-12-31").next_day().is_err());
    assert!(
        epoch
            .duration_since(epoch.checked_add_ms(1).unwrap())
            .is_err()
    );
    assert!(Instant::from_epoch_ms(i64::MAX).is_err());
}

#[test]
fn timed_intervals_are_positive_and_half_open() {
    let a = instant("2026-09-07T23:30:00.000Z");
    let b = instant("2026-09-08T00:00:00.000Z");
    let c = instant("2026-09-08T00:30:00.000Z");
    let left = TimedSpan::new(a, b).unwrap();
    let right = TimedSpan::new(b, c).unwrap();
    assert!(!left.overlaps(&right));
    assert!(left.intersection(&right).is_none());
    assert!(!left.contains(b));
    assert!(left.contains(a));
    assert_eq!(TimedSpan::new(a, c).unwrap().duration_ms(), 3_600_000);
    assert_eq!(
        left.intersection(&TimedSpan::new(a, c).unwrap()),
        Some(left.clone())
    );
    assert!(TimedSpan::new(a, a).is_err());
    assert!(TimedSpan::new(b, a).is_err());
}

#[test]
fn civil_days_follow_pinned_rules_instead_of_twenty_four_hour_addition() {
    let rules = TimezoneRules::bundled();
    assert_eq!(rules.version(), "2025b");
    let spring = DateSpan::new(date("2026-03-08"), date("2026-03-09"), toronto()).unwrap();
    let fall = DateSpan::new(date("2026-11-01"), date("2026-11-02"), toronto()).unwrap();
    let spring_instants = spring.resolve(&rules).unwrap();
    let fall_instants = fall.resolve(&rules).unwrap();
    assert_eq!(spring_instants.start(), instant("2026-03-08T05:00:00.000Z"));
    assert_eq!(spring_instants.end(), instant("2026-03-09T04:00:00.000Z"));
    assert_eq!(spring_instants.duration_ms(), 23 * 3_600_000);
    assert_eq!(fall_instants.start(), instant("2026-11-01T04:00:00.000Z"));
    assert_eq!(fall_instants.end(), instant("2026-11-02T05:00:00.000Z"));
    assert_eq!(fall_instants.duration_ms(), 25 * 3_600_000);
    assert_eq!(
        TemporalSpan::Dates(fall).resolve(&rules).unwrap(),
        fall_instants
    );
}

#[test]
fn gaps_are_invalid_and_folds_require_an_explicit_choice() {
    let rules = TimezoneRules::bundled();
    let gap = local("2026-03-08T02:30:00.000");
    for choice in [
        LocalResolution::RejectAmbiguous,
        LocalResolution::Earlier,
        LocalResolution::Later,
        LocalResolution::OffsetSeconds(-14_400),
    ] {
        assert_eq!(
            rules.resolve_local(gap, toronto(), choice),
            Err(TimeError::NonexistentLocalTime)
        );
    }
    let fold = local("2026-11-01T01:30:00.000");
    assert_eq!(
        rules.resolve_local(fold, toronto(), LocalResolution::RejectAmbiguous),
        Err(TimeError::AmbiguousLocalTime)
    );
    for choice in [
        LocalResolution::Earlier,
        LocalResolution::OffsetSeconds(-14_400),
    ] {
        assert_eq!(
            rules.resolve_local(fold, toronto(), choice).unwrap(),
            instant("2026-11-01T05:30:00.000Z")
        );
    }
    for choice in [
        LocalResolution::Later,
        LocalResolution::OffsetSeconds(-18_000),
    ] {
        assert_eq!(
            rules.resolve_local(fold, toronto(), choice).unwrap(),
            instant("2026-11-01T06:30:00.000Z")
        );
    }
    assert_eq!(
        rules.resolve_local(fold, toronto(), LocalResolution::OffsetSeconds(0)),
        Err(TimeError::OffsetZoneMismatch)
    );
}

#[test]
fn offset_and_zone_must_agree_and_no_zone_is_invented() {
    let rules = TimezoneRules::bundled();
    let time = local("2026-09-07T09:00:00.000");
    assert_eq!(
        rules.resolve_offset(time, -14_400, None).unwrap(),
        instant("2026-09-07T13:00:00.000Z")
    );
    assert_eq!(
        rules
            .resolve_offset(time, -14_400, Some(toronto()))
            .unwrap(),
        instant("2026-09-07T13:00:00.000Z")
    );
    assert_eq!(
        rules.resolve_offset(time, -18_000, Some(toronto())),
        Err(TimeError::OffsetZoneMismatch)
    );
    assert!(rules.resolve_offset(time, 86_400, None).is_err());
    assert!(
        rules
            .resolve_offset(local("0001-01-01T00:00:00.000"), 3600, None)
            .is_err()
    );
}

#[test]
fn a_missing_or_repeated_civil_boundary_is_not_silently_resolved() {
    let rules = TimezoneRules::bundled();
    assert_eq!(
        rules.start_of_date(date("2011-12-30"), "Pacific/Apia".parse().unwrap()),
        Err(TimeError::UnresolvableCivilBoundary)
    );
    assert_eq!(
        rules.start_of_date(date("2026-11-01"), "America/Havana".parse().unwrap()),
        Err(TimeError::UnresolvableCivilBoundary)
    );
}

#[test]
fn exact_and_whole_date_deadlines_have_different_equality_rules() {
    let rules = TimezoneRules::bundled();
    let endpoint = instant("2026-11-02T05:00:00.000Z");
    let exact = Cutoff::At {
        instant: endpoint,
        original_zone: Some(toronto()),
    };
    let whole_date = Cutoff::OnDate {
        date: date("2026-11-01"),
        zone: toronto(),
    };
    assert_eq!(
        exact.endpoint(&rules).unwrap(),
        whole_date.endpoint(&rules).unwrap()
    );
    assert_eq!(
        exact
            .position(endpoint.checked_sub_ms(1).unwrap(), &rules)
            .unwrap(),
        CutoffPosition::Upcoming
    );
    assert_eq!(
        exact.position(endpoint, &rules).unwrap(),
        CutoffPosition::DueNow
    );
    assert_eq!(
        exact
            .position(endpoint.checked_add_ms(1).unwrap(), &rules)
            .unwrap(),
        CutoffPosition::Overdue
    );
    assert_eq!(
        whole_date
            .position(instant("2026-11-01T03:59:59.999Z"), &rules)
            .unwrap(),
        CutoffPosition::Upcoming
    );
    assert_eq!(
        whole_date
            .position(instant("2026-11-01T04:00:00.000Z"), &rules)
            .unwrap(),
        CutoffPosition::DueToday
    );
    assert_eq!(
        whole_date
            .position(endpoint.checked_sub_ms(1).unwrap(), &rules)
            .unwrap(),
        CutoffPosition::DueToday
    );
    assert_eq!(
        whole_date.position(endpoint, &rules).unwrap(),
        CutoffPosition::Overdue
    );
    assert!(
        Cutoff::OnDate {
            date: date("9999-12-31"),
            zone: toronto()
        }
        .endpoint(&rules)
        .is_err()
    );
}

#[test]
fn display_zone_changes_labels_without_moving_the_deadline() {
    let rules = TimezoneRules::bundled();
    let now = instant("2026-11-02T04:30:00.000Z");
    let cutoff = Cutoff::OnDate {
        date: date("2026-11-01"),
        zone: toronto(),
    };
    assert_eq!(rules.date_at(now, toronto()).unwrap(), date("2026-11-01"));
    assert_eq!(
        rules.date_at(now, "UTC".parse().unwrap()).unwrap(),
        date("2026-11-02")
    );
    assert_eq!(
        cutoff.position(now, &rules).unwrap(),
        CutoffPosition::DueToday
    );
    assert_eq!(
        cutoff.endpoint(&rules).unwrap(),
        instant("2026-11-02T05:00:00.000Z")
    );
}

#[test]
fn evaluation_reads_the_injected_clock_exactly_once() {
    struct CountingClock {
        now: Instant,
        calls: Cell<u32>,
    }
    impl Clock for CountingClock {
        fn now(&self) -> Instant {
            self.calls.set(self.calls.get() + 1);
            self.now
        }
    }
    let now = instant("2026-09-07T13:00:00.000Z");
    let clock = CountingClock {
        now,
        calls: Cell::new(0),
    };
    let captured = now.checked_sub_ms(1).unwrap();
    let evaluation =
        EvaluationTime::capture(captured, now.checked_add_ms(1000).unwrap(), &clock).unwrap();
    assert_eq!(clock.calls.get(), 1);
    assert_eq!(evaluation.now(), now);
    assert_eq!(evaluation.span().duration_ms(), 1000);
    assert_eq!(clock.calls.get(), 1);
}

#[test]
fn virtual_time_advances_without_mutating_snapshot_time() {
    let captured = instant("2026-09-07T13:00:00.000Z");
    let clock = FrozenClock::new(captured);
    let end = captured.checked_add_ms(86_400_000).unwrap();
    clock.advance_ms(1000).unwrap();
    assert_eq!(
        EvaluationTime::capture(captured, end, &clock)
            .unwrap()
            .now(),
        captured.checked_add_ms(1000).unwrap()
    );
    assert_eq!(captured.to_string(), "2026-09-07T13:00:00.000Z");
    assert_eq!(
        EvaluationTime::new(captured, captured.checked_sub_ms(1).unwrap(), end),
        Err(TimeError::EvaluationBeforeSnapshot)
    );
    assert!(EvaluationTime::new(captured, end, end).is_err());
    assert_eq!(
        clock.advance_ms(u64::MAX),
        Err(TimeError::ArithmeticOverflow)
    );
    assert_eq!(clock.now(), captured.checked_add_ms(1000).unwrap());
}
