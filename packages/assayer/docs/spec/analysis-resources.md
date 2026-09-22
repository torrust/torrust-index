### Chapter (Convergence and Resource Bounds) · `chap:spec:convergence-and-resources`

What the system costs to converge and what it costs to run, in one place. The chapter is arithmetic on parameters fixed elsewhere: the dimension the feature chapter constructs, the update the label chapter performs, the buffer the publication chapter sizes. It fixes one thing of its own, which is the reference configuration every figure in the document is quoted at.

The cost tables are cited from the operational chapters rather than duplicated into them. That is deliberate and it is the discipline that keeps two copies of a cost from drifting apart: the tables mint here and the chapters that spend the cost cite them.

**Bound (The convergence budget)** · `bound:resource:convergence-budget`

A Bayesian linear model with $p$ parameters needs on the order of $2p$ observations before its posterior is shaped by data rather than by its prior. That is the whole of the rule of thumb, and everything below is it evaluated somewhere.

The system has two independent timelines and the host waits for the later:

$$T_{\text{system}} = \max\!\big(T_{\text{core}},\; T_{\hat{q}_c}\big)$$

The Core's own timeline is sequential, because each stage needs the one before it:

$$T_{\text{core}} = T_{\text{spatial}} + T_{\text{warm-up}} + T_{\text{baseline}} + T_{\text{assayer}}$$

| Stage | Formula | Typical range |
| --- | --- | --- |
| $T_{\text{spatial}}$ | $\theta / R_{\text{obs}}$ | Seconds to hours |
| $T_{\text{warm-up}}$ | $\text{noise rounds} / R_{\text{noise}}$ | Seconds to minutes |
| $T_{\text{baseline}}$ | $400 / R_{\text{batch}}$ | Minutes to hours |
| $T_{\text{assayer}}$ | $2p / (R_{\text{label}} \times f_{\text{elig}})$ | Days to months |
| $T_{\hat{q}_c}$ | $20 / R_{\text{contributing}}$ | Days to months |

The first three stages belong to the spatial index and the spectral Sentinel rather than to this system, which observes their completion only indirectly through the batch report; they are named here because a host waiting for calibrated output waits for them too, and they are measured in seconds and minutes against the last stage's days and months. The Companion's timeline runs in parallel and is not part of the sum; its row above is the no-decay arrival time, and at untargeted rates the effective sample size ceilings below twenty, so the wait there is not long but unending (`bound:companion:convergence`). A host reading risk assessments alone waits for $T_\text{core}$ and nothing else.

The last stage is the one that matters, and it is $2p$ eligible labels divided by the rate at which eligible labels arrive — the single convention used wherever a convergence figure appears in this document. Eligibility is what turns a label rate into an arrival rate (`tab:eligibility:training`), so a deployment that labels heavily but confounds most of it converges slowly:

| Deployment | $n$ | $m_s$ | $p$ | Eligible labels to the sister | At 200 labels/day, 60% eligible |
| --- | --- | --- | --- | --- | --- |
| Minimal | 3 | 0 | 286 | 572 | ~4.8 days |
| Standard | 8 | 1 | 638 | 1,276 | ~10.6 days |
| Standard+ | 8 | 3 | 682 | 1,364 | ~11.4 days |
| Large | 12 | 1 | 910 | 1,820 | ~15.2 days |

Each $p$ above is the dimension formula evaluated at that deployment's Sentinel and axis counts rather than an estimate (`tab:feature:dimension-formula`), and each day figure is $2p$ divided by the hundred and twenty eligible labels a day that the stated rates produce. The anchor is exempt from all of it: fifteen parameters converge within about thirty labels whatever the deployment's size, which is why the system has a usable direction long before it has a converged sister (`def:risk:anchor-model`).

**Proposition (Adding a Sentinel or axis after convergence)** · `prop:resource:incremental-addition`

Adding a Sentinel to a converged system does not reconverge it. The new dimensions are independent of the existing ones under the prior (`thm:gaussian:extension`), so the model needs on the order of $q + 1$ eligible labels to learn the new Sentinel's weights rather than the $2p$ a cold start costs, and the existing dimensions carry on at their converged values while the new ones fill in.

Adding an outcome axis costs on two accounts, which are worth separating. Every existing model gains the axis's feature weights and needs on the order of $r_a$ labels to learn them, which is incremental in the same sense. The axis's own prediction model is a new model, starting from its prior at dimension $p + r_a$, and it needs on the order of $p + r_a$ labels — a full convergence, because it is a full model (`def:axis:per-axis-model`).

The property is what makes the registries usable in production rather than only at design time. A deployment can add a source of evidence without paying the cold start again, and the cost of doing so is legible in advance from the width the new source contributes (`def:extraction:slot`).

**Table (Per-assessment cost)** · `tab:resource:assessment-cost`

| Operation | Cost |
| --- | --- |
| Coordinate routing, all Sentinels | $O(n_s \log \lvert\mathcal{A}^*\rvert)$ |
| Per-Sentinel extraction and alarm summary | $O(n \times q)$ |
| Aggregation | $O(n)$ |
| Identity observation and features | $O(D \cdot d_\text{geo} + \sum_d \lvert\mathcal{E}_d\rvert)$ |
| Feature assembly and standardisation | $O(p)$ |
| Risk estimates, two at $p$ and the anchor at $p_a$ | $O(2p^2 + p_a^2)$ |
| Outcome predictions, $m$ models | $O(m \cdot p^2)$ |
| Pending buffer write | $O(p)$ |
| **Total at $p = 638$, $m = 1$** | **~1.22M operations, ~1.22 ms** |

The quadratic terms dominate and they are the posterior reads, which is the expected shape for a model of this class (`tab:gaussian:operation-costs`). The pipeline these operations belong to is specified with the assessment interface (`alg:runtime:assessment-pipeline`). The derivation's own cost is not in this table and does not belong in it: it is linear in the declared actions plus a constant, negligible beside the above, and paid by the host outside the Core (`sig:landscape:derivation-function`).

**Table (Per-label cost)** · `tab:resource:label-cost`

| Operation | Cost |
| --- | --- |
| Reconstruction and re-standardisation | $O(p)$ |
| Operational model update | $O(p^2)$ |
| Sister model update, if eligible | $O(p^2)$ |
| Anchor model update, if eligible | $O(p_a^2)$ |
| Outcome axis models, if eligible | $O(m_\text{elig} \cdot p^2)$ |
| Standardisation update | $O(p)$ |
| Ledger update, all-layers routing | $O(n_s \cdot d_\text{max})$ |
| Calibration refit, amortised | $O(N_\text{cal,buf}) / N_\text{refit}$ |
| **Total at $p = 638$, $m = 1$** | **~1.22M operations, ~1.22 ms** |

A label costs about what an assessment costs, which is the useful thing to know when sizing a deployment: the label path is not the cheap path. The steps are enumerated where the update is specified (`alg:runtime:update-path`), and the conditional rows are conditional on the eligibility table rather than on anything this chapter decides. No challenge-effectiveness update appears here, because the Companion is updated by the host and not on this path (`alg:companion:update`).

**Table (Memory summary)** · `tab:resource:memory`

At the reference configuration, with model matrices in double precision:

| Component | Size |
| --- | --- |
| Core models, two at $p = 638$ | ~12.4 MB |
| Anchor model, $p_a = 15$ | ~7.4 KB |
| Outcome axis models, $m = 1$ | ~6.2 MB |
| Standardisation | ~10.2 KB |
| Calibration buffer | ~64 KB |
| Identity layer, two graphs | ~30 MB |
| Signal cache, 100K entities | ~8 MB |
| Pending buffer, single-precision features | **~3.0 GB at 1M capacity** |
| Outcome Ledger | ~32 MB |
| Routing indices | ~2 MB |
| **Core total** | **~3.1 GB at 1M capacity** |
| Companion tracker, per channel | ~100 bytes |

Each full model holds a mean, a precision and a covariance, and the two matrices dominate it at $2p^2$ doubles apiece; per-model memory therefore grows as $O(p^2)$, and double precision is fixed because the precision-health machinery depends on it (`alg:gaussian:synchronisation-monitor`). Everything else on the list is rounding error beside one row. The pending buffer is ninety-seven per cent of the total, it stores features at single precision (`def:runtime:storage-precision`), and right-sizing its capacity to the deployment's request rate and labelling latency is the only memory lever worth pulling (`req:publication:pending-buffer`). The one-million figure is the capacity at which this table is computed; the label chapter instead specifies capacity as a value computed from rate and latency (`req:runtime:buffer-capacity`), so a deployment that chooses a million entries is paying three gigabytes for that capacity.

**Table (The reference configuration)** · `tab:resource:reference-configuration`

| Parameter | Value |
| --- | --- |
| Sentinels ($n$) | 8 |
| Outcome axes ($m$) | 1, with spatial features enabled, so $m_s = 1$ |
| Identity dimensions ($D$) | 2, at about ten competitive cells each |
| $p_\text{sig}$ | 0 |
| Per-Sentinel extraction width ($q$) | $60 + 2 \times 1 = 62$ |
| Slot width | $q + 1 = 63$ |
| $p_\text{agg}$ | 15 |
| $p_\text{id-dim}$, per dimension | $8 + 3 \times 1 = 11$ |
| $p_\text{id-cross}$ | 8 |
| $p_\text{int}$ | $5 \times 8 + 8 + 20 = 68$ |
| $p_\text{id-comp}$ | $10 + 10 = 20$ |
| **$p$** | $1 + 15 + 11 \times 2 + 8 + 0 + 8 \times 63 + 68 + 20 = \mathbf{638}$ |

This is the configuration every worked figure in the document is quoted at, and it is stated as a table so that a reader can tell which of those figures move when a deployment differs. The construction is the feature vector's blocks in order — the bias, the aggregate block (`tab:feature:aggregate-block`), the per-dimension and cross-dimension identity blocks (`tab:keyspace:dimension-features`), the signal block, the eight Sentinel slots, the interaction block and the competitive indicators (`def:feature:vector-structure`) — and the full construction with every block's derivation is the dimensional summary (`app:spec:dimensional-summary`).

The total is exact rather than approximate: the blocks sum to six hundred and thirty-eight. The same construction evaluated at the other three deployment sizes of this chapter's budget gives their dimensions exactly too, which is the check worth having, because a reference configuration whose arithmetic only works at one point is a worked example rather than a reference.
