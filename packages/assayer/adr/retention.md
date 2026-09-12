# Retention · `rec:retention:published-and-inflight-state`

This record answers one question in three registers: how long a piece of state lives, and what its loss costs. The three answers are deliberately different — published state is replaced whole, in-flight state is bounded and may be evicted, cached state may simply vanish — and the record's work is keeping them apart rather than folding them into a single retention policy that would be wrong about two of the three.

It realises four layer choices: the composition of what is published (`dec:representation:published-composition`), the two exclusions from it (`dec:representation:independent-publication`), the retention of assessments in flight (`dec:representation:pending-retention`) and the losability of the cache (`dec:representation:cache-losable`).

**Decision (Published model state is one snapshot, one allocation, one swap)** · `dec:retention:monolithic-snapshot`

What readers see is a single snapshot built whole in one allocation and installed with one swap, so a reader observes the previous state entire or the next state entire and never a mixture. Its contents are the specification's (`def:publication:model-snapshot`); the swap discipline it publishes under is the concurrency record's (`dec:concurrency:snapshot-swap`).

Monolithic is the decision. The parts could have been published separately and more cheaply, and the cost of doing so is a reader holding two pieces of state that were never simultaneously true — which is discussed below (`disc:retention:split-swap-alternative`).

**Decision (The precision matrix is excluded and reconstructed on demand)** · `dec:retention:precision-excluded`

The precision matrix is not carried in the snapshot. The paths that need it reconstruct it; the path that answers a request does not need it, because it reads the covariance only (`dec:substrate:covariance-only`).

Carrying both would double the published bulk to serve a path that never reads the second. The specification records why both are tracked at all (`rem:gaussian:dual-tracking`), and nothing here disputes that — tracking and publishing are different questions, and only the second is this record's.

**Decision (Health publishes through a second, independent swap)** · `dec:retention:health-independent`

Health state is published through its own swap, independent of the model's. A health update does not force a model publication and a model publication does not wait on health. The two are separate installations of separate values, and a reader observing a new one has learned nothing about the other.

The health a result carries inline is a distinct thing from the state this swap publishes (`schema:output:health-snapshot`), and conflating them is the error this separation exists to prevent: one travels with an answer, the other is polled.

**Decision (Companion state is excluded entirely and is not published at all)** · `dec:retention:companion-excluded`

Companion state is absent from the model snapshot and from every other publication the package performs. It is not excluded for bulk and not published on a slower cadence: the package does not own it, so there is nothing for it to publish.

Absence is what makes the independence structural rather than asserted (`inv:guarantee:companion-independence`). A Companion field carried in the snapshot and documented as host-owned would leave the guarantee resting on everyone agreeing not to read it.

**Decision (An assessment awaiting its label is held in a concurrent map)** · `dec:retention:pending-map`

An assessment awaiting its label is held in a concurrent map with the eviction ordering maintained separately beside it (`src/pending/buffer.rs`). The map carries the entry the specification defines (`def:runtime:pending-entry`) and is readable concurrently with the paths that write it (`req:publication:pending-buffer`).

Splitting the lookup structure from the ordering structure is the load-bearing part. A single ordered structure would serialise lookups behind eviction bookkeeping, and lookup is what the label path does on every submission while eviction is occasional.

**Decision (In-flight features are retained at reduced precision)** · `dec:retention:reduced-precision`

Features held in flight are stored at reduced precision, under a declared capacity (`req:runtime:buffer-capacity`). The precision is the specification's (`def:runtime:storage-precision`); what is decided here is that the buffer holds the reduced form rather than the form the models compute in.

The buffer's size is the product of its capacity and its entry width, and only the second was available to change without changing what the buffer promises. Halving the width halves the memory a bounded backlog costs, at an accuracy loss the label path absorbs because it is reconstructing rather than continuing a computation.

**Decision (Eviction is lazy and capped per insert)** · `dec:retention:lazy-eviction`

Eviction happens on insert, and at most a fixed number of entries are removed per insert. There is no sweep, no background pass, and no caller that pays for the whole backlog at once.

The cap is what makes the bound affordable rather than merely maintained. Without it, the insert that happens to find a large expired backlog pays for all of it, so the worst-case latency of a submission would depend on how long the system had been idle. With it, that latency is constant and the bound is approached over several inserts instead of restored in one.

**Decision (The journal-serialisable subset omits what replay does not consume)** · `dec:retention:journal-subset`

What is written to the journal is a subset of the entry, not the entry. Fields that replay does not read are not serialised, so the journal carries what a replay needs and nothing kept for the convenience of a reader that does not exist.

The subset is defined by consumption, which is what keeps it honest: a field becomes journal-serialisable when replay starts reading it and not when someone judges it interesting. Replay's exactness rests on what the journal holds being sufficient, and that is the durability record's (`cor:durability:replay-exactness`).

**Decision (Entity-persistent signals are cached pre-encoded)** · `dec:retention:cache-preencoded`

Signals that persist per entity are cached in encoded form rather than re-encoded on every read (`tab:keyspace:signal-cache`). The cache stores the result of the encoding, so a repeat visitor costs a lookup where a first-time one costs an encode.

Caching the encoded form rather than the raw value is the choice. The raw form would be smaller and would leave the per-read encoding cost in place, which is the cost the cache exists to remove.

**Decision (The cache is not persisted and is allowed to be lost)** · `dec:retention:cache-losable`

The cache is not persisted, is not checkpointed, and repopulates through ordinary traffic after a restart. It is the one piece of state the package is willing to lose outright.

Willingness rests on what the loss costs, which is accuracy for entities not seen again since the restart and nothing else — the specification records the consequence rather than concealing it (`cav:limitation:signal-cache`). No label is lost, no published state is wrong, and the deficit closes as traffic returns.

**Remark (Why the buffer and the cache are separate choices)** · `rem:retention:loss-asymmetry`

The two look alike — bounded stores of transient state, evicting under pressure — and their answers to *what happens when this is lost* are opposites. Losing a pending entry loses a label the host already submitted and cannot resubmit. Losing a cache entry loses work that will be redone on the next read.

That asymmetry is why the buffer is bounded reluctantly and the cache enthusiastically, why one is journalled in part and the other not at all, and why a single retention policy over both would have to be the strict one, paying the cache's costs for a guarantee the cache does not need.

**Remark (Why the two exclusions from the snapshot are separate)** · `rem:retention:exclusion-asymmetry`

Health and Companion state are both absent from the model snapshot, for opposite reasons. Health is excluded because it publishes on its own cadence and coupling the two would make each wait for the other. Companion state is excluded because the package does not own it and has nothing to publish.

Reading them as one exclusion is what previously let the corpus treat the Companion's independence as a composition decision. It is an ownership fact (`dec:retention:companion-excluded`), and the health swap is a cadence decision (`dec:retention:health-independent`); merging them would lose the reason for each.

**Remark (What the cap of sixteen buys)** · `rem:retention:eviction-bound-figure`

The decision above settles that there is a cap, not what it is (`dec:retention:lazy-eviction`). The figure is sixteen, and what it governs is the length of one critical section. The pass that chooses victims runs under the single mutex guarding the eviction order, so a submission holds that mutex for at most sixteen turns of a length check, a front lookup and a pop, and every concurrent submission queues behind exactly that (`src/pending/buffer.rs`). The map removals the pass selected happen after the mutex is released, so the work that scales with the cap is not all of it work another caller waits on.

The shape of the figure is derived even though the figure is not. Capacity is twice the expected peak request rate times the expected median labelling latency (`req:runtime:buffer-capacity`), and submissions arrive at that same rate, so a buffer whose every entry has expired is cleared in twice the latency divided by the cap: the rate cancels, and what is left depends on the labelling latency and the cap alone. At sixteen that is an eighth of the labelling latency — four hundred and fifty seconds at the reference pair of one hundred requests a second and an hour (`def:config:reference-peak-request-rate-100-per-second`), (`def:config:reference-label-latency-3600-seconds`) — against an expiry horizon of twenty-four hours (`tab:config:pending-buffer`). Any cap above zero drains a backlog eventually; what sixteen buys is draining a full one inside a small fraction of the window in which those entries' labels were expected.

Sixteen exactly is a shipped operating point and not a computed optimum. The derivation above constrains the figure to a range and no further, and nothing in this package measures the critical section it bounds, so no evidence here distinguishes sixteen from eight or from thirty-two. The measurement that would let it be revised on evidence is the eviction counter this record already defers (`entry:retention:eviction-counter`). Until that lands the cap stands as a chosen figure with a stated consequence, and is revisable.

**Discussion (The rejected snapshot of independently swapped parts)** · `disc:retention:split-swap-alternative`

The alternative was to publish the snapshot's parts independently, each behind its own swap. It is cheaper — a change to one part costs one small allocation instead of a whole snapshot — and it admits a reader that loads two parts and observes a combination that was never simultaneously true.

The cost is not hypothetical for this package: a model and a dimension map that disagree describe features at positions that do not mean what the reader thinks. One allocation per publication is the price of never having to reason about which combinations are observable.

**Discussion (The rejected unbounded retention)** · `disc:retention:unbounded-alternative`

The alternative was to keep every pending assessment until its label arrived, bounding nothing. It buys the guarantee that a label always finds its assessment, and it buys it with a memory footprint set by whichever host submits fewest labels.

The guarantee is one a host cannot use. It holds only until the process is restarted or the machine runs out of memory, so what it actually offers is an unbounded footprint in exchange for a promise with an unstated and externally-determined expiry. A declared capacity and a stated loss are worth more than that.

**Caveat (A label arriving after its entry is evicted is lost)** · `cav:retention:evicted-label`

Eviction can outrun a slow host. A label submitted after its pending entry has been evicted has nothing to attach to and is lost: the submission does not fail, and the model does not learn from it. The specification states this rather than promising otherwise (`cav:limitation:buffer-eviction`).

It is stated here too because it is the direct cost of the bound this record chose, and a reader who has just read that eviction is lazy, capped and cheap is owed the sentence saying what it destroys. The mitigation is capacity and horizon configuration, not a guarantee.

**Entry (Pending-buffer eviction counter on the assessment snapshot)** · `entry:retention:eviction-counter`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-pending-eviction-counter`). The pending buffer increments its atomic counter only when lazy eviction successfully removes a live map entry; an identifier left in the FIFO after a label consumed its entry does not count (`src/pending/buffer.rs`). The assessment path takes and clears that counter after inserting its own pending entry, so any eviction caused by the insertion travels inline on that same result as `HealthSnapshot.pending_buffer_evictions` (`src/assessment.rs`).

The completion trigger is met: capacity and expiry eviction of live entries both increment, the read transfers the interval exactly once, and the compact snapshot exposes the transferred count.

**Entry (Pending-buffer eviction rate and oldest entry age)** · `entry:retention:eviction-visibility`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-buffer-eviction-visibility`). Buffer health now carries an eviction rate and the optional age of the oldest live pending entry beside capacity and utilisation (`src/health/summary.rs`). Assembly reads both from the buffer (`src/api/health.rs`): the rate is successful live-entry evictions divided by pending insertions over the buffer's lifetime, the same lifetime-event basis used by the report's other ratios; the age is elapsed monotonic time since the earliest timestamp still present in the live map, and is absent when that map is empty.

The completion trigger is met. The lifetime numerator is advanced by the same successful removal that advances the inline interval counter above (`entry:retention:eviction-counter`), while the non-destructive report reading cannot compete with the assessment path's take-and-clear operation.
