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
  .month { display: flex; flex-direction: column; height: 100%; min-height: 0; font-family: var(--font); color: var(--ink); background: var(--bg); }
  .weekdays { display: grid; grid-template-columns: repeat(7, minmax(0, 1fr)); border-bottom: var(--rule); }
  .weekdays span { padding: 8px 10px; font-family: var(--font-mono); font-size: 10.5px; letter-spacing: .1em; text-transform: uppercase; color: var(--muted); border-left: 1px solid var(--line); }
  .weekdays span:first-child { border-left: 0; }
  .grid { flex: 1; min-height: 0; display: grid; grid-template-columns: repeat(7, minmax(0, 1fr)); grid-template-rows: repeat(6, minmax(0, 1fr)); }
  .cell { position: relative; display: flex; flex-direction: column; gap: 2px; min-width: 0; min-height: 0; overflow: hidden; padding: 6px 6px 4px; border-left: 1px solid var(--line); border-bottom: 1px solid var(--line); }
  .cell:nth-child(7n + 1) { border-left: 0; }
  .cell.outside { background: var(--closed); }
  .cell.outside .date { color: var(--faint); }
  .cell-top { display: flex; align-items: flex-start; justify-content: space-between; margin-bottom: 2px; }
  .date { border: 0; padding: 0 2px; background: transparent; font-family: var(--font-display); font-stretch: 125%; font-weight: 800; font-size: 24px; line-height: .95; letter-spacing: -.03em; text-transform: none; color: var(--ink); font-variant-numeric: tabular-nums; }
  .date:hover:not(:disabled) { background: transparent; color: var(--accent); }
  .cell.today { background: var(--invert-bg); color: var(--invert-ink); }
  .cell.today .date { color: var(--accent); }
  .add { opacity: 0; border: 1px solid currentColor; padding: 0 6px; font-size: 12px; line-height: 16px; color: inherit; background: transparent; }
  .cell:hover .add, .cell:focus-within .add { opacity: 1; }
  .add:hover:not(:disabled) { background: var(--accent); border-color: var(--accent); color: var(--accent-ink); }
  .chip { display: flex; align-items: center; gap: 5px; width: 100%; min-width: 0; padding: 1px 4px; border: 0; background: transparent; color: inherit; font-family: var(--font-mono); font-size: 10.5px; letter-spacing: 0; text-transform: none; line-height: 1.45; text-align: left; }
  .chip .text { min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
  .chip .glyph { flex: none; font-size: 8px; }
  .chip:hover:not(:disabled) { background: var(--invert-bg); color: var(--invert-ink); }
  .cell.today .chip:hover:not(:disabled) { background: var(--bg); color: var(--ink); }
  .chip.bar { background: var(--invert-bg); color: var(--invert-ink); font-family: var(--font); font-weight: 600; font-size: 11px; }
  .cell.today .chip.bar { background: var(--bg); color: var(--ink); }
  .chip.intention, .chip.routine, .chip.bar.intention, .chip.bar.routine { background: transparent; color: inherit; outline: 1px dashed currentColor; outline-offset: -1px; font-family: var(--font-mono); font-weight: 400; font-size: 10.5px; }
  .chip.risk { background: var(--risk); color: var(--bg); }
  .chip[aria-pressed='true'] { outline: 2px solid var(--accent); outline-offset: 0; }
  .more { align-self: flex-start; border: 0; padding: 0 4px; background: transparent; color: inherit; opacity: .7; font-size: 10px; }
  .more:hover:not(:disabled) { opacity: 1; background: transparent; color: var(--accent); }
</style>
