//! Independent rejection fixtures from S16's cross-scenario matrix.
use super::{Case, bases};
use serde_json::{Value, json};
use temporal_core::validation::IssueCode;

pub struct RejectionCase {
    pub name: &'static str,
    pub json: String,
    pub expected: IssueCode,
}
fn invalid(
    base: &Case,
    name: &'static str,
    expected: IssueCode,
    change: impl FnOnce(&mut Value),
) -> RejectionCase {
    let mut value = serde_json::to_value(&base.input).unwrap();
    change(&mut value);
    RejectionCase {
        name,
        json: value.to_string(),
        expected,
    }
}

pub fn all() -> Vec<RejectionCase> {
    use IssueCode::*;
    let bases = bases::all();
    let b = |number| {
        bases
            .iter()
            .find(|case| case.name == format!("S{number:02}/base"))
            .unwrap()
    };
    let core = b(16);
    let work_path = "/deadline_annotations/0/work/value";
    let mut cases = vec![
        invalid(core, "schema_version", UnsupportedVersion, |value| {
            value["schema_version"] = json!(2)
        }),
        invalid(core, "unknown_field", UnknownField, |value| {
            value["unexpected"] = json!(true)
        }),
        invalid(core, "unknown_enum_field", UnknownField, |value| {
            value["deadline_annotations"][0]["work"]["value"]["unexpected"] = json!(true)
        }),
        invalid(core, "unknown_tag", InvalidValue, |value| {
            value["task_refs"][0]["status"] = json!("future_state")
        }),
        invalid(core, "duplicate_record_id", DuplicateIdentity, |value| {
            let record = value["deadlines"][0].clone();
            value["deadlines"].as_array_mut().unwrap().push(record);
        }),
        invalid(core, "nil_id", InvalidIdentity, |value| {
            value["deadlines"][0]["meta"]["id"] = json!("00000000-0000-0000-0000-000000000000")
        }),
        invalid(core, "malformed_id", InvalidIdentity, |value| {
            value["deadlines"][0]["meta"]["id"] = json!("bad-id")
        }),
        invalid(core, "duplicate_requirement", DuplicateIdentity, |value| {
            let source = value["required_sources"][0].clone();
            value["required_sources"]
                .as_array_mut()
                .unwrap()
                .push(source);
        }),
        invalid(core, "duplicate_context_set", DuplicateIdentity, |value| {
            value.pointer_mut(work_path).unwrap()["required_contexts"] = json!(["desk", "desk"])
        }),
        invalid(
            core,
            "duplicate_declared_context_set",
            DuplicateIdentity,
            |value| value["availability"][0]["contexts"]["value"] = json!(["desk", "desk"]),
        ),
        invalid(b(8), "duplicate_imported_key", DuplicateIdentity, |value| {
            let mut anchor = value["anchors"][0].clone();
            anchor["meta"]["id"] = json!("00000000-0000-4000-8000-008000000999");
            value["anchors"].as_array_mut().unwrap().push(anchor);
        }),
        invalid(core, "missing_task", MissingReference, |value| {
            value["task_refs"] = json!([])
        }),
        invalid(core, "missing_deadline", MissingReference, |value| {
            value["deadlines"] = json!([])
        }),
        invalid(core, "missing_source", MissingReference, |value| {
            value["sources"].as_array_mut().unwrap().remove(1);
        }),
        invalid(core, "wrong_species_link", InvalidRelationship, |value| {
            value["task_annotations"][0]["target_id"] = value["deadlines"][0]["meta"]["id"].clone()
        }),
        invalid(
            b(6),
            "conflicting_work_owners",
            InvalidRelationship,
            |value| {
                let task = value["task_refs"][0]["meta"]["id"].clone();
                let mut deadline = value["deadlines"][0].clone();
                deadline["meta"]["id"] = json!("00000000-0000-4000-8000-006000000007");
                deadline["provenance"] = json!({"kind":"local_user","value":{"source_id":value["sources"][0]["id"],"asserted_at":value["captured_now"]}});
                deadline["fulfillment"] = json!({"kind":"recorded","value":{"kind":"unresolved"}});
                let annotation = json!({"target_id":deadline["meta"]["id"],"revision":1,"created_at":value["captured_now"],"updated_at":value["captured_now"],"importance":"unspecified","milestone":false,"work":{"kind":"task","value":task}});
                value["deadlines"].as_array_mut().unwrap().push(deadline);
                value["deadline_annotations"]
                    .as_array_mut()
                    .unwrap()
                    .push(annotation);
            },
        ),
        invalid(core, "overlapping_availability", InvalidTime, |value| {
            let mut availability = value["availability"][0].clone();
            availability["meta"]["id"] = json!("00000000-0000-4000-8000-016000000004");
            value["availability"]
                .as_array_mut()
                .unwrap()
                .push(availability);
        }),
        invalid(core, "empty_span", InvalidTime, |value| {
            value["availability"][0]["span"]["end"] =
                value["availability"][0]["span"]["start"].clone()
        }),
        invalid(core, "reversed_span", InvalidTime, |value| {
            value["availability"][0]["span"]["end"] = json!("2026-09-07T12:00:00.000Z")
        }),
        invalid(core, "negative_effort", InvalidValue, |value| {
            value.pointer_mut(work_path).unwrap()["effort"]["value"] = json!(-1)
        }),
        invalid(core, "fractional_effort", InvalidValue, |value| {
            value.pointer_mut(work_path).unwrap()["effort"]["value"] = json!(1.5)
        }),
        invalid(core, "zero_chunk", InvalidValue, |value| {
            value.pointer_mut(work_path).unwrap()["minimum_chunk_minutes"] = json!(0)
        }),
        invalid(core, "chunk_exceeds_estimate", InvalidValue, |value| {
            value.pointer_mut(work_path).unwrap()["minimum_chunk_minutes"] = json!(61)
        }),
        invalid(core, "zero_with_chunk", InvalidValue, |value| {
            value.pointer_mut(work_path).unwrap()["effort"]["value"] = json!(0)
        }),
        invalid(core, "zero_lower_bound", InvalidValue, |value| {
            value.pointer_mut(work_path).unwrap()["effort"] = json!({"kind":"at_least","value":0})
        }),
        invalid(core, "null_required_field", InvalidValue, |value| {
            value["deadlines"][0]["title"] = Value::Null
        }),
        invalid(core, "null_optional_field", InvalidValue, |value| {
            value["task_refs"][0]["source_priority"] = Value::Null
        }),
        invalid(core, "u32_overflow", InvalidValue, |value| {
            value.pointer_mut(work_path).unwrap()["effort"]["value"] = json!(4_294_967_296u64)
        }),
        invalid(
            core,
            "source_expiry_overflow",
            ArithmeticOverflow,
            |value| value["source_states"][0]["fresh_for_ms"] = json!(u64::MAX),
        ),
        invalid(core, "evaluation_before_snapshot", InvalidTime, |value| {
            value["evaluation"]["now"] = json!("2026-09-07T12:59:59.999Z")
        }),
        invalid(core, "local_audit_after_snapshot", InvalidTime, |value| {
            value["deadlines"][0]["meta"]["updated_at"] = json!("2026-09-07T13:00:00.001Z")
        }),
        invalid(core, "missing_time", InvalidTime, |value| {
            value["deadlines"][0]["meta"]
                .as_object_mut()
                .unwrap()
                .remove("updated_at");
        }),
        invalid(core, "unresolved_zone", InvalidTime, |value| {
            value["evaluation"]["display_zone"] = json!("America/Invented")
        }),
        invalid(core, "unpinned_rules", InvalidValue, |value| {
            value["evaluation"]["timezone_rules_version"] = json!("host-current")
        }),
        invalid(
            b(13),
            "unfinished_with_completion_time",
            InconsistentState,
            |value| {
                value["task_refs"][0]["status"] = json!("now");
                value["task_refs"][0]["source_completed_at"] = value["captured_now"].clone();
            },
        ),
        invalid(
            b(13),
            "wrong_fulfillment_authority",
            InvalidRelationship,
            |value| value["deadlines"][1]["fulfillment"] = json!({"kind":"recorded","value":{"kind":"satisfied","value":{"authority":"local_user","recorded_at":value["captured_now"]}}}),
        ),
        invalid(
            b(12),
            "complete_without_success",
            InconsistentState,
            |value| {
                value["source_states"][0]
                    .as_object_mut()
                    .unwrap()
                    .remove("last_success_at");
            },
        ),
        invalid(b(12), "never_with_coverage", InconsistentState, |value| {
            let state = value["source_states"][0].as_object_mut().unwrap();
            state.insert("last_attempt_outcome".into(), json!("never"));
            state.remove("last_attempt_at");
            state.remove("last_success_at");
        }),
        invalid(b(14), "empty_weekdays", InvalidValue, |value| {
            value["routines"][0]["rule"]["value"]["weekdays"] = json!([])
        }),
        invalid(b(14), "duplicate_weekday", DuplicateIdentity, |value| {
            value["routines"][0]["rule"]["value"]["weekdays"] = json!(["sun", "sun"])
        }),
        invalid(b(14), "invalid_until_bound", InvalidTime, |value| {
            value["routines"][0]["rule"]["value"]["until_date_exclusive"] = json!("2026-08-30")
        }),
        invalid(b(14), "unsupported_recurrence", InvalidValue, |value| {
            value["routines"][0]["rule"]["kind"] = json!("rrule")
        }),
        invalid(b(10), "suggestion_overdue_field", UnknownField, |value| {
            value["prior_suggestions"][0]["overdue"] = json!(true)
        }),
        invalid(
            b(10),
            "suggestion_completion_field",
            UnknownField,
            |value| value["prior_suggestions"][0]["completed"] = json!(false),
        ),
        invalid(b(10), "inference_in_facts", InvalidValue, |value| {
            let suggestion = value["prior_suggestions"][0].clone();
            value["deadlines"].as_array_mut().unwrap().push(suggestion);
        }),
        invalid(core, "wrong_reason_payload", InvalidValue, |value| {
            value["prior_suggestions"][1]["reasons"][0]["payload"]["opportunity_ms"] = json!(0)
        }),
        invalid(core, "unknown_reason_field", UnknownField, |value| {
            value["prior_suggestions"][0]["reasons"][0]["payload"]["invented"] = json!(true)
        }),
        invalid(
            core,
            "suggestion_key_mismatch",
            InvalidRelationship,
            |value| value["prior_suggestions"][0]["key"]["kind"] = json!("risk_notice"),
        ),
    ];
    let original = serde_json::to_string(&core.input).unwrap();
    cases.push(RejectionCase {
        name: "duplicate_json_key",
        json: original.replacen(
            "\"schema_version\":1",
            "\"schema_version\":1,\"schema_version\":1",
            1,
        ),
        expected: InvalidValue,
    });
    cases.push(RejectionCase {
        name: "duplicate_nested_json_key",
        json: original.replacen(
            "\"title\":\"D1\"",
            "\"title\":\"D1\",\"title\":\"duplicate\"",
            1,
        ),
        expected: InvalidValue,
    });
    cases.push(RejectionCase {
        name: "u64_overflow",
        json: original.replacen(
            "\"snapshot_revision\":1",
            "\"snapshot_revision\":18446744073709551616",
            1,
        ),
        expected: InvalidValue,
    });
    cases
}
