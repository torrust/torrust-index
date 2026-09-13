# Layer 2: Representations · `spec:representation:state-representations`

This document fixes the conceptual choices of the representation layer. It is one of the five layer outlines that supply the middle term of the projection (`dec:assayer:projection-principle`): the specification fixes the concepts, the layer outlines fix the choices those concepts leave open, and the decision-record set is derived from the choices. It is not a summary of records; the record census found the layer documents had only ever been that (`obs:assayer:adr-layers-not-outlines`).

A *conceptual choice* here is a question the specification poses but does not answer, where the answer binds every implementation of the layer and could have gone another way. Each choice below is stated exactly once, which is the remedy the census prescribed for a corpus whose every contradiction was a drift between copies (`obs:assayer:adr-contradiction-mechanism`). This layer is where that prescription bites hardest: the census found one structure written out in five records with three different shapes, and the rewritten set is to cite the owning environment instead (`obs:assayer:adr-duplicate-definition`).

Each choice names, in prose, the record cluster expected to own its decisions (`plan:assayer:decision-record-architecture`). Most of those clusters have since landed, and a choice whose cluster has landed now carries a citation of the landed record's identity decision alongside the naming; a choice whose cluster remains unlanded is still named only. The tracking relation over every landed record is carried once, by the master register (`docs/adrs.md`), and is not repeated here: this outline connects to its records by citing their identity decisions rather than by a tracking table of its own.

## Scope · `sec:representation:scope`

**Summary (What this layer fixes)** · `summ:representation:scope`

The representation layer fixes what the system's state is made of: the substrate the posterior is carried in, what the published snapshot contains and what it deliberately omits, what is held per entity while a decision is in flight, what is cached and allowed to be lost, and how state keyed by position in the coordinate space is stored and reached. It inherits the model's shape and the publication discipline from the structural layer (`summ:structural:scope`) and supplies the operational layer with the structures its paths traverse.

Nine choices. The census derived five for this layer (`tab:assayer:adr-disposition-representation`); this outline splits four of them, on the reasoning recorded at each.

**Convention (One owner per structure, cited never restated)** · `conv:representation:single-definition`

Every structure this layer fixes has exactly one owning environment, and every other environment that needs it cites that owner. No structure is reproduced. This is the layer's operative discipline rather than a stylistic preference: the five-way divergence of one structure's field list, and the two records that specified opposite triangle-mirror directions while each attributing the choice to the other, are what restatement produces (`reg:assayer:adr-contradictions-records`).

## The conceptual choices · `sec:representation:choices`

**Decision (Symmetry is a type invariant, not a caller's duty)** · `dec:representation:matrix-substrate`

The posterior's matrices are carried in a wrapper type whose symmetry is an invariant restored after every mutating operation, not a property callers are trusted to preserve. No raw mutable access is exposed; all mutation goes through named operations, each of which leaves the invariant standing. Mutation writes one triangle and mirrors it into the other, in one fixed direction. Two constructor trust levels separate a matrix computed by the package from one restored from storage, so a stored matrix is validated where a computed one is not.

This is where the census found two records specifying opposite mirror directions, each attributing its direction to the other, with one conceding in its own review table that it was wrong and never amending its body (`reg:assayer:adr-contradictions-records`). The direction is a single choice with a single statement, made here. The invariant is what lets the specification's extension and marginalisation results (`thm:gaussian:extension`) and (`thm:gaussian:marginalisation`) be applied without revalidating their premise at each call, and what the precision guarantee rests on (`inv:guarantee:precision`).

Expected owner: the *model substrate* record (`dec:substrate:dense-dynamic`).

**Decision (One backend, isolated behind the wrapper)** · `dec:representation:backend-isolation`

A single linear-algebra backend serves the whole workspace, and the wrapper isolates it so that replacing it touches no caller. Serialisation is written by hand rather than derived from the backend's own types, so the persisted form is owned by this package and does not change when the backend does.

The census grouped backend choice with the matrix substrate. It is separated here because it answers a different question — *what does the package depend on* rather than *what does the package guarantee about its own type* — and because the isolation is what makes the operation costs the specification tabulates (`tab:gaussian:operation-costs`) a property of the algorithm rather than of the dependency. Expected owner: the *model substrate* record (`dec:substrate:dense-dynamic`).

**Decision (One published snapshot, one allocation, one swap)** · `dec:representation:published-composition`

The published model state is a single monolithic snapshot built in one allocation and installed with one swap, so a reader either sees the previous state entire or the next state entire (`def:publication:model-snapshot`). The precision matrix is excluded from it and reconstructed on demand by the paths that need it, because the assessment path reads the covariance only and carrying both would double the published bulk to serve a path that does not read it — the specification records why both are tracked at all (`rem:gaussian:dual-tracking`).

The alternative was a snapshot of independently swapped parts, which is cheaper to publish and admits a reader observing a mixture of two states. Expected owner: the *published and transient state* record (`dec:retention:monolithic-snapshot`).

**Decision (Health and Companion state publish outside the model snapshot)** · `dec:representation:independent-publication`

Health state is published through a second swap, independent of the model's, so a health update never forces a model publication and a model publication never waits on health. Companion state is excluded from the model snapshot entirely and is not published by the package at all. The health the assessment result carries inline is a distinct thing from the state this swap publishes (`schema:output:health-snapshot`), and the Companion's exclusion is what makes its independence structural rather than asserted (`inv:guarantee:companion-independence`).

The census grouped both exclusions with the snapshot's composition. They are separated here because the composition choice is about cost and atomicity while this one is about ownership: the two excluded things are excluded for opposite reasons, one because it publishes on its own cadence and one because the package does not own it. Expected owner: the *published and transient state* record (`dec:retention:monolithic-snapshot`).

**Decision (In-flight assessments are held lossily and evicted lazily)** · `dec:representation:pending-retention`

An assessment awaiting its label is held in a concurrent map with a separate eviction queue, at reduced precision (`def:runtime:pending-entry`) and (`def:runtime:storage-precision`), under a declared capacity (`req:runtime:buffer-capacity`) and readable concurrently with the paths that write it (`req:publication:pending-buffer`). Eviction is lazy and capped per insert rather than swept, so no caller pays for a sweep and the buffer's bound is maintained incrementally. The journal-serialisable subset omits what replay does not consume. A label arriving after its entry has been evicted is lost, and the specification says so rather than promising otherwise (`cav:limitation:buffer-eviction`).

The alternatives were unbounded retention, which trades a memory bound for a guarantee the host cannot use, and a swept eviction, which concentrates the cost the lazy form spreads. Expected owner: the *published and transient state* record (`dec:retention:monolithic-snapshot`).

**Decision (The signal cache is losable by construction)** · `dec:representation:cache-losable`

Entity-persistent signals are cached pre-encoded rather than re-encoded per read (`tab:keyspace:signal-cache`). The cache is not persisted, is not checkpointed, and repopulates through ordinary traffic after a restart. Losing it costs accuracy for entities not yet seen again and costs nothing else, which is why it is the one piece of state the layer is willing to lose — and the specification records the consequence rather than concealing it (`cav:limitation:signal-cache`).

The census grouped the cache with the pending buffer as one retention choice. They are separated here because their answers to *what happens when this is lost* are opposites: losing a pending entry loses a label the host already submitted, and losing a cache entry loses only work. Expected owner: the *published and transient state* record (`dec:retention:monolithic-snapshot`).

**Decision (Spatial state is keyed by coordinate and depth)** · `dec:representation:spatial-accumulation`

State accumulated over the coordinate space is keyed by a coordinate-and-depth pair. Reads walk depth and exit at the first hit; writes visit every containing layer (`alg:ledger:all-layers-update`), so a read is bounded by depth while a write pays for the hierarchy that makes the read cheap. The root entry is permanent, and deleting an entry neither merges its evidence upward nor passes it down (`def:ledger:root-semantics`). Collection is a bounded periodic sweep rather than lazy work on the access path. Per-axis rows materialise on first reference rather than at axis registration, so an axis nobody reports on costs nothing. Concurrent access to this state is governed by its own invariant (`inv:publication:ledger-concurrency`), and the index the assessment path uses to reach it is fixed by the specification (`def:runtime:ledger-index`).

Expected owner: the *spatial accumulation* record (`dec:memory:coordinate-depth-key`).

**Decision (One owner holds the identity graphs; assessment defers to it)** · `dec:representation:identity-ownership`

A dedicated owner holds every identity graph, and the assessment path never mutates one: it defers its observations, which are drained on the owner's own loop (`alg:publication:identity-draining`). The observation surface accepts a coordinate and nothing else (`alg:keyspace:observation-protocol`), which is what makes the feed-forward direction structural at this layer rather than conventional (`dec:structural:boundary-direction`) — an observation cannot carry a Sentinel reference because the surface has nowhere to put one. Competitive sets are published per dimension under the swap discipline the structural layer fixed (`dec:structural:publication`). Overflow of the observation queue is degradation rather than a hard error, per the layer's failure posture (`dec:structural:failure-posture`). The host's obligations at this boundary are the specification's (`req:keyspace:encoding-contract`).

Expected owner: the *spatial accumulation* record (`dec:memory:coordinate-depth-key`).

**Decision (The two spatial key types stay distinct)** · `dec:representation:key-types-distinct`

The identity layer's key and the outcome accumulator's key are distinct types despite having identical structure. They index different spaces with different lifetimes and different semantics for a missing entry, and the specification tabulates the difference (`tab:ledger:versus-identity`); unifying them would make every routing site accept a key it must not.

The census treated this as a row under the identity layer. It is a choice in its own right because it is the only place the layer deliberately declines a simplification the type system would otherwise permit, and the reason is the whole of its content. Expected owner: the *spatial accumulation* record (`dec:memory:coordinate-depth-key`).

## Unsettled at this layer · `sec:representation:unsettled`

**Entry (Accumulation-layer deferrals)** · `entry:representation:identity-open`

The identity-side deferrals that originally shared this entry are delivered at (`entry:memory:identity-thresholds`), (`entry:memory:audit-metadata`), (`entry:memory:total-importance`), (`entry:memory:decay-view`) and (`entry:memory:warm-start`). Three Ledger health deferrals have joined them: the depth distribution (`entry:memory:depth-histogram`), accumulated cell-set deltas (`entry:memory:cell-deltas`) and immature-cell count (`entry:memory:immature-count`). Materiality has joined them and its register citation is retired at (`entry:assayer:defer-ledger-materiality`). The specification decision it was blocked on — what evidence identifies attenuation-limited traffic — is settled, the Ledger entry now carrying the raw undecayed adverse and eligible counts and a window of recent eligible arrivals, one for each of the theorem's two conditions (`entry:memory:materiality`). The concentration historically recorded here is discharged in full (`tab:assayer:adr-deferral-homes`).

**Entry (Buffer visibility)** · `entry:representation:buffer-open`

The two deferrals that hung off the retention choices are complete and retired: the assessment snapshot carries the take-and-clear eviction count (`entry:assayer:defer-pending-eviction-counter`), and buffer health carries the lifetime eviction ratio and oldest-live-entry age (`entry:assayer:defer-buffer-eviction-visibility`). Both are held by the *published and transient state* record that owns their completed triggers.
