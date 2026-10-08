//! The Calendar utility's read model: stored facts laid out over any range.
//!
//! Rows are projections of stored records. Deadline risk is copied from the
//! one Horizon evaluation when the deadline is inside its extent; nothing here
//! computes pressure, suggests time, or changes a fact.
use crate::{
    local::LocalState,
    series::{self, EventDetails},
    store::StoredView,
};
use serde::Serialize;
use std::collections::BTreeMap;
use temporal_core::{domain::*, results::*, time::*};

/// The widest range one request may cover (a six-week month plus margins).
pub const MAX_RANGE_MS: u64 = 70 * 86_400_000;

#[derive(Clone, Debug, Serialize)]
pub struct CalendarEvent {
    pub id: String,
    pub kind: &'static str,
    pub title: String,
    pub start: i64,
    pub end: i64,
    pub all_day: bool,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub source_id: String,
    pub source_label: String,
    pub source_kind: &'static str,
    pub color: Option<String>,
    pub editable: bool,
    pub series_id: Option<String>,
    pub occurrence_date: Option<String>,
    pub location: Option<String>,
    pub notes: Option<String>,
    pub occupancy: Option<Occupancy>,
    pub tentative: bool,
    pub state: String,
    pub risk: Option<Risk>,
}

#[derive(Clone, Debug, Serialize)]
pub struct AvailabilitySpan {
    pub start: i64,
    pub end: i64,
}

#[derive(Clone, Debug, Serialize)]
pub struct CalendarRange {
    pub from: i64,
    pub to: i64,
    pub zone: String,
    pub events: Vec<CalendarEvent>,
    pub availability: Vec<AvailabilitySpan>,
}

pub fn source_kind(kind: SourceKind) -> &'static str {
    match kind {
        SourceKind::Local => "local",
        SourceKind::Trace => "trace",
        SourceKind::Quercus => "quercus",
        SourceKind::Google => "google",
        SourceKind::Outlook => "outlook",
        SourceKind::Calendar => "calendar",
    }
}

fn anchor_state(span: &TimedSpan, now: Instant) -> &'static str {
    if span.end() <= now {
        "past"
    } else if span.start() <= now {
        "now"
    } else {
        "upcoming"
    }
}

/// Shared shape for an Anchor row, local or imported.
pub struct AnchorRow<'a> {
    pub id: String,
    pub anchor: &'a Anchor,
    pub source: &'a Source,
    pub color: Option<String>,
    pub editable: bool,
    pub series: Option<(AnchorId, LocalDate)>,
    pub details: Option<&'a EventDetails>,
    pub place: Option<String>,
}

pub fn anchor_event(row: AnchorRow<'_>, now: Instant) -> Result<CalendarEvent, TimeError> {
    let rules = TimezoneRules::bundled();
    let resolved = row.anchor.span.resolve(&rules)?;
    let (all_day, start_date, end_date) = match &row.anchor.span {
        TemporalSpan::Dates(span) => (
            true,
            Some(span.start_date().to_string()),
            Some(span.end_date_exclusive().to_string()),
        ),
        TemporalSpan::Timed(_) => (false, None, None),
    };
    Ok(CalendarEvent {
        id: row.id,
        kind: "anchor",
        title: row.anchor.title.clone(),
        start: resolved.start().epoch_ms(),
        end: resolved.end().epoch_ms(),
        all_day,
        start_date,
        end_date,
        source_id: row.source.id.to_string(),
        source_label: row.source.label.clone(),
        source_kind: source_kind(row.source.kind),
        color: row.color,
        editable: row.editable,
        series_id: row.series.map(|(id, _)| id.to_string()),
        occurrence_date: row.series.map(|(_, date)| date.to_string()),
        location: row
            .place
            .or_else(|| row.details.and_then(|d| d.place.clone()))
            .or_else(|| row.anchor.location.as_ref().map(ToString::to_string)),
        notes: row.details.and_then(|d| d.notes.clone()),
        occupancy: Some(row.anchor.occupancy),
        tentative: row.anchor.reported_certainty == ReportedCertainty::Tentative,
        state: anchor_state(&resolved, now).into(),
        risk: None,
    })
}

/// Shared shape for a Deadline row, local or imported.
pub struct DeadlineRow<'a> {
    pub deadline: &'a Deadline,
    pub source: &'a Source,
    pub color: Option<String>,
    pub editable: bool,
    pub details: Option<&'a EventDetails>,
    pub resolved: Option<bool>,
}

pub fn deadline_event(
    row: DeadlineRow<'_>,
    now: Instant,
    risks: &BTreeMap<DeadlineId, (Risk, String)>,
) -> Result<CalendarEvent, TimeError> {
    let rules = TimezoneRules::bundled();
    let endpoint = row.deadline.cutoff.endpoint(&rules)?;
    let (all_day, start_date, end_date, start) = match &row.deadline.cutoff {
        Cutoff::OnDate { date, zone } => (
            true,
            Some(date.to_string()),
            Some(date.next_day()?.to_string()),
            rules.start_of_date(*date, *zone)?,
        ),
        Cutoff::At { .. } => (false, None, None, endpoint),
    };
    let evaluated = risks.get(&row.deadline.meta.id);
    let state = match (&row.deadline.fulfillment, row.resolved) {
        (Fulfillment::Recorded(RecordedResolution::Satisfied(_)), _) => "satisfied".to_string(),
        (Fulfillment::Recorded(RecordedResolution::Cancelled(_)), _) => "cancelled".to_string(),
        (_, Some(true)) => "satisfied".to_string(),
        _ => match evaluated {
            Some((_, phase)) => phase.clone(),
            None if endpoint <= now => "overdue".to_string(),
            None => "upcoming".to_string(),
        },
    };
    Ok(CalendarEvent {
        id: row.deadline.meta.id.to_string(),
        kind: "deadline",
        title: row.deadline.title.clone(),
        start: start.epoch_ms(),
        end: if all_day {
            endpoint.epoch_ms()
        } else {
            start.epoch_ms()
        },
        all_day,
        start_date,
        end_date,
        source_id: row.source.id.to_string(),
        source_label: row.source.label.clone(),
        source_kind: source_kind(row.source.kind),
        color: row.color,
        editable: row.editable,
        series_id: None,
        occurrence_date: None,
        location: row.details.and_then(|d| d.place.clone()),
        notes: row.details.and_then(|d| d.notes.clone()),
        occupancy: None,
        tentative: false,
        state,
        risk: evaluated.map(|(risk, _)| *risk),
    })
}

fn name(value: &impl Serialize) -> String {
    serde_json::to_value(value)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

/// Deadline risk and phase from the Horizon evaluation, by deadline.
pub fn evaluated_risks(
    input: &EvaluationInput,
) -> Result<BTreeMap<DeadlineId, (Risk, String)>, String> {
    let output = temporal_core::evaluate(input)
        .map_err(|_| "Cached temporal data could not be evaluated.".to_string())?;
    let phases: BTreeMap<DeadlineId, String> = output
        .deadline_states
        .iter()
        .map(|s| (s.id, name(&s.phase)))
        .collect();
    Ok(output
        .pressures
        .iter()
        .map(|p| {
            (
                p.deadline_id,
                (
                    p.risk,
                    phases.get(&p.deadline_id).cloned().unwrap_or_default(),
                ),
            )
        })
        .collect())
}

fn weekday_of(date: LocalDate) -> Weekday {
    match date.weekday_number() {
        1 => Weekday::Mon,
        2 => Weekday::Tue,
        3 => Weekday::Wed,
        4 => Weekday::Thu,
        5 => Weekday::Fri,
        6 => Weekday::Sat,
        _ => Weekday::Sun,
    }
}

fn local_rows(
    local: &LocalState,
    now: Instant,
    from: Instant,
    to: Instant,
    risks: &BTreeMap<DeadlineId, (Risk, String)>,
) -> Result<Vec<CalendarEvent>, String> {
    let rules = TimezoneRules::bundled();
    let range = TimedSpan::new(from, to).map_err(|e| e.to_string())?;
    let source = &local.source;
    let mut events = Vec::new();
    for anchor in local
        .anchors
        .iter()
        .filter(|a| a.presence == Presence::Present)
    {
        if !anchor
            .span
            .resolve(&rules)
            .is_ok_and(|s| s.overlaps(&range))
        {
            continue;
        }
        let id = anchor.meta.id.to_string();
        events.push(
            anchor_event(
                AnchorRow {
                    details: local.event_details.get(&id),
                    id,
                    anchor,
                    source,
                    color: None,
                    editable: true,
                    series: None,
                    place: None,
                },
                now,
            )
            .map_err(|e| e.to_string())?,
        );
    }
    for record in &local.anchor_series {
        for occurrence in
            series::expand_series(record, source.id, from, to).map_err(|e| e.to_string())?
        {
            events.push(
                anchor_event(
                    AnchorRow {
                        id: occurrence.anchor.meta.id.to_string(),
                        anchor: &occurrence.anchor,
                        source,
                        color: None,
                        editable: true,
                        series: Some((occurrence.series_id, occurrence.date)),
                        details: Some(&record.details),
                        place: None,
                    },
                    now,
                )
                .map_err(|e| e.to_string())?,
            );
        }
    }
    for deadline in local
        .deadlines
        .iter()
        .filter(|d| d.presence == Presence::Present)
    {
        let endpoint = deadline
            .cutoff
            .endpoint(&rules)
            .map_err(|e| e.to_string())?;
        if endpoint < from || endpoint > to {
            continue;
        }
        events.push(
            deadline_event(
                DeadlineRow {
                    deadline,
                    source,
                    color: None,
                    editable: true,
                    details: local.event_details.get(&deadline.meta.id.to_string()),
                    resolved: None,
                },
                now,
                risks,
            )
            .map_err(|e| e.to_string())?,
        );
    }
    for intention in local
        .intentions
        .iter()
        .filter(|i| i.presence == Presence::Present)
    {
        let Some(span) = &intention.preferred_span else {
            continue;
        };
        let resolved = span.resolve(&rules).map_err(|e| e.to_string())?;
        if !resolved.overlaps(&range) {
            continue;
        }
        let (all_day, start_date, end_date) = match span {
            TemporalSpan::Dates(s) => (
                true,
                Some(s.start_date().to_string()),
                Some(s.end_date_exclusive().to_string()),
            ),
            TemporalSpan::Timed(_) => (false, None, None),
        };
        events.push(CalendarEvent {
            id: intention.meta.id.to_string(),
            kind: "intention",
            title: intention.title.clone(),
            start: resolved.start().epoch_ms(),
            end: resolved.end().epoch_ms(),
            all_day,
            start_date,
            end_date,
            source_id: source.id.to_string(),
            source_label: source.label.clone(),
            source_kind: "local",
            color: None,
            editable: true,
            series_id: None,
            occurrence_date: None,
            location: None,
            notes: None,
            occupancy: None,
            tentative: false,
            state: name(&intention.state),
            risk: None,
        });
    }
    for routine in local
        .routines
        .iter()
        .filter(|r| r.presence == Presence::Present && r.state == RoutineState::Active)
    {
        let RoutineRule::Weekly(rule) = &routine.rule;
        let mut date = rules.date_at(from, rule.zone).map_err(|e| e.to_string())?;
        let last = rules.date_at(to, rule.zone).map_err(|e| e.to_string())?;
        while date <= last {
            let in_rule = date >= rule.start_date
                && rule.until_date_exclusive.is_none_or(|until| date < until)
                && rule.weekdays.contains(&weekday_of(date));
            if in_rule {
                let start = rules
                    .start_of_date(date, rule.zone)
                    .map_err(|e| e.to_string())?;
                let next = date.next_day().map_err(|e| e.to_string())?;
                let end = rules
                    .start_of_date(next, rule.zone)
                    .map_err(|e| e.to_string())?;
                let outcome = local
                    .routine_outcomes
                    .iter()
                    .find(|o| o.routine_id == routine.meta.id && o.date == date)
                    .map(|o| name(&o.outcome));
                events.push(CalendarEvent {
                    id: format!("{}:{}", routine.meta.id, date),
                    kind: "routine",
                    title: routine.title.clone(),
                    start: start.epoch_ms(),
                    end: end.epoch_ms(),
                    all_day: true,
                    start_date: Some(date.to_string()),
                    end_date: Some(next.to_string()),
                    source_id: source.id.to_string(),
                    source_label: source.label.clone(),
                    source_kind: "local",
                    color: None,
                    editable: true,
                    series_id: Some(routine.meta.id.to_string()),
                    occurrence_date: Some(date.to_string()),
                    location: None,
                    notes: None,
                    occupancy: None,
                    tentative: false,
                    state: outcome.unwrap_or_else(|| "preferred".into()),
                    risk: None,
                });
            }
            date = date.next_day().map_err(|e| e.to_string())?;
        }
    }
    Ok(events)
}

fn availability(
    local: &LocalState,
    from: Instant,
    to: Instant,
) -> Result<Vec<AvailabilitySpan>, String> {
    let range = TimedSpan::new(from, to).map_err(|e| e.to_string())?;
    let mut spans: Vec<AvailabilitySpan> = local
        .availability
        .iter()
        .filter(|d| d.span.overlaps(&range))
        .map(|d| AvailabilitySpan {
            start: d.span.start().epoch_ms(),
            end: d.span.end().epoch_ms(),
        })
        .collect();
    if let Some(usual) = &local.usual_availability {
        spans.extend(
            series::expand_usual(usual, &local.availability, from, to)
                .map_err(|e| e.to_string())?
                .into_iter()
                .map(|d| AvailabilitySpan {
                    start: d.span.start().epoch_ms(),
                    end: d.span.end().epoch_ms(),
                }),
        );
    }
    spans.sort_by_key(|s| (s.start, s.end));
    Ok(spans)
}

/// Every stored fact and soft item touching `[from, to)`, plus declared
/// availability, ordered by start, kind, and identity.
pub fn range(
    stored: &StoredView,
    now: Instant,
    from: Instant,
    to: Instant,
) -> Result<CalendarRange, String> {
    let span = to
        .duration_since(from)
        .map_err(|_| "The calendar range must end after it starts.".to_string())?;
    if span == 0 || span > MAX_RANGE_MS {
        return Err("Ask for a calendar range between one minute and ten weeks.".into());
    }
    let risks = evaluated_risks(&stored.input)?;
    let mut events = local_rows(&stored.local, now, from, to, &risks)?;
    events.extend(stored.imported_events(now, from, to, &risks)?);
    events.sort_by(|a, b| (a.start, a.kind, &a.id).cmp(&(b.start, b.kind, &b.id)));
    Ok(CalendarRange {
        from: from.epoch_ms(),
        to: to.epoch_ms(),
        zone: stored.settings.zone().to_string(),
        events,
        availability: availability(&stored.local, from, to)?,
    })
}
