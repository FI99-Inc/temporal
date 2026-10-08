<script lang="ts">
  import type { CalendarEvent, Weekday } from '../lib/calendar-types.ts';
  import { allDayFor, dayDeadlines, formatDate, formatMonth, formatTime, layoutDay, minutesLabel, monthGrid, weekdayOf, zonedParts } from '../lib/calendar.ts';

  let { month, weekStartsOn, zone, now, events, selectedId, onselect, onpickday, oncreateallday }: {
    month: string;
    weekStartsOn: 'mon' | 'sun';
    zone: string;
    now: number;
    events: CalendarEvent[];
    selectedId: string | null;
    onselect: (id: string) => void;
    onpickday: (date: string) => void;
    oncreateallday: (date: string) => void;
  } = $props();

  type Chip = { key: string; event: CalendarEvent; kind: CalendarEvent['kind']; bar: boolean; text: string; label: string; risk: boolean };

  const VISIBLE_CHIPS = 3;
  const GLYPH: Record<CalendarEvent['kind'], string> = { anchor: '▮', deadline: '◆', intention: '○', routine: '○' };
  const KIND_WORD: Record<CalendarEvent['kind'], string> = { anchor: 'Fixed', deadline: 'Deadline', intention: 'Intention', routine: 'Routine' };
  const WEEKDAY_NAME: Record<Weekday, string> = { mon: 'Mon', tue: 'Tue', wed: 'Wed', thu: 'Thu', fri: 'Fri', sat: 'Sat', sun: 'Sun' };

  const todayDate = $derived(zonedParts(now, zone).date);
  const dates = $derived(monthGrid(month, weekStartsOn));
  const headers = $derived(dates.slice(0, 7).map((date) => WEEKDAY_NAME[weekdayOf(date)]));
  const cells = $derived(dates.map((date) => buildCell(date)));

  function riskWord(event: CalendarEvent): string {
    if (event.state === 'overdue') return 'Overdue';
    if (event.risk === 'insufficient') return 'Not enough time';
    if (event.risk === 'tight') return 'Tight';
    return '';
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

  function buildCell(date: string) {
    const chips: Chip[] = [];
    for (const event of allDayFor(events, date, zone)) {
      const continued = event.start_date !== null && event.start_date < date;
      const risk = riskWord(event);
      chips.push({
        key: `all:${event.id}`,
        event,
        kind: event.kind,
        bar: true,
        text: `${continued ? '… ' : ''}${event.title}${risk ? ` · ${risk}` : ''}`,
        label: describe(event, date, 'all day'),
        risk: risk !== '',
      });
    }
    const timed = [
      ...layoutDay(events, date, zone).map((segment) => ({ event: segment.event, minutes: segment.top, continued: segment.continuesBefore })),
      ...dayDeadlines(events, date, zone).map((item) => ({ event: item.event, minutes: item.minutes, continued: false })),
    ].sort((a, b) => a.minutes - b.minutes || (a.event.title < b.event.title ? -1 : a.event.title > b.event.title ? 1 : 0));
    for (const item of timed) {
      const { event } = item;
      const risk = riskWord(event);
      const isDeadline = event.kind === 'deadline';
      const when = item.continued
        ? 'continues from earlier'
        : isDeadline ? formatTime(event.start, zone) : `${formatTime(event.start, zone)} to ${formatTime(event.end, zone)}`;
      const lead = item.continued ? '…' : minutesLabel(item.minutes);
      chips.push({
        key: `timed:${event.id}`,
        event,
        kind: event.kind,
        bar: false,
        text: `${lead} ${event.title}${risk ? ` · ${risk}` : ''}`,
        label: describe(event, date, when),
        risk: risk !== '',
      });
    }
    return {
      date,
      day: Number(date.slice(8)),
      inMonth: date.startsWith(`${month}-`),
      visible: chips.slice(0, VISIBLE_CHIPS),
      hidden: Math.max(0, chips.length - VISIBLE_CHIPS),
    };
  }
</script>

<div class="month">
  <div class="weekdays">
    {#each headers as name (name)}<span>{name}</span>{/each}
  </div>
  <div class="grid" role="region" aria-label={formatMonth(month)}>
    {#each cells as cell (cell.date)}
      <div class="cell" class:outside={!cell.inMonth} class:today={cell.date === todayDate}>
        <div class="cell-top">
          <button type="button" class="date" aria-label="Open {formatDate(cell.date, 'long')} in day view" onclick={() => onpickday(cell.date)}>{cell.day}</button>
          <button type="button" class="add" aria-label="Add all-day item on {formatDate(cell.date, 'long')}" onclick={() => oncreateallday(cell.date)}>+</button>
        </div>
        {#each cell.visible as chip (chip.key)}
          <button
            type="button"
            class="chip {chip.kind}"
            class:bar={chip.bar}
            class:risk={chip.risk}
            aria-pressed={selectedId === chip.event.id}
            aria-label={chip.label}
            title={chip.label}
            onclick={() => onselect(chip.event.id)}
          >
            <span class="glyph" aria-hidden="true">{GLYPH[chip.kind]}</span><span class="text">{chip.text}</span>{#if !chip.event.editable}<span aria-hidden="true"> ⤓</span>{/if}
          </button>
        {/each}
        {#if cell.hidden > 0}
          <button type="button" class="more" aria-label="{cell.hidden} more on {formatDate(cell.date, 'long')}, open in day view" onclick={() => onpickday(cell.date)}>+{cell.hidden} more</button>
        {/if}
      </div>
    {/each}
  </div>
</div>

<style>
  .month {
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
  .weekdays, .grid { display: grid; grid-template-columns: repeat(7, minmax(0, 1fr)); }
  .weekdays { border-bottom: 1px solid var(--line); }
  .weekdays span { padding: 7px 8px; font-size: 12px; color: var(--muted); }
  .weekdays span + span { border-left: 1px solid var(--line); }
  .grid { flex: 1; min-height: 0; grid-template-rows: repeat(6, minmax(0, 1fr)); }

  .cell {
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
    min-height: 0;
    padding: 4px 5px;
    overflow: hidden;
    border-right: 1px solid var(--line);
    border-bottom: 1px solid var(--line);
  }
  .cell.outside { background-color: var(--surface-2); }
  .cell-top { display: flex; align-items: center; justify-content: space-between; gap: 4px; min-width: 0; }

  /* Global button rules in app.css paint hover states with fixed colours. Each
     control keeps its token fill and edge on hover as well as at rest. */
  .date, .add, .chip, .more {
    --fill: transparent;
    --edge: transparent;
    margin: 0;
    font: inherit;
    color: var(--ink);
    background-color: var(--fill);
    border: 1px solid var(--edge);
    border-radius: var(--radius-sm);
  }
  .date:hover:not(:disabled), .add:hover:not(:disabled), .chip:hover:not(:disabled), .more:hover:not(:disabled) {
    background-color: var(--fill);
    border-color: var(--edge);
  }
  .date:focus-visible, .add:focus-visible, .chip:focus-visible, .more:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 1px;
  }

  .date { min-width: 26px; height: 26px; padding: 0 6px; border-radius: 50%; font-size: 12px; text-align: center; }
  .cell.outside .date { color: var(--faint); }
  .cell.today .date { --fill: var(--accent); --edge: var(--accent); color: var(--accent-ink); font-weight: 600; }

  .add {
    --fill: var(--surface);
    --edge: var(--line-strong);
    width: 22px;
    height: 22px;
    padding: 0;
    border-radius: 50%;
    font-size: 13px;
    line-height: 1;
    opacity: 0;
  }
  .cell:hover .add, .cell:focus-within .add { opacity: 1; }

  .chip {
    --fill: var(--anchor-soft);
    --edge: var(--anchor);
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    min-width: 0;
    padding: 1px 5px;
    border-left-width: 3px;
    font-size: 11px;
    line-height: 1.35;
    text-align: left;
  }
  .chip.bar { padding-block: 0; min-height: 17px; }
  .chip.anchor { --fill: var(--anchor-soft); --edge: var(--anchor); }
  .chip.deadline { --fill: var(--deadline-soft); --edge: var(--deadline); color: var(--deadline); }
  .chip.intention, .chip.routine { --fill: var(--soft-bg); --edge: var(--soft); border: 1.5px dashed var(--soft); border-left-width: 1.5px; }
  .chip.risk { --fill: var(--risk-soft); --edge: var(--risk); color: var(--risk); }
  .chip[aria-pressed='true'] { outline: 2px solid var(--ink); outline-offset: 0; }
  .chip .glyph { flex: none; font-size: 9px; }
  .chip .text { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }

  .more { --edge: transparent; padding: 0 4px; font-size: 11px; color: var(--muted); text-align: left; }
  .more:hover:not(:disabled) { color: var(--ink); }
</style>
