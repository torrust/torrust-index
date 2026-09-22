# Plan for Three Independent Decay Clocks · `plan:assayer:intent-three-decay-clocks-run-independently`

This plan keeps (´claim:ledger:three-decay-clocks-run-independently´) by making one entity expose the separate identity, Ledger, and model-retention curves under deterministic virtual time, establishing that none of the three configured rates governs either of the other two quantities.

## What the promise says, precisely · `sec:assayer:intent-three-decay-clocks-run-independently-precision`

The shipped hourly retentions are $\gamma_{t,I}=0.998$ for identity cell outcome averages, $\gamma_{t,L}=0.999$ for Ledger averages, and $\gamma_{t,M}=0.9999$ for all Core model parameters; these are three separately host-configurable fields (´tab:config:temporal´) in the complete decay inventory (´tab:temporal:decay-inventory´).

For an elapsed interval of $h$ hours with no new label, the identity retention is $R_I(h)=I_h/I_0=0.998^h$, the Ledger retention is $R_L(h)=L_h/L_0=0.999^h$, and model precision retention is $R_M(h)=V_{raw}/V_{corrected}=0.9999^h$. Equivalently, time-corrected model variance is inflated by $1/R_M(h)$ while the posterior mean is unchanged (´alg:temporal:lazy-application´) (´def:runtime:time-correction´).

The observable identity scalar is one seeded competitive cell's adverse-rate EWMA, a member of the outcome state defined by (´tab:keyspace:outcome-state´), rather than the graph's separately configured spatial importance. The observable Ledger scalar is the bad-rate EWMA of the entry selected by the same request's Sentinel route, whose pure read-time attenuation is fixed by (´def:ledger:time-decay´).

At fourteen days $R_I$ lies in $[0.50,0.52]$; at twenty-nine days $R_L$ lies in $[0.49,0.51]$; and at two hundred and ninety days $R_M$ lies in $[0.49,0.51]$, equivalently placing model variance inflation in $[1.96,2.04]$. At every positive common checkpoint the strict ordering is $R_I<R_L<R_M<1$.

The primary assertions use `decay_recurrence` with a unit baseline, no label steps, the configured hourly rate, and the elapsed hours, then compare each normalized reading under the `Tolerances` default bound (´tab:assayer:harness-scenario-tolerances´). This supplies a specification-formula route independent of the production power calculation (´dec:harness:oracle-tier´) (´entry:assayer:harness-oracle-tier´); the bands above state the promised approximate horizons and do not replace that analytical oracle.

The longest interval remains below the timestamp interface's one-year clamp (´dec:clock:embedded-clamp´). No labels arrive after the baseline, so label-indexed forgetting is absent and elapsed time is the only stimulus under the independent-mechanism definition (´def:temporal:two-mechanisms´). Model decay remains lazy at label time and is repaired at assessment time by the read-time correction, consistent with the head-of-path ordering (´dec:ordering:decay-at-head´).

## What the code offers today · `sec:assayer:intent-three-decay-clocks-run-independently-current-surface`

The unified `World` and `WorldBuilder` construct the real `Assayer`; `scenario_with_config` supplies an explicit temporal configuration and deterministic seed; `LabelSpec` builds repeatable labels; and `World::flush_observations`, `World::flush_labels`, and `World::flush_identity_maintenance` are the queue-specific barriers this setup needs. These are current members of the finished harness roster (´tab:assayer:harness-implementation-library-roster´).

`World::advance` and `World::travel_to` now move both timestamp domains forward and cross identity-maintenance, lifecycle-publication, and Ledger-collection work made due by the movement. Tests no longer move time through `World::clock` or `VirtualClock::advance`; the scenario verbs are the completed harness surface under the two-domain boundary and shared decay funnel (´entry:assayer:harness-scenario-time´) (´cor:clock:harness-control´) (´dec:clock:two-domains´) (´dec:clock:shared-functions´).

**Entry (The identity-maintenance publication barrier is complete)** · `entry:assayer:intent-three-decay-clocks-run-independently-identity-barrier`

`World::flush_identity_maintenance` sends the checkpoint command, waits inside the harness's liveness boundary, drains identity observations and deferred cache writes, publishes accepted lifecycle changes through the model owner, and returns only after that publication. The closed barrier entry records this exact composition, so the plan no longer proposes `MaintenanceCommand::PrepareCheckpoint`, `ACK_DEADLINE`, or a compound wait in the test (´entry:assayer:harness-closed-barriers´) (´cor:concurrency:harness-barriers´).

The landed probe tier supplies `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView`; their `World` entry points return owned readings under the single-load, barrier-aware probe contract (´entry:assayer:harness-probe-contract´) (´dec:harness:probe-contract´). `PublishedModelBlock` exposes a published mean and covariance, but none of the three existing projections exposes the routed Ledger EWMA, the active identity cell's outcome EWMA, the frozen standardised direction, or the production-corrected sister variance required to place all three curves in one reading.

The harness also supplies `PlaybackRow`, `PlaybackBarrierPolicy`, `PlaybackProgress`, and `playback` for ordered stimuli; `decay_recurrence` for the independent temporal oracle; `TrainedStateFixture` and `TrainedStateBaseline` for callers that require a trained precondition; and `run_seeded_sweep` for invariant packs over several seeds (´entry:assayer:harness-tape-runner´) (´dec:harness:declarative-playback´) (´dec:harness:guarded-fixtures´) (´dec:harness:seeded-sweeps´). This witness needs declarative playback and the decay oracle, but it needs neither a converged model nor a seed sweep: its setup guard requires only a stable route and three non-zero measured baselines.

Identity state already implements the pure `CellOutcomeState::read_decayed` view (´claim:identity:a-decayed-outcome-view-is-exactly-the-mutating-decay-result-without-changing-storage´), and Ledger state exposes the corresponding `LedgerEntry::read_decayed` view under (´def:ledger:time-decay´); the old plan's `LedgerEntry::read_at` name no longer exists. The assessment pipeline computes its Core snapshot-age correction from `gamma_t_core` under (´def:runtime:time-correction´), and configuration projection carries all three host-set temporal fields unchanged (´claim:labelling:every-pipeline-setting-is-carried-across-from-the-host-configuration´).

The promise remains unkept. Arithmetic integration tests establish the twenty-nine-day, two-hundred-and-ninety-day, and identity half-life calculations separately (´test:integration:decay-factor-29-day-half-life´) (´test:integration:decay-factor-290-day-half-life´) (´test:integration:decay-factor-identity-half-life´). The nearest stateful witnesses establish a pure identity decayed view (´test:unit:cell-outcome-state-decayed-view-is-exact-and-pure´), a pure Ledger timeline created by rewinding one entry (´test:crate:ledger-decay-twenty-nine-day-half-life-timeline´), and variance movement from an already selected correction factor (´test:unit:time-correction-affects-variance´); none drives one entity through all three configured rates or detects two configuration fields wired to the same consumer.

## The witness · `sec:assayer:intent-three-decay-clocks-run-independently-witness`

The integration witness belongs to cargo target `temporal_decay` under the function path `temporal_decay::three_decay_clocks_run_independently`. It drives production construction, registration, assessment, labeling, and time through `World`; all internal readings are owned numeric copies governed by the probe contract.

- Setup: build a `scenario_with_config` world with a deterministic seed, reference temporal rates, double-precision pending storage, one outcome axis, one identity dimension, one stable reported Sentinel, and one named entity whose repeated request resolves to a stable coordinate. Set only the identity graph's spatial-importance retention to unity so that `World::advance` cannot evict the cell while the separate identity outcome retention remains $0.998$.

- Structural preparation: settle the cold standardisation ramp and cross `World::flush_observations`; use `CompetitiveCellSpec` with `World::register_identity_with_cells` to establish and verify the exact active cell after the method's identity-maintenance and lifecycle-publication boundary.

- State seeding: supply sixteen identical adverse assessment-label rows through a local `PlaybackRow`; run `playback` with a `PlaybackBarrierPolicy` containing `PlaybackBarrier::FlushLabels`, so every row is published before the next begins and no test-authored loop chooses its own wait. Sixteen is a fixture population chosen to put both EWMAs well above their non-zero guard without claiming convergence.

- Baseline capture: after the named setup barriers, bind the entity, active identity cell, routed Ledger entry, one published model snapshot, and the standardised sister-model direction selected by the same request. Record separate typed time provenance for persistent EWMA decay and monotonic snapshot age, and retain the frozen direction so later identity and Ledger feature movement cannot alter the model uncertainty denominator.

- Stimulus: call `World::advance` by fourteen days, then fifteen days, then two hundred and sixty-one days, reaching cumulative checkpoints of fourteen, twenty-nine, and two hundred and ninety days with no intervening label, report, or assessment. Each call supplies the time-activated identity-maintenance and Ledger-collection boundaries; the reading supplies its own probe boundary.

- Observation: at every checkpoint read the identity and Ledger EWMAs through their production pure-decay methods, and read raw and time-corrected sister-model variance along the frozen direction from the same published snapshot. Derive $R_I$, $R_L$, and $R_M$ only by division against the captured non-zero baselines.

- Curve assertion: compare all nine milestone readings with the corresponding `decay_recurrence` result, apply the three horizon bands only at their named checkpoints, assert the strict ordering at every common checkpoint, require the model mean and raw covariance snapshot to remain bit-identical, and require every pure read to leave its stored EWMA and timestamp unchanged.

- Rate-isolation control: repeat the same guarded fixture with $\gamma_{t,I}=0.91$, $\gamma_{t,L}=0.92$, and $\gamma_{t,M}=0.93$, advance one hour through `World::advance`, and require the three normalized readings to equal those values in that order. This control distinguishes separate field wiring from hard-coded defaults without introducing another time source.

The fails-before is direct. Routing all components through the Ledger rate collapses at least two curves, swapping identity and Ledger rates reverses their exact expectations, omitting model read-time correction leaves $R_M=1$, charging a pure read mutates its stored baseline, and aliasing configuration fields fails the one-hour control even if the reference half-life bands remain ordered.

## What is missing · `sec:assayer:intent-three-decay-clocks-run-independently-missing`

**Entry (An entity-bound decay projection)** · `entry:assayer:intent-three-decay-clocks-run-independently-decay-probe`

Extend the existing private `testing::probes` module and its flattened exports rather than adding a parallel probe module. The addition consists of one owned selector that binds an entity, active identity cell, routed Ledger key, snapshot version, and frozen standardised sister direction, plus one owned reading containing the identity adverse rate, Ledger bad rate, stored values and timestamps needed for purity checks, raw sister variance, corrected sister variance, and sister mean. Its `World` entry points cross the relevant existing barriers, take exactly one published snapshot load per capture or reading, use only scenario time, return no borrow or mutation path, and expose no precision field (´dec:harness:probe-contract´) (´cav:retention:probe-boundary´). The current `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView` do not supply these entity-bound quantities.

**Entry (The two-rate-set scenario fixture)** · `entry:assayer:intent-three-decay-clocks-run-independently-scenario-fixture`

The `temporal_decay` integration target needs a local row type implementing `PlaybackRow` and one setup routine for the reference and artificial temporal configurations. The routine establishes the exact cell, runs the sixteen-row adverse stimulus, captures the owned selector, measures all three baselines, and refuses with those measurements unless the route is stable and every baseline is finite and non-zero. That guard establishes setup while `decay_recurrence` separately judges the result (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

No new clock, barrier, wait, deadline, playback runner, analytical oracle, trained-state fixture, seed sweep, production accessor, or `World::advance` wrapper is missing. The owned entity-bound projection is the only shared-harness addition; the scenario fixture and witness remain local to their integration target.

## Risks and open questions · `sec:assayer:intent-three-decay-clocks-run-independently-risks`

**Observation (Model uncertainty needs a fixed direction)** · `obs:assayer:intent-three-decay-clocks-run-independently-model-direction`

Public `sigma_eff` is not an oracle for the two-hundred-and-ninety-day curve because identity and Ledger features decay at the same time, changing the quadratic-form direction. Comparing raw and corrected sister variance along the projection's frozen baseline direction isolates only the Core covariance multiplier while retaining the actual learned snapshot.

**Observation (Identity names outcome memory, not graph importance)** · `obs:assayer:intent-three-decay-clocks-run-independently-identity-quantity`

The identity layer also decays spatial importance at a host-chosen rate, and `World::advance` now crosses the maintenance work that applies it. The fixture must set only that graph rate to unity so total importance and competitive routing cannot become a second time-varying input to this witness. The promised fourteen-day quantity remains the competitive cell's adverse-rate EWMA because the inventory assigns $0.998$ to outcome averages, not to the independently configured graph contour (´tab:keyspace:decay-rates´).

**Observation (The smallest tail needs relative normalization)** · `obs:assayer:intent-three-decay-clocks-run-independently-relative-tail`

After two hundred and ninety days the identity retention is below one millionth, so the harness's EWMA absolute bound could make zero indistinguishable from the intended tail. The test divides by a measured non-zero double-precision baseline before applying the `Tolerances` default bound; the broad bands are applied only near each component's own horizon.

**Observation (Two asynchronous owners require distinct barriers)** · `obs:assayer:intent-three-decay-clocks-run-independently-two-barriers`

Cold-ramp observations settle through `World::flush_observations`, identity observations and their lifecycle changes settle through `World::flush_identity_maintenance`, and label rows settle through `PlaybackBarrier::FlushLabels`. `World::register_identity_with_cells` and `World::advance` compose the barriers their own operations require, but no assertion infers completion of an excluded queue from another barrier (´cor:concurrency:harness-barriers´).

**Observation (The read-only projection is the smallest honest surface)** · `obs:assayer:intent-three-decay-clocks-run-independently-probe-scope`

The public assessment exposes corrected effective uncertainty but neither the matching raw quadratic form nor the component EWMAs, and inferring them from a nonlinear blended score would admit compensating errors. Extending the existing probe tier with immutable owned copies preserves the finished contract and lets every assertion name the production quantity it judges without widening the host surface (´dec:harness:probe-contract´) (´cor:surface:test-support-opacity´).

No maintainer decision remains open: the specification selects the three quantities, their rates, their lazy read semantics, and the model's reciprocal variance interpretation. The remaining work is the bounded probe extension, the guarded local fixture, and the witness.

## Acceptance · `sec:assayer:intent-three-decay-clocks-run-independently-acceptance`

The implementation report names cargo target `temporal_decay` and function `temporal_decay::three_decay_clocks_run_independently`, its claim citation, the entity-bound projection and its probe-contract checks, the reference and artificial temporal rate sets, the unit graph-importance retention, the deterministic seed, the sixteen-row setup population, the three elapsed checkpoints, each `decay_recurrence` factor, every half-life band, and the named `Tolerances` fields used.

The report shows the witness fail against deliberate local breaks that route all three quantities through one rate, swap the identity and Ledger rates, omit model read-time correction, charge a pure read, and alias one artificial-rate configuration field, with every break removed before the final gates.

The report shows formatting, clippy, the focused `temporal_decay` integration target in development and release profiles, a `--no-default-features` spot check where test-support gating permits it, documentation tests, and the package test suite green, with command output and exit status for every gate.

The report also shows that the generated test index and coverage report resolve `temporal_decay::three_decay_clocks_run_independently` to (´claim:ledger:three-decay-clocks-run-independently´), that every projection read leaves stored timestamps and values unchanged, and that the promise no longer appears in the uncovered-intent set.
