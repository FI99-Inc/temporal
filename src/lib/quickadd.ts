// Deterministic one-line quick add. Pure: no clock reads, no system-zone Intl.
// Every date is a 'YYYY-MM-DD' string; all arithmetic goes through Date.UTC.
import type { Weekday } from './calendar-types.ts';

export interface QuickNow { date: string; time: string }
export interface QuickRepeat {
  freq: 'daily' | 'weekly' | 'monthly';
  interval: number;
  weekdays: Weekday[];
  until: string | null;
  count: number | null;
}
export type QuickKind = 'event' | 'deadline' | 'intention';
export interface QuickParse {
  kind: QuickKind;
  title: string;
  date: string;
  endDate: string | null;
  allDay: boolean;
  start: string | null;
  end: string | null;
  location: string | null;
  repeat: QuickRepeat | null;
  explicitDate: boolean;
  notes: string[];
}

const DAY_MS = 86400000;
const WEEKDAYS: Weekday[] = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'];
// JS getUTCDay() returns 0 for Sunday. Map to our Monday-first list.
const JS_DAY_TO_WEEKDAY: Weekday[] = ['sun', 'mon', 'tue', 'wed', 'thu', 'fri', 'sat'];
const MONTHS = ['jan', 'feb', 'mar', 'apr', 'may', 'jun', 'jul', 'aug', 'sep', 'oct', 'nov', 'dec'];
const MONTH_NAMES = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];
const DAY_NAMES = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];

// ---- date helpers (YYYY-MM-DD only; no system zone) ----

interface Ymd { y: number; m: number; d: number }

function daysInMonth(y: number, m: number): number {
  // Day 0 of the following month is the last day of month m (1-based).
  return new Date(Date.UTC(y, m, 0)).getUTCDate();
}

function parseYmd(s: string): Ymd | null {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(s);
  if (!match) return null;
  const y = Number(match[1]), m = Number(match[2]), d = Number(match[3]);
  if (m < 1 || m > 12 || d < 1 || d > daysInMonth(y, m)) return null;
  return { y, m, d };
}

function pad(n: number, width: number): string {
  let s = String(n);
  while (s.length < width) s = '0' + s;
  return s;
}

function formatYmd(y: number, m: number, d: number): string {
  return pad(y, 4) + '-' + pad(m, 2) + '-' + pad(d, 2);
}

// Days since epoch for a valid YYYY-MM-DD.
function dayNumber(s: string): number {
  const p = parseYmd(s);
  if (!p) throw new Error('invalid date ' + s);
  return Date.UTC(p.y, p.m - 1, p.d) / DAY_MS;
}

function dateFromDayNumber(n: number): string {
  const t = new Date(n * DAY_MS);
  return formatYmd(t.getUTCFullYear(), t.getUTCMonth() + 1, t.getUTCDate());
}

function addDays(s: string, n: number): string {
  return dateFromDayNumber(dayNumber(s) + n);
}

function weekdayOf(s: string): Weekday {
  const p = parseYmd(s);
  if (!p) throw new Error('invalid date ' + s);
  return JS_DAY_TO_WEEKDAY[new Date(Date.UTC(p.y, p.m - 1, p.d)).getUTCDay()];
}

// Next date on or after `from` that falls on weekday `wd` (today counts).
function nextOnOrAfter(from: string, wd: Weekday): string {
  const offset = (WEEKDAYS.indexOf(wd) - WEEKDAYS.indexOf(weekdayOf(from)) + 7) % 7;
  return addDays(from, offset);
}

// "next <weekday>": take the next occurrence strictly after today, then add 7 days.
// On a Thursday, "next tue" is the Tuesday 12 days out (Oct 8 -> Oct 20), and
// "next thu" is the Thursday 14 days out (never today, and not the coming Thursday).
function nextWeekOccurrence(from: string, wd: Weekday): string {
  const offset = (WEEKDAYS.indexOf(wd) - WEEKDAYS.indexOf(weekdayOf(from)) + 7) % 7;
  const strictlyAfter = offset === 0 ? 7 : offset;
  return addDays(from, strictlyAfter + 7);
}

function compareYmd(a: string, b: string): number {
  return a < b ? -1 : a > b ? 1 : 0;
}

function plural(n: number, unit: string): string {
  return n + ' ' + unit + (n === 1 ? '' : 's');
}

// Minutes as a short phrase: 60 -> "1 hour", 90 -> "1 hour 30 minutes", 15 -> "15 minutes".
function durationPhrase(minutes: number): string {
  const h = Math.floor(minutes / 60), m = minutes % 60;
  const parts: string[] = [];
  if (h > 0) parts.push(plural(h, 'hour'));
  if (m > 0) parts.push(plural(m, 'minute'));
  return parts.length ? parts.join(' ') : '0 minutes';
}

// ---- failure and masking: consumed grammar is blanked in a "view" so later passes never re-match it ----

class QuickFail extends Error {}
function fail(message: string): never {
  throw new QuickFail(message);
}

interface Ctx {
  orig: string;
  mask: boolean[];
  now: QuickNow;
  notes: string[];
}
interface Hit { start: number; end: number; m: RegExpExecArray }

function viewOf(ctx: Ctx): string {
  let out = '';
  for (let i = 0; i < ctx.orig.length; i++) out += ctx.mask[i] ? ' ' : ctx.orig.charAt(i);
  return out;
}

function maskRange(ctx: Ctx, start: number, end: number): void {
  for (let i = start; i < end; i++) ctx.mask[i] = true;
}

// Visits each match in the current view. `take` returns true to consume the match
// (it is masked) or false to leave it in the title. Returns the number consumed.
function scan(ctx: Ctx, re: RegExp, take: (h: Hit, view: string) => boolean): number {
  const view = viewOf(ctx);
  const g = new RegExp(re.source, re.flags.indexOf('g') >= 0 ? re.flags : re.flags + 'g');
  let consumed = 0;
  let m: RegExpExecArray | null;
  while ((m = g.exec(view)) !== null) {
    const h: Hit = { start: m.index, end: m.index + m[0].length, m };
    if (take(h, view)) {
      maskRange(ctx, h.start, h.end);
      consumed++;
    }
    if (m[0].length === 0) g.lastIndex++;
  }
  return consumed;
}

// ---- kind markers: "maybe", "try to", "want to", trailing "?" ----

function stripIntentionMarkers(ctx: Ctx): boolean {
  let intention = false;
  const lead = /^(?:maybe|try to|want to)\s+/i.exec(ctx.orig);
  if (lead) {
    maskRange(ctx, 0, lead[0].length);
    intention = true;
  }
  const question = /\?\s*$/.exec(ctx.orig);
  if (question) {
    maskRange(ctx, question.index, ctx.orig.length);
    intention = true;
  }
  return intention;
}

// ---- weekday names ----

const DAY_PAT = 'monday|mon|tuesday|tues|tue|wednesday|wed|thursday|thurs|thur|thu|friday|fri|saturday|sat|sunday|sun';
const DAY_ONE = '(?:' + DAY_PAT + ')\\b';
// "mon wed", "mon, wed and fri", "tue/thu", "mon & fri"
const DAY_LIST = DAY_ONE + '(?:(?:\\s*,\\s*|\\s*\\/\\s*|\\s*&\\s*|\\s+and\\s+|\\s+)' + DAY_ONE + ')*';

function dayFromName(name: string): Weekday {
  const key = name.slice(0, 3).toLowerCase();
  const found = WEEKDAYS.indexOf(key as Weekday);
  if (found < 0) throw new Error('quickadd: unknown weekday ' + name);
  return WEEKDAYS[found];
}

// Distinct weekdays named in `text`, sorted Monday first.
function dayListOf(text: string): Weekday[] {
  const named: Weekday[] = [];
  const re = new RegExp(DAY_ONE, 'gi');
  let m: RegExpExecArray | null;
  while ((m = re.exec(text)) !== null) named.push(dayFromName(m[0]));
  return WEEKDAYS.filter((d) => named.indexOf(d) >= 0);
}

// ---- repeat rules (events and intentions only) ----

// `days` is null for a weekly rule that repeats on the start date's own weekday.
interface RepeatSpec { freq: 'daily' | 'weekly' | 'monthly'; interval: number; days: Weekday[] | null }

function parseRepeat(ctx: Ctx): RepeatSpec | null {
  const found: RepeatSpec[] = [];
  const specs: Array<[RegExp, (m: RegExpExecArray) => RepeatSpec]> = [
    [/\bevery\s+other\s+weeks?\b/i, () => ({ freq: 'weekly', interval: 2, days: null })],
    [/\bevery\s+(\d+)\s+(days?|weeks?|months?)\b/i, (m) => {
      const interval = Number(m[1]);
      if (interval < 1) fail('Repeat interval must be at least 1');
      const unit = m[2].toLowerCase();
      if (unit.indexOf('day') === 0) return { freq: 'daily', interval, days: null };
      if (unit.indexOf('week') === 0) return { freq: 'weekly', interval, days: null };
      return { freq: 'monthly', interval, days: null };
    }],
    [/\bevery\s+weekdays?\b|\bweekdays\b/i, () => ({ freq: 'weekly', interval: 1, days: ['mon', 'tue', 'wed', 'thu', 'fri'] })],
    [/\bevery\s+weekends?\b|\bweekends\b/i, () => ({ freq: 'weekly', interval: 1, days: ['sat', 'sun'] })],
    [/\b(?:every\s+day|daily)\b/i, () => ({ freq: 'daily', interval: 1, days: null })],
    [/\b(?:every\s+week|weekly)\b/i, () => ({ freq: 'weekly', interval: 1, days: null })],
    [/\b(?:every\s+month|monthly)\b/i, () => ({ freq: 'monthly', interval: 1, days: null })],
    [new RegExp('\\bevery\\s+(' + DAY_LIST + ')', 'i'), (m) => ({ freq: 'weekly', interval: 1, days: dayListOf(m[1]) })],
  ];
  for (const [re, make] of specs) {
    scan(ctx, re, (h) => {
      if (found.length > 0) fail('Use one repeat rule');
      found.push(make(h.m));
      return true;
    });
  }
  return found.length > 0 ? found[0] : null;
}

// "for 10 weeks", "10 times", "x10": how many occurrences.
function parseCount(ctx: Ctx): number | null {
  const found: number[] = [];
  const patterns = [/\bfor\s+(\d+)\s*(?:weeks?|days?|months?|times?|occurrences?)\b/i, /\b(\d+)\s+times\b/i, /\bx(\d+)\b/i];
  for (const re of patterns) {
    scan(ctx, re, (h) => {
      found.push(Number(h.m[1]));
      return true;
    });
  }
  if (found.length > 1) fail('Use one count');
  if (found.length === 0) return null;
  if (found[0] < 1) fail('Count must be at least 1');
  return found[0];
}

// "for 90m", "for 90 min", "for 1h", "for 1.5h", "for 2 hours", "for 1h30": minutes.
function parseDuration(ctx: Ctx): number | null {
  const found: number[] = [];
  scan(ctx, /\bfor\s+(\d+(?:\.\d+)?)\s*(?:hours?|hrs?|h)(?:(\d{1,2})(?!\d)|\s+(\d{1,2})\s*(?:minutes?|mins?|m))?\b/i, (h) => {
    const extraText = h.m[2] !== undefined ? h.m[2] : h.m[3];
    const extra = extraText === undefined ? 0 : Number(extraText);
    if (extra >= 60) fail('Minutes after an hour must be under 60');
    found.push(Math.round(Number(h.m[1]) * 60) + extra);
    return true;
  });
  scan(ctx, /\bfor\s+(\d+)\s*(?:minutes?|mins?|m)\b/i, (h) => {
    found.push(Number(h.m[1]));
    return true;
  });
  if (found.length > 1) fail('Use one duration');
  if (found.length === 0) return null;
  if (found[0] <= 0) fail('Duration must be more than zero');
  return found[0];
}

// ---- dates ----

const MONTH_PAT = 'january|jan|february|feb|march|mar|april|apr|may|june|jun|july|jul|august|aug|september|sept|sep|october|oct|november|nov|december|dec';
const MON = '(' + MONTH_PAT + ')\\b\\.?';
const DAY_NUM = '(\\d{1,2})(?:st|nd|rd|th)?';
const YEAR_OPT = '(?:,?\\s+(20\\d{2}))?';

function monthIndex(name: string): number {
  const found = MONTHS.indexOf(name.slice(0, 3).toLowerCase());
  if (found < 0) throw new Error('quickadd: unknown month ' + name);
  return found + 1;
}

function yearOf(ymd: string): number {
  return Number(ymd.slice(0, 4));
}

// Month and day with an optional year. Without a year: this year, or the next valid
// occurrence when that date is already past. Always noted.
function resolveMonthDay(ctx: Ctx, month: number, day: number, year: number | null): string {
  if (year !== null) {
    const exact = formatYmd(year, month, day);
    if (!parseYmd(exact)) fail('That date does not exist');
    return exact;
  }
  const base = yearOf(ctx.now.date);
  for (let y = base; y <= base + 8; y++) {
    const candidate = formatYmd(y, month, day);
    if (parseYmd(candidate) && compareYmd(candidate, ctx.now.date) >= 0) {
      ctx.notes.push('No year given; read as ' + y);
      return candidate;
    }
  }
  fail('That date does not exist');
}

function checkedYmd(s: string): string {
  if (!parseYmd(s)) fail('That date does not exist');
  return s;
}

// The next "on the Nth": this month if still ahead, otherwise the next month that has it.
function nextOnTheDay(ctx: Ctx, day: number): string {
  if (day < 1 || day > 31) fail('That date does not exist');
  let y = yearOf(ctx.now.date);
  let mo = Number(ctx.now.date.slice(5, 7));
  for (let i = 0; i < 25; i++) {
    const candidate = formatYmd(y, mo, day);
    if (parseYmd(candidate) && compareYmd(candidate, ctx.now.date) >= 0) return candidate;
    mo++;
    if (mo > 12) {
      mo = 1;
      y++;
    }
  }
  fail('That date does not exist');
}

interface DateRes { date: string; endDate: string | null; tonight: boolean }

// Finds every date token. A date right after "until" is recorded as the repeat's until
// date (and the word "until" is consumed with it); any other date is the start date.
function parseDates(ctx: Ctx): { date: string | null; endDate: string | null; until: string | null; tonight: boolean } {
  const today = ctx.now.date;
  const primary: DateRes[] = [];
  const untils: DateRes[] = [];
  const record = (h: Hit, view: string, res: DateRes): boolean => {
    const lead = /\buntil\s+$/i.exec(view.slice(0, h.start));
    if (lead) {
      maskRange(ctx, lead.index, h.start);
      untils.push(res);
    } else {
      primary.push(res);
    }
    return true;
  };
  const single = (date: string, tonight = false): DateRes => ({ date, endDate: null, tonight });

  // ISO ranges and single dates
  scan(ctx, /\b(\d{4})-(\d{2})-(\d{2})\s*(?:-|–|—|to)\s*(\d{4})-(\d{2})-(\d{2})\b/i, (h, view) => {
    const s = checkedYmd(h.m[1] + '-' + h.m[2] + '-' + h.m[3]);
    const e = checkedYmd(h.m[4] + '-' + h.m[5] + '-' + h.m[6]);
    if (compareYmd(e, s) < 0) fail('The end date is before the start');
    return record(h, view, { date: s, endDate: e === s ? null : e, tonight: false });
  });
  scan(ctx, /\b(\d{4})-(\d{2})-(\d{2})\b/i, (h, view) => record(h, view, single(checkedYmd(h.m[0]))));

  // Month and day, with optional range end and year
  scan(ctx, new RegExp('\\b' + MON + '\\s+' + DAY_NUM + '\\s*(?:-|–|—|to)\\s*(?:' + MON + '\\s+)?' + DAY_NUM + YEAR_OPT + '\\b', 'i'), (h, view) => {
    const month = monthIndex(h.m[1]);
    const year = h.m[5] !== undefined ? Number(h.m[5]) : null;
    const s = resolveMonthDay(ctx, month, Number(h.m[2]), year);
    const endMonth = h.m[3] !== undefined ? monthIndex(h.m[3]) : month;
    const e = checkedYmd(formatYmd(year !== null ? year : yearOf(s), endMonth, Number(h.m[4])));
    if (compareYmd(e, s) < 0) fail('The end date is before the start');
    return record(h, view, { date: s, endDate: e === s ? null : e, tonight: false });
  });
  scan(ctx, new RegExp('\\b' + MON + '\\s+' + DAY_NUM + YEAR_OPT + '\\b', 'i'), (h, view) => {
    const year = h.m[3] !== undefined ? Number(h.m[3]) : null;
    return record(h, view, single(resolveMonthDay(ctx, monthIndex(h.m[1]), Number(h.m[2]), year)));
  });
  scan(ctx, new RegExp('\\b' + DAY_NUM + '\\s+' + MON + YEAR_OPT + '\\b', 'i'), (h, view) => {
    const year = h.m[3] !== undefined ? Number(h.m[3]) : null;
    return record(h, view, single(resolveMonthDay(ctx, monthIndex(h.m[2]), Number(h.m[1]), year)));
  });

  // Numeric: 10/12 is month/day
  scan(ctx, /\b(\d{1,2})\/(\d{1,2})(?:\/(20\d{2}))?\b/, (h, view) => {
    ctx.notes.push('Read ' + h.m[1] + '/' + h.m[2] + (h.m[3] !== undefined ? '/' + h.m[3] : '') + ' as month/day');
    const year = h.m[3] !== undefined ? Number(h.m[3]) : null;
    return record(h, view, single(resolveMonthDay(ctx, Number(h.m[1]), Number(h.m[2]), year)));
  });

  // Weekdays. "next <day>" is the following week's occurrence; a bare day is the next one on or after today.
  scan(ctx, new RegExp('\\bnext\\s+(' + DAY_PAT + ')\\b', 'i'), (h, view) => record(h, view, single(nextWeekOccurrence(today, dayFromName(h.m[1])))));
  scan(ctx, new RegExp('\\b(' + DAY_PAT + ')\\s*(?:-|–|—|to)\\s*(' + DAY_PAT + ')\\b', 'i'), (h, view) => {
    const s = nextOnOrAfter(today, dayFromName(h.m[1]));
    const e = nextOnOrAfter(s, dayFromName(h.m[2]));
    return record(h, view, { date: s, endDate: e === s ? null : e, tonight: false });
  });
  scan(ctx, /\bon\s+the\s+(\d{1,2})(?:st|nd|rd|th)?\b/i, (h, view) => {
    const found = nextOnTheDay(ctx, Number(h.m[1]));
    ctx.notes.push('Read "' + h.m[0] + '" as the next such date');
    return record(h, view, single(found));
  });
  scan(ctx, new RegExp('\\b(?:on\\s+)?(' + DAY_PAT + ')\\b', 'i'), (h, view) => record(h, view, single(nextOnOrAfter(today, dayFromName(h.m[1])))));

  // Relative
  scan(ctx, /\bin\s+(\d{1,3})\s+(days?|weeks?)\b/i, (h, view) => {
    const days = h.m[2].toLowerCase().indexOf('week') === 0 ? Number(h.m[1]) * 7 : Number(h.m[1]);
    return record(h, view, single(addDays(today, days)));
  });
  scan(ctx, /\b(today|tonight|tomorrow|tmrw|tmr)\b/i, (h, view) => {
    const word = h.m[1].toLowerCase();
    if (word === 'tomorrow' || word === 'tmr' || word === 'tmrw') return record(h, view, single(addDays(today, 1)));
    return record(h, view, single(today, word === 'tonight'));
  });

  if (primary.length > 1) fail('Use one date');
  if (untils.length > 1) fail('Use one until date');
  const until = untils.length > 0 ? untils[0] : null;
  if (until !== null && until.endDate !== null) fail('Until takes one date');
  const first = primary.length > 0 ? primary[0] : null;
  return {
    date: first !== null ? first.date : null,
    endDate: first !== null ? first.endDate : null,
    until: until !== null ? until.date : null,
    tonight: first !== null && first.tonight,
  };
}

// ---- times ----

interface Clock { h: number; m: number; suffix: 'am' | 'pm' | null; hasMinutes: boolean; word: number | null; label: string }

// One side of a range: "3", "3:30pm", "3.30pm", "15:00", "9a", "noon", "midnight".
// The single letters "a" and "p" only attach directly ("9a"), so "2 a book" stays a title.
const TIME_PART = '\\d{1,2}(?:[:.]\\d{2})?(?:\\s*(?:am|pm)\\b|[ap]\\b)?|noon|midnight';

// Only a colon marks a 24-hour clock ("15:00"). A dotted minute without am/pm ("3.30-4pm")
// is an hour that takes its am/pm from the other side, like a bare hour.
function makeClock(hh: string, mm: string | undefined, sfx: string | undefined, label: string): Clock {
  const suffix = sfx === undefined ? null : sfx.toLowerCase().charAt(0) === 'p' ? 'pm' : 'am';
  return { h: Number(hh), m: mm === undefined ? 0 : Number(mm), suffix, hasMinutes: mm !== undefined && label.indexOf(':') >= 0, word: null, label };
}

function clockFromPart(text: string): Clock {
  const lower = text.toLowerCase();
  if (lower === 'noon') return { h: 12, m: 0, suffix: null, hasMinutes: false, word: 720, label: text };
  if (lower === 'midnight') return { h: 0, m: 0, suffix: null, hasMinutes: false, word: 0, label: text };
  const m = /^(\d{1,2})(?:[:.](\d{2}))?\s*(am|pm|a|p)?$/i.exec(lower);
  if (!m) fail('Check the time ' + text);
  return makeClock(m[1], m[2], m[3], text);
}

// Minutes after midnight. `suffix` overrides the clock's own suffix (used when inferring).
function minutesOf(c: Clock, suffix: 'am' | 'pm' | null): number {
  if (c.word !== null) return c.word;
  if (c.m > 59) fail('Check the time ' + c.label);
  if (suffix !== null) {
    if (c.h < 1 || c.h > 12) fail('Check the time ' + c.label);
    return ((c.h % 12) + (suffix === 'pm' ? 12 : 0)) * 60 + c.m;
  }
  if (c.h > 23) fail('Check the time ' + c.label);
  return c.h * 60 + c.m;
}

// A bare number ("3" in "3-4pm") takes the other side's am/pm when that reads sensibly,
// and otherwise the other reading. Hours above 12 or 0 can only be 24-hour.
function bareCandidates(h: number, other: 'am' | 'pm' | null): Array<'am' | 'pm' | null> {
  if (other === null || h === 0 || h > 12) return [null];
  return [other, other === 'am' ? 'pm' : 'am'];
}

// Returns null when neither side is a clock (for example "12-14"), so the text stays in the title.
function timeRangeOf(a: Clock, b: Clock): { start: number; end: number } | null {
  const aBare = a.word === null && !a.hasMinutes && a.suffix === null;
  const bBare = b.word === null && !b.hasMinutes && b.suffix === null;
  if (aBare && bBare) return null;
  if (aBare) {
    const end = minutesOf(b, b.suffix);
    for (const s of bareCandidates(a.h, b.suffix)) {
      if (minutesOf(a, s) < end) return { start: minutesOf(a, s), end };
    }
    fail('Add am or pm to the start time');
  }
  if (bBare) {
    const start = minutesOf(a, a.suffix);
    for (const s of bareCandidates(b.h, a.suffix)) {
      if (minutesOf(b, s) > start) return { start, end: minutesOf(b, s) };
    }
    fail('Add am or pm to the end time');
  }
  return { start: minutesOf(a, a.suffix), end: minutesOf(b, b.suffix) };
}

function parseTimes(ctx: Ctx): { start: number | null; end: number | null } {
  const found: Array<{ start: number; end: number | null }> = [];
  const single = (start: number): boolean => {
    found.push({ start, end: null });
    return true;
  };

  // Ranges: "3-4pm", "11-1pm", "3pm to 5pm", "from 3 to 5pm", "15:00–16:00"
  scan(ctx, new RegExp('(?:\\b(?:from|at)\\s+|@\\s*)?\\b(' + TIME_PART + ')\\s*(?:-|–|—|to)\\s*(' + TIME_PART + ')\\b', 'i'), (h) => {
    const range = timeRangeOf(clockFromPart(h.m[1]), clockFromPart(h.m[2]));
    if (range === null) return false;
    found.push({ start: range.start, end: range.end });
    return true;
  });
  // Single with am/pm: "3pm", "3:30pm", "3.30pm", "9a"
  scan(ctx, /(?:\bat\s+|@\s*)?\b(\d{1,2})(?:[:.](\d{2}))?(?:\s*(am|pm)|([ap]))\b/i, (h) => {
    const c = makeClock(h.m[1], h.m[2], h.m[3] !== undefined ? h.m[3] : h.m[4], h.m[0]);
    return single(minutesOf(c, c.suffix));
  });
  // 24-hour with colon: "15:00". After "at", 1 to 7 reads as PM (noted).
  scan(ctx, /(?:\bat\s+|@\s*)?\b(\d{1,2}):(\d{2})\b/i, (h) => {
    let hour = Number(h.m[1]);
    const minute = h.m[2];
    if (/^(at|@)/i.test(h.m[0]) && hour >= 1 && hour <= 7) {
      ctx.notes.push('Read ' + h.m[1] + ':' + minute + ' as ' + h.m[1] + ':' + minute + ' PM');
      hour += 12;
    }
    return single(minutesOf(makeClock(String(hour), minute, undefined, h.m[0]), null));
  });
  scan(ctx, /(?:\bat\s+|@\s*)?\b(noon|midnight)\b/i, (h) => single(h.m[1].toLowerCase() === 'noon' ? 720 : 0));
  // "at 3" reads as 3 PM for 1 to 7, otherwise 24-hour. Bare numbers elsewhere are not times.
  scan(ctx, /\bat\s+(\d{1,2})\b(?!\s*(?:[:.]|-|–|—|to\b|am\b|pm\b|a\b|p\b))/i, (h) => {
    const n = Number(h.m[1]);
    if (n > 23) return false;
    if (n >= 1 && n <= 7) {
      ctx.notes.push('Read ' + n + ' as ' + n + ' PM');
      return single((n + 12) * 60);
    }
    ctx.notes.push('Read ' + n + ' as ' + (n < 10 ? '0' : '') + n + ':00');
    return single(n * 60);
  });

  if (found.length > 1) fail('Use one time');
  if (found.length === 0) return { start: null, end: null };
  return { start: found[0].start, end: found[0].end };
}

// ---- location: "at <place>" or "@ <place>", running to the next consumed token or the end ----

function parseLocation(ctx: Ctx): string | null {
  const view = viewOf(ctx);
  const m = /(?:^|\s)(?:at\s+|@\s*)(?=\S)/i.exec(view);
  if (!m) return null;
  const markerStart = m.index + (m[0].charAt(0) === ' ' || m[0].charAt(0) === '\t' ? 1 : 0);
  const textStart = m.index + m[0].length;
  let cutoff = textStart;
  while (cutoff < ctx.orig.length && !ctx.mask[cutoff]) cutoff++;
  const text = ctx.orig.slice(textStart, cutoff).trim();
  if (text === '') return null;
  maskRange(ctx, markerStart, cutoff);
  return text;
}

// ---- assembly ----

function hhmm(minutes: number): string {
  return pad(Math.floor(minutes / 60), 2) + ':' + pad(minutes % 60, 2);
}

function earliestOnOrAfter(from: string, days: Weekday[]): string {
  let best: string | null = null;
  for (const d of days) {
    const candidate = nextOnOrAfter(from, d);
    if (best === null || compareYmd(candidate, best) < 0) best = candidate;
  }
  if (best === null) throw new Error('quickadd: a weekly rule needs at least one weekday');
  return best;
}

// Whatever is left after consuming tokens is the title. Only a lone "x" is treated as empty.
function cleanTitle(ctx: Ctx): string {
  const title = viewOf(ctx).replace(/\s+/g, ' ').replace(/^[\s,;:\-–—]+|[\s,;:\-–—]+$/g, '').trim();
  if (title === '' || title.toLowerCase() === 'x') fail('Add a title');
  return title;
}

function build(text: string, now: QuickNow, defaultMinutes: number): QuickParse {
  const orig = text.trim();
  if (orig === '') fail('Type something to add');
  if (!parseYmd(now.date) || !/^\d{2}:\d{2}$/.test(now.time)) {
    throw new Error('quickadd: now needs a local YYYY-MM-DD date and HH:MM time');
  }
  const ctx: Ctx = { orig, mask: new Array<boolean>(orig.length).fill(false), now, notes: [] };

  const intention = stripIntentionMarkers(ctx);
  const hasDue = scan(ctx, /\bdue(?:\s+by)?\b/i, () => true) > 0;
  const kind: QuickKind = hasDue ? 'deadline' : intention ? 'intention' : 'event';
  const dueEndOfDay = hasDue && scan(ctx, /\b(?:by\s+)?(?:end of day|eod)\b/i, () => true) > 0;

  const repeat = parseRepeat(ctx);
  const count = parseCount(ctx);
  if (kind === 'deadline' && (repeat !== null || count !== null || /\bevery\b/i.test(viewOf(ctx)))) {
    fail('Deadlines cannot repeat');
  }
  if (count !== null && repeat === null) fail('Add a repeat before the count');
  const duration = parseDuration(ctx);
  const dates = parseDates(ctx);
  if (dates.until !== null && kind === 'deadline') fail('Deadlines cannot repeat');
  if (dates.until !== null && repeat === null) fail('Add a repeat before "until"');
  const times = parseTimes(ctx);
  const location = parseLocation(ctx);
  if (/\buntil\b/i.test(viewOf(ctx))) fail('Add a date after "until"');
  if (/\bevery\b/i.test(viewOf(ctx))) fail('Use a repeat like "every mon" or "every 2 weeks"');
  const title = cleanTitle(ctx);
  const notes = ctx.notes;

  const today = now.date;
  const explicit = dates.date !== null;
  let date = dates.date !== null ? dates.date : today;
  let endDate: string | null = dates.endDate;
  let allDay = true;
  let start: string | null = null;
  let end: string | null = null;
  let wrapDays = 0;

  if (kind === 'deadline') {
    if (times.end !== null || duration !== null) fail('A deadline takes one due time');
    if (endDate !== null) fail('A deadline takes one due date');
    if (times.start !== null) {
      if (dueEndOfDay) fail('A deadline takes one due time');
      allDay = false;
      start = hhmm(times.start);
    } else {
      allDay = true;
      notes.push('Due during the whole date');
    }
  } else {
    let startMin = times.start;
    if (startMin === null && dates.tonight) {
      startMin = 19 * 60;
      notes.push('Tonight starts at ' + clock(hhmm(startMin)));
    }
    if (startMin === null) {
      if (duration !== null) fail('Add a start time for the duration');
      allDay = true;
    } else {
      if (endDate !== null) fail('A timed event takes one date');
      allDay = false;
      start = hhmm(startMin);
      let endTotal: number;
      if (times.end !== null) {
        if (duration !== null) fail('Use an end time or a duration, not both');
        endTotal = times.end <= startMin ? times.end + 1440 : times.end;
      } else if (duration !== null) {
        endTotal = startMin + duration;
      } else {
        endTotal = startMin + defaultMinutes;
        notes.push('No end time; assumed ' + durationPhrase(defaultMinutes));
      }
      wrapDays = Math.floor(endTotal / 1440);
      end = hhmm(endTotal % 1440);
    }
  }

  let repeatOut: QuickRepeat | null = null;
  if (repeat !== null) {
    if (!explicit) {
      if (repeat.freq === 'weekly' && repeat.days !== null) {
        date = earliestOnOrAfter(today, repeat.days);
        notes.push('No start date; starts on the first matching day');
      } else {
        notes.push('No start date; starts today');
      }
    } else if (repeat.freq === 'weekly' && repeat.days !== null && repeat.days.indexOf(weekdayOf(date)) < 0) {
      notes.push('First repeat is the next matching day after the start date');
    }
    let weekdays: Weekday[] = [];
    if (repeat.freq === 'weekly') weekdays = repeat.days !== null ? repeat.days : [weekdayOf(date)];
    if (dates.until !== null && compareYmd(dates.until, date) < 0) fail('The until date is before the first occurrence');
    repeatOut = { freq: repeat.freq, interval: repeat.interval, weekdays, until: dates.until, count };
  } else if (!explicit) {
    if (kind === 'intention') notes.push('No preferred day; shown today as optional');
    else notes.push('No date; set for today');
  }

  if (wrapDays > 0) {
    endDate = addDays(date, wrapDays);
    notes.push('Ends after midnight on ' + shortDate(endDate));
  }

  return { kind, title, date, endDate, allDay, start, end, location, repeat: repeatOut, explicitDate: explicit, notes };
}

// Never throws for user input: parse problems come back as { error }.
export function parseQuickAdd(text: string, now: QuickNow, defaultMinutes: number = 60): QuickParse | { error: string } {
  try {
    return build(text, now, defaultMinutes);
  } catch (err) {
    if (err instanceof QuickFail) return { error: err.message };
    throw err;
  }
}

// ---- describe: the confirmation line shown before anything is saved ----

function cap(d: Weekday): string {
  return d.charAt(0).toUpperCase() + d.slice(1);
}

// "Tue, Oct 13"
function shortDate(ymd: string): string {
  const p = parseYmd(ymd);
  if (!p) return ymd;
  const weekday = DAY_NAMES[new Date(Date.UTC(p.y, p.m - 1, p.d)).getUTCDay()];
  return weekday + ', ' + MONTH_NAMES[p.m - 1].slice(0, 3) + ' ' + p.d;
}

// "Dec 5, 2026"
function monthDayYear(ymd: string): string {
  const p = parseYmd(ymd);
  if (!p) return ymd;
  return MONTH_NAMES[p.m - 1].slice(0, 3) + ' ' + p.d + ', ' + p.y;
}

// "14:00" -> "2:00 PM", "00:00" -> "12:00 AM"
function clock(hhmmText: string): string {
  const h = Number(hhmmText.slice(0, 2));
  const minutes = hhmmText.slice(3, 5);
  const h12 = h % 12 === 0 ? 12 : h % 12;
  return h12 + ':' + minutes + ' ' + (h < 12 ? 'AM' : 'PM');
}

function repeatText(r: QuickRepeat): string {
  let text: string;
  if (r.freq === 'daily') {
    text = r.interval === 1 ? 'daily' : 'every ' + r.interval + ' days';
  } else if (r.freq === 'monthly') {
    text = r.interval === 1 ? 'monthly' : 'every ' + r.interval + ' months';
  } else {
    text = (r.interval === 1 ? 'weekly' : 'every ' + r.interval + ' weeks') + ' on ' + r.weekdays.map(cap).join(', ');
  }
  if (r.until !== null) text += ' until ' + monthDayYear(r.until);
  if (r.count !== null) text += ', ' + r.count + ' times';
  return text;
}

// Examples: "Event · Tue, Oct 13 · 2:00 PM – 4:00 PM · weekly on Tue until Dec 5, 2026 · at Robarts"
//           "Deadline · Fri, Oct 9 · due 11:59 PM"   "Optional · no preferred time"
export function describe(p: QuickParse): string {
  const parts: string[] = [];
  parts.push(p.kind === 'event' ? 'Event' : p.kind === 'deadline' ? 'Deadline' : 'Optional');
  if (p.kind !== 'intention' || p.explicitDate) {
    parts.push(p.allDay && p.endDate !== null ? shortDate(p.date) + ' – ' + shortDate(p.endDate) : shortDate(p.date));
  }
  if (p.kind === 'deadline') {
    parts.push(p.start === null ? 'due by end of day' : 'due ' + clock(p.start));
  } else if (p.start === null) {
    parts.push(p.kind === 'intention' ? 'no preferred time' : 'all day');
  } else {
    const endText = p.end === null ? '' : clock(p.end);
    parts.push(clock(p.start) + ' – ' + endText + (p.endDate !== null ? ' (next day)' : ''));
  }
  if (p.repeat !== null) parts.push(repeatText(p.repeat));
  if (p.location !== null) parts.push('at ' + p.location);
  return parts.join(' · ');
}
