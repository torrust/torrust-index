# Sparse reporting pays in recomputation cadence and label time · `plan:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost`

Keeping (´claim:audit:ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost´) establishes that two measurement surfaces of equal width carry different label-path costs when one reports sparsely, and that the extra work is the adaptive maintenance which preserves a usable posterior under the resulting ill-conditioning.

## What the promise says, precisely · `sec:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost-precision`

The comparison is a pair of converged worlds with the same configuration, registrations, reports, entity sequence, outcomes, eligibility, and label count. Each world registers nine Sentinels; six appear on every request, while the remaining three appear at three fixed positions in every hundred-request cycle in the sparse arm and on every request in the dense arm. Three of nine reporting on exactly three per cent of requests is the population described by (´claim:linalg:sparsely-reporting-sentinels-ill-condition-the-precision-matrix´).

Equal registration sets make the feature widths equal even when request occupancy differs. Distinct deterministic report readings and rotating coordinates keep the dense control from becoming collinear merely because every Sentinel is present.

The precondition is read from the operational model after a rebuild: the sparse arm reports the true condition number `kappa` above $10^4$, while the dense arm remains below that boundary. The true eigenvalue ratio, not the cheap diagonal ratio, is the quantity the specification names as conditioning (´def:monitoring:condition-number´), and the disclosed limitation permits it to grow under sparse evidence (´cav:limitation:conditioning´).

The cadence quantity is realised rebuild spacing in the converged measurement window. The test records label indices at which the operational model's `cholesky_recomputes` counter advances, discards the two boundary-censored gaps, and requires the median complete gap to lie from two hundred and fifty through three hundred and fifty labels. This turns “roughly every three hundred labels” into a symmetric, observable band without equating a trigger visit with a factorisation.

The cost quantity is the median elapsed label-path time per accepted label in the sparse arm divided by the same quantity in the dense arm. The inclusive acceptance band is $[4/3, 3/2]$, the direct measurable form of an increase by a third to a half. Construction, report ingestion, assessment preparation, full-health reads, and cold convergence remain outside each timed block.

The shipped policy starts from a configured interval of one thousand labels, uses a conditioning-growth factor of two, and admits no configured interval below one hundred (´tab:config:risk-model´). Conditioning growth and changes in the count of floored dimensions can force a rebuild before the counter interval, while counter visits may measure and rebuild nothing (´alg:gaussian:condition-adaptive-recompute´) (´dec:posterior:recomputation-trigger´).

The ordinary label path is quadratic in model width and the resource table states its update mix without an amortised cadence term (´tab:resource:label-cost´). The additional visits perform matrix products and, when required, a factorisation; the comparison therefore measures the complete public label path rather than deriving a time ratio from asymptotic classes (´alg:runtime:update-path´).

Correctness accompanies cost: every prepared label is accepted, every probe assessment is finite, the operational model's synchronisation residual remains at or below its width-scaled threshold after each adopted rebuild, no alarm or cascade terminus appears, and the label path remains live (´def:monitoring:synchronisation-error´). A schedule that is fast because it stopped learning does not satisfy the promise.

The standing testing plan contains no gap entry covering this promise.

## What the code offers today · `sec:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost-code-today`

The scenario and world harnesses expose `scenario_with_config`, deterministic seeds, custom `AssayerConfig`, Sentinel registration and report ingestion, request construction, assessment, label submission, and `flush_labels`. The completed skeleton already establishes these as the preferred integration path (´entry:assayer:harness-stage-skeleton´).

The public `Assayer::full_health_report` reached through `World::assayer` exposes per-model `kappa`, `diagonal_ratio`, `last_measurement_labels`, `n_recompute_effective`, `measurements`, `cholesky_recomputes`, synchronisation readings, alarms, and cascade-terminus counts. The detailed tier deliberately keeps per-model readings together, which is the tier this comparison needs (´dec:health:tiered-queries´).

The cadence implementation performs the cheap diagonal and floored-count scan on every label, distinguishes visits from rebuilds, and publishes cumulative counters. The conditioning-growth and floored-count arms rebuild directly because their question concerns movement of the precision matrix, while the counter arm lets the drift measurement decide; only real residual drift shortens the effective interval, and clean visits recover it slowly (´dec:posterior:recomputation-trigger´) (´dec:posterior:adaptive-cadence´).

The shared liveness harness supplies owner-panic failure, progress watching, and generous liveness deadlines. Those deadlines diagnose a stopped run and are not latency bounds, so they can protect a long release-mode measurement without entering its oracle.

The existing synchronisation-drift instrument already contains a configurable long-run driver, progress output, per-model health extraction, and wall measurements. Its cadence sweep establishes only that its target geometry was reached, leaving the promised sparse-versus-dense cadence and cost ratio unasserted (´test:integration:the-configured-cadence-does-not-move-the-synchronisation-error´).

The existing precision-definiteness instrument supplies the nearest long-running correctness pattern: a public-surface label schedule, periodic health checkpoints, liveness guards, and a release-only ignored test that rejects a factorisation terminus (´test:integration:a-dense-competitive-schedule-leaves-every-precision-matrix-factorisable´). The later cadence study also shows that changing the mix of measurements and rebuilds can move wall time even when reported numerical health does not move (´rep:assayer:declined-rebuild-cadence´).

No existing helper constructs equal-width dense and sparse Sentinel occupancy, stages assessments outside a timed label block, or reduces per-model rebuild events and paired timings into the two promised ratios.

## The witness · `sec:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost-witness`

The witness is one ignored release-mode integration test beside the existing synchronisation-drift instrumentation (´test:integration:the-configured-cadence-does-not-move-the-synchronisation-error´). Its module index and test documentation cite the intent, state both numeric bands, and explain why default-profile execution is excluded.

- Setup: construct dense and sparse worlds from one configuration and seed, attach nine named Sentinels with distinct deterministic reports, and settle the cold standardisation ramp before collecting training labels.
- Training stimulus: drive the same balanced deterministic outcome stream through both worlds until full health reports convergence and at least one post-convergence rebuild has supplied a true condition number for the operational model.
- Occupancy stimulus: attach all nine Sentinel coordinates in the dense arm; attach six on every sparse-arm request and the other three on exactly three positions of each hundred-request cycle, with the same coordinates whenever a sparse Sentinel is present.
- Fixture gate: require identical registration and accepted-label counts, require sparse `kappa > 10^4`, require dense `kappa < 10^4`, and reject either world if an alarm, degradation, or stopped label path makes the contrast invalid.
- Cadence observation: advance in short untimed chunks, read full health only between chunks, record each change in the operational model's recompute counter, and compute complete event-to-event gaps after convergence.
- Cost observation: prepare matched pending assessments before each timed block, start the timer immediately before label submission, stop it after `flush_labels`, alternate which arm runs first, and retain the paired sparse-to-dense per-label ratios.
- Assertion: require the complete-gap median in `[250, 350]`, the paired-time median in `[4/3, 3/2]`, several rebuild events in each arm, finite post-window assessments, bounded synchronisation residuals, zero alarms and termini, and every label accounted for.
- Diagnostic output: print each arm's true condition number, diagonal ratio, effective interval, measurement and rebuild deltas, complete gaps, elapsed time per label, ratio, and health verdict so a failed band identifies fixture, cadence, timing, or correctness.

The fails-before is an implementation that ignores conditioning and visits only at the configured counter cadence: the sparse arm remains near one rebuild per thousand labels and its timed label cost approaches the dense control, so both bands reject it. An implementation that rebuilds on every label also fails in the opposite direction, with gaps below the cadence band and cost above the promised ceiling; an implementation that gains speed by dropping labels or stopping the owner fails the accounting and liveness assertions.

## What is missing · `sec:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost-missing`

**Entry (The equal-width sparse-reporting pair)** · `entry:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost-sparse-pair`

Add a test-local fixture beside the existing synchronisation-drift instrument (´test:integration:the-configured-cadence-does-not-move-the-synchronisation-error´) that builds the two worlds, varies report readings by Sentinel, applies the hundred-request occupancy cycle, prepares matched labels, and gates convergence and conditioning from public health. The fixture is roughly one hundred to one hundred and forty lines, depends on (´entry:assayer:harness-stage-skeleton´), and has no other lane dependency; it can be shared with a later witness for (´claim:linalg:sparsely-reporting-sentinels-ill-condition-the-precision-matrix´) without making that witness a prerequisite.

**Entry (The paired cadence and cost sampler)** · `entry:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost-sampler`

Add a test-local sampler that snapshots per-model health, detects rebuild-counter changes outside timed regions, stages pending labels, times submission through the flush barrier, alternates arm order, computes medians, and prints the full evidence row. It is roughly eighty to one hundred twenty lines, depends on (´entry:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost-sparse-pair´), and reuses the existing liveness helpers without changing production or shared-harness APIs.

## Risks and open questions · `sec:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost-risks`

**Observation (Realised cadence is not the effective interval field)** · `obs:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost-cadence-semantics`

The public effective interval can take the discrete halving sequence from one thousand toward one hundred, while conditioning growth can force a rebuild before that interval and a counter visit can finish without rebuilding. Realised event spacing is the quantity that both says “every three hundred labels” and incurs factorisation cost. If the intent meant `n_recompute_effective` instead, its approximate figure does not name one attainable default-policy state; that alternative requires the intent to name the policy field and its expected discrete value before a test can keep it.

**Observation (Wall time needs a controlled execution home)** · `obs:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost-timing-home`

Elapsed time is the only current public observation that establishes a one-third-to-one-half cost increase; cumulative visit and rebuild counters establish work frequency but carry no calibrated conversion to time. Release mode, paired blocks, alternating order, medians, a quiet reference runner, and exclusion of setup and health queries limit noise. The test remains ignored in the ordinary suite and acceptance includes its explicit release-mode execution on the designated build machine.

**Observation (The numeric bands have one authority)** · `obs:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost-band-authority`

Neither the cost table nor the posterior record derives three hundred labels or the timing ratio; the intent is their only authority. If repeated controlled runs center outside either band while conditioning and correctness hold, the honest alternatives are to revise the intent to the measured operating range or to replace elapsed cost with a specified deterministic work metric and rewrite the promise around that metric. Widening the test until current behavior passes would establish neither alternative.

**Observation (Condition readings are rebuild-aged)** · `obs:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost-reading-age`

The true condition number is absent before the first rebuild and thereafter describes the matrix at the last rebuild, while the diagonal ratio is current and only a lower bound (´def:monitoring:condition-number´). The fixture begins its cadence window immediately after a rebuild and records both readings at every event, preventing a stale true reading or a fresh proxy from silently standing in for current conditioning.

**Observation (Equal width does not mean equal assessment work)** · `obs:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost-cost-isolation`

The sparse arm carries fewer occupied Sentinel slots per request even though both vectors have the same width. Timing assessment would therefore mix cheaper extraction with more expensive posterior maintenance. Preparing assessments before the timer and timing label submission through the flush barrier isolates the cost named by the promise; identical label eligibility keeps the number of updated models equal.

## Acceptance · `sec:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost-acceptance`

The implementing lane's report names the integration test, its intent citation, the two entries landed, and the coverage report that resolves the intent to the witness. It shows formatting, lint, documentation, ordinary package tests, no-default-features coverage, release tests, and the explicit ignored release-mode run with commands, outputs, exit status, and wall time.

The report carries every paired run's registered-Sentinel count, accepted-label count, operational-model true condition number and diagonal ratio, measurement and rebuild deltas, complete rebuild gaps, effective interval, elapsed label time, per-label time, and sparse-to-dense ratio. It states the median gap and timing verdict against `[250, 350]` and `[4/3, 3/2]` without substituting a configured interval or asymptotic operation count.

Acceptance requires the sparse arm above $10^4$, the dense control below it, equal width and eligible work, all labels applied, finite assessments, synchronisation within its specified bound after adopted rebuilds, zero alarms and cascade termini, and a live label owner. It also identifies the assertion that rejects fixed-cadence, every-label-rebuild, and label-dropping defects; no executed mutation is required.
