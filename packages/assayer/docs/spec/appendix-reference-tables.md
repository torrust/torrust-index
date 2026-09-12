## Appendix (Extraction Reference) · `app:spec:extraction-reference`

The per-Sentinel extraction is specified feature group by feature group in the extraction chapter (`setup:extraction:from-batch-report`); this appendix states the result as one index, position by position, so that a reader holding an offset can find out what sits there without reconstructing the groups. It is a reference table in the strict sense: it adds nothing to the specification, and its whole value is that it can be looked up.

**Table (The extraction index)** · `tab:extraction:reference-index`

Every index below is extraction-relative — a position within $\mathbf{g}_s \in \mathbb{R}^{60 + 2 m_s}$, not within the Sentinel's slot. To convert a row to a slot-relative offset, add one for the occupancy indicator (`conv:extraction:offsets`). The four per-axis features are named for the axes in their fixed order: novelty, drift, spread and coordination.

| Index | Group | View | Features |
| --- | --- | --- | --- |
| $0$–$3$ | Chain z-scores | Cell | The four axes |
| $4$–$7$ | Chain z-scores | Root | The four axes |
| $8$–$11$ | Chain z-scores | Maximum | The four axes |
| $12$–$15$ | Chain z-scores | Mean | The four axes |
| $16$–$19$ | Chain z-scores | Gradient | The four axes |
| $20$–$23$ | Chain z-scores | Spread | The four axes |
| $24$–$27$ | Chain CUSUMs | Cell | The four axes |
| $28$–$31$ | Chain CUSUMs | Root | The four axes |
| $32$–$35$ | Chain CUSUMs | Maximum | The four axes |
| $36$–$43$ | Chain structure | — | Rank and energy at two views each, maturity at two views, chain length, report staleness |
| $44$–$47$ | Coordination | Per-axis maximum z-score | The four axes |
| $48$–$51$ | Coordination | Per-axis maximum CUSUM | The four axes |
| $52$–$55$ | Coordination | — | Concordance, context fraction, depth, root z-score |
| $56$ | Ledger | — | Cell exponentially-weighted adverse rate |
| $57$ | Ledger | — | Compressed valence, exponentially weighted |
| $58$ | Ledger | — | Raw valence, exponentially weighted |
| $59 \ldots 58 + 2 m_s$ | Ledger, per axis | — | Compressed and raw axis means, one pair for each of the $m_s$ eligible axes |
| $59 + 2 m_s$ | Batch context | — | The logarithm of one plus the sample count |

The groups partition the range without gap or overlap, and the partition is checkable from the table alone: six z-score views at four axes each fill $0$–$23$, three CUSUM views fill $24$–$35$, chain structure fills $36$–$43$, coordination fills $44$–$55$, the Ledger's fixed features fill $56$–$58$, the per-axis pairs follow, and batch context is last. Summing the groups gives $60 + 2 m_s$, which is the width the extraction declares (`tab:extraction:chain-z-scores`).

The width formula, the dimension-map allocation and the slot conversion must agree (`def:dimension:layers`). Because every row mirrors a constant, this appendix is the document's best candidate for a mechanical check — a table that is generated from the extraction's own indices cannot drift from them, and a table transcribed by hand eventually will.

## Appendix (Dimensional Summary) · `app:spec:dimensional-summary`

The feature vector's dimension is an accounting identity over eight blocks, and this appendix states it, evaluates it at the reference configuration, and tabulates how it scales. The identity matters beyond bookkeeping: the Gaussian algebra's costs are stated in the dimension (`tab:gaussian:operation-costs`), so a reader sizing a deployment reads this appendix first and the resource chapter second.

**Equation (The dimension identity)** · `eq:dimension:total`

$$p = 1 + p_\text{agg} + (8 + 3m)D + 8\,\mathbb{1}[D > 0] + p_\text{sig} + n(q+1) + p_\text{int} + \sum_d \lvert\mathcal{E}_d\rvert$$

The terms are, in order: the bias; the Sentinel aggregate block; the identity per-dimension features over $D$ declared identity dimensions with $m$ outcome axes; the cross-dimension block, present whenever any identity dimension is declared; the host signal block; the $n$ Sentinel slots of $q+1$ positions each (`def:extraction:slot`); the interaction block; and the competitive indicators of each identity dimension (`def:keyspace:competitive-indicators`). Every term is a block of the dimension map and appears in the map in this order (`def:dimension:layers`).

**Table (Dimensions at reference, and their scaling)** · `tab:dimension:summary`

At the reference configuration (`tab:resource:reference-configuration`) the eight blocks contribute as follows.

| Block | Formula | At reference |
| --- | --- | --- |
| Bias | $1$ | $1$ |
| Sentinel aggregate | $p_\text{agg}$ | $15$ |
| Identity per-dimension | $(8 + 3m) D$ | $11 \times 2 = 22$ |
| Identity cross-dimension | $8\,\mathbb{1}[D > 0]$ | $8$ |
| Host signals | $p_\text{sig}$ | $0$ |
| Sentinel slots | $n(61 + 2 m_s)$ | $8 \times 63 = 504$ |
| Interactions | $5n + 8 + \sum_d \lvert\mathcal{E}_d\rvert$ | $40 + 8 + 20 = 68$ |
| Competitive indicators | $\sum_d \lvert\mathcal{E}_d\rvert$ | $10 + 10 = 20$ |
| **Total** | | **638** |

Holding two identity dimensions and no host signals, the dimension scales in the Sentinel count and the per-Sentinel eligible-axis count as follows.

| Sentinels | $m_s = 0$ | $m_s = 1$ | $m_s = 3$ | $m_s = 5$ |
| --- | --- | --- | --- | --- |
| $3$ | $286$ | $298$ | $322$ | $346$ |
| $8$ | $616$ | $638$ | $682$ | $726$ |
| $12$ | $880$ | $910$ | $970$ | $1{,}030$ |
| $15$ | $1{,}078$ | $1{,}114$ | $1{,}186$ | $1{,}258$ |
| $20$ | $1{,}408$ | $1{,}454$ | $1{,}546$ | $1{,}638$ |

The reference configuration is the $638$ entry, at eight Sentinels and one eligible axis apiece. The marginal cost of one further Sentinel is $66 + 2 m_s$ features, and of one further outcome axis $2n\,\mathbb{1}[\text{spatial}] + 3D$ features, which is what makes adding a source after convergence a bounded operation rather than a re-architecture (`prop:resource:incremental-addition`).

At the reference configuration, the aggregate block is fifteen, the slot width is sixty-three, eight slots give five hundred and four, and the dimension identity totals six hundred and thirty-eight. The Sentinel-slot row is the arithmetic link to the extraction reference — sixty-one is the occupancy indicator plus the sixty fixed extraction positions, and the $2 m_s$ is the per-axis Ledger pair (`tab:extraction:reference-index`).
