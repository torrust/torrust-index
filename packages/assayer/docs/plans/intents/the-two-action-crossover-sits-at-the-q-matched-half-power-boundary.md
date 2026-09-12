# Keeping the Two-Action Q-Matched Half-Power Crossover · `plan:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary`

This plan keeps the promise that an Allow/Block channel has one policy crossover and renders both boundary actions at the Q-matched half-power offsets fixed by the specification (´claim:resonance:the-two-action-crossover-sits-at-the-q-matched-half-power-boundary´).

## What the promise says, precisely · `sec:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-promise`

The declared action set is the ordered list the host supplies, and Allow followed by Block is a valid set because any ordered subset with at least two actions is admitted (´def:channel:actions´) and the derivation record requires one generic path over the adjacent pairs it is handed (´dec:derivation:ordered-actions´).

Write $J=2$ for that action count. The landscape then contains exactly $J-1=1$ crossover and $J=2$ regimes (´sig:landscape:output´), with the sole crossover record naming Allow as `from` and Block as `to` (´def:landscape:action-crossover´).

Write the crossover logit as $\ell^*=u+b_{\mathrm{Allow}\to\mathrm{Block}}(\hat q_c)$ and its posture as $\pi^*=\sigma(\ell^*)$. The shared risk term $u$, the reward-and-challenge offset $b$, and their closed-form sum are fixed by (´eq:landscape:rigid-decomposition´), while the logit-to-posture mapping is fixed by (´eq:landscape:crossover´).

The rendering computes each boundary bandwidth as $Q_a=\max(c_Q/\sqrt{2\operatorname{Var}(\ell^*)},Q_{\mathrm{floor}})$ (´def:rendering:bandwidths´). Because both actions share the same sole crossover, the unfloored expression and the floor are identical for Allow and Block.

With no interior action, the placement reduction is $\ell(\mu_{\mathrm{Allow}})=\ell^*-1/Q_{\mathrm{Allow}}$ and $\ell(\mu_{\mathrm{Block}})=\ell^*+1/Q_{\mathrm{Block}}$ (´alg:rendering:placement´). A fixture that keeps the drawable-interval constraints idle therefore observes the closed form directly rather than an endpoint-constrained substitute.

At the crossover, each action is one reciprocal bandwidth from its own peak, so the Cauchy kernel gives $f_a(\ell^*)=A_a/(1+Q_a^2(1/Q_a)^2)=A_a/2$ (´eq:rendering:kernel´). The measurable half-power assertion is per action and accompanies the exact negative and positive logit offsets.

The comparison uses the harness's named crossover tolerance, whose current declared value is $10^{-6}$ and whose standing is explicitly a harness reading of the specification's matching condition (´tab:assayer:harness-scenario-tolerances´).

## What the code offers today · `sec:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-current-code`

The public host path separates the decision landscape from optional rendering as required by (´dec:landscape:rendering-optional´): `derive_landscape` returns the crossover-bearing `DecisionLandscape`, and `render_resonances` takes that landscape, its `RiskBasis`, and a `ResonanceConfig` under the public rendering contract (´sig:rendering:contract´).

The public `DerivedReckoning` exposes both `landscape` and `profile`, so an integration test can compare the decision boundary and its display without private-module access. The Core ends at the risk assessment and the composition remains host-side (´dec:ordering:core-boundary´).

The concrete two-action rendering branch implementing (´alg:rendering:placement´) reads `landscape.crossovers[0]`, subtracts and adds the reciprocal action bandwidths, maps both logits through the stable sigmoid, and then applies the drawable-interval constraints. That branch is deterministic because rendering is a pure transform over explicit arguments (´dec:derivation:pure-transform´).

The completed harness skeleton supplies `scenario_with`, `WorldBuilder::channel`, `World::derive_for_request`, deterministic seeds, and named tolerances (´entry:assayer:harness-stage-skeleton´). Its assertion surface already supplies `assert_near` and `find_tag`, while `numerics_export` exposes `stable_logit` and `stable_sigmoid` to integration tests.

The nearest crate witness proves only that the two action tags exist, remain live, and appear on opposite sides (´test:crate:two-action-channel´). It does not compare either offset with $1/Q$ or evaluate either kernel at the crossover.

The general crossover witness compares adjacent boundary and interior kernels under the ordinary action set (´test:crate:crossover-matching´). It does not enter the special two-action branch, where neither tag is interior and the specified result is half power rather than a magnitude-ratio match.

The nearest integration coverage compares two-action magnitude with a richer channel and leaves both crossover count and placement unobserved (´test:integration:action-regime-widths-follow-channel-policy-without-moving-core´). The standing testing plan has no gap entry covering this promise.

## The witness · `sec:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-witness`

The witness belongs beside the existing public-surface geometry scenarios, including the action-set comparison at (´test:integration:three-action-challenge-at-least-as-strong-as-four-action´), and its module index carries the same measurable statement as its test documentation.

- Setup: build one seeded `Scenario` with `scenario_with`, register a channel whose `ChannelPolicy.actions` is `[Action::Allow, Action::Block]`, and derive one cold request through `World::derive_for_request`.

- Structural observation: require exactly one landscape crossover, require its transition to be Allow-to-Block, require exactly two regimes in that order, and select exactly the Allow and Block action tags from the profile.

- Controlled rendering stimulus: read the positive finite variance of the sole crossover and render the same landscape twice, choosing $c_Q=Q_{\mathrm{target}}\sqrt{2\operatorname{Var}(\ell^*)}$ for distinct target bandwidths of one and two while leaving $Q_{\mathrm{floor}}$ below both targets.

- Bandwidth oracle: independently compute $Q_{\mathrm{expected}}=\max(c_Q/\sqrt{2\operatorname{Var}(\ell^*)},Q_{\mathrm{floor}})$ and require both action tags to report it within the named crossover tolerance.

- Placement oracle: compute the expected logits $\ell^*\mp1/Q_{\mathrm{expected}}$, map them through `stable_sigmoid`, first require those raw locations to lie inside Allow's and Block's drawable intervals, and then compare them with the reported locations.

- Half-power oracle: evaluate each tag's public `kernel_at_logit` at $\ell^*$ and compare it with half that tag's own magnitude; also compare `stable_logit(tag.location)-crossover.logit` with the signed reciprocal-Q offset so a kernel implementation error cannot make two wrong quantities agree.

- Isolation: hold the landscape and risk basis fixed across the two renders. The crossover remains the sole decision point while only Q changes, demonstrating that the tag peaks follow Q rather than a neighbouring-regime endpoint or a fixed displacement.

- Health observation: finish with `assert_health_clean`; no labels, lifecycle operations, clock advances, liveness waits, or cross-assessment comparisons enter the scenario.

The fails-before is a two-action branch that places the tags at the crossover, reuses a fixed offset, borrows an endpoint from the multi-action algorithm, or applies the reciprocal of the wrong Q. Such an implementation can preserve tag count, order, positivity, and opposite-side placement, leaving the existing witnesses green, but at least one target-Q render fails the logit-offset and half-power assertions.

## What is missing · `sec:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-missing`

No shared harness capability or other lane blocks this witness. The missing material is confined to the existing integration module and composes from the completed skeleton.

**Entry (The Q-normalised two-action fixture)** · `entry:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-q-normalised-fixture`

An inline setup block of roughly twenty lines builds the two-action scenario, captures the public reckoning, validates the sole crossover's finite positive variance, and constructs the two rendering configurations from that variance. It depends on (´entry:assayer:harness-stage-skeleton´) and on no other intent lane.

**Entry (The public half-power witness)** · `entry:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-integration-witness`

A focused integration test of roughly forty lines adds the structural, bandwidth, placement, half-power, two-Q liveness, and clean-health assertions described above. It depends on (´entry:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-q-normalised-fixture´) and requires no new production or shared-harness surface.

## Risks and open questions · `sec:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-risks`

**Observation (Half power is per tag, not an action-probability tie)** · `obs:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-half-power-is-per-tag`

The two boundary magnitudes are the two posture-side regime masses and need not agree. Reciprocal-Q placement makes each kernel equal half of its own peak at $\ell^*$; it does not imply equal Allow and Block contributions unless their magnitudes also agree. The witness avoids strengthening the promise into that unsupported equality.

**Observation (The location constraints stay outside the oracle)** · `obs:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-clamps-stay-idle`

The rendering constrains Allow and Block peaks to separate drawable intervals after applying the closed form (´alg:rendering:placement´). The normalized target bandwidths keep the expected locations away from those constraints, and explicit fixture-precondition assertions distinguish an unsuitable fixture from a placement defect.

**Observation (Numerical error is local and bounded)** · `obs:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-numerics-are-local`

One cold assessment supplies the held inputs and both comparisons occur in logit space before a stable sigmoid round trip. The fixed seed, virtual clock, lack of training, and reuse of one landscape remove timing and scheduling variation; the crossover tolerance covers only ordinary floating-point evaluation.

**Observation (The witness isolates placement from covariance)** · `obs:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-placement-isolated-from-covariance`

Choosing $c_Q$ from the reported crossover variance normalizes the bandwidth and intentionally does not re-prove the two-source variance formula. The test still computes the specified bandwidth independently from configuration and landscape fields, while the promise under study remains the two-action placement reduction.

No maintainer decision remains open: the existing public surface, deterministic fixture, numerical bound, and integration-test home determine the witness completely.

## Acceptance · `sec:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-acceptance`

The implementing lane's report identifies the new public-surface resonance geometry scenario, confirms its module index and test documentation cite the intent, and shows the coverage report resolving the intent to that integration witness.

The report shows the targeted integration test passing and the package's prescribed formatting, lint, feature, documentation, and test gates passing, with any unrelated failure separated explicitly.

The reported assertions establish the Allow-to-Block transition, the single crossover and two regimes, both expected Q values, both signed reciprocal-Q offsets, both half-peak kernel readings, idle drawable constraints, and clean health.

The report states the fails-before in terms of the concrete two-action placement branch and names which offset or half-power assertion rejects a fixed, zero, endpoint-derived, or wrong-Q placement; no executed mutation is required.

Acceptance excludes new production hooks, timing allowances, statistical sampling, and shared fixtures because the witness is a one-shot deterministic read of the public decision and rendering surfaces.
