//! Bounded weekly Routine expansion and explicit occurrence-state inspection.

use crate::{
    domain::{EvaluationInput, Outcome, Routine, RoutineRule, RoutineState, Weekday},
    reasons::{BoundaryPayload, Reason},
    results::{OccurrenceKey, OccurrencePhase, Reference, RoutineOccurrence},
    time::{DateSpan, Instant, TimeError, TimedSpan, TimezoneRules},
};

/// Expand active present weekly routines only across the finite evaluation
/// interval. Historical occurrences that intersect the interval are retained
/// with `past_unrecorded`; callers choose whether to expose them as candidates.
pub fn expand_routines(input: &EvaluationInput) -> Result<Vec<RoutineOccurrence>, TimeError> {
    expand_routines_with_rules(input, &TimezoneRules::bundled())
}

pub(crate) fn expand_routines_with_rules(
    input: &EvaluationInput,
    rules: &TimezoneRules,
) -> Result<Vec<RoutineOccurrence>, TimeError> {
    let evaluation_span = TimedSpan::new(input.evaluation.now, input.evaluation.evaluation_end)?;
    let mut routines: Vec<&Routine> = input
        .routines
        .iter()
        .filter(|routine| {
            routine.presence == crate::domain::Presence::Present
                && routine.state == RoutineState::Active
        })
        .collect();
    routines.sort_by_key(|routine| routine.meta.id);

    let mut occurrences = Vec::new();
    for routine in routines {
        let RoutineRule::Weekly(rule) = &routine.rule;
        let first = rules.date_at(input.evaluation.now, rule.zone)?;
        let last = rules.date_at(input.evaluation.evaluation_end, rule.zone)?;
        let mut date = first.max(rule.start_date);
        while date <= last {
            if selected(rule, date) {
                let occurrence = occurrence_state_with_rules(
                    routine,
                    &input.routine_outcomes,
                    date,
                    input.evaluation.now,
                    rules,
                )?;
                if occurrence.span.resolve(rules)?.overlaps(&evaluation_span) {
                    occurrences.push(occurrence);
                }
            }
            if date == last {
                break;
            }
            date = date.next_day()?;
        }
    }
    occurrences.sort_by_key(|occurrence| occurrence.key);
    Ok(occurrences)
}

/// Return only current and future unrecorded occurrences. This filter is a
/// derived candidate choice; it does not erase historical state.
pub fn eligible_occurrences(input: &EvaluationInput) -> Result<Vec<RoutineOccurrence>, TimeError> {
    Ok(expand_routines(input)?
        .into_iter()
        .filter(|occurrence| {
            matches!(
                occurrence.phase,
                OccurrencePhase::Current | OccurrencePhase::Future
            )
        })
        .collect())
}

/// Inspect an explicit date, including a historical date that is outside the
/// normal generated candidate set. A changed rule or paused/removed Routine
/// makes the occurrence inactive while retaining its stable key and date span.
pub fn occurrence_state(
    routine: &Routine,
    outcomes: &[crate::domain::RoutineOutcome],
    date: crate::time::LocalDate,
    now: Instant,
    rules: &TimezoneRules,
) -> Result<RoutineOccurrence, TimeError> {
    occurrence_state_with_rules(routine, outcomes, date, now, rules)
}

fn occurrence_state_with_rules(
    routine: &Routine,
    outcomes: &[crate::domain::RoutineOutcome],
    date: crate::time::LocalDate,
    now: Instant,
    rules: &TimezoneRules,
) -> Result<RoutineOccurrence, TimeError> {
    let RoutineRule::Weekly(rule) = &routine.rule;
    let span = DateSpan::new(date, date.next_day()?, rule.zone)?;
    let key = OccurrenceKey {
        routine_id: routine.meta.id,
        date,
    };
    let resolved = span.resolve(rules)?;
    let selected = routine.presence == crate::domain::Presence::Present
        && routine.state == RoutineState::Active
        && selected(rule, date);
    let outcome = outcomes
        .iter()
        .find(|outcome| outcome.routine_id == routine.meta.id && outcome.date == date)
        .map(|outcome| outcome.outcome);

    let (phase, mut reasons) = if !selected {
        (OccurrencePhase::Inactive, Vec::new())
    } else if let Some(outcome) = outcome {
        match outcome {
            Outcome::Done => (OccurrencePhase::Done, Vec::new()),
            Outcome::Skipped => (OccurrencePhase::Skipped, Vec::new()),
        }
    } else if now < resolved.start() {
        (OccurrencePhase::Future, Vec::new())
    } else if now < resolved.end() {
        (OccurrencePhase::Current, Vec::new())
    } else {
        (
            OccurrencePhase::PastUnrecorded,
            vec![Reason::RoutinePastUnrecorded {
                references: vec![Reference::RoutineOccurrence(key)],
                payload: BoundaryPayload {
                    boundary: resolved.end(),
                    now,
                },
            }],
        )
    };
    reasons.sort_by_key(|reason| reason.code().as_str());
    Ok(RoutineOccurrence {
        key,
        span,
        phase,
        reasons,
    })
}

fn selected(rule: &crate::domain::WeeklyRule, date: crate::time::LocalDate) -> bool {
    if date < rule.start_date || rule.until_date_exclusive.is_some_and(|until| date >= until) {
        return false;
    }
    let weekday = match date.weekday_number() {
        1 => Weekday::Mon,
        2 => Weekday::Tue,
        3 => Weekday::Wed,
        4 => Weekday::Thu,
        5 => Weekday::Fri,
        6 => Weekday::Sat,
        7 => Weekday::Sun,
        _ => unreachable!("weekday number is one through seven"),
    };
    rule.weekdays.contains(&weekday)
}
