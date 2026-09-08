# Product Constitution

## One-sentence definition

Temporal Engine is a local-first personal time system that represents the near future as a nonlinear field of fixed obligations, flexible work, intentions, and rising pressure rather than as a wall of calendar rectangles.

## Problem

Traditional calendars are useful databases but poor primary interfaces for this user.

The failure modes to design against are:

- tedious manual maintenance
- rigid event semantics
- visually overwhelming grids
- guilt-inducing overdue piles
- redundancy with task systems
- weak prioritization
- poor representation of uncertainty
- no meaningful distinction between "this exists at 3 PM" and "I should probably do this today"
- equal visual treatment of psychologically unequal temporal distance

## Core questions the product must answer

At a glance:

1. What is happening now?
2. What is the next immovable thing?
3. What matters before then?
4. What is beginning to become risky?
5. Which items are facts, and which are merely suggestions?

## Primary interface

**Horizon** is home.

Horizon is not a standard day/week/month grid. It is a nonlinear near-future surface where temporal distance compresses as time recedes.

The immediate future receives more visual space than the distant future.

Traditional calendar views may exist later as secondary utilities where they genuinely outperform Horizon.

## Product principles

### Finite, edited attention

The primary surface should not dump every task and event.

A daily editorial layer should usually surface only a small number of recommended actions, plus fixed anchors and a compact radar of approaching risk.

### Facts are not guesses

A source-provided deadline is a fact.

"Start Tuesday" is an inference.

A user-created intention is deliberately soft.

The data model and UI must preserve those differences.

### Plans do not become moral debt

A flexible task suggested for Monday does not become "overdue" on Tuesday unless Monday was a real deadline.

Unfinished flexible work returns to the candidate pool.

### Pressure over rigid auto-scheduling

The engine should calculate whether work is becoming dangerous.

It should not initially create a minute-by-minute machine-authored itinerary.

### Explainability over magic

If the system raises an item, the user should be able to inspect why.

### Capture stays fast

Trace already solves instant task capture. Temporal Engine must not reproduce or burden that workflow.

### Local-first

The temporal model, rankings, personal metadata, and personal history live locally.

External services are source adapters, not the product's home.

## Initial user experience

A strong v1 should be useful with:

- Trace tasks
- manual temporal objects
- Quercus deadlines
- a correct Horizon
- a small Today editorial layer

Google/Outlook imports, travel calculations, effort learning, advanced analytics, and AI parsing are not required to prove the core product. Mobile application work is out of scope.

## Explicit non-goals

Initial versions must not become:

- a Google Calendar clone
- a generic task manager
- a kanban system
- an AI executive assistant
- an automatic appointment rescheduler
- a social calendar
- a team scheduling product
- a productivity analytics dashboard
- a mobile application
- a cloud SaaS product
- a companion mobile app
- an FI99 platform

## Success test

After real use, the product succeeds if the user opens it naturally and understands the shape of the next hours/days faster and with less dread than a traditional calendar or task list.

Feature count is not a success metric.
