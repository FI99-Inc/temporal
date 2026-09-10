# Primitive Today policy — today-v1

This is the bounded Gate 2 implementation of Horizon's daily edit. It refines
the Domain Contract; it changes no lifecycle, fit, pressure, or source rule.
The visual grammar and compression remain provisional pending user feedback.

## Inputs and bounds

Use one validated snapshot and its core evaluation at the injected `now`.
Today is the civil date in the explicit display zone, resolved with the pinned
timezone rules. Advice is bounded by the earlier of the next civil midnight
and the evaluation end. It never assumes a 24-hour civil day.

- **Fixed:** present Anchors intersecting today's civil day, plus present real
  Deadlines whose displayed due date is today. For an on-date cutoff, display
  the date containing the instant immediately before its exclusive endpoint;
  exact cutoffs use the endpoint itself. Keep resolved/passed facts labelled.
  Sort by time, then species/ID. Initially show four, with a disclosure for the
  rest; do not silently discard today's fixed facts.
- **Worth doing:** at most three distinct eligible Task/standalone Deadline
  work targets with a definite useful-session fit in the remainder of today.
  A Deadline linked to Trace uses its Task's single work identity.
- **Loose:** at most two eligible Intentions/Routine occurrences with the same
  definite-fit requirement. Preferred times remain soft; past preferences
  acquire neither pressure nor overdue state.
- **On the radar:** at most three remaining present unresolved/unknown real
  Deadlines within the evaluation range (including older overdue cutoffs), or
  future Anchors within that range. Awareness covers only what the edit has not
  already shown: exclude anything in Fixed, any Deadline a flexible row targets,
  and any Deadline whose risk a flexible row already displays. Sort overdue,
  insufficient, tight, then all other awareness by actual time and species/ID.
  Show the number omitted; the full Horizon remains inspectable.

An empty group is valid. Never fill a quota using unknown fit, completed or
removed work, zero effort, or a fabricated availability declaration. Known
fit can be conditional on source health; expose that qualification.

## Stable work ordering

For each eligible work target, ask the existing core fit calculation to assess
only the remainder of today. Choose its earliest definite fitting Window;
break equal starts by end and canonical Window key. This is an advisory range,
not a reserved block or a proposed minimum-duration itinerary.

Order Worth doing lexicographically by:

1. A range that ends by the start of the next present, nontransparent Anchor
   later today comes first. When no such Anchor exists the test is vacuous.
2. Associated deadline risk: insufficient, tight, other known unresolved,
   unknown, then no unresolved deadline.
3. Earliest associated real cutoff (no cutoff last).
4. Explicit annotation importance: high, normal, low, unspecified.
5. Earliest fitting range start, then canonical WorkTarget order/ID.

Loose uses the same before-next-Anchor preference, then a currently preferred
Intention or a current Routine date, then earliest fitting range and canonical
WorkTarget order/ID. That preference affects ordering only. A past preference
is no more urgent than an undated one. WorkTarget's established canonical order
is Deadline, Intention, RoutineOccurrence, Task; occurrence keys include date.
Trace Now/Later/Someday, source priority, and source sort order retain their
source meaning and do not become commitments or hidden ranking weights.

## Advice and explanation

Each selected flexible target produces a `consider_work` Suggestion with the
core EvaluationKey, typed target, chosen advisory range, `created_at=now`, and
`valid_until=range.end`. Its nonempty typed reasons retain the producing Window,
the today-clipped fit, source qualification, an explicit individual-capacity
limitation, and the associated unresolved pressure evidence when one exists.
Run the existing core validity calculation on the selected advice before
presenting it; every shown row must be current.
Display the policy, ordering rationale, conditional source basis, and expiry
separately from the target's stored facts. Individual fitting ranges can overlap;
the list does not certify combined capacity or promise completion.

Selection writes nothing. Selecting a row only opens its evidence. New input
or a later evaluation regenerates the edit; historical advice, when supplied,
keeps the core's invalidated/expired/ineligible/current rules. New suggestions
are not persisted as obligations. Native My time re-evaluates the cache on a
minute tick while visible and on window focus, so the edit does not silently go
stale; the synthetic preview remains explicitly frozen.
The display always identifies its evaluation time.

No learned weights, AI, shared-capacity allocation, acceptance state, reminders,
calendar mutation, or new temporal species are part of this policy.
