<script lang="ts">
  import { onMount } from 'svelte';
  import Horizon from './Horizon.svelte';
  import { catalog, snapshot, desktop, personalSnapshot, importTrace, type Scenario, type Snapshot, type TraceInfo, type PersonalView } from './lib/api.ts';

  let scenarios = $state<Scenario[]>([]);
  let data = $state<Snapshot | null>(null);
  let selectedId = $state<string | null>(null);
  let scenarioId = $state('S02');
  let busy = $state(true);
  let error = $state('');
  let personal = $state(false);
  let trace = $state<TraceInfo | null>(null);
  let filePicker = $state<HTMLInputElement>();
  let requestNumber = 0;
  let selected = $derived(data?.items.find(i => i.id === selectedId));
  let nextAnchor = $derived(data?.items.filter(i => i.species === 'anchor' && i.phase === 'upcoming').sort((a, b) => a.start! - b.start!)[0]);

  async function load(id: string, minutes = 0) {
    const request = ++requestNumber;
    busy = true; error = '';
    try {
      const next = await snapshot(id, minutes);
      if (request !== requestNumber) return;
      if (personal || id !== data?.scenario.id || !next.items.some(i => i.id === selectedId)) selectedId = null;
      data = next; scenarioId = id; personal = false; trace = null;
    } catch (e) {
      if (request === requestNumber) error = String(e instanceof Error ? e.message : e);
    } finally { if (request === requestNumber) busy = false; }
  }
  function showPersonal(next: PersonalView) {
    if (!personal || !next.view.items.some(i => i.id === selectedId)) selectedId = null;
    data = next.view; trace = next.trace; personal = true;
  }
  async function loadPersonal() {
    const request = ++requestNumber;
    busy = true; error = '';
    try { const next = await personalSnapshot(); if (request === requestNumber) showPersonal(next); }
    catch (e) { if (request === requestNumber) error = String(e instanceof Error ? e.message : e); }
    finally { if (request === requestNumber) busy = false; }
  }
  async function importFile(file: File | undefined) {
    if (!file) return;
    const request = ++requestNumber;
    busy = true; error = '';
    try {
      if (file.size > 10 * 1024 * 1024) throw new Error('Trace export exceeds 10 MiB. Previous snapshot kept.');
      const json = new TextDecoder('utf-8', { fatal:true }).decode(await file.arrayBuffer());
      const result = await importTrace(json);
      if (request === requestNumber) { showPersonal(result.personal); error = result.error ?? ''; }
    } catch (e) { if (request === requestNumber) error = String(e instanceof Error ? e.message : e); }
    finally { if (request === requestNumber) busy = false; if (filePicker) filePicker.value = ''; }
  }
  onMount(() => {
    void (async () => {
      try { scenarios = await catalog(); if (desktop) await loadPersonal(); else await load(scenarioId); }
      catch (e) { error = String(e); busy = false; }
    })();
  });
  const readable = (value: string) => value.replaceAll('_', ' ');
</script>

<svelte:head><title>Temporal Engine — Horizon</title></svelte:head>

<a class="skip-link" href="#main">Skip to Horizon</a>
<div class="app-shell">
  <header class="app-header">
    <div class="brand"><span class="brand-mark" aria-hidden="true"><i></i><i></i><i></i></span><span>Temporal Engine</span></div>
    <span class="sample-mode">{personal ? (desktop ? 'Local data' : 'Preview cache') : 'Sample data'}</span>
    <nav class="view-switch" aria-label="Data view">
      <button aria-pressed={personal} disabled={busy} onclick={loadPersonal}>My time</button>
      <button aria-pressed={!personal} disabled={busy} onclick={() => load(scenarioId)}>Examples</button>
    </nav>
    {#if !personal}<label class="scenario-picker">Example week
      <select aria-label="Example week" value={scenarioId} disabled={busy || !scenarios.length} onchange={(e) => load(e.currentTarget.value)}>
        {#each scenarios as scenario (scenario.id)}<option value={scenario.id}>{scenario.title}</option>{/each}
      </select>
    </label>{:else}<div class="import-control">
      <input class="file-input" type="file" accept=".json,application/json" aria-label="Trace JSON export" bind:this={filePicker} onchange={(e) => importFile(e.currentTarget.files?.[0])} />
      <button disabled={busy} onclick={() => filePicker?.click()}>Import Trace JSON</button>
    </div>{/if}
  </header>

  <main id="main" aria-busy={busy}>
    <section class="page-heading" aria-label="Current virtual time">
      <div><h1>Horizon</h1><p>{data?.date_label ?? 'Your near future, in view'}</p></div>
      <div class="clock-controls">
        <div class="clock"><span class="now-dot" aria-hidden="true"></span><strong>{data?.clock_label ?? '—'}</strong><span>{personal ? (desktop ? 'Local time' : 'Frozen preview time') : 'Virtual time'}</span></div>
        {#if personal}<button class="refresh-view" disabled={busy} onclick={loadPersonal}>Update view</button>
        {:else}<div class="time-buttons" aria-label="Virtual clock controls">
          <button disabled={busy || !data || data.offset_minutes === 0} onclick={() => load(scenarioId, 0)}>Reset time</button>
          <button disabled={busy || !data || data.offset_minutes + 60 > data.max_offset_minutes} onclick={() => data && load(scenarioId, data.offset_minutes + 60)}>+1 hour</button>
          <button disabled={busy || !data || data.offset_minutes + 1440 > data.max_offset_minutes} onclick={() => data && load(scenarioId, data.offset_minutes + 1440)}>+1 day</button>
        </div>{/if}
      </div>
    </section>

    {#if error}
      <div class="error" role="alert"><strong>Horizon could not update</strong><p>{error}</p><button onclick={() => personal ? loadPersonal() : load(scenarioId)}>Reload view</button></div>
    {/if}

    {#if data}
      <div class="scenario-note"><p>{data.scenario.description}</p><span role="status">{busy ? 'Updating…' : personal ? 'Stored on this computer' : 'Time changes only this sample'}</span></div>
      {#if personal && trace}<section class="trace-overview" aria-label="Trace snapshot">
        <div><strong>{trace.exported_at ? `${trace.present_tasks} Trace tasks · ${trace.completed_tasks} completed` : 'Bring your tasks into view'}</strong>
          <p>{trace.exported_at ? `Exported ${trace.exported_at} · Last import ${trace.imported_at}` : 'In Trace, press Ctrl+K → Export as JSON. Import that file here.'}</p>
          {#if trace.unresolved_dates}<p>{trace.unresolved_dates} source date{trace.unresolved_dates === 1 ? '' : 's'} need precision before becoming deadlines. Select a task to inspect the original value.</p>{/if}
        </div>
        <div class="trace-import-state"><span>Last import: {readable(trace.last_attempt)}</span><small>{desktop ? 'Snapshot only; later Trace edits need a new export.' : 'Browser preview: use synthetic exports only.'}</small></div>
      </section>{/if}
      <div class="workspace">
        <Horizon {data} {selectedId} onselect={(id) => { selectedId = id; }} />
        <aside class="inspector" aria-label="Selected item details">
          {#if selected}
            <div class="inspector-heading"><span class="detail-kind"><span class="species-shape {selected.species}"></span>{selected.species}</span><button class="close" aria-label="Close details" onclick={() => { selectedId = null; }}>×</button></div>
            <h2>{selected.title}</h2>
            <p class="detail-when">{selected.when}</p>
            <div class="source-box"><strong>{selected.ownership}</strong><span>{selected.source}</span>{#if selected.milestone}<span>◇ Marked as a milestone</span>{/if}</div>
            <dl class="state-details">
              <div><dt>State</dt><dd>{readable(selected.phase)}</dd></div>
              {#if selected.risk}<div><dt>Pressure</dt><dd class:risk={['tight', 'insufficient', 'overdue'].includes(selected.risk)}>{readable(selected.risk)}</dd></div>{/if}
            {#if selected.conditional}<div><dt>Source basis</dt><dd>Conditional; inspect freshness and coverage below</dd></div>{/if}
            </dl>
            <h3>Why this appears</h3>
            <ul class="reasons">{#each selected.reasons as reason}<li>{reason}</li>{/each}</ul>
            {#if selected.facts.length}<h3>Evidence and calculation</h3><dl class="facts">{#each selected.facts as fact}<div><dt>{fact.label}</dt><dd>{fact.value}</dd></div>{/each}</dl>{/if}
          {:else}
            <span class="detail-kind">A closer look</span>
            <h2>Time, with context.</h2>
            <p class="inspector-intro">Select an item to see what is recorded, what is estimated, and why it appears here.</p>
            {#if nextAnchor}<div class="next-anchor"><span>Next recorded anchor</span><strong>{nextAnchor.title}</strong><p>{nextAnchor.when}</p><button onclick={() => { selectedId = nextAnchor!.id; }}>Inspect anchor</button></div>{/if}
            <div class="legend" aria-label="Visual key">
              <p><span class="species-shape anchor"></span>Bars show fixed occurrences</p>
              <p><span class="species-shape deadline"></span>Markers show real deadlines</p>
              <p><span class="species-shape window"></span>Open spans show derived opportunity</p>
              <p><span class="species-shape suggestion"></span>Dashed shapes are advice</p>
            </div>
            <p class="small-note">A fitting window is not a reservation. Pressure describes one item's declared opportunity at a time.</p>
          {/if}
          <details class="source-status"><summary>Source state</summary>{#each data.sources as source}<div><span>{source.label}</span><strong class:qualified={source.health !== 'healthy'}>{readable(source.health)}</strong></div>{/each}<p>{personal ? 'Health describes the last imported snapshot. Update view re-evaluates this cache; import a new export for newer Trace facts.' : 'These sources are synthetic examples.'}</p></details>
        </aside>
      </div>
    {:else if !error}<div class="loading" role="status">Preparing the synthetic Horizon…</div>{/if}
  </main>
  <footer><span>Personal time, in perspective.</span><span>Prototype for exploring the shape of Horizon</span></footer>
</div>
