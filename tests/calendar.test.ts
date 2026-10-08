import test from 'node:test';
import assert from 'node:assert/strict';
import type { CalendarEvent } from '../src/lib/calendar-types.ts';
import {
  addDays, allDayFor, dayDeadlines, dayRange, eventDates, formatDate, formatMonth, formatTime,
  layoutDay, minutesLabel, monthGrid, snapMinutes, startOfWeek, weekDays, weekdayOf, wallToString, zonedParts,
} from '../src/lib/calendar.ts';

const TORONTO = 'America/Toronto';

function ev(partial: Partial<CalendarEvent> & Pick<CalendarEvent, 'id' | 'kind'>): CalendarEvent {
  return {
    title: partial.id,
    start: 0,
    end: 0,
    all_day: false,
    start_date: null,
    end_date: null,
    source_id: 'local',
    source_label: 'Synthetic local',
    source_kind: 'local',
    color: null,
    editable: true,
    series_id: null,
    occurrence_date: null,
    location: null,
    notes: null,
    occupancy: 'busy',
    tentative: false,
    state: 'upcoming',
    risk: null,
    ...partial,
  };
}

// Toronto is EDT (-04:00) on these October dates.
function timed(id: string, kind: CalendarEvent['kind'], start: string, end: string, extra: Partial<CalendarEvent> = {}): CalendarEvent {
  return ev({ id, kind, start: Date.parse(start), end: Date.parse(end), ...extra });
}

function allDay(id: string, kind: CalendarEvent['kind'], startDate: string, endDate: string | null, extra: Partial<CalendarEvent> = {}): CalendarEvent {
  return ev({ id, kind, all_day: true, start_date: startDate, end_date: endDate, start: Date.parse(`${startDate}T00:00:00-04:00`), end: Date.parse(`${startDate}T00:00:00-04:00`), ...extra });
}

test('zonedParts reads the wall clock on both sides of the spring-forward gap', () => {
  assert.deepEqual(zonedParts(Date.parse('2026-03-08T06:30:00Z'), TORONTO), { date: '2026-03-08', minutes: 90, weekday: 'sun' });
  assert.deepEqual(zonedParts(Date.parse('2026-03-08T07:30:00Z'), TORONTO), { date: '2026-03-08', minutes: 210, weekday: 'sun' });
});

test('zonedParts reads repeated fall-back wall times on the same civil date', () => {
  assert.equal(zonedParts(Date.parse('2026-11-01T05:30:00Z'), TORONTO).minutes, 90); // 01:30 EDT, first pass
  assert.equal(zonedParts(Date.parse('2026-11-01T06:30:00Z'), TORONTO).minutes, 90); // 01:30 EST, second pass
  assert.equal(zonedParts(Date.parse('2026-11-01T04:59:00Z'), TORONTO).minutes, 59);
  assert.equal(zonedParts(Date.parse('2026-11-01T06:30:00Z'), TORONTO).date, '2026-11-01');
});

test('zonedParts maps local midnight to minute 0 on the same civil date', () => {
  assert.deepEqual(zonedParts(Date.parse('2026-10-13T04:00:00Z'), TORONTO), { date: '2026-10-13', minutes: 0, weekday: 'tue' });
});

test('zonedParts rolls the civil date back when the zone is behind UTC', () => {
  assert.deepEqual(zonedParts(Date.parse('2026-10-13T03:30:00Z'), TORONTO), { date: '2026-10-12', minutes: 1410, weekday: 'mon' });
});

test('zonedParts supports a half-hour offset zone', () => {
  assert.deepEqual(zonedParts(Date.parse('2026-10-13T00:00:00Z'), 'Asia/Kolkata'), { date: '2026-10-13', minutes: 330, weekday: 'tue' });
});

test('addDays crosses month, year, and leap-day boundaries', () => {
  assert.equal(addDays('2026-01-31', 1), '2026-02-01');
  assert.equal(addDays('2026-12-31', 1), '2027-01-01');
  assert.equal(addDays('2026-03-01', -1), '2026-02-28');
  assert.equal(addDays('2028-02-28', 1), '2028-02-29');
  assert.equal(addDays('2026-10-13', 0), '2026-10-13');
});

test('weekdayOf names the civil weekday from the date alone', () => {
  assert.equal(weekdayOf('2026-10-13'), 'tue');
  assert.equal(weekdayOf('2026-10-12'), 'mon');
  assert.equal(weekdayOf('2026-10-11'), 'sun');
});

test('startOfWeek honours Monday and Sunday starts', () => {
  assert.equal(startOfWeek('2026-10-08', 'mon'), '2026-10-05');
  assert.equal(startOfWeek('2026-10-08', 'sun'), '2026-10-04');
  assert.equal(startOfWeek('2026-10-04', 'mon'), '2026-09-28');
  assert.equal(startOfWeek('2026-10-04', 'sun'), '2026-10-04');
});

test('weekDays returns seven consecutive dates from the week start', () => {
  const days = weekDays('2026-10-13', 'mon');
  assert.equal(days.length, 7);
  assert.equal(days[0], '2026-10-12');
  assert.equal(days[6], '2026-10-18');
  assert.deepEqual(days.map(weekdayOf), ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun']);
});

test('monthGrid covers February 2026 from a Monday start in 42 cells', () => {
  const grid = monthGrid('2026-02', 'mon');
  assert.equal(grid.length, 42);
  assert.equal(grid[0], '2026-01-26');
  assert.equal(grid[41], '2026-03-08');
  assert.ok(grid.includes('2026-02-01'));
  assert.ok(grid.includes('2026-02-28'));
});

test('monthGrid covers October 2026 from a Sunday start in 42 cells', () => {
  const grid = monthGrid('2026-10', 'sun');
  assert.equal(grid.length, 42);
  assert.equal(grid[0], '2026-09-27');
  assert.equal(grid[41], '2026-11-07');
  assert.equal(weekdayOf(grid[0]), 'sun');
});

test('monthGrid contains every day of the month exactly once', () => {
  for (const [month, days] of [['2026-10', 31], ['2026-02', 28]] as const) {
    const inMonth = monthGrid(month, 'mon').filter((date) => date.startsWith(`${month}-`));
    assert.equal(inMonth.length, days);
    assert.equal(new Set(inMonth).size, days);
  }
});

test('monthGrid rejects malformed month strings', () => {
  assert.throws(() => monthGrid('2026-13', 'mon'), RangeError);
  assert.throws(() => monthGrid('October 2026', 'mon'), RangeError);
});

test('eventDates keeps a timed event on its own civil date', () => {
  const event = timed('lecture', 'anchor', '2026-10-13T09:00:00-04:00', '2026-10-13T10:30:00-04:00');
  assert.deepEqual(eventDates(event, TORONTO), ['2026-10-13']);
});

test('eventDates does not touch the next date when a timed event ends at midnight', () => {
  const event = timed('late', 'anchor', '2026-10-13T22:00:00-04:00', '2026-10-14T00:00:00-04:00');
  assert.deepEqual(eventDates(event, TORONTO), ['2026-10-13']);
});

test('eventDates lists every date a timed event crosses', () => {
  const overnight = timed('night', 'anchor', '2026-10-13T22:00:00-04:00', '2026-10-14T01:00:00-04:00');
  assert.deepEqual(eventDates(overnight, TORONTO), ['2026-10-13', '2026-10-14']);
  const threeDays = timed('trip', 'anchor', '2026-10-12T10:00:00-04:00', '2026-10-14T10:00:00-04:00');
  assert.deepEqual(eventDates(threeDays, TORONTO), ['2026-10-12', '2026-10-13', '2026-10-14']);
});

test('eventDates covers all-day multi-day events with an exclusive end', () => {
  const event = allDay('camp', 'anchor', '2026-10-12', '2026-10-15');
  assert.deepEqual(eventDates(event, TORONTO), ['2026-10-12', '2026-10-13', '2026-10-14']);
});

test('eventDates gives a deadline only its own date', () => {
  assert.deepEqual(eventDates(timed('ps3', 'deadline', '2026-10-13T23:59:00-04:00', '2026-10-13T23:59:00-04:00'), TORONTO), ['2026-10-13']);
  assert.deepEqual(eventDates(allDay('ps4', 'deadline', '2026-10-20', '2026-10-21'), TORONTO), ['2026-10-20']);
});

test('eventDates caps very long all-day spans at 62 dates', () => {
  const dates = eventDates(allDay('year', 'anchor', '2026-01-01', '2027-01-01'), TORONTO);
  assert.equal(dates.length, 62);
  assert.equal(dates[0], '2026-01-01');
  assert.equal(dates[61], '2026-03-03');
});

test('dayRange returns null when a span only touches the day boundary', () => {
  const start = Date.parse('2026-10-12T22:00:00-04:00');
  const end = Date.parse('2026-10-13T00:00:00-04:00');
  assert.equal(dayRange(start, end, '2026-10-13', TORONTO), null);
  assert.deepEqual(dayRange(start, end, '2026-10-12', TORONTO), { top: 1320, bottom: 1440, continuesBefore: false, continuesAfter: false });
});

test('layoutDay places three mutually overlapping events in three columns', () => {
  const events = [
    timed('a', 'anchor', '2026-10-13T09:00:00-04:00', '2026-10-13T10:00:00-04:00'),
    timed('b', 'anchor', '2026-10-13T09:15:00-04:00', '2026-10-13T10:15:00-04:00'),
    timed('c', 'anchor', '2026-10-13T09:30:00-04:00', '2026-10-13T10:30:00-04:00'),
  ];
  const segments = layoutDay(events, '2026-10-13', TORONTO);
  assert.equal(segments.length, 3);
  assert.ok(segments.every((segment) => segment.columns === 3));
  assert.deepEqual(new Set(segments.map((segment) => segment.column)), new Set([0, 1, 2]));
});

test('layoutDay reuses a column when only a chain of events overlaps', () => {
  const events = [
    timed('a', 'anchor', '2026-10-13T09:00:00-04:00', '2026-10-13T10:00:00-04:00'),
    timed('b', 'anchor', '2026-10-13T09:30:00-04:00', '2026-10-13T11:00:00-04:00'),
    timed('c', 'anchor', '2026-10-13T10:30:00-04:00', '2026-10-13T11:30:00-04:00'),
  ];
  const byId = Object.fromEntries(layoutDay(events, '2026-10-13', TORONTO).map((segment) => [segment.event.id, segment]));
  assert.equal(byId.a.columns, 2);
  assert.equal(byId.b.columns, 2);
  assert.equal(byId.c.columns, 2);
  assert.equal(byId.a.column, byId.c.column);
  assert.notEqual(byId.a.column, byId.b.column);
});

test('layoutDay gives an isolated event one full-width column', () => {
  const [segment] = layoutDay([timed('solo', 'anchor', '2026-10-13T09:00:00-04:00', '2026-10-13T10:00:00-04:00')], '2026-10-13', TORONTO);
  assert.equal(segment.column, 0);
  assert.equal(segment.columns, 1);
});

test('layoutDay treats back-to-back events as non-overlapping', () => {
  const events = [
    timed('first', 'anchor', '2026-10-13T09:00:00-04:00', '2026-10-13T10:00:00-04:00'),
    timed('second', 'anchor', '2026-10-13T10:00:00-04:00', '2026-10-13T11:00:00-04:00'),
  ];
  const segments = layoutDay(events, '2026-10-13', TORONTO);
  assert.ok(segments.every((segment) => segment.columns === 1 && segment.column === 0));
});

test('layoutDay enforces a 20-minute visual minimum capped at the end of the day', () => {
  const [short] = layoutDay([timed('blip', 'anchor', '2026-10-13T09:00:00-04:00', '2026-10-13T09:05:00-04:00')], '2026-10-13', TORONTO);
  assert.equal(short.top, 540);
  assert.equal(short.bottom - short.top, 20);
  const [late] = layoutDay([timed('late', 'anchor', '2026-10-13T23:55:00-04:00', '2026-10-13T23:58:00-04:00')], '2026-10-13', TORONTO);
  assert.equal(late.top, 1435);
  assert.equal(late.bottom, 1440);
});

test('layoutDay clips a multi-day event to each civil date it touches', () => {
  const events = [timed('trip', 'anchor', '2026-10-12T22:00:00-04:00', '2026-10-14T08:00:00-04:00')];
  const on = (date: string) => layoutDay(events, date, TORONTO)[0];
  assert.deepEqual(
    { top: on('2026-10-12').top, bottom: on('2026-10-12').bottom, before: on('2026-10-12').continuesBefore, after: on('2026-10-12').continuesAfter },
    { top: 1320, bottom: 1440, before: false, after: true },
  );
  assert.deepEqual(
    { top: on('2026-10-13').top, bottom: on('2026-10-13').bottom, before: on('2026-10-13').continuesBefore, after: on('2026-10-13').continuesAfter },
    { top: 0, bottom: 1440, before: true, after: true },
  );
  assert.deepEqual(
    { top: on('2026-10-14').top, bottom: on('2026-10-14').bottom, before: on('2026-10-14').continuesBefore, after: on('2026-10-14').continuesAfter },
    { top: 0, bottom: 480, before: true, after: false },
  );
  assert.equal(layoutDay(events, '2026-10-15', TORONTO).length, 0);
});

test('layoutDay omits an event on the civil date where its midnight end begins', () => {
  const events = [timed('night', 'anchor', '2026-10-13T22:00:00-04:00', '2026-10-14T00:00:00-04:00')];
  assert.equal(layoutDay(events, '2026-10-14', TORONTO).length, 0);
  assert.equal(layoutDay(events, '2026-10-13', TORONTO)[0].bottom, 1440);
});

test('layoutDay excludes all-day rows and deadlines', () => {
  const events = [
    allDay('holiday', 'anchor', '2026-10-13', '2026-10-14'),
    timed('ps3', 'deadline', '2026-10-13T23:59:00-04:00', '2026-10-13T23:59:00-04:00'),
  ];
  assert.deepEqual(layoutDay(events, '2026-10-13', TORONTO), []);
});

test('layoutDay places the longer block first when starts tie', () => {
  const events = [
    timed('short', 'anchor', '2026-10-13T09:00:00-04:00', '2026-10-13T09:30:00-04:00'),
    timed('long', 'anchor', '2026-10-13T09:00:00-04:00', '2026-10-13T12:00:00-04:00'),
  ];
  const byId = Object.fromEntries(layoutDay(events, '2026-10-13', TORONTO).map((segment) => [segment.event.id, segment]));
  assert.equal(byId.long.column, 0);
  assert.equal(byId.short.column, 1);
  assert.equal(byId.long.columns, 2);
});

test('dayDeadlines returns the timed deadlines of one date sorted by minute', () => {
  const events = [
    timed('late', 'deadline', '2026-10-13T23:59:00-04:00', '2026-10-13T23:59:00-04:00'),
    timed('early', 'deadline', '2026-10-13T09:00:00-04:00', '2026-10-13T09:00:00-04:00'),
    timed('other-day', 'deadline', '2026-10-14T10:00:00-04:00', '2026-10-14T10:00:00-04:00'),
    allDay('date-only', 'deadline', '2026-10-13', null),
    timed('not-deadline', 'anchor', '2026-10-13T12:00:00-04:00', '2026-10-13T13:00:00-04:00'),
  ];
  const result = dayDeadlines(events, '2026-10-13', TORONTO);
  assert.deepEqual(result.map((item) => item.event.id), ['early', 'late']);
  assert.deepEqual(result.map((item) => item.minutes), [540, 1439]);
});

test('allDayFor orders rows by kind and then title', () => {
  const events = [
    allDay('gym', 'routine', '2026-10-13', null, { title: 'Gym' }),
    allDay('holiday', 'anchor', '2026-10-13', null, { title: 'Holiday' }),
    allDay('read', 'intention', '2026-10-13', null, { title: 'Read' }),
    allDay('due', 'deadline', '2026-10-13', null, { title: 'Due' }),
    allDay('break', 'anchor', '2026-10-13', null, { title: 'Break' }),
  ];
  assert.deepEqual(allDayFor(events, '2026-10-13', TORONTO).map((event) => event.title), ['Due', 'Break', 'Holiday', 'Read', 'Gym']);
});

test('allDayFor includes multi-day rows only on the dates they touch', () => {
  const events = [
    allDay('camp', 'anchor', '2026-10-12', '2026-10-15'),
    timed('class', 'anchor', '2026-10-13T09:00:00-04:00', '2026-10-13T10:00:00-04:00'),
  ];
  assert.deepEqual(allDayFor(events, '2026-10-11', TORONTO).map((event) => event.id), []);
  assert.deepEqual(allDayFor(events, '2026-10-12', TORONTO).map((event) => event.id), ['camp']);
  assert.deepEqual(allDayFor(events, '2026-10-14', TORONTO).map((event) => event.id), ['camp']);
  assert.deepEqual(allDayFor(events, '2026-10-15', TORONTO).map((event) => event.id), []);
});

test('formatTime renders zone-local twelve-hour labels', () => {
  assert.equal(formatTime(Date.parse('2026-10-13T13:00:00Z'), TORONTO), '9:00 AM');
  assert.equal(formatTime(Date.parse('2026-10-13T16:00:00Z'), TORONTO), '12:00 PM');
  assert.equal(formatTime(Date.parse('2026-10-13T04:00:00Z'), TORONTO), '12:00 AM');
  assert.equal(formatTime(Date.parse('2026-10-13T03:59:00Z'), TORONTO), '11:59 PM');
});

test('minutesLabel labels noon and midnight and wraps 1440 to midnight', () => {
  assert.equal(minutesLabel(0), '12:00 AM');
  assert.equal(minutesLabel(555), '9:15 AM');
  assert.equal(minutesLabel(720), '12:00 PM');
  assert.equal(minutesLabel(1439), '11:59 PM');
  assert.equal(minutesLabel(1440), '12:00 AM');
});

test('formatDate and formatMonth name days and months from the civil date alone', () => {
  assert.equal(formatDate('2026-10-13', 'short'), 'Tue 13');
  assert.equal(formatDate('2026-10-13', 'long'), 'Tuesday, October 13');
  assert.equal(formatDate('2026-03-01', 'long'), 'Sunday, March 1');
  assert.equal(formatMonth('2026-10'), 'October 2026');
});

test('snapMinutes floors to the requested step and rejects a zero step', () => {
  assert.equal(snapMinutes(37), 30);
  assert.equal(snapMinutes(37, 5), 35);
  assert.equal(snapMinutes(14), 0);
  assert.equal(snapMinutes(1439), 1425);
  assert.equal(snapMinutes(45, 30), 30);
  assert.throws(() => snapMinutes(10, 0), RangeError);
});

test('wallToString rolls minute 1440 into the next civil date', () => {
  assert.equal(wallToString('2026-10-13', 1440), '2026-10-14T00:00');
  assert.equal(wallToString('2026-12-31', 1440), '2027-01-01T00:00');
  assert.equal(wallToString('2026-10-13', 555), '2026-10-13T09:15');
  assert.equal(wallToString('2026-10-13', 0), '2026-10-13T00:00');
});
