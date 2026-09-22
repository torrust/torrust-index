# Keeping the Wide Random-Churn Pool Aligned · `plan:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair`

This plan keeps the promise that a wide, randomly churned Sentinel pool preserves the exact published feature width and leaves each changing model's precision and maintained covariance within the width-scaled synchronisation bound (´claim:lifespan:a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair´).

## What the promise says, precisely · `sec:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-promise`

The fixture begins from a cold working copy with one bias feature, fifteen aggregate features, and no signals, identity dimensions, outcome axes, interactions, or competitive indicators. For $n$ live Sentinels the independently computed dimension is therefore $p_{\mathrm{expected}}=1+15+n(60+1)=16+61n$ (´tab:feature:dimension-formula´), because a Sentinel with no spatial outcome axis owns sixty extracted coordinates and one occupancy coordinate (´def:extraction:slot´).

Eight initial registrations give $p_{\mathrm{expected}}=504$. Each retirement leaves seven registrations and $p_{\mathrm{expected}}=443$; the fresh replacement restores eight registrations and $p_{\mathrm{expected}}=504$.

The width quantity is the published dimension-map width, checked against the independently maintained live-registration set rather than another structure derived from that map. The published operational and sister parameter widths, covariance storage, standardisation-vector lengths, Sentinel-slot count, slot identities, and bounded disjoint slot ranges must agree with that independent value, as required by version consistency (´inv:dimension:version-consistency´) and the sole-resolver decision (´dec:vector:sole-resolver´).

Each retirement follows the Sentinel lifecycle table, applying the regularised numerical form of Gaussian marginalisation before compacting and rebuilding the layout (´tab:registry:sentinel-operations´) (´thm:gaussian:marginalisation´) (´alg:gaussian:regularised-schur´) (´alg:dimension:compaction´). Each replacement extends every changing full-feature model by the independent prior block (´thm:gaussian:extension´).

For each changing model the inverse-pair quantity is the whole-matrix Frobenius departure $\epsilon_{\mathrm{sync}}=\lVert B\tilde\Sigma-I\rVert_F$, not a diagonal proxy and not the cadence's prior-adjusted residual (´def:monitoring:synchronisation-error´). The admitted ceiling is inclusive and equals $10^{-6}p$ (´def:config:synchronisation-threshold´); `Tolerances::precision_sync_ceiling` applies the coefficient carried by `DEFAULT_TOLERANCES` (´tab:assayer:harness-scenario-tolerances´).

The operational and sister models are the changing full-feature models present in this fixture. The anchor remains at its fixed width (´dec:substrate:anchor-invariant´), and the fixture creates no outcome-axis models, so neither the anchor nor an outcome-axis model belongs in the terminal pair assertion.

The measurable duration is one hundred completed retirement-and-replacement rounds after the pool first reaches eight. One declared seed supplies a reproducible, non-rotational selector for each round, and the victim remains a function of that selector and the current live pool (´dec:harness:seeded-sweeps´).

## What the code offers today · `sec:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-current-code`

The promise remains unkept. `tests::lifecycle::gauntlet_deregister_register_churn_100_cycles` already starts from eight Sentinels, chooses a live victim with a seeded `TestRng`, installs a fresh identifier, and repeats for one hundred rounds (´test:crate:gauntlet-deregister-register-churn-100-cycles´). It observes only the restored edge, derives the expected width from the dimension map under test, and never multiplies the maintained precision by its covariance.

The public integration witness `lifecycle_churn_register_deregister_32_rounds` rotates four fixed names for thirty-two rounds, performs one assessment per registration, and finishes with clean health (´test:integration:lifecycle-churn-register-deregister-32-rounds´). It proves liveness and finite output rather than an independent feature count or inverse-pair bound.

Two campaign tests now cover adjacent mechanisms without keeping this claim. `tests::dimension_map::p_matches_model_assertion` checks generated layouts against the independent block formula through `dimension_width` (´test:crate:p-matches-model-assertion´), while `tests::seeded_publication_invariants::lifecycle_publication_seeded_sweep` proves complete pre-event and post-event publications for every lifecycle direction (´test:crate:lifecycle-publication-seeded-sweep´). Neither test sustains a wide random-churn pool or reads the maintained pair after that churn.

The finished oracle and sweep modules export `DimensionBlocks`, `dimension_width`, and `run_seeded_sweep` (´tab:assayer:harness-implementation-library-roster´) (´dec:harness:oracle-tier´) (´dec:harness:seeded-sweeps´). The width oracle replaces the plan's proposed local formula helper, and the seeded sweep supplies the case index, drawn selector, seed, and rejection reason on failure.

The finished `World` surface supplies `World::register_sentinel`, `World::deregister_sentinel`, `World::observed_runtime_layout`, `World::published_model_block`, and `World::published_slot_moments`; each observation method crosses its named publication barrier and loads one owned reading (´dec:harness:probe-contract´). Those probes deliberately omit precision and a complete slot-key-and-range projection, so they cannot alone observe this promise's terminal pair and exact live set (´dec:retention:precision-excluded´).

The existing crate-local `LifecycleEnv` remains the direct observation boundary for the excluded state. `LifecycleEnv::submit` calls `handle_lifecycle_submission` synchronously, mutates the `WorkingCopy`, and installs the resulting `ModelSnapshot` in `SharedState`; the same environment retains the operational and sister `BayesianLinearModel` values needed by `BayesianLinearModel::precision`, `BayesianLinearModel::covariance`, and `model::recompute::synchronisation_error`. The harness contract explicitly places a witness of excluded state in a direct crate-level home rather than a nonconforming probe (´dec:harness:probe-contract´).

The witness begins cold, moves no time, and waits on no asynchronous queue. It needs neither `TrainedStateFixture` nor declarative playback: `run_seeded_sweep` owns the invariant-case loop, while each direct lifecycle submission returns only after its publication is installed.

The standing testing plan still names the remaining mechanism under the recovery-residual gap: crate-level or instrumented evidence that lifecycle work publishes internally matched precision, covariance, and dimension-map state (´entry:assayer:gap-recovery-residuals´).

## The witness · `sec:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-witness`

Strengthen `tests::lifecycle::gauntlet_deregister_register_churn_100_cycles` in the `torrust_assayer` test target, preserving its fixture, test label, finite-state guards, and fresh-identifier sequence (´test:crate:gauntlet-deregister-register-churn-100-cycles´).

- Setup: construct the cold `LifecycleEnv`, register Sentinel identifiers one through eight, maintain their identifiers separately from the dimension map, load the installed `ModelSnapshot` once after the eighth publication, and compare it with `dimension_width` over a `DimensionBlocks` value whose only populated field is the live Sentinel count.

- Seeded churn: call `run_seeded_sweep` with the declared seed and one hundred cases. Each drawn `u64` selects its victim modulo the current eight-element live vector, so the sweep owns repetition and diagnostics while the victim continues to depend on the changing live set.

- Removal edge: remove the selected identifier from the independent live vector, submit `LifecycleEvent::DeregisterSentinel`, load the installed snapshot once, and require `dimension_width` to return 443 for the seven identifiers that remain.

- Removal observation: require the snapshot's dimension-map width, operational and sister parameter widths, mean lengths, square covariance-storage lengths, standardisation-vector lengths, exact Sentinel-slot key set, and bounded disjoint slot ranges to agree with the independent width and live set. Require the corresponding `WorkingCopy` widths and feature-class length to agree as well.

- Replacement edge: allocate a never-before-used identifier, submit `LifecycleEvent::RegisterSentinel`, add the identifier to the independent live vector, load that publication once, and repeat the complete observation against the restored width 504 and eight-element set.

- Terminal pair observation: after the hundredth replacement publication, call `model::recompute::synchronisation_error` separately on the precision and covariance returned by the working operational and sister models.

- Terminal pair assertion: compare both whole Frobenius readings with `Tolerances::precision_sync_ceiling` from `DEFAULT_TOLERANCES` at width 504 through `testing::assert_below`; do not recompute either covariance and do not substitute the last health reading.

- Existing guards: retain finite model means, finite non-negative standardisation statistics, bounded disjoint slots, the conserved pool count, and the fresh-identifier liveness check.

- Claim placement: replace the weaker claim mint on the existing crate test with the intended lifespan claim, remove the intent-only mint from the statements-of-intent catalogue (´sec:assayer:intents´), and refresh the test indexes so the claim resolves to the strengthened witness without duplicate ownership.

The width fails-before is a rebuild that drops a coordinate, retains one from the victim, or updates every dimension-bearing object to the same wrong width. Agreement among internal objects can remain green, while `dimension_width` rejects 443 or 504 at the first faulty publication.

The inverse-pair fails-before is a marginalisation or extension that updates only one of $B$ and $\tilde\Sigma$, gathers one through stale positions, or accumulates roundoff beyond the specified allowance. Such an implementation can keep every value finite and every health report clean while the terminal whole-product assertion exceeds $10^{-6}p$.

## What is missing · `sec:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-missing`

No production hook, public API, barrier, probe, playback feature, clock control, trained fixture, oracle, or cross-intent dependency is missing. Only the following test-local applications and their index updates remain.

**Entry (The independent Sentinel-only width oracle is applied at both edges)** · `entry:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-width-oracle`

The `tests::lifecycle` module needs one local assertion that accepts a `ModelSnapshot`, the corresponding `WorkingCopy`, and the independently maintained live identifiers; obtains the expected width from `dimension_width`; and checks every published and working width, exact slot key, and slot boundary named above. It adds no formula or shared oracle because `DimensionBlocks` and `dimension_width` are finished harness items (´dec:harness:oracle-tier´).

**Entry (The two-edge churn and terminal pair assertions)** · `entry:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-strengthened-gauntlet`

The existing crate test needs to route its one hundred selectors through `run_seeded_sweep`, invoke the local width assertion after both lifecycle edges, measure the two terminal pairs through `model::recompute::synchronisation_error`, carry the intended claim citation, and refresh its module and folder indexes. It depends only on the preceding entry, `run_seeded_sweep`, `DEFAULT_TOLERANCES`, `Tolerances::precision_sync_ceiling`, `testing::assert_below`, and the direct crate state it already owns.

## Risks and open questions · `sec:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-risks`

**Observation (The crate-level home preserves the retention boundary)** · `obs:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-crate-level-home`

Moving this witness to integration scope would require precision to cross a surface that deliberately excludes it. Direct access inside the existing crate test is the contract's stated home for excluded state, while the one installed `ModelSnapshot` supplies the published half without widening the API (´dec:harness:probe-contract´) (´dec:retention:precision-excluded´).

**Observation (The whole reading is the promise's quantity)** · `obs:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-whole-reading`

The monitor separates prior-induced departure from arithmetic residual for cadence decisions, but no label or forgetting step runs in this fixture and the promise names the full Frobenius reading. Subtracting a component would weaken the assertion and contradict the quantity fixed by the synchronisation-error definition (´def:monitoring:synchronisation-error´).

**Observation (Seeded randomness is reproducible, not scheduled rotation)** · `obs:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-seeded-randomness`

One declared seed makes failures replayable. Each complete drawn selector is reported by `run_seeded_sweep` and chooses from the current live vector, so the retirement order remains distinct from the fixed four-name integration rotation (´dec:harness:seeded-sweeps´).

**Observation (Both publication edges prevent cancellation)** · `obs:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-both-publication-edges`

Checking only the restored width permits an erroneous removal and replacement to cancel. One snapshot load after each synchronous submission makes every shrinking and growing transition observable and checks the published layout against the registrations standing at that edge (´inv:guarantee:lifecycle-publication´).

**Observation (The direct matrix product stays terminal)** · `obs:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-terminal-product`

The full synchronisation reading costs cubic work at a width near five hundred. The promise concerns the pair left by the completed churn and slow accumulated departure, so one direct terminal product per changing model carries the required evidence without multiplying that cost by every round.

**Observation (The model scope follows the fixture)** · `obs:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-model-scope`

The operational and sister models are every lifecycle-changing full-feature model in this fixture. Adding an outcome axis would add another changing model and alter the width formula, while including the fixed anchor would test a separate invariant; neither strengthens the stated eight-Sentinel witness.

No maintainer decision remains open: the cold fixture, finished width oracle, direct retained matrices, seeded sweep, and specified tolerance determine the witness completely.

## Acceptance · `sec:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair-acceptance`

The `torrust_assayer` test target runs `tests::lifecycle::gauntlet_deregister_register_churn_100_cycles`; its module index and test documentation carry the intended claim, and the intent catalogue and generated coverage indexes resolve that claim to (´test:crate:gauntlet-deregister-register-churn-100-cycles´).

The report gives the declared seed and the independently derived widths 443 and 504, states that both publication edges of every round were checked, and records the terminal operational and sister Frobenius readings beside their width-scaled ceilings.

The report shows the focused crate test and the package's prescribed formatting, lint, feature, documentation, and test gates passing, with any unrelated failure separated explicitly.

The report states the two concrete fails-before classes: a common but formula-wrong published width rejected at its first edge, and a finite but desynchronised maintained pair rejected by the terminal complete-product bound; no executed mutation is required.

Acceptance excludes a public precision accessor, cadence-triggered repair, a diagonal-only drift proxy, timing allowances, unseeded sampling, and dependence on another intent.
