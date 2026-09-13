# Keeping Signals and Competitive Cells Independent · `plan:assayer:intent-signals-and-competitive-cells-stand-independently`

This plan keeps the promise that entity-persistent signals and competitive-cell membership remain separately available inputs to an ordinary assessment, including either input without the other (´claim:identity:signals-and-competitive-cells-stand-independently´).

## What the promise says, precisely · `sec:assayer:intent-signals-and-competitive-cells-stand-independently-promise`

For an entity $e$, let $C_e$ mean that the signal cache holds an entry, let $A_e$ be the set of active competitive cells containing the entity's coordinate, and let $S_e$ be the raw signal block used by assessment. Cache presence and cell presence are independent booleans: $C_e$ is decided by the cache's least-recently-used retention, while $A_e$ is decided by the dimension graph's published competitive set (´tab:keyspace:signal-cache´), (´def:keyspace:competitive-set´).

The witness declares one entity-persistent scalar clipped to $[0,1]$ and supplies the exactly representable value $0.75$. The signal schema fixes a one-position block at construction, a missing value zero-fills that position, and persistence reads the encoded value by entity key (´schema:signal:shapes´), (´req:signal:schema-fixed´).

The four measurable states are exhaustive and all must complete through the same assessment call:

- $C_e=0, |A_e|=0$: the signal block is `[0.0]` and every competitive indicator is zero.
- $C_e=1, |A_e|=0$: the signal block is `[0.75]` and every competitive indicator is zero.
- $C_e=0, |A_e|>0$: the signal block is `[0.0]` and at least one competitive indicator is one.
- $C_e=1, |A_e|>0$: the signal block is `[0.75]` and at least one competitive indicator is one.

Cell presence is active membership for this request, not merely a non-empty dimension-wide set. Each active cell maps to an indicator of one and every other cell maps to zero (´def:keyspace:competitive-indicators´); the fixed-width per-dimension and cross-dimension summaries remain ordinary and zero-fill when no cell is active (´tab:keyspace:dimension-features´), (´tab:keyspace:cross-dimension-features´).

The fixture sets signal-cache capacity to one, the minimum admitted capacity, and uses a depth cutoff of three with the host-supplied split threshold set to ten. These are configuration quantities rather than response tolerances (´tab:config:identity´). One hot entity receives enough observations to cross the split threshold; a replacement entity stays below it while its cache insertion evicts the hot entity's signal.

No numerical tolerance belongs to the feature oracle. Scalar encoding preserves $0.75$, absence produces $0.0$, and membership indicators are binary. The host-facing result is checked only for well-formed finite output because learned risk is not the quantity this promise fixes.

Eviction may lose the cached signal and nothing else: it neither removes competitive history nor changes the cell set (´cav:limitation:signal-cache´). Conversely, deferred graph promotion changes competitive membership without requiring or rewriting a cache entry, under the independently published competitive-set decision (´dec:memory:competitive-publication´).

## What the code offers today · `sec:assayer:intent-signals-and-competitive-cells-stand-independently-current-code`

The assessment entry point implements the production composition in one call. It encodes the entity on each identity dimension and reads active cells, then calls the signal cache's merge path, and finally passes both collections to raw feature assembly under the assessment pipeline (´alg:runtime:assessment-pipeline´).

Feature assembly fills the signal range from the merged signal vector and fills competitive positions from the active-cell map. Both ranges are resolved through the dimension map rather than by local offset arithmetic (´dec:vector:sole-resolver´), and the two are disjoint blocks in the canonical feature structure (´def:feature:vector-structure´).

The signal cache stores pre-encoded entity values (´dec:retention:cache-preencoded´). A cache miss starts from a schema-width zero vector, a hit supplies the remembered entity positions, and a deferred insertion or recency touch is applied by the identity-maintenance loop rather than on the assessment path.

The identity-maintenance loop drains identity observations and deferred cache writes in the same pass, but each drain mutates only its own structure. The configured wake cadence bounds when the pass occurs and is not an ordering oracle for a test (´dec:memory:maintenance-cadences´).

`World` already supplies configured construction through `scenario_with_config` and `WorldBuilder::signal_schema`, custom requests through `RequestContext::with_signal`, public identity registration through `World::assayer`, assessment and derivation verbs, full health, and model-owner publication barriers. These are the completed skeleton's public-path pieces (´entry:assayer:harness-stage-skeleton´).

The nearest signal tests prove that a persistent value is returned without resupply and that capacity eviction produces a cold miss (´test:integration:hit-returns-cached-entity-persistent´), (´test:integration:evicts-oldest-at-capacity´). They drive `SignalCache` directly and never register an identity dimension.

The nearest identity integration test proves assessment remains well formed after dimension registration (´test:integration:identity-register-allows-assess-and-derive´). The focused assembly tests separately prove signal placement and active-cell indicator placement (´test:crate:assembly-signal-features´), (´test:crate:competitive-indicator-active´). None combines the real cache, maintenance-driven promotion, eviction, and one assessment-time observation.

The pending assessment already retains raw merged signal features and the active-cell identifiers that informed the assessment. Neither is exposed through `RiskAssessment`, and `World::flush_observations` settles the model owner's cold-ramp queue rather than the identity-maintenance queue.

The standing gap register has no Entry covering this promise (´sec:assayer:testing-plan-gaps´). The later harness roadmap names general snapshot diffing and domain assertions (´entry:assayer:harness-stage-tapes´), but it does not assign this promise to that stage; the focused read-only projection below is the smaller prerequisite the current code requires.

## The witness · `sec:assayer:intent-signals-and-competitive-cells-stand-independently-witness`

The witness belongs in the identity-layer integration module, where the full public lifecycle is already exercised (´test:integration:identity-register-allows-assess-and-derive´). Its test documentation cites the intent, and its module index states the four-state assertion rather than restating the setup.

- Setup: build a seeded scenario with one entity-persistent scalar named `trust`, signal-cache capacity one, and one identity dimension using depth cutoff three and split threshold ten, so one replacement observation cannot promote its coordinate.
- Establish cached-only: assess hot entity `alpha` with `trust = 0.75`, cross the identity-maintenance barrier, then assess `alpha` without signals before promotion; the probe for that assessment must read signal `[0.75]` and active-cell count zero.
- Establish both present: cross the barrier after the cached-only probe, then submit a bounded sequence of signal-free assessments for `alpha`, crossing the maintenance barrier after each one until the next probe reports a positive active-cell count; that assessment must still read signal `[0.75]`.
- Stimulate independent cache eviction: cross the maintenance barrier once more to drain the final cache hit, assess a far-coordinate entity `bravo` with `trust = 0.75`, cross the barrier, and require full health to report capacity one, size one, and an eviction while `alpha` remains competitively active.
- Observe signal-only: assess `bravo` without resupplying the signal and require `[0.75]` with active-cell count zero; the fixture also verifies that `bravo` remains below the configured split threshold.
- Observe cell-only: assess `alpha` without resupplying the signal and require `[0.0]` with a positive active-cell count.
- Observe neither: assess a third far-coordinate entity `charlie` without signals and require `[0.0]` with active-cell count zero.
- Ordinary-result assertion: every quadrant returns a well-formed reckoning, all four probes have the same one-position signal width, the feature-degradation counters remain clean, and no tolerance is applied to signal values or membership counts.

The probe reads the pending entry by the returned assessment identifier, so its signal vector and active-cell counts describe the same request. The existing indicator-placement witness ensures that a positive active-cell count is carried into the raw vector as one-valued competitive indicators rather than merely retained as metadata (´test:crate:competitive-indicator-active´).

The fails-before is direct. An implementation that gates cache lookup on competitive membership returns `[0.0]` for the cached-only assessment; one that gates active-cell lookup on a cache hit reports no cell after eviction; one that lets cache eviction clear identity state loses `alpha`'s active count; and one that treats either absence as exceptional fails to return a well-formed result for its quadrant.

## What is missing · `sec:assayer:intent-signals-and-competitive-cells-stand-independently-missing`

**Entry (A synchronous identity-maintenance barrier)** · `entry:assayer:intent-signals-and-competitive-cells-stand-independently-maintenance-barrier`

Roughly thirty to fifty lines in `World` test support add a test-gated `World::flush_identity_maintenance` verb. It sends the existing maintenance checkpoint command, waits under `ACK_DEADLINE` while the loop drains observations and deferred cache writes and emits cell changes, then crosses the model-owner barrier so every emitted lifecycle event is published before returning. It adds no sleep, production configuration, or host API and depends only on the completed harness skeleton (´entry:assayer:harness-stage-skeleton´).

**Entry (A read-only assessment-input projection)** · `entry:assayer:intent-signals-and-competitive-cells-stand-independently-input-probe`

Roughly forty to seventy lines under `src/testing/` add a test-gated projection keyed by `AssessmentId`, returning the pending entry's raw signal values and active competitive-cell count per dimension without exposing the pending entry or mutable engine state. The projection uses the existing non-consuming pending lookup, performs no publication or decay, and depends on no other lane.

**Entry (The four-quadrant integration witness)** · `entry:assayer:intent-signals-and-competitive-cells-stand-independently-integration-witness`

Roughly seventy to one hundred lines in the identity-layer integration module add the configured scenario, bounded one-assessment-at-a-time promotion driver, cache-health checks, four exact probes, fails-before documentation, module-index row, and clean-health assertion. It depends on (´entry:assayer:intent-signals-and-competitive-cells-stand-independently-maintenance-barrier´) and (´entry:assayer:intent-signals-and-competitive-cells-stand-independently-input-probe´), and on no sibling intent lane.

## Risks and open questions · `sec:assayer:intent-signals-and-competitive-cells-stand-independently-risks`

**Observation (Promotion is observed as state, not guessed from elapsed time)** · `obs:assayer:intent-signals-and-competitive-cells-stand-independently-promotion-state`

Graph promotion is deferred and depends on the configured graph thresholds. The driver advances one assessment per synchronous barrier and reads the active-cell predicate only after that barrier; a failed semantic bound reports the submitted observation count and configuration, while `ACK_DEADLINE` guards only the liveness of each acknowledgement. This cadence also prevents the capacity-one deferred-cache queue from dropping recency touches. No wall-clock sleep, virtual-time advance, or decay is part of the oracle.

**Observation (The cold ramp cannot stand in for block inspection)** · `obs:assayer:intent-signals-and-competitive-cells-stand-independently-raw-oracle`

Standardisation can map a raw zero to a non-zero standardised value, and model weights can make distinct raw inputs produce similar risks. The input projection therefore reads the pre-standardisation signal block and active membership retained for the assessment; risk is checked for finiteness only.

**Observation (One maintenance loop does not couple the stores)** · `obs:assayer:intent-signals-and-competitive-cells-stand-independently-shared-loop`

Cache writes and identity observations share a draining thread and barrier, but they remain separate queues and state structures. The witness distinguishes shared scheduling from shared preconditions by driving a promotion with no cache transition and an eviction with no loss of competitive membership.

**Observation (The projection stays narrower than snapshot diffing)** · `obs:assayer:intent-signals-and-competitive-cells-stand-independently-probe-scope`

The probe exposes only immutable copies already retained for one pending assessment and only under test support. It does not expose the dimension map, model parameters, cache object, graph, or working snapshot, so it supplies the missing oracle without turning the general snapshot-diffing stage into a prerequisite.

No maintainer decision remains open: the specification fixes both zero-fill semantics and cache/cell independence, the code already retains the exact assessment inputs needed by the oracle, and the existing maintenance checkpoint command supplies the required deterministic boundary.

## Acceptance · `sec:assayer:intent-signals-and-competitive-cells-stand-independently-acceptance`

The implementing lane's report identifies the new scenario in the identity-layer integration module, confirms that its test documentation cites the intent, updates the module index, and shows the coverage report resolving the intent to the integration witness.

The report identifies the test-gated maintenance barrier and assessment-input projection, confirms that the barrier drains both deferred queues before settling emitted lifecycle events, and confirms that the projection is non-consuming, read-only, and absent from the host API.

The reported evidence includes the exact four pairs `([0.0], 0)`, `([0.75], 0)`, `([0.0], positive)`, and `([0.75], positive)`; one cache eviction at capacity one; stable competitive membership for `alpha` across that eviction; well-formed results for every quadrant; and clean degradation health.

The report shows the targeted integration test and the nearest signal, identity, and assembly tests passing under the package's prescribed formatting, lint, feature, documentation, development, and release gates, with unrelated failures separated explicitly.

The report states which quadrant rejects cache lookup gated by membership, cell lookup gated by cache presence, cache eviction coupled to identity removal, a short signal block, or an exceptional absence. No executed mutation is required.
