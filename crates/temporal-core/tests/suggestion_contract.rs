use temporal_core::{evaluate, reasons::ReasonCode, results::*};

mod support;

#[test]
fn old_advice_expires_without_turning_flexible_work_into_obligations() {
    for (name, status) in [
        ("S10/base", SuggestionStatus::Current),
        ("S10/expiry_equality", SuggestionStatus::Expired),
        ("S10/tuesday", SuggestionStatus::Expired),
        ("S10/trace_done", SuggestionStatus::Invalidated),
    ] {
        let case = support::case(name);
        let before = case.input.clone();
        let output = evaluate(&case.input).unwrap();
        assert_eq!(output.suggestion_validity[0].status, status, "{name}");
        assert!(output.deadline_states.is_empty(), "{name}");
        assert!(output.pressures.is_empty(), "{name}");
        assert_eq!(case.input, before, "{name}");
        if name == "S10/trace_done" {
            assert!(output.fits.is_empty());
        }
    }
}

#[test]
fn advice_rechecks_its_proposed_span_and_risk_against_the_current_clock() {
    let case = support::case("S16/half_hour_later");
    let output = evaluate(&case.input).unwrap();
    let work = &output.suggestion_validity[0];
    let risk = &output.suggestion_validity[1];
    assert_eq!(work.status, SuggestionStatus::Ineligible);
    assert!(
        work.reasons
            .iter()
            .any(|reason| reason.code() == ReasonCode::ChunkTooShort)
    );
    assert_eq!(risk.status, SuggestionStatus::Current);
    assert_eq!(risk.reasons, output.pressures[0].reasons);
    assert_eq!(
        output.pressures[0].ratio.unwrap().opportunity_ms.get(),
        90 * 60_000
    );
    assert!(output.fits.iter().all(|row| row.status == FitStatus::Fits));
    for name in ["S16/new_snapshot_basis", "S16/new_settings_basis"] {
        assert!(
            evaluate(&support::case(name).input)
                .unwrap()
                .suggestion_validity
                .iter()
                .all(|row| row.status == SuggestionStatus::Invalidated)
        );
    }
    let room = evaluate(&support::case("S16/current_basis_room_notice").input).unwrap();
    assert_eq!(
        room.suggestion_validity[1].status,
        SuggestionStatus::Ineligible
    );
}

#[test]
fn invalid_snapshots_fail_before_any_partial_evaluation() {
    let mut input = support::case("S16/base").input;
    input.availability.push(input.availability[0].clone());
    assert!(evaluate(&input).is_err());
}

fn consider(input: &temporal_core::domain::EvaluationInput, target: WorkTarget) -> Suggestion {
    Suggestion {
        key: SuggestionKey {
            evaluation_key: input.evaluation.key(input.snapshot_revision),
            kind: SuggestionKind::ConsiderWork,
            target,
            proposed_span: None,
        },
        kind: SuggestionKind::ConsiderWork,
        target,
        proposed_span: None,
        created_at: input.evaluation.now,
        valid_until: input.evaluation.evaluation_end,
        reasons: vec![temporal_core::reasons::Reason::DeclaredAvailability {
            references: vec![
                Reference::from(target),
                Reference::Availability(input.availability[0].meta.id),
            ],
            payload: temporal_core::reasons::AvailabilityPayload {
                span: input.availability[0].span.clone(),
                clipped_span: Some(input.availability[0].span.clone()),
            },
        }],
    }
}

#[test]
fn current_work_advice_retains_implicit_task_and_failed_calendar_source_evidence() {
    let mut input = support::case("S16/base").input;
    input.required_sources.clear();
    input.source_states[0].fresh_for_ms = std::num::NonZeroU64::new(1).unwrap();
    input.prior_suggestions = vec![consider(
        &input,
        WorkTarget::Task(input.task_refs[0].meta.id),
    )];
    input.evaluation.now = input.evaluation.now.checked_add_ms(1).unwrap();
    let output = evaluate(&input).unwrap();
    let row = &output.suggestion_validity[0];
    assert_eq!(row.status, SuggestionStatus::Current);
    assert!(row.reasons.iter().any(|reason|matches!(reason,
        temporal_core::reasons::Reason::SourceHealth {payload,..} if payload.health==Health::Stale)));

    let mut input = support::case("S12/failed").input;
    input.prior_suggestions = vec![consider(
        &input,
        WorkTarget::Deadline(input.deadlines[0].meta.id),
    )];
    let output = evaluate(&input).unwrap();
    let row = &output.suggestion_validity[0];
    assert_eq!(row.status, SuggestionStatus::Current);
    assert!(row.reasons.iter().any(|reason|matches!(reason,
        temporal_core::reasons::Reason::SourceHealth {payload,..} if payload.health==Health::Unavailable)));
}

#[test]
fn a_routine_preference_can_become_ineligible_before_advice_expires() {
    let mut input = support::case("S14/base").input;
    let target = WorkTarget::RoutineOccurrence(OccurrenceKey {
        routine_id: input.routines[0].meta.id,
        date: "2026-09-06".parse().unwrap(),
    });
    input.prior_suggestions = vec![consider(&input, target)];
    assert_eq!(
        evaluate(&input).unwrap().suggestion_validity[0].status,
        SuggestionStatus::Current
    );
    input.evaluation.now = "2026-09-07T13:00:00.000Z".parse().unwrap();
    let before = input.clone();
    let output = evaluate(&input).unwrap();
    assert_eq!(
        output.suggestion_validity[0].status,
        SuggestionStatus::Ineligible
    );
    assert!(
        output
            .routine_occurrences
            .iter()
            .all(|r| r.key.date.to_string() != "2026-09-06")
    );
    assert_eq!(input, before);
}

#[test]
fn soft_intention_advice_needs_current_fit_but_not_an_unpassed_preference() {
    let mut input = support::case("S07/declared_evening").input;
    input.prior_suggestions = input
        .intentions
        .iter()
        .map(|i| consider(&input, WorkTarget::Intention(i.meta.id)))
        .collect();
    let output = evaluate(&input).unwrap();
    assert_eq!(
        output.suggestion_validity[0].status,
        SuggestionStatus::Current
    );
    assert_eq!(
        output.suggestion_validity[1].status,
        SuggestionStatus::Ineligible
    );
    assert!(
        output.suggestion_validity[1]
            .reasons
            .iter()
            .any(|r| r.code() == ReasonCode::ChunkUnknown)
    );
    assert!(output.pressures.is_empty());
}

#[test]
fn current_notices_use_recalculated_risk_for_every_short_circuit() {
    for (name, current) in [
        ("S09/base", true),
        ("S09/satisfied", false),
        ("S13/unknown_own_due", true),
        ("S16/unknown_effort", true),
        ("S16/zero_effort", false),
        ("S16/effort_59", false),
    ] {
        let mut input = support::case(name).input;
        let id = input.deadlines[0].meta.id;
        let target = WorkTarget::Deadline(id);
        let mut advice = consider(&input, target);
        advice.kind = SuggestionKind::RiskNotice;
        advice.key.kind = SuggestionKind::RiskNotice;
        advice.reasons = vec![temporal_core::reasons::Reason::IndividualCapacityOnly {
            references: vec![Reference::Pressure(PressureKey {
                evaluation_key: advice.key.evaluation_key.clone(),
                deadline_id: id,
            })],
            payload: temporal_core::reasons::EmptyPayload {},
        }];
        input.prior_suggestions = vec![advice];
        let output = evaluate(&input).unwrap();
        assert_eq!(
            output.suggestion_validity[0].status,
            if current {
                SuggestionStatus::Current
            } else {
                SuggestionStatus::Ineligible
            },
            "{name}"
        );
        assert_eq!(
            output.suggestion_validity[0].reasons, output.pressures[0].reasons,
            "{name}"
        );
    }
}
