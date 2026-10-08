//! Read-only iCalendar (RFC 5545) import: a pure parser and bounded recurrence expansion.
//!
//! Nothing in this module touches the filesystem, the network, the clock, or Tauri.
//! Local wall times are resolved only through zones the caller supplies, so the same
//! text and window always produce the same occurrences.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::fmt;
use std::hash::Hash;

use chrono::{
    DateTime, Days, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone, Utc,
};
use chrono_tz::Tz;
use rrule::{RRule, RRuleSet, Tz as RruleTz, Unvalidated};

/// Largest accepted input, in bytes.
pub const MAX_BYTES: usize = 20 * 1024 * 1024;
/// Most occurrences produced for one master (recurring series or single event).
pub const MAX_OCCURRENCES_PER_EVENT: usize = 2000;

/// Component nesting deeper than this is not tracked; its contents are ignored.
const MAX_NESTING: usize = 16;
/// Rule instances examined per recurring master before its rule counts as unexpandable.
const SCAN_BUDGET: usize = 250_000;

/// A start, end, or recurrence value as written in the source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IcsTime {
    /// `VALUE=DATE`: a calendar day with no time of day.
    Date(NaiveDate),
    /// A date-time with a trailing `Z`, in UTC.
    Utc(NaiveDateTime),
    /// A local date-time with a `TZID` that maps to an IANA zone.
    Zoned(NaiveDateTime, Tz),
    /// A local date-time with no `TZID` and no `Z`.
    Floating(NaiveDateTime),
}

/// One event as stored in a calendar: a series master, or an override when `recurrence_id` is set.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcsEvent {
    pub uid: String,
    /// Unescaped `SUMMARY`; a missing or blank summary becomes `"(No title)"`.
    pub summary: String,
    pub location: Option<String>,
    pub url: Option<String>,
    pub start: IcsTime,
    /// `DTEND`, or `DTSTART` plus `DURATION`, or the RFC default when neither is present.
    pub end: Option<IcsTime>,
    /// Raw value after `RRULE:`.
    pub rrule: Option<String>,
    pub rdates: Vec<IcsTime>,
    pub exdates: Vec<IcsTime>,
    pub recurrence_id: Option<IcsTime>,
    /// `STATUS:CANCELLED`.
    pub cancelled: bool,
    /// `STATUS:TENTATIVE`.
    pub tentative: bool,
    /// `TRANSP:TRANSPARENT`.
    pub transparent: bool,
    /// `SEQUENCE`, defaulting to 0.
    pub sequence: i64,
    /// A `TZID` was present but could not be mapped to a zone.
    pub zone_guessed: bool,
}

/// A parsed calendar.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IcsCalendar {
    /// `X-WR-CALNAME`.
    pub name: Option<String>,
    pub prodid: Option<String>,
    /// `X-WR-TIMEZONE`, when it maps to a zone.
    pub default_zone: Option<Tz>,
    /// Masters and overrides. Duplicate masters are resolved by `SEQUENCE`.
    pub events: Vec<IcsEvent>,
    /// VEVENTs dropped for lacking a UID or DTSTART, or for unparsable times.
    pub skipped: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IcsError {
    /// No `BEGIN:VCALENDAR` line was found.
    NotCalendar,
    /// The input exceeds [`MAX_BYTES`].
    TooLarge,
    /// The calendar is structurally incomplete; the message says how.
    Malformed(String),
}

impl fmt::Display for IcsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotCalendar => f.write_str("This file is not an iCalendar calendar."),
            Self::TooLarge => f.write_str("This calendar is too large to import."),
            Self::Malformed(reason) => write!(f, "This calendar could not be read: {reason}."),
        }
    }
}

impl std::error::Error for IcsError {}

/// Parses iCalendar text. Floating times without a calendar default zone are read in `fallback_zone`.
pub fn parse(text: &str, fallback_zone: Tz) -> Result<IcsCalendar, IcsError> {
    if text.len() > MAX_BYTES {
        return Err(IcsError::TooLarge);
    }
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let scanned = scan(&unfold(text));
    if !scanned.saw_calendar {
        return Err(IcsError::NotCalendar);
    }
    if scanned.unterminated {
        return Err(IcsError::Malformed(
            "the calendar ends before its END:VCALENDAR line".into(),
        ));
    }

    let name = text_of(&scanned.root, "X-WR-CALNAME").filter(|name| !name.trim().is_empty());
    let prodid = text_of(&scanned.root, "PRODID");
    let default_zone =
        property(&scanned.root, "X-WR-TIMEZONE").and_then(|line| map_tzid(&line.value));

    let mut events = Vec::new();
    let mut masters: HashMap<String, usize> = HashMap::new();
    let mut overrides: HashMap<(String, String), usize> = HashMap::new();
    let mut skipped = 0;
    for props in &scanned.events {
        let Some(event) = build_event(props, default_zone, fallback_zone) else {
            skipped += 1;
            continue;
        };
        match event.recurrence_id.as_ref().map(identity) {
            Some(rid) => keep(&mut events, &mut overrides, (event.uid.clone(), rid), event),
            None => keep(&mut events, &mut masters, event.uid.clone(), event),
        }
    }

    Ok(IcsCalendar {
        name,
        prodid,
        default_zone,
        events,
        skipped,
    })
}

/// Keeps one event per key: the higher `SEQUENCE` wins, and on a tie the later event wins.
fn keep<K: Eq + Hash>(
    events: &mut Vec<IcsEvent>,
    seen: &mut HashMap<K, usize>,
    key: K,
    event: IcsEvent,
) {
    match seen.get(&key).copied() {
        Some(index) => match events.get_mut(index) {
            Some(existing) if event.sequence >= existing.sequence => *existing = event,
            _ => {}
        },
        None => {
            seen.insert(key, events.len());
            events.push(event);
        }
    }
}

/// Maps a TZID (IANA, path-prefixed IANA, or Windows name) to an IANA zone.
pub fn map_tzid(tzid: &str) -> Option<Tz> {
    let name = tzid.trim().trim_matches('"').trim();
    if name.is_empty() {
        return None;
    }
    if let Some(zone) = iana(name) {
        return Some(zone);
    }
    // Path-prefixed forms such as "/mozilla.org/20050126_1/America/New_York".
    let segments: Vec<&str> = name
        .split('/')
        .filter(|segment| !segment.is_empty())
        .collect();
    for count in [2, 3] {
        let tail: Vec<&str> = segments.iter().rev().take(count).rev().copied().collect();
        let candidate = (tail.len() == count).then(|| tail.join("/"));
        if let Some(zone) = candidate.as_deref().and_then(iana) {
            return Some(zone);
        }
    }
    windows_zone(name)
}

/// Windows zone names from the CLDR `windowsZones` mapping (territory 001).
const WINDOWS_ZONES: &[(&str, &str)] = &[
    ("Eastern Standard Time", "America/New_York"),
    ("Central Standard Time", "America/Chicago"),
    ("Mountain Standard Time", "America/Denver"),
    ("Pacific Standard Time", "America/Los_Angeles"),
    ("US Mountain Standard Time", "America/Phoenix"),
    ("Alaskan Standard Time", "America/Anchorage"),
    ("Hawaiian Standard Time", "Pacific/Honolulu"),
    ("Atlantic Standard Time", "America/Halifax"),
    ("Newfoundland Standard Time", "America/St_Johns"),
    ("Canada Central Standard Time", "America/Regina"),
    ("Central Standard Time (Mexico)", "America/Mexico_City"),
    ("Mountain Standard Time (Mexico)", "America/Mazatlan"),
    ("Pacific Standard Time (Mexico)", "America/Tijuana"),
    ("Eastern Standard Time (Mexico)", "America/Cancun"),
    ("Central America Standard Time", "America/Guatemala"),
    ("Cuba Standard Time", "America/Havana"),
    ("SA Pacific Standard Time", "America/Bogota"),
    ("Venezuela Standard Time", "America/Caracas"),
    ("SA Western Standard Time", "America/La_Paz"),
    ("Pacific SA Standard Time", "America/Santiago"),
    ("E. South America Standard Time", "America/Sao_Paulo"),
    ("Argentina Standard Time", "America/Argentina/Buenos_Aires"),
    ("Montevideo Standard Time", "America/Montevideo"),
    ("Paraguay Standard Time", "America/Asuncion"),
    ("Greenland Standard Time", "America/Nuuk"),
    ("UTC", "UTC"),
    ("Coordinated Universal Time", "UTC"),
    ("GMT Standard Time", "Europe/London"),
    ("Greenwich Standard Time", "Atlantic/Reykjavik"),
    ("W. Europe Standard Time", "Europe/Berlin"),
    ("Romance Standard Time", "Europe/Paris"),
    ("Central Europe Standard Time", "Europe/Budapest"),
    ("Central European Standard Time", "Europe/Warsaw"),
    ("E. Europe Standard Time", "Europe/Chisinau"),
    ("FLE Standard Time", "Europe/Kyiv"),
    ("GTB Standard Time", "Europe/Bucharest"),
    ("Turkey Standard Time", "Europe/Istanbul"),
    ("Russian Standard Time", "Europe/Moscow"),
    ("Belarus Standard Time", "Europe/Minsk"),
    ("Kaliningrad Standard Time", "Europe/Kaliningrad"),
    ("Israel Standard Time", "Asia/Jerusalem"),
    ("Jordan Standard Time", "Asia/Amman"),
    ("Middle East Standard Time", "Asia/Beirut"),
    ("Syria Standard Time", "Asia/Damascus"),
    ("Egypt Standard Time", "Africa/Cairo"),
    ("South Africa Standard Time", "Africa/Johannesburg"),
    ("W. Central Africa Standard Time", "Africa/Lagos"),
    ("E. Africa Standard Time", "Africa/Nairobi"),
    ("Morocco Standard Time", "Africa/Casablanca"),
    ("Namibia Standard Time", "Africa/Windhoek"),
    ("Arab Standard Time", "Asia/Riyadh"),
    ("Arabic Standard Time", "Asia/Baghdad"),
    ("Arabian Standard Time", "Asia/Dubai"),
    ("Iran Standard Time", "Asia/Tehran"),
    ("Afghanistan Standard Time", "Asia/Kabul"),
    ("Pakistan Standard Time", "Asia/Karachi"),
    ("West Asia Standard Time", "Asia/Tashkent"),
    ("India Standard Time", "Asia/Kolkata"),
    ("Sri Lanka Standard Time", "Asia/Colombo"),
    ("Nepal Standard Time", "Asia/Kathmandu"),
    ("Bangladesh Standard Time", "Asia/Dhaka"),
    ("Myanmar Standard Time", "Asia/Yangon"),
    ("SE Asia Standard Time", "Asia/Bangkok"),
    ("China Standard Time", "Asia/Shanghai"),
    ("Singapore Standard Time", "Asia/Singapore"),
    ("Taipei Standard Time", "Asia/Taipei"),
    ("W. Australia Standard Time", "Australia/Perth"),
    ("Tokyo Standard Time", "Asia/Tokyo"),
    ("Korea Standard Time", "Asia/Seoul"),
    ("Cen. Australia Standard Time", "Australia/Adelaide"),
    ("AUS Central Standard Time", "Australia/Darwin"),
    ("E. Australia Standard Time", "Australia/Brisbane"),
    ("AUS Eastern Standard Time", "Australia/Sydney"),
    ("Tasmania Standard Time", "Australia/Hobart"),
    ("New Zealand Standard Time", "Pacific/Auckland"),
    ("Fiji Standard Time", "Pacific/Fiji"),
    ("Tonga Standard Time", "Pacific/Tongatapu"),
    ("Samoa Standard Time", "Pacific/Apia"),
    ("Azores Standard Time", "Atlantic/Azores"),
    ("Cape Verde Standard Time", "Atlantic/Cape_Verde"),
    ("Mauritius Standard Time", "Indian/Mauritius"),
    ("Caucasus Standard Time", "Asia/Yerevan"),
    ("Georgian Standard Time", "Asia/Tbilisi"),
    ("Azerbaijan Standard Time", "Asia/Baku"),
    ("Ekaterinburg Standard Time", "Asia/Yekaterinburg"),
    ("N. Central Asia Standard Time", "Asia/Novosibirsk"),
    ("North Asia Standard Time", "Asia/Krasnoyarsk"),
    ("North Asia East Standard Time", "Asia/Irkutsk"),
    ("Yakutsk Standard Time", "Asia/Yakutsk"),
    ("Vladivostok Standard Time", "Asia/Vladivostok"),
    ("Magadan Standard Time", "Asia/Magadan"),
];

fn iana(name: &str) -> Option<Tz> {
    if let Ok(zone) = name.parse::<Tz>() {
        return Some(zone);
    }
    chrono_tz::TZ_VARIANTS
        .iter()
        .copied()
        .find(|zone| zone.name().eq_ignore_ascii_case(name))
}

fn windows_zone(name: &str) -> Option<Tz> {
    WINDOWS_ZONES
        .iter()
        .find(|(windows, _)| windows.eq_ignore_ascii_case(name))
        .and_then(|(_, iana_name)| iana(iana_name))
}

// ---------------------------------------------------------------------------
// Lines, components and properties
// ---------------------------------------------------------------------------

/// One content line: upper-case name, parameters, and the raw (still escaped) value.
#[derive(Clone, Debug)]
struct Line {
    name: String,
    params: Vec<(String, String)>,
    value: String,
}

struct Scanned {
    saw_calendar: bool,
    unterminated: bool,
    root: Vec<Line>,
    events: Vec<Vec<Line>>,
}

/// Joins folded lines. A line that starts with a space or tab continues the previous line.
fn unfold(text: &str) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    for raw in text.split('\n') {
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
        match (raw.strip_prefix([' ', '\t']), lines.last_mut()) {
            (Some(rest), Some(last)) => last.push_str(rest),
            _ => lines.push(raw.to_string()),
        }
    }
    lines
}

fn scan(lines: &[String]) -> Scanned {
    let mut scanner = Scanner::default();
    for line in lines {
        scanner.feed(line);
    }
    scanner.finish()
}

/// Root-level calendar properties that the module reads.
const ROOT_PROPERTIES: [&str; 3] = ["X-WR-CALNAME", "X-WR-TIMEZONE", "PRODID"];

#[derive(Default)]
struct Scanner {
    stack: Vec<String>,
    /// BEGIN lines beyond `MAX_NESTING` whose matching END lines are still to come.
    overflow: usize,
    saw_calendar: bool,
    root: Vec<Line>,
    events: Vec<Vec<Line>>,
    open_event: Option<Vec<Line>>,
}

impl Scanner {
    fn feed(&mut self, text: &str) {
        let Some(line) = parse_line(text) else {
            return;
        };
        match line.name.as_str() {
            "BEGIN" => self.begin(&line.value),
            "END" => self.end(&line.value),
            _ => {
                if self.overflow == 0 {
                    self.property(line);
                }
            }
        }
    }

    fn begin(&mut self, component: &str) {
        let component = component.trim().to_ascii_uppercase();
        if self.stack.len() >= MAX_NESTING {
            self.overflow += 1;
            return;
        }
        if component == "VCALENDAR" {
            self.saw_calendar = true;
        }
        if component == "VEVENT" && self.at_calendar_root() {
            self.close_event();
            self.open_event = Some(Vec::new());
        }
        self.stack.push(component);
    }

    fn end(&mut self, component: &str) {
        if self.overflow > 0 {
            self.overflow -= 1;
            return;
        }
        let component = component.trim().to_ascii_uppercase();
        if let Some(position) = self.stack.iter().rposition(|open| *open == component) {
            self.stack.truncate(position);
            if !self.in_top_level_event() {
                self.close_event();
            }
        }
    }

    fn property(&mut self, line: Line) {
        if self.at_calendar_root() {
            if ROOT_PROPERTIES.contains(&line.name.as_str()) {
                self.root.push(line);
            }
            return;
        }
        if !self.in_top_level_event() {
            return;
        }
        if let Some(props) = self.open_event.as_mut() {
            props.push(line);
        }
    }

    fn at_calendar_root(&self) -> bool {
        matches!(self.stack.as_slice(), [root] if root == "VCALENDAR")
    }

    /// True only for a VEVENT directly inside the calendar, so alarms and other nested components never count.
    fn in_top_level_event(&self) -> bool {
        matches!(self.stack.as_slice(), [root, event] if root == "VCALENDAR" && event == "VEVENT")
    }

    fn close_event(&mut self) {
        if let Some(props) = self.open_event.take() {
            self.events.push(props);
        }
    }

    fn finish(mut self) -> Scanned {
        self.close_event();
        Scanned {
            saw_calendar: self.saw_calendar,
            unterminated: self.stack.iter().any(|open| open == "VCALENDAR"),
            root: self.root,
            events: self.events,
        }
    }
}

/// Parses `NAME;PARAM=VALUE;PARAM="quoted;value":VALUE`. The first colon outside quotes ends the header.
fn parse_line(text: &str) -> Option<Line> {
    let colon = unquoted_position(text, ':')?;
    let head = text.get(..colon)?;
    let value = text.get(colon + 1..)?;
    let mut parts = split_unquoted(head, ';').into_iter();
    let name = parts.next()?.trim().to_ascii_uppercase();
    if name.is_empty()
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return None;
    }
    let params = parts
        .filter_map(|part| {
            let (key, raw) = part.split_once('=')?;
            Some((
                key.trim().to_ascii_uppercase(),
                unquote(raw.trim()).to_string(),
            ))
        })
        .collect();
    Some(Line {
        name,
        params,
        value: value.to_string(),
    })
}

fn unquoted_position(text: &str, target: char) -> Option<usize> {
    let mut quoted = false;
    for (index, ch) in text.char_indices() {
        if ch == '"' {
            quoted = !quoted;
        } else if ch == target && !quoted {
            return Some(index);
        }
    }
    None
}

fn split_unquoted(text: &str, separator: char) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut quoted = false;
    for (index, ch) in text.char_indices() {
        if ch == '"' {
            quoted = !quoted;
        } else if ch == separator && !quoted {
            parts.push(text.get(start..index).unwrap_or(""));
            start = index + ch.len_utf8();
        }
    }
    parts.push(text.get(start..).unwrap_or(""));
    parts
}

fn unquote(value: &str) -> &str {
    value
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(value)
}

fn property<'a>(props: &'a [Line], name: &str) -> Option<&'a Line> {
    props.iter().find(|line| line.name == name)
}

fn param<'a>(line: &'a Line, name: &str) -> Option<&'a str> {
    line.params
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, value)| value.as_str())
}

fn text_of(props: &[Line], name: &str) -> Option<String> {
    property(props, name).map(|line| unescape_text(&line.value))
}

/// Undoes RFC 5545 TEXT escaping. Unknown escapes are kept literally.
fn unescape_text(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('n' | 'N') => out.push('\n'),
            Some(',') => out.push(','),
            Some(';') => out.push(';'),
            Some('\\') => out.push('\\'),
            Some(other) => {
                out.push('\\');
                out.push(other);
            }
            None => out.push('\\'),
        }
    }
    out
}

// ---------------------------------------------------------------------------
// Times and durations
// ---------------------------------------------------------------------------

struct TimeContext {
    default_zone: Option<Tz>,
    fallback: Tz,
    guessed: bool,
}

fn parse_time_text(text: &str, line: &Line, ctx: &mut TimeContext) -> Option<IcsTime> {
    let text = text.trim();
    let date_only = param(line, "VALUE").is_some_and(|value| value.eq_ignore_ascii_case("DATE"));
    if date_only || (text.len() == 8 && text.bytes().all(|byte| byte.is_ascii_digit())) {
        return parse_date(text).map(IcsTime::Date);
    }
    let (body, utc) = match text.strip_suffix(['Z', 'z']) {
        Some(body) => (body, true),
        None => (text, false),
    };
    let local = parse_naive_datetime(body)?;
    if utc {
        return Some(IcsTime::Utc(local));
    }
    match param(line, "TZID").filter(|zone| !zone.is_empty()) {
        None => Some(IcsTime::Floating(local)),
        Some(tzid) => match map_tzid(tzid) {
            Some(zone) => Some(IcsTime::Zoned(local, zone)),
            None => {
                ctx.guessed = true;
                Some(IcsTime::Zoned(
                    local,
                    ctx.default_zone.unwrap_or(ctx.fallback),
                ))
            }
        },
    }
}

fn number(text: &str) -> Option<u32> {
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn parse_date(text: &str) -> Option<NaiveDate> {
    if text.len() != 8 {
        return None;
    }
    let year = i32::try_from(number(text.get(0..4)?)?).ok()?;
    let month = number(text.get(4..6)?)?;
    let day = number(text.get(6..8)?)?;
    NaiveDate::from_ymd_opt(year, month, day)
}

fn parse_naive_datetime(text: &str) -> Option<NaiveDateTime> {
    let (date, time) = text.split_once(['T', 't'])?;
    let date = parse_date(date)?;
    if time.len() != 4 && time.len() != 6 {
        return None;
    }
    let hour = number(time.get(0..2)?)?;
    let minute = number(time.get(2..4)?)?;
    let second = if time.len() == 6 {
        number(time.get(4..6)?)?
    } else {
        0
    };
    let time = NaiveTime::from_hms_opt(hour, minute, second)?;
    Some(date.and_time(time))
}

/// Parses `[+-]P[nW][nD][T[nH][nM][nS]]` into signed seconds.
fn parse_duration(text: &str) -> Option<i64> {
    let text = text.trim().to_ascii_uppercase();
    let (sign, rest) = if let Some(rest) = text.strip_prefix('-') {
        (-1, rest)
    } else if let Some(rest) = text.strip_prefix('+') {
        (1, rest)
    } else {
        (1, text.as_str())
    };
    let body = rest.strip_prefix('P')?;
    let mut total: i64 = 0;
    let mut amount: Option<i64> = None;
    let mut in_time = false;
    let mut units = 0usize;
    for ch in body.chars() {
        if let Some(digit) = ch.to_digit(10) {
            let next = amount
                .unwrap_or(0)
                .checked_mul(10)?
                .checked_add(i64::from(digit))?;
            amount = Some(next);
            continue;
        }
        if ch == 'T' && !in_time && amount.is_none() {
            in_time = true;
            continue;
        }
        let value = amount.take()?;
        let seconds_per_unit: i64 = match (ch, in_time) {
            ('W', false) => 604_800,
            ('D', false) => 86_400,
            ('H', true) => 3_600,
            ('M', true) => 60,
            ('S', true) => 1,
            _ => return None,
        };
        total = total.checked_add(value.checked_mul(seconds_per_unit)?)?;
        units += 1;
    }
    if amount.is_some() || units == 0 {
        return None;
    }
    Some(sign * total)
}

/// Adds a duration to a start. All-day durations round up to whole days, with a minimum of one day.
fn add_duration(start: &IcsTime, seconds: i64) -> IcsTime {
    match start {
        IcsTime::Date(date) => {
            let days = seconds.saturating_add(86_399).div_euclid(86_400).max(1);
            let days = u64::try_from(days).unwrap_or(1);
            IcsTime::Date(date.checked_add_days(Days::new(days)).unwrap_or(*date))
        }
        IcsTime::Utc(local) => IcsTime::Utc(shift(local, seconds)),
        IcsTime::Zoned(local, zone) => IcsTime::Zoned(shift(local, seconds), *zone),
        IcsTime::Floating(local) => IcsTime::Floating(shift(local, seconds)),
    }
}

fn shift(local: &NaiveDateTime, seconds: i64) -> NaiveDateTime {
    TimeDelta::try_seconds(seconds)
        .and_then(|delta| local.checked_add_signed(delta))
        .unwrap_or(*local)
}

fn end_of(event: &IcsEvent) -> IcsTime {
    event
        .end
        .clone()
        .unwrap_or_else(|| add_duration(&event.start, 0))
}

/// The calendar day an instant, date or local time falls on in its own frame.
fn date_of(time: &IcsTime) -> NaiveDate {
    match time {
        IcsTime::Date(date) => *date,
        IcsTime::Utc(local) | IcsTime::Zoned(local, _) | IcsTime::Floating(local) => local.date(),
    }
}

fn identity(time: &IcsTime) -> String {
    match time {
        IcsTime::Date(date) => format!("D{date}"),
        IcsTime::Utc(local) => format!("U{local}"),
        IcsTime::Zoned(local, zone) => format!("Z{local}{}", zone.name()),
        IcsTime::Floating(local) => format!("F{local}"),
    }
}

// ---------------------------------------------------------------------------
// Event construction
// ---------------------------------------------------------------------------

fn build_event(props: &[Line], default_zone: Option<Tz>, fallback: Tz) -> Option<IcsEvent> {
    let mut ctx = TimeContext {
        default_zone,
        fallback,
        guessed: false,
    };
    let uid = text_of(props, "UID").filter(|uid| !uid.trim().is_empty())?;
    let start_line = property(props, "DTSTART")?;
    let start = parse_time_text(&start_line.value, start_line, &mut ctx)?;
    let dtend = match property(props, "DTEND") {
        Some(line) => Some(parse_time_text(&line.value, line, &mut ctx)?),
        None => None,
    };
    let duration = property(props, "DURATION").and_then(|line| parse_duration(&line.value));
    let end = dtend.unwrap_or_else(|| add_duration(&start, duration.unwrap_or(0)));
    let recurrence_id = match property(props, "RECURRENCE-ID") {
        Some(line) => Some(parse_time_text(&line.value, line, &mut ctx)?),
        None => None,
    };
    let rrule = property(props, "RRULE")
        .map(|line| line.value.trim().to_string())
        .filter(|rule| !rule.is_empty());
    let rdates = list_times(props, "RDATE", &mut ctx);
    let exdates = list_times(props, "EXDATE", &mut ctx);
    let status = property(props, "STATUS")
        .map(|line| line.value.trim().to_ascii_uppercase())
        .unwrap_or_default();
    let transparent = property(props, "TRANSP")
        .is_some_and(|line| line.value.trim().eq_ignore_ascii_case("TRANSPARENT"));
    let sequence = property(props, "SEQUENCE")
        .and_then(|line| line.value.trim().parse::<i64>().ok())
        .unwrap_or(0);
    let summary = text_of(props, "SUMMARY")
        .filter(|summary| !summary.trim().is_empty())
        .unwrap_or_else(|| "(No title)".to_string());
    let location = text_of(props, "LOCATION").filter(|text| !text.trim().is_empty());
    let url = text_of(props, "URL").filter(|text| !text.trim().is_empty());

    Some(IcsEvent {
        uid,
        summary,
        location,
        url,
        start,
        end: Some(end),
        rrule,
        rdates,
        exdates,
        recurrence_id,
        cancelled: status == "CANCELLED",
        tentative: status == "TENTATIVE",
        transparent,
        sequence,
        zone_guessed: ctx.guessed,
    })
}

/// Collects every item of a repeatable list property. Unparsable items are ignored.
/// PERIOD values contribute only their start.
fn list_times(props: &[Line], name: &str, ctx: &mut TimeContext) -> Vec<IcsTime> {
    let mut times = Vec::new();
    for line in props.iter().filter(|line| line.name == name) {
        for item in line.value.split(',') {
            let start = item.split('/').next().unwrap_or("");
            if let Some(time) = parse_time_text(start, line, ctx) {
                times.push(time);
            }
        }
    }
    times
}

// ---------------------------------------------------------------------------
// Local time resolution
// ---------------------------------------------------------------------------

/// Resolves a local wall time to an instant. Ambiguous times take the earlier instant;
/// times in a spring-forward gap move forward one hour and are resolved again.
fn resolve<Z: TimeZone>(zone: &Z, local: NaiveDateTime) -> Option<DateTime<Z>> {
    match zone.from_local_datetime(&local) {
        LocalResult::Single(at) | LocalResult::Ambiguous(at, _) => Some(at),
        LocalResult::None => {
            let shifted = local.checked_add_signed(TimeDelta::try_hours(1)?)?;
            match zone.from_local_datetime(&shifted) {
                LocalResult::Single(at) | LocalResult::Ambiguous(at, _) => Some(at),
                LocalResult::None => None,
            }
        }
    }
}

fn local_ms(zone: Tz, local: NaiveDateTime) -> Option<i64> {
    resolve(&zone, local).map(|at| at.timestamp_millis())
}

#[derive(Clone, Copy)]
struct Zones {
    /// Zone for floating times: the calendar default, else the caller's zone.
    floating: Tz,
    /// Zone whose local midnights bound all-day occurrences.
    dates: Tz,
}

impl Zones {
    fn midnight(self, date: NaiveDate) -> Option<i64> {
        local_ms(self.dates, date.and_hms_opt(0, 0, 0)?)
    }

    /// Epoch milliseconds for a time. `hint` is the zone for floating times.
    fn instant(self, time: &IcsTime, hint: Tz) -> Option<i64> {
        match time {
            IcsTime::Date(date) => self.midnight(*date),
            IcsTime::Utc(local) => Some(local.and_utc().timestamp_millis()),
            IcsTime::Zoned(local, zone) => local_ms(*zone, *local),
            IcsTime::Floating(local) => local_ms(hint, *local),
        }
    }
}

// ---------------------------------------------------------------------------
// Occurrence placement
// ---------------------------------------------------------------------------

/// Identity of an instance: its original start, as a calendar day or an epoch instant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Slot {
    Day(NaiveDate),
    Instant(i64),
}

#[derive(Clone, Copy)]
struct Window {
    from: i64,
    to: i64,
}

#[derive(Clone, Debug)]
struct Placed {
    span: OccurrenceSpan,
    /// Lower and upper bounds for window tests, in epoch milliseconds.
    lo: i64,
    hi: i64,
}

impl Placed {
    /// Half-open overlap with the window. Zero-length spans count when their start is inside it.
    fn overlaps(&self, window: Window) -> bool {
        if self.hi == self.lo {
            self.lo >= window.from && self.lo < window.to
        } else {
            self.lo < window.to && self.hi > window.from
        }
    }
}

fn day_placed(zones: Zones, start: NaiveDate, end_exclusive: NaiveDate) -> Option<Placed> {
    Some(Placed {
        span: OccurrenceSpan::Dates {
            start,
            end_exclusive,
        },
        lo: zones.midnight(start)?,
        hi: zones.midnight(end_exclusive)?,
    })
}

fn zone_of(start: &IcsTime, zones: Zones) -> Option<Tz> {
    match start {
        IcsTime::Date(_) | IcsTime::Utc(_) => None,
        IcsTime::Zoned(_, zone) => Some(*zone),
        IcsTime::Floating(_) => Some(zones.floating),
    }
}

/// Places one event from its own start and end. Used for overrides and non-recurring events.
fn place_event(event: &IcsEvent, zones: Zones) -> Option<Placed> {
    let end = end_of(event);
    match &event.start {
        IcsTime::Date(start) => {
            let mut end_date = date_of(&end);
            if end_date <= *start {
                end_date = start.checked_add_days(Days::new(1))?;
            }
            day_placed(zones, *start, end_date)
        }
        start => {
            let start_ms = zones.instant(start, zones.floating)?;
            let end_ms = zones.instant(&end, zones.floating)?.max(start_ms);
            Some(Placed {
                span: OccurrenceSpan::Timed {
                    start_ms,
                    end_ms,
                    zone: zone_of(start, zones),
                },
                lo: start_ms,
                hi: end_ms,
            })
        }
    }
}

/// How a recurring series (or a single event) places its instances.
#[derive(Clone, Copy)]
struct Profile {
    /// The master starts on a calendar day rather than a time.
    day: bool,
    /// Zone for local start times, rule datetimes and UNTIL values. UTC for day and UTC series.
    zone: Tz,
    /// Zone attached to timed occurrences; `None` for UTC.
    occurrence_zone: Option<Tz>,
    local_start: NaiveDateTime,
    duration_ms: i64,
    days: i64,
}

impl Profile {
    fn new(master: &IcsEvent, zones: Zones) -> Option<Self> {
        let end = end_of(master);
        let (day, zone, occurrence_zone, local_start) = match &master.start {
            IcsTime::Date(date) => (true, chrono_tz::UTC, None, date.and_hms_opt(0, 0, 0)?),
            IcsTime::Utc(local) => (false, chrono_tz::UTC, None, *local),
            IcsTime::Zoned(local, zone) => (false, *zone, Some(*zone), *local),
            IcsTime::Floating(local) => (false, zones.floating, Some(zones.floating), *local),
        };
        let (days, duration_ms) = if day {
            let days = (date_of(&end) - date_of(&master.start)).num_days().max(1);
            (days, 0)
        } else {
            let start = zones.instant(&master.start, zone)?;
            let finish = zones.instant(&end, zone)?;
            (1, finish.saturating_sub(start).max(0))
        };
        Some(Self {
            day,
            zone,
            occurrence_zone,
            local_start,
            duration_ms,
            days,
        })
    }

    fn dt_start(&self) -> Option<DateTime<RruleTz>> {
        resolve(&RruleTz::Tz(self.zone), self.local_start)
    }

    fn slot_for(&self, time: &IcsTime, zones: Zones) -> Option<Slot> {
        if self.day {
            return Some(Slot::Day(date_of(time)));
        }
        match time {
            IcsTime::Date(_) => None,
            other => zones.instant(other, self.zone).map(Slot::Instant),
        }
    }

    fn slot_of(&self, at: &DateTime<RruleTz>) -> Slot {
        if self.day {
            Slot::Day(at.date_naive())
        } else {
            Slot::Instant(at.timestamp_millis())
        }
    }

    fn place(&self, slot: Slot, zones: Zones) -> Option<Placed> {
        match slot {
            Slot::Day(start) => {
                let end = start.checked_add_days(Days::new(u64::try_from(self.days).ok()?))?;
                day_placed(zones, start, end)
            }
            Slot::Instant(start) => {
                let end = start.saturating_add(self.duration_ms);
                Some(Placed {
                    span: OccurrenceSpan::Timed {
                        start_ms: start,
                        end_ms: end,
                        zone: self.occurrence_zone,
                    },
                    lo: start,
                    hi: end,
                })
            }
        }
    }
}

fn key_of(slot: Slot) -> String {
    match slot {
        Slot::Day(date) => date.format("%Y%m%d").to_string(),
        Slot::Instant(ms) => DateTime::<Utc>::from_timestamp_millis(ms)
            .map_or_else(String::new, |at| at.format("%Y%m%dT%H%M%SZ").to_string()),
    }
}

fn override_key(recurrence_id: &IcsTime, hint: Tz, zones: Zones) -> Option<String> {
    match recurrence_id {
        IcsTime::Date(date) => Some(key_of(Slot::Day(*date))),
        other => zones
            .instant(other, hint)
            .map(|ms| key_of(Slot::Instant(ms))),
    }
}

// ---------------------------------------------------------------------------
// Recurrence rules
// ---------------------------------------------------------------------------

/// Builds a validated rule set for the profile. Returns `None` when the rule cannot be used.
fn rule_set(raw: &str, profile: &Profile) -> Option<RRuleSet> {
    let start = profile.dt_start()?;
    let text = normalize_rule(raw, profile.zone);
    if has_zero_value(&text) {
        return None;
    }
    let rule: RRule<Unvalidated> = text.parse().ok()?;
    let rule = rule.validate(start).ok()?;
    Some(RRuleSet::new(start).limit().rrule(rule))
}

/// rrule drops a zero BYMONTHDAY and fills in the DTSTART day instead, and it treats
/// INTERVAL=0 as an empty rule. Both are malformed, so they must count as unexpandable.
fn has_zero_value(rule: &str) -> bool {
    rule.split(';').any(|part| match part.split_once('=') {
        Some(("BYMONTHDAY", values)) => values
            .split(',')
            .any(|value| value.trim().parse::<i32>() == Ok(0)),
        Some(("INTERVAL", value)) => value.trim().parse::<u32>() == Ok(0),
        _ => false,
    })
}

/// Upper-cases the rule and rewrites every UNTIL to UTC. The rrule crate reads a
/// zone-less UNTIL in the host's local zone, which would make results machine-dependent.
fn normalize_rule(raw: &str, zone: Tz) -> String {
    let upper = raw.trim().to_ascii_uppercase();
    let body = upper.strip_prefix("RRULE:").unwrap_or(&upper);
    body.split(';')
        .map(|part| match part.strip_prefix("UNTIL=") {
            Some(value) => format!("UNTIL={}", normalize_until(value, zone)),
            None => part.to_string(),
        })
        .collect::<Vec<_>>()
        .join(";")
}

/// A date-only UNTIL means the end of that day in the series zone.
fn normalize_until(value: &str, zone: Tz) -> String {
    if value.ends_with('Z') {
        return value.to_string();
    }
    let local = if value.len() == 8 {
        parse_date(value).and_then(|date| date.and_hms_opt(23, 59, 59))
    } else {
        parse_naive_datetime(value)
    };
    match local.and_then(|local| resolve(&zone, local)) {
        Some(at) => at.with_timezone(&Utc).format("%Y%m%dT%H%M%SZ").to_string(),
        None => value.to_string(),
    }
}

// ---------------------------------------------------------------------------
// Expansion
// ---------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct Row {
    lo: i64,
    occurrence: Occurrence,
}

struct Override<'a> {
    event: &'a IcsEvent,
    placed: Option<Placed>,
    key: Option<String>,
}

fn occurrence(event: &IcsEvent, key: Option<String>, span: OccurrenceSpan) -> Occurrence {
    Occurrence {
        uid: event.uid.clone(),
        occurrence_key: key,
        summary: event.summary.clone(),
        location: event.location.clone(),
        url: event.url.clone(),
        span,
        tentative: event.tentative,
        transparent: event.transparent,
        zone_guessed: event.zone_guessed,
    }
}

/// Expands one master (or one orphaned set of overrides) within the window.
struct Series<'a> {
    master: Option<&'a IcsEvent>,
    profile: Option<Profile>,
    /// The master is recurring (has RRULE or RDATE), so its instances carry keys.
    recurring: bool,
    zones: Zones,
    window: Window,
    overrides: Vec<Override<'a>>,
    /// Original starts replaced by an override. The master instance is suppressed.
    replaced: HashSet<Slot>,
    exdate_slots: HashSet<Slot>,
    /// EXDATEs that were all-day values on a timed series, matched by local date.
    exdate_days: HashSet<NaiveDate>,
    fixed_slots: HashSet<Slot>,
    rows: Vec<Row>,
}

impl<'a> Series<'a> {
    fn new(
        master: Option<&'a IcsEvent>,
        overrides: &[&'a IcsEvent],
        zones: Zones,
        window: Window,
    ) -> Self {
        let profile = master.and_then(|event| Profile::new(event, zones));
        let hint = profile.map_or(zones.floating, |profile| profile.zone);
        let mut replaced = HashSet::new();
        let overrides: Vec<Override<'a>> = overrides
            .iter()
            .filter_map(|&event| {
                let recurrence_id = event.recurrence_id.as_ref()?;
                if let Some(slot) = profile.and_then(|p| p.slot_for(recurrence_id, zones)) {
                    replaced.insert(slot);
                }
                Some(Override {
                    event,
                    placed: place_event(event, zones),
                    key: override_key(recurrence_id, hint, zones),
                })
            })
            .collect();

        let mut exdate_slots = HashSet::new();
        let mut exdate_days = HashSet::new();
        if let (Some(event), Some(profile)) = (master, profile) {
            for exdate in &event.exdates {
                match profile.slot_for(exdate, zones) {
                    Some(slot) => {
                        exdate_slots.insert(slot);
                    }
                    None => {
                        if let IcsTime::Date(date) = exdate {
                            exdate_days.insert(*date);
                        }
                    }
                }
            }
        }

        Self {
            master,
            profile,
            recurring: master
                .is_some_and(|event| event.rrule.is_some() || !event.rdates.is_empty()),
            zones,
            window,
            overrides,
            replaced,
            exdate_slots,
            exdate_days,
            fixed_slots: HashSet::new(),
            rows: Vec::new(),
        }
    }

    /// Returns true when the rule could not be expanded and only fixed instances were kept.
    fn run(&mut self) -> bool {
        let (Some(master), Some(profile)) = (self.master, self.profile) else {
            return false;
        };
        if master.cancelled {
            return false;
        }
        self.add_fixed(master, profile);
        let Some(rule) = master.rrule.as_deref() else {
            return false;
        };
        if self.expand_rule(rule, profile) {
            return false;
        }
        self.rows.clear();
        self.fixed_slots.clear();
        self.add_fixed(master, profile);
        true
    }

    /// DTSTART and RDATEs. DTSTART always counts as an instance, even when the rule would not produce it.
    fn add_fixed(&mut self, event: &IcsEvent, profile: Profile) {
        for time in std::iter::once(&event.start).chain(event.rdates.iter()) {
            let Some(slot) = profile.slot_for(time, self.zones) else {
                continue;
            };
            if !self.fixed_slots.insert(slot) {
                continue;
            }
            let Some(placed) = profile.place(slot, self.zones) else {
                continue;
            };
            if !self.consider(&profile, slot, placed) {
                break;
            }
        }
    }

    /// Returns true when the whole rule was scanned within budget.
    fn expand_rule(&mut self, rule: &str, profile: Profile) -> bool {
        let Some(set) = rule_set(rule, &profile) else {
            return false;
        };
        let dt_start = profile.dt_start();
        let count = set
            .get_rrule()
            .first()
            .and_then(|rule| rule.get_count())
            .and_then(|count| usize::try_from(count).ok());
        let mut examined = 0usize;
        let mut yielded = 0usize;
        let mut start_in_rule = false;
        for at in &set {
            examined += 1;
            if examined > SCAN_BUDGET {
                return false;
            }
            if examined == 1 {
                start_in_rule = dt_start.as_ref() == Some(&at);
            }
            yielded += 1;
            // RFC 5545 counts DTSTART as the first member of the set. When the rule does not
            // produce DTSTART, the rule's COUNT leaves room for one fewer of its own instances.
            if let Some(count) = count {
                let allowed = if start_in_rule {
                    count
                } else {
                    count.saturating_sub(1)
                };
                if yielded > allowed {
                    break;
                }
            }
            let slot = profile.slot_of(&at);
            let Some(placed) = profile.place(slot, self.zones) else {
                continue;
            };
            if placed.lo >= self.window.to {
                break;
            }
            if self.fixed_slots.contains(&slot) {
                continue;
            }
            if !self.consider(&profile, slot, placed) {
                break;
            }
        }
        true
    }

    /// Adds one master instance if it is not excluded, replaced, or outside the window.
    /// Returns false when no further instances should be examined.
    fn consider(&mut self, profile: &Profile, slot: Slot, placed: Placed) -> bool {
        if self.rows.len() >= MAX_OCCURRENCES_PER_EVENT {
            return false;
        }
        let Some(event) = self.master else {
            return true;
        };
        if !placed.overlaps(self.window)
            || self.replaced.contains(&slot)
            || self.is_excluded(profile, slot)
        {
            return true;
        }
        let key = self.recurring.then(|| key_of(slot));
        self.rows.push(Row {
            lo: placed.lo,
            occurrence: occurrence(event, key, placed.span),
        });
        true
    }

    fn is_excluded(&self, profile: &Profile, slot: Slot) -> bool {
        if self.exdate_slots.contains(&slot) {
            return true;
        }
        match slot {
            Slot::Instant(ms) if !self.exdate_days.is_empty() => {
                DateTime::<Utc>::from_timestamp_millis(ms).is_some_and(|at| {
                    self.exdate_days
                        .contains(&at.with_timezone(&profile.zone).date_naive())
                })
            }
            _ => false,
        }
    }

    /// Adds overrides. Each is shown by its own span, unless it is cancelled or outside the window.
    /// Overrides are shown even when their original instance was never generated.
    fn finish(self) -> Vec<Row> {
        let mut rows = self.rows;
        let pending: Vec<Row> = self
            .overrides
            .iter()
            .filter_map(|row| {
                if row.event.cancelled {
                    return None;
                }
                let placed = row.placed.as_ref()?;
                let key = row.key.as_ref()?;
                if !placed.overlaps(self.window) {
                    return None;
                }
                Some(Row {
                    lo: placed.lo,
                    occurrence: occurrence(row.event, Some(key.clone()), placed.span.clone()),
                })
            })
            .collect();
        for row in pending {
            if rows.len() >= MAX_OCCURRENCES_PER_EVENT {
                break;
            }
            rows.push(row);
        }
        rows
    }
}

/// Occurrences intersecting `[from_ms, to_ms)`, sorted by (start, uid, key). The second value
/// counts recurring masters whose rule could not be expanded; their DTSTART and RDATE instances
/// are still returned.
///
/// Floating times use the calendar's default zone if it has one, else `floating_zone`. All-day
/// spans are compared as local midnights in `floating_zone`.
pub fn expand(
    cal: &IcsCalendar,
    from_ms: i64,
    to_ms: i64,
    floating_zone: Tz,
) -> (Vec<Occurrence>, usize) {
    if from_ms >= to_ms {
        return (Vec::new(), 0);
    }
    let zones = Zones {
        floating: cal.default_zone.unwrap_or(floating_zone),
        dates: floating_zone,
    };
    let window = Window {
        from: from_ms,
        to: to_ms,
    };

    let mut masters = Vec::new();
    let mut orphans: BTreeMap<&str, Vec<&IcsEvent>> = BTreeMap::new();
    for event in &cal.events {
        if event.recurrence_id.is_some() {
            orphans.entry(event.uid.as_str()).or_default().push(event);
        } else {
            masters.push(event);
        }
    }

    let mut rows = Vec::new();
    let mut unexpanded = 0;
    for master in masters {
        let attached = orphans.remove(master.uid.as_str()).unwrap_or_default();
        let mut series = Series::new(Some(master), &attached, zones, window);
        if series.run() {
            unexpanded += 1;
        }
        rows.extend(series.finish());
    }
    for attached in orphans.values() {
        rows.extend(Series::new(None, attached, zones, window).finish());
    }

    rows.sort_by(|left, right| {
        left.lo
            .cmp(&right.lo)
            .then_with(|| left.occurrence.uid.cmp(&right.occurrence.uid))
            .then_with(|| {
                left.occurrence
                    .occurrence_key
                    .cmp(&right.occurrence.occurrence_key)
            })
    });
    (
        rows.into_iter().map(|row| row.occurrence).collect(),
        unexpanded,
    )
}

/// One occurrence of an event within a window.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Occurrence {
    pub uid: String,
    /// `None` for a non-recurring event; otherwise the original start (`YYYYMMDD` or UTC).
    pub occurrence_key: Option<String>,
    pub summary: String,
    pub location: Option<String>,
    pub url: Option<String>,
    pub span: OccurrenceSpan,
    pub tentative: bool,
    pub transparent: bool,
    pub zone_guessed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OccurrenceSpan {
    /// Epoch milliseconds, with `end_ms >= start_ms`. `zone` is `None` for UTC.
    Timed {
        start_ms: i64,
        end_ms: i64,
        zone: Option<Tz>,
    },
    /// All-day span. `end_exclusive` is the day after the last day.
    Dates {
        start: NaiveDate,
        end_exclusive: NaiveDate,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_windows_zone_entry_maps_to_a_chrono_tz_zone() {
        for (windows, iana_name) in WINDOWS_ZONES {
            let expected: Tz = iana_name
                .parse()
                .unwrap_or_else(|_| panic!("{iana_name} is not a chrono-tz zone"));
            assert_eq!(map_tzid(windows), Some(expected), "{windows}");
        }
    }

    #[test]
    fn path_prefixed_and_case_insensitive_names_map() {
        assert_eq!(
            map_tzid("/mozilla.org/20050126_1/America/New_York"),
            Some(chrono_tz::America::New_York)
        );
        assert_eq!(
            map_tzid("\"eastern standard time\""),
            Some(chrono_tz::America::New_York)
        );
        assert_eq!(map_tzid("Synthetic/Unknown_Zone"), None);
    }

    #[test]
    fn durations_parse_and_reject_malformed_input() {
        assert_eq!(parse_duration("PT1H30M"), Some(5_400));
        assert_eq!(parse_duration("-P1W"), Some(-604_800));
        assert_eq!(parse_duration("P1DT2H"), Some(93_600));
        assert_eq!(parse_duration("PT"), None);
        assert_eq!(parse_duration("P1M"), None);
        assert_eq!(parse_duration("P99999999999999999999D"), None);
    }
}
