# A Witness for Conditioning-Growth Refactorisation · `plan:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation`

This plan resolves what a test keeping (´claim:bayes:a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation´) has to distinguish: conditioning brings a covariance rebuild forward before the label counter does, and the adopted rebuild leaves the precision and covariance pair within its synchronisation ceiling.

## What the promise says precisely · `sec:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation-promise`

The promise as written has an absolute trigger and a postcondition. Before the configured recomputation interval expires, a conditioning estimate becomes strictly greater than $10^5$; that crossing causes one Cholesky refactorisation; and the refactorisation leaves $\lVert B\tilde{\Sigma}-I\rVert_F$ near zero.

The specification now separates the two conditioning quantities (´def:monitoring:condition-number´). The per-label quantity is $\rho(B)=\max_j B_{jj}/\min_j B_{jj}$, an $O(p)$ lower bound on the spectral condition number rather than an estimate of it; the true $\kappa(B)$ is measured from the extreme eigenvalues only while a refactorisation is already running, and the public names preserve that distinction (´req:monitoring:conditioning-bound-named´).

The configured quantities are $N_\text{recompute}$, whose default is one thousand labels, and $\kappa_\text{growth}$, whose default is two (´tab:config:risk-model´). The current trigger is therefore measurable as $\rho_\text{now}>\kappa_\text{growth}\rho_\text{baseline}$ while the absorbed-label count remains below $N_\text{recompute}$, not as $\rho_\text{now}>10^5$ (´alg:gaussian:condition-adaptive-recompute´).

The postcondition uses the specified synchronisation error and its width-scaled threshold $10^{-6}p$ (´def:monitoring:synchronisation-error´). A successful witness sees a clean factorisation, adoption of the precision and covariance together, an after-reading no worse than the before-reading and below that ceiling, and no alarm (´dec:posterior:measured-adoption´).

## What the code offers today · `sec:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation-current-surface`

The production path is `Assayer::label` through the label path: every accepted label updates each Bayesian model, evaluates `should_recompute`, and, when an arm fires, runs `synchronisation_visit` before applying its outcome. The conditioning-growth and changed-floor-count arms bypass a good pre-rebuild drift verdict because they report that the precision matrix has moved away from the spectrum last certified (´dec:posterior:recomputation-trigger´).

The host's `CholeskyConfig` exposes `n_recompute` and `kappa_growth_factor`, so an integration fixture can leave a wide counter interval while retaining the relative conditioning policy (´tab:config:risk-model´). `PrecisionHealthDetail` exposes the target model's `kappa`, `diagonal_ratio`, `last_measurement_labels`, `cholesky_recomputes`, `last_sync_error`, `last_sync_error_after`, `last_rebuild_verdict`, `last_rebuild_adopted`, `n_recompute_effective`, `dimensions_at_floor`, and alarm counters through `Assayer::full_health_report`.

The completed harness skeleton supplies configured `World` construction, named channels and axes, deterministic request generation, `LabelSpec`, label submission, `World::flush_labels`, the engine escape hatch for health reports, seeded randomness, and the liveness watcher (´entry:assayer:harness-stage-skeleton´). Its tolerance bundle already carries the specified per-dimension synchronisation coefficient (´tab:assayer:harness-scenario-tolerances´).

The closest public-surface instrument is the conditioning-floor sweep (´test:integration:the-floor-moves-the-condition-estimate-and-the-recompute-count-but-not-the-error´); it already constructs the dense competitive geometry, drives labels, flushes the label path, and samples the relevant precision-health fields. It is a long-running parameter study whose assertion only proves that the run reached a minimum geometry, so it neither isolates a trigger transition nor asserts the refactorisation's after-reading.

The dense factorisability instrument adds a bounded progress watcher and proves that the same geometry reaches factorisations without a cascade terminus (´test:integration:a-dense-competitive-schedule-leaves-every-precision-matrix-factorisable´). The crate-level recomputation suite separately proves that the relative arms fire on their own quantities (´test:crate:the-relative-arms-fire-on-their-own-quantities´) and that an isolated highly conditioned model can adopt every rebuild (´test:crate:an-isolated-model-at-the-schedules-conditioning-adopts-every-rebuild´); the missing coverage is their live-label-path composition.

The standing gap register has no Entry covering this promise (´sec:assayer:testing-plan-gaps´).

## The witness · `sec:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation-witness`

The code-aligned witness is one integration test over one identified `ModelId`, using the shared dense-conditioning fixture extracted from the two existing instruments. It configures a long $N_\text{recompute}$, fixes the seed and clock, registers the fixture's outcome axis and identity dimensions, enables the liveness watcher, and drives public assess-label-flush cycles until the target model has one clean adopted rebuild that establishes its baseline.

The setup records that baseline's diagonal ratio, recomputation count, effective interval, dimensions-at-floor count, and true condition number. A valid starting point has a finite positive baseline ratio, no alarm, and enough interval remaining that the next transition can occur strictly before the counter arm.

The stimulus continues the same deterministic stream one label at a time and samples the target after every flush. It stops at the first publication whose diagonal lower bound is greater than both $10^5$ and $\kappa_\text{growth}$ times the baseline, while the dimensions-at-floor count is unchanged and the labels absorbed since the baseline remain below the effective interval.

The observation is the single transition bracketing that label: the preceding sample has the old recomputation count and a ratio at or below the relative boundary; the following sample has exactly one additional Cholesky recomputation, a newly measured true condition number, a clean verdict, an adopted pair, and no additional alarm. The unchanged floor count and unexpired counter exclude the other ordinary trigger arms, so the remaining cause is conditioning growth.

The numerical assertion compares `last_sync_error_after` with the target model's width-scaled ceiling rather than with an invented epsilon. It also requires the after-reading to be no worse than the before-reading, which ties the near-zero result to the pair the model actually adopted rather than to a discarded factorisation (´dec:posterior:measured-adoption´).

One further label is a reset control. Its fresh trigger diagnostic remains above $10^5$ while the recomputation count stays unchanged, because the visit reset the relative baseline; this is the observable distinction between the current relative trigger and the retired permanently-satisfied absolute trigger.

The fails-before for a deliberately counter-only implementation is a ratio that clears the relative boundary while `cholesky_recomputes` stays unchanged until $N_\text{recompute}$. The fails-before for a deliberately weakened rebuild is an incremented recomputation count paired with `last_rebuild_adopted == Some(false)`, an alarm, or an after-reading outside the width-scaled ceiling.

## What is missing · `sec:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation-missing`

**Entry (The trigger contract has one meaning)** · `entry:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation-trigger-contract`

The intent's fixed $10^5$ cause conflicts with the current specification, record, implementation, and a direct crate witness: conditioning at an absolute ratio of a million million calls for nothing when it has not grown relative to its baseline (´claim:bayes:no-absolute-conditioning-reading-calls-for-a-rebuild´), kept by (´test:crate:conditioning-alone-never-calls-for-a-rebuild´). The code-aligned alternative restates the intent around growth of the diagonal lower bound, retaining $10^5$ as a fixture regime rather than as the cause; the literal alternative introduces an absolute policy with a reset or hysteresis rule that prevents the same unchanged matrix from firing forever. The first alternative is approximately fifteen prose lines; the second is approximately eighty to a hundred and forty lines across configuration, trigger logic, records, and specification before any test. Neither depends on a sibling lane.

**Entry (A deterministic conditioning stream is shared)** · `entry:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation-conditioning-fixture`

A fixture in the shared testing harness factors the duplicated encoders, identity registrations, label cycle, target-model selection, bounded progress, and drive-until predicate out of the two public-surface instruments cited above. It returns consecutive precision-health samples and fails explicitly when the geometry reaches its wall without producing the required baseline and isolated conditioning-growth transition. The estimate is approximately a hundred and twenty to a hundred and eighty lines, with corresponding deletions available in those instruments; it depends on the trigger-contract entry and on no sibling lane.

**Entry (A precision checkpoint carries its own ceiling)** · `entry:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation-precision-checkpoint`

The public detail exposes the after-reading but not the model width needed to evaluate $10^{-6}p$. A test-support checkpoint in the shared testing harness pairs one model's `PrecisionHealthDetail` with the current feature width and the ceiling computed through `Tolerances::precision_sync_ceiling`, without exposing either matrix. The estimate is approximately thirty to fifty lines; it depends on no sibling lane and lets the witness use the same threshold the existing scenario assertions declare.

## Risks and open questions · `sec:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation-risks`

**Observation (An absolute threshold is the rejected policy)** · `obs:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation-absolute-threshold-conflict`

The current algorithm states that an absolute conditioning threshold is permanently true after a rebuild because rebuilding the covariance does not change the precision matrix (´alg:gaussian:condition-adaptive-recompute´). The public-surface evidence records the resulting refactorisation on every label (´rep:assayer:synchronisation-drift-study´). A test that merely arranges for $\rho(B)>10^5$ and observes a rebuild would therefore be vacuous under the relative policy and harmful under the literal one; the trigger-contract entry is a real prerequisite rather than wording polish.

**Observation (The competing arms can counterfeit causality)** · `obs:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation-competing-arms`

The counter, changed-floor-count, and degenerate-diagonal arms can all produce the same public recomputation increment. Consecutive snapshots, a baseline established by an adopted rebuild, a strictly unexpired counter, stable floor count, and finite positive diagonals are the evidence that attributes the transition to conditioning growth.

**Observation (The geometry is deterministic in stimulus and variable in trajectory)** · `obs:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation-numerical-trajectory`

The seeded stream does not fix parallel floating-point reduction order or the label at which competitive cells publish. The witness consequently waits for inequalities and a bounded transition instead of pinning a label ordinal, a condition number, or an exact after-reading; failure to reach the state is a fixture failure with the last checkpoint in its message.

**Observation (The witness belongs outside the ordinary fast tier until measured)** · `obs:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation-runtime-tier`

The existing dense public fixtures run for minutes because they grow matrices into the thousands. Extraction may permit a narrower geometry that crosses the relative boundary, but the implementation report determines from measured wall time whether the test runs by default or carries the same explicit ignored-test contract as the instruments it replaces.

## Acceptance · `sec:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation-acceptance`

The implementation report identifies which trigger contract is in force and cites the matching specification and record. Under the code-aligned contract, the intent text names the diagonal lower bound and relative growth while treating $10^5$ only as a scenario precondition; under the literal contract, the report includes the reset semantics that prevent permanent firing.

The report shows the target model, baseline ratio, crossing ratio, growth factor, labels absorbed against the effective interval, unchanged dimensions-at-floor count, recomputation counts on both sides of the transition, true condition number after the rebuild, verdict, adoption, alarm delta, before-reading, after-reading, model width, and synchronisation ceiling.

The report shows that the conditioning transition precedes the counter, causes exactly one rebuild, leaves the next-label reset control quiet, and installs a pair whose after-reading is no worse than its before-reading and below $10^{-6}p$.

The test module's index and the integration matrix name the landed witness, and its failure output carries the final checkpoint when the fixture misses a precondition. The report includes the isolated test command, its exit status and wall time, the package test result, and whether the measured cost places the witness in the default or ignored tier.
