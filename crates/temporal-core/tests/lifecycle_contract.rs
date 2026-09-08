use temporal_core::{
    lifecycle,
    results::{AnchorPhase, DeadlinePhase, IntentionPhase, Resolution},
    time::TimezoneRules,
};

mod support;

#[test]
fn anchor_phases_use_half_open_span_boundaries_and_removed_is_inactive() {
    let case = support::case("S05/base");
    let anchor = &case.input.anchors[0];
    let rules = TimezoneRules::bundled();

    assert_eq!(
        lifecycle::anchor_state(anchor, "2026-09-07T13:00:00.000Z".parse().unwrap(), &rules)
            .unwrap()
            .phase,
        AnchorPhase::Upcoming
    );
    assert_eq!(
        lifecycle::anchor_state(anchor, "2026-09-07T14:00:00.000Z".parse().unwrap(), &rules)
            .unwrap()
            .phase,
        AnchorPhase::Ongoing
    );
    assert_eq!(
        lifecycle::anchor_state(anchor, "2026-09-07T15:30:00.000Z".parse().unwrap(), &rules)
            .unwrap()
            .phase,
        AnchorPhase::Passed
    );
    let retired = support::case("S11/retired_anchor");
    assert_eq!(
        lifecycle::anchor_state(
            &retired.input.anchors[0],
            retired.input.evaluation.now,
            &rules,
        )
        .unwrap()
        .phase,
        AnchorPhase::Inactive
    );
}

#[test]
fn deadlines_keep_resolution_authority_separate_from_cutoff_phase() {
    let rules = TimezoneRules::bundled();
    let overdue = support::case("S09/base");
    let row = lifecycle::deadline_state(
        &overdue.input.deadlines[0],
        &overdue.input.task_refs,
        overdue.input.evaluation.now,
        &rules,
    )
    .unwrap();
    assert_eq!(row.resolution, Resolution::Unresolved);
    assert_eq!(row.phase, DeadlinePhase::Overdue);

    let satisfied = support::case("S09/satisfied");
    let row = lifecycle::deadline_state(
        &satisfied.input.deadlines[0],
        &satisfied.input.task_refs,
        satisfied.input.evaluation.now,
        &rules,
    )
    .unwrap();
    assert_eq!(row.resolution, Resolution::Satisfied);
    assert_eq!(row.phase, DeadlinePhase::Inactive);

    let exact = support::case("S11/exact_base");
    let deadline = &exact.input.deadlines[0];
    let endpoint = deadline.cutoff.endpoint(&rules).unwrap();
    assert_eq!(
        lifecycle::deadline_state(deadline, &exact.input.task_refs, endpoint, &rules)
            .unwrap()
            .phase,
        DeadlinePhase::DueNow
    );
    assert_eq!(
        lifecycle::deadline_state(
            deadline,
            &exact.input.task_refs,
            endpoint.checked_add_ms(1).unwrap(),
            &rules,
        )
        .unwrap()
        .phase,
        DeadlinePhase::Overdue
    );

    let date = support::case("S11/base");
    let date_deadline = &date.input.deadlines[0];
    let endpoint = date_deadline.cutoff.endpoint(&rules).unwrap();
    assert_eq!(
        lifecycle::deadline_state(
            date_deadline,
            &date.input.task_refs,
            date.input.evaluation.now,
            &rules,
        )
        .unwrap()
        .phase,
        DeadlinePhase::DueToday
    );
    assert_eq!(
        lifecycle::deadline_state(date_deadline, &date.input.task_refs, endpoint, &rules)
            .unwrap()
            .phase,
        DeadlinePhase::Overdue
    );
}

#[test]
fn trace_completion_and_unknown_status_never_invent_deadline_evidence() {
    let rules = TimezoneRules::bundled();
    let base = support::case("S13/base");
    let done_projection = lifecycle::deadline_state(
        &base.input.deadlines[0],
        &base.input.task_refs,
        base.input.evaluation.now,
        &rules,
    )
    .unwrap();
    assert_eq!(done_projection.resolution, Resolution::Satisfied);
    assert!(
        done_projection
            .reasons
            .iter()
            .any(|reason| reason.code().as_str() == "task_completion_basis")
    );

    let independent = lifecycle::deadline_state(
        &base.input.deadlines[1],
        &base.input.task_refs,
        base.input.evaluation.now,
        &rules,
    )
    .unwrap();
    assert_eq!(independent.resolution, Resolution::Unresolved);
    assert_ne!(independent.phase, DeadlinePhase::Inactive);

    let unknown = support::case("S13/unknown_own_due");
    let row = lifecycle::deadline_state(
        &unknown.input.deadlines[0],
        &unknown.input.task_refs,
        unknown.input.evaluation.now,
        &rules,
    )
    .unwrap();
    assert_eq!(row.resolution, Resolution::Unknown);
    assert_eq!(row.phase, DeadlinePhase::Unknown);
}

#[test]
fn intention_preference_passage_keeps_active_state_soft() {
    let case = support::case("S07/base");
    let rules = TimezoneRules::bundled();
    let states = case
        .input
        .intentions
        .iter()
        .map(|intention| {
            lifecycle::intention_state(intention, case.input.evaluation.now, &rules).unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(states[0].phase, IntentionPhase::PreferencePassed);
    assert_eq!(states[1].phase, IntentionPhase::PreferredNow);
    assert!(
        states[0]
            .reasons
            .iter()
            .any(|reason| reason.code().as_str() == "soft_preference_passed")
    );
    assert!(
        case.input
            .intentions
            .iter()
            .all(|intention| intention.state == temporal_core::domain::IntentionState::Active)
    );
}

#[test]
fn lifecycle_output_is_sorted_and_does_not_mutate_the_snapshot() {
    let case = support::case("S14/base");
    let before = case.input.clone();
    let output = lifecycle::evaluate_lifecycle(&case.input).unwrap();
    assert_eq!(output.anchor_states.len(), case.input.anchors.len());
    assert_eq!(output.deadline_states.len(), case.input.deadlines.len());
    assert_eq!(output.intention_states.len(), case.input.intentions.len());
    assert_eq!(case.input, before);
    assert!(
        output
            .routine_occurrences
            .windows(2)
            .all(|window| window[0].key <= window[1].key)
    );
}
