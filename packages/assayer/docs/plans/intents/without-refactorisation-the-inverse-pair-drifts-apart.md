# Keeping the Drift of an Unrefactorised Inverse Pair · `plan:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart`

This plan keeps the promise that the inverse pair drifts apart without refactorisation (´claim:linalg:without-refactorisation-the-inverse-pair-drifts-apart´) by making five public health readings over one controlled label stream establish visible, cumulative synchronisation loss.

## What the promise says, precisely · `sec:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-precision`

The model maintains a precision matrix $B_n$ and a covariance approximation $\tilde{\Sigma}_n$ through separate per-label operations, a deliberate dual representation whose floating-point disagreement is bounded by periodic exact reconstruction (´rem:gaussian:dual-tracking´), (´alg:update:sherman-morrison´).

At label count $n$, the quantity under test is the published whole synchronisation reading $e_n = \lVert B_n\tilde{\Sigma}_n-I\rVert_F$ and its operand-derived resolution $r_n$ (´def:monitoring:synchronisation-error´).

The witness takes readings at $n \in \{1{,}000,2{,}000,3{,}000,4{,}000,5{,}000\}$, so five thousand labels and five equally spaced observations are fixed properties of the case rather than values selected after a run.

“Climbs steadily” means every adjacent uncertainty interval is strictly higher than its predecessor: $e_{n+1{,}000}-r_{n+1{,}000} > e_n+r_n$ at all four transitions. This establishes resolvable growth at every checkpoint and excludes both a plateau hidden by multiplication error and one late jump standing in for accumulation.

The fixture keeps the replenishment-floor contribution at measurement resolution and asserts $|e_n-d_n| \le r_n$ for the reported arithmetic residual $d_n$. The growing whole reading is therefore evidence about jointly lossy inverse maintenance rather than the conservative prior component that the monitor reports separately (´req:gaussian:prior-replenishment-floor´), (´def:monitoring:synchronisation-error´).

The specification's abnormal threshold is $10^{-6}p$, with equality quiet and a strict crossing active (´def:config:synchronisation-threshold´). The test raises the configured coefficient to $1.0$ per dimension solely to turn the five counter visits into measurement-only visits; that suppression control is not a substitute threshold for the growth assertion.

Exact refactorisation is suppressed only when all five visits report measurements, zero Cholesky recomputes, zero alarms, no after-rebuild reading, no adoption verdict, the configured one-thousand-label interval, and no change in the count of dimensions at the replenishment floor. This accounts for the counter, conditioning, and floor-change paths that can initiate a visit (´dec:posterior:recomputation-trigger´), (´alg:gaussian:condition-adaptive-recompute´).

The “individually sound” half of the promise rests on the exact update identities, including exactness of each observation relative to the posterior it receives (´inv:guarantee:per-observation-exactness´). The new test does not duplicate those local algebra checks; it observes the loss produced by composing the maintained pair for five thousand labels.

## What the code offers today · `sec:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-code-today`

The public path is `World::builder(AssayerConfig)`, a request-scoped signal schema and declared channel, `World::derive_for_request`, `World::label`, the observation and label flushes, and `World::assayer().health_report_full()`. It exercises the host API and model-owner queue rather than reaching into `BayesianLinearModel`.

The completed world harness supplies `World`, `WorldBuilder`, the fixed virtual clock, seeded request construction, deterministic flushes, and the raw Assayer escape hatch; its shared fixtures supply clipped scalar declarations and benign and adverse labels (´entry:assayer:harness-stage-skeleton´).

The shared liveness support supplies `fail_fast_on_model_owner_panic`, `watch_progress`, `advanced`, and `finished`; its acknowledgement and state deadlines make a stalled owner or health query fail rather than wait indefinitely (´entry:assayer:harness-stage-skeleton´).

The public configuration already exposes `cholesky.n_recompute`, `cholesky.kappa_growth_factor`, the model forgetting rates, and `monitoring.sync_error_threshold_per_dimension`. A one-thousand-label interval schedules the five measurements, a maximum finite conditioning-growth factor prevents that cheap arm from pre-empting the schedule, forgetting set to one prevents replenishment, and the raised synchronisation coefficient prevents a counter visit from rebuilding (´def:config:synchronisation-threshold´), (´dec:posterior:recomputation-trigger´).

The Bayesian model update performs the decay, replenishment clamp, precision rank-one update, covariance Sherman–Morrison update, and label-count increment separately (´alg:update:sherman-morrison´). The recomputation monitor measures the full product, resets the visit counter even when no rebuild occurs, and records disjoint measurement, recompute, and alarm counters in the form required by the monitoring algorithm (´alg:gaussian:synchronisation-monitor´), (´dec:posterior:adaptive-cadence´).

The detailed health tier publishes each model's whole reading, prior-induced component, residual, resolution verdict, effective interval, floor count, measurement count, rebuild count, alarm count, last post-rebuild reading, and adoption verdict (´dec:health:tiered-queries´). This is enough to observe both drift and the absence of repair without exporting either matrix.

The closest synchronisation-drift integration instrument already selects these fields after driving public labels, but its cadence comparison only proves that a conditioning trigger made the configured cadence inert and remains ignored (´test:integration:the-configured-cadence-does-not-move-the-synchronisation-error´). It provides field-selection precedent, not this witness.

The nearest direct-model probes show that the model remains finite past one recomputation window and that a deterministic near-collinear stream accumulates drift which a fresh inverse repairs (´test:crate:n1-window-safety´), (´test:crate:a-drifted-covariance-is-repaired-and-the-rebuild-adopted´). The public witness retains that useful geometry while refusing direct matrix access.

The standing testing plan has no gap entry covering this promise. Its generic precision coefficient remains the specification's abnormality ceiling, not an accumulated-growth tolerance (´tab:assayer:harness-scenario-tolerances´).

## The witness · `sec:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-witness`

The test joins the synchronisation-drift integration module, whose existing purpose and detailed health reader already surround the promise (´test:integration:the-configured-cadence-does-not-move-the-synchronisation-error´). Its module index gains the new guard and distinguishes it from the ignored exploratory sweeps.

Setup builds one fixed-clock world with one channel and twenty-four request-scoped clipped scalar signals. The existing sixteen structural coordinates plus those signals give a fixed operational width of forty, following the layout behavior already held by the builder probe (´test:crate:cold-start-dimension-with-signals´).

The model uses operational and sister forgetting rates of $1.0$, `n_recompute = 1_000`, `kappa_growth_factor = f64::MAX`, and `sync_error_threshold_per_dimension = 1.0`. The fixed width stays below any need for identity registration, outcome-axis growth, persistence, or maintenance traffic.

The stimulus is a deterministic small-state recurrence adapted from the direct-model repair probe: three signal coordinates sit near $0.75$, the remaining twenty-one sit near $0.25$, and independent positive jitter stays within the declared clips. One entity receives alternating benign and adverse labels, producing a strongly correlated but nonconstant trajectory without scheduler-selected randomness.

Requests are derived in batches of one hundred, their observations are flushed, their `LabelSpec` values are submitted in derivation order, and labels are flushed once per batch. The liveness counter advances on each completed batch, while the virtual clock remains fixed so time-indexed decay cannot perturb the label-indexed trajectory.

After every ten batches, the test obtains the detailed report, selects the operational model, and copies the five quantities needed after the report guard is released: whole reading, prior component, residual, resolution, and recomputation telemetry.

Each checkpoint asserts the expected width and label count, finite readings, a prior component no greater than resolution, whole/residual agreement within resolution, a residual above resolution, `last_measurement_at_resolution == false`, an unchanged effective interval, the expected ordinal measurement count, no floor dimensions, no shortening, no recompute, no alarm, and absent rebuild-result fields.

After the fifth checkpoint, the test applies the four disjoint-interval comparisons to the whole readings. Diagnostics print all five tuples as label count, whole reading, residual, prior component, and resolution, so a failure exposes the trajectory rather than only the failed pair.

A deliberately broken implementation that reconstructs the covariance on every label leaves all five readings at resolution or flat and fails the growth comparisons. An implementation that rebuilds at a visit fails the zero-recompute and absent-result assertions even if the next reading is small; a stale or diagonal-only monitor fails to reproduce the off-diagonal growth; a replenishment leak fails the floor and prior-component isolation before it can masquerade as arithmetic drift.

## What is missing · `sec:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-missing`

No production hook or shared harness capability blocks the test. The missing material is local to the integration test and composes the completed skeleton (´entry:assayer:harness-stage-skeleton´).

**Entry (The fixed-width inverse-drift stream)** · `entry:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-fixed-width-stream`

Add approximately seventy lines to the synchronisation-drift integration module for the twenty-four-signal schema, fixed configuration, deterministic recurrence, request construction, batching, and liveness guard (´test:integration:the-configured-cadence-does-not-move-the-synchronisation-error´). This entry depends on the existing `World`, signal builders, `LabelSpec`, flush methods, and liveness helpers; it has no sibling-lane dependency.

**Entry (The operational precision sampler)** · `entry:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-operational-sampler`

Add approximately thirty-five lines beside the existing health reader to select the operational model and return an owned checkpoint containing the five readings and suppression counters. This entry depends on the fixed-width stream only for its expected model identity and requires no health-surface change.

**Entry (The five-checkpoint guard)** · `entry:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-five-checkpoint-guard`

Add approximately fifty lines for the test head, the five samples, per-checkpoint assertions, four resolution-separated growth assertions, and trajectory diagnostics. This entry depends on the stream and sampler entries and mints the integration-test label only when the implementation lands.

## Risks and open questions · `sec:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-risks`

**Observation (The whole reading must denote arithmetic loss)** · `obs:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-whole-reading-denotation`

The promise names the whole Frobenius reading, while an active replenishment clamp can make that reading grow for a conservative reason. Forgetting of one and the paired prior/residual assertions make the whole reading and arithmetic residual observationally equal; a wide, floor-dominated geometry is unsuitable even if its whole reading rises (´test:crate:the-studys-reading-is-the-prior-and-its-residual-is-rounding´).

**Observation (Suppression is a reported condition)** · `obs:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-suppression-is-reported`

A raised monitoring threshold alone does not suppress conditioning-growth, floor-count-change, or degenerate triggers because those paths force a rebuild after measuring. Maximum finite conditioning growth, constant zero floor count, finite-state checks, and the published recompute counters make suppression an assertion rather than an assumption (´dec:posterior:recomputation-trigger´).

**Observation (Resolution is the only numerical allowance)** · `obs:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-resolution-only`

The operand-derived $r_n$ changes with the conditioned matrices and is the only justified uncertainty for comparing readings (´def:monitoring:synchronisation-error´). The generic harness ceiling and the deliberately raised monitoring threshold classify health or control the schedule; neither can be widened to make an unresolvable growth sequence pass.

**Observation (The deterministic trajectory remains a fixture choice)** · `obs:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-trajectory-choice`

The exact twenty-four-signal recurrence is grounded in the existing direct-model repair geometry, but its public feature expansion adds structural coordinates and leverage bounding. If the implementation report shows an adjacent pair whose resolution intervals overlap, the fixture varies the fixed signal count or jitter scale while retaining five thousand labels and the four strict comparisons; weakening the comparisons would stop keeping “climbs steadily.”

**Observation (Queue order is closed at each checkpoint)** · `obs:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-queue-order`

One driving thread, fixed request order, observation flushes before labels, label flushes before health reads, a fixed virtual clock, and no maintenance or identity work leave no intended source of nondeterminism. Liveness deadlines detect a stopped owner but do not impose timing assertions on a progressing run.

**Observation (The run remains a default guard)** · `obs:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-default-runtime`

Five thousand labels at fixed width forty replace the ignored instrument's expanding competitive geometry. The test remains unignored; an implementation whose measured wall time cannot fit the package's normal integration gate requires a smaller fixed width that still clears all four resolution-separated comparisons, not a reduced label count.

## Acceptance · `sec:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart-acceptance`

The implementing lane adds one indexed, unignored integration guard whose documentation cites the promise and whose body drives only public host operations.

Its report shows the five checkpoint tuples, proves that every adjacent whole-reading interval moved upward, and shows five measurements with zero recomputes, zero alarms, zero floor dimensions, no shortenings, and no rebuild-result fields.

The report confirms that the prior component stayed within each reading's resolution and that each whole reading agreed with its residual within that resolution, so the asserted rise belongs to incremental inverse maintenance.

The targeted integration binary passes repeatedly with the same tuples, the package's ordinary all-feature and no-default-feature test gates pass, release mode passes, documentation and documentation tests pass, and the new guard is not ignored.

No production visibility, matrix export, sleep, wall-clock assertion, new shared tolerance, or mutation-only test hook enters with the witness.
