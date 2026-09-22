# Planning One Investigated Label Across Every Subsystem · `plan:assayer:intent-one-investigated-label-moves-every-subsystem-at-once`

This plan keeps the promise that one challenged, passed, investigated request moves every outcome-learning subsystem through the same label (´claim:labelling:one-investigated-label-moves-every-subsystem-at-once´); keeping it establishes joint causality rather than another collection of separately passing mechanism tests.

## What the promise says, precisely · `sec:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-promise`

The witness begins with one issued assessment whose pending entry retains the request, reporting Sentinel, extracted chain, identity coordinate and active cells, and registered outcome axis required by the later write path (´def:runtime:pending-entry´). The label for that assessment records `Action::Challenge`, `ChallengeResult::Pass`, positive valence, `ground_truth = true`, and one positive outcome-axis value.

Positive valence gives the risk target $y=+1$ (´def:risk:target´), and established ground truth makes the label eligible independent of the recorded action (´tab:eligibility:training´) (´conv:eligibility:ground-truth´). The global and eligible positive-class rates therefore both move upward from their neutral baselines.

The operational and sister mean vectors each move by more than `DEFAULT_TOLERANCES.default`, which is $10^{-9}$, and the anchor mean also moves under the stronger whole-engine reading of the normative model triple (´def:risk:model-triple´). The next assessment of the same entity and Sentinel coordinate has `RiskBasis.p_bad`, `p_bad_operational`, and `p_bad_sister` each greater than its pre-label value by more than that same tolerance (´schema:risk:basis´).

The channel Companion begins at the uniform Beta prior and, with scenario time fixed, preserves $α=1$ while increasing the pass pseudo-count from $β=1$ to $β=2$ exactly (´def:companion:state´) (´def:companion:prior´) (´alg:companion:update´). This count is host-owned evidence beside the Core rather than a field inside its snapshot (´dec:challenge:host-ownership´).

The four-cell golden report supplies one Sentinel chain at depths zero, four, eight, and twelve (´dec:assayer:golden-report-stimulus´). At every one of those entries the Challenge action count, recent eligible count, adverse eligible count, and eligible-arrival load each increase by one; adverse-rate, raw-valence, and compressed-valence averages become positive; and the reported axis appears with positive raw and compressed values, as required by the all-layers write (´alg:ledger:all-layers-update´) and the depth-walk decision (´dec:memory:depth-walk´).

The guarded identity fixture establishes and verifies an exact non-empty competitive set declared for the entity. Every recorded cell in that data-dependent set changes from neutral to positive adverse-rate, raw-valence, and compressed-valence state and gains the reported axis in both per-axis maps, which measures the identity outcome update rather than assuming unsupported cell depths (´alg:runtime:identity-outcome-update´) (´dec:harness:guarded-fixtures´).

The registered axis model's mean moves by more than the default tolerance toward its positive compressed target (´def:axis:per-axis-model´) (´def:axis:training-target´). Ledger and identity EWMAs use `DEFAULT_TOLERANCES.ewma`, $10^{-6}$, while exact counts, memberships, identifiers, and untouched Companion $α$ use `DEFAULT_TOLERANCES.exact` (´tab:assayer:harness-scenario-tolerances´).

These observations cover the ordered label function's model, memory, axis, identity, monitoring, and publication stages in one before-and-after interval (´alg:runtime:update-path´) (´dec:ordering:label-function´). They establish that every movement consumes the same assessment identifier and retained pending context; they do not redefine the host and Core updates as one atomic transaction.

## What the code offers today · `sec:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-code-today`

The unified `World` owns the real engine, scenario clock, registry, tolerances, and lifecycle barriers (´tab:assayer:harness-implementation-library-roster´). It already registers a Sentinel, ingests `golden_report_4cell`, registers a spatial outcome axis, settles the cold ramp through `World::settle_cold_ramp_with`, builds a request with `World::request_with_sentinel`, derives it with `World::derive_for_request`, records its challenge result with `World::record_challenge_result`, submits a `LabelSpec` through `World::label`, and crosses the label queue through `World::flush_labels`.

`CompetitiveCellSpec` and `World::register_identity_with_cells` now establish an exact declared competitive set through real observations and refuse a mismatched set; the fixture crosses `World::flush_identity_maintenance` before it returns (´dec:harness:guarded-fixtures´) (´test:crate:register-identity-with-cells-observes-the-declared-set´). The former proposal to block the maintenance loop is therefore obsolete.

`World::published_model_block` returns an owned `PublishedModelBlock` after the publication barrier and one snapshot load, and `World::pending_entry_view` returns an owned `PendingEntryView` without consuming the pending assessment (´dec:harness:probe-contract´) (´test:crate:published-model-block-crosses-publication-barrier´) (´test:crate:pending-entry-view-is-non-consuming-and-keeps-storage-width´). The published probe covers one named model block, while the pending probe deliberately exposes signal storage and active-cell counts rather than the identifiers and routes this joined witness must compare.

`OutcomeLedger::snapshot` and `Assayer::health_summary` already provide owned readings for the Ledger and class-rate counters. No existing owned reading combines those values with identity-cell outcome state, every model block from one publication, the pending routes, and the host-owned Companion posterior.

Declarative `playback`, the `OracleProvenance` computations, `TrainedStateFixture`, and `run_seeded_sweep` have landed (´dec:harness:declarative-playback´) (´dec:harness:oracle-tier´) (´dec:harness:guarded-fixtures´) (´dec:harness:seeded-sweeps´). This witness has one stimulus, derives no analytical result covered by the oracle tier, needs a purpose-built guarded competitive set rather than a trained baseline, and quantifies over no seed space, so none is part of its execution.

The existing ground-truth witnesses establish eligibility override and its eligible-rate effect separately (´test:integration:ground-truth-flag-overrides-block-eligibility´) (´test:crate:steps4-8-ground-truth-overrides´). The Companion witnesses establish an external tracker update and assessment-identifier routing separately (´test:crate:companion-boundary-tracker-updates-outside-core´) (´test:integration:world-routes-challenge-result-to-companion-tracker´), while the Ledger all-layers, identity fixture, and model-probe witnesses establish their own mechanisms (´test:crate:all-layers-write-hits-all´) (´test:crate:register-identity-with-cells-observes-the-declared-set´) (´test:crate:published-model-block-is-from-one-version´). No test mints or cites this claim, and none joins those effects to the same pending assessment and label.

## The witness · `sec:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-witness`

**Decision (One assessment identifier joins the host and Core effects)** · `dec:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-one-identifier-joins-effects`

The integration target `label_integration` owns `label_integration::one_investigated_label_moves_every_subsystem_at_once`. `World` drives the real request and label APIs, and every diagnostic observation is an owned, read-only projection that follows the probe contract without widening the production surface (´dec:harness:probe-contract´).

- **Setup.** Build one seeded default-channel `World`; register one Sentinel and ingest `golden_report_4cell`; register one spatial axis; register one identity dimension through `World::register_identity_with_cells` with an exact non-empty `CompetitiveCellSpec`; and settle the cold ramp through `World::settle_cold_ramp_with`. Keep scenario time fixed, so neither `World::advance` nor `World::travel_to` is needed and no elapsed-time decay enters the comparison (´cor:clock:harness-control´).

- **Baseline.** Derive the measured request once with `World::derive_for_request`, assert that its retained route names the same assessment, reporting Sentinel chain, axis, identity dimension, and exact guarded competitive set, and capture the joined baseline before recording any outcome. Fixture guards and result assertions remain distinct (´dec:harness:separate-validation´).

- **Stimulus.** Require `World::record_challenge_result` to accept that assessment identifier with `ChallengeResult::Pass`, submit exactly one `LabelSpec::adverse(id).action(Action::Challenge).ground_truth().outcome(axis, 2.0)` through `World::label`, and cross `World::flush_labels`. The named barrier, rather than a sleep, poll, or deadline, establishes completion of the label queue (´cor:concurrency:harness-barriers´).

- **Observation.** Capture the joined post-label state, then call `World::derive_for_request` for the same entity and Sentinel coordinate and retain its `RiskBasis`. The state observation crosses the label barrier itself, loads the published model state once, and returns owned copies of the class rates, model means and version, routed Ledger entries, routed identity state, axis state, and Companion posterior (´dec:harness:probe-contract´).

- **Assertion.** Require both class-rate increases; operational, sister, anchor, and axis mean movement; the exact Companion pass-count delta; exact Challenge and eligible-count deltas at all four Ledger depths; positive Ledger and identity EWMAs with the reported axis present at every routed cell; one publication advance; and directional increases in all three next-risk probabilities, using only the declared scenario tolerances stated above.

- **Join.** Compare the route captured from the pending assessment with the Sentinel chain and competitive-cell identifiers carried by both state readings, and pass that same assessment identifier to the Companion call and `LabelSpec`. No assertion may substitute the latest arbitrary cell, a second entity, a second assessment, another channel, or an aggregate detached from the route.

- **Fails-before.** Omitting any ordered update leaves its corresponding before-and-after component equal: skipped eligible training leaves a promised model still, deepest-only Ledger writing leaves an ancestor count unchanged, a different identity set leaves a recorded cell neutral, a dropped axis outcome leaves its model and maps unchanged, and a lost identifier route leaves Companion $β$ at one. A partial publication or detached pending context likewise breaks the version, route, or next-estimate assertion; no production mutation or test-only fault switch is needed.

## What is missing · `sec:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-missing-support`

**Entry (World exposes the channel posterior to tests)** · `entry:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-companion-posterior-probe`

`testing::world` needs a read-only `World::challenge_posterior` entry point that resolves a named channel and returns its decayed `ChallengePosterior` at scenario time. It reuses the existing `ChallengeEffectivenessProvider` and returns raw $α$ and $β$ without inferring counts from rendered output or exposing mutation.

**Entry (The joined state projection keeps one routed baseline)** · `entry:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-joined-state-projection`

`testing::probes` needs owned `JoinedLabelRoute` and `JoinedLabelState` values plus `World` entry points in `testing::world`. The route is copied from the live pending entry before labelling and contains the assessment identifier, reporting Sentinel chain, spatial axes, identity dimensions, and competitive-cell identifiers; the state entry point crosses the label barrier, performs one published snapshot load, and copies the publication version, four model means, class rates, routed Ledger entries, routed identity outcome states, and Companion posterior. Both projections are read-only and bounded to fields the witness asserts (´dec:harness:probe-contract´).

**Entry (The composite label integration witness owns the promise)** · `entry:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-composite-witness`

The integration target `label_integration` needs `label_integration::one_investigated_label_moves_every_subsystem_at_once` to perform the setup, single-label stimulus, joined diff, and next-assessment comparison described above. Its documentation mints the promise and one test label; the test indexes and intent catalogue then cite those mints without duplicating them.

## Risks and open questions · `sec:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-risks-open-questions`

**Observation (Both core models and the model triple use different counts)** · `obs:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-model-count-wording`

The promise says both core models, while the normative state says: “Three Bayesian linear models estimate risk from the feature vector $\phi$. The operational model trains on every labelled outcome, whatever the host did; the sister model trains only on unconfounded outcomes; the counterfactual anchor model trains on the same unconfounded outcomes as the sister but over a fixed low-dimensional projection.” (´def:risk:model-triple´). The witness retains all three means, which contains the narrower pair reading, but this specification disagreement remains a wording decision rather than a number the plan may invent.

**Observation (At once is causal composition, not cross-owner atomicity)** · `obs:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-causal-not-atomic`

The Companion has no Core interface (´inv:companion:boundary´), so no single Core call can atomically commit both states. The invariant is that the host records the pass for the issued assessment identifier beside the one Core label and every Core effect reads that assessment's retained context; requiring a combined transaction would contradict the ownership record.

**Observation (The guarded competitive set replaces fixed depths)** · `obs:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-identity-fixture-scope`

Identity competition does not promise cells at predetermined depths. `World::register_identity_with_cells` instead verifies the exact non-empty set declared by the fixture and crosses identity maintenance before returning; the pending route must reproduce that set, and both joined state readings must use it. A later identity observation or time movement between the pending-route capture and the post-label state would invalidate that guard and is excluded.

**Observation (The joined probe must not mix publication versions)** · `obs:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-directional-thresholds`

Four separate `World::published_model_block` calls each satisfy the probe contract but do not prove that all four blocks came from one version. The joined state probe therefore performs one published load and reports its version, while exact counts and memberships use bit identity and directional model and EWMA comparisons use the declared scenario tolerances; no threshold is fitted to an observed run (´dec:harness:probe-contract´).

## Acceptance · `sec:assayer:intent-one-investigated-label-moves-every-subsystem-at-once-acceptance`

The integration target `label_integration` contains `label_integration::one_investigated_label_moves_every_subsystem_at_once`, and its test documentation mints the promise and a test label. The test index cites those mints, and the intent catalogue cites the resulting witness without duplicating either label.

The witness records the single assessment identifier, the exact guarded competitive set, the before and after Companion pseudo-counts, the operational, sister, anchor, and axis movement magnitudes from one publication per observation, the four Ledger depths and their exact count deltas, every routed identity cell and its updated fields, the publication versions, and the three next-risk deltas with their named tolerances.

Focused verification shows the new integration witness passing and the existing ground-truth, Ledger, identity-fixture, model-probe, and Companion-routing neighbours remaining green. Package formatting, lint, and test gates pass without an exemption.

Acceptance requires exactly one submitted label, fixed scenario time, completion through `World::flush_labels`, an unchanged pending route, and distinct fixture-precondition and result assertions. Aggregate-only evidence, a live-maintenance timing allowance, a second assessment as the label source, a mixed-version model reading, and inference of Companion counts from rendered output do not keep the promise.
