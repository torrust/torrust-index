# Vector · `rec:vector:feature-assembly`

This record owns how the feature vector is assembled and how the statistics it is standardised against are acquired and updated. The two halves are one record because the order between them is the decision: what is standardised, against statistics observed when, is not answerable from either half alone.

It realises two layer choices: the composition of the vector (`dec:operational:feature-composition`) and the acquisition and timing of its statistics (`dec:operational:standardisation`). It also owns the one temporal decision that is genuinely a standardisation decision rather than a decision about time, on the reasoning at (`dec:assayer:record-clock-record`).

**Decision (The vector has one canonical block order)** · `dec:vector:block-order`

The feature vector is a sequence of blocks in one fixed, total order — bias first, then aggregates, then per-dimension identity, then signals, then per-Sentinel slots, then interactions, then competitive indicators (`src/feature/dimension_map.rs`). The layout itself is the specification's (`def:feature:vector-structure`), and this record decides only that there is exactly one of it.

Canonical rather than negotiable is the choice. A vector whose block order depended on registration order, or on anything else that varies between instances, would make a persisted model uninterpretable by any instance built differently — and the failure would not be a mismatch but a silent reinterpretation of every coefficient. One order fixed in the source is what lets a stored parameter vector mean the same thing wherever it is loaded.

**Decision (One dimension map is the sole resolver of every feature index)** · `dec:vector:sole-resolver`

Every feature index comes from the dimension map and from nowhere else (`src/feature/dimension_map.rs`). No assembly site, extraction site or model site computes an offset of its own from block widths, and no site holds a constant index. The map is the specification's structure (`sec:dimension:map`) and its record form is defined there (`schema:dimension:map-record`).

Sole resolution is the decision that the census's evidence most directly supports. Index arithmetic restated at several sites is arithmetic that can disagree at several sites, and disagreement here does not fail: it reads the wrong feature and returns a plausible number. Routing everything through one resolver means the layout can be wrong in only one place, which is the difference between a bug and a class of bugs.

**Decision (Interaction templates name their operands semantically)** · `dec:vector:semantic-templates`

An interaction template names the features it multiplies by what they are, not by where they sit (`src/feature/interaction.rs`). The names are resolved into indices when the map is rebuilt at a lifecycle change, so a template written once survives every change of layout. The compilation is the specification's (`alg:dimension:compilation-pipeline`), and the lifecycle interaction it runs under is fixed there too (`alg:feature:lifecycle-interactions`).

Naming by position would be the smaller mechanism and it is incompatible with a dynamic dimension. Registering a Sentinel or a dimension moves every block after the one that grew, so a positional template silently becomes a template about different features — the same failure as a disagreeing offset, arriving through a different door. Semantic names make the rebuild the moment the question is asked, and a name that no longer resolves is a failure at rebuild rather than a wrong product per request.

**Decision (Interactions are computed from unstandardised bases, then standardised once)** · `dec:vector:unstandardised-bases`

An interaction is the product of its operands' raw values, and the standardisation pass runs afterwards over the whole assembled vector, interactions included (`src/feature/assembly.rs`, `src/feature/standardisation.rs`). Nothing is standardised twice, and no interaction is formed from already-standardised inputs.

The order is not cosmetic, because standardisation is affine and multiplication is not: the product of two standardised values is not the standardisation of their product, and it carries the two centring terms into a quantity that has no use for them. Computing raw and standardising once gives the interaction a scale that describes the interaction, and it is the order the specification's pipeline fixes (`sec:feature:interactions`).

**Decision (Label-time assembly freezes what was fixed and re-derives the rest)** · `dec:vector:label-time-assembly`

When a label arrives, the vector is rebuilt rather than retrieved: blocks whose values were fixed at assessment time are restored from what was retained, and blocks that depend on state that has since moved are re-derived (`src/feature/assembly.rs`). The reconstruction is the specification's (`alg:runtime:reconstruction`), and the two assembly times are the ordering record's to sequence (`dec:ordering:label-function`).

The split is the decision, and each half answers a different obligation. Freezing the fixed blocks is what makes the update an update about the assessment that actually happened, rather than about a reconstruction of it. Re-deriving the rest is what keeps the model learning against current structure instead of against a snapshot of the world at request time. Retaining the whole vector instead would be simpler and would cost the buffer its bound (`dec:retention:reduced-precision`).

**Decision (Statistics are observed at assessment time; the continuing update is at label time)** · `dec:vector:statistic-timing`

The running statistics see a vector when it is assembled for an assessment, and the continuing exponential update that tracks later distribution change is applied when the label for that assessment arrives — not at the moment of observation (`src/feature/standardisation.rs`). The requirement is the specification's (`req:standardisation:timing`), and the procedure the label runs is defined there (`alg:standardisation:label-time-procedure`). The cold ramp is the one acquisition that both observes and advances on the assessment clock (`dec:vector:prior-mass-ramp`), and it does not cost the separation below: what an accepted observation advances is a later coordinate snapshot, never the one the observing request scored against (`dec:ordering:score-before-evolve`).

Separating the two moments is what keeps standardisation from being circular within a single request. A statistic updated at assessment time would have already absorbed the vector it is about to standardise, so each request would be scaled partly against itself, and the effect would be largest exactly when the sample is smallest. Deferring the update to label time costs a little staleness and removes the self-reference entirely.

**Decision (Standardisation statistics carry no time-indexed decay)** · `dec:vector:no-time-decay`

The running means and variances are not decayed with elapsed time. They are not exempted by oversight: nothing anywhere applies a rate to them (`src/feature/standardisation.rs`), and this record owns that exemption rather than the record that owns decay. The clock record names the exception and points here (`rem:clock:standardisation-exception`); the specification makes the same choice for the class-rate trackers and for the same reason (`dec:weighting:no-time-decay`).

The reason is what the statistics are for. A standardisation statistic describes the distribution a feature is scaled against, and decaying it makes the scale itself drift — so a feature whose raw distribution never moved would acquire a moving standardised value, and every coefficient fitted against it would be chasing an artefact. Everything else in the system decays because old evidence is worth less; this does not, because it is not evidence.

**Decision (Two acquisition mechanisms, and no third)** · `dec:vector:two-acquisitions`

Statistics are acquired in exactly two ways: a batch initialisation that acquires them from the instance's own earliest assessments, and a per-Sentinel bootstrap that fills a newly registered Sentinel's slot afterwards (`src/feature/bootstrap.rs`, `src/feature/standardisation.rs`). Both algorithms are the specification's (`alg:standardisation:batch-initialisation`), as is the second (`alg:standardisation:sentinel-bootstrap`). Before the first accepted observation, standardisation runs on class priors; from there to the ramp's horizon it runs on a mixture of those priors and the empirical moments of what has been accepted, and at the horizon on the empirical moments alone (`dec:vector:prior-mass-ramp`).

Two is a closed set and that is the decision. Each mechanism answers a distinct cold start — the instance's and the Sentinel's — and a third would mean a third path along which a statistic could enter service without a stated provenance. The phase marker is what keeps the distinction legible at runtime: a reader can tell which regime a standardised value came from, instead of inferring it from how long the process has been up, or from the moments themselves (`dec:health:standardisation-phase-reported`).

**Decision (Cold standardisation is a finite prior-mass ramp)** · `dec:vector:prior-mass-ramp`

The class priors carry the whole of the standardisation mass when an instance is built, and each accepted raw assessment vector retires an equal share of it — one share per accepted observation, over a horizon whose value the specification tabulates (`tab:config:standardisation`). At the horizon the prior mass is zero and the published moments are exactly the empirical moments of the accepted sample: exactly, not within a tolerance, and with no rate left running underneath them. The base each share is retired against is the position's derived class prior (`dec:vector:derived-class-priors`), the bias position takes no part, and the arithmetic of the mixture is the batch initialisation algorithm's to state (`alg:standardisation:batch-initialisation`).

Two parts of that are decisions rather than consequences of arithmetic. The variance published at an intermediate count is the variance of the mixture and not a mixture of the two variances: the term that carries how far the observed mean has travelled from the prior mean is published with them, because a variance that dropped it would be neither the prior's nor the sample's and would understate the spread worst exactly while the two populations disagree most. And the accepted count is the whole of the ramp's state — the three phases a reader is told about, before the first accepted observation, between it and the horizon, and at the horizon, are readings of that count rather than a second quantity that could disagree with it (`dec:health:standardisation-phase-reported`). A lifecycle change that moves the layout takes the moments it publishes as the ramp's new base and discards the sample gathered under the layout it replaced; it rolls no published position back to that position's prior.

The ramp replaces a completion gate, and the gate was the cost. Until the ramp replaced it the priors stood untouched until one wholesale replacement at the count, so the coordinate system moved once, by its entire prior-to-empirical distance, between two adjacent requests — and results from either side of that step were results in different coordinate systems that nothing on the wire distinguished. Spreading the same distance over the same horizon moves neither endpoint and reverses no direction; what it removes is the discontinuity, and what it costs is a coordinate system in motion for the whole of the ramp instead of at one instant of it. That cost is affordable only because it is disclosed, which is why the reporting obligation is decided rather than left to a reader's arithmetic.

**Decision (The per-Sentinel accumulator is transient and deliberately not checkpointed)** · `dec:vector:transient-accumulators`

A per-Sentinel bootstrap accumulator is not written to a checkpoint and is not restored from one (`src/feature/standardisation.rs`). After a restart it is re-acquired through the bootstrap itself (`alg:standardisation:sentinel-bootstrap`), and until it is, that Sentinel's slot standardises against its class priors exactly as a newly registered one does.

Deliberate is the operative word, because this looks like an omission and is not. Persisting it would make a restored slot's scale depend on a partial sample gathered against a feature layout that the restore may have changed — and the compatibility check that guards a restore covers structure, not the distributional assumptions a statistic encodes (`dec:durability:structural-compatibility`). Re-acquiring costs a warm-up whose bound is stated (`bound:standardisation:restandardisation`); restoring would cost a wrong scale that nothing detects.

The cold ramp once stood under that reasoning and now stands outside it, because what it carries is not the same kind of thing. A bootstrap accumulator holds a partial sample waiting to be blended into a scale the rest of the vector already has; the ramp holds the coordinate system the instance is publishing, and re-acquiring it is not a bounded slot warm-up but the entire cold start again — the discontinuity that (`dec:vector:prior-mass-ramp`) exists to remove, reintroduced at every restart. Its progress is therefore checkpointed state, and what a restore does with it is the durability record's (`dec:durability:ramp-resumption`).

**Decision (Feature classes carry per-class priors derived from the map)** · `dec:vector:derived-class-priors`

Every feature belongs to a class, and each class carries the prior mean and variance used before empirical statistics exist. The class of a feature is derived from the dimension map rather than declared alongside it (`src/feature/dimension_map.rs`), which is what the specification requires (`req:standardisation:class-assignment`); the priors themselves are tabulated there (`tab:standardisation:class-priors`).

Derivation rather than declaration is the choice, and it exists so the two cannot drift. A declared class list is a second description of the layout, and a second description of the layout is the thing this record spends its other decisions preventing: it would have to be updated in step with every block change, and the failure of doing so would be a feature standardised against the wrong class's prior — wrong only during warm-up, and therefore wrong exactly where nobody is looking.

**Remark (What the dimension map must hold)** · `rem:vector:map-obligations`

The map carries three obligations, and the specification states each as an invariant: it covers every index of the vector (`inv:dimension:covering`), each block occupies a contiguous range (`inv:dimension:contiguity`), and it is versioned so a consumer holding an older one can be detected (`inv:dimension:version-consistency`).

The third is the one worth drawing out. The first two make the map a correct description; the third makes a *stale* map a detected condition rather than a silently wrong one. Without a version, a consumer that cached a map across a lifecycle change would keep resolving names to indices that have since moved, and every answer would be well-formed. Versioning converts that from a wrong number into a refusal, which is the only form in which this class of error is survivable — and compaction, which moves indices wholesale, is precisely when it matters (`alg:dimension:compaction`).

**Caveat (A Sentinel added late standardises against a moving target)** · `cav:vector:late-standardisation`

A Sentinel registered long after the instance went into service acquires its slot statistics by bootstrap, from a much smaller sample than the established slots were initialised from. Its standardised values are therefore noisier for a period, and the models see a block whose scale is still settling while the rest of the vector's is not.

The bootstrap bounds the cost; it does not remove it. This is stated rather than smoothed over because the specification states it (`cav:limitation:late-standardisation`) and because the mitigation is configuration and patience rather than a guarantee. A reader deciding when to register a Sentinel should know that the answer *early* is not merely conventional.

**Caveat (The width accounting defines the vector rather than deciding about it)** · `cav:vector:width-accounting`

The block-by-block width accounting — how wide each block is, and the formula that sums them into the vector's dimension — is not in this record. It is the specification's (`tab:feature:dimension-formula`), and the per-block widths it composes are stated there as well (`rem:signal:shape-widths`).

The omission is the standing rule applied, not an oversight. Width accounting describes what the vector *is*; this record decides how it is ordered, resolved, timed and scaled. Restating the arithmetic here would give the corpus two places to keep a formula correct, which is the failure the record set was derived to remove, and the one this record would be least excused for reintroducing given that its first decision is that the layout has exactly one authority.
