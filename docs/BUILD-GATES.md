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
- no unresolved implementation-blocking semantic ambiguity
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
