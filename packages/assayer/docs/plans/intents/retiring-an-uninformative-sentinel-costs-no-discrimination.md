# Retiring an uninformative Sentinel costs no discrimination · `plan:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination`

Keeping (´claim:wellness:retiring-an-uninformative-sentinel-costs-no-discrimination´) establishes that an observed below-threshold removal cost predicts the result of the corresponding lifecycle action: destructive deregistration leaves the model's held-out rank discrimination no worse than it was before the Sentinel left.

## What the promise says, precisely · `sec:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-precise-promise`

For a Sentinel slot $\mathcal{B}$, the operational score is $\rho=\hat\varphi\cdot\mu$, the slot supplies $\rho_{\mathcal{B}}$, and the contribution reading is the decayed, importance-weighted excess logistic loss of $\rho-\rho_{\mathcal{B}}$ over the loss of $\rho$ (´def:monitoring:slot-contribution´).

The reading becomes present only after the block reaches an effective sample size of thirty. Its retirement edge is the specification constant $0.10$ of the score's loss, and the qualifying precondition is strict: $C(\mathcal{B})<0.10$ (´tab:monitoring:observability-summary´).

Negative contribution remains below the edge and means that the slot was harming the score; the witness therefore compares the reading directly with $0.10$ rather than comparing its magnitude.

Discrimination is tie-corrected rank AUC over prediction-outcome pairs (´alg:monitoring:auc´). For one fixed balanced holdout population, let $D_{\mathrm{before}}$ be AUC from risks scored immediately before deregistration and $D_{\mathrm{after}}$ the AUC from the same observations scored after the lifecycle publication.

The measurable promise is one-sided because its operative statement is that the model gets no worse: $D_{\mathrm{after}} + \epsilon_{\mathrm{auc}} \geq D_{\mathrm{before}}$. An improvement is not a failure, while a loss beyond the harness AUC tolerance is.

The default harness tolerance is $\epsilon_{\mathrm{auc}}=0.005$, declared specifically for agreement between two AUC readings of one sample (´tab:assayer:harness-scenario-tolerances´). The witness also requires both AUCs above $0.65$ and the pre-removal AUC below $1-\epsilon_{\mathrm{auc}}$, so preserved discrimination is useful and is not hidden at the statistic's ceiling; the lower guard follows the existing public convergence witness rather than introducing a system-wide alert threshold (´test:integration:scalar-signal-population-split-converges-directionally´).

Deregistration identifies the Sentinel slot and every interaction involving it, applies the regularised Schur complement to each full feature-space model, removes standardisation and routing state, and rebuilds the dimension map (´alg:registry:sentinel-deregistration´). The numerical lifecycle is an approximation to exact marginalisation and may fall back to the kept block (´alg:gaussian:regularised-schur´), so the test observes the end-to-end risk ordering rather than substituting an exact-matrix claim.

The public destructive operation is the relevant stimulus: it marginalises the entity and archives nothing (´dec:construction:destructive-deregistration´). Hibernation would add a future restoration promise that this witness does not need.

## What the code offers today · `sec:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-code-offers`

The scenario builder provides `scenario_with`, and the deterministic testing world provides `World`, named Sentinel registration, report ingestion, request construction, assessment and derivation, health access, named tolerances, and destructive `World::deregister_sentinel`.

Every `World` lifecycle verb ends on the model-owner publication barrier. This closes the production surface's two-phase visibility interval (´dec:construction:two-phase-visibility´), so the post-removal probe begins from the reduced model without polling or sleeping.

The integration-test module for slot legibility already contains the keyed and hash-fed `Surface` fixtures, deterministic avalanche streams, `ingest_surface`, and a fifteen-hundred-label public assess-then-label driver. Its nearest witness proves that the hash-fed slot's removal-cost reading is a small fraction of the keyed slot's (´test:integration:contribution-separates-a-sentinel-the-score-leans-on-from-a-hash-fed-one´), but it neither asserts the absolute $0.10$ precondition nor performs deregistration.

The public convergence witness already scores unlabelled held-out requests through `World::derive_for_request` and computes tie-corrected pairwise AUC (´test:integration:scalar-signal-population-split-converges-directionally´). Its AUC helper is local to that integration binary and is not currently reusable from the slot-legibility integration module.

The public lifecycle-algebra witness proves the name-based registration and destructive deregistration round trip (´test:integration:register-deregister-round-trip´). It observes identity and health bookkeeping rather than prediction ordering.

The completed skeleton supplies every engine-facing verb, the seeded randomness, the barrier, and the tolerance bundle needed for this witness (´entry:assayer:harness-stage-skeleton´). No clock travel, liveness loop, drift budget, internal posterior probe, long-streaming tape, or new production API is required.

The health report exposes the target Sentinel's `contribution` before removal, but its aggregate discrimination is cached at calibration refit and remains unchanged until another refit (´dec:health:cached-discrimination´). A post-removal health query therefore cannot be the outcome observation; fresh held-out assessments provide it.

The standing testing plan has no gap entry covering this promise. Its completed skeleton is sufficient, and the missing work is local fixture composition and the integration assertion rather than a new roadmap stage.

## The witness · `sec:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-witness`

The integration test belongs beside the witness that already establishes the hash-fed contribution reading (´test:integration:contribution-separates-a-sentinel-the-score-leans-on-from-a-hash-fed-one´), in the same integration-test module, and adds its own module-index row and test mint when implemented.

Setup builds the existing two-Sentinel scenario under its fixed seed, registers the keyed and hash-fed surfaces with no interaction templates, ingests their reports, and drives the existing balanced deterministic training stream until the contribution reading is present.

The precondition observation reads one published full-health report, extracts the hash-fed Sentinel's contribution, and asserts $C_{\mathrm{hash}}<0.10$. It records the keyed contribution, both association readings, and the cumulative Schur-correction skip count as diagnostics without turning their relative ordering into another promise.

The holdout fixture contains sixty-four adverse and sixty-four benign cases. The keyed route agrees with each class for three cases out of four and reverses for the fourth, while the hash-fed route comes from an independent avalanche; this creates overlap by construction and prevents the AUC ceiling from concealing a changed ordering.

The first observation scores the whole holdout without labelling it, attaching both Sentinel coordinates exactly as the training surface expects, and computes $D_{\mathrm{before}}$ from `assessment.risk.p_bad` with pairwise tie correction.

The stimulus is one successful `World::deregister_sentinel` call naming the hash-fed Sentinel. The verb's publication barrier completes before the test confirms that the name no longer resolves and begins the second probe.

The second observation scores the identical holdout in the identical order with the keyed coordinate still attached and the retired coordinate omitted. No label lands between or during the two probes, so model learning, calibration, contribution accumulation, and virtual time remain fixed across the comparison.

The assertion requires the contribution precondition, $D_{\mathrm{before}}>0.65$, $D_{\mathrm{before}}<1-\epsilon_{\mathrm{auc}}$, $D_{\mathrm{after}}>0.65$, and $D_{\mathrm{after}}+\epsilon_{\mathrm{auc}}\geq D_{\mathrm{before}}$. It separately requires the cumulative Schur-correction skip count to remain unchanged across deregistration and applies the existing clean-health assertion, so a preserved AUC cannot conceal marginalisation fallback or assessment degradation.

The failure message prints the seed, training length, class counts, contribution, both AUCs, their signed difference, $\epsilon_{\mathrm{auc}}$, and the Schur-skip delta, making a moved threshold, fallback, dead fixture, and genuine discrimination loss distinguishable.

A deliberately broken deregistration that reinitialises surviving weights, removes the keyed positions through a stale map, or publishes the reduced registry before the reduced model drives $D_{\mathrm{after}}$ towards ties or reversals and violates the one-sided AUC bound. The fails-before is this expected behavior of a weakened implementation; no mutation is executed as part of the test.

## What is missing · `sec:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-missing`

**Entry (The trained pair remains available across retirement)** · `entry:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-trained-pair-fixture`

Refactor approximately forty to sixty lines around the existing hash-fed contribution witness (´test:integration:contribution-separates-a-sentinel-the-score-leans-on-from-a-hash-fed-one´) so the driver can return the live `Scenario` and both Sentinel identifiers together with its readings instead of dropping the world after the health query. Existing association and contribution tests continue to consume the same training path. This entry depends on (´entry:assayer:harness-stage-skeleton´) and has no sibling-lane dependency.

**Entry (A paired held-out AUC probe witnesses the lifecycle cost)** · `entry:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-paired-auc-probe`

Add approximately sixty to ninety lines beside the existing hash-fed contribution witness (´test:integration:contribution-separates-a-sentinel-the-score-leans-on-from-a-hash-fed-one´) for the balanced overlapping holdout descriptors, a local pairwise-AUC fold, the before/deregister/after test, its statement, and its test-index row. The helper remains local because this plan needs one statistic in one binary; promotion to the shared testing assertions becomes justified only when another integration binary needs the same implementation. This entry depends on (´entry:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-trained-pair-fixture´) and has no sibling-lane dependency.

## Risks and open questions · `sec:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-risks`

**Observation (Loss contribution does not bound rank movement analytically)** · `obs:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-loss-versus-rank`

The contribution reading is a ratio of excess cross-entropy while AUC depends only on pair ordering. An arbitrarily small score movement can reverse a nearly tied pair, so $C<0.10$ supplies no mathematical AUC bound by itself. The fixed overlapping holdout is therefore the empirical witness the intent lacks, not a proof for every possible population.

**Observation (The comparison is one-sided)** · `obs:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-one-sided-bound`

“No worse” permits deregistration to improve ranking by removing noise. An absolute-difference assertion would reject that valid outcome and turn the promise into score invariance, while the one-sided tolerance measures exactly the discrimination cost named by the title.

**Observation (Cached health discrimination cannot observe the stimulus)** · `obs:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-cached-discrimination`

The full-health AUC describes the last calibration refit and does not move merely because the lifecycle changed (´dec:health:cached-discrimination´). Driving fresh unlabelled holdouts is necessary; forcing enough post-removal labels to trigger another refit would train a different model and confound the removal cost with recovery.

**Observation (The fixture guards against a ceiling witness)** · `obs:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-nontrivial-holdout`

The existing keyed training relation is perfectly class-aligned and can produce AUC one, where unchanged AUC says little about small ordering damage. The three-in-four holdout relation guarantees adverse and benign populations on both keyed routes, and the explicit pre-removal ceiling guard fails if incidental hash ordering still makes the measured fixture total.

**Observation (AUC resolution is finer than its declared tolerance)** · `obs:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-auc-resolution`

Sixty-four cases per class produce four thousand ninety-six cross-class comparisons, so one changed pair moves AUC by less than $0.005$. The declared tolerance can absorb a few boundary inversions without becoming too narrow to observe the statistic; a smaller holdout would quantise AUC too coarsely for the same bound.

**Observation (The block reading and lifecycle scope are deliberately not collapsed)** · `obs:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-block-scope`

Contribution is read over the Sentinel's own slot, while deregistration also removes configured interactions and changes the surviving aggregate inputs (´alg:registry:sentinel-deregistration´). The fixture leaves interactions empty but retains the real aggregate path, and the post-removal AUC is the check that effects outside the measured slot do not invalidate the retirement decision.

**Observation (Publication ordering replaces timing)** · `obs:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-publication-ordering`

The production lifecycle is two-phase, but `World::deregister_sentinel` settles the model-owner publication before returning. The witness relies on that barrier and fixed virtual time; sleeps, retries, and wall-clock deadlines would add scheduler variance without observing a stronger condition.

## Acceptance · `sec:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-acceptance`

The implementation report names the integration-test mint and path, the fixed seed and training length, the exact adverse and benign holdout counts, the target contribution and strict threshold, both AUCs, their signed difference, and the declared AUC tolerance.

The report shows the target health reading was present and below $0.10$ before removal, the pre-removal AUC was useful and below the ceiling, the hash-fed name stopped resolving after the settled destructive operation, the post-removal AUC remained useful, and the one-sided no-loss assertion passed with clean health.

The report includes the fails-before analysis: a deliberately weakened deregistration that resets or misaddresses surviving weights drives the paired holdout beyond the AUC loss bound. The mutation is described and is not executed as part of the test.

The report records repeated deterministic runs and every named package gate, confirms that no label or clock advance occurred between the paired probes, accounts for the approximate additions under both entries, and reports any change required to the proposed fixture guards rather than silently tuning them.
