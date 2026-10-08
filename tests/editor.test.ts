import { test } from 'node:test';
import assert from 'node:assert/strict';
import { blankDraft, deleteMutations, describeRepeat, draftFromQuick, draftFromSeries, saveMutations, validateDraft } from '../src/lib/editor.ts';
import { parseQuickAdd, type QuickParse } from '../src/lib/quickadd.ts';
import type { AnchorSeries } from '../src/lib/local.ts';

const zone = 'America/Toronto';
const meta = { id: '00000000-0000-4000-8000-0000000000aa', revision: 1, created_at: '2026-09-09T12:00:00.000Z', updated_at: '2026-09-09T12:00:00.000Z' };

test('a new single event saves one anchor with wall times in its zone', () => {
  const draft = blankDraft('2026-10-13', zone, '14:00', 90);
  draft.title = ' Lab ';
  draft.place = 'BA 1130';
  const [m] = saveMutations(draft, { kind: 'new' });
  assert.deepEqual(m, { action: 'upsert', kind: 'anchor', title: 'Lab', zone, occupancy: 'busy', certainty: 'confirmed', place: 'BA 1130', notes: '', start: '2026-10-13T14:00', end: '2026-10-13T15:30' });
});

test('an end at or before the start ends the next day', () => {
  const draft = blankDraft('2026-10-09', zone, '22:00');
  draft.title = 'Party'; draft.end = '01:00';
  const [m] = saveMutations(draft, { kind: 'new' });
  assert.equal(m.end, '2026-10-10T01:00');
});

test('all-day events send an exclusive end date', () => {
  const draft = blankDraft('2026-10-12', zone);
  draft.title = 'Reading week'; draft.allDay = true; draft.endDate = '2026-10-16';
  const [m] = saveMutations(draft, { kind: 'new' });
  assert.equal(m.all_day, true); assert.equal(m.start_date, '2026-10-12'); assert.equal(m.end_date_exclusive, '2026-10-17');
});

test('repeating events become a series with an exclusive until date', () => {
  const draft = blankDraft('2026-10-13', zone, '14:00', 120);
  draft.title = 'Lab'; draft.repeat = 'weekly'; draft.ends = 'on'; draft.until = '2026-12-05';
  const [m] = saveMutations(draft, { kind: 'new' });
  assert.equal(m.kind, 'anchor_series'); assert.equal(m.frequency, 'weekly');
  assert.deepEqual(m.weekdays, ['tue']); assert.equal(m.until_date_exclusive, '2026-12-06');
  assert.equal(describeRepeat(draft), 'Every week on Tue, until 2026-12-05');
  draft.repeat = 'custom'; draft.unit = 'month'; draft.interval = 2; draft.ends = 'after'; draft.count = 4;
  const [monthly] = saveMutations(draft, { kind: 'new' });
  assert.equal(monthly.frequency, 'monthly'); assert.equal(monthly.interval, 2); assert.equal(monthly.count, 4);
  assert.deepEqual(monthly.weekdays, []);
});

test('editing one occurrence skips it and adds a standalone event; all keeps the series start', () => {
  const draft = blankDraft('2026-10-15', zone, '15:00', 60);
  draft.title = 'Studio'; draft.repeat = 'weekly';
  const target = { kind: 'series' as const, id: 'series-1', occurrenceDate: '2026-10-15', firstDate: '2026-09-10' };
  const one = saveMutations(draft, target, 'one');
  assert.deepEqual(one[0], { action: 'upsert', kind: 'series_skip', id: 'series-1', occurrence_date: '2026-10-15' });
  assert.equal(one[1].kind, 'anchor'); assert.equal(one[1].start, '2026-10-15T15:00');
  const all = saveMutations(draft, target, 'all');
  assert.equal(all.length, 1); assert.equal(all[0].id, 'series-1'); assert.equal(all[0].start, '2026-09-10T15:00');
  draft.repeat = 'none';
  const stop = saveMutations(draft, target, 'all');
  assert.deepEqual(stop.map(m => [m.action, m.kind]), [['remove', 'anchor_series'], ['upsert', 'anchor']]);
});

test('a single event that starts repeating is replaced by a series in one batch', () => {
  const draft = blankDraft('2026-10-13', zone);
  draft.title = 'Gym'; draft.repeat = 'weekdays';
  const batch = saveMutations(draft, { kind: 'anchor', id: 'a-1' });
  assert.deepEqual(batch[0], { action: 'remove', kind: 'anchor', id: 'a-1' });
  assert.deepEqual(batch[1].weekdays, ['mon', 'tue', 'wed', 'thu', 'fri']);
});

test('deletes respect scope', () => {
  const target = { kind: 'series' as const, id: 's', occurrenceDate: '2026-10-15', firstDate: '2026-09-10' };
  assert.equal(deleteMutations(target, 'one')[0].kind, 'series_skip');
  assert.deepEqual(deleteMutations(target, 'all')[0], { action: 'remove', kind: 'anchor_series', id: 's' });
  assert.deepEqual(deleteMutations({ kind: 'new' }), []);
});

test('deadlines carry work only when given, and clear previous work explicitly', () => {
  const draft = blankDraft('2026-10-09', zone, '23:59');
  draft.kind = 'deadline'; draft.title = 'PS3'; draft.effort = 180; draft.chunk = 45;
  const [m] = saveMutations(draft, { kind: 'new' });
  assert.equal(m.due, '2026-10-09T23:59'); assert.equal(m.effort_minutes, 180); assert.equal(m.minimum_chunk_minutes, 45); assert.equal(m.replace_work, true);
  draft.effort = null; draft.chunk = null; draft.allDay = true;
  const [edit] = saveMutations(draft, { kind: 'deadline', id: 'd', hadWork: true });
  assert.equal(edit.due, '2026-10-09'); assert.equal(edit.all_day, true); assert.equal(edit.clear_work, true); assert.equal(edit.effort_minutes, undefined);
});

test('an optional item without a day clears its preference', () => {
  const draft = blankDraft('2026-10-08', zone);
  draft.kind = 'intention'; draft.title = 'Call grandma'; draft.noDate = true;
  const [m] = saveMutations(draft, { kind: 'new' });
  assert.equal(m.kind, 'intention'); assert.equal(m.clear_preference, true); assert.equal(m.start, undefined);
});

test('quick add drafts keep the parser interpretation', () => {
  const parsed = parseQuickAdd('Lab Tue 2-4pm every week until Dec 5', { date: '2026-10-08', time: '10:00' }) as QuickParse;
  const draft = draftFromQuick(parsed, zone, 60);
  assert.equal(draft.title, 'Lab'); assert.equal(draft.date, '2026-10-13'); assert.equal(draft.start, '14:00'); assert.equal(draft.end, '16:00');
  assert.equal(draft.repeat, 'weekly'); assert.equal(draft.ends, 'on'); assert.equal(draft.until, '2026-12-05');
  const due = draftFromQuick(parseQuickAdd('PS3 due Fri 11:59pm', { date: '2026-10-08', time: '10:00' }) as QuickParse, zone, 60);
  assert.equal(due.kind, 'deadline'); assert.equal(due.start, '23:59'); assert.equal(due.date, '2026-10-09');
});

test('series drafts round-trip their rule', () => {
  const series: AnchorSeries = {
    meta, title: 'Seminar', presence: 'present', zone, first_date: '2026-09-07',
    timing: { kind: 'timed', start_minute: 600, duration_minutes: 90 },
    rule: { frequency: 'weekly', interval: 2, weekdays: ['mon', 'wed'], until_date_exclusive: '2026-12-08' },
    skipped: [], occupancy: 'busy', reported_certainty: 'tentative', details: { place: 'Room 2' },
  };
  const draft = draftFromSeries(series, '2026-09-21');
  assert.equal(draft.start, '10:00'); assert.equal(draft.end, '11:30');
  assert.equal(draft.repeat, 'custom'); assert.equal(draft.unit, 'week'); assert.equal(draft.interval, 2);
  assert.equal(draft.until, '2026-12-07'); assert.equal(draft.tentative, true); assert.equal(draft.place, 'Room 2');
  const [m] = saveMutations(draft, { kind: 'series', id: 's', occurrenceDate: '2026-09-21', firstDate: '2026-09-07' });
  assert.equal(m.start, '2026-09-07T10:00'); assert.deepEqual(m.weekdays, ['mon', 'wed']); assert.equal(m.interval, 2); assert.equal(m.until_date_exclusive, '2026-12-08');
});

test('validation explains what is missing', () => {
  const draft = blankDraft('2026-10-08', zone);
  assert.equal(validateDraft(draft), 'Add a title.');
  draft.title = 'x'; draft.end = draft.start;
  assert.match(validateDraft(draft)!, /end time/);
  draft.end = '10:00'; draft.repeat = 'custom'; draft.unit = 'week'; draft.weekdays = [];
  assert.match(validateDraft(draft)!, /weekday/);
  draft.repeat = 'none'; draft.kind = 'deadline'; draft.effort = 30; draft.chunk = 60;
  assert.match(validateDraft(draft)!, /smallest session/);
});
