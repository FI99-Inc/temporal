import type { CalendarEvent, Weekday } from './calendar-types.ts';

// Pure date and layout helpers for the Calendar utility. Civil dates are
// 'YYYY-MM-DD' strings; wall times are minutes since local midnight in the
// display zone. Nothing here reads the system zone or the current time.

export interface ZonedParts { date: string; minutes: number; weekday: Weekday }

export interface Segment {
  event: CalendarEvent;
  date: string;
  top: number;
  bottom: number;
  column: number;
  columns: number;
  continuesBefore: boolean;
  continuesAfter: boolean;
}

export interface DayRange { top: number; bottom: number; continuesBefore: boolean; continuesAfter: boolean }

const DAY_MS = 86_400_000;
const MINUTES_PER_DAY = 1440;
const MIN_VISUAL_MINUTES = 20;
const MAX_DATES = 62;
const WEEKDAYS: Weekday[] = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'];
const WEEKDAY_SHORT = ['Sun', 'Mon', 'Tue', 'Wed', 'Thu', 'Fri', 'Sat'];
const WEEKDAY_LONG = ['Sunday', 'Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday'];
const MONTH_LONG = ['January', 'February', 'March', 'April', 'May', 'June', 'July', 'August', 'September', 'October', 'November', 'December'];
const KIND_ORDER: Record<CalendarEvent['kind'], number> = { deadline: 0, anchor: 1, intention: 2, routine: 3 };
const formatters = new Map<string, Intl.DateTimeFormat>();

function pad2(value: number): string {
  return String(value).padStart(2, '0');
}

function parseDate(date: string): [number, number, number] {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(date);
  if (!match) throw new RangeError(`Invalid civil date: ${date}`);
  return [Number(match[1]), Number(match[2]), Number(match[3])];
}

function utcMs(date: string): number {
  const [year, month, day] = parseDate(date);
  return Date.UTC(year, month - 1, day);
}

function dayNumber(date: string): number {
  return Math.round(utcMs(date) / DAY_MS);
}

function isoFromUtc(ms: number): string {
  const d = new Date(ms);
  return `${String(d.getUTCFullYear()).padStart(4, '0')}-${pad2(d.getUTCMonth() + 1)}-${pad2(d.getUTCDate())}`;
}

function compareText(a: string, b: string): number {
  return a < b ? -1 : a > b ? 1 : 0;
}

function formatterFor(zone: string): Intl.DateTimeFormat {
  let formatter = formatters.get(zone);
  if (!formatter) {
    formatter = new Intl.DateTimeFormat('en-US', {
      timeZone: zone,
      hourCycle: 'h23',
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit',
    });
    formatters.set(zone, formatter);
  }
  return formatter;
}

export function zonedParts(ms: number, zone: string): ZonedParts {
  const parts = formatterFor(zone).formatToParts(ms);
  const read = (type: string) => parts.find((part) => part.type === type)?.value ?? '';
  let hour = Number(read('hour'));
  if (hour === 24) hour = 0; // some engines report midnight as 24 under h23
  const minute = Number(read('minute'));
  const date = `${read('year')}-${read('month')}-${read('day')}`;
  return { date, minutes: hour * 60 + minute, weekday: weekdayOf(date) };
}

export function addDays(date: string, n: number): string {
  return isoFromUtc(utcMs(date) + n * DAY_MS);
}

export function weekdayOf(date: string): Weekday {
  // getUTCDay: 0 = Sunday. Map to the Monday-based weekday order.
  return WEEKDAYS[(new Date(utcMs(date)).getUTCDay() + 6) % 7];
}

export function startOfWeek(date: string, weekStartsOn: 'mon' | 'sun'): string {
  const dow = new Date(utcMs(date)).getUTCDay(); // 0 = Sunday
  const back = weekStartsOn === 'sun' ? dow : (dow + 6) % 7;
  return addDays(date, -back);
}

export function weekDays(date: string, weekStartsOn: 'mon' | 'sun'): string[] {
  const start = startOfWeek(date, weekStartsOn);
  return Array.from({ length: 7 }, (_, index) => addDays(start, index));
}

export function monthGrid(month: string, weekStartsOn: 'mon' | 'sun'): string[] {
  if (!/^\d{4}-(0[1-9]|1[0-2])$/.test(month)) throw new RangeError(`Invalid month: ${month}`);
  const start = startOfWeek(`${month}-01`, weekStartsOn);
  return Array.from({ length: 42 }, (_, index) => addDays(start, index));
}

// Civil dates from `first` up to, not including, `endExclusive`, capped.
function spanDates(first: string, endExclusive: string): string[] {
  const out: string[] = [];
  for (let date = first; date < endExclusive && out.length < MAX_DATES; date = addDays(date, 1)) out.push(date);
  return out;
}

// All-day rows: the first civil date and the exclusive end. A deadline is a
// point, so it touches only its own date even when a date-precision end is set.
function allDayRange(event: CalendarEvent, zone: string): { first: string; end: string } {
  const first = event.start_date ?? zonedParts(event.start, zone).date;
  if (event.kind === 'deadline') return { first, end: addDays(first, 1) };
  const end = event.end_date !== null && event.end_date > first ? event.end_date : addDays(first, 1);
  return { first, end };
}

export function eventDates(event: CalendarEvent, zone: string): string[] {
  if (event.kind === 'deadline') {
    if (event.all_day && event.start_date !== null) return [event.start_date];
    return [zonedParts(event.start, zone).date];
  }
  if (event.all_day) {
    const { first, end } = allDayRange(event, zone);
    return spanDates(first, end);
  }
  const start = zonedParts(event.start, zone);
  if (event.end <= event.start) return [start.date];
  const end = zonedParts(event.end, zone);
  // An end exactly at local midnight does not touch the following civil date.
  const endExclusive = end.minutes === 0 ? end.date : addDays(end.date, 1);
  return spanDates(start.date, endExclusive > start.date ? endExclusive : addDays(start.date, 1));
}

// Clip a timed span to one civil date using wall-clock minutes. Absolute
// minutes (day number * 1440 + wall minutes) make an end at local midnight
// fall exactly on the boundary, so it never touches the next date.
export function dayRange(start: number, end: number, date: string, zone: string): DayRange | null {
  const from = zonedParts(start, zone);
  const startAbs = dayNumber(from.date) * MINUTES_PER_DAY + from.minutes;
  let endAbs = startAbs;
  if (end > start) {
    const to = zonedParts(end, zone);
    endAbs = Math.max(startAbs, dayNumber(to.date) * MINUTES_PER_DAY + to.minutes);
  }
  const dayStart = dayNumber(date) * MINUTES_PER_DAY;
  const dayEnd = dayStart + MINUTES_PER_DAY;
  const zeroLength = endAbs === startAbs;
  const outside = zeroLength ? startAbs < dayStart || startAbs >= dayEnd : endAbs <= dayStart || startAbs >= dayEnd;
  if (outside) return null;
  return {
    top: Math.max(startAbs, dayStart) - dayStart,
    bottom: Math.min(endAbs, dayEnd) - dayStart,
    continuesBefore: startAbs < dayStart,
    continuesAfter: endAbs > dayEnd,
  };
}

type Block = Omit<Segment, 'column' | 'columns'>;

function columnise(cluster: Block[]): Segment[] {
  // Greedy first-fit in start order. Each column records the bottom of the
  // last block placed in it, so a block reuses the first column that is free.
  const ends: number[] = [];
  const columnOf: number[] = [];
  for (const block of cluster) {
    let column = ends.findIndex((end) => end <= block.top);
    if (column === -1) {
      column = ends.length;
      ends.push(block.bottom);
    } else {
      ends[column] = block.bottom;
    }
    columnOf.push(column);
  }
  return cluster.map((block, index) => ({ ...block, column: columnOf[index], columns: ends.length }));
}

export function layoutDay(events: CalendarEvent[], date: string, zone: string): Segment[] {
  const blocks: Block[] = [];
  for (const event of events) {
    if (event.all_day || event.kind === 'deadline') continue;
    const range = dayRange(event.start, event.end, date, zone);
    if (!range) continue;
    blocks.push({
      event,
      date,
      top: range.top,
      bottom: Math.min(MINUTES_PER_DAY, Math.max(range.bottom, range.top + MIN_VISUAL_MINUTES)),
      continuesBefore: range.continuesBefore,
      continuesAfter: range.continuesAfter,
    });
  }
  blocks.sort((a, b) =>
    a.top - b.top
    || (b.bottom - b.top) - (a.bottom - a.top)
    || compareText(a.event.id, b.event.id));

  // Transitive overlaps form one cluster. Blocks are sorted by top, so a block
  // starting before the cluster's latest bottom overlaps something in it.
  const out: Segment[] = [];
  let cluster: Block[] = [];
  let clusterEnd = Number.NEGATIVE_INFINITY;
  for (const block of blocks) {
    if (cluster.length > 0 && block.top >= clusterEnd) {
      out.push(...columnise(cluster));
      cluster = [];
    }
    cluster.push(block);
    clusterEnd = Math.max(clusterEnd, block.bottom);
  }
  if (cluster.length > 0) out.push(...columnise(cluster));
  return out;
}

export function dayDeadlines(events: CalendarEvent[], date: string, zone: string): { event: CalendarEvent; minutes: number }[] {
  const out: { event: CalendarEvent; minutes: number }[] = [];
  for (const event of events) {
    if (event.kind !== 'deadline' || event.all_day) continue;
    const parts = zonedParts(event.start, zone);
    if (parts.date !== date) continue;
    out.push({ event, minutes: parts.minutes });
  }
  return out.sort((a, b) => a.minutes - b.minutes || compareText(a.event.title, b.event.title) || compareText(a.event.id, b.event.id));
}

export function allDayFor(events: CalendarEvent[], date: string, zone: string): CalendarEvent[] {
  return events
    .filter((event) => {
      if (!event.all_day) return false;
      const { first, end } = allDayRange(event, zone);
      return first <= date && date < end;
    })
    .sort((a, b) => KIND_ORDER[a.kind] - KIND_ORDER[b.kind] || compareText(a.title, b.title) || compareText(a.id, b.id));
}

// '9:00 AM' from minutes since midnight. 1440 wraps to 12:00 AM.
export function minutesLabel(minutes: number): string {
  const hour = Math.floor(minutes / 60) % 24;
  const minute = minutes % 60;
  const hour12 = hour % 12 === 0 ? 12 : hour % 12;
  return `${hour12}:${pad2(minute)} ${hour < 12 ? 'AM' : 'PM'}`;
}

export function formatTime(ms: number, zone: string): string {
  return minutesLabel(zonedParts(ms, zone).minutes);
}

export function formatDate(date: string, style: 'short' | 'long'): string {
  const [, month, day] = parseDate(date);
  const weekday = new Date(utcMs(date)).getUTCDay();
  return style === 'short'
    ? `${WEEKDAY_SHORT[weekday]} ${day}`
    : `${WEEKDAY_LONG[weekday]}, ${MONTH_LONG[month - 1]} ${day}`;
}

export function formatMonth(month: string): string {
  if (!/^\d{4}-(0[1-9]|1[0-2])$/.test(month)) throw new RangeError(`Invalid month: ${month}`);
  const [year, monthNumber] = month.split('-').map(Number);
  return `${MONTH_LONG[monthNumber - 1]} ${year}`;
}

export function snapMinutes(minutes: number, step = 15): number {
  if (!(step > 0)) throw new RangeError('Snap step must be positive');
  return Math.floor(minutes / step) * step;
}

// 'YYYY-MM-DDTHH:MM'. Minutes at or past 1440 roll into the following date.
export function wallToString(date: string, minutes: number): string {
  const dayOffset = Math.floor(minutes / MINUTES_PER_DAY);
  const within = minutes - dayOffset * MINUTES_PER_DAY;
  return `${addDays(date, dayOffset)}T${pad2(Math.floor(within / 60))}:${pad2(within % 60)}`;
}
