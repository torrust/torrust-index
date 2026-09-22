### Chapter (Concurrency) · `chap:spec:concurrency`

The chapter states what may wait on what. One principle governs it, and the rest of the chapter is that principle applied component by component: what is published and how, what drains off the hot path, what staleness each component may carry, and what budget each shared structure is held to. Three of the document's formal properties are stated here as the whole content of an environment and therefore mint here rather than in the register that collects them.

**Invariant (Non-blocking assessment)** · `inv:guarantee:non-blocking`

An assessment does not wait on any other operation beyond a bounded, per-component budget. No label, no lifecycle event, no report reception, no identity maintenance and no concurrent assessment may delay an assessment's return by more than the budget declared for the component it reads. Most paths declare a structural budget, meaning lock-free and therefore none; a small number declare a bounded budget with an explicit upper bound and a tail probability (`tab:publication:tiers`).

An assessment may read state that is not the most recent. One computed during a Sentinel registration may reflect the pre-registration model; one computed while a label is processing may not include that label. This is accepted, and the acceptance is the invariant's whole content: **staleness is acceptable and delay is not**. Stale state is bounded and self-correcting (`inv:guarantee:staleness`); an unbounded wait is neither, and a request-path component that can block is a component that will, under exactly the load where blocking is least affordable.

Every other operation in the system must be organised so the budget holds. The specification does not prescribe how — double buffering, sharding, deferral and atomic swaps are all admissible, and the chapter names the ones the design uses rather than requiring them. It requires the property: published state is loaded lock-free on the assessment path.

**Definition (The published model snapshot)** · `def:publication:model-snapshot`

Assessments read a published snapshot acquired once, at the start of the call. The snapshot carries a version and a publication timestamp, the three risk models, the per-axis models with their own scales and eligibility modes, the dimension map, the standardisation means and variances, the two class-rate trackers, the three calibration parameters, the calibration buffer, and the drift state per model.

The snapshot is self-consistent by construction. Model parameters, dimension map, standardisation statistics, competitive sets, calibration state and every derived quantity correspond to one version, and no assessment ever observes a mixture of two (`inv:dimension:version-consistency`). That is what makes the dimension map and the parameters indexed against it safe to read without a lock: they cannot disagree, because they are published together or not at all.

Publication is componentised. Each parameter block, the map, the standardisation vectors and the calibration state sit behind independent shared pointers, and structurally unchanged components are shared across versions rather than copied, so an ordinary label never copies the lifecycle-stable state. Because the rank-one update touches every element of a model's precision and covariance (`alg:update:sherman-morrison`), the parameter components are double-buffered: the label path mutates the working buffer, publication is a pointer swap, and resynchronising the now-stale buffer is charged to label processing rather than to any reader.

No challenge effectiveness state appears, because that is the Companion's (`def:companion:state`). Sentinel reports are published independently and may be read at a different version than the model, which is correct: a report is a measurement and not a parameter.

**Requirement (The publication interval)** · `req:publication:interval`

The publication interval is an amortisation knob, defaulting to one label. At a value of $k$ the working copy is published every $k$ labels rather than every one, amortising the resynchronisation copy across them at the cost of widening every staleness bound by up to $k$ labels (`tab:publication:staleness-bounds`).

The trade is explicit and belongs to the host. A deployment whose label rate is high enough that the per-label resynchronisation is material can buy it back with staleness it can quantify; a deployment that would rather see every label immediately keeps the default and pays. Nothing else changes with the setting: the read-time uncertainty correction goes on correcting for snapshot age whatever the interval is, so a host trading freshness for throughput still reads honest uncertainty rather than confident staleness (`def:runtime:time-correction`).

Pre-seeding is the clearest case for a value above one, publishing once at the end of a historical batch instead of once per historical label (`alg:host:pre-seeding`). The Core configuration therefore carries this interval, and the label path publishes when the interval elapses; a field of the same name on the blend-statistics configuration governs a different quantity entirely.

**Algorithm (Deferred identity observation draining)** · `alg:publication:identity-draining`

Identity dimension graphs take their observation volume from assessment traffic, and performing those observations inline would put graph maintenance — splitting, eviction, competitive restructuring — on the request path. They are deferred instead: the assessment records each observation to a per-dimension queue and returns (`alg:keyspace:observation-protocol`).

A background task drains the queues on its own cadence, batched, independent of both assessment and label processing. For each queued observation it performs the graph observation, maintains importance, and triggers whatever structural operations follow (`alg:keyspace:cell-entry`) and (`alg:keyspace:cell-exit`). Those operations publish through the same lifecycle protocol every other structural change uses (`inv:guarantee:lifecycle-publication`), and assessments read the competitive set from the most recently published state (`def:keyspace:competitive-set`).

The enqueue is the only cost the assessment path bears, and the queue is bounded and drops oldest under sustained pressure. Dropping is safe here in a way it would not be elsewhere: unit observations are exchangeable, so a dropped one slows the competitive set's adaptation and corrupts nothing.

**Invariant (Ledger concurrency)** · `inv:publication:ledger-concurrency`

Assessments read Ledger entries and never modify them. The time-indexed decay is computed at read time as a pure function of elapsed time, so a read produces the decayed value without storing it (`def:ledger:time-decay`). Stored entries move only during label processing, under the bounded per-Sentinel write budget (`alg:ledger:all-layers-update`).

The property is what makes outcome memory affordable on the request path at all. A decay-and-store read would make every assessment a writer of every entry it touched, which would put a write lock on the one structure an assessment consults most and turn read contention into write contention. Purity removes the question on both paths.

**Table (Staleness bounds)** · `tab:publication:staleness-bounds`

| Component | Staleness bound | Evolution timescale |
| --- | --- | --- |
| Model parameters | Labels processed since the last publication | Per label, per weight |
| Dimension map | Lifecycle events since the last publication | Hours to never |
| Standardisation statistics | Accepted cold observations since the last ramp publication, while transitioning; labels processed since the last publication, in service | Per accepted observation while transitioning; per label, per feature, in service |
| Identity competitive set | Restructuring events since the last publication | Hours to days |
| Calibration parameters | Refits since the last publication | Every refit interval |
| Sentinel batch report | Reports received since the last read | Seconds to minutes |
| Ledger averages | None; the read-time decay is exact | — |

Each bound is scaled by the publication interval where one is set above the default (`req:publication:interval`). The table's shape is its argument: the components whose bounds are counted in labels move slowly per label, and the one component with no bound at all is the one read through a pure function rather than from published state. Standardisation is the one row counted two ways, because it is fed by two authorities and the clock changes when the cold ramp reaches its horizon (`alg:standardisation:batch-initialisation`); a reader holding the phase knows which of its two bounds applies. Every row is a published-and-swapped component, including the transitioning bound: each accepted cold observation publishes the ramp's next coordinate snapshot.

**Invariant (Bounded staleness)** · `inv:guarantee:staleness`

Every read is stale by at most its component's declared bound, and the bounds are the table above. Nothing is unboundedly stale, and nothing is stale in a way the reader cannot see.

Two mechanisms make the property useful rather than merely true. Staleness is self-correcting: every bound is measured in events that are themselves arriving, so a component's staleness returns to zero on its next publication without any reconciliation step. And where staleness has a quantitative consequence, that consequence is computed and reported rather than absorbed — an uncertainty read from an old snapshot is inflated by exactly the decay that has not yet been applied (`def:runtime:time-correction`).

The invariant is what a host is owed in exchange for the non-blocking guarantee. An assessment never waits, and in return it may read state a few labels old; without the bound that would be an unlimited licence, and with it the trade is one a host can reason about. The read-time correction applies.

**Invariant (Lifecycle publication)** · `inv:guarantee:lifecycle-publication`

A lifecycle event — the registration or deregistration of a Sentinel, an outcome axis or an identity dimension — modifies model structure. No such modification is ever visible to an assessment already in flight. An assessment beginning before the event completes reads pre-event state throughout; one beginning after reads post-event state throughout. No assessment is drained, paused or delayed by a lifecycle event.

The two halves are separate promises and both are load-bearing. Atomicity is what makes the structural guarantee of the lifecycle operations meaningful at runtime: even a correctly partitioned operation is worth nothing if a reader can observe it half-applied and compute against a map that never existed (`inv:dimension:version-consistency`). Non-interruption is what makes lifecycle events usable in a live system rather than operations requiring a quiet period — a host may register a Sentinel at peak traffic and no request pays for it.

Structural changes therefore publish exactly as parameter changes do, through the same swap, and the standardisation entries a lifecycle event creates move under the same publication (`req:standardisation:lifecycle-entries`).

**Requirement (The pending buffer under concurrency)** · `req:publication:pending-buffer`

The pending buffer is written by assessment and read by label processing (`def:runtime:pending-entry`). Neither path may delay the other, and in particular the recording of an entry must not delay the assessment that records it.

The requirement is easy to state and easy to violate, because a buffer with a capacity bound invites a lock around the eviction decision. The buffer is therefore a concurrent map, insertion is lock-free or sharded, and the capacity discipline is enforced without serialising the writers (`req:runtime:buffer-capacity`).

**Requirement (The label queue)** · `req:publication:label-queue`

Labels arriving faster than the model-owner path can absorb are held in a bounded queue off the assessment path. Accepted labels remain first-in-first-out. On overflow the owning concurrency decision refuses the newest submission immediately rather than silently dropping an accepted one (`dec:concurrency:no-silent-drop`).

A refused label is lost evidence unless the host retries it, so every full-queue refusal is counted. The typed error remains the per-call acknowledgement; queue depth and the cumulative loss counter are the public health signal that label volume has outrun processing capacity (`chap:spec:output-structures`). The capacity is tabulated with the other concurrency parameters (`tab:config:concurrency`).

**Requirement (Report reception)** · `req:publication:report-reception`

Reception publishes a Sentinel's new report by an atomic swap, and cell-set maintenance against the Ledger runs inside the same call, before the swap (`alg:runtime:report-reception`) and (`alg:runtime:cell-set-maintenance`).

The ordering is the requirement. Maintenance creates Ledger entries for newly reported cells and deletes entries for persistently absent ones; if the report were published first, assessments in the interval would route against a report index describing cells the Ledger index does not yet hold, and would read the ancestor entry where a fresh entry was about to exist. Doing the maintenance first makes the swap the single moment at which both indices become current together. The maintenance falls under the Ledger's bounded write budget; the publication itself is structural.

**Table (The concurrency tiers)** · `tab:publication:tiers`

| Component | Tier | Budget |
| --- | --- | --- |
| Model snapshot | Structural | Lock-free load of a swapped pointer |
| Dimension map, within the snapshot | Structural | Published with the snapshot |
| Per-Sentinel report index | Structural | Lock-free load of a swapped pointer |
| Pending buffer insertion | Structural | Concurrent map, lock-free or sharded |
| Concordance window and bootstrap accumulators | Structural | Sharded lock-free accumulators |
| Deferred identity-observation enqueue | Structural | Bounded queue, drop-oldest |
| Identity graph observation | Structural | Deferred entirely off this path |
| Per-dimension measurement state | Bounded | $B = 20\,\mu\text{s}$ at $p \le 10^{-4}$ |
| Per-Sentinel Ledger | Bounded | $B = 50\,\mu\text{s}$ at $p \le 10^{-4}$ |

And four properties over the whole call.

| Property | Force |
| --- | --- |
| Assessment is lock-free on every structural component | Unconditional |
| Assessment's total wait on bounded components is within the budget | Unconditional bound; the tail is governed by $p$ |
| Assessment reads self-consistent state at each version | Unconditional |
| Assessment may read stale state | Bounded and self-correcting |

Two bounded rows out of nine is the whole of the system's blocking, and both are per-component locks over small updates rather than anything held across a model computation. Every structural row is a lock-free swap, and the two bounded rows are the two locks named.
