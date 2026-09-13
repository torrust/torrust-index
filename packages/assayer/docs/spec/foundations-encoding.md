### Chapter (The Encoding Contract) · `chap:spec:encoding-contract`

The chapter develops the encoding constraint (`assum:constraint:encoding-dependence`) into a contract. It states what the host owes before a Sentinel exists, gives the guidance a host needs to discharge it, and fixes the one width relationship the Core and the host must agree on. The contract binds; the guidance between it and the width does not, and is not written as though it did.

**Requirement (The host's encoding contract)** · `req:encoding:host-contract`

Every Sentinel analyses a stream of integer coordinates in its declared domain under the assumption that shared prefixes imply shared context. The Core combines the statistical output of several Sentinels, each watching a different facet of the data, possibly at different widths. The quality of the Core's risk estimates is bounded above by the quality of the Sentinels' measurements, which is bounded above by the quality of the host's encoding decisions.

The encoding decision is the host's, and it is the single most consequential decision in a deployment. Before a Sentinel is created, the host answers the three questions of this chapter for the data source that Sentinel will watch, and declares its answers at registration: what the coordinate space is, what a shared prefix asserts about two coordinates in it, and how deep the meaningful structure runs. The declarations exist for three reasons — to force engagement with the encoding question before a Sentinel exists rather than after it has been running for a quarter, to make the deployment's encoding assumptions auditable by someone who did not make them, and to give a diagnosis of an uninformative Sentinel somewhere to start.

The Core has no mechanism to detect a bad encoding and no ability to compensate for one. Two things partially compensate and neither substitutes: the health report's association reading (`def:monitoring:slot-association`) raises an early warning where a block's coordinates have stopped telling the outcome anything, and the model eventually stops leaning on features that carry no information (`rem:encoding:downstream-fallback`). A partial compensation that is mistaken for the contract is worse than no compensation, because it converts a decision the host must make into one the host believes has been made for it.

**Definition (Hierarchical structure)** · `def:encoding:question-hierarchy`

The first question is whether the data source has hierarchical positional structure at all. Values whose leading bits encode progressively finer categorical or spatial membership — where a shared prefix implies shared context — are suitable. Values with pseudo-random, encrypted, hashed, or uniformly distributed bit patterns are not.

The failure mode is silent, which is why the question comes first. Processing unsuitable values produces a Sentinel that runs without error, whose spatial partitioning is arbitrary, whose multi-scale analysis is noise, and whose four-axis scores are statistically valid and semantically vacuous. Nothing in the system reports an error, because nothing has failed: the instrument measured exactly what it was given.

**Definition (What a shared prefix asserts)** · `def:encoding:question-meaning`

The second question is what the prefix hierarchy means for this source. A Sentinel's partitioning creates cells at dyadic boundaries, and a cell at a given depth groups all values sharing a prefix of that many bits. The host must ensure that this grouping corresponds to a grouping that is meaningful for the data source.

For network addresses the correspondence is automatic: a routing prefix corresponds to a cell at the matching depth, grouping addresses in one subnet. For geographic codes it depends on the encoding scheme. For categorical data the host must choose an encoding under which the categories' own tree structure maps onto the binary prefix tree, rather than assuming that any integer encoding will do. For multi-dimensional data the host must choose a space-filling curve (`tab:encoding:curve-guidance`), and the choice determines which notion of proximity the prefix hierarchy captures.

**Definition (Depth of meaningful structure)** · `def:encoding:question-depth`

The third question is whether the hierarchical structure runs deep enough to justify multi-scale analysis. A Sentinel's value proposition is that it discovers structure at every resolved depth and provides ancestor-chain coverage through models at every ancestor level. Where a source has only two or three meaningful hierarchical levels, the Sentinel creates a few spatial levels before the prefix structure stops meaning anything: the ancestor chain is short, the coordination tier has few contexts, and the system works while providing modest value over a simple per-group anomaly detector.

Sources with deep, meaningful hierarchy exploit the architecture fully — network addresses, whose prefix structure runs to the full address width; fine-grained geographic codes, with twenty levels or more; richly branching categorical taxonomies. The question is not whether the source has many distinct values but whether it has many meaningful levels.

**Table (Suitable sources)** · `tab:encoding:suitable-sources`

Sources whose prefixes carry meaning, and what the meaning is. The table is guidance for answering (`def:encoding:question-meaning`), not an enumeration of what the Core accepts.

| Source | Natural hierarchy | What a shared prefix means |
| --- | --- | --- |
| Network addresses | Network prefix tree | The same network, allocation, or autonomous system |
| Hierarchical geographic codes | Spatial containment | The same region, at progressively finer scale |
| Categorical taxonomies with positional encoding | Category tree | The same branch of the taxonomy |
| Temporally bucketed sequences with bit-interleaved timestamps | Temporal proximity | The same time window |
| Multi-dimensional data mapped through a space-filling curve | Spatial proximity, curve-dependent | The same spatial neighbourhood, with the curve's locality properties |

**Table (Unsuitable sources)** · `tab:encoding:unsuitable-sources`

Sources whose prefixes carry no meaning, and why. Each fails (`def:encoding:question-hierarchy`), and each fails silently.

| Source | Why unsuitable |
| --- | --- |
| Cryptographic hashes | Deliberately destroy positional structure |
| Random session tokens | No prefix semantics |
| Uniformly distributed synthetic identifiers | No hierarchy |
| Raw floating-point values without quantisation | The bit-level structure of the representation does not correspond to value proximity in the way the analysis requires |
| Categorical data with arbitrary integer labels | Two adjacent labels sharing a prefix is an artefact of the encoding, not a semantic relationship |

**Table (Space-filling curve guidance)** · `tab:encoding:curve-guidance`

For multi-dimensional data mapped to a single coordinate stream, the curve determines which notion of proximity the prefix hierarchy captures.

| Curve | Locality property | Best for |
| --- | --- | --- |
| Bit-interleaved ordering | Axis-aligned rectangles are contiguous | Data with axis-aligned structure |
| Hilbert ordering | Compact regions are contiguous | Data with isotropic spatial structure |
| Double-referenced interleave | Vocabulary-change-aware rank encoding | Joint structure discovery between two one-dimensional vocabularies |

A wrong curve does not produce errors. It produces a Sentinel whose partitioning cuts across the data's natural structure rather than along it, reducing multi-scale analysis to noise at exactly the scales where the misalignment falls.

**Remark (Downstream learning is not the remedy)** · `rem:encoding:downstream-fallback`

The Core accepts whatever the host provides. It extracts features, learns weights, and produces risk estimates from any Sentinel's output. If a Sentinel's measurements are noise, the Core will eventually learn from labelled outcomes that its features are uninformative, and downweight them.

This self-correction is real and slow, and it is worth being precise about what it does, because the obvious reading of it is wrong. What converges to nothing is the block's contribution to the score — measured, the removal cost of a hash-fed Sentinel fell from $0.125$ of the score's loss towards $0.0005$ as the model converged (`def:monitoring:slot-contribution`). What does not converge to nothing is the weight the model carries there: standardised-space ridge weights on uninformative coordinates settle at noise scale, and the mean absolute slot weight of that same Sentinel stayed within a tenth of a perfect control's throughout (`def:monitoring:encoding-effectiveness`). The learning stops relying on the features; it does not empty their weights.

The correction costs hundreds of labels and wastes model capacity while it runs, and during that window the deployment is paying full price for a Sentinel contributing nothing. The association reading (`def:monitoring:slot-association`) is intended to surface the failure earlier than either of those, because it is model-free: it reads the stream rather than what the model made of it, and so has no convergence to wait for.

Neither is the remedy. The correct remedy is not to rely on downstream learning to compensate for upstream encoding failures; the correct remedy is to get the encoding right. Downstream learning is what limits the damage of an encoding error, not what licenses one.

**Definition (Domain width)** · `def:encoding:domain-width`

Each Sentinel analyses coordinates in its own native domain $[0, 2^{N_s})$, whose bit-width $N_s$ the host declares per data source: 32 for a four-octet network address, 128 for a sixteen-octet one, 40 or 48 for a bit-interleaved geographic code. Different Sentinels in one Core may declare different widths; the Core places no uniformity constraint across the registry.

The Core's internal coordinate width is fixed at 128 bits. Coordinates are stored and compared at that width in every per-Sentinel spatial structure — the report index, the outcome memory, the pending buffer's coordinate store — and on every internal routing path, which fixes a single arithmetic width across the Core.

The declared width is an upper bound the Core checks. A registration declaring a width greater than the internal width is rejected at registration (`alg:registry:sentinel-registration`), and that is the only width constraint the Core imposes. Every other consequence of the declared width is the host's: the Core does not verify that reported coordinates lie within the declared domain, and does not verify that the declaration describes the source at all.

**Algorithm (Coordinate lift)** · `alg:encoding:coordinate-lift`

For a Sentinel whose declared width $N_s$ is narrower than the internal width, coordinates are left-justified into the internal representation at the integration boundary:

$$x_{\text{internal}} = (x_{\text{native}} \text{ as u128}) \ll (128 - N_s)$$

The lift is the responsibility of the integration layer that delivers batch reports to the Core. The host, or a thin adapter, widens the batch report by left-justifying every reported cell's start and end by the shift above, and lifts the entity coordinates used for per-Sentinel routing at assessment and label time identically. The Core's report-reception surface accepts a post-lift report, matching its internal representation; the Core does not perform the lift, and has no way to tell a lifted coordinate from a natively wide one.

Cell-depth semantics are preserved exactly. A native cell at a given depth covers a dyadic interval of a width set by the declared domain; after lifting it covers a dyadic interval of the corresponding width in the internal representation, at the same depth. The receiving-cell relationship, the ancestor chain, and the all-layers routing of outcome memory are properties of the dyadic hierarchy, and the shift preserves that hierarchy, so none of them is affected.

**Decision (Fixed internal width)** · `dec:encoding:fixed-internal-width`

The internal coordinate width is fixed rather than generic over a per-Sentinel coordinate type. Making it generic would require either enum erasure or trait-object dispatch across every per-Sentinel container — the report index, the outcome memory, the pending buffer — adding dispatch cost and structural complexity for a use case that never arises: no operation in this architecture compares coordinates across Sentinels, and the feed-forward invariant (`inv:guarantee:feed-forward`) is part of why none can.

The cost of the fixed width is bounded. A Sentinel declaring a narrow domain leaves the high bits of its internal coordinates unused, but its storage footprint is dominated by cell count rather than by coordinate width, so the waste does not scale with anything that grows. The fixed width is the cheaper design, and it fixes one arithmetic width across the Core rather than distributing a width question over every internal path.

**Remark (The declared width is unchecked)** · `rem:encoding:unchecked-width`

The declared domain width is an assertion, not a measurement. Beyond rejecting a declaration wider than the internal width, the Core does not check it: it does not verify that reported coordinates fall inside the declared domain, does not verify that the lift was performed at the declared shift, and cannot detect a declaration that is simply wrong.

A wrong declaration is therefore silent in the same way a wrong encoding is. If the lift shifts by the wrong amount, every coordinate lands at the wrong place in the internal representation, every cell interval is dyadic at the wrong depth, and the outcome memory accumulates against a hierarchy that does not correspond to the source's. Nothing reports an error, because at the internal width the values remain well-formed.

The correspondence between the declared width, the lift, and the source is the host's to maintain. This is the same division of responsibility the contract states (`req:encoding:host-contract`), applied to the one number in it the Core consumes arithmetically.
