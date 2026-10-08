// The Calendar utility's read model. Rust produces these rows from stored facts;
// the UI only lays them out. Nothing here is advice or a reservation.

export type CalKind = 'anchor' | 'deadline' | 'intention' | 'routine';
export type SourceKind = 'local' | 'trace' | 'quercus' | 'google' | 'outlook' | 'calendar';
export type Weekday = 'mon' | 'tue' | 'wed' | 'thu' | 'fri' | 'sat' | 'sun';

export interface CalendarEvent {
  /** Stable item id; equals the Horizon item id for the same fact. */
  id: string;
  kind: CalKind;
  title: string;
  /** Epoch ms. For all-day rows: start of `start_date` in the display zone. */
  start: number;
  /** Epoch ms, exclusive. A deadline is a point: `end === start`. */
  end: number;
  all_day: boolean;
  /** `YYYY-MM-DD` first civil date for all-day/date-precision rows, else null. */
  start_date: string | null;
  /** `YYYY-MM-DD` exclusive end date for all-day rows, else null. */
  end_date: string | null;
  source_id: string;
  source_label: string;
  source_kind: SourceKind;
  /** Hex colour chosen for an imported calendar; null uses the kind's token. */
  color: string | null;
  /** Only local records can be edited here. Imported rows are read-only. */
  editable: boolean;
  /** The local series this occurrence came from, if any. */
  series_id: string | null;
  /** `YYYY-MM-DD` occurrence date within a series or routine, if any. */
  occurrence_date: string | null;
  location: string | null;
  /** Display-only notes for local events and deadlines. */
  notes: string | null;
  occupancy: 'busy' | 'transparent' | 'unknown' | null;
  tentative: boolean;
  /** Lifecycle label from the stored fact, e.g. upcoming, overdue, satisfied. */
  state: string;
  /** Pressure risk for deadlines inside the Horizon range, else null. */
  risk: string | null;
}

export interface AvailabilitySpan { start: number; end: number }

export interface CalendarRange {
  from: number;
  to: number;
  zone: string;
  events: CalendarEvent[];
  availability: AvailabilitySpan[];
}
