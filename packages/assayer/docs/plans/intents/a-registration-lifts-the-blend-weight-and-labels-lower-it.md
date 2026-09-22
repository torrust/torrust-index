# A reporting label reveals and later labels repay a registration leverage differential · `plan:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it`

This plan keeps the promise that the first eligible reporting label after registering a Sentinel into a converged deployment raises the anchor's blend weight relative to the same reporting-label block without the registration, while later eligible labels lower the weight toward a new residual (´claim:risk:a-registration-lifts-the-blend-weight-and-labels-lower-it´).

## What the promise says, precisely · `sec:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-promise`

Fix one reported request $x$ and write $s_i=x_g^\top\Sigma_{\mathrm{inh},g,i}x_g$ for the sister's raw quadratic form on the gathered anchor coordinates, $a_i=\tilde{x}^\top\Sigma_{\mathrm{anc},i}\tilde{x}$ for the anchor form, and $w_i=(1-a_i/(s_i+\varepsilon))^+$ for the anchor weight at checkpoint $i$ (´def:risk:subspace-blend´). The implementation computes the weight from raw forms because their common time correction cancels from the ratio (´dec:calibration:raw-weight-forms´); the specification's conflicting statement that elapsed time moves the weight remains an open corpus issue, so this witness holds virtual time fixed.

The pre-registration state is a guarded trained fixture followed by a measured blend precondition, not a nominal label count: after the incumbent is registered, let $p_0$ be the sister dimension and require the checkpoints at $7p_0$, $8p_0$, $9p_0$, and $10p_0$ stationary eligible labels to have $0.02\leq w_i\leq0.10$. That interval is the deployment-dependent residual the specification states; neither a centre near $0.05$ nor a separate fitted stability width is promised (´rem:risk:blend-mechanisms´) (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

Let $\tau$ be `World::tol().default` and let $\delta=10\tau$, a numerical separation guard derived before the fixture runs from the registered scenario tolerance rather than from an observed margin (´tab:assayer:harness-scenario-tolerances´). The claim selects the first eligible reporting label, not the first block of $p$ labels: under the later-label counterfactual reading, the treatment registers a new Sentinel and the control does not, both receive the same next eligible request and label, and only the treatment request names the newly reporting Sentinel. Immediately after that one published label, $s_1^{\mathrm{reg}}>s_1^{\mathrm{ctl}}+\delta$ and $w_1^{\mathrm{reg}}>w_1^{\mathrm{ctl}}+\delta$.

The component reading must also show that the sister-form differential is sufficient for the lift without assuming that the anchor form stayed fixed: $(1-a_1^{\mathrm{ctl}}/(s_1^{\mathrm{reg}}+\varepsilon))^+>w_1^{\mathrm{ctl}}+\delta$, while each actual $w_1$ agrees with its own $s_1$ and $a_1$ within $\tau$. This counterfactual respects the restriction of activation to the shared subspace even though the reporting Sentinel can change anchor inputs on the treatment row (´inv:guarantee:blend-subspace´).

The recovery phase continues the treatment's stationary eligible labels through a budget of $10p$ post-registration labels, where $p$ is the post-registration sister dimension. At the terminal checkpoint $s_N<s_1^{\mathrm{reg}}-\delta$ and $w_N<w_1^{\mathrm{reg}}-\delta$, and the checkpoint weights at $7p$, $8p$, $9p$, and $10p$ all lie in $[0.02,0.10]$; the new residual need not equal the old one because it is emergent rather than configured (´dec:risk:anchor-floor´).

The $p$-scaled horizon is an upper fixture budget drawn from the approximate interaction-maturity boundary, not a production deadline or a claim of per-label monotonicity (´tab:warmup:stages´). The registration boundary, the first-label differential, and the terminal envelope are separate observations.

## What the code offers today · `sec:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-code`

The public path is `Assayer::register_sentinel` followed by ordinary assessment and label calls. `World::register_sentinel` allocates a stable name, invokes that public registration, and crosses the model-owner publication barrier before returning, closing the production surface's two-phase visibility interval for the scenario (´dec:construction:two-phase-visibility´) (´dec:harness:no-ad-hoc-waits´).

The lifecycle owner appends the Sentinel slot and interaction coordinates to every full-dimensional model, leaves the anchor fixed, extends standardisation, and requests an early calibration refit; the focused registration witness keeps the dimension-growth half of that path (´test:unit:register-sentinel-extends-model´). `BayesianLinearModel::extend` preserves every old mean, precision, and covariance entry and adds an independent prior block, implementing Sentinel registration, exact Gaussian extension, and anchor invariance (´alg:registry:sentinel-registration´) (´thm:gaussian:extension´) (´dec:substrate:anchor-invariant´).

The blend path computes the full sister variance, shared-subspace sister variance, anchor variance, and weight in `BlendResult`. Unit and seeded witnesses check the component identities and excluded-coordinate invariance (´test:unit:variance-diagnostics-populated´) (´test:crate:blend-component-diagnostics-seeded-sweep´). The host-visible `RiskBasis` exposes `anchor_weight` and `anchor_converged` but none of those component variances, despite the schema naming full sister and anchor uncertainty; completing those two schema fields would still not expose the shared-subspace sister form $s$ that this causal witness needs (´schema:risk:basis´).

The unified harness now supplies `World::trained_state`, `TrainedStateFixture`, and `TrainedStateBaseline`; `World::register_sentinel`, `World::receive_report`, `World::request_with_sentinel`, `World::assess`, `World::label`, `World::runtime_layout`, `World::flush_observations`, `World::flush_labels`, and `World::flush_identity_maintenance`; and the owned `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView` projections. The trained fixture reports the measured precondition that admitted its world, and a model projection crosses publication, loads once, and remains independent of later publication (´tab:assayer:harness-implementation-library-roster´) (´dec:harness:probe-contract´) (´test:crate:trained-state-fixture-returns-its-measured-precondition´) (´test:crate:published-model-block-crosses-publication-barrier´).

Long streams now use a subject-owned `PlaybackRow` with `PlaybackBarrierPolicy`, `PlaybackCheckpoint`, `PlaybackProgress`, `PlaybackBatchSize`, and `playback`; a selected barrier completes before its checkpoint (´dec:harness:declarative-playback´) (´test:unit:held-barrier-completes-before-its-checkpoint´). The oracle tier contains `OracleProvenance`, `pairwise_rank`, `regularised_schur_complement`, `decay_recurrence`, and `dimension_width`, while `run_seeded_sweep` owns reproducible generated cases; none computes the blend components at one assessment boundary, and this single deterministic counterfactual needs no seed sweep (´dec:harness:oracle-tier´) (´dec:harness:seeded-sweeps´).

Existing tests touch the mechanism without keeping this claim. The outcome-axis registration witness compares one risk basis across a lifecycle boundary, the scalar convergence witness supplies the nearest public assess-label trajectory, and the Sentinel association witness supplies a long reported-Sentinel stream (´test:integration:outcome-axis-registration-does-not-perturb-risk-basis´) (´test:integration:scalar-signal-population-split-converges-directionally´) (´test:integration:association-separates-a-keyed-sentinel-from-a-hash-fed-one´). No test mints or cites this plan's claim, and the `lifecycle_identity_convergence` target assigned by the convergence split still contains no witness (´dec:harness:convergence-split´).

The standing testing plan's open anchor-behaviour gap still covers the missing behavioural composition (´entry:assayer:gap-anchor-behaviour´). Its historical stage-tapes dependency has landed as the finished probes, playback, fixtures, oracles, and barriers above; it is no longer a blocker.

## The witness · `sec:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-witness`

The witness below is implementable only if the outstanding specification ruling selects the later-label counterfactual. `lifecycle_identity_convergence` is its integration target and `a_registration_lifts_the_blend_weight_and_labels_lower_it` is its function, because lifecycle admission is the boundary whose effect must converge (´dec:harness:convergence-split´).

- Setup: obtain treatment and control from `World::trained_state` with the same seed and distinct instance names, attach the same incumbent golden reporting Sentinel to each, and retain each `TrainedStateBaseline` as setup evidence rather than as the result oracle (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

- Baseline playback: read $p_0$ from each matched `World::runtime_layout`, define a target-local `PlaybackRow` whose `play` method assesses one stationary request and submits its eligible `LabelSpec` without waiting, and run identical alternating-valence rows through both worlds with `PlaybackBarrierPolicy::new` selecting `PlaybackBarrier::FlushLabels`. Use declared `PlaybackCheckpoint` callbacks at $7p_0$, $8p_0$, $9p_0$, and $10p_0$, and reject setup unless every treatment weight is in the residual band and every matched control reading agrees within $\delta$ (´dec:harness:declarative-playback´).

- Registration boundary: retain sister and anchor `PublishedModelBlock` readings, call `World::register_sentinel` only in the treatment, call `World::flush_labels` as the control's matched barrier, and issue neither a new-Sentinel report nor a label. Require the old sister block to remain bit-identical inside the extension and the anchor block to remain wholly bit-identical; after the missing blend-component reading lands, reassess the identical incumbent-only probe and require its $s$, $a$, and $w$ to be unchanged.

- First eligible label: ingest the same golden report for the treatment's newcomer, then play exactly one matched row whose treatment request names the incumbent and newcomer while the control request names the incumbent alone. After `PlaybackBarrier::FlushLabels`, assess the same incumbent-only fixed probe in both worlds and record $(s_1^{\mathrm{reg}},a_1^{\mathrm{reg}},w_1^{\mathrm{reg}})$ and $(s_1^{\mathrm{ctl}},a_1^{\mathrm{ctl}},w_1^{\mathrm{ctl}})$.

- Differential assertion: require the treatment-control $s$ and $w$ separations and the control-anchor counterfactual from the precise promise. Recompute each actual weight from its own components and the reported blend guard within $\tau$; do not require $a_1^{\mathrm{reg}}$ to equal $a_1^{\mathrm{ctl}}$, because the reporting Sentinel changes the treatment row's anchor inputs.

- Recovery playback: read $p$ from the treatment's `World::runtime_layout`, continue the same stationary row sequence to a total of $10p$ post-registration labels under the same flush policy, and place `PlaybackCheckpoint` callbacks at $7p$, $8p$, $9p$, and $10p$. Virtual time does not move.

- Recovery assertion: require the terminal $s$ and $w$ decreases, require each of the final four checkpoint weights to lie in the specified residual band, and require finite assessments and clean health. `PlaybackProgress` is diagnostic only and supplies no latency or convergence threshold.

The fails-before is a treatment whose first eligible label leaves the shared form or weight indistinguishable from the control. Registration-time widening fails the exact-extension branch, full-space activation without a shared-form differential fails the component and counterfactual guards, a formula mismatch fails the per-checkpoint recomputation, and a permanent step fails the terminal decline or residual envelope.

## What is missing · `sec:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-missing`

**Entry (Exact extension places the lift at the first reporting label)** · `entry:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-transition-contract`

The corpus still lacks the ruling that selects among registration-time shared-block widening, the later-label counterfactual described above, and a compound registration-plus-first-report event. The harness audit records this as one of the specification conflicts no fixture may conceal (´tab:assayer:testing-harness-record-audit-backlog-entries´) (´obs:assayer:testing-harness-record-audit-six-rulings´). If the later-label branch is selected, the `lifecycle_identity_convergence` target needs a small boundary helper that compares the old sister block and the fixed anchor through owned `PublishedModelBlock` readings; either other branch requires the promise and witness to be rewritten before code is added.

**Entry (A read-only blend-component probe)** · `entry:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-blend-probe`

The `testing` module still lacks an owner-consistent assessment reading containing the raw shared-subspace sister form $s$, anchor form $a$, blend guard, and resulting weight. Its `World` entry point must cross the label-publication barrier, take the assessment's one published view, return an owned value, and expose no mutation, following the landed probe contract (´dec:harness:probe-contract´). `PublishedModelBlock` lacks the assembled fixed-probe vector and anchor index map needed to reconstruct $s$, while the specified full sister uncertainty on `RiskBasis` is not the shared form.

**Entry (A stationary Sentinel trajectory tape)** · `entry:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-trajectory-tape`

The shared runner is complete; only a subject-owned row and checkpoint collectors are missing in `lifecycle_identity_convergence`. The row must build the treatment request with the newcomer only when that name exists, submit the matched label without an internal wait, and leave `PlaybackBarrierPolicy` to cross `PlaybackBarrier::FlushLabels`; the collectors retain fixed-probe component readings at the predetermined baseline, first-label, and recovery boundaries (´tab:assayer:harness-implementation-library-roster´) (´dec:harness:declarative-playback´).

**Entry (The public registration-transient witness)** · `entry:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-integration-witness`

The `lifecycle_identity_convergence` target still needs `a_registration_lifts_the_blend_weight_and_labels_lower_it`, with indexed documentation carrying the intent claim, the guarded matched baseline, the selected transition assertion, first-label component and formula assertions, the recovery envelope, non-vacuity checks, and final health check. It depends on the ruling and blend-component reading; every other harness dependency exists.

## Risks and open questions · `sec:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-risks`

**Observation (Exact extension places the lift at the first eligible reporting label)** · `obs:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-extension-conflict`

Exact Gaussian extension preserves the old posterior as a marginal and gives the new prior block zero cross-terms, while the anchor-floor decision says a new Sentinel raises leverage and weight (´thm:gaussian:extension´) (´dec:risk:anchor-floor´). The plan's later-label reading makes registration itself unchanged and attributes the lift to the first eligible update, but the corpus has not selected that reading over shared-block widening or a compound registration-plus-report event; implementation stops at that choice.

**Observation (The registration boundary remains report-free and unchanged)** · `obs:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-immediate-boundary`

Under the later-label branch, registration carries identity and name while measurement arrives through a later report. The boundary checkpoint must therefore precede every newcomer report and label, use the settled `World::register_sentinel` publication boundary, and prove exact extension from owned model projections before the first reporting row is allowed to move the models (´dec:construction:two-phase-visibility´) (´dec:harness:probe-contract´).

**Observation (The residual band binds, while numerical guards are declared)** · `obs:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-thresholds`

The two-to-ten-percent residual comes from the blend analysis; $\tau$ comes from the harness tolerance register, and $\delta=10\tau$ is declared before execution. No observed margin may be fitted back into acceptance. `TrainedStateFixture` establishes class counts, held-out separation, and rank, not a blend residual or Sentinel-specific convergence, so the four in-band baseline readings remain a separate measured setup guard rather than an assumption imported from the fixture (´rem:risk:blend-mechanisms´) (´tab:assayer:harness-scenario-tolerances´) (´test:crate:trained-state-fixture-returns-its-measured-precondition´).

**Observation (Recovery is an envelope, not per-label monotonicity)** · `obs:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-recovery-envelope`

Forgetting, leverage caps, standardisation bootstrap, and alternating outcomes can move adjacent checkpoint weights in either direction while evidence reduces the lifted shared uncertainty overall. The peak-to-terminal decrease and four terminal in-band readings keep the promised transient without inventing a monotonicity rule absent from the model update and Sentinel bootstrap (´prop:update:leverage-bound´) (´alg:standardisation:sentinel-bootstrap´).

**Observation (Publication barriers replace timing assumptions)** · `obs:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-publication-order`

Registration publication and label publication are asynchronous engine operations with deterministic harness boundaries. Every model reading follows `World::register_sentinel`, `World::flush_labels`, or a probe that crosses the same publication barrier; virtual time remains fixed, and no sleep, poll, authored deadline, or progress-derived latency enters the result oracle (´dec:harness:no-ad-hoc-waits´) (´step:assayer:harness-polling-retirement´).

The unresolved work is the transition ruling, one permitted blend-component reading, the target-local row and checkpoints, and the public witness. The general scenario, barrier, playback, fixture, probe, oracle, and sweep infrastructure is complete.

## Acceptance · `sec:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-acceptance`

Before implementation, the governing corpus selects one registration-lift branch. Acceptance below applies to the later-label counterfactual; another ruling updates the promise and this witness before test code lands (´tab:assayer:testing-harness-record-audit-backlog-entries´).

The integration target is `lifecycle_identity_convergence` and the function is `a_registration_lifts_the_blend_weight_and_labels_lower_it`. Its indexed documentation cites this intent claim, and its setup reports the guarded trained-state baseline and the separate four-checkpoint residual precondition.

The test drives registration, report ingestion, assessment, and labels through `World`; long rows run through `playback` with `PlaybackBarrier::FlushLabels`; the registration boundary uses owned `PublishedModelBlock` readings; and any test-only blend-component surface obeys the probe contract and remains distinct from host-visible `RiskBasis`.

The assertions establish an unchanged report-free registration boundary, a first-label treatment-control lift in the shared form and weight, sufficiency of the shared-form differential without assuming equal anchor forms, agreement with the blend equation, a material peak-to-terminal decline, and four terminal weights inside the specified residual band.

The fails-before evidence names the assertion rejecting no first-label differential, registration-time widening, full-space-only activation, formula drift, and a permanent step. No executed mutation substitutes for those assertions, no observed margin becomes a tolerance, and the package's formatting, lint, test, and corpus gates pass without an allowance.
