# The Dual-Tree Value-Stratified Index: A Coding-Theoretic Foundation

## A Conceptual Design Document From First Principles in Information Theory

---

# Part I: The Code

---

## Chapter 1. A Binary Partition Is a Code

### 1.1 The Kraft Identity

Let $[0, 2^N)$ be a discrete domain. A **complete dyadic partition** $\mathcal{C}$ of this domain is a collection of half-open dyadic intervals

$$\mathcal{C} = \bigl\{[l_i, r_i) : r_i - l_i = 2^{k_i},\; k_i \geq 0\bigr\}$$

that tile the domain without gaps or overlaps:

$$\bigsqcup_{i} [l_i, r_i) = [0, 2^N)$$

Define the **depth** of each cell as $d_i = N - k_i$ (the number of bisections from the full domain to this cell's width). Then:

$$\sum_{i} 2^{-d_i} = \sum_{i} \frac{r_i - l_i}{2^N} = \frac{1}{2^N}\sum_{i}(r_i - l_i) = \frac{2^N}{2^N} = 1$$

This is the **Kraft equality**. Not an inequality — an equality. The partition is complete (no unused codewords) and prefix-free (no cell is a prefix of another, because they tile disjointly). Every complete dyadic partition satisfies this identity. Every set of depths $\{d_i\}$ satisfying Kraft with equality corresponds to a unique complete dyadic partition.

**This is the foundational identity of the entire architecture.** A binary tree's leaf set, viewed as an adaptive partition of the domain, is isomorphic to a complete prefix-free binary code. The leaf depths are the codeword lengths. The partition is the code. The code is the partition. This is not an analogy.

### 1.2 What the Code Describes

In source coding (compression), codewords are assigned to symbols: frequent symbols get short codewords, rare symbols get long codewords. The code allocation is **away from** distributional mass — short codes for heavy symbols, which take few bits precisely because they appear often.

The G-Tree operates in the **dual direction**: it is a **description code**. Deep cells (long codewords) are assigned to regions of high intensity. Shallow cells (short codewords) cover regions of low intensity. The code allocates precision **toward** distributional mass.

This is the rate-distortion dual of source coding. Source coding minimises the description length of a message drawn from a distribution. The description code minimises the spatial distortion of a distribution given a precision budget. Both are governed by the same mathematics — the Kraft equality, the entropy bound, the trade-off between fidelity and cost — but with the direction reversed.

|                        | Source Coding               | Description Coding (G-Tree)      |
| ---------------------- | --------------------------- | -------------------------------- |
| **Depth assignment**   | Short for frequent          | Deep for intense                 |
| **Direction**          | Away from mass              | Toward mass                      |
| **Objective**          | Minimise description length | Minimise spatial distortion      |
| **Constraint**         | Kraft equality              | Kraft equality                   |
| **Optimal allocation** | $d_i = \log_2(1/p_i)$       | $d_i \propto$ local significance |
| **Overhead**           | Relative to entropy         | Relative to entropy              |

### 1.3 The Code Is Dynamic

The G-Tree's partition is not fixed. It evolves through three mutations:

| Mutation                                               | Code operation                                                                         | Kraft effect                       |
| ------------------------------------------------------ | -------------------------------------------------------------------------------------- | ---------------------------------- |
| **Refinement** (split a cell)                          | Replace one codeword of length $d$ with two of length $d+1$                            | $-2^{-d} + 2 \cdot 2^{-(d+1)} = 0$ |
| **Restoration** (re-split a partially contracted cell) | Same as refinement                                                                     | $0$                                |
| **Eviction** (remove a cell, parent absorbs)           | Replace two codewords of length $d+1$ with one of length $d$ (when both children gone) | $-2 \cdot 2^{-(d+1)} + 2^{-d} = 0$ |

Every mutation preserves the Kraft equality. The code is always complete, always prefix-free, always a valid partition. This is not maintained by careful bookkeeping — it is a structural invariant of binary partition. You cannot split a dyadic interval into two dyadic halves and violate Kraft. The partition cannot become incomplete or redundant through any sequence of splits and merges.

The intermediate state — one child evicted, one surviving (semi-internal node) — represents a code where one codeword at length $d$ has been partially refined. The parent covers the vacated half at depth $d$; the surviving child covers its half at depth $d+1$. The partition is still complete:

$$2^{-(d+1)} + 2^{-(d+1)} = 2^{-d} \quad\longrightarrow\quad 2^{-(d+1)} + 2^{-d} \cdot \frac{1}{2} = 2^{-d} \cdot \frac{1}{2} + 2^{-(d+1)}$$

The semi-internal node contributes $2^{-d} \cdot (1/2)$ (the uncovered half) and the child contributes $2^{-(d+1)}$ (the covered half). Sum = $2^{-d}$. Kraft holds.

### 1.4 The Code Has Finite Precision

The domain $[0, 2^N)$ has $2^N$ addressable points. The maximum codeword length is $N$ (a cell of width 1). This is the code's resolution limit. No codeword can represent finer structure than a single point.

For floating-point domains, $N$ is an explicit parameter bounding the maximum refinement depth — the user's declaration of how many levels of spatial (or abstract) precision are meaningful.

### 1.5 The Code Is the G-Tree

Every statement about the G-Tree in the specification is equivalently a statement about this dynamic code:

| G-Tree concept                                            | Code concept                                     |
| --------------------------------------------------------- | ------------------------------------------------ |
| G-node at depth $d$                                       | Codeword of length $d$                           |
| Contour (set of terminal + uncovered semi-internal cells) | Current codebook                                 |
| Contour refinement (split)                                | Codeword lengthening                             |
| Contour simplification (eviction)                         | Codeword shortening                              |
| G-Tree sum propagation                                    | Distributional mass accounting                   |
| Root node                                                 | The empty codeword (length 0, covers everything) |
| Dyadic interval $[l, r)$                                  | The symbol this codeword describes               |

The specification's 200 pages of spatial language — routing, contours, plateaus, midpoints — are the operational mechanics of maintaining this dynamic code. The spatial language is the **implementation**. The code is the **mathematical object**.

---

## Chapter 2. The Allocation Problem

### 2.1 The Problem

Given a finite precision budget (bounded number of codewords, equivalently bounded number of G-nodes), how should codeword lengths be assigned to regions of the domain?

More precisely: observations arrive as pairs $(x, \Delta)$ — a coordinate and a weight. The code must continuously allocate its precision budget to track the evolving distribution of observations. Regions receiving heavy observation should have fine codewords (deep cells). Regions receiving light observation should have coarse codewords (shallow cells). The allocation must adapt as the distribution changes.

This is a **rate-distortion allocation problem**. The "rate" is the precision budget (codeword count). The "distortion" is the spatial error introduced by representing a continuous distribution with a finite codebook. The optimal trade-off is governed by the distribution's entropy.

### 2.2 Why the Problem Is Hard

A static allocation is easy: given a known distribution $p(x)$, assign $d_i \approx \log_2(1/p_i)$ and you achieve near-optimal rate-distortion. But:

1. **The distribution is unknown.** Observations arrive one at a time. The true distribution is never fully revealed.
2. **The distribution is non-stationary.** Regions that were hot become cold. New hotspots emerge. The code must adapt.
3. **The adaptation must be local.** Global restructuring on every observation is $O(n)$. Practical systems need per-observation cost that is sublinear.
4. **The adaptation must be monotone in evidence.** A region that has consistently received heavy observations should not lose precision due to a transient fluctuation elsewhere.

The architectural answer is: separate the code from the policy that shapes it.

### 2.3 The Two Trees

The code (the G-Tree) maintains the partition and accounts for accumulated mass. The policy (the V-Tree) decides where to allocate and deallocate codewords.

| Concern                                            | Owner  | Object                                  |
| -------------------------------------------------- | ------ | --------------------------------------- |
| What partition exists right now                    | G-Tree | The current codebook (contour)          |
| How much mass each codeword region has accumulated | G-Tree | Sum propagation (G-I1)                  |
| Which codewords should be lengthened (refined)     | V-Tree | Competitive ranking → shallow position  |
| Which codewords should be shortened (evicted)      | V-Tree | Competitive ranking → deep position     |
| How efficiently the policy adapts                  | V-Tree | Fibonacci depth bound ($1.44 \times H$) |

This separation is natural from the coding perspective: a code is a mathematical structure; the allocation policy is a separate decision process. Conflating the two — as most tree-based data structures do — limits the policy to structural properties of the code (balance factors, node counts) rather than distributional properties of the data (importance, entropy).

---

## Chapter 3. The Policy: A Competitive Tournament

### 3.1 The Policy Interface

The V-Tree ranks codeword entries by an opaque **importance** measure. The ranking governs two code mutations:

1. **Codeword lengthening (refinement):** Only entries with high rank (shallow V-Tree position) may split their cells. This ensures precision is allocated to globally significant codewords.

2. **Codeword shortening (eviction):** Only entries with low rank (deep V-Tree position) and no structural dependents may be removed. This ensures precision is deallocated from globally insignificant codewords.

The policy never sees coordinates, intervals, or spatial structure. It sees only importance values through an opaque algebraic interface. This opacity is what enables self-composition (Part III): the policy works identically regardless of what the coordinates represent.

### 3.2 The Algebraic Interface

The importance values inhabit a **value space** $\mathcal{V} = (I, \oplus, \nu, \preceq)$:

| Component              | Role                                  | Code-theoretic meaning                         |
| ---------------------- | ------------------------------------- | ---------------------------------------------- |
| $I$ (carrier)          | The set of importance values          | The measurement type for codeword significance |
| $\oplus$ (aggregation) | Combine two importances               | Aggregate significance of a code subtree       |
| $\nu$ (ground)         | Starting importance for new codewords | A fresh codeword has contributed nothing       |
| $\preceq$ (ordering)   | Compare significances                 | Which codeword is more informative             |

The base axioms (P0) require closure, commutativity, compatibility ($a \preceq b \implies a \oplus c \preceq b \oplus c$), and total ordering. These are the minimum needed for the tournament to function coherently.

Optional properties enable additional features:

| Property               | Statement                                       | Feature enabled                                 |
| ---------------------- | ----------------------------------------------- | ----------------------------------------------- |
| P1 (bounded below)     | $\exists \bot: \forall x, \bot \preceq x$       | Proportional sampling; Fibonacci bound          |
| P2 (grounded)          | $\forall a: \nu \preceq a$                      | Violation-free codeword insertion               |
| P3 (idempotent ground) | $\nu \oplus \nu = \nu$                          | Violation-free structural nodes                 |
| P4 (identity)          | $\forall a: \nu \oplus a = a$                   | Ghost eviction fast path                        |
| P5 (associativity)     | $(a \oplus b) \oplus c = a \oplus (b \oplus c)$ | Propagation-free rearrangement; Fibonacci bound |

**Theorem (Additive Collapse).** Under ordinary addition on a connected subset of $\mathbb{R}$, the unique fully-featured value space is $(\mathbb{R}_{\geq 0}, +, 0, \leq)$, which satisfies P0–P5 simultaneously. Every other additive choice loses at least one property.

This theorem is important because it means the standard configuration is not a design choice — it is the unique algebraic optimum under addition. The architecture discovers the right importance type rather than imposing it.

### 3.3 The Tournament Structure

The V-Tree is a tree with branching factor 2 or 3 where:

- **Leaves** are codeword entries (V-entries backed by G-nodes)
- **Internal nodes** are pure structural scaffolding (no external identity, freely created and destroyed)
- **High-importance entries sit near the root; low-importance entries sit deep**

The tournament maintains a single structural invariant:

$$\textbf{V-I3 (Max-Uncle):}\quad c.\text{int} \leq \max\{u.\text{int} : u \in \text{uncles}(c)\}$$

A node may not outrank every one of its uncles. When it does, the tournament restructures — promoting the node toward the root.

**Why max-uncle rather than a balance condition?** Balance conditions (AVL, red-black) constrain the tree's shape: heights, colors, node counts. These structural properties have no natural connection to the distributional properties we care about (importance, entropy, rate-distortion). The uncle constraint is a **value-based** constraint: it constrains the relationship between importance values, not structural properties. This is what enables the entropy-sensitive depth bound.

### 3.4 The Three Shields

The code and policy protect each other through three mechanisms that create a stable governance cycle:

**Shield 1: Observation routing (G-Tree → V-Tree, upward).** When a codeword is refined (cell splits), its children intercept all future observations in the parent's range. The parent's importance freezes. This creates a fixed benchmark — the competitive bar that children must surpass to earn promotion.

Without this shield, the parent would continue accumulating importance alongside its children. The benchmark would be a moving target. Children could never outgrow it. The competitive mechanism would be dead. **The code's spatial topology (routing) is the mechanism that creates fixed benchmarks for the policy.**

**Shield 2: Uncle constraint (V-Tree → entries, downward).** The max-uncle constraint stabilises children's positions. Three siblings of comparable importance coexist indefinitely — a violation requires beating _all_ uncles. The tournament restructures only on dramatic dominance, not on every fluctuation.

Without this shield, every minor importance change would cause restructuring. The code would thrash — codewords continuously lengthening and shortening in response to noise rather than signal.

**Shield 3: Structural immunity (G-Tree → eviction, structural).** Only codewords with no dependents (no finer sub-codewords relying on them) may be evicted. The code contracts from its finest tips inward.

Without this shield, evicting a codeword could orphan its sub-codewords — creating gaps in the partition, violating the Kraft equality.

### 3.5 The Lifecycle of a Codeword

The three shields create a complete lifecycle:

```
ACTIVE CODEWORD (on the contour, receiving observations)
    │
    │ importance exceeds θ, shallow V-position
    │ → REFINE: cell splits, codeword lengthens
    ▼
FROZEN BENCHMARK (above the contour, no observations)
    │ children intercept all traffic
    │ importance frozen at pre-split value
    │
    ├─── one child evicted ───► PARTIALLY ACTIVE
    │                            (semi-internal: half exposed)
    │                            │
    │                            ├─── other child evicted ───► ACTIVE again
    │                            │     (fully exposed, cycle restarts)
    │                            │
    │                            └─── competitive promotion ──► FROZEN again
    │                                  (missing child restored)
    │
    └─── (never reached directly: both children gone) ──► ACTIVE
         (cycle restarts, or evicted if globally insignificant)
```

**The competitive promotion mechanism** is the key innovation. When a partially active codeword (one child evicted) accumulates enough importance to outgrow all its uncles in the tournament, the tournament restructures — and the restructuring itself creates the missing child. The codeword re-freezes as a benchmark. The contour regrows at that point.

No separate threshold, no explicit "regrowth" operation. The same competitive mechanism that governs the tournament doubles as the gate for contour growth. **The code grows through competition.**

---

## Chapter 4. The Efficiency Bound

### 4.1 The Fibonacci Depth Theorem

**Theorem.** In any V-Tree satisfying V-I3 (max-uncle) and V-I2 (branching factor 2 or 3), with P1 (bounded below) and P5 (associativity), an entry with weight fraction $w_i = v_i.\text{int} / I_{\text{total}}$ has depth:

$$d_i \leq \log_\varphi\!\left(\frac{1}{w_i}\right) + 1$$

where $\varphi = \frac{1+\sqrt{5}}{2}$.

**Proof.** Consider a path from root to entry $v$ at depth $d$: $v_0, v_1, \ldots, v_d = v$. Write $I_k = v_k.\text{int}$.

**The Fibonacci recurrence.** For $k \geq 2$, the triple $(v_k, v_{k-1}, v_{k-2})$ is child–parent–grandparent. V-I3 requires that $v_k$ does not exceed every uncle. Since $v_{k-2}$ has at least 2 children (V-I2), $v_{k-1}$ has at least one sibling $u$ with $u.\text{int} \geq v_k.\text{int}$. Therefore:

$$I_{k-2} = I_{k-1} + \sum_{\text{siblings}} u.\text{int} \geq I_{k-1} + I_k$$

This uses P1 (non-negative importance, so additional siblings cannot decrease the sum) and P5 (associativity, so the telescoping is valid). The recurrence $I_{k-2} \geq I_{k-1} + I_k$ runs **backward from the leaf** — it is the Fibonacci recurrence with initial conditions $I_d = v.\text{int}$ and $I_{d-1} \geq v.\text{int}$.

**Unwinding.** By induction, $I_{d-j} \geq F_{j+1} \cdot I_d$, where $F_n$ is the $n$-th Fibonacci number. At the root: $I_0 = I_{\text{total}}$, so:

$$\frac{1}{w_i} = \frac{I_{\text{total}}}{I_d} = \frac{I_0}{I_d} \geq F_{d+1} \geq \varphi^{d-1}$$

$$\boxed{d_i \leq \log_\varphi\!\left(\frac{1}{w_i}\right) + 1}$$

$\blacksquare$

### 4.2 The Coding-Theoretic Meaning

The Fibonacci depth bound is an entropy bound in disguise. Every step down the V-Tree gains at least $\log_2 \varphi \approx 0.694$ bits of information about which codeword entry will be reached. The bound says:

$$\text{depth to reach entry } i \leq \frac{\log_2(1/w_i)}{\log_2 \varphi} + O(1) = \frac{\text{self-information of } i}{\log_2 \varphi} + O(1)$$

The self-information $\log_2(1/w_i)$ is the minimum number of bits needed to specify entry $i$ from the distribution $\{w_i\}$. The Fibonacci bound says the V-Tree requires at most $1/\log_2\varphi \approx 1.44$ times this minimum. **The V-Tree is a near-optimal probability routing structure.**

### 4.3 Expected Sampling Cost

**Corollary.** Proportional sampling — choosing entry $i$ with probability $w_i$ — has expected cost:

$$E[\text{cost}] = \sum_i w_i \cdot d_i \leq \sum_i w_i \cdot \left(\frac{\log_2(1/w_i)}{\log_2\varphi} + 1\right) = \frac{H}{\log_2\varphi} + 1 \approx 1.44\,H + 1$$

where $H = \sum_i w_i \log_2(1/w_i)$ is the Shannon entropy. $\square$

**Comparison to optimal:**

- **Information-theoretic lower bound:** $H / \log_2 3 \approx 0.63\,H$ (maximum $\log_2 3$ bits per step at a 3-node)
- **V-Tree upper bound:** $1.44\,H + 1$
- **Gap:** $\log_\varphi 3 \approx 2.28$

The 2.28× gap is the cost of maintaining the ranking through purely local operations (the uncle constraint). A globally optimal structure would achieve the lower bound but would require non-local restructuring on every update. The V-Tree's locality — every violation is resolved by touching $O(1)$ nodes in a bounded neighbourhood — is what makes the structure practical.

### 4.4 Tightness

The bound is tight. A Fibonacci chain construction achieves it: set $w_i \propto \varphi^{-i}$ along a degenerate 2-node chain. Every entry sits at maximum permitted depth. The uncle constraint is satisfied with equality at every level: $I_{k-2} = I_{k-1} + I_k$ exactly.

The leading coefficient $1/\log_2\varphi$ is not an artefact of the proof. It is the genuine per-step information yield of the Fibonacci recurrence.

### 4.5 The AVL Parallel

The Fibonacci recurrence appears in both AVL trees and V-Trees:

|                | AVL                                     | V-Tree                                  |
| -------------- | --------------------------------------- | --------------------------------------- |
| **Constraint** | Height-balance ($\|h_L - h_R\| \leq 1$) | Max-uncle ($c \leq \max\{u\}$)          |
| **Nature**     | Structural (heights)                    | Value-based (importances)               |
| **Recurrence** | $N_h \geq N_{h-1} + N_{h-2} + 1$        | $I_{k-2} \geq I_{k-1} + I_k$            |
| **Growth**     | Minimum nodes grow as $\varphi^h$       | Minimum importance grows as $\varphi^d$ |
| **Bound**      | $h \leq \log_\varphi n + O(1)$          | $d \leq \log_\varphi(1/w_i) + O(1)$     |
| **Overhead**   | $1/\log_2\varphi \approx 1.44$          | $1/\log_2\varphi \approx 1.44$          |

Both sacrifice the same 44% overhead for the same reason: the local constraint's extremal case follows the Fibonacci sequence, and $\log_2\varphi \approx 0.694$ is the per-step entropy of that sequence.

The difference: AVL bounds height by node count (a structural quantity). The V-Tree bounds depth by weight fraction (a distributional quantity). The V-Tree's bound is sensitive to the distribution — concentrated distributions give shallow depth, uniform distributions give logarithmic depth, heavy-tailed distributions give constant expected depth. AVL's bound is always $\Theta(\log n)$ regardless of the distribution.

### 4.6 Regime-Dependent Behaviour

| Distribution regime   | $H$         | Expected sampling cost | Balanced tree    |
| --------------------- | ----------- | ---------------------- | ---------------- |
| $k$ hotspots          | $O(\log k)$ | $O(\log k)$            | $\Theta(\log L)$ |
| Uniform               | $\log_2 L$  | $\leq 1.44 \log_2 L$   | $\log_2 L$       |
| Zipf ($\alpha > 1$)   | $O(1)$      | $O(1)$                 | $\Theta(\log L)$ |
| Geometric ($\beta^i$) | $O(1)$      | $O(1)$                 | $\Theta(\log L)$ |

The V-Tree's cost adapts to the distribution's entropy. A balanced segment tree always costs $\Theta(\log L)$. For concentrated distributions — a few dominant codewords — the V-Tree achieves $O(1)$ expected sampling while the balanced tree still costs $O(\log L)$.

---

## Chapter 5. The Projection: Bridging Mass to Policy

### 5.1 The Fundamental Separation

The code (G-Tree) accumulates **ledger values** $T$ — the raw measurement. The policy (V-Tree) ranks by **importance values** $I$ — the governance signal. These types are intentionally distinct:

- **Ledger** ($T$): may be signed, complex, vector-valued — whatever the domain needs.
- **Importance** ($I$): must satisfy the value space axioms (ordered, aggregatable, optionally non-negative).

The bridge between them is the **projection function**:

$$\pi : I \times T \to I$$

The projection is the user's declaration of what "mattering" means. Under identity projection $\pi(i, \Delta) = i + \Delta$, importance equals raw accumulation. Under absolute projection $\pi(i, \Delta) = i + |\Delta|$, importance tracks cumulative absolute activity regardless of sign. Under a custom projection, importance tracks whatever the user decides is significant.

### 5.2 The Three Standard Configurations

The additive collapse theorem (§3.2) restricts the landscape:

| Configuration | Carrier $I$           | Ground $\nu$ | Projection $\pi$ | Properties | Missing features          |
| ------------- | --------------------- | ------------ | ---------------- | ---------- | ------------------------- | ----- | ----------------------- |
| **Standard**  | $\mathbb{R}_{\geq 0}$ | $0$          | $i + \Delta$     | P0–P5      | None                      |
| **Absolute**  | $\mathbb{R}_{\geq 0}$ | $0$          | $i +             | \Delta     | $                         | P0–P5 | None (importance ≠ own) |
| **Signed**    | $\mathbb{R}$          | $0$          | $i + \Delta$     | P0, P3–P5  | Sampling, Fibonacci bound |

The standard and absolute configurations share the same value space $(\mathbb{R}_{\geq 0}, +, 0, \leq)$ and differ only in projection. Both achieve P0–P5, giving all features including the Fibonacci bound. The signed configuration loses P1 (no lower bound in $\mathbb{R}$) and with it loses proportional sampling and the depth guarantee.

The exhaustiveness theorem proves these three are the only non-dominated additive configurations: every other choice under ordinary addition on a connected carrier is either equivalent to one of these or strictly dominated by one.

### 5.3 Single-Entry Update Principle

Each observation updates exactly one V-entry's importance — the receiving codeword's entry. The V-Tree's total importance is determined by the projection:

- Under standard configuration: $I_{\text{total}} = \sum_i \Delta_i = G_{\text{root}}.\text{sum}$
- Under absolute configuration: $I_{\text{total}} = \sum_i |\Delta_i|$

No double-counting. Clean accounting. **The code's total importance is a conserved quantity across all mutations** (splits, evictions, absorptions). This conservation is what makes the Fibonacci bound meaningful — the weight fractions $w_i$ always sum to 1.

---

## Chapter 6. The Dynamics: How the Code Evolves

### 6.1 The Observation Flow as Code Maintenance

An observation $(x, \Delta)$ triggers a sequence of code maintenance operations:

1. **Route to codeword.** Descend the code tree to find which current codeword covers coordinate $x$. Cost: $O(d_{\text{geo}})$.

2. **Accumulate mass.** Update the codeword's ledger ($g.\text{own} \mathrel{+}= \Delta$) and importance ($g.\text{importance} \leftarrow \pi(g.\text{importance}, \Delta)$). This is the raw measurement step.

3. **Propagate aggregates.** Update the code tree's cumulative sums (G-I1) along the root-ward path, and the tournament's structural aggregates (V-I1) along the tournament's root-ward path. These are independent propagations over independent trees.

4. **Detect violations.** The importance increase may cause the entry or its tournament ancestors to exceed their uncles. Walk the tournament path, checking V-I3 at each level.

5. **Attempt code lengthening.** If the receiving codeword has accumulated sufficient importance ($> \theta$) and holds a globally significant tournament position ($\text{depth}_V \leq D_{\text{create}}$), split the cell. This creates two new codewords and freezes the parent as a benchmark.

6. **Resolve violations.** The tournament restructures to restore V-I3 via contraction, standard promote, skip promote, or legacy promote. Each primitive touches $O(1)$ nodes in a bounded neighbourhood.

7. **Code contraction.** Globally insignificant codewords (deep in the tournament, no structural dependents) are evicted. Their mass is absorbed by the parent codeword. The code shortens.

**Every step is a code maintenance operation.** Steps 1–3 are measurement and accounting. Step 4 is violation detection. Step 5 is code lengthening (rate allocation). Step 6 is tournament maintenance (policy update). Step 7 is code shortening (rate deallocation).

### 6.2 Catalytic Splitting: The Code Remembers

When a codeword splits, the parent is not consumed. It persists as a V-entry carrying pre-split importance — a **frozen benchmark**. The children start at ground importance $\nu$ and must earn their way up against this fixed bar.

This is the coding-theoretic essence of the catalytic property:

$$\text{Code at time } t: \quad [\underbrace{l, r}_{\text{codeword } c, \text{ importance } I})\qquad\longrightarrow$$

$$\text{Code at time } t+1: \quad \underbrace{[l, m)}_{\text{codeword } c_L, \text{ importance } \nu}\quad \underbrace{[m, r)}_{\text{codeword } c_R, \text{ importance } \nu}\quad + \quad \underbrace{c}_{\text{frozen benchmark, importance } I}$$

The frozen benchmark is the code's **memory** at this scale. It records: "at the moment the code decided to allocate more precision to this region, the undifferentiated energy was $I$." This is a direct measurement at the moment of the code's structural decision. It is exact, scale-specific, and spatially localised.

### 6.3 The Inversion: Code and Policy Diverge

For internal codewords (both children present), the code's and policy's rankings of the same node diverge over time and eventually invert:

$$\frac{g.\text{own}}{g.\text{sum}} \to 0 \qquad\text{as descendants accumulate}$$

The code (ranking by sum) sees the node as increasingly important — it contains ever more mass. The policy (ranking by importance) sees it as increasingly negligible — a shrinking fraction of the original.

The root is the extreme: highest sum (it contains everything) but lowest eventual importance (frozen at the earliest, most ancient value).

The inversion is the **derivative/integral duality**:

|           | G-Tree (sum)                 | V-Tree (importance)                      |
| --------- | ---------------------------- | ---------------------------------------- |
| Direction | Top-down: parents ≥ children | Bottom-up: children eventually > parents |
| Measures  | Cumulative mass (integral)   | Scale-specific contribution (derivative) |
| Root      | Maximum                      | Minimum among frozen entries             |
| Leaves    | Minimum                      | Maximum among active entries             |

The code says "how much mass is contained here and below." The policy says "how much mass arrived _at this scale specifically_." One is the integral. The other is the derivative. They measure the same signal from complementary directions.

### 6.4 Eviction as Code Contraction

When a codeword is evicted:

1. Its mass is absorbed by the parent: $p.\text{own} \mathrel{+}= g.\text{sum}$
2. The parent's tournament entry strengthens
3. The parent may re-enter the active contour
4. The code shortens by one codeword

**Energy conservation.** The root's sum is unchanged by eviction. The absorbed mass moves from child contribution to parent's own — the two terms swap, the total is invariant. No mass is created or destroyed. The Kraft equality is preserved (one fewer cell at depth $d$, the parent covers the vacated range at depth $d-1$).

**Code contraction is the inverse of code lengthening.** Lengthening (split) allocates precision; eviction deallocates it. The two operations compose to create a self-regulating cycle where the code continuously reallocates its precision budget toward active regions.

### 6.5 Benchmark Compounding: The Code Hardens

Repeated lengthening–shortening cycles at a given code position compound the frozen benchmark:

$$B^{(k)} \approx B^{(k-1)} \cdot \alpha_k, \qquad \alpha_k > 1$$

After $k$ cycles, reaching depth $D$ along a root-to-leaf path costs:

$$\sum_{d=0}^{D} B_d \sim \frac{D^2 \cdot \theta}{2}$$

The cost grows quadratically with depth. Each additional level of precision costs linearly more. This is the code's **long-term memory**: it remembers that a region was previously explored and found wanting, and demands stronger evidence before re-investing.

Under temporal attenuation ($\lambda < 1$), benchmarks soften — the code forgives old judgments. Under amplification ($\lambda > 1$), benchmarks harden — the code becomes more demanding. Under annihilation ($\lambda = 0$), benchmarks reset — the code starts fresh.

### 6.6 Convergence to Fixed Point

Under sustained contraction (no new observations, static eviction threshold), the code converges to a fixed point:

**Potential function.** $\Phi = |G| + S$ where $S$ is the semi-internal count.

| Event                                   | $\Delta | G    | $    | $\Delta S$ | $\Delta\Phi$ |
| --------------------------------------- | ------- | ---- | ---- | ---------- | ------------ |
| Eviction (parent becomes semi-internal) | $-1$    | $+1$ | $0$  |
| Eviction (parent becomes terminal)      | $-1$    | $-1$ | $-2$ |
| Legacy promotion                        | $+1$    | $-1$ | $0$  |

$\Phi$ is weakly non-increasing. Starting $\Phi_0 \leq 2|G_0| - 1$, the system converges to a fixed point where no eviction-eligible codewords remain.

**Total contraction work (when P4 holds):** $O(|G_0| \cdot h_V)$ — linear in the initial code size times the tournament height. Each positive-importance eviction triggers at most $O(h_V)$ legacy promotions. Ghost evictions (zero importance) are $O(1)$ each via the identity element fast path.

---

## Chapter 7. Temporal Modulation: The Code's Time Constant

### 7.1 The Filter Contract

A **temporal filter** modifies importance and ledger accumulators throughout the code tree. The architecture provides a contract — a set of invariants that must be restored after the filter completes — and one built-in implementation. The filter is the user's temporal policy; the architecture adapts to whatever the filter produces.

### 7.2 The Built-In Three-Parameter Filter

The built-in filter scales accumulators by a depth-dependent factor:

$$\lambda(d) = \text{att}^{1 + q \cdot (2d_{\text{local}}/D - 1)}$$

Three parameters control the scaling:

| Parameter                   | Range         | Effect                                              |
| --------------------------- | ------------- | --------------------------------------------------- |
| $\text{att}$ (scale factor) | $[0, \infty)$ | Direction: annihilation, attenuation, amplification |
| $q$ (selectivity)           | $[0, 1]$      | Depth-dependence: uniform to fully selective        |
| root                        | G-subtree     | Spatial targeting                                   |

The per-depth exponent is log-linear, producing:

| Depth within subtree             | Factor             |
| -------------------------------- | ------------------ |
| Root ($d_{\text{local}} = 0$)    | $\text{att}^{1-q}$ |
| Midpoint                         | $\text{att}$       |
| Deepest ($d_{\text{local}} = D$) | $\text{att}^{1+q}$ |

**Q-invariance.** The selectivity $q$ is invariant under composition: $k$ ticks at $(\text{att}, q)$ equals one tick at $(\text{att}^k, q)$. This makes the filter bank commute with time discretisation — a property unique to the log-linear profile.

### 7.3 Four Temporal Regimes

| $\text{att}$ | Effect        | Code-theoretic meaning                                                                                  |
| ------------ | ------------- | ------------------------------------------------------------------------------------------------------- |
| $0$          | Annihilation  | Hard reset of codeword significances. Zeroed entries lose all competitive standing.                     |
| $(0, 1)$     | Attenuation   | Exponential forgetting. Old codewords become less significant. The code contracts to track the present. |
| $1$          | Identity      | No temporal policy. Raw accumulation. The code reflects lifetime significance.                          |
| $> 1$        | Amplification | Reinforcement. Surviving codewords become more significant. The code sharpens.                          |

**Annihilation ($\text{att} = 0$) with $q = 1$: the detail flush.** The convention $0^0 = 1$ preserves the subtree root while zeroing all descendants. The coarsest codeword in the targeted region survives; all refinement must be re-earned. This is the code-theoretic equivalent of resetting to the coarsest resolution while preserving the base-level measurement.

### 7.4 V-I3 Preservation Under Scaling

**Uniform scaling** ($q = 0$, floating-point): all importances scale by the same factor. The ordering is preserved: $a \leq b \implies \lambda a \leq \lambda b$ for $\lambda > 0$. V-I3 is preserved without rebalancing. At $\lambda = 0$: all values collapse to zero; $0 \leq 0$ everywhere. **No rebalancing needed.**

**Non-uniform scaling** ($q > 0$): different code depths receive different factors. An uncle at code depth 5 (strong attenuation) may weaken below a nephew at code depth 2 (mild attenuation). **V-I3 violations are expected.** The trailing rebalance restores the tournament.

**Integer types** under uniform scaling: floor-truncation produces different rounding patterns across siblings, potentially creating violations even under uniform $\lambda$. The violations are rare (arising only from rounding discrepancies) but not statically excludable.

### 7.5 The Code as a Spatiotemporal Filter Bank

When temporal modulation is applied, the code's state is a three-stage spatiotemporal filter:

1. **Analysis.** The code's depth structure defines spatial subbands. Each contour depth corresponds to a spatial resolution. Observations route to individual cells; sum propagation accounts for each observation at every coarser scale.

2. **Processing.** The tournament ranks entries across subbands. Temporal scaling adjusts subbands at depth-dependent rates: attenuation suppresses cold subbands; amplification boosts fine-scale subbands; annihilation resets them.

3. **Synthesis.** The PEWEI reconstruction (Part II) reassembles the filtered subbands into a spatial intensity function.

---

# Part II: The Code's Output

---

## Chapter 8. The Progressive Entropic-Wavelet Exposure Image

### 8.1 The PEWEI as Code Readout

The PEWEI is a static reading of the dynamic code's state. It is extracted by a single breadth-first walk of the V-Tree — not the G-Tree — producing layers in approximate significance order.

The V-Tree's competitive ranking determines the layer ordering. The G-Tree's spatial structure determines each entry's region, baseline, and total. The PEWEI reads both trees through their shared nodes.

### 8.2 Two Node Types

**Phase transition nodes** (internal/semi-internal G-nodes): codewords that have been refined. They carry:

| Field                  | Source          | Code-theoretic meaning                             |
| ---------------------- | --------------- | -------------------------------------------------- |
| Region $[l, r)$        | G-node interval | The symbol this codeword describes                 |
| Baseline $B$           | $g.\text{own}$  | Pre-refinement measurement (the frozen benchmark)  |
| Total $S$              | $g.\text{sum}$  | Total mass in this region including all refinement |
| Refinement $R = S - B$ | Derived         | Mass attributed to confirmed sub-scale structure   |
| Ratio $R/B$            | Derived         | Refinement relative to baseline                    |

**Terminal nodes** (zero-child G-nodes): codewords at the finest current resolution. They carry:

| Field           | Source                        | Code-theoretic meaning                                       |
| --------------- | ----------------------------- | ------------------------------------------------------------ |
| Region $[l, r)$ | G-node interval               | The symbol this codeword describes                           |
| Intensity $I$   | $g.\text{own} = g.\text{sum}$ | Direct measurement at the code's finest available resolution |

### 8.3 Progressive Reconstruction

The PEWEI supports truncation at any layer depth $k$. Reconstruction uses G-I1 ($g.\text{sum} = g.\text{own} + \sum_c c.\text{sum}$) to separate baselines from refinements:

- **Expanded node** (children visible at truncation depth): the baseline persists as uniform background. Children add finer structure additively.
- **Truncated node** (children not visible): the total $g.\text{sum}$ serves as a lump substitute. No spatial detail below this codeword.

This additive reconstruction preserves total energy exactly (by G-I1). Each additional layer refines the estimate by splitting coarse regions into sub-regions with confirmed structure.

**Replacement is wrong.** Stamping a parent's total and then overwriting with children's values loses the parent's baseline — the pre-refinement energy that persists as background. The additive property is a consequence of the catalytic split: the parent was not consumed, so its energy persists.

### 8.4 Self-Calibrating Baselines

Each frozen benchmark is a direct measurement at the moment the code decided to allocate more precision. Under a spatially uniform null hypothesis, the baseline is the expected energy in the region before spatial structure was resolved. The refinement $R = S - B$ is the energy attributed to confirmed sub-scale structure.

The baselines are:

- **Exact** (direct measurement, not an estimate)
- **Spatially adaptive** (each region has its own)
- **Scale-adaptive** (each refinement level has its own)
- **Zero-cost** (no separate calibration pass)
- **Self-referential** (deposited by the same observations they calibrate)

The self-referentiality is the trade-off: the same data determines both the signal structure and the calibration threshold. Under intensity-proportional noise models (Poisson, counting), the baseline $B$ is both the expected background and the natural noise scale. Under additive noise models (Gaussian), converting $B$ to a noise floor requires an external variance estimate.

### 8.5 The Wavelet Parallel

The G-Tree's decomposition is structurally parallel to wavelet analysis:

| Wavelet concept     | G-Tree structural parallel                  |
| ------------------- | ------------------------------------------- |
| Scaling coefficient | $g.\text{own}$ (pre-refinement measurement) |
| Detail coefficient  | Children's sum asymmetry ($S_L - S_R$)      |
| Zero coefficient    | Evicted children (no structure found)       |

The parallel is richer than exact: a G-Tree node carries three independent quantities ($g.\text{own}$, $S_L$, $S_R$) versus a wavelet node's two (scaling + detail). The additional degree of freedom separates pre-refinement energy from post-refinement energy — a distinction wavelets do not make.

### 8.6 The Coding Structure of the PEWEI

| Coding concept                   | PEWEI mechanism                            |
| -------------------------------- | ------------------------------------------ |
| Code structure                   | Kraft equality (contour partition)         |
| Codeword lengths                 | G-Tree depths                              |
| Decomposition coefficients       | $g.\text{own}$ at each scale               |
| Significance ordering            | V-Tree layer assignment ($\leq 1.44H + 1$) |
| Progressive truncation           | Breadth-first V-Tree walk                  |
| Zerotree / insignificant subtree | Evicted code region (contracted, absorbed) |
| Per-coefficient calibration      | Frozen baseline at split time              |

The PEWEI encodes signal, significance ordering, and calibration context in a single extraction pass at cost $O(|V|)$.

---

## Chapter 9. Plateaus: The Code's Structural Complexity

### 9.1 Definition

A **plateau** is a maximal contiguous run of the code's contour at a single depth. It is the natural unit of the code's spatial output — within a plateau, all codewords have the same length, meaning the code has judged the region to be uniform at this resolution.

### 9.2 The Plateau Count as Structural Entropy

The number of distinct plateaus $P$ measures the code's structural complexity. A uniform code has $P = 1$. A maximally fragmented code has $P \propto L$ (number of codewords).

The bound $P \leq |G|$ holds unconditionally. Under sustained contraction, the ceiling tightens monotonically: fewer codewords means fewer possible depth changes.

The plateau count is a direct measure of the code's departure from uniformity. In information-theoretic terms, it measures the **structural entropy** of the code — the complexity of the depth profile, independent of the values carried by the codewords.

### 9.3 Thatching

At semi-internal code boundaries, the parent plateau and child plateau overlap in spatial extent. The parent's basis element (the semi-internal node) has sum $g.\text{sum}$ that includes the child's contribution. This multi-counting is correct by design — each layer of thatch is real energy from a different epoch of the code's history:

- Pre-split energy (observations before the code refined)
- Post-split routing (observations in the vacated half)
- Absorbed energy (from evicted siblings)

Thatching is the code's manifestation of its multi-scale memory. The ordered map's basis edges tile the domain (no gaps); the thatching shows where the code's history layers overlap.

---

# Part III: Self-Composition

---

## Chapter 10. Why the Code Composes

### 10.1 The Key Observation

The code (G-Tree) maintains a dynamic Kraft-tight partition of $[0, 2^N)$, allocating precision toward observed mass, governed by a competitive tournament within $1.44\times$ of entropy-optimal.

**Nothing in this description mentions spatial coordinates.**

The code accepts integers in $[0, 2^N)$ as observation coordinates. It routes observations to the current codeword covering each integer. It accumulates mass, propagates sums, detects violations, resolves the tournament, and adjusts the codebook. At no point does it inspect the coordinate's semantic content.

The code works identically on:

- Physical spatial coordinates ($x$ ∈ $[0, 2^N)$)
- Ordinal positions in a learned vocabulary ($\text{ord}(x) \in [0, p)$)
- Z-interleaved rank–co-rank pairs ($\zeta(\hat{k}, \hat{j}) \in [0, 2^{N_z})$)
- Hilbert curve images ($\eta(x, y) \in [0, 2^{N_H})$)
- Any deterministic mapping to integers in the domain

The Kraft equality holds for any partition of any integer range. The Fibonacci bound holds for any tournament satisfying V-I3. The eviction convergence holds for any code with the three shields. **Every theorem in the specification holds under arbitrary coordinate substitution.**

This is why the composition sections in the original documents are brief. From the coding-theoretic perspective, feeding derived coordinates into the code is using a code as a code. There is nothing to prove. There is nothing to justify. The code doesn't know and doesn't care what the coordinates represent.

### 10.2 From Spatial Index to Universal Code

The reframing:

| "Spatial index" framing                | Coding-theoretic framing                            |
| -------------------------------------- | --------------------------------------------------- |
| The G-Tree indexes spatial coordinates | The code partitions an integer range                |
| Routing finds the cell containing $x$  | The code identifies the codeword for symbol $x$     |
| Refinement subdivides a spatial region | The code lengthens a codeword (adds precision)      |
| Eviction coarsens spatial resolution   | The code shortens a codeword (removes precision)    |
| The contour is a spatial step function | The codebook is the current set of codewords        |
| Plateau: contiguous same-depth region  | Codeword run: consecutive symbols at same precision |
| **Derived coordinates are special**    | **All coordinates are the same**                    |

The last row is the difference. Under the spatial framing, feeding non-spatial data into a spatial index requires justification. Under the coding framing, it's the default.

---

## Chapter 11. Factored Composition via Tensor-Product Z-Curves

### 11.1 The Problem

Given observations in a 2D domain $\Omega = [0, 2^{N_x}) \times [0, 2^{N_y})$, learn both the marginal structure (what spatial features exist along each axis independently) and the joint structure (which pairs of features interact significantly).

### 11.2 The Architecture

Three independent 1D codes:

$$\mathcal{G}_x \text{ over } [0, 2^{N_x}), \qquad \mathcal{G}_y \text{ over } [0, 2^{N_y}), \qquad \mathcal{G}_z \text{ over } [0, 2^{N_z})$$

The first two learn per-axis feature vocabularies. The third discovers significant feature interactions.

### 11.3 Double-Referenced Ordinals

For an element at position $k$ in a vocabulary of size $p$, define the **rank–co-rank pair**:

$$\hat{k} = (k, \; p - 1 - k)$$

The pair lies on the anti-diagonal $k + (p - 1 - k) = p - 1$. When the vocabulary changes:

| Mutation      | Positions $< m$              | Positions $> m$              |
| ------------- | ---------------------------- | ---------------------------- |
| Insert at $m$ | Rank unchanged, co-rank $+1$ | Rank $+1$, co-rank unchanged |
| Delete at $m$ | Rank unchanged, co-rank $-1$ | Rank $-1$, co-rank unchanged |

**Total cascade property:** Every vocabulary change of size 1 shifts every surviving element's double ordinal by $\pm 1$ in exactly one component. No element is unchanged. This is the key innovation: under single ordinals, positions below the mutation point are unaffected — creating mixed-version references. The rank–co-rank encoding guarantees that all references are invalidated simultaneously.

### 11.4 The Z-Interleave

Define the joint coordinate by 4-fold bit interleaving:

$$\zeta(\hat{k}, \hat{j}) = \bigoplus_{b \geq 0} \left(k_b \cdot 2^{4b+3} + (p_x - 1 - k)_b \cdot 2^{4b+2} + j_b \cdot 2^{4b+1} + (p_y - 1 - j)_b \cdot 2^{4b}\right)$$

The 4-fold interleave preserves Z-locality: a $\pm 1$ change in any component flips a low-order bit in that component's lane, producing bounded displacement in Z-space.

### 11.5 The Observation Protocol

On observation $(x, y, \Delta)$:

$$\mathcal{G}_x.\text{observe}(x, \Delta) \to k = \text{ord}_x(x), \quad p_x = |P_x|$$
$$\mathcal{G}_y.\text{observe}(y, \Delta) \to j = \text{ord}_y(y), \quad p_y = |P_y|$$
$$\mathcal{G}_z.\text{observe}\!\left(\zeta\bigl((k, p_x{-}1{-}k),\;(j, p_y{-}1{-}j)\bigr), \Delta\right)$$

Each tree is a standard unmodified code. Only the routing (coordinate computation) is novel.

### 11.6 Vocabulary Change as Benchmark Transfer

When $\mathcal{G}_x$ refines or contracts a plateau, $p_x$ changes by $\pm 1$. By the total cascade property, **every** double ordinal shifts. Every Z-address in $\mathcal{G}_z$ moves. Entries at pre-mutation addresses cease receiving observations — their importances freeze as competitive benchmarks. Fresh entries accumulate at corrected addresses from ground importance $\nu$.

**This is the catalytic split mechanism operating across codes.** The marginals' structural decisions become the joint code's admission criteria through ordinal instability alone. No explicit inter-code communication exists. The competitive mechanism is the transfer channel.

The instability is the point. It forces all joint entries to re-validate against the current vocabulary state. The V-Tree tournament resolves which re-validated pairings are genuinely significant (they outgrow their benchmarks through fresh observation) and which were artefacts of the old vocabulary (they never accumulate enough to promote). Self-prioritised healing: the most actively observed pairings heal first.

### 11.7 Properties

Every property of the base code holds for the joint code — because the joint code is a base code:

| Property              | Guarantee                                                       |
| --------------------- | --------------------------------------------------------------- | --- | ------------------------------------------ |
| Kraft equality        | $\sum 2^{-d_i} = 1$ over joint codewords                        |
| Fibonacci depth bound | $d_i \leq \log_\varphi(1/w_i) + 1$ for joint entries            |
| Sampling cost         | $\leq 1.44\,H_z + 1$ expected                                   |
| Eviction convergence  | $O(                                                             | G_z | _0 \cdot h_{V_z})$ total contraction work  |
| Energy conservation   | $\mathcal{G}_z.\text{root.sum}$ conserved through all mutations |
| Vocabulary coherence  | Total cascade → all live entries reference current vocabulary   |
| Benchmark locality    | $\pm 1$ displacement → Z-adjacent stale/fresh pairs             |
| Sparsity              | $                                                               | G_z | \leq \min(n\_{\text{obs}}, p_x \cdot p_y)$ |

---

## Chapter 12. Multi-View Composition

### 12.1 Complementary Views

The factored construction uses axis-aligned marginal codes ($\mathcal{G}_x$, $\mathcal{G}_y$). A Hilbert curve code $\mathcal{G}_H$ provides a complementary view:

| Structure in 2D             | $\mathcal{G}_z$ (axis-aligned) cost | $\mathcal{G}_H$ (Hilbert) cost            |
| --------------------------- | ----------------------------------- | ----------------------------------------- | --- | ------------------------------- |
| Axis-aligned rectangle      | $O(1)$                              | $O(\text{perimeter} / \text{resolution})$ |
| Rank-1 separable $f(x)g(y)$ | $O(\text{rank})$                    | $O(\text{support area})$                  |
| Diagonal line               | $O(p \cdot                          | \sin 2\alpha                              | )$  | $O(1)$ up to quadrant crossings |
| Compact convex region       | $O(p_x + p_y)$                      | $O(1)$                                    |

The two views have complementary representation costs. Structures that are simple in one view may be complex in the other, and vice versa.

### 12.2 The Five-Code Architecture

$$\boxed{(x,y,\Delta)} \xrightarrow{\text{marginals + Hilbert}} \underbrace{\mathcal{G}_x \otimes \mathcal{G}_y}_{\text{axis features}} \;\Big\|\; \underbrace{\mathcal{G}_H}_{\text{spatial features}} \xrightarrow{\zeta(\text{r-cr})} \underbrace{\mathcal{G}_z}_{\text{interactions}} \;\Big\|\; \mathcal{G}_H \xrightarrow{\zeta(\text{r-cr})} \underbrace{\mathcal{G}_\mu}_{\text{cross-view}}$$

Five independent 1D codes in two layers:

**Layer 1:** Three independent views of $\Omega$:

- $\mathcal{G}_x$: marginal $x$-structure
- $\mathcal{G}_y$: marginal $y$-structure
- $\mathcal{G}_H$: 2D spatial proximity structure (via Hilbert curve)

**Layer 2:** Two interaction-discovery codes:

- $\mathcal{G}_z$: significant axis-aligned feature interactions (from $\mathcal{G}_x \otimes \mathcal{G}_y$)
- $\mathcal{G}_\mu$: cross-view correspondences (from $\mathcal{G}_z \otimes \mathcal{G}_H$)

The meta-code $\mathcal{G}_\mu$ discovers which axis-aligned feature interactions correspond to which contiguous spatial structures. It provides connectivity (multiple $\mathcal{G}_z$-cells mapping to the same $\mathcal{G}_H$-cluster are fragments of one feature), decomposition (each spatial cluster is annotated with its axis-aligned interactions), and consistency (significance agreement across views increases confidence).

Each observation triggers five independent code updates. No code communicates with any other except through the shared observation's derived coordinates. The architecture is embarrassingly parallel at the per-observation level.

---

# Part IV: The Complete Picture

---

## Chapter 13. The Architecture as Rate-Distortion Optimiser

### 13.1 The Optimisation Objective

The G-V Graph continuously solves:

$$\min_{\mathcal{C}} D(\mathcal{C}, \mu) \quad \text{subject to} \quad |\mathcal{C}| \leq \text{budget}$$

where $\mathcal{C}$ is the codebook (contour), $\mu$ is the observed distribution, $D$ measures the spatial distortion of representing $\mu$ with codebook $\mathcal{C}$, and the budget is the node count ceiling.

The solution is approximate and dynamic:

- The V-Tree governs which codewords earn precision (code lengthening) and which lose it (eviction), producing a locally optimal allocation at each step.
- The Fibonacci bound guarantees the allocation is within $1.44\times$ of entropy-optimal for sampling.
- The budget mechanism (dynamic $D_{\text{evict}}$) provides a hard ceiling with graceful degradation.
- Temporal modulation controls the time constant of the optimisation — how quickly the code forgets old evidence and adapts to new.

### 13.2 The Separation Principle

The architecture cleanly separates three concerns:

**The code** (G-Tree): maintains the Kraft-tight partition, accounts for mass, propagates sums. Purely structural. Correct for any partition of any integer range.

**The policy** (V-Tree): ranks codewords by competitive importance, governs allocation and deallocation. Correct for any importance values satisfying the value space axioms.

**The temporal filter** (user-supplied): modulates the importance landscape over time. Correct for any filter satisfying the temporal contract.

These three concerns are mutually independent:

- The code works with any policy. (Different value spaces produce different allocation strategies; all result in valid codes.)
- The policy works with any code content. (The V-Tree never inspects coordinates, intervals, or ledger values.)
- The filter works with any code/policy state. (The filter modifies accumulators and lets the policy adapt.)

### 13.3 The Golden Ratio's Role

The golden ratio $\varphi$ appears in the architecture's **efficiency**, not its **structure**:

| Where $\varphi$ appears                                           | What it measures                           |
| ----------------------------------------------------------------- | ------------------------------------------ |
| Fibonacci depth bound ($\log_\varphi(1/w_i)$)                     | Maximum depth per unit of self-information |
| Expected sampling cost ($1.44H + 1$)                              | Policy overhead per unit of entropy        |
| Maximum information per step ($\log_2\varphi \approx 0.694$ bits) | Per-step routing efficiency                |
| Extremal configuration                                            | Fibonacci chain (degenerate 2-node path)   |

$\varphi$ lives in the policy — in how quickly the tournament identifies where the code's precision budget should next be spent. It does not appear in the code's structure (the Kraft equality holds regardless of $\varphi$) or in the code's content (the mass accumulated is independent of $\varphi$).

The Fibonacci recurrence $I_{k-2} \geq I_{k-1} + I_k$ is the tightest configuration the max-uncle constraint permits. The ratio of consecutive Fibonacci numbers converges to $\varphi$. Therefore $\log_2\varphi$ is the per-step entropy of the constraint's extremal case. Any local constraint with minimum branching factor 2 and the "no child beats all uncles" property produces the same recurrence and the same constant.

### 13.4 Cost Summary

| Operation                        | Cost                                                          | Code-theoretic meaning          |
| -------------------------------- | ------------------------------------------------------------- | ------------------------------- | ------------------- | ------------------------- |
| Observation (route + accumulate) | $O(d_{\text{geo}})$                                           | Route to codeword, update mass  |
| Tournament update                | $O(h_V)$                                                      | Update policy ranking           |
| Rebalance per observation        | $O(h_V)$ typical, $O(1)$ amortised under proportional traffic | Restore tournament invariant    |
| Code lengthening (split)         | $O(1)$ when P2+P3                                             | Create two new codewords        |
| Code shortening (eviction)       | $O(h_V)$                                                      | Remove one codeword             |
| Sampling                         | $O(1.44H + 1)$ expected                                       | Route through tournament        |
| Point query                      | $O(\log P)$                                                   | Plateau lookup                  |
| Range query                      | $O(N)$                                                        | Code-tree segment decomposition |
| PEWEI extraction                 | $O(                                                           | V                               | )$                  | Single breadth-first walk |
| Full contraction                 | $O(                                                           | G_0                             | \cdot h_V)$ with P4 | Drain all codewords       |

---

## Chapter 14. The Information-Theoretic Content

### 14.1 What the Code Stores

At any moment, the G-V Graph's state encodes:

1. **The current codebook** (contour): a Kraft-tight partition reflecting the code's current judgment about where precision should be allocated.

2. **The mass distribution** (G-node sums): how much observed mass each codeword region contains, at every scale from codeword to root.

3. **The significance ranking** (V-Tree structure): which codewords are most important, maintained within $1.44\times$ of entropy-optimal.

4. **The refinement history** (frozen benchmarks): at every scale where the code decided to allocate more precision, the pre-refinement measurement is preserved as a datum.

5. **The temporal state** (accumulated importances): the combined effect of past observations and past temporal modulations, determining each codeword's competitive position.

### 14.2 What the Code Produces

**Three orthogonal projections** of this state:

**The plateau map** (G-Tree projection): a spatially-ordered step function. Each plateau reports its depth (precision level), basis elements, and total energy. This is the code's learned shape — where it invested precision and how much mass it found.

**The PEWEI** (V-Tree projection): a significance-ordered layer sequence. Each layer contains the next-most-significant codewords with their baselines, totals, and refinements. This is the code's learned ranking — which structures are most important and at what scale.

**Proportional sampling** (V-Tree operation): an entropy-optimal random walk that reaches codeword $i$ with probability $w_i$ in expected cost $\leq 1.44H + 1$. This is the code's stochastic interface — attention allocation proportional to significance.

Neither projection subsumes the others. The plateau map is spatial but not significance-ordered. The PEWEI is significance-ordered but not spatial. Sampling is probabilistic. Three views of the same code.

### 14.3 What Makes It Universal

The code accepts any integers as coordinates. The policy works through any value space satisfying P0. The temporal filter is user-supplied. The projections work regardless of coordinate semantics.

Therefore:

- A single 1D code over physical coordinates is a spatial index.
- A single 1D code over Hilbert-transformed 2D coordinates is a 2D spatial index.
- A single 1D code over Z-interleaved rank–co-rank ordinals is a joint interaction discoverer.
- Multiple 1D codes composed through derived coordinates form a multi-view analyser.

In every case, the code maintains Kraft equality, the tournament provides $1.44\times$-efficient ranking, eviction converges, energy is conserved, and the PEWEI is extractable. The theorems don't change because the mathematics doesn't depend on coordinate semantics.

### 14.4 The Fundamental Identity

A binary partition over a dyadic domain is a prefix-free code. A dynamic binary partition governed by a competitive tournament is a dynamic code with near-optimal significance ordering. The Kraft equality is the structural invariant. The Fibonacci bound is the efficiency guarantee. The three shields are the mechanism that makes the dynamics stable. The PEWEI is the readout format. Self-composition is coordinate substitution.

Every component of the architecture is a consequence of taking the Kraft identity seriously — not as a theorem about codes, but as the definition of what the data structure IS.

$$\boxed{\sum_{i} 2^{-d_i} = 1}$$

The code is the partition. The partition is the code. Everything else follows.

---

# Appendix A. Proof Catalogue

---

## A.1 Kraft Preservation Under Code Mutation

**Theorem.** Every code mutation (split, eviction, restoration) preserves $\sum_i 2^{-d_i} = 1$.

**Sketch.** Split replaces one cell at depth $d$ with two at depth $d+1$: $\Delta K = -2^{-d} + 2\cdot 2^{-(d+1)} = 0$. Eviction and restoration are the reverse. The semi-internal intermediate (one child gone) still tiles correctly: uncovered half contributes $2^{-(d+1)}$ through the parent, surviving child contributes $2^{-(d+1)}$, total $= 2^{-d}$. Every mutation swaps contributions of equal weight. $\square$

---

## A.2 Fibonacci Depth Bound

**Theorem.** Under V-I3, V-I2, P1, P5: entry $i$ with weight $w_i$ has depth $d_i \leq \log_\varphi(1/w_i) + 1$.

**Sketch.** On the root-to-leaf path $v_0, \ldots, v_d$, write $I_k = v_k.\text{int}$. For $k \geq 2$: $v_{k-2}$ has $\geq 2$ children (V-I2), so $v_{k-1}$ has a sibling $u$ with $u.\text{int} \geq I_k$ (by V-I3). Non-negative importance (P1) ensures additional siblings don't decrease the sum. Hence $I_{k-2} \geq I_{k-1} + I_k$ — the Fibonacci recurrence backward from the leaf.

Normalise: $\alpha_j = I_{d-j}/I_d$, with $\alpha_0 = 1$, $\alpha_1 \geq 1$. Induction gives $\alpha_j \geq F_{j+1}$. At the root: $1/w_i = \alpha_d \geq F_{d+1} \geq \varphi^{d-1}$. Invert: $d \leq \log_\varphi(1/w_i) + 1$. $\square$

---

## A.3 Expected Sampling Cost

**Corollary.** $E[\text{cost}] \leq H/\log_2\varphi + 1 \approx 1.44H + 1$.

**Sketch.** Apply A.2 per entry: $d_i \leq \log_2(1/w_i)/\log_2\varphi + 1$. Weight by $w_i$ and sum: $\sum w_i d_i \leq H/\log_2\varphi + 1$. $\square$

---

## A.4 Tightness of the Leading Coefficient

**Theorem.** The coefficient $1/\log_2\varphi$ is achieved by a Fibonacci chain.

**Sketch.** Construct a 2-node chain with entry importances $I_k = C\varphi^{-k}$. Each uncle constraint is satisfied: $I_{k+2} = C\varphi^{-(k+2)} \leq C\varphi^{-k} = I_k$ since $\varphi > 1$. The Fibonacci recurrence holds with equality throughout: structural aggregates telescope via $1 + \varphi^{-1} = \varphi$. Weight fractions $w_k \propto \varphi^{-k}$ give $E[\text{cost}] = H/\log_2\varphi$ asymptotically. $\square$

---

## A.5 Violation-Free Catalytic Splits (P2 + P3)

**Theorem.** Under P2 ($\nu \preceq a$ for all $a$) and P3 ($\nu \oplus \nu = \nu$), catalytic splits create no V-I3 violations.

**Sketch.** New entries have importance $\nu$. By P2, $\nu \preceq u$ for every uncle $u$. No violation. The new structural node has importance $\nu \oplus \nu = \nu$ by P3 — same argument. Existing grandchildren's max uncle is unchanged or strengthened (the new sibling $s$ at importance $\nu$ can only raise the max, not lower it). $\square$

---

## A.6 Additive Collapse Theorem

**Theorem.** Under $\oplus = +$ on connected $I \subseteq \mathbb{R}$: (1) P3 ⟺ P4 ⟺ $\nu = 0$. (2) P1 ⟺ $I = [m,\infty)$, $m \geq 0$. (3) The unique P0–P5 space is $(\mathbb{R}_{\geq 0}, +, 0, \leq)$.

**Sketch.**

(1) P3: $\nu + \nu = \nu \implies \nu = 0$. P4: $\nu + a = a \implies \nu = 0$. Converses immediate.

(2) P1 gives minimum $m$. Closure: $m + m = 2m \in I$ requires $m \geq 0$. Unbounded above (else $a + a$ escapes for $a$ near sup). Connectedness yields $[m, \infty)$.

(3) P2 forces $\nu = m$ (minimum). P3 forces $\nu = 0$. Together: $m = 0$, $I = [0,\infty)$. P5 is free (addition is associative). $\square$

---

## A.7 Eviction Convergence

**Theorem.** Under no observations, static $D_{\text{evict}}$, and P4: total contraction work is $O(|G|_0 \cdot h_V)$.

**Sketch.** Classify events: PIE (positive-importance eviction), LP (legacy promotion), GE (ghost eviction).

- **(A)** PIE $\leq |G|_0 - 1$ (each destroys a distinct non-root node).
- **(B)** Ghost inertness (P4): absorbing $\nu$ is a no-op ($\nu \oplus a = a$). GE triggers zero violations, hence zero LPs. Per-GE cost: $O(1)$.
- **(C)** Each PIE's cascade traverses $\leq O(h_V)$ V-levels, triggering $\leq O(h_V)$ LPs.
- **(D)** GE $\leq$ LP (each ghost was created by exactly one LP).
- **(E)** No self-excitation: ghosts (B) cannot seed cascades. Every LP traces to a PIE.

Total: PIE $\cdot O(h_V)$ cost + LP at $O(1)$ each + GE at $O(1)$ each = $O(|G|_0 \cdot h_V)$.

Without P4: GE costs $O(h_V)$ each, total degrades to $O(|G|_0 \cdot h_V^2)$. $\square$

---

## A.8 Rebalance Termination

**Theorem.** The `rebalance()` loop terminates finitely.

**Sketch.**

**When P2 holds:** Define $\Phi = \sum_\ell \ell.\text{int}\cdot d(\ell)$ over entries. Importances fixed during rebalancing. Skip/legacy promote: entry rises one level, $\Delta\Phi \leq -c.\text{int} < 0$ (under P2, $\nu = 0$ contributes nothing for legacy heirs). Standard promote: $\Delta\Phi \leq 0$; escalation (§IDEA M-11.10) breaks cycles via skip promote with $\Delta\Phi < 0$. Contraction paired with promotion: net $\Delta\Phi \leq 0$. Since $\Phi \geq 0$ and strictly decreases on each promotion, termination follows.

**When P2 fails:** Legacy promotion may create entries with $\nu > 0$, so $\Phi$ may increase. Alternative: bound total operations by $(n + S) \cdot h_V$ — at most $n + S$ entries, each promoted at most $h_V$ times (depth bounded below by 0), $O(1)$ contractions per promotion. Finite. $\square$

---

## A.9 Plateau-Count Bound

**Proposition.** $P \leq |G|$.

**Sketch.** Each plateau has $\geq 1$ contour cell. Distinct plateaus have disjoint cells. Each cell has a unique backing G-node. Therefore $P \leq L \leq |G|$. $\square$

---

## A.10 Bounded Plateau Change Per Eviction

**Lemma.** $\Delta P \in \{-2, -1, 0, +1, +2\}$ per eviction.

**Sketch.** Eviction at depth $d$ changes the contour to $d-1$ at coordinates $[l, r)$. Two boundary points, each contributing $\Delta \in \{-1, 0, +1\}$ depending on whether the far-side depth equals $d$ (new transition: $+1$), $d-1$ (removed transition: $-1$), or neither (unchanged: $0$). Sum of two such contributions gives $\Delta P \in \{-2, \ldots, +2\}$. All five values are constructively achievable. $\square$

---

## A.11 Budget Invariant

**Invariant.** $|G| + S + 2 \leq G_{\max}$ at every call boundary.

**Sketch.** Per `observe()`: the catalytic split adds 2 G-nodes ($\Delta(|G|+S) = +2$). Legacy promotions: $\Delta|G| = +1$, $\Delta S = -1$, net $\Delta(|G|+S) = 0$. Evictions: $\Delta|G| = -1$, $\Delta S \in \{-1, +1\}$, net $\Delta(|G|+S) \leq 0$. So the only positive change is the split's $+2$. The $+2$ headroom in the invariant absorbs exactly one split. The soft trigger fires before headroom is exhausted; evictions restore it. Minimum $G_{\max} \geq 5$: initialization gives $1 + 0 + 2 = 3$; bootstrap split pushes to $3 + 0 + 2 = 5$. $\square$

---

## A.12 Semi-Internal Consumption Lemma

**Lemma.** During any trailing rebalance, legacy promotions $L \leq S$ (semi-internal count at start).

**Sketch.** Each LP consumes one semi-internal (converts it to internal by creating the missing child). No other rebalancing primitive (contraction, standard/skip promote) touches the G-Tree. No G-children are added or removed except by LP. The semi-internal pool can only shrink. Stock $S$, each draw costs 1, so $L \leq S$. $\square$

---

## A.13 Two-Path Coverage Lemma

**Lemma.** Steps 4 and 8 of eviction together detect every persisting V-I3 violation.

**Sketch.** Violations require either (a) self-importance increased past uncle, or (b) uncle decreased below self. Step 3's absorption increases importances along **Path 1** (parent → V-root); Step 4 walks Path 1 catching (a). Step 7's removal decreases importances along **Path 2** (removed entry's parent → V-root); Step 8's push functions walk Path 2 checking siblings' children, catching (b).

Off-path node $X$: own importance unchanged (not on either path). Uncle on Path 1 → strengthened → no violation. Uncle on Path 2 → weakened → $X$ adjacent to Path 2, caught by push functions. Uncle on neither → unchanged → no violation. Transient inflation between Steps 3–7 can only produce false positives (filtered by re-check), never false negatives. $\square$

---

## A.14 V-I3 Preservation Under Uniform Scaling

**Theorem.** Uniform $\lambda \geq 0$ on float types preserves V-I3 without rebalancing.

**Sketch.** For $\lambda > 0$: $a \leq b \implies \lambda a \leq \lambda b$. Every importance and aggregate scales by the same factor. Ordering preserved. For $\lambda = 0$: all values become 0; $0 \leq 0$ everywhere.

Fails for non-uniform scaling ($q > 0$): uncle at deep G-depth attenuates more than nephew at shallow G-depth, potentially inverting the ordering. Fails for integer uniform scaling: floor-truncation of aggregates $\sum\lfloor\lambda c_i\rfloor$ may differ from $\lfloor\lambda\sum c_i\rfloor$, creating violations from rounding discrepancies. $\square$

---

## A.15 Q-Invariance of the Built-In Filter

**Theorem.** $k$ applications of $(\text{att}, q)$ equal one application of $(\text{att}^k, q)$.

**Sketch.** The scaling factor is $\lambda(d) = \text{att}^{e(d)}$ where $e(d) = 1 + q(2d/D - 1)$. Then $\lambda(d)^k = \text{att}^{k\cdot e(d)} = (\text{att}^k)^{e(d)}$. The exponent profile $e(d)$ — and hence $q$ — is invariant under raising $\text{att}$ to a power. $\square$

---

## A.16 Total Cascade of Double Ordinals

**Theorem.** Every vocabulary change of size 1 shifts every surviving element's rank–co-rank pair by $\pm 1$ in exactly one component.

**Sketch.** Insert at position $m$ in vocabulary of size $p$. Element at position $k$:

- $k < m$: rank unchanged, co-rank $= (p-1-k) \to (p-k)$, change $(0, +1)$.
- $k \geq m$: rank $= k \to k+1$, co-rank $= (p-1-k) \to (p-1-k)$, change $(+1, 0)$.

Deletion symmetric with $-1$. Every element shifts in exactly one component. No element unchanged. Anti-diagonal shifts from $p-1$ to $p$ (insert) or $p-2$ (delete), consistent with vocabulary size change.

**Consequence:** All Z-addresses in the joint code shift, forcing universal freeze-and-recompete — no mixed-vocabulary references. $\square$

---

# Appendix B. The Information-Theoretic Invariants

These invariants hold throughout the system's lifecycle:

| Invariant                      | Statement                                                      | Origin                                    |
| ------------------------------ | -------------------------------------------------------------- | ----------------------------------------- | -------------------------- | ------------------------------------ |
| **Kraft Equality**             | $\sum_{\text{contour}} 2^{-d_i} = 1$                           | Structural: dyadic partition completeness |
| **Energy Conservation**        | Root sum = total observed mass                                 | G-I1: summation invariant                 |
| **Tournament Ordering**        | $c.\text{int} \leq \max\{u.\text{int}\}$ at all non-root nodes | V-I3: max-uncle constraint                |
| **Depth Bound**                | $d_i \leq \log_\varphi(1/w_i) + 1$ (when P1 + P5)              | Fibonacci recurrence from V-I3            |
| **Sampling Bound**             | $E[\text{cost}] \leq 1.44H + 1$ (when P1 + P5)                 | Corollary of depth bound                  |
| **Budget Ceiling**             | $                                                              | G                                         | + S + 2 \leq G\_{\max}$    | Per-observe accounting               |
| **Contraction Convergence**    | Fixed point reached in $O(                                     | G_0                                       | \cdot h_V)$ work (when P4) | Potential function + ghost inertness |
| **Compositional Universality** | All above hold under arbitrary coordinate substitution         | Code accepts any integer input            |

The first invariant is structural — it holds unconditionally for any complete dyadic partition. The next five are maintained by the dynamic mechanisms (three shields, tournament rebalancing, budget control). The last is the universality property that enables self-composition.

Together they constitute the complete information-theoretic content of the architecture: a dynamic prefix-free code with near-optimal governance, conserved energy, bounded resources, guaranteed convergence, and universal applicability.

---

_The code is the partition. The partition is the code. $\varphi$ lives in how efficiently the policy learns where to spend the code's precision budget. Everything else follows._
