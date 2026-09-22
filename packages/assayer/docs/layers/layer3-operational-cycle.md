# Layer 3: The Operational Cycle · `spec:operational:assessment-and-learning-cycle`

This document fixes the conceptual choices of the operational layer. It is one of the five layer outlines that supply the middle term of the projection (`dec:assayer:projection-principle`): the specification fixes the concepts, the layer outlines fix the choices those concepts leave open, and the decision-record set is derived from the choices. It is not a summary of records; the record census found the layer documents had only ever been that (`obs:assayer:adr-layers-not-outlines`).

A *conceptual choice* here is a question the specification poses but does not answer, where the answer binds every implementation of the layer and could have gone another way. Each choice below is stated exactly once, which is the remedy the census prescribed for a corpus whose every contradiction was a drift between copies (`obs:assayer:adr-contradiction-mechanism`).

Each choice names, in prose, the record cluster expected to own its decisions (`plan:assayer:decision-record-architecture`). Most of those clusters have since landed, and a choice whose cluster has landed now carries a citation of the landed record's identity decision alongside the naming; a choice whose cluster remains unlanded is still named only. The tracking relation over every landed record is carried once, by the master register (`docs/adrs.md`), and is not repeated here: this outline connects to its records by citing their identity decisions rather than by a tracking table of its own.

## Scope · `sec:operational:scope`

**Summary (What this layer fixes)** · `summ:operational:scope`

The operational layer fixes the orderings: what happens on each of the three runtime paths, in what sequence, over the structures the representation layer supplies (`summ:representation:scope`) and under the serialisation the structural layer fixed (`summ:structural:scope`). It also fixes what time means to the system, how the feature vector is assembled, how its statistics are acquired, and what the system's own convergence is allowed to gate.

Nine choices. The census derived seven for this layer (`tab:assayer:adr-disposition-cycle`); this outline splits two of them, on the reasoning recorded at each.

**Convention (A path is a choice about order, not about steps)** · `conv:operational:path-granularity`

Where the specification enumerates a path's steps, this layer does not restate the enumeration. The census classified those enumerations as material defining what the system is rather than choices about it, and directed them to the specification (`data:assayer:adr-disposition-totals`). What this layer fixes is the shape of the ordering — how it is factored, what it may not interleave, and where it ends — and the enumeration is cited.

## The conceptual choices · `sec:operational:choices`

**Decision (Assessment is one function, not a stage pipeline)** · `dec:operational:assessment-path`

The assessment path is one per-request function with factored helpers rather than a pipeline of independently scheduled stages, and the specification's enumeration of its steps (`alg:runtime:assessment-pipeline`) is a description of that function rather than a decomposition into components. A batch is processed sequentially, sharing one timestamp across the whole batch, so two requests in one batch cannot disagree about the present. The path's writes are exactly those the specification enumerates (`inv:runtime:enumerated-writes`), and the interface it serves is the specification's (`req:runtime:assessment-interface`).

The alternative was a staged pipeline with per-stage scheduling, which buys parallelism the specification's cost budget does not need and pays for it with a visibility surface at every stage boundary. Expected owner: the *runtime paths* record (`dec:ordering:assessment-function`).

**Decision (The Core path ends at the risk assessment)** · `dec:operational:core-derivation-split`

The Core's work ends at the risk assessment; turning that assessment into a recommendation is an explicit, separate step the host invokes. The only Core information crossing into it is the risk sufficient statistic (`schema:risk:basis`) — not the model, not the features, not the identity state — and the receiving function's signature is fixed by the specification (`sig:landscape:derivation-function`). What the crossing buys is a derivation that is pure by construction (`inv:guarantee:derivation-purity`) rather than by discipline.

The alternative was a single call returning a recommendation, which would have made channel policy an input to the Core and put the host's economics inside the package. Expected owner: the *runtime paths* record (`dec:ordering:assessment-function`), with the derivation-side decisions held by the *derivation* record (`dec:derivation:pure-transform`).

**Decision (The label path applies decay once, at its head)** · `dec:operational:learning-path`

The label path is one function executed sequentially in ordered groups (`alg:runtime:update-path`), serving the specification's label interface (`req:runtime:label-interface`) within its cost budget (`bound:runtime:label-cost`). Time decay is applied once at the head of the path, before any model update, so no update is applied to a model in a partially decayed state and no quantity is decayed twice. Both the model snapshot and the health summary publish at the end of the path, not partway through it, so a reader never observes a model that has been updated but not yet published alongside health that has.

The alternative was decay applied lazily at each reader, which spreads the cost but makes the decayed value depend on when it is read. Expected owner: the *runtime paths* record (`dec:ordering:assessment-function`).

**Decision (Report reception is synchronous and serialised per Sentinel)** · `dec:operational:report-ingestion`

Reception is synchronous — the caller learns the outcome on return — and serialised per Sentinel by a reception lock, so two reports from one Sentinel cannot interleave while reports from different Sentinels proceed independently (`alg:runtime:report-reception`) and (`req:publication:report-reception`). Validation is two-phase under the layer's failure posture (`dec:structural:failure-posture`): a structural failure rejects the report, a data-quality problem degrades the affected cell and is reported (`pre:architecture:report-validation`). Accumulator maintenance completes before the new index becomes visible, so a reader never sees an index whose backing state is half-updated. Lock ordering is fixed and total: reception is acquired before the accumulator, never the reverse.

Expected owner: the *runtime paths* record (`dec:ordering:assessment-function`).

**Decision (Reports carry no sequence; the last write wins)** · `dec:operational:no-report-sequencing`

Reports carry no sequence number and are not ordered against each other. The last write wins, and a report that arrives out of order simply loses. Staleness is therefore a monitoring concern rather than an ingestion concern: the specification defines the measure (`def:extraction:report-staleness`) and states what the gap between reports costs (`cav:limitation:inter-report`).

The census grouped this under report ingestion. It is separated here because it is the layer's one deliberate refusal of a guarantee — sequencing was available and declined — and because the declining is what makes reception cheap enough to be synchronous. Expected owner: the *runtime paths* record (`dec:ordering:assessment-function`).

**Decision (Two timestamp domains, separated by type)** · `dec:operational:temporal-governance`

The system carries two timestamp domains — one intra-process, one persistent — and they are separated by type so that a value from one cannot be used where the other is required. Decay is applied lazily at the point of use (`alg:temporal:lazy-application`) over the mechanisms the specification distinguishes (`def:temporal:two-mechanisms`) and the inventory it fixes (`tab:temporal:decay-inventory`). The clamp on elapsed time is embedded in the timestamp interface itself, so a caller cannot compute an unclamped interval; and all decay flows through a small set of shared functions, with direct exponentiation prohibited at every other site.

The alternative — one timestamp type with a convention about which values are persistable — is the arrangement the type separation exists to prevent, and the prohibition on direct exponentiation is what keeps the clamp from being bypassable by arithmetic. This choice named two expected owners — the *durability and recovery* record for the persistence half, the *features and standardisation* record for the decay inventory's model-side rates — and was given one of its own instead (`dec:clock:two-domains`), on the reasoning at (`dec:assayer:record-clock-record`).

**Decision (One canonical block order and one resolver)** · `dec:operational:feature-composition`

The feature vector has one canonical block order, fixed and total (`def:feature:vector-structure`), and a single dimension map is the sole resolver of every feature index — no site computes an offset independently. The map is covering (`inv:dimension:covering`), contiguous per block (`inv:dimension:contiguity`), and versioned so a stale consumer is detected rather than misled (`inv:dimension:version-consistency`). Interaction templates name their operands semantically rather than by index and are compiled at lifecycle rebuild (`alg:dimension:compilation-pipeline`), so a registration changes what the templates resolve to without changing what they mean. Interactions are computed from unstandardised bases and standardised once afterwards, never composed from already-standardised operands. Label-time assembly freezes the blocks whose values were fixed at assessment and re-derives those that were not.

Expected owner: the *features and standardisation* record (`dec:vector:block-order`).

**Decision (Statistics are observed at assessment, updated at label, never decayed)** · `dec:operational:standardisation`

Standardisation statistics are observed at assessment time and updated at label time (`req:standardisation:timing`), so the statistic a feature is standardised against is the one current when it was observed. They carry no time-indexed decay at all — the same choice the specification makes for the class-rate trackers and for the same reason (`dec:weighting:no-time-decay`). Two acquisition mechanisms exist and no third: a batch initialisation for the system, and a per-Sentinel bootstrap for a Sentinel joining later (`alg:standardisation:sentinel-bootstrap`). Accumulators are transient and deliberately not checkpointed. Feature classes carry per-class priors derived from the dimension map rather than declared (`req:standardisation:class-assignment`).

The consequence the specification records is that a Sentinel added late standardises against a moving target (`cav:limitation:late-standardisation`); the bootstrap bounds that cost rather than removing it. Expected owner: the *features and standardisation* record (`dec:vector:block-order`).

**Decision (Convergence is diagnostic and gates nothing)** · `dec:operational:convergence-diagnostic`

Convergence is reported, never enforced: no path is blocked, no output is withheld, and no behaviour changes because a tracker has or has not converged. Each process carries its own concrete tracker — there is no shared trait and no global state machine — and the composite stage is computed on demand from them, so it may regress after a lifecycle change rather than ratcheting. Early refit is triggered unconditionally by any registration or deregistration.

The unconditional trigger is a collapse of the census's clearest supersession defect: an earlier conditional check was replaced in prose by two later records and generalised by a third, none of which ever stated the superseded condition in full, so the corpus held a supersession whose object could not be read (`reg:assayer:adr-contradictions-records`). There is one trigger, it is unconditional, and it is stated here. What a reader may trust before the first refit is the specification's (`rem:warmup:pre-calibration`), over the stages (`tab:warmup:stages`) and milestones (`tab:warmup:milestones`) it fixes.

Expected owner: the *calibration and blending* record for the refit trigger (`dec:calibration:per-regime-search`), the *observability* record for the trackers' reporting (`dec:health:independent-publication`).

## Unsettled at this layer · `sec:operational:unsettled`

**Entry (Cross-layer latency)** · `entry:operational:latency-open`

Settled. End-to-end feedback latency across the three paths is defined by the specification and now surfaced: the wire contract carries a source-observation age, so the first stage is measurable as a lower bound and the remaining three are differences on the engine's own monotonic clock. The deferral was registered (`entry:assayer:defer-cross-layer-latency`), migrated into the *observability* record, and is completed there (`entry:health:cross-layer-latency`).
