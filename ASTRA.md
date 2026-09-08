# ASTRA — Durable Project State

This file is the compact handoff ledger for Temporal Engine.

It should remain concise. Detailed specifications live in `docs/`. Detailed implementation evidence belongs in commits, tests, and gate reports.

## Current state

**Project phase:** Temporal core proof

**Current gate:** Gate 1 — Temporal core proof (in progress)

**Current task:** 1.2 Implement explicit time primitives and the injected clock

**Status:** Gate 0 complete; Task 1.1 complete; Task 1.2 next

The subsequent user instruction authorizes building the rest of the application using wayfinder. The local map is `.scratch/temporal-engine/map.md`; it carries execution through the existing gates. Gate 1 currently contains only the minimal internal Rust workspace. No desktop shell, database, UI, or adapter has been implemented.

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

Resume **1.2 Implement explicit time primitives and the injected clock** in `docs/BUILD-GATES.md`, against `docs/DOMAIN-CONTRACT.md` version 1 and `docs/SCENARIOS.md`. Complete and verify each numbered task before committing and advancing. Do not scaffold the eventual desktop stack as part of Gate 1.

## Latest evidence

- Local Git repository initialized with no remote; untouched foundation preserved in `4bbc312` (`docs: establish temporal engine constitution`).
- Task 0.1: `d28fd89`, constitution audit and surgical clarifications.
- Task 0.2: `b342b8d`, normalized domain contract and bounded proof-v1 policy.
- Task 0.3: `8a8124b`, S01–S16 synthetic scenarios with arithmetic/timezone checks.
- Task 0.4: `309767d`, Gate 1 tasks 1.1–1.10 with acceptance and verification requirements.
- Fresh cross-document/baseline-diff review is complete; the Gate 0 report and exact completion marker are in `docs/BUILD-GATES.md`. The final evidence commit also clarifies retained historical Suggestion validation.
- Task 1.1: Rust 1.98.0/cargo 1.98.0 pinned; one unpublished library with no dependencies. Workspace metadata, locked check, format check, and diff whitespace check passed. MSVC build tools are installed; substantive test linking begins in 1.2. No remote is configured.
- No temporal tests or visual application checks have run yet. The scaffold check is not engine evidence.

## Open questions and residual limits

No unresolved semantic blocker remains for Gate 1. O-001–O-007 in `docs/DECISIONS.md` remain open: naming, compression, visual grammar, Trace transport/schema, calibration, travel provider, and local AI. Rust is pinned; timezone-rule/dependency versions are selected in 1.2. The wayfinder Trace research ticket is investigating source code only, without changing the current normalized contract or reading private task data.

proof-v1 measures individual pressure against declared opportunity; it does not allocate shared capacity or establish real-world calibration. Source adapters, visual usability, and personal-local behavior are still unproven. Preserve those limits in later gate reports.

## Handoff rule

On takeover, inspect commits and working tree first, then resume the first incomplete task in `docs/BUILD-GATES.md`.

Do not reconstruct project state from conversation memory when the repository can answer it.
