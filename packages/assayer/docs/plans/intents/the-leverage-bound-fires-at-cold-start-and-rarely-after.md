# The leverage bound retires after cold start · `plan:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after`

Keeping (´claim:bayes:the-leverage-bound-fires-at-cold-start-and-rarely-after´) establishes that the leverage cap protects the opening posterior from concentrated evidence and then yields to ordinary importance weighting once representative directions have accumulated precision.

## What the promise says, precisely · `sec:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-promise`

For one attempted operational-model update, let $h = \hat{\phi}^{\mathsf T}\Sigma\hat{\phi}$, let $f_t=\gamma_t^{\Delta t}$, let $h_\text{policy}=h/f_t$, let $w_* = \min(w_\text{target}, w_\text{ceiling})$, and let $F = \mathbb{1}[c/(h_\text{policy}+\varepsilon) < w_*]$. The strict inequality is the firing predicate implemented by `leverage_bound_fired`; the policy leverage is the covariance reading corrected to the steward processing instant, and the update uses the minimum of the target, ceiling, and leverage quantities (´alg:update:sherman-morrison´) (´prop:update:leverage-bound´).

The cold quantity is $R_\text{cold}=\sum_{i=1}^{200}F_i/200$ over the first two hundred processed labels, with the acceptance interval $0.30 \leq R_\text{cold} \leq 0.60$. Every admitted reference-population label must reach the operational model, so the attempted-update denominator must equal two hundred rather than silently dropping a refused update (´data:gaussian:binding-frequency´).

The steady-state quantities split the operational updates by the host's valence convention: $v>0$ is positive and adverse, while $v\leq0$ is non-positive and benign or neutral (´conv:valence:sign´). For the declared steady measurement window, $R_+=F_+/N_+$ must lie in $[0.02,0.08]$ and $R_-=F_-/N_-$ must be strictly below $0.001$.

The percentages are empirical observations, not universal bounds: the specification makes that standing explicit and says materially different label mixes may produce different rates (´data:gaussian:binding-frequency´). The witness therefore keeps the promise only for one independently warranted reference population; it does not turn the bands into a theorem about arbitrary feature distributions.

The mechanism predicts the direction of the transition. Early high covariance makes policy leverage large and lets the leverage term dominate; accumulated precision lowers typical leverage until the class-balancing weight becomes operative (´disc:weighting:leverage-interaction´). The default safety factor, prior precision, label-indexed forgetting rates, importance ceiling, and initial positive rate come from the reference risk-model configuration; the core model's time-indexed rate comes from the temporal configuration, and the cap is chosen before mutation rather than repaired afterwards (´tab:config:risk-model´) (´tab:config:temporal´) (´dec:posterior:leverage-before´).

The first two hundred labels reach the end of the warm-up table's approximate pre-calibration range, while the table places steady state only beyond the feature-width-dependent interaction-maturity range and explicitly denies that its stages are implementation transitions (´tab:warmup:stages´). The two measured windows are therefore separated by an independently declared burn-in; intervening labels train the model and contribute to neither asserted rate.

The standing testing-plan entry covering this debt is the deeper convergence gap (´entry:assayer:gap-convergence-depth´). The finished playback and probe contracts now supply the long-stream and owned-reading mechanics that entry once lacked; only the leverage count, reference population, and witness remain specific to this promise (´entry:assayer:harness-tape-runner´) (´entry:assayer:harness-probe-contract´).

## What the code offers today · `sec:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-code`

`leverage_bound_fired` implements the exact strict predicate beside `compute_effective_weight`, but it has no caller. `owner::label_path::apply_model_label_update` computes stored and policy leverage and then calls `compute_effective_weight` for each admitted operational, sister, anchor, and outcome-axis update, but discards which term was the minimum; `LabelAck` and both health tiers expose no leverage firing event or count.

The label path already provides the closest accumulation pattern. `ImportanceAccumulator` separates the operational and eligible streams, records the ceiling-capped balancing weights passed toward the update, and publishes their ceiling-binding fractions and gradient balances; it does not see policy leverage or effective weight (´entry:health:importance-ceiling´) (´test:crate:importance-health-accumulates-each-model-stream´).

The unified harness supplies configured `World` construction, `World::assess`, `World::label`, the queue-specific `World::flush_observations` and `World::flush_labels` barriers, and forward-only `World::advance` and `World::travel_to`. Long stimuli use `PlaybackRow`, `PlaybackBarrierPolicy`, `PlaybackCheckpoint`, `PlaybackProgress`, and `playback`, so the witness no longer needs a handwritten loop or any wait of its own (´dec:harness:no-ad-hoc-waits´) (´dec:harness:declarative-playback´) (´tab:assayer:harness-implementation-library-roster´).

The finished owned probes are `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView`. They provide coherent covariance, standardisation, or retained-input readings, but none exposes the target weight, policy leverage, effective weight, or a leverage-binding count, and the probe contract rules out reconstructing the answer from mixed engine state (´dec:harness:probe-contract´) (´entry:assayer:harness-probe-contract´).

The nearest model tests establish two pieces separately. One proves that effective weight is the minimum of the target, ceiling, and leverage terms (´test:crate:effective-weight-is-the-smallest-of-target-ceiling-and-leverage-bound´); another samples varied vectors against an unchanged prior and sees the bound fire on essentially all of them (´test:crate:leverage-bound-fires-at-cold-start´). The latter is not a sequential convergence witness and its near-total prior-only rate is not the promised mixed-population band.

The real threaded path still has no rate witness. `repeated_identical_benign_cycle_stays_finite` now cites the promise, so the citation census treats it as covered, but its own test label and assertions establish only that two hundred identical benign cycles stay finite and healthy; the adverse companion proves the same floor for the other valence branch (´test:integration:repeated-identical-benign-cycle-stays-finite´) (´test:integration:repeated-identical-adverse-cycle-stays-finite´). `scalar_signal_population_split_converges_directionally` proves that five hundred public cycles learn a direction, not that leverage binding enters any rate band (´test:integration:scalar-signal-population-split-converges-directionally´).

`Assayer::pre_seed` reaches the ordinary label path efficiently, but its synthetic pending entries carry occupied zero-featured Sentinel slots and schema-derived cached signals. It can seed class rates and posterior history, but it cannot stand in for a population whose feature-direction and time distributions decide policy leverage.

`World::trained_state` returns a guarded `TrainedStateFixture`, the oracle tier supplies four unrelated analytical computations, and `run_seeded_sweep` supplies reproducible invariant sweeps. None answers this empirical one-population transition: starting from the trained fixture would discard the cold half of the trajectory, no existing oracle computes a firing rate, and sweeping seeds would replace the promised reference population with another claim (´dec:harness:guarded-fixtures´) (´dec:harness:oracle-tier´) (´dec:harness:seeded-sweeps´).

## The witness · `sec:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-witness`

The `learning_convergence` integration target is the destination for `learning_convergence::leverage_bound_fires_at_cold_start_and_rarely_after`. Its test documentation cites the intent, and its module documentation indexes the function in the same form as the surrounding integration corpus.

- Setup: construct one guarded `World` at the reference risk-model and temporal configurations, install the reference population's declared schema, admit every row with `Action::Allow`, and assert the construction baseline and clean health before playback. The witness deliberately does not start from `World::trained_state`, because one trajectory must contain both the opening and steady windows (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

- Stimulus: a subject-owned row implementing `PlaybackRow` moves to its declared instant only through `World::advance` or `World::travel_to`, builds the public `RequestContext`, calls `World::assess`, and submits the positive or non-positive `LabelSpec`. Run the complete stimulus through `playback` with `PlaybackBarrierPolicy::new([PlaybackBarrier::FlushObservations, PlaybackBarrier::FlushLabels])`, bounded materialisation, and `PlaybackProgress`; declared `PlaybackCheckpoint` callbacks observe the boundary after label two hundred, after burn-in, and after the steady window (´dec:harness:declarative-playback´) (´entry:assayer:harness-closed-barriers´) (´test:crate:selected-label-barrier-completes-before-checkpoint´).

- Observation: subtract the cumulative operational-stream readings captured at the three checkpoints to obtain cold all-label, steady positive, and steady non-positive attempted and fired counts. The counter records the strict predicate evaluated from the update's actual policy leverage, requested weight, ceiling, $c$, and $\varepsilon$; it never infers firing from posterior movement or from an effective-versus-target comparison that would confuse the importance ceiling with leverage.

- Assertion: require exactly two hundred cold attempts and the inclusive $30$–$60\%$ cold band, require the steady positive and non-positive denominators to equal the reference population's declared counts, then apply the inclusive $2$–$8\%$ positive band and strict below-$0.1\%$ non-positive bound. Print every numerator, denominator, rate, population identity, feature width, time schedule, and burn-in length on failure; no denominator or tolerance is fitted from the observed firing rates.

- Control: require cumulative processed labels and operational attempts to advance together, require `StandardisationPhase::InService` before the steady window, and require realised row count, class counts, eligibility, feature width, and timestamps to equal the population declaration. These fixture guards make a skipped update, queue race, or changed population fail as setup rather than masquerade as a rate change (´dec:harness:separate-validation´).

- Fails-before: an implementation that removes the leverage term reports zero firings and fails the cold lower bound, while one that keeps reading prior-level covariance or ceases to accumulate precision keeps firing through the steady window and fails its upper bounds. A counter placed after the minimum and unable to distinguish the importance ceiling also over-counts positive firings and fails the class-split assertions.

## What is missing · `sec:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-missing`

**Entry (The label path exposes the exact operational firing count to tests)** · `entry:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-operational-probe`

A test-only `LeverageBindingSnapshot` in `testing::probes` carries cumulative attempted and fired counts split into positive and non-positive operational updates. `owner::label_path` evaluates `leverage_bound_fired` from the same policy leverage and weights passed to `compute_effective_weight`, records only an admitted operational update, and publishes the cumulative reading through crate-internal test-support state; `World::leverage_binding_snapshot` crosses `World::flush_labels`, loads once, and returns an owned copy. The surface leaves `LabelAck` and host health schemas unchanged, follows the probe contract, and receives a focused crate test proving that ceiling binding alone is not leverage binding (´dec:harness:probe-contract´).

**Entry (The empirical population becomes a deterministic reference tape)** · `entry:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-reference-tape`

The source population behind (´data:gaussian:binding-frequency´) is recovered and encoded in `learning_convergence` as a generated, seeded sequence of subject-owned playback rows rather than a literal list tuned to the bands. Its guarded declaration fixes schema and resulting width, feature distribution, valence rate and ordering, eligibility, seed, monotonic time schedule, cold extent, burn-in rule, and both steady-window class counts; construction checks the complete realised population before the engine sees a row. The row type delegates cadence, checkpoints, bounded batching, and progress to the finished playback runner (´dec:harness:declarative-playback´) (´entry:assayer:harness-tape-runner´).

**Entry (One public-path scenario keeps all three rate bands)** · `entry:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-integration-witness`

`learning_convergence::leverage_bound_fires_at_cold_start_and_rarely_after` runs the declared rows through `playback`, captures three cumulative readings at declared checkpoints, checks every fixture guard, and asserts all three bands in one trajectory. Keeping the windows together matters: separate cold and steady fixtures could each pass while no posterior ever made the promised transition. This entry depends on both entries above.

## Risks and open questions · `sec:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-risks`

**Observation (The empirical bands have no measurement population in the corpus)** · `obs:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-population-provenance`

The specification records the rates but no schema, width, class balance, eligibility mix, feature law, seed, time schedule, burn-in, or sample size that produced them. Recovering that measurement population preserves the present promise. If it cannot be recovered, the alternatives are a newly declared reference population and window sizes justified independently of their observed rates, or a separate specification decision that replaces the numeric promise with a relational cold-greater-than-steady property; tuning rows, timing, burn-in, or denominators until they enter the existing bands is not evidence and is excluded.

**Observation (A label can attempt several geometrically different updates)** · `obs:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-denominator`

The operational model sees every processed label, the sister and anchor see only eligible labels, the anchor has a smaller feature vector, and each registered outcome axis adds another opportunity to fire. The witness uses operational attempts because that stream alone is in one-to-one correspondence with the promise's all-label denominator. The finished `PublishedModelBlock` cannot substitute for the narrow counter: covariance alone omits the assessment-time standardised vector, requested weight, and policy-time correction, while widening it to working precision would violate the retained probe boundary (´dec:harness:probe-contract´). If recovered provenance counted another stream, that disagreement requires a specification decision rather than silent pooling.

**Observation (The runtime convergence enum is not the steady-state oracle)** · `obs:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-steady-boundary`

The warm-up table describes interaction maturity as approximately $2p$–$10p$ labels and explicitly says its stages are approximate descriptions rather than transitions; it cannot supply an exact fallback burn-in (´tab:warmup:stages´). `CompositeConvergenceStage::SteadyState` also depends on calibration and identity stability, while `TrainedStateFixture` guards a balanced score-verified population rather than the covariance occupancy this rate needs. The witness therefore uses recovered measurement burn-in or stops for an independently justified replacement precondition; it does not promote either existing diagnostic to an oracle (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

**Observation (Determinism costs one barrier per phase of every row)** · `obs:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-runtime`

Assessment-time standardisation observations and labels travel on different queues, and `World::flush_labels` does not settle the observation queue. The ordered playback policy crosses both after every row, which gives every run the same next-row coordinate state but makes a long steady window expensive; bounded playback batches share only materialisation cost and do not weaken row boundaries (´entry:assayer:harness-closed-barriers´) (´cor:ordering:tape-batches´). Runtime is measured before merging the witness, and a budget breach is reported rather than answered with a local synchronous driver, missing barriers, or smaller denominators.

**Observation (Broad bands still have sharp edges)** · `obs:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-numerical-fragility`

The fixture is deterministic, so a rate near an endpoint is evidence that the population does not robustly keep the claim, not sampling noise to hide with tolerance. The implementation report records distance from every endpoint and repeats the focused test; an endpoint remains admitted by the inclusive cold and positive intervals, while the negative ceiling remains strict. `run_seeded_sweep` is not a remedy for a fragile reference population because the promise names one population rather than a distribution over seeds (´dec:harness:seeded-sweeps´).

## Acceptance · `sec:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-acceptance`

The implementation report identifies the recovered or newly declared population provenance, the selected model-stream denominator, the feature width, valence and eligibility mix, seed, time schedule, cold extent, burn-in rule, and steady measurement-window counts. A newly declared population includes its independent rationale and does not cite the observed firing rates as the reason for its shape.

The report names cargo integration target `learning_convergence` and Rust function `learning_convergence::leverage_bound_fires_at_cold_start_and_rarely_after`, shows its intent citation, lists the exact attempted and fired counts and derived rates for all three assertions, shows the standardisation and label-count controls, and records each rate's distance from its nearest boundary.

The report shows focused crate coverage for `LeverageBindingSnapshot` and `World::leverage_binding_snapshot`, the focused integration target passing twice with identical counts, the Assayer package verification required by the repository, and the corpus linter accepting the new test label, module index, generated integration matrix, and all citations.

The report explains how disabling leverage fails the cold assertion and how preventing posterior concentration fails the steady assertions, gives the measured runtime of the full witness, and states any departure from the population, time schedule, denominators, barriers, thresholds, or instrumentation shape in this plan.
