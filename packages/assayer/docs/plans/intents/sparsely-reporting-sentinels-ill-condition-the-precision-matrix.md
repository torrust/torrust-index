# Keeping the Sparse-Sentinel Conditioning Witness · `plan:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix`

This plan keeps the promise that sparse Sentinel participation leaves a converged precision matrix badly conditioned by witnessing the declared reporting fraction and the true spectral condition number on the public learning path (´claim:linalg:sparsely-reporting-sentinels-ill-condition-the-precision-matrix´).

## What the promise says, precisely · `sec:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-promise`

The witness registers nine Sentinels and designates exactly three of them as sparse. Each sparse Sentinel contributes a coordinate to exactly three requests in every hundred-request schedule period, so one third of the population reports on exactly 3% of requests; the other six follow balanced, independently phased 50% schedules rather than constant occupancy.

Reporting means that a request carries the Sentinel coordinate needed to route through a valid cached report. An omitted coordinate leaves the registered Sentinel active but silent, and label-time reconstruction restores occupancy with zero extracted features for that slot (´def:extraction:slot´), (´alg:runtime:reconstruction´).

The fixture enables one wildcard interaction template over a nonconstant extracted feature. That template creates one coordinate per Sentinel pair and makes each coordinate informative only when both members report (´def:feature:template-wildcard´); the three sparse schedules occupy disjoint phases, so the three sparse-to-sparse coordinates receive no joint observation while the sparse marginal rate remains exactly 3%.

With no signals, identity dimensions, outcome axes, or competitive indicators, nine 61-position Sentinel slots and the wildcard's thirty-six pair coordinates give $p=1+15+9\cdot61+36=601$ under the dimension formula (´tab:feature:dimension-formula´). The spectrum probe confirms that runtime width rather than trusting the arithmetic silently.

Convergence is a fixture boundary, not the health composite: both the sparse world and its control absorb at least $10p$ balanced ground-truth labels, rounded up to a whole schedule period. This places the observation beyond the specification's interaction-maturity interval (´tab:warmup:stages´) and well beyond the posterior's order-$2p$ data-shaping budget (´bound:resource:convergence-budget´).

The measured quantity is the operational model's current $\kappa(B)=\lambda_{\max}(B)/\lambda_{\min}(B)$, with finite positive extremes, not the diagonal ratio $\rho(B)$ that only lower-bounds it (´def:monitoring:condition-number´), (´req:monitoring:conditioning-bound-named´). The keeping assertion is strict: after convergence, $\kappa(B)>10^4$.

A matched control uses the same reports, labels, template, feature width, and number of cycles but raises the three sparse schedules to the well-populated 50% regime. Its current operational condition number remains below $10^4$, making the sparse participation pattern rather than a degenerate report tape the distinguishing input.

The coordinatewise replenishment floor is not a substitute for this witness because it bounds diagonal entries without bounding the spectrum (´req:gaussian:prior-replenishment-floor´). The expected separation follows the specified mechanism: forgetting erodes precision in directions that observations do not exercise, and rare cross-Sentinel directions can drive the condition number into the $10^5$ range and beyond (´alg:gaussian:synchronisation-monitor´).

## What the code offers today · `sec:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-current-code`

The public integration path starts with `WorldBuilder`, registers the nine names through `World::register_sentinels`, ingests valid `BatchReport` values through `World::receive_report`, constructs each `RequestContext` by chaining `with_sentinel` for the scheduled coordinates, assesses it, submits a `LabelSpec` with ground truth, and waits at `World::flush_labels`. The operational model receives every accepted label while the sister and anchor updates are eligibility-gated (´alg:runtime:update-path´).

The completed harness skeleton already supplies seeded `World` construction, stable name allocation, request and label verbs, `TestRng`, virtual time, named tolerances, and publication barriers (´entry:assayer:harness-stage-skeleton´). Its exported `make_cell_report`, `default_health`, and report builders provide valid material for a deterministic varying report tape.

`World::assayer().full_health_report().precision[ModelId::Operational]` exposes `kappa`, `lambda_min`, `diagonal_ratio`, floor share, and recomputation counters. The full tier correctly publishes the true condition reading while the compact tier publishes only the cheaper bound (´dec:health:tiered-queries´).

That public `kappa` is the spectrum measured at the last rebuild, not necessarily the matrix held after the final label. A cadence visit may measure synchronisation and rebuild nothing (´dec:posterior:recomputation-trigger´), so `flush_labels` orders the work but does not make the published condition number current. A test-support spectrum query is therefore required for the terminal assertion.

`AssayerBuilder::interaction_templates` accepts the wildcard template, but `WorldBuilder` carries no corresponding input and currently builds every world with an empty interaction list. The fixture cannot reach the pairwise mechanism through the existing high-level harness without a small construction seam.

The convergence scenario nearest in shape proves directional learning after a fixed stream but never inspects precision (´test:integration:scalar-signal-population-split-converges-directionally´). The dense precision scenario proves factorisability and clean cascade health under a growing width but reads no sparse surface (´test:integration:a-dense-competitive-schedule-leaves-every-precision-matrix-factorisable´).

The nearest conditioning scenario varies the replenishment floor and stores `diagonal_ratio` under a local condition-shaped name (´test:integration:the-floor-moves-the-condition-estimate-and-the-recompute-count-but-not-the-error´). It is useful as a long-run liveness pattern, but its lower bound cannot keep this true-spectrum promise.

The standing testing plan has no gap entry covering this promise. Its open tape stage nevertheless names the streaming fixtures, convergence milestones, model-state inspection, and health accessors this witness needs (´entry:assayer:harness-stage-tapes´).

## The witness · `sec:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-witness`

The witness belongs in a new integration test, `tests/precision_conditioning.rs`, so its true-spectrum vocabulary cannot be confused with the cited diagonal-ratio experiments; its module index and test documentation cite the intent.

- Setup: build sparse and control worlds from the same default model configuration, fixed seed, frozen virtual clock, empty signal schema, one wildcard pair template, one channel, and the same nine Sentinel registrations.

- Tape validation: before driving either world, enumerate one full deterministic schedule period and assert three sparse names, exactly three presences per sparse name, disjoint sparse phases, balanced 50% presence for every well-populated name, and identical control dimensions with only the three marginal schedules changed.

- Report stimulus: on each scheduled presence, ingest the next valid report for that Sentinel and attach a routing coordinate that reaches it. The tape varies the selected extracted feature and the other slot values across occurrences so constant or duplicated report columns cannot manufacture the terminal spectrum.

- Label stimulus: alternate benign and adverse ground-truth `LabelSpec` values, use the same entity and report sequences in both worlds, and flush after bounded batches while a progress watcher distinguishes slow $O(p^2)$ work from a stalled owner thread.

- Convergence observation: read the model width from the test-support probe, require $p=601$, and continue until each operational model has absorbed at least $10p$ labels and completed a whole schedule period; the reported composite stage remains diagnostic because convergence is reported rather than enforced (´dec:health:reports-never-gates´).

- Spectrum observation: after the final publication barrier, request a read-only current spectrum from the model-owner thread for `ModelId::Operational` in each world and record $\lambda_{\min}$, $\lambda_{\max}$, $\kappa$, dimension, and absorbed-label count from one ordered snapshot.

- Assertion: require positive finite eigenvalue extremes, require the sparse reading to satisfy $\kappa(B)>10^4$, require the matched control to remain below $10^4$, and require equal widths and label counts. The full health report also remains free of a cascade terminus.

- Reproducibility: the failure output prints the seed, period, marginal and pairwise presence counts, $p$, labels absorbed, both spectra, diagonal ratios, floor shares, and recomputation counts; no wall-clock sleep or probabilistic confidence interval enters the verdict.

The fails-before is an implementation that fills an absent slot from the last cached extraction, treats every active Sentinel as reporting, or reconstructs wildcard products from cached rather than request-present operands. Any of those defects gives the sparse pair coordinates dense updates, keeps the sparse spectrum on the control side of $10^4$, and fails the terminal assertion while the existing dense and diagonal-ratio witnesses can remain green.

## What is missing · `sec:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-missing`

**Entry (World construction carries interaction templates)** · `entry:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-world-interactions`

Roughly twenty lines add an `interaction_templates` field and fluent setter to `WorldBuilder` and forward it to `AssayerBuilder`. A small builder check confirms the resulting runtime dimension includes the declared wildcard block. This entry depends on (´entry:assayer:harness-stage-skeleton´) and on no other intent lane.

**Entry (The owner answers a current-spectrum probe)** · `entry:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-current-spectrum-probe`

Roughly seventy lines add a `test-support`-only request and response on `ModelOwnerCommand`, measure the current operational precision through the existing symmetric eigensolver without changing it, and expose the result through `World`. The ordered response includes dimension and absorbed-label count so the condition reading and convergence boundary describe the same owner state. This entry depends on (´entry:assayer:harness-stage-tapes´), not on a production health-field change or another intent lane.

**Entry (The sparse measurement tape)** · `entry:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-surface-tape`

Roughly ninety lines in the integration module define the named population, exact periodic occupancy masks, paired control masks, varying valid reports, routing coordinates, alternating ground-truth labels, and schedule counters. It depends on (´entry:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-world-interactions´) and the existing report builders.

**Entry (The converged conditioning witness)** · `entry:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-integration-witness`

Roughly eighty lines drive both worlds to the $10p$ boundary, obtain the ordered spectra, apply the treatment and control assertions, emit diagnostic context, and maintain the new module index. It depends on (´entry:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-current-spectrum-probe´) and (´entry:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-surface-tape´), with no dependency on a sibling intent lane.

## Risks and open questions · `sec:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-risks`

**Observation (Sparse reporting is request occupancy)** · `obs:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-reporting-semantics`

The public health field `sentinel_coverage` measures whether registered Sentinels have cached reports, not how often requests carry their coordinates. It reaches full coverage after every Sentinel has reported once and cannot serve as the 3% oracle; explicit schedule counters and reconstructed occupancy express the promise's measurable meaning.

**Observation (The interaction set is an explicit fixture choice)** · `obs:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-interaction-scope`

The recommended interaction set contains no wildcard template (´tab:feature:default-interaction-set´), and the current builders default to no templates at all. The selected wildcard exercises the specification's direct pairwise mechanism and makes the promise a valid configured-surface statement; a default-configuration interpretation instead tests only sparse Sentinel slots and may not imply the stated threshold. That alternative requires a different claim or an explicit scope ruling rather than silent substitution in this witness.

**Observation (The operational matrix is the primary scope)** · `obs:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-model-scope`

The promise names one $B$ while the implementation carries operational, sister, anchor, and optional outcome-axis models. The operational matrix is the stable primary interpretation because it absorbs every label and spans the full configured feature space; sister conditioning is recorded diagnostically, while the fixed anchor projection and absent outcome axes do not answer this surface claim. A requirement over every full-width model needs an explicit broader statement.

**Observation (The last rebuild is not the terminal matrix)** · `obs:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-spectrum-freshness`

Using `PrecisionHealthDetail::kappa` after a barrier can yield a reproducible but stale verdict because healthy cadence visits carry the last rebuild spectrum forward. Forcing a rebuild would mutate the matrix through the spectral floor (´dec:posterior:spectral-floor´); the read-only test-support probe pays for a fresh decomposition without changing the production state under test.

**Observation (The control rejects fixture-made singularity)** · `obs:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-control-conditioning`

Fixed reports or constant one occupancy across all Sentinels create collinear columns independently of sparse participation. Varying reports, nonconstant well-populated schedules, and a matched control below the threshold make either pathology a fixture failure instead of evidence for the promise.

**Observation (Runtime is bounded but intentionally long)** · `obs:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-runtime`

Two worlds beyond $10p$ labels at a width above six hundred perform substantial quadratic work and two terminal eigendecompositions. The integration test is ignored by default, names its exact release-mode invocation in the module index, uses progress-based liveness deadlines, and contains no timing assertion.

The unresolved semantic questions are whether the intent is limited to a host-enabled pairwise interaction surface and whether $B$ means the operational model alone. The witness records the narrow code-grounded choices above; changing either choice changes the proposition being kept.

## Acceptance · `sec:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-acceptance`

The implementing lane's report identifies the new integration test, `tests/precision_conditioning.rs`, the `WorldBuilder` interaction seam, and the current-spectrum probe; it confirms that the module index, test documentation, and coverage matrix resolve the intent to the new integration witness.

The report prints the configured population, exact marginal and pairwise schedule counts, interaction template, runtime $p$, convergence boundary, labels absorbed, seed, and both worlds' $\lambda_{\min}$, $\lambda_{\max}$, $\kappa$, diagonal ratio, floor share, recomputation count, and cascade-terminus count.

The reported assertions show exactly three of nine Sentinels at 3%, at least $10p$ operational labels, positive finite spectra, sparse $\kappa(B)>10^4$, matched-control $\kappa(B)<10^4$, equal dimensions and label counts, and no cascade terminus.

The report shows the targeted ignored integration witness passing in release mode and the package's prescribed formatting, lint, feature, documentation, and test gates passing, with any unrelated failure separated explicitly.

The report states which sparse-slot or interaction update a deliberately broken implementation would perform and which terminal assertion rejects it. No executed mutation is part of acceptance.

Acceptance excludes the diagonal ratio as a substitute for $\kappa$, the composite health stage as a convergence oracle, a forced rebuild as an observation mechanism, uncontrolled randomness, wall-clock sleeps, and changes to a production health contract.
