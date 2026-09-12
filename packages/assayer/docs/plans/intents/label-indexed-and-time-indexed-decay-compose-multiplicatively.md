# Plan for Multiplicative Label-Indexed and Time-Indexed Decay · `plan:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively`

This plan keeps (´claim:numerics:label-indexed-and-time-indexed-decay-compose-multiplicatively´) by making the live posterior, fixed historical evidence, and both published class-rate trackers answer to one deterministic two-clock witness.

## What the promise says, precisely · `sec:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-precision`

The two independent mechanisms are a label count $n$ with per-label retention $γ$ and an elapsed interval $Δt$ in hours with hourly retention $γ_t$; their only joint factor is $F(n, Δt) = γ^n γ_t^{Δt}$ (´def:temporal:two-mechanisms´).

For posterior state that receives no new evidence while the clocks advance, precision must become $B_1 = F B_0$, covariance must become $Σ_1 = Σ_0/F$, and the mean must remain $μ_1 = μ_0$. Applying the label factor before the time factor or the time factor before the label factor must produce the same three results.

For a live label update, the old posterior is discounted once by the one combined factor before the new rank-one evidence is added. The factor is computed once per model per label (´dec:posterior:combined-factor´), and elapsed decay occupies the head of the label path rather than acting on the evidence just added (´dec:ordering:decay-at-head´).

The operational model uses $γ_\text{opr}$, the sister and anchor use $γ_\text{inh}$, and all use the core hourly rate $γ_{t,\text{core}}$; the shipped values and their half-lives stand in (´tab:risk:forgetting-rates´), while the wider inventory of quantities that do and do not take each mechanism stands in (´tab:temporal:decay-inventory´).

The global class-rate tracker follows $P'_g = γ_\text{opr} P_g + (1-γ_\text{opr})I_+$ on every label, and the eligible tracker follows the same recurrence with $γ_\text{inh}$ on eligible labels only (´tab:weighting:trackers´) (´alg:weighting:tracker-update´).

Elapsed time is absent from both tracker recurrences. A drought preserves each published $P_+$ exactly, and the next qualifying label applies one label-indexed recurrence rather than a recurrence multiplied by $γ_t^{Δt}$ (´dec:weighting:no-time-decay´).

The witness uses legal, deliberately separated fixture values $γ_\text{opr}=0.8$, $γ_\text{inh}=0.9$, $γ_t=0.85$, $n=3$, and $Δt=4$ hours. The product factors remain well above the default replenishment floor and far enough from either single factor that the harness's default $10^{-9}$ comparison bound cannot hide a dropped or doubled factor.

## What the code offers today · `sec:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-current-surface`

The world harness builds the real `Assayer` through `WorldBuilder`, injects one `VirtualClock` into the engine, exposes it through `World::clock()`, and provides assessment, label, cycle, and `flush_labels` verbs (´entry:assayer:harness-stage-skeleton´). The virtual clock already advances both timestamp domains deterministically, in accordance with the two-domain clock boundary (´dec:clock:two-domains´).

The promise has a standing testing-plan dependency: the persistent-clock stage names this exact claim among the decay-driven surfaces it gates (´entry:assayer:harness-stage-clock´). The core label path and hibernation restore already read the injected clock, so this witness does not wait for every remaining persistent-time call site or for a new clock abstraction.

The public lifecycle surface supplies `Assayer::hibernate_sentinel` and `Assayer::register_sentinel`, while `World::assayer()` and `World::sentinel()` expose the engine and stable identifier needed to call them. `World::flush_labels()` is the publication barrier after hibernation, re-registration, and every submitted label.

`Assayer::health_summary()` publishes `p_positive_global` and `p_positive_eligible`, so the no-time-decay half of the promise has a public scalar oracle and needs no internal tracker access.

The label path forms `gamma_opr * gamma_t_dt`, `gamma_inh * gamma_t_dt`, and each axis's corresponding product for updated models; mutually exclusive standalone calls apply time decay only to models skipped by that label (´dec:posterior:combined-factor´). Hibernation restore independently computes the product of label-count and elapsed-time factors for archived state, with both exponentiations routed through the shared decay functions (´alg:registry:hibernation´) (´dec:clock:shared-functions´).

The nearest integration witness proves that elapsed-time factors compose over consecutive intervals but never mixes a label index with a time index (´test:integration:decay-composition-property´). The nearest crate witness checks one hibernation restore against a hand-computed two-clock product but advances the label sequence by assignment, exercises only one event order, and does not inspect the class-rate trackers (´test:crate:hibernated-sentinel-returns-aged-on-both-clocks´).

The existing learning-policy witness proves the tracker recurrence and model-rate selection without advancing virtual time (´test:crate:learning-policy-conforms-row-by-row´). None of these tests fails when elapsed time is incorrectly introduced into the trackers or when the two indices cease to commute for fixed evidence.

## The witness · `sec:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-witness`

The integration test belongs beside the existing numerical-decay scenarios and is named `label_and_time_decay_compose_without_moving_class_rates`. It drives production construction, assessment, label, health, and lifecycle methods through `World`; the test-only probe reads state but performs no mutation.

- Setup: construct paired worlds with the separated rates above, the default prior precision $0.1$, the default replenishment floor $0.001$, identical seeds, and a single registered Sentinel whose prior block is captured before hibernation.

- Fixed-evidence stimulus: hibernate the Sentinel in both worlds, advance the first world by four hours and then process three eligible benign labels, process the same three labels first in the second world and then advance it by four hours, and re-register the same Sentinel identifier in both worlds before the archive expiry.

- Fixed-evidence observation: after each lifecycle and label barrier, read the returning Sentinel's operational and sister mean, precision diagonal, and covariance diagonal through the narrow probe, and read both class rates through `health_summary()`.

- Fixed-evidence assertion: every returned operational precision entry equals its captured value times $0.8^3 0.85^4$, every sister entry uses $0.9^3 0.85^4$, covariance uses the reciprocal factors, means are unchanged, and the two event orders agree within `world.tol().default`.

- Tracker assertion: starting from a configured $P_{+,0}=0.4$, three eligible benign labels leave the global rate at $0.4\cdot0.8^3$ and the eligible rate at $0.4\cdot0.9^3$ in both worlds; neither expected value contains $0.85^4$.

- Live-path control: construct one more paired fixture from identical cold state, pin both initial class rates at `0.999` with one positive eligible label so its importance weight is identical in both arms, advance one arm by four hours under the original rates, and leave the other arm at zero elapsed time with each label rate replaced by that model's original rate times $0.85^4$.

- Live-path assertion: after one label and one barrier, the probed operational and sister blocks agree between the elapsed-time arm and the collapsed-rate arm. Equality establishes that the live path supplied the same single effective factor to the posterior update; a dropped factor, a second standalone application, or a factor attached after the new evidence makes the blocks diverge.

The test's fails-before is concrete. Replacing the product with either constituent leaves the returned precision too large, adding the constituents leaves it larger still, applying the time factor twice leaves it too small, charging time after the new rank-one evidence breaks the live-path control, and applying time decay to either class tracker introduces the absent $0.85^4$ term. An implementation whose result depends on which clock advanced first makes the paired archived blocks disagree.

## What is missing · `sec:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-missing`

**Entry (A read-only named Sentinel block probe)** · `entry:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-block-probe`

The missing harness piece is an integration-test-only value type in the world harness carrying a named Sentinel block's mean, precision diagonal, and covariance diagonal for the operational and sister models, plus a `World` accessor that takes one published snapshot and resolves the Sentinel's current range from that same snapshot. The estimate is approximately 60 lines including documentation and export wiring.

The probe is a narrow slice of the snapshot inspection anticipated by (´entry:assayer:harness-stage-tapes´), but it depends only on the existing `World`, name registry, and published snapshot and therefore does not depend on that broader entry or another lane. It exposes immutable numeric copies, never a model handle, and its single-snapshot read prevents a lifecycle publication from mixing a range from one layout with matrices from another.

**Entry (A paired two-clock scenario fixture)** · `entry:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-paired-fixture`

The missing fixture is a test-local builder and three small verbs in the numerical-decay integration module: construction of the configured world, hibernation and re-registration of the stable Sentinel identifier through the public API escape hatch, and processing of a stated count of eligible labels with a barrier. The estimate is approximately 35 lines, and it depends on (´entry:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-block-probe´) with no dependency on another lane.

No new clock verb, tape abstraction, liveness deadline, drift budget, or production accessor is missing. `World::clock().advance`, `World::cycle_on`, `World::flush_labels`, the lifecycle API, and `HealthSummary` already provide those parts.

## Risks and open questions · `sec:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-risks`

**Observation (Commutativity applies to fixed evidence)** · `obs:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-fixed-evidence`

Moving the arrival time of a new label is not a valid commutativity oracle: time after that arrival legitimately discounts the evidence the label added, while time before it does not. Hibernation holds one historical block outside the update stream, so reversing the clock events changes neither the evidence being aged nor either exponent.

**Observation (The replenishment floor can erase the oracle)** · `obs:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-floor-margin`

An aged block below the replenishment floor is restored at the prior, making several wrong factors observationally identical. The selected rates and interval keep the smaller operational factor above one quarter, so a prior precision of $0.1$ remains more than an order of magnitude above the $0.001$ floor after decay; the test asserts this fixture precondition before comparing blocks.

**Observation (Asynchronous publication is bounded by existing barriers)** · `obs:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-publication-order`

Lifecycle and label work is asynchronous even though the clock is deterministic. Every observation follows `flush_labels()`, and the paired worlds never share a clock, engine, or identifier registry, so scheduler order cannot alter either $n$ or $Δt$.

**Observation (The active and archived paths are both necessary)** · `obs:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-two-paths`

The archived-block arm provides the clean order oracle but exercises the restore's product; the collapsed-rate arm exercises the active label path but cannot by itself distinguish the age of old evidence from the arrival time of new evidence. Keeping both arms prevents either implementation site from standing in for the other.

No maintainer decision remains open: the specification fixes the formula, the model-specific rates, the tracker exception, and the order in which new evidence enters.

## Acceptance · `sec:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively-acceptance`

The implementation report identifies the added integration test and its claim citation, the block-probe API and its test-only gate, and the exact fixture rates, label count, elapsed hours, analytical factors, floor margin, and tolerance used.

The report shows the integration test failing against at least one deliberate local break from each class before the break is removed: one missing or doubled posterior factor, one wrong order that discounts newly added evidence, and one time factor introduced into a class-rate tracker. These are described mutation checks, not retained mutations.

The report shows formatting, clippy, the focused `torrust-assayer` integration test in development and release profiles, a `--no-default-features` spot check where the test-support gate permits it, documentation tests, and the package test suite all green, with command output and exit status for each gate.

The report also shows that the generated test index and coverage report resolve the new test to (´claim:numerics:label-indexed-and-time-indexed-decay-compose-multiplicatively´), leaving no duplicate mint and removing this statement from the uncovered-intent set.
