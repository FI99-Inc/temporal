//! Repeating local events and usual weekly availability.
//!
//! Both are app-owned conveniences over records the core already understands.
//! A series expands into ordinary individual Anchors and a usual pattern into
//! ordinary availability declarations, each with a stable identity derived
//! from its origin. The core never sees a rule; it sees bounded occurrences.
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    num::{NonZeroU32, NonZeroU64},
};
use temporal_core::{domain::*, time::*};

/// Free text that describes a fact for the person reading it. It is display
/// only: it never enters evaluation, matching, or ranking.
#[derive(Clone, Debug, Default, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EventDetails {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub place: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl EventDetails {
    pub fn is_empty(&self) -> bool {
        self.place.is_none() && self.notes.is_none()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SeriesFrequency {
    Daily,
    Weekly,
    Monthly,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SeriesTiming {
    /// Wall-clock start in the series zone and an absolute duration.
    Timed {
        start_minute: u16,
        duration_minutes: NonZeroU32,
    },
    /// Whole civil dates starting on each occurrence date.
    AllDay { days: NonZeroU32 },
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SeriesRule {
    pub frequency: SeriesFrequency,
    pub interval: NonZeroU32,
    /// Weekly rules only. Empty means the weekday of `first_date`.
    #[serde(default)]
    pub weekdays: BTreeSet<Weekday>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until_date_exclusive: Option<LocalDate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub count: Option<NonZeroU32>,
}

/// A repeating fixed event. Each occurrence is an ordinary local Anchor.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnchorSeries {
    pub meta: RecordMeta<AnchorId>,
    pub title: String,
    pub presence: Presence,
    pub zone: ZoneId,
    pub first_date: LocalDate,
    pub timing: SeriesTiming,
    pub rule: SeriesRule,
    #[serde(default)]
    pub skipped: BTreeSet<LocalDate>,
    pub occupancy: Occupancy,
    pub reported_certainty: ReportedCertainty,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<ContextTag>,
    #[serde(default, skip_serializing_if = "EventDetails::is_empty")]
    pub details: EventDetails,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeeklyBlock {
    pub weekday: Weekday,
    pub start_minute: u16,
    /// Exclusive; at most 1440 (midnight ending the day).
    pub end_minute: u16,
}

/// The time a person is usually willing to use, declared once.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UsualAvailability {
    pub meta: RecordMeta<AvailabilityId>,
    pub zone: ZoneId,
    pub blocks: Vec<WeeklyBlock>,
    pub contexts: DeclaredContexts,
    pub energy_capacity: EnergyCapacity,
}

/// One expanded occurrence and the date that identifies it within its series.
#[derive(Clone, Debug)]
pub struct SeriesOccurrence {
    pub anchor: Anchor,
    pub series_id: AnchorId,
    pub date: LocalDate,
}

/// A stable, canonical UUIDv4-shaped identity derived from an origin and a key.
/// The same origin and key always produce the same identity.
pub fn derive_id<I: std::str::FromStr>(origin: &str, key: &str) -> I
where
    I::Err: std::fmt::Debug,
{
    let namespace = uuid::Uuid::parse_str(origin).unwrap_or(uuid::Uuid::NAMESPACE_OID);
    let hashed = uuid::Uuid::new_v5(&namespace, key.as_bytes());
    uuid::Builder::from_random_bytes(*hashed.as_bytes())
        .into_uuid()
        .to_string()
        .parse()
        .expect("derived identity is a canonical UUIDv4")
}

/// Resolve a civil wall time the way a calendar does: an ambiguous time takes
/// the earlier offset and a skipped time moves forward by the gap.
pub fn resolve_wall(date: LocalDate, minute: u32, zone: ZoneId) -> Result<Instant, TimeError> {
    let rules = TimezoneRules::bundled();
    let (day, minute) = if minute >= 1440 {
        (date.next_day()?, minute - 1440)
    } else {
        (date, minute)
    };
    let local = day.at(minute / 60, minute % 60, 0, 0)?;
    match rules.resolve_local(local, zone, LocalResolution::Earlier) {
        Err(TimeError::NonexistentLocalTime) => {
            let shifted = minute + 60;
            let (day, minute) = if shifted >= 1440 {
                (day.next_day()?, shifted - 1440)
            } else {
                (day, shifted)
            };
            rules.resolve_local(
                day.at(minute / 60, minute % 60, 0, 0)?,
                zone,
                LocalResolution::Earlier,
            )
        }
        other => other,
    }
}

fn weekday_of(date: LocalDate) -> Weekday {
    match date.weekday_number() {
        1 => Weekday::Mon,
        2 => Weekday::Tue,
        3 => Weekday::Wed,
        4 => Weekday::Thu,
        5 => Weekday::Fri,
        6 => Weekday::Sat,
        _ => Weekday::Sun,
    }
}

fn add_days(date: LocalDate, days: i64) -> Result<LocalDate, TimeError> {
    let mut value = date;
    if days >= 0 {
        for _ in 0..days {
            value = value.next_day()?;
        }
    } else {
        for _ in 0..(-days) {
            value = value.previous_day()?;
        }
    }
    Ok(value)
}

fn parse_date_parts(date: LocalDate) -> (i32, u32, u32) {
    let text = date.to_string();
    let mut parts = text.split('-');
    let year = parts.next().and_then(|v| v.parse().ok()).unwrap_or(1970);
    let month = parts.next().and_then(|v| v.parse().ok()).unwrap_or(1);
    let day = parts.next().and_then(|v| v.parse().ok()).unwrap_or(1);
    (year, month, day)
}

/// Rule dates from the first date, in order, up to and including `last`.
/// Counts follow RFC 5545: a skipped date still consumes the count.
pub fn rule_dates(series: &AnchorSeries, last: LocalDate) -> Result<Vec<LocalDate>, TimeError> {
    const MAX_DATES: usize = 20_000;
    let rule = &series.rule;
    let interval = i64::from(rule.interval.get());
    let limit = rule.until_date_exclusive;
    let count = rule.count.map(|c| c.get() as usize);
    let mut dates = Vec::new();
    let accept = |date: LocalDate, dates: &mut Vec<LocalDate>| -> bool {
        if limit.is_some_and(|until| date >= until) || date > last {
            return false;
        }
        if count.is_some_and(|count| dates.len() >= count) {
            return false;
        }
        dates.push(date);
        dates.len() < MAX_DATES
    };
    match rule.frequency {
        SeriesFrequency::Daily => {
            let mut date = series.first_date;
            while accept(date, &mut dates) {
                date = add_days(date, interval)?;
            }
        }
        SeriesFrequency::Weekly => {
            let weekdays: BTreeSet<Weekday> = if rule.weekdays.is_empty() {
                [weekday_of(series.first_date)].into()
            } else {
                rule.weekdays.clone()
            };
            let offset = i64::from(series.first_date.weekday_number()) - 1;
            let mut week_start = add_days(series.first_date, -offset)?;
            'weeks: loop {
                for day in &weekdays {
                    let date = add_days(week_start, i64::from(day.number()) - 1)?;
                    if date < series.first_date {
                        continue;
                    }
                    if !accept(date, &mut dates) {
                        break 'weeks;
                    }
                }
                week_start = add_days(week_start, 7 * interval)?;
            }
        }
        SeriesFrequency::Monthly => {
            let (mut year, mut month, day) = parse_date_parts(series.first_date);
            let mut guard = 0;
            loop {
                guard += 1;
                if guard > 12 * 400 {
                    break;
                }
                if let Ok(date) = LocalDate::new(year, month, day)
                    && !accept(date, &mut dates)
                {
                    break;
                }
                let next = month - 1 + rule.interval.get();
                year += i32::try_from(next / 12).unwrap_or(0);
                month = next % 12 + 1;
                if let Ok(date) = LocalDate::new(year, month, 1)
                    && (date > last || limit.is_some_and(|until| date >= until))
                {
                    break;
                }
            }
        }
    }
    Ok(dates)
}

fn occurrence_span(series: &AnchorSeries, date: LocalDate) -> Result<TemporalSpan, TimeError> {
    match &series.timing {
        SeriesTiming::Timed {
            start_minute,
            duration_minutes,
        } => {
            let start = resolve_wall(date, u32::from(*start_minute), series.zone)?;
            let end = start.checked_add_ms(u64::from(duration_minutes.get()) * 60_000)?;
            Ok(TemporalSpan::Timed(
                TimedSpan::new(start, end)?.with_original_zone(series.zone),
            ))
        }
        SeriesTiming::AllDay { days } => Ok(TemporalSpan::Dates(DateSpan::new(
            date,
            add_days(date, i64::from(days.get()))?,
            series.zone,
        )?)),
    }
}

/// Occurrences of one present series that intersect `[from, to)`.
pub fn expand_series(
    series: &AnchorSeries,
    source_id: SourceId,
    from: Instant,
    to: Instant,
) -> Result<Vec<SeriesOccurrence>, TimeError> {
    if series.presence != Presence::Present || to <= from {
        return Ok(Vec::new());
    }
    let rules = TimezoneRules::bundled();
    let range = TimedSpan::new(from, to)?;
    let last = rules.date_at(to, series.zone)?.next_day()?;
    let mut occurrences = Vec::new();
    for date in rule_dates(series, last)? {
        if series.skipped.contains(&date) {
            continue;
        }
        let span = occurrence_span(series, date)?;
        if !span.resolve(&rules)?.overlaps(&range) {
            continue;
        }
        let id = series.meta.id.to_string();
        occurrences.push(SeriesOccurrence {
            anchor: Anchor {
                meta: RecordMeta {
                    id: derive_id(&id, &format!("occurrence:{date}")),
                    revision: series.meta.revision,
                    created_at: series.meta.created_at,
                    updated_at: series.meta.updated_at,
                },
                title: series.title.clone(),
                presence: Presence::Present,
                provenance: Provenance::LocalUser(LocalAssertion {
                    source_id,
                    asserted_at: series.meta.updated_at,
                }),
                span,
                rigidity: AnchorRigidity::Fixed,
                reported_certainty: series.reported_certainty,
                occupancy: series.occupancy,
                location: series.location.clone(),
            },
            series_id: series.meta.id,
            date,
        });
    }
    Ok(occurrences)
}

fn subtract(span: &TimedSpan, holes: &[TimedSpan]) -> Vec<TimedSpan> {
    let mut pieces = vec![span.clone()];
    for hole in holes {
        let mut next = Vec::new();
        for piece in pieces {
            if !piece.overlaps(hole) {
                next.push(piece);
                continue;
            }
            if piece.start() < hole.start()
                && let Ok(left) = TimedSpan::new(piece.start(), hole.start())
            {
                next.push(left);
            }
            if hole.end() < piece.end()
                && let Ok(right) = TimedSpan::new(hole.end(), piece.end())
            {
                next.push(right);
            }
        }
        pieces = next;
    }
    pieces
}

/// The usual pattern as explicit declarations intersecting `[from, to)`.
/// Explicit one-off declarations take precedence where they overlap.
pub fn expand_usual(
    usual: &UsualAvailability,
    explicit: &[AvailabilityDeclaration],
    from: Instant,
    to: Instant,
) -> Result<Vec<AvailabilityDeclaration>, TimeError> {
    if to <= from || usual.blocks.is_empty() {
        return Ok(Vec::new());
    }
    let rules = TimezoneRules::bundled();
    let range = TimedSpan::new(from, to)?;
    let holes: Vec<TimedSpan> = explicit.iter().map(|d| d.span.clone()).collect();
    let mut date = rules.date_at(from, usual.zone)?.previous_day()?;
    let last = rules.date_at(to, usual.zone)?.next_day()?;
    let origin = usual.meta.id.to_string();
    let mut declarations = Vec::new();
    while date <= last {
        let weekday = weekday_of(date);
        for (index, block) in usual
            .blocks
            .iter()
            .enumerate()
            .filter(|(_, b)| b.weekday == weekday)
        {
            let start = resolve_wall(date, u32::from(block.start_minute), usual.zone)?;
            let end = resolve_wall(date, u32::from(block.end_minute), usual.zone)?;
            let Ok(span) = TimedSpan::new(start, end) else {
                continue;
            };
            for (piece_index, piece) in subtract(&span, &holes).into_iter().enumerate() {
                if !piece.overlaps(&range) {
                    continue;
                }
                declarations.push(AvailabilityDeclaration {
                    meta: RecordMeta {
                        id: derive_id(&origin, &format!("usual:{date}:{index}:{piece_index}")),
                        revision: usual.meta.revision,
                        created_at: usual.meta.created_at,
                        updated_at: usual.meta.updated_at,
                    },
                    span: piece.with_original_zone(usual.zone),
                    contexts: usual.contexts.clone(),
                    energy_capacity: usual.energy_capacity,
                });
            }
        }
        date = date.next_day()?;
    }
    Ok(declarations)
}

/// Validate a usual pattern: bounded minutes and no overlap within a weekday.
pub fn validate_blocks(blocks: &[WeeklyBlock]) -> Result<(), String> {
    if blocks.len() > 7 * 12 {
        return Err("Use at most twelve usual blocks per day.".into());
    }
    for block in blocks {
        if block.end_minute > 1440 || block.start_minute >= block.end_minute {
            return Err("Each usual block needs a start before its end, within one day.".into());
        }
    }
    for (index, block) in blocks.iter().enumerate() {
        for other in &blocks[index + 1..] {
            if block.weekday == other.weekday
                && block.start_minute < other.end_minute
                && other.start_minute < block.end_minute
            {
                return Err("Usual blocks on the same day cannot overlap.".into());
            }
        }
    }
    Ok(())
}

/// Parse `HH:MM` (24-hour, `24:00` allowed for an end) into minutes.
pub fn parse_minute(value: &str, allow_end_of_day: bool) -> Result<u16, String> {
    let (hours, minutes) = value
        .trim()
        .split_once(':')
        .ok_or_else(|| format!("Use HH:MM for {value}"))?;
    let hours: u16 = hours
        .parse()
        .map_err(|_| format!("Use HH:MM for {value}"))?;
    let minutes: u16 = minutes
        .parse()
        .map_err(|_| format!("Use HH:MM for {value}"))?;
    let total = hours * 60 + minutes;
    if minutes >= 60 || total > 1440 || (total == 1440 && !allow_end_of_day) {
        return Err(format!("{value} is not a time of day"));
    }
    Ok(total)
}

pub fn revision_after(value: NonZeroU64) -> Result<NonZeroU64, String> {
    value
        .get()
        .checked_add(1)
        .and_then(NonZeroU64::new)
        .ok_or_else(|| "local record revision limit reached".into())
}

/// Bounded free text for display; empty strings clear the field.
pub fn clean_text(
    value: Option<&str>,
    limit: usize,
    label: &str,
) -> Result<Option<String>, String> {
    match value.map(str::trim) {
        None | Some("") => Ok(None),
        Some(text) if text.chars().count() > limit => {
            Err(format!("{label} is limited to {limit} characters."))
        }
        Some(text) => Ok(Some(text.to_string())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn meta(id: &str) -> RecordMeta<AnchorId> {
        let at: Instant = "2026-09-01T12:00:00.000Z".parse().unwrap();
        RecordMeta {
            id: id.parse().unwrap(),
            revision: NonZeroU64::MIN,
            created_at: at,
            updated_at: at,
        }
    }

    fn weekly(days: &[Weekday], first: &str) -> AnchorSeries {
        AnchorSeries {
            meta: meta("00000000-0000-4000-8000-0000000000aa"),
            title: "Synthetic lecture".into(),
            presence: Presence::Present,
            zone: "America/Toronto".parse().unwrap(),
            first_date: first.parse().unwrap(),
            timing: SeriesTiming::Timed {
                start_minute: 9 * 60,
                duration_minutes: NonZeroU32::new(90).unwrap(),
            },
            rule: SeriesRule {
                frequency: SeriesFrequency::Weekly,
                interval: NonZeroU32::MIN,
                weekdays: days.iter().copied().collect(),
                until_date_exclusive: None,
                count: None,
            },
            skipped: BTreeSet::new(),
            occupancy: Occupancy::Busy,
            reported_certainty: ReportedCertainty::Confirmed,
            location: None,
            details: EventDetails::default(),
        }
    }

    #[test]
    fn weekly_dates_respect_weekdays_count_and_until() {
        let mut series = weekly(&[Weekday::Tue, Weekday::Thu], "2026-10-08");
        let dates = rule_dates(&series, "2026-10-22".parse().unwrap()).unwrap();
        let shown: Vec<String> = dates.iter().map(ToString::to_string).collect();
        assert_eq!(
            shown,
            [
                "2026-10-08",
                "2026-10-13",
                "2026-10-15",
                "2026-10-20",
                "2026-10-22"
            ]
        );
        series.rule.count = NonZeroU32::new(3);
        assert_eq!(
            rule_dates(&series, "2026-12-31".parse().unwrap())
                .unwrap()
                .len(),
            3
        );
        series.rule.count = None;
        series.rule.until_date_exclusive = Some("2026-10-15".parse().unwrap());
        assert_eq!(
            rule_dates(&series, "2026-12-31".parse().unwrap())
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn biweekly_and_monthly_rules_skip_correctly() {
        let mut series = weekly(&[Weekday::Mon], "2026-10-05");
        series.rule.interval = NonZeroU32::new(2).unwrap();
        let dates = rule_dates(&series, "2026-11-02".parse().unwrap()).unwrap();
        let shown: Vec<String> = dates.iter().map(ToString::to_string).collect();
        assert_eq!(shown, ["2026-10-05", "2026-10-19", "2026-11-02"]);
        let mut monthly = weekly(&[], "2026-01-31");
        monthly.rule.frequency = SeriesFrequency::Monthly;
        let dates = rule_dates(&monthly, "2026-06-30".parse().unwrap()).unwrap();
        let shown: Vec<String> = dates.iter().map(ToString::to_string).collect();
        assert_eq!(shown, ["2026-01-31", "2026-03-31", "2026-05-31"]);
    }

    #[test]
    fn expansion_keeps_wall_time_across_dst_and_stable_identity() {
        let series = weekly(&[Weekday::Fri], "2026-10-23");
        let source: SourceId = "00000000-0000-4000-8000-0000000000bb".parse().unwrap();
        let from: Instant = "2026-10-20T00:00:00.000Z".parse().unwrap();
        let to: Instant = "2026-11-10T00:00:00.000Z".parse().unwrap();
        let first = expand_series(&series, source, from, to).unwrap();
        let again = expand_series(&series, source, from, to).unwrap();
        assert_eq!(first.len(), 3);
        let starts: Vec<String> = first
            .iter()
            .map(|o| match &o.anchor.span {
                TemporalSpan::Timed(span) => span.start().to_string(),
                TemporalSpan::Dates(_) => unreachable!(),
            })
            .collect();
        // 09:00 Toronto is 13:00Z before 2026-11-01 and 14:00Z after.
        assert_eq!(
            starts,
            [
                "2026-10-23T13:00:00.000Z",
                "2026-10-30T13:00:00.000Z",
                "2026-11-06T14:00:00.000Z"
            ]
        );
        assert!(
            first
                .iter()
                .zip(&again)
                .all(|(a, b)| a.anchor.meta.id == b.anchor.meta.id)
        );
        let ids: BTreeSet<_> = first.iter().map(|o| o.anchor.meta.id).collect();
        assert_eq!(ids.len(), 3);
    }

    #[test]
    fn skipped_dates_and_nonexistent_times_are_handled() {
        let mut series = weekly(&[Weekday::Sun], "2026-03-01");
        series.timing = SeriesTiming::Timed {
            start_minute: 2 * 60 + 30,
            duration_minutes: NonZeroU32::new(60).unwrap(),
        };
        series.skipped.insert("2026-03-15".parse().unwrap());
        let source: SourceId = "00000000-0000-4000-8000-0000000000bb".parse().unwrap();
        let from: Instant = "2026-03-01T00:00:00.000Z".parse().unwrap();
        let to: Instant = "2026-03-20T00:00:00.000Z".parse().unwrap();
        let occurrences = expand_series(&series, source, from, to).unwrap();
        let dates: Vec<String> = occurrences.iter().map(|o| o.date.to_string()).collect();
        assert_eq!(dates, ["2026-03-01", "2026-03-08"]);
        // 02:30 on 2026-03-08 does not exist in Toronto; it moves to 03:30 EDT.
        match &occurrences[1].anchor.span {
            TemporalSpan::Timed(span) => {
                assert_eq!(span.start().to_string(), "2026-03-08T07:30:00.000Z")
            }
            TemporalSpan::Dates(_) => unreachable!(),
        }
    }

    #[test]
    fn usual_availability_yields_to_explicit_declarations() {
        let at: Instant = "2026-09-01T12:00:00.000Z".parse().unwrap();
        let usual = UsualAvailability {
            meta: RecordMeta {
                id: "00000000-0000-4000-8000-0000000000cc".parse().unwrap(),
                revision: NonZeroU64::MIN,
                created_at: at,
                updated_at: at,
            },
            zone: "America/Toronto".parse().unwrap(),
            blocks: vec![WeeklyBlock {
                weekday: Weekday::Thu,
                start_minute: 9 * 60,
                end_minute: 17 * 60,
            }],
            contexts: DeclaredContexts::Unknown,
            energy_capacity: EnergyCapacity::Normal,
        };
        let explicit = AvailabilityDeclaration {
            meta: RecordMeta {
                id: "00000000-0000-4000-8000-0000000000dd".parse().unwrap(),
                revision: NonZeroU64::MIN,
                created_at: at,
                updated_at: at,
            },
            span: TimedSpan::new(
                "2026-10-08T16:00:00.000Z".parse().unwrap(),
                "2026-10-08T17:00:00.000Z".parse().unwrap(),
            )
            .unwrap(),
            contexts: DeclaredContexts::Unknown,
            energy_capacity: EnergyCapacity::Deep,
        };
        let from: Instant = "2026-10-08T00:00:00.000Z".parse().unwrap();
        let to: Instant = "2026-10-09T04:00:00.000Z".parse().unwrap();
        let declarations = expand_usual(&usual, &[explicit], from, to).unwrap();
        let spans: Vec<(String, String)> = declarations
            .iter()
            .map(|d| (d.span.start().to_string(), d.span.end().to_string()))
            .collect();
        assert_eq!(
            spans,
            [
                (
                    "2026-10-08T13:00:00.000Z".into(),
                    "2026-10-08T16:00:00.000Z".into()
                ),
                (
                    "2026-10-08T17:00:00.000Z".into(),
                    "2026-10-08T21:00:00.000Z".into()
                )
            ]
        );
        assert!(validate_blocks(&usual.blocks).is_ok());
        let overlapping = vec![
            usual.blocks[0].clone(),
            WeeklyBlock {
                weekday: Weekday::Thu,
                start_minute: 16 * 60,
                end_minute: 18 * 60,
            },
        ];
        assert!(validate_blocks(&overlapping).is_err());
    }

    #[test]
    fn minute_parsing_bounds() {
        assert_eq!(parse_minute("09:30", false).unwrap(), 570);
        assert_eq!(parse_minute("24:00", true).unwrap(), 1440);
        assert!(parse_minute("24:00", false).is_err());
        assert!(parse_minute("9:75", false).is_err());
    }
}
