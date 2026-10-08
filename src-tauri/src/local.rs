//! App-owned temporal records and the narrow mutation boundary for local input.
use crate::series::{
    self, AnchorSeries, EventDetails, SeriesFrequency, SeriesRule, SeriesTiming, UsualAvailability,
    WeeklyBlock,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    num::{NonZeroU32, NonZeroU64},
};
use temporal_core::{domain::*, time::*};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalState {
    pub source: Source,
    pub revision: NonZeroU64,
    #[serde(default)]
    pub updated_at: Option<Instant>,
    pub anchors: Vec<Anchor>,
    pub deadlines: Vec<Deadline>,
    pub intentions: Vec<Intention>,
    pub routines: Vec<Routine>,
    pub routine_outcomes: Vec<RoutineOutcome>,
    pub availability: Vec<AvailabilityDeclaration>,
    pub anchor_annotations: Vec<AnchorAnnotation>,
    pub deadline_annotations: Vec<DeadlineAnnotation>,
    pub task_annotations: Vec<TaskAnnotation>,
    /// Repeating fixed events; each occurrence evaluates as an ordinary Anchor.
    #[serde(default)]
    pub anchor_series: Vec<AnchorSeries>,
    /// The usual weekly availability, expanded into explicit declarations.
    #[serde(default)]
    pub usual_availability: Option<UsualAvailability>,
    /// Display-only place and notes for single local Anchors and Deadlines.
    #[serde(default)]
    pub event_details: BTreeMap<String, EventDetails>,
}

impl LocalState {
    pub fn empty(source_id: SourceId) -> Self {
        Self {
            source: Source {
                id: source_id,
                kind: SourceKind::Local,
                label: "My local time".into(),
            },
            revision: NonZeroU64::MIN,
            updated_at: None,
            anchors: vec![],
            deadlines: vec![],
            intentions: vec![],
            routines: vec![],
            routine_outcomes: vec![],
            availability: vec![],
            anchor_annotations: vec![],
            deadline_annotations: vec![],
            task_annotations: vec![],
            anchor_series: vec![],
            usual_availability: None,
            event_details: BTreeMap::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalAction {
    Upsert,
    Remove,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalKind {
    Anchor,
    Deadline,
    Intention,
    Routine,
    RoutineOutcome,
    Availability,
    TaskAnnotation,
    AnchorAnnotation,
    DeadlineAnnotation,
    AnchorSeries,
    SeriesSkip,
    UsualAvailability,
}

/// One usual weekly block as typed in the Settings page.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BlockInput {
    pub weekday: Weekday,
    pub start: String,
    pub end: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalCompletion {
    Unresolved,
    Satisfied,
    Cancelled,
    Done,
    Dismissed,
}

/// A deliberately small UI/IPC input. Wall-clock fields are interpreted in the
/// supplied IANA zone; the core receives only normalized values after parsing.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalMutation {
    pub action: LocalAction,
    pub kind: LocalKind,
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    /// Timed values are `YYYY-MM-DDTHH:MM` or the normalized 23-character form.
    /// All-day anchors/intentions use `start_date` and `end_date_exclusive`.
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub end: Option<String>,
    #[serde(default)]
    pub due: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub end_date_exclusive: Option<String>,
    #[serde(default)]
    pub until_date_exclusive: Option<String>,
    #[serde(default)]
    pub zone: String,
    #[serde(default)]
    pub all_day: bool,
    #[serde(default)]
    pub effort_minutes: Option<u32>,
    #[serde(default)]
    pub minimum_chunk_minutes: Option<u32>,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub weekdays: Vec<Weekday>,
    #[serde(default)]
    pub trace_external_id: Option<String>,
    #[serde(default)]
    pub importance: Option<Importance>,
    #[serde(default)]
    pub milestone: Option<bool>,
    #[serde(default)]
    pub occupancy: Option<Occupancy>,
    #[serde(default)]
    pub certainty: Option<ReportedCertainty>,
    /// Replace the full work annotation, including deliberate unknown/empty fields.
    #[serde(default)]
    pub replace_work: bool,
    #[serde(default)]
    pub clear_preference: bool,
    #[serde(default)]
    pub clear_work: bool,
    #[serde(default)]
    pub paused: Option<bool>,
    #[serde(default)]
    pub completion: Option<LocalCompletion>,
    #[serde(default)]
    pub occurrence_date: Option<String>,
    #[serde(default)]
    pub outcome: Option<Outcome>,
    #[serde(default)]
    pub energy_requirement: Option<EnergyRequirement>,
    #[serde(default)]
    pub energy_capacity: Option<EnergyCapacity>,
    /// Series rule fields. Supplying `frequency` replaces the whole rule.
    #[serde(default)]
    pub frequency: Option<SeriesFrequency>,
    #[serde(default)]
    pub interval: Option<u32>,
    #[serde(default)]
    pub count: Option<u32>,
    /// Display-only free text; an empty string clears it.
    #[serde(default)]
    pub place: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub blocks: Vec<BlockInput>,
}

fn default_zone() -> String {
    "America/Toronto".into()
}

impl LocalMutation {
    pub fn upsert(kind: LocalKind) -> Self {
        Self {
            action: LocalAction::Upsert,
            kind,
            id: None,
            title: None,
            start: None,
            end: None,
            due: None,
            start_date: None,
            end_date_exclusive: None,
            until_date_exclusive: None,
            zone: default_zone(),
            all_day: false,
            effort_minutes: None,
            minimum_chunk_minutes: None,
            context: None,
            weekdays: vec![],
            trace_external_id: None,
            importance: None,
            milestone: None,
            occupancy: None,
            certainty: None,
            replace_work: false,
            clear_preference: false,
            clear_work: false,
            paused: None,
            completion: None,
            occurrence_date: None,
            outcome: None,
            energy_requirement: None,
            energy_capacity: None,
            frequency: None,
            interval: None,
            count: None,
            place: None,
            notes: None,
            blocks: vec![],
        }
    }
}

pub(crate) fn apply(
    state: &mut LocalState,
    mutation: &LocalMutation,
    trace_tasks: &[(String, TaskRefId)],
    now: Instant,
) -> Result<(), String> {
    if mutation.kind == LocalKind::TaskAnnotation
        && (mutation.title.is_some()
            || mutation.due.is_some()
            || mutation.start.is_some()
            || mutation.end.is_some())
    {
        return Err(
            "Trace owns task text, dates, and state; only scheduling annotations are editable"
                .into(),
        );
    }
    if mutation.completion.is_some()
        && !matches!(mutation.kind, LocalKind::Deadline | LocalKind::Intention)
    {
        return Err("only local Deadlines and Intentions accept completion state".into());
    }
    if mutation.milestone.is_some()
        && !matches!(
            mutation.kind,
            LocalKind::Anchor
                | LocalKind::Deadline
                | LocalKind::AnchorAnnotation
                | LocalKind::DeadlineAnnotation
        )
    {
        return Err("only Anchors and Deadlines can be milestones".into());
    }
    if mutation.clear_preference && mutation.kind != LocalKind::Intention {
        return Err("only an Intention has a removable preference".into());
    }
    if mutation.clear_work
        && !matches!(
            mutation.kind,
            LocalKind::Deadline | LocalKind::DeadlineAnnotation
        )
    {
        return Err("only a Deadline has a removable work association".into());
    }
    if mutation.clear_work && (has_work(mutation) || mutation.trace_external_id.is_some()) {
        return Err("clear work cannot also supply a work association".into());
    }
    if mutation.paused.is_some() && mutation.kind != LocalKind::Routine {
        return Err("only a Routine can be paused".into());
    }
    if mutation.outcome.is_some() && mutation.kind != LocalKind::RoutineOutcome {
        return Err("an outcome needs a Routine occurrence".into());
    }
    if mutation.occurrence_date.is_some()
        && !matches!(
            mutation.kind,
            LocalKind::RoutineOutcome | LocalKind::SeriesSkip
        )
    {
        return Err("an occurrence date needs a Routine or repeating event".into());
    }
    if (mutation.occupancy.is_some() || mutation.certainty.is_some())
        && !matches!(mutation.kind, LocalKind::Anchor | LocalKind::AnchorSeries)
    {
        return Err("occupancy and certainty apply only to an Anchor".into());
    }
    if (mutation.frequency.is_some() || mutation.interval.is_some() || mutation.count.is_some())
        && mutation.kind != LocalKind::AnchorSeries
    {
        return Err("only a repeating event has a repeat rule".into());
    }
    if (mutation.place.is_some() || mutation.notes.is_some())
        && !matches!(
            mutation.kind,
            LocalKind::Anchor | LocalKind::Deadline | LocalKind::AnchorSeries
        )
    {
        return Err("place and notes describe events and deadlines".into());
    }
    if !mutation.blocks.is_empty() && mutation.kind != LocalKind::UsualAvailability {
        return Err("weekly blocks describe usual availability".into());
    }
    // Supplying an ID edits that existing species. New IDs are allocated here.
    if let Some(id) = &mutation.id {
        let exists = match mutation.kind {
            LocalKind::Anchor => state.anchors.iter().any(|r| r.meta.id.to_string() == *id),
            LocalKind::Deadline => state.deadlines.iter().any(|r| r.meta.id.to_string() == *id),
            LocalKind::Intention => state
                .intentions
                .iter()
                .any(|r| r.meta.id.to_string() == *id),
            LocalKind::Routine => state.routines.iter().any(|r| r.meta.id.to_string() == *id),
            LocalKind::Availability => state
                .availability
                .iter()
                .any(|r| r.meta.id.to_string() == *id),
            LocalKind::AnchorSeries | LocalKind::SeriesSkip => state
                .anchor_series
                .iter()
                .any(|r| r.meta.id.to_string() == *id),
            _ => true,
        };
        if !exists {
            return Err("local record does not exist for this type".into());
        }
    }
    match mutation.action {
        LocalAction::Upsert => upsert(state, mutation, trace_tasks, now),
        LocalAction::Remove => remove(state, mutation, trace_tasks, now),
    }
}

fn upsert(
    state: &mut LocalState,
    mutation: &LocalMutation,
    trace_tasks: &[(String, TaskRefId)],
    now: Instant,
) -> Result<(), String> {
    match mutation.kind {
        LocalKind::Anchor => upsert_anchor(state, mutation, now),
        LocalKind::Deadline => upsert_deadline(state, mutation, trace_tasks, now),
        LocalKind::Intention => upsert_intention(state, mutation, now),
        LocalKind::Routine => upsert_routine(state, mutation, now),
        LocalKind::RoutineOutcome => upsert_routine_outcome(state, mutation, now),
        LocalKind::Availability => upsert_availability(state, mutation, now),
        LocalKind::TaskAnnotation => upsert_task_annotation(state, mutation, trace_tasks, now),
        LocalKind::AnchorAnnotation => upsert_anchor_annotation(state, mutation, now),
        LocalKind::DeadlineAnnotation => {
            upsert_deadline_annotation(state, mutation, trace_tasks, now)
        }
        LocalKind::AnchorSeries => upsert_series(state, mutation, now),
        LocalKind::SeriesSkip => skip_occurrence(state, mutation, now, true),
        LocalKind::UsualAvailability => upsert_usual(state, mutation, now),
    }
}

fn remove(
    state: &mut LocalState,
    mutation: &LocalMutation,
    trace_tasks: &[(String, TaskRefId)],
    now: Instant,
) -> Result<(), String> {
    match mutation.kind {
        LocalKind::Anchor => {
            let id = parse_id::<AnchorId>(mutation.id.as_deref())?;
            let source_id = state.source.id;
            let record = find_mut(&mut state.anchors, id, "anchor")?;
            record.presence = Presence::Removed;
            touch(&mut record.meta, now)?;
            record.provenance = local_provenance(source_id, now);
            Ok(())
        }
        LocalKind::Deadline => {
            let id = parse_id::<DeadlineId>(mutation.id.as_deref())?;
            let source_id = state.source.id;
            let record = find_mut(&mut state.deadlines, id, "deadline")?;
            record.presence = Presence::Removed;
            touch(&mut record.meta, now)?;
            record.provenance = local_provenance(source_id, now);
            Ok(())
        }
        LocalKind::Intention => {
            let id = parse_id::<IntentionId>(mutation.id.as_deref())?;
            let source_id = state.source.id;
            let record = find_mut(&mut state.intentions, id, "intention")?;
            record.presence = Presence::Removed;
            touch(&mut record.meta, now)?;
            record.provenance = local_user_provenance(source_id, now);
            Ok(())
        }
        LocalKind::Routine => {
            let id = parse_id::<RoutineId>(mutation.id.as_deref())?;
            let source_id = state.source.id;
            let record = find_mut(&mut state.routines, id, "routine")?;
            record.presence = Presence::Removed;
            touch(&mut record.meta, now)?;
            record.provenance = local_user_provenance(source_id, now);
            Ok(())
        }
        LocalKind::RoutineOutcome => remove_routine_outcome(state, mutation),
        LocalKind::Availability => {
            let id = parse_id::<AvailabilityId>(mutation.id.as_deref())?;
            remove_by_id(&mut state.availability, id, "availability")
        }
        LocalKind::TaskAnnotation => {
            let target = trace_target(mutation, trace_tasks)?;
            remove_by_id(&mut state.task_annotations, target, "task annotation")
        }
        LocalKind::AnchorAnnotation => {
            let target = parse_id::<AnchorId>(mutation.id.as_deref())?;
            remove_by_id(&mut state.anchor_annotations, target, "anchor annotation")
        }
        LocalKind::DeadlineAnnotation => {
            let target = parse_id::<DeadlineId>(mutation.id.as_deref())?;
            remove_by_id(
                &mut state.deadline_annotations,
                target,
                "deadline annotation",
            )
        }
        LocalKind::AnchorSeries => {
            let id = parse_id::<AnchorId>(mutation.id.as_deref())?;
            let record = state
                .anchor_series
                .iter_mut()
                .find(|r| r.meta.id == id)
                .ok_or("repeating event does not exist")?;
            record.presence = Presence::Removed;
            touch(&mut record.meta, now)?;
            Ok(())
        }
        LocalKind::SeriesSkip => skip_occurrence(state, mutation, now, false),
        LocalKind::UsualAvailability => {
            if state.usual_availability.take().is_none() {
                return Err("usual availability is not declared".into());
            }
            Ok(())
        }
    }
}

fn remove_routine_outcome(state: &mut LocalState, input: &LocalMutation) -> Result<(), String> {
    let routine_id = parse_id::<RoutineId>(input.id.as_deref())?;
    let date = required(
        input
            .occurrence_date
            .as_deref()
            .or(input.start_date.as_deref()),
        "routine occurrence date",
    )?
    .parse::<LocalDate>()
    .map_err(time_error)?;
    let before = state.routine_outcomes.len();
    state
        .routine_outcomes
        .retain(|record| !(record.routine_id == routine_id && record.date == date));
    if state.routine_outcomes.len() == before {
        Err("routine outcome does not exist".into())
    } else {
        Ok(())
    }
}

fn upsert_anchor(
    state: &mut LocalState,
    input: &LocalMutation,
    now: Instant,
) -> Result<(), String> {
    let id = upsert_id::<AnchorId>(input.id.as_deref())?;
    let existing = state.anchors.iter().find(|record| record.meta.id == id);
    let title = match input.title.as_deref() {
        Some(value) => required(Some(value), "anchor title")?.to_string(),
        None => existing
            .map(|record| record.title.clone())
            .ok_or_else(|| "anchor title is required".to_string())?,
    };
    let has_span = input.all_day
        || input.start.is_some()
        || input.end.is_some()
        || input.start_date.is_some()
        || input.end_date_exclusive.is_some();
    let span = if has_span {
        temporal_span(input)?
    } else {
        existing
            .map(|record| record.span.clone())
            .ok_or_else(|| "anchor time is required".to_string())?
    };
    let location = if input.context.is_some() {
        context_tag(input.context.as_deref())?
    } else {
        existing.and_then(|record| record.location.clone())
    };
    let meta = next_meta(existing.map(|record| &record.meta), id, now)?;
    let record = Anchor {
        meta,
        title,
        presence: Presence::Present,
        provenance: local_provenance(state.source.id, now),
        span,
        rigidity: AnchorRigidity::Fixed,
        reported_certainty: input.certainty.unwrap_or_else(|| {
            existing.map_or(ReportedCertainty::Confirmed, |r| r.reported_certainty)
        }),
        occupancy: input
            .occupancy
            .or_else(|| existing.map(|r| r.occupancy))
            .ok_or_else(|| "declare whether this Anchor blocks availability".to_string())?,
        location,
    };
    replace_or_push(&mut state.anchors, record);
    update_details(state, &id.to_string(), input)?;
    if input.milestone.is_some() || input.importance.is_some() {
        let mut annotation_input = input.clone();
        annotation_input.id = Some(id.to_string());
        upsert_anchor_annotation(state, &annotation_input, now)?;
    }
    Ok(())
}

fn upsert_deadline(
    state: &mut LocalState,
    input: &LocalMutation,
    trace_tasks: &[(String, TaskRefId)],
    now: Instant,
) -> Result<(), String> {
    let id = upsert_id::<DeadlineId>(input.id.as_deref())?;
    let existing = state.deadlines.iter().find(|record| record.meta.id == id);
    let title = match input.title.as_deref() {
        Some(value) => required(Some(value), "deadline title")?.to_string(),
        None => existing
            .map(|record| record.title.clone())
            .ok_or_else(|| "deadline title is required".to_string())?,
    };
    let has_cutoff =
        input.all_day || input.due.is_some() || input.start.is_some() || input.start_date.is_some();
    let cutoff = if has_cutoff {
        cutoff(input)?
    } else {
        existing
            .map(|record| record.cutoff.clone())
            .ok_or_else(|| "deadline time is required".to_string())?
    };
    let fulfillment = local_deadline_fulfillment(existing, input.completion, now)?;
    let record = Deadline {
        meta: next_meta(existing.map(|record| &record.meta), id, now)?,
        title,
        presence: Presence::Present,
        provenance: local_provenance(state.source.id, now),
        cutoff,
        fulfillment,
    };
    replace_or_push(&mut state.deadlines, record);
    update_details(state, &id.to_string(), input)?;
    if input.milestone.is_some()
        || input.importance.is_some()
        || has_work(input)
        || input.trace_external_id.is_some()
        || input.clear_work
    {
        let mut annotation_input = input.clone();
        annotation_input.id = Some(id.to_string());
        upsert_deadline_annotation(state, &annotation_input, trace_tasks, now)?;
    }
    Ok(())
}

fn local_deadline_fulfillment(
    existing: Option<&Deadline>,
    completion: Option<LocalCompletion>,
    now: Instant,
) -> Result<Fulfillment, String> {
    match completion {
        None => Ok(existing
            .map(|record| record.fulfillment.clone())
            .unwrap_or(Fulfillment::Recorded(RecordedResolution::Unresolved))),
        Some(LocalCompletion::Unresolved) => {
            Ok(Fulfillment::Recorded(RecordedResolution::Unresolved))
        }
        Some(LocalCompletion::Satisfied) => Ok(Fulfillment::Recorded(
            RecordedResolution::Satisfied(ResolutionEvidence {
                authority: EvidenceAuthority::LocalUser,
                recorded_at: now,
                effective_at: None,
            }),
        )),
        Some(LocalCompletion::Cancelled) => Ok(Fulfillment::Recorded(
            RecordedResolution::Cancelled(ResolutionEvidence {
                authority: EvidenceAuthority::LocalUser,
                recorded_at: now,
                effective_at: None,
            }),
        )),
        Some(LocalCompletion::Done | LocalCompletion::Dismissed) => {
            Err("deadline completion must be unresolved, satisfied, or cancelled".into())
        }
    }
}

fn upsert_intention(
    state: &mut LocalState,
    input: &LocalMutation,
    now: Instant,
) -> Result<(), String> {
    let id = upsert_id::<IntentionId>(input.id.as_deref())?;
    let existing = state.intentions.iter().find(|record| record.meta.id == id);
    let title = match input.title.as_deref() {
        Some(value) => required(Some(value), "intention title")?.to_string(),
        None => existing
            .map(|record| record.title.clone())
            .ok_or_else(|| "intention title is required".to_string())?,
    };
    let has_span = input.all_day
        || input.start.is_some()
        || input.end.is_some()
        || input.start_date.is_some()
        || input.end_date_exclusive.is_some();
    let preferred_span = if input.clear_preference {
        if has_span {
            return Err("clear preference cannot also supply a time".into());
        }
        None
    } else if has_span {
        Some(temporal_span(input)?)
    } else {
        existing.and_then(|record| record.preferred_span.clone())
    };
    let work = if has_work(input) {
        Some(updated_work(input, existing.and_then(|r| r.work.as_ref()))?)
    } else {
        existing.and_then(|record| record.work.clone())
    };
    let (state_value, state_evidence) = local_intention_state(existing, input.completion, now)?;
    let record = Intention {
        meta: next_meta(existing.map(|record| &record.meta), id, now)?,
        title,
        presence: Presence::Present,
        provenance: local_user_provenance(state.source.id, now),
        preferred_span,
        work,
        state: state_value,
        state_evidence,
    };
    replace_or_push(&mut state.intentions, record);
    Ok(())
}

fn local_intention_state(
    existing: Option<&Intention>,
    completion: Option<LocalCompletion>,
    now: Instant,
) -> Result<(IntentionState, Option<UserStateEvidence>), String> {
    match completion {
        None => Ok(existing.map_or((IntentionState::Active, None), |record| {
            (record.state, record.state_evidence.clone())
        })),
        Some(LocalCompletion::Unresolved) => Ok((IntentionState::Active, None)),
        Some(LocalCompletion::Done) => Ok((
            IntentionState::Done,
            Some(UserStateEvidence {
                recorded_at: now,
                effective_at: None,
            }),
        )),
        Some(LocalCompletion::Dismissed) => Ok((
            IntentionState::Dismissed,
            Some(UserStateEvidence {
                recorded_at: now,
                effective_at: None,
            }),
        )),
        Some(LocalCompletion::Satisfied | LocalCompletion::Cancelled) => {
            Err("intention completion must be active, done, or dismissed".into())
        }
    }
}

fn upsert_routine(
    state: &mut LocalState,
    input: &LocalMutation,
    now: Instant,
) -> Result<(), String> {
    let id = upsert_id::<RoutineId>(input.id.as_deref())?;
    let existing = state.routines.iter().find(|record| record.meta.id == id);
    let title = match input.title.as_deref() {
        Some(value) => required(Some(value), "routine title")?.to_string(),
        None => existing
            .map(|record| record.title.clone())
            .ok_or_else(|| "routine title is required".to_string())?,
    };
    let has_rule = !input.weekdays.is_empty()
        || input.start_date.is_some()
        || input.until_date_exclusive.is_some();
    let rule = if has_rule || existing.is_none() {
        if input.weekdays.is_empty() {
            return Err("a routine needs at least one weekday".into());
        }
        if input.weekdays.iter().collect::<BTreeSet<_>>().len() != input.weekdays.len() {
            return Err("routine weekdays must be distinct".into());
        }
        let start_date = required(input.start_date.as_deref(), "routine start date")?
            .parse::<LocalDate>()
            .map_err(time_error)?;
        let until_date_exclusive = input
            .until_date_exclusive
            .as_deref()
            .map(str::parse::<LocalDate>)
            .transpose()
            .map_err(time_error)?;
        if until_date_exclusive.is_some_and(|until| until <= start_date) {
            return Err("routine end date must be after its start date".into());
        }
        RoutineRule::Weekly(WeeklyRule {
            weekdays: input.weekdays.iter().copied().collect(),
            start_date,
            until_date_exclusive,
            zone: parse_zone(&input.zone)?,
        })
    } else {
        existing
            .map(|record| record.rule.clone())
            .ok_or_else(|| "routine rule is required".to_string())?
    };
    let routine_state = match input.paused {
        Some(true) => RoutineState::Paused,
        Some(false) => RoutineState::Active,
        None => existing.map_or(RoutineState::Active, |record| record.state),
    };
    let routine_work = if has_work(input) {
        updated_work(input, existing.map(|r| &r.work))?
    } else {
        existing.map_or_else(|| work(input), |record| Ok(record.work.clone()))?
    };
    let record = Routine {
        meta: next_meta(existing.map(|record| &record.meta), id, now)?,
        title,
        presence: Presence::Present,
        provenance: local_user_provenance(state.source.id, now),
        state: routine_state,
        work: routine_work,
        rule,
    };
    replace_or_push(&mut state.routines, record);
    Ok(())
}

fn upsert_routine_outcome(
    state: &mut LocalState,
    input: &LocalMutation,
    now: Instant,
) -> Result<(), String> {
    let routine_id = parse_id::<RoutineId>(input.id.as_deref())?;
    let routine = state
        .routines
        .iter()
        .find(|record| record.meta.id == routine_id)
        .ok_or_else(|| "routine does not exist".to_string())?;
    if routine.presence != Presence::Present {
        return Err("a removed routine cannot receive an outcome".into());
    }
    let date = required(
        input
            .occurrence_date
            .as_deref()
            .or(input.start_date.as_deref()),
        "routine occurrence date",
    )?
    .parse::<LocalDate>()
    .map_err(time_error)?;
    let outcome = input
        .outcome
        .ok_or_else(|| "routine outcome is required".to_string())?;
    let mut active = routine.clone();
    active.state = RoutineState::Active;
    let occurrence = temporal_core::recurrence::occurrence_state(
        &active,
        &[],
        date,
        now,
        &TimezoneRules::bundled(),
    )
    .map_err(time_error)?;
    if occurrence.phase == temporal_core::results::OccurrencePhase::Inactive
        && !state
            .routine_outcomes
            .iter()
            .any(|r| r.routine_id == routine_id && r.date == date)
    {
        return Err("this date is not selected by the Routine rule".into());
    }
    if let Some(existing) = state
        .routine_outcomes
        .iter_mut()
        .find(|record| record.routine_id == routine_id && record.date == date)
    {
        existing.revision = increment(existing.revision)?;
        existing.outcome = outcome;
        existing.recorded_at = now;
    } else {
        state.routine_outcomes.push(RoutineOutcome {
            routine_id,
            date,
            outcome,
            revision: NonZeroU64::MIN,
            recorded_at: now,
        });
    }
    Ok(())
}

fn upsert_availability(
    state: &mut LocalState,
    input: &LocalMutation,
    now: Instant,
) -> Result<(), String> {
    let span = timed_span(input)?;
    let id = upsert_id::<AvailabilityId>(input.id.as_deref())?;
    let record = AvailabilityDeclaration {
        meta: next_meta(
            state
                .availability
                .iter()
                .find(|record| record.meta.id == id)
                .map(|record| &record.meta),
            id,
            now,
        )?,
        span,
        contexts: declared_contexts(input.context.as_deref())?,
        energy_capacity: input.energy_capacity.unwrap_or(EnergyCapacity::Unknown),
    };
    replace_or_push(&mut state.availability, record);
    Ok(())
}

fn upsert_task_annotation(
    state: &mut LocalState,
    input: &LocalMutation,
    trace_tasks: &[(String, TaskRefId)],
    now: Instant,
) -> Result<(), String> {
    let target_id = trace_target(input, trace_tasks)?;
    let existing = state
        .task_annotations
        .iter()
        .find(|record| record.target_id == target_id);
    let revision = existing.map_or(Ok(NonZeroU64::MIN), |record| increment(record.revision))?;
    let record = TaskAnnotation {
        target_id,
        revision,
        created_at: existing.map_or(now, |record| record.created_at),
        updated_at: now,
        importance: input
            .importance
            .unwrap_or_else(|| existing.map_or(Importance::Unspecified, |r| r.importance)),
        work: updated_work(input, existing.map(|r| &r.work))?,
    };
    replace_or_push(&mut state.task_annotations, record);
    Ok(())
}

fn upsert_anchor_annotation(
    state: &mut LocalState,
    input: &LocalMutation,
    now: Instant,
) -> Result<(), String> {
    let target_id = parse_id::<AnchorId>(input.id.as_deref())?;
    let existing = state
        .anchor_annotations
        .iter()
        .find(|record| record.target_id == target_id);
    let revision = existing.map_or(Ok(NonZeroU64::MIN), |record| increment(record.revision))?;
    let record = AnchorAnnotation {
        target_id,
        revision,
        created_at: existing.map_or(now, |record| record.created_at),
        updated_at: now,
        importance: input
            .importance
            .unwrap_or_else(|| existing.map_or(Importance::Unspecified, |r| r.importance)),
        milestone: input
            .milestone
            .unwrap_or_else(|| existing.is_some_and(|r| r.milestone)),
        confirmation: existing.and_then(|record| record.confirmation.clone()),
    };
    replace_or_push(&mut state.anchor_annotations, record);
    Ok(())
}

fn upsert_deadline_annotation(
    state: &mut LocalState,
    input: &LocalMutation,
    trace_tasks: &[(String, TaskRefId)],
    now: Instant,
) -> Result<(), String> {
    let target_id = parse_id::<DeadlineId>(input.id.as_deref())?;
    let existing = state
        .deadline_annotations
        .iter()
        .find(|record| record.target_id == target_id);
    let revision = existing.map_or(Ok(NonZeroU64::MIN), |record| increment(record.revision))?;
    let record = DeadlineAnnotation {
        target_id,
        revision,
        created_at: existing.map_or(now, |record| record.created_at),
        updated_at: now,
        importance: input
            .importance
            .unwrap_or_else(|| existing.map_or(Importance::Unspecified, |r| r.importance)),
        milestone: input
            .milestone
            .unwrap_or_else(|| existing.is_some_and(|r| r.milestone)),
        work: if input.clear_work {
            DeadlineWork::Unspecified
        } else if input.trace_external_id.is_some() {
            if input.effort_minutes.is_some() || input.minimum_chunk_minutes.is_some() {
                return Err("linked work uses the Trace task's one annotation; do not duplicate its estimate".into());
            }
            DeadlineWork::Task(trace_target(input, trace_tasks)?)
        } else if has_work(input) {
            if !input.replace_work
                && existing.is_some_and(|r| matches!(r.work, DeadlineWork::Task(_)))
            {
                return Err("edit the linked Trace work annotation, or explicitly replace the work association".into());
            }
            DeadlineWork::Standalone(updated_work(
                input,
                existing.and_then(|r| match &r.work {
                    DeadlineWork::Standalone(work) => Some(work),
                    _ => None,
                }),
            )?)
        } else {
            existing.map_or(DeadlineWork::Unspecified, |r| r.work.clone())
        },
        confirmation: existing.and_then(|record| record.confirmation.clone()),
    };
    replace_or_push(&mut state.deadline_annotations, record);
    Ok(())
}

/// Place and notes are display text kept beside the fact, never inside it.
fn update_details(state: &mut LocalState, id: &str, input: &LocalMutation) -> Result<(), String> {
    if input.place.is_none() && input.notes.is_none() {
        return Ok(());
    }
    let mut details = state.event_details.get(id).cloned().unwrap_or_default();
    if input.place.is_some() {
        details.place = series::clean_text(input.place.as_deref(), 200, "Place")?;
    }
    if input.notes.is_some() {
        details.notes = series::clean_text(input.notes.as_deref(), 4000, "Notes")?;
    }
    if details.is_empty() {
        state.event_details.remove(id);
    } else {
        state.event_details.insert(id.to_string(), details);
    }
    Ok(())
}

/// `YYYY-MM-DDTHH:MM[...]` into a civil date and minute of day.
fn split_wall(value: &str) -> Result<(LocalDate, u32), String> {
    let date = value
        .get(..10)
        .ok_or("use YYYY-MM-DDTHH:MM")?
        .parse::<LocalDate>()
        .map_err(time_error)?;
    let minute = series::parse_minute(value.get(11..16).ok_or("use YYYY-MM-DDTHH:MM")?, false)?;
    Ok((date, u32::from(minute)))
}

fn upsert_series(
    state: &mut LocalState,
    input: &LocalMutation,
    now: Instant,
) -> Result<(), String> {
    let id = upsert_id::<AnchorId>(input.id.as_deref())?;
    let existing = state
        .anchor_series
        .iter()
        .find(|record| record.meta.id == id)
        .cloned();
    let title = match input.title.as_deref() {
        Some(value) => required(Some(value), "event title")?.to_string(),
        None => existing
            .as_ref()
            .map(|record| record.title.clone())
            .ok_or("event title is required")?,
    };
    let zone = parse_zone(&input.zone)?;
    let has_span = input.all_day || input.start.is_some() || input.start_date.is_some();
    let (first_date, timing) = if !has_span {
        let record = existing.as_ref().ok_or("event time is required")?;
        (record.first_date, record.timing.clone())
    } else if input.all_day {
        let start = required(input.start_date.as_deref(), "first date")?
            .parse::<LocalDate>()
            .map_err(time_error)?;
        let end = required(input.end_date_exclusive.as_deref(), "end date")?
            .parse::<LocalDate>()
            .map_err(time_error)?;
        let mut days = 0u32;
        let mut cursor = start;
        while cursor < end {
            cursor = cursor.next_day().map_err(time_error)?;
            days += 1;
            if days > 366 {
                return Err("an all-day occurrence can last at most a year".into());
            }
        }
        let days = NonZeroU32::new(days).ok_or("the end date must follow the first date")?;
        (start, SeriesTiming::AllDay { days })
    } else {
        let (date, start_minute) = split_wall(required(input.start.as_deref(), "start time")?)?;
        let (end_date, end_minute) = split_wall(required(input.end.as_deref(), "end time")?)?;
        let start = series::resolve_wall(date, start_minute, zone).map_err(time_error)?;
        let end = series::resolve_wall(end_date, end_minute, zone).map_err(time_error)?;
        let minutes = end
            .duration_since(start)
            .map_err(|_| "the end must follow the start".to_string())?
            / 60_000;
        let minutes = u32::try_from(minutes)
            .ok()
            .filter(|m| *m <= 7 * 24 * 60)
            .and_then(NonZeroU32::new)
            .ok_or("an occurrence lasts between one minute and one week")?;
        (
            date,
            SeriesTiming::Timed {
                start_minute: u16::try_from(start_minute).map_err(|_| "invalid start")?,
                duration_minutes: minutes,
            },
        )
    };
    let rule = match (input.frequency, existing.as_ref()) {
        (Some(frequency), _) => {
            let until = input
                .until_date_exclusive
                .as_deref()
                .filter(|value| !value.trim().is_empty())
                .map(|value| value.parse::<LocalDate>().map_err(time_error))
                .transpose()?;
            if until.is_some_and(|until| until <= first_date) {
                return Err("the repeat must end after the first occurrence".into());
            }
            SeriesRule {
                frequency,
                interval: NonZeroU32::new(input.interval.unwrap_or(1))
                    .filter(|n| n.get() <= 99)
                    .ok_or("repeat every 1 to 99 periods")?,
                weekdays: if frequency == SeriesFrequency::Weekly {
                    input.weekdays.iter().copied().collect()
                } else {
                    BTreeSet::new()
                },
                until_date_exclusive: until,
                count: input
                    .count
                    .map(|count| {
                        NonZeroU32::new(count)
                            .filter(|n| n.get() <= 5000)
                            .ok_or("repeat between 1 and 5000 times")
                    })
                    .transpose()?,
            }
        }
        (None, Some(record)) => record.rule.clone(),
        (None, None) => return Err("choose how this event repeats".into()),
    };
    let mut details = existing
        .as_ref()
        .map(|record| record.details.clone())
        .unwrap_or_default();
    if input.place.is_some() {
        details.place = series::clean_text(input.place.as_deref(), 200, "Place")?;
    }
    if input.notes.is_some() {
        details.notes = series::clean_text(input.notes.as_deref(), 4000, "Notes")?;
    }
    let record = AnchorSeries {
        meta: next_meta(existing.as_ref().map(|record| &record.meta), id, now)?,
        title,
        presence: Presence::Present,
        zone,
        first_date,
        timing,
        rule,
        skipped: existing
            .as_ref()
            .map(|record| record.skipped.clone())
            .unwrap_or_default(),
        occupancy: input
            .occupancy
            .or_else(|| existing.as_ref().map(|r| r.occupancy))
            .ok_or("declare whether this event blocks availability")?,
        reported_certainty: input.certainty.unwrap_or_else(|| {
            existing
                .as_ref()
                .map_or(ReportedCertainty::Confirmed, |r| r.reported_certainty)
        }),
        location: if input.context.is_some() {
            context_tag(input.context.as_deref())?
        } else {
            existing.as_ref().and_then(|record| record.location.clone())
        },
        details,
    };
    if let Some(slot) = state
        .anchor_series
        .iter_mut()
        .find(|record| record.meta.id == id)
    {
        *slot = record;
    } else {
        state.anchor_series.push(record);
    }
    Ok(())
}

fn skip_occurrence(
    state: &mut LocalState,
    input: &LocalMutation,
    now: Instant,
    skip: bool,
) -> Result<(), String> {
    let id = parse_id::<AnchorId>(input.id.as_deref())?;
    let date = required(input.occurrence_date.as_deref(), "occurrence date")?
        .parse::<LocalDate>()
        .map_err(time_error)?;
    let record = state
        .anchor_series
        .iter_mut()
        .find(|record| record.meta.id == id)
        .ok_or("repeating event does not exist")?;
    let changed = if skip {
        record.skipped.insert(date)
    } else {
        record.skipped.remove(&date)
    };
    if !changed {
        return Err(if skip {
            "that occurrence is already skipped".into()
        } else {
            "that occurrence is not skipped".into()
        });
    }
    touch(&mut record.meta, now)
}

fn upsert_usual(state: &mut LocalState, input: &LocalMutation, now: Instant) -> Result<(), String> {
    let blocks = input
        .blocks
        .iter()
        .map(|block| {
            Ok(WeeklyBlock {
                weekday: block.weekday,
                start_minute: series::parse_minute(&block.start, false)?,
                end_minute: series::parse_minute(&block.end, true)?,
            })
        })
        .collect::<Result<Vec<_>, String>>()?;
    series::validate_blocks(&blocks)?;
    let existing = state.usual_availability.as_ref();
    let id = existing.map_or_else(|| upsert_id::<AvailabilityId>(None), |u| Ok(u.meta.id))?;
    let meta = next_meta(existing.map(|u| &u.meta), id, now)?;
    state.usual_availability = Some(UsualAvailability {
        meta,
        zone: parse_zone(&input.zone)?,
        blocks,
        contexts: declared_contexts(input.context.as_deref())?,
        energy_capacity: input.energy_capacity.unwrap_or(EnergyCapacity::Unknown),
    });
    Ok(())
}

fn temporal_span(input: &LocalMutation) -> Result<TemporalSpan, String> {
    if input.all_day {
        let start = required(input.start_date.as_deref(), "all-day start date")?
            .parse::<LocalDate>()
            .map_err(time_error)?;
        let end = required(
            input.end_date_exclusive.as_deref(),
            "all-day exclusive end date",
        )?
        .parse::<LocalDate>()
        .map_err(time_error)?;
        return Ok(TemporalSpan::Dates(
            DateSpan::new(start, end, parse_zone(&input.zone)?).map_err(time_error)?,
        ));
    }
    Ok(TemporalSpan::Timed(timed_span(input)?))
}

fn cutoff(input: &LocalMutation) -> Result<Cutoff, String> {
    if input.all_day {
        let date = required(
            input.due.as_deref().or(input.start_date.as_deref()),
            "deadline date",
        )?
        .parse::<LocalDate>()
        .map_err(time_error)?;
        Ok(Cutoff::OnDate {
            date,
            zone: parse_zone(&input.zone)?,
        })
    } else {
        let local = required(
            input.due.as_deref().or(input.start.as_deref()),
            "deadline time",
        )?;
        let zone = parse_zone(&input.zone)?;
        Ok(Cutoff::At {
            instant: resolve_local(local, zone)?,
            original_zone: Some(zone),
        })
    }
}

fn timed_span(input: &LocalMutation) -> Result<TimedSpan, String> {
    let zone = parse_zone(&input.zone)?;
    let start = resolve_local(required(input.start.as_deref(), "start time")?, zone)?;
    let end = resolve_local(required(input.end.as_deref(), "end time")?, zone)?;
    TimedSpan::new(start, end)
        .map(|span| span.with_original_zone(zone))
        .map_err(time_error)
}

fn resolve_local(value: &str, zone: ZoneId) -> Result<Instant, String> {
    let normalized = match value.len() {
        16 => format!("{value}:00.000"),
        19 => format!("{value}.000"),
        _ => value.to_string(),
    };
    let local = normalized.parse::<LocalDateTime>().map_err(time_error)?;
    TimezoneRules::bundled()
        .resolve_local(local, zone, LocalResolution::RejectAmbiguous)
        .map_err(time_error)
}

fn work(input: &LocalMutation) -> Result<WorkMetadata, String> {
    let effort = match input.effort_minutes {
        None => Effort::Unknown,
        Some(minutes) => Effort::Estimate(minutes),
    };
    let chunk = input
        .minimum_chunk_minutes
        .map(|minutes| {
            NonZeroU32::new(minutes).ok_or("minimum session must be greater than zero".to_string())
        })
        .transpose()?;
    if let (Effort::Estimate(minutes), Some(chunk)) = (effort, chunk)
        && chunk.get() > minutes
    {
        return Err("minimum session cannot exceed the effort estimate".into());
    }
    Ok(WorkMetadata {
        effort,
        minimum_chunk_minutes: chunk,
        required_contexts: context_tags(input.context.as_deref())?,
        energy_requirement: input.energy_requirement.unwrap_or_default(),
        ..WorkMetadata::default()
    })
}

fn has_work(input: &LocalMutation) -> bool {
    input.replace_work
        || input.effort_minutes.is_some()
        || input.minimum_chunk_minutes.is_some()
        || input.context.is_some()
        || input.energy_requirement.is_some()
}

fn updated_work(
    input: &LocalMutation,
    existing: Option<&WorkMetadata>,
) -> Result<WorkMetadata, String> {
    if input.replace_work || existing.is_none() {
        return work(input);
    }
    let mut result = existing.cloned().unwrap_or_default();
    if let Some(minutes) = input.effort_minutes {
        result.effort = Effort::Estimate(minutes);
    }
    if let Some(minutes) = input.minimum_chunk_minutes {
        result.minimum_chunk_minutes =
            Some(NonZeroU32::new(minutes).ok_or("minimum session must be greater than zero")?);
    }
    if input.context.is_some() {
        result.required_contexts = context_tags(input.context.as_deref())?;
    }
    if let Some(energy) = input.energy_requirement {
        result.energy_requirement = energy;
    }
    Ok(result)
}

fn context_tags(value: Option<&str>) -> Result<BTreeSet<ContextTag>, String> {
    value
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| {
            value
                .parse()
                .map_err(|error: temporal_core::domain::InvalidContextTag| error.to_string())
        })
        .collect()
}

fn declared_contexts(value: Option<&str>) -> Result<DeclaredContexts, String> {
    if value.is_none() || value.is_some_and(|value| value.trim().is_empty()) {
        Ok(DeclaredContexts::Unknown)
    } else {
        Ok(DeclaredContexts::Known(context_tags(value)?))
    }
}

fn context_tag(value: Option<&str>) -> Result<Option<ContextTag>, String> {
    value
        .filter(|value| !value.trim().is_empty())
        .map(|value| {
            value
                .trim()
                .parse()
                .map_err(|error: temporal_core::domain::InvalidContextTag| error.to_string())
        })
        .transpose()
}

fn trace_target(
    input: &LocalMutation,
    trace_tasks: &[(String, TaskRefId)],
) -> Result<TaskRefId, String> {
    let external = required(input.trace_external_id.as_deref(), "Trace task ID")?;
    trace_tasks
        .iter()
        .find(|(id, _)| id == external)
        .map(|(_, id)| *id)
        .ok_or_else(|| "Trace task is not present in the retained snapshot".into())
}

fn local_provenance(source_id: SourceId, asserted_at: Instant) -> Provenance {
    Provenance::LocalUser(LocalAssertion {
        source_id,
        asserted_at,
    })
}
fn local_user_provenance(source_id: SourceId, asserted_at: Instant) -> UserProvenance {
    UserProvenance::LocalUser(LocalAssertion {
        source_id,
        asserted_at,
    })
}

fn required<'a>(value: Option<&'a str>, label: &str) -> Result<&'a str, String> {
    value
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| format!("{label} is required"))
}
fn parse_zone(value: &str) -> Result<ZoneId, String> {
    value.parse::<ZoneId>().map_err(time_error)
}
fn time_error(error: TimeError) -> String {
    format!("invalid local time: {error}")
}
fn parse_id<I: std::str::FromStr>(value: Option<&str>) -> Result<I, String>
where
    I::Err: std::fmt::Display,
{
    required(value, "record ID")?
        .parse()
        .map_err(|error: I::Err| error.to_string())
}
fn upsert_id<I: std::str::FromStr>(value: Option<&str>) -> Result<I, String>
where
    I::Err: std::fmt::Display,
{
    value.map_or_else(
        || {
            uuid::Uuid::new_v4()
                .to_string()
                .parse()
                .map_err(|error: I::Err| error.to_string())
        },
        |value| value.parse().map_err(|error: I::Err| error.to_string()),
    )
}
fn increment(value: NonZeroU64) -> Result<NonZeroU64, String> {
    value
        .get()
        .checked_add(1)
        .and_then(NonZeroU64::new)
        .ok_or_else(|| "local record revision limit reached".into())
}
fn next_meta<I: Copy>(
    existing: Option<&RecordMeta<I>>,
    id: I,
    now: Instant,
) -> Result<RecordMeta<I>, String> {
    Ok(RecordMeta {
        id,
        revision: existing.map_or(Ok(NonZeroU64::MIN), |meta| increment(meta.revision))?,
        created_at: existing.map_or(now, |meta| meta.created_at),
        updated_at: now,
    })
}
fn touch<I>(meta: &mut RecordMeta<I>, now: Instant) -> Result<(), String> {
    meta.revision = increment(meta.revision)?;
    meta.updated_at = now;
    Ok(())
}
fn replace_or_push<T, I: PartialEq + Copy>(records: &mut Vec<T>, record: T)
where
    T: RecordWithId<I>,
{
    if let Some(slot) = records
        .iter_mut()
        .find(|existing| existing.record_id() == record.record_id())
    {
        *slot = record;
    } else {
        records.push(record);
    }
}
trait RecordWithId<I> {
    fn record_id(&self) -> I;
}
impl RecordWithId<AnchorId> for Anchor {
    fn record_id(&self) -> AnchorId {
        self.meta.id
    }
}
impl RecordWithId<DeadlineId> for Deadline {
    fn record_id(&self) -> DeadlineId {
        self.meta.id
    }
}
impl RecordWithId<IntentionId> for Intention {
    fn record_id(&self) -> IntentionId {
        self.meta.id
    }
}
impl RecordWithId<RoutineId> for Routine {
    fn record_id(&self) -> RoutineId {
        self.meta.id
    }
}
impl RecordWithId<AvailabilityId> for AvailabilityDeclaration {
    fn record_id(&self) -> AvailabilityId {
        self.meta.id
    }
}
impl RecordWithId<TaskRefId> for TaskAnnotation {
    fn record_id(&self) -> TaskRefId {
        self.target_id
    }
}
impl RecordWithId<AnchorId> for AnchorAnnotation {
    fn record_id(&self) -> AnchorId {
        self.target_id
    }
}
impl RecordWithId<DeadlineId> for DeadlineAnnotation {
    fn record_id(&self) -> DeadlineId {
        self.target_id
    }
}

fn find_mut<'a, T, I>(records: &'a mut [T], id: I, label: &str) -> Result<&'a mut T, String>
where
    T: RecordWithId<I>,
    I: Copy + PartialEq,
{
    records
        .iter_mut()
        .find(|record| record.record_id() == id)
        .ok_or_else(|| format!("{label} does not exist"))
}
fn remove_by_id<T, I>(records: &mut Vec<T>, id: I, label: &str) -> Result<(), String>
where
    T: RecordWithId<I>,
    I: Copy + PartialEq,
{
    let before = records.len();
    records.retain(|record| record.record_id() != id);
    if records.len() == before {
        Err(format!("{label} does not exist"))
    } else {
        Ok(())
    }
}
