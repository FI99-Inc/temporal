# ASTRA — Durable Project State

This is the compact handoff ledger. Specifications, tests, Git history, and the
reports in `docs/BUILD-GATES.md` carry the detailed evidence.

## Current state

**Project phase:** Primitive Windows application

**Current gate:** Gate 2 — Trace-backed primitive Horizon (not started)

**Current task:** 2.1 Run a synthetic Horizon in the Windows app

**Status:** Gates 0 and 1 complete; Task 2.1 next

The user has authorized building the application and emphasized prompt delivery
of a highly personal app. Keep implementation focused on a usable Horizon. One
internal Rust crate now evaluates validated snapshots into lifecycle, source
health, Windows, conflicts, fit, pressure, and suggestion-validity results.
The desktop shell, persistence, source adapters, and UI remain unimplemented.

## Settled posture

- Personal/private-first Windows application; nonlinear Horizon is Home.
- Tauri 2 + Rust + Svelte 5 + TypeScript + SQLite.
- Trace remains separate and canonical for task capture/completion; no direct
  mutation of its database. Quercus follows in Gate 3.
- Facts, user state, derived results, and suggestions remain distinct.
- Deterministic behavior with injected time; flexible advice never becomes debt.
- No runtime agent dependency, mobile companion, cloud personal database,
  accounts,
  default telemetry, or premature FI99/public platform work.

## Immediate objective

Execute **2.1 Run a synthetic Horizon in the Windows app** from
`docs/BUILD-GATES.md`: a minimal runnable Tauri/Svelte app presenting the ten
required synthetic weeks through the existing core, with virtual-time controls
and selectable explanations. Show the prototype for user feedback. Do not
reopen the completed core proof or delay the visible app for extra architecture.

Gate 2 contains five commit-sized tasks: synthetic app, safe Trace read/cache,
local input/annotations, finite Today/advice, and app verification. Its Trace
contract investigation is required before the adapter, not before Task 2.1.

## Latest evidence

- Foundation import: `4bbc312`; Gate 0 completion: `f504182`. History preserved;
  no remote is configured.
- Gate 1 Tasks 1.1–1.8: `d6cb671` through `4063c6f`. Task 1.9: `9f27f14`.
  Separate task commits implement the domain contract and complete evaluator.
- Task 1.10: fresh Gate 1 diff/contract/scope review and manual synthetic output
  inspection are recorded in the Gate 1 report. All 78 integration tests and
  3 compile-fail ownership doctests pass; none failed or ignored. Explicit
  expectations cover 81 valid scenario inputs; 51 invalid fixtures are rejected.
  Repeated/permuted full outputs are canonical and leave their inputs unchanged.
- Format, all-target check, clippy with warnings denied, all tests, and diff
  checks pass. Cargo verification ran locked/offline. Rust/cargo 1.98.0,
  Chrono 0.4.45, Chrono-TZ 0.10.4, IANA 2025b; no new dependency in final review.
- Both completion markers and their evidence are in `docs/BUILD-GATES.md`.
  No Gate 2 implementation or dependency acquisition occurred during this handoff.

## Open questions and residual limits

No unresolved blocker remains for Gate 1 or the synthetic app in Task 2.1.
O-001–O-007 in `docs/DECISIONS.md` remain open: naming, compression, visual
grammar, Trace transport/schema, calibration, travel provider, and local AI.
Compression/visual grammar need prototype feedback. Interrupted Trace research
has not established a durable supported transport; inspect and resolve it in 2.2.
The local tracker is `.scratch/temporal-engine/map.md`.

proof-v1 measures individual pressure against declared opportunity. It does not
allocate shared capacity or establish real-world calibration. Source adapter
reconciliation, snapshot revision production, visual usability, large personal
datasets, and personal-local behavior remain unproven. Keep these limits visible.

## Handoff rule

Inspect commits and the working tree, read the documents in README order, and
resume the first incomplete task in `docs/BUILD-GATES.md`. Do not reconstruct
project state from conversation memory or restart completed work.
