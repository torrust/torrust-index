# Layer 5: Contracts and Boundaries · `spec:contracts:public-contracts`

This document fixes the conceptual choices of the contracts layer. It is the last of the five layer outlines that supply the middle term of the projection (`dec:assayer:projection-principle`): the specification fixes the concepts, the layer outlines fix the choices those concepts leave open, and the decision-record set is derived from the choices. It is not a summary of records; the record census found the layer documents had only ever been that (`obs:assayer:adr-layers-not-outlines`).

A *conceptual choice* here is a question the specification poses but does not answer, where the answer binds every implementation of the layer and could have gone another way. Each choice below is stated exactly once. This layer needs that discipline most: the census found that every record in the contract series above a thousand lines contradicted *itself*, because each carried recurring trailing tables restating its own body, and when the body was revised the restatement was not (`obs:assayer:adr-self-contradiction`). This outline carries no such table and the records derived from it will not either.

Each choice names, in prose, the record cluster expected to own its decisions (`plan:assayer:decision-record-architecture`). Most of those clusters have since landed, and a choice whose cluster has landed now carries a citation of the landed record's identity decision alongside the naming; a choice whose cluster remains unlanded is still named only. The tracking relation over every landed record is carried once, by the master register (`docs/adrs.md`), and is not repeated here: this outline connects to its records by citing their identity decisions rather than by a tracking table of its own.

## Scope · `sec:contracts:scope`

**Summary (What this layer fixes)** · `summ:contracts:scope`

The contracts layer fixes what the host may ask of the package and what it may never see: the shape of each public surface, what each call promises synchronously, where configuration lives, what may be observed about the package's own health, and where the package's responsibility stops. It exposes the orderings the operational layer fixes (`summ:operational:scope`) and hides everything the representation layer holds (`summ:representation:scope`).

Nine choices, matching the census's nine for this layer (`tab:assayer:adr-disposition-contracts`) with no split and no merge; the divergences at this layer are all about *where* a choice is stated, and four contradictions collapse accordingly.

**Convention (No status, and no restated body)** · `conv:contracts:no-status`

No surface here carries a status field. The census found the corpus's status vocabulary did not separate *decided* from *built*, and eight records disagreed with their own register about their standing (`obs:assayer:adr-status-vocabulary`). Where a choice is decided and not built, the outline says so in a caveat naming the gap, which is a claim both a reader and the linter can check.

## The conceptual choices · `sec:contracts:choices`

**Decision (Assessment takes a batch and cannot fail)** · `dec:contracts:assessment-surface`

The assessment call takes a batch and returns a result per request with no error channel (`req:runtime:assessment-interface`), realising the structural failure posture at the surface (`dec:structural:failure-posture`); degradation and health travel inline on every result (`schema:output:assessment`). Encoding is performed internally and the host supplies raw values, so the host cannot encode inconsistently with the package. An identifier is assigned per assessment and is the label's only handle back to it. Channel policy is *not* an input: it belongs to host-side derivation, on the split fixed at (`dec:operational:core-derivation-split`).

That last clause is a collapse. The census found this contract deciding a channel-free input and then asserting a channel field in its own trailing guarantee table (`obs:assayer:adr-self-contradiction`). There is no channel input; it is stated here and nowhere else. Expected owner: the *host contract* record (`dec:surface:batch-infallible`).

**Decision (Label submission is asynchronous and irreversible at the boundary)** · `dec:contracts:label-surface`

Label submission is asynchronous: the call acknowledges only what is known synchronously (`req:runtime:label-interface`) under the reporting contract the specification fixes (`tab:host:label-reporting`). The pending entry is consumed at the boundary — before journalling and irreversibly — so a resubmission of the same identifier finds nothing, which is what makes the ordering fixed at (`dec:structural:durability`) observable to the host. Anomalous values are sanitised at the boundary rather than rejected, since rejecting them would fail a call the surface has promised cannot fail for data reasons. There is no idempotency, no batch variant, and no completion variant.

The census found this surface reasoning about a not-found error on an expiry one factor of twenty-four away from the configured default (`reg:assayer:adr-contradictions-records`). The expiry is a construction parameter and is stated once as such (`dec:contracts:construction-surface`); this surface carries no figure of its own. Expected owner: the *host contract* record (`dec:surface:batch-infallible`).

**Decision (Report reception acknowledges with maintenance diagnostics)** · `dec:contracts:report-surface`

Reception is synchronous per the ordering fixed at (`dec:operational:report-ingestion`), and it acknowledges with maintenance diagnostics rather than a bare success, so the caller learns what its report cost as well as that it landed. Slots are held in a concurrent map keyed by Sentinel, distinct from the feature index map. Reception touches no model, no identity graph, no cache, and no calibration state — the isolation is what allows it to run concurrently with everything else.

The slot's own shape is the specification's (`def:extraction:slot`) with its offsets (`conv:extraction:offsets`), cited and not reproduced. The census found that structure written out in five records with three different shapes, the register tracking three of the restatements and not the two that conflicted (`reg:assayer:adr-contradictions-records`); a citation is the whole of the remedy. Expected owner: the *host contract* record (`dec:surface:batch-infallible`).

**Decision (Construction validates eagerly and either cold-starts or restores)** · `dec:contracts:construction-surface`

A builder validates the signal schema, the interaction templates, and every parameter eagerly, so a misconfiguration fails at construction rather than at first use (`req:host:construction`). The schema is declared once and fixed thereafter (`req:host:schema-declaration`) and (`req:signal:schema-fixed`); axes are registered through their own declaration (`req:host:axis-registration`). Configuration is Core-only: derivation and Companion configuration are host-owned and the builder does not accept them. Construction either cold-starts or restores, and a structural mismatch on restore cold-starts rather than failing, on the compatibility rule fixed at (`dec:structural:durability`). Threads are spawned at build and named per instance. Pre-seeding is a post-construction, synchronous step (`alg:host:pre-seeding`), not a builder argument.

Every numeric default the surfaces reason about is a parameter of this contract, including the pending-entry expiry (`tab:config:pending-buffer`); no other surface states one. Expected owner: the *host contract* record, landed apart from the four call surfaces as *construction* (`dec:construction:eager-validation`).

**Decision (Lifecycle is six methods with two-phase visibility)** · `dec:contracts:lifecycle-surface`

Lifecycle is six methods, all taking a shared receiver and returning a result, over the operations the specification tabulates (`tab:registry:sentinel-operations`) and (`tab:registry:axis-operations`). Visibility is two-phase: infrastructure becomes visible synchronously on return, the model asynchronously at the next publication (`inv:guarantee:lifecycle-publication`) — the consequence of the serialisation fixed at (`dec:structural:write-serialisation`), against the model-set rule the specification states (`inv:registry:model-set-serialisation`). Identity deregistration blocks until the published snapshot has stopped referencing the dimension, so a reader never holds a snapshot naming a dimension that is gone. A compound batch continues past non-fatal failures and publishes once (`req:registry:compound-events`). Deregistration is destructive by default and hibernating on request (`alg:registry:hibernation`), and what hibernation does and does not preserve is the specification's (`cav:limitation:hibernation`).

Registration creates a slot in its initial state, and that state is fixed here once: the census found two contract records giving the same call opposite initial states (`reg:assayer:adr-contradictions-records`), and this is the call's owner. Expected owner: the *host contract* record, landed as *construction* (`dec:construction:eager-validation`); the hibernation deferral migrated with it and is now retired (`entry:construction:hibernation`).

**Decision (Health publishes independently and is queried in cost tiers)** · `dec:contracts:health-surface`

Health is published independently of the model snapshot, on the split fixed at (`dec:representation:independent-publication`), and reports without ever acting (`inv:monitoring:report-only`). Events are pushed through one bounded channel that drops with a counter on overflow rather than blocking a producer or growing without bound. Queries are tiered by cost, from a cheap summary (`schema:output:health-snapshot`) to a full report, so a caller polling frequently is not forced to pay for a report it does not read. Discrimination metrics are computed once at refit and cached, not recomputed per query. The report type is non-exhaustive, so a field may be added without breaking a host. What health must make visible is the specification's (`inv:guarantee:drift-visibility`) and (`inv:guarantee:discrimination-visibility`).

The marginalisation completion event now ships. Each removal's event-local aggregate diagnostics travel in its lifecycle completion result and through the same bounded health channel described above. The earlier census contradiction (`reg:assayer:adr-contradictions-records`) is retired at (`entry:assayer:defer-marginalise-event`).

**Decision (Metric export is passive)** · `dec:contracts:metrics-surface`

The package renders nothing and depends on no monitoring crate: it exposes a neutral sample type as the intermediate form a host renders from, and export is therefore passive. The catalogue is a compile-time constant rather than a runtime registry, so its contents cannot drift from what the code emits. Cardinality is bounded by construction.

The catalogue's size is not stated here, or anywhere outside the code. The census found it given five different sizes across three documents, matching the code in none of them, with a test asserting only that it fell in a wide band (`reg:assayer:adr-contradictions-register`) — a number restated in six places is a number nobody maintains. What the surface must report per assessment is the specification's (`req:detection:per-channel-reporting`), over the summary it fixes (`tab:monitoring:observability-summary`). Expected owner: the *observability* record (`dec:metrics:passive-export`).

**Decision (Guidance is read-only, bounded, and reveals nothing internal)** · `dec:contracts:guidance-surface`

Guidance is a read-only, infallible, synchronous query (`sig:guidance:interface`) returning three independently ranked categories — risk-informative (`def:guidance:risk-informative`), investigation (`def:guidance:investigation`), and starvation relief (`def:guidance:starvation`) — with duplication across them allowed and recorded rather than removed (`dec:guidance:no-deduplication`). Every scan is explicitly bounded, and an exhausted budget shortens the output rather than failing the call. Guidance exposes no derivation quantity, no model internal, and no Companion state; what it does expose is the exploration signal the specification guarantees (`inv:guarantee:exploration`).

This is the one surface the census found already at the target shape, with no contradiction of any kind (`cav:assayer:adr-short-not-sharp`). Expected owner: the *host contract* record (`dec:surface:batch-infallible`).

**Decision (Companion state is host-owned and reaches no Core structure)** · `dec:contracts:companion-boundary`

Companion state is owned by the host and reaches no Core structure: it is absent from the working copy and from the published snapshot alike (`dec:representation:independent-publication`), which is what makes the independence structural (`inv:guarantee:companion-independence`) rather than asserted, and the boundary is the specification's (`inv:companion:boundary`). Estimation is infallible, returning a prior, an override, or a posterior, over the model fixed at (`dec:numerics:challenge-model`).

Expected owner: the Companion cluster, landed as (`dec:challenge:conjugate-model`); the current surface and the arrangement still awaiting a decision are stated at (`cav:contracts:companion-arrangement`).

## Unsettled at this layer · `sec:contracts:unsettled`

**Caveat (The Companion arrangement remains open)** · `cav:contracts:companion-arrangement`

The current package carries two of the surfaces discussed here. The replacement trait is public at the crate root and the concrete tracker implements it (`req:companion:replacement-trait`). The tracker's clocked health report and its per-channel detail are likewise public and exported, and the specification enumerates their fields (`tab:companion:health`). A Companion metrics mapper does not exist. The metric catalogue instead carries host-emitted challenge names under the Assayer prefix, consistent with Companion state reaching no Core structure (`dec:contracts:companion-boundary`).

Those facts choose no final arrangement. Which of the existing surfaces survive, whether a mapper is added and which prefix owns it remain the open question the register carries at (`entry:assayer:companion-arrangement`). The choice at (`dec:contracts:companion-boundary`) fixes only what is true today — host ownership and infallible estimation — and the arrangement stays open until that entry is decided.

**Entry (Observability deferrals)** · `entry:contracts:observability-open`

No registered deferral remains on the health choice. The three are gone by different routes. Cross-layer feedback latency is delivered, the wire contract having gained the source-observation age its completion trigger asked for, and its register entry is retired (`entry:assayer:defer-cross-layer-latency`); the first of its four stages ships as an explicit lower bound rather than being completed by inference. Per-Sentinel informativeness is delivered and its register entry retired (`entry:assayer:defer-sentinel-informativeness`), while encoding effectiveness is retired as superseded design history rather than delivered, its legacy three-component formula having been replaced by that same informativeness reading (`entry:assayer:defer-encoding-effectiveness`).

The five deferrals that hung off the metrics choice have completed and their register entries are retired: Sentinel coverage (`entry:assayer:defer-metric-sentinel-coverage`), the zero-Sentinel counter (`entry:assayer:defer-metric-zero-sentinel-counter`), signal-cache metrics (`entry:assayer:defer-metric-signal-cache`), positive-class prior metrics (`entry:assayer:defer-metric-p-positive`), and the drift-reset counter (`entry:assayer:defer-metric-drift-resets`).
