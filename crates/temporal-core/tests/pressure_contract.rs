use temporal_core::{
    pressure,
    results::{Opportunity, Qualification, Resolution, Risk, Workload},
};

mod support;

#[test]
fn time_steps_change_opportunity_and_risk_but_not_stored_effort() {
    for (name, expected_risk, expected_opportunity) in [
        ("S03/base", Risk::Room, 1_200_u64 * 60_000),
        ("S03/wednesday", Risk::Tight, 600_u64 * 60_000),
        ("S03/thursday", Risk::Insufficient, 300_u64 * 60_000),
    ] {
        let case = support::case(name);
        let before = case.input.clone();
        let row = &pressure::derive(&case.input).unwrap()[0];
        assert_eq!(row.risk, expected_risk, "{name}");
        assert_eq!(row.workload, Some(Workload::Estimate(480)), "{name}");
        assert_eq!(row.ratio.unwrap().work_ms, 480 * 60_000, "{name}");
        assert_eq!(
            row.ratio.unwrap().opportunity_ms.get(),
            expected_opportunity,
            "{name}"
        );
        assert_eq!(case.input, before, "{name}");
    }
}

#[test]
fn equal_endpoints_sort_by_canonical_deadline_identity_without_shared_allocation() {
    let case = support::case("S04/base");
    let rows = pressure::derive(&case.input).unwrap();
    assert_eq!(rows.len(), 6);
    assert!(rows.windows(2).all(|pair| {
        (pair[0].endpoint, pair[0].deadline_id) <= (pair[1].endpoint, pair[1].deadline_id)
    }));
    assert_eq!(rows[0].risk, Risk::Tight);
    assert_eq!(rows[1].risk, Risk::Tight);
    assert!(rows[2..].iter().all(|row| row.risk == Risk::Room));
    assert!(rows.iter().all(|row| {
        row.limitations
            .contains(&temporal_core::reasons::ReasonCode::IndividualCapacityOnly)
    }));
}

#[test]
fn fragmented_capacity_and_zero_opportunity_remain_explicit() {
    let base = support::case("S06/base");
    let row = &pressure::derive(&base.input).unwrap()[0];
    assert_eq!(row.resolution, Resolution::Unresolved);
    assert_eq!(row.workload, Some(Workload::Estimate(240)));
    assert_eq!(row.risk, Risk::Insufficient);
    assert_eq!(row.ratio.unwrap().opportunity_ms.get(), 120 * 60_000);

    let short = support::case("S06/chunk_90");
    let row = &pressure::derive(&short.input).unwrap()[0];
    assert_eq!(row.risk, Risk::Insufficient);
    assert!(matches!(
        row.opportunity,
        Some(Opportunity::Known {
            milliseconds: 0,
            known_qualifying_ms: 0,
            ..
        })
    ));
    assert!(row.ratio.is_none());
    assert!(
        row.reasons
            .iter()
            .any(|reason| reason.code().as_str() == "chunk_too_short")
    );
    assert!(
        row.reasons
            .iter()
            .any(|reason| reason.code().as_str() == "zero_opportunity")
    );
}

#[test]
fn overdue_and_authoritative_resolution_short_circuit_capacity() {
    let overdue = support::case("S09/base");
    let row = &pressure::derive(&overdue.input).unwrap()[0];
    assert_eq!(row.risk, Risk::Overdue);
    assert!(row.workload.is_none());
    assert!(row.opportunity.is_none());
    assert!(row.ratio.is_none());

    let satisfied = support::case("S09/satisfied");
    let row = &pressure::derive(&satisfied.input).unwrap()[0];
    assert_eq!(row.risk, Risk::NotApplicable);
    assert_eq!(row.resolution, Resolution::Satisfied);
    assert!(row.workload.is_none());
    assert!(row.opportunity.is_none());
}

#[test]
fn completion_unknown_and_lower_bound_work_remain_distinct() {
    let case = support::case("S13/base");
    let rows = pressure::derive(&case.input).unwrap();
    assert_eq!(rows.len(), 5);
    let d1 = rows
        .iter()
        .find(|row| row.deadline_id == case.input.deadlines[0].meta.id)
        .unwrap();
    assert_eq!(d1.risk, Risk::NotApplicable);
    let d2 = rows
        .iter()
        .find(|row| row.deadline_id == case.input.deadlines[1].meta.id)
        .unwrap();
    assert_eq!(d2.risk, Risk::NoKnownWork);
    assert_eq!(d2.workload, Some(Workload::Estimate(0)));
    assert!(
        d2.reasons
            .iter()
            .any(|reason| reason.code().as_str() == "task_completion_basis")
    );
    let d3 = rows
        .iter()
        .find(|row| row.deadline_id == case.input.deadlines[2].meta.id)
        .unwrap();
    assert_eq!(d3.risk, Risk::Unknown);
    assert_eq!(d3.workload, Some(Workload::Unknown));
    assert!(d3.opportunity.is_none());
    let d4 = rows
        .iter()
        .find(|row| row.deadline_id == case.input.deadlines[3].meta.id)
        .unwrap();
    assert_eq!(d4.risk, Risk::Insufficient);
    assert_eq!(
        d4.workload,
        Some(Workload::AtLeast(std::num::NonZeroU32::new(240).unwrap()))
    );
    assert!(d4.ratio.is_some());
    let d5 = rows
        .iter()
        .find(|row| row.deadline_id == case.input.deadlines[4].meta.id)
        .unwrap();
    assert_eq!(d5.risk, Risk::NoKnownWork);
    assert_eq!(d5.workload, Some(Workload::Estimate(0)));
}

#[test]
fn authoritative_task_removal_does_not_preserve_a_done_work_conclusion() {
    let case = support::case("S13/authoritative_removal");
    let rows = pressure::derive(&case.input).unwrap();
    let independent = rows
        .iter()
        .find(|row| row.deadline_id == case.input.deadlines[1].meta.id)
        .unwrap();
    assert_eq!(independent.risk, Risk::Unknown);
    assert_eq!(independent.workload, Some(Workload::Unknown));
    assert!(independent.opportunity.is_none());
    assert!(
        independent
            .reasons
            .iter()
            .any(|reason| reason.code().as_str() == "work_unavailable")
    );
}

#[test]
fn source_health_qualifies_known_numbers_without_erasing_them() {
    let fresh = support::case("S12/freshness_equality");
    let row = &pressure::derive(&fresh.input).unwrap()[0];
    assert_eq!(row.risk, Risk::Tight);
    assert_eq!(row.qualification, Qualification::DeclaredInputs);
    assert_eq!(row.ratio.unwrap().opportunity_ms.get(), 90 * 60_000);

    let stale = support::case("S12/freshness_plus_ms");
    let row = &pressure::derive(&stale.input).unwrap()[0];
    assert_eq!(row.risk, Risk::Insufficient);
    assert_eq!(row.qualification, Qualification::Conditional);
    assert_eq!(row.ratio.unwrap().opportunity_ms.get(), 60 * 60_000);
}

#[test]
fn threshold_boundaries_keep_unreduced_integer_ratios() {
    for (name, expected_risk, expected_work_minutes) in [
        ("S16/effort_59", Risk::Room, 59_u64),
        ("S16/effort_60", Risk::Tight, 60),
        ("S16/effort_120", Risk::Tight, 120),
        ("S16/effort_121", Risk::Insufficient, 121),
    ] {
        let row = &pressure::derive(&support::case(name).input).unwrap()[0];
        assert_eq!(row.risk, expected_risk, "{name}");
        let ratio = row.ratio.unwrap();
        assert_eq!(ratio.work_ms, expected_work_minutes * 60_000, "{name}");
        assert_eq!(ratio.opportunity_ms.get(), 120 * 60_000, "{name}");
    }

    let unknown = &pressure::derive(&support::case("S16/unknown_effort").input).unwrap()[0];
    assert_eq!(unknown.risk, Risk::Unknown);
    assert_eq!(unknown.workload, Some(Workload::Unknown));
    assert!(unknown.opportunity.is_none());
    assert!(unknown.ratio.is_none());

    let zero = &pressure::derive(&support::case("S16/zero_effort").input).unwrap()[0];
    assert_eq!(zero.risk, Risk::NoKnownWork);
    assert_eq!(zero.workload, Some(Workload::Estimate(0)));
    assert!(zero.opportunity.is_none());
    assert!(zero.ratio.is_none());
}

#[test]
fn exact_cutoff_equality_is_due_now_with_known_zero_and_plus_millisecond_is_overdue() {
    let equality = &pressure::derive(&support::case("S11/exact_equality").input).unwrap()[0];
    assert_eq!(
        equality.phase,
        temporal_core::results::DeadlinePhase::DueNow
    );
    assert_eq!(equality.risk, Risk::Insufficient);
    assert!(matches!(
        equality.opportunity,
        Some(Opportunity::Known {
            milliseconds: 0,
            known_qualifying_ms: 0,
            ..
        })
    ));
    assert!(equality.ratio.is_none());

    let overdue = &pressure::derive(&support::case("S11/exact_plus_ms").input).unwrap()[0];
    assert_eq!(
        overdue.phase,
        temporal_core::results::DeadlinePhase::Overdue
    );
    assert_eq!(overdue.risk, Risk::Overdue);
    assert!(overdue.workload.is_none());
    assert!(overdue.opportunity.is_none());
    assert!(overdue.ratio.is_none());
}

#[test]
fn source_staleness_is_conditional_even_when_effective_work_is_zero() {
    let row = &pressure::derive(&support::case("S13/stale_trace").input).unwrap()[1];
    assert_eq!(row.risk, Risk::NoKnownWork);
    assert_eq!(row.qualification, Qualification::Conditional);
    assert!(
        row.source_qualifications
            .iter()
            .any(|qualification| { qualification.health == temporal_core::results::Health::Stale })
    );
    assert!(row.ratio.is_none());
}

#[test]
fn input_order_and_importance_or_trace_priority_do_not_change_pressure_arithmetic() {
    let base = support::case("S04/base");
    let expected = pressure::derive(&base.input).unwrap();
    let mut permuted = base.clone();
    permuted.input.deadlines.reverse();
    permuted.input.availability.reverse();
    assert_eq!(pressure::derive(&permuted.input).unwrap(), expected);

    let importance = support::case("S16/importance_only");
    let ordinary = support::case("S16/base");
    let ordinary_row = &pressure::derive(&ordinary.input).unwrap()[0];
    let importance_row = &pressure::derive(&importance.input).unwrap()[0];
    assert_eq!(importance_row.risk, ordinary_row.risk);
    assert_eq!(importance_row.workload, ordinary_row.workload);
    match (&importance_row.opportunity, &ordinary_row.opportunity) {
        (
            Some(Opportunity::Known {
                milliseconds: left,
                known_qualifying_ms: left_known,
                ..
            }),
            Some(Opportunity::Known {
                milliseconds: right,
                known_qualifying_ms: right_known,
                ..
            }),
        ) => {
            assert_eq!(left, right);
            assert_eq!(left_known, right_known);
        }
        (left, right) => panic!("importance changed opportunity shape: {left:?} != {right:?}"),
    }
    assert_eq!(importance_row.ratio, ordinary_row.ratio);

    let task_base = support::case("S06/base");
    let mut priority = task_base.clone();
    priority.input.task_refs[0].source_priority = Some("synthetic-high".into());
    let base_row = &pressure::derive(&task_base.input).unwrap()[0];
    let priority_row = &pressure::derive(&priority.input).unwrap()[0];
    assert_eq!(priority_row.risk, base_row.risk);
    assert_eq!(priority_row.workload, base_row.workload);
    assert_eq!(priority_row.opportunity, base_row.opportunity);
    assert_eq!(priority_row.ratio, base_row.ratio);
}

#[test]
fn unknown_availability_and_mathematical_zero_are_not_interchangeable() {
    let unknown = support::case("S15/no_availability");
    let row = &pressure::derive(&unknown.input).unwrap()[0];
    assert_eq!(row.risk, Risk::Unknown);
    assert!(matches!(row.opportunity, Some(Opportunity::Unknown { .. })));
    assert!(row.ratio.is_none());

    let zero = support::case("S15/earliest_at_cutoff");
    let row = &pressure::derive(&zero.input).unwrap()[0];
    assert_eq!(row.risk, Risk::Insufficient);
    assert!(matches!(
        row.opportunity,
        Some(Opportunity::Known {
            milliseconds: 0,
            known_qualifying_ms: 0,
            ..
        })
    ));
    assert!(row.ratio.is_none());
}

#[test]
fn range_end_plus_millisecond_is_unknown_without_partial_ratio() {
    let case = support::case("S16/range_plus_ms");
    let row = &pressure::derive(&case.input).unwrap()[0];
    assert_eq!(row.risk, Risk::Unknown);
    assert!(row.workload.is_none());
    assert!(row.opportunity.is_none());
    assert!(row.ratio.is_none());
    assert!(
        row.reasons
            .iter()
            .any(|reason| reason.code().as_str() == "outside_evaluation_range")
    );
}

#[test]
fn every_valid_synthetic_snapshot_produces_deterministic_pressure_rows() {
    for case in support::all_cases() {
        let before = case.input.clone();
        let first = pressure::derive(&case.input)
            .unwrap_or_else(|error| panic!("{} failed pressure derivation: {error}", case.name));
        let second = pressure::derive(&case.input).unwrap();
        assert_eq!(first, second, "{}", case.name);
        assert_eq!(case.input, before, "{}", case.name);
        assert!(first.windows(2).all(|pair| {
            (pair[0].endpoint, pair[0].deadline_id) <= (pair[1].endpoint, pair[1].deadline_id)
        }));
    }
}
