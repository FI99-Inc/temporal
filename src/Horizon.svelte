<script lang="ts">
  import type { Item, Snapshot, Species } from './lib/api.ts';
  import { pack, position, visibleTicks } from './lib/horizon.ts';

  let { data, selectedId, onselect }: { data: Snapshot; selectedId: string | null; onselect: (id: string) => void } = $props();
  let width = $state(700);
  type TimedItem = Item & { start: number };
  const tracks: { species: Species; title: string; hint: string }[] = [
    { species: 'anchor', title: 'Fixed', hint: 'Recorded occurrences' },
    { species: 'deadline', title: 'Deadlines', hint: 'Real cutoffs' },
    { species: 'window', title: 'Opportunity', hint: 'Within declared time' },
  ];
  let ticks = $derived(visibleTicks(data.ticks, data.now, data.end, width));
  let earlier = $derived(data.items.filter(i => ['anchor', 'deadline'].includes(i.species) && i.start !== null && (i.end ?? i.start) < data.now));
  let flexible = $derived(data.items.filter(i => ['task', 'intention', 'routine', 'suggestion'].includes(i.species)));

  function rows(species: Species) {
    return pack(data.items.filter((i): i is TimedItem => i.species === species && i.start !== null && (i.end ?? i.start) >= data.now && i.start <= data.end), data.now, data.end, width);
  }
  function stateLabel(item: Item): string {
    if (item.risk === 'overdue') return 'Overdue deadline';
    if (item.risk === 'insufficient') return 'Insufficient opportunity';
    if (item.risk === 'no_known_work') return 'No known work';
    if (item.risk === 'not_applicable') return 'Resolved';
    if (item.risk) return item.risk === 'room' ? 'Room in declared time' : item.risk === 'tight' ? 'Tight' : 'Pressure unknown';
    if (item.species === 'suggestion') return item.phase === 'expired' ? 'Expired advice' : `${item.phase.replaceAll('_', ' ')} advice`;
    if (['intention', 'routine'].includes(item.species) && ['done', 'dismissed', 'skipped', 'inactive'].includes(item.phase)) return item.phase;
    if (item.phase === 'preference_passed') return 'Still optional';
    if (item.species === 'intention') return 'Optional';
    if (item.species === 'routine') return 'Preferred day';
    if (item.species === 'task') return `Trace: ${item.phase}`;
    if (item.species === 'window') return 'Potential time';
    return item.phase;
  }
</script>

<section class="horizon" aria-label="Horizon time view">
  <div class="horizon-heading"><h2>The shape of your time</h2><span>Near time expands. Distant time compresses.</span></div>
  {#if earlier.length}
    <div class="earlier">
      <h3>Earlier</h3>
      {#each earlier as item (item.id)}
        <button class:chosen={selectedId === item.id} class:overdue={item.risk === 'overdue'} onclick={() => onselect(item.id)} aria-pressed={selectedId === item.id}>
          <span class="species-shape {item.species}"></span><strong>{item.title}</strong><span>{stateLabel(item)}</span><small>{item.when}</small>
        </button>
      {/each}
    </div>
  {/if}
  <div class="time-surface" bind:clientWidth={width}>
    <div class="axis" aria-label="Nonlinear time scale">
      {#each ticks as tick, index (tick.at)}
        <span class:first={index === 0} class:last={index === ticks.length - 1} style:left="{position(tick.at, data.now, data.end) * 100}%">{tick.label}</span>
      {/each}
    </div>
    <div class="track-surface">
      <div class="time-guides" aria-hidden="true">
        {#each ticks as tick (tick.at)}<i class:now={tick.at === data.now} style:left="{position(tick.at, data.now, data.end) * 100}%"></i>{/each}
      </div>
      {#each tracks as track (track.species)}
        {@const placed = rows(track.species)}
        {@const lanes = placed.length ? Math.max(...placed.map(r => r.lane)) + 1 : 0}
        <section class="track {track.species}" aria-label={track.title}>
          <div class="track-label"><h3>{track.title}</h3><span>{track.hint}</span></div>
          <div class="track-items" style:height="{Math.max(60, lanes * 96)}px">
            {#each placed as row (row.item.id)}
              {@const item = row.item}
              <div class="time-mark {item.species}" class:imported={item.ownership.startsWith('Imported')} style:top="{row.lane * 96 + 10}px" aria-hidden="true">
                <i class="time-stem" style:left="{row.x}px"></i>
                {#if item.end !== null}<i class="duration-bar" style:left="{row.x}px" style:width="{Math.max(2, row.barEnd - row.x)}px"></i>{/if}
              </div>
              <button class="temporal-item {item.species}" class:chosen={selectedId === item.id} class:conditional={item.conditional}
                style:left="{row.labelX}px" style:width="{row.width}px" style:top="{row.lane * 96 + 19}px"
                onclick={() => onselect(item.id)} aria-pressed={selectedId === item.id} title={`${item.title}: ${item.when}. ${stateLabel(item)}${item.conditional ? ' (conditional)' : ''}. ${item.source}`}>
                <span class="item-name">{#if item.milestone}<span class="milestone" aria-label="Milestone">◇</span>{/if}{item.title}</span>
                <span class="item-state" class:risk={item.risk === 'tight' || item.risk === 'insufficient'}>{stateLabel(item)}{item.conditional ? ' · conditional' : ''}</span>
                <span class="item-source">{item.source}</span>
              </button>
            {:else}
              <p class="track-empty">{track.species === 'anchor' ? 'No upcoming anchors in these inputs.' : track.species === 'deadline' ? 'No upcoming real deadlines.' : data.has_declarations ? 'No declared opportunity remains in this range.' : 'Availability has not been declared.'}</p>
            {/each}
          </div>
        </section>
      {/each}
    </div>
    <div class="axis-caption"><span>Now</span><span>{data.zone}</span><span>End of this snapshot</span></div>
  </div>
  <section class="flexible" aria-label="Flexible work and preferences">
    <div class="flexible-heading"><h3>Flexible</h3><span>Work, preferences, and advice. No time is reserved.</span></div>
    <div class="flexible-items">
      {#each flexible as item (item.id)}
        <button class="flexible-item {item.species}" class:chosen={selectedId === item.id} onclick={() => onselect(item.id)} aria-pressed={selectedId === item.id}>
          <span class="flexible-type"><span class="species-shape {item.species}"></span>{item.species}</span>
          <strong>{item.title}</strong><span>{stateLabel(item)}</span>
          <small>{item.species === 'routine' ? item.when : item.source}</small>
        </button>
      {:else}<p class="quiet">No separate flexible items in this sample.</p>{/each}
    </div>
  </section>
</section>
