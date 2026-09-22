# Keeping a Ledger Label Whole Across Its Depth Walk · `plan:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all`

Keeping (´claim:ledger:a-concurrently-updated-average-reads-whole-or-not-at-all´) establishes that assessment can observe a concurrently landing label only as a complete pre-label or post-label Ledger state, never as a scalar or ancestry combination no completed label produced.

## What the promise says, precisely · `sec:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-precision`

The scalar formulation starts one cell's bad-rate average at `0.0`, applies one adverse label with the test retention factor `lambda_L = 0.5`, and obtains `0.5` from the specified update `new = lambda_L * old + (1 - lambda_L) * observation` (´data:ledger:attenuation´) (´alg:ledger:all-layers-update´). Zero and one half are exactly representable, so the oracle compares their bit patterns with no numerical tolerance.

The literal endurance quantity is strictly more than one hundred thousand racing reads, each of which must return either the pre-label value or the post-label value. A schedule-sampled loop can miss the critical interleaving on every iteration, so it is supporting load evidence rather than the decisive oracle.

The Ledger stores the bad-rate average in every entry (´tab:ledger:entry-state´). The standing plan correctly lifts the promise one level above a scalar (´obs:assayer:reframe-ledger-atomicity´): one label visits every containing entry from the deepest present cell through the permanent root (´dec:memory:depth-walk´), so the decisive observation is leaf and root both at `0.0` or both at `0.5`; leaf `0.5` beside root `0.0` is the forbidden intermediate.

Assessment reads a pure time-decayed view while stored entries move only on label processing (´inv:publication:ledger-concurrency´). Holding the read timestamp equal to the write timestamp makes the elapsed factor unity under (´def:ledger:time-decay´), isolating concurrency from decay.

The deterministic threshold is universal exclusion at the forced midpoint: after the deepest entry is updated and before the root is updated, acquisition of the per-Sentinel read guard must fail; after the writer completes and releases its guard, the waiting reader must obtain the exact post-label pair. Because the same write guard encloses every depth, that exclusion covers every racing read and is stronger than a finite schedule sample.

## What the code offers today · `sec:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-current-code`

`OutcomeLedger` stores each `SentinelLedger` behind one shared read-write lock. The label path takes that Sentinel's write guard once and passes the guarded ledger to `update_all_layers`; the all-layers update performs the complete deepest-to-root mutation before the guard leaves scope (´alg:ledger:all-layers-update´).

The assessment path takes the same Sentinel's read guard before extraction and retains it while `read_route` selects the deepest entry, computes its decayed view, and copies Ledger values into the feature row. Existing extraction coverage proves that routing reaches the deepest tracked cell (´test:unit:ledger-read-takes-the-deepest-covering-entry´), while focused update coverage proves that the write reaches every containing entry (´test:unit:update-all-layers-hits-all´); both are sequential mechanism witnesses.

The crate-level `tests::ledger::label_assess_race_no_torn_reads` already constructs a real `OutcomeLedger`, obtains its reference post-label state through `update_all_layers`, cuts the same update open beneath the per-Sentinel guard, and checks exact pre-label, mixed, and post-label states (´test:crate:label-assess-race-no-torn-reads´). It mints the synonymous statement (´claim:ledger:a-reader-racing-a-writer-sees-one-whole-value-never-a-partial-one´) instead of citing the canonical intent, so the coverage census still reports this promise as unkept.

The existing handshake does not run in the order its documentation claims: the reader's `try_read` fails while both entries are still pre-label, and only afterwards does the writer create and inspect the mixed state. Holding the same write guard across both events makes the mutual-exclusion argument sound, but it does not directly demonstrate failed acquisition while the mixed state exists; the witness below moves the reader attempt after the leaf update.

The finished shared harness now exposes one `Scenario` over `World`; the public flow through `World::register_sentinel`, `World::receive_report`, `World::request_with_sentinel`, `World::assess`, `World::label`, and `World::flush_labels`; forward-only `World::advance` and `World::travel_to`; the queue barriers `World::flush_observations`, `World::flush_labels`, and `World::flush_identity_maintenance`; and `golden_report`. Its owned probes are `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView`; long streams use `PlaybackRow`, `PlaybackBarrierPolicy`, `PlaybackCheckpoint`, `PlaybackProgress`, and `playback`; the oracle tier supplies `OracleProvenance`, `pairwise_rank`, `regularised_schur_complement`, `decay_recurrence`, and `dimension_width`; guarded training uses `TrainedStateFixture` and `TrainedStateBaseline`; reproducible property cases use `run_seeded_sweep` (´tab:assayer:harness-implementation-library-roster´).

None of those probes exposes raw Ledger entries or the per-Sentinel lock, and `RiskAssessment` exposes derived risk, outcome predictions, alarm summaries, and health rather than the routed Ledger slot or its ancestry. The public verbs and label barrier can observe only completed processing; playback, the analytical oracles, the trained-state fixture, and seeded sweeps do not create a controllable midpoint. The direct crate witness is therefore the correct level, with `ACK_DEADLINE` from the liveness module as its only shared-harness dependency for the test-thread acknowledgement (´dec:harness:no-ad-hoc-waits´).

## The witness · `sec:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-witness`

The witness remains `tests::ledger::label_assess_race_no_torn_reads` on the `torrust_assayer` library test target (´test:crate:label-assess-race-no-torn-reads´); moving it to a public integration target would discard the only direct oracle for the protected intermediate without producing a truer stimulus.

- Setup: create one `OutcomeLedger` Sentinel with a root and a depth-four leaf containing the same coordinate, stamp both at one fixed `PersistentTimestamp`, and define one eligible adverse `LedgerUpdate` with `lambda_L = 0.5` and the configured hourly factor.

- Reference: run production `update_all_layers` once beneath the Sentinel write guard and require exactly two touched entries, each with bad-rate bits for `0.5`.

- Pre-label observation: acquire the read guard and require the routed leaf and root both to carry bad-rate bits for `0.0`, preventing a test that observes only the final state from passing vacuously.

- Forced midpoint: acquire the write guard and apply `LedgerEntry::apply_write_decay_and_update` to the leaf exactly as the first step of the production walk does, leaving the root unchanged; require leaf `0.5` beside root `0.0` while the guard remains held.

- Exclusion: start the reader only after the mixed state exists, require its non-blocking `try_read` to fail, and acknowledge that failed attempt to the writer through the existing channel. Bound only that acknowledgement with `ACK_DEADLINE`; it is a liveness failure detector, not elapsed-time evidence (´dec:harness:no-ad-hoc-waits´).

- Completion: after receiving the acknowledgement, apply the identical update to the root, release the write guard, join the reader, and require its one guarded observation and the final ledger both to equal the production reference exactly.

- Coverage binding: replace the test documentation's synonymous bare claim mint with a citation of the canonical intent, retaining the test label and updating the module-index sentence to describe the corrected ordering.

The fails-before is a deliberately weakened Ledger that locks entries separately or releases the Sentinel write guard between the leaf and root updates. The reader then acquires during the forced midpoint and returns leaf `0.5` beside root `0.0`; a write that omits the root instead fails the final equality with the production reference.

## What is missing · `sec:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-missing`

No new harness primitive, production hook, probe, playback row, oracle, fixture, clock verb, barrier, or sweep is missing. The remaining work is a local correction and coverage binding in the existing crate witness.

**Entry (Bind the forced-interleaving witness to the intent)** · `entry:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-canonical-binding`

In `tests::ledger::label_assess_race_no_torn_reads`, move the leaf update before the reader starts, require `try_read` to fail while the mixed state is held, replace the local `HANDSHAKE` duration with `ACK_DEADLINE`, replace the synonymous claim mint with the canonical intent citation, and update the module-index sentence. This is a small edit to the existing test module, depends on no other entry or lane, and introduces no public or test-support API.

## Risks and open questions · `sec:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-risks`

**Observation (Mutual exclusion is stronger than a race count)** · `obs:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-exclusion-over-sampling`

A loop of more than one hundred thousand reads measures only the schedules that happened to run and can remain green after losing the critical interleaving. The forced midpoint demonstrates that the forbidden state exists during mutation and that the reader cannot enter it, making the finite count unnecessary as an acceptance threshold rather than silently weakening it.

**Observation (The public result has no Ledger-state oracle)** · `obs:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-public-oracle`

Changes in public risk after a label include model publication and may include newer standardisation moments, while `RiskAssessment` omits raw feature slots. The finished `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView` probes preserve their single-load owned-copy contract but expose no Ledger state (´dec:harness:probe-contract´); adding another probe solely to restate a property already enforced at the lock boundary would widen the harness without making the concurrency claim more direct.

**Observation (The standing lock citation names another lock pair)** · `obs:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-lock-citation-scope`

The reframing observation cites (´dec:ordering:lock-order´), whose record text orders report reception before its accumulator for deadlock freedom; it does not state per-Sentinel Ledger read-write exclusion. The witness rests instead on the Ledger concurrency invariant, the shared-lock shape in code, and failed acquisition at the mixed state; correcting the standing testing plan's citation is outside this intent lane and does not block the test binding.

**Observation (Exact arithmetic removes numerical ambiguity)** · `obs:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-exact-arithmetic`

Zero and one half are exact, and the shared timestamp makes decay a multiplication by one. A tolerance would admit values no completed cell held and would weaken the promise; bitwise comparison is the required numerical posture.

**Observation (Atomicity and latency remain separate)** · `obs:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-atomicity-not-latency`

The concurrency tiers give the per-Sentinel Ledger a bounded wait budget (´tab:publication:tiers´), but a test-controlled pause inside a held guard intentionally exceeds ordinary critical-section duration. `ACK_DEADLINE` detects a dead test-thread handshake and supplies no performance threshold. This witness establishes state atomicity only; it neither measures nor relaxes the separately specified latency budget.

No maintainer decision blocks the binding. The standing testing plan should describe this forced-midpoint witness and replace its unrelated lock-order citation, but that neighboring correction does not belong to this one-file lane.

## Acceptance · `sec:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-acceptance`

The implementing lane names the Cargo library test target `torrust_assayer` and the Rust function `tests::ledger::label_assess_race_no_torn_reads`, shows its documentation citing the canonical intent, confirms that the synonymous claim is no longer minted, and shows the coverage report resolving the intent to (´test:crate:label-assess-race-no-torn-reads´).

The reported assertions include the exact pre-label pair, the deliberately constructed mixed midpoint under the writer guard, failed reader acquisition while that midpoint exists, the exact post-label pair after release, and equality between the cut-open write and `update_all_layers`.

The report shows the targeted library test and the Assayer package's prescribed formatting, lint, feature, documentation, documentation-test, and broader test gates passing, with unrelated failures separated explicitly.

The report states which assertion rejects per-entry locking, a guard released between depths, a skipped ancestor, or a tolerance-admitted scalar; no executed mutation is required.

Acceptance adds no probabilistic stress loop, sleep, wall-clock performance assertion, public accessor, production hook, or widened tolerance. The proof is the forced interleaving under the same lock and routing functions used by assessment extraction and label processing.
