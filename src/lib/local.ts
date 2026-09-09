// Local editor fields are a projection of stored values, never a second evaluator.
export type LocalKind = 'anchor' | 'deadline' | 'intention' | 'routine' | 'availability' | 'task_annotation';
export type Importance = 'unspecified' | 'low' | 'normal' | 'high';
export type Energy = 'unrestricted' | 'light' | 'normal' | 'deep';
export type Capacity = 'unknown' | 'light' | 'normal' | 'deep';
export type Occupancy = 'busy' | 'transparent' | 'unknown';
export type Certainty = 'confirmed' | 'tentative' | 'unspecified';
export type Weekday = 'mon' | 'tue' | 'wed' | 'thu' | 'fri' | 'sat' | 'sun';
export const weekdays: Weekday[] = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'];
type Meta = { id: string; revision: number; created_at: string; updated_at: string };
type Record = { meta: Meta; title: string; presence: 'present' | 'removed' };
export type TimedSpan = { start: string; end: string; original_zone?: string };
type Span = { kind: 'timed'; value: TimedSpan } | { kind: 'dates'; value: { start_date: string; end_date_exclusive: string; zone: string } };
type Work = { effort: { kind: 'unknown' } | { kind: 'estimate' | 'at_least'; value: number }; minimum_chunk_minutes?: number; required_contexts: string[]; energy_requirement: Energy };
type Significance = { target_id: string; importance: Importance; milestone: boolean };
type DeadlineWork = { kind: 'unspecified' } | { kind: 'standalone'; value: Work } | { kind: 'task'; value: string };
export interface TraceChoice { id: string; external_id: string; title: string; status: string; presence: 'present' | 'removed' }
export interface LocalState {
  revision: number;
  anchors: (Record & { span: Span; occupancy: Occupancy; reported_certainty: Certainty; location?: string })[];
  deadlines: (Record & { cutoff: { kind: 'at'; value: { instant: string; original_zone?: string } } | { kind: 'on_date'; value: { date: string; zone: string } }; fulfillment: { kind: 'recorded'; value: { kind: 'unresolved' | 'satisfied' | 'cancelled' } } })[];
  intentions: (Record & { preferred_span?: Span; work?: Work; state: 'active' | 'done' | 'dismissed' })[];
  routines: (Record & { state: 'active' | 'paused'; work: Work; rule: { kind: 'weekly'; value: { weekdays: Weekday[]; start_date: string; until_date_exclusive?: string; zone: string } } })[];
  availability: { meta: Meta; span: TimedSpan; contexts: { kind: 'unknown' } | { kind: 'known'; value: string[] }; energy_capacity: Capacity }[];
  anchor_annotations: Significance[];
  deadline_annotations: (Significance & { work: DeadlineWork })[];
  task_annotations: { target_id: string; importance: Importance; work: Work }[];
  routine_outcomes: { routine_id: string; date: string; outcome: 'done' | 'skipped'; revision: number; recorded_at: string }[];
}
export interface LocalMutation {
  action: 'upsert' | 'remove'; kind: LocalKind | 'routine_outcome'; id?: string;
  title?: string; zone?: string; all_day?: boolean; start?: string; end?: string; due?: string;
  start_date?: string; end_date_exclusive?: string; until_date_exclusive?: string;
  weekdays?: Weekday[]; paused?: boolean; occupancy?: Occupancy; certainty?: Certainty;
  importance?: Importance; milestone?: boolean; trace_external_id?: string;
  effort_minutes?: number; minimum_chunk_minutes?: number; context?: string;
  energy_requirement?: Energy; energy_capacity?: Capacity; replace_work?: boolean;
  clear_preference?: boolean; clear_work?: boolean;
  completion?: 'unresolved' | 'satisfied' | 'cancelled' | 'done' | 'dismissed';
  occurrence_date?: string; outcome?: 'done' | 'skipped';
}
export interface LocalForm {
  kind: LocalKind; id?: string; title: string; zone: string; precision: 'none' | 'timed' | 'dates';
  start: string; end: string; due: string; startDate: string; endDate: string; until: string;
  days: Weekday[]; paused: boolean; effort: number | undefined; chunk: number | undefined;
  context: string; energy: Energy; capacity: Capacity; occupancy: Occupancy; certainty: Certainty;
  importance: Importance; milestone: boolean; traceId: string; workBasis: 'unspecified' | 'standalone' | 'task';
}
export function emptyForm(kind: LocalKind = 'anchor', zone = 'America/Toronto'): LocalForm {
  return { kind, title: '', zone, precision: kind === 'intention' ? 'none' : 'timed',
    start: '', end: '', due: '', startDate: '', endDate: '', until: '', days: [], paused: false,
    effort: undefined, chunk: undefined, context: '', energy: 'unrestricted', capacity: 'unknown',
    occupancy: 'busy', certainty: 'confirmed', importance: 'unspecified', milestone: false,
    traceId: '', workBasis: 'unspecified' };
}
export function wallTime(instant: string | number, zone: string): string {
  const parts = new Intl.DateTimeFormat('en-CA', { timeZone: zone, hourCycle: 'h23',
    year: 'numeric', month: '2-digit', day: '2-digit', hour: '2-digit', minute: '2-digit', second: '2-digit', fractionalSecondDigits: 3,
  }).formatToParts(new Date(instant));
  const p = (name: string) => parts.find(part => part.type === name)!.value;
  return `${p('year')}-${p('month')}-${p('day')}T${p('hour')}:${p('minute')}:${p('second')}.${p('fractionalSecond')}`;
}
function setTimed(form: LocalForm, span: TimedSpan) {
  form.zone = span.original_zone ?? form.zone;
  form.start = wallTime(span.start, form.zone); form.end = wallTime(span.end, form.zone);
}
function setSpan(form: LocalForm, span: Span) {
  form.precision = span.kind === 'timed' ? 'timed' : 'dates';
  if (span.kind === 'timed') setTimed(form, span.value);
  else { form.zone = span.value.zone; form.startDate = span.value.start_date; form.endDate = span.value.end_date_exclusive; }
}
function setWork(form: LocalForm, work?: Work) {
  if (!work) return;
  if (work.effort.kind === 'at_least') throw new Error('This lower-bound estimate needs an explicit replacement before editing.');
  form.effort = work.effort.kind === 'estimate' ? work.effort.value : undefined;
  form.chunk = work.minimum_chunk_minutes; form.context = work.required_contexts.join(', '); form.energy = work.energy_requirement;
}
export function editForm(kind: LocalKind, id: string, local: LocalState, tasks: TraceChoice[], zone: string): LocalForm {
  const form = emptyForm(kind, zone); form.id = id;
  if (kind === 'anchor') {
    const record = local.anchors.find(r => r.meta.id === id)!;
    form.title = record.title; setSpan(form, record.span); form.occupancy = record.occupancy; form.certainty = record.reported_certainty;
    form.context = record.location ?? '';
    const note = local.anchor_annotations.find(r => r.target_id === id);
    if (note) { form.importance = note.importance; form.milestone = note.milestone; }
  } else if (kind === 'deadline') {
    const record = local.deadlines.find(r => r.meta.id === id)!; form.title = record.title;
    if (record.cutoff.kind === 'at') {
      form.zone = record.cutoff.value.original_zone ?? zone; form.due = wallTime(record.cutoff.value.instant, form.zone);
    } else { form.precision = 'dates'; form.zone = record.cutoff.value.zone; form.due = record.cutoff.value.date; }
    const note = local.deadline_annotations.find(r => r.target_id === id);
    if (note) {
      form.importance = note.importance; form.milestone = note.milestone; form.workBasis = note.work.kind;
      if (note.work.kind === 'standalone') setWork(form, note.work.value);
      if (note.work.kind === 'task') {
        const target = note.work.value;
        form.traceId = tasks.find(t => t.id === target)?.external_id ?? '';
        if (!form.traceId) throw new Error('The linked Trace identity is missing from the cache. Reload before editing this link.');
      }
    }
  } else if (kind === 'intention') {
    const record = local.intentions.find(r => r.meta.id === id)!; form.title = record.title;
    if (record.preferred_span) setSpan(form, record.preferred_span); setWork(form, record.work);
  } else if (kind === 'routine') {
    const record = local.routines.find(r => r.meta.id === id)!; form.title = record.title;
    const rule = record.rule.value;
    form.zone = rule.zone; form.startDate = rule.start_date; form.until = rule.until_date_exclusive ?? '';
    form.days = [...rule.weekdays]; form.paused = record.state === 'paused'; setWork(form, record.work);
  } else if (kind === 'availability') {
    const record = local.availability.find(r => r.meta.id === id)!;
    setTimed(form, record.span); form.context = record.contexts.kind === 'known' ? record.contexts.value.join(', ') : '';
    form.capacity = record.energy_capacity;
  } else {
    const task = tasks.find(t => t.id === id)!; form.id = undefined; form.traceId = task.external_id;
    const note = local.task_annotations.find(r => r.target_id === id);
    if (note) { form.importance = note.importance; setWork(form, note.work); }
  }
  return form;
}
export function formMutation(form: LocalForm): LocalMutation {
  const m: LocalMutation = { action: 'upsert', kind: form.kind };
  if (form.id) m.id = form.id;
  const kind = form.kind;
  if (['anchor', 'deadline', 'intention', 'routine'].includes(kind)) m.title = form.title.trim();
  if (kind !== 'task_annotation') m.zone = form.zone;
  if (kind === 'anchor' || kind === 'intention') {
    if (form.precision === 'none') m.clear_preference = true;
    else if (form.precision === 'dates') { m.all_day = true; m.start_date = form.startDate; m.end_date_exclusive = form.endDate; }
    else { m.start = form.start; m.end = form.end; }
  }
  if (kind === 'deadline') { m.all_day = form.precision === 'dates'; m.due = form.due; }
  if (kind === 'anchor') { m.occupancy = form.occupancy; m.certainty = form.certainty; m.context = form.context; }
  if (kind === 'routine') { m.weekdays = form.days; m.start_date = form.startDate; if (form.until) m.until_date_exclusive = form.until; m.paused = form.paused; }
  if (kind === 'availability') { m.start = form.start; m.end = form.end; m.context = form.context; m.energy_capacity = form.capacity; }
  if (kind === 'task_annotation' || (kind === 'deadline' && form.workBasis === 'task')) m.trace_external_id = form.traceId;
  if (kind === 'deadline' && form.workBasis === 'unspecified') m.clear_work = true;
  if (['intention', 'routine', 'task_annotation'].includes(kind) || (kind === 'deadline' && form.workBasis === 'standalone')) {
    for (const value of [form.effort, form.chunk]) {
      if (value !== undefined && (!Number.isInteger(value) || value < 0 || value > 4_294_967_295)) throw new Error('Enter whole minutes within the supported range.');
    }
    m.replace_work = true; m.effort_minutes = form.effort; m.minimum_chunk_minutes = form.chunk;
    m.context = form.context; m.energy_requirement = form.energy;
  }
  if (kind === 'anchor' || kind === 'deadline') { m.importance = form.importance; m.milestone = form.milestone; }
  if (kind === 'task_annotation') m.importance = form.importance;
  return m;
}
