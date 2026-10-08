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

## Using Temporal (0.2, calendar-replacement release)

Gate 4 (D-014) turns the prototype into an app you can live in. Install it
with the per-user Windows installer (`Temporal Engine_0.2.0_x64-setup.exe`);
no administrator rights are needed, and WebView2 is fetched only if Windows
lacks it. Everything is stored on this computer in
`%APPDATA%\local.temporal.engine\temporal-engine.sqlite3`.

First run:

1. **Settings → Usual availability.** Declare the hours you are usually
   willing to work. Temporal only suggests work inside declared time; an
   empty calendar is never treated as free time.
2. **Sources → Add a calendar.** Subscribe with a link (Google's "secret
   address in iCal format", Outlook's published ICS link, or Quercus's
   Calendar Feed) or import an `.ics` file. Choose *Coursework* for Quercus so
   assignments become real deadlines. Links are kept in Windows Credential
   Manager; subscriptions refresh at start-up and every three hours.
3. **Sources → Trace tasks.** Choose your Trace JSON export once; Temporal
   re-reads it whenever the file changes.

Daily use:

- **Horizon** (home) answers what is fixed, what fits before it, what is
  getting risky, and what is only a suggestion. Select anything to see why.
- **Calendar** shows Day, Week, and Month. Drag on the grid to create an
  event; imported events are read-only and keep their source colour.
- **Quick add** (press `/`): `Lab Tue 2-4pm every week until Dec 5`,
  `PS3 due Fri 11:59pm`, `maybe call grandma sunday`. The interpretation is
  shown before anything is saved; `Shift+Enter` opens the full editor.
- Repeating events can be edited or skipped one occurrence at a time.
- Past assignments from a coursework feed count as overdue until you mark
  them handled (D-015); the source itself is never changed.
- Reminders appear before fixed events (Settings). Closing the window keeps
  Temporal in the notification area so reminders continue; quit from its
  tray menu.

Keyboard: `/` quick add · `N` new event · `H` Horizon · `C` Calendar ·
`3` Sources · `4` Settings · in Calendar `T` today, `←`/`→` move, `D`/`W`/`M`
change view · `Ctrl+Enter` saves in the editor · `Esc` closes.

## Development

Gates 0–2 are complete; Gate 4 is the user-directed calendar-replacement
release (see `docs/BUILD-GATES.md`). Trace still owns task text, status,
completion, priority, context, and its exported due value; Temporal Engine
never writes Trace's database or any calendar it reads.

Use Node **24.x** and the Rust toolchain pinned in `rust-toolchain.toml`
(1.98.0, rustfmt, clippy). Dependencies are locked in `package-lock.json` and
`Cargo.lock`.

```powershell
npm ci
npm run app            # development app with hot reload
npx tauri build        # release executable and per-user NSIS installer
```

The installer lands in `target/release/bundle/nsis/`. On Windows this needs
the MSVC C++ build tools and Windows SDK. The GitHub Actions workflow in
`.github/workflows/build.yml` runs every check on Linux and builds the
installer on `windows-latest`, uploading it as a workflow artifact.

Cross-building from Linux (used when no Windows host is available) needs
MinGW-w64 and NSIS (`gcc-mingw-w64-x86-64`, `nsis`) and the
`x86_64-pc-windows-gnu` Rust target:

```bash
npx tauri build --target x86_64-pc-windows-gnu --bundles nsis
```

`npm run preview` serves the same Rust command surface at
`http://127.0.0.1:1420` for browser verification. Its development-only bridge
runs the `scenario-preview` binary against a synthetic cache in
`.cache/browser-preview`, with time frozen (override with `PREVIEW_NOW`). It is
absent from the bundled app. The twelve synthetic example weeks remain under
**Settings → Explore example weeks**.

Checks (run the frontend build before Rust's all-feature checks so bundled
assets exist):

```text
npm test
npm run build
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-features --locked
```

`temporal_core::evaluate(&input)` validates a normalized snapshot and returns
the complete deterministic `EvaluationOutput`, using the input's explicit
time. The app boundary normalizes everything it feeds the core: local series
and usual availability expand into ordinary Anchors and declarations, and
iCalendar recurrence expands into bounded occurrences keyed by the source's
own identity. Time calculations use pinned Chrono 0.4.45 and Chrono-TZ 0.10.4
(IANA 2025b); RRULE expansion uses rrule 0.14.0.

Keep private local inputs under ignored `local-private/`; repository fixtures
must be synthetic. Never commit calendar links, exports, or personal data.
