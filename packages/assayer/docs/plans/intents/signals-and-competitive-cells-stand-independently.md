# Keeping Signals and Competitive Cells Independent · `plan:assayer:intent-signals-and-competitive-cells-stand-independently`

This plan keeps the promise that entity-persistent signals and competitive-cell membership remain separately available inputs to an ordinary assessment, including either input without the other (´claim:identity:signals-and-competitive-cells-stand-independently´).

## What the promise says, precisely · `sec:assayer:intent-signals-and-competitive-cells-stand-independently-promise`

For an entity $e$, let $C_e$ mean that the signal cache holds an entry, let $A_e$ be the set of active competitive cells containing the entity's coordinate, and let $S_e$ be the raw signal block used by assessment. Cache presence and cell presence are independent booleans: $C_e$ is decided by the cache's least-recently-used retention, while $A_e$ is decided by the dimension graph's published competitive set (´tab:keyspace:signal-cache´), (´def:keyspace:competitive-set´).

The witness declares one entity-persistent scalar clipped to $[0,1]$ and supplies the exactly representable value $0.75$. The signal schema therefore fixes a one-position block at construction, preserves $0.75$ unchanged, reads persistence by entity key, and zero-fills that position when the cache has no entry (´schema:signal:shapes´), (´req:signal:schema-fixed´).

None of the four quadrant assessments resupplies the signal, so $S_e$ is determined only by whether the cache holds the entity. The requests that supply $0.75$ are setup stimuli used to prime or replace the cache, not quadrant observations.

The four measurable states are exhaustive and all must complete through the same assessment call:

- $C_e=0, |A_e|=0$: the signal block is `[0.0]` and every competitive indicator is zero.
- $C_e=1, |A_e|=0$: the signal block is `[0.75]` and every competitive indicator is zero.
- $C_e=0, |A_e|>0$: the signal block is `[0.0]` and at least one competitive indicator is one.
- $C_e=1, |A_e|>0$: the signal block is `[0.75]` and at least one competitive indicator is one.

Cell presence is active membership for this request, not merely a non-empty dimension-wide set. Each active cell maps to an indicator of one and every other cell maps to zero (´def:keyspace:competitive-indicators´); the fixed-width per-dimension and cross-dimension summaries remain ordinary and zero-fill when no cell is active (´tab:keyspace:dimension-features´), (´tab:keyspace:cross-dimension-features´).

The signal-cache capacity is one, the minimum admitted capacity. No depth cutoff, split threshold, or observation count is part of the promise: those are host configuration or fixture inputs, and the witness guards the cell-present precondition instead of treating any configured threshold as proof that promotion occurred (´tab:config:identity´), (´dec:harness:guarded-fixtures´), (´dec:harness:separate-validation´).

No numerical tolerance belongs to the feature oracle. Scalar encoding preserves $0.75$, absence produces $0.0$, and membership indicators are binary. The host-facing result is checked only for well-formed finite output because learned risk is not the quantity this promise fixes.

Eviction may lose the cached signal and nothing else: it neither removes competitive history nor changes the cell set (´cav:limitation:signal-cache´). Conversely, deferred graph publication changes competitive membership without requiring or rewriting a cache entry, under the independently published competitive-set decision (´dec:memory:competitive-publication´).

## What the code offers today · `sec:assayer:intent-signals-and-competitive-cells-stand-independently-current-code`

The assessment entry point already performs the production composition in one call: it encodes the entity on every identity dimension, enqueues the identity observation, reads the currently published active cells, merges the signal cache, assembles both disjoint feature ranges, and retains the request inputs in the pending entry (´alg:runtime:assessment-pipeline´). The dimension map remains the sole resolver of both ranges (´dec:vector:sole-resolver´), and the canonical feature structure keeps the signal and competitive-indicator blocks disjoint (´def:feature:vector-structure´).

The signal cache stores encoded entity values and defers both insertions and hit touches to identity maintenance (´dec:retention:cache-preencoded´), (´test:integration:cache-write-is-deferred-until-the-drain´). An assessment reads the published cell set before its own observation is drained, so one request can establish a cell-absent row and the named maintenance barrier can publish cell state for a later row without any elapsed-time inference (´alg:keyspace:observation-protocol´), (´dec:harness:no-ad-hoc-waits´).

`scenario_with_config`, `WorldBuilder::signal_schema`, `World::register_identity`, `World::assess`, `World::derive`, and `World::assayer` provide the configured real-engine lifecycle the witness needs, while the production `RequestContext::with_signal` attaches the scalar. `assert_reckoning_well_formed` and `assert_health_clean` supply the result and degradation checks; the shared harness no longer has pending skeleton work on this path (´tab:assayer:harness-implementation-library-roster´), (´dec:harness:real-engine´), (´entry:assayer:harness-complete-builder´).

**Entry (The synchronous identity-maintenance barrier is supplied)** · `entry:assayer:intent-signals-and-competitive-cells-stand-independently-maintenance-barrier`

`World::flush_identity_maintenance` now drains earlier identity observations and deferred signal-cache writes, detects competitive-set changes, and crosses model-owner publication before returning. Its queue coverage and exclusions are declared by the completed barrier entry, so the witness needs no sleep, poll, local deadline, or additional barrier (´entry:assayer:harness-closed-barriers´), (´tab:assayer:harness-implementation-library-roster´).

**Entry (The read-only assessment-input projection is supplied)** · `entry:assayer:intent-signals-and-competitive-cells-stand-independently-input-probe`

`World::pending_entry_view` performs one non-consuming lookup and returns an owned `PendingEntryView` containing the raw retained signal block at its actual precision and the active-cell count for each dimension. `PendingEntryView::single_precision_signals` reads the configured default storage form without widening it; the focused probe test already proves the view is non-consuming and width-preserving (´dec:harness:probe-contract´), (´entry:assayer:harness-probe-contract´), (´test:crate:pending-entry-view-is-non-consuming-and-keeps-storage-width´).

The direct signal tests prove persistence after a drain and least-recently-used eviction at capacity (´test:integration:hit-returns-cached-entity-persistent´), (´test:integration:evicts-oldest-at-capacity´). The identity integration test proves registration leaves assessment and derivation well formed (´test:integration:identity-register-allows-assess-and-derive´), while the assembly tests prove raw signal placement and the one-or-zero mapping from active cells to competitive indicators (´test:crate:assembly-signal-features´), (´test:crate:competitive-indicator-active´).

The seeded assessment-write sweep now combines one persistent signal, a registered identity dimension, the real cache, and `PendingEntryView`, but it checks only the drawn signal and the dimension's presence in the count map (´test:crate:assessment-write-set-seeded-sweep´). It does not evict the signal, require positive and zero active-cell counts, or establish all four pairs, so the independence claim remains unkept.

The finished surface also exposes forward-only `World::advance` and `World::travel_to`, declarative `playback` through `PlaybackRow` and `PlaybackBarrierPolicy`, the independent `pairwise_rank`, `regularised_schur_complement`, `decay_recurrence`, and `dimension_width` oracles, the guarded `TrainedStateFixture`, and `run_seeded_sweep` (´dec:harness:declarative-playback´), (´dec:harness:oracle-tier´), (´dec:harness:guarded-fixtures´), (´dec:harness:seeded-sweeps´), (´tab:assayer:harness-implementation-library-roster´). This witness needs none of them: it moves no time, every stimulus is one assessment rather than a long sequence, its four Boolean combinations are exhaustive, its expected signal and indicator values come directly from the specification, and it requires neither learned convergence nor a population sweep.

## The witness · `sec:assayer:intent-signals-and-competitive-cells-stand-independently-witness`

The witness belongs in the `identity_layer` integration target as `signals_and_competitive_cells_stand_independently`, beside the existing public-lifecycle scenario (´test:integration:identity-register-allows-assess-and-derive´). Its test documentation cites this intent, and the target's module index states the four-state assertion rather than restating setup.

- Setup: use `scenario_with_config` to declare one default channel, one entity-persistent scalar named `trust`, and signal-cache capacity one. Before registering an identity dimension, assess `alpha` with `trust = 0.75` and call `World::flush_identity_maintenance` so the deferred cache insertion completes without enqueuing any identity observation; then call `World::register_identity`, whose new competitive set is empty.
- Read one request: call `World::assess`, immediately obtain its `PendingEntryView` by assessment identifier, call `World::derive` on that same assessment for the default channel, and require a well-formed reckoning. This keeps each raw input pair and ordinary result tied to one production assessment.
- Observe signal-only: assess `alpha` without resupplying `trust` before crossing identity maintenance again, and require `PendingEntryView::single_precision_signals` to return the cached `[0.75]` while the registered dimension's active-cell count is zero.
- Observe neither: still before the barrier, assess `charlie` without a signal and require `[0.0]` with active-cell count zero.
- Establish the guarded cell-present state: after the signal-only and neither assessments, call `World::flush_identity_maintenance`, assess `alpha`, and first require its projected active-cell count to be positive as the fixture precondition. The existing guarded identity fixture establishes that assessment observations followed by this barrier can publish a declared cell set (´test:crate:register-identity-with-cells-observes-the-declared-set´), while this local guard reports the exact count the witness will use.
- Observe both present: on that guarded `alpha` assessment, independently require the signal block to remain `[0.75]` and retain the positive active-cell count.
- Evict only the signal: assess `bravo` with `trust = 0.75`, call `World::flush_identity_maintenance`, and require the full health report to show capacity one, size one, and one eviction. The barrier drains the preceding `alpha` hit touch before the `bravo` insertion, making `alpha` the evicted entity while leaving its already published cell membership untouched.
- Observe cell-only: assess `alpha` without resupplying the signal and require `[0.0]` with a positive active-cell count.
- Complete the oracle: require the four exact pairs `([0.75], 0)`, `([0.0], 0)`, `([0.75], positive)`, and `([0.0], positive)`, well-formed reckonings for all four, equal one-position signal widths, and clean health apart from the expected cache eviction counters.

The probe reads the pending entry by the returned assessment identifier, so its signal vector and active-cell count describe the same request. The existing indicator-placement witness ensures that a positive count is carried into the raw vector as one-valued competitive indicators while an empty active set leaves all of those indicators zero (´test:crate:competitive-indicator-active´).

The fails-before is direct. Gating cache lookup on competitive membership breaks the signal-only row; gating active-cell lookup on a cache hit or coupling cache eviction to identity removal breaks the cell-only row; conflating either absence with failure breaks the corresponding ordinary result; and assembling either block from the other's presence breaks at least one exact pair.

## What is missing · `sec:assayer:intent-signals-and-competitive-cells-stand-independently-missing`

**Entry (The four-quadrant integration witness)** · `entry:assayer:intent-signals-and-competitive-cells-stand-independently-integration-witness`

The `identity_layer` integration target still needs `signals_and_competitive_cells_stand_independently`: a configured capacity-one scenario, cache priming before identity registration, the local guarded promotion precondition, four exact `PendingEntryView` readings around one maintenance publication and one cache eviction, ordinary-result and health assertions, fails-before documentation, and a module-index row. No new shared harness item is required.

## Risks and open questions · `sec:assayer:intent-signals-and-competitive-cells-stand-independently-risks`

**Observation (Promotion is observed across the queue boundary)** · `obs:assayer:intent-signals-and-competitive-cells-stand-independently-promotion-state`

An assessment records its identity observation only after reading the currently published competitive set, so the two cell-absent rows must be captured before `World::flush_identity_maintenance` and the cell-present precondition must be checked after it. The guard reports the projected count instead of inferring promotion from a split threshold, an observation total, virtual time, or elapsed time (´alg:keyspace:observation-protocol´), (´dec:harness:no-ad-hoc-waits´).

**Observation (The cold ramp cannot stand in for block inspection)** · `obs:assayer:intent-signals-and-competitive-cells-stand-independently-raw-oracle`

Standardisation can map a raw zero to a non-zero standardised value, and model weights can make distinct raw inputs produce similar risks. `PendingEntryView` therefore supplies the pre-standardisation signal block and active membership retained for the assessment; risk is checked only for a well-formed result (´dec:harness:probe-contract´).

**Observation (One maintenance barrier does not couple the stores)** · `obs:assayer:intent-signals-and-competitive-cells-stand-independently-shared-loop`

Deferred cache writes and identity observations share `World::flush_identity_maintenance`, but they retain separate queues and state. The witness distinguishes shared completion from shared preconditions by reading cell absence before publication, cell presence after publication, and the same published membership after a capacity-one cache eviction (´entry:assayer:harness-closed-barriers´).

**Observation (The landed projection is exactly narrow enough)** · `obs:assayer:intent-signals-and-competitive-cells-stand-independently-probe-scope`

`PendingEntryView` exposes immutable copies of the retained signal values and per-dimension active-cell counts, which are exactly the two inputs the promise relates. It exposes neither the cache nor graph, and it offers no mutation path, so the witness observes the production composition without widening the host API or inventing snapshot diffing (´dec:harness:probe-contract´), (´entry:assayer:harness-probe-contract´).

No maintainer decision remains open: the specification fixes zero-fill, binary membership, and cache/cell independence; the completed barrier and probe expose the necessary deterministic boundary and owned reading; only the integration witness remains.

## Acceptance · `sec:assayer:intent-signals-and-competitive-cells-stand-independently-acceptance`

The implementing report identifies the `identity_layer` integration target and `signals_and_competitive_cells_stand_independently`, confirms that its test documentation cites this intent, updates the module index, and shows the coverage report resolving the intent to that integration witness.

The report confirms that `World::flush_identity_maintenance` first primes the cache before identity registration, then separates the cell-absent readings from the guarded cell-present readings, and later drains the capacity-one eviction, while `World::pending_entry_view` obtains each raw signal block and active-cell count from the same pending assessment.

The evidence includes the exact four pairs `([0.75], 0)`, `([0.0], 0)`, `([0.75], positive)`, and `([0.0], positive)`; one cache eviction at capacity one; stable positive membership for `alpha` across that eviction; well-formed results for every quadrant; equal one-position signal widths; and clean degradation health.

The targeted integration test and the nearest signal, identity, assembly, barrier, probe, and seeded assessment-write tests pass under the package's prescribed formatting, lint, and test gates.

The report states which quadrant rejects cache lookup gated by membership, cell lookup gated by cache presence, cache eviction coupled to identity removal, a short signal block, or an exceptional absence. No executed mutation is required.
