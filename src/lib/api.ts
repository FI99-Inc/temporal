import { invoke, isTauri } from '@tauri-apps/api/core';

export interface Scenario { id: string; title: string; description: string }
export type Species = 'anchor' | 'deadline' | 'task' | 'intention' | 'routine' | 'window' | 'suggestion';
export interface Detail { label: string; value: string }
export interface Item {
  id: string; species: Species; title: string; source: string; ownership: string;
  phase: string; start: number | null; end: number | null; when: string;
  milestone: boolean; risk: string | null; conditional: boolean;
  facts: Detail[]; reasons: string[];
}
export interface Tick { at: number; label: string }
export interface Snapshot {
  scenario: Scenario; now: number; end: number; offset_minutes: number; max_offset_minutes: number;
  zone: string; date_label: string; clock_label: string; ticks: Tick[];
  items: Item[]; sources: { label: string; health: string }[];
  has_declarations: boolean;
}

async function request<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  if (isTauri()) return invoke<T>(command, args);
  if (!import.meta.env.DEV) throw new Error('Open this build in the Temporal Engine desktop app.');
  const response = await fetch(`/__temporal/${command}?${new URLSearchParams(Object.entries(args).map(([k, v]) => [k, String(v)]))}`);
  if (!response.ok) throw new Error(await response.text());
  return response.json() as Promise<T>;
}

export const catalog = () => request<Scenario[]>('scenario_catalog');
export const snapshot = (scenarioId: string, offsetMinutes: number) => request<Snapshot>('evaluate_scenario', { scenarioId, offsetMinutes });
