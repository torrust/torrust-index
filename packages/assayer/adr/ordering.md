# Ordering · `rec:ordering:assessment-and-label-paths`

This record fixes the three runtime paths as orderings over shared state: how each is factored, what it may not interleave, where it ends, and — once — the guarantee it declines to offer. It realises five layer choices: the assessment path (`dec:operational:assessment-path`), the split between the Core and the derivation (`dec:operational:core-derivation-split`), the label path (`dec:operational:learning-path`), report ingestion (`dec:operational:report-ingestion`) and the refusal of report sequencing (`dec:operational:no-report-sequencing`).

It is the largest of the operational records and carries no enumeration of any kind, for the reason stated at (`conv:ordering:granularity`).

**Decision (Assessment is one function, not a stage pipeline)** · `dec:ordering:assessment-function`

Assessment is one per-request function with factored helpers. It is not a pipeline of independently scheduled stages, and the specification's enumeration of its steps (`alg:runtime:assessment-pipeline`) describes that function rather than decomposing it into components that could be scheduled apart.

The distinction is observable rather than stylistic. A stage boundary is a place where state can be inspected, retried, or reordered, and every such place is a surface that must be specified and kept true. One function has one entry and one exit; the helpers inside it are an arrangement of code and not an arrangement of the system. Its writes are exactly the enumerated ones (`inv:runtime:enumerated-writes`).

**Decision (An assessment scores against the snapshot it acquired before it observed)** · `dec:ordering:score-before-evolve`

The snapshot is acquired before the raw vector is offered to the cold standardisation ramp, and the score is taken against the snapshot already held (`dec:vector:prior-mass-ramp`). The offer mutates nothing and waits for nothing: what an accepted observation may move is a later coordinate snapshot, never the one the offering request is answering from. The single writer that applies it, the swap that makes it visible and the refusal that reports a channel too full to take it are the concurrency record's and are not restated here (`dec:concurrency:single-steward`) (`dec:concurrency:snapshot-swap`) (`dec:concurrency:no-silent-drop`).

The order is the whole of the decision, and what it forbids is the arrangement that looks harmless. An assessment that offered before it acquired, or that reached for the current snapshot again after offering, would score partly against a coordinate system its own observation had moved — and the distortion would be largest at the smallest counts, where a single observation retires the most prior mass. Fixing the acquisition ahead of the offer makes the answer a function of a coordinate system that existed before the request did, which is what lets a result be reproduced against a named snapshot rather than against a moment.

One accepted observation advances the ramp once and publishes once, so a reader counting published coordinate states is counting accepted observations and not requests. The two counts come apart whenever an observation is refused, and the refusal travels back to the caller rather than being absorbed, which is what keeps the gap between them a fact the host is told instead of one it would have to infer.

**Decision (A batch shares one timestamp)** · `dec:ordering:batch-timestamp`

A batch is processed sequentially and every request in it is answered against one timestamp captured at the batch's head (`src/api/assess.rs`). Two requests in one batch therefore cannot disagree about the present.

This does not make the batch a unit in any other respect. The snapshot is still loaded per request, so a publication landing mid-batch is visible to the remainder of it (`dec:concurrency:per-request-load`): the batch shares a clock and does not share a view. Those two are separable and are deliberately decided apart, because time is a property of the request's meaning while freshness is a property of the answer's quality.

**Decision (The Core's work ends at the risk assessment)** · `dec:ordering:core-boundary`

The Core produces a risk assessment and stops. Turning that assessment into a recommendation is a separate step the host invokes explicitly, on its own schedule and with its own configuration; nothing in the Core calls it and nothing in the Core depends on its having been called.

The boundary is where the package's knowledge ends and the host's economics begin. What action a given risk warrants depends on what the host is willing to lose, which the package is not told and should not guess. The alternative — one call returning a recommendation — is discussed below (`disc:ordering:single-call-alternative`).

**Decision (Only the risk sufficient statistic crosses into derivation)** · `dec:ordering:closed-crossing`

The crossing carries the risk sufficient statistic (`schema:risk:basis`) and nothing else: not the model, not the feature vector, not identity state, not the pending entry. The receiving signature is fixed by the specification (`sig:landscape:derivation-function`).

A closed input set is what makes the derivation pure by construction rather than by discipline (`inv:guarantee:derivation-purity`). A derivation handed the model could be pure and would be pure only for as long as nobody reached; a derivation handed a statistic has nothing to reach for. The far side of this crossing is not this record's, on the reasoning at (`cav:ordering:derivation-half`).

**Decision (Assessment moves observational geometry and no outcome-learned state)** · `dec:ordering:evidence-authority`

What separates the two paths is not that one writes and the other does not, but which authority a write answers to. An assessment may advance the bookkeeping the specification enumerates (`inv:runtime:enumerated-writes`) and the observational measurement its two acquisition mechanisms take (`dec:vector:two-acquisitions`), and it advances nothing that an outcome taught. Model parameters, calibration, outcome memory and the continuing standardisation average move on the label path and nowhere else.

Stated as evidence authority rather than as an absence of residue is the decision, and the wording is the substance of it. *Repeated assessment leaves no residue* is not retained and is not true: an accepted cold observation is residue, deliberately so, and the ramp exists in order to leave it (`dec:vector:prior-mass-ramp`). A promise that broad would be falsified by the mechanism the corpus just adopted, and a promise falsified by a mechanism is worse than the narrower promise it was standing in for.

Three narrower truths survive it and they are separately checkable. Derivation of one held assessment is deterministic and reads no state, which is the far side's to state and is reaffirmed rather than touched here (`dec:derivation:pure-transform`). No assessment advances outcome-learned state, which is this decision. And an accepted observation reaches only a later coordinate snapshot, disclosed with the phase and count that position it (`dec:ordering:score-before-evolve`). What none of the three promises is that two assessments of the same subject in different coordinate states return the same risk value — they never did, and the discarded wording was the only sentence in the corpus that could be read as promising it.

**Decision (The label path is one function in ordered groups)** · `dec:ordering:label-function`

The label path is one function executed sequentially in ordered groups, serving the specification's label interface (`req:runtime:label-interface`) within its stated cost budget (`bound:runtime:label-cost`). The groups are an ordering, not a set of stages: no group is separately schedulable and none may be skipped or reordered by a caller.

What the ordering buys is that each group may assume the previous groups have completed. The alternative is defensive re-derivation inside each group, which is the same computation performed several times against the possibility that it was not performed before.

**Decision (Time decay is applied once, at the head of the path)** · `dec:ordering:decay-at-head`

Elapsed time decay is applied once, before any model update. No update is therefore applied to a model in a partially decayed state, and no quantity is decayed twice on one label. The temporal discipline this rests on is the layer's (`dec:operational:temporal-governance`), which fixes what a decay is and where it may be computed.

Position is the whole decision. Decay applied after the updates would decay the evidence the label just added; decay applied between groups would decay some models and not others on the same label, and the difference would depend on the order the groups happen to run in.

**Decision (The model snapshot and health summary both publish at the end)** · `dec:ordering:publish-at-end`

Both publications happen at the end of the path and neither happens partway. A reader never observes a model that has been updated alongside health that has not, and never the reverse.

The two are independent installations — the retention record decides that they are separate swaps (`dec:retention:health-independent`) — and this record decides that both fall at the end of the same path. Independence is about what waits for what; position is about what a reader can catch mid-flight, and the answer here is nothing.

**Decision (Reception is synchronous and serialised per Sentinel)** · `dec:ordering:synchronous-reception`

Report reception is synchronous: the caller learns the outcome on return rather than being told that the report was accepted for later processing. It is serialised per Sentinel, so two reports from one Sentinel cannot interleave while reports from different Sentinels proceed independently (`req:publication:report-reception`).

Per-Sentinel rather than global is the choice worth naming. A global lock would have been simpler and would have made every Sentinel's reporting rate a function of every other's; the serialisation is placed exactly where the state is shared and nowhere wider.

**Decision (Validation is two-phase)** · `dec:ordering:two-phase-validation`

Validation runs in two phases with different consequences. A structural failure rejects the report; a data-quality problem degrades the affected cell, is reported, and lets the rest of the report proceed (`pre:architecture:report-validation`).

The partition is not this record's to justify — it is the degradation record's (`dec:degradation:error-partition`) — and what this record decides is that validation is ordered into two passes rather than one mixed check. Structure is established first because a data-quality judgment about a cell presupposes that the cell exists.

**Decision (Maintenance completes before the new index becomes visible)** · `dec:ordering:maintenance-first`

Accumulator maintenance completes, and only then is the new index installed (`def:runtime:report-index`). A reader that observes a new index observes one whose backing state is already fully updated.

The ordering is the guarantee. Installing the index first and maintaining afterwards would be faster to acknowledge and would open a window in which the index is visible and its backing state is half-updated — a state no reader can detect and every reader would misread, since an index is trusted precisely because it is supposed to summarise something finished.

**Decision (Lock ordering is fixed and total)** · `dec:ordering:lock-order`

Reception is acquired before the accumulator, never the reverse (`src/report/ingestion.rs`). The order is fixed, total over the locks this path takes, and stated once here.

A total order is the whole of deadlock freedom for this path, and it is worth more as a stated rule than as an observed fact: the current code takes the two in this order, and what keeps a future path from taking them in the other is that the order is written down and cited rather than inferred by reading both call sites.

**Decision (Reports carry no sequence; the last write wins)** · `dec:ordering:no-sequencing`

Reports carry no sequence number and are not ordered against one another. The last write wins, and a report arriving out of order loses — it is not detected, not rejected, and not merged.

This is the record's one deliberate refusal of a guarantee. Sequencing was available and declined, because declining it is what keeps reception cheap enough to be synchronous. Staleness becomes a monitoring concern instead: the specification defines the measure (`def:extraction:report-staleness`) and states what the gap between reports costs (`cav:limitation:inter-report`).

**Convention (This record fixes orderings, never enumerations)** · `conv:ordering:granularity`

Where the specification enumerates a path's steps, this record cites the enumeration and does not restate it. What is decided here is the shape of an ordering — how it is factored, what it may not interleave, and where it ends. The steps themselves define what the system is rather than recording a choice about it, and the census sent them to the specification on that ground; the layer states the same rule (`conv:operational:path-granularity`).

The rule has teeth in this record specifically. The three enumerations it would otherwise carry are among the longest in the corpus, and transcribing them is precisely how the legacy records grew to contradict the specification they were transcribing from.

**Discussion (The rejected staged pipeline)** · `disc:ordering:staged-alternative`

The alternative was a pipeline of independently scheduled stages. It buys parallelism the cost budget does not need — the assessment path is already within its bound — and it pays at every stage boundary, each of which becomes a place where intermediate state is observable and must therefore be specified, versioned, and kept consistent with the stage on either side.

The staging would also have to be undone at the boundaries the record already fixes: a batch sharing one timestamp and a path publishing only at its end are both statements about a path that runs as a unit.

**Discussion (The rejected lazy per-reader decay)** · `disc:ordering:lazy-decay-alternative`

The alternative was decay applied lazily, by each reader, at the moment it reads. It spreads a cost that the head of the label path currently concentrates, and it makes the value a reader gets depend on when the reader happened to look.

That dependence is worse than the cost. Two readers of the same state at different instants would legitimately obtain different numbers, so any comparison between them would be meaningless without also comparing read times, and the replay of a recorded run would reproduce its inputs but not its outputs.

**Discussion (The rejected single call returning a recommendation)** · `disc:ordering:single-call-alternative`

The alternative was one call that assessed and recommended together. It is a smaller interface and a shorter integration, and it imports the host's economics into the package: the call would need channel policy as an input, and the package would be deciding what a given risk is worth.

It also dissolves the closed crossing of (`dec:ordering:closed-crossing`). With the derivation inside the call there is no boundary for a sufficient statistic to be the only thing to cross, and the purity that boundary establishes becomes a property nothing enforces.

**Caveat (This record fixes only the Core's side of the split)** · `cav:ordering:derivation-half`

The crossing has two sides and this record owns one. What the Core sends, and that it sends nothing else, is decided here; what the far side does with it — the transform it is, what may be added to what it receives, and the allowlist that keeps it out of Core state — belongs to the derivation record (`dec:derivation:pure-transform`).

The division is named rather than papered over. A reader looking for the derivation's own decisions will not find them here, and should not read this record's silence about them as a claim that the Core's side is the whole of the split.
