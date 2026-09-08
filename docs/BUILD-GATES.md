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

**Status: COMPLETE.** All 12 imported files were inspected; the eight original `docs/` files were read in README order. The untouched import is preserved in baseline commit `4bbc312`.

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

**Completed scope:** Tasks 0.1–0.4 only: audited/ratified the constitution, defined the normalized domain contract, specified deterministic synthetic scenarios, and planned the first executable core proof. Local Git was initialized and the untouched import preserved in `4bbc312`; task commits are `d28fd89` (0.1), `b342b8d` (0.2), `8a8124b` (0.3), and `309767d` (0.4). The final evidence/handoff is a separate `docs: complete gate 0` commit.

**Documents:** created `DOMAIN-CONTRACT.md` and `SCENARIOS.md`. Updated root README, AGENTS, ASTRA, and FILE-STRUCTURE; updated `ARCHITECTURE.md`, `BUILD-GATES.md`, `HORIZON.md`, `PRODUCT.md`, `TEMPORAL-MODEL.md`, and `TRACE-CONTRACT.md`. `DECISIONS.md` and `PRIVACY.md` remain unchanged from the baseline.

**Verification performed:**

- Read the complete foundation in README order and reviewed the documents as one system against D-001–D-013 and the user's invariants. Reconciled independent read-only constitution, contract, scenario, plan, and fresh cross-document audits. Reviewed the full change from baseline `4bbc312`, including the new documents and final historical-Suggestion clarification.
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

**Status: IN PROGRESS.** The subsequent user instruction authorizes building the application. Execute this plan against `DOMAIN-CONTRACT.md` version 1 and `SCENARIOS.md` S01–S16 before proceeding to the later gates.

Gate 1 builds one internal Rust library crate, `crates/temporal-core`, plus local synthetic tests. The eventual desktop stack is unchanged, but no Tauri/Svelte/TypeScript application, SQLite/persistence crate, source adapter, UI/renderer, compression formula, network service, background process, AI, travel, companion surfaces, mobile, FI99 package/SDK, packaging, public distribution, or remote publication belongs in this gate. Source kinds and health are normalized synthetic data only. No real Trace database/calendar/token is a test dependency.

Use the task order below, one task per commit. Record actual files/check results in each task's evidence and update ASTRA to the next incomplete task only after its acceptance criteria pass. Future file/module paths below are expected organization, not files that already exist. A small private helper may be colocated differently without changing semantics; document meaningful deviations. Local focused commands use the test/module names established by the corresponding task, with the gate-wide commands fixed below.

### 1.1 Establish the minimal Rust core workspace

**Status: COMPLETE — 2026-09-08.** Added a single unpublished `temporal-core` library, workspace/lockfile, Rust 1.98.0 toolchain pin, line-ending/build/private-input rules, and README commands. The requested wayfinder map and its decision prerequisites are local Markdown under `.scratch/temporal-engine`; they retain this gate plan's authority.

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

Gate 1 is complete only when all ten task acceptances and these checks pass, every documented Gate 1 semantic scenario/variant is covered, canonical output is deterministic, the source/Trace/inference boundaries are preserved, and the scope review is clean. Evidence must distinguish synthetic algorithm proof from still-deferred visual, adapter, calibration, and personal-use validation. Append an actual gate report before adding a Gate 1 completion marker; this plan contains no such marker.

---

## Gate 2 — Trace-backed primitive Horizon

Broad intent only. Do not implement until Gate 1 is complete.

Expected:

- safe Trace adapter
- source health and conservative last-known-state behavior from the first adapter
- local annotations
- primitive Horizon renderer
- finite Today edit
- inspectable "why now?"

---

## Gate 3 — Quercus and real personal trial

Broad intent only.

Expected:

- Quercus adapter
- source health
- personal-local trial
- pressure behavior against real coursework
- no cloud personal datastore

---

## Future gates

External calendars, companion-surface integration, travel/opportunity quality, effort calibration, refined visual design, packaging, and FI99 evaluation remain future work.

Do not pre-build them.
