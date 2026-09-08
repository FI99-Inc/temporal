//! Whole-snapshot validation. Invalid records are never partially evaluated.

mod advice;
mod relationships;

use crate::clock::EvaluationTime;
use crate::domain::*;
use crate::time::{Instant, TemporalSpan, TimezoneRules};
use std::collections::{BTreeMap, BTreeSet};

#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum IssueCode {
    ArithmeticOverflow,
    DuplicateIdentity,
    InconsistentState,
    InvalidIdentity,
    InvalidRelationship,
    InvalidTime,
    InvalidValue,
    MissingReference,
    UnknownField,
    UnsupportedVersion,
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd, serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationIssue {
    pub path: String,
    pub code: IssueCode,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub record_id: Option<String>,
    pub related_ids: Vec<String>,
}
impl ValidationIssue {
    pub fn new(code: IssueCode, path: impl Into<String>) -> Self {
        let path = path.into();
        Self {
            path: if path == "." { String::new() } else { path },
            code,
            record_id: None,
            related_ids: vec![],
        }
    }
}

#[derive(Debug)]
pub struct ValidatedInput<'a>(&'a EvaluationInput);
impl<'a> ValidatedInput<'a> {
    pub fn input(&self) -> &'a EvaluationInput {
        self.0
    }
}

pub fn validate(input: &EvaluationInput) -> Result<ValidatedInput<'_>, Vec<ValidationIssue>> {
    let mut context = Context {
        input,
        issues: Vec::new(),
        identities: BTreeMap::new(),
        imports: BTreeMap::new(),
        rules: TimezoneRules::bundled(),
    };
    context.inventory();
    context.records();
    context.sources();
    relationships::validate(&mut context);
    advice::validate(&mut context);
    for issue in &mut context.issues {
        issue.related_ids.sort();
        issue.related_ids.dedup();
    }
    context.issues.sort();
    context.issues.dedup();
    if context.issues.is_empty() {
        Ok(ValidatedInput(input))
    } else {
        Err(context.issues)
    }
}

struct Context<'a> {
    input: &'a EvaluationInput,
    issues: Vec<ValidationIssue>,
    identities: BTreeMap<String, RecordId>,
    imports: BTreeMap<ImportedIdentity, String>,
    rules: TimezoneRules,
}

impl Context<'_> {
    fn issue(&mut self, code: IssueCode, path: impl Into<String>) {
        self.issues.push(ValidationIssue::new(code, path));
    }
    fn identity(&mut self, id: RecordId, path: String) {
        let raw = id.uuid_string();
        if self.identities.insert(raw.clone(), id).is_some() {
            let mut issue = ValidationIssue::new(IssueCode::DuplicateIdentity, path);
            issue.record_id = Some(raw);
            self.issues.push(issue);
        }
    }
    fn reference(&mut self, expected: RecordId, path: impl Into<String>) -> bool {
        let id = expected.uuid_string();
        let code = match self.identities.get(&id) {
            None => Some(IssueCode::MissingReference),
            Some(actual) if *actual != expected => Some(IssueCode::InvalidRelationship),
            Some(_) => None,
        };
        if let Some(code) = code {
            let mut issue = ValidationIssue::new(code, path);
            issue.related_ids.push(id);
            self.issues.push(issue);
            false
        } else {
            true
        }
    }
    fn recorded(&mut self, time: Instant, path: impl Into<String>) {
        if time > self.input.captured_now {
            self.issue(IssueCode::InvalidTime, path);
        }
    }
    fn audit(&mut self, created: Instant, updated: Instant, path: &str) {
        if created > updated {
            self.issue(IssueCode::InvalidTime, format!("{path}.created_at"));
        }
        self.recorded(updated, format!("{path}.updated_at"));
    }
    fn title(&mut self, title: &str, path: &str) {
        if title.trim().is_empty() {
            self.issue(IssueCode::InvalidValue, format!("{path}.title"));
        }
    }
    fn span(&mut self, span: &TemporalSpan, path: impl Into<String>) {
        if span.resolve(&self.rules).is_err() {
            self.issue(IssueCode::InvalidTime, path);
        }
    }
    fn work(&mut self, work: &WorkMetadata, path: &str) {
        if let (Effort::Estimate(minutes), Some(chunk)) = (work.effort, work.minimum_chunk_minutes)
            && (minutes == 0 || chunk.get() > minutes)
        {
            self.issue(
                IssueCode::InvalidValue,
                format!("{path}.minimum_chunk_minutes"),
            );
        }
    }
    fn local(&mut self, assertion: &LocalAssertion, path: &str) {
        self.reference(
            RecordId::Source(assertion.source_id),
            format!("{path}.source_id"),
        );
        if self
            .input
            .sources
            .iter()
            .any(|source| source.id == assertion.source_id && source.kind != SourceKind::Local)
        {
            self.issue(IssueCode::InvalidRelationship, format!("{path}.source_id"));
        }
        self.recorded(assertion.asserted_at, format!("{path}.asserted_at"));
    }
    fn imported(
        &mut self,
        imported: &ImportedProvenance,
        id: String,
        projections: &[Projection],
        trace_only: bool,
        path: &str,
    ) {
        self.reference(
            RecordId::Source(imported.source_id),
            format!("{path}.source_id"),
        );
        if self.input.sources.iter().any(|source| {
            source.id == imported.source_id
                && (source.kind == SourceKind::Local
                    || (trace_only && source.kind != SourceKind::Trace))
        }) {
            self.issue(IssueCode::InvalidRelationship, format!("{path}.source_id"));
        }
        if !projections.contains(&imported.projection) {
            self.issue(IssueCode::InvalidRelationship, format!("{path}.projection"));
        }
        if imported.external_id.is_empty() {
            self.issue(IssueCode::InvalidIdentity, format!("{path}.external_id"));
        }
        if imported
            .occurrence_key
            .as_ref()
            .is_some_and(String::is_empty)
        {
            self.issue(IssueCode::InvalidIdentity, format!("{path}.occurrence_key"));
        }
        if let Some(previous) = self.imports.insert(imported.identity(), id.clone()) {
            let mut issue = ValidationIssue::new(IssueCode::DuplicateIdentity, path);
            issue.record_id = Some(id);
            issue.related_ids.push(previous);
            self.issues.push(issue);
        }
        self.recorded(imported.observed_at, format!("{path}.observed_at"));
    }
    fn provenance(
        &mut self,
        provenance: &Provenance,
        id: String,
        projections: &[Projection],
        path: &str,
    ) {
        match provenance {
            Provenance::LocalUser(assertion) => self.local(assertion, &format!("{path}.value")),
            Provenance::Imported(imported) => {
                self.imported(imported, id, projections, false, &format!("{path}.value"))
            }
        }
    }
    fn inventory(&mut self) {
        let input = self.input;
        if input.schema_version != 1 {
            self.issue(IssueCode::UnsupportedVersion, "schema_version");
        }
        if EvaluationTime::new(
            input.captured_now,
            input.evaluation.now,
            input.evaluation.evaluation_end,
        )
        .is_err()
        {
            self.issue(IssueCode::InvalidTime, "evaluation");
        }
        if input.evaluation.timezone_rules_version != self.rules.version() {
            self.issue(IssueCode::InvalidValue, "evaluation.timezone_rules_version");
        }
        for (index, source) in input.sources.iter().enumerate() {
            self.identity(RecordId::Source(source.id), format!("sources[{index}].id"));
        }
        macro_rules! register {
            ($collection:ident, $kind:ident) => {
                for (index, record) in input.$collection.iter().enumerate() {
                    self.identity(
                        RecordId::$kind(record.meta.id),
                        format!("{}[{index}].meta.id", stringify!($collection)),
                    );
                }
            };
        }
        register!(anchors, Anchor);
        register!(deadlines, Deadline);
        register!(task_refs, TaskRef);
        register!(intentions, Intention);
        register!(routines, Routine);
        register!(availability, Availability);
    }
    fn records(&mut self) {
        let input = self.input;
        for (index, anchor) in input.anchors.iter().enumerate() {
            let path = format!("anchors[{index}]");
            self.title(&anchor.title, &path);
            self.audit(
                anchor.meta.created_at,
                anchor.meta.updated_at,
                &format!("{path}.meta"),
            );
            self.provenance(
                &anchor.provenance,
                anchor.meta.id.to_string(),
                &[Projection::Anchor],
                &format!("{path}.provenance"),
            );
            self.span(&anchor.span, format!("{path}.span"));
        }
        for (index, deadline) in input.deadlines.iter().enumerate() {
            let path = format!("deadlines[{index}]");
            self.title(&deadline.title, &path);
            self.audit(
                deadline.meta.created_at,
                deadline.meta.updated_at,
                &format!("{path}.meta"),
            );
            self.provenance(
                &deadline.provenance,
                deadline.meta.id.to_string(),
                &[Projection::Deadline, Projection::TaskDue],
                &format!("{path}.provenance"),
            );
            if deadline.cutoff.endpoint(&self.rules).is_err() {
                self.issue(IssueCode::InvalidTime, format!("{path}.cutoff"));
            }
        }
        for (index, task) in input.task_refs.iter().enumerate() {
            let path = format!("task_refs[{index}]");
            self.title(&task.title, &path);
            self.audit(
                task.meta.created_at,
                task.meta.updated_at,
                &format!("{path}.meta"),
            );
            self.imported(
                task.provenance.imported(),
                task.meta.id.to_string(),
                &[Projection::Task],
                true,
                &format!("{path}.provenance.value"),
            );
            let unknown = task.status == TaskStatus::Unknown;
            if (unknown
                && task
                    .unknown_status_label
                    .as_ref()
                    .is_none_or(String::is_empty))
                || (!unknown && task.unknown_status_label.is_some())
            {
                self.issue(
                    IssueCode::InconsistentState,
                    format!("{path}.unknown_status_label"),
                );
            }
            if matches!(
                task.status,
                TaskStatus::Now | TaskStatus::Later | TaskStatus::Someday
            ) && task.source_completed_at.is_some()
            {
                self.issue(
                    IssueCode::InconsistentState,
                    format!("{path}.source_completed_at"),
                );
            }
            if let TaskDue::Unresolved { value, reason } = &task.due
                && (value.is_empty() || reason.is_empty())
            {
                self.issue(IssueCode::InvalidValue, format!("{path}.due"));
            }
        }
        for (index, intention) in input.intentions.iter().enumerate() {
            let path = format!("intentions[{index}]");
            self.title(&intention.title, &path);
            self.audit(
                intention.meta.created_at,
                intention.meta.updated_at,
                &format!("{path}.meta"),
            );
            self.local(
                intention.provenance.assertion(),
                &format!("{path}.provenance.value"),
            );
            if let Some(span) = &intention.preferred_span {
                self.span(span, format!("{path}.preferred_span"));
            }
            if let Some(work) = &intention.work {
                self.work(work, &format!("{path}.work"));
            }
            if (intention.state == IntentionState::Active) == intention.state_evidence.is_some() {
                self.issue(
                    IssueCode::InconsistentState,
                    format!("{path}.state_evidence"),
                );
            }
            if let Some(evidence) = &intention.state_evidence {
                self.recorded(
                    evidence.recorded_at,
                    format!("{path}.state_evidence.recorded_at"),
                );
            }
        }
        for (index, routine) in input.routines.iter().enumerate() {
            let path = format!("routines[{index}]");
            self.title(&routine.title, &path);
            self.audit(
                routine.meta.created_at,
                routine.meta.updated_at,
                &format!("{path}.meta"),
            );
            self.local(
                routine.provenance.assertion(),
                &format!("{path}.provenance.value"),
            );
            self.work(&routine.work, &format!("{path}.work"));
            let RoutineRule::Weekly(rule) = &routine.rule;
            if rule.weekdays.is_empty() {
                self.issue(
                    IssueCode::InvalidValue,
                    format!("{path}.rule.value.weekdays"),
                );
            }
            if rule
                .until_date_exclusive
                .is_some_and(|end| end <= rule.start_date)
            {
                self.issue(
                    IssueCode::InvalidTime,
                    format!("{path}.rule.value.until_date_exclusive"),
                );
            }
        }
        for (index, declaration) in input.availability.iter().enumerate() {
            self.audit(
                declaration.meta.created_at,
                declaration.meta.updated_at,
                &format!("availability[{index}].meta"),
            );
            for other in &input.availability[..index] {
                if declaration.span.overlaps(&other.span) {
                    self.issue(
                        IssueCode::InvalidTime,
                        format!("availability[{index}].span"),
                    );
                }
            }
        }
    }
    fn sources(&mut self) {
        let input = self.input;
        let mut states = BTreeSet::new();
        for (index, state) in input.source_states.iter().enumerate() {
            let path = format!("source_states[{index}]");
            if !states.insert(state.source_id) {
                self.issue(IssueCode::DuplicateIdentity, &path);
            }
            self.reference(
                RecordId::Source(state.source_id),
                format!("{path}.source_id"),
            );
            if input
                .sources
                .iter()
                .any(|source| source.id == state.source_id && source.kind == SourceKind::Local)
            {
                self.issue(IssueCode::InvalidRelationship, &path);
            }
            if let Some(attempt) = state.last_attempt_at {
                self.recorded(attempt, format!("{path}.last_attempt_at"));
            }
            if let Some(success) = state.last_success_at {
                self.recorded(success, format!("{path}.last_success_at"));
                if state
                    .last_attempt_at
                    .is_none_or(|attempt| success > attempt)
                {
                    self.issue(
                        IssueCode::InconsistentState,
                        format!("{path}.last_success_at"),
                    );
                }
                if success.checked_add_ms(state.fresh_for_ms.get()).is_err() {
                    self.issue(
                        IssueCode::ArithmeticOverflow,
                        format!("{path}.fresh_for_ms"),
                    );
                }
            } else if state.anchor_coverage.is_some()
                || state.deadline_coverage.is_some()
                || state.tasks_complete
            {
                self.issue(IssueCode::InconsistentState, &path);
            }
            match state.last_attempt_outcome {
                AttemptOutcome::Never => {
                    if state.last_attempt_at.is_some()
                        || state.last_success_at.is_some()
                        || state.anchor_coverage.is_some()
                        || state.deadline_coverage.is_some()
                        || state.tasks_complete
                    {
                        self.issue(IssueCode::InconsistentState, &path);
                    }
                }
                AttemptOutcome::Complete => {
                    if state.last_success_at.is_none()
                        || state.last_attempt_at != state.last_success_at
                    {
                        self.issue(IssueCode::InconsistentState, &path);
                    }
                }
                AttemptOutcome::Partial | AttemptOutcome::Failed | AttemptOutcome::Incompatible => {
                    if state.last_attempt_at.is_none() {
                        self.issue(IssueCode::InconsistentState, &path);
                    }
                }
            }
        }
        for source in &input.sources {
            if source.kind != SourceKind::Local && !states.contains(&source.id) {
                self.issue(IssueCode::InconsistentState, "source_states");
            }
        }
        let mut required = BTreeSet::new();
        for (index, requirement) in input.required_sources.iter().enumerate() {
            let path = format!("required_sources[{index}]");
            self.reference(
                RecordId::Source(requirement.source_id),
                format!("{path}.source_id"),
            );
            if !required.insert(*requirement) {
                self.issue(IssueCode::DuplicateIdentity, &path);
            }
        }
    }
}
