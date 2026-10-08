<script lang="ts">
  import { untrack } from 'svelte';
  import type { AvailabilitySpan, CalendarEvent } from '../lib/calendar-types.ts';
  import { allDayFor, dayDeadlines, dayRange, formatDate, formatTime, layoutDay, minutesLabel, snapMinutes, wallToString, zonedParts, type Segment } from '../lib/calendar.ts';

  let { days, zone, now, events, availability, selectedId, scrollToHour = 7, onselect, oncreate, oncreateallday }: {
    days: string[];
    zone: string;
    now: number;
    events: CalendarEvent[];
    availability: AvailabilitySpan[];
    selectedId: string | null;
    scrollToHour?: number;
    onselect: (id: string) => void;
    oncreate: (draft: { start: string; end: string }) => void;
    oncreateallday: (date: string) => void;
  } = $props();

  const PX_PER_MINUTE = 0.8;
  const HOUR_PX = 48;
  const GRID_PX = 24 * HOUR_PX;
  const DAY_MINUTES = 1440;
  const DRAFT_MINUTES = 60;
  const SNAP_MINUTES = 15;
  const ALL_DAY_LIMIT = 4;
  const DEADLINE_STACK_PX = 20;
  const hours = Array.from({ length: 24 }, (_, hour) => hour);
  const GLYPH: Record<CalendarEvent['kind'], string> = { anchor: '▮', deadline: '◆', intention: '○', routine: '○' };
  const KIND_WORD: Record<CalendarEvent['kind'], string> = { anchor: 'Fixed', deadline: 'Deadline', intention: 'Intention', routine: 'Routine' };

  let scroller = $state<HTMLElement>();
  let expandAllDay = $state(false);
  let drag = $state<{ date: string; from: number; to: number } | null>(null);
  // Pointer bookkeeping is not rendered, so it stays out of reactive state.
  let pointer: { date: string; anchor: number; column: HTMLDivElement } | null = null;

  const today = $derived(zonedParts(now, zone));
  const todayTop = $derived(today.minutes * PX_PER_MINUTE);
  const daysKey = $derived(days.join('|'));
  const todayVisible = $derived(days.includes(today.date));
  const columns = $derived(days.map((date) => {
    const deadlines = dayDeadlines(events, date, zone);
    return {
      date,
      head: dayHead(date),
      segments: layoutDay(events, date, zone),
      deadlines: deadlines.map((item, index) => ({
        ...item,
        stack: deadlines.slice(0, index).filter((earlier) => earlier.minutes === item.minutes).length,
      })),
      allDay: allDayFor(events, date, zone),
      bands: availability.flatMap((span) => {
        const range = dayRange(span.start, span.end, date, zone);
        return range && range.bottom > range.top ? [range] : [];
      }),
    };
  }));
  const anyAllDayOverflow = $derived(columns.some((column) => column.allDay.length > ALL_DAY_LIMIT));

  function dayHead(date: string): { weekday: string; number: string } {
    const [weekday, number] = formatDate(date, 'short').split(' ');
    return { weekday, number };
  }

  // Scroll to the chosen hour, or to an hour before now when today is visible and
  // later than that hour. Keyed on the visible days, so minute ticks do not jump the view.
  $effect(() => {
    void daysKey;
    const element = scroller;
    if (!element) return;
    untrack(() => {
      let topMinute = scrollToHour * 60;
      if (todayVisible && today.minutes > topMinute) topMinute = today.minutes - 60;
      element.scrollTop = Math.max(0, topMinute) * PX_PER_MINUTE;
    });
  });

  function riskWord(event: CalendarEvent): string {
    if (event.state === 'overdue') return 'Overdue';
    if (event.risk === 'insufficient') return 'Not enough time';
    if (event.risk === 'tight') return 'Tight';
    return '';
  }

  function riskSuffix(event: CalendarEvent): string {
    const word = riskWord(event);
    return word ? ` · ${word}` : '';
  }

  // Imported calendars may carry a colour. Only a strict hex value is accepted.
  function edgeVar(color: string | null): string {
    return color !== null && /^#[0-9a-fA-F]{6}$/.test(color) ? `--edge-color:${color}` : '';
  }

  function rangeLabel(event: CalendarEvent): string {
    const from = zonedParts(event.start, zone);
    const to = zonedParts(event.end, zone);
    const crosses = from.date !== to.date;
    const left = `${crosses ? `${formatDate(from.date, 'short')}, ` : ''}${minutesLabel(from.minutes)}`;
    const right = `${crosses ? `${formatDate(to.date, 'short')}, ` : ''}${minutesLabel(to.minutes)}`;
    return `${left} to ${right}`;
  }

  function describe(event: CalendarEvent, date: string, when: string): string {
    const parts = [event.title, formatDate(date, 'long'), when, KIND_WORD[event.kind]];
    const risk = riskWord(event);
    if (risk) parts.push(risk);
    if (event.tentative) parts.push('tentative');
    if (event.occupancy === 'transparent') parts.push('does not block time');
    if (!event.editable) parts.push(event.source_label, 'read-only');
    return parts.join(', ');
  }

  function softPrefix(event: CalendarEvent): string {
    return event.kind === 'intention' || event.kind === 'routine' ? '○ ' : '';
  }

  function blockStyle(segment: Segment): string {
    const width = 100 / segment.columns;
    return [
      `top:${segment.top * PX_PER_MINUTE}px`,
      `height:${(segment.bottom - segment.top) * PX_PER_MINUTE}px`,
      `left:calc(${segment.column * width}% + 2px)`,
      `width:calc(${width}% - 4px)`,
      edgeVar(segment.event.color),
    ].filter(Boolean).join(';');
  }

  function hourLabel(hour: number): string {
    return `${hour % 12 || 12} ${hour < 12 ? 'AM' : 'PM'}`;
  }

  function stopPointer(event: PointerEvent) {
    event.stopPropagation();
  }

  // Snapped minute under the pointer, clamped to the day.
  function minuteAt(column: HTMLElement, clientY: number): number {
    const rect = column.getBoundingClientRect();
    return Math.min(DAY_MINUTES, Math.max(0, snapMinutes((clientY - rect.top) / PX_PER_MINUTE, SNAP_MINUTES)));
  }

  // A press without movement is a 60-minute draft. A drag keeps its snapped span.
  function draftRange(from: number, to: number): [number, number] {
    const start = Math.min(from, to);
    const end = Math.max(from, to);
    return end === start ? [start, Math.min(DAY_MINUTES, start + DRAFT_MINUTES)] : [start, end];
  }

  function trackDrag(event: PointerEvent) {
    if (!pointer) return;
    drag = { date: pointer.date, from: pointer.anchor, to: minuteAt(pointer.column, event.clientY) };
  }

  function detachDrag() {
    window.removeEventListener('pointermove', trackDrag);
    window.removeEventListener('pointerup', finishDrag);
    window.removeEventListener('pointercancel', cancelDrag);
  }

  function cancelDrag() {
    detachDrag();
    pointer = null;
    drag = null;
  }

  function finishDrag() {
    const started = pointer;
    const current = drag;
    cancelDrag();
    if (!started || !current) return;
    const [start, end] = draftRange(current.from, current.to);
    oncreate({ start: wallToString(started.date, start), end: wallToString(started.date, end) });
  }

  function beginDrag(event: PointerEvent, date: string) {
    if (event.button !== 0) return;
    const column = event.currentTarget as HTMLDivElement;
    // Anchor at most one step before midnight so a plain press can still make a draft.
    const anchor = Math.min(DAY_MINUTES - SNAP_MINUTES, minuteAt(column, event.clientY));
    pointer = { date, anchor, column };
    drag = { date, from: anchor, to: anchor };
    window.addEventListener('pointermove', trackDrag);
    window.addEventListener('pointerup', finishDrag);
    window.addEventListener('pointercancel', cancelDrag);
  }

  // Remove window listeners if the grid unmounts mid-drag.
  $effect(() => detachDrag);
</script>

<div class="week" class:declared={availability.length > 0} style="--n: {days.length}">
  <section class="scroller" bind:this={scroller} aria-label="Time grid">
    <div class="sticky">
      <div class="head">
        <div class="gutter-cell" aria-hidden="true"></div>
        {#each columns as column (column.date)}
          <div class="day-head" class:today={column.date === today.date}>
            <span class="weekday">{column.head.weekday}</span>
            <span class="number">{column.head.number}</span>
          </div>
        {/each}
      </div>
      <div class="allday">
        <div class="gutter-cell allday-label">
          <span>All day</span>
          {#if expandAllDay && anyAllDayOverflow}
            <button type="button" class="chip-btn" onclick={() => { expandAllDay = false; }}>Fewer</button>
          {/if}
        </div>
        {#each columns as column (column.date)}
          <div class="allday-cell">
            <button type="button" class="allday-add" aria-label="Add all-day item on {formatDate(column.date, 'long')}" onclick={() => oncreateallday(column.date)}></button>
            {#each expandAllDay ? column.allDay : column.allDay.slice(0, ALL_DAY_LIMIT) as event (event.id)}
              <button
                type="button"
                class="allday-item {event.kind}"
                class:tentative={event.tentative}
                class:risk={riskWord(event) !== ''}
                style={edgeVar(event.color)}
                aria-pressed={selectedId === event.id}
                aria-label={describe(event, column.date, 'all day')}
                title={describe(event, column.date, 'all day')}
                onclick={() => onselect(event.id)}
              >
                <span class="glyph" aria-hidden="true">{GLYPH[event.kind]}</span>
                <span class="text">{event.start_date !== null && event.start_date < column.date ? '… ' : ''}{event.title}{riskSuffix(event)}{#if !event.editable}<span aria-hidden="true"> ⤓</span>{/if}</span>
              </button>
            {/each}
            {#if !expandAllDay && column.allDay.length > ALL_DAY_LIMIT}
              <button type="button" class="chip-btn" onclick={() => { expandAllDay = true; }}>+{column.allDay.length - ALL_DAY_LIMIT} more</button>
            {/if}
          </div>
        {/each}
      </div>
    </div>

    <div class="body" style="height: {GRID_PX}px">
      <div class="gutter">
        {#each hours as hour (hour)}
          <span class="hour" style="top: {hour * HOUR_PX + 3}px">{hourLabel(hour)}</span>
        {/each}
      </div>
      {#each columns as column (column.date)}
        <div
          class="col"
          class:today={column.date === today.date}
          role="group"
          aria-label={formatDate(column.date, 'long')}
          onpointerdown={(event) => beginDrag(event, column.date)}
        >
          {#each column.bands as band}
            <div class="band" style="top: {band.top * PX_PER_MINUTE}px; height: {(band.bottom - band.top) * PX_PER_MINUTE}px" aria-hidden="true"></div>
          {/each}

          {#each column.segments as segment (segment.event.id)}
            <button
              type="button"
              class="block {segment.event.kind}"
              class:tentative={segment.event.tentative}
              class:transparent={segment.event.occupancy === 'transparent'}
              style={blockStyle(segment)}
              aria-pressed={selectedId === segment.event.id}
              aria-label={describe(segment.event, column.date, rangeLabel(segment.event))}
              title={describe(segment.event, column.date, rangeLabel(segment.event))}
              onpointerdown={stopPointer}
              onclick={() => onselect(segment.event.id)}
            >
              <span class="title">{softPrefix(segment.event)}{segment.event.title}{#if !segment.event.editable}<span class="readonly" aria-hidden="true"> ⤓</span>{/if}</span>
              {#if segment.columns <= 2 && segment.bottom - segment.top >= 40}
                <span class="meta">{rangeLabel(segment.event)}</span>
              {/if}
              {#if segment.columns <= 2 && segment.bottom - segment.top >= 75 && segment.event.location}
                <span class="meta">{segment.event.location}</span>
              {/if}
            </button>
          {/each}

          {#each column.deadlines as item (item.event.id)}
            {#if item.stack === 0}
              <div class="rule" class:risk={riskWord(item.event) !== ''} style="top: {item.minutes * PX_PER_MINUTE}px" aria-hidden="true"></div>
            {/if}
            <button
              type="button"
              class="deadline"
              class:risk={riskWord(item.event) !== ''}
              style="top: {Math.max(0, item.minutes * PX_PER_MINUTE + item.stack * DEADLINE_STACK_PX - 9)}px"
              aria-pressed={selectedId === item.event.id}
              aria-label={describe(item.event, column.date, formatTime(item.event.start, zone))}
              title={describe(item.event, column.date, formatTime(item.event.start, zone))}
              onpointerdown={stopPointer}
              onclick={() => onselect(item.event.id)}
            >
              <span aria-hidden="true">◆</span> {formatTime(item.event.start, zone)} {item.event.title}{riskSuffix(item.event)}{#if !item.event.editable}<span aria-hidden="true"> ⤓</span>{/if}
            </button>
          {/each}

          {#if drag && drag.date === column.date}
            {@const range = draftRange(drag.from, drag.to)}
            <div class="selection" style="top: {range[0] * PX_PER_MINUTE}px; height: {Math.max(range[1] - range[0], SNAP_MINUTES) * PX_PER_MINUTE}px" aria-hidden="true">
              <span>{minutesLabel(range[0])} to {minutesLabel(range[1])}</span>
            </div>
          {/if}

          {#if column.date === today.date}
            <div class="now" style="top: {todayTop}px" aria-hidden="true"><span class="dot"></span></div>
          {/if}
        </div>
      {/each}
    </div>
  </section>
</div>

<style>
  .week {
    display: flex;
    flex-direction: column;
    height: 100%;
    min-height: 0;
    font-family: var(--font);
    color: var(--ink);
    background: var(--surface);
    border: 1px solid var(--line);
    border-radius: var(--radius);
    box-shadow: var(--shadow);
    overflow: hidden;
  }
  .scroller { flex: 1; min-height: 0; overflow: auto; position: relative; }
  .sticky { position: sticky; top: 0; z-index: 5; min-width: calc(56px + var(--n) * 84px); background: var(--surface); }
  .head, .allday, .body { display: grid; grid-template-columns: 56px repeat(var(--n), minmax(84px, 1fr)); min-width: calc(56px + var(--n) * 84px); }
  .head { border-bottom: 1px solid var(--line); }
  .allday { border-bottom: 1px solid var(--line-strong); }
  .body { position: relative; }
  .gutter-cell { border-right: 1px solid var(--line); }
  .allday-label { display: flex; flex-direction: column; align-items: flex-end; justify-content: center; gap: 4px; padding: 4px 6px; font-size: 10px; color: var(--faint); }

  .day-head { display: flex; flex-direction: column; align-items: center; gap: 2px; padding: 8px 4px 7px; border-left: 1px solid var(--line); font-size: 12px; color: var(--muted); }
  .day-head .number { display: grid; place-items: center; min-width: 28px; height: 28px; border-radius: 50%; font-size: 15px; color: var(--ink); }
  .day-head.today .weekday { color: var(--ink); font-weight: 600; }
  .day-head.today .number { background: var(--accent); color: var(--accent-ink); }

  .allday-cell { position: relative; display: flex; flex-direction: column; gap: 3px; min-width: 0; padding: 4px; border-left: 1px solid var(--line); }
  .allday-add { position: absolute; inset: 0; z-index: 0; margin: 0; padding: 0; border: 0; border-radius: 0; background-color: transparent; }
  .allday-add:hover:not(:disabled) { background-color: var(--surface-2); }
  .allday-add:focus-visible { outline: 2px solid var(--accent); outline-offset: -2px; }

  /* Global button rules in app.css paint hover states with fixed colours. Each
     control keeps its token fill and edge on hover as well as at rest. */
  .allday-item, .chip-btn, .block, .deadline {
    --fill: var(--anchor-soft);
    --edge: var(--line-strong);
    --hatch: none;
    margin: 0;
    font: inherit;
    color: var(--ink);
    background-color: var(--fill);
    background-image: var(--hatch);
    border: 1px solid var(--edge);
    border-radius: var(--radius-sm);
  }
  .allday-item:hover:not(:disabled), .chip-btn:hover:not(:disabled), .block:hover:not(:disabled), .deadline:hover:not(:disabled) {
    background-color: var(--fill);
    background-image: var(--hatch);
    border-color: var(--edge);
  }
  .allday-item:focus-visible, .chip-btn:focus-visible, .block:focus-visible, .deadline:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
  }

  .allday-item { position: relative; z-index: 1; display: flex; align-items: center; gap: 5px; width: 100%; min-width: 0; padding: 2px 6px; border-left-width: 3px; border-left-color: var(--edge-color, var(--line-strong)); font-size: 11px; line-height: 1.3; text-align: left; }
  .allday-item .text { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .allday-item .glyph { flex: none; font-size: 10px; }
  .allday-item.anchor { --fill: var(--anchor-soft); --edge: var(--anchor); border-left-color: var(--edge-color, var(--anchor)); }
  .allday-item.anchor.tentative { --hatch: repeating-linear-gradient(135deg, color-mix(in srgb, var(--anchor) 28%, transparent) 0 3px, transparent 3px 8px); }
  .allday-item.deadline { --fill: var(--deadline-soft); --edge: var(--deadline); color: var(--deadline); }
  .allday-item.intention, .allday-item.routine { --fill: var(--soft-bg); --edge: var(--soft); border: 1.5px dashed var(--soft); border-left-width: 1.5px; color: var(--ink); }
  .allday-item.risk { --fill: var(--risk-soft); --edge: var(--risk); color: var(--risk); }

  .chip-btn { --fill: transparent; --edge: transparent; position: relative; z-index: 1; padding: 2px 4px; font-size: 11px; color: var(--muted); text-align: left; }
  .chip-btn:hover:not(:disabled) { color: var(--ink); }

  .gutter { position: relative; border-right: 1px solid var(--line); }
  .hour { position: absolute; right: 6px; font-size: 11px; line-height: 1; color: var(--faint); white-space: nowrap; }
  .col {
    position: relative;
    min-width: 0;
    border-left: 1px solid var(--line);
    background-color: transparent;
    touch-action: none;
    user-select: none;
  }
  /* Once usual hours exist, time outside them is quietly shaded and declared
     time reads as open, the way working hours do in a conventional grid. */
  .week.declared .col { background-color: var(--surface-2); }
  .col::after {
    content: '';
    position: absolute;
    inset: 0;
    z-index: 1;
    pointer-events: none;
    background-image:
      radial-gradient(circle at 50% 50%, var(--line) 1px, transparent 1.5px),
      repeating-linear-gradient(to bottom, var(--line) 0 1px, transparent 1px 48px);
    background-size: 6px 48px, 100% 100%;
    background-repeat: repeat, no-repeat;
  }

  .band { position: absolute; left: 0; right: 0; z-index: 0; background: var(--surface); border-left: 2px solid var(--avail-line); pointer-events: none; }

  .block {
    --fill: var(--anchor-soft);
    --edge: var(--line-strong);
    position: absolute;
    z-index: 2;
    display: flex;
    flex-direction: column;
    gap: 1px;
    min-width: 0;
    overflow: hidden;
    padding: 3px 6px 3px 7px;
    border-left-width: 3px;
    font-size: 12px;
    line-height: 1.25;
    text-align: left;
  }
  .block.anchor { --fill: var(--anchor-soft); --edge: var(--anchor); border-left-color: var(--edge-color, var(--anchor)); }
  .block.anchor.tentative { --hatch: repeating-linear-gradient(135deg, color-mix(in srgb, var(--anchor) 28%, transparent) 0 3px, transparent 3px 8px); }
  .block.anchor.transparent { --fill: transparent; border-color: var(--edge-color, var(--anchor)); }
  .block.intention, .block.routine { --fill: var(--soft-bg); --edge: var(--soft); border: 1.5px dashed var(--soft); border-left-width: 1.5px; }
  .block[aria-pressed='true'] { outline: 2px solid var(--ink); outline-offset: 0; }
  .block .title { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-weight: 600; }
  .block .meta { display: block; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; font-size: 11px; color: var(--muted); }
  .block .readonly { font-weight: 400; color: var(--muted); }

  .rule { position: absolute; left: 0; right: 0; z-index: 3; height: 0; border-top: 1px solid var(--deadline); pointer-events: none; }
  .rule.risk { border-top-color: var(--risk); }
  .deadline {
    --fill: var(--deadline-soft);
    --edge: var(--deadline);
    position: absolute;
    left: 4px;
    right: 4px;
    z-index: 4;
    display: block;
    height: 18px;
    padding: 0 6px;
    font-size: 11px;
    line-height: 16px;
    color: var(--deadline);
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .deadline.risk { --fill: var(--risk-soft); --edge: var(--risk); color: var(--risk); }
  .deadline[aria-pressed='true'] { outline: 2px solid var(--ink); outline-offset: 0; }

  .selection { position: absolute; left: 2px; right: 2px; z-index: 5; display: flex; align-items: flex-start; padding: 2px 5px; overflow: hidden; pointer-events: none; font-size: 11px; color: var(--ink); background-color: var(--accent-soft); border: 1px solid var(--accent); border-radius: var(--radius-sm); opacity: 0.85; }

  .now { position: absolute; left: 0; right: 0; z-index: 6; height: 0; border-top: 2px solid var(--now); pointer-events: none; }
  .now .dot { position: absolute; left: -4px; top: -3px; width: 8px; height: 8px; border-radius: 50%; background: var(--now); }
</style>
