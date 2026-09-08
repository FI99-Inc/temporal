use temporal_core::codec::decode_input;
use temporal_core::validation::IssueCode;

mod support;

#[test]
fn duplicate_json_keys_are_rejected_before_values_can_be_overwritten() {
    let errors = decode_input(r#"{"schema_version":1,"schema_version":2}"#).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.code == IssueCode::InvalidValue)
    );
}

#[test]
fn unsupported_versions_are_reported_explicitly() {
    let errors = decode_input(r#"{"schema_version":2}"#).unwrap_err();
    assert_eq!(errors[0].code, IssueCode::UnsupportedVersion);
    assert_eq!(errors[0].path, "schema_version");
}

#[test]
fn all_rejection_fixtures_fail_whole_input_with_the_documented_category() {
    let cases = support::negative::all();
    println!("{} independent rejection fixtures", cases.len());
    for case in cases {
        let errors = decode_input(&case.json).unwrap_err();
        assert!(
            errors.iter().any(|issue| issue.code == case.expected),
            "{}: expected {:?}, got {errors:#?}",
            case.name,
            case.expected
        );
        let mut sorted = errors.clone();
        sorted.sort();
        assert_eq!(errors, sorted, "{}", case.name);
    }
}

#[test]
fn authoritative_fulfillment_distinguishes_valid_done_links_from_competing_work() {
    use temporal_core::{fulfillment, results::Resolution};
    let input = support::case("S13/base").input;
    assert_eq!(
        fulfillment::resolve(&input.deadlines[0], &input.task_refs).resolution,
        Resolution::Satisfied
    );
    assert_eq!(
        fulfillment::resolve(&input.deadlines[1], &input.task_refs).resolution,
        Resolution::Unresolved
    );
    assert!(
        fulfillment::resolve(&input.deadlines[0], &input.task_refs)
            .evidence
            .is_none()
    );
    let own = support::case("S13/unknown_own_due").input;
    assert_eq!(
        fulfillment::resolve(&own.deadlines[0], &own.task_refs).resolution,
        Resolution::Unknown
    );
    for name in [
        "S13/base",
        "S13/unknown_own_due",
        "S13/unknown_independent_work",
    ] {
        temporal_core::validation::validate(&support::case(name).input).unwrap();
    }
}

#[test]
fn historical_advice_survives_changed_completion_resolution_and_removal() {
    use temporal_core::domain::{
        EvidenceAuthority, Fulfillment, Presence, RecordedResolution, ResolutionEvidence,
    };
    let base = support::case("S16/base");
    let removed = support::variants::mutate(&base, "retained_removed_target", |input| {
        input.deadlines[0].presence = Presence::Removed
    });
    let satisfied = support::variants::mutate(&base, "retained_satisfied_target", |input| {
        input.deadlines[0].fulfillment =
            Fulfillment::Recorded(RecordedResolution::Satisfied(ResolutionEvidence {
                authority: EvidenceAuthority::LocalUser,
                recorded_at: input.evaluation.now,
                effective_at: None,
            }))
    });
    for case in [removed, satisfied, support::case("S10/trace_done")] {
        let json =
            String::from_utf8(temporal_core::codec::canonical_bytes(&case.input).unwrap()).unwrap();
        decode_input(&json).unwrap_or_else(|errors| panic!("{}: {errors:#?}", case.name));
    }
}
