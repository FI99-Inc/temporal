import { test } from 'node:test';
import assert from 'node:assert/strict';
import { emptyForm, editForm, formMutation, wallTime, type LocalState, type TraceChoice } from '../src/lib/local.ts';

const meta = { id: '00000000-0000-4000-8000-000000000001', revision: 1, created_at: '2026-09-09T12:00:00.000Z', updated_at: '2026-09-09T12:00:00.000Z' };
const local = (): LocalState => ({ revision: 1, anchors: [], deadlines: [], intentions: [], routines: [], availability: [], anchor_annotations: [], deadline_annotations: [], task_annotations: [], routine_outcomes: [] });
test('numeric input values and explicit unknown/zero effort reach the mutation without coercion', () => {
  const form = emptyForm('intention'); form.title = 'Synthetic reading'; form.effort = 90; form.chunk = 30;
  const mutation = formMutation(form);
  assert.equal(mutation.effort_minutes, 90); assert.equal(mutation.minimum_chunk_minutes, 30);
  assert.equal(mutation.clear_preference, true); assert.equal(mutation.start, undefined);
  form.effort = undefined; form.chunk = undefined;
  assert.equal(formMutation(form).replace_work, true);
  assert.equal(formMutation(form).effort_minutes, undefined);
  form.effort = 0;
  assert.equal(formMutation(form).effort_minutes, 0);
  assert.equal(formMutation(form).completion, undefined);
});
test('editing a stored all-day anchor keeps date precision, zone, occupancy and significance', () => {
  const state = local();
  state.anchors.push({ meta, title: 'Synthetic marker', presence: 'present', occupancy: 'transparent', reported_certainty: 'tentative', location: 'campus', span: { kind: 'dates', value: { start_date: '2026-09-10', end_date_exclusive: '2026-09-11', zone: 'America/Vancouver' } } });
  state.anchor_annotations.push({ target_id: meta.id, importance: 'high', milestone: true });
  const form = editForm('anchor', meta.id, state, [], 'America/Toronto'); form.title = 'Revised marker';
  const mutation = formMutation(form);
  assert.equal(mutation.zone, 'America/Vancouver'); assert.equal(mutation.all_day, true);
  assert.equal(mutation.start_date, '2026-09-10'); assert.equal(mutation.end_date_exclusive, '2026-09-11');
  assert.equal(mutation.occupancy, 'transparent'); assert.equal(mutation.certainty, 'tentative');
  assert.equal(mutation.milestone, true); assert.equal(mutation.importance, 'high'); assert.equal(mutation.context, 'campus');
  form.milestone = false; assert.equal(formMutation(form).milestone, false);
});
test('a rule edit starts with the stored weekdays, pause state and work fields', () => {
  const state = local(); state.routines.push({ meta, title: 'Synthetic review', presence: 'present', state: 'paused',
    work: { effort: { kind: 'estimate', value: 45 }, minimum_chunk_minutes: 15, required_contexts: ['desk'], energy_requirement: 'normal' },
    rule: { kind: 'weekly', value: { weekdays: ['fri', 'sun'], start_date: '2026-09-04', until_date_exclusive: '2026-10-01', zone: 'America/Toronto' } } });
  const form = editForm('routine', meta.id, state, [], 'UTC'); form.title = 'Changed title';
  const m = formMutation(form);
  assert.deepEqual(m.weekdays, ['fri', 'sun']); assert.equal(m.paused, true);
  assert.equal(m.until_date_exclusive, '2026-10-01'); assert.equal(m.effort_minutes, 45); assert.equal(m.minimum_chunk_minutes, 15);
  form.until = ''; assert.equal(formMutation(form).until_date_exclusive, undefined);
});
test('Trace work selection retains its exact opaque identity and never emits sourced edits', () => {
  const state = local(); const task: TraceChoice = { id: meta.id, external_id: ' opaque ID ', title: 'Imported title', status: 'later', presence: 'removed' };
  state.task_annotations.push({ target_id: meta.id, importance: 'high', work: { effort: { kind: 'estimate', value: 60 }, minimum_chunk_minutes: 20, required_contexts: ['quiet'], energy_requirement: 'deep' } });
  const form = editForm('task_annotation', task.id, state, [task], 'America/Toronto'); form.effort = 90;
  const m = formMutation(form);
  assert.equal(m.trace_external_id, ' opaque ID '); assert.equal(m.effort_minutes, 90);
  assert.equal(m.context, 'quiet'); assert.equal(m.energy_requirement, 'deep'); assert.equal(m.importance, 'high');
  assert.equal(m.title, undefined); assert.equal(m.due, undefined); assert.equal(m.completion, undefined); assert.equal(m.milestone, undefined);
});
test('time formatting preserves seconds, milliseconds and the stored object timezone', () => {
  assert.equal(wallTime('2026-09-10T13:00:12.345Z', 'America/Toronto'), '2026-09-10T09:00:12.345');
  const state = local(); state.availability.push({ meta, span: { start: '2026-09-10T13:00:12.345Z', end: '2026-09-10T14:00:12.345Z', original_zone: 'America/Toronto' }, contexts: { kind: 'unknown' }, energy_capacity: 'unknown' });
  const m = formMutation(editForm('availability', meta.id, state, [], 'UTC'));
  assert.equal(m.start, '2026-09-10T09:00:12.345'); assert.equal(m.end, '2026-09-10T10:00:12.345'); assert.equal(m.zone, 'America/Toronto');
});
