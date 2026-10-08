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
    { value: 'light', label: 'Paper' },
    { value: 'dark', label: 'Ink' },
    { value: 'rose', label: 'Rose' },
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

  // Track scale: hour ticks every three hours, labelled under the first row.
  const ticks = [0, 3, 6, 9, 12, 15, 18, 21];
  const pad2 = (n: number) => String(n).padStart(2, '0');

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

  // Declared hours: the union of each day's valid ranges, so an overlap is never counted twice.
  function declaredMinutes(source: Draft): number {
    let total = 0;
    for (const day of weekdays) {
      let covered = 0;
      let edge = -1;
      for (const span of spansOf(source[day]).filter(s => s.end > s.start).sort((a, b) => a.start - b.start)) {
        const from = Math.max(span.start, edge);
        if (span.end > from) {
          covered += span.end - from;
          edge = span.end;
        }
      }
      total += covered;
    }
    return total;
  }
  const hoursText = (minutes: number) => (minutes % 60 === 0 ? `${minutes / 60} h` : `${(minutes / 60).toFixed(1)} h`);
  const declaredText = $derived(hoursText(declaredMinutes(draft)));

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
  <section class="headline">
    <h1>Settings</h1>
    <div class="aside">Stored on this computer<br />No account · No cloud · No telemetry</div>
  </section>

  <section class="panel" aria-labelledby="display-heading">
    <div class="section-bar"><span class="index">01</span><h2 class="bar-title" id="display-heading">Time and display</h2></div>

    <div class="row">
      <div class="name-cell">
        <label class="name" for="zone">Time zone</label>
        <p class="desc" id="zone-note">Each event keeps its own zone; this only changes how time is shown.</p>
      </div>
      <div class="control">
        <select id="zone" value={zoneValue} disabled={busy} onchange={setZone} aria-describedby="zone-note">
          <option value={LOCAL}>This computer: {localZone}</option>
          {#each zoneOptions as zone (zone)}
            <option value={zone}>{zone}</option>
          {/each}
        </select>
      </div>
    </div>

    <div class="row">
      <div class="name-cell"><span class="name" id="week-start-label">Week starts on</span></div>
      <div class="control">
        <div class="seg" role="group" aria-labelledby="week-start-label">
          <button type="button" aria-pressed={settings.week_starts_on === 'mon'} disabled={busy} onclick={() => setWeekStart('mon')}>Monday</button>
          <button type="button" aria-pressed={settings.week_starts_on === 'sun'} disabled={busy} onclick={() => setWeekStart('sun')}>Sunday</button>
        </div>
      </div>
    </div>

    <div class="row">
      <div class="name-cell"><label class="name" for="duration">New events last</label></div>
      <div class="control">
        <select id="duration" value={String(settings.default_event_minutes)} disabled={busy} onchange={setDuration}>
          {#each durationOptions as minutes (minutes)}
            <option value={String(minutes)}>{minutes} minutes</option>
          {/each}
        </select>
      </div>
    </div>

    <div class="row">
      <div class="name-cell"><label class="name" for="day-start">Day and week views open at</label></div>
      <div class="control">
        <select id="day-start" value={String(settings.day_start_hour)} disabled={busy} onchange={setDayStart}>
          {#each hours as hour (hour)}
            <option value={String(hour)}>{hourLabel(hour)}</option>
          {/each}
        </select>
      </div>
    </div>

    <div class="row">
      <div class="name-cell">
        <span class="name" id="theme-label">Theme</span>
        <p class="desc">System follows this computer: Ink when it is dark, Paper otherwise.</p>
      </div>
      <div class="control">
        <div class="seg" role="group" aria-labelledby="theme-label">
          {#each themes as option (option.value)}
            <button type="button" aria-pressed={settings.theme === option.value} disabled={busy} onclick={() => setTheme(option.value)}>{option.label}</button>
          {/each}
        </div>
      </div>
    </div>
  </section>

  <section class="panel" aria-labelledby="reminder-heading">
    <div class="section-bar"><span class="index">02</span><h2 class="bar-title" id="reminder-heading">Reminders</h2></div>

    <div class="row">
      <div class="name-cell">
        <span class="name">Fixed events</span>
        <p class="desc">A reminder shows only the event title and time.</p>
      </div>
      <div class="control">
        <label class="check">
          <input type="checkbox" checked={reminderOn} disabled={busy} onchange={setReminders} />
          <span>Remind me before fixed events</span>
        </label>
        <div class="inline">
          <label class="caption" for="lead">Lead time</label>
          <select id="lead" value={String(leadValue)} disabled={busy || !reminderOn} onchange={setLead}>
            {#each leadOptions as minutes (minutes)}
              <option value={String(minutes)}>{leadLabel(minutes)}</option>
            {/each}
          </select>
        </div>
      </div>
    </div>

    <div class="row">
      <div class="name-cell"><span class="name">When closed</span></div>
      <div class="control">
        <label class="check">
          <input type="checkbox" checked={settings.keep_running_in_tray} disabled={busy} onchange={setTray} />
          <span>Keep running in the notification area when the window is closed, so reminders continue</span>
        </label>
        <label class="check">
          <input type="checkbox" checked={settings.open_at_login} disabled={busy || !desktop} onchange={setLogin} />
          <span>Open Temporal quietly in the notification area when I sign in</span>
        </label>
        {#if !desktop}<p class="desc">Desktop app only.</p>{/if}
      </div>
    </div>
  </section>

  <section class="panel" aria-labelledby="usual-heading">
    <div class="section-bar">
      <span class="index">03</span>
      <h2 class="bar-title" id="usual-heading">Usual availability</h2>
      <span class="spacer"></span>
      <span class="label">Declared hours / week: {declaredText}</span>
    </div>
    <p class="lede">Temporal suggests work only inside time you say you're willing to use. An empty calendar is not free time.</p>

    <div class="presets" role="group" aria-label="Presets">
      {#each presets as preset (preset.label)}
        <button type="button" disabled={busy} onclick={() => applyPreset(preset.days, preset.start, preset.end)}>{preset.label}</button>
      {/each}
      <button type="button" disabled={busy} onclick={clearAll}>Clear all</button>
      <span class="caption">Presets replace the whole week until you save.</span>
    </div>

    {#each orderedDays as day, index (day)}
      <div class="day" role="group" aria-labelledby="usual-{day}">
        <span class="day-name" aria-hidden="true">{shortNames[day]}</span>
        <span class="sr-only" id="usual-{day}">{dayNames[day]}</span>

        <div class="track-cell">
          <div class="track" aria-hidden="true">
            {#each ticks as hour (hour)}
              <span class="tick" style:left="{(hour / 24) * 100}%"></span>
            {/each}
            {#each barsFor(day) as bar, n (n)}
              <span class="block" style:left="{bar.left}%" style:width="{bar.width}%"></span>
            {/each}
          </div>
          {#if index === 0}
            <div class="axis" aria-hidden="true">
              {#each ticks as hour (hour)}
                <span style:left="{(hour / 24) * 100}%">{pad2(hour)}</span>
              {/each}
            </div>
          {/if}
        </div>

        <div class="editor">
          {#each draft[day] as range (range.key)}
            <div class="range">
              <label class="time"><span class="sr-only">From</span><input type="time" step="900" bind:value={range.start} disabled={busy} /></label>
              <span class="dash" aria-hidden="true">–</span>
              <label class="time"><span class="sr-only">To</span><input type="time" step="900" bind:value={range.end} disabled={busy} /></label>
              {#if range.end === '00:00'}<span class="midnight">midnight</span>{/if}
              <button type="button" class="ghost x" aria-label="Remove {dayNames[day]} time" disabled={busy} onclick={() => removeRange(day, range.key)}>×</button>
            </div>
          {:else}
            <span class="caption">Not available</span>
          {/each}
          <button type="button" disabled={busy} onclick={() => addRange(day)}>+ Add</button>
        </div>
      </div>
    {/each}

    <div class="row">
      <div class="name-cell">
        <label class="name" for="places">Places these hours usually happen</label>
        <p class="desc" id="places-hint">Comma-separated, lowercase. Leave blank if unknown.</p>
      </div>
      <div class="control">
        <input id="places" type="text" bind:value={placesText} placeholder="home, campus" autocomplete="off" disabled={busy} aria-describedby="places-hint" />
      </div>
    </div>

    <div class="row">
      <div class="name-cell"><label class="name" for="energy">Energy in these hours</label></div>
      <div class="control">
        <select id="energy" bind:value={energyChoice} disabled={busy}>
          {#each energyOptions as option (option.value)}
            <option value={option.value}>{option.label}</option>
          {/each}
        </select>
      </div>
    </div>

    <div class="actions">
      <p class="problem" aria-live="polite">{problem ?? ''}</p>
      <div class="buttons">
        <button type="button" class="primary" disabled={busy || !dirty || problem !== null} onclick={save}>Save usual availability →</button>
        {#if personal.local.usual_availability}
          <button type="button" disabled={busy} onclick={removeUsual}>Remove usual availability</button>
        {/if}
        {#if dirty}<span class="unsaved" role="status">Unsaved changes</span>{/if}
      </div>
    </div>
  </section>

  <section class="panel" aria-labelledby="about-heading">
    <div class="section-bar"><span class="index">04</span><h2 class="bar-title" id="about-heading">About and data</h2></div>

    <dl>
      <div class="row">
        <dt class="name">Data</dt>
        <dd class="value">Everything stays on this computer. No account, no cloud, no telemetry.</dd>
      </div>
      <div class="row">
        <dt class="name">Version</dt>
        <dd class="value">Temporal Engine 0.1 · personal build</dd>
      </div>
      <div class="row">
        <dt class="name">Examples</dt>
        <dd class="value">
          <button type="button" disabled={busy} onclick={onexamples}>Explore example weeks →</button>
          <p class="desc">Synthetic scenarios used to check how Horizon behaves.</p>
        </dd>
      </div>
    </dl>
  </section>
</div>

<style>
  .settings {
    padding-bottom: 48px;
    color: var(--ink);
    font-size: 13px;
    line-height: 1.45;
  }

  /* Visually hidden, but still the accessible name of each time input. */
  .sr-only {
    position: absolute;
    width: 1px;
    height: 1px;
    margin: -1px;
    padding: 0;
    overflow: hidden;
    clip: rect(0 0 0 0);
    white-space: nowrap;
    border: 0;
  }

  /* Section titles sit in the global section bar; mono caps, not the display face. */
  .bar-title {
    margin: 0;
    font-family: var(--font-mono);
    font-stretch: 100%;
    font-weight: 400;
    font-size: 11px;
    letter-spacing: 0.07em;
    line-height: inherit;
    text-transform: uppercase;
  }

  /* Sections are full-bleed and separated by rules. */
  .panel { border-bottom: var(--rule); }
  .lede {
    padding: 14px 28px;
    border-bottom: 1px solid var(--line);
    font-size: 13px;
    line-height: 1.6;
    color: var(--muted);
  }

  /* Setting rows: name on the left, control on the right. */
  .row {
    display: grid;
    grid-template-columns: 260px minmax(0, 1fr);
    column-gap: 24px;
    align-items: start;
    padding: 14px 28px;
    border-bottom: 1px solid var(--line);
  }
  .row:last-child { border-bottom: 0; }
  .row > :first-child { padding-top: 8px; }
  .name {
    display: block;
    font-family: var(--font-mono);
    font-size: 11px;
    font-weight: 400;
    letter-spacing: 0.06em;
    line-height: 1.45;
    text-transform: uppercase;
    color: var(--ink);
  }
  .desc {
    margin: 6px 0 0;
    font-size: 12px;
    line-height: 1.5;
    color: var(--muted);
  }
  .caption {
    font-family: var(--font-mono);
    font-size: 11px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--faint);
  }
  .control {
    display: grid;
    gap: 10px;
    justify-items: start;
    min-width: 0;
  }
  .check {
    display: flex;
    align-items: center;
    gap: 10px;
    font-size: 13px;
    cursor: pointer;
  }
  .inline {
    display: flex;
    align-items: center;
    gap: 10px;
  }
  .inline .caption { color: var(--muted); }
  .control input[type='text'] {
    width: 100%;
    max-width: 440px;
  }

  /* Segmented controls: joined buttons on one ink rule; the pressed one inverts. */
  .seg { display: inline-flex; }
  .seg button {
    position: relative;
    padding: 7px 14px;
  }
  .seg button + button { margin-left: -1px; }
  .seg button[aria-pressed='true'] {
    z-index: 1;
    background: var(--invert-bg);
    color: var(--invert-ink);
  }

  /* Usual availability. */
  .presets {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
    padding: 12px 28px;
    border-bottom: 1px solid var(--line);
  }
  .presets button { padding: 5px 9px; }
  .presets .caption { margin-left: 8px; }

  .day {
    position: relative;
    display: grid;
    grid-template-columns: 76px minmax(0, 1fr) minmax(0, 380px);
    column-gap: 24px;
    align-items: start;
    padding: 12px 28px;
    border-bottom: 1px solid var(--line);
  }
  .day-name {
    display: flex;
    align-items: center;
    height: 26px;
    font-family: var(--font-display);
    font-stretch: 115%;
    font-weight: 760;
    font-size: 18px;
    line-height: 1;
    text-transform: uppercase;
  }
  .track-cell {
    display: grid;
    gap: 4px;
    min-width: 0;
  }
  .track {
    position: relative;
    height: 26px;
    border: var(--rule);
    background: var(--closed);
  }
  .tick {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 1px;
    background: var(--line);
  }
  .block {
    position: absolute;
    top: 0;
    bottom: 0;
    background: var(--invert-bg);
  }
  .axis {
    position: relative;
    height: 12px;
    font-family: var(--font-mono);
    font-size: 9.5px;
    line-height: 12px;
    letter-spacing: 0.04em;
    color: var(--muted);
  }
  .axis span {
    position: absolute;
    top: 0;
    transform: translateX(-50%);
    white-space: nowrap;
  }
  .axis span:first-child { transform: none; }

  .editor {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px 12px;
    min-width: 0;
    min-height: 26px;
  }
  .range {
    display: inline-flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 6px;
  }
  .time {
    position: relative;
    display: inline-flex;
    align-items: center;
  }
  .settings .editor input[type='time'] {
    width: 6.5rem;
    height: 26px;
    padding: 0 6px;
  }
  .dash {
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--muted);
  }
  .midnight {
    font-family: var(--font-mono);
    font-size: 10.5px;
    letter-spacing: 0.06em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .x {
    width: 26px;
    height: 26px;
    padding: 0;
    font-size: 14px;
    line-height: 1;
  }

  /* Save bar: validation and unsaved messages in mono. */
  .actions {
    display: grid;
    gap: 10px;
    padding: 14px 28px 16px;
  }
  .buttons {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px;
  }
  .problem {
    min-height: 1.3em;
    font-family: var(--font-mono);
    font-size: 12px;
    line-height: 1.3;
    color: var(--risk);
  }
  .unsaved {
    display: inline-flex;
    align-items: center;
    gap: 7px;
    font-family: var(--font-mono);
    font-size: 12px;
    color: var(--muted);
  }
  .unsaved::before {
    content: '';
    width: 6px;
    height: 6px;
    background: var(--accent);
  }

  /* About and data: key / value spec rows. */
  .value {
    display: grid;
    gap: 6px;
    justify-items: start;
    margin: 0;
    padding-top: 7px;
    font-size: 13px;
    line-height: 1.5;
    color: var(--ink);
    overflow-wrap: anywhere;
  }

  @media (max-width: 760px) {
    .lede, .presets { padding-left: 16px; padding-right: 16px; }
    .row {
      grid-template-columns: minmax(0, 1fr);
      row-gap: 10px;
      padding: 14px 16px;
    }
    .row > :first-child { padding-top: 0; }
    .value { padding-top: 0; }
    .day {
      grid-template-columns: minmax(0, 1fr);
      row-gap: 8px;
      padding: 12px 16px;
    }
    .actions { padding-left: 16px; padding-right: 16px; }
  }
</style>
