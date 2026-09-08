//! Literal synthetic inputs from SCENARIOS.md, in its declaration/ID order.
use super::builder::*;
use std::{collections::BTreeSet, num::NonZeroU32};
use temporal_core::{domain::*, reasons::*, results::*, time::*};

const MONDAY: &str = "2026-09-07T13:00:00.000Z";
const LOCAL: &[SourceKind] = &[SourceKind::Local];
const TRACE: &[SourceKind] = &[SourceKind::Local, SourceKind::Trace];
const CALENDAR: &[SourceKind] = &[SourceKind::Local, SourceKind::Google];
const DAYS: [&str; 5] = [
    "2026-09-07",
    "2026-09-08",
    "2026-09-09",
    "2026-09-10",
    "2026-09-11",
];

pub fn all() -> Vec<Case> {
    vec![
        s01(),
        s02(),
        s03(),
        s04(),
        s05(),
        s06(),
        s07(),
        s08(),
        s09(),
        s10(),
        s11(false),
        s12(),
        s13(),
        s14(),
        s15(),
        s16(),
    ]
}

fn s01() -> Case {
    let mut b = Builder::new(1, MONDAY, TRACE);
    b.anchor(
        "A1",
        TemporalSpan::Timed(span("2026-09-11", "14:00", "15:00")),
        SourceKind::Local,
    );
    b.input.anchors[0].title = "Short appointment".into();
    b.task("T1", TaskStatus::Now, work(15, 15));
    b.input.task_refs[0].title = "Small errand".into();
    for day in [DAYS[0], DAYS[2]] {
        b.availability(span(day, "09:00", "11:00"));
    }
    b.finish()
}
fn s02() -> Case {
    let mut b = Builder::new(2, MONDAY, TRACE);
    for (index, day) in DAYS.iter().enumerate() {
        b.anchor(
            &format!("A{}", 2 * index + 1),
            TemporalSpan::Timed(span(day, "10:00", "12:00")),
            SourceKind::Local,
        );
        b.anchor(
            &format!("A{}", 2 * index + 2),
            TemporalSpan::Timed(span(day, "14:00", "17:00")),
            SourceKind::Local,
        );
    }
    let mut large = work(480, 90);
    large.required_contexts.insert("campus".parse().unwrap());
    b.local_deadline("D1", DAYS[4], "17:00", large);
    b.input.deadlines[0].title = "Course preparation".into();
    let mut small = work(30, 15);
    small.required_contexts.insert("campus".parse().unwrap());
    b.task("T1", TaskStatus::Later, small);
    b.input.task_refs[0].title = "Short reading".into();
    for day in DAYS {
        b.availability(span(day, "09:00", "17:00"));
    }
    for availability in &mut b.input.availability {
        availability.contexts =
            DeclaredContexts::Known(BTreeSet::from(["campus".parse().unwrap()]));
    }
    b.finish()
}
fn s03() -> Case {
    let mut b = Builder::new(3, MONDAY, LOCAL);
    b.local_deadline("D1", DAYS[4], "09:00", work(480, 60));
    b.input.deadlines[0].title = "Large assignment".into();
    b.input.deadline_annotations[0].importance = Importance::High;
    b.input.deadline_annotations[0].milestone = true;
    for day in &DAYS[..4] {
        b.availability(span(day, "09:00", "14:00"));
    }
    b.finish()
}
fn s04() -> Case {
    let mut b = Builder::new(4, MONDAY, LOCAL);
    for (index, clock) in ["10:00", "10:30", "11:00", "11:30", "12:00", "12:00"]
        .iter()
        .enumerate()
    {
        b.local_deadline(&format!("D{}", index + 1), DAYS[0], clock, work(45, 15));
    }
    b.availability(span(DAYS[0], "09:00", "12:00"));
    b.finish()
}
fn s05() -> Case {
    let mut b = Builder::new(5, MONDAY, CALENDAR);
    b.anchor(
        "A1",
        TemporalSpan::Timed(span(DAYS[0], "10:00", "11:30")),
        SourceKind::Local,
    );
    b.anchor(
        "A2",
        TemporalSpan::Timed(span(DAYS[0], "11:00", "12:00")),
        SourceKind::Google,
    );
    for anchor in &mut b.input.anchors {
        anchor.title = "Seminar".into();
    }
    b.local_deadline("D1", DAYS[0], "13:00", work(90, 30));
    b.availability(span(DAYS[0], "09:00", "13:00"));
    b.finish()
}
fn s06() -> Case {
    let mut b = Builder::new(6, MONDAY, TRACE);
    let task = b.task("T1", TaskStatus::Later, work(240, 60));
    b.input.task_refs[0].title = "Long work item".into();
    b.due("D1", task, DAYS[1], "17:00");
    b.anchor(
        "A1",
        TemporalSpan::Timed(span(DAYS[0], "10:00", "12:00")),
        SourceKind::Local,
    );
    b.anchor(
        "A2",
        TemporalSpan::Timed(span(DAYS[1], "09:00", "11:00")),
        SourceKind::Local,
    );
    for day in &DAYS[..2] {
        b.availability(span(day, "09:00", "12:00"));
    }
    b.finish()
}
fn s07() -> Case {
    let mut b = Builder::new(7, MONDAY, LOCAL);
    b.intention(
        "Groceries maybe",
        Some(date_span("2026-09-06", "2026-09-07")),
        Some(work(30, 15)),
    );
    b.intention(
        "Sketch an idea",
        Some(date_span("2026-09-07", "2026-09-08")),
        None,
    );
    b.finish()
}
fn s08() -> Case {
    let mut b = Builder::new(8, "2026-09-07T17:00:00.000Z", CALENDAR);
    let anchor = b.anchor(
        "A1",
        TemporalSpan::Timed(span(DAYS[0], "14:00", "15:00")),
        SourceKind::Google,
    );
    b.input.anchor_annotations.push(AnchorAnnotation {
        target_id: anchor,
        revision: nz(1),
        created_at: b.audit(),
        updated_at: b.audit(),
        importance: Importance::Unspecified,
        milestone: true,
        confirmation: Some(Confirmation {
            confirmed_at: b.audit(),
            record_revision: nz(1),
        }),
    });
    b.local_deadline("D1", DAYS[0], "16:00", work(90, 45));
    b.input.deadlines[0].title = "Prepare notes".into();
    b.intention("Optional outline", None, Some(work(30, 15)));
    b.availability(span(DAYS[0], "13:00", "16:00"));
    b.finish()
}
fn s09() -> Case {
    let mut b = Builder::new(9, "2026-09-08T13:00:00.000Z", LOCAL);
    b.local_deadline("D1", DAYS[0], "17:00", work(60, 30));
    b.availability(span(DAYS[1], "09:00", "11:00"));
    b.finish()
}
fn s10() -> Case {
    let mut b = Builder::new(10, MONDAY, TRACE);
    let task = b.task("T1", TaskStatus::Now, work(60, 30));
    b.input.task_refs[0].title = "Flexible reading".into();
    for day in &DAYS[..2] {
        b.availability(span(day, "09:00", "11:00"));
    }
    b.input.prior_suggestions.push(work_suggestion(
        &b.input,
        WorkTarget::Task(task),
        date_span(DAYS[0], DAYS[1]),
        "2026-09-08T04:00:00.000Z".parse().unwrap(),
        0,
    ));
    b.finish()
}
pub fn s11(spring: bool) -> Case {
    let (now, day, next, end) = if spring {
        (
            "2026-03-08T05:00:00.000Z",
            "2026-03-08",
            "2026-03-09",
            "2026-03-09T04:00:00.000Z",
        )
    } else {
        (
            "2026-11-01T04:00:00.000Z",
            "2026-11-01",
            "2026-11-02",
            "2026-11-02T05:00:00.000Z",
        )
    };
    let mut b = Builder::new(11, now, LOCAL);
    b.anchor("A1", date_span(day, next), SourceKind::Local);
    b.deadline(
        "D1",
        Cutoff::OnDate {
            date: day.parse().unwrap(),
            zone: "America/Toronto".parse().unwrap(),
        },
        DeadlineWork::Standalone(work(60, 30)),
        SourceKind::Local,
    );
    b.availability(
        TimedSpan::new(now.parse().unwrap(), end.parse().unwrap())
            .unwrap()
            .with_original_zone("America/Toronto".parse().unwrap()),
    );
    let mut case = b.finish();
    if spring {
        case.name = "S11/spring".into();
    }
    case
}
fn s12() -> Case {
    let mut b = Builder::new(12, MONDAY, CALENDAR);
    let source = &mut b.input.source_states[0];
    source.last_success_at = Some(at(DAYS[0], "08:30"));
    source.last_attempt_at = source.last_success_at;
    source.fresh_for_ms = nz(3_600_000);
    b.anchor(
        "A1",
        TemporalSpan::Timed(span(DAYS[0], "10:00", "11:00")),
        SourceKind::Google,
    );
    b.local_deadline("D1", DAYS[0], "12:00", work(90, 30));
    b.availability(span(DAYS[0], "09:00", "12:00"));
    b.finish()
}
fn s13() -> Case {
    let mut b = Builder::new(
        13,
        MONDAY,
        &[SourceKind::Local, SourceKind::Trace, SourceKind::Quercus],
    );
    let task = b.task("T1", TaskStatus::Done, work(120, 30));
    b.due("D1", task, "2026-09-06", "17:00");
    b.deadline(
        "D2",
        Cutoff::At {
            instant: at(DAYS[0], "12:00"),
            original_zone: Some("America/Toronto".parse().unwrap()),
        },
        DeadlineWork::Task(task),
        SourceKind::Quercus,
    );
    b.local_deadline("D3", DAYS[0], "12:00", unknown_work(30));
    let mut lower_bound = work(240, 60);
    lower_bound.effort = Effort::AtLeast(NonZeroU32::new(240).unwrap());
    b.local_deadline("D4", DAYS[0], "12:00", lower_bound);
    b.local_deadline("D5", DAYS[0], "12:00", work(0, 0));
    b.availability(span(DAYS[0], "09:00", "12:00"));
    b.finish()
}
fn s14() -> Case {
    let mut b = Builder::new(14, "2026-09-06T13:00:00.000Z", LOCAL);
    let meta = b.meta();
    b.input.routines.push(Routine {
        meta,
        title: "Weekly reflection".into(),
        presence: Presence::Present,
        provenance: UserProvenance::LocalUser(b.local()),
        state: RoutineState::Active,
        work: work(30, 15),
        rule: RoutineRule::Weekly(WeeklyRule {
            weekdays: BTreeSet::from([Weekday::Sun]),
            start_date: "2026-08-30".parse().unwrap(),
            until_date_exclusive: None,
            zone: "America/Toronto".parse().unwrap(),
        }),
    });
    for day in ["2026-09-06", "2026-09-13"] {
        b.availability(span(day, "09:00", "10:00"));
    }
    b.finish()
}
fn s15() -> Case {
    let mut b = Builder::new(15, MONDAY, LOCAL);
    let mut deep = work(90, 45);
    deep.required_contexts.insert("desk".parse().unwrap());
    deep.energy_requirement = EnergyRequirement::Deep;
    b.local_deadline("D1", DAYS[0], "12:00", deep);
    for (start, end) in [("09:00", "10:00"), ("10:00", "11:00"), ("11:00", "12:00")] {
        b.availability(span(DAYS[0], start, end));
    }
    b.input.availability[0].contexts =
        DeclaredContexts::Known(BTreeSet::from(["campus".parse().unwrap()]));
    b.input.availability[1].contexts = DeclaredContexts::Unknown;
    b.input.availability[2].energy_capacity = EnergyCapacity::Normal;
    b.finish()
}
fn s16() -> Case {
    let mut b = Builder::new(16, MONDAY, TRACE);
    let deadline = b.local_deadline("D1", DAYS[0], "11:00", work(60, 15));
    let task = b.task("T1", TaskStatus::Now, work(60, 60));
    b.availability(span(DAYS[0], "09:00", "11:00"));
    b.input.prior_suggestions.push(work_suggestion(
        &b.input,
        WorkTarget::Task(task),
        TemporalSpan::Timed(span(DAYS[0], "09:00", "10:00")),
        at(DAYS[0], "10:00"),
        0,
    ));
    let evaluation_key = b.input.evaluation.key(b.input.snapshot_revision);
    let target = WorkTarget::Deadline(deadline);
    b.input.prior_suggestions.push(Suggestion {
        key: SuggestionKey {
            evaluation_key: evaluation_key.clone(),
            kind: SuggestionKind::RiskNotice,
            target,
            proposed_span: None,
        },
        kind: SuggestionKind::RiskNotice,
        target,
        proposed_span: None,
        created_at: b.input.evaluation.now,
        valid_until: at(DAYS[0], "11:00"),
        reasons: vec![Reason::PressureRatio {
            references: vec![Reference::Pressure(PressureKey {
                evaluation_key,
                deadline_id: deadline,
            })],
            payload: RatioPayload {
                work_ms: 3_600_000,
                opportunity_ms: 7_200_000,
                risk: Risk::Tight,
            },
        }],
    });
    b.finish()
}
