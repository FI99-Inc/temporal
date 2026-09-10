# ASTRA — Durable Project State

This is the compact handoff ledger. Specifications, tests, Git history, and the
reports in `docs/BUILD-GATES.md` carry the detailed evidence.

## Current state

**Project phase:** Primitive Windows application

**Current gate:** Gate 3 — Quercus and real personal trial (not started)

**Current task:** 3.1 Verify the Quercus source contract

**Status:** Gates 0, 1, and 2 complete; Task 3.1 next

The user has authorized building the application and emphasized prompt delivery
of a highly personal app. Keep implementation focused on a usable Horizon. One
internal Rust crate now evaluates validated snapshots into lifecycle, source
health, Windows, conflicts, fit, pressure, and suggestion-validity results.
The Windows Tauri/Svelte shell now presents twelve synthetic weeks with virtual
time, nonlinear geometry, separate temporal species, and inspectable evidence.
Trace 1.0 JSON imports into an app-owned SQLite cache with stable mappings,
atomic reconciliation, retained last-known state, and visible source health.
Local temporal forms and work annotations now persist in the same app-owned
store, with explicit completion and Routine outcomes. A bounded `today-v1` edit
selects Fixed, Worth doing, Loose, and On the radar from one evaluation and
explains itself without writing anything. The app is verified against synthetic
inputs and one snapshot source; it has never met real coursework.

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

Execute **3.1 Verify the Quercus source contract** from `docs/BUILD-GATES.md`:
read-only investigation of what Quercus/Canvas actually exposes, its
authentication, pagination, rate limits, error shapes, and the precision and
timezone of every date field, before any adapter code. Record what the source
does not guarantee. No credential, payload, or personal course record may enter
the repository.

Gate 3 contains six commit-sized tasks: source contract, OS-backed credential
storage, the read-only Quercus adapter and cache, two real sources presented
without merging, the personal-local trial, and gate verification. The trial is
where the model first meets real coursework; separate observation from change.

## Latest evidence

- Foundation import: `08b9d6b`; Gate 0 completion: `98d16af`. History preserved;
  no remote is configured.
- Gate 1 Tasks 1.1–1.8: `1b3b296` through `1154df2`. Task 1.9: `32e989c`.
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
- Task 2.3: typed local records, schema-v2 migration, transactional mutations,
  collapsed forms, retained Trace work picker, and explicit completion/outcomes.
  Nine local-store tests and nine frontend tests pass. Full workspace: 102
  integration tests plus 3 doctests; final changes rechecked with all 24 app
  tests, all-target/all-feature clippy, frontend type/build, format, and bundled
  Windows build. Manual synthetic forms, Tab/Enter submission, prefilled edit,
  completion/outcome display, and reload were checked in the browser preview.
  Full evidence and remaining editor limits are in the Task 2.3 report.
- Task 2.4: `docs/TODAY-POLICY.md` documents `today-v1` before its
  implementation in `src-tauri/src/today.rs`, projected through
  `presentation.rs` into `src/Today.svelte`. Selection reuses one core
  evaluation, clips fit to the remainder of the civil day, and produces
  revalidating `consider_work` advice with producing keys and expiry. Nine new
  tests and two compile-fail doctests; full workspace 111 integration tests and
  5 doctests, frontend tests/checks/build, and the bundled Windows build pass.
  Browser walkthrough covered S02, S03 over three days, S07, S09, and S10.
  Local time now re-evaluates each minute while visible and on focus.
- Task 2.5: full suite passed (111 integration tests, 5 doctests, 9 frontend
  tests, type/build checks, format, all-feature clippy with warnings denied),
  the bundled Windows executable launched with a responding titled window on
  real system time, and the shipped bundle has no development bridge and no
  Tauri filesystem/network/shell permission. All ten required weeks were
  reviewed at base time and one injected day later. A rejected partial import
  kept both tasks, seven local records, the annotation, and the import time
  while exposing `partial` health; re-import restored healthy state. A process
  restart preserved the whole cache. Gate 2 is complete; its report carries the
  full evidence.

## Open questions and residual limits

No unresolved blocker remains for Gates 0–2. O-001, O-002, and O-005–O-007 in
`docs/DECISIONS.md` remain open: naming, the compression curve, calibration,
travel provider, and local AI. O-004 is settled for the initial JSON-export
boundary. O-003's composition is settled: asked on 2026-09-10, the user chose
"Direction holds" — a finite Today leads with the compressed surface below,
keeping both and refining from real use — and chose to mark Gate 2 complete.
The colour/shape treatment and density stay open for the Gate 3 trial. Do not
extend those two selections into preferences the user did not state. The local
tracker is local Markdown kept outside this repository.

proof-v1 measures individual pressure against declared opportunity. It does not
allocate shared capacity or establish real-world calibration. today-v1 selects
and orders by explicit typed signals only; overlapping advisory ranges certify
nothing about combined capacity, and it holds no acceptance state. Source
adapter reconciliation, local persistence, and Today selection are proven
against synthetic inputs and have never met real coursework. Large personal
datasets and the personal-local trial remain ahead. A deadline due today can
appear in both Fixed and Worth doing; the Trace boundary stays a manual
snapshot, so a task changed in Trace is invisible until re-export. The local
editor rejects ambiguous DST times, has no lower-bound effort input, and uses
Toronto for display. Keep these limits visible.

## Handoff rule

Inspect commits and the working tree, read the documents in README order, and
resume the first incomplete task in `docs/BUILD-GATES.md`. Do not reconstruct
project state from conversation memory or restart completed work.
