<script lang="ts">
  import { onMount, tick } from 'svelte';
  import type { Weekday } from '../lib/calendar-types.ts';
  import { formatDate, weekdayOf } from '../lib/calendar.ts';
  import { deleteMutations, describeRepeat, minutesOf, hhmm, saveMutations, validateDraft, type EditTarget, type EventDraft, type Scope } from '../lib/editor.ts';
  import type { LocalMutation } from '../lib/local.ts';

  let { target, initial, busy, onsave, onclose }: {
    target: EditTarget;
    initial: EventDraft;
    busy: boolean;
    onsave: (label: string, mutations: LocalMutation[]) => Promise<boolean>;
    onclose: () => void;
  } = $props();

  // The draft is a private copy taken once; the editor is re-created (keyed) for each
  // edit, and the stored record changes only on Save.
  // svelte-ignore state_referenced_locally
  let draft = $state<EventDraft>(structuredClone($state.snapshot(initial)) as EventDraft);
  let scope = $state<Scope>('one');
  let error = $state('');
  let confirmDelete = $state(false);
  let dialog: HTMLDialogElement | undefined = $state();
  let titleInput: HTMLInputElement | undefined = $state();
  const isNew = $derived(target.kind === 'new');
  const isSeries = $derived(target.kind === 'series');
  const days: { day: Weekday; label: string }[] = [
    { day: 'mon', label: 'M' }, { day: 'tue', label: 'T' }, { day: 'wed', label: 'W' }, { day: 'thu', label: 'T' },
    { day: 'fri', label: 'F' }, { day: 'sat', label: 'S' }, { day: 'sun', label: 'S' },
  ];
  const dayNames: Record<Weekday, string> = { mon: 'Monday', tue: 'Tuesday', wed: 'Wednesday', thu: 'Thursday', fri: 'Friday', sat: 'Saturday', sun: 'Sunday' };
  const heading = $derived(`${isNew ? 'New' : 'Edit'} ${draft.kind === 'deadline' ? 'deadline' : draft.kind === 'intention' ? 'optional item' : 'event'}`);
  const durationLabel = $derived.by(() => {
    if (draft.allDay) return '';
    let minutes = minutesOf(draft.end) - minutesOf(draft.start);
    if (minutes <= 0) minutes += 1440;
    const h = Math.floor(minutes / 60), m = minutes % 60;
    return `${h ? `${h} h` : ''}${h && m ? ' ' : ''}${m ? `${m} min` : ''}${minutesOf(draft.end) <= minutesOf(draft.start) ? ' · ends next day' : ''}`;
  });

  onMount(() => {
    dialog?.showModal();
    void tick().then(() => titleInput?.focus());
  });

  function setStart(value: string) {
    if (!value) return;
    const length = (minutesOf(draft.end) - minutesOf(draft.start) + 1440) % 1440 || 60;
    draft.start = value;
    draft.end = hhmm(minutesOf(value) + length);
  }
  function setDate(value: string) {
    if (!value) return;
    const span = Math.max(0, Math.round((Date.parse(`${draft.endDate}T00:00:00Z`) - Date.parse(`${draft.date}T00:00:00Z`)) / 86_400_000));
    draft.date = value;
    draft.endDate = new Date(Date.parse(`${value}T00:00:00Z`) + span * 86_400_000).toISOString().slice(0, 10);
    if (draft.repeat !== 'custom') draft.weekdays = [weekdayOf(value)];
  }
  function toggleDay(day: Weekday) {
    draft.weekdays = draft.weekdays.includes(day) ? draft.weekdays.filter(d => d !== day) : [...draft.weekdays, day];
  }
  function numberOrNull(value: string): number | null {
    if (value.trim() === '') return null;
    const n = Number(value);
    return Number.isFinite(n) ? Math.round(n) : null;
  }
  function setEffort(value: string) {
    draft.effort = numberOrNull(value);
    // A work estimate needs a useful session to be assessable; suggest one the person can change.
    if (draft.effort !== null && draft.chunk === null) draft.chunk = Math.min(30, Math.max(1, draft.effort));
  }

  async function save() {
    error = validateDraft(draft) ?? '';
    if (error) return;
    const mutations = saveMutations(draft, target, scope);
    const label = isNew ? `Added “${draft.title.trim()}”` : 'Saved';
    if (await onsave(label, mutations)) close();
  }
  async function remove(which: Scope) {
    if (await onsave(which === 'one' && isSeries ? 'Occurrence removed' : 'Deleted', deleteMutations(target, which))) close();
  }
  function close() { dialog?.close(); onclose(); }
  function keydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && (event.ctrlKey || event.metaKey)) { event.preventDefault(); void save(); }
  }
</script>

<dialog bind:this={dialog} class="editor" aria-labelledby="editor-title" oncancel={(e) => { e.preventDefault(); close(); }} onkeydown={keydown}>
  <form method="dialog" onsubmit={(e) => { e.preventDefault(); void save(); }}>
    <header>
      <h2 id="editor-title">{heading}</h2>
      {#if isNew}
        <div class="segmented" role="group" aria-label="Kind">
          <button type="button" aria-pressed={draft.kind === 'event'} onclick={() => { draft.kind = 'event'; }}>Event</button>
          <button type="button" aria-pressed={draft.kind === 'deadline'} onclick={() => { draft.kind = 'deadline'; draft.repeat = 'none'; }}>Deadline</button>
          <button type="button" aria-pressed={draft.kind === 'intention'} onclick={() => { draft.kind = 'intention'; draft.repeat = 'none'; }}>Optional</button>
        </div>
      {/if}
      <button type="button" class="close" aria-label="Close" onclick={close}>×</button>
    </header>

    <fieldset disabled={busy}>
      <label class="title-field">
        <span class="sr-only">Title</span>
        <input bind:this={titleInput} bind:value={draft.title} placeholder={draft.kind === 'deadline' ? 'What is due?' : draft.kind === 'intention' ? 'What would you like to do?' : 'Add a title'} maxlength="200" required />
      </label>
      <p class="kind-hint">
        {#if draft.kind === 'event'}A fixed event: it happens at a time and blocks that time.
        {:else if draft.kind === 'deadline'}A real cutoff. It can become overdue; Temporal tracks the pressure before it.
        {:else}Soft and optional. A preferred day that passes creates no debt.{/if}
      </p>

      {#if draft.kind === 'intention'}
        <label class="check"><input type="checkbox" bind:checked={draft.noDate} /> No particular day</label>
      {/if}

      {#if !(draft.kind === 'intention' && draft.noDate)}
        <div class="row">
          <label>{draft.kind === 'deadline' ? 'Due date' : draft.allDay && draft.kind === 'event' ? 'First day' : 'Date'}
            <input type="date" value={draft.date} onchange={(e) => setDate(e.currentTarget.value)} required />
          </label>
          {#if draft.kind === 'deadline'}
            {#if !draft.allDay}<label>Due time<input type="time" bind:value={draft.start} required /></label>{/if}
          {:else if !draft.allDay}
            <label>Start<input type="time" value={draft.start} onchange={(e) => setStart(e.currentTarget.value)} required /></label>
            <label>End<input type="time" bind:value={draft.end} required /></label>
          {:else if draft.kind === 'event'}
            <label>Last day<input type="date" bind:value={draft.endDate} min={draft.date} required /></label>
          {/if}
        </div>
        <div class="row tight">
          <label class="check"><input type="checkbox" bind:checked={draft.allDay} /> {draft.kind === 'deadline' ? 'Due by the end of the day' : 'All day'}</label>
          {#if durationLabel && draft.kind !== 'deadline'}<span class="muted">{durationLabel}</span>{/if}
        </div>
      {/if}

      {#if draft.kind === 'event'}
        <label>Repeat
          <select bind:value={draft.repeat} onchange={() => { if (draft.repeat === 'custom' && !draft.weekdays.length) draft.weekdays = [weekdayOf(draft.date)]; }}>
            <option value="none">Does not repeat</option>
            <option value="daily">Every day</option>
            <option value="weekly">Every week on {dayNames[weekdayOf(draft.date)]}</option>
            <option value="weekdays">Every weekday (Monday to Friday)</option>
            <option value="monthly">Every month on day {Number(draft.date.slice(8, 10))}</option>
            <option value="custom">Custom…</option>
          </select>
        </label>
        {#if draft.repeat === 'custom'}
          <div class="custom">
            <div class="row tight">
              <span>Every</span>
              <input class="narrow" type="number" min="1" max="99" bind:value={draft.interval} aria-label="Interval" />
              <select bind:value={draft.unit} aria-label="Period">
                <option value="day">{draft.interval === 1 ? 'day' : 'days'}</option>
                <option value="week">{draft.interval === 1 ? 'week' : 'weeks'}</option>
                <option value="month">{draft.interval === 1 ? 'month' : 'months'}</option>
              </select>
            </div>
            {#if draft.unit === 'week'}
              <div class="days" role="group" aria-label="Weekdays">
                {#each days as d}<button type="button" aria-pressed={draft.weekdays.includes(d.day)} aria-label={dayNames[d.day]} onclick={() => toggleDay(d.day)}>{d.label}</button>{/each}
              </div>
            {/if}
          </div>
        {/if}
        {#if draft.repeat !== 'none'}
          <div class="row tight ends">
            <span>Ends</span>
            <label class="check"><input type="radio" name="ends" value="never" bind:group={draft.ends} /> Never</label>
            <label class="check"><input type="radio" name="ends" value="on" bind:group={draft.ends} /> On</label>
            {#if draft.ends === 'on'}<input type="date" bind:value={draft.until} min={draft.date} aria-label="Last date" />{/if}
            <label class="check"><input type="radio" name="ends" value="after" bind:group={draft.ends} /> After</label>
            {#if draft.ends === 'after'}<input class="narrow" type="number" min="1" max="5000" bind:value={draft.count} aria-label="Number of times" /><span>times</span>{/if}
          </div>
          <p class="muted">{describeRepeat(draft)}{isSeries && scope === 'all' ? ' · the series keeps its first date' : ''}</p>
        {/if}
      {/if}

      {#if draft.kind !== 'intention'}
        <label>Place<input bind:value={draft.place} placeholder="Optional" maxlength="200" /></label>
      {/if}

      {#if draft.kind === 'event'}
        <div class="row tight">
          <div class="segmented small" role="group" aria-label="Shows as">
            <button type="button" aria-pressed={draft.busy} onclick={() => { draft.busy = true; }}>Busy</button>
            <button type="button" aria-pressed={!draft.busy} onclick={() => { draft.busy = false; }}>Free</button>
          </div>
          <label class="check"><input type="checkbox" bind:checked={draft.tentative} /> Tentative</label>
        </div>
      {/if}

      {#if draft.kind !== 'event'}
        <div class="row">
          <label>Work left (minutes)<input type="number" min="0" step="5" value={draft.effort ?? ''} oninput={(e) => setEffort(e.currentTarget.value)} placeholder="Unknown" /></label>
          <label>Smallest useful session (minutes)<input type="number" min="1" step="5" value={draft.chunk ?? ''} oninput={(e) => { draft.chunk = numberOrNull(e.currentTarget.value); }} placeholder="Unknown" /></label>
        </div>
        <p class="muted">{draft.kind === 'deadline' ? 'With an estimate, Temporal can tell when this is getting tight against the time you usually have.' : 'With an estimate, Temporal can suggest it when it fits your day.'} Blank stays unknown.</p>
      {/if}

      {#if draft.kind !== 'intention'}
        <label>Notes<textarea rows="2" bind:value={draft.notes} maxlength="4000" placeholder="Optional"></textarea></label>
      {/if}

      {#if isSeries}
        <div class="scope" role="radiogroup" aria-label="Apply changes to">
          <span>Apply changes to</span>
          <label class="check"><input type="radio" name="scope" value="one" bind:group={scope} /> Only {formatDate((target as { occurrenceDate: string }).occurrenceDate, 'long')}</label>
          <label class="check"><input type="radio" name="scope" value="all" bind:group={scope} /> Every occurrence</label>
        </div>
      {/if}

      {#if error}<p class="error" role="alert">{error}</p>{/if}
    </fieldset>

    <footer>
      {#if !isNew}
        {#if confirmDelete}
          <span class="confirm">
            {#if isSeries}
              <button type="button" class="danger" disabled={busy} onclick={() => remove('one')}>Delete this one</button>
              <button type="button" class="danger" disabled={busy} onclick={() => remove('all')}>Delete all</button>
            {:else}
              <button type="button" class="danger" disabled={busy} onclick={() => remove('all')}>Confirm delete</button>
            {/if}
            <button type="button" class="ghost" onclick={() => { confirmDelete = false; }}>Keep</button>
          </span>
        {:else}
          <button type="button" class="danger" disabled={busy} onclick={() => { confirmDelete = true; }}>Delete…</button>
        {/if}
      {/if}
      <span class="spacer"></span>
      <span class="hint">Ctrl+Enter saves</span>
      <button type="button" onclick={close}>Cancel</button>
      <button type="submit" class="primary" disabled={busy}>Save</button>
    </footer>
  </form>
</dialog>

<style>
  .editor { width: min(560px, calc(100vw - 32px)); max-height: calc(100vh - 48px); padding: 0; border: 1px solid var(--line-strong); border-radius: 12px; background: var(--surface); color: var(--ink); box-shadow: var(--shadow-lg); }
  .editor::backdrop { background: rgba(10, 20, 30, .35); }
  form { display: flex; flex-direction: column; max-height: calc(100vh - 50px); }
  header { display: flex; align-items: center; gap: 12px; padding: 16px 18px 12px; border-bottom: 1px solid var(--line); }
  header h2 { font-size: 16px; margin-right: auto; }
  .close { font-size: 20px; border: 0; background: transparent; padding: 0 6px; }
  fieldset { border: 0; margin: 0; padding: 14px 18px; display: grid; gap: 12px; overflow-y: auto; min-width: 0; }
  label { display: grid; gap: 4px; font-size: 12px; color: var(--muted); min-width: 0; }
  label input, label select, label textarea { color: var(--ink); font-size: 13px; }
  .title-field input { font-size: 18px; font-weight: 600; border: 0; border-bottom: 2px solid var(--line-strong); border-radius: 0; padding: 4px 2px 6px; background: transparent; }
  .title-field input:focus-visible { outline: none; border-bottom-color: var(--accent); }
  .kind-hint { font-size: 12px; color: var(--faint); margin-top: -6px; }
  .row { display: flex; flex-wrap: wrap; gap: 12px; align-items: end; }
  .row > label { flex: 1 1 140px; }
  .row.tight { gap: 10px; align-items: center; font-size: 13px; }
  .check { display: flex; flex-direction: row; align-items: center; gap: 7px; font-size: 13px; color: var(--ink); }
  .muted { font-size: 12px; color: var(--muted); }
  .narrow { width: 72px; }
  .custom { display: grid; gap: 10px; padding: 10px 12px; background: var(--surface-2); border-radius: var(--radius-sm); }
  .days { display: flex; gap: 6px; }
  .days button { width: 32px; height: 32px; padding: 0; border-radius: 50%; font-size: 12px; }
  .days button[aria-pressed='true'], .segmented button[aria-pressed='true'] { background: var(--accent); color: var(--accent-ink); border-color: var(--accent); }
  .segmented { display: inline-flex; border: 1px solid var(--line-strong); border-radius: var(--radius-sm); overflow: hidden; }
  .segmented button { border: 0; border-radius: 0; padding: 5px 12px; font-size: 12px; background: var(--surface); }
  .segmented button + button { border-left: 1px solid var(--line-strong); }
  .ends > label { flex: 0 0 auto; }
  .ends input { width: auto; }
  .scope { display: flex; flex-wrap: wrap; gap: 14px; align-items: center; padding: 10px 12px; border: 1px solid var(--line); border-radius: var(--radius-sm); font-size: 13px; }
  .scope > span { font-weight: 600; }
  .error { color: var(--risk); font-size: 13px; }
  footer { display: flex; align-items: center; gap: 8px; padding: 12px 18px; border-top: 1px solid var(--line); background: var(--surface-2); border-radius: 0 0 12px 12px; }
  .spacer { flex: 1; }
  .hint { font-size: 11px; color: var(--faint); }
  .confirm { display: flex; gap: 6px; }
  .sr-only { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
  @media (max-width: 600px) { .hint { display: none; } }
</style>
