# Keeping the Anchor Ahead of a Regime Change · `plan:assayer:intent-the-anchor-catches-a-regime-change-first`

Keeping (´claim:risk:the-anchor-catches-a-regime-change-first´) establishes that the fixed low-dimensional model supplies a newly correct direction while the full sister still predicts from the regime that just ended.

## What the promise says, precisely · `sec:assayer:intent-the-anchor-catches-a-regime-change-first-promise`

The promise concerns eligible-label convergence of the anchor and sister, not elapsed execution time. Both models receive the same unconfounded outcomes under (´tab:eligibility:training´), and their different parameter counts determine the comparison.

The anchor is the fixed fifteen-coordinate projection of (´def:risk:anchor-model´); the sister is the deployment-width member of the triple in (´def:risk:model-triple´). The standard reference configuration has eight Sentinels, one spatial outcome axis, two identity dimensions, twenty competitive cells, no signals, and a 68-position interaction block (´tab:resource:reference-configuration´); the dimension formula (´tab:feature:dimension-formula´) produces a sister width of 638 from those blocks.

The convergence rule is a rule of thumb: a Bayesian linear model needs on the order of two observations per parameter before data shape its posterior (´bound:resource:convergence-budget´). Applied to the reference widths, it gives checkpoints at about thirty eligible labels for the anchor and 1,276 for the sister; the ratio is approximately 42.5. Those counts express the specified evidence-budget difference, not exact crossing deadlines for every previously trained posterior.

The behavioural quantity is the assessment-time prediction residual $e_{m,t}=r_t-\hat{\rho}_{m,t}$ for model $m$, with $r_t=+1$ after the changed class becomes adverse under (´def:risk:target´). A model still carrying the benign regime has $\hat{\rho}_{m,t}<0$ and therefore $e_{m,t}>1$; a model that has recovered the correct direction has $\hat{\rho}_{m,t}>0$ and therefore $e_{m,t}<1$.

For the authored reference scenario, the witness takes the specification-derived budgets as checkpoints: immediately before the flip both model medians for the changed class are negative; after thirty eligible post-change labels the anchor median is positive while the sister median remains negative; after 1,276 eligible post-change labels the sister median is positive. The first pair establishes firstness and the last establishes delayed recovery rather than permanent incapacity; none turns the convergence rule of thumb into a universal deadline.

The width assertion is exact rather than tolerant: the live anchor and sister widths are 15 and 638, and their two-observations-per-parameter budgets are therefore 30 and 1,276. The witness reports their approximately 42.5 ratio instead of inventing a fitted band around “roughly forty.”

The stimulus belongs to the anchor's actual subspace. The changed population varies the measurement-only maximum raw z-score whose provenance is fixed by (´prop:risk:anchor-measurement-only´), while independently varying Sentinel-specific detail that gives the sister its wider learning problem.

The fixed dimension is not incidental test geometry. The substrate record makes the anchor invariant across lifecycle changes (´dec:substrate:anchor-invariant´), so the guarded setup constructs the full shape before either regime and admits no registration or restructuring during the measured transition.

This promise remains unkept under the standing gap (´entry:assayer:gap-anchor-behaviour´). The stationary scalar-signal witness proves that the public assess-then-label loop can learn a directional population split, but it neither changes regime nor compares anchor and sister recovery (´test:integration:scalar-signal-population-split-converges-directionally´).

## What the code offers today · `sec:assayer:intent-the-anchor-catches-a-regime-change-first-current-code`

The finished harness roster supplies one real-engine `World`, `WorldBuilder`, named registration and report ingestion, `World::settle_cold_ramp_with`, `World::assess`, `World::label`, queue-specific barriers, and forward-only `World::advance` and `World::travel_to` (´tab:assayer:harness-implementation-library-roster´). This label-count witness keeps scenario time fixed, but it no longer needs a raw clock or an authored wait.

The construction gap named by the earlier plan has closed. `WorldBuilder::interaction_templates` retains the fourteen reference templates, `World::register_axis` and `World::register_sentinel` build the fixed populations, `World::register_identity_with_cells` derives and verifies declared competitive cells through real observations, and `World::runtime_layout` checks the published result against the builder's `RuntimeLayout`; the completed path already reproduces width 638 (´test:crate:public-scenario-reproduces-reference-layout-p638´).

`LabelSpec::benign` and `LabelSpec::adverse` supply eligible `Allow` labels. The report helpers supply authored `BatchReport` values, cell constructors, routing coordinates, and `World::receive_report`; their readings are stimuli rather than generated observations (´dec:assayer:golden-report-stimulus´) (´tab:assayer:harness-implementation-library-roster´). `cycle_request` remains suitable for the short stationary smoke, while this long two-regime stimulus belongs on declarative playback.

`PlaybackRow`, `PlaybackBarrierPolicy`, `PlaybackCheckpoint`, `PlaybackProgress`, and `playback` now own typed rows, queue completion, exact checkpoints, progress diagnostics, and bounded materialisation (´dec:harness:declarative-playback´). The runner already proves that a selected label barrier completes before its checkpoint reads publication (´test:crate:selected-label-barrier-completes-before-checkpoint´).

`World::published_model_block` returns an owned `PublishedModelBlock` after the publication barrier, including the model dimension and mean from one snapshot (´dec:harness:probe-contract´) (´test:crate:published-model-block-is-from-one-version´). The oracle tier supplies `dimension_width` as an independent evaluation of the specification formula (´dec:harness:oracle-tier´) (´test:crate:dimension-width-oracle-sums-the-reference-blocks´).

`TrainedStateFixture` now guards a converged scalar-signal world and reports the baseline that admitted it (´dec:harness:guarded-fixtures´) (´test:crate:trained-state-fixture-returns-its-measured-precondition´). Its fixed signal schema and population do not provide this witness's report-routed reference layout or two-regime precondition, so the witness still needs a subject-specific guarded setup. `run_seeded_sweep` is also available, but this claim is one authored reference scenario with one reported seed rather than an invariant over a seed distribution (´dec:harness:seeded-sweeps´).

`RiskBasis` publishes the blended raw score, blend weight, calibration scale, and sister probability, but not the raw anchor score. `PendingEntryView` deliberately exposes retained request inputs rather than component predictions, so neither surface can provide both unblended assessment-time predictors without inverting the blend (´dec:harness:probe-contract´).

`Assayer::full_health_report` publishes one `DriftHealth` entry per `ModelId`, but the producer still feeds every entry the same blended residual. `PendingRiskBasis` retains only the blended predictor, and the label path applies it to all risk and outcome-axis drift states; the audit records that divergence against the per-model rule (´tab:assayer:owner-remaining´) (´alg:monitoring:drift-cusums´).

## The witness · `sec:assayer:intent-the-anchor-catches-a-regime-change-first-witness`

The `learning_convergence` integration target remains the destination beside the stationary public-API smoke because the subject is labelled learning under changed evidence, not lifecycle, posterior geometry, or calibration (´dec:harness:convergence-split´) (´test:integration:scalar-signal-population-split-converges-directionally´).

A guarded reference-regime setup builds one seeded `World` through `WorldBuilder::interaction_templates`, registers one spatial axis and eight Sentinels, and uses two `CompetitiveCellSpec` declarations with `World::register_identity_with_cells` to establish ten cells per identity dimension. Its declared `RuntimeLayout` obtains 638 from `dimension_width` over the specification's block counts, and the guard reads back `World::runtime_layout` plus anchor and sister `PublishedModelBlock` values before returning. Construction failure and a result disagreement remain separate reports (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

The setup ingests authored reports whose three fixed routes share their coordinate and reporting distributions across both regimes. One route is the changed class: its coarse maximum raw z-score contrast is visible in the anchor projection while the remaining slot readings vary by Sentinel and tape position. An unchanged adverse control preserves a stable positive direction, and an unchanged benign control prevents a global positive shift from masquerading as changed-class recovery. `World::settle_cold_ramp_with` drives all three routes to the in-service coordinate state, and `World::flush_observations` establishes that publication before training begins (´dec:assayer:golden-report-stimulus´) (´tab:assayer:harness-implementation-library-roster´).

The old-regime stimulus is a subject-owned `PlaybackRow` sequence that interleaves benign labels for the changed class with the adverse and benign controls at a fixed declared cadence until the sister's 1,276-label evidence budget has been supplied. Its `PlaybackBarrierPolicy` orders `PlaybackBarrier::FlushLabels`, `PlaybackBarrier::FlushObservations`, and `PlaybackBarrier::FlushIdentityMaintenance` after every row; the setup guard then requires exact layout retention, negative changed-class medians, a positive adverse-control median, a negative benign-control median for both models, and finite health before it returns (´dec:harness:declarative-playback´) (´dec:harness:guarded-fixtures´).

The post-change sequence uses the same row type, reports, routes, coordinates, feature population, clock, action, eligibility, and cadence, but changes the changed-class label from benign to adverse. Each labelled row assesses first, copies the retained operational, sister, and anchor component predictors through the missing owned reading, computes each model's $r-\hat{\rho}$ residual, submits the label, and leaves queue completion to the same playback policy. Probe rows use the same fixed changed-class and control populations without submitting labels, so they do not train a model; their row boundaries still cross the selected barriers (´dec:harness:declarative-playback´) (´dec:harness:probe-contract´) (´alg:monitoring:drift-cusums´).

`PlaybackCheckpoint` callbacks after the pre-change probe, the thirtieth eligible post-change label, and the 1,276th eligible post-change label record the fixed-population medians from the owned component-score readings. The first result checkpoint requires a positive anchor median and a negative sister median; the second requires a positive sister median; every checkpoint requires a positive adverse-control median, a negative benign-control median, exact model widths, unchanged runtime layout, and finite health. `PlaybackProgress` identifies the first incomplete row if any application, barrier, or checkpoint fails (´dec:harness:declarative-playback´).

The result oracle uses raw predictors and the target definition directly. It does not use calibrated probabilities, reconstruct a component by dividing through the blend weight, or treat the guarded setup's old-regime signs as evidence for the post-change result. The width expectation alone uses `dimension_width`; no shared oracle exists for a regime-crossing count, so the sign assertions remain the subject's direct semantic boundary (´dec:harness:oracle-tier´) (´dec:harness:separate-validation´).

The fails-before is direct. Retaining only the blended score keeps the component reading absent and makes all published model-health traces identical; widening the anchor breaks the exact width guard; removing its measurement-only coordinate leaves its median negative at the early checkpoint; and feeding the sister the anchor projection makes the sister positive at that checkpoint.

## What is missing · `sec:assayer:intent-the-anchor-catches-a-regime-change-first-missing`

**Entry (Model-specific assessment-time residual retention)** · `entry:assayer:intent-the-anchor-catches-a-regime-change-first-model-residual-retention`

`PendingRiskBasis` must retain the operational, sister, and anchor predictors that assessment already computes, and its durable journal projection must carry the same values so replay preserves the assessment-time rule. The label path must update each risk model from its own retained predictor and update an outcome-axis drift state only from a reported axis value and that axis's retained predictor; eligibility remains unchanged. This is production conformance required by (´alg:monitoring:drift-cusums´), not a harness substitute.

**Entry (A guarded standard-width regime tape)** · `entry:assayer:intent-the-anchor-catches-a-regime-change-first-standard-regime-tape`

The construction verbs, competitive-cell fixture, playback runner, barriers, and reference-width proof have landed, so no new general tape runner or builder pass-through is missing. The `learning_convergence` subject still needs its typed three-route row and a guarded setup that composes those surfaces, runs the old regime, and returns only with the exact layout and measured old-regime sign baseline; its refusal reports the expected and observed baseline under the guarded-fixture contract (´dec:harness:guarded-fixtures´).

**Entry (Component-score and residual tracing)** · `entry:assayer:intent-the-anchor-catches-a-regime-change-first-width-and-residual-trace`

Model-width access is complete through `PublishedModelBlock`, and playback rows should own their subject-specific residual trace rather than adding a general recorder. The remaining probe work is an owned assessment-scoped component-score projection in `testing::probes`, reached through `World`, that copies the retained operational, sister, and anchor predictors for one assessment without exposing mutation or widening `RiskBasis`; it follows the pending-entry single-read boundary and the probe contract (´dec:harness:probe-contract´).

**Entry (The regime-change integration witness)** · `entry:assayer:intent-the-anchor-catches-a-regime-change-first-integration-witness`

The integration target still needs the guarded setup, typed playback rows, checkpoint assertions, trace diagnostics, claim citation, module-index row, and fails-before arms described above. It depends on the model-specific retention and owned component-score reading, while every other harness prerequisite is already present.

## Risks and open questions · `sec:assayer:intent-the-anchor-catches-a-regime-change-first-risks`

**Observation (Fortyfold is an evidence ratio)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-evidence-ratio-not-runtime`

Wall-clock duration measures matrix cost, scheduling, and hardware rather than statistical convergence. The witness keeps the stated factor through the exact live widths and their specification-derived evidence budgets, then keeps firstness through the signs observed at those checkpoints.

**Observation (The coarse contrast must survive standardisation)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-standardisation-control`

The cold ramp sees all three routes before labels begin, both regimes replay the same report distribution, and the setup guard records the old-regime signs after the ramp is in service. This prevents a moving coordinate system or an untrained fixture from impersonating anchor adaptation.

**Observation (Residual means prediction error)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-residual-identity`

The relevant residual is $r-\hat{\rho}$ from (´alg:monitoring:drift-cusums´), not the precision–covariance synchronisation residual. The former changes when the conditional outcome changes; the latter diagnoses numerical representation and has no claim to move during this stimulus.

**Observation (Calibration stays outside the oracle)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-calibration-independent-oracle`

The assertions use raw model scores and their training-target residuals rather than calibrated probabilities. A Platt refit may move probability scale during the transition without changing which model learned the new direction first.

**Observation (The exact first crossing is diagnostic)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-crossing-is-diagnostic`

The trace reports each model's first stable sign crossing but does not compare those counts to an invented tolerance. The binding evidence is the required sign separation at the two specification-derived budget checkpoints.

**Observation (Re-convergence is a reference-scenario claim)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-reconvergence-scope`

The two-observations-per-parameter budget describes statistical convergence, not a universal reversal theorem for a posterior carrying old evidence. Sister and anchor share the label-indexed and time-indexed rates under (´tab:risk:forgetting-rates´), so old-regime duration and feature excitation affect their reversal; the guarded authored scenario keeps the promised ordering without generalising it to every data distribution.

**Observation (The sign margins remain visible)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-sign-margin`

Zero is the semantic directional boundary, so no fitted epsilon replaces it. Diagnostics print every checkpoint median's distance from zero, allowing a near-boundary failure to be distinguished from a comfortably wrong sign without turning that distance into another acceptance threshold.

**Observation (Barriers remove scheduler timing from the result)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-barrier-determinism`

`World::flush_labels` does not cover cold-ramp observations, and `World::flush_identity_maintenance` does not cover them either. The explicit playback policy crosses the label, observation, and identity-maintenance queues in order before every checkpoint, while the virtual clock remains fixed; scheduler latency therefore cannot decide which model is first (´dec:harness:no-ad-hoc-waits´).

**Observation (Competitive cells have two fixture boundaries)** · `obs:assayer:intent-the-anchor-catches-a-regime-change-first-competitive-cell-boundary`

`World::register_identity_with_cells` now establishes the twenty cells through real observations rather than installing them directly, and later scenario traffic can offer more identity observations. The setup guard proves the initial set and every result checkpoint rechecks the exact runtime layout, so an unintended restructuring is a named fixture or checkpoint failure rather than hidden variation in the model width.

## Acceptance · `sec:assayer:intent-the-anchor-catches-a-regime-change-first-acceptance`

The integration target `learning_convergence` contains `learning_convergence::the_anchor_catches_a_regime_change_first`, whose documentation cites (´claim:risk:the-anchor-catches-a-regime-change-first´) and whose module index states the regime-change witness in full.

The setup report shows the guarded old-regime baseline, exact anchor and sister widths of 15 and 638, the derived eligible-label budgets of 30 and 1,276, their approximately 42.5 ratio, and the unchanged reference runtime layout computed by `dimension_width`.

The result report shows changed-class component-score medians and residual medians immediately before the flip, after thirty eligible post-change labels, and after 1,276; the anchor alone is positive at the first post-change checkpoint, both models are positive at the last, and the unchanged adverse and benign controls retain their respective positive and negative signs for both models at every checkpoint.

Every driven label is eligible, every playback row completes the declared label, observation, and identity-maintenance barriers, `PlaybackProgress` finishes, the per-model health readings are no longer a blended duplicate, and numerical health remains clean. The fixed seed and every sign distance are printed on failure; no cross-profile bit identity or elapsed-time claim is required.

The deliberately broken arms fail for their intended reasons: blended residual fan-out leaves the per-model health readings identical, widening the anchor violates the width baseline, removing the measurement-only coordinate misses the early anchor checkpoint, and training the sister on the anchor projection destroys the early separation.
