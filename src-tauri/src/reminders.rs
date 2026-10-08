//! Which reminders are due. Pure and deterministic: the caller supplies `now`
//! and remembers which reminders it already showed.
use std::collections::BTreeSet;
use temporal_core::{domain::*, time::*};

/// How late a reminder may still fire after its moment (missed ticks, sleep).
const GRACE_MS: u64 = 2 * 60_000;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reminder {
    /// Identity plus start, so a moved event reminds again.
    pub key: String,
    pub title: String,
    pub body: String,
}

/// Timed, present Anchors whose reminder moment (`start - lead`) has arrived
/// and whose start is not long past. Contains only title and time.
pub fn due(
    input: &EvaluationInput,
    now: Instant,
    lead_minutes: u32,
    shown: &BTreeSet<String>,
    time_label: impl Fn(Instant) -> String,
) -> Vec<Reminder> {
    let lead = u64::from(lead_minutes) * 60_000;
    let mut reminders = Vec::new();
    for anchor in input
        .anchors
        .iter()
        .filter(|a| a.presence == Presence::Present)
    {
        let TemporalSpan::Timed(span) = &anchor.span else {
            continue;
        };
        let start = span.start();
        let Ok(moment) = start.checked_sub_ms(lead) else {
            continue;
        };
        let Ok(latest) = start.checked_add_ms(GRACE_MS) else {
            continue;
        };
        if now < moment || now > latest {
            continue;
        }
        let key = format!("{}@{}", anchor.meta.id, start.epoch_ms());
        if shown.contains(&key) {
            continue;
        }
        let minutes = start
            .duration_since(now)
            .map_or(0, |ms| ms.div_ceil(60_000));
        let when = if minutes == 0 {
            format!("Starting now · {}", time_label(start))
        } else {
            format!("In {minutes} min · {}", time_label(start))
        };
        reminders.push(Reminder {
            key,
            title: anchor.title.clone(),
            body: when,
        });
    }
    reminders.sort_by(|a, b| a.key.cmp(&b.key));
    reminders
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU64;

    fn input_with(start: &str, end: &str) -> EvaluationInput {
        let at: Instant = "2026-09-09T12:00:00.000Z".parse().unwrap();
        let source: SourceId = "00000000-0000-4000-8000-000000000001".parse().unwrap();
        let mut input: EvaluationInput = serde_json::from_value(serde_json::json!({
            "schema_version": 1,
            "snapshot_revision": 1,
            "captured_now": at,
            "sources": [{"id": source, "kind": "local", "label": "Synthetic"}],
            "source_states": [], "required_sources": [], "task_refs": [], "anchors": [],
            "deadlines": [], "intentions": [], "routines": [], "anchor_annotations": [],
            "deadline_annotations": [], "task_annotations": [], "routine_outcomes": [],
            "availability": [], "prior_suggestions": [],
            "evaluation": {"now": at, "evaluation_end": "2026-09-23T12:00:00.000Z",
                "display_zone": "America/Toronto", "timezone_rules_version": "2025b",
                "policy_version": "proof-v1"}
        }))
        .unwrap();
        input.anchors.push(Anchor {
            meta: RecordMeta {
                id: "00000000-0000-4000-8000-0000000000a1".parse().unwrap(),
                revision: NonZeroU64::MIN,
                created_at: at,
                updated_at: at,
            },
            title: "Synthetic seminar".into(),
            presence: Presence::Present,
            provenance: Provenance::LocalUser(LocalAssertion {
                source_id: source,
                asserted_at: at,
            }),
            span: TemporalSpan::Timed(
                TimedSpan::new(start.parse().unwrap(), end.parse().unwrap()).unwrap(),
            ),
            rigidity: AnchorRigidity::Fixed,
            reported_certainty: ReportedCertainty::Confirmed,
            occupancy: Occupancy::Busy,
            location: None,
        });
        input
    }

    #[test]
    fn reminders_fire_once_inside_their_window() {
        let input = input_with("2026-09-09T13:00:00.000Z", "2026-09-09T14:00:00.000Z");
        let label = |_: Instant| "9:00 AM".to_string();
        let early: Instant = "2026-09-09T12:49:00.000Z".parse().unwrap();
        assert!(due(&input, early, 10, &BTreeSet::new(), label).is_empty());
        let on_time: Instant = "2026-09-09T12:50:00.000Z".parse().unwrap();
        let shown = due(&input, on_time, 10, &BTreeSet::new(), label);
        assert_eq!(shown.len(), 1);
        assert_eq!(shown[0].body, "In 10 min · 9:00 AM");
        let remembered: BTreeSet<String> = shown.iter().map(|r| r.key.clone()).collect();
        assert!(due(&input, on_time, 10, &remembered, label).is_empty());
        let late: Instant = "2026-09-09T13:03:00.000Z".parse().unwrap();
        assert!(due(&input, late, 10, &BTreeSet::new(), label).is_empty());
        let at_start: Instant = "2026-09-09T13:00:30.000Z".parse().unwrap();
        assert_eq!(
            due(&input, at_start, 0, &BTreeSet::new(), label)[0].body,
            "Starting now · 9:00 AM"
        );
    }
}
