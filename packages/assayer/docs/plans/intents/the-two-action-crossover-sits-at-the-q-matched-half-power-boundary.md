# Keeping the Two-Action Q-Matched Half-Power Crossover · `plan:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary`

This plan keeps the promise that an Allow/Block channel has one policy crossover and renders both boundary actions at the Q-matched half-power offsets fixed by the specification (´claim:resonance:the-two-action-crossover-sits-at-the-q-matched-half-power-boundary´).

## What the promise says, precisely · `sec:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-promise`

The declared action set is the ordered list the host supplies, and Allow followed by Block is valid because any ordered subset with at least two actions is admitted (´def:channel:actions´) and the derivation uses one generic path over the adjacent pairs it is handed (´dec:derivation:ordered-actions´).

Write $J=2$ for that action count. The landscape contains exactly $J-1=1$ crossover and $J=2$ regimes (´sig:landscape:output´), with the sole crossover record naming Allow as `from` and Block as `to` (´def:landscape:action-crossover´).

Write the crossover logit as $\ell^*=u+b_{\mathrm{Allow}\to\mathrm{Block}}(\hat q_c)$ and its posture as $\pi^*=\sigma(\ell^*)$. The shared risk term $u$, the reward-and-challenge offset $b$, and their sum are fixed by (´eq:landscape:rigid-decomposition´), while the logit-to-posture mapping is fixed by (´eq:landscape:crossover´).

For either boundary action the rendering computes $Q_a=\max(c_Q/\sqrt{2\operatorname{Var}(\ell^*)},Q_{\mathrm{floor}})$ (´def:rendering:bandwidths´). Both actions use the same sole crossover, so they have the same raw bandwidth and the same floored bandwidth.

With no interior action, the placement reduction is $\ell(\mu_{\mathrm{Allow}})=\ell^*-1/Q_{\mathrm{Allow}}$ and $\ell(\mu_{\mathrm{Block}})=\ell^*+1/Q_{\mathrm{Block}}$ before the boundary locations are constrained to their drawable intervals (´alg:rendering:placement´). A fixture that proves both raw locations lie strictly inside those intervals observes the closed form rather than an interval-constrained substitute.

At the crossover, each action is one reciprocal bandwidth from its own peak, so the Cauchy kernel gives $f_a(\ell^*)=A_a/(1+Q_a^2(1/Q_a)^2)=A_a/2$ (´eq:rendering:kernel´). The assertion is half of each action's own magnitude, not equality between the two actions.

Bandwidth and signed-offset comparisons use the harness's one-shot `default` tolerance of $10^{-9}$; the half-power kernel comparison uses its `crossover` tolerance of $10^{-6}$, whose standing is the harness's reading of the specification's matching condition (´tab:assayer:harness-scenario-tolerances´).

## What the code offers today · `sec:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-current-code`

The public host path separates the decision landscape from optional rendering as required by (´dec:landscape:rendering-optional´): `derive_landscape` returns the crossover-bearing `DecisionLandscape`, and `render_resonances` takes that landscape, its `RiskBasis`, and a `ResonanceConfig` under the public rendering contract (´sig:rendering:contract´).

The public `DerivedReckoning` exposes its `assessment`, `landscape`, and `profile`, so an integration test can reuse the owned `RiskBasis` and `DecisionLandscape` from one assessment while rendering two display configurations. The Core ends at the risk assessment and this composition remains host-side (´dec:ordering:core-boundary´).

The concrete two-action branch reads the sole crossover, subtracts and adds the reciprocal action bandwidths, maps both logits through `stable_sigmoid`, and then constrains the locations to the drawable intervals. `TagResonance::kernel_at_logit` exposes the unnormalised Cauchy contribution needed for the half-power reading, and the rendering remains a pure transform over explicit arguments (´alg:rendering:placement´) (´dec:derivation:pure-transform´).

The finished common harness supplies `Scenario`, `scenario_with`, `WorldBuilder::channel`, `World::derive_for_request`, `World::tol`, `assert_near`, `find_tag`, `assert_health_clean`, deterministic seeds, and the `numerics_export` forms of `stable_logit` and `stable_sigmoid`; the scenario owns a real engine and one common vocabulary (´dec:harness:single-scenario´) (´dec:harness:real-engine´) (´tab:assayer:harness-implementation-library-roster´).

The same finished roster supplies forward-only `World::advance` and `World::travel_to`, the queue-specific `World::flush_observations`, `World::flush_labels`, and `World::flush_identity_maintenance` barriers, the owned `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView` probes, the `PlaybackRow`, `PlaybackBarrierPolicy`, `PlaybackCheckpoint`, `PlaybackProgress`, and `playback` stimulus surface, the `OracleProvenance`, `pairwise_rank`, `regularised_schur_complement`, `decay_recurrence`, and `dimension_width` oracle tier, the guarded `TrainedStateFixture` and `TrainedStateBaseline`, and `run_seeded_sweep` (´dec:harness:no-ad-hoc-waits´) (´dec:harness:probe-contract´) (´dec:harness:declarative-playback´) (´dec:harness:oracle-tier´) (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´) (´dec:harness:seeded-sweeps´) (´tab:assayer:harness-implementation-library-roster´).

This witness needs none of those stateful extensions: it moves no time, waits on no queue, reads no retained state, trains no model, drives no long stimulus, and quantifies over no seeded input space. Its result is already an owned public reckoning, and the shared oracle tier has no rendering-placement oracle; the one-caller expectations therefore remain direct test-local evaluations of the specification rather than a new shared helper (´dec:harness:oracle-tier´).

The landscape-shape witness already proves that a two-action declaration produces one crossover and two boundary regimes (´test:crate:landscape-shape-is-stable´). The tag-shape witness proves only that the Allow and Block tags are live and lie on opposite sides (´test:crate:two-action-channel´); it describes the half-power reason but asserts neither reciprocal-Q offset nor kernel value.

The general crossover witness compares each outer action with its neighbouring interior action under the ordinary action set (´test:crate:crossover-matching´). It does not enter the special two-action branch, where both tags are boundary tags and the specified result is half of each tag's own magnitude rather than equality between neighbouring kernels.

The nearest integration coverage compares two-action magnitude with richer channels and leaves crossover placement and half-power unobserved (´test:integration:action-regime-widths-follow-channel-policy-without-moving-core´). The standing testing plan has no gap entry covering this promise.

## The witness · `sec:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-witness`

The witness belongs in the `resonance_geometry` integration target beside the existing public-surface action-set comparison (´test:integration:three-action-challenge-at-least-as-strong-as-four-action´), and its module index and test documentation state the half-power promise it measures.

- Setup: build one seeded `Scenario` with `scenario_with`, register a channel whose `ChannelPolicy.actions` is `[Action::Allow, Action::Block]`, and derive one cold request through `World::derive_for_request`.

- Structural observation: require exactly one landscape crossover, require its transition to be Allow-to-Block, require exactly two regimes in that order, and select exactly the Allow and Block action tags from each rendered profile.

- Controlled rendering stimulus: retain the reckoning's owned `DecisionLandscape` and `RiskBasis`, require the sole crossover variance to be finite and positive, and call `render_resonances` twice with $c_Q=Q_{\mathrm{target}}\sqrt{2\operatorname{Var}(\ell^*)}$ for target bandwidths one and two while keeping $Q_{\mathrm{floor}}$ below both targets.

- Fixture precondition: for both target bandwidths, compute the raw logits $\ell^*\mp1/Q_{\mathrm{target}}$, map them with `stable_sigmoid`, and require the Allow value to lie strictly inside $(0.01,0.49)$ and the Block value strictly inside $(0.51,0.99)$ before treating either profile as a placement witness (´alg:rendering:placement´).

- Bandwidth oracle: independently evaluate $\max(c_Q/\sqrt{2\operatorname{Var}(\ell^*)},Q_{\mathrm{floor}})$ and compare both action tags with that value under `World::tol().default` (´def:rendering:bandwidths´) (´tab:assayer:harness-scenario-tolerances´).

- Placement oracle: compare each reported location with the corresponding `stable_sigmoid` result and compare `stable_logit(tag.location)-crossover.logit` with the signed reciprocal-Q offset under `World::tol().default`, so a sigmoid or location-space coincidence cannot hide a wrong logit displacement (´alg:rendering:placement´).

- Half-power oracle: evaluate each tag's `TagResonance::kernel_at_logit` at the crossover logit and compare it with half that tag's own magnitude under `World::tol().crossover` (´eq:rendering:kernel´) (´tab:assayer:harness-scenario-tolerances´).

- Isolation: hold the landscape and risk basis fixed across both pure rendering calls. The sole decision boundary cannot move; only the declared display scale changes, and both action peaks must follow its resulting Q rather than a regime endpoint or fixed displacement (´dec:landscape:rendering-optional´).

- Health observation: finish with `assert_health_clean`. The scenario performs no clock movement, label submission, lifecycle operation, state-probe read, playback, trained fixture, or seed sweep, so no completion barrier or liveness deadline enters the evidence.

The fails-before is a two-action branch that places either tag at the crossover, reuses a fixed offset, borrows an endpoint from the multi-action algorithm, applies the reciprocal of the wrong Q, or evaluates the wrong kernel coordinate. Such an implementation can preserve the existing count, order, positivity, opposite-side placement, and magnitude comparisons, but at least one target-Q render fails a signed-offset or half-power assertion.

## What is missing · `sec:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-missing`

No production or shared-harness capability is missing. The remaining work is one focused scenario in the existing `resonance_geometry` integration target, composed entirely from the finished public and harness surfaces (´tab:assayer:harness-implementation-library-roster´).

**Entry (The Q-normalised two-action fixture)** · `entry:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-q-normalised-fixture`

An inline setup block in `resonance_geometry` builds the guarded seeded scenario, captures one owned reckoning, validates the sole crossover's finite positive variance and both pairs of unclamped raw locations, and constructs the two rendering configurations from that variance. It adds no reusable fixture because no second caller or trained-state precondition exists (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

**Entry (The public half-power witness)** · `entry:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-integration-witness`

The integration function `resonance_geometry::two_action_crossover_sits_at_q_matched_half_power_boundary` adds the structural, bandwidth, placement, half-power, two-Q isolation, and clean-health assertions described above. It depends only on the Q-normalised setup entry and requires no new production hook, probe, barrier, playback row, shared oracle, trained fixture, or sweep.

## Risks and open questions · `sec:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-risks`

**Observation (Half power is per tag, not an action-probability tie)** · `obs:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-half-power-is-per-tag`

The two boundary magnitudes are the two posture-side regime masses and need not agree. Reciprocal-Q placement makes each unnormalised kernel equal half of its own peak at $\ell^*$; it implies neither equal Allow and Block contributions nor equal normalised tag probabilities (´eq:rendering:kernel´).

**Observation (The location constraints stay outside the oracle)** · `obs:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-clamps-stay-idle`

The rendering constrains Allow and Block peaks to separate drawable intervals after applying the closed form (´alg:rendering:placement´). The explicit raw-location precondition distinguishes an unsuitable fixture from a placement defect and prevents a constrained endpoint from being accepted as the reciprocal-Q result.

**Observation (Numerical error is local and bounded)** · `obs:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-numerics-are-local`

One cold assessment supplies the held inputs, both renderings are pure, and the comparisons use the tolerance whose declared standing matches each operation: `default` for the one-shot bandwidth and logit arithmetic, `crossover` for the kernel matching condition (´tab:assayer:harness-scenario-tolerances´). No timing, scheduling, convergence, or sampling allowance participates.

**Observation (The witness isolates placement from covariance)** · `obs:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-placement-isolated-from-covariance`

Choosing $c_Q$ from the reported crossover variance normalises the bandwidth and intentionally does not re-prove the two-source variance formula. The test still computes the specified bandwidth independently from configuration and landscape fields, while the promise under study remains the two-action placement reduction.

No maintainer decision remains open: the public owned reckoning, separate rendering call, guarded scenario construction, named tolerances, and integration-test home determine the witness completely.

## Acceptance · `sec:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary-acceptance`

The implementation adds the integration target `resonance_geometry` function `resonance_geometry::two_action_crossover_sits_at_q_matched_half_power_boundary`; its module index and test documentation cite this intent and the existing claim, and the coverage report resolves the promise to that witness.

The focused test and the Assayer formatting, lint, test, and corpus gates pass without a timing allowance, ignored case, statistical sampling, new dependency, production hook, or shared-harness addition.

The assertions establish the Allow-to-Block transition, one crossover and two regimes, both expected bandwidths, both signed reciprocal-Q offsets, both half-peak kernel readings, idle drawable constraints, isolation across the two display configurations, and clean health.

The in-test preconditions distinguish invalid fixture geometry from a product failure, and the fails-before explanation names which signed-offset or half-power assertion rejects fixed, zero, endpoint-derived, wrong-Q, or wrong-coordinate placement; no executed mutation is required.
