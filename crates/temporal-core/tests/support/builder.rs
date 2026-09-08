use std::{
    collections::BTreeSet,
    num::{NonZeroU32, NonZeroU64},
    str::FromStr,
};
use temporal_core::{domain::*, results::*, time::*};

#[derive(Clone, Debug)]
pub struct Case {
    pub name: String,
    pub input: EvaluationInput,
}

pub fn nz(value: u64) -> NonZeroU64 {
    NonZeroU64::new(value).unwrap()
}
pub fn source_id(kind: SourceKind) -> SourceId {
    let ordinal = match kind {
        SourceKind::Local => 1,
        SourceKind::Trace => 2,
        SourceKind::Google => 3,
        SourceKind::Quercus => 4,
        SourceKind::Outlook => 5,
    };
    format!("10000000-0000-4000-8000-{ordinal:012}")
        .parse()
        .unwrap()
}
pub fn at(day: &str, clock: &str) -> Instant {
    let clock = if clock.len() == 5 {
        format!("{clock}:00.000")
    } else {
        clock.into()
    };
    TimezoneRules::bundled()
        .resolve_local(
            format!("{day}T{clock}").parse().unwrap(),
            "America/Toronto".parse().unwrap(),
            LocalResolution::RejectAmbiguous,
        )
        .unwrap()
}
pub fn span(day: &str, start: &str, end: &str) -> TimedSpan {
    TimedSpan::new(at(day, start), at(day, end))
        .unwrap()
        .with_original_zone("America/Toronto".parse().unwrap())
}
pub fn date_span(start: &str, end: &str) -> TemporalSpan {
    TemporalSpan::Dates(
        DateSpan::new(
            start.parse().unwrap(),
            end.parse().unwrap(),
            "America/Toronto".parse().unwrap(),
        )
        .unwrap(),
    )
}
pub fn work(minutes: u32, chunk: u32) -> WorkMetadata {
    WorkMetadata {
        effort: Effort::Estimate(minutes),
        minimum_chunk_minutes: NonZeroU32::new(chunk),
        ..WorkMetadata::default()
    }
}
pub fn unknown_work(chunk: u32) -> WorkMetadata {
    WorkMetadata {
        minimum_chunk_minutes: NonZeroU32::new(chunk),
        ..WorkMetadata::default()
    }
}

pub struct Builder {
    pub input: EvaluationInput,
    scenario: u32,
    ordinal: u32,
}
impl Builder {
    pub fn new(scenario: u32, now: &str, kinds: &[SourceKind]) -> Self {
        let now: Instant = now.parse().unwrap();
        let audit = now.checked_sub_ms(1).unwrap();
        let end = now.checked_add_ms(14 * 86_400_000).unwrap();
        let coverage = TimedSpan::new(
            now.checked_sub_ms(7 * 86_400_000).unwrap(),
            end.checked_add_ms(86_400_000).unwrap(),
        )
        .unwrap();
        let mut input = EvaluationInput {
            schema_version: 1,
            snapshot_revision: nz(1),
            captured_now: now,
            sources: vec![],
            source_states: vec![],
            required_sources: vec![],
            anchors: vec![],
            deadlines: vec![],
            task_refs: vec![],
            intentions: vec![],
            routines: vec![],
            anchor_annotations: vec![],
            deadline_annotations: vec![],
            task_annotations: vec![],
            routine_outcomes: vec![],
            availability: vec![],
            evaluation: EvaluationRequest {
                now,
                evaluation_end: end,
                display_zone: "America/Toronto".parse().unwrap(),
                timezone_rules_version: "2025b".into(),
                policy_version: PolicyVersion::ProofV1,
            },
            prior_suggestions: vec![],
        };
        for &kind in kinds {
            let id = source_id(kind);
            let (label, role) = match kind {
                SourceKind::Local => ("Synthetic local", None),
                SourceKind::Trace => ("Synthetic Trace", Some(SourceRole::Tasks)),
                SourceKind::Google => ("Synthetic calendar", Some(SourceRole::Anchors)),
                SourceKind::Quercus => ("Synthetic coursework", Some(SourceRole::Deadlines)),
                SourceKind::Outlook => ("Synthetic Outlook", Some(SourceRole::Anchors)),
            };
            input.sources.push(Source {
                id,
                kind,
                label: label.into(),
            });
            if let Some(role) = role {
                input.source_states.push(SourceState {
                    source_id: id,
                    last_attempt_outcome: AttemptOutcome::Complete,
                    fresh_for_ms: nz(14 * 86_400_000),
                    last_attempt_at: Some(audit),
                    last_success_at: Some(audit),
                    anchor_coverage: (role == SourceRole::Anchors).then(|| coverage.clone()),
                    deadline_coverage: (role == SourceRole::Deadlines).then(|| coverage.clone()),
                    tasks_complete: role == SourceRole::Tasks,
                });
                input.required_sources.push(RequiredSource {
                    source_id: id,
                    role,
                });
            }
        }
        Self {
            input,
            scenario,
            ordinal: 0,
        }
    }
    pub fn audit(&self) -> Instant {
        self.input.captured_now.checked_sub_ms(1).unwrap()
    }
    pub fn meta<I: FromStr>(&mut self) -> RecordMeta<I>
    where
        I::Err: std::fmt::Debug,
    {
        self.ordinal += 1;
        RecordMeta {
            id: format!(
                "00000000-0000-4000-8000-{:03}{:09}",
                self.scenario, self.ordinal
            )
            .parse()
            .unwrap(),
            revision: nz(1),
            created_at: self.audit(),
            updated_at: self.audit(),
        }
    }
    pub fn imported(
        &self,
        alias: &str,
        kind: SourceKind,
        projection: Projection,
    ) -> ImportedProvenance {
        ImportedProvenance {
            source_id: source_id(kind),
            external_id: format!("synthetic-{alias}"),
            occurrence_key: None,
            projection,
            observed_at: self.audit(),
            source_revision: Some("synthetic-1".into()),
            source_created_at: None,
            source_updated_at: None,
        }
    }
    pub fn local(&self) -> LocalAssertion {
        LocalAssertion {
            source_id: source_id(SourceKind::Local),
            asserted_at: self.audit(),
        }
    }
    pub fn provenance(&self, alias: &str, kind: SourceKind, projection: Projection) -> Provenance {
        if kind == SourceKind::Local {
            Provenance::LocalUser(self.local())
        } else {
            Provenance::Imported(self.imported(alias, kind, projection))
        }
    }
    pub fn anchor(&mut self, alias: &str, span: TemporalSpan, kind: SourceKind) -> AnchorId {
        let meta = self.meta();
        let id = meta.id;
        self.input.anchors.push(Anchor {
            meta,
            title: alias.into(),
            presence: Presence::Present,
            provenance: self.provenance(alias, kind, Projection::Anchor),
            span,
            rigidity: AnchorRigidity::Fixed,
            reported_certainty: ReportedCertainty::Confirmed,
            occupancy: Occupancy::Busy,
            location: None,
        });
        id
    }
    pub fn deadline(
        &mut self,
        alias: &str,
        cutoff: Cutoff,
        work: DeadlineWork,
        kind: SourceKind,
    ) -> DeadlineId {
        let meta = self.meta();
        let id = meta.id;
        self.input.deadlines.push(Deadline {
            meta,
            title: alias.into(),
            presence: Presence::Present,
            provenance: self.provenance(alias, kind, Projection::Deadline),
            cutoff,
            fulfillment: Fulfillment::Recorded(RecordedResolution::Unresolved),
        });
        if work != DeadlineWork::Unspecified {
            self.input.deadline_annotations.push(DeadlineAnnotation {
                target_id: id,
                revision: nz(1),
                created_at: self.audit(),
                updated_at: self.audit(),
                importance: Importance::Unspecified,
                milestone: false,
                work,
                confirmation: None,
            });
        }
        id
    }
    pub fn local_deadline(
        &mut self,
        alias: &str,
        day: &str,
        clock: &str,
        work: WorkMetadata,
    ) -> DeadlineId {
        self.deadline(
            alias,
            Cutoff::At {
                instant: at(day, clock),
                original_zone: Some("America/Toronto".parse().unwrap()),
            },
            DeadlineWork::Standalone(work),
            SourceKind::Local,
        )
    }
    pub fn task(&mut self, alias: &str, status: TaskStatus, work: WorkMetadata) -> TaskRefId {
        let meta = self.meta();
        let id = meta.id;
        self.input.task_refs.push(TaskRef {
            meta,
            title: alias.into(),
            presence: Presence::Present,
            provenance: TaskProvenance::Imported(self.imported(
                alias,
                SourceKind::Trace,
                Projection::Task,
            )),
            status,
            due: TaskDue::None,
            raw_input: None,
            unknown_status_label: None,
            source_priority: None,
            source_context: None,
            source_sort_order: None,
            source_link: None,
            source_completed_at: None,
        });
        self.input.task_annotations.push(TaskAnnotation {
            target_id: id,
            revision: nz(1),
            created_at: self.audit(),
            updated_at: self.audit(),
            importance: Importance::Unspecified,
            work,
        });
        id
    }
    pub fn due(&mut self, alias: &str, task_id: TaskRefId, day: &str, clock: &str) -> DeadlineId {
        let meta = self.meta();
        let id = meta.id;
        let task = self
            .input
            .task_refs
            .iter_mut()
            .find(|task| task.meta.id == task_id)
            .unwrap();
        task.due = TaskDue::Deadline(id);
        let mut imported = task.provenance.imported().clone();
        imported.projection = Projection::TaskDue;
        self.input.deadlines.push(Deadline {
            meta,
            title: alias.into(),
            presence: Presence::Present,
            provenance: Provenance::Imported(imported),
            cutoff: Cutoff::At {
                instant: at(day, clock),
                original_zone: Some("America/Toronto".parse().unwrap()),
            },
            fulfillment: Fulfillment::TraceTask(task_id),
        });
        id
    }
    pub fn intention(
        &mut self,
        alias: &str,
        preferred_span: Option<TemporalSpan>,
        work: Option<WorkMetadata>,
    ) -> IntentionId {
        let meta = self.meta();
        let id = meta.id;
        self.input.intentions.push(Intention {
            meta,
            title: alias.into(),
            presence: Presence::Present,
            provenance: UserProvenance::LocalUser(self.local()),
            preferred_span,
            work,
            state: IntentionState::Active,
            state_evidence: None,
        });
        id
    }
    pub fn availability(&mut self, span: TimedSpan) -> AvailabilityId {
        let meta = self.meta();
        let id = meta.id;
        self.input.availability.push(AvailabilityDeclaration {
            meta,
            span,
            contexts: DeclaredContexts::Known(BTreeSet::from(["desk".parse().unwrap()])),
            energy_capacity: EnergyCapacity::Deep,
        });
        id
    }
    pub fn finish(self) -> Case {
        Case {
            name: format!("S{:02}/base", self.scenario),
            input: self.input,
        }
    }
}

pub fn work_suggestion(
    input: &EvaluationInput,
    target: WorkTarget,
    proposed: TemporalSpan,
    valid_until: Instant,
    availability_index: usize,
) -> Suggestion {
    use temporal_core::reasons::{AvailabilityPayload, Reason};
    let producing = input.evaluation.key(input.snapshot_revision);
    let availability = &input.availability[availability_index];
    let window = WindowKey {
        evaluation_key: producing.clone(),
        availability_id: availability.meta.id,
        start: availability.span.start(),
        end: availability.span.end(),
    };
    Suggestion {
        key: SuggestionKey {
            evaluation_key: producing,
            kind: SuggestionKind::ConsiderWork,
            target,
            proposed_span: Some(proposed.clone()),
        },
        kind: SuggestionKind::ConsiderWork,
        target,
        proposed_span: Some(proposed),
        created_at: input.evaluation.now,
        valid_until,
        reasons: vec![Reason::DeclaredAvailability {
            references: vec![
                Reference::Availability(availability.meta.id),
                Reference::Window(window),
            ],
            payload: AvailabilityPayload {
                span: availability.span.clone(),
                clipped_span: Some(availability.span.clone()),
            },
        }],
    }
}
