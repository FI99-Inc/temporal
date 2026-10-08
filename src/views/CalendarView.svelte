<script lang="ts">
  import MonthView from '../calendar/MonthView.svelte';
  import WeekView from '../calendar/WeekView.svelte';
  import ItemDetail from '../components/ItemDetail.svelte';
  import { calendarRange, type PersonalView } from '../lib/api.ts';
  import type { CalendarRange } from '../lib/calendar-types.ts';
  import { addDays, formatDate, formatMonth, monthGrid, weekDays, zonedParts } from '../lib/calendar.ts';
  import { blankDraft, hhmm, minutesOf, type EditTarget, type EventDraft } from '../lib/editor.ts';
  import type { LocalMutation } from '../lib/local.ts';

  type Mode = 'day' | 'week' | 'month';
  let { personal, busy, version, now, onedit, onapply }: {
    personal: PersonalView;
    busy: boolean;
    /** Changes whenever stored data may have changed, so the range reloads. */
    version: number;
    now: number;
    onedit: (target: EditTarget, draft: EventDraft) => void;
    onapply: (label: string, mutations: LocalMutation[]) => Promise<boolean>;
  } = $props();

  const zone = $derived(personal.view.zone);
  const settings = $derived(personal.settings);
  const today = $derived(zonedParts(now, zone).date);
  let mode = $state<Mode>(readMode());
  let anchor = $state<string>('');
  let range = $state<CalendarRange | null>(null);
  let error = $state('');
  let selectedId = $state<string | null>(null);
  let request = 0;

  function readMode(): Mode {
    try { const saved = localStorage.getItem('temporal.calendar.mode'); if (saved === 'day' || saved === 'week' || saved === 'month') return saved; } catch { /* storage unavailable */ }
    return 'week';
  }
  $effect(() => { try { localStorage.setItem('temporal.calendar.mode', mode); } catch { /* storage unavailable */ } });
  $effect(() => { if (!anchor) anchor = today; });

  const days = $derived(!anchor ? [] : mode === 'day' ? [anchor] : weekDays(anchor, settings.week_starts_on));
  const month = $derived(anchor.slice(0, 7));
  const grid = $derived(anchor ? monthGrid(month, settings.week_starts_on) : []);
  const span = $derived(mode === 'month' ? [grid[0], grid[grid.length - 1]] : [days[0], days[days.length - 1]]);
  const label = $derived(!anchor ? '' : mode === 'month' ? formatMonth(month)
    : mode === 'day' ? formatDate(anchor, 'long')
    : `${formatDate(days[0], 'short')} – ${formatDate(days[6], 'short')} · ${formatMonth(days[3].slice(0, 7))}`);

  // Fetch a padded UTC range; the views place rows by civil date in the display zone.
  $effect(() => {
    const [first, last] = span;
    void version;
    if (!first || !last) return;
    const from = Date.parse(`${first}T00:00:00Z`) - 36 * 3_600_000;
    const to = Date.parse(`${addDays(last, 1)}T00:00:00Z`) + 36 * 3_600_000;
    const mine = ++request;
    calendarRange(from, to)
      .then(next => { if (mine === request) { range = next; error = ''; } })
      .catch(e => { if (mine === request) error = String(e instanceof Error ? e.message : e); });
  });

  const selectedEvent = $derived(range?.events.find(e => e.id === selectedId) ?? null);
  const selectedItem = $derived(selectedId ? personal.view.items.find(i => i.id === selectedId) ?? null : null);

  function step(direction: number) {
    if (mode === 'day') anchor = addDays(anchor, direction);
    else if (mode === 'week') anchor = addDays(anchor, 7 * direction);
    else {
      const [y, m] = anchor.split('-').map(Number);
      const next = new Date(Date.UTC(y, m - 1 + direction, 1));
      anchor = next.toISOString().slice(0, 10);
    }
  }
  function createTimed(draft: { start: string; end: string }) {
    const date = draft.start.slice(0, 10);
    const start = draft.start.slice(11, 16);
    let minutes = minutesOf(draft.end.slice(11, 16)) - minutesOf(start);
    if (draft.end.slice(0, 10) !== date) minutes += 1440;
    if (minutes === 60) minutes = settings.default_event_minutes;
    onedit({ kind: 'new' }, blankDraft(date, zone, start, Math.max(15, minutes)));
  }
  function createAllDay(date: string) {
    const draft = blankDraft(date, zone);
    draft.allDay = true;
    onedit({ kind: 'new' }, draft);
  }
  function newEvent() {
    const parts = zonedParts(now, zone);
    const start = anchor === today ? hhmm(Math.min(23 * 60, Math.ceil((parts.minutes + 1) / 30) * 30)) : '09:00';
    onedit({ kind: 'new' }, blankDraft(anchor || today, zone, start, settings.default_event_minutes));
  }
  function keydown(event: KeyboardEvent) {
    const target = event.target as HTMLElement | null;
    if (event.defaultPrevented || event.ctrlKey || event.metaKey || event.altKey) return;
    if (target && (target.closest('input, textarea, select, dialog') || target.isContentEditable)) return;
    const keys: Record<string, () => void> = {
      t: () => { anchor = today; }, ArrowLeft: () => step(-1), ArrowRight: () => step(1), j: () => step(1), k: () => step(-1),
      d: () => { mode = 'day'; }, w: () => { mode = 'week'; }, m: () => { mode = 'month'; },
      Escape: () => { selectedId = null; },
    };
    const action = keys[event.key];
    if (action) { event.preventDefault(); action(); }
  }
</script>

<svelte:window onkeydown={keydown} />

<div class="calendar-page">
  <div class="toolbar">
    <button onclick={() => { anchor = today; }} title="Today (T)">Today</button>
    <div class="nav">
      <button class="ghost icon" aria-label="Previous" title="Previous (←)" onclick={() => step(-1)}>‹</button>
      <button class="ghost icon" aria-label="Next" title="Next (→)" onclick={() => step(1)}>›</button>
    </div>
    <h1>{label}</h1>
    <span class="spacer"></span>
    <div class="segmented" role="group" aria-label="Calendar view">
      {#each [['day', 'Day', 'D'], ['week', 'Week', 'W'], ['month', 'Month', 'M']] as [value, text, key]}
        <button aria-pressed={mode === value} title="{text} ({key})" onclick={() => { mode = value as Mode; }}>{text}</button>
      {/each}
    </div>
    <button class="primary" disabled={busy} onclick={newEvent}>New event</button>
  </div>

  {#if error}<div class="notice warn" role="alert"><p>{error}</p></div>{/if}

  <div class="body" class:with-detail={selectedId && (selectedEvent || selectedItem)}>
    <div class="surface">
      {#if range && anchor}
        {#if mode === 'month'}
          <MonthView {month} weekStartsOn={settings.week_starts_on} {zone} {now} events={range.events} {selectedId}
            onselect={(id) => { selectedId = id; }} onpickday={(date) => { anchor = date; mode = 'day'; }} oncreateallday={createAllDay} />
        {:else}
          <WeekView {days} {zone} {now} events={range.events} availability={range.availability} {selectedId}
            scrollToHour={settings.day_start_hour}
            onselect={(id) => { selectedId = id; }} oncreate={createTimed} oncreateallday={createAllDay} />
        {/if}
      {:else if !error}
        <div class="loading" role="status">Loading your calendar…</div>
      {/if}
    </div>
    {#if selectedId && (selectedEvent || selectedItem)}
      <aside class="detail" aria-label="Selected item details">
        <ItemDetail event={selectedEvent} item={selectedItem} {personal} {busy} {onedit} {onapply} onclose={() => { selectedId = null; }} />
      </aside>
    {/if}
  </div>
</div>

<style>
  .calendar-page { display: flex; flex-direction: column; height: 100%; min-height: 0; }
  .toolbar { display: flex; align-items: center; gap: 10px; padding: 12px 20px; border-bottom: 1px solid var(--line); background: var(--surface); flex-wrap: wrap; }
  .toolbar h1 { font-size: 19px; letter-spacing: -.2px; }
  .nav { display: flex; }
  .icon { font-size: 20px; line-height: 1; padding: 2px 10px; }
  .spacer { flex: 1; }
  .segmented { display: inline-flex; border: 1px solid var(--line-strong); border-radius: var(--radius-sm); overflow: hidden; }
  .segmented button { border: 0; border-radius: 0; padding: 5px 13px; font-size: 13px; background: var(--surface); }
  .segmented button + button { border-left: 1px solid var(--line-strong); }
  .segmented button[aria-pressed='true'] { background: var(--accent-soft); font-weight: 600; }
  .body { flex: 1; min-height: 0; display: grid; grid-template-columns: minmax(0, 1fr); }
  .body.with-detail { grid-template-columns: minmax(0, 1fr) 330px; }
  .surface { min-height: 0; min-width: 0; height: 100%; overflow: hidden; background: var(--surface); }
  .detail { border-left: 1px solid var(--line); padding: 16px 18px; overflow-y: auto; background: var(--bg); }
  .notice { margin: 10px 20px 0; }
  @media (max-width: 1100px) { .body.with-detail { grid-template-columns: minmax(0, 1fr) 280px; } }
  @media (max-width: 760px) { .body.with-detail { grid-template-columns: minmax(0, 1fr); grid-template-rows: minmax(0, 1fr) auto; } .detail { border-left: 0; border-top: 1px solid var(--line); max-height: 45vh; } }
</style>
