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

## Open questions

These are intentionally not blockers for documentation Gate 0 unless a gate explicitly requires them.

### O-001 — Product name

"Temporal Engine" is a working name.

Do not spend implementation time on branding yet.

### O-002 — Exact Horizon compression function

Needs prototyping and scenario testing.

### O-003 — Exact visual grammar

Rigidity, provenance, pressure, and object type need a distinctive but restrained visual system.

Design after the model is stable.

### O-004 — Trace transport

Preferred long-term transport is a narrow versioned local contract. Exact mechanism is not yet frozen.

Do not directly mutate Trace's SQLite database.

### O-005 — Effort calibration model

Initial implementation should be simple/deterministic. Learning behavior is later.

### O-006 — Travel-time provider

Deferred.

### O-007 — Local AI

Deferred until deterministic parsing/ranking demonstrates an actual gap.
