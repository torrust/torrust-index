# The leverage bound retires after cold start · `plan:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after`

Keeping (´claim:bayes:the-leverage-bound-fires-at-cold-start-and-rarely-after´) establishes that the leverage cap protects the opening posterior from concentrated evidence and then yields to ordinary importance weighting once representative directions have accumulated precision.

## What the promise says, precisely · `sec:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-promise`

For one attempted operational-model update, let $h = \hat{\phi}^{\mathsf T}\Sigma\hat{\phi}$, $w_* = \min(w_\text{target}, w_\text{ceiling})$, and $F = \mathbb{1}[c/(h+\varepsilon) < w_*]$. The strict inequality is the code's firing predicate and distinguishes leverage binding from an importance ceiling that already selected the same or a smaller weight; the update itself uses the minimum of all three quantities (´alg:update:sherman-morrison´) and (´prop:update:leverage-bound´).

The cold quantity is $R_\text{cold}=\sum_{i=1}^{200}F_i/200$ over the first two hundred processed labels, with the acceptance interval $0.30 \leq R_\text{cold} \leq 0.60$. Every healthy fixture label reaches the operational model, so its attempted-update denominator must equal two hundred rather than silently dropping dimension mismatches or refused updates.

The steady-state quantities split the operational updates by the host's valence convention: $v>0$ is positive and adverse, while $v\leq0$ is non-positive and benign or neutral (´conv:valence:sign´). For a declared steady measurement window, $R_+=F_+/N_+$ must lie in $[0.02,0.08]$ and $R_-=F_-/N_-$ must be strictly below $0.001$.

The percentages are empirical observations, not universal bounds: the specification makes that standing explicit and says materially different label mixes may produce different rates (´data:gaussian:binding-frequency´). The witness therefore keeps the promise only for one independently warranted reference population; it does not turn the bands into a theorem about arbitrary feature distributions.

The mechanism predicts the direction of the transition. Early high covariance makes $h$ large and lets the leverage term dominate; accumulated precision lowers typical leverage until the class-balancing weight becomes operative (´disc:weighting:leverage-interaction´). The default safety factor, prior precision, forgetting rates, importance ceiling, and initial positive rate come from the reference configuration (´tab:config:risk-model´), and the decision record fixes that the cap is chosen before mutation rather than repaired afterwards (´dec:posterior:leverage-before´).

The first two hundred labels end the specification's pre-calibration stage, while steady state begins only after the feature-width-dependent convergence stages have passed (´tab:warmup:stages´). The two windows are consequently separated by an explicit burn-in; labels between them train the model and contribute to neither asserted rate.

The standing testing-plan entry covering this debt is the deeper convergence gap (´entry:assayer:gap-convergence-depth´). Its stated need for long streaming fixtures and model-state inspection is supplied more specifically here by the tape and event probe below, both narrow slices of (´entry:assayer:harness-stage-tapes´).

## What the code offers today · `sec:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-code`

The model-update helper already implements `leverage_bound_fired` with the exact strict predicate above, beside `compute_effective_weight`. The label path computes $h$ and the effective weight for each operational, sister, anchor, and outcome-axis update, but it discards whether the leverage term was the minimum; neither `LabelAck` nor either health tier exposes a firing event or count.

The label path already provides the closest accumulation pattern. `ImportanceAccumulator` separates the operational and eligible streams, records the exact requested balancing weights used by the update path, and publishes their ceiling-binding fractions and gradient balances; the completed health entry describes that constant-work design (´entry:health:importance-ceiling´), and its crate witness proves both streams use the accumulated values (´test:crate:importance-health-accumulates-each-model-stream´).

The world harness supplies configured `World` construction, `assess`, `label`, `flush_observations`, and `flush_labels`; its signal fixtures supply small variable-feature schemas; and its test generator supplies a stable seeded stream. These are the public-path ingredients already delivered by the harness skeleton (´entry:assayer:harness-stage-skeleton´).

The nearest model tests establish the two pieces separately. One proves that effective weight is the minimum of the target, ceiling, and leverage terms (´test:crate:effective-weight-is-the-smallest-of-target-ceiling-and-leverage-bound´); another samples varied vectors against an unchanged prior and sees the bound fire on essentially all of them (´test:crate:leverage-bound-fires-at-cold-start´). The latter is deliberately not a sequential convergence witness and its near-total cold rate is not the promised mixed-stream band.

The nearest integration tests exercise the real threaded path without observing the bound. The repeated benign and adverse cycles prove both valence branches stay finite under two hundred identical updates (´test:integration:repeated-identical-benign-cycle-stays-finite´) and (´test:integration:repeated-identical-adverse-cycle-stays-finite´), while the scalar population split proves five hundred public assess-label cycles learn directionally (´test:integration:scalar-signal-population-split-converges-directionally´). None distinguishes a leverage-capped update from an uncapped one.

`Assayer::pre_seed` reaches the ordinary label path efficiently, but its synthetic pending entries carry zero-featured Sentinel slots and schema-derived cached signals. It can seed class rates and posterior history, but it cannot stand in for a population whose feature-direction distribution is the quantity deciding leverage.

## The witness · `sec:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-witness`

The witness is one integration test in a focused leverage-frequency module. Its documentation cites the intent, and its module documentation indexes the test in the same form as the surrounding integration corpus.

- Setup: construct a `World` at the reference risk-model configuration, install the reference tape's signal schema, hold the virtual clock fixed, admit every row with `Action::Allow`, and assert clean health so eligibility, time decay, lifecycle changes, and numerical refusal cannot alter a denominator.

- Stimulus: for each tape row, build the row's public `RequestContext`, assess it, flush the observation queue so the cold standardisation ramp advances deterministically, submit the row's positive or non-positive `LabelSpec`, and flush the label queue before the next row. Snapshot the probe after label two hundred, continue through the independently declared burn-in, snapshot again, then run the steady measurement window and take a final snapshot.

- Observation: subtract cumulative operational-stream snapshots to obtain cold all-label, steady positive, and steady non-positive attempted and fired counts. The snapshot records the predicate at the same $h$, requested weight, ceiling, $c$, and $\varepsilon$ used by the update; it never infers firing from posterior movement or from `w_\text{eff}<w_\text{target}`, which would confuse the importance ceiling with leverage.

- Assertion: require exactly two hundred cold attempts and the $30$–$60\%$ cold band, then require at least five hundred positive and ten thousand non-positive steady attempts before applying the $2$–$8\%$ and below-$0.1\%$ bands. Print every numerator, denominator, rate, tape identity, feature width, and burn-in length on failure.

- Control: require the cumulative label count and the probe's operational-attempt count to advance together, require the standardisation phase to be in service before the steady window, and require the tape's realised valence mix to equal its declaration. These checks make a skipped update, queue race, or changed population fail as setup rather than masquerade as a rate change.

- Fails-before: an implementation that removes the leverage term reports zero firings and fails the cold lower bound, while one that keeps reading prior-level covariance or ceases to accumulate precision keeps firing through the steady window and fails its upper bounds. A counter placed after the minimum and unable to distinguish the importance ceiling also over-counts positive firings and fails the class-split assertions.

## What is missing · `sec:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-missing`

**Entry (The label path exposes the exact operational firing count to tests)** · `entry:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-operational-probe`

A cumulative `LeverageBindingSnapshot` in the shared test harness carries attempted and fired counts split into positive and non-positive operational updates. The label path evaluates `leverage_bound_fired` before mutation, increments only after the update is admitted, and places the counters in the crate-internal published owner state; `World` performs the label barrier and returns a copy. The surface is test-only, leaves production acknowledgements and host health schemas unchanged, and receives a focused crate test proving that ceiling binding alone does not count as leverage binding. The change is approximately ninety to one hundred forty lines and depends on no other entry.

**Entry (The empirical population becomes a deterministic reference tape)** · `entry:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-reference-tape`

The source population behind (´data:gaussian:binding-frequency´) is recovered and encoded as a generated, seeded tape rather than as a literal list tuned to the bands. The fixture declares schema and resulting width, feature distribution, valence rate and ordering, eligibility, seed, cold extent, burn-in rule, and measurement-window size; its constructor asserts the declared class counts before the engine sees a row. This is approximately eighty to one hundred forty lines, depends on the operational-probe entry, and is the focused leverage-frequency slice of (´entry:assayer:harness-stage-tapes´).

**Entry (One public-path scenario keeps all three rate bands)** · `entry:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-integration-witness`

The integration module runs the tape through the barriered `World` loop, takes three snapshots, checks the control invariants, and asserts all three bands in one trajectory. Keeping the windows together matters: separate cold and steady fixtures could each pass while no posterior ever made the promised transition. The test and module index are approximately seventy to one hundred lines and depend on both entries above.

## Risks and open questions · `sec:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-risks`

**Observation (The empirical bands have no measurement population in the corpus)** · `obs:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-population-provenance`

The specification records the rates but no schema, width, class balance, eligibility mix, feature law, seed, burn-in, or sample size that produced them. Recovering that measurement population preserves the present promise. If it cannot be recovered, the alternatives are a newly declared reference population justified independently of its observed rates, or a separate specification decision that replaces the numeric promise with a relational cold-greater-than-steady property; tuning a tape until it enters the existing bands is not evidence and is excluded.

**Observation (A label can attempt several geometrically different updates)** · `obs:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-denominator`

The operational model sees every processed label, the sister and anchor see only eligible labels, the anchor has a smaller feature vector, and each registered outcome axis adds another opportunity to fire. The primary witness uses operational attempts because that stream alone is in one-to-one correspondence with the promise's all-label denominator. If the recovered measurement instead counted any-model labels or per-model attempts, its aggregation replaces this interpretation explicitly; pooling all updates without naming the denominator does not.

**Observation (The runtime convergence enum is not the steady-state oracle)** · `obs:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-steady-boundary`

The warm-up table defines steady state analytically beyond the feature-width-dependent convergence cascade, while the runtime composite stage also depends on Platt refits and identity stability and does not encode the same boundary. The tape uses the recovered measurement's burn-in when available and otherwise the table's beyond-$10p$ eligible-label boundary with every row eligible; it records the resulting count rather than waiting on `ConvergenceStage::SteadyState`.

**Observation (Determinism costs one barrier per phase of every row)** · `obs:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-runtime`

Assessment-time standardisation observations and labels travel on different queues, and `flush_labels` does not settle the observation queue. Explicit observation and label barriers give every run the same coordinate trajectory but make a long steady window expensive. Runtime is measured before merging the witness; if it exceeds the package's integration budget, the tape is replayed through a synchronous harness driver that invokes the same label pipeline and retains one smaller public-path equivalence case, rather than dropping barriers or shrinking denominators until the rare-negative assertion loses resolution.

**Observation (Broad bands still have sharp edges)** · `obs:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-numerical-fragility`

The fixture is deterministic, so a rate near an endpoint is evidence that the population does not robustly keep the claim, not sampling noise to hide with tolerance. The implementation report records distance from every endpoint and repeats the focused test; a result on an endpoint remains admitted by the inclusive positive and cold intervals, while the negative ceiling remains strict.

## Acceptance · `sec:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after-acceptance`

The implementation report identifies the recovered or newly declared population provenance, the selected model-stream denominator, the feature width, valence and eligibility mix, seed, cold extent, burn-in rule, and steady measurement-window counts. A newly declared population includes its independent rationale and does not cite the observed firing rates as the reason for its shape.

The report names the integration test and its intent citation, shows the exact attempted and fired counts and derived rates for all three assertions, shows the standardisation and label-count controls, and records each rate's distance from its nearest boundary.

The report shows focused unit coverage for the probe, the focused integration binary passing twice with identical counts, the Assayer package verification required by the repository, and the corpus linter accepting the new test label, module index, generated integration matrix, and all citations.

The report explains how disabling leverage fails the cold assertion and how preventing posterior concentration fails the steady assertions, gives the measured runtime of the full witness, and states any departure from the tape, denominators, barriers, thresholds, or instrumentation shape in this plan.
