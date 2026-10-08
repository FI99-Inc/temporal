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
  <span class="prompt" aria-hidden="true">Add&nbsp;›</span>
  <input bind:this={input} bind:value={text} onkeydown={keydown} onfocus={() => { focused = true; }} onblur={() => { focused = false; }}
    placeholder={'Lab Tue 2-4pm every week  ·  PS3 due Fri 11:59pm  ·  maybe call grandma sunday'} aria-label="Quick add" aria-describedby="quick-preview" disabled={busy} maxlength="300" />
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
  .quick { position: relative; flex: 1; display: flex; align-items: center; gap: 14px; padding: 0 20px; min-width: 0; }
  .quick:focus-within { background: var(--surface); }
  .prompt { font-family: var(--font-mono); font-size: 11.5px; font-weight: 600; letter-spacing: .08em; text-transform: uppercase; color: var(--accent); white-space: nowrap; }
  input { flex: 1; border: 0 !important; background: transparent !important; padding: 12px 0 !important; outline: none !important; font-family: var(--font-mono) !important; font-size: 13px !important; min-width: 0; }
  kbd { font: 500 10.5px var(--font-mono); color: var(--muted); border: 1px solid var(--line-strong); padding: 1px 5px; background: transparent; }
  .quick:focus-within > kbd { display: none; }
  .preview { position: absolute; left: -1px; right: 0; top: 100%; z-index: 40; background: var(--bg); border: var(--rule); box-shadow: var(--shadow-lg); padding: 14px 20px; display: none; gap: 8px; }
  .quick.open .preview { display: grid; }
  .line { display: flex; gap: 10px; align-items: center; font-size: 15px; }
  .line strong { font-family: var(--font-display); font-stretch: 115%; font-weight: 760; text-transform: uppercase; letter-spacing: -.01em; }
  .kind { font: 600 10px var(--font-mono); text-transform: uppercase; letter-spacing: .08em; padding: 3px 6px; background: var(--invert-bg); color: var(--invert-ink); }
  .kind.deadline { background: var(--accent); color: var(--accent-ink); }
  .kind.intention { background: transparent; color: var(--ink); outline: 1px dashed var(--ink); outline-offset: -1px; }
  .desc { font-family: var(--font-mono); font-size: 12px; color: var(--ink); }
  .desc.err { color: var(--risk); }
  .notes { font-family: var(--font-mono); font-size: 11px; color: var(--muted); }
  .keys { display: flex; gap: 16px; font-family: var(--font-mono); font-size: 10.5px; text-transform: uppercase; letter-spacing: .05em; color: var(--muted); margin-top: 2px; }
</style>
