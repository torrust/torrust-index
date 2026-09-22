# Keeping the Drift of an Unrefactorised Inverse Pair · `plan:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart`

This plan keeps the promise that the inverse pair drifts apart without refactorisation (´claim:linalg:without-refactorisation-the-inverse-pair-drifts-apart´) by making five readings of the detailed health tier, taken over one declarative label stream, establish visible and cumulative synchronisation loss.

## What the promise says, precisely · `sec:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-precision`

The model maintains a precision matrix $B_n$ and a covariance approximation $\tilde{\Sigma}_n$ through separate per-label operations, a deliberate dual representation whose floating-point disagreement the periodic recomputation schedule clears rather than prevents (´rem:gaussian:dual-tracking´), (´alg:update:sherman-morrison´).

At label count $n$, the quantity under test is the published whole synchronisation reading $e_n = \lVert B_n\tilde{\Sigma}_n-I\rVert_F$ and its operand-derived resolution $r_n$ (´def:monitoring:synchronisation-error´).

The witness takes readings at $n \in \{1{,}000,2{,}000,3{,}000,4{,}000,5{,}000\}$, so five thousand labels and five equally spaced observations are fixed properties of the case rather than values selected after a run.

“Climbs steadily” means every adjacent uncertainty interval is strictly higher than its predecessor: $e_{n+1{,}000}-r_{n+1{,}000} > e_n+r_n$ at all four transitions. The resolution is the absolute uncertainty the reading carries, read off the operands rather than chosen, so the comparison separates intervals the specification derives rather than figures a tolerance holds apart (´def:monitoring:synchronisation-error´). This establishes resolvable growth at every checkpoint and excludes both a plateau hidden by multiplication error and one late jump standing in for accumulation.

The fixture keeps the replenishment-floor contribution at measurement resolution and asserts $|e_n-d_n| \le r_n$ for the reported arithmetic residual $d_n$, which is the whole reading less that contribution, floored at zero. The growing whole reading is therefore evidence about jointly lossy inverse maintenance rather than the conservative prior component that the monitor reports separately (´req:gaussian:prior-replenishment-floor´), (´def:monitoring:synchronisation-error´).

The specification's abnormal threshold is $10^{-6}p$, with equality quiet and the smallest represented crossing active (´def:config:synchronisation-threshold´). The test raises the configured coefficient to $1.0$ per dimension solely to turn the five counter visits into measurement-only visits; at the case's fixed width of forty that puts the threshold at forty, and a visit ends at its measurement only while both components of the reading stand under it (´alg:gaussian:synchronisation-monitor´). That suppression control is not a substitute threshold for the growth assertion, and the band it leaves is the band the case has to land in: growth resolvable at all four transitions, both components still under forty at the fifth visit.

Exact refactorisation is suppressed only when all five visits report measurements, zero Cholesky recomputes, zero alarms, no after-rebuild reading, no adoption verdict, the configured one-thousand-label interval, and no change in the count of dimensions at the replenishment floor. This accounts for every path that can initiate a visit: the counter arm, which defers to the measurement, and the conditioning-growth, floored-count and degenerate arms, which proceed to a rebuild whatever the measurement says (´dec:posterior:recomputation-trigger´), (´alg:gaussian:condition-adaptive-recompute´).

The “individually sound” half of the promise rests on the exact update identities, including exactness of each observation relative to the posterior it receives (´inv:guarantee:per-observation-exactness´). The new test does not duplicate those local algebra checks; it observes the loss produced by composing the maintained pair for five thousand labels.

## What the code offers today · `sec:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-code-today`

The public path is `World::builder`, a request-scoped signal schema and declared channel through `WorldBuilder::signal_schema` and `WorldBuilder::channel`, `World::settle_cold_ramp_with`, `World::derive_for_request`, `World::label`, and `Assayer::full_health_report` reached through `World::assayer`. It exercises the host API and model-owner queue rather than reaching into `BayesianLinearModel` (´tab:assayer:harness-implementation-library-roster´), (´entry:assayer:harness-stage-skeleton´).

Waiting is a named barrier and never a sleep or a poll. `World::flush_labels` completes every command and label through publication and states that it leaves the cold-ramp queue alone, `World::flush_observations` is that queue's own barrier, and `World::flush_identity_maintenance` completes identity maintenance; a reading may rely on the barrier it crossed and infer nothing about an excluded queue (´cor:concurrency:harness-barriers´), (´dec:harness:no-ad-hoc-waits´). Time moves only forward, through `World::advance` and `World::travel_to`, which cross the barriers their own movement makes due (´cor:clock:harness-control´).

Long stimuli are declarative. A subject implements `PlaybackRow` for its own row type and the `playback` runner owns barrier cadence, checkpoint placement, bounded materialisation through `PlaybackBatchSize`, and progress; `PlaybackBarrierPolicy` names the ordered `PlaybackBarrier` set crossed at every row boundary, `PlaybackCheckpoint::after_row` declares one callback at one zero-based row index, and a batch is an allocation boundary rather than a shared view (´dec:harness:declarative-playback´), (´cor:ordering:tape-batches´).

The probe projections `PublishedModelBlock`, `PublishedSlotMoments` and `PendingEntryView` cross a barrier and perform one published load, but published precision is deliberately outside that surface and none of them carries synchronisation telemetry (´dec:harness:probe-contract´), (´cav:retention:probe-boundary´). The quantities this witness needs are host-visible on the detailed health tier instead, so it reads the report rather than adding a projection that would have to breach that boundary (´dec:health:tiered-queries´).

`World::observed_runtime_layout` returns an owned `RuntimeLayout` of every published block width after the publication barrier, and `WorldBuilder::expected_runtime_layout` turns a declared width into a construction guard: the build reports the mismatch instead of producing a world whose geometry a later assertion would have to discover (´dec:harness:guarded-fixtures´).

The oracle tier supplies `dimension_width` over `DimensionBlocks`, which evaluates the dimension formula block by block and declares `OracleProvenance::SpecificationFormula`, so an expected width is a second computation rather than a literal (´dec:harness:oracle-tier´), (´tab:feature:dimension-formula´). No oracle predicts an accumulating rounding trajectory, which is why the growth is compared against the reading's own resolution and not against a curve.

The shared liveness support supplies `fail_fast_on_model_owner_panic`, `watch_progress`, `advanced`, `finished` and `Progress`, and remains the only home for bounded waiting; `PlaybackProgress` names the first incomplete row of a stalled run and is liveness evidence rather than latency evidence (´tab:assayer:harness-implementation-library-roster´), (´cav:harness:progress-not-latency´).

The public configuration already exposes `CholeskyConfig::n_recompute`, `CholeskyConfig::kappa_growth_factor`, `ModelConfig::gamma_opr`, `ModelConfig::gamma_inh`, and `MonitoringConfig::sync_error_threshold_per_dimension`. A one-thousand-label interval schedules the five measurements, a maximum finite conditioning-growth factor prevents that cheap arm from pre-empting the schedule, forgetting set to one prevents replenishment, and the raised synchronisation coefficient prevents a counter visit from rebuilding (´def:config:synchronisation-threshold´), (´dec:posterior:recomputation-trigger´).

The Bayesian model update performs the decay, replenishment clamp, precision rank-one update, covariance Sherman–Morrison update, and label-count increment separately (´alg:update:sherman-morrison´). The recomputation monitor measures the full product, resets the visit counter even when no rebuild occurs, and records disjoint measurement, recompute, and alarm counters in the form required by the monitoring algorithm (´alg:gaussian:synchronisation-monitor´), (´dec:posterior:adaptive-cadence´).

`PrecisionHealthDetail` publishes, per model, the whole reading, the prior-induced component, the residual, the resolution, the at-resolution flag, the labels absorbed when the last visit measured, the effective interval, the floor count, the measurement, recompute, alarm and shortening counters, and the post-rebuild reading, verdict and adoption fields (´dec:health:tiered-queries´). This is enough to observe both drift and the absence of repair without exporting either matrix.

The synchronisation-drift integration target now carries five instruments, all ignored, whose shared reader already selects most of those fields after driving public labels and whose own stated position is that none of them is a guard (´test:integration:the-configured-cadence-does-not-move-the-synchronisation-error´). That reader omits the resolution, which this witness compares against, selects every model rather than one, and drives its stimulus through a hand-written loop rather than playback rows. It provides field-selection precedent, not this witness.

The nearest direct-model probes show that the model remains finite past one recomputation window, and that a deterministic near-collinear stream over four thousand labels at width twenty-four accumulates drift which a fresh inverse repairs by a factor near twenty-five while both readings stay under the width-scaled threshold (´test:crate:n1-window-safety´), (´test:crate:a-drifted-covariance-is-repaired-and-the-rebuild-adopted´). The second is the evidence that this geometry produces drift large enough to read and small enough to leave the schedule suppressed; the public witness retains it while refusing direct matrix access.

The standing testing plan has no gap entry covering this promise, and its generic precision coefficient remains the specification's abnormality ceiling rather than an accumulated-growth tolerance (´tab:assayer:harness-scenario-tolerances´).

## The witness · `sec:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-witness`

The test joins the synchronisation-drift integration target, whose subject and health reader already surround the promise (´test:integration:the-configured-cadence-does-not-move-the-synchronisation-error´). It is the first guard in that target, so the target's index gains it and the target's own prose stops saying that nothing in it is a guard.

Setup builds one world through `World::builder` with one channel, a declared seed, and twenty-four request-persistence `SignalShape::Scalar` declarations. The layout handed to `WorldBuilder::expected_runtime_layout` comes from `dimension_width` over a `DimensionBlocks` carrying those twenty-four signals and nothing else, which is forty: the structural sixteen plus one coordinate per declaration, whatever each declaration's shape (´test:crate:cold-start-dimension-with-signals´), (´dec:harness:oracle-tier´). A width the builder declares is a width the build refuses to differ from, so no checkpoint re-asserts it.

The model uses `ModelConfig::gamma_opr` and `ModelConfig::gamma_inh` of $1.0$, `CholeskyConfig::n_recompute` of one thousand, `CholeskyConfig::kappa_growth_factor` of `f64::MAX`, and `MonitoringConfig::sync_error_threshold_per_dimension` of $1.0$. The fixed width stays below any need for identity registration, outcome-axis growth, persistence, or maintenance traffic, so identity maintenance is never a queue this case depends on.

`World::settle_cold_ramp_with` drives the standardisation ramp into service before the measured stream begins. Its loop crosses the observation barrier on every iteration and exits on a published phase rather than on elapsed time, so the five readings are taken against one settled coordinate system rather than against one still moving under them.

The stimulus is a deterministic small-state recurrence adapted from the direct-model repair probe: three signal coordinates sit near $0.75$, the remaining twenty-one sit near $0.25$, and independent positive jitter stays within the declared clips. One entity receives alternating benign and adverse labels built through `LabelSpec`, producing a strongly correlated but nonconstant trajectory without scheduler-selected randomness.

The stream is fifty playback rows of one hundred labels each. A row derives its hundred requests through `World::derive_for_request` and submits their labels in derivation order, and chooses no settling cadence of its own: the `PlaybackBarrierPolicy` crossing `PlaybackBarrier::FlushLabels` at every row boundary is what completes the model-owner queue (´dec:harness:declarative-playback´). The policy names that queue alone, because the label queue is the only one each checkpoint's reading depends on and the ramp was settled before the stream, so nothing is inferred across a boundary the policy does not cross (´cor:concurrency:harness-barriers´). Neither `World::advance` nor `World::travel_to` is called, so the virtual clock stands still and no time-indexed decay can perturb the label-indexed trajectory.

Five `PlaybackCheckpoint::after_row` callbacks stand at rows nine, nineteen, twenty-nine, thirty-nine and forty-nine, which are the thousand-label boundaries the cadence visits. Each takes one `Assayer::full_health_report`, selects `ModelId::Operational` from its precision map, and copies the five readings and the recomputation telemetry into an owned checkpoint before the report is dropped.

Each checkpoint asserts finite readings, labels absorbed at the last measurement equal to its own thousand-label boundary, a prior component no greater than resolution, whole and residual agreement within resolution, a residual above resolution, `last_measurement_at_resolution` false, an unchanged effective interval, the expected ordinal measurement count, no floor dimensions, no shortening, no recompute, no alarm, and absent rebuild-result fields.

After playback returns, the test applies the four disjoint-interval comparisons to the five whole readings. Diagnostics print all five tuples as label count, whole reading, residual, prior component and resolution, so a failure exposes the trajectory rather than only the failed pair, and a run that stops early leaves `PlaybackProgress` naming the row it stopped on.

A deliberately broken implementation that reconstructs the covariance on every label leaves all five readings at resolution or flat and fails the growth comparisons. An implementation that rebuilds at a visit fails the zero-recompute and absent-result assertions even if the next reading is small; a stale or diagonal-only monitor fails to reproduce the off-diagonal growth; a replenishment leak fails the floor and prior-component isolation before it can masquerade as arithmetic drift.

## What is missing · `sec:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-missing`

No production hook or shared harness capability blocks the test. The barriers, the playback runner and its checkpoints, the width oracle, the construction guard and the cold-ramp settle are all declared, and the missing material is local to the integration target (´tab:assayer:harness-implementation-library-roster´).

**Entry (The fixed-width inverse-drift stream)** · `entry:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-fixed-width-stream`

Add approximately eighty lines to the synchronisation-drift integration target for the twenty-four-signal schema, the oracle-derived declared layout, the fixed configuration, the cold-ramp settle, the deterministic recurrence, and one row type whose `PlaybackRow` implementation derives a hundred requests and submits their labels without settling (´test:integration:the-configured-cadence-does-not-move-the-synchronisation-error´). This entry depends on the existing `World` verbs, `SignalDeclaration`, `LabelSpec`, `dimension_width` and the `playback` runner; it has no sibling-lane dependency.

**Entry (The operational precision sampler)** · `entry:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-operational-sampler`

Add approximately thirty-five lines for a sampler that selects `ModelId::Operational` from one `Assayer::full_health_report` and returns an owned checkpoint carrying the five readings and the suppression counters. The target's existing reader selects neither the resolution nor a single model, so this is new rather than reusable; it requires no health-surface change and depends on the stream entry only for the model identity it expects.

**Entry (The five-checkpoint guard)** · `entry:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-five-checkpoint-guard`

Add approximately fifty-five lines for the test head, the five `PlaybackCheckpoint::after_row` declarations, the per-checkpoint assertions, the four resolution-separated growth comparisons, and the trajectory diagnostics. This entry depends on the stream and sampler entries and mints the integration-test label only when the implementation lands.

## Risks and open questions · `sec:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-risks`

**Observation (The whole reading must denote arithmetic loss)** · `obs:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-whole-reading-denotation`

The promise names the whole Frobenius reading, while an active replenishment clamp can make that reading grow for a conservative reason. A forgetting rate of one leaves the precision diagonal undecayed, so the clamp never engages, the floored count never leaves zero and the prior-induced component is zero by construction rather than merely small; the paired prior and residual assertions then make the whole reading and the arithmetic residual observationally equal. A wide, floor-dominated geometry is unsuitable even if its whole reading rises (´test:crate:the-studys-reading-is-the-prior-and-its-residual-is-rounding´).

**Observation (Suppression is a reported condition)** · `obs:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-suppression-is-reported`

A raised monitoring threshold alone does not suppress the conditioning-growth, floored-count or degenerate arms, because each of those proceeds to a rebuild after measuring. Maximum finite conditioning growth, a constant zero floor count, finite-state checks and the published recompute counters make suppression an assertion rather than an assumption, and a model that has never rebuilt has no record for the two relative arms to fire against at all (´dec:posterior:recomputation-trigger´). The counter arm's suppression is the one with a ceiling: it holds while both components stand under the raised threshold, so a case whose fifth reading reached forty would rebuild and fail its own suppression assertions rather than pass quietly.

**Observation (Resolution is the only numerical allowance)** · `obs:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-resolution-only`

The operand-derived $r_n$ changes with the conditioned matrices and is the only justified uncertainty for comparing readings (´def:monitoring:synchronisation-error´). The generic harness ceiling and the deliberately raised monitoring threshold classify health or control the schedule; neither can be widened to make an unresolvable growth sequence pass (´tab:assayer:harness-scenario-tolerances´).

**Observation (The deterministic trajectory remains a fixture choice)** · `obs:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-trajectory-choice`

The exact twenty-four-signal recurrence is grounded in the existing direct-model repair geometry, but the public path standardises each declared signal before it reaches the feature vector and adds structural coordinates and leverage bounding, so the declared values fix the trajectory's shape rather than the coordinates the model actually sees. If the implementation report shows an adjacent pair whose resolution intervals overlap, the fixture varies the fixed signal count or jitter scale while retaining five thousand labels and the four strict comparisons; weakening the comparisons would stop keeping “climbs steadily”.

**Observation (Queue order is closed at each checkpoint)** · `obs:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-queue-order`

One driving thread, fixed request order, a label barrier at every row boundary, a ramp settled before the stream, a stationary virtual clock and no maintenance or identity work leave no intended source of nondeterminism. The runner owns the cadence, so no part of the stimulus decides when state is settled, and each checkpoint runs once at its declared row boundary after those barriers have been crossed (´dec:harness:declarative-playback´). Liveness support detects a stopped owner but imposes no timing assertion on a progressing run (´cav:harness:progress-not-latency´).

**Observation (The run remains a default guard)** · `obs:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-default-runtime`

Five thousand labels at fixed width forty replace the ignored instruments' expanding competitive geometry, and the hundred-label row puts the barrier count at fifty rather than five thousand. The test remains unignored; an implementation whose measured wall time cannot fit the package's normal integration gate requires a smaller fixed width that still clears all four resolution-separated comparisons, not a reduced label count.

## Acceptance · `sec:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-acceptance`

The implementing lane adds one indexed, unignored integration guard to the `synchronisation_drift` target, named `without_refactorisation_the_inverse_pair_drifts_apart`, whose documentation cites the promise and whose body drives only public host operations.

Its report shows the five checkpoint tuples, proves that every adjacent whole-reading interval moved upward, and shows five measurements with zero recomputes, zero alarms, zero floor dimensions, no shortenings, and no rebuild-result fields.

The report confirms that the prior component stayed within each reading's resolution and that each whole reading agreed with its residual within that resolution, so the asserted rise belongs to incremental inverse maintenance.

The targeted integration binary passes repeatedly with the same tuples, the package's ordinary all-feature and no-default-feature test gates pass, release mode passes, documentation and documentation tests pass, and the new guard is not ignored.

No production visibility, matrix export, sleep, poll, local deadline, wall-clock assertion, new shared tolerance, or mutation-only test hook enters with the witness.
