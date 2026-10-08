<script lang="ts">
  import { onMount } from 'svelte';
  import HorizonView from './HorizonView.svelte';
  import { catalog, snapshot, type Scenario, type Snapshot } from '../lib/api.ts';

  // Synthetic weeks used to check Horizon's behaviour. Nothing here is stored.
  let { onclose }: { onclose: () => void } = $props();
  let scenarios = $state<Scenario[]>([]);
  let scenarioId = $state('S02');
  let data = $state<Snapshot | null>(null);
  let busy = $state(true);
  let error = $state('');
  let request = 0;

  async function load(id: string, minutes = 0) {
    const mine = ++request;
    busy = true; error = '';
    try { const next = await snapshot(id, minutes); if (mine === request) { data = next; scenarioId = id; } }
    catch (e) { if (mine === request) error = String(e instanceof Error ? e.message : e); }
    finally { if (mine === request) busy = false; }
  }
  onMount(() => { void catalog().then(list => { scenarios = list; }).catch(e => { error = String(e); }); void load(scenarioId); });
</script>

<div class="notice">
  <div><strong>Example weeks (synthetic)</strong><p>Fixed scenarios with virtual time. They never touch your own data.</p></div>
  <div class="example-controls">
    <label class="sr" for="scenario">Example week</label>
    <select id="scenario" value={scenarioId} disabled={busy || !scenarios.length} onchange={(e) => load(e.currentTarget.value)}>
      {#each scenarios as scenario (scenario.id)}<option value={scenario.id}>{scenario.title}</option>{/each}
    </select>
    <button disabled={busy || !data || data.offset_minutes === 0} onclick={() => load(scenarioId, 0)}>Reset time</button>
    <button disabled={busy || !data || data.offset_minutes + 60 > data.max_offset_minutes} onclick={() => data && load(scenarioId, data.offset_minutes + 60)}>+1 hour</button>
    <button disabled={busy || !data || data.offset_minutes + 1440 > data.max_offset_minutes} onclick={() => data && load(scenarioId, data.offset_minutes + 1440)}>+1 day</button>
    <button class="primary" onclick={onclose}>Back to my time</button>
  </div>
</div>
{#if error}<div class="notice warn" role="alert"><p>{error}</p></div>{/if}
{#if data}
  <p class="scenario-note">{data.scenario.description}</p>
  <HorizonView {data} busy={true} onedit={() => {}} onapply={async () => false} />
{:else if !error}
  <div class="loading" role="status">Preparing the example…</div>
{/if}

<style>
  .sr { position: absolute; width: 1px; height: 1px; overflow: hidden; clip: rect(0 0 0 0); }
</style>
