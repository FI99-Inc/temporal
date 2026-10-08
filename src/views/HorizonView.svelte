<script lang="ts">
  import Horizon from '../Horizon.svelte';
  import LocalEditor from '../LocalEditor.svelte';
  import Today from '../Today.svelte';
  import ItemDetail from '../components/ItemDetail.svelte';
  import type { PersonalView, Snapshot } from '../lib/api.ts';
  import { zonedParts } from '../lib/calendar.ts';
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
  const risky = $derived(data.items.filter(i => i.species === 'deadline' && (i.risk === 'overdue' || i.risk === 'insufficient' || i.risk === 'tight')).sort((a, b) => (a.start ?? 0) - (b.start ?? 0)));
  const overdue = $derived(risky.filter(i => i.risk === 'overdue').length);
  const todayDate = $derived(zonedParts(data.now, data.zone).date);
  // Declared opportunity still ahead today, summed from the evaluated windows.
  const openToday = $derived(data.items
    .filter(i => i.species === 'window' && i.start !== null && i.end !== null && i.end > data.now && zonedParts(i.start, data.zone).date === todayDate)
    .reduce((sum, i) => sum + (i.end! - Math.max(i.start!, data.now)), 0));
  const noAvailability = $derived(Boolean(personal && !personal.local.usual_availability && !personal.local.availability.length));
  const pastUnhandled = $derived(personal?.calendars.reduce((sum, c) => sum + c.past_unhandled.length, 0) ?? 0);
  const degraded = $derived(personal?.calendars.filter(c => !c.hidden && c.health !== 'healthy') ?? []);
  const empty = $derived(Boolean(personal && !personal.calendars.length && !personal.local.anchors.length && !personal.local.anchor_series.length && !personal.trace.exported_at));
  const readable = (value: string) => value.replaceAll('_', ' ');

  function until(at: number): string {
    const minutes = Math.max(0, Math.round((at - data.now) / 60_000));
    if (minutes < 60) return `in ${minutes} min`;
    const hours = Math.floor(minutes / 60);
    if (hours < 24) return `in ${hours} h ${String(minutes % 60).padStart(2, '0')} min`;
    return `in ${Math.round(hours / 24)} days`;
  }
  function duration(ms: number): string {
    const minutes = Math.round(ms / 60_000);
    return minutes < 60 ? `${minutes} min` : `${Math.floor(minutes / 60)} h ${String(minutes % 60).padStart(2, '0')}`;
  }
  async function saveLocal(mutation: LocalMutation): Promise<void> {
    if (!(await onapply('Saved', [mutation]))) throw new Error('Not saved; see the message for details.');
  }
  $effect(() => { if (selectedId && !selected) selectedId = null; });
</script>

<section class="headline">
  <h1>Horizon</h1>
  <div class="aside"><strong>{data.date_label}</strong><br />Near time expands<br />Distant time compresses<br />Evaluated {data.clock_label}{personal ? '' : ' · virtual'}</div>
</section>

<div class="stats" role="group" aria-label="At a glance">
  <button class="stat" disabled={!nextAnchor} onclick={() => { if (nextAnchor) selectedId = nextAnchor.id; }}>
    <span class="label">Next fixed</span>
    <span class="stat-value">{nextAnchor?.title ?? '—'}</span>
    <span class="stat-sub">{nextAnchor ? `${until(nextAnchor.start!)} · ${nextAnchor.when.split(', ')[1]?.split(' – ')[0] ?? ''}` : 'Nothing fixed ahead'}</span>
  </button>
  <button class="stat" disabled={!risky.length} onclick={() => { if (risky[0]) selectedId = risky[0].id; }}>
    <span class="label">At risk</span>
    <span class="stat-value" class:signal={risky.length > 0}>{risky.length ? `${risky.length} deadline${risky.length === 1 ? '' : 's'}` : 'Clear'}</span>
    <span class="stat-sub">{risky.length ? `${overdue ? `${overdue} overdue · ` : ''}${risky[0].title}` : 'No tight, short, or overdue deadline'}</span>
  </button>
  <div class="stat">
    <span class="label">Worth doing</span>
    <span class="stat-value">{data.today.worth_doing.length ? `${data.today.worth_doing.length} fit today` : 'None fits'}</span>
    <span class="stat-sub">{data.today.worth_doing[0]?.title ?? 'Declare usual hours to get suggestions'}</span>
  </div>
  <div class="stat">
    <span class="label">Open time left today</span>
    <span class="stat-value">{openToday ? duration(openToday) : '0 min'}</span>
    <span class="stat-sub">{data.has_declarations ? 'Inside your declared hours' : 'No hours declared'}</span>
  </div>
</div>

{#if empty}
  <div class="notice"><span class="label">Setup</span><div><strong>Start by bringing in your time.</strong><p>Connect Google, Outlook, or Quercus calendars, or type an event in the line above.</p></div>
    <button class="primary" onclick={() => onnavigate?.('sources')}>Add calendars →</button></div>
{/if}
{#if noAvailability}
  <div class="notice"><span class="label">Setup</span><div><strong>Tell Temporal when you're usually willing to work.</strong><p>Suggestions only use time you declare; an empty calendar is not free time.</p></div>
    <button onclick={() => onnavigate?.('settings')}>Set usual hours →</button></div>
{/if}
{#if pastUnhandled}
  <div class="notice warn"><span class="label">Attention</span><div><strong>{pastUnhandled} past imported deadline{pastUnhandled === 1 ? '' : 's'} not marked handled.</strong><p>The feed cannot say what you submitted, so they count as overdue until you mark them.</p></div>
    <button onclick={() => onnavigate?.('sources')}>Review →</button></div>
{/if}
{#each degraded as calendar (calendar.id)}
  <div class="notice warn"><span class="label">Source</span><div><strong>{calendar.label}: {readable(calendar.health)}.</strong><p>Showing last-known events{calendar.last_success ? ` from ${calendar.last_success}` : ''}. Rows that depend on it are marked conditional.</p></div>
    <button onclick={() => onnavigate?.('sources')}>Open sources →</button></div>
{/each}

<Today today={data.today} {selectedId} onselect={(id) => { selectedId = id; }} />

<div class="section-bar"><span class="index">02</span><span>The shape of your time</span><span class="spacer"></span><span class="label">Fourteen days, compressed with distance</span></div>
<div class="workspace">
  <Horizon {data} {selectedId} onselect={(id) => { selectedId = id; }} />
  <aside class="inspector" aria-label="Selected item details">
    {#if selected}
      <ItemDetail item={selected} {personal} {busy} {onedit} {onapply} onclose={() => { selectedId = null; }} />
    {:else}
      <span class="detail-kind">Inspector</span>
      <h2>Time, with context.</h2>
      <p class="inspector-intro">Select anything to see what is recorded, what is estimated, and why it appears.</p>
      {#if nextAnchor}<div class="next-anchor"><span>Next fixed event</span><strong>{nextAnchor.title}</strong><p>{nextAnchor.when}</p><button onclick={() => { selectedId = nextAnchor!.id; }}>Inspect →</button></div>{/if}
      <div class="legend" aria-label="Visual key">
        <p><span class="species-shape anchor"></span>Solid bars are fixed events</p>
        <p><span class="species-shape deadline"></span>Diamonds are real deadlines</p>
        <p><span class="species-shape window"></span>Hatched spans are time you declared</p>
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
