# Keeping the Thousand-Count Injection Clamp · `plan:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand`

This plan keeps the promise that one oversized injection into the host-owned Companion tracker contributes exactly the default ceiling to state and emits a warning that makes the clamp visible (´claim:risk:injected-pseudo-counts-are-clamped-at-a-thousand´).

## What the promise says, precisely · `sec:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-promise`

The tracker state separates failure pseudo-count $\alpha$, pass pseudo-count $\beta$, their priors, the last-write time, the decay rate, the per-call injection ceiling, and any override (´def:companion:state´). The default prior is $\operatorname{Beta}(1,1)$ (´def:companion:prior´).

The stimulus is one call with `failures = 5000.0` and `passes = 0.0` against a fresh channel carrying the default injection ceiling. That ceiling is the configuration value `1000.0`, applies independently to each argument of each call, and is fixed by (´tab:config:companion´) and the code pin (´const:assayer:evidence-injection-ceiling-scalar-1000p0´).

Injection first decays existing state to the supplied timestamp, then adds the two non-negative counts after clamping each at the ceiling (´alg:companion:injection´). Holding construction, injection, and observation at one timestamp makes the expected post-injection posterior exactly $\operatorname{Beta}(1001,1)$: one thousand injected failures above the alpha prior, no injected passes above the beta prior, and an effective sample size of exactly one thousand.

The literal state expectations matter independently. An effective sample size of one thousand alone would also fit five hundred failures plus five hundred passes, while the promise requires all admitted evidence on the failure side and nothing on the pass side.

The oversized failure count also produces a warning. The state assertion and warning assertion are conjunctive: a correct clamp applied silently keeps only half the promise, and a warning paired with an unclamped or wrongly clamped state keeps the other half only.

The record makes the same pairing a design decision: injected evidence enters the ordinary counts, while the cap prevents one host import from overwhelming observation (´dec:challenge:capped-injection´).

## What the code offers today · `sec:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-current-code`

The crate root exports `ChallengeEffectivenessState`, `ChallengeEffectivenessTracker`, `ChallengeEffectivenessProvider`, `ChallengePosterior`, and the error and health types, so the witness can remain an integration test over the application-facing API.

`ChallengeEffectivenessTracker::inject_evidence` validates both arguments, creates an absent channel from its default state, and delegates to `ChallengeEffectivenessState::inject_evidence` (´claim:risk:a-channel-created-on-demand-caps-injections-at-a-thousand-pseudo-counts-by-default´). The state method decays first, clamps failures and passes separately, adds them to alpha and beta, and returns the resulting estimate (´claim:risk:injected-evidence-is-capped-at-the-channels-ceiling-before-it-reaches-the-beta-state´).

The clamp helper emits `tracing::warn!` synchronously when a value exceeds the ceiling, carrying the evidence kind, requested value, ceiling, and a clamp message. No warning-observation helper exists in the test harness; `init_tracing` installs a formatted test writer for diagnostics but exposes no captured events to assertions.

The replacement surface returns the shipped tracker's exact decayed pseudo-count pair rather than reconstructing it from moments (´req:companion:replacement-trait´) and (´sig:companion:posterior´). Calling `ChallengeEffectivenessProvider::challenge_posterior` at the injection timestamp therefore observes alpha and beta without private access, while `health_report` independently exposes the effective sample size specified by (´tab:companion:health´).

The completed harness skeleton supplies deterministic scenario construction, clocks, and tracing setup (´entry:assayer:harness-stage-skeleton´). `World` owns a tracker at the default ceiling and exposes observed challenge-result routing, but it exposes no evidence-injection or warning-inspection verb; adding either would widen shared harness surface for a host-owned operation already exported directly.

The nearest configurable-ceiling unit witness proves that five failures against a ceiling of two yield two effective samples (´test:unit:tracker-inject-evidence-clamps-to-ceiling´). The nearest default-ceiling unit witness offers five thousand failures and observes the derived estimate and one thousand effective samples (´test:unit:tracker-inject-evidence-default-ceiling-is-one-thousand´). Neither captures the warning, and neither directly asserts the unchanged beta count.

The nearest integration witness drives the exported tracker with one observed failure and checks its returned estimate and channel creation (´test:integration:companion-tracker-records-challenge-result´). It establishes the public host path but never calls evidence injection.

The standing testing plan has no gap entry covering this promise.

## The witness · `sec:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-witness`

The witness belongs in the public Companion tracker integration module, beside (´test:integration:companion-tracker-records-challenge-result´), and its module index carries the same measurable statement as its test documentation.

- Setup: create a `ChallengeEffectivenessTracker`, one `ChannelId`, and one fixed `PersistentTimestamp`; insert a `ChallengeEffectivenessState` built from defaults with `t_last` replaced by that timestamp, and assert the default injection ceiling against the literal `1000.0` before using it.

- Warning fixture: install a thread-scoped tracing subscriber around only the injection call, collecting structured events in memory rather than relying on captured terminal output or a process-global subscriber.

- Stimulus: call `inject_evidence(channel, 5000.0, 0.0, &now)` exactly once and require the result to be successful.

- State observation: call `ChallengeEffectivenessProvider::challenge_posterior` at the same timestamp and read `health_report` at that timestamp, so the posterior pair and effective sample size describe the state immediately after injection under the specified non-mutating read (´alg:companion:inference´).

- State assertion: require the exact posterior `alpha == 1001.0` and `beta == 1.0`, and require the channel health detail to report `effective_sample_size == 1000.0`. These exactly representable values need no tolerance.

- Warning assertion: require exactly one captured event at WARN for the isolated call, identifying a failure-count clamp from `5000.0` to the `1000.0` ceiling. The assertion reads structured event data and does not pin timestamp, ANSI rendering, field order, or a complete formatted line.

- Isolation: do not create a `World`, perform assessments, submit labels, advance time, sleep, or wait on background work. The Companion operation and trace emission are synchronous, and the fixed timestamp makes decay exactly inert.

The fails-before has two independent forms. Removing or misplacing the clamp leaves alpha at `5001.0`, caps the prior together with evidence at `1000.0`, changes beta, or reports an effective sample other than `1000.0`; retaining the numeric clamp while deleting the warning or lowering it to a non-warning level leaves the state assertions green and fails the event assertion.

## What is missing · `sec:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-missing`

**Entry (A scoped warning-event capture fixture)** · `entry:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-warning-capture-fixture`

A test-local fixture of roughly forty lines records event level and the clamp fields through a thread-scoped `tracing-subscriber` layer, then returns the captured events for assertion. It uses the tracing dependencies already activated by `test-support`, changes no manifest, cannot intercept sibling test threads, and depends on no other intent lane.

**Entry (The public injection integration witness)** · `entry:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-integration-witness`

A focused integration test of roughly thirty-five lines adds the fixed state, one oversized injection, exact posterior and health assertions, and the warning assertion described above, together with its module-index row and test documentation. It depends on (´entry:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-warning-capture-fixture´) and requires no production or shared-`World` change.

## Risks and open questions · `sec:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-risks`

**Observation (The ceiling is per argument and per call)** · `obs:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-per-call-boundary`

The specification caps each count in one injection, not a channel's lifetime evidence. The witness makes one oversized failure injection and zero passes; it does not strengthen the contract into a cumulative cap that would wrongly reject a later legitimate injection.

**Observation (Prior mass is not injected evidence)** · `obs:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-prior-separation`

The stored alpha is one thousand and one because the uniform prior contributes one and the clamped injection contributes one thousand. Asserting both the raw posterior and the prior-net effective sample prevents an implementation from counting the prior inside the ceiling or from reaching the right total with evidence on the wrong side.

**Observation (Warning semantics outrank formatting)** · `obs:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-warning-semantics`

The promise fixes warning severity and disclosure of the clamp, not a formatter's byte sequence. Structured capture checks that the isolated oversized failure emits one WARN carrying the requested value and ceiling while leaving target spelling, timestamps, field order, and presentation free to change.

**Observation (Time and scheduling do not enter the result)** · `obs:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-deterministic-observation`

The state begins at and is read at the same explicit timestamp, so lazy decay has an elapsed interval of zero. The tracker performs no asynchronous work, and a thread-scoped subscriber prevents concurrently running integration tests from adding or consuming the event.

No maintainer decision remains open: the exported exact-posterior surface, existing tracing event, fixed timestamp, and integration-test home determine the witness completely.

## Acceptance · `sec:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand-acceptance`

The implementing lane's report identifies the new test in the public Companion tracker integration module beside (´test:integration:companion-tracker-records-challenge-result´), confirms its module index and test documentation cite the intent, and shows the coverage report resolving the intent to that integration witness.

The report shows the targeted integration binary passing and the package's prescribed formatting, clippy, documentation, default-feature, no-default-feature, debug, and release test gates passing, with unrelated failures separated explicitly.

The reported assertions pin the literal default ceiling, the exact `Beta(1001, 1)` posterior, the one-thousand prior-net effective sample, and one synchronous WARN that discloses the five-thousand failure request and one-thousand ceiling.

The report states which numeric assertion rejects an absent, total-state, or wrong-threshold clamp and which event assertion rejects silent or non-warning clamping; no executed mutation is required.

Acceptance excludes production changes, a `World` injection verb, timing allowances, statistical sampling, and formatted-log snapshots because the exported host path and a scoped structured event capture keep the promise directly.
