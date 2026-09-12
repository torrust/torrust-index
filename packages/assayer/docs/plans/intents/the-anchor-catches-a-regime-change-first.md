# Keeping the Anchor Ahead of a Regime Change · `plan:assayer:intent-the-anchor-catches-a-regime-change-first`

Keeping (´claim:risk:the-anchor-catches-a-regime-change-first´) establishes that the fixed low-dimensional model supplies a newly correct direction while the full sister still predicts from the regime that just ended.

## What the promise says, precisely · `sec:assayer:intent-the-anchor-catches-a-regime-change-first-promise`

The promise concerns eligible-label convergence of the anchor and sister, not elapsed execution time. Both models receive the same unconfounded outcomes under (´tab:eligibility:training´), and their different parameter counts determine the comparison.

The anchor is the fixed fifteen-coordinate projection of (´def:risk:anchor-model´); the sister is the deployment-width member of the triple in (´def:risk:model-triple´). The standard reference configuration has eight Sentinels, one spatial outcome axis, two identity dimensions, twenty competitive cells, no signals, and a 68-position interaction block (´tab:resource:reference-configuration´); the dimension formula (´tab:feature:dimension-formula´) produces a sister width of 638 from those blocks.

The convergence rule is approximately two eligible observations per parameter (´bound:resource:convergence-budget´). It therefore gives the anchor a thirty-label budget and the standard sister a 1,276-label budget, whose ratio is approximately 42.5 and supplies the measurable meaning of “roughly forty times faster.”

The behavioural quantity is the assessment-time prediction residual $e_{m,t}=r_t-\hat{\rho}_{m,t}$ for model $m$, with $r_t=+1$ after the changed class becomes adverse under (´def:risk:target´). A model still carrying the benign regime has $\hat{\rho}_{m,t}<0$ and therefore $e_{m,t}>1$; a model that has recovered the correct direction has $\hat{\rho}_{m,t}>0$ and therefore $e_{m,t}<1$.

The primary threshold is directional and introduces no fitted epsilon: by thirty eligible post-change labels, the median anchor prediction over the fixed changed-class probe set is positive while the median sister prediction remains negative. By 1,276 eligible post-change labels, the sister median is positive too, proving delayed recovery rather than permanent incapacity.

The structural threshold accompanies the behavioural one: the live snapshot reports widths 15 and 638, and the sister-to-anchor width ratio lies between 40 and 45. This assertion keeps the numerical premise from passing against a narrow sister fixture that makes firstness trivial.

The stimulus belongs to the anchor’s actual subspace. The report population varies the measurement-only maximum raw z-score whose provenance is fixed by (´prop:risk:anchor-measurement-only´), while independently varying the Sentinel-specific details that make the sister rich.

The fixed dimension is not incidental test geometry. The substrate record makes the anchor invariant across lifecycle changes (´dec:substrate:anchor-invariant´), so the witness constructs the full shape before either regime and performs no registration during the measured transition.

This promise is covered by the standing gap (´entry:assayer:gap-anchor-behaviour´).

## What the code offers today · `sec:assayer:intent-the-anchor-catches-a-regime-change-first-current-code`

The world harness already supplies `WorldBuilder`, named Sentinel and spatial-axis registration, report ingestion, `settle_cold_ramp_with`, public `assess` and `label` calls, and the label and observation barriers required to put each reading after a known publication.

The label-specification helper supplies eligible `Allow` labels with explicit benign and adverse constructors. The world harness’s `cycle_request` already drives the public assess-then-label path used by the nearest convergence witness (´test:integration:scalar-signal-population-split-converges-directionally´).

The report-fixture module supplies mutable `BatchReport` fixtures, cell constructors, routing coordinates, and ingestion helpers. The golden report’s values are authored stimuli rather than generated observations (´dec:assayer:golden-report-stimulus´), which is the right precedent for a two-regime report tape whose contrasting cells are chosen to expose a known direction.

The anchor projection and its computed measurement coordinates already have focused witnesses, including (´test:crate:anchor-subvector-computes-its-last-two-inputs´). Its fixed lifecycle width and small-model update arithmetic are separately kept by (´test:crate:anchor-dimension-unchanged´) and (´test:crate:anchor-p15-dual-tracking´).

The exact reference layout is already kept at the feature-map boundary by (´test:crate:rebuild-reference-config-p638´). `AssayerBuilder::interaction_templates` accepts its fourteen templates, but `WorldBuilder` does not pass templates through, and `World::register_identity` does not install a chosen competitive-cell set; the public scenario path therefore cannot yet reproduce that layout deterministically.

`Assayer::full_health_report()` already publishes a `DriftHealth` entry for each `ModelId`, including the mean absolute residual and residual-sign EWMA specified by (´tab:monitoring:drift-diagnostics´). Those readings are the natural observation surface for the transition.

`RiskBasis` publishes the blended raw score, the blend weight, the calibration scale, and the sister probability, but it does not publish the raw anchor score. A fixed probe can therefore drive the public assessment path today, but a test-support observation hook is needed to read both unblended predictors without numerically inverting the blend.

The present producer does not yet make those entries model-specific. The pending risk basis retains only the blended predictor, and step sixteen of the label path feeds its residual to every model’s drift state; the existing audit records that divergence in (´tab:assayer:owner-remaining´) against the per-model rule (´alg:monitoring:drift-cusums´).

The harness has no standard-width converged factory, no label or report tape, and no residual trace that turns the smoothed health readings back into each label’s assessment-time residual. Those absences are the relevant part of the open stage-three entry (´entry:assayer:harness-stage-tapes´).

## The witness · `sec:assayer:intent-the-anchor-catches-a-regime-change-first-witness`

The integration test belongs beside the public-API directional smoke (´test:integration:scalar-signal-population-split-converges-directionally´), because it extends that learning-convergence module from one stationary population to a long deterministic transition without mixing in lifecycle or persistence.

The setup builds one seeded `World` with eight registered Sentinels, one spatial outcome axis, two identity dimensions with ten fixed competitive cells each, no request signals, and the reference configuration’s fourteen explicit interaction templates: five type-one templates, eight type-four templates, and one type-five template. It ingests one two-route report per Sentinel, settles the cold standardisation ramp across both routes, and asserts the published model widths are exactly 15 and 638 before training.

One route is the changed class. Its cells carry a coherent coarse measurement contrast visible in the anchor projection, while their remaining slot readings vary by Sentinel and tape position. The other route is an unchanged adverse control that prevents a one-class old regime and keeps a stable direction beside the class that flips.

The old-regime tape alternates the changed class as benign and the control as adverse for 1,276 eligible labels. A fixed probe batch then establishes a negative median anchor prediction and a negative median sister prediction for the changed class, while the control remains positive.

The stimulus flips only the changed class from benign to adverse. Reports, coordinates, feature distributions, clock, action, eligibility, and control labels remain fixed, so the changed conditional outcome rather than a changed input distribution is the cause of recovery.

Each post-change cycle assesses, captures the three model-specific pre-update predictors, submits the adverse or control label, flushes the label path, and appends the reconstructed model-specific residuals and full health readings to the trace. Fixed changed-class probe batches at the anchor and sister budgets use the same component-score hook but receive no labels and therefore do not train either model.

At thirty post-change eligible labels, the changed-class probe median is positive for the anchor and negative for the sister, equivalently placing the median signed anchor residual below one and the sister residual above one. The final ten anchor residuals also have a lower median absolute value than the sister’s over the same labels.

At 1,276 post-change eligible labels, the sister probe median is positive and its final ten residuals have a lower median absolute value than its first ten post-change residuals. The unchanged control stays positive at every checkpoint, excluding a global sign inversion as the source of the result.

The width assertion and the two budget checkpoints make the fortyfold statement a ratio of specified evidence budgets rather than a stopwatch comparison. The trace prints the widths, checkpoint medians, residual medians, and first stable sign crossing for each model so a failure names the quantity that moved.

The fails-before is direct. With the current blended-residual fan-out, the anchor and sister residual traces are identical and their residual-separation assertion fails; with an anchor widened to the sister shape, deprived of the measurement-only coordinate, or held on its old weights, its probe median remains negative at the anchor checkpoint; with a sister that adapts on the anchor vector, the sister turns positive too early.

## What is missing · `sec:assayer:intent-the-anchor-catches-a-regime-change-first-missing`

**Entry (Model-specific assessment-time residual retention)** · `entry:assayer:intent-the-anchor-catches-a-regime-change-first-model-residual-retention`

The pending risk basis retains the operational, sister, and anchor predictors that the blend already computes, and the label path updates each risk model’s diagnostic from its own stored predictor while retaining the eligibility gate. The journal form carries the same values so replay preserves the assessment-time rule. Estimate: approximately 70–110 lines across the pending form, persistence projection, label path, and focused tests; no sibling intent is a semantic dependency.

**Entry (A standard-width regime tape)** · `entry:assayer:intent-the-anchor-catches-a-regime-change-first-standard-regime-tape`

A focused fixture passes interaction templates through `WorldBuilder`, installs two deterministic identity dimensions with ten competitive cells each through a test-support lifecycle verb, builds the remaining eight-Sentinel and one-spatial-axis blocks, authors paired report routes with invariant feature distributions, and emits old- and new-regime label schedules from one seed. It asserts the 638-wide layout already kept by (´test:crate:rebuild-reference-config-p638´). Estimate: approximately 130–190 lines across the world harness, report fixtures, and a new narrow tape module; it depends on (´entry:assayer:harness-stage-tapes´) in concept but not on the whole stage-three roadmap landing first.

**Entry (Model-width, component-score, and residual tracing)** · `entry:assayer:intent-the-anchor-catches-a-regime-change-first-width-and-residual-trace`

The `World` harness exposes a read-only model-width tuple, a test-support component-score observation that accompanies a public assessment without widening the production response, and a residual-step recorder that samples `full_health_report()` on both sides of a label and reconstructs the latest signed residual from the configured absolute-value and sign EWMA recurrences. Estimate: approximately 60–100 lines plus focused helper tests; residual tracing depends on the model-specific retention entry.

**Entry (The regime-change integration witness)** · `entry:assayer:intent-the-anchor-catches-a-regime-change-first-integration-witness`

The labeled test, its module index row, fixed thresholds, trace formatting, and fails-before evidence land in the learning-convergence integration module beside (´test:integration:scalar-signal-population-split-converges-directionally´). Estimate: approximately 90–130 lines; it depends on all three entries above.

## Risks and open questions · `sec:assayer:intent-the-anchor-catches-a-regime-change-first-risks`

**Observation (Fortyfold is an evidence ratio)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-evidence-ratio-not-runtime`

Wall-clock duration measures quadratic matrix cost, scheduling, and hardware, not statistical convergence. The witness keeps the stated factor through live dimensions and eligible-label budgets, then keeps the behavioural consequence at those budgets.

**Observation (The coarse contrast must survive standardisation)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-standardisation-control`

The cold ramp sees both routes before labels begin, and both regimes replay the same report distribution. This prevents a newly moving coordinate system from impersonating anchor adaptation and follows the existing harness discipline for cross-state comparisons.

**Observation (Residual means prediction error)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-residual-identity`

The relevant residual is $r-\hat{\rho}$ from (´alg:monitoring:drift-cusums´), not the precision–covariance synchronisation residual. The former changes when the conditional outcome changes; the latter diagnoses numerical representation and has no claim to move during this stimulus.

**Observation (Calibration stays outside the oracle)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-calibration-independent-oracle`

The assertions use raw model scores and their training-target residuals rather than calibrated probabilities. A Platt refit may move probability scale during the transition without changing which model learned the new direction first.

**Observation (The exact first crossing is diagnostic)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-crossing-is-diagnostic`

The first stable sign-crossing counts are reported but not compared to an invented exact ratio. The binding thresholds are the thirty- and 1,276-label checkpoints supplied by the convergence budget; requiring an exact crossing count would turn an order-of-magnitude specification into unsupported precision.

**Observation (Re-convergence is a reference-scenario claim)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-reconvergence-scope`

The two-observations-per-parameter budget describes statistical convergence, not a universal crossing theorem for a posterior carrying old evidence. Sister and anchor share a forgetting rate under (´tab:risk:forgetting-rates´), so old-regime duration and feature excitation also affect their reversal; the witness keeps the promised ordering for one authored reference scenario and does not generalise it to every data distribution.

**Observation (The sign margins remain visible)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-sign-margin`

Zero is the semantic directional boundary, but a checkpoint median arbitrarily close to it is numerically fragile. The trace and implementation report include each median’s distance from zero, and the fixed fixture is accepted only when repeated debug and release verification preserve the required signs.

**Observation (Barriers remove scheduler timing from the result)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-barrier-determinism`

Every checkpoint follows the existing label and observation publication barriers, and the virtual clock remains fixed during the tape. Harness deadlines detect a stalled owner but contribute no performance assertion, so scheduler latency cannot decide which model is first.

**Observation (Competitive cells have two fixture boundaries)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-competitive-cell-boundary`

A test-support lifecycle event can install the twenty fixed competitive cells directly, while a scripted maintenance sequence can produce them through identity splits. Direct installation isolates risk-model re-convergence and is the focused boundary; scripted splitting exercises more production machinery but introduces a second adaptive process and additional nondeterminism.

## Acceptance · `sec:assayer:intent-the-anchor-catches-a-regime-change-first-acceptance`

The implementation report names the integration test and the promise it cites, shows the actual 15 and 638 model widths, the approximately 42.5 width and evidence-budget ratio, and the old-regime, thirty-label, and 1,276-label probe medians for both models.

It shows the final-ten residual medians at the anchor checkpoint, the sister’s first-ten and final-ten residual medians across its recovery, and the unchanged control at every checkpoint.

It shows that every driven label was eligible, both publication barriers completed, the health report stayed free of numerical degradation, and the fixed seed reproduces the same crossings and checkpoint readings in debug and release verification.

It includes the deliberately broken outcomes: blended residual fan-out makes the model traces identical, disabling or widening the anchor misses the early checkpoint, and feeding the sister the anchor projection makes it recover before the promised separation window.
