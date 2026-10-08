# Settled Decisions and Open Questions

This file records product tradeoffs that should not be silently reopened during implementation.

## Settled decisions

### D-001 — Separate application

Temporal Engine is its own Windows application.

It is not implemented as a feature inside Trace.

Reason: Trace's value comes from low-friction task capture. The temporal system needs a much richer model and interface.

### D-002 — Horizon is primary

The home interface is nonlinear Horizon, not a traditional calendar grid.

Traditional views may be secondary utilities later.

### D-003 — Personal first

Build specifically for the first user.

Do not generalize for hypothetical public users.

### D-004 — FI99 is deferred

The temporal core may eventually become an FI99 engine, but only if real use demonstrates generalizable value.

No current work should create a public engine/SDK/API solely for that possibility.

### D-005 — Companion mobile app excluded

No companion mobile-app or phone integration.

Reason: cross-device synchronization adds complexity without improving the desktop core proof.

### D-006 — Desktop-first stack

Use Tauri 2 + Rust + Svelte 5 + TypeScript + SQLite.

### D-007 — Trace remains canonical for tasks

Trace remains the source of truth for ordinary captured tasks and their completion state.

Temporal Engine owns scheduling interpretation/annotations.

### D-008 — Facts and inference are separate

Source-provided facts may never be silently rewritten by inference.

### D-009 — No guilt rollover

A missed flexible suggestion does not become overdue merely because its suggested time passed.

### D-010 — Deterministic core

Pressure, ranking, opportunity, and Horizon placement logic must be deterministic and inspectable.

AI is optional assistance.

### D-011 — No rigid auto-time-blocking in v1

The product may recommend and surface pressure/windows.

It should not initially generate a rigid minute-by-minute daily plan.

### D-012 — Read-only external calendars first

When Google/Outlook integrations arrive, begin with read-only import.

### D-013 — Virtual time is a core engineering requirement

Temporal behavior must be testable with an injected clock and synthetic scenarios.

### D-014 — Calendar-replacement release (user directive, 2026-10-08)

The user asked to "finish temporal, fix the UI, engine logic, and get a fully
functioning version ready … to start using temporal as a true calendar
(replacement)", delivered as "a proper executable desktop app". Under the
authority order in `AGENTS.md`, this instruction reorders Gate 3 and authorizes
the following, without reopening D-001–D-013:

- Horizon stays Home. A secondary **Calendar** utility (Day, Week, Month) is
  now earned: it is the place to place, read, and edit fixed facts quickly.
- Local fixed events (Anchors) may repeat. A bounded local series is expanded
  by the application boundary into ordinary individual Anchors with stable
  derived identities; the core still receives only normalized occurrences.
- A **usual weekly availability** may be declared once. It expands into the
  explicit, non-overlapping availability declarations the core already
  consumes. An explicit one-off declaration takes precedence over the usual
  pattern for the time it covers. Nothing infers free time from an empty grid.
- **Read-only iCalendar import** from a chosen file or a subscription URL
  (Google, Outlook, Quercus/Canvas, or another calendar). The adapter expands
  recurrence into bounded occurrences keyed by the source's own recurrence
  identifier, keeps per-calendar source identity and health, and never writes
  back. A subscription URL is treated as a credential.
- **Desktop reminders** before fixed events and a Windows **installer**.
  Gate 3.6's "packaging" prohibition is superseded for this personal build;
  public distribution, signing, auto-update, and stores remain excluded.
- The Quercus REST adapter (Tasks 3.1–3.3) is deferred, not abandoned:
  Canvas's own calendar feed reaches coursework dates through the iCalendar
  boundary. The personal-local trial (3.5) now runs on this release.

Still excluded: mobile, cloud storage, accounts, telemetry, write-back to any
external source, machine-authored time blocking, and AI.

### D-015 — Marking an imported deadline handled (provisional)

An iCalendar feed (including Canvas/Quercus) reports deadlines but never their
submission state, and the domain contract lets only the owning source resolve
a source-owned deadline. Without a remedy every past assignment would remain
overdue forever, filling the radar with work already done.

The app therefore keeps an explicit, local **handled** acknowledgement beside
an imported deadline. It never writes a resolution into the source fact: the
deadline keeps its provenance, stays visible in the Calendar labelled
"handled", and only leaves pressure and the daily edit. Undo is one action.
Unacknowledged past imported deadlines remain overdue, as the contract says.
Revisit when a source can report fulfillment (the deferred Quercus adapter).

## Open questions

These are intentionally not blockers for documentation Gate 0 unless a gate explicitly requires them.

### O-001 — Product name

"Temporal Engine" is a working name.

Do not spend implementation time on branding yet.

### O-002 — Exact Horizon compression function

Needs prototyping and scenario testing.

The Gate 2 prototype uses `log1p(elapsed / 6h) / log1p(extent / 6h)` over a
14-day extent. The curve itself is still provisional; judge it during the Gate 3
personal trial, not from synthetic weeks.

### O-003 — Exact visual grammar

Rigidity, provenance, pressure, and object type need a distinctive but restrained visual system.

Design after the model is stable.

**Composition settled 2026-09-10.** Asked how the Gate 2 prototype reads, the
user chose "Direction holds": a finite Today leads and the compressed time
surface stays below it, keeping both and refining later from real use. Treat
that composition as decided. The compression curve, colour/shape treatment, and
density remain open pending the Gate 3 trial.

### O-004 — Trace transport

The initial personal transport is the supported Trace 1.0 JSON export imported
through an explicit file choice. It is a complete last-known snapshot, not a
live feed. The adapter keeps the app-owned cache when an export is partial,
incompatible, stale, out of order, or otherwise rejected. A future live local
contract may be considered after personal use demonstrates that a snapshot is
insufficient.

Do not directly mutate Trace's SQLite database.

### O-005 — Effort calibration model

Initial implementation should be simple/deterministic. Learning behavior is later.

### O-006 — Travel-time provider

Deferred.

### O-007 — Local AI

Deferred until deterministic parsing/ranking demonstrates an actual gap.
