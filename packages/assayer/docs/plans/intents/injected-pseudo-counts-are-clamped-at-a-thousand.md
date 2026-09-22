# Keeping the Thousand-Count Injection Clamp · `plan:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand`

This plan keeps the promise that one oversized injection into the host-owned Companion tracker contributes exactly the default ceiling to state and emits a warning that makes the clamp visible (´claim:risk:injected-pseudo-counts-are-clamped-at-a-thousand´).

## What the promise says, precisely · `sec:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-promise`

The tracker state separates failure pseudo-count $\alpha$, pass pseudo-count $\beta$, their priors, the last-write time, the decay rate, the injection ceiling, and any override (´def:companion:state´). The default prior is $\operatorname{Beta}(1,1)$ (´def:companion:prior´).

The stimulus is one call with `failures = 5000.0` and `passes = 0.0` against a fresh channel carrying the default injection ceiling. The configuration table fixes that ceiling per call at one thousand (´tab:config:companion´), the injection algorithm caps each injection at it and requires the clamping to be reported rather than applied silently (´alg:companion:injection´), and the code pins the same figure as `1000.0` (´const:assayer:evidence-injection-ceiling-scalar-1000p0´).

Injection decays the existing state to the supplied timestamp first and then adds the two non-negative counts, each capped at the ceiling (´alg:companion:injection´). Holding construction, injection, and observation at one timestamp makes the expected post-injection posterior exactly $\operatorname{Beta}(1001,1)$: one thousand injected failures above the alpha prior, no injected passes above the beta prior, and an effective sample size — the pseudo-counts net of the prior — of exactly one thousand (´alg:companion:inference´).

The literal state expectations matter independently. An effective sample size of one thousand alone would also fit five hundred failures plus five hundred passes, while the promise requires all admitted evidence on the failure side and nothing on the pass side.

The oversized failure count also produces a warning. The state assertion and warning assertion are conjunctive: a correct clamp applied silently keeps only half the promise, and a warning paired with an unclamped or wrongly clamped state keeps the other half only.

The record makes the same pairing a design decision: injected evidence enters the ordinary counts, while the cap prevents one host import from overwhelming observation (´dec:challenge:capped-injection´).

## What the code offers today · `sec:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-current-code`

The crate root exports `ChallengeEffectivenessState`, `ChallengeEffectivenessTracker`, `ChallengeEffectivenessProvider`, `ChallengePosterior`, `ChallengeEvidenceError`, `ChallengeHealthReport`, and `ChallengeHealthDetail`, so the witness can remain an integration test over the application-facing API.

`ChallengeEffectivenessTracker::inject_evidence` validates both arguments, creates an absent channel from its default state, and delegates to `ChallengeEffectivenessState::inject_evidence` (´claim:risk:a-channel-created-on-demand-caps-injections-at-a-thousand-pseudo-counts-by-default´). The state method decays first, clamps failures and passes separately, adds them to alpha and beta, and returns the resulting estimate (´claim:risk:injected-evidence-is-capped-at-the-channels-ceiling-before-it-reaches-the-beta-state´).

The clamp helper emits `tracing::warn!` synchronously when a value exceeds the ceiling, carrying the evidence kind, the requested value, the ceiling, and a clamp message. Nothing in the finished harness observes such an event: the library roster declares probes, playback, oracles, guarded fixtures, and seeded sweeps and no event recorder (´tab:assayer:harness-implementation-library-roster´), and the subscriber the scenario constructors install writes formatted diagnostics through the test writer under an environment filter, exposing no records to an assertion.

The replacement surface returns the shipped tracker's exact decayed pseudo-count pair rather than reconstructing it from moments (´req:companion:replacement-trait´) and (´sig:companion:posterior´). `ChallengeEffectivenessProvider::challenge_posterior` hands back a `ChallengePosterior` carrying both pseudo-counts as public fields, so a reading at the injection timestamp observes alpha and beta without private access, while `health_report` independently exposes the effective sample size the health table specifies (´tab:companion:health´).

The finished harness holds nothing this witness needs. `World` owns a challenge tracker whose channels carry the default injection ceiling and routes host-observed challenge results into it, but declares no evidence-injection verb and no event reader; a permitted probe crosses its barrier, takes one published load of engine state, and returns owned copies, which a recorder of `tracing` events is not (´dec:harness:probe-contract´). The capability survey reaches the same conclusion from the other side, placing this promise among the few whose witness demands none of the shared capabilities and only a test-local subscriber layer (´tab:assayer:testing-harness-concept-plan-demands´).

The nearest configurable-ceiling unit witness proves that five failures against a ceiling of two yield two effective samples (´test:unit:tracker-inject-evidence-clamps-to-ceiling´). The nearest default-ceiling unit witness offers five thousand failures and asserts the derived estimate $1001/1002$ together with one thousand effective samples, which pin the stored pair between them (´test:unit:tracker-inject-evidence-default-ceiling-is-one-thousand´). Neither reads the posterior the host is handed, and neither observes the warning at all.

The nearest integration witness drives the exported tracker with one observed failure and checks its returned estimate and channel creation (´test:integration:companion-tracker-records-challenge-result´). It establishes the public host path but never calls evidence injection.

The standing testing plan has no gap entry covering this promise.

## The witness · `sec:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-witness`

The `label_integration` integration target is the destination for the test `companion_tracker_clamps_injected_pseudo_counts_at_a_thousand_and_warns`, beside (´test:integration:companion-tracker-records-challenge-result´); its documentation and its test-index row cite the intent rather than minting a second claim.

- Setup: create a `ChallengeEffectivenessTracker`, one `ChannelId`, and one fixed `PersistentTimestamp`; insert a `ChallengeEffectivenessState` built from defaults with its last-write time at that timestamp, and assert that state's injection ceiling against the literal `1000.0` before using it. Inserting rather than letting the injection create the channel keeps the starting state the witness's own, since on-demand creation is a separate promise a unit witness already carries.

- Warning fixture: install a thread-scoped tracing subscriber around only the injection call, collecting structured events in memory rather than relying on captured terminal output or on the process-global subscriber a neighbouring scenario may already have installed.

- Stimulus: call `inject_evidence(channel, 5000.0, 0.0, &now)` exactly once and require the result to be successful.

- State observation: call `ChallengeEffectivenessProvider::challenge_posterior` at the same timestamp and read `health_report` at that timestamp, so the posterior pair and effective sample size describe the state immediately after injection under the specified non-mutating read (´alg:companion:inference´); the sufficiency threshold that read takes enters no assertion.

- State assertion: require the exact posterior `alpha == 1001.0` and `beta == 1.0`, and require the channel health detail to report `effective_sample_size == 1000.0`. No tolerance is derived because none is available to derive: at zero elapsed time the decay factor is one, so each of these values is an exactly representable integer in binary64.

- Warning assertion: require exactly one captured event at WARN for the isolated call, identifying a failure-count clamp from `5000.0` to the `1000.0` ceiling. The assertion reads structured event data and does not pin timestamp, ANSI rendering, field order, or a complete formatted line.

- Isolation: build no `World`, so no time moves through `World::advance` or `World::travel_to` and no queue barrier applies — `World::flush_observations`, `World::flush_labels`, and `World::flush_identity_maintenance` each complete a queue this witness never feeds. Nothing else waits either: the Companion operation and its trace emission are synchronous on the calling thread, the fixed timestamp makes decay exactly inert, and a test authors no sleep, poll, or deadline of its own (´dec:harness:no-ad-hoc-waits´).

The fails-before has two independent forms. Removing or misplacing the clamp leaves alpha at `5001.0`, caps the prior together with evidence at `1000.0`, changes beta, or reports an effective sample other than `1000.0`; retaining the numeric clamp while deleting the warning or lowering it below WARN leaves the state assertions green and fails the event assertion.

## What is missing · `sec:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-missing`

**Entry (A scoped warning-event capture fixture)** · `entry:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-warning-capture-fixture`

A test-local fixture of roughly forty lines, in the `label_integration` target rather than in the harness, records event level and the clamp fields through a thread-scoped `tracing-subscriber` layer and returns the captured events for assertion. The crate's development dependencies already carry that crate with its registry and environment-filter features, so the fixture changes no manifest; being thread-scoped, it cannot intercept a sibling test thread; and it depends on no other intent lane. Its home is the target because a recorder of `tracing` events is not a projection of published engine state and so is not a probe (´dec:harness:probe-contract´).

**Entry (The public injection integration witness)** · `entry:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-integration-witness`

A focused integration test of roughly thirty-five lines adds the fixed state, one oversized injection, exact posterior and health assertions, and the warning assertion described above, together with its test-index row and test documentation. It depends on (´entry:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-warning-capture-fixture´) and requires no production change, no `World` verb, and no new harness surface.

## Risks and open questions · `sec:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-risks`

**Observation (The ceiling is per argument and per call)** · `obs:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-per-call-boundary`

The configuration table fixes the ceiling per call (´tab:config:companion´) and the shipped clamp applies it to each of the two counts in that call; neither caps a channel's lifetime evidence. The witness makes one oversized failure injection and no pass injection, so it does not separate the two granularities, and it must not be strengthened into a cumulative cap that would wrongly reject a later legitimate injection.

**Observation (Prior mass is not injected evidence)** · `obs:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-prior-separation`

The stored alpha is one thousand and one because the uniform prior contributes one and the clamped injection contributes one thousand. Asserting both the raw posterior and the prior-net effective sample prevents an implementation from counting the prior inside the ceiling or from reaching the right total with evidence on the wrong side.

**Observation (Warning semantics outrank formatting)** · `obs:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-warning-semantics`

The promise fixes warning severity and disclosure of the clamp, not a formatter's byte sequence. Structured capture checks that the isolated oversized failure emits one WARN carrying the requested value and ceiling while leaving target spelling, timestamps, field order, and presentation free to change.

**Observation (Time and scheduling do not enter the result)** · `obs:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-deterministic-observation`

The state begins at and is read at the same explicit timestamp, so lazy decay has an elapsed interval of zero, and the tracker performs no asynchronous work. The hazard the finished harness adds is the shared binary: the scenario constructors install one process-global subscriber, filtered from the environment and idempotent, so the first caller in a target wins (´tab:assayer:harness-implementation-library-roster´), and this witness sits beside tests that call them. Capturing on the calling thread around the injection alone is what makes the reading the witness's own rather than a function of an environment variable and of which test ran first, and it equally prevents a concurrently running test from adding or consuming the event.

No maintainer decision remains open: the exported exact-posterior surface, the existing warning, the fixed timestamp, and the integration-test home determine the witness completely.

## Acceptance · `sec:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-acceptance`

The implementing lane's report identifies `companion_tracker_clamps_injected_pseudo_counts_at_a_thousand_and_warns` in the `label_integration` target, confirms that its test-index row and test documentation cite the intent, and shows the coverage report resolving the intent to that integration witness.

The report shows the targeted integration binary passing and the package's prescribed formatting, clippy, documentation, default-feature, no-default-feature, debug, and release test gates passing, with unrelated failures separated explicitly.

The reported assertions pin the literal default ceiling, the exact $\operatorname{Beta}(1001,1)$ posterior, the one-thousand prior-net effective sample, and one synchronous WARN that discloses the five-thousand failure request and one-thousand ceiling.

The report states which numeric assertion rejects an absent, total-state, or wrong-threshold clamp and which event assertion rejects silent or non-warning clamping; no executed mutation is required.

Acceptance excludes production changes, a `World` injection verb, a harness event reader, timing tolerances, statistical sampling, and formatted-log snapshots, because the exported host path and a scoped structured event capture keep the promise directly.
