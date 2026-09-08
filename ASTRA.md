# ASTRA — Durable Project State

This file is the compact handoff ledger for Temporal Engine.

It should remain concise. Detailed specifications live in `docs/`. Detailed implementation evidence belongs in commits, tests, and gate reports.

## Current state

**Project phase:** Foundation

**Current gate:** Gate 0 — Constitution and implementation contract

**Current task:** Gate 0 verification and durable evidence report

**Status:** Gate 0 in progress; documentation only

No product source code has been authorized yet.

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

Complete fresh cross-document/Git verification and the Gate 0 evidence report. Advance the handoff to Task 1.1 only if every Gate 0 criterion is satisfied, then stop before Gate 1 implementation.

## Latest evidence

- Local Git repository initialized with no remote; untouched foundation preserved in `4bbc312` (`docs: establish temporal engine constitution`).
- Task 0.1 audited all foundation documents and made surgical clarifications; evidence is recorded in `docs/BUILD-GATES.md`.
- Task 0.2 defines `docs/DOMAIN-CONTRACT.md` version 1 and the bounded proof-v1 policy; independent semantic review is reconciled in its task evidence.
- Task 0.3 defines S01–S16 in `docs/SCENARIOS.md`; semantic review and local arithmetic/timezone checks are recorded in its task evidence.
- Task 0.4 defines Gate 1 tasks 1.1–1.10 with per-task acceptance, dependency, scope, and verification requirements. No application/core scaffold exists yet.
- There is no unresolved product-direction blocker; O-001 through O-007 remain open. Fresh Gate 0 verification and the final evidence commit remain.

## Handoff rule

On takeover, inspect commits and working tree first, then resume the first incomplete task in `docs/BUILD-GATES.md`.

Do not reconstruct project state from conversation memory when the repository can answer it.
