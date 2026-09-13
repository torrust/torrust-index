## Part (The Data Model) · `part:spec:data-model`

Part II defines what the Core sees. Three registries govern the shape of the feature vector, each Sentinel's report is compressed to a fixed width, a key space is partitioned into ranges the models can weigh, the host's own signals are declared, and the blocks are assembled into one vector with a map over it. Nothing here references actions, channels, posture, or cost.

### Chapter (Registries and Lifecycle) · `chap:spec:registries-and-lifecycle`

The chapter makes the dynamic-registry assumption (`assum:constraint:dynamic-registries`) operational. Three registries — Sentinels, outcome axes, identity dimensions — each admit runtime addition and removal, and each discharges the change through the Gaussian algebra rather than through retraining. The chapter states what is declared at registration, what each lifecycle event does to which model set, and what happens when several events arrive together.

#### The Sentinel registry · `sec:registry:sentinel`

The division carries no material of its own. It collects the registration record, the lifecycle operations, the two protocols, and the coverage, hibernation and scale statements that belong to Sentinels.

**Schema (Sentinel registration record)** · `schema:registry:sentinel-record`

A Sentinel is registered with five fields: an identifier, a name, a description, a declared domain width, and its coordinate semantics.

| Field | Carries |
| --- | --- |
| Identifier | The registry key, assigned by the host and unique across the registry |
| Name | The identifier used in declared cross-Sentinel interaction templates; unique across currently registered Sentinels |
| Description | What this Sentinel watches, in the host's own words |
| Declared domain width | The width $N_s$ of the Sentinel's native coordinate space $[0, 2^{N_s})$ (`def:encoding:domain-width`) |
| Coordinate semantics | What the coordinate space is, how it is encoded, and whether hierarchical structure is asserted |

The coordinate semantics carry three things in turn: a description of the domain, an encoding description — native hierarchical with the meaning of depth stated, a space-filling curve with its type and source dimensions (`tab:encoding:curve-guidance`), a categorical encoding with its taxonomy, or some other encoding the host describes — and the host's assertion that shared prefixes imply shared context.

The Core records the coordinate semantics and does not act on them. It does not refuse a registration that asserts no hierarchical structure, and it does not alter extraction or model weights on the strength of an encoding description. The three declarations that carry the encoding contract — description, declared width, and coordinate semantics — exist for the three reasons the contract gives (`req:encoding:host-contract`): to force engagement with the encoding question before the Sentinel exists, to make the deployment's assumptions auditable by someone who did not make them, and to give the diagnosis of an uninformative Sentinel somewhere to start.

**Table (Sentinel lifecycle operations)** · `tab:registry:sentinel-operations`

Three operations against the four surfaces a Sentinel touches. Full feature-space models are the operational model, the sister model, and every active outcome-axis prediction model; the fixed-dimension anchor model is neither extended nor marginalised by a Sentinel event.

| Operation | Interpreter | Estimator | Outcome memory | Dimension map |
| --- | --- | --- | --- | --- |
| Add | Create the extractor and the routing indices | Extend every full feature-space model by $r$ dimensions (`thm:gaussian:extension`) | Create the spatial outcome map | Rebuild with a contiguous slot and the instantiated interactions |
| Remove | Destroy the extractor and the routing indices | Marginalise every full feature-space model by $r$ dimensions (`thm:gaussian:marginalisation`) | Drop the spatial map | Rebuild after removing the owned ranges |
| Offline | No extraction this cycle | Occupancy zero, features zero | No update | Unchanged |

Lifecycle operations publish atomically with respect to assessment and label calls: an in-flight assessment reads pre-event state, and one starting after publication reads post-event state. Where marginalisation and time-indexed decay fall in the same call, marginalisation runs first, which preserves the wider removed block for the Schur computation (`alg:gaussian:regularised-schur`).

**Remark (Anomaly dilution)** · `rem:registry:anomaly-dilution`

Registering a Sentinel dilutes established anomaly profiles rather than distorting them. An entity whose other Sentinels have been alarming sees its aggregate anomaly features fall in proportion to one over the number of reporting Sentinels, because the new Sentinel contributes nothing until it has observed anything.

The dilution is conservative — it lowers risk estimates rather than raising them — and it is self-correcting within roughly twenty assessments per entity, as the new Sentinel's own features begin to carry signal. No compensating mechanism is needed, and one would have to be undone later.

**Algorithm (Sentinel registration)** · `alg:registry:sentinel-registration`

On registration, in order:

1. **Validate the declared width.** Reject a declaration of zero, and reject one greater than the internal width (`dec:encoding:fixed-internal-width`). This is the only structural validation the Core performs on a registration; everything else the record declares is an assertion the Core records and does not check (`rem:encoding:unchecked-width`).
2. **Allocate the slot.** Assign the next contiguous block of $q + 1$ indices, the occupancy indicator and the extraction width (`def:extraction:slot`).
3. **Instantiate interactions.** For each applicable template, create the features and assign their indices.
4. **Create the routing indices.** Allocate the per-Sentinel current report index and the spatial outcome index. The declared width is recorded alongside them: it is required to interpret reported cell depths, and it fixes the shift at which the integration layer lifts native coordinates into the internal representation (`alg:encoding:coordinate-lift`).
5. **Extend every full feature-space model.** Extend the mean with zeros and the precision and covariance with the prior at the new block, with zero off-diagonals (`thm:gaussian:extension`).
6. **Extend standardisation.** Append entries at the feature-class priors.
7. **Update the dimension map.**

**Algorithm (Sentinel deregistration)** · `alg:registry:sentinel-deregistration`

On deregistration, in order:

1. **Identify the dimensions to remove.** The Sentinel's slot together with every interaction feature involving it.
2. **Marginalise every full feature-space model.** Apply the regularised Schur complement (`alg:gaussian:regularised-schur`) to the operational model, the sister model, and every active outcome-axis model.
3. **Remove the standardisation entries.**
4. **Destroy the routing indices.** The spatial outcome index's cell state ends with the Sentinel; hibernation keeps the parameter block and never the outcome memory (`alg:registry:hibernation`).
5. **Update the dimension map.**

Deregistration removes the same structure whatever the Sentinel's history (`thm:gaussian:two-level-guarantee`). The exact Schur identity transfers the whole correction into the surviving precision; the specified regularisation attenuates it, and a kept-block fallback discards it.

**Table (Coverage states)** · `tab:registry:coverage-states`

Not every Sentinel produces a report for every observation. A timing Sentinel has nothing until the second observation in a session; a categorical Sentinel has nothing for an observation lacking the attribute. The Core handles this structurally rather than by imputation, through four states.

| State | Occupancy | Features | Meaning |
| --- | --- | --- | --- |
| Not registered | Dimensions absent | — | The source does not exist |
| Active, no batch report | 0 | Zero | The source is offline this cycle |
| Active, no data for the request | 1 | Zero | The source is active but silent for this observation |
| Active, reporting | 1 | Populated | The source is reporting |

The occupancy indicator is what separates the second state from the third. Both present zero features, and the model must be able to tell "this source said nothing" from "this source is not there", because the two license different inferences from the same numbers.

**Algorithm (Hibernation)** · `alg:registry:hibernation`

Hibernation is an optional lifecycle state. On deregistration, the Sentinel's parameter block — its sub-vectors of the mean, the relevant rows and columns of the precision and covariance, and its standardisation entries — may be stored against its identifier instead of being dropped. Where the same Sentinel re-registers before the store expires, the block is decayed by the time-indexed rate over the elapsed wall-clock interval and by the label-indexed rate over the labels processed meanwhile, and the models are re-extended with the decayed block rather than with the prior.

Hibernation preserves the Sentinel's self-structure: the parameter relationships among its own $q + 1$ features. Cross-feature relationships — how this Sentinel's features correlated with identity features or with another Sentinel's — are not preserved, because extension always writes zero off-diagonal blocks. Those must be relearned. Nor is the outcome memory preserved: the store holds parameters and standardisation entries, and a cell's accumulated averages are decay-only state that ends with the entity that fed them. The marginalisation path of (`alg:registry:sentinel-deregistration`) remains the fallback wherever the stored block has been reclaimed or the Sentinel's configuration has changed.

**Definition (No online Sentinel)** · `def:registry:no-online-sentinel`

Where no Sentinel has data for a request, every Sentinel slot and every aggregate feature is zero. Roughly forty per cent of the feature vector remains nonzero — the identity block, the signal block, and the bias — and the posterior reflects the rest correctly as high uncertainty rather than as evidence of normality.

The condition is reported rather than inferred. The assessment carries a flag saying that no Sentinel was online, so a host reading a low risk estimate can tell an estimate made from measurement from one made in its absence.

**Table (Sentinel count and label rate)** · `tab:registry:sentinel-scale`

The architecture imposes no ceiling on the number of registered Sentinels. The practical limit is convergence: each Sentinel adds dimensions, and dimensions need labels. Comfortable counts by label rate:

| Label rate | Comfortable count | Rationale |
| --- | --- | --- |
| 1,000 per day | 15–20 | Convergence within two to three days |
| 200 per day | 8–12 | Convergence within one to two weeks |
| 50 per day | 3–6 | Convergence within two to four weeks |

The figures are guidance rather than limits. A deployment that registers more Sentinels than its label rate supports is not in error; it is slow, and the slowness is visible in the posterior's uncertainty rather than hidden in it.

#### The outcome axis registry · `sec:registry:outcome-axis`

The division carries no material of its own. It collects what an outcome axis is, the registration record, the lifecycle operations and protocols, partial reporting, scaling, hibernation, and the spatial-feature policy.

**Definition (Outcome axis)** · `def:registry:outcome-axis`

An outcome axis is a secondary outcome quantity the host observes at label time and declares to the Core: financial loss, investigation depth, an external confidence score, a compliance metric, any domain scalar. Each registered axis receives its own Bayesian prediction model at the current full dimension, and each axis's outcome history feeds back as features into the risk models and into every other axis's model.

The Core does not interpret any axis (`assum:constraint:outcome-axes`). It imposes no assumption about what an axis means, which direction of it is good, or how a prediction should influence a decision. It predicts, reports calibrated uncertainty, and says nothing further. This is the measure-not-decide principle (`prin:principle:measure-not-decide`) applied to a quantity the Core has no way to understand: interpreting an axis would require domain knowledge the Core does not have and cannot acquire from the axis's values.

**Schema (Outcome axis registration record)** · `schema:registry:axis-record`

An axis is registered with an identifier, a name unique across currently registered axes, a description, a training-eligibility mode, an initial compression scale, a forgetting rate, and a spatial-feature policy.

| Field | Carries |
| --- | --- |
| Identifier | The registry key |
| Name | Unique across currently registered axes |
| Description | What the axis measures, in the host's own words |
| Eligibility | Train on unconfounded labels only, as the sister model does, or on all labels |
| Initial compression scale | The scale at which the axis's values are compressed into a bounded training target |
| Forgetting rate | The axis model's label-indexed forgetting rate |
| Spatial-feature policy | Whether per-Sentinel spatial outcome features are carried for this axis (`disc:registry:spatial-policy`) |

Axes are maintained in registration order, and an axis's zero-indexed position in that order is its index. The dimension map uses that index, and no other name, to locate the axis's features within each identity dimension's block, so the registration order is part of the record rather than an artefact of it.

**Table (Outcome axis lifecycle operations)** · `tab:registry:axis-operations`

Three operations across the three surfaces an axis touches. Full feature-space models here are the operational model, the sister model, and every *other* active axis model; the anchor is untouched.

| Operation | Interpreter | Estimator | Outcome memory |
| --- | --- | --- | --- |
| Add | Add per-axis features to every registered identity dimension, and to every Sentinel extractor where the spatial policy is enabled | Extend the full feature-space models by $r_a$ dimensions and create the axis model | Add per-axis running averages to every cell, where spatial |
| Remove | Remove the axis's features from the extractors | Marginalise the surviving full feature-space models by $r_a$ dimensions and destroy the axis model | Drop the per-axis averages, where spatial |
| Value absent for a label | — | The axis model does not update; its existing features are unchanged | No update for that axis |

**Algorithm (Outcome axis registration)** · `alg:registry:axis-registration`

On registration of an axis, in order:

1. **Compute the new dimensions.** Where the spatial policy is enabled, each active Sentinel gains two outcome-memory features for the axis; where it is disabled, none. Each registered identity dimension gains three. The count is $r_a = 2n \cdot \mathbb{1}[\text{spatial}] + 3D$ for $n$ Sentinels and $D$ identity dimensions, plus any declared interaction features.
2. **Extend every full feature-space model** by $r_a$ dimensions, with the prior at the new block and zero off-diagonals (`thm:gaussian:extension`).
3. **Create the axis prediction model** at the full dimension after extension, at the prior.
4. **Initialise per-axis cell state** in the identity layer always, and in the spatial outcome memory where the policy is enabled. Every new running average starts at zero.
5. **Extend standardisation.** Append entries at the feature-class priors.
6. **Initialise the compression scale** from the record.
7. **Update the dimension map.**

Every vector the map governs is extended by the same $r_a$ at the same event, whatever the identity dimension count. The map, the model parameters, and the standardisation statistics describe one dimension at every published version (`inv:dimension:version-consistency`), and a registration that extended some of them and not others would leave the three disagreeing with nothing to detect it.

**Algorithm (Outcome axis deregistration)** · `alg:registry:axis-deregistration`

On deregistration of an axis, in order:

1. **Identify the dimensions to remove.** The axis's three features in each registered identity dimension, its two outcome-memory features per Sentinel where the spatial policy was enabled, and any interaction features.
2. **Marginalise every surviving full feature-space model** by the regularised Schur complement (`alg:gaussian:regularised-schur`).
3. **Destroy the axis prediction model.**
4. **Drop the per-cell running averages.** The axis's rows leave every outcome-memory entry together with the features they fed, and nothing is kept for a later return.
5. **Remove the standardisation entries.**
6. **Update the dimension map.**

The averages end with the axis, and hibernation does not reach them: what an archive holds is the entity's own parameter block and its standardisation entries, not the outcome memory (`rem:registry:axis-hibernation`). This is a position rather than an omission. An average that survived the gap would describe a feature population and a standardisation regime that no longer exists once the marginalisation has run — the Schur complement folds the axis's block into the survivors, and step 5 removes the standardisation entries the average was accumulated against — so restoring it would mean reading a number against a vector it was never measured in. A re-registered axis therefore starts cold by design, at the zeroed cell state its registration creates (`alg:registry:axis-registration`), and the warm-up stages are what carry that condition (`tab:warmup:stages`). The cost of the alternative is larger than the cold start it would save: an axis identifier is reusable, and a surviving average would let a new axis read its predecessor's history as its own warm start.

**Requirement (Partial outcome reporting)** · `req:registry:partial-outcome-reporting`

Not every label carries a value for every axis. Where an axis's value is absent from a label, three things follow and no others: that axis's prediction model does not update, that axis's per-cell running averages do not update, and that axis's existing features remain in the vector at their current values.

No information is lost by the omission. The other models continue to see whatever outcome history the axis has accumulated, and the axis model simply learns nothing from a label that told it nothing. The absent value is not imputed, and a label reporting no axis values at all is a valid label for every model except the axis models.

**Table (Outcome axis scaling)** · `tab:registry:axis-scaling`

Each axis adds features and one model. The marginal feature count is $2n \cdot \mathbb{1}[\text{spatial}] + 3D$; at eight Sentinels and two identity dimensions:

| Spatial policy | Features per axis | Per-label cost per axis |
| --- | --- | --- |
| Enabled | $2n + 3D = 22$ | $O(p^2)$ |
| Disabled | $3D = 6$ | $O(p^2)$ |

The per-label cost of an axis model is quadratic in the dimension regardless of the policy, so the count of models rather than the growth of the dimension is the dominant driver. The policy reduces the dimension the cost is quadratic in:

| Axes | Policy | $\Delta p$ | $p$ | Per-label operations | Against no axis |
| --- | --- | --- | --- | --- | --- |
| 0 | — | 0 | 616 | 0.76M | 1.00× |
| 1 | enabled | 22 | 638 | 1.22M | 1.61× |
| 3 | all enabled | 66 | 682 | 2.33M | 3.07× |
| 5 | all enabled | 110 | 726 | 3.69M | 4.86× |
| 5 | all disabled | 30 | 646 | 2.92M | 3.85× |
| 10 | all enabled | 220 | 836 | 8.38M | 11.05× |
| 10 | all disabled | 60 | 676 | 5.48M | 7.22× |

There is no architectural ceiling on the axis count. The binding constraint is convergence: each axis model needs labels that both are eligible and carry the axis's value, and that is a scarcer thing than a label.

**Remark (Outcome axis hibernation)** · `rem:registry:axis-hibernation`

An axis hibernates under the protocol Sentinels hibernate under (`alg:registry:hibernation`), with the same preservation scope: the axis's own parameter block is decayed by elapsed time and by intervening labels and re-extended on re-registration, and its cross-feature relationships are relearned. The scope being the same is also what settles the outcome memory: the per-cell running averages dropped at deregistration (`alg:registry:axis-deregistration`) are not among what returns, and a hibernating axis comes back to a zeroed cell state exactly as a fresh one does. Nothing about an axis changes the protocol, which is why it is stated once and referred to here rather than written twice.

**Discussion (When to enable spatial features)** · `disc:registry:spatial-policy`

The spatial policy governs whether the axis carries two per-Sentinel features drawn from the spatial outcome memory (`tab:extraction:ledger-features`). Those features say: requests routed through this cell in this Sentinel have tended to produce axis values of this magnitude.

Enable the policy where the axis value correlates with where the request falls in a Sentinel's coordinate space — financial loss, whose patterns concentrate spatially; response latency, which is network-dependent; jurisdiction-dependent policy metrics. Disable it where the value depends on the entity rather than the position — investigation depth, an external confidence score, a satisfaction measure. When uncertain, enable: the model learns to downweight an uninformative feature, and the cost of carrying one is dimensions rather than error.

Disabling is an optimisation for deployments carrying many axes, where the dimension saving is material. It has a second consequence worth stating: an axis registered without spatial features holds no per-cell state in the outcome memory, and is therefore outside the contamination pathway that reaches spatially-enabled axes (`disc:valence:axis-pathway`).

#### The identity dimension registry · `sec:registry:identity-dimension`

The division carries no material of its own. It collects what an identity dimension is and the two lifecycle protocols; the layer itself is the subject of (`chap:spec:key-space-identity`).

**Definition (Identity dimension)** · `def:registry:identity-dimension`

An identity dimension is a host-declared structured key space over which the Core tracks identity: an organisational hierarchy, a tenant space, a geographic prefix space, a device fingerprint hierarchy. Each registered dimension gets its own graph instance over the declared integer domain, from which the layer selects competitive cells and reports range-membership indicators as features (`mot:keyspace:purpose`).

The registry manages dimensions through the algebra that governs Sentinels and axes, so a dimension may be registered or deregistered at runtime without retraining and without disturbing what the models have learned about anything else.

**Algorithm (Identity dimension registration)** · `alg:registry:dimension-registration`

On registration of a dimension, in order:

1. **Create the graph instance** at the declared budget parameters.
2. **Determine the initial competitive set**, which is empty until observations accumulate and cells promote.
3. **Extend every model but the anchor** by the dimension's competitive indicators — initially none — and by its per-dimension feature block. Where this is the first registered dimension, extend also by the cross-dimension aggregate block (`tab:keyspace:cross-dimension-features`). Then recompute the anchor's projection indices during the map rebuild, so that the anchor's identity features resolve to the current positions of the cross-dimension block: at the first dimension they change from absent to present, and otherwise they may still shift, because the cross-dimension block follows every per-dimension block.
4. **Extend standardisation.** Append entries at the feature-class priors.
5. **Update the dimension map.**

**Algorithm (Identity dimension deregistration)** · `alg:registry:dimension-deregistration`

On deregistration of a dimension, in order:

1. **Identify the dimensions to remove.** Every competitive indicator of the dimension, its per-dimension feature block, and every competitive interaction feature involving it. Where this was the last registered dimension, remove the cross-dimension aggregate block as well.
2. **Gather the removed partition and marginalise every model but the anchor.** The pre-event map supplies the per-dimension range, the dimension's indicator sub-range, and the interaction indices. These are not contiguous in the feature vector, so they are gathered into one removed partition (`def:gaussian:gathered-partition`) and discharged by a single regularised Schur complement per model (`alg:gaussian:regularised-schur`), after which the survivors are compacted into the post-event layout.
3. **Recompute the anchor's projection indices** during the map rebuild. The anchor's identity features become absent where no dimension remains, and otherwise resolve to the shifted cross-dimension positions.
4. **Remove the standardisation entries.**
5. **Destroy the graph instance.**
6. **Update the dimension map.**

The gather is what makes the removal one operation rather than several. A dimension owns three disjoint stretches of the vector, and marginalising them separately would apply three Schur complements where one is exact (`def:gaussian:schur-complement`).

#### Lifecycle interaction · `sec:registry:lifecycle-interaction`

The division carries no material of its own. It collects the serialisation invariant across the three registries and the treatment of compound events.

**Invariant (Model sets under lifecycle events)** · `inv:registry:model-set-serialisation`

Every registry event touches a determinate set of models, and the sets are stated once here. Adding or removing a Sentinel extends or marginalises the operational model, the sister model, and every outcome-axis model. Adding or removing an axis extends or marginalises the operational model, the sister model, and every *other* axis model. Adding or removing an identity dimension extends or marginalises every full feature-space model. The anchor model is in none of these sets: it is fixed-dimensional, and identity and axis events reach it only through its projection indices.

All three registries publish through one protocol, and lifecycle changes are never visible mid-call. Each assessment reads one self-consistent published version, so no assessment can observe a model set part-way through an event — which is what allows the sets above to be stated as facts about the published state rather than as facts about an instant.

**Requirement (Compound events)** · `req:registry:compound-events`

Where several lifecycle events must occur together — removing a Sentinel while registering an identity dimension, say — they are processed sequentially as state updates, each seeing the state the previous one left.

The order does not affect the final posterior, because extension and marginalisation commute on disjoint index sets — extension (`thm:gaussian:extension`) adds a block the other's removed partition does not meet, and marginalisation (`thm:gaussian:marginalisation`) removes one the other's extension did not touch. Order does affect the intermediate work: marginalisation should precede extension where both apply to the same model, so that the extension's cost is paid at the smaller dimension rather than the larger. The commutation is a property of the algebra and not of the implementation, and an implementation that processes compound events in some other order owes the same final posterior.
