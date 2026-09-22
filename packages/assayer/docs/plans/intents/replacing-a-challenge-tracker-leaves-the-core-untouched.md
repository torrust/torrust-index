# Keeping Challenge-Tracker Replacement Outside the Core · `plan:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched`

This plan keeps the promise that replacing a channel's challenge-effectiveness provider changes only the challenge-sensitive decision layer while the same observation retains its Core belief and a saved old posterior still reproduces its old reckoning exactly (´claim:channel:replacing-a-challenge-tracker-leaves-the-core-untouched´).

## What the promise says, precisely · `sec:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-promise`

The Core assessment call has no channel-policy input and returns a channel-independent assessment (´req:runtime:assessment-interface´). The Companion has no interface to the Core, the Core holds no Companion state, and the host alone reads or replaces the provider (´inv:companion:boundary´); the corresponding record makes that absence structural by keeping Companion state out of both the working copy and the published snapshot (´dec:challenge:host-ownership´) (´cor:challenge:structural-independence´).

The replacement is observable at the public type boundary: a host can exchange one `ChallengeEffectivenessProvider` for another, ask the replacement for a `ChallengePosterior` at the same `ChannelId` and `PersistentTimestamp`, and pass that value to derivation. The specification requires a scalar estimate strictly inside the unit interval with strictly positive variance (´req:companion:replacement-trait´), while the implemented default reproduces those moments only in the Beta domain where variance is below $\hat{q}_c(1-\hat{q}_c)$ and otherwise falls back to the uniform prior. The witness therefore uses a pair inside that mathematical domain and observes the moment-matched posterior required at the derivation boundary (´sig:companion:posterior´).

The Core oracle is every field on the current public `RiskBasis` carrier: `p_bad`, `uncertainty`, `borrowed_share`, `anchor_weight`, `rho_eff`, `sigma_eff`, `kappa_eff`, `p_bad_sister`, `p_bad_operational`, `intervention_effectiveness`, `anchor_converged`, `n_sentinels_reporting`, `sister_regime_calibration_records`, and `anchor_regime_calibration_records`. Floating-point fields compare at the harness's `exact` tolerance of zero and discrete fields compare exactly, implementing the complete-basis boundary (´schema:risk:basis´) with the declared exactness setting (´tab:assayer:harness-scenario-tolerances´).

Replay holds the assessment, `ChannelPolicy`, old posterior, and `ResonanceConfig` fixed after the provider is gone. Every public field of the resulting `DecisionLandscape` and `ResonanceProfile` must reproduce as raw floating-point bits or exact discrete values, because `derive_landscape` is a stateless function of its three explicit arguments (´sig:landscape:derivation-function´) and its purity proof promises one result per argument triple (´pf:landscape:purity´); the separate `render_resonances` call is likewise fixed by the saved landscape, risk basis, and display configuration (´sig:rendering:contract´).

The replacement estimate is a distinct stimulus, not a tolerated perturbation. Eight `ChallengeResult::Fail` updates from the uniform prior at one held timestamp leave the old tracker at Beta(9,1) by the specified conjugate update (´alg:companion:update´). A scalar replacement at $\hat{q}_c = 0.25$ and variance $0.015625$ yields the moment-matched Beta(2.75,8.25): the Beta moment equations give total concentration eleven, hence alpha 2.75 and beta 8.25 (´alg:companion:inference´) (´req:companion:replacement-trait´). Every figure in that pair is exactly representable in binary floating point, so the fixture can reject a fallback without fitting a tolerance. The landscape's risk-only `u` and `sigma_u` and policy-only `beta_sum` remain bit-identical, while posterior-dependent crossovers and regimes may move because channel constants are recomputed from the current estimate on every call (´dec:derivation:per-call-constants´).

The rendered profile divides into classification and action families (´tab:rendering:spectrum´). Classification tag kind, location, magnitude, bandwidth, and domination flag remain bit-identical because their inputs are the unchanged risk basis and display configuration. Action tag kinds remain the policy's declared sequence, while the Allow-to-Challenge crossover and at least one action tag's location, magnitude, or bandwidth must differ in raw bits: the specification says a lower challenge estimate moves that crossover toward the cautious end, so this liveness check needs no fitted tolerance (´data:channel:challenge-interaction´).

## What the code offers today · `sec:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-current-code`

The public challenge-tracking surface exports `ChallengeEffectivenessTracker`, `ChallengeEffectivenessProvider`, `ChallengeEstimate`, `ChallengePosterior`, and `ChallengePosteriorInput`. One unit witness proves that a scalar-only provider receives the moment-matched default (´test:unit:replacement-trait-default-posterior-moment-matches´); another proves that the shipped tracker answers through the same trait with its exact decayed conjugate counts (´test:unit:shipped-tracker-implements-the-replacement-surface´).

The public decision surface exposes `derive_landscape` over an explicit assessment, policy, and posterior and `render_resonances` over that landscape, its risk basis, and a display configuration (´sig:landscape:derivation-function´) (´sig:rendering:contract´). Neither call needs a live tracker after its posterior has been copied, matching the decision that derivation is a pure transform (´dec:derivation:pure-transform´).

The finished harness supplies one `World`, the guarded `World::trained_state` constructor returning `TrainedStateFixture` and `TrainedStateBaseline`, `World::assess`, the queue-specific `World::flush_observations` barrier, named tolerances, complete risk-basis and health assertions, and forward-only time verbs for witnesses that need time to move (´tab:assayer:harness-implementation-library-roster´) (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´). This witness holds time fixed and waits only on `World::flush_observations`, consistent with the declared no-ad-hoc-waits boundary (´dec:harness:no-ad-hoc-waits´).

The finished probes return published model blocks, slot moments, and pending entries; none is the public assessment or decision output this witness reads, so adding a projection would widen the harness without improving the oracle (´dec:harness:probe-contract´). The shared oracle tier has no challenge-posterior or landscape computation, and this fixed deterministic case has no seed population to sweep (´dec:harness:oracle-tier´) (´dec:harness:seeded-sweeps´). The witness authors no long row stream: its trained population comes from the guarded fixture, so it needs neither a new `PlaybackRow` nor a `PlaybackBarrierPolicy`; any later long subject-owned stimulus must use the existing playback runner (´dec:harness:declarative-playback´).

The nearest crate witness proves that a host-owned tracker update and a Core label update are separate state-machine operations (´test:crate:companion-boundary-tracker-updates-outside-core´). The nearest routing witness proves that a known assessment identifier feeds the harness-owned tracker and changes a later Challenge tag (´test:integration:world-routes-challenge-result-to-companion-tracker´). Neither exchanges a provider or reassesses one observation across that exchange.

The multi-channel boundary matrix drives separate Pass and Fail histories and proves matching cores with diverging action layers (´test:integration:challenge-pass-fail-core-independence´); it does not retire one provider, retain its posterior, or replay through the replacement trait. Its private support remains the declared home for channel matrices and paired ceremony (´tab:assayer:harness-implementation-support-roster´) (´dec:harness:specialised-side-harnesses´).

The existing purity witness repeats one held assessment under one unchanged default estimate (´test:integration:public-derive-landscape-and-render-resonances-bit-identical´). Its landscape fingerprint omits the posterior, crossover action identities, reward differentials, posture intervals, regime action identities, and domination probabilities, while its profile fingerprint omits the posture domain and echoed display configuration, so it does not establish exhaustive old-posterior replay after provider replacement.

The standing testing plan has no gap entry covering this promise, and no test mints or cites its claim.

## The witness · `sec:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-witness`

The witness belongs in the `multi_channel` integration target beside `challenge_pass_fail_core_independence`, with its scalar provider in `support::derivation` and its exhaustive comparators in `support::assertions`; that placement retains the specialised channel vocabulary while using the common `World`, fixture, and barrier surfaces (´dec:harness:specialised-side-harnesses´).

- Compile-time surface: define a scalar provider implementing `ChallengeEffectivenessProvider`, hold the active provider as `Box<dyn ChallengeEffectivenessProvider>`, and replace a boxed `ChallengeEffectivenessTracker` with the scalar provider. Compilation proves that a host can name, implement, erase, and exchange the public replacement surface without a Core handle.

- Setup: obtain `TrainedStateFixture` through `World::trained_state` and retain its measured `TrainedStateBaseline` as setup evidence rather than as the result oracle (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´). Build one request with `with_standard_final_score_verified`, because the guarded world declares the score-and-verification schema, and clone that complete payload for both assessments. Construct the old external tracker independently, feed it eight `ChallengeResult::Fail` updates for one `ChannelId` at one held `PersistentTimestamp`, place it behind the trait object, and require the captured posterior to be Beta(9,1). Hold `ChannelPolicy::default`, `ResonanceConfig::default`, and scenario time fixed.

- Before stimulus: call `World::assess` with the saved request, cross `World::flush_observations`, obtain the old provider's posterior, derive and render, and retain the assessment, policy, posterior, landscape, profile, and display configuration as host-owned values. The named observation barrier settles the only assessment-side queue this comparison could otherwise sample in flight (´dec:harness:no-ad-hoc-waits´).

- Stimulus: replace the boxed tracker with a scalar provider whose `ChallengeEstimate` is $\hat{q}_c = 0.25$ with variance $0.015625$, read its posterior at the same channel and timestamp, and require Beta(2.75,8.25). Perform no Core label, report, lifecycle, clock, identity, or signal operation during the swap.

- Core observation: call `World::assess` with a clone of the identical saved request, cross `World::flush_observations`, and compare every current `RiskBasis` field with `assert_risk_basis_near` at `world.tol().exact`; compare the assessment identifiers only for inequality, because allocation identity is not part of belief.

- Old-posterior replay: after replacement, call `derive_landscape` on the saved pre-swap assessment, saved policy, and saved old posterior, then call `render_resonances` with the saved risk basis and configuration. Require exhaustive raw-bit equality with the original landscape and profile, including the `ChallengePosteriorInput` variant and payload, every crossover and regime field, every `TagResonance` field, `posture_domain`, and every `config_echo` field.

- New-posterior isolation: call `derive_landscape` on the post-swap assessment with the replacement posterior and the same policy, then render with the same configuration. Require raw-bit equality for `u`, `sigma_u`, and `beta_sum`; require the three classification tags to replay field for field; require action tag kinds to equal the declared policy sequence; and require the Allow-to-Challenge crossover plus at least one action-tag numeric field to differ in raw bits.

- Liveness and health: require the old and replacement posterior shapes, means, and variances to differ before reading action divergence, retain the guarded fixture baseline in the failure context, and finish with `assert_health_clean` so a malformed setup or fallback posterior cannot counterfeit isolation.

The fails-before has four independent signatures. A Core surface that acquires the active provider breaks the compile-time separation or changes at least one basis field after the swap; a derivation that consults anything but the saved posterior fails old-posterior replay; a renderer that lets challenge state reach classification fails the exact classification comparison; and a derivation that ignores the replacement passes the stability checks but fails the crossover and action-tag liveness assertions.

## What is missing · `sec:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-missing`

No production or shared-harness surface is missing for this witness's in-domain replacement. The remaining pieces are private to the existing multi-channel integration target and build on the finished harness.

**Entry (The exchangeable provider fixture)** · `entry:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-provider-fixture`

`support::derivation` needs a test-local scalar provider plus the `Box<dyn ChallengeEffectivenessProvider>` exchange that makes the public trait the stimulus rather than simulating replacement with two bare estimates. The fixture uses `ChallengeEstimate::new` for the scalar pair and the provider's default `challenge_posterior`; it depends on the public replacement surface and its existing unit witnesses, not on a new dependency or compile-fail harness (´req:companion:replacement-trait´) (´test:unit:replacement-trait-default-posterior-moment-matches´) (´test:unit:shipped-tracker-implements-the-replacement-surface´).

**Entry (The exhaustive derivation replay comparator)** · `entry:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-replay-comparator`

`support::assertions` needs private comparators for every public `DecisionLandscape`, `ActionCrossover`, `ActionRegime`, `ChallengePosteriorInput`, `ResonanceProfile`, `TagResonance`, and `ResonanceConfig` field. Floating-point payloads compare with `to_bits`, variants and actions compare exactly, and vector shape and order compare before their elements. These comparators extend rather than replace the partial fingerprint retained by the existing purity witness (´test:integration:public-derive-landscape-and-render-resonances-bit-identical´).

**Entry (The provider-replacement integration witness)** · `entry:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-integration-witness`

The `multi_channel` target needs the focused test `replacement_provider_leaves_core_untouched_and_old_posterior_replays`, using `World::trained_state`, the exchangeable provider, the observation barrier, the second assessment, exhaustive old replay, permitted new-posterior movement, liveness controls, and clean-health assertion. It depends on (´entry:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-provider-fixture´) and (´entry:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-replay-comparator´), and on no sibling intent.

## Risks and open questions · `sec:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-risks`

**Observation (The coordinate state must be settled before the comparison)** · `obs:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-settled-coordinate-state`

Repeated assessment is not generally pure: the cold ramp may advance observational geometry even though no outcome-learned state moves (´dec:ordering:evidence-authority´). `World::trained_state` returns only after its trained preconditions and completed setup have been measured, and `World::flush_observations` after each assessment closes the observation queue before the next comparison. No identity dimension is registered and time does not move, so neither the identity-maintenance barrier nor a time verb is part of this witness; inferring either queue's completion from the observation barrier would be a defect.

**Observation (The risk-basis schema and carrier disagree on four substitutions)** · `obs:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-risk-basis-field-disagreement`

The risk-basis schema (´schema:risk:basis´) and the current public carrier agree on the effective estimate and uncertainty, calibration parameter, risk probability and uncertainty, borrowed share, intervention effectiveness, blend weight, and two calibration counts. The schema's operational estimate, sister uncertainty, anchor uncertainty, and label count instead correspond in the carrier to an operational probability, a sister probability, an anchor-convergence flag, and `n_sentinels_reporting`. The witness compares every field the public carrier actually yields; reconciling those four substitutions is a separate conformance decision and does not justify omitting a current carrier field from this promise's oracle.

**Observation (Exact replay and replacement liveness use different thresholds)** · `obs:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-two-thresholds`

Old-posterior replay and unchanged Core and classification fields compare exact bits or exact discrete values because their full arguments are held. Replacement liveness is an inequality over a deliberately different posterior: the named crossover and one action field must differ in raw bits. It uses no numeric threshold, so a fitted drift budget cannot hide a leak or become an unsupported lower bound on how far the specification requires a tag to move.

**Observation (The positive API probe needs no compilation-failure fixture)** · `obs:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-positive-api-probe`

Replacement is a positive type-shape property: the custom provider, trait object, swap, posterior read, and explicit derivation must compile. A surface weakened to the concrete tracker, made non-object-safe, or no longer exported fails the integration target at compilation; no forbidden program or stderr snapshot is part of this promise, so a UI-test harness would add no discrimination.

The broader scalar-provider domain remains a specification question: the requirement admits every strictly positive variance, while a Beta moment match exists only below $\hat{q}_c(1-\hat{q}_c)$ and the implementation returns the uniform prior outside that range. This witness deliberately stays inside the shared domain and therefore neither hides nor resolves that discrepancy.

The carrier substitutions and scalar-provider domain are the open specification questions. The finished probes omit no state this witness needs because the assessment, landscape, and profile are already owned public values; the guarded fixture meets the trained-state precondition; and `World::flush_observations` covers the only queue the witness waits on.

## Acceptance · `sec:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-acceptance`

The implementing lane adds the integration target `multi_channel` test `replacement_provider_leaves_core_untouched_and_old_posterior_replays`, its private exhaustive comparators and scalar-provider fixture, and the module-index and test documentation citation that make the intent resolve to the witness.

The report shows that targeted test passing, the coverage report removing the intent from the unwitnessed set, and the package's prescribed formatting, lint, feature, documentation, and test gates passing, with unrelated failures separated explicitly.

The reported assertions enumerate the unchanged current risk-basis fields, the exact Beta(9,1) and Beta(2.75,8.25) posteriors, the exhaustive bit-identical old landscape and profile, the exact risk-only landscape scalars and classification tags under the replacement posterior, the stable action-tag sequence, the changed Allow-to-Challenge crossover and action-layer field, the guarded trained-state baseline, and clean health.

The report states which of the four deliberate defects each oracle rejects; no executed mutation is required.

Acceptance excludes production hooks, a harness-owned provider replacement method, direct clock access, clock advancement, cross-world comparison, statistical sampling, a new probe or shared oracle, and any tolerance on old-posterior replay, because the host-owned provider can be exchanged and both derivations can be observed through the existing public surface.
