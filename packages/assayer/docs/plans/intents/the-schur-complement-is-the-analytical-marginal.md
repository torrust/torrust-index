# A Live Witness for the Schur Marginal · `plan:assayer:intent-the-schur-complement-is-the-analytical-marginal`

Keeping (´claim:bayes:the-schur-complement-is-the-analytical-marginal´) establishes that retiring a Sentinel narrows a learned live Gaussian to its surviving coordinates rather than substituting a different posterior.

## What the promise says precisely · `sec:assayer:intent-the-schur-complement-is-the-analytical-marginal-precision`

Immediately before deregistration, the operational model has mean $\mu$, covariance $\Sigma$, precision $B=\Sigma^{-1}$, and dimension $p$; the target Sentinel's slot supplies the removed indices $r$, and their ascending complement supplies the kept indices $k$.

The analytical marginal is $\mu_k$ and $\Sigma_{kk}$, while its exact precision is $S_0=B_{kk}-B_{kr}B_{rr}^{-1}B_{rk}$; the measurable identity is $S_0^{-1}=\Sigma_{kk}$ (´thm:gaussian:marginalisation´), and the correction must carry non-zero cross-information (´def:gaussian:schur-complement´).

The implementation evaluates $S_\delta=B_{kk}-B_{kr}(B_{rr}+\delta I)^{-1}B_{rk}$ with $\delta=\lambda_{\mathrm{prior}}\varepsilon_{\mathrm{Schur}}$, may fall back to $B_{kk}$, and reports the approximation path it took (´alg:gaussian:regularised-schur´).

The witness configures the strictly positive host value $\varepsilon_{\mathrm{Schur}}=\mathtt{f64::EPSILON}$, derives $\delta$ from the configured prior precision, and requires the inverse covariances of $S_0$ and $S_\delta$ to differ by no more than `DEFAULT_TOLERANCES.default`, the harness's declared $10^{-9}$ budget for one-shot floating-point comparisons (´tab:assayer:harness-scenario-tolerances´).

For every matrix comparison the error statistic is $\max_{i,j}|A_{ij}-E_{ij}|$. The post-event covariance must be within `DEFAULT_TOLERANCES.default` of both $S_\delta^{-1}$ and $\Sigma_{kk}$, the retained mean must equal $\mu_k$ bit for bit, and the post-event width must equal $|k|$.

The setup is admissible only when $\max_{i,j}|(B_{kk}^{-1})_{ij}-(\Sigma_{kk})_{ij}|$ exceeds `DEFAULT_TOLERANCES.default`, so replacing the Schur correction with the bare kept block cannot satisfy the comparison budget.

The destructive event uses the Sentinel lifecycle algorithm over every full feature-space model and becomes visible at one atomic publication boundary (´alg:registry:sentinel-deregistration´) (´inv:guarantee:lifecycle-publication´).

## What the code offers today · `sec:assayer:intent-the-schur-complement-is-the-analytical-marginal-code-today`

The unified `Scenario` owns one real `World`; `scenario_with_config` accepts the explicit Schur configuration, applies a deterministic seed, and returns only after the construction fixture has checked the built world. `World::register_sentinel`, `World::receive_report`, and `World::deregister_sentinel` drive the public lifecycle and report surfaces, and each lifecycle verb settles model-owner publication before returning (´dec:harness:single-scenario´) (´dec:harness:real-engine´) (´dec:harness:guarded-fixtures´) (´tab:assayer:harness-implementation-library-roster´).

The golden-report helpers `attach_golden_reporting_sentinel` and `refresh_golden_report` provide authored Sentinel stimuli (´dec:assayer:golden-report-stimulus´). `World::settle_cold_ramp_with` drives observations through `World::flush_observations`, while `World::flush_labels` is the model-owner command-and-label publication barrier; neither operation relies on elapsed time (´dec:harness:no-ad-hoc-waits´) (´entry:assayer:harness-closed-barriers´).

`World::published_model_block` returns an owned `PublishedModelBlock` containing one model's mean, column-major covariance, dimension, publication version, and layout generation after a publication barrier. `World::published_slot_moments`, `World::observed_runtime_layout`, and `World::sentinel_departing_block_width` provide the settled slot width, block widths, and complete departing width needed to identify the removal while preserving the exclusion of precision (´dec:harness:probe-contract´) (´entry:assayer:harness-probe-contract´) (´dec:retention:precision-excluded´).

The shared `regularised_schur_complement` oracle evaluates the specification formula by pivoted Gauss-Jordan elimination independently of the production Cholesky half-solve, and accepts either zero or the configured $\delta$ (´dec:harness:oracle-tier´) (´entry:assayer:harness-oracle-tier´). Long label stimuli can use subject-owned `PlaybackRow` values with `PlaybackBarrierPolicy`, `PlaybackProgress`, `PlaybackBatchSize`, and `playback`, leaving publication cadence to the shared runner (´dec:harness:declarative-playback´) (´entry:assayer:harness-tape-runner´).

The crate test (´test:crate:axis-lifecycle-seeded-sweep´) already trains a real world, reconstructs precision from `PublishedModelBlock`, applies `regularised_schur_complement`, and checks the resulting covariance after outcome-axis deregistration. It establishes the regularised oracle route for a different lifecycle entity, not the exact live Sentinel marginal required here. The seeded publication test (´test:crate:lifecycle-publication-seeded-sweep´) includes Sentinel deregistration but checks publication and width coherence rather than the marginal covariance.

The direct-model tests (´test:crate:round-trip-extend-observe-marginalise´) and (´test:crate:sigma-prime-from-fresh-cholesky´) establish cross-term transfer and covariance refactorisation after a multi-coordinate shrink, while the constructed-matrix test (´test:integration:woodbury-is-the-exact-rank1-correction´) separates exact, regularised, and bare-block answers. None starts from a posterior learned through public Sentinel traffic and then retires that Sentinel.

The published snapshot carries $\mu$, $\Sigma$, and the dimension map but deliberately excludes $B$ (´def:publication:model-snapshot´) (´dec:retention:precision-excluded´). The witness must therefore reconstruct $B$ from the captured live $\Sigma$ rather than widening either production publication or the probe contract.

## The witness · `sec:assayer:intent-the-schur-complement-is-the-analytical-marginal-witness`

**Decision (The scenario learns before it removes)** · `dec:assayer:intent-the-schur-complement-is-the-analytical-marginal-live-setup`

The test extends the lifecycle-algebra integration witness (´test:integration:label-cycle-under-sentinel-churn-stays-finite´). It uses `scenario_with_config` to set `f64::EPSILON` as the Schur factor, declares no interaction templates, attaches retained and retiring golden-report Sentinels, and settles the cold ramp on fixed requests through the observation barrier.

The balanced training population is represented by a subject-owned `PlaybackRow` carrying a Sentinel name, coordinate, entity, and label valence. `playback` runs those rows with a `PlaybackBarrierPolicy` containing `PlaybackBarrier::FlushLabels`, a bounded `PlaybackBatchSize`, and a `PlaybackProgress` diagnostic, so every next row sees the preceding label publication and no row authors its own wait (´dec:harness:declarative-playback´) (´tab:assayer:harness-trained-state-fixture-figures´).

After playback, the setup loads the operational `PublishedModelBlock`, both `PublishedSlotMoments` values, and the `RuntimeLayout`; it requires matching publication versions for the model and slots, finite symmetric covariance, successful Cholesky inversion, the declared empty interaction block, a regularised-to-exact covariance gap within `DEFAULT_TOLERANCES.default`, and bare-block separation beyond that budget. Those measured preconditions are reported before the stimulus, separately from the result oracle (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

The fixed seed makes the guarded learned posterior reproducible. Scenario time does not move, and no elapsed-time assertion, maintenance poll, convergence poll, deadline, or cross-world drift budget participates.

**Decision (One public retirement is the stimulus)** · `dec:assayer:intent-the-schur-complement-is-the-analytical-marginal-stimulus`

The target is the first registered Sentinel. The removal starts after the pre-event probes have recorded the target slot width and the health report's Schur-skip count. The Sentinel block begins after the bias, aggregate, identity, cross-dimension, and signal widths in `RuntimeLayout`; because the target was registered first and the fixture declares no interactions, its contiguous slot supplies $r$, and the ascending complement supplies $k$ (´def:dimension:layers´) (´def:gaussian:gathered-partition´).

The stimulus is one call to `World::deregister_sentinel`. That verb crosses the lifecycle publication barrier before returning, so the following `World::published_model_block` is the post-event owned reading and requires no additional wait (´dec:construction:two-phase-visibility´) (´dec:harness:probe-contract´).

Removing the first Sentinel rather than the tail changes the positions of the surviving slot, so the bitwise mean and covariance comparisons exercise kept-index order as well as the matrix identity.

**Decision (The observation compares live state to an independent oracle)** · `dec:assayer:intent-the-schur-complement-is-the-analytical-marginal-observation`

Before retirement, the oracle converts the captured column-major covariance to a dense symmetric matrix, reconstructs $B$ by Cholesky inversion, gathers $B_{kk}$ and $\Sigma_{kk}$, and calls `regularised_schur_complement` with zero and with the configured $\delta$ to obtain $S_0$ and $S_\delta$. It inverts those three precision candidates before loading the post-event covariance, so no expected value is derived from the result under test (´dec:harness:oracle-tier´).

The assertions prove $S_0^{-1}\approx\Sigma_{kk}$, prove $S_\delta^{-1}\approx S_0^{-1}$ inside `DEFAULT_TOLERANCES.default`, compare the post-event covariance with both expected matrices, compare the retained mean with $\mu_k$ bit for bit, and compare the reduced dimension with $|k|$.

The health report's Schur-skip count must not rise across retirement. This rejects an answer obtained through the explicit kept-block fallback independently of the matrix comparison.

**Decision (The red case is carried inside the green case)** · `dec:assayer:intent-the-schur-complement-is-the-analytical-marginal-fails-before`

A deliberately broken implementation that returns $B_{kk}$ produces $B_{kk}^{-1}$ after covariance refactorisation and fails the setup's measured separation from $\Sigma_{kk}$. Omitting or misordering a removed index changes the post-event width, a retained mean bit, or at least one covariance entry.

The passing test records the bare-block separation as part of its guarded baseline, so later fixture changes cannot make the fallback indistinguishable from the analytical marginal.

## What is missing · `sec:assayer:intent-the-schur-complement-is-the-analytical-marginal-missing`

**Entry (A version-coherent live-posterior capture)** · `entry:assayer:intent-the-schur-complement-is-the-analytical-marginal-published-view`

The `lifecycle_algebra` integration target needs a test-local capture helper that composes `World::published_model_block`, `World::published_slot_moments`, `World::observed_runtime_layout`, and `World::sentinel_departing_block_width`, checks model-slot version agreement and the no-interaction declaration, and returns owned pre-event values. The shared probes have landed; no new harness projection or production field is required (´entry:assayer:harness-probe-contract´) (´dec:retention:precision-excluded´).

**Entry (The analytical block comparison)** · `entry:assayer:intent-the-schur-complement-is-the-analytical-marginal-block-oracle`

The same target needs test-local adapters for column-major covariance conversion, checked Cholesky inversion, ascending complement construction, symmetric gathers, and maximum absolute matrix error. The Schur calculation itself must use the landed `regularised_schur_complement` oracle with its declared specification-formula provenance rather than duplicate production or shared-oracle logic (´entry:assayer:harness-oracle-tier´) (´test:crate:axis-lifecycle-seeded-sweep´).

**Entry (A guarded coupled live Sentinel fixture)** · `entry:assayer:intent-the-schur-complement-is-the-analytical-marginal-coupled-fixture`

The target needs a subject-owned playback row and setup helper that build the configured two-report scenario, drive the balanced label population, and return only after reporting the version, finite-matrix, Cholesky, regularisation-gap, and bare-block-separation preconditions. The shared `TrainedStateFixture` is fixed to its score-verified schema and default construction, so it cannot supply this configured Sentinel state; the specialised helper must follow the landed guarded-fixture and separate-validation contracts without widening the shared fixture (´entry:assayer:harness-guarded-fixtures´) (´dec:harness:specialised-side-harnesses´).

## Risks and open questions · `sec:assayer:intent-the-schur-complement-is-the-analytical-marginal-risks`

**Observation (Exact identity and shipped regularisation are distinct contracts)** · `obs:assayer:intent-the-schur-complement-is-the-analytical-marginal-regularisation-contract`

The claim states the unregularised theorem, whereas ordinary deregistration defaults to a positive offset and is explicitly approximate whenever coupling is non-zero (´inv:guarantee:structural-exactness´). The near-unregularised configuration keeps the live result inside the declared floating-point budget only when the measured $S_0^{-1}$-to-$S_\delta^{-1}$ guard passes; a default-configuration witness would instead establish agreement with $S_\delta^{-1}$ and would keep a different promise.

**Observation (The tolerance is earned by the fixture)** · `obs:assayer:intent-the-schur-complement-is-the-analytical-marginal-numerical-margin`

The generic absolute budget is meaningful here only for a finite, symmetric, Cholesky-invertible posterior whose measured exact-to-regularised gap fits it and whose bare-block error exceeds it. The guard reports each quantity before result validation, and the health counter separately proves that production did not take a fallback.

**Observation (Asynchronous work has explicit boundaries)** · `obs:assayer:intent-the-schur-complement-is-the-analytical-marginal-determinism`

Cold observations settle through `World::settle_cold_ramp_with`; each playback row crosses `PlaybackBarrier::FlushLabels`; `World::deregister_sentinel` settles lifecycle publication; and each model probe crosses the publication barrier before its single load. `ACK_DEADLINE` and `STATE_DEADLINE` remain confined to liveness machinery and supply no timing evidence (´dec:harness:no-ad-hoc-waits´) (´entry:assayer:harness-closed-barriers´).

**Observation (Composed probes require a stable version)** · `obs:assayer:intent-the-schur-complement-is-the-analytical-marginal-observation-boundary`

`PublishedModelBlock`, `PublishedSlotMoments`, and `RuntimeLayout` are separate owned readings rather than one composite snapshot. The fixture therefore performs no concurrent writes while capturing them, checks the model and slot versions that the probes expose, derives the target range only under the no-interaction declaration, and reconstructs excluded precision from the one model covariance. This preserves both the one-load probe contract and the precision-retention boundary (´dec:harness:probe-contract´) (´dec:retention:precision-excluded´).

## Acceptance · `sec:assayer:intent-the-schur-complement-is-the-analytical-marginal-acceptance`

The implementation report names integration target `lifecycle_algebra` and Rust test path `lifecycle_algebra::sentinel_deregistration_matches_analytical_marginal`, shows the focused test passing in development and release profiles, and shows the Assayer package gates and corpus linter green.

It reports the exact-identity error, exact-to-regularised gap, post-event error against both expected covariances, bare-block separation, removed and retained widths, pre-event publication version, and unchanged Schur-skip count, with every comparison tied to `DEFAULT_TOLERANCES.default` or exact equality rather than an unexplained number.

It shows that suppressing the correction is rejected by the measured bare-block separation, that the intent is no longer listed as uncovered, and that repeated fixed-seed runs preserve the same verdict without widening the tolerance.
