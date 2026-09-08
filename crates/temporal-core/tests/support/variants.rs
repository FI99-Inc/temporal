//! Named mutations from SCENARIOS.md. No expected engine outputs are calculated.
use super::{bases, builder::*};
use std::{collections::BTreeSet, num::NonZeroU32};
use temporal_core::{domain::*, reasons::*, results::*, time::*};

fn base(cases: &[Case], scenario: u32) -> &Case {
    cases
        .iter()
        .find(|case| case.name == format!("S{scenario:02}/base"))
        .unwrap()
}
fn clock(base: &Case, name: &str, now: Instant) -> Case {
    let mut result = base.clone();
    result.name = name.into();
    result.input.evaluation.now = now;
    result
}

// Bump only changed factual/user records. Explicit historical audit overrides
// (S13's stale-source variant) are retained. Advice is never rewritten here.
pub fn mutate(base: &Case, name: &str, change: impl FnOnce(&mut EvaluationInput)) -> Case {
    let mut result = base.clone();
    result.name = name.into();
    change(&mut result.input);
    result.input.snapshot_revision = nz(base.input.snapshot_revision.get() + 1);
    result.input.captured_now = result.input.evaluation.now;
    let now = serde_json::to_value(result.input.captured_now).unwrap();
    let before = serde_json::to_value(&base.input).unwrap();
    let mut after = serde_json::to_value(&result.input).unwrap();
    for collection in [
        "anchors",
        "deadlines",
        "task_refs",
        "intentions",
        "routines",
        "availability",
    ] {
        for record in after[collection].as_array_mut().unwrap() {
            let old = before[collection]
                .as_array()
                .unwrap()
                .iter()
                .find(|candidate| candidate["meta"]["id"] == record["meta"]["id"]);
            if old == Some(&*record) {
                continue;
            }
            record["meta"]["revision"] = serde_json::Value::from(
                old.map(|old| old["meta"]["revision"].as_u64().unwrap() + 1)
                    .unwrap_or(1),
            );
            if old.is_none() {
                record["meta"]["created_at"] = now.clone();
            }
            if old.is_none_or(|old| old["meta"]["updated_at"] == record["meta"]["updated_at"]) {
                record["meta"]["updated_at"] = now.clone();
            }
            for field in ["asserted_at", "observed_at"] {
                let pointer = format!("/provenance/value/{field}");
                if record.pointer(&pointer).is_some()
                    && old.is_none_or(|old| old.pointer(&pointer) == record.pointer(&pointer))
                {
                    *record.pointer_mut(&pointer).unwrap() = now.clone();
                }
            }
        }
    }
    for collection in [
        "anchor_annotations",
        "deadline_annotations",
        "task_annotations",
    ] {
        for record in after[collection].as_array_mut().unwrap() {
            let old = before[collection]
                .as_array()
                .unwrap()
                .iter()
                .find(|candidate| candidate["target_id"] == record["target_id"]);
            if old == Some(&*record) {
                continue;
            }
            record["revision"] = serde_json::Value::from(
                old.map(|old| old["revision"].as_u64().unwrap() + 1)
                    .unwrap_or(1),
            );
            if old.is_none() {
                record["created_at"] = now.clone();
            }
            if old.is_none_or(|old| old["updated_at"] == record["updated_at"]) {
                record["updated_at"] = now.clone();
            }
        }
    }
    result.input = serde_json::from_value(after).unwrap();
    result
}

fn deadline_work(input: &mut EvaluationInput, index: usize) -> &mut WorkMetadata {
    let DeadlineWork::Standalone(work) = &mut input.deadline_annotations[index].work else {
        panic!("fixture requires standalone work")
    };
    work
}
fn retain_sources(input: &mut EvaluationInput, kinds: &[SourceKind]) {
    let ids: BTreeSet<_> = kinds.iter().map(|kind| source_id(*kind)).collect();
    input.sources.retain(|source| ids.contains(&source.id));
    input
        .source_states
        .retain(|state| ids.contains(&state.source_id));
    input
        .required_sources
        .retain(|source| ids.contains(&source.source_id));
}

pub fn all(bases: &[Case]) -> Vec<Case> {
    let mut cases = Vec::new();
    let b = base(bases, 3);
    cases.push(clock(b, "S03/wednesday", at("2026-09-09", "09:00")));
    cases.push(clock(b, "S03/thursday", at("2026-09-10", "09:00")));
    cases.push(mutate(base(bases, 5), "S05/transparent", |input| {
        input.anchors[1].occupancy = Occupancy::Transparent
    }));
    cases.push(mutate(base(bases, 6), "S06/chunk_90", |input| {
        input.task_annotations[0].work.minimum_chunk_minutes = NonZeroU32::new(90)
    }));
    cases.push(mutate(base(bases, 7), "S07/declared_evening", |input| {
        input.availability.push(AvailabilityDeclaration {
            meta: RecordMeta {
                id: "00000000-0000-4000-8000-007000000003".parse().unwrap(),
                revision: nz(1),
                created_at: input.captured_now,
                updated_at: input.captured_now,
            },
            span: span("2026-09-07", "18:00", "19:00"),
            contexts: DeclaredContexts::Known(BTreeSet::from(["desk".parse().unwrap()])),
            energy_capacity: EnergyCapacity::Deep,
        });
    }));
    cases.push(mutate(base(bases, 8), "S08/source_move", |input| {
        input.anchors[0].span = TemporalSpan::Timed(span("2026-09-07", "15:00", "16:00"));
        let Provenance::Imported(provenance) = &mut input.anchors[0].provenance else {
            unreachable!()
        };
        provenance.source_revision = Some("synthetic-2".into());
    }));
    cases.push(mutate(
        base(bases, 8),
        "S08/same_text_distinct_identity",
        |input| {
            let mut second = input.anchors[0].clone();
            second.meta.id = "00000000-0000-4000-8000-008000000005".parse().unwrap();
            let Provenance::Imported(provenance) = &mut second.provenance else {
                unreachable!()
            };
            provenance.external_id = "synthetic-A2".into();
            input.anchors.push(second);
        },
    ));
    cases.push(mutate(base(bases, 9), "S09/satisfied", |input| {
        input.deadlines[0].fulfillment =
            Fulfillment::Recorded(RecordedResolution::Satisfied(ResolutionEvidence {
                authority: EvidenceAuthority::LocalUser,
                recorded_at: at("2026-09-08", "08:30"),
                effective_at: Some(at("2026-09-07", "16:30")),
            }))
    }));
    cases.push(mutate(base(bases, 9), "S09/zero_unresolved", |input| {
        *deadline_work(input, 0) = work(0, 0)
    }));
    let b = base(bases, 10);
    cases.push(clock(
        b,
        "S10/expiry_equality",
        "2026-09-08T04:00:00.000Z".parse().unwrap(),
    ));
    let tuesday = clock(b, "S10/tuesday", at("2026-09-08", "09:00"));
    cases.push(mutate(&tuesday, "S10/trace_done", |input| {
        input.task_refs[0].status = TaskStatus::Done
    }));
    cases.push(tuesday);
    let b = base(bases, 11);
    cases.push(mutate(b, "S11/retired_anchor", |input| {
        input.anchors[0].presence = Presence::Removed
    }));
    let late = clock(
        b,
        "S11/date_still_today",
        "2026-11-02T04:30:00.000Z".parse().unwrap(),
    );
    cases.push(mutate(&late, "S11/utc_display", |input| {
        input.evaluation.display_zone = "UTC".parse().unwrap()
    }));
    cases.push(late);
    let endpoint: Instant = "2026-11-02T05:00:00.000Z".parse().unwrap();
    cases.push(clock(b, "S11/date_overdue_equality", endpoint));
    let exact = mutate(b, "S11/exact_base", |input| {
        input.deadlines[0].cutoff = Cutoff::At {
            instant: endpoint,
            original_zone: Some("America/Toronto".parse().unwrap()),
        }
    });
    let equality = clock(&exact, "S11/exact_equality", endpoint);
    cases.push(mutate(&equality, "S11/exact_no_availability", |input| {
        input.availability.clear()
    }));
    cases.push(clock(
        &exact,
        "S11/exact_plus_ms",
        endpoint.checked_add_ms(1).unwrap(),
    ));
    cases.push(equality);
    cases.push(exact);
    let spring = bases::s11(true);
    cases.push(mutate(&spring, "S11/spring_transparent", |input| {
        input.anchors[0].occupancy = Occupancy::Transparent
    }));
    cases.push(spring);
    let b = base(bases, 12);
    for (name, outcome) in [
        ("S12/failed", AttemptOutcome::Failed),
        ("S12/partial", AttemptOutcome::Partial),
        ("S12/incompatible", AttemptOutcome::Incompatible),
    ] {
        cases.push(mutate(b, name, |input| {
            input.source_states[0].last_attempt_outcome = outcome;
            input.source_states[0].last_attempt_at = Some(input.evaluation.now);
        }));
    }
    cases.push(clock(
        b,
        "S12/freshness_equality",
        at("2026-09-07", "09:30"),
    ));
    cases.push(clock(
        b,
        "S12/freshness_plus_ms",
        at("2026-09-07", "09:30:00.001"),
    ));
    cases.push(mutate(b, "S12/short_coverage", |input| {
        let state = &mut input.source_states[0];
        state.anchor_coverage = Some(
            TimedSpan::new(
                state.anchor_coverage.as_ref().unwrap().start(),
                at("2026-09-07", "10:00"),
            )
            .unwrap(),
        );
    }));
    cases.push(mutate(b, "S12/never_loaded_empty", |input| {
        input.anchors.clear();
        let state = &mut input.source_states[0];
        state.last_attempt_outcome = AttemptOutcome::Never;
        state.last_attempt_at = None;
        state.last_success_at = None;
        state.anchor_coverage = None;
    }));
    cases.push(mutate(b, "S12/successful_empty", |input| {
        input.anchors.clear()
    }));
    let b = base(bases, 13);
    let stale = mutate(b, "S13/stale_trace", |input| {
        let old = at("2026-09-07", "08:00");
        let state = input
            .source_states
            .iter_mut()
            .find(|state| state.source_id == source_id(SourceKind::Trace))
            .unwrap();
        state.last_success_at = Some(old);
        state.last_attempt_at = Some(old);
        state.fresh_for_ms = nz(1_800_000);
        let task = &mut input.task_refs[0];
        task.meta.created_at = old;
        task.meta.updated_at = old;
        let TaskProvenance::Imported(provenance) = &mut task.provenance;
        provenance.observed_at = old;
        let deadline = &mut input.deadlines[0];
        deadline.meta.created_at = old;
        deadline.meta.updated_at = old;
        let Provenance::Imported(provenance) = &mut deadline.provenance else {
            unreachable!()
        };
        provenance.observed_at = old;
    });
    cases.push(mutate(
        &stale,
        "S13/stale_trace_implicit_dependency",
        |input| {
            input
                .required_sources
                .retain(|source| source.source_id != source_id(SourceKind::Trace))
        },
    ));
    cases.push(stale);
    cases.push(mutate(b, "S13/unknown_own_due", |input| {
        retain_sources(input, &[SourceKind::Local, SourceKind::Trace]);
        let id = input.deadlines[0].meta.id;
        input.deadlines.retain(|deadline| deadline.meta.id == id);
        input.deadline_annotations.clear();
        input.task_refs[0].status = TaskStatus::Unknown;
        input.task_refs[0].unknown_status_label = Some("SyntheticFutureStatus".into());
    }));
    cases.push(mutate(b, "S13/unknown_independent_work", |input| {
        let id = input.deadlines[1].meta.id;
        input.deadlines.retain(|deadline| deadline.meta.id == id);
        input
            .deadline_annotations
            .retain(|annotation| annotation.target_id == id);
        input.task_refs[0].due = TaskDue::None;
        input.task_refs[0].status = TaskStatus::Unknown;
        input.task_refs[0].unknown_status_label = Some("SyntheticFutureStatus".into());
    }));
    cases.push(mutate(b, "S13/authoritative_removal", |input| {
        input.task_refs[0].presence = Presence::Removed;
        input.deadlines[0].presence = Presence::Removed;
    }));
    cases.push(mutate(b, "S13/quercus_satisfied", |input| {
        input.deadlines[1].fulfillment =
            Fulfillment::Recorded(RecordedResolution::Satisfied(ResolutionEvidence {
                authority: EvidenceAuthority::Source,
                recorded_at: input.evaluation.now,
                effective_at: None,
            }))
    }));
    cases.push(mutate(b, "S13/local_confirmation_only", |input| {
        input.deadline_annotations[0].confirmation = Some(Confirmation {
            confirmed_at: input.evaluation.now,
            record_revision: input.deadlines[1].meta.revision,
        })
    }));
    let b = base(bases, 14);
    cases.push(clock(b, "S14/monday", at("2026-09-07", "09:00")));
    let skipped = mutate(b, "S14/skipped", |input| {
        input.routine_outcomes.push(RoutineOutcome {
            routine_id: input.routines[0].meta.id,
            date: "2026-09-06".parse().unwrap(),
            outcome: Outcome::Skipped,
            revision: nz(1),
            recorded_at: input.evaluation.now,
        })
    });
    cases.push(mutate(&skipped, "S14/edited_rule", |input| {
        let RoutineRule::Weekly(rule) = &mut input.routines[0].rule;
        rule.weekdays = BTreeSet::from([Weekday::Mon]);
    }));
    cases.push(skipped);
    cases.push(mutate(b, "S14/paused", |input| {
        input.routines[0].state = RoutineState::Paused
    }));
    cases.push(mutate(b, "S14/second_routine", |input| {
        let mut second = input.routines[0].clone();
        second.meta.id = "00000000-0000-4000-8000-014000000004".parse().unwrap();
        input.routines.push(second);
    }));
    let b = base(bases, 15);
    cases.push(mutate(b, "S15/known_context", |input| {
        input.availability[1].contexts =
            DeclaredContexts::Known(BTreeSet::from(["desk".parse().unwrap()]))
    }));
    cases.push(mutate(b, "S15/mismatch_with_unknown_energy", |input| {
        input.availability[0].energy_capacity = EnergyCapacity::Unknown
    }));
    let empty = mutate(b, "S15/no_availability", |input| input.availability.clear());
    for (name, time) in [
        ("S15/earliest_at_cutoff", "12:00"),
        ("S15/earliest_after_cutoff", "13:00"),
    ] {
        cases.push(mutate(&empty, name, |input| {
            let work = deadline_work(input, 0);
            work.earliest_start = Some(at("2026-09-07", time));
            work.minimum_chunk_minutes = None;
        }));
    }
    cases.push(empty);
    cases.push(clock(b, "S15/exact_equality", at("2026-09-07", "12:00")));
    cases.push(clock(
        b,
        "S15/overdue_plus_ms",
        at("2026-09-07", "12:00:00.001"),
    ));
    let b = base(bases, 16);
    cases.push(clock(b, "S16/half_hour_later", at("2026-09-07", "09:30")));
    for minutes in [59, 60, 120, 121] {
        cases.push(mutate(b, &format!("S16/effort_{minutes}"), |input| {
            deadline_work(input, 0).effort = Effort::Estimate(minutes)
        }));
    }
    cases.push(mutate(b, "S16/unknown_effort", |input| {
        deadline_work(input, 0).effort = Effort::Unknown
    }));
    cases.push(mutate(b, "S16/zero_effort", |input| {
        *deadline_work(input, 0) = work(0, 0)
    }));
    cases.push(mutate(b, "S16/lower_bound", |input| {
        deadline_work(input, 0).effort = Effort::AtLeast(NonZeroU32::new(60).unwrap())
    }));
    cases.push(mutate(b, "S16/importance_only", |input| {
        input.deadline_annotations[0].importance = Importance::High
    }));
    cases.push(mutate(b, "S16/priority_only", |input| {
        input.task_refs[0].source_priority = Some("synthetic-high".into())
    }));
    cases.push(mutate(b, "S16/new_snapshot_basis", |_| {}));
    cases.push(mutate(b, "S16/new_settings_basis", |input| {
        input.evaluation.display_zone = "UTC".parse().unwrap()
    }));
    let mut room = mutate(b, "S16/current_basis_room_notice", |input| {
        deadline_work(input, 0).effort = Effort::Estimate(59)
    });
    let key = room.input.evaluation.key(room.input.snapshot_revision);
    let deadline_id = room.input.deadlines[0].meta.id;
    let notice = &mut room.input.prior_suggestions[1];
    notice.key.evaluation_key = key.clone();
    notice.reasons = vec![Reason::PressureRatio {
        references: vec![Reference::Pressure(PressureKey {
            evaluation_key: key,
            deadline_id,
        })],
        payload: RatioPayload {
            work_ms: 3_540_000,
            opportunity_ms: 7_200_000,
            risk: Risk::Room,
        },
    }];
    cases.push(room);
    cases.push(mutate(b, "S16/range_equality", |input| {
        input.deadlines[0].cutoff = Cutoff::At {
            instant: input.evaluation.evaluation_end,
            original_zone: Some("America/Toronto".parse().unwrap()),
        }
    }));
    cases.push(mutate(b, "S16/range_plus_ms", |input| {
        input.deadlines[0].cutoff = Cutoff::At {
            instant: input.evaluation.evaluation_end.checked_add_ms(1).unwrap(),
            original_zone: Some("America/Toronto".parse().unwrap()),
        }
    }));
    cases.push(mutate(b, "S16/unresolved_due", |input| {
        input.task_refs[0].due = TaskDue::Unresolved {
            value: "Friday-ish".into(),
            reason: "meaning_not_established".into(),
        }
    }));
    cases
}
