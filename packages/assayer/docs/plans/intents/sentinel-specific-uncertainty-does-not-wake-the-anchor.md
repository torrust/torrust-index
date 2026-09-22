# Keeping Sentinel-Local Uncertainty out of the Anchor Weight · `plan:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor`

Keeping (´claim:risk:sentinel-specific-uncertainty-does-not-wake-the-anchor´) establishes that a request can remain uncertain in one Sentinel's private feature block without transferring any more of its estimate to a model that cannot represent that block.

## What the promise says, precisely · `sec:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-promise`

The blend compares the anchor variance $\sigma^2_\text{anc}(\tilde{\phi})$ with the sister variance $\sigma^2_{\text{inh},S_g}(\phi)$ restricted to the gathered coordinates both models share, and sets $w(\phi)=\left(1-\sigma^2_\text{anc}/(\sigma^2_{\text{inh},S_g}+\varepsilon)\right)^+$ (´def:risk:subspace-blend´).

For two assessments $F$ and $S$ read from one model publication, the precondition is $\tilde{\phi}_F=\tilde{\phi}_S$: they have the same bias, aggregate values, absent identity values, computed maximum and reporting-Sentinel count. Their full vectors differ because the same report extraction occupies the well-fed Sentinel slot in $F$ and the label-starved Sentinel slot in $S$.

The kept invariant is $w(\phi_F)=w(\phi_S)$ even while the public full-direction uncertainty satisfies $\sigma_\text{eff}(\phi_S)>\sigma_\text{eff}(\phi_F)$. Exact weight equality keeps the exclusion; the strict uncertainty inequality proves that the pair differs in a direction the public estimate still regards as uncertain.

“Does not wake” means both assessments remain in the sister-dominated regime $w<0.3$, whose boundary is fixed by (´def:platt:regimes´), and both report `RiskBasis::anchor_converged` as true. The approximately two-to-ten-per-cent residual described by (´rem:risk:blend-mechanisms´) remains diagnostic because its exact magnitude is deployment-specific (´cav:limitation:anchor-floor´).

The anchor cannot read either Sentinel slot. Its fixed fifteen-position projection contains only the shared sources enumerated by (´def:dimension:anchor-projection´), and the anchor model remains fixed while the full model carries every dynamic slot (´def:risk:anchor-model´).

The specification states the same property as the invariant that activation is insensitive to Sentinel-specific and axis-specific uncertainty (´inv:guarantee:blend-subspace´). The calibration record makes the restriction structural: the projected quadratic form indexes only anchor coordinates (´dec:calibration:indexed-form´), and no off-subspace component is ever read (´cor:calibration:subspace-containment´).

Elapsed time is not a hidden threshold. The weight uses raw forms because the common time correction cancels from its ratio (´dec:calibration:raw-weight-forms´); the virtual clock remains fixed throughout the witness.

The promise remains part of the standing gap for behavioural anchor coverage (´entry:assayer:gap-anchor-behaviour´).

## What the code offers today · `sec:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-current-code`

`compute_blend` already computes the full sister variance, shared-subspace sister variance and anchor variance. It derives `BlendResult::anchor_weight` from only the latter two and uses the full sister variance in the effective mixture, exactly separating the quantity this witness holds invariant from the uncertainty it requires to differ.

The seeded crate witness `blend_component_diagnostics_seeded_sweep` now changes excluded feature coordinates across independently generated diagonal cases and requires bit-identical anchor weights while independently recomputing all three component variances (´test:crate:blend-component-diagnostics-seeded-sweep´). It uses `run_seeded_sweep` under the reproducible-sweep contract (´dec:harness:seeded-sweeps´), but it calls `compute_blend` directly and therefore does not establish the claim through report ingestion, assessment, standardisation and publication.

The earlier focused witnesses still establish indexed projection against an extracted submatrix (´test:unit:projected-qf-matches-submatrix´), component-variance consistency (´test:unit:variance-diagnostics-populated´), anchor activation under genuine shared uncertainty (´test:unit:anchor-only-w-near-one´), and cancellation of time correction from the weight (´test:unit:time-correction-cancels-in-weight´).

The public `RiskBasis` exposes `anchor_weight`, `sigma_eff`, `anchor_converged` and `n_sentinels_reporting`; `HealthSnapshot` exposes the publication version, snapshot age, standardisation phase and per-assessment degradation. Those readings are sufficient for the result oracle without exposing covariance or adding a production field (´schema:risk:basis´).

The finished scenario surface supplies `scenario`, `World::register_sentinel`, `World::receive_report`, `World::request_with_sentinel`, `World::settle_cold_ramp_with`, `World::flush_observations`, `World::flush_labels`, `World::assess`, `World::label` and `World::block_model_owner_for_test` (´tab:assayer:harness-implementation-library-roster´). The observation and label barriers cover different queues, so the setup must call the one whose state it reads rather than infer completion from the other (´dec:harness:no-ad-hoc-waits´).

`World::published_slot_moments` returns an owned `PublishedSlotMoments` after one publication barrier and one snapshot load, which is enough to compare both slots' bootstrap baselines without retaining engine state (´dec:harness:probe-contract´) (´test:crate:published-slot-moments-cross-publication-as-one-owned-slot´).

Long label streams now use subject-owned `PlaybackRow` values through `playback`, with a declared `PlaybackBarrierPolicy`, `PlaybackCheckpoint`, `PlaybackProgress` and `PlaybackBatchSize` (´dec:harness:declarative-playback´). `PlaybackBarrier::FlushLabels` publishes each row before its checkpoint, a contract already exercised by (´test:crate:selected-label-barrier-completes-before-checkpoint´).

The golden-report surface retains `golden_report` and `GOLDEN_COORD`. Its discriminating readings are authored stimulus (´dec:assayer:golden-report-stimulus´), so using the same report and coordinate for both Sentinels makes their raw aggregate inputs equal by construction.

`LabelSpec::benign` and `LabelSpec::adverse` both default to `Action::Allow`, so alternating them sends eligible, unconfounded labels to the sister and anchor under (´tab:eligibility:training´) while only the reporting Sentinel's private slot is populated in each stored vector.

`TrainedStateFixture` guards a score-verified scalar population, not equal-report Sentinel slots or a one-sided label history, so its baseline is not a valid precondition for this witness (´dec:harness:guarded-fixtures´). None of `pairwise_rank`, `regularised_schur_complement`, `decay_recurrence` or `dimension_width` recomputes this per-assessment blend ratio, and the result needs no new shared oracle because its required equality is exact and its deliberate defect changes which quadratic form the production weight reads (´dec:harness:oracle-tier´).

The nearest public learning witness drives a fixed assess-label population to directional separation (´test:integration:scalar-signal-population-split-converges-directionally´). The nearest two-Sentinel witness drives a 1,500-row labelled stream through two routed slots (´test:integration:association-separates-a-keyed-sentinel-from-a-hash-fed-one´), but neither compares matched shared projections with one slot starved of labels.

## The witness · `sec:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-witness`

The cargo integration target `posterior_convergence` and test function `posterior_convergence::sentinel_specific_uncertainty_does_not_wake_the_anchor` are the destination because posterior uncertainty and projection are their principal subject; the target already reserves that convergence split (´dec:harness:convergence-split´).

The setup calls `scenario` once to build one seeded `Scenario`, registers Sentinels `fed` and `starved` before learning, ingests `golden_report` for each, and forms two requests with the same entity and `GOLDEN_COORD`, each naming exactly one Sentinel (´dec:harness:single-scenario´). No identity dimension or request signal is registered, so the bias, aggregate block, absent identity block and both computed anchor positions are equal by construction.

`World::settle_cold_ramp_with` drives the two requests until the full-vector ramp reports `StandardisationPhase::InService`, crossing `World::flush_observations` on every advance. The setup captures both pre-bootstrap slots through `World::published_slot_moments`, issues at least one hundred further unlabelled assessments through each Sentinel, crosses `World::flush_labels` for their bootstrap-completion commands, and reads both slots again; the guard requires the two post-bootstrap moment vectors to be bit-identical and different from their respective baselines before one-sided training starts (´alg:standardisation:batch-initialisation´) (´alg:standardisation:sentinel-bootstrap´) (´dec:harness:probe-contract´).

A subject-owned label row calls `World::assess` for the `fed` request, builds alternating `LabelSpec::benign` and `LabelSpec::adverse` values from the returned assessment identifiers, and submits them through `World::label` without choosing its own wait. `playback` drives 1,500 rows with `PlaybackBatchSize::default()`, matching the established two-Sentinel stream budget, under `PlaybackBarrierPolicy::new` with only `PlaybackBarrier::FlushLabels`; a final `PlaybackCheckpoint` records the applied eligible-label count, and `PlaybackProgress` must report every row complete (´dec:harness:declarative-playback´) (´test:integration:association-separates-a-keyed-sentinel-from-a-hash-fed-one´).

Alternation holds the target mean near zero so the final uncertainty contrast is not manufactured solely by anchor-sister point-estimate disagreement, while the repeated shared projection concentrates both models along the direction the final pair asks about. The fixed row count is fixture input rather than a product threshold; the guarded baseline reports the completed rows, accepted labels, standardisation phase and bootstrap measurements before the result assertions run (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

After the final playback barrier, `World::block_model_owner_for_test` parks publication while `World::assess` reads the `fed` and `starved` requests (´tab:assayer:harness-implementation-library-roster´). Their `HealthSnapshot::snapshot_version`, `HealthSnapshot::snapshot_age_seconds`, standardisation phase and reporting count must match, and each per-Sentinel alarm summary must carry the same golden-report readings; only the occupied full-model slot differs.

The primary assertion compares `fed.risk.anchor_weight.to_bits()` with `starved.risk.anchor_weight.to_bits()` for exact equality. Both assessments execute the same raw quadratic forms over one shared vector and one immutable publication, so a tolerance would admit influence from a coordinate the formula is defined not to read.

The regime assertion requires the common weight to be below $0.3$, both `RiskBasis::anchor_converged` values to be true, and both `RiskBasis::n_sentinels_reporting` values to equal one. These checks establish a learned, sister-dominated shared direction rather than obtaining equality from two untouched priors.

The non-vacuity assertion requires the `starved` `RiskBasis::sigma_eff` to exceed the `fed` value by more than `World::tol().default`, the declared one-shot floating-point budget (´tab:assayer:harness-scenario-tolerances´). It proves that the excluded direction changes public uncertainty without claiming a component covariance the public surface does not expose.

Both inline degradation records must be clean, both coordinate snapshots must be in service, and `assert_health_clean` must find no sanitisation, model fallback, cascade terminus, stopped label path or dropped health event (´tab:assayer:harness-implementation-library-roster´). Failure output prints the two weights, uncertainties, convergence flags, reporting counts, snapshot versions, standardisation phases and playback progress.

The fails-before replaces the projected sister quadratic form used for the weight in `compute_blend` with the full sister quadratic form while leaving the effective mixture unchanged. The fed and starved denominators then read their different full vectors and the exact-equality assertion fails; the existing seeded component witness fails for the same defect before the public-path result is considered (´test:crate:blend-component-diagnostics-seeded-sweep´).

## What is missing · `sec:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-missing`

**Entry (A matched two-Sentinel starvation fixture)** · `entry:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-matched-starvation-fixture`

The `posterior_convergence` target still needs a private guarded `MatchedStarvationFixture` with a measured `MatchedStarvationBaseline`. It owns the equal reports, symmetric ramp and bootstrap, subject-specific playback rows and one-sided labelled stream; before returning it reports the pre- and post-bootstrap slot moments, phase, completed row count, accepted label count and health state, and refuses an unmet precondition without testing anchor-weight equality or the uncertainty contrast. This is target-local ceremony over the finished harness, not a new production or reusable harness API (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´) (´dec:harness:specialised-side-harnesses´).

**Entry (The public-path exclusion witness)** · `entry:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-public-witness`

The labelled integration test, its module-index row, exact shared-publication checks, low-regime assertion, uncertainty non-vacuity guard, clean-health checks and diagnostic output remain absent. They add no production field, probe, oracle, seed sweep or broader harness capability.

## Risks and open questions · `sec:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-risks`

**Observation (Standardisation can impersonate starvation)** · `obs:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-standardisation-confound`

A starved bootstrap and a starved posterior are different deficits. The cold ramp uses the observation queue, Sentinel bootstrap completion uses the command queue, and `World::flush_labels` does not settle cold observations; the guarded setup must cross each named barrier and prove the two bootstraps symmetric before the one-sided label playback begins (´dec:harness:no-ad-hoc-waits´).

**Observation (The promise is directional)** · `obs:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-directional-precondition`

The quadratic forms estimate uncertainty along the request vector, not convergence of every direction in the shared covariance matrix. Repeating one shared projection is sufficient for this assessment pair and establishes nothing about an orthogonal shared direction.

**Observation (The public uncertainty is a non-vacuity check)** · `obs:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-public-uncertainty-limit`

`RiskBasis::sigma_eff` is a tempered mixture containing full sister variance, anchor variance and model disagreement, not the raw full sister form. Alternating targets limits the disagreement term, but the strict inequality remains only evidence that the public pair is non-interchangeable; exact weight equality and the mutation control establish the exclusion.

**Observation (Low has a regime boundary, not a universal floor)** · `obs:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-low-threshold`

The specification fixes $0.3$ as the sister-to-anchor regime boundary and describes the smaller residual as deployment-dependent. The binding result is therefore $w<0.3$ plus exact invariance between assessments; the observed residual remains diagnostic and does not become a configured promise.

**Observation (One publication carries the equality oracle)** · `obs:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-publication-control`

Equality across moving publications would compare training progress as well as feature direction. The label barrier settles training first, and the owner block then keeps both final assessments on one immutable publication; matching snapshot versions are asserted rather than inferred from call order.

## Acceptance · `sec:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-acceptance`

The integration target `posterior_convergence` contains `posterior_convergence::sentinel_specific_uncertainty_does_not_wake_the_anchor`, whose documentation cites the intent and the claim, and its module index states the matched shared projection, exact weight invariance and strict public-uncertainty contrast.

The guarded setup reports symmetric completed bootstraps, an in-service coordinate system, all playback rows and eligible labels applied, and clean health before the result oracle runs. The result reports equal snapshot versions and ages, one reporting Sentinel per assessment, bit-identical anchor weights, a common weight below $0.3$, true anchor convergence, and starved uncertainty above fed uncertainty by more than `World::tol().default`.

The deliberate full-form substitution fails both the existing seeded component witness and the public-path weight equality, while the restored implementation passes the package tests and corpus check.

The generated test catalogue agrees with the module index, the intent is no longer listed as unwitnessed, and no production surface, shared probe, oracle, fixture or dependency was added.
