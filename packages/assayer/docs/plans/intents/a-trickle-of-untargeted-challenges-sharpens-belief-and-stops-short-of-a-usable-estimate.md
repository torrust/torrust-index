# A trickle sharpens belief without becoming sufficient · `plan:assayer:intent-a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate`

Keeping (´claim:risk:a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate´) establishes that the shipped Companion turns a sparse, clocked stream into measurably narrower belief without misreporting that belief as sufficiently evidenced, and that a host override substitutes only for the estimate exposed to derivation while learning continues beneath it.

## What the promise says precisely · `sec:assayer:intent-a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate-precision`

The witness starts from the default uniform Beta prior, whose two pseudo-counts are one, whose mean is one half, and whose variance is one twelfth (´def:companion:prior´).

A contributing label is an observed Pass or Fail result for a challenged adverse request, and each result adds one count to its own side after decay to the label timestamp (´alg:companion:update´). The untargeted population rate is `0.08` contributing labels per day, so a regular deterministic fixture places one contribution every `12.5` days, or three hundred hours, and reaches twenty arrivals at day two hundred and fifty (´data:companion:contributing-rate´).

The default Companion decay factor is `0.9998` per hour and blends only the evidence above the prior back toward that prior (´alg:companion:decay´). With twenty alternating Pass and Fail results at the stated cadence, the day-two-hundred-and-fifty read has about twelve effective observations, total pseudo-count mass near fourteen, variance near `0.017`, and prior variance divided by observed variance near five.

The exact oracle is the finite geometric recurrence `n = n * gamma.powf(300.0) + 1.0` applied once per arrival. The semantic assertions additionally place total mass in `[13.5, 14.5]`, variance in `[0.016, 0.018]`, and the sharpening ratio in `[4.5, 5.5]`; the recurrence comparison uses the harness default floating-point tolerance rather than those explanatory bands (´tab:assayer:harness-scenario-tolerances´).

At the same cadence the immediate-post-arrival effective-sample ceiling is `(1 - gamma.powf(300.0)).recip()`, about `17.17`, so the total pseudo-count ceiling including the prior is about `19.17`. The default sufficiency floor is twenty effective observations, remains false at day two hundred and fifty, and is unreachable because the ceiling lies below it (´bound:companion:convergence´) (´dec:challenge:sufficiency-floor-owned-here´).

An active override supplies the estimate and variance that cross the derivation boundary, while effective sample size and stored pseudo-counts remain properties of the underlying Beta state (´alg:companion:override´) (´tab:companion:health´). Contributions continue updating that state, and clearing the override exposes the posterior that includes them (´dec:challenge:override-accumulates´).

## What the code offers today · `sec:assayer:intent-a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate-code-today`

The public Companion surface offers `ChallengeEffectivenessTracker::update`, `estimate`, `health_report`, `set_override`, and `clear_override` over explicit `PersistentTimestamp` values, and `ChallengeEffectivenessProvider::challenge_posterior` preserves exact Beta counts when no override is active (´claim:risk:the-shipped-tracker-answers-the-replacement-surface-with-its-exact-posterior-counts´). `derive_landscape` accepts that posterior explicitly and carries it on `DecisionLandscape`, matching the specified boundary (´sig:companion:posterior´).

Existing direct integration coverage proves that a tracker accepts a Fail result and that `World::record_challenge_result` changes a later derivation (´test:integration:companion-tracker-records-challenge-result´) (´test:integration:world-routes-challenge-result-to-companion-tracker´). The existing held-assessment witness isolates derivation inputs from a moving Core (´test:integration:public-derive-reckoning-bit-identical´).

Mechanism-level tests already establish read-time decay, health reporting, and override replacement separately (´test:unit:estimate-decays-on-read-without-mutating´) (´test:unit:tracker-health-report-contains-channel´) (´test:unit:tracker-override-replaces-estimate-until-cleared´). None composes the specified sparse cadence, finite-horizon sharpening, unreachable sufficiency ceiling, and override-time accumulation.

The world harness owns a `VirtualClock`, a per-channel `ChallengeEffectivenessTracker`, the mapping from assessment identifiers to channels, `derive`, `record_challenge_result`, and `cycle_on`. Its `LabelSpec` can express challenged adverse labels with observable Pass and Fail results, and `cycle_on` synchronously routes those results to the Companion before flushing the Core label path.

The standing testing plan explicitly places this promise behind the partly completed persistent-clock stage (´entry:assayer:harness-stage-clock´). `VirtualClock::advance` and `VirtualClock::travel_to` exist in the harness, and `World::clock` exposes the handle, but the promised `World::advance` and `World::travel_to` scenario verbs do not yet exist.

## The witness · `sec:assayer:intent-a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate-witness`

The integration test belongs beside the existing Companion public-surface scenarios and uses one `World::cold` with its default channel, default tracker prior, default Companion decay rate, deterministic clock, and default tolerance bundle.

The setup reads Companion health before any contribution and asserts zero effective samples, insufficiency at the tracker default floor, mean one half, and variance one twelfth. It then holds one assessment for every later derivation comparison, so changes in the cold standardisation ramp cannot masquerade as Companion effects.

The stimulus advances the world by three hundred hours before each of twenty cycles. Each cycle submits an adverse `LabelSpec` with action `Challenge` and an observable result, alternating Fail and Pass so the fixture sharpens around the neutral mean rather than testing directional imbalance at the same time.

The day-two-hundred-and-fifty observation reads the channel's Companion health and the posterior carried by derivation of the held assessment. It compares effective sample size with the independently accumulated recurrence, reconstructs total mass by adding the two default prior counts, and reads posterior variance from both surfaces.

The assertions require all four parts together: total mass and variance occupy the measurable bands; prior variance divided by the observed variance occupies the fivefold band; effective sample size is below twenty and `sufficient_evidence` is false; and the analytic steady-state ceiling is near `17.17` effective samples, near `19.17` total mass, and strictly below the sufficiency floor.

The override arm sets a deliberately distinct point and explicit variance, derives the held assessment, and asserts that `DecisionLandscape::posterior` carries those values. It then advances one more interval and records one more contributing result while the override stands: the reported estimate remains the override, the raw effective sample changes according to the decay-and-increment recurrence, and a second derivation still carries the override.

Clearing the override and deriving the same held assessment completes the witness. The landscape returns to a conjugate posterior whose effective mass includes the contribution received under the override, rather than either the override values or the pre-override Beta state.

The fails-before is diagnostic. A decay-free implementation reports twenty effective samples, total mass twenty-two, variance around `0.011`, and sufficiency at day two hundred and fifty; an implementation that drops sparse evidence stays at the prior variance; an implementation that freezes under override returns the pre-override posterior after clearing; and an implementation that fails to substitute the override leaves the derived posterior conjugate while the override stands.

## What is missing · `sec:assayer:intent-a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate-missing`

**Entry (World owns the scenario's time verbs)** · `entry:assayer:intent-a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate-world-time-verbs`

Add `World::advance(Duration)` and `World::travel_to(SystemTime)` as thin calls into the world's existing `VirtualClock`, with the same forward-only contract. This is approximately fifteen lines in the world harness plus its test-index coverage, depends only on the existing virtual clock, and discharges the portion of (´entry:assayer:harness-stage-clock´) this scenario consumes; it has no sibling-lane dependency.

**Entry (World exposes its host-owned Companion controls and health)** · `entry:assayer:intent-a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate-world-companion-surface`

Add named-channel harness verbs that set and clear a Companion override and return the current `ChallengeHealthDetail` or report at `ChallengeEffectivenessTracker::DEFAULT_SUFFICIENT_EVIDENCE_THRESHOLD`. The methods lock the existing tracker and read the same world clock used by derivation, requiring approximately thirty-five lines in the world harness; they depend on the time-verbs entry for the scenario and on no other lane.

No tape, random generator, persistence fixture, or Core-state escape hatch is required. Twenty explicit cycles keep the cadence legible, and the existing `cycle_on` path supplies the routing and label flush.

## Risks and open questions · `sec:assayer:intent-a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate-risks`

**Observation (The oracle separates arithmetic from the rounded promise)** · `obs:assayer:intent-a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate-oracle`

The rounded values near fourteen, `0.017`, and nineteen express meaning but are too loose to catch many wrong recurrences. The exact finite recurrence and ceiling formula use the configured gamma and cadence as an independent test-side oracle, while the broader bands establish that the exact result still says what the promise says.

**Observation (The first contribution lands after one interval)** · `obs:assayer:intent-a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate-arrival-phase`

The first contribution occurs at hour three hundred and the twentieth at hour six thousand. Placing the first contribution at the epoch instead would describe twenty arrivals over only nineteen decay intervals and would no longer witness the stated two-hundred-and-fifty-day history.

**Observation (The clocked Companion path is deterministic)** · `obs:assayer:intent-a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate-determinism`

Companion updates and reads are synchronous under the world's tracker mutex, and the persistent clock is atomic and test-controlled. Holding one assessment for the override comparisons removes the asynchronous standardisation ramp from the observation; `flush_labels` remains in the cycles only because the scenario also submits the corresponding Core labels. The two timestamp domains remain distinct as required by (´dec:clock:two-domains´), and Companion decay reaches the shared arithmetic funnel fixed by (´dec:clock:shared-functions´).

**Observation (The harness still carries Companion decay through channel rewards)** · `obs:assayer:intent-a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate-decay-ownership`

`WorldBuilder` seeds each channel's tracker from `ChannelPolicy.reward.gamma_q_t`, although the specification places the rate in Companion configuration. This witness uses the default value shared by both surfaces and checks it against `ChallengeEffectivenessState::DEFAULT_GAMMA_QT`; it does not treat the harness coupling as authority or require its resolution.

**Observation (The health accessor has two honest shapes)** · `obs:assayer:intent-a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate-health-accessor-shape`

The harness can return the whole `ChallengeHealthReport`, mirroring the public host surface, or clone one named channel's `ChallengeHealthDetail`, matching the scenario vocabulary. The full report preserves future multi-channel use and is the preferred shape; either keeps the witness public-surface based and neither changes the assertions.

## Acceptance · `sec:assayer:intent-a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate-acceptance`

The implementation report names the integration test and its claim citation, the two harness entries it landed, and the exact commands and green outputs for formatting, linting, the Assayer package tests with all targets and features, the no-default-features spot check, release tests, documentation, and documentation tests.

The report shows the finite-horizon effective sample, total mass, variance, sharpening ratio, sufficiency verdict, and analytic ceiling observed by the test, together with the override-visible posterior and the post-clear conjugate posterior. It also states that the test fails for the decay-free, ignored-override, and frozen-under-override defects described above, whether demonstrated by targeted mutation or by direct inspection of the assertion each defect violates.

Acceptance requires deterministic time with no wall-clock sleeps, no widened general-purpose tolerance, no direct access to Core model state, and no assertion that twenty arrivals are twenty effective observations.
