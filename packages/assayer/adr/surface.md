# Surface · `rec:surface:host-facing-contracts`

This record owns the four call surfaces the host uses at runtime: what each one takes, what each promises by the time it returns, and what none of them will ever show. It realises the assessment choice (`dec:contracts:assessment-surface`), the label choice (`dec:contracts:label-surface`), the report choice (`dec:contracts:report-surface`), and the guidance choice (`dec:contracts:guidance-surface`).

It is the widest record in the set, and it carries no status field and no trailing table restating its own body. That absence is the point rather than an omission: the restating table is the mechanism behind every self-contradiction the census found in the legacy contract series (`obs:assayer:adr-self-contradiction`).

**Decision (The assessment call takes a batch and cannot fail)** · `dec:surface:batch-infallible`

The assessment call accepts a batch of requests and returns one result per request, positionally aligned, with no error channel and no per-request error type on the Core path (`req:runtime:assessment-interface`).

The infallibility is not this record's to argue: it is the failure posture arriving at the surface (`dec:degradation:infallible-core`). What this record adds is the shape that posture forces. A call that cannot fail must answer every request it was handed, so the return is positional rather than keyed and an empty batch returns an empty result rather than a complaint. Anomalies therefore have nowhere to go except onto the result they belong to, which is what the next decision but one is about.

**Decision (Channel policy is not an input)** · `dec:surface:no-channel-input`

The request carries no channel field and the call performs no channel validation. Channel policy belongs to host-side derivation, on the boundary the operational layer fixes (`dec:ordering:core-boundary`), and it is one of the three arguments the far side receives (`rule:derivation:closed-inputs`); a host wanting tags for several channels calls the derivation once per channel against one assessment.

Stating it in exactly one place is the whole of the decision. The legacy assessment contract decided a channel-free input in its body and then asserted a channel field in a result back-reference, in a numbered consequence, and in its own closing verification table — the last of which certified a mechanism the body forbids. The collapse is recorded with its evidence at (`reg:surface:collapsed-contradictions`).

**Decision (Degradation and health travel inline on every result)** · `dec:surface:inline-health`

Every result carries a compact health summary including its own degradation, present whether or not anything degraded (`schema:output:assessment`). A summary reading all zeros is the positive statement that the result was computed without a fallback, not the absence of information.

Travelling in band is what makes the missing error channel affordable, and that consequence is already owned (`cor:degradation:in-band-travel`). This record decides the narrower thing: the inline summary is a distinct thing from the on-demand query rather than a prefix of it. They answer different questions — what this result cost, against what the package's condition is — and the second belongs to the health surface (`dec:contracts:health-surface`).

The summary stays system-wide, which is why the borrowed share is not in it. That share is a property of one result — of the models read along one request's own features — and the summary's whole shape is that every request in a batch reads the same one, so a per-result quantity placed here would be the first field of it a reader could not compare across two results side by side. It rides on the risk basis instead, beside the uncertainty it tempered (`dec:risk:evidence-only-uncertainty`), on the same division that already puts per-request Sentinel coverage there and system-wide coverage here.

**Decision (Encoding is internal and the host supplies raw values)** · `dec:surface:internal-encoding`

The host passes raw domain values and the package applies the declared shape's encoding itself (`src/api/assess.rs`). Hashing, transformation and normalisation all happen inside; no pre-encoded feature crosses the boundary.

The alternative splits one piece of logic across the boundary. If the host encoded, the encoding rule would live in two codebases that must agree forever, and the schema — which the package validates once at construction (`cav:construction:schema-fixity`) — would describe something the package no longer controls. Keeping encoding inside means a host cannot encode inconsistently with the package, because there is no host-side encoding to be inconsistent with.

**Decision (An identifier is assigned per assessment)** · `dec:surface:assessment-identifier`

Every request reaching the assessment path is assigned an identifier as its pending entry is inserted, and that identifier is the label's only handle back to the assessment (`src/api/assess.rs`).

One handle rather than several is what keeps the label surface narrow. A label identified by its subject rather than by its assessment would have to be matched against whatever assessments that subject accumulated, which is a policy the package would have to invent and the host would have to guess. The entry the identifier reaches is the retention record's (`dec:retention:pending-map`), and what its loss costs is stated there rather than promised away here.

**Decision (Label submission is asynchronous)** · `dec:surface:async-label`

Label submission returns as soon as the label is accepted into the pipeline, and acknowledges only what is known synchronously (`req:runtime:label-interface`), under the reporting contract the specification fixes (`tab:host:label-reporting`). The model update happens afterwards, on the steward.

Acceptance is therefore a claim about the boundary and not about the model: it says the entry was found and consumed, and the work enqueued. It does not say any parameter moved. The distinction is worth stating plainly because an acknowledgement that looked like a completion would make every host that waited on it correct by accident and every host that did not wait wrong for no reason.

The surface carries one state that is about the model rather than about the label, and it is permanent. When a numeric checkpoint judges the working copy corrupt, reverts the label, and then cannot rebuild the working copy from the last published snapshot, the label path stops: no further label is applied, and every submission from then on is refused with the recorded cause — which model's reconstruction was refused, at which pivot, and how far into the label stream it happened. Stopping is the only disposition that neither invents state nor conceals its loss. Continuing on the state the checkpoint has just judged corrupt would train the models on a posterior the engine has already disowned; falling back to a fresh prior would discard everything learned without saying so, and would answer the next assessment from a model that had silently forgotten; freezing without a signal would leave a host reading a healthy-looking surface while nothing it submitted had any effect.

What does not stop is everything that does not learn. Assessments go on answering from the last published snapshot, which is a state that was good, and the command path goes on serving lifecycle, checkpoint and shutdown, so a host can take a checkpoint and restart, or wind the instance down, rather than having to kill the process. The refusal is taken before the pending entry is consumed and before anything is journalled, which is deliberate: it costs the submission nothing beyond the entry's own expiry, and it stops the journal growing with labels that would replay into an engine which will not apply them. Journalled labels written before the stop replay under the recovery the durability record defines (`dec:durability:checkpoint-journal`); replay is deterministic, so a journal replayed whole onto the checkpoint it was written against reaches the state that stopped the path and stops it again, and the exits are an earlier checkpoint or a restart that does not replay the suffix past the recorded label count. Which exit to take is the host's, and this record does not choose for it.

A host that never submits another label learns the same thing from the health surface, which carries the stop as a severe flag on the compact tier and the cause whole on the report (`dec:health:tiered-queries`).

**Decision (The pending entry is consumed at the boundary)** · `dec:surface:consume-at-boundary`

The pending entry is removed at the boundary, before any journalling and irreversibly (`src/api/label.rs`). A second submission of the same identifier finds nothing.

One refusal is taken ahead of the removal and is the only one: a submission to a stopped label path (`dec:surface:async-label`). It is not an exception to what this record decides, because a submission refused there proceeds past nothing — what the removal serialises is the submissions that go on to be journalled and enqueued, and after the stop there are none.

Consuming first is what makes the ordering observable to the host: the removal is the serialisation point, so exactly one submission per identifier can proceed past it, and idempotency is a consequence of consumption rather than a feature built on top of it. The order this sits in — sanitise, then consume, then journal — is the durability record's and is stated once there (`conv:durability:sanitise-before-journal`). The cost is real and is not hidden: a failure after consumption loses the label.

**Decision (Anomalous values are sanitised, not rejected)** · `dec:surface:sanitise-not-reject`

Non-finite and out-of-range values are sanitised at the boundary and counted, rather than rejected (`src/api/label.rs`). A report carrying a bad cell is accepted with that cell degraded and the remainder intact.

Rejection is unavailable rather than merely unattractive. The assessment surface has promised it cannot fail for data reasons, and a label surface that rejected on data would make the pair inconsistent about what a bad value means. The partition that decides which failures are hard is the degradation record's (`dec:degradation:error-partition`); what this record fixes is that the boundary is where sanitisation happens, so no computation downstream needs its own guard.

**Decision (There is no idempotency, no batch variant, and no completion variant)** · `dec:surface:no-variants`

The label surface has one shape. There is no idempotent resubmission, no batch submission, and no variant that waits for the update to complete.

Each absence is a refusal with the same shape. Idempotency would require distinguishing a consumed identifier from an evicted or invented one, which the consumption of (`dec:surface:consume-at-boundary`) deliberately destroys. A batch variant would import transaction semantics into journalling without shortening end-to-end latency, since labels arrive singly. A completion variant would contradict the asynchrony the surface is built on. Three variants declined for three reasons, none of them tidiness.

**Decision (What the host did and what a derivation proposes are distinct types)** · `dec:surface:distinct-action-types`

The action a label reports and the tag a derivation produces are different types, not one type used twice (`src/api/label.rs`).

They are different because they answer to different authorities. A derivation proposes from the package's evidence; a host acts on its own, and the specification is explicit that it may act outside anything proposed. Collapsing them into one vocabulary would make the label path unable to represent the case the system most needs to learn from — the action nobody suggested — and would quietly convert a record of what happened into a record of what was recommended. The learning consumes the first.

**Decision (Reception acknowledges with maintenance diagnostics)** · `dec:surface:diagnostic-ack`

Report reception returns what the report cost: how many cells it carried, how many the accumulator created and removed, and how many degraded (`src/api/report.rs`). It does not return a bare success.

The diagnostics are affordable precisely because reception is synchronous (`dec:ordering:synchronous-reception`) — every piece of the work is finished when the call returns, so reporting it is a read rather than a promise. A bare acknowledgement would force a caller wanting the same facts into a health query, which is a second call answering about the package's condition rather than about this report, and would arrive after other reports had moved the numbers.

**Decision (Slots are held in a concurrent map keyed by Sentinel)** · `dec:surface:slot-map`

Per-Sentinel slots live in a concurrent map keyed by Sentinel, which is a different structure from the map resolving feature indices (`src/report/slot.rs`). The slot's own contents are the specification's (`def:extraction:slot`), with its offsets (`conv:extraction:offsets`), cited and not reproduced here.

Keeping the two maps apart is the decision. They have different access patterns and different ordering needs — feature assembly requires a deterministic order and reception does not — and merging them would impose the stricter requirement on the hotter structure. Not reproducing the slot is the other half: the census found that structure written out in five records with three different shapes.

**Decision (Reception touches no model, graph, cache, or calibration state)** · `dec:surface:reception-isolation`

Reception validates, builds an index, maintains the accumulator's cell set, and installs the index (`src/report/ingestion.rs`). It reads and writes no model parameter, no identity graph, no signal cache, no calibration state, and no pending entry.

The isolation is what lets reception run concurrently with everything else, so it is a structural claim rather than a description of current behaviour: anything reception touched would become a thing the assessment path could contend with. It is also what makes the per-Sentinel serialisation sufficient rather than merely convenient — a lock that covered shared state would have to be wider, and the ordering record fixes exactly how wide (`dec:ordering:lock-order`).

**Decision (Guidance is a read-only, infallible, synchronous query)** · `dec:surface:read-only-guidance`

Guidance answers from published state and returns without an error channel (`sig:guidance:interface`). It sends no command to the steward, takes no part in the assessment path, and sanitises implausible parameters rather than refusing them (`src/api/guidance.rs`).

Read-only is what makes it safe to call opportunistically, which is the only way a host can use it: a guidance call that could block on the steward would be a guidance call no latency-sensitive host would make, and one that could fail would need a policy for what to do instead. The surface is deliberately the least privileged of the four, and the census found it already at the target shape.

**Decision (Three independently ranked categories)** · `dec:surface:three-categories`

Guidance returns three categories ranked independently and truncated independently — risk-informative (`def:guidance:risk-informative`), investigation (`def:guidance:investigation`), and starvation relief (`def:guidance:starvation`). A candidate may appear in more than one, and where it does the duplication is recorded rather than removed (`dec:guidance:no-deduplication`).

Independence is the decision and duplication is its honest consequence. A single merged ranking would need weights between three incommensurable reasons for wanting a label, and inventing those weights is exactly the host economics the Core declines elsewhere. Recording the overlap instead of suppressing it lets a host see that a candidate is wanted for several reasons, which is information a deduplicated list destroys.

**Decision (Every scan is bounded and an exhausted budget shortens the output)** · `dec:surface:bounded-scans`

Every scan guidance performs is explicitly bounded, and so is every lock it takes (`src/guidance/mod.rs`). When a budget is exhausted the result is a shorter list; an empty list is a valid answer.

Shortening rather than failing is what keeps the surface infallible under load, and it follows from the same reasoning as the bounds themselves. A guidance call whose cost scaled with the pending population would become least answerable exactly when the system was busiest. Because the output is a ranking and not a set, a truncated answer is still a correct answer to a smaller question — which is why this surface can degrade by length where others degrade by value.

**Decision (Guidance exposes nothing internal)** · `dec:surface:guidance-opacity`

Guidance exposes no derivation quantity, no model internal, and no Companion state. What it does expose is the Core half of the exploration signal the specification guarantees (`inv:guarantee:exploration`) and the identity of what is pending; the half this surface withholds is owned on the far side (`dec:derivation:exploration-layer`).

The opacity is enforced structurally rather than by review: the guidance module sits inside the same inward-facing import restriction the boundary record owns (`rule:ownership:import-allowlist`), so a quantity from the derivation layer cannot be reached from here even by accident. Without that, opacity would be a property of the current implementation, and the difference between the two is whether a future edit can quietly remove it.

**Convention (No surface carries a status field)** · `conv:surface:no-status`

No surface here carries a status field, and this record carries no status of its own. Where something is decided but not built, it is stated in a caveat that names the gap, which is a claim a reader and the linter can both check.

The rule earns its place in this record specifically. The census found the corpus's status vocabulary did not separate *decided* from *built* (`obs:assayer:adr-status-vocabulary`), and the four contracts collapsed here carried statuses that disagreed with their own bodies in both directions — a surface marked as a proposal while three other records cited it as settled law, and a surface marked complete while depending on two proposals.

**Register (The three contradictions this record collapses)** · `reg:surface:collapsed-contradictions`

Three, each now owned in one place. The **channel field**: the assessment contract decided a channel-free request and asserted a channel field in a result back-reference, a numbered consequence, and its verification table; the body is right and the input is channel-free (`dec:surface:no-channel-input`). The **slot's shape**: five records gave it three different shapes, and the register tracked three restatements and not the two that conflicted; it is now cited, never reproduced (`dec:surface:slot-map`). The **not-found expiry**: the label contract reasoned about its error on an expiry one factor of twenty-four from the configured default.

The third is the one that shows why the split exists. The expiry is a construction parameter, so this record states no figure for it and the construction record states it once (`tab:construction:parameters`). The census recorded all three with their evidence (`reg:assayer:adr-contradictions-records`), and in each case the code adjudicates.

**Caveat (This record states no numeric default of its own)** · `cav:surface:no-local-figures`

Nothing here carries a capacity, a budget, an expiry, a threshold, or a count. Every figure the four surfaces reason about is a parameter of the construction contract and is named there once (`tab:construction:parameters`), with its value in the code rather than in prose.

This is a real constraint on the record and not a stylistic preference. The four legacy contracts collapsed here stated figures in the dozens, several of them inconsistently with the configuration record and with the code — which is the failure a number restated across documents produces rather than an accident of maintenance. A reader wanting a value should read the code; a reader wanting to know which parameter governs a surface should read the construction record.
