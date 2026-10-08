<script lang="ts">
  import Horizon from '../Horizon.svelte';
  import LocalEditor from '../LocalEditor.svelte';
  import Today from '../Today.svelte';
  import ItemDetail from '../components/ItemDetail.svelte';
  import type { PersonalView, Snapshot } from '../lib/api.ts';
  import type { EditTarget, EventDraft } from '../lib/editor.ts';
  import type { LocalMutation } from '../lib/local.ts';

  let { data, personal = null, busy, onedit, onapply, onnavigate }: {
    data: Snapshot;
    personal?: PersonalView | null;
    busy: boolean;
    onedit: (target: EditTarget, draft: EventDraft) => void;
    onapply: (label: string, mutations: LocalMutation[]) => Promise<boolean>;
    onnavigate?: (view: 'calendar' | 'sources' | 'settings') => void;
  } = $props();

  let selectedId = $state<string | null>(null);
  const selected = $derived(data.items.find(i => i.id === selectedId) ?? null);
  const nextAnchor = $derived(data.items.filter(i => i.species === 'anchor' && i.phase === 'upcoming' && i.start !== null).sort((a, b) => a.start! - b.start!)[0]);
  const noAvailability = $derived(Boolean(personal && !personal.local.usual_availability && !personal.local.availability.length));
  const pastUnhandled = $derived(personal?.calendars.reduce((sum, c) => sum + c.past_unhandled.length, 0) ?? 0);
  const degraded = $derived(personal?.calendars.filter(c => !c.hidden && c.health !== 'healthy') ?? []);
  const empty = $derived(Boolean(personal && !personal.calendars.length && !personal.local.anchors.length && !personal.local.anchor_series.length && !personal.trace.exported_at));
  const readable = (value: string) => value.replaceAll('_', ' ');

  async function saveLocal(mutation: LocalMutation): Promise<void> {
    if (!(await onapply('Saved', [mutation]))) throw new Error('Not saved; see the message for details.');
  }
  $effect(() => { if (selectedId && !selected) selectedId = null; });
</script>

<div class="page-heading">
  <div><h1>Horizon</h1><p>{data.date_label} · near time expands, distant time compresses</p></div>
  <span class="evaluated">Evaluated {data.clock_label}{personal ? '' : ' (virtual time)'}</span>
</div>

{#if empty}
  <div class="notice"><div><strong>Start by bringing in your time.</strong><p>Connect Google, Outlook, or Quercus calendars, or add events from the box above.</p></div>
    <button class="primary" onclick={() => onnavigate?.('sources')}>Add calendars</button></div>
{/if}
{#if noAvailability}
  <div class="notice"><div><strong>Tell Temporal when you're usually willing to work.</strong><p>Suggestions only use time you declare; an empty calendar is not free time.</p></div>
    <button onclick={() => onnavigate?.('settings')}>Set usual hours</button></div>
{/if}
{#if pastUnhandled}
  <div class="notice warn"><div><strong>{pastUnhandled} past imported deadline{pastUnhandled === 1 ? '' : 's'} not marked handled.</strong><p>The feed cannot say what you submitted, so they count as overdue until you mark them.</p></div>
    <button onclick={() => onnavigate?.('sources')}>Review in Sources</button></div>
{/if}
{#each degraded as calendar (calendar.id)}
  <div class="notice warn"><div><strong>{calendar.label}: {readable(calendar.health)}.</strong><p>Showing last-known events{calendar.last_success ? ` from ${calendar.last_success}` : ''}. Rows that depend on it are marked conditional.</p></div>
    <button onclick={() => onnavigate?.('sources')}>Open Sources</button></div>
{/each}

<Today today={data.today} {selectedId} onselect={(id) => { selectedId = id; }} />

<div class="workspace">
  <Horizon {data} {selectedId} onselect={(id) => { selectedId = id; }} />
  <aside class="inspector" aria-label="Selected item details">
    {#if selected}
      <ItemDetail item={selected} {personal} {busy} {onedit} {onapply} onclose={() => { selectedId = null; }} />
    {:else}
      <span class="detail-kind">A closer look</span>
      <h2>Time, with context.</h2>
      <p class="inspector-intro">Select anything to see what is recorded, what is estimated, and why it appears.</p>
      {#if nextAnchor}<div class="next-anchor"><span>Next fixed event</span><strong>{nextAnchor.title}</strong><p>{nextAnchor.when}</p><button onclick={() => { selectedId = nextAnchor!.id; }}>Inspect</button></div>{/if}
      <div class="legend" aria-label="Visual key">
        <p><span class="species-shape anchor"></span>Bars are fixed events</p>
        <p><span class="species-shape deadline"></span>Diamonds are real deadlines</p>
        <p><span class="species-shape window"></span>Open spans are time you declared</p>
        <p><span class="species-shape intention"></span>Dashed shapes are optional or advice</p>
      </div>
      <p class="small-note">A fitting window is not a reservation. Pressure describes one item against the time you declared.</p>
    {/if}
    <details class="source-status"><summary>Source state</summary>{#each data.sources as source}<div><span>{source.label}</span><strong class:qualified={source.health !== 'healthy'}>{readable(source.health)}</strong></div>{/each}<p>{personal ? 'Health describes each source’s last successful read.' : 'These sources are synthetic examples.'}</p></details>
  </aside>
</div>

{#if personal}
  <LocalEditor local={personal.local} tasks={personal.trace.tasks} now={data.now} zone={data.zone} {busy} onsave={saveLocal} />
{/if}

<style>
  .evaluated { font-size: 12px; color: var(--muted); }
  .notice strong { font-size: 13px; }
</style>
