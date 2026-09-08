//! Explicit millisecond and civil-time primitives using bundled IANA rules.

use chrono::{
    DateTime, Datelike, FixedOffset, LocalResult, NaiveDate, NaiveDateTime, Offset, TimeZone,
    Timelike, Utc,
};
use chrono_tz::Tz;
use std::{cmp::Ordering, fmt, str::FromStr};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimeError {
    InvalidFormat,
    OutOfRange,
    InvalidZone,
    InvalidOffset,
    InvalidSpan,
    ArithmeticOverflow,
    NonexistentLocalTime,
    AmbiguousLocalTime,
    OffsetZoneMismatch,
    UnresolvableCivilBoundary,
    EvaluationBeforeSnapshot,
}

impl fmt::Display for TimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::InvalidFormat => "invalid normalized time format",
            Self::OutOfRange => "time outside years 0001 through 9999",
            Self::InvalidZone => "unknown IANA timezone",
            Self::InvalidOffset => "invalid UTC offset",
            Self::InvalidSpan => "interval must have a positive duration",
            Self::ArithmeticOverflow => "time arithmetic overflow",
            Self::NonexistentLocalTime => "nonexistent local time",
            Self::AmbiguousLocalTime => "ambiguous local time requires an explicit choice",
            Self::OffsetZoneMismatch => "offset disagrees with timezone rules",
            Self::UnresolvableCivilBoundary => "unresolvable_civil_boundary",
            Self::EvaluationBeforeSnapshot => "evaluation precedes the captured snapshot",
        })
    }
}

impl std::error::Error for TimeError {}

/// A valid UTC instant. Construction never reads a clock or rounds precision.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Instant(i64);

impl Instant {
    pub fn from_epoch_ms(value: i64) -> Result<Self, TimeError> {
        let datetime =
            DateTime::<Utc>::from_timestamp_millis(value).ok_or(TimeError::OutOfRange)?;
        if !(1..=9999).contains(&datetime.year()) {
            return Err(TimeError::OutOfRange);
        }
        Ok(Self(value))
    }

    pub fn epoch_ms(self) -> i64 {
        self.0
    }

    pub fn checked_add_ms(self, milliseconds: u64) -> Result<Self, TimeError> {
        let delta = i64::try_from(milliseconds).map_err(|_| TimeError::ArithmeticOverflow)?;
        Self::from_epoch_ms(
            self.0
                .checked_add(delta)
                .ok_or(TimeError::ArithmeticOverflow)?,
        )
    }

    pub fn checked_sub_ms(self, milliseconds: u64) -> Result<Self, TimeError> {
        let delta = i64::try_from(milliseconds).map_err(|_| TimeError::ArithmeticOverflow)?;
        Self::from_epoch_ms(
            self.0
                .checked_sub(delta)
                .ok_or(TimeError::ArithmeticOverflow)?,
        )
    }

    pub fn duration_since(self, earlier: Self) -> Result<u64, TimeError> {
        u64::try_from(
            self.0
                .checked_sub(earlier.0)
                .ok_or(TimeError::ArithmeticOverflow)?,
        )
        .map_err(|_| TimeError::InvalidSpan)
    }

    fn datetime(self) -> DateTime<Utc> {
        // Private construction guarantees this representable millisecond value.
        DateTime::from_timestamp_millis(self.0).expect("validated instant")
    }
}

impl fmt::Display for Instant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.datetime().format("%Y-%m-%dT%H:%M:%S%.3fZ"))
    }
}

impl FromStr for Instant {
    type Err = TimeError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != 24 || !value.is_ascii() || !value.ends_with('Z') {
            return Err(TimeError::InvalidFormat);
        }
        let parsed = DateTime::parse_from_rfc3339(value).map_err(|_| TimeError::InvalidFormat)?;
        if parsed.nanosecond() >= 1_000_000_000 {
            return Err(TimeError::InvalidFormat);
        }
        let result = Self::from_epoch_ms(parsed.timestamp_millis())?;
        if result.to_string() != value {
            return Err(TimeError::InvalidFormat);
        }
        Ok(result)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct LocalDate(NaiveDate);

impl LocalDate {
    pub fn new(year: i32, month: u32, day: u32) -> Result<Self, TimeError> {
        if !(1..=9999).contains(&year) {
            return Err(TimeError::OutOfRange);
        }
        Ok(Self(
            NaiveDate::from_ymd_opt(year, month, day).ok_or(TimeError::InvalidFormat)?,
        ))
    }

    pub fn next_day(self) -> Result<Self, TimeError> {
        let next = self.0.succ_opt().ok_or(TimeError::OutOfRange)?;
        Self::new(next.year(), next.month(), next.day())
    }

    pub fn previous_day(self) -> Result<Self, TimeError> {
        let previous = self.0.pred_opt().ok_or(TimeError::OutOfRange)?;
        Self::new(previous.year(), previous.month(), previous.day())
    }

    /// Monday is 1 and Sunday is 7, independent of display locale.
    pub fn weekday_number(self) -> u32 {
        self.0.weekday().number_from_monday()
    }

    pub fn at(
        self,
        hour: u32,
        minute: u32,
        second: u32,
        millisecond: u32,
    ) -> Result<LocalDateTime, TimeError> {
        if second >= 60 || millisecond >= 1000 {
            return Err(TimeError::InvalidFormat);
        }
        Ok(LocalDateTime(
            self.0
                .and_hms_milli_opt(hour, minute, second, millisecond)
                .ok_or(TimeError::InvalidFormat)?,
        ))
    }
}

impl fmt::Display for LocalDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d"))
    }
}

impl FromStr for LocalDate {
    type Err = TimeError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != 10 || !value.is_ascii() {
            return Err(TimeError::InvalidFormat);
        }
        let parsed =
            NaiveDate::parse_from_str(value, "%Y-%m-%d").map_err(|_| TimeError::InvalidFormat)?;
        let result = Self::new(parsed.year(), parsed.month(), parsed.day())?;
        if result.to_string() != value {
            return Err(TimeError::InvalidFormat);
        }
        Ok(result)
    }
}

/// A normalization input, never a substitute for a normalized absolute instant.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct LocalDateTime(NaiveDateTime);

impl FromStr for LocalDateTime {
    type Err = TimeError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        if value.len() != 23 || !value.is_ascii() {
            return Err(TimeError::InvalidFormat);
        }
        let parsed = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.3f")
            .map_err(|_| TimeError::InvalidFormat)?;
        let date = LocalDate::new(parsed.year(), parsed.month(), parsed.day())?;
        let result = date.at(
            parsed.hour(),
            parsed.minute(),
            parsed.second(),
            parsed.nanosecond() / 1_000_000,
        )?;
        if parsed.format("%Y-%m-%dT%H:%M:%S%.3f").to_string() != value {
            return Err(TimeError::InvalidFormat);
        }
        Ok(result)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ZoneId(Tz);

impl ZoneId {
    pub fn name(self) -> &'static str {
        self.0.name()
    }
}
impl Ord for ZoneId {
    fn cmp(&self, other: &Self) -> Ordering {
        self.name().cmp(other.name())
    }
}
impl PartialOrd for ZoneId {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl fmt::Display for ZoneId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}
impl FromStr for ZoneId {
    type Err = TimeError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value
            .parse::<Tz>()
            .map(Self)
            .map_err(|_| TimeError::InvalidZone)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LocalResolution {
    RejectAmbiguous,
    Earlier,
    Later,
    OffsetSeconds(i32),
}

/// Explicit access to the version pinned by the manifest and lockfile.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TimezoneRules {
    _private: (),
}

impl TimezoneRules {
    pub const fn bundled() -> Self {
        Self { _private: () }
    }
    pub fn version(&self) -> &'static str {
        chrono_tz::IANA_TZDB_VERSION
    }

    pub fn resolve_local(
        &self,
        local: LocalDateTime,
        zone: ZoneId,
        choice: LocalResolution,
    ) -> Result<Instant, TimeError> {
        let datetime = match zone.0.from_local_datetime(&local.0) {
            LocalResult::None => return Err(TimeError::NonexistentLocalTime),
            LocalResult::Single(value) => {
                if let LocalResolution::OffsetSeconds(offset) = choice
                    && value.offset().fix().local_minus_utc() != offset
                {
                    return Err(TimeError::OffsetZoneMismatch);
                }
                value
            }
            LocalResult::Ambiguous(first, second) => match choice {
                LocalResolution::RejectAmbiguous => return Err(TimeError::AmbiguousLocalTime),
                LocalResolution::Earlier => first.min(second),
                LocalResolution::Later => first.max(second),
                LocalResolution::OffsetSeconds(offset) => {
                    if first.offset().fix().local_minus_utc() == offset {
                        first
                    } else if second.offset().fix().local_minus_utc() == offset {
                        second
                    } else {
                        return Err(TimeError::OffsetZoneMismatch);
                    }
                }
            },
        };
        Instant::from_epoch_ms(datetime.timestamp_millis())
    }

    pub fn resolve_offset(
        &self,
        local: LocalDateTime,
        offset_seconds: i32,
        zone: Option<ZoneId>,
    ) -> Result<Instant, TimeError> {
        let offset = FixedOffset::east_opt(offset_seconds).ok_or(TimeError::InvalidOffset)?;
        if let Some(zone) = zone {
            return self.resolve_local(local, zone, LocalResolution::OffsetSeconds(offset_seconds));
        }
        let datetime = offset
            .from_local_datetime(&local.0)
            .single()
            .ok_or(TimeError::OutOfRange)?;
        Instant::from_epoch_ms(datetime.timestamp_millis())
    }

    pub fn start_of_date(&self, date: LocalDate, zone: ZoneId) -> Result<Instant, TimeError> {
        self.resolve_local(date.at(0, 0, 0, 0)?, zone, LocalResolution::RejectAmbiguous)
            .map_err(|error| match error {
                TimeError::NonexistentLocalTime | TimeError::AmbiguousLocalTime => {
                    TimeError::UnresolvableCivilBoundary
                }
                other => other,
            })
    }

    pub fn date_at(&self, instant: Instant, zone: ZoneId) -> Result<LocalDate, TimeError> {
        let local = instant.datetime().with_timezone(&zone.0).date_naive();
        LocalDate::new(local.year(), local.month(), local.day())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TimedSpan {
    start: Instant,
    end: Instant,
    original_zone: Option<ZoneId>,
}

impl TimedSpan {
    pub fn new(start: Instant, end: Instant) -> Result<Self, TimeError> {
        if start >= end {
            return Err(TimeError::InvalidSpan);
        }
        Ok(Self {
            start,
            end,
            original_zone: None,
        })
    }
    pub fn with_original_zone(mut self, zone: ZoneId) -> Self {
        self.original_zone = Some(zone);
        self
    }
    pub fn start(&self) -> Instant {
        self.start
    }
    pub fn end(&self) -> Instant {
        self.end
    }
    pub fn original_zone(&self) -> Option<ZoneId> {
        self.original_zone
    }
    pub fn duration_ms(&self) -> u64 {
        self.end
            .duration_since(self.start)
            .expect("validated positive span")
    }
    pub fn contains(&self, instant: Instant) -> bool {
        self.start <= instant && instant < self.end
    }
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }
    /// Derived intersections have no new assertion about an original source zone.
    pub fn intersection(&self, other: &Self) -> Option<Self> {
        Self::new(self.start.max(other.start), self.end.min(other.end)).ok()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DateSpan {
    start_date: LocalDate,
    end_date_exclusive: LocalDate,
    zone: ZoneId,
}

impl DateSpan {
    pub fn new(
        start_date: LocalDate,
        end_date_exclusive: LocalDate,
        zone: ZoneId,
    ) -> Result<Self, TimeError> {
        if start_date >= end_date_exclusive {
            return Err(TimeError::InvalidSpan);
        }
        Ok(Self {
            start_date,
            end_date_exclusive,
            zone,
        })
    }
    pub fn start_date(&self) -> LocalDate {
        self.start_date
    }
    pub fn end_date_exclusive(&self) -> LocalDate {
        self.end_date_exclusive
    }
    pub fn zone(&self) -> ZoneId {
        self.zone
    }
    pub fn resolve(&self, rules: &TimezoneRules) -> Result<TimedSpan, TimeError> {
        Ok(TimedSpan::new(
            rules.start_of_date(self.start_date, self.zone)?,
            rules.start_of_date(self.end_date_exclusive, self.zone)?,
        )?
        .with_original_zone(self.zone))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TemporalSpan {
    Timed(TimedSpan),
    Dates(DateSpan),
}

impl TemporalSpan {
    pub fn resolve(&self, rules: &TimezoneRules) -> Result<TimedSpan, TimeError> {
        match self {
            Self::Timed(span) => Ok(span.clone()),
            Self::Dates(span) => span.resolve(rules),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Cutoff {
    At {
        instant: Instant,
        original_zone: Option<ZoneId>,
    },
    OnDate {
        date: LocalDate,
        zone: ZoneId,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CutoffPosition {
    Upcoming,
    DueNow,
    DueToday,
    Overdue,
}

impl Cutoff {
    pub fn endpoint(&self, rules: &TimezoneRules) -> Result<Instant, TimeError> {
        match *self {
            Self::At { instant, .. } => Ok(instant),
            Self::OnDate { date, zone } => {
                rules.start_of_date(date, zone)?;
                rules.start_of_date(date.next_day()?, zone)
            }
        }
    }
    pub fn position(
        &self,
        now: Instant,
        rules: &TimezoneRules,
    ) -> Result<CutoffPosition, TimeError> {
        let endpoint = self.endpoint(rules)?;
        match *self {
            Self::At { .. } => Ok(match now.cmp(&endpoint) {
                Ordering::Less => CutoffPosition::Upcoming,
                Ordering::Equal => CutoffPosition::DueNow,
                Ordering::Greater => CutoffPosition::Overdue,
            }),
            Self::OnDate { date, zone } => Ok(if now >= endpoint {
                CutoffPosition::Overdue
            } else if now >= rules.start_of_date(date, zone)? {
                CutoffPosition::DueToday
            } else {
                CutoffPosition::Upcoming
            }),
        }
    }
}
