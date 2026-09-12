### Chapter (The Assessment Interface) · `chap:spec:assessment-interface`

The chapter carries the Core's principal output and the path that produces it. Reports arrive and are published; requests arrive and are answered; between the two sit two routing indices, a pipeline that reads almost everything and writes almost nothing, and an output structure that is a determination of position rather than a recommendation of course. The chapter ends with the composition pattern, which is the only place in the document where all three components appear in one worked sequence.

**Algorithm (Sentinel report reception)** · `alg:runtime:report-reception`

A batch report arrives for one Sentinel and atomically replaces that Sentinel's cached report. Publication is a pointer swap: assessments already in flight continue reading the previous report, assessments beginning afterwards read the new one, and no assessment sees a half-replaced report or waits for the replacement to finish. Calls for one Sentinel are serialised; calls for different Sentinels proceed independently.

The report consumed here is the post-lift form, its coordinates already carried into the Core's internal width (`alg:encoding:coordinate-lift`). The Core reads the per-cell analyses, the scoring and maturity records, the scoring geometry, the coordination reports, the contour snapshot and the analysis-set summary, and does not read the per-sample scores, the inline health report or the wavelet portrait; what it does not read is enumerated where the boundary is verified (`tab:boundary:not-consumed`). Non-consumption is not a judgment of value: the portrait explains how a Sentinel's partition came to be, which is a direct Sentinel-to-host diagnostic and not a thing the Core could estimate.

Two further effects run inside the same call, before the swap: the report index is rebuilt (`def:runtime:report-index`) and the Ledger's cell set is maintained against the new report (`alg:runtime:cell-set-maintenance`).

**Requirement (The assessment interface)** · `req:runtime:assessment-interface`

The interface takes a batch of request contexts and returns one assessment per request. A context carries the entity key, a timestamp, the declared signals, and one coordinate per Sentinel.

Concurrent calls are the normal case and are not a degradation of one. Each request in a batch is processed independently: no cross-request state, no ordering dependency, and no consistency guarantee within the batch beyond each request seeing whatever state was current when it ran. A batch is an amortisation of call overhead, not a transaction.

There is no channel field, because the Core has no channels. One assessment is channel-independent and may be carried into any number of landscapes by successive derivations against different policies, which is what lets a single quadratic-cost assessment serve a dozen channels (`sig:landscape:derivation-function`).

**Algorithm (The assessment pipeline)** · `alg:runtime:assessment-pipeline`

The pipeline reads published model state acquired once at the start of the call, the current Sentinel reports, and the Ledger — the last through a decay computed at read time as a pure function rather than persisted (`def:ledger:time-decay`). For each request, in order:

1. **Route** the request's coordinates to receiving cells in each online Sentinel's cached report (`def:runtime:report-index`).
2. **Extract** the per-Sentinel features and compute the alarm summaries (`def:runtime:alarm-summary`).
3. **Observe identity.** Encode the entity key on each registered dimension, record the observation for deferred processing (`alg:publication:identity-draining`), read the current competitive set from published state, populate the competitive indicators, update the per-range measurement state, and populate the per-dimension and cross-dimension features (`def:keyspace:competitive-set`).
4. **Aggregate** across Sentinels and accumulate the sub-scores into the concordance window (`def:feature:template-aggregate`).
5. **Assemble, offer and standardise** the feature vector: fixed blocks directly, dynamic blocks from the extractions and the identity state, interactions by iterating the compiled triples (`alg:dimension:compilation-pipeline`). While cold initialisation is still active, offer the assembled raw vector to the standardisation ramp (`alg:standardisation:batch-initialisation`). Then standardise the whole against the means and variances of the snapshot acquired at the start of the call; acceptance of the offer may reach only a later snapshot.
6. **Estimate risk**: the two regressions, the blend weight, the blended raw estimate and its probability-scale image with uncertainty (`def:risk:subspace-blend`) and (`def:risk:probability`).
7. **Predict each active axis** (`tab:axis:inference-outputs`).
8. **Record** the request in the pending buffer (`def:runtime:pending-entry`).
9. **Return** the assessment (`schema:output:assessment`).

There is no derivation step. The pipeline ends at an assessment, and a host wanting a landscape asks for one separately. Two properties hold over the whole sequence: per-request independence, so that a batch of $n$ produces what $n$ single calls would, and the enumerated write set below.

**Invariant (The enumerated assessment-path writes)** · `inv:runtime:enumerated-writes`

The assessment path directly writes exactly the three pieces of shared state in this table and nothing else.

| Write | Where it happens | What it is |
| --- | --- | --- |
| Per-dimension measurement state | Identity observation | A bounded per-dimension update in the active competitive cells |
| The concordance window | Aggregation | A sharded accumulator taking one sub-score set |
| The pending entry | Recording | One insertion into a concurrent map |

No model parameter, no dimension map, no published standardisation statistic, no identity graph structure — importance, topology or competitive set — and no Ledger entry is written by an assessment, ever. The Ledger in particular is read through a pure decay computation rather than a decay-and-store, which is precisely what allows an assessment to consult outcome memory without taking a write lock (`def:ledger:time-decay`).

Beside those three an assessment may enqueue work to the thread that owns it: the identity observation, the signal-cache insertion, the cold standardisation observation and the per-Sentinel bootstrap observation. The two bootstrap accumulators may also advance while they are active. None of these changes the immutable snapshot the assessment acquired, and none of them is a write to a model.

What separates them from the label path is evidence authority rather than the absence of a trace. An assessment never advances an operational, sister, anchor or outcome-axis posterior, a Platt parameter or a calibration diagnostic, an outcome-memory value, or the continuing standardisation average: every one of those is taught by an outcome, and every one of them moves on the label path alone. An accepted cold or per-Sentinel observation may move observational geometry, and moves it only into a snapshot later than the one its own request was answered from (`alg:standardisation:batch-initialisation`). What this invariant promises is therefore not that repeated assessment leaves no residue — the ramp exists in order to leave it — but that nothing an assessment leaves behind was taught by an outcome.

The invariant is what makes the concurrency chapter's non-blocking claim checkable rather than aspirational: a reader can enumerate three writes and confirm each is behind a bounded or structural budget (`tab:publication:tiers`), and the enqueues are enqueues rather than mutations of the structures they feed, drained on the maintenance thread's cycle (`alg:publication:identity-draining`).

**Table (The numeric checkpoints)** · `tab:runtime:numeric-checkpoints`

The system takes six numbered numeric readings between its boundary and its models, placed from the boundary inward. The placement is definition rather than decision — where each reading is taken and what it does on a non-finite finding — and it lives here because the degradation record declines to own it (`cav:degradation:checkpoint-placement`). The first four are taken on every assessment, and what each sanitises is counted into that assessment's degradation context; the last two guard model state at the end of the label update path (`alg:runtime:update-path`), and each reverts the label's whole effect to the last published snapshot and counts the revert on the published health summary.

| Checkpoint | Where it is taken | Reading, and the response |
| --- | --- | --- |
| CP1 | The signal boundary, before the cache merge | A non-finite host signal value is counted and sanitised to zero |
| CP2 | Each Sentinel's extraction | An extraction carrying any non-finite value is zeroed whole, occupancy included, and the Sentinel recorded |
| CP3 | The assembled feature vector, after standardisation | A non-finite position is sanitised to zero and counted |
| CP4 | The model estimates, after the blend | A non-finite point estimate or uncertainty falls back to the prior, per model |
| CP5 | The working copy, after a label's updates | Any model with a non-finite mean entry, or a precision matrix a factorisation refuses, reverts the label; so does an update any model refused, the covariance it read having yielded no non-negative leverage |
| CP6 | The snapshot about to publish | A non-finite mean or covariance entry, or a non-positive covariance-diagonal entry, reverts likewise and a clean snapshot publishes |

All six are performed as placed. The two halves differ in posture on purpose: an assessment must answer, so its four checkpoints substitute and disclose; a label may be refused whole, so its two checkpoints revert rather than repair, and the reverted state is the last state known good — where the rebuild that produces it succeeds.

CP5's precision reading is a verdict rather than a diagonal scan, and it costs nothing extra to take. A strictly positive precision diagonal is necessary for definiteness and is not sufficient for it (`req:gaussian:positive-definiteness`), so a diagonal scan admits exactly the matrices this checkpoint stands between and the published snapshot; a spectral reading, or even a factorisation, taken per model per label would be cubic work on a path that is otherwise linear in the width. It does not have to be taken here, because it has already been taken: the recomputation check runs earlier on the same label (`alg:runtime:update-path`), so a model that rebuilt this label carries this label's verdict, and a model that did not carries its last rebuild's.

That stored verdict is current rather than stale, because every mutation between two rebuilds preserves definiteness. The bounded update adds a positive-semidefinite rank-one term, its leverage being non-negative or the update having been refused — and a refused update reverts the label through this checkpoint's other reading rather than through this one. The forgetting scales the matrix by a positive factor, which preserves the definite cone exactly. The replenishment clamp only raises diagonal entries (`req:gaussian:prior-replenishment-floor`). So a matrix a factorisation accepted at the last rebuild is accepted still. A model that has never rebuilt carries no verdict and is not read: its precision matrix is the prior, definite by construction. The non-finite scans stay beside the verdict and are not folded into it, because a not-a-number entry is not a definiteness question and no earlier factorisation's verdict speaks to it.

The revert is a rebuild of the working copy from the published snapshot, and the rebuild can itself be refused, because the snapshot's covariance may not be positive definite in the working arithmetic. That verdict is the factorisation's own, and there is one factorisation to take it from (`dec:posterior:repair-cascade`). Where the rebuild is refused, the checkpoint has judged the working copy corrupt and cannot replace it, and there is no last state known good left to revert to.

The label path then stops, which is what the two checkpoints do when the disposition they exist to take is unavailable (`dec:surface:async-label`). No further label is applied; labels already queued are drained and refused rather than applied to the state the checkpoint has just disowned; the published snapshot stays as the read surface, so assessments go on answering; and the cause is recorded once — the model whose reconstruction was refused, the pivot, and how far into the label stream it happened — reaching the host at its next submission and on both tiers of the health surface. The command path is untouched, so a checkpoint or a shutdown requested after the stop still completes. The rebuild itself is at the deployment's declared prior rather than at the library default, so a revert does not retune the deployment on its way past; there is no second numerical parameter for it to take, because the factorisation it performs carries none (`dec:posterior:one-inversion-utility`).

One of CP5's two readings is taken inside the update rather than after it, and for a reason particular to what that reading is. The leverage is a quadratic form in the covariance, so it is non-negative for every feature vector exactly while the covariance is positive semi-definite, and a negative one is the report that the matrix is no longer a covariance. The update cannot consume that reading and disclose it afterwards, because the weighting divides by the leverage: a negative denominator turns the effective weight negative, a negative weight subtracts information the model never received, and the subtraction carries the precision matrix further from the property whose loss was being reported. So the update stops at the reading, mutating nothing, and the checkpoint then disposes of the label the way it disposes of any other corrupt finding. The other five readings are of state already written, and are taken where the table places them.

**Definition (Time-corrected uncertainty)** · `def:runtime:time-correction`

Model parameters decay in elapsed time as well as in labels, but the decay is applied when the working copy is available for mutation, which is at label time. Between labels the published snapshot's covariance stands still while the clock does not, so an uncertainty read from it would be too narrow by exactly the decay that has not yet been applied.

The correction is a scalar on the quadratic form, applied at read time to each model before blending:

$$\sigma^2(\phi) = \frac{1}{\gamma_t^{\Delta t_\text{snap}}} \cdot \hat{\phi}^\top \Sigma_\text{snapshot} \hat{\phi}$$

where the elapsed interval is measured from the snapshot's publication. The point estimate is untouched, because time-indexed decay scales the precision and the covariance reciprocally and leaves the mean where it was (`alg:temporal:lazy-application`).

The correction is exact rather than a safety margin, and it is the reason an assessment taken from a snapshot published hours ago reports honest uncertainty rather than the uncertainty of the moment the snapshot was cut. It applies on every model's quadratic form.

**Definition (The current report index)** · `def:runtime:report-index`

The current report index is a sorted interval map over one Sentinel's coordinate space, holding the cells present in that Sentinel's most recent batch report. Each entry maps a coordinate range to the cell's report data: its scores at every ancestor level, its coordination features, and its batch statistics.

The index is rebuilt in full on each reception and published by the same atomic swap the report itself is (`alg:runtime:report-reception`). Rebuilding rather than patching is the simpler discipline and the correct one: a batch report is a complete statement of what a Sentinel currently sees, so an index that merged it with an older statement would be describing a partition that never existed.

Where several reported cells cover a coordinate at different depths, lookup takes the deepest, because the deepest reported cell is the finest statement the Sentinel has made about that coordinate and the coarser ones are its ancestors. Report staleness — how far a Sentinel's baselines have moved since it reported — is carried as a feature rather than corrected for here (`def:extraction:report-staleness`). The index is published behind a lock-free swap.

**Definition (The Ledger spatial index)** · `def:runtime:ledger-index`

The Ledger index is a layered depth-priority map over each Sentinel's domain. Entries may overlap: several can cover one coordinate at different depths, and that overlap is the structure rather than a defect of it.

The two routings are deliberately different. A read finds the *deepest* entry containing the coordinate and reads only that one, so extraction sees the finest outcome memory available and never averages it with a coarser statement. A write finds *all* entries containing the coordinate, deepest to root, and updates each independently, so a label at a leaf informs its ancestors as well (`alg:ledger:all-layers-update`). Read routing always succeeds because the root entry always exists (`def:ledger:root-semantics`).

The index is populated incrementally from batch reports: a reported cell without an entry gets one (`alg:ledger:entry-creation`). Entries then persist across reports, accumulating outcome history that no single report contains, and leave only by the disappearance threshold or by collection (`alg:ledger:entry-deletion`) and (`alg:ledger:garbage-collection`). Reads and writes use their distinct routings.

**Algorithm (Extraction routing)** · `alg:runtime:extraction-routing`

For each Sentinel and each request's coordinate in that Sentinel's space, three lookups run and their failures are handled differently.

Batch features come from the deepest reported cell covering the coordinate; if no reported cell covers it, the Sentinel's batch features are zero for this request, which is the honest encoding of "this Sentinel currently says nothing here" (`tab:extraction:chain-z-scores`). Coordination features are read from the same cell's coordination summary and are zero on the same condition (`tab:extraction:coordination`).

Ledger features come from the deepest Ledger entry containing the coordinate, decayed to the present as a pure function and read without mutation (`tab:extraction:ledger-features`). This lookup cannot fail. The asymmetry is the point: a Sentinel may fall silent over a region and the Core loses its current measurement there, but outcome memory is the Core's own and remains available at whatever depth it has accumulated.

All three lookups follow the stated routing, the Ledger one taking the Ledger's own read routing so that the depth-walk the memory record specifies is the walk the assessment path performs (`dec:memory:depth-walk`), and the read leaving the stored entry untouched. Assessments in split regions therefore read the deeper, more specific outcome history, and the feature distributions the models see shift once as online adaptation absorbs the new reading. Nothing compensates for the shift, because a shim holding shallower distributions steady would preserve a dilution, and the adaptation the models already perform is the mechanism that absorbs a one-off shift of exactly this kind.

**Algorithm (Reported cell-set maintenance)** · `alg:runtime:cell-set-maintenance`

On each reception the Ledger index is reconciled against the reporting Sentinel's new report in three steps. Every cell in the report that lacks an entry at its exact interval gets one, created neutral. Every entry below the root that was previously associated with a reported cell has its consecutive-absence counter incremented if its interval is absent from this report and reset to zero if present. Every entry whose counter reaches the threshold — three, by default — is deleted.

No merge is needed on deletion, because all-layers write routing has kept the ancestor entry current all along (`alg:ledger:all-layers-update`).

The Core classifies nothing. It does not decide whether a vanished cell was split, evicted, restored or subdivided; it observes appearance and persistent disappearance and acts on those two facts alone (`conv:terminology:abstraction-boundary`). The Sentinel-internal cause is not merely unavailable but irrelevant, and treating it as relevant is the error this discipline exists to prevent (`cav:limitation:abstraction`). The contour snapshot's structural metadata is relayed to the host as diagnostic context and drives no Ledger operation.

**Bound (The cost of routing)** · `bound:runtime:routing-cost`

| Operation | Cost | When |
| --- | --- | --- |
| Rebuilding the report index | $O(\lvert R\rvert \log \lvert R\rvert)$ over the report's cells | Per reception |
| Ledger cell-set maintenance | $O(\lvert R\rvert \log \lvert L\rvert + \lvert T\rvert + \lvert D\rvert)$ over report, Ledger, tracked entries and deletions | Per reception |
| Read routing, all Sentinels | $O(n_s(\log \lvert R\rvert + \log \lvert L\rvert))$ | Per assessment |
| Write routing, all Sentinels | $O(n_s \cdot d_\text{max})$ | Per label |

At eight Sentinels and a maximum depth of eight, a label costs at most sixty-four constant-time updates to the Ledger. Against the quadratic model updates on the same path this is not a cost worth optimising, and the table is carried to say so rather than to warn.

**Bound (Ledger memory)** · `bound:runtime:ledger-memory`

The Ledger's memory is bounded in practice by the number of distinct cells reported within the decay horizon, not by the number ever reported: the time-indexed decay carries an unvisited entry toward zero and collection removes it once it arrives (`def:ledger:time-decay`) and (`alg:ledger:garbage-collection`).

At a thousand cells per Sentinel per day with thirty percent overlap between days, steady state is around twenty thousand entries per Sentinel; at roughly two hundred bytes each and eight Sentinels, some thirty-two megabytes. This is small enough to be uninteresting beside the pending buffer, which is where the system's memory actually goes (`tab:resource:memory`).

**Schema (The risk assessment)** · `schema:output:assessment`

An assessment is a determination of position. It carries five things.

| Field | Carrying |
| --- | --- |
| Identifier | The handle a label is later reported against |
| Risk basis | The full risk estimate and its provenance (`schema:risk:basis`) |
| Outcome predictions | One prediction per active axis, with uncertainty (`tab:axis:inference-outputs`) |
| Per-Sentinel summaries | One alarm summary per reporting Sentinel (`def:runtime:alarm-summary`) |
| Health snapshot | The compact validity flags a per-request consumer needs (`schema:output:health-snapshot`) |

The basis is transparent by design: the blend weight shows how much of the estimate rests on the coarse anchor rather than the fine sister model, the two regime record counts show whether the calibration is mature enough to be believed (`req:platt:minimum-samples`), and the borrowed share shows how much of the precision behind the reported uncertainty the models' own floors were holding up along this request's features rather than the evidence (`dec:risk:evidence-only-uncertainty`). The uncertainty beside it has already been widened by that share, so the share is what a host reads to undo the widening or to apply a correction of its own. Probability-scale values serve the host; raw-scale values serve the derivation; both are always present, because deciding which a consumer needs is not the Core's business.

What is absent is as specified as what is present. No tags, no posture, no channel reference, no action probabilities, no fragility. Every one of those is a derivation output, computed from an assessment by a host that has declared what its actions cost (`prin:principle:measure-not-decide`).

**Schema (The embedded health snapshot)** · `schema:output:health-snapshot`

Every assessment carries a compact, fixed-size health snapshot of eleven specified fields, so that a per-request consumer can tell a trustworthy assessment from a provisional one without fetching anything.

| Field | Carrying |
| --- | --- |
| Snapshot version | The join key into the full health report stream |
| Snapshot age | The elapsed interval the uncertainty correction was computed at |
| Warm-up stage | Where the deployment stands against the warm-up sequence |
| Sister calibrated | Whether the sister regime has enough records to be believed |
| Anchor regime frozen | Whether the anchor's calibration has been fixed |
| Drift flag | Whether any model's drift accumulator is near its threshold |
| Coverage fraction | Reporting Sentinels over registered ones, for this assessment |
| Uncertainty inflation | The most recent empirical inflation factor |
| Zero Sentinels | Whether the assessment was computed with none reporting |
| Standardisation phase | `WaitingForInit`, `Transitioning` or `InService`, for the coordinate snapshot this assessment acquired |
| Standardisation observations | Accepted cold-ramp observations represented by that snapshot, from zero through $N_\text{init}$ |

The version is the field the design turns on. Everything per-Sentinel or per-dimension stays in the full report and is fetched by version when forensics need it (`chap:spec:output-structures`), so an assessment log stays small and is still fully auditable after the fact — the snapshot says which report to go and read. It is also the coordinate version, because every advance of the ramp publishes a model snapshot (`alg:standardisation:batch-initialisation`): the version says which coordinate system a result was computed in, and the phase and the count say how far through its acquisition that system was. A second version namespace would be a second authority over one fact. Ramp maturity is the count divided by $N_\text{init}$ and is not carried beside it, for the same reason: a stored fraction and a stored count are two descriptions of one position, and two descriptions of one thing maintained apart eventually disagree.

The last two rows describe the snapshot the assessment used and not the ramp's position at the moment of reading, which is the whole of their forensic value. A consumer comparing two results is comparing two coordinate systems whenever the versions differ, and these fields say which.

Two of the fields are computed elsewhere on the same path and are merely carried here, the age by the uncertainty correction (`def:runtime:time-correction`) and the inflation factor by the coverage diagnostic (`alg:monitoring:empirical-coverage`); the stage and the drift flag likewise restate what their own environments own (`tab:warmup:stages`) and (`alg:monitoring:drift-cusums`). The snapshot carries all eleven fields, including the version, phase, accepted count and anchor-regime frozen flag, so the forensic join and the ramp position this environment establishes can both be read directly.

**Definition (The per-Sentinel alarm summary)** · `def:runtime:alarm-summary`

The alarm summary is a deterministic compression of one Sentinel's extraction into eight diagnostic fields: the peak z-score, the peak accumulator, the peak coordination z-score, a composite, the chain depth, the maturity, whether the Sentinel's coordinates are hierarchical, and whether the covering Ledger entry is immature.

It does not enter any model. It exists so a host can see which Sentinel is alarming and how confidently, which is a question the risk probability cannot answer because it has already aggregated the answer away.

The composite is the maximum of the cell-level peak z-score and the cell-level peak accumulator after normalisation, giving one scalar per Sentinel. The hierarchical flag reflects the coordinate semantics declared at registration. The immaturity flag is a maturity predicate: it holds when the deepest Ledger entry covering the request has fewer eligible labels than the materiality threshold, or when its estimated eligible label rate implies a steady-state attenuation below the floor — the two conditions under which the entry's outcome features are present but not yet worth weighting (`thm:ledger:materiality`) and (`data:ledger:attenuation`). The predicate is taken against the entry the extraction routing actually reached rather than against the mere existence of an entry (`alg:runtime:extraction-routing`).

Each input is read where it is stored. The eligible-label count and the arrival window that names the steady-state attenuation are fields of every entry (`tab:ledger:entry-state`); the count arm compares against (`def:config:ledger-materiality-threshold`) and the rate arm against (`def:config:attenuation-materiality-floor`). The health surface reads the same two fields over the whole Ledger, but it does not report this flag aggregated: it counts cells against the threshold alone (`def:monitoring:immature-cells`) and reports the attenuation condition separately, as a share of traffic rather than a count of cells (`def:monitoring:ledger-value`). The evidence is shared and the readings are three, so neither health figure is the fleet-wide total of this flag and none of the three may be substituted for another.

Two boundary readings are fixed rather than left to a caller. An entry whose arrival window has emptied names no attenuation, and the rate arm takes the limit the attenuation formula gives there — zero, which lies below every floor the configuration admits (`def:config:attenuation-materiality-floor`) — so a cell that has stopped receiving evidence cannot pass as mature on a count it stopped adding to. A Sentinel with no Ledger entry at all is immature for want of any evidence whatever; with the root in place that state does not arise, because the root contains every coordinate (`def:ledger:root-semantics`).

**Example (Composing the three components)** · `ex:runtime:composition`

The host is the only place the three components meet, and the pattern is short. It assesses a request; it reads the Companion's posterior; it derives a landscape from the two against a channel policy; it reads the landscape at its posture for an action and for that action's fragility; it decides and acts. Nothing in that sequence is a call the Core makes.

Multiple channels cost almost nothing. One assessment is derived against each channel policy in turn — the quadratic-cost work is paid once and each derivation is linear in the declared actions plus a handful of quantile evaluations.

Replay needs three stored values and no component state: the assessment, the channel policy, and the challenge posterior. Those reproduce the landscape exactly, without access to model parameters, pseudo-counts, Sentinel reports or the pending buffer, which is what makes an audit of a past decision possible at all (`inv:guarantee:replay`).

Feedback is two independent reports, not one. The Core is labelled with the outcome; the Companion is updated with the challenge result, and only when a challenge was performed on an adverse request (`alg:companion:update`). They touch different components and may go in either order. The Core never sees the challenge result and the Companion never sees the valence magnitude or the feature vector, so the boundary the specification draws holds at runtime and not only on paper (`inv:companion:boundary`). A host wanting neither landscapes nor challenges runs the first and last steps alone, and needs no Companion at all.
