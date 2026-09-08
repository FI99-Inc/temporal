# Architecture

## Architecture goal

Keep the personal application simple while preserving a clean reusable temporal core.

Do not prematurely extract a public engine.

## Technology decisions

### Desktop

- Tauri 2
- Rust native/backend layer

Responsibilities:

- application lifecycle
- Windows integration
- local source adapters
- local persistence access where appropriate
- deterministic temporal engine modules
- secure credential integration
- background refresh where explicitly implemented
- IPC surface to Svelte

### Interface

- Svelte 5
- TypeScript
- Vite
- HTML/CSS/SVG for Horizon rendering

### Storage

- SQLite
- local machine only for v1

## Proposed module boundary

The eventual source tree may resemble:

```text
src/                        # Svelte UI

src-tauri/
  src/                      # Tauri application/native integration

crates/
  temporal-core/            # domain objects + temporal semantics
  temporal-ranking/         # pressure, recommendation, opportunity logic
  temporal-store/           # persistence abstractions / migrations
  source-trace/             # Trace adapter
  source-quercus/           # later gate

tests/
  fixtures/                 # synthetic, never personal
  scenarios/                # virtual-time scenarios
```

This is an architectural direction, not authorization to scaffold all modules immediately.

Follow `docs/BUILD-GATES.md`.

## Dependency direction

Preferred:

```text
source adapters
      ↓
normalized temporal objects
      ↓
temporal-core
      ↓
temporal-ranking
      ↓
application query/view model
      ↓
Horizon UI
```

UI must not contain hidden business rules that should live in the core.

Source adapters must not directly manipulate Horizon concepts.

## Clock abstraction

Core logic must receive time through an injectable clock abstraction.

Do not scatter direct wall-clock reads throughout ranking/domain logic.

This enables synthetic-time simulation and deterministic tests.

## Source normalization

External and local sources normalize into a shared model while retaining:

- provenance
- external identifiers
- source timestamps where useful
- certainty
- mutability/read-only state

Normalization must not erase source-specific truth.

## Determinism

Core ranking and pressure calculations should be deterministic for identical:

- objects
- settings
- clock
- source state

If AI is later used, AI output must enter as explicit candidate metadata/inference rather than silently changing sourced facts.

## Local inter-app communication

Trace and Temporal Engine remain separate applications.

Prefer a narrow, versioned local contract.

Do not directly mutate Trace's SQLite database.

Read access to Trace's data may be used only if no cleaner contract exists and must remain isolated behind the Trace adapter. The preferred long-term boundary is a supported export/query mechanism or local IPC/API owned by Trace.

## Credentials

Use OS-backed secure credential storage for external source credentials.

Never store OAuth tokens or Quercus tokens in plaintext configuration, repo files, logs, fixtures, or test snapshots.

## Background behavior

Keep background work minimal.

No always-running heavy inference.

Refresh only as required for useful source freshness.

The core application should remain useful offline with last-known local state clearly identified.

## Failure posture

Source failure is not absence.

If an import source fails, preserve last-known data with source health metadata.

Do not tell the user they are free/caught up based on an incomplete source refresh.

## FI99 boundary

No FI99 package, API, public SDK, or generic engine repository is created now.

If real use later proves `temporal-core` and `temporal-ranking` broadly reusable, extraction can occur behind their existing clean seams.
