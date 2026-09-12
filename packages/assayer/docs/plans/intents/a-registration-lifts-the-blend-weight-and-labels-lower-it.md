# Registration creates and repays a blend-weight transient · `plan:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it`

This plan keeps the promise that registering a Sentinel into a converged deployment raises the anchor's blend weight because shared-subspace uncertainty has risen, while later eligible labels lower the weight toward a new residual (´claim:risk:a-registration-lifts-the-blend-weight-and-labels-lower-it´).

## What the promise says, precisely · `sec:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-promise`

Fix one reported request $x$ and write $s_i=x_g^\top\Sigma_{\mathrm{inh},g,i}x_g$ for the sister's raw quadratic form on the gathered anchor coordinates, $a_i=\tilde{x}^\top\Sigma_{\mathrm{anc},i}\tilde{x}$ for the anchor form, and $w_i=(1-a_i/(s_i+\varepsilon))^+$ for the anchor weight at checkpoint $i$ (´def:risk:subspace-blend´).

The pre-registration checkpoint is a steady-state fixture rather than a nominal label count: four consecutive checkpoints separated by $p$ eligible labels all have $0.02\leq w_i\leq0.10$, and their span is at most $0.01$. The interval is the specified deployment-dependent residual, while the promise's value near $0.05$ is its representative centre rather than a universal constant (´rem:risk:blend-mechanisms´).

Let $\delta=10\,\tau$, where $\tau$ is `World::tol().default` from the named scenario-tolerance bundle (´tab:assayer:harness-scenario-tolerances´). The immediate checkpoint follows the registration publication barrier with no intervening Sentinel report, assessment label, or clock advance, and it requires $s_1>s_0+\delta$, $|a_1-a_0|\leq\delta$, and $w_1>w_0+\delta$.

The recovery phase admits stationary eligible labels in blocks of $p$, where $p$ is the post-registration sister dimension. By the checkpoint at $10p$ labels, $s_N<s_1-\delta$ and $w_N<w_1-\delta$, the last four weights again span at most $0.01$, and every one lies in the residual interval $[0.02,0.10]$; the new residual need not equal the old one because it is emergent rather than configured (´dec:risk:anchor-floor´).

The $p$-scaled horizon is an upper fixture budget drawn from the approximate interaction-maturity boundary, not a new production deadline (´tab:warmup:stages´). The assertion concerns the sampled envelope and does not require every individual label to lower the weight.

The causal reading requires the component forms as well as $w$: a full-space uncertainty increase confined to the new Sentinel's coordinates cannot satisfy the shared-form assertion, which preserves the promise that Sentinel-specific uncertainty does not activate the anchor (´inv:guarantee:blend-subspace´).

## What the code offers today · `sec:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-code`

The public path is `Assayer::register_sentinel` followed by the ordinary assessment and label APIs. `World::register_sentinel` supplies a stable name, calls that public registration, and settles the model-owner publication before returning under the two-phase visibility contract (´dec:construction:two-phase-visibility´).

The lifecycle owner appends the Sentinel slot and interaction coordinates to every full-dimensional model, leaves the anchor fixed, extends standardisation, and marks an early calibration refit; the focused registration witness already keeps the dimension-growth half of that path (´test:unit:register-sentinel-extends-model´). The Bayesian model's `extend` method copies every old mean, precision, and covariance entry exactly and adds an independent prior block, implementing Sentinel registration (´alg:registry:sentinel-registration´), exact Gaussian extension (´thm:gaussian:extension´), and anchor invariance (´dec:substrate:anchor-invariant´).

The blend computation already calculates $s_i$, $a_i$, and $w_i$ by indexed projection, and its internal `BlendResult` retains all three; the component-variance witness establishes that those diagnostics are populated and mutually consistent (´test:unit:variance-diagnostics-populated´). The host-visible `RiskBasis` exposes `anchor_weight` but omits the sister and anchor variances carried by the specified risk basis (´schema:risk:basis´), so a public integration test can observe the trajectory but cannot yet establish its stated cause.

The world harness supplies configured worlds, virtual clocks, `settle_cold_ramp_with`, `assess`, `derive_for_request`, `cycle_request`, `LabelSpec`, and explicit label flushing. The report fixtures supply deterministic golden reports and reporting-Sentinel attachment, but the standing `LabelTape`, `ReportTape`, `World::converged()`, and snapshot-diffing stage remains open (´entry:assayer:harness-stage-tapes´).

The outcome-axis registration witness already demonstrates a before/barrier/after comparison of one risk basis through the public lifecycle path (´test:integration:outcome-axis-registration-does-not-perturb-risk-basis´). The scalar convergence witness supplies the nearest deterministic assess-label loop (´test:integration:scalar-signal-population-split-converges-directionally´), while the Sentinel association witness supplies a long public stream with a reporting Sentinel (´test:integration:association-separates-a-keyed-sentinel-from-a-hash-fed-one´).

The standing testing plan's open anchor-behaviour gap covers this promise because the missing evidence is a lifecycle-driven period in which the anchor becomes more useful and then yields as the sister learns (´entry:assayer:gap-anchor-behaviour´). The stage-tapes entry is its infrastructure dependency rather than a second behavioural obligation.

The present registration implementation cannot produce the immediate checkpoint defined above. Exact extension leaves $\Sigma_{\mathrm{inh},g}$ unchanged, the fixed anchor is untouched, a fixed probe supplies zeros in the new coordinates, and no update runs before publication; consequently $s_1=s_0$, $a_1=a_0$, and $w_1=w_0$.

## The witness · `sec:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-witness`

The witness belongs beside the scalar-signal convergence integration witness (´test:integration:scalar-signal-population-split-converges-directionally´) because its dominant cost and assertion are a labelled convergence trajectory, with the registration boundary kept as one explicit phase.

The steps below keep the promise's literal immediate-registration reading. Selecting either non-literal transition contract replaces the stimulus boundary and the component comparisons before the witness is implemented; it does not silently weaken these assertions.

- Setup: construct matched treatment and control worlds from one configuration and seed, register one incumbent reporting Sentinel in each, settle the cold standardisation ramp, and use one stationary alternating-valence tape whose labels are all sister- and anchor-eligible.

- Baseline: run both worlds in $p$-label blocks until the treatment's last four fixed-probe weights meet the residual and span conditions, then require the control to agree within $\delta$ and capture $(s_0,a_0,w_0)$ from the treatment.

- Stimulus: freeze both virtual clocks, register a second Sentinel only in the treatment through `World::register_sentinel`, issue no report or label for it, flush the control as a matched barrier, and immediately reassess the same incumbent-only probe.

- Immediate observation: capture $(s_1,a_1,w_1)$ and assert the shared-form lift, stable anchor form, and weight lift above. The unchanged control reading proves that an assessment or publication boundary alone did not create the movement.

- Recovery stimulus: make the new Sentinel report on every treatment request, replay the stationary eligible population in blocks of the new $p$, and take the identical fixed probe after each flushed block through the $10p$ horizon.

- Recovery assertion: require a material fall from the peak in both $s$ and $w$, a last-four-checkpoint span no wider than $0.01$, and all four terminal weights inside the specified residual band. Require finite assessments and clean health at completion so a numerical failure cannot masquerade as convergence.

- Formula guard: at every captured checkpoint, independently recompute $(1-a_i/(s_i+\varepsilon))^+$ and compare it with the public `anchor_weight` within $\tau$, so the diagnostic and host surface cannot drift apart.

The fails-before is the current exact-extension behaviour or a deliberately broken implementation that omits the registration response: the immediate shared form and weight remain equal to baseline. A full-covariance blend that wakes on prior uncertainty in the new slot fails the shared-form and formula guards, while a permanent lifecycle step passes the lift and fails the terminal decline and residual assertions.

## What is missing · `sec:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-missing`

**Entry (The registration transition has one algebraic contract)** · `entry:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-transition-contract`

Approximately twenty to one hundred and twenty lines across the risk and registry specification and, for the literal contract, the lifecycle implementation resolve the conflict between exact extension and an immediate shared-subspace lift. The selected rule either adds an explicit positive-definite widening of the sister's shared block after exact structural extension, moves the promised effect to the first eligible reporting-label block as a counterfactual leverage differential, or defines the lifecycle stimulus to include the first report; it depends on no other intent lane and gates every entry below.

**Entry (A read-only blend-component probe)** · `entry:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-blend-probe`

Approximately fifty to ninety lines expose $s$, $a$, and the blend guard used at one owner-consistent assessment boundary, either by completing the specified `RiskBasis` fields or by a test-gated read-only diagnostic in the shared test harness. It reuses `compute_blend` outputs rather than recomputing model internals, depends on the transition contract for the meaning of its checkpoint, and has no dependency on another intent lane.

**Entry (A stationary Sentinel trajectory tape)** · `entry:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-trajectory-tape`

Approximately ninety to one hundred and forty lines in the shared test harness provide paired seeded worlds, incumbent and newcomer golden reports, a repeatable eligible alternating-label population, $p$-sized block playback, fixed-probe checkpoints, and the four-reading stability predicate. This is a focused slice of (´entry:assayer:harness-stage-tapes´), depends on the blend probe, and does not require another intent lane to land first.

**Entry (The public registration-transient witness)** · `entry:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-integration-witness`

Approximately seventy to one hundred and ten lines beside the scalar-signal convergence integration witness (´test:integration:scalar-signal-population-split-converges-directionally´) add the indexed test documentation, matched baseline, isolated registration boundary, component and formula assertions, recovery stream, non-vacuity checks, and final health check. It depends on the transition contract, blend probe, and trajectory tape.

## Risks and open questions · `sec:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-risks`

**Observation (Exact extension and an immediate shared lift are incompatible)** · `obs:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-extension-conflict`

Three coherent contracts remain. A literal post-registration lift requires an explicit shared-block widening after structural extension and narrows the scope in which the old posterior is an exact marginal. Preserving exact extension moves the effect to later labels and establishes only a relative slowdown against a no-registration control. Treating registration plus the first report as one deployment event permits a directional quadratic-form lift, but establishes a report-shaped scenario rather than registration alone. The witness cannot encode all three without ceasing to state one promise.

**Observation (The immediate boundary excludes a first report)** · `obs:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-immediate-boundary`

The public registration record carries identity and name, while measurement arrives through a later report. Including that report changes aggregate anchor features, the standardised request direction, and the number of reporting Sentinels, so a before-and-after weight difference would not isolate a covariance response to registration. The literal arm therefore probes before any report and makes a production uncertainty transition visible if that arm is selected.

**Observation (The residual band binds, while the stability width is a scenario discriminator)** · `obs:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-thresholds`

The two-to-ten-percent residual comes from the blend analysis; the $0.01$ four-checkpoint span and the ten-tolerance separation guard belong to this deterministic fixture. The implementing lane records the observed margins before retaining those widths, because an unmeasured wider budget could admit no recovery and an unrealistically narrow one could turn harmless floating-point drift into the subject of the test.

**Observation (Recovery is an envelope, not per-label monotonicity)** · `obs:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-recovery-envelope`

Forgetting, leverage caps, standardisation bootstrap, and alternating outcomes can make adjacent checkpoint weights move in either direction even while evidence reduces the lifted uncertainty overall. The peak-to-terminal decrease and terminal window keep the promised transient without imposing a monotonicity rule absent from the model update (´prop:update:leverage-bound´) and Sentinel bootstrap (´alg:standardisation:sentinel-bootstrap´).

**Observation (Publication barriers replace timing assumptions)** · `obs:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-publication-order`

Registration publication and label publication are asynchronous engine operations but deterministic harness boundaries. Every reading follows `World::register_sentinel` or `flush_labels`, virtual time remains fixed across the immediate boundary, and no sleep, liveness deadline, random scheduler race, or health-statistics publication interval enters the oracle.

One semantic decision remains open: the transition-contract entry selects which of the literal lifecycle response, the post-label leverage response, or the registration-plus-report event the corpus intends to promise.

## Acceptance · `sec:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it-acceptance`

The implementing lane's report names the selected transition contract and the corresponding specification, intent, and production changes; it states why the two rejected event boundaries no longer define this promise.

The report identifies the integration witness and its intent citation, confirms that registration and all labels travel through public APIs, and identifies any test-gated diagnostic separately from the host-visible `RiskBasis` fields.

The reported assertions show a stable pre-registration residual, an isolated immediate shared-form and weight lift with an unchanged anchor form and matched control, agreement with the blend equation, a material peak-to-terminal decline, and a stable new residual inside the specified band.

The report names the unchanged-extension, full-space-activation, and permanent-step defects and the assertion that rejects each one; no executed mutation is part of the evidence.

The report shows the focused integration test, its nearest lifecycle and convergence neighbours, the package's required formatting and lint gates, and the corpus linter passing, with any unrelated failure separated explicitly.
