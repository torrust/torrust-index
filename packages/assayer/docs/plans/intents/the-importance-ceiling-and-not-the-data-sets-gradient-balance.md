# The importance ceiling sets rare-class gradient balance · `plan:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance`

Keeping (´claim:bayes:the-importance-ceiling-and-not-the-data-sets-gradient-balance´) establishes through the host-visible configuration and health surfaces that the importance ceiling, rather than a rarer data set alone, limits the positive class's share of balancing weight and makes that share move when the configured ceiling moves.

## What the promise says, precisely · `sec:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-promise`

The positive-valence rate is $p = 0.001$. The balancing definition writes $w_+(p, c) = \min(1/(2p + \varepsilon), c)$ and $w_-(p, c) = \min(1/(2(1-p) + \varepsilon), c)$, where $c$ is the configured importance ceiling, and separately states that each uncapped class contributes exactly half the total gradient weight (´def:weighting:balancing-weights´).

The measurable gradient balance is the positive share of accumulated balancing mass, $G_+(p,c) = p w_+(p,c) / (p w_+(p,c) + (1-p) w_-(p,c))$. It is neither the ratio $w_+/w_-$ nor the fraction of labels that are positive; the health contract accumulates the weights each stream actually used (´cav:limitation:ceiling´) (´entry:health:importance-ceiling´).

At $p = 0.001$ and $c = 100$, the definition's table and the guard-clamped implementation give $w_+ = 100$ and $w_- = 1/1.998 \approx 0.5005$. One positive among one thousand labels therefore contributes mass $100$, the nine hundred ninety-nine negatives contribute mass $500$, and $G_+ = 1/6 \approx 0.1667$. The ceiling binds because the rate is below $1/(2c)$ (´def:weighting:balancing-weights´).

The ceiling is a host parameter with default 100 and admissible domain $c \geq 1$ (´tab:config:risk-model´). Moving it changes the maximum requested importance of one rare label, while the leverage policy may reduce the effective posterior update further before mutation (´disc:weighting:leverage-interaction´) (´dec:posterior:leverage-before´).

At the same rate and $c = 500$, the settled promise gives positive and negative accumulated masses of $500$, hence $G_+ = 1/2$. One third would instead follow from $c = 250$ at that rate, or from $p = 0.0005$ at $c = 500$ under the same guard-clamped reciprocal calculation.

The label path updates the relevant class-rate tracker before deriving that label's weight (´alg:weighting:tracker-update´), then applies the operational update unconditionally and the sister and anchor updates only for eligible labels (´alg:runtime:update-path´). A population witness must therefore control label order and eligibility as well as the population ratio, and must check both published tracker rates before interpreting the accumulated balance.

## What the code offers today · `sec:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-code`

`AssayerConfig::model` carries a `ModelConfig`; `ModelConfig::w_ceiling` supplies the operator dial, while `ModelConfig::p_plus_init`, `ModelConfig::gamma_opr`, and `ModelConfig::gamma_inh` let a deterministic scenario begin at the target rate and bound tracker motion over the finite population. `AssayerBuilder` validates the ceiling, rates, and ordering constraint between the two forgetting rates before projecting them into the label path.

`compute_importance_weight` clamps the rate to the interval from $0.001$ through $0.999$, applies the reciprocal-half calculation, and caps the result. Its focused unit tests cover balanced data (´test:unit:importance-weight-balanced´), an uncapped one-in-ten rate (´test:unit:importance-weight-rare-positive´), and a capped one-in-a-hundred rate (´test:unit:importance-weight-ceiling-applied´). The landed crate witness `learning_policy_conforms_row_by_row` additionally covers both sides of the default ceiling boundary after the tracker update (´test:crate:learning-policy-conforms-row-by-row´), but none of them compares ceilings 100 and 500 at the promised rate.

The label path computes separate operational and eligible weights, passes them to the models, and accumulates label count, ceiling hits, total balancing weight, and positive balancing weight for each stream. `Assayer::health_summary` exposes both published tracker rates, while `Assayer::full_health_report` exposes the accumulations through `ImportanceCeilingHealth::ceiling_binding_fraction`, `ImportanceCeilingHealth::gradient_balance`, `ImportanceCeilingHealth::sister_ceiling_binding_fraction`, and `ImportanceCeilingHealth::sister_gradient_balance` (´entry:health:importance-ceiling´). The crate witness `importance_health_accumulates_each_model_stream` proves that both streams accumulate the requested weights supplied to their updates, but its ceiling-one sequence cannot distinguish the promised rare-rate values or a host ceiling change (´test:crate:importance-health-accumulates-each-model-stream´).

The finished harness offers one `Scenario` over one `World`, guarded construction through `scenario_with_config`, and the queue-specific `World::flush_labels` barrier (´tab:assayer:harness-implementation-library-roster´) (´entry:assayer:harness-closed-barriers´). Long populations now run through `playback` with subject-owned `PlaybackRow` values, an explicit `PlaybackBarrierPolicy`, bounded `PlaybackBatchSize`, and `PlaybackProgress`; the runner, rather than a test-local loop or `World::cycle_on`, owns the settling cadence (´dec:harness:declarative-playback´) (´entry:assayer:harness-tape-runner´).

The owned `HealthSummary` and `SystemHealthReport` values returned by the two host health calls are the promised readings, so the state-projection types `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView` are not needed here. The shared oracle tier contains `pairwise_rank`, `regularised_schur_complement`, `decay_recurrence`, and `dimension_width`, none of which computes an affine class-rate recurrence or accumulated balancing mass; the one-caller expectation therefore remains a specification-derived helper beside this test rather than a new shared oracle (´dec:harness:probe-contract´) (´dec:harness:oracle-tier´) (´entry:assayer:harness-oracle-tier´).

No trained or converged model state is a precondition: the configured tracker rate and the post-population tracker readings are the guard, so `TrainedStateFixture` and `TrainedStateBaseline` do not apply. The stimulus has no randomized input, so `run_seeded_sweep` does not apply either; construction remains guarded and reports its measured baseline before the witness begins (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´) (´dec:harness:seeded-sweeps´) (´entry:assayer:harness-guarded-fixtures´).

The public edge-case target is the subject-aligned home: its existing repeated-adverse witness already reaches the importance-weighted branch (´test:integration:repeated-identical-adverse-cycle-stays-finite´), while the directional-learning target is about posterior separation rather than configuration-to-health arithmetic (´test:integration:scalar-signal-population-split-converges-directionally´). The testing plan has no entry for this invariant; its convergence-depth entry concerns posterior convergence and starvation and still describes playback and probes as prerequisites even though both have landed (´entry:assayer:gap-convergence-depth´).

## The witness · `sec:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-witness`

The `edge_cases` integration target is the destination for `edge_cases::importance_ceiling_controls_rare_class_gradient_balance`; its documentation cites the existing claim and mints only the test's standard test label.

- Setup: construct two independent scenarios through `scenario_with_config`, set `ModelConfig::p_plus_init` to $0.001$, set `ModelConfig::gamma_opr` and `ModelConfig::gamma_inh` to the two nearest ordered representable values below one, and vary only `ModelConfig::w_ceiling` between 100 and 500.

- Fixture: create one thousand eligible rows in a fixed order, with nine hundred ninety-nine benign `Action::Allow` rows followed by one adverse `Action::Allow` row. Each test-local `PlaybackRow` calls `World::derive_default` and `World::label` without waiting, and the same rows drive both scenarios.

- Stimulus: call `playback` with `PlaybackBarrierPolicy::new([PlaybackBarrier::FlushLabels])`, a bounded `PlaybackBatchSize`, no checkpoints, and a fresh `PlaybackProgress` for each scenario. Require each progress reading to finish after one thousand rows, so every submitted label crossed the named publication barrier and no sleep, poll, deadline, clock movement, or random draw participates.

- Control: derive each tracker's expected terminal rate from the recurrence in (´alg:weighting:tracker-update´), and compare it with `HealthSummary::p_positive_global` and `HealthSummary::p_positive_eligible` from `Assayer::health_summary` under the declared EWMA budget in (´tab:assayer:harness-scenario-tolerances´). This proves that both streams stayed in the promised rare-rate regime rather than merely receiving the desired initial configuration.

- Observation: after playback's final `PlaybackBarrier::FlushLabels`, call `Assayer::full_health_report` once per scenario and retain the two owned reports. Read only the four `ImportanceCeilingHealth` fields; no prediction, calibration result, model coefficient, probe, or internal state participates.

- Expected-value calculation: independently apply the tracker recurrence and the guard-clamped formula from (´def:weighting:balancing-weights´) to the declared rows, sum positive and total requested weight for each stream, and derive the comparison budget from the row count, `f64::EPSILON`, and the largest summed weight. The calculation must place the ideal centres $1/6$, $1/2$, and $1/3$ inside that derived budget without calling `compute_importance_weight` or fitting a tolerance to observed output.

- Default-ceiling assertion: require both gradient balances to match the independently accumulated $1/6$ expectation, and both binding fractions to match one ceiling hit among one thousand labels.

- Upper-ceiling assertion: require both gradient balances to match the independently accumulated $1/2$ expectation. Do not require a ceiling-hit count at 500, because the promised rate lies on the boundary and post-update rounding decides whether equality is recorded as a hit.

- Dial assertion: require each upper-ceiling balance minus its corresponding default-ceiling balance to match $1/3$ under the same derived budget, so two plausible but identical readings cannot pass.

- Fails-before: a hard-coded default ceiling makes the paired worlds agree; removing the cap makes the default arm approach one half; using an odds ratio changes the negative mass; reporting the raw positive-label fraction returns $0.001$; applying the sister tracker to the operational stream breaks the distinct-rate oracle; and asserting one third at ceiling 500 fails against the retained promise.

## What is missing · `sec:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-missing`

**Entry (The higher-ceiling row has one numerical contract)** · `entry:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-numerical-contract`

The `edge_cases` target lacks a private specification-derived expectation helper that advances both tracker recurrences over the declared row sequence, applies the guard-clamped reciprocal formula independently of `compute_importance_weight`, accumulates per-stream mass, and returns the four expected health readings plus a floating-point budget derived from its operation count. This is test-local arithmetic, not a fifth shared oracle.

**Entry (A deterministic rare-rate population is a test-local fixture)** · `entry:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-rare-rate-fixture`

The `edge_cases` target lacks a private `PlaybackRow` implementation for one eligible benign or adverse assess-label row and a deterministic iterator yielding the promised population. Its `play` method uses `World::derive_default` and `World::label`; `PlaybackBarrierPolicy` supplies the only wait, so no new `World` verb, probe, clock seam, trained fixture, or sweep is missing.

**Entry (The public health assertion keeps both streams)** · `entry:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-health-assertion`

The `edge_cases::importance_ceiling_controls_rare_class_gradient_balance` body is still absent. It must construct the paired scenarios, run the shared rows through playback, guard both terminal tracker rates through `Assayer::health_summary`, read one owned full health report per scenario, and assert both streams' binding fractions, balances, and between-ceiling deltas against the independent expectations.

## Risks and open questions · `sec:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-risks`

**Observation (The higher-ceiling row follows the weighting definition)** · `obs:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-higher-row-conflict`

The definition prints an additive $\varepsilon$ in each denominator but then states exact equal class masses until the ceiling binds; `compute_importance_weight` realizes the guard by clamping the rate and applying no additive denominator term. The settled claim and current code therefore support one half at ceiling 500, but the witness must keep the guard choice explicit rather than inventing a magnitude for $\varepsilon$ or hiding the distinction inside a fitted tolerance.

**Observation (Tracker motion can impersonate a ceiling effect)** · `obs:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-tracker-motion`

Each label moves the global and eligible rates before its own weight is computed, and the two trackers have distinct legal forgetting rates. The nearest ordered representable rates below one bound that movement without violating construction, while the independent recurrence and both published final-rate assertions detect an ordering, eligibility, or configuration error before the health balances are interpreted.

**Observation (Health balance is requested balancing mass, not realised posterior motion)** · `obs:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-health-scope`

The health accumulator records the importance weights requested for model updates; the leverage bound may lower an individual update's effective weight afterwards. The witness establishes the operator's importance dial and its balancing share, while the separate leverage policy prevents the stronger claim that one positive label moves the posterior by the whole configured weight.

**Observation (The boundary at ceiling 500 is numerically sharp)** · `obs:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-boundary-rounding`

At $p = 0.001$, the guard-clamped positive weight is exactly the intended upper ceiling in real arithmetic. The upper arm therefore treats gradient balance as load-bearing and does not require a ceiling-hit predicate whose equality can be perturbed by the tracker update or binary representation; the default arm at 100 remains well inside the binding regime and carries the binding assertion.

**Observation (Synchronous pre-seeding removes timing without bypassing behaviour)** · `obs:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-preseed-determinism`

`Assayer::pre_seed` still travels the ordinary label path and blocks on publication (´dec:construction:post-seeding´) (´alg:host:pre-seeding´), but it would make the production call rather than the shared runner own the long population's progress and settling cadence. The finished harness therefore replaces the earlier direct-pre-seed design with assess-label `PlaybackRow` values and an explicit label barrier while retaining deterministic order and ordinary label-path behaviour (´dec:harness:declarative-playback´) (´dec:harness:no-ad-hoc-waits´).

## Acceptance · `sec:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-acceptance`

The implementation adds the `edge_cases` integration target's `edge_cases::importance_ceiling_controls_rare_class_gradient_balance` test, cites the existing claim in its documentation, and updates that target's generated test index without introducing another behavioural claim.

The implementation report records the two exact configurations, ordered class population, playback barrier policy, terminal tracker rates, specification-derived weights, per-stream balances, default binding fraction, between-ceiling delta, and operation-count-derived floating-point budget beside the assertions.

The report shows the focused `edge_cases` target passing, the Assayer package tests passing under the required feature profiles, and the corpus linter accepting the new test label, generated test indexes, and every citation.

The report explains which deliberate defect each assertion rejects, confirms that no sleep, poll, local deadline, clock movement, random draw, shared mutable fixture, probe, or internal model-state read enters the witness, and reports any runtime or numerical departure from this plan.
