//! Read-only projection for the first Horizon. It consumes core decisions;
//! it never creates suggestions, changes facts, or calculates a second pressure.
use crate::Scenario;
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serde::Serialize;
use temporal_core::{domain::*, reasons::Reason, results::*, time::*};

#[derive(Serialize)]
pub struct Detail {
    label: String,
    value: String,
}
#[derive(Serialize)]
pub struct Item {
    id: String,
    species: &'static str,
    title: String,
    source: String,
    ownership: String,
    phase: String,
    start: Option<i64>,
    end: Option<i64>,
    when: String,
    milestone: bool,
    risk: Option<Risk>,
    conditional: bool,
    facts: Vec<Detail>,
    reasons: Vec<String>,
}
#[derive(Serialize)]
pub struct Tick {
    at: i64,
    label: String,
}
#[derive(Serialize)]
pub struct SourceSummary {
    label: String,
    health: Health,
}
#[derive(Serialize)]
pub struct Snapshot {
    scenario: Scenario,
    now: i64,
    end: i64,
    offset_minutes: u32,
    max_offset_minutes: u32,
    zone: String,
    date_label: String,
    clock_label: String,
    ticks: Vec<Tick>,
    items: Vec<Item>,
    sources: Vec<SourceSummary>,
    has_declarations: bool,
}

fn name(value: &impl Serialize) -> String {
    serde_json::to_value(value)
        .expect("typed enum")
        .as_str()
        .expect("unit enum")
        .into()
}
fn duration(ms: u64) -> String {
    let minutes = ms / 60_000;
    match (minutes / 60, minutes % 60) {
        (0, m) => format!("{m}m"),
        (h, 0) => format!("{h}h"),
        (h, m) => format!("{h}h {m}m"),
    }
}
fn date(at: Instant, zone: ZoneId, format: &str) -> String {
    DateTime::<Utc>::from_timestamp_millis(at.epoch_ms())
        .expect("validated instant")
        .with_timezone(&zone.name().parse::<Tz>().expect("validated IANA zone"))
        .format(format)
        .to_string()
}
fn at(at: Instant, zone: ZoneId) -> String {
    date(at, zone, "%a %b %-d, %-I:%M %p")
}
fn span_label(span: &TemporalSpan, zone: ZoneId) -> String {
    match span {
        TemporalSpan::Timed(s) => format!(
            "{} – {}",
            at(s.start(), zone),
            date(s.end(), zone, "%a %-I:%M %p")
        ),
        TemporalSpan::Dates(s) => {
            let last = s
                .end_date_exclusive()
                .previous_day()
                .expect("nonempty date span");
            if last == s.start_date() {
                format!("Whole date {}, in {}", s.start_date(), s.zone())
            } else {
                format!(
                    "Whole dates {} through {}, in {}",
                    s.start_date(),
                    last,
                    s.zone()
                )
            }
        }
    }
}
fn source_label(input: &EvaluationInput, id: SourceId) -> String {
    input
        .sources
        .iter()
        .find(|s| s.id == id)
        .expect("validated source")
        .label
        .clone()
}
fn fact(item: &mut Item, label: &str, value: impl Into<String>) {
    item.facts.push(Detail {
        label: label.into(),
        value: value.into(),
    });
}
fn source_state(item: &mut Item, id: SourceId, output: &EvaluationOutput) {
    let health = output
        .source_health
        .iter()
        .find(|s| s.source_id == id)
        .expect("complete source health")
        .health;
    item.conditional |= health != Health::Healthy;
    fact(item, "Source freshness", name(&health).replace('_', " "));
}
fn item(
    id: String,
    species: &'static str,
    title: String,
    source: String,
    ownership: &str,
    phase: String,
) -> Item {
    Item {
        id,
        species,
        title,
        source,
        ownership: ownership.into(),
        phase,
        start: None,
        end: None,
        when: "Unscheduled".into(),
        milestone: false,
        risk: None,
        conditional: false,
        facts: vec![],
        reasons: vec![],
    }
}
fn timed(item: &mut Item, span: &TemporalSpan, zone: ZoneId) -> Result<(), TimeError> {
    let resolved = span.resolve(&TimezoneRules::bundled())?;
    item.start = Some(resolved.start().epoch_ms());
    item.end = Some(resolved.end().epoch_ms());
    item.when = span_label(span, zone);
    Ok(())
}
fn ownership(provenance: &Provenance) -> &'static str {
    match provenance {
        Provenance::LocalUser(_) => "Local fact",
        Provenance::Imported(_) => "Imported fact",
    }
}
fn work_details(item: &mut Item, work: &WorkMetadata) {
    fact(
        item,
        "Remaining effort",
        match work.effort {
            Effort::Unknown => "Not estimated".into(),
            Effort::Estimate(m) => format!("{} estimated", duration(u64::from(m) * 60_000)),
            Effort::AtLeast(m) => format!("At least {}", duration(u64::from(m.get()) * 60_000)),
        },
    );
    fact(
        item,
        "Useful session",
        work.minimum_chunk_minutes
            .map_or("Not declared".into(), |m| {
                duration(u64::from(m.get()) * 60_000)
            }),
    );
    fact(
        item,
        "Context",
        if work.required_contexts.is_empty() {
            "No restriction".into()
        } else {
            work.required_contexts
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", ")
        },
    );
    fact(item, "Energy", name(&work.energy_requirement));
}
fn add_fit(item: &mut Item, target: WorkTarget, output: &EvaluationOutput, zone: ZoneId) {
    for row in output.fits.iter().filter(|r| r.target == target) {
        let label = match row.status {
            FitStatus::Fits => "Fits declared window",
            FitStatus::DoesNotFit => "Does not fit",
            FitStatus::Unknown => "Fit unknown",
        };
        if let Some(span) = &row.span {
            fact(
                item,
                label,
                format!(
                    "{} ({} span)",
                    at(span.start(), zone),
                    duration(span.duration_ms())
                ),
            );
        }
        for reason in &row.reasons {
            item.reasons.push(explain(reason, zone));
        }
    }
}
fn add_pressure(item: &mut Item, row: &PressureResult, input: &EvaluationInput) {
    item.risk = Some(row.risk);
    item.conditional |= row.qualification == Qualification::Conditional;
    fact(item, "Deadline state", name(&row.resolution));
    if let Some(opportunity) = &row.opportunity {
        let value = match opportunity {
            Opportunity::Known { milliseconds, .. } => duration(*milliseconds),
            Opportunity::Unknown {
                known_qualifying_ms,
                ..
            } => format!("Unknown ({} known)", duration(*known_qualifying_ms)),
        };
        fact(item, "Before-cutoff opportunity", value);
    }
    if let Some(ratio) = &row.ratio {
        fact(
            item,
            "Pressure basis",
            format!(
                "{} work / {} opportunity",
                duration(ratio.work_ms),
                duration(ratio.opportunity_ms.get())
            ),
        );
    }
    for qualification in &row.source_qualifications {
        fact(
            item,
            &source_label(input, qualification.source_id),
            format!(
                "{}; {} coverage {}",
                name(&qualification.health),
                name(&qualification.role),
                if qualification.covered {
                    "sufficient"
                } else {
                    "incomplete"
                }
            ),
        );
    }
    item.reasons.extend(
        row.reasons
            .iter()
            .map(|r| explain(r, input.evaluation.display_zone)),
    );
}
fn explain(reason: &Reason, zone: ZoneId) -> String {
    match reason {
        Reason::DeclaredAvailability { payload, .. } => format!("Declared opportunity: {} – {}.", at(payload.span.start(), zone), date(payload.span.end(), zone, "%-I:%M %p")),
        Reason::AnchorBlocked { payload, .. } => format!("A recorded anchor occupies {} within the declared span; overlapping anchors are subtracted as a union.", duration(payload.blocked_ms)),
        Reason::ChunkTooShort { payload, .. } => format!("{} is shorter than the declared {} useful session.", duration(payload.available_ms), duration(payload.chunk_ms)),
        Reason::PressureRatio { payload, .. } => format!("{} estimated work against {} qualifying opportunity before the real cutoff.", duration(payload.work_ms), duration(payload.opportunity_ms)),
        Reason::DeadlinePhase { payload, .. } => format!("The real deadline is {} at this evaluation time.", name(&payload.phase).replace('_', " ")),
        Reason::SourceHealth { payload, .. } => format!("Source freshness is {}. Last-known records retain their identity.", name(&payload.health).replace('_', " ")),
        Reason::SuggestionExpired { payload, .. } => format!("This advice expired at {}. It creates no obligation.", at(payload.boundary, zone)),
        Reason::SoftPreferencePassed { .. } => "The preferred time passed. This intention is still optional.".into(),
        Reason::EffortLowerBound { payload, .. } => format!("The estimate is a lower bound of {}. The calculation cannot certify enough time.", duration(u64::from(payload.minutes) * 60_000)),
        _ => match reason.code().as_str() {
            "individual_capacity_only" => "This assesses one item. Windows are not reserved or allocated between competing work.",
            "source_coverage" => "The source coverage needed for this result is checked separately from refresh freshness.",
            "conservative_anchor" => "Tentative or unknown occupancy blocks time conservatively.",
            "anchor_conflict" => "These are distinct overlapping fixed anchors. Neither is moved or discarded.",
            "zero_opportunity" => "There is no qualifying declared time in this assessment span.",
            "availability_unknown" => "No availability was declared. An empty calendar is not evidence of free time.",
            "effort_unknown" => "Remaining effort is unknown; a pressure ratio cannot be established.",
            "chunk_unknown" => "A minimum useful session has not been declared; fit remains unknown.",
            "context_mismatch" => "The required context is not available in this window.",
            "context_unknown" => "This window's context is unknown.",
            "energy_mismatch" => "The declared energy capacity does not meet this work's requirement.",
            "energy_unknown" => "The energy needed for this work has no matching capacity declaration.",
            "before_earliest_start" => "The assessment respects the work's declared earliest start.",
            "zero_work_unresolved" => "No remaining work is known, but the real obligation is still unresolved.",
            "work_unavailable" => "The target's current work state is unavailable or ineligible.",
            "fulfillment_unknown" => "Authoritative fulfillment is unknown; completion is not inferred.",
            "task_completion_basis" => "Task completion comes from Trace's reported state.",
            "resolution_recorded" => "An authoritative resolution was explicitly recorded.",
            "outside_evaluation_range" => "The full deadline assessment is outside this snapshot's evaluation range.",
            "routine_past_unrecorded" => "An unrecorded past routine occurrence creates no debt.",
            "suggestion_invalidated" => "The source or settings basis changed; this historical advice is no longer current.",
            _ => "This result is derived from the current declared inputs.",
        }.into(),
    }
}

pub fn project(
    scenario: Scenario,
    input: &EvaluationInput,
    output: &EvaluationOutput,
    offset_minutes: u32,
) -> Result<Snapshot, TimeError> {
    let zone = input.evaluation.display_zone;
    let mut items = Vec::new();
    for anchor in &input.anchors {
        let state = output
            .anchor_states
            .iter()
            .find(|s| s.id == anchor.meta.id)
            .expect("complete output");
        let mut row = item(
            anchor.meta.id.to_string(),
            "anchor",
            anchor.title.clone(),
            source_label(input, anchor.provenance.source_id()),
            ownership(&anchor.provenance),
            name(&state.phase),
        );
        timed(&mut row, &anchor.span, zone)?;
        source_state(&mut row, anchor.provenance.source_id(), output);
        fact(&mut row, "Rigidity", name(&anchor.rigidity));
        fact(&mut row, "Occupancy", name(&anchor.occupancy));
        fact(
            &mut row,
            "Reported certainty",
            name(&anchor.reported_certainty),
        );
        if let Some(annotation) = input
            .anchor_annotations
            .iter()
            .find(|a| a.target_id == anchor.meta.id)
        {
            row.milestone = annotation.milestone;
            if let Some(confirmation) = &annotation.confirmation {
                fact(
                    &mut row,
                    "User confirmation",
                    if confirmation.is_current(anchor.meta.revision) {
                        "Current for this source revision"
                    } else {
                        "Earlier source revision"
                    },
                );
            }
        }
        row.reasons.push(
            "A recorded occurrence. Time passing does not record attendance or change its source."
                .into(),
        );
        for conflict in output
            .conflicts
            .iter()
            .filter(|c| c.anchor_ids.contains(&anchor.meta.id))
        {
            fact(
                &mut row,
                "Overlapping anchor",
                format!(
                    "{} ({} overlap)",
                    at(conflict.intersection.start(), zone),
                    duration(conflict.intersection.duration_ms())
                ),
            );
            row.reasons.push("The overlapping anchors remain distinct facts. Their occupied time is subtracted once.".into());
        }
        items.push(row);
    }
    for deadline in &input.deadlines {
        let state = output
            .deadline_states
            .iter()
            .find(|s| s.id == deadline.meta.id)
            .expect("complete output");
        let mut row = item(
            deadline.meta.id.to_string(),
            "deadline",
            deadline.title.clone(),
            source_label(input, deadline.provenance.source_id()),
            ownership(&deadline.provenance),
            name(&state.phase),
        );
        row.start = Some(state.endpoint.epoch_ms());
        source_state(&mut row, deadline.provenance.source_id(), output);
        row.when = match &deadline.cutoff {
            Cutoff::At { instant, .. } => at(*instant, zone),
            Cutoff::OnDate { date, zone } => format!("Due on {date}, whole date in {zone}"),
        };
        if let Some(annotation) = input
            .deadline_annotations
            .iter()
            .find(|a| a.target_id == deadline.meta.id)
        {
            row.milestone = annotation.milestone;
        }
        match temporal_core::fulfillment::deadline_work(input, deadline) {
            DeadlineWork::Standalone(work) => {
                work_details(&mut row, &work);
                add_fit(
                    &mut row,
                    WorkTarget::Deadline(deadline.meta.id),
                    output,
                    zone,
                );
            }
            DeadlineWork::Task(id) => {
                fact(
                    &mut row,
                    "Linked Trace task",
                    input
                        .task_refs
                        .iter()
                        .find(|t| t.meta.id == id)
                        .expect("validated task")
                        .title
                        .clone(),
                );
            }
            DeadlineWork::Unspecified => fact(&mut row, "Associated work", "Not declared"),
        }
        if let Some(pressure) = output
            .pressures
            .iter()
            .find(|p| p.deadline_id == deadline.meta.id)
        {
            add_pressure(&mut row, pressure, input);
        }
        items.push(row);
    }
    for task in &input.task_refs {
        let mut row = item(
            task.meta.id.to_string(),
            "task",
            task.title.clone(),
            source_label(input, task.provenance.source_id()),
            "Imported task",
            name(&task.status),
        );
        fact(&mut row, "Trace status", name(&task.status));
        source_state(&mut row, task.provenance.source_id(), output);
        fact(
            &mut row,
            "Due",
            match &task.due {
                TaskDue::None => "No deadline".into(),
                TaskDue::Unresolved { value, reason } => format!("Unresolved: {value} ({reason})"),
                TaskDue::Deadline(id) => input
                    .deadlines
                    .iter()
                    .find(|d| d.meta.id == *id)
                    .map(|d| format!("Separate deadline: {}", d.title))
                    .expect("validated due link"),
            },
        );
        let work = input
            .task_annotations
            .iter()
            .find(|a| a.target_id == task.meta.id)
            .map(|a| a.work.clone())
            .unwrap_or_default();
        work_details(&mut row, &work);
        add_fit(&mut row, WorkTarget::Task(task.meta.id), output, zone);
        row.reasons.push("Trace owns task state and completion. Its Now/Later/Someday status does not assign a time here.".into());
        items.push(row);
    }
    for intention in &input.intentions {
        let state = output
            .intention_states
            .iter()
            .find(|s| s.id == intention.meta.id)
            .expect("complete output");
        let mut row = item(
            intention.meta.id.to_string(),
            "intention",
            intention.title.clone(),
            source_label(input, intention.provenance.source_id()),
            "User preference",
            name(&state.phase),
        );
        if let Some(span) = &intention.preferred_span {
            row.when = span_label(span, zone);
        }
        work_details(&mut row, &intention.work.clone().unwrap_or_default());
        add_fit(
            &mut row,
            WorkTarget::Intention(intention.meta.id),
            output,
            zone,
        );
        row.reasons
            .push("This is a soft preference, not a commitment or a deadline.".into());
        row.reasons
            .extend(state.reasons.iter().map(|r| explain(r, zone)));
        items.push(row);
    }
    for occurrence in &output.routine_occurrences {
        let routine = input
            .routines
            .iter()
            .find(|r| r.meta.id == occurrence.key.routine_id)
            .expect("validated routine");
        let mut row = item(
            format!("{}:{}", routine.meta.id, occurrence.key.date),
            "routine",
            routine.title.clone(),
            source_label(input, routine.provenance.source_id()),
            "User preference",
            name(&occurrence.phase),
        );
        row.when = format!("{} preferred day", occurrence.key.date);
        work_details(&mut row, &routine.work);
        add_fit(
            &mut row,
            WorkTarget::RoutineOccurrence(occurrence.key),
            output,
            zone,
        );
        row.reasons.push("A flexible weekly occurrence. Unrecorded earlier occurrences do not accumulate as debt.".into());
        items.push(row);
    }
    for window in &output.windows {
        let mut row = item(
            format!("window:{}:{}", window.declaration_ref, window.span.start()),
            "window",
            duration(window.span.duration_ms()),
            "Declared availability".into(),
            "Derived opportunity",
            "potential".into(),
        );
        timed(&mut row, &TemporalSpan::Timed(window.span.clone()), zone)?;
        row.conditional = window
            .source_qualifications
            .iter()
            .any(|q| q.health != Health::Healthy || !q.covered);
        fact(
            &mut row,
            "Context",
            match &window.contexts {
                DeclaredContexts::Unknown => "Unknown".into(),
                DeclaredContexts::Known(tags) => tags
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
            },
        );
        fact(&mut row, "Energy capacity", name(&window.energy_capacity));
        for q in &window.source_qualifications {
            fact(
                &mut row,
                &source_label(input, q.source_id),
                format!(
                    "{}; coverage {}",
                    name(&q.health),
                    if q.covered {
                        "sufficient"
                    } else {
                        "incomplete"
                    }
                ),
            );
        }
        row.reasons
            .extend(window.reasons.iter().map(|r| explain(r, zone)));
        row.reasons.push("Potential opportunity within an explicit declaration. Compatibility is assessed per work item; this is not guaranteed free time.".into());
        items.push(row);
    }
    for (index, suggestion) in input.prior_suggestions.iter().enumerate() {
        let validity = output
            .suggestion_validity
            .iter()
            .find(|s| s.key == suggestion.key)
            .expect("complete output");
        let target_title = match suggestion.target {
            WorkTarget::Task(id) => input
                .task_refs
                .iter()
                .find(|t| t.meta.id == id)
                .map(|t| t.title.as_str()),
            WorkTarget::Deadline(id) => input
                .deadlines
                .iter()
                .find(|d| d.meta.id == id)
                .map(|d| d.title.as_str()),
            WorkTarget::Intention(id) => input
                .intentions
                .iter()
                .find(|i| i.meta.id == id)
                .map(|i| i.title.as_str()),
            WorkTarget::RoutineOccurrence(key) => input
                .routines
                .iter()
                .find(|r| r.meta.id == key.routine_id)
                .map(|r| r.title.as_str()),
        }
        .expect("validated suggestion target");
        let mut row = item(
            format!("suggestion:{index}"),
            "suggestion",
            format!("Consider {target_title}"),
            "System inference".into(),
            "Suggestion",
            name(&validity.status),
        );
        if let Some(span) = &suggestion.proposed_span {
            row.when = format!("Proposed {}", span_label(span, zone));
        }
        fact(&mut row, "Valid until", at(suggestion.valid_until, zone));
        fact(&mut row, "Current validity", name(&validity.status));
        row.reasons.push("Advice is optional. Its proposed time never creates a deadline or marks work complete.".into());
        row.reasons
            .extend(validity.reasons.iter().map(|r| explain(r, zone)));
        items.push(row);
    }
    for row in &mut items {
        let mut seen = std::collections::BTreeSet::new();
        row.reasons.retain(|r| seen.insert(r.clone()));
    }
    let now = input.evaluation.now;
    let end = input.evaluation.evaluation_end;
    let mut ticks = Vec::new();
    for hours in [0, 1, 3, 6, 24, 72, 168] {
        let tick = now.checked_add_ms(hours * 3_600_000)?;
        if tick < end {
            ticks.push(Tick {
                at: tick.epoch_ms(),
                label: if hours == 0 {
                    "Now".into()
                } else if hours < 24 {
                    date(tick, zone, "%-I:%M %p")
                } else {
                    date(tick, zone, "%a %-d")
                },
            });
        }
    }
    ticks.push(Tick {
        at: end.epoch_ms(),
        label: date(end, zone, "%a %-d"),
    });
    let sources = output
        .source_health
        .iter()
        .map(|s| SourceSummary {
            label: source_label(input, s.source_id),
            health: s.health,
        })
        .collect();
    Ok(Snapshot {
        scenario,
        now: now.epoch_ms(),
        end: end.epoch_ms(),
        offset_minutes,
        max_offset_minutes: (end.duration_since(input.captured_now)? / 60_000 - 1) as u32,
        zone: zone.to_string(),
        date_label: date(now, zone, "%A, %B %-d"),
        clock_label: date(now, zone, "%-I:%M %p"),
        ticks,
        items,
        sources,
        has_declarations: !input.availability.is_empty(),
    })
}
