# Concurrency · `rec:concurrency:snapshot-stewardship`

This record fixes how many writers there are, how their work becomes visible, and what a reader is guaranteed in return. It realises two layer choices — the publication discipline (`dec:structural:publication`) and the write serialisation that stands behind it (`dec:structural:write-serialisation`) — and it carries the collapse of the sharpest contradiction the record census found, recorded below at (`reg:concurrency:collapsed-loop`).

The specification defines the snapshot and states the guarantees; this record decides the arrangement that delivers them, and cites rather than restates.

**Decision (Learned state reaches readers only as a swapped snapshot)** · `dec:concurrency:snapshot-swap`

Learned state becomes visible to a reader in exactly one way: an immutable snapshot, built whole and installed by an atomic swap. There is no partially published state and no moment at which a reader observes a model mid-mutation, because no reader ever observes a model — it observes a snapshot that is already finished. The snapshot's contents are the specification's (`def:publication:model-snapshot`), and it is installed on the interval the specification fixes (`req:publication:interval`).

This is the whole of the visibility mechanism. Every later claim in this record about what a reader may see is a consequence of it rather than an addition to it.

**Decision (Readers load per request, not per batch)** · `dec:concurrency:per-request-load`

A reader loads the current snapshot once per request. It does not load once per batch and hold that pointer across the batch's requests, which means a publication landing in the middle of a batch is visible to the remainder of that batch. Two requests in one batch may therefore be answered against different snapshots, and that is intended: the alternative buys one pointer load and pays for it in freshness, which is discussed below (`disc:concurrency:per-batch-alternative`).

**Decision (Report indices publish independently of the model)** · `dec:concurrency:index-independence`

The per-Sentinel report indices publish under the same swap discipline as the model and independently of it. Neither publication waits on the other, and neither is ordered against the other: a reader that observes a new index has learned nothing about which model snapshot it will observe. Coupling them would have made every index update wait for a model publication interval it has no reason to share.

**Decision (One steward owns the working copy)** · `dec:concurrency:single-steward`

One steward owns the working copy exclusively. There is no shared mutable model state anywhere in the package — not behind a lock, not behind an atomic, not under a convention about which thread may touch what. Everything that mutates the model happens on the steward, and everything else reads a snapshot.

This is what makes the swap of (`dec:concurrency:snapshot-swap`) sound without any further synchronisation: the writer is single, so there is no write-write race to prevent, and the readers are on snapshots, so there is no read-write race either. The lock that a different design would need is discussed at (`disc:concurrency:lock-alternative`).

**Decision (Structure preempts observation)** · `dec:concurrency:channel-preemption`

Work reaches the steward on two channels — structural commands and observations — and the command channel drains completely before any observation is processed. A registration is therefore never overtaken by the labels that follow it, which is the specification's lifecycle-publication guarantee (`inv:guarantee:lifecycle-publication`) obtained by the order of two drains rather than by any barrier or generation counter.

The preemption is unconditional. It is not a priority hint and not a heuristic about which queue looks busier; the observation channel is not consulted while the command channel is non-empty.

**Decision (Neither channel may drop silently)** · `dec:concurrency:no-silent-drop`

Overflow on either channel is an error returned to the caller. Nothing is discarded quietly, and no counter stands in for an item that was dropped without the submitter learning. The specification fixes the queue and its capacity (`req:publication:label-queue`); what is decided here is that exhausting it is the caller's news rather than the package's secret.

The posture is the degradation record's, applied to a queue: a condition the host can act on is reported to the host, and a condition it cannot act on is not turned into an error it can only log (`dec:degradation:error-partition`).

**Decision (The steward is a dedicated named thread)** · `dec:concurrency:named-thread`

The steward runs on a dedicated thread with a name fixed per instance, not on a pool and not as a task on a shared executor. Its identity is therefore stable for the process's lifetime, and a stall is attributable to it by name in a stack dump or a scheduler trace without correlating anything. A pooled or task-based steward would have satisfied the single-writer property equally and would have made this diagnosis a research exercise. Where the thread is spawned and named is the construction record's decision, not this one.

**Decision (A traversal under a shared lock is cut into bounded pieces)** · `dec:concurrency:bounded-traversal`

Two paths walk a structure that other threads are reading under a lock they must share — the Ledger's collector and the guidance call's starvation scoring — and neither may hold its lock for the length of its walk. Each is therefore given a figure, and the figures are three.

The collector is given two, because it takes two kinds of lock. It inspects at most five hundred and twelve entries under one read acquisition, and it removes at most sixty-four under one write acquisition. Which of the two is the smaller is decided: a read acquisition excludes only writers, while a write acquisition excludes every reader, so the same wall-clock hold is paid for by more threads and is bought in smaller amounts. How much smaller is not decided — the factor of eight between them is a shipped operating point and nothing here derives it.

Guidance is given one: at most five hundred and twelve read acquisitions per Sentinel in one call. It matches the collector's scan figure in spelling only. The two are not derived from each other and moving one implies nothing about the other.

What the two bounds mean is not the same thing, and the difference is the point of stating them together. The collector's figures bound the size of an acquisition and not the work: it reacquires and continues until the sweep is done, so a Ledger twice the size costs twice the acquisitions and collects exactly as much. Guidance's figure bounds the number of acquisitions and therefore the work itself, which is what the caveat below is about.

**Decision (The deferral queues are bounded, and overflow is counted)** · `dec:concurrency:deferral-depth`

Two queues carry work off a hot path to a background consumer: deferred identity observations leaving the assessment path, and health events leaving wherever they arise. Both are bounded, and neither ever blocks its producer. A full queue discards the offered item and increments a counter, which is what keeps the discard inside (`dec:concurrency:no-silent-drop`) — the drop is not silent because the count is the thing that says it happened.

The depths are a hundred thousand for observations and a thousand for health events, and the two orders of magnitude between them are the one part of this that is reasoned rather than picked. The observation queue is offered to by every assessment, so its depth buys time against a consumer that has fallen behind on the system's hottest path. Health events arise from state changes rather than from traffic, so the same wall-clock cushion costs far less depth. The ordering of the two is decided; the magnitudes are shipped operating points, and this record does not pretend to derive them.

What makes that admission tolerable is that neither figure has to be argued from first principles to be maintained. Each queue counts its own discards, so a depth that is too small announces itself in the only way that matters — a nonzero count under load that the consumer should have absorbed — and the counter, not an argument, is the evidence that would justify moving either figure.

**Caveat (Guidance's ceiling truncates rather than continuing)** · `cav:concurrency:guidance-truncation`

A guidance call that reaches its per-Sentinel ceiling does not reacquire and carry on. It skips the remaining cells of that Sentinel, and because a candidate's relief score is the maximum over the cells actually read, a skipped cell can only fail to raise it. The call therefore returns a score that may be understated, with nothing in the result marking that it was.

This is a real cost and is recorded rather than resolved. It is not the silent drop that (`dec:concurrency:no-silent-drop`) forbids — nothing submitted is lost, and the path is advisory rather than one that owes an acknowledgement — but it is the same shape of defect one level down, and the honest statement is that the bound was chosen high enough that reaching it was not expected rather than that reaching it is handled. Unlike the collector's two figures, which a caller may replace, this one is fixed in the code with no host-facing dial.

**Corollary (What a reader is owed)** · `cor:concurrency:reader-obligations`

Three things follow, and they are the specification's guarantees rather than new promises. A reader never blocks on a writer (`inv:guarantee:non-blocking`), because it takes a snapshot and the writer never takes anything a reader holds. A reader is bounded in staleness rather than in freshness (`inv:guarantee:staleness`): it is promised that what it sees is no older than a stated bound, and it is promised nothing at all about seeing the newest state. A reader never observes a lifecycle event partway (`inv:guarantee:lifecycle-publication`).

The second is the one worth stating plainly, because it is the one a reader is most likely to assume the opposite of. Freshness is not offered here and no downstream record may promise it.

**Register (The one loop and the three statements it replaces)** · `reg:concurrency:collapsed-loop`

There is one steward loop and one selection discipline, and this environment is its only statement. The corpus previously carried three: one record rewrote the loop with an unbiased selection and an immediate exit, while two others decided a biased selection and the command drain — and each of the three asserted that the others were preserved. The census recorded the divergence with its evidence (`reg:assayer:adr-contradictions-records`); what it recorded was not a disagreement about what to decide but three drifted copies of one decision.

The discipline that stands is the preemption of (`dec:concurrency:channel-preemption`): commands drain, then observations are processed. It is stated there, once, and everything else in the corpus cites it.

**Discussion (The rejected lock)** · `disc:concurrency:lock-alternative`

The alternative was a lock over shared mutable model state, with readers taking it briefly. It is simpler to describe and it trades away the guarantee the package exists to offer: under it a reader's latency depends on a writer's, and the non-blocking guarantee becomes a claim about how short the critical sections happen to be. It also dissolves the single-writer property, so every future mutation site becomes a place where the lock discipline can be got wrong.

**Discussion (The rejected per-batch load)** · `disc:concurrency:per-batch-alternative`

The alternative was to load the snapshot once per batch. It saves one atomic pointer load per request, which is not a cost anything in the profile attributes to this path. It buys that saving by making a batch's answers depend on when the batch happened to start, so a publication that lands during a long batch is invisible to it entirely. Visibility is worth more than the load.

**Remark (What a compound structural event costs the steward)** · `rem:concurrency:compound-drain`

A compound structural event drains as one unit (`req:registry:compound-events`): the steward does not interleave observations between its constituent commands, because a reader observing the midpoint of a compound event would observe a structure that was never intended to exist. The cost is that the observation channel waits for the whole compound event rather than for its longest single command, and a large compound event is therefore the longest pause the observation path can experience. That cost is accepted here and is not smoothed over; the construction record owns what a compound batch does about failures partway through.

**Entry (Intra-label parallelism for independent model updates)** · `entry:concurrency:label-parallelism`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-label-parallelism`).

The label path updates the operational, sister, anchor and per-axis models in sequence, even though the updates are data-independent once the shared feature vector has been built. Whether they may proceed in parallel underneath the single steward is open. The single-writer property of (`dec:concurrency:single-steward`) does not settle it: parallelism here would be internal to one steward's handling of one label, not a second writer.

The trigger's profiling premise was measured in release mode on the worker server at the representative dense width of 646 positions and five outcome axes. Each run warmed eight labels, timed 128 labels, and set the recomputation interval to 10,000 so periodic factorisation did not enter the sample. The instrumented comparison timed the operational, sister, anchor and five axis rank-one updates directly, then timed the complete label publication path with every model asserted to have received all eight warm-up labels.

Three accepted runs attributed 3,246,529,322 of 3,697,559,733 nanoseconds (87.8019%), 3,341,274,456 of 3,776,556,923 nanoseconds (88.4741%), and 2,647,728,111 of 2,864,018,926 nanoseconds (92.4480%) to that sequential update block. The median share was 88.4741%, so the profiling premise is met by a wide margin (`test:crate:label-model-update-profile`).

The label path now uses one `std::thread::scope` beneath the steward. One helper updates the operational record, a second updates sister then anchor in their existing order, and the steward updates the axis records in label order. Two helpers are the fixed bound; the scope joins before standalone decay, recomputation checks, events, or publication, so the single steward remains the only owner and no partially updated state is observable. No dependency was added. A focused equivalence test proves that the three execution groups publish means and covariances bit-for-bit identical to the former sequential order (`claim:concurrency:bounded-label-model-update-groups-preserve-the-sequential-results`).

The completion trigger is met: repeated profiling attributes publication latency to the independent updates, and those updates now execute with a fixed two-helper bound beneath the single steward.
