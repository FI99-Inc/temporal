import { invoke, isTauri } from '@tauri-apps/api/core';
import type { LocalState, LocalMutation, TraceChoice } from './local.ts';
export type { LocalState } from './local.ts';

export interface Scenario { id: string; title: string; description: string }
export type Species = 'anchor' | 'deadline' | 'task' | 'intention' | 'routine' | 'window' | 'suggestion';
export interface Detail { label: string; value: string }
export interface Item {
  id: string; species: Species; title: string; source: string; ownership: string;
  phase: string; start: number | null; end: number | null; when: string;
  milestone: boolean; risk: string | null; conditional: boolean;
  facts: Detail[]; reasons: string[];
}
export interface TodayRow {
  id: string; species: Species; title: string; when: string; state: string;
  risk: string | null; conditional: boolean; notes: string[];
}
export interface TodayView {
  policy: string; date_label: string; day_end_label: string;
  fixed: TodayRow[]; fixed_preview: number;
  worth_doing: TodayRow[]; loose: TodayRow[];
  radar: TodayRow[]; radar_omitted: number;
}
export interface Tick { at: number; label: string }
export interface Snapshot {
  scenario: Scenario; now: number; end: number; offset_minutes: number; max_offset_minutes: number;
  zone: string; date_label: string; clock_label: string; ticks: Tick[];
  items: Item[]; sources: { label: string; health: string }[];
  has_declarations: boolean; today: TodayView;
}
export interface TraceInfo {
  exported_at: string | null; imported_at: string | null; present_tasks: number;
  completed_tasks: number; unresolved_dates: number; last_attempt: string;
  tasks: TraceChoice[];
}
export interface PersonalView { view: Snapshot; trace: TraceInfo; local: LocalState }
export interface ImportResult { personal: PersonalView; error: string | null }
export const desktop = isTauri();

async function request<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  if (isTauri()) return invoke<T>(command, args);
  if (!import.meta.env.DEV) throw new Error('Open this build in the Temporal Engine desktop app.');
  const response = command === 'import_trace_json' || command === 'mutate_local'
    ? await fetch(`/__temporal/${command}`, {
        method:'POST',
        headers:{'Content-Type':'application/json'},
        body: command === 'import_trace_json' ? String(args.json) : JSON.stringify(args.mutation),
      })
    : await fetch(`/__temporal/${command}?${new URLSearchParams(Object.entries(args).map(([k, v]) => [k, String(v)]))}`);
  if (!response.ok) throw new Error(await response.text());
  return response.json() as Promise<T>;
}

export const catalog = () => request<Scenario[]>('scenario_catalog');
export const snapshot = (scenarioId: string, offsetMinutes: number) => request<Snapshot>('evaluate_scenario', { scenarioId, offsetMinutes });
export const personalSnapshot = () => request<PersonalView>('personal_snapshot');
export const importTrace = (json: string) => request<ImportResult>('import_trace_json',{json});
export const mutateLocal = (mutation: LocalMutation) => request<PersonalView>('mutate_local', { mutation });
