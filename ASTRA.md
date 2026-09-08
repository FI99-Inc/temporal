# ASTRA — Durable Project State

This file is the compact handoff ledger for Temporal Engine.

It should remain concise. Detailed specifications live in `docs/`. Detailed implementation evidence belongs in commits, tests, and gate reports.

## Current state

**Project phase:** Foundation  
**Current gate:** Gate 0 — Constitution and repository foundation  
**Current task:** 0.1 — Review and ratify the initial constitution  
**Status:** NOT STARTED IN CODE

No product source code has been authorized yet.

## Settled posture

- Personal/private-first Windows application
- Horizon-first, not calendar-grid-first
- Tauri 2 + Rust + Svelte 5 + TypeScript + SQLite
- Trace is a separate source/capture system
- Quercus is an early source
- Google/Outlook are later read-only imports
- a companion mobile app integration is excluded
- Mobile is excluded from the initial product
- Deterministic temporal engine; AI is optional assistance only
- Local-first
- FI99 extraction is a future possibility that must be earned through real use

## Immediate objective

Validate that the product constitution is coherent enough to support implementation without inventing missing semantics in code.

The first Codex/Astra run should:

1. inspect these documents as a system
2. identify contradictions, missing definitions, or implementation-blocking ambiguity
3. make only surgical documentation corrections that preserve the agreed product
4. produce the executable Gate 1 implementation plan
5. stop before scaffolding product code unless the user's prompt explicitly authorizes it

## Handoff rule

On takeover, inspect commits and working tree first, then resume the first incomplete task in `docs/BUILD-GATES.md`.

Do not reconstruct project state from conversation memory when the repository can answer it.
