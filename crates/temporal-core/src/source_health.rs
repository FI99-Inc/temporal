//! Deterministic source freshness and factual dependency qualification.

use crate::{
    domain::{
        Deadline, EvaluationInput, Fulfillment, Occupancy, Presence, Projection, SourceKind,
        SourceRole, SourceState,
    },
    fulfillment,
    reasons::{Reason, SourceCoveragePayload, SourceHealthPayload},
    results::{Health, Reference, SourceHealthRow, SourceQualification},
    time::{Instant, TimeError, TimedSpan},
};
use std::collections::BTreeMap;

/// The coverage evidence required by one source/role dependency.
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(
    tag = "kind",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum CoverageNeed {
    AnchorInterval { interval: TimedSpan },
    DeadlineEndpoint { endpoint: Instant },
    TasksCatalog,
    NoAdditionalCoverage,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyNeed {
    pub source_id: crate::domain::SourceId,
    pub role: SourceRole,
    pub coverage: CoverageNeed,
}

/// Derive one health row per source in the catalog, in canonical source-ID
/// order. Local input is healthy without fabricated refresh timestamps.
pub fn source_health(input: &EvaluationInput) -> Result<Vec<SourceHealthRow>, TimeError> {
    let mut rows = Vec::with_capacity(input.sources.len());
    for source in &input.sources {
        let (health, reasons) = match source.kind {
            SourceKind::Local => (Health::Healthy, Vec::new()),
            _ => {
                let state = input
                    .source_states
                    .iter()
                    .find(|state| state.source_id == source.id);
                let state = state.cloned().unwrap_or_else(|| SourceState {
                    source_id: source.id,
                    last_attempt_outcome: crate::domain::AttemptOutcome::Never,
                    fresh_for_ms: std::num::NonZeroU64::new(1).unwrap(),
                    last_attempt_at: None,
                    last_success_at: None,
                    anchor_coverage: None,
                    deadline_coverage: None,
                    tasks_complete: false,
                });
                let health = derive_health(&state, input.evaluation.now)?;
                let reasons = vec![source_health_reason(source.id, health, &state)];
                (health, reasons)
            }
        };
        rows.push(SourceHealthRow {
            source_id: source.id,
            health,
            reasons,
        });
    }
    rows.sort_by_key(|row| row.source_id);
    Ok(rows)
}

/// Apply the contract's precedence without allowing stale timestamps to
/// override a newer failed/partial/incompatible attempt.
pub fn derive_health(state: &SourceState, now: Instant) -> Result<Health, TimeError> {
    use crate::domain::AttemptOutcome;
    Ok(match state.last_attempt_outcome {
        AttemptOutcome::Incompatible => Health::Incompatible,
        AttemptOutcome::Failed => Health::Unavailable,
        AttemptOutcome::Partial => Health::Partial,
        AttemptOutcome::Never => Health::NeverLoaded,
        AttemptOutcome::Complete => {
            let Some(success) = state.last_success_at else {
                return Ok(Health::NeverLoaded);
            };
            let expiry = success.checked_add_ms(state.fresh_for_ms.get())?;
            if now > expiry {
                Health::Stale
            } else {
                Health::Healthy
            }
        }
    })
}

/// Qualify one source/role dependency. `covered` describes query coverage;
/// health remains a separate fact so stale or failed last-known records are
/// retained rather than discarded.
pub fn qualify(
    input: &EvaluationInput,
    need: &DependencyNeed,
) -> Result<SourceQualification, TimeError> {
    let source_kind = input
        .sources
        .iter()
        .find(|source| source.id == need.source_id)
        .map(|source| source.kind);
    let local = source_kind == Some(SourceKind::Local);
    let state = input
        .source_states
        .iter()
        .find(|state| state.source_id == need.source_id);
    let health = if local {
        Health::Healthy
    } else if let Some(state) = state {
        derive_health(state, input.evaluation.now)?
    } else {
        Health::NeverLoaded
    };
    let covered = if local {
        true
    } else {
        coverage_satisfies(state, &need.coverage)
    };

    let mut reasons = Vec::new();
    if let Some(state) = state {
        reasons.push(source_health_reason(need.source_id, health, state));
    } else if !local {
        reasons.push(source_health_reason(
            need.source_id,
            Health::NeverLoaded,
            &SourceState {
                source_id: need.source_id,
                last_attempt_outcome: crate::domain::AttemptOutcome::Never,
                fresh_for_ms: std::num::NonZeroU64::new(1).unwrap(),
                last_attempt_at: None,
                last_success_at: None,
                anchor_coverage: None,
                deadline_coverage: None,
                tasks_complete: false,
            },
        ));
    }
    // Window-only Deadline dependencies check health without asserting a
    // Deadline endpoint. Do not fabricate an incomplete coverage payload.
    if !(need.role == SourceRole::Deadlines && need.coverage == CoverageNeed::NoAdditionalCoverage)
    {
        reasons.push(coverage_reason(need, state));
    }
    reasons.sort_by_key(|reason| reason.code().as_str());

    Ok(SourceQualification {
        source_id: need.source_id,
        role: need.role,
        health,
        covered,
        reasons,
    })
}

/// Build the factual dependency set for a Deadline assessment. The caller
/// supplies the future Anchor interval only when capacity is being assessed;
/// an overdue/inactive assessment can pass `None` and therefore requires no
/// backwards or future Anchor coverage.
pub fn dependencies_for_deadline(
    input: &EvaluationInput,
    deadline: &Deadline,
    calculation_endpoint: Instant,
    anchor_interval: Option<&TimedSpan>,
) -> Result<Vec<DependencyNeed>, TimeError> {
    let mut needs = Vec::new();
    for required in &input.required_sources {
        let coverage = match required.role {
            SourceRole::Anchors => match anchor_interval {
                Some(interval) => CoverageNeed::AnchorInterval {
                    interval: interval.clone(),
                },
                None => CoverageNeed::NoAdditionalCoverage,
            },
            SourceRole::Deadlines => CoverageNeed::DeadlineEndpoint {
                endpoint: calculation_endpoint,
            },
            SourceRole::Tasks => CoverageNeed::TasksCatalog,
        };
        needs.push(DependencyNeed {
            source_id: required.source_id,
            role: required.role,
            coverage,
        });
    }

    let deadline_role = match &deadline.provenance {
        crate::domain::Provenance::Imported(imported)
            if imported.projection == Projection::TaskDue =>
        {
            SourceRole::Tasks
        }
        _ => SourceRole::Deadlines,
    };
    add_need(
        &mut needs,
        DependencyNeed {
            source_id: deadline.provenance.source_id(),
            role: deadline_role,
            coverage: if deadline_role == SourceRole::Tasks {
                CoverageNeed::TasksCatalog
            } else {
                CoverageNeed::DeadlineEndpoint {
                    endpoint: calculation_endpoint,
                }
            },
        },
    );

    let linked_task = match deadline.fulfillment {
        Fulfillment::TraceTask(task_id) => Some(task_id),
        Fulfillment::Recorded(_) => match fulfillment::deadline_work(input, deadline) {
            crate::domain::DeadlineWork::Task(task_id) => Some(task_id),
            _ => None,
        },
    };
    if let Some(task_id) = linked_task
        && let Some(task) = input.task_refs.iter().find(|task| task.meta.id == task_id)
    {
        add_need(
            &mut needs,
            DependencyNeed {
                source_id: task.provenance.source_id(),
                role: SourceRole::Tasks,
                coverage: CoverageNeed::TasksCatalog,
            },
        );
    }

    if let Some(interval) = anchor_interval {
        for anchor in input.anchors.iter().filter(|anchor| {
            anchor.presence == Presence::Present
                && anchor
                    .span
                    .resolve(&crate::time::TimezoneRules::bundled())
                    .is_ok_and(|span| span.overlaps(interval))
        }) {
            add_need(
                &mut needs,
                DependencyNeed {
                    source_id: anchor.provenance.source_id(),
                    role: SourceRole::Anchors,
                    coverage: CoverageNeed::AnchorInterval {
                        interval: interval.clone(),
                    },
                },
            );
        }
    }

    Ok(merge_needs(needs))
}

/// Build the factual dependency set for one opportunity interval. Explicit
/// requirements are retained, and every present blocking Anchor intersecting
/// the interval is included even when its source was omitted by the caller.
/// Deadline-role requirements do not need additional coverage for a Window;
/// their source health is still qualified.
pub fn dependencies_for_interval(
    input: &EvaluationInput,
    interval: &TimedSpan,
) -> Vec<DependencyNeed> {
    let mut needs = Vec::new();
    for required in &input.required_sources {
        let coverage = match required.role {
            SourceRole::Anchors => CoverageNeed::AnchorInterval {
                interval: interval.clone(),
            },
            SourceRole::Tasks => CoverageNeed::TasksCatalog,
            SourceRole::Deadlines => CoverageNeed::NoAdditionalCoverage,
        };
        needs.push(DependencyNeed {
            source_id: required.source_id,
            role: required.role,
            coverage,
        });
    }

    for anchor in input.anchors.iter().filter(|anchor| {
        anchor.presence == Presence::Present
            && matches!(anchor.occupancy, Occupancy::Busy | Occupancy::Unknown)
            && anchor
                .span
                .resolve(&crate::time::TimezoneRules::bundled())
                .is_ok_and(|span| span.overlaps(interval))
    }) {
        needs.push(DependencyNeed {
            source_id: anchor.provenance.source_id(),
            role: SourceRole::Anchors,
            coverage: CoverageNeed::AnchorInterval {
                interval: interval.clone(),
            },
        });
    }
    merge_needs(needs)
}

fn add_need(needs: &mut Vec<DependencyNeed>, need: DependencyNeed) {
    needs.push(need);
}

fn merge_needs(needs: Vec<DependencyNeed>) -> Vec<DependencyNeed> {
    let mut merged: BTreeMap<(crate::domain::SourceId, SourceRole), CoverageNeed> = BTreeMap::new();
    for need in needs {
        let key = (need.source_id, need.role);
        let value = match (merged.remove(&key), need.coverage) {
            (None, coverage) => coverage,
            (Some(CoverageNeed::TasksCatalog), CoverageNeed::TasksCatalog) => {
                CoverageNeed::TasksCatalog
            }
            (
                Some(CoverageNeed::DeadlineEndpoint { endpoint: first }),
                CoverageNeed::DeadlineEndpoint { endpoint: second },
            ) => CoverageNeed::DeadlineEndpoint {
                endpoint: first.max(second),
            },
            (
                Some(CoverageNeed::AnchorInterval { interval: first }),
                CoverageNeed::AnchorInterval { interval: second },
            ) => CoverageNeed::AnchorInterval {
                interval: TimedSpan::new(
                    first.start().min(second.start()),
                    first.end().max(second.end()),
                )
                .expect("merged positive coverage intervals"),
            },
            (
                Some(CoverageNeed::NoAdditionalCoverage),
                coverage @ CoverageNeed::AnchorInterval { .. },
            )
            | (
                Some(coverage @ CoverageNeed::AnchorInterval { .. }),
                CoverageNeed::NoAdditionalCoverage,
            ) => coverage,
            (Some(CoverageNeed::NoAdditionalCoverage), CoverageNeed::NoAdditionalCoverage) => {
                CoverageNeed::NoAdditionalCoverage
            }
            (Some(previous), _) => previous,
        };
        merged.insert(key, value);
    }
    merged
        .into_iter()
        .map(|((source_id, role), coverage)| DependencyNeed {
            source_id,
            role,
            coverage,
        })
        .collect()
}

fn coverage_satisfies(state: Option<&SourceState>, need: &CoverageNeed) -> bool {
    let Some(state) = state else { return false };
    match need {
        CoverageNeed::AnchorInterval { interval } => {
            state.anchor_coverage.as_ref().is_some_and(|coverage| {
                coverage.start() <= interval.start() && coverage.end() >= interval.end()
            })
        }
        CoverageNeed::DeadlineEndpoint { endpoint } => state
            .deadline_coverage
            .as_ref()
            .is_some_and(|coverage| coverage.contains(*endpoint)),
        CoverageNeed::TasksCatalog => state.tasks_complete,
        CoverageNeed::NoAdditionalCoverage => true,
    }
}

fn source_health_reason(
    source_id: crate::domain::SourceId,
    health: Health,
    state: &SourceState,
) -> Reason {
    Reason::SourceHealth {
        references: vec![Reference::Source(source_id)],
        payload: SourceHealthPayload {
            health,
            last_attempt_outcome: state.last_attempt_outcome,
            last_attempt_at: state.last_attempt_at,
            last_success_at: state.last_success_at,
        },
    }
}

fn coverage_reason(need: &DependencyNeed, state: Option<&SourceState>) -> Reason {
    let (required_interval, required_endpoint, tasks_complete, known_interval) =
        match &need.coverage {
            CoverageNeed::AnchorInterval { interval } => (
                Some(interval.clone()),
                None,
                None,
                state.and_then(|state| state.anchor_coverage.clone()),
            ),
            CoverageNeed::DeadlineEndpoint { endpoint } => (
                None,
                Some(*endpoint),
                None,
                state.and_then(|state| state.deadline_coverage.clone()),
            ),
            CoverageNeed::TasksCatalog => (
                None,
                None,
                Some(state.is_some_and(|state| state.tasks_complete)),
                None,
            ),
            CoverageNeed::NoAdditionalCoverage => (None, None, None, None),
        };
    Reason::SourceCoverage {
        references: vec![Reference::Source(need.source_id)],
        payload: SourceCoveragePayload {
            role: need.role,
            required_interval,
            required_endpoint,
            known_interval,
            tasks_complete,
        },
    }
}
