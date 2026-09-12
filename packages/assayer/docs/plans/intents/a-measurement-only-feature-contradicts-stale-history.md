# Measurement disagrees with stale history · `plan:assayer:intent-a-measurement-only-feature-contradicts-stale-history`

Keeping (´claim:risk:a-measurement-only-feature-contradicts-stale-history´) establishes that normalized live Sentinel measurements and adverse outcome memory remain distinguishable at prediction time, that sparse eligible benign labels turn the distinction into model-specific drift evidence, and that elapsed-time Ledger decay reduces the discrepancy on its specified horizon.

## What the promise says precisely · `sec:assayer:intent-a-measurement-only-feature-contradicts-stale-history-precision`

Anchor position 13 is $z_{\max}=\max_s\max_a z_{s,a}$, the maximum current z-score over reporting Sentinels and scoring axes. It is computed as the fourteenth value of the fixed fifteen-position projection, while position 14 is the logarithm of one plus the reporting-Sentinel count (´def:dimension:anchor-projection´).

The provenance is the first invariant: $z_{\max}$ comes from the current batch report and never from the Ledger or identity outcome memory. Changing stale outcome history while holding the report fixed cannot change it; replacing an alarming report by a normalized report changes it immediately without waiting for a label (´prop:risk:anchor-measurement-only´) (´setup:extraction:from-batch-report´).

The contrasting quantity is the deepest routed cell's outcome-memory block: adverse rate, compressed valence, raw valence, and any spatial-axis pairs are read from the Ledger rather than from the report (´tab:extraction:ledger-features´) (´def:ledger:purpose´). The witness uses no identity dimension, so anchor positions 7 through 12 are zero and cannot provide a second historical path.

The training target is $r_\rho=-1$ for a benign label and $+1$ for an adverse label (´def:risk:target´). For a benign probe, model $m$ grows its negative accumulator by $\max(0,\hat\rho_m+1-\kappa_{\mathrm{drift}})$ before any threshold reset, so stale-history over-prediction is measured as $\Delta S^-_m>0$ (´alg:monitoring:drift-cusums´).

The shipped allowance is $\kappa_{\mathrm{drift}}=0.1$ and the strict automatic-reset boundary is $h=10$ (´tab:config:monitoring´) (´tab:monitoring:drift-resets´). The scenario leaves the allowance at its default and sets its test-local $h_{\mathrm{probe}}$ to ten times the shipped boundary so that cumulative evidence remains readable; it separately asserts that no reset event occurred.

Ledger averages apply $\lambda_L=0.999$ at each eligible label and $\gamma_{t,L}=0.999$ per elapsed hour at access. Under starvation, the latter takes an adverse rate to about one half after twenty-nine days, one quarter after fifty-eight, and one eighth after eighty-seven (´def:ledger:time-decay´) (´thm:temporal:ledger-decay-bound´).

The measurable promise is therefore a conjunction. With the normalized report held fixed and a material adverse Ledger value present, consecutive sparse benign probes produce a larger negative-CUSUM increment for the sister than for the anchor by more than the harness EWMA tolerance; the Ledger value follows the configured decay recurrence as time advances; and the late-horizon sister-minus-anchor increment is smaller than the pre-decay increment. The CUSUM need not fall, because its recurrence only accumulates or resets; elapsed decay removes the continuing increment rather than erasing evidence already banked.

This interpretation also fixes the role of the blend. The anchor's residual blend weight is deployment-dependent and is not an exact floor (´rem:risk:blend-mechanisms´) (´dec:risk:anchor-floor´); the witness reads the per-model drift streams, not a minimum change in the blended probability.

## What the code offers today · `sec:assayer:intent-a-measurement-only-feature-contradicts-stale-history-code-today`

The host-facing route is present. The world harness supplies `World::cold`, Sentinel registration, `receive_report`, `request_with_sentinel`, `assess`, `label`, `flush_labels`, `settle_cold_ramp_with`, and access to the underlying `Assayer`; `LabelSpec` supplies eligible benign and adverse labels. The completed harness skeleton records these public scenario verbs (´entry:assayer:harness-stage-skeleton´).

`World::clock` exposes the `VirtualClock`, and `VirtualClock::advance` drives the same persistent domain used by assessment-time Ledger reads and label-time Ledger updates. The standing persistent-clock entry explicitly covers this promise and remains partly complete, but its already-injected assessment and label sites are the sites this witness consumes (´entry:assayer:harness-stage-clock´) (´dec:clock:two-domains´).

The report fixtures already construct cell reports and re-ingest a current report through `refresh_golden_report`. `DimensionMap::extract_anchor_subvector` computes position 13 from the four aggregate per-axis maximum-z positions and position 14 from the reporting count; the existing projection test proves those two positions are computed rather than gathered (´test:crate:anchor-subvector-computes-its-last-two-inputs´).

The public observation surface is adequate for the behavioural result. `RiskBasis::p_bad_sister` exposes the sister prediction, and `Assayer::full_health_report` publishes `DriftHealth` by `ModelId`, including both CUSUMs, residual summaries, and the steps since reset. The host reset path already has a focused witness (´test:crate:host-initiated-drift-reset-reaches-the-accumulators´).

The nearest integration arithmetic establishes the configured twenty-nine-day decay factor without driving a Ledger cell (´test:integration:decay-factor-29-day-half-life´). The existing Outcome Ledger integration suite drives Sentinel lifecycle and repeated public assessments but contains no time-decay, normalized-report, or model-disagreement scenario. The standing gap names the missing behavioural anchor coverage rather than another mechanism test (´entry:assayer:gap-anchor-behaviour´).

One production path blocks the intended witness. `BlendResult` computes separate `rho_inherited` and `rho_anchor` values, but `PendingRiskBasis` retains only `rho_effective`; label step sixteen consequently computes one blended residual and applies it to operational, sister, anchor, and outcome-axis drift states. That makes the published risk-model CUSUM trajectories identical even though the specification requires each model's assessment-time prediction (´alg:runtime:update-path´) (´def:runtime:pending-entry´) (´inv:guarantee:drift-visibility´).

## The witness · `sec:assayer:intent-a-measurement-only-feature-contradicts-stale-history-witness`

The integration test belongs beside the existing public Sentinel and Ledger lifecycle tests and uses a fixed seed, one channel, one Sentinel, no identity dimensions, the default decay constants, the default drift allowance, and the elevated observation-only reset threshold described above.

The setup builds a matched report pair with identical cell intervals, sample counts, coverage, coordination values, and reporting count. The training report has a quiet region and a loud region; the normalized report preserves the geometry but sets every z-score and current Sentinel cumulative-sum reading on the affected chain to its quiet value, so report-shape or coverage changes cannot explain the result.

The cold ramp observes both request shapes before training. A deterministic eligible stream then routes benign labels through the quiet region and adverse labels through the loud region until the public sister probabilities separate in the intended direction and the deepest loud-cell adverse rate clears a named materiality floor. Training has a fixed upper bound and fails with the observed probabilities and Ledger reading if that bound is reached; it never waits on wall time.

The stimulus ingests the normalized report at the current virtual monotonic reading, giving it a fresh arrival stamp, and keeps routing the formerly loud coordinate. A small fixed tranche of eligible benign probes follows immediately, small enough to remain below one hundredth of the training label count, so the labels make the stale estimate measurable without becoming enough evidence to retrain it away.

After every probe, `flush_labels` supplies the publication barrier and `full_health_report` supplies the observation. The assertion records the stepwise increments of `ModelId::Sister` and `ModelId::Anchor`, requires each immediate sister increment to be positive, and requires the sister increment to exceed the anchor increment by more than `World::tol().ewma` on every immediate probe.

The clock then advances to the twenty-nine-, fifty-eight-, and eighty-seven-day horizons without another report change. At each horizon the test assesses the same request before submitting one eligible benign probe, reads the decayed Ledger rate through the focused harness accessor, and compares it with the independent recurrence over $\gamma_{t,L}$ and the intervening $\lambda_L$ benign updates.

The late-horizon assertions require the report-derived anchor stream to remain within its immediate normalized band, the sister public probability and negative-CUSUM increment to move toward the anchor's, and the eighty-seven-day sister-minus-anchor increment to be materially below its pre-decay value. The Ledger oracle uses the EWMA tolerance, while probability and CUSUM comparisons are relational and use named margins derived from the drift allowance rather than exact learned coefficients.

The test also asserts that every normalized assessment still reports the Sentinel, that its report timestamp is current, that the CUSUM step counters equal the submitted probe count, and that `drift_resets` does not change. These guards exclude missing-report fallback, staleness, asynchronous publication, and threshold reset as alternate explanations.

The fails-before is diagnostic. The current blended-residual implementation advances sister and anchor CUSUMs identically and fails the first differential assertion; an anchor projection that gathered a stale-history position makes the anchor follow the sister after normalization; an assessment that reuses the old report fails the current-report guards; and a Ledger that ignores persistent time keeps both its rate and the sister's excess increment high at the decay horizons.

## What is missing · `sec:assayer:intent-a-measurement-only-feature-contradicts-stale-history-missing`

**Entry (Pending state carries each model's assessment-time prediction)** · `entry:assayer:intent-a-measurement-only-feature-contradicts-stale-history-per-model-pending-predictions`

Extend the pending risk basis with the operational, sister, and anchor linear predictions already computed during assessment, retain the matching raw assessment-time prediction for every active outcome axis, and make step sixteen select the stored value belonging to each `ModelId`. Constructor, replay, persistence, and focused label-path expectations change together. This is approximately ninety to one hundred and forty lines across assessment construction, pending storage, label processing, and their crate tests; it depends on no sibling lane and is required independently by (´alg:monitoring:drift-cusums´).

**Entry (A matched alarming and normalized report pair)** · `entry:assayer:intent-a-measurement-only-feature-contradicts-stale-history-matched-report-pair`

Add a report fixture to the existing report-fixture module that returns the two fixed-geometry surfaces for ingestion through `World::receive_report`. The helper exposes the quiet and loud coordinates and changes only live score fields between variants. This is approximately thirty to fifty lines plus its fixture check, depends on the existing report builders, and has no sibling-lane dependency.

**Entry (World reads one decayed Ledger cell as a test precondition)** · `entry:assayer:intent-a-measurement-only-feature-contradicts-stale-history-decayed-ledger-probe`

Add a narrowly hidden test-support query that resolves a named Sentinel and coordinate through the production depth walk and returns the bad-rate value after pure read-time decay at the world's current persistent clock. The method exposes no mutable model state and exists to prove that the behavioural change is paired with the configured Ledger recurrence rather than with sparse retraining. This is approximately forty to seventy lines across the internal query and world harness, depends on the persistent-clock stage (´entry:assayer:harness-stage-clock´), and has no sibling-lane dependency.

The existing `World::clock().advance`, label barrier, full health report, reset threshold configuration, and tolerance bundle are sufficient. No tape, persistence fixture, random schedule, wall-clock sleep, or general model-state escape hatch is required.

## Risks and open questions · `sec:assayer:intent-a-measurement-only-feature-contradicts-stale-history-risks`

**Observation (The behavioural witness composes with the projection witness)** · `obs:assayer:intent-a-measurement-only-feature-contradicts-stale-history-provenance-composition`

The integration result cannot attribute an anchor prediction change to position 13 alone because positions 0 through 6 also carry current aggregate measurements. The existing projection test proves the exact $z_{\max}$ computation, while the new integration test proves that the anchor's report-derived subspace diverges from the stale Ledger path. Together they keep the promise without exposing a test-only anchor feature vector or claiming that one coefficient has a guaranteed magnitude.

**Observation (The training margin is empirical but bounded)** · `obs:assayer:intent-a-measurement-only-feature-contradicts-stale-history-training-margin`

Bayesian updates, standardisation, and the leverage bound make a guessed fixed training count fragile. The implementation lane measures one deterministic seed, names the smallest stable upper bound that clears both the public probability separation and Ledger materiality preconditions, and retains the bound as a liveness failure rather than looping until convergence. No acceptance assertion rests on an exact learned probability.

**Observation (Sparse probes also update the memory they inspect)** · `obs:assayer:intent-a-measurement-only-feature-contradicts-stale-history-probe-perturbation`

Every eligible benign probe applies label-indexed Ledger decay and then moves the average toward zero before the next assessment (´alg:ledger:all-layers-update´). The independent oracle includes that update, and the probe-to-training ratio keeps it subordinate to elapsed-time decay. Treating the initial Ledger value as though only the clock touched it would overstate the expected horizon values.

**Observation (Per-model retention is a conformance prerequisite)** · `obs:assayer:intent-a-measurement-only-feature-contradicts-stale-history-per-model-conformance`

The specification admits no interpretation in which one blended residual is every model's residual: each CUSUM is defined against that model's stored assessment-time prediction. Retaining separate values is therefore the preferred path. Recasting the promise as a blended-score test would make the current code testable but would not keep either the per-model monitoring rule or the stated anchor-versus-history disagreement.

**Observation (The clock dependency is narrower than the standing stage)** · `obs:assayer:intent-a-measurement-only-feature-contradicts-stale-history-clock-scope`

The standing persistent-clock entry still includes production call sites outside assessment and labelling and still lacks semantic `World::advance` verbs. This scenario directly advances the exposed `VirtualClock` and reaches only already-injected Ledger reads and writes, so it depends on the entry's landed portion and does not wait for unrelated timestamp sites.

## Acceptance · `sec:assayer:intent-a-measurement-only-feature-contradicts-stale-history-acceptance`

The implementation report names the integration test and its claim citation, the three entries landed, the fixed seed and bounded training count, the immediate and horizon Ledger readings, the sister and anchor CUSUM increments, the sister probabilities, the configured allowance and reset threshold, and the unchanged drift-reset counter.

The report shows that the projection mechanism test and the new public-path scenario both pass, and that the scenario fails when every model receives the blended residual, when the anchor's computed maximum is replaced by a stale-history input, and when Ledger time decay is disabled, whether those falsifiers are demonstrated by targeted mutation or by direct identification of the assertion each defect violates.

Acceptance requires deterministic virtual time, refreshed reports, explicit publication barriers, no exact learned-coefficient oracle, no widened shared tolerance, no identity dimension, no unbounded convergence loop, and no direct mutation of Ledger or model state. The package's formatting, lint, all-target all-feature tests, no-default-features spot check, release tests, documentation build, and documentation tests are green with the new scenario included.
