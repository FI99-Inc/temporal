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

### 0.4 Produce Gate 1 task plan

Break Gate 1 into numbered, commit-sized tasks with:

- dependencies
- files/modules expected
- acceptance criteria
- focused verification
- gate-wide verification

### Gate 0 completion criteria

- documentation set is internally consistent
- no unresolved implementation-blocking semantic ambiguity for the authorized Gate 1 scope; explicitly deferred later-gate questions may remain open
- initial domain contract is precise enough to test
- scenario suite is defined
- Gate 1 implementation plan is executable
- no product source code was added unless separately authorized

When satisfied, append an evidence report below and mark:

`GATE 0 COMPLETE`

---

## Gate 1 — Temporal core proof

**Goal:** prove the domain model, virtual clock, deterministic pressure skeleton, and scenario harness before building the real UI.

Gate 1 exact tasks are to be produced by Gate 0.4 and ratified before implementation.

Expected broad scope:

- minimal repository scaffold
- `temporal-core`
- injected clock
- normalized object model
- deterministic serialization/validation
- synthetic scenario harness
- first pressure/risk primitives
- tests

Explicitly out of scope:

- polished Horizon UI
- Quercus OAuth/token handling
- Google/Outlook
- background sync
- AI
- travel
- mobile
- FI99 extraction

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
