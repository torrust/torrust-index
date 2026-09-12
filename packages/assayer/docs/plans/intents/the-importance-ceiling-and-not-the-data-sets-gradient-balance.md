# The importance ceiling sets rare-class gradient balance · `plan:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance`

Keeping (´claim:bayes:the-importance-ceiling-and-not-the-data-sets-gradient-balance´) establishes through the host-visible configuration and health surfaces that the importance ceiling, rather than a rarer data set alone, limits the positive class's share of balancing weight and makes that share move when the configured ceiling moves.

## What the promise says, precisely · `sec:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-promise`

The positive-valence rate is $p = 0.001$. The balancing definition gives $w_+(p, c) = \min(1/(2p), c)$ and $w_-(p, c) = \min(1/(2(1-p)), c)$, where $c$ is the configured importance ceiling (´def:weighting:balancing-weights´).

The measurable gradient balance is the positive share of expected balancing mass, $G_+(p,c) = p w_+(p,c) / (p w_+(p,c) + (1-p) w_-(p,c))$. It is not the ratio $w_+/w_-$ and it is not the fraction of labels that are positive.

At $p = 0.001$ and $c = 100$, $w_+ = 100$, $w_- = 1/1.998 \approx 0.5005$, the positive mass is $0.1$, the negative mass is $0.5$, and $G_+ = 1/6 \approx 0.1667$. The ceiling binds because the rate is below $1/(2c)$, the threshold disclosed by (´cav:limitation:ceiling´).

The ceiling is a host parameter with default 100 and admissible domain $c \geq 1$ (´tab:config:risk-model´). Moving it changes the maximum requested importance of one rare label, while the leverage policy may reduce the effective posterior update further before mutation (´disc:weighting:leverage-interaction´) and (´dec:posterior:leverage-before´).

At the same $p = 0.001$, the specified equation gives $G_+(p,500) = 1/2$, not one third: $p w_+ = 0.5$ and $(1-p)w_- = 0.5$. One third follows from $c = 250$ at that rate, or from $p = 0.0005$ at $c = 500$. The promise's higher-ceiling row therefore requires a numerical-contract decision before an assertion can keep it literally.

The label path updates the relevant class-rate tracker before deriving the label's weight (´alg:weighting:tracker-update´), then applies the operational update unconditionally and the sister and anchor updates only for eligible labels (´alg:runtime:update-path´). A witness using a label population consequently controls both the empirical class ratio and tracker drift.

## What the code offers today · `sec:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-code`

`AssayerConfig.model.w_ceiling` supplies the operator dial, while `AssayerConfig.model.p_plus_init`, `gamma_opr`, and `gamma_inh` let a deterministic fixture begin at the target rate and keep both rate trackers effectively fixed over a short population. `AssayerBuilder` validates these values and projects them into the label-path configuration.

The importance-weight helper implements `compute_importance_weight` as the capped reciprocal-half formula. Its existing focused tests cover balanced data (´test:unit:importance-weight-balanced´), an uncapped one-in-ten rate (´test:unit:importance-weight-rare-positive´), and a capped one-in-a-hundred rate (´test:unit:importance-weight-ceiling-applied´), but they do not cover the target rate, compare two ceilings, or observe the resulting class-mass share.

The label path computes separate operational and eligible weights, passes them to the models, and accumulates label count, ceiling hits, total balancing weight, and positive balancing weight for each stream. The full public health report exposes those accumulations as `ceiling_binding_fraction`, `gradient_balance`, `sister_ceiling_binding_fraction`, and `sister_gradient_balance`, implementing the completed health record (´entry:health:importance-ceiling´).

The crate-level mixed-label witness already proves that both health streams accumulate the balancing weights supplied to their model updates (´claim:labelling:importance-health-accumulates-the-weights-used-by-each-model-stream´). Its ceiling-one sequence forces every label to bind and therefore cannot distinguish the rare-rate values or show that a host ceiling changes the result.

The world harness supplies `World`, configured construction, the underlying `Assayer` escape hatch, and deterministic post-label barriers. The pre-seed fixture builder supplies `PreSeedSpec`; `Assayer::pre_seed` sends every entry through the ordinary label path and blocks until publication, as required by the post-construction seeding decision (´dec:construction:post-seeding´).

The directional-learning integration witness is the nearest public assess-label stream (´test:integration:scalar-signal-population-split-converges-directionally´). The target witness belongs in the public edge-case integration-test module, where an extreme class rate and a bounded positive branch fit the existing public-surface scope without turning the assertion into a convergence test.

The standing testing plan has no entry covering this configuration-to-weight-to-health invariant. Its class-imbalance entry concerns posterior convergence and starvation over long streams (´entry:assayer:gap-convergence-depth´), while this promise is a closed-form balance row observable without model-state inspection or convergence tapes.

## The witness · `sec:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-witness`

The witness is one test in the public edge-case integration-test module, named `importance_ceiling_controls_rare_class_gradient_balance`, whose documentation cites the intent rather than minting a second claim.

- Setup: construct two independent worlds through `scenario_with_config`, set `p_plus_init` to $0.001$, set the two model forgetting rates to distinct representable values immediately below one, and vary only `w_ceiling` between 100 and the resolved upper-row value.

- Stimulus: build one thousand eligible `PreSeedSpec` entries with exactly one adverse label and nine hundred ninety-nine benign labels, submit the same ordered population to both worlds through `Assayer::pre_seed`, and assert that every entry was processed.

- Control: read each returned final positive-rate estimate and require it to remain within the ordinary floating-point tolerance of $0.001$, proving that the fixture exercised the promised rate rather than a transient tracker value.

- Observation: call `full_health_report()` after the synchronous pre-seed and read both operational and sister importance-ceiling health fields; no prediction, calibration result, model coefficient, clock advance, random sample, or sleep participates.

- Default-ceiling assertion: require both gradient balances to equal $1/6$ within a tight arithmetic tolerance and both binding fractions to identify the single positive ceiling hit in the one-thousand-label population.

- Upper-ceiling assertion: after the numerical contract is resolved, require both streams to equal either $1/2$ for the specified $c = 500$ equation or $1/3$ for a corrected $c = 250$ intent row; the test contains one resolved branch, never a tolerance broad enough to admit both.

- Dial assertion: require the upper-ceiling balance to exceed the default-ceiling balance by the resolved closed-form amount, so a test cannot pass merely because both configurations produce plausible isolated readings.

- Fails-before: an implementation that hard-codes the default ceiling makes the two worlds agree; an implementation that removes the cap makes the default world report one half; an odds-ratio implementation moves the negative weight away from one half; a counter that reports the raw positive-label fraction returns $0.001$; and a literal one-third assertion at ceiling 500 fails against the present specified implementation by reporting one half.

## What is missing · `sec:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-missing`

**Entry (The higher-ceiling row has one numerical contract)** · `entry:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-numerical-contract`

The implementing change resolves the conflict before fixing an expected value: retain ceiling 500 and correct the promised balance to one half, or retain one third and change the promised ceiling to 250. The alternative of retaining both 500 and one third entails a different weighting equation or a different positive rate and therefore a specification change wider than this witness. The corpus edit is approximately five to ten lines and depends on a maintainer choice, not on another lane.

**Entry (A deterministic rare-rate population is a test-local fixture)** · `entry:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-rare-rate-fixture`

A helper in that integration-test module builds the ordered `PreSeedSpec` population once and reuses it for both configured worlds. It is approximately twenty to thirty lines, depends on the numerical-contract entry for its upper arm, and needs no new `World` verb, tape abstraction, clock seam, model-state accessor, or other lane.

**Entry (The public health assertion keeps both streams)** · `entry:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-health-assertion`

The test body constructs the paired worlds, submits the fixture, and checks processed counts, final rates, binding fractions, per-stream balances, and the cross-configuration delta. It is approximately forty to sixty lines plus generated test-index updates and depends only on the two entries above.

## Risks and open questions · `sec:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-risks`

**Observation (The published higher-ceiling number contradicts the weighting definition)** · `obs:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-higher-row-conflict`

The one-third expectation cannot be derived from the cited balancing formula at the stated rate and ceiling. Encoding it unchanged produces a red test against conforming code, while widening the tolerance to include one half erases the promise; the numerical-contract entry is therefore a semantic prerequisite rather than test tuning.

**Observation (Tracker motion can impersonate a ceiling effect)** · `obs:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-tracker-motion`

Production forgetting rates move $P_+$ on every label, and the current label participates before its own weight is computed. Rates immediately below one isolate the ceiling algebra over the finite fixture, while the explicit final-rate assertion detects a configuration or ordering change that would invalidate that isolation.

**Observation (Health balance is requested balancing mass, not realised posterior motion)** · `obs:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-health-scope`

The health accumulator records the importance weights presented to the model updates; the leverage bound may lower an individual update's effective weight afterwards. The witness establishes the operator's importance dial and its balancing share, while the separate leverage policy keeps the stronger statement that no single positive label necessarily moves the posterior by the whole configured weight.

**Observation (The boundary at ceiling 500 is numerically sharp)** · `obs:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-boundary-rounding`

At $p = 0.001$, the uncapped positive weight is exactly the intended upper ceiling of 500 in real arithmetic. The upper arm therefore treats gradient balance as load-bearing and does not require a ceiling-hit predicate whose equality can be perturbed by the tracker update or binary representation; the default arm at 100 remains well inside the binding regime and carries the binding assertion.

**Observation (Synchronous pre-seeding removes timing without bypassing behaviour)** · `obs:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-preseed-determinism`

Pre-seeding blocks on the same label path that live labels use, so the health read has a publication barrier and no liveness deadline. The ordered fixture and near-unit rates remove random class placement and scheduler-dependent tracker drift without replacing production weight computation with a test oracle.

## Acceptance · `sec:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance-acceptance`

The implementation report names the selected numerical contract and shows the corresponding intent or specification correction, with the rejected alternative stated explicitly.

The report identifies the integration test and its cited intent, records the exact configuration and class population, and shows the hand-derived expected weights, balances, binding fraction, and between-ceiling delta beside the assertions.

The report shows the focused integration binary passing, the Assayer package tests passing under the required feature profiles, and the corpus linter accepting the new test label, generated test indexes, and every citation.

The report explains which deliberate defect each assertion rejects, confirms that no sleep, clock advance, random draw, shared mutable fixture, or internal model-state read enters the witness, and reports any runtime or numerical-tolerance departure from this plan.
