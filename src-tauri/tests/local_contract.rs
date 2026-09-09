use temporal_app::{
    local::{LocalAction, LocalCompletion, LocalKind, LocalMutation},
    store::Store,
};
use temporal_core::domain::*;
use temporal_core::time::Instant;

fn now() -> Instant {
    "2026-09-09T12:00:00.000Z".parse().unwrap()
}

fn timed(kind: LocalKind, title: &str, start: &str, end: &str) -> LocalMutation {
    let mut mutation = LocalMutation::upsert(kind);
    mutation.title = Some(title.into());
    mutation.start = Some(start.into());
    mutation.end = Some(end.into());
    if kind == LocalKind::Anchor {
        mutation.occupancy = Some(Occupancy::Busy);
    }
    mutation
}

fn trace_export() -> String {
    serde_json::json!({
        "version":"1.0",
        "exported_at":now().to_string(),
        "tasks":[{
            "id":"trace-one","text":"Trace work","raw_input":null,"link":null,
            "status":"later","context":"desk","priority":2,
            "due_at":null,"created_at":"2026-09-09T10:00:00.000Z",
            "updated_at":"2026-09-09T11:00:00.000Z","completed_at":null,"sort_order":1.5
        }]
    })
    .to_string()
}

fn read(store: &Store, at: Instant) -> EvaluationInput {
    store
        .read(at, "America/Toronto".parse().unwrap())
        .unwrap()
        .input
}

#[test]
fn local_records_and_annotations_validate_and_survive_restart() {
    let path =
        std::env::temp_dir().join(format!("temporal-local-{}.sqlite3", uuid::Uuid::new_v4()));
    let mut store = Store::open(&path).unwrap();

    let mut anchor = timed(
        LocalKind::Anchor,
        "Dentist",
        "2026-09-10T09:00",
        "2026-09-10T10:00",
    );
    anchor.context = Some("clinic".into());
    anchor.milestone = Some(true);
    store.mutate_local(&anchor, now()).unwrap();

    let mut deadline = LocalMutation::upsert(LocalKind::Deadline);
    deadline.title = Some("Submit form".into());
    deadline.due = Some("2026-09-11T17:00".into());
    deadline.effort_minutes = Some(60);
    deadline.minimum_chunk_minutes = Some(30);
    store.mutate_local(&deadline, now()).unwrap();

    let mut intention = LocalMutation::upsert(LocalKind::Intention);
    intention.title = Some("Walk outside".into());
    intention.all_day = true;
    intention.start_date = Some("2026-09-10".into());
    intention.end_date_exclusive = Some("2026-09-11".into());
    intention.effort_minutes = Some(30);
    store.mutate_local(&intention, now()).unwrap();

    let mut routine = LocalMutation::upsert(LocalKind::Routine);
    routine.title = Some("Weekly review".into());
    routine.weekdays = vec![Weekday::Fri];
    routine.start_date = Some("2026-09-11".into());
    routine.effort_minutes = Some(45);
    routine.minimum_chunk_minutes = Some(15);
    store.mutate_local(&routine, now()).unwrap();

    let mut availability = timed(
        LocalKind::Availability,
        "",
        "2026-09-10T08:00",
        "2026-09-10T12:00",
    );
    availability.context = Some("home,quiet".into());
    availability.energy_capacity = Some(EnergyCapacity::Normal);
    store.mutate_local(&availability, now()).unwrap();

    store.import_json(&trace_export(), now()).unwrap();
    let mut task_annotation = LocalMutation::upsert(LocalKind::TaskAnnotation);
    task_annotation.trace_external_id = Some("trace-one".into());
    task_annotation.importance = Some(Importance::High);
    task_annotation.effort_minutes = Some(90);
    task_annotation.minimum_chunk_minutes = Some(30);
    store.mutate_local(&task_annotation, now()).unwrap();

    let before = store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap()
        .input;
    assert_eq!(before.anchors.len(), 1);
    assert_eq!(before.deadlines.len(), 1);
    assert_eq!(before.intentions.len(), 1);
    assert_eq!(before.routines.len(), 1);
    assert_eq!(before.availability.len(), 1);
    assert_eq!(before.anchor_annotations.len(), 1);
    assert_eq!(before.task_annotations.len(), 1);
    assert_eq!(
        before
            .sources
            .iter()
            .filter(|source| source.kind == SourceKind::Local)
            .count(),
        1
    );
    assert_eq!(before.task_refs[0].source_context.as_deref(), Some("desk"));
    assert_eq!(before.task_annotations[0].importance, Importance::High);
    assert!(temporal_core::evaluate(&before).is_ok());

    drop(store);
    let reopened = Store::open(&path).unwrap();
    let after = reopened
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap()
        .input;
    assert_eq!(after, before);
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn local_edit_and_explicit_remove_preserve_identity_and_revision() {
    let mut store = Store::memory().unwrap();
    let mut mutation = timed(
        LocalKind::Anchor,
        "First title",
        "2026-09-10T09:00",
        "2026-09-10T10:00",
    );
    store.mutate_local(&mutation, now()).unwrap();
    let first = store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap()
        .input
        .anchors[0]
        .clone();

    mutation.id = Some(first.meta.id.to_string());
    mutation.title = Some("Edited title".into());
    mutation.start = Some("2026-09-10T11:00".into());
    mutation.end = Some("2026-09-10T12:00".into());
    store.mutate_local(&mutation, now()).unwrap();
    let edited = store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap()
        .input
        .anchors[0]
        .clone();
    assert_eq!(edited.meta.id, first.meta.id);
    assert_eq!(edited.meta.created_at, first.meta.created_at);
    assert!(edited.meta.revision > first.meta.revision);
    assert_eq!(edited.title, "Edited title");

    mutation.action = LocalAction::Remove;
    mutation.title = None;
    mutation.start = None;
    mutation.end = None;
    store.mutate_local(&mutation, now()).unwrap();
    let removed = store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap()
        .input
        .anchors[0]
        .clone();
    assert_eq!(removed.presence, Presence::Removed);
    assert_eq!(removed.meta.id, first.meta.id);
}

#[test]
fn invalid_local_changes_are_rejected_without_partial_writes() {
    let mut store = Store::memory().unwrap();
    let baseline = store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap()
        .input;

    let mut invalid_span = timed(
        LocalKind::Anchor,
        "Bad",
        "2026-09-10T09:00",
        "2026-09-10T09:00",
    );
    assert!(store.mutate_local(&invalid_span, now()).is_err());
    invalid_span.end = Some("2026-09-10T10:00".into());
    invalid_span.context = Some("Not a valid context".into());
    assert!(store.mutate_local(&invalid_span, now()).is_err());

    let mut invalid_routine = LocalMutation::upsert(LocalKind::Routine);
    invalid_routine.title = Some("No days".into());
    invalid_routine.start_date = Some("2026-09-10".into());
    assert!(store.mutate_local(&invalid_routine, now()).is_err());

    let mut missing_task = LocalMutation::upsert(LocalKind::TaskAnnotation);
    missing_task.trace_external_id = Some("not-present".into());
    assert!(store.mutate_local(&missing_task, now()).is_err());

    let current = store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap()
        .input;
    assert_eq!(current, baseline);
}

#[test]
fn local_completion_and_routine_outcomes_are_explicit_and_revisioned() {
    let mut store = Store::memory().unwrap();

    let mut deadline = LocalMutation::upsert(LocalKind::Deadline);
    deadline.title = Some("Pay invoice".into());
    deadline.due = Some("2026-09-10T17:00".into());
    store.mutate_local(&deadline, now()).unwrap();
    let deadline_id = store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap()
        .input
        .deadlines[0]
        .meta
        .id;

    let mut satisfied = LocalMutation::upsert(LocalKind::Deadline);
    satisfied.id = Some(deadline_id.to_string());
    satisfied.completion = Some(LocalCompletion::Satisfied);
    store.mutate_local(&satisfied, now()).unwrap();
    let deadline = &store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap()
        .input
        .deadlines[0];
    assert!(matches!(
        deadline.fulfillment,
        Fulfillment::Recorded(RecordedResolution::Satisfied(_))
    ));

    let mut intention = LocalMutation::upsert(LocalKind::Intention);
    intention.title = Some("Read a chapter".into());
    intention.effort_minutes = Some(30);
    store.mutate_local(&intention, now()).unwrap();
    let intention_id = store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap()
        .input
        .intentions[0]
        .meta
        .id;
    let mut done = LocalMutation::upsert(LocalKind::Intention);
    done.id = Some(intention_id.to_string());
    done.completion = Some(LocalCompletion::Done);
    store.mutate_local(&done, now()).unwrap();
    let projected =
        serde_json::to_value(temporal_app::personal::view(&store, now()).unwrap()).unwrap();
    let shown = projected["view"]["items"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["id"] == intention_id.to_string())
        .unwrap();
    assert_eq!(shown["phase"], "done");

    let mut routine = LocalMutation::upsert(LocalKind::Routine);
    routine.title = Some("Weekly review".into());
    routine.weekdays = vec![Weekday::Fri];
    routine.start_date = Some("2026-09-04".into());
    routine.effort_minutes = Some(30);
    store.mutate_local(&routine, now()).unwrap();
    let routine_id = store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap()
        .input
        .routines[0]
        .meta
        .id;
    let mut outcome = LocalMutation::upsert(LocalKind::RoutineOutcome);
    outcome.id = Some(routine_id.to_string());
    outcome.occurrence_date = Some("2026-09-11".into());
    outcome.outcome = Some(Outcome::Done);
    store.mutate_local(&outcome, now()).unwrap();

    let input = store
        .read(now(), "America/Toronto".parse().unwrap())
        .unwrap()
        .input;
    assert_eq!(input.intentions[0].state, IntentionState::Done);
    assert_eq!(input.routine_outcomes[0].outcome, Outcome::Done);
    assert!(input.routine_outcomes[0].recorded_at <= now());
    assert!(temporal_core::evaluate(&input).is_ok());
}

#[test]
fn partial_edits_preserve_evidence_and_full_work_edits_can_clear_unknowns() {
    let mut store = Store::memory().unwrap();
    let mut d = LocalMutation::upsert(LocalKind::Deadline);
    d.title = Some("Submit synthetic work".into());
    d.all_day = true;
    d.due = Some("2026-09-10".into());
    d.effort_minutes = Some(90);
    d.minimum_chunk_minutes = Some(30);
    d.context = Some("desk,quiet".into());
    d.energy_requirement = Some(EnergyRequirement::Deep);
    d.milestone = Some(true);
    d.importance = Some(Importance::High);
    store.mutate_local(&d, now()).unwrap();
    let before = read(&store, now());
    let id = before.deadlines[0].meta.id;
    let mut edit = LocalMutation::upsert(LocalKind::Deadline);
    edit.id = Some(id.to_string());
    edit.completion = Some(LocalCompletion::Satisfied);
    let later = now().checked_add_ms(60_000).unwrap();
    store.mutate_local(&edit, later).unwrap();
    let evidence = read(&store, later).deadlines[0].fulfillment.clone();
    edit.completion = None;
    edit.title = Some("Edited submission".into());
    edit.importance = Some(Importance::Low);
    store.mutate_local(&edit, later).unwrap();
    let after = read(&store, later);
    assert_eq!(after.deadlines[0].fulfillment, evidence);
    assert_eq!(after.deadlines[0].cutoff, before.deadlines[0].cutoff);
    assert_eq!(
        after.deadline_annotations[0].work,
        before.deadline_annotations[0].work
    );
    assert!(after.deadline_annotations[0].milestone);
    edit.importance = None;
    edit.effort_minutes = Some(120);
    store.mutate_local(&edit, later).unwrap();
    let DeadlineWork::Standalone(work) = read(&store, later).deadline_annotations[0].work.clone()
    else {
        panic!()
    };
    assert_eq!(work.minimum_chunk_minutes.unwrap().get(), 30);
    assert_eq!(work.required_contexts.len(), 2);
    edit.effort_minutes = None;
    edit.replace_work = true;
    edit.milestone = Some(false);
    store.mutate_local(&edit, later).unwrap();
    let cleared = read(&store, later);
    assert_eq!(
        cleared.deadline_annotations[0].work,
        DeadlineWork::Standalone(WorkMetadata::default())
    );
    assert!(!cleared.deadline_annotations[0].milestone);
    edit.effort_minutes = Some(0);
    edit.completion = Some(LocalCompletion::Unresolved);
    store.mutate_local(&edit, later).unwrap();
    let output = temporal_core::evaluate(&read(&store, later)).unwrap();
    assert_eq!(
        output.pressures[0].risk,
        temporal_core::results::Risk::NoKnownWork
    );
    assert_eq!(
        output.pressures[0].resolution,
        temporal_core::results::Resolution::Unresolved
    );
}

#[test]
fn trace_refresh_removal_and_restore_preserve_annotations_and_independent_deadlines() {
    let mut store = Store::memory().unwrap();
    store.import_json(&trace_export(), now()).unwrap();
    let original = read(&store, now()).task_refs[0].clone();
    let mut annotation = LocalMutation::upsert(LocalKind::TaskAnnotation);
    annotation.trace_external_id = Some("trace-one".into());
    annotation.effort_minutes = Some(60);
    annotation.minimum_chunk_minutes = Some(30);
    annotation.context = Some("quiet".into());
    store.mutate_local(&annotation, now()).unwrap();
    let saved_note = read(&store, now()).task_annotations[0].clone();
    let mut deadline = LocalMutation::upsert(LocalKind::Deadline);
    deadline.title = Some("Independent cutoff".into());
    deadline.due = Some("2026-09-11T17:00".into());
    deadline.trace_external_id = Some("trace-one".into());
    store.mutate_local(&deadline, now()).unwrap();
    // A second unresolved work owner is rejected atomically.
    let before = read(&store, now());
    assert!(store.mutate_local(&deadline, now()).is_err());
    assert_eq!(read(&store, now()), before);
    let mut wire: serde_json::Value = serde_json::from_str(&trace_export()).unwrap();
    for (index, status) in ["done", "removed", "later"].iter().enumerate() {
        let at = now().checked_add_ms((index as u64 + 1) * 60_000).unwrap();
        wire["exported_at"] = at.to_string().into();
        if *status == "removed" {
            wire["tasks"] = serde_json::json!([]);
        } else {
            let base: serde_json::Value = serde_json::from_str(&trace_export()).unwrap();
            wire["tasks"] = base["tasks"].clone();
            wire["tasks"][0]["status"] = (*status).into();
            wire["tasks"][0]["text"] = "Source edited title".into();
        }
        store.import_json(&wire.to_string(), at).unwrap();
        let current = read(&store, at);
        assert_eq!(current.task_refs[0].meta.id, original.meta.id);
        assert_eq!(current.task_annotations[0], saved_note);
        let projected =
            serde_json::to_value(temporal_app::personal::view(&store, at).unwrap()).unwrap();
        assert_eq!(
            projected["trace"]["tasks"][0]["id"],
            original.meta.id.to_string()
        );
        assert_eq!(
            projected["trace"]["tasks"][0]["presence"],
            if *status == "removed" {
                "removed"
            } else {
                "present"
            }
        );
        assert_eq!(
            current.deadlines[0].fulfillment,
            Fulfillment::Recorded(RecordedResolution::Unresolved)
        );
        if *status == "done" {
            let output = temporal_core::evaluate(&current).unwrap();
            assert_eq!(
                output.pressures[0].workload,
                Some(temporal_core::results::Workload::Estimate(0))
            );
        }
    }
    // The annotation command cannot pretend to edit source-owned text or completion.
    let later = now().checked_add_ms(180_000).unwrap();
    let before = read(&store, later);
    annotation.title = Some("Local attempted title".into());
    assert!(store.mutate_local(&annotation, later).is_err());
    assert_eq!(read(&store, later), before);
}

#[test]
fn time_occupancy_availability_and_identity_boundaries_reject_without_partial_writes() {
    let mut store = Store::memory().unwrap();
    let mut declaration = timed(
        LocalKind::Availability,
        "",
        "2026-09-10T08:00",
        "2026-09-10T12:00",
    );
    store.mutate_local(&declaration, now()).unwrap();
    let before = read(&store, now());
    declaration.start = Some("2026-09-10T11:00".into());
    declaration.end = Some("2026-09-10T13:00".into());
    assert!(store.mutate_local(&declaration, now()).is_err());
    assert_eq!(read(&store, now()), before);
    let mut anchor = timed(LocalKind::Anchor, "Synthetic all day", "", "");
    anchor.start = None;
    anchor.end = None;
    anchor.all_day = true;
    anchor.start_date = Some("2026-09-10".into());
    anchor.end_date_exclusive = Some("2026-09-11".into());
    anchor.occupancy = None;
    assert!(store.mutate_local(&anchor, now()).is_err());
    anchor.occupancy = Some(Occupancy::Transparent);
    store.mutate_local(&anchor, now()).unwrap();
    let output = temporal_core::evaluate(&read(&store, now())).unwrap();
    assert_eq!(output.windows.len(), 1);
    for start in ["2026-03-08T02:30", "2026-11-01T01:30"] {
        let bad = timed(
            LocalKind::Anchor,
            "Unresolved DST",
            start,
            "2026-11-02T09:00",
        );
        let before = read(&store, now());
        assert!(store.mutate_local(&bad, now()).is_err());
        assert_eq!(read(&store, now()), before);
    }
    let mut no_zone = timed(
        LocalKind::Anchor,
        "No implicit zone",
        "2026-09-10T15:00",
        "2026-09-10T16:00",
    );
    no_zone.zone.clear();
    assert!(store.mutate_local(&no_zone, now()).is_err());
    no_zone.zone = "America/Toronto".into();
    no_zone.id = Some(read(&store, now()).availability[0].meta.id.to_string());
    assert!(store.mutate_local(&no_zone, now()).is_err());
    assert!(
        serde_json::from_str::<LocalMutation>(
            r#"{"action":"upsert","kind":"task_annotation","status":"done"}"#
        )
        .is_err()
    );
}

#[test]
fn routine_rule_edits_keep_outcomes_and_soft_time_never_creates_debt() {
    let mut store = Store::memory().unwrap();
    let mut routine = LocalMutation::upsert(LocalKind::Routine);
    routine.title = Some("Synthetic review".into());
    routine.weekdays = vec![Weekday::Fri];
    routine.start_date = Some("2026-09-04".into());
    store.mutate_local(&routine, now()).unwrap();
    let id = read(&store, now()).routines[0].meta.id;
    let mut outcome = LocalMutation::upsert(LocalKind::RoutineOutcome);
    outcome.id = Some(id.to_string());
    outcome.occurrence_date = Some("2026-09-04".into());
    outcome.outcome = Some(Outcome::Done);
    store.mutate_local(&outcome, now()).unwrap();
    outcome.occurrence_date = Some("2026-09-05".into());
    assert!(store.mutate_local(&outcome, now()).is_err());
    routine.id = Some(id.to_string());
    routine.weekdays = vec![Weekday::Sun];
    store.mutate_local(&routine, now()).unwrap();
    let input = read(&store, now());
    assert_eq!(input.routine_outcomes.len(), 1);
    let historical = temporal_core::recurrence::occurrence_state(
        &input.routines[0],
        &input.routine_outcomes,
        "2026-09-06".parse().unwrap(),
        now(),
        &temporal_core::time::TimezoneRules::bundled(),
    )
    .unwrap();
    assert_eq!(
        historical.phase,
        temporal_core::results::OccurrencePhase::PastUnrecorded
    );
    assert!(
        temporal_core::evaluate(&input)
            .unwrap()
            .pressures
            .is_empty()
    );
    let mut intention = LocalMutation::upsert(LocalKind::Intention);
    intention.title = Some("Past preference".into());
    intention.all_day = true;
    intention.start_date = Some("2026-09-01".into());
    intention.end_date_exclusive = Some("2026-09-02".into());
    store.mutate_local(&intention, now()).unwrap();
    intention = LocalMutation::upsert(LocalKind::Intention);
    intention.id = Some(read(&store, now()).intentions[0].meta.id.to_string());
    intention.clear_preference = true;
    store.mutate_local(&intention, now()).unwrap();
    assert!(read(&store, now()).intentions[0].preferred_span.is_none());
    assert_eq!(
        read(&store, now()).intentions[0].state,
        IntentionState::Active
    );
}

#[test]
fn migration_local_failure_and_clock_rewind_keep_last_committed_state() {
    let path = std::env::temp_dir().join(format!(
        "temporal-local-migration-{}.sqlite3",
        uuid::Uuid::new_v4()
    ));
    let mut store = Store::open(&path).unwrap();
    store.import_json(&trace_export(), now()).unwrap();
    let original = read(&store, now()).task_refs;
    drop(store);
    // Build a synthetic v1 cache using the prior committed table schema.
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch("DROP TABLE local_temporal_state; PRAGMA user_version=1;")
        .unwrap();
    drop(connection);
    let mut store = Store::open(&path).unwrap();
    assert_eq!(read(&store, now()).task_refs, original);
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection.execute_batch("CREATE TRIGGER reject_local BEFORE UPDATE ON local_temporal_state BEGIN SELECT RAISE(ABORT, 'synthetic failure'); END;").unwrap();
    let mut entry = timed(
        LocalKind::Availability,
        "",
        "2026-09-10T09:00",
        "2026-09-10T10:00",
    );
    let baseline = read(&store, now());
    assert!(store.mutate_local(&entry, now()).is_err());
    assert_eq!(read(&store, now()), baseline);
    connection
        .execute_batch("DROP TRIGGER reject_local;")
        .unwrap();
    drop(connection);
    let later = now().checked_add_ms(60_000).unwrap();
    store.mutate_local(&entry, later).unwrap();
    entry.id = Some(read(&store, later).availability[0].meta.id.to_string());
    entry.action = LocalAction::Remove;
    store.mutate_local(&entry, later).unwrap();
    assert!(
        store
            .read(now(), "America/Toronto".parse().unwrap())
            .is_err()
    );
    let baseline = read(&store, later);
    drop(store);
    let reopened = Store::open(&path).unwrap();
    assert_eq!(read(&reopened, later), baseline);
    drop(reopened);
    std::fs::remove_file(path).unwrap();
}
