<script lang="ts">
  import type { Item, ItemLink, PersonalView } from '../lib/api.ts';
  import type { CalendarEvent } from '../lib/calendar-types.ts';
  import type { LocalMutation } from '../lib/local.ts';
  import { deleteMutations, draftFromAnchor, draftFromDeadline, draftFromIntention, draftFromSeries, type EditTarget, type EventDraft } from '../lib/editor.ts';

  let { item = null, event = null, personal, busy, onedit, onapply, onclose }: {
    /** The Horizon evaluation's view of this fact, when it is inside the Horizon. */
    item?: Item | null;
    /** The Calendar's row for this fact, when opened from the Calendar. */
    event?: CalendarEvent | null;
    personal: PersonalView | null;
    busy: boolean;
    onedit: (target: EditTarget, draft: EventDraft) => void;
    onapply: (label: string, mutations: LocalMutation[]) => Promise<boolean>;
    onclose: () => void;
  } = $props();

  let confirming = $state<'delete-one' | 'delete-all' | null>(null);
  const id = $derived(event?.id ?? item?.id ?? '');
  const link = $derived<ItemLink | null>(personal?.links[id] ?? null);
  const local = $derived(personal?.local ?? null);
  const zone = $derived(personal?.view.zone ?? 'America/Toronto');
  const species = $derived(event?.kind ?? item?.species ?? 'anchor');
  const title = $derived(event?.title ?? item?.title ?? '');
  const seriesId = $derived(event?.kind === 'anchor' ? event.series_id : link?.series_id ?? null);
  const occurrenceDate = $derived(event?.occurrence_date ?? link?.occurrence_date ?? null);
  const series = $derived(seriesId && local ? local.anchor_series.find(s => s.meta.id === seriesId) ?? null : null);
  const localAnchor = $derived(local?.anchors.find(a => a.meta.id === id && a.presence === 'present') ?? null);
  const localDeadline = $derived(local?.deadlines.find(d => d.meta.id === id && d.presence === 'present') ?? null);
  const intention = $derived(local?.intentions.find(i => i.meta.id === id && i.presence === 'present') ?? null);
  const routineId = $derived(species === 'routine' ? id.split(':')[0] : null);
  const routineDate = $derived(species === 'routine' ? (event?.occurrence_date ?? id.split(':')[1] ?? null) : null);
  const routineOutcome = $derived(routineId && routineDate ? local?.routine_outcomes.find(o => o.routine_id === routineId && o.date === routineDate)?.outcome ?? null : null);
  const imported = $derived(Boolean(link?.imported || (event && !event.editable)));
  const handled = $derived(Boolean(local?.handled_imports[id]));
  const place = $derived(event?.location ?? link?.place ?? null);
  const notes = $derived(event?.notes ?? link?.notes ?? null);
  const sourceLabel = $derived(event?.source_label ?? item?.source ?? '');
  const color = $derived(event?.color ?? link?.color ?? null);
  const kindLabel = $derived(({ anchor: 'Fixed event', deadline: 'Deadline', task: 'Trace task', intention: 'Optional', routine: 'Routine', window: 'Open time', suggestion: 'Suggestion' } as Record<string, string>)[species] ?? species);
  const when = $derived(item?.when ?? (event ? eventWhen(event) : ''));
  const readable = (value: string) => value.replaceAll('_', ' ');

  function eventWhen(e: CalendarEvent): string {
    const fmt = (ms: number, opts: Intl.DateTimeFormatOptions) => new Intl.DateTimeFormat('en-US', { timeZone: zone, ...opts }).format(new Date(ms));
    if (e.all_day && e.start_date) {
      const day = (d: string) => new Intl.DateTimeFormat('en-US', { timeZone: 'UTC', weekday: 'short', month: 'short', day: 'numeric' }).format(new Date(`${d}T12:00:00Z`));
      const last = e.end_date ? new Date(Date.parse(`${e.end_date}T12:00:00Z`) - 86_400_000).toISOString().slice(0, 10) : e.start_date;
      return last === e.start_date ? `${day(e.start_date)} · all day` : `${day(e.start_date)} – ${day(last)}`;
    }
    const date = fmt(e.start, { weekday: 'short', month: 'short', day: 'numeric' });
    const time = (ms: number) => fmt(ms, { hour: 'numeric', minute: '2-digit' });
    return e.kind === 'deadline' ? `${date} · due ${time(e.start)}` : `${date} · ${time(e.start)} – ${time(e.end)}`;
  }

  function edit() {
    try {
      if (series && occurrenceDate) onedit({ kind: 'series', id: series.meta.id, occurrenceDate, firstDate: series.first_date }, draftFromSeries(series, occurrenceDate));
      else if (localAnchor && local) onedit({ kind: 'anchor', id }, draftFromAnchor(local, id, zone));
      else if (localDeadline && local) {
        const note = local.deadline_annotations.find(a => a.target_id === id);
        onedit({ kind: 'deadline', id, hadWork: note?.work.kind === 'standalone' }, draftFromDeadline(local, id, zone));
      } else if (intention && local) onedit({ kind: 'intention', id }, draftFromIntention(local, id, zone));
    } catch (e) { void onapply(String(e instanceof Error ? e.message : e), []); }
  }
  async function remove(scope: 'one' | 'all') {
    const target: EditTarget | null = series && occurrenceDate ? { kind: 'series', id: series.meta.id, occurrenceDate, firstDate: series.first_date }
      : localAnchor ? { kind: 'anchor', id } : localDeadline ? { kind: 'deadline', id, hadWork: false } : intention ? { kind: 'intention', id } : null;
    if (!target) return;
    confirming = null;
    if (await onapply(scope === 'one' && target.kind === 'series' ? 'Occurrence removed' : 'Deleted', deleteMutations(target, scope))) onclose();
  }
  const editable = $derived(Boolean(series || localAnchor || localDeadline || intention));
  // Imported deadlines and Trace tasks keep their source facts; a local work
  // estimate beside them is what lets pressure and Today assess the work.
  const traceTask = $derived(species === 'task' ? personal?.trace.tasks.find(t => t.id === id) ?? null : null);
  const estimateKind = $derived(imported && species === 'deadline' ? 'deadline' : traceTask ? 'task' : null);
  const currentWork = $derived.by(() => {
    if (!local) return null;
    if (estimateKind === 'deadline') {
      const note = local.deadline_annotations.find(a => a.target_id === id);
      return note?.work.kind === 'standalone' ? note.work.value : null;
    }
    if (estimateKind === 'task') return local.task_annotations.find(a => a.target_id === id)?.work ?? null;
    return null;
  });
  let effortText = $state('');
  let chunkText = $state('');
  $effect(() => {
    const work = currentWork;
    effortText = work && work.effort.kind !== 'unknown' ? String(work.effort.value) : '';
    chunkText = work?.minimum_chunk_minutes ? String(work.minimum_chunk_minutes) : '';
  });
  async function saveEstimate(event: SubmitEvent) {
    event.preventDefault();
    const effort = String(effortText).trim() === '' ? undefined : Math.round(Number(effortText));
    let chunk = String(chunkText).trim() === '' ? undefined : Math.round(Number(chunkText));
    if (effort !== undefined && chunk === undefined) chunk = Math.max(1, Math.min(30, effort));
    const base = { action: 'upsert' as const, replace_work: true, effort_minutes: effort, minimum_chunk_minutes: chunk, zone };
    await onapply('Estimate saved', [estimateKind === 'deadline'
      ? { ...base, kind: 'deadline_annotation', id }
      : { ...base, kind: 'task_annotation', trace_external_id: traceTask!.external_id }]);
  }
  const deadlineState = $derived(localDeadline?.fulfillment.value.kind ?? null);
</script>

<div class="item-detail">
  <div class="inspector-heading">
    <span class="detail-kind">
      {#if color}<span class="swatch" style:background={color}></span>{:else}<span class="species-shape {species}"></span>{/if}{kindLabel}
    </span>
    <button class="close" aria-label="Close details" onclick={onclose}>×</button>
  </div>
  <h2>{title}</h2>
  <p class="detail-when">{when}</p>

  <div class="detail-actions">
    {#if editable}<button class="primary" disabled={busy} onclick={edit}>Edit</button>{/if}
    {#if series && occurrenceDate}
      <button disabled={busy} onclick={() => { confirming = 'delete-one'; }}>Skip this one</button>
      <button class="danger" disabled={busy} onclick={() => { confirming = 'delete-all'; }}>Delete series</button>
    {:else if localAnchor || intention}
      <button class="danger" disabled={busy} onclick={() => { confirming = 'delete-all'; }}>Delete</button>
    {/if}
    {#if localDeadline}
      {#if deadlineState === 'unresolved'}
        <button disabled={busy} onclick={() => onapply('Marked done', [{ action: 'upsert', kind: 'deadline', id, completion: 'satisfied' }])}>Mark done</button>
      {:else}
        <button disabled={busy} onclick={() => onapply('Reopened', [{ action: 'upsert', kind: 'deadline', id, completion: 'unresolved' }])}>Reopen</button>
      {/if}
      <button class="danger" disabled={busy} onclick={() => { confirming = 'delete-all'; }}>Delete</button>
    {/if}
    {#if intention}
      {#if intention.state === 'active'}
        <button disabled={busy} onclick={() => onapply('Marked done', [{ action: 'upsert', kind: 'intention', id, completion: 'done' }])}>Done</button>
        <button disabled={busy} onclick={() => onapply('Dismissed', [{ action: 'upsert', kind: 'intention', id, completion: 'dismissed' }])}>Dismiss</button>
      {:else}
        <button disabled={busy} onclick={() => onapply('Reopened', [{ action: 'upsert', kind: 'intention', id, completion: 'unresolved' }])}>Reopen</button>
      {/if}
    {/if}
    {#if imported && species === 'deadline'}
      {#if handled}
        <button disabled={busy} onclick={() => onapply('No longer marked handled', [{ action: 'remove', kind: 'imported_handled', id }])}>Undo handled</button>
      {:else}
        <button disabled={busy} onclick={() => onapply('Marked handled', [{ action: 'upsert', kind: 'imported_handled', id }])}>Mark handled</button>
      {/if}
    {/if}
    {#if routineId && routineDate}
      {#if routineOutcome}
        <button disabled={busy} onclick={() => onapply('Cleared', [{ action: 'remove', kind: 'routine_outcome', id: routineId, occurrence_date: routineDate }])}>Clear “{routineOutcome}”</button>
      {:else}
        <button disabled={busy} onclick={() => onapply('Recorded', [{ action: 'upsert', kind: 'routine_outcome', id: routineId, occurrence_date: routineDate, outcome: 'done' }])}>Done this day</button>
        <button disabled={busy} onclick={() => onapply('Skipped', [{ action: 'upsert', kind: 'routine_outcome', id: routineId, occurrence_date: routineDate, outcome: 'skipped' }])}>Skip this day</button>
      {/if}
    {/if}
  </div>

  {#if confirming}
    <div class="confirm" role="alert">
      <p>{confirming === 'delete-one' ? 'Skip only this occurrence? The rest of the series stays.' : series ? 'Delete every occurrence of this series?' : 'Delete this from your local time?'}</p>
      <div>
        <button class="danger" disabled={busy} onclick={() => remove(confirming === 'delete-one' ? 'one' : 'all')}>{confirming === 'delete-one' ? 'Skip it' : 'Delete'}</button>
        <button disabled={busy} onclick={() => { confirming = null; }}>Cancel</button>
      </div>
    </div>
  {/if}

  <div class="source-box">
    <strong>{imported ? `From ${sourceLabel}` : species === 'task' ? 'From Trace' : 'Your local time'}</strong>
    <span>{imported ? 'Read-only here. Change it in its own calendar.' : species === 'task' ? 'Trace owns its text, dates, and completion.' : series ? 'Part of a repeating event.' : 'Stored only on this computer.'}</span>
    {#if handled}<span>Marked handled by you. The source still lists it.</span>{/if}
    {#if item?.milestone}<span>◇ Marked as a milestone</span>{/if}
  </div>

  {#if estimateKind}
    <form class="estimate" onsubmit={saveEstimate}>
      <strong>Work estimate</strong>
      <div class="estimate-fields">
        <label>Work left (min)<input type="number" min="0" step="5" bind:value={effortText} placeholder="Unknown" /></label>
        <label>Smallest session (min)<input type="number" min="1" step="5" bind:value={chunkText} placeholder="Unknown" /></label>
        <button type="submit" disabled={busy}>Save</button>
      </div>
      <p class="small-note">Kept on this computer. {estimateKind === 'deadline' ? 'The deadline from its source is unchanged.' : 'Trace keeps the task itself.'} Blank stays unknown.</p>
    </form>
  {/if}

  {#if place || notes || event?.tentative || event?.occupancy === 'transparent'}
    <dl class="state-details">
      {#if place}<div><dt>Place</dt><dd>{place}</dd></div>{/if}
      {#if event?.tentative}<div><dt>Certainty</dt><dd>Tentative</dd></div>{/if}
      {#if event?.occupancy === 'transparent'}<div><dt>Shows as</dt><dd>Free (does not block time)</dd></div>{/if}
      {#if notes}<div><dt>Notes</dt><dd class="notes">{notes}</dd></div>{/if}
    </dl>
  {/if}

  {#if item}
    <dl class="state-details">
      <div><dt>State</dt><dd>{readable(item.phase)}</dd></div>
      {#if item.risk}<div><dt>Pressure</dt><dd class:risk={['tight', 'insufficient', 'overdue'].includes(item.risk)}>{readable(item.risk)}</dd></div>{/if}
      {#if item.conditional}<div><dt>Source basis</dt><dd>Conditional — rests on last-known source data</dd></div>{/if}
    </dl>
    <details class="evidence" open={species === 'deadline' || species === 'task' || species === 'intention'}>
      <summary>Why it appears</summary>
      <ul class="reasons">{#each item.reasons as reason}<li>{reason}</li>{/each}</ul>
      {#if item.facts.length}<h3>Evidence and calculation</h3><dl class="facts">{#each item.facts as fact}<div><dt>{fact.label}</dt><dd>{fact.value}</dd></div>{/each}</dl>{/if}
    </details>
  {:else if event}
    <dl class="state-details"><div><dt>State</dt><dd>{readable(event.state)}</dd></div></dl>
    <p class="small-note">Outside the next two weeks, the Horizon does not evaluate pressure for this item.</p>
  {/if}
</div>

<style>
  .item-detail { display: grid; align-content: start; }
  .swatch { width: 10px; height: 10px; display: inline-block; border: 1px solid var(--line-strong); }
  .confirm { border: 1px solid var(--risk); padding: 10px 12px; margin-bottom: 14px; font-size: 13px; display: grid; gap: 8px; }
  .confirm p { font-family: var(--font-mono); font-size: 11.5px; }
  .confirm div { display: flex; gap: 6px; }
  .notes { white-space: pre-wrap; }
  .evidence summary { cursor: pointer; font-family: var(--font-mono); font-size: 11px; letter-spacing: .07em; text-transform: uppercase; margin: 6px 0 10px; list-style: none; }
  .evidence summary::before { content: '+ '; }
  .evidence[open] summary::before { content: '– '; }
  .estimate { display: grid; gap: 8px; padding: 12px; border: var(--rule); margin-bottom: 16px; font-size: 13px; }
  .estimate strong { font-family: var(--font-mono); font-size: 11px; letter-spacing: .07em; text-transform: uppercase; }
  .estimate-fields { display: flex; gap: 8px; align-items: end; flex-wrap: wrap; }
  .estimate-fields label { display: grid; gap: 3px; font-family: var(--font-mono); font-size: 10px; letter-spacing: .05em; text-transform: uppercase; color: var(--muted); flex: 1 1 100px; }
  .estimate-fields input { width: 100%; }
</style>
