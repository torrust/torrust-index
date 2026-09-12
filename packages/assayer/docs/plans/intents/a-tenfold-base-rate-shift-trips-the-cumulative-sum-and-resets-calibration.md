# Keeping the Tenfold Base-Rate Drift Promise · `plan:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration`

This plan keeps the promise that a tenfold change in adverse-label prevalence is detected before its five-hundred-label transition ends and that the resulting calibration displacement clears stale drift evidence (´claim:wellness:a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration´).

## What the promise says, precisely · `sec:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-precise-promise`

The stimulus has two contiguous label phases on one unchanged request population: one thousand labels at a 5% adverse rate, followed immediately by five hundred labels whose adverse rate moves monotonically from 5% to 50%. There is no post-shift tail from which a late detector can borrow evidence.

Every adverse label carries positive valence and every benign label non-positive valence, so the binary training target is respectively $+1$ and $-1$ (´def:risk:target´). Ground-truth labels keep both classes eligible, and the label path updates the class-rate trackers, models, calibration buffer, and drift state in its specified order (´alg:runtime:update-path´).

For each label, a one-sided accumulator banks the assessment-time residual beyond the configured allowance $\kappa_\text{drift}=0.1$; the positive and negative directions cannot cancel (´alg:monitoring:drift-cusums´). The automatic trigger is strict: $S^+ > h$ or $S^- > h$ with $h=10$, followed by a CUSUM reset and a drift event (´tab:monitoring:drift-resets´), (´tab:config:monitoring´).

Periodic calibration refits occur every two hundred labels once the weighted per-regime sample floors are available, over the two-thousand-record calibration buffer (´tab:platt:refit-cadence´), (´constr:platt:buffer´), (´tab:config:calibration´). A refit computes $\delta_\text{cal}=|\log \kappa_\text{new}-\log \kappa_\text{old}|$ per fitted regime; a strict crossing of $0.1$ resets the affected drift accumulators and reports the event (´alg:platt:drift-integration´).

The detection deadline is the publication following label 1,500. An automatic-reset event first observed only after another assessment or label is late, because the transition has already ended.

The calibration half means that calibration is refitted and its displacement causes a drift-state reset; it does not mean that either fitted calibration parameter returns to its initial value.

## What the code offers today · `sec:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-current-code`

The world harness supplies `scenario_with_config`, `World::settle_cold_ramp_with`, `World::cycle_default`, `World::flush_labels`, and the `World::assayer` escape hatch. The label specification builder supplies adverse and benign `LabelSpec` builders and the ground-truth marker. These pieces already drive a real engine through the public assessment and label surfaces; they are the landed skeleton (´entry:assayer:harness-stage-skeleton´).

`World::settle_cold_ramp_with` removes asynchronous standardisation progress from the experiment before the first labelled phase. Each later cycle assesses the same request, attaches the tape outcome to that assessment identifier, submits the label, and waits for the model owner to publish it before the next prediction is made.

`Assayer::full_health_report()` exposes `calibration.delta_cal`, both fitted calibration parameters, and each model's two accumulator values and `steps_since_reset`. `Assayer::drain_health_events()` exposes ordered `DriftReset` events; automatic resets carry zero calibration displacement, while calibration-triggered resets carry a positive displacement and a count of affected models. The full query and bounded event stream are deliberate public tiers (´dec:health:tiered-queries´), (´dec:health:bounded-events´).

The production label path refits and applies a calibration-triggered reset before updating the same label's drift accumulators. A health reading immediately after that label therefore shows an affected `steps_since_reset` of at most one rather than requiring zero.

The nearest public streaming witness is the five-hundred-cycle signal-population test, which establishes the assess-label-flush shape but not drift or calibration (´test:integration:scalar-signal-population-split-converges-directionally´). The nearest health-surface witness proves that assessments alone move neither labels nor calibration (´test:integration:repeated-assessment-advances-no-outcome-learned-state´).

The mechanism tests remain separate: one proves that an accumulator crossing reports its automatic reset (´test:crate:drift-auto-reset-triggers´), one proves that fitted calibration can exceed the displacement threshold (´test:unit:delta-cal-exceeds-threshold´), and one proves that the current calibration reset narrows to the affected regime (´test:unit:calibration-drift-reset-narrows-to-the-affected-regime´). No existing test composes these mechanisms through the public label stream.

The standing testing plan places this promise in the open tapes-and-fixtures stage (´entry:assayer:harness-stage-tapes´).

## The witness · `sec:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-witness`

- Placement: a focused drift-detection integration module carries the test index and one public witness named `tenfold_base_rate_shift_trips_cusum_and_resets_calibration_drift`.
- Configuration: `scenario_with_config` pins `monitoring.kappa_drift` to `0.1`, `monitoring.h_threshold` to `10.0`, `platt.delta_cal_threshold` to `0.1`, `platt.n_refit` to `200`, and the remaining calibration buffer and sample floors to their specified defaults; the explicit assignments make every oracle visible at setup.
- Population: one channel, one entity, no Sentinels, signals, identities, or axes, and one settled request vector isolate a change in the outcome base rate from a change in feature distribution.
- Stable stimulus: the first one thousand tape positions contain exactly fifty adverse labels, distributed as one adverse position in every consecutive block of twenty, with every label marked as ground truth.
- Ramp stimulus: the next five hundred positions use $p_i=0.05+0.45i/499$ and a cumulative-quota scheduler that emits an adverse label whenever the integral count crosses the next integer. The prefix discrepancy from the declared linear rate stays below one label without seeded sampling noise.
- Observation: after every assess-label-flush cycle, the runner drains events and captures the full health report with the current label index, phase, fitted calibration parameters, `delta_cal`, per-model accumulator state, reset counter, and dropped-event count.
- Baseline control: the last two-hundred-label refit interval of the stable phase contains no automatic-reset event, the event channel loses nothing, and the boundary report has finite calibration and drift readings. This control makes a ramp event evidence of the shift rather than evidence of a detector that alarms continuously on the 5% regime.
- CUSUM assertion: at least one zero-displacement `DriftReset` event occurs at an index from 1,001 through 1,500 inclusive. Its automatic-reset semantics establish the strict crossing of the configured $h=10$ even though the crossing value is cleared before public state can expose it.
- Calibration assertion: at least one refit published in the same ramp has an individual sister or anchor log-parameter displacement strictly above `0.1`, and the event drain at that index contains a positive-displacement `DriftReset` with a non-zero affected-model count.
- Reset assertion: every regime whose fitted parameter moved past the threshold has `steps_since_reset <= 1` in the report following that label, while a regime below the threshold keeps its earlier step history unless an independently recorded automatic event reset it on the same label.
- Causality assertion: the trace contains no labels beyond 1,500, so neither headline assertion can pass from evidence gathered after the shift.

The deliberately disabled or delayed detector produces no zero-displacement reset event by label 1,500. A refit that reports $\delta_\text{cal}>0.1$ without applying the coupling produces a calibration event/state mismatch. A detector that alarms throughout the stable suffix fails the negative control before the shift begins. Recomputing residuals after model update reduces the accumulated error and moves or removes the in-ramp crossing, which is the failure the assessment-time pending value prevents.

## What is missing · `sec:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-missing`

**Entry (A deterministic base-rate label tape)** · `entry:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-base-rate-tape`

A new label-tape harness module gains a pure label-outcome sequence with constant-rate and linear-rate constructors, exact cumulative-quota scheduling, phase metadata, and iterator access. The focused implementation is approximately 60–80 lines and depends on the open general tape stage (´entry:assayer:harness-stage-tapes´), not on another intent lane.

**Entry (Label-indexed playback and health tracing)** · `entry:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-labelled-trace`

The world harness gains tape playback that performs an online cycle per outcome and invokes a checkpoint callback after the label barrier; the test-support module exports the tape. The test-local checkpoint collector attaches drained events and full-health readings to the tape index. This is approximately 50–70 harness lines plus 30–40 test-helper lines, depends on the base-rate-tape entry, and needs no production accessor or model-state escape hatch.

## Risks and open questions · `sec:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-risks`

**Observation (The ramp needs a deterministic interpolation)** · `obs:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-ramp-interpolation`

The promise fixes endpoints and duration but not the interpolation. A deterministic linear cumulative-quota ramp is the strongest ordinary reading: an abrupt step does not exercise detection while a change is unfolding, and Bernoulli sampling makes both the realised rate and the crossing label seed-dependent.

**Observation (A stable-regime false alarm can counterfeit detection)** · `obs:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-baseline-confound`

The label path currently feeds the same stored effective residual to the operational, sister, and anchor drift states. The specification describes per-model assessment-time predictions. The stable suffix control exposes a material consequence of that difference without expanding this witness into a separate per-model conformance claim.

**Observation (The crossing is intentionally transient)** · `obs:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-transient-crossing`

Automatic reset clears the value that exceeded $h$ before the next health report. The zero-displacement reset event is therefore the public crossing witness; sampling only `full_health_report()` can miss a correct detection and sampling only the monotone reset counter cannot place it within the ramp.

**Observation (Calibration reset scope is inconsistent)** · `obs:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-reset-scope`

Total reset in the calibration decision (´dec:calibration:drift-reset´) and affected-regime reset in the specification and current label path (´alg:platt:drift-integration´) differ. This promise needs only their common consequence: a fitted displacement past `0.1` resets at least the affected regime. Total-versus-narrow scope remains a separate decision and does not block this witness.

**Observation (The event stream is bounded)** · `obs:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-event-loss`

Draining after every flushed label keeps the event-to-label join deterministic and prevents expected traffic from filling the bounded channel. A non-zero dropped-event count invalidates the trace instead of being treated as absence of a reset (´dec:health:bounded-events´).

## Acceptance · `sec:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-acceptance`

The implementing lane reports the exact tape construction, explicit configuration, stable-suffix control, first automatic-reset label, each in-ramp calibration refit and individual log displacement, the corresponding reset event, affected post-label step counts, and a zero dropped-event count.

The new integration test cites the intent, appears in its module test index and the generated integration matrix, uses only `World` plus public health surfaces after the two named harness entries land, and passes under the package's required debug, release, no-default-features, documentation, formatting, and lint gates.

The report includes the assertion output from a normal run and describes the disabled-detector, missing-coupling, continuous-baseline-alarm, and post-update-residual failures without executing mutations.
