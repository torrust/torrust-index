# Keeping the Sparse-Sentinel Conditioning Witness · `plan:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix`

This plan keeps the promise that sparse Sentinel participation leaves a converged precision matrix badly conditioned by witnessing the declared reporting fraction and the true spectral condition number on the public learning path (´claim:linalg:sparsely-reporting-sentinels-ill-condition-the-precision-matrix´).

## What the promise says, precisely · `sec:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-promise`

The witness registers nine Sentinels and designates exactly three as sparse. Each sparse Sentinel contributes a coordinate to exactly three requests in every hundred-request schedule period, so one third of the population reports on exactly 3% of requests; the other six follow balanced, independently phased 50% schedules rather than constant occupancy.

Reporting means that a request carries the Sentinel coordinate needed to route through a valid cached report. An omitted coordinate leaves the registered Sentinel active but silent, and label-time reconstruction restores occupancy with zero extracted features for that slot (´def:extraction:slot´) (´alg:runtime:reconstruction´).

The fixture enables one wildcard interaction template over a nonconstant extracted feature. That template creates one coordinate per unordered Sentinel pair and makes each coordinate informative only when both members report (´def:feature:template-wildcard´); the three sparse schedules occupy disjoint phases, so the three sparse-to-sparse coordinates receive no joint observation while each sparse marginal remains exactly 3%.

With no signals, identity dimensions, outcome axes, or competitive indicators, the bias, fifteen aggregate positions, nine 61-position Sentinel slots, and thirty-six wildcard-pair coordinates give $p=1+15+9\cdot61+36=601$ (´tab:feature:dimension-formula´). The specification-formula oracle `dimension_width` supplies the independent expected width, and `World::runtime_layout` checks the published layout rather than trusting the arithmetic alone (´dec:harness:oracle-tier´) (´test:crate:dimension-width-oracle-sums-the-reference-blocks´).

Both the sparse world and its control absorb balanced ground-truth labels through the first whole hundred-request period beyond $10p$, which is 6,100 labels at the declared width. That boundary is beyond the interaction-maturity interval of approximately $2p$ through $10p$ (´tab:warmup:stages´) and beyond the posterior's order-$2p$ data-shaping budget (´bound:resource:convergence-budget´); it is a measured fixture precondition, not the health composite or a runtime gate (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

The measured quantity is the operational model's true condition number from its most recent Cholesky rebuild, $\kappa(B)=\lambda_{\max}(B)/\lambda_{\min}(B)$, with a finite positive least eigenvalue. It is not the current diagonal ratio $\rho(B)$, which only lower-bounds the true condition number (´def:monitoring:condition-number´) (´req:monitoring:conditioning-bound-named´). The keeping assertion is strict: a rebuild completed after the convergence boundary reports $\kappa(B)>10^4$.

A matched control uses the same seed, label sequence, report generator, wildcard template, feature width, and number of schedule periods, but gives the three treatment Sentinels balanced 50% schedules. It is a non-vacuity control for the sparse schedule rather than an additional promise: the specification expects order-$10$ conditioning for well-populated features, but the public true-condition field may remain absent when a healthy model has measured and skipped every post-construction rebuild (´alg:gaussian:synchronisation-monitor´) (´dec:posterior:recomputation-trigger´).

The coordinatewise replenishment floor cannot substitute for the witness because it bounds diagonal entries without bounding the spectrum (´req:gaussian:prior-replenishment-floor´). The expected separation is the specified mechanism: forgetting erodes precision in directions that observations do not exercise, and rare cross-Sentinel directions can drive the condition number into the $10^5$ range and beyond (´alg:gaussian:synchronisation-monitor´).

## What the code offers today · `sec:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-current-code`

The finished scenario surface is `Scenario` over one `World`. `scenario_with` configures a seeded `WorldBuilder`; `WorldBuilder::interaction_templates` now forwards `InteractionTemplate` declarations to the real engine, `World::register_sentinels` installs the named population, `World::receive_report` ingests valid reports, `World::request` and `RequestContext::with_sentinel` form routed requests, and `World::label` plus `World::flush_labels` complete the public learning path (´dec:harness:single-scenario´) (´tab:assayer:harness-implementation-library-roster´). The builder seam that this plan previously called missing is established directly by the invalid-template and reference-layout witnesses (´test:crate:world-builder-forwards-invalid-interaction-template´) (´test:crate:public-scenario-reproduces-reference-layout-p638´).

The report helpers `make_cell_report` and `default_health` can produce valid varying inputs, while `LabelSpec` expresses alternating benign and adverse ground truth. `World::settle_cold_ramp_with` and `World::flush_observations` settle standardisation on declared requests without a sleep or scheduler poll; the scenario clock remains fixed because this witness isolates label forgetting from elapsed-time decay (´dec:posterior:combined-factor´).

Long stimuli now have the shared `PlaybackRow`, `PlaybackBarrierPolicy`, `PlaybackCheckpoint`, `PlaybackProgress`, and `playback` surface. A subject-owned row can carry one hundred-request schedule period, and a policy ordered as `PlaybackBarrier::FlushObservations` followed by `PlaybackBarrier::FlushLabels` places every period checkpoint after both queues have completed (´dec:harness:declarative-playback´) (´dec:harness:no-ad-hoc-waits´) (´test:unit:held-barrier-completes-before-its-checkpoint´).

The shared oracle tier now supplies `DimensionBlocks` and `dimension_width`, and the construction guard supplies `RuntimeLayout`; together they compare the independently calculated width with one published layout. `TestRng` and the explicit scenario seed make replay deterministic, so this single fixed-schedule witness needs no seed sweep; any later multi-seed invariant pack must use `run_seeded_sweep` (´dec:harness:oracle-tier´) (´dec:harness:seeded-sweeps´).

`Assayer::full_health_report` exposes `PrecisionHealthDetail::kappa`, `PrecisionHealthDetail::lambda_min`, `PrecisionHealthDetail::diagonal_ratio`, `PrecisionHealthDetail::floor_share`, `PrecisionHealthDetail::cholesky_recomputes`, and `PrecisionHealthDetail::cascade_terminus_count` for `ModelId::Operational`. The condition number and least eigenvalue describe the last rebuild, while the diagonal ratio is refreshed on every label (´dec:health:tiered-queries´) (´dec:posterior:recomputation-trigger´).

`PublishedModelBlock` is not a route to the working precision matrix: it owns the published mean, covariance, dimension, and floor masses, and deliberately excludes precision. The probe contract preserves that boundary, so this witness must use the public condition reading and prove that its source rebuild occurred after convergence rather than add a current-precision probe (´dec:harness:probe-contract´) (´obs:assayer:testing-harness-record-audit-probe-universal-conflict´).

No current test cites or mints this promise. `a_dense_competitive_schedule_leaves_every_precision_matrix_factorisable` covers factorisability and cascade health at large width but not sparse occupancy (´test:integration:a-dense-competitive-schedule-leaves-every-precision-matrix-factorisable´), while `the_floor_moves_the_condition_estimate_and_the_recompute_count_but_not_the_error` varies the replenishment floor and reads the diagonal-ratio lower bound rather than this true-spectrum threshold (´test:integration:the-floor-moves-the-condition-estimate-and-the-recompute-count-but-not-the-error´). The `posterior_convergence` target exists as empty scaffolding under the finished convergence split (´dec:harness:convergence-split´).

The testing plan's broad convergence gap still covers the unwritten behavioural depth, but its tapes-and-fixtures stage is historical rather than a live harness dependency (´entry:assayer:gap-convergence-depth´) (´entry:assayer:harness-stage-tapes´). What remains is the subject fixture, tape, freshness checkpoint, and claim-bearing witness described below, not general harness construction.

## The witness · `sec:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-witness`

The integration target `posterior_convergence` is the destination, and the test function is `posterior_convergence::sparsely_reporting_sentinels_ill_condition_the_precision_matrix`. Its module index and documentation cite the intent and the claim, keeping this true-spectrum result distinct from the diagonal-ratio experiments (´dec:harness:convergence-split´).

- Setup: construct sparse and control `Scenario` values with `scenario_with`, the same explicit seed, fixed virtual clock, default model configuration, one channel, no optional feature blocks, and one `InteractionTemplate::type3` over a varying extraction position. Declare the expected `RuntimeLayout` from `dimension_width`, register the same nine Sentinel names, and require each published layout to equal that declaration.

- Tape validation: materialise one deterministic hundred-request period before either world is trained and assert that exactly three names have three presences each, their phases are disjoint, each of the other six names has fifty presences, the control raises only the three sparse schedules to fifty presences, and both worlds retain thirty-six pair coordinates. These exact counters guard the fixture and remain separate from the condition-number result (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

- Standardisation setup: prime every Sentinel with a valid cached report, build the treatment and control period requests from their respective masks, and pass each complete period to `World::settle_cold_ramp_with`. The helper's observation barrier establishes in-service standardisation while preserving each period's declared marginal frequencies; no label or virtual-time movement occurs.

- Period playback: implement one subject-owned `PlaybackRow` per hundred-request period. Each row deterministically varies the selected extracted feature, ingests a fresh valid report for every scheduled presence, attaches only those Sentinel coordinates to the request, assesses, and submits an alternating benign or adverse ground-truth `LabelSpec`. Run the same number of treatment and control rows through `playback` with `PlaybackBarrierPolicy` ordered as `PlaybackBarrier::FlushObservations` then `PlaybackBarrier::FlushLabels`; bounded row materialisation changes allocation cost only, not the per-period observation boundary (´dec:harness:declarative-playback´).

- Convergence checkpoint: after the first whole period beyond $10p$, use `PlaybackCheckpoint` to read each full health report once, require 6,100 accepted ground-truth labels and the declared runtime width, and record the operational `cholesky_recomputes` count. The health composite remains diagnostic because convergence reports and never gates behaviour (´dec:health:reports-never-gates´).

- Fresh-spectrum tail: play exactly the configured 1,000-label recomputation interval as ten further complete periods (´tab:config:risk-model´). At the terminal checkpoint, require the sparse operational `cholesky_recomputes` count to have advanced beyond its convergence-checkpoint value before reading `kappa` and `lambda_min`; that comparison proves that the reported spectrum was rebuilt after the convergence boundary without inspecting excluded precision state.

- Assertion: require a present, finite sparse $\kappa$, a positive finite sparse $\lambda_{\min}$, and $\kappa(B)>10^4$; derive the sparse $\lambda_{\max}=\kappa\lambda_{\min}$ only for diagnostics. Require equal widths and label counts, a finite control `diagonal_ratio`, and zero operational `cascade_terminus_count` in both worlds; record any control true-condition reading that exists without turning its permitted absence into a failure.

- Reproducibility: failure output reports the seed, period, marginal and pairwise presence counts, runtime width, labels absorbed, convergence and terminal rebuild counts, the sparse spectrum, both diagonal ratios, both optional reported condition readings, floor shares, and cascade termini. `PlaybackProgress` names the first incomplete period if execution stalls; no sleep, poll, wall-clock assertion, fitted tolerance, or probabilistic confidence interval enters the verdict (´cav:harness:progress-not-latency´).

The fails-before is an implementation that fills an absent slot from the last cached extraction, treats every active Sentinel as reporting, or reconstructs wildcard products from cached rather than request-present operands. Any of those defects gives the sparse pair coordinates dense updates, keeps the sparse reading on the control side of $10^4$, and fails the terminal assertion while the existing dense and diagonal-ratio witnesses can remain green.

## What is missing · `sec:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-missing`

**Entry (The sparse fixture declares and guards its interaction layout)** · `entry:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-world-interactions`

Under `testing::fixtures`, add the subject fixture that configures both scenarios through the landed `WorldBuilder::interaction_templates` seam, registers the named population, settles each cold ramp on its complete schedule period, and returns only after the schedule counters and `RuntimeLayout` agree with the `dimension_width` baseline. The guard reports the expected and measured masks and widths; it does not inspect the terminal condition number.

**Entry (The terminal checkpoint proves spectrum freshness)** · `entry:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-current-spectrum-probe`

In the `posterior_convergence` target, add the paired convergence and terminal `PlaybackCheckpoint` callbacks that retain the operational rebuild counts, read one full health report per world at each boundary, and reject a sparse terminal spectrum whose `cholesky_recomputes` count did not advance after convergence. This uses the existing public health reading; it adds neither a `PublishedModelBlock` precision field nor another owner command.

**Entry (The sparse measurement tape)** · `entry:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-surface-tape`

In the `posterior_convergence` target, define the hundred-request subject row, exact sparse and control masks, varying valid-report generator, routing coordinates, alternating ground-truth labels, and schedule counters, then implement `PlaybackRow` and the two-queue `PlaybackBarrierPolicy`. The row owns the subject-specific stimulus while the shared runner owns settling, checkpoints, bounded materialisation, and progress.

**Entry (The converged conditioning witness)** · `entry:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-integration-witness`

In the `posterior_convergence` target, add `posterior_convergence::sparsely_reporting_sentinels_ill_condition_the_precision_matrix`, drive both guarded worlds through the convergence boundary and fixed freshness tail, apply the treatment and control assertions, emit the complete replay context, and add the claim-bearing row to the module index. No production contract or sibling intent is a prerequisite.

## Risks and open questions · `sec:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-risks`

**Observation (Sparse reporting is request occupancy)** · `obs:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-reporting-semantics`

The public health field `sentinel_coverage` measures whether registered Sentinels have cached reports, not how often requests carry their coordinates. It reaches full coverage after every Sentinel has reported once and cannot serve as the 3% oracle; the validated schedule masks and counters express the promise's measurable meaning.

**Observation (The interaction set is an explicit fixture choice)** · `obs:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-interaction-scope`

The recommended interaction set contains no wildcard template (´tab:feature:default-interaction-set´), and `WorldBuilder` defaults to an empty template list. The explicit wildcard exercises the specification's direct pairwise mechanism and makes the promise a configured-surface statement; interpreting the promise as default-only would remove the rare pair coordinates and require a different claim rather than a silent fixture substitution.

**Observation (The operational matrix is the primary scope)** · `obs:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-model-scope`

The promise names one $B$ while the implementation carries operational, sister, anchor, and optional outcome-axis models. The operational matrix is the narrow code-grounded interpretation because it absorbs every label and spans the complete configured feature space (´alg:runtime:update-path´); sister conditioning remains diagnostic, while the fixed anchor projection and absent outcome axes do not answer this surface claim. A requirement over every full-width model needs an explicit broader statement.

**Observation (The last rebuild must follow convergence)** · `obs:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-spectrum-freshness`

`PrecisionHealthDetail::kappa` describes the last rebuild, not necessarily the matrix after the final label, and `World::flush_labels` orders learning without forcing another decomposition (´dec:posterior:recomputation-trigger´). The pre-tail and terminal `cholesky_recomputes` readings therefore form a necessary freshness guard. If the fixed configured-cadence tail produces no sparse rebuild, the witness has no admissible true-spectrum observation and must report that fixture failure rather than poll, force a rebuild, or widen the probe boundary (´dec:harness:probe-contract´).

**Observation (The control rejects fixture-made singularity)** · `obs:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-control-conditioning`

Fixed reports or constant occupancy across all Sentinels can create collinear columns independently of sparse participation. Varying reports, nonconstant well-populated schedules, exact schedule validation, the control's current diagonal-ratio diagnostic, and any true-condition reading its ordinary rebuilds produce expose those fixture pathologies without inventing a requirement that a healthy control rebuild merely to populate an optional field.

**Observation (Runtime is bounded but intentionally substantial)** · `obs:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-runtime`

Two worlds past $10p$ labels at width 601 perform substantial quadratic work and periodic decompositions. Playback bounds materialisation, its progress reading localises a stall, and the test carries no timing assertion; runtime measurement may inform later suite placement but cannot weaken the ordinary claim-bearing test or turn liveness into latency (´cav:harness:progress-not-latency´).

The remaining semantic questions are whether the intent is limited to a host-enabled wildcard interaction surface and whether $B$ means the operational model alone. The witness records the narrow code-grounded choices above; changing either choice changes the proposition being kept.

## Acceptance · `sec:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix-acceptance`

The integration target `posterior_convergence` contains `posterior_convergence::sparsely_reporting_sentinels_ill_condition_the_precision_matrix`; its module index and test documentation cite the intent and claim, and the coverage census resolves the promise to that witness.

The report prints the configured population, exact marginal and pairwise schedule counts, wildcard template, runtime width, convergence boundary, labels absorbed, seed, convergence and terminal rebuild counts, the sparse world's $\lambda_{\min}$, derived $\lambda_{\max}$ and $\kappa$, and both worlds' diagonal ratio, optional condition reading, floor share, and cascade-terminus count.

The reported assertions show exactly three of nine Sentinels at 3%, at least $10p$ operational labels, a sparse rebuild after that boundary, a positive finite sparse spectrum reading with $\kappa(B)>10^4$, equal dimensions and label counts, a finite control diagonal ratio, and no cascade terminus.

The report shows the claim-bearing integration test and the package's prescribed formatting, lint, and test gates passing, with no ignored test and any unrelated failure separated explicitly.

The report states which sparse-slot or interaction update a deliberately broken implementation would perform and which terminal assertion rejects it. No executed mutation is part of acceptance.

Acceptance excludes the diagonal ratio as a substitute for $\kappa$, the composite health stage as a convergence oracle, a forced rebuild or precision probe as an observation mechanism, uncontrolled randomness, wall-clock sleeps or polls, and changes to a production health contract.
