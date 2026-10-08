<script lang="ts">
  import type { CalendarKind, CalendarMode, CalendarSummary, ImportResult, PersonalView } from '../lib/api.ts';
  import {
    addCalendarFile, addCalendarUrl, desktop, forgetTraceExport, importTrace, mutateBatch, pickTraceExport,
    readTextFile, refreshCalendarFile, refreshCalendars, reimportTrace, removeCalendar, updateCalendar,
  } from '../lib/api.ts';
  import type { LocalMutation } from '../lib/local.ts';

  let { personal, busy, run }: {
    personal: PersonalView;
    busy: boolean;
    /** Runs one operation with the app's busy/error handling and applies a returned view. Resolves false if it failed or was refused. */
    run: (label: string, task: () => Promise<PersonalView | ImportResult | null | void>) => Promise<boolean>;
  } = $props();

  const CALENDAR_LIMIT = 20 * 1024 * 1024;
  const TRACE_LIMIT = 10 * 1024 * 1024;
  const MARK_BATCH = 16;

  /** Calendar colours. A calendar's own colour is the one place a literal hex is used. */
  const palette = [
    { hex: '#286580', name: 'Blue' },
    { hex: '#3d7a5c', name: 'Green' },
    { hex: '#8a5a2b', name: 'Brown' },
    { hex: '#7a4a7a', name: 'Plum' },
    { hex: '#5a6b8a', name: 'Slate' },
    { hex: '#9a4e38', name: 'Rust' },
    { hex: '#4f7b7b', name: 'Teal' },
    { hex: '#6b6b3d', name: 'Olive' },
  ];
  const kindLabels: Record<CalendarKind, string> = {
    google: 'Google Calendar', outlook: 'Outlook', quercus: 'Quercus', calendar: 'Calendar',
  };
  const kindOptions: { value: CalendarKind; label: string }[] = [
    { value: 'google', label: 'Google Calendar' },
    { value: 'outlook', label: 'Outlook' },
    { value: 'quercus', label: 'Quercus / Canvas' },
    { value: 'calendar', label: 'Other calendar' },
  ];
  const linkHelp: Record<CalendarKind, string> = {
    google: 'Google Calendar → Settings → your calendar → Integrate calendar → Secret address in iCal format.',
    outlook: 'Outlook on the web → Settings → Calendar → Shared calendars → Publish a calendar → ICS link.',
    quercus: 'Quercus → Calendar → Calendar Feed (bottom right) → copy the link.',
    calendar: 'Any https or webcal iCalendar link.',
  };

  const readable = (value: string) => value.replaceAll('_', ' ');
  const plural = (n: number, one: string, many: string) => (n === 1 ? one : many);

  function healthPill(c: CalendarSummary): { text: string; tone: 'ok' | 'risk' | 'neutral' } {
    switch (c.health) {
      case 'healthy': return { text: 'Up to date', tone: 'ok' };
      case 'stale': return { text: 'Stale — showing last-known events', tone: 'neutral' };
      case 'unavailable':
      case 'partial':
      case 'incompatible': return { text: 'Last refresh failed — showing last-known events', tone: 'risk' };
      case 'never_loaded': return { text: 'Not loaded', tone: 'neutral' };
      default: return { text: readable(c.health), tone: 'neutral' };
    }
  }

  // Calendar row state: at most one inline editor and one pending removal at a time.
  let editingId: string | null = $state(null);
  let editLabel = $state('');
  let editColor = $state(palette[0].hex);
  let editMode: CalendarMode = $state('events');
  let editShown = $state(true);
  let removingId: string | null = $state(null);

  // The shared hidden picker for replacing a file calendar.
  let refreshTargetId: string | null = $state(null);
  let refreshInput: HTMLInputElement | null = $state(null);

  // Add-a-calendar form state.
  let addMethod: 'link' | 'file' = $state(desktop ? 'link' : 'file');
  let addName = $state('');
  let addKind: CalendarKind = $state('google');
  let addMode: CalendarMode = $state('events');
  let addColor = $state(palette[0].hex);
  let addUrl = $state('');
  let addFile: File | null = $state(null);
  let addFileInput: HTMLInputElement | null = $state(null);
  const nameOk = $derived(addName.trim().length > 0 && addName.trim().length <= 80);
  const addReady = $derived(nameOk && (addMethod === 'link' ? desktop && addUrl.trim() !== '' : addFile !== null));

  // Trace state.
  let traceInput: HTMLInputElement | null = $state(null);

  function toggleEdit(c: CalendarSummary) {
    if (editingId === c.id) {
      editingId = null;
      return;
    }
    editingId = c.id;
    editLabel = c.label;
    editColor = c.color;
    editMode = c.mode;
    editShown = !c.hidden;
    removingId = null;
  }

  async function saveEdit(event: SubmitEvent, c: CalendarSummary) {
    event.preventDefault();
    const label = editLabel.trim();
    if (!label) return;
    const ok = await run('Calendar updated', () => updateCalendar({ id: c.id, label, color: editColor, mode: editMode, hidden: !editShown }));
    if (ok) editingId = null;
  }

  async function remove(c: CalendarSummary) {
    const ok = await run('Calendar removed', () => removeCalendar(c.id));
    if (ok) {
      removingId = null;
      if (editingId === c.id) editingId = null;
    }
  }

  function refreshFile(c: CalendarSummary) {
    refreshTargetId = c.id;
    refreshInput?.click();
  }

  async function onRefreshPicked(input: HTMLInputElement) {
    const file = input.files?.[0];
    const id = refreshTargetId;
    input.value = '';
    refreshTargetId = null;
    if (!file || !id) return;
    await run('Calendar replaced', async () => refreshCalendarFile(id, await readTextFile(file, CALENDAR_LIMIT, 'Calendar file')));
  }

  function markAllHandled(ids: string[]) {
    return run('Marked handled', async () => {
      let view: PersonalView | null = null;
      for (let start = 0; start < ids.length; start += MARK_BATCH) {
        const batch = ids.slice(start, start + MARK_BATCH).map((id): LocalMutation => ({ action: 'upsert', kind: 'imported_handled', id }));
        view = await mutateBatch(batch);
      }
      return view;
    });
  }

  function chooseKind(next: CalendarKind) {
    addKind = next;
    addMode = next === 'quercus' ? 'coursework' : 'events';
  }

  function chooseMethod(next: 'link' | 'file') {
    addMethod = next;
    addFile = null;
    if (addFileInput) addFileInput.value = '';
  }

  function resetAdd() {
    addName = '';
    addKind = 'google';
    addMode = 'events';
    addColor = palette[0].hex;
    addUrl = '';
    addFile = null;
    if (addFileInput) addFileInput.value = '';
  }

  async function submitAdd(event: SubmitEvent) {
    event.preventDefault();
    if (!addReady) return;
    const base = { label: addName.trim(), kind: addKind, mode: addMode, color: addColor };
    const file = addFile;
    let ok = false;
    if (addMethod === 'link') {
      const url = addUrl.trim();
      ok = await run('Calendar added', () => addCalendarUrl({ ...base, url }));
    } else if (file) {
      ok = await run('Calendar added', async () => addCalendarFile({ ...base, file_name: file.name, text: await readTextFile(file, CALENDAR_LIMIT, 'Calendar file') }));
    }
    if (ok) resetAdd();
  }

  async function onTraceJson(input: HTMLInputElement) {
    const file = input.files?.[0];
    input.value = '';
    if (!file) return;
    await run('Trace imported', async () => importTrace(await readTextFile(file, TRACE_LIMIT, 'Trace export')));
  }
</script>

{#snippet colourPicker(value: string, pick: (hex: string) => void, disabled: boolean)}
  <fieldset class="colours">
    <legend>Colour</legend>
    <div class="swatches">
      {#each palette as p (p.hex)}
        <button
          type="button"
          class="swatch-btn"
          style:background={p.hex}
          aria-label={p.name}
          aria-pressed={value === p.hex}
          {disabled}
          onclick={() => pick(p.hex)}
        ></button>
      {/each}
      <label class="custom">
        Custom
        <input type="color" {value} {disabled} oninput={(event) => pick(event.currentTarget.value)} />
      </label>
    </div>
  </fieldset>
{/snippet}

<div class="sources">
  <header class="page-head">
    <h1>Sources</h1>
    <p class="intro">Temporal reads these and never writes back. Each keeps its own freshness.</p>
  </header>

  <section class="card" aria-labelledby="sources-calendars">
    <h2 id="sources-calendars">Calendars</h2>

    {#if personal.calendars.length === 0}
      <p class="empty">No calendars yet. Add Google, Outlook, or your Quercus course calendar below.</p>
    {:else}
      <ul class="calendar-list">
        {#each personal.calendars as c (c.id)}
          {@const pill = healthPill(c)}
          {@const unhandled = c.past_unhandled.length}
          <li class="calendar">
            <div class="calendar-main">
              <span class="swatch" style:background={c.color} aria-hidden="true"></span>
              <div class="calendar-body">
                <div class="calendar-title">
                  <strong>{c.label}</strong>
                  <span class="badge">{kindLabels[c.kind]}</span>
                  <span class="pill {pill.tone}">{pill.text}</span>
                </div>
                <p class="meta">{c.mode === 'coursework' ? 'Coursework: assignments become deadlines' : 'Events'}</p>
                <p class="meta">{c.origin === 'file' ? 'File' : 'Subscribed'} · {c.origin_label}</p>
                <p class="meta">
                  {#if c.last_success}Updated {c.last_success} · {/if}{c.event_count} {plural(c.event_count, 'entry', 'entries')}{#if c.skipped > 0} · {c.skipped} unreadable {plural(c.skipped, 'entry', 'entries')} skipped{/if}
                </p>
                {#if c.last_error}<p class="error">{c.last_error}</p>{/if}
                {#if c.hidden}<p class="meta faint">Hidden from Horizon and Calendar</p>{/if}
              </div>
            </div>

            <div class="actions">
              {#if c.origin === 'file'}
                <button class="btn" type="button" disabled={busy} aria-label="Refresh {c.label}" onclick={() => refreshFile(c)}>Refresh</button>
              {:else}
                <button class="btn" type="button" disabled={busy || !desktop} aria-label="Refresh {c.label}" onclick={() => run('Calendar refreshed', () => refreshCalendars(c.id))}>Refresh</button>
                {#if !desktop}<span class="desk-note">Available in the desktop app</span>{/if}
              {/if}
              <button class="btn" type="button" disabled={busy} aria-label="Edit {c.label}" aria-expanded={editingId === c.id} onclick={() => toggleEdit(c)}>Edit</button>
              <button class="btn" type="button" disabled={busy} aria-label="Remove {c.label}" onclick={() => { removingId = c.id; }}>Remove</button>
            </div>

            {#if removingId === c.id}
              <div class="confirm">
                <p>Remove {c.label}? Its events disappear from Temporal; the source is not changed.</p>
                <div class="actions">
                  <button class="btn danger" type="button" disabled={busy} onclick={() => remove(c)}>Remove</button>
                  <button class="btn" type="button" disabled={busy} onclick={() => { removingId = null; }}>Cancel</button>
                </div>
              </div>
            {/if}

            {#if editingId === c.id}
              <form class="editor" onsubmit={(event) => saveEdit(event, c)}>
                <label class="field">
                  <span>Name</span>
                  <input type="text" required maxlength="80" bind:value={editLabel} disabled={busy} />
                </label>
                {@render colourPicker(editColor, (hex) => { editColor = hex; }, busy)}
                <div class="grid-2">
                  <label class="field">
                    <span>Mode</span>
                    <select bind:value={editMode} disabled={busy}>
                      <option value="events">Events</option>
                      <option value="coursework">Coursework</option>
                    </select>
                  </label>
                  <label class="check">
                    <input type="checkbox" bind:checked={editShown} disabled={busy} />
                    Show in Horizon and Calendar
                  </label>
                </div>
                <div class="actions">
                  <button class="btn primary" type="submit" disabled={busy || editLabel.trim() === ''}>Save</button>
                  <button class="btn" type="button" disabled={busy} onclick={() => { editingId = null; }}>Cancel</button>
                </div>
              </form>
            {/if}

            {#if unhandled > 0}
              <div class="notice">
                <p>{unhandled} past {plural(unhandled, 'deadline', 'deadlines')} from this calendar {unhandled === 1 ? 'is' : 'are'} not marked handled, so {unhandled === 1 ? 'it counts' : 'they count'} as overdue.</p>
                <button class="btn" type="button" disabled={busy} onclick={() => markAllHandled(c.past_unhandled)}>Mark all handled</button>
              </div>
            {/if}
          </li>
        {/each}
      </ul>
    {/if}

    <input
      bind:this={refreshInput}
      type="file"
      accept=".ics,text/calendar"
      hidden
      aria-label="Replacement calendar file"
      onchange={(event) => onRefreshPicked(event.currentTarget)}
    />
  </section>

  <section class="card" aria-labelledby="sources-add">
    <h2 id="sources-add">Add a calendar</h2>

    <form class="add" onsubmit={submitAdd}>
      <div class="grid-2">
        <label class="field">
          <span>Name</span>
          <input type="text" required maxlength="80" bind:value={addName} disabled={busy} />
        </label>
        <label class="field">
          <span>Type</span>
          <select value={addKind} disabled={busy} onchange={(event) => chooseKind(event.currentTarget.value as CalendarKind)}>
            {#each kindOptions as option (option.value)}
              <option value={option.value}>{option.label}</option>
            {/each}
          </select>
        </label>
        <label class="field">
          <span>Mode</span>
          <select bind:value={addMode} disabled={busy}>
            <option value="events">Fixed events</option>
            <option value="coursework">Coursework (assignments become deadlines)</option>
          </select>
        </label>
      </div>

      {@render colourPicker(addColor, (hex) => { addColor = hex; }, busy)}

      <div class="method">
        <div class="segmented" role="group" aria-label="How to add it">
          <button
            type="button"
            class="seg"
            aria-pressed={addMethod === 'link'}
            disabled={busy || !desktop}
            onclick={() => chooseMethod('link')}
          >Subscribe with a link</button>
          <button
            type="button"
            class="seg"
            aria-pressed={addMethod === 'file'}
            disabled={busy}
            onclick={() => chooseMethod('file')}
          >Import a file</button>
        </div>
        {#if !desktop}<span class="desk-note">Available in the desktop app</span>{/if}
      </div>

      {#if addMethod === 'link'}
        <label class="field">
          <span>Calendar link</span>
          <input type="url" required placeholder="https://… or webcal://…" bind:value={addUrl} disabled={busy || !desktop} />
        </label>
        <p class="help">{linkHelp[addKind]}</p>
        <p class="privacy">The link works like a password. It is kept in Windows Credential Manager; only its host name is shown here.</p>
      {:else}
        <label class="field">
          <span>Calendar file</span>
          <input
            type="file"
            accept=".ics,text/calendar"
            bind:this={addFileInput}
            disabled={busy}
            onchange={(event) => { addFile = event.currentTarget.files?.[0] ?? null; }}
          />
        </label>
        <p class="help">Export an .ics file from your calendar. A file is a snapshot: re-import it to update.</p>
      {/if}

      <div class="actions">
        <button class="btn primary" type="submit" disabled={busy || !addReady}>Add calendar</button>
      </div>
    </form>
  </section>

  <section class="card" aria-labelledby="sources-trace">
    <h2 id="sources-trace">Trace tasks</h2>

    {#if personal.trace.exported_at}
      <div class="trace-summary">
        <p>
          <strong>{personal.trace.present_tasks} {plural(personal.trace.present_tasks, 'task', 'tasks')}</strong>
          · {personal.trace.completed_tasks} completed
        </p>
        <p class="meta">Exported {personal.trace.exported_at}{#if personal.trace.imported_at} · imported {personal.trace.imported_at}{/if}</p>
        {#if personal.trace.last_attempt}<p class="meta">Last attempt: {readable(personal.trace.last_attempt)}</p>{/if}
        {#if personal.trace.unresolved_dates > 0}
          <p class="notice-text">
            {personal.trace.unresolved_dates} Trace due {plural(personal.trace.unresolved_dates, 'value', 'values')} {personal.trace.unresolved_dates === 1 ? 'needs' : 'need'} precision before {personal.trace.unresolved_dates === 1 ? 'it becomes' : 'they become'} deadlines.
          </p>
        {/if}
      </div>
    {:else}
      <p class="empty">Bring your tasks into view: in Trace press Ctrl+K → Export as JSON.</p>
    {/if}

    <div class="actions">
      <button class="btn" type="button" disabled={busy || !desktop} onclick={() => run('Trace imported', () => pickTraceExport())}>Choose export file…</button>
      {#if !desktop}<span class="desk-note">Available in the desktop app</span>{/if}
      <button class="btn" type="button" disabled={busy} onclick={() => traceInput?.click()}>Import a JSON file once</button>
    </div>

    {#if personal.trace.export_path}
      <div class="watch">
        <p>Watching {personal.trace.export_path}. Re-export from Trace and Temporal picks it up when you return.</p>
        <div class="actions">
          <button class="btn" type="button" disabled={busy || !desktop} onclick={() => run('Trace re-read', () => reimportTrace())}>Re-read now</button>
          <button class="btn" type="button" disabled={busy || !desktop} onclick={() => run('Stopped watching', async () => { await forgetTraceExport(); })}>Stop watching</button>
        </div>
      </div>
    {/if}

    <input
      bind:this={traceInput}
      type="file"
      accept=".json,application/json"
      hidden
      aria-label="Trace export file"
      onchange={(event) => onTraceJson(event.currentTarget)}
    />

    <p class="note">Trace stays the owner of task text, dates, and completion. Temporal never writes to Trace.</p>
  </section>
</div>

<style>
  .sources {
    max-width: 920px;
    margin: 0 auto;
    padding: 24px 16px 48px;
    display: grid;
    gap: 16px;
    color: var(--ink);
    font-family: var(--font);
  }
  .page-head h1 {
    margin: 0;
    font-family: var(--font-display);
    font-size: 28px;
    font-weight: 600;
    line-height: 1.2;
    color: var(--ink);
  }
  .intro { margin: 6px 0 0; font-size: 14px; color: var(--muted); }

  .card {
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    padding: 20px;
    display: grid;
    gap: 16px;
    min-width: 0;
  }
  @media (max-width: 480px) {
    .card { padding: 16px; }
  }
  .card h2 { margin: 0; font-size: 17px; font-weight: 600; color: var(--ink); }

  .empty, .note { margin: 0; font-size: 14px; color: var(--muted); }
  .note { font-size: 12px; color: var(--faint); }
  .meta { margin: 2px 0 0; font-size: 13px; color: var(--muted); overflow-wrap: anywhere; }
  .faint { color: var(--faint); }
  .help { margin: 0; font-size: 13px; color: var(--muted); }
  .privacy { margin: 0; font-size: 12px; color: var(--faint); }
  .desk-note { font-size: 12px; color: var(--faint); }

  /* Calendars */
  .calendar-list { list-style: none; margin: 0; padding: 0; display: grid; gap: 12px; }
  .calendar {
    display: grid;
    gap: 12px;
    padding: 14px;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    min-width: 0;
  }
  .calendar-main { display: grid; grid-template-columns: 12px minmax(0, 1fr); gap: 12px; align-items: start; }
  .swatch { width: 12px; height: 12px; border-radius: 50%; margin-top: 5px; }
  .calendar-body { min-width: 0; }
  .calendar-title { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; }
  .calendar-title strong { font-size: 15px; overflow-wrap: anywhere; }
  .badge {
    font-size: 12px;
    padding: 1px 7px;
    border-radius: 4px;
    background: var(--surface-3);
    color: var(--muted);
  }
  .pill { font-size: 12px; padding: 2px 8px; border-radius: 999px; }
  .pill.ok { background: var(--ok-soft); color: var(--ok); }
  .pill.risk { background: var(--risk-soft); color: var(--risk); }
  .pill.neutral { background: var(--surface-3); color: var(--muted); }
  .error { margin: 4px 0 0; font-size: 13px; color: var(--risk); overflow-wrap: anywhere; }

  .confirm, .notice {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 10px;
    padding: 10px 12px;
    border-radius: var(--radius-sm);
  }
  .confirm { background: var(--surface-3); }
  .confirm p { margin: 0; font-size: 13px; color: var(--ink); }
  .notice { background: var(--risk-soft); }
  .notice p { margin: 0; font-size: 13px; color: var(--ink); }

  .editor { display: grid; gap: 14px; padding-top: 4px; border-top: 1px dashed var(--line-strong); }

  /* Shared controls */
  .actions { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; }
  .btn {
    font: inherit;
    font-size: 13px;
    color: var(--ink);
    background: var(--surface);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    padding: 6px 12px;
    cursor: pointer;
  }
  .btn:hover:not(:disabled) { background: var(--surface-3); }
  .btn.primary { background: var(--accent); color: var(--accent-ink); border-color: var(--accent); }
  .btn.danger { color: var(--risk); border-color: var(--risk); }
  .btn:disabled, .seg:disabled { opacity: 0.5; cursor: default; }
  .btn:focus-visible,
  .seg:focus-visible,
  .swatch-btn:focus-visible,
  input:focus-visible,
  select:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .field { display: grid; gap: 4px; min-width: 0; font-size: 13px; }
  .field > span { font-weight: 600; color: var(--ink); }
  .check {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 14px;
    color: var(--ink);
    align-self: end;
    padding-bottom: 6px;
  }
  .grid-2 { display: grid; grid-template-columns: repeat(auto-fit, minmax(190px, 1fr)); gap: 12px; align-items: start; }
  .add { display: grid; gap: 14px; }

  input[type='text'],
  input[type='url'],
  select {
    font: inherit;
    font-size: 14px;
    color: var(--ink);
    background: var(--surface);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    padding: 7px 9px;
    width: 100%;
    min-width: 0;
    box-sizing: border-box;
  }
  input[type='file'] { font-size: 13px; color: var(--ink); max-width: 100%; }
  input[type='checkbox'] { accent-color: var(--accent); }
  input[type='color'] {
    width: 36px;
    height: 30px;
    padding: 0;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    background: var(--surface);
    cursor: pointer;
  }
  input[hidden] { display: none; }

  .colours { border: 0; margin: 0; padding: 0; min-width: 0; display: grid; gap: 6px; }
  .colours legend { padding: 0; margin-bottom: 6px; font-size: 13px; font-weight: 600; color: var(--ink); }
  .swatches { display: flex; flex-wrap: wrap; align-items: center; gap: 10px; }
  .swatch-btn {
    width: 26px;
    height: 26px;
    padding: 0;
    border-radius: 50%;
    border: 2px solid var(--surface);
    box-shadow: 0 0 0 1px var(--line-strong);
    cursor: pointer;
  }
  .swatch-btn[aria-pressed='true'] { box-shadow: 0 0 0 2px var(--surface), 0 0 0 4px var(--ink); }
  .custom { display: inline-flex; align-items: center; gap: 8px; font-size: 13px; color: var(--muted); }

  .method { display: flex; flex-wrap: wrap; align-items: center; gap: 10px; }
  .segmented {
    display: inline-flex;
    max-width: 100%;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    overflow: hidden;
    background: var(--surface);
  }
  .seg {
    font: inherit;
    font-size: 13px;
    color: var(--muted);
    background: transparent;
    border: 0;
    padding: 7px 14px;
    cursor: pointer;
  }
  .seg + .seg { border-left: 1px solid var(--line-strong); }
  .seg[aria-pressed='true'] { background: var(--accent-soft); color: var(--ink); font-weight: 600; }

  /* Trace */
  .trace-summary { display: grid; gap: 4px; }
  .trace-summary p { margin: 0; font-size: 14px; color: var(--ink); }
  .trace-summary .notice-text { color: var(--risk); }
  .watch {
    display: grid;
    gap: 8px;
    padding: 12px;
    border-radius: var(--radius-sm);
    background: var(--surface-2);
    border: 1px solid var(--line);
  }
  .watch p { margin: 0; font-size: 13px; color: var(--ink); overflow-wrap: anywhere; }
</style>
