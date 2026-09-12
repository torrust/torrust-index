# Keeping Challenge-Tracker Replacement Outside the Core · `plan:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched`

This plan keeps the promise that replacing a channel's challenge-effectiveness provider changes only the challenge-sensitive decision layer while the same observation retains its Core belief and a saved old posterior still reproduces its old reckoning exactly (´claim:channel:replacing-a-challenge-tracker-leaves-the-core-untouched´).

## What the promise says, precisely · `sec:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-promise`

The Core assessment call has no channel-policy input and returns a channel-independent assessment (´req:runtime:assessment-interface´). The Companion has no interface to the Core, the Core holds no Companion state, and the host alone reads or replaces the provider (´inv:companion:boundary´); the corresponding record makes that absence structural by keeping Companion state out of both the working copy and the published snapshot (´dec:challenge:host-ownership´) (´cor:challenge:structural-independence´).

The replacement is observable at the public type boundary: a host can exchange one `ChallengeEffectivenessProvider` for another, ask the replacement for a posterior at the same `ChannelId` and timestamp, and pass that value to derivation. A conforming scalar provider supplies an estimate strictly inside the unit interval with strictly positive variance, while its default posterior is the moment-matched Beta required by the replacement surface (´req:companion:replacement-trait´) (´sig:companion:posterior´).

The Core oracle is every field on the current public `RiskBasis` carrier: `p_bad`, `uncertainty`, `borrowed_share`, `anchor_weight`, `rho_eff`, `sigma_eff`, `kappa_eff`, `p_bad_sister`, `p_bad_operational`, `intervention_effectiveness`, `anchor_converged`, `n_sentinels_reporting`, `sister_regime_calibration_records`, and `anchor_regime_calibration_records`. Floating-point fields compare at the harness's `bit_identical` tolerance of zero and discrete fields compare exactly, implementing the complete-basis boundary (´schema:risk:basis´) with the declared exactness setting (´tab:assayer:harness-scenario-tolerances´).

Replay holds the assessment, policy, old posterior, and display configuration fixed after the provider is gone. Every public field of the resulting `DecisionLandscape` and `ResonanceProfile` must reproduce as raw floating-point bits or exact discrete values, because the derivation is a stateless function of its three explicit arguments (´sig:landscape:derivation-function´) and its purity proof promises one result per argument triple (´pf:landscape:purity´).

The replacement estimate is a distinct stimulus, not a tolerated perturbation: eight Fail observations from the uniform prior leave the old tracker at Beta(9,1), while a scalar replacement at `q_c = 0.2` and variance `0.01` supplies a valid, visibly different posterior. The landscape's risk-only `u` and `sigma_u` and policy-only `beta_sum` remain exact, while posterior-dependent crossovers and regimes may move because channel constants are recomputed from the current estimate on every call (´dec:derivation:per-call-constants´) and the challenge estimate moves crossover position and uncertainty (´data:channel:challenge-interaction´).

The rendered profile divides into classification and action families (´tab:rendering:spectrum´). Classification tag kind, location, magnitude, bandwidth, and domination flag remain bit-identical because their inputs are the unchanged risk basis and display configuration; action tag kinds remain the declared action set, while at least one action tag's location, magnitude, or bandwidth must move by more than `PURITY_DRIFT`, currently `1e-6`, so a provider swap that the derivation ignores cannot pass.

## What the code offers today · `sec:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-current-code`

The public challenge-tracking surface exports the concrete `ChallengeEffectivenessTracker`, the object-safe `ChallengeEffectivenessProvider`, exact `ChallengePosterior` values, and the scalar-provider default. The shipped tracker already proves that its trait-object view returns exact decayed conjugate counts rather than flattening them (´test:unit:shipped-tracker-implements-the-replacement-surface´).

The public landscape derivation function exposes `derive_landscape` over an explicit assessment, policy, and posterior (´sig:landscape:derivation-function´), and the rendering contract exposes `render_resonances` over that landscape, its risk basis, and a display configuration (´sig:rendering:contract´). Neither call needs a live tracker after its posterior has been copied, matching the record that makes derivation a pure transform (´dec:derivation:pure-transform´).

The completed harness skeleton supplies `World`, a deterministic virtual clock, `World::settle_cold_ramp_with`, `World::assess`, channel lookup, label-cycle helpers, named tolerances, and clean-health assertions (´entry:assayer:harness-stage-skeleton´). Its shared assertion layer already compares every current `RiskBasis` field at a caller-supplied tolerance, and the multi-channel support layer already separates classification tags from action tags and supplies a non-degenerate action-layer divergence assertion.

The nearest routing test holds one assessment and proves that updating the harness-owned tracker changes a later Challenge tag (´test:integration:world-routes-challenge-result-to-companion-tracker´); it neither replaces the provider nor reassesses the observation. The nearest boundary matrix drives separate Pass and Fail histories and proves matching cores with diverging action layers (´test:integration:challenge-pass-fail-core-independence´); it does not retire one provider, retain its old posterior, or replay through the replacement boundary.

The existing purity witness repeats one held assessment under one unchanged default estimate and compares a fingerprint (´test:integration:public-derive-reckoning-bit-identical´). Its local landscape fingerprint omits the carried posterior, crossover transition identities, differential fields, posture intervals, and regime domination probabilities, while its profile fingerprint omits the posture domain and echoed display configuration, so it cannot by itself establish exact replay of the whole old reckoning required here.

The standing testing plan has no gap entry covering this promise.

## The witness · `sec:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-witness`

The witness belongs beside the existing multi-channel boundary matrix (´test:integration:challenge-pass-fail-core-independence´), where the channel/Core boundary and action-versus-classification assertions already live, with its private exhaustive comparators in that binary's support layer.

- Compile-time surface: define a small scalar provider implementing `ChallengeEffectivenessProvider`, hold the active provider as `Box<dyn ChallengeEffectivenessProvider>`, and replace a boxed shipped tracker with the scalar provider. Compilation proves that a host can name, implement, erase, and exchange the public replacement surface without a Core handle.

- Setup: build one default seeded `Scenario`, settle the cold standardisation ramp for the observation, run an ordinary challenge-result-free label history to move the Core away from its untouched opening state, flush the label path, and hold the virtual clock fixed. Construct the old external tracker independently, feed it eight Fail results at one held persistent timestamp, and capture its Beta(9,1) posterior.

- Before stimulus: assess the chosen entity once, derive and render with the old posterior, and retain the assessment, policy, posterior, landscape, profile, and display configuration as host-owned values.

- Stimulus: replace the boxed tracker with the scalar provider at `q_c = 0.2` and variance `0.01`; perform no Core label, report, lifecycle, clock, identity, or signal operation during the swap.

- Core observation: assess the identical entity and payload again and compare every current `RiskBasis` field with `assert_risk_basis_near` at `world.tol().bit_identical`; compare the two assessment identifiers only for inequality, because identity allocation is not part of belief.

- Old-posterior replay: after the replacement, derive the saved pre-swap assessment again from the saved old posterior and render it with the saved display configuration; require exhaustive raw-bit equality with the original landscape and profile, including posterior representation, crossover and regime vectors, tag fields, posture domain, and configuration echo.

- New-posterior isolation: derive the post-swap assessment with the replacement posterior and the same policy and configuration; require exact `u`, `sigma_u`, and `beta_sum`, require the three classification tags to replay bit for bit, require the action-tag sequence to remain the declared sequence, and require at least one action numeric field to move past `PURITY_DRIFT`.

- Liveness and health: require the captured posterior means and variances to differ before using tag divergence as an oracle, require the warmed basis to be finite and the label history to have been processed, and finish with `assert_health_clean` so a fallback cannot counterfeit stability.

The fails-before has four independent signatures. A Core that reads or retains the active provider changes at least one basis field after the swap; a derivation that consults the current provider instead of the saved argument fails old-posterior replay; a renderer that lets challenge state reach classification fails the exact classification comparison; and a derivation that ignores the replacement passes the stability checks but fails the action-divergence liveness assertion.

## What is missing · `sec:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-missing`

No production surface or other intent lane blocks the witness. The missing pieces are private to the existing multi-channel integration binary and build on the completed harness.

**Entry (The exchangeable provider fixture)** · `entry:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-provider-fixture`

A test-local scalar provider and boxed-provider swap of roughly fifteen lines make the public trait the actual stimulus rather than simulating replacement with two bare estimates. It depends on (´req:companion:replacement-trait´), the crate-root exports already exercised by (´test:unit:shipped-tracker-implements-the-replacement-surface´), and no new dependency or compile-fail harness.

**Entry (The exhaustive derivation replay comparator)** · `entry:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-replay-comparator`

A private comparator of roughly seventy lines in the multi-channel assertion layer compares every public landscape, crossover, regime, posterior, profile, tag, posture-domain, and configuration field, using raw bits for floating-point values and exact equality for variants and action kinds. It depends only on current public output types and replaces no existing helper; the local fingerprint retained by the existing purity witness remains untouched (´test:integration:public-derive-reckoning-bit-identical´).

**Entry (The provider-replacement integration witness)** · `entry:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-integration-witness`

A focused integration test of roughly sixty lines adds the warmed setup, provider exchange, second assessment, exact old replay, permitted new-posterior movement, liveness controls, and clean-health assertion. It depends on (´entry:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-provider-fixture´) and (´entry:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-replay-comparator´), and on no sibling lane.

## Risks and open questions · `sec:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-risks`

**Observation (The coordinate state must be settled before the comparison)** · `obs:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-settled-coordinate-state`

Repeated assessment is not generally pure: the cold ramp may advance observational geometry even though no outcome-learned state moves (´dec:ordering:evidence-authority´). Settling the ramp before the labelled precondition and holding the clock and structural state fixed ensures that a basis difference after replacement denotes a Companion leak rather than an authorised coordinate or temporal change.

**Observation (The risk-basis schema and carrier disagree on one field)** · `obs:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-risk-basis-field-disagreement`

The risk-basis schema (´schema:risk:basis´) lists a label count among its fourteen fields, while the current public carrier exposes `n_sentinels_reporting` and no label count. The witness compares every field the public carrier actually yields, including `n_sentinels_reporting`; reconciling the specification row and carrier is a separate conformance decision and does not justify omitting either current carrier field from this promise's oracle.

**Observation (Exact replay and replacement liveness use different thresholds)** · `obs:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-two-thresholds`

Old-posterior replay and unchanged Core/classification fields are exact because their full arguments are held; the new action layer is intentionally different and needs a lower-bound liveness check instead. Using zero for sameness and `PURITY_DRIFT` for visible movement prevents a broad tolerance from hiding a leak and prevents an accidentally equivalent replacement fixture from satisfying isolation vacuously.

**Observation (The positive API probe needs no compilation-failure fixture)** · `obs:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-positive-api-probe`

Replacement is a positive type-shape property: the custom provider, trait object, swap, posterior read, and explicit derivation must compile. A surface weakened to the concrete tracker, made non-object-safe, or no longer exported fails the integration target at compilation; no forbidden program or stderr snapshot is part of this promise, so a UI-test harness would add no discrimination.

The carrier discrepancy is the only open specification question and does not block the witness; the public API, deterministic setup, allowed movement, exact replay boundary, and test home otherwise determine it completely.

## Acceptance · `sec:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched-acceptance`

The implementing lane's report identifies the new multi-channel integration scenario, its private comparator and provider fixture, and the module-index and test documentation citation that make the intent resolve to the witness.

The report shows the targeted integration test passing, the coverage report removing the intent from the unwitnessed set, and the package's prescribed formatting, lint, feature, documentation, and test gates passing, with unrelated failures separated explicitly.

The reported assertions enumerate the unchanged current risk-basis fields, the distinct old and replacement posteriors, the exhaustive bit-identical old landscape and profile, the exact risk-only landscape scalars and classification tags under the new posterior, the stable action-tag sequence, the action-layer movement beyond `PURITY_DRIFT`, and clean health.

The report states which of the four deliberate defects each oracle rejects; no executed mutation is required.

Acceptance excludes production hooks, a harness-owned provider replacement method, clock advancement, cross-world comparison, statistical sampling, and any tolerance on old-posterior replay, because the host-owned provider can be exchanged and both derivations can be observed through the existing public surface.
