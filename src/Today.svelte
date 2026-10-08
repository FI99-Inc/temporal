<script lang="ts">
  import type { TodayRow, TodayView } from './lib/api.ts';

  let { today, selectedId, onselect }: { today: TodayView; selectedId: string | null; onselect: (id: string) => void } = $props();
  let showAllFixed = $state(false);

  let fixed = $derived(showAllFixed ? today.fixed : today.fixed.slice(0, today.fixed_preview));
  let hiddenFixed = $derived(Math.max(0, today.fixed.length - today.fixed_preview));

  const readable = (value: string) => value.replaceAll('_', ' ');
  function stateLabel(row: TodayRow): string {
    switch (row.risk) {
      case 'overdue': return 'Overdue';
      case 'insufficient': return 'Not enough declared time';
      case 'tight': return 'Tight';
      case 'room': return 'Room in declared time';
      case 'unknown': return 'Pressure unknown';
      case 'no_known_work': return 'No known work';
      case 'not_applicable': return 'Resolved';
    }
    if (row.species === 'task') return `Trace: ${readable(row.state)}`;
    return readable(row.state);
  }
  /** The row's leading time, shortened for the time column. */
  function stamp(row: TodayRow, withDay: boolean): { day: string; time: string } {
    const match = row.when.match(/^(\w{3}) (\w{3}) (\d{1,2}), (\d{1,2}:\d{2}) ([AP]M)/);
    if (match) return { day: withDay ? `${match[1]} ${match[3]}` : '', time: `${match[4]}${match[5] === 'PM' ? 'p' : 'a'}` };
    const date = row.when.match(/(\d{4}-\d{2}-\d{2})/);
    return { day: withDay && date ? date[1].slice(5).replace('-', '.') : '', time: date ? 'Day' : '—' };
  }
  const atRisk = (row: TodayRow) => row.risk === 'overdue' || row.risk === 'insufficient' || row.risk === 'tight';
  const groups = $derived([
    { key: 'fixed', title: 'Fixed', hint: 'Recorded facts that fall today.', rows: fixed, total: today.fixed.length, empty: 'Nothing fixed today', advice: false, day: false },
    { key: 'worth', title: 'Worth doing', hint: 'Work that fits the time you declared. Nothing is reserved.', rows: today.worth_doing, total: today.worth_doing.length, empty: 'No work has a definite fit today', advice: true, day: false },
    { key: 'loose', title: 'Loose', hint: 'Optional. A preferred day that passes creates no debt.', rows: today.loose, total: today.loose.length, empty: 'Nothing loose fits today', advice: true, day: false },
    { key: 'radar', title: 'On the radar', hint: 'Not today, but worth knowing about.', rows: today.radar, total: today.radar.length + today.radar_omitted, empty: 'Nothing else in range', advice: false, day: true },
  ]);
</script>

<section class="today" aria-label="Today">
  <div class="section-bar"><span class="index">01</span><span>Today</span><span class="spacer"></span><span class="label">{today.date_label} · until {today.day_end_label}</span></div>
  <div class="today-groups">
    {#each groups as group (group.key)}
      <div class="today-group" aria-label={group.title}>
        <div class="group-head">
          <h2>{group.title}<span class="count">{String(group.total).padStart(2, '0')}</span></h2>
          <p class="group-hint">{group.hint}</p>
        </div>
        {#each group.rows as row (row.id)}
          {@const at = stamp(row, group.day)}
          <button class="today-row {row.species}" class:advice={group.advice} class:at-risk={atRisk(row)} class:chosen={selectedId === row.id} aria-pressed={selectedId === row.id} onclick={() => onselect(row.id)} title={row.when}>
            <span class="row-time">{#if at.day}{at.day}<br />{/if}{at.time}</span>
            <strong>{row.title}</strong>
            <small>{stateLabel(row)}{row.conditional ? ' · conditional' : ''}</small>
          </button>
        {:else}
          <p class="group-empty">{group.empty}</p>
        {/each}
        {#if group.key === 'fixed' && hiddenFixed}
          <button class="disclosure" onclick={() => { showAllFixed = !showAllFixed; }}>{showAllFixed ? 'Show fewer' : `+${hiddenFixed} more today`}</button>
        {/if}
        {#if group.key === 'radar' && today.radar_omitted}
          <p class="group-empty">+{today.radar_omitted} more in the Horizon below</p>
        {/if}
      </div>
    {/each}
  </div>

  {#if today.worth_doing.length || today.loose.length}
    <details class="today-why">
      <summary>Why these, and what they are not</summary>
      <div class="today-why-body">
        {#each [...today.worth_doing, ...today.loose] as row (row.id)}
          <div class="today-why-row">
            <h3>{row.title}</h3>
            <ul>{#each row.notes as note}<li>{note}</li>{/each}</ul>
          </div>
        {/each}
      </div>
      <p class="small-note">Each range is assessed on its own. Overlapping ranges do not certify that everything fits together, and selecting a row records nothing.</p>
    </details>
  {/if}
</section>
