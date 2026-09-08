# Temporal Model

The temporal model is the semantic core of the product.

The model must represent different kinds of temporal claims without flattening them into a generic "event."

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

An event/deadline with elevated personal significance.

Examples:

- deferred exam
- major presentation
- trip departure
- important application decision

Milestone is an elevation of significance, not a substitute for the object's underlying temporal semantics.

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

## Completion and overdue semantics

A real deadline can become overdue.

A fixed event can pass.

A flexible plan or suggestion does not become overdue merely because its suggested time passed.

If flexible work was not completed, it returns to the scheduling/ranking candidate pool.

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
