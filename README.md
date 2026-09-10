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
12. `docs/TODAY-POLICY.md`
13. `docs/BUILD-GATES.md`

The documents are part of the product contract, not background notes.

`AGENTS.md` defines authority and working discipline. `docs/DECISIONS.md` holds settled decisions; the product/spec documents elaborate them. Implementation contracts must refine those semantics, not override them. `docs/BUILD-GATES.md` controls work scope and evidence; `ASTRA.md` points to the first incomplete task. Neither progress document can settle a new product decision.

## Development

Gates 0 and 1 are complete. Gate 2 now has a runnable synthetic Horizon, a
read-only Trace 1.0 JSON import into Temporal Engine's own SQLite cache, and a
bounded daily edit above it. Choose an example week, advance virtual time, or
select My time to inspect the retained local snapshot. Trace still owns task
text, status, completion, priority, context, and its exported due value;
Temporal Engine does not write Trace's database. A non-null Trace due value
remains visibly unresolved until a source contract preserves its precision and
timezone.

Use Node **24.x** (verified with 24.15.0/npm 11.12.1), the Rust toolchain below,
and the Windows WebView2 runtime. Dependencies are pinned in `package-lock.json`
and `Cargo.lock`. From the repository root, install the locked frontend packages
once, then launch the development app:

```powershell
npm ci
npm run app
```

For a local executable with the frontend bundled, without a development server:

```powershell
npm run build
cargo build -p temporal-app --bin temporal-app --features custom-protocol --locked
.\target\debug\temporal-app.exe
```

This is a local debug prototype, not an installer. Run it in your normal Windows
session so WebView2 can create its app-specific profile. The twelve synthetic
examples include all ten required Horizon cases plus source freshness and a
flexible Routine. Time controls change only the selected synthetic snapshot.

`npm run preview` provides the same synthetic app boundary at
`http://127.0.0.1:1420` for browser verification. Its development-only bridge
executes the fixed `scenario-preview` binary and uses a synthetic cache under
`.cache/browser-preview`; it is absent from the bundled app. The preview clock
is frozen so imports and screenshots are repeatable. Stop that server before
starting `npm run app`, which uses the same port.

To import personal Trace data, use Trace's `Ctrl+K` → `Export as JSON`, then
choose `Import Trace JSON` in My time. The export is read as bytes and copied
only into Temporal Engine's own app-data cache. The first adapter has a 10 MiB
input limit and a 24-hour source-freshness limit. Failed or stale imports leave
the previous snapshot visible and expose the source health result. Re-export
after changing tasks in Trace; this first boundary is intentionally a snapshot,
not a live connection.

Open **Manage local time** in My time to add and edit fixed Anchors,
real Deadlines, soft Intentions, weekly Routines, declared Availability, and work
annotations through the retained Trace task picker. Removed Trace references stay
labelled and available for managing their local work links. Local edits stay in Temporal Engine's own
SQLite cache, retain stable IDs and revisions across restart, and never write back to
Trace. Use an IANA timezone and the date-only controls when the precision is civil-day
based; leaving effort blank preserves an unknown estimate. Completion and routine
pause/outcome actions are explicit user state, so passage alone does not close or
overdue a flexible record.

**Today** presents the finite daily edit documented in `docs/TODAY-POLICY.md`:
Fixed facts that fall today, at most three Worth doing candidates, at most two
Loose ones, and at most three awareness rows On the radar. A flexible row shows
an advisory range inside declared availability that fits before civil midnight;
it reserves nothing, records nothing, and creates no deadline. Selecting a row
opens the same inspector the Horizon uses, and **Why these, and what they are
not** lists the ordering rationale, the conditional source basis, the expiry,
and the typed evidence. An empty group is a real answer: an empty calendar does
not establish availability, so unknown compatibility never fills a quota. Local
time re-evaluates each minute while visible and on focus; the preview stays
frozen.

The first local editor accepts exact estimates or unknown effort and bounded
weekly rules. Ambiguous or nonexistent daylight-saving wall times are rejected;
it has no offset-choice control yet. The display zone is currently America/Toronto,
while each dated record retains its explicit zone.

`temporal_core::evaluate(&input)` validates a normalized snapshot and returns
the complete deterministic `EvaluationOutput`, using the input's explicit time.
`codec::decode_input` accepts strict synthetic JSON and `codec::canonical_bytes`
encodes stable comparison bytes. `tests/scenarios.rs` runs the 81 documented
inputs through this boundary with explicit semantic expectations.

Use Rust **1.98.0**, pinned in `rust-toolchain.toml`, with rustfmt and clippy. On Windows, install the MSVC C++ build tools and Windows SDK (Visual Studio's Desktop development with C++ workload). The initial setup was checked with Visual Studio 2022 Community's MSVC toolchain. Rust dependencies are locked in `Cargo.lock`.

Run the frontend build before Rust's all-feature checks so bundled assets exist:

```text
npm test
npm run build
cargo metadata --no-deps --format-version 1
cargo check --workspace --all-targets --locked
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
```

Keep private local inputs under ignored `local-private/`; repository fixtures must be synthetic. The [local wayfinder map](.scratch/temporal-engine/map.md) tracks remaining decisions. `ASTRA.md` and the gate evidence remain the implementation handoff. Do not place personal exports in the repository.

Time calculations use pinned [Chrono 0.4.45](https://docs.rs/chrono/0.4.45/chrono/) with only its `std` feature and [Chrono-TZ 0.10.4](https://docs.rs/chrono-tz/0.10.4/chrono_tz/) with bundled IANA **2025b** rules. The core does not enable Chrono's system-clock or machine-local-zone features. Updating these pins is an explicit dependency change requiring the civil-time tests to pass. After initial dependency acquisition, the checks also run with Cargo's `--offline` flag.
