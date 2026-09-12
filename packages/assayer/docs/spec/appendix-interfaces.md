## Appendix (Interface Map) · `app:spec:interface-map`

This appendix is the boundary verification. Every other statement in the document about what crosses a boundary is local to the chapter that makes it; this is the one place the crossings are enumerated together, and it is normative on that account — the feed-forward invariant either has a complete inventory behind it or it does not (`inv:guarantee:feed-forward`).

Three interfaces meet in this system: the one between a Sentinel and the Core, the one between a Sentinel and the graph it observes into, and the one between the Core and the host. The second of these was missing from the superseded text, which described itself as a complete interface map while enumerating two of its three interfaces; it is restored below, because an inventory that omits a boundary cannot be checked against the invariant it exists to support.

**Table (What the Core consumes)** · `tab:boundary:consumed`

The Core's relationship with each Sentinel is mediated entirely through periodic batch reports, received at one entry point (`alg:runtime:report-reception`). The batch report is the complete interface: there is no other call, query or subscription, and a record that does not appear in a report does not reach the Core at all.

| Record the Sentinel publishes | Where the Core consumes it |
| --- | --- |
| Cell analysis report, competitive and ancestor | The chain z-score features (`tab:extraction:chain-z-scores`) |
| Scoring records across the four axes | The chain z-scores and the chain CUSUMs (`tab:extraction:chain-cusums`) |
| Maturity record | The chain structure features (`tab:extraction:chain-structure`) |
| Scoring geometry record | The chain structure features, as structural reliability (`tab:extraction:chain-structure`) |
| Coordination report | The coordination features (`tab:extraction:coordination`) |
| Contour snapshot | Diagnostic only — cell-set maintenance and the health snapshot (`schema:output:health-snapshot`) |
| Analysis set summary | The chain structure features (`tab:extraction:chain-structure`) |

A record consumed here that the reception section does not read would be an interface this appendix invented, so the two inventories must agree.

**Table (What the Core does not consume)** · `tab:boundary:not-consumed`

Three published records are not read, and the reason in each case is a statement about the Core's design rather than about the record's quality.

| Record the Sentinel publishes | Why the Core does not read it |
| --- | --- |
| Per-sample scores | The Core reads batch summaries; per-request scoring would make the Core's cost linear in the observation stream rather than in the report |
| Inline health report | Sentinel-internal diagnostics, which the Core neither interprets nor relays |
| Wavelet portrait | The formation history of a spatial partition, not its current alarm state; it is a direct Sentinel-to-host diagnostic and reaches the host without passing through the Core |

Non-consumption is not a judgment of value. The portrait in particular explains how a Sentinel's partition came to be, which is exactly the question the Core has no machinery to ask and no reason to: the Core consumes alarm state, and formation history is the host's to read.

The Sentinel corpus also publishes a contour query interface — point query, range query, iteration, plateau count — at its output chapter's contour-queries section, which the Core does not use because it maintains its own spatial index over the reported cells.

**Table (The graph-to-Sentinel interface)** · `tab:boundary:graph-interface`

A Sentinel observes into a dual-tree value-stratified index, and this is the third of the system's interfaces. The Core participates in it in one direction only and in one capacity, and the pointers below are into the two upstream corpora rather than into this document, in the qualified form the front matter fixes (`conv:spec:upstream-references`).

| What crosses | Which corpus states it |
| --- | --- |
| The capabilities a Sentinel uses of the graph | The capability section of the spectral Sentinel specification, which enumerates the operations a Sentinel invokes |
| The mathematical properties the Sentinel assumes of the graph | The properties appendix of the dual-tree value-stratified index specification, assumed by the Sentinel at its stated-assumptions section |
| Observations, from the Sentinel to the graph | Unit-volume observation at a coordinate; the graph owns everything downstream of the call |
| Anything at all, from the graph to the Sentinel | Nothing: the graph has no knowledge of any Sentinel's existence |

The row that matters to a reader of *this* document is the last. A Sentinel's graph is not the Core's graph, and the Core has no interface to it in either direction (`tab:boundary:verification`). The Core does operate graphs of the same kind, one for each declared identity dimension, and those it owns, sizes and observes into itself (`def:registry:identity-dimension`) — a structural relationship, and not the observational one it has with every Sentinel.

**Table (The Sentinel symbol cross-reference)** · `tab:boundary:symbols`

A translation aid, and nothing more: the upstream corpus and this document name the same quantities differently, and a reader moving between them needs the correspondence written down once.

| Symbol | What the Sentinel corpus means by it | Where this document uses it |
| --- | --- | --- |
| $\eta$ | Noise fraction, as a maturity measure | Maturity features, entering as $1 - \eta$ (`tab:extraction:chain-structure`) |
| $k$ | Cell sample count | The batch context feature (`tab:extraction:batch-context`) |
| $\sigma_1$ | Primary singular value of the score geometry | Structural reliability (`tab:extraction:chain-structure`) |
| $\bar{s}$, $\bar{v}$ | Per-axis baseline mean and variance | Z-score normalisation (`tab:extraction:chain-z-scores`) |
| $\bar{\rho}$ | Per-axis CUSUM accumulators | The drift features (`tab:extraction:chain-cusums`) |

The table explains and binds nothing. A correspondence does not gain force by being written in an appendix, and stating it as a translation aid is what keeps a reader from treating it as an interface in its own right.

**Table (Feed-forward boundary verification)** · `tab:boundary:verification`

The complete inventory. Every boundary in the system appears here with its permitted direction and the mechanism that holds it.

| Boundary | Direction | Mechanism |
| --- | --- | --- |
| Core to Sentinel | Never | The Core reads batch reports and never writes (`inv:guarantee:feed-forward`) |
| Sentinel to its graph | Observe only | Unit-volume observation at a coordinate; the graph handles everything internal to itself |
| Derivation Function to Core | Never | It reads the risk basis and writes nothing (`inv:guarantee:derivation-purity`) |
| Derivation Function to Companion | Never | It reads a challenge posterior and writes nothing (`sig:companion:posterior`) |
| Companion and Core, in either direction | Never | No interface exists in either direction (`inv:companion:boundary`) |
| Core to a Sentinel-owned graph | Never | No interface exists (`tab:boundary:graph-interface`) |

Six boundaries, of which five are never crossed in either direction and the sixth is observe-only. The only cycle in the system passes through the host: the Core assesses, the host decides, acts and reports, and the Core learns. The Derivation Function and the Companion are consulted and never consulted back, which is the same claim the information-flow figure draws rather than states (`fig:architecture:information-flow`).


## Appendix (Cross-Layer Terminology) · `app:spec:terminology-mapping`

Three layers meet in this system and each has its own vocabulary for what is partly the same subject matter. This appendix translates between them. It is expository throughout — it introduces no obligation and constrains no implementation — but the error it exists to prevent is one the Core's design depends on not making, which is why it carries a rule for reading rather than only two tables.

**Table (Structural operations across the three vocabularies)** · `tab:terminology:structural`

The Core responds to observable changes in the batch report, and never to a Sentinel-internal event.

| The shared concept | The index corpus calls it | The Sentinel corpus calls it | What the Core observes, and does |
| --- | --- | --- | --- |
| Finer cells appear | Refinement | Catalytic split | A cell appears, and a fresh Ledger entry is created (`alg:ledger:entry-creation`) |
| A cell disappears | Eviction | Various mechanisms | After the persistence threshold, the entry is deleted (`alg:ledger:entry-deletion`) |
| An absent cell reappears | Restoration | Restoration | A cell appears, indistinguishable from any other appearance (`alg:ledger:entry-creation`) |
| Mass on removal | Exact conservation | The same | No merge is performed — the ancestor entry is already current (`just:ledger:no-state-transfer`) |
| The observation-receiving surface | Bottom contour, at the bottom-contour section of the dual-tree value-stratified index specification | Contour, at the domain-and-spatial-partitioning section of the Sentinel corpus | Not observed directly — the Core reads only the aggregate snapshot |

The fourth column is the only one this document owns, and its entries are effects rather than counterparts. The Core performs no refinement, eviction or restoration; it makes two responses to two observations.

**Table (What "importance" means in each layer)** · `tab:terminology:importance`

One word, three meanings, and the differences are load-bearing.

| Layer | What "importance" means there |
| --- | --- |
| The index and Sentinel corpora | Accumulated unit-volume observation at a coordinate, stated at the Sentinel corpus's observation-and-importance section and fixed algebraically at the index corpus's coding-theoretic foundation's algebraic-interface section |
| The Core, reading a Sentinel | Not read directly — it reaches the Core only through the competitive selection the batch report already performed |
| The Core, in its identity layer | Read directly — competitive sets are defined by importance-ranked depth in the dimension's own graph (`def:keyspace:competitive-set`) |

The second and third rows are the same word doing opposite work, and the distinction is exactly the one the Core's two relationships with a graph rest on: what a Sentinel's graph ranks is the Sentinel's business, and what the Core's own graphs rank is the Core's.

**Convention (The abstraction boundary)** · `conv:terminology:abstraction-boundary`

Causation is traced; operations are not corresponded. The structural mapping above is offered to a reader who understands all three layers, and its Core column describes observable effects, not corresponding operations. Nothing in this document licenses the reading that the Core performs a split, an eviction or a restoration, or that it detects one.

What the Core does is narrower and is stated as such wherever it acts: it observes cell-set changes and responds with entry creation or deletion, and the Sentinel-internal cause of a change is not merely unavailable to it but irrelevant to what it does (`alg:runtime:cell-set-maintenance`). The convention binds as a rule for reading because the alternative reading is the tempting one and it is wrong in a way that propagates: a reader who believes the Core detects evictions will expect it to distinguish an eviction from a restoration, will look for the mechanism that does so, and will find a limitation where there is a boundary (`cav:limitation:abstraction`). No Sentinel-internal event type reaches the maintenance path.
