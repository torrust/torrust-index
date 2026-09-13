# A Live Witness for the Schur Marginal · `plan:assayer:intent-the-schur-complement-is-the-analytical-marginal`

Keeping (´claim:bayes:the-schur-complement-is-the-analytical-marginal´) establishes that retiring a Sentinel narrows a learned live Gaussian to its surviving coordinates rather than substituting a different posterior.

## What the promise says precisely · `sec:assayer:intent-the-schur-complement-is-the-analytical-marginal-precision`

Immediately before deregistration, the operational model has mean $\mu$, covariance $\Sigma$, precision $B=\Sigma^{-1}$, and dimension $p$; the target Sentinel's slot supplies the removed indices $r$, and their ascending complement supplies the kept indices $k$.

The analytical marginal is $\mu_k$ and $\Sigma_{kk}$, while its exact precision is $S_0=B_{kk}-B_{kr}B_{rr}^{-1}B_{rk}$; the measurable identity is $S_0^{-1}=\Sigma_{kk}$ (´thm:gaussian:marginalisation´), and the correction must carry non-zero cross-information (´def:gaussian:schur-complement´).

The implementation actually evaluates $S_\delta=B_{kk}-B_{kr}(B_{rr}+\delta I)^{-1}B_{rk}$ with $\delta=\lambda_{\mathrm{prior}}\varepsilon_{\mathrm{Schur}}$, may fall back to $B_{kk}$, and reports either approximation (´alg:gaussian:regularised-schur´).

The witness configures the strictly positive host value $\varepsilon_{\mathrm{Schur}}=\mathtt{f64::EPSILON}$, computes both $S_0$ and $S_\delta$ from the captured live posterior, and requires their inverse covariances to differ by no more than `Tolerances::default`, the harness's one-shot absolute budget of $10^{-9}$ (´tab:assayer:harness-scenario-tolerances´).

For every compared matrix, the error statistic is $\max_{i,j}|A_{ij}-E_{ij}|$; the post-event covariance must be within `Tolerances::default` of both $S_\delta^{-1}$ and $\Sigma_{kk}$, while the retained mean must equal $\mu_k$ bit for bit and the post-event width must equal $|k|$.

The fixture is admissible only when $\max_{i,j}|(B_{kk}^{-1})_{ij}-(\Sigma_{kk})_{ij}|>100\,\texttt{Tolerances::default}$, so replacing the Schur correction with the bare kept block cannot pass inside rounding slack.

The removal is destructive, uses the Sentinel lifecycle algorithm over every full feature-space model, and completes at one atomic publication boundary (´alg:registry:sentinel-deregistration´), (´inv:guarantee:lifecycle-publication´).

## What the code offers today · `sec:assayer:intent-the-schur-complement-is-the-analytical-marginal-code-today`

The world harness provides the real engine, a fixed `VirtualClock`, seeded randomness, `register_sentinel`, `receive_report`, assess-label cycles, queue barriers, `deregister_sentinel`, and named tolerances; each lifecycle verb settles the asynchronous model owner after the public call.

The scenario builder provides `scenario_with_config`, so the fixture can set the Schur factor while retaining the tracing span, deterministic seed, and ordinary `WorldBuilder` construction.

The golden-report helpers provide `attach_golden_reporting_sentinel` and `refresh_golden_report`; two such Sentinels make both slot blocks live through the public report surface rather than through planted matrix entries (´dec:assayer:golden-report-stimulus´).

The world harness also provides `settle_cold_ramp_with`, `cycle_on`, and `flush_labels`, which make standardisation and label application explicit before either posterior capture.

The public lifecycle test (´test:integration:label-cycle-under-sentinel-churn-stays-finite´) already proves that assess-label traffic survives Sentinel retirement, but observes only finite outputs and therefore cannot distinguish a Schur correction from $B_{kk}$.

The direct-model test (´test:crate:round-trip-extend-observe-marginalise´) builds cross-terms and checks dual tracking after a shrink, while (´test:crate:sigma-prime-from-fresh-cholesky´) establishes that a multi-coordinate removal refactors covariance from the retained precision; both start from a model object rather than a posterior learned through the public engine.

The constructed-matrix witnesses already contain Cholesky inversion and block-gather idioms and show why regularised and exact marginals differ (´test:integration:woodbury-is-the-exact-rank1-correction´), but their matrices are fixtures and their single-coordinate path is not Sentinel deregistration.

The published model snapshot carries $\mu$, $\Sigma$, and the dimension map but deliberately excludes $B$ (´def:publication:model-snapshot´), (´dec:retention:precision-excluded´); the oracle therefore reconstructs $B$ from the captured live $\Sigma$ instead of widening production publication.

The standing testing plan has no gap entry covering this promise. Its open snapshot-diffing and model-inspection stage (´entry:assayer:harness-stage-tapes´) is the nearest reusable infrastructure, but this witness needs only the narrow read-only view named below and does not wait on streaming tapes.

## The witness · `sec:assayer:intent-the-schur-complement-is-the-analytical-marginal-witness`

**Decision (The scenario learns before it removes)** · `dec:assayer:intent-the-schur-complement-is-the-analytical-marginal-live-setup`

The test extends the lifecycle-algebra integration witness (´test:integration:label-cycle-under-sentinel-churn-stays-finite´). It builds a configured `World`, attaches retained and retiring golden-report Sentinels, settles the cold ramp on a fixed request, and submits a deterministic mixture of benign and adverse labels while both reports are present.

The precondition reads the operational posterior after the label barrier, verifies finite symmetric covariance, reconstructs $B$ by Cholesky inversion, and rejects a fixture whose removed-to-kept coupling is too weak to place $B_{kk}^{-1}$ beyond the stated rejection margin.

The fixed seed and fixed virtual time make the learned posterior reproducible; no time advance, elapsed-time assertion, cross-world drift budget, maintenance polling, or convergence heuristic participates.

**Decision (One public retirement is the stimulus)** · `dec:assayer:intent-the-schur-complement-is-the-analytical-marginal-stimulus`

The stimulus is one call to `World::deregister_sentinel` for the first Sentinel, after the pre-event view records its contiguous slot range and confirms that the fixture declared no interaction templates; the call's existing settlement barrier makes the next view post-publication (´dec:construction:two-phase-visibility´).

The first Sentinel is removed rather than the tail Sentinel, so compaction changes the positions of a surviving live slot and the comparison exercises the kept-index order as well as the matrix identity.

**Decision (The observation compares live state to an independent oracle)** · `dec:assayer:intent-the-schur-complement-is-the-analytical-marginal-observation`

The oracle derives $k$, $B_{kk}$, $B_{kr}$, $B_{rr}$, $S_0$, $S_\delta$, $S_0^{-1}$, $S_\delta^{-1}$, and $B_{kk}^{-1}$ from the pre-event view before loading the post-event covariance.

The assertions first prove the oracle identity $S_0^{-1}\approx\Sigma_{kk}$, then prove the configured regularisation gap is inside `Tolerances::default`, then compare the post-event mean and covariance to the analytical marginal and the regularised implementation oracle.

The health report's Schur-skip count must not rise across the event, so agreement obtained through the explicit kept-block fallback is rejected even before the matrix separation assertion.

**Decision (The red case is carried inside the green case)** · `dec:assayer:intent-the-schur-complement-is-the-analytical-marginal-fails-before`

A deliberately broken implementation that returns $B_{kk}$ produces $B_{kk}^{-1}$ after the existing covariance refactor and misses $\Sigma_{kk}$ by more than one hundred comparison budgets; omitting or misordering a removed index changes the post-event width or at least one compared entry.

The passing test records that separation as an assertion, so its sensitivity remains visible without executing a mutation and cannot disappear when later fixture changes weaken the coupling.

## What is missing · `sec:assayer:intent-the-schur-complement-is-the-analytical-marginal-missing`

**Entry (A narrow published-posterior view)** · `entry:assayer:intent-the-schur-complement-is-the-analytical-marginal-published-view`

Add about 45 lines to the test-support harness for a `#[doc(hidden)]` owned view containing the operational mean, covariance, width, and a named Sentinel's slot range, plus a `World` accessor that takes one atomic published-snapshot load. It depends on the existing snapshot access described by (´entry:assayer:harness-stage-tapes´) but preserves the precision exclusion (´dec:retention:precision-excluded´).

**Entry (The analytical block oracle)** · `entry:assayer:intent-the-schur-complement-is-the-analytical-marginal-block-oracle`

Add about 55 lines of test-support matrix helpers for checked Cholesky inversion, ascending complement construction, symmetric submatrix and cross-block gathers, and maximum absolute matrix error. It depends on (´entry:assayer:intent-the-schur-complement-is-the-analytical-marginal-published-view´) and reuses the backend idioms exercised by the exact rank-one correction witness (´test:integration:woodbury-is-the-exact-rank1-correction´) without calling the production Schur helper whose result is under test.

**Entry (A coupled live Sentinel fixture)** · `entry:assayer:intent-the-schur-complement-is-the-analytical-marginal-coupled-fixture`

Add about 35 lines alongside the lifecycle-algebra integration witness (´test:integration:label-cycle-under-sentinel-churn-stays-finite´) for the two-report setup, cold-ramp settlement, deterministic label stream, and non-vacuity checks. It depends on the published view and block oracle entries, while the existing `LabelSpec` and golden-report helpers supply the traffic.

## Risks and open questions · `sec:assayer:intent-the-schur-complement-is-the-analytical-marginal-risks`

**Observation (Exact identity and shipped regularisation are distinct contracts)** · `obs:assayer:intent-the-schur-complement-is-the-analytical-marginal-regularisation-contract`

The unchanged intent states the unregularised theorem, whereas ordinary deregistration defaults to a positive offset and is explicitly approximate whenever coupling is non-zero (´inv:guarantee:structural-exactness´). The planned near-unregularised configuration keeps the stated identity through the real lifecycle and measures that its residual bias is below the assertion budget; a default-configuration witness instead compares against $S_\delta^{-1}$ and keeps a different promise about declared approximation.

**Observation (The tolerance is earned by the fixture)** · `obs:assayer:intent-the-schur-complement-is-the-analytical-marginal-numerical-margin`

An absolute matrix budget can be too tight for a poorly scaled posterior and too loose for a nearly uncoupled one. The scenario therefore checks Cholesky success, finite entries, the exact-versus-regularised gap, and the bare-block rejection margin before treating `Tolerances::default` as evidence.

**Observation (Asynchronous work has explicit boundaries)** · `obs:assayer:intent-the-schur-complement-is-the-analytical-marginal-determinism`

The virtual clock stays fixed, cold observations settle before training, labels flush before the pre-event view, and deregistration settles before the post-event view. `ACK_DEADLINE` and `STATE_DEADLINE` remain liveness ceilings rather than timing assertions, and the witness introduces no scheduler-sized numerical allowance.

**Observation (Published covariance is enough, and precision remains private)** · `obs:assayer:intent-the-schur-complement-is-the-analytical-marginal-observation-boundary`

Reconstructing $B$ from the one published $\Sigma$ keeps the observation on a state a reader can actually obtain and preserves the record's exclusion of precision. The existing fresh-Cholesky witness ties the post-event published covariance to the precision produced by general marginalisation (´dec:posterior:always-refactor´), so a wrong correction remains observable at this boundary.

## Acceptance · `sec:assayer:intent-the-schur-complement-is-the-analytical-marginal-acceptance`

The implementation report names the new integration test and helper changes, shows the focused test passing in development and release profiles, and shows the Assayer package gates and corpus linter green.

It reports the observed exact-identity error, exact-versus-regularised gap, post-event error, bare-block separation, removed and retained widths, and unchanged Schur-skip count, with each compared against the named threshold rather than presented as an unexplained number.

It shows that suppressing the correction is rejected by the in-test bare-block separation, that the intent is no longer listed as uncovered, and that repeated runs preserve the same verdict without widening the tolerance.
