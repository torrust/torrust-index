# Planning One Investigated Label Across Every Subsystem · `plan:assayer:intent-one-investigated-label-moves-every-subsystem-at-once`

This plan keeps the promise that one challenged, passed, investigated request moves every outcome-learning subsystem through the same label (´claim:labelling:one-investigated-label-moves-every-subsystem-at-once´); keeping it establishes joint causality rather than another collection of separately passing mechanism tests.

## What the promise says, precisely · `sec:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-promise`

The witness begins with one issued assessment whose pending entry retains the request, reporting Sentinel, extracted chain, identity coordinate and active cells, and registered outcome axis required by the later write path (´def:runtime:pending-entry´). The label for that assessment records `Action::Challenge`, `ChallengeResult::Pass`, positive valence, `ground_truth = true`, and one positive outcome-axis value.

Positive valence gives the risk target $y=+1$ (´def:risk:target´), and established ground truth makes the label eligible independent of the recorded action (´tab:eligibility:training´) (´conv:eligibility:ground-truth´). The global and eligible positive-class rates therefore both move upward from their neutral baselines.

The operational and sister mean vectors each move by more than `DEFAULT_TOLERANCES.default`, which is $10^{-9}$, and the anchor mean also moves under the stronger whole-engine reading of the normative model triple (´def:risk:model-triple´). The next assessment of the same entity and Sentinel coordinate has `RiskBasis.p_bad`, `p_bad_operational`, and `p_bad_sister` each greater than its pre-label value by more than that same tolerance (´schema:risk:basis´).

The channel Companion begins at the uniform Beta prior and, with the virtual clock fixed, preserves $α=1$ while increasing the pass pseudo-count from $β=1$ to $β=2$ exactly (´def:companion:state´) (´def:companion:prior´) (´alg:companion:update´). This count is host-owned evidence beside the Core rather than a field inside its snapshot (´dec:challenge:host-ownership´).

The four-cell golden report supplies one Sentinel chain at depths zero, four, eight, and twelve (´dec:assayer:golden-report-stimulus´). At every one of those entries the Challenge action count, recent eligible count, adverse eligible count, and eligible-arrival load each increase by one; adverse-rate, raw-valence, and compressed-valence averages become positive; and the reported axis appears with positive raw and compressed values, as required by the all-layers write (´alg:ledger:all-layers-update´) and the depth-walk decision (´dec:memory:depth-walk´).

A fixed nested identity chain at depths zero, four, and eight contains the entity coordinate and is recorded in the pending assessment. Every still-competitive recorded cell changes from neutral to positive adverse-rate, raw-valence, and compressed-valence state and gains the reported axis in both per-axis maps, which measures the identity outcome update rather than merely the presence of active indicators (´alg:runtime:identity-outcome-update´).

The registered axis model's mean moves by more than the default tolerance toward its positive compressed target (´def:axis:per-axis-model´) (´def:axis:training-target´). Ledger and identity EWMAs use `DEFAULT_TOLERANCES.ewma`, $10^{-6}$, while exact counts, memberships, identifiers, and untouched Companion $α$ use `DEFAULT_TOLERANCES.bit_identical` (´tab:assayer:harness-scenario-tolerances´).

These observations cover the ordered label function's model, memory, axis, identity, monitoring, and publication stages in one before-and-after interval (´alg:runtime:update-path´) (´dec:ordering:label-function´). They establish that every movement consumes the same assessment identifier and pending context; they do not redefine the host and Core updates as one atomic transaction.

## What the code offers today · `sec:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-code-today`

The public `World` harness path already registers a Sentinel, ingests `golden_report_4cell`, registers a spatial outcome axis and an identity dimension, settles the cold standardisation ramp, builds a request with `World::request_with_sentinel`, derives it with `World::derive_for_request`, records the known assessment's challenge result, submits a `LabelSpec`, flushes labels, and derives the next estimate. The completed skeleton supplies the seeded world, virtual clock, name registry, lifecycle barriers, label builder, tolerances, and liveness deadlines (´entry:assayer:harness-stage-skeleton´).

The existing `LabelSpec` builder combines `Action::Challenge`, adverse valence, ground truth, a challenge result, and axis outcomes. `World::record_challenge_result` uses the derived assessment identifier to recover the channel, while `World::label` submits the Core payload and `World::flush_labels` crosses the deterministic post-label barrier.

The published `ModelSnapshot` exposes operational, sister, anchor, and per-axis means and covariances after the one end-of-path swap (´dec:ordering:publish-at-end´). The crate-level escape hatch also reaches `OutcomeLedger::snapshot` and the identity dimension cell state, so the witness can inspect internal effects without widening the production API.

The public ground-truth integration witness proves only that a challenged or blocked label is accepted and leaves a well-formed next result (´test:integration:ground-truth-flag-overrides-block-eligibility´). Its crate-level neighbour proves only that ground truth advances the eligible class rate (´test:crate:steps4-8-ground-truth-overrides´).

The nearest Companion tests separately prove an external tracker update (´test:crate:companion-boundary-tracker-updates-outside-core´) and assessment-identifier routing through `World` (´test:integration:world-routes-challenge-result-to-companion-tracker´). None joins those effects to the model triple, reported Ledger ancestry, identity cells, one outcome axis, and the next estimate.

The standing testing plan contains no gap entry dedicated to this promise. Its open tape and state-inspection stage is the nearest infrastructure entry, but this witness needs only a narrow one-label projection rather than the full streaming facility (´entry:assayer:harness-stage-tapes´).

## The witness · `sec:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-witness`

**Decision (One assessment identifier joins the host and Core effects)** · `dec:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-one-identifier-joins-effects`

The witness belongs beside the crate-level label-pipeline tests: `World` drives the public request and label APIs, while crate visibility exposes the otherwise unpublished per-model, Ledger, and identity state. This placement preserves the Companion boundary and avoids converting diagnostic test state into a host contract.

- **Setup.** Build one seeded default-channel world on a fixed `VirtualClock`; register one Sentinel and ingest `golden_report_4cell`; register one spatial axis; settle the cold ramp with the exact Sentinel-bearing request; register one identity dimension; block its maintenance loop; install neutral nested cells at depths zero, four, and eight; and publish their lifecycle entries before the measured request.

- **Baseline.** Derive the measured request once so its assessment identifier maps to the channel and its pending entry records the Sentinel and identity routes. Before recording any outcome, capture all three model means, the chosen axis mean, the Companion posterior, all four routed Ledger entries, all three identity cell outcome states, and the returned risk basis. Assessment itself changes no outcome-learned state (´dec:ordering:evidence-authority´).

- **Stimulus.** Assert that `World::record_challenge_result` accepts that assessment identifier with `ChallengeResult::Pass`, then submit exactly one `LabelSpec::adverse(id).action(Action::Challenge).ground_truth().outcome(axis, 2.0)` through `World::label` and cross `World::flush_labels`. No other label enters the world.

- **Observation.** Capture the same internal projection after the barrier, then assess the same entity at the same Sentinel coordinate for the next risk basis. Every comparison uses the baseline taken after the measured assessment, so report ingestion, lifecycle publication, and ramp settlement cannot masquerade as label effects.

- **Assertion.** Require both class-rate increases; operational, sister, anchor, and axis mean movement; the exact Companion pass-count delta; exact Challenge and eligible-count deltas at all four Ledger depths; positive Ledger and identity EWMAs with the reported axis present at every routed cell; one publication advance; and directional increases in all three public risk probabilities.

- **Join.** Carry the assessment identifier into the Companion call and `LabelSpec`, resolve Ledger entries from the Sentinel and golden coordinate, and resolve identity cells from the active set captured in that assessment. No assertion may substitute the latest arbitrary cell, a second entity, a second assessment, or an aggregate from another channel.

- **Fails-before.** A deliberately broken implementation that omits one ordered update leaves its corresponding before-and-after component equal: skipping eligible training leaves sister or anchor still, updating only the deepest Ledger entry leaves an ancestor count unchanged, consulting a different identity set leaves a recorded cell neutral, dropping the axis outcome leaves its model and per-axis maps unchanged, or losing the identifier route leaves Companion β at one. Publishing a partial working copy or detaching the update from the pending context likewise breaks the joined projection or the next-estimate assertions.

## What is missing · `sec:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-missing-support`

**Entry (An identity-maintenance block guard fixes the active chain)** · `entry:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-identity-maintenance-block`

A test-only maintenance command and RAII guard block the identity maintenance loop after registration and acknowledge entry before the fixture publishes its competitive set. The existing cell-entry lifecycle submission installs matching model coordinates, and the guard remains held until the post-label assessment is captured, eliminating a scheduler-dependent replacement of the recorded set.

The estimate is roughly 45–70 lines across the identity-maintenance loop and world harness support. It depends on existing bounded acknowledgement channels and `ACK_DEADLINE`, introduces no production-visible method, and has no sibling-lane dependency.

**Entry (World exposes the channel posterior to crate tests)** · `entry:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-companion-posterior-probe`

A crate-visible `World` accessor locks the host-owned tracker and returns its decayed `ChallengePosterior` for a named channel at the world's virtual timestamp. It is a read-only test probe, so raw $α$ and $β$ can be compared without inferring counts from a rendered derivation tag.

The estimate is roughly 10–20 lines in the world harness support. It depends on the existing `ChallengeEffectivenessProvider` implementation and name registry, and on no other entry or lane.

**Entry (The joined state projection keeps one baseline)** · `entry:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-joined-state-projection`

A test-local projection clones the published model and axis means, the four golden Ledger entries, the three installed identity outcome states, and the Companion posterior into one diagnostic value. Assertion helpers report the subsystem, depth or cell, field, before value, after value, and applicable named tolerance.

The estimate is roughly 55–80 lines in the crate-level label-pipeline test module. It depends on the identity guard (´entry:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-identity-maintenance-block´), the Companion probe (´entry:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-companion-posterior-probe´), the golden report fixture, and the existing tolerance bundle; it does not depend on the complete tape stage or another intent lane.

**Entry (The composite label-pipeline witness owns the promise)** · `entry:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-composite-witness`

One crate test performs the setup, stimulus, joined diff, next-assessment comparison, and module-index update described above. The promise mint moves from the intent catalogue to the test documentation, one new test label names the witness, and catalogues cite both without duplicating either mint.

The estimate is roughly 70–100 lines in the crate-level label-pipeline test module plus the existing generated or curated test indexes. It depends on the joined projection (´entry:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-joined-state-projection´) and introduces no cross-lane dependency.

## Risks and open questions · `sec:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-risks-open-questions`

**Observation (Both core models and the model triple use different counts)** · `obs:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-model-count-wording`

The promise says both core models, while the normative state is an operational, sister, and anchor triple. One reading limits the promised pair to the two full-feature estimates exposed in `RiskBasis`; the other reads whole engine literally and includes the eligible anchor. The witness proceeds with all three means, which contains the narrower reading and detects a disconnected anchor, while acceptance reports the anchor separately so a later wording decision can narrow rather than silently broaden the promise.

**Observation (At once is causal composition, not cross-owner atomicity)** · `obs:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-causal-not-atomic`

The Companion has no Core interface (´inv:companion:boundary´), so no single Core call can atomically commit both states. The meaningful invariant is that the host records the pass for the same issued assessment identifier immediately beside the one Core label and that every Core effect reads that assessment's retained context; requiring a combined transaction would contradict the ownership record rather than strengthen the witness.

**Observation (The fixed identity chain isolates label-time fan-out)** · `obs:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-identity-fixture-scope`

The witness assumes that three cells are already competitive and tests their update from the recorded active set. Competitive-cell formation has separate witnesses; allowing live maintenance here would add timing and graph-threshold variables without adding evidence about whether one label reaches every recorded cell.

**Observation (Direction replaces brittle posterior constants)** · `obs:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-directional-thresholds`

Bayesian updates depend on the complete standardised feature vector and registered layout, so hard-coded posterior coordinates would couple this promise to unrelated fixture evolution. Exact integer and membership deltas carry exact assertions, while model and next-risk movements use the declared default tolerance and memory averages use the declared EWMA tolerance; the fixed clock removes elapsed-time decay from both comparisons.

## Acceptance · `sec:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-acceptance`

The implementing lane's report identifies the new crate witness and its test index, confirms that the intent catalogue now cites the claim minted by that witness, and lists every production or testing file changed for the two narrow probes and the test.

The report records the single assessment identifier, the before and after Companion pseudo-counts, the operational, sister, anchor, and axis movement magnitudes, the four Ledger depths and their exact count deltas, the three identity cells and their updated fields, the publication versions, and the three next-risk deltas with their named tolerances.

The verification evidence shows the focused crate witness passing, the existing ground-truth and Companion-routing neighbours remaining green, and the package's prescribed formatting, lint, feature, documentation, and test gates succeeding. Any unrelated failure is separated from the witness result.

The report states that the identity guard held across both observations, that only one label was submitted, that no mutation was executed for fails-before, and which model-count interpretation was retained. Acceptance excludes aggregate-only evidence, a live-maintenance timing allowance, a second assessment as the label source, and inference of Companion counts from rendered output.
