# Temporal Domain Contract

## Status and scope

Version 1 is the normalized, internal contract for Gate 1. It refines `TEMPORAL-MODEL.md` under `DECISIONS.md` and the authority order in `AGENTS.md`. It is not a public API, storage schema, Trace wire format, or authorization to build adapters. Rust can implement these definitions as separate structs and small enums in one internal crate.

The initial calculation policy is **proof-v1**. Its explicit thresholds and restrictions make the core testable; they are not a claim that effort calibration or the final Today ranking has been solved. Changing this policy later requires a versioned, tested change, not an undocumented tuning constant.

## 1. Vocabulary and ownership

| Classification | Meaning and examples | Writer |
| --- | --- | --- |
| Stored facts | Explicit local assertions: an Anchor exists at a given time; a real Deadline has a cutoff | User, through local application commands |
| Imported facts | A source reports an Anchor, Deadline, or Trace Task Reference, including reported status and timestamps | Owning source, through its future adapter |
| Stored user state | Effort estimates, work links, availability declarations, intentions, routine rules/outcomes, local resolution, significance, confirmation annotations | User, through local commands |
| Derived state | Temporal phase, overdue status, Windows, fit assessments, pressure, source health, explanations | Deterministic evaluation of explicit inputs |
| Inferred suggestions | Recommendations about possible work or attention, with an evaluation basis and expiry | Explicit inference producer; never a factual record writer |

An imported fact means **the source reported it**, not that the event certainly occurs or the source is current. An estimate stored by a user remains an estimate. Persisting or caching derived output does not promote it to a fact.

There is no universal mutable `event` record with optional fields for every species. Share identifiers, time primitives, metadata, and work annotations; keep Anchor, Deadline, Trace Task Reference, Intention, and Routine separate. Window and Suggestion are outputs. Milestone is an overlay on an Anchor or Deadline.

## 2. Identity and record metadata

### Canonical identifiers

- `SourceId`, `AnchorId`, `DeadlineId`, `TaskRefId`, `IntentionId`, `RoutineId`, and `AvailabilityId` are distinct typed identifiers with canonical lowercase, hyphenated UUID strings. Stored IDs are non-nil UUID v4 values, allocated once at creation/first normalization, outside evaluation.
- IDs are immutable and unique across a snapshot, including removed records. Evaluation never generates random IDs. Synthetic fixtures use fixed valid IDs.
- A source instance has a stable `SourceId`; two calendars/accounts/databases of the same kind are distinct sources. This is local identity, not a public account system.
- An imported identity key is `(source_id, external_id, occurrence_key?, projection)`. External strings are nonempty, opaque, case-sensitive, and never trimmed or matched by title. `projection` is one of `anchor`, `deadline`, `task`, `task_due`.
- Future adapters keep a durable key-to-ID mapping. Re-importing the same key reuses the ID and local annotations through text, due-time, or status edits. Different sources are not merged automatically. Proven duplicate records from one key are an input error, not two obligations.
- An imported recurring occurrence uses a source-stable occurrence key, normally the source's original recurrence identifier, not its possibly moved start. Missing stable identity is a normalization failure, not permission to use text/time heuristics.
- A Trace task's due projection uses its task external ID with `projection=task_due`; moving its due date preserves the Deadline ID. Removing and later restoring the due value reuses that identity. An unrelated Deadline is never identified by matching the task's title.

### Common metadata for stored records

Every temporal object and AvailabilityDeclaration has `meta={id, revision, created_at, updated_at}`: its typed ID, a positive integer revision, and local observation/command instants. Revisions increase on accepted changes. `created_at <= updated_at`. Imported source timestamps are separate and never substituted for these audit times. Source catalog entries and keyed annotations/outcomes use the smaller field inventories below; they do not acquire a second object identity.

Anchor, Deadline, Task Reference, Intention, and Routine also have a nonblank `title` and `presence=present|removed`. Removed means explicitly retired from the active dataset, not completed. Retain identity and annotations in a tombstone; physical persistence/deletion policy is later work.

`Provenance` on factual records is either:

- `local_user`: local source ID and the explicit user assertion's recorded time; or
- `imported`: source ID, imported identity key, `observed_at`, optional opaque source revision, and optional `source_created_at` / `source_updated_at`.

The imported source must exist in the input source catalog. Source timestamps may be unreliable; preserve them without using them to order revisions or prove completion. Observation times and local audit times come from injected time. Snapshot audit/observation times cannot lie after its captured `now`; a counterfactual earlier evaluation needs an earlier snapshot.

Local annotations have their own revision and audit times and are keyed by the target's ID. Refreshing source fields does not overwrite annotations. No credentials, raw database copies, or arbitrary source payload bags belong in this contract.

## 3. Time primitives and clock

### Instants and spans

- `Instant` is a UTC instant at millisecond precision, serialized as `YYYY-MM-DDTHH:mm:ss.sssZ`. Arithmetic uses checked integer milliseconds. Input precision must be established before normalization; never turn an ambiguous source string into a precise instant by guessing.
- `LocalDate` is an ISO Gregorian date, `YYYY-MM-DD`. `ZoneId` is an IANA timezone identifier. An offset alone resolves a single instant; it is not a recurrence timezone.
- `TimedSpan` is `[start, end)` with `start < end`, plus an optional original source timezone. Endpoint-touching spans do not overlap. A point-like Anchor still needs an explicitly supplied end; missing duration is not defaulted to one hour or zero.
- `DateSpan` is `[start_date, end_date_exclusive)` with `start_date < end_date_exclusive`, and a required timezone. It represents whole civil dates, not a floating flag on a midnight event.
- `TemporalSpan` is exactly `TimedSpan` or `DateSpan`. Resolve DateSpan boundaries to instants using its timezone for calculations, while retaining its original dates for display/serialization. A civil day can be 23 or 25 hours.
- Supported normalized dates/instants are years 0001 through 9999, with every needed endpoint representable. Overflow or an unresolvable endpoint is a validation error; do not clamp silently.

### Timezone resolution

Absolute offset-bearing source timestamps can normalize to UTC even if the source supplies no IANA zone. Zone-less local date/time input requires an explicit source zone or a recorded user choice of zone; the machine's current zone is never an implicit fallback. If an offset and zone are both provided, they must agree at that instant.

A nonexistent local time in a daylight-saving gap is invalid. An ambiguous local time in a fold needs an explicit offset or explicit earlier/later choice recorded during normalization; never silently choose one. Civil-date boundary resolution must also be unique, otherwise report `unresolvable_civil_boundary`.

Timezone rules are an explicit, pinned dependency of evaluation/tests. The test harness records the rules version it uses and does not depend on the Windows machine's current timezone. A display-zone change changes formatting and the meaning of display labels such as Today; it never moves stored instants, source date spans, or deadline cutoffs. A user changing an object's date timezone is an explicit revision.

### Evaluation time

The application boundary captures `Clock.now()` once per evaluation. The pure core receives that frozen `now`, the snapshot, an explicit `evaluation_end > now`, a display zone, timezone rules, and `policy_version=proof-v1`. No domain, recurrence, fit, pressure, or sorting code reads wall-clock time, the OS zone, random numbers, or network state.

The evaluation interval is `[now, evaluation_end)`, with `captured_now <= now`. A future deadline beyond that interval receives an out-of-range assessment, not invented future capacity. Advancing time is a new explicit evaluation of the same or a newer snapshot; it is not a mutation of source facts or completion state.

`EvaluationKey` is the tuple `(snapshot_revision, now, evaluation_end, display_zone, timezone_rules_version, policy_version)`. The caller changes the snapshot revision whenever any object, annotation, source state, or setting changes. Equal snapshots in different collection orders retain the same revision and must produce equal output.

## 4. Cross-cutting dimensions

### Certainty and confirmation

Provenance answers who asserted a fact. `reported_certainty=confirmed|tentative|unspecified` describes the assertion, not a probability. Anchor carries this field; a local tentative Anchor and an imported tentative Anchor are valid.

An optional stored user `confirmation` annotation records `confirmed_at` and the factual record revision it confirms. It is current only while that revision matches. Confirmation does not change provenance, source-reported certainty, times, completion, or mutability. A source update leaves an older confirmation as history, not current endorsement.

Deadline normalization requires an established real cutoff, not a tentative proposed work time. Unknown source due semantics remain unresolved on the Task Reference or in a future adapter's normalization error. User-authored Intentions and Routines are soft user state, not falsely labelled sourced facts. Window/pressure are derived; Suggestion is always inferred, even if a user has read or acted on it.

### Rigidity and significance

| Species | Rigidity | Meaning |
| --- | --- | --- |
| Anchor | `fixed` or `constrained` | Its recorded interval is an actual occurrence/constraint; only an authorized explicit edit can change it |
| Deadline | `fixed` | Real endpoint, not a proposed session |
| Trace Task Reference | `movable` | Work is flexible; its separate due Deadline may be fixed |
| Intention | `fuzzy` | A preference, not a commitment |
| Routine occurrence | `fuzzy` | A soft recurrence preference, not an Anchor |
| Window | not applicable | Computed opportunity candidate, not an object to move |
| Suggestion | not applicable | Its time hint is advisory |

`constrained` does not authorize automatic rescheduling or require a new constraint solver. Both Anchor rigidities use the same occupancy rules in proof-v1; rigidity alone does not make a transparent Anchor block time.

Local significance annotations contain `importance=unspecified|low|normal|high`, default `unspecified`. Importance is not urgency, Trace priority, or a pressure multiplier in proof-v1. Trace priority remains source-owned. `milestone=true|false` (default false) is allowed only on Anchor/Deadline; it adds personal significance without creating another ID, workload, or temporal state. It does not automatically set importance or alter risk arithmetic.

### Work metadata

`WorkMetadata` is stored user state with:

| Field | Values / semantics |
| --- | --- |
| `effort` | `unknown`, `estimate(minutes)`, or `at_least(minutes)`; integer minutes, nonnegative estimates, strictly positive lower bounds |
| `minimum_chunk_minutes` | Optional positive integer: smallest useful session the user declares. No hidden default |
| `earliest_start` | Optional Instant: explicit earliest useful work bound, not a deadline |
| `required_contexts` | Set of exact semantic tags; empty means no context restriction |
| `energy_requirement` | `unrestricted`, `light`, `normal`, or `deep`; default unrestricted |

The 15m, 30m, 1h, and 2h entry buckets become estimates 15, 30, 60, and 120; `4h+` becomes `at_least(240)`. They are not measured facts. Zero estimated work never means completed; it can mean that only confirmation/submission remains. With a positive estimate, a declared minimum chunk must not exceed that estimate. A zero estimate has no minimum chunk and is not a work recommendation candidate.

Missing effort or minimum chunk is explicit uncertainty; Gate 1 must not invent it. For positive known effort, fit/pressure need a declared minimum chunk. Session splitting is permitted only to the extent implied by that chunk; proof-v1 sums qualifying spans, without assigning sessions.

Context tags are user-defined nonempty lowercase ASCII slugs (`a-z`, digits, hyphen), compared exactly. They represent meanings such as `desk` or `campus`, not coordinates. Source context strings remain separate source fields; mapping/refinement to tags is an explicit user annotation or verified later adapter mapping. Required tags use all-of matching. No travel time or physical location is inferred.

An explicitly declared energy capacity `light` supports light work, `normal` supports light/normal, and `deep` supports all three. An unknown capacity cannot prove compatibility with a stated requirement. Energy is user input, not sensed physiology. Unrestricted work needs no energy assertion.

## 5. Stored temporal species

### Anchor

Fields: `meta`, `title`, `presence`, `provenance`, `span: TemporalSpan`, `rigidity`, `reported_certainty`, `occupancy=busy|transparent|unknown`, and optional `location` semantic tag. Imported recurrence's occurrence key lives in provenance. Local significance/confirmation annotations are separate.

Occupancy is an explicit source or user assertion. Busy blocks opportunity. Transparent conveys the occurrence but does not block it. Unknown occupancy and tentative busy/unknown Anchors block conservatively, with explanations identifying the assumption. Tentativeness must not silently make the user free. All-day Anchors follow the same rule; they are not necessarily busy simply because they are all-day.

For a present Anchor with resolved `[s,e)`, phase is `upcoming` if `now < s`, `ongoing` if `s <= now < e`, and `passed` if `now >= e`. Removed Anchors are inactive. There is no automatically inferred attendance/completion or overdue state. Overlapping Anchors remain separate facts; their blocking union is used for capacity and their conflict is reported separately. A local edit may change time; imported time is source-owned and read-only in the initial product.

### Deadline

Fields: `meta`, `title`, `presence`, `provenance`, `cutoff`, and `fulfillment`; user work/significance/confirmation annotations are separate. Rigidity is fixed by type.

`cutoff` is either:

- `at(instant, original_zone?)`: an exact inclusive endpoint. Phase is `upcoming` for `now < instant`, `due_now` at equality, and `overdue` for `now > instant` if unresolved; or
- `on_date(date, zone)`: a real due date known to allow that whole civil date. Phase is `upcoming` before that date, `due_today` during it, and `overdue` starting exactly at the next date's boundary if unresolved. Its calculation endpoint is that exclusive next-date boundary. Do not display a fabricated 23:59 deadline.

An unknown-time source string is not necessarily an inclusive whole-date deadline. The adapter must establish that meaning before using `on_date`; otherwise it stays unresolved. All-day semantics do not apply via a second boolean on Deadline.

`fulfillment` is one of:

- `recorded`: `unresolved`, `satisfied(evidence)`, or `cancelled(evidence)`. Evidence includes an explicit source/user assertion and recorded time; optional effective time is retained only if actually known. Local deadlines take local assertions; imported deadlines take their owning source's assertions. `unresolved` means no authoritative satisfaction/cancellation has been recorded, not proof the user failed.
- `trace_task(TaskRefId)`: only for that Trace task's normalized due projection. Done satisfies its own due Deadline; known non-Done states leave it unresolved. Unknown task status makes fulfillment unknown, not overdue/fulfilled with certainty.

Satisfied, cancelled, or removed Deadline records are not overdue and require no remaining-work recommendation. Output resolution is `unresolved|satisfied|cancelled|unknown`; output phase is `inactive` for satisfied/cancelled/removed records, `unknown` for unknown fulfillment, and otherwise follows the cutoff rules above. Preserve the original cutoff and evidence after resolution. Removal is reported separately as presence, never as satisfaction/cancellation. A recorded link to work does not by itself satisfy the deadline.

`DeadlineWork` is a local annotation: `unspecified`, `standalone(WorkMetadata)`, or `task(TaskRefId)`. Standalone means deadline-associated local work, not a new general-purpose task manager. Task uses that task's one WorkMetadata record; it cannot carry a second effort estimate here. A Trace due projection must reference its own task as work and use `trace_task` fulfillment. Other work links do not change recorded fulfillment when the task completes.

In proof-v1 a Task Reference may supply work to at most one present Deadline with unresolved or unknown fulfillment; multi-deadline work graphs require a later explicit contract. Conflicting links fail validation rather than double-counting effort. An independent Deadline whose linked task is Done has zero known remaining linked work but remains unresolved until its own evidence arrives. A removed/unknown task makes linked work unknown; it does not delete or satisfy an independently sourced Deadline.

### Trace Task Reference

Fields: `meta`, source `title`, `presence`, imported Trace `provenance`, `status=now|later|someday|done|unknown`, `due`, and optional `raw_input`, `unknown_status_label`, `source_priority`, `source_context`, `source_sort_order`, `source_link`, and `source_completed_at`. Priority/context/link/raw input are preserved opaque strings, not new ranking enums; sort order is a signed 64-bit integer. Unknown status requires a nonempty original label; other statuses have no unknown label. Source created/updated timestamps live only in provenance, and source_completed_at is an Instant if actually known.

`due` is `none`, `unresolved(value, reason)`, or `deadline(DeadlineId)`. Unresolved value/reason are nonempty strings. The resolved reference must point to this task's Trace due projection, with a reciprocal task link. External due edits update that projection; authoritative due removal retires it and changes due to none. Task removal retires its due projection but may retain the old reference inside the removed tombstone. A present projection cannot depend on a removed task. Failure to refresh does none of these. Unknown due meaning must remain inspectable and cannot generate overdue state.

Known Now, Later, and Someday are all unfinished task states in this proof. They are preserved distinctly but do not imply a time commitment, priority multiplier, eligibility time, or automatic action today. Final editorial use of those states is later work. Done is completed; no inferred effort, suggestion response, or local annotation can set it. A missing completed timestamp on a Done task stays unknown. A completed timestamp on a known unfinished task is inconsistent input and must be rejected for normalization review.

Task annotations hold one WorkMetadata and optional local importance. Trace owns text, status, priority, context, due value, and completion. Temporal Engine cannot edit or complete these fields in the initial integration. Presence removed excludes the task from candidates and retains its annotations. No real Trace transport/schema is assumed by Gate 1.

### Intention

Fields: `meta`, `title`, `presence`, local user `provenance`, optional `preferred_span: TemporalSpan`, optional `work: WorkMetadata` (absence means unknown effort), and `state=active|done|dismissed`, with `state_evidence` required for done/dismissed and absent for active. Evidence has recorded time and optional known effective time. Rigidity is fuzzy by type.

The span is a preference, not an availability declaration, reservation, or due cutoff. After its end, an active intention is `preference_passed` and remains soft eligible work; it is not overdue or automatically rewritten to today. An intention without a preferred span is simply active. Done, dismissed, or removed intentions are not candidates. A later decision to make a real obligation creates a separately identified Anchor/Deadline with explicit provenance; inference cannot silently convert the intention.

The exact Intention phase values are `inactive` when removed/done/dismissed, `no_preference` when active without a span, `preference_upcoming` before its span, `preferred_now` within it, and `preference_passed` at or after its end.

### Routine

Fields: `meta`, `title`, `presence`, local user `provenance`, `state=active|paused`, `work: WorkMetadata`, and one supported `rule`: `weekly { weekdays, start_date, until_date_exclusive?, zone }`. Weekdays serialize as `mon` through `sun`; every matching date in the nonempty weekday set and bounded rule range is a preferred occurrence date. No minute-of-day time is implied.

An occurrence key is `(RoutineId, LocalDate)` in the rule's timezone. It has the soft DateSpan for that date and inherits the routine's work metadata. Expand only within the evaluation's finite range (include the local date containing now). Paused/removed rules produce no candidates. Invalid bounds or unknown rules fail validation; no infinite expansion or generic RFC recurrence engine is needed.

Explicit user outcome records keyed by occurrence are `done` or `skipped`, with recorded time. An elapsed occurrence with no outcome is `past_unrecorded`, never automatically skipped, failed, overdue, or accumulated as debt. Only current/future eligible occurrences can be considered. Editing a rule recomputes future occurrences and retains logged outcomes by key; old outcomes do not create occurrences no longer selected by the rule. Full recurrence editing UI and import expansion are later work.

An occurrence-state helper accepts an explicit `(RoutineId, date)` as well as generated dates: inactive if the rule is paused/removed or no longer selects that date; otherwise done/skipped if recorded, else future/current/past_unrecorded relative to its civil-day span. Normal evaluation emits only occurrences intersecting its range; checking a historical occurrence or an expired suggestion's target must not add that occurrence back to the candidate list.

Imported recurring Anchors arrive as bounded, normalized individual occurrences with stable source identity. Gate 1 does not parse RRULE/ICS, extrapolate past source coverage, or pretend a recurring fixed appointment is a Routine. Unsupported source recurrence remains an adapter normalization problem.

## 6. Source state and mutability

A source catalog records stable ID, kind (`local`, `trace`, `quercus`, `google`, `outlook`), and user-visible synthetic/local label. These are finite source kinds, not a plugin interface. Multiple kinds can be represented in synthetic inputs without implementing any adapter.

For each external source, the input has `last_attempt_at?`, `last_attempt_outcome=never|complete|partial|failed|incompatible`, `last_success_at?`, and a positive `fresh_for_ms`. Coverage of the last complete success is explicit: optional Anchor interval, optional Deadline interval, and a Task catalog completeness flag. Coverage says what was authoritatively queried, not that the source knows all of the user's life.

The injected evaluator derives health in this order:

1. Latest outcome incompatible -> `incompatible`.
2. Latest outcome failed -> `unavailable`.
3. Latest outcome partial -> `partial`.
4. No complete success -> `never_loaded`.
5. `now > last_success_at + fresh_for_ms` -> `stale` (equal is still fresh).
6. Otherwise -> `healthy`.

Last complete coverage and records survive failures/partial attempts. `last_success_at <= last_attempt_at <= now` where present; never has neither timestamp nor coverage, and complete requires equal attempt/success times. A first failed/partial attempt may have no success/coverage. Local input is healthy without synthetic network refresh timestamps.

Evaluation explicitly lists required sources and roles (`anchors`, `tasks`, `deadlines`). It also always includes the source of the assessed Deadline, its linked Task, and every Anchor considered for blocking, even if the caller omitted them from that list. A required external source can have zero imported records and still be never-loaded/stale. Every external catalog entry needs a SourceState; represent an unqueried source as never, not an absent state.

For a non-overdue capacity assessment, required Anchor sources must be healthy and cover `[now, calculation_endpoint)`; Task sources need healthy complete Task catalog coverage; Deadline sources need healthy coverage containing the deadline's original calculation endpoint. Deadline coverage is `[start,end)` and therefore needs `start <= endpoint < end`. On-date uses the resolved next-date boundary for this test. Overdue/inactive assessments check the target's source health and its endpoint coverage (and Task catalog when linked), but require no future Anchor coverage because they compute no capacity. Missing coverage/unhealthy required sources make results conditional even if known records happen to be empty. Unrelated sources not explicitly required do not qualify unrelated calculations. Source problems qualify unknown results too; they cannot hide behind missing effort.

A Trace `task_due` projection inherits the complete Task catalog coverage of its owning task; it does not additionally require Deadline interval coverage. Source health still applies. Other imported Deadlines require their own source's Deadline coverage, even when their work references a Trace task.

Only explicit source deletion or omission within a successful complete authoritative reconciliation of that same scope can retire records. Omission in a failed, partial, filtered, or out-of-range result is not deletion. A Task due projection follows authoritative task/due removal; independent linked deadlines retain their own identity/state. Reconciliation transport/persistence is Gate 2+; Gate 1 tests the normalized retained/removed states.

Initial mutation rules:

| Data | Allowed writer | Evaluation effect |
| --- | --- | --- |
| Local Anchor/Deadline facts and local resolution | Explicit user command | New revision and recomputation |
| Imported facts, source completion/removal | Owning source adapter only | New snapshot; retain local annotations |
| Work/significance/confirmation annotations | Explicit user command | Recompute; never rewrite source fields |
| Intention/Routine states and outcomes, availability | Explicit user command | Recompute soft candidates/opportunity |
| Window, pressure, phase, explanation | Evaluator only | Derived output, never factual overwrite |
| Suggestion | Inference producer | Advisory output; acting on it supplies no completion proof |

No writes to Trace's SQLite database or task state are part of this contract. User confirmation of an imported item is neither a writeback nor imported completion evidence.

## 7. Windows and primitive opportunity

### Explicit availability input

An `AvailabilityDeclaration` is stored user state: ID/audit metadata, a bounded TimedSpan, `contexts=unknown|known(set of tags)`, and `energy_capacity=unknown|light|normal|deep`. It asserts willingness to consider work in that interval, not guaranteed free time. It is not a new temporal species or an automatic calendar schedule.

Input availability spans must be disjoint (touching is allowed). Overlapping declarations are invalid in proof-v1; the core must not double-count them or invent how conflicting contexts combine. Adjacent declarations retain their boundary, so a single useful chunk must fit within one declaration. No declarations means unknown opportunity, not an empty obligation-free day. Nonempty declarations form the complete declared work envelope for this evaluation: outside them contributes zero declared capacity, without claiming the user is actually unavailable.

### Window derivation

Clip declarations to `[now, evaluation_end)`. Resolve and intersect all present blocking Anchors, union their overlaps, and subtract that union once. Keep positive maximal remainders within each declaration. Transparent or removed Anchors do not subtract. Retain declaration context/energy and source-health qualifications.

`Window` contains `key=(EvaluationKey, AvailabilityId, start, end)`, span, declaration reference, sorted blocking-Anchor references used in that declaration, declared context/energy, source qualifications, and ordered reasons. It is derived state. Anchor conflicts are positive intersections of two present blocking Anchors within the evaluation range, reported as sorted ID pairs with the intersection span. Transparent overlaps are not scheduling conflicts. Conflict reporting is separate from the blocking union.

No gap inferred merely from an empty calendar is a Window in this proof. A Window is potential opportunity conditional on declared availability and known source coverage, not proof of actual free time.

### Work fit and usable capacity

For a work target, clip each Window by `earliest_start` (if any) and the calculation endpoint (deadline or evaluation end). Routine occurrence fit is also clipped to its own preferred civil date; expired occurrences are not fit candidates. Intention preferences are soft and do not impose hard clipping. Require a positive duration at least `minimum_chunk_minutes * 60,000`, all required context tags, and compatible energy. Then sum full durations of qualifying spans as usable milliseconds. Do not bridge short gaps or split a chunk across declarations. Windows ending at a deadline are usable up to, but excluding, that endpoint.

Before testing declarations or fit uncertainty, if `max(now, earliest_start if present) >= calculation_endpoint`, capacity is known zero. An exact deadline with now at its cutoff, or an earliest useful start after the cutoff, leaves no permissible interval regardless of unknown availability/chunk/context. This mathematical zero takes precedence over missing-input uncertainty.

Evaluate each span as `fits`, `does_not_fit`, or `unknown`. A definite temporal/context/energy mismatch excludes it even if another input is unknown. Otherwise missing chunk or required unknown context/energy makes its contribution unknown; do not count it as fitting or as proven zero. Unknown effort alone does not prevent a known minimum chunk fitting, but prevents a pressure ratio. Zero estimated work is not a work-fit candidate.

If any otherwise potentially qualifying span has unknown fit, usable capacity is unknown; expose known qualifying milliseconds separately, without using that subtotal as the full denominator. If all are definite, capacity is known, including zero. Missing all availability declarations is unknown capacity. External source problems qualify a known-input calculation as conditional; they do not erase last-known blockers or create capacity.

Fit is a compatibility assessment, not a reservation. Multiple work items can fit the same Window. Never infer that all individually fitting work can jointly be completed. proof-v1 reports individual risk and shared-opportunity limitations; allocating capacity or producing a rigid timetable is outside its scope.

The complete fit target set is: every present known unfinished Task Reference with positive/unknown effective work; every present unresolved Deadline with standalone positive/unknown work; every present active Intention with positive/unknown work; and every generated current/future unrecorded occurrence of an active present Routine with positive/unknown work. Done/dismissed/skipped/removed/unknown-status targets and zero estimates are excluded. An unspecified Deadline work annotation is assessed as unknown by pressure, but does not invent a fit target. Linked Deadline work is represented only by its Task target.

Emit one fit row for every target × derived Window pair, including does_not_fit rows for empty clipping; no Windows means no fit rows. A Task's endpoint is the cutoff of its sole present unresolved/unknown linked Deadline, if any, otherwise evaluation_end. A standalone Deadline target uses its own cutoff, even if overdue (so it has no before-cutoff fit). Intentions use evaluation_end; Routine occurrences use the earlier of evaluation_end and the occurrence's end. Unknown due strings supply no cutoff and must remain visibly unresolved, not classified as undated proof. Fit rows with empty permissible intersection have no span and are does_not_fit regardless of missing chunk/context. When a deadline lies beyond evaluation_end, Window clipping remains bounded by evaluation_end; fitting within that known portion does not resolve the unknown full pressure assessment.

## 8. Pressure contract: proof-v1

### Scope and arithmetic

Produce one `PressureResult` for each present Deadline, ordered by calculation endpoint then ID. No deadline-based pressure ratio exists for an undated Task, Intention, Routine, or Suggestion. An association, suggested day, importance, or source task status cannot manufacture a cutoff.

Resolve fulfillment first. A Trace due projection gets satisfaction from its own task; independently recorded deadlines require their own evidence. For unresolved independent deadlines linked to Done tasks, the workload estimate is zero and a confirmation reason is required. Preserve any original user estimate; this effective zero is derived from authoritative task completion.

For unresolved, in-range, non-overdue deadlines, let `E` be known remaining estimated/lower-bound minutes converted to milliseconds and `O` known usable milliseconds from Section 7. Use exact rational pairs `(E, O)` with integer cross-multiplication for comparisons, not floating-point sorting. No division by zero or infinity/NaN in output. Overflow is a typed evaluation error.

| Condition, evaluated in this order | Risk | Ratio |
| --- | --- | --- |
| Satisfied or cancelled | `not_applicable` | absent |
| Unknown fulfillment | `unknown` | absent |
| Unresolved and overdue | `overdue` | absent; elapsed time is the reason, not infinite pressure |
| Calculation endpoint beyond evaluation_end | `unknown` | absent; `outside_evaluation_range` |
| Unknown effort | `unknown` | absent |
| E = 0 | `no_known_work` | absent; resolution still required |
| Unknown opportunity | `unknown` | absent |
| E > 0 and O = 0 | `insufficient` | absent; explicit zero opportunity |
| 0 < E/O < 1/2 | `room` | present |
| 1/2 <= E/O <= 1 | `tight` | present |
| E/O > 1 | `insufficient` | present |

An exact timed deadline at equality is due_now, not overdue; positive remaining work and no remaining opportunity yields insufficient. On-date deadlines are overdue at the next date's boundary. `room` means room relative to the declared inputs for this individual item; it is never a promise of success or of joint feasibility.

An `at_least` estimate gives a lower-bound ratio/risk, explicitly labelled as such. Room/tight on a lower bound cannot certify safety. Stale, partial, unavailable, incompatible, uncovered, or unknown required source state makes results conditional, including apparent overdue/satisfaction according to last-known state. Keep arithmetic and data-quality qualifiers separate; bad health must not fabricate a cutoff or completion transition.

For identical positive work and constraints, reducing known usable opportunity cannot lower the ratio/risk. Completing authoritative work can reduce it; the clock alone cannot reduce stored effort. Greater importance or Trace priority does not change E/O in proof-v1. Dependency models, competition allocation, learned estimates, travel, a ranked Today edit, and pressure-zone placement are deferred.

### Result shape

`PressureResult` contains:

- `evaluation_key`, `deadline_id`, `work_target?`, and calculation endpoint with original cutoff precision;
- resolution, presence, and temporal phase as defined in Section 5, retaining their evidence;
- workload `unknown|estimate|at_least`, effective minutes if known, and the referenced user/source basis;
- opportunity `known(milliseconds)|unknown`, known qualifying subtotal, and contributing Window keys;
- optional ratio `{work_ms, opportunity_ms}` with strictly positive denominator, using unreduced original E/O values;
- `risk` from the table; workload's estimate/at_least tag preserves estimate versus lower-bound meaning;
- `qualification=declared_inputs|conditional`, with affected source IDs, coverage/freshness details, and all limitations;
- ordered `reasons` and `limitations`, including `individual_capacity_only` on every quantified assessment.

The short-circuit table also fixes numeric-field presence: not_applicable, fulfillment-unknown, overdue, and outside-range rows omit workload/opportunity/ratio; effort-unknown includes workload=unknown and omits opportunity/ratio; zero-work includes workload=estimate(0) and omits opportunity/ratio; unknown-opportunity includes known workload and unknown opportunity with its known subtotal/Window keys, but no ratio; all remaining rows include workload and known opportunity with its subtotal/Window keys, and include ratio only for O > 0. Always compute source qualifications independently of that short circuit. Do not include opportunistic extra calculations in early-return rows. No aggregate free/caught-up verdict is emitted.

## 9. Explanations and Suggestions

### Reasons

A `Reason` is structured data: stable `code`, sorted typed record/source/window references, and a payload specific to that code (numbers, enums, instants/spans, no arbitrary prose as the assertion source). User-facing text is rendered from these fields and is not used for arithmetic or test equality.

The initial codes and payloads are:

| Code family | Required payload |
| --- | --- |
| `deadline_phase` | `{cutoff, endpoint, phase, now}` |
| `resolution_recorded` | `{resolution, evidence?}`; evidence only for satisfied/cancelled |
| `task_completion_basis` | `{task_status, source_completed_at?}` |
| `declared_availability` | `{span, clipped_span?}`; no clipped_span if intersection empty |
| `anchor_blocked` | `{anchor_span, intersection, blocked_ms}`; per-anchor intersection, not additive across overlapping blockers |
| `anchor_conflict` | `{intersection}` |
| `conservative_anchor` | `{reported_certainty, occupancy}` |
| `before_earliest_start` | `{earliest_start, window_span}` |
| `chunk_too_short` | `{available_ms, chunk_ms}` |
| `context_mismatch`, `context_unknown` | `{required_contexts, declared_contexts}` using known/unknown declaration type |
| `energy_mismatch`, `energy_unknown` | `{energy_requirement, energy_capacity}` |
| `effort_unknown`, `chunk_unknown`, `availability_unknown`, `work_unavailable`, `fulfillment_unknown` | `{field}` naming the unavailable input; target is in references |
| `zero_opportunity` | `{now, endpoint, earliest_start?}` |
| `zero_work_unresolved` | `{basis}` with basis user_estimate/task_completion |
| `outside_evaluation_range` | `{endpoint, evaluation_end}` |
| `pressure_ratio` | `{work_ms, opportunity_ms, risk}` |
| `effort_lower_bound` | `{minutes}` |
| `source_health` | `{health, last_attempt_outcome, last_attempt_at?, last_success_at?}` |
| `source_coverage` | `{role, required_interval?, required_endpoint?, known_interval?, tasks_complete?}`; anchors uses required_interval/known_interval, deadlines uses required_endpoint/known_interval, tasks uses tasks_complete |
| `individual_capacity_only` | Empty payload; Deadline/work references carry scope |
| `soft_preference_passed`, `routine_past_unrecorded`, `suggestion_expired` | `{boundary, now}` |
| `suggestion_invalidated` | `{previous_basis, current_basis}` (EvaluationKeys excluding now) |

Payloads are typed per code family; a code with missing/wrong payload is invalid. Each reason is exactly `{code, references, payload}`. References are tagged source/object IDs, a Window key, a Routine occurrence key, or `pressure` with `(EvaluationKey, DeadlineId)`. They must resolve in the evaluated snapshot or to its derived outputs; prior-suggestion validity may reference its retained producing key as historical evidence. Deduplicate equal reasons and sort by code, then reference tuple, then canonical payload. Every pressure result explains which rule row applies, where E and O came from (or why unavailable), and all qualifications. Reproducing a result must require no hidden AI reasoning or host context.

### Suggestion

A Suggestion contains `key=(EvaluationKey, kind, target, proposed_span?)`, `kind=consider_work|risk_notice`, a typed target, optional advisory `proposed_span`, `created_at`, `valid_until`, and nonempty structured reasons/input references. A consider_work target is a known unfinished present Task Reference, a present unresolved Deadline with standalone positive/unknown work, an active present Intention, or a current/future unrecorded occurrence of an active present Routine. Zero estimated work is ineligible. Deadline work linked to a Task uses that Task target, avoiding a duplicate work identity. A risk_notice targets a present unresolved/unknown Deadline and requires a pressure result as a reason input. It is an inferred output, with no completion or overdue field.

`created_at` equals the producing evaluation's now and must precede `valid_until`. If a span is proposed, it must end after creation and `valid_until` cannot exceed its end. A current suggestion requires the same snapshot/settings/timezone-policy basis, a still-eligible target, and `now < valid_until`. Compare the basis excluding now when checking a later clock tick; an input revision change invalidates it. At equality or later it is expired. Recompute work assessments against the current time before reusing it.

After basis/expiry/target checks, revalidation has an explicit outcome. For consider_work, recompute fit within the intersection of current Windows, target bounds, and the optional proposed span; at least one definite fits result is required, otherwise validity is ineligible with current missing-input/mismatch reasons. Conditional source health may still accompany a fits result, but must be included in validity reasons so it is never presented as guaranteed free time. For risk_notice, recompute its PressureResult: tight/insufficient/overdue/unknown remains current with the new result's reasons; room/no_known_work/not_applicable is ineligible. The historical Suggestion/reasons are immutable; the validity row holds current reasons. These checks update no factual or user state.

Expiration/invalidation creates no obligation and writes no task/deadline state. The underlying still-active work can be considered again; an old suggestion is not carried forward as debt. Accepting a suggestion is not proof of completion or a silent conversion to an Anchor. Suggestion generation/editorial selection is deferred beyond Gate 1; Gate 1 supports the typed shape and deterministic validity/rollover checks using synthetic suggestions.

## 10. Validation and serialization boundary

The versioned synthetic fixture envelope has these exact top-level fields. Collections are required even when empty; optional fields inside records are omitted when unknown/absent. Imported/native fields and user annotations serialize separately. This is for local fixtures and internal exchange, not a frozen database or external adapter format.

| Field | Shape |
| --- | --- |
| `schema_version`, `snapshot_revision`, `captured_now` | Literal 1, positive integer, Instant |
| `sources` | Array of `{id: SourceId, kind, label}` |
| `source_states` | Array of `{source_id, last_attempt_outcome, fresh_for_ms, last_attempt_at?, last_success_at?, anchor_coverage?, deadline_coverage?, tasks_complete}` for external sources; tasks_complete is false without complete catalog evidence |
| `required_sources` | Array of `{source_id, role}` with role anchors/tasks/deadlines |
| `anchors`, `deadlines`, `task_refs`, `intentions`, `routines` | Arrays of the field inventories in Section 5 |
| `anchor_annotations` | Array of `{target_id, revision, created_at, updated_at, importance, milestone, confirmation?}` |
| `deadline_annotations` | Array of `{target_id, revision, created_at, updated_at, importance, milestone, work: DeadlineWork, confirmation?}` |
| `task_annotations` | Array of `{target_id, revision, created_at, updated_at, importance, work: WorkMetadata}` |
| `routine_outcomes` | Array of `{routine_id, date, outcome, revision, recorded_at}` with outcome done/skipped |
| `availability` | Array of `{meta, span: TimedSpan, contexts, energy_capacity}` |
| `evaluation` | `{now, evaluation_end, display_zone, timezone_rules_version, policy_version}`; policy_version is proof-v1 |
| `prior_suggestions` | Array of separately classified inferred outputs for validity checks; never stored factual objects |

An absent annotation means the documented default significance/confirmation and unknown work, except a Trace due projection whose mandatory work/fulfillment reference is intrinsic. At most one current annotation per target per annotation collection is allowed. Explicit deadline work annotation on a Trace due projection, if present, must agree with that intrinsic task link. Annotation audit times obey the common injected-time rules. Outcome keys are unique by routine/date; source requirements are unique by source/role.

In JSON, all-unit enums are strings. Enums mixing unit/payload cases use `{kind: tag}` for unit cases and `{kind: tag, value: payload}` for payload cases. Single primitive/reference payloads use that value directly; multi-field payloads use an object with the named fields. For example, effort is `{"kind":"estimate","value":60}`, due is `{"kind":"deadline","value":"00000000-0000-4000-8000-000000000001"}`, and cutoff is `{"kind":"on_date","value":{"date":"2026-09-07","zone":"America/Toronto"}}`. Weekly rule, imported provenance, and spans use the same tagged convention. Identity newtypes serialize as their UUID string; typed target/reference unions use tags, not title lookup.

Provenance payload names are `source_id, asserted_at` for local_user and `source_id, external_id, occurrence_key?, projection, observed_at, source_revision?, source_created_at?, source_updated_at?` for imported. Timed-span payload is `start, end, original_zone?`; DateSpan payload is `start_date, end_date_exclusive, zone`. At-cutoff payload is `instant, original_zone?`. Recorded fulfillment wraps the unresolved/satisfied/cancelled variant; satisfied/cancelled carry `{authority, recorded_at, effective_at?}`, with authority local_user/source constrained by ownership. Confirmation is `{confirmed_at, record_revision}`. All local evidence/outcome/confirmation recorded times are at most captured_now. Effort/chunk minutes use unsigned 32-bit integers; durations/revisions use unsigned 64-bit integers; epoch milliseconds use signed 64-bit integers; ratio cross-products use checked unsigned 128-bit arithmetic.

Evaluation exposes typed states/occurrences, Windows/conflicts/fit results, PressureResults, and prior-suggestion validity separately. A fit result is `{target, window_key, span?, status, reasons}`; span is the positive clipped assessment span when one exists, and status fits/does_not_fit/unknown. Suggestion validity is `{key, status, reasons}` with status current/expired/invalidated/ineligible. Resolve validity in order: changed non-clock basis -> invalidated; passed valid_until -> expired; no longer eligible -> ineligible; otherwise current (subject to current-time fit/pressure revalidation). State rows identify the record/occurrence, its phase/state, and ordered reasons. No renderer, recommendation generator, or aggregate success verdict is part of this output.

The evaluation output envelope is `{evaluation_key, source_health, anchor_states, deadline_states, intention_states, routine_occurrences, windows, conflicts, fits, pressures, suggestion_validity}`. All fields are arrays except evaluation_key. Source health rows are `{source_id, health, reasons}`; Anchor/Intention rows are `{id, phase, reasons}`; Deadline rows are `{id, presence, resolution, phase, cutoff, endpoint, reasons}`. Routine rows are `{key, span, phase, reasons}`, using the explicit occurrence helper's phase rules. Conflicts are `{anchor_ids, intersection, reasons}` with exactly two sorted IDs. Windows/pressures use Sections 7/8; PressureResult field names are `evaluation_key, deadline_id, work_target?, cutoff, endpoint, resolution, presence, phase, workload?, opportunity?, ratio?, risk, qualification, source_qualifications, reasons, limitations`. Opportunity is a known/unknown tagged payload with `known_qualifying_ms, window_keys` and, only when known, `milliseconds`. A source qualification is `{source_id, role, health, covered, reasons}`. Workload uses the effort tagged representation. Limitations is the code set, containing individual_capacity_only whenever opportunity was calculated. All result arrays have a deterministic complete inventory; no top-N truncation is performed in Gate 1.

Canonical JSON uses UTF-8, snake_case fields/enums, the tagged convention above, integer numbers, canonical time/UUID forms, lexicographically sorted object keys, no insignificant whitespace, and no trailing newline in canonical bytes. Absent optional fields are omitted, never null. Arrays representing sets sort by their canonical value; record collections sort by ID, annotations by target_id, outcomes by routine_id/date, source requirements by source ID then role, routine weekdays Monday through Sunday, PressureResults by endpoint/ID, and other output rows by their structured key or typed target then Window key. Input record order must not affect output; duplicate IDs/set entries are errors. Test pretty-printing is allowed, but equality uses canonical bytes/typed results, never object-map iteration order.

String ordering is by Unicode scalar values, without locale/case folding or Unicode normalization. Encode non-ASCII text directly as UTF-8, escape quote/backslash, use the JSON short escapes for backspace/tab/newline/form-feed/carriage-return, and lowercase `\u00xx` for other control characters. Do not escape ordinary slashes or printable characters. Object keys in this schema are ASCII. Integer output is base ten without leading plus/zeros or exponent notation.

Unknown schema versions, normalized enum tags/fields, duplicate JSON keys, invalid IDs, non-finite/fractional integer values, malformed/missing times, unresolved timezones, invalid intervals, arithmetic overflow, unresolved references, cross-source identity collisions, wrong species links, conflicting work owners, impossible completion combinations, and source-state inconsistencies are errors. The normalized fixture parser is strict; a future external adapter may retain unknown source values explicitly (for example unresolved due or unknown task status) before normalization.

Validation returns ordered issues `{code, path, record_id?, related_ids}`. Code categories are `unsupported_version`, `unknown_field`, `invalid_value`, `invalid_time`, `invalid_identity`, `duplicate_identity`, `missing_reference`, `invalid_relationship`, `inconsistent_state`, and `arithmetic_overflow`; order by path then code then IDs. No evaluation succeeds on a partially accepted invalid snapshot. A future adapter quarantines rejected source material and reports partial/incompatible health instead of dropping it and asserting new free time.

### Invariants to test

1. IDs/provenance survive ordinary source edits; no text matching or cross-source merging.
2. Stored facts, user state, derived state, and suggestions cannot deserialize into each other's variants.
3. Only authoritative state resolves a Deadline or completes a Trace task; due passage/zero effort/expired suggestions cannot.
4. Milestone neither changes species nor duplicates workload; linked effort has one owner.
5. Every time boundary is explicit, timezone-stable, and evaluated from injected now.
6. Anchor overlap is subtracted once; source failure and missing declarations never create asserted free time.
7. Unknown/lower-bound effort and conditional source coverage remain visible in pressure and reasons.
8. Soft intentions and routine preferences remain soft after passage; completed/removed targets stay excluded.
9. Identical snapshots, settings, rules, and now produce identical canonical outputs, including reasons and ties.
10. The core needs no UI, database, adapters, AI, telemetry, network, or agent runtime to prove these rules.
