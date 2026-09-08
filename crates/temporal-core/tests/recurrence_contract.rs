use temporal_core::{
    recurrence,
    results::OccurrencePhase,
    time::{LocalDate, TimezoneRules},
};

mod support;

#[test]
fn weekly_expansion_is_finite_date_keyed_and_includes_the_intersecting_end_date() {
    let case = support::case("S14/base");
    let occurrences = recurrence::expand_routines(&case.input).unwrap();
    assert_eq!(occurrences.len(), 3);
    assert_eq!(
        occurrences
            .iter()
            .map(|occurrence| occurrence.key.date.to_string())
            .collect::<Vec<_>>(),
        vec!["2026-09-06", "2026-09-13", "2026-09-20"]
    );
    assert_eq!(occurrences[0].phase, OccurrencePhase::Current);
    assert_eq!(occurrences[1].phase, OccurrencePhase::Future);
    assert_eq!(occurrences[2].phase, OccurrencePhase::Future);
    assert!(
        occurrences
            .iter()
            .all(|occurrence| occurrence.span.zone().name() == "America/Toronto")
    );
}

#[test]
fn occurrence_state_can_inspect_history_without_reintroducing_a_candidate() {
    let case = support::case("S14/base");
    let routine = &case.input.routines[0];
    let date: LocalDate = "2026-09-06".parse().unwrap();
    let now = "2026-09-07T13:00:00.000Z".parse().unwrap();
    let row = recurrence::occurrence_state(
        routine,
        &case.input.routine_outcomes,
        date,
        now,
        &TimezoneRules::bundled(),
    )
    .unwrap();
    assert_eq!(row.phase, OccurrencePhase::PastUnrecorded);
    assert!(
        row.reasons
            .iter()
            .any(|reason| reason.code().as_str() == "routine_past_unrecorded")
    );

    let mut changed = case.input.clone();
    changed.evaluation.now = now;
    let candidates = recurrence::eligible_occurrences(&changed).unwrap();
    assert!(
        candidates
            .iter()
            .all(|occurrence| occurrence.key.date != date)
    );
}

#[test]
fn outcomes_and_rule_edits_are_explicit_without_recurrence_debt() {
    let skipped = support::case("S14/skipped");
    let all = recurrence::expand_routines(&skipped.input).unwrap();
    assert_eq!(all[0].phase, OccurrencePhase::Skipped);
    assert!(
        recurrence::eligible_occurrences(&skipped.input)
            .unwrap()
            .iter()
            .all(|occurrence| occurrence.phase == OccurrencePhase::Future)
    );

    let edited = support::case("S14/edited_rule");
    let edited_dates: Vec<_> = recurrence::expand_routines(&edited.input)
        .unwrap()
        .into_iter()
        .map(|occurrence| occurrence.key.date.to_string())
        .collect();
    assert_eq!(edited_dates, vec!["2026-09-07", "2026-09-14"]);
    let old = recurrence::occurrence_state(
        &edited.input.routines[0],
        &edited.input.routine_outcomes,
        "2026-09-06".parse().unwrap(),
        edited.input.evaluation.now,
        &TimezoneRules::bundled(),
    )
    .unwrap();
    assert_eq!(old.phase, OccurrencePhase::Inactive);

    let paused = support::case("S14/paused");
    assert!(
        recurrence::expand_routines(&paused.input)
            .unwrap()
            .is_empty()
    );
    assert!(paused.input.routine_outcomes.is_empty());
}

#[test]
fn same_date_routines_keep_distinct_keys_and_no_shared_state() {
    let case = support::case("S14/second_routine");
    let rows = recurrence::expand_routines(&case.input).unwrap();
    assert_eq!(rows.len(), 6);
    let same_date: Vec<_> = rows
        .iter()
        .filter(|row| row.key.date.to_string() == "2026-09-06")
        .collect();
    assert_eq!(same_date.len(), 2);
    assert_ne!(same_date[0].key.routine_id, same_date[1].key.routine_id);
}
