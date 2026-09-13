# Plan for Three Independent Decay Clocks · `plan:assayer:intent-three-decay-clocks-run-independently`

This plan keeps (´claim:ledger:three-decay-clocks-run-independently´) by making one entity expose the separate identity, Ledger, and model-retention curves under deterministic virtual time, establishing that none of the three configured rates governs either of the other two quantities.

## What the promise says, precisely · `sec:assayer:intent-three-decay-clocks-run-independently-precision`

The shipped hourly retentions are $\gamma_{t,I}=0.998$ for identity cell outcome averages, $\gamma_{t,L}=0.999$ for Ledger averages, and $\gamma_{t,M}=0.9999$ for all Core model parameters; these are three separately host-configurable fields (´tab:config:temporal´) in the complete decay inventory (´tab:temporal:decay-inventory´).

For an elapsed interval of $h$ hours with no new label, the identity retention is $R_I(h)=I_h/I_0=0.998^h$, the Ledger retention is $R_L(h)=L_h/L_0=0.999^h$, and model precision retention is $R_M(h)=V_{raw}/V_{corrected}=0.9999^h$. Equivalently, time-corrected model variance is inflated by $1/R_M(h)$ while the posterior mean is unchanged (´alg:temporal:lazy-application´) (´def:runtime:time-correction´).

The observable identity scalar is one seeded competitive cell's adverse-rate EWMA, a member of the outcome state defined by (´tab:keyspace:outcome-state´), rather than the graph's separately configured spatial importance. The observable Ledger scalar is the bad-rate EWMA of the entry selected by the same request's Sentinel route, whose pure read-time attenuation is fixed by (´def:ledger:time-decay´).

At fourteen days $R_I$ lies in $[0.50,0.52]$; at twenty-nine days $R_L$ lies in $[0.49,0.51]$; and at two hundred and ninety days $R_M$ lies in $[0.49,0.51]$, equivalently placing model variance inflation in $[1.96,2.04]$. At every positive common checkpoint the strict ordering is $R_I<R_L<R_M<1$.

The primary assertions compare each normalized reading with its exact power at the harness's generic $10^{-9}$ tolerance; the bands above state the promised approximate horizons and do not replace the analytical oracle. The longest interval remains below the timestamp interface's one-year clamp (´dec:clock:embedded-clamp´).

No labels arrive after the baseline, so label-indexed forgetting is absent and elapsed time is the only stimulus under the independent-mechanism definition (´def:temporal:two-mechanisms´). Model decay remains lazy at label time and is repaired at assessment time by the read-time correction, consistent with the head-of-path ordering (´dec:ordering:decay-at-head´).

## What the code offers today · `sec:assayer:intent-three-decay-clocks-run-independently-current-surface`

The testing harness's `World` implementation constructs the real `Assayer` with `WorldBuilder`, registers axes, dimensions, and Sentinels, drives assessment and labels, settles the cold standardisation ramp, and supplies `flush_observations` and `flush_labels` publication barriers.

The testing harness's `VirtualClock::advance` moves virtual time, and `World::clock()` exposes the same clock injected into both timestamp domains. The type separation and shared decay funnel are fixed by (´dec:clock:two-domains´) and (´dec:clock:shared-functions´).

The configured-scenario helper, report fixtures, and `LabelSpec` builder provide repeatable worlds, a stable reported Sentinel, and repeatable labels. `Tolerances` provides the analytical comparison bound, while `ACK_DEADLINE` provides the acknowledgement deadline for a new maintenance barrier.

Identity state implements pure reads with `CellOutcomeState::read_decayed` (´claim:identity:a-decayed-outcome-view-is-exactly-the-mutating-decay-result-without-changing-storage´); Ledger state implements the corresponding `LedgerEntry::read_at` under (´def:ledger:time-decay´); and the assessment pipeline computes the Core snapshot-age correction from `gamma_t_core` under (´def:runtime:time-correction´). The label path updates all three stored timestamps from the injected persistent clock when the baseline labels are processed, and its configuration projection carries every host-set pipeline field unchanged (´claim:labelling:every-pipeline-setting-is-carried-across-from-the-host-configuration´).

The standing testing plan names this promise under its partly completed persistent-clock entry (´entry:assayer:harness-stage-clock´). The assessment and label paths needed here already use the injected clock, and `World::clock().advance` is sufficient for this witness; the remaining direct wall-clock constructors are neutralized when the seeded labels replace their initial timestamps.

The arithmetic integration tests establish the twenty-nine-day, two-hundred-and-ninety-day, and identity half-life calculations independently (´test:integration:decay-factor-29-day-half-life´) (´test:integration:decay-factor-290-day-half-life´) (´test:integration:decay-factor-identity-half-life´).

The nearest stateful witnesses still isolate one mechanism at a time: the Ledger timeline manually rewinds one entry (´test:crate:ledger-decay-twenty-nine-day-half-life-timeline´), the identity unit compares a cloned decayed view (´test:unit:cell-outcome-state-decayed-view-is-exact-and-pure´), and the model unit passes an already chosen correction factor into the blend (´test:unit:time-correction-affects-variance´). None drives one entity through all three configured rates or detects two configuration fields wired to the same consumer.

## The witness · `sec:assayer:intent-three-decay-clocks-run-independently-witness`

The integration test belongs in a new integration test file, `tests/temporal_decay.rs`, and is named `three_decay_clocks_run_independently`. It drives production construction, registration, assessment, labeling, and time injection through `World`; a test-support probe returns immutable numeric copies and performs no state mutation.

- Setup: build a default-configured world with deterministic seed, double-precision pending storage, one outcome axis, one identity dimension, one stable reported Sentinel, and one named entity whose repeated request resolves to a stable coordinate.

- Structural preparation: settle the cold ramp with that request, cross an identity maintenance barrier that drains its observation queue and publishes the resulting competitive set, then cross the model-owner barrier so the cell lifecycle and feature layout are visible together.

- State seeding: assess the same entity sixteen times at one virtual instant, attach an eligible adverse label to every assessment, flush the label path, and require non-zero identity adverse rate, Ledger bad rate, and raw model variance before normalizing any curve.

- Baseline capture: bind an entity decay probe to the active identity cell, routed Ledger entry, published model snapshot, and standardised model direction selected by one baseline assessment. The probe records one common persistent timestamp and preserves the model direction so later identity and Ledger feature movement cannot alter the uncertainty denominator.

- Stimulus: advance the existing virtual clock cumulatively to fourteen, twenty-nine, and two hundred and ninety days, with no intervening label, report, checkpoint, or assessment, and read all three quantities from the same bound probe at each milestone.

- Observation: read the identity and Ledger EWMAs through their production pure-decay methods; read raw and time-corrected sister-model variance along the frozen direction from the same published snapshot; derive $R_I$, $R_L$, and $R_M$ only by division against the captured non-zero baselines.

- Curve assertion: compare all nine milestone readings with their component's exact hourly power, apply the three half-life bands at their named horizons, assert the strict common-checkpoint ordering, assert the model mean and raw covariance snapshot remain bit-identical, and assert that none of the pure reads changes a stored EWMA or timestamp.

- Rate-isolation control: repeat the fixture in one matched world configured with $\gamma_{t,I}=0.91$, $\gamma_{t,L}=0.92$, and $\gamma_{t,M}=0.93$, advance one hour, and require the three normalized readings to equal those three values in that order. This control distinguishes separate field wiring from hard-coded defaults without introducing another source of time.

The fails-before is direct. Routing all components through the Ledger rate collapses at least two curves, swapping identity and Ledger rates reverses their exact expectations, omitting model read-time correction leaves $R_M=1$, charging a pure read mutates the baseline, and aliasing configuration fields fails the one-hour control even if the default half-life bands remain ordered.

## What is missing · `sec:assayer:intent-three-decay-clocks-run-independently-missing`

**Entry (An identity-maintenance publication barrier)** · `entry:assayer:intent-three-decay-clocks-run-independently-identity-barrier`

The existing `World` implementation gains a method that sends `MaintenanceCommand::PrepareCheckpoint`, waits with the shared acknowledgement deadline, then calls `flush_labels` so drained identity observations, the published competitive set, and resulting lifecycle changes form one deterministic preparation boundary. The estimate is approximately 35 lines, with no dependency on another lane.

**Entry (An entity-bound decay probe)** · `entry:assayer:intent-three-decay-clocks-run-independently-decay-probe`

A new test-support file, `src/testing/decay.rs`, gains an integration-test-only probe that captures one assessment's entity, active cell, Ledger route, snapshot version, standardised model direction, and baseline timestamp, then returns the identity adverse rate, Ledger bad rate, raw sister variance, corrected sister variance, and model mean as numeric copies. The estimate is approximately 110 lines including exports, validation, and a unit check that repeated reads do not mutate state.

The probe is a narrow decay-specific slice of the future instrumentation in (´entry:assayer:harness-stage-tapes´), but it depends only on existing `World`, name resolution, pending context, published snapshot, and component read methods, so it does not wait for that broader entry or another lane. The identity barrier entry precedes probe capture.

**Entry (The two-rate-set scenario fixture)** · `entry:assayer:intent-three-decay-clocks-run-independently-scenario-fixture`

The new integration test file, `tests/temporal_decay.rs`, gains a local builder that creates the default and artificial-rate worlds, performs the common structural preparation and sixteen-label seed, and returns a bound probe only after its three baselines are non-zero. The estimate is approximately 55 lines and depends on both entries above.

No new virtual clock, drift budget, wall-time sleep, general tape abstraction, production accessor, or `World::advance` wrapper is required.

## Risks and open questions · `sec:assayer:intent-three-decay-clocks-run-independently-risks`

**Observation (Model uncertainty needs a fixed direction)** · `obs:assayer:intent-three-decay-clocks-run-independently-model-direction`

Public `sigma_eff` alone is not an oracle for the 290-day curve because the request's identity and Ledger features decay at the same time, changing the quadratic-form direction. Comparing raw and corrected sister variance along the probe's frozen baseline direction isolates only the Core covariance multiplier while retaining the actual learned snapshot.

**Observation (Identity names outcome memory, not graph importance)** · `obs:assayer:intent-three-decay-clocks-run-independently-identity-quantity`

The identity layer also decays spatial importance at a host-chosen rate, but that quantity controls contour evolution and is not one of the three fixed Core horizons. The inventory expressly assigns the fourteen-day rate to competitive-cell outcome averages, so the adverse-rate EWMA is the measurable reading and no maintainer choice is needed.

**Observation (The smallest tail needs relative normalization)** · `obs:assayer:intent-three-decay-clocks-run-independently-relative-tail`

After 290 days the identity retention is below one millionth, so the harness's EWMA absolute tolerance could make zero indistinguishable from the intended tail. The test stores non-zero double-precision baselines and compares normalized ratios with the tighter generic tolerance; the broad half-life bands are applied only near each component's own horizon.

**Observation (Two asynchronous owners require two barriers)** · `obs:assayer:intent-three-decay-clocks-run-independently-two-barriers`

`flush_observations` settles standardisation and `flush_labels` settles model-owner work, but neither drains the identity maintenance queue. The dedicated identity barrier is used only before baseline capture, and no assessment follows the capture, so background observations cannot split, replace, or reroute the cell while virtual time advances.

**Observation (The read-only probe is the smallest honest surface)** · `obs:assayer:intent-three-decay-clocks-run-independently-probe-scope`

The public assessment exposes corrected effective uncertainty but neither the matching raw quadratic form nor the component EWMAs, and inferring them from a nonlinear score would admit compensating errors. Immutable test-only copies preserve encapsulation and let every assertion name the production quantity it judges.

No maintainer decision remains open: the specification selects the three quantities, their rates, their lazy read semantics, and the model's reciprocal variance interpretation.

## Acceptance · `sec:assayer:intent-three-decay-clocks-run-independently-acceptance`

The implementation report identifies the new integration test and its claim citation, the identity barrier and decay-probe APIs with their test-support gating, the two rate sets, the seed count, every elapsed checkpoint, each analytical factor, every half-life band, and the tolerance used.

The report shows the witness fail against deliberate local breaks that route all three quantities through one rate, swap the identity and Ledger rates, omit the model read-time correction, and alias one artificial-rate configuration field, with every break removed before the final gates.

The report shows formatting, clippy, the focused integration test in development and release profiles, a `--no-default-features` spot check where test-support gating permits it, documentation tests, and the package test suite green, with command output and exit status for every gate.

The report also shows that the generated test index and coverage report resolve the new test to (´claim:ledger:three-decay-clocks-run-independently´), that every probe read leaves stored timestamps and values unchanged, and that the promise no longer appears in the uncovered-intent set.
