<script lang="ts">
  import { tick } from 'svelte';
  import { emptyForm, editForm, formMutation, wallTime, weekdays, type LocalKind, type LocalState, type LocalMutation, type TraceChoice } from './lib/local.ts';

  let { local, tasks, now, zone, busy, onsave }: {
    local: LocalState; tasks: TraceChoice[]; now: number; zone: string; busy: boolean;
    onsave: (mutation: LocalMutation) => Promise<void>;
  } = $props();
  let open = $state(false);
  let form = $state(emptyForm());
  let error = $state('');
  let message = $state('');
  let firstInput: HTMLInputElement | undefined = $state();
  let showForm = $state(false);
  let outcomeRoutine = $state('');
  let outcomeDate = $state('');
  const kinds: { kind: LocalKind; label: string }[] = [
    { kind: 'anchor', label: 'Anchor' }, { kind: 'deadline', label: 'Deadline' },
    { kind: 'intention', label: 'Intention' }, { kind: 'routine', label: 'Weekly routine' },
    { kind: 'availability', label: 'Availability' }, { kind: 'task_annotation', label: 'Trace work annotation' },
  ];
  let records = $derived([
    ...local.anchors.filter(r => r.presence === 'present').map(r => ({ kind: 'anchor' as const, id: r.meta.id, title: r.title, state: 'Fixed occurrence' })),
    ...local.deadlines.filter(r => r.presence === 'present').map(r => ({ kind: 'deadline' as const, id: r.meta.id, title: r.title, state: r.fulfillment.value.kind })),
    ...local.intentions.filter(r => r.presence === 'present').map(r => ({ kind: 'intention' as const, id: r.meta.id, title: r.title, state: r.state })),
    ...local.routines.filter(r => r.presence === 'present').map(r => ({ kind: 'routine' as const, id: r.meta.id, title: r.title, state: r.state })),
    ...local.availability.map(r => ({ kind: 'availability' as const, id: r.meta.id, title: `${wallTime(r.span.start, r.span.original_zone ?? zone).slice(0,16).replace('T', ' ')} – ${wallTime(r.span.end, r.span.original_zone ?? zone).slice(0,16).replace('T', ' ')}`, state: r.span.original_zone ?? zone })),
  ]);
  let needsWork = $derived(['intention', 'routine', 'task_annotation'].includes(form.kind) || (form.kind === 'deadline' && form.workBasis === 'standalone'));

  async function create(kind: LocalKind) {
    form = emptyForm(kind, zone); showForm = true; error = ''; message = '';
    await tick(); firstInput?.focus();
  }
  async function edit(kind: LocalKind, id: string) {
    error = ''; message = '';
    try { form = editForm(kind, id, local, tasks, zone); showForm = true; await tick(); firstInput?.focus(); }
    catch (e) { error = String(e instanceof Error ? e.message : e); }
  }
  async function apply(mutation: LocalMutation, success: string): Promise<boolean> {
    if (busy) return false;
    error = ''; message = '';
    try {
      await onsave(mutation); message = success;
      if (mutation.action === 'remove' && mutation.id === form.id) showForm = false;
      return true;
    }
    catch (e) { error = String(e instanceof Error ? e.message : e); return false; }
  }
  async function save() {
    try {
      if (await apply(formMutation(form), 'Saved on this computer.')) { showForm = false; form = emptyForm(form.kind, zone); }
    } catch (e) { error = String(e instanceof Error ? e.message : e); }
  }
  function chooseTask(id: string) {
    const task = tasks.find(t => t.external_id === id);
    if (task) form = editForm('task_annotation', task.id, local, tasks, zone);
  }
  function startOutcome(id: string) {
    outcomeRoutine = id;
    const rule = local.routines.find(r => r.meta.id === id)!.rule.value;
    outcomeDate = wallTime(now, rule.zone).slice(0, 10);
  }
</script>

<details class="local-panel" bind:open>
  <summary>Manage local time <span>{records.length} records · add, edit, and record outcomes</span></summary>
  <div class="local-toolbar" aria-label="Add local input">
    {#each kinds as choice}<button disabled={busy} onclick={() => create(choice.kind)}>+ {choice.label}</button>{/each}
  </div>
  {#if error}<p class="local-error" role="alert">{error}</p>{/if}
  {#if message}<p class="local-message" role="status">{message}</p>{/if}
  {#if showForm}
    <form class="local-form" onsubmit={(event) => { event.preventDefault(); void save(); }}>
      <h3>{form.id ? 'Edit' : 'Add'} {kinds.find(k => k.kind === form.kind)?.label}</h3>
      <fieldset disabled={busy} class="local-fields">
        {#if form.kind !== 'availability' && form.kind !== 'task_annotation'}
          <label>Title<input bind:this={firstInput} bind:value={form.title} required /></label>
        {/if}
        {#if form.kind === 'task_annotation'}
          <label>Trace task<select required value={form.traceId} onchange={(e) => chooseTask(e.currentTarget.value)}>
            <option value="" disabled>Select a retained Trace task</option>
            {#each tasks as task}<option value={task.external_id}>{task.title} · {task.presence === 'removed' ? 'removed from Trace' : task.status}</option>{/each}
          </select></label>
          <p class="form-note">Trace owns text, dates, and completion. These fields describe how its work fits your time.</p>
        {/if}
        {#if ['anchor', 'deadline', 'intention'].includes(form.kind)}
          <label>{form.kind === 'intention' ? 'Preferred time' : 'Time precision'}<select bind:value={form.precision} onchange={() => { form.start = ''; form.end = ''; form.due = ''; }}>
            {#if form.kind === 'intention'}<option value="none">No time preference</option>{/if}
            <option value="timed">Exact local time</option><option value="dates">Whole civil date{form.kind === 'deadline' ? '' : 's'}</option>
          </select></label>
        {/if}
        {#if form.kind === 'deadline'}
          <label>{form.precision === 'dates' ? 'Due date (whole day allowed)' : 'Due time'}<input type={form.precision === 'dates' ? 'date' : 'datetime-local'} step={form.precision === 'dates' ? '1' : '0.001'} bind:value={form.due} required /></label>
          <label>Associated work<select bind:value={form.workBasis}><option value="unspecified">Not described</option><option value="standalone">Local effort estimate</option><option value="task">Work from a Trace task</option></select></label>
          {#if form.workBasis === 'task'}<label>Work from Trace<select bind:value={form.traceId} required><option value="" disabled>Select a task</option>{#each tasks as task}<option value={task.external_id}>{task.title} · {task.presence === 'removed' ? 'removed from Trace' : task.status}</option>{/each}</select></label><p class="form-note">Uses that task's work annotation. Completing it in Trace still requires explicit satisfaction of this independent deadline.</p>{/if}
        {/if}
        {#if (form.kind === 'anchor' || form.kind === 'intention') && form.precision === 'dates'}
          <label>First date<input type="date" bind:value={form.startDate} required /></label>
          <label>Ends before date<input type="date" bind:value={form.endDate} required /></label>
        {:else if ((form.kind === 'anchor' || form.kind === 'intention') && form.precision === 'timed') || form.kind === 'availability'}
          <label>Start<input type="datetime-local" step="0.001" bind:value={form.start} required /></label>
          <label>End<input type="datetime-local" step="0.001" bind:value={form.end} required /></label>
        {/if}
        {#if form.kind === 'anchor'}
          <label>Availability<select bind:value={form.occupancy}><option value="busy">Blocks this time</option><option value="transparent">Does not block time</option><option value="unknown">Not known (blocks conservatively)</option></select></label>
          <label>Certainty<select bind:value={form.certainty}><option value="confirmed">Confirmed</option><option value="tentative">Tentative</option><option value="unspecified">Unspecified</option></select></label>
          <label>Place or context<input bind:value={form.context} placeholder="e.g. campus" /></label>
        {/if}
        {#if form.kind === 'routine'}
          <fieldset class="weekday-picker"><legend>Preferred weekdays</legend>{#each weekdays as day}<label><input type="checkbox" value={day} bind:group={form.days} /> {day}</label>{/each}</fieldset>
          <label>Starts on<input type="date" bind:value={form.startDate} required /></label><label>Ends before (optional)<input type="date" bind:value={form.until} /></label>
          <label class="check-label"><input type="checkbox" bind:checked={form.paused} /> Paused</label>
        {/if}
        {#if needsWork}
          <label>Remaining effort (minutes)<input type="number" min="0" max="4294967295" step="1" bind:value={form.effort} placeholder="Unknown if blank" /></label>
          <label>Smallest useful session (minutes)<input type="number" min="1" max="4294967295" step="1" bind:value={form.chunk} placeholder="Unknown if blank" /></label>
          <label>Required contexts<input bind:value={form.context} placeholder="desk, quiet" /></label>
          <label>Energy needed<select bind:value={form.energy}><option value="unrestricted">No restriction</option><option value="light">Light</option><option value="normal">Normal</option><option value="deep">Deep</option></select></label>
          <p class="form-note">Blank effort or session length stays unknown. Zero effort does not mark anything complete. Contexts use lowercase tags, separated by commas.</p>
        {/if}
        {#if form.kind === 'availability'}
          <label>Available contexts<input bind:value={form.context} placeholder="Unknown if blank; e.g. home, quiet" /></label>
          <label>Energy available<select bind:value={form.capacity}><option value="unknown">Unknown</option><option value="light">Light</option><option value="normal">Normal</option><option value="deep">Deep</option></select></label>
          <p class="form-note">Declare a bounded time you are willing to use. Declarations may touch but cannot overlap; fixed anchors are subtracted by the engine.</p>
        {/if}
        {#if ['anchor', 'deadline', 'task_annotation'].includes(form.kind)}
          <label>Importance<select bind:value={form.importance}><option value="unspecified">Unspecified</option><option value="low">Low</option><option value="normal">Normal</option><option value="high">High</option></select></label>
        {/if}
        {#if form.kind === 'anchor' || form.kind === 'deadline'}<label class="check-label"><input type="checkbox" bind:checked={form.milestone} /> Mark as a milestone</label>{/if}
        {#if form.kind !== 'task_annotation'}<label>Timezone (IANA)<input bind:value={form.zone} required placeholder="America/Toronto" /></label>{/if}
        <div class="local-form-actions"><button type="submit">Save {form.kind === 'task_annotation' ? 'annotation' : 'local entry'}</button><button type="button" onclick={() => { showForm = false; }}>Cancel</button></div>
      </fieldset>
    </form>
  {/if}
  <div class="local-records">
    {#each records as record (record.id)}
      <div class="local-row">
        <div><strong>{record.title}</strong><small>{record.kind} · {record.state}</small></div>
        <div class="local-row-actions">
          {#if record.kind === 'deadline' || record.kind === 'intention'}
            {#if record.state === 'unresolved' || record.state === 'active'}
              <button disabled={busy} onclick={() => apply({ action: 'upsert', kind: record.kind, id: record.id, completion: record.kind === 'deadline' ? 'satisfied' : 'done' }, 'Completion recorded.')}>{record.kind === 'deadline' ? 'Mark satisfied' : 'Mark done'}</button>
              <button disabled={busy} onclick={() => apply({ action: 'upsert', kind: record.kind, id: record.id, completion: record.kind === 'deadline' ? 'cancelled' : 'dismissed' }, 'Local state updated.')}>{record.kind === 'deadline' ? 'Cancel deadline' : 'Dismiss'}</button>
            {:else}<button disabled={busy} onclick={() => apply({ action: 'upsert', kind: record.kind, id: record.id, completion: 'unresolved' }, 'Local entry reopened.')}>Reopen</button>{/if}
          {/if}
          {#if record.kind === 'routine'}<button disabled={busy} onclick={() => startOutcome(record.id)}>Record a day</button>{/if}
          <button disabled={busy} onclick={() => edit(record.kind, record.id)}>Edit</button>
          <button disabled={busy} onclick={() => apply({ action: 'remove', kind: record.kind, id: record.id }, 'Removed from local time.')}>Remove</button>
        </div>
      </div>
    {/each}
    {#if !records.length}<p class="quiet">Add an anchor or deadline to begin. Intentions and routines stay flexible.</p>{/if}
    {#if outcomeRoutine}
      <div class="outcome-form"><strong>{local.routines.find(r => r.meta.id === outcomeRoutine)?.title}</strong><label>Occurrence date<input type="date" bind:value={outcomeDate} required /></label>
        {#each ['done', 'skipped'] as outcome}<button disabled={busy || !outcomeDate} onclick={() => apply({ action: 'upsert', kind: 'routine_outcome', id: outcomeRoutine, occurrence_date: outcomeDate, outcome: outcome as 'done' | 'skipped' }, 'Day recorded.')}>{outcome === 'done' ? 'Done on this date' : 'Skip this date'}</button>{/each}
        <button disabled={busy || !local.routine_outcomes.some(r => r.routine_id === outcomeRoutine && r.date === outcomeDate)} onclick={() => apply({ action: 'remove', kind: 'routine_outcome', id: outcomeRoutine, occurrence_date: outcomeDate }, 'Outcome cleared.')}>Clear outcome</button>
        <button onclick={() => { outcomeRoutine = ''; }}>Close</button>
        <p class="form-note">{local.routine_outcomes.find(r => r.routine_id === outcomeRoutine && r.date === outcomeDate)?.outcome ?? 'No outcome recorded'}. Passing a preferred day creates no overdue obligation.</p>
      </div>
    {/if}
    {#if tasks.length}<details class="annotation-list"><summary>Trace work annotations</summary>
      {#each tasks as task}<div class="local-row"><div><strong>{task.title}</strong><small>Trace · {task.presence === 'removed' ? 'removed' : task.status} · {local.task_annotations.some(r => r.target_id === task.id) ? 'Work annotated' : 'No work annotation'}</small></div><div class="local-row-actions"><button disabled={busy} onclick={() => edit('task_annotation', task.id)}>Edit work</button>{#if local.task_annotations.some(r => r.target_id === task.id)}<button disabled={busy} onclick={() => apply({ action: 'remove', kind: 'task_annotation', trace_external_id: task.external_id }, 'Annotation removed; Trace unchanged.')}>Clear annotation</button>{/if}</div></div>{/each}
    </details>{/if}
  </div>
</details>
