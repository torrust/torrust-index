# Keeping a Hibernating Sentinel's Own Weights · `plan:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights`

This plan keeps the promise that a hibernating Sentinel returns with its own aged model block, without moving learning retained by the live Sentinels or importing correlations with measurement surfaces that did not coexist with it (´claim:lifespan:a-hibernating-sentinel-returns-with-its-own-weights´).

## What the promise says, precisely · `sec:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-promise`

Let Sentinel $A$ hibernate at persistent time $t_0$ and steward label sequence $L_0$, and let it return at $t_1$ after $n=L_1-L_0$ accepted labels and $Δt=(t_1-t_0)$ hours. The two clocks are independent and compose as $d_m=\gamma_m^n\gamma_{t,\mathrm{core}}^{\Delta t}$ for each full-dimension model $m$ (´def:temporal:two-mechanisms´), while one combined factor is applied once per model (´dec:posterior:combined-factor´).

Write $S_A$ for $A$'s own slot positions at hibernation and $S'_A$ for the same semantic positions after re-registration. The archive contains the mean sub-vector $\mu_{S_A}$, the precision block $B_{S_AS_A}$, the covariance block $\Sigma_{S_AS_A}$, and the standardisation entries, but no rows or columns coupling $S_A$ to another block (´alg:registry:hibernation´).

For every restored model $m$, the measurable return is $\mu'_{S'_A}=\mu_{S_A}$, $B'_{S'_AS'_A}=d_m B_{S_AS_A}$, and $\Sigma'_{S'_AS'_A}=d_m^{-1}\Sigma_{S_AS_A}$. The mean is unchanged because lazy posterior decay widens confidence without moving the estimate (´alg:temporal:lazy-application´); “aged weights” therefore means the archived posterior block under this stated transformation, not multiplication of the mean itself.

The operational model uses `model.gamma_opr`, the sister model uses `model.gamma_inh`, and both use `temporal.gamma_t_core`. The fixture chooses distinct rates, a counted absence stream, and a non-zero clock advance so the two independently computed factors differ from one and from each other by more than ten times the generic scenario tolerance.

Restoration is expected only while the configured archive lifetime remains open, the returning slot has the archived layout, every archived precision is admissible, and the aged minimum precision remains at or above `model.lambda_floor` (´req:gaussian:prior-replenishment-floor´). The fixture holds those preconditions explicitly so a return at the prior is a failure of the promise rather than an accidental exercise of a specified refusal.

Let $K$ be the union of every position belonging to live Sentinel $B$ and Sentinel $C$, where $C$ is registered only after $A$ hibernates. Immediately before and after $A$ returns, $\mu_K$, $B_{KK}$, and $\Sigma_{KK}$ are bit-identical: registration first extends the old posterior as an exact marginal and changes no existing entry (´thm:gaussian:extension´).

Every cross-block entry $B'_{S'_AK}$ and $\Sigma'_{S'_AK}$ is exactly zero. In particular, the $A$–$C$ block has no evidence because the two surfaces never operated together, and the more general preservation limit discards every old coupling as well (´cav:limitation:hibernation´).

Scaled-block comparisons use `World::tol().default`, currently declared as the one-shot floating-point tolerance; unchanged blocks and zero cross-blocks use `World::tol().bit_identical`, which is an exact-equality requirement (´tab:assayer:harness-scenario-tolerances´). Non-vacuity guards require $A$'s archived block to differ from the prior, the absence stream to move $B$ or the $B$–$C$ coupling by more than ten generic tolerances, and the returning block to remain above the replenishment floor.

## What the code offers today · `sec:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-current-code`

The public lifecycle path already exposes `Assayer::hibernate_sentinel(SentinelId)` and `Assayer::register_sentinel(SentinelRegistration)`. Re-registration restores only when the host supplies the same identifier, and both calls publish their model phase asynchronously under the two-phase visibility contract (´dec:construction:two-phase-visibility´).

The lifecycle owner takes the archive record during registration, extends every full-dimension model at the prior, admits the record, and asks `WorkingCopy::restore_self_structure` to overwrite only the returning slot. The working snapshot computes one decay per model from the archived label sequence and persistent timestamp, scales precision and covariance reciprocally, leaves the mean unchanged, and writes standardisation only when at least one model restores; this is the completed archive contract (´entry:construction:hibernation´).

The completed harness skeleton supplies `scenario_with_config`, `World::assayer`, `World::clock`, `World::flush_labels`, golden reports, reported requests, `cycle_request`, `LabelSpec`, deterministic seeds, and named tolerances (´entry:assayer:harness-stage-skeleton´). The virtual clock already reaches the engine's persistent timestamp used by lifecycle submissions, so `World::clock().advance(...)` determines the hibernation interval without sleeping.

The nearest crate witness hand-seeds a block and proves operational and sister ageing against both clocks (´test:crate:hibernated-sentinel-returns-aged-on-both-clocks´). Its sibling establishes a coupling directly in the working model and proves restored precision and covariance cross-blocks are zero (´test:crate:restored-sentinel-cross-feature-blocks-are-zero´).

Those crate witnesses drive `handle_lifecycle_submission` directly and do not cover the public API, the asynchronous publication barriers, a Sentinel learning while another is absent, or a new Sentinel joining during the absence. The new witness composes those omitted facts without replacing the focused mechanism tests.

The nearest integration lifecycle witness exercises destructive registration and deregistration through `World` (´test:integration:register-deregister-round-trip´), while the nearest re-registration witness proves that the destructive path creates a fresh lifetime (´test:integration:root-destroyed-on-deregister-then-fresh-on-reregister´). Neither invokes the hibernating disposition or observes model parameters.

The published model snapshot contains $\mu$, $\Sigma$, and the dimension map but deliberately excludes $B$ (´dec:retention:precision-excluded´), and `World` currently exposes only a narrow standardisation reading from it. The standing testing plan assigns this promise to the open tapes, fixtures, and snapshot-diffing stage (´entry:assayer:harness-stage-tapes´); that entry covers the promise, and no separate standing gap entry does.

## The witness · `sec:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-witness`

The witness belongs beside the public lifecycle scenarios, and its module index carries the same measurable statement as its test documentation while the test cites the intent.

- Setup: build a seeded world with deliberately distinct `gamma_opr`, `gamma_inh`, and `gamma_t_core`, an archive lifetime longer than the planned interval, and a floor low enough that the chosen schedule cannot refuse the archived blocks.

- Initial learning: register reporting Sentinels $A$ and $B$, settle the cold standardisation ramp, and run a short alternating ground-truth stream whose requests contain both coordinates. Capture an owner-consistent probe and require $A$'s operational and sister blocks to be distinguishable from their priors before hibernation.

- Hibernation boundary: hibernate $A$ by name, retain its host-supplied identifier as dormant harness state, and wait on the lifecycle publication barrier. Capture the post-marginalisation state from which learning during the absence is measured.

- Absence stimulus: register reporting Sentinel $C$, which has never coexisted with $A$; advance the virtual persistent clock by the declared $\Delta t$; then run exactly $n$ alternating ground-truth cycles with $B$ and $C$ co-active and flush every accepted label.

- Absence observation: capture the pre-return probe, require the live $B$–$C$ subspace to have moved from its post-hibernation state by more than the non-vacuity threshold, and record its complete $\mu$, $B$, and $\Sigma$ blocks as the state re-registration must leave alone.

- Return stimulus: re-register dormant $A$ under its original identifier and declaration, then wait until the lifecycle publication is visible before taking the final probe.

- Ageing oracle: independently compute $d_{\mathrm{opr}}=\gamma_{\mathrm{opr}}^n\gamma_{t,\mathrm{core}}^{\Delta t}$ and $d_{\mathrm{inh}}=\gamma_{\mathrm{inh}}^n\gamma_{t,\mathrm{core}}^{\Delta t}$ from the fixture's declared count and clock advance. For each model, compare the returned mean, precision, and covariance slot blocks with the archived probe under the three equations above and require the result to differ materially from a prior block.

- Isolation oracle: compare the pre-return and final probes over the full $B$–$C$ subspace bit for bit, including its learned coupling, then require every precision and covariance entry between $A$ and $B$ or $C$ to be exact zero.

- Completion observation: require the original identifier to resolve live again, require one well-formed reported assessment through each Sentinel, and finish with clean health so parameter agreement cannot conceal a degraded lifecycle path.

The fails-before is an implementation that takes the hibernation record but extends $A$ at the prior. It preserves registry success, finite assessments, the survivors, and clean health, so the existing integration witnesses remain green, while the returned self-block fails both the archived-block equations and the non-prior guard. Applying only one clock, sharing one label rate across models, rewriting the survivor block, or restoring stale couplings is rejected by the factor-specific, bit-identity, or zero-cross-block assertions respectively.

## What is missing · `sec:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-missing`

**Entry (Identity-preserving hibernation verbs)** · `entry:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-hibernation-verbs`

Roughly forty to sixty lines in the named-Sentinel registry and world harness add `World::hibernate_sentinel(name)` and `World::restore_sentinel(name)`. Hibernation moves the name and identifier from the live registry into a dormant registry only after the public call succeeds; restoration submits the original registration, waits on `World::settle`, and moves the same pair back. This depends on (´entry:assayer:harness-stage-skeleton´), has no production change, and has no dependency on another intent lane.

**Entry (Owner-consistent model-block probe)** · `entry:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-model-block-probe`

Roughly ninety to one hundred and thirty lines in the testing harness and test-gated owner command plumbing add a synchronous, read-only `ModelBlockProbe`. It projects the current working copy's label sequence, dimension ranges, standardisation entries, and operational and sister $\mu$, $B$, and $\Sigma$ without exposing `WorkingCopy` itself or adding $B$ to the production snapshot. The owner answers after earlier commands and labels are settled and performs no decay, publication, or mutation. This is the snapshot-inspection slice of (´entry:assayer:harness-stage-tapes´), depends only on the single steward (´dec:concurrency:single-steward´), and is independent of other intent lanes.

**Entry (The public hibernation integration witness)** · `entry:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-integration-witness`

Roughly seventy to one hundred lines in the public lifecycle integration module add the setup, counted training phases, four probes, independent decay oracle, non-vacuity guards, restored-block assertions, survivor comparison, cross-block zero checks, and final public assessment described above. It depends on (´entry:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-hibernation-verbs´) and (´entry:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-model-block-probe´), while the existing golden report and `cycle_request` remove any need for a general `LabelTape` or `ReportTape` first.

## Risks and open questions · `sec:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-risks`

**Observation (Ageing changes confidence, not the posterior mean)** · `obs:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-ageing-semantics`

The intent's phrase “weights aged” can be read as multiplying $\mu$, but the specification and archive record settle the executable meaning: $\mu$ returns unchanged while $B$ scales down and $\Sigma$ scales up reciprocally. The witness states all three equations and rejects a mean multiplied by either decay factor.

**Observation (Semantic blocks move when the layout moves)** · `obs:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-semantic-blocks`

$A$ occupies an earlier range before hibernation and an appended range after $C$ joins. Every comparison resolves ranges by Sentinel identifier in the probe taken at that boundary; retaining numeric indices across snapshots would compare unrelated features and could pass while restoration was wrong.

**Observation (Exact and approximate assertions have different jobs)** · `obs:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-numerical-oracles`

The decay oracle contains exponentiation and block scaling, so it uses the generic one-shot tolerance and an independent arithmetic expression. Extension copies survivor entries and leaves new couplings as stored zeros, so widening those assertions to the same tolerance would hide a small but real mutation and is not justified.

**Observation (The fixture keeps every refusal path idle)** · `obs:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-refusal-preconditions`

Expiry, layout mismatch, an inadmissible archived matrix, and decay below the replenishment floor all specify a return at the prior rather than a failed registration. The fixture records the live layout, stays inside the configured lifetime, keeps spatial-axis membership unchanged, and checks the floor inequality before asserting restoration, making any violated precondition an invalid witness rather than evidence against hibernation.

**Observation (The combined scenario adds the missing evidence)** · `obs:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-compositional-scope`

The focused crate tests already hold ageing and zeroing in isolation. This witness earns the intent by placing real public label learning and a new registration between the archive and restore, proving that the returning, surviving, and newly admitted blocks coexist under one published lifecycle rather than repeating the lower-level tests with a different fixture.

No maintainer decision remains open: the specification fixes the preservation scope and ageing equations, the construction record fixes archive custody and refusal behavior, and the current harness determines the narrow missing inspection and lifecycle conveniences.

## Acceptance · `sec:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-acceptance`

The implementing lane's report identifies the new public lifecycle scenario, confirms its module index and test documentation cite the intent, and shows the coverage report resolving the intent to that integration witness.

The report identifies the two test-support additions, confirms the model probe is test-gated, synchronous, read-only, and absent from the production snapshot, and confirms hibernation and restoration preserve one host-supplied Sentinel identifier.

The reported assertions show non-prior archived blocks, distinct operational and sister decay factors, the expected unchanged means and reciprocally scaled precision and covariance blocks, absence learning that materially moved the $B$–$C$ subspace, bit-identical survival of that subspace across $A$'s return, exact zero $A$ cross-blocks, live post-return assessments, and clean health.

The report shows the targeted integration test and the existing focused hibernation crate tests passing under the package's prescribed debug, release, feature, lint, formatting, and documentation gates, with unrelated failures separated explicitly.

The report states the deliberately broken prior-reset behavior and names the archived-block assertion that rejects it; it also names the assertions that reject a missing clock factor, a shared model rate, survivor mutation, and stale cross-terms. No executed mutation is required.
