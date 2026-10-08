<script lang="ts">
  import { zonedParts } from '../lib/calendar.ts';
  import { draftFromQuick, hhmm, saveMutations, type EditTarget, type EventDraft } from '../lib/editor.ts';
  import type { LocalMutation } from '../lib/local.ts';
  import { describe, parseQuickAdd } from '../lib/quickadd.ts';

  let { zone, now, defaultMinutes, busy, onapply, onedit, input = $bindable() }: {
    zone: string;
    /** The app's current instant (frozen in the development preview). */
    now: number;
    defaultMinutes: number;
    busy: boolean;
    onapply: (label: string, mutations: LocalMutation[]) => Promise<boolean>;
    onedit: (target: EditTarget, draft: EventDraft) => void;
    input?: HTMLInputElement;
  } = $props();

  let text = $state('');
  let focused = $state(false);
  // Parsing is deterministic for the supplied instant and display zone.
  const parsed = $derived.by(() => {
    if (!text.trim()) return null;
    const wall = zonedParts(now, zone);
    return parseQuickAdd(text, { date: wall.date, time: hhmm(wall.minutes) }, defaultMinutes);
  });
  const ok = $derived(parsed && !('error' in parsed) ? parsed : null);
  const kindWord = $derived(ok ? ({ event: 'Event', deadline: 'Deadline', intention: 'Optional' } as const)[ok.kind] : '');

  async function add() {
    if (!ok || busy) return;
    const draft = draftFromQuick(ok, zone, defaultMinutes);
    if (await onapply(`Added “${ok.title}”`, saveMutations(draft, { kind: 'new' }))) text = '';
  }
  function details() {
    const draft: EventDraft = ok ? draftFromQuick(ok, zone, defaultMinutes) : (() => {
      const wall = zonedParts(now, zone);
      return draftFromQuick({ kind: 'event', title: text.trim(), date: wall.date, endDate: null, allDay: false, start: null, end: null, location: null, repeat: null, explicitDate: false, notes: [] }, zone, defaultMinutes);
    })();
    onedit({ kind: 'new' }, draft);
    text = '';
  }
  function keydown(event: KeyboardEvent) {
    if (event.key === 'Enter' && (event.altKey || event.shiftKey)) { event.preventDefault(); details(); }
    else if (event.key === 'Enter') { event.preventDefault(); void add(); }
    else if (event.key === 'Escape') { text = ''; input?.blur(); }
  }
</script>

<div class="quick" class:open={focused && text.trim()}>
  <svg viewBox="0 0 24 24" aria-hidden="true"><path d="M12 5v14M5 12h14" /></svg>
  <input bind:this={input} bind:value={text} onkeydown={keydown} onfocus={() => { focused = true; }} onblur={() => { focused = false; }}
    placeholder={'Add… e.g. "Lab Tue 2-4pm every week" or "PS3 due Fri 11:59pm"'} aria-label="Quick add" aria-describedby="quick-preview" disabled={busy} maxlength="300" />
  <kbd>/</kbd>
  {#if text.trim()}
    <div class="preview" id="quick-preview" role="status">
      {#if ok}
        <div class="line"><span class="kind {ok.kind}">{kindWord}</span><strong>{ok.title}</strong></div>
        <div class="desc">{describe(ok)}</div>
        {#if ok.notes.length}<div class="notes">{ok.notes.join(' · ')}</div>{/if}
        <div class="keys"><span><kbd>Enter</kbd> add</span><span><kbd>Shift</kbd>+<kbd>Enter</kbd> edit details</span><span><kbd>Esc</kbd> clear</span></div>
      {:else if parsed && 'error' in parsed}
        <div class="desc err">{parsed.error}</div>
        <div class="keys"><span><kbd>Shift</kbd>+<kbd>Enter</kbd> open the editor instead</span></div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .quick { position: relative; flex: 1; max-width: 640px; display: flex; align-items: center; gap: 8px; padding: 0 10px; border: 1px solid var(--line-strong); border-radius: 8px; background: var(--surface-2); }
  .quick:focus-within { border-color: var(--accent); background: var(--surface); box-shadow: 0 0 0 3px var(--accent-soft); }
  svg { width: 16px; height: 16px; stroke: var(--muted); fill: none; stroke-width: 2; stroke-linecap: round; flex-shrink: 0; }
  input { flex: 1; border: 0 !important; background: transparent !important; padding: 9px 0 !important; outline: none !important; font-size: 13.5px; }
  kbd { font: 11px var(--font); color: var(--faint); border: 1px solid var(--line-strong); border-bottom-width: 2px; border-radius: 4px; padding: 0 5px; background: var(--surface); }
  .quick:focus-within > kbd { display: none; }
  .preview { position: absolute; left: -1px; right: -1px; top: calc(100% + 6px); z-index: 40; background: var(--surface); border: 1px solid var(--line-strong); border-radius: 8px; box-shadow: var(--shadow-lg); padding: 12px 14px; display: none; gap: 6px; }
  .quick.open .preview { display: grid; }
  .line { display: flex; gap: 8px; align-items: center; font-size: 14px; }
  .kind { font-size: 10px; font-weight: 700; text-transform: uppercase; letter-spacing: .06em; padding: 2px 6px; border-radius: 4px; background: var(--anchor-soft); color: var(--anchor); }
  .kind.deadline { background: var(--deadline-soft); color: var(--deadline); }
  .kind.intention { background: var(--soft-bg); color: var(--soft); border: 1px dashed var(--soft); }
  .desc { font-size: 13px; color: var(--muted); }
  .desc.err { color: var(--risk); }
  .notes { font-size: 11px; color: var(--faint); }
  .keys { display: flex; gap: 14px; font-size: 11px; color: var(--faint); margin-top: 2px; }
</style>
