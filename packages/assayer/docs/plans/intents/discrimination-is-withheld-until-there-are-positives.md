# Withholding discrimination until the configured AUC class gate · `plan:assayer:intent-discrimination-is-withheld-until-there-are-positives`

Keeping (´claim:wellness:discrimination-is-withheld-until-there-are-positives´) establishes through the public health surface that discrimination remains absent until the retained calibration rows contain the configured AUC class gate's required positive count, whose conforming floor is five; the separate three-positive calibration-fitting gate can permit a parameter refit but does not publish a discrimination figure.

## What the promise says, precisely · `sec:assayer:intent-discrimination-is-withheld-until-there-are-positives-promise`

Let $P_+$ be the raw count of retained calibration rows whose label valence is positive and let $P_-$ be the corresponding count whose valence is non-positive. Every public health observation taken after a periodic refit with $P_+ < N_\text{auc,min}$ must report aggregate discrimination as absent, even when $P_-$ and the total row count are ample; conforming configuration cannot set $N_\text{auc,min}$ below five.

Each calibration row stores whether valence was positive together with the effective estimate captured at assessment time and the eventual outcome recorded at label time (´constr:platt:buffer´).

Calibration fitting and discrimination publication count different evidence. A regime is fitted only when its weighted partition reaches the thirty-sample floor and the independently configured positive and negative floors, which default to three apiece (´req:platt:minimum-samples´), (´tab:config:calibration´); three raw positive rows therefore reach the nominal configured count but do not by themselves prove that either weighted regime partition fitted.

Rank discrimination has a separate gate. Aggregate, recent, sister-regime, and anchor-regime AUC are each reported only when their own population contains at least $N_\text{auc,min}$ positive and negative rows, default twenty (´alg:monitoring:auc´), and conforming configuration cannot lower that gate below five (´tab:config:monitoring´).

The measurable boundary is the minimum valid AUC configuration. Periodic refits observe cumulative positive counts zero, three, four, and five while negative evidence remains above every relevant floor. `HealthSummary::auc_aggregate` and `SystemHealthReport::discrimination` remain absent at zero, three, and four; at five, the cheap aggregate and the detailed object's aggregate are both present. The three-positive checkpoint distinguishes the raw count matching the calibration configuration from the higher publication gate without asserting that a weighted regime fit succeeded.

Health remains diagnostic: withholding this reading changes no assessment, label, or policy behaviour (´dec:health:reports-never-gates´).

## What the code offers today · `sec:assayer:intent-discrimination-is-withheld-until-there-are-positives-code`

The public configuration exposes `PlattConfig::n_refit`, `PlattConfig::n_cal_pos`, and `MonitoringConfig::min_auc_class_count` under the calibration and monitoring registers (´tab:config:calibration´), (´tab:config:monitoring´). Construction validates every numerical parameter eagerly (´dec:construction:eager-validation´), and the existing configuration witness rejects an AUC class gate of four (´test:crate:validation-reaches-every-monitoring-threshold´).

The label path appends `CalibrationEntry` values, applies the weighted per-regime sample floors, computes discrimination during each triggered calibration refit, and publishes a discrimination object only when aggregate AUC is present. The result is cached rather than recomputed by a query (´dec:calibration:per-regime-search´), (´dec:health:cached-discrimination´).

`Assayer::health_summary()` projects the cached aggregate into `HealthSummary::auc_aggregate`, while `Assayer::full_health_report()` projects the cached object into `SystemHealthReport::discrimination`; these are the intended cheap and detailed query tiers (´dec:health:tiered-queries´).

The finished harness provides configured seeded construction through `scenario_with_config`, request and label submission through `World::request`, `World::assess`, and `World::label`, deterministic adverse and benign values through `LabelSpec`, and queue-specific completion through `World::flush_labels` (´tab:assayer:harness-implementation-library-roster´), (´entry:assayer:harness-closed-barriers´). `World::flush_labels` covers owner commands and labels through publication and deliberately does not claim completion of the cold-observation queue.

Long streams now use `playback` with a subject-owned `PlaybackRow`, an explicit `PlaybackBarrierPolicy`, exact `PlaybackCheckpoint` callbacks, bounded `PlaybackBatchSize`, and `PlaybackProgress`; the runner owns those execution semantics (´dec:harness:declarative-playback´), (´entry:assayer:harness-tape-runner´). The specification-formula oracle `pairwise_rank` independently applies the configured class gate and is available for the option-presence expectation (´dec:harness:oracle-tier´), (´entry:assayer:harness-oracle-tier´).

The owned `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView` probes do not project public health. `TrainedStateFixture` establishes an already trained population, and `run_seeded_sweep` owns varying generated cases; neither is appropriate for a fixed cold-start boundary whose evidence accumulation is the subject of the witness (´dec:harness:probe-contract´), (´dec:harness:guarded-fixtures´), (´dec:harness:seeded-sweeps´), (´tab:assayer:harness-implementation-library-roster´).

The closest fitter test supplies fifty negative and two positive rows and proves that the under-floor regime is not fitted (´test:unit:minimum-sample-guard´). The closest discrimination test calls the internal metric computation and proves absence immediately below a configured five-per-class gate and presence at equality (´test:crate:discrimination-honours-configured-class-gate-and-recent-window´). The existing public surface test reads a compact assessment and a full report around a publication boundary (´test:integration:standardisation-transition-reported-on-both-surfaces´), but no test carries this intent's claim through assessment, label ingestion, refit, cache publication, and both health queries. The standing gap register has no entry for it (´sec:assayer:testing-plan-gaps´).

## The witness · `sec:assayer:intent-discrimination-is-withheld-until-there-are-positives-witness`

The integration target `assess_integration` gains the root test function `discrimination_is_withheld_until_positive_evidence_reaches_the_public_gate`, beside its configured-scenario and public-health witnesses (´test:integration:standardisation-transition-reported-on-both-surfaces´).

Setup uses `scenario_with_config` to declare one seeded scenario, one default channel, `PlattConfig::n_refit` of fifty labels, and `MonitoringConfig::min_auc_class_count` of five, while retaining every other calibration and monitoring default. The two-hundred-row stimulus remains below the default calibration-buffer capacity, so no retained row is evicted.

A local label-row type implements `PlaybackRow`. Its `play` method builds a uniquely named request with `World::request`, obtains the public assessment through `World::assess`, records `RiskBasis::rho_eff` beside the row's adverse flag for the oracle, and submits `LabelSpec::adverse` or `LabelSpec::benign` through `World::label`. The raw score is part of the public assessment schema (´schema:output:assessment´), while the row owns only subject data and playback owns settlement and checkpoint placement (´dec:harness:declarative-playback´).

The ordered rows form four fifty-label blocks containing respectively zero, three, one, and one adverse labels, so cumulative $P_+$ is zero, three, four, and five after zero-based rows 49, 99, 149, and 199. `playback` runs them with `PlaybackBarrierPolicy::new([PlaybackBarrier::FlushLabels])`, a bounded `PlaybackBatchSize`, one `PlaybackCheckpoint` at each boundary, and one `PlaybackProgress` reading. The policy crosses label publication before every checkpoint and excludes `World::flush_observations`, because this witness reads the label-owned discrimination cache rather than cold-ramp progress (´entry:assayer:harness-tape-runner´), (´entry:assayer:harness-closed-barriers´).

Each checkpoint first verifies its locally recorded positive and negative counts, the public total-label count, and a one-step advance of `HealthSummary::platt_refits_completed`. It then asks `pairwise_rank` for the option implied by the recorded assessment-time scores and the configured class gate; that independent route returns absence at zero, three, and four positives and presence at five, without fitting a numerical AUC expectation (´alg:monitoring:auc´), (´dec:harness:oracle-tier´).

At the first three checkpoints `HealthSummary::auc_aggregate` and `SystemHealthReport::discrimination` must both be absent. At the five-positive checkpoint the cheap aggregate and the detailed object's aggregate must both be present. No assertion requires either calibration parameter to move at three raw positives, because fitting reads weighted regime counts while AUC publication reads raw class counts.

The fails-before implementation computes or publishes aggregate AUC when both classes are merely non-empty, ignores the configured class gate, substitutes a number for absence, or wraps absent fields in a detailed discrimination object. Each weakening returns a public value at the three- or four-positive checkpoint; an implementation that never publishes discrimination fails the five-positive control.

The witness advances no time, authors no wait, uses no trained-state fixture, and performs no seed sweep. Its only asynchronous dependency is label publication, named by the playback barrier.

## What is missing · `sec:assayer:intent-discrimination-is-withheld-until-there-are-positives-missing`

No reusable harness primitive, probe, fixture, production accessor, clock seam, tolerance, or liveness deadline is missing.

**Entry (Public sparse-positive discrimination witness)** · `entry:assayer:intent-discrimination-is-withheld-until-there-are-positives-public-witness`

Add roughly a hundred lines to the `assess_integration` target for the subject-owned playback row, the four configured blocks and checkpoints, the independent option-presence oracle, and assertions on both public health tiers. Update that target's test index and the generated integration catalogue. The work consumes the completed playback, barrier, and oracle entries (´entry:assayer:harness-tape-runner´), (´entry:assayer:harness-closed-barriers´), (´entry:assayer:harness-oracle-tier´) and has no sibling-plan dependency.

## Risks and open questions · `sec:assayer:intent-discrimination-is-withheld-until-there-are-positives-risks`

**Observation (Three and five govern different operations)** · `obs:assayer:intent-discrimination-is-withheld-until-there-are-positives-two-thresholds`

The configured calibration positive floor defaults to three, while the AUC class gate defaults to twenty and cannot be configured below five (´tab:config:calibration´), (´tab:config:monitoring´). The three-positive checkpoint proves withholding after the raw positive count has reached the lower configured number; it does not infer a regime fit from that count, because regime eligibility is weighted.

**Observation (Raw labels and effective regime samples are distinct)** · `obs:assayer:intent-discrimination-is-withheld-until-there-are-positives-count-semantics`

AUC counts raw retained rows per selected population, while calibration fitting counts importance- and regime-weighted samples (´constr:platt:buffer´), (´req:platt:minimum-samples´). The local row ledger and `pairwise_rank` guard the raw class schedule; `HealthSummary::platt_refits_completed` guards that the intended periodic computation ran, not that either weighted fit changed a parameter.

**Observation (Seen and retained differ after eviction)** · `obs:assayer:intent-discrimination-is-withheld-until-there-are-positives-retention-scope`

The claim concerns retained calibration rows, and the cold-start witness stays below capacity. Production retains the last successful discrimination object when a later refit cannot compute aggregate AUC (´dec:health:cached-discrimination´), so disappearance after eviction would be a separate stale-cache promise rather than part of this boundary witness.

**Observation (Presence has no numerical fragility)** · `obs:assayer:intent-discrimination-is-withheld-until-there-are-positives-determinism`

The oracle is option presence at exact integer class counts. Playback crosses `PlaybackBarrier::FlushLabels` before each checkpoint; its exclusion of the cold-observation queue cannot change row valence or the AUC gate, and no assertion depends on score ordering. Fixed rows, a refit counter, a non-evicting buffer, and the specification-formula `pairwise_rank` route leave no floating-point tolerance or scheduler-dependent wait in the verdict.

## Acceptance · `sec:assayer:intent-discrimination-is-withheld-until-there-are-positives-acceptance`

Acceptance names cargo integration target `assess_integration` and Rust test path `discrimination_is_withheld_until_positive_evidence_reaches_the_public_gate`, and the test cites (´claim:wellness:discrimination-is-withheld-until-there-are-positives´) at its standard documentation position.

The test reports the configured fifty-label refit cadence and five-row AUC gate; proves cumulative positive counts zero, three, four, and five with ample negatives; and shows the public refit counter advancing at each checkpoint. It reports the oracle's absent, absent, absent, and present option sequence, absence on both public health tiers at the first three checkpoints, and aggregate presence on both tiers at the last.

The implementing change refreshes the target's test index and generated integration catalogue, contains no sleep, poll, local deadline, new harness surface, or production change, and passes the package's required formatting, lint, test, and corpus-name gates. The fails-before argument identifies the below-gate publication and never-publication defects that the two boundary assertions reject.
