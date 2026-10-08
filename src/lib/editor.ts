// The event editor's model: a flat draft, and the exact local mutations that
// save or delete it. Rust validates and stores; nothing here evaluates time.
import type { Weekday } from './calendar-types.ts';
import { addDays, weekdayOf } from './calendar.ts';
import { wallTime, type AnchorSeries, type LocalMutation, type LocalState } from './local.ts';
import type { QuickParse } from './quickadd.ts';

export type EditorKind = 'event' | 'deadline' | 'intention';
export type RepeatChoice = 'none' | 'daily' | 'weekly' | 'weekdays' | 'monthly' | 'custom';
export type Scope = 'one' | 'all';
const WEEKDAYS: Weekday[] = ['mon', 'tue', 'wed', 'thu', 'fri'];
const ORDER: Weekday[] = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'];

export interface EventDraft {
  kind: EditorKind;
  title: string;
  allDay: boolean;
  /** First civil date (`YYYY-MM-DD`). */
  date: string;
  /** Inclusive last date of an all-day event. */
  endDate: string;
  /** Wall times `HH:MM`; an end at or before the start ends the next day. */
  start: string;
  end: string;
  zone: string;
  repeat: RepeatChoice;
  /** The period of a custom repeat. */
  unit: 'day' | 'week' | 'month';
  interval: number;
  weekdays: Weekday[];
  ends: 'never' | 'on' | 'after';
  /** Inclusive last date of a repeat. */
  until: string;
  count: number;
  place: string;
  notes: string;
  busy: boolean;
  tentative: boolean;
  /** Remaining work for a deadline or optional item, in minutes. */
  effort: number | null;
  /** Smallest useful work session, in minutes. */
  chunk: number | null;
  /** An optional item without a preferred day. */
  noDate: boolean;
}

/** What the editor is changing. */
export type EditTarget =
  | { kind: 'new' }
  | { kind: 'anchor'; id: string }
  | { kind: 'series'; id: string; occurrenceDate: string; firstDate: string }
  | { kind: 'deadline'; id: string; hadWork: boolean }
  | { kind: 'intention'; id: string };

const pad = (n: number) => String(n).padStart(2, '0');
export const hhmm = (minutes: number) => `${pad(Math.floor(minutes / 60) % 24)}:${pad(minutes % 60)}`;
export const minutesOf = (value: string) => { const [h, m] = value.split(':').map(Number); return h * 60 + m; };

export function blankDraft(date: string, zone: string, start = '09:00', minutes = 60): EventDraft {
  const end = hhmm(minutesOf(start) + minutes);
  return {
    kind: 'event', title: '', allDay: false, date, endDate: date, start, end, zone,
    repeat: 'none', unit: 'week', interval: 1, weekdays: [weekdayOf(date)], ends: 'never', until: addDays(date, 90), count: 10,
    place: '', notes: '', busy: true, tentative: false, effort: null, chunk: null, noDate: false,
  };
}

/** A draft from a parsed quick-add line; the person reviews it before saving. */
export function draftFromQuick(q: QuickParse, zone: string, defaultMinutes: number): EventDraft {
  const draft = blankDraft(q.date, zone, q.start ?? '09:00', defaultMinutes);
  draft.kind = q.kind;
  draft.title = q.title;
  draft.allDay = q.allDay;
  draft.endDate = q.allDay && q.endDate ? q.endDate : q.date;
  if (q.start) draft.start = q.start;
  if (q.end) draft.end = q.end;
  draft.place = q.location ?? '';
  draft.noDate = q.kind === 'intention' && !q.explicitDate;
  if (q.repeat) {
    const r = q.repeat;
    draft.interval = r.interval;
    draft.weekdays = r.weekdays.length ? [...r.weekdays] : [weekdayOf(q.date)];
    draft.unit = r.freq === 'daily' ? 'day' : r.freq === 'monthly' ? 'month' : 'week';
    draft.repeat = r.freq === 'daily' ? (r.interval === 1 ? 'daily' : 'custom')
      : r.freq === 'monthly' ? (r.interval === 1 ? 'monthly' : 'custom')
      : r.interval === 1 && sameDays(draft.weekdays, WEEKDAYS) ? 'weekdays'
      : r.interval === 1 && sameDays(draft.weekdays, [weekdayOf(q.date)]) ? 'weekly' : 'custom';
    if (r.until) { draft.ends = 'on'; draft.until = r.until; }
    else if (r.count) { draft.ends = 'after'; draft.count = r.count; }
  }
  return draft;
}

function sameDays(a: Weekday[], b: Weekday[]): boolean {
  return a.length === b.length && a.every(day => b.includes(day));
}

function splitWall(value: string): { date: string; time: string } {
  return { date: value.slice(0, 10), time: value.slice(11, 16) };
}

export function draftFromAnchor(local: LocalState, id: string, zone: string): EventDraft {
  const record = local.anchors.find(r => r.meta.id === id);
  if (!record) throw new Error('That event is no longer stored.');
  const details = local.event_details[id] ?? {};
  let draft: EventDraft;
  if (record.span.kind === 'timed') {
    const z = record.span.value.original_zone ?? zone;
    const start = splitWall(wallTime(record.span.value.start, z));
    const end = splitWall(wallTime(record.span.value.end, z));
    draft = blankDraft(start.date, z, start.time);
    draft.end = end.time;
  } else {
    draft = blankDraft(record.span.value.start_date, record.span.value.zone);
    draft.allDay = true;
    draft.endDate = addDays(record.span.value.end_date_exclusive, -1);
  }
  draft.title = record.title;
  draft.busy = record.occupancy !== 'transparent';
  draft.tentative = record.reported_certainty === 'tentative';
  draft.place = details.place ?? '';
  draft.notes = details.notes ?? '';
  return draft;
}

export function draftFromSeries(series: AnchorSeries, occurrenceDate: string): EventDraft {
  const draft = blankDraft(occurrenceDate, series.zone);
  draft.title = series.title;
  if (series.timing.kind === 'timed') {
    draft.start = hhmm(series.timing.start_minute);
    draft.end = hhmm(series.timing.start_minute + series.timing.duration_minutes);
  } else {
    draft.allDay = true;
    draft.endDate = addDays(occurrenceDate, series.timing.days - 1);
  }
  const rule = series.rule;
  draft.interval = rule.interval;
  draft.weekdays = rule.weekdays.length ? [...rule.weekdays] : [weekdayOf(series.first_date)];
  draft.unit = rule.frequency === 'daily' ? 'day' : rule.frequency === 'monthly' ? 'month' : 'week';
  draft.repeat = rule.frequency === 'daily' ? (rule.interval === 1 ? 'daily' : 'custom')
    : rule.frequency === 'monthly' ? (rule.interval === 1 ? 'monthly' : 'custom')
    : rule.interval === 1 && sameDays(draft.weekdays, WEEKDAYS) ? 'weekdays'
    : rule.interval === 1 && sameDays(draft.weekdays, [weekdayOf(series.first_date)]) ? 'weekly' : 'custom';
  if (rule.until_date_exclusive) { draft.ends = 'on'; draft.until = addDays(rule.until_date_exclusive, -1); }
  else if (rule.count) { draft.ends = 'after'; draft.count = rule.count; }
  draft.busy = series.occupancy !== 'transparent';
  draft.tentative = series.reported_certainty === 'tentative';
  draft.place = series.details?.place ?? '';
  draft.notes = series.details?.notes ?? '';
  return draft;
}

export function draftFromDeadline(local: LocalState, id: string, zone: string): EventDraft {
  const record = local.deadlines.find(r => r.meta.id === id);
  if (!record) throw new Error('That deadline is no longer stored.');
  let draft: EventDraft;
  if (record.cutoff.kind === 'at') {
    const z = record.cutoff.value.original_zone ?? zone;
    const due = splitWall(wallTime(record.cutoff.value.instant, z));
    draft = blankDraft(due.date, z, due.time);
  } else {
    draft = blankDraft(record.cutoff.value.date, record.cutoff.value.zone);
    draft.allDay = true;
  }
  draft.kind = 'deadline';
  draft.title = record.title;
  const note = local.deadline_annotations.find(a => a.target_id === id);
  if (note?.work.kind === 'standalone') {
    const effort = note.work.value.effort;
    draft.effort = effort.kind === 'unknown' ? null : effort.value;
    draft.chunk = note.work.value.minimum_chunk_minutes ?? null;
  }
  const details = local.event_details[id] ?? {};
  draft.place = details.place ?? '';
  draft.notes = details.notes ?? '';
  return draft;
}

export function draftFromIntention(local: LocalState, id: string, zone: string): EventDraft {
  const record = local.intentions.find(r => r.meta.id === id);
  if (!record) throw new Error('That item is no longer stored.');
  const today = wallTime(Date.parse(record.meta.updated_at), zone).slice(0, 10);
  let draft = blankDraft(today, zone);
  draft.noDate = true;
  draft.allDay = true;
  const span = record.preferred_span;
  if (span?.kind === 'dates') {
    draft = blankDraft(span.value.start_date, span.value.zone);
    draft.allDay = true;
    draft.endDate = addDays(span.value.end_date_exclusive, -1);
  } else if (span?.kind === 'timed') {
    const z = span.value.original_zone ?? zone;
    const start = splitWall(wallTime(span.value.start, z));
    draft = blankDraft(start.date, z, start.time);
    draft.end = splitWall(wallTime(span.value.end, z)).time;
  }
  draft.kind = 'intention';
  draft.title = record.title;
  if (record.work) {
    draft.effort = record.work.effort.kind === 'unknown' ? null : record.work.effort.value;
    draft.chunk = record.work.minimum_chunk_minutes ?? null;
  }
  return draft;
}

/** A human message for an incomplete draft, or null when it can be saved. */
export function validateDraft(draft: EventDraft): string | null {
  if (!draft.title.trim()) return 'Add a title.';
  if (!/^\d{4}-\d{2}-\d{2}$/.test(draft.date) && !(draft.kind === 'intention' && draft.noDate)) return 'Choose a date.';
  if (draft.kind === 'event' && draft.allDay && draft.endDate < draft.date) return 'The last day cannot be before the first.';
  if (draft.kind === 'event' && !draft.allDay && draft.start === draft.end) return 'The event needs an end time after its start.';
  if (draft.repeat !== 'none' && draft.kind === 'event') {
    if (draft.repeat === 'custom' && (draft.interval < 1 || draft.interval > 99)) return 'Repeat every 1 to 99 periods.';
    if (draft.ends === 'on' && draft.until < draft.date) return 'The repeat must end on or after the first day.';
    if (draft.ends === 'after' && (draft.count < 1 || draft.count > 5000)) return 'Repeat between 1 and 5000 times.';
    if (draft.repeat === 'custom' && draft.unit === 'week' && draft.weekdays.length === 0) return 'Choose at least one weekday.';
  }
  for (const value of [draft.effort, draft.chunk]) {
    if (value !== null && (!Number.isInteger(value) || value < 0 || value > 100_000)) return 'Use whole minutes for work estimates.';
  }
  if (draft.effort !== null && draft.chunk !== null && draft.chunk > draft.effort) return 'The smallest session cannot be longer than the work left.';
  if (draft.chunk === 0) return 'The smallest session must be at least one minute.';
  return null;
}

function timedFields(draft: EventDraft, date: string): Partial<LocalMutation> {
  const wraps = minutesOf(draft.end) <= minutesOf(draft.start);
  return { start: `${date}T${draft.start}`, end: `${wraps ? addDays(date, 1) : date}T${draft.end}` };
}

function spanFields(draft: EventDraft, date: string): Partial<LocalMutation> {
  if (draft.allDay) {
    const days = Math.max(1, dayCount(draft.date, draft.endDate));
    return { all_day: true, start_date: date, end_date_exclusive: addDays(date, days) };
  }
  return timedFields(draft, date);
}

function dayCount(first: string, lastInclusive: string): number {
  return Math.round((Date.parse(`${lastInclusive}T00:00:00Z`) - Date.parse(`${first}T00:00:00Z`)) / 86_400_000) + 1;
}

function eventFields(draft: EventDraft): Partial<LocalMutation> {
  return {
    title: draft.title.trim(), zone: draft.zone,
    occupancy: draft.busy ? 'busy' : 'transparent',
    certainty: draft.tentative ? 'tentative' : 'confirmed',
    place: draft.place.trim(), notes: draft.notes.trim(),
  };
}

function ruleFields(draft: EventDraft): Partial<LocalMutation> {
  const frequency = draft.repeat === 'daily' || (draft.repeat === 'custom' && draft.unit === 'day') ? 'daily'
    : draft.repeat === 'monthly' || (draft.repeat === 'custom' && draft.unit === 'month') ? 'monthly' : 'weekly';
  const weekdays = draft.repeat === 'weekdays' ? WEEKDAYS
    : draft.repeat === 'weekly' ? [weekdayOf(draft.date)]
    : draft.repeat === 'custom' ? ORDER.filter(day => draft.weekdays.includes(day)) : [];
  return {
    frequency,
    interval: draft.repeat === 'custom' ? draft.interval : 1,
    weekdays: frequency === 'weekly' ? weekdays : [],
    ...(draft.ends === 'on' ? { until_date_exclusive: addDays(draft.until, 1) } : {}),
    ...(draft.ends === 'after' ? { count: draft.count } : {}),
  };
}

function workFields(draft: EventDraft): Partial<LocalMutation> {
  if (draft.effort === null && draft.chunk === null) return {};
  return {
    replace_work: true,
    ...(draft.effort !== null ? { effort_minutes: draft.effort } : {}),
    ...(draft.chunk !== null ? { minimum_chunk_minutes: draft.chunk } : {}),
  };
}

const single = (draft: EventDraft, id?: string): LocalMutation =>
  ({ action: 'upsert', kind: 'anchor', ...(id ? { id } : {}), ...eventFields(draft), ...spanFields(draft, draft.date) });
const series = (draft: EventDraft, firstDate: string, id?: string): LocalMutation =>
  ({ action: 'upsert', kind: 'anchor_series', ...(id ? { id } : {}), ...eventFields(draft), ...spanFields(draft, firstDate), ...ruleFields(draft) });

/** The mutations that save `draft` over `target`, applied as one batch. */
export function saveMutations(draft: EventDraft, target: EditTarget, scope: Scope = 'all'): LocalMutation[] {
  if (draft.kind === 'deadline') {
    const id = target.kind === 'deadline' ? target.id : undefined;
    const work = workFields(draft);
    const clear = target.kind === 'deadline' && target.hadWork && draft.effort === null && draft.chunk === null;
    return [{
      action: 'upsert', kind: 'deadline', ...(id ? { id } : {}), title: draft.title.trim(), zone: draft.zone,
      all_day: draft.allDay, due: draft.allDay ? draft.date : `${draft.date}T${draft.start}`,
      place: draft.place.trim(), notes: draft.notes.trim(), ...work, ...(clear ? { clear_work: true } : {}),
    }];
  }
  if (draft.kind === 'intention') {
    const id = target.kind === 'intention' ? target.id : undefined;
    const when: Partial<LocalMutation> = draft.noDate ? { clear_preference: true } : spanFields(draft, draft.date);
    return [{ action: 'upsert', kind: 'intention', ...(id ? { id } : {}), title: draft.title.trim(), zone: draft.zone, ...when, replace_work: true,
      ...(draft.effort !== null ? { effort_minutes: draft.effort } : {}), ...(draft.chunk !== null ? { minimum_chunk_minutes: draft.chunk } : {}) }];
  }
  const repeats = draft.repeat !== 'none';
  switch (target.kind) {
    case 'new': case 'deadline': case 'intention':
      return [repeats ? series(draft, draft.date) : single(draft)];
    case 'anchor':
      return repeats
        ? [{ action: 'remove', kind: 'anchor', id: target.id }, series(draft, draft.date)]
        : [single(draft, target.id)];
    case 'series':
      if (scope === 'one') {
        return [
          { action: 'upsert', kind: 'series_skip', id: target.id, occurrence_date: target.occurrenceDate },
          single({ ...draft, repeat: 'none' }),
        ];
      }
      return repeats
        ? [series(draft, target.firstDate, target.id)]
        : [{ action: 'remove', kind: 'anchor_series', id: target.id }, single(draft)];
  }
}

/** The mutations that delete `target`; `scope` matters only for a series. */
export function deleteMutations(target: EditTarget, scope: Scope = 'all'): LocalMutation[] {
  switch (target.kind) {
    case 'new': return [];
    case 'anchor': return [{ action: 'remove', kind: 'anchor', id: target.id }];
    case 'deadline': return [{ action: 'remove', kind: 'deadline', id: target.id }];
    case 'intention': return [{ action: 'remove', kind: 'intention', id: target.id }];
    case 'series': return scope === 'one'
      ? [{ action: 'upsert', kind: 'series_skip', id: target.id, occurrence_date: target.occurrenceDate }]
      : [{ action: 'remove', kind: 'anchor_series', id: target.id }];
  }
}

/** A short, plain description of a repeat rule for the editor and details. */
export function describeRepeat(draft: EventDraft): string {
  const names: Record<Weekday, string> = { mon: 'Mon', tue: 'Tue', wed: 'Wed', thu: 'Thu', fri: 'Fri', sat: 'Sat', sun: 'Sun' };
  let text = draft.repeat === 'none' ? 'Does not repeat'
    : draft.repeat === 'daily' ? 'Every day'
    : draft.repeat === 'weekdays' ? 'Every weekday'
    : draft.repeat === 'weekly' ? `Every week on ${names[weekdayOf(draft.date)]}`
    : draft.repeat === 'monthly' ? `Every month on day ${Number(draft.date.slice(8, 10))}`
    : draft.unit === 'week'
      ? `Every ${draft.interval === 1 ? '' : `${draft.interval} `}week${draft.interval === 1 ? '' : 's'} on ${ORDER.filter(d => draft.weekdays.includes(d)).map(d => names[d]).join(', ')}`
      : `Every ${draft.interval === 1 ? '' : `${draft.interval} `}${draft.unit}${draft.interval === 1 ? '' : 's'}`;
  if (draft.repeat !== 'none') {
    if (draft.ends === 'on') text += `, until ${draft.until}`;
    if (draft.ends === 'after') text += `, ${draft.count} times`;
  }
  return text;
}
