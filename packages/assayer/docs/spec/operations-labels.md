### Chapter (The Label Interface and Learning Pipeline) · `chap:spec:label-pipeline`

The chapter is where the system learns. A host reports an outcome against a stored assessment; the stored assessment is found, its feature vector rebuilt in the current coordinate system and restandardised against the current statistics, and seventeen ordered steps then move every quantity a label is evidence about. The buffer that holds assessments between the two halves is specified here too, because its contents are exactly what reconstruction has to work from, and because what it does not store is what reconstruction cannot rebuild.

**Requirement (The label interface)** · `req:runtime:label-interface`

A label is reported against a stored assessment and carries the five fields the host contract fixes (`tab:host:label-reporting`). The call returns an acknowledgement, or an error where the assessment is not in the buffer — expired, evicted, or already labelled.

Two properties bind. Label processing is serialised internally on the path that owns the models, because the updates are not commutative in their conditioning and a second label applied concurrently to the first would produce a posterior neither of them justifies. And label processing never blocks assessment: a label arriving under load is queued rather than executed inline, and the queue is bounded (`req:publication:label-queue`) and (`inv:guarantee:non-blocking`).

The sign convention governing valence is the document's, fixed once and read here (`conv:valence:sign`). The five fields form one payload, and the call returns a typed acknowledgement.

**Algorithm (The label update path)** · `alg:runtime:update-path`

When a host labels assessment $r$, seventeen steps run in this order.

1. Retrieve the pending entry (`def:runtime:pending-entry`).
2. Reconstruct the feature vector in the current coordinate system (`alg:runtime:reconstruction`).
3. Restandardise against the current statistics (`bound:standardisation:restandardisation`).
4. Compute the risk target from the sign of the valence (`def:risk:target`).
5. Update the valence compression scale, where the valence is non-zero (`def:risk:compression-scale`).
6. Determine eligibility from the action taken and the ground-truth flag (`tab:eligibility:training`).
7. Update the class-rate trackers: the global tracker always, the eligible tracker only where the label is eligible (`alg:weighting:tracker-update`).
8. Compute the importance weight from the appropriate tracker (`def:weighting:balancing-weights`).
9. Update the operational model (`alg:update:sherman-morrison`).
10. Where the label is eligible, update the sister and anchor models.
11. For each active axis with a reported value: compress it, update its compression scale, determine its eligibility, and update its model where eligible (`def:axis:adaptive-compression`) and (`def:axis:training-target`).
12. Update the identity dimension outcome state (`alg:runtime:identity-outcome-update`).
13. Update the Ledger, per Sentinel and per reported axis (`alg:ledger:all-layers-update`).
14. Update the calibration buffer, refitting where the cadence triggers (`constr:platt:buffer`) and (`tab:platt:refit-cadence`).
15. Update the standardisation statistics (`alg:standardisation:label-time-procedure`).
16. Update the prediction drift accumulators (`alg:monitoring:drift-cusums`).
17. Publish the working copy as the new snapshot (`def:publication:model-snapshot`).

No challenge effectiveness update appears, because the Companion is the host's to feed and the Core never sees a challenge result (`inv:companion:boundary`). Steps two and three are where the coordinate system is reconciled: the buffer holds raw features, and the statistics they are standardised against have moved since the assessment, by a bounded amount. Step nine runs unconditionally and step ten does not, which is the whole of the eligibility distinction in operational terms. Steps nine, ten and each axis update in step eleven are mutually independent — each reads the shared standardised vector and the target and writes only its own model — so they may run in parallel, reducing the wall-clock cost from a multiple of the quadratic update to roughly one of them.

**Algorithm (The identity outcome update)** · `alg:runtime:identity-outcome-update`

At step twelve, for each registered identity dimension: the stored entity key is re-encoded, and the competitive cells recorded at assessment time are walked. For each such cell still competitive, lazy decay is applied and then its adverse rate, its compressed and raw valence averages, and its per-axis outcome averages are updated (`tab:keyspace:decay-rates`). A cell no longer competitive is skipped: its outcome state no longer exists, its information having been transferred into the surviving dimensions when it left (`alg:keyspace:cell-exit`).

The update reaches every competitive cell containing the coordinate and not only the deepest. An outcome from one host address updates the address's cell, its subnet's cell and its block's cell alike, wherever all three are competitive, because each is a distinct claim about a distinct population.

The asymmetry with reconstruction is deliberate and worth stating. Outcome averages are written to the *stored* active set, filtered to cells still competitive; reconstruction populates indicator features from the *current* set (`alg:runtime:reconstruction`). A cell promoted in the interval therefore gets a reconstructed indicator and no outcome update this cycle, and a demoted one gets neither. The outcome average belongs to the entity's history and must not be written into a cell that does not hold that history; the indicator describes the model's present parameterisation and must match it. The divergence is small, because competitive restructuring is rare, and it self-corrects on the next label (`cav:limitation:competitive-churn`).

**Bound (Per-label cost)** · `bound:runtime:label-cost`

The per-label cost is tabulated once, with the other resource bounds, and is cited here rather than duplicated (`tab:resource:label-cost`). At the reference configuration it is roughly $1.22$ million operations, on the order of a millisecond (`tab:resource:reference-configuration`).

The shape of that figure is what matters operationally: it is dominated by the quadratic model updates of steps nine through eleven, so it grows with the square of the feature dimension and linearly in the number of active axes, and is essentially independent of the number of Sentinels, the Ledger's size and the identity layer's depth. A deployment whose label cost is a problem has too many dimensions, not too much traffic.

**Definition (The pending entry)** · `def:runtime:pending-entry`

An assessment is recorded in the pending buffer so that a label arriving later can be matched to what was actually computed. The entry carries twelve things.

| Field | Carrying |
| --- | --- |
| Identifier and timestamp | The handle the label is reported against, and when |
| Entity key | The key, for re-encoding on each identity dimension |
| Per-Sentinel extractions | One frozen extraction per reporting Sentinel |
| Active Sentinels | Which Sentinels were registered at assessment time |
| Reporting Sentinels | Which of those were actually reporting |
| Identity coordinates | The encoded coordinate on each dimension |
| Identity active cells | The competitive cells the coordinate fell in |
| Entity base features | The frozen per-dimension identity features |
| Entity axis features | The frozen per-dimension, per-axis identity features |
| Signal features | The frozen encoded signals |
| Risk basis | The estimate, retained for drift and calibration |

The buffer stores the per-Sentinel extractions separately rather than the assembled vector, and that is the design decision the rest of the chapter rests on. An assembled vector is indexed against the dimension map that existed when it was assembled; a lifecycle event between assessment and label invalidates those indices, and the stored vector would then be a set of numbers whose meaning had moved. Stored components can be replaced into a current map instead (`alg:runtime:reconstruction`).

No channel field and no stored landscape: the entry describes what the Core computed, not what the host decided from it. The basis is retained for the drift accumulators and the calibration buffer, which read the prediction rather than the features (`schema:risk:basis`).

**Definition (Storage precision)** · `def:runtime:storage-precision`

Pending entries store feature values at single precision by default, configurably, and upcast to double at label time.

The reasoning is a comparison of two errors rather than an appeal to frugality. Single-precision quantisation costs about one part in ten million per feature. Restandardisation — the same feature expressed against statistics that have moved between assessment and label — costs about one part in a hundred (`bound:standardisation:restandardisation`). The second is five orders of magnitude larger, so storing at double precision would halve the buffer's capacity to remove a hundred-thousandth of the error already present, which is not a trade any deployment should want.

The same reasoning is applied at the same width where extraction is stored, for the same reason (`rem:extraction:single-precision`).

**Requirement (Buffer capacity and eviction)** · `req:runtime:buffer-capacity`

Capacity is computed from the deployment's own numbers rather than guessed: twice the expected peak request rate times the expected median labelling latency, so that a burst at peak rate is still fully labelled at median delay.

| Request rate | Labelling latency | Required capacity | Memory at single precision |
| --- | --- | --- | --- |
| $100$ per second | One hour | $720{,}000$ | About $2.2$ GB |
| $10$ per second | Fifteen minutes | $18{,}000$ | About $54$ MB |
| $1$ per second | One hour | $7{,}200$ | About $22$ MB |

The table's first row is the reference configuration: the peak request rate (`def:config:reference-peak-request-rate-100-per-second`) paired with the median labelling latency (`def:config:reference-label-latency-3600-seconds`).

The buffer is the system's dominant memory consumer at any serious request rate, which is why the sizing rule is stated as a computation: a fixed default is either wasteful at low rates or silently lossy at high ones. Beyond capacity the oldest entries are evicted, and a label arriving for an evicted entry is lost — the call errors, and the outcome it carried never reaches any model (`cav:limitation:buffer-eviction`). Eviction is therefore not a graceful degradation but a measured loss of evidence, and the eviction rate belongs in the health report where a host will see it (`chap:spec:output-structures`). The parameters are tabulated with the rest of the configuration surface (`tab:config:pending-buffer`). Capacity is $2 \times R \times L$ from the reference peak request rate (`def:config:reference-peak-request-rate-100-per-second`) and the reference median labelling latency (`def:config:reference-label-latency-3600-seconds`), and health reports the lifetime share of pending insertions lost to live eviction alongside the age of the oldest live pending entry. The interval count also travels inline on the assessment whose insertion observed it (`entry:retention:eviction-counter`), (`entry:retention:eviction-visibility`).

**Algorithm (Reconstruction at label time)** · `alg:runtime:reconstruction`

At label time the feature vector is rebuilt from the stored components against the *current* published dimension map and the current model parameters, which are one self-consistent version (`inv:dimension:version-consistency`). Rebuilding against the current map is what makes a lifecycle event between assessment and label harmless: features of a deregistered entity are in the buffer and are simply not placed, because the map no longer holds indices for them, and their information has already been transferred into the surviving dimensions (`def:gaussian:schur-complement`).

Six numbered sources fill the vector, and every index has exactly one of them.

1. **Fixed blocks.** The bias, the stored signal features, and the stored identity base features.
2. **Identity dimension features.** For each currently registered dimension: re-encode the stored coordinate, look up the *current* competitive set, populate the indicators from it, and recompute the aggregates (`def:keyspace:competitive-indicators`).
3. **Axis features.** For each currently registered axis: the stored features where the axis existed at assessment time, zeros where it was registered afterwards.
4. **Sentinel slots.** For each currently registered Sentinel: the stored extraction where it was reporting, occupancy with zero features where it was active but silent, and no slot at all where it is no longer registered (`def:extraction:slot`).
5. **Aggregate features.** Recomputed from the stored extractions over the currently reporting Sentinels (`def:feature:template-aggregate`).
6. **Interactions.** Recomputed by applying the current templates to the reconstructed features (`alg:dimension:compilation-pipeline`).

The covering invariant is what guarantees the enumeration is complete: every index in the reconstructed vector is claimed by exactly one source, so there is no position whose value is a matter of which step ran last (`inv:dimension:covering`). With the working state unchanged between assessment and label the reconstruction reproduces the assessment-time raw vector exactly, block for block, which is the equality that settles it.
