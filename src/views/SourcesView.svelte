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
  <section class="headline">
    <h1>Sources</h1>
    <div class="aside">Temporal reads these and never writes back.<br />Each keeps its own freshness.</div>
  </section>

  <section class="block" aria-labelledby="sources-calendars">
    <div class="section-bar">
      <span class="index">01</span>
      <h2 id="sources-calendars">Calendars</h2>
      <span class="spacer"></span>
      <span class="label">{personal.calendars.length} connected</span>
    </div>

    {#if personal.calendars.length === 0}
      <div class="list-empty"><p class="caps">No calendars yet. Add Google, Outlook, or your Quercus course calendar below.</p></div>
    {:else}
      <ul class="calendar-list">
        {#each personal.calendars as c (c.id)}
          {@const pill = healthPill(c)}
          {@const unhandled = c.past_unhandled.length}
          <li class="calendar" class:is-hidden={c.hidden}>
            <div class="calendar-row">
              <span class="swatch" style:background={c.color} aria-hidden="true"></span>

              <div class="name">
                <strong>{c.label}</strong>
                {#if c.hidden}<span class="label">Hidden</span>{/if}
              </div>

              <div class="meta">
                <p>{kindLabels[c.kind]} · {c.mode === 'coursework' ? 'Coursework: assignments become deadlines' : 'Events'} · {c.origin === 'file' ? 'File' : 'Subscribed'} · {c.origin_label}</p>
                <p>{#if c.last_success}Updated {c.last_success}{' · '}{/if}{c.event_count} {plural(c.event_count, 'entry', 'entries')}{#if c.skipped > 0} · {c.skipped} unreadable {plural(c.skipped, 'entry', 'entries')} skipped{/if}</p>
                {#if c.hidden}<p class="faint">Hidden from Horizon and Calendar</p>{/if}
              </div>

              <span class="health" title={pill.text}>
                {#if c.health === 'healthy'}
                  <i class="sq ink" aria-hidden="true"></i>Up to date
                {:else if c.health === 'stale'}
                  <i class="sq signal" aria-hidden="true"></i>Stale — last-known
                {:else if c.health === 'unavailable' || c.health === 'partial' || c.health === 'incompatible'}
                  <i class="sq risk" aria-hidden="true"></i>Refresh failed
                {:else}
                  <i class="sq" aria-hidden="true"></i>{c.health === 'never_loaded' ? 'Not loaded' : readable(c.health)}
                {/if}
              </span>

              <div class="actions">
                {#if c.origin === 'file'}
                  <button type="button" disabled={busy} aria-label="Refresh {c.label}" onclick={() => refreshFile(c)}>Refresh</button>
                {:else}
                  <button type="button" disabled={busy || !desktop} aria-label="Refresh {c.label}" onclick={() => run('Calendar refreshed', () => refreshCalendars(c.id))}>Refresh</button>
                  {#if !desktop}<span class="desk-note">Available in the desktop app</span>{/if}
                {/if}
                <button type="button" disabled={busy} aria-label="Edit {c.label}" aria-expanded={editingId === c.id} onclick={() => toggleEdit(c)}>Edit</button>
                <button type="button" disabled={busy} aria-label="Remove {c.label}" onclick={() => { removingId = c.id; }}>Remove</button>
              </div>

              {#if c.last_error}<p class="error">{c.last_error}</p>{/if}
            </div>

            {#if removingId === c.id}
              <div class="notice warn">
                <span class="label">Confirm</span>
                <div>
                  <strong>Remove {c.label}?</strong>
                  <p>Its events disappear from Temporal; the source is not changed.</p>
                </div>
                <div class="actions">
                  <button class="danger" type="button" disabled={busy} onclick={() => remove(c)}>Remove</button>
                  <button type="button" disabled={busy} onclick={() => { removingId = null; }}>Cancel</button>
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
                <div class="editor-row">
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
                  <button class="primary" type="submit" disabled={busy || editLabel.trim() === ''}>Save</button>
                  <button type="button" disabled={busy} onclick={() => { editingId = null; }}>Cancel</button>
                </div>
              </form>
            {/if}

            {#if unhandled > 0}
              <div class="notice warn">
                <span class="label">Overdue</span>
                <div>
                  <p>{unhandled} past {plural(unhandled, 'deadline', 'deadlines')} from this calendar {unhandled === 1 ? 'is' : 'are'} not marked handled, so {unhandled === 1 ? 'it counts' : 'they count'} as overdue.</p>
                </div>
                <button type="button" disabled={busy} onclick={() => markAllHandled(c.past_unhandled)}>Mark all handled →</button>
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

  <section class="block" aria-labelledby="sources-add">
    <div class="section-bar">
      <span class="index">02</span>
      <h2 id="sources-add">Add a calendar</h2>
    </div>

    <form class="body" onsubmit={submitAdd}>
      <div class="field-grid">
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
        <button class="primary" type="submit" disabled={busy || !addReady}>Add calendar →</button>
      </div>
    </form>
  </section>

  <section class="block" aria-labelledby="sources-trace">
    <div class="section-bar">
      <span class="index">03</span>
      <h2 id="sources-trace">Trace tasks</h2>
    </div>

    <div class="body">
      {#if personal.trace.exported_at}
        <dl class="spec">
          <div class="spec-row">
            <dt>Tasks</dt>
            <dd><strong>{personal.trace.present_tasks} {plural(personal.trace.present_tasks, 'task', 'tasks')}</strong> · {personal.trace.completed_tasks} completed</dd>
          </div>
          <div class="spec-row">
            <dt>Exported</dt>
            <dd>{personal.trace.exported_at}</dd>
          </div>
          {#if personal.trace.imported_at}
            <div class="spec-row">
              <dt>Imported</dt>
              <dd>{personal.trace.imported_at}</dd>
            </div>
          {/if}
          {#if personal.trace.last_attempt}
            <div class="spec-row">
              <dt>Last attempt</dt>
              <dd>{readable(personal.trace.last_attempt)}</dd>
            </div>
          {/if}
          {#if personal.trace.unresolved_dates > 0}
            <div class="spec-row">
              <dt>Unresolved dates</dt>
              <dd class="risk">{personal.trace.unresolved_dates} Trace due {plural(personal.trace.unresolved_dates, 'value', 'values')} {personal.trace.unresolved_dates === 1 ? 'needs' : 'need'} precision before {personal.trace.unresolved_dates === 1 ? 'it becomes' : 'they become'} deadlines.</dd>
            </div>
          {/if}
        </dl>
      {:else}
        <p class="caps">Bring your tasks into view: in Trace press Ctrl+K → Export as JSON.</p>
      {/if}

      <div class="actions">
        <button type="button" disabled={busy || !desktop} onclick={() => run('Trace imported', () => pickTraceExport())}>Choose export file…</button>
        {#if !desktop}<span class="desk-note">Available in the desktop app</span>{/if}
        <button type="button" disabled={busy} onclick={() => traceInput?.click()}>Import a JSON file once</button>
      </div>

      {#if personal.trace.export_path}
        <div class="watch">
          <p class="note">Watching {personal.trace.export_path}. Re-export from Trace and Temporal picks it up when you return.</p>
          <div class="actions">
            <button type="button" disabled={busy || !desktop} onclick={() => run('Trace re-read', () => reimportTrace())}>Re-read now</button>
            <button type="button" disabled={busy || !desktop} onclick={() => run('Stopped watching', async () => { await forgetTraceExport(); })}>Stop watching</button>
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
    </div>
  </section>
</div>

<style>
  .sources { min-width: 0; padding-bottom: 48px; color: var(--ink); }

  /* Ruled blocks: the section bar is the rule; bodies run full bleed. */
  .block + .block { border-top: var(--rule); }
  .section-bar h2 { margin: 0; font: inherit; letter-spacing: inherit; text-transform: inherit; }
  .body { display: grid; gap: 18px; min-width: 0; padding: 20px 28px; }
  .list-empty { padding: 14px 28px; }
  .caps { margin: 0; font-family: var(--font-mono); font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; line-height: 1.6; color: var(--muted); }
  .note { margin: 0; font-size: 12px; line-height: 1.5; color: var(--muted); overflow-wrap: anywhere; }
  .faint { color: var(--faint); }
  .desk-note { font-family: var(--font-mono); font-size: 10.5px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--faint); }
  .actions { display: flex; flex-wrap: wrap; align-items: center; gap: 8px; }

  /* Calendars: one ruled row each. Confirm and edit open inside the same ruled block. */
  .calendar-list { list-style: none; margin: 0; padding: 0; }
  .calendar { min-width: 0; border-bottom: 1px solid var(--line); }
  .calendar:last-child { border-bottom: 0; }
  .calendar.is-hidden .calendar-row { opacity: 0.55; }
  .calendar-row {
    display: grid;
    grid-template-columns: 14px minmax(150px, 1fr) minmax(0, 1.5fr) auto auto;
    grid-template-areas: 'sw name meta health actions' '. err err err err';
    column-gap: 18px;
    align-items: start;
    padding: 14px 28px;
    min-width: 0;
  }
  .swatch { grid-area: sw; width: 14px; height: 14px; margin-top: 4px; border: 1px solid var(--ink); }
  .name {
    grid-area: name;
    display: flex;
    flex-wrap: wrap;
    align-items: baseline;
    gap: 4px 12px;
    min-width: 0;
    font-family: var(--font-display);
    font-stretch: 115%;
    font-weight: 760;
    font-size: 20px;
    line-height: 1.05;
    letter-spacing: -0.01em;
    text-transform: uppercase;
  }
  .name strong { font-weight: inherit; overflow-wrap: anywhere; }
  .name .label { font-weight: 500; color: var(--muted); }
  .meta {
    grid-area: meta;
    display: grid;
    gap: 2px;
    min-width: 0;
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.06em;
    line-height: 1.55;
    text-transform: uppercase;
    color: var(--muted);
    overflow-wrap: anywhere;
  }
  .meta p { margin: 0; }
  .health {
    grid-area: health;
    display: flex;
    align-items: center;
    gap: 8px;
    padding-top: 3px;
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.06em;
    line-height: 1.2;
    text-transform: uppercase;
    white-space: nowrap;
  }
  .sq { flex-shrink: 0; width: 8px; height: 8px; border: 1px solid var(--ink); }
  .sq.ink { background: var(--ink); }
  .sq.signal { background: var(--accent); border-color: var(--accent); }
  .sq.risk { background: var(--risk); border-color: var(--risk); }
  .calendar-row > .actions { grid-area: actions; justify-content: flex-end; }
  .error {
    grid-area: err;
    margin: 6px 0 0;
    font-family: var(--font-mono);
    font-size: 11px;
    line-height: 1.5;
    color: var(--risk);
    overflow-wrap: anywhere;
  }

  /* Rows that open under a calendar: confirm, edit, and the overdue notice. */
  .calendar > .notice { border-top: 1px solid var(--line); border-bottom: 0; }
  .calendar > .notice p { margin-top: 0; color: var(--ink); }
  .calendar > .notice > .actions { margin-right: 28px; }
  .editor {
    display: grid;
    gap: 16px;
    min-width: 0;
    margin: 0 28px 18px;
    padding: 16px;
    border: var(--rule);
  }

  /* Labels above fields: mono caps. Inputs keep their global square ruled style. */
  .field { display: grid; gap: 6px; min-width: 0; }
  .field > span { font-family: var(--font-mono); font-size: 10.5px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--muted); }
  .field > input[type='text'],
  .field > input[type='url'],
  .field > select { width: 100%; }
  .field > input[type='file']::file-selector-button {
    margin-right: 12px;
    padding: 7px 11px;
    font-family: var(--font-mono);
    font-size: 11.5px;
    font-weight: 500;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--ink);
    background: transparent;
    border: 1px solid var(--line-strong);
    cursor: pointer;
  }
  .field > input[type='file']::file-selector-button:hover { background: var(--invert-bg); color: var(--invert-ink); }
  .field-grid { display: grid; grid-template-columns: repeat(3, minmax(0, 1fr)); gap: 16px; }
  .editor-row { display: grid; grid-template-columns: repeat(2, minmax(0, 1fr)); gap: 16px; align-items: end; }
  .check {
    display: flex;
    align-items: center;
    gap: 10px;
    min-height: 32px;
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
  }

  /* Colour: 24px squares, selected is a 2px ink outline set off by 2px. */
  .colours { display: grid; gap: 8px; min-width: 0; margin: 0; padding: 0; border: 0; }
  .colours legend { padding: 0 0 6px; font-family: var(--font-mono); font-size: 10.5px; letter-spacing: 0.06em; text-transform: uppercase; color: var(--muted); }
  .swatches { display: flex; flex-wrap: wrap; align-items: center; gap: 10px; }
  .swatch-btn { width: 24px; height: 24px; padding: 0; border: 1px solid var(--ink); }
  .swatch-btn[aria-pressed='true'] { outline: 2px solid var(--ink); outline-offset: 2px; }
  .custom {
    display: inline-flex;
    align-items: center;
    gap: 8px;
    margin-left: 6px;
    font-family: var(--font-mono);
    font-size: 10.5px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .custom input[type='color'] { width: 24px; height: 24px; padding: 0; }

  /* Joined two-way control: shared borders, inverted when active. */
  .method { display: flex; flex-wrap: wrap; align-items: center; gap: 14px; }
  .segmented { display: inline-flex; max-width: 100%; }
  .seg { padding: 8px 16px; }
  .seg + .seg { margin-left: -1px; }
  .seg[aria-pressed='true'] { background: var(--invert-bg); color: var(--invert-ink); }
  .help { margin: 0; font-size: 12px; line-height: 1.5; color: var(--muted); }
  .privacy {
    margin: 0;
    font-family: var(--font-mono);
    font-size: 10.5px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .privacy::before { content: '▲ '; }

  /* Trace: a two-column spec table. */
  .spec { margin: 0; border-top: 1px solid var(--line); }
  .spec-row {
    display: grid;
    grid-template-columns: 220px minmax(0, 1fr);
    gap: 16px;
    align-items: baseline;
    padding: 11px 0;
    border-bottom: 1px solid var(--line);
    font-size: 13px;
    line-height: 1.5;
  }
  .spec dt { font-family: var(--font-mono); font-size: 11px; letter-spacing: 0.06em; text-transform: uppercase; }
  .watch {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding-top: 14px;
    border-top: 1px solid var(--line);
  }
  .watch .note { flex: 1 1 280px; min-width: 0; }

  input[hidden] { display: none; }

  @media (max-width: 1080px) {
    .calendar-row {
      grid-template-columns: 14px minmax(0, 1fr) auto;
      grid-template-areas: 'sw name name' '. meta meta' '. health actions' '. err err';
      row-gap: 10px;
    }
  }
  @media (max-width: 760px) {
    .calendar-row,
    .body,
    .list-empty { padding-left: 16px; padding-right: 16px; }
    .calendar-row {
      grid-template-columns: 14px minmax(0, 1fr);
      grid-template-areas: 'sw name' '. meta' '. health' '. actions' '. err';
    }
    .calendar-row > .actions { justify-content: flex-start; }
    .editor { margin: 0 16px 18px; }
    .calendar > .notice > .actions { margin: 0 16px 12px; justify-self: start; }
  }
  @media (max-width: 560px) {
    .field-grid, .editor-row { grid-template-columns: minmax(0, 1fr); }
    .spec-row { grid-template-columns: minmax(0, 1fr); gap: 4px; }
  }
</style>
