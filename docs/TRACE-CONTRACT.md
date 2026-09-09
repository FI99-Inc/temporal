# Trace Integration Contract

## Relationship

Trace is the fast task-capture system.

Temporal Engine is the time interpretation/navigation system.

Neither replaces the other.

## Source of truth

Trace remains authoritative for ordinary task identity and task-level state such as:

- task text
- task status
- priority
- context
- due date captured by Trace
- completion

Temporal Engine may maintain its own scheduling annotations keyed to a Trace task identifier.

Examples:

- estimated effort
- energy class
- earliest useful start
- temporal preference
- location/context refinement
- ranking history

Do not burden Trace's capture schema with Temporal Engine-only metadata in the first version.

## Expected conceptual fields from Trace

The intended mapping uses this conceptual set. This foundation does not include a verified Trace wire schema; the adapter gate must verify the actual supported contract before relying on field names or encodings:

- stable task id
- text/raw input
- status: Now / Later / Someday / Done
- context
- priority
- due date
- created/updated/completed timestamps
- sort order
- optional link

The adapter must treat unknown future fields conservatively.

Trace's Now / Later / Someday states are source task states, not timestamps or deadlines. A due value can become a normalized Deadline only when its meaning and time precision are established by the supported contract. Preserve unresolved source values without guessing a cutoff. Temporal Engine annotations must not overwrite Trace's priority, context, due value, or completion.

## Mutability

Initial integration posture should be conservative.

Preferred first stage:

- Temporal Engine reads task state
- Temporal Engine stores its own annotations
- completing or editing Trace tasks from Temporal Engine is deferred until a deliberate bidirectional contract exists

Do not directly write into Trace's database from Temporal Engine.

## Synchronization semantics

The adapter must handle:

- Trace unavailable
- database/schema version mismatch
- deleted task
- completed task
- due date changed in Trace
- task moved between Now/Later/Someday
- task text edited
- duplicate/imported state prevention

Source failure must be visible in source health state.

## Identity

Never match tasks by text alone.

Use a stable Trace task identifier.

Temporal Engine annotations must survive ordinary Trace edits where the task ID remains stable.

## Companion surface

A future Horizon bay may live visually inside a companion application's own
sidebar surface.

That is a presentation integration, not permission to merge the applications.

The bay may show a very small amount of temporal state such as:

- next anchor
- time until next anchor
- current usable window
- one recommended item

That surface must not become a second full Horizon.

## Versioning

Any local contract between Trace and Temporal Engine must be versioned.

Breaking changes require explicit migration/fallback behavior.

## Privacy

Never copy a real Trace database into repository fixtures.

Use synthetic task fixtures.

## Verified initial transport — Task 2.2

Source inspection on 2026-09-09 read a clean checkout of Trace's own source.
Trace is a separate private application; its internals are deliberately not
reproduced here. No personal database or export was read.

Trace's supported JSON export is the initial transport. It emits
`{version:"1.0", exported_at:<ISO instant>, tasks:[...]}` covering the whole
task catalog, and the export command becomes available only after Trace's
first successful load. This is a complete last-known catalog at export time,
not a live connection and not a guarantee about an unsaved edit. No supported
task-read endpoint exists besides the export. Use the export; do not fall back
to Trace database access when this boundary exists.

The app accepts one personal Trace instance, selected explicitly through a JSON
file input. The native import receives bytes, not a database path or SQL command.
The input must fit the 10 MiB bound. Version, export time, and tasks are required;
unknown envelope fields or versions are incompatible. Parse and validate every
task before writing any facts. Unknown task fields are ignored. IDs must be
nonempty and unique within the export. Never infer a missing required field from
an older cached record. The nullable `link` field may be absent in older 1.0 dumps.

| Trace field | Normalization |
| --- | --- |
| `id` | Opaque external ID; persistent mapping to a caller-generated UUIDv4 TaskRefId. Source ID is also allocated once and retained. |
| `text`, `raw_input`, `context`, `link` | Preserve source strings; title must be nonempty. Links are displayed as data, never automatically opened. |
| `status` | `now`, `later`, `someday`, `done` map directly. Other nonempty strings map to unknown with the original label. |
| `priority` | Integer 0–5 preserved as an opaque source-priority string. No ranking multiplier. |
| `due_at` | Null means no due value. Every non-null string remains `TaskDue::Unresolved` with its original value and an explicit precision/zone explanation. |
| `created_at`, `updated_at`, `completed_at` | Parse source ISO instants into provenance/completion fields; preserve unknown completion time. Reject completed timestamps on known unfinished states and invalid timestamp order. |
| `sort_order` | Preserve the finite fractional number in the app cache. Derive an integer ordinal within each status, sorting by `(sort_order, external_id)`, for the core's ordering hint. Never truncate a fraction. |

Trace's own capture mixes date-only values, local end-of-day relative dates,
and hour offsets into the same `due_at` string. The export omits the original zone
and precision. Parsing that string or `raw_input` again cannot recover a reliable
cutoff. This does not authorize converting an unresolved date to a Deadline or
rewriting Trace. A future supported source contract may resolve these values.

### Reconciliation, age, and storage

- Store only in Temporal Engine's own app-data `temporal-engine.sqlite3` file.
  Its versioned schema holds source state, original imported fields, canonical
  mappings, and normalized Task References. Never open Trace's database.
- A validated, newer complete export replaces the present catalog atomically.
  Absent IDs become removed tombstones; their canonical IDs and annotations stay.
  A genuine empty complete export retires the catalog. Done is completion, not
  deletion; uncompletion and reappearance keep the same identity.
- Older exports and conflicting payloads with the same export timestamp are
  rejected. Identical repeated input keeps object identity and fact revision.
  A changed fact/presence gets a higher adapter revision; do not rely on source
  `updated_at` as a monotonic version (Trace undo can restore older fields).
- Malformed, duplicate, partial, incompatible, or out-of-order loads retain the
  previous facts. Record the attempt/outcome separately and expose its health.
  Local I/O/transaction failures do not turn an old catalog into an empty one.
- The initial maximum snapshot age is 24h, an explicit source-freshness setting,
  not a risk weight. Reject an export at least 24h old or from the future while
  retaining the prior cache. On successful import at `now`, set last attempt and
  success to `now`, and `fresh_for_ms = 24h - (now - exported_at)`. Thus freshness
  expires at the same absolute boundary even if the same file is imported again.
  Preserve exported_at and actual import time separately for inspection.
- Every core evaluation gets one captured/injected current instant. Advancing
  evaluation time changes source health, not stored task status or effort.
  A system clock earlier than cached audit timestamps is an error, not permission
  to rewrite those timestamps. Synthetic tests inject time and never sleep.

This contract permits Task 2.2 implementation; it does not itself prove the
adapter or complete Gate 2. Live refresh, bidirectional writes, automatic due
normalization, and other Trace instances remain outside this initial transport.
