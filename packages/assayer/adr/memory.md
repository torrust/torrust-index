# Memory · `rec:memory:spatial-state-custody`

This record owns state keyed by position: how it is reached, who is allowed to mutate it, when it is collected, and why the two stores that share a shape do not share a type. Both stores accumulate over the coordinate space — one per Sentinel over outcomes, one per dimension over identity — and the questions they raise are the same questions, which is why one record answers them once.

It realises three layer choices: the accumulation of state over the coordinate space (`dec:representation:spatial-accumulation`), the custody of the identity graphs (`dec:representation:identity-ownership`), and the separation of the two key types (`dec:representation:key-types-distinct`).

Nine deferrals ride with it, the largest load in the set. That is a measurement of where the implementation is least finished rather than a defect of the record: these two stores are the ones whose diagnostics were specified furthest ahead of their plumbing. Each deferral below states the gap, the code that proves it is still open, and what would close it.

**Decision (State is keyed by a coordinate-and-depth pair)** · `dec:memory:coordinate-depth-key`

State accumulated over the coordinate space is addressed by a pair: the lower bound of a dyadic interval and the depth at which that interval sits (`src/types.rs`). The key is not the coordinate — it is the interval containing it — so a single key names the whole set of coordinates that fall inside it.

The pair is what makes hierarchy addressable rather than traversable. A tree of linked nodes would answer the same questions by walking pointers; a flat map from interval to state answers them by computing a key, which costs arithmetic instead of indirection and leaves no structure to keep balanced. The depths are fixed by the encoding's question depth (`def:encoding:question-depth`), so the key space is bounded by construction rather than by policy.

**Decision (Reads walk depth to the first hit; writes visit every containing layer)** · `dec:memory:depth-walk`

A read descends from the deepest configured layer and stops at the first interval that both contains the coordinate and has an entry (`src/ledger/routing.rs`). A write does the opposite: it visits every interval containing the coordinate, from the deepest present to the root, and updates each. The write side is the specification's (`alg:ledger:all-layers-update`).

The asymmetry is the decision, and it is deliberate in both directions. The read wants the most specific evidence that exists and has no use for the general evidence above it, so stopping early is not an optimisation but the correct answer. The write cannot stop early, because the coarse layers are what the read falls back to when the fine ones are empty, and a write that skipped them would leave the fallback describing a past that stopped being updated.

**Decision (The root is permanent, and deletion neither merges nor inherits)** · `dec:memory:root-permanence`

The root entry always exists and is never collected (`src/ledger/gc.rs`). When any other entry is deleted, its evidence is not merged into its parent and is not passed down to its children: it is simply gone. The semantics are the specification's (`def:ledger:root-semantics`), and the refusal to transfer state is stated there as well (`just:ledger:no-state-transfer`).

Permanence and non-transfer are one choice, not two. A read that walks depth must terminate somewhere, and the root is what guarantees it terminates with an answer rather than with an absence the caller has to interpret. Transfer on deletion would be the natural-looking alternative and it is the one that corrupts: evidence gathered about a narrow interval, folded upward, becomes evidence apparently gathered about a broad one, and the aggregate silently acquires a confidence nothing measured.

**Decision (Collection is a bounded periodic sweep, not lazy work on the access path)** · `dec:memory:periodic-sweep`

Stale entries are removed by a periodic sweep that is bounded twice: a cap on how many entries are inspected under one read lock, and a cap on how many are deleted under one write lock (`src/ledger/gc.rs`). Nothing is collected on the read or write path. The eligibility rule is the specification's (`alg:ledger:garbage-collection`).

Putting collection on the access path is the cheaper design and the one this record refuses. It makes the cost of an access depend on how much garbage happens to lie in front of it, which is a function of traffic history rather than of the request, so the tail of the latency distribution is set by something the caller cannot see. A sweep with two caps moves that cost off the path entirely and bounds the lock hold time in both phases, at the price of collecting later than a lazy scheme would.

**Decision (Per-axis rows materialise on first reference)** · `dec:memory:lazy-axis-rows`

An entry's per-axis storage is not allocated when an axis is registered. It comes into existence the first time something writes to that axis at that position (`src/ledger/sentinel_ledger.rs`), so an axis that nothing reports on costs nothing anywhere in the store. The memory bound is the specification's (`bound:runtime:ledger-memory`).

Registration is a declaration of what may arrive, not a prediction that it will. Axes are registered per deployment and the coordinate space is large, so eager allocation multiplies the two and charges for every combination that is possible rather than every one that happened. The cost of laziness is a branch on the write path and a store whose footprint cannot be computed from its configuration alone — which is the honest position, since the footprint genuinely depends on traffic.

**Decision (A dedicated owner holds every identity graph)** · `dec:memory:graph-owner`

Identity graphs are owned by one dedicated maintenance thread, and no other path mutates one (`src/identity/maintenance_loop.rs`). The assessment path reads published state and defers anything it wants to record; it never reaches into a graph. The custody itself is the ownership record's (`dec:ownership:graph-custody`).

Single ownership is what makes the graphs' invariants checkable in one place. A graph is a mutable structure with split, create and evict operations whose correctness is a property of their ordering, and admitting a second mutator would turn every one of those invariants into a locking argument. Deferral is the price: an observation made during an assessment is not reflected in that assessment, which is stated rather than hidden and is what the observation protocol was designed around (`alg:keyspace:observation-protocol`).

**Decision (The observation surface accepts a coordinate and nothing else)** · `dec:memory:observation-surface`

The channel by which the assessment path records an observation takes a coordinate and no other argument (`src/identity/observation.rs`). There is no magnitude, no weight, and no way for a caller to say how much the observation should count: the increment is fixed inside the maintenance loop.

The narrowness is the enforcement. The direction of information flow is the ownership record's decision (`dec:ownership:feed-forward`), and a surface that accepted a magnitude would be the exact hole through which an assessment could steer the structure it is being assessed against. Removing the argument makes the influence unwritable rather than merely discouraged, which is the same technique the guarantee rests on everywhere else (`inv:guarantee:feed-forward`).

**Decision (Competitive sets are published per dimension under the swap discipline)** · `dec:memory:competitive-publication`

Each identity dimension publishes its competitive set independently, through the same swap the model publishes under (`src/identity/snapshot.rs`). A dimension whose set has changed installs a new one without waiting for any other dimension, and readers load per request. The discipline is the concurrency record's (`dec:concurrency:index-independence`), and the set is the specification's (`def:keyspace:competitive-set`).

Per-dimension rather than global is the decision here. Dimensions change on unrelated schedules, and one published structure covering all of them would serialise their maintenance behind a single swap — so a busy dimension would delay a quiet one for no reason other than that they were stored together. The draining discipline that makes a deregistration safe under this arrangement is also the specification's (`alg:publication:identity-draining`).

**Decision (Observation-queue overflow degrades rather than fails)** · `dec:memory:overflow-degrades`

The per-dimension observation channel is bounded. When it is full the observation is dropped and a counter is incremented; nothing blocks and nothing returns an error (`src/identity/observation.rs`). The posture is the degradation record's (`dec:degradation:retain-and-flag`), and the queue face of it is stated there (`rem:degradation:queue-face`).

The alternative postures are both worse for this queue specifically. Blocking would put the assessment path behind the maintenance thread and make identity maintenance able to stall assessment — inverting the dependency the deferral exists to create. Failing the call would convert a structural-enrichment miss into a caller-visible error, when what was actually lost is one increment to a statistic that is already an approximation. A counted drop is the only response proportionate to the loss.

**Decision (The two spatial key types stay distinct)** · `dec:memory:key-types-distinct`

The Ledger's key and the identity layer's cell identifier have the same two fields and the same meaning for each, and they are nonetheless separate types (`src/types.rs`, `src/identity/competitive.rs`). Neither converts into the other, and the identity type says in its own definition that mirroring the other's shape is not a reason to be the other. The specification tabulates what separates the two stores (`tab:ledger:versus-identity`).

Identical structure is the argument for merging and it is not sufficient, because the types differ in what an absence means. A key with no Ledger entry means no outcome evidence has been recorded for that interval, and the read walks up to find some. A cell identifier with no identity entry means the interval is not a competitive cell at all, and walking up would be meaningless. Sharing a type would make the two absences interchangeable in a signature, and the mistake would be invisible at every call site where it was made.

**Decision (The owner's two cadences answer to different things)** · `dec:memory:maintenance-cadences`

The graph owner waits on its command channel with a timeout rather than sleeping (`src/identity/maintenance_loop.rs`). A command therefore wakes it at once, and the timeout governs only the work nothing signals. Two figures follow, and they are not one figure twice.

The wake cadence is a tenth of a second, and what it bounds is staleness. An observation is not a signal: the assessment path pushes a coordinate onto a bounded channel (`dec:memory:observation-surface`) and the owner drains that channel with a non-blocking read on its next pass, so a coordinate recorded just after a pass waits for the next one before it reaches a graph. The same pass recomputes each dimension's competitive set and publishes any change (`dec:memory:competitive-publication`), so the figure bounds how far behind traffic both the graphs and the published sets may run. It is the harness of a choice about how often to pay a full pass for possibly no work, and it is a shipped operating point: no measurement here distinguishes a tenth of a second from a twentieth or a half.

The decay cadence is one minute, and it is deliberately not a second statement of the decay timescale. What is applied every minute is not a per-minute rate but the hourly rate re-derived for a minute, so the tabulated hourly figure (`tab:keyspace:decay-rates`) survives the choice of cadence intact (`rem:memory:decay-cadence-composes`). Freed of the timescale, the figure answers only to cost against resolution — often enough that importance ordering tracks elapsed time between passes, rarely enough that the sweep is not a per-pass expense — and is a shipped operating point on that ground.

**Remark (Why the decay cadence is not a decay timescale)** · `rem:memory:decay-cadence-composes`

The conversion is exact in the arithmetic and inexact in the schedule, and both halves matter to a reader who wants to move the cadence.

Exact in the arithmetic: the per-interval attenuation is the hourly rate raised to the interval's share of an hour, so composing sixty of the per-minute factors returns the hourly factor. The package checks this rather than asserting it, and the check also fixes what the hourly figure means in days — importance halves on a fortnight's timescale, not within a day (`claim:identity:the-per-interval-attenuation-composes-to-the-tabulated-hourly-rate-and-a-fortnight-half-life`). So the cadence is free of the timescale: however often the owner wakes, an hour of wall time is one hour of decay.

Inexact in the schedule, and this is the part the arithmetic hides. The factor is computed once from the nominal interval and then applied whenever at least that long has passed, with the clock reset to the moment of application rather than advanced by the interval. Overshoot is therefore not carried forward, and realised decay runs systematically slower than tabulated by whatever fraction of the interval a wake arrives late. The overshoot is bounded by one wake cadence, which is why the two figures cannot be chosen independently: at a tenth of a second inside a minute the loss is under two parts in a thousand, and a cadence brought near the wake interval would make the error a large fraction of the decay itself.

**Remark (What the depth walk trades)** · `rem:memory:read-write-tradeoff`

The two paths pay differently and deliberately. A read is bounded by the configured depth and usually returns well before it, since the deepest entries are the ones traffic creates. A write is bounded by the same depth and always pays all of it, because every containing layer must see the update.

The trade is worth naming because it looks backwards. Reads are far more frequent than writes here, so loading the cost onto writes buys the common case with the rare one — and the hierarchy the write maintains is exactly what lets the read stop early. The costs are not independent: cheapening the write by updating fewer layers would deepen the read, because the layers it skipped are the ones the read would then have to pass through to find anything.

**Entry (Host-configurable identity graph thresholds)** · `entry:memory:identity-thresholds`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-identity-thresholds`). `IdentityBudget` now carries the split threshold, create and evict depths, and hourly spatial decay rate beside the existing graph and queue limits. The shipped constructor preserves the former behaviour: create depth equals the competitive cutoff, eviction stays two levels deeper, the split threshold remains ten, and spatial importance retains its tabulated hourly rate. Every field is public and may be overridden by the host (`claim:identity:registration-copies-host-graph-thresholds-without-deriving-them-downstream`).

The completion trigger is met. Lifecycle construction copies the three Mudlark thresholds directly from the registration budget instead of deriving or fixing them, and the maintenance owner derives its interval attenuation from the registered spatial rate (`claim:identity:the-graph-uses-the-spatial-decay-rate-supplied-at-registration`).

**Entry (Identity dimension audit metadata)** · `entry:memory:audit-metadata`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-identity-audit-metadata`). The public registration and the stored dimension both carry the host's description and coordinate-semantics declaration. Full health reports project both strings beside the dimension name (`claim:identity:dimension-audit-metadata-is-reported-and-does-not-drive-core-behaviour`).

The completion trigger is met. The metadata is read only while assembling the diagnostic health report; graph construction, feature layout, extraction and model behaviour do not consult it.

**Entry (Published identity total importance)** · `entry:memory:total-importance`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-identity-total-importance`). `CompetitiveSetIndex` carries the total graph importance captured with its cells. The maintenance owner republishes when either the set or the total changes and refreshes the scalar after spatial decay (`claim:identity:a-competitive-index-publishes-the-graph-total-beside-its-cells`).

The completion trigger is met. The assessment shared-state implementation reads the scalar from the same per-dimension swap as active-cell routing, and the feature retains the same logarithmic transform. A checkpoint-barrier comparison proves the published scalar equals the graph-total reconstruction it replaces (`claim:identity:published-total-importance-equals-the-checkpoint-reconstruction-it-replaces`).

**Entry (Identity outcome read-time decay view)** · `entry:memory:decay-view`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-identity-decay-view`). `CellOutcomeState::read_decayed` clones the stored state and runs the same private decay operation the write path uses, through the shared elapsed-time decay function. It mutates neither the stored scalars nor their maps or timestamp (`claim:identity:a-decayed-outcome-view-is-exactly-the-mutating-decay-result-without-changing-storage`).

The completion trigger is met. Assessment supplies its one persistent timestamp to every identity outcome read; per-axis features, cross-dimension outcome aggregates and pre-seed synthesis all consume the returned decayed view. The equality test compares it value for value with mutating decay on a clone and separately proves storage unchanged.

**Entry (Competitive-cell outcome warm-start retention)** · `entry:memory:warm-start`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-competitive-warm-start`). Competitive exit now replaces only `MeasurementState` with its zero value. The cell and its outcome state stay in the map, and a later entry finds and reuses them rather than installing a neutral replacement (`claim:identity:competitive-exit-resets-measurement-while-outcome-warm-starts-reentry`).

The completion trigger is met. After graph observation and any resulting eviction, cleanup compares retained keys with the exact terminal and transition intervals in a full graph extraction. Competitive demotion leaves a live interval in that census and retains its outcome; graph eviction removes the interval and its state (`claim:identity:retained-outcome-state-is-cleaned-only-when-its-graph-interval-no-longer-exists`). Focused tests prove the exit/re-entry split and the interval-existence cleanup separately.

**Entry (Ledger depth-tier distribution)** · `entry:memory:depth-histogram`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-ledger-depth-histogram`). Per-Sentinel Ledger health now carries an ordered entry-count histogram keyed by depth beside the existing maximum (`src/health/summary.rs`). Full health scans every current Ledger key under its existing read guard and increments the key's depth bin (`src/api/health.rs`).

The completion trigger is met by that query-time scan. A focused populated-tier test proves the distribution counts every active entry at its own depth rather than inferring a distribution from the maximum (`claim:wellness:ledger-health-counts-every-active-entry-at-its-own-depth`).

**Entry (Ledger cell-set deltas in health)** · `entry:memory:cell-deltas`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-ledger-cell-deltas`). Each Sentinel slot now accumulates created and deleted cell totals after successful cell-set maintenance, and full health reports both process-lifetime counters beside that Sentinel's Ledger state (`src/report/slot.rs`, `src/report/ingestion.rs`, `src/api/health.rs`).

The arguable status is resolved deliberately in favour of the register's narrower reading: a latest-report acknowledgement does not satisfy *accumulated per Sentinel*. The new counters do. A focused sequence of reports proves creations and later deletions survive beyond the latest acknowledgement (`claim:wellness:ledger-cell-set-deltas-accumulate-per-sentinel-across-reports`).

**Entry (Ledger immature cell count)** · `entry:memory:immature-count`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-ledger-immature-count`). Full health evaluates the strict eligible-label threshold from (`def:monitoring:immature-cells`) over every current entry under the Ledger read guard. It reports both the per-Sentinel count and the fleet total (`src/api/health.rs`, `src/health/summary.rs`).

The completion trigger is met by that query-time scan, which now reads the public monitoring configuration rather than a local constant. A focused non-default boundary test proves entries below the threshold count as immature while entries at and above it do not (`test:crate:ledger-health-counts-cells-below-the-materiality-threshold`).

**Entry (Ledger materiality: value realised and attenuation limited)** · `entry:memory:materiality`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-ledger-materiality`). Full health reports both traffic-weighted quantities, per Sentinel and fleet-wide: the realised value and the attenuation-limited share (`src/api/health.rs`, `src/health/summary.rs`).

The deferral was a definition STOP, and what discharges it is licensing the Ledger to store the evidence the theorem's predicate is actually about rather than settling for an approximation of it. The predicate is counterfactual — it asks what a cell's rate would be were the attenuation not applied — and no decayed average can answer a counterfactual about its own decay. Each entry therefore carries two further fields: the raw undecayed adverse count beside the eligible count it already held, and an exponentially time-weighted window of recent eligible arrivals (`tab:ledger:entry-state`). Both are maintained by the all-layers update that already writes the averages and read by the read-time decay that already reads them, so neither a maintenance phase nor any background work was added (`alg:ledger:all-layers-update`), (`def:ledger:time-decay`).

The predicate is then the theorem taken literally. A cell is realising its value when its remembered rate stands more than half a standard deviation above the standardisation mean, which is the theorem's own boundary written as the definition's own formula (`thm:ledger:materiality`), (`def:monitoring:ledger-value`). It is attenuation-limited when it is not realising, its true rate — read off the raw pair — clears that same boundary, and the attenuation its arrival window implies leaves the attenuated rate at or below it. Holding the window as a time-weighted arrival count is what makes the second arm a division rather than a rate estimate: a window settled at $L$ arrivals has $\gamma_{t,L}^{\Delta h} = 1 - 1/L$, so the tabulated attenuation follows directly and reproduces all four of its rows (`data:ledger:attenuation`). An earlier approximation of this predicate, which counted every non-realising immature cell as attenuation-limited and so conflated two conditions the theorem holds independent, is not resurrected.

Storage carries the cost. The Ledger crosses the checkpoint whole and the codec is not self-describing, so the layout generation is bumped and the format policy already in force applies unchanged: a checkpoint written before the evidence existed is refused and the instance cold-starts, its Ledger rebuilt empty and its evidence beginning immature exactly as a newly created cell's does (`alg:ledger:entry-creation`). A field default would have been the wrong instrument, since it would let a short payload decode into entries whose raw counts disagreed with the averages standing beside them.

The completion trigger is met by that query-time scan. A producer-path falsifier places one cell in each of the theorem's four cases — realising; both conditions met; true rate high but arrivals dense; arrivals sparse but true rate ordinary — and proves the traffic weighting and the disjointness of the two shares (`test:crate:ledger-health-materiality-is-traffic-weighted-and-theorem-exact`). A second probes each arm of the predicate from both sides of its own boundary, so neither comparison could be inverted and still pass (`test:crate:ledger-health-materiality-holds-at-both-boundaries`). Focused unit tests hold the evidence itself: that the raw pair counts eligible labels on both sides, that the stored window reproduces the tabulated attenuation at every tabulated rate, and that a window read after silence falls (`claim:ledger:the-raw-materiality-pair-counts-eligible-labels-on-both-sides`), (`claim:ledger:the-stored-arrival-window-reproduces-the-tabulated-steady-state-attenuation`), (`claim:ledger:the-arrival-window-falls-with-elapsed-silence-at-read-time`).
