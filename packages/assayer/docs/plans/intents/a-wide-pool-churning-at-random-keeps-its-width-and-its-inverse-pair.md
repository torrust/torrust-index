# Keeping the Wide Random-Churn Pool Aligned · `plan:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair`

This plan keeps the promise that a wide, randomly churned Sentinel pool preserves the exact published feature width and leaves each changing model's precision and maintained covariance within the width-scaled synchronisation bound (´claim:lifespan:a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair´).

## What the promise says, precisely · `sec:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-promise`

The fixture begins from the existing cold working copy with one bias feature, fifteen aggregate features, and no signals, identity dimensions, outcome axes, interactions, or competitive indicators. For $n$ live Sentinels the independent dimension formula is therefore $p_{\mathrm{expected}}=1+15+n(60+1)=16+61n$ (´tab:feature:dimension-formula´), because each Sentinel owns its sixty extracted coordinates plus one occupancy coordinate (´def:extraction:slot´).

Eight initial registrations give $p_{\mathrm{expected}}=504$. Within every churn round, the random retirement leaves seven registrations and $p_{\mathrm{expected}}=443$, while the fresh replacement restores eight registrations and $p_{\mathrm{expected}}=504$.

The width quantity is the published `DimensionMap.p`, checked against the independently counted live registrations rather than against another structure derived from that map. The published operational and sister parameter widths, covariance storage, standardisation-vector lengths, Sentinel-slot count, slot identities, and bounded disjoint slot ranges agree with the same value, as required by version consistency (´inv:dimension:version-consistency´) and by the sole-resolver decision (´dec:vector:sole-resolver´).

Every retirement marginalises the departing slot and every replacement extends all changing full-feature models according to the Sentinel lifecycle table (´tab:registry:sentinel-operations´), using Gaussian marginalisation (´thm:gaussian:marginalisation´), Gaussian extension (´thm:gaussian:extension´), and the canonical compaction rebuild (´alg:dimension:compaction´).

For each changing model the inverse-pair quantity is the whole-matrix Frobenius departure $\epsilon_{\mathrm{sync}}=\lVert B\tilde\Sigma-I\rVert_F$, not a diagonal proxy and not the cadence's prior-adjusted residual (´def:monitoring:synchronisation-error´). The admitted ceiling is inclusive and equals $10^{-6}p$ under the reference configuration (´def:config:synchronisation-threshold´); `DEFAULT_TOLERANCES.precision_sync_ceiling(p)` carries that coefficient into tests (´tab:assayer:harness-scenario-tolerances´).

The operational and sister models are the changing full-feature models present in this fixture. The anchor remains at its fixed width by design (´dec:substrate:anchor-invariant´), and the fixture creates no outcome-axis models, so neither belongs in the churn-pair assertion.

The measurable duration is one hundred completed retirement-and-replacement rounds after the pool first reaches eight. The eviction choices come from a fixed `TestRng` seed, giving a reproducible non-rotational sequence while still making each victim a function of the current live pool.

## What the code offers today · `sec:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-current-code`

The public path is `Assayer::register_sentinel` and `Assayer::deregister_sentinel`; the `World` harness wraps those calls with `World::register_sentinel` and `World::deregister_sentinel` and settles each one at a publication barrier. The completed harness skeleton supplies `World`, deterministic `TestRng`, name allocation, lifecycle verbs, liveness deadlines, and the tolerance bundle (´entry:assayer:harness-stage-skeleton´).

That public path can drive the churn, but it cannot observe both quantities in the promise. `ModelSnapshot` publishes the dimension map and covariance-bearing `ModelParameters` atomically (´def:publication:model-snapshot´), while precision is deliberately excluded from the snapshot (´dec:retention:precision-excluded´); the health surface reports the last monitor visit rather than a fresh direct product after the terminal lifecycle event.

The crate-local `LifecycleEnv` beside the existing random-churn gauntlet (´test:crate:gauntlet-deregister-register-churn-100-cycles´) offers the required observation boundary without changing production visibility. Its `submit` path invokes `handle_lifecycle_submission`, mutates the `WorkingCopy`, and installs each chain's immutable snapshot in `SharedState`, matching the one-swap publication discipline (´dec:concurrency:snapshot-swap´) and lifecycle publication invariant (´inv:guarantee:lifecycle-publication´).

The same environment exposes `working.operational` and `working.sister`, whose `precision()` and `covariance()` accessors feed `model::recompute::synchronisation_error`. `testing::assert_below` and `DEFAULT_TOLERANCES.precision_sync_ceiling` already express the inclusive bound with a diagnostic value and limit.

The existing crate witness already prepopulates eight Sentinels, chooses a live victim through seeded `TestRng`, registers a fresh identifier, and repeats for one hundred rounds (´test:crate:gauntlet-deregister-register-churn-100-cycles´). It checks only the completed round, derives `p` from the dimension map itself, compares working-copy structures with that shared value, and asserts finite means and statistics; it never reads either intermediate publication or multiplies precision by covariance.

The nearest integration witness cycles four fixed names in a rotation for thirty-two rounds, makes one reckoning per registration, and finishes with clean health (´test:integration:lifecycle-churn-register-deregister-32-rounds´). Its own lightened-variant comment is exact: it establishes liveness and finite output, not an independent feature count or inverse-pair drift.

The standing testing plan covers this promise through the recovery-residual gap, which calls for crate-level or instrumented evidence that lifecycle work publishes internally matched precision, covariance, and dimension-map states (´entry:assayer:gap-recovery-residuals´).

## The witness · `sec:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-witness`

The witness strengthens the existing random-churn gauntlet (´test:crate:gauntlet-deregister-register-churn-100-cycles´); its fixture, random eviction sequence, test label, and finite-state guards remain the foundation.

- Setup: retain the cold `LifecycleEnv`, register Sentinel identifiers one through eight, hold the live identifiers independently of `DimensionMap`, and assert the first published state has eight matching slots and width 504.

- Removal stimulus: on every round, use seeded `TestRng::next_range` over the current live vector, remove that victim from the expected live set, submit its deregistration, and load the snapshot installed by that submission before any replacement is registered.

- Removal observation: derive 443 from the seven identifiers still live and require the published dimension map, operational parameters, sister parameters, covariance lengths, standardisation vectors, and exact Sentinel-slot key set to describe that width and set.

- Replacement stimulus: allocate a never-before-used identifier, submit its registration, add it to the expected live set, and load the resulting publication independently of the removal publication.

- Replacement observation: derive 504 from the restored eight-identifier set and repeat the published width, storage, standardisation, key-set, and slot-range assertions against that independent oracle.

- Terminal pair observation: after the hundredth replacement publication, compute `synchronisation_error(model.precision(), model.covariance())` separately for the working operational and sister models at their common width.

- Terminal pair assertion: compare each complete Frobenius reading with `DEFAULT_TOLERANCES.precision_sync_ceiling(p)` through `assert_below`; do not invoke recomputation or substitute the last health reading before the measurement.

- Existing guards: retain finite means and variances, non-negative variances, bounded disjoint slots, the conserved pool count, and the fresh-identifier liveness check so the stronger witness continues to reject crashes and malformed states alongside quiet drift.

- Claim placement: replace the existing weaker crate-test claim mint with the intended lifespan claim, remove the intent-only mint from the statements-of-intent catalogue (´sec:assayer:intents´), and refresh the test indexes so the promise resolves to this witness without duplicate ownership.

The width fails-before is a rebuild that drops one coordinate, retains one coordinate from the victim, or updates every dimension-bearing object to the same wrong size: agreement among internal objects can remain green, while the formula-derived 443 or 504 assertion fails at the first faulty publication.

The inverse-pair fails-before is a marginalisation or extension that updates only one of $B$ and $\tilde\Sigma$, gathers one of them through stale positions, or accumulates roundoff beyond the declared allowance. Such an implementation can keep every value finite and every health report clean, while the terminal whole-product assertion exceeds $10^{-6}p$.

## What is missing · `sec:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-missing`

No production hook, new public API, asynchronous fixture, clock control, or other intent lane blocks the witness. The missing material is confined to the existing crate test and its generated or curated indexes.

**Entry (The independent Sentinel-only width oracle)** · `entry:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-width-oracle`

A fixture-local helper of roughly ten lines computes $16+61n$ from the independently maintained live-identifier set and checks a loaded publication's map, parameter, covariance-storage, standardisation, key-set, and range widths. It depends on the existing `LifecycleEnv` and on no other lane.

**Entry (The two-edge churn and terminal pair assertions)** · `entry:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-strengthened-gauntlet`

Roughly thirty lines extend the existing gauntlet with a snapshot check after each removal and replacement, two terminal direct synchronisation readings, the intended claim citation, and index updates. It depends on (´entry:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-width-oracle´), `TestRng`, `synchronisation_error`, `assert_below`, and `DEFAULT_TOLERANCES`, with no production or cross-lane dependency.

## Risks and open questions · `sec:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-risks`

**Observation (The crate-level home preserves the retention boundary)** · `obs:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-crate-level-home`

An integration-only witness would need precision added to a published or test-only public surface, despite the explicit precision exclusion. The local lifecycle environment already drives the production handler and publication swap while exposing both maintained matrices, so crate placement tests the mechanism without widening the API.

**Observation (The whole reading is the promise's quantity)** · `obs:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-whole-reading`

The monitor separates prior-induced departure from arithmetic residual for cadence decisions, but no label or forgetting step runs in this fixture and the promise names the full Frobenius reading. Subtracting a component here would weaken the assertion and depart from the cited definition.

**Observation (Seeded randomness is reproducible, not scheduled rotation)** · `obs:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-seeded-randomness`

One fixed seed makes failures replayable and removes probabilistic flakiness. The victim still depends on a pseudo-random draw from the changing live set, so the retirement order is not the four-name rotation exercised by the lighter integration case; failure diagnostics carry the round, victim, and seed.

**Observation (Both publication edges prevent cancellation)** · `obs:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-both-publication-edges`

Checking only the restored width after a complete round permits an erroneous removal and an erroneous replacement to cancel. Reading the snapshot between them makes every shrinking and growing transition observable and proves the published layout matches the registrations that stand at that moment.

**Observation (The direct matrix product stays terminal)** · `obs:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-terminal-product`

The full synchronisation reading costs cubic work at a width near five hundred. The promise requires the inverse pair left by the complete churn and calls out slow accumulation, so one direct terminal product per changing model detects the accumulated departure without multiplying that cost by every round; width remains asserted at every publication.

**Observation (The model scope follows the fixture)** · `obs:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-model-scope`

The operational and sister models are every lifecycle-changing full-feature model in the selected fixture. Adding an outcome axis would add another changing model and change the width formula, while including the fixed anchor would test a separate invariant; neither strengthens this promise's stated eight-Sentinel witness.

No maintainer decision remains open: the existing fixture, independent width formula, retained working matrices, seeded churn, and specified tolerance determine the witness completely.

## Acceptance · `sec:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-acceptance`

The implementing lane's report identifies the strengthened crate witness, confirms its module index and test documentation carry the intended claim, and shows the intent catalogue and coverage indexes resolving that claim to (´test:crate:gauntlet-deregister-register-churn-100-cycles´).

The report gives the fixed seed and the independently derived widths 443 and 504, states that both publication edges of every round were checked, and records the terminal operational and sister Frobenius readings beside their width-scaled ceilings.

The report shows the focused crate test passing and the package's prescribed formatting, lint, feature, documentation, and test gates passing, with any unrelated failure separated explicitly.

The report states the two concrete fails-before classes: a common but formula-wrong published width rejected at its first edge, and a finite but desynchronised maintained pair rejected by the terminal complete-product bound; no executed mutation is required.

Acceptance excludes a new public precision accessor, a cadence-triggered repair, a diagonal-only drift proxy, timing allowances, unseeded sampling, and dependence on another intent lane.
