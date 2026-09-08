# Temporal Engine

Temporal Engine is a private-first Windows application for representing a person's near future without reducing life to a conventional calendar grid.

It is not primarily a calendar client, task manager, AI scheduler, or time-blocking app.

Its central interface is **Horizon**: a continuously compressed, nonlinear view of upcoming time. It combines fixed events, deadlines, flexible tasks, intentions, routines, and derived scheduling pressure while keeping sourced facts visibly distinct from system inference.

## Product posture

Temporal Engine is currently a personal application.

Build it specifically for its first user. Do not compromise the product for hypothetical public users, plugin ecosystems, enterprise deployment, mobile sync, or an eventual FI99 extraction.

The architecture should preserve clean seams around the reusable temporal model and ranking logic. If those parts prove broadly useful through real use, they may later be extracted into an FI99 engine. That is a future decision, not a current requirement.

## Initial stack

- **Desktop shell / native integration:** Tauri 2 + Rust
- **Interface:** Svelte 5 + TypeScript
- **Storage:** SQLite
- **Primary platform:** Windows 11
- **Network posture:** local-first; external integrations are adapters
- **AI posture:** optional and subordinate; core scheduling behavior must remain deterministic and inspectable

## Initial source hierarchy

1. Trace tasks
2. Manually entered anchors, deadlines, intentions, and routines
3. Quercus / Canvas
4. Later: Google / Outlook read-only imports
5. Later only if earned: travel/location services

A companion mobile app is explicitly out of scope. Do not add phone synchronization.

## Read order

Before implementing anything, read:

1. `AGENTS.md`
2. `ASTRA.md`
3. `docs/PRODUCT.md`
4. `docs/TEMPORAL-MODEL.md`
5. `docs/HORIZON.md`
6. `docs/ARCHITECTURE.md`
7. `docs/TRACE-CONTRACT.md`
8. `docs/PRIVACY.md`
9. `docs/DECISIONS.md`
10. `docs/BUILD-GATES.md`

The documents are part of the product contract, not background notes.
