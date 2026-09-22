# Plan for Multiplicative Label-Indexed and Time-Indexed Decay · `plan:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively`

This plan keeps (´claim:numerics:label-indexed-and-time-indexed-decay-compose-multiplicatively´) by making fixed historical evidence, the production label path, and both published class-rate trackers answer to one deterministic two-clock witness.

## What the promise says, precisely · `sec:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-precision`

The independent mechanisms are a label count $n$ with per-label retention $γ$ and an elapsed interval $Δt$ in hours with hourly retention $γ_t$; their joint factor is $F(n, Δt) = γ^n γ_t^{Δt}$, with no correction term between the two exponents (´def:temporal:two-mechanisms´).

For fixed historical posterior state, precision becomes $B_1 = F B_0$, covariance becomes $Σ_1 = Σ_0/F$, and the mean remains $μ_1 = μ_0$. A hibernated block receives exactly that treatment when labels and elapsed time pass while it is outside the update stream, so advancing either index first must produce the same restored state (´alg:temporal:lazy-application´) (´alg:registry:hibernation´).

For a live label update, the old posterior is discounted by one combined factor before the new rank-one evidence is added. The factor is computed and applied once per model per label (´dec:posterior:combined-factor´), and elapsed decay occurs at the head of the label path rather than discounting the evidence just added (´dec:ordering:decay-at-head´).

The operational model uses $γ_\text{opr}$, the sister and anchor use $γ_\text{inh}$, and all Core models use $γ_{t,\text{core}}$; the shipped rates and half-lives stand in (´tab:risk:forgetting-rates´), and the complete assignment of mechanisms to quantities stands in (´tab:temporal:decay-inventory´).

The global class-rate tracker follows $P'_g = γ_\text{opr} P_g + (1-γ_\text{opr})I_+$ on every label, and the eligible tracker follows the same recurrence with $γ_\text{inh}$ on eligible labels only (´tab:weighting:trackers´) (´alg:weighting:tracker-update´).

Elapsed time is absent from both tracker recurrences. A drought preserves each published $P_+$ exactly, and the next qualifying label applies one label-indexed recurrence without a factor of $γ_t^{Δt}$ (´dec:weighting:no-time-decay´).

The witness uses legal separating values $γ_\text{opr}=0.8$, $γ_\text{inh}=0.9$, $γ_t=0.95$, $n=3$, and $Δt=4$ hours (´tab:config:risk-model´) (´tab:config:temporal´). The time factor is $0.81450625$, the operational product is $0.4170272$, and the sister product is $0.59377505625$; each product is separated from either constituent by far more than the harness's default $10^{-9}$ comparison bound (´const:assayer:scenario-tolerance-defaults´).

## What the code offers today · `sec:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-current-surface`

The unified `Scenario` and its `World` build the real `Assayer` through `WorldBuilder`. `World::advance` and `World::travel_to` move both typed clock readings forward and settle the work their movement makes due, while `World::flush_labels` and `World::flush_observations` provide the queue-specific barriers this witness needs (´tab:assayer:harness-implementation-library-roster´) (´entry:assayer:harness-scenario-time´) (´entry:assayer:harness-closed-barriers´) (´dec:harness:single-scenario´).

`World::published_model_block` now returns an owned `PublishedModelBlock` after the label-publication barrier and one snapshot load. It supplies a complete model mean and covariance but deliberately supplies neither precision nor the published dimension map, so it cannot isolate one named Sentinel's range by itself (´entry:assayer:harness-probe-contract´) (´dec:harness:probe-contract´) (´cav:retention:probe-boundary´).

The independent `decay_recurrence` oracle computes the specification formula without using the production exponentiation path and declares `OracleProvenance::SpecificationFormula`; it replaces the plan's hand-computed expected factors (´entry:assayer:harness-oracle-tier´) (´dec:harness:oracle-tier´).

The shared roster also contains `PlaybackRow`, `PlaybackBarrierPolicy`, `TrainedStateFixture`, and `run_seeded_sweep` (´tab:assayer:harness-implementation-library-roster´). This witness has three fixed labels, begins from a declared cold state rather than a trained state, and quantifies over no generated population, so declarative playback, the guarded trained fixture, and a seeded sweep would add no coverage here (´dec:harness:declarative-playback´) (´dec:harness:guarded-fixtures´) (´dec:harness:seeded-sweeps´).

The public lifecycle surface supplies `Assayer::hibernate_sentinel` and `Assayer::register_sentinel`; `World::assayer` and `World::sentinel` supply the engine and stable identifier, and `World::flush_labels` is the publication barrier after both lifecycle calls. `Assayer::health_summary` publishes `p_positive_global` and `p_positive_eligible`, so the tracker half of the promise needs no internal-state probe (´tab:assayer:harness-implementation-library-roster´).

The production label path forms the operational, sister, anchor, and axis products and applies standalone elapsed decay only to a model skipped by that label (´dec:posterior:combined-factor´). Hibernation restore uses each archived block's label count and elapsed interval through the shared decay functions (´alg:registry:hibernation´) (´dec:clock:shared-functions´).

No existing test cites the claim this plan keeps. `decay_composition_property` composes two elapsed intervals but has no label-indexed factor (´test:integration:decay-composition-property´); `combined_decay_equivalence` compares a direct Bayesian model's combined update with a time-then-label control but does not drive the production label path or trackers (´test:crate:combined-decay-equivalence´); `hibernated_sentinel_returns_aged_on_both_clocks` assigns the intervening label sequence directly, restores in one event order, and does not inspect trackers (´test:crate:hibernated-sentinel-returns-aged-on-both-clocks´); and `learning_policy_conforms_row_by_row` proves the two tracker rates without moving scenario time (´test:crate:learning-policy-conforms-row-by-row´).

## The witness · `sec:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-witness`

Add the integration test target `numerics` function `numerics::label_and_time_decay_compose_without_moving_class_rates`. It uses `World` for construction, time, assessment, labels, lifecycle calls, barriers, and health, and uses only owned test-support projections for model state (´dec:harness:real-engine´) (´dec:harness:probe-contract´).

- Archived setup: construct two worlds with the separated rates above, $P_{+,0}=0.4$, the specified default prior precision $0.1$, the specified default replenishment floor $0.001$, identical seeds, and one registered Sentinel whose named operational and sister covariance blocks are captured before hibernation (´tab:config:risk-model´).

- Archived stimulus: call `Assayer::hibernate_sentinel` with the stable identifier in both worlds and cross `World::flush_labels`. In the time-first world call `World::advance` by four hours and then process three eligible benign labels; in the label-first world process the same three labels and then call `World::advance` by four hours. Re-register the same identifier in each world before archive expiry and cross the label barrier after each registration (´entry:assayer:harness-scenario-time´) (´entry:assayer:harness-closed-barriers´).

- Archived observation: obtain each returned Sentinel's named operational and sister mean and covariance block through the owned one-load projection described below, and obtain both class rates through `Assayer::health_summary`. The projection owns its values and exposes no model or snapshot handle (´dec:harness:probe-contract´).

- Archived assertion: use `decay_recurrence` to derive the operational factor $0.4170272$ and sister factor $0.59377505625$. Each returned covariance block must equal its captured block divided by the applicable factor, each mean must be unchanged, and the two event orders must agree within `World::tol().default`; the existing direct crate witness retains the precision-times-factor assertion that the probe boundary excludes (´dec:harness:oracle-tier´) (´test:crate:hibernated-sentinel-returns-aged-on-both-clocks´).

- Tracker assertion: immediately after each four-hour drought, both class rates must remain bit-identical to their pre-drought values. After three eligible benign labels, the global rate must be $0.4\cdot0.8^3$ and the eligible rate $0.4\cdot0.9^3$ in both worlds; neither expectation contains $0.95^4$ (´dec:weighting:no-time-decay´).

- Live-path control: construct two further identical worlds with $P_{+,0}=0.1$, importance ceiling $1$, and leverage safety factor `f64::MAX` (´tab:config:risk-model´). Advance one world by four hours under rates $0.8$ and $0.9$, and leave the other at zero elapsed time with rates $0.651605$ and $0.733055625$; their post-label positive-class rates are respectively $0.28$ against $0.4135555$ and $0.19$ against $0.3402499375$, all below one half, so the ceiling makes the positive eligible label's effective weight exactly $1$ while the finite leverage cannot bind. Capture identical assessments before the advance, cross `World::flush_observations` in both worlds, submit identical labels, and cross `World::flush_labels` (´entry:assayer:harness-closed-barriers´).

- Live-path assertion: use `World::published_model_block` to compare the operational and sister mean and covariance payloads, excluding projection metadata, within `World::tol().default`. Equality establishes that the production update received one factor of $0.8\cdot0.95^4$ or $0.9\cdot0.95^4$ before adding identical evidence; a dropped factor, a doubled factor, or a factor applied after the new evidence makes the payloads differ (´dec:posterior:combined-factor´) (´dec:ordering:decay-at-head´).

The fails-before is direct: either constituent in place of the archived product gives the wrong covariance, applying elapsed decay twice makes it too wide, reversing the order around the live rank-one evidence breaks the collapsed-rate control, and introducing elapsed decay into either tracker adds an observable factor of $0.95^4$.

## What is missing · `sec:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-missing`

**Entry (A read-only named Sentinel block probe)** · `entry:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-block-probe`

The whole-model probe has landed, but the private `testing::probes` module still lacks an owned projection of one named Sentinel's mean and covariance sub-block. Its `World` entry point must cross `World::flush_labels`, resolve the name and range from the same published snapshot that supplies both model payloads, copy the permitted values, and return no precision, borrow, guard, shared owner, or mutation route (´entry:assayer:harness-probe-contract´) (´dec:harness:probe-contract´) (´cav:retention:probe-boundary´).

The projection is the remaining slice-specific application of the finished probe contract, not a new production accessor. Precision remains at the existing direct crate-level witness because the retention decision excludes it from test-support projections.

**Entry (A paired two-clock scenario fixture)** · `entry:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-paired-fixture`

The remaining fixture work is local to the `numerics` integration target: constructors for the archived and live paired worlds, a helper that hibernates and re-registers one stable identifier through `World::assayer`, and a three-label helper that uses `World::cycle_on` and its label barrier. Each constructor declares the rates, seed, prior, floor, tracker start, importance ceiling, and leverage setting it relies on; no state-establishing shared fixture is missing (´tab:assayer:harness-implementation-library-roster´).

No clock verb, queue barrier, playback runner, analytical oracle, trained-state fixture, seeded-sweep runner, health accessor, or production lifecycle method remains missing (´tab:assayer:harness-implementation-library-roster´).

## Risks and open questions · `sec:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-risks`

**Observation (Commutativity applies to fixed evidence)** · `obs:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-fixed-evidence`

Moving a new label's arrival time is not a valid commutativity oracle: time after that arrival legitimately discounts the evidence the label added, while time before it does not. Hibernation holds one historical block outside the update stream, so reversing the two clock advances changes neither the evidence being aged nor either exponent.

**Observation (The replenishment floor can erase the oracle)** · `obs:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-floor-margin`

An archived block below the replenishment floor returns at the prior, making several wrong factors observationally identical. The smaller operational factor is $0.4170272$, so prior precision $0.1$ becomes $0.04170272$, more than forty times the $0.001$ floor; the test asserts this derived precondition before comparing blocks (´tab:config:risk-model´).

**Observation (Asynchronous publication is bounded by existing barriers)** · `obs:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-publication-order`

Lifecycle, label, and cold-ramp observation work remain asynchronous even though time is deterministic. Every lifecycle and label observation follows `World::flush_labels`, each pending-assessment setup follows `World::flush_observations`, and each time movement goes through `World::advance`; no sleep, poll, or local deadline supplies ordering (´entry:assayer:harness-closed-barriers´) (´dec:harness:no-ad-hoc-waits´).

**Observation (The active and archived paths are both necessary)** · `obs:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-two-paths`

The archived arm provides the clean order oracle but needs the still-missing named covariance slice because `PublishedModelBlock` exposes neither a Sentinel range nor precision. The collapsed-rate arm exercises the active label path through the landed whole-model probe but cannot establish commutativity for evidence added between the two clocks. Keeping both arms, while leaving the excluded precision assertion with its existing crate witness, prevents either implementation site from standing in for the other (´dec:harness:probe-contract´) (´cav:retention:probe-boundary´).

No maintainer decision remains open: the specification fixes the formula, model-specific rates, tracker exception, and position of elapsed decay relative to new evidence.

## Acceptance · `sec:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-acceptance`

The integration target `numerics` contains `numerics::label_and_time_decay_compose_without_moving_class_rates`, and its documentation cites (´claim:numerics:label-indexed-and-time-indexed-decay-compose-multiplicatively´). Its test index states the archived-order, live-path, and no-time-tracker coverage without claiming the excluded precision reading.

The implementation report names the owned Sentinel covariance projection and its test-support gate; the exact rates, label count, elapsed hours, analytical factors, tracker starts, importance and leverage controls, floor margin, and named comparison tolerance; and the existing precision witness that completes the state-transition evidence.

The report shows the new test failing against one deliberate local break from each defect class before the break is removed: a missing or doubled posterior factor, elapsed decay applied after new evidence, and elapsed decay introduced into a class-rate tracker. These are described mutation checks, not retained mutations.

Formatting, clippy, the focused `torrust-assayer` integration target in development and release profiles, documentation tests, and the package test suite are green. The generated test index and coverage report resolve the new test to (´claim:numerics:label-indexed-and-time-indexed-decay-compose-multiplicatively´), leave no duplicate mint, and remove the claim from the uncovered-intent set.
