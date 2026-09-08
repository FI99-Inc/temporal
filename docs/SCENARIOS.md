# Virtual-Time Scenarios

## Fixture conventions

These are synthetic, deterministic specifications for `DOMAIN-CONTRACT.md` version 1 and **proof-v1**. Gate 1 turns them into fixtures and assertions; Gate 0 contains no executable engine or fixture files. S01–S10 correspond, in order, to the ten required cases in `HORIZON.md`. S11–S16 expose semantic boundaries, not additional features.

Each scenario is independent. Variants start from its base input unless stated otherwise. The defaults below are part of every case, so fixture authors must not invent missing metadata:

- The stated base `now` is also `captured_now`; snapshot revision is 1. `evaluation_end` is base now plus exactly 14 elapsed days, calculated once. Clock-only steps keep the snapshot, that end, and all facts/estimates unchanged. Mutation variants increment snapshot revision and affected record/annotation revisions, set captured_now to the variant's explicit now (base now if unchanged), and use that instant for newly recorded/updated/observed fields. Existing historical fields remain unchanged.
- Display/object timezone is **America/Toronto** unless specified. September times below are local with offset `-04:00`; each case supplies UTC now. Every date/time converts to the canonical UTC millisecond form. Unqualified hour ranges use the stated date and are half-open. Pin timezone rules in Gate 1 and record the version; the DST cases below fix the required resolutions.
- IDs use aliases in this document only. Sources are L=`10000000-0000-4000-8000-000000000001` (local), T=`10000000-0000-4000-8000-000000000002` (Trace), C=`10000000-0000-4000-8000-000000000003` (Google calendar), Q=`10000000-0000-4000-8000-000000000004` (Quercus). Labels are `Synthetic local`, `Synthetic Trace`, `Synthetic calendar`, and `Synthetic coursework`. Include only named sources.
- For a stored object/availability alias, allocate `00000000-0000-4000-8000-SSSNNNNNNNNN`, where SSS is the three-digit scenario number and N is its nine-digit declaration ordinal. Assign in table/bullet order; expand repeated dates chronologically. Keep IDs unchanged across variants; new aliases get new ordinals. These are fixed synthetic UUID v4 values, never random test IDs. Suggestions/occurrences/Windows use the contract's derived keys.
- Records are present, revision 1, with created/updated/asserted/observed times at base now minus 1 millisecond. Titles are the quoted synthetic label, or the alias if no label is given. Local records use L provenance. Imported records use the named source, external_id=`synthetic-<alias>`, projection matching their species; a Trace due projection reuses its task external_id with projection task_due. Source revision is `synthetic-1`; source created/updated/completed times are omitted unless supplied.
- External sources default to a complete successful observation at base now minus 1 millisecond, fresh_for_ms=1,209,600,000, Task catalog complete for T, and Anchor/Deadline coverage `[base now - 7 elapsed days, evaluation_end + 1 elapsed day)` for the roles they supply. Trace task_due uses Task catalog coverage, with no separate Deadline coverage. Required roles are T/tasks, C/anchors, Q/deadlines when those sources appear. Other fields use the contract's absent/empty forms.
- Anchors are fixed, confirmed, busy TimedSpans unless changed explicitly. Deadlines use exact `at` cutoffs and recorded unresolved fulfillment unless declared on_date or task_due. Intentions are active. Routine rules are active. Importance is unspecified; milestone is false; there is no confirmation annotation unless supplied.
- Each WorkMetadata specifies effort and chunk below. Unless stated otherwise: no earliest_start, required_contexts is empty, energy_requirement is unrestricted. `E=60/chunk=15` means estimate 60 minutes with a 15-minute minimum useful session. `E=?` means unknown; `E>=240` means at_least(240). Zero work has no chunk.
- Every availability declaration defaults to contexts known `{desk}` and energy capacity deep. Declarations are the complete declared work envelope for the evaluation, not claims about all possible free time. No declarations means unknown opportunity, except mathematically empty permissible time which is known zero.
- Omitted species/annotation/outcome/prior-suggestion collections are empty arrays in the eventual fixture. Annotations needed for stated work/significance/confirmation are explicit stored user state with the same audit defaults. No real tasks, calendars, locations, credentials, or source requests are involved.

All numerical opportunity totals below are minutes for readability. Expected output uses milliseconds (`minutes * 60,000`), and ratio payloads contain the unreduced original work/opportunity milliseconds. Fractions printed here are only readable comparisons.

For every valid fixture: check exact classification/phase, Window/fit inventory, stated risk and numeric-field presence, provenance, relevant reason codes/input references, source qualifications, unchanged input bytes after evaluation, and deterministic output under repeated evaluation and reordered input collections. Recommendation constraints are checks on fit/validity and eventual presentation, not authorization to build a recommendation generator in Gate 1. Horizon assertions are semantic acceptance evidence for Gate 2; Gate 1 proves their data prerequisites without a renderer or compression formula.

## S01 — Almost empty week

**Clock:** `2026-09-07T13:00:00.000Z` (Monday 09:00). **Zone:** America/Toronto. **Sources:** L, T.

| Alias | Input |
| --- | --- |
| A1 | Local Anchor, Friday Sep 11 14:00–15:00, "Short appointment" |
| T1 | Trace Task, Now, no due value, E=15/chunk=15, "Small errand" |
| V1, V2 | Availability Monday Sep 7 and Wednesday Sep 9, each 09:00–11:00 |

**Expected:** A1 is upcoming and the next known Anchor; T1 is movable unfinished work. Exactly two 120-minute Windows and two fitting T1/Window rows exist. There are no Deadline or pressure results. Declared source inputs are healthy.

**Recommendation constraints:** T1 may be considered within either declared Window; neither is reserved. No minimum number of recommendations is required.

**Horizon meaning:** a quiet near future, a distant fixed item on the radar, and one flexible candidate. Near versus distant time retains the nonlinear Horizon requirement; no particular coordinate is prescribed.

**Must not conclude:** the entire remaining week is free; lack of deadlines proves caught-up status; T1 is due today; unused space needs invented tasks.

## S02 — Class-heavy week

**Clock:** `2026-09-07T13:00:00.000Z`. **Zone:** America/Toronto. **Sources:** L, T.

| Alias | Input |
| --- | --- |
| A1–A10 | Local class Anchors Monday Sep 7 through Friday Sep 11: each day 10:00–12:00 then 14:00–17:00 |
| D1 | Local Deadline Friday Sep 11 17:00, standalone E=480/chunk=90, required_contexts={campus}, "Course preparation" |
| T1 | Trace Task, Later, no due value, E=30/chunk=15, required_contexts={campus}, "Short reading" |
| V1–V5 | Availability each weekday Sep 7–11 09:00–17:00, contexts known {campus} |

**Expected:** ten distinct fixed Anchors; ten Windows (daily 09:00–10:00 and 12:00–14:00). Raw remaining Window time is 900 minutes. D1 uses only the five 120-minute spans because its chunk is 90: O=600, E/O=4/5, tight. T1 fits all ten Windows. The next Anchor starts Monday 10:00.

**Recommendation constraints:** no 90-minute work suggestion can use a 60-minute gap or bridge a class. Later does not manufacture a time assignment. The class Anchors remain fixed.

**Horizon meaning:** fixed load dominates the week; the short immediate Window is distinguishable from longer later opportunities; D1 has explainable pressure.

**Must not conclude:** five weekday rectangles are all available work time; every gap is compatible with every task; classes can move to make an assignment fit; T1 is overdue because it is in Later.

## S03 — One large assignment due in four days

**Clock:** `2026-09-07T13:00:00.000Z`. **Zone:** America/Toronto. **Source:** L.

| Alias | Input |
| --- | --- |
| D1 | Deadline Friday Sep 11 09:00, standalone E=480/chunk=60, high importance, milestone=true, "Large assignment" |
| V1–V4 | Availability Monday Sep 7 through Thursday Sep 10, each 09:00–14:00 |

**Expected:** one fixed Deadline with significance overlay and one workload, no duplicate Milestone object. Initially O=1,200, E/O=2/5, room. With the same snapshot at `2026-09-09T13:00:00.000Z` (Wednesday 09:00), O=600 and risk is tight (4/5). At `2026-09-10T13:00:00.000Z`, O=300 and risk is insufficient (8/5). Work remains 480 minutes through all clock steps.

**Recommendation constraints:** suggest attention only from explicit work/Window evidence; no generated eight-hour appointment. Importance/significance do not multiply E/O. Increasing pressure does not prove work was attempted or skipped.

**Horizon meaning:** the Deadline approaches from the radar while pressure becomes more prominent; its actual cutoff remains separate from possible start opportunities.

**Must not conclude:** elapsed time reduces effort; the assignment was started; milestone adds a second deadline; a suggested start zone is a sourced time constraint.

## S04 — Many small deadlines

**Clock:** `2026-09-07T13:00:00.000Z`. **Zone:** America/Toronto. **Source:** L.

| Alias | Input |
| --- | --- |
| D1–D6 | Six local Deadlines Monday Sep 7 at 10:00, 10:30, 11:00, 11:30, 12:00, 12:00; each standalone E=45/chunk=15 |
| V1 | Availability Monday Sep 7 09:00–12:00 |

**Expected:** six real deadlines and one Window. Individual O values are 60, 90, 120, 150, 180, 180. Ratios are 3/4, 1/2, 3/8, 3/10, 1/4, 1/4: D1/D2 tight, D3–D6 room. Each target has one fitting clipped row. D5 precedes D6 on equal endpoints by canonical ID. Each quantified result includes individual_capacity_only.

The fixture author can see total work is 270 minutes against 180 declared minutes, but proof-v1 emits no joint-feasibility verdict or allocation. That limitation is an assertion, not a hidden second scheduling algorithm.

**Recommendation constraints:** individual fit cannot justify promising all six completions or reserving the same minutes six times. Any later finite edit must keep real deadlines inspectable.

**Horizon meaning:** compact approaching deadline load and explained individual pressure, without six invented rigid work blocks.

**Must not conclude:** mostly room ratings mean everything jointly fits; equal cutoff times justify unstable sorting; the engine has solved competition for capacity.

## S05 — Conflicting fixed anchors

**Clock:** `2026-09-07T13:00:00.000Z`. **Zone:** America/Toronto. **Sources:** L, C.

| Alias | Input |
| --- | --- |
| A1 | Local Anchor Monday Sep 7 10:00–11:30, "Seminar" |
| A2 | Imported C Anchor Monday Sep 7 11:00–12:00, also "Seminar" |
| D1 | Local Deadline Monday Sep 7 13:00, standalone E=90/chunk=30 |
| V1 | Availability Monday Sep 7 09:00–13:00 |

**Expected:** A1 and A2 remain two facts with different provenance despite matching text. One conflict spans 11:00–11:30. Their blocking union is 10:00–12:00 (120 minutes, not 150). Windows are 09:00–10:00 and 12:00–13:00; O=120, ratio=3/4, tight.

**Variant:** set A2 occupancy transparent, preserving its time/source. No scheduling conflict remains. Windows are 09:00–10:00 and 11:30–13:00; O=150, ratio=3/5, tight. A2 still exists.

**Recommendation constraints:** neither Anchor moves automatically; no work fits across the blocking union. An imported event cannot be rewritten as a local suggestion to resolve conflict.

**Horizon meaning:** show the conflict and both sources; distinguish it from work pressure.

**Must not conclude:** duplicate text proves duplicate identity; overlapping blocks subtract twice; a transparent overlap is a scheduling conflict; either event was cancelled.

## S06 — Large task with insufficient remaining opportunity

**Clock:** `2026-09-07T13:00:00.000Z`. **Zone:** America/Toronto. **Sources:** L, T.

| Alias | Input |
| --- | --- |
| T1 | Trace Task Later, E=240/chunk=60, due references D1, "Long work item" |
| D1 | T1's task_due projection, Tuesday Sep 8 17:00; intrinsic task work and trace_task fulfillment |
| A1, A2 | Local Anchors Monday Sep 7 10:00–12:00 and Tuesday Sep 8 09:00–11:00 |
| V1, V2 | Availability Monday Sep 7 and Tuesday Sep 8, each 09:00–12:00 |

**Expected:** one task/work identity and its separate fixed due Deadline. Only Monday 09:00–10:00 and Tuesday 11:00–12:00 remain. E=240, O=120, ratio=2, insufficient. Task catalog coverage is sufficient for the healthy Trace due projection; no separate Deadline coverage is needed.

**Variant:** change chunk to 90. Raw Windows still total 120, but neither can hold a useful session: O=0, insufficient, no ratio, explicit zero_opportunity/chunk_too_short reasons.

**Recommendation constraints:** identify shortage; do not reserve unavailable time, shrink the user's estimate, change Trace status, or move the deadline.

**Horizon meaning:** risky flexible work is distinct from its real endpoint and fixed blockers.

**Must not conclude:** 120 fragmented minutes satisfy 240 minutes of work; smaller gaps can be stitched across Anchors; insufficient opportunity proves the user will fail.

## S07 — Soft intentions only

**Clock:** `2026-09-07T13:00:00.000Z`. **Zone:** America/Toronto. **Source:** L.

| Alias | Input |
| --- | --- |
| I1 | Intention "Groceries maybe", preferred DateSpan Sep 6–7, E=30/chunk=15 |
| I2 | Intention "Sketch an idea", preferred DateSpan Sep 7–8, unknown effort and chunk |

No availability is declared. There are no Anchors, Deadlines, or Tasks.

**Expected:** I1 is preference_passed but active; I2 is preferred_now. There are no Windows, fit rows, or pressure results. Opportunity is not established by the empty calendar.

**Variant:** add V1, availability Monday Sep 7 18:00–19:00. I1 fits; I2 fit is unknown because its chunk is unknown. The passed preference does not change I1's original DateSpan or completion state.

**Recommendation constraints:** optional soft consideration only where current inputs justify it; no required daily quota and no definite suggestion for I2 based on invented metadata.

**Horizon meaning:** a quiet Loose area; preference passage has no overdue alarm or obligation styling.

**Must not conclude:** groceries were promised Sunday; Sunday created debt; I2 is due tonight; a preference declares availability.

## S08 — Imported anchors plus local work

**Clock:** `2026-09-07T17:00:00.000Z` (Monday 13:00). **Zone:** America/Toronto. **Sources:** L, C.

| Alias | Input |
| --- | --- |
| A1 | Imported C Anchor Monday Sep 7 14:00–15:00; local milestone=true and current confirmation of record revision 1 |
| D1 | Local Deadline Monday Sep 7 16:00, standalone E=90/chunk=45, "Prepare notes" |
| I1 | Local Intention "Optional outline", E=30/chunk=15, no preferred span |
| V1 | Availability Monday Sep 7 13:00–16:00 |

**Expected:** imported fixed A1, local fixed D1, and fuzzy local I1 remain distinct. Windows are 13:00–14:00 and 15:00–16:00; D1 O=120, ratio=3/4, tight. A1's confirmation/significance does not change imported provenance, mutability, occupancy, or object count.

**Variant:** at the same frozen now, a source revision moves A1 to 15:00–16:00, retaining external identity and canonical ID. Stored annotation survives, but its record_revision=1 confirmation is no longer current. One Window 13:00–15:00 remains; O=120 and D1 stays tight.

**Recommendation constraints:** work may fit around the recorded Anchor; local scheduling metadata cannot move it or become source truth.

**Horizon meaning:** visible source provenance and confirmation/significance overlay without duplicate obligations; work and suggestions remain separate from the imported time.

**Must not conclude:** user confirmation makes an import locally authored; source refresh can erase annotations; an annotation can edit imported start/end; a source time edit creates a new identity.

## S09 — Overdue real deadline

**Clock:** `2026-09-08T13:00:00.000Z` (Tuesday 09:00). **Zone:** America/Toronto. **Source:** L.

| Alias | Input |
| --- | --- |
| D1 | Local Deadline Monday Sep 7 17:00, standalone E=60/chunk=30, unresolved |
| V1 | Availability Tuesday Sep 8 09:00–11:00 |

**Expected:** D1 is unresolved/overdue. Pressure risk is overdue; workload/opportunity/ratio are omitted by the early-return rule. The current Window does not become before-cutoff capacity: D1's fit row is does_not_fit with no span. No infinite/negative ratio is emitted.

**Variants:** explicit local satisfaction recorded Tuesday 08:30, with known effective completion Monday 16:30, makes D1 inactive/satisfied and not_applicable; the cutoff is retained. Separately, changing only E to zero (no chunk) leaves unresolved D1 overdue.

**Recommendation constraints:** surface the real missed cutoff and need for status/action; no automated recovery timetable or new due date.

**Horizon meaning:** distinguish a real overdue deadline from soft rollover; expose its factual/local evidence.

**Must not conclude:** the task was completed by passage; zero effort proves satisfaction; the user failed to attend an Anchor; present free time retroactively satisfies a deadline.

## S10 — Flexible suggestion whose proposed day has passed

**Base clock:** `2026-09-07T13:00:00.000Z`. **Evaluation step:** `2026-09-08T13:00:00.000Z`. **Zone:** America/Toronto. **Sources:** L, T. Keep base captured_now, snapshot revision, and evaluation_end across the step.

| Alias | Input |
| --- | --- |
| T1 | Trace Task Now, no due value, E=60/chunk=30, "Flexible reading" |
| V1, V2 | Availability Monday Sep 7 and Tuesday Sep 8, each 09:00–11:00 |
| G1 | Prior consider_work Suggestion targeting T1, produced at base now, proposed DateSpan Sep 7–8, valid_until=`2026-09-08T04:00:00.000Z`; reason declared_availability references V1 and its producing evaluation |

**Expected:** G1 is current at base now and expired at valid_until equality and Tuesday's evaluation. T1 remains Now/unfinished with the same due=none and effort. At Tuesday 09:00, only V2 supplies a current Window and T1 still fits it. There are no Deadline/pressure results and no automatically generated replacement suggestion.

**Variant:** an authoritative Trace Done snapshot revision arrives at Tuesday 09:00. G1 is invalidated before the expiry check because its basis changed; T1 is excluded from fit candidates. The change comes from Trace, not G1's elapsed time.

**Recommendation constraints:** a new producer may later reconsider still-active T1 from current inputs. Reusing G1 as current is prohibited.

**Horizon meaning:** the old inference expires quietly; eligible work can remain flexible. No overdue badge or rollover pile for G1 or T1.

**Must not conclude:** suggested Monday means a real Monday deadline; G1 was accepted/completed/failed; missed suggestions create obligations; completed work returns to the pool.

## S11 — Civil dates, exact boundaries, and daylight saving

**Fall base clock:** `2026-11-01T04:00:00.000Z` (Sunday 00:00, offset -04:00). **Zone:** America/Toronto. **Source:** L. The usual 14 elapsed-day evaluation end applies.

| Alias | Input |
| --- | --- |
| A1 | Busy DateSpan Nov 1–2 |
| D1 | on_date Nov 1 in America/Toronto, standalone E=60/chunk=30 |
| V1 | Timed availability `[2026-11-01T04:00:00.000Z, 2026-11-02T05:00:00.000Z)` |

**Expected:** the civil day lasts 25 hours. A1 blocks all 1,500 declared minutes; D1 is due_today with zero opportunity and insufficient risk. Retiring A1 at base now produces one 1,500-minute Window, ratio=1/25, room. Its date representation is preserved, not converted into a stored arbitrary midnight event.

**Boundary checks:** at `2026-11-02T04:30:00.000Z`, D1 is still due_today (Toronto Nov 1 23:30), even if display_zone is UTC. At `2026-11-02T05:00:00.000Z`, D1 is overdue and A1 passed. In an independent variant with D1 cutoff changed to exact at that same instant, equality is due_now/insufficient with known zero opportunity; one millisecond later is overdue. No declarations is still zero at that exact endpoint.

**Spring independent fixture:** base now=`2026-03-08T05:00:00.000Z` (00:00 -05:00); substitute DateSpan Mar 8–9, on_date Mar 8, and V1 ending `2026-03-09T04:00:00.000Z`. All audit defaults reset to this earlier base. The day is 23 hours/1,380 minutes. Busy all-day occupancy removes it exactly; transparent occupancy preserves it and creates no scheduling conflict.

**Normalization checks:** local `2026-03-08 02:30` is invalid (gap). Local `2026-11-01 01:30` without a disambiguation is invalid; explicit -04:00 resolves to 05:30Z and -05:00 to 06:30Z. A date with no established zone or unknown meaning is not normalized by guessing the host zone or end-of-day precision.

**Recommendation constraints / Horizon meaning:** whole-date facts keep their date/zone identity; day-length and endpoint calculations use real civil boundaries. A date-only deadline is distinguishable from an exact-time deadline without a fabricated 23:59 label.

**Must not conclude:** every day is 24 hours; display-zone changes move commitments; a due date ends at the start of its own date; an exact inclusive cutoff is overdue at equality.

## S12 — Source health, coverage, and apparent empty time

**Clock:** `2026-09-07T13:00:00.000Z`. **Zone:** America/Toronto. **Sources:** L, C. C overrides freshness: last complete attempt/success Monday 08:30; fresh_for_ms=3,600,000; full default Anchor coverage.

| Alias | Input |
| --- | --- |
| A1 | Imported C Anchor Monday Sep 7 10:00–11:00 |
| D1 | Local Deadline Monday Sep 7 12:00, standalone E=90/chunk=30 |
| V1 | Availability Monday Sep 7 09:00–12:00 |

**Expected:** healthy C; two Windows 09:00–10:00 and 11:00–12:00; O=120, ratio=3/4, tight, qualification declared_inputs.

| Independent variant | Expected consequence |
| --- | --- |
| Latest attempt at 09:00 failed, partial, or incompatible; preserve A1 and last success | Respective unavailable/partial/incompatible health; same known-input O=120 and tight risk, conditional with source reasons |
| Same snapshot clock at 09:30 | C is still healthy at exact freshness equality; O=90, ratio=1, tight |
| Same snapshot clock at 09:30:00.001 | C stale; pre-10:00 gap is now shorter than chunk=30, so O=60, ratio=3/2, insufficient and conditional |
| Healthy C coverage ends 10:00, before D1's 12:00 endpoint | Known A1 retained; O=120 remains conditional because coverage is insufficient |
| Independent never-loaded C with no A1 and no success/coverage, explicitly required for anchors | One declared 180-minute Window and numeric known-input ratio=1/2, but conditional/never_loaded; absence is not verified availability |
| Independently successful complete empty C catalog with full coverage and no A1 | One 180-minute Window, ratio=1/2, tight with declared_inputs; still conditional on user-declared availability in the ordinary product sense |

Failed/partial refresh variants specify the same retained normalized Anchor. Gate 1 proves evaluation of retained state; the actual adapter's no-deletion-on-failure transition is a Gate 2 test obligation.

**Recommendation constraints / Horizon meaning:** keep last-known fixed events visible with source status. Advice based on stale/incomplete sources carries that qualification; empty results cannot silently announce free/caught-up status.

**Must not conclude:** an error cancels A1; expired freshness deletes it; omitted required sources can be ignored; healthy records prove source coverage outside the completed query.

## S13 — Completion authority, unknown work, and lower bounds

**Clock:** `2026-09-07T13:00:00.000Z`. **Zone:** America/Toronto. **Sources:** L, T, Q.

| Alias | Input |
| --- | --- |
| T1 | Trace Done, completed timestamp absent, stored E=120/chunk=30, due references D1 |
| D1 | T1's Trace due projection, Sunday Sep 6 17:00, trace_task fulfillment |
| D2 | Independent Q Deadline Monday Sep 7 12:00, recorded unresolved, user work link to T1 |
| D3 | Local Deadline Monday Sep 7 12:00, standalone E=?/chunk=30 |
| D4 | Local Deadline Monday Sep 7 12:00, standalone E>=240/chunk=60 |
| D5 | Local Deadline Monday Sep 7 12:00, standalone E=0, no chunk |
| V1 | Availability Monday Sep 7 09:00–12:00 |

**Expected:** D1 is satisfied/inactive and not_applicable, without fabricating completed_at. D2 remains unresolved/upcoming with effective linked workload estimate(0), no_known_work, zero_work_unresolved/task_completion_basis reasons. T1's stored 120-minute annotation is unchanged. D3 risk unknown with workload unknown and no computed opportunity. D4 O=180, lower-bound ratio=4/3, insufficient labelled at_least. D5 is no_known_work but unresolved. T1 is not a fit candidate; only D3/D4 generate fit rows (D3 can fit its known chunk even though pressure effort is unknown).

**Stale-source variant:** at unchanged base now, T's complete attempt/success is Monday 08:00, fresh_for_ms=1,800,000; T1/D1's observation and local record audit times are also 08:00. T is stale. Both D1's last-known satisfaction and D2's effective zero are conditional even if T/tasks is removed from explicit required_sources; factual dependencies still require it.

**Unknown-status variants:** use two independent reduced fixtures to preserve the single-work-owner invariant. First, keep only L/T sources, T1, D1, and V1; set T1 status unknown and unknown_status_label=`SyntheticFutureStatus`. D1 fulfillment/phase/risk is unknown. Second, keep L/T/Q, T1, D2, and V1 with that unknown status/label, but T1 due=none and no D1 projection; D2 work/risk is unknown. Neither variant creates two unresolved/unknown work links.

**Other variants:** authoritative T1 removal also retires D1, retains T1 annotations, and leaves D2 unresolved with unknown work. Explicit Q satisfaction of D2 resolves it; local confirmation alone does not.

**Recommendation constraints / Horizon meaning:** show a resolved Trace due fact separately from an unresolved external obligation; unknown/lower-bound estimates and source freshness are inspectable. Neither zero estimate is a completion badge.

**Must not conclude:** completing associated work satisfies every linked deadline; a missing timestamp can be invented; 4h+ means exactly 4h; source status can be rewritten by local effort annotations.

## S14 — Soft routine rollover

**Clock:** `2026-09-06T13:00:00.000Z` (Sunday 09:00). **Zone:** America/Toronto. **Source:** L.

| Alias | Input |
| --- | --- |
| R1 | Active weekly Routine, weekdays={sun}, start_date=2026-08-30, no until, E=30/chunk=15, "Weekly reflection" |
| V1, V2 | Availability Sunday Sep 6 and Sunday Sep 13, each 09:00–10:00 |

**Expected:** the bounded evaluation generates Sep 6/current and Sep 13/future occurrences; each fits only its own date's Window. Sep 20's midnight–09:00 portion also intersects the 14-elapsed-day evaluation range, so that occurrence is generated but has no own-date availability fit. No Deadline or pressure exists.

**Clock step:** at `2026-09-07T13:00:00.000Z`, the explicit occurrence helper classifies Sep 6 as past_unrecorded; normal current/future output excludes it. Sep 13 and Sep 20 retain stable `(R1,date)` keys. No completion/skip log is added and no missed-instance debt accumulates.

**Variants:** explicit skipped outcome for Sep 6 at base now excludes that occurrence from fit; paused R1 produces no candidates. For rule editing, first record that skipped outcome, then explicitly change R1 weekdays to {mon} at the same frozen now; start_date and bounds remain unchanged. The outcome is retained, Sep 6's helper state is inactive, and generated dates are Sep 7 and Sep 14, with no fitting Monday availability. Separately, add R2 with R1's original Sunday rule/work/title and no outcomes; same-date R1/R2 occurrences have distinct keys, and no shared Window is reserved by either.

**Recommendation constraints / Horizon meaning:** routine dates are loose preferences, never fixed blocks or due obligations. Expired preferences do not crowd the present; future opportunities remain optional.

**Must not conclude:** recurrence makes every occurrence an Anchor; an unrecorded Sunday was skipped or failed; a pause erases history; the engine should expand recurrence without a finite bound.

## S15 — Unknown compatibility and definite impossibility

**Clock:** `2026-09-07T13:00:00.000Z`. **Zone:** America/Toronto. **Source:** L.

| Alias | Input |
| --- | --- |
| D1 | Deadline Monday Sep 7 12:00, standalone E=90/chunk=45, required_contexts={desk}, energy_requirement=deep |
| V1 | Availability 09:00–10:00, contexts known {campus}, deep capacity |
| V2 | Availability 10:00–11:00, contexts unknown, deep capacity |
| V3 | Availability 11:00–12:00, contexts known {desk}, normal capacity |

**Expected:** three raw Windows; fits are does_not_fit (context), unknown (context), does_not_fit (energy). Pressure opportunity unknown with known qualifying subtotal zero, risk unknown, no ratio. Unknown V2 is not counted as either usable or proven unusable.

**Variants:** V2 contexts known {desk} gives O=60, ratio=3/2, insufficient. V1 energy unknown remains a definite mismatch because its known context fails. No availability gives unknown opportunity. In that no-availability variant, also setting earliest_start to Monday 12:00 (or later), and leaving chunk unknown, gives known O=0/insufficient: no permissible time exists. Evaluating at exact D1 cutoff likewise gives due_now/insufficient; at cutoff plus one millisecond it is overdue.

**Recommendation constraints / Horizon meaning:** distinguish raw space, known compatibility, unknown compatibility, and actual shortage; no inferred location, travel time, energy, or session size.

**Must not conclude:** unknown context is a match; known subtotal zero proves total zero; uncertainty overrides a mathematically empty interval; deep work fits normal capacity without an explicit change.

## S16 — Thresholds, validity, identity, and invalid snapshots

**Clock:** `2026-09-07T13:00:00.000Z`. **Zone:** America/Toronto. **Sources:** L, T.

| Alias | Input |
| --- | --- |
| D1 | Local Deadline Monday Sep 7 11:00, standalone E=60/chunk=15 |
| T1 | Trace Now, no due value, E=60/chunk=60 |
| V1 | Availability Monday Sep 7 09:00–11:00 |
| G1 | consider_work T1, proposed TimedSpan 09:00–10:00, created at base now, valid_until 10:00; reason declared_availability for V1 |
| G2 | risk_notice D1, no proposed span, created at base now, valid_until 11:00; reason references its producing PressureResult |

**Expected:** D1 O=120, ratio=1/2, tight; both suggestions current at base now. At `2026-09-07T13:30:00.000Z` (09:30), G1 is ineligible even though it has not expired: only 30 minutes remain in its proposed span against T1's 60-minute chunk. T1 still fits the broader 09:30–11:00 Window. G2 remains current with recalculated O=90/ratio=2/3 and current reasons. The stored suggestions/facts do not mutate.

**Threshold variants at base now:** set D1 effort to 59 -> room; 60 -> tight; 120 -> tight; 121 -> insufficient, all with chunk=15. Unknown effort -> unknown and no opportunity computed; zero/no chunk -> no_known_work; at_least(60) -> tight with lower-bound meaning. Changing only importance to high or T1 source_priority to the opaque string `synthetic-high` does not change pressure arithmetic. A new snapshot/settings basis invalidates both old suggestions before their other checks. A new synthetic risk_notice constructed with the room variant's own producing EvaluationKey is ineligible after revalidation.

**Range/due variants:** D1 cutoff at exactly evaluation_end remains in range: O=120, tight. Cutoff at evaluation_end plus one millisecond yields unknown/outside_evaluation_range, with workload/opportunity/ratio omitted; fitting the known Window does not establish full remaining capacity. Separately, T1 due=`unresolved("Friday-ish", "meaning_not_established")` stays inspectable and unresolved; it creates no additional Deadline or pressure result and cannot be called proof of undated work.

**Determinism/negative fixtures:** make each following mutation independently to the named case; reject the entire normalized fixture with a typed validation issue. None may be partially evaluated into more apparent free time.

| Mutation | Required issue category / preserved boundary |
| --- | --- |
| S16 schema_version=2, unknown normalized field, unknown normalized tag, duplicate JSON key | unsupported_version / unknown_field / invalid_value / invalid_value respectively |
| S16 duplicate ID, nil/malformed UUID, or duplicate required-source/set entry | duplicate_identity / invalid_identity / duplicate_identity |
| S08 duplicate imported identity key under another ID; separately, same text under distinct source keys | First rejected duplicate_identity; second remains two distinct facts |
| S16 missing task/deadline/source reference; wrong species in a typed link | missing_reference / invalid_relationship |
| S06 second present unresolved Deadline reusing T1 work | invalid_relationship; no double-counted effort |
| S16 overlapping availability or start >= end | invalid_time; no double-counted spans |
| S16 negative/fractional minutes, zero chunk, positive estimate smaller than its chunk, null required field | invalid_value |
| S16 effort beyond unsigned 32-bit or revision beyond unsigned 64-bit; separately, T fresh_for_ms=18,446,744,073,709,551,615 causing expiry arithmetic overflow | invalid_value for out-of-range inputs; arithmetic_overflow for the checked expiry calculation, never a clamped healthy source |
| S16 now before captured_now or a local recorded time after captured_now | invalid_time |
| S13 known unfinished T1 with source_completed_at present; local-user fulfillment evidence on imported D2 | inconsistent_state / invalid_relationship |
| S12 complete source outcome with no success timestamp, or never with fabricated coverage | inconsistent_state |
| S14 empty weekdays, invalid until bound, unsupported recurrence kind | invalid_value / invalid_time / invalid_value |
| S10 Suggestion with a completion/overdue field, or inferred record placed in factual collection | unknown_field / invalid_value |

For valid base/variants, permute record arrays and context sets, evaluate twice at identical now, and compare canonical bytes including tie order, IDs, reasons, and input immutability. Verify unreduced E/O pairs and row-specific absence of uncomputed values; never encode infinity/NaN. Advance only injected time without sleeps. No OS timezone, source transport, UI, random IDs, or live data participates.

**Recommendation constraints / Horizon meaning:** expired, invalidated, and currently unsupported suggestions cannot retain authoritative-looking advice. Invalid data produces explicit errors, never invented certainty or visually hidden obligations.

**Must not conclude:** current-by-expiry alone proves current fit; source task priority changes effort pressure; invalid input can be silently dropped; two runs may disagree because map iteration or wall-clock time differs.

## Gate 1 coverage handoff

Gate 1 must implement all S01–S16 base cases and named variants at their appropriate layer: validation/normalization, temporal lifecycle, bounded recurrence, source health, opportunity/fit, pressure, and suggestion validity. Assertions about future Horizon rendering remain documented acceptance constraints; Gate 1 reports them as semantic prerequisites proven, visual verification deferred. Adapter refresh transition tests remain Gate 2 work, using these retained-state cases as their expected normalized outputs.

No scenario settles O-002/O-003's compression/visual design or O-004's real Trace transport. No scenario establishes a joint scheduler, learned estimate, private dataset, integration, mobile surface, or FI99 API.
