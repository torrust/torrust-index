# A long benign stream converges without making the next adverse label a shock · `plan:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk`

Keeping (´claim:bayes:a-stream-with-no-adverse-outcome-converges-to-near-zero-risk´) establishes that sustained non-positive evidence drives class-rate estimates and predictions into a declared near-zero regime, reaches the positive importance ceiling without collapsing the posterior, and still bounds the next adverse update by its leverage.

## What the promise says, precisely · `sec:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk-promise`

For a segment of $N=5{,}000$ eligible labels with $v_i\leq0$, both positive-rate trackers follow $P_{+,N}=\gamma^N P_{+,0}$ because the tracker moves before the label's own weight is derived (´alg:weighting:tracker-update´). The operational stream uses $\gamma_\text{opr}$ and the eligible stream uses $\gamma_\text{inh}$, and both rates are configuration values tied to their models' forgetting horizons (´def:weighting:initial-value´) and (´tab:config:risk-model´).

The positive importance weight is $w_+=\min(1/(2P_++\varepsilon),100)$, so the configured ceiling of one hundred binds once the updated tracker lies below approximately $0.005$ (´def:weighting:balancing-weights´). The witness reads both tracker values immediately before the injected adverse label, recomputes the values after that label's tracker update, and requires the positive weight requested by both streams to equal the ceiling.

Near-zero risk means the public `RiskBasis.p_bad`, its operational component, and its sister component lie below a resolved probability threshold $\delta_\text{risk}$ on a held-out benign probe, while the spread across a resolved terminal window lies below $\delta_\text{settle}$. These quantities are the calibrated sigmoid images of the effective raw estimate (´def:risk:probability´) and are distinct from $P_+$.

Non-degeneration means every probability and raw basis field is finite, every probability remains strictly inside $(0,1)$, uncertainty remains finite and positive, calibration scale remains finite and positive, each detailed precision reading has a finite positive least eigenvalue, no precision alarm or cascade terminus appears, and synchronisation residual stays within the dimension-scaled harness budget. These checks exercise the public basis (´schema:risk:basis´), the maintained spectral floor (´dec:posterior:spectral-floor´), and the specified synchronisation diagnostic (´def:monitoring:synchronisation-error´).

For the first $v>0$ label after the segment, let $h=\hat\phi^{\mathsf T}\Sigma\hat\phi$, $w_*=\min(w_\text{target},100)$, and $L=c/(h+\varepsilon)$. The promised protection is the observable strict relation $L<w_*$ and effective weight $w_\text{eff}=L$, followed by a finite positive movement in the probe's adverse-risk estimate rather than the movement from the full importance weight; this is the minimum selected by the update algorithm (´alg:update:sherman-morrison´) and the precision-ratio protection of the leverage proposition (´prop:update:leverage-bound´).

The ordering is load-bearing: leverage is measured against the pre-update covariance and bounded before mutation, never inferred from a repaired posterior (´dec:posterior:leverage-before´). The two caps protect different hazards, so an importance-ceiling hit alone is not evidence that leverage fired (´disc:weighting:leverage-interaction´).

The standing testing-plan debt is the deeper convergence gap (´entry:assayer:gap-convergence-depth´), which names convergence under class imbalance and the need for the long streaming tapes and state inspection of (´entry:assayer:harness-stage-tapes´).

## What the code offers today · `sec:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk-code`

The configured-scenario helper exposes `scenario_with_config`, and the world harness exposes configured `World` construction, request assessment, label submission, cold-ramp settlement, and separate observation and label barriers. The signal fixtures supply the scalar-and-binary request schema needed to keep one benign direction trained while leaving one injection direction comparatively novel; these pieces are part of the delivered harness skeleton (´entry:assayer:harness-stage-skeleton´).

The label fixture supplies explicit benign and adverse labels, while the liveness harness supplies the progress watcher needed around a five-thousand-cycle asynchronous path. The scenario tolerance budget, including the per-dimension synchronisation coefficient, is already declared by (´tab:assayer:harness-scenario-tolerances´).

`Assayer::health_summary` publishes both $P_+$ trackers, calibration scales, label counts, floor share, synchronisation readings, cascade status, and label-path status. `Assayer::full_health_report` adds per-model least eigenvalue, spectral floor, floor share, alarms, rebuild verdicts, and importance-ceiling binding fractions and gradient balances; the latter fields implement the completed ceiling-health entry (´entry:health:importance-ceiling´).

The public risk basis publishes the point probability, operational and sister probabilities, uncertainty, raw estimate, raw uncertainty, calibration scale, borrowed share, and regime calibration counts (´schema:risk:basis´). A test can therefore keep prediction convergence and posterior health entirely through public results after deterministic barriers.

The model update helper computes both the effective weight and the exact `leverage_bound_fired` predicate (´alg:update:sherman-morrison´), and the label path computes $h$ for each model before mutation as part of the ordered update path (´alg:runtime:update-path´). That evidence is discarded after the update: acknowledgements, health reports, and the testing harness expose neither $h$, the three candidate caps, effective weight, nor a per-label firing record.

The nearest integration checks stop short on opposite sides. The repeated benign cycle proves only that two hundred identical benign updates remain finite (´test:integration:repeated-identical-benign-cycle-stays-finite´), while the scalar population split proves directional separation after a balanced stream rather than near-zero convergence under starvation (´test:integration:scalar-signal-population-split-converges-directionally´).

`Assayer::pre_seed` reaches the production label path and is suitable for tracker arithmetic, but its synthetic pending assessments use prior-level risk and zero-featured model inputs. It cannot establish learned request probabilities or create the high-leverage direction that the adverse injection needs.

## The witness · `sec:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk-witness`

The witness is one integration test beside the existing public-path learning test (´test:integration:scalar-signal-population-split-converges-directionally´), which already establishes the module's convergence scope. Its documentation cites the intent and its module index states the benign segment, terminal prediction oracle, posterior-health oracle, and capped injection.

- Setup: build one seeded world through `scenario_with_config`, register the scalar-and-binary signal schema, hold the virtual clock fixed, settle the cold standardisation ramp with both request shapes, and run a deterministic balanced calibration prefix containing the minimum positive and negative support required by (´req:platt:minimum-samples´). The prefix is outside the five-thousand-label segment, ends with both tracker values at or below one half, and proves that calibration has refitted rather than relying on the initial scale of (´def:platt:initial-value´).

- Configuration: the fixed-length branch uses $\gamma_\text{opr}=0.998$, $\gamma_\text{inh}=0.999$, $P_{+,0}=0.5$, $c=5$, and $w_\text{ceiling}=100$. These admissible values make even a one-half segment-start rate fall below $0.005$ in both streams after five thousand benign labels and remain below it after the injected positive label; all other model and calibration parameters remain at their reference values.

- Stimulus: replay exactly five thousand eligible `Action::Allow` labels with $v=-1$ against the same benign scalar population, completing the observation barrier before each label and the label barrier before its milestone can be read. Capture tracker, held-out prediction, basis, and health snapshots at the segment start, at declared intermediate milestones, and over the terminal window, then assess and label one otherwise matching request whose dormant binary signal creates the declared high-leverage direction.

- Observation: use `health_summary` for processed counts and the two tracker recurrences, `full_health_report` for ceiling and posterior diagnostics, and fresh held-out assessments for the risk basis. A test-only update trace captures the injected label's model identifier, valence branch, $h$, target weight, ceiling-limited weight, leverage limit, effective weight, and firing predicate from the same pre-update values the model consumed.

- Convergence assertion: require five thousand processed and eligible labels in the segment, match each terminal tracker to $\gamma^{5000}P_{+,\text{start}}$ within the scalar tolerance, place both trackers below $0.005$, place all three public risk probabilities below $\delta_\text{risk}$, and bound the terminal-window spread by $\delta_\text{settle}$. The calibration record counts and refit counter prove that the probability oracle is calibrated evidence rather than an unchanged cold-start sigmoid.

- Health assertion: require finite basis fields, positive uncertainty and calibration scale, borrowed share in $[0,1]$, finite positive `lambda_min` and `kappa` wherever a recomputation has published them, floor share below one, no alarm, no refused rebuild, no cascade, a live label path, and per-model synchronisation residual no greater than `precision_sync_per_dimension` times model width.

- Injection assertion: recompute both tracker values after the positive update, require the requested weights to equal one hundred, require the trace to show $L<w_*=100$ and $w_\text{eff}=L$ for the declared operational model, and require the held-out injection-direction probability to rise while remaining finite and within the resolved maximum single-label movement. A trace sequence keyed to the label acknowledgement prevents another concurrent update from satisfying the assertion.

- Fails-before: a tracker with the wrong decay or ordering misses the closed-form endpoint; an uncapped reciprocal weight exceeds one hundred; a missing leverage minimum records effective weight one hundred on the injection and violates the movement bound; a frozen or degenerate calibration misses the probability and terminal-spread oracles; and a broken precision update produces a non-finite basis, an unhealthy spectrum, excess synchronisation residual, or a stopped label path.

## What is missing · `sec:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk-missing`

**Entry (The fixed-length promise receives one numerical contract)** · `entry:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk-numerical-contract`

The contract selects whether five thousand applies at reference defaults or at an explicitly configured admissible forgetting pair, defines $\delta_\text{risk}$, $\delta_\text{settle}$, the terminal-window milestones, and the maximum injection movement, and states whether a calibration prefix may precede the adverse-free segment. The coherent configured-rate branch above preserves the stated segment length and every model mechanism; the reference-default branch requires a longer segment or a different initial rate. This is approximately ten to twenty lines of specification and intent prose and depends on a maintainer choice.

**Entry (A barriered label tape carries milestones and progress)** · `entry:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk-label-tape`

A `LabelTape` slice in the testing harness declares ordered request and label rows, the segment boundary, milestone callbacks, dual-queue barrier policy, and a liveness progress increment after each completed row. It reports the last completed row on a stall and keeps generated rows out of a five-thousand-item literal. This is approximately eighty to one hundred twenty lines and is a focused delivery of (´entry:assayer:harness-stage-tapes´) with no dependency beyond the delivered harness skeleton.

**Entry (The injection exposes the cap values consumed by one update)** · `entry:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk-update-trace`

The test-only operational probe planned by (´entry:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-operational-probe´) gains a bounded latest-event record carrying label sequence, $h$, target weight, ceiling-limited weight, leverage limit, effective weight, and firing predicate. `World` reads it only after the label barrier, and a crate test proves ceiling binding without leverage does not report a leverage fire. The extension is approximately fifty to ninety lines and depends on that probe entry or replaces it with an equivalent single shared implementation.

**Entry (The calibrated benign fixture keeps the complete witness)** · `entry:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk-integration-witness`

Test-local builders create the balanced calibration prefix, benign tape rows, held-out probes, dormant-direction injection, expected tracker recurrence, and terminal-window assertions. The integration test then composes the tape and update trace with the existing public health surfaces. This is approximately one hundred to one hundred forty lines plus the generated module and integration indexes, and depends on the numerical-contract, label-tape, and update-trace entries.

## Risks and open questions · `sec:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk-risks`

**Observation (Five thousand reference-default labels do not reach the ceiling regime)** · `obs:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk-default-horizon`

Starting at $0.5$, the reference rates yield approximately $0.0410$ globally and $0.1839$ on the eligible tracker after five thousand benign labels, both above the $0.005$ ceiling boundary. The alternatives are the configured-rate witness above, a longer reference-default run of roughly nine thousand two hundred global and twenty-three thousand eligible labels, or a host-supplied initial rate below the boundary; asserting default convergence at five thousand contradicts the specified recurrence.

**Observation (An adverse-free buffer cannot calibrate itself)** · `obs:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk-calibration-floor`

The calibration rule retains its current parameter unless the buffer contains at least three positive and three negative outcomes (´req:platt:minimum-samples´). A world cold-started directly into the benign segment therefore cannot turn its all-negative raw score into a newly fitted near-zero probability. The alternatives are a two-class calibration prefix outside the measured segment, an explicit seeded calibration fixture, or a narrower promise about raw risk or $P_+$ rather than `p_bad`; the witness uses the prefix only if the numerical contract admits it.

**Observation (The next adverse label needs a declared direction)** · `obs:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk-injection-geometry`

Five thousand repetitions concentrate covariance along the benign direction, so the next label on exactly that direction can be ceiling-limited without being leverage-limited. Toggling a dormant binary signal preserves the stream's population while supplying a high-uncertainty direction; if the promise instead requires an identical feature vector, the leverage-firing clause becomes a separate empirical condition and can legitimately fail after concentration.

**Observation (Determinism and runtime pull in opposite directions)** · `obs:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk-runtime`

Observations and labels use separate queues, and a label barrier does not settle standardisation observations. Per-row dual barriers fix the feature trajectory but can dominate the runtime of five thousand cycles; batching is admissible only if the tape proves that every later assessment sees the same published standardisation state, and the progress watcher converts a stalled queue into a diagnostic rather than a hang.

**Observation (Near-zero and settled need margins, not endpoint equality)** · `obs:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk-numerical-margin`

The sigmoid never reaches zero at a finite raw estimate, calibration refits can create discrete probability steps, and the spectral floor deliberately prevents exact posterior collapse. Acceptance uses a positive risk threshold, a separate terminal-spread threshold, and recorded distance from both boundaries; zero equality, monotonicity at every label, and a zero uncertainty assertion would each encode a mechanism the specification rejects.

## Acceptance · `sec:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk-acceptance`

The implementation report names the selected numerical contract, configuration, calibration precondition, benign and injection geometries, exact segment and terminal-window boundaries, probability and settling thresholds, movement bound, and the derivation of every tracker endpoint.

The report identifies the integration test and its intent citation, shows both $P_+$ values before and after the injection, the requested and effective positive weights, $h$, leverage limit, firing predicate, terminal risk range, calibration evidence, and per-model posterior-health readings.

The report shows focused coverage for the tape and update trace, the integration binary passing twice with identical milestones and trace values, the Assayer package verification required by the repository, and the corpus linter accepting the new test label, indexes, and citations.

The report demonstrates each fails-before case without executing a source mutation, gives the full witness runtime and any barrier or liveness diagnostics, and states every departure from the selected thresholds, configuration, fixture geometry, trace shape, or health oracle in this plan.
