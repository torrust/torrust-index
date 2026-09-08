# Spectral Sentinel — Algorithm Specification · `spec:sentinel:algorithm-specification`

---

# Part I — Scope and Foundations · `sec:sentinel:algorithm-scope-and-foundations`

---

## Chapter 1. Purpose, Scope, and Principles · `sec:sentinel:algorithm-purpose-scope-and-principles`

### 1.1 Overview · `sec:sentinel:algorithm-purpose-overview`

The Spectral Sentinel is a hierarchical online anomaly detector for streams of positionally structured coordinate values. It maintains adaptive spatial partitioning of an $N$-bit input domain $[0, 2^N)$, selects significant regions for statistical analysis via competitive ranking, and scores incoming observations against learned low-rank linear subspace models.

### 1.2 Parameterisation · `sec:sentinel:algorithm-parameterisation`

The system is defined over three parameters:

| Parameter | Role                                | Requirements                                                                                                                                                       |
| --------- | ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| $N$       | Domain bit-width (positive integer) | Determines tree height and domain cardinality $2^N$. Typical values: 128, 64.                                                                                      |
| $C$       | Coordinate type                     | Total ordering over $[0, 2^N)$; bit-positional decomposition; dyadic interval arithmetic (§2.3).                                                                   |
| $V$       | Accumulator type                    | Non-negative values with a zero element; addition; multiplicative scaling by a non-negative factor; approximate conversion to floating-point for reporting (§3.3). |

All formulas throughout this specification are stated in terms of $N$. Concrete examples use $N = 128$ unless otherwise noted.

### 1.3 Design Principles · `sec:sentinel:algorithm-design-principles`

**Measure, don't decide.** All outputs are raw statistical quantities. The system never emits threat levels, recommended actions, or policy decisions.

**Adapt, don't control.** The spatial structure evolves autonomously under observation and decay. The host controls resource ceilings, temporal policy, and analysis budgets.

**Feed forward, don't feed back.** Anomaly scores flow outward to the host but are never fed back into the spatial layer's importance signal. The spatial layer sees $\Delta = 1$ per observation — pure volume counting. This prevents the anomaly detector from influencing its own spatial structure.

> _Design note (why volume-only importance)._ If anomaly scores boosted importance, the affected cell would earn finer resolution, changing its statistical model, changing its scores, changing its importance — an unstable feedback loop. With $\Delta = 1$, an adversary cannot influence the spatial structure except through observation volume, which is precisely what the spatial layer is designed to handle.

> _Design note (why timing matters)._ An observer who can measure response latency learns when the spatial structure changes: a spike means "a new cell was created for this region," leaking distribution evolution and split dynamics. Moving warm-up work off the hot path (§11) eliminates the structural source of variance.

### 1.4 Three-Layer Architecture · `sec:sentinel:algorithm-three-layer-architecture`

```
Layer 1: Spatial Index
         Adaptive spatial partitioning of [0, 2^N)
         Pure volume tracking (Δ = 1 per observation)
         Competitive ranking by observation volume
              │
              │ Significance ranking → top-K selection
              ▼
Layer 2: Analysis Selector
         Selects significant cells for statistical analysis
         Closes selection under spatial ancestry
              │
              │ Suffix bit vectors at every ancestor depth
              ▼
Layer 3: Analysis Engine
         Per-cell subspace models at selected and ancestor cells
         Hierarchical coordination across related cells
              │
              ▼
         Analysis report → host
```

### 1.5 Layer Responsibilities · `sec:sentinel:algorithm-layer-responsibilities`

| Concern                                                          | Owner                                       |
| ---------------------------------------------------------------- | ------------------------------------------- |
| Spatial partitioning of $[0, 2^N)$                               | Spatial Layer (Layer 1)                     |
| Competitive significance ranking                                 | Spatial Layer — Value Tree                  |
| Spatial lifecycle (split, evict, absorb, restore)                | Spatial Layer                               |
| Spatial memory (temporal decay, contour evolution)               | Spatial Layer + host policy                 |
| Investment commitment (which cells receive warm-up and trackers) | Analysis Selector (Layer 2)                 |
| Production selection (which invested cells produce scores)       | Analysis Selector (Layer 2)                 |
| Ancestor closure (spatial ancestry of investment targets)        | Analysis Selector (Layer 2)                 |
| Statistical modelling within each cell                           | Analysis Engine (Layer 3)                   |
| Anomaly scoring (four axes)                                      | Analysis Engine                             |
| Drift detection                                                  | Analysis Engine                             |
| Cross-cell coordination detection                                | Analysis Engine — hierarchical coordination |
| Interpretation and response                                      | Host (external)                             |

### 1.6 Responsibility Boundaries · `sec:sentinel:algorithm-responsibility-boundaries`

| Property                                       | Source                                 |
| ---------------------------------------------- | -------------------------------------- |
| Contour structure, ranking, budget enforcement | Inherited — Spatial Layer (§3)         |
| Temporal decay schedule                        | Host policy, executed by Spatial Layer |
| Investment set and producing set membership    | Sentinel — Layer 2 (§8)                |
| Statistical modelling and scoring              | Sentinel — Layer 3 (§4–7)              |
| Importance signal ($\Delta = 1$, no feedback)  | Sentinel's invariant (§1.3)            |

---

## Chapter 2. Domain, Encoding, and Notation · `sec:sentinel:algorithm-domain-encoding-and-notation`

### 2.1 The Domain · `sec:sentinel:algorithm-domain`

The input domain is $[0, 2^N)$, where $N$ is the domain bit-width. The spatial layer partitions this into dyadic cells aligned with bit positions. A cell at spatial tree depth $d$ covers an interval of width $2^{N-d}$, corresponding to a $d$-bit prefix shared by all values in the cell.

**Example depths at $N = 128$:**

| Spatial tree depth $d$ | Prefix length | Suffix width $w$ | Potential cells at full coverage |
| ---------------------- | ------------- | ---------------- | -------------------------------- |
| 0                      | 0 bits        | 128              | 1                                |
| 16                     | 16 bits       | 112              | 65,536                           |
| 32                     | 32 bits       | 96               | $\approx 4.3 \times 10^{9}$      |
| 48                     | 48 bits       | 80               | $\approx 2.8 \times 10^{14}$     |
| 64                     | 64 bits       | 64               | $\approx 1.8 \times 10^{19}$     |
| 96                     | 96 bits       | 32               | $\approx 7.9 \times 10^{28}$     |
| 128                    | 128 bits      | 0                | $\approx 3.4 \times 10^{38}$     |

At $N = 64$, the maximum depth is 64 and the root suffix width is 64.

The spatial layer materialises only cells where observations have warranted refinement. The bottom contour (§3.2) is the observation-receiving surface — deep where volume is concentrated, shallow where it is sparse. The contour determines where observations route; the analysis selector separately decides where statistical trackers run by scanning V-Tree entries by competitive significance and analysis width, then adding G-Tree ancestors for multi-scale context (§8).

### 2.2 Input Requirements · `sec:sentinel:algorithm-input-requirements`

The system analyses bit-positional structure: leading bits determine routing, suffix bits provide statistical content. This is meaningful only when the coordinate values have **hierarchical positional structure** — values whose leading bits encode progressively finer categorical or spatial membership, so that shared prefixes imply shared context.

Values with pseudo-random bit distributions (cryptographic hashes, random nonces, uniformly sampled identifiers) have no such structure and defeat the analysis. The system processes any stream of coordinate values without complaint; the host is responsible for the structural guarantee.

### 2.3 Centred Bit Representation · `sec:sentinel:algorithm-centred-bit-representation`

Each raw coordinate value $v$ of type $C$ becomes a centred bit vector $\mathbf{x} \in \{-0.5, +0.5\}^{N}$:

$$x_i = \begin{cases} +0.5 & \text{if bit } (N - 1 - i) \text{ of } v \text{ is 1} \\ -0.5 & \text{otherwise} \end{cases} \qquad i = 0, \ldots, N-1$$

Index 0 is the most significant bit. Centring gives $\mathbb{E}[x_i] = 0$ under a uniform bit distribution — a prerequisite for subspace analysis without explicit mean subtraction.

The coordinate type $C$ must support:

- Total ordering over $[0, 2^N)$.
- Extraction of individual bits by position (for the centred representation above).
- Dyadic interval arithmetic: midpoint computation and interval containment testing (for spatial routing).

### 2.4 Suffix Extraction · `sec:sentinel:algorithm-suffix-extraction`

For a cell at depth $d$, the first $d$ prefix bits are constant — resolved by routing. The working observation is the **suffix**:

$$\mathbf{x}^{(\text{cell})} = (x_d, \ldots, x_{N-1}) \in \{-0.5, +0.5\}^w, \quad w = N - d$$

### 2.5 Constant-Norm Property · `sec:sentinel:algorithm-constant-norm-property`

Every suffix vector at width $w$ satisfies $\|\mathbf{x}^{(\text{cell})}\|^2 = w/4$. This fixed-energy property means that total observation energy is constant and carries no information. By the Pythagorean theorem, projection energy $\|\hat{\mathbf{x}}_i\|^2/k$ is a perfect affine function of novelty (Pearson $r = -1$). The system therefore uses four independent scoring axes rather than five (§5).

> _Note._ This constraint is specific to centred binary inputs. Continuous-valued inputs with variable norms would decouple projection energy from novelty.

### 2.6 Notation · `sec:sentinel:algorithm-notation`

| Symbol                | Domain                                      | Definition                                                                                                                                                                                                              |
| --------------------- | ------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| $N$                   | $\mathbb{Z}_{>0}$                           | Domain bit-width; spatial tree height                                                                                                                                                                                   |
| $C$                   | —                                           | Coordinate type; spatial addressing, interval bounds (§2.3)                                                                                                                                                             |
| $V$                   | —                                           | Accumulator type; importance accounting in the spatial layer (§3.3)                                                                                                                                                     |
| $d$                   | $\{0, \ldots, N\}$                          | Spatial tree depth of a cell (prefix length in bits)                                                                                                                                                                    |
| $w$                   | $\{0, \ldots, N\}$                          | Analysis width; $w = N - d$ (suffix length)                                                                                                                                                                             |
| $n$                   | $\mathbb{Z}_{\geq 0}$                       | Ingestion batch size (coordinate values per `SentinelIngest` call)                                                                                                                                                      |
| $b$                   | $\mathbb{Z}_{>0}$                           | Per-tracker batch size (rows of $X$ fed to one tracker in one call; see batch-size note below)                                                                                                                          |
| $b_{\text{noise}}$    | $\mathbb{Z}_{>0}$                           | Noise batch size (synthetic samples per warm-up round; §11.1)                                                                                                                                                           |
| $k$                   | $\{1, \ldots, \text{cap}\}$                 | Current active rank of the learned subspace (§4)                                                                                                                                                                        |
| $\text{cap}$          | $\{1, \ldots, \min(w, r_{\max})\}^\dagger$  | Hard ceiling on rank; $r_{\max}$ is the maximum rank parameter                                                                                                                                                          |
| $\lambda$             | $(0, 1)$                                    | Forgetting factor                                                                                                                                                                                                       |
| $\alpha$              | $(0, 1)$                                    | Learning rate; $\alpha = 1 - \lambda$                                                                                                                                                                                   |
| $\varepsilon$         | $\mathbb{R}_{>0}$                           | Numerical stability constant                                                                                                                                                                                            |
| $\tau$                | $(0, 1)$                                    | Cumulative energy threshold                                                                                                                                                                                             |
| $U$                   | $\mathbb{R}^{w \times \text{cap}}$          | Orthonormal basis of the learned subspace (§4)                                                                                                                                                                          |
| $\sigma$              | $\mathbb{R}^{\text{cap}}_{\geq 0}$          | Singular values (energy per basis vector) (§4)                                                                                                                                                                          |
| $\mu^{(z)}$           | $\mathbb{R}^{\text{cap}}$                   | EWMA mean of latent coordinates (§4)                                                                                                                                                                                    |
| $\nu^{(z)}$           | $\mathbb{R}^{\text{cap}}_{>0}$              | EWMA variance of latent coordinates (§4)                                                                                                                                                                                |
| $\Gamma$              | $\mathbb{R}^{\text{cap} \times \text{cap}}$ | EWMA second-moment matrix of latent coordinates (§4)                                                                                                                                                                    |
| $X$                   | $\mathbb{R}^{b \times w}$                   | Suffix observation matrix (one batch at one cell)                                                                                                                                                                       |
| $Z$                   | $\mathbb{R}^{b \times k}$                   | Latent projection of $X$                                                                                                                                                                                                |
| $\hat{X}$             | $\mathbb{R}^{b \times w}$                   | Reconstruction of $X$ from $Z$                                                                                                                                                                                          |
| $m$                   | $\mathbb{Z}_{>0}$                           | Coordination group size (§7)                                                                                                                                                                                            |
| $\lambda_s$           | $(\lambda, 1)$                              | Slow EWMA decay for per-tracker drift detection (§6)                                                                                                                                                                    |
| $\lambda_{s,m}$       | $(\lambda, 1)$                              | Slow EWMA decay for coordination drift detection (§7)                                                                                                                                                                   |
| $\kappa_\sigma$       | $\mathbb{R}_{\geq 0}$                       | Drift-detector noise allowance in slow-baseline $\sigma$ units (§6)                                                                                                                                                     |
| $n_\sigma$            | $\mathbb{R}_{> 0}$                          | Outlier clip width in $\sigma$ units (§6)                                                                                                                                                                               |
| $S$                   | $\mathbb{R}_{\geq 0}$                       | Drift-detector accumulator value (§6)                                                                                                                                                                                   |
| $\bar{\rho}$          | $[0, 1]$                                    | Per-axis clip-pressure EWMA (§6.1.1, §6.4)                                                                                                                                                                              |
| $\lambda_\rho$        | $(0, 1)$                                    | Clip-pressure EWMA decay rate (§6.1.1, §6.4)                                                                                                                                                                            |
| $\rho_t$              | $[0, 1]$                                    | Batch clip ratio: fraction of samples clipped in one batch (§6.1.1)                                                                                                                                                     |
| $\mu^{(\text{in})}$   | $\mathbb{R}^4$                              | Running-mean centring reference for coordination input (§7)                                                                                                                                                             |
| $\eta$                | $[0, 1]$                                    | Noise influence fraction (tracker maturity) (§11)                                                                                                                                                                       |
| $K$                   | $\mathbb{Z}_{>0}$                           | Maximum number of competitively selected analysis cells                                                                                                                                                                 |
| $L$                   | $\mathbb{Z}_{\geq 0}$                       | Value Tree depth cutoff for analysis eligibility                                                                                                                                                                        |
| $\mathcal{T}$         | $\subseteq$ V-entries                       | Competitive targets (top $K$ by importance within depth cutoff) (§8)                                                                                                                                                    |
| $\mathcal{A}$         | $\subseteq$ V-entries                       | Producing competitive set ($\mathcal{I} \cap \mathcal{T} \cap \text{Online}$) (§8)                                                                                                                                      |
| $\mathcal{A}^*$       | $\supseteq \mathcal{A}$                     | Producing full set ($\mathcal{I} \cap \text{Online}$) (§8)                                                                                                                                                              |
| $\mathcal{I}$         | $\subseteq$ V-entries + ancestors           | Investment set (competitive targets + spatial ancestors, regardless of online status) (§8)                                                                                                                              |
| $G_{\max}$            | $\mathbb{Z}_{\geq 5}$                       | Hard ceiling on total spatial nodes                                                                                                                                                                                     |
| $\lambda_{\text{sp}}$ | $[0,\infty)$                                | Spatial decay attenuation factor (host-controlled) (§10)                                                                                                                                                                |
| $h_V$                 | $\mathbb{Z}_{\geq 0}$                       | V-Tree height (maximum root-to-leaf path length in the V-Tree) (§3.9)                                                                                                                                                   |
| $d_{\text{geo}}$      | $\mathbb{Z}_{\geq 0}$                       | G-Tree depth of a node: the number of materialised ancestors on the path from the node to the root (= prefix depth $d$) (§3.1).                                                                                         |
| $\bar{D}$             | $\mathbb{R}_{\geq 0}$                       | Average G-Tree depth of competitive targets; $\bar{D} = \frac{1}{K}\sum_{i=1}^{K} d_i$. Typical values 2–6 under default parameters (§3.1). The worst-case investment set size is $1 + K\bar{D}$ before sharing (§8.2). |

$^\dagger$ The domain $\{1, \ldots, \min(w, r_{\max})\}$ is non-empty only when $w \geq 1$. Tracker creation requires $w \geq 2$ (§4.1); cells below this threshold are excluded from $\mathcal{I}$.

> **Batch-size note.** Three distinct quantities govern how many rows a tracker processes per call. **Ingestion batch size** ($n$) is caller-controlled: the number of coordinate values submitted in one `SentinelIngest` call. **Per-tracker batch size** ($b$) is the row count of $X$ at a given tracker. At the root, $b = n$ (every observation routes through the root). At non-root cells during live operation, $b$ is the number of ingestion-batch observations whose spatial routing passes through that cell; $b \in \{0, \ldots, n\}$ depending on the batch's spatial distribution. Cells with $b = 0$ receive no observations and skip the core loop entirely. **Noise batch size** ($b_{\text{noise}}$) is a configured parameter (§11.1) controlling synthetic samples per warm-up round. All core-loop formulas (§4–§6) are parametric in $b$; the distinction matters for cost analysis (§12) and warm-up calibration (§11.9, Appendix A).

Vectors are row vectors when they represent observations and column vectors when they represent basis directions. Subscript $i$ indexes samples; subscript $j$ indexes subspace dimensions.

---

## Chapter 3. The Spatial Layer · `sec:sentinel:algorithm-spatial-layer`

This chapter specifies the capabilities and contracts of the underlying spatial partitioning structure — referred to throughout this specification as the **G-V Graph** — to the extent required for understanding the Spectral Sentinel. It is not a complete specification of the G-V Graph; it describes what the Sentinel uses, at the level of detail needed to follow the algorithms in subsequent chapters and reason about their properties.

---

### 3.1 Purpose and Architecture · `sec:sentinel:algorithm-spatial-purpose-and-architecture`

The G-V Graph is a self-organising spatial index that continuously solves a precision-allocation problem over a one-dimensional domain $[0, 2^N)$. It decides, based on accumulated observation volume, where to invest fine spatial resolution and where to leave resolution coarse. The Sentinel uses this adaptive partitioning as the foundation for its analysis: cells that the G-V Graph identifies as significant receive statistical modelling; cells it identifies as insignificant do not.

The structure consists of two trees sharing a common set of nodes. Each tree owns a different aspect of the system's state:

**The Geometric Tree** **(G-Tree)** is a binary tree over dyadic intervals. It is the spatial ledger: every materialised node stores the accumulated observation value for its range. The G-Tree maintains a **contour** — the observation-receiving surface of the domain — a step function whose depth at each coordinate reflects how finely the tree has resolved that region.

**The Value Tree** **(V-Tree)** is a dynamic tournament bracket with branching factor 2 or 3. Nodes from the G-Tree sit at the leaves as competitors; internal nodes are pure structural scaffolding. The V-Tree ranks competitors by **importance** — accumulated observation volume, under the configuration the Sentinel uses. High-importance entries reside near the root; low-importance entries are consolidated deeper.

Both trees reference the same underlying nodes. A node exists simultaneously in both trees: it occupies a position in the G-Tree's dyadic hierarchy (determined by its interval) and a position in the V-Tree's tournament bracket (determined by its competitive importance).

**Uncompressed materialisation.** The G-Tree is a fully materialised binary trie: every node on the path from a leaf to the root exists as a distinct materialised node. A cell at G-Tree depth $d$ has exactly $d$ materialised ancestors. Catalytic bisection (§3.5) creates children at depth $d + 1$; tip-only eviction (§3.12, property 5) removes leaves but never compresses interior chains. Ancestor walks — for sum propagation, investment-set closure (§8.2), and multi-scale delivery (§9.3) — visit every materialised level.

> _Why trees stay shallow._ The competitive mechanism (§3.4) and the host-configured depth gate `depth_create` (typically 3) bound how deep the G-Tree grows in practice. Under typical parameters, competitive cells sit at G-Tree depths 2–8, so ancestor counts are inherently small — not because of path compression, but because the system rarely creates deep structure. The worst-case investment set size before sharing is $1 + K\bar{D}$ (§8.2), but the Steiner tree structure of the ancestor closure guarantees extensive sharing, and the practical size is dominated by $2K$.

### 3.2 Domain and Spatial Partitioning · `sec:sentinel:algorithm-spatial-domain-and-partitioning`

The input domain is $[0, 2^N)$, partitioned into dyadic cells aligned with bit positions. A cell at G-Tree depth $d$ covers an interval of width $2^{N-d}$, corresponding to a $d$-bit prefix shared by all values in the cell. The root covers the entire domain at depth 0; a unit cell at depth $N$ covers a single value.

The G-V Graph materialises only cells where observations have warranted refinement. The **bottom contour** is the observation-receiving surface of the domain — a step function whose depth at each coordinate reflects how finely the tree has resolved that region. Deep where volume is concentrated, shallow where it is sparse. The contour determines where observations route; the Sentinel's analysis selection (§8.1) determines, separately, where statistical trackers run. The analysis set includes cells above the contour (internal G-Tree nodes serving as ancestor trackers), fed through multi-scale delivery rather than spatial routing.

The contour is organised into **plateaus** — maximal contiguous runs at a single depth. Within a plateau, all contour cells sit at the same depth and interval width. The boundaries between plateaus mark where the tree found non-uniformity worth resolving. The plateau count $P$ is a natural measure of the tree's structural complexity: $P = 1$ for a perfectly uniform tree; $P$ grows as the tree learns structure.

### 3.3 Observation and Importance · `sec:sentinel:algorithm-spatial-observation-and-importance`

When a value $v$ at coordinate $x$ arrives, the G-V Graph:

1. **Routes** to the receiving cell — the node where `RouteToReceiver(x)` terminates because no child exists in the direction of $x$.
2. **Accumulates** the observation delta $\Delta$ in the receiving cell's ledger.
3. **Updates importance** — the receiving cell's competitive ranking value grows.
4. **Propagates** sums upward through both trees.
5. **Checks structural triggers** — whether the cell qualifies for refinement, whether competitive violations need resolution, whether cold cells should be evicted.

The Sentinel uses the G-V Graph in its **standard configuration**: importance equals accumulated observation volume, the ground element is zero, and the importance type is non-negative. Under this configuration, all architectural features are available — proportional sampling, the Fibonacci depth bound, violation-free splits, and fast eviction of unobserved cells.

**The Sentinel's feed-forward invariant.** The Sentinel always observes with $\Delta = 1$ — pure volume counting. Anomaly scores are never fed back into the importance signal. This prevents the anomaly detector from influencing its own spatial structure.

The accumulator type $V$ must support:

- Non-negative values with a zero element.
- Addition (for accumulation of $\Delta$).
- Multiplicative scaling by a non-negative real factor (for temporal decay, §3.6).
- Approximate conversion to a floating-point value (for reporting and comparison).

### 3.4 The Competitive Mechanism · `sec:sentinel:algorithm-spatial-competitive-mechanism`

The V-Tree is governed by the **max-uncle constraint**: no node may outrank every one of its uncles (siblings of its parent at the grandparent level). This constraint encodes competitive dominance and drives the tree's lifecycle.

When a cell splits, its V-Tree entry freezes — children intercept all future observations, so the parent's importance stops growing. The frozen entry becomes the competitive benchmark that children must exceed. Children start at zero importance and must earn their way up. When a child's importance exceeds every uncle's, the V-Tree restructures — promoting the child to a shallower position and potentially triggering further spatial refinement.

Three siblings of comparable importance coexist indefinitely under a 3-node parent — a violation requires beating _both_ uncles. The V-Tree restructures only when a node dramatically outgrows its entire neighbourhood, not on every minor importance fluctuation. This structural stability is inherent — no additional hysteresis is needed.

**V-Tree depth as a significance measure.** The competitive mechanism pushes high-importance entries to shallow V-Tree depths and low-importance entries deep. V-Tree depth is therefore a global significance ranking: shallow entries have proven sustained competitive importance against their neighbourhood; deep entries have not. The Sentinel's analysis selector (§8) uses V-Tree depth as the eligibility criterion for statistical modelling.

The max-uncle constraint implies at least Fibonacci-rate decay along root-to-leaf paths, yielding a depth bound of $\log_\varphi(1/w_i) + O(1)$ for an entry with weight fraction $w_i$, where $\varphi = (1+\sqrt{5})/2 \approx 1.618$. Expected proportional sampling cost is at most $1.44 H + O(1)$ where $H$ is the Shannon entropy of the importance distribution.

### 3.5 Spatial Lifecycle · `sec:sentinel:algorithm-spatial-lifecycle`

Three forces shape the contour:

**Refinement.** When a fully exposed contour cell accumulates sufficient importance and holds a shallow enough V-Tree position, it splits into two finer cells. The split is _catalytic_: the parent persists as a frozen competitive benchmark; its children start at zero importance and must earn their way up. No information is destroyed by spatial refinement.

**Eviction.** Unprotected contour cells (zero children) that sit past a configurable V-Tree depth threshold are removed. The parent absorbs the evicted cell's accumulated value and partially or fully re-joins the contour. Eviction proceeds from the tips inward — only nodes with no dependents can be removed — so the contour coarsens gradually, never catastrophically.

**Restoration.** When a semi-internal node (one child present, one evicted) accumulates enough importance through the competitive mechanism, the V-Tree promotes it. This promotion creates the missing child as a side effect, restoring the contour without any separate operation. The V-Tree's competitive mechanism is the sole gate for contour growth.

**Depth gates** govern the lifecycle. Two V-Tree depth thresholds — a creation gate and an eviction gate separated by a mandatory buffer zone — control where new resolution may be added and where it is withdrawn. A node-count budget enables dynamic adjustment of these thresholds under memory pressure, providing a hard ceiling on total materialised nodes.

### 3.6 Temporal Decay · `sec:sentinel:algorithm-spatial-temporal-decay`

The G-V Graph stores exact accumulated values by default. It imposes no automatic temporal model. The host controls temporal semantics through an explicit decay operation that scales accumulators by a configurable factor:

- **Attenuation** (factor $< 1$): cold cells lose standing and become eviction candidates. This produces recency — "what matters now."
- **Amplification** (factor $> 1$): existing structure is reinforced. With depth-selective amplification, fine-scale detail is sharpened relative to coarse.
- **Annihilation** (factor $= 0$): hard reset. Targeted annihilation zeroes a subtree while leaving the rest of the graph intact.
- **Detail flush** (factor $= 0$, depth-selective): the subtree root is preserved while all descendants are zeroed — preserving coarse measurement while forcing fine structure to be re-earned.

Decay is subband-adaptive: different G-Tree depths can be scaled at different rates. This enables the host to implement frequency-selective temporal filtering — high-resolution subbands can decay faster than coarse ones.

The Sentinel's statistical decay (EWMA forgetting in trackers, §4) is independent of spatial decay. A cell can retain its spatial position under slow spatial decay while its statistical model adapts rapidly, and vice versa.

### 3.7 Dual Shielding · `sec:sentinel:algorithm-spatial-dual-shielding`

The two trees protect each other through three complementary mechanisms:

| Direction           | Shield                                           | What it protects                                                       |
| ------------------- | ------------------------------------------------ | ---------------------------------------------------------------------- |
| V-Tree → downward   | Parent's frozen entry stands as uncle            | Children from competitive displacement while the benchmark holds       |
| G-Tree → upward     | Children intercept observations meant for parent | Parent's V-entry from growing — it stays frozen as a fixed benchmark   |
| G-Tree → structural | Only unprotected nodes are eviction-eligible     | Protected nodes from premature removal while structurally load-bearing |

Without the upward shield, the parent's importance would keep pace with its children — no fixed benchmark, no competitive mechanism. Without the downward shield, children's positions would be unstable under every fluctuation. Without the structural shield, eviction could tear the contour by removing nodes that support finer-scale structure.

### 3.8 Benchmark Compounding · `sec:sentinel:algorithm-spatial-benchmark-compounding`

Repeated expand–contract cycles at a given node harden its competitive benchmark. Each eviction absorption folds descendants' accumulated value into the node's own accumulation, raising the frozen bar that future children must exceed. After $j$ full expand–contract cycles, the benchmark grows approximately geometrically, and the total observation cost to re-reach depth $D$ along a path grows quadratically: $\sim D^2 \theta / 2$ where $\theta$ is the split threshold. This ensures that deep spatial structure is re-created only when justified by sustained, concentrated observation volume.

Under temporal attenuation, benchmarks weaken — the bar softens as historical evidence ages. Under annihilation, benchmarks reset to zero — the region starts from scratch.

### 3.9 Capabilities Used by the Sentinel · `sec:sentinel:algorithm-spatial-sentinel-capabilities`

The Sentinel uses the following G-V Graph operations:

| Operation                        | Description                                          | Cost                               |
| -------------------------------- | ---------------------------------------------------- | ---------------------------------- |
| Observe(coordinate, $\Delta$)    | Route, accumulate, trigger structural updates        | $O(d_{\text{geo}} + h_V)$          |
| RouteToReceiver($x$)             | Find the receiving cell for coordinate $x$           | $O(d_{\text{geo}})$                |
| Decay(root, factor, selectivity) | Apply temporal scaling to a subtree                  | $O(\text{subtree size} \cdot h_V)$ |
| Value Tree depth query           | V-Tree depth of a cell's entry                       | $O(h_V)$ or $O(1)$ cached          |
| Value Tree importance query      | Importance value of a cell's entry                   | $O(1)$                             |
| Plateau point query              | Plateau containing a coordinate                      | $O(\log P)$                        |
| Plateau iteration                | All plateaus in spatial order                        | $O(P)$                             |
| Plateau count                    | Number of distinct plateaus                          | $O(1)$                             |
| G-Tree ancestor walk             | All materialised ancestors of a node                 | $O(d_{\text{geo}})$                |
| G-Tree node sum query            | g.sum of a node (own accumulation + descendant sums) | $O(1)$                             |
| Total importance                 | Sum of all importance in the graph                   | $O(1)$                             |
| Node count                       | Total materialised G-Tree nodes                      | $O(1)$                             |
| Semi-internal count              | Number of one-child nodes                            | $O(1)$                             |

### 3.10 Node States · `sec:sentinel:algorithm-spatial-node-states`

Every G-Tree node exists in one of three states, determined by how many children it has:

| Children | State         | Contour relationship               | Observation behaviour                         | Eviction eligible                     |
| -------- | ------------- | ---------------------------------- | --------------------------------------------- | ------------------------------------- |
| 0        | Terminal      | On the contour — fully exposed     | Receives all observations in its range        | Yes (if past depth gate and not root) |
| 1        | Semi-internal | On the contour — partially exposed | Receives observations in the uncovered half   | No (has a dependent)                  |
| 2        | Internal      | Above the contour                  | Receives no observations (children intercept) | No (has dependents)                   |

The Sentinel's analysis selector (§8) does not filter candidates by G-Tree state. It scans V-Tree entries within the configured V-depth cutoff and with sufficient analysis width, so a competitive target can be terminal, semi-internal, or internal. Internal nodes can remain competitive because their V-entry retains frozen pre-split importance, and their tracker is still fed through Sentinel's multi-scale interval delivery (§9).

Since the G-Tree is fully materialised (§3.1), the investment set's ancestor closure (§8.2) includes every node on the path from each competitive target to the root, providing hierarchical context at every dyadic scale from the target to the entire domain.

### 3.11 Configuration Parameters · `sec:sentinel:algorithm-spatial-configuration-parameters`

| Parameter                           | Role                                     | Sentinel's typical value            |
| ----------------------------------- | ---------------------------------------- | ----------------------------------- |
| $N$ (domain bit-width)              | Tree height, analysis width at root      | 128 (default) or 64                 |
| $\theta$ (split threshold)          | Minimum importance for split eligibility | Application-dependent               |
| $D_{\text{create}}$ (creation gate) | Maximum V-Tree depth for splits          | Application-dependent               |
| $D_{\text{evict}}$ (eviction gate)  | Minimum V-Tree depth for eviction        | $D_{\text{create}} + \text{buffer}$ |
| Budget                              | Soft node-count target                   | Application-dependent               |
| $G_{\max}$                          | Hard ceiling on total G-Tree nodes       | $> \text{budget}$, $\geq 5$         |

The Sentinel treats these as pass-through configuration: it forwards them to the G-V Graph at construction time and does not modify them during operation. The host controls all spatial policy through these parameters and through the timing and parameters of decay calls.

### 3.12 Key Properties · `sec:sentinel:algorithm-spatial-key-properties`

The following properties of the G-V Graph are assumed throughout this specification:

1. **Complete tiling.** The contour always tiles the full domain $[0, 2^N)$ with no gaps. Every coordinate has exactly one receiving cell.

2. **Summation invariant.** Every node's sum equals its own accumulation plus its children's sums. Total energy is conserved across all structural operations.

3. **Single-entry accounting.** Each observation updates exactly one node's importance — the receiver's. No double-counting.

4. **Competitive stability.** Three siblings of comparable importance coexist indefinitely. Restructuring requires a node to outgrow its entire neighbourhood.

5. **Tip-only eviction.** Only nodes with zero children can be evicted. The contour contracts from the tips inward.

6. **Root permanence.** The G-Tree root is never evicted. The tree always has at least one node.

7. **Frozen benchmarks.** Internal nodes' importance values are frozen — a direct consequence of observation routing, not an explicit mechanism.

8. **Budget enforcement.** The total node count never exceeds $G_{\max}$ across any single operation.

9. **Fibonacci depth bound.** Under the standard configuration, V-Tree depth is bounded by $\log_\varphi(1/w_i) + O(1)$ for weight fraction $w_i$.

10. **Feed-forward compatibility.** The G-V Graph accepts any non-negative $\Delta$ without interpreting it. The Sentinel's choice of $\Delta = 1$ is invisible to the graph.

---

# Part II — The Mathematical Model · `sec:sentinel:algorithm-mathematical-model`

---

## Chapter 4. The Subspace Model · `sec:sentinel:algorithm-subspace-model`

Each cell in the full analysis set $\mathcal{A}^*$ (§8) maintains a subspace tracker — a low-rank linear subspace model that scores observations against learned structure and then evolves.

### 4.1 State · `sec:sentinel:algorithm-subspace-state`

**Minimum analysis width.** All formulas in §4–§7 require $w \geq 2$. At $w = 0$, $\text{cap} = 0$ while $k$ would need to be 1 — the rank exceeds capacity, the basis $U$ is vacuous, and every formula in §4.2 and §5 is undefined. At $w = 1$, the tracker is novelty-saturated from birth ($\text{cap} = 1 = k$, residual DOF $= 0$) and the single basis vector spans the entire space, leaving no statistical content to detect departures from. Cells with $w < 2$ are excluded from the eligible set $\mathcal{E}$ (§8.1) and reported via the degenerate-cell counter (§14.11). The spatial layer continues to route and accumulate observations at these cells normally; only statistical modelling is withheld.

A tracker at analysis width $w$ with capacity $\text{cap} = \min(w, r_{\max})$ maintains:

| Component          | Shape                                                             | Description                                                |
| ------------------ | ----------------------------------------------------------------- | ---------------------------------------------------------- |
| $U$                | $(w, \text{cap})$                                                 | Orthonormal basis (columns $1{:}k$ active)                 |
| $\sigma$           | $(\text{cap},)$                                                   | Singular values ($1{:}k$ meaningful)                       |
| $\mu^{(z)}$        | $(\text{cap},)$                                                   | EWMA mean of latent coordinates ($1{:}k$ active)           |
| $\nu^{(z)}$        | $(\text{cap},)$                                                   | EWMA variance of latent coordinates ($1{:}k$ active)       |
| $\Gamma$           | $(\text{cap}, \text{cap})$                                        | EWMA second-moment matrix ($1{:}k$ active, upper triangle) |
| Per-axis baselines | 4 × {fast EWMA, slow EWMA, drift accumulator, clip-pressure EWMA} | Baseline tracking (§6)                                     |
| Rank $k$           | integer in $[1, \text{cap}]$                                      | Current active dimensionality                              |
| Step counter       | integer                                                           | Batches processed since creation                           |

**Initial state.** A newly created tracker begins with:

- $k = 1$. The core loop's algebra presupposes $k \geq 1$: at $k = 0$, Phase 1 produces a $b \times 0$ projection, all within-subspace scores are zero or undefined, and Phase 5's cumulative energy fractions are ill-defined. No formulas in §4.2 or §5 define behaviour at $k = 0$.
- $U$: any set of orthonormal columns in $\mathbb{R}^{w \times \text{cap}}$. The identity submatrix (column $j$ is the $j$-th standard basis vector) is a convenient deterministic choice. The initial basis is overwritten by the first Phase 2 SVD; the choice does not affect post-warm-up behaviour.
- $\sigma_{1:\text{cap}}$: initial values satisfying $\sum \sigma_j^2 \leq \delta_{\text{init}} \cdot bw/4$ for some small $\delta_{\text{init}}$ (e.g. $10^{-4}$; not the global stability constant $\varepsilon$). At $\delta_{\text{init}} = 0$, the first Phase 2 seeds the subspace purely from the first batch; at $\delta_{\text{init}} > 0$, a faint ghost of the initial basis persists for one step and is overwritten on the second. The value $0.01$ per component is conformant at per-cell analysis widths (where $bw \gg \text{cap}$). At coordination trackers ($w = 4$, $b = m \geq 2$, $\text{cap} = 4$) the effective $\delta_{\text{init}}$ rises to $\text{cap} \times 10^{-4} / (bw/4) = 2 \times 10^{-4}$ at $b = 2$ — still negligible (the initial energy is 0.02% of one batch), and the ghost is overwritten on the first Phase 2 step during noise warm-up (§11.7) long before live traffic arrives.
- $\mu^{(z)} = \mathbf{0}$, $\nu^{(z)} = \mathbf{1}$, $\Gamma = \mathbf{0}$: pre-allocated at capacity, with values chosen for safe behaviour on rank increase (Phase 3 below) and benign first-batch direct seeding (§11.2). The $\nu^{(z)} = 1.0$ pre-allocation is a conservative overestimate (roughly $4\times$ the null-hypothesis value of $0.25$ for centred $\pm 0.5$ bit vectors). It serves as the surprise denominator during the first Phase 1 scoring pass, before Phase 3 seeds $\nu^{(z)}$ from data. At $b = 1$, the seed is $z_{1j}^2 \approx 0.25$ in expectation — correctly scaled without special-case branching.
- Step counter $= 0$. This triggers the first-batch direct seeding path (Phase 3 below, $t = 0$), which overwrites $\mu^{(z)}$, $\nu^{(z)}$, and $\Gamma$ from data. The pre-seeding values above are therefore transient — they affect at most one scoring pass before being replaced.
- Baselines uninitialised. Noise influence $\eta = 1.0$.

> _Design note (rank convergence coupling)._ During noise injection (§11.1), rank adaptation fires every $T_{\text{rank}}$ batches and moves rank by at most 1. The rank at the end of noise injection is $k_{\text{post-noise}} = \min(1 + \lfloor\text{noise rounds} / T_{\text{rank}}\rfloor, \; \text{cap})$. At $T_{\text{rank}} = 100$ with 450 noise rounds at root, the tracker enters real-traffic service at $k \approx 5$–$6$, substantially below capacity. This is by design: the noise schedule is calibrated for baseline convergence (§11.9), not rank convergence; rank continues climbing under real traffic. The coherence axis (requiring $k \geq 2$) activates after $T_{\text{rank}}$ batches, contributing to the coherence convergence bottleneck documented in §11.9.

### 4.2 The Core Loop · `sec:sentinel:algorithm-core-loop`

Each tracker processes a batch $X \in \mathbb{R}^{b \times w}$ in five strictly ordered phases. **Scoring precedes evolution** — the batch is measured against the _prior_ model, then the model updates.

#### Phase 1 — Score · `sec:sentinel:algorithm-core-loop-score`

$$Z = X \, U_k \qquad \hat{X} = Z \, U_k^\top \qquad R = X - \hat{X}$$

Compute four per-sample scores from $Z$, $\hat{X}$, and $R$ (§5). Assemble batch summary statistics.

#### Phase 2 — Evolve Subspace · `sec:sentinel:algorithm-core-loop-evolve-subspace`

Construct the combined matrix:

$$M = \begin{bmatrix} \sqrt{\lambda} \; U_k \, \operatorname{diag}(\sigma_{1:k}) & \Big| & X^\top \end{bmatrix} \in \mathbb{R}^{w \times (k + b)}$$

Compute the thin SVD $M = \tilde{U} \, \tilde{S} \, \tilde{V}^\top$. Retain the top $n = \min(\min(w, k+b), \; \text{cap})$ components:

$$U_{:, 1:n} \leftarrow \tilde{U}_{:, 1:n} \qquad \sigma_{1:n} \leftarrow \operatorname{diag}(\tilde{S})_{1:n} \qquad \sigma_{n+1:\text{cap}} \leftarrow 0$$

The zeroing of trailing entries is semantically compelled: $M$ has rank at most $n$, so components beyond this index carry no energy from either the attenuated history or the current batch.

$\tilde{V}$ is discarded.

**Decay profile.** Without reinforcement, $\sigma^{(t)} = \lambda^{t/2} \, \sigma^{(0)}$. Energy half-life: $t_{1/2} = \ln 2 / \ln(1/\lambda)$.

| $\lambda$ | Half-life (steps) | Character   |
| --------- | ----------------- | ----------- |
| 0.99      | $\approx 69$      | Long memory |
| 0.95      | $\approx 14$      | Medium      |
| 0.90      | $\approx 7$       | Short       |

**Numerical failure guard.** If the thin SVD fails to converge (numerically degenerate input), the basis $U$ and singular values $\sigma$ are left unchanged. The batch is scored (Phase 1 completed) but the model does not evolve. Implementations should record this event.

**Identical-observation guard.** If all $b$ rows of $X$ are identical, the combined matrix $M$ has rank at most $k + 1$. The SVD is well-conditioned in this case; no special handling is needed beyond the standard thin-SVD computation.

#### Phase 3 — Evolve Latent Distribution · `sec:sentinel:algorithm-core-loop-evolve-latent-distribution`

$Z$ is the projection matrix computed in Phase 1 (prior-basis projection). Phase 3 does **not** recompute $Z$ using the updated basis from Phase 2. The latent statistics therefore always describe the distribution under the basis that was used for scoring; the consequence is a one-step lag after basis rotations, whose transient effects are analysed in the design note below.

**Phase 3 internal order.** The update order is $\nu^{(z)}$, then $\mu^{(z)}$, then $\Gamma$. The variance update evaluates deviations against the pre-update mean, matching the principle that scoring (Phase 1) uses the prior model.

**First batch ($t = 0$).** On the tracker's very first batch, seed the latent statistics directly from data rather than blending with initial values. The seeding uses the pre-allocated $\mu^{(z)} = \mathbf{0}$ as the centring reference:

$$\nu^{(z)}_j \leftarrow \max\!\left(\frac{1}{b}\sum_{i=1}^{b} z_{ij}^2,\;\varepsilon\right)$$

$$\mu^{(z)}_j \leftarrow \bar{Z}_j$$

$$\Gamma_{jl} \leftarrow \frac{1}{b}\sum_{i=1}^{b} z_{ij}\,z_{il}$$

No batch-size-conditional branching is needed within the $t = 0$ seeding path. At $b = 1$, the variance seed is $z_{1j}^2$ — noisy but correctly scaled ($\mathbb{E}[z_j^2] = \sigma^2 \approx 0.25$ under centred binary inputs with approximately zero-mean latent projections). The EWMA smooths the noise within $O(t_{1/2})$ subsequent batches.

> _The rationale for direct seeding — and the nature of the data typically present at $t = 0$ during warm-up — is analysed in §11.2._

**Subsequent batches ($t > 0$).** Compute the variance update first, using the pre-update mean:

$$\nu^{(z)}_j \leftarrow \lambda\,\nu^{(z)}_j + \alpha \cdot \max\!\left(\frac{1}{b}\sum_{i=1}^{b}(z_{ij} - \mu^{(z)}_j)^2,\;\varepsilon\right) \qquad j = 1,\ldots,k$$

Then update the mean:

$$\mu^{(z)}_j \leftarrow \lambda\,\mu^{(z)}_j + \alpha\,\bar{Z}_j \qquad j = 1, \ldots, k$$

Then update the second-moment matrix:

$$\Gamma_{jl} \leftarrow \lambda\,\Gamma_{jl} + \alpha\,\frac{1}{b} \sum_{i=1}^{b} z_{ij}\,z_{il} \qquad j < l$$

Only the upper triangle is stored.

**Runtime floor.** After each update (including $t = 0$ seeding), clamp: $\nu^{(z)}_j \leftarrow \max(\nu^{(z)}_j, 10^{-2})$. This floor is $25\times$ below the null-hypothesis value and should never bind during correct operation. It exists as defence-in-depth against implementation defects (e.g., a tracker created without warm-up) or degenerate identical-observation streams where all $z_{ij}$ are identical across consecutive batches. When it does bind, the maximum per-dimension surprise contribution is $(z_j - \mu_j)^2 / 10^{-2} \leq 100$ — large but not the catastrophic $10^5$ that $\varepsilon$-floored variance produces.

The inner $\max(\cdot, \varepsilon)$ and the runtime $\max(\cdot, 10^{-2})$ are complementary: the inner floor prevents a zero-energy EWMA _input_ (from identical observations within a batch); the runtime floor prevents the _accumulated EWMA value_ from being too small due to a prolonged sequence of near-$\varepsilon$ inputs. Neither is redundant.

**Behaviour on rank change.** All three latent statistics are pre-allocated at capacity $\text{cap}$ (§4.3). The EWMA loops above iterate over $j = 1, \ldots, k$, so entries beyond the active rank are never written.

When rank increases from $k$ to $k + 1$:

- $\mu^{(z)}_{k+1}$: retains its pre-allocated value of **zero**. Zero is the expected latent mean for a new basis direction over centred data; the EWMA converges to the true mean within $O(t_{1/2})$ steps.
- $\nu^{(z)}_{k+1}$: retains its pre-allocated value of **$1.0$**. This is a conservative overestimate: with typical steady-state variance $\approx 0.25$ for centred $\pm 0.5$ bit vectors, the surprise denominator is $4\times$ too large, **dampening** the new dimension rather than spiking it. Under the EWMA-mean-centred formula, the overestimate self-corrects as new batches contribute $(z_{i,k+1} - \mu^{(z)}_{k+1})^2$ terms, converging with half-life $t_{1/2}$ regardless of batch size.
- $\Gamma_{j,k+1}$ and $\Gamma_{k+1,l}$: new entries are already **zero** from construction.

When rank decreases, outer entries of all three statistics ($\mu^{(z)}$, $\nu^{(z)}$, $\Gamma$) are ignored but preserved. If rank later increases back, the preserved values provide a warm starting point rather than the cold defaults, reducing reconvergence time.

> _Design note (cold-start analogy)._ The $\nu^{(z)} = 1.0$ initialisation on rank increase is the same class of mismatch that §11.2 eliminates at $t = 0$ via direct seeding. On rank increase the transient is more benign: it affects only one dimension among $k$ (diluted by $1/k$ in the surprise average) and is always in the safe direction (dampening, not spiking). Per-dimension cold-seeding on rank change — analogous to the $t = 0$ logic — would eliminate this transient entirely, at the cost of per-dimension initialisation tracking.

> _Design note (why EWMA-mean-centred variance)._ The variance update uses deviations from $\mu^{(z)}_j$ rather than deviations from the batch mean $\bar{Z}_j$. This makes $\nu^{(z)}_j$ a direct estimator of the surprise numerator's expected value, yielding a self-consistent ratio at every batch size.
>
> The within-batch population variance $\operatorname{Var}(Z_{:,j})$ has a systematic negative bias of $\frac{b-1}{b}$ that is catastrophic at $b = 1$ (erosion to $\varepsilon$, surprise inflation to $O(10^5)$), severe at $b = 2$ ($-50\%$), and materially significant below $b \approx 16$. The EWMA-mean-centred formula eliminates this entire bias class through exact cancellation: the within-batch component contributes $\frac{b-1}{b}\sigma^2$ and the batch-mean-vs-EWMA-mean component contributes $\frac{\sigma^2}{b}$, summing to $\sigma^2$ regardless of $b$. The residual bias is $\operatorname{Var}(\mu^{(z)}_j) \approx \frac{\alpha}{b(1+\lambda)}\sigma^2$ — positive (safe direction: dampens surprise), small, and decreasing as $1/b$ with worst case $+0.5\%$ at $b = 1$ and $\lambda = 0.99$. Both the surprise numerator and the $\nu$ input share the same $\mu^{(z)}_j$ reference with identical bias, so the expected surprise ratio is $1 + O(\alpha^2)$ regardless of $b$ — the self-consistency property.
>
> When per-tracker batch size $b$ varies across ingestion cycles (as it does in practice, depending on the spatial distribution of observations), self-consistency is preserved: each batch's $\nu$ input uses the same $\mu^{(z)}$ reference as that batch's surprise computation.
>
> The formula couples $\nu^{(z)}$ to $\mu^{(z)}$, providing automatic gain control during regime transitions: when the mean estimate is stale, the variance estimate inflates proportionally, dampening transient surprise spikes. This coupling reduces surprise-axis sensitivity to gradual mean drift — $\nu$ absorbs the same signal that inflates the numerator and the ratio converges to $\approx 1$. This is an acceptable trade-off: displacement is the primary mean-shift detector, and surprise's primary role is detecting per-dimension distributional shape anomalies, which are preserved.
>
> The original formula's within-batch variance was invariant to Phase 2 basis rotations (re-centring on $\bar{Z}_j$ subtracted out any mean shift caused by the rotated projection). The EWMA-mean-centred formula is not: after a basis rotation, $z_{ij}$ are projections onto the new basis while $\mu^{(z)}_j$ reflects the old basis, inflating $(z_{ij} - \mu^{(z)}_j)^2$ and therefore $\nu^{(z)}_j$. This sensitivity is in the safe direction (dampening surprise through $\nu$ inflation), is transient ($O(t_{1/2})$ batches), and is precisely the automatic gain control described above.

> _Design note (raw second moments vs. centred covariances in $\Gamma$)._ $\Gamma$ tracks raw second moments $\mathbb{E}[z_j z_l]$, not centred covariances $\operatorname{Cov}(z_j, z_l)$. The coherence score (§5.5) compares against $\Gamma_{jl}$ directly. Since $\mathbb{E}[z_j z_l] = \operatorname{Cov}(z_j, z_l) + \mu_j^{(z)} \mu_l^{(z)}$, a shift in the latent mean changes $\Gamma_{jl}$ even when the correlation structure is perfectly stable. The alternative — using centred products $(z_j - \mu_j^{(z)})(z_l - \mu_l^{(z)})$ in both the score and the $\Gamma$ update — would isolate coherence to detect only correlation structure changes. This section analyses the trade-off and explains why raw second moments are retained.
>
> **Steady-state behaviour: the coupling is invisible.** At steady state, the $\mu_j \mu_l$ contribution to $\Gamma_{jl}$ is constant and perfectly absorbed by the coherence fast baseline $\bar{s}_{\text{coh}}$. The host-facing z-scores are computed against this baseline, so the permanent coupling between mean-structure and coherence-structure produces no observable effect. The coupling manifests _only_ during transitions — which is exactly the regime where the two formulations differ.
>
> **Transient behaviour: both formulations have artefacts.** During a regime transition that shifts the latent mean, the raw formulation produces a coherence spike because $z_j z_l$ reflects the new second moment while $\Gamma_{jl}$ still tracks the old one. Surprise fires simultaneously (detecting the same mean shift through $(z_j - \mu_j)^2 / \nu_j$), creating partial redundancy. However, the score $z_j z_l$ is computed from _fresh data with no lag_ — only the baseline $\Gamma_{jl}$ is stale. The departure is therefore clean: it reflects the genuine second-moment change (including the mean contribution), and it decays monotonically as $\Gamma$ absorbs the shift at rate $\lambda$.
>
> The centred formulation has a qualitatively worse transient. Staleness enters the _score computation itself_: the centred product $(z_j - \mu_j^{(\text{old})})(z_l - \mu_l^{(\text{old})})$ injects a ghost correlation $(\mu_j^{(\text{new})} - \mu_j^{(\text{old})})(\mu_l^{(\text{new})} - \mu_l^{(\text{old})})$ with a specific directional pattern in the $k \times k$ upper triangle determined by which dimensions shifted most. This ghost correlation is indistinguishable, from the baseline's perspective, from a genuine correlation structure change. The baseline _learns_ the ghost as if it were real, then must _unlearn_ it as $\mu^{(z)}$ converges — creating a **non-monotonic** transient. The raw formulation's monotonic, cleanly interpretable transient is a strictly better property, even though both have $O(t_{1/2})$ timescale.
>
> **Magnitude bound under centred binary inputs.** The input encoding (§2.3) is $\{-0.5, +0.5\}^w$ with $\mathbb{E}[x_i] = 0$ under uniform bits. Latent means $\mu_j^{(z)}$ are projections onto learned basis directions — they are tightly bounded and typically small. The cross-product $\mu_j \mu_l$ is therefore small relative to $\operatorname{Cov}(z_j, z_l)$ at steady state. A distributional shift large enough to make $\mu_j \mu_l$ dominate $\Gamma_{jl}$ is a violent regime change that should maximally alarm on every available axis.
>
> **State coupling.** With raw second moments, $\Gamma$ is a self-contained EWMA depending only on its own history and the observed products $z_j z_l$. With centred products, both the score and the baseline update depend on $\mu^{(z)}$, creating a hidden coupling between the surprise axis (which uses $\mu^{(z)}$ for scoring) and the coherence axis (which would use it for both scoring and baseline computation). A convergence artefact in the mean estimate would simultaneously corrupt two axes instead of one. The raw formulation keeps the axes' state dependencies more separated.
>
> **Coordination-layer exposure.** At the coordination layer (§7), the §7.3 running-mean centring subtracts $\mu^{(\text{in})}_g$ from the raw score matrix before the coordination tracker sees it. During a transition, $\mu^{(\text{in})}_g$ lags the actual group mean, so the tracker's inputs carry residual mean that the centring didn't remove. The coordination tracker's internal $\mu^{(z)}$ then tracks this residual, creating a second lag. The $\mu_j \mu_l$ contamination of coordination-level $\Gamma$ therefore reflects the square of a lagged mean estimate, which can amplify the artefact's persistence relative to the per-cell level. Under gradual drift (the regime where the concern is most relevant), the running-mean centring tracks the linearly increasing group mean with a steady-state lag of $\delta\mu / \alpha$, where $\delta\mu$ is the per-batch mean increment (the standard lag of an EWMA tracking a linear trend). The tracker's inputs therefore carry a constant residual bias of $O(\delta\mu / \alpha)$, and the squared cross-product contamination is $O((\delta\mu / \alpha)^2)$. However, this constant bias is itself absorbed by the coordination tracker's internal statistics — the tracker's own $\mu^{(z)}$ and $\Gamma$ converge to include the bias — so the contamination manifests only during the transient before the tracker reaches its own steady state. Under a sudden shift, the full residual $\Delta\mu$ is visible on the first batch and decays as $\lambda^t \Delta\mu$ — but during this transient, surprise fires simultaneously at the per-cell level, making the coherence artefact redundant. In both regimes, the concern is real but quantitatively minor: the contamination is transient rather than small in absolute terms, and it overlaps temporally with surprise signals that already capture the same event.
>
> **Verdict.** The raw formulation trades a small, well-bounded artefact (mean-squared leakage into a second-moment tracker, monotonic decay, invisible at steady state) for the centred formulation's subtler, harder-to-bound artefact (stale-mean ghost correlations, non-monotonic transient, hidden state coupling between axes). The raw formulation is retained.

> _Design note (subspace energy weighting vs. latent EWMA weighting)._ Phase 2 and Phase 3 apply different effective weightings to new data, and this is a deliberate design choice whose consequences bear understanding.
>
> **The apparent asymmetry.** In the combined matrix $M = [\sqrt{\lambda}\, U_k \operatorname{diag}(\sigma) \mid X^\top]$, new observations enter at full energy ($\|x_i\|^2 = w/4$ each), while old structure is attenuated by $\sqrt{\lambda}$. In Phase 3, the standard EWMA $\mu \leftarrow \lambda\mu + \alpha\bar{Z}$ weights new data at $\alpha = 1 - \lambda$. At first glance, the subspace gives the current batch weight $\sim 1$ while the latent statistics give it weight $\alpha$ — a factor-of-$1/\alpha$ mismatch.
>
> **At energy steady state, there is no mismatch.** The SVD operates on the Frobenius norm of $M$. In the steady state where total singular-value energy $\Sigma = \sum \sigma_j^2$ has stabilised, $\Sigma = bw/(4\alpha)$. The new batch's fraction of total energy in $M$ is $(bw/4)/\Sigma = \alpha$ — exactly the EWMA learning rate. Both mechanisms forget old information at rate $\lambda^t$ per step and weight new information at effective fraction $\alpha$. The energy half-lives are identical: $t_{1/2} = \ln 2 / \ln(1/\lambda)$.
>
> **But the SVD has a rotational degree of freedom that the EWMA lacks.** The real asymmetry is not in effective sample size but in the _nature_ of the response to structural change. The EWMA is a linear filter: after a regime change, $\mu^{(z)}$ converges to the new mean exponentially with time constant $t_{1/2}$, regardless of how different the new regime is. The SVD is a nonlinear rank-ordered decomposition: it can _swap_ basis directions in a single step once a new direction's singular value exceeds a decaying old direction's. This discrete jump has no analog in the continuous EWMA.
>
> **The transient surprise spike after regime changes.** After a sudden distributional shift, the subspace may rotate substantially within a few batches — new directions appear in $U$ once their energy dominates decaying old singular values. But $\mu^{(z)}$ and $\nu^{(z)}$ carry state from the old basis. After a direction swap, $\mu^{(z)}_j$ was tracking the mean projection onto old direction $u_j^{(\text{old})}$; it is now being updated with projections onto the new direction $u_j^{(\text{new})}$. The stale reference produces a transient spike in the surprise score $(z_j - \mu_j)^2 / \nu_j$ that persists for $O(t_{1/2})$ batches while the EWMA converges.
>
> **This is the correct behaviour, for three reasons.**
>
> First, the spike is _informative_. It correctly signals "observations that don't match the historical within-subspace distribution" — which is literally true during a regime change. A system that silently absorbed regime changes would fail at the core detection task. The drift accumulator (§6.3) distinguishes transient spikes (which don't accumulate much) from sustained shifts (which do).
>
> Second, the subspace _must_ adapt its directions faster than the EWMA adapts its scalars. An incorrect basis direction wastes an entire detection axis — every projection onto a stale direction is uninformative. An incorrect latent mean is a quantitative error within a still-meaningful projection. The SVD's ability to rapidly discover new structure is why it is used instead of a linear update for the subspace.
>
> Third, matching the rates would be strictly worse. Scaling new data by $\alpha$ in $M$ (giving $M = [\sqrt{\lambda}\, U_k \operatorname{diag}(\sigma) \mid \sqrt{\alpha}\, X^\top]$) would require $\sim 1/\alpha$ batches of coherent new-direction data before the SVD even recognised its existence — eliminating the system's ability to detect emerging structure promptly.
>
> **Consequences for the host.** During regime transitions, expect elevated surprise z-scores for $O(t_{1/2})$ batches as the latent statistics reconverge. The fast EWMA baseline (§6.1) absorbs this transient — the z-scores are computed against the fast baseline, which itself adapts at rate $\lambda$. The drift accumulator may register modest growth during the transient, bounded by $\sim t_{1/2} \times (\text{spike magnitude} - \kappa)$; the slow baseline (§6.2) eventually absorbs the shift and the accumulator returns to zero. Hosts that expect frequent regime changes (e.g., diurnal patterns) should interpret surprise drift accumulation in light of the known transition schedule.
>
> Note: under the EWMA-mean-centred variance formula (Phase 3 above), the surprise spike's persistence is shorter than described above. After the initial batch fires at full strength (scored against the prior $\nu$ in Phase 1), subsequent batches' $\nu$ co-adapts — inflating as the stale $\mu^{(z)}$ inflates the $(z - \mu)^2$ terms — progressively dampening the surprise ratio. Displacement inherits the primary detection role during transitions. The first-batch detection at full strength is preserved; the $O(t_{1/2})$ sustained elevation is not.
>
> _The asymmetry is not between two forgetting rates but between a linear filter (EWMA) that can only exponentially forget and a nonlinear decomposition (SVD) that can structurally reorganise. The latent statistics inherit the SVD's directional decisions but adapt their scalar parameters at the EWMA rate — a deliberate separation of concerns between "which directions matter" (fast, nonlinear) and "what the typical projection onto those directions looks like" (smooth, linear)._

#### Phase 4 — Update Baselines and Drift Detector · `sec:sentinel:algorithm-core-loop-update-baselines-and-drift-detector`

Each scoring axis maintains a fast EWMA, slow EWMA, drift accumulator, and clip-pressure EWMA (§6). **Exception:** coherence baselines are not updated while $k < 2$ (§5.5).

#### Phase 5 — Adapt Rank · `sec:sentinel:algorithm-core-loop-adapt-rank`

Rank is recomputed every $T_{\text{rank}}$ steps (the rank update interval):

1. Compute cumulative energy fractions:

$$c_i = \frac{\sum_{j=1}^{i} \sigma_j^2}{\sum_{j=1}^{\text{cap}} \sigma_j^2 + \varepsilon} \qquad i = 1, \ldots, \text{cap}$$

2. Find the target rank:

$$k^* = \min\!\big\{i : c_i \geq \tau\big\} + 1 \qquad \text{clamped to } [1, \text{cap}]$$

If the set $\{i : c_i \geq \tau\}$ is empty, then $k^* = \text{cap}$. This requires total energy $\sum \sigma_j^2 < \varepsilon \tau / (1 - \tau)$ — with the default $\varepsilon = 10^{-6}$ and $\tau = 0.9$, total energy below $9 \times 10^{-6}$. Since each Phase 2 evolve step injects fresh batch energy into the SVD, this threshold is not reachable during normal operation; the fallback exists as a safety net against numerical edge cases or future algorithm changes. (Approximately equal nonzero singular values produce a _non-empty_ set — the threshold is met at high index, yielding $k^* \approx \text{cap}$ via the normal path; see §A.5.)

> _Design note (the $+1$ buffer dimension)._ The $+1$ ensures the active subspace always extends one dimension beyond the energy threshold. This serves three purposes.
>
> First, it **enables coherence** (§5.5). Coherence requires $k \geq 2$, so without the buffer a tracker whose energy is dominated by a single direction would be permanently stuck at $k = 1$ with no off-diagonal detection capability. The $+1$ guarantees that a single dominant component yields $k^* = 2$, not $k^* = 1$.
>
> Second, it provides **early visibility into emerging structure**. The dimension just past the energy threshold is the first place new non-noise structure will appear as the data distribution evolves. By including it in the active subspace, the system can detect that emergence through the within-subspace axes (displacement, surprise, coherence) rather than relying solely on the coarser novelty axis, which only measures aggregate residual energy.
>
> Third, it acts as a **stability margin** against threshold boundary oscillation. A dimension whose cumulative energy fraction fluctuates near $\tau$ would cause rank to toggle on every adaptation step without the buffer; the extra dimension absorbs this boundary noise.

3. Move rank by at most one step:

$$k \leftarrow k + \operatorname{clamp}(k^* - k, \; -1, \; +1)$$

**On rank drop from $\geq 2$ to $1$:** destroy coherence baselines — return the fast EWMA, slow EWMA, drift accumulator, **and clip-pressure EWMA** for the coherence axis to their uninitialised state ($\bar{\rho} = 0$). This prevents stale state from a previous $k \geq 2$ epoch from contaminating a future one. Note: under the current rank formula, $k^* \geq 2$ whenever $\text{cap} \geq 2$ (the $+1$ buffer ensures this — see design note above), so this clause cannot fire during normal operation for any tracker with $\text{cap} \geq 2$. It exists as a safety net against implementation defects or future algorithm changes that might introduce a pathway to $k = 1$ from above.

### 4.3 Memory per Tracker · `sec:sentinel:algorithm-tracker-memory`

At width $w$ with capacity $\text{cap}$:

| Component                               | Size (elements)              |
| --------------------------------------- | ---------------------------- |
| $U$                                     | $w \times \text{cap}$        |
| $\sigma$                                | $\text{cap}$                 |
| $\mu^{(z)}, \nu^{(z)}$                  | $2 \times \text{cap}$        |
| $\Gamma$ (upper triangle)               | $\text{cap}(\text{cap}-1)/2$ |
| Baselines (§6)                          | $4 \times 8 = 32$            |
| Scalar state (rank, step counter, etc.) | $\sim 10$                    |

At $w = 96$, $\text{cap} = 16$: approximately 1,750 floating-point elements. Total analysis memory is bounded by $|\mathcal{A}^*| \times (\text{max per-tracker size})$; see §8 for the bound on $|\mathcal{A}^*|$ and §12 for global resource accounting.

---

## Chapter 5. Scoring · `sec:sentinel:algorithm-scoring`

### 5.1 Polarity Invariant · `sec:sentinel:algorithm-scoring-polarity-invariant`

All four scoring axes share a **polarity invariant**: higher values indicate greater anomalous departure from baseline. A score of zero (or near zero) is maximally normal; positive excursions indicate increasing anomaly.

This invariant is essential for the baseline tracking mechanism (§6): the upper-tail outlier filter prevents sustained high scores from poisoning baselines, which requires that anomalous behaviour consistently produces _high_ scores, never low.

### 5.2 Novelty · `sec:sentinel:algorithm-scoring-novelty`

$$\text{novelty}_i = \frac{\|\mathbf{x}_i - \hat{\mathbf{x}}_i\|^2}{w - k}$$

Average residual energy per orthogonal degree of freedom. **Measures:** unexplained structure — energy outside the learned subspace. **Range:** $[0, \infty)$. **Requires:** $k \geq 1$.

**Degeneracy at $k = w$.** When $\text{cap} = w$ (which requires $r_{\max} \geq w$), rank adaptation (§4.2, Phase 5) can push $k$ to $w$. The residual $R_i = 0$ identically and the denominator $w - k = 0$, giving the indeterminate form $0/0$. Implementations define $\text{novelty}_i = 0$ in this case (clamping the denominator to 1). The axis carries no information — detection relies entirely on the three within-subspace axes. This condition is reported as _novelty-saturated_ (§14.7) and arises naturally at coordination trackers ($w = 4$) and deep cells with generous $r_{\max}$.

### 5.3 Displacement · `sec:sentinel:algorithm-scoring-displacement`

$$\text{displacement}_i = \frac{\|\mathbf{z}_i\|^2}{k + \|\mathbf{z}_i\|^2}$$

Bounded distance from the subspace origin. **Measures:** total within-subspace energy — how far the observation sits from the learned centre. **Range:** $[0, 1)$. **Requires:** $k \geq 1$.

> _Design note (polarity)._ The natural measure of proximity is $q_i = k / (k + \|\mathbf{z}_i\|^2)$, where anomalies push $q$ downward. This violates the polarity invariant: anomalous values (low $q$) would pass the upper-tail filter (§6.2) and gradually drag the baseline downward, making the departure invisible. The complement $1 - q_i$ restores correct polarity.

### 5.4 Surprise · `sec:sentinel:algorithm-scoring-surprise`

$$\text{surprise}_i = \frac{1}{k} \sum_{j=1}^{k} \frac{(z_{ij} - \mu^{(z)}_j)^2}{\nu^{(z)}_j + \varepsilon}$$

Average diagonal Mahalanobis deviation. **Measures:** per-dimension magnitude deviation from learned latent means. **Range:** $[0, \infty)$. **Requires:** $k \geq 1$.

Complementary to displacement: an observation can have normal total energy but unusual distribution across dimensions.

### 5.5 Coherence · `sec:sentinel:algorithm-scoring-coherence`

$$\text{coherence}_i = \frac{2}{k(k-1)} \sum_{j < l} \big(z_{ij} \, z_{il} - \Gamma_{jl}\big)^2$$

Average squared deviation of pairwise latent products from their learned second moments. **Measures:** off-diagonal covariance deviation — unusual combinations of co-activation. **Range:** $[0, \infty)$. **Requires:** $k \geq 2$.

**Defined as $0$ when $k < 2$.** Coherence baselines are not updated while $k < 2$. When rank first reaches 2, baselines begin tracking from the first real coherence values. On rank drop back to 1, baselines are destroyed (§4.2, Phase 5).

### 5.6 Summary · `sec:sentinel:algorithm-scoring-summary`

| Axis            | Score        | Range         | Measures                | Requires   |
| --------------- | ------------ | ------------- | ----------------------- | ---------- |
| Subspace        | Novelty      | $[0, \infty)$ | Unexplained structure   | $k \geq 1$ |
| Within-subspace | Displacement | $[0, 1)$      | Distance from centroid  | $k \geq 1$ |
| Within-subspace | Surprise     | $[0, \infty)$ | Per-dimension magnitude | $k \geq 1$ |
| Within-subspace | Coherence    | $[0, \infty)$ | Pairwise co-activation  | $k \geq 2$ |

### 5.7 Geometric Picture · `sec:sentinel:algorithm-scoring-geometric-picture`

```
Full observation space ℝ^w
┌───────────────────────────────────────────┐
│                                           │
│  Learned subspace ℝ^k                     │
│  ┌──────────────────┐                     │
│  │  μ^(z) centroid   │  Residual: x − x̂   │
│  │    ·              │ ◄─── novelty ───►  │
│  │   /|              │                     │
│  │  z |              │                     │
│  │  ├─┤ displacement │                     │
│  │  ├─┤ surprise     │                     │
│  │  z₁·z₂ coherence │                     │
│  └──────────────────┘                     │
└───────────────────────────────────────────┘
```

### 5.8 Why Four Axes · `sec:sentinel:algorithm-scoring-four-axes`

The three within-subspace scores decompose the latent activation pattern along orthogonal statistical concerns:

| Score        | Input                                   | Covariance structure  |
| ------------ | --------------------------------------- | --------------------- |
| Displacement | $\|\mathbf{z}\|^2$                      | Total energy (scalar) |
| Surprise     | $(z_j - \mu_j)^2/\nu_j$ per $j$         | Diagonal              |
| Coherence    | $(z_j z_l - \Gamma_{jl})^2$ per $j < l$ | Off-diagonal          |

Together they cover the full covariance structure without assembling or inverting a dense $k \times k$ matrix.

A fifth axis — projection energy $\|\hat{\mathbf{x}}_i\|^2/k$ — is omitted because the constant-norm property (§2.5) makes it a perfect affine function of novelty. It carries zero independent information and violates the polarity invariant.

---

## Chapter 6. Baseline Tracking and Drift Detection · `sec:sentinel:algorithm-baseline-tracking-and-drift-detection`

Each scoring axis maintains four components: a fast EWMA for instantaneous z-scores, a slow EWMA as a long-memory reference, a one-sided drift accumulator for detecting gradual shifts, and a clip-pressure EWMA $\bar{\rho}$ that tracks the fraction of samples clipped per axis (§6.4). Together, these provide normalised scoring (z-scores), temporal persistence (drift detection), and adaptive clip-width modulation on top of the raw scores from §5.

### 6.1 Fast EWMA · `sec:sentinel:algorithm-fast-ewma`

The fast EWMA tracks running mean $\bar{s}$ and variance $\bar{v}$ at decay $\lambda$.

#### 6.1.1 Update Rule · `sec:sentinel:algorithm-fast-ewma-update-rule`

**Clip-pressure state.** Each axis maintains a clip-pressure EWMA $\bar{\rho}$, initialized to $0$ at tracker creation. At warm-up completion (§11.4), $\bar{\rho}$ is reset to $0$ alongside the CUSUM reset and slow-from-fast seeding. On rank drop from $\geq 2$ to $1$, the coherence axis's $\bar{\rho}$ is destroyed alongside the coherence baselines (§4.2, Phase 5).

Given per-sample scores $\mathbf{s} = (s_1, \ldots, s_b)$ from one batch:

**1. Compute effective clip ceiling.** Let $p = \max(\eta, \bar{\rho})$ where $\eta$ is the noise influence (§11.5) and $\bar{\rho}$ is the clip-pressure EWMA for this axis. Compute:

$$n_\sigma^{\text{eff}} = n_\sigma\left(1 + \frac{p}{1 - p + \varepsilon}\right)$$

**When the baseline is uninitialised** (no valid $\bar{s}$ or $\bar{v}$), skip clipping entirely — placeholder values are not a meaningful reference. When clipping is skipped (uninitialised baseline), set $\rho_t = 0$ and update $\bar{\rho}$ normally. This causes $\bar{\rho}$ to decay toward zero during the uninitialised phase, ensuring no residual pressure when the baseline initialises.

**When the baseline is initialised**, compute the clip ceiling:

$$c_{\text{clip}} = \bar{s} + n_\sigma^{\text{eff}} \sqrt{\bar{v}}$$

Retain samples with $s_i < c_{\text{clip}}$. Compute the batch clip ratio $\rho_t = n_{\text{clipped}} / b$.

**If all samples are rejected**, the batch is processed without clipping (degenerate lockout guard — fires only during the first few batches of an extreme shift before $\bar{\rho}$ has risen sufficiently). Set $\rho_t = 1$.

Update the clip-pressure EWMA:

$$\bar{\rho} \leftarrow \lambda_\rho \, \bar{\rho} + (1 - \lambda_\rho) \, \rho_t$$

The filter is **upper-tail only** because all four scoring axes are non-negative, right-skewed, and satisfy the polarity invariant (§5.1): anomalous departure inflates scores, never deflates. The clip ceiling prevents sustained high scores from poisoning the baseline upward.

**2. Fast EWMA update.** Let $\bar{s}_{\text{batch}}$ be the mean of **retained** samples (or all samples if the degenerate guard fired).

- _Uninitialised → initialised:_ $\bar{s} \leftarrow \bar{s}_{\text{batch}}$
- _Subsequent:_ $\bar{s} \leftarrow \lambda \, \bar{s} + \alpha \, \bar{s}_{\text{batch}}$

**3. Variance update.** Let $\bar{v}_{\text{batch}}$ be the population variance of **retained** samples (requires $\geq 2$ retained; otherwise **freeze**: leave $\bar{v}$ unchanged — see design note below).

- _Uninitialised → initialised:_ $\bar{v} \leftarrow \max(\bar{v}_{\text{batch}}, \; 10^{-4})$
- _Subsequent:_ $\bar{v} \leftarrow \lambda \, \bar{v} + \alpha \, \max(\bar{v}_{\text{batch}}, \; 10^{-4})$

The $10^{-4}$ floor prevents degenerate zero-variance baselines.

> _Design note (variance freeze on < 2 retained)._ When clipping retains fewer than 2 samples, $\bar{v}$ is neither updated nor decayed — it holds its last successfully computed value. Three alternatives were considered:
>
> 1. **Pure decay** ($\bar{v} \leftarrow \lambda\,\bar{v}$). This creates a positive feedback loop: shrinking variance → tighter clip ceiling → more clipping → fewer retained → more decay. The loop is self-reinforcing and can collapse variance to near-zero, worsening the very lockout condition that caused the skip.
> 2. **Decay with floor injection** ($\bar{v} \leftarrow \lambda\,\bar{v} + \alpha \cdot 10^{-4}$). The steady-state variance converges to $10^{-4}$ regardless of $\lambda$, which is 1–3 orders of magnitude below realistic axis variances ($\sim 0.001$–$0.1$). The collapse from $\bar{v} = 0.01$ to $\bar{v} = 0.0001$ inflates z-scores by $\sim$10× and tightens the clip ceiling by nearly the same factor — a milder but structurally similar failure to pure decay. Making the injection proportional to the current $\bar{v}$ approximates a freeze with extra arithmetic and a slow downward bias.
> 3. **Single-sample deviation proxy** ($\bar{v} \leftarrow \lambda\,\bar{v} + \alpha \cdot \max\!\big((s_{\text{retained}} - \bar{s})^2,\; 10^{-4}\big)$). This tracks the right order of magnitude during regime transitions. However, the retained sample passed the clip filter and is bounded by $c_{\text{clip}} - \bar{s} = n_\sigma^{\text{eff}}\sqrt{\bar{v}}$, so the proxy is bounded by $(n_\sigma^{\text{eff}})^2 \bar{v}$. During early lockout (before clip-pressure has risen, $n_\sigma^{\text{eff}} \approx n_\sigma$), the retained sample is likely near $\bar{s}$, causing systematic variance underestimation — a mild version of option 1's failure mode in exactly the phase where it is least tolerable. This alternative merits evaluation if small-batch deployments reveal the freeze to be problematic; it is not adopted here.
>
> The freeze produces a stale-but-data-derived estimate rather than a convergence target that is aggressively wrong. Its failure mode is **directionally asymmetric**: if the old regime had high variance, the frozen $\bar{v}$ suppresses z-scores and reduces detection sensitivity in the new regime; if the old regime had low variance, the frozen $\bar{v}$ inflates z-scores and false alarms. The freeze is least-bad _on average_ across regime transitions precisely because you do not know which direction the regime shifted.
>
> Critically, the freeze window is **self-limiting** under the clip-pressure mechanism (§6.4): once $\bar{\rho}$ rises enough to widen the ceiling, more samples are retained and variance updates resume. The staleness duration is bounded by the clip-pressure recovery time ($\sim$20–40 batches at $\lambda_\rho = 0.95$), not by any property of the variance itself. During this window, detection sensitivity is calibrated to the _previous_ regime's spread. The host can observe this condition via the clip-pressure value ($\bar{\rho}$) in the per-axis report (§14.4).

**4. Slow EWMA update.** Using the same **retained** samples from step 1 (single shared clip filter — see design note below), update slow mean and slow variance at rate $\lambda_s$ following the same rules as steps 2–3.

**5. Drift accumulator update.** Using the **raw** batch mean $\bar{s}_{\text{raw}} = \frac{1}{b}\sum_{i=1}^b s_i$ (all $b$ samples, no clipping):

$$S_t = \max\!\Big(0, \; S_{t-1} + \big(\bar{s}_{\text{raw}} - \bar{s}_{\text{slow}}\big) - \kappa_\sigma \sqrt{\bar{v}_{\text{slow}}}\Big)$$

> _Design note (single shared clip filter)._ Both the fast and slow EWMAs receive samples from the same clip filter, computed against the fast EWMA's clip-pressure-adjusted ceiling. This is a deliberate departure from independent per-EWMA clipping. The slow EWMA's conservatism is provided by its longer time constant ($\lambda_s > \lambda$), not by independent input filtering. Independent slow-EWMA clipping would create a secondary lockout surface that the clip-pressure mechanism cannot reach. See §6.4 for the contamination analysis.

> _Design note (pre-clip drift accumulator input)._ The drift accumulator uses the raw batch mean because it is a detector, not an estimator. Its output $S$ flows outward to the report (§14.5) and is never fed back into any model. Clipping protects estimators from contamination; it makes detectors blind. The raw input ensures the accumulator registers the full departure magnitude during the first batches of a shift, before the clip-pressure mechanism has opened the ceiling.

#### 6.1.2 Z-Score Computation · `sec:sentinel:algorithm-fast-ewma-z-score-computation`

$$\zeta(s) = \frac{s - \bar{s}}{\sqrt{\bar{v}} + \varepsilon}$$

Two z-scores per axis per batch:

- $\zeta(\max_i s_i)$ — loudest alarm in the batch.
- $\zeta(\bar{s}_{\text{batch}})$ — sustained elevation of the batch.

### 6.2 Slow EWMA · `sec:sentinel:algorithm-slow-ewma`

Each axis maintains a second EWMA at decay $\lambda_s > \lambda$ (per-tracker) or $\lambda_{s,m} > \lambda$ (coordination). The slow EWMA provides the reference for the drift accumulator.

The constraint $\lambda_s > \lambda$ is structural — the slow baseline must have strictly longer memory than the fast one.

| Decay               | Half-life           | Role           |
| ------------------- | ------------------- | -------------- |
| $\lambda = 0.99$    | $\approx 69$ steps  | Fast baseline  |
| $\lambda_s = 0.999$ | $\approx 693$ steps | Slow reference |

**Update rule.** The slow EWMA receives the same retained samples as the fast EWMA (single shared clip filter) and applies the same EWMA update at rate $\lambda_s$. The complete per-batch procedure is specified in §6.1.1, step 4.

### 6.3 CUSUM Drift Accumulator · `sec:sentinel:algorithm-cusum-drift-accumulator`

One-sided Page's test detecting sustained upward drift of raw batch means from the slow baseline:

$$S_t = \max\!\Big(0,\; S_{t-1} + \big(\bar{s}_{\text{raw},t} - \bar{s}_{\text{slow},t}\big) - \kappa\Big)$$

where $\bar{s}_{\text{raw},t}$ is the **raw** (pre-clip) batch mean of all $b$ samples, and $\kappa = \kappa_\sigma \cdot \sqrt{\bar{v}_{\text{slow},t}}$ is the noise allowance. [This formula is repeated from §6.1.1, step 5 for self-contained reference.]

**Input signal.** The drift accumulator uses the **raw** (pre-clip) batch mean as its input. The complete per-batch procedure is specified in §6.1.1, step 5. The raw-input choice is analysed in §6.4.

Under normal conditions, fluctuations are absorbed by $\kappa$. Under a gradual shift, $\bar{s}_{\text{raw}}$ consistently exceeds $\bar{s}_{\text{slow}} + \kappa$, and the accumulator grows monotonically.

> _Design note (why dual-EWMA, not a frozen reference)._ A frozen checkpoint would require manual host resets after legitimate regime changes — operational burden and a policy decision. A slow EWMA adapts automatically, just slowly enough to catch shifts before absorption. After a legitimate change, the slow baseline catches up and the accumulator returns to zero without intervention.

Three reset mechanisms:

1. **Automatic.** The slow baseline eventually absorbs a legitimate regime change, driving $S \to 0$.
2. **Host-initiated.** The host inspects, decides the shift is legitimate, and zeroes $S$.
3. **Post-warm-up seeding.** After initial warm-up completes, the slow EWMA is seeded from the fast EWMA's converged values and $S$ is reset. The rationale and procedure are specified in §11.

### 6.4 Clip-Pressure Dynamics and Contamination Trade-offs · `sec:sentinel:algorithm-clip-pressure-dynamics-and-contamination-tradeoffs`

The clip-pressure mechanism (§6.1.1) introduces a controlled trade-off between baseline lockout recovery and contamination resistance. This section analyses the dynamics under three regimes and the feedback paths through which contamination propagates.

#### 6.4.1 Sustained Lockout Recovery · `sec:sentinel:algorithm-clip-pressure-sustained-lockout-recovery`

Under a regime shift that clips 100% of samples ($\rho_t = 1.0$ every batch), $\bar{\rho}$ rises monotonically and the ceiling opens progressively:

| Batches | $\bar{\rho}$ | $n_\sigma^{\text{eff}}$ (at $n_\sigma = 3$) | Status                     |
| ------: | -----------: | ------------------------------------------: | -------------------------- |
|       0 |         0.00 |                                         3.0 | Locked                     |
|       3 |         0.14 |                                         3.5 | Softening                  |
|       7 |         0.30 |                                         4.3 | Opening — some data passes |
|      14 |         0.51 |                                         6.1 | Ceiling doubled            |
|      20 |         0.64 |                                         8.3 | Wide — most data passes    |
|      30 |         0.79 |                                          14 | Effectively unclipped      |
|      40 |         0.87 |                                          23 | Baseline actively tracking |

The degenerate lockout guard (§6.1.1, step 1) fires during the first few batches before $\bar{\rho}$ has risen enough to widen the ceiling. Once $\bar{\rho} > 0.3$, the graduated ceiling handles recovery without the guard.

#### 6.4.2 Transient Spike Recovery · `sec:sentinel:algorithm-clip-pressure-transient-spike-recovery`

A single anomalous batch ($\rho_t = 1.0$ for one batch, then $\rho_t = 0$) produces minimal ceiling disturbance:

|     Batch | $\rho_t$ | $\bar{\rho}$ | $n_\sigma^{\text{eff}}$ |
| --------: | -------: | -----------: | ----------------------: |
| 1 (spike) |      1.0 |         0.05 |                    3.16 |
|         7 |        0 |         0.04 |                    3.12 |
|        15 |        0 |         0.02 |                    3.07 |

The peak ceiling widening (5%) is negligible. Self-correcting within ~15 batches.

#### 6.4.3 Oscillatory Convergence Under Moderate Shifts · `sec:sentinel:algorithm-clip-pressure-oscillatory-convergence-under-moderate-shifts`

When a regime shift clips a substantial fraction but not all samples (e.g., 50–80%), convergence is oscillatory rather than monotonic:

1. $\bar{\rho}$ rises → ceiling widens → some extreme samples pass through.
2. Baseline absorbs retained data, shifts toward new regime.
3. Fewer samples are clipped (less extreme relative to updated baseline) → $\bar{\rho}$ decays.
4. Ceiling tightens — but baseline hasn't fully converged.
5. Clipping resumes at a moderate rate → $\bar{\rho}$ rises again.
6. Cycle repeats with decreasing amplitude.

The oscillation is inherent when $\lambda_\rho < \lambda$: the ceiling responds faster than the baseline adapts. Each cycle moves the baseline closer to the new regime. The baseline converges monotonically through the oscillation — only the ceiling oscillates.

At $\lambda_\rho = 0.95$ and $\lambda = 0.99$, each ceiling-open cycle lasts approximately 14 batches (the clip-pressure half-life), during which the baseline absorbs $1 - \lambda^{14} \approx 13\%$ of the remaining gap. Typical moderate shifts (initial clip rate 40–70%) converge within 3–5 oscillatory cycles (~60–100 batches). Near-total lockout shifts (initial clip rate above 90%) take proportionally longer — roughly 8–12 cycles (~120–170 batches) — because the ceiling must open much wider before substantial data passes.

The damping condition for non-oscillatory convergence is $\lambda_\rho \geq \lambda$, but satisfying this at $\lambda = 0.99$ would require $\lambda_\rho \geq 0.99$ (half-life ~69 batches), making lockout recovery unacceptably slow. The oscillatory regime at $\lambda_\rho = 0.95$ is the correct trade-off.

The drift accumulator is unaffected by ceiling dynamics — it uses raw batch means (§6.1.1, step 5) and sees the full departure throughout the oscillation.

#### 6.4.4 Contamination Feedback Paths · `sec:sentinel:algorithm-clip-pressure-contamination-feedback-paths`

Under elevated clip pressure, the widened ceiling admits a larger fraction of extreme scores, which shift the baselines. The shifted baselines affect three downstream quantities:

1. **Z-score sensitivity.** A contaminated fast EWMA mean $\bar{s}$ closer to adversarial score values produces smaller z-scores for those observations. Detection sensitivity for adversarial traffic is reduced in proportion to the contamination.

2. **Clip ceiling elevation.** The ceiling $c_{\text{clip}} = \bar{s} + n_\sigma^{\text{eff}} \sqrt{\bar{v}}$ rises with $\bar{s}$ and $\bar{v}$, admitting slightly more extreme observations in subsequent batches. This is a mild positive feedback loop, bounded by the clip-pressure mechanism's self-correcting dynamics: as clipping decreases, $\bar{\rho}$ decays, tightening $n_\sigma^{\text{eff}}$ toward $n_\sigma$.

3. **Drift noise allowance.** The allowance $\kappa = \kappa_\sigma \sqrt{\bar{v}_{\text{slow}}}$ increases if the slow EWMA's variance is contaminated, reducing drift accumulator sensitivity. This is a second-order effect (variance contamination is slower than mean contamination) but reduces the system's ability to detect gradual shifts during sustained adversarial presence.

#### 6.4.5 Contamination vs. Bias · `sec:sentinel:algorithm-clip-pressure-contamination-vs-bias`

**Under hard clipping** with a sustained adversary controlling fraction $f$ of observations, the baseline tracks only the clean $(1-f)$ fraction — biased but uncontaminated. Z-scores for clean observations near the clipping boundary are systematically inflated, generating false positives for legitimate traffic. Adversarial observations are invisible to all baseline machinery including the drift accumulator.

**Under clip-pressure modulation**, the baseline tracks the actual observation mixture — slightly contaminated but unbiased. Z-scores for both clean and adversarial observations are measured against the true mixture distribution. The drift accumulator (using raw means) reports the full departure regardless.

For "Measure, don't decide" (§1.3), the unbiased estimate is more honest: the host receives baselines that reflect the actual observation distribution, including adversarial influence. A biased baseline misrepresents the distribution to the host. The host can make contamination judgments from the drift accumulator evidence and the per-axis clip-pressure values in the report (§14.4).

> _Design note (one-sided drift detection and sub-clip contamination)._ The CUSUM drift accumulator (§6.3) is deliberately one-sided upward, consistent with the polarity invariant (§5.1). It detects the _rate_ at which the fast baseline diverges above the slow baseline — not the _absolute level_ of either baseline. A patient adversary producing sustained sub-clip scores can inflate both baselines in lockstep, keeping the fast-slow gap below $\kappa$ and accumulating zero drift evidence, while reducing the system's sensitivity to future anomalous traffic. This is a known consequence of the adaptive baseline design: the same property that enables automatic adaptation to legitimate regime changes (the slow baseline eventually absorbs any sustained shift, §6.3 reset mechanism 1) also enables slow adversarial contamination. The CUSUM does not — and should not — detect this, because flagging slow baseline drift as suspicious would trigger on every legitimate regime change. Instead, the system reports the slow baseline values (§14.5) and clip-pressure state (§14.4), enabling hosts to build secular-trend monitors externally. See §17.5 for the full evasion assessment.

#### 6.4.6 $\lambda_\rho$ Guidance · `sec:sentinel:algorithm-clip-pressure-lambda-rho-guidance`

The trade-off axis is **detection latency vs. contamination resistance**, parameterized by $\lambda_\rho$:

| $\lambda_\rho$ | Ceiling half-life | Lockout recovery       | Contamination leakage | Regime                  |
| -------------: | ----------------: | ---------------------- | --------------------- | ----------------------- |
|           0.90 |        ~7 batches | Fast (~20 batches)     | Higher                | High-churn environments |
|           0.95 |       ~14 batches | Moderate (~40 batches) | Moderate              | **Default**             |
|           0.99 |       ~69 batches | Slow (~150 batches)    | Minimal               | Security-critical       |

The default optimizes for the common case where regime shifts are more frequent than sustained adversarial presence.

---

## Chapter 7. Hierarchical Coordination · `sec:sentinel:algorithm-hierarchical-coordination`

The coordination tier detects anomalous patterns in the _distribution of scores across spatially related cells_ at every level of the spatial tree hierarchy. It reuses the subspace tracker (§4) at a second level of abstraction: instead of modelling suffix bit vectors, it models cross-cell score summaries.

The coordination tier operates exclusively on the **competitive set** $\mathcal{A}$ (§8), not the full analysis set $\mathcal{A}^*$. Ancestor trackers (§8) provide per-value depth-of-defence; the coordination tier provides cross-cell pattern detection. These are interlocking constraints — their interaction is analysed in §16–17.

---

### 7.1 Coordination Contexts · `sec:sentinel:algorithm-coordination-contexts`

Every internal node of the spatial tree whose subtree contains **competitively selected** cells in **both** its left and right children's subtrees is a **coordination context**. The contexts form a binary hierarchy mirroring the G-Tree:

```
                 root                ← sees all K cells
                /    \
             g_L      g_R           ← each sees its subtree
            /   \    /   \
         ...   ...  ...   ...       ← narrower groups
          ↓     ↓    ↓     ↓
        cells cells cells cells
```

The **coordination group** at context $g$:

$$\mathcal{C}(g) = \big\{c \in \mathcal{A} : c \text{ is in the subtree of } g\big\}$$

A context is **active** when both subtrees contribute: $\mathcal{C}(g.\text{left}) \neq \emptyset$ and $\mathcal{C}(g.\text{right}) \neq \emptyset$.

**Group count.** A binary tree with $K$ leaves has at most $K - 1$ internal nodes, so the number of active contexts is at most $K - 1$. When analysed cells cluster spatially, most internal nodes have cells in only one subtree, and the count is much smaller.

**Nesting.** Groups nest by containment: descendant contexts have subsets of ancestor contexts' groups. A cell at G-Tree depth $d$ participates in at most $d$ contexts.

---

### 7.2 The Coordination Signal · `sec:sentinel:algorithm-coordination-signal`

After per-cell scoring (§4.2, Phase 1), each **competitive** cell $c \in \mathcal{A}$ that processed observations has a 4-dimensional summary:

$$\mathbf{o}_c = \big(\bar{s}_{c,\text{nov}},\; \bar{s}_{c,\text{disp}},\; \bar{s}_{c,\text{surp}},\; \bar{s}_{c,\text{coh}}\big) \in \mathbb{R}^4$$

These are **raw batch mean scores** — not z-scores. The coordination tracker learns its own normalisation.

For each active context $g$ with reporting group $\mathcal{C}_{\text{active}}(g) = \{c \in \mathcal{C}(g) : c \text{ has scores this batch}\}$, assemble:

$$O_g = \begin{pmatrix} \mathbf{o}_{c_1} \\ \vdots \\ \mathbf{o}_{c_m} \end{pmatrix} \in \mathbb{R}^{m \times 4}$$

**Cells with no observations in a given batch are excluded.** Only cells that processed at least one observation contribute rows. The group size $m$ may vary batch to batch. A context with $m < 2$ reporting cells in a given batch skips coordination for that batch.

---

### 7.3 Running-Mean Centring · `sec:sentinel:algorithm-coordination-running-mean-centring`

Each context maintains a 4-dimensional EWMA reference $\mu^{(\text{in})}_g \in \mathbb{R}^4$ at decay $\lambda$:

$$O_g^{(\text{c})} = O_g - \mathbf{1}_m \cdot \big(\mu^{(\text{in})}_g\big)^\top$$

After feeding the tracker, update:

$$\mu^{(\text{in})}_g \leftarrow \lambda \, \mu^{(\text{in})}_g + (1 - \lambda) \, \text{colmeans}(O_g)$$

On the first batch, $\mu^{(\text{in})}_g \leftarrow \text{colmeans}(O_g)$.

Running-mean centring detects both **differential** coordination (some cells anomalous, others normal — unusual structure in the centred vectors) and **uniform** coordination (all cells shifting together — the EWMA lags behind the shift, creating bias the tracker sees as elevated displacement). Per-batch centring would destroy the uniform signal.

---

### 7.4 Bottom-Up Assembly · `sec:sentinel:algorithm-coordination-bottom-up-assembly`

Coordination propagates from the leaves of the spatial tree toward the root. At each active context, the centred score matrix is assembled from the context's reporting group and fed through the context's coordination tracker.

```
procedure PropagateCoordination(g, CellScores):
    Reports ← empty list

    if g has no spatial children:
        if g is in the competitive set and has scores:
            return Reports, Cells = [(g, CellScores[g])]
        return Reports, Cells = empty

    LeftReports, LeftCells  ← PropagateCoordination(g.left, CellScores)
    RightReports, RightCells ← PropagateCoordination(g.right, CellScores)
    Reports ← LeftReports concatenated with RightReports

    MyCells ← LeftCells concatenated with RightCells
    if g is in the competitive set and has scores:
        append (g, CellScores[g]) to MyCells

    -- Fire coordination when both subtrees contribute
    if LeftCells is non-empty and RightCells is non-empty and |MyCells| ≥ 2:
        m ← |MyCells|
        O ← assemble m × 4 matrix from MyCells
        O_c ← O − 1_m · (μ_in[g])^T                       (§7.3)
        Report ← g.CoordTracker.Observe(O_c)                (§7.5)
        μ_in[g] ← λ · μ_in[g] + α · colmeans(O)
        append Report to Reports

    return Reports, MyCells
```

For semi-internal spatial nodes (one child), the single child's cells propagate upward without triggering coordination.

> _Implementation note._ The pseudocode recurses through the full spatial tree for clarity, visiting $|G|$ nodes at $O(1)$ each. An implementation should walk only the reduced Steiner tree of the investment set (§8.2) — whose internal branching nodes are exactly the active coordination contexts — visiting $O(|\mathcal{I}|)$ nodes rather than $|G|$. Note that competitive cells that are internal G-Tree nodes (§8.8) sit on the Steiner tree as interior nodes, not leaves; a bottom-up walk must inject their score contributions at the appropriate merge point rather than expecting them to propagate from below.

---

### 7.5 The Coordination Tracker · `sec:sentinel:algorithm-coordination-tracker`

Each active context maintains a subspace tracker (§4) with analysis width $w = 4$ and capacity $\text{cap} = \min(4, r_{\max})$. It runs the identical five-phase core loop, operating on $O_g^{(\text{c})} \in \mathbb{R}^{m \times 4}$ — centred score summaries — rather than suffix bit vectors.

At this level, the four axes measure:

| Coordination Axis | What It Measures                                                     |
| ----------------- | -------------------------------------------------------------------- |
| **Novelty**       | A _new kind_ of cross-cell score pattern not in the learned subspace |
| **Displacement**  | The group's mean score vector departed from its historical position  |
| **Surprise**      | A specific axis is systematically anomalous across cells             |
| **Coherence**     | An unusual _combination_ of axis elevations across cells             |

Each axis carries its own fast EWMA ($\lambda$), slow EWMA ($\lambda_{s,m}$), drift accumulator, and clip-pressure EWMA ($\lambda_\rho$).

> _Steady-state novelty saturation._ At $w = 4$ with $\text{cap} = 4$, the coordination tracker's rank adaptation commonly reaches $k = 4$ in steady state, at which point the novelty axis is saturated (§5.2). This is expected for moderate-to-large groups ($m \geq 4$) with diverse cross-cell traffic, where all four score-space dimensions carry comparable variance. For small groups ($m = 2$–$3$) with heterogeneous member cells, the novelty axis may remain active with 1 residual DOF, detecting unusual differential score-fluctuation directions. In both regimes, the three within-subspace axes (displacement, surprise, coherence) cover the coordination tier's primary detection mission — identifying optimisation-induced cross-cell correlations (§17.2). Hosts should interpret a consistently novelty-saturated coordination tracker as normal operation, not as a detection gap. The novelty-saturated flag (§14.7) and the scoring geometry distribution (§14.11.1) provide the reporting mechanism.

---

### 7.6 Multi-Scale Detection · `sec:sentinel:algorithm-coordination-multi-scale-detection`

The hierarchy's power derives from complementary sensitivity profiles at different scales.

**Localised anomaly** (affecting a small cluster of cells):

| Level                         | Sensitivity                   | Mechanism                                |
| ----------------------------- | ----------------------------- | ---------------------------------------- |
| Immediate parent ($m \sim 2$) | **Strong** — minimum dilution | Direct pattern change                    |
| Grandparent ($m \sim 4$–8)    | **Moderate**                  | Novelty: pattern not in learned subspace |
| Root ($m = K$)                | **Negligible**                | Diluted across all cells                 |

**System-wide shift** (all cells shifting):

| Level                     | Sensitivity                            | Mechanism                                        |
| ------------------------- | -------------------------------------- | ------------------------------------------------ |
| Leaf pairs ($m = 2$)      | **Slow** — fast EWMA adapts quickly    | Displacement rises                               |
| Mid-level ($m \sim 8$–32) | **Moderate** — SNR $\propto \sqrt{m}$  | Common-mode accumulation                         |
| Root ($m = K$)            | **Strongest** — SNR $\propto \sqrt{K}$ | Aggregate displacement; noise cancels coherently |

**The complementarity principle.** Localised anomalies are caught by small groups (high per-cell sensitivity). Global shifts are caught by large groups (high signal-to-noise through aggregation). The hierarchy covers the full spatial spectrum without configuration.

**Distinguishing partial from uniform coordination.** Compare per-cell and coordination signals. Partial coordination shows elevated per-cell drift accumulators in a subset with strong local coordination novelty. Uniform coordination shows individually unremarkable per-cell z-scores with strong root-level displacement. The _level_ at which coordination peaks indicates the _scale_ of the coordinated behaviour.

---

### 7.7 Lifecycle · `sec:sentinel:algorithm-coordination-lifecycle`

Coordination contexts are **lazily materialised** during the Step 5 coordination walk (§9.1) and **pruned** after the walk completes. No explicit activation or deactivation calls appear in Steps 0–4.

#### 7.7.1 Lazy Materialisation · `sec:sentinel:algorithm-coordination-lifecycle-lazy-materialisation`

When `PropagateCoordination` (§7.4) walks the reduced Steiner tree and encounters an internal node whose left and right subtrees both contribute competitive cells with scores in this batch, it checks whether a coordination context already exists for that node. If not, a fresh context is created — comprising a coordination tracker (§7.5), a running-mean reference $\mu^{(\text{in})}_g$, and associated state — and **inline-warmed** per §11.7 before the first real observation feeds the tracker. The inline warm-up runs to completion within the same Step 5 invocation.

This lazy model eliminates the bookkeeping of tracking coordination-eligible membership across phases. The walk itself discovers which contexts are needed, creates them on demand, and proceeds to use them — all within a single bottom-up pass.

#### 7.7.2 Post-Walk Pruning · `sec:sentinel:algorithm-coordination-lifecycle-post-walk-pruning`

After `PropagateCoordination` completes, contexts whose topology no longer warrants them — because one subtree no longer contains any online competitive cells — are **destroyed**, along with their coordination tracker and running-mean reference. The criterion is _membership_: whether online competitive cells exist in both subtrees, not whether those cells happened to contribute scores in this particular batch (a cell with zero observations in the current batch still counts as a member). On subsequent reactivation (when both subtrees again contribute), a fresh context is created and warmed per §7.7.1.

The rationale parallels per-cell tracker destruction on competitive exit (§8.5): a stale coordination model from a different competitive membership may be actively misleading. The competitive set membership that produced the old tracker's learned subspace, baselines, and drift accumulator state may differ substantially from the membership at reactivation. Preserving the old state would risk false drift evidence (the new score distribution differs from the old baseline) or suppressed detection (the old baseline absorbs anomalous patterns that the new composition would flag).

> _Design note (why not preserve)._ Preservation across deactivation gaps would require tracking which cells contributed to the old model and whether the new group is "close enough" to reuse it — a judgment call that introduces implicit assumptions about distributional continuity. Destruction and re-warm-up is simple, correct, and bounded-cost: the coordination tracker at $w = 4$ warms up quickly (§11.7, §7.8).\_

#### 7.7.3 Membership Changes Within a Cycle · `sec:sentinel:algorithm-coordination-lifecycle-cycle-membership-changes`

Step 3 reconciliation may change competitive set membership, which affects coordination contexts in two ways:

**Topology invalidation.** If a membership change empties one subtree of an active context — the last competitive cell in that subtree exits $\mathcal{T}$ — the `PropagateCoordination` walk's guard (`LeftCells is non-empty and RightCells is non-empty`) prevents the context from firing. Post-walk pruning (§7.7.2) then destroys the context. No "last firing" occurs; the context transitions directly from active to destroyed.

**Membership reduction.** If a membership change removes one or more cells from a context's group while leaving both subtrees populated, the context fires normally with the reduced group. The coordination tracker's learned model — subspace $U$, baselines, drift accumulator, and running-mean reference $\mu^{(\text{in})}_g$ — was trained on batches that included the departed cell's contributions. The model is therefore slightly stale relative to the current group composition: it expects score-matrix rows from $m_{\text{old}}$ cells and now receives $m_{\text{new}} < m_{\text{old}}$. This staleness is self-correcting: the EWMA adaptation absorbs the membership change over $O(t_{1/2})$ subsequent batches. During the transient, modest baseline disturbance is possible but bounded by the single-member contribution to the group statistics.

The symmetric case — membership _growth_ (a new competitive cell entering a subtree) — produces the analogous model staleness in the opposite direction. If the growth _creates_ a context (previously only one subtree was populated), the lazy materialisation mechanism (§7.7.1) handles it: a fresh context is created and inline-warmed before processing its first real observation.

> _Timing-channel note._ Because Step 3 updates `CurrentCompetitiveSet` before Step 4 populates `CellReports`, and Step 5's walk builds its leaf set exclusively from `CellReports`, no coordination report can contain a cell that has exited the competitive set. This is stronger than the original §7.7.3 claimed: there is no grace-period leak of recently-competitive membership through the coordination tier.

---

### 7.8 Memory and Computation · `sec:sentinel:algorithm-coordination-memory-and-computation`

**Per context:** approximately 200 floating-point elements for the subspace tracker at $w = 4$, plus the 4-element running-mean reference. Total across all contexts: at most $(K - 1) \times (\text{per-context size})$.

**Per-context per-batch computation:** $O(m)$ — the $w = 4$ constraint makes every operation linear in group size. The SVD of an $\mathbb{R}^{4 \times (k+m)}$ matrix costs $O(16(k+m)) \approx 16m$ multiply-adds — less than 0.3% of a single per-cell SVD at $w = 96$.

**Total per-batch coordination work:** $O(K \cdot \bar{d})$ useful work, where $\bar{d}$ is average coordination participation depth. The pseudocode (§7.4) visits the full spatial tree for clarity; an implementation walking the investment set's reduced Steiner tree (§8.2) achieves this bound with $O(|\mathcal{I}|)$ traversal overhead (approaching $O(K)$ under typical spatial clustering). Negligible relative to per-cell scoring in either case.

---

# Part III — Selection and Assembly · `sec:sentinel:algorithm-selection-and-assembly`

---

## Chapter 8. Cell Selection · `sec:sentinel:algorithm-cell-selection`

The analysis selector determines which cells from the spatial layer receive statistical modelling. It maintains two related but distinct sets: the **investment set** (cells that have been allocated trackers and warm-up resources) and the **producing set** (the online subset that actively generates scores). Competitive selection identifies significant cells, ancestor closure guarantees a complete model chain to the root, and the warm-up pipeline (§11) brings invested cells online in g.sum order.

### 8.1 Competitive Selection · `sec:sentinel:algorithm-competitive-selection`

Let $\mathcal{E} = \{v \in \text{V-entries} : \text{depth}_V(v) \leq L \;\wedge\; w(v) \geq 2\}$ be the **eligible set** — all V-entries within the depth cutoff whose analysis width $w = N - d$ is at least 2. The $w \geq 2$ predicate excludes cells where the subspace algebra is undefined (§4.1). The **competitive targets** are:

$$\mathcal{T} = \text{top}_K\!\big(\mathcal{E},\; v.\text{importance}\big)$$

If $|\mathcal{E}| \leq K$, then $\mathcal{T} = \mathcal{E}$.

| Parameter             | Constraint | Role                                           |
| --------------------- | ---------- | ---------------------------------------------- |
| $K$ (analysis budget) | $\geq 1$   | Maximum number of competitively selected cells |
| $L$ (depth cutoff)    | $\geq 0$   | V-Tree depth ceiling for eligibility           |

The selection criteria use V-Tree ranking (depth and importance) as the sole competitive mechanism. The $w \geq 2$ predicate is a static geometric precondition excluding cells where the subspace algebra is undefined (§4.1), not a dynamic structural filter. The analysis selector does not inspect G-Tree structural state (terminal, semi-internal, or internal). This is a deliberate design choice; see the design note below.

**Ties at the boundary.** When more than $K$ eligible entries exist, ties in importance at the $K$-th position are broken by the left endpoint of the cell's spatial interval (deterministic, spatially stable).

**Recomputation.** The competitive targets $\mathcal{T}$ are recomputed after each observation pass (§9, Step 2), since splits, evictions, and rebalancing may change V-Tree depths and importance values. The investment set and producing sets are derived from $\mathcal{T}$ during Step 3. An implementation may maintain $\mathcal{T}$ incrementally via a bounded priority structure keyed on importance, updated during rebalancing notifications.

> _Design note (why no G-Tree state filter)._ When a high-importance contour cell splits, the parent becomes an internal G-Tree node with frozen importance — children intercept all spatial-layer routing (§3.7). One might exclude such nodes from $\mathcal{A}$ on the grounds that they receive no routed observations and their slot duplicates what the ancestor closure (§8.2) provides for free. We deliberately do **not** apply this filter, for three reasons:
>
> 1. **Single ranking authority.** The V-Tree is the sole arbiter of competitive significance (§3.4). Its depth encodes proven importance relative to the entire neighbourhood. Adding a G-Tree structural predicate means the analysis selector second-guesses the ranking mechanism with spatial-layer implementation state — a cross-layer coupling the architecture otherwise avoids.
> 2. **The transient is the V-Tree working correctly.** The frozen parent holds a shallow V-Tree position because it _earned_ that position through sustained observation volume. Its children start at zero importance and have not yet demonstrated significance. Selecting the parent during this period is the V-Tree making a factually correct statement: this region has proven importance; its subdivisions have not. The max-uncle constraint (§3.4) and temporal decay (§3.6, §3.8) organically resolve the situation — children's importance grows, the frozen benchmark weakens, the V-Tree restructures, and the parent sinks past $L$ or out of the top-$K$.
> 3. **Coordination coverage during transition.** If the parent were immediately ejected from $\mathcal{A}$ upon splitting, the highest-importance region in the system would have _no competitive-level representation_ in the coordination tier (§7) until its children earn entry. Retaining the parent provides the only coordination-tier coverage of that region during the transition. Its tracker is fully active — the multi-scale delivery mechanism (§9.3) feeds it every observation in its subtree — so the slot is not dormant.
>
> The cost of this choice is that during the transient, one $K$-slot is occupied by a node whose tracker does the same work an ancestor tracker would do for free. This cost is bounded: it persists only until the V-Tree's competitive dynamics push the frozen entry below the selection threshold, which is proportional to $O(\theta / \text{observation\_rate})$ batches under typical decay. At small $K$ the cost is one displaced contour cell; at large $K$ it is negligible. The organic resolution — without cross-layer intervention — is the preferred design.

### 8.2 The Investment Set · `sec:sentinel:algorithm-investment-set`

The **investment set** closes the competitive targets under spatial tree ancestry:

$$\mathcal{I} = \mathcal{T} \;\cup\; \bigcup_{v \in \mathcal{T}} \text{Ancestors}(v.\text{node})$$

where $\text{Ancestors}(g)$ is the set of all materialised G-Tree nodes on the path from $g$ to the G-Tree root, inclusive. Since the G-Tree is fully materialised (§3.1), a target at depth $d$ contributes ancestors at depths $0, 1, \ldots, d - 1$. Every competitive target receives a **complete chain of allocated trackers** from itself to the root.

The investment set determines resource commitment: every member of $\mathcal{I}$ has a tracker allocated and, if not yet online, is enqueued for warm-up. Not all members of $\mathcal{I}$ produce scores — only online members do (§8.3).

The investment set forms a **Steiner tree** connecting the competitive targets to the root. Its size depends on how much the targets share ancestors and on the G-Tree depth of each target. By the Steiner tree property, the reduced tree (suppressing degree-2 path nodes) connecting $K$ leaves to a common root has at most $K - 1$ internal branching points, yielding at most $2K - 1$ reduced nodes. However, the full investment set includes all materialised intermediate nodes on the paths, which may exceed this.

**Size bound.** Each competitive target at G-Tree depth $d_i$ contributes $d_i$ ancestors; the root is shared by all. Before accounting for sharing:

$$|\mathcal{I}| \leq 1 + \sum_{i=1}^{K} d_i = 1 + K\bar{D}$$

with equality when all $K$ targets share only the root. The reduced Steiner tree connecting $K$ leaves to the root has at most $K - 1$ internal branching nodes and at most $2K - 1$ reduced nodes. The full materialised tree adds chain intermediaries — degree-2 nodes suppressed in the reduction — whose count depends on the depth profile. Under concentrated observation volume, extensive ancestor sharing absorbs chain intermediaries into the shared structure, and the practical investment set size approaches $2K$.

**Example 1 (typical, `depth_create = 3`):**

```
Root /0                        ← shared by all
├── /1-A                       ← shared by c₁, c₂
│   ├── /2-A                   ← shared by c₁, c₂
│   │   ├── /3-A  ★ c₁
│   │   └── /3-B  ★ c₂
│   └── (unresolved)           ← no split warranted
└── /1-B                       ← unique to c₃
    └── /2-C                   ← unique to c₃
        └── /3-C  ★ c₃

Targets: 3 (each at depth 3, D̄ = 3)
Unique ancestors: 5 (root, /1-A, /2-A, /1-B, /2-C)
|I| = 8
Worst-case bound: 1 + 3×3 = 10;  actual: 8 (sharing root, /1-A, /2-A)
```

Three targets, five unique ancestors. The worst-case bound gives $1 + 3 \times 3 = 10$; the actual count is 8 because the root, /1-A, and /2-A are each shared by multiple targets. The reduced Steiner tree has 5 nodes ($2K - 1$: root, /2-A, and the three targets); the full materialised tree adds 3 chain intermediaries (/1-A, /1-B, /2-C) for a total of 8.

**Example 2 (deep concentration, 256-bit domain):**

```
Root /0
│  ... 232 shared levels ...
Depth 232 (hot region)
├── branching: ≤ 999 internal nodes
└── 1000 competitive targets at depth ~235

Targets: 1000    D̄ ≈ 235    Sharing: 232 trunk + ≤999 branch
Worst-case: 1 + 1000×235 = 235,001
Actual: 1000 + 999 + 232 = 2231  (sharing absorbs 99%)
```

All 1000 targets share 232 trunk ancestors. The branching region adds at most $K - 1 = 999$ internal nodes, plus a few chain intermediaries per branch. Total investment: $\sim 2231$, dominated by $2K$. The trunk and branching structure are shared, not multiplied per target — concentration creates the depth that makes sharing inevitable.

**Dimension guard.** Under the $w \geq 2$ eligibility predicate in §8.1, no member of $\mathcal{I}$ can have $w < 2$: every competitive target has $w \geq 2$ by eligibility, and every ancestor has $w' = N - d' > N - d \geq 2$ since ancestors sit at strictly shallower G-Tree depth. This guard is retained as defensive specification against future changes to the eligibility predicate: if any member of $\mathcal{I}$ were to have $w < 2$, no tracker would be allocated, and the exclusion would be reported as a degenerate-cell count (§14.11).

### 8.3 The Producing Sets · `sec:sentinel:algorithm-producing-sets`

The **producing sets** are the online subsets of the investment set:

$$\mathcal{A} = \mathcal{I} \cap \mathcal{T} \cap \text{Online}$$

$$\mathcal{A}^* = \mathcal{I} \cap \text{Online}$$

Only members of $\mathcal{A}^*$ deliver suffix vectors to trackers and emit scores. A cell transitions from invested to producing when its warm-up completes and it is promoted to online status (§11.6).

**Relationship to the investment set.** Every member of $\mathcal{A}^*$ is a member of $\mathcal{I}$. The converse does not hold during warm-up: warming cells are in $\mathcal{I}$ but not in $\mathcal{A}^*$. In steady state with no warming cells, $\mathcal{A}^* = \mathcal{I}$.

**Slot-vacancy invariant.** While any competitive target is warming, the producing competitive set has a corresponding vacancy:

$$|\mathcal{A}| = |\mathcal{T} \cap \text{Online}| = K - |\mathcal{T} \cap \text{Warming}|$$

A warming cell holds a $\mathcal{T}$ slot (resource commitment) without occupying an $\mathcal{A}$ slot (production output). At promotion (§11.6.3, Step 0), the cell transitions to Online, and the next Step 3 recomputation yields $\mathcal{A} = \mathcal{I} \cap \mathcal{T} \cap \text{Online}$ with the promoted cell filling its own formerly vacant slot. No displacement logic is needed: $|\mathcal{A}| \leq K$ is maintained structurally by Step 3's top-$K$ recomputation of $\mathcal{T}$ (§8.1), from which $\mathcal{A}$ is derived by intersection.

> _Design note (no promotion-time displacement)._ Competitive-set turnover — an existing $\mathcal{T}$ member falling below the top-$K$ boundary — can occur in the same batch as a promotion, but the two events are causally independent. The ejection is driven by Step 2's V-Tree mutations (splits, rebalancing, decay) feeding into Step 3's top-$K$ selection, not by the promotion itself. The Step 3 pseudocode (§9.1) contains no displacement step because none is required: `NewCompetitive ← NewTargets ∩ Online` is capped at $K$ by construction.

### 8.4 The Root Tracker · `sec:sentinel:algorithm-root-tracker`

The G-Tree root tracker deserves special attention. It:

- Sees **every** observation — there is no way to avoid it.
- Operates at width $N$ — the full domain.
- Has the largest training set — learns the most stable model.
- Provides the global reference against which everything is measured.

The root tracker answers: _"Does this value look like a normal member of the overall population?"_ Every other tracker answers: _"Does this value look normal for its specific region?"_

A value can be perfectly normal for its region (low /48 scores) but unusual globally (high /0 scores) — for example, if the entire region is unusual. Or it can be unusual locally (high /48 scores) but unremarkable globally (low /0 scores) — a local anomaly within a normal region. The combination is diagnostic.

For evasion analysis: the root tracker is the hardest to evade. It has the most data, the most stable subspace, and captures the broadest correlations. An adversary who successfully mimics local distributional patterns at /48 may still produce an unusual projection at /0, because the root model captures global cross-region correlations that no single cell's model can learn.

The root tracker is **permanent** — created at system initialisation, never destroyed. Its warm-up time is amortised across the system's lifetime.

### 8.5 Entry and Exit · `sec:sentinel:algorithm-cell-selection-entry-and-exit`

**Investment entry.** When a cell first appears in $\mathcal{I}$ (either as a competitive target or as an ancestor of one):

1. Create a subspace tracker (§4) at width $w = N - d$.
2. Enqueue for warm-up (§11.6).

**Promotion to producing.** When a warming cell's warm-up completes (§11.6.3):

1. Transition from warming to online.
2. If the cell is a competitive target, it enters $\mathcal{A}$.
3. Coordination contexts involving this cell materialise lazily during the next Step 5 coordination walk (§7.7.1).

**Investment exit (eager removal).** When a cell leaves $\mathcal{I}$ — because its competitive target dropped out of the top-$K$ and no other target needs it as an ancestor:

1. **If warming:** cancel warm-up, destroy the tracker immediately.
2. **If online:** destroy the tracker immediately. Stale coordination contexts are pruned by the Step 5 post-walk pass (§7.7.2).

Destruction is **eager** — it occurs within the Step 3 reconciliation that discovers the exit, not deferred to a later pass. The tracker, its subspace, baselines, drift accumulators, and any in-progress warm-up state are released immediately.

**The root tracker exception.** The root tracker is never destroyed, even if it is temporarily the only member of $\mathcal{I}$. It is created at system initialisation and persists for the system's lifetime.

**No hysteresis.** The V-Tree's max-uncle constraint (§3.4) provides structural stability: demotion requires genuine competitive loss, not transient fluctuation. This structural guarantee _is_ the hysteresis.

**Ancestor stability under sharing.** An ancestor survives in $\mathcal{I}$ as long as **any** competitive target descends from it. Investment churn at the competitive boundary does not destroy shared ancestors. The root's lifetime is the system's lifetime.

### 8.6 Budget Accounting · `sec:sentinel:algorithm-cell-selection-budget-accounting`

The budget parameter $K$ governs the **competitive target** selection. Ancestor trackers are not counted against $K$.

**Investment set size.** $|\mathcal{I}| \leq 1 + K\bar{D}$ before sharing (§8.2). The reduced Steiner tree has at most $2K - 1$ nodes; the full materialised tree adds chain intermediaries. Under concentrated observation volume, extensive ancestor sharing brings the practical size close to $2K$.

**Producing set size.** $|\mathcal{A}^*| \leq |\mathcal{I}|$, with equality in steady state (no warming cells).

**Peak memory.** Bounded by $|\mathcal{I}| \times (\text{max per-tracker size})$, since all invested cells — whether warming or online — have allocated trackers. This bound applies during warm-up; in steady state, memory equals $|\mathcal{A}^*| \times (\text{max per-tracker size})$.

Ancestor trackers at coarser depths are more expensive per unit because they have wider suffixes. Taking $N = 128$ and $\text{cap} = 16$ as an example:

| Depth    | Width $w$ | $U$ matrix elements    | Approximate per-tracker size |
| -------- | --------- | ---------------------- | ---------------------------- |
| 0 (root) | $N$       | $128 \times 16 = 2048$ | $\sim 2100$ elements         |
| 16       | $N - 16$  | $112 \times 16 = 1792$ | $\sim 1850$ elements         |
| 32       | $N - 32$  | $96 \times 16 = 1536$  | $\sim 1600$ elements         |
| 48       | $N - 48$  | $80 \times 16 = 1280$  | $\sim 1350$ elements         |

Total additional memory for ancestors is roughly proportional to $K$. See §12 for global resource accounting.

### 8.7 Tracker Configuration at Different Levels · `sec:sentinel:algorithm-cell-selection-level-specific-tracker-configuration`

Ancestor trackers use the same maximum rank, forgetting factor, and other parameters as competitive trackers by default. A natural gradient is available via optional per-depth overrides: coarser levels see more observations and broader patterns, suggesting higher rank (more principal components for richer structure) and slower forgetting (more stable baselines for the broader population). The root tracker particularly benefits from higher rank — it learns the global structure of the $N$-bit domain, which is likely higher-dimensional than within-cell structure.

This is a configuration refinement. The default — same parameters everywhere — is a reasonable starting point.

### 8.8 Edge Cases · `sec:sentinel:algorithm-cell-selection-edge-cases`

**Cells receiving no observations in a batch.** An analysed cell may receive zero routed observations in a given batch. Its tracker processes no data and emits no scores. Competitive cells with no observations are excluded from that batch's coordination matrices (§7.2).

**Internal G-Tree nodes in the competitive targets.** An internal G-Tree node (both children present) receives no observations via normal spatial routing (§3.10) — its importance is frozen from the moment of its second child's creation (§3.7). However, the competitive selection (§8.1) does not filter by G-Tree state: the V-Tree ranking is the sole eligibility criterion. A freshly-split internal node with high historical importance can therefore occupy a competitive target slot.

This is not a defect. The node's tracker is fully active: the multi-scale delivery mechanism (§9.3) feeds it every observation in its subtree, it scores them, and it participates in coordination (§7). Its analysis is identical to what an ancestor tracker would provide — the cost is one $K$-slot that duplicates the ancestor closure's work. The V-Tree's competitive dynamics (§3.4) resolve this organically: children's importance grows from live observations, the frozen benchmark decays under temporal attenuation (§3.8), the max-uncle constraint triggers restructuring, and the parent sinks below the selection threshold. The transient duration is bounded by $O(\theta / \text{observation\_rate})$ batches under typical decay.

The alternative — filtering by G-Tree state to eject internal nodes immediately — was considered and rejected. It would create cross-layer coupling (the analysis selector inspecting spatial-layer structural state) and would leave the highest-importance region with no competitive-level coordination representation during the transition. See the design note in §8.1.

---

## Chapter 9. The Observation Algorithm · `sec:sentinel:algorithm-observation-algorithm`

### 9.1 The Algorithm · `sec:sentinel:algorithm-observation-algorithm-procedure`

If the ingestion batch is empty ($n = 0$), the observation algorithm returns an empty report without modifying any state. The remainder of this procedure assumes $n \geq 1$.

```
procedure SentinelIngest(Batch):
    Input: Batch — a sequence of coordinate values of type C
    if Batch is empty:
        return EmptyReport()

    -- Step 0 — Promote warmed-up cells from staging (§11)
    PromoteReadyCells()

    -- Step 1 — Convert to centred bit vectors (§2.3)
    Vectors ← ConvertToCentredBits(Batch)

    -- Step 2 — Spatial layer volume accounting (§3.3)
    for each value v in Batch:
        SpatialLayer.Observe(coordinate = v, Δ = 1)
    -- Routes, accumulates, may split/rebalance/evict.

    -- Step 3 — Update investment and producing sets (§8)
    OldInvestment ← CurrentInvestmentSet
    NewTargets ← ComputeCompetitiveTargets()                       (§8.1)
    NewInvestment ← CloseUnderAncestry(NewTargets)                  (§8.2)

    -- Eager removal of exiting nodes
    Exiting ← OldInvestment \ NewInvestment
    for cell in Exiting:
        if cell = root: continue                                    (§8.4)
        if cell is warming:
            CancelWarmUp(cell)
        DestroyTracker(cell)                                        (§8.5)

    -- Investment entry for new nodes
    Entering ← NewInvestment \ OldInvestment
    for cell in Entering:
        CreateTracker(cell)
        EnqueueForWarmUp(cell)                                      (§11.6)

    -- Derive producing sets from investment + online status
    NewCompetitive ← NewTargets ∩ Online
    NewFull ← NewInvestment ∩ Online

    -- No explicit coordination activation/deactivation here.
    -- Coordination context lifecycle is managed lazily in Step 5.

    CurrentInvestmentSet ← NewInvestment
    CurrentCompetitiveTargets ← NewTargets
    CurrentCompetitiveSet ← NewCompetitive
    CurrentFullAnalysisSet ← NewFull

    -- Step 4 — Deliver suffix vectors and score (§9.3, §9.4)
    CellReports ← empty list
    AncestorReports ← empty list
    for each value v in Batch:
        Receiver ← RouteToOnlineReceiver(v)                        (§9.3)
        for each node g on the path from Receiver to root:
            if g is in CurrentFullAnalysisSet:
                Suffix ← Vectors[v] restricted to positions [g.depth .. N−1]
                append Suffix to g.PendingBatch

    for each cell in CurrentFullAnalysisSet:
        if cell.PendingBatch is empty: continue
        X ← assemble matrix from cell.PendingBatch
        Report ← cell.Tracker.Observe(X)                          (§4.2)
        if cell is in CurrentCompetitiveSet:
            append (cell, Report) to CellReports
        else:
            append (cell, Report) to AncestorReports
        clear cell.PendingBatch

    -- Step 5 — Hierarchical coordination (§7.4)
    -- Coordination contexts are lazily created and inline-warmed
    -- during the walk; stale contexts are pruned afterward (§7.7).
    CoordinationReports ← PropagateCoordination(root, CellReports)
    PruneStaleCoordinationContexts()                               (§7.7.2)

    -- Step 6 — Assemble report (§14)
    return AssembleReport(CellReports, AncestorReports,
                          CoordinationReports)
```

> **Per-cell batch size.** For each cell $c$ in the producing set $\mathcal{A}^*$, the per-tracker batch size $b$ equals the number of ingestion-batch observations whose spatial routing passes through $c$. At the root, $b = n$; at non-root cells, $b \in \{0, \ldots, n\}$ depends on the spatial distribution of the current batch. Cells with $b = 0$ accumulate no pending observations and are skipped by the `if cell.PendingBatch is empty: continue` guard in Step 4. The effective per-batch work is therefore $\sum_{c \in \mathcal{A}^* :\, b_c > 0} O\!\big(\min(w_c,\, k_c + b_c)^2 \cdot \max(w_c,\, k_c + b_c)\big)$, which may be substantially less than the worst-case bound in §12.7 when observations are spatially concentrated.

### 9.2 Step Ordering · `sec:sentinel:algorithm-observation-step-ordering`

| Ordering                          | Reason                                                                                                                                                                                                                                    |
| --------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Step 0 before Step 2              | Newly promoted cells must participate in observation routing.                                                                                                                                                                             |
| Step 2 before Step 3              | Splits and rebalancing change V-Tree ranking. The analysis set is computed against the post-observation state.                                                                                                                            |
| Step 3: investment set reconciled | Exiting nodes eagerly removed. Entering nodes enqueued for warm-up (§11). No inline per-cell warm-up work; no coordination lifecycle management.                                                                                          |
| Step 4 before Step 5              | Suffix vectors must be collected and scored before coordination.                                                                                                                                                                          |
| Step 5: coordination lifecycle    | Coordination contexts are lazily materialised and inline-warmed during the coordination walk (§11.7). Stale contexts are pruned after the walk. Inline warm-up is justified by the trivial cost at $w = 4$ (~50 rounds of $O(16m)$ work). |
| Step 5 before Step 6              | Coordination requires per-cell score summaries.                                                                                                                                                                                           |

### 9.3 Multi-Scale Delivery · `sec:sentinel:algorithm-observation-multi-scale-delivery`

A single observation is delivered to **every tracker on its G-Tree ancestor path**, from the receiving cell up to and including the root. This delivery is **mandatory**, not contingent on competitive selection — the ancestor closure (§8.2) guarantees that every node on the path hosts a tracker.

If a value routes to a depth-4 cell with materialised ancestors at depths 0, 1, 2, and 3, all five trackers receive the observation — at analysis widths $N$, $N-1$, $N-2$, $N-3$, and $N-4$ respectively. Each ancestor covers twice the dyadic range of its child, providing a complete hierarchy of spatial context from the cell's local population to the global population in $d$ logarithmic steps. The delivery occurs regardless of whether the intermediate G-Tree nodes receive observations through normal spatial layer routing. Under typical parameters (`depth_create` $\approx 3$, competitive mechanism limiting depth to $\sim 2$–$8$), this fan-out is modest: a depth-3 cell produces 4 tracker updates per observation; a depth-6 cell produces 7.

**Routing during warm-up.** Observations destined for a cell in $\mathcal{I}$ that is not yet online are routed to the nearest **online** ancestor on the G-Tree path, or to the root if no closer ancestor is online. The root is always online (§8.4). Because the warm-up pipeline processes cells in g.sum order (§11.6.2), ancestors come online before descendants — the routing fallback naturally shortens as warming progresses.

```
procedure RouteToOnlineReceiver(value):
    g ← SpatialLayer.RouteToReceiver(value)
    while g is not null:
        if g is online:
            return g
        g ← g.SpatialParent
```

Because the root completes warm-up at construction (§11.6), the while loop has a guaranteed termination point: the root is always online. For cells whose closer ancestors are still warming, the root is returned; as ancestors complete warm-up (in g.sum order, §11.6.2), the returned node moves progressively closer to the target.

No scoring degradation occurs during this period — the ancestor already has a mature model. Resolution stays at the coarser level until the descendant completes warm-up and is promoted to online status. The host sees the descendant's spatial interval covered by the ancestor's report until promotion.

> _Design note (overlapping inputs)._ Observation sets overlap by construction — a depth-16 ancestor sees every suffix that its depth-32 children see, plus more. The overlapping input is by design: the depth-16 tracker learns coarser structure than the depth-32 tracker from the same observations. The coordination tier (§7) operates on derived score summaries from the competitive set $\mathcal{A}$, not from ancestor trackers, so input overlap does not produce double-counting at the coordination level.

### 9.4 Per-Value Scoring Pipeline · `sec:sentinel:algorithm-observation-per-value-scoring-pipeline`

Under the ancestor closure, a single observation at a competitive cell of depth $d$ produces scores at every ancestor level:

```
Value arrives at depth-4 cell
│
├── /0 tracker: novelty₀, displacement₀, surprise₀, coherence₀
│   └── z-scores against /0 baselines
│
├── /1 tracker: novelty₁, displacement₁, surprise₁, coherence₁
│   └── z-scores against /1 baselines
│
├── /2 tracker: novelty₂, displacement₂, surprise₂, coherence₂
│   └── z-scores against /2 baselines
│
├── /3 tracker: novelty₃, displacement₃, surprise₃, coherence₃
│   └── z-scores against /3 baselines
│
├── /4 tracker: novelty₄, displacement₄, surprise₄, coherence₄
│   └── z-scores against /4 baselines
│
└── Coordination (on competitive cells only)
    └── Cross-cell pattern detection (§7)
```

The report for a single value includes scores at every ancestor level. The host sees not just "this value is anomalous" but "this value is anomalous at the depth-3 level but normal at depth 1 and depth 4" — diagnostic information about the **scale** at which the anomaly manifests.

- An anomaly at depth 4 only: unusual relative to the specific cell's population but normal in the broader context — a local anomaly.
- An anomaly at depth 1 and above: unusual in the broader regional context — a stronger, coarser-scale signal.
- An evasion succeeding at depth 4 but failing at depth 3: the adversary matched local patterns but missed cross-cell correlations captured by the coarser model. This is the nesting defence.

### 9.5 Batched Observation Semantics · `sec:sentinel:algorithm-observation-batched-semantics`

Step 2 accumulates $\Delta = 1$ per value. If $n$ values in a batch route to the same cell, its importance increases by $n$. Steps 2 and 4 may share a single tree descent in implementation.

---

# Part IV — Temporal and Operational Concerns · `sec:sentinel:algorithm-temporal-and-operational-concerns`

---

## Chapter 10. Temporal Semantics · `sec:sentinel:algorithm-temporal-semantics`

Two independent temporal mechanisms govern two independent concerns. Neither subsumes the other; both operate simultaneously.

### 10.1 Spatial Decay · `sec:sentinel:algorithm-temporal-spatial-decay`

The host periodically invokes the spatial layer's temporal decay operation (§3.6). The spatial layer applies the requested scaling factor to accumulated importance values, producing one of four effects:

| Regime        | Factor                 | Effect                                                                                   |
| ------------- | ---------------------- | ---------------------------------------------------------------------------------------- |
| Attenuation   | $(0, 1)$               | Cold cells lose standing, eventually becoming eviction candidates                        |
| Amplification | $> 1$                  | Hot cells reinforced; with depth-selective amplification, fine-scale detail is sharpened |
| Annihilation  | $= 0$                  | Hard reset — entire regions zeroed                                                       |
| Detail flush  | $= 0$, depth-selective | Subtree root preserved; all descendants zeroed                                           |

The host controls the decay schedule: when to invoke it, at what factor, against which subtree, and with what depth selectivity. Spatial decay governs the **contour's memory** — which cells exist and in what competitive standing.

### 10.2 Statistical Decay · `sec:sentinel:algorithm-temporal-statistical-decay`

Each subspace tracker's EWMA at rate $\lambda$ governs the subspace, latent statistics, and baselines independently of spatial decay. Statistical decay governs the **model's memory** — what structure has been learned and how recent observations are weighted.

| What decays          | Mechanism                   | Rate        | Purpose                     |
| -------------------- | --------------------------- | ----------- | --------------------------- |
| Importance (spatial) | Spatial layer               | Host-chosen | Contour evolution           |
| Subspace $\sigma$    | Tracker SVD (§4.2, Phase 2) | $\lambda$   | Forget old correlations     |
| Fast baselines       | Tracker EWMA (§6.1)         | $\lambda$   | Adapt to score distribution |
| Slow baselines       | Tracker EWMA (§6.2)         | $\lambda_s$ | Long-memory drift reference |

A cell can retain its spatial position (strong volume, slow spatial decay) while its statistical model adapts rapidly (fast $\lambda$), and vice versa.

### 10.3 Independence · `sec:sentinel:algorithm-temporal-independence`

The two decay mechanisms are entirely orthogonal:

- Spatial decay does not touch tracker state. It zeroes importance, which causes cells to lose V-Tree standing, exit the competitive set, and have their trackers destroyed through the normal exit mechanism (§8.5). The feed-forward invariant (§1.3) is maintained.
- Statistical decay does not touch importance. It governs how quickly a tracker forgets old subspace structure and baseline values.

The host sets the temporal policy for both mechanisms independently. Fast spatial decay with slow statistical decay produces a rapidly evolving contour with stable models. Slow spatial decay with fast statistical decay produces a stable contour with rapidly adapting models.

### 10.4 Annihilation Use Cases · `sec:sentinel:algorithm-temporal-annihilation-use-cases`

- **Regime change.** Apply annihilation at the G-Tree root with zero depth selectivity: resets the entire spatial structure. All cells lose standing and are eventually evicted; their trackers are destroyed.
- **Suspected poisoning.** Apply annihilation at a subtree root with full depth selectivity (detail flush): preserves coarse measurement, zeroes fine structure. The subtree root retains its importance; descendants must re-earn theirs.
- **Stale cleanup.** Apply targeted annihilation to force eviction in the next tidal pass. Useful for host-directed pruning of known-dead regions.

---

## Chapter 11. System Initialisation and Warm-Up · `sec:sentinel:algorithm-system-initialisation-and-warmup`

A newly created tracker has an orthonormal basis (§4.1) and uninitialised baselines. Before it can produce meaningful scores, its baselines must reflect a representative score distribution. This chapter specifies the complete warm-up procedure: noise injection, cold-start seeding, clip-width modulation, slow-baseline seeding, deferred scheduling, and coordination warm-up. These mechanisms address different aspects of a single problem — cold-start mismatch — and are presented together because their interactions determine the system's convergence behaviour.

### 11.1 Noise Injection · `sec:sentinel:algorithm-noise-injection`

#### 11.1.1 Purpose · `sec:sentinel:algorithm-noise-injection-purpose`

Noise injection feeds synthetic uniformly random centred bit vectors through the tracker's standard observation path (§4.2) to **warm the EWMA baselines** so they reflect structureless-noise score distributions rather than uninitialised placeholders.

> _Design note._ Noise injection's primary value is baseline warming, not subspace shaping. Random bit vectors in $\{-0.5, +0.5\}^w$ have no dominant direction — after injection, singular values are roughly equal, similar to the initial state. The subspace acquires meaningful structure only once real observations arrive.

#### 11.1.2 Trigger Events · `sec:sentinel:algorithm-noise-injection-trigger-events`

Noise injection fires on every new tracker creation: competitive set entry, ancestor activation, split-induced creation, or legacy promotion.

#### 11.1.3 Parameters · `sec:sentinel:algorithm-noise-injection-parameters`

| Parameter        | Description                          | Default                                                      | Constraint                                   |
| ---------------- | ------------------------------------ | ------------------------------------------------------------ | -------------------------------------------- |
| Noise schedule   | Depth-tiered synthetic batch count   | Geometric: 450 rounds at root, halving per depth, minimum 50 | See below                                    |
| Noise batch size | Samples per synthetic batch          | 16                                                           | $\geq 2$ recommended; $\geq 1$ required      |
| Random seed      | Seed for the pseudo-random generator | Deterministic (fixed seed)                                   | Optional; absent means implementation-chosen |

> _Operational note (noise batch size)._ Under the EWMA-mean-centred variance formula (§4.2 Phase 3, [ADR-S-021](../adr/021-ewma-mean-centred-variance.md)), the first-batch seeding computes $\nu^{(z)}_j = \max(z_{1j}^2, \varepsilon)$, which equals approximately $0.25$ in expectation for centred $\pm 0.5$ inputs — correctly scaled at every batch size including $b = 1$. The previous formula computed within-batch population variance, which is identically zero at $b = 1$; the EWMA-mean-centred formula eliminated this entire class of batch-size bias. Smaller batch sizes produce noisier initial seeds (higher variance of $z_{1j}^2$ across dimensions), but the EWMA smooths this within $O(t_{1/2})$ subsequent batches without a recovery phase. The default of 16 remains well above the threshold for low-noise seeding.

The noise schedule determines how many synthetic batches each tracker receives before transitioning to real observations. Two schedule forms are supported:

- **Geometric.** Parameterised by a root count, a per-depth decay factor, and a minimum: $\text{rounds}(d) = \max(\text{minimum}, \; \lfloor\text{root} \times \text{decay}^d + 0.5\rfloor)$
- **Explicit.** A sequence of per-depth round counts; the last entry repeats for all deeper levels.

The schedule must produce enough rounds for worst-case baseline convergence at each depth (see §11.9 for empirical convergence data; the same schedule parameterises coordination warm-up in §11.7).

After injection completes, all four drift accumulators (§6.3) are reset to zero.

### 11.2 Cold-Start Latent Seeding · `sec:sentinel:algorithm-cold-start-latent-seeding`

On a tracker's very first batch (Phase 3 of the core loop, §4.2), the latent statistics $\mu^{(z)}$, $\nu^{(z)}$, and $\Gamma$ are seeded directly from the batch data rather than blended with default initial values.

Without direct seeding, the latent variance $\nu^{(z)}$ retains its pre-allocated value of $1.0$. The EWMA-mean-centred formula (§4.2, Phase 3) converges to the true variance from any initialisation, but the $4\times$ overestimate dampens surprise scores during the convergence period, delaying baseline settling. Direct seeding at $t = 0$ eliminates this delay: $\nu^{(z)}_j \leftarrow \max(\frac{1}{b}\sum z_{ij}^2, \varepsilon)$ produces a correctly-scaled seed at every batch size, including $b = 1$.

The procedure is specified in §4.2, Phase 3.

### 11.3 Clip-Width Modulation During Warm-Up · `sec:sentinel:algorithm-warmup-clip-width-modulation`

Clip-width modulation during warm-up is handled by the unified clip-pressure mechanism (§6.1.1) via the noise influence signal $\eta$. The effective clip ceiling uses $p = \max(\eta, \bar{\rho})$ where $\eta$ is the noise influence (§11.5) and $\bar{\rho}$ is the per-axis clip-pressure EWMA:

$$n_\sigma^{\text{eff}} = n_\sigma\left(1 + \frac{p}{1 - p + \varepsilon}\right)$$

During warm-up, $\eta \approx 1$ dominates and the ceiling is effectively open — preventing the positive feedback loop between tight clipping and low baselines that would otherwise extend convergence time by an order of magnitude. As $\eta$ decays with real observations, $\bar{\rho}$ takes over if the baselines become stale in production. The full mechanism, including dynamics tables and contamination analysis, is specified in §6.1.1 and §6.4.

### 11.4 Slow-From-Fast CUSUM Seeding · `sec:sentinel:algorithm-warmup-slow-from-fast-cusum-seeding`

After noise injection completes, the slow EWMA (§6.2) is **seeded from the fast EWMA's converged values** and the drift accumulator (§6.3) is **reset to zero**.

The slow EWMA's time constant ($\lambda_s$, e.g., 0.999 with half-life $\approx 693$ steps) is far longer than the fast EWMA's ($\lambda$, e.g., 0.99 with half-life $\approx 69$ steps). Without seeding, the fast baseline reaches steady state much sooner than the slow baseline. During this gap, the drift accumulator interprets the fast-slow difference as drift evidence, producing false accumulation.

Seeding the slow EWMA mean and variance from the fast EWMA at the noise-to-real transition eliminates this gap at its source. The procedure is:

1. Copy the fast EWMA's mean and variance to the slow EWMA's mean and variance.
2. Reset all four drift accumulators to zero.
3. Reset all four clip-pressure EWMAs $\bar{\rho}$ to zero (§6.1.1).

### 11.5 Maturity Tracking · `sec:sentinel:algorithm-maturity-tracking`

Each tracker records:

| Field                  | Description                                           |
| ---------------------- | ----------------------------------------------------- |
| Real observations      | Count of genuine observations processed               |
| Noise observations     | Count of synthetic observations processed             |
| Noise influence $\eta$ | Fraction of baseline not yet established by real data |

The noise influence decays exponentially with real observations:

$$\eta_{t+1} = \begin{cases} \lambda \, \eta_t + (1 - \lambda) & \text{noise observation} \\ \lambda \, \eta_t & \text{real observation} \end{cases}$$

After $n$ real observations: $\eta_n = \lambda^n$. The system reports $\eta$ without interpretation. The host should treat scores from trackers with high $\eta$ (e.g., $> 0.5$) as preliminary.

**Initial value.** $\eta_0 = 1.0$ at tracker creation, indicating that no real observations have yet established the baseline. After noise injection (which maintains $\eta \approx 1.0$) and $n$ real observations, $\eta_n \approx \lambda^n$.

#### 11.5.1 Three Convergence Concepts · `sec:sentinel:algorithm-maturity-tracking-convergence-concepts`

The system has three distinct notions of convergence, operating at different timescales:

| Convergence type                                        | Definition                                    | Timescale                                    | Notes                                                                       |
| ------------------------------------------------------- | --------------------------------------------- | -------------------------------------------- | --------------------------------------------------------------------------- |
| **$\eta$-convergence** (maturity)                       | $\eta_n = \lambda^n < \epsilon$               | $\lceil\ln\epsilon/\ln\lambda\rceil$ batches | Exact closed-form                                                           |
| **Trajectory convergence** (EWMA mean)                  | $\|\bar{\mu}_n - \bar{\mu}_\infty\| < \delta$ | Tens to hundreds of rounds                   | With clip-pressure modulation (§6.1.1, §6.4) and cold-start seeding (§11.2) |
| **Distributional stationarity** (baseline distribution) | Windowed-mean comparison $< \epsilon$         | Per-axis; varies by batch size               | Baselines wander even at true steady state                                  |

$\eta$-convergence is a **necessary but not sufficient** indicator of system readiness. EWMA baselines converge at a rate determined by cascaded EWMA interactions, clipping policy, batch size, and axis-specific score distributions — not by the simple exponential $\eta_n = \lambda^n$.

### 11.6 Deferred Cell Warm-Up · `sec:sentinel:algorithm-deferred-cell-warmup`

Noise injection is performed **outside** the main observation path. New trackers transition through a three-state lifecycle:

```
┌─────────┐         ┌─────────┐         ┌─────────┐
│ Created │────────►│ Warming │────────►│ Online  │
└─────────┘         └─────────┘         └─────────┘
                         │
                         │  asynchronous warm-up
                         │  volume-priority scheduling
                         ▼
                   noise injection
```

#### 11.6.1 Cell States · `sec:sentinel:algorithm-deferred-cell-warmup-states`

| State   | Receives real observations? | Contributes to reports? | Warm-up status  |
| ------- | --------------------------- | ----------------------- | --------------- |
| Created | No                          | No                      | Not yet started |
| Warming | No                          | No                      | In progress     |
| Online  | Yes                         | Yes                     | Complete        |

**Root warm-up exception.** The root tracker (§8.4) completes its warm-up synchronously during system construction, before the Sentinel accepts its first `ingest()` call. This ensures the "always online" invariant (§9.3) holds from the first observation. The root is the highest-priority cell under g.sum ordering and the only cell that exists at construction time; warming it synchronously is a one-time fixed cost that does not violate the work-variance bound (§12.9), which governs per-`ingest()` cost. All subsequently created cells follow the deferred warm-up lifecycle described below.

#### 11.6.2 Priority Scheduling · `sec:sentinel:algorithm-deferred-cell-warmup-priority-scheduling`

The warm-up pipeline always works on whichever member of the investment set's warming subset has the highest g.sum:

$$\text{priority}(c) = \text{g.sum}(c.\text{node})$$

where g.sum is the G-Tree node sum (§3.12 property 2) — the node's own accumulation plus all descendant sums.

This ordering has four consequences:

1. **Volume drives priority.** A higher g.sum node has more observation traffic flowing through its spatial region — bringing it online has the greatest impact on analysis quality.

2. **Ancestors emerge first.** By the summation invariant (§3.12 property 2), every ancestor's g.sum is at least as great as any descendant's. Sorting the warming set by g.sum produces an ordering where every ancestor appears before every descendant. This is a topological sort of the ancestor chains — obtained for free from a single sort, without encoding depth into the priority.

3. **Full chain at promotion.** Because ancestors come online before descendants, when a competitive target completes warm-up, its entire ancestor chain is already online. The multi-scale defence (§16) is complete from the first batch the target processes. No secondary warm-up gap exists.

4. **No starvation.** Every warming cell in an active region accumulates positive g.sum — the observations that justified its investment continue flowing through its spatial range. Cells that lose priority temporarily are eventually reached.

5. **Preemption is correct.** A cell that was highest-priority at one time may not be later — observation patterns shift, investment set membership changes. Working on the current highest-g.sum cell ensures the most valuable work is always done first. Partial warm-up is never wasted: the preempted cell's EWMA baselines retain their progress.

#### 11.6.3 Promotion · `sec:sentinel:algorithm-deferred-cell-warmup-promotion`

At the start of each observation cycle (§9.1, Step 0), before spatial layer observation routing:

```
procedure PromoteReadyCells():
    for each cell in the warming set where warm-up is complete:
        move cell from warming set to online set
```

This typically promotes zero or one cell per cycle. Promotion occurs atomically from the observation algorithm's perspective, before any observation routing. Coordination contexts involving promoted cells materialise lazily during the Step 5 coordination walk (§7.7.1), not at promotion time.

#### 11.6.4 Observation Routing During Warm-Up · `sec:sentinel:algorithm-deferred-cell-warmup-observation-routing`

Observations destined for a warming cell are routed to the nearest online ancestor, as specified in §9.3. The spatial layer's own volume tracking is unaffected: the observation accumulates importance at the warming cell's spatial node regardless of whether an analysis tracker exists there. The warming cell's competitive position in the V-Tree is maintained. It holds an investment slot in $\mathcal{I}$ and transitions to the producing set $\mathcal{A}$ upon promotion.

#### 11.6.5 Coordination During Warm-Up · `sec:sentinel:algorithm-deferred-cell-warmup-coordination`

Warming cells do **not** participate in coordination. Because coordination contexts are lazily materialised during the Step 5 coordination walk (§7.7.1), and only online competitive cells with scores contribute to the walk's Steiner tree, warming cells are structurally excluded — they produce no scores for the walk to discover. Their synthetic noise is never propagated through the coordination hierarchy, avoiding synthetic-on-synthetic artefacts during system formation.

#### 11.6.6 Eviction Before Completion · `sec:sentinel:algorithm-deferred-cell-warmup-precompletion-eviction`

A warming cell can be evicted from the spatial layer before warm-up completes — its spatial node may be absorbed by rebalancing or budget eviction. If this happens, the warm-up process discards the partially warmed cell. The work is lost, but correctly so: the spatial layer determined that the cell's interval no longer warrants a dedicated node.

Similarly, a warming cell can exit the investment set before warm-up completes — its competitive target may drop out of the top-$K$ (§8.5). The warm-up is cancelled and the tracker destroyed via the eager removal mechanism. The work is correctly discarded: the competitive landscape has determined the cell no longer warrants investment.

#### 11.6.7 Investment Slots vs. Production Slots · `sec:sentinel:algorithm-deferred-cell-warmup-investment-vs-production-slots`

Warming cells hold **investment slots** in $\mathcal{I}$ — they have allocated trackers and are enqueued for warm-up. They do not hold **production slots** in $\mathcal{A}$ — they do not produce scores, emit reports, or participate in coordination.

By the slot-vacancy invariant (§8.3), $|\mathcal{A}| = K - |\mathcal{T} \cap \text{Warming}|$: every warming competitive target corresponds to a vacant production slot. At promotion, the cell fills its own vacancy — no online cell is displaced. The $|\mathcal{A}| \leq K$ bound is maintained structurally by Step 3's top-$K$ recomputation (§8.1), not by promotion-time ejection.

The warm-up pipeline's g.sum ordering (§11.6.2) ensures that ancestors come online before descendants. By the time a competitive target is promoted, its entire ancestor chain is online and producing scores. The system provides progressively deeper coverage as warm-up progresses, rather than a single transition from no coverage to full coverage.

#### 11.6.8 Pipeline Advancement · `sec:sentinel:algorithm-deferred-cell-warmup-pipeline-advancement`

The warm-up pipeline advances at a granularity of **one noise batch per scheduling quantum**. After each quantum, the pipeline re-evaluates the g.sum priority ordering and continues with the highest-priority warming cell. This re-evaluation enables preemption: a newly enqueued cell with higher g.sum displaces an in-progress cell after at most one noise batch of delay.

The pipeline advances **independently of observation cadence** — warm-up throughput is not gated by the rate of `ingest()` calls. Promotion to Online status remains gated by the observation algorithm: it occurs at Step 0 of the next `ingest()` call after warm-up completes (§9.1).

**Mutual-exclusion invariant.** The warm-up pipeline and the observation algorithm never operate on the same tracker concurrently. Newly created trackers are fully initialised before becoming visible to the pipeline. Tracker destruction (§8.5) waits for (or cancels) any in-progress noise batch before proceeding. The implementation is free to achieve this invariant through any mechanism — a dedicated thread with synchronised handoff, a cooperative executor, or synchronous drain — provided the four properties above hold.

### 11.7 Coordination-Specific Warm-Up · `sec:sentinel:algorithm-coordination-specific-warmup`

When a coordination context materialises during the Step 5 coordination walk (§7.7.1) — for example, when two previously unrelated competitive cells first share an ancestor — the context's coordination tracker requires warm-up before processing its first real observation. This warm-up runs **inline to completion** within the same Step 5 invocation, immediately after context creation and before the walk feeds the first real group matrix.

#### 11.7.1 Scheduling Model · `sec:sentinel:algorithm-coordination-warmup-scheduling-model`

Coordination warm-up is **inline at materialisation**, not deferred through a parallel warming pipeline. This contrasts with per-cell warm-up (§11.6), which is deferred with g.sum priority scheduling. The difference is justified by cost:

- **Per-cell warm-up** operates at analysis widths $w$ up to $N = 128$, requiring hundreds of rounds of full SVD updates. The cost motivates deferral and priority scheduling.
- **Coordination warm-up** operates at $w = 4$ with ~50 rounds of $O(16m)$ work — microseconds on any real hardware, less than 0.3% of a single per-cell SVD (§7.8). The cost does not justify a second scheduling tier.

The "no inline warm-up work" principle stated in §9.2 for Step 3 applies to per-cell tracker warm-up. Coordination warm-up is a separate concern, handled in Step 5, where the trivial cost makes inline execution the simpler and more correct choice.

#### 11.7.2 Procedure · `sec:sentinel:algorithm-coordination-warmup-procedure`

1. Snapshot each contributing cell's current baseline mean $\bar{s}_c$ and variance $\bar{v}_c$ for all four scoring axes. The snapshot is taken at Step 5 time — after Step 4 scoring has updated the contributing cells' baselines with this cycle's observations.

2. For each synthetic round: for each cell $c$ in the coordination group, generate a synthetic score vector. For each scoring axis, sample from a non-negative distribution matching $c$'s snapshot baselines. A Gamma distribution with shape $\alpha_c = \bar{s}_c^2 / \bar{v}_c$ and rate $\beta_c = \bar{s}_c / \bar{v}_c$ is the natural choice: it preserves the learned baseline statistics while respecting non-negativity.

3. Assemble the synthetic score vectors into a group matrix, centre it (§7.3), and feed it through the coordination tracker (§7.5).

4. Repeat for the number of rounds prescribed by the noise schedule (§11.1.3) at the coordination context's G-Tree depth — that is, the depth of the internal G-Tree node that hosts this context, using the same depth parameter $d$ that governs per-cell warm-up.

5. Reset the coordination tracker's drift accumulators to zero.

If a cell's baseline is uninitialised (no valid mean or variance), use per-axis defaults: mean $= 0.25$, variance $= 0.01$ for novelty and surprise; mean $= 0.1$, variance $= 0.01$ for displacement and coherence.

> _Design note (why a non-negative distribution)._ All four scoring axes produce non-negative values. A symmetric distribution centred at a small positive mean produces negative samples, which are meaningless in this context. The Gamma distribution with matched moments is the simplest non-negative distribution that preserves the learned baseline statistics.

> _Design note (snapshot timing)._ The baseline snapshot is taken after Step 4 scoring, meaning baselines reflect this cycle's observations. The alternative — snapshotting before Step 4 — would require coordination lifecycle management in Step 0 or Step 3, adding bookkeeping complexity for a negligible difference (one batch of EWMA update at rate $\alpha \approx 0.01$). The post-Step-4 snapshot is arguably preferable: the baselines are more current.

> _Design note (depth taper for coordination)._ The noise schedule's depth-based tapering was designed for per-cell trackers, where deeper G-Tree nodes have narrower analysis widths and thus faster baseline convergence. Coordination trackers all operate at $w = 4$ regardless of their G-Tree depth, so the convergence time is depth-independent — the taper rationale does not transfer. Under the default schedule, every coordination context receives the schedule minimum (50 rounds), since even the shallowest possible context at depth 1 converges in well under 225 rounds at $w = 4$. The shared schedule is retained for simplicity; implementations may use a flat 50-round schedule for coordination warm-up without observable difference.

### 11.8 System-Level Warm-Up Stages · `sec:sentinel:algorithm-system-warmup-stages`

During early operation, the system-level behaviour differs from steady state:

**Stage 1: Pre-Split.** Below the split threshold ($< \theta$ total observations). A single spatial node covers the entire domain. The root tracker at $w = N$ is always present (permanent). No competitive targets yet; the investment set contains only the root. No coordination possible. All scores reflect global structure only.

**Stage 2: Spatial Formation.** From $\theta$ to approximately $10\theta$ observations. The spatial tree begins splitting. Cells enter and exit the competitive targets frequently as the V-Tree ranking stabilises. Investment set membership churns as targets shift. The producing set grows progressively as warm-up completes for each invested cell — ancestors first, then competitive targets. Scores from early-promoted ancestors are available before any competitive cell comes online.

**Stage 3: Stabilisation.** From approximately $10\theta$ to $100\theta$ observations. The competitive targets settle. The investment set stabilises and the producing set converges toward $\mathcal{I}$ as remaining warming cells complete their warm-up. Trackers mature ($\eta$ declining). Ancestor chains lengthen as the spatial tree deepens. Coordination contexts materialise as competitive target pairs come online (§7.7.1). Drift accumulators begin accumulating meaningful evidence.

**Stage 4: Steady State.** Trackers have $\eta \ll 1$. Drift accumulators reflect genuine baseline departures. Ancestor chains provide multi-scale defence. The system detects anomalies at its designed sensitivity.

The transition timescale depends on observation concentration, split threshold $\theta$, and forgetting factor $\lambda$.

### 11.9 Empirical Baseline Convergence · `sec:sentinel:algorithm-empirical-baseline-convergence`

The following table gives measured EWMA trajectory convergence times — the number of batches until the windowed-mean baseline is within tolerance of its steady-state value. These were obtained with clip-pressure modulation (§6.1.1), cold-start latent seeding (§11.2), and slow-from-fast drift-accumulator seeding (§11.4) all active.

| Component                | $\lambda{=}0.95$, $b_{\text{noise}}{=}4$ | $\lambda{=}0.95$, $b_{\text{noise}}{=}16$ | $\lambda{=}0.99$, $b_{\text{noise}}{=}16$ |
| ------------------------ | ---------------------------------------: | ----------------------------------------: | ----------------------------------------: |
| $\eta$ (noise influence) |                               59 (exact) |                                59 (exact) |                               299 (exact) |
| Latent mean/variance     |                        $\sim 1$ (seeded) |                         $\sim 1$ (seeded) |                         $\sim 1$ (seeded) |
| Novelty baseline         |                                       21 |                                        21 |                                       101 |
| Displacement baseline    |                         21–475 (bimodal) |                                        21 |                                       101 |
| Surprise baseline        |                                   **65** |                                    **70** |                                   **398** |
| Coherence baseline       |                                  **406** |                                   **173** |                                   **315** |
| **System (worst-case)**  |                            **$\sim$406** |                             **$\sim$173** |                             **$\sim$398** |

Per-axis empirical coefficients of variation at steady state (windowed-mean):

| Axis         | CV ($b_{\text{noise}}{=}4$) | CV ($b_{\text{noise}}{=}16$) | Notes                                                     |
| ------------ | :-------------------------: | :--------------------------: | --------------------------------------------------------- |
| Novelty      |            0.13%            |            0.06%             | Constant-norm property — inherently stable                |
| Displacement |            5.9%             |             3.0%             | Bimodal across random seeds at $b_{\text{noise}}{=}4$     |
| Surprise     |            9.0%             |             3.6%             | Improved substantially by cold-start seeding              |
| Coherence    |            19.9%            |            11.5%             | Rank-gating delay ($k < 2$) is the convergence bottleneck |

**The noise schedule must produce enough rounds for the worst-case convergence time** at each depth for baselines to be settled before real observations arrive. At $\lambda = 0.99$, $b_{\text{noise}} = 16$ (a typical production configuration), this is $\geq 400$ batches at depth 0 (root). Deeper cells need fewer rounds — the geometric taper reflects the faster convergence at narrower analysis widths.

---

## Chapter 12. Resource Bounds, Spray Resistance, and Complexity · `sec:sentinel:algorithm-resource-bounds-spray-resistance-and-complexity`

### 12.1 Three Independent Ceilings · `sec:sentinel:algorithm-resource-independent-ceilings`

| Ceiling    | Parameter                 | Bounds               | Layer             |
| ---------- | ------------------------- | -------------------- | ----------------- |
| $G_{\max}$ | Hard spatial node ceiling | Total spatial nodes  | Spatial Layer     |
| $K$        | Analysis budget           | Competitive trackers | Analysis Selector |
| $\|\mathcal{I}\|$ | (derived)                 | $\leq 1 + K\bar{D}$ before sharing; dominated by $2K$ in practice (§8.2) | Analysis Selector |
| $\|\mathcal{A}^\*\|$ | (derived)                 | $\leq \|\mathcal{I}\|$; equals $\|\mathcal{I}\|$ in steady state | Analysis Selector |

The budget $K$ governs competitive selection; ancestor trackers are not counted against the budget. Total invested tracker count $|\mathcal{I}|$ is bounded by $1 + K\bar{D}$ before sharing (§8.2), where $\bar{D}$ is the average G-Tree depth of competitive targets. Because the competitive mechanism and depth gate keep trees shallow (§3.1), $\bar{D}$ is typically 2–6, and ancestor sharing under concentration brings the practical size close to $2K$. Peak memory is determined by $|\mathcal{I}|$ (including warming trackers); per-batch computation is determined by $|\mathcal{A}^*|$ (online trackers only). Total memory: $G_{\max} \times (\text{per-node size}) + |\mathcal{I}| \times (\text{max per-tracker size})$. Both terms are hard-bounded.

### 12.2 Spray Resistance · `sec:sentinel:algorithm-resource-spray-resistance`

A **spray** is a high-entropy input pattern: an adversary (or a degenerate data source) emitting maximally diverse values, forcing the system to allocate structure across the full domain rather than concentrating precision on structured regions. The spray is an entropy attack on the spatial partition budget — it attempts to exhaust the spatial layer's node ceiling through diversity rather than volume.

**Spatial budget.** The spatial layer's budget invariant $|G| + S + 2 \leq G_{\max}$ (§3.12, property 8) bounds node creation absolutely. Its spray defence mechanisms are inherited in full.

**Analysis budget.** New cells start at ground importance and sit deep in the V-Tree — far below the analysis cutoff $L$. Only sustained observation volume pushes a cell into the top $K$. **Value diversity creates spatial nodes but not competitive trackers.** The competitive budget $K$ is inherently robust against diversity-based spray. Ancestor trackers are bounded by the investment set's ancestor closure (§8.2): at most $K\bar{D}$ additional ancestor nodes before sharing (reduced to near $K$ under typical concentration), regardless of how many spatial nodes exist.

### 12.3 Steady-State Bound · `sec:sentinel:algorithm-resource-steady-state-bound`

Under sustained spray at rate $R$ with spatial attenuation $\lambda_{\text{sp}} \in (0, 1)$ and split threshold $\theta$:

$$L_{\text{steady}} = \frac{R}{(1 - \lambda_{\text{sp}}) \cdot \theta}$$

Independent of domain size and spray duration. At most $K$ of these receive trackers. For $\lambda_{\text{sp}} \geq 1$ (amplification or no-decay regimes), no decay-driven equilibrium exists; the spatial hard ceiling $G_{\max}$ (§3.12, property 8) is the operative bound.

### 12.4 Benchmark Compounding · `sec:sentinel:algorithm-resource-benchmark-compounding`

The spatial layer's benchmark compounding (§3.8) provides long-term spray memory. After $j$ expand–contract cycles, re-expansion to depth $D$ requires $\sim D^2 \theta / 2$ observations. This ensures that deep spatial structure is re-created only when justified by sustained, concentrated observation volume.

### 12.5 Coverage Displacement · `sec:sentinel:algorithm-resource-coverage-displacement`

An adversary can influence _which_ cells occupy the $K$ competitive slots by generating concentrated observations to chosen prefixes, displacing established cells. This is a coverage attack, not a resource attack. The host can detect it via the analysis set summary in the report (§14).

### 12.6 Per-Tracker Complexity · `sec:sentinel:algorithm-resource-per-tracker-complexity`

| Operation                     | Cost                                                     | Notes             |
| ----------------------------- | -------------------------------------------------------- | ----------------- |
| Projection and reconstruction | $O(bwk)$                                                 | Matrix multiply   |
| Novelty                       | $O(bw)$                                                  | Residual norm     |
| Displacement and surprise     | $O(bk)$                                                  | Latent space      |
| Coherence                     | $O(bk^2)$                                                | Pairwise products |
| Streaming SVD                 | $O\!\big(\min(w,\, k{+}b)^2 \cdot \max(w,\, k{+}b)\big)$ | **Dominant**      |
| Second-moment update          | $O(bk^2)$                                                | Upper triangle    |
| Baselines and drift detection | $O(1)$ per axis                                          |                   |

The SVD input $M \in \mathbb{R}^{w \times (k+b)}$ is a thin SVD whose cost depends on the aspect ratio. When $w \geq k + b$ (tall matrix — typical at the root and shallow cells), the cost is $O(w(k{+}b)^2)$. When $k + b > w$ (wide matrix — routine at deep competitive cells, e.g. $w = 32$, $k + b = 80$), the cost is $O(w^2(k{+}b))$. The general form covers both regimes.

### 12.7 Per-Batch Complexity · `sec:sentinel:algorithm-resource-per-batch-complexity`

**Step 2 (spatial layer).** Per observation: $O(d_{\text{geo}} + h_V)$. Per batch: $O(n(d_{\text{geo}} + h_V))$ plus amortised structural operations.

**Step 4 (per-cell scoring, all online trackers in $\mathcal{A}^*$).** Each tracker's cost depends on its per-tracker batch size $b$, which varies by cell (§2.6 batch-size note). Only trackers with $b > 0$ execute the core loop; the effective cost is $\sum_{c \in \mathcal{A}^* :\, b_c > 0} O\!\big(\min(w_c,\, k_c + b_c)^2 \cdot \max(w_c,\, k_c + b_c)\big)$, dominated by SVD. The worst-case bound, assuming all trackers receive observations, is $O\!\big(|\mathcal{A}^*| \cdot \min(w_{\max},\, k + b_{\max})^2 \cdot \max(w_{\max},\, k + b_{\max})\big)$. In steady state $|\mathcal{A}^*| = |\mathcal{I}| \leq 1 + K\bar{D}$ before sharing (§8.2); under realistic observation distributions with concentration-driven sharing, the effective size approaches $2K$.

**Ancestor chain cost.** Each observation is processed by every tracker on its ancestor path. The dominant term is the **root tracker**: it has the widest suffix ($w = N$) and sees the full ingestion batch ($b = n$). Its SVD input is $\mathbb{R}^{N \times (k_0 + n)}$. For $N = 128$, $k_0 = 16$, and $n = 64$, this is a $(128, 80)$ matrix — in the tall regime ($w > k + b$), costing $O(128 \times 80^2) \approx 819\text{K}$. At deeper competitive cells the SVD input is in the wide regime ($k + b > w$); for example, a depth-96 cell has $w = 32$, and with $k = 16$, $b = 64$, the $(32, 80)$ matrix costs $O(32^2 \times 80) \approx 82\text{K}$ — not $O(32 \times 80^2) \approx 205\text{K}$. Callers ingesting large batches ($n > N - k$) push even the root into the wide regime.

Intermediate ancestor levels' costs are smaller (narrower suffixes, smaller batches — only observations routing through their spatial range) and partially amortised by sharing. The per-observation fan-out equals the target cell's G-Tree depth $d$ plus one (the cell itself). At typical operational depths ($d \approx 2\text{–}6$ under default parameters), this is a modest multiplier. At deeper operational depths, the cost grows linearly with $d$ but is bounded by $O(\log_2 |\text{domain}|)$ — each ancestor doubles the spatial coverage, so $d$ steps span the full dynamic range from a single cell to the entire domain. The total ancestor chain cost across all competitive targets is further reduced by sharing: the Steiner tree structure of the ancestor closure (§8.2) ensures that the aggregate unique ancestor count is far less than $K \times d$ — approaching $K$ under typical spatial concentration.

**Step 5 (coordination).** Per context: $O(m)$. Total useful work: $O(K \cdot \bar{d})$ where $\bar{d}$ is average coordination participation depth. The pseudocode (§7.4) visits the full spatial tree for clarity; an implementation walking the investment set's reduced Steiner tree (§8.2) achieves this bound with $O(|\mathcal{I}|)$ traversal overhead (approaching $O(K)$ under typical spatial clustering). Negligible relative to Step 4.

**Overall.** The observation algorithm is dominated by Step 4. Typically dominated by Step 4.

### 12.8 Matrix Dimensions · `sec:sentinel:algorithm-resource-matrix-dimensions`

**Per competitive tracker:**

| Matrix          | Shape             |
| --------------- | ----------------- |
| $X$             | $(b, w)$          |
| $U$             | $(w, \text{cap})$ |
| $Z$             | $(b, k)$          |
| $M$ (SVD input) | $(w, k+b)$        |

Typical competitive: $w \in \{32, \ldots, 112\}$, $k \leq 16$, $b \leq 64$. Largest competitive SVD: approximately $(112, 80)$. Note: $b$ here is the per-tracker count of observations routing to the cell (§2.6 batch-size note); at deep competitive cells, $b$ may be substantially smaller than the ingestion batch size $n$.

**Root tracker:** $w = N$, $b = n$ (full ingestion batch). SVD input $(N, k + n)$. For $N = 128$, $k = 16$, $n = 64$: $(128, 80)$.

**Per coordination context:**

| Matrix    | Shape          |
| --------- | -------------- |
| $O$       | $(m, 4)$       |
| SVD input | $(4, k_m + m)$ |

The coordination SVD is at most $(4, K + 4)$ — trivial.

### 12.9 Work Variance · `sec:sentinel:algorithm-resource-work-variance`

With deferred warm-up (§11.6), the per-call cost of the observation algorithm is bounded by:

$$O\!\Big(n(d_{\text{geo}} + h_V) \;+\; |\mathcal{A}^*| \cdot \min(w_{\max},\, k + b)^2 \cdot \max(w_{\max},\, k + b)\Big)$$

on **every** call. (The previous expression $|\mathcal{A}^*| \cdot w_{\max} \cdot (k + b)^2$ is a valid but loose upper bound; the $\min/\max$ form is tight in both the tall-matrix and wide-matrix regimes — see §12.6.) Cell creation and noise injection contribute zero cost to any observation call. The only call-to-call variance arises from:

- **Batch size variation** ($b$ differs per cell and per call; $n$ differs between calls).
- **Producing set size variation** ($|\mathcal{A}^*|$ changes by $O(1)$ per call as cells promote or exit via investment set reconciliation).
- **Rank variation** ($k$ changes by at most 1 per rank update interval).

All three change slowly relative to call frequency.

> _Design note._ The deferred warm-up removes the dominant structural source of timing variance (inline noise injection), making the observation algorithm operationally predictable. It does **not** make it constant-time — the remaining variance, though small, is observable to a sufficiently precise adversary. Constant-time guarantees, if required, are an implementation concern beyond this specification.

---

# Part V — External Interface · `sec:sentinel:algorithm-external-interface`

---

## Chapter 13. Configuration · `sec:sentinel:algorithm-configuration`

All parameters are organised by the layer they govern. An implementation should accept these at construction time. Parameters marked as host-controlled may additionally be modified during operation through the mechanisms noted.

### 13.1 Analysis Engine Parameters · `sec:sentinel:algorithm-configuration-analysis-engine-parameters`

These govern the subspace tracker (§4) and baseline tracking (§6) within each analysed cell.

| Parameter                                 | Description                                                              | Default   | Constraint     | Guidance                                                                                                                                                                                                                                  |
| ----------------------------------------- | ------------------------------------------------------------------------ | --------- | -------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Maximum rank ($r_{\max}$)                 | Hard ceiling on subspace dimensionality                                  | 16        | $\geq 1$       | Higher captures more complex structure; memory scales as $w \times \text{cap}$.                                                                                                                                                           |
| Forgetting factor ($\lambda$)             | EWMA decay rate                                                          | 0.99      | $(0, 1)$       | Lower = faster adaptation, shorter memory. Match to the regime change timescale.                                                                                                                                                          |
| Rank update interval ($T_{\text{rank}}$)  | Batches between rank adaptation (§4.2, Phase 5)                          | 100       | $\geq 1$       | Lower = more responsive rank. Higher = more stable.                                                                                                                                                                                       |
| Energy threshold ($\tau$)                 | Cumulative variance target for rank selection                            | 0.90      | $(0, 1)$       | 0.90 retains 90% of variance. Higher → rank grows, novelty range shrinks.                                                                                                                                                                 |
| Stability constant ($\varepsilon$)        | Floor for numerical stability                                            | $10^{-6}$ | $> 0$          | Rarely needs adjustment.                                                                                                                                                                                                                  |
| Clip width ($n_\sigma$)                   | Base outlier clip width in $\sigma$ units (§6.1.1)                       | 3.0       | $> 0$          | Wider → fewer legitimate rejections, slower poisoning resistance. Narrower → more false rejections.                                                                                                                                       |
| Slow decay ($\lambda_s$)                  | Per-tracker slow EWMA decay (§6.2)                                       | 0.999     | $(\lambda, 1)$ | Ratio $\lambda_s / \lambda$ controls drift detection sensitivity window.                                                                                                                                                                  |
| Coordination slow decay ($\lambda_{s,m}$) | Coordination slow EWMA decay (§7.5)                                      | 0.999     | $(\lambda, 1)$ | Same guidance as per-tracker slow decay.                                                                                                                                                                                                  |
| Drift allowance ($\kappa_\sigma$)         | Drift accumulator noise allowance in slow-baseline $\sigma$ units (§6.3) | 0.5       | $\geq 0$       | Lower → more sensitive, more false positives.                                                                                                                                                                                             |
| Clip-pressure decay ($\lambda_\rho$)      | EWMA decay for per-axis clip ratio tracking (§6.1.1, §6.4)               | 0.95      | $(0, 1)$       | Lower → faster recovery from stale baselines, more contamination leakage. Higher → slower recovery, better poisoning resistance. At 0.95 (half-life ~14 batches): moderate shifts converge in ~60–100 batches through damped oscillation. |
| Per-sample scores                         | Whether to include per-observation score vectors in reports              | false     | boolean        | Enable for forensic analysis; increases report size.                                                                                                                                                                                      |

### 13.2 Analysis Selector Parameters · `sec:sentinel:algorithm-configuration-analysis-selector-parameters`

These govern which cells receive statistical modelling (§8).

| Parameter             | Description                                   | Default | Constraint | Guidance                                                                                                                                                                           |
| --------------------- | --------------------------------------------- | ------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Analysis budget ($K$) | Maximum competitively selected cells          | 1024    | $\geq 1$   | Total invested trackers $\leq 1 + K\bar{D}$ before sharing (§8.2); dominated by $2K$ under concentration. Scale with observation diversity. 256 for focused; 4096 for large-scale. |
| Depth cutoff ($L$)    | Maximum V-Tree depth for analysis eligibility | 6       | $\geq 0$   | Larger $L$ admits less significant cells. Keep near $D_{\text{create}} + 2$.                                                                                                       |

The competitive targets $\mathcal{T}$, investment set $\mathcal{I}$, and producing sets $\mathcal{A}$, $\mathcal{A}^*$ are derived from these parameters as specified in §8.

### 13.3 Spatial Layer Parameters · `sec:sentinel:algorithm-configuration-spatial-layer-parameters`

These are passed through to the spatial layer (§3) at construction time. The Sentinel does not modify them during operation.

| Parameter                           | Description                              | Default | Constraint | Guidance                                                                                                                                       |
| ----------------------------------- | ---------------------------------------- | ------- | ---------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| Split threshold ($\theta$)          | Minimum importance for split eligibility | 100     | $> 0$      | Lower → finer resolution faster. Must be reachable by sustained observations in the desired response time. Type is the accumulator $V$ (§1.2). |
| Creation gate ($D_{\text{create}}$) | Maximum V-Tree depth for splits          | 3       | $\geq 0$   | Application-dependent.                                                                                                                         |
| Eviction gate ($D_{\text{evict}}$)  | Minimum V-Tree depth for eviction        | 6       | $\geq 1$   | Must exceed $D_{\text{create}}$ by a buffer.                                                                                                   |
| Soft budget                         | Soft node-count target                   | 100,000 | $> 0$      | Application-dependent.                                                                                                                         |
| Hard ceiling ($G_{\max}$)           | Absolute maximum spatial nodes           | 200,000 | $\geq 5$   | Must exceed soft budget.                                                                                                                       |

### 13.4 Temporal Policy (Host-Controlled) · `sec:sentinel:algorithm-configuration-host-controlled-temporal-policy`

These are not construction-time parameters but rather arguments to the spatial decay operation, invoked by the host at its discretion (§10.1).

| Parameter          | Suggested value | Guidance                                                                                                   |
| ------------------ | --------------- | ---------------------------------------------------------------------------------------------------------- |
| Attenuation factor | 0.99            | Attenuation per call. Compound with interval: effective half-life $= t_{1/2} \times \text{interval}$.      |
| Depth selectivity  | 0.0             | Values $> 0$ cause fine structure to decay faster than coarse. Use 0.3–0.5 for depth-selective forgetting. |
| Decay interval     | 60 seconds      | Shorter → more responsive contour. Longer → more stable.                                                   |

### 13.5 Warm-Up Parameters · `sec:sentinel:algorithm-configuration-warmup-parameters`

These govern the noise injection and warm-up procedure (§11).

| Parameter        | Description                        | Default                                          | Constraint                                   |
| ---------------- | ---------------------------------- | ------------------------------------------------ | -------------------------------------------- |
| Noise schedule   | Depth-tiered synthetic batch count | Geometric: root = 450, decay = 0.5, minimum = 50 | See §11.1.3                                  |
| Noise batch size | Samples per synthetic batch        | 16                                               | $\geq 1$                                     |
| Random seed      | Seed for pseudo-random generator   | Deterministic (fixed)                            | Optional; absent means implementation-chosen |

---

## Chapter 14. Output · `sec:sentinel:algorithm-output`

### 14.1 Report Overview · `sec:sentinel:algorithm-output-report-overview`

Each observation cycle (§9) produces a report containing four sections: per-cell reports from competitive trackers, per-cell reports from ancestor trackers, coordination reports from the hierarchical coordination tier, and summary information about the system's current state. The report also includes contour, health, and analysis set summaries.

Importance values in the report are converted to floating-point approximations at the report boundary (via the approximate conversion capability required of $V$, §1.2). Hosts needing exact accumulator values should query the spatial layer directly.

The report also states the age of its own evidence, described in full below (`sec:sentinel:algorithm-output-observation-age`).

#### 14.1.1 Observation Age · `sec:sentinel:algorithm-output-observation-age`

Every report carries one further figure: how old the oldest observation in the batch was at the moment the report was emitted, as a duration in microseconds.

| Field                 | Type                                | Description                                                                     |
| --------------------- | ----------------------------------- | ------------------------------------------------------------------------------- |
| Oldest observation age | optional non-negative integer, µs  | Age of the batch's oldest observation at report emission; absent where there is none |

Four properties fix what the figure means.

**It is a duration and never an instant.** Both ends of the interval are read from the sentinel's own monotonic clock: one reading as the batch arrives, one as the report is assembled. No wall-clock time is recorded anywhere in the report and no clock of the sentinel's is ever compared with another machine's, so there is no skew for the figure to carry and none for a consumer to correct.

**One figure bounds the whole batch.** A batch arrives whole, so its observations share a single arrival and the oldest of them is no older than that arrival. The reported age is therefore the maximum over the batch: no observation in it is older than the figure, and the oldest is exactly that old.

**Absence means there is no age, not an age of nothing.** A batch carrying no observations has no oldest observation, and the field is absent rather than zero. A payload written before the field existed is absent in the same way and for the same reason — nothing measured it. The two are spelled alike because they say the same thing to a consumer, and a zero would say something different and untrue: that the evidence was fresh when the report went out.

**The host's forwarding delay is an unmeasured residual.** How long the host held the observations before handing them over is real, is not in this figure, and is deliberately not measured. Measuring it would require comparing the host's clock with the sentinel's, which is exactly the cross-machine comparison this design does without; a consumer reading the age therefore reads a lower bound on how old the evidence is, not the whole of it.

### 14.2 Cell Analysis Report · `sec:sentinel:algorithm-output-cell-analysis-report`

One record per analysed cell (competitive or ancestor) that processed at least one observation in the batch.

| Field              | Type                            | Description                                              |
| ------------------ | ------------------------------- | -------------------------------------------------------- |
| Interval           | pair of $C$ values              | Spatial bounds of the cell (half-open interval)          |
| Depth              | non-negative integer            | G-Tree depth (0 = root)                                  |
| Analysis width     | non-negative integer            | $w = N - \text{depth}$                                   |
| Sample count       | non-negative integer            | Observations delivered to this cell in this batch        |
| Rank               | non-negative integer            | Current active subspace dimensionality $k$               |
| Energy ratio       | float                           | Fraction of total energy captured by the active subspace |
| Top singular value | float                           | Largest singular value $\sigma_1$                        |
| Maturity           | maturity record (§14.6)         | Noise influence and observation counts                   |
| Scoring geometry   | geometry record (§14.7)         | Structural reliability of each scoring axis              |
| Scores             | scoring record (§14.3)          | All four axes with baselines and drift state             |
| Per-sample scores  | optional list of sample records | Present only when per-sample reporting is enabled        |

**Competitive vs. ancestor reports.** Both use the same record structure. They are separated in the report so the host can distinguish cells selected as competitive targets (members of $\mathcal{A}$) from cells included as ancestors (members of $\mathcal{A}^* \setminus \mathcal{A}$). Only online (producing) cells appear in the report; warming cells in $\mathcal{I} \setminus \mathcal{A}^*$ do not emit reports.

### 14.3 Scoring Record · `sec:sentinel:algorithm-output-scoring-record`

One per cell, containing all four axes.

| Field        | Type                       | Description |
| ------------ | -------------------------- | ----------- |
| Novelty      | score distribution (§14.4) |             |
| Displacement | score distribution (§14.4) |             |
| Surprise     | score distribution (§14.4) |             |
| Coherence    | score distribution (§14.4) |             |

### 14.4 Score Distribution · `sec:sentinel:algorithm-output-score-distribution`

One per axis per cell, summarising the batch and its relationship to baselines.

| Field           | Type                      | Description                                                          |
| --------------- | ------------------------- | -------------------------------------------------------------------- |
| Minimum         | float                     | Minimum raw score across samples in this batch                       |
| Maximum         | float                     | Maximum raw score across samples in this batch                       |
| Mean            | float                     | Mean raw score across samples in this batch                          |
| Maximum z-score | float                     | $\zeta(\max_i s_i)$ — z-score of the loudest sample (§6.1.2)         |
| Mean z-score    | float                     | $\zeta(\bar{s}_{\text{batch}})$ — z-score of the batch mean (§6.1.2) |
| Clip pressure   | float in $[0, 1]$         | Current clip-pressure EWMA $\bar{\rho}$ for this axis (§6.1.1)       |
| Baseline        | baseline snapshot (§14.5) | Current fast EWMA state                                              |
| Drift state     | drift snapshot (§14.5)    | Current slow EWMA and drift accumulator state                        |

### 14.5 Baseline and Drift Snapshots · `sec:sentinel:algorithm-output-baseline-and-drift-snapshots`

**Baseline snapshot** (fast EWMA state):

| Field    | Type  | Description                          |
| -------- | ----- | ------------------------------------ |
| Mean     | float | Current fast EWMA mean $\bar{s}$     |
| Variance | float | Current fast EWMA variance $\bar{v}$ |

**Drift snapshot** (slow EWMA and drift accumulator state):

| Field                  | Type                 | Description                                |
| ---------------------- | -------------------- | ------------------------------------------ |
| Accumulator            | float                | Current drift accumulator value $S$ (§6.3) |
| Slow baseline mean     | float                | Current slow EWMA mean                     |
| Slow baseline variance | float                | Current slow EWMA variance                 |
| Steps since reset      | non-negative integer | Batches since last drift accumulator reset |

> _Host guidance (secular-trend monitoring)._ The drift accumulator $S$ detects fast-vs-slow baseline divergence but is blind to slow monotonic inflation of both baselines (see §6.4.5 design note). Hosts concerned about long-horizon contamination should monitor secular trends in the slow baseline mean across trackers over timescales much longer than $1/(1-\lambda_s)$. Sustained upward drift in $\bar{s}_{\text{slow}}$ across multiple trackers and axes, especially when accompanied by persistently elevated $\bar{\rho}$ (§14.4), may indicate patient sub-clip contamination. The annihilation mechanism (§10) provides a remediation path: targeted state destruction forces re-learning from the current observation distribution.

### 14.6 Maturity Record · `sec:sentinel:algorithm-output-maturity-record`

| Field              | Type                 | Description                                                            |
| ------------------ | -------------------- | ---------------------------------------------------------------------- |
| Real observations  | non-negative integer | Genuine observations processed since creation                          |
| Noise observations | non-negative integer | Synthetic observations processed during warm-up                        |
| Noise influence    | float in $[0, 1]$    | $\eta$ — fraction of baseline not yet established by real data (§11.5) |

### 14.7 Scoring Geometry Record · `sec:sentinel:algorithm-output-scoring-geometry-record`

One per cell or coordination tracker, describing the structural reliability of each scoring axis under the tracker's current rank and dimensionality.

| Field          | Type                 | Description                                              |
| -------------- | -------------------- | -------------------------------------------------------- |
| Analysis width | non-negative integer | Working dimensionality $w$ of this tracker's input space |
| Capacity       | non-negative integer | Maximum reachable rank: $\text{cap} = \min(w, r_{\max})$ |
| Residual DOF   | non-negative integer | Residual degrees of freedom: $w - k$                     |

Two derived predicates assist the host:

| Predicate         | Condition                 | Meaning                                                                     |
| ----------------- | ------------------------- | --------------------------------------------------------------------------- |
| Novelty-saturated | $\text{residual DOF} = 0$ | Novelty axis is identically zero — the subspace spans the full space (§5.2) |
| Novelty-saturable | $\text{cap} \geq w$       | Novelty _can_ become saturated as rank adapts — the host should monitor     |

### 14.8 Per-Sample Score Record · `sec:sentinel:algorithm-output-per-sample-score-record`

Present only when per-sample reporting is enabled. One per observation delivered to the cell.

| Field        | Type  | Description                            |
| ------------ | ----- | -------------------------------------- |
| Novelty      | float | Raw novelty score for this sample      |
| Displacement | float | Raw displacement score for this sample |
| Surprise     | float | Raw surprise score for this sample     |
| Coherence    | float | Raw coherence score for this sample    |

### 14.9 Coordination Report · `sec:sentinel:algorithm-output-coordination-report`

One record per active coordination context (§7.1) that fired in this batch, ordered by G-Tree depth (shallowest first).

| Field              | Type                            | Description                                                 |
| ------------------ | ------------------------------- | ----------------------------------------------------------- |
| Context interval   | pair of $C$ values              | Spatial extent of the coordination group                    |
| Context depth      | non-negative integer            | G-Tree depth of the coordination node                       |
| Group size         | non-negative integer            | Total competitive cells in this group                       |
| Left count         | non-negative integer            | Competitive cells in the left subtree                       |
| Right count        | non-negative integer            | Competitive cells in the right subtree                      |
| Rank               | non-negative integer            | Current subspace dimensionality of the coordination tracker |
| Energy ratio       | float                           | Fraction of total energy captured                           |
| Top singular value | float                           | Largest singular value                                      |
| Maturity           | maturity record (§14.6)         | Warm-up state of the coordination tracker                   |
| Scoring geometry   | geometry record (§14.7)         | Structural reliability of coordination scoring axes         |
| Scores             | scoring record (§14.3)          | Four axes of coordination scoring                           |
| Per-member scores  | optional list of member records | Per-cell breakdown from this context's model                |

**Per-member score record** (when present):

| Field                | Type               | Description                                      |
| -------------------- | ------------------ | ------------------------------------------------ |
| Cell interval        | pair of $C$ values | Spatial bounds of the contributing cell          |
| Novelty              | float              | Raw coordination novelty for this cell           |
| Displacement         | float              | Raw coordination displacement for this cell      |
| Surprise             | float              | Raw coordination surprise for this cell          |
| Coherence            | float              | Raw coordination coherence for this cell         |
| Novelty z-score      | float              | Z-score of this cell's coordination novelty      |
| Displacement z-score | float              | Z-score of this cell's coordination displacement |
| Surprise z-score     | float              | Z-score of this cell's coordination surprise     |
| Coherence z-score    | float              | Z-score of this cell's coordination coherence    |

### 14.10 Contour Snapshot · `sec:sentinel:algorithm-output-contour-snapshot`

The contour snapshot describes the G-Tree's observation-receiving surface — the spatial layer's internal structural state — not the set of cells in the batch report. The cell count here is the total contour cell count (terminal + semi-internal nodes), which may differ from the number of cells in the batch report (which includes ancestor cells above the contour).

| Field                          | Type                 | Description                                                                      |
| ------------------------------ | -------------------- | -------------------------------------------------------------------------------- |
| Plateau count                  | non-negative integer | Number of distinct plateaus in the current contour (§3.2)                        |
| Cell count                     | non-negative integer | Total contour cells                                                              |
| Total importance               | float                | Approximate total importance across all cells                                    |
| Splits since last report       | `u32`                | Cells created by catalytic or bootstrap bisection since the previous report      |
| Net removals since last report | `u32`                | Net structural removals (evictions minus restorations) since the previous report |

**Structural mutation counts.** The two mutation fields are derived from `terminal_count()` and `node_count()` deltas between successive `ingest()` calls, using the identities:

- Each split creates 2 nodes and net +1 terminal: $\Delta N = +2$, $\Delta T = +1$.
- Each eviction removes 1 node and 1 terminal: $\Delta N = -1$, $\Delta T = -1$.
- Each restoration creates 1 node and 1 terminal: $\Delta N = +1$, $\Delta T = +1$.

Solving:

$$\text{splits} = \Delta N - \Delta T$$

$$\text{net\_removals} = \text{splits} - \Delta T = \text{evictions} - \text{restorations}$$

Both values are exact. Restorations (legacy promotions) are rare — they occur only when an eviction leaves a semi-internal node — so `net_removals` typically equals the raw eviction count.

The implementation snapshots `terminal_count()` and `node_count()` at the end of each `ingest()` call and computes the deltas on the next call. No graph-internal event counters are required.

### 14.11 Health Report · `sec:sentinel:algorithm-output-health-report`

Available both inline in the batch report (for batch-level completeness) and via a standalone query (for on-demand inspection between observation cycles). Both sources produce identical data.

| Field                        | Type                                    | Description                                                                                                                                                                                                |
| ---------------------------- | --------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Total spatial nodes          | non-negative integer                    | Current materialised G-Tree node count                                                                                                                                                                     |
| Semi-internal count          | non-negative integer                    | One-child spatial nodes                                                                                                                                                                                    |
| Active competitive trackers  | non-negative integer                    | $                                                                                                                                                                                                          \| \mathcal{A}                          \| $                                                          |
| Active ancestor trackers     | non-negative integer                    | $                                                                                                                                                                                                          \| \mathcal{A}^\* \setminus \mathcal{A} \| $                                                          |
| Active coordination contexts | non-negative integer                    | Currently active coordination contexts (§7.1)                                                                                                                                                              |
| Investment set size          | non-negative integer                    | $                                                                                                                                                                                                          \| \mathcal{I}                          \| $ — total cells with allocated trackers (online + warming) |
| Warming trackers             | non-negative integer                    | Members of $\mathcal{I}$ currently in the warm-up pipeline (§11.6)                                                                                                                                         |
| Warming competitive targets  | non-negative integer                    | Competitive targets in $\mathcal{I}$ not yet promoted to $\mathcal{A}$                                                                                                                                     |
| Lifetime observations        | non-negative integer                    | Total observations processed since system creation                                                                                                                                                         |
| Rank distribution            | list of non-negative integers           | Count of trackers at each rank value                                                                                                                                                                       |
| Clip-pressure distribution   | histogram or summary statistics         | Distribution of $\bar{\rho}$ values across all active trackers and axes (§6.4)                                                                                                                             |
| Degenerate-cell count        | non-negative integer                    | Cells excluded from $\mathcal{I}$ due to $w < 2$ (§4.1, §8.2); currently always zero under the $w \geq 2$ eligibility predicate in §8.1 — retained as a defensive field against future eligibility changes |
| Geometry distribution        | geometry distribution record (§14.11.1) | Fleet-wide scoring axis reliability                                                                                                                                                                        |

#### 14.11.1 Geometry Distribution · `sec:sentinel:algorithm-output-health-report-geometry-distribution`

Summarises the structural reliability of scoring axes across all active per-cell trackers.

| Field                    | Type                 | Description                                                          |
| ------------------------ | -------------------- | -------------------------------------------------------------------- |
| Novelty-saturated count  | non-negative integer | Trackers where $k = w$ (novelty axis degenerate)                     |
| Novelty-saturable count  | non-negative integer | Trackers where $\text{cap} \geq w$ (novelty _can_ become degenerate) |
| Coherence-inactive count | non-negative integer | Trackers where $k < 2$ (coherence axis undefined)                    |

### 14.12 Analysis Set Summary · `sec:sentinel:algorithm-output-analysis-set-summary`

| Field                      | Type                          | Description                                                       |
| -------------------------- | ----------------------------- | ----------------------------------------------------------------- |
| Investment set size        | non-negative integer          | $                                                                 \| \mathcal{I}    \| $ (competitive targets + ancestors, online + warming) |
| Producing competitive size | non-negative integer          | $                                                                 \| \mathcal{A}    \| $ (online competitive targets)                        |
| Producing full size        | non-negative integer          | $                                                                 \| \mathcal{A}^\* \| $ (online investment members)                         |
| Depth range                | pair of non-negative integers | (minimum depth, maximum depth) across competitive cells           |
| Importance range           | pair of floats                | (minimum importance, maximum importance) across competitive cells |
| V-Tree depth range         | pair of non-negative integers | (minimum V-depth, maximum V-depth) across competitive cells       |

### 14.13 Contour Queries · `sec:sentinel:algorithm-output-contour-queries`

The spatial layer's contour is available for direct querying through the spatial layer's interface:

| Query         | Description                                | Cost                             |
| ------------- | ------------------------------------------ | -------------------------------- |
| Point query   | The plateau containing a given coordinate  | $O(\log P)$                      |
| Range query   | All plateaus intersecting a given interval | $O(\log P + \text{result size})$ |
| Iteration     | All plateaus in spatial order              | $O(P)$                           |
| Plateau count | Number of distinct plateaus                | $O(1)$                           |

These queries are independent of the observation cycle and may be invoked at any time.

For a deeper diagnostic view of the spatial partition — including its formation history, pre-refinement baselines, and progressive energy decomposition — see the Wavelet Portrait (§14.14).

### 14.14 The Wavelet Portrait · `sec:sentinel:algorithm-wavelet-portrait`

The spatial layer supports on-demand extraction of the Progressive Entropic-Wavelet Exposure Image (PEWEI): a concrete, layered readout of the spatial structure the Sentinel has discovered. It is not another alarm score. It is a diagnostic portrait of the spatial partition itself: where structure was confirmed, at what scale it emerged, and how much observation volume had accumulated before refinement was authorised.

#### 14.14.1 What the Portrait Contains · `sec:sentinel:algorithm-wavelet-portrait-contents`

A PEWEI is an ordered sequence of layers keyed to V-Tree depth. The V-Tree's competitive ranking places high-importance entries at shallow depths and lower-importance entries deeper in the sequence, so the layer order is an approximate significance order with the G-V Graph's $1.44\times$ golden-ratio overhead relative to Shannon entropy (§PEWEI M-7). Structural-only V-Tree depths may contribute no emitted entries, but every emitted node records its V-depth.

Each layer contains two node populations:

| Population | Nodes | Fields |
| ---------- | ----- | ------ |
| Phase transition nodes | Internal or semi-internal G-nodes with confirmed sub-scale structure | Region $[l, r)$, baseline $B = g.\text{own}$, total $S = g.\text{sum}$, refinement $R = S - B$, baseline ratio $R/B$ when $B > 0$ |
| Terminal nodes | Leaf G-nodes at the finest confirmed resolution for their regions | Region $[l, r)$, intensity $I = g.\text{own} = g.\text{sum}$ |

For a phase transition node, the baseline $B$ says: this region accumulated $B$ units of observation volume before the Sentinel decided it had internal structure worth resolving. This is a formation-history datum, not the current alarm level for the region. Fully internal G-nodes provide a clean frozen pre-refinement measurement; semi-internal nodes may carry a hybrid baseline because evicted child energy and later observations in the uncovered half can be folded into $g.\text{own}$ (§PEWEI M-2).

The total $S$ is the complete energy currently accounted for under that region, including all descendants. The refinement $R = S - B$ is the energy attributed to confirmed sub-scale structure after the transition. The baseline ratio $R/B$ reports how large that confirmed sub-scale structure is relative to the pre-refinement benchmark; a large ratio means the subtree has accumulated much more structure than existed when the split decision was made, while a ratio near zero means the split has little subsequent refinement or the children have not accumulated much energy yet. When $B = 0$, the ratio is undefined and must be treated as absent.

For a terminal node, the intensity $I$ is the finest-resolution measurement the Sentinel has confirmed for that region. Terminals are the leaves of the current decomposition at the time the portrait is extracted.

#### 14.14.2 Progressive Reconstruction · `sec:sentinel:algorithm-wavelet-portrait-progressive-reconstruction`

The PEWEI supports truncation at any layer depth while preserving exact total energy across the domain. A host that reads only the first $k$ layers obtains a correctly normalised coarse spatial summary; each additional layer refines the summary by exposing the next significant spatial details.

Reconstruction is additive across scales. For every phase transition visible at depth $\leq k$ whose children are not visible, the host uses the transition's total $S$ as a lump estimate for $[l, r)$. For every transition whose children are visible, the host contributes the transition's baseline $B$ as uniform background under the finer child regions, then adds the child contributions recursively. The parent baseline is not replaced by the children; it persists beneath them.

This follows directly from the summation invariant:

$$g.\text{sum} = g.\text{own} + \sum_c c.\text{sum}$$

Truncation at a coarse depth contributes $g.\text{sum}$ as a lump. Expansion contributes $g.\text{own}$ plus the child sums. Both paths contribute the same total energy. For example, a shallow read may show a transition over $[0, 64)$ with total $S$; a deeper read replaces that lump with the same transition's baseline background plus visible child regions, while the total over $[0, 64)$ remains exactly $S$ (§PEWEI M-10).

#### 14.14.3 Self-Calibration · `sec:sentinel:algorithm-wavelet-portrait-self-calibration`

The frozen baselines are self-calibrating datums: each was deposited by the same observations it later calibrates. This gives the portrait useful operational properties. There is no separate noise-estimation pass, no auxiliary calibration structure, and no extra runtime beyond the extraction walk. The baseline is spatially adaptive because each region carries its own benchmark, and scale-adaptive because each refinement level carries its own benchmark.

The same property is also the portrait's main statistical limitation. The data that discovered the signal also determined the calibration threshold. The frozen benchmark at scale $k$ was the evidence that justified splitting at scale $k$; the children's energy is then measured against this same value. The PEWEI therefore has no statistically independent noise reference such as a holdout sample or an independent fine-scale summary (§PEWEI M-6.3).

Under intensity-proportional noise regimes — event counts, request counts, photon counts, and other processes where variance scales with intensity — this circularity is usually acceptable. The baseline is simultaneously the expected background energy and the natural noise-calibration scale, which matches the Sentinel's standard $\Delta = 1$ observation model. Under additive noise regimes, or when the host needs confidence intervals with frequentist coverage guarantees, the baseline supplies a measured mean but not an independent variance estimate; the host must provide the noise model.

#### 14.14.4 Complementarity With Core Spatial Information · `sec:sentinel:algorithm-wavelet-portrait-core-spatial-complementarity`

When a Sentinel feeds a host, the host core's normal spatial inputs answer questions about current alarm state and outcome history. The PEWEI answers a different question: why the spatial partition has the shape it currently has.

| Pathway | What it tells the host | Timescale | Updated by |
| ------- | ---------------------- | --------- | ---------- |
| Per-cell analysis records (§§14.2–14.7) | How alarmed this cell is right now | Current batch | Each Sentinel observation cycle |
| Contour snapshot (§14.10) | How structurally complex the spatial partition is | Current state | Each Sentinel observation cycle |
| Host outcome ledger | What has historically happened to traffic routed through this cell | Decaying history | Host labels |
| PEWEI (§14.14) | What cumulative spatial structure has been discovered, at what scale, and at what evidence level | Cumulative partition record | Extraction on demand |

The per-cell records contain scores, z-scores, baselines, drift accumulators, maturity, and geometry for cells in the current report. They do not say that a region accumulated a large baseline before splitting, or that one half then required another large baseline before its own structure was confirmed. The PEWEI records that partition-formation history directly.

#### 14.14.5 Host Use Cases · `sec:sentinel:algorithm-wavelet-portrait-host-use-cases`

**Encoding quality verification.** The host can inspect phase transition locations with large baselines and ask whether they align with domain-meaningful boundaries: network allocation boundaries, jurisdictional borders, categorical taxonomy branch points, or other known structure implied by the coordinate encoding (§2). Meaningful alignment indicates that the encoding is exposing real structure to the Sentinel. Transitions at boundaries with no domain interpretation suggest the encoding may be misaligned with the data's natural hierarchy.

**Spatial investment audit.** Deep chains of phase transitions show where the Sentinel has spent its resolution budget. Regions that remain single terminals may be genuinely uniform, or they may be under-resolved because the split threshold is too high for their traffic volume. The portrait makes that distinction inspectable by showing where refinement has and has not occurred.

**Multi-resolution spatial summary.** A host with bounded analysis time can read the first few layers and obtain the dominant spatial structures with correct normalisation. Because the layer order follows the V-Tree competitive ranking, early layers approximate the highest-energy structures before later layers add finer detail.

**Drift detection complement.** Comparing baseline ratios across successive portrait extractions can reveal slow structural change. A cell's per-axis drift accumulators may be quiet because its current scores match recent adaptive baselines, while the PEWEI's $R/B$ ratio keeps growing because the subtree is accumulating more confirmed structure than existed at formation time.

**Temporal decay impact assessment.** After the host applies temporal decay to the spatial layer, the portrait's baselines attenuate along with the graph. Extracting the portrait before and after decay shows how much historical grounding remains. Under depth-selective decay, fine-scale baselines attenuate faster than coarse baselines, making depth-dependent compression visible in the ratios.

#### 14.14.6 Extraction · `sec:sentinel:algorithm-wavelet-portrait-extraction`

The PEWEI is extracted directly from the Sentinel's spatial layer on demand. It is not part of the periodic batch report, and the host core does not mediate the extraction. A host that wants the portrait calls the Sentinel extraction interface and consumes the Sentinel-to-host output directly, consistent with the feed-forward invariant (§1.3).

Extraction is a V-Tree traversal, with cost $O(L + S)$ where $L$ is the number of V-entries and $S$ is the number of V-structural nodes. Every emitted field already exists on the graph except the derived refinement and baseline ratio, which are computed in constant time for each phase transition node (§PEWEI M-9).

#### 14.14.7 Limitations · `sec:sentinel:algorithm-wavelet-portrait-limitations`

The portrait is a snapshot of the Sentinel's state at the moment of extraction. Under temporal decay, baselines attenuate between extractions. Under splits, evictions, and restoration, the structure of the portrait changes. A baseline can say how much evidence existed when a region was formed, but it is not a guarantee that the region's current semantic character is unchanged.

The PEWEI is a diagnostic and explanatory tool, not a standalone detection mechanism. It does not produce z-scores, current alarm levels, outcome probabilities, or policy recommendations. It explains what spatial structure the Sentinel has learned and what evidence levels shaped that structure.

---

# Part VI — Properties and Defence Analysis · `sec:sentinel:algorithm-properties-and-defence-analysis`

This part analyses the properties that emerge from the interaction of the mechanisms defined in Parts II–IV. Each mechanism was specified independently; this part examines what the composition guarantees.

---

## Chapter 15. Scoring Properties · `sec:sentinel:algorithm-scoring-properties`

### 15.1 Axis Independence Under Binary Inputs · `sec:sentinel:algorithm-scoring-properties-binary-input-axis-independence`

The four scoring axes (§5) decompose the observation's relationship to the learned model along orthogonal statistical concerns:

| Score        | Input                                   | Covariance structure                            |
| ------------ | --------------------------------------- | ----------------------------------------------- |
| Novelty      | $\|R_i\|^2 / (w - k)$                   | Residual energy (scalar, orthogonal complement) |
| Displacement | $\|z_i\|^2 / (k + \|z_i\|^2)$           | Total latent energy (scalar)                    |
| Surprise     | $(z_j - \mu_j)^2 / \nu_j$ per $j$       | Diagonal of latent covariance                   |
| Coherence    | $(z_j z_l - \Gamma_{jl})^2$ per $j < l$ | Off-diagonal of latent covariance               |

Together they cover the full covariance structure of the latent space without assembling or inverting a dense $k \times k$ matrix. Each axis can fire independently of the others:

- **Novelty only.** The observation has unusual structure outside the subspace, but its within-subspace projection is unremarkable. Indicates a genuinely novel direction.
- **Displacement only.** The observation projects normally onto the subspace directions but with unusual total magnitude. Indicates a scaling anomaly.
- **Surprise only.** Total projection energy is normal, but its distribution across dimensions is unusual. Indicates an unusual _combination_ of known directions at normal overall energy.
- **Coherence only.** Individual dimensional magnitudes are normal, but their pairwise products are unusual. Indicates an unusual _co-activation pattern_ — dimensions that normally vary independently are varying together (or vice versa).

### 15.2 Projection Energy Redundancy · `sec:sentinel:algorithm-scoring-properties-projection-energy-redundancy`

Under the centred binary encoding (§2.3), $\|\mathbf{x}_i\|^2 = w/4$ for every suffix vector (§2.5). By the Pythagorean theorem:

$$\|\hat{\mathbf{x}}_i\|^2 + \|R_i\|^2 = \|\mathbf{x}_i\|^2 = w/4$$

Therefore projection energy $\|\hat{\mathbf{x}}_i\|^2 / k$ is a perfect affine function of novelty $\|R_i\|^2 / (w - k)$:

$$\frac{\|\hat{\mathbf{x}}_i\|^2}{k} = \frac{w/4 - (w - k) \cdot \text{novelty}_i}{k}$$

Pearson correlation $r = -1$. A fifth axis based on projection energy would carry zero independent information and would violate the polarity invariant (§5.1) — low projection energy would indicate anomaly, but the upper-tail filter (§6.1.1) would fail to protect the baseline.

This redundancy is specific to centred binary inputs. Continuous-valued inputs with variable norms would decouple the two measures. If the system is ever extended to non-binary encodings, projection energy should be reconsidered as an independent axis.

### 15.3 Per-Axis Sensitivity Profiles · `sec:sentinel:algorithm-scoring-properties-per-axis-sensitivity-profiles`

Each axis has a characteristic sensitivity profile that determines what kinds of anomalies it catches best and what kinds it misses:

| Axis         | Most sensitive to                     | Least sensitive to                                                         |
| ------------ | ------------------------------------- | -------------------------------------------------------------------------- |
| Novelty      | New directions never seen before      | Anomalies that stay within the learned subspace                            |
| Displacement | Large-magnitude projections           | Anomalies at normal magnitude but unusual direction                        |
| Surprise     | Per-dimension distributional shifts   | Shifts that affect all dimensions equally (caught by displacement instead) |
| Coherence    | Changed inter-dimension relationships | Single-dimension anomalies (caught by surprise instead)                    |

The axes are complementary by design: what one axis misses, another catches. An adversary who optimises values to minimise one axis's score is forced into a region of observation space that elevates another axis's score — unless the values are genuinely indistinguishable from normal observations on all axes simultaneously.

### 15.4 The Role of Rank · `sec:sentinel:algorithm-scoring-properties-role-of-rank`

The active rank $k$ partitions the observation space into two subspaces of complementary sensitivity:

- The **within-subspace** model ($k$ dimensions) catches displacement, surprise, and coherence anomalies but is blind to novel directions.
- The **residual** model ($w - k$ dimensions) catches novel directions but has no directional sensitivity within the residual space — only aggregate residual energy.

Low rank (small $k$) allocates most dimensions to the residual, giving high sensitivity to novelty but coarse within-subspace modelling. High rank (large $k$) captures more within-subspace structure at the cost of less residual sensitivity. The rank adaptation mechanism (§4.2, Phase 5) balances automatically by tracking cumulative energy.

---

## Chapter 16. Ancestor Chain Properties · `sec:sentinel:algorithm-ancestor-chain-properties`

The ancestor closure (§8.2) transforms a set of independently selected competitive cells into a structured defence. This chapter analyses the properties that emerge from the guaranteed chain of models from each competitive cell to the root.

### 16.1 Nesting Constraint Guarantee · `sec:sentinel:algorithm-ancestor-chain-nesting-constraint-guarantee`

Every competitively selected cell has a **guaranteed complete chain** of models from itself to the root. For every value the adversary submits, the number of models it must simultaneously satisfy equals the number of materialised G-Tree ancestors of the target cell, plus one (the cell itself).

Without the ancestor closure, nesting defence would work only when multiple cells on the same G-Tree path _happened_ to be independently promoted by V-Tree ranking. Under the investment set's ancestor closure, the constraint multiplication is structural and unconditional. Moreover, the g.sum-ordered warm-up pipeline (§11.6.2) ensures that at the moment any competitive target comes online, its entire ancestor chain is already online — the full defence depth is available from the first batch.

### 16.2 Null-Space Coverage · `sec:sentinel:algorithm-ancestor-chain-null-space-coverage`

At each level $\ell$ on the ancestor chain, the model operates on suffix width $w_\ell = N - \ell$ and has learned a rank-$k_\ell$ subspace from the observation population at that level. The model constrains $k_\ell$ directions and is blind to $w_\ell - k_\ell$ directions (its null space).

The directions each model captures are **partially independent across levels** for three reasons:

1. **Different observation populations.** Each level sees a different set of observations — broader at coarser levels. The statistical structure learned from a population of 100,000 observations at depth 0 differs from that learned from 500 observations at depth 48.

2. **Different suffix dimensions.** Each level's suffix includes additional bit positions. The prefix bits at finer levels become suffix bits at coarser levels. A depth-16 model operates on $N - 16$ dimensions; a depth-48 model operates on $N - 48$ dimensions. The $32$ additional dimensions available to the depth-16 model carry cross-region correlations invisible at depth 48.

3. **Cross-level correlations.** The correlations between the additional bits and the deeper suffix bits are precisely what the coarser models learn. These correlations connect the null spaces across levels.

The adversary's "free" bits at level $\ell$ (the null space of the level-$\ell$ model) are constrained at level $\ell - 1$ with probability proportional to how much the coarser model captured cross-level correlations. Each ancestor level peels away some of the adversary's freedom.

### 16.3 Defence Depth Scales With Observation Importance · `sec:sentinel:algorithm-ancestor-chain-defence-depth-by-observation-importance`

Competitive cells sit at depths determined by the spatial layer's adaptive refinement — high-volume regions get deeper cells. The ancestor chain length for each competitive cell equals its G-Tree depth. Consequently, **the cells with the most volume have the longest ancestor chains and the most layers of defence.**

An adversary targeting a busy depth-$d$ cell faces models at all $d + 1$ depths from 0 to $d$ — every materialised level on the ancestor path (§3.1). An adversary targeting a quiet depth-2 cell faces only depths 0, 1, and 2. Fewer constraints — but also a less significant target. The defence depth automatically scales with observation importance.

### 16.4 Constraint Multiplication · `sec:sentinel:algorithm-ancestor-chain-constraint-multiplication`

The per-value constraint count grows with ancestry depth. At each level, the model imposes approximately $k_\ell^2 / 2$ constraints (from the four scoring axes: $k_\ell$ novelty directions, 1 displacement scalar, $k_\ell$ surprise values, and $k_\ell(k_\ell - 1)/2$ coherence pairs). Across a chain of $D$ ancestor levels, the total constraint count is approximately:

$$\sum_{\ell=0}^{D} \frac{k_\ell^2}{2}$$

These constraints are not fully independent (because the observations overlap), but neither are they redundant (because the populations and suffix dimensions differ). The effective constraint count grows meaningfully with depth, even accounting for overlap.

### 16.5 Scale Diagnostics · `sec:sentinel:algorithm-ancestor-chain-scale-diagnostics`

The ancestor chain provides diagnostic information about the **scale** at which an anomaly manifests:

| Pattern                                                | Interpretation                                                                                                                         |
| ------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------- |
| Anomalous at depth 48 only                             | Unusual relative to the specific cell's population but normal in the broader context — a local anomaly.                                |
| Anomalous at depths 16 and above                       | Unusual in the broader regional context — a stronger, coarser-scale signal.                                                            |
| Anomalous at depth 0, normal at depth 48               | The entire region is unusual, but this value is typical within it — the value inherits its region's anomaly.                           |
| Evasion succeeding at depth 48 but failing at depth 32 | The adversary matched local patterns but missed cross-cell correlations captured by the coarser model — the nesting defence in action. |

The host receives scores at every ancestor level and can reconstruct the full scale profile without any additional mechanism.

---

## Chapter 17. Compound Defence and Coverage · `sec:sentinel:algorithm-compound-defence-and-coverage`

### 17.1 Interlocking Constraints: Ancestor Chain × Coordination · `sec:sentinel:algorithm-compound-defence-ancestor-chain-coordination-constraints`

The ancestor chain (§16) and the coordination tier (§7) are not independent fallbacks. The investment set's ancestor closure guarantees the chain exists; the g.sum warm-up ordering guarantees it is online; the coordination tier operates on the producing competitive set. They operate simultaneously on the same observations and impose **interlocking constraints** — satisfying one makes satisfying the other harder.

**The mechanisms' roles.** The ancestor chain provides _per-value depth_: every individual value must produce normal scores at every level from its target cell to the root. The coordination tier provides _cross-cell breadth_: the distribution of scores across spatially related competitive cells must look normal at every level of the hierarchy.

**The interaction.** The same values that must score normally at level $\ell$ per-value also contribute to the coordination inputs at level $\ell - 1$. Per-value evasion at level $\ell$ constrains where in score space cell $c$'s summary can land, which constrains an element of the coordination input matrix at level $\ell - 1$. The optimisation itself leaves fingerprints.

### 17.2 Why Evasion Optimisation Creates Detectable Correlations · `sec:sentinel:algorithm-compound-defence-evasion-correlations`

Legitimate observations arise from independent sources operating independently across different regions of the domain. Score summaries across cells are approximately independent (modulo shared common-mode effects that the coordination tracker has already learned).

Values optimised to evade per-value scoring have a different property: all crafted values are generated by the **same optimisation process** targeting the same set of models. Even if each value individually scores normally, the _population_ of crafted values shares structural regularities imposed by the optimisation:

- **Clustering in safe zones.** If the adversary uses a common algorithm to find safe-zone values, the resulting values cluster in the safe zone rather than spreading across it the way legitimate observations do.
- **Complementary restriction.** If the adversary's values avoid specific null-space directions at a coarser level (to satisfy the ancestor chain), they cluster in the complement — a restriction that manifests as unusual displacement or surprise patterns at the coordination level.
- **Template reuse.** If the adversary reuses structural templates (modifying a few bits per value to maintain evasion while varying the functional content), the templates create correlated residual patterns across cells.

Each coordination axis catches a different signature of optimisation-induced correlation:

| Coordination Axis | Optimisation Fingerprint                                                                                                                                              |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Novelty**       | The cross-cell score pattern is a _new kind_ not in the coordination subspace — the optimisation created a score distribution that normal observations never produce. |
| **Displacement**  | All cells' scores are pinned near the same percentile of the safe zone — individually unremarkable but systematically elevated as a group.                            |
| **Surprise**      | One specific scoring axis is systematically elevated across cells — the optimisation consistently produces slightly high surprise (or another axis) in every cell.    |
| **Coherence**     | Cross-axis correlations are unusual — cells with elevated novelty also have elevated displacement, because the safe-zone geometry links the two.                      |

### 17.3 The Multi-Objective Dilemma · `sec:sentinel:algorithm-compound-defence-multi-objective-dilemma`

Three objectives are in tension for an adversary:

1. **Per-value evasion** (ancestor chain): each value must land in the intersection of safe zones across all ancestor levels. This pushes toward a specific region of the domain → uniformity.

2. **Cross-cell diversity** (coordination): the distribution of scores across cells must match normal cross-cell distributions. This requires values sent to different cells to produce _different_ score profiles → against uniformity.

3. **Cross-level consistency**: the same values' scores at different levels must be jointly consistent with normal cross-level statistics → constrains both.

The adversary must solve a constrained optimisation that satisfies per-value evasion at all levels while producing statistically natural diversity across cells and consistency across levels. Adding calibrated noise to restore diversity helps but is insufficient: the perturbation must itself mimic the natural _structure_ of variation in legitimate observations, not just be "varied." Random perturbations within the safe zone produce uniform distributions within that zone, which may not match the actual distribution of baseline projections (typically clustered along specific subspace directions).

### 17.4 The Irreducible Floor · `sec:sentinel:algorithm-compound-defence-irreducible-floor`

The only remaining evasion strategy is values that are genuinely statistically identical to legitimate observations at every level, every cell, every axis, every cross-cell correlation, and every cross-level consistency measure. This requires characterising the baseline observations' complete joint distribution at every cell AND the cross-cell distribution AND the cross-level consistency — simultaneously, for every value, in every batch.

For a practical adversary operating with imperfect knowledge, time constraints, and operational requirements (the values must serve some external purpose, not just look normal), the composition raises the bar very substantially. But it does not eliminate the possibility: **perfect mimicry of all distributions at all scales remains the irreducible floor** of any detection system based on distributional comparison.

### 17.5 Evasion Assessment by Strategy · `sec:sentinel:algorithm-compound-defence-evasion-strategy-assessment`

The following table summarises how each evasion strategy fares against each defence mechanism individually and in composition:

| Evasion strategy                          | Ancestor chain alone                                                                                                                                                                                                                      | Coordination alone                                                                                                                                                                                           | Composed                                                                                                                                                                                                                                                                                                |
| ----------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Mimic local distribution at one cell      | Must match $d$ models simultaneously                                                                                                                                                                                                      | Not designed for single-cell evasion                                                                                                                                                                         | Match $d$ models **and** produce cross-cell scores matching the natural joint distribution                                                                                                                                                                                                              |
| Exploit null space of target model        | Constrained by ancestor models' coverage of the null space (§16.2)                                                                                                                                                                        | Irrelevant for single-cell evasion                                                                                                                                                                           | Ancestors constrain null space; **coordination detects systematic null-space clustering across cells**                                                                                                                                                                                                  |
| Gradual baseline manipulation             | Must manipulate baselines at all $d$ levels simultaneously, each with different EWMA dynamics                                                                                                                                             | Coordination drift accumulator detects cross-cell drift                                                                                                                                                      | Must manipulate $d$ baselines **without** creating correlated cross-cell drift patterns                                                                                                                                                                                                                 |
| Use common evasion algorithm across cells | Each cell evaded independently                                                                                                                                                                                                            | Detects optimisation-induced uniformity across cells                                                                                                                                                         | **Individually invisible, jointly visible** — the optimisation fingerprint spans cells                                                                                                                                                                                                                  |
| Add noise to restore cross-cell diversity | Noise may violate per-value constraints across levels                                                                                                                                                                                     | Random noise may not match natural correlation structure                                                                                                                                                     | Must simultaneously satisfy per-value constraints at all levels **and** match natural cross-cell correlation structure                                                                                                                                                                                  |
| Patient sub-clip contamination            | Must stay below $c_{\text{clip}}$ at all $d$ levels simultaneously while inflating baselines slowly enough that fast-slow gap never exceeds $\kappa$; multi-level EWMA dynamics differ, so the required injection rate is level-dependent | Sustained sub-clip presence across cells inflates coordination baselines, but correlated inflation across cells elevates coordination scores — coordination drift accumulator detects the cross-cell pattern | Must inflate $d$ per-cell baselines **and** coordination baselines at all levels without producing correlated cross-cell drift patterns; bounded by the same multi-objective dilemma as gradual manipulation, with the additional constraint that scores must remain below all clip ceilings throughout |
| Perfect mimicry of all distributions      | Still works                                                                                                                                                                                                                               | Still works                                                                                                                                                                                                  | Still works — the irreducible floor                                                                                                                                                                                                                                                                     |

> _Design note (patient sub-clip contamination vs. gradual baseline manipulation)._ The "gradual baseline manipulation" strategy assumes an adversary actively targeting baselines with knowledge of the EWMA dynamics. Patient sub-clip contamination is subtler and does not require such knowledge: any sustained adversarial presence producing moderately elevated scores below $c_{\text{clip}}$ will, as a side effect, inflate the fast EWMA $\bar{s}$ and eventually the slow EWMA $\bar{s}_{\text{slow}}$. If the adversary's influence grows slowly enough — less than $\kappa_\sigma \sqrt{\bar{v}_{\text{slow}}}$ per slow-EWMA time constant — the one-sided CUSUM accumulates nothing, because it detects _rate of change_ between fast and slow baselines, not _absolute regime level_. The slow baseline absorbs the contamination and the system's sensitivity to future anomalous traffic degrades invisibly to the drift accumulator. This is an inherent limitation of any adaptive baseline system: the mechanism that enables automatic adaptation to legitimate regime changes is the same mechanism an adversary exploits through patient contamination. The multi-level ancestor chain and coordination tier constrain this attack (the adversary must inflate baselines at all levels without producing correlated cross-cell patterns), but do not eliminate it. Hosts concerned about long-horizon contamination should monitor secular trends in the reported $\bar{s}_{\text{slow}}$ and $\bar{\rho}$ values (§14.5, §14.4) across trackers and flag sustained monotonic increases. This monitoring is a host-level policy responsibility consistent with "Measure, don't decide" (§1.3).

### 17.6 Coverage Matrix · `sec:sentinel:algorithm-compound-defence-coverage-matrix`

The combination of per-cell scoring, ancestor chain scoring, and hierarchical coordination covers six anomaly modalities across three temporal patterns:

```
                Sudden     Gradual    Sudden     Gradual    Sudden      Gradual
                single     single     partial    partial    system      system

Per-cell z      ✓ strong   ✗ absorbed ✓ each     ✗ absorbed ✗ mild      ✗ both
Per-cell drift  redundant  ✓ drift    redundant  ✓ each     ✗ mild      ✗ diluted
Ancestor chain  ✓ depth    ✓ multi-   ✓ depth    ✓ multi-   ✓ root      ✓ root
                defence    baseline   defence    baseline   catches     drift
Coord (local)   ✗ few      ✗ few      ✓ asymm    ✓ asymm    ✗ n/a       ✗ n/a
Coord (mid)     ✗ diluted  ✗ diluted  ✓ struct   ✓ accum    ✓ moderate  ✓ moderate
Coord (root)    ✗ diluted  ✗ diluted  ✗ diluted  ✗ diluted  ✓ strong    ✓ accum
```

**Every modality is detected by at least one mechanism.** The hardest case — gradual system-wide coordination — is caught by the root drift accumulator, which accumulates aggregate departure across all $K$ cells at signal-to-noise ratio proportional to $\sqrt{K}$.

### 17.7 Compound Effect Summary · `sec:sentinel:algorithm-compound-defence-effect-summary`

The ancestor chain and coordination tier do not merely add constraints — they create conflicting objectives. The ancestor chain forces per-value evasion into narrow safe zones, creating the very uniformity that the coordination tier detects. The coordination tier forces cross-cell diversity, creating degrees of freedom that the ancestor chain constrains. The composition raises evasion from a single-model matching problem to a multi-scale, multi-cell, multi-axis constrained optimisation with no known efficient solution — except producing values genuinely indistinguishable from normal observations at every scale simultaneously.

---

# Appendices · `sec:sentinel:algorithm-appendices`

---

## Appendix A. Empirical Warm-Up Analysis · `sec:sentinel:algorithm-empirical-warmup-analysis`

This appendix provides the methodology and extended data behind the convergence times reported in §11.9. These results characterise the interaction between the cold-start mitigations (§11.2–11.4) and the baseline tracking machinery (§6), and inform the noise schedule parameters recommended in §13.5.

**Data currency note.** The convergence times below were measured before the EWMA-mean-centred variance formula ([ADR-S-021](../adr/021-ewma-mean-centred-variance.md)) was implemented. Post-implementation measurements show a modest improvement: worst-case convergence at the production configuration ($\lambda = 0.99$, $b = 16$) improved from 289 to 282 rounds, consistent with the elimination of the original formula's $-6.25\%$ variance bias at $b = 16$. The absolute convergence times reported below remain valid as conservative upper bounds.

### A.1 Methodology · `sec:sentinel:algorithm-empirical-warmup-methodology`

**Setup.** A single subspace tracker at analysis width $w = 96$ and capacity $\text{cap} = 16$ is created, noise-injected according to the configured schedule, and then fed uniformly random centred bit vectors ($\{-0.5, +0.5\}^{96}$, independent and identically distributed) for a sustained observation period. The structureless input ensures that all observed convergence dynamics are properties of the tracker machinery, not of input structure.

**Measurement.** For each scoring axis, the fast EWMA mean $\bar{s}$ is recorded after every batch. A windowed mean over the last 50 batches is computed and compared against a reference value obtained from a long burn-in run (10,000+ batches). Trajectory convergence is declared when the windowed mean enters and remains within a tolerance band (1% relative error for novelty and displacement; 5% for surprise and coherence, reflecting their higher intrinsic variability).

**Controls.** Each configuration is tested across 20 independent random seeds. The convergence time reported is the median; the range across seeds is reported where it is informative (particularly for displacement at small batch sizes, which exhibits bimodal convergence).

### A.2 Cold-Start Mitigations and Their Effects · `sec:sentinel:algorithm-empirical-warmup-cold-start-mitigations`

Three mitigations operate during the noise-to-real transition. Each addresses a specific source of convergence delay:

| Mitigation                              | Source of delay addressed                                                                                                                                            | Effect on convergence                                                                                                                                                                                    |
| --------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Cold-start latent seeding (§11.2)       | Latent variance $\nu^{(z)}$ initialises far from steady state → surprise score non-stationary for tens of rounds                                                     | Eliminates latent variance mismatch at $t = 0$; surprise converges in $\sim$65 rounds instead of $\sim$200+                                                                                              |
| Clip-pressure modulation (§6.1.1, §6.4) | First batch produces near-zero variance → ultra-tight clip ceiling → self-reinforcing low-baseline loop; also, stale baselines cause permanent lockout in production | Breaks the positive feedback loop during warm-up (via $\eta$); prevents permanent lockout in production (via $\bar{\rho}$); displacement converges monotonically instead of exhibiting plateau behaviour |
| Slow-from-fast seeding (§11.4)          | Slow EWMA half-life ($\sim$693 steps) far exceeds fast EWMA half-life ($\sim$69 steps) → drift accumulator interprets the gap as evidence                            | Eliminates false drift accumulation at the noise-to-real boundary; reduces spurious accumulation by $\sim$97%                                                                                            |

### A.3 Convergence Times by Configuration · `sec:sentinel:algorithm-empirical-warmup-convergence-times-by-configuration`

**Configuration A: $\lambda = 0.95$, $b_{\text{noise}} = 4$**

| Component                   |          Without mitigations | With all mitigations | Notes                                                                                          |
| --------------------------- | ---------------------------: | -------------------: | ---------------------------------------------------------------------------------------------- |
| Novelty baseline            |                           21 |                   21 | Unaffected — constant-norm property makes novelty inherently stable                            |
| Displacement baseline       |               500+ (plateau) |     21–475 (bimodal) | Bimodality across seeds: 14/20 seeds converge by round 21; 6/20 seeds require $\sim$475 rounds |
| Surprise baseline           |                         200+ |               **65** | Cold-start seeding is the critical mitigation                                                  |
| Coherence baseline          |                         500+ |              **406** | Dominated by rank-gating delay: $k < 2$ for the first $\sim$100–300 rounds                     |
| Drift accumulator (novelty) | 198 units false accumulation |            5.7 units | Slow-from-fast seeding is the critical mitigation                                              |

**Configuration B: $\lambda = 0.95$, $b_{\text{noise}} = 16$**

| Component                   | Without mitigations | With all mitigations | Notes                                                            |
| --------------------------- | ------------------: | -------------------: | ---------------------------------------------------------------- |
| Novelty baseline            |                  21 |                   21 |                                                                  |
| Displacement baseline       |                300+ |                   21 | Larger batch eliminates bimodality                               |
| Surprise baseline           |                200+ |               **70** |                                                                  |
| Coherence baseline          |                500+ |              **173** | Larger batch accelerates rank growth → shorter rank-gating delay |
| Drift accumulator (novelty) |           180 units |            4.2 units |                                                                  |

**Configuration C: $\lambda = 0.99$, $b_{\text{noise}} = 16$**

| Component                   | Without mitigations | With all mitigations | Notes                                                |
| --------------------------- | ------------------: | -------------------: | ---------------------------------------------------- |
| Novelty baseline            |                 101 |                  101 | Slower $\lambda$ → proportionally longer convergence |
| Displacement baseline       |                800+ |                  101 |                                                      |
| Surprise baseline           |               1000+ |              **398** | Slowest-converging axis at this $\lambda$            |
| Coherence baseline          |                800+ |              **315** |                                                      |
| Drift accumulator (novelty) |           450 units |            8.1 units |                                                      |

### A.4 Displacement Bimodality at Small Batch Sizes · `sec:sentinel:algorithm-empirical-warmup-small-batch-displacement-bimodality`

At $b_{\text{noise}} = 4$ with $\lambda = 0.95$, displacement convergence exhibits bimodal behaviour across random seeds: a majority of seeds converge rapidly ($\sim$21 rounds), while a minority require $\sim$475 rounds with a visible plateau.

The mechanism: displacement's bounded range $[0, 1)$ compresses near zero, where the initial noise-warmed baseline sits. Small batches ($b_{\text{noise}} = 4$) produce high variance in the batch displacement mean. If the first few real batches happen to produce displacement values slightly above the noise-warmed baseline, the clip ceiling widens naturally and convergence proceeds. If they produce values slightly below, the baseline drifts downward, tightening the clip ceiling from below (a direction the upper-tail filter does not protect against), creating a slow recovery.

At $b_{\text{noise}} = 16$, the batch mean variance is $4\times$ smaller, eliminating the bimodality. This effect is specific to displacement at small batch sizes and does not manifest in the other three axes.

### A.5 Coherence Convergence Bottleneck · `sec:sentinel:algorithm-empirical-warmup-coherence-bottleneck`

Coherence is defined only when $k \geq 2$ (§5.5). A newly created tracker starts at $k = 1$ and must accumulate sufficient rank before coherence baselines can even begin tracking. The rank adaptation mechanism (§4.2, Phase 5) evaluates every $T_{\text{rank}}$ steps and moves rank by at most one step, so the minimum time to $k = 2$ is $T_{\text{rank}}$ batches.

In practice, rank growth depends on the energy distribution across singular values. Under structureless noise (uniform random bits), singular values are approximately equal, so the energy threshold $\tau = 0.90$ requires $k \approx 0.9 \times \text{cap}$ — many more than 2. Rank reaches 2 relatively quickly (within $2 \times T_{\text{rank}}$ batches typically), but the coherence baseline then requires its own convergence time on top of the rank-gating delay.

The rank-gating delay is the dominant contributor to coherence being the slowest-converging axis.

### A.6 Steady-State Variability · `sec:sentinel:algorithm-empirical-warmup-steady-state-variability`

Even after convergence, baseline values exhibit ongoing variability. The following coefficients of variation (standard deviation of windowed-mean divided by its mean, measured over 1,000 post-convergence batches) characterise the intrinsic wander:

| Axis         | CV ($b_{\text{noise}} = 4$) | CV ($b_{\text{noise}} = 16$) | Interpretation                                                                         |
| ------------ | :-------------------------: | :--------------------------: | -------------------------------------------------------------------------------------- |
| Novelty      |            0.13%            |            0.06%             | Extremely stable — the constant-norm property (§2.5) eliminates input energy variation |
| Displacement |            5.9%             |             3.0%             | Moderate — bounded range compresses variance reporting                                 |
| Surprise     |            9.0%             |             3.6%             | Moderate — sensitive to per-dimension variance fluctuations                            |
| Coherence    |            19.9%            |            11.5%             | High — pairwise products amplify input variance; $k(k{-}1)/2$ terms compound           |

These CVs represent a **floor** on baseline precision: no amount of warm-up time reduces variability below these values. They should be considered when interpreting z-scores — a z-score of 2.0 on an axis with 20% CV is less significant than a z-score of 2.0 on an axis with 0.1% CV.

### A.7 Noise Schedule Implications · `sec:sentinel:algorithm-empirical-warmup-noise-schedule-implications`

The noise schedule must produce enough rounds at each depth for the worst-case axis (typically coherence or surprise) to reach steady state before real observations arrive. The default geometric schedule (root = 450, decay = 0.5, minimum = 50) is calibrated against Configuration C ($\lambda = 0.99$, $b_{\text{noise}} = 16$) — the default forgetting factor:

| Depth | Rounds (default schedule) | Worst-case convergence ($\lambda = 0.99$) | Margin   |
| ----: | ------------------------: | ----------------------------------------: | -------- |
|     0 |                       450 |                            398 (surprise) | +13%     |
|     1 |                       225 |                                 $\sim$200 | +13%     |
|     3 |                        56 |                                  $\sim$50 | +12%     |
|    4+ |              50 (minimum) |                                  $\sim$50 | Adequate |

The previous default (root = 50, minimum = 10) was calibrated against $\lambda = 0.95$ and produced **negative margin** at the default $\lambda = 0.99$. The updated default ensures baselines are settled before real observations arrive for all default parameters.

For non-default $\lambda$ values, the schedule should be adjusted. Recommended configurations:

| Configuration                                | Root | Decay | Minimum |
| -------------------------------------------- | ---: | ----: | ------: |
| $\lambda = 0.95$, $b_{\text{noise}} \geq 16$ |   50 |   0.5 |      10 |
| $\lambda = 0.95$, $b_{\text{noise}} = 4$     |  200 |   0.5 |      15 |
| $\lambda = 0.99$, $b_{\text{noise}} \geq 16$ |  450 |   0.5 |      50 |
| $\lambda = 0.99$, $b_{\text{noise}} = 4$     |  500 |   0.5 |      50 |

These values include a $\sim$13–15% margin above the measured worst-case convergence times. The geometric decay factor of 0.5 reflects the empirical observation that convergence accelerates at narrower analysis widths (fewer dimensions → simpler subspace → faster baseline settling). The minimum of 50 for $\lambda = 0.99$ configurations accounts for the $\sim$5× longer convergence at high $\lambda$ even at deep (narrow) cells.

---

## Appendix B. Glossary · `sec:sentinel:algorithm-glossary`

| Term                                      | Definition                                                                                                                                                                       | Reference      |
| ----------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------- |
| Analysis budget ($K$)                     | Maximum number of competitive targets; drives the investment set size                                                                                                            | §8.1           |
| Analysis width ($w$)                      | Number of suffix bits available for statistical analysis at a given cell; $w = N - d$                                                                                            | §2.4           |
| Ancestor closure                          | The operation that extends the competitive targets to include all spatial ancestors, forming the investment set $\mathcal{I}$                                                    | §8.2           |
| Baseline                                  | Running mean and variance of a scoring axis, maintained by EWMA, against which z-scores are computed                                                                             | §6.1           |
| Centred bit vector                        | Representation of a coordinate value as a vector in $\{-0.5, +0.5\}^N$                                                                                                           | §2.3           |
| Coherence                                 | Scoring axis measuring departure of pairwise latent products from their learned second moments                                                                                   | §5.5           |
| Competitive targets ($\mathcal{T}$)       | The top $K$ V-entries by importance within the depth cutoff; the selection that drives the investment set                                                                        | §8.1           |
| Constant-norm property                    | The fact that every centred binary suffix vector has squared norm $w/4$                                                                                                          | §2.5           |
| Contour                                   | The step function over the domain formed by the observation-receiving surface of the spatial tree                                                                                | §3.2           |
| Coordination context                      | An internal spatial node whose subtree contains competitive cells in both children; hosts a coordination tracker                                                                 | §7.1           |
| Coordination tracker                      | A subspace tracker operating on cross-cell score summaries ($w = 4$) at a coordination context                                                                                   | §7.5           |
| Degenerate cell                           | A spatial cell with analysis width $w < 2$, excluded from the investment set because the subspace algebra requires $w \geq 2$; reported via the degenerate-cell counter (§14.11) | §4.1, §8.2     |
| Displacement                              | Scoring axis measuring total within-subspace energy as a bounded distance from the subspace origin                                                                               | §5.3           |
| Clip-pressure EWMA ($\bar{\rho}$)         | Per-axis running average of the batch clip ratio; modulates the effective clip ceiling to prevent baseline lockout                                                               | §6.1.1, §6.4   |
| Drift accumulator                         | One-sided CUSUM detecting sustained upward drift of raw (pre-clip) batch means from the slow EWMA baseline                                                                       | §6.3           |
| Dyadic cell                               | An interval of the form $[a \cdot 2^{N-d}, (a+1) \cdot 2^{N-d})$ at depth $d$ in the spatial tree                                                                                | §2.1, §3.2     |
| Eager removal                             | Immediate destruction of trackers and cancellation of warm-up when a cell exits the investment set, within the Step 3 reconciliation                                             | §8.5           |
| Fast EWMA                                 | Exponentially weighted moving average at decay $\lambda$ tracking running mean and variance of a scoring axis                                                                    | §6.1           |
| Feed-forward invariant                    | The guarantee that anomaly scores never influence the spatial layer's importance signal; $\Delta = 1$ always                                                                     | §1.3           |
| Forgetting factor ($\lambda$)             | EWMA decay rate controlling the balance between memory and adaptation                                                                                                            | §4.2           |
| Full analysis set ($\mathcal{A}^*$)       | See: Producing full set                                                                                                                                                          | §8.3           |
| G-Tree (Geometric Tree)                   | The binary tree over dyadic intervals forming the spatial partitioning structure                                                                                                 | §3.1           |
| g.sum                                     | A G-Tree node's total accumulation: its own value plus all descendant sums; used as warm-up priority                                                                             | §3.12, §11.6.2 |
| Investment set ($\mathcal{I}$)            | Competitive targets closed under spatial ancestry; all members have allocated trackers regardless of online status                                                               | §8.2           |
| Max-uncle constraint                      | The V-Tree invariant: no node may outrank all of its uncles                                                                                                                      | §3.4           |
| Noise influence ($\eta$)                  | Fraction of a tracker's baseline not yet established by real observations; decays as $\lambda^n$                                                                                 | §11.5          |
| Noise injection                           | Feeding synthetic random centred bit vectors through a tracker to warm its baselines before real observations arrive                                                             | §11.1          |
| Novelty                                   | Scoring axis measuring average residual energy per degree of freedom outside the learned subspace; degenerate when $k = w$                                                       | §5.2           |
| Novelty-saturated                         | Condition where $k = w$ and the novelty axis is identically zero                                                                                                                 | §5.2, §14.7    |
| Novelty-saturable                         | Condition where $\text{cap} \geq w$, meaning novelty can become saturated as rank adapts                                                                                         | §14.7          |
| Plateau                                   | A maximal contiguous run of contour cells at a single depth                                                                                                                      | §3.2           |
| Polarity invariant                        | The convention that higher scores indicate greater anomalous departure; shared by all four axes                                                                                  | §5.1           |
| Producing competitive set ($\mathcal{A}$) | Online competitive targets; $\mathcal{T} \cap \text{Online}$                                                                                                                     | §8.3           |
| Producing full set ($\mathcal{A}^*$)      | Online members of the investment set; $\mathcal{I} \cap \text{Online}$                                                                                                           | §8.3           |
| Root tracker                              | The permanent subspace tracker at the G-Tree root ($w = N$), seeing every observation                                                                                            | §8.4           |
| Slow EWMA                                 | EWMA at decay $\lambda_s > \lambda$ providing a long-memory reference for the drift accumulator                                                                                  | §6.2           |
| Steiner tree property                     | The ancestor closure forms a Steiner tree connecting $K$ competitive targets to the root; the reduced tree has at most $2K - 1$ nodes. The full materialised tree satisfies $    \| \mathcal{I}    \| \leq 1 + K\bar{D}$ before sharing; ancestor sharing under concentration brings the practical size close to $2K$ | §8.2, §8.6 |
| Subspace tracker                          | The per-cell statistical model maintaining a low-rank subspace, latent statistics, baselines, and drift accumulators                                                             | §4             |
| Suffix                                    | The trailing $w = N - d$ bits of an observation's centred bit vector, after the $d$ routing prefix bits are removed                                                              | §2.4           |
| Surprise                                  | Scoring axis measuring average diagonal Mahalanobis deviation of latent coordinates from their learned means                                                                     | §5.4           |
| V-Tree (Value Tree)                       | The dynamic tournament bracket ranking spatial cells by competitive importance                                                                                                   | §3.1, §3.4     |

---

## Appendix C. Notation Quick Reference · `sec:sentinel:algorithm-notation-quick-reference`

Reproduced from §2.6 for convenience.

| Symbol                | Domain                                      | Definition                                                                                    |
| --------------------- | ------------------------------------------- | --------------------------------------------------------------------------------------------- |
| $N$                   | $\mathbb{Z}_{>0}$                           | Domain bit-width; spatial tree height                                                         |
| $d$                   | $\{0, \ldots, N\}$                          | Spatial tree depth of a cell (prefix length in bits)                                          |
| $w$                   | $\{0, \ldots, N\}$                          | Analysis width; $w = N - d$ (suffix length)                                                   |
| $n$                   | $\mathbb{Z}_{\geq 0}$                       | Ingestion batch size (coordinate values per `SentinelIngest` call)                            |
| $b$                   | $\mathbb{Z}_{>0}$                           | Per-tracker batch size (rows of $X$ fed to one tracker in one call; see §2.6 batch-size note) |
| $b_{\text{noise}}$    | $\mathbb{Z}_{>0}$                           | Noise batch size (synthetic samples per warm-up round; §11.1)                                 |
| $k$                   | $\{1, \ldots, \text{cap}\}$                 | Current active rank of the learned subspace                                                   |
| $\text{cap}$          | $\{1, \ldots, \min(w, r_{\max})\}^\dagger$  | Hard ceiling on rank                                                                          |
| $\lambda$             | $(0, 1)$                                    | Forgetting factor                                                                             |
| $\alpha$              | $(0, 1)$                                    | Learning rate; $\alpha = 1 - \lambda$                                                         |
| $\varepsilon$         | $\mathbb{R}_{>0}$                           | Numerical stability constant                                                                  |
| $\tau$                | $(0, 1)$                                    | Cumulative energy threshold                                                                   |
| $U$                   | $\mathbb{R}^{w \times \text{cap}}$          | Orthonormal basis of the learned subspace                                                     |
| $\sigma$              | $\mathbb{R}^{\text{cap}}_{\geq 0}$          | Singular values                                                                               |
| $\mu^{(z)}$           | $\mathbb{R}^{\text{cap}}$                   | EWMA mean of latent coordinates                                                               |
| $\nu^{(z)}$           | $\mathbb{R}^{\text{cap}}_{>0}$              | EWMA variance of latent coordinates                                                           |
| $\Gamma$              | $\mathbb{R}^{\text{cap} \times \text{cap}}$ | EWMA second-moment matrix of latent coordinates                                               |
| $X$                   | $\mathbb{R}^{b \times w}$                   | Suffix observation matrix                                                                     |
| $Z$                   | $\mathbb{R}^{b \times k}$                   | Latent projection of $X$                                                                      |
| $\hat{X}$             | $\mathbb{R}^{b \times w}$                   | Reconstruction of $X$ from $Z$                                                                |
| $m$                   | $\mathbb{Z}_{>0}$                           | Coordination group size                                                                       |
| $\lambda_s$           | $(\lambda, 1)$                              | Slow EWMA decay for per-tracker drift detection                                               |
| $\lambda_{s,m}$       | $(\lambda, 1)$                              | Slow EWMA decay for coordination drift detection                                              |
| $\kappa_\sigma$       | $\mathbb{R}_{\geq 0}$                       | Drift accumulator noise allowance in slow-baseline $\sigma$ units                             |
| $n_\sigma$            | $\mathbb{R}_{>0}$                           | Outlier clip width in $\sigma$ units                                                          |
| $S$                   | $\mathbb{R}_{\geq 0}$                       | Drift accumulator value                                                                       |
| $\bar{\rho}$          | $[0, 1]$                                    | Per-axis clip-pressure EWMA                                                                   |
| $\lambda_\rho$        | $(0, 1)$                                    | Clip-pressure EWMA decay rate                                                                 |
| $\rho_t$              | $[0, 1]$                                    | Batch clip ratio                                                                              |
| $\mu^{(\text{in})}$   | $\mathbb{R}^4$                              | Running-mean centring reference for coordination input                                        |
| $\eta$                | $[0, 1]$                                    | Noise influence fraction (tracker maturity)                                                   |
| $K$                   | $\mathbb{Z}_{>0}$                           | Maximum number of competitively selected analysis cells                                       |
| $L$                   | $\mathbb{Z}_{\geq 0}$                       | V-Tree depth cutoff for analysis eligibility                                                  |
| $\mathcal{A}$         | $\subseteq$ V-entries                       | Producing competitive set ($\mathcal{I} \cap \mathcal{T} \cap \text{Online}$)                 |
| $\mathcal{A}^*$       | $\supseteq \mathcal{A}$                     | Producing full set ($\mathcal{I} \cap \text{Online}$)                                         |
| $\mathcal{T}$         | $\subseteq$ V-entries                       | Competitive targets (top $K$ by importance within depth cutoff)                               |
| $\mathcal{I}$         | $\subseteq$ V-entries + ancestors           | Investment set (competitive targets + spatial ancestors, regardless of online status)         |
| $G_{\max}$            | $\mathbb{Z}_{\geq 5}$                       | Hard ceiling on total spatial nodes                                                           |
| $\lambda_{\text{sp}}$ | $[0, \infty)$                               | Spatial decay attenuation factor                                                              |
| $h_V$                 | $\mathbb{Z}_{\geq 0}$                       | V-Tree height (maximum root-to-leaf path length in the V-Tree)                                |

$^\dagger$ The domain $\{1, \ldots, \min(w, r_{\max})\}$ is non-empty only when $w \geq 1$. Tracker creation requires $w \geq 2$ (§4.1); cells below this threshold are excluded from $\mathcal{I}$.

Vectors are row vectors when representing observations and column vectors when representing basis directions. Subscript $i$ indexes samples; subscript $j$ indexes subspace dimensions.
