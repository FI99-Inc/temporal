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

## Initial source rollout order

1. Trace tasks
2. Manually entered anchors, deadlines, intentions, and routines
3. Quercus / Canvas
4. Later: Google / Outlook read-only imports
5. Later only if earned: travel/location services

This is an integration order, not precedence for overwriting fields. Trace owns task-level state; other sources retain their own facts. Linking related objects must preserve each source's provenance.

A companion mobile app is explicitly out of scope. Do not add phone synchronization.

## Read order

Before implementing anything, read:

1. `AGENTS.md`
2. `ASTRA.md`
3. `docs/PRODUCT.md`
4. `docs/TEMPORAL-MODEL.md`
5. `docs/DOMAIN-CONTRACT.md`
6. `docs/HORIZON.md`
7. `docs/SCENARIOS.md`
8. `docs/ARCHITECTURE.md`
9. `docs/TRACE-CONTRACT.md`
10. `docs/PRIVACY.md`
11. `docs/DECISIONS.md`
12. `docs/BUILD-GATES.md`

The documents are part of the product contract, not background notes.

`AGENTS.md` defines authority and working discipline. `docs/DECISIONS.md` holds settled decisions; the product/spec documents elaborate them. Implementation contracts must refine those semantics, not override them. `docs/BUILD-GATES.md` controls work scope and evidence; `ASTRA.md` points to the first incomplete task. Neither progress document can settle a new product decision.

## Development

The initial executable proof is one internal Rust library in `crates/temporal-core`. The desktop shell and integrations follow later gates.

Use Rust **1.98.0**, pinned in `rust-toolchain.toml`, with rustfmt and clippy. On Windows, install the MSVC C++ build tools and Windows SDK (Visual Studio's Desktop development with C++ workload). The initial setup was checked with Visual Studio 2022 Community's MSVC toolchain. Rust dependencies are locked in `Cargo.lock`.

From the repository root:

```text
cargo metadata --no-deps --format-version 1
cargo check --workspace --all-targets --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
```

Keep private local inputs under ignored `local-private/`; repository fixtures must be synthetic. The [local wayfinder map](.scratch/temporal-engine/map.md) tracks remaining decisions. `ASTRA.md` and the gate evidence remain the implementation handoff.

Time calculations use pinned [Chrono 0.4.45](https://docs.rs/chrono/0.4.45/chrono/) with only its `std` feature and [Chrono-TZ 0.10.4](https://docs.rs/chrono-tz/0.10.4/chrono_tz/) with bundled IANA **2025b** rules. The core does not enable Chrono's system-clock or machine-local-zone features. Updating these pins is an explicit dependency change requiring the civil-time tests to pass. After initial dependency acquisition, the checks also run with Cargo's `--offline` flag.
