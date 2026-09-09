# ASTRA — Durable Project State

This is the compact handoff ledger. Specifications, tests, Git history, and the
reports in `docs/BUILD-GATES.md` carry the detailed evidence.

## Current state

**Project phase:** Primitive Windows application

**Current gate:** Gate 2 — Trace-backed primitive Horizon (in progress)

**Current task:** 2.3 Store local temporal input and scheduling annotations

**Status:** Gates 0 and 1 complete; Tasks 2.1 and 2.2 complete; Task 2.3 next

The user has authorized building the application and emphasized prompt delivery
of a highly personal app. Keep implementation focused on a usable Horizon. One
internal Rust crate now evaluates validated snapshots into lifecycle, source
health, Windows, conflicts, fit, pressure, and suggestion-validity results.
The Windows Tauri/Svelte shell now presents twelve synthetic weeks with virtual
time, nonlinear geometry, separate temporal species, and inspectable evidence.
Trace 1.0 JSON imports into an app-owned SQLite cache with stable mappings,
atomic reconciliation, retained last-known state, and visible source health.
Local input forms, annotations, and Today selection remain next.

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

Execute **2.3 Store local temporal input and scheduling annotations** from
`docs/BUILD-GATES.md`. Keep Trace facts read-only while adding the smallest
personal local state needed for a useful Horizon. Never read a real task
database for research or fixtures, write Trace's database, or infer its
due/completion semantics from field names.

Gate 2 contains five commit-sized tasks: synthetic app, safe Trace read/cache,
local input/annotations, finite Today/advice, and app verification. The prototype
is available for feedback (README launch commands). Exact compression and visual
grammar remain provisional; do not invent user feedback to close O-002/O-003.

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
- Task 2.1: root frontend, one internal app workspace member, narrow synthetic
  IPC/projection, and deterministic geometry. All ten required Horizon weeks
  plus S12/S14 use the existing core fixtures. No core source or spec changed.
- App/frontend checks pass: 4 presentation tests, 4 geometry tests, all 78 core
  integration tests and 3 doctests; format, all-target native check, all-feature
  clippy with warnings denied, and frontend type/build checks. Final targeted
  checks cover the added stale-source assertion and frontend selection fix.
- All ten required weeks were inspected in the browser preview against the
  same Rust boundary. Clock/risk, overdue versus expired advice, source health,
  keyboard details controls, and desktop widths were checked; browser logs clean.
  Native bundled debug executable launched on Windows with its titled window
  and WebView content. Native launch required normal access for its WebView
  profile; interaction verification used the browser preview.
- Tauri 2.11.5, Svelte 5.57.0, TypeScript 6.0.3, Vite 8.2.2; Node 24.15.0.
  Task 2.1's synthetic prototype had no persistence; Task 2.2 adds only the
  app-owned SQLite Trace snapshot cache. There is no Trace database access,
  recommendation generator, network service in the bundled app, or remote
  publishing.
- Task 2.2: the verified Trace 1.0 JSON export is imported through an explicit
  file boundary into `temporal-engine.sqlite3` under app data. Stable opaque
  external IDs map to UUIDv4 Task References; unresolved due values remain
  source facts; failed, stale, incompatible, older, and partial imports keep
  the prior cache. Eleven adapter/cache tests and package-level app/core checks
  pass. The preview cache was populated only with the committed synthetic file.

## Open questions and residual limits

No unresolved blocker remains for Gate 1 or Tasks 2.1–2.2. O-001–O-003 and
O-005–O-007 in `docs/DECISIONS.md` remain open: naming, compression, visual
grammar, calibration, travel provider, and local AI. O-004 is settled for the
initial JSON-export boundary; a future live contract remains optional.
Compression/visual grammar need feedback on the now-running prototype; crowded
weeks require vertical scrolling and fixture aliases are terse. The local
tracker is `.scratch/temporal-engine/map.md`.

proof-v1 measures individual pressure against declared opportunity. It does not
allocate shared capacity or establish real-world calibration. Source adapter
reconciliation is proven against synthetic exports; local input, visual
usability, large personal datasets, and personal-local behavior remain next.
Keep these limits visible.

## Handoff rule

Inspect commits and the working tree, read the documents in README order, and
resume the first incomplete task in `docs/BUILD-GATES.md`. Do not reconstruct
project state from conversation memory or restart completed work.
