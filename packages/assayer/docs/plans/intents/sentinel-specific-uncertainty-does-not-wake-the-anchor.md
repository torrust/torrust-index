# Keeping Sentinel-Local Uncertainty out of the Anchor Weight · `plan:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor`

Keeping (´claim:risk:sentinel-specific-uncertainty-does-not-wake-the-anchor´) establishes that a request can remain uncertain in one Sentinel's private feature block without transferring any more of its estimate to a model that cannot represent that block.

## What the promise says, precisely · `sec:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-promise`

The blend compares the anchor variance $\sigma^2_\text{anc}(\tilde{\phi})$ with the sister variance $\sigma^2_{\text{inh},S_g}(\phi)$ restricted to the gathered coordinates both models share, and sets $w(\phi)=\left(1-\sigma^2_\text{anc}/(\sigma^2_{\text{inh},S_g}+\varepsilon)\right)^+$ (´def:risk:subspace-blend´).

For two probes $F$ and $S$ read from one model snapshot, the measurable precondition is $\tilde{\phi}_F=\tilde{\phi}_S$: they carry the same bias, aggregate values, cross-dimension values, measurement-only maximum, and reporting-Sentinel count. Their full vectors differ only by moving the same extraction between a well-fed Sentinel slot and a label-starved Sentinel slot.

The kept invariant is then $w(\phi_F)=w(\phi_S)$ even while the public full-direction uncertainty satisfies $\sigma_\text{eff}(\phi_S)>\sigma_\text{eff}(\phi_F)$. The first equality keeps the exclusion; the second inequality proves that the stimulus created uncertainty rather than two interchangeable requests.

“Does not wake” means both probes remain in the sister-dominated regime $w<0.3$, whose boundary is fixed by (´def:platt:regimes´), and both report `anchor_converged = true`. The approximately two-to-ten-per-cent residual described by (´rem:risk:blend-mechanisms´) is printed as a diagnostic rather than asserted as a universal interval, because its exact magnitude is deployment-specific (´cav:limitation:anchor-floor´).

The anchor cannot read either Sentinel slot. Its fixed fifteen-position projection contains only the shared sources enumerated by (´def:dimension:anchor-projection´), and the anchor model remains fixed while the full model carries every dynamic slot (´def:risk:anchor-model´).

The specification states the same property as the invariant that activation is insensitive to Sentinel-specific and axis-specific uncertainty (´inv:guarantee:blend-subspace´). The calibration record makes the restriction structural: the projected quadratic form indexes only anchor coordinates (´dec:calibration:indexed-form´), and no off-subspace component is ever read (´cor:calibration:subspace-containment´).

Elapsed time is not a hidden threshold. The weight uses raw forms because the common time correction cancels from its ratio (´dec:calibration:raw-weight-forms´); the virtual clock remains fixed throughout the witness.

This promise is covered by the standing gap for behavioural anchor coverage (´entry:assayer:gap-anchor-behaviour´).

## What the code offers today · `sec:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-current-code`

The blend implementation already computes all three relevant forms: full sister variance, shared-subspace sister variance, and anchor variance. `compute_blend` uses only the latter two for `anchor_weight` and uses the full sister variance in the effective mixture, matching the distinction the witness observes.

The nearby unit witnesses establish that indexed projection equals an extracted submatrix (´test:unit:projected-qf-matches-submatrix´), that the diagnostic shared form does not exceed the full form (´test:unit:variance-diagnostics-populated´), and that genuine shared-subspace uncertainty gives the anchor nearly all the weight (´test:unit:anchor-only-w-near-one´). None varies only an excluded coordinate while holding the public assessment path and shared projection fixed.

The public risk basis carries the needed outputs: `anchor_weight`, `sigma_eff`, `anchor_converged`, `n_sentinels_reporting`, and per-assessment degradation. That is enough to observe the promised behavior without exposing covariance or adding a production field to the risk-basis schema (´schema:risk:basis´).

The world harness supplies named Sentinel registration, report ingestion, `request_with_sentinel`, `cycle_request`, label and observation barriers, and `block_model_owner_for_test`. These pieces belong to the completed skeleton (´entry:assayer:harness-stage-skeleton´) and can hold the final pair on one publication.

The golden-report fixture supplies `golden_report` and `GOLDEN_COORD`. The report's readings are authored discriminating stimulus (´dec:assayer:golden-report-stimulus´), so ingesting the same report for both Sentinels makes their shared aggregate inputs equal by construction.

The label builder supplies alternating benign and adverse `Allow` labels. Both sister and anchor receive those unconfounded outcomes under (´tab:eligibility:training´), while only the reporting Sentinel's private block is nonzero in each stored training vector.

The closest public-path convergence test already drives fixed-seed assess-label cycles (´test:integration:scalar-signal-population-split-converges-directionally´). The closest two-Sentinel fixture already places two separately routed slots in one labeled stream (´test:integration:association-separates-a-keyed-sentinel-from-a-hash-fed-one´); the new witness simplifies that geometry by giving both slots identical reports and starving one of labels entirely.

## The witness · `sec:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-witness`

The integration test belongs beside the existing public-API convergence smoke (´test:integration:scalar-signal-population-split-converges-directionally´), because its setup is a deterministic evidence-convergence contrast rather than a lifecycle or extraction-layout test.

The setup builds one seeded `World` with the default channel, registers Sentinels `fed` and `starved` before learning, ingests the same golden report for each, and forms two requests with the same entity and `GOLDEN_COORD`, each naming exactly one of the Sentinels.

The setup first settles the cold full-vector ramp across both requests under (´alg:standardisation:batch-initialisation´), then makes at least one hundred unlabelled assessments through each Sentinel and flushes the resulting completion commands. This completes both per-Sentinel bootstraps under (´alg:standardisation:sentinel-bootstrap´) before starvation begins, so the measured difference is posterior evidence rather than one slot still using its standardisation priors.

The stimulus sends 1,500 alternating benign and adverse eligible labels through the `fed` request only. Alternation holds the target mean near zero so the final uncertainty contrast is not manufactured by a large anchor-sister point-estimate disagreement, while the repeated shared projection concentrates both models along the direction the probes ask about.

After the last label barrier, `block_model_owner_for_test` parks publication while the final `fed` and `starved` requests are assessed. The shared projection, report age, model parameters, reporting count, clock, entity, and coordinate are then identical; only which Sentinel slot carries the extraction differs.

The observations are the two public risk bases. The test records each `anchor_weight`, `sigma_eff`, `anchor_converged`, reporting count, standardisation phase and snapshot version, together with the compact degradation summary.

The primary assertion compares `anchor_weight.to_bits()` for exact equality. Both calls execute the same raw quadratic forms over the same shared vector and snapshot, so a tolerance would admit influence from a coordinate the calculation is defined not to read.

The regime assertion requires the common weight to be below $0.3$, both `anchor_converged` flags to be true, and both reporting counts to equal one. This establishes a learned, sister-dominated shared direction rather than obtaining a low weight vacuously from two untouched priors.

The starvation assertion requires `starved.sigma_eff` to exceed `fed.sigma_eff` by more than the harness's `default` tolerance from (´tab:assayer:harness-scenario-tolerances´). It is a non-vacuity guard on the contrast, not an oracle for either component covariance.

The health assertion requires finite risk fields and no numerical fallback, sanitisation, dropped event, or incomplete standardisation state. The test prints both weights, both uncertainties, both convergence flags and the snapshot version so every failed premise is visible.

The fails-before replaces the projected sister quadratic form in `compute_blend` with the full sister quadratic form while leaving the mixture unchanged. The starved probe then presents the larger denominator, earns a larger anchor weight than the fed probe, and fails the exact-equality assertion; a sufficiently strong fixture also moves it toward or across the $0.3$ anchor-regime boundary.

## What is missing · `sec:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-missing`

**Entry (A matched two-Sentinel starvation fixture)** · `entry:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-matched-starvation-fixture`

A test-local fixture registers the matched reports, completes both standardisation paths symmetrically, emits the fixed alternating label stream through one slot, and returns the two same-publication risk bases. Estimate: approximately 55–80 lines in the convergence integration module beside (´test:integration:scalar-signal-population-split-converges-directionally´); it depends only on the completed harness skeleton (´entry:assayer:harness-stage-skeleton´), not on the open tape or converged-world roadmap.

**Entry (The public-path exclusion witness)** · `entry:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-public-witness`

The labelled test, its module index row, the shared-projection checks, the uncertainty non-vacuity guard, the sister-regime threshold, and diagnostic output complete the witness. Estimate: approximately 35–55 lines in that integration module; it depends on the matched starvation fixture and adds no production or reusable harness API.

## Risks and open questions · `sec:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-risks`

**Observation (Standardisation can impersonate starvation)** · `obs:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-standardisation-confound`

A starved bootstrap and a starved posterior are different deficits. Completing both Sentinels' bootstraps from identical unlabelled requests before the one-sided label stream removes the first from the witness and leaves only the second.

**Observation (The promise is directional)** · `obs:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-directional-precondition`

The quadratic forms estimate uncertainty along the request vector, not convergence of every direction in the shared covariance matrix. Repeating one shared projection is sufficient for this probe and establishes nothing about an orthogonal shared direction.

**Observation (The public uncertainty is a non-vacuity check)** · `obs:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-public-uncertainty-limit`

`sigma_eff` is a tempered mixture that includes model disagreement, not the raw full sister form. Alternating targets suppress the disagreement term, and the mutation control proves which denominator governs the weight; adding an internal covariance accessor would make the test more coupled without strengthening the public behavior it keeps.

**Observation (Low has a regime boundary, not a universal floor)** · `obs:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-low-threshold`

The specification fixes $0.3$ as the sister-to-anchor regime boundary and describes the smaller residual as deployment-dependent. The binding threshold is therefore $w<0.3$ plus exact invariance between probes; the observed residual remains diagnostic and does not become a new configured promise.

**Observation (One publication carries the equality oracle)** · `obs:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-publication-control`

An equality across independently moving snapshots would compare training progress as well as feature direction. Parking the owner only around the final pair makes both risk bases readings of one immutable publication and leaves all training on the ordinary public path.

## Acceptance · `sec:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor-acceptance`

The implementation report names the integration test and its citation of the intent, shows both snapshot versions and reporting counts, and gives the fed and starved values for `anchor_weight`, `sigma_eff`, and `anchor_converged`.

It shows exact anchor-weight equality, a common weight below $0.3$, a starved uncertainty above the fed uncertainty by more than the declared harness tolerance, completed standardisation, and clean health on the fixed seed.

It shows that replacing the projected sister form with the full sister form makes the test fail at the weight equality, then shows the restored implementation green under the package's debug, release, documentation, no-default-feature, formatting, clippy, corpus-lint, and coverage checks.

The coverage evidence shows that the intent is no longer unwitnessed, the test module index and generated integration catalogue agree with the function, and no broader harness or production surface was introduced.
