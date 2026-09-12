# Keeping a Ledger Label Whole Across Its Depth Walk · `plan:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all`

Keeping (´claim:ledger:a-concurrently-updated-average-reads-whole-or-not-at-all´) establishes that assessment can observe a concurrently landing label only as a complete pre-label or post-label Ledger state, never as a scalar or ancestry combination no completed label produced.

## What the promise says, precisely · `sec:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-precision`

The scalar formulation starts one cell's bad-rate average at `0.0`, applies one adverse label with a test smoothing factor of `lambda_L = 0.5`, and obtains `0.5` from the specified EWMA update (´alg:ledger:all-layers-update´). The pre-label and post-label values are exactly representable, so the oracle compares their bit patterns with no numerical tolerance.

The literal endurance quantity is strictly more than one hundred thousand racing reads, each of which must return either the pre-label value or the post-label value. A schedule-sampled loop can miss the critical interleaving on every iteration, so it is supporting load evidence rather than the decisive oracle.

The Ledger entry holds the average beside its label count and other outcome state (´tab:ledger:entry-state´). A whole scalar observation therefore pairs `total_assessments = 0` with bad-rate bits for `0.0`, or `total_assessments = 1` with bad-rate bits for `0.5`; either crossed pairing is torn even if the floating-point load itself is indivisible.

The standing plan correctly lifts the promise one level above a scalar (´obs:assayer:reframe-ledger-atomicity´). One label visits every containing entry from the deepest present cell through the permanent root (´dec:memory:depth-walk´), so the full observation is a routed leaf and its root both before the label or both after it; a post-label leaf beside a pre-label root is the intermediate that matters.

Assessment reads a pure time-decayed view while stored entries move only on label processing (´inv:publication:ledger-concurrency´). Holding the read timestamp equal to the write timestamp makes the elapsed factor unity under (´def:ledger:time-decay´), isolating concurrency from decay.

The measurable threshold is universal exclusion at the forced midpoint: while the deepest entry is post-label and the root is pre-label, acquisition of the per-Sentinel read guard must fail; after the writer completes and releases its guard, the waiting reader must obtain the exact post-label pair. That exclusion subsumes any finite race count because no admitted schedule can enter the protected intermediate.

## What the code offers today · `sec:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-current-code`

The outcome-ledger container stores each `SentinelLedger` behind one shared read-write lock. The label path takes that Sentinel's write guard once and passes a mutable ledger to `update_all_layers`; the all-layers update performs the complete deepest-to-root mutation before the guard leaves scope (´alg:ledger:all-layers-update´).

The assessment function obtains the same Sentinel's read guard before calling the extraction path and retains it while routing, computing the decayed entry view, and copying the Ledger values into the feature row. Existing extraction coverage proves that the row reads the deepest tracked cell (´test:unit:ledger-read-takes-the-deepest-covering-entry´).

The all-layers mechanism has a focused unit witness that every containing entry is updated (´test:unit:update-all-layers-hits-all´). That witness is sequential and cannot establish visibility while the walk is between entries.

The crate-level Ledger suite already contains the deterministic concurrency witness (´test:crate:label-assess-race-no-torn-reads´). It constructs a real `OutcomeLedger`, computes the reference post-label state through `update_all_layers`, then cuts the same write open beneath the per-Sentinel guard and parks a reader at the midpoint.

That test currently mints the synonymous statement (´claim:ledger:a-reader-racing-a-writer-sees-one-whole-value-never-a-partial-one´) instead of citing the intent. The implementation exists, its generated crate-test matrix names it, and only the canonical promise remains uncovered.

The public host path is available through `World::register_sentinel`, `World::receive_report`, `World::request_with_sentinel`, `World::assess`, `World::label`, and `World::flush_labels` in the world harness, with `golden_report` supplying a routed depth chain. These are parts of the completed skeleton (´entry:assayer:harness-stage-skeleton´), and the nearest integration scenario exercises many assess-label cycles through one Sentinel (´test:integration:root-serves-many-entities-independently´).

The public `RiskAssessment` exposes the derived risk basis and alarm summary, not the raw Ledger slots or both entries in the ancestry. A public race would therefore conflate Ledger visibility with model publication and standardisation movement, while still depending on the scheduler to land inside a very short critical section.

The standing testing plan has no gap Entry covering this promise. Its dedicated reframing Observation is (´obs:assayer:reframe-ledger-atomicity´), and the existing crate witness already implements that observation's stronger multi-entry shape.

## The witness · `sec:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-witness`

The witness remains the existing crate-level `label_assess_race_no_torn_reads` (´test:crate:label-assess-race-no-torn-reads´); moving it beside the public Ledger integration coverage would discard the only direct oracle for the protected intermediate without gaining a truer stimulus.

- Setup: create one `OutcomeLedger` Sentinel with a root and a depth-four leaf containing the same coordinate, stamp both at one fixed `PersistentTimestamp`, and define one eligible adverse update with `lambda_L = 0.5` and the configured hourly factor.

- Reference: run the production `update_all_layers` once beneath the Sentinel write guard and require exactly two touched entries, each at count one and bad-rate `0.5` by exact bits.

- Pre-label observation: acquire the read guard and require the routed leaf and root both at count zero and bad-rate `0.0`, preventing a test that observes only the final state from passing vacuously.

- Stimulus: acquire the write guard, start a reader, update the leaf exactly as the production walk does, and hold the root unchanged until the reader has attempted acquisition. The handshake uses a liveness deadline only to turn a dead partner into a failure; elapsed time is not an oracle.

- Midpoint observation: under the writer's exclusive guard, require the intentionally exposed internal state to be leaf `(1, 0.5)` and root `(0, 0.0)`, while the reader's non-blocking acquisition reports exclusion.

- Completion: apply the identical update to the root, release the write guard, join the reader, and require its one guarded observation and the final ledger both to equal the reference post-label state exactly.

- Coverage binding: replace the test documentation's synonymous bare claim mint with a citation of the intent, retaining the test label, test body, and module-index statement.

The fails-before is a deliberately weakened Ledger that locks entries separately or releases the Sentinel write guard between the leaf and root updates. The parked reader can then acquire during the forced midpoint and return leaf `(1, 0.5)` beside root `(0, 0.0)`; a write that omits the root instead fails the final equality with the production reference.

## What is missing · `sec:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-missing`

No new harness primitive, production hook, fixture, clock verb, drift budget, or sibling lane blocks this witness. The missing material is the coverage relationship between the existing test and the canonical intent.

**Entry (Bind the forced-interleaving witness to the intent)** · `entry:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-canonical-binding`

Replace the synonymous claim mint on the existing crate witness (´test:crate:label-assess-race-no-torn-reads´) with the canonical intent citation and retain the existing forced-interleaving assertions. This is approximately one authored source line plus any mechanically regenerated indexes required by the linter, depends on no other entry or lane, and introduces no public or test-support API.

## Risks and open questions · `sec:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-risks`

**Observation (Mutual exclusion is stronger than a race count)** · `obs:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-exclusion-over-sampling`

A loop of more than one hundred thousand reads measures which schedules happened to run and can remain green after losing the critical interleaving. The forced midpoint demonstrates that the forbidden state exists during mutation and that the reader cannot enter it, making the finite count unnecessary as an acceptance threshold rather than silently weakening it.

**Observation (The public result has no Ledger-state oracle)** · `obs:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-public-oracle`

Changes in public risk after a label include the model update and may include a newer standardisation snapshot, while `RiskAssessment` omits raw feature slots. Adding a test-only raw-vector accessor solely to restate a property already enforced and witnessed at the lock boundary would enlarge the harness without making the concurrency claim more direct.

**Observation (The standing lock citation names another lock pair)** · `obs:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-lock-citation-scope`

The reframing observation cites (´dec:ordering:lock-order´), whose record text orders report reception before its accumulator for deadlock freedom; it does not state per-Sentinel Ledger read-write exclusion. The witness rests instead on the Ledger concurrency invariant, the shared-lock shape in code, and the forced acquisition failure; correcting the standing plan's citation is outside this intent lane and does not block the test binding.

**Observation (Exact arithmetic removes numerical ambiguity)** · `obs:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-exact-arithmetic`

Zero, one half, and the two integer counts are exact, and the shared timestamp makes decay a multiplication by one. A tolerance would admit values no completed cell held and would weaken the promise; the existing bitwise comparison is the required numerical posture.

**Observation (Atomicity and latency remain separate)** · `obs:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-atomicity-not-latency`

The concurrency tiers give the per-Sentinel Ledger a bounded wait budget (´tab:publication:tiers´), but a test-controlled pause inside a held guard intentionally exceeds ordinary critical-section duration. This witness establishes state atomicity only; it neither measures nor relaxes the separately specified latency budget.

No maintainer decision blocks the binding. A later documentation change may rewrite the intent gloss around multi-entry atomicity and correct the standing plan's lock citation, but neither change belongs to the implementation that makes the existing witness keep this promise.

## Acceptance · `sec:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all-acceptance`

The implementing lane's report identifies `label_assess_race_no_torn_reads`, shows its documentation citing the canonical intent, confirms that the synonymous claim is no longer minted, and shows the coverage report resolving the intent to (´test:crate:label-assess-race-no-torn-reads´).

The report shows the targeted crate test passing in the prescribed development and release gates, followed by the Assayer package's formatting, lint, feature, documentation, documentation-test, and broader test gates, with unrelated failures separated explicitly.

The reported assertions include the exact pre-label pair, the deliberately constructed mixed midpoint under the writer guard, failed reader acquisition at that midpoint, the exact post-label pair after release, and equality between the cut-open write and `update_all_layers`.

The report states which assertion rejects per-entry locking, a guard released between depths, a skipped ancestor, or a tolerance-admitted scalar; no executed mutation is required.

Acceptance adds no probabilistic stress loop, sleep, wall-clock performance assertion, public accessor, production hook, or widened tolerance. The proof is the forced interleaving under the same lock and routing functions used by assessment extraction and label processing.
