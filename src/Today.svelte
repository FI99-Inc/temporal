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
</script>

<section class="today" aria-label="Today">
  <div class="today-heading">
    <h2>Today</h2>
    <span>{today.date_label} · through {today.day_end_label}</span>
  </div>

  <div class="today-groups">
    <div class="today-group" aria-label="Fixed today">
      <h3>Fixed</h3>
      <p class="group-hint">Recorded facts that fall today.</p>
      {#if today.fixed.length}
        {#each fixed as row (row.id)}
          <button class="today-row {row.species}" class:chosen={selectedId === row.id} aria-pressed={selectedId === row.id} onclick={() => onselect(row.id)}>
            <strong>{row.title}</strong>
            <span>{row.when}</span>
            <small>{stateLabel(row)}{row.conditional ? ' · conditional' : ''}</small>
          </button>
        {/each}
        {#if hiddenFixed}
          <button class="disclosure" onclick={() => { showAllFixed = !showAllFixed; }}>
            {showAllFixed ? 'Show fewer' : `Show ${hiddenFixed} more today`}
          </button>
        {/if}
      {:else}
        <p class="group-empty">Nothing fixed today.</p>
      {/if}
    </div>

    <div class="today-group" aria-label="Worth doing">
      <h3>Worth doing</h3>
      <p class="group-hint">Work that fits in the time you declared. Nothing is reserved.</p>
      {#if today.worth_doing.length}
        {#each today.worth_doing as row (row.id)}
          <button class="today-row advice {row.species}" class:chosen={selectedId === row.id} aria-pressed={selectedId === row.id} onclick={() => onselect(row.id)}>
            <strong>{row.title}</strong>
            <span>{row.when}</span>
            <small>{stateLabel(row)}{row.conditional ? ' · conditional' : ''}</small>
          </button>
        {/each}
      {:else}
        <p class="group-empty">No work has a definite fit in the rest of today.</p>
      {/if}
    </div>

    <div class="today-group" aria-label="Loose">
      <h3>Loose</h3>
      <p class="group-hint">Optional. A preferred day that passes creates no debt.</p>
      {#if today.loose.length}
        {#each today.loose as row (row.id)}
          <button class="today-row advice {row.species}" class:chosen={selectedId === row.id} aria-pressed={selectedId === row.id} onclick={() => onselect(row.id)}>
            <strong>{row.title}</strong>
            <span>{row.when}</span>
            <small>{stateLabel(row)}</small>
          </button>
        {/each}
      {:else}
        <p class="group-empty">Nothing loose fits the rest of today.</p>
      {/if}
    </div>

    <div class="today-group" aria-label="On the radar">
      <h3>On the radar</h3>
      <p class="group-hint">Not today, but worth knowing about.</p>
      {#if today.radar.length}
        {#each today.radar as row (row.id)}
          <button class="today-row {row.species}" class:chosen={selectedId === row.id} aria-pressed={selectedId === row.id} onclick={() => onselect(row.id)}>
            <strong>{row.title}</strong>
            <span>{row.when}</span>
            <small>{stateLabel(row)}{row.conditional ? ' · conditional' : ''}</small>
          </button>
        {/each}
        {#if today.radar_omitted}
          <p class="group-empty">{today.radar_omitted} more in the full Horizon below.</p>
        {/if}
      {:else}
        <p class="group-empty">Nothing else in range.</p>
      {/if}
    </div>
  </div>

  {#if today.worth_doing.length || today.loose.length}
    <details class="today-why">
      <summary>Why these, and what they are not</summary>
      {#each [...today.worth_doing, ...today.loose] as row (row.id)}
        <div class="today-why-row">
          <strong>{row.title}</strong>
          <ul>{#each row.notes as note}<li>{note}</li>{/each}</ul>
        </div>
      {/each}
      <p class="small-note">Each range is assessed on its own. Overlapping ranges do not certify that everything fits together, and selecting a row records nothing.</p>
    </details>
  {/if}
</section>
