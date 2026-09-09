use serde_json::{Value, json};
use temporal_app::store::Store;
use temporal_core::{domain::*, time::Instant};

fn now() -> Instant {
    "2026-09-09T12:00:00.000Z".parse().unwrap()
}
fn row(id: &str) -> Value {
    json!({"id":id,"text":"Synthetic reading","raw_input":"Synthetic reading tomorrow",
        "link":null,"status":"now","context":"desk","priority":2,
        "due_at":"2026-09-10T03:59:59.999Z","created_at":"2026-09-09T10:00:00.000Z",
        "updated_at":"2026-09-09T11:00:00.000Z","completed_at":null,"sort_order":1.5})
}
fn export(at: Instant, rows: Vec<Value>) -> String {
    json!({"version":"1.0","exported_at":at.to_string(),"tasks":rows}).to_string()
}
fn input(store: &Store, at: Instant) -> EvaluationInput {
    store
        .read(at, "America/Toronto".parse().unwrap())
        .unwrap()
        .input
}

#[test]
fn source_facts_and_unresolved_due_survive_normalization_without_new_obligations() {
    let mut store = Store::memory().unwrap();
    let mut future_status = row("unknown");
    future_status["status"] = json!("waiting");
    future_status["new_source_field"] = json!("ignored");
    store
        .import_json(&export(now(), vec![row("first"), future_status]), now())
        .unwrap();
    let data = input(&store, now());
    assert_eq!(data.task_refs.len(), 2);
    assert!(data.deadlines.is_empty());
    for task in &data.task_refs {
        assert_eq!(task.meta.id.to_string().as_bytes()[14], b'4');
        assert_eq!(task.source_priority.as_deref(), Some("2"));
        assert_eq!(task.source_context.as_deref(), Some("desk"));
        assert!(
            matches!(&task.due, TaskDue::Unresolved { value, .. } if value == "2026-09-10T03:59:59.999Z")
        );
    }
    let unknown = data
        .task_refs
        .iter()
        .find(|t| t.provenance.imported().external_id == "unknown")
        .unwrap();
    assert_eq!(unknown.status, TaskStatus::Unknown);
    assert_eq!(unknown.unknown_status_label.as_deref(), Some("waiting"));
    assert_eq!(data, input(&store, now()));
    temporal_core::evaluate(&data).unwrap();
}

#[test]
fn repeat_import_keeps_identity_and_fact_revision_without_extending_freshness() {
    let mut store = Store::memory().unwrap();
    let wire = export(now(), vec![row("same")]);
    store.import_json(&wire, now()).unwrap();
    let first = input(&store, now());
    let later = now().checked_add_ms(3_600_000).unwrap();
    store.import_json(&wire, later).unwrap();
    let second = input(&store, later);
    assert_eq!(first.task_refs, second.task_refs);
    assert_eq!(first.sources, second.sources);
    let source = &second.source_states[0];
    assert_eq!(source.last_success_at, Some(later));
    assert_eq!(source.fresh_for_ms.get(), 23 * 3_600_000);
    let expired = now().checked_add_ms(24 * 3_600_000 + 1).unwrap();
    assert_eq!(
        temporal_core::evaluate(&input(&store, expired))
            .unwrap()
            .source_health
            .iter()
            .find(|s| s.source_id == source.source_id)
            .unwrap()
            .health,
        temporal_core::results::Health::Stale
    );
    assert!(store.import_json(&wire, expired).is_err());
    assert_eq!(input(&store, expired).task_refs, first.task_refs);
}

#[test]
fn edits_completion_and_unknown_state_keep_source_ownership_and_canonical_id() {
    let mut store = Store::memory().unwrap();
    store
        .import_json(&export(now(), vec![row("one")]), now())
        .unwrap();
    let original = input(&store, now()).task_refs[0].clone();
    let later = now().checked_add_ms(60_000).unwrap();
    let mut done = row("one");
    done["text"] = json!("Edited synthetic reading");
    done["status"] = json!("done");
    done["due_at"] = Value::Null;
    store
        .import_json(&export(later, vec![done]), later)
        .unwrap();
    let changed = input(&store, later).task_refs[0].clone();
    assert_eq!(changed.meta.id, original.meta.id);
    assert!(changed.meta.revision > original.meta.revision);
    assert_eq!(changed.status, TaskStatus::Done);
    assert_eq!(changed.source_completed_at, None);
    assert_eq!(changed.due, TaskDue::None);
    assert!(input(&store, later).deadlines.is_empty());
    let next = later.checked_add_ms(60_000).unwrap();
    store
        .import_json(&export(next, vec![row("one")]), next)
        .unwrap();
    assert_eq!(input(&store, next).task_refs[0].status, TaskStatus::Now);
    assert_eq!(input(&store, next).task_refs[0].meta.id, original.meta.id);
}

#[test]
fn invalid_partial_duplicate_and_incompatible_imports_never_apply_partial_facts() {
    let mut store = Store::memory().unwrap();
    store
        .import_json(&export(now(), vec![row("keep")]), now())
        .unwrap();
    let before = input(&store, now()).task_refs;
    let later = now().checked_add_ms(60_000).unwrap();
    let mut bad = row("bad");
    bad.as_object_mut().unwrap().remove("due_at");
    let mut contradiction = row("bad");
    contradiction["completed_at"] = json!("2026-09-09T11:00:00.000Z");
    let incompatible =
        json!({"version":"2.0","exported_at":later.to_string(),"tasks":[]}).to_string();
    for wire in [
        "{".to_string(),
        export(later, vec![row("new"), bad]),
        export(later, vec![row("duplicate"), row("duplicate")]),
        export(later, vec![contradiction]),
        incompatible,
    ] {
        assert!(store.import_json(&wire, later).is_err());
        assert_eq!(input(&store, later).task_refs, before);
        assert_ne!(
            input(&store, later).source_states[0].last_attempt_outcome,
            AttemptOutcome::Complete
        );
    }
}

#[test]
fn empty_complete_export_retires_but_reappearance_reuses_the_mapping() {
    let mut store = Store::memory().unwrap();
    store
        .import_json(&export(now(), vec![row("one")]), now())
        .unwrap();
    let original = input(&store, now()).task_refs[0].meta.id;
    let later = now().checked_add_ms(60_000).unwrap();
    store.import_json(&export(later, vec![]), later).unwrap();
    let tombstone = input(&store, later).task_refs[0].clone();
    assert_eq!(tombstone.meta.id, original);
    assert_eq!(tombstone.presence, Presence::Removed);
    let next = later.checked_add_ms(60_000).unwrap();
    store
        .import_json(&export(next, vec![row("one")]), next)
        .unwrap();
    assert_eq!(input(&store, next).task_refs[0].meta.id, original);
    assert_eq!(input(&store, next).task_refs[0].presence, Presence::Present);
}

#[test]
fn old_future_and_same_timestamp_conflicts_cannot_rewind_the_cache() {
    let mut store = Store::memory().unwrap();
    let first = now();
    let later = first.checked_add_ms(60_000).unwrap();
    store
        .import_json(&export(later, vec![row("keep")]), later)
        .unwrap();
    let before = input(&store, later).task_refs;
    for wire in [
        export(first, vec![]),
        export(later, vec![]),
        export(later.checked_add_ms(1).unwrap(), vec![]),
    ] {
        assert!(store.import_json(&wire, later).is_err());
        assert_eq!(input(&store, later).task_refs, before);
    }
}

#[test]
fn fractional_sort_order_becomes_a_stable_ordinal_without_truncation() {
    let mut store = Store::memory().unwrap();
    let mut a = row("a");
    a["sort_order"] = json!(1.9);
    let mut b = row("b");
    b["sort_order"] = json!(1.1);
    store
        .import_json(&export(now(), vec![a, b]), now())
        .unwrap();
    let data = input(&store, now());
    let order = |id| {
        data.task_refs
            .iter()
            .find(|t| t.provenance.imported().external_id == id)
            .unwrap()
            .source_sort_order
    };
    assert!(order("b") < order("a"));
}

#[test]
fn cache_survives_restart_and_refuses_future_schema_without_reset() {
    let path = std::env::temp_dir().join(format!(
        "temporal-synthetic-{}.sqlite3",
        uuid::Uuid::new_v4()
    ));
    let mut store = Store::open(&path).unwrap();
    store
        .import_json(&export(now(), vec![row("persist")]), now())
        .unwrap();
    let before = input(&store, now());
    drop(store);
    let reopened = Store::open(&path).unwrap();
    assert_eq!(input(&reopened, now()), before);
    drop(reopened);
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch("PRAGMA user_version = 2;")
        .unwrap();
    drop(connection);
    assert!(Store::open(&path).is_err());
    let connection = rusqlite::Connection::open(&path).unwrap();
    assert_eq!(
        connection
            .pragma_query_value(None, "user_version", |r| r.get::<_, i32>(0))
            .unwrap(),
        2
    );
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn a_foreign_database_is_left_unchanged() {
    let path = std::env::temp_dir().join(format!(
        "temporal-synthetic-{}.sqlite3",
        uuid::Uuid::new_v4()
    ));
    let connection = rusqlite::Connection::open(&path).unwrap();
    connection
        .execute_batch(
            "CREATE TABLE unrelated(value TEXT); INSERT INTO unrelated VALUES ('synthetic');",
        )
        .unwrap();
    assert!(Store::open(&path).is_err());
    assert_eq!(
        connection
            .query_row("SELECT value FROM unrelated", [], |r| r.get::<_, String>(0))
            .unwrap(),
        "synthetic"
    );
    assert_eq!(
        connection
            .pragma_query_value(None, "user_version", |r| r.get::<_, i32>(0))
            .unwrap(),
        0
    );
    drop(connection);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn a_write_failure_rolls_back_all_facts_and_records_failed_health() {
    let path = std::env::temp_dir().join(format!(
        "temporal-synthetic-{}.sqlite3",
        uuid::Uuid::new_v4()
    ));
    let mut store = Store::open(&path).unwrap();
    store
        .import_json(&export(now(), vec![row("keep")]), now())
        .unwrap();
    let before = input(&store, now());
    let connection = rusqlite::Connection::open(&path).unwrap();
    // The first INSERT succeeds; the next fails. Neither may survive rollback.
    connection.execute_batch("CREATE TRIGGER synthetic_failure BEFORE INSERT ON trace_tasks WHEN NEW.external_id = 'z-fail' BEGIN SELECT RAISE(FAIL, 'synthetic failure'); END;").unwrap();
    let later = now().checked_add_ms(60_000).unwrap();
    assert!(
        store
            .import_json(&export(later, vec![row("a-new"), row("z-fail")]), later)
            .is_err()
    );
    let after = input(&store, later);
    assert_eq!(after.task_refs, before.task_refs);
    assert_eq!(after.source_states[0].last_success_at, Some(now()));
    assert_eq!(
        after.source_states[0].last_attempt_outcome,
        AttemptOutcome::Failed
    );
    assert_eq!(after.source_states[0].last_attempt_at, Some(later));
    assert!(
        store
            .read(now(), "America/Toronto".parse().unwrap())
            .is_err()
    );
    drop(connection);
    drop(store);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn unsupported_envelopes_remain_incompatible_even_with_different_task_shapes() {
    let mut store = Store::memory().unwrap();
    for wire in [
        r#"{"version":"2.0","exported_at":"future-format","tasks":[{"title":"new format"}]}"#,
        r#"{"version":"1.0","exported_at":"2026-09-09T12:00:00.000Z","tasks":[],"filtered":true}"#,
    ] {
        assert!(store.import_json(wire, now()).is_err());
        let data = input(&store, now());
        assert!(data.task_refs.is_empty());
        assert_eq!(
            data.source_states[0].last_attempt_outcome,
            AttemptOutcome::Incompatible
        );
    }
}
