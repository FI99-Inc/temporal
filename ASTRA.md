# ASTRA — Durable Project State

This file is the compact handoff ledger for Temporal Engine.

It should remain concise. Detailed specifications live in `docs/`. Detailed implementation evidence belongs in commits, tests, and gate reports.

## Current state

**Project phase:** Temporal core proof

**Current gate:** Gate 1 — Temporal core proof (in progress)

**Current task:** 1.9 Assemble deterministic evaluation and suggestion validity

**Status:** Gate 0 complete; Tasks 1.1–1.8 complete; Task 1.9 next

The subsequent user instruction authorizes building the rest of the application using wayfinder. The local map is `.scratch/temporal-engine/map.md`; it carries execution through the existing gates. Gate 1 has a minimal internal Rust workspace, verified injected-time primitives, separate domain/result types, source qualification, declared Windows, conflicts, fit rows, and individual opportunity aggregation. No desktop shell, database, UI, or adapter has been implemented.

## Settled posture

- Personal/private-first Windows application
- Horizon-first, not calendar-grid-first
- Tauri 2 + Rust + Svelte 5 + TypeScript + SQLite
- Trace is a separate source/capture system
- Quercus is an early source
- Google/Outlook are later read-only imports
- a companion mobile app integration is excluded
- Mobile is excluded
- Deterministic temporal engine; AI is optional assistance only
- Local-first
- FI99 extraction is a future possibility that must be earned through real use

## Immediate objective

Resume **1.9 Assemble deterministic evaluation and suggestion validity** in `docs/BUILD-GATES.md`, against `docs/DOMAIN-CONTRACT.md` version 1 and the validated S01–S16 fixture harness. Reuse lifecycle rows, source health, Windows, fit rows, and proof-v1 pressure from Tasks 1.5–1.8. Complete and verify each numbered task before committing and advancing; do not scaffold the eventual desktop stack as part of Gate 1.

## Latest evidence

- Local Git repository initialized with no remote; untouched foundation preserved in `4bbc312` (`docs: establish temporal engine constitution`).
- Task 0.1: `d28fd89`, constitution audit and surgical clarifications.
- Task 0.2: `b342b8d`, normalized domain contract and bounded proof-v1 policy.
- Task 0.3: `8a8124b`, S01–S16 synthetic scenarios with arithmetic/timezone checks.
- Task 0.4: `309767d`, Gate 1 tasks 1.1–1.10 with acceptance and verification requirements.
- Fresh cross-document/baseline-diff review is complete; the Gate 0 report and exact completion marker are in `docs/BUILD-GATES.md`. The final evidence commit also clarifies retained historical Suggestion validation.
- Task 1.1: Rust 1.98.0/cargo 1.98.0 pinned; one unpublished library with no dependencies. Workspace metadata, locked check, format check, and diff whitespace check passed. MSVC build tools are installed; substantive test linking begins in 1.2. No remote is configured.
- Task 1.2: 12 time-contract tests pass, including S11 DST/equality/offset/civil-boundary cases and S16 time/overflow constraints. Workspace check, clippy, all tests, and format pass locked/offline. MSVC test binaries linked and ran. Chrono 0.4.45/Chrono-TZ 0.10.4 pin bundled IANA 2025b; dependency features and source search show no system-clock/local-zone access.
- Task 1.3: 8 domain tests plus 3 compile-fail ownership checks pass; all 20 integration tests remain green. Workspace check/clippy/format pass locked/offline. Stored facts/user annotations, derived results, and inferred suggestions have separate types and typed reason payloads.
- Task 1.4: strict duplicate-key/schema parsing, canonical JSON, whole-snapshot graph/ownership/time/source validation, checked expiry preflight, and the clock-independent fulfillment resolver are implemented. 81 valid synthetic inputs and 51 rejection fixtures are covered by the executable manifest; locked/offline check, clippy, all tests/doctests, format, and diff checks pass. Lifecycle, pressure, adapters, and visual application behavior remain unimplemented and unverified.
- Task 1.5: pure Anchor/Deadline/Intention lifecycle, exact/date-only boundary handling, authoritative fulfillment phases, bounded weekly Routine expansion, stable occurrence keys, explicit outcomes/pause/rule edits, and historical occurrence inspection are implemented. Focused lifecycle and recurrence tests plus locked/offline workspace check, clippy, all tests/doctests, format, and diff checks pass. Source health, opportunity, pressure, adapters, and visual application behavior remain unimplemented and unverified.
- Task 1.6: deterministic source health and dependency coverage are implemented, including precedence, exact freshness equality, checked expiry, retained last-known records, local health, implicit factual dependencies, Trace task-catalog inheritance, and no-future-Anchor coverage for overdue/inactive checks. Focused source-health tests plus locked/offline workspace check, clippy, all tests/doctests, format, and diff checks pass. Opportunity, pressure, adapters, and visual application behavior remain unimplemented and unverified.
- Task 1.7: declared availability is clipped to the evaluation range; present Busy/Unknown Anchors are union-subtracted once, transparent/removed Anchors remain non-blocking, and positive conflicts are separate derived rows. Stable Windows carry declaration context/energy and source qualifications. Deterministic target × Window fit rows cover Task, standalone unresolved Deadline, active Intention, and current/future Routine targets with endpoint, routine-date, earliest-start, context, energy, chunk, definite-mismatch, unknown, and mathematical-zero semantics. Individual opportunity aggregation preserves known qualifying subtotals without shared allocation. `opportunity_contract` (4 tests) and `fit_contract` (6 tests) pass alongside the full locked/offline check, clippy, all tests/doctests (52 substantive integration tests plus 3 compile-fail doctests), format, and diff checks. Pressure, evaluation assembly, adapters, persistence, and UI remain unimplemented.
- Task 1.8: proof-v1 individual pressure is derived deterministically for every present Deadline with authoritative fulfillment ordering, exact integer E/O ratios, threshold risks, explicit zero/unknown/lower-bound outcomes, source qualifications, structured reasons, and individual-capacity limitations. `pressure_contract` (14 tests) plus the full locked/offline check, clippy, all tests/doctests (66 substantive integration tests plus 3 compile-fail doctests), format, and diff checks pass. Complete evaluation assembly, suggestion validity, adapters, persistence, and UI remain unimplemented.

## Open questions and residual limits

No unresolved semantic blocker remains for Gate 1. O-001–O-007 in `docs/DECISIONS.md` remain open: naming, compression, visual grammar, Trace transport/schema, calibration, travel provider, and local AI. Rust and timezone rules are pinned. The Trace research session found export-contract limitations but was interrupted before a durable final report; its ticket remains unresolved. This does not block the synthetic core proof.

proof-v1 measures individual pressure against declared opportunity; it does not allocate shared capacity or establish real-world calibration. Source adapters, visual usability, and personal-local behavior are still unproven. Preserve those limits in later gate reports.

## Handoff rule

On takeover, inspect commits and working tree first, then resume the first incomplete task in `docs/BUILD-GATES.md`.

Do not reconstruct project state from conversation memory when the repository can answer it.
