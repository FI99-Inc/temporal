use temporal_core::{
    codec,
    domain::*,
    evaluate,
    reasons::{Reason, ReasonCode},
    results::*,
};
#[path = "support/expected.rs"]
mod expected;
mod support;

#[test]
fn every_documented_scenario_has_complete_semantic_acceptance_values() {
    let cases = support::all_cases();
    assert_eq!(cases.len(), 81);
    for case in cases {
        let expected = expected::expected(&case.name);
        let output = evaluate(&case.input).unwrap_or_else(|e| panic!("{}: {e:?}", case.name));
        assert_semantics(&case.name, &output, &expected);
        assert_inventory(&case.input, &output);
        if matches!(
            case.name.as_str(),
            "S03/base"
                | "S05/base"
                | "S09/base"
                | "S10/base"
                | "S10/tuesday"
                | "S12/base"
                | "S12/short_coverage"
                | "S13/base"
                | "S13/stale_trace_implicit_dependency"
        ) {
            // Visible with --nocapture for the gate's manual evidence review.
            // These summaries are observations, never generated expectations.
            println!(
                "{} {}",
                case.name,
                serde_json::json!({
                    "now": output.evaluation_key.now,
                    "anchors": output.anchor_states.iter().map(|a| a.phase).collect::<Vec<_>>(),
                    "conflicts": output.conflicts.len(),
                    "windows_minutes": output.windows.iter().map(|w| w.span.duration_ms() / 60_000).collect::<Vec<_>>(),
                    "fits": output.fits.iter().map(|f| f.status).collect::<Vec<_>>(),
                    "pressure": output.pressures.iter().map(|p| serde_json::json!({
                        "phase": p.phase, "resolution": p.resolution,
                        "risk": p.risk, "ratio": p.ratio, "qualification": p.qualification,
                        "sources": p.source_qualifications.iter().map(|q| (q.role, q.health, q.covered)).collect::<Vec<_>>(),
                        "reasons": p.reasons.iter().map(Reason::code).collect::<Vec<_>>()
                    })).collect::<Vec<_>>(),
                    "suggestions": output.suggestion_validity.iter().map(|s| s.status).collect::<Vec<_>>()
                })
            );
        }
    }
}

fn assert_semantics(name: &str, output: &EvaluationOutput, expected: &expected::Expected) {
    assert_eq!(
        output
            .windows
            .iter()
            .map(|w| w.span.duration_ms())
            .collect::<Vec<_>>(),
        expected.windows_ms,
        "{name} windows"
    );
    let fits: String = output
        .fits
        .iter()
        .map(|f| match f.status {
            FitStatus::Fits => 'F',
            FitStatus::DoesNotFit => 'X',
            FitStatus::Unknown => '?',
        })
        .collect();
    assert_eq!(fits, expected.fits, "{name} fit matrix");
    let anchors: String = output
        .anchor_states
        .iter()
        .map(|a| match a.phase {
            AnchorPhase::Upcoming => 'U',
            AnchorPhase::Ongoing => 'O',
            AnchorPhase::Passed => 'P',
            AnchorPhase::Inactive => 'I',
        })
        .collect();
    assert_eq!(anchors, expected.anchors, "{name} Anchor phases");
    assert_eq!(
        output
            .intention_states
            .iter()
            .map(|r| r.phase)
            .collect::<Vec<_>>(),
        expected.intentions,
        "{name} Intention phases"
    );
    assert_eq!(
        output
            .routine_occurrences
            .iter()
            .map(|r| (r.key.date.to_string(), r.phase))
            .collect::<Vec<_>>(),
        expected
            .occurrences
            .iter()
            .map(|(d, p)| (d.to_string(), *p))
            .collect::<Vec<_>>(),
        "{name} Routine occurrences"
    );
    assert_eq!(
        output.conflicts.len(),
        expected.conflicts,
        "{name} conflicts"
    );
    assert_eq!(
        output
            .source_health
            .iter()
            .map(|r| r.health)
            .collect::<Vec<_>>(),
        expected.health,
        "{name} source health"
    );
    assert_eq!(
        output
            .suggestion_validity
            .iter()
            .map(|r| r.status)
            .collect::<Vec<_>>(),
        expected.suggestions,
        "{name} advice"
    );
    assert_eq!(
        output.pressures.len(),
        expected.pressures.len(),
        "{name} pressure inventory"
    );
    for (actual, expected) in output.pressures.iter().zip(&expected.pressures) {
        assert_eq!(
            (
                actual.risk,
                actual.phase,
                actual.resolution,
                actual.workload,
                actual.qualification
            ),
            (
                expected.risk,
                expected.phase,
                expected.resolution,
                expected.work,
                expected.qualification
            ),
            "{name} pressure state"
        );
        let opportunity = actual.opportunity.as_ref().map(|o| match o {
            Opportunity::Known {
                milliseconds,
                known_qualifying_ms,
                ..
            } => (Some(*milliseconds), *known_qualifying_ms),
            Opportunity::Unknown {
                known_qualifying_ms,
                ..
            } => (None, *known_qualifying_ms),
        });
        assert_eq!(
            opportunity,
            expected
                .opportunity
                .map(|(total, subtotal)| (total.map(|m| m * 60_000), subtotal * 60_000)),
            "{name} opportunity"
        );
        if let Some((Some(minutes), _)) = expected.opportunity
            && minutes > 0
        {
            let effort = match expected.work.unwrap() {
                Workload::Estimate(m) => m,
                Workload::AtLeast(m) => m.get(),
                Workload::Unknown => panic!("invalid expectation"),
            };
            let ratio = actual.ratio.unwrap();
            assert_eq!(
                (ratio.work_ms, ratio.opportunity_ms.get()),
                (u64::from(effort) * 60_000, minutes * 60_000),
                "{name} unreduced ratio"
            );
        } else {
            assert!(actual.ratio.is_none(), "{name} uncomputed ratio");
        }
        assert_eq!(
            actual
                .limitations
                .contains(&ReasonCode::IndividualCapacityOnly),
            actual.opportunity.is_some(),
            "{name} limitations"
        );
        assert!(
            actual
                .reasons
                .iter()
                .any(|r| r.code() == ReasonCode::DeadlinePhase),
            "{name} phase evidence"
        );
    }
}

fn assert_inventory(input: &EvaluationInput, output: &EvaluationOutput) {
    assert_eq!(
        output.evaluation_key,
        input.evaluation.key(input.snapshot_revision)
    );
    assert_eq!(output.anchor_states.len(), input.anchors.len());
    assert_eq!(output.deadline_states.len(), input.deadlines.len());
    assert_eq!(output.intention_states.len(), input.intentions.len());
    assert_eq!(
        output.pressures.len(),
        input
            .deadlines
            .iter()
            .filter(|d| d.presence == Presence::Present)
            .count()
    );
    assert_eq!(
        output.suggestion_validity.len(),
        input.prior_suggestions.len()
    );
    for pressure in &output.pressures {
        let state = output
            .deadline_states
            .iter()
            .find(|s| s.id == pressure.deadline_id)
            .unwrap();
        assert_eq!(
            (state.resolution, state.phase, state.endpoint),
            (pressure.resolution, pressure.phase, pressure.endpoint)
        );
        assert_eq!(state.cutoff, pressure.cutoff);
        assert_eq!(pressure.evaluation_key, output.evaluation_key);
        if let Some(opportunity) = &pressure.opportunity {
            let keys = match opportunity {
                Opportunity::Known { window_keys, .. }
                | Opportunity::Unknown { window_keys, .. } => window_keys,
            };
            for key in keys {
                assert!(output.windows.iter().any(|w| &w.key == key));
            }
        }
    }
    for fit in &output.fits {
        let window = output
            .windows
            .iter()
            .find(|w| w.key == fit.window_key)
            .unwrap();
        if let Some(span) = &fit.span {
            assert!(span.start() >= window.span.start() && span.end() <= window.span.end());
        }
    }
    let value = serde_json::to_value(output).unwrap();
    check_evidence(&value, input, output);
    for name in [
        "aggregate",
        "schedule",
        "reservations",
        "completed",
        "overdue",
    ] {
        assert!(value.get(name).is_none());
    }
}

fn check_evidence(value: &serde_json::Value, input: &EvaluationInput, output: &EvaluationOutput) {
    match value {
        serde_json::Value::Object(fields) => {
            if let Some(reasons) = fields.get("reasons") {
                let reasons: Vec<Reason> = serde_json::from_value(reasons.clone()).unwrap();
                let mut seen = Vec::new();
                for reason in &reasons {
                    assert!(!reason.references().is_empty());
                    assert!(!seen.contains(&reason));
                    seen.push(reason);
                    assert!(reason.references().windows(2).all(|p| p[0] < p[1]));
                    for reference in reason.references() {
                        let resolves = match reference {
                            Reference::Anchor(id) => input.anchors.iter().any(|r| r.meta.id == *id),
                            Reference::Availability(id) => {
                                input.availability.iter().any(|r| r.meta.id == *id)
                            }
                            Reference::Deadline(id) => {
                                input.deadlines.iter().any(|r| r.meta.id == *id)
                            }
                            Reference::Task(id) => input.task_refs.iter().any(|r| r.meta.id == *id),
                            Reference::Intention(id) => {
                                input.intentions.iter().any(|r| r.meta.id == *id)
                            }
                            Reference::Routine(id) => {
                                input.routines.iter().any(|r| r.meta.id == *id)
                            }
                            Reference::RoutineOccurrence(key) => {
                                input.routines.iter().any(|r| r.meta.id == key.routine_id)
                            }
                            Reference::Source(id) => input.sources.iter().any(|r| r.id == *id),
                            Reference::Window(key) => output.windows.iter().any(|r| r.key == *key),
                            Reference::Pressure(key) => {
                                key.evaluation_key == output.evaluation_key
                                    && output
                                        .pressures
                                        .iter()
                                        .any(|r| r.deadline_id == key.deadline_id)
                            }
                        };
                        assert!(resolves, "unresolved reason input: {reference:?}");
                    }
                }
            }
            for child in fields.values() {
                check_evidence(child, input, output);
            }
        }
        serde_json::Value::Array(values) => {
            for child in values {
                check_evidence(child, input, output);
            }
        }
        _ => {}
    }
}

#[test]
fn complete_outputs_are_canonical_repeatable_and_do_not_mutate_inputs() {
    for case in support::all_cases() {
        let before = serde_json::to_vec(&case.input).unwrap();
        let output = evaluate(&case.input).unwrap();
        let bytes = codec::canonical_bytes(&output).unwrap();
        assert_eq!(
            bytes,
            codec::canonical_bytes(&evaluate(&case.input).unwrap()).unwrap(),
            "{} repeat",
            case.name
        );
        assert_eq!(
            output,
            serde_json::from_slice::<EvaluationOutput>(&bytes).unwrap(),
            "{} typed canonical order",
            case.name
        );
        let mut shuffled = case.input.clone();
        shuffled.sources.reverse();
        shuffled.source_states.reverse();
        shuffled.required_sources.reverse();
        shuffled.anchors.reverse();
        shuffled.deadlines.reverse();
        shuffled.task_refs.reverse();
        shuffled.intentions.reverse();
        shuffled.routines.reverse();
        shuffled.availability.reverse();
        shuffled.anchor_annotations.reverse();
        shuffled.deadline_annotations.reverse();
        shuffled.task_annotations.reverse();
        shuffled.routine_outcomes.reverse();
        shuffled.prior_suggestions.reverse();
        for suggestion in &mut shuffled.prior_suggestions {
            suggestion.reasons.reverse();
            for reason in &mut suggestion.reasons {
                reason.references_mut().reverse();
            }
        }
        assert_eq!(
            output,
            evaluate(&shuffled).unwrap(),
            "{} permutation",
            case.name
        );
        assert_eq!(
            before,
            serde_json::to_vec(&case.input).unwrap(),
            "{} mutation",
            case.name
        );
    }
}
