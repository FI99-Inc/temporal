# Temporal Model

The temporal model is the semantic core of the product.

The model must represent different kinds of temporal claims without flattening them into a generic "event."

`DOMAIN-CONTRACT.md` defines the implementation-level types, ownership, invariants, and proof-v1 calculation policy for this ontology. The conceptual examples and possible later factors below do not override that bounded contract or the settled decisions.

## Core object classes

### Anchor

A fixed or externally constrained occurrence.

Examples:

- lecture at 14:00
- dentist appointment
- interview
- train departure

Properties may include:

- start
- end
- all-day flag
- location
- source
- source identifier
- rigidity
- confirmation state

Anchors can be local or imported.

### Deadline

A fixed endpoint by which something must occur.

Examples:

- assignment due Thursday at 23:59
- application closes Friday
- exam submission cutoff

A deadline is not the same as the work required to satisfy it.

A deadline may have associated estimated remaining effort.

### Task

Flexible work, normally originating in Trace.

Examples:

- message X
- finish valuation slide
- read case section

Tasks may have due dates, but a task may also be entirely unscheduled.

Temporal Engine enriches tasks with scheduling metadata without replacing Trace as the canonical capture/task system.

### Intention

A deliberately fuzzy possible future action.

Examples:

- maybe see Alex this weekend
- groceries Sunday
- work on Lacquer sometime tonight

An intention is neither a commitment nor a deadline.

### Routine

A recurring behavior with some degree of flexibility.

Examples:

- laundry on Sunday-ish
- weekly review
- gym routine

Routines should support recurrence without pretending all occurrences are rigid anchors.

### Window

A derived span of potentially usable time.

Examples:

- 52 free minutes before class
- 90 minutes on campus between fixed events

A Window is computed, not asserted as an obligation.

A Window must not be equated with genuinely usable opportunity until context, location, duration, and eventually energy are considered.

### Milestone

An Anchor or Deadline with elevated personal significance.

Examples:

- deferred exam
- major presentation
- trip departure
- important application decision

Milestone is an elevation of significance, not a substitute for the object's underlying temporal semantics. It does not create a second occurrence, deadline, or workload.

### Suggestion

A system inference.

Examples:

- start the case tonight
- use this 40-minute window for a light task
- this assignment is becoming risky

Suggestions must always remain visibly inferential.

## Cross-cutting dimensions

Object class alone is insufficient. Temporal objects may carry these dimensions.

### Source

Examples:

- local
- Trace
- Quercus
- Google
- Outlook

Imported provenance must be preserved.

### Certainty

Suggested initial vocabulary:

- sourced
- user-confirmed
- inferred
- tentative

Do not invent certainty.

These labels are not a single mutually exclusive scale: a source can report a tentative event. Provenance, user confirmation, and tentativeness must remain distinguishable in the implementation contract. None is a numerical probability that the event will happen.

### Rigidity

Suggested conceptual scale:

- fixed
- constrained
- movable
- fuzzy

Rigidity describes how costly or invalid it is to move an item in time.

### Importance

Importance is distinct from urgency.

### Effort

Optional estimated remaining effort.

Initial quick-entry buckets may include:

- 15m
- 30m
- 1h
- 2h
- 4h+

Do not pretend these estimates are precise.

### Energy

Later-capable but model-compatible:

- deep
- normal
- light

### Location / context

Optional semantic location or working context.

Examples:

- home
- campus
- library
- computer
- errands

Do not continuously track physical location in early versions.

## Fact versus inference

This boundary is mandatory.

Examples:

**Fact**
- Quercus says assignment due Sep 12 at 23:59.

**Inference**
- The user should start Sep 9.

**Fact**
- Outlook contains a meeting 15:00–16:00.

**Inference**
- The 45 minutes before it are usable for a light task.

Inferences may reference facts, but must not overwrite them.

Windows, pressure, and recommended start zones are also derived outputs. They must retain input references and explanations, and must not be presented as sourced availability or user commitments.

## Completion and overdue semantics

A real deadline can become overdue.

A fixed event can pass.

A flexible plan or suggestion does not become overdue merely because its suggested time passed.

If flexible work is still active and eligible, it returns to the scheduling/ranking candidate pool. Completed or explicitly removed work does not return merely because a suggestion expired.

Passing time is not evidence of completion, cancellation, or failure to attend. Deadline satisfaction needs explicit authoritative state; merely associating a task with a deadline must not silently make completing that task satisfy the deadline. Source unavailability is not evidence of deletion or completion.

## Pressure

Pressure is a derived property, not stored truth.

Conceptually:

`pressure = remaining work / realistic remaining opportunity`

modified by factors such as:

- deadline distance
- importance
- priority
- estimated effort
- availability
- contextual fit
- dependencies
- user calibration
- known upcoming load

These are possible factors, not a requirement to implement every factor in the first proof. The implementation contract and gate plan must specify the initial subset and how missing inputs are reported. A calendar gap alone is not proof of usable opportunity; unknown effort or availability must not be silently treated as zero work or unlimited free time.

The initial engine should prefer transparent deterministic formulas over ML.

Every pressure result must be explainable from inputs.

## Temporal compression

Temporal distance on Horizon is intentionally nonlinear.

Near time receives more representational space.

Farther time compresses.

The compression function must be deterministic, continuous enough for stable rendering, and testable with virtual time.

The exact formula is an implementation decision to be evaluated against visual/product criteria, not guessed into this document.

## Virtual-time requirement

Core temporal behavior must be testable against an injected clock.

Tests should be able to advance synthetic time without waiting for wall-clock time.

This is required for:

- deadline transitions
- pressure evolution
- recurrence
- Horizon positioning
- rollover behavior
- "overdue" semantics
