import { test } from 'node:test';
import assert from 'node:assert/strict';
import { parseQuickAdd, describe } from '../src/lib/quickadd.ts';
import type { QuickParse } from '../src/lib/quickadd.ts';

const NOW = { date: '2026-10-08', time: '10:00' }; // a Thursday
const LATER = { date: '2026-12-30', time: '09:00' }; // for year rollover

function ok(text: string, now = NOW, defaultMinutes?: number): QuickParse {
  const r = parseQuickAdd(text, now, defaultMinutes);
  if ('error' in r) assert.fail(`expected a parse for ${JSON.stringify(text)}, got error: ${r.error}`);
  return r;
}

function fails(text: string, now = NOW): string {
  const r = parseQuickAdd(text, now);
  if (!('error' in r)) assert.fail(`expected an error for ${JSON.stringify(text)}`);
  return r.error;
}

test('empty and whitespace-only input are refused', () => {
  assert.equal(fails(''), 'Type something to add');
  assert.equal(fails('   '), 'Type something to add');
});

test('a title that is empty once tokens are consumed is refused', () => {
  assert.equal(fails('tomorrow 3pm'), 'Add a title');
  assert.equal(fails('oct 12'), 'Add a title');
});

test('a lone x with only date tokens is refused', () => {
  assert.equal(fails('x oct 12'), 'Add a title');
  assert.equal(fails('x tomorrow'), 'Add a title');
});

test('Lab Tue 2-4pm every week until Dec 5 is a weekly event', () => {
  const p = ok('Lab Tue 2-4pm every week until Dec 5');
  assert.equal(p.kind, 'event');
  assert.equal(p.title, 'Lab');
  assert.equal(p.date, '2026-10-13');
  assert.equal(p.allDay, false);
  assert.equal(p.start, '14:00');
  assert.equal(p.end, '16:00');
  assert.equal(p.explicitDate, true);
  assert.deepEqual(p.repeat, { freq: 'weekly', interval: 1, weekdays: ['tue'], until: '2026-12-05', count: null });
  assert.equal(describe(p), 'Event · Tue, Oct 13 · 2:00 PM – 4:00 PM · weekly on Tue until Dec 5, 2026');
});

test('a deadline with a weekday and a time keeps the due time', () => {
  const p = ok('MAT223 PS3 due Fri 11:59pm');
  assert.equal(p.kind, 'deadline');
  assert.equal(p.title, 'MAT223 PS3');
  assert.equal(p.date, '2026-10-09');
  assert.equal(p.allDay, false);
  assert.equal(p.start, '23:59');
  assert.equal(p.end, null);
  assert.equal(describe(p), 'Deadline · Fri, Oct 9 · due 11:59 PM');
});

test('a deadline without a time is due the whole date', () => {
  const p = ok('essay due oct 20');
  assert.equal(p.kind, 'deadline');
  assert.equal(p.date, '2026-10-20');
  assert.equal(p.allDay, true);
  assert.equal(p.start, null);
  assert.ok(p.notes.includes('Due during the whole date'));
  assert.equal(describe(p), 'Deadline · Tue, Oct 20 · due by end of day');
});

test('by end of day is read as the whole date, with no stray words left in the title', () => {
  const p = ok('essay due oct 20 by end of day');
  assert.equal(p.title, 'essay');
  assert.equal(p.allDay, true);
  assert.equal(describe(p), 'Deadline · Tue, Oct 20 · due by end of day');
});

test('deadlines cannot repeat and take one due time', () => {
  assert.equal(fails('essay due fri every week'), 'Deadlines cannot repeat');
  assert.equal(fails('Lab due 3-4pm'), 'A deadline takes one due time');
});

test('a location after at ends at the next date or time token', () => {
  const p = ok('Lunch with Sam at Robarts tomorrow 12:30');
  assert.equal(p.title, 'Lunch with Sam');
  assert.equal(p.location, 'Robarts');
  assert.equal(p.date, '2026-10-09');
  assert.equal(p.start, '12:30');
  assert.equal(p.end, '13:30');
  assert.ok(p.notes.includes('No end time; assumed 1 hour'));
  assert.equal(describe(p), 'Event · Fri, Oct 9 · 12:30 PM – 1:30 PM · at Robarts');
});

test('a location runs to the end of the text when no token follows it', () => {
  const p = ok('Exam oct 20 9am-12pm @ EX100');
  assert.equal(p.title, 'Exam');
  assert.equal(p.location, 'EX100');
  assert.equal(p.date, '2026-10-20');
  assert.equal(p.start, '09:00');
  assert.equal(p.end, '12:00');
});

test('at or @ before a time is consumed and is not a location', () => {
  for (const text of ['Call mom at 9pm', 'Call mom @ 9pm']) {
    const p = ok(text);
    assert.equal(p.location, null, text);
    assert.equal(p.start, '21:00', text);
    assert.equal(p.title, 'Call mom', text);
  }
});

test('a weekly on several days starts on the first matching day', () => {
  const p = ok('Gym every mon wed fri 7am for 1h');
  assert.equal(p.title, 'Gym');
  assert.equal(p.date, '2026-10-09');
  assert.equal(p.explicitDate, false);
  assert.equal(p.start, '07:00');
  assert.equal(p.end, '08:00');
  assert.deepEqual(p.repeat, { freq: 'weekly', interval: 1, weekdays: ['mon', 'wed', 'fri'], until: null, count: null });
  assert.ok(p.notes.includes('No start date; starts on the first matching day'));
  assert.equal(describe(p), 'Event · Fri, Oct 9 · 7:00 AM – 8:00 AM · weekly on Mon, Wed, Fri');
});

test('a day range is an inclusive all-day span', () => {
  const p = ok('Reading week oct 12-16');
  assert.equal(p.title, 'Reading week');
  assert.equal(p.date, '2026-10-12');
  assert.equal(p.endDate, '2026-10-16');
  assert.equal(p.allDay, true);
  assert.equal(p.start, null);
  assert.equal(describe(p), 'Event · Mon, Oct 12 – Fri, Oct 16 · all day');
});

test('a maybe prefix makes a soft intention with a date', () => {
  const p = ok('maybe call grandma sunday');
  assert.equal(p.kind, 'intention');
  assert.equal(p.title, 'call grandma');
  assert.equal(p.date, '2026-10-11');
  assert.equal(p.explicitDate, true);
});

test('a trailing question mark makes an intention and is removed from the title', () => {
  const p = ok('Maybe read book?');
  assert.equal(p.kind, 'intention');
  assert.equal(p.title, 'read book');
  assert.equal(ok('Read book?').title, 'Read book');
});

test('try to and want to also make intentions', () => {
  assert.equal(ok('try to read').kind, 'intention');
  assert.equal(ok('try to read').title, 'read');
  const p = ok('want to swim tomorrow 7pm for 45m');
  assert.equal(p.kind, 'intention');
  assert.equal(p.title, 'swim');
  assert.equal(p.date, '2026-10-09');
  assert.equal(p.start, '19:00');
  assert.equal(p.end, '19:45');
});

test('an intention without a date is shown today as optional', () => {
  const p = ok('maybe read book');
  assert.equal(p.date, '2026-10-08');
  assert.equal(p.explicitDate, false);
  assert.equal(p.allDay, true);
  assert.ok(p.notes.includes('No preferred day; shown today as optional'));
  assert.equal(describe(p), 'Optional · no preferred time');
});

test('standup on weekdays with a 15 minute duration', () => {
  const p = ok('Standup weekdays 9:15 for 15m');
  assert.equal(p.title, 'Standup');
  assert.equal(p.date, '2026-10-08');
  assert.equal(p.explicitDate, false);
  assert.equal(p.start, '09:15');
  assert.equal(p.end, '09:30');
  assert.deepEqual(p.repeat?.weekdays, ['mon', 'tue', 'wed', 'thu', 'fri']);
});

test('a time that wraps past midnight ends the next day and says so', () => {
  const p = ok('Party fri 10pm-1am');
  assert.equal(p.date, '2026-10-09');
  assert.equal(p.endDate, '2026-10-10');
  assert.equal(p.start, '22:00');
  assert.equal(p.end, '01:00');
  assert.ok(p.notes.includes('Ends after midnight on Sat, Oct 10'));
  assert.equal(describe(p), 'Event · Fri, Oct 9 · 10:00 PM – 1:00 AM (next day)');
});

test('next weekday is the occurrence strictly after today, plus 7 days', () => {
  assert.equal(ok('Dentist next tue 3pm').date, '2026-10-20');
  assert.equal(ok('Thing next thu').date, '2026-10-22');
  assert.equal(ok('Thing next fri').date, '2026-10-16');
  assert.equal(ok('Thing next mon').date, '2026-10-19');
});

test('a plain weekday is the next one on or after today, and today counts', () => {
  assert.equal(ok('Thing fri').date, '2026-10-09');
  assert.equal(ok('Thing thu').date, '2026-10-08');
  assert.equal(ok('Thing thu').explicitDate, true);
});

test('a numeric date reads as month/day and says so', () => {
  const p = ok('Seminar 10/14 3pm');
  assert.equal(p.title, 'Seminar');
  assert.equal(p.date, '2026-10-14');
  assert.equal(p.start, '15:00');
  assert.ok(p.notes.includes('Read 10/14 as month/day'));
});

test('a numeric date with a year and an ISO date are read exactly', () => {
  assert.equal(ok('Review 10/12/2026').date, '2026-10-12');
  assert.equal(ok('Dentist 2026-10-12 2pm').date, '2026-10-12');
  assert.equal(ok('Dentist 2026-10-12 2pm').start, '14:00');
});

test('an ISO date range is an inclusive all-day span', () => {
  const p = ok('Trip 2026-11-20 to 2026-11-23');
  assert.equal(p.title, 'Trip');
  assert.equal(p.date, '2026-11-20');
  assert.equal(p.endDate, '2026-11-23');
  assert.equal(p.allDay, true);
});

test('month-day forms with ordinals, day-first, and full month names agree', () => {
  for (const text of ['Talk Oct 12th', 'Talk 12 oct', 'Talk october 12']) {
    const p = ok(text);
    assert.equal(p.title, 'Talk', text);
    assert.equal(p.date, '2026-10-12', text);
  }
});

test('an explicit year is used and not noted as an assumption', () => {
  const p = ok('essay oct 12 2027');
  assert.equal(p.date, '2027-10-12');
  assert.equal(p.title, 'essay');
  assert.ok(!p.notes.some((n) => n.startsWith('No year given')));
});

test('a month-day that has passed rolls to next year, and that is noted', () => {
  assert.equal(ok('jan 3 flight', LATER).date, '2027-01-03');
  assert.equal(ok('jan 3 flight', LATER).title, 'flight');
  assert.ok(ok('jan 3 flight', LATER).notes.includes('No year given; read as 2027'));
  assert.equal(ok('jan 3 flight').date, '2027-01-03');
});

test('impossible dates are refused', () => {
  assert.equal(fails('Feb 30 dinner'), 'That date does not exist');
  assert.equal(fails('31 apr dinner'), 'That date does not exist');
});

test('relative day phrases resolve from today', () => {
  assert.equal(ok('Thing in 3 days').date, '2026-10-11');
  assert.equal(ok('Thing in 2 weeks').date, '2026-10-22');
  for (const word of ['tomorrow', 'tmr', 'tmrw']) {
    assert.equal(ok(`Thing ${word}`).date, '2026-10-09', word);
  }
  const today = ok('Thing today');
  assert.equal(today.date, '2026-10-08');
  assert.equal(today.explicitDate, true);
});

test('time spellings all read to 24-hour clock values', () => {
  const cases: Array<[string, string]> = [
    ['3 pm', '15:00'],
    ['3:30pm', '15:30'],
    ['3.30pm', '15:30'],
    ['15:00', '15:00'],
    ['9a', '09:00'],
    ['9p', '21:00'],
    ['noon', '12:00'],
    ['midnight', '00:00'],
  ];
  for (const [spelling, expected] of cases) {
    assert.equal(ok(`Thing ${spelling}`).start, expected, spelling);
  }
});

test('time ranges, including am/pm inference from the other side', () => {
  const cases: Array<[string, string, string]> = [
    ['3-4pm', '15:00', '16:00'],
    ['11-1pm', '11:00', '13:00'],
    ['3pm-4:30pm', '15:00', '16:30'],
    ['from 3 to 5pm', '15:00', '17:00'],
    ['3pm to 5pm', '15:00', '17:00'],
    ['15:00–16:00', '15:00', '16:00'],
  ];
  for (const [range, start, end] of cases) {
    const p = ok(`Thing ${range}`);
    assert.equal(p.start, start, range);
    assert.equal(p.end, end, range);
    assert.equal(p.allDay, false, range);
  }
});

test('bare numbers are not times unless they form a range with am or pm', () => {
  const bare = ok('Thing 3-4');
  assert.equal(bare.title, 'Thing 3-4');
  assert.equal(bare.allDay, true);
  assert.equal(ok('Gym 3').title, 'Gym 3');
  assert.equal(ok('Gym 3').allDay, true);
});

test('at 3 reads as 3 PM and at 9 reads as 24-hour, both noted', () => {
  const three = ok('Lunch at 3');
  assert.equal(three.start, '15:00');
  assert.ok(three.notes.includes('Read 3 as 3 PM'));
  const nine = ok('Bus at 9');
  assert.equal(nine.start, '09:00');
  assert.ok(nine.notes.includes('Read 9 as 09:00'));
});

test('impossible clock times are refused by name', () => {
  assert.equal(fails('Meeting 13pm'), 'Check the time 13pm');
  assert.equal(fails('Meet 25:00'), 'Check the time 25:00');
});

test('a range whose am/pm cannot be inferred asks for it', () => {
  assert.equal(fails('Thing 10pm-1'), 'Add am or pm to the end time');
});

test('durations of every spelling add to the start time', () => {
  const cases: Array<[string, string]> = [
    ['for 90m', '10:30'],
    ['for 90 min', '10:30'],
    ['for 1h', '10:00'],
    ['for 1.5h', '10:30'],
    ['for 2 hours', '11:00'],
    ['for 1h30', '10:30'],
  ];
  for (const [duration, end] of cases) {
    const p = ok(`Thing 9am ${duration}`);
    assert.equal(p.start, '09:00', duration);
    assert.equal(p.end, end, duration);
    assert.equal(p.title, 'Thing', duration);
  }
});

test('a duration that crosses midnight rolls the end to the next day', () => {
  const p = ok('Movie 11pm for 2h');
  assert.equal(p.start, '23:00');
  assert.equal(p.end, '01:00');
  assert.equal(p.endDate, '2026-10-09');
});

test('a missing end time gets the default length, noted with its length', () => {
  const p = ok('Call 9:30pm');
  assert.equal(p.end, '22:30');
  assert.ok(p.notes.includes('No end time; assumed 1 hour'));
  const longer = ok('Call 9:30pm', NOW, 90);
  assert.equal(longer.end, '23:00');
  assert.ok(longer.notes.includes('No end time; assumed 1 hour 30 minutes'));
});

test('a duration without a start time, or with both an end and a duration, is refused', () => {
  assert.equal(fails('Read for 90m'), 'Add a start time for the duration');
  assert.equal(fails('Thing 9am-10am for 1h'), 'Use an end time or a duration, not both');
});

test('midnight as an end time wraps to the next day', () => {
  const p = ok('Late shift 10pm to midnight');
  assert.equal(p.start, '22:00');
  assert.equal(p.end, '00:00');
  assert.equal(p.endDate, '2026-10-09');
});

test('no time means all day, and a timed event cannot span a date range', () => {
  const p = ok('Holiday oct 12');
  assert.equal(p.allDay, true);
  assert.equal(p.start, null);
  assert.equal(p.end, null);
  assert.equal(fails('Test oct 12-14 9am'), 'A timed event takes one date');
});

test('tonight defaults to 7 PM and says so', () => {
  const p = ok('Dinner tonight');
  assert.equal(p.date, '2026-10-08');
  assert.equal(p.start, '19:00');
  assert.ok(p.notes.includes('Tonight starts at 7:00 PM'));
});

test('weekday names and abbreviations resolve to the next occurrence', () => {
  const cases: Array<[string, string]> = [
    ['tues', '2026-10-13'],
    ['thurs', '2026-10-08'],
    ['Monday', '2026-10-12'],
    ['sat', '2026-10-10'],
    ['sun', '2026-10-11'],
  ];
  for (const [word, date] of cases) {
    assert.equal(ok(`Thing ${word}`).date, date, word);
  }
});

test('a weekday range is an inclusive all-day span', () => {
  const p = ok('Conference mon-wed');
  assert.equal(p.date, '2026-10-12');
  assert.equal(p.endDate, '2026-10-14');
  assert.equal(p.allDay, true);
});

test('repeat phrases map to daily, weekly, and monthly rules', () => {
  const cases: Array<[string, QuickParse['repeat']]> = [
    ['Stretch every day', { freq: 'daily', interval: 1, weekdays: [], until: null, count: null }],
    ['Stretch daily', { freq: 'daily', interval: 1, weekdays: [], until: null, count: null }],
    ['Stretch every 3 days', { freq: 'daily', interval: 3, weekdays: [], until: null, count: null }],
    ['Stretch every weekday', { freq: 'weekly', interval: 1, weekdays: ['mon', 'tue', 'wed', 'thu', 'fri'], until: null, count: null }],
    ['Stretch every weekend', { freq: 'weekly', interval: 1, weekdays: ['sat', 'sun'], until: null, count: null }],
    ['Stretch every week', { freq: 'weekly', interval: 1, weekdays: ['thu'], until: null, count: null }],
    ['Stretch weekly', { freq: 'weekly', interval: 1, weekdays: ['thu'], until: null, count: null }],
    ['Stretch every other week', { freq: 'weekly', interval: 2, weekdays: ['thu'], until: null, count: null }],
    ['Stretch every 2 weeks', { freq: 'weekly', interval: 2, weekdays: ['thu'], until: null, count: null }],
    ['Stretch every month', { freq: 'monthly', interval: 1, weekdays: [], until: null, count: null }],
    ['Stretch monthly', { freq: 'monthly', interval: 1, weekdays: [], until: null, count: null }],
  ];
  for (const [text, repeat] of cases) {
    assert.deepEqual(ok(text).repeat, repeat, text);
  }
});

test('a list of weekdays may be comma, slash, or "and" separated, sorted and unique', () => {
  assert.deepEqual(ok('Gym every mon, wed and fri').repeat?.weekdays, ['mon', 'wed', 'fri']);
  assert.deepEqual(ok('Gym every tue/thu').repeat?.weekdays, ['tue', 'thu']);
  assert.deepEqual(ok('Gym every fri mon fri').repeat?.weekdays, ['mon', 'fri']);
});

test('the number of times comes from x10, 10 times, or for 10 weeks', () => {
  assert.equal(ok('Gym every mon x10').repeat?.count, 10);
  assert.equal(ok('Gym every mon 10 times').repeat?.count, 10);
  assert.equal(ok('Gym every mon for 10 weeks').repeat?.count, 10);
  assert.equal(ok('Gym every mon for 10 weeks').title, 'Gym');
});

test('until is an inclusive last date and is shown in the description', () => {
  const p = ok('Gym every mon until oct 26');
  assert.equal(p.repeat?.until, '2026-10-26');
  assert.equal(describe(p), 'Event · Mon, Oct 12 · all day · weekly on Mon until Oct 26, 2026');
});

test('until needs a repeat, must not precede the first occurrence, and must be a date', () => {
  assert.equal(fails('Lab until dec 5'), 'Add a repeat before "until"');
  assert.equal(fails('Gym every mon until oct 1 2026'), 'The until date is before the first occurrence');
  assert.equal(fails('Gym every mon until'), 'Add a date after "until"');
});

test('a repeat phrase that is not understood is refused rather than guessed', () => {
  assert.equal(fails('Gym every 3rd day'), 'Use a repeat like "every mon" or "every 2 weeks"');
});

test('two repeat rules, or a count without a repeat, are refused', () => {
  assert.equal(fails('Gym every day every week'), 'Use one repeat rule');
  assert.equal(fails('Gym x10'), 'Add a repeat before the count');
});

test('two start dates are refused rather than guessed between', () => {
  assert.equal(fails('Oil change tue wed'), 'Use one date');
});

test('a missing date defaults to today and says so', () => {
  const p = ok('Walk 3pm');
  assert.equal(p.date, '2026-10-08');
  assert.equal(p.explicitDate, false);
  assert.ok(p.notes.includes('No date; set for today'));
});

test('a deadline with only a time is due today when no date is given', () => {
  const p = ok('Report due 5pm');
  assert.equal(p.kind, 'deadline');
  assert.equal(p.date, '2026-10-08');
  assert.equal(p.start, '17:00');
  assert.equal(describe(p), 'Deadline · Thu, Oct 8 · due 5:00 PM');
});

test('due inside another word does not make a deadline', () => {
  assert.equal(ok('overdue library book').kind, 'event');
  assert.equal(ok('overdue library book').title, 'overdue library book');
});

test('the same input and clock always give the same result, and next weekday follows the clock', () => {
  assert.deepEqual(ok('Lab Tue 2-4pm every week until Dec 5'), ok('Lab Tue 2-4pm every week until Dec 5'));
  assert.equal(ok('Dentist next tue 3pm', { date: '2026-10-13', time: '10:00' }).date, '2026-10-27');
});

test('descriptions read as the confirmation line for each kind', () => {
  assert.equal(describe(ok('Reading week oct 12-16')), 'Event · Mon, Oct 12 – Fri, Oct 16 · all day');
  assert.equal(describe(ok('maybe read book')), 'Optional · no preferred time');
  assert.equal(describe(ok('Office hours every other week thu 3-4pm')), 'Event · Thu, Oct 8 · 3:00 PM – 4:00 PM · every 2 weeks on Thu');
  assert.equal(describe(ok('Trip 2026-11-20 to 2026-11-23')), 'Event · Fri, Nov 20 – Mon, Nov 23 · all day');
});

test('the title keeps its casing and its words, with spacing collapsed', () => {
  const p = ok('  Team   Sync   tomorrow 2pm ');
  assert.equal(p.title, 'Team Sync');
  assert.equal(p.date, '2026-10-09');
});

test('on the 1st is the next 1st, noted, and repeats monthly when asked', () => {
  const p = ok('Pay rent every month on the 1st');
  assert.equal(p.title, 'Pay rent');
  assert.equal(p.date, '2026-11-01');
  assert.equal(p.allDay, true);
  assert.deepEqual(p.repeat, { freq: 'monthly', interval: 1, weekdays: [], until: null, count: null });
  assert.ok(p.notes.includes('Read "on the 1st" as the next such date'));
  assert.equal(ok('Rent on the 20th').date, '2026-10-20');
});

test('a custom default length changes the assumed end and its note', () => {
  const p = ok('Call 9:30pm', NOW, 45);
  assert.equal(p.end, '22:15');
  assert.ok(p.notes.includes('No end time; assumed 45 minutes'));
});

test('a dotted minute takes am/pm from the other side of a range', () => {
  const p = ok('Thing 3.30-4pm');
  assert.equal(p.start, '15:30');
  assert.equal(p.end, '16:00');
});

test('a month-day range can be written with "to" and an inclusive end', () => {
  const p = ok('Reading oct 12 to oct 14');
  assert.equal(p.title, 'Reading');
  assert.equal(p.date, '2026-10-12');
  assert.equal(p.endDate, '2026-10-14');
  assert.equal(p.allDay, true);
  assert.equal(fails('Reading oct 14 to oct 12'), 'The end date is before the start');
});
