<script lang="ts">
  import { onMount } from 'svelte';
  import EventEditor from './components/EventEditor.svelte';
  import QuickAdd from './components/QuickAdd.svelte';
  import CalendarView from './views/CalendarView.svelte';
  import ExamplesView from './views/ExamplesView.svelte';
  import HorizonView from './views/HorizonView.svelte';
  import SettingsView from './views/SettingsView.svelte';
  import SourcesView from './views/SourcesView.svelte';
  import { desktop, mutateBatch, onChanged, personalSnapshot, reimportTrace, updateSettings, type ImportResult, type PersonalView, type SettingsPatch } from './lib/api.ts';
  import { blankDraft, hhmm, type EditTarget, type EventDraft } from './lib/editor.ts';
  import { zonedParts } from './lib/calendar.ts';
  import type { LocalMutation } from './lib/local.ts';

  type View = 'horizon' | 'calendar' | 'sources' | 'settings' | 'examples';
  type Toast = { id: number; text: string; error: boolean };
  let view = $state<View>(initialView());
  let personal = $state<PersonalView | null>(null);
  let busy = $state(false);
  let loadError = $state('');
  let toasts = $state<Toast[]>([]);
  let version = $state(0);
  let wall = $state(Date.now());
  // The desktop app follows the system clock; the development preview is frozen
  // at its injected evaluation time so every view agrees on "now".
  const now = $derived(desktop || !personal ? wall : personal.view.now);
  let editing = $state<{ target: EditTarget; draft: EventDraft; key: number } | null>(null);
  let quickInput = $state<HTMLInputElement>();
  let toastId = 0;

  const zone = $derived(personal?.view.zone ?? Intl.DateTimeFormat().resolvedOptions().timeZone);
  const clockParts = $derived(new Intl.DateTimeFormat('en-US', { timeZone: zone, hour: 'numeric', minute: '2-digit', hour12: true }).formatToParts(new Date(now)));
  const clock = $derived(clockParts.filter(p => p.type === 'hour' || p.type === 'minute' || p.type === 'literal').map(p => p.value).join('').trim().padStart(5, '0'));
  const meridiem = $derived(clockParts.find(p => p.type === 'dayPeriod')?.value ?? '');
  const weekday = $derived(new Intl.DateTimeFormat('en-US', { timeZone: zone, weekday: 'short' }).format(new Date(now)));
  const dayMonth = $derived(new Intl.DateTimeFormat('en-GB', { timeZone: zone, day: '2-digit', month: 'short' }).format(new Date(now)));
  const sourceLine = $derived(personal ? `${personal.calendars.filter(c => !c.hidden && c.health === 'healthy').length}/${personal.calendars.filter(c => !c.hidden).length} calendars current` : 'Opening store');
  const attention = $derived(personal ? personal.calendars.filter(c => !c.hidden && c.health !== 'healthy').length + (personal.calendars.some(c => c.past_unhandled.length) ? 1 : 0) : 0);

  function initialView(): View {
    try { const saved = localStorage.getItem('temporal.view'); if (saved === 'horizon' || saved === 'calendar' || saved === 'sources' || saved === 'settings') return saved; } catch { /* storage unavailable */ }
    return 'horizon';
  }
  $effect(() => { if (view !== 'examples') try { localStorage.setItem('temporal.view', view); } catch { /* storage unavailable */ } });
  $effect(() => {
    const theme = personal?.settings.theme ?? 'system';
    if (theme === 'system') delete document.documentElement.dataset.theme;
    else document.documentElement.dataset.theme = theme;
  });

  function toast(text: string, error = false) {
    const id = ++toastId;
    toasts = [...toasts, { id, text, error }];
    setTimeout(() => { toasts = toasts.filter(t => t.id !== id); }, error ? 9000 : 3500);
  }
  function accept(result: PersonalView | ImportResult | null | void): boolean {
    if (!result) return true;
    if ('view' in result) { personal = result; version++; return true; }
    personal = result.personal; version++;
    if (result.error) { toast(result.error, true); return false; }
    return true;
  }
  /** One operation at a time; failures become a visible message, never a silent loss. */
  async function run(label: string, task: () => Promise<PersonalView | ImportResult | null | void>): Promise<boolean> {
    if (busy) { toast('Another change is still saving.', true); return false; }
    busy = true;
    try {
      const ok = accept(await task());
      if (ok && label) toast(label);
      return ok;
    } catch (e) { toast(String(e instanceof Error ? e.message : e), true); return false; }
    finally { busy = false; }
  }
  const apply = (label: string, mutations: LocalMutation[]) => mutations.length
    ? run(label, () => mutateBatch(mutations)) : (label ? (toast(label, true), Promise.resolve(false)) : Promise.resolve(false));
  const saveSettings = (patch: SettingsPatch) => run('', () => updateSettings(patch));
  const mutateOne = (mutation: LocalMutation) => apply('Saved', [mutation]);

  async function refresh() {
    if (busy) return;
    try {
      if (desktop && personal?.trace.export_path) {
        const imported = await reimportTrace().catch(() => null);
        if (imported) { accept(imported); return; }
      }
      const next = await personalSnapshot();
      personal = next; version++; loadError = '';
    } catch (e) { loadError = String(e instanceof Error ? e.message : e); }
  }
  function openEditor(target: EditTarget, draft: EventDraft) { editing = { target, draft, key: Date.now() }; }
  function newEvent() {
    const parts = zonedParts(now, zone);
    openEditor({ kind: 'new' }, blankDraft(parts.date, zone, hhmm(Math.min(23 * 60, Math.ceil((parts.minutes + 1) / 30) * 30)), personal?.settings.default_event_minutes ?? 60));
  }

  onMount(() => {
    let stop: (() => void) | undefined;
    void (async () => {
      try {
        let first = await personalSnapshot();
        if (!first.settings.display_zone) {
          const system = Intl.DateTimeFormat().resolvedOptions().timeZone;
          first = await updateSettings({ display_zone: system }).catch(() => first);
        }
        personal = first; version++;
      } catch (e) { loadError = String(e instanceof Error ? e.message : e); }
      stop = await onChanged(() => { void refresh(); });
    })();
    // The daily edit must not go stale: re-evaluate every minute while visible and on focus.
    const tick = setInterval(() => { wall = Date.now(); if (document.visibilityState === 'visible') void refresh(); }, 60_000);
    const clockTick = setInterval(() => { wall = Date.now(); }, 15_000);
    const focus = () => { wall = Date.now(); void refresh(); };
    window.addEventListener('focus', focus);
    return () => { clearInterval(tick); clearInterval(clockTick); window.removeEventListener('focus', focus); stop?.(); };
  });

  function keydown(event: KeyboardEvent) {
    const target = event.target as HTMLElement | null;
    const typing = Boolean(target && (target.closest('input, textarea, select, dialog') || target.isContentEditable));
    if (editing) return;
    if ((event.ctrlKey || event.metaKey) && event.key.toLowerCase() === 'n') { event.preventDefault(); newEvent(); return; }
    if (typing || event.ctrlKey || event.metaKey || event.altKey) return;
    if (event.key === '/') { event.preventDefault(); quickInput?.focus(); }
    else if (event.key === 'n') { event.preventDefault(); newEvent(); }
    else if (event.key === 'h' || event.key === '1') view = 'horizon';
    else if (event.key === 'c' || event.key === '2') view = 'calendar';
    else if (event.key === '3') view = 'sources';
    else if (event.key === '4') view = 'settings';
  }
  const nav: { id: View; label: string; key: string }[] = [
    { id: 'horizon', label: 'Horizon', key: 'H' },
    { id: 'calendar', label: 'Calendar', key: 'C' },
    { id: 'sources', label: 'Sources', key: '3' },
    { id: 'settings', label: 'Settings', key: '4' },
  ];
</script>

<svelte:window onkeydown={keydown} />
<svelte:head><title>Temporal Engine</title></svelte:head>

<a class="skip-link" href="#main">Skip to content</a>
<div class="frame">
  <header class="masthead">
    <div class="mark"><span class="brand-mark" aria-hidden="true"><i></i><i></i><i></i></span><span class="wordmark">Temporal</span></div>
    <nav class="tabs" aria-label="Main">
      {#each nav as item, index (item.id)}
        <button class="tab" aria-current={view === item.id ? 'page' : undefined} title="{item.label} ({item.key})" onclick={() => { view = item.id; }}>
          <span class="index">0{index + 1}</span>{item.label}
          {#if item.id === 'sources' && attention}<span class="tab-badge" aria-label="{attention} need attention">{attention}</span>{/if}
        </button>
      {/each}
    </nav>
    <div class="readout" aria-label="Current time">
      <div class="readout-date">{weekday}<span>{dayMonth}</span></div>
      <strong class="readout-clock">{clock}<sup>{meridiem}</sup></strong>
    </div>
  </header>

  <div class="command">
    <QuickAdd {zone} {now} defaultMinutes={personal?.settings.default_event_minutes ?? 60} busy={busy || !personal} onapply={apply} onedit={openEditor} bind:input={quickInput} />
    <button class="primary" disabled={busy || !personal} onclick={newEvent} title="New event (N)">New event →</button>
  </div>

  <main id="main" class="page" class:flush={view === 'calendar'} aria-busy={busy}>
    {#if loadError && !personal}
      <div class="notice warn" role="alert"><span class="label">Error</span><div><strong>Temporal could not open its local store.</strong><p>{loadError}</p></div><button onclick={refresh}>Try again</button></div>
    {/if}
    {#if view === 'examples'}
      <ExamplesView onclose={() => { view = 'horizon'; }} />
    {:else if personal}
      {#if view === 'horizon'}
        <HorizonView data={personal.view} {personal} {busy} onedit={openEditor} onapply={apply} onnavigate={(next) => { view = next; }} />
      {:else if view === 'calendar'}
        <CalendarView {personal} {busy} {version} {now} onedit={openEditor} onapply={apply} />
      {:else if view === 'sources'}
        <SourcesView {personal} {busy} {run} />
      {:else if view === 'settings'}
        <SettingsView {personal} {busy} onsettings={saveSettings} onmutate={mutateOne} onexamples={() => { view = 'examples'; }} />
      {/if}
    {:else if !loadError}
      <div class="loading" role="status">Opening your time…</div>
    {/if}
  </main>

  <footer class="status">
    <span class="live">Temporal Engine</span>
    <span>v0.2.0</span>
    <span>{personal ? 'Local store' : desktop ? 'Opening store' : 'Preview'}</span>
    <span>{zone}</span>
    <span class:warn={attention > 0}>{sourceLine}</span>
    <span>No cloud · No telemetry</span>
  </footer>
</div>

{#if editing}
  {#key editing.key}
    <EventEditor target={editing.target} initial={editing.draft} {busy} onsave={apply} onclose={() => { editing = null; }} />
  {/key}
{/if}

<div class="toast-stack" aria-live="polite">
  {#each toasts as t (t.id)}
    <div class="toast" class:error={t.error} role={t.error ? 'alert' : 'status'}><p>{t.text}</p><button class="ghost" aria-label="Dismiss" onclick={() => { toasts = toasts.filter(x => x.id !== t.id); }}>×</button></div>
  {/each}
</div>
