# Retiring an uninformative Sentinel costs no discrimination · `plan:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination`

Keeping (´claim:wellness:retiring-an-uninformative-sentinel-costs-no-discrimination´) establishes that an observed below-threshold removal cost predicts the result of the corresponding lifecycle action: destructive deregistration leaves the model's held-out rank discrimination no worse than it was before the Sentinel left.

## What the promise says, precisely · `sec:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-precise-promise`

For a Sentinel slot $\mathcal{B}$, the operational score is $\rho=\hat\varphi\cdot\mu$, the slot supplies $\rho_{\mathcal{B}}$, and the contribution reading is the decayed, balancing-weighted excess logistic loss of $\rho-\rho_{\mathcal{B}}$ over the loss of $\rho$ (´def:monitoring:slot-contribution´).

The reading is absent until the block reaches an effective sample size of thirty. The contribution alert band's lower edge is $0.10$ of the score's loss, and the claim's qualifying precondition is strict: $C(\mathcal{B})<0.10$. The observability summary classifies that reading as confirmation of a sustained association alert per Sentinel, so this witness tests the claim's contribution-to-discrimination implication without recasting $0.10$ as a standalone system alert (´tab:monitoring:observability-summary´).

Negative contribution remains below the edge and means that the slot was harming the score; the witness therefore compares the signed reading directly with $0.10$ rather than comparing its magnitude.

Discrimination is tie-corrected rank AUC over prediction-outcome pairs (´alg:monitoring:auc´). For one fixed balanced holdout population, let $D_{\mathrm{before}}$ be AUC from risks scored immediately before deregistration and $D_{\mathrm{after}}$ the AUC from the same observations scored after lifecycle publication.

The measurable promise is one-sided because its operative statement is that the model gets no worse: $D_{\mathrm{after}} + \epsilon_{\mathrm{auc}} \geq D_{\mathrm{before}}$. An improvement is not a failure, while a loss beyond the declared AUC agreement budget is.

The harness declares $\epsilon_{\mathrm{auc}}=0.005$ for agreement between two AUC readings of one sample (´tab:assayer:harness-scenario-tolerances´). The fixture also requires both AUCs above $0.65$ and the pre-removal AUC below $1-\epsilon_{\mathrm{auc}}$: $0.65$ is the specification's lower edge of moderate rank discrimination, already exercised by the public convergence witness, while the upper guard prevents a ceiling result from hiding changed ordering (´tab:monitoring:interpretation´) (´test:integration:scalar-signal-population-split-converges-directionally´).

Deregistration identifies the Sentinel slot and every interaction involving it, applies the regularised Schur complement to each full feature-space model, removes standardisation and routing state, and rebuilds the dimension map (´alg:registry:sentinel-deregistration´). The numerical lifecycle is an approximation to exact marginalisation and may fall back to the kept block (´alg:gaussian:regularised-schur´), so the test observes end-to-end risk ordering and separately refuses a newly skipped Schur correction rather than substituting an exact-matrix claim.

The public destructive operation is the relevant stimulus: it marginalises the entity and archives nothing (´dec:construction:destructive-deregistration´). Hibernation would add a future restoration promise that this witness does not need.

## What the code offers today · `sec:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-code-offers`

The finished harness has one `Scenario` over one `World`; `scenario_with`, named registration, report ingestion, request construction, `World::derive_for_request`, `World::sentinel`, named tolerances, and `World::deregister_sentinel` are present on that shared surface (´dec:harness:single-scenario´) (´tab:assayer:harness-implementation-library-roster´).

`World::deregister_sentinel` crosses the model-owner publication barrier before returning, closing the production API's two-phase visibility interval for the scenario. `World::flush_observations` and `World::settle_cold_ramp_with` separately settle the cold-ramp queue; neither elapsed time nor a caller-authored poll is needed (´dec:construction:two-phase-visibility´) (´dec:harness:no-ad-hoc-waits´).

Long training stimuli now have `PlaybackRow`, `PlaybackBarrierPolicy`, `PlaybackProgress`, `PlaybackBatchSize`, and `playback`. A policy containing `PlaybackBarrier::FlushLabels` settles each training row before the next boundary and before any checkpoint, so a handwritten assess-label-flush loop is no longer the harness contract (´dec:harness:declarative-playback´) (´test:crate:selected-label-barrier-completes-before-checkpoint´).

The shared oracle `pairwise_rank` computes class-gated, tie-aware rank discrimination by a route independent of the production sort and rank-sum implementation (´dec:harness:oracle-tier´). It replaces the inline pair enumeration in the public convergence witness and the local fold this plan previously proposed.

`TrainedStateFixture` and `TrainedStateBaseline` provide the guarded-fixture shape for the score-verified convergence population, but they do not register Sentinel surfaces or establish the below-threshold contribution precondition this witness needs. A Sentinel-specific fixture must follow the same separation: its guard admits the trained starting state, while `pairwise_rank` judges the post-deregistration result (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

The owned probes `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView` are available under the finished probe contract, but none is needed: the witness reads public owned risk values and one public owned full-health report rather than internal posterior state (´dec:harness:probe-contract´) (´tab:assayer:harness-implementation-library-roster´). The test uses one fixed reproducible seed rather than a seed sweep, so `run_seeded_sweep` has no role.

The slot-legibility target still owns private `Surface` fixtures, `ingest_surface`, and the deterministic keyed and hash-fed streams. Its contribution witness proves only that the hash-fed reading is a fraction of the keyed reading; it neither asserts $C(\mathcal{B})<0.10$ nor retains the live scenario for deregistration (´test:integration:contribution-separates-a-sentinel-the-score-leans-on-from-a-hash-fed-one´).

The public convergence witness scores unlabelled held-out requests through `World::derive_for_request`, but its pairwise AUC calculation remains inline (´test:integration:scalar-signal-population-split-converges-directionally´). The shared `pairwise_rank` oracle is the reusable implementation for this witness.

The lifecycle-algebra witness proves name-based registration and destructive deregistration but observes identity and health bookkeeping rather than prediction ordering (´test:integration:register-deregister-round-trip´). The `lifecycle_identity_convergence` target remains empty scaffolding, and no current test mints or cites the claim this plan keeps.

The public full-health report exposes the target Sentinel's contribution before removal and the cumulative Schur-correction skip count, but its aggregate discrimination is cached at calibration refit and remains unchanged until another refit (´dec:health:cached-discrimination´). Fresh held-out risks, not a post-removal health AUC, are the result observation.

## The witness · `sec:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-witness`

The cargo integration target is `lifecycle_identity_convergence`, because Sentinel retirement is the witness's principal subject under the convergence split (´dec:harness:convergence-split´). The test function is `lifecycle_identity_convergence::retiring_an_uninformative_sentinel_costs_no_discrimination`.

Setup constructs the existing keyed and hash-fed Sentinel surfaces in one `Scenario`, registers both with no interaction templates, ingests both reports, and calls `World::settle_cold_ramp_with` on representative requests before training. That named harness operation crosses `World::flush_observations` until the published standardisation phase is in service, preventing either held-out arm from measuring cold-ramp motion.

Training is an ordered sequence of typed rows carrying the existing deterministic outcome bit, keyed coordinate, independent hash-fed coordinate, request identity, and `LabelSpec`. Each row's `PlaybackRow::play` derives and submits its label without choosing a wait; `playback` runs the rows under `PlaybackBarrierPolicy` containing `PlaybackBarrier::FlushLabels`, with a declared bounded `PlaybackBatchSize` and `PlaybackProgress` retained for failure diagnostics (´dec:harness:declarative-playback´).

Before the fixture returns, its guard loads one owned full-health report, requires both contribution readings to be present, requires the hash-fed value to satisfy $C_{\mathrm{hash}}<0.10$, and records both association readings and the cumulative Schur-correction skip count. The guard then scores the pre-removal holdout, computes $D_{\mathrm{before}}$ with `pairwise_rank` using the configured minimum class count and zero tie tolerance, and requires $0.65 \lt D_{\mathrm{before}} \lt 1-\epsilon_{\mathrm{auc}}$; result validation remains outside the fixture (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

The holdout contains sixty-four adverse and sixty-four benign descriptors. The keyed route agrees with each class on three cases out of four and reverses on the fourth, while the hash-fed route comes from an independent avalanche; this creates overlap by construction and prevents a ceiling AUC from concealing changed ordering.

The first observation scores every descriptor without labelling it, attaches both Sentinel coordinates exactly as training does, and reads `RiskBasis::p_bad`. The fixture stores those risks and their class labels so the result comparison cannot silently regenerate a different pre-removal population.

The stimulus is one successful `World::deregister_sentinel` call naming the hash-fed Sentinel. The verb's publication barrier completes before the test requires `World::sentinel` to stop resolving that name and begins the second observation.

The second observation scores the same descriptors in the same order with the keyed coordinate still attached and the retired coordinate omitted, then computes $D_{\mathrm{after}}$ through `pairwise_rank` with the same class gate and tie rule. No label lands and scenario time does not move between the two observations, so model learning, calibration, contribution accumulation, and decay remain fixed across the comparison.

The result requires $D_{\mathrm{after}}>0.65$ and $D_{\mathrm{after}}+\epsilon_{\mathrm{auc}}\geq D_{\mathrm{before}}$. It also requires `SystemHealthReport::schur_corrections_skipped` to equal the fixture's pre-removal count and applies `assert_health_clean`, so a preserved AUC cannot conceal marginalisation fallback or assessment degradation.

The failure message prints the seed, completed training rows, class counts, contribution, both AUCs, their signed difference, $\epsilon_{\mathrm{auc}}$, and the Schur-skip delta, distinguishing a moved threshold, incomplete fixture, fallback, and genuine discrimination loss.

A deliberately weakened deregistration that reinitialises surviving weights, removes the keyed positions through a stale map, or publishes the reduced registry before the reduced model drives $D_{\mathrm{after}}$ towards ties or reversals and violates the one-sided AUC bound. This is the fails-before analysis; the test does not execute a mutation.

## What is missing · `sec:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-missing`

**Entry (The trained pair remains available across retirement)** · `entry:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-trained-pair-fixture`

Add a public guarded Sentinel-pair fixture and its measured baseline to `testing::fixtures`, then re-export both through the gated testing roster. The fixture owns the live `Scenario`, both Sentinel identifiers, the fixed holdout descriptors, the pre-removal risks, and a baseline containing training progress, class counts, both contribution and association readings, pre-removal AUC, and the Schur-skip count. Its constructor factors the existing `Surface` and `ingest_surface` setup out of the slot-legibility target, drives typed training rows through `playback`, settles the cold ramp through `World::settle_cold_ramp_with`, and refuses with the complete measured baseline unless contribution presence, the strict $0.10$ edge, class gates, useful AUC, and the non-ceiling guard all hold. Existing slot-legibility tests consume the shared setup rather than retaining a second training path. This entry depends only on the finished playback, oracle, barrier, and guarded-fixture contracts (´dec:harness:declarative-playback´) (´dec:harness:oracle-tier´) (´dec:harness:no-ad-hoc-waits´) (´dec:harness:guarded-fixtures´).

**Entry (A paired held-out AUC probe witnesses the lifecycle cost)** · `entry:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-paired-auc-probe`

Add `lifecycle_identity_convergence::retiring_an_uninformative_sentinel_costs_no_discrimination` to the `lifecycle_identity_convergence` integration target with its module-index row and test mint. It consumes the guarded fixture, deregisters the hash-fed Sentinel through `World::deregister_sentinel`, rescans the retained holdout descriptors, calls `pairwise_rank`, compares the paired AUCs using the `Tolerances::auc` value returned by `World::tol`, verifies name removal and an unchanged Schur-skip count, and applies `assert_health_clean`. No local AUC helper, wait loop, clock movement, model probe, seed sweep, or production API is missing. This entry depends on (´entry:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-trained-pair-fixture´).

## Risks and open questions · `sec:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-risks`

**Observation (Loss contribution does not bound rank movement analytically)** · `obs:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-loss-versus-rank`

The contribution reading is a ratio of excess cross-entropy while AUC depends only on pair ordering. An arbitrarily small score movement can reverse a nearly tied pair, so $C<0.10$ supplies no mathematical AUC bound by itself. The fixed overlapping holdout is therefore the empirical witness the intent lacks, not a proof for every possible population.

**Observation (The comparison is one-sided)** · `obs:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-one-sided-bound`

“No worse” permits deregistration to improve ranking by removing noise. An absolute-difference assertion would reject that valid outcome and turn the promise into score invariance, while the one-sided tolerance measures exactly the discrimination cost named by the title.

**Observation (Cached health discrimination cannot observe the stimulus)** · `obs:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-cached-discrimination`

The full-health AUC describes the last calibration refit and does not move merely because the lifecycle changed (´dec:health:cached-discrimination´). Fresh unlabelled holdout risks are necessary; forcing enough post-removal labels to trigger another refit would train a different model and confound removal cost with recovery.

**Observation (The fixture guards against a ceiling witness)** · `obs:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-nontrivial-holdout`

The existing keyed training relation is perfectly class-aligned and can produce AUC one, where unchanged AUC says little about small ordering damage. The three-in-four holdout relation puts both classes on both keyed routes, and the fixture's measured pre-removal lower and upper guards refuse a dead, near-chance, or ceiling setup before deregistration is judged.

**Observation (AUC resolution is finer than its declared tolerance)** · `obs:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-auc-resolution`

Sixty-four cases per class produce four thousand ninety-six cross-class comparisons, so one changed pair moves AUC by $1/4096$, below $0.005$. The declared agreement budget therefore admits about twenty boundary inversions while remaining much narrower than the fixture's distance from chance; changing either population size requires re-deriving this resolution rather than fitting a new tolerance.

**Observation (The block reading and lifecycle scope are deliberately not collapsed)** · `obs:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-block-scope`

Contribution is read over the Sentinel's own slot, while deregistration also removes configured interactions and changes the surviving aggregate inputs (´alg:registry:sentinel-deregistration´). The fixture declares no interactions but retains the real aggregate path, and the post-removal AUC checks that effects outside the measured slot do not invalidate the retirement decision.

**Observation (Publication ordering replaces timing)** · `obs:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-publication-ordering`

`World::deregister_sentinel` settles model-owner publication but does not drain the cold-ramp observation queue. The guarded setup therefore reaches the in-service standardisation phase through `World::settle_cold_ramp_with` before either paired reading; after that boundary the witness relies on the lifecycle barrier and fixed scenario time, and adds no sleep, retry, deadline, or poll (´dec:harness:no-ad-hoc-waits´).

## Acceptance · `sec:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination-acceptance`

The implementation report names cargo integration target `lifecycle_identity_convergence` and test function `lifecycle_identity_convergence::retiring_an_uninformative_sentinel_costs_no_discrimination`, the fixed seed and completed training-row count, the adverse and benign holdout counts, the target contribution and strict threshold, both AUCs, their signed difference, and the `Tolerances::auc` value returned by `World::tol`.

The report shows that the guarded fixture settled the cold ramp, completed playback, observed both contribution readings, admitted the hash-fed reading below $0.10$, and admitted useful non-ceiling pre-removal AUC before exposing the live scenario to the result assertion.

The report shows the hash-fed name stopped resolving after the settled destructive operation, the post-removal AUC remained above $0.65$, the one-sided no-loss assertion passed, the Schur-skip count did not increase, and `assert_health_clean` passed.

The report includes the fails-before analysis: a deliberately weakened deregistration that resets or misaddresses surviving weights drives the paired holdout beyond the AUC loss bound. The mutation is described and is not executed as part of the test.

The report records repeated deterministic runs and every named package gate, confirms that no label or scenario-time movement occurred between the paired observations, and reports any changed fixture guard as a changed precondition rather than silently tuning it.
