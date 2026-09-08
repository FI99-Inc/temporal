use temporal_core::{opportunity, results::Health, time::TimedSpan};

mod support;

fn duration_ms(spans: impl IntoIterator<Item = TimedSpan>) -> u64 {
    spans.into_iter().map(|span| span.duration_ms()).sum()
}

#[test]
fn declared_envelope_and_union_subtraction_are_conservative() {
    let case = support::case("S02/base");
    let output = opportunity::derive(&case.input).unwrap();
    assert_eq!(output.windows.len(), 10);
    assert_eq!(
        duration_ms(output.windows.iter().map(|window| window.span.clone())),
        900 * 60_000
    );
    assert!(
        output
            .windows
            .windows(2)
            .all(|pair| pair[0].key < pair[1].key)
    );

    let union = support::case("S05/base");
    let output = opportunity::derive(&union.input).unwrap();
    assert_eq!(output.windows.len(), 2);
    assert_eq!(output.conflicts.len(), 1);
    assert_eq!(output.conflicts[0].intersection.duration_ms(), 30 * 60_000);
    assert!(
        output
            .windows
            .iter()
            .all(|window| window.blocking_anchor_refs.len() == 2)
    );

    let transparent = support::case("S05/transparent");
    let output = opportunity::derive(&transparent.input).unwrap();
    assert_eq!(output.windows.len(), 2);
    assert!(output.conflicts.is_empty());
    assert!(
        output
            .windows
            .iter()
            .all(|window| window.blocking_anchor_refs.len() == 1)
    );
}

#[test]
fn civil_day_blocking_and_removed_anchors_preserve_time_rules() {
    let blocked = support::case("S11/base");
    assert!(
        opportunity::derive(&blocked.input)
            .unwrap()
            .windows
            .is_empty()
    );

    let retired = support::case("S11/retired_anchor");
    let output = opportunity::derive(&retired.input).unwrap();
    assert_eq!(output.windows.len(), 1);
    assert_eq!(output.windows[0].span.duration_ms(), 25 * 60 * 60_000);
}

#[test]
fn input_order_does_not_change_windows_or_conflicts() {
    let case = support::case("S05/base");
    let expected = opportunity::derive(&case.input).unwrap();
    let mut permuted = case.input.clone();
    permuted.anchors.reverse();
    permuted.availability.reverse();
    let actual = opportunity::derive(&permuted).unwrap();
    assert_eq!(actual, expected);
}

#[test]
fn source_qualification_retains_stale_health_on_last_known_blockers() {
    let case = support::case("S12/freshness_plus_ms");
    let output = opportunity::derive(&case.input).unwrap();
    assert_eq!(output.windows.len(), 2);
    assert!(
        output
            .windows
            .iter()
            .flat_map(|window| &window.source_qualifications)
            .any(|qualification| qualification.health == Health::Stale)
    );
}
