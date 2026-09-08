# AGENTS.md — Temporal Engine

This repository is built through Astra-style gated development. The repository, commits, tests, and durable project documents are the handoff. Chat context is not authoritative.

## First action on every takeover

Before editing:

1. Inspect the repository tree.
2. Read `ASTRA.md`.
3. Read the product/spec documents listed in `README.md`.
4. Inspect recent commits and the working tree.
5. Identify the **first incomplete gate/task** in `docs/BUILD-GATES.md`.
6. Resume there. Do not restart completed work.
7. Preserve existing commits and partial work.
8. Do not run destructive reset/clean commands merely to simplify the state.

## Authority order

When instructions conflict, use this order:

1. User's latest explicit instruction
2. `docs/DECISIONS.md` settled decisions
3. Product/spec documents
4. `docs/BUILD-GATES.md`
5. Existing implementation
6. Implementation convenience

If a contradiction cannot be resolved safely, stop that branch of work and record it in `ASTRA.md` rather than silently choosing a new product direction.

## Core product invariants

- Temporal Engine is **not** a conventional calendar with a custom skin.
- Horizon is the primary interface.
- Sourced facts and inferred suggestions are distinct types and must never be silently collapsed.
- A flexible plan that was not completed is not automatically an overdue obligation.
- Real deadlines may become overdue.
- Pressure is preferred over machine-authored rigid time blocking.
- Suggestions must be inspectable and explainable.
- Core ranking/scheduling logic must be deterministic.
- AI may assist parsing or interpretation, but must not become the source of truth for deadlines, event existence, completion, free time, or scheduling state.
- Trace remains a separate fast task-capture application.
- Temporal Engine may consume Trace through a narrow contract. Do not turn Temporal Engine into a replacement task manager.
- Do not make Temporal Engine depend on Astra, Codex, or any agent at runtime.
- Companion mobile-app integration is out of scope.
- No mobile app.
- No cloud database.
- No telemetry by default.
- No public account system, plugin marketplace, generic SDK, or enterprise abstractions.
- Do not design for FI99 extraction yet. Preserve clean seams only.

## Working discipline

Use one numbered task per commit unless a task is too small to stand alone and the gate explicitly groups it.

For implementation work:

1. State the task and acceptance criteria.
2. Add or identify failing tests where appropriate.
3. Implement the smallest complete change.
4. Run focused tests.
5. Run the relevant broader test/type/build checks.
6. Review the diff for scope violations and accidental product changes.
7. Commit with a concise task-oriented message.
8. Update `ASTRA.md` and `docs/BUILD-GATES.md` only when the evidence justifies advancing state.

Never mark a gate complete because code "looks done."

## Gate completion evidence

A gate report must include:

- exact scope completed
- relevant files/modules
- tests/checks run and results
- manual verification, if applicable
- deviations or ambiguity
- residual risks
- explicit `GATE N COMPLETE` only when every gate criterion is satisfied

## Product change discipline

Do not casually edit settled specifications to make implementation easier.

When a real product decision is required:

- record the question in `ASTRA.md`
- preserve the current behavior
- state alternatives and tradeoffs
- wait for a user decision if the choice materially changes product behavior

## Security and privacy

Never commit:

- OAuth tokens
- Quercus tokens
- calendar credentials
- personal calendar exports
- Trace database copies
- personal locations
- screenshots or fixtures containing private real-world data

Use synthetic fixtures for tests.

## Definition of success

The first meaningful success condition is not feature count.

It is: a working Horizon built on a correct temporal model can, using synthetic and then personal-local inputs, answer:

- What is fixed?
- What matters before the next fixed thing?
- What is becoming risky?
- What is only a suggestion?
