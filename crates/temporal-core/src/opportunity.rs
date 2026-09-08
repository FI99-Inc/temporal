//! Deterministic primitive opportunity derivation.
//!
//! A Window is the positive remainder of one explicit availability
//! declaration after present blocking Anchors are union-subtracted. This
//! module never guesses availability from an empty calendar and never
//! allocates a Window between competing work items.

use crate::{
    domain::{AvailabilityDeclaration, EvaluationInput, Occupancy, Presence},
    reasons::{
        AnchorBlockedPayload, AvailabilityPayload, ConservativeAnchorPayload, IntersectionPayload,
        Reason,
    },
    results::{AnchorConflict, OpportunityOutput, Reference, Window, WindowKey},
    source_health::{self, qualify},
    time::{TimeError, TimedSpan, TimezoneRules},
};

#[derive(Clone)]
struct BlockingAnchor {
    id: crate::domain::AnchorId,
    span: TimedSpan,
    occupancy: Occupancy,
    certainty: crate::domain::ReportedCertainty,
}

/// Derive Windows and independent positive Anchor conflicts for one input.
pub fn derive(input: &EvaluationInput) -> Result<OpportunityOutput, TimeError> {
    let rules = TimezoneRules::bundled();
    let evaluation_span = TimedSpan::new(input.evaluation.now, input.evaluation.evaluation_end)?;
    let blockers = blocking_anchors(input, &rules)?;
    let mut windows = Vec::new();

    let mut declarations: Vec<&AvailabilityDeclaration> = input.availability.iter().collect();
    declarations.sort_by_key(|declaration| declaration.meta.id);
    for declaration in declarations {
        let Some(clipped) = declaration.span.intersection(&evaluation_span) else {
            continue;
        };
        let declaration_blockers: Vec<(BlockingAnchor, TimedSpan)> = blockers
            .iter()
            .filter_map(|anchor| {
                anchor
                    .span
                    .intersection(&clipped)
                    .map(|intersection| (anchor.clone(), intersection))
            })
            .collect();
        let union = union_blockers(&declaration_blockers);
        let blocking_refs = declaration_blockers
            .iter()
            .map(|(anchor, _)| anchor.id)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let qualifications = source_health::dependencies_for_interval(input, &clipped)
            .into_iter()
            .map(|need| qualify(input, &need))
            .collect::<Result<Vec<_>, _>>()?;

        let mut remainders = Vec::new();
        let mut cursor = clipped.start();
        for blocked in &union {
            if cursor < blocked.start() {
                remainders.push(TimedSpan::new(cursor, blocked.start())?);
            }
            if cursor < blocked.end() {
                cursor = blocked.end();
            }
        }
        if cursor < clipped.end() {
            remainders.push(TimedSpan::new(cursor, clipped.end())?);
        }

        for remainder in remainders {
            let key = WindowKey {
                evaluation_key: input.evaluation.key(input.snapshot_revision),
                availability_id: declaration.meta.id,
                start: remainder.start(),
                end: remainder.end(),
            };
            let mut reasons = vec![Reason::DeclaredAvailability {
                references: vec![
                    Reference::Availability(declaration.meta.id),
                    Reference::Window(key.clone()),
                ],
                payload: AvailabilityPayload {
                    span: declaration.span.clone(),
                    clipped_span: Some(clipped.clone()),
                },
            }];
            for (anchor, intersection) in &declaration_blockers {
                reasons.push(Reason::AnchorBlocked {
                    references: vec![Reference::Anchor(anchor.id), Reference::Window(key.clone())],
                    payload: AnchorBlockedPayload {
                        anchor_span: anchor.span.clone(),
                        intersection: intersection.clone(),
                        blocked_ms: intersection.duration_ms(),
                    },
                });
                if anchor.occupancy == Occupancy::Unknown
                    || anchor.certainty == crate::domain::ReportedCertainty::Tentative
                {
                    reasons.push(Reason::ConservativeAnchor {
                        references: vec![
                            Reference::Anchor(anchor.id),
                            Reference::Window(key.clone()),
                        ],
                        payload: ConservativeAnchorPayload {
                            reported_certainty: anchor.certainty,
                            occupancy: anchor.occupancy,
                        },
                    });
                }
            }
            reasons.sort_by_key(|reason| reason.code().as_str());
            windows.push(Window {
                key,
                span: remainder,
                declaration_ref: declaration.meta.id,
                blocking_anchor_refs: blocking_refs.clone(),
                contexts: declaration.contexts.clone(),
                energy_capacity: declaration.energy_capacity,
                source_qualifications: qualifications.clone(),
                reasons,
            });
        }
    }
    windows.sort_by_key(|window| window.key.clone());
    let conflicts = conflicts(&blockers, &evaluation_span);
    Ok(OpportunityOutput { windows, conflicts })
}

/// Alias with an explicit name for callers that only need derived state.
pub fn derive_windows(input: &EvaluationInput) -> Result<OpportunityOutput, TimeError> {
    derive(input)
}

/// Evidence for the complete declared envelope, including declarations fully
/// consumed by Anchors (which produce no Window to carry their explanation).
pub(crate) fn reasons_for_interval(
    input: &EvaluationInput,
    interval: &TimedSpan,
) -> Result<Vec<Reason>, TimeError> {
    let blockers = blocking_anchors(input, &TimezoneRules::bundled())?;
    let mut reasons = Vec::new();
    for declaration in &input.availability {
        let clipped = declaration.span.intersection(interval);
        reasons.push(Reason::DeclaredAvailability {
            references: vec![Reference::Availability(declaration.meta.id)],
            payload: AvailabilityPayload {
                span: declaration.span.clone(),
                clipped_span: clipped.clone(),
            },
        });
        let Some(clipped) = clipped else { continue };
        for anchor in &blockers {
            let Some(intersection) = anchor.span.intersection(&clipped) else {
                continue;
            };
            let references = vec![
                Reference::Anchor(anchor.id),
                Reference::Availability(declaration.meta.id),
            ];
            reasons.push(Reason::AnchorBlocked {
                references: references.clone(),
                payload: AnchorBlockedPayload {
                    anchor_span: anchor.span.clone(),
                    blocked_ms: intersection.duration_ms(),
                    intersection,
                },
            });
            if anchor.occupancy == Occupancy::Unknown
                || anchor.certainty == crate::domain::ReportedCertainty::Tentative
            {
                reasons.push(Reason::ConservativeAnchor {
                    references,
                    payload: ConservativeAnchorPayload {
                        reported_certainty: anchor.certainty,
                        occupancy: anchor.occupancy,
                    },
                });
            }
        }
    }
    crate::reasons::normalize(&mut reasons);
    Ok(reasons)
}

fn blocking_anchors(
    input: &EvaluationInput,
    rules: &TimezoneRules,
) -> Result<Vec<BlockingAnchor>, TimeError> {
    let mut anchors = Vec::new();
    for anchor in input.anchors.iter().filter(|anchor| {
        anchor.presence == Presence::Present
            && matches!(anchor.occupancy, Occupancy::Busy | Occupancy::Unknown)
    }) {
        anchors.push(BlockingAnchor {
            id: anchor.meta.id,
            span: anchor.span.resolve(rules)?,
            occupancy: anchor.occupancy,
            certainty: anchor.reported_certainty,
        });
    }
    anchors.sort_by_key(|anchor| anchor.id);
    Ok(anchors)
}

fn union_blockers(blockers: &[(BlockingAnchor, TimedSpan)]) -> Vec<TimedSpan> {
    let mut spans: Vec<TimedSpan> = blockers.iter().map(|(_, span)| span.clone()).collect();
    spans.sort_by_key(|span| (span.start(), span.end()));
    let mut union: Vec<TimedSpan> = Vec::new();
    for span in spans {
        if let Some(last) = union.last_mut()
            && last.end() >= span.start()
        {
            if span.end() > last.end() {
                *last = TimedSpan::new(last.start(), span.end())
                    .expect("merged blocker spans remain positive");
            }
            continue;
        }
        union.push(span);
    }
    union
}

fn conflicts(blockers: &[BlockingAnchor], evaluation_span: &TimedSpan) -> Vec<AnchorConflict> {
    let mut output = Vec::new();
    for (index, first) in blockers.iter().enumerate() {
        for second in blockers.iter().skip(index + 1) {
            let Some(intersection) = first
                .span
                .intersection(&second.span)
                .and_then(|span| span.intersection(evaluation_span))
            else {
                continue;
            };
            let [first_id, second_id] = if first.id <= second.id {
                [first.id, second.id]
            } else {
                [second.id, first.id]
            };
            let mut reasons = vec![Reason::AnchorConflict {
                references: vec![Reference::Anchor(first_id), Reference::Anchor(second_id)],
                payload: IntersectionPayload {
                    intersection: intersection.clone(),
                },
            }];
            reasons.sort_by_key(|reason| reason.code().as_str());
            output.push(AnchorConflict {
                anchor_ids: [first_id, second_id],
                intersection,
                reasons,
            });
        }
    }
    output.sort_by_key(|conflict| (conflict.anchor_ids, conflict.intersection.start()));
    output
}
