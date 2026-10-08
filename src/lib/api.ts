import { invoke, isTauri } from '@tauri-apps/api/core';
import type { LocalState, LocalMutation, TraceChoice } from './local.ts';
import type { CalendarRange } from './calendar-types.ts';
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
  tasks: TraceChoice[]; export_path: string | null;
}
export type Theme = 'system' | 'light' | 'dark' | 'rose';
export interface Settings {
  display_zone: string | null; week_starts_on: 'mon' | 'sun'; reminder_minutes: number | null;
  theme: Theme; default_event_minutes: number; keep_running_in_tray: boolean;
  day_start_hour: number; trace_export_path: string | null; open_at_login: boolean;
}
export interface SettingsPatch {
  display_zone?: string; week_starts_on?: 'mon' | 'sun'; reminder_minutes?: number; reminders_off?: boolean;
  theme?: Theme; default_event_minutes?: number; keep_running_in_tray?: boolean; day_start_hour?: number;
  open_at_login?: boolean;
}
export type CalendarKind = 'quercus' | 'google' | 'outlook' | 'calendar';
export type CalendarMode = 'events' | 'coursework';
export interface CalendarSummary {
  id: string; label: string; kind: CalendarKind; mode: CalendarMode; color: string;
  origin: 'file' | 'subscription'; origin_label: string; health: string;
  last_success: string | null; last_attempt: string; last_error: string | null;
  calendar_name: string | null; event_count: number; skipped: number; hidden: boolean;
  past_unhandled: string[];
}
/** Editing context for one Horizon item; the item itself is never changed. */
export interface ItemLink {
  editable: boolean; series_id: string | null; occurrence_date: string | null;
  place: string | null; notes: string | null; imported: boolean;
  source_id: string | null; color: string | null;
}
export interface PersonalView {
  view: Snapshot; trace: TraceInfo; local: LocalState; settings: Settings;
  calendars: CalendarSummary[]; links: Record<string, ItemLink>;
}
export interface ImportResult { personal: PersonalView; error: string | null }
export interface CalendarFileInput { label: string; kind: CalendarKind; mode: CalendarMode; color: string; file_name: string; text: string }
export interface CalendarUrlInput { label: string; kind: CalendarKind; mode: CalendarMode; color: string; url: string }
export interface CalendarPatch { id: string; label?: string; mode?: CalendarMode; color?: string; hidden?: boolean }

export const desktop = isTauri();

async function preview<T>(path: string, init?: RequestInit): Promise<T> {
  if (!import.meta.env.DEV) throw new Error('Open this build in the Temporal Engine desktop app.');
  const response = await fetch(`/__temporal/${path}`, init);
  if (!response.ok) throw new Error(await response.text());
  return response.json() as Promise<T>;
}

/** One personal command, through Tauri IPC or the development preview bridge. */
async function call<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  if (isTauri()) return invoke<T>('app_call', { command, args });
  return preview<T>('call', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ command, args }) });
}

function desktopOnly<T>(command: string, args: Record<string, unknown> = {}): Promise<T> {
  if (!isTauri()) return Promise.reject(new Error('This needs the Temporal Engine desktop app.'));
  return invoke<T>(command, args);
}

export const catalog = () => isTauri() ? invoke<Scenario[]>('scenario_catalog') : preview<Scenario[]>('scenario_catalog');
export const snapshot = (scenarioId: string, offsetMinutes: number) => isTauri()
  ? invoke<Snapshot>('evaluate_scenario', { scenarioId, offsetMinutes })
  : preview<Snapshot>(`evaluate_scenario?${new URLSearchParams({ scenarioId, offsetMinutes: String(offsetMinutes) })}`);

export const personalSnapshot = () => call<PersonalView>('personal_snapshot');
export const calendarRange = (from: number, to: number) => call<CalendarRange>('calendar_range', { from, to });
export const mutateLocal = (mutation: LocalMutation) => call<PersonalView>('mutate_local', { mutation });
export const mutateBatch = (mutations: LocalMutation[]) => call<PersonalView>('mutate_local_batch', { mutations });
export const updateSettings = (patch: SettingsPatch) => call<PersonalView>('update_settings', { patch });
export const importTrace = (json: string) => call<ImportResult>('import_trace_json', { json });
export const addCalendarFile = (input: CalendarFileInput) => call<PersonalView>('add_calendar_text', { ...input });
export const refreshCalendarFile = (id: string, text: string) => call<ImportResult>('refresh_calendar_text', { id, text });
export const updateCalendar = (patch: CalendarPatch) => call<PersonalView>('update_calendar', { ...patch });
export const removeCalendar = (id: string) => call<PersonalView>('remove_calendar', { id });

// Desktop-only: network, file dialogs, and the OS credential store.
export const addCalendarUrl = (input: CalendarUrlInput) => desktopOnly<PersonalView>('add_calendar_url', { input });
export const refreshCalendars = (id?: string) => desktopOnly<ImportResult>('refresh_calendars', { id: id ?? null });
export const pickTraceExport = () => desktopOnly<ImportResult | null>('pick_trace_export');
export const reimportTrace = () => desktopOnly<ImportResult | null>('reimport_trace');
export const forgetTraceExport = () => desktopOnly<void>('forget_trace_export');

/** Background refreshes announce themselves so open views can reload. */
export async function onChanged(handler: () => void): Promise<() => void> {
  if (!isTauri()) return () => {};
  const { listen } = await import('@tauri-apps/api/event');
  return listen('temporal://changed', handler);
}

/** Read a chosen file as strict UTF-8 text with a size limit. */
export async function readTextFile(file: File, limitBytes: number, label: string): Promise<string> {
  if (file.size > limitBytes) throw new Error(`${label} is larger than ${Math.round(limitBytes / 1024 / 1024)} MiB.`);
  return new TextDecoder('utf-8', { fatal: true }).decode(await file.arrayBuffer());
}
