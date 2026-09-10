//! Bounded deterministic Today selection (today-v1).
//!
//! This module selects from one existing core evaluation. It derives no new
//! temporal fact, writes nothing, reserves no time, and creates no obligation.
//! Its documented policy is `docs/TODAY-POLICY.md`.

use serde::Serialize;
use temporal_core::{
    EvaluationError,
    domain::*,
    fit,
    reasons::{EmptyPayload, Reason},
    results::*,
    time::{Cutoff, Instant, LocalDate, TemporalSpan, TimeError, TimedSpan, TimezoneRules, ZoneId},
};

/// The bounded selection identified by `docs/TODAY-POLICY.md`.
pub const POLICY: &str = "today-v1";
pub const WORTH_DOING_MAX: usize = 3;
pub const LOOSE_MAX: usize = 2;
pub const RADAR_MAX: usize = 3;
/// Presentation bound only. Every fixed fact for today stays inspectable.
pub const FIXED_PREVIEW: usize = 4;

#[derive(Debug)]
pub enum TodayError {
    Evaluation(EvaluationError),
    Time(TimeError),
}

impl std::fmt::Display for TodayError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Evaluation(error) => error.fmt(formatter),
            Self::Time(error) => error.fmt(formatter),
        }
    }
}
impl std::error::Error for TodayError {}
impl From<TimeError> for TodayError {
    fn from(error: TimeError) -> Self {
        Self::Time(error)
    }
}

/// A stored fact selected for awareness. Its evidence stays in the evaluation.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum FactRef {
    Anchor(AnchorId),
    Deadline(DeadlineId),
}

/// Why one flexible row is placed where it is. This is ordering rationale,
/// separate from the stored facts of the target and from its typed reasons.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
pub struct AdviceBasis {
    pub before_next_anchor: bool,
    pub preferred_today: bool,
    pub deadline: Option<DeadlineId>,
    pub risk: Option<Risk>,
    pub cutoff: Option<Instant>,
    pub importance: Importance,
    pub qualification: Qualification,
}

/// One advisory row. The range is a compatible span, never a reservation.
///
/// Selecting a row opens its evidence. It is not completion:
/// ```compile_fail
/// use temporal_app::today::AdviceRow;
/// fn accept(row: &mut AdviceRow) { row.completed = true; }
/// ```
/// and it never becomes a stored obligation:
/// ```compile_fail
/// use temporal_app::today::AdviceRow;
/// use temporal_core::domain::EvaluationInput;
/// fn commit(input: &mut EvaluationInput, row: AdviceRow) { input.deadlines.push(row); }
/// ```
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct AdviceRow {
    pub target: WorkTarget,
    pub range: TimedSpan,
    pub window: WindowKey,
    pub basis: AdviceBasis,
    pub suggestion: Suggestion,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct TodayEdit {
    pub policy: &'static str,
    pub date: LocalDate,
    pub zone: ZoneId,
    pub now: Instant,
    pub day_end: Instant,
    pub bound: Instant,
    pub fixed: Vec<FactRef>,
    pub worth_doing: Vec<AdviceRow>,
    pub loose: Vec<AdviceRow>,
    pub radar: Vec<FactRef>,
    pub radar_omitted: usize,
}

/// Evaluate one snapshot and select its bounded edit from that single result.
pub fn evaluate(input: &EvaluationInput) -> Result<(EvaluationOutput, TodayEdit), TodayError> {
    let output = temporal_core::evaluate(input).map_err(TodayError::Evaluation)?;
    let edit = select(input, &output)?;
    Ok((output, edit))
}

/// Select from an evaluation that was already derived for this exact input.
pub fn select(input: &EvaluationInput, output: &EvaluationOutput) -> Result<TodayEdit, TimeError> {
    let rules = TimezoneRules::bundled();
    let zone = input.evaluation.display_zone;
    let now = input.evaluation.now;
    let date = rules.date_at(now, zone)?;
    let day_start = rules.start_of_date(date, zone)?;
    let day_end = rules.start_of_date(date.next_day()?, zone)?;
    // Advice never outlives the civil day or the evaluated range.
    let bound = day_end.min(input.evaluation.evaluation_end);
    let today = TimedSpan::new(day_start, day_end)?;
    let remainder = TimedSpan::new(now, bound)?;

    let fixed = fixed(input, &rules, zone, date, &today)?;
    let (worth_doing, loose) = advice(input, output, &rules, &remainder)?;
    let shown = shown_facts(&fixed, &worth_doing, &loose);
    let (radar, radar_omitted) = radar(input, output, &rules, &shown)?;

    Ok(TodayEdit {
        policy: POLICY,
        date,
        zone,
        now,
        day_end,
        bound,
        fixed,
        worth_doing,
        loose,
        radar,
        radar_omitted,
    })
}

fn fixed(
    input: &EvaluationInput,
    rules: &TimezoneRules,
    zone: ZoneId,
    date: LocalDate,
    today: &TimedSpan,
) -> Result<Vec<FactRef>, TimeError> {
    let mut rows = Vec::new();
    for anchor in present_anchors(input) {
        let span = anchor.span.resolve(rules)?;
        if span.overlaps(today) {
            rows.push((span.start(), FactRef::Anchor(anchor.meta.id)));
        }
    }
    for deadline in input
        .deadlines
        .iter()
        .filter(|deadline| deadline.presence == Presence::Present)
    {
        let (shown_at, shown_date) = displayed_cutoff(&deadline.cutoff, rules, zone)?;
        if shown_date == date {
            rows.push((shown_at, FactRef::Deadline(deadline.meta.id)));
        }
    }
    rows.sort_unstable();
    Ok(rows.into_iter().map(|(_, fact)| fact).collect())
}

/// An on-date cutoff belongs to the date containing the last instant it covers.
fn displayed_cutoff(
    cutoff: &Cutoff,
    rules: &TimezoneRules,
    zone: ZoneId,
) -> Result<(Instant, LocalDate), TimeError> {
    match *cutoff {
        Cutoff::At { instant, .. } => Ok((instant, rules.date_at(instant, zone)?)),
        Cutoff::OnDate { .. } => {
            let endpoint = cutoff.endpoint(rules)?;
            let last = endpoint.checked_sub_ms(1)?;
            Ok((endpoint, rules.date_at(last, zone)?))
        }
    }
}

/// Awareness covers what the edit has not already put in front of the user.
/// An associated cutoff shown on a flexible row is not repeated here.
fn shown_facts(fixed: &[FactRef], worth_doing: &[AdviceRow], loose: &[AdviceRow]) -> Vec<FactRef> {
    let mut shown = fixed.to_vec();
    for row in worth_doing.iter().chain(loose) {
        if let WorkTarget::Deadline(id) = row.target {
            shown.push(FactRef::Deadline(id));
        }
        if let Some(id) = row.basis.deadline {
            shown.push(FactRef::Deadline(id));
        }
    }
    shown.sort_unstable();
    shown.dedup();
    shown
}

fn radar(
    input: &EvaluationInput,
    output: &EvaluationOutput,
    rules: &TimezoneRules,
    shown: &[FactRef],
) -> Result<(Vec<FactRef>, usize), TimeError> {
    let end = input.evaluation.evaluation_end;
    let mut rows = Vec::new();
    for state in &output.deadline_states {
        let fact = FactRef::Deadline(state.id);
        if state.presence != Presence::Present
            || !matches!(
                state.resolution,
                Resolution::Unresolved | Resolution::Unknown
            )
            || shown.contains(&fact)
            || state.endpoint > end
        {
            continue;
        }
        let risk = output
            .pressures
            .iter()
            .find(|row| row.deadline_id == state.id)
            .map(|row| row.risk);
        let rank = if state.phase == DeadlinePhase::Overdue {
            0
        } else if risk == Some(Risk::Insufficient) {
            1
        } else if risk == Some(Risk::Tight) {
            2
        } else {
            3
        };
        rows.push((rank, state.endpoint, fact));
    }
    for state in output
        .anchor_states
        .iter()
        .filter(|state| state.phase == AnchorPhase::Upcoming)
    {
        let fact = FactRef::Anchor(state.id);
        let Some(anchor) = present_anchors(input).find(|anchor| anchor.meta.id == state.id) else {
            continue;
        };
        let start = anchor.span.resolve(rules)?.start();
        if shown.contains(&fact) || start > end {
            continue;
        }
        rows.push((3, start, fact));
    }
    rows.sort_unstable();
    let omitted = rows.len().saturating_sub(RADAR_MAX);
    Ok((
        rows.into_iter()
            .take(RADAR_MAX)
            .map(|(_, _, fact)| fact)
            .collect(),
        omitted,
    ))
}

fn advice(
    input: &EvaluationInput,
    output: &EvaluationOutput,
    rules: &TimezoneRules,
    remainder: &TimedSpan,
) -> Result<(Vec<AdviceRow>, Vec<AdviceRow>), TimeError> {
    // The same core fit calculation, asked only about the rest of today. A
    // clipped assessment span never becomes a new availability declaration.
    let clipped: Vec<Window> = output
        .windows
        .iter()
        .filter_map(|window| {
            window.span.intersection(remainder).map(|span| Window {
                span,
                ..window.clone()
            })
        })
        .collect();
    let rows = fit::derive(input, &clipped)?;
    let next_anchor = next_anchor_start(input, rules, input.evaluation.now, remainder.end())?;

    let mut worth_doing = Vec::new();
    let mut loose = Vec::new();
    for target in fit::work_targets(input)? {
        let Some(row) = best_fit(&rows, target) else {
            continue;
        };
        let window = output
            .windows
            .iter()
            .find(|window| window.key == row.window_key)
            .expect("a clipped Window retains its identity");
        let range = row.span.clone().expect("a fitting row has a span");
        let associated = associated_pressure(output, target);
        let basis = AdviceBasis {
            before_next_anchor: next_anchor.is_some_and(|start| range.end() <= start),
            preferred_today: preferred_today(output, target),
            deadline: associated.map(|pressure| pressure.deadline_id),
            risk: associated.map(|pressure| pressure.risk),
            cutoff: associated.map(|pressure| pressure.endpoint),
            importance: importance(input, target),
            qualification: qualification(input, output, window, target, associated),
        };
        let advice = AdviceRow {
            target,
            suggestion: suggestion(input, window, row, &range, associated),
            window: row.window_key.clone(),
            range,
            basis,
        };
        match target {
            WorkTarget::Task(_) | WorkTarget::Deadline(_) => worth_doing.push(advice),
            WorkTarget::Intention(_) | WorkTarget::RoutineOccurrence(_) => loose.push(advice),
        }
    }
    worth_doing.sort_by_key(work_order);
    loose.sort_by_key(loose_order);
    worth_doing.truncate(WORTH_DOING_MAX);
    loose.truncate(LOOSE_MAX);
    Ok((worth_doing, loose))
}

fn best_fit(rows: &[FitResult], target: WorkTarget) -> Option<&FitResult> {
    rows.iter()
        .filter(|row| row.target == target && row.status == FitStatus::Fits)
        .min_by(|left, right| {
            let (left_span, right_span) = (
                left.span.as_ref().expect("a fitting row has a span"),
                right.span.as_ref().expect("a fitting row has a span"),
            );
            left_span
                .start()
                .cmp(&right_span.start())
                .then(left_span.end().cmp(&right_span.end()))
                .then_with(|| left.window_key.cmp(&right.window_key))
        })
}

fn work_order(row: &AdviceRow) -> (u8, u8, u8, Instant, u8, Instant, WorkTarget) {
    (
        u8::from(!row.basis.before_next_anchor),
        risk_rank(row.basis.risk),
        u8::from(row.basis.cutoff.is_none()),
        row.basis.cutoff.unwrap_or(row.range.start()),
        importance_rank(row.basis.importance),
        row.range.start(),
        row.target,
    )
}

fn loose_order(row: &AdviceRow) -> (u8, u8, Instant, WorkTarget) {
    (
        u8::from(!row.basis.before_next_anchor),
        u8::from(!row.basis.preferred_today),
        row.range.start(),
        row.target,
    )
}

fn risk_rank(risk: Option<Risk>) -> u8 {
    match risk {
        Some(Risk::Insufficient) => 0,
        Some(Risk::Tight) => 1,
        Some(Risk::Unknown) => 3,
        Some(_) => 2,
        None => 4,
    }
}

fn importance_rank(importance: Importance) -> u8 {
    match importance {
        Importance::High => 0,
        Importance::Normal => 1,
        Importance::Low => 2,
        Importance::Unspecified => 3,
    }
}

fn present_anchors(input: &EvaluationInput) -> impl Iterator<Item = &Anchor> {
    input
        .anchors
        .iter()
        .filter(|anchor| anchor.presence == Presence::Present)
}

fn next_anchor_start(
    input: &EvaluationInput,
    rules: &TimezoneRules,
    now: Instant,
    bound: Instant,
) -> Result<Option<Instant>, TimeError> {
    let mut next: Option<Instant> = None;
    for anchor in present_anchors(input).filter(|anchor| anchor.occupancy != Occupancy::Transparent)
    {
        let start = anchor.span.resolve(rules)?.start();
        if start > now && start < bound {
            next = Some(next.map_or(start, |current| current.min(start)));
        }
    }
    Ok(next)
}

fn associated_pressure(output: &EvaluationOutput, target: WorkTarget) -> Option<&PressureResult> {
    output
        .pressures
        .iter()
        .filter(|row| {
            row.presence == Presence::Present
                && matches!(row.resolution, Resolution::Unresolved | Resolution::Unknown)
                && match target {
                    WorkTarget::Deadline(id) => row.deadline_id == id,
                    _ => row.work_target == Some(target),
                }
        })
        .min_by_key(|row| (row.endpoint, row.deadline_id))
}

fn preferred_today(output: &EvaluationOutput, target: WorkTarget) -> bool {
    match target {
        WorkTarget::Intention(id) => output
            .intention_states
            .iter()
            .any(|state| state.id == id && state.phase == IntentionPhase::PreferredNow),
        WorkTarget::RoutineOccurrence(key) => output
            .routine_occurrences
            .iter()
            .any(|row| row.key == key && row.phase == OccurrencePhase::Current),
        _ => false,
    }
}

fn importance(input: &EvaluationInput, target: WorkTarget) -> Importance {
    match target {
        WorkTarget::Task(id) => input
            .task_annotations
            .iter()
            .find(|annotation| annotation.target_id == id)
            .map_or(Importance::Unspecified, |annotation| annotation.importance),
        WorkTarget::Deadline(id) => input
            .deadline_annotations
            .iter()
            .find(|annotation| annotation.target_id == id)
            .map_or(Importance::Unspecified, |annotation| annotation.importance),
        _ => Importance::Unspecified,
    }
}

fn target_source(input: &EvaluationInput, target: WorkTarget) -> Option<SourceId> {
    match target {
        WorkTarget::Task(id) => input
            .task_refs
            .iter()
            .find(|task| task.meta.id == id)
            .map(|task| task.provenance.source_id()),
        WorkTarget::Deadline(id) => input
            .deadlines
            .iter()
            .find(|deadline| deadline.meta.id == id)
            .map(|deadline| deadline.provenance.source_id()),
        WorkTarget::Intention(id) => input
            .intentions
            .iter()
            .find(|intention| intention.meta.id == id)
            .map(|intention| intention.provenance.source_id()),
        WorkTarget::RoutineOccurrence(key) => input
            .routines
            .iter()
            .find(|routine| routine.meta.id == key.routine_id)
            .map(|routine| routine.provenance.source_id()),
    }
}

/// Known fit may still rest on last-known source data. Say so instead of
/// presenting a conditional row as an unqualified declared-input result.
fn qualification(
    input: &EvaluationInput,
    output: &EvaluationOutput,
    window: &Window,
    target: WorkTarget,
    associated: Option<&PressureResult>,
) -> Qualification {
    let conditional_window = window
        .source_qualifications
        .iter()
        .any(|row| row.health != Health::Healthy || !row.covered);
    let conditional_pressure =
        associated.is_some_and(|row| row.qualification == Qualification::Conditional);
    let conditional_source = target_source(input, target).is_some_and(|source| {
        output
            .source_health
            .iter()
            .any(|row| row.source_id == source && row.health != Health::Healthy)
    });
    if conditional_window || conditional_pressure || conditional_source {
        Qualification::Conditional
    } else {
        Qualification::DeclaredInputs
    }
}

/// Produce advice that carries its producing key, its assessed range, its
/// expiry, and the typed evidence the core already derived for this instant.
fn suggestion(
    input: &EvaluationInput,
    window: &Window,
    row: &FitResult,
    range: &TimedSpan,
    associated: Option<&PressureResult>,
) -> Suggestion {
    let proposed = TemporalSpan::Timed(range.clone());
    let key = SuggestionKey {
        evaluation_key: input.evaluation.key(input.snapshot_revision),
        kind: SuggestionKind::ConsiderWork,
        target: row.target,
        proposed_span: Some(proposed.clone()),
    };
    let mut reasons = window.reasons.clone();
    reasons.extend(row.reasons.iter().cloned());
    for qualification in &window.source_qualifications {
        reasons.extend(qualification.reasons.iter().cloned());
    }
    if let Some(pressure) = associated {
        reasons.extend(pressure.reasons.iter().cloned());
    }
    // Individual fitting ranges may overlap. The edit certifies no combined
    // capacity and promises no completion.
    reasons.push(Reason::IndividualCapacityOnly {
        references: vec![Reference::from(row.target)],
        payload: EmptyPayload {},
    });
    normalize(&mut reasons);
    Suggestion {
        key,
        kind: SuggestionKind::ConsiderWork,
        target: row.target,
        proposed_span: Some(proposed),
        created_at: input.evaluation.now,
        valid_until: range.end(),
        reasons,
    }
}

/// The canonical typed reason order of the core, applied to composed evidence.
fn normalize(reasons: &mut Vec<Reason>) {
    for reason in reasons.iter_mut() {
        reason.references_mut().sort();
        reason.references_mut().dedup();
    }
    reasons.sort_by_cached_key(|reason| {
        (
            reason.code().as_str(),
            reason.references().to_vec(),
            temporal_core::codec::canonical_bytes(reason).expect("typed reason is serializable"),
        )
    });
    reasons.dedup();
}
