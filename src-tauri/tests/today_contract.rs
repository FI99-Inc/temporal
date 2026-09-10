#[path = "../../crates/temporal-core/tests/support/bases.rs"]
mod bases;
#[allow(dead_code)]
#[path = "../../crates/temporal-core/tests/support/builder.rs"]
mod builder;

use temporal_core::{domain::*, results::*, time::*};

fn case(id: &str) -> EvaluationInput {
    bases::all()
        .into_iter()
        .find(|c| c.name == format!("{id}/base"))
        .unwrap()
        .input
}

#[test]
fn every_scenario_has_a_bounded_repeatable_edit_and_valid_advice_without_writes() {
    for case in bases::all() {
        let input = case.input;
        let before = input.clone();
        let (_, first) = temporal_app::today::evaluate(&input).unwrap();
        let (_, second) = temporal_app::today::evaluate(&input).unwrap();
        assert_eq!(first, second);
        assert_eq!(input, before);
        assert!(first.worth_doing.len() <= 3 && first.loose.len() <= 2 && first.radar.len() <= 3);
        let mut checking = input.clone();
        checking.prior_suggestions = first
            .worth_doing
            .iter()
            .chain(&first.loose)
            .map(|r| r.suggestion.clone())
            .collect();
        let output = temporal_core::evaluate(&checking).unwrap();
        assert!(
            output
                .suggestion_validity
                .iter()
                .all(|r| r.status == SuggestionStatus::Current)
        );
        let mut permuted = input.clone();
        permuted.anchors.reverse();
        permuted.deadlines.reverse();
        permuted.task_refs.reverse();
        permuted.intentions.reverse();
        permuted.routines.reverse();
        permuted.availability.reverse();
        permuted.source_states.reverse();
        permuted.task_annotations.reverse();
        let (_, reversed) = temporal_app::today::evaluate(&permuted).unwrap();
        assert_eq!(first, reversed);
    }
}

#[test]
fn missing_opportunity_does_not_fill_the_edit_and_expired_advice_is_not_a_deadline() {
    let mut input = case("S07");
    input.availability.clear();
    let (_, edit) = temporal_app::today::evaluate(&input).unwrap();
    assert!(edit.worth_doing.is_empty() && edit.loose.is_empty());
    let mut expired = case("S10");
    expired.evaluation.now = expired.evaluation.now.checked_add_ms(86_400_000).unwrap();
    let (output, edit) = temporal_app::today::evaluate(&expired).unwrap();
    assert!(
        output
            .suggestion_validity
            .iter()
            .any(|r| r.status == SuggestionStatus::Expired)
    );
    assert!(edit.fixed.is_empty() && edit.radar.is_empty());
    assert!(expired.deadlines.is_empty());
    let overdue = case("S09");
    let (_, edit) = temporal_app::today::evaluate(&overdue).unwrap();
    assert!(
        edit.radar
            .iter()
            .any(|r| matches!(r, temporal_app::today::FactRef::Deadline(_)))
    );
    assert!(edit.worth_doing.is_empty());
}

#[test]
fn clipping_at_civil_midnight_rechecks_useful_session_length() {
    let mut input = case("S01");
    input.anchors.clear();
    input.anchor_annotations.clear();
    input.evaluation.now = "2026-09-08T03:50:00.000Z".parse().unwrap();
    input.availability[0].span = TimedSpan::new(
        input.evaluation.now,
        input.evaluation.now.checked_add_ms(3_600_000).unwrap(),
    )
    .unwrap();
    input.availability.truncate(1);
    input.task_annotations[0].work.effort = Effort::Estimate(60);
    input.task_annotations[0].work.minimum_chunk_minutes = std::num::NonZeroU32::new(30);
    let (_, edit) = temporal_app::today::evaluate(&input).unwrap();
    assert!(edit.worth_doing.is_empty());
    input.task_annotations[0].work.minimum_chunk_minutes = std::num::NonZeroU32::new(5);
    let (_, edit) = temporal_app::today::evaluate(&input).unwrap();
    assert_eq!(edit.worth_doing.len(), 1);
    assert_eq!(
        edit.worth_doing[0].suggestion.valid_until.to_string(),
        "2026-09-08T04:00:00.000Z"
    );
}

#[test]
fn completion_removal_and_revision_changes_retire_advice_without_rewriting_targets() {
    let mut input = case("S01");
    let (_, edit) = temporal_app::today::evaluate(&input).unwrap();
    assert!(!edit.worth_doing.is_empty());
    input.prior_suggestions = edit
        .worth_doing
        .iter()
        .map(|r| r.suggestion.clone())
        .collect();
    input.task_refs[0].status = TaskStatus::Done;
    input.snapshot_revision = std::num::NonZeroU64::new(input.snapshot_revision.get() + 1).unwrap();
    let (output, edit) = temporal_app::today::evaluate(&input).unwrap();
    assert!(edit.worth_doing.is_empty());
    assert!(
        output
            .suggestion_validity
            .iter()
            .all(|r| r.status == SuggestionStatus::Invalidated)
    );
    input.task_refs[0].presence = Presence::Removed;
    let (_, edit) = temporal_app::today::evaluate(&input).unwrap();
    assert!(edit.worth_doing.is_empty());
}

#[test]
fn the_edit_ranks_risk_first_and_stays_bounded_without_discarding_todays_facts() {
    let (_, edit) = temporal_app::today::evaluate(&case("S04")).unwrap();
    // Six real cutoffs fall today. The edit is bounded; none of them are lost.
    assert_eq!(edit.fixed.len(), 6);
    assert!(edit.radar.is_empty() && edit.radar_omitted == 0);
    assert_eq!(edit.worth_doing.len(), 3);
    let risks: Vec<_> = edit.worth_doing.iter().map(|r| r.basis.risk).collect();
    assert_eq!(
        risks,
        vec![Some(Risk::Tight), Some(Risk::Tight), Some(Risk::Room)]
    );
    let cutoffs: Vec<_> = edit.worth_doing.iter().map(|r| r.basis.cutoff).collect();
    assert!(cutoffs.windows(2).all(|pair| pair[0] <= pair[1]));

    let (_, edit) = temporal_app::today::evaluate(&case("S13")).unwrap();
    assert_eq!(
        edit.worth_doing
            .iter()
            .map(|r| r.basis.risk)
            .collect::<Vec<_>>(),
        vec![Some(Risk::Insufficient), Some(Risk::Unknown)]
    );
}

#[test]
fn unknown_compatibility_is_excluded_and_last_known_sources_stay_qualified() {
    // Context and energy uncertainty never becomes a definite recommendation.
    let (_, edit) = temporal_app::today::evaluate(&case("S15")).unwrap();
    assert!(edit.worth_doing.is_empty() && edit.loose.is_empty());

    let fresh = case("S12");
    let (_, edit) = temporal_app::today::evaluate(&fresh).unwrap();
    assert_eq!(edit.worth_doing.len(), 1);
    assert_eq!(
        edit.worth_doing[0].basis.qualification,
        Qualification::DeclaredInputs
    );
    let mut stale = fresh;
    stale.evaluation.now = stale.evaluation.now.checked_add_ms(3_600_000).unwrap();
    let (_, edit) = temporal_app::today::evaluate(&stale).unwrap();
    assert_eq!(edit.worth_doing.len(), 1);
    assert_eq!(
        edit.worth_doing[0].basis.qualification,
        Qualification::Conditional
    );
    assert!(edit.worth_doing[0].suggestion.valid_until > stale.evaluation.now);
}

#[test]
fn awareness_does_not_repeat_a_cutoff_the_edit_already_put_in_front_of_the_user() {
    let (_, edit) = temporal_app::today::evaluate(&case("S02")).unwrap();
    let advised: Vec<_> = edit
        .worth_doing
        .iter()
        .filter_map(|r| r.basis.deadline)
        .collect();
    assert!(!advised.is_empty());
    assert!(
        edit.radar.iter().all(
            |r| !matches!(r, temporal_app::today::FactRef::Deadline(id) if advised.contains(id))
        )
    );
    // Awareness that is genuinely unshown still surfaces, with the omission counted.
    assert!(!edit.radar.is_empty() && edit.radar_omitted > 0);
}
