# Keeping a Systematically Mislabelled Entity Apart from the Aggregate Within Ten Labels · `plan:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels`

Keeping (´claim:wellness:a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels´) establishes through the public assess-label-report loop that concentrated outcome-sign corruption becomes a per-entity diagnostic while the population-level model remains useful, no later than the entity's tenth corrupted label.

## What the promise says, precisely · `sec:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-precision`

For entity $e$, let $n_e$ be the retained label count and let $C_e$ be the fraction of those labels whose reported outcome sign agrees with the effective-risk sign frozen into the corresponding assessment. The per-entity algorithm defines this fraction and compares it with aggregate rank discrimination $A$ (´alg:monitoring:per-entity-concordance´), while the completed health record fixes the frozen assessment-time input, bounded entity map, and detailed-report projection (´entry:health:per-entity-concordance´).

The default eligibility gate is $N_{\text{conc,min}}=5$: a count below five cannot flag and equality enters the comparison (´def:config:concordance-minimum-labels´). The flag predicate is strict, $n_e \geq N_{\text{conc,min}} \land C_e < A-\delta_{\text{conc}}$, with default $\delta_{\text{conc}}=0.25$ and equality explicitly unflagged (´def:config:concordance-deficit-threshold´).

The aggregate $A$ is AUC over the calibration buffer at a calibration refit, present only when the configured positive and negative class gates are met (´alg:monitoring:auc´). The health record retains that computed value until another refit rather than recomputing it when a report is requested (´dec:health:cached-discrimination´).

The ten-label figure is the promise's deadline rather than the configuration gate: the witness records the first flag, proves that it cannot precede the five-label gate, and proves that it exists by the tenth corrupted label. Ten opposite signs make $C_e=0$, so the measured final deficit is $A$ itself and must be strictly greater than $0.25$.

The result is diagnostic only. Raising the flag changes no assessment, model, threshold, rate, or routing decision (´inv:monitoring:report-only´).

## What the code offers today · `sec:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-code`

The public route is `World::request` or `with_score_verified`, `World::derive_for_request`, `LabelSpec`, `World::label`, `World::flush_labels`, and `Assayer::full_health_report`. The world harness's reusable `cycle_request` helper already performs the assess-label-flush round, and its `scenario_with` plus `score_verified_request_schema` helpers build the deterministic signal-bearing world.

On the production path, the label path compares the label's risk-target sign with the pending assessment's frozen effective-risk sign (´claim:labelling:the-risk-target-is-a-sign-and-not-a-magnitude´) and updates `PerEntityConcordanceTracker`; the detailed health query projects that tracker with the configured gate, configured deficit, and current `auc_aggregate` into `SystemHealthReport.concordance.per_entity`. The detailed query is the intended host surface for readings that retain entity keys and supporting values (´dec:health:tiered-queries´).

The mechanism already has narrow unit witnesses: one covers flagging plus least-recently-observed eviction (´test:unit:per-entity-concordance-flags-and-evicts-least-recent´), and one covers the configured count gate and strict deficit boundary (´test:unit:per-entity-concordance-honours-configured-gate-and-deficit´). Neither passes a labelled assessment through the owner and back out through the public report.

The nearest integration witness trains the same request-scoped scalar signal through the public cycle until adverse and benign populations rank apart (´test:integration:scalar-signal-population-split-converges-directionally´). The nearest detailed-health pattern drives a labelled stream and reads a diagnostic solely from `full_health_report` (´test:integration:association-separates-a-keyed-sentinel-from-a-hash-fed-one´).

The standing gap register has no entry covering this promise (´sec:assayer:testing-plan-gaps´).

## The witness · `sec:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-witness`

The witness occupies a dedicated per-entity concordance integration-test module, is named for the promised deadline, and carries the existing intent claim as its claim mint when that mint moves out of the intent catalogue (´sec:assayer:intents´).

- Setup: build a seeded default-channel scenario with `score_verified_request_schema`, then drive six hundred alternating low-score benign and high-score adverse labels through distinct population entities; this extends the proven signal-split fixture through a default refit boundary and leaves a mature, balanced calibration buffer.
- Baseline: read `full_health_report`, require a present aggregate AUC above one half, and retain that value before introducing the corrupted entity; failure here identifies a population fixture that never established the comparator rather than a concordance failure.
- Control: use the same high-score request shape for a correctly labelled control entity and the target entity, assert that every frozen `rho_eff` remains positive, report adverse labels for the control, and report benign labels marked as ground truth for the target.
- Stimulus: interleave ten control cycles with ten target cycles so both entities see the same learned side of the population while only the target's reported sign is inverted; each cycle ends with `flush_labels`, making every subsequent report a post-label observation.
- Observation: after each target cycle, read the target and control entries from `SystemHealthReport.concordance.per_entity` using `World::entity`, record the first target count whose `flagged` field is true, and keep the aggregate AUC from the same report beside the entity reading.
- Assertion: before five target labels the flag is false; the first flagged count lies between five and ten inclusive; at ten the target reports `label_count == 10`, `concordance == 0`, `auc_aggregate - concordance > 0.25`, and `flagged == true`; the ten-label control reports concordance one and remains unflagged.
- Fails-before: an implementation that omits the label-time observation, keys the map by anything broader than the entity, compares against no aggregate AUC, reads the post-update estimate instead of the frozen estimate, reverses the deficit inequality, or fails to publish the tracker leaves the target absent, dilutes its zero concordance, or keeps its flag false despite the independently established comparator.

No wall-clock assertion, virtual-time advance, random outcome generation, floating tolerance, Sentinel report, or identity-dimension lifecycle participates in the witness. The exact zero and one concordances come from integer agreement counts, and the AUC is tested only across margins relevant to the predicate.

## What is missing · `sec:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-missing`

The shared harness already exposes every required production action and observation. The only prerequisite is a test-local fixture that establishes the aggregate comparator without copying the engine's concordance calculation.

**Entry (A local trained-population fixture)** · `entry:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-trained-population-fixture`

Approximately 25 lines in that integration-test module construct scored requests, drive the balanced training stream, and return the published aggregate AUC after asserting its presence and above-chance margin. The fixture reuses `score_verified_request_schema`, `with_score_verified`, `cycle_request`, and `LabelSpec`; it depends on the established convergence shape (´test:integration:scalar-signal-population-split-converges-directionally´) and on no other entry or lane.

No new `World` method, tape, clock seam, drift budget, liveness deadline, or production accessor is part of this entry.

## Risks and open questions · `sec:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-risks`

**Observation (Ten labels is an upper bound over a five-label gate)** · `obs:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-deadline-semantics`

The intent slug says “within ten labels,” while its prose says “after ten labels,” and the specification permits the default flag at five. The witness reads ten as the latest acceptable count and preserves the specified five-label lower gate; an exact transition on the tenth label would instead require a different configured gate or a specification change.

**Observation (The aggregate is a cached prerequisite)** · `obs:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-cached-aggregate`

Per-entity observations advance on every label, but $A$ advances only on a refit. The baseline report therefore proves that $A$ exists before the ten-label window starts, and the assertion reads the aggregate and entity entry from one report; scheduler timing never substitutes for that precondition (´dec:health:cached-discrimination´).

**Observation (The compared quantities use different summaries)** · `obs:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-metric-scales`

$C_e$ is sign agreement and $A$ is rank discrimination, so the test does not manufacture equality between them or reuse a pairwise holdout AUC as the published aggregate. It asserts each public value independently and then applies only the strict deficit comparison the specification defines (´alg:monitoring:per-entity-concordance´).

**Observation (The only asynchronous edge ends at the label barrier)** · `obs:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-label-barrier`

Label submission acknowledges queue acceptance rather than completion, while `cycle_request` finishes with `flush_labels`. Reports are read only after that barrier; no sleep, polling loop, clock movement, or widened tolerance masks a missed publication.

## Acceptance · `sec:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels-acceptance`

The implementation-lane report establishes all of the following.

- The intent claim has moved from the intent catalogue (´sec:assayer:intents´) to the integration test's documentation, the new test has its own unique integration-test label, and generated indexes resolve both without duplicate or dangling labels.
- The test uses only the public scenario cycle and detailed health report, with no construction of `PerEntityConcordanceTracker`, no mutation of an implementation to obtain fails-before evidence, and no access to owner state.
- The baseline report contains an above-chance aggregate AUC before the target stream, and every target assessment has the positive frozen risk sign that makes the benign report an actual disagreement in the implemented statistic.
- The report evidence names the first flagged target count, the tenth-label target count and concordance, the strict deficit, the final flag, and the matched control's unflagged concordance.
- Package formatting, linting, the focused integration test, package tests with all targets and features, the no-default-features spot check, documentation tests, and release-mode package tests are green, with exact commands, exit codes, and wall times reported.
- The described broken behaviours fail the witness through its public assertions, while the shipped implementation passes without timing retries or probabilistic allowances.
