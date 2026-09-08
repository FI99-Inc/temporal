//! Strict normalized JSON and deterministic canonical bytes, never a source parser.

use crate::domain::EvaluationInput;
use crate::results::{
    EvaluationKey, Reference, SuggestionKey, SuggestionKind, WindowKey, WorkTarget,
};
use crate::time::{Instant, LocalDate, TemporalSpan, ZoneId};
use crate::validation::{IssueCode, ValidationIssue, validate};
use serde::{
    Deserialize, Serialize,
    de::{self, MapAccess, SeqAccess, Visitor},
};
use serde_json::Value;
use std::{collections::BTreeSet, fmt};

struct StrictValue(Value);
impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct StrictVisitor;
        impl<'de> Visitor<'de> for StrictVisitor {
            type Value = StrictValue;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("JSON without duplicate object keys")
            }
            fn visit_bool<E: de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::Bool(value)))
            }
            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(StrictValue(value.into()))
            }
            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Self::Value, E> {
                Ok(StrictValue(value.into()))
            }
            fn visit_f64<E: de::Error>(self, _: f64) -> Result<Self::Value, E> {
                Err(E::custom("normalized numbers must be bounded integers"))
            }
            fn visit_str<E: de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::String(value.into())))
            }
            fn visit_string<E: de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(StrictValue(Value::String(value)))
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Err(E::custom("null is not a normalized value"))
            }
            fn visit_seq<A: SeqAccess<'de>>(
                self,
                mut sequence: A,
            ) -> Result<Self::Value, A::Error> {
                let mut values = Vec::new();
                while let Some(StrictValue(value)) = sequence.next_element()? {
                    values.push(value);
                }
                Ok(StrictValue(Value::Array(values)))
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut values = serde_json::Map::new();
                while let Some(key) = map.next_key::<String>()? {
                    if values.contains_key(&key) {
                        return Err(de::Error::custom("duplicate JSON key"));
                    }
                    let StrictValue(value) = map.next_value()?;
                    values.insert(key, value);
                }
                Ok(StrictValue(Value::Object(values)))
            }
        }
        deserializer.deserialize_any(StrictVisitor)
    }
}

pub fn decode_input(json: &str) -> Result<EvaluationInput, Vec<ValidationIssue>> {
    let mut deserializer = serde_json::Deserializer::from_str(json);
    let StrictValue(value) =
        serde_path_to_error::deserialize(&mut deserializer).map_err(|error| {
            vec![ValidationIssue::new(
                IssueCode::InvalidValue,
                error.path().to_string(),
            )]
        })?;
    deserializer
        .end()
        .map_err(|_| vec![ValidationIssue::new(IssueCode::InvalidValue, "")])?;
    if let Some(version) = value.get("schema_version")
        && version != &Value::from(1)
    {
        return Err(vec![ValidationIssue::new(
            IssueCode::UnsupportedVersion,
            "schema_version",
        )]);
    }
    let mut issues = Vec::new();
    preflight_sets(&value, "", "", &mut issues);
    for collection in [
        "anchors",
        "deadlines",
        "task_refs",
        "intentions",
        "routines",
    ] {
        if let Some(records) = value.get(collection).and_then(Value::as_array) {
            for (index, record) in records.iter().enumerate() {
                if matches!(
                    record.get("kind").and_then(Value::as_str),
                    Some("consider_work" | "risk_notice")
                ) {
                    issues.push(ValidationIssue::new(
                        IssueCode::InvalidValue,
                        format!("{collection}[{index}]"),
                    ));
                }
            }
        }
    }
    if !issues.is_empty() {
        issues.sort();
        return Err(issues);
    }
    let input: EvaluationInput = serde_path_to_error::deserialize(value).map_err(|error| {
        let message = error.inner().to_string();
        let mut path = error.path().to_string();
        let code = if message.contains("unknown field") {
            IssueCode::UnknownField
        } else if message.contains("UUIDv4") {
            IssueCode::InvalidIdentity
        } else if message.contains("normalized time")
            || message.contains("years 0001")
            || message.contains("IANA timezone")
            || message.contains("interval must")
        {
            IssueCode::InvalidTime
        } else {
            IssueCode::InvalidValue
        };
        if (message.starts_with("unknown field `") || message.starts_with("missing field `"))
            && let Some(field) = message.split('`').nth(1)
        {
            path = child_path(&path, field);
        }
        let code = if message.starts_with("missing field")
            && path.rsplit('.').next().is_some_and(is_time_field)
        {
            IssueCode::InvalidTime
        } else {
            code
        };
        vec![ValidationIssue::new(code, path)]
    })?;
    validate(&input)?;
    Ok(input)
}

fn is_time_field(field: &str) -> bool {
    matches!(
        field,
        "created_at"
            | "updated_at"
            | "asserted_at"
            | "observed_at"
            | "recorded_at"
            | "confirmed_at"
            | "now"
            | "captured_now"
            | "evaluation_end"
            | "start"
            | "end"
            | "start_date"
            | "end_date_exclusive"
            | "instant"
            | "date"
            | "zone"
            | "span"
    )
}

fn child_path(path: &str, name: &str) -> String {
    if path.is_empty() || path == "." {
        name.into()
    } else {
        format!("{path}.{name}")
    }
}

fn preflight_sets(value: &Value, path: &str, field: &str, issues: &mut Vec<ValidationIssue>) {
    match value {
        Value::Array(values) => {
            if matches!(
                field,
                "required_contexts" | "weekdays" | "limitations" | "contexts_known"
            ) {
                let mut seen = BTreeSet::new();
                for (index, value) in values.iter().enumerate() {
                    if !seen.insert(value.to_string()) {
                        issues.push(ValidationIssue::new(
                            IssueCode::DuplicateIdentity,
                            format!("{path}[{index}]"),
                        ));
                    }
                }
            }
            for (index, value) in values.iter().enumerate() {
                preflight_sets(value, &format!("{path}[{index}]"), "", issues);
            }
        }
        Value::Object(fields) => {
            for (key, child) in fields {
                let child_field = if key == "value"
                    && fields.get("kind").and_then(Value::as_str) == Some("known")
                    && child.is_array()
                {
                    "contexts_known"
                } else {
                    key
                };
                preflight_sets(child, &child_path(path, key), child_field, issues);
            }
        }
        _ => {}
    }
}

/// Input collections and derived sets are ordered by their contract keys.
/// JSON's UTF-8/escaping/integer encoding is provided by the pinned serializer.
pub fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, serde_json::Error> {
    let mut value = serde_json::to_value(value)?;
    canonicalize(&mut value, "");
    serde_json::to_vec(&value)
}

fn canonicalize(value: &mut Value, field: &str) {
    match value {
        Value::Object(fields) => {
            for (name, child) in fields.iter_mut() {
                canonicalize(child, name);
            }
        }
        Value::Array(values) => {
            for value in values.iter_mut() {
                canonicalize(value, "");
            }
            if field == "weekdays" {
                values.sort_by_key(|value| {
                    ["mon", "tue", "wed", "thu", "fri", "sat", "sun"]
                        .iter()
                        .position(|day| value.as_str() == Some(day))
                });
            } else {
                values.sort_by_key(|value| array_key(field, value));
            }
            if field == "reasons" || field == "references" {
                values.dedup();
            }
        }
        _ => {}
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum SpanOrder {
    Dates(LocalDate, LocalDate, ZoneId),
    Timed(Instant, Instant, Option<ZoneId>),
}
impl From<TemporalSpan> for SpanOrder {
    fn from(span: TemporalSpan) -> Self {
        match span {
            TemporalSpan::Dates(span) => {
                Self::Dates(span.start_date(), span.end_date_exclusive(), span.zone())
            }
            TemporalSpan::Timed(span) => {
                Self::Timed(span.start(), span.end(), span.original_zone())
            }
        }
    }
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
enum ArrayKey {
    Reason(String, Vec<Reference>, String),
    Reference(Reference),
    Window(WindowKey),
    Fit(WorkTarget, WindowKey),
    Suggestion(EvaluationKey, SuggestionKind, WorkTarget, Option<SpanOrder>),
    Fields(Vec<String>),
}

fn parsed<T: serde::de::DeserializeOwned>(value: Option<&Value>) -> Option<T> {
    value.and_then(|value| serde_json::from_value(value.clone()).ok())
}

fn array_key(field: &str, value: &Value) -> ArrayKey {
    let typed = match field {
        "reasons" => parsed::<crate::reasons::Reason>(Some(value)).map(|reason| {
            ArrayKey::Reason(
                reason.code().as_str().into(),
                reason.references().to_vec(),
                value["payload"].to_string(),
            )
        }),
        "references" => parsed(Some(value)).map(ArrayKey::Reference),
        "window_keys" => parsed(Some(value)).map(ArrayKey::Window),
        "windows" => parsed(value.get("key")).map(ArrayKey::Window),
        "fits" => parsed(value.get("target"))
            .zip(parsed(value.get("window_key")))
            .map(|(target, window)| ArrayKey::Fit(target, window)),
        "suggestion_validity" | "prior_suggestions" => parsed::<SuggestionKey>(value.get("key"))
            .map(|key| {
                ArrayKey::Suggestion(
                    key.evaluation_key,
                    key.kind,
                    key.target,
                    key.proposed_span.map(SpanOrder::from),
                )
            }),
        _ => None,
    };
    if let Some(key) = typed {
        return key;
    }
    let paths: &[&str] = match field {
        "anchors" | "deadlines" | "task_refs" | "intentions" | "routines" | "availability" => {
            &["/meta/id"]
        }
        "sources" | "anchor_states" | "deadline_states" | "intention_states" => &["/id"],
        "source_states" | "source_health" => &["/source_id"],
        "anchor_annotations" | "deadline_annotations" | "task_annotations" => &["/target_id"],
        "required_sources" | "source_qualifications" => &["/source_id", "/role"],
        "routine_outcomes" => &["/routine_id", "/date"],
        "routine_occurrences" => &["/key/routine_id", "/key/date"],
        "pressures" => &["/endpoint", "/deadline_id"],
        "windows" => &[
            "/key/evaluation_key",
            "/key/availability_id",
            "/key/start",
            "/key/end",
        ],
        "window_keys" => &["/evaluation_key", "/availability_id", "/start", "/end"],
        "fits" => &[
            "/target",
            "/window_key/evaluation_key",
            "/window_key/availability_id",
            "/window_key/start",
            "/window_key/end",
        ],
        "suggestion_validity" | "prior_suggestions" => &["/key"],
        "conflicts" => &["/anchor_ids"],
        "reasons" => &["/code", "/references", "/payload"],
        _ => &[],
    };
    if paths.is_empty() {
        ArrayKey::Fields(vec![value.to_string()])
    } else {
        ArrayKey::Fields(
            paths
                .iter()
                .map(|path| {
                    value
                        .pointer(path)
                        .map(Value::to_string)
                        .unwrap_or_default()
                })
                .collect(),
        )
    }
}

pub(crate) fn compare_suggestion_keys(
    left: &SuggestionKey,
    right: &SuggestionKey,
) -> std::cmp::Ordering {
    let key = |value: &SuggestionKey| {
        (
            value.evaluation_key.clone(),
            value.kind,
            value.target,
            value.proposed_span.clone().map(SpanOrder::from),
        )
    };
    key(left).cmp(&key(right))
}
