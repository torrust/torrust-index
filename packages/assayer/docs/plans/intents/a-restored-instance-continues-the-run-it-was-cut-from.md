# Keeping a Restored Instance on Its Interrupted Run · `plan:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from`

Keeping (´claim:persistence:a-restored-instance-continues-the-run-it-was-cut-from´) establishes that a checkpoint plus its journal is a continuation boundary: every durable Core quantity and every host-visible reading after the boundary agrees with an otherwise identical run that crossed no process restart, while incompatible coordinates select an observable cold start and numerical retuning preserves the saved state.

## What the promise says, precisely · `sec:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from-precision`

Let $P$ be a deterministic prefix ending at checkpoint sequence $h$, let $J$ be the ordered labels durably appended after $h$ and before the crash, and let $S$ be a common suffix. The primary equality is between the uninterrupted execution $P \circ J \circ S$ and `restore(checkpoint(P), J) \circ S`; the replay side admits exactly the entries with sequence greater than $h$, in journal order, and applies each once, as required by (´dec:durability:checkpoint-journal´) and (´cor:durability:replay-exactness´).

“Model state” comprises every operational, sister, anchor and outcome-axis posterior; their means, precision matrices, maintained covariances, calibration scalars, class-rate estimates, recomputation state, drift state, marginalisation accounting, hibernation archive, feature moments, dimension map, and cold-ramp sample and position. A partial ramp resumes at its accepted count rather than returning to the priors or jumping to service (´dec:durability:ramp-resumption´), whose measurable phase and count follow the batch-initialisation algorithm (´alg:standardisation:batch-initialisation´).

“Published risk basis” means every stable field of the public assessment payload: `RiskBasis`, outcome predictions, per-Sentinel alarms, and embedded health, excluding only the newly allocated assessment identifier (´schema:output:assessment´). A fixed probe set names the same channels, entities, reports, Sentinel IDs, identity coordinates and outcome axes in both branches.

“Ledger views” means identical Sentinel membership, dyadic keys, traffic and eligibility counts, per-axis membership, and timestamps, with the bad-rate and raw and compressed valence EWMAs compared numerically. “Identity graph” means identical registered dimensions, graph terminal and transition counts, competitive-cell membership, cell outcome counts, per-axis membership, and corresponding EWMAs; the registration record fixes the structure whose identity is retained (´schema:keyspace:dimension-record´).

“Companion trackers” is read here as the checkpoint-carried trackers beside the model: the calibration buffer, Platt tracker, per-dimension convergence trackers, concordance tracker, label counters, health counters and model-health accumulators. The separately branded Companion remains host-owned and outside the Core boundary (´inv:guarantee:companion-independence´).

Discrete membership, ordering, sequence marks, phases, counts, flags and identifiers compare exactly. Generic floating quantities use `Tolerances::default = 1e-9`, EWMAs use `Tolerances::ewma = 1e-6`, and AUC readings use `Tolerances::auc = 5e-3`; precision remains valid below `1e-6 × model width`, while exact serialized scalars may use `bit_identical = 0.0`, all from (´tab:assayer:harness-scenario-tolerances´). The `standardisation = 0.14` budget bounds store-time versus label-time coordinate drift and does not widen equality between the two branches.

Structural compatibility covers the construction-time signal schema in name, shape and order, the ordered interaction templates and the fixed anchor width. Any mismatch takes the cold construction path, while changing a scalar parameter such as `monitoring.h_threshold` leaves the checkpoint admissible and preserves its learned state (´dec:durability:structural-compatibility´) and (´dec:construction:two-starts´).

## What the code offers today · `sec:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from-code`

The public route begins with `World::builder` or `scenario_with_config`, supplies persistence directories through `WorldBuilder::persistence_dirs`, registers Sentinels, outcome axes and identity dimensions, ingests reports, calls `derive_for_request`, submits `LabelSpec` data, and ends asynchronous work through `flush_observations` and `flush_labels`. `VirtualClock` and `World::clock` hold both branches at the same instant, and `World::tol` supplies the named comparison budgets; these verbs and fixtures belong to the completed harness skeleton (´entry:assayer:harness-stage-skeleton´).

The checkpoint payload serializes the four model families, sequence mark and timestamp, Ledger and identity payloads, structural metadata, concordance, marginalisation and hibernation state, drift, standardisation and cold-ramp state, plus the owner-held calibration buffer, trackers and counters (´dec:durability:checkpoint-journal´). The recovery path validates structure, reconstructs the working copy, applies component-specific elapsed decay, filters the journal above the high-water mark, and returns the recovered side state (´dec:durability:decay-once´) and (´dec:durability:structural-compatibility´).

Restored construction attaches the recovered Ledger, concordance and owner state, opens the journal after the restored sequence, and sends replay entries through the ordinary owner pipeline (´dec:durability:checkpoint-journal´). It does not yet attach the recovered identity payload; it also constructs empty runtime Sentinel, identity and outcome-axis registration maps and resets the assessment counter, blend tracker, several degradation and label-integrity counters, health-event counts and publication numbering. Those fresh values make the full promise observably false even where the model matrices restore.

The shared assertions already compare risk bases, outcome maps, per-Sentinel alarms and embedded health under the completed harness skeleton (´entry:assayer:harness-stage-skeleton´), but they have no comparator for a complete durable snapshot, full health report, Ledger contents or identity graph. The world harness has no persistence-image fork, restart constructor that replays runtime declarations, or barrier proving that the identity maintenance owner has drained its work, all within the fixture work registered by (´entry:assayer:harness-stage-tapes´).

The crate tests prove a payload round trip and selected production restoration in (´test:crate:checkpoint-complete-round-trip´), (´test:crate:checkpoint-whole-state-production-path´), and (´test:crate:checkpoint-round-trip-end-to-end´). The nearest public restart witness proves that a label journalled across owner shutdown is replayed and cleared (´test:integration:label-reports-model-owner-shutdown-journal-replays-on-restart´), but none compares an uninterrupted suffix with a restored suffix or pairs structural refusal with numerical admission.

The standing gap is (´entry:assayer:gap-recovery-residuals´), which calls for round-trip proof of Ledger, identity, calibration and health payloads and sequence-aware recovery. The broader fixture work is already registered under (´entry:assayer:harness-stage-tapes´); the persistent clock remainder is (´entry:assayer:harness-stage-clock´).

## The witness · `sec:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from-witness`

The witness is one new integration scenario with three rebuild arms derived from one stable persistence image. Its documentation takes custody of (´claim:persistence:a-restored-instance-continues-the-run-it-was-cut-from´) from the intent catalogue, and its own integration-test label names the concrete scenario.

- Setup: build a seeded persisted `World` with one scalar signal, one Sentinel report, one spatial outcome axis and one identity dimension; drive a deterministic prefix containing both outcome classes, enough identity coordinates to establish graph and competitive-cell state, non-zero Ledger cells, calibration progress, drift residue and a deliberately partial cold ramp.
- Cut preparation: drain observation, lifecycle, identity-maintenance and label work, force a checkpoint, verify that the journal is at its header, and copy the quiescent checkpoint-plus-journal image into an isolated crash-staging directory.
- Control branch: retain the original process as the never-crashed continuation, derive a fixed batch, drain its cold observations, and apply its labels normally in a declared order before driving the common suffix.
- Crash branch: build from the staging image, first assert a baseline state match to the retained control at the cut, derive the same fixed batch, drain its cold observations, stop only the model owner after the pending contexts exist, submit the labels so their self-contained journal records succeed while enqueueing fails, then drop the process and fork the closed crash image into ordinary, numerical-change and structural-change directories.
- Restore stimulus: rebuild the ordinary arm with the original configuration, reattach the same runtime declarations without allocating new IDs or mutating the restored layout, drain replay through a label barrier, and drive the same reports, assessments, labels, clock readings and suffix order as the control.
- Observation: capture a normalized durable-state projection immediately after replay and after the common suffix, then obtain public assessments over the fixed probes and both public health tiers, together with hidden Ledger and identity projections, from both branches at quiescent barriers.
- Continuation assertion: compare all discrete state exactly and every floating domain under its named budget; require the checkpoint high-water mark plus journal tail to equal the resulting processed sequence, require the partial ramp count and phase to continue, and require the post-suffix public risk and health surfaces to match the control.
- Numerical arm: rebuild a copy of the crash image with only `monitoring.h_threshold` changed from the prefix's non-triggering value to half a positive carried drift accumulator, prove from the unique prefix state, sequence mark, Ledger cells, identity cells, calibration rows and ramp count that it restored rather than cold-started, then require the next eligible label to take the new threshold's reset path.
- Structural arm: rebuild another copy with the saved scalar signal changed to a cyclic shape, require a successful cold start with zero learned counters, an initial cold-ramp position, no recovered runtime registrations or replayed journal labels, and prior-state probe output distinct from the learned control.
- Fails-before: omitting replay loses $J$; replaying at or below $h$ double-applies evidence; dropping one payload family changes its projection or public health; resetting publication or assessment counters changes host-visible metrics; failing to rebind runtime identities removes reports or graphs; accepting the cyclic schema restores coefficients onto different coordinates; and treating `h_threshold` as structural erases the unique learned markers in the numerical arm.

The witness does not mutate an implementation to obtain fails-before evidence. It uses a fixed seed, fixed tape, fixed virtual instant and explicit barriers, so a difference reports a recovery boundary rather than elapsed wall time or queue progress; `ACK_DEADLINE` bounds a stalled acknowledgement only and is never a success threshold.

## What is missing · `sec:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from-missing`

**Entry (A persistence image can be forked and restarted deterministically)** · `entry:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from-persistence-fork`

Approximately 90–130 lines in shared testing support add a fixture that forces a checkpoint, holds the owner and journal quiescent while copying their files, rebuilds from an isolated image with the same `VirtualClock`, and retains a declarative registration tape. It depends on (´entry:assayer:harness-stage-tapes´), uses the already injected persistent clock without advancing it, and has no sibling-lane dependency.

**Entry (Durable state has one normalized projection and comparator)** · `entry:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from-state-projection`

Approximately 140–220 lines in shared testing support expose a hidden test-only snapshot of model parameters and maintenance counters, Ledger state, identity graph and cell state, owner trackers, standardisation and sequencing, plus assertions that select exact, default, EWMA, AUC and precision budgets by field. It depends on the persistence-fork entry and the existing tolerance bundle, and extends the state-inspection part of (´entry:assayer:harness-stage-tapes´).

**Entry (Restored declarations rebind runtime owners)** · `entry:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from-runtime-rebinding`

Approximately 140–220 lines across construction, lifecycle and test support reconnect restored Sentinel, axis and identity declarations to their runtime maps, encode closures and dedicated owners without adding duplicate feature positions; the restored identity payload seeds the graph, competitive set and cell outcomes when its host-supplied closure is rebound. It depends on the registration tape and state projection, and it is a production prerequisite exposed by this witness rather than a test-only convenience.

**Entry (Host-visible health continuity has a declared durable boundary)** · `entry:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from-health-boundary`

Approximately 100–180 lines extend the checkpoint and reconstruction path for publication version, total assessments, blend statistics, assessment degradation, label-integrity and event counters that the promise includes, with quiescent buffer and queue readings compared as instantaneous state. It depends on the state projection and requires no other intent lane; any deliberately process-local metric is instead named in the promise or output contract before the comparator excludes it.

**Entry (Identity maintenance has a completion barrier)** · `entry:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from-identity-barrier`

Approximately 50–90 lines add a test-support command that acknowledges after deferred graph writes and competitive-set publications preceding it are visible, with `ACK_DEADLINE` reporting a stalled owner. It depends on the runtime-rebinding entry, replaces polling or sleeps in the prefix and suffix, and completes only the identity-specific barrier needed by the persistence fixture.

## Risks and open questions · `sec:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from-risks`

**Observation (The branded Companion remains a host-side alternative)** · `obs:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from-companion-boundary`

The working interpretation covers Core-adjacent trackers because the Companion is structurally independent and `World` currently creates its challenge tracker afresh. If “companion trackers” intends that branded state, the fixture saves and restores `ChallengeEffectivenessTracker` beside the Assayer files and compares derived landscapes as a host-composed continuation; extending the Core checkpoint would contradict (´inv:guarantee:companion-independence´).

**Observation (Publication identity and cumulative health are included)** · `obs:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from-observable-boundary`

Snapshot version, total assessments and cumulative health counters are host-visible metrics, so the plan compares them exactly even though the present checkpoint omits them; assessment IDs are excluded because assessments in flight are deliberately not durable and the promise names the published basis rather than request identity. An alternative that makes more fields process-local requires an explicit narrowing of the health-metric phrase before implementation.

**Observation (Elapsed downtime belongs to the adjacent decay witness)** · `obs:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from-decay-boundary`

Holding virtual time fixed isolates state continuation from the once-only discounting promise (´claim:persistence:restore-time-decay-is-applied-exactly-once´). A non-zero downtime arm becomes appropriate after (´entry:assayer:harness-stage-clock´) completes, but it is not necessary for this witness to distinguish a complete restore from a partial or cold one.

**Observation (A warm start needs positive evidence, not build success)** · `obs:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from-warm-start-oracle`

Both a permitted numerical change and a rejected structural change return a successfully built instance under the two-start contract. The numerical arm therefore requires several unique learned markers to survive, while the structural arm requires those same markers to be absent; construction success alone proves neither disposition.

**Observation (Quiescence is part of the comparison state)** · `obs:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from-quiescence`

Model ownership, identity maintenance, cold observations and journal replay use different queues. The comparison follows acknowledgements from each owner and observes empty transient buffers on both branches; the liveness deadline reports a missing acknowledgement, while no sleep, retry tolerance or insertion-order accident stands in for a barrier.

## Acceptance · `sec:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from-acceptance`

The implementation-lane report identifies the integration scenario and its intent citation, names the exact prefix, checkpoint high-water mark, journal-tail length, suffix, probe set, scalar numerical change and scalar-to-cyclic structural change, and states which persistence image feeds each arm.

The report presents the before-cut baseline, after-replay and after-suffix durable-state diffs; the public assessment, Ledger, identity and health comparisons; the partial-ramp position; and the warm-versus-cold markers, with every exact assertion and every named tolerance shown.

The report shows that the ordinary and numerical arms restored, the structural arm cold-started without replaying incompatible journal state, and each fails-before defect reaches a specific assertion without an executed mutation.

The report resolves the Companion interpretation and the durable health boundary, records every production or harness prerequisite implemented from the entries above, and names any residual excluded as process-local with its governing specification citation.

Package formatting, linting, the focused integration binary, all Assayer targets and features, the no-default-features spot check, documentation tests and release-mode package tests are green, with commands, exit codes, wall times and corpus-label resolution reported.
