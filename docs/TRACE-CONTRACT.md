# Trace Integration Contract

## Relationship

Trace is the fast task-capture system.

Temporal Engine is the time interpretation/navigation system.

Neither replaces the other.

## Source of truth

Trace remains authoritative for ordinary task identity and task-level state such as:

- task text
- task status
- priority
- context
- due date captured by Trace
- completion

Temporal Engine may maintain its own scheduling annotations keyed to a Trace task identifier.

Examples:

- estimated effort
- energy class
- earliest useful start
- temporal preference
- location/context refinement
- ranking history

Do not burden Trace's capture schema with Temporal Engine-only metadata in the first version.

## Initial fields available from Trace

The current Trace model provides the useful conceptual set:

- stable task id
- text/raw input
- status: Now / Later / Someday / Done
- context
- priority
- due date
- created/updated/completed timestamps
- sort order
- optional link

The adapter must treat unknown future fields conservatively.

## Mutability

Initial integration posture should be conservative.

Preferred first stage:

- Temporal Engine reads task state
- Temporal Engine stores its own annotations
- completing or editing Trace tasks from Temporal Engine is deferred until a deliberate bidirectional contract exists

Do not directly write into Trace's database from Temporal Engine.

## Synchronization semantics

The adapter must handle:

- Trace unavailable
- database/schema version mismatch
- deleted task
- completed task
- due date changed in Trace
- task moved between Now/Later/Someday
- task text edited
- duplicate/imported state prevention

Source failure must be visible in source health state.

## Identity

Never match tasks by text alone.

Use a stable Trace task identifier.

Temporal Engine annotations must survive ordinary Trace edits where the task ID remains stable.

## Companion surface

A future Horizon bay may live visually inside a companion application's own
sidebar surface.

That is a presentation integration, not permission to merge the applications.

The bay may show a very small amount of temporal state such as:

- next anchor
- time until next anchor
- current usable window
- one recommended item

That surface must not become a second full Horizon.

## Versioning

Any local contract between Trace and Temporal Engine must be versioned.

Breaking changes require explicit migration/fallback behavior.

## Privacy

Never copy a real Trace database into repository fixtures.

Use synthetic task fixtures.
