# Cross-Sentinel Correlation Convergence Witness · `plan:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge`

This plan keeps (´claim:wellness:cross-sentinel-correlation-is-caught-only-once-interactions-converge´) with a public assess-and-label witness in which every Sentinel and every fleet summary is individually uninformative, while a declared cross-Sentinel product moves from near-chance discrimination at the early checkpoint to useful discrimination at the mature checkpoint.

## What the promise says, precisely · `sec:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-promise`

The measured quantity is host-side rank discrimination over an independent balanced holdout population, computed from `RiskAssessment::risk.p_bad` by the tie-aware `pairwise_rank` specification-formula oracle with twenty examples from each class and exact ties credited one half; this is the statistic the monitoring surface defines and class-gates (´alg:monitoring:auc´), the assessment schema exposes (´schema:output:assessment´), and the finished oracle tier implements independently (´dec:harness:oracle-tier´).

The early checkpoint is exactly two thousand successfully processed labels and passes only when the treatment world's joint holdout remains in the near-chance band, AUC below 0.55; the mature checkpoint is exactly eight thousand successfully processed labels and passes only when that same holdout reaches at least the moderate band, AUC at or above 0.65 (´tab:monitoring:interpretation´).

Each Sentinel-alone holdout remains below 0.55 at both checkpoints, and an otherwise identical world constructed without interaction templates remains below 0.55 on the joint holdout at both checkpoints. These controls make “ordinary alone” and “only in the correlation” measured conditions rather than descriptions of the generator.

The two label counts are scenario thresholds minted by the intent, not configuration constants. The specification places cross-Sentinel maturity beyond roughly five thousand labels in its deployment reading aid (´tab:warmup:milestones´), describes interaction maturity as the interval from roughly twice to ten times the model width (´tab:warmup:stages´), and places wildcard co-occurrence between one tenth and one per cent in the cascade that governs convergence (´tab:feature:interaction-convergence´).

The structural premise is that a linear model cannot express the conjunction without a product feature (´mot:feature:interaction-overview´), the wildcard template supplies one product for every unordered Sentinel pair (´def:feature:template-wildcard´), and the cross-Sentinel gap persists until the relevant product has converged (´cav:limitation:cross-sentinel-gap´). The convergence-window analysis says that aggregate features provide only coarse coverage while wildcard products mature (´tab:detection:convergence-window´), and the compound-detection analysis says that the named Sentinel slots preserve the identities those products join (´disc:detection:compound´).

The standing convergence-depth gap remains open (´entry:assayer:gap-convergence-depth´). Its former tapes-and-fixtures dependency is historical rather than open: the complete builder, shared playback runner, oracle tier, guarded fixtures, and closed barrier set are finished capabilities (´entry:assayer:harness-complete-builder´), (´entry:assayer:harness-tape-runner´), (´entry:assayer:harness-oracle-tier´), (´entry:assayer:harness-guarded-fixtures´), and (´entry:assayer:harness-closed-barriers´).

The governing records require templates to survive layout changes by semantic identity (´dec:vector:semantic-templates´), require products to be formed from raw operands before the vector is standardised once (´dec:vector:unstandardised-bases´), and require construction to reject invalid declarations eagerly (´dec:construction:eager-validation´).

## What the code offers today · `sec:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-code-today`

`WorldBuilder::interaction_templates` now stores the test declaration and forwards it to `AssayerBuilder::interaction_templates`, while the gated testing surface re-exports `InteractionTemplate`; invalid declarations reach the engine's typed construction error instead of being ignored (´entry:assayer:harness-complete-builder´), (´test:crate:world-builder-forwards-invalid-interaction-template´). Construction is no longer the blocker the earlier plan described.

`InteractionTemplate::type3(0)` constructs the wildcard over extraction-relative offset zero, the cell view's novelty maximum (´tab:extraction:reference-index´), and the dimension map expands it over unordered pairs before compiling output and operand indices (´alg:dimension:compilation-pipeline´). The wildcard population count is already pinned independently (´test:unit:type3-feature-count-is-combinations´), and interaction assembly is pinned to raw operands before standardisation (´test:unit:compute-interactions-unstandardised´).

The unified `Scenario` over one `World` supplies deterministic construction, Sentinel registration, report ingestion, owned assessment results, label submission, and queue-specific barriers; the finished roster records that common surface (´dec:harness:single-scenario´), (´tab:assayer:harness-implementation-library-roster´). This witness uses `World::register_sentinels`, `World::receive_report`, `World::derive_for_request`, `World::label`, `World::flush_observations`, and `World::flush_labels`; it needs no sleep, poll, or test-authored deadline under the declared-barrier rule (´dec:harness:no-ad-hoc-waits´).

Long training input now runs through subject-owned `PlaybackRow` values and the shared `playback` runner. `PlaybackBarrierPolicy`, `PlaybackBarrier`, `PlaybackCheckpoint`, `PlaybackProgress`, and `PlaybackBatchSize` own the ordered completion policy, exact checkpoint positions, first-incomplete diagnostic, and bounded materialisation (´dec:harness:declarative-playback´), (´entry:assayer:harness-tape-runner´); the real label-path witness proves a selected label barrier completes before its checkpoint reads publication (´test:crate:selected-label-barrier-completes-before-checkpoint´).

The oracle tier now provides `pairwise_rank`, including tie credit, per-class gates, and a unit-interval result (´entry:assayer:harness-oracle-tier´), with a hand-counted tie witness independent of the production rank-sum route (´test:crate:pairwise-rank-oracle-counts-ties-by-pairs´). The former plan's local AUC fold and proposed reusable AUC helper are therefore obsolete.

The finished probe projections `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView` cross only their documented boundaries and return owned readings (´dec:harness:probe-contract´), but this witness reads the owned `RiskAssessment` returned by assessment and needs none of them. `TrainedStateFixture` is a guarded scalar-signal state and is intentionally not reused: this witness begins cold and must observe its own early and mature states, while its tape constructor guards the balance and marginal-equality preconditions separately from the result oracle (´dec:harness:guarded-fixtures´), (´dec:harness:separate-validation´). `run_seeded_sweep` exists for multi-seed invariant packs (´dec:harness:seeded-sweeps´), but this threshold witness uses one declared replay seed and reports it rather than claiming a seed-wide invariant.

Existing tests touch every principal mechanism without keeping this promise: the builder test establishes template forwarding, the wildcard-count and raw-product tests establish expansion and assembly, and the nearest public convergence test establishes directional scalar learning over five hundred labels with held-out AUC above 0.65 (´test:integration:scalar-signal-population-split-converges-directionally´). None compares interaction and template-free worlds at the two promised label checkpoints, and no test mints or cites this plan's claim.

## The witness · `sec:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-witness`

The `posterior_convergence` integration target owns `posterior_convergence::cross_sentinel_correlation_is_caught_only_once_interactions_converge`, because the target's declared subject is interaction geometry under convergence (´dec:harness:convergence-split´).

The setup constructs two `Scenario` values through `scenario_with` from one declared seed, with empty signal and identity schemas, one default channel, and four registered Sentinels reading equal-shape report surfaces. The treatment world passes `InteractionTemplate::type3(0)` through `WorldBuilder::interaction_templates`; the control world passes an empty template list.

Each report surface uses `make_cell_report` to build a root and a ladder of disjoint dyadic cells whose cell-view novelty maxima span quiet through loud values while every other report property is fixed. The same surface is ingested under all four Sentinel names through `World::receive_report`, so a rank denotes the same authored stimulus in every slot under the golden-report discipline (´dec:assayer:golden-report-stimulus´).

Before the first label, both worlds call `World::settle_cold_ramp_with` on the same representative quiet and loud requests. That queue-driving helper reads the published standardisation phase after `World::flush_observations` and refuses stalled progress, so the treatment-control comparison does not begin from scheduler-selected positions on the coordinate ramp.

One `TestRng` seed constructs the tape in blocks of four hundred labels. Three hundred and ninety-six background rows carry balanced adverse and benign outcomes without Sentinel coordinates, while four joint rows carry all four coordinates, giving wildcard pair products one per cent co-occurrence and therefore twenty non-zero rows by the early checkpoint and eighty by the mature checkpoint.

The four joint pattern families are adverse concordant-high, adverse concordant-low, benign discordant-high-low, and benign discordant-low-high for the target pair. The other two Sentinels receive the complementary ranks, leaving every joint request with the same four-value multiset; deterministic rotations exchange concordant and discordant orientations without changing the target identities and choose ranks within the declared quiet and loud strata.

The tape constructor refuses to return unless each checkpoint prefix is class-balanced, every Sentinel has the same rank multiset in both outcome classes, every joint request has the same aggregate multiset, the joint-row count is exactly one per cent, and both worlds consume the same row values in the same order. Those measured preconditions belong to setup, not to the result assertions (´dec:harness:guarded-fixtures´), (´dec:harness:separate-validation´).

A witness-local correlation row implements `PlaybackRow`: its `PlaybackRow::play` method starts from `World::request`, resolves each named source through `World::sentinel`, appends its coordinate with `RequestContext::with_sentinel`, calls `World::derive_for_request`, builds `LabelSpec::adverse` or `LabelSpec::benign` with the default `Action::Allow`, and submits it through `World::label`. The same immutable tape is replayed once into each world through `playback`, with `PlaybackBarrierPolicy::new` selecting `PlaybackBarrier::FlushObservations` followed by `PlaybackBarrier::FlushLabels`, a non-zero `PlaybackBatchSize`, and one `PlaybackProgress` per run; each row therefore completes both queues before the next row and no batch is treated as a shared published view (´dec:harness:declarative-playback´), (´cor:ordering:tape-batches´).

`PlaybackCheckpoint::after_row` places callbacks after rows 1,999 and 7,999, where the selected barriers have completed exactly two thousand and eight thousand labels respectively. Each callback confirms the processed count through the owned `World::construction_baseline` reading, assesses an independent balanced holdout without submitting its labels, collects `RiskAssessment::risk.p_bad` for the joint arm and four Sentinel-alone arms, and computes every AUC with `pairwise_rank` using the specification's twenty-per-class gate and an exact-zero tie tolerance (´alg:monitoring:auc´), (´dec:harness:oracle-tier´).

The observation records, for every checkpoint and world, the joint AUC, four Sentinel-alone AUCs, classwise mean risk, minimum and maximum risk, processed label count, non-zero joint-row count, replay seed, world role, and playback progress. The holdout uses distinct entity names at each checkpoint and remains below the declared pending-buffer capacity, because its assessments must not become training labels.

The assertions require the treatment joint AUC below 0.55 at two thousand labels and at least 0.65 at eight thousand, every Sentinel-alone AUC below 0.55 in both worlds at both checkpoints, and the control joint AUC below 0.55 throughout; `assert_health_clean` closes both worlds. No fitted tolerance weakens a band edge: the edges are the intent's reading of the published interpretation table, and exact ties are handled inside the oracle.

A deliberately broken implementation that drops wildcard triples, multiplies the wrong extraction positions, or leaves interaction weights at the prior follows the control trajectory and fails the mature treatment assertion. An implementation that leaks the outcome through a slot, aggregate, ordering pattern, or fixture imbalance fails a Sentinel-alone, early, template-free, or setup-guard assertion.

## What is missing · `sec:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-missing`

**Entry (A constructible semantic wildcard declaration)** · `entry:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-constructible-wildcard`

The testing surface can now construct the wildcard, but the production interaction module still represents its operand as a legacy extraction offset rather than the semantic selector required by the vector record. The missing production shape is a wildcard declaration whose operand is `FeatureSelector::SlotFeature`, resolved during map compilation; the integration witness can isolate today's `InteractionTemplate::type3(0)` translation in one local constructor, but must not describe the positional form as the finished host contract (´dec:vector:semantic-templates´).

**Entry (The balanced sparse-correlation tape)** · `entry:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-balanced-tape`

The `posterior_convergence` target still needs its subject-owned tape constructor, equal-shape report surfaces, independent holdout generator, and measured guards for class balance, per-Sentinel marginal equality, fixed aggregate multisets, prefix counts, and one per cent co-occurrence. This is witness-local fixture geometry, not a missing shared harness fixture.

**Entry (Checkpointed bulk label driving)** · `entry:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-bulk-driver`

The shared driver has landed; the target still needs its correlation-row `PlaybackRow` implementation, two `PlaybackCheckpoint` callbacks, explicit observation-then-label `PlaybackBarrierPolicy`, and paired invocation of `playback`. No new sleep, poll, deadline, queue barrier, progress type, or bulk runner belongs in the target (´entry:assayer:harness-tape-runner´), (´dec:harness:no-ad-hoc-waits´).

**Entry (A reusable tie-corrected discrimination assertion)** · `entry:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-auc-assertion`

The reusable computation has landed as `pairwise_rank`; the target still needs a witness-local checkpoint observation that forms its labelled score populations, requires the twenty-per-class gate, reports the full reading, and applies the intent's 0.55 and 0.65 band assertions. It must not duplicate rank arithmetic or turn `Tolerances::auc` into slack around a behavioural threshold (´entry:assayer:harness-oracle-tier´).

## Risks and open questions · `sec:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-risks`

**Observation (The detection bands are guidance rather than a binding threshold)** · `obs:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-detection-band`

The specification explicitly makes the AUC interpretation bands non-binding. Using their near-chance and moderate edges gives “evades” and “detected” existing meanings and matches the nearest convergence witness; the alternative is a new intent-level probability-lift threshold, which has no present authority and requires a specification decision before it can be an oracle.

**Observation (The label horizons depend on the declared co-occurrence profile)** · `obs:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-cooccurrence-profile`

One per cent is the upper edge of the wildcard range in the convergence cascade and yields twenty informative joint rows by the early checkpoint and eighty by the mature checkpoint. A different rate changes the effective evidence count and therefore changes the promise being tested; the tape reports both total and joint counts.

**Observation (The public interaction declaration is not yet a coherent host surface)** · `obs:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-template-surface`

The gated test surface now re-exports `InteractionTemplate`, and `WorldBuilder::interaction_templates` forwards it, but `InteractionTemplate::type3` still names a position while the governing record requires a semantic operand. The witness can be written today only by confining extraction offset zero to one constructor justified by the extraction index; the durable repair is the semantic wildcard declaration described above, not wider exposure of positional layout.

**Observation (Ordering and assessment-side state can counterfeit a milestone)** · `obs:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-ordering`

Online updates are order-sensitive, standardisation observations use a queue that `World::flush_labels` explicitly does not cover, concordance thresholds evolve on assessment traffic, and unlabelled holdout probes remain pending. A common tape, matched report ingestion, a settled cold ramp, the ordered two-barrier playback policy, independent checkpoint entity names, and a holdout bounded below pending capacity hold those effects equal; removing either named barrier would make the checkpoint a scheduler reading rather than label-indexed evidence (´cor:concurrency:harness-barriers´), (´dec:harness:no-ad-hoc-waits´).

**Observation (AUC can change abruptly on a small discrete holdout)** · `obs:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-numerical-fragility`

The report ladder and holdout use several ranks per stratum rather than four repeated points, exact tie correction is mandatory, and the mature treatment reading is reported beside the control. If the fixed-seed result sits on either band boundary, the fixture lacks a stable witness even when one run passes; the remedy is better authored separation with the same declared population, not fitted tolerance or seed shopping.

## Acceptance · `sec:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge-acceptance`

The implementation lands in the `posterior_convergence` integration target as `posterior_convergence::cross_sentinel_correlation_is_caught_only_once_interactions_converge`; its test documentation cites the intent claim and mints its own test label at the function, and its index states the two-checkpoint treatment, Sentinel-alone, and template-free result.

The implementation report identifies the wildcard selector and extraction feature, treatment and control construction, training and holdout seeds, exact total and joint-label counts at each checkpoint, tape guards, playback batch size, ordered barrier policy, checkpoint placement, and pending-capacity bound.

The report prints the joint and four Sentinel-alone AUCs for both worlds at both checkpoints, their distances from the applicable bands, the treatment-to-control mature difference, classwise risk summaries, processed counts, and finite-health evidence.

The report demonstrates that the treatment is near chance at two thousand labels, reaches at least moderate discrimination by eight thousand, stays near chance when each Sentinel is assessed alone, and has no corresponding late lift without interactions.

The report includes focused target output, package formatting and lint verdicts, the package test result, a deterministic repeated run of the fixed seed, wall time for the long witness, and every remaining deviation from this plan.
