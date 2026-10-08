<script lang="ts">
  import { untrack } from 'svelte';
  import { desktop, type PersonalView, type SettingsPatch, type Theme } from '../lib/api.ts';
  import { weekdays, type Capacity, type LocalMutation, type UsualAvailability, type Weekday } from '../lib/local.ts';

  let { personal, busy, onsettings, onmutate, onexamples }: {
    personal: PersonalView;
    busy: boolean;
    onsettings: (patch: SettingsPatch) => Promise<boolean>;   // resolves false on failure (parent shows the error)
    onmutate: (mutation: LocalMutation) => Promise<boolean>;
    onexamples: () => void;                                     // open the synthetic example weeks
  } = $props();

  const settings = $derived(personal.settings);

  // Shared helpers.
  function withValue(options: number[], current: number): number[] {
    return options.includes(current) ? options : [...options, current].sort((a, b) => a - b);
  }

  // Time and display.
  const LOCAL = '__local__';
  const localZone = Intl.DateTimeFormat().resolvedOptions().timeZone;
  const curatedZones = [
    'America/Toronto', 'America/Vancouver', 'America/Edmonton', 'America/Winnipeg', 'America/Halifax',
    'America/St_Johns', 'America/New_York', 'America/Chicago', 'America/Denver', 'America/Los_Angeles',
    'Europe/London', 'Europe/Paris', 'Europe/Berlin', 'Asia/Kolkata', 'Asia/Shanghai', 'Asia/Tokyo',
    'Australia/Sydney', 'UTC',
  ];
  const zoneOptions = $derived(settings.display_zone && !curatedZones.includes(settings.display_zone)
    ? [settings.display_zone, ...curatedZones] : curatedZones);
  const zoneValue = $derived(settings.display_zone ?? LOCAL);
  const durationOptions = $derived(withValue([15, 30, 45, 60, 90, 120], settings.default_event_minutes));
  const hours = Array.from({ length: 24 }, (_, hour) => hour);
  const hourLabel = (hour: number) => `${hour % 12 === 0 ? 12 : hour % 12} ${hour < 12 ? 'AM' : 'PM'}`;
  const themes: { value: Theme; label: string }[] = [
    { value: 'system', label: 'System' },
    { value: 'light', label: 'Light' },
    { value: 'dark', label: 'Dark' },
  ];

  async function setZone(event: Event) {
    const select = event.currentTarget as HTMLSelectElement;
    const zone = select.value === LOCAL ? localZone : select.value;
    if (!(await onsettings({ display_zone: zone }))) select.value = zoneValue;
  }
  async function setWeekStart(value: 'mon' | 'sun') {
    if (settings.week_starts_on !== value) await onsettings({ week_starts_on: value });
  }
  async function setDuration(event: Event) {
    const select = event.currentTarget as HTMLSelectElement;
    if (!(await onsettings({ default_event_minutes: Number(select.value) }))) select.value = String(settings.default_event_minutes);
  }
  async function setDayStart(event: Event) {
    const select = event.currentTarget as HTMLSelectElement;
    if (!(await onsettings({ day_start_hour: Number(select.value) }))) select.value = String(settings.day_start_hour);
  }
  async function setTheme(value: Theme) {
    if (settings.theme !== value) await onsettings({ theme: value });
  }

  // Reminders. The lead time is remembered while reminders are off, so turning them back on restores it.
  const reminderOn = $derived(settings.reminder_minutes !== null);
  let reminderPick = $state<number | null>(null);
  const leadValue = $derived(settings.reminder_minutes ?? reminderPick ?? 10);
  const leadOptions = $derived(withValue([0, 5, 10, 15, 30, 60], leadValue));
  const leadLabel = (minutes: number) => (minutes === 0 ? 'At start' : minutes === 60 ? '1 hour before' : `${minutes} minutes before`);

  async function setReminders(event: Event) {
    const box = event.currentTarget as HTMLInputElement;
    let ok: boolean;
    if (box.checked) {
      ok = await onsettings({ reminder_minutes: leadValue });
    } else {
      if (settings.reminder_minutes !== null) reminderPick = settings.reminder_minutes;
      ok = await onsettings({ reminders_off: true });
    }
    if (!ok) box.checked = reminderOn;
  }
  async function setLead(event: Event) {
    const select = event.currentTarget as HTMLSelectElement;
    if (!(await onsettings({ reminder_minutes: Number(select.value) }))) select.value = String(leadValue);
  }
  async function setTray(event: Event) {
    const box = event.currentTarget as HTMLInputElement;
    if (!(await onsettings({ keep_running_in_tray: box.checked }))) box.checked = settings.keep_running_in_tray;
  }
  async function setLogin(event: Event) {
    const box = event.currentTarget as HTMLInputElement;
    if (!(await onsettings({ open_at_login: box.checked }))) box.checked = settings.open_at_login;
  }

  // Usual availability: clock helpers. Minutes run 0..1440; an end of 00:00 always means midnight (1440).
  type TimeRange = { key: number; start: string; end: string };
  type Draft = Record<Weekday, TimeRange[]>;

  function parseClock(text: string): number {
    const match = /^(\d{2}):(\d{2})/.exec(text);
    if (!match) return NaN;
    const hour = Number(match[1]);
    const minute = Number(match[2]);
    return hour > 23 || minute > 59 ? NaN : hour * 60 + minute;
  }
  const endMinute = (text: string) => {
    const minute = parseClock(text);
    return minute === 0 ? 1440 : minute;
  };
  const clock = (minute: number) => `${String(Math.floor(minute / 60)).padStart(2, '0')}:${String(minute % 60).padStart(2, '0')}`;
  const endText = (minute: number) => (minute === 1440 ? '00:00' : clock(minute));
  const spansOf = (ranges: TimeRange[]) => ranges.map(range => ({ start: parseClock(range.start), end: endMinute(range.end) }));
  const spansFor = (source: Draft) => weekdays.flatMap(day => spansOf(source[day]).map(span => ({ weekday: day, ...span })));
  const contextTags = (text: string) => [...new Set(text.split(',').map(tag => tag.trim().toLowerCase()).filter(tag => tag !== ''))];

  const dayNames: Record<Weekday, string> = {
    mon: 'Monday', tue: 'Tuesday', wed: 'Wednesday', thu: 'Thursday', fri: 'Friday', sat: 'Saturday', sun: 'Sunday',
  };
  const shortNames: Record<Weekday, string> = {
    mon: 'Mon', tue: 'Tue', wed: 'Wed', thu: 'Thu', fri: 'Fri', sat: 'Sat', sun: 'Sun',
  };
  const orderedDays = $derived<Weekday[]>(settings.week_starts_on === 'sun' ? ['sun', ...weekdays.slice(0, 6)] : weekdays);
  const WORKDAYS: Weekday[] = ['mon', 'tue', 'wed', 'thu', 'fri'];
  const presets: { label: string; days: Weekday[]; start: string; end: string }[] = [
    { label: 'Weekdays 9–5', days: WORKDAYS, start: '09:00', end: '17:00' },
    { label: 'Weekdays 9–9', days: WORKDAYS, start: '09:00', end: '21:00' },
    { label: 'Every day 9–9', days: weekdays, start: '09:00', end: '21:00' },
  ];
  const energyOptions: { value: Capacity; label: string }[] = [
    { value: 'unknown', label: 'Unknown' },
    { value: 'light', label: 'Light' },
    { value: 'normal', label: 'Normal' },
    { value: 'deep', label: 'Deep' },
  ];
  const EMPTY_PROBLEM = 'Add at least one time range to save. To clear usual availability, use Remove usual availability.';

  const emptyDraft = (): Draft => ({ mon: [], tue: [], wed: [], thu: [], fri: [], sat: [], sun: [] });
  let nextKey = 0;
  let draft = $state<Draft>(emptyDraft());
  let placesText = $state('');
  let energyChoice = $state<Capacity>('unknown');

  function loadDraft(usual: UsualAvailability | null) {
    const next = emptyDraft();
    for (const block of usual?.blocks ?? []) {
      next[block.weekday].push({ key: nextKey++, start: clock(block.start_minute), end: endText(block.end_minute) });
    }
    for (const day of weekdays) next[day].sort((a, b) => parseClock(a.start) - parseClock(b.start));
    draft = next;
    placesText = usual?.contexts.kind === 'known' ? usual.contexts.value.join(', ') : '';
    energyChoice = usual?.energy_capacity ?? 'unknown';
  }

  // Re-initialise only when the stored pattern itself changes, never on unrelated settings updates.
  const storedKey = $derived(personal.local.usual_availability
    ? `${personal.local.usual_availability.meta.id}:${personal.local.usual_availability.meta.revision}`
    : 'none');
  $effect.pre(() => {
    void storedKey;
    untrack(() => loadDraft(personal.local.usual_availability));
  });

  // Validation: every range complete, ends after it starts, and no overlap within a day.
  function findProblem(source: Draft): string | null {
    for (const day of weekdays) {
      const name = dayNames[day];
      const spans = spansOf(source[day]);
      if (spans.some(span => Number.isNaN(span.start) || Number.isNaN(span.end))) return `${name}: fill in both times for each range.`;
      if (spans.some(span => span.end <= span.start)) return `${name}: each range must end after it starts.`;
      spans.sort((a, b) => a.start - b.start);
      for (let i = 1; i < spans.length; i++) {
        if (spans[i].start < spans[i - 1].end) return `${name}: time ranges overlap.`;
      }
    }
    return null;
  }

  function blocksFor(source: Draft) {
    return weekdays.flatMap(day => spansOf(source[day]).sort((a, b) => a.start - b.start)
      .map(span => ({ weekday: day, start: clock(span.start), end: span.end === 1440 ? '24:00' : clock(span.end) })));
  }

  function signature(spans: { weekday: Weekday; start: number; end: number }[], tags: string[], energy: Capacity): string {
    const days = weekdays.map(day => spans.filter(span => span.weekday === day)
      .sort((a, b) => a.start - b.start).map(span => `${span.start}-${span.end}`).join(','));
    return `${days.join('|')}#${[...tags].sort().join(',')}#${energy}`;
  }

  const storedSignature = $derived.by(() => {
    const usual = personal.local.usual_availability;
    return signature(
      (usual?.blocks ?? []).map(block => ({ weekday: block.weekday, start: block.start_minute, end: block.end_minute })),
      usual?.contexts.kind === 'known' ? contextTags(usual.contexts.value.join(',')) : [],
      usual?.energy_capacity ?? 'unknown',
    );
  });
  const draftSignature = $derived(signature(spansFor(draft), contextTags(placesText), energyChoice));
  const dirty = $derived(draftSignature !== storedSignature);
  const hasRanges = $derived(weekdays.some(day => draft[day].length > 0));
  const problem = $derived(findProblem(draft) ?? (dirty && !hasRanges ? EMPTY_PROBLEM : null));

  // Editor actions.
  function applyPreset(days: Weekday[], start: string, end: string) {
    const next = emptyDraft();
    for (const day of days) next[day] = [{ key: nextKey++, start, end }];
    draft = next;
  }
  function clearAll() {
    draft = emptyDraft();
  }
  function addRange(day: Weekday) {
    const latest = draft[day].reduce((max, range) => {
      const end = endMinute(range.end);
      return Number.isNaN(end) ? max : Math.max(max, end);
    }, -1);
    // The hour after the last range, ending at 17:00 or an hour later; otherwise 09:00 to 17:00.
    let start = 540;
    let end = 1020;
    if (latest >= 0 && latest + 60 < 1380) {
      start = latest + 60;
      end = Math.min(1440, Math.max(1020, start + 60));
    }
    draft[day].push({ key: nextKey++, start: clock(start), end: endText(end) });
  }
  function removeRange(day: Weekday, key: number) {
    draft[day] = draft[day].filter(range => range.key !== key);
  }
  function barsFor(day: Weekday) {
    return spansOf(draft[day])
      .filter(span => span.end > span.start)
      .map(span => ({ left: (span.start / 1440) * 100, width: ((span.end - span.start) / 1440) * 100 }));
  }

  async function save() {
    if (!dirty || problem !== null) return;
    const context = contextTags(placesText);
    const mutation: LocalMutation = {
      action: 'upsert',
      kind: 'usual_availability',
      zone: settings.display_zone ?? 'America/Toronto',
      blocks: blocksFor(draft),
      energy_capacity: energyChoice,
    };
    if (context.length) mutation.context = context.join(', ');
    await onmutate(mutation);
  }
  async function removeUsual() {
    await onmutate({ action: 'remove', kind: 'usual_availability' });
  }
</script>

<div class="settings">
  <h1>Settings</h1>

  <section class="card" aria-labelledby="display-heading">
    <h2 id="display-heading">Time and display</h2>

    <div class="row">
      <div class="label-col"><label for="zone">Time zone</label></div>
      <div class="control-col">
        <select id="zone" value={zoneValue} disabled={busy} onchange={setZone} aria-describedby="zone-note">
          <option value={LOCAL}>This computer: {localZone}</option>
          {#each zoneOptions as zone (zone)}
            <option value={zone}>{zone}</option>
          {/each}
        </select>
        <p class="note" id="zone-note">Each event keeps its own zone; this only changes how time is shown.</p>
      </div>
    </div>

    <div class="row">
      <div class="label-col"><span id="week-start-label">Week starts on</span></div>
      <div class="control-col">
        <div class="seg" role="group" aria-labelledby="week-start-label">
          <button type="button" aria-pressed={settings.week_starts_on === 'mon'} disabled={busy} onclick={() => setWeekStart('mon')}>Monday</button>
          <button type="button" aria-pressed={settings.week_starts_on === 'sun'} disabled={busy} onclick={() => setWeekStart('sun')}>Sunday</button>
        </div>
      </div>
    </div>

    <div class="row">
      <div class="label-col"><label for="duration">New events last</label></div>
      <div class="control-col">
        <select id="duration" value={String(settings.default_event_minutes)} disabled={busy} onchange={setDuration}>
          {#each durationOptions as minutes (minutes)}
            <option value={String(minutes)}>{minutes} minutes</option>
          {/each}
        </select>
      </div>
    </div>

    <div class="row">
      <div class="label-col"><label for="day-start">Day and week views open at</label></div>
      <div class="control-col">
        <select id="day-start" value={String(settings.day_start_hour)} disabled={busy} onchange={setDayStart}>
          {#each hours as hour (hour)}
            <option value={String(hour)}>{hourLabel(hour)}</option>
          {/each}
        </select>
      </div>
    </div>

    <div class="row">
      <div class="label-col"><span id="theme-label">Theme</span></div>
      <div class="control-col">
        <div class="seg" role="group" aria-labelledby="theme-label">
          {#each themes as option (option.value)}
            <button type="button" aria-pressed={settings.theme === option.value} disabled={busy} onclick={() => setTheme(option.value)}>{option.label}</button>
          {/each}
        </div>
      </div>
    </div>
  </section>

  <section class="card" aria-labelledby="reminder-heading">
    <h2 id="reminder-heading">Reminders</h2>

    <div class="row">
      <div class="label-col"><span class="field-name">Fixed events</span></div>
      <div class="control-col">
        <label class="check">
          <input type="checkbox" checked={reminderOn} disabled={busy} onchange={setReminders} />
          <span>Remind me before fixed events</span>
        </label>
        <div class="inline">
          <label for="lead">Lead time</label>
          <select id="lead" value={String(leadValue)} disabled={busy || !reminderOn} onchange={setLead}>
            {#each leadOptions as minutes (minutes)}
              <option value={String(minutes)}>{leadLabel(minutes)}</option>
            {/each}
          </select>
        </div>
        <p class="note">A reminder shows only the event title and time.</p>
      </div>
    </div>

    <div class="row">
      <div class="label-col"><span class="field-name">When closed</span></div>
      <div class="control-col">
        <label class="check">
          <input type="checkbox" checked={settings.keep_running_in_tray} disabled={busy} onchange={setTray} />
          <span>Keep running in the notification area when the window is closed, so reminders continue</span>
        </label>
        <label class="check">
          <input type="checkbox" checked={settings.open_at_login} disabled={busy || !desktop} onchange={setLogin} />
          <span>Open Temporal quietly in the notification area when I sign in</span>
        </label>
        {#if !desktop}<p class="note">Desktop app only.</p>{/if}
      </div>
    </div>
  </section>

  <section class="card" aria-labelledby="usual-heading">
    <h2 id="usual-heading">Usual availability</h2>
    <p class="lede">Temporal suggests work only inside time you say you're willing to use. An empty calendar is not free time.</p>

    <div class="presets" role="group" aria-label="Presets">
      {#each presets as preset (preset.label)}
        <button type="button" class="btn small" disabled={busy} onclick={() => applyPreset(preset.days, preset.start, preset.end)}>{preset.label}</button>
      {/each}
      <button type="button" class="btn small" disabled={busy} onclick={clearAll}>Clear all</button>
      <span class="hint">Presets replace the whole week until you save.</span>
    </div>

    <div class="days">
      {#each orderedDays as day (day)}
        <div class="day" role="group" aria-labelledby="usual-{day}">
          <div class="day-name" id="usual-{day}">{dayNames[day]}</div>
          <div class="day-body">
            {#each draft[day] as range (range.key)}
              <div class="range">
                <label class="time"><span>From</span><input type="time" step="900" bind:value={range.start} disabled={busy} /></label>
                <label class="time"><span>To</span><input type="time" step="900" bind:value={range.end} disabled={busy} /></label>
                {#if range.end === '00:00'}<span class="midnight">midnight</span>{/if}
                <button type="button" class="icon" aria-label="Remove {dayNames[day]} time" disabled={busy} onclick={() => removeRange(day, range.key)}>×</button>
              </div>
            {:else}
              <span class="none">Not available</span>
            {/each}
            <button type="button" class="btn small" disabled={busy} onclick={() => addRange(day)}>+ Add time</button>
          </div>
        </div>
      {/each}
    </div>

    <div class="glance" aria-hidden="true">
      <span class="glance-title">Week at a glance</span>
      {#each orderedDays as day (day)}
        <div class="glance-row">
          <span class="glance-day">{shortNames[day]}</span>
          <div class="bar">
            {#each barsFor(day) as bar, index (index)}
              <span class="block" style:left="{bar.left}%" style:width="{bar.width}%"></span>
            {/each}
          </div>
        </div>
      {/each}
      <div class="glance-axis"><span>12 AM</span><span>6 AM</span><span>12 PM</span><span>6 PM</span><span>12 AM</span></div>
    </div>

    <div class="fields">
      <div class="field">
        <label for="places">Places these hours usually happen</label>
        <input id="places" type="text" bind:value={placesText} placeholder="home, campus" autocomplete="off" disabled={busy} aria-describedby="places-hint" />
        <span class="hint" id="places-hint">Comma-separated, lowercase. Leave blank if unknown.</span>
      </div>
      <div class="field">
        <label for="energy">Energy in these hours</label>
        <select id="energy" bind:value={energyChoice} disabled={busy}>
          {#each energyOptions as option (option.value)}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
      </div>
    </div>

    <p class="problem" aria-live="polite">{problem ?? ''}</p>
    <div class="actions">
      <button type="button" class="btn primary" disabled={busy || !dirty || problem !== null} onclick={save}>Save usual availability</button>
      {#if personal.local.usual_availability}
        <button type="button" class="btn" disabled={busy} onclick={removeUsual}>Remove usual availability</button>
      {/if}
      {#if dirty}<span class="status" role="status">Unsaved changes</span>{/if}
    </div>
  </section>

  <section class="card" aria-labelledby="about-heading">
    <h2 id="about-heading">About and data</h2>
    <p>Everything stays on this computer. No account, no cloud, no telemetry.</p>

    <div class="row">
      <div class="label-col"><span class="field-name">Examples</span></div>
      <div class="control-col">
        <button type="button" class="btn" disabled={busy} onclick={onexamples}>Explore example weeks</button>
        <p class="note">Synthetic scenarios used to check how Horizon behaves.</p>
      </div>
    </div>

    <p class="version">Temporal Engine 0.1 · personal build</p>
  </section>
</div>

<style>
  .settings {
    max-width: 920px;
    margin: 0 auto;
    padding: 24px 16px 48px;
    color: var(--ink);
    font-family: var(--font);
    font-size: 14px;
    line-height: 1.45;
  }
  h1 {
    font-family: var(--font-display);
    font-size: 28px;
    font-weight: 600;
    margin: 0 0 4px;
  }
  .lede { margin: 0 0 14px; color: var(--muted); }
  .card {
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    padding: 20px;
    margin: 16px 0 0;
  }
  .card h2 {
    font-family: var(--font-display);
    font-size: 18px;
    font-weight: 600;
    margin: 0 0 8px;
  }
  .row {
    display: grid;
    grid-template-columns: 200px minmax(0, 1fr);
    gap: 8px 24px;
    padding: 14px 0;
    border-top: 1px solid var(--line);
    align-items: start;
  }
  h2 + .row { border-top: none; padding-top: 6px; }
  .label-col { padding-top: 6px; font-weight: 600; }
  .label-col label { font-weight: 600; }
  .field-name { font-weight: 600; }
  .control-col { min-width: 0; }
  .note, .hint { margin: 6px 0 0; font-size: 13px; color: var(--muted); }
  .hint { display: block; }
  .version { margin: 16px 0 0; font-size: 12px; color: var(--faint); }
  .inline {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 10px;
  }
  .inline label { color: var(--muted); }

  select, input[type='text'], input[type='time'] {
    font: inherit;
    color: var(--ink);
    background: var(--surface);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    padding: 6px 8px;
    min-height: 32px;
    box-sizing: border-box;
  }
  input[type='time'] { width: 7.5rem; }
  input[type='text'] { width: 100%; }

  .btn {
    font: inherit;
    color: var(--ink);
    background: var(--surface);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    padding: 6px 12px;
    cursor: pointer;
  }
  .btn.primary {
    background: var(--accent);
    color: var(--accent-ink);
    border-color: var(--accent);
    font-weight: 600;
  }
  .btn.small { padding: 4px 10px; font-size: 13px; }
  .btn:disabled, select:disabled, input:disabled { opacity: 0.5; cursor: not-allowed; }
  :is(button, input, select):focus-visible { outline: 2px solid var(--accent); outline-offset: 2px; }

  .seg {
    display: inline-flex;
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  .seg button {
    font: inherit;
    color: var(--ink);
    background: var(--surface);
    border: 0;
    padding: 6px 14px;
    cursor: pointer;
  }
  .seg button + button { border-left: 1px solid var(--line-strong); }
  .seg button[aria-pressed='true'] { background: var(--accent-soft); font-weight: 600; }
  .seg button:disabled { opacity: 0.5; cursor: not-allowed; }

  .check {
    display: flex;
    gap: 8px;
    align-items: flex-start;
    cursor: pointer;
  }
  .check input { margin-top: 3px; accent-color: var(--accent); }

  .presets {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    align-items: center;
    margin-bottom: 10px;
  }
  .presets .hint { margin: 0 0 0 4px; }

  .days {
    display: grid;
    border: 1px solid var(--line);
    border-radius: var(--radius-sm);
    overflow: hidden;
  }
  .day {
    display: grid;
    grid-template-columns: 120px minmax(0, 1fr);
    gap: 12px;
    padding: 10px 12px;
    background: var(--surface);
  }
  .day + .day { border-top: 1px solid var(--line); }
  .day-name { font-weight: 600; padding-top: 6px; }
  .day-body {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 8px;
    min-width: 0;
  }
  .range {
    display: inline-flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
  }
  .time {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    font-size: 13px;
    color: var(--muted);
  }
  .midnight { font-size: 12px; color: var(--muted); }
  .none { color: var(--faint); padding-top: 6px; }
  .icon {
    font: inherit;
    line-height: 1;
    width: 28px;
    height: 28px;
    padding: 0;
    color: var(--muted);
    background: var(--surface);
    border: 1px solid var(--line-strong);
    border-radius: var(--radius-sm);
    cursor: pointer;
  }

  .glance {
    display: grid;
    gap: 6px;
    margin-top: 14px;
  }
  .glance-title {
    font-size: 12px;
    color: var(--faint);
  }
  .glance-row {
    display: grid;
    grid-template-columns: 40px minmax(0, 1fr);
    align-items: center;
    gap: 10px;
  }
  .glance-day { font-size: 12px; color: var(--muted); }
  .bar {
    position: relative;
    height: 16px;
    background: var(--surface);
    background-image: linear-gradient(to right, var(--line) 1px, transparent 1px);
    background-size: 25% 100%;
    border: 1px solid var(--line);
    border-radius: 3px;
  }
  .block {
    position: absolute;
    top: 0;
    bottom: 0;
    box-sizing: border-box;
    background: var(--avail);
    border: 1px solid var(--avail-line);
    border-radius: 2px;
  }
  .glance-axis {
    display: flex;
    justify-content: space-between;
    padding-left: 50px;
    font-size: 11px;
    color: var(--faint);
  }

  .fields {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 12px 24px;
    margin-top: 16px;
  }
  .field { display: grid; gap: 4px; align-content: start; }
  .field label { font-weight: 600; }

  .problem {
    min-height: 1.3em;
    margin: 10px 0 0;
    font-size: 13px;
    color: var(--risk);
  }
  .actions {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
    margin-top: 6px;
  }
  .status { font-size: 13px; color: var(--muted); }

  @media (max-width: 640px) {
    .row { grid-template-columns: 1fr; }
    .label-col { padding-top: 0; }
    .day { grid-template-columns: 1fr; gap: 6px; }
    .day-name { padding-top: 0; }
  }
</style>
