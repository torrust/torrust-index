# Patient label corruption stays bounded and silent · `plan:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing`

Keeping (´claim:wellness:one-corrupted-label-in-two-hundred-costs-almost-nothing´) establishes through the public surface that sparse, distributed label corruption raises no integrity alarm, reduces rank discrimination only slightly, and reaches a stationary cost instead of accumulating across a long stream.

## What the promise says, precisely · `sec:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing-precision`

The measured horizon contains exactly ten thousand live assess–label cycles after a clean burn-in. Every contiguous block of two hundred cycles contains one deliberately inverted reported valence, giving fifty corrupted labels, while a clean control receives the same requests in the same order with every true valence intact.

Each block contains forty truly adverse and one hundred sixty truly benign outcomes. Corruption changes one benign outcome to adverse, so the rate is exactly one in two hundred and the class mix makes the discrimination cost analytically visible rather than lost at the AUC tolerance.

The stream rotates two hundred entity keys through every block. Each entity receives fifty measured labels, and the fifty corruptions land on distinct entities, so corruption can remove at most one agreement from any entity's measured history. The resulting additional concordance deficit is at most `1 / 50 = 0.02`, far inside the configured `0.25` flagging deficit after the five-label gate (´alg:monitoring:per-entity-concordance´) (´tab:config:monitoring´).

The model drift accumulators subtract the configured allowance `kappa_drift = 0.1` on every step and reset only after a strict crossing of `h = 10` (´alg:monitoring:drift-cusums´). The Sentinel alarm–outcome accumulators subtract `kappa_lab = 0.02`; placing the corrupted pulse first in each block leaves one hundred ninety-nine matching observations to return every directional accumulator to zero before the block observation (´def:monitoring:alarm-outcome-cusums´) (´tab:config:monitoring´).

Rank discrimination is the tie-corrected AUC over assessment-time predictions and eventual outcomes (´alg:monitoring:auc´). The default calibration buffer retains two thousand rows and the periodic refit runs every two hundred labels, so each two-thousand-label checkpoint in this stationary fixture contains four hundred true adverse rows, ten corrupted benign rows reported as adverse, and one thousand five hundred ninety correctly reported benign rows (´constr:platt:buffer´) (´tab:platt:refit-cadence´).

Once the keyed surface totally orders the two true classes, clean AUC is `1.0` and corrupted AUC is `405 / 410`, because the four hundred high-scored reported positives win every comparison while the ten low-scored reported positives tie the correctly benign low-scored rows. The drop is about `0.0122`, inside the promised interval from one to two hundredths, and the comparison uses the declared AUC tolerance (´tab:assayer:harness-scenario-tolerances´).

The operational model forgets with a label-indexed rate of `0.9995`, while the sister and anchor use `0.9998`; their approximate label half-lives are fourteen hundred and thirty-five hundred respectively (´tab:risk:forgetting-rates´). The combined decay factor is applied once per model per label (´dec:posterior:combined-factor´).

The leverage cap limits a single observation's precision increase along its own direction to a factor of six at the default safety factor of five (´prop:update:leverage-bound´). The implementation reduces the effective weight before mutation rather than repairing a full-weight update afterwards (´dec:posterior:leverage-before´).

The measurable promise is the conjunction: every two-thousand-label checkpoint keeps the AUC loss between `0.01` and `0.02`, no per-entity concordance flag rises, no discrimination-instability or feature-stable-drift flag rises, no corruption-only drift reset appears, and every alarm–outcome accumulator is back at zero at the block boundary. This is the patient-corruption blind spot stated by the specification rather than a claim that the labels are detected and corrected (´cav:limitation:patient-corruption´) (´cav:monitoring:integrity-scope´).

## What the code offers today · `sec:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing-code-today`

The public path is `WorldBuilder` to `World::register_sentinel`, `World::receive_report`, `World::derive_for_request`, `World::label`, `World::flush_labels`, and `Assayer::full_health_report`. `World::settle_cold_ramp_with` supplies the observation barrier needed before comparing two worlds, and `scenario_with_config` admits explicit monitoring, calibration, and model configuration without private model access.

The `LabelSpec` builder expresses the truthful or inverted valence after the assessment identifier exists. The `golden_report` and `make_cell_report` builders and their report types can build one occupied quiet region and one occupied loud region whose frozen alarm values cross the configured quiet and strong boundaries.

`SystemHealthReport` exposes per-model drift states, cached discrimination including its instability flag, per-entity concordance flags, per-Sentinel directional alarm–outcome accumulators, and the feature-stable outcome-drift flag. `HealthSummary` exposes the total drift-reset count. The cheap and comprehensive queries are intentionally distinct (´dec:health:tiered-queries´), and discrimination is computed only at refit and cached between refits (´dec:health:cached-discrimination´).

The per-entity tracker and alarm–outcome producers are complete implementation facilities rather than planned production work (´entry:health:per-entity-concordance´) (´entry:health:alarm-cusums´). Their focused tests already pin the configured concordance boundary and exact Page allowance separately (´test:unit:per-entity-concordance-honours-configured-gate-and-deficit´) (´test:crate:alarm-outcome-cusums-honour-configured-boundaries´).

The nearest public learning witness drives a scalar population through repeated assess–label cycles and computes held-out pairwise AUC (´test:integration:scalar-signal-population-split-converges-directionally´). The nearest public Sentinel witness builds outcome-keyed quiet and loud regions, drives a long labelled stream, and reads the full health report (´test:integration:association-separates-a-keyed-sentinel-from-a-hash-fed-one´). The crate-level AUC ceiling witness supplies the exact total-separation control (´test:crate:discrimination-auc-reaches-one-under-total-separation´).

The shared `Progress`, `watch_progress`, `advanced`, `finished`, and `fail_fast_on_model_owner_panic` liveness helpers turn a stalled long stream into an attributable failure. The named acknowledgement and state deadlines remain liveness guards rather than timing assertions.

The standing testing plan has no gap entry covering this promise. Its open stage-three entry does cover the reusable label tape and health assertion vocabulary that this long-stream witness consumes (´entry:assayer:harness-stage-tapes´).

## The witness · `sec:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing-witness`

The integration witness belongs in a focused corruption-tolerance integration-test binary with its own generated test index. It constructs clean and corrupted worlds from the same explicit configuration and seed, registers one Sentinel on each, ingests identical two-region reports, and settles both cold standardisation ramps on the same quiet and loud request shapes.

The setup drives an identical two-thousand-label clean burn-in through both worlds, using distinct one-use entity keys so no burn-in key reaches the concordance gate, and requires the keyed surface to have total separation with aggregate AUC present in both reports. It then captures drift-reset counters and full-health readings, so construction and calibration transients cannot be charged to the corruption phase.

The tape divides each measured run into fifty blocks of two hundred steps. Position zero is benign and is the only inverted label in the corrupted world; forty positions are adverse; the remaining positions are benign; entity assignment rotates by block so each block visits every entity once and each corruption lands on a new entity.

Each step builds matched requests with the same entity and Sentinel coordinate, derives once on each world, reports the true valence to the clean world and the scheduled valence to the corrupted world, flushes both label paths, and advances the shared progress counter. The coordinate is quiet for a true benign outcome and loud for a true adverse outcome, independent of whether the reported valence is corrupted.

At every block boundary the observation reads both full reports. It records the clean and corrupted per-entity flags, directional alarm–outcome arrays, per-model drift states, drift-reset counts since the captured baseline, feature-stable-drift flag, and discrimination-instability flag.

At every tenth block, immediately after the scheduled refit, the observation also requires both aggregate AUC values to be present. It compares the clean reading with total separation, the corrupted reading with `405 / 410`, and their difference with the inclusive `[0.01, 0.02]` semantic band using the named AUC tolerance only for comparison with the analytic readings.

The assertion joins all observations. Exactly fifty labels are inverted; every entity remains unflagged; alarm–outcome accumulators are zero at each block boundary; the corrupted world emits no drift reset beyond the clean control and raises neither integrity flag; and all five two-thousand-label AUC checkpoints remain in the same loss band rather than widening with elapsed blocks.

The fails-before is concrete. An implementation that stops subtracting the Page allowance retains and compounds one pulse per block; one that flags any per-entity disagreement marks the fifty touched entities; one that omits the drift allowance emits corruption-only resets; one that lets the recent window or aggregate buffer drift from configuration changes the checkpoint arithmetic; and one that replays or applies corrupted updates without the declared forgetting and leverage protections drives a later AUC checkpoint beyond the two-hundredths ceiling.

## What is missing · `sec:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing-missing`

**Entry (A label tape preserves online order and paired stimuli)** · `entry:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing-paired-label-tape`

Add a small `LabelTape` facility to the shared testing harness whose rows describe an already-built request, true valence, and optional reported-valence transform, and whose runner binds the live assessment identifier, submits the label, flushes, and exposes block callbacks. The facility is approximately one hundred lines plus approximately forty lines of harness tests, depends on (´entry:assayer:harness-stage-tapes´), and has no dependency on another intent lane.

The runner accepts a `Progress` handle and advances only after the paired worlds have both flushed a row. It preserves online learning by refusing to batch assessments ahead of their labels and leaves policy about block size, corruption placement, and assertions in the scenario.

**Entry (The stationary corruption fixture fixes its own arithmetic)** · `entry:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing-stationary-fixture`

Add approximately seventy lines to the corruption-tolerance integration test for the two-region Sentinel report, the two-hundred-row block constructor, rotating entity assignment, two-thousand-row clean burn-in, and the analytic `405 / 410` oracle. It depends on the paired-label-tape entry and reuses the existing report builders, `LabelSpec`, cold-ramp barrier, liveness watcher, full health reports, and default tolerance bundle.

**Entry (The public corruption witness keeps the conjunction)** · `entry:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing-integration-witness`

Add one integration test of approximately ninety lines that drives both worlds, records block and AUC checkpoints, and asserts the entire conjunction rather than splitting silence from damage. It depends on the stationary-fixture entry, changes no production surface, and mints the test label while citing the intent claim.

## Risks and open questions · `sec:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing-risks`

**Observation (The run horizon is not the AUC sample size)** · `obs:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing-window-horizon`

Ten thousand labels exercise repeated exposure and model memory, while the published aggregate AUC reads only the latest two thousand calibration rows. The stationary block composition makes every full buffer an exact miniature of the measured horizon; describing the final AUC as computed over ten thousand rows would contradict the implemented bounded buffer.

**Observation (The asymmetric flip makes the promised loss measurable)** · `obs:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing-asymmetric-flip`

Symmetric corruption at one half per cent under total separation lowers observed AUC by only about one half of one hundredth. Flipping one benign label in a twenty-per-cent-adverse block instead yields the stated one-to-two-hundredths loss without tuning model output, because the expected AUC follows from class tallies once total separation is established.

**Observation (The pulse phase is part of the fixture)** · `obs:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing-pulse-phase`

Putting the corrupted row first leaves a full block for the Page allowance to erase it before observation. Putting it last would leave a transient value near one at every checkpoint and test phase rather than accumulation; the first-position convention distinguishes persistent damage from a pulse the diagnostic is designed to absorb.

**Observation (Paired worlds begin after the asynchronous ramp)** · `obs:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing-paired-determinism`

The standardisation steward advances on its own queue, so identical construction does not by itself give two worlds identical coordinate snapshots. Settling both ramps before clean burn-in and flushing every label makes later divergence attributable to the changed valence rather than to scheduler-dependent ramp position.

**Observation (The mechanisms remain behind a behavioural boundary)** · `obs:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing-mechanism-boundary`

The public report exposes the consequences of forgetting and leverage but not the age or capped weight of each corrupted update. The witness therefore keeps the promise by bounding repeated public damage and cites the separate mathematical and record-level guarantees for the mechanism; it does not add a model-state escape hatch or claim direct observation of each bad label aging out.

**Observation (Runtime is observed, never constrained)** · `obs:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing-runtime-liveness`

Two online worlds require more than twenty-four thousand assess–label flushes including burn-in. The progress watcher and owner-panic hook make a stall fail with evidence, while no wall-clock duration enters acceptance and no timeout is treated as a performance budget.

No maintainer decision remains open: the class mix, corruption direction, entity rotation, checkpoint phase, public observations, and analytic AUC oracle make the promise deterministic without strengthening it beyond its stated interval.

## Acceptance · `sec:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing-acceptance`

The implementing lane's report names the new integration test and its citation of the intent, the tape and fixture entries it landed, and the generated test index and integration matrix updates that resolve the intent to its witness.

The report shows the exact measured and corrupted label counts, the clean and corrupted AUC at every two-thousand-label checkpoint, every per-entity flag, both directional alarm–outcome arrays, both integrity flags, and the clean-relative drift-reset counts.

The report includes green formatting, linting, Assayer package tests with all targets and features, a no-default-features spot check, release tests, documentation, and documentation tests, with any unrelated failure separated explicitly.

The report states which assertion rejects each fails-before defect. No executed mutation is required, no general-purpose tolerance is widened, no wall-clock sleep is introduced, and no private Core model state is read.
