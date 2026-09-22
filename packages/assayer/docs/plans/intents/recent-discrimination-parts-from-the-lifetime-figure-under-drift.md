# Recent discrimination parts from the lifetime figure under drift · `plan:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift`

Keeping (´claim:wellness:recent-discrimination-parts-from-the-lifetime-figure-under-drift´) establishes that the public health surface preserves a visible, directional separation between recent and aggregate discrimination when a changed population loses the ranking quality learned from its history.

## What the promise says, precisely · `sec:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-promise`

The aggregate quantity is the rank-discrimination AUC over every entry currently retained in the calibration buffer, while the recent quantity is the same statistic over the newest configured count of entries; both quantities and their minimum positive and negative class gates are defined by (´alg:monitoring:auc´).

The reference configuration retains 2,000 calibration rows and refits every 200 labels (´tab:config:calibration´), takes the newest 500 labels as the recent population, and requires at least 20 positive and 20 negative rows in each reported population (´tab:config:monitoring´).

The measurable assertion is directional: after a stable, strongly ranked prefix and a rank-neutral recent suffix whose adverse base rate rises, `auc_aggregate - auc_recent > 0.1`; the detailed report also sets `discrimination_instability` because the specified absolute separation is strictly greater than 0.1 (´alg:monitoring:auc´).

The word lifetime denotes the whole bounded calibration buffer rather than all process history: the circular buffer stores the assessment-time effective estimate, uncertainty and blend weight beside the label-time outcome and importance weight, preserves label order, and evicts its oldest row at capacity (´constr:platt:buffer´).

The witness keeps both class counts above their AUC gates and the calibration fit above its separate evidence floor (´req:platt:minimum-samples´), so absence of either AUC cannot masquerade as stability.

The readings remain diagnostics only: the report exposes the drift without changing scoring or policy (´inv:monitoring:report-only´), and the package guarantee requires rank discrimination to remain visible through the health surface (´inv:guarantee:discrimination-visibility´).

## What the code offers today · `sec:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-code`

The public configuration exposes `MonitoringConfig::min_auc_class_count`, `MonitoringConfig::recent_discrimination_window`, `PlattConfig::n_cal_buf`, and `PlattConfig::n_refit`; eager builder validation rejects invalid overrides before a scenario starts (´dec:construction:eager-validation´).

The published health state carries `DiscriminationMetrics::auc_aggregate`, `DiscriminationMetrics::auc_recent`, and `DiscriminationMetrics::discrimination_instability`. `compute_discrimination_metrics_with_monitoring` computes all three at calibration refit, but it currently assigns ranks over the whole buffer before filtering the recent and regime populations, so the aggregate is ranked correctly and each filtered AUC can be wrong.

`Assayer::health_summary` returns the cached aggregate and recent AUCs with `HealthSummary::platt_refits_completed`, while `Assayer::full_health_report` returns the complete `DiscriminationMetrics`; the cheap and detailed tiers are the intended split (´dec:health:tiered-queries´), and both read the discrimination record cached at the last refit (´dec:health:cached-discrimination´).

The unified harness supplies `World`, the guarded `World::trained_state` constructor with its measured `TrainedStateFixture` and `TrainedStateBaseline`, `World::derive_for_request`, `World::label`, `World::flush_labels`, and `World::assayer` (´tab:assayer:harness-implementation-library-roster´). The trained-state guard establishes setup and reports its measured baseline without serving as the result oracle (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´) (´test:crate:trained-state-fixture-returns-its-measured-precondition´).

Long streams use subject-owned implementations of `PlaybackRow` through `playback`; `PlaybackBarrierPolicy`, `PlaybackBarrier::FlushLabels`, `PlaybackCheckpoint`, `PlaybackProgress`, and `PlaybackBatchSize` own settlement, exact checkpoints, progress, and bounded materialisation (´dec:harness:declarative-playback´) (´tab:assayer:harness-implementation-library-roster´) (´test:crate:shared-runner-accepts-unrelated-subject-row-types´). A label barrier is therefore selected declaratively rather than wrapped in a test-authored wait, and its completion precedes the row checkpoint (´dec:harness:no-ad-hoc-waits´) (´test:crate:selected-label-barrier-completes-before-checkpoint´).

The oracle tier supplies `pairwise_rank`, a class-gated, tie-aware positive-negative pair enumeration independent of the production sort and rank-sum route (´test:crate:pairwise-rank-oracle-counts-ties-by-pairs´); comparisons with the published AUCs use `Tolerances::auc` (´dec:harness:oracle-tier´) (´tab:assayer:harness-implementation-library-roster´) (´tab:assayer:harness-scenario-tolerances´).

The signal and label fixtures retain `with_score_verified`, `LabelSpec::adverse`, and `LabelSpec::benign`. The owned `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView` probes expose model, slot, and pending-entry state under the probe contract, but none is needed because this witness reads owned public health values (´dec:harness:probe-contract´). The witness has no elapsed-time predicate and no invariant seed family, so it needs neither `World::advance`, `World::travel_to`, nor `run_seeded_sweep`; the latter remains the required runner when a witness is a seed sweep (´dec:harness:seeded-sweeps´).

The nearest public witness proves that the score-verified request path learns a directional population split and independently ranks held-out populations (´test:integration:scalar-signal-population-split-converges-directionally´).

The nearest metric witness feeds calibration rows directly and raises the instability flag when a recent half differs from the aggregate (´test:crate:instability-flag-rises-on-recent-departure-from-aggregate´); it bypasses the public assess-label-refit-publication path, does not cite this promise, and does not compare either published value with `pairwise_rank`, so the promise remains unkept.

## The witness · `sec:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-witness`

The integration target `calibration_discrimination_convergence` receives the test `calibration_discrimination_convergence::recent_discrimination_parts_from_lifetime_figure_under_drift`, which drives the real engine through the public API and the documentation-hidden testing harness under the convergence split (´dec:harness:convergence-split´).

Setup calls `World::trained_state` with one fixed seed and takes the returned fixture's `World` only after its `TrainedStateBaseline` carries the declared class counts, endpoint gap, and held-out pairwise rank (´tab:assayer:harness-trained-state-fixture-figures´). The world therefore starts from measured score-verified separation rather than from an assumed cold-ramp outcome (´dec:harness:guarded-fixtures´), while the final health comparison remains a separate oracle judgment (´dec:harness:separate-validation´).

The first `playback` drives 1,600 stable closed-loop rows through `with_score_verified`, with a five-percent adverse rate, adverse outcomes on the high score rung, benign outcomes on the low score rung, `PlaybackBarrierPolicy::new` selecting `PlaybackBarrier::FlushLabels`, and `PlaybackBatchSize::default` bounding materialisation. A final-row `PlaybackCheckpoint` records the prefix refit stamp and requires the prefix-only `pairwise_rank` to reach the specification's good band of 0.80 after both classes clear `MonitoringConfig::min_auc_class_count` (´tab:monitoring:interpretation´).

The suffix contains 500 assessments in five equal blocks whose adverse rates are 10, 20, 30, 40, and 50 percent. Within each block, half the rows use each score rung and each block's adverse rows are divided equally between the rungs with their positions rotated, so the two outcomes have identical score distributions over the full suffix and its independently computed AUC is exactly one half.

The suffix assessments run through a second `playback` under `PlaybackBarrierPolicy::none` before any suffix label is revealed. Each subject-owned row retains the public assessment identifier, `RiskBasis::rho_eff`, score rung, outcome, and submission order needed to build the deferred labels and the independent populations.

The deferred suffix labels then run in captured order through a third `playback` with `PlaybackBarrier::FlushLabels`. `PlaybackProgress` identifies an incomplete row without measuring elapsed time, and a `PlaybackCheckpoint` after the final row reads the cheap summary and detailed report only after that row's label and publication barrier have completed (´dec:harness:declarative-playback´).

The guarded fixture contributes 500 labels (´tab:assayer:harness-trained-state-fixture-figures´) and leaves 100 labels toward the next default refit (´tab:config:calibration´). The 1,600 stable rows and 500 deferred suffix labels therefore put the last suffix label on a refit boundary; at that boundary the 2,000-row buffer has evicted the fixture population and the first 100 stable rows, retaining exactly 1,500 stable rows followed by the 500-row suffix.

The independent calculation selects the same newest `PlattConfig::n_cal_buf` captured rows and the same newest `MonitoringConfig::recent_discrimination_window` rows, verifies the positive and negative counts against `MonitoringConfig::min_auc_class_count`, and calls `pairwise_rank` separately on the aggregate and recent slices with exact tie recognition. It never calls the production discrimination calculation.

The fixture guards require the recent oracle to equal 0.5 exactly and the oracle difference to exceed `0.1 + 2 * Tolerances::auc`; the second term is the maximum loss of separation when each of two published AUCs differs from its oracle by the registered agreement tolerance, so the guard is derived rather than fitted (´tab:assayer:harness-scenario-tolerances´).

The product assertions require both public AUCs to lie in the unit interval, each to agree with its corresponding oracle within `Tolerances::auc`, `auc_aggregate - auc_recent > 0.1`, `DiscriminationMetrics::discrimination_instability` to be true, `HealthSummary::platt_refits_completed` to advance beyond the prefix stamp, and the cheap summary to carry the detailed report's same cached AUC pair.

The fails-before is the current whole-buffer-rank implementation: its filtered recent population reuses aggregate ranks before subtracting the recent population's triangular term, so the range or independent-oracle assertion fails. Reranking every filtered population locally makes the same witness green without weakening the strict threshold, class gates, or surface-consistency assertions.

## What is missing · `sec:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-missing`

No shared harness capability is missing: the finished roster supplies the guarded fixture, declarative playback, label barrier, progress, checkpoints, independent rank oracle, and named tolerance this witness needs (´tab:assayer:harness-implementation-library-roster´). The remaining work is one target-local tape and witness plus the production subset-reranking correction; the historical tape umbrella owns no open work (´entry:assayer:harness-stage-tapes´).

**Entry (Ordered two-phase discrimination tape)** · `entry:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-ordered-tape`

The integration target still needs a private `DiscriminationPlaybackRow` implementing `PlaybackRow`, a private `DiscriminationSample` that owns the assessment identifier, effective score, score rung, outcome, and order, and one shared test-local capture because `PlaybackRow::play` receives a shared row reference. Assessment rows append the frozen cohort, closed-loop rows append a sample and submit its label, deferred-label rows use the captured identifier, and all three use the landed `playback` runner rather than adding a harness module.

**Entry (Independent subset-rank AUC oracle)** · `entry:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-auc-oracle`

No oracle implementation remains missing. The target must select the retained aggregate and recent `DiscriminationSample` slices, validate their class counts, project each to score-outcome pairs, and call the existing `pairwise_rank` separately; local sorting, rank sums, and unit-interval logic must not be reintroduced (´dec:harness:oracle-tier´).

**Entry (Moving-base-rate rank-collapse fixture)** · `entry:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-drift-fixture`

The integration target still needs the deterministic stable-row and five-block suffix generators, their score-balanced outcome rotation, the exact-recent-AUC guard, the derived separation guard, and diagnostics that print the measured class counts and oracle values. The starting separation comes from `World::trained_state`; this entry owns only the drift population laid over that guarded state.

**Entry (Recent-subset reranking correction)** · `entry:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-subset-reranking`

`health::published` still needs to form ranks within each recent or regime subset before applying the Mann–Whitney triangular term; the aggregate may retain its whole-buffer ranks. The correction must preserve tie handling, the configured per-class gate, the unit interval, and the absolute instability comparison specified by (´alg:monitoring:auc´).

**Entry (Public drift witness)** · `entry:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-public-witness`

The empty integration target still needs `calibration_discrimination_convergence::recent_discrimination_parts_from_lifetime_figure_under_drift`, with the guarded setup, three playback phases, final refit-freshness check, independent aggregate and recent oracles, directional separation, instability flag, and cheap-versus-detailed consistency described above.

## Risks and open questions · `sec:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-risks`

**Observation (Aggregate is bounded history)** · `obs:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-bounded-history`

The promise's lifetime wording can suggest process-lifetime accumulation, but the specification and code define the comparator over the bounded calibration buffer. Driving enough post-fixture rows to evict the guarded fixture and the first 100 stable rows leaves exactly the intended 1,500-row stable prefix and 500-row suffix in the final comparison.

**Observation (Prevalence drift does not imply AUC drift)** · `obs:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-prevalence-and-ranking`

A base-rate shift alone does not force rank discrimination to move when conditional score ordering is unchanged. The five-block schedule supplies the population shift, while the score-balanced outcome assignment independently removes score-outcome association in the suffix; a prevalence-only fixture would not witness this promise.

**Observation (Filtered populations require local ranks)** · `obs:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-local-ranks`

The current implementation assigns ranks across the whole buffer before filtering the recent and regime populations, while the Mann–Whitney formula subtracts the triangular term for the filtered population. A filtered value can therefore leave the unit interval; separate `pairwise_rank` calls and explicit range assertions expose the defect rather than accommodating it.

**Observation (Refit freshness is observable)** · `obs:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-refit-freshness`

The health record is cached at refit rather than recomputed by a query (´dec:health:cached-discrimination´). The fixture population, default cadence, prefix length, and suffix length deliberately align the final row with a refit, while an advancing `HealthSummary::platt_refits_completed` stamp proves that the final observation is fresh.

**Observation (Numerical slack belongs in the fixture guard)** · `obs:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-numerical-slack`

The contractual threshold remains the strict value 0.1. Requiring the oracle difference above `0.1 + 2 * Tolerances::auc` derives exactly the slack needed for two implementation-versus-oracle comparisons and does not weaken the product assertion.

**Observation (Order and concurrency remain controlled)** · `obs:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-ordering`

Assessments are synchronous, the trained fixture has already settled its cold ramp, and only deferred label completion is asynchronous. The assessment playback therefore selects no barrier, while both label-bearing playbacks select `PlaybackBarrier::FlushLabels`; frozen suffix assessment, captured-order label playback, exact checkpoints, and progress diagnostics remove scheduler timing without a sleep, poll, local deadline, or compound wait (´dec:harness:no-ad-hoc-waits´).

## Acceptance · `sec:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-acceptance`

Acceptance names the integration target `calibration_discrimination_convergence` and the function `calibration_discrimination_convergence::recent_discrimination_parts_from_lifetime_figure_under_drift`, and identifies the target-local playback rows plus the `health::published` subset-reranking correction.

The implementation report records the default buffer, recent window, class gate and refit cadence; the guarded fixture baseline; the submitted and retained prefix counts; the five suffix base rates; the fixed seed; and `Tolerances::auc`, with no expected value obtained from the production discrimination route.

The report shows the prefix guard, final retained aggregate and recent class counts, oracle AUCs, corresponding public AUCs, directional difference, instability flag, prefix and final refit stamps, and agreement between the cheap and detailed health tiers.

The report includes the current implementation's red result for filtered whole-buffer ranks, a green result after local reranking and the public witness land, and the package's required formatting, lint, and test gates with their commands and verdicts.

Acceptance requires deterministic repetition under the fixed seed, no wall-clock sleep or test-authored wait, no widened production visibility, no lint allowance, no assertion derived from the implementation under test, and a final public result satisfying `auc_aggregate - auc_recent > 0.1` with both values inside the unit interval.
