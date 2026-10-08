//! Read-only iCalendar sources: one source identity per imported calendar.
//!
//! The adapter keeps the last successfully parsed text and normalizes it into
//! bounded Anchors and (for coursework calendars) Deadlines with source-stable
//! identity: UID plus the source's original recurrence start. A failed refresh
//! keeps last-known events and records the attempt; nothing is written back.
use crate::{ics, series::derive_id};
use serde::{Deserialize, Serialize};
use std::num::NonZeroU64;
use temporal_core::{domain::*, time::*};

pub const FILE_FRESH_MS: u64 = 7 * 86_400_000;
pub const SUBSCRIPTION_FRESH_MS: u64 = 24 * 3_600_000;
/// An iCalendar file or feed describes its whole timeline; coverage is bounded
/// only so it stays a finite span.
const COVERAGE_MS: u64 = 3650 * 86_400_000;
const CANVAS_ASSIGNMENT: &str = "event-assignment-";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CalendarMode {
    /// Every entry is a fixed event (Anchor).
    Events,
    /// Canvas/Quercus: assignment entries and zero-length entries are real
    /// Deadlines; everything else is a fixed event.
    Coursework,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum CalendarOrigin {
    /// A file the person chose. Only its name is kept for display.
    File { name: String },
    /// A subscription. The URL lives in the OS credential store under
    /// `credential`; `host` is kept only so the person can recognise it.
    Subscription { credential: String, host: String },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CalendarSource {
    pub source: Source,
    pub state: SourceState,
    pub mode: CalendarMode,
    pub color: String,
    pub origin: CalendarOrigin,
    pub revision: NonZeroU64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calendar_name: Option<String>,
    #[serde(default)]
    pub event_count: usize,
    #[serde(default)]
    pub skipped: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_error: Option<String>,
    #[serde(default)]
    pub hidden: bool,
}

impl CalendarSource {
    pub fn fresh_for(origin: &CalendarOrigin) -> NonZeroU64 {
        NonZeroU64::new(match origin {
            CalendarOrigin::File { .. } => FILE_FRESH_MS,
            CalendarOrigin::Subscription { .. } => SUBSCRIPTION_FRESH_MS,
        })
        .expect("nonzero freshness")
    }

    pub fn required(&self) -> Vec<RequiredSource> {
        let mut roles = vec![RequiredSource {
            source_id: self.source.id,
            role: SourceRole::Anchors,
        }];
        if self.mode == CalendarMode::Coursework {
            roles.push(RequiredSource {
                source_id: self.source.id,
                role: SourceRole::Deadlines,
            });
        }
        roles
    }

    /// Record a successful parse: complete outcome and whole-timeline coverage.
    pub fn record_success(
        &mut self,
        parsed: &ics::IcsCalendar,
        now: Instant,
    ) -> Result<(), String> {
        let coverage = TimedSpan::new(
            now.checked_sub_ms(COVERAGE_MS).map_err(|e| e.to_string())?,
            now.checked_add_ms(COVERAGE_MS).map_err(|e| e.to_string())?,
        )
        .map_err(|e| e.to_string())?;
        self.state.last_attempt_outcome = AttemptOutcome::Complete;
        self.state.last_attempt_at = Some(now);
        self.state.last_success_at = Some(now);
        self.state.anchor_coverage = Some(coverage.clone());
        self.state.deadline_coverage = Some(coverage);
        self.calendar_name = parsed.name.clone();
        self.event_count = parsed.events.len();
        self.skipped = parsed.skipped;
        self.last_error = None;
        self.revision = bump(self.revision)?;
        Ok(())
    }

    /// Record a failed attempt. Last-known events and coverage are kept.
    pub fn record_failure(
        &mut self,
        outcome: AttemptOutcome,
        message: String,
        now: Instant,
    ) -> Result<(), String> {
        self.state.last_attempt_outcome = outcome;
        self.state.last_attempt_at = Some(now);
        self.last_error = Some(message);
        self.revision = bump(self.revision)?;
        Ok(())
    }
}

fn bump(value: NonZeroU64) -> Result<NonZeroU64, String> {
    value
        .get()
        .checked_add(1)
        .and_then(NonZeroU64::new)
        .ok_or_else(|| "calendar revision limit reached".into())
}

pub fn new_source(
    id: SourceId,
    kind: SourceKind,
    label: String,
    mode: CalendarMode,
    color: String,
    origin: CalendarOrigin,
) -> CalendarSource {
    CalendarSource {
        source: Source { id, kind, label },
        state: SourceState {
            source_id: id,
            last_attempt_outcome: AttemptOutcome::Never,
            fresh_for_ms: CalendarSource::fresh_for(&origin),
            last_attempt_at: None,
            last_success_at: None,
            anchor_coverage: None,
            deadline_coverage: None,
            tasks_complete: false,
        },
        mode,
        color,
        origin,
        revision: NonZeroU64::MIN,
        calendar_name: None,
        event_count: 0,
        skipped: 0,
        last_error: None,
        hidden: false,
    }
}

/// Validate a user-chosen colour: `#rrggbb`.
pub fn clean_color(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.len() == 7
        && value.starts_with('#')
        && value[1..].bytes().all(|b| b.is_ascii_hexdigit())
    {
        Ok(value.to_ascii_lowercase())
    } else {
        Err("Choose a colour like #286580.".into())
    }
}

pub fn clean_label(value: &str) -> Result<String, String> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > 80 {
        Err("Name the calendar in 1 to 80 characters.".into())
    } else {
        Ok(value.to_string())
    }
}

/// One normalized imported occurrence and its display-only text.
#[derive(Clone, Debug)]
pub enum Imported {
    Anchor {
        anchor: Anchor,
        place: Option<String>,
        zero_length: bool,
    },
    Deadline {
        deadline: Deadline,
        place: Option<String>,
    },
}

fn tz_to_zone(tz: chrono_tz::Tz) -> Option<ZoneId> {
    tz.name().parse().ok()
}

/// Normalize the calendar's occurrences that intersect `[from, to)`.
/// Returns the rows and the number of recurring entries that could not be
/// expanded (kept visible as source health detail, never guessed).
pub fn normalize(
    source: &CalendarSource,
    parsed: &ics::IcsCalendar,
    from: Instant,
    to: Instant,
    floating: ZoneId,
) -> (Vec<Imported>, usize) {
    let Some(observed_at) = source.state.last_success_at else {
        return (Vec::new(), 0);
    };
    let floating_tz: chrono_tz::Tz = floating
        .name()
        .parse()
        .unwrap_or(chrono_tz::America::Toronto);
    let (occurrences, unexpandable) =
        ics::expand(parsed, from.epoch_ms(), to.epoch_ms(), floating_tz);
    let origin = source.source.id.to_string();
    let mut seen = std::collections::BTreeSet::new();
    let mut rows = Vec::new();
    for occurrence in occurrences {
        let zero_length = matches!(
            occurrence.span,
            ics::OccurrenceSpan::Timed { start_ms, end_ms, .. } if end_ms <= start_ms
        );
        let is_deadline = source.mode == CalendarMode::Coursework
            && (occurrence.uid.starts_with(CANVAS_ASSIGNMENT) || zero_length);
        let projection = if is_deadline {
            Projection::Deadline
        } else {
            Projection::Anchor
        };
        let key = format!(
            "{}|{}|{}",
            if is_deadline { "deadline" } else { "anchor" },
            occurrence.uid,
            occurrence.occurrence_key.as_deref().unwrap_or("")
        );
        if !seen.insert(key.clone()) {
            continue;
        }
        let provenance = ImportedProvenance {
            source_id: source.source.id,
            external_id: occurrence.uid.clone(),
            occurrence_key: occurrence.occurrence_key.clone(),
            projection,
            observed_at,
            source_revision: None,
            source_created_at: None,
            source_updated_at: None,
        };
        let meta_at = observed_at;
        if is_deadline {
            let cutoff = match &occurrence.span {
                ics::OccurrenceSpan::Timed { start_ms, zone, .. } => {
                    let Ok(instant) = Instant::from_epoch_ms(*start_ms) else {
                        continue;
                    };
                    Cutoff::At {
                        instant,
                        original_zone: zone.and_then(tz_to_zone),
                    }
                }
                ics::OccurrenceSpan::Dates { start, .. } => {
                    let Ok(date) = start.to_string().parse::<LocalDate>() else {
                        continue;
                    };
                    Cutoff::OnDate {
                        date,
                        zone: floating,
                    }
                }
            };
            rows.push(Imported::Deadline {
                deadline: Deadline {
                    meta: RecordMeta {
                        id: derive_id(&origin, &key),
                        revision: source.revision,
                        created_at: meta_at,
                        updated_at: meta_at,
                    },
                    title: occurrence.summary.clone(),
                    presence: Presence::Present,
                    provenance: Provenance::Imported(provenance),
                    cutoff,
                    fulfillment: Fulfillment::Recorded(RecordedResolution::Unresolved),
                },
                place: occurrence.location.clone(),
            });
            continue;
        }
        let span = match &occurrence.span {
            ics::OccurrenceSpan::Timed {
                start_ms,
                end_ms,
                zone,
            } => {
                let (Ok(start), Ok(end)) = (
                    Instant::from_epoch_ms(*start_ms),
                    Instant::from_epoch_ms((*end_ms).max(start_ms + 60_000)),
                ) else {
                    continue;
                };
                let Ok(span) = TimedSpan::new(start, end) else {
                    continue;
                };
                TemporalSpan::Timed(match zone.and_then(tz_to_zone) {
                    Some(zone) => span.with_original_zone(zone),
                    None => span,
                })
            }
            ics::OccurrenceSpan::Dates {
                start,
                end_exclusive,
            } => {
                let (Ok(start), Ok(end)) = (
                    start.to_string().parse::<LocalDate>(),
                    end_exclusive.to_string().parse::<LocalDate>(),
                ) else {
                    continue;
                };
                let Ok(span) = DateSpan::new(start, end, floating) else {
                    continue;
                };
                TemporalSpan::Dates(span)
            }
        };
        rows.push(Imported::Anchor {
            anchor: Anchor {
                meta: RecordMeta {
                    id: derive_id(&origin, &key),
                    revision: source.revision,
                    created_at: meta_at,
                    updated_at: meta_at,
                },
                title: occurrence.summary.clone(),
                presence: Presence::Present,
                provenance: Provenance::Imported(provenance),
                span,
                rigidity: AnchorRigidity::Fixed,
                reported_certainty: if occurrence.tentative {
                    ReportedCertainty::Tentative
                } else {
                    ReportedCertainty::Confirmed
                },
                // A zero-length entry is a point in time; it blocks nothing.
                occupancy: if occurrence.transparent || zero_length {
                    Occupancy::Transparent
                } else {
                    Occupancy::Busy
                },
                location: None,
            },
            place: occurrence.location.clone(),
            zero_length,
        });
    }
    (rows, unexpandable)
}
