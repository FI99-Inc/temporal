use temporal_core::{
    domain::{SourceKind, SourceRole},
    results::Health,
    source_health::{
        CoverageNeed, DependencyNeed, dependencies_for_deadline, derive_health, qualify,
        source_health,
    },
    time::TimedSpan,
};

mod support;

fn source_id(case: &support::Case, kind: SourceKind) -> temporal_core::domain::SourceId {
    case.input
        .sources
        .iter()
        .find(|source| source.kind == kind)
        .unwrap()
        .id
}

fn google_need(case: &support::Case) -> DependencyNeed {
    DependencyNeed {
        source_id: source_id(case, SourceKind::Google),
        role: SourceRole::Anchors,
        coverage: CoverageNeed::AnchorInterval {
            interval: TimedSpan::new(
                "2026-09-07T13:00:00.000Z".parse().unwrap(),
                "2026-09-07T16:00:00.000Z".parse().unwrap(),
            )
            .unwrap(),
        },
    }
}

#[test]
fn health_precedence_preserves_last_known_records_and_freshness_equality() {
    for (name, expected) in [
        ("S12/base", Health::Healthy),
        ("S12/failed", Health::Unavailable),
        ("S12/partial", Health::Partial),
        ("S12/incompatible", Health::Incompatible),
        ("S12/freshness_equality", Health::Healthy),
        ("S12/freshness_plus_ms", Health::Stale),
        ("S12/never_loaded_empty", Health::NeverLoaded),
        ("S12/successful_empty", Health::Healthy),
    ] {
        let case = support::case(name);
        let google = source_id(&case, SourceKind::Google);
        let row = source_health(&case.input)
            .unwrap()
            .into_iter()
            .find(|row| row.source_id == google)
            .unwrap();
        assert_eq!(row.health, expected, "{name}");
        if expected != Health::Healthy || name != "S12/successful_empty" {
            assert!(
                row.reasons
                    .iter()
                    .any(|reason| reason.code().as_str() == "source_health")
            );
        }
    }
    let case = support::case("S12/base");
    let local = source_id(&case, SourceKind::Local);
    assert_eq!(
        source_health(&case.input)
            .unwrap()
            .into_iter()
            .find(|row| row.source_id == local)
            .unwrap()
            .health,
        Health::Healthy
    );
}

#[test]
fn coverage_is_separate_from_health_and_uses_half_open_boundaries() {
    let case = support::case("S12/base");
    assert!(qualify(&case.input, &google_need(&case)).unwrap().covered);

    let short = support::case("S12/short_coverage");
    let need = google_need(&short);
    assert!(!qualify(&short.input, &need).unwrap().covered);

    let mut at_end = support::case("S12/base");
    let google = source_id(&at_end, SourceKind::Google);
    let state = at_end
        .input
        .source_states
        .iter_mut()
        .find(|state| state.source_id == google)
        .unwrap();
    state.anchor_coverage = Some(
        TimedSpan::new(
            "2026-09-07T13:00:00.000Z".parse().unwrap(),
            "2026-09-07T16:00:00.000Z".parse().unwrap(),
        )
        .unwrap(),
    );
    state.deadline_coverage = Some(
        TimedSpan::new(
            "2026-09-07T13:00:00.000Z".parse().unwrap(),
            "2026-09-07T16:00:00.000Z".parse().unwrap(),
        )
        .unwrap(),
    );
    let exact = google_need(&at_end);
    assert!(qualify(&at_end.input, &exact).unwrap().covered);
    let endpoint = match exact.coverage {
        CoverageNeed::AnchorInterval { interval } => interval.end(),
        _ => unreachable!(),
    };
    let deadline_need = DependencyNeed {
        source_id: google,
        role: SourceRole::Deadlines,
        coverage: CoverageNeed::DeadlineEndpoint { endpoint },
    };
    assert!(!qualify(&at_end.input, &deadline_need).unwrap().covered);

    let mut overflow = support::case("S12/base");
    overflow.input.source_states[0].fresh_for_ms = std::num::NonZeroU64::new(u64::MAX).unwrap();
    assert_eq!(
        derive_health(
            &overflow.input.source_states[0],
            overflow.input.evaluation.now,
        )
        .unwrap_err(),
        temporal_core::time::TimeError::ArithmeticOverflow
    );
}

#[test]
fn dependency_builder_adds_factual_sources_even_when_explicit_requirements_omit_them() {
    let case = support::case("S13/stale_trace_implicit_dependency");
    let deadline = &case.input.deadlines[0];
    let needs = dependencies_for_deadline(
        &case.input,
        deadline,
        deadline
            .cutoff
            .endpoint(&temporal_core::time::TimezoneRules::bundled())
            .unwrap(),
        None,
    )
    .unwrap();
    let trace = source_id(&case, SourceKind::Trace);
    assert!(
        needs
            .iter()
            .any(|need| need.source_id == trace && need.role == SourceRole::Tasks)
    );
    assert!(
        !needs
            .iter()
            .any(|need| { need.source_id == trace && need.role == SourceRole::Deadlines })
    );

    let reduced = support::case("S13/unknown_independent_work");
    let deadline = &reduced.input.deadlines[0];
    let needs = dependencies_for_deadline(
        &reduced.input,
        deadline,
        deadline
            .cutoff
            .endpoint(&temporal_core::time::TimezoneRules::bundled())
            .unwrap(),
        None,
    )
    .unwrap();
    let quercus = source_id(&reduced, SourceKind::Quercus);
    assert!(
        needs
            .iter()
            .any(|need| need.source_id == quercus && need.role == SourceRole::Deadlines)
    );
    assert!(
        needs
            .iter()
            .any(|need| need.source_id == trace && need.role == SourceRole::Tasks)
    );
}

#[test]
fn trace_due_uses_task_catalog_coverage_without_a_deadline_interval() {
    let case = support::case("S13/unknown_own_due");
    let deadline = &case.input.deadlines[0];
    let trace = source_id(&case, SourceKind::Trace);
    let needs = dependencies_for_deadline(
        &case.input,
        deadline,
        deadline
            .cutoff
            .endpoint(&temporal_core::time::TimezoneRules::bundled())
            .unwrap(),
        None,
    )
    .unwrap();
    assert!(needs.iter().any(|need| {
        need.source_id == trace
            && need.role == SourceRole::Tasks
            && matches!(need.coverage, CoverageNeed::TasksCatalog)
    }));
    assert!(
        !needs
            .iter()
            .any(|need| need.source_id == trace && need.role == SourceRole::Deadlines)
    );
}
