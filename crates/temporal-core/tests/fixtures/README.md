# Temporal core fixture coverage

This directory records the executable fixture inventory for Task 1.4. The
fixture constructors are in `../support/bases.rs` and
`../support/variants.rs`; they contain synthetic values only and never read a
personal database, source export, clock, or network.

`fixture_contract::every_documented_base_and_named_input_variant_validates_and_round_trips`
is the coverage manifest's executable check. It currently reports **81 valid
inputs**: the 16 scenario bases below plus 65 named deterministic mutations.
`validation_contract::all_rejection_fixtures_fail_whole_input_with_the_documented_category`
reports **51 independent rejection fixtures**, including the S16 matrix and
strict JSON/schema cases.

## Valid inputs

| scenario | base | named variants |
| --- | --- | --- |
| S01 | `S01/base` | — |
| S02 | `S02/base` | — |
| S03 | `S03/base` | `S03/wednesday`, `S03/thursday` |
| S04 | `S04/base` | — |
| S05 | `S05/base` | `S05/transparent` |
| S06 | `S06/base` | `S06/chunk_90` |
| S07 | `S07/base` | `S07/declared_evening` |
| S08 | `S08/base` | `S08/source_move`, `S08/same_text_distinct_identity` |
| S09 | `S09/base` | `S09/satisfied`, `S09/zero_unresolved` |
| S10 | `S10/base` | `S10/expiry_equality`, `S10/tuesday`, `S10/trace_done` |
| S11 | `S11/base` | `S11/date_still_today`, `S11/utc_display`, `S11/date_overdue_equality`, `S11/exact_base`, `S11/exact_equality`, `S11/exact_no_availability`, `S11/exact_plus_ms`, `S11/spring`, `S11/spring_transparent`, `S11/retired_anchor` |
| S12 | `S12/base` | `S12/failed`, `S12/partial`, `S12/incompatible`, `S12/freshness_equality`, `S12/freshness_plus_ms`, `S12/short_coverage`, `S12/never_loaded_empty`, `S12/successful_empty` |
| S13 | `S13/base` | `S13/stale_trace`, `S13/stale_trace_implicit_dependency`, `S13/unknown_own_due`, `S13/unknown_independent_work`, `S13/authoritative_removal`, `S13/quercus_satisfied`, `S13/local_confirmation_only` |
| S14 | `S14/base` | `S14/monday`, `S14/skipped`, `S14/edited_rule`, `S14/paused`, `S14/second_routine` |
| S15 | `S15/base` | `S15/known_context`, `S15/mismatch_with_unknown_energy`, `S15/no_availability`, `S15/earliest_at_cutoff`, `S15/earliest_after_cutoff`, `S15/exact_equality`, `S15/overdue_plus_ms` |
| S16 | `S16/base` | `S16/half_hour_later`, `S16/effort_59`, `S16/effort_60`, `S16/effort_120`, `S16/effort_121`, `S16/unknown_effort`, `S16/zero_effort`, `S16/lower_bound`, `S16/importance_only`, `S16/priority_only`, `S16/new_snapshot_basis`, `S16/new_settings_basis`, `S16/current_basis_room_notice`, `S16/range_equality`, `S16/range_plus_ms`, `S16/unresolved_due` |

The S16 effort names are generated from the explicit set `59, 60, 120, 121`
in `variants.rs`; they are not a wildcard fixture. `all_cases()` concatenates
the bases and mutations in declaration order, while canonical encoding tests
also permute collections to prove order-independent bytes.

## Implemented fixture stages

- **Implemented in Task 1.4:** strict normalized JSON decoding (including
  duplicate-key rejection), schema/version and tagged-field rejection, typed
  identity/reference/ownership/link validation, timestamp/interval/source
  consistency, checked source-expiry preflight, canonical JSON, and the
  clock-independent fulfillment resolver.
- **Implemented in Task 1.5:** Anchor, Deadline, and Intention lifecycle
  phases; authoritative fulfillment/resolution rows; finite weekly Routine
  expansion; stable occurrence keys; explicit done/skipped/paused/rule-edit
  handling; and historical `past_unrecorded` inspection without candidate
  resurrection. Focused lifecycle coverage exercises S05, S07, S09, S11, S13,
  and S14 boundaries; the remaining scenario assertions belong to their
  dependent health, opportunity, pressure, and evaluation tasks.
- **Implemented in Task 1.6:** source-health precedence, freshness equality,
  checked expiry, half-open role coverage, retained last-known records, local
  health, and automatic factual dependency inclusion for Deadline, Task, and
  blocking-Anchor sources. Trace due projections use task-catalog coverage
  without a separate Deadline interval.
- **Implemented in Task 1.7:** declared-availability clipping, one-time
  blocking-Anchor union subtraction, separate conflict reporting, stable Window
  keys, source qualifications on derived Windows, deterministic target × Window
  fit rows, compatibility/chunk/earliest/routine bounds, and known/unknown
  individual opportunity aggregation. Focused opportunity/fit coverage exercises
  S02–S07, S09, S11–S12, S14–S15, and S03–S04 endpoint/clock boundaries.
- **Implemented in Task 1.8:** proof-v1 individual pressure with authoritative
  fulfillment ordering, exact integer ratios, threshold risks, zero/unknown/
  lower-bound work handling, source qualifications, deterministic reasons, and
  explicit individual-capacity limitations. Focused pressure coverage exercises
  S03, S04, S06, S09, S12, S13, S15, and S16; the full valid-input manifest
  also derives pressure rows twice for determinism and input immutability.
- **Implemented in Task 1.9:** the complete validated evaluation and historical
  suggestion revalidation. `scenarios.rs` checks explicit semantic expectations
  from `support/expected.rs` for every one of the 81 inputs above; unlisted
  variants fail. It also checks complete inventories, referenced evidence,
  canonical typed/byte output, input permutations, and input immutability.
  `suggestion_contract` covers expiry/basis precedence, current fit/risk,
  source qualifications, and soft work eligibility. Integration pressure
  regressions cover fully blocked declarations, full-range effort conversion,
  and source coverage when effort is unknown.
- **Verified in Task 1.10:** the complete gate suite passes with 78 integration
  tests and 3 ownership doctests. Representative synthetic observations can be
  inspected with `cargo test -p temporal-core --test scenarios every_documented_scenario --locked --offline -- --nocapture`.
  These printed observations are not the expectations used by assertions.
- **Deferred product evidence:** visual Horizon usability, source-adapter
  reconciliation, and personal-local trials. All Gate 1 semantic stages and
  the final gate review are complete; the evidence report is in BUILD-GATES.

All expected rejection categories are declared beside their mutation in
`support/negative.rs`; no expected output is generated by the implementation
under test.
