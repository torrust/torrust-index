# Keeping a Systematically Mislabelled Entity Apart from the Aggregate Within Ten Labels · `plan:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels`

Keeping (´claim:wellness:a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels´) establishes through the public assess-label-report loop that concentrated outcome-sign corruption becomes a per-entity diagnostic while the population-level model remains useful, no later than the entity's tenth corrupted label.

## What the promise says, precisely · `sec:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-precision`

For entity $e$, let $n_e$ be the retained label count and let $C_e$ be the fraction of those labels whose reported outcome sign agrees with the effective-risk sign frozen into the corresponding assessment. The per-entity algorithm defines this fraction and compares it with aggregate rank discrimination $A$ (´alg:monitoring:per-entity-concordance´), while the completed health record fixes the frozen assessment-time input, bounded entity map, and detailed-report projection (´entry:health:per-entity-concordance´).

The default eligibility gate is $N_{\text{conc,min}}=5$: a count below five cannot flag and equality enters the comparison (´def:config:concordance-minimum-labels´). The flag predicate is strict, $n_e \geq N_{\text{conc,min}} \land C_e < A-\delta_{\text{conc}}$, with default $\delta_{\text{conc}}=0.25$ and equality explicitly unflagged (´def:config:concordance-deficit-threshold´).

The aggregate $A$ is AUC over the calibration buffer at a calibration refit, present only when the configured positive and negative class gates are met (´alg:monitoring:auc´). The health record retains that computed value until another refit rather than recomputing it when a report is requested (´dec:health:cached-discrimination´).

The ten-label figure is the promise's deadline rather than the configuration gate: the witness records the first flag, proves that it cannot precede the five-label gate, and proves that the flag is present after the tenth corrupted label. Ten opposite signs make $C_e=0$, so the measured final deficit is $A$ itself and must be strictly greater than the specified $0.25$ threshold.

The result is diagnostic only. Raising the flag changes no assessment, model, threshold, rate, or routing decision (´inv:monitoring:report-only´).

## What the code offers today · `sec:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-code`

The finished harness provides `World::trained_state`, which returns a `TrainedStateFixture` only after its `TrainedStateBaseline` has checked the balanced training counts, held-out class counts, endpoint gap, and pairwise rank (´dec:harness:guarded-fixtures´), (´dec:harness:separate-validation´), (´test:crate:trained-state-fixture-returns-its-measured-precondition´). That shared fixture already owns the deterministic score-verified population this witness needs; the finished roster records it as library infrastructure rather than target-local setup (´tab:assayer:harness-implementation-library-roster´).

After setup, `World::request` and `with_score_verified` build the matched requests, while `cycle_request` uses `World::derive_for_request`, `LabelSpec`, `World::label`, and `World::flush_labels` for one assess-label-publication round. The label barrier is therefore part of the reusable cycle rather than a wait authored by this witness (´dec:harness:no-ad-hoc-waits´), (´tab:assayer:harness-implementation-library-roster´).

On the production path, the label path compares the label's risk-target sign with the pending assessment's frozen effective-risk sign (´claim:labelling:the-risk-target-is-a-sign-and-not-a-magnitude´) and updates `PerEntityConcordanceTracker`; the detailed health query projects that tracker with the configured gate, configured deficit, and current `auc_aggregate` into `SystemHealthReport.concordance.per_entity`. The detailed query is the intended host surface for readings that retain entity keys and supporting values (´dec:health:tiered-queries´).

The mechanism already has narrow unit witnesses: one covers flagging plus least-recently-observed eviction (´test:unit:per-entity-concordance-flags-and-evicts-least-recent´), and one covers the configured count gate and strict deficit boundary (´test:unit:per-entity-concordance-honours-configured-gate-and-deficit´). Neither passes a labelled assessment through the owner and back out through the public report.

The nearest integration witness trains the same request-scoped scalar signal through the public cycle until adverse and benign populations rank apart (´test:integration:scalar-signal-population-split-converges-directionally´). The nearest detailed-health pattern drives a labelled stream and reads a diagnostic solely from `Assayer::full_health_report` (´test:integration:association-separates-a-keyed-sentinel-from-a-hash-fed-one´). Neither test cites the intent claim or carries its entity-specific corrupted-label assertion.

The standing gap register has no entry covering this promise (´sec:assayer:testing-plan-gaps´).

## The witness · `sec:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-witness`

The `calibration_discrimination_convergence` integration target is the decision-assigned destination for this per-entity concordance witness (´dec:harness:convergence-split´). Its function is `systematically_mislabelled_entity_parts_from_aggregate_within_ten_labels`; its documentation cites the existing intent claim, which remains minted in the intent catalogue (´sec:assayer:intents´).

- Setup: call `World::trained_state` with a declared seed and retain the returned `TrainedStateBaseline` beside its `World`; the fixture's guard establishes the trained score-verified population before this witness makes any result assertion (´dec:harness:guarded-fixtures´), (´dec:harness:separate-validation´).
- Baseline: read `Assayer::full_health_report` through `World::assayer`, require `report.discrimination` and its `auc_aggregate` to be present, require $A>0.25$, and retain $A$ before introducing the corrupted entity; this is the exact precondition that makes zero concordance cross the specified strict deficit.
- Control: use `World::request` and `with_score_verified` to give a correctly labelled control entity and the target entity the same high-score, verified request shape; for every returned assessment require `risk.rho_eff > 0`, submit `LabelSpec::adverse(...).ground_truth()` for the control, and submit `LabelSpec::benign(...).ground_truth()` for the target.
- Stimulus: for each target count from one through ten, run the control cycle and then the target cycle through `cycle_request`; each call returns only after `World::flush_labels`, so the following report observes both completed labels without a sleep, poll, deadline, or clock movement (´dec:harness:no-ad-hoc-waits´).
- Observation: after each target cycle, read one `Assayer::full_health_report`, locate both entries in `report.concordance.per_entity` with `World::entity`, record the first target `label_count` whose `flagged` field is true, and retain the report's cached aggregate AUC beside those entity readings.
- Assertion: target counts below five remain unflagged; the first flagged count is five; after the tenth target label the target has `label_count == 10`, `concordance == 0`, `auc_aggregate - concordance > 0.25`, and `flagged == true`; the control has `label_count == 10`, `concordance == 1`, and remains unflagged.
- Fails-before: an implementation that omits the label-time observation, keys the map by anything broader than the entity, compares against no aggregate AUC, reads the post-update estimate instead of the frozen estimate, reverses the deficit inequality, or fails to publish the tracker leaves the target absent, dilutes its zero concordance, or keeps its flag false despite the independently established comparator.

No wall-clock assertion, virtual-time advance, random outcome generation, floating tolerance, Sentinel report, or identity-dimension lifecycle participates in the witness. The exact zero and one concordances come from integer agreement counts, and the AUC is tested only across margins relevant to the predicate.

## What is missing · `sec:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-missing`

The shared harness exposes every required action, barrier, and public observation. The only missing item is the integration witness itself.

**Entry (The guarded trained population feeds the missing witness)** · `entry:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-trained-population-fixture`

The `calibration_discrimination_convergence` target adds `systematically_mislabelled_entity_parts_from_aggregate_within_ten_labels`, consumes `World::trained_state`, checks the witness-specific published-AUC precondition, drives the ten matched pairs, and documents the intent claim. It adds no local fixture: topic targets reuse shared fixture behaviour under the convergence split (´dec:harness:convergence-split´), and `World::trained_state` already carries the trained-population guard (´test:crate:trained-state-fixture-returns-its-measured-precondition´).

No new `World` method, playback row, probe, clock seam, oracle, sweep, liveness deadline, or production accessor is part of this entry.

## Risks and open questions · `sec:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-risks`

**Observation (Ten labels is an upper bound over a five-label gate)** · `obs:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-deadline-semantics`

The intent slug says “within ten labels,” while its prose says “after ten labels,” and the specification permits the default flag at five. With $C_e=0$ and $A>0.25$, the specified predicate makes five the first flagged count; the witness also proves that the flag is still present after ten labels. Requiring the transition itself on the tenth label would contradict the default gate and predicate.

**Observation (The aggregate is a cached prerequisite)** · `obs:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-cached-aggregate`

Per-entity observations advance on every label, but $A$ advances only on a refit. `TrainedStateBaseline` guards class counts and held-out separation, not publication of `auc_aggregate`, so the baseline report must prove that the cached comparator exists and exceeds $0.25$ before the ten-label window starts; each later assertion reads the aggregate and entity entry from one report (´dec:health:cached-discrimination´), (´dec:harness:separate-validation´).

**Observation (The compared quantities use different summaries)** · `obs:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-metric-scales`

$C_e$ is sign agreement and $A$ is rank discrimination, so the test does not manufacture equality between them or reuse a pairwise holdout AUC as the published aggregate. It asserts each public value independently and then applies only the strict deficit comparison the specification defines (´alg:monitoring:per-entity-concordance´).

**Observation (The only asynchronous edge ends at the label barrier)** · `obs:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-label-barrier`

Label submission acknowledges queue acceptance rather than completion, while `cycle_request` finishes with `World::flush_labels`. That barrier covers the label-derived concordance this witness reads but does not claim to drain the separate observation queue; reports follow the target cycle directly, with no sleep, polling loop, clock movement, or widened tolerance (´dec:harness:no-ad-hoc-waits´).

## Acceptance · `sec:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-acceptance`

The implementation-lane report establishes all of the following.

- The `calibration_discrimination_convergence` target contains `systematically_mislabelled_entity_parts_from_aggregate_within_ten_labels`; its documentation cites the claim retained in the intent catalogue (´sec:assayer:intents´), mints one unique integration-test label, and generated indexes resolve both without duplicate or dangling labels.
- The test consumes `World::trained_state`, uses `cycle_request` and the public detailed health report, constructs no `PerEntityConcordanceTracker`, mutates no implementation to obtain fails-before evidence, and accesses no owner state.
- Before the target stream, the report contains aggregate AUC strictly greater than the specified $0.25$ deficit; every control and target assessment has the positive frozen risk sign that makes the adverse control agree and the benign target disagree.
- The evidence names the first flagged target count of five, the tenth-label target count and zero concordance, the strict deficit, the final flag, and the matched control's unit concordance and unflagged state.
- Package formatting, linting, the focused integration target, and package tests are green, with exact commands, exit codes, and wall times reported.
- The described broken behaviours fail the witness through its public assertions, while the shipped implementation passes without timing retries or probabilistic allowances.
