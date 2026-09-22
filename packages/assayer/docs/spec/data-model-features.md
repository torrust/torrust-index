### Chapter (The Feature Vector) · `chap:spec:feature-vector`

The chapter assembles Part II. Everything the preceding chapters define — each Sentinel's fixed-width extraction, each identity dimension's competitive features, the host's declared signals — arrives here as a block of one vector, augmented with cross-Sentinel summaries and with products of pairs. The chapter states the vector's structure and its dimension, the aggregate block, the template system that generates the products, the map that gives every position a name, and the standardisation that puts every position on a comparable scale.

**Definition (The structure of the feature vector)** · `def:feature:vector-structure`

The feature vector is the concatenation of eight block types in a fixed order: the bias, the aggregate block, one block per identity dimension, the cross-dimension block, the signal block, one slot per registered Sentinel, the interaction block, and the competitive indicators.

$$\phi = \bigl[\underbrace{1}_\text{bias} \;\big|\; \underbrace{\phi_\text{agg}}_{p_\text{agg}} \;\big|\; \underbrace{\phi_{\text{dim}_1}}_{8+3m} \;\big|\; \cdots \;\big|\; \underbrace{\phi_{\text{dim}_D}}_{8+3m} \;\big|\; \underbrace{\phi_\text{cross}}_{8} \;\big|\; \underbrace{\phi_\text{sig}}_{p_\text{sig}} \;\big|\; \underbrace{\text{slot}_{s_1}}_{q+1} \;\big|\; \cdots \;\big|\; \underbrace{\text{slot}_{s_n}}_{q+1} \;\big|\; \underbrace{\phi_\text{int}}_{p_\text{int}} \;\big|\; \underbrace{\phi_\text{id-comp}}_{C}\bigr]$$

Here $D$ is the number of registered identity dimensions, $n$ the number of registered Sentinels, $m$ the number of active outcome axes, $m_s$ the number of those with spatial features enabled, and $C$ the total competitive indicator count across every dimension. The cross-dimension block is present exactly when $D$ is positive.

The order is not arbitrary. The bias and the aggregate block precede every dynamic block, which is what makes their positions stable under all lifecycle events; the competitive indicators, the most volatile block of all, sit last, where their coming and going disturbs the fewest positions.

**Table (The dimension formula)** · `tab:feature:dimension-formula`

The total dimension is the sum of the block widths.

$$p = 1 + p_\text{agg} + (8 + 3m)D + 8\mathbb{1}[D > 0] + p_\text{sig} + n(61 + 2m_s) + p_\text{int} + C$$

| Block | Width | What changes it |
| --- | --- | --- |
| Bias | 1 | Nothing |
| Aggregate | $p_\text{agg} = 15$ | Nothing (`tab:feature:aggregate-block`) |
| Identity per-dimension | $8 + 3m$ each | Identity dimension or outcome axis lifecycle (`tab:keyspace:dimension-features`) |
| Identity cross-dimension | $8$ when $D > 0$ | The first and last identity dimension (`tab:keyspace:cross-dimension-features`) |
| Signal | $p_\text{sig}$ | Nothing after construction (`req:signal:schema-fixed`) |
| Sentinel slot | $q + 1 = 61 + 2m_s$ each | Sentinel lifecycle; outcome axis lifecycle changing $m_s$ (`def:extraction:slot`) |
| Interactions | $p_\text{int}$ | Every event the declared templates expand over |
| Competitive indicators | $C$ | Competitive set restructuring (`def:keyspace:competitive-set`) |

Three of the eight are fixed for the life of the deployment and five move. Each of the five moves under a named lifecycle event and under no other, which is what makes the dimension a derived quantity rather than a configured one: no deployment sets $p$, and every change to it is the arithmetic consequence of a registration, a deregistration, or a restructuring.

#### Aggregate features · `sec:feature:aggregate`

The division carries no material of its own. It collects the purpose of the aggregate block, the fifteen features it holds, and the recalibration of the thresholds five of those features are counted against.

**Motivation (Why the aggregate block exists)** · `mot:feature:aggregate-purpose`

Per-Sentinel slots record what each specialist reports. The aggregate block records what the fleet reports together: how loud the loudest alarm is, how far the Sentinels agree, how many kinds of anomaly are active at once, and how much of the measurement surface is online at all. These are not recoverable from the slots by a linear model, because a maximum and a standard deviation across slots are not linear in the slots, and a model given only the slots would have to approximate them from weights that also have to serve every other purpose.

The block has fixed width whatever the number of Sentinels or axes, so its positions never move, and it is populated whenever any Sentinel reports at all. That combination makes it the fastest-converging evidence the Core holds: it needs no per-cell outcome history and no per-Sentinel history, only a report, which is why it is one of the pathways that carry detection while the slower layers are still learning (`tab:ledger:low-maturity-detection`).

**Table (The aggregate feature block)** · `tab:feature:aggregate-block`

Fifteen features summarise the current batch across every reporting Sentinel.

| Feature | Width | Carries |
| --- | --- | --- |
| Maximum cell-level maximum z-score across active Sentinels | 1 | The loudest alarm anywhere |
| Mean cell-level maximum z-score | 1 | The average alarm |
| Standard deviation of the cell-level maximum z-score | 1 | Agreement against disagreement |
| Per-axis maximum z-score across Sentinels | 4 | The worst alarm of each kind |
| Per-axis concordance | 4 | The fraction of reporting Sentinels above that axis's threshold |
| Cross-axis product | 1 | Novelty against displacement, compounded |
| Axis breadth | 1 | How many kinds of anomaly are above threshold |
| Coverage | 1 | Reporting Sentinels over registered Sentinels |
| Maximum cumulative sum across every Sentinel and axis | 1 | The worst sustained drift |

The four concordance features and the breadth feature are the only ones read against a threshold rather than reported directly, which is why the thresholds need a calibration of their own (`alg:feature:concordance-recalibration`). The remaining ten are order statistics over the batch and carry no state between assessments.

**Algorithm (Concordance threshold recalibration)** · `alg:feature:concordance-recalibration`

The per-axis threshold is the eightieth percentile of the axis's recent sub-scores, so that concordance reports how many Sentinels are unusually loud for this deployment rather than how many exceed a constant chosen elsewhere.

1. **Accumulate**, on the assessment path, one sub-score per axis per reporting Sentinel into a rolling window holding the last ten thousand.
2. **Recompute** every thousand assessments, taking each axis's threshold as the eightieth percentile of its window.
3. **Publish** the thresholds as state of their own, outside the model snapshot.

Thresholds start at one half and move only at a recomputation. They are held apart from the model snapshot deliberately: a threshold change alters what the concordance features mean but not what any model parameter means, so it triggers neither a calibration recomputation nor a drift reset. The accumulation is an auxiliary write on the assessment path (`inv:runtime:enumerated-writes`) and nothing on that path reads a model to perform it.

#### The interaction template system · `sec:feature:interactions`

The division carries no material of its own. It collects the five template types, the default set built from them, their convergence behaviour, what lifecycle events do to them, and the interaction dimension the default set implies.

**Motivation (Interactions as pairwise products)** · `mot:feature:interaction-overview`

The core models are linear in the feature vector (`cav:limitation:linear`), so a conjunction — this Sentinel is loud *and* this entity is already suspect — cannot be represented by weights on the two features alone. An interaction feature is the product of two features already in the vector, which gives the model exactly the conjunctions it is declared to want and nothing else.

Interactions are declared as templates rather than as features because the features they stand for do not exist yet. A template names two operands symbolically and a rule for expanding over the current registries; the expansion produces the features. Five types differ in what they expand over, and therefore in how their count scales and how fast each feature converges: a template expanding over nothing produces one feature that every labelled request touches, and a template expanding over pairs of Sentinels produces many features that few labelled requests touch. The choice among them is a choice about where to spend model capacity.

**Definition (The per-Sentinel template)** · `def:feature:template-per-sentinel`

A per-Sentinel template names one feature within a Sentinel slot and one context feature — identity, signal, or aggregate — and expands to one feature per registered Sentinel, each the product of that Sentinel's slot feature with the shared context feature. The count grows linearly in the number of Sentinels.

The template's Sentinel operand is written with a quantifier rather than a name, and the quantifier is what binds it to each expansion in turn. This is the type that asks whether a *particular* source's alarm means more when the entity is already suspect than the same alarm from another source would, and it is the cheapest way to ask that question, since the alternative — one declared template per Sentinel — would have to be edited every time a Sentinel is registered.

**Definition (The named-pair template)** · `def:feature:template-named-pair`

A named-pair template names two Sentinels explicitly and expands to exactly one feature, the product of a feature from each. Its count does not grow with the number of Sentinels at all. The feature exists only while both named Sentinels are registered: it is created when the second of the pair registers and destroyed when either deregisters.

This is the type for a conjunction the operator already suspects — two specific sources whose simultaneous alarm is known to mean more than either alone. It buys the cross-Sentinel question at constant cost by requiring the pair to be named in advance, which is the whole of the trade: a named pair costs one dimension and answers one question, and a pair nobody thought to name is not asked about.

**Definition (The wildcard template)** · `def:feature:template-wildcard`

A wildcard template names no Sentinel and expands to one feature for every unordered pair of registered Sentinels, so its count grows as the square of the number of Sentinels. At ten Sentinels a single wildcard template produces forty-five features.

The type is to be used sparingly, and the quadratic count is only half the reason. Each of those forty-five features is the product of two specific Sentinels' values, so it is informative only on requests where both of that pair report — a low rate for most pairs — and the convergence of a feature is governed by how often it is non-zero in labelled data (`cav:limitation:interaction-type-three`). A wildcard template therefore buys a large block of model capacity that takes months to exploit, and it buys it uniformly across pairs, including every pair whose joint behaviour means nothing.

**Definition (The aggregate template)** · `def:feature:template-aggregate`

An aggregate template names one feature of the aggregate block and one context feature, and expands to exactly one feature regardless of how many Sentinels or dimensions are registered. Its count is constant.

This is the fastest-converging interaction type, and for the same reason the aggregate block itself converges fastest: the aggregate operand is populated on every request with at least one reporting Sentinel, so the product is non-zero on a large fraction of labelled requests rather than on the narrow slice where one particular Sentinel or one particular pair happened to report. It asks the cross-source question without naming a source, which is a weaker question than the pair types ask and a very much cheaper one.

**Definition (The competitive template)** · `def:feature:template-competitive`

A competitive template names one context feature and expands to one feature per competitive cell per identity dimension, each the product of that cell's indicator with the context feature. Its count is the total competitive indicator count, so it grows and shrinks with the competitive sets rather than with any registry.

This is the only template type whose expansion changes without a lifecycle event: competitive entry (`alg:keyspace:cell-entry`) adds one feature per template per dimension and competitive exit (`alg:keyspace:cell-exit`) removes one, each under the same extension and marginalisation the indicator itself takes. It asks whether alarm from this particular range means more than the same alarm elsewhere — the per-range refinement of the identity layer's own additive decomposition, learned per cell rather than per dimension.

**Table (The default interaction set)** · `tab:feature:default-interaction-set`

The recommended set declares five per-Sentinel templates, eight aggregate templates, one competitive template per identity dimension, and no wildcard template at all.

| Type | First operand | Second operand |
| --- | --- | --- |
| Per-Sentinel | Cell maximum z-score | Identity maximum suspicion |
| Per-Sentinel | Cell maximum z-score | Identity coverage depth |
| Per-Sentinel | Cell maximum z-score | Identity has-competitive |
| Per-Sentinel | Cell cumulative sum | Identity volatility |
| Per-Sentinel | Cell adverse rate | Cell maximum z-score |
| Aggregate | Maximum alarm | Identity maximum suspicion |
| Aggregate | Maximum alarm | Identity volatility |
| Aggregate | Concordance | Identity coverage depth |
| Aggregate | Coverage | Identity coverage depth |
| Aggregate | Coverage | Maximum alarm |
| Aggregate | Axis breadth | Identity maximum suspicion |
| Aggregate | Cross-axis product | Identity maximum alarm |
| Aggregate | Maximum cumulative sum | Identity maximum alarm |
| Competitive | The cell's indicator | Maximum alarm |

The competitive row is declared once and expands per dimension; the other thirteen are declared once each. Every identity operand is drawn from the cross-dimension aggregate block (`tab:keyspace:cross-dimension-features`), never from a per-dimension block, which is why the per-Sentinel and aggregate template counts do not change when a dimension is registered. The set is a recommendation and not a constraint: the templates are declared by the host at construction, and a deployment that knows its own Sentinels may name pairs the default set cannot know to name.

**Table (Interaction convergence)** · `tab:feature:interaction-convergence`

An interaction feature converges at a rate set by how often it is non-zero in labelled data, so the types converge on quite different timescales.

| Features | Co-occurrence | At two hundred labels a day |
| --- | --- | --- |
| Raw per-Sentinel features | Near certain | About a day |
| Aggregate templates | 5 to 20 per cent | Two to seven days |
| Per-Sentinel templates | 0.5 to 5 per cent | One to eight weeks |
| Competitive templates | 1 to 10 per cent | One to eight weeks |
| Named-pair templates | 0.1 to 1 per cent | Two to twenty weeks |
| Wildcard templates | 0.1 to 1 per cent | Two to twenty weeks |

The ordering is the design. While the pair types are still converging, the faster layers already cover most of what they will eventually cover: raw features carry per-source alarm within days, aggregate templates carry cross-source alarm without source identity within days, and the per-Sentinel and competitive templates carry alarm against identity state within weeks. What the pair types add over all of that is the one thing none of the others can see — a joint signature across two named sources, each of which looks unremarkable alone (`cav:limitation:cross-sentinel-gap`).

**Algorithm (Interactions under lifecycle events)** · `alg:feature:lifecycle-interactions`

Every lifecycle event that changes what a template expands over changes the interaction block, and each change is discharged through the same algebra as the block that caused it.

1. **On Sentinel registration**, add one feature per per-Sentinel template, one per named-pair template whose other Sentinel is already registered, and one per wildcard template for each Sentinel already registered.
2. **On Sentinel deregistration**, remove exactly the features the registration would have added, identified from the resolution map rather than by scanning (`alg:registry:sentinel-deregistration`).
3. **On identity dimension registration or deregistration**, add or remove every competitive template's features for that dimension, in one batch.
4. **On competitive entry or exit**, add or remove one feature per competitive template for the owning dimension.
5. **Re-resolve and recompile** the declared templates against the registries as they now stand.

Declarations are never re-read at runtime and never change; only their expansion does. That is what allows step five to be a total recomputation from the registries rather than a reconciliation against the previous expansion.

**Equation (The default interaction dimension)** · `eq:feature:default-interaction-dimension`

Under the default set, the interaction block's width follows from the template counts and the expansion rules:

$$p_\text{int} = 5n + 8 + C$$

The five per-Sentinel templates contribute one feature each per registered Sentinel, the eight aggregate templates contribute one feature each whatever the registries hold, and the single competitive template per dimension contributes one feature per competitive cell, which totals the competitive indicator count. There is no quadratic term because the default set declares no wildcard template; adding one would add a term in $n(n-1)/2$, and the formula is written to make that addition visible rather than to hide it inside a constant.

#### The dimension map · `sec:dimension:map`

The map is the isomorphism between semantic identity and dense vector index. Every parameter, feature, and standardisation statistic lives at a position; the map is what gives each position a name and each name a position, and it is what turns "marginalise this Sentinel" from a phrase into an index set. The division collects its three invariants, its compaction and rebuild, its layers and conventions, its compilation pipeline, its competitive and anchor structure, and the record that carries it.

**Invariant (Covering)** · `inv:dimension:covering`

Every index below the total dimension is owned by exactly one semantic entity. No index is unassigned and no index is assigned twice.

The invariant is what makes a learned weight interpretable at all: a weight whose index had two owners would be a weight about two things, and the principle that identities survive into the model (`prin:principle:identity-preservation`) would be false at the level where it matters most. It is also what makes reconstruction at label time well defined (`alg:runtime:reconstruction`), where a vector assembled under one map must be read under another: features whose entity has since been deregistered are absent from the current map, features whose entity registered after the assessment are absent from the buffer and take zero, and every index in the reconstructed vector has exactly one source because every index has exactly one owner.

**Invariant (Per-block contiguity)** · `inv:dimension:contiguity`

Each physical block, each slot, and each template expansion group occupies a deterministic contiguous range of indices.

Contiguity is asserted per block and not per lifecycle scope, and the difference is the point. An identity dimension owns a per-dimension block, a run of competitive indicators, and a set of competitive interaction features, and these sit in three separate regions of the vector; the dimension's removed index set is therefore several ranges and not one. What the map must do is identify that set completely, so the removed partition can be formed either by slicing or by a transient gather (`def:gaussian:gathered-partition`). The Schur algebra requires an identifiable partition, never a physically adjacent one, which is precisely why contiguity can be demanded of blocks and not of owners.

**Invariant (Version consistency)** · `inv:dimension:version-consistency`

At every published version, the map, the model parameters, and the standardisation statistics share one dimension and one index semantics. No assessment ever reads a model whose dimension disagrees with the map it is read under.

The invariant is unconditional, and it is unconditional because there is no useful weaker form: a vector standardised under one index semantics and scored under another is not approximately right, it is meaningless position by position. Publication is atomic for exactly this reason (`def:publication:model-snapshot`) — the map and the parameters and the statistics become visible together or not at all. The obligation this places on every lifecycle path is therefore uniform and unforgiving: a path that extends the models by one count and the standardisation vectors by another has broken the invariant at the moment it publishes, and no reader downstream is in a position to notice.

**Algorithm (Compaction and rebuild)** · `alg:dimension:compaction`

After every marginalisation the affected parameters and standardisation vectors are physically compacted: removed rows and columns are excised, and every higher-indexed entry shifts down to close the gap. Extension appends at the end and disturbs nothing. The rebuilt map's ranges therefore correspond exactly to physical positions, and no translation layer stands between them.

The map is rebuilt on every registry lifecycle event and on every competitive restructuring, and on no other event. A rebuild is a total recomputation from the current registries and competitive sets; no prior map state is consulted, and the old map is used only to identify the marginalisation scope before the rebuild begins. Where one restructuring event promotes or demotes several cells, every extension and marginalisation is applied first and one rebuild follows (`req:registry:compound-events`). Rebuild cost is linear in the dimension plus the pair enumeration of any wildcard templates, which is sub-millisecond at the reference configuration and dominated by that enumeration at high Sentinel counts.

**Definition (Block, slot and feature layers)** · `def:dimension:layers`

The map resolves a position at three depths.

| Layer | Answers | Changes on |
| --- | --- | --- |
| Block | Which of the eight block types owns this range | Lifecycle events and restructuring |
| Slot | Which Sentinel, dimension, cell, or template expansion owns this sub-range | The same events |
| Feature | Which feature sits at which offset within a slot | Nothing; it is derived |

The feature layer is derived rather than stored, because the extraction layout and the per-dimension layout are fixed by specification: the map holds a slot's base address and the reader adds a known offset. Two such offsets are named and held, both slot-relative under the slot convention (`conv:extraction:offsets`). The base outcome-memory offset is the position within a slot at which the Ledger features begin, invariant at fifty-seven under every lifecycle event, since it depends only on the fixed extraction groups preceding them. The per-axis outcome-memory offsets give, for each spatially-enabled axis, where that axis's pair of Ledger features begins; axes absent from it contribute no per-Sentinel Ledger features and do not affect the slot width.

**Convention (The per-dimension axis index)** · `conv:dimension:axis-index`

Within an identity dimension's block, the three features of axis $a$ (`tab:keyspace:outcome-state`) begin at offset $8 + 3\,\text{index}(a)$, where the index is the axis's zero-based position in the outcome axis registry's order.

The registry keeps axes in registration order, and deregistration shifts every later axis down by one (`alg:registry:axis-deregistration`). That order is the authoritative definition of the index, and it is authoritative rather than merely conventional because nothing else records it: the offset is computed from the order at every use site, so a second ordering anywhere would not disagree loudly, it would silently read one axis's history as another's. The lifecycle sequence is fixed accordingly — the pre-event map identifies the indices to remove, the Schur complement is applied at those indices, and the new map is built from the post-event registry.

**Algorithm (The interaction compilation pipeline)** · `alg:dimension:compilation-pipeline`

Declared templates reach the assessment path through three stages, each serving a different consumer.

1. **Declaration.** The symbolic templates as the host wrote them: a type, and two operand selectors naming a slot feature, an aggregate feature, an identity feature, a declared signal, or the owning competitive indicator. Fixed at construction, and retained in the map so a snapshot can rebuild the later stages without the host.
2. **Resolution.** One interaction identity per expanded feature, each mapped to its output index. The identity records what the feature is *of* — which template, which Sentinel or pair, which dimension and cell — so a deregistration derives the affected indices from the entity it removes rather than by scanning the interaction range.
3. **Compilation.** A flat list of index triples, one per feature: output, first operand, second operand. This is what feature assembly iterates (`alg:runtime:assessment-pipeline`), with no name lookup and no template interpretation left in it.

Resolution and compilation are recomputed on every map rebuild; declaration is not. The three stages exist because audit wants names, lifecycle wants identities, and the assessment path wants neither.

**Definition (The competitive range and its ordering)** · `def:dimension:competitive-range`

Competitive indicators occupy the final block of the vector, after the interactions. The map holds the block's extent and a per-dimension, per-cell lookup into it that is a bijection between the cells of every competitive set and the indices of the block.

Within the block, indicators are grouped by identity dimension in registration order, and within a dimension they are ordered by time of competitive entry — the sequence number of the restructuring event that promoted the cell, with ties broken by the cell's canonical dyadic key of depth and start. Surviving cells keep their order across rebuilds. The ordering is deterministic and not spatially meaningful: it records competitive history, not the geometry of the key space. Determinism is the whole requirement, and it is a real one, because an ordering that varied between rebuilds would move learned weights between cells without any event having occurred.

**Definition (The anchor projection)** · `def:dimension:anchor-projection`

The anchor model reads a fixed fifteen-position vector (`def:risk:anchor-model`), and the projection is what holds that width fixed while the vector underneath it changes. Thirteen positions are gathered directly and two are computed.

| Positions | Source |
| --- | --- |
| 0 to 6 | The bias and aggregate blocks; always present |
| 7 to 12 | The cross-dimension block; absent, and read as zero, when no dimension is registered |
| 13 | The maximum over the four per-axis maximum z-score positions |
| 14 | The logarithm of one plus the reporting Sentinel count |

Positions 0 to 6 precede every dynamic block and their indices never move. Positions 7 to 12 move whenever the preceding per-dimension blocks change width, and are toggled between present and absent by the first and last dimension registration; the rebuild updates them. No lifecycle event ever extends or marginalises the anchor model. The projection absorbs the change instead, which is the entire mechanism by which the anchor stays a fixed model over a moving vector.

**Schema (The dimension map record)** · `schema:dimension:map-record`

The map is one record, carrying the total dimension and the structure of every layer beneath it.

| Group | Carries |
| --- | --- |
| Dimension | The total width the parameters and statistics must agree with |
| Block layer | The bias index and the extent of each of the eight blocks, per Sentinel and per dimension where the block is one of many |
| Slot layer | The uniform Sentinel slot width, and the two derived outcome-memory offsets (`def:dimension:layers`) |
| Interaction pipeline | The declarations, the resolution map, and the compiled triples (`alg:dimension:compilation-pipeline`) |
| Competitive | The per-dimension, per-cell index lookup (`def:dimension:competitive-range`) |
| Anchor | The projection (`def:dimension:anchor-projection`) |

Every map in the record is insertion-ordered, and the requirement is not a preference. Insertion order is registration order for Sentinels, axes and dimensions, and dimension-grouped entry order for competitive cells; an unordered map would satisfy covering and fail contiguity, because contiguity is a claim about a deterministic order of entries within a block. The record is authoritative at the block and slot level and deliberately not below it: positions within a slot come from the fixed extraction layout, positions within a dimension block from the axis-index convention (`conv:dimension:axis-index`), and positions within the signal block from the schema's declaration order (`req:signal:schema-fixed`).

#### Online standardisation · `sec:standardisation:online`

The division carries no material of its own. It collects why the features are standardised, where the statistics live and what feeds them, the label-time procedure, the priors a new position starts at and the rule assigning them, the two initialisation mechanisms, and the mismatch that re-standardisation costs.

**Definition (Why standardisation is needed)** · `def:standardisation:purpose`

Each position is centred and scaled by a running mean and variance before any model reads it:

$$\hat{\phi}_j = \frac{\phi_j - \bar{\mu}_j}{\sqrt{\bar{v}_j} + \varepsilon}$$

The models take an isotropic prior — one precision on every position — and an isotropic prior is a statement about scale as much as about belief. Raw, the vector mixes z-scores at natural scale near one, log-scaled features near three to six, and cumulative sums ranging over tens; one prior precision across those would be simultaneously far too tight for the large-scale positions and far too loose for the small-scale ones, and the model would spend its early labels correcting an artefact of units. Standardisation makes the single prior defensible by making the positions comparable.

**Requirement (Where the statistics live)** · `req:standardisation:timing`

The standardisation statistics are part of the published snapshot (`def:publication:model-snapshot`). An assessment acquires one immutable coordinate system, scores against it, and mutates no statistic in it. Three mechanisms feed later snapshots, and all three publish through the model owner:

1. **The cold-start prior-mass ramp** (`alg:standardisation:batch-initialisation`) replaces the cold base moments with accepted raw assessment-time vectors, in equal increments over a finite horizon.
2. **Per-Sentinel bootstrap** (`alg:standardisation:sentinel-bootstrap`) accumulates raw slot values for one new Sentinel and blends them in when complete.
3. **Exponential tracking** advances the working copy at the label-time standardisation step and nowhere else, once cold standardisation is in service.

The first two are observation-authorised. They observe on the assessment path, and that observation is a bounded enqueue or an auxiliary-accumulator write (`inv:runtime:enumerated-writes`) rather than an update to anything an outcome taught. An accepted cold observation may publish only after the assessment that supplied it has fixed its snapshot, so what it moves is a later coordinate system and never the one its own request was answered in. The tracking average is indexed by labels and carries no time-indexed decay: a deployment that stops receiving labels stops moving its coordinate system, which is the correct behaviour for a coordinate system. It is inactive while the cold ramp is waiting or transitioning.

**Algorithm (The label-time standardisation procedure)** · `alg:standardisation:label-time-procedure`

At label time, in order:

1. **Read** the current statistics from the working copy.
2. **Clip** for the statistics update only: where a position deviates from its mean by more than ten standard deviations, the clipped value is used for the update and the unclipped value for the model.
3. **Standardise** the raw features.
4. **Update the models** on the standardised vector (`alg:runtime:update-path`).
5. **Update the statistics** at rate $\gamma_\text{std} = 0.9998$, the mean toward the clipped value and the variance toward its squared deviation from the previous mean.
6. **Apply the variance floor.**
7. **Publish.**

Steps 5 to 7 — and with them the clip at step 2, which exists only to feed them — run if and only if the full-vector standardisation phase is `InService` (`alg:standardisation:batch-initialisation`). In `WaitingForInit` and `Transitioning` the label reads the published coordinate system and does not also move it: the cold observation ramp is the sole owner of full-vector standardisation until it reaches its horizon, and a label advancing the average underneath it would put a label's authority into a transition an observation's authority owns. Steps 1 to 4 run in every phase, so the models learn from the first label onward whatever the coordinate system is doing.

The order carries two decisions. Models update before the statistics, so a label is scored against the coordinate system its prediction was made in rather than one it has already moved. And the clip applies to the statistics alone: an extreme observation is evidence for the model and contamination for the running variance, and treating it as both would let one outlier widen every subsequent standardisation. The bias position is never standardised.

**Table (Feature-class priors)** · `tab:standardisation:class-priors`

Every position starts at the prior mean and variance of its class.

| Class | Prior mean | Prior variance |
| --- | --- | --- |
| Bias | — | — |
| Z-scores | 0 | 1 |
| Cumulative sums | 2 | 25 |
| Rates and fractions | 0.3 | 0.05 |
| Log-scaled | 3 | 4 |
| Binary | 0.5 | 0.25 |
| Hashed categorical | 0 | 0.25 |
| Cyclic | 0 | 0.5 |
| Interaction products | 0 | 1 |
| Occupancy indicators | 0.5 | 0.25 |
| Competitive cell indicators | 0.5 | 0.25 |
| Identity measurement alarm | 0 | 1 |
| Identity measurement suspicion | 0.1 | 0.05 |
| Identity measurement volatility | 0.1 | 0.05 |
| Cross-dimension maximum aggregates | 0 | 1 |
| Cross-dimension binary | 0.5 | 0.25 |
| Outcome axis average, compressed | 0 | 0.25 |
| Outcome axis average, raw | 0 | 1 |
| Outcome axis stability | 0.1 | 0.05 |

Nineteen classes, and the values are approximations of the settled distribution rather than guesses: a binary indicator at even odds has mean one half and variance one quarter exactly, and a z-score is defined to have mean zero and unit variance. At cold start they carry the whole of the standardisation mass; each accepted observation retires exactly $1/N_\text{init}$ of it, and at the ramp's horizon they carry none (`alg:standardisation:batch-initialisation`). Once the phase is in service the label-indexed average tracks later distribution change at the specified rate, and its half-life — some three thousand five hundred labels — describes that continuing adaptation rather than the retirement of the priors, which is finished before the first in-service label.

**Requirement (Feature-class assignment)** · `req:standardisation:class-assignment`

Every position is assigned a class, and the assignment is derived from three authorities and configured by nobody. The dimension map assigns the block and the slot; the fixed extraction layout assigns the class within a Sentinel slot; the signal schema assigns the classes of the signal block, expanded from each declared shape (`schema:signal:shapes`).

The rule is stated because the priors table is worthless without it. A table of class priors and no assignment rule leaves each new position's class to whichever code path creates it, and two paths creating the same kind of position — a lifecycle extension and a batch reset, say — would be free to disagree without contradicting anything written down. Deriving the class from the map rather than storing it also means a position cannot carry a stale class: the class is recomputed wherever the map is, so a class and an index never drift apart.

**Algorithm (Batch initialisation)** · `alg:standardisation:batch-initialisation`

Cold full-vector standardisation is a finite prior-mass ramp over the first $N_\text{init}$ accepted raw assessment vectors (`tab:config:standardisation`).

1. **Start.** Cold construction publishes the feature-class prior moments (`tab:standardisation:class-priors`), the phase `WaitingForInit`, and an accepted-observation count of zero.
2. **Offer after acquisition.** An assessment assembles its raw vector after acquiring its model snapshot, and offers that vector without blocking (`inv:guarantee:non-blocking`). It goes on to standardise and score against the snapshot it already holds.
3. **Accumulate.** The model owner advances a numerically stable empirical mean and variance over the accepted vectors, skipping non-finite entries element by element, and resets at the new width if the dimension changes.
4. **Mix.** At accepted count $n$, write $a_n = n/N_\text{init}$ for the retired share, $(\mu_{0,j}, v_{0,j})$ for the base moments and $(\mu_{e,j}, v_{e,j})$ for the empirical moments of the accepted sample. Every non-bias position publishes

   $$\bar{\mu}_j = (1 - a_n)\,\mu_{0,j} + a_n\,\mu_{e,j}$$

   $$\bar{v}_j = \max\Big(v_\text{floor},\;(1 - a_n)\,v_{0,j} + a_n\,v_{e,j} + a_n(1 - a_n)\,(\mu_{e,j} - \mu_{0,j})^2\Big)$$

   The phase is `Transitioning` while $0 < n < N_\text{init}$, and the bias position takes no part.
5. **Publish progress.** Every accepted advance publishes one immutable model snapshot, increments its snapshot version, and carries the accepted count. A command channel too full to take the observation skips it, reports that load condition in band, and increments nothing.
6. **Complete.** At $n = N_\text{init}$ the prior mass is zero, the published moments are exactly the empirical moments of the accepted sample, the phase becomes `InService`, and the completion event is emitted.
7. **Persist.** A checkpoint carries the phase, the count, the base moments and the empirical sufficient statistics, and a restore resumes the ramp at the count it stopped at.

The third term of the variance is the between-population variance, and it is published rather than dropped. A variance mixed without it would be neither the prior's nor the sample's: it would understate the spread by exactly the amount by which the two populations disagree about the mean, and it would understate it worst in the middle of the ramp, where $a_n(1 - a_n)$ is largest and where a reader has least other evidence about which coordinate system a result came from. At the horizon that term and the prior weight are both zero, so the endpoint is the empirical moments exactly — not within a tolerance, and with no rate left running underneath them.

The accumulation is a numerically stable one-pass update rather than a sum of squares, and it costs storage linear in the dimension for the length of the ramp. The reset on a dimension change is the only sound response available: a part-accumulated vector at one width says nothing about a position that did not exist when accumulation began, and blending the two would attribute one position's distribution to another's index. A lifecycle change therefore never applies an old-layout observation to a new layout — it discards the incompatible sample and rebases the ramp on what the lifecycle published (`req:standardisation:lifecycle-entries`).

Step 2's sample is best-effort under contention: the assessment path may not wait, so an observation the owner cannot take at that moment is skipped rather than waited for. What that costs is exactness of the sample and the pace of the ramp — the moments describe a scheduler-dependent subset of the earliest assessments rather than all of them, so two runs over identical traffic reach the horizon from different subsets and after differently many requests. What it does not cost is a silent gap. A refused observation is reported to the caller in band and advances no count, which is what keeps it inside (`dec:concurrency:no-silent-drop`) rather than exempt from it: under a completion gate the whole accumulation rode on one delivery and a refusal had to be retried or the sample was lost, and under the ramp there is no one-shot left to lose — a refused observation is one share of prior mass retired later, and the host is told which.

**Bound (The re-standardisation mismatch)** · `bound:standardisation:restandardisation`

The pending buffer stores raw features (`def:runtime:pending-entry`), so a label re-standardises them against a later standardisation snapshot than the one its assessment scored in. The mismatch has two components:

$$|\hat{\phi}_j^{(\text{lab})} - \hat{\phi}_j^{(\text{rec})}| \leq \underbrace{\frac{|\Delta\bar{\mu}_j|}{\sqrt{\bar{v}_j}}}_{\text{mean drift}} + \underbrace{\frac{|\phi_j - \bar{\mu}_j| \cdot |\Delta\bar{v}_j|}{2\bar{v}_j^{3/2}}}_{\text{variance drift}}$$

In `InService` the coordinate system moves under the label-indexed average. Write $d_t$ for the clipped observation's displacement and $z_t$ for the stored raw feature's displacement, both in the pre-update coordinate system at step $t$. Its path sensitivity over $\Delta k$ intervening labels is the first-order envelope

$$B_\text{disp} = \sum_{t=1}^{\Delta k}(1 - \gamma_\text{std})\left(|d_t| + \frac{|z_t|}{2}|d_t^2 - 1|\right).$$

This is a local linearisation: for a finite variance change the exact comparison uses the difference of the endpoint transforms, so the sum is not an exact finite-change inequality. With the configured $\gamma_\text{std} = 0.9998$ and clip width ten, the clip-boundary evaluation gives about $0.002\,\Delta k$ from the mean and, coarsely, $0.1\,\Delta k$ from the variance, or about 5.1 over fifty labels. As an illustrative two-standard-deviation variance evaluation, the same sensitivity gives $0.0008\,\Delta k$; retaining the global clip-boundary mean allowance gives about 0.14 over fifty labels. These are worked evaluations of the path sensitivity, not stationary-deployment tolerances or expectations.

During `Transitioning` the movement is not label-indexed at all, and the $\Delta k$ figures do not describe it. What moves the coordinate system is the finite prior-mass ramp: one accepted observation retires exactly $1/N_\text{init}$ of the base mass (`alg:standardisation:batch-initialisation`), and the phase, the accepted count and the snapshot version identify the two coordinate states a mismatch lies between (`schema:output:health-snapshot`). That is a bound on the coordinate system and not on the risk uncertainty: the standardised vector enters directional quadratic forms and a nonlinear probability transform (`def:risk:subspace-blend`), and no scalar factor carries a mass fraction through them.

The large figure needs an extreme feature value and extreme variance evolution at once, and it is bounded in effect as well as in size: the leverage bound (`prop:update:leverage-bound`) limits what any single label's mismatch can do to the posterior, in either phase.

**Requirement (Standardisation under lifecycle events)** · `req:standardisation:lifecycle-entries`

A lifecycle event that adds positions gives each new position its class prior; one that removes positions deletes their entries. The map update and the standardisation update are published together as one atomic lifecycle state change (`inv:guarantee:lifecycle-publication`). If the cold ramp is still active, that publication becomes the ramp's new base: the sufficient statistics gathered under the layout being replaced are discarded, the accepted count resets to zero, and every position that survives the change keeps the moments just published for it rather than reverting to its original prior (`alg:standardisation:batch-initialisation`). The next accepted raw vector must be assembled under the same layout it will be mixed into.

The requirement is one sentence long and it is the sentence the version invariant (`inv:dimension:version-consistency`) rests on. Every lifecycle path must add exactly as many standardisation entries as it adds model dimensions, counted the same way on both sides — per Sentinel and per identity dimension alike, for as many dimensions as are registered. A path that counts one way for the models and another for the statistics publishes a snapshot whose parts disagree, and because both counts are computed from the same registries at the same moment, the two must be derived from a single count rather than written twice.

**Algorithm (The per-Sentinel standardisation bootstrap)** · `alg:standardisation:sentinel-bootstrap`

A Sentinel registered after the system has warmed up starts with class priors that may be far from its actual distribution, and would spend a standardisation half-life — some three thousand five hundred labels — being standardised against them (`cav:limitation:late-standardisation`). The bootstrap shortens that to a hundred assessments.

1. **Open** a per-Sentinel accumulator of capacity one hundred on registration.
2. **Accumulate** the new Sentinel's slot values, occupancy included, from each assessment it reports in, skipping non-finite entries and resetting at the new width if the slot width changes.
3. **Blend** the empirical statistics into the Sentinel's standardisation entries at weight $\alpha_\text{boot} = 0.8$, leaving the remaining fifth on the priors.
4. **Release** the accumulator.

The blend is soft rather than a replacement because a hundred assessments is enough to locate a distribution and not enough to characterise its tail; the retained fifth of the prior is what keeps an unlucky first hundred from fixing a variance the Sentinel will spend the next half-life escaping.
