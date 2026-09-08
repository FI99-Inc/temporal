# Horizon Specification

## Purpose

Horizon is the primary interface of Temporal Engine.

It should communicate the shape of the near future without requiring the user to parse a conventional calendar grid.

## Primary mental model

Time progresses across a continuous surface.

The immediate future is expanded.

The distant future is progressively compressed.

Items appear to approach as time advances.

This is not decorative. The spatial treatment encodes psychological and operational relevance.

## Default horizon

Initial design target:

- now
- remainder of today
- tomorrow
- next several days
- approximately 14 days of radar

The exact visible extent may adapt, but v1 should not become an infinite timeline.

## Core visual distinctions

The interface must make these classes distinguishable without requiring verbose labels:

- fixed Anchor
- Deadline
- Task
- Intention
- Routine
- Milestone
- Suggestion
- Imported source state

Visual grammar should communicate rigidity and provenance quietly.

Do not solve this with a rainbow of unrelated category colors.

## Home questions

Horizon should make these answers immediately discoverable:

- what is fixed?
- what comes next?
- what can fit before it?
- what is gaining pressure?
- what is merely soft/inferred?

## NOW

The present moment is the strongest reference point.

Near NOW, Horizon may show:

- next anchor
- time until next anchor
- current available window
- one or a few fitting tasks
- imminent deadline pressure

Do not flood NOW with the complete database.

## Today editorial layer

Horizon should include a finite daily edit, conceptually:

### Fixed
Today's anchors and real deadlines.

### Worth doing
Usually 3–5 recommended flexible actions.

### On the radar
Approaching items that deserve awareness but are not necessarily today's work.

### Loose
A small set of intentions/routines that may fit.

This is an editorial surface, not a second task manager.

## Pressure visualization

Pressure should emerge gradually.

An item can have:

- a good start zone
- a should-start zone
- a risky zone
- its actual deadline

The visual treatment should avoid alarmist red-by-default guilt mechanics.

Pressure is information, not scolding.

## Interaction posture

Initial interactions should favor:

- click/select for details
- keyboard navigation
- inspect "why now?"
- lightweight drag where semantics are genuinely movable
- natural language quick capture for local temporal objects later

Dragging an intention is cheap.

Moving an imported fixed event is not.

The UI must respect those semantics.

## Imported objects

Imported events should retain visible provenance.

They must not quietly look locally authored.

Initial external calendar integrations are read-only.

## Conventional views

A month or week grid may exist later as a secondary utility called something other than Home.

It must not become the default just because implementation libraries make it easy.

## Motion

Animation should clarify temporal movement and state changes.

Do not build an ambient GPU toy.

The app must remain lightweight on a ThinkPad-class Windows laptop.

Respect reduced-motion settings.

## Accessibility

Horizon cannot encode semantic differences using color alone.

Every important state needs shape, label, texture, iconography, position, or accessible text in addition to any color treatment.

## Required prototype evaluation

Before polishing the visual system, test Horizon against synthetic weeks representing:

1. almost empty week
2. class-heavy week
3. one large assignment due in four days
4. many small deadlines
5. conflicting fixed anchors
6. large task with insufficient opportunity
7. soft intentions only
8. imported anchors plus local work
9. overdue real deadline
10. flexible suggestion whose proposed day has passed

The last case must not be rendered as a moralized overdue item.

`SCENARIOS.md` supplies the frozen inputs, semantic assertions, and boundary cases for these evaluations. Gate 1 proves their engine prerequisites; visual rendering and compression evaluation remain later-gate work.
