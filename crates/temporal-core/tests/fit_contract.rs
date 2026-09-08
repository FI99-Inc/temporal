use temporal_core::{
    fit, opportunity,
    results::{FitStatus, Opportunity, WorkTarget},
};

mod support;

fn rows_for(
    rows: &[temporal_core::results::FitResult],
    target: WorkTarget,
) -> Vec<&temporal_core::results::FitResult> {
    rows.iter().filter(|row| row.target == target).collect()
}

#[test]
fn complete_matrix_keeps_short_fragments_and_single_work_owners() {
    let case = support::case("S02/base");
    let opportunity = opportunity::derive(&case.input).unwrap();
    let rows = fit::derive(&case.input, &opportunity.windows).unwrap();
    let deadline = WorkTarget::Deadline(case.input.deadlines[0].meta.id);
    let task = WorkTarget::Task(case.input.task_refs[0].meta.id);
    assert_eq!(rows_for(&rows, deadline).len(), 10);
    assert_eq!(
        rows_for(&rows, deadline)
            .iter()
            .filter(|row| row.status == FitStatus::Fits)
            .count(),
        5
    );
    assert_eq!(
        rows_for(&rows, task)
            .iter()
            .filter(|row| row.status == FitStatus::Fits)
            .count(),
        10
    );

    let opportunities = fit::opportunities(&case.input, &opportunity.windows).unwrap();
    assert!(matches!(
        opportunities.get(&deadline),
        Some(Opportunity::Known {
            milliseconds,
            known_qualifying_ms,
            ..
        }) if *milliseconds == 5 * 120 * 60_000 && *known_qualifying_ms == 5 * 120 * 60_000
    ));
}

#[test]
fn context_energy_chunk_and_unknown_inputs_have_distinct_statuses() {
    let case = support::case("S15/base");
    let output = opportunity::derive(&case.input).unwrap();
    let rows = fit::derive(&case.input, &output.windows).unwrap();
    let target = WorkTarget::Deadline(case.input.deadlines[0].meta.id);
    let rows = rows_for(&rows, target);
    assert_eq!(rows.len(), 3);
    assert_eq!(rows[0].status, FitStatus::DoesNotFit);
    assert_eq!(rows[1].status, FitStatus::Unknown);
    assert_eq!(rows[2].status, FitStatus::DoesNotFit);
    assert!(
        rows[0]
            .reasons
            .iter()
            .any(|reason| reason.code().as_str() == "context_mismatch")
    );
    assert!(
        rows[1]
            .reasons
            .iter()
            .any(|reason| reason.code().as_str() == "context_unknown")
    );
    assert!(
        rows[2]
            .reasons
            .iter()
            .any(|reason| reason.code().as_str() == "energy_mismatch")
    );

    let chunk = support::case("S06/chunk_90");
    let output = opportunity::derive(&chunk.input).unwrap();
    let rows = fit::derive(&chunk.input, &output.windows).unwrap();
    assert_eq!(rows.len(), 2);
    assert!(rows.iter().all(|row| row.status == FitStatus::DoesNotFit));
    assert!(rows.iter().all(|row| {
        row.reasons
            .iter()
            .any(|reason| reason.code().as_str() == "chunk_too_short")
    }));
    let opportunities = fit::opportunities(&chunk.input, &output.windows).unwrap();
    let target = WorkTarget::Task(chunk.input.task_refs[0].meta.id);
    assert!(matches!(
        opportunities.get(&target),
        Some(Opportunity::Known {
            milliseconds: 0,
            known_qualifying_ms: 0,
            window_keys,
        }) if window_keys.is_empty()
    ));
}

#[test]
fn soft_intentions_and_routine_dates_do_not_become_hard_deadlines() {
    let intention = support::case("S07/declared_evening");
    let output = opportunity::derive(&intention.input).unwrap();
    let rows = fit::derive(&intention.input, &output.windows).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].status, FitStatus::Fits);
    assert_eq!(rows[1].status, FitStatus::Unknown);

    let routine = support::case("S14/base");
    let output = opportunity::derive(&routine.input).unwrap();
    let rows = fit::derive(&routine.input, &output.windows).unwrap();
    assert_eq!(rows.len(), 6);
    assert_eq!(
        rows.iter()
            .filter(|row| row.status == FitStatus::Fits)
            .count(),
        2
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row.status == FitStatus::DoesNotFit && row.span.is_none())
            .count(),
        4
    );
}

#[test]
fn overdue_targets_are_explicitly_does_not_fit_and_math_precedes_unknowns() {
    let overdue = support::case("S09/base");
    let output = opportunity::derive(&overdue.input).unwrap();
    let rows = fit::derive(&overdue.input, &output.windows).unwrap();
    let target = WorkTarget::Deadline(overdue.input.deadlines[0].meta.id);
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].target, target);
    assert_eq!(rows[0].status, FitStatus::DoesNotFit);
    assert!(rows[0].span.is_none());
    assert!(
        rows[0]
            .reasons
            .iter()
            .any(|reason| reason.code().as_str() == "zero_opportunity")
    );

    let exact = support::case("S15/earliest_at_cutoff");
    let output = opportunity::derive(&exact.input).unwrap();
    assert!(output.windows.is_empty());
    assert!(
        fit::derive(&exact.input, &output.windows)
            .unwrap()
            .is_empty()
    );
}

#[test]
fn endpoints_and_clock_steps_clip_the_same_declared_windows() {
    let base = support::case("S03/base");
    let output = opportunity::derive(&base.input).unwrap();
    let rows = fit::derive(&base.input, &output.windows).unwrap();
    assert_eq!(output.windows.len(), 4);
    assert_eq!(rows.len(), 4);
    assert!(rows.iter().all(|row| row.status == FitStatus::Fits));

    let wednesday = support::case("S03/wednesday");
    let output = opportunity::derive(&wednesday.input).unwrap();
    assert_eq!(output.windows.len(), 2);
    assert!(
        fit::derive(&wednesday.input, &output.windows)
            .unwrap()
            .iter()
            .all(|row| row.status == FitStatus::Fits)
    );

    let many = support::case("S04/base");
    let output = opportunity::derive(&many.input).unwrap();
    let rows = fit::derive(&many.input, &output.windows).unwrap();
    assert_eq!(output.windows.len(), 1);
    assert_eq!(rows.len(), 6);
    assert!(rows.iter().all(|row| row.status == FitStatus::Fits));
    assert!(rows.windows(2).all(|pair| pair[0].target < pair[1].target));
}

#[test]
fn one_millisecond_of_clock_advance_can_remove_a_chunk_without_erasing_windows() {
    let case = support::case("S12/freshness_plus_ms");
    let output = opportunity::derive(&case.input).unwrap();
    assert_eq!(output.windows.len(), 2);
    let rows = fit::derive(&case.input, &output.windows).unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].status, FitStatus::DoesNotFit);
    assert_eq!(rows[1].status, FitStatus::Fits);
    assert!(
        rows[0]
            .reasons
            .iter()
            .any(|reason| reason.code().as_str() == "chunk_too_short")
    );
}
