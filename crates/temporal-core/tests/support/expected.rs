//! Explicit acceptance values transcribed from SCENARIOS.md, never captured
//! from the evaluator. An unknown named variant fails the inventory check.
use std::num::NonZeroU32;
use temporal_core::results::*;

pub struct Pressure {
    pub risk: Risk,
    pub phase: DeadlinePhase,
    pub resolution: Resolution,
    pub work: Option<Workload>,
    // (known total, known subtotal), in minutes; None total means unknown.
    pub opportunity: Option<(Option<u64>, u64)>,
    pub qualification: Qualification,
}
pub struct Expected {
    pub windows_ms: Vec<u64>,
    pub fits: String,
    pub anchors: String, // upcoming, ongoing, passed, inactive = U,O,P,I
    pub intentions: Vec<IntentionPhase>,
    pub occurrences: Vec<(&'static str, OccurrencePhase)>,
    pub conflicts: usize,
    pub pressures: Vec<Pressure>,
    pub suggestions: Vec<SuggestionStatus>,
    pub health: Vec<Health>, // canonical source ID order
}

fn p(work: u32, opportunity: u64, risk: Risk) -> Pressure {
    Pressure {
        risk,
        phase: DeadlinePhase::Upcoming,
        resolution: Resolution::Unresolved,
        work: Some(Workload::Estimate(work)),
        opportunity: Some((Some(opportunity), opportunity)),
        qualification: Qualification::DeclaredInputs,
    }
}
fn early(
    risk: Risk,
    phase: DeadlinePhase,
    resolution: Resolution,
    work: Option<Workload>,
) -> Pressure {
    Pressure {
        risk,
        phase,
        resolution,
        work,
        opportunity: None,
        qualification: Qualification::DeclaredInputs,
    }
}
fn unknown_work() -> Pressure {
    early(
        Risk::Unknown,
        DeadlinePhase::Upcoming,
        Resolution::Unresolved,
        Some(Workload::Unknown),
    )
}
fn zero_work() -> Pressure {
    early(
        Risk::NoKnownWork,
        DeadlinePhase::Upcoming,
        Resolution::Unresolved,
        Some(Workload::Estimate(0)),
    )
}
fn satisfied() -> Pressure {
    early(
        Risk::NotApplicable,
        DeadlinePhase::Inactive,
        Resolution::Satisfied,
        None,
    )
}
fn overdue() -> Pressure {
    early(
        Risk::Overdue,
        DeadlinePhase::Overdue,
        Resolution::Unresolved,
        None,
    )
}
fn lower() -> Pressure {
    let mut row = p(240, 180, Risk::Insufficient);
    row.work = Some(Workload::AtLeast(NonZeroU32::new(240).unwrap()));
    row
}
fn unknown_capacity() -> Pressure {
    let mut row = p(90, 0, Risk::Unknown);
    row.opportunity = Some((None, 0));
    row
}

pub fn expected(name: &str) -> Expected {
    use {
        DeadlinePhase as D, Health as H, IntentionPhase as I, OccurrencePhase as O, Risk as R,
        SuggestionStatus as S,
    };
    let scenario = &name[..3];
    let mut e = Expected {
        windows_ms: vec![],
        fits: String::new(),
        anchors: String::new(),
        intentions: vec![],
        occurrences: vec![],
        conflicts: 0,
        pressures: vec![],
        suggestions: vec![],
        health: vec![H::Healthy],
    };
    let (minutes, fits, anchors) = match scenario {
        "S01" => {
            e.health.push(H::Healthy);
            (vec![120, 120], "FF", "U")
        }
        "S02" => {
            e.health.push(H::Healthy);
            e.pressures = vec![p(480, 600, R::Tight)];
            (
                vec![60, 120, 60, 120, 60, 120, 60, 120, 60, 120],
                "XFXFXFXFXFFFFFFFFFFF",
                "UUUUUUUUUU",
            )
        }
        "S03" => {
            e.pressures = vec![p(480, 1200, R::Room)];
            (vec![300; 4], "FFFF", "")
        }
        "S04" => {
            e.pressures = vec![
                p(45, 60, R::Tight),
                p(45, 90, R::Tight),
                p(45, 120, R::Room),
                p(45, 150, R::Room),
                p(45, 180, R::Room),
                p(45, 180, R::Room),
            ];
            (vec![180], "FFFFFF", "")
        }
        "S05" => {
            e.health.push(H::Healthy);
            e.conflicts = 1;
            e.pressures = vec![p(90, 120, R::Tight)];
            (vec![60, 60], "FF", "UU")
        }
        "S06" => {
            e.health.push(H::Healthy);
            e.pressures = vec![p(240, 120, R::Insufficient)];
            (vec![60, 60], "FF", "UU")
        }
        "S07" => {
            e.intentions = vec![I::PreferencePassed, I::PreferredNow];
            (vec![], "", "")
        }
        "S08" => {
            e.health.push(H::Healthy);
            e.intentions = vec![I::NoPreference];
            e.pressures = vec![p(90, 120, R::Tight)];
            (vec![60, 60], "FFFF", "U")
        }
        "S09" => {
            e.pressures = vec![overdue()];
            (vec![120], "X", "")
        }
        "S10" => {
            e.health.push(H::Healthy);
            e.suggestions = vec![S::Current];
            (vec![120, 120], "FF", "")
        }
        "S11" => {
            let mut row = p(60, 0, R::Insufficient);
            row.phase = D::DueToday;
            e.pressures = vec![row];
            (vec![], "", "O")
        }
        "S12" => {
            e.health.push(H::Healthy);
            e.pressures = vec![p(90, 120, R::Tight)];
            (vec![60, 60], "FF", "U")
        }
        "S13" => {
            e.health = vec![H::Healthy; 3];
            e.pressures = vec![
                satisfied(),
                zero_work(),
                unknown_work(),
                lower(),
                zero_work(),
            ];
            (vec![180], "FF", "")
        }
        "S14" => {
            e.occurrences = vec![
                ("2026-09-06", O::Current),
                ("2026-09-13", O::Future),
                ("2026-09-20", O::Future),
            ];
            (vec![60, 60], "FXXFXX", "")
        }
        "S15" => {
            e.pressures = vec![unknown_capacity()];
            (vec![60, 60, 60], "X?X", "")
        }
        "S16" => {
            e.health.push(H::Healthy);
            e.pressures = vec![p(60, 120, R::Tight)];
            e.suggestions = vec![S::Current; 2];
            (vec![120], "FF", "")
        }
        _ => panic!("missing scenario expectation: {name}"),
    };
    e.windows_ms = minutes.into_iter().map(|m| m * 60_000).collect();
    e.fits = fits.into();
    e.anchors = anchors.into();
    // All S16 mutations change the basis except the explicit clock step.
    if scenario == "S16" && name != "S16/base" && name != "S16/half_hour_later" {
        e.suggestions = vec![S::Invalidated; 2];
    }
    match name {
        "S01/base" | "S02/base" | "S03/base" | "S04/base" | "S05/base" | "S06/base"
        | "S07/base" | "S08/base" | "S09/base" | "S10/base" | "S11/base" | "S12/base"
        | "S13/base" | "S14/base" | "S15/base" | "S16/base" => {}
        "S03/wednesday" => {
            e.windows_ms = vec![300 * 60_000; 2];
            e.fits = "FF".into();
            e.pressures = vec![p(480, 600, R::Tight)];
        }
        "S03/thursday" => {
            e.windows_ms = vec![300 * 60_000];
            e.fits = "F".into();
            e.pressures = vec![p(480, 300, R::Insufficient)];
        }
        "S05/transparent" => {
            e.conflicts = 0;
            e.windows_ms = vec![60 * 60_000, 90 * 60_000];
            e.pressures = vec![p(90, 150, R::Tight)];
        }
        "S06/chunk_90" => {
            e.fits = "XX".into();
            e.pressures = vec![p(240, 0, R::Insufficient)];
        }
        "S07/declared_evening" => {
            e.windows_ms = vec![60 * 60_000];
            e.fits = "F?".into();
        }
        "S08/source_move" => {
            e.windows_ms = vec![120 * 60_000];
            e.fits = "FF".into();
        }
        "S08/same_text_distinct_identity" => {
            e.anchors = "UU".into();
            e.conflicts = 1;
        }
        "S09/satisfied" => {
            e.pressures = vec![satisfied()];
            e.fits.clear();
        }
        "S09/zero_unresolved" => {
            e.fits.clear();
        }
        "S10/expiry_equality" | "S10/tuesday" => {
            e.windows_ms = vec![120 * 60_000];
            e.fits = "F".into();
            e.suggestions = vec![S::Expired];
        }
        "S10/trace_done" => {
            e.windows_ms = vec![120 * 60_000];
            e.fits.clear();
            e.suggestions = vec![S::Invalidated];
        }
        "S11/date_still_today" | "S11/utc_display" | "S11/spring" => {}
        "S11/date_overdue_equality" | "S11/exact_plus_ms" => {
            e.anchors = "P".into();
            e.pressures = vec![overdue()];
        }
        "S11/exact_base" => {
            e.pressures[0].phase = D::Upcoming;
        }
        "S11/exact_equality" | "S11/exact_no_availability" => {
            e.anchors = "P".into();
            e.pressures[0].phase = D::DueNow;
        }
        "S11/retired_anchor" => {
            e.anchors = "I".into();
            e.windows_ms = vec![1500 * 60_000];
            e.fits = "F".into();
            e.pressures = vec![p(60, 1500, R::Room)];
            e.pressures[0].phase = D::DueToday;
        }
        "S11/spring_transparent" => {
            e.windows_ms = vec![1380 * 60_000];
            e.fits = "F".into();
            e.pressures = vec![p(60, 1380, R::Room)];
            e.pressures[0].phase = D::DueToday;
        }
        "S12/failed" | "S12/partial" | "S12/incompatible" => {
            e.health[1] = match name {
                "S12/failed" => H::Unavailable,
                "S12/partial" => H::Partial,
                _ => H::Incompatible,
            };
            e.pressures[0].qualification = Qualification::Conditional;
        }
        "S12/freshness_equality" => {
            e.windows_ms = vec![30 * 60_000, 60 * 60_000];
            e.pressures = vec![p(90, 90, R::Tight)];
        }
        "S12/freshness_plus_ms" => {
            e.windows_ms = vec![30 * 60_000 - 1, 60 * 60_000];
            e.fits = "XF".into();
            e.health[1] = H::Stale;
            e.pressures = vec![p(90, 60, R::Insufficient)];
            e.pressures[0].qualification = Qualification::Conditional;
        }
        "S12/short_coverage" => {
            e.pressures[0].qualification = Qualification::Conditional;
        }
        "S12/never_loaded_empty" | "S12/successful_empty" => {
            e.windows_ms = vec![180 * 60_000];
            e.fits = "F".into();
            e.anchors.clear();
            e.pressures = vec![p(90, 180, R::Tight)];
            if name == "S12/never_loaded_empty" {
                e.health[1] = H::NeverLoaded;
                e.pressures[0].qualification = Qualification::Conditional;
            }
        }
        "S13/stale_trace" | "S13/stale_trace_implicit_dependency" => {
            e.health[1] = H::Stale;
            let count = if name == "S13/stale_trace" { 5 } else { 2 };
            for row in &mut e.pressures[..count] {
                row.qualification = Qualification::Conditional;
            }
        }
        "S13/unknown_own_due" => {
            e.health = vec![H::Healthy; 2];
            e.pressures = vec![early(R::Unknown, D::Unknown, Resolution::Unknown, None)];
            e.fits.clear();
        }
        "S13/unknown_independent_work" => {
            e.pressures = vec![unknown_work()];
            e.fits.clear();
        }
        "S13/authoritative_removal" => {
            e.pressures = vec![unknown_work(), unknown_work(), lower(), zero_work()];
        }
        "S13/quercus_satisfied" => {
            e.pressures[1] = satisfied();
        }
        "S13/local_confirmation_only" => {}
        "S14/monday" => {
            e.windows_ms = vec![60 * 60_000];
            e.fits = "FX".into();
            e.occurrences.remove(0);
        }
        "S14/skipped" => {
            e.occurrences[0].1 = O::Skipped;
            e.fits = "XFXX".into();
        }
        "S14/edited_rule" => {
            e.occurrences = vec![("2026-09-07", O::Future), ("2026-09-14", O::Future)];
            e.fits = "XXXX".into();
        }
        "S14/paused" => {
            e.occurrences.clear();
            e.fits.clear();
        }
        "S14/second_routine" => {
            e.occurrences.extend(e.occurrences.clone());
            e.fits = "FXXFXXFXXFXX".into();
        }
        "S15/known_context" => {
            e.fits = "XFX".into();
            e.pressures = vec![p(90, 60, R::Insufficient)];
        }
        "S15/mismatch_with_unknown_energy" => {}
        "S15/no_availability" => {
            e.windows_ms.clear();
            e.fits.clear();
        }
        "S15/earliest_at_cutoff" | "S15/earliest_after_cutoff" | "S15/exact_equality" => {
            e.windows_ms.clear();
            e.fits.clear();
            e.pressures = vec![p(90, 0, R::Insufficient)];
            if name == "S15/exact_equality" {
                e.pressures[0].phase = D::DueNow;
            }
        }
        "S15/overdue_plus_ms" => {
            e.windows_ms.clear();
            e.fits.clear();
            e.pressures = vec![overdue()];
        }
        "S16/half_hour_later" => {
            e.windows_ms = vec![90 * 60_000];
            e.pressures = vec![p(60, 90, R::Tight)];
            e.suggestions = vec![S::Ineligible, S::Current];
        }
        "S16/effort_59" | "S16/current_basis_room_notice" => {
            e.pressures = vec![p(59, 120, R::Room)];
            if name == "S16/current_basis_room_notice" {
                e.suggestions[1] = S::Ineligible;
            }
        }
        "S16/effort_60"
        | "S16/importance_only"
        | "S16/priority_only"
        | "S16/new_snapshot_basis"
        | "S16/new_settings_basis"
        | "S16/range_equality"
        | "S16/unresolved_due" => {}
        "S16/effort_120" => {
            e.pressures = vec![p(120, 120, R::Tight)];
        }
        "S16/effort_121" => {
            e.pressures = vec![p(121, 120, R::Insufficient)];
        }
        "S16/unknown_effort" => {
            e.pressures = vec![unknown_work()];
        }
        "S16/zero_effort" => {
            e.pressures = vec![zero_work()];
            e.fits = "F".into();
        }
        "S16/lower_bound" => {
            e.pressures[0].work = Some(Workload::AtLeast(NonZeroU32::new(60).unwrap()));
        }
        "S16/range_plus_ms" => {
            e.pressures = vec![early(R::Unknown, D::Upcoming, Resolution::Unresolved, None)];
        }
        _ => panic!("missing named variant expectation: {name}"),
    }
    e
}
