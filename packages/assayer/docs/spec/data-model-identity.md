### Chapter (The Key Space Identity Layer) · `chap:spec:key-space-identity`

The chapter specifies the layer that finds significant ranges in a host-declared key space and reports membership in them as features. It states what the layer is for and what it refuses to do, what a dimension declares, how an observation is processed, what the competitive set is and is not, which features each dimension contributes, what state a competitive cell holds and how it decays, and what entering and leaving the competitive set costs. Dimension lifecycle is not restated here. Registration follows (`alg:registry:dimension-registration`) and removal follows (`alg:registry:dimension-deregistration`), both under the algebra that governs Sentinels.

**Motivation (What the identity layer is for)** · `mot:keyspace:purpose`

Consider a deployment watching network traffic, tracking entity identity by source address. It needs to learn that requests from a particular subnet have historically been associated with adverse outcomes — and it needs that grouping to emerge from observation volume rather than from a static configuration, since nobody knows in advance which subnet will matter. The identity layer does this: it maintains a graph over the address space, discovers significant ranges by competitive ranking, and hands the models an indicator saying that this request comes from a range the system considers significant. The model learns each range's risk weight from labelled outcomes.

Three things the layer does not do. It does not track entities. It does not maintain per-entity state as a primary function. And it interprets nothing: it partitions integer ranges, ranks cells by observation volume, and reports which competitive cells contain a coordinate. The host reads a cell as an entity, a group, an organisation, a device family, or a region, and the mechanism reads it as a range.

**Schema (Identity dimension registration record)** · `schema:keyspace:dimension-record`

A dimension is registered with an identifier, a name, a description, a declared domain width, its coordinate semantics, an encoding function from entity keys to coordinates, a budget, and a competitive depth cutoff.

| Field | Carries |
| --- | --- |
| Identifier | The registry key |
| Name | Unique across currently registered dimensions |
| Description | What this key space is, in the host's own words |
| Declared domain width | The width of the dimension's integer domain (`def:encoding:domain-width`) |
| Coordinate semantics | The same declaration a Sentinel makes (`schema:registry:sentinel-record`) |
| Encoding function | The map from an entity key to a coordinate in the domain |
| Budget | The graph's sizing and decay parameters |
| Competitive depth cutoff | The depth beyond which an entry is not competitive |

The budget carries five parameters: the graph's maximum size, the split threshold, the depth at which a cell may be created, the depth at which one is evicted, and the spatial decay rate applied to importance. They are the parameters the underlying graph is sized by, and they are declared at registration rather than derived, because a host that cannot set them cannot control what the layer costs.

The coordinate semantics are the Sentinel's declaration in every respect, including the assertion that shared prefixes imply shared context, and the Core does not verify the assertion here either.

**Requirement (The encoding contract for key spaces)** · `req:keyspace:encoding-contract`

The encoding contract (`req:encoding:host-contract`) applies to an identity dimension exactly as it applies to a Sentinel, and its three questions are asked of the key space rather than of a data source. Does this key space have hierarchical positional structure (`def:encoding:question-hierarchy`)? Network addresses do, through the prefix tree; electronic mail addresses do under a reversed-domain encoding; composite tenant keys do with the tenant in the high bits; session tokens do not. What does a shared prefix assert (`def:encoding:question-meaning`)? For addresses, a shared network; for reversed domains, a shared organisation; for composite keys, the same tenant. And does the hierarchy run deep enough (`def:encoding:question-depth`)? Address spaces do, at thirty-two bits or a hundred and twenty-eight; a domain encoding is moderate; a hashed token has no hierarchy at any depth.

A dimension should not be registered for a key space without hierarchical structure. The mechanism will run without error — that is the difficulty — but the competitive cells will correspond to arbitrary groupings of strangers, the indicators will carry no meaning, and the models will learn weights at or near zero on all of them. The cost is not a wrong answer; it is model dimensions spent, convergence slowed, and a layer that looks like it is working.

**Algorithm (Observation protocol)** · `alg:keyspace:observation-protocol`

At each assessment, for each registered dimension:

1. **Compute the coordinate** by applying the dimension's encoding function to the entity key.
2. **Record the observation** at unit weight for deferred processing by the dimension's graph, which maintains importance and triggers structural operations asynchronously. The assessment does not wait for it: the competitive set is read from the most recently published state, and the deferred queue is drained off the assessment path.
3. **Determine the active competitive cells** — those cells of the competitive set whose range contains the coordinate.
4. **Set the indicators**, one for each active cell and zero for the rest.

The weight recorded at step two is always unit. No anomaly score and no risk estimate reaches it, which is how the feed-forward invariant (`inv:guarantee:feed-forward`) holds through the one graph the Core operates itself: the Core observes into the graph, and nothing the Core computes influences what the graph does with the observation.

Step three is a walk rather than a search. Because dyadic intervals either nest or are disjoint, the active set is a chain under containment, so the mechanism walks from the deepest containing cell toward the root and collects the competitive cells on the path, at a cost set by that cell's depth — bounded by the domain width, and in practice three to eight levels. The number of active indicators per request per dimension is data-dependent and unconstrained, typically none to three.

**Definition (The competitive set)** · `def:keyspace:competitive-set`

The competitive set of a dimension is the set of the dimension's graph entries standing within its configured depth cutoff. There is no further filtering: no state filter, no minimum width, no coverage requirement. At a cutoff of three, with a branching factor of two to three, the set holds roughly ten to twenty-five entries, the exact count set by the observation distribution and by competitive dynamics rather than by configuration. The depth bound is what floors an entry's importance: an entry at depth three carries at least about fifteen per cent of the dimension's total importance.

The set has no spatial invariant, and the absences are deliberate. It is not a partition: cells may nest, and gaps between them are ordinary. It is not a cover: most coordinates lie in no competitive cell at all. It is not required to tile any fraction of the domain. Nesting is permitted, and because dyadic intervals nest or are disjoint, an active set is always a chain and never partially overlapping. And the set's composition changes whenever the underlying tree restructures. Everything downstream is written against a set with these properties, so a reader looking for a guarantee of coverage will not find one here because there is none to state.

**Definition (Competitive indicators)** · `def:keyspace:competitive-indicators`

Each competitive cell contributes one indicator feature, so a dimension's indicator count equals its competitive set's current size and changes when the set restructures.

$$\phi_{\text{dim}_d} = \bigl(\mathbb{1}[x_d \in c_1],\; \ldots,\; \mathbb{1}[x_d \in c_{|\mathcal{E}_d|}]\bigr)$$

The model learns one weight per cell, and a request's identity contribution is the sum of the weights of the cells containing its coordinate:

$$\Delta\hat{\rho}_{\text{dim}_d} = \sum_{c \in \text{active}_d(x_d)} \mu_c$$

The sum has no terms where no competitive cell covers the coordinate, and one term per covering cell where some do. The terms compose additively, which is what decomposes risk across whatever scales the competitive mechanism happened to select (`prin:principle:multi-level-decomposition`): the entity, its enclosing group, and the broader region each contribute their own learned weight where each is competitive.

**Table (Per-dimension features)** · `tab:keyspace:dimension-features`

Every registered dimension contributes a contiguous block of $8 + 3m$ features for $m$ registered outcome axes, combining structure, measurement, and outcome history over the dimension's active competitive cells (`def:keyspace:competitive-set`).

| Feature | Width | Source | Updated |
| --- | --- | --- | --- |
| Coverage depth | 1 | The receiving cell's depth over the domain width | Assessment |
| Active indicator count | 1 | Active cells over competitive set size | Assessment |
| Total dimension importance, logarithmic | 1 | The dimension's total importance | Assessment |
| Maximum alarm across active cells | 1 | Per-range measurement state | Assessment |
| Mean alarm across active cells | 1 | Per-range measurement state | Assessment |
| Maximum suspicion across active cells | 1 | Per-range measurement state | Assessment |
| Volatility of the deepest active cell | 1 | The most specific active cell | Assessment |
| Has a competitive cell | 1 | Whether any cell is active | Assessment |
| Mean compressed axis average, per axis | $m$ | Per-cell outcome state | Label |
| Mean raw axis average, per axis | $m$ | Per-cell outcome state | Label |
| Axis stability, per axis | $m$ | Spread of the compressed averages across active cells | Label |

The block is fixed-width per dimension: it does not change when the competitive set restructures, which is what lets a dimension's indicators change count without the rest of its block moving. The first eight are written at assessment time and the remaining $3m$ at label time. The per-axis features are independent of the spatial-feature policy — that policy governs the Sentinel outcome-memory features (`tab:extraction:ledger-features`), not identity cell state.

**Table (Cross-dimension aggregates)** · `tab:keyspace:cross-dimension-features`

Where at least one dimension is registered, a fixed block of eight features summarises across dimensions by maximum. Six of them are consumed by the fixed-dimension anchor model at named positions in its projection (`def:dimension:anchor-projection`), and five carry a name the default interaction templates refer to them by.

| Feature | Aggregation | Anchor position | Template name |
| --- | --- | --- | --- |
| Maximum suspicion | Across dimensions, over active cells | 7 | Maximum suspicion |
| Maximum alarm | Across dimensions, over active cells | — | Maximum alarm |
| Maximum volatility | Across dimensions, over deepest active cells | 12 | Volatility |
| Maximum adverse rate | Across dimensions, over active cells | 8 | — |
| Maximum absolute compressed valence | Across dimensions, over active cells | 10 | — |
| Maximum absolute raw valence | Across dimensions, over active cells | 11 | — |
| Has any competitive cell | Across dimensions | 9 | Has competitive |
| Maximum coverage depth | Across dimensions | — | Coverage depth |

The block is zero-filled where no cell is active in any dimension. It is created when the first dimension is registered and destroyed when the last is deregistered, and it changes width neither on restructuring nor on any lifecycle event within the layer — which is what allows the anchor's projection to name positions in it at all.

**Table (The signal cache)** · `tab:keyspace:signal-cache`

Entity-persistent signals — authentication status, account age, risk tier — are operationally important and must not be gated on competitive standing. They are held in a per-entity cache independent of the layer's graphs, a hash map with least-recently-used eviction.

| Property | Value |
| --- | --- |
| Capacity | Configurable, default 100,000 entries |
| Entry size | About 80 bytes, ten signals of eight bytes |
| Eviction | Least recently used |
| Total memory | About 8 MB at the default capacity |

The cache is deliberately outside the identity layer proper. An entity's authentication status is a fact about the entity whether or not its range has earned competitive standing, and coupling the two would make an operationally important signal disappear exactly when an entity became uninteresting to the competitive mechanism. Competitive entry and exit are unaffected by cache state, and cache eviction loses evidence without touching the layer.

**Table (Per-range measurement state)** · `tab:keyspace:measurement-state`

Each competitive cell in every dimension carries measurement state: how the requests routed through this range currently look to the Sentinel measurement layer.

| Component | Update rule |
| --- | --- |
| Per-Sentinel maximum composite alarm average | Exponential average at the measurement smoothing factor, over the request's composite alarm from that Sentinel |
| Suspicion | Derived: the mean of the clipped normalised alarms |
| Volatility | Exponential average over the squared change in the alarm average |
| Step count | Incremented once per assessment |

The state is updated at each assessment for the active competitive cells of every registered dimension, taking the current batch reports' composite alarm as its input. It contributes the measurement features of (`tab:keyspace:dimension-features`) rather than a block of its own, and is zero-filled where a dimension has no active cell.

Measurement state exists only in competitive cells. It initialises to zero on entry, is discarded on exit, propagates through neither merge nor split, and is held by no non-competitive cell. It is a feature of competitive attention rather than a property of the key space, and at the default smoothing factor its averages converge within about twenty assessments, so the warm-up after entry is short.

**Table (Per-cell outcome state)** · `tab:keyspace:outcome-state`

Where measurement state tracks what the measurement layer currently sees, outcome state tracks what happened to labelled requests from the range. It is written at label time, and each competitive cell in every dimension holds six components.

| Component | Update rule |
| --- | --- |
| Adverse rate average | Exponential average over the positive-valence indicator (`conv:valence:sign`) |
| Compressed valence average | Exponential average over the compressed valence, saturating at unit magnitude |
| Raw valence average | Exponential average over the reported valence, unbounded |
| Per-axis compressed average | One per registered axis, over the compressed axis value |
| Per-axis raw average | One per registered axis, over the reported axis value |
| Last updated | Overwritten, for lazy time-indexed decay |

A label updates every competitive cell containing its coordinate in each dimension. Because an active set can hold several nested cells, one label can write several cells in one dimension — the outcome applies to the specific entity, to the enclosing group, and to the broader region, wherever all three are competitive.

The state serves two purposes and no third. It supplies the three per-axis features each dimension contributes for each registered axis (`tab:keyspace:dimension-features`), computed across the active cells at assessment time and falling back to zero where no cell is active, or where at most one is for the stability feature. And it supplies warm-start context when the set restructures: a cell entering a region where a parent's outcome state survives can inherit a blended state rather than starting cold.

Under the recommended default, outcome state is not extracted as per-cell features. The per-cell weight learned from labels already captures the cell-specific outcome signal, and the running average is redundant with it: the weight is the more powerful of the two because it conditions on every other feature, and the average is the faster to converge because it conditions on nothing. The three per-axis features are aggregates across active cells, not additional per-cell slots, and that is the whole of what outcome state contributes to the vector.

**Table (Decay in the identity layer)** · `tab:keyspace:decay-rates`

Three mechanisms decay identity state, at three rates and for three reasons.

| What decays | Mechanism | Rate | Purpose |
| --- | --- | --- | --- |
| Cell importance in the graph | Spatial decay, per hour | The dimension's declared spatial decay rate | Governs how the contour evolves and who stays competitive |
| Outcome averages in competitive cells | Time-indexed lazy decay, per hour | Default $0.998$, a half-life near fourteen days | Prevents stale outcome state |
| Measurement averages in competitive cells | Per-assessment exponential average | Default $0.95$ | Tracks the current alarm profile |

Importance decay governs how quickly an inactive entity loses competitive standing: at the default rate an entity unseen for a fortnight has its importance halved, and if that drops it past the cutoff it exits the set, its features are marginalised out, and its cell is absorbed into its parent where the graph evicts it.

Outcome decay is what breaks the contamination loop (`alg:valence:contamination-loop`) in this layer: stale outcome history dissolves on a fourteen-day timescale whatever the label flow, so a starved cell still forgets. That is faster than the spatial outcome memory's own rate (`def:ledger:time-decay`), and deliberately so — identity cell state is the more volatile of the two, describing a range's current occupants rather than a coordinate region's long history.

**Algorithm (Competitive entry)** · `alg:keyspace:cell-entry`

The batching unit for competitive lifecycle is one restructuring event, not one cell. Where an event promotes or demotes several cells, every resulting extension and marginalisation is applied in deterministic event order and followed by a single map rebuild, so the rebuild's linear cost is paid once per event.

On promotion of a cell into the competitive layers:

1. **Extend every model but the anchor** by one indicator dimension plus the competitive interaction dimensions declared for this dimension — one by default — at the prior, with zero off-diagonals (`thm:gaussian:extension`).
2. **Extend standardisation** at the class priors: a binary indicator at mean one half and variance one quarter, an interaction product at mean zero and unit variance.
3. **Initialise outcome state**, at zero or at the parent's blended state where the parent was previously competitive.
4. **Initialise measurement state** at zero.
5. **Record the new membership** for the event's final map rebuild.

Entry costs one pass per model, and the anchor is untouched.

**Algorithm (Competitive exit)** · `alg:keyspace:cell-exit`

On demotion of a cell past the depth cutoff:

1. **Marginalise every model but the anchor** by the same one plus the declared competitive interaction dimensions, removed together in a single step. At the default this is a rank-two correction, quadratic in the dimension per model (`alg:gaussian:regularised-schur`). The covariance path reads its block directly, and the residual discrepancy is cleared at the next scheduled recomputation (`rem:gaussian:dual-tracking`).
2. **Remove the standardisation entries.**
3. **Discard the measurement state.**
4. **Retain the outcome state** in the cell, for the warm start of a future entry from this region, or discard it where the graph has evicted the cell.
5. **Record the removed membership** for the event's final map rebuild.

Exit preserves more than it appears to. The Schur complement transfers what the models learned about the relationship between this cell's indicator and every surviving feature into the survivors' precision (`def:gaussian:schur-complement`): if the models learned that this cell's indicator together with an elevated address alarm predicted adverse outcomes, that correlation's contribution to the address feature's precision survives the removal. What does not survive is the cell's own weight, which starts from zero if the cell later re-enters. At three models and the reference dimension, an exit is roughly 1.2M operations, and the set turns over a few times a day under typical decay parameters, so the lifecycle's total cost is negligible beside the label path.

**Requirement (What the host does)** · `req:keyspace:host-duties`

At construction the host does four things. It decides which facets of entity identity have hierarchical structure worth exploiting. It supplies, per dimension, an encoding function preserving that structure under the same contract Sentinel encoding is held to (`req:encoding:host-contract`). It configures each dimension's budget parameters, which are the parameters the underlying graph's spatial layer defines and which the Sentinel algorithm document declares at its section 3.11 — a host told to configure a parameter and not told where it is defined has been told nothing. And it declares its entity-persistent signals (`schema:signal:shapes`).

At assessment time the host provides the entity key and the current values of any entity-persistent signals. At label time it provides the label, and the outcome update reaches every competitive cell containing the labelled coordinate automatically.

Three ongoing duties. Monitor each dimension's association with the outcome (`def:monitoring:slot-association`) over that dimension's own block: where it sits at the null floor across a sustained stretch of traffic, and the dimension's contribution (`def:monitoring:slot-contribution`) confirms it by sitting below its own threshold, the encoding is returning nothing and the dimension should be revised or removed.

The trigger is two readings and not one because each answers what the other cannot. The association reading is model-free, so it screens: it is what raises, and it cannot be climbed by a model that has run out of residual. The contribution reading is the removal cost itself, so it confirms: it is what says the removal is actually free. Either alone misleads in a direction the other covers. A block can carry the outcome and still cost nothing to remove, because a second block carries the same information; a block can be idle across a stretch of traffic without having stopped mattering to the score. Retiring on the first alone discards dimensions that were earning their width, and retiring on the second alone keeps dimensions that were not.

The two terms are not equally evidenced at this scope, and a host reading them should know which is which. The screening term is measured: driven against a null dimension differing only in whether its coordinate was drawn from the half of the pool the outcome named, a keyed dimension's association stood at least five and a half times the null one's at every point of a schedule that rebuilt the layout halfway through, the keyed figure clear of the alert band and the null figure inside it (`def:monitoring:slot-association`). The confirming term is not: driven against the same pair on the same schedule, its two figures crossed each other in both directions across repeated runs, because unlike the screening term it scales with the operational weights and so with a convergence state a schedule does not fix (`def:monitoring:slot-contribution`). So a per-dimension alert that the screening term raises is an alert on measured ground, and the confirmation the duty asks for beside it is a reading whose discrimination at this scope no measurement yet supports. That is a reason to weigh the confirmation rather than to take a silent one as licence, and it is not a reason to retire on the screen alone: the failure the second reading covers — a block carrying the outcome that a duplicate makes free to remove — is real whether or not the reading has been shown to catch it.

The quantity the duty names is the per-dimension restriction of both readings and not the per-Sentinel one they are the sibling of (`def:monitoring:encoding-effectiveness`) — all of these are taken over disjoint blocks of the vector, so a Sentinel-scoped figure could not say which dimension to revise. The dimension's weight mass (`def:monitoring:dimension-informativeness`) is reported beside them and is deliberately not the trigger. It says how much weight the model carries on the dimension's block, which a block predicting nothing earns about as readily as one that predicts: driven in two worlds differing only in whether the entity predicted the outcome, a null dimension read between $67\%$ and $143\%$ of the keyed one's figure and crossed it in both directions, and the later measurement that established the association reading's separation found the same crossing on its own pair over a wider range. A duty triggered on it would send hosts to revise dimensions at random with respect to whether the dimension was returning anything. The dimension's cell weights are reported beside it on the same footing and for the same reason a second reading is wanted at all: the block readings are over the dimension's block and not over its competitive indicators, and a dimension can carry weight in the one and none in the other. Monitor competitive set stability: frequent churn suggests the split threshold or the decay rate wants adjusting. Monitor coverage: a set covering less than a tenth of assessment volume suggests a cutoff too restrictive or a threshold too high.

Everything else is the mechanism's. Which entities earn competitive standing, what the ranges are, how the weights decompose risk across scales, when cells enter and leave, and what marginalisation preserves when they do — none of it is the host's to decide. The host supplies the key space, the encoding, and the budget.
