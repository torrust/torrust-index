# Keeping the Tenfold Base-Rate Drift Promise · `plan:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration`

This plan keeps the promise that a tenfold change in adverse-label prevalence is detected before its five-hundred-label transition ends and that a calibration displacement past the specified threshold resets the drift evidence measured under the superseded calibration (´claim:wellness:a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration´).

## What the promise says, precisely · `sec:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-precise-promise`

The stimulus has two contiguous label phases on one unchanged request population: one thousand labels at a 5% adverse rate, followed immediately by five hundred labels whose adverse rate moves monotonically from 5% to 50%. There is no post-shift tail from which a late detector can borrow evidence.

Every adverse label carries positive valence and every benign label non-positive valence, so the binary training target is respectively $+1$ and $-1$ (´def:risk:target´). Marking every label as ground truth makes either class eligible independently of the recorded action (´tab:eligibility:training´), and the label path updates the class-rate trackers, models, calibration buffer, drift state, and publication in its specified order (´alg:runtime:update-path´).

For each label, the positive and negative accumulators independently bank the assessment-time residual beyond the configured allowance $\kappa_\text{drift}=0.1$ (´alg:monitoring:drift-cusums´). The automatic trigger is the strict crossing $S^+ > h$ or $S^- > h$ with $h=10$; it emits a drift event and clears the accumulators (´tab:monitoring:drift-resets´) (´tab:config:monitoring´).

Periodic calibration refits are attempted every two hundred labels, fit each regime only after its weighted record and class floors are met, and read at most the two-thousand-record calibration buffer (´tab:platt:refit-cadence´) (´req:platt:minimum-samples´) (´constr:platt:buffer´) (´tab:config:calibration´). For each fitted regime the refit computes $\delta=|\log \kappa_\text{new}-\log \kappa_\text{old}|$; a strict crossing of `0.1` resets the affected regime's drift accumulators and reports the event (´alg:platt:drift-integration´).

The detection deadline is the publication following label 1,500. An automatic reset first observed only after another assessment or label is late, because the transition has already ended.

The calibration half requires a refit whose displacement exceeds `0.1` and a corresponding drift-state reset; it does not require either fitted calibration parameter to return to its initial value.

## What the code offers today · `sec:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-current-code`

The unified real-engine harness supplies `scenario_with_config`, `World::settle_cold_ramp_with`, `World::derive_default`, `World::label`, `World::flush_labels`, `World::drain_health_events`, the `World::assayer` escape hatch, and `LabelSpec::adverse`, `LabelSpec::benign`, and `LabelSpec::ground_truth`. These are current members of the finished library roster rather than a staged skeleton (´dec:harness:single-scenario´) (´tab:assayer:harness-implementation-library-roster´).

Declarative long-stream support has landed as `PlaybackRow`, `PlaybackBarrierPolicy`, `PlaybackBarrier`, `PlaybackBatchSize`, `PlaybackCheckpoint`, `PlaybackProgress`, and `playback`. A row submits one assessment and label without choosing its own wait; `PlaybackBarrier::FlushLabels` publishes that label before its checkpoint, an ordering the focused harness test already establishes (´dec:harness:declarative-playback´) (´test:crate:selected-label-barrier-completes-before-checkpoint´).

`World::drain_health_events` crosses every event-producing queue before returning the accepted events and leaves the receiver empty at the drain point (´test:crate:health-event-barrier-orders-producers-before-drain´). After that barrier, `Assayer::full_health_report` exposes `CalibrationHealth::kappa_sister`, `CalibrationHealth::kappa_anchor`, `CalibrationHealth::delta_cal`, `CalibrationHealth::refits_completed`, each model's `DriftHealth::s_plus`, `DriftHealth::s_minus`, and `DriftHealth::steps_since_reset`, and `SystemHealthReport::health_events_dropped`; `LifecycleHealthEvent::DriftReset` distinguishes an automatic reset by zero displacement from a calibration reset by positive displacement and reports the number of models reset. The comprehensive query and bounded event stream remain deliberate public tiers (´dec:health:tiered-queries´) (´dec:health:bounded-events´).

The production label path applies a calibration-triggered reset before updating the same label's drift accumulators. A health reading after that publication therefore shows an affected `DriftHealth::steps_since_reset` no greater than one, not necessarily zero.

The finished probe tier provides `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView`; the oracle tier provides `OracleProvenance`, `pairwise_rank`, `regularised_schur_complement`, `decay_recurrence`, and `dimension_width`; the guarded setup tier provides `TrainedStateFixture` and `TrainedStateBaseline`; and seeded invariant iteration is `run_seeded_sweep` (´dec:harness:probe-contract´) (´dec:harness:oracle-tier´) (´dec:harness:guarded-fixtures´) (´dec:harness:seeded-sweeps´) (´tab:assayer:harness-implementation-library-roster´). None substitutes for this witness: the probes do not project health events or calibration, no shared oracle computes this deterministic tape's expected refit, the fixed trained fixture would replace rather than establish the promised thousand-label phase, and the stimulus has no seed sweep.

The component tests remain separate: one proves that an accumulator crossing reports its automatic reset (´test:crate:drift-auto-reset-triggers´), one proves that a fitted calibration can exceed the displacement threshold (´test:unit:delta-cal-exceeds-threshold´), and one proves that the calibration reset narrows to the affected regime (´test:unit:calibration-drift-reset-narrows-to-the-affected-regime´). The `calibration_discrimination_convergence` target still contains no witness, and no existing test composes the tenfold stream, the in-ramp automatic event, and the calibration-triggered reset.

## The witness · `sec:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-witness`

- Placement: the `calibration_discrimination_convergence` integration target carries the test index and the public witness `calibration_discrimination_convergence::tenfold_base_rate_shift_trips_cusum_and_resets_calibration_drift` under the convergence-topic split (´dec:harness:convergence-split´).
- Configuration: `scenario_with_config` sets `config.monitoring.kappa_drift` to `0.1`, `config.monitoring.h_threshold` to `10.0`, `config.platt.delta_cal_threshold` to `0.1`, and `config.platt.n_refit` to `200`; it leaves the calibration capacity and sample floors at their specified defaults, then `World::settle_cold_ramp_with` removes cold standardisation progress from the label experiment.
- Population: one channel, one entity, no Sentinels, signals, identities, or axes, and one unchanged request isolate the outcome base rate from feature-distribution movement.
- Stable stimulus: the first one thousand rows contain exactly fifty adverse outcomes, with the twentieth row of every consecutive block adverse and all other rows benign, and mark every label as ground truth.
- Ramp stimulus: for zero-based ramp position $i\in[0,499]$, the declared rate is $p_i=0.05+0.45i/499$; a cumulative-quota scheduler starts with quota one and emits an adverse outcome whenever the running sum of these rates reaches or passes the next integer, then increments the quota. The realised prefix count differs from the declared cumulative rate by less than one label and contains no sampling noise.
- Playback: a witness-local row implements `PlaybackRow` by calling `World::derive_default`, constructing a ground-truth `LabelSpec`, and calling `World::label` without flushing. The guarded fixture invokes `playback` for the stable rows, then the test invokes it for the ramp rows on the returned scenario; both calls use `PlaybackBarrierPolicy::new([PlaybackBarrier::FlushLabels])`, `PlaybackBatchSize::default()`, one `PlaybackCheckpoint::after_row` per row, and a `PlaybackProgress` reading, so the runner rather than the row owns completion semantics (´dec:harness:declarative-playback´) (´dec:harness:no-ad-hoc-waits´).
- Observation: each checkpoint calls `World::drain_health_events`, then loads one `Assayer::full_health_report` through `World::assayer` and records the phase offset plus the zero-based playback location as a one-based global label index, together with the events, fitted parameters, `CalibrationHealth::delta_cal`, `CalibrationHealth::refits_completed`, per-model drift state, and `SystemHealthReport::health_events_dropped`. Comparing `CalibrationHealth::refits_completed` with the preceding checkpoint identifies exactly which label published a refit.
- Baseline guard: a witness-local guarded fixture plays the stable phase and returns the same scenario and its trace only when the last two hundred stable labels, one specified periodic cadence, contain no zero-displacement `LifecycleHealthEvent::DriftReset`, the dropped-event count is zero, and the boundary report's calibration and drift readings are finite. Failure is a setup refusal with the measured baseline, separate from the ramp assertions (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).
- CUSUM assertion: at least one zero-displacement `LifecycleHealthEvent::DriftReset` occurs at a label index from 1,001 through 1,500 inclusive. The event establishes the strict crossing of the configured $h=10$ even though the accumulator that crossed is cleared before public state can expose it.
- Calibration assertion: at least one refit published in that same inclusive range moves `CalibrationHealth::kappa_sister` or `CalibrationHealth::kappa_anchor` by a strict absolute log difference greater than `0.1`, reports that maximum as `CalibrationHealth::delta_cal`, and emits a positive-displacement `LifecycleHealthEvent::DriftReset` with a non-zero affected-model count at the same checkpoint.
- Reset assertion: for each sister or anchor parameter whose log displacement strictly exceeds `0.1`, the corresponding `DriftHealth::steps_since_reset` is no greater than one in the report following that label. A regime below the threshold advances its preceding step count by one unless a zero-displacement event independently reset it on the same label.
- Causality assertion: playback contains no row after label 1,500, so neither headline assertion can pass with evidence gathered after the shift.

A disabled or delayed detector produces no zero-displacement reset by label 1,500. A refit that reports a displacement above `0.1` without applying the coupling produces an event-and-state mismatch. A detector that alarms throughout the stable suffix fails the baseline guard. Recomputing the residual after the model update reduces the accumulated error and moves or removes the in-ramp crossing, which is the failure the retained assessment-time prediction prevents.

## What is missing · `sec:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-missing`

**Entry (A deterministic base-rate label tape)** · `entry:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-base-rate-tape`

The integration target still needs a witness-local typed row, a pure constructor for the constant-rate and linear cumulative-quota phases, and a guarded stable-phase fixture that returns the configured scenario and measured trace only after the baseline precondition passes. The row implements the existing `PlaybackRow` contract and owns only the outcome and phase metadata; the fixture follows the landed guard/result separation, and no shared tape module, random sampler, production accessor, or new harness verb is missing.

**Entry (Label-indexed playback and health tracing)** · `entry:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-labelled-trace`

The integration target still needs a witness-local trace collector that builds one `PlaybackCheckpoint` for each declared row, captures the vector returned by `World::drain_health_events`, and then copies the public health fields used by the assertions. Playback, the label barrier, event-producer ordering, bounded progress, and full-health access already exist; the remaining work is the test's typed observation record and assertions.

## Risks and open questions · `sec:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-risks`

**Observation (The ramp needs a deterministic interpolation)** · `obs:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-ramp-interpolation`

The promise fixes endpoints and duration but not interpolation. A deterministic linear cumulative-quota ramp is the strongest ordinary reading: an abrupt step does not exercise detection while a change is unfolding, and Bernoulli sampling makes the realised rate and crossing label seed-dependent.

**Observation (A stable-regime false alarm can counterfeit detection)** · `obs:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-baseline-confound`

The label path feeds the same stored effective residual to the operational, sister, and anchor drift states, while the specification describes each model's assessment-time prediction. The stable-suffix guard exposes whether that implementation difference makes the proposed stimulus inconclusive without expanding this witness into a separate per-model conformance claim. The shared `TrainedStateFixture` cannot supply this precondition because its balanced signal population and label budget would replace the promise's fixed one-thousand-label, 5% phase, so the guard must remain witness-local.

**Observation (The crossing is intentionally transient)** · `obs:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-transient-crossing`

Automatic reset clears the value that exceeded $h$ before the next health report. The zero-displacement `LifecycleHealthEvent::DriftReset` is therefore the public crossing witness; sampling only `Assayer::full_health_report` can miss a correct detection, and sampling only the monotone reset counter cannot place it within the ramp.

**Observation (Calibration reset scope is inconsistent)** · `obs:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-reset-scope`

The calibration decision calls for a total reset (´dec:calibration:drift-reset´), while the specification and current label path reset only the affected regime (´alg:platt:drift-integration´). The promise requires only their common consequence: a fitted displacement past `0.1` resets at least its affected regime. Total-versus-narrow scope remains a separate decision conflict and does not block this witness.

**Observation (The event stream is bounded)** · `obs:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-event-loss`

`PlaybackBarrier::DrainHealthEvents` proves completion but discards the returned vector inside the barrier policy, so this witness must select `PlaybackBarrier::FlushLabels` and capture `World::drain_health_events` in each checkpoint. Draining every row keeps the event-to-label join deterministic; any non-zero `SystemHealthReport::health_events_dropped` invalidates the trace rather than proving that no reset occurred (´dec:health:bounded-events´).

## Acceptance · `sec:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration-acceptance`

The completed witness reports the exact tape construction, explicit configuration, stable-suffix guard, first automatic-reset label, every in-ramp calibration refit and individual log displacement, the corresponding reset event, affected post-label step counts, and a zero dropped-event count.

The `calibration_discrimination_convergence` target's `calibration_discrimination_convergence::tenfold_base_rate_shift_trips_cusum_and_resets_calibration_drift` test cites this intent, appears in the target's test index and generated integration matrix, uses the real `World`, declarative playback, declared barriers, and public health surfaces, and carries the full lint rules.

Its evidence includes the ordinary-run trace and diagnostics explaining how disabled detection, missing calibration coupling, continuous stable-regime alarms, and post-update residuals would fail the assertions; mutation execution is not part of this witness.
