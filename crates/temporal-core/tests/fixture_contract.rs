mod support;

use std::collections::BTreeSet;
use temporal_core::codec::{canonical_bytes, decode_input};
use temporal_core::validation::validate;

#[test]
fn every_documented_base_and_named_input_variant_validates_and_round_trips() {
    let cases = support::all_cases();
    println!(
        "{} valid synthetic inputs (bases and named variants)",
        cases.len()
    );
    let mut names = BTreeSet::new();
    for case in cases {
        assert!(
            names.insert(case.name.clone()),
            "duplicate case {}",
            case.name
        );
        validate(&case.input).unwrap_or_else(|errors| panic!("{}: {errors:#?}", case.name));
        let canonical = canonical_bytes(&case.input).unwrap();
        let decoded = decode_input(std::str::from_utf8(&canonical).unwrap())
            .unwrap_or_else(|errors| panic!("{}: {errors:#?}", case.name));
        assert_eq!(
            canonical_bytes(&decoded).unwrap(),
            canonical,
            "{}",
            case.name
        );
    }
    for scenario in 1..=16 {
        assert!(names.contains(&format!("S{scenario:02}/base")));
    }
}

#[test]
fn canonical_inputs_ignore_collection_order_and_preserve_unicode_text() {
    for case in support::all_cases() {
        let expected = canonical_bytes(&case.input).unwrap();
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
            canonical_bytes(&shuffled).unwrap(),
            expected,
            "{}",
            case.name
        );
    }
    let mut case = support::case("S16/base");
    case.input.deadlines[0].title = "café / e\u{301} \"quoted\" \\ line\n\u{0001}".into();
    let bytes = canonical_bytes(&case.input).unwrap();
    let text = std::str::from_utf8(&bytes).unwrap();
    assert!(text.contains("café / e\u{301}"));
    assert!(text.contains("\\u0001"));
    assert!(!text.ends_with('\n'));
    assert!(!text.contains(":null"));
    assert_eq!(
        decode_input(text).unwrap().deadlines[0].title,
        case.input.deadlines[0].title
    );
}

#[test]
fn fixture_changes_preserve_snapshot_and_annotation_ownership_rules() {
    let before = support::case("S03/base").input;
    let later = support::case("S03/thursday").input;
    assert_eq!(before.captured_now, later.captured_now);
    assert_eq!(before.snapshot_revision, later.snapshot_revision);
    assert_eq!(before.deadline_annotations, later.deadline_annotations);
    let before = support::case("S08/base").input;
    let moved = support::case("S08/source_move").input;
    assert_eq!(before.anchors[0].meta.id, moved.anchors[0].meta.id);
    assert_eq!(moved.anchors[0].meta.revision.get(), 2);
    assert_eq!(before.anchor_annotations, moved.anchor_annotations);
    let before = support::case("S16/base").input;
    let effort = support::case("S16/effort_59").input;
    assert_eq!(before.deadlines, effort.deadlines);
    assert_eq!(effort.deadline_annotations[0].revision.get(), 2);
    assert_eq!(before.prior_suggestions, effort.prior_suggestions);
}

#[test]
fn structured_reference_keys_sort_by_identity_and_numeric_basis_before_dates() {
    use temporal_core::results::{OccurrenceKey, Reference, WindowKey};
    let input = support::case("S14/second_routine").input;
    let first = Reference::RoutineOccurrence(OccurrenceKey {
        routine_id: input.routines[0].meta.id,
        date: "2026-09-13".parse().unwrap(),
    });
    let second = Reference::RoutineOccurrence(OccurrenceKey {
        routine_id: input.routines[1].meta.id,
        date: "2026-09-06".parse().unwrap(),
    });
    let mut key = input.evaluation.key(input.snapshot_revision);
    key.snapshot_revision = std::num::NonZeroU64::new(10).unwrap();
    let mut late = WindowKey {
        evaluation_key: key,
        availability_id: input.availability[0].meta.id,
        start: input.availability[0].span.start(),
        end: input.availability[0].span.end(),
    };
    let revision_ten = Reference::Window(late.clone());
    late.evaluation_key.snapshot_revision = std::num::NonZeroU64::new(2).unwrap();
    let revision_two = Reference::Window(late);
    let value = serde_json::json!({"references": [revision_ten, second, revision_two, first]});
    let canonical: serde_json::Value =
        serde_json::from_slice(&canonical_bytes(&value).unwrap()).unwrap();
    assert_eq!(
        canonical["references"],
        serde_json::json!([first, second, revision_two, revision_ten])
    );
}
