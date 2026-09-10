# Build Gates

Astra advances this project through explicit gates.

A gate is complete only with evidence.

## Gate 0 — Constitution and implementation contract

**Goal:** make the repository specific enough that implementation does not invent the product.

### 0.1 Review initial constitution

Read every foundation document.

Identify:

- contradiction
- undefined term
- duplicated authority
- implementation-blocking ambiguity
- scope leak
- missing invariant

Make only surgical documentation corrections consistent with settled decisions.

**Status: COMPLETE.** All 12 imported files were inspected; the eight original `docs/` files were read in README order. The untouched import is preserved in baseline commit `3333945`.

Audit evidence:

- Clarified document authority and that source rollout order is not field precedence.
- Separated data-flow arrows from code dependencies; preserved a small internal core.
- Clarified Milestone as an Anchor/Deadline significance overlay, explicit completion evidence, conservative source reconciliation, and unknown opportunity/effort.
- Identified the unverified Trace transport/schema boundary and separated source task states from deadlines.
- Made the mobile exclusion consistent with the current instruction. No settled product direction was changed.
- Deferred exact identifiers, certainty axes, time boundaries, linked deadline satisfaction, and pressure shapes to Task 0.2, where they can be specified together. Existing O-001 through O-007 remain open; none blocks this audit.

Verification: reviewed the full Task 0.1 diff against D-001 through D-013 and the user invariants; reconciled an independent read-only specification audit. No code, dependencies, integrations, or personal fixtures were added.

### 0.2 Freeze initial normalized domain contract

Produce an implementation-level schema/spec for:

- Anchor
- Deadline
- Task reference
- Intention
- Routine
- Window
- Milestone
- Suggestion
- source metadata
- certainty
- rigidity
- temporal timestamps/timezones
- stable identifiers

Do not write product code yet.

**Status: COMPLETE.** `DOMAIN-CONTRACT.md` version 1 defines the separate species, typed identity/provenance, field ownership, civil/absolute time, lifecycle/fulfillment, bounded recurrence, source-health qualification, explicit availability, deterministic fit/pressure/reasons, suggestion validity, and strict synthetic serialization/validation. README and the file inventory point to it.

Verification: checked every Task 0.2 requirement against the contract and reconciled two rounds of independent semantic review, including zero-time precedence, Trace due-projection coverage, stale completion, complete fit inventories, and expired/invalidated suggestions. `git diff --cached --check` passed before commit. proof-v1 assesses individual capacity only; visual compression, editorial ranking, real transports, and calibration remain later work.

### 0.3 Define virtual-time scenario suite

Specify synthetic scenarios and expected semantic outcomes.

At minimum cover the ten Horizon cases in `HORIZON.md`.

**Status: COMPLETE.** `SCENARIOS.md` defines S01–S10 in the required Horizon order and S11–S16 for civil-time boundaries, source health/coverage, completion ownership, routine rollover, unknown compatibility, deterministic output, and invalid input. Each case has frozen time, timezone, synthetic identity/source/work defaults, numerical/semantic expectations, recommendation constraints, Horizon meaning, and forbidden conclusions.

Verification: independently reviewed every base case and variant against the domain contract; reconciled snapshot timing, work-owner, recurrence, range, and suggestion-basis findings. Checked minute/ratio arithmetic and Toronto DST resolutions with local PowerShell/.NET calculations (23-hour spring day, 25-hour fall day, gap/fold). Confirmed all ten required cases and all 16 scenario IDs are present. These are documentation checks; Rust tests and visual verification have not run.

### 0.4 Produce Gate 1 task plan

Break Gate 1 into numbered, commit-sized tasks with:

- dependencies
- files/modules expected
- acceptance criteria
- focused verification
- gate-wide verification

**Status: COMPLETE.** Gate 1 below contains ten numbered tasks, each with purpose, dependencies, expected files, acceptance criteria, focused verification, and prohibited adjacent work. A single internal Rust crate proves the contract before application scaffolding or integrations. The final task specifies full checks and an evidence-based handoff.

Verification: reviewed task dependency order and scenario coverage; reconciled independent review by placing fulfillment resolution and checked expiry preflight before graph/source validation needs them. Mechanically checked all six required task fields across 1.1–1.10. The plan was checked against the authorized exclusions; no planned files were created and no toolchain/dependencies were installed.

### Gate 0 completion criteria

- documentation set is internally consistent
- no unresolved implementation-blocking semantic ambiguity for the authorized Gate 1 scope; explicitly deferred later-gate questions may remain open
- initial domain contract is precise enough to test
- scenario suite is defined
- Gate 1 implementation plan is executable
- no product source code was added unless separately authorized

### Gate 0 evidence report — 2026-09-08

**Completed scope:** Tasks 0.1–0.4 only: audited/ratified the constitution, defined the normalized domain contract, specified deterministic synthetic scenarios, and planned the first executable core proof. Local Git was initialized and the untouched import preserved in `3333945`; task commits are `d2ec978` (0.1), `a31d21a` (0.2), `e8bb201` (0.3), and `74e144c` (0.4). The final evidence/handoff is a separate `docs: complete gate 0` commit.

**Documents:** created `DOMAIN-CONTRACT.md` and `SCENARIOS.md`. Updated root README, AGENTS, ASTRA, and FILE-STRUCTURE; updated `ARCHITECTURE.md`, `BUILD-GATES.md`, `HORIZON.md`, `PRODUCT.md`, `TEMPORAL-MODEL.md`, and `TRACE-CONTRACT.md`. `DECISIONS.md` and `PRIVACY.md` remain unchanged from the baseline.

**Verification performed:**

- Read the complete foundation in README order and reviewed the documents as one system against D-001–D-013 and the user's invariants. Reconciled independent read-only constitution, contract, scenario, plan, and fresh cross-document audits. Reviewed the full change from baseline `3333945`, including the new documents and final historical-Suggestion clarification.
- `git diff --check 4bbc312` and per-task staged whitespace checks passed. Git history/author/body review confirms the untouched baseline, separate task commits, configured user identity, and no attribution trailers. No remote is configured.
- Read-order/file-inventory checks found 14 repository documents, all 10 `docs/` files reachable in README's read order, no missing targets, and no non-document product files. S01–S16 are present, including all ten required Horizon cases. All ten Gate 1 tasks have the six required planning fields.
- Independently checked opportunity/ratio arithmetic, exact threshold/equality cases, and the bounded recurrence dates. Local PowerShell/.NET timezone checks confirmed Toronto's 2026 spring gap/fall fold and 23/25-hour civil days. Synthetic identifiers, timestamps, source assumptions, and variant transitions were manually reviewed for determinism.
- No production code, dependency installation, application scaffold, database, adapter, UI, real personal fixture, credential, or remote publication was introduced. Rust/type/build tests and visual/manual application verification were **not run** because no executable application/core exists yet.

**Resolved ambiguities:**

- Document authority and source rollout versus overwrite precedence; data flow versus code dependency direction.
- Provenance versus confirmation/tentativeness; Milestone significance versus duplicate objects; explicit completion/satisfaction versus elapsed time or zero effort.
- Stable Trace/source identities, due projection ownership/coverage, independent deadline work links, and conservative last-known-source handling.
- Exact versus whole-date cutoffs, DST/zone resolution, bounded soft recurrence, and injected snapshot/evaluation time.
- Raw Windows versus compatible opportunity, missing versus zero capacity, lower-bound effort, individual pressure limitations, and deterministic explanations/serialization.
- Creation-time Suggestion eligibility versus retained historical records: a completed/removed target does not invalidate the fixture itself; basis/expiry/current eligibility determine the validity result, with no overdue obligation or factual mutation.
- Gate 1 validation prerequisites and staged scenario evidence, so later algorithms are not claimed complete by an earlier parser test.

**Remaining non-blocking questions:** O-001 name, O-002 exact Horizon compression, O-003 visual grammar, O-004 actual supported Trace transport/schema, O-005 effort calibration, O-006 travel provider, and O-007 local AI. They remain recorded in `DECISIONS.md`; Gate 1 requires none of them to be settled. Exact installed Rust/dependency/timezone-rule versions will be inspected and pinned during the responsible Gate 1 tasks.

**Deviations and residual risks:** no product-direction deviation or scope expansion. Gate 0 proves documentation coherence, not engine behavior or visual usefulness. proof-v1 is an explicit initial policy, uses declared/possibly incomplete opportunity, and assesses individual work without allocating shared capacity. Its practical calibration, actual source normalization, and usefulness with personal-local data remain unproven and belong to later evidence. Implementation may reveal defects; resolve them against this authority order rather than weakening invariants.

**Handoff:** Gate 1 is planned and unstarted. The exact next task is **1.1 Establish the minimal Rust core workspace**. This run stops here.

GATE 0 COMPLETE

---

## Gate 1 — Temporal core proof

**Goal:** prove the domain model, virtual clock, deterministic pressure skeleton, and scenario harness before building the real UI.

**Status: COMPLETE — 2026-09-08.** The subsequent user instruction authorizes building the application. Tasks 1.1–1.10 prove `DOMAIN-CONTRACT.md` version 1 and `SCENARIOS.md` S01–S16. The evidence report below distinguishes this synthetic core proof from the application work in Gate 2.

Gate 1 builds one internal Rust library crate, `crates/temporal-core`, plus local synthetic tests. The eventual desktop stack is unchanged, but no Tauri/Svelte/TypeScript application, SQLite/persistence crate, source adapter, UI/renderer, compression formula, network service, background process, AI, travel, companion surfaces, mobile, FI99 package/SDK, packaging, public distribution, or remote publication belongs in this gate. Source kinds and health are normalized synthetic data only. No real Trace database/calendar/token is a test dependency.

Use the task order below, one task per commit. Record actual files/check results in each task's evidence and update ASTRA to the next incomplete task only after its acceptance criteria pass. Future file/module paths below are expected organization, not files that already exist. A small private helper may be colocated differently without changing semantics; document meaningful deviations. Local focused commands use the test/module names established by the corresponding task, with the gate-wide commands fixed below.

### 1.1 Establish the minimal Rust core workspace

**Status: COMPLETE — 2026-09-08.** Added a single unpublished `temporal-core` library, workspace/lockfile, Rust 1.98.0 toolchain pin, line-ending/build/private-input rules, and README commands. The requested wayfinder map and its decision prerequisites are kept as local Markdown outside this repository; they retain this gate plan's authority.

**Evidence:** `cargo metadata --no-deps --format-version 1` reports exactly one workspace member and no dependencies; `cargo check --workspace --locked`, `cargo fmt --all -- --check`, and `git diff --check` passed. Compiler: rustc 1.98.0 (`88d9e12ae`, 2026-08-18), cargo 1.98.0 (`797e8a9bc`, 2026-08-05), x86_64-pc-windows-msvc; rustfmt/clippy installed for the pinned release. Visual Studio 2022 Community with the MSVC C++ component is installed. Linking is verified by the first substantive test binaries in Task 1.2, not claimed by this library check. No temporal behavior or application shell exists yet.

- **Purpose:** make a repeatable local build/check entry point for the core proof, independent of the eventual desktop shell.
- **Dependencies:** completed Gate 0 and its committed domain/scenario contract; inspect toolchain availability before choosing versions.
- **Expected files:** root `Cargo.toml`, `Cargo.lock`, `rust-toolchain.toml`, `.gitignore`, `.gitattributes`; `crates/temporal-core/Cargo.toml`, `crates/temporal-core/src/lib.rs`; README development commands.
- **Acceptance:** one workspace member and an internal library with `publish=false`; explicitly pinned Rust release and committed lockfile. Ignore build products/local private inputs without hiding documentation/fixtures. Declare only dependencies needed for the following approved core tasks, chosen and pinned against the actual toolchain; no speculative crate tree. Record the exact rustc/cargo versions and local Windows build prerequisites that were verified. An empty business module at this scaffold step is acceptable and is not engine evidence.
- **Focused verification:** `cargo metadata --no-deps --format-version 1`, `cargo check --workspace --locked`, and `cargo fmt --all -- --check`; inspect workspace membership and dependency directions. No artificial arithmetic test merely to manufacture a green test count.
- **Prohibited adjacent work:** application shell, WebView setup, frontend package manager, database schema, CI hosting, installer, remote repository, extra public libraries, or implementation of temporal behavior in the scaffold commit.

### 1.2 Implement explicit time primitives and the injected clock

**Status: COMPLETE — 2026-09-08.** Added validated UTC millisecond/civil date/IANA zone types, positive half-open spans, date-span resolution, exact/date-only cutoff boundaries, explicit offset/fold resolution, checked arithmetic, `Clock`/`FrozenClock`, and a once-captured evaluation interval. Direct time dependencies are pinned to Chrono 0.4.45 (std only) and Chrono-TZ 0.10.4, bundled IANA 2025b. No OS timezone or system-time feature is enabled.

**Evidence:** the focused `time_contract` target first failed for missing time/clock modules, then all 12 substantive tests passed. Tests cover S11 Toronto 23/25-hour days, gaps/folds and offset disagreement, nonexistent/repeated civil midnights, exact/date-only equality, display-zone invariance, normalized precision/year bounds, span intersection, overflow, snapshot bounds, failed advance atomicity, and one clock read. Full workspace check, clippy with `-D warnings`, all tests, format, and diff whitespace checks passed; Cargo checks/tests ran locked and offline after dependency acquisition. Test executables linked and ran with the installed Windows MSVC toolchain. No lifecycle/source/opportunity behavior is claimed yet.

- **Purpose:** make every later rule operate on a captured instant and explicit civil-time rules.
- **Dependencies:** 1.1; Domain Contract Sections 2–3 and S11/S16 time cases.
- **Expected files:** `crates/temporal-core/src/clock.rs`, `time.rs`, exports from `lib.rs`; focused `tests/time_contract.rs`; manifest/lockfile changes only for required time support.
- **Acceptance:** UTC millisecond instants, checked duration arithmetic, TimedSpan/DateSpan/cutoff primitives, explicit IANA rules and recorded rules version. One injected clock read at the evaluation boundary; frozen/advanceable test clock. Resolve only explicit zone/offset choices, enforce half-open intervals and civil-date endpoints, and reject ambiguous/nonexistent/unrepresentable input. Distinguish a snapshot's captured time from later evaluation time. Host zone and wall-clock reads cannot affect pure logic.
- **Focused verification:** exact cutoff equality versus +1ms, inclusive due-date next-boundary behavior, touching/overlapping spans, cross-midnight spans, S11's 23/25-hour days and gap/fold resolutions, display-zone invariance, backwards-before-snapshot rejection, and checked overflow. Search the new core code for uncontrolled system-clock/local-zone reads. Run focused tests plus workspace check/format.
- **Prohibited adjacent work:** compression coordinates, animation timers, recurring import parsing, reminders, OS background scheduling, or implicit local-time defaults.

### 1.3 Implement distinct domain types and field ownership

**Status: COMPLETE — 2026-09-08.** Added typed canonical UUIDv4 IDs, narrow common audit metadata, separate stored species, local/imported/Trace-only provenance, keyed annotations/outcomes, finite source states/roles, work/context/energy types, and distinct evaluation/result/suggestion types. Structured reason variants fix each code's payload at compile time. Revisions/chunks/lower bounds use nonzero integer types where zero is invalid. No evaluator-side allocation, transport, persistence, or ranking was introduced.

**Evidence:** the focused domain target first failed for missing modules, then all 8 domain tests passed. They cover identity/projection separation, opaque source keys, current/stale confirmation without fact mutation, unknown versus zero work/context, Trace due versus independent fulfillment, soft state/occurrence identity, non-clock evaluation basis, and typed derived evidence. Three compile-fail doc tests verify species-ID separation, no Suggestion completion field, and no insertion of inference into factual deadlines. All 20 integration tests and 3 doc tests pass; locked/offline workspace check, clippy with `-D warnings`, format, and diff whitespace checks pass. Cross-record validation, serialization, lifecycle, and pressure remain assigned to subsequent tasks.

- **Purpose:** encode the species/provenance distinctions so later code cannot flatten them accidentally.
- **Dependencies:** 1.2; Domain Contract Sections 1–6 and typed result definitions in Sections 7–10.
- **Expected files:** `crates/temporal-core/src/domain/` (`ids.rs`, `objects.rs`, `work.rs`, `source.rs`, module exports), `results.rs`, `reasons.rs`; `tests/domain_types.rs`.
- **Acceptance:** typed fixed UUID identities; source-instance/import keys; separate Anchor, Deadline, Trace Task Reference, Intention, Routine and keyed annotations/outcomes. Define Window/Suggestion/pressure/reason result types separately from stored facts. Encode DateSpan rather than an all-day boolean; explicit recorded versus Trace-task fulfillment; standalone versus single linked effort; certainty, confirmation revision, occupancy, significance, unknown/at_least effort, context and energy. Trace due projections and Milestone overlays cannot duplicate work or become a second generic event. No evaluator-side ID allocation or source writer exists.
- **Focused verification:** constructors and representative synthetic values for each species; current/stale confirmation representation, source task state preservation, no completion field on Suggestion, no standalone Milestone workload, and explicit unknown variants. Use meaningful type/ownership/shape tests; lifecycle calculations belong to 1.5. Run focused tests and broader workspace check/format.
- **Prohibited adjacent work:** generic event/property-bag schema, generic source SDK, database models, actual Trace schema assumptions, task editing/writeback, source deduplication heuristics, or ranking behavior.

### 1.4 Add strict serialization, validation, and the synthetic fixture harness

**Status: COMPLETE — 2026-09-08.** Added strict normalized JSON decoding with duplicate-key detection, schema/version and unknown-field rejection, canonical encoding, whole-snapshot validation, checked source-expiry preflight, and the clock-independent authoritative fulfillment resolver. Added explicit synthetic S01–S16 bases and named mutations with stable IDs plus a coverage manifest. Validation rejects invalid ownership, references, identities, links, timestamps, intervals, source states, recurrence boundaries, and retained suggestion payloads before any later evaluator stage can consume partial input.

**Evidence:** `fixture_contract` validates and canonical-round-trips 81 valid inputs (16 bases plus 65 named mutations), including provenance and unknown-value preservation, reordered-collection byte invariance, Unicode/escaping, and ownership mutations. `validation_contract` rejects 51 independent fixtures with deterministic `ValidationIssue` categories, including duplicate JSON keys, unsupported schema versions, S16 source-expiry overflow, duplicate identities, invalid links, timestamp/interval/source inconsistencies, and suggestion/fact boundary violations. The fulfillment tests prove valid Done-linked S13 work resolves to satisfied without satisfying a separate Deadline, while S16's competing unresolved owners fail validation. `cargo fmt --all`, `cargo check --workspace --all-targets --locked --offline`, `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`, `cargo test --workspace --locked --offline`, format-check, and `git diff --check` all pass. Coverage and stage boundaries are recorded in `crates/temporal-core/tests/fixtures/README.md`; source health, opportunity, pressure, complete evaluation, and suggestion revalidation remain deferred to Tasks 1.6–1.9.

- **Purpose:** make the contract executable as inspectable inputs and reject invalid states before evaluation.
- **Dependencies:** 1.3; Domain Contract Section 10; SCENARIOS fixture conventions and S16 rejection matrix.
- **Expected files:** `crates/temporal-core/src/validation.rs`, `codec.rs`, `fulfillment.rs`; `crates/temporal-core/tests/support/`, `tests/fixtures/`, `tests/validation_contract.rs`, `tests/fixture_contract.rs`; a concise fixture coverage manifest in `tests/fixtures/README.md`.
- **Acceptance:** strict schema/version/field/tag parsing including duplicate JSON-key detection, ownership/reference/identity/link validation, timestamp/interval/source consistency, and deterministic issue ordering. Include the minimal clock-independent authoritative fulfillment resolver needed to count unresolved/unknown work owners correctly (valid Done-linked S13 versus invalid duplicate-owner S16); 1.5 must reuse it. Preflight source-expiry addition with 1.2's checked arithmetic so the S16 overflow fixture fails validation before health evaluation. Canonical JSON honors the contract's integer, escaping, optional-field, set, and key rules. Materialize every S01–S16 base and named variant as explicit synthetic fixtures or named deterministic fixture mutations, with stable IDs and frozen settings. The harness can select a scenario/stage and report expected validation issues. Record which semantic stages are implemented; do not mark the whole scenario suite passed yet.
- **Focused verification:** all valid snapshots parse/validate/round-trip without losing provenance or unknown values; S16 invalid variants fail as specified without partial output; canonical bytes agree across reordered collections; each scenario/variant ID is accounted for in the coverage manifest. Prior Suggestions remain a separate inferred collection. Run focused tests, all existing workspace tests, check/format/clippy.
- **Prohibited adjacent work:** real source parsing, ICS/RRULE support, Trace transport, SQLite migrations, new fixture semantics beyond the contract, expected values generated from the implementation being tested, or skipped tests that pretend unimplemented engine stages are complete.

### 1.5 Implement temporal lifecycle and bounded routine occurrences

**Status: COMPLETE — 2026-09-08.** Added pure lifecycle derivation for Anchor, Deadline, and Intention state rows, with exact timed/date-only cutoff boundaries and the Task 1.4 fulfillment resolver. Added bounded weekly Routine expansion in the rule timezone, stable `(RoutineId, LocalDate)` occurrence keys, explicit outcome and pause/rule-edit handling, and a historical occurrence helper that reports `past_unrecorded` without creating debt or mutating the snapshot. Candidate filtering remains separate from lifecycle state.

**Evidence:** `lifecycle_contract` has 5 tests covering half-open Anchor phases, removed Anchors, overdue/satisfied/unknown Deadlines, exact equality and on-date boundaries, Done-linked Trace work versus independent Deadline fulfillment, and active soft Intention passage. `recurrence_contract` has 4 tests covering finite S14 expansion through the intersecting end date, stable ordering and timezone, historical inspection, skipped/paused/edited rules, no debt, and distinct same-date routines. The complete locked/offline workspace check, clippy with warnings denied, all tests/doctests (38 substantive integration tests plus 3 compile-fail doctests), format check, and diff check pass. No attendance inference, source-health, opportunity, pressure, adapter, persistence, or UI behavior is claimed.

- **Purpose:** establish fixed/soft passage, explicit completion, and stable recurrence semantics before capacity calculation.
- **Dependencies:** 1.4; Domain Contract Section 5; S01–S03/S07–S11/S13–S14 lifecycle cases.
- **Expected files:** `crates/temporal-core/src/lifecycle.rs`, `recurrence.rs`; `tests/lifecycle_contract.rs`, `tests/recurrence_contract.rs`; scenario-stage assertions/coverage updates.
- **Acceptance:** Anchor upcoming/ongoing/passed/inactive; complete Deadline phase/resolution table including unknown fulfillment and exact/date-only boundaries, reusing 1.4's fulfillment resolver. Done-linked independent work can have derived zero effort without satisfying its Deadline. Intention preference passage preserves active user state. Weekly Routine expansion is bounded, uses stable date keys, respects pause/outcomes/rule edits, and never accumulates debt. Helpers can inspect historical occurrence state without restoring it as a candidate.
- **Focused verification:** S09 real overdue versus S10 underlying flexible work, S11 temporal boundaries, both valid one-link S13 unknown-status fixtures, authoritative removal/completion, and all S14 current/future/past/paused/edited-rule/outcome cases. Advance only the injected clock and assert stored input equality. Run focused and existing broader checks.
- **Prohibited adjacent work:** attendance inference, effort learning, Trace completion commands, automatic intention-to-Deadline conversion, general recurrence engines, or UI rollover behavior.

### 1.6 Derive source health and dependency coverage

**Status: COMPLETE — 2026-09-08.** Added deterministic source-health derivation with the contract’s incompatible/unavailable/partial/never-loaded/stale/healthy precedence and exact freshness equality. Added owned dependency/coverage requirements and qualifications that preserve health separately from query coverage, retain last-known records, qualify local input without fabricated refresh timestamps, and automatically include Deadline, linked Task, and blocking Anchor source dependencies. Trace due projections require Task catalog coverage rather than a separate Deadline interval; overdue/inactive assessments can retain source health without requiring future Anchor coverage.

**Evidence:** `source_health_contract` has 4 tests covering all S12 health variants, empty successful catalogs, freshness equality and +1ms staleness, checked expiry overflow, half-open coverage, omitted Trace requirements, independent linked Deadline coverage, and Trace due Task catalog inheritance. The complete locked/offline workspace check, clippy with warnings denied, all tests/doctests (42 substantive integration tests plus 3 compile-fail doctests), format check, and diff check pass. No adapter refresh, deletion reconciliation, opportunity arithmetic, pressure, persistence, or UI behavior is claimed.

- **Purpose:** prevent missing or stale sources from producing unqualified conclusions.
- **Dependencies:** 1.5; Domain Contract Section 6; S06/S08/S12/S13.
- **Expected files:** `crates/temporal-core/src/source_health.rs`; `tests/source_health_contract.rs`; harness stage/coverage updates.
- **Acceptance:** deterministic never_loaded/healthy/stale/partial/unavailable/incompatible precedence; exact freshness equality; checked expiry arithmetic. Calculate coverage per role and automatically include factual dependencies even when omitted from explicit requirements. Trace due projections inherit complete Task catalog coverage; unrelated linked deadlines retain their own source coverage. Empty required sources remain visible, last-known records remain inputs, and overdue/inactive assessments need no backwards future interval. Source health qualifies facts without rewriting status/provenance.
- **Focused verification:** all S12 variants including 09:30 equality/+1ms health, S13 stale Done/zero work despite omitted explicit T requirements, healthy task_due without separate Deadline coverage, and wrong role/interval coverage. S16 expiry overflow remains covered by 1.4's preflight validation. Separate health tests from opportunity arithmetic until 1.7. Run focused and broader existing checks.
- **Prohibited adjacent work:** adapter reconciliation transport, actual refresh jobs, database queries, credentials, OAuth, source discovery, or automatic deletion/completion on failure.

### 1.7 Derive Windows and the complete work-fit matrix

**Status: COMPLETE — 2026-09-08.** Added deterministic declared-opportunity
derivation and work-fit evaluation. Availability declarations are clipped to
the injected evaluation range; present Busy/Unknown Anchors are unioned and
subtracted once, transparent/removed Anchors remain non-blocking, and positive
Anchor conflicts are reported independently. Windows retain declaration
context/energy and source-health/coverage qualifications with stable keys.
Added the complete eligible target × Window matrix for Tasks, standalone
unresolved Deadlines, active Intentions, and current/future Routine
occurrences, including empty clipping rows, routine-date bounds, earliest
starts, context/energy compatibility, minimum chunks, definite mismatch versus
unknown, and mathematical zero precedence. Added deterministic individual
opportunity aggregation with known qualifying subtotals; shared capacity is
never allocated.

**Evidence:** `opportunity_contract` has 4 tests covering S02 raw opportunity,
S05 union/conflict/transparency, S11 civil-day blocking/removal, S12 stale
source qualification, and input-order invariance. `fit_contract` has 6 tests
covering S02 short fragments and single-work ownership, S03/S04 endpoint and
tie clipping, S05/S06/S15 compatibility and chunk boundaries, S07 soft
intentions, S09 overdue empty clipping, S12 +1ms chunk loss, S14 occurrence
dates, and known/unknown opportunity aggregation. The complete locked/offline
workspace check, clippy with warnings denied, all tests/doctests (52
substantive integration tests plus 3 compile-fail doctests), format check, and
diff check pass. No pressure, evaluation assembly, adapters, persistence, or
UI behavior is claimed.

- **Purpose:** prove primitive usable opportunity from declared willingness, fixed blockers, and explicit compatibility.
- **Dependencies:** 1.6; Domain Contract Section 7; S01–S08/S11–S16.
- **Expected files:** `crates/temporal-core/src/opportunity.rs`, `fit.rs`; `tests/opportunity_contract.rs`, `tests/fit_contract.rs`; fixture-stage assertions/coverage.
- **Acceptance:** clip declarations to evaluation range, union blocking Anchors once, preserve transparent/tentative/unknown occupancy semantics, and expose conflicts separately. Derive stable Window keys and every eligible target × Window fit row, including empty clipping. Match context/energy and useful chunk duration explicitly; handle earliest bounds, routine dates, single-work targets, and out-of-range deadlines. Known-zero temporal impossibility precedes missing-input uncertainty. Unknown fit capacity retains its known subtotal; shared Windows are never reservations.
- **Focused verification:** S02 raw versus usable opportunity, S05 union/conflict/transparency, S06 fragments/zero, S11 civil-day durations, S12 chunk loss after +1ms, S14 complete occurrence/Window pairing, and S15 definite mismatch versus unknown and mathematical zero. Check interval containment, disjoint Windows, capacity conservation, and input-order invariance. Run focused and broader existing checks.
- **Prohibited adjacent work:** guessed free time, working-hours defaults, inferred travel/location/energy, calendar auto-blocking, shared-capacity allocation, task scheduling, or visual Window layout.

### 1.8 Implement proof-v1 pressure with structured reasons

**Status: COMPLETE — 2026-09-08.** Added deterministic proof-v1 pressure
derivation over the Task 1.7 Windows and fit matrix. Each present Deadline is
ordered by calculation endpoint and canonical ID; fulfillment and overdue/range
short-circuits precede work and opportunity arithmetic. Known estimates and
lower bounds use exact unreduced integer E/O pairs with room/tight/insufficient
thresholds, while zero work, unknown work/opportunity, mathematical zero, and
overdue outcomes preserve the contract's field omissions. Source health and
coverage qualifications are retained independently, and every quantified result
explains its individual-capacity limitation and contributing fit/source reasons.
Importance and Trace priority do not affect arithmetic; no shared allocation or
aggregate verdict is emitted.

**Evidence:** `pressure_contract` has 14 tests covering S03 time escalation,
S04 endpoint ties, S06 fragmented/zero opportunity, S09 overdue/resolution,
S12 freshness equality and conditional known numbers, S13 completion/unknown/
lower-bound/removal work, S15 unknown versus mathematical zero, S16 range and
threshold behavior, and deterministic pressure over all 81 valid synthetic
inputs. The complete locked/offline workspace check, clippy with warnings
denied, all tests/doctests (66 substantive integration tests plus 3
compile-fail doctests), format check, and diff check pass. No complete
evaluation assembly, suggestion validity, adapter, persistence, or UI behavior
is claimed.

- **Purpose:** calculate the first transparent individual-work risk assessment from the contract's exact table.
- **Dependencies:** 1.7; Domain Contract Sections 8–9; all scenario pressure expectations.
- **Expected files:** `crates/temporal-core/src/pressure.rs`, reason construction in `reasons.rs`; `tests/pressure_contract.rs`; pressure-stage fixture assertions/coverage.
- **Acceptance:** one result per present Deadline, endpoint/ID order, authoritative fulfillment first, exact unreduced integer ratios, room/tight/insufficient thresholds, zero/unknown/lower-bound cases, and consistent early-return field omission. Source qualifications are computed even for unknown/overdue/resolved outcomes. Explain inputs, cutoff, blocking/fit decisions, shortages/uncertainty, and individual_capacity_only. Importance and Trace priority do not modify E/O; no aggregate success verdict is emitted.
- **Focused verification:** S03 time-driven escalation with unchanged effort, S04 tie order/shared-capacity limitation, S06 shortage and zero denominator, S09 overdue early return, S13 effective zero/unknown/lower bounds, and S16 threshold/range equality and +1ms variants. Check that less usable capacity cannot lower a fixed positive-work ratio, and that every emitted value is reproducible from referenced inputs. Run focused and broader existing checks.
- **Prohibited adjacent work:** learned/calibrated weights, a final Today ranking, pressure-zone coordinates, dependence graphs, global optimization, AI explanations, or changing contract thresholds to make tests pass.

### 1.9 Assemble deterministic evaluation and suggestion validity

**Status: COMPLETE — 2026-09-08.** Added the validated `evaluate` entry point
and deterministic suggestion revalidation. The complete output retains one
evaluation basis, ordered typed arrays, and sorted/deduplicated reasons with
resolvable input references. Old advice is checked in basis/expiry/eligibility/
current-fit-or-risk order, preserving the producing record and its historical
reasons. The evaluator reuses the existing fit matrix for opportunity and
pressure; proposed spans clip assessments without creating Windows.

**Evidence:** `scenarios` executes explicit acceptance values for all 81
documented S01–S16 inputs: Window durations, complete fit inventory/statuses,
lifecycle phases, recurrence dates/outcomes, conflicts, source health, pressure
fields/ratios/qualifications, and advice status. A second check compares repeated
and permuted inputs, typed canonical output, and unchanged input bytes. Seven
`suggestion_contract` tests cover expiry/invalidation precedence, current span
and risk revalidation, all four work species, implicit/stale/failed source
evidence, and whole-input rejection. Full locked/offline check, clippy with
warnings denied, 78 integration tests, 3 compile-fail doctests, format, and diff
checks pass, with no ignored tests.

**Integration corrections:** fully blocked nonempty declarations now yield
known zero opportunity with blocking evidence; unsigned effort is widened
before conversion to milliseconds; unknown effort cannot suppress missing
Anchor coverage; health-only Window dependencies omit incomplete Deadline
coverage payloads; reason ordering uses typed references. Three pressure
regressions exercise the first three cases. These restore the existing contract;
no threshold, scope, dependency, or settled product decision changed. Final
gate review remains Task 1.10; visual and adapter behavior remain unverified.

- **Purpose:** expose a complete pure evaluation result and prove that old advice cannot become current obligations.
- **Dependencies:** 1.8; Domain Contract Sections 3/9/10; full S01–S16 expectations.
- **Expected files:** `crates/temporal-core/src/evaluate.rs`, `suggestion.rs`; final exports; `tests/scenarios.rs`, `tests/suggestion_contract.rs`; complete fixture expected assertions/coverage manifest.
- **Acceptance:** one frozen EvaluationKey, validated immutable input, complete typed output arrays, canonical ordering/reasons, and no hidden environment reads. Check basis change, expiry equality, target eligibility, and current fit/pressure revalidation in the specified order. Retain historical Suggestion reasons while validity carries current evidence. No recommendation generator is required. Every scenario and named variant now has all Gate 1 semantic assertions active; no semantic TODO/ignored test may masquerade as a pass.
- **Focused verification:** S10 clock-only expiry versus later-snapshot invalidation and no task mutation; S16 unexpired-but-no-longer-fitting G1, recalculated G2, and a newly based room notice becoming ineligible. Run the full scenario harness twice with the same inputs, permuted collections, fixed timezone rules, and explicit virtual-time steps; compare canonical output bytes and original input bytes. Run focused and all broader checks.
- **Prohibited adjacent work:** creating a Today edit, showing a Horizon UI, persisting evaluation history, promoting advice to facts, emitting rigid schedules, or hiding unimplemented semantics behind serialization snapshots.

### 1.10 Verify and report the temporal core proof

**Status: COMPLETE — 2026-09-08.** All gate-wide checks passed after Task 1.9
commit `ef241e5`. Reviewed the Gate 1 change from `8a59569`, reconciled the
scenario/invariant coverage, and inspected the representative evaluator results
listed in the report below. The existing scenario test now prints bounded
synthetic summaries with `--nocapture` so this inspection can be repeated.
No further production-code correction was needed. Gate 2 has an executable
five-task plan; none of its files or dependencies has been introduced.

- **Purpose:** establish evidence for Gate 1 completion and an honest Gate 2 handoff.
- **Dependencies:** 1.9 and every earlier task's recorded evidence.
- **Expected files:** tests/coverage corrections only where verification exposes an existing-contract gap; `docs/BUILD-GATES.md`, `ASTRA.md`, and README commands if needed. No new product feature/module is planned in this task.
- **Acceptance:** reconcile every S01–S16 base/named variant and all contract invariants against executed tests; perform a fresh full Gate 1 diff review. Resolve failures within the existing contract before claiming completion. Record exact toolchain/timezone versions, commands, results, manual inspection, deviations, and residual risks. The four core product questions must be answerable from synthetic outputs; visual usability remains explicitly unverified until Gate 2. Update ASTRA to the first genuinely incomplete Gate 2 task only after its executable scope has been recorded; do not start Gate 2.
- **Focused verification:** all gate-wide checks below and manual inspection of representative synthetic outputs from S03, S05, S09, S10, S12, and S13 for facts versus inference, unknown data, and source-qualified evidence. Verify no personal data, credentials, runtime agent dependency, unsolicited modules, remote publishing, or integration code entered the gate. Verify the working tree/history preserve one-task commits.
- **Prohibited adjacent work:** improving visuals, expanding scope to the desktop/integrations, performing a personal-data trial, or marking the gate complete while tests are skipped/failing or semantic behavior is still undecided.

### Gate 1 verification and completion criteria

Execute from the workspace root with the committed toolchain/lockfile:

```text
cargo fmt --all -- --check
cargo check --workspace --all-targets --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
git diff --check
```

Use focused tests after their responsible task and the relevant broader checks after each change; run this full set for 1.10. No UI/type build command is required because no frontend exists. Subsequent core test runs must need no source access, wall-clock waits, credentials, or personal dataset. Dependency acquisition during initial toolchain setup is distinct from runtime/test network access.

Gate 1 is complete only when all ten task acceptances and these checks pass, every documented Gate 1 semantic scenario/variant is covered, canonical output is deterministic, the source/Trace/inference boundaries are preserved, and the scope review is clean. Evidence must distinguish synthetic algorithm proof from still-deferred visual, adapter, calibration, and personal-use validation.

### Gate 1 evidence report — 2026-09-08

- **Completed scope:** Tasks 1.1–1.10; one unpublished Rust library with explicit
  time, separate domain/ownership types, strict validation/serialization,
  lifecycle and finite weekly recurrence, source health/coverage, declared
  Windows and conflicts, work fit, proof-v1 individual pressure, structured
  explanations, and complete immutable evaluation with prior-advice validity.
- **Files:** workspace/toolchain/lockfile; `crates/temporal-core/src/` and
  `tests/` (including the fixture inventory and explicit expectations); README,
  file inventory, this gate ledger, ASTRA, and local Markdown handoff tickets.
  Tasks 1.1–1.9 are separate commits from `d408bcb` through `ef241e5`.
- **Executed checks:** `cargo fmt --all -- --check`;
  `cargo check --workspace --all-targets --locked --offline`;
  `cargo clippy --workspace --all-targets --locked --offline -- -D warnings`;
  `cargo test --workspace --locked --offline`; `git diff --check`. All passed.
  **78 integration tests and 3 compile-fail ownership doctests; zero failed or
  ignored.** The full suite evaluates 81 valid inputs (16 bases and 65 named
  variants), rejects 51 invalid fixtures, and repeats/permutates full outputs
  while checking canonical bytes and unchanged input bytes.
- **Environment:** Windows `x86_64-pc-windows-msvc`; rustc 1.98.0
  (`88d9e12ae`, 2026-08-18), cargo 1.98.0 (`797e8a9bc`, 2026-08-05);
  Chrono 0.4.45 and Chrono-TZ 0.10.4 with IANA 2025b. No dependency changed
  during final verification; checks needed no network, credentials, or clock waits.
- **Coverage reconciliation:** the fixture inventory names every S01–S16
  variant. Domain/fixture/validation tests cover contract invariants 1–4;
  time/lifecycle/recurrence tests cover 3/5/8; opportunity/fit/source-health/
  pressure tests cover 4/6/7; suggestion tests cover 2/3/8. Full-evaluation
  tests cover 9, including typed evidence references and complete result
  inventories. Dependency/source/scope review covers 10. No deferred assertion
  or generated expectation is counted as a pass.
- **Manual output inspection:** ran
  `cargo test -p temporal-core --test scenarios every_documented_scenario --locked --offline -- --nocapture`.
  S03 reports 8h work / 20h declared opportunity (`room`); S05 retains both
  conflicting Anchors, subtracts their union once, and reports 90m / 120m
  (`tight`). S09 is an overdue real Deadline, with no fictitious remaining
  ratio. S10 changes from current to expired advice on Tuesday and has no
  Deadline/pressure row. S12 short coverage preserves 90m / 120m but marks it
  conditional despite healthy refresh status. S13 distinguishes Trace's own
  satisfied due projection, independent unresolved zero-work obligations,
  unknown effort, and lower-bound insufficient work; stale Trace qualifies
  only the calculations that depend on it unless explicitly required.
- **Four product questions:** fixed facts are available as Anchor records,
  phases, and conflicts; candidate work before a fixed event is inspectable
  through Window/fit rows; risk is available in Deadline pressure and reasons;
  advice has its own type and validity. This supplies the semantic inputs to
  Horizon. It does not claim visual usability or a completed Today selector.
- **Diff/scope review:** reviewed the complete Gate 1 change against `8a59569`,
  including ownership/validation, civil-time boundaries, empty versus unknown
  capacity, fulfillment, qualifiers, inference, fixtures, and dependency
  direction. The settled product/spec documents are unchanged. No UI,
  persistence, source adapter, personal data, runtime agent dependency, network
  service, FI99 package, or remote publishing entered this gate. Git retains
  the imported foundation and task commits; no remote is configured.
- **Resolved integration defects:** Task 1.9 corrected fully blocked declared
  time being reported as unknown, effort conversion overflow, source coverage
  hidden by missing effort, and incomplete/unstably ordered explanation
  payloads. These implement the existing contract; no new policy was adopted.
- **Remaining questions/risks:** O-001–O-007 remain open. Compression/visual
  grammar and the actual Trace read contract are Gate 2 work; they do not
  block the synthetic shell in 2.1. proof-v1 is individual declared capacity,
  not a shared allocation or empirical safety promise. Adapter reconciliation,
  producer-managed revisions, Windows UI behavior, large personal datasets,
  and real-use calibration remain unverified. The pinned timezone rules need
  deliberate regression testing when updated. No personal-data trial occurred.

GATE 1 COMPLETE

---

## Gate 2 — Trace-backed primitive Horizon

**Status: IN PROGRESS — Tasks 2.1–2.3 complete; Task 2.4 next.** Deliver a usable personal Windows application using the
proven core. Keep one app backend plus the existing internal core; do not create
the speculative crate tree in Architecture. Each numbered task is one reviewable
commit. Build a visible synthetic app first, then integrate Trace before enabling
manual local inputs, following README's source rollout order.

### 2.1 Run a synthetic Horizon in the Windows app

**Status: COMPLETE — 2026-09-09.** Exact visual policy remains a prototype.

- **Purpose:** put the verified temporal model on screen immediately.
- **Dependencies:** Gate 1 complete; HORIZON and S01–S10; core `evaluate` output.
- **Expected files:** minimal root Svelte 5/TypeScript/Vite setup and lockfile;
  `src/`, `src-tauri/` with Tauri 2 configuration, narrow evaluation IPC, and
  synthetic fixtures; README launch/check commands. Add only the app workspace
  member and required dependencies. Keep fixture reuse internal.
- **Acceptance:** launch on Windows; select the ten required synthetic weeks
  and advance injected time. Render a bounded, continuous nonlinear Horizon
  with NOW, distinguishable species/provenance, pressure/unknown state, and
  selectable details. Near time has more space; distant time compresses;
  temporal order is preserved. Expose source qualifications and expired advice
  accurately. Keep the mapping deterministic and testable, and treat its exact
  function/visual grammar as an O-002/O-003 prototype requiring user feedback.
- **Focused verification:** core regression suite, frontend type/build checks,
  native app check/launch, mapping order/boundary tests, and visual inspection
  of S01–S10 including keyboard access, non-color semantics, and reduced motion.
  Present the running prototype for feedback without claiming feedback occurred.
- **Prohibited adjacent work:** SQLite, real sources, recommendation generation,
  polished motion/design system, conventional-calendar Home, extra reusable
  packages, public installers, or modifying the core model for renderer ease.

**Task 2.1 evidence:**

- Added the root Svelte/TypeScript/Vite setup, `src/`, `tests/horizon.test.ts`,
  and one unpublished `src-tauri/` workspace member. Narrow commands return a
  read-only presentation of existing S01–S10, S12, and S14 inputs and core output.
  README documents development, browser-preview, and bundled local-exe launch.
- Continuous logarithmic geometry preserves order and expands near time.
  Collision lanes move labels only; time stems and durations keep their true
  mapped positions. Facts, imported provenance, milestones, flexible preferences,
  retained advice, pressure, and source qualifications remain distinguishable.
- Passed: 4 app presentation tests, 4 Node geometry tests, all 78 core integration
  tests and 3 compile-fail doctests. `cargo fmt --all -- --check`, workspace
  all-target check, all-feature clippy with `-D warnings`, all-feature tests,
  frontend check (zero errors/warnings), production frontend build, native
  custom-protocol debug build, and diff checks. Cargo checks used locked/offline
  dependencies after initial acquisition. Final focused checks cover source
  freshness and the final frontend update.
- Visually inspected S01–S10 in the browser preview, including conflicting
  distinct Anchors, import/milestone identity, unknown availability, genuine
  overdue state, and expired advice. S03 stays at 8h effort while advancing time
  changes pressure from room to tight to insufficient. S10 remains a flexible
  Trace task after its advice expires. S12 exposes stale-source qualifications;
  S14 exposes flexible date-keyed occurrences. Keyboard Tab/Enter can inspect
  and close details; shapes/text supplement color. Desktop 900px/1440px layouts
  were inspected. No animations are introduced; reduced-motion CSS also disables
  transitions. The preview reported no browser warnings/errors.
- Launched `target/debug/temporal-app.exe` with bundled frontend on Windows;
  verified the titled native window and its WebView content. Normal app access
  was needed to create WebView2's own profile outside the workspace sandbox.
  Interactive UI checks were performed in the browser preview using the same
  Rust application boundary, not through native window automation.
- Scope review: no core source/spec change, SQLite, real Trace/calendar data,
  source adapter, recommendation generator, network service in the bundled app,
  mobile project, public installer, or remote publication. The localhost preview
  bridge is development-only and invokes a fixed synthetic binary.
- Residual limits: no personal-local trial yet; dense weeks need vertical
  scrolling, terse fixture names are not finished content, and native platform
  interaction coverage is limited. The prototype was presented for feedback;
  O-002/O-003 remain open pending an actual user response. Gate 2 is not complete.

### 2.2 Read Trace safely and retain last-known state

**Status: COMPLETE — 2026-09-09.** The first supported personal read boundary is
the versioned Trace 1.0 JSON export. The cache is app-owned and read-only with
respect to Trace.

- **Purpose:** replace synthetic Task References with a narrow, reliable local
  read boundary while retaining safe offline behavior.
- **Dependencies:** 2.1; verify O-004 against actual Trace source/documentation.
  The interrupted research ticket is evidence to inspect, not a settled schema.
- **Expected files:** `docs/TRACE-CONTRACT.md` verified mapping/transport notes;
  `src-tauri/src/trace.rs`, `store.rs`, app-owned SQLite migration(s), IPC/source
  status wiring, and synthetic adapter/store tests. Modules may be split only
  when their implemented size warrants it.
- **Acceptance:** establish the supported transport/version, stable IDs, status,
  due precision, completion, and reconciliation semantics before coding their
  mapping. Prefer supported export/query/IPC. Direct database read is eligible
  only if no cleaner contract exists and must be isolated/read-only; never
  write Trace's database. Persist canonical-ID mappings and cached source state
  in Temporal Engine's own local SQLite file. Refresh is atomic; failed/partial/
  incompatible loads retain prior facts and report health. Remove only on
  explicit deletion or authoritative complete reconciliation. Unknown fields,
  status, or due meaning cannot invent completion, deadlines, or free time.
  If no usable read contract can be established, record the precise blocker
  and retain synthetic mode; do not change Trace or guess its schema.
- **Focused verification:** synthetic stable-ID edits, due changes/removal,
  Done and unknown states, repeated import/restart, duplicates, empty complete
  versus failed/partial refresh, schema mismatch, and source health in the app.
  Inspect the adapter for write paths into Trace. Run app/core checks.
- **Prohibited adjacent work:** bidirectional task edits/completion, replacement
  task capture, real database fixtures, public adapter framework, cloud sync,
  Quercus or external calendar adapters.

**Task 2.2 evidence:**

- Inspected a clean checkout of Trace's own source without reproducing its
  internals. The verified export is the complete task catalog emitted after its
  first successful refresh; no database path or task-read IPC exists. The
  contract in `docs/TRACE-CONTRACT.md` records field ownership, missing
  precision, limits, reconciliation, age, and failure semantics. The research
  notes stay local and are not published.
- Added `trace.rs`, `store.rs`, `personal.rs`, the app-owned migration, the
  narrow native/preview commands, and a synthetic fixture. Import parses and
  validates the whole export before writing. Unknown envelope/task fields,
  unsupported versions, duplicate IDs, invalid timestamp/completion state,
  malformed/partial/old/future/conflicting exports, foreign/future cache
  schemas, and a deliberate mid-write SQLite failure retain prior facts. Empty
  complete exports create removed tombstones; reappearance reuses the canonical
  ID. Repeated imports retain record revisions and freshness expires at the
  original export-time boundary.
- Eleven `trace_contract` tests pass, including restart, stable UUID mappings,
  fractional sort order, status/completion edits, unresolved due values,
  source-health attempts, atomic rollback, and clock/schema boundaries. Four
  presentation tests, all core tests and doctests, frontend tests, frontend
  check/build, formatting, all-target/all-feature check, and all-feature clippy
  with `-D warnings` pass. The browser preview imported the committed synthetic
  export; the cache was read back through `scenario-preview` with two tasks,
  one completed task, one unresolved date, and healthy source state.
- The full all-feature workspace test command was attempted but Windows kept the
  existing test-owned `temporal-app.exe` open, so Cargo could not replace it.
  Package-level all-feature core tests and app presentation/Trace tests pass;
  the remaining lock is an environment limitation. The native bundled build
  is unchanged from Task 2.1 and awaits that process being closed. No real
  Trace database, export, or personal data was used.

### 2.3 Store local temporal input and scheduling annotations

**Status: COMPLETE — 2026-09-09.** Local records and work annotations are stored
in the application's own SQLite database and editable from My time.

- **Purpose:** make the app useful with the user's own explicit temporal state.
- **Dependencies:** 2.2; Domain Contract ownership, time, and work semantics.
- **Expected files:** app store/migrations and narrow mutation commands; simple
  local input/detail forms in `src/`; synthetic persistence/validation tests.
- **Acceptance:** create/edit local Anchors, Deadlines, Intentions, weekly
  Routines, explicit availability, and scheduling annotations on stable Trace
  references. Persist revisions and user evidence across restart; source edits
  retain annotations without overwriting sourced fields. Capture time once at
  the app boundary, inject it into the core, and keep virtual time available
  for scenarios. Require explicit timezone/precision and preserve all-day,
  completion, unknown effort/chunk, and source-ownership semantics.
- **Focused verification:** synthetic create/edit/restart and annotation
  survival; invalid overlap/time/link rejection without partial writes;
  source-owned field protection; local completion and Routine rollover; app
  and core checks plus manual form/keyboard use.
- **Prohibited adjacent work:** general task-manager CRUD, natural-language/AI
  parsing, generic persistence abstractions, imported-event dragging, travel,
  alternate recurrence systems, or broad settings infrastructure.

**Task 2.3 evidence:**

- Added `src-tauri/src/local.rs`, schema migration `002_local_state.sql`, and
  transactional store/IPC support. Separate core types retain local facts,
  user preferences, annotations, explicit completion evidence, and date-keyed
  Routine outcomes. UUID identities, revisions, and audit times survive restart.
  Source refresh cannot overwrite annotations or satisfy an independent local
  Deadline. Removed Trace identities remain labelled in the annotation picker.
- Added the collapsed `src/LocalEditor.svelte` manager and typed form mapping in
  `src/lib/local.ts`. Create/edit/remove Anchors, Deadlines, Intentions, weekly
  Routines, and bounded Availability; edit Trace work separately. Date-only
  precision, zone, occupancy, certainty, importance/milestone, unknown/zero
  effort, context/energy, pause, completion/reopen, and explicit outcomes remain
  distinct. Completed flexible entries no longer display as active preferences.
- Nine synthetic local-store tests cover create/edit/restart, v1 migration,
  source refresh/completion/removal/restore, atomic write failures, invalid
  overlaps/times/links, evidence retention, clock rewind, and Routine rollover.
  The full workspace passed 102 integration tests and 3 ownership doctests.
  After final display/picker fixes, all 24 app tests passed again. Nine frontend
  tests, frontend type/build checks, format, all-target/all-feature clippy with
  warnings denied, and the bundled Windows debug build passed. Cargo ran
  locked/offline; no dependencies changed.
- Browser walkthrough used synthetic data only: numeric Intention entry,
  timed Anchor/Deadline, prefilled deadline work, weekly Routine and recorded
  outcome, Availability split around an Anchor, Trace work picker, completion,
  and reload. Tab/Enter reached and submitted an Intention form. The expanded
  manager was inspected visually. Automated tests cover atomic rejection and
  date-only/DST boundaries; this is not a personal-data usability trial.
- Reviewed the task diff against ownership, time, soft-state, and gate scope.
  README and file inventory describe the new boundary. No Trace database,
  network integration, AI, mobile, or extraction work was added. The stale
  wayfinder summary now matches its already-resolved Trace research ticket.
- Residual limits: the first editor accepts exact or unknown effort, one weekly
  rule form, and an explicit IANA zone. DST gaps/folds are rejected without an
  offset-choice UI. Display zone remains America/Toronto. The finite Today edit,
  actual prototype feedback, and gate-wide native/offline walkthrough remain
  Tasks 2.4–2.5; Gate 2 is not complete.

### 2.4 Add the finite Today edit and inspectable advice

**Status: COMPLETE — 2026-09-10.** A bounded deterministic daily edit selects
from one existing evaluation, explains itself, and writes nothing.


- **Purpose:** make Fixed, Worth doing, On the radar, and Loose useful without
  asking the user to inspect the full result inventory.
- **Dependencies:** 2.3; existing fit/pressure/reasons/suggestion validity and
  the prototype feedback. Resolve bounded selection details in the contract
  before implementing them; a material product change needs a user decision.
- **Expected files:** a small internal Rust selection module, its documented
  deterministic policy and scenario tests; Today/details components in `src/`.
- **Acceptance:** a finite Today view with normally 3–5 flexible candidates,
  stable documented tie-breaking, and inspectable current fit/risk/source
  reasons. Suggestions carry producing keys and expiry and pass existing
  revalidation. Show uncertainty and individual-capacity limits; preserve
  Trace status instead of silently treating Now/Later/Someday as commitments.
  Do not fill a quota with ineligible work. User confirmation/selection is not
  completion, a reservation, or a new Deadline.
- **Focused verification:** fixed time/permutation tests for selection and
  explanation, insufficient/unknown opportunity, completed/removed targets,
  expiry and snapshot changes; S09 overdue versus S10 expired advice in the
  UI; full app/core checks and accessible keyboard inspection.
- **Prohibited adjacent work:** opaque ranking weights, learned calibration,
  shared-capacity optimization, rigid automatic schedules, guilt rollover,
  AI, or turning Today into another task manager.

**Task 2.4 evidence:**

- Added `docs/TODAY-POLICY.md` (`today-v1`) before implementing it: bounds,
  group definitions, caps, stable lexicographic ordering, advice shape, and the
  explicit non-goals. It refines the Domain Contract and changes no lifecycle,
  fit, pressure, or source rule. Implementation clarified two policy lines: the
  before-next-Anchor test names the next present, nontransparent Anchor later
  today, and awareness omits any cutoff the edit has already put in front of the
  user rather than only those in Fixed.
- Added `src-tauri/src/today.rs`. `today::evaluate` runs one core evaluation and
  `today::select` chooses from that single result. Today is the civil date in
  the display zone; advice is bounded by the earlier of civil midnight and the
  evaluation end, so a 23- or 25-hour day needs no special case. Fit for the
  remainder of today reuses the core calculation on Windows clipped to that
  remainder, keeping each producing Window identity, so a shortened day
  rechecks the declared useful-session length instead of assuming it still fits.
- Each selected flexible row carries a `consider_work` Suggestion with the core
  EvaluationKey, typed target, advisory range, `created_at = now`, and
  `valid_until = range.end`. Its typed reasons retain the Window, the
  today-clipped fit, source qualification, an explicit individual-capacity
  limitation, and associated unresolved pressure evidence. Feeding the produced
  advice back through `temporal_core::evaluate` returns Current for every row in
  all sixteen scenarios; a completion or snapshot-revision change retires it
  through the existing invalidated/expired rules instead of rewriting a target.
- Added `src/Today.svelte`, its projection in `presentation.rs`, and the
  `today` field on the app snapshot. Fixed shows four with a disclosure and
  discards nothing; Worth doing, Loose, and On the radar stay capped at three,
  two, and three with the omitted count visible. A row selects into the same
  inspector the Horizon uses, and a collapsed disclosure lists ordering
  rationale, conditional source basis, expiry, and typed evidence separately
  from stored facts. Trace status renders as `Trace: now/later/someday`.
- Nine new tests: seven in `src-tauri/tests/today_contract.rs` and two in
  `presentation_contract.rs`. They cover repeatability and input-permutation
  equality across all sixteen scenarios with the input left unchanged, caps and
  ordering, cleared availability, unknown compatibility, conditional last-known
  sources, civil-midnight clipping against the useful-session length, completed
  and removed targets, snapshot-revision invalidation, and awareness that does
  not repeat an advised cutoff. Two compile-fail doctests keep a selected row
  from carrying completion or becoming a stored obligation.
- Verification: `cargo fmt --all -- --check`, all-target/all-feature
  `cargo check`, `cargo clippy -- -D warnings`, and `cargo test --workspace
  --all-features` passed (111 integration tests, 5 ownership doctests, none
  failed or ignored; 33 of those are app tests). Nine frontend tests, `svelte-check` with warnings denied,
  the Vite build, and the bundled Windows debug build passed. Cargo ran
  locked/offline; no dependency changed.
- Browser walkthrough used synthetic data only. S02 showed today's two classes
  in Fixed, two advisory ranges, and awareness that no longer repeated the
  advised cutoff. S09 showed an empty Worth doing with the overdue cutoff on the
  radar; S10 advanced one day showed the task still flexible with the expired
  advice absent from Today and still inspectable in the Horizon. S07 with no
  declared availability produced four empty groups. S03 across three injected
  days moved the same estimate from room to tight to insufficient. Today rows
  are real buttons with text names, `aria-pressed` state, and normal tab order;
  activation was confirmed by click, since the harness could not deliver a
  synthetic Return to any focused control, including pre-existing ones.
- Local time now re-evaluates the cache each minute while visible and on window
  focus so the edit cannot silently go stale; the synthetic preview stays
  frozen and both surfaces state their evaluation time.
- Residual limits: `today-v1` orders by explicit typed signals only. It performs
  no shared-capacity allocation, no learned calibration, and no acceptance
  state, so overlapping ranges certify nothing about combined capacity. Effort
  and useful-session values remain uncalibrated user estimates. Compression and
  visual grammar (O-002/O-003) still await real use. The gate-wide native and
  offline walkthrough remains Task 2.5; Gate 2 is not complete.

### 2.5 Verify the primitive app and report Gate 2

**Status: COMPLETE — 2026-09-10.** Every Gate 2 criterion was checked against
the running app; the user recorded their own prototype reaction.


- **Purpose:** establish an honest working-app handoff before Quercus and the
  Gate 3 personal-local trial.
- **Dependencies:** 2.1–2.4, supported Trace boundary evidence, and actual
  prototype feedback; unresolved blockers preclude gate completion.
- **Expected files:** focused corrections where verification exposes gaps;
  README commands, `docs/BUILD-GATES.md`, ASTRA, and synthetic test evidence.
- **Acceptance:** the Windows app answers the five home questions from
  synthetic inputs and can consume the verified Trace contract while preserving
  local annotations and last-known source state. Review all ten Horizon weeks,
  virtual-time transitions, keyboard/reduced-motion behavior, restart/offline
  behavior, and the full gate diff. Record actual user feedback, checks,
  residual risks, and an executable Gate 3 plan before advancing ASTRA.
- **Focused verification:** all core/app tests, frontend type/build checks,
  native build/launch, synthetic adapter/store failure cases, and a manual
  end-to-end Horizon/Today/details/restart walkthrough. Keep personal content
  out of committed evidence. Mark completion only with all criteria satisfied.
- **Prohibited adjacent work:** Quercus implementation or a personal-data
  trial in this task, cosmetic expansion, public distribution, or declaring
  unperformed usability/integration checks passed.

**Task 2.5 evidence:**

- Full suite: `cargo fmt --all -- --check`, all-target and all-feature
  `cargo check`, `cargo clippy -- -D warnings`, and the whole `cargo test`
  passed with 111 integration tests and 5 compile-fail ownership doctests; none
  failed or ignored. Nine frontend tests, `svelte-check` with warnings denied,
  and the Vite build passed. Cargo ran locked/offline; no dependency changed.
- Native build and launch: the bundled `custom-protocol` executable started on
  Windows with its titled window responding, showing the real system date, the
  `Local data` badge, an empty local store, and a Today edit that said so
  instead of inventing rows. Its app-data cache was deliberately left untouched,
  so no synthetic or personal record was written to it; the personal-data trial
  belongs to Gate 3.
- Offline posture: the shipped bundle contains no occurrence of the development
  bridge path, retains the guard that refuses a non-Tauri build, and the Tauri
  capability set grants no filesystem, network, or shell permission. The app
  reads only its own app-data SQLite file and bytes the user explicitly chooses.
- All ten required Horizon weeks were reviewed at their base time and one
  injected day later. S01 advises one item, then nothing when the day has no
  declared availability. S02 keeps two fixed facts, two advisory ranges, and
  three awareness rows with five more counted. S03 moves the same 8h estimate
  from room to tight to insufficient. S04 shows six cutoffs today, three
  advisory rows, then six overdue and no recommendation the next day, with the
  three unshown counted rather than dropped. S05 keeps both seminars distinct.
  S06 recommends the insufficient item and says so. S07 stays empty in every
  group. S08 keeps an imported anchor, a local deadline, and a loose intention
  distinct. S09 shows an overdue cutoff as awareness with nothing recommended.
  S10 advanced one day keeps the task ordinary and flexible; its expired advice
  is absent from Today and still inspectable in the Horizon, never moralized.
- Adapter and store failure cases were exercised in the running app, not only in
  tests. A partial export was rejected with an explicit message; both cached
  Trace tasks, all seven local records, the retained work annotation, and the
  original import time survived, and source health became `partial` while the
  local source stayed healthy. Re-importing the valid export restored healthy
  health, kept the annotation, and left the unresolved due value unresolved.
- Restart: stopping and restarting the app boundary process left the cache with
  two Trace tasks, one completed, seven local records, one work annotation, one
  Routine outcome, and the original import timestamp.
- Accessibility and motion: every important state is carried in text as well as
  in shape and position — phase, risk wording, `· conditional`, source label,
  and source health. No element encodes overdue by colour without the word. The
  page has zero animated or transitioning elements, with the reduced-motion rule
  kept as a guard. Thirty-nine tabbable controls follow document order, skip
  link first, and Today rows are ordinary buttons with `aria-pressed` state.
- The five home questions in `docs/HORIZON.md` are answered: Fixed and the
  Anchor track answer what is fixed; the next-anchor panel and On the radar
  answer what comes next; Worth doing with its before-next-commitment rationale
  answers what can fit before it; risk wording and the Deadlines track answer
  what is gaining pressure; Loose, the dashed suggestion grammar, and expiry
  answer what is merely soft or inferred. The Gate 2 plan previously said "four
  home questions"; that count was corrected to five.
- Cross-document review of the full Gate 2 diff (`7fbea00..HEAD`, 57 files):
  the temporal core crate is unchanged. Only four documents changed —
  `DECISIONS.md` resolving O-004 to the verified export transport,
  `TRACE-CONTRACT.md` recording that verified transport, the new
  `TODAY-POLICY.md`, and this plan. No product decision was reinterpreted, no
  Trace database is touched, and no AI, mobile, cloud, travel, telemetry, or
  extraction work entered the gate.
- **Recorded user feedback, 2026-09-10.** Asked how the prototype actually
  reads now that a finite Today sits above the compressed surface, the user
  chose "Direction holds": Today leading with the Horizon below works; keep
  both and refine later from real use. Asked whether anything blocked the gate,
  the user chose "Mark it complete". O-003's composition is settled on that
  basis in `docs/DECISIONS.md`; the compression curve and the wider visual
  grammar stay open for the Gate 3 trial. No preference was inferred beyond
  those two selections.
- Residual risks: the model has never met real coursework, so pressure has no
  calibration evidence and effort remains an uncalibrated user estimate.
  today-v1 orders by explicit typed signals only and allocates no shared
  capacity, so overlapping advisory ranges certify nothing about a whole day. A
  deadline due today can appear in both Fixed and Worth doing, which is two true
  claims about one object but may read as repetition. The Trace boundary is a
  manual snapshot, so a task changed in Trace is invisible until re-export. The
  local editor rejects ambiguous daylight-saving times without an offset choice,
  has no lower-bound effort input, and the display zone is fixed to
  America/Toronto. Large personal datasets are unproven.

GATE 2 COMPLETE

Gate 2 excludes AI, travel, companion surfaces, mobile, cloud personal storage,
accounts, telemetry by default, FI99 extraction, and public distribution.

---

## Gate 3 — Quercus and real personal trial

Gate 2 proved the app against synthetic inputs and one snapshot source. Gate 3
adds the second real source and, for the first time, judges the model against
actual coursework. Work through these tasks in order, one task per commit.

### 3.1 Verify the Quercus source contract

- **Purpose:** establish what Quercus/Canvas actually exposes before writing an
  adapter, exactly as Task 2.2 did for Trace.
- **Dependencies:** Gate 2 complete; the user's own institutional access.
- **Expected files:** a verified-transport section in a source contract
  document; no credential, payload, or personal record in the repository.
- **Acceptance:** record the concrete read-only endpoints, authentication
  method, pagination, rate limits, and error shapes. Document the precision and
  timezone of every date field and which objects map to Deadline versus Anchor
  versus nothing. State explicitly what the source does not guarantee. Do not
  claim a field is reliable without evidence from the live API's own responses.
- **Focused verification:** the documented shapes match observed responses;
  synthetic fixtures are hand-written from the documented shape, never captured
  from a personal course. No token appears in any file, log, or test.
- **Prohibited adjacent work:** adapter implementation, write access, grade or
  submission data, announcements, discussion content, or any endpoint beyond
  what a deadline needs.

### 3.2 Store source credentials in OS-backed secure storage

- **Purpose:** hold one institutional token safely before any network call.
- **Dependencies:** 3.1.
- **Expected files:** a narrow credential module and its synthetic tests.
- **Acceptance:** the token is written to and read from Windows Credential
  Manager only. It never reaches the frontend, the SQLite cache, a
  configuration file, a log line, an error message, or a test snapshot. Removing
  the credential disables the source cleanly instead of failing obscurely.
- **Focused verification:** synthetic vault tests for store, read, absent, and
  removal; a repository scan proving no credential-shaped string is committed.
- **Prohibited adjacent work:** OAuth flows, multiple accounts, Google/Outlook
  credentials, sync services, or a general settings system.

### 3.3 Add the read-only Quercus adapter and its cache

- **Purpose:** bring real coursework deadlines into the same app-owned store.
- **Dependencies:** 3.1, 3.2; the Gate 2 store and source-health machinery.
- **Expected files:** a source adapter module, a schema migration, synthetic
  fixtures, and adapter/store tests.
- **Acceptance:** read-only fetch with minimum practical scope. Normalize into
  the existing `quercus` source kind with stable external-ID mapping, atomic
  reconciliation, tombstones, retained annotations, and last-known state on
  failure. A partial, stale, unauthorized, rate-limited, or offline refresh
  keeps the previous facts and exposes the source health. A date whose
  precision or zone the source does not establish stays unresolved rather than
  becoming a cutoff.
- **Focused verification:** synthetic fixtures for success, partial, malformed,
  unauthorized, rate-limited, empty, and out-of-order refreshes; restart
  persistence; offline behavior with last-known state clearly identified.
- **Prohibited adjacent work:** background polling loops, write-back, Google or
  Outlook adapters, notification infrastructure, or a generic adapter framework.

### 3.4 Present two real sources at once without merging them

- **Purpose:** keep provenance, health, and identity distinct when the same
  obligation appears in both Trace and Quercus.
- **Dependencies:** 3.3.
- **Expected files:** presentation and Today/Horizon adjustments; multi-source
  synthetic scenarios.
- **Acceptance:** each object keeps its own source identity and health. A
  coursework deadline and a Trace task about the same work are relatable
  without either overwriting the other, and the user can say which one owns the
  work estimate. Pressure and Today qualification reflect the weaker of the
  contributing sources. One failing source degrades its own rows, not the view.
- **Focused verification:** synthetic scenarios with both sources healthy, one
  stale, one unavailable, and the same assignment present in both; Today
  ordering and the full core/app suites.
- **Prohibited adjacent work:** automatic deduplication heuristics, fuzzy title
  matching, AI linking, or a general entity-resolution layer.

### 3.5 Run the personal-local trial

- **Purpose:** find out whether the model is right, using real coursework
  instead of synthetic inputs.
- **Dependencies:** 3.4; the user's willingness to use the app for a defined
  period.
- **Expected files:** a trial-observation record with no personal content
  committed; focused corrections where the trial exposes defects.
- **Acceptance:** the app runs on the user's real Trace and Quercus data for an
  agreed period. Record what the user actually reported: whether Today was
  worth reading, whether pressure matched felt urgency, where effort estimates
  were wrong, and any advice that felt like a fabricated obligation. Separate
  observation from change: a correction to explanation or normalization is in
  scope, a change to the temporal model is a new user decision.
- **Focused verification:** the recorded observations are the user's own words,
  not inferred; every corrective change carries a synthetic regression test.
- **Prohibited adjacent work:** silently retuning pressure, adding learned
  calibration, committing personal data, or expanding scope because the trial
  surfaced an interesting idea.

### 3.6 Verify Gate 3 and settle the visual decisions

- **Purpose:** close the open prototype questions with real evidence and hand
  off honestly.
- **Dependencies:** 3.1–3.5.
- **Expected files:** `docs/DECISIONS.md`, `docs/BUILD-GATES.md`, `ASTRA.md`,
  and the local tracker.
- **Acceptance:** O-002 and O-003 are decided from actual use, or explicitly
  kept open with the reason. Record checks, residual risks, and the next gate's
  plan. Mark completion only with every criterion satisfied.
- **Focused verification:** full core/app/frontend suites, native build and
  launch, offline and restart behavior, and a fresh gate diff review.
- **Prohibited adjacent work:** external calendars, travel, companion surfaces,
  mobile,
  cloud storage, FI99 extraction, packaging, or public distribution.

Gate 3 excludes a cloud personal datastore, always-running inference, telemetry
by default, and any write back to Trace or Quercus.

---

## Future gates

External calendars, companion-surface integration, travel/opportunity quality, effort calibration, refined visual design, packaging, and FI99 evaluation remain future work.

Do not pre-build them.
