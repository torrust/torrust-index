# Cross-Sentinel Correlation Convergence Witness · `plan:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge`

This plan keeps (´claim:wellness:cross-sentinel-correlation-is-caught-only-once-interactions-converge´) with a public assess-and-label witness in which every Sentinel and every fleet summary is individually uninformative, while a declared cross-Sentinel product moves from near-chance discrimination at the early checkpoint to useful discrimination at the mature checkpoint.

## What the promise says, precisely · `sec:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-promise`

The measured quantity is host-side, tie-corrected rank discrimination over an independent balanced holdout population, computed from `RiskAssessment::risk.p_bad` after the model-owner barrier; this is the same rank statistic the monitoring surface defines (´alg:monitoring:auc´) and the assessment schema exposes (´schema:output:assessment´).

The early checkpoint is exactly two thousand successfully processed labels and passes only when the joint holdout remains in the near-chance band, AUC below 0.55; the mature checkpoint is exactly eight thousand successfully processed labels and passes only when the same joint holdout reaches at least the moderate band, AUC at or above 0.65 (´tab:monitoring:interpretation´).

Each Sentinel-alone holdout remains below 0.55 at both checkpoints, and an otherwise identical world constructed without interaction templates remains below 0.55 on the joint holdout at both checkpoints. These controls make “ordinary alone” and “only in the correlation” measured conditions rather than descriptions of the generator.

The two label counts are scenario thresholds minted by the intent, not configuration constants. The specification places cross-Sentinel maturity beyond roughly five thousand labels in its deployment reading aid (´tab:warmup:milestones´), describes interaction maturity as the interval from roughly twice to ten times the model width (´tab:warmup:stages´), and fixes co-occurrence frequency as the quantity governing the cascade (´tab:feature:interaction-convergence´).

The structural premise is that a linear model cannot express the conjunction without a product feature (´mot:feature:interaction-overview´), the wildcard template supplies one product for every unordered Sentinel pair (´def:feature:template-wildcard´), and the cross-Sentinel gap persists until that product has converged (´cav:limitation:cross-sentinel-gap´). The convergence window and compound-detection analysis establish the operational reading (´tab:detection:convergence-window´) and (´disc:detection:compound´).

The standing testing-plan entry covering this promise is the open convergence-depth gap (´entry:assayer:gap-convergence-depth´); its long-streaming dependency is the open tapes-and-fixtures stage (´entry:assayer:harness-stage-tapes´).

The governing records require templates to survive layout changes by semantic identity (´dec:vector:semantic-templates´), require products to be formed from raw operands before the vector is standardised once (´dec:vector:unstandardised-bases´), and require construction to reject an invalid declaration eagerly (´dec:construction:eager-validation´).

## What the code offers today · `sec:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-code-today`

The production builder already carries `AssayerBuilder::interaction_templates`, validates every template during construction, and passes the fixed list into the real engine as part of the host construction contract (´req:host:construction´). Templates cannot change after construction, matching the fixed-schema boundary (´cav:construction:schema-fixity´).

The interaction implementation provides the wildcard as `InteractionTemplate::type3(feature_offset)`, and the dimension map expands it over unordered pairs and compiles the output and operand indices through the specified compilation pipeline (´alg:dimension:compilation-pipeline´). Offset zero selects the cell novelty maximum from the extraction block described by (´def:extraction:slot´).

The external construction path is incomplete: `InteractionTemplate` is private to the crate even though the public builder method accepts it, and the world harness neither stores templates on `WorldBuilder` nor forwards them to `AssayerBuilder`. A nonempty interaction set is therefore not constructible from an integration test today.

Once constructed, `World` already supplies the rest of the public path: deterministic seeded scenarios, ordered Sentinel registration with publication barriers, report ingestion, requests carrying several Sentinel coordinates, batched assessment and derivation, `LabelSpec`, label submission, and model-owner flushing. These operations drive the specified assessment and label paths (´req:runtime:assessment-interface´), (´alg:runtime:assessment-pipeline´), and (´alg:runtime:update-path´).

The golden-report helpers supply mutable reports and public cell builders under the authored-stimulus discipline (´dec:assayer:golden-report-stimulus´), while the existing Sentinel-legibility witness shows how identical-shape quiet and loud regions are routed through two real Sentinel slots by a seeded stream (´test:integration:association-separates-a-keyed-sentinel-from-a-hash-fed-one´).

The nearest convergence witness supplies the output oracle: a public assess-then-label population split, a held-out probability gap, and host-side pairwise AUC (´test:integration:scalar-signal-population-split-converges-directionally´). It is a short directional smoke and has neither sparse interactions nor two checkpoints.

The aggregate block and each per-Sentinel slot remain live in the proposed witness (´tab:feature:aggregate-block´) and (´def:extraction:slot´); the generator and the template-free world neutralise them instead of bypassing them.

## The witness · `sec:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-witness`

The setup constructs two same-seed worlds with empty signal and identity schemas, one default channel, and four registered Sentinels reading identical report surfaces. The treatment world declares one wildcard interaction over extraction offset zero; the control world declares no interactions.

Each report surface contains a root and a ladder of disjoint dyadic cells whose novelty maxima span quiet through loud values while every other report property is held fixed. The same surface is ingested under all four Sentinel names, so a cell rank has the same meaning in every slot.

The training tape is partitioned into blocks of four hundred labels. Three hundred and ninety-six background rows carry balanced adverse and benign outcomes with no Sentinel coordinates, while four joint rows carry all four coordinates, giving the relevant interaction exactly one per cent co-occurrence.

The four joint pattern families are adverse concordant-high, adverse concordant-low, benign discordant-high-low, and benign discordant-low-high for the target pair. The other two Sentinels receive the complementary ranks, leaving every joint request with the same four-value multiset.

Seeded rotations exchange the concordant and discordant orientations without changing which two Sentinel identities form the target pair, and choose ranks within quiet and loud strata. At each checkpoint every Sentinel has the same rank distribution in both outcome classes, every request has the same aggregate multiset, the two classes are balanced, and only pair membership distinguishes them.

Both worlds receive each request and label in the same order. Labels use `Allow` and adverse or benign `LabelSpec` values, so both the operational and eligible models learn from the stream; submissions are flushed in bounded batches, and a checkpoint is observed only after all preceding acknowledgements and the model-owner barrier.

An independent balanced holdout seed generates enough joint and Sentinel-alone probes for tie-corrected AUC without submitting their class labels to either model. The same holdout is assessed after the two-thousand-label barrier and again after the eight-thousand-label barrier in both worlds.

The observation records, for every checkpoint and world, the joint AUC, four Sentinel-alone AUCs, classwise mean risk, and minimum and maximum risk. A failure prints the seed, label checkpoint, co-occurrence count, world role, and all readings, making a threshold crossing reproducible.

The assertions require the treatment joint AUC below 0.55 at two thousand labels and at least 0.65 at eight thousand, every Sentinel-alone AUC below 0.55 at both checkpoints, and the control joint AUC below 0.55 throughout. The treatment's mature AUC must also exceed the control by more than the harness AUC tolerance.

A deliberately broken implementation that drops wildcard triples, multiplies the wrong slot positions, or leaves interaction weights at the prior follows the control trajectory and fails the mature treatment assertion. An implementation that leaks the outcome through a slot, aggregate, ordering pattern, or fixture imbalance fails a Sentinel-alone, early, or template-free control assertion.

## What is missing · `sec:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-missing`

**Entry (A constructible semantic wildcard declaration)** · `entry:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-constructible-wildcard`

The integration harness needs a construction-time method that declares the wildcard product by semantic feature name and forwards the internal template to `AssayerBuilder`; the change is approximately thirty lines including its focused construction test and has no dependency on another lane.

**Entry (The balanced sparse-correlation tape)** · `entry:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-balanced-tape`

A deterministic fixture needs the four equal-shape report surfaces, block generator, independent holdout generator, and self-checks for class balance, per-Sentinel marginal equality, fixed aggregate multisets, and one per cent co-occurrence. The fixture is approximately one hundred and forty lines and depends on the constructible wildcard entry.

**Entry (Checkpointed bulk label driving)** · `entry:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-bulk-driver`

The one-label `cycle_request` helper flushes once per label and is too expensive for two worlds at the mature horizon. A tape runner needs bounded assessment batches, ordered label submission, periodic liveness checks, model-owner barriers at requested label counts, and matched-world diagnostics; it is approximately ninety lines and is a focused slice of (´entry:assayer:harness-stage-tapes´), depending on the balanced tape.

**Entry (A reusable tie-corrected discrimination assertion)** · `entry:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-auc-assertion`

The harness advertises but does not implement its Stage-three AUC assertion. A helper that computes ties, rejects empty classes and non-finite risks, compares a named band, and prints the population is approximately forty lines and depends only on the existing tolerance bundle.

## Risks and open questions · `sec:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-risks`

**Observation (The detection bands are guidance rather than a binding threshold)** · `obs:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-detection-band`

The specification explicitly makes the AUC interpretation bands non-binding. Using their near-chance and moderate edges gives “evades” and “detected” existing meanings and matches the nearest convergence test; the alternative is a new intent-level probability-lift threshold, which has no present authority and requires a specification decision before it can be an oracle.

**Observation (The label horizons depend on the declared co-occurrence profile)** · `obs:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-cooccurrence-profile`

One per cent is the upper edge of the wildcard range in the convergence cascade and yields twenty informative joint rows by the early checkpoint and eighty by the mature checkpoint. A different rate changes the effective evidence count and therefore changes the promise being tested; the tape reports both total and joint counts.

**Observation (The public interaction declaration is not yet a coherent host surface)** · `obs:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-template-surface`

Three implementation routes exist: export the legacy offset enum, land the semantic declaration type required by the vector record, or expose a narrow hidden test-support method that translates a semantic cell-novelty selector internally. The witness assumes the narrow method, which keeps the test at the public harness boundary without making the legacy offset representation a host contract; a production-facing repair remains a separate decision.

**Observation (Ordering and assessment-side state can counterfeit a milestone)** · `obs:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-ordering`

Online updates are order-sensitive, concordance thresholds evolve on assessment traffic, and early holdout probes remain pending. Fixed block rotations, matched treatment and control order, bounded flushing, independent holdout identifiers, and a pending-capacity assertion hold those effects equal; repeated fixed-seed runs establish that neither checkpoint rests on scheduler timing.

**Observation (AUC can change abruptly on a small discrete holdout)** · `obs:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-numerical-fragility`

The report ladder and holdout use several ranks per stratum rather than four repeated points, tie correction is mandatory, and the mature margin is recorded against both 0.65 and the control. If the fixed-seed result sits within the harness AUC tolerance of either boundary, the fixture lacks a stable witness even when one run passes.

## Acceptance · `sec:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-acceptance`

The implementation report identifies the integration-test file, its claim and test mints, the wildcard selector it declares, the treatment and control configurations, the fixed training and holdout seeds, the exact total and joint-label counts at each checkpoint, and the batch and barrier discipline.

The report prints the joint and four Sentinel-alone AUCs for both worlds at both checkpoints, their distances from the applicable bands, the treatment-to-control mature margin, classwise risk summaries, and clean finite-health evidence.

The report demonstrates that the treatment is near chance at two thousand labels, reaches at least moderate discrimination by eight thousand, stays near chance when each Sentinel is assessed alone, and has no corresponding late lift without interactions.

The report includes focused test output, package formatting and lint verdicts, package tests under the feature combinations required by the repository, the deterministic repeated-run result, wall time for the long witness, and any remaining deviation from this plan.
