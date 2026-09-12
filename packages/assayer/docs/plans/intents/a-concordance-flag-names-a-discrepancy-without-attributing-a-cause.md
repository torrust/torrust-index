# Keeping the Concordance Flag Causally Neutral · `plan:assayer:intent-a-concordance-flag-names-a-discrepancy-without-attributing-a-cause`

This plan keeps the promise that the per-entity concordance flag exposes a measured discrepancy and no diagnosis of why it arose (´claim:wellness:a-concordance-flag-names-a-discrepancy-without-attributing-a-cause´).

## What the promise says, precisely · `sec:assayer:intent-a-concordance-flag-names-a-discrepancy-without-attributing-a-cause-promise`

For each retained entity $e$, let $n_e$ be its observed label count and let $a_{e,i}$ be one exactly when label $i$'s binary risk target and the effective risk estimate frozen at assessment time have the same sign. The target is $+1$ for positive valence and $-1$ otherwise (´def:risk:target´), and the entity's concordance is the running fraction $c_e = n_e^{-1}\sum_i a_{e,i}$ (´alg:monitoring:per-entity-concordance´).

The comparison figure $A$ is the currently published aggregate rank discrimination over the calibration buffer, with tie correction and its own class-count gate (´alg:monitoring:auc´). It is the system's population-level performance comparator, not a second sign-agreement fraction.

The flag is eligible exactly when $n_e \geq N_{\text{conc,min}}$; the configured default is five labels and equality reaches the gate (´def:config:concordance-minimum-labels´). Given an available $A$, the flag is true exactly when $c_e < A - \delta_{\text{conc}}$; the configured default deficit is $0.25$, equality stays unflagged, and the comparison is strict (´def:config:concordance-deficit-threshold´). An absent aggregate AUC leaves every entity unflagged because there is no population comparator to read.

Those inputs contain an entity key, a count, a sign-agreement fraction, aggregate discrimination, and two configured thresholds. They contain no observation of label-production integrity, environmental randomness, or another causal variable. The monitoring contract therefore permits the conclusion that this entity differs from the population and forbids the stronger conclusion that its labels are wrong.

The limitation is structural rather than statistical: the Core trusts labels and its integrity diagnostics surface signatures without establishing corruption (´inv:guarantee:honest-uncertainty´), (´cav:monitoring:integrity-scope´). A legitimate regime change can reproduce a corruption signature wherever the observed inputs coincide (´cav:limitation:mimic-regime´).

The flag is a report-only reading. It changes no model, threshold, rate, route, or decision (´inv:monitoring:report-only´), and the health record independently fixes that reported condition as a non-gating quantity (´dec:health:reports-never-gates´). A host receives grounds to investigate and no engine-issued grounds to accuse or act.

## What the code offers today · `sec:assayer:intent-a-concordance-flag-names-a-discrepancy-without-attributing-a-cause-current-code`

The public detailed-health query is `Assayer::full_health_report()`. It returns the root-exported `SystemHealthReport`; `report.concordance.per_entity[entity]` reaches the inferred per-entity health value through the detailed query tier the health record defines (´dec:health:tiered-queries´).

The public `EntityConcordanceHealth` value exposes exactly `label_count`, `concordance`, and `flagged`. It exposes no cause, diagnosis, corruption verdict, confidence in a cause, or recommended action. `ConcordanceHealth` adds map capacity and eviction accounting around those values but no causal classification.

The label-path producer computes one Boolean from `risk_target.signum() == rho_effective.signum()` before the label updates the models. Publication clones the bounded tracker, and the detailed health query projects it against the current aggregate AUC and the configured minimum-label and deficit values. The completed health-record entry describes this same producer and query path (´entry:health:per-entity-concordance´).

Two focused unit witnesses already keep the arithmetic and storage mechanics: one proves flagging at the gate together with least-recently-observed eviction (´claim:wellness:per-entity-concordance-flags-at-the-gate-and-evicts-the-least-recently-observed´), and one proves the configured gate and strict deficit boundary (´claim:wellness:per-entity-concordance-honours-the-configured-gate-and-strict-deficit´). Neither asks what a host can learn about cause from the public type.

The nearest label-path integration witness accepts an inverted host valence convention without an integrity check (´test:integration:inverted-valence-signs-accepted-without-corruption´), but it neither raises nor reads per-entity concordance. The nearest detailed-health pattern drives a labelled stream and reads nested diagnostics through `full_health_report` (´test:integration:association-separates-a-keyed-sentinel-from-a-hash-fed-one´), but it concerns Sentinel legibility rather than causal neutrality.

The completed harness skeleton supplies `World`, `WorldBuilder`, `LabelSpec`, deterministic seeds, assess-label cycles, label flushing, and the raw `Assayer` escape hatch (´entry:assayer:harness-stage-skeleton´). None is needed to prove an unavailable public capability: a runtime scenario would have to invent two causal stories outside the input and could only rediscover that identical inputs produce identical fields.

The package declares no UI compile-fail runner or compile-failure dependency, and its integration suite contains no UI fixture directory. The standing testing plan has no entry covering this promise.

## The witness · `sec:assayer:intent-a-concordance-flag-names-a-discrepancy-without-attributing-a-cause-witness`

The witness belongs in a focused concordance-surface integration driver with one compile-fail fixture beneath the integration suite. Its test documentation cites the intent, and the module index repeats the witness statement.

- Setup: the UI fixture imports `EntityKey` and `SystemHealthReport`, then declares a function taking borrowed values of those two public types. It constructs no engine and executes no code.

- Positive surface probe: the function reaches `report.concordance.per_entity[entity]` and reads `label_count`, `concordance`, and `flagged`, so every component that names the discrepancy must type-check before the forbidden expression is reached.

- Stimulus: the final expression attempts to read `.cause` from that same inferred per-entity value, expressing exactly the causal capability the public diagnostic declines to provide.

- Observation: compilation fails at that expression with the unknown-field diagnostic for `cause`; the checked diagnostic identifies the three available discrepancy fields, which makes the failure specific to the missing causal member rather than to an import, an inaccessible type, or an unrelated syntax error.

- Assertion: a diagnostic-checking UI runner requires that fixture to fail for the recorded reason. The ordinary compile of the driver and the successful earlier field projections keep the accessible half of the surface live.

- Isolation: the fixture contains no labels, learned parameters, random samples, clocks, waits, floating-point comparisons, or feature-gated test-support API. The witness asks only what a host can express against the shipped public type.

The fails-before is deliberately inverted. A weakened `EntityConcordanceHealth` with a public `cause` member makes the forbidden fixture compile, so a runner expecting compilation failure turns red even though all existing numerical concordance tests remain green.

## What is missing · `sec:assayer:intent-a-concordance-flag-names-a-discrepancy-without-attributing-a-cause-missing`

**Entry (A reason-specific UI compile-fail harness)** · `entry:assayer:intent-a-concordance-flag-names-a-discrepancy-without-attributing-a-cause-ui-harness`

Approximately forty lines across the package dev-dependency declaration, the focused integration driver, the source fixture, and its checked diagnostic add a `trybuild` compile-fail case whose only intended type error is the attempted `.cause` access. The entry depends on no production change, no `World` extension, no clock or liveness support, and no other intent lane; generated test indexes and the integration matrix move with the new test in the ordinary test-document update.

## Risks and open questions · `sec:assayer:intent-a-concordance-flag-names-a-discrepancy-without-attributing-a-cause-risks`

**Observation (The population comparator is rank discrimination)** · `obs:assayer:intent-a-concordance-flag-names-a-discrepancy-without-attributing-a-cause-population-comparator`

The entity statistic is sign agreement while the comparator is aggregate AUC. The witness preserves that specified asymmetry and does not restate the flag as a difference between two concordance fractions; the existing unit coverage, not the type-shape probe, owns the numerical comparison.

**Observation (The diagnostic snapshot follows the Rust toolchain)** · `obs:assayer:intent-a-concordance-flag-names-a-discrepancy-without-attributing-a-cause-diagnostic-stability`

A checked compiler diagnostic is intentionally stricter than a bare `compile_fail` doctest and can move when the pinned Rust toolchain changes its rendering. Such churn is mechanical and visible, while a doctest that accepts any compilation error could stay green after the intended field-access refusal stopped being the reason.

**Observation (The field inventory is the semantic ratchet)** · `obs:assayer:intent-a-concordance-flag-names-a-discrepancy-without-attributing-a-cause-field-inventory`

The outer system report remains open to additive health fields as decided by (´dec:health:non-exhaustive-report´). The witness pins only the per-entity diagnostic's current inventory, where any new member deserves an explicit reconsideration of whether it is another measured property or a causal attribution; it does not freeze unrelated health domains.

**Observation (No statistical fixture can manufacture causality)** · `obs:assayer:intent-a-concordance-flag-names-a-discrepancy-without-attributing-a-cause-runtime-free`

Systematic mislabelling and genuine unpredictability are interpretations of how the same observable sequence arose, not two fields accepted by the label API. Naming two runtime entities after those interpretations would not give the engine evidence of either cause, so equality between their reports would test fixture symmetry rather than the promise. The compile-time refusal tests the actual boundary.

No numerical, timing, scheduling, or nondeterministic question remains open. No maintainer decision is required by the current public shape and compiler-test convention selected here.

## Acceptance · `sec:assayer:intent-a-concordance-flag-names-a-discrepancy-without-attributing-a-cause-acceptance`

The implementing lane's report identifies the UI driver, source fixture, checked diagnostic, and dev dependency; confirms the driver documentation cites the intent; and shows the generated module index, integration matrix, and coverage report resolving the intent to the compile-time witness.

The report shows the positive projections of `label_count`, `concordance`, and `flagged` compiling, then shows the fixture rejected specifically at `.cause` by the expected unknown-field diagnostic. It states the inverted fails-before: adding a public causal member makes the forbidden program compile and therefore makes the compile-fail test fail; no executed mutation is required.

The report shows the focused UI test and the package's prescribed formatting, lint, default-feature, all-feature, documentation, debug-test, and release-test gates passing, with unrelated failures separated explicitly.

Acceptance adds no runtime workload, tolerance, timing allowance, production hook, or causal enum. The test keeps both halves of the promise at the public boundary: the discrepancy fields remain readable and a causal diagnosis remains inexpressible.
