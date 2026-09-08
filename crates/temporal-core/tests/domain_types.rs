use std::collections::BTreeSet;
use std::num::{NonZeroU32, NonZeroU64};
use temporal_core::domain::*;
use temporal_core::results::*;
use temporal_core::time::*;

fn id<T: std::str::FromStr>(ordinal: u32) -> T
where
    T::Err: std::fmt::Debug,
{
    format!("00000000-0000-4000-8000-{ordinal:012}")
        .parse()
        .unwrap()
}
fn instant() -> Instant {
    "2026-09-07T13:00:00.000Z".parse().unwrap()
}
fn revision() -> NonZeroU64 {
    NonZeroU64::new(1).unwrap()
}
fn meta<I>(id: I) -> RecordMeta<I> {
    RecordMeta {
        id,
        revision: revision(),
        created_at: instant(),
        updated_at: instant(),
    }
}

#[test]
fn identifiers_accept_only_canonical_v4_values_and_preserve_species() {
    let anchor: AnchorId = id(1);
    assert_eq!(anchor.to_string(), "00000000-0000-4000-8000-000000000001");
    for invalid in [
        "00000000-0000-0000-0000-000000000000",
        "00000000-0000-1000-8000-000000000001",
        "00000000-0000-4000-0000-000000000001",
        "00000000-0000-4000-8000-00000000000A",
        "00000000000040008000000000000001",
        "not-an-id",
    ] {
        assert!(invalid.parse::<AnchorId>().is_err(), "{invalid}");
    }
    assert_ne!(RecordId::Anchor(anchor), RecordId::Deadline(id(1)));
    assert_eq!(
        id::<TaskRefId>(2).to_string(),
        id::<DeadlineId>(2).to_string()
    );
    // The validator checks cross-species collisions; the Rust types retain both species.
}

#[test]
fn source_identity_is_opaque_and_distinguishes_due_projections() {
    let imported = ImportedProvenance {
        source_id: id(10),
        external_id: "Task/A".into(),
        occurrence_key: None,
        projection: Projection::Task,
        observed_at: instant(),
        source_revision: None,
        source_created_at: None,
        source_updated_at: None,
    };
    let mut due = imported.clone();
    due.projection = Projection::TaskDue;
    assert_ne!(imported.identity(), due.identity());
    due.projection = Projection::Task;
    due.external_id = "task/a".into();
    assert_ne!(imported.identity(), due.identity());
    due.external_id = imported.external_id.clone();
    due.source_id = id(11);
    assert_ne!(imported.identity(), due.identity());
}

#[test]
fn user_confirmation_does_not_rewrite_imported_certainty_or_revision() {
    let anchor = Anchor {
        meta: meta(id(1)),
        title: "Synthetic class".into(),
        presence: Presence::Present,
        provenance: Provenance::Imported(ImportedProvenance {
            source_id: id(10),
            external_id: "class/1".into(),
            occurrence_key: Some("2026-09-07".into()),
            projection: Projection::Anchor,
            observed_at: instant(),
            source_revision: None,
            source_created_at: None,
            source_updated_at: None,
        }),
        span: TemporalSpan::Dates(
            DateSpan::new(
                "2026-09-07".parse().unwrap(),
                "2026-09-08".parse().unwrap(),
                "America/Toronto".parse().unwrap(),
            )
            .unwrap(),
        ),
        rigidity: AnchorRigidity::Fixed,
        reported_certainty: ReportedCertainty::Tentative,
        occupancy: Occupancy::Busy,
        location: Some("campus".parse().unwrap()),
    };
    let confirmation = Confirmation {
        confirmed_at: instant(),
        record_revision: revision(),
    };
    assert!(confirmation.is_current(anchor.meta.revision));
    assert!(!confirmation.is_current(NonZeroU64::new(2).unwrap()));
    assert_eq!(anchor.reported_certainty, ReportedCertainty::Tentative);
    assert_eq!(anchor.provenance.writer(), FactWriter::Source(id(10)));
    let annotation = AnchorAnnotation {
        target_id: anchor.meta.id,
        revision: revision(),
        created_at: instant(),
        updated_at: instant(),
        importance: Importance::High,
        milestone: true,
        confirmation: Some(confirmation),
    };
    assert_eq!(annotation.target_id, anchor.meta.id);
    assert!(annotation.milestone); // Significance has no second identity or workload.
}

#[test]
fn unknown_work_and_context_are_not_zero_or_empty_claims() {
    let unknown = WorkMetadata::default();
    assert_eq!(unknown.effort, Effort::Unknown);
    assert!(unknown.minimum_chunk_minutes.is_none());
    assert_eq!(unknown.energy_requirement, EnergyRequirement::Unrestricted);
    assert_ne!(Effort::Unknown, Effort::Estimate(0));
    assert_ne!(
        Effort::AtLeast(NonZeroU32::new(240).unwrap()),
        Effort::Estimate(240)
    );
    assert_ne!(
        DeclaredContexts::Unknown,
        DeclaredContexts::Known(BTreeSet::new())
    );
    for invalid in ["", "Desk", "home office", "café"] {
        assert!(invalid.parse::<ContextTag>().is_err());
    }
    assert_eq!(
        "campus-north".parse::<ContextTag>().unwrap().as_str(),
        "campus-north"
    );
}

#[test]
fn trace_state_and_independent_deadline_fulfillment_remain_separate() {
    let task = TaskRef {
        meta: meta(id(2)),
        title: "Synthetic assignment".into(),
        presence: Presence::Present,
        provenance: TaskProvenance::Imported(ImportedProvenance {
            source_id: id(10),
            external_id: "assignment/1".into(),
            occurrence_key: None,
            projection: Projection::Task,
            observed_at: instant(),
            source_revision: None,
            source_created_at: None,
            source_updated_at: None,
        }),
        status: TaskStatus::Done,
        due: TaskDue::Deadline(id(3)),
        raw_input: None,
        unknown_status_label: None,
        source_priority: Some("opaque-high".into()),
        source_context: Some("School".into()),
        source_sort_order: Some(-3),
        source_link: None,
        source_completed_at: None,
    };
    let source_due = Fulfillment::TraceTask(task.meta.id);
    let independent = Fulfillment::Recorded(RecordedResolution::Unresolved);
    assert_ne!(source_due, independent);
    assert_eq!(DeadlineWork::Task(task.meta.id), DeadlineWork::Task(id(2)));
    assert!(task.source_completed_at.is_none());
    assert_eq!(task.status, TaskStatus::Done);
    assert_ne!(TaskStatus::Now, TaskStatus::Later);
    assert_ne!(TaskStatus::Later, TaskStatus::Someday);
    assert_ne!(
        TaskDue::None,
        TaskDue::Unresolved {
            value: "Friday-ish".into(),
            reason: "meaning not established".into()
        }
    );
}

#[test]
fn soft_user_state_and_occurrence_keys_keep_their_own_types() {
    let provenance = UserProvenance::LocalUser(LocalAssertion {
        source_id: id(10),
        asserted_at: instant(),
    });
    let intention = Intention {
        meta: meta(id(4)),
        title: "Read".into(),
        presence: Presence::Present,
        provenance: provenance.clone(),
        preferred_span: None,
        work: None,
        state: IntentionState::Active,
        state_evidence: None,
    };
    let routine = Routine {
        meta: meta(id(5)),
        title: "Weekly review".into(),
        presence: Presence::Present,
        provenance,
        state: RoutineState::Active,
        work: WorkMetadata::default(),
        rule: RoutineRule::Weekly(WeeklyRule {
            weekdays: BTreeSet::from([Weekday::Mon, Weekday::Sun]),
            start_date: "2026-09-07".parse().unwrap(),
            until_date_exclusive: None,
            zone: "America/Toronto".parse().unwrap(),
        }),
    };
    let key = OccurrenceKey {
        routine_id: routine.meta.id,
        date: "2026-09-07".parse().unwrap(),
    };
    assert_ne!(
        WorkTarget::Intention(intention.meta.id),
        WorkTarget::RoutineOccurrence(key)
    );
    let outcome = RoutineOutcome {
        routine_id: routine.meta.id,
        date: key.date,
        outcome: Outcome::Skipped,
        revision: revision(),
        recorded_at: instant(),
    };
    assert_eq!(outcome.key(), key);
    assert_eq!(routine.state, RoutineState::Active);
}

#[test]
fn evaluation_basis_excludes_only_the_advancing_clock() {
    let request = EvaluationRequest {
        now: instant(),
        evaluation_end: instant().checked_add_ms(86_400_000).unwrap(),
        display_zone: "America/Toronto".parse().unwrap(),
        timezone_rules_version: "2025b".into(),
        policy_version: PolicyVersion::ProofV1,
    };
    let first = request.key(revision());
    let mut later = request.clone();
    later.now = instant().checked_add_ms(1000).unwrap();
    assert_ne!(first, later.key(revision()));
    assert_eq!(first.basis(), later.key(revision()).basis());
    assert_ne!(
        first.basis(),
        request.key(NonZeroU64::new(2).unwrap()).basis()
    );
    later.display_zone = "UTC".parse().unwrap();
    assert_ne!(first.basis(), later.key(revision()).basis());
}

#[test]
fn suggestions_and_windows_carry_derived_keys_and_typed_evidence() {
    use temporal_core::reasons::{EmptyPayload, Reason, ReasonCode};
    let deadline = Deadline {
        meta: meta(id(3)),
        title: "Synthetic deadline".into(),
        presence: Presence::Present,
        provenance: Provenance::LocalUser(LocalAssertion {
            source_id: id(10),
            asserted_at: instant(),
        }),
        cutoff: Cutoff::At {
            instant: instant().checked_add_ms(3_600_000).unwrap(),
            original_zone: None,
        },
        fulfillment: Fulfillment::Recorded(RecordedResolution::Unresolved),
    };
    let declaration = AvailabilityDeclaration {
        meta: meta(id(6)),
        span: TimedSpan::new(instant(), instant().checked_add_ms(3_600_000).unwrap()).unwrap(),
        contexts: DeclaredContexts::Unknown,
        energy_capacity: EnergyCapacity::Unknown,
    };
    let key = EvaluationRequest {
        now: instant(),
        evaluation_end: declaration.span.end(),
        display_zone: "UTC".parse().unwrap(),
        timezone_rules_version: "2025b".into(),
        policy_version: PolicyVersion::ProofV1,
    }
    .key(revision());
    let window_key = WindowKey {
        evaluation_key: key.clone(),
        availability_id: declaration.meta.id,
        start: declaration.span.start(),
        end: declaration.span.end(),
    };
    let reason = Reason::IndividualCapacityOnly {
        references: vec![
            Reference::Deadline(deadline.meta.id),
            Reference::Window(window_key),
        ],
        payload: EmptyPayload {},
    };
    assert_eq!(reason.code(), ReasonCode::IndividualCapacityOnly);
    assert_eq!(reason.code().as_str(), "individual_capacity_only");
    assert_eq!(reason.references().len(), 2);
    let suggestion = Suggestion {
        key: SuggestionKey {
            evaluation_key: key,
            kind: SuggestionKind::ConsiderWork,
            target: WorkTarget::Deadline(deadline.meta.id),
            proposed_span: None,
        },
        kind: SuggestionKind::ConsiderWork,
        target: WorkTarget::Deadline(deadline.meta.id),
        proposed_span: None,
        created_at: instant(),
        valid_until: declaration.span.end(),
        reasons: vec![reason],
    };
    assert_eq!(suggestion.target, WorkTarget::Deadline(deadline.meta.id));
    assert_eq!(
        deadline.fulfillment,
        Fulfillment::Recorded(RecordedResolution::Unresolved)
    );
    assert_ne!(
        Opportunity::Unknown {
            known_qualifying_ms: 0,
            window_keys: vec![]
        },
        Opportunity::Known {
            milliseconds: 0,
            known_qualifying_ms: 0,
            window_keys: vec![]
        }
    );
}
