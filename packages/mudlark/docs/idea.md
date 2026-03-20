# Formal Specification: Dual-Tree Value-Stratified Index

A φ-Bounded Geometric-Value Graph.

## Chapter 1. Overview

This chapter introduces the architecture at a conceptual level. It presents the two trees and their shared structure (§1.2), the measures each tree maintains and the rankings they produce (§1.3), a brief operational summary of the observation and importance machinery (§1.4), the governance mechanisms by which the two trees protect and drive each other (§1.5), the design principles that emerge from the architecture (§1.6), and the separation between architectural mechanism and user policy (§1.7). The algebraic framework that underpins the importance interface — the value space, its properties, and the features they enable — is developed in Chapter 2.

---

## 1.1 Conceptual Summary

The G-V Graph continuously solves a rate-distortion allocation problem. The geometric tree maintains a dynamic base-2 description code — a Kraft equality $\sum 2^{-d_i} = 1$ over the spatial domain, allocating precision toward observed structure, not away from it: the dual of source coding. The value tree governs this allocation through competitive ranking: the max-uncle constraint (§1.5.3) implies at least Fibonacci-rate decay along root-to-leaf paths, bounding the governance overhead at $1/\log_2\varphi \approx 1.44$ times Shannon entropy. The Fibonacci recurrence (§18.1) proves the bound tight (§18.3). The G-Tree is the code. The V-Tree is the policy that shapes it. $\varphi$ lives in the policy's efficiency — in how quickly the system identifies where the code's precision budget should next be spent — not in the code's structure.

---

## 1.2 The Two Trees

The structure consists of two trees sharing a common set of G-nodes. Each tree owns a different aspect of the system's state.

### 1.2.1 The Geometric Tree

The **Geometric Tree (G-Tree)** is a binary tree over dyadic intervals of $[0, 2^N)$. It is the **ledger**: every materialised node stores the accumulated observation value for its range since creation. Ledger values are of type $T$ — signed, complex, or any additive type the domain needs.

The G-Tree answers spatial queries. It maintains a **contour** — the observation-receiving surface of the domain — a step function whose depth at each coordinate reflects how finely the tree has resolved that region. The contour is the tree's external interface: stability guarantees, query results, and structural complexity measures (§5.6) are all stated in terms of contour cells, not individual nodes.

### 1.2.2 The Value Tree

The **Value Tree (V-Tree)** is a dynamic tree with branching factor 2 or 3, governed by the **max-uncle constraint**: no node may outrank every one of its uncles.

The V-Tree is a tournament bracket. G-nodes sit at the leaves as competitors; internal nodes are pure structural scaffolding. The V-Tree ranks by **importance** — values satisfying an opaque `Importance` interface (§1.4, Chapter 2). The V-Tree is blind to the ledger type $T$ and to the G-Tree's spatial topology; it sees only what the interface exposes: comparison, aggregation, and — when the importance type is non-negative — a bottom element that enables fast paths and the full analytical regime (§2.7).

High-importance entries reside near the root; low-importance entries are consolidated deeper.

### 1.2.3 Shared Structure

Both trees reference the same G-nodes. The G-Tree owns spatial structure and the ledger ($T$). The V-Tree owns attention structure and the importance ranking ($I$). A G-node exists simultaneously in both trees: it occupies a position in the G-Tree's dyadic hierarchy (determined by its interval) and a position in the V-Tree's tournament bracket (determined by its competitive importance).

---

## 1.3 Measures and Rankings

Every G-node carries three accumulators. The two trees see different ones and produce different rankings from them.

### 1.3.1 The Three Accumulators

| Measure               | Type         | Definition                                                          | Used by |
| --------------------- | ------------ | ------------------------------------------------------------------- | ------- |
| $g.\text{sum}$        | $T$          | $g.\text{own} + \sum_{\text{children}} c.\text{sum}$                | G-Tree  |
| $g.\text{own}$        | $T$          | Direct ledger accumulation at this node                             | G-Tree  |
| $g.\text{importance}$ | `Importance` | Accumulated via the projection function $\pi$ (see §1.4, Chapter 2) | V-Tree  |

### 1.3.2 The Spatial Ranking

The G-Tree ranks by sum — spatial containment. Under non-negative observations (the standard and absolute configurations), parents are always at least as heavy as children because they contain them: $g.\text{sum} = g.\text{own} + \sum c.\text{sum}$, and all terms are non-negative. The root carries the maximum sum. Under signed observations, $g.\text{own}$ may be negative, so the containment monotonicity $g.\text{sum} \geq c.\text{sum}$ can fail even though G-I1 still holds.

The G-Tree answers: _where is activity concentrated spatially?_

### 1.3.3 The Importance Ranking

The V-Tree ranks by importance — the user-defined signal exposed through the value space (Chapter 2) and projection $\pi$. Under the standard configuration (identity projection), importance equals own and the two rankings agree at terminal nodes. Under absolute projection, importance tracks cumulative $|\Delta|$ while own tracks signed accumulation — the two measures diverge even at leaves. For internal nodes, importance is frozen at its pre-split level while children accumulate fresh importance and eventually outrank the parent.

The V-Tree answers: _where is importance concentrated, and at what scale?_

### 1.3.4 The Inversion

For internal G-nodes, the two measures diverge over time. The node's own value is frozen at its pre-split level while its sum grows as descendants accumulate. Under non-negative importance, the ratio $g.\text{own}\,/\,g.\text{sum}$ trends monotonically toward zero:

$$\frac{g.\text{own}}{g.\text{sum}} \;\to\; 0 \qquad \text{as descendants accumulate (non-negative}\;\Delta\text{)}$$

The G-Tree considers the node increasingly important — it contains ever more activity. The V-Tree considers it increasingly negligible — a shrinking historical footnote. The root is the extreme case: highest possible sum (it contains everything), lowest eventual importance (frozen at the earliest, most ancient value). Under signed observations, $g.\text{sum}$ may oscillate rather than grow monotonically; the inversion tendency is the same but the convergence is not guaranteed.

For terminal nodes, the ledger measures are identical: $g.\text{own} = g.\text{sum}$. Under the standard configuration, importance also equals own. Leaves are where the two trees agree.

The analogy is structural rather than algebraic: `sum` aggregates downward (cumulative), `importance` measures contribution at a specific scale (local). One is the integral. The other is the derivative. The G-Tree says "how much has accumulated here and below." The V-Tree says "how much is arriving _at this scale specifically_."

### 1.3.5 Temporal Regimes

Under user-applied temporal scaling (§14), the V-Tree's ranking reflects the user's chosen temporal policy:

- **Attenuation** ($\text{att} < 1$) produces recency — "what matters now."
- **Amplification** ($\text{att} > 1$) with depth selectivity ($q > 0$) produces sharpening — fine-scale structure is reinforced relative to coarse.
- **Annihilation** ($\text{att} = 0$) produces a hard reset — zeroed entries lose all competitive standing.

Under raw accumulation (the default), the ranking reflects scale-specific historical significance — "what mattered most at each level of refinement." The architecture adapts to whichever regime the user's temporal filter produces.

---

## 1.4 Observations and Importance

This section provides the operational summary needed to follow the governance mechanisms of §1.5. The formal algebraic framework — the value space, its properties, and the features they enable — is developed in Chapter 2.

### 1.4.1 What Arrives

Observations are scalar values arriving at coordinates in the domain $[0, 2^N)$. Each observation carries a coordinate $x$ and a delta $\Delta$. Under the standard configuration, $\Delta$ is a non-negative real. Under the absolute configuration, $\Delta$ may be any real (importance tracks $|\Delta|$). Under the signed configuration, $\Delta$ may be any real (importance tracks $\Delta$ directly). The system does not interpret the semantic meaning of observations — it accumulates, compares, and aggregates them through an opaque interface.

### 1.4.2 What the System Does With It

The importance interface exposes four operations to the V-Tree:

| Operation     | Notation         | Intuition                                 | Standard example              |
| ------------- | ---------------- | ----------------------------------------- | ----------------------------- |
| **Aggregate** | $a \oplus b$     | Combine two importance values into one    | $3 + 5 = 8$                   |
| **Compare**   | $a \preceq b$    | Decide which value is more important      | $3 \leq 8$                    |
| **Project**   | $\pi(i, \Delta)$ | Update importance given a new observation | $\pi(i, \Delta) = i + \Delta$ |
| **Ground**    | $\nu$            | The importance a fresh entry starts with  | $0$                           |

These four operations, together with a set of algebraic axioms and optional properties, determine which architectural features are available: proportional sampling, violation-free splits, the Fibonacci depth bound, and others. Chapter 2 defines the axioms, proves the collapse theorem for additive value spaces, and establishes the feature table.

### 1.4.3 Quick Reference: Recommended Configurations

| Name         | Observations          | Ground | Importance equals          | All features?                        |
| ------------ | --------------------- | ------ | -------------------------- | ------------------------------------ |
| **Standard** | $\mathbb{R}_{\geq 0}$ | $0$    | $g.\text{own}$             | Yes                                  |
| **Absolute** | $\mathbb{R}$          | $0$    | $\sum\lvert\Delta_i\rvert$ | Yes                                  |
| **Signed**   | $\mathbb{R}$          | $0$    | $g.\text{own}$             | No — loses sampling, Fibonacci bound |

Chapter 2 (§2.5) proves that under ordinary addition on connected carriers, these three are exhaustive: every other additive configuration is either equivalent to one of these or strictly dominated by one.

---

## 1.5 Governance

The two trees govern each other through complementary protection mechanisms. This section presents the shields (§1.5.1), traces a G-node through its complete lifecycle under these shields (§1.5.2), and distils the competitive mechanism that drives the lifecycle (§1.5.3).

### 1.5.1 The Three Shields

| Direction           | Shield                                                      | What it protects                                                                                                 | Mechanism            |
| ------------------- | ----------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- | -------------------- |
| V-Tree → downward   | Parent's entry is uncle to children's entries               | Children from competitive reshuffling — they cannot be displaced as long as the parent's entry is a strong uncle | Max-uncle constraint |
| G-Tree → upward     | Children intercept observations meant for parent's range    | Parent's V-entry from growing — it stays frozen as a fixed benchmark children can eventually surpass             | Observation routing  |
| G-Tree → structural | Only unprotected G-nodes (0 children) are eviction-eligible | Protected G-nodes from premature removal — they cannot be evicted while structurally load-bearing                | Dependents check     |

**Without the upward shield (routing interception):** The parent's V-entry would grow with every observation to its range, keeping pace with its children. The frozen benchmark would not exist. Children could never outgrow their uncle. The competitive mechanism would be dead.

**Without the downward shield (uncle constraint):** Children's positions would be unstable. Every intensity fluctuation would cause restructuring. The V-Tree would thrash rather than settle into a stable tournament.

**Without the structural shield (dependents check):** Evicting an entry could destroy a G-node that supports other contour cells. The contour would tear — creating orphaned cells with no backing G-node. Instead, only unprotected G-nodes (zero children, no dependents) can be evicted. The contour contracts smoothly from the tips inward.

### 1.5.2 The G-Node Lifecycle

A G-node breathes through a full cycle. The three shields govern every transition.

```
    ON THE CONTOUR — fully exposed (no dependents)
        │                                                   ┄┄┄► DESTROYED
        │ catalytic split → 2 children created              (evicted: V-deep,
        ▼                                                    unprotected, §§12.3–12.5)
    ABOVE THE CONTOUR (not exposed, has dependents, frozen benchmark)
        │                              ↑
        │ one child evicted            │
        │ (absorb, partially exposed)  │
        ▼                              │
    ON THE CONTOUR — partially exposed (has one dependent)
        │                              │
        ├─── other child evicted ──► ON THE CONTOUR — fully exposed
        │         (absorb,                 (no dependents, cycle restarts
        │          no dependents)           — or evicted if V-deep)
        │
        └─── legacy promotion ─────► ABOVE THE CONTOUR
               (entry outgrew              (not exposed, has dependents,
                all uncles)                 frozen benchmark, cycle continues)
```

The dashed exit arrow represents eviction: a terminal node past the eviction depth threshold with no dependents is destroyed. The lifecycle is cyclic for nodes that remain competitively significant; eviction is the exit for nodes that do not.

#### 1.5.2.1 Creation (Catalytic Split)

When children are created (by refinement or restoration): the parent leaves the contour. It stops accumulating because children intercept all observations in the parent's range. The children join the contour at ground importance ($\nu$) and must earn their way up against a fixed bar. The parent is now eviction-immune.

The split is **catalytic**: the parent is not consumed. It persists as a V-entry carrying pre-split history, serving as the competitive benchmark its children must exceed. No information is destroyed by spatial refinement.

#### 1.5.2.2 Eviction and Absorption

When a child is evicted: the parent absorbs that child's accumulated value, partially re-joins the contour for the vacated range, and starts receiving observations there again. Its V-entry strengthens — which protects the surviving child with an even heavier uncle shield. If both children are gone, the parent fully re-joins the contour and becomes eviction-eligible once more.

#### 1.5.2.3 Restoration (Legacy Promotion)

When the parent earns legacy promotion: the parent's V-entry earned a competitive promotion by outgrowing every one of its uncles (a V-I3 violation). The promotion restores the missing child (§11.6), the parent goes back above the contour, and its entry freezes as a benchmark. The overlapping reinforcement dissolves — both halves are now self-defending.

The V-Tree's competitive mechanism is the gate — no separate operation or threshold is needed. The contour grows through competition.

### 1.5.3 The Competitive Mechanism

The max-uncle constraint encodes competitive dominance. When a G-node splits, its V-entry freezes (the G-Tree's upward shield stops observations from reaching it). The frozen entry becomes uncle to its children's entries. The constraint says: **a child cannot outrank the frozen benchmark without triggering promotion.** The child starts at importance $\nu$, accumulates through observations, and must earn its way up against a fixed bar.

When the non-negative importance property holds (P1, which on the standard carrier ensures non-negative importance; see §2.3), this is a monotone climb — the child can only grow. The uncle constraint can only be _newly_ violated by the child growing past the frozen uncle, never by the uncle shrinking (it is frozen). The moment the child exceeds every uncle, the V-Tree restructures — which is exactly the moment the child region has proven more significant than the parent region was at split time.

Three siblings of comparable importance under a 3-node parent coexist without violations indefinitely — a violation requires beating _both_ uncles. Under a 2-node parent, the bar is lower: a single uncle must be exceeded. The 3-node stability is a direct consequence of the "max over uncles" formulation, which makes the V-Tree permissive of local balance. The V-Tree restructures only when a node dramatically outgrows its entire neighbourhood, not on every minor importance fluctuation.

---

## 1.6 Design Principles

The following principles are individually named for cross-reference throughout the specification. Each captures a structural commitment that shapes the architecture.

### 1.6.1 Catalytic Creation

When a G-node splits, it is not consumed. It persists as a V-entry carrying pre-split history, serving as the competitive benchmark its children must exceed. No information is destroyed by spatial refinement.

### 1.6.2 Dual Shielding

The V-Tree shields downward: a parent's frozen entry stands as an uncle, stabilising its children's positions. The G-Tree shields upward: children intercept observations, freezing the parent's V-entry at its pre-split value. The G-Tree also shields structurally: only unprotected G-nodes (no dependents) are eviction-eligible, so the contour contracts from the tips inward, never leaving orphans. Each tree's protection enables the other tree's dynamics.

### 1.6.3 Tournament Structure

The V-Tree is a bracket. G-node entries are competitors at the leaves. Structural internal nodes are freely created and destroyed with no external consequences. The decision that internal V-nodes carry no external identity is what allows rebalancing to restructure the tournament without side effects.

### 1.6.4 Single-Entry Accounting

Each observation updates exactly one V-entry's importance — the receiving node's. The V-Tree's total importance is determined by the value space and projection; under the standard configuration it equals the G-Tree root's sum, under absolute projection the total absolute observation volume $\sum |\Delta_i|$. No double-counting, clean accounting.

### 1.6.5 Max-Uncle Stability

The heaviest uncle acts as a shield. Three siblings of comparable importance coexist without violations. The V-Tree restructures only when a node outgrows its entire neighbourhood.

### 1.6.6 Contour as Interface

The G-Tree's bottom contour is the observation-receiving surface — the tree's external interface. Refinement adds resolution to the contour. Eviction removes it. Nodes are the implementation; the contour is what the tree exposes. Stability guarantees are stated in terms of contour cells, not individual nodes.

### 1.6.7 Legacy Promotion

When a semi-internal entry earns competitive promotion, the promotion itself restores the missing child. The V-Tree's competitive mechanism is the gate — no separate operation or threshold is needed. The contour grows through competition.

### 1.6.8 Bottom-Up Contraction

Eviction removes only unprotected contour cells — G-nodes with zero children. Parents absorb the evicted value and may themselves join the contour, eligible for eviction in the next round. The contour coarsens through gradual tip-smoothing, never catastrophic subtree removal.

---

## 1.7 Separation of Concerns

### 1.7.1 Mechanism vs. Policy

The architecture provides two mechanisms. The user provides two policies. The four are mutually independent:

| Concern               | Mechanism                                                      | Responsibility                                |
| --------------------- | -------------------------------------------------------------- | --------------------------------------------- |
| Value space           | $(I, \oplus, \nu, \preceq)$ with P0 required (Chapter 2)       | **User**                                      |
| Projection            | $\pi : (I, T) \to I$, carrier-preserving (§2.6)                | **User**                                      |
| Property satisfaction | P1–P5, checkable at construction time (§2.3)                   | **User** declares; **Architecture** validates |
| Feature availability  | Derived from properties (§2.4)                                 | **Architecture**                              |
| Spatial structure     | Depth-gated catalytic splits (G-Tree)                          | **Architecture**                              |
| Competitive ranking   | Uncle constraint + promotion/contraction (V-Tree)              | **Architecture**                              |
| Temporal semantics    | User-supplied filter on values (exact accumulation by default) | **User**                                      |

The architecture provides scaffolding. The user plugs in meaning. A user who wants "what matters now" applies decay. A user who wants "most significant ever" does nothing. The V-Tree adapts to whatever the numbers say.

### 1.7.2 Terminology Conventions

The following terms are used consistently throughout this specification:

| Term                       | Meaning                                                               |
| -------------------------- | --------------------------------------------------------------------- |
| **Value space**            | The quadruple $(I, \oplus, \nu, \preceq)$ (Chapter 2)                 |
| **Carrier**                | The set $I$                                                           |
| **Aggregation**            | The operation $\oplus$                                                |
| **Ground**                 | The distinguished element $\nu$ (`New()`)                             |
| **Projection**             | The function $\pi : I \times T \to I$ (`accumulate_importance`)       |
| **Property tag**           | One of P0–P5 (defined in §2.3)                                        |
| **Feature**                | A capability of the architecture enabled by one or more property tags |
| **Configuration**          | A specific $\{$value space, projection$\}$ pair                       |
| **Standard configuration** | $(\mathbb{R}_{\geq 0}, +, 0, \leq)$ with identity projection          |
| **Absolute configuration** | $(\mathbb{R}_{\geq 0}, +, 0, \leq)$ with absolute projection          |
| **Signed configuration**   | $(\mathbb{R}, +, 0, \leq)$ with identity projection                   |

> _Note._ The standard and absolute configurations share the same value space and differ only in projection. Informally, when the distinction is unimportant, "standard modes" refers to both. Where precision matters, the specific configuration name is used.

## Chapter 2. The Value Space

The V-Tree requires its ranking values to inhabit an algebraic structure with specific properties. This chapter defines the structure (§2.1), its base axioms (§2.2), the optional properties that enable specific features (§2.3), the feature table those properties unlock (§2.4), the collapse theorem that simplifies the property landscape under ordinary addition (§2.5), the projection function that bridges ledger values to governance importance (§2.6), the recommended configurations for common use cases (§2.7), and the binding that connects the abstract framework to concrete G-node fields (§2.8).

Features are enabled by properties of the algebraic structure, not by membership in a named tier. The remainder of this specification develops the additive case fully and provides the general framework for non-additive cases. Sections that specialise to the additive case note this.

---

## 2.1 The Importance Interface

### 2.1.1 Formal Definition

The importance interface is expressed as a quadruple:

$$\mathcal{V} = (I,\; \oplus,\; \nu,\; \preceq)$$

where:

| Component | Name        | Type                                  |
| --------- | ----------- | ------------------------------------- |
| $I$       | Carrier     | A set of importance values            |
| $\oplus$  | Aggregation | A binary operation $I \times I \to I$ |
| $\nu$     | Ground      | A distinguished element of $I$        |
| $\preceq$ | Ordering    | A binary relation on $I$              |

The carrier is the type of importance values. The aggregation combines children's importances into a parent's structural aggregate. The ground is the importance assigned to freshly created entries. The ordering governs competitive comparisons — uncle constraints, violation detection, and proportional sampling.

### 2.1.2 Design Intent

The value space is the user's declaration of what "mattering" means. The architecture never interprets importance semantically — it only aggregates, compares, and routes based on the interface. A user who cares about raw intensity uses identity projection on non-negative reals. A user who cares about activity regardless of sign uses absolute projection. A user with a domain-specific notion of significance defines a custom value space and projection.

The interface is opaque: the V-Tree accesses importance through $\oplus$, $\preceq$, and $\nu$ without knowing the concrete type. The G-Tree stores the importance accumulator on each G-node; the V-Tree holds a reference to it (§2.8). Updates happen at the G-node via the projection function (§2.6); the V-entry sees them through the shared reference.

---

## 2.2 Base Axioms (P0)

The base axioms are **always required**. Together they constitute property **P0** — the floor of the property hierarchy. Any value space satisfying P0 supports the core architecture: governance, rebalancing, promotion, eviction.

### 2.2.1 Axiom Summary

| Axiom             | Statement                                            | Purpose                         |
| ----------------- | ---------------------------------------------------- | ------------------------------- |
| **Closure**       | $\forall\, a, b \in I:\; a \oplus b \in I$           | Aggregation is type-correct     |
| **Well-typing**   | $\nu \in I$                                          | Ground is in the carrier        |
| **Commutativity** | $\forall\, a, b \in I:\; a \oplus b = b \oplus a$    | Sibling order irrelevant        |
| **Compatibility** | $a \preceq b \implies a \oplus c \preceq b \oplus c$ | Governance respects aggregation |
| **Totality**      | $\preceq$ is a total order on $I$                    | `Ord` requirement               |

P0 is the floor. Nothing beyond P0 is assumed by the structural algorithms (rebalancing, violation detection, contraction, promotion). All features beyond basic governance require additional properties (§2.3).

### 2.2.2 Closure and Well-Typing

Closure ensures that aggregating two importance values produces a valid importance value. Without it, V-I1 (structural sum invariant) cannot be stated — the parent's aggregate might not inhabit the carrier.

Well-typing ensures that freshly created entries carry a valid importance. Without it, initialization and catalytic split (§10.2) would produce entries with values outside the carrier.

Under ordinary addition on a half-line $[m, \infty)$, closure requires $m \geq 0$: if $m < 0$, then $m + m = 2m < m \notin [m, \infty)$. This is the concrete constraint that rules out carriers like $[-1, \infty)$ under addition.

> _Note._ Closure is stated as an explicit axiom rather than derived from the carrier definition. Under ordinary addition on $[m, \infty)$, closure requires $m \geq 0$ — a constraint implementors must verify.

### 2.2.3 Commutativity

**Commutativity is a hard structural requirement.** Rebalancing merges sibling V-nodes with $a \oplus b$ where the argument order depends on rotation direction — an internal detail invisible to the user. Without commutativity, the same logical merge produces different values depending on which rotation the architecture chose.

Concretely: contraction (§11.3) merges two children of a 3-node parent. The architecture chooses which child appears as the left argument and which as the right. This choice is determined by implementation details (child list ordering, rotation direction). Commutativity guarantees that the result is independent of this choice.

### 2.2.4 Compatibility

**Compatibility is the unique axiom binding the ordering to the aggregation.** The V-Tree uses ordering for governance (uncle comparisons) and aggregation for structural sums. Compatibility ensures coherence: if $a$ is less important than $b$, then $a$ combined with any $c$ is less important than $b$ combined with the same $c$.

$$a \preceq b \implies a \oplus c \preceq b \oplus c$$

Without compatibility, tournament semantics — aggregating children's importances and comparing the aggregate against an uncle — are unsound. A subtree with individually light children could aggregate into something heavier than an uncle, yet the individual children would not be in violation. Compatibility prevents this incoherence.

### 2.2.5 Total Ordering

The ordering $\preceq$ must be a total order: reflexive, antisymmetric, transitive, and total (every pair of elements is comparable). This is the standard `Ord` requirement. Partial orders with incomparable elements would make violation detection (§11.2) non-deterministic: `is_violated` could encounter a pair where neither $c.\text{int} \preceq u.\text{int}$ nor $u.\text{int} \preceq c.\text{int}$ holds, leaving the violation status undefined.

---

## 2.3 Optional Properties (P1–P5)

Properties P1–P5 are optional, individually checkable properties of the value space. Each enables specific features (§2.4). They form a partial order of logical dependencies, not a linear chain.

### 2.3.1 Property Definitions

```
P0  Closure + Commutativity + Compatibility + Totality + Well-typing
│
├── P1  Bounded Below       ∃ ⊥ ∈ I : ∀ x ∈ I, ⊥ ⪯ x
│
├── P2  Grounded            ∀ a ∈ I : ν ⪯ a              (implies P1)
│
├── P3  Idempotent Ground   ν ⊕ ν = ν         (independent of P1, P2)
│
├── P4  Identity Element    ∀ a ∈ I : ν ⊕ a = a
│                                          (implies P3; independent of P1, P2)
│
└── P5  Associativity       ∀ a,b,c ∈ I : (a ⊕ b) ⊕ c = a ⊕ (b ⊕ c)
                                           (independent of P1–P4)
```

All five optional properties are direct children of P0. Two internal implications exist: P2 $\implies$ P1 and P4 $\implies$ P3. No other implication holds in the general (non-additive) case.

Informally:

- **P1** says the carrier has a floor — a value that nothing can be less important than.
- **P2** says the ground _is_ the floor — fresh entries start at minimum importance.
- **P3** says combining two fresh entries produces another fresh entry ($\nu \oplus \nu = \nu$).
- **P4** says the ground is invisible under aggregation — adding a fresh entry to anything leaves it unchanged.
- **P5** says aggregation is associative — parenthesisation doesn't matter.

### 2.3.2 Logical Relationships

The two internal implications:

- **P4 $\implies$ P3.** Substitute $a = \nu$ in $\nu \oplus a = a$: $\nu \oplus \nu = \nu$.
- **P2 $\implies$ P1.** If $\nu \preceq a$ for all $a \in I$, then $\nu$ is a lower bound: take $\bot = \nu$.

The following non-implications are established by counterexample:

**P1 $\not\implies$ P2.** $([0, \infty),\; +,\; 1,\; \leq)$: $\bot = 0$ exists (P1 holds), but $\nu = 1 \not\preceq 0$ (P2 fails).

**P2 $\not\implies$ P3.** $([1, \infty),\; +,\; 1,\; \leq)$: $\nu = 1 \leq a$ for all $a \geq 1$ (P2 holds), but $1 + 1 = 2 \neq 1$ (P3 fails).

**P3 $\not\implies$ P2 (in general).** $(\mathbb{R},\; +,\; 0,\; \leq)$: $0 + 0 = 0$ (P3 holds), but $0 \not\preceq -1$ (P2 fails).

**P3, P4, P5 are each independent of P1 and P2.** The signed configuration $(\mathbb{R},\; +,\; 0,\; \leq)$ demonstrates: P3 holds ($0 + 0 = 0$), P4 holds ($0 + a = a$), P5 holds (addition is associative), but P1 fails ($\mathbb{R}$ has no minimum) and P2 fails ($0 \not\preceq -1$).

**P5 is independent of P1–P4 in both directions.** A non-associative operation can satisfy P1–P4, and an associative operation can lack a lower bound.

> _Note (additive collapse)._ Under ordinary addition on connected carriers, the independence structure collapses dramatically: P3 $\iff$ P4 $\iff$ $\nu = 0$, and P5 is free. Given P1, P3 additionally forces P2. The full collapse is developed in §2.5.

### 2.3.3 Axiom Classification

The property hierarchy separates into structural and semantic groups:

**Structural axioms (P0, P1, P2).** Hard requirements for specific features. The architecture produces incorrect internal state without them for the features they enable.

**Semantic axioms (P4, P5).** The architecture operates correctly without them, but the user's accounting loses its expected meaning:

- **P4 (Identity).** If $\nu \oplus a \neq a$, sums drift on eviction: the parent absorbs a child's value, the child is removed, and the missing slot contributes $\nu$ to the recomputed sum. No crash — the system faithfully tracks the non-conserving arithmetic.

- **P5 (Associativity).** The architecture nests sums in a fixed convention. If associativity fails, the result is deterministic — the user simply needs to understand the convention. The semantic meaning of "total importance" is the user's concern. Additionally, without P5, structural rearrangements (contraction, promotion) that regroup children under a parent require explicit sum propagation to maintain V-I1. With P5, the regrouped sum is algebraically identical and no propagation is needed — a concrete performance benefit.

**Bridge property (P3).** P3 (idempotent ground) is structural for violation-free splits (§10.4) and semantic for sum conservation.

---

## 2.4 The Feature Table

| Feature                                         | Enabling Property | Mechanism                                                                                       |
| ----------------------------------------------- | ----------------- | ----------------------------------------------------------------------------------------------- |
| Governance (compare, rebalance, promote, evict) | P0                | Base architecture                                                                               |
| Proportional sampling                           | P1                | Non-negative ratios; $\bot$ gives zero-weight baseline                                          |
| Fibonacci depth bound ($\log_\varphi$)          | P1 + P5           | Proof requires non-negative values and associative telescoping                                  |
| Ghost eviction fast path                        | P4                | $\nu \oplus a = a$ — absorption is a no-op; Steps 3–4 of §12.5 skipped                          |
| Violation-free insertion                        | P2                | New entry $\preceq$ all existing entries                                                        |
| Violation-free structural nodes after split     | P3                | $\nu \oplus \nu = \nu$ so new structural node = ground                                          |
| Sum-propagation early termination               | P4                | Adding $\nu$ is a no-op; propagation can stop                                                   |
| Propagation-free structural rearrangement       | P5                | Contraction and promotion regroup children; associativity ensures the parent's sum is unchanged |
| Correct multi-level structural aggregation      | P5                | Parenthesisation of sums irrelevant                                                             |

Features are unlockable, not tier-gated. An implementation can check which properties its value space satisfies and enable the corresponding fast paths. No runtime dispatch — the checks are resolvable at construction time (or compile time for monomorphic instantiations).

> _Note (propagation-free rearrangement)._ Contraction (§11.3) takes a 3-node parent with children $\{h, a, b\}$ and produces a 2-node with children $\{h, s\}$ where $s.\text{int} = a \oplus b$. The parent's old aggregate was $(h \oplus a) \oplus b$ (under some parenthesisation); the new aggregate is $h \oplus (a \oplus b)$. These are equal iff P5 holds. Standard promote and skip promote involve analogous regroupings. Without P5, the rearranged parent's stored sum diverges from its recomputed-from-children sum, and `propagate_v_sums_from(p)` must be called to restore V-I1. With P5, the stored sum is already correct and the $O(h_V)$ propagation is avoided. The pseudocode in §§11.3–11.5 comments "no propagation needed" — **this claim is conditional on P5.** Implementations supporting non-associative value spaces must add the propagation call.

> _Note (ghost detection vs. ghost fast path)._ A **ghost** is an entry whose importance equals $\nu$. Under P2, $\nu$ is the minimum element of $I$, so a ghost has the lowest possible importance. Under P4, absorbing a ghost's value is a no-op ($\nu \oplus a = a$), enabling the fast eviction path. P2 provides a **semantic** signal: "importance equals $\nu$" means "at minimum, never observed." P4 provides the **computational** fast path. Under ordinary addition on $[0, \infty)$, P2 implies P4 (P2 forces $\nu = 0$ on this carrier, giving P4), so the distinction does not arise. On other carriers (e.g., $[1, \infty)$ with $\nu = 1$), P2 holds without P4 — ghosts are detectable but the fast path is unavailable. For general value spaces, P4 is the enabling property; P2 adds interpretive clarity but is neither necessary nor sufficient for the fast path.

---

## 2.5 Collapse Under Ordinary Addition

When $\oplus$ is ordinary addition on a connected subset of $\mathbb{R}$, the five optional properties lose most of their independence. This section proves the collapse and extracts the consequences.

### 2.5.1 The Additive Collapse Theorem

**Theorem.** Let $\oplus = +$ (ordinary addition) on a connected subset $I \subseteq \mathbb{R}$. Then:

1. P3 $\iff$ P4 $\iff$ $\nu = 0$
2. P1 $\iff$ $I = [m, \infty)$ for some $m \geq 0$
3. P2 $\iff$ $I = [\nu, \infty)$ with $\nu \geq 0$
4. P2 + P3 $\iff$ $I = [0, \infty)$ and $\nu = 0$
5. P1 through P5 all hold if and only if $\mathcal{V} = (\mathbb{R}_{\geq 0},\; +,\; 0,\; \leq)$

_Proof._

**(1)** P3: $\nu + \nu = \nu \implies \nu = 0$. Conversely, $0 + 0 = 0$. P4: $\nu + a = a$ for all $a \implies \nu = 0$. Conversely, $0 + a = a$.

**(2)** ($\Rightarrow$) Closure requires $I$ closed under $+$. If $I$ has minimum $m$: $m + m = 2m \in I$ requires $2m \geq m$, hence $m \geq 0$. If $I$ had a finite supremum $M$, then for $a$ close to $M$, $a + a > M \notin I$, contradicting closure. So $I = [m, \infty)$. ($\Leftarrow$) $[m, \infty)$ with $m \geq 0$ is closed under $+$ and has minimum $m$. (The degenerate singleton $\{0\}$ also satisfies P0 and P1 under $+$ but is operationally vacuous — §8.1's nonzero precondition rejects $\Delta = 0$, so no observation can be recorded. The theorem assumes a non-degenerate carrier with more than one element.) $\square$

**(3)** ($\Rightarrow$) P2 says $\nu \leq a$ for all $a \in I$, so $\nu$ is the minimum of $I$. By (2), $I = [\nu, \infty)$ with $\nu \geq 0$. ($\Leftarrow$) If $I = [\nu, \infty)$, then $\nu \leq a$ for all $a \in I$. $\square$

**(4)** ($\Rightarrow$) By (3), P2 gives $I = [\nu, \infty)$. By (1), P3 gives $\nu = 0$. So $I = [0, \infty)$. ($\Leftarrow$) $\nu = 0$ on $[0, \infty)$ gives P2 ($0 \leq a$ for all $a \geq 0$) and P3 ($0 + 0 = 0$). $\square$

**(5)** (4) gives the carrier and ground. P5 is free (ordinary addition is always associative). P4 follows from (1) since $\nu = 0$. P1 follows from P2. $\square$

> _Note (P2 does not imply P3)._ $([1, \infty),\; +,\; 1,\; \leq)$ satisfies P2 ($\nu = 1 \leq a$ for all $a \geq 1$) but fails P3 ($1 + 1 = 2 \neq 1$). This configuration has properties $\{\text{P0, P1, P2, P5}\}$ — strictly dominated by the standard configuration, which additionally satisfies P3 and P4. The corrected statement of the P2 + P3 relationship is part (4): P2 + P3 $\iff$ $I = [0, \infty)$ and $\nu = 0$.

### 2.5.2 Corollaries

**Corollary (Uniqueness of the Standard Configuration).** Under ordinary addition on a connected subset of $\mathbb{R}$, the unique value space satisfying P0–P5 is $(\mathbb{R}_{\geq 0},\; +,\; 0,\; \leq)$. The distinguished element $\nu = 0$ is simultaneously the least element, the additive identity, and the unique idempotent of $+$.

**Corollary (Practical Reduction).** Under ordinary addition, the five properties reduce to a binary question: _is the value space $(\mathbb{R}_{\geq 0},\; +,\; 0,\; \leq)$ or not?\_ If yes, all features are available. If no, the user has either a non-zero ground (losing P3–P4 features), an above-minimum ground (losing P2 features), no minimum at all (losing P1 features), or some combination thereof.

**Corollary (Zero-Centrality).** Under ordinary addition, the ground element $\nu$ must equal $0$ for P3 or P4 to hold — regardless of the carrier. Given P1 (carrier $= [m, \infty)$), $\nu = 0$ additionally forces $m = 0$ (by well-typing $\nu \in I$) and thereby implies P2. No value of $\nu$ other than $0$ admits P3 or P4.

Note that $\nu \neq 0$ does not reduce the value space to P0 and P5 alone. The configuration $([1, \infty),\; +,\; 1,\; \leq)$ satisfies P0, P1, P2, and P5 with $\nu = 1$; it loses only P3 and P4. The configuration $([0, \infty),\; +,\; 1,\; \leq)$ satisfies P0, P1, and P5 with $\nu = 1$; it loses P2, P3, and P4.

**Corollary (Additive Dependency Chain).** Under ordinary addition on connected carriers, the implications between P1–P4 form a strict chain with P5 orthogonal:

$$\text{P4} \;\longleftrightarrow\; \text{P3} \;\Longrightarrow\; \text{P2}\;(\text{given P1}) \;\Longrightarrow\; \text{P1}$$

No reverse arrow holds. On the specific carrier $[0, \infty)$, P2 additionally implies P3 (since P2 forces $\nu = 0$ on this carrier), tightening the chain to P2 $\iff$ P3 $\iff$ P4 with P1 always holding. This tighter equivalence is specific to $[0, \infty)$ and does not generalise to arbitrary connected carriers.

### 2.5.3 Exhaustiveness

The collapse theorem implies that under ordinary addition, the choice of value space is tightly constrained. Every additive configuration on a connected carrier is characterised by two parameters: the carrier minimum $m$ and the ground $\nu$. The standard configuration ($m = 0$, $\nu = 0$) satisfies all properties. Any other choice loses at least one. §2.7.4 develops the dominance argument: the three recommended configurations (§2.7.1–2.7.3) are the only non-dominated choices.

---

## 2.6 The Projection Function

### 2.6.1 Definition and Role

The projection is part of the value space's interface:

$$\pi : I \times T \to I$$

where $T$ is the ledger type. This is the user's declaration of what "mattering" means. The architecture calls $\pi$ exactly once per observation, on the receiving entry. The result replaces the entry's current importance:

$$g.\text{importance} \leftarrow \pi(g.\text{importance},\; \Delta)$$

**The projection is a construction-time binding**, not a runtime parameter. It is associated with the value space, not stored as a field on the graph. Changing the projection after the first observation invalidates the semantic relationship between all stored importance accumulators and the ledger — a construction-time constraint, not a dynamic one.

The projection must preserve the carrier: $\pi(i, \delta) \in I$ for all valid $i, \delta$. Under identity projection, this requires non-negative $\delta$ (to keep $i + \delta \geq 0$ on the carrier $[0, \infty)$). Under absolute projection, this is automatic ($|\delta| \geq 0$ for all $\delta$).

### 2.6.2 Identity vs. Absolute Projection

| Name     | Definition                    | Accepts               | Notes                                     |
| -------- | ----------------------------- | --------------------- | ----------------------------------------- | ------------ | -------------------------------------------------------- |
| Identity | $\pi(i, \delta) = i + \delta$ | Non-negative $\delta$ | Importance = own accumulation. Monotonic. |
| Absolute | $\pi(i, \delta) = i +         | \delta                | $                                         | Any $\delta$ | Importance = cumulative absolute activity. Always grows. |

Under identity projection, `g.importance` and `g.own` are numerically identical at all times for terminal nodes. A single storage field suffices for both (§2.8).

Under absolute projection, `g.importance` and `g.own` diverge as soon as a negative $\delta$ arrives. `g.own` may decrease (net-negative accumulation); `g.importance` can only grow (absolute activity always adds). The V-Tree ranks by activity volume, not net signed value.

---

## 2.7 Recommended Configurations

### 2.7.1 Standard (Non-Negative, Identity)

```
T = ℝ≥0,   I = ℝ≥0,   ν = 0,   ⊕ = +,   π(i, Δ) = i + Δ
```

_Properties:_ P0–P5 all hold (§2.5.1 part 5). All features available.

Importance equals `g.own`. The two accumulators are redundant — a single field suffices. When all deltas are non-negative, closure is satisfied by construction and $\nu = 0$ is the bottom element.

This is the default configuration. It is used throughout §16 (Worked Example) and serves as the reference case for most algorithmic discussions.

### 2.7.2 Absolute (Non-Negative, Absolute)

```
T = ℝ,     I = ℝ≥0,   ν = 0,   ⊕ = +,   π(i, Δ) = i + |Δ|
```

_Properties:_ P0–P5 all hold. Same value space as standard; different projection.

The ledger is signed; importance tracks cumulative absolute activity. `g.importance` diverges from `g.own` as soon as a negative $\Delta$ arrives. Closure is satisfied unconditionally ($|\Delta|$ is always non-negative) and $\nu = 0$ is the bottom element. This configuration retains **all features** despite mixed-sign observations.

### 2.7.3 Signed (All Reals, Identity)

```
T = ℝ,     I = ℝ,     ν = 0,   ⊕ = +,   π(i, Δ) = i + Δ
```

_Properties:_ P0, P3, P4, P5. P1 fails ($\mathbb{R}$ has no minimum element). P2 fails ($0 \not\preceq -1$). P3 holds ($0 + 0 = 0$). P4 holds ($0 + a = a$). P5 holds (ordinary addition is associative). These properties are independent of P1 under the general hierarchy (§2.3.2) — the signed configuration demonstrates their independence.

Both types are signed. Governance (P0) works: the V-Tree can compare, aggregate, and rebalance. What is **unavailable** when P1 fails:

- **Sampling** is undefined — no bottom element, so probability ratios are meaningless.
- **The Fibonacci depth bound** (§18.1) does not hold. Depth may reach $O(L)$ under adversarial sign patterns.
- **Violation-free insertion** is not guaranteed — new entries at importance $\nu = 0$ may exceed existing entries with negative importance.

What **is** available despite P1 failure:

- **The ghost eviction fast path** (§12.5 Step 2) — P4 holds ($0 + a = a$), so absorption of a ghost's importance is a no-op.
- **Violation-free structural nodes after split** — P3 holds ($0 + 0 = 0$), so the new structural node carries ground importance and cannot exceed any uncle.
- **Sum-propagation early termination** — P4 holds, so adding $\nu$ is a no-op and propagation can stop.
- **Propagation-free structural rearrangement** — P5 holds, so regrouped sums are exact.

This is a legitimate configuration when the user wants spatial adaptation driven by net signed value but does not need proportional sampling.

### 2.7.4 Dominance and Equivalence

These three are exhaustive under ordinary addition in the following sense: every other additive configuration on a connected carrier is either equivalent to one of these or strictly dominated by one.

**Dominated examples:**

- $([1, \infty),\; +,\; 1,\; \leq)$ has $\{\text{P0, P1, P2, P5}\}$ — strictly dominated by Standard, which adds P3 and P4.
- $([0, \infty),\; +,\; \nu,\; \leq)$ with $\nu > 0$ has $\{\text{P0, P1, P5}\}$ — also dominated by Standard.

**Configuration comparison:**

| Property                      | Standard | Absolute |   Signed    |
| ----------------------------- | :------: | :------: | :---------: |
| Properties                    |  P0–P5   |  P0–P5   | P0,P3,P4,P5 |
| Sampling [P1]                 |    ✓     |    ✓     |      ✗      |
| Fibonacci bound [P1+P5]       |    ✓     |    ✓     |      ✗      |
| Ghost fast path [P4]          |    ✓     |    ✓     |      ✓      |
| Violation-free splits [P2+P3] |    ✓     |    ✓     |      ✗      |
| Mixed-sign observations       |    ✗     |    ✓     |      ✓      |
| Importance = own              |    ✓     |    ✗     |      ✓      |

All configurations use the same G-Tree routing and sum propagation. Only the projection differs. All V-Tree structural machinery (rebalancing, uncle constraint, structural aggregation) depends only on P0, which all configurations provide.

> _Note (violation-free splits)._ The "Violation-free splits" row requires **both** P2 (new entries at importance $\nu$ cannot exceed any uncle, since $\nu \preceq a$ for all $a$) **and** P3 ($\nu \oplus \nu = \nu$, so the new structural node carries ground importance). Signed has P3 but not P2: the structural node is violation-free, but new entries may exceed existing entries with negative importance. The overall split is not violation-free.

> _Design note (the separation)._ The ledger type $T$ and the importance type are intentionally distinct concerns. The G-Tree stores $T$ — signed, complex, or vector-valued, whatever the domain needs. The V-Tree sees only importance through the opaque interface: ordered, aggregatable, optionally non-negative. The projection $\pi$ is the user's declaration of what "mattering" means — specifically, how the user projects ledger data into the governance signal. Under the standard configuration with non-negative deltas, the two types coincide and the distinction is invisible. It becomes meaningful when deltas carry mixed signs: the user chooses whether to project via absolute value (absolute configuration, retaining P0–P5) or identity (signed configuration, accepting the loss of P1–P2 features).

---

## 2.8 Binding to G-Nodes

### 2.8.1 The Three Accumulator Fields

The G-node carries an importance accumulator implementing the value space interface. The V-entry references it — the V-Tree accesses importance through the opaque interface, never inspecting the concrete type or touching the ledger.

| Field                 | Type | Updated by                               | Seen through         |
| --------------------- | ---- | ---------------------------------------- | -------------------- |
| $g.\text{sum}$        | $T$  | G-Tree sum propagation (§8.3.4)          | G-Tree range queries |
| $g.\text{own}$        | $T$  | Observation accumulation (§8.3.2)        | G-Tree range queries |
| $g.\text{importance}$ | $I$  | Projection $\pi$ at observation (§8.3.3) | V-Tree governance    |

Under the standard configuration (identity projection), `g.importance` and `g.own` are numerically identical at all times for terminal nodes. An implementation may use a single storage field for both, provided the V-entry's reference (§2.8.2) resolves to the same storage.

Under absolute projection, `g.importance` and `g.own` are independent fields with independent update paths. Both must be maintained and both must be scaled in place by temporal filters (§14.1).

Updates happen at the G-node via $\pi(g.\text{importance}, \Delta)$ whenever the node receives an observation; the V-entry sees them because it holds a reference. Different configurations (§2.7) pair different projections with the same or different value spaces, producing different property profiles and different semantic relationships between the ledger and the importance signal.

### 2.8.2 Invariants

The binding between the two trees is captured by a single invariant (formally numbered as G-I4; restated in §5.4 alongside the other G-Tree invariants):

$$\textbf{G-I4 (Importance Reference):}\quad g.\text{entry} \neq \text{null} \implies g.\text{entry.int}\ \text{references}\ g.\text{importance}$$

G-I4 connects the two trees. A V-entry's importance is not a separate bookkeeping register — it is a reference to the backing G-node's importance accumulator. The V-Tree accesses it through the opaque `Importance` interface without knowing the concrete type. Updates happen at the G-node via the projection; the V-entry sees them because it holds a reference.

**Reference semantics (normative).** `v.int` is a **reference** to `v.gnode.importance`, not an independent copy. When the projection updates the G-node's importance accumulator during observation (§8.3.3), the V-entry sees the new value immediately because it holds a reference to the same storage. Implementations must ensure reference identity — e.g., a pointer, a shared cell, or structural arrangement where the V-entry reads directly from the G-node's field. Copying the value at construction time and maintaining it independently would silently violate G-I4 whenever the importance accumulator is updated.

_Invariant checking._ G-I4 is structural — the reference identity is checked, not numeric equality. An implementation may verify the reference at debug time. Under the standard configuration, `g.importance` also equals `g.own` (since aggregation is addition and the starting values coincide). Under absolute projection, `g.importance` diverges from `g.own` (absolute vs. signed accumulation), but the reference is still valid.

When the node is internal (both children present), the importance accumulator is frozen — a direct consequence of observation routing (§5.2). No observations reach the internal node; its importance does not change. When the node splits, its children start at $\nu$; when a child is evicted, the parent absorbs the child's importance via $p.\text{importance} \leftarrow p.\text{importance} \oplus g.\text{importance}$ (§8.7).

---

## Chapter 3. Domain

Fix an integer $N \geq 1$. The domain is $[0, 2^N)$, a one-dimensional
half-open interval. The coordinate type $C$ must be capable of
representing every value in the domain; that is, $N$ must not exceed the
representable bit-width of $C$. This is a construction-time constraint:
an $N$ that overflows $C$ is rejected before the first observation.

The domain is required to be a power-of-two interval starting at zero.
This is a structural requirement of dyadic bisection: every subdivision
produces two half-width children whose widths remain exact powers of two.
Arbitrary ranges $[a, b)$ can be handled by embedding into the smallest
containing dyadic interval $[0, 2^{\lceil\log_2(b - a)\rceil})$ and
translating coordinates at the interface boundary. The unused portion of
the domain remains structurally valid but receives no observations.

This specification describes the one-dimensional case. Multi-dimensional
domains are out of scope.

The system begins with a single G-node covering the entire domain. It grows
only where the V-Tree authorizes spatial refinement.

### 3.1 Coordinate Space Properties

The coordinate type $C$ is the spatial type threading through every
G-node interval, every routing comparison, and every range query.
It is independent of the ledger type $T$ and the importance type —
the coordinate space and the value space never interact
algebraically. The sole bridge is pro-rating during range queries,
where an interval width in $C$ is converted to a ratio.

A correct coordinate type must satisfy seven structural properties.
These are not separate operations — they are consequences of the
type's arithmetic and are relied upon by the G-Tree's invariants:

1. **Dyadic closure.** The midpoint of an interval $[a, b)$ must
   fall strictly between $a$ and $b$ when the interval is
   divisible. The split guard verifies this; if it fails, the tree
   cannot subdivide further.

2. **Width–depth coherence.** For dyadic intervals, the width must
   be a power of two and the depth computation
   $N - \log_2(\text{width})$ must yield an integer. The G-Tree
   derives depth from width — incoherent widths produce incorrect
   depth assignments.

3. **Total ordering.** All coordinate values must be totally ordered
   (reflexive, antisymmetric, transitive, total). The ordered map
   underlying the plateau representation (§5.6) depends on
   this — partially ordered types with incomparable elements would
   violate its key invariants.

4. **Domain spanning.** The root interval $[0, 2^N)$ must be
   representable and must cover the valid coordinate range. The root
   G-node is initialised with these bounds; all descendant intervals
   are subsets by construction.

5. **Midpoint determinism.** Midpoint computation must be
   deterministic and pure. The G-Tree does not store midpoints — it
   recomputes them from $(l, r)$ on every routing step.
   Non-deterministic midpoints would cause observations to route
   inconsistently.

6. **Non-negative domain.** The domain starts at zero. The spec
   defines it as $[0, 2^N)$; negative coordinates have no meaning
   in the dyadic framework.

7. **Finality.** Every interval must eventually become indivisible
   under repeated bisection. For integer types, finality is
   natural: subdivision terminates when the interval width reaches
   one (the unit cell). For floating-point types, finality is
   artificial: subdivision terminates when the G-Tree depth of the
   interval equals $N$. The finality predicate
   $\text{is\_final}(l, r, d, N)$ encodes both cases uniformly.
   Without finality, the split guard (§10.1) could not prevent
   unbounded recursion.

### 3.2 Integer and Floating-Point Domains

Coordinate types fall into two families with subtly different
semantics:

| Property       | Integer                                                          | Floating-point                                                                    |
| -------------- | ---------------------------------------------------------------- | --------------------------------------------------------------------------------- |
| Domain maximum | $2^N$ (or the type's maximum when $N$ equals the full bit-width) | $2^N$ as a floating-point value                                                   |
| Midpoint       | $a + (b - a) / 2$ (integer division, overflow-safe)              | $a + (b - a) / 2$                                                                 |
| Finality       | Subdivision terminates naturally when $r - l = 1$ (unit cell)    | Subdivision terminates artificially when depth equals $N$                         |
| Successor      | Well-defined ($x + 1$)                                           | Not meaningful in the dyadic context                                              |
| Total ordering | Natively totally ordered                                         | Partially ordered; a separate total-order comparator is required for ordered maps |
| NaN            | Impossible                                                       | Must be explicitly rejected at every entry point before routing or querying       |

For integer coordinates, the domain decomposes cleanly: every dyadic
interval has power-of-two width, midpoints are exact, and recursion
terminates at unit cells. For floating-point coordinates, the
domain is continuous; recursion is bounded by the depth parameter $N$
rather than by reaching an indivisible interval. Floating-point
coordinates enable real-valued domains but introduce five additional
concerns that require explicit treatment.

#### 3.2.1 The Split Guard and Finality

The split guard (§10.1) must decide whether an interval can be
subdivided. The universal guard is: **compute the midpoint $m$ of
$[l, r)$ and verify that $m$ is strictly greater than $l$.** If
$m = l$, the interval is indivisible — further subdivision would
produce a degenerate zero-width child — and the attempt is
abandoned.

For integers, this guard is equivalent to checking $r - l > 1$: the
midpoint of a unit cell $[k, k+1)$ equals $k$ under integer
division, so the guard naturally catches unit cells. For
floating-point coordinates, the midpoint guard is strictly necessary
because finality is depth-gated rather than width-gated (the deeper
motivation for depth-gating in float domains is developed in §3.2.2).
The predicate $\text{is\_final}(l, r, d, N)$ returns true when
$\text{depth} \geq N$, but the midpoint guard provides an
independent safety net: even if a floating-point interval has not
reached depth $N$, rounding may cause $a + (b - a)/2 = a$ for very
narrow intervals, preventing degenerate splits.

The two guards cooperate:

- The **finality predicate** prevents the tree from exceeding the
  configured maximum depth $N$. For integers, it fires at unit
  cells ($r - l = 1$). For floating-point types, it fires at
  $\text{depth} = N$.
- The **midpoint guard** prevents degenerate splits where the
  computed midpoint collapses to an endpoint due to arithmetic
  limitations (integer unit cells or floating-point rounding).

In practice, the depth-gated finality predicate fires first for
floating-point domains (well before the midpoint collapses), and the
midpoint guard fires first for integer domains (at the unit cell).
Both guards must be present: the finality predicate alone does not
protect against midpoint collapse at depths below $N$; the midpoint
guard alone does not enforce the configured depth ceiling.

#### 3.2.2 The Role of $N$ in Floating-Point Domains

For integer coordinates, $N$ defines the domain $[0, 2^N)$ and
implicitly the maximum tree depth: at most $N$ bisections are
possible before reaching unit cells. For floating-point coordinates,
$N$ serves a dual role:

1. **Domain extent.** The domain is $[0.0,\, 2^N)$, a closed–open
   interval of floating-point values.
2. **Maximum resolution.** $N$ bounds the maximum G-Tree depth.
   Without this bound, floating-point bisection could continue
   for hundreds of levels (limited only by the floating-point
   mantissa), producing a tree far deeper than useful. The depth
   parameter $N$ is the user's declaration of how many levels of
   spatial resolution are meaningful for the application.

The finality predicate enforces both roles: a node at depth $N$
cannot be subdivided, regardless of whether the floating-point
interval could be further bisected. This prevents runaway
refinement in floating-point domains and gives the user explicit
control over the resolution–memory trade-off.

Choosing $N$ for a floating-point domain requires balancing
resolution against resource cost. Each additional level of depth
doubles the potential node count. Practical guidance:

- For most applications, $N$ between 16 and 32 provides ample
  resolution. $N = 20$ partitions $[0, 2^{20})$ into cells of
  width $\approx 1$, sufficient for most spatial indexing tasks.
- Extreme values of $N$ (e.g., 52 for 64-bit floats) are legal
  but rarely useful — the budget mechanism (§7.1) prevents the
  tree from materializing more than a small fraction of the
  $2^N$ potential leaf cells, but the depth bound still governs
  the maximum per-path cost of routing and range queries.

#### 3.2.3 Midpoint Precision

The midpoint formula $a + (b - a) / 2$ is chosen for numerical
stability. The naive formula $(a + b) / 2$ risks overflow for both
integer and floating-point types when $a + b$ exceeds the type's
representable range. For integers, the sum wraps or saturates; for
floats, it produces infinity, and $\infty / 2 = \infty$, corrupting
the result. The stabilized form computes the difference first, halves
it, then adds back — avoiding overflow because $b - a$ is the
interval width (always representable for dyadic intervals within the
domain) and halving cannot overflow.

For floating-point coordinates, this formula satisfies the dyadic
closure property (§3.1 Property 1) for all power-of-two-width
intervals $[k \cdot 2^{-d},\, (k+1) \cdot 2^{-d})$ within the IEEE
754 normal range. The midpoint is exactly representable because
division by 2 is an exact operation in binary floating point
(exponent decrement, no rounding). The midpoint is also
deterministic — no dependence on rounding mode or platform. For the
dyadic domain $[0, 2^N)$ with $N \geq 0$, all interval widths are
$\geq 2^0 = 1$ at the finest level, so subnormal values never arise
and the normal-range qualification is always satisfied in practice.

Width–depth coherence (§3.1 Property 2) also holds: widths of
dyadic intervals are exact powers of two, and $\log_2(2^k)$ returns
an exact integer in IEEE 754 arithmetic for all representable $k$.

#### 3.2.4 NaN Rejection

Floating-point types admit NaN (Not-a-Number) values. NaN violates
the total ordering property (§3.1 Property 3): NaN is not less
than, greater than, or equal to any value, including itself. If a
NaN coordinate entered the routing logic, every comparison would
fail, producing undefined routing behaviour.

NaN is rejected at every entry point that accepts a coordinate:

- **Observation:** NaN coordinates are rejected before routing
  (Step 1 of §8.2).
- **Point query:** NaN coordinates are rejected before routing.
- **Range query:** NaN bounds are rejected before bound resolution.

Rejection is immediate and unconditional — a programming error, not
a domain condition. The rejection mechanism is a hard failure (e.g.,
panic, assertion, exception), not silent clamping or substitution.
NaN in a coordinate is never meaningful; silent handling would mask
bugs.

Infinity ($+\infty$, $-\infty$) does not require special rejection.
Infinite coordinates participate in ordering normally and are
handled by domain clamping (§3.2.5).

#### 3.2.5 Domain Clamping

Out-of-domain coordinates — values less than zero or greater than or
equal to $2^N$ — are clamped to the domain rather than rejected:

- Coordinates below zero are clamped to zero.
- Coordinates at or above $2^N$ are clamped to $2^N$.

The clamped upper value $2^N$ is technically outside the half-open
domain $[0, 2^N)$. This is intentional: for integer coordinates,
clamping to $2^N - 1$ would land within the domain, but for
floating-point coordinates there is no clean "last value before
$2^N$" in the open interval. The universal clamping target $2^N$
handles both type families uniformly. Routing treats the clamped
value correctly: $x < m$ / else branching sends $x = 2^N$ rightward
at every level, landing in the rightmost terminal — a well-defined
and consistent result, identical to routing $2^N - 1$ for integers
or $\text{nextDown}(2^N)$ for floats.

Clamping is applied at observation and query entry points, before
routing. For integer coordinates, out-of-domain values are unusual
(the domain typically spans the type's full range). For
floating-point coordinates, domain clamping is a practical
convenience — it allows edge values to be queried without requiring
the caller to pre-clamp.

#### 3.2.6 Range Query Bound Resolution

Range queries accept bounds of three kinds: included, excluded, and
unbounded. Converting excluded bounds to the half-open form $[a, b)$
used internally requires a successor operation: an excluded start
bound $a$ becomes included start $a + 1$; an included end bound $b$
becomes excluded end $b + 1$.

For integer coordinates, the successor is well-defined: $x + 1$.
For floating-point coordinates, the successor operation is **not
meaningful** in the dyadic context. A floating-point range query
with an excluded start or included end cannot be converted to
half-open form by incrementing. Instead, floating-point range
queries should use only the bound kinds that do not require a
successor: included-start and excluded-end (the natural half-open
form $[a, b)$), or unbounded. If an excluded start or included end
is supplied with a floating-point coordinate, the operation must
fail immediately rather than silently produce an incorrect range.

This is not a limitation of the data structure — it is a property
of floating-point arithmetic. There is no "next floating-point
value" that is meaningful as a dyadic boundary; the next
representable float is not generally a dyadic rational and would
break the bisection invariants.

> _Implementation note._ Where the caller's intent is genuinely "all
> values strictly greater than $a$" or "all values up to and
> including $b$," the IEEE 754 `nextUp` operation can be applied
> _at the caller's discretion_ before invoking the range query with
> included-start / excluded-end bounds. This is semantically sound
> for routing purposes — the G-Tree routes by $x < m$ / else
> branching, so the exact position of the bound within a contour
> cell does not affect which cells are visited, only whether a
> partial-overlap pro-ration at the boundary cell shifts by one ULP.
> The conversion is the caller's responsibility, not the
> architecture's: the data structure accepts only $[a, b)$ form and
> rejects bound kinds that would require an implicit successor.

#### 3.2.7 Ordered Map Keys

The plateau representation (§5.6) stores contour edges as keys in
an ordered map. Ordered maps require their key type to be totally
ordered. Floating-point types are only partially ordered (NaN is
incomparable with all values). The plateau key type must therefore
provide a total-order comparator that extends the partial order to
a total order.

The standard approach is the IEEE 754 `totalOrder` predicate, which
defines a deterministic total order over all floating-point bit
patterns, including NaN (sorted after $+\infty$), signed zeros
($-0 < +0$), and subnormals. This comparator is used exclusively
for map key ordering — it does not affect the spatial semantics of
coordinates (which use the standard partial order for routing).

Since NaN coordinates are rejected at every entry point (§3.2.4),
NaN values never appear as map keys in practice. The total-order
comparator is a structural safeguard, not a runtime concern.

---

## Chapter 4. Node Types

### 4.1 G-Node

A G-node $g$ carries:

| Field                            | Type            | Description                                                                                              |
| -------------------------------- | --------------- | -------------------------------------------------------------------------------------------------------- |
| $g.l,\ g.r$                      | $C$             | Dyadic range $[l, r)$, width $r - l = 2^k$; $C$ is the coordinate type satisfying the properties of §3.1 |
| $g.\text{sum}$                   | $T$             | Total ledger value: own accumulation plus children's sums                                                |
| $g.\text{own}$                   | $T$             | Direct ledger accumulation at this node (observations received here)                                     |
| $g.\text{importance}$            | $I$             | Ranking accumulator — initialized to $\nu$ at G-node creation, updated via $\oplus$ and $\pi$ (§8.6)     |
| $g.\text{left},\ g.\text{right}$ | G-Node or null  | Children (null if absent)                                                                                |
| $g.\text{geo\_parent}$           | G-Node or null  | Parent in G-Tree                                                                                         |
| $g.\text{entry}$                 | V-Entry or null | V-Tree membership token                                                                                  |

A G-node exists in one of three states:

| Children | State         | Contour relationship                   | Observation behavior                    | Can be refined         | Can be restored            | Can be evicted          |
| -------- | ------------- | -------------------------------------- | --------------------------------------- | ---------------------- | -------------------------- | ----------------------- |
| 0        | Terminal      | **On the contour** — fully exposed     | Receives all observations in $[l, r)$   | Yes (catalytic split)  | No (nothing to restore)    | **Yes** (no dependents) |
| 1        | Semi-internal | **On the contour** — partially exposed | Receives observations in uncovered half | No (not fully exposed) | Yes (via legacy promotion) | No (has dependents)     |
| 2        | Internal      | **Above the contour** — not exposed    | Receives no observations                | No (not exposed)       | No (nothing missing)       | No (has dependents)     |

The following predicates formalize the three states and the two
orthogonal properties — exposure and dependents — that govern
operations throughout the specification.

```
has_dependents(g) → bool:
    return g.left ≠ null or g.right ≠ null
```

```
uncovered_range(g) → interval or null:
    if g.left = null and g.right = null:
        return [g.l, g.r)              // terminal: entire range is exposed
    m ← g.l + (g.r − g.l) / 2         // stabilized midpoint (§3.2.3)
    if g.left = null:
        return [g.l, m)                // semi-internal: left half exposed
    if g.right = null:
        return [m, g.r)               // semi-internal: right half exposed
    return null                        // internal: above the contour
```

```
is_semi_internal(g) → bool:
    return (g.left = null) ≠ (g.right = null)
```

A node is **exposed** (on the contour) iff `uncovered_range(g) ≠ null`.
Terminal nodes are **fully exposed**; semi-internal nodes are
**partially exposed**.

A node **has dependents** iff it has at least one G-child
(`has_dependents(g)` returns true).

These are orthogonal properties. Semi-internal nodes are exposed AND
have dependents. Eviction requires **no dependents**. Restoration
requires **has dependents AND exposed** (i.e., semi-internal).

The one-child state arises when one branch is evicted and its sibling
survives (§12.6). The parent absorbs the evicted child's accumulated value,
becomes semi-internal, and receives observations in the vacated half while
the surviving child continues to shield the other half.

Only G-nodes with no dependents (zero children) are eviction-eligible.
Nodes with dependents are structurally load-bearing — their G-children
depend on them — and must persist until all descendants have been removed
first. The tree contracts from the tips inward.

> _Pseudocode convention._ Throughout this specification, `new G-Node(...)`,
> `new V-Entry(...)`, and `new V-Structural(...)` constructors list only
> the fields being explicitly initialized. Unlisted reference-typed fields
> (e.g., `entry`, `geo_parent`, `left`, `right`, `val_parent`) default to
> null. Unlisted value-typed fields default to their type's zero or identity
> element. The same convention applies to all constructor call sites.

### 4.2 V-Entry (V-Tree Leaf)

Every V-Tree leaf is a G-node's membership token in the tournament. V-Entries
are permanently distinct from V-Structural nodes (V-I5): an entry never
becomes structural, and a structural node never becomes an entry. Entries
are always leaves of the V-Tree — they have no V-children.

| Field                    | Type                 | Description                                                                                                                                                          |
| ------------------------ | -------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| $v.\text{int}$           | `Importance`         | Reference to $v.\text{gnode}.\text{importance}$ — the backing G-node's ranking accumulator; the V-Tree accesses this value through the opaque `Importance` interface |
| $v.\text{val\_parent}$   | V-Structural or null | Parent in V-Tree                                                                                                                                                     |
| $v.\text{gnode}$         | G-Node               | Backing G-node                                                                                                                                                       |
| $v.\text{is\_exposed}$   | bool                 | True iff backing G-node has uncovered range (terminal or semi-internal)                                                                                              |
| $v.\text{is\_evictable}$ | bool                 | True iff backing G-node has zero children (caches $\neg\,\text{has\_dependents}(v.\text{gnode})$)                                                                    |

The exposed flag tracks whether the backing G-node is on the contour —
whether it receives observations. It is true for terminals AND
semi-internals. It describes what the node **is** (exposed to observations),
not what can be done to it. It changes when the G-node splits (flips to
false when both children exist) and when children are removed (flips to true
when uncovered range reappears).

The evictable flag caches the dependents check for the backing G-node.
A terminal G-node (zero children) is evictable; a semi-internal or fully
internal node is not. This cache avoids repeated pointer chasing into the
G-Tree during eviction scans and flag propagation. It is maintained by
split (flips to false when a child is added) and eviction/restoration
(flips to true when all children are removed). The G-root may have
`is_evictable = true` (when it is terminal with zero children); the root
exemption (§12.5) is a separate eviction guard that prevents the root from
being destroyed regardless of this flag.

> _Naming note._ `is_evictable` is a structural predicate — it caches
> whether the backing G-node has zero children (no dependents). It does
> not account for the root exemption (§12.5) or the depth gate (D-I2),
> which are separate eviction guards. The G-root’s entry may have
> `is_evictable = true` while being permanently exempt from eviction.

> **Reference semantics (normative).** `v.int` is a **reference** to
> `v.gnode.importance`, not an independent copy. When
> `accumulate_importance` updates the G-node’s importance accumulator
> during observation (§8.3.3), the V-entry sees the new value immediately
> because it holds a reference to the same storage. Implementations must
> ensure reference identity — e.g., a pointer, a shared cell, or
> structural arrangement where the V-entry reads directly from the
> G-node’s field. Copying the value at construction time and maintaining
> it independently would silently violate G-I4 whenever the importance
> accumulator is updated.

### 4.3 V-Structural (V-Tree Internal)

Pure scaffolding. No external identity. Freely created and destroyed by
rebalancing.

| Field                     | Type                 | Description                                                                                                |
| ------------------------- | -------------------- | ---------------------------------------------------------------------------------------------------------- |
| $v.\text{int}$            | `Importance`         | Aggregate of children's importance values (computed via `Add`)                                             |
| $v.\text{val\_parent}$    | V-Structural or null | Parent in V-Tree                                                                                           |
| $v.\text{children}$       | List of V-nodes      | Length 2 or 3                                                                                              |
| $v.\text{has\_evictable}$ | bool                 | True iff any descendant entry has `is_evictable = true` (i.e., backs a terminal G-node with zero children) |

The structural flag is the logical OR over children's flags — standard
bottom-up propagation. For entry children, the propagation reads
`c.is_evictable`; for structural children, it reads `c.has_evictable`.
The flag enables efficient eviction scanning: subtrees where
$\text{has\_evictable}$ is false contain nothing evictable and can
be skipped entirely.

Note the asymmetry: `is_exposed` tracks whether the entry receives
observations (contour membership — governs refinement and restoration
eligibility). `is_evictable` on entries caches whether the backing G-node
has zero children (governs eviction eligibility). `has_evictable` on
structural nodes tracks whether anything in the subtree is unprotected and
can be safely deallocated (governs eviction scanning). These serve
different purposes.

**Why no flag for legacy promotion?** The `has_evictable` flag exists
because the eviction scan must search the entire V-Tree for evictable
leaves — the flag lets it prune cold subtrees in $O(1)$. The `is_exposed`
flag tracks contour membership — whether the backing G-node receives
observations. The `is_evictable` flag on entries caches the backing
G-node's terminal status, avoiding repeated pointer chases into the G-Tree.
For `has_evictable` propagation, the V-Structural node reads
`c.is_evictable` on each entry child (the cached terminal check), because
semi-internal entries are exposed but not evictable (they have a surviving
child). Structural children contribute their own `has_evictable`.
Legacy promotion needs neither kind of flag. It has no scan — it
piggybacks on the V-Tree's violation resolution, which naturally visits
exactly the entry that just promoted. At that point, a single pointer chase
into `c.gnode` determines whether the node is semi-internal (one child
present, one missing). No subtree aggregate, no propagation path, so no
flag.

The V-Tree is **blind to G-Tree topology and to the ledger type** for
ranking purposes. It does not distinguish terminal G-nodes from internal
G-nodes in the tournament — both appear as V-Tree leaf entries carrying
importance values through the opaque interface. The exposed and evictable
flags reference G-node _state_ (number of children), not G-node _geometry_
(coordinates, intervals). They are used solely for contour membership and
eviction eligibility, not for competitive ranking.

### 4.4 Destruction and Liveness

`destroy(node)` deallocates a node. After destruction, any handle to the
node is invalid. Destruction applies to V-Structural nodes (removed
during rebalancing collapses and promotions), V-Entries (removed during
eviction), and G-nodes (removed during eviction). Each eviction destroys
two objects: the V-entry and its backing G-node. The allocator may reuse
the slot.

`slot_occupied(node)` returns true iff the arena slot backing `node` is
currently occupied by a live object. Used as a liveness guard where
stale handles may persist in work queues — the rebalance loop (§11.8)
and the eviction scan (§12.6) both check `slot_occupied` before
accessing a queued node. With bitset-tracked arena allocation, this
is an $O(1)$ operation.

---

## Chapter 5. The Geometric Tree

### 5.1 Structure

The G-Tree is a tree of variable depth and variable fanout (0, 1, or 2
children per node) over $[0, 2^N)$.

- The root covers $[0, 2^N)$ and always exists.
- Each child $[l, m)$ or $[m, r)$ is a dyadic half of its parent $[l, r)$
  where $m = \text{midpoint}(l, r)$.
- Terminal nodes (0 children) exist at variable depths, from 0 (root is
  terminal) to $N$ (unit cell $[x, x{+}1)$).
- The tree grows by **contour refinement** (§10), subdividing fully
  exposed contour cells into finer cells, and by **contour
  restoration** (§11.6), where a semi-internal entry's competitive
  promotion creates the missing child. It shrinks by **contour
  simplification** (§12), removing an unprotected contour cell. The
  contour is the observation-receiving surface — the bottom edge of
  the tree — and every mutation either refines it (adds resolution),
  restores it (regrows a gap through competition), or simplifies it
  (removes resolution).

**The G-Tree only adds or removes resolution. It never moves boundaries.**

**Midpoint convention.** All pseudocode in this chapter uses
$\text{midpoint}(l, r)$ as shorthand for the stabilized midpoint formula
of §3.2.3:

$$\text{midpoint}(l,\, r) \;\equiv\; l \;+\; (r - l)\,/\,2$$

The naive formula $(l + r) / 2$ risks overflow for both integer and
floating-point types when $l + r$ exceeds the representable range (§3.2.3).
The G-Tree does not store midpoints — they are recomputed from $(l, r)$ on
every routing step, split guard, and range query. The stabilized form is
exact for all dyadic intervals within the IEEE 754 normal range (§3.2.3).

---

### 5.2 Routing

**Observation routing** finds the G-node that should receive an observation
at coordinate $x$:

```
function route_to_receiver(g, x) → G-node:
    m ← midpoint(g.l, g.r)
    if x < m:
        if g.left ≠ null: return route_to_receiver(g.left, x)
        else: return g
    else:
        if g.right ≠ null: return route_to_receiver(g.right, x)
        else: return g
```

This handles all three G-node states uniformly:

| State         | Behaviour                                                            |
| ------------- | -------------------------------------------------------------------- |
| Terminal      | No children — both branches fall through to `return g`               |
| Semi-internal | Routes to the surviving child, or falls through for the vacated half |
| Internal      | Always routes to one of the two children                             |

The routing function is the mechanism that freezes and unfreezes V-entries
(§5.3). When a G-node is internal, all observations to its range are
intercepted by children. The node's V-entry never receives $\Delta$. When
one child is evicted, the parent becomes the receiver for the vacated half —
its V-entry unfreezes for that range. Cost: $O(d_{\text{geo}})$, at most
$O(N)$.

**Geometric depth.** The depth of a G-node in the dyadic hierarchy is
determined by its interval width:

$$\text{depth\_geo}(g) \;=\; N - \log_2(g.r - g.l)$$

where $N$ is the configured maximum depth (§3.1 Property 2). The root
has depth 0 ($g.r - g.l = 2^N$); a unit-width cell has depth $N$.
This function is referenced by `is_final` in §10.1 for the finality
predicate (§3.2.1).

---

### 5.3 Accounting

For any G-node $g$, $g.\text{own}$ records the total of all observations
received directly at $g$. $g.\text{sum} = g.\text{own} + \sum_{\text{children}} c.\text{sum}$ (G-I1). $g.\text{importance}$ is updated via `accumulate_importance` whenever $g$ receives an observation (§8.3.3; configurations in §8.6).

When $g$ is internal (not receiving observations), both $g.\text{own}$ and $g.\text{importance}$ are frozen — a direct consequence of observation routing (§8.4.2). When $g$ splits, its children start at zero; when a child is evicted, $g$ absorbs the child's sum into $g.\text{own}$ (§8.7).

**The inversion.** For internal G-nodes, the two measures diverge over time.
The node's own value is frozen at its pre-split level while its sum grows
as descendants accumulate. When P1 holds (non-negative importance), the
ratio $g.\text{own}\,/\,g.\text{sum}$ trends monotonically toward zero:

$$\frac{g.\text{own}}{g.\text{sum}} \;\to\; 0 \qquad \text{as descendants accumulate (non-negative}\;\Delta\text{)}$$

The G-Tree (ranking by sum) considers the node increasingly important — it
contains ever more activity. The V-Tree (ranking by importance) considers it
increasingly negligible — a shrinking historical footnote. The root is the
extreme case: highest possible sum (it contains everything), lowest eventual
importance (frozen at the earliest, most ancient value). Under signed
observations, $g.\text{sum}$ may oscillate rather than grow monotonically;
the inversion tendency is the same but the convergence is not guaranteed.

For terminal nodes, the ledger measures are identical: $g.\text{own} =
g.\text{sum}$. Under the standard configuration, importance also equals
own. No inversion occurs. Leaves are where the two trees agree.

---

### 5.4 Invariants

$$\textbf{G-I1 (Summation):}\quad g.\text{sum} = g.\text{own} + \textstyle\sum_{c\,\in\,\text{children}(g)} c.\text{sum}$$

At terminal nodes, $g.\text{sum} = g.\text{own}$ (no children).

$$\textbf{G-I2 (Variable Fanout):}\quad \text{Every G-node has zero, one, or two children.}$$

Zero children: fully exposed. One child: semi-internal (created by
asymmetric eviction). Two children: internal (created by catalytic split).
G-I2 is enforced by construction — splits add exactly two children and
evictions remove exactly one — but is stated as an invariant for reference
by other sections.

$$\textbf{G-I3 (Dyadic):}\quad \text{Every G-node covers a dyadic interval } [l, r) \text{ where } r - l = 2^k \text{ for some } k \geq 0.$$

$$\textbf{G-I4 (Importance Reference):}\quad g.\text{entry} \neq \text{null} \implies g.\text{entry.int}\ \text{references}\ g.\text{importance}$$

(Restated from §2.8.2 for completeness.)

G-I4 connects the two trees. A V-entry's importance is not a separate
bookkeeping register — it is a reference to the backing G-node's
importance accumulator. The V-Tree accesses it through the opaque
`Importance` interface without knowing the concrete type. Updates happen
at the G-node via `accumulate_importance`; the V-entry sees them
because it holds a reference. When the node is internal (both children
present), the importance accumulator is frozen (no observations reach it).
When the node is terminal or semi-internal (exposed to observations),
the accumulator grows.

_Invariant checking._ G-I4 is structural — the reference identity is
checked, not numeric equality. An implementation may verify the reference
at debug time. Under the standard configuration, `g.importance` also
equals `g.own` (since aggregation is addition and the starting values
coincide). Under absolute projection, `g.importance` diverges from `g.own`
(absolute vs. signed accumulation), but the reference is still valid.

---

### 5.5 Queries

#### 5.5.1 Point Query

**Point query** for coordinate $x$: route (§5.2) to the deepest-expanded
G-node whose range contains $x$ (the receiver). Returns the receiver's
**uncovered range** ($\text{uncovered\_range}(g)$ as defined in §4.1) and
own-accumulation. For terminal nodes, the uncovered range is the full
interval $[g.l, g.r)$. For semi-internal nodes, it is the uncovered half —
the sub-interval not shielded by the surviving child. Cost:
$O(d_{\text{geo}})$, at most $O(N)$.

A complementary plateau-level query (§5.6.7) returns the structural region
containing $x$ with thatched energy, at cost $O(\log P)$ where $P$ is
the plateau count. The two queries serve different purposes: the routing
point query gives the **observation-level truth** (which cell receives
$x$), while the plateau query gives the **structural truth** (what
contiguous region $x$ belongs to and what energy it carries).

#### 5.5.2 Range Sum

**Range sum** over $[a, b)$: segment-tree decomposition. Fully-contained
nodes contribute $g.\text{sum}$ exactly. At partial overlaps, the node's
own accumulation is pro-rated uniformly within the cell.

```
function range_sum(g, a, b) → T:
    if g = null: return 0
    if b ≤ g.l or g.r ≤ a: return 0
    if a ≤ g.l and g.r ≤ b: return g.sum
    child_sum ← range_sum(g.left, a, b) + range_sum(g.right, a, b)
    overlap ← min(g.r, b) − max(g.l, a)
    own_share ← prorate(g.own, overlap, g.r − g.l)
    return own_share + child_sum
```

Cost: $O(N)$. Null children contribute zero. Fully-contained queries aligned
with node boundaries are exact. Partial overlaps of own accumulation are
pro-rated.

**Capability requirement.** The `prorate` step — computing
$g.\text{own} \times \text{overlap}\, /\, (g.r - g.l)$ — requires the
Range pro-ration capability (§8.8). Without it, only **aligned** range
queries are supported: queries where $[a, b)$ is a union of node
boundaries, so every visited node is either fully contained or fully
excluded. The core ledger type ($\text{zero}, +$) does not guarantee
pro-ration. For integer $T$, the behaviour of integer division in the
pro-ration (truncation direction, rounding) is implementation-defined;
the sum of pro-rated shares across a full partition may not exactly
equal $g.\text{own}$ under truncating division.

> _Design note (semi-internal pro-ration)._ For a semi-internal node,
> $g.\text{own}$ conflates two populations with different spatial
> distributions: (a) pre-split observations, which were spatially
> undifferentiated across $[g.l, g.r)$, and (b) post-eviction
> observations, which routed specifically to the uncovered half.
> The uniform pro-ration treats both populations identically. This is a
> deliberate simplification — tracking the spatial split within
> $g.\text{own}$ would require an additional accumulator per
> semi-internal node. The routing point query (§5.5.1) correctly
> distinguishes the two halves; `range_sum` does not.

---

### 5.6 The Bottom Contour: Plateaus

Imagine the G-Tree drawn as a filled shape: the domain $[0, 2^N)$ on the
$x$-axis, depth on the $y$-axis increasing downward. The tree fills in
wherever it has refined. Deeper regions are where it invested more
resolution; shallower regions are where it did not. The **bottom contour**
is the edge along the underside of that shape — a step function over the
domain.

```
depth 0:  ┌─────────────────────────────────────────────┐
depth 1:  │  ┌────────────────┐                         │
depth 2:  │  │  ┌──────┐      │                         │
          │  │  │[0,2) │[2,4) │        [4,8)            │
          └──┴──┴──────┴──────┴─────────────────────────┘
          0     2      4                                 8

Contour:  ▁▁▁▁▁▁ ▁▁▁▁▁▁▁ ▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁▁
          depth2  depth1          depth0
              3 plateaus
```

At every position $x$, there is a unique deepest-expanded G-node (either a
terminal or a semi-internal node whose child at $x$ is null). The depth of
that node is the contour's value at $x$.

> **Definition.** A **plateau** is a maximal contiguous run of the contour
> at a single depth $d$. Equivalently, it is a maximal set of adjacent
> terminal (or null-child semi-internal) nodes that all sit at depth $d$.

**Plateaus, not terminals, are the semantic unit of the G-Tree's output.**
Within a plateau, all contour cells are at the same depth and interval
width. Whatever intensity variation exists between them is below the tree's
discrimination threshold — if it were significant, the tree would have split
unevenly, introduced a depth change, and broken the plateau in two. The tree
has already rendered its verdict: "this region is uniform at this
resolution." Sub-plateau variation across individual terminals is noise the
tree chose not to model. The boundaries between plateaus are where the tree
found non-uniformity worth resolving — its learned feature edges.

#### 5.6.1 Plateau Basis

Each plateau maintains a **basis**: the minimal set of G-nodes whose `sum`
values capture the plateau's energy.

A G-node $R$ is a basis element of plateau $P$ if:

1. $R$'s contour contribution falls entirely within $P$ — either $R$'s
   subtree produces only contour cells belonging to $P$, or $R$ is a
   semi-internal whose uncovered half belongs to $P$.
2. No ancestor of $R$ also satisfies condition 1 for $P$.
3. $R$ belongs to exactly one plateau's basis.

The plateau bases partition the G-Tree's structurally relevant nodes
across plateaus. The plateau's total energy is:

$$P.\text{sum} = \sum_{R\, \in\, \text{basis}(P)} R.\text{sum}$$

Three cases govern which nodes serve as basis elements:

- **Terminal nodes** are basis elements of their plateau, unless a higher
  balanced ancestor subsumes them.
- **Semi-internal nodes** are always basis elements. Their uncovered half
  belongs to exactly one plateau, and their `sum` — which includes the
  child subtree's energy — becomes part of that plateau's total. These
  are **partial-basis** elements: the source of thatching (§5.6.3).
- **Balanced internal nodes** are basis elements when their entire subtree
  falls within a single plateau. One basis element replaces potentially
  many terminals. When a balanced internal node's children span different
  plateaus, it cannot be a basis element of either.

A fully balanced tree has one plateau whose sole basis element is the
G-Tree root. $P.\text{sum} = \text{root}.\text{sum}$. The $2^d$ terminals
do not appear individually — the balanced root subsumes them all.

A plateau can have **multiple** basis elements when contiguous same-depth
terminals span a region whose lowest common ancestor also covers other
plateaus.

> _Note._ Condition 1 determines plateau membership using the node's
> **tile** — its contour contribution (§5.6.2). The sum formula uses
> $R.\text{sum}$, which covers the node's full **span** (§5.6.2). For
> semi-internal nodes, the span extends beyond the tile into the child's
> territory. This deliberate asymmetry between membership and measurement
> is the source of thatching (§5.6.3).

#### 5.6.2 Tiles, Edges, and Spans

Every basis element has three spatial extents — the tile, the edge, and the
span — which coincide for terminals and balanced internals but diverge for
semi-internals.

> **Definition.** The **tile** of a basis element $R$ is the interval of
> the domain where $R$ contributes to the contour:
>
> | Basis element type                              | $\text{tile}(R)$              |
> | ----------------------------------------------- | ----------------------------- |
> | Terminal $[l, r)$                               | $[l, r)$                      |
> | Balanced internal $[l, r)$                      | $[l, r)$                      |
> | Semi-internal $[l, r)$, child on left $[l, m)$  | $[m, r)$ — the uncovered half |
> | Semi-internal $[l, r)$, child on right $[m, r)$ | $[l, m)$ — the uncovered half |
>
> The **edge** of $R$ is $\text{edge}(R) = \min(\text{tile}(R))$ — the
> leftmost coordinate of the tile. The **span** of $R$ is the full
> G-node interval $[R.l,\, R.r)$, which for semi-internals extends
> beyond the tile into the thatched child's territory.

For terminals and balanced internals, tile = span. For semi-internals,
$\text{tile} \subsetneq \text{span}$. The excess — the covered half where
the surviving child intercepts observations — is the thatched territory.
The plateau sum formula uses $R.\text{sum}$ (computed over the full span),
while plateau membership is determined by the tile. This divergence is by
design (§5.6.3).

#### 5.6.3 Thatching: Multi-Counting by Design

At semi-internal boundaries, the parent plateau and the child plateau
overlap in spatial extent. The parent's partial-basis element is the
semi-internal node, whose `sum` includes the child subtree's energy. The
child's plateau has its own basis elements with their own `sum`. The overlap
region is counted in both plateaus.

**Every layer of thatch is real energy:**

- **Pre-split energy:** observations that arrived at the parent before the
  child existed.
- **Post-split routing:** observations in the uncovered half that route to
  the parent after the child exists.
- **Absorbed energy:** if a sibling child was evicted, its `sum` is folded
  into the parent's `own`.

The multi-counting is correct and by design. The ordered map's basis edge
keys tile the domain — consecutive keys partition $[0, 2^N)$ with no gaps
(P-I1). But each plateau's run reflects its basis elements' true footprint,
which overlaps at every semi-internal boundary like overlapping layers of
thatch. Where there is no thatching (a fully balanced tree, no
partial-basis elements), every plateau's run matches its key range exactly.

**Bilateral thatching.** A plateau $P_M$ can be thatched from both sides
simultaneously — a left-side semi-internal in $\text{basis}(P_L)$ and a
right-side semi-internal in $\text{basis}(P_R)$ each thatch $P_M$
independently:

```
[0,16) internal
  ├─ [0,8)  semi-int (right child only)  ← partial-basis of P_L
  │    └─ [4,8) terminal
  └─ [8,16) semi-int (left child only)   ← partial-basis of P_R
       └─ [8,12) terminal

Contour:  [0,4) d1    [4,12) d2    [12,16) d1
          P_L         P_M          P_R
```

$P_M$ is thatched from the left by $P_L$'s basis element $[0,8)$ and from
the right by $P_R$'s basis element $[8,16)$. Each thatch relationship is
independent and one-hop (P-I4).

**Transitive stacking.** Nested semi-internals create multi-layer thatch at
a single coordinate:

```
[0,8) semi-int (left child only)        ← thatches P_A
  └─ [0,4) semi-int (right child only)  ← thatches P_B
       └─ [2,4) terminal

Contour:  [0,2) d1    [2,4) d2    [4,8) d0
          P_A         P_B         P_C
```

At coordinate 3: covered by $P_C$'s basis $[0,8)$, $P_A$'s basis $[0,4)$,
and $P_B$'s basis $[2,4)$. Thatch depth = 3. Each layer is one-hop per
P-I4; the stacking is the transitive consequence. The depth is bounded by
P-I5.

#### 5.6.4 Plateau Invariants

$$\textbf{P-I1 (Deterministic Tiling).}$$

Let $\sigma\colon [0, 2^N) \to \mathbb{N}$ be the contour depth function
and $0 = a_0 < a_1 < \cdots < a_{P-1}$ its step coordinates
($a_0 = 0$; the contour depth changes at $a_i$ for $i > 0$).
Set $a_P = 2^N$. Then:

$$\text{(i)}\;\; \text{keys}(\text{plateaus}) = \{a_0,\, \ldots,\, a_{P-1}\}$$

$$\text{(ii)}\;\; \bigcup_{R\, \in\, \text{basis}(P_i)} \text{tile}(R) \;=\; [a_i,\; a_{i+1})$$

$$\text{(iii)}\;\; P_i.\text{run} \;=\; \Bigl[\min_R R.l,\;\max_R R.r\Bigr) \;\supseteq\; [a_i,\; a_{i+1})$$

with equality in (iii) iff $\text{basis}(P_i)$ contains no semi-internal
element.

P-I1(i) says the map keys are exactly the contour step coordinates — each
key marks where the contour depth changes (or the domain origin). Because
the step set is derived from $\sigma$, the key set is unique.

P-I1(ii) says the contour tiles of a plateau's basis elements partition
the domain along those steps — no gaps, no excess.

P-I1(iii) says the run — the basis elements' full spatial extent — contains
the contour tile range. The excess is thatched territory (§5.6.3), where
semi-internal basis elements contribute their full $[R.l, R.r)$ rather than
just the uncovered half.

**Implication.** Consecutive plateaus have different depths (since each key
requires a depth transition in $\sigma$). Together with P-I2 (unique minimal
basis), the entire plateau structure — count, keys, basis sets, runs, and
sums — is a deterministic function of the G-Tree state.

$$\textbf{P-I2 (Minimal Deterministic Basis):}\quad \forall\, R \in \text{basis}(P):\; (1)\; R\text{'s contour} \subseteq P, \;(2)\; \nexists\, A \supset R \text{ s.t. } A\text{'s contour} \subseteq P, \;(3)\; R \in \text{basis}(P) \text{ only}$$

The basis is the **minimal** set: no element can be replaced by an ancestor
that also satisfies condition (1). This is the formalization of §5.6.1
conditions 1–3. A fully balanced tree of depth $d$ has a single basis
element — the root — not $2^d$ terminals or $2^{d-1}$ one-level-balanced
internals.

> **Lemma (Non-minimal basis is unsound).** A basis that violates
> condition (2) undercounts the plateau's energy. Suppose ancestor $A$
> satisfies condition (1) for plateau $P$, but the basis contains $A$'s
> descendants $\{D_1, \ldots, D_k\}$ instead of $A$. By G-I1,
> $A.\text{sum} = A.\text{own} + \sum_c c.\text{sum}$, expanded
> recursively down to the $D_i$. Each intermediate internal node on the
> path from $A$ to the $D_i$ contributes a frozen $\text{own}$ term —
> energy accumulated before that node split. The formula
> $P.\text{sum} = \sum_{R} R.\text{sum}$ sums only the basis elements'
> `sum` values; it does not add intermediate nodes' `own` separately.
> Therefore $\sum_i D_i.\text{sum} \neq A.\text{sum}$ whenever the
> intermediate nodes' $\text{own}$ values have non-zero net sum —
> undercounting when the net is positive, overcounting when it is
> negative. (Under non-negative $T$, every non-zero intermediate `own`
> is positive, so any single positive intermediate suffices for strict
> undercounting.) Consolidating to the maximal ancestor is not merely
> canonical — it is necessary for the plateau sum to be correct.

$$\textbf{P-I3 (Tile Disjointness):}\quad \forall\, P_i \neq P_j,\; \forall\, R \in \text{basis}(P_i),\; \forall\, R' \in \text{basis}(P_j):\; \text{tile}(R) \cap \text{tile}(R') = \emptyset$$

The tiles of basis elements from different plateaus are pairwise disjoint.
This is a corollary of P-I1(ii) — distinct plateaus own disjoint tile
ranges $[a_i, a_{i+1})$, and by (ii) each plateau's tiles partition its
range exactly. Nevertheless, P-I3 is stated separately as a checkable
cross-invariant: it catches basis bookkeeping errors that P-I1's
contour-derived check alone would miss.

Note that tiles, not spans, are disjoint. Semi-internal basis elements'
spans extend into the child plateau's territory — that overlap is
thatching, governed by P-I4.

$$\textbf{P-I4 (Thatch — one-hop):}\quad g \in \text{basis}(P),\; g \text{ semi-internal},\; \text{child}(g) \in \text{basis}(P') \implies P' \neq P \;\wedge\; P' \text{ is unique}$$

Every semi-internal basis element $g \in \text{basis}(P)$ thatches the
plateau that $g$'s direct G-child belongs to. The thatch relationship is
one-hop: $g$ thatches the plateau of its _immediate_ child node, not all
transitively contained plateaus. Transitive spatial overlap from nested
semi-internals is a consequence of stacked one-hop thatches, not a single
thatch relationship.

$$\textbf{P-I5 (Thatch Depth):}\quad \text{thatch\_depth}(x) \leq d_{\text{geo}}(x) + 1$$

where $d_{\text{geo}}(x)$ is the G-Tree depth of the contour cell at
coordinate $x$, and $\text{thatch\_depth}(x)$ is the number of plateaus
whose basis elements' spans (not tiles) include $x$.

The owning plateau always contributes 1. Each semi-internal ancestor along
the root-to-contour path can contribute at most one additional thatch layer,
and there are at most $d_{\text{geo}}(x)$ such ancestors.

| Configuration                           | Thatch depth             | Bound           |
| --------------------------------------- | ------------------------ | --------------- |
| Fully balanced tree                     | 1 everywhere             | $d + 1$         |
| Single semi-internal at depth 1         | 2 at covered coords      | $1 + 1 = 2$     |
| Nested semi-internal chain to depth $d$ | $d + 1$ at deepest coord | $d + 1$ (tight) |

#### 5.6.5 Basis Edge and Floor-Key Lookup

The ordered map key for plateau $P_i$ is the **basis edge**:
$\text{key}(P_i) = \min_{R\, \in\, \text{basis}(P_i)} \text{edge}(R) = a_i$,
which by P-I1(i) is the contour step coordinate where depth transitions to
$P_i$'s depth. The tile and edge definitions appear in §5.6.2.

**Floor-key lookup.** A plateau query for coordinate $x$:
$\text{plateaus}.\text{range}(..{=}\text{BasisEdge}(x)).\text{next\_back}()$.
Returns the unique plateau whose tile range contains $x$, in $O(\log P)$.

#### 5.6.6 Structural Complexity

The number of distinct plateaus $P$ is a natural measure of the G-Tree's
structural complexity. A perfectly uniform tree has $P = 1$; a maximally
fragmented tree has $P \propto L$ (number of contour cells). All contour
mutations — refinements and evictions — change $P$ by at most a bounded
constant (see Lemma 12.2).

The invariant $P \leq L \leq |G|$ always holds, where $L$ is
the contour cell count and $|G|$ is the total live node count.
$P \approx L$ means maximal structural variation. $P \ll L$ means the tree
is mostly uniform.

**Proof of $P \leq L \leq |G|$.** Define a _contour cell_ as a maximal
interval where a single G-node is the receiver: each terminal contributes
its full range; each semi-internal contributes its uncovered half. Let $L$
be the contour cell count. Each contour cell belongs to exactly one plateau
(plateaus are maximal same-depth runs of the contour). Each plateau is
non-empty, so $P \leq L$. Each contour cell corresponds to a distinct
G-node (the receiver), and these form a subset of all G-nodes, so
$L \leq |G|$. $\square$

#### 5.6.7 Live Projection

The ordered map is the contour's external interface. The G-Tree
maintains the contour internally; the ordered map projects it.

The contour is maintained as an $\text{OrderedMap}\langle \text{BasisEdge},\,
\text{Plateau} \rangle$ mirror of the G-Tree, stored as a field of the
graph and updated incrementally as the tree mutates — not built on demand.
Each plateau entry carries the thatched spatial extent (run), contour
depth, basis set (§5.6.1), and total energy (`sum`).

| Field   | Type           | Description                                                |
| ------- | -------------- | ---------------------------------------------------------- |
| `depth` | integer        | Contour depth of this plateau                              |
| `basis` | set of G-nodes | Minimal set of basis elements (§5.6.1)                     |
| `sum`   | $T$            | Total energy: $\sum_{R \in \text{basis}} R.\text{sum}$     |
| `run`   | interval       | Thatched spatial extent: $[\min R.l, \max R.r)$ over basis |

The basis edge key (§5.6.5) determines the plateau's
position in the map. The mirror is always consistent with the tree.

- **Point query:** floor-key lookup on BasisEdge, $O(\log P)$. Returns the
  plateau containing the coordinate — a structural summary with thatched
  energy.
- **Range query:** all plateaus stepping across $[a, b)$, via range lookup.
- **Iteration:** plateaus in spatial order, left to right.
- **Length:** $O(1)$ — the plateau count as a structural heartbeat. Rising
  means the tree is learning new structure; falling means it is forgetting.

Incremental maintenance: `observe` recomputes affected basis elements'
contributions along the propagation path, $O(\text{depth})$ — specifically,
Step 4's upward `g.sum` walk (§8.2) updates every basis element on the
propagation path, so plateau `sum` fields, which are derived from those
`g.sum` values, are implicitly maintained without a separate update step.
`refine`
and `evict` insert or remove $O(1)$ ordered map entries, $O(\log P)$ each.
Temporal scaling (`decay()`, §14) updates all $O(P)$ plateaus; dominated by the $O(|G|)$ scaling cost itself.

**The contour governs.** The operations that mutate the G-Tree —
refinement (§10), restoration (§11.6), and eviction (§12) — are
contour mutations. A refinement subdivides a contour cell, inserting
a plateau boundary. A restoration regrows a partially exposed cell,
dissolving a thatched plateau boundary. An eviction removes a
contour cell, dissolving a plateau boundary. The V-Tree decides
_which_ contour cells to refine or evict; competitive promotion
decides which partially exposed cells to restore; the contour is the
surface where those decisions are executed.

**Two complementary point queries** exist at different abstraction levels.
The routing point query (§5.5.1) returns the individual terminal or
semi-internal receiver with its uncovered range — observation-level truth,
$O(d_{\text{geo}})$. The ordered map's floor-key returns a plateau with
thatched energy — structural truth, $O(\log P)$. The first answers "where
does this observation route?"; the second answers "what structural region
contains this coordinate?"

**Split maintenance.** When a terminal G-node splits:

1. Remove the splitting node from its plateau's basis.
2. Each new child becomes a terminal basis element. If an adjacent plateau
   exists at the same depth, the child joins it; otherwise a new plateau is
   created.
3. The splitting node, now internal, may become a basis element of an
   existing or new plateau at depth $d+1$ if its subtree is fully balanced.
4. Recompute affected plateau sums and basis edges.

Cost: $O(d_{\text{geo}} + \log P)$ — ancestor walk to check balanced-subtree
membership for basis recalculation ($O(d_{\text{geo}})$, reduced to $O(1)$
typical by path compression §5.7), plus ordered map insertion ($O(\log P)$).

**Eviction maintenance.** When an unprotected contour cell is
evicted (§12.6):

1. Remove the evicted node from its plateau's basis.
2. The parent's state changes (internal → semi-internal, or semi-internal
   → fully exposed). Update its basis membership accordingly.
3. If the parent becomes fully exposed and an adjacent plateau exists at the
   same depth, merge them.
4. Walk G-Tree ancestors to recalculate affected plateau sums and basis
   edges.

Cost: $O(d_{\text{geo}} + \log P)$ — each ancestor may change basis
membership if a previously balanced subtree became unbalanced. Path
compression (§5.7) reduces the ancestor walk to $O(1)$ typical by avoiding
materialized intermediate nodes.

**Restoration maintenance.** When a semi-internal G-node is
legacy promoted:

1. The restored node was a partial-basis element of some plateau.
   Remove it from that plateau's basis. If no basis elements remain,
   remove the plateau.
2. The new child becomes a terminal basis element — joins an adjacent
   same-depth plateau or creates a new one.
3. The restored node is now internal. If its subtree is balanced
   (both children's contour at the same depth), it becomes a balanced
   internal basis element. Otherwise it is not a basis element — its
   `sum` serves as a frozen benchmark outside the plateau projection.
4. The thatch relationship that the parent plateau held over the
   surviving child's plateau dissolves — the one-hop link (P-I4) no
   longer exists. The surviving child's plateau energy is unchanged;
   only the overlapping reinforcement layer is removed. The
   reinforcement completed its purpose: both halves are now
   self-defending.
5. Recompute affected plateau sums and basis edges.

Cost: $O(d_{\text{geo}} + \log P)$.

---

### 5.7 Path Compression (Implementation Note)

Long chains of nodes where only one branch carries significant intensity can
be compressed into single edges with explicit range annotations, reducing
materialized node count from $O(N \cdot L)$ to $O(L)$. The invariants and
algorithms are unaffected; only the in-memory representation changes.

---

## Chapter 6. The Value Tree

### 6.1 Structure

The V-Tree is a dynamic tree where:

- **Leaves** are V-Entries (G-node membership tokens).
- **Internal nodes** are V-Structural (pure scaffolding with 2 or 3
  children).
- The root may be either an entry (when only one entry exists) or
  structural. The root transitions from entry to structural on the
  first split (§10.3) and never reverts.

The V-Tree knows nothing about spatial coordinates or the ledger type $T$.
It ranks entries purely by importance, accessed through the opaque
`Importance` interface. The exposed and evictable flags reference G-node
_state_ (number of children), not G-node _geometry_ (coordinates,
intervals). They are used solely for contour membership and eviction
eligibility, not for competitive ranking.

### 6.2 Invariants

$$\textbf{V-I0 (Non-emptiness):}\quad \text{After initialization, the V-Tree contains at least one entry.}$$

The G-root's entry is permanently exempt from eviction (§12.5). This
guarantees that the sampling distribution (§6.5) is always defined when
total importance is positive, and that the V-Tree is never structurally
empty during operation.

$$\textbf{V-I1 (Summation):}\quad v.\text{int} = \sum_{c\, \in\, v.\text{children}} c.\text{int} \qquad \text{for every structural V-node}$$

V-I1 must hold whenever $v.\text{int}$ is read for sampling (§6.5),
violation detection (§11.2), or structural aggregation (§11.3). It is a
correctness condition on the values that callers observe, not a constraint
that must hold at every intermediate machine state. How and when
propagation maintains this between operations is an implementation concern;
the `observe()` flow (§8) eagerly propagates in Step 3 before any
rebalancing or sampling occurs, and each rebalancing primitive (§§11.3–11.6)
locally restores V-I1 at the nodes it touches.

$$\textbf{V-I2 (Branching):}\quad |\,v.\text{children}\,| \in \{2,\, 3\} \qquad \text{for every structural V-node}$$

$$\textbf{V-I3 (Max-Uncle):}\quad c.\text{int}\ \leq\ \max\!\bigl\{u.\text{int} : u \in \text{siblings}(p,\, g)\bigr\} \qquad \text{for every V-node } c \text{ with parent } p \text{ and grandparent } g$$

A node is in violation only when it is strictly heavier than **every**
uncle. In a 2-node grandparent (one uncle), the node must beat that single
uncle. In a 3-node grandparent (two uncles), a violation requires beating
both.

(Nodes at V-depth 0 or 1 have no grandparent and are unconstrained by
V-I3. The `is_violated` function in §11.2 handles this by returning false
when the grandparent is null. This is an important architectural property:
the shallowest entries are free to hold any importance without constraint,
which is why the heaviest entries naturally sit at depth 1.)

$$\textbf{V-I4 (Unique Backing):}\quad \text{Every V-Entry corresponds to exactly one G-node.}$$
$$\text{Every G-node has at most one V-Entry.}$$

$$\textbf{V-I5 (Entry-Leaf):}\quad \text{Every V-Entry is a leaf of the V-Tree.}$$
$$\text{Every V-Tree internal node is structural.}$$

V-I5 implies entries never become structural; structural nodes never become
entries. The two types are permanently distinct. Rebalancing creates and
destroys only structural nodes. Entries are rearranged but never transmuted.

$$\textbf{V-I6 (Exposed Flag):}\quad v.\text{is\_exposed} = \bigl(\text{uncovered\_range}(v.\text{gnode}) \neq \text{null}\bigr)$$
$$\text{for every V-Entry } v.$$

$$\textbf{V-I6b (Evictable Flag):}\quad v.\text{is\_evictable} = \neg\,\text{has\_dependents}(v.\text{gnode})$$
$$\text{for every V-Entry } v.$$

$$\textbf{V-I7 (Subtree Evictable Flag):}\quad v.\text{has\_evictable} = \bigvee_{e\, \in\, \text{entry-descendants}(v)} e.\text{is\_evictable}$$
$$\text{for every V-Structural node } v.$$

The disjunction in V-I7 ranges over all descendant V-entries (not
descendant structural nodes). This matches the propagation in §9.3, which
reads `c.is_evictable` for entry children and `c.has_evictable` for
structural children — the structural flag is the transitive summary of its
entry-leaf descendants' evictability.

V-I6, V-I6b, and V-I7 are maintained by flag propagation piggy-backed on
split, restoration, and eviction operations.

### 6.3 Uncle Constraint Semantics

The max-uncle formulation encodes competitive dominance. When a G-node
splits, its V-entry freezes (the G-Tree's upward shield stops observations
from reaching it). The frozen entry becomes uncle (§6.4) to its children's
entries. The constraint says: **a child cannot outrank the frozen benchmark
without triggering promotion.** The child starts at importance $\nu$,
accumulates through observations, and must earn its way up against a fixed
bar. The same competitive dynamic applies to entries created by legacy
promotion (§11.6): the heir starts at $\nu$ and must earn its way up
against its uncles.

When P1 holds (non-negative importance), this is a monotone climb — the
child can only grow. The uncle constraint can only be _newly_ violated by
the child growing past the frozen uncle, never by the uncle shrinking (it
is frozen). The moment the child exceeds every uncle, V-I3 fires and
rebalancing promotes the child — which is exactly the moment the child
region has proven more significant than the parent region was at split
time. When P1 fails (signed configuration), the child may also
_decrease_, making the competition non-monotone — a region receiving
net-negative observations becomes less important rather than earning
promotion. Both sides of the comparison can change, producing more complex
violation patterns.

Three siblings of comparable importance under a 3-node parent coexist
without violations indefinitely — a violation requires beating _both_
uncles. Under a 2-node parent, the bar is lower: a single uncle must be
exceeded. The 3-node stability is a direct consequence of the "max over
uncles" formulation, which makes the V-Tree permissive of local balance.
The V-Tree restructures only when a node dramatically outgrows its entire
neighbourhood, not on every minor importance fluctuation.

### 6.4 Definitions

**Uncle.** Given node $c$ with parent $p$ and grandparent $g$, any sibling
of $p$ in $g$ is an uncle of $c$.

**Violation.** Node $c$ is **in violation** if
$c.\text{int} > \max\{u.\text{int} : u \in \text{siblings}(p,\, g)\}$ — it
outranks every uncle. The comparison uses the `Ord` method of the
`Importance` interface.

**Sibling.** $\text{siblings}(c, p) = \{x \in p.\text{children} : x \neq c\}$.

**Sole sibling.** For a 2-node parent $p$:

```
sibling(c, p) → node:
    Precondition: p is a 2-node.
    Return the unique element of p.children \ {c}.
```

**3-node / 2-node.** A structural node $p$ is a 3-node if
$|p.\text{children}| = 3$, a 2-node if $|p.\text{children}| = 2$.

**Heaviest child.**

```
heaviest_child(p) → child:
    Return argmax { c.int : c ∈ p.children }.
    Ties broken by a deterministic implementation-defined rule.
```

Any deterministic tie-breaking rule preserves correctness. The choice
affects which contraction and escalation paths are taken when siblings
have equal importance, potentially producing different (but equally valid)
V-Tree shapes. Implementations requiring reproducible behaviour across
platforms should specify a canonical rule (e.g., first child in list order,
or by backing G-node coordinate for entries).

**V-Tree depth.**

```
depth_V(v) → integer:
    The number of edges on the path from the V-root to v.
    The V-root has depth 0.
```

Computing $\text{depth}\_V(v)$ requires traversing the parent chain to the
root, costing $O(h_V)$. Implementations may cache depth as a maintained
field on each V-node to reduce this to $O(1)$; see §12.6 for a discussion
of depth caching strategies, including a lazy relative-depth scheme that
reduces maintenance to $O(1)$ per structural operation.

### 6.5 Proportional Sampling

The V-Tree enables sampling an entry with probability proportional to
importance. **This operation requires P1 (bounded below)** — without a
known bottom element, the probability ratios
$c.\text{int}\,/\,v.\text{int}$ are undefined (values may be negative,
making ratios meaningless). When P1 fails, sampling is unavailable.

**Preconditions.** V-I0 (non-emptiness): the V-Tree contains at least one
entry. V-I1 (summation): structural nodes' aggregates are current, so that
$\sum c.\text{int} = v.\text{int}$ at every structural node visited during
the walk.

```
function sample(v) → V-Entry | ⊥:
    if v.int = ν and P2 holds: return ⊥   // ground importance → no distribution
    if v is entry: return v
    choose child c with probability c.int / v.int
    return sample(c)
```

When P2 holds ($\nu \preceq a$ for all $a \in I$): total importance equals $\nu$ means "never
observed" = "at minimum." The guard returns $\perp$ — the honest answer
that ground importance means no distribution to sample from. This can
occur on a freshly constructed tree before any observations.

When P1 holds but P2 fails: new entries have non-minimum importance
($\nu$ is not the minimum of $I$). Sampling is technically well-defined but includes
phantom importance from unobserved entries. The caller should interpret
results accordingly.

When P1 fails: sampling is undefined (precondition not met).

Every entry reached during sampling is guaranteed to be live. No defensive
checks are needed: entries are removed from the V-Tree before their backing
G-nodes are destroyed, and only unprotected G-nodes — which have no
dependents — are ever evicted. The V-Tree is always structurally clean.

The expected sampling cost is:

$$E[\text{cost}] = \sum_{i} w_i \cdot \text{depth}_V(v_i)$$

where $w_i = v_i.\text{int}\,/\,I_{\text{total}}$ is the weight fraction
of entry $v_i$. Since V-I3 pushes high-importance entries to shallow depth,
the expected cost scales with the **entropy of the importance distribution**,
not with $\log L$. The depth bound (§18.1) yields:

$$E[\text{cost}] \;\leq\; \frac{H}{\log_2 \phi} + 1 \;\approx\; 1.44\,H + 1$$

where $H = \sum_i w_i \log_2(1/w_i)$ is the Shannon entropy (proof in
§18.2). For concentrated distributions — a few dominant hotspots — sampling
approaches $O(1)$.

A V-Tree sample returns a G-node at whatever scale the system has found
significant. Sampling does not just identify a _location_ — it identifies
a location _and a scale_. The returned entry's backing G-node carries an
interval $[l, r)$ whose width reflects the resolution at which the system
confirmed significance. A sampled internal G-node represents "this region
is significant as a whole"; a sampled terminal G-node represents "this
fine-grained cell is significant." This multi-scale property distinguishes
V-Tree sampling from flat proportional sampling over a fixed set of bins.
Sampling preferentially directs attention toward regions that have proven
sustained significance through repeated competitive promotion.

> **No thatching in sampling.** Plateau thatching (§5.6.3)
> multi-counts energy across overlapping plateaus via `g.sum`.
> Sampling uses `g.own` — the V-entry's importance — which the
> V-Tree propagates with clean summation (V-I1). The two mechanisms
> operate on different accumulators: thatching answers "what is the
> total energy in this plateau region?" while sampling answers
> "which contour cell should receive attention?" The returned cell
> is trimmed to the uncovered half for semi-internals (§5.5.1),
> with `intensity = g.own` — no plateau overlap enters the result.

---

## Chapter 7. Depth Gates

Two V-Tree depth thresholds govern the G-Tree's contour mutations. The
**creation gate** $D_{\text{create}}$ controls where new spatial resolution
may be added: only entries at shallow V-Tree depth — those that have proven
global competitive significance — may split. The **eviction gate**
$D_{\text{evict}}$ controls where resolution is withdrawn: unprotected
contour cells past the eviction threshold — the globally least
significant — are removed. A mandatory buffer zone separates the two gates,
preventing immediate oscillation between creation and eviction.

A pure importance threshold is a local condition — a leaf may exceed
$\theta$ yet be globally insignificant relative to the rest of the tree.
V-Tree depth is a global condition: a shallow position means the entry has
outcompeted its neighbourhood. The combination of a basic importance
eligibility ($g.\text{importance} > \theta$) with the depth gate
($\text{depth}_V \leq D_{\text{create}}$) ensures that only locally
significant AND globally important nodes earn spatial refinement.

### 7.1 Parameters

| Parameter           | Type          | Constraint              | Role                                                                 |
| ------------------- | ------------- | ----------------------- | -------------------------------------------------------------------- |
| $\theta$            | $I$ (carrier) | $> \nu$ (via $\preceq$) | Minimum importance for split eligibility (local gate; see §10.1)     |
| $D_{\text{create}}$ | integer       | $\geq 0$                | Maximum V-Tree depth at which a split is permitted (global gate)     |
| $D_{\text{evict}}$  | integer       | $\geq 1$                | Minimum V-Tree depth beyond which unprotected entries are evicted    |
| $\text{buffer}$     | integer       | $\geq 1$                | $D_{\text{evict}} - D_{\text{create}}$ — mandatory gap between gates |

The split threshold $\theta$ must exceed $\nu$ — the ground element
assigned to freshly created entries — otherwise every new entry immediately
qualifies for splitting. ($\theta > \nu$ ensures freshly created entries
at importance $\nu$ never immediately qualify for splitting; under the
standard configuration where $\nu = 0$, this reduces to $\theta > 0$.)
$D_{\text{evict}} \geq 1$ ensures at
least the V-root's direct children survive eviction. $D_{\text{create}} \geq 0$
is implied by $D_{\text{evict}} \geq 1$ and $\text{buffer} \geq 1$, but is
stated as an explicit constraint since dynamic adjustment (§7.4) must clamp
to maintain it. All four parameters may be adjusted at runtime; the
constraints are maintained at every adjustment.

### 7.2 Invariants

$$\textbf{D-I1 (Creation Gate):}\quad \text{split}(g) \implies \text{depth}_V(g.\text{entry}) \leq D_{\text{create}}$$

$$\textbf{D-I2 (Eviction Gate):}\quad \text{evict}(v) \iff \text{depth}_V(v) > D_{\text{evict}} \;\wedge\; \neg\,\text{has\_dependents}(v.\text{gnode}) \;\wedge\; v.\text{gnode} \neq G_{\text{root}}$$

$$\textbf{D-I3 (Buffer Zone):}\quad D_{\text{create}} < D_{\text{evict}}$$

$$\textbf{D-I4 (Parameter Bounds):}\quad \theta > \nu,\quad D_{\text{evict}} \geq 1,\quad D_{\text{create}} \geq 0$$

D-I2 is the complete specification of eviction eligibility. Three
conditions must hold simultaneously: the entry sits past the eviction
threshold (competitive insignificance), the backing G-node has no children
(structurally unprotected), and the node is not the G-root. The root
exemption guarantees V-I0 (the V-Tree always contains at least one entry
after initialisation) and ensures the sampling distribution (§6.5) is
always defined when total importance is positive. The root exemption cannot
fire vacuously: after the bootstrap split, the G-root's entry typically
sits at V-depth 1, and $D_{\text{evict}} \geq 1$ means
$\text{depth}_V = 1 \not> 1$. Under rebalancing, the G-root's entry can be
pushed to V-depth $\geq 2$; if $D_{\text{evict}}$ is simultaneously
tightened to 1, the root entry would satisfy the depth and dependents
conditions yet be correctly exempted by the third clause.

### 7.3 The Buffer Zone

The interval $(D_{\text{create}},\, D_{\text{evict}}]$ is where entries
live on borrowed time. They cannot create children (too deep for D-I1) and
are not yet evicted (not past D-I2's threshold). They can still receive
observations and promote upward through the competitive mechanism. The
buffer gives them time to prove themselves.

The eviction condition requires both depth and the absence of dependents.
An internal G-node's entry may sit past $D_{\text{evict}}$ indefinitely —
it is structurally load-bearing and eviction-immune. Its terminal
descendants, being the coldest and lightest, will be evicted first. As
they are removed, the internal node progressively loses children, and when
both are gone it becomes unprotected and eviction-eligible itself. The
tree contracts from the tips inward.

**Legacy promotion is depth-gated at $D_{\text{evict}}$.** When a
semi-internal entry's violation is resolved and
$\text{depth}_V(c) > D_{\text{evict}}$, the dispatcher (§11.9) uses skip
promote instead of legacy promote — the heir would land at the vacated
depth, past the eviction threshold, and be immediately eligible for
eviction next tide. Suppressing the creation avoids a futile allocate–evict
round-trip to the same end state. The V-Tree's competitive mechanism
remains the primary gate; the depth check is an implementation optimisation
that eliminates transient node churn in the eviction zone.

> _Implementation note (buffer oscillation)._ The buffer zone does not
> guarantee that newly created entries survive the trailing rebalance. A
> cascade of rebalancing operations triggered by the same observation can
> push entries deeper than the buffer width. This produces a **futile
> cycle**: split → rebalance deepens children past $D_{\text{evict}}$ →
> eviction removes children → parent absorbs zero and retries. The
> depth gate on legacy promotion eliminates one class of futile
> cycle — heirs that would be immediately evictable are never created —
> but catalytic-split children can still be pushed past $D_{\text{evict}}$
> by the trailing rebalance. Futile cycles are self-limiting and
> invariant-preserving. In most cases the cascade restructures the
> neighbourhood, breaking the cycle. In the degenerate case where the
> V-Tree structure is identical each iteration (parent splits, children
> evicted with zero accumulation, parent re-splits), the cycle persists
> but incurs only $O(1)$ overhead per observation — bounded irreducible
> cost, not unbounded repetition. All structural and accounting invariants
> hold throughout. No finite buffer eliminates futile cycles universally:
> the cascade depth depends on intensity imbalance, number of competing
> fronts, and V-Tree height. However, futile cycle frequency is monotone
> non-increasing in buffer width, and empirically they constitute a small
> fraction of total evictions (typically $\leq 5\%$, worst observed
> $\leq 15\%$). Wider buffers trade memory for stability;
> $\text{buffer} \geq D_{\text{create}}$ is a practical heuristic for most
> workloads.

### 7.4 Dynamic Depth Control

$D_{\text{evict}}$ and $D_{\text{create}}$ need not be compile-time
constants. A single node counter — incremented on G-node creation,
decremented on destruction — enables runtime adjustment.

**Runtime parameters.**

| Parameter               | Type    | Constraint   | Role                               |
| ----------------------- | ------- | ------------ | ---------------------------------- |
| $\text{budget}$         | integer | $> 0$        | Soft node-count target             |
| $\alpha_{\text{relax}}$ | real    | $\in (0, 1)$ | Hysteresis fraction for relaxation |

```
total_nodes ← 0   // maintained O(1) per split/eviction

function adjust_depth_gates():
    if total_nodes > budget:
        D_evict  ← max(1, D_evict − 1)          // maintain D-I4
        D_create ← max(0, D_evict − buffer)      // maintain D-I3, D-I4
    else if total_nodes < budget × α_relax:
        D_evict  ← D_evict + 1
        D_create ← max(0, D_evict − buffer)      // maintain D-I4
```

The `max(0, ...)` clamp on $D_{\text{create}}$ is necessary: when
$\text{buffer} \geq D_{\text{evict}}$, the subtraction produces a negative
value, violating D-I4. The clamp yields $D_{\text{create}} = 0$, which
means only V-root children may split — the most restrictive possible
creation policy, short of disabling splits entirely. D-I3 is maintained:
$0 < D_{\text{evict}}$ holds since $D_{\text{evict}} \geq 1$.

**Why this works.** Lowering $D_{\text{evict}}$ by 1 exposes one additional
layer of unprotected contour cells to eviction. The next
`check_evictions` pass removes them. Their parents absorb the value
and may themselves become unprotected — cascading contraction exactly as
§12.7 describes, but triggered by memory pressure rather than competitive
depth. The entries removed are the globally least significant unprotected
entries: they sit deepest in the V-Tree precisely because they lost the
competitive tournament.

**Double effect.** Tightening $D_{\text{evict}}$ simultaneously tightens
$D_{\text{create}}$ (via D-I3). The tree stops expanding and starts
contracting in the same adjustment. When pressure eases, both thresholds
relax and the tree can expand where the V-Tree authorises it.

**The feedback loop is stable:**

```
Memory pressure rises
    → lower D_evict
    → unprotected contour tips evicted, parents absorb
    → total_nodes decreases
    → stop lowering D_evict

Memory pressure falls
    → raise D_evict
    → buffer zone widens, more detail tolerated
    → tree expands where warranted
```

This yields a soft memory bound with graceful degradation, governed by a
single integer that the system adjusts automatically. No garbage collection,
no LRU eviction, no complex memory pooling. The existing split and eviction
machinery does all the work.

> **Invocation.** `adjust_depth_gates()` is called in Step 6b of
> `observe()` (§8.2) — after rebalancing and before eviction. It may
> also be called by standalone maintenance triggers — external
> maintenance code should call `adjust_depth_gates()` before standalone
> `check_evictions()` invocations (§12.7) — or by `decay()` (between
> Phase 4 and Phase 5 of §14.4). Each invocation is $O(1)$.

### 7.5 Hard Budget Guarantee

A hard ceiling $G_{\max}$ must accommodate the maximum transient G-node count
from a single `observe()` call. This section derives the budget invariant
that guarantees no call can exceed $G_{\max}$.

**Parameters.**

| Parameter  | Type    | Constraint                     | Role                          |
| ---------- | ------- | ------------------------------ | ----------------------------- |
| $G_{\max}$ | integer | $\geq 5$ and $> \text{budget}$ | Hard ceiling on total G-nodes |

#### 7.5.1 The Semi-Internal Consumption Lemma

> **Lemma.** During any trailing rebalance (Phase 3 of `check_evictions`
> or Step 6 of `observe`), the number of legacy promotions $L$ satisfies
> $L \leq S$, where $S$ is the semi-internal G-node count at the start of
> that rebalance.
>
> _Proof._ (1) Each legacy promotion consumes exactly one semi-internal
> G-node — it creates the missing child, converting semi-internal to
> internal. (2) No other rebalancing primitive mutates the G-Tree:
> contraction, standard promote, and skip promote create and destroy only
> V-Structural nodes (§§11.3–11.5). (3) No G-child is added (except by
> legacy promote) or removed (eviction is Phase 2 only) during a trailing
> rebalance, so no G-node can _become_ semi-internal during the
> rebalance. The semi-internal population can only decrease. Each legacy
> promotion draws from a stock that can only shrink. $\square$

#### 7.5.2 Per-Observe Node Accounting

Let $S_0$ = semi-internal count at entry to `observe()`. Tracking G-node
mutations across one call:

- _Step 5 (split):_ catalytic split creates 2 new G-nodes. The parent
  goes terminal → internal, skipping semi-internal. $\Delta S = 0$.
- _Step 6 (rebalance):_ may trigger $L_1$ legacy promotions, each
  consuming one semi-internal. $S_6 = S_0 - L_1$. Creates $L_1$ G-nodes.
- _Step 7, Phase 2 (evictions):_ $E$ evictions destroy $E$ G-nodes. Each
  may convert a parent from internal → semi-internal ($\Delta S = +1$)
  or semi-internal → terminal ($\Delta S = -1$). $S_{\text{post}} \leq S_6 + E$.
- _Step 6b (depth adjustment):_ creates no G-nodes; omitted from the accounting.
- _Step 7, Phase 3 (trailing rebalance):_ $L_2$ legacy promotions,
  $L_2 \leq S_{\text{post}} \leq S_0 - L_1 + E$. Creates $L_2$ G-nodes.

Summing: $\Delta|G| = 2 + L_1 + L_2 - E$. Since
$L_1 + L_2 \leq L_1 + (S_0 - L_1 + E) = S_0 + E$, the $L_1$ cancels
internally and the $E$ cancels against evictions:

$$\boxed{\Delta|G|_{\text{observe}} \leq 2 + S_0}$$

The "+2" is the catalytic split. The "+$S_0$" is the maximum net legacy
promotions after internal cancellation.

**Tightness.** The bound $L \leq S$ is correct but loose. Only
semi-internals that become _violated_ (importance exceeds all uncles)
actually fire legacy promotion. Each eviction absorption creates at most
one initial violation; the cascade from that violation traverses at most
$O(h_V)$ levels, creating $O(1)$ side-effect violations per level. So:

$$L \leq \min\!\bigl(E \cdot h_V,\; S_{\text{post}}\bigr)$$

Under typical operation, $L \approx E$: most legacy promotions correspond
1-to-1 with evictions (the strengthened parent is itself semi-internal and
violates). The excess ($L > E$) requires cascade effects reaching distant,
barely-shielded semi-internals — a structurally fragile configuration that
rebalancing itself tends to dissolve.

#### 7.5.3 The Budget Invariant

The budget invariant guarantees that any single `observe()` call cannot
push $|G|$ past $G_{\max}$:

$$\boxed{|G| + S + 2 \leq G_{\max}}$$

where $S$ is the current semi-internal count. **The soft trigger** fires
when $|G| > G_{\max} - 2 - S$. At that point, `adjust_depth_gates()` (§7.4)
lowers $D_{\text{evict}}$ to expose more eviction candidates. The existing
tightening mechanism is unchanged; only the threshold formula accounts for
the semi-internal headroom.

The budget invariant is frame-invariant: it holds at every call boundary
regardless of whether the caller is `observe()` or an external maintenance
trigger (§12.7).

> **Minimum ceiling.** The hard ceiling must satisfy $G_{\max} \geq 5$. At initialization, $|G| + S + 2 = 1 + 0 + 2 = 3$. The first bootstrap split (§10.3) creates 2 G-nodes without removing any, pushing $|G|$ to 3 with $S = 0$ (terminal → internal skips semi-internal). The invariant then requires $3 + 0 + 2 = 5 \leq G_{\max}$. Without this minimum, a graph constructed with $G_{\max} < 5$ would violate the budget invariant on its first structural operation. The constraint $G_{\max} \geq 5$ is independent of the soft budget target — it is the minimum headroom needed for the tree to perform a single split.

#### 7.5.4 Maintaining $S$

A single integer counter, incremented and decremented at each G-node state
transition:

| Transition               | Trigger                     | $\Delta S$                |
| ------------------------ | --------------------------- | ------------------------- |
| Terminal → internal      | Catalytic/bootstrap split   | $0$ (skips semi-internal) |
| Internal → semi-internal | Eviction of one child       | $+1$                      |
| Semi-internal → terminal | Eviction of surviving child | $-1$                      |
| Semi-internal → internal | Legacy promotion            | $-1$                      |

Cost: $O(1)$ per transition, piggy-backed on the operations that already
perform the G-Tree mutation.

#### 7.5.5 Static Fallback

If tracking $S$ is undesirable, a static bound suffices.

> **Lemma ($S \leq |G| - 1$).** In any G-Tree with $|G| \geq 1$,
> $S \leq |G| - 1$.
>
> _Proof._ Every semi-internal G-node has exactly one child. Consider the
> tree's leaf set — nodes with zero children (terminals). Every tree with
> at least one node has at least one leaf. (Proof: start at any node;
> while it has a child, descend. The tree is finite, so the walk
> terminates at a leaf.) A leaf is not semi-internal (zero children, not
> one). Therefore at least one G-node is not semi-internal, giving
> $S \leq |G| - 1$.
>
> _Tightness._ The bound is achieved by a degenerate linear chain
> (§12.10): $|G| - 1$ semi-internal nodes each with one child, ending at
> a single terminal. $\square$

Substituting $S \leq |G| - 1$ into the budget invariant:

$$|G| + (|G| - 1) + 2 \leq G_{\max} \implies |G| \leq \frac{G_{\max} - 1}{2}$$

This reserves approximately half of the budget as permanent headroom —
correct but conservative. The dynamic mechanism (§7.5.4) is strongly
recommended for any implementation where the headroom cost is material.

#### 7.5.6 Eviction Pass Accounting

For the eviction pass alone (Steps 7 only of `observe()`):
$-E \leq \Delta|G|_{\text{eviction}} = -E + L_2 \leq S_0 - L_1 \leq S_0$.
When $S_0 = 0$ (no semi-internal G-nodes at entry), $L_1 = 0$ and
$L_2 \leq E$ (legacy promotions are funded entirely by semi-internals
created by Phase 2 evictions), so $\Delta|G|_{\text{eviction}} \leq 0$.
In general, the eviction pass is bounded above by $S_0$, not
unconditionally non-positive.

Legacy promotions may fire during the trailing rebalance when an
eviction's absorption (Step 3 of §12.5) strengthens a G-parent's
V-entry at a shallow V-position, triggering a violation that the §11.9
dispatcher resolves via legacy promote. These are genuine competitive
shifts — the heir lands at a viable V-depth ($\leq D_{\text{evict}}$).
When the heir would land past $D_{\text{evict}}$ (and thus be
immediately eviction-eligible), the depth gate dispatches to skip
promote instead, avoiding a futile allocate–evict round-trip.

Accounting for standalone invocations of `check_evictions` (e.g., during
dynamic $D_{\text{evict}}$ adjustment or external maintenance triggers
per §12.7): such calls have their own $S_0$ at entry. The per-observe
budget arithmetic (§7.5.2) applies identically with the catalytic split
term removed ($\Delta|G| \leq S_0$, not $2 + S_0$). The budget invariant
$|G| + S + 2 \leq G_{\max}$ absorbs this bound.

---

## Chapter 8. Observation Flow

This chapter specifies what happens when an observation arrives. §8.1
defines the input contract. §8.2 presents the algorithm. §8.3 develops
the step-by-step rationale. §8.4 establishes accounting properties
that hold across the entire flow. §8.5 introduces the observation type
($O$ vs $T$). §8.6 defines the accumulation configurations that bridge
ledger values to governance importance. §8.7 specifies how eviction
absorption interacts with each configuration. §8.8 decomposes the ledger
type's capabilities. §8.9 defines the propagation helpers referenced
throughout.

---

### 8.1 Input Contract

**Parameters.**

| Parameter | Type              | Constraint                         |
| --------- | ----------------- | ---------------------------------- |
| $x$       | Coordinate ($C$)  | Must not be NaN; clamped to domain |
| $\Delta$  | Observation ($O$) | Must not be zero                   |

**Validation rules.**

- **NaN rejection (§3.2.4).** If $x$ is NaN, the call is rejected
  immediately with a hard failure (panic, assertion, exception). NaN
  in a coordinate is never meaningful; silent handling would mask bugs.

- **Domain clamping (§3.2.5).** Coordinates below zero are clamped to
  zero. Coordinates at or above $2^N$ are clamped to $2^N$. Routing
  treats the clamped value correctly: $x = 2^N$ routes rightward at
  every level, landing in the rightmost terminal.

- **Nonzero delta.** $\Delta = 0$ is rejected by precondition. A zero
  delta would propagate $O(d_{\text{geo}} + h_V)$ zero increments
  harmlessly but wastefully, and could trigger a vacuous split attempt
  if the receiver already exceeds $\theta$. Excluding zero by
  precondition eliminates ambiguity about whether a zero-delta
  observation constitutes a meaningful event.

---

### 8.2 The Algorithm

```
function observe(x, Δ):

    // Step 1: Validate and route to the receiving node
    reject if x is NaN                          // §8.1
    x ← clamp(x, 0, 2^N)                       // §8.1
    assert Δ ≠ 0                                // §8.1
    receiver ← route_to_receiver(G_root, x)     // §5.2

    // Step 2: Accumulate in the ledger
    receiver.own ← receiver.own + Δ             // §8.3.2

    // Step 3: Update importance, propagate V-sums, detect violations
    assert receiver.entry ≠ null                // §8.3.3
    receiver.importance ← π(receiver.importance, Δ)   // §2.6; configurations §8.6
    propagate_v_sums(receiver.entry)            // §8.9
    check_id ← receiver.entry
    while check_id ≠ null:
        if is_violated(check_id): push check_id // §11.11.2
        check_id ← check_id.val_parent

    // Step 4: Propagate G-Tree sums upward
    g ← receiver
    while g ≠ null:
        g.sum ← g.sum + Δ                      // §8.3.4
        g ← g.geo_parent
    // Note: plateau sums (§5.6.7) are derived views of basis elements'
    // g.sum fields; the upward walk above implicitly maintains them.

    // Step 5: Attempt contour refinement
    attempt_refine(receiver)                    // §10.1

    // Step 6: Restore V-I3
    rebalance()                                 // §11.8

    // Step 6b: Adjust depth gates if budget pressure exists
    adjust_depth_gates()                        // §7.4

    // Step 7: Evict unprotected contour cells past depth threshold
    check_evictions()                           // §12.6
```

The pseudocode uses `+` between a $T$-valued accumulator and $\Delta$.
When the observation type equals the ledger type ($O = T$), this is
ordinary addition. When $O \neq T$, each `+` should be read as the
observation type's mapping function $\text{accumulate}_T: (T, O) \to T$.
The distinction is developed in §8.5; the common case is $O = T$.

---

### 8.3 Step-by-Step Rationale

#### 8.3.1 Step 1: Validation and Routing

Step 1 performs all entry-point validation before any mutation occurs.
NaN rejection and domain clamping are specified in §8.1. The nonzero
assertion is a precondition check: it documents a caller obligation,
not a runtime filter.

After validation, `route_to_receiver(G_root, x)` (§5.2) descends the
G-Tree to the deepest-expanded node whose range contains $x$. This
handles all three G-node states uniformly:

| Receiver state | Behaviour                                                             |
| -------------- | --------------------------------------------------------------------- |
| Terminal       | No children — both branches fall through. Receives all observations.  |
| Semi-internal  | Routes to the surviving child, or falls through for the vacated half. |
| Internal       | Always routes to one of the two children — never the receiver.        |

Cost: $O(d_{\text{geo}})$, at most $O(N)$.

#### 8.3.2 Step 2: Ledger Accumulation

Only the receiver's `own` field is updated. All other G-nodes are
updated through sum propagation in Step 4. The receiver is the unique
contour cell for coordinate $x$ — one mutation site, no ambiguity.

When $O \neq T$, the bare `+` should be read as
$\text{accumulate}_T(\text{receiver.own}, \Delta)$ (§8.5).

#### 8.3.3 Step 3: Importance Update and Violation Detection

Step 3 performs three logically distinct operations in sequence.

**(a) Importance accumulation.** The receiver's importance is updated
through the user-supplied `accumulate_importance` function, defined
via the projection $\pi$ (§2.6; concrete configurations in §8.6).
The V-entry references `receiver.importance` (G-I4), so it sees the
new value immediately.

**(b) V-sum propagation.** `propagate_v_sums(receiver.entry)` (§8.9)
walks structural ancestors, recomputing aggregates via the
`Importance` interface's `Add` method. This restores V-I1 at every
structural node on the root-ward path.

**(c) Ancestor violation walk.** The importance increase may cause
the entry itself or any structural ancestor to exceed its uncle.
The walk checks `is_violated` (§11.2) at every node from the entry
to the V-root, pushing violators to the rebalance queue:

```
    check_id ← receiver.entry
    while check_id ≠ null:
        if is_violated(check_id): push check_id
        check_id ← check_id.val_parent
```

This catches both direct violations (the entry outgrew its uncle)
and indirect violations (a structural ancestor's aggregate grew past
_its_ uncle). The walk costs $O(h_V)$, matching `propagate_v_sums`
— the two can share the same ancestor traversal in implementation.

**Why the assertion holds.** The assertion `receiver.entry ≠ null`
documents an invariant, not a conditional. Every live G-node has an
entry: initialization creates the root with an entry (§15), splits
create children with entries (§10.2, §10.3), legacy promotion creates
new nodes with entries (§11.6), and eviction destroys the entry and
G-node simultaneously (§12.5). A destroyed G-node cannot be a
receiver (it has been deallocated and is not reachable by routing).

#### 8.3.4 Step 4: G-Tree Sum Propagation

The walk from receiver to G-root adds $\Delta$ to every ancestor's
`sum`, restoring G-I1:

```
    g ← receiver
    while g ≠ null:
        g.sum ← g.sum + Δ
        g ← g.geo_parent
```

Cost: $O(d_{\text{geo}})$, at most $O(N)$. This touches only the
G-Tree — no V-Tree fields are read or written. When $O \neq T$,
each `+` is the observation mapping (§8.5).

**Independence from Step 3.** Steps 3 and 4 modify different trees'
aggregates. `propagate_v_sums` reads V-entries' importance (updated
in Step 3a) and writes V-structural aggregates. G-sum propagation
reads and writes G-nodes' sums. The two are independent and could
execute in either order. The current ordering (V before G) is a
convention, not a correctness requirement.

#### 8.3.5 Step 5: Contour Refinement

`attempt_refine(receiver)` (§10.1) checks whether the receiver is
eligible for catalytic split: fully exposed (no dependents),
importance exceeds $\theta$, V-depth at most $D_{\text{create}}$,
and the interval is divisible. If eligible, the receiver is split
into two finer contour cells.

Step 5 may push _additional_ violations to the rebalance queue from
two sources: (a) the preprocessing contraction (§10.1: when the
V-parent is a 3-node, it contracts before splitting) and (b)
post-split violation checks that catch configurations where new
entries or their structural grouping node may exceed existing uncles.
Under the standard configuration (P2 + P3 + P4), source (b) finds
nothing — catalytic splits are violation-free (§10.4). Under
non-standard configurations (P1 fails, or $\nu \oplus \nu \neq \nu$), the
post-split checks may also push an $O(h_V)$ ancestor walk. These
violations are distinct from the observation-triggered violations
pushed in Step 3 and all sets must be in the queue before
`rebalance()` runs.

#### 8.3.6 Step 6: Rebalancing

`rebalance()` (§11.8) drains the violation queue populated by Steps 3
and 5. It resolves every V-I3 violation through contraction, standard
promote, skip promote, or legacy promote (§§11.3–11.6). Each
resolution may create side-effect violations that are pushed to the
queue and resolved in subsequent iterations. The loop terminates by
the arguments in §11.13 (Φ-decrease when P2 holds, entry-creation bound when P2 fails).

After Step 6 completes, V-I3 holds everywhere.

#### 8.3.6b Step 6b: Dynamic Depth Adjustment

`adjust_depth_gates()` (§7.4) checks the current node count against
the soft budget target and adjusts $D_{\text{evict}}$ and
$D_{\text{create}}$ accordingly. It runs after rebalancing (which
may have created nodes via legacy promotion) and before eviction
(which uses the adjusted thresholds).

When $|G| > \text{budget}$, the function lowers $D_{\text{evict}}$
by 1, exposing one additional layer of unprotected contour cells to
Phase 2 of `check_evictions`. When $|G| < \text{budget} \times
\alpha_{\text{relax}}$, it raises $D_{\text{evict}}$ by 1, allowing
the tree to retain more detail. In the common case (node count
within the hysteresis band), the function is a no-op.

Cost: $O(1)$ — two comparisons and at most two assignments.

#### 8.3.7 Step 7: Eviction

`check_evictions()` (§12.6) scans for unprotected contour cells past
$D_{\text{evict}}$, removes them, and runs a trailing rebalance for
eviction-triggered violations. Step 7 uses post-rebalance V-depths
(from Step 6) to determine eligibility. Eviction is restricted to
terminal G-nodes (zero children) — the structural shield.

Step 7 may remove entries that were created or promoted during
Steps 5–6. This interplay is the buffer oscillation discussed in §7.3.

---

### 8.4 Accounting Properties

#### 8.4.1 Single V-Entry Update

Each observation updates exactly one G-node's importance accumulator —
the receiver's. The G-Tree sum propagation in Step 4 maintains G-I1
for range queries but does not touch importance. This eliminates
double-counting: under the standard configuration, the V-Tree's total
importance equals the G-Tree root's sum; under absolute projection,
it equals the total absolute observation volume $\sum |\Delta_i|$.

#### 8.4.2 Routing as the Freeze Mechanism

When a G-node is internal (both children present), all observations
to its range are intercepted by children via `route_to_receiver`
(§5.2). The node's V-entry never receives $\Delta$. It is frozen at
its pre-split value — the fixed benchmark its children must exceed.

When one child is evicted, the parent absorbs the child's sum into
$g.\text{own}$ (§12.5), becomes the receiver for the vacated range,
and its V-entry unfreezes for that range. The G-Tree's routing
topology is the mechanism that freezes and unfreezes V-entries —
no explicit flag or lock is involved.

#### 8.4.3 Transient Invariant Windows

Between steps, the two trees pass through states where specific
invariants are temporarily violated. Under the sequential model
(§17.3), no external operation (query, sample, or concurrent
observation) occurs during these windows.

| After step | G-I1 (Sum)                               | V-I1 (V-Sum)                       | V-I3 (Uncle)                                            |
| ---------- | ---------------------------------------- | ---------------------------------- | ------------------------------------------------------- |
| Step 2     | **Violated** at receiver and ancestors   | Holds                              | Holds (V-Tree untouched)                                |
| Step 3     | **Violated** (G-sums not yet propagated) | **Restored** by `propagate_v_sums` | Violations detected and queued                          |
| Step 4     | **Restored** by sum propagation          | Holds                              | Violations still queued                                 |
| Step 5     | Holds                                    | Holds (new entries at $\nu$)       | Additional violations may be queued                     |
| Step 6     | Holds                                    | Holds                              | **Restored** by `rebalance()`                           |
| Step 6b    | Holds                                    | Holds                              | Holds (depth gates adjusted; no structural changes)     |
| Step 7     | Holds                                    | Holds                              | Holds (trailing rebalance resolves eviction violations) |

**Notable windows:**

- **Steps 2–4.** G-I1 is violated: the receiver's `own` has increased
  but `sum` fields along the ancestor path have not. No G-Tree query
  should execute during this window.

- **Steps 3–6.** V-I3 violations exist in the queue but are not yet
  resolved. The V-Tree's shape is valid (V-I1, V-I2, V-I5 all hold)
  but the competitive ranking does not reflect the new importance.
  Sampling during this window would use stale structural positions.

- **Steps 3–4 (cross-tree).** After Step 3, the V-Tree total reflects
  the new observation (importance updated and propagated), but the
  G-Tree root sum does not (sum propagation hasn't run). The two
  trees' totals disagree. This is cosmetic under the sequential
  model — no operation reads both totals — but concurrent
  implementations must ensure consistent snapshots (§17.3).

---

### 8.5 The Observation Type

The observation delta $\Delta$ need not be of the same type as the
stored accumulator $T$. It may be of a distinct **observation type**
$O$ that carries a mapping contract: given the current stored value
and a delta, produce the updated value. This decouples arithmetic
precision from storage — a tree storing integer counts may accept
floating-point deltas, performing the arithmetic in the wider type
and narrowing back for storage.

#### 8.5.1 Mapping Requirements

The observation type imposes no algebraic axioms of its own. It is a
deterministic mapping, not a monoid:

- $\text{accumulate}_T(v, \delta)$ must be deterministic (same inputs
  produce the same output).
- $\text{accumulate}_T(\text{zero}_T, \delta)$ must produce a
  well-defined $T$ — this serves as the $O \to T$ conversion path.
  If it produces garbage, any auxiliary sum maintenance drifts.

#### 8.5.2 When $O = T$

When the observation type equals the ledger type, the mapping reduces
to ordinary addition — no ceremony required. The `observe()`
pseudocode (§8.2) uses bare `+` as shorthand for this common case.

#### 8.5.3 Independent Multiplicative Mapping

A separate multiplicative mapping $\text{scale}(v, \delta)$ may be
defined independently of the additive path. It is semantically
independent from temporal modulation — "scale during observation"
and "modulate over time" are distinct operations that may have different
implementations. The built-in temporal filter (§14) uses concrete floating-point
parameters ($\text{att}$, $q$) that are properties of the filter, not of the
observation type.

---

### 8.6 Accumulation Configurations

The architecture is parameterised by a value space
$\mathcal{V} = (I, \oplus, \nu, \preceq)$ and a projection function $\pi$.
This section classifies common configurations by their algebraic properties
and operational consequences. The classification is descriptive — the
architecture does not store or branch on a mode parameter. It derives all
behavioral variants from the checkable properties P0–P5 (§2.3).

The observation flow separates two concerns:

- **Ledger accumulation** updates `g.own` and `g.sum` using $T$'s
  additive structure (§8.3.2, §8.3.4).
- **Importance accumulation** updates `g.importance` using the
  projection $\pi : I \times T \to I$ (§8.3.3, §2.6).

The V-Tree never sees $T$. It accesses `g.importance` through the
opaque importance interface (§2.1), using $\preceq$ for governance
and $\oplus$ for structural aggregation. The concrete meaning of
importance is determined entirely by the user's choice of $\pi$.

#### 8.6.1 Configuration Taxonomy

Each configuration is characterised by its value space and projection.
The value space determines which properties P0–P5 hold; the projection
determines how ledger data maps to importance.

| Configuration | Value Space                                                | Projection                                          | Properties     |
| ------------- | ---------------------------------------------------------- | --------------------------------------------------- | -------------- |
| **Standard**  | $(\mathbb{R}_{\geq 0},\; +,\; 0,\; \leq)$                  | Identity: $\pi(i, \delta) = i + \delta$             | P0–P5          |
| **Absolute**  | $(\mathbb{R}_{\geq 0},\; +,\; 0,\; \leq)$                  | Absolute: $\pi(i, \delta) = i + \lvert\delta\rvert$ | P0–P5          |
| **Signed**    | $(\mathbb{R},\; +,\; 0,\; \leq)$                           | Identity: $\pi(i, \delta) = i + \delta$             | P0, P3, P4, P5 |
| **Elevated**  | $(\mathbb{R}_{\geq 0},\; +,\; \nu,\; \leq)$ with $\nu > 0$ | Identity                                            | P0, P1, P5     |

#### 8.6.2 Standard Configuration (Default)

```
T = ℝ≥0,   I = ℝ≥0,   ν = 0,   ⊕ = +
π(current, Δ) = current + Δ
```

_Properties:_ P0–P5 all hold (§2.5.1). All features available.

Importance equals `g.own`. The two accumulators are redundant — a
single field suffices. G-I4 holds trivially (the reference points to
the sole accumulator). When all deltas are non-negative (the common
case: counting), closure is satisfied by construction and
$\nu = 0$ is the bottom element.

#### 8.6.3 Absolute Configuration

```
T = ℝ,     I = ℝ≥0,   ν = 0,   ⊕ = +
π(current, Δ) = current + |Δ|
```

_Properties:_ P0–P5 all hold. Same value space as standard; different projection.

The ledger is signed; importance tracks cumulative absolute activity.
`g.importance` diverges from `g.own` as soon as a negative $\Delta$
arrives. Closure is satisfied unconditionally ($|\Delta|$ is
always non-negative) and $\nu = 0$ is the bottom element. This
configuration retains all features despite mixed-sign observations.

#### 8.6.4 Signed Configuration

```
T = ℝ,     I = ℝ,     ν = 0,   ⊕ = +
π(current, Δ) = current + Δ
```

_Properties:_ P0, P3, P4, P5. P1 fails ($\mathbb{R}$ has no minimum).
P2 fails ($0 \not\preceq -1$). P3 holds ($0 + 0 = 0$).
P4 holds ($0 + a = a$). P5 holds (ordinary addition is associative).
These properties are independent of P1 under the corrected hierarchy
(§2.3.2) — the signed configuration demonstrates their independence.

Both types are signed. Governance (P0) works: the V-Tree can
compare, aggregate, and rebalance. What is unavailable when P1 fails:

- **Sampling** is undefined — no bottom element, so probability ratios
  are meaningless.
- **The Fibonacci depth bound** (§18.1) does not hold. Depth may
  reach $O(L)$ under adversarial sign patterns.
- **Violation-free insertion** is not guaranteed — new entries at
  importance $\nu = 0$ may exceed existing entries with negative
  importance (P2 requires $\nu \preceq a$ for all $a \in I$, which fails since $0 \not\preceq -1$).

What **is** available despite P1 failure:

- **The ghost eviction fast path** (§12.5 Step 2) — P4 holds
  ($0 + a = a$), so absorption of a ghost's importance is a no-op.
- **Violation-free structural nodes after split** — P3 holds
  ($0 + 0 = 0$), so the new structural node carries ground importance
  and cannot exceed any uncle.
- **Sum-propagation early termination** — P4 holds, so adding $\nu$
  is a no-op and propagation can stop.

This is a legitimate configuration when the user wants spatial
adaptation driven by net signed value but does not need proportional
sampling.

#### 8.6.5 Elevated Configuration

```
T = ℝ≥0,   I = ℝ≥0,   ν > 0,   ⊕ = +
π(current, Δ) = current + Δ
```

_Properties:_ P0, P1, P5. P2 fails ($\nu > 0$ but $0 \in I$ and $\nu \not\preceq 0$). P3 fails
($\nu + \nu = 2\nu \neq \nu$ for $\nu > 0$). P4 fails
($\nu + a \neq a$ for $\nu > 0$).

A value space may have a known bottom element ($\bot = 0$) while
$\nu$ is strictly above it — e.g., importance in $\mathbb{R}_{\geq 0}$
with $\nu = 1$. The floor exists; new entries start above it.

**Relationship to the signed configuration.** Elevated and signed are
**incomparable** in features. Elevated has P1, gaining **sampling** and
the **Fibonacci depth bound** — the most practically important non-governance
features. Signed has P3 and P4, gaining **violation-free structural nodes
after split**, the **ghost eviction fast path**, and **sum-propagation early
termination**. Neither strictly dominates the other. For most applications,
sampling is the decisive capability, making elevated the more practical
choice when $\nu > 0$ is desired.

**Less capable than standard/absolute.** Catalytic splits and
new-importance insertions are **not** violation-free (the newcomer at
$\nu$ may outrank existing light entries since P2 fails), requiring a
rebalance pass after every split. The ghost eviction fast path is
**unavailable** (P4 fails: $\nu + a \neq a$ for $\nu > 0$, so
absorption of a ghost's importance genuinely changes the parent's
importance — Steps 3–4 of §12.5 cannot be skipped). In practice this means
higher per-split overhead and slower contraction, but no loss of
correctness.

The three standard configurations (§2.7) avoid this — both standard
and absolute have $\nu = 0$, which is the minimum of $[0,\infty)$, satisfying P2 by
construction — but a user-supplied projection with a non-zero initial
bias would land here.

#### 8.6.6 Configuration Comparison

| Property                      | Standard | Absolute |   Signed    | Elevated |
| ----------------------------- | :------: | :------: | :---------: | :------: |
| Properties                    |  P0–P5   |  P0–P5   | P0,P3,P4,P5 | P0,P1,P5 |
| Sampling [P1]                 |    ✓     |    ✓     |      ✗      |    ✓     |
| Fibonacci bound [P1+P5]       |    ✓     |    ✓     |      ✗      |    ✓     |
| Ghost fast path [P4]          |    ✓     |    ✓     |      ✓      |    ✗     |
| Violation-free splits [P2+P3] |    ✓     |    ✓     |      ✗      |    ✗     |
| Mixed-sign observations       |    ✗     |    ✓     |      ✓      |    ✗     |
| Importance = own              |    ✓     |    ✗     |      ✓      |    ✓     |

All configurations use the same G-Tree routing and sum propagation. Only
the projection differs. All V-Tree structural machinery (rebalancing,
uncle constraint, structural aggregation) depends only on P0, which all
configurations provide.

> _Note (violation-free splits)._ The "Violation-free splits" row requires
> **both** P2 (new entries at importance $\nu$ cannot exceed any uncle, since $\nu \preceq a$ for all $a$) **and**
> P3 ($\nu \oplus \nu = \nu$, so the new structural node carries ground
> importance). Signed has P3 but not P2: the structural node is
> violation-free, but new entries may exceed existing entries with negative
> importance. The overall split is not violation-free.

> _Design note (the separation)._ The ledger type $T$ and the
> importance type are intentionally distinct concerns. The G-Tree
> stores $T$ — signed, complex, or vector-valued, whatever the domain
> needs. The V-Tree sees only importance through the opaque
> interface: ordered, aggregatable, optionally non-negative. The
> projection $\pi$ is the user's declaration of what
> "mattering" means — specifically, how the user projects ledger data
> into the governance signal. Under the standard configuration
> with non-negative deltas, the two types coincide and the distinction
> is invisible. It becomes meaningful when deltas carry mixed signs:
> the user chooses whether to project via absolute value (absolute
> configuration, retaining P0–P5) or identity (signed
> configuration, accepting P0 only for feature-gating purposes). Future
> projections (quadratic, windowed, entropy) are additional maps into
> the non-negative regime — P0–P5 is where the architecture's full
> guarantees live.

---

### 8.7 Eviction Absorption

When a child $g$ is evicted (§12.5), the parent $p$ absorbs the child's
importance. The operation is universal across all configurations.

#### 8.7.1 Universal Absorption Rule

When entry $g$ is evicted with parent $p$:

1. **Ledger absorption:** $p.\text{own} \leftarrow p.\text{own} + g.\text{sum}$ (§12.5 Step 1)
2. **Importance absorption:** $p.\text{importance} \leftarrow p.\text{importance} \oplus g.\text{importance}$

Step 2 uses the value space's aggregation operation $\oplus$, the same
operation used for structural sums (V-I1). This is correct for all value
spaces because:

- The evicted node is terminal (§12.3 condition 2), so $g.\text{importance}$
  is its complete importance.
- $\oplus$ is the architecture's aggregation operator — the same operation
  V-I1 uses to combine child importances.
- Conservation is maintained: the parent gains exactly what the removed
  entry carried.

#### 8.7.2 Aliasing Optimisation (Identity Projection)

When the projection is identity ($\pi(i, \delta) = i + \delta$) and
$\oplus = +$, the importance field is numerically equal to the own field at
all times for terminal nodes. Step 1 implicitly performs Step 2.
Implementations may use a single storage field for both and omit Step 2 as
a no-op.

#### 8.7.3 Verification Under Common Configurations

The following table verifies that the universal absorption rule (§8.7.1)
produces the correct result under each common configuration:

| Configuration                  | Concrete absorption                             | Conservation identity                                                       |
| ------------------------------ | ----------------------------------------------- | --------------------------------------------------------------------------- |
| Standard (identity projection) | `p.importance ← p.own` (after `p.own += g.sum`) | Parent gains `g.sum`; evicted entry (same value) removed. Net: zero.        |
| Absolute (absolute projection) | `p.importance ← p.importance + g.importance`    | Parent gains `g.importance`; evicted entry (same value) removed. Net: zero. |
| Signed (identity projection)   | `p.importance ← p.own` (after `p.own += g.sum`) | Same algebra as standard. Net: zero.                                        |

#### 8.7.4 Why the Absolute Configuration Uses `g.importance`, Not `|g.sum|`

Under the absolute configuration, the parent absorbs `g.importance`
($= \sum |\Delta_i|$ for the evicted node), _not_ `|g.sum|`
($= |\sum \Delta_i|$). Since $\sum|\Delta_i| \geq |\sum\Delta_i|$
by the triangle inequality, these differ whenever the evicted node
received mixed-sign observations. The universal rule — absorbing
`g.importance` via $\oplus$ — is correct for conservation: the V-Tree
total is unchanged because the parent gains exactly what the removed entry
carried.

#### 8.7.5 Structural Role

The value space and projection are structural properties of the graph —
they determine which properties P0–P5 hold, how eviction absorption
works, and which features are available. They are construction-time
parameters, not observation-level concerns.

---

### 8.8 Capability Decomposition

The ledger type $T$ need not support every operation the architecture
can provide. The core structure — observation, split, rebalance,
sum propagation, and violation detection — requires only the minimal
algebraic surface. Additional operations are layered as independent,
opt-in capabilities.

#### 8.8.1 Core Requirements

The core requires only two properties of $T$: a zero element and
addition.

Tracing through the core operations:

| Core operation              | Ledger operations used                                      |
| --------------------------- | ----------------------------------------------------------- |
| Step 2 (accumulate own)     | `+`                                                         |
| Step 4 (propagate sums)     | `+`                                                         |
| Split (initialize children) | `zero` (children start at $\text{zero}_T$)                  |
| Eviction absorption         | `+` ($p.\text{own} \leftarrow p.\text{own} + g.\text{sum}$) |
| Aligned range sum           | `+` and `zero` (null children contribute zero)              |

No core path requires subtraction, ordering, or scaling on $T$ itself.
Ordering is a property of the `Importance` type (§2.1), not the
ledger type. The architecture never compares two $T$ values in core
operations.

The core acceptance set is therefore any **commutative monoid**:
$(T,\, +,\, \text{zero})$ where `+` is commutative and associative
with identity `zero`. This is the widest possible acceptance set for
user-defined types.

#### 8.8.2 Extended Capabilities

Each extension is independent and opt-in:

| Capability            | Required operation on $T$                                                                                                            | Enables                                                                                                        |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------- |
| **Temporal scaling**  | $\text{modulate}(v, f) \to v'$ — multiplicative scaling by a factor $f \geq 0$; must satisfy $\text{modulate}(v, 0) = \text{zero}_T$ | Subband-adaptive temporal filter (§14): attenuation ($f < 1$), amplification ($f > 1$), annihilation ($f = 0$) |
| **Weighted sampling** | $\text{weight}(v) \to w$ — project to a non-negative scalar weight                                                                   | Proportional sampling (§6.5), PEWEI extraction (§PEWEI M-9)                                                      |
| **Range pro-ration**  | $\text{prorate}(v, \text{portion}, \text{total}) \to v'$, $\text{scale}(v, r) \to v'$                                                | Partial-overlap range queries (§5.5.2)                                                                         |
| **Subtraction**       | $a - b$ — additive inverse                                                                                                           | PEWEI refinement ($R = S - B$, §PEWEI M-2), sliding window decay (§14)                                           |
| **Diagnostics**       | $\text{to\_approx}(v) \to w$ — lossy scalar projection                                                                               | Invariant checking, debug output                                                                               |

> _Note (naming)._ The operation is called `modulate` — regime-neutral multiplicative scaling. It encompasses annihilation ($f = 0$), attenuation ($f < 1$), identity ($f = 1$), and amplification ($f > 1$). Crucially, `modulate` is a pure value operation: it scales accumulators in place but does not add, remove, or reparent any G-node. The G-Tree topology, contour structure, and plateau map are structurally unchanged by modulation — even at $f = 0$, where all values become zero but every node, edge, and contour cell persists. Structural changes (node removal) occur only through the separate eviction mechanism (§12), which may subsequently collect zeroed nodes if they sit past $D_{\text{evict}}$ in Va-depth. The $f = 0$ requirement ($\text{modulate}(v, 0) = \text{zero}_T$) is needed for annihilation support (§14.3). Types that cannot produce zero from multiplication (if any exist) lose annihilation but retain attenuation and amplification.

#### 8.8.3 Dependency Structure

```
                  Core
              (zero, add)
    /      |      \       \          \
Temporal  Sampling  Pro-ration  Subtraction  Diagnostics
Scaling
```

Five independent leaves — no capability depends on another
capability. A minimal $T$ implements only the core and gets observe
plus structural operations. Each additional capability is opt-in:
if $T$ cannot be scaled, temporal filtering does not compile; if $T$ has no
scalar weight projection, sampling does not compile; if $T$ has no
subtraction, the PEWEI's refinement computation and sliding window
decay are unavailable. There are no runtime surprises — the
capability boundary is a construction-time property.

All standard numeric types (integer and floating-point) support all
five extensions. A user-defined type (e.g., saturating arithmetic,
fixed-point) implements whichever subset matches its semantics.

> _Note (subtraction as a capability, not a core requirement)._ The
> core architecture never subtracts $T$ values. Subtraction appears
> in two user-facing contexts: the PEWEI's refinement computation
> $R = S - B$ (§PEWEI M-2), which is output processing, and sliding
> window decay (§14), which subtracts expired observations from
> accumulators. Both are optional. Integer types without general
> subtraction (e.g., saturating unsigned integers) are valid core
> ledger types.

---

### 8.9 Propagation Helpers

Two helper functions are referenced throughout the observation flow
and by other chapters. Both walk the V-Tree's parent chain,
recomputing aggregates.

#### 8.9.1 `propagate_v_sums`

Recomputes importance aggregates from the entry's parent to the
V-root. The entry's own importance is assumed to be already current.

```
function propagate_v_sums(v):
    p ← v.val_parent
    while p ≠ null:
        p.int ← sum of p.children's .int
        p ← p.val_parent
```

Cost: $O(h_V)$ — one addition per structural ancestor. At each
structural node, the sum uses the `Importance` interface's `Add`
method, never inspecting the concrete type.

#### 8.9.2 `propagate_v_sums_from`

Variant that includes the starting node in the recomputation. Used
after structural changes (child removal, collapse) where the
starting node's own importance aggregate is stale.

```
function propagate_v_sums_from(start):
    if start is structural:
        start.int ← sum of start.children's .int
    propagate_v_sums(start)
```

Cost: $O(h_V)$.

> _Cross-reference note._ The step numbering in this chapter is
> referenced by §3.2.4 (NaN rejection, implemented in Step 1 of §8.2), §7.5.2 ("Step 5 (split)",
> "Step 6 (rebalance)", "Step 7
> (evictions)"), §11.11.2 (ancestor violation walk, now inline in
> Step 3), and §12.6 (trailing rebalance). All cross-references
> remain valid under this revision. The violation-detection walk in
> Step 3 subsumes the §11.11.2 ancestor walk that was previously
> described but not invoked in the `observe()` pseudocode.
>
> §10.1 (`attempt_refine`) now explicitly calls `push_side_effect_violations`
> and `push_promoted_violations` (§11.11.1) for post-split violation
> detection and `update_plateau_map_after_split` (§5.6.7) for contour
> maintenance.

---

## Chapter 9. V-Tree Entry Management

### 9.1 New-Importance Insertion

**Purpose.** Add a G-node as a new V-Tree entry with importance $\nu$.

**Precondition.** $g$ has no G-children ($\neg\,\text{has\_dependents}(g)$)
and no existing V-entry ($g.\text{entry} = \text{null}$).

> _Design note._ This function specifies the general-purpose insertion
> algorithm. The specialized creation paths — catalytic split (§10.2),
> bootstrap split (§10.3), legacy promote (§11.6), and initialization
> (§15) — each wire the new entry into a context-specific V-Tree
> location with tailored structural bookkeeping. They implement
> equivalent but context-specific logic; this function serves as the
> reference algorithm for insertions without a predetermined target
> position.

```
function vtree_insert(g):
    e ← new V-Entry(int = ν, gnode = g, is_exposed = true, is_evictable = true)
    g.entry ← e

    // Case 1: empty V-Tree
    if V_root = null:
        V_root ← e
        e.val_parent ← null
        return

    // Case 2: V-root is a single entry
    if V_root is entry:
        s ← new V-Structural()
        s.int ← V_root.int + e.int
        s.has_evictable ← V_root.is_evictable or e.is_evictable
        s.children ← [V_root, e]
        V_root.val_parent ← s
        e.val_parent ← s
        s.val_parent ← null
        V_root ← s
        return

    // Case 3: general — descend toward a light entry, buddy-insert
    node ← V_root
    while node is structural:
        node ← argmin { c.int : c ∈ node.children }
    buddy ← node
    p ← buddy.val_parent

    s ← new V-Structural()
    s.int ← buddy.int + e.int
    s.has_evictable ← buddy.is_evictable or e.is_evictable
    s.children ← [buddy, e]
    buddy.val_parent ← s
    e.val_parent ← s
    s.val_parent ← p

    replace buddy with s in p.children
    propagate_v_sums_from(p)           // restore V-I1 at p and ancestors
    propagate_evictable_flags(p)
```

> _Note (greedy descent)._ The Case 3 descent follows the
> lightest-child path at each structural node. This is a greedy
> heuristic that finds a _light_ entry — not necessarily the globally
> lightest. A structural node with a small aggregate may contain a
> moderately light entry, while a sibling with a larger aggregate may
> contain an even lighter individual entry. The placement does not
> inserted. The heuristic governs only the V-Tree depth at which the
> new entry initially sits, which affects sampling cost but not
> structural integrity. Under P2 ($\nu \preceq a$ for all $a$), the new entry
> at importance $\nu$ cannot violate V-I3 regardless of where it is
> inserted.

**Invariant preservation.**

- **V-I1:** Case 2: $s.\text{int} = V\_\text{root}.\text{int} +
  e.\text{int} = V\_\text{root}.\text{int} + \text{New}()$. $s$ is the
  new V-root (no ancestors); no propagation needed. $\checkmark$
  Case 3: $s.\text{int} = \text{buddy.int} + e.\text{int}$. Replacing
  buddy with $s$ in $p$'s children changes $p$'s sum by $e.\text{int} =
  \nu$. `propagate_v_sums_from(p)` restores V-I1 at $p$ and
  all ancestors. $\checkmark$
  When $\nu$ is the additive identity (P4), $e.\text{int}$ contributes
  zero and the propagation walks to the root rewriting unchanged
  values. An implementation may add a change-detection guard to
  `propagate_v_sums_from` for early termination; the pseudocode in §8.9
  does not include one, so the walk costs $O(h_V)$ unconditionally.

- **V-I2:** $s$ has 2 children. Parent's child count unchanged
  (replacement, not addition). $\checkmark$

- **V-I3 (conditional).** When P2 holds ($\nu \preceq a$ for all $a$):
  $e.\text{int} = \nu \leq$ anything — the new entry has the minimum
  importance, so no violation is possible. Buddy's new uncles are its
  former siblings — and it was the lightest child (by the descent
  heuristic). $\checkmark$ **No rebalancing is required.**
  When P2 fails ($\nu$ is not the minimum):
  $e.\text{int}$ may exceed existing entries. The insertion is still
  structurally correct, but V-I3 is **not** guaranteed. A
  `rebalance()` pass is required after insertion.

- **V-I4:** One new entry for one G-node. $\checkmark$

- **V-I5:** $e$ is an entry (V-leaf). $s$ is structural. $\checkmark$

- **V-I6, V-I6b:** The precondition guarantees $g$ is terminal — no
  dependents and fully exposed. The hardcoded `is_exposed = true` and
  `is_evictable = true` are correct by the precondition. $\checkmark$

- **V-I7:** Propagated upward from insertion point via
  `propagate_evictable_flags(p)`. $\checkmark$

### 9.2 Entry Removal

**Purpose.** Remove a V-Entry from the V-Tree. Used by eviction (§12.6).

**Postconditions.** After `vtree_remove_leaf` returns:

1. The entry $v$ has been detached from the V-Tree and
   $v.\text{gnode}.\text{entry}$ set to null. The entry and its
   backing G-node have **not** been deallocated — the caller is
   responsible for calling `destroy(v)` and `destroy(v.\text{gnode})`
   (§12.5 Step 10).

2. V-I1 is restored at all structural ancestors via
   `propagate_v_sums_from`. V-I2 is maintained (3→2 transition or
   2-node collapse). V-I5 and V-I7 are maintained.

3. V-I3 is **not** guaranteed. The removal decreases ancestor
   importances, potentially weakening uncle shields at every level
   (§11.12 sources 6–9). **The caller must walk the ancestor chain
   after removal, checking siblings' children for violations at each
   level (§11.11.3), and enqueue any newly violated nodes for the
   trailing rebalance.**

4. V-I0 is **not** guarded. If $v$ is the last entry, the V-Tree
   becomes empty. The caller must ensure this does not occur — the
   root exemption (§12.5) is the standard guard.

```
function vtree_remove_leaf(v):
    p ← v.val_parent

    if p = null:
        V_root ← null
        v.gnode.entry ← null
        return

    p.children ← p.children \ {v}
    v.gnode.entry ← null

    if |p.children| ≥ 2:
        propagate_v_sums_from(p)
        propagate_evictable_flags(p)
        return

    // p has 1 child — collapse p into its sole child
    sole ← p.children[0]
    g ← p.val_parent
    sole.val_parent ← g

    if g = null:
        V_root ← sole
    else:
        replace p with sole in g.children
        propagate_v_sums_from(g)
        propagate_evictable_flags(g)

    destroy(p)
```

**Invariant notes.**

- **V-I2:** If $p$ was a 3-node, it becomes a 2-node — valid. If $p$
  was a 2-node, the collapse replaces $p$ with its sole child. The
  structural collapse itself is a single step, but the resulting
  violation propagation may cascade upward (see Postcondition 3).
  $\checkmark$

- **V-I3:** Uncle relationships change at **every ancestor level**,
  not just the immediate neighbourhood. Both sub-cases (3→2 and
  2-node collapse) trigger `propagate_v_sums_from`, which decreases
  the importance of every structural ancestor up to the root. Each
  decreased ancestor serves as uncle to its siblings' children at the
  grandparent level. A node that was safe before the removal may now
  exceed its weakened uncle. See Postcondition 3 — the caller handles
  this.

- **V-I5:** Only entries are removed. Structural nodes may be
  destroyed by collapse. No type transmutation. $\checkmark$

- **V-I7:** Evictable flags propagated after structural changes.
  $\checkmark$

### 9.3 Evictable Flag Propagation

```
function propagate_evictable_flags(v):
    while v ≠ null:
        if v is entry:
            v ← v.val_parent
            continue
        old ← v.has_evictable
        v.has_evictable ← any child c:
            if c is entry: c.is_evictable
            else: c.has_evictable
        if v.has_evictable = old: return   // no change, stop early
        v ← v.val_parent
```

Cost: $O(h_V)$ worst case, but early-termination on unchanged flags makes
this $O(1)$ typical. Piggy-backs on operations that already traverse the
V-Tree for sum propagation.

The early termination is sound: `has_evictable` is a monotone OR over
descendant entries' `is_evictable` flags. If the recomputed value at
node $v$ is unchanged, all ancestors of $v$ — which each compute OR
over $v$'s contribution and other children's contributions (which
haven't changed) — must also be unchanged.

> _Note._ The function handles the case where $v$ is an entry (skip to
> parent). At all current call sites, $v$ is a structural node or the
> V-parent of an entry. The entry-skip path is a defensive guard for
> generality.

---

## Chapter 10. Contour Refinement

**Midpoint convention.** All pseudocode in this chapter uses
$\text{midpoint}(l, r)$ as shorthand for the stabilised midpoint
formula of §3.2.3:

$$\text{midpoint}(l,\, r) \;\equiv\; l \;+\; (r - l)\,/\,2$$

The naive formula $(l + r) / 2$ risks overflow (§3.2.3). The
stabilised form is exact for all dyadic intervals within the IEEE 754
normal range.

### 10.1 Attempt Refine

When a fully exposed contour cell has accumulated sufficient importance
($g.\text{importance} > \theta$) and its V-entry holds a shallow enough
competitive position ($\text{depth}_V \leq D_{\text{create}}$), the
cell is refined: its range is subdivided into two finer contour
cells.

```
function attempt_refine(g):
    if has_dependents(g): return          // not fully exposed

    // Split guards (§3.2.1): both are required.
    // The finality predicate prevents refinement past the configured
    // maximum depth N. For integer types, it fires at unit cells
    // (width 1). For floating-point types, it fires at depth = N —
    // the only guard at that depth, since the midpoint remains valid
    // for width-1 float intervals.
    // The midpoint guard prevents degenerate splits where the
    // computed midpoint collapses to an endpoint due to arithmetic
    // limitations (integer unit cells or floating-point rounding at
    // extreme depths below N).
    if is_final(g.l, g.r, depth_geo(g), N): return   // finality (§3.2.1)
    m ← midpoint(g.l, g.r)                            // stabilised formula (§3.2.3)
    if m ≤ g.l: return                                 // midpoint guard (§3.2.1)

    if g.importance ≤ θ: return
    if g.entry = null: return

    // Bootstrap case: entry is V-root with no parent
    if g.entry.val_parent = null:
        bootstrap_split(g)

        // Post-split violation check (§10.4). Under the standard
        // configuration (P2 + P3), these find nothing. Under non-standard
        // configurations, push_side_effect_violations catches violations
        // from new entries (le, re) exceeding their uncle (g.entry) —
        // possible when P1 fails and the uncle's importance
        // is negative, or when ν ⊕ ν ≠ ν (P3 fails). No ancestor walk
        // needed: V_root has no parent.
        push_side_effect_violations(V_root)

        update_plateau_map_after_split(g)
        return

    p ← g.entry.val_parent

    // Preprocessing: ensure 2-node parent (always isolate heaviest)
    if |p.children| = 3:
        merged ← contract(p, isolate = heaviest_child(p))
        push_side_effect_violations(p)
        push_side_effect_violations(merged)         // §11.11.1
        // Check all children of p for violations — no skip needed
        // because no node is being promoted in this context (contrast
        // with §11.9 Phase 1, which skips the node under resolution).
        push_promoted_violations(p)                 // §11.11.1
        p ← g.entry.val_parent

    // Depth gate (checked against post-contraction position)
    if depth_V(g.entry) > D_create: return

    catalytic_split(g)

    // Post-split violation check. Under the standard configuration
    // (P2: ν ⪯ a for all a, and P3: ν ⊕ ν = ν),
    // catalytic splits are violation-free (§10.4) and these checks
    // find nothing. Under non-standard configurations, they catch:
    // (a) new entries exceeding negative uncles (when P1 fails
    //     — e.g., signed configuration);
    // (b) the structural node s exceeding uncles (when
    //     ν ⊕ ν ≠ ν — P3 failure).
    // Cost: O(1) — bounded number of is_violated calls. In the
    // standard case, all return false immediately.
    push_side_effect_violations(p)        // grandchildren of p
    push_promoted_violations(p)           // children of p (including s)

    // When ν ⊕ ν ≠ ν (P3 failure), the
    // V-sum increase propagated by catalytic_split's
    // propagate_v_sums_from(p) may push p or its ancestors past
    // their uncle thresholds. Under the standard configuration,
    // ν ⊕ ν = ν and no V-sum increase occurred — the
    // propagation rewrote unchanged values and this walk is skipped.
    if ν ⊕ ν ≠ ν:
        check_id ← p
        while check_id ≠ null:
            if is_violated(check_id): push check_id
            check_id ← check_id.val_parent

    // Update plateau ordered map (§5.6.7 split maintenance)
    update_plateau_map_after_split(g)
```

The guard is "is this node fully exposed?" — no dependents, no uncovered
range ambiguity. Semi-internal nodes are handled exclusively by
legacy promotion in §11.6.

> _Note (preprocessing cost)._ The preprocessing contraction fires
> whenever $p$ is a 3-node, even if the depth gate will subsequently
> reject the split. Since contraction can increase $g.\text{entry}$'s
> depth by at most 1 (if $g.\text{entry}$ is merged rather than
> isolated), a fast pre-check
> `if depth_V(g.entry) > D_create + 1: return` could avoid the
> wasted work. This is an optimisation, not a correctness concern —
> the contraction is structurally valid and its side-effect violations
> are resolved in Step 6's rebalance.

> _Note (violation check cost)._ The unconditional O(1) push calls
> (`push_side_effect_violations` and `push_promoted_violations`) add a
> bounded constant number of `is_violated` checks per split (§11.11.4).
> Under the standard configuration, every check returns false. The
> conditional O(h_V) ancestor walk fires only when $\nu \oplus \nu \neq \nu$
> (P3 failure), which excludes the standard configuration ($\nu = 0$) and
> the absolute configuration ($\nu = 0$). For standard usage, the total
> overhead of the violation checks is a bounded constant — negligible
> relative to the split's allocation and propagation costs.

### 10.2 Catalytic Split

**Precondition.** $g$ is a fully exposed G-node (zero children).
$g.\text{entry.val\_parent}$ is a 2-node.
$\text{depth}_V(g.\text{entry}) \leq D_{\text{create}}$.

```
function catalytic_split(g):
    m ← midpoint(g.l, g.r)                // stabilised formula (§3.2.3)

    // G-Tree: create children
    left  ← new G-Node([g.l, m), sum = 0, own = 0, importance = ν)
    right ← new G-Node([m, g.r), sum = 0, own = 0, importance = ν)
    g.left  ← left
    g.right ← right
    left.geo_parent  ← g
    right.geo_parent ← g

    // V-Tree: create entries for children
    le ← new V-Entry(int = ν, gnode = left, is_exposed = true, is_evictable = true)
    re ← new V-Entry(int = ν, gnode = right, is_exposed = true, is_evictable = true)
    left.entry  ← le
    right.entry ← re

    // V-Tree: structural node grouping the children
    s ← new V-Structural()
    s.int ← le.int + re.int              // = ν ⊕ ν; see invariant note
    s.has_evictable ← true
    s.children ← [le, re]
    le.val_parent ← s
    re.val_parent ← s

    // Add s as third child of g.entry's parent
    p ← g.entry.val_parent
    p.children ← p.children ∪ {s}
    s.val_parent ← p

    // Restore V-I1 at p and ancestors.
    // p.int must increase by s.int. When ν is the additive
    // identity (P4), s.int = 0 and the propagation rewrites unchanged
    // values. When P4 fails, s.int > 0 and the
    // propagation makes a real correction.
    propagate_v_sums_from(p)

    // Update flags: g is no longer on the contour and now has dependents
    g.entry.is_exposed ← false
    g.entry.is_evictable ← false             // maintain V-I6b: g now has children
    propagate_evictable_flags(p)
```

**Result.** The G-parent's V-entry becomes uncle to its own G-children's
entries. Shown for the standard modes where $\text{New}() = 0$:

```
p (3-node)
├── g.entry (int = I, exposed = false)    ← uncle, FROZEN, eviction-immune
├── existing_sibling (int = E)             ← also uncle
└── s (structural, int = 0)
    ├── left.entry (int = 0, exposed = true)
    └── right.entry (int = 0, exposed = true)
```

The parent's entry is now frozen: both G-children exist, so all observations
to $[g.l, g.r)$ route to children via `route_to_receiver`. The parent's own
value — and therefore its entry importance — will not change until a child is
evicted. The parent is also eviction-immune: it has dependents.

**Invariant preservation.**

- **G-I1:** $g.\text{sum}$ unchanged. Children start at 0.
  $g.\text{own} = g.\text{sum}$. $\checkmark$
- **G-I4:** Children's entries have $\text{int} = \nu =
  \text{importance}$. Parent's entry unchanged. (§4.2 reference semantics.) $\checkmark$
- **V-I1:** $s.\text{int} = \text{le.int} + \text{re.int} =
  \nu \oplus \nu$. `propagate_v_sums_from(p)` restores
  $p.\text{int}$ and all ancestors. When P4 holds,
  $\nu \oplus \nu = \nu$ and $s$
  contributes zero to $p$'s sum — the propagation rewrites unchanged
  values. $\checkmark$
- **V-I2:** $p$ is now a 3-node. $s$ is a 2-node. $\checkmark$
- **V-I5:** New entries are V-leaves. New structural node is V-internal.
  $\checkmark$
- **V-I6:** $g.\text{entry.is\_exposed}$ correctly set to false (now
  has children). New entries correctly set to true (exposed). $\checkmark$
- **V-I6b:** $g.\text{entry.is\_evictable}$ correctly set to false (now
  has dependents). New entries correctly set to true (terminal). $\checkmark$
- **V-I7:** Propagated from $p$ upward. $\checkmark$

**V-I3 is preserved without rebalancing** when P2 holds. See §10.4.
When P2 fails, violations are caught by the
`push_side_effect_violations(p)` call in §10.1 after `catalytic_split`
returns.

### 10.3 Bootstrap Split

When the V-Tree contains a single entry (the G-root at initialization), the
first split is handled specially.

```
function bootstrap_split(g):
    m ← midpoint(g.l, g.r)                // stabilised formula (§3.2.3)

    // G-Tree: create children
    left  ← new G-Node([g.l, m), sum = 0, own = 0, importance = ν)
    right ← new G-Node([m, g.r), sum = 0, own = 0, importance = ν)
    g.left ← left;  g.right ← right
    left.geo_parent ← g;  right.geo_parent ← g

    // V-Tree: create entries for children
    le ← new V-Entry(int = ν, gnode = left, is_exposed = true, is_evictable = true)
    re ← new V-Entry(int = ν, gnode = right, is_exposed = true, is_evictable = true)
    left.entry ← le;  right.entry ← re

    // V-Tree: structural node for children
    child_s ← new V-Structural()
    child_s.int ← le.int + re.int        // = ν ⊕ ν
    child_s.has_evictable ← true
    child_s.children ← [le, re]
    le.val_parent ← child_s
    re.val_parent ← child_s

    // V-Tree: structural root holding g.entry and child_s
    root_s ← new V-Structural()
    root_s.int ← g.entry.int + child_s.int
    root_s.has_evictable ← true
    root_s.children ← [g.entry, child_s]
    g.entry.val_parent ← root_s
    child_s.val_parent ← root_s
    root_s.val_parent ← null

    // Update flags: g is no longer on the contour and now has dependents
    g.entry.is_exposed ← false
    g.entry.is_evictable ← false             // maintain V-I6b: g now has children

    V_root ← root_s
```

**Result.** Shown for the standard configuration where $\nu = 0$:

```
root_s (structural, 2-node, int = I)
├── g.entry (int = I, exposed = false)  ← FROZEN, eviction-immune
└── child_s (structural, int = 0)
    ├── le (int = 0, exposed = true)
    └── re (int = 0, exposed = true)
```

In general, $\text{child\_s.int} = \nu \oplus \nu$ and
$\text{root\_s.int} = I + \nu \oplus \nu$, where $I$ is
$g.\text{entry.int}$ at split time. When P4 holds ($\nu$ is the
identity), $\text{child\_s.int} = \nu$ and $\text{root\_s.int} = I$.

**Invariant preservation.**

- **G-I1:** $g.\text{sum}$ unchanged. Children start at 0. $\checkmark$
- **G-I4:** Children's entries reference their G-nodes' importance
  accumulators (initialized to $\nu$). Parent's entry unchanged.
  (§4.2 reference semantics.) $\checkmark$
- **V-I1:** $\text{child\_s.int} = \text{le.int} + \text{re.int} =
  \nu \oplus \nu$. $\text{root\_s.int} =
  g.\text{entry.int} + \text{child\_s.int}$. $\text{root\_s}$ is the
  V-root (no ancestors). $\checkmark$
- **V-I2:** $\text{root\_s}$ is a 2-node. $\text{child\_s}$ is a
  2-node. $\checkmark$
- **V-I3 (when P2 holds):** $\text{le}(\nu)$, uncle
  $g.\text{entry}(I)$. Since $\nu \preceq a$ for all $a \in I$:
  $\nu \leq I$. $\checkmark$
  When P2 fails: violations are caught by the
  `push_side_effect_violations(V\_root)` call in §10.1 after
  `bootstrap_split` returns.
- **V-I5:** New entries are V-leaves. New structural nodes are
  V-internal. $\checkmark$
- **V-I6:** $g.\text{entry.is\_exposed}$ correctly set to false.
  New entries correctly set to true. $\checkmark$
- **V-I6b:** $g.\text{entry.is\_evictable}$ correctly set to false.
  New entries correctly set to true. $\checkmark$

### 10.4 Why Catalytic Splits Create No Violations

After a catalytic split, the new structural node $s$ becomes a sibling of
$g.\text{entry}(I)$ and the existing sibling ($E$) under the now-3-node
parent $p$.

**When P2 holds ($\forall\, a \in I:\; \nu \preceq a$):**

**New children:** importance $\nu$ (the minimum element under P2), uncles
$\{g.\text{entry}(I), \text{existing}(E)\}$. Max uncle $\geq I >
\nu$ (the split guard requires $I > \theta > \nu$). P2 gives $\nu \preceq a$ for all $a \in I$. A freshly created entry has importance $\nu$, which satisfies $\nu \preceq u$ for every uncle $u$. Therefore no V-I3 violation is created.
[P2: violation-free insertion — new entry $\preceq$ all existing entries.]

**New structural node $s$:** importance $\nu \oplus \nu$.
When P3 holds ($\nu \oplus \nu = \nu$), $s.\text{int}$ equals
the bottom element and cannot exceed any uncle — the same argument as
for the new children applies. Under P2, $\nu$ is the minimum element of $I$, so $\nu \preceq u$ for every uncle $u$. $\checkmark$
[P3: violation-free structural nodes after split.]

When P3 fails ($\nu \oplus \nu \neq \nu$), $\nu \oplus \nu$ may be strictly
greater than $\nu$. An uncle whose importance equals $\nu$ (e.g., a freshly
created entry from a prior split that has not yet received observations)
would be exceeded, making $s$ violated. This case is caught by the
unconditional `push_promoted_violations(p)` call in §10.1 after
`catalytic_split` returns, which checks $s$ against its uncle context.

> _Note (collapse result)._ Under ordinary addition, P3 holds if and only
> if $\nu = 0$ (§2.5), which is the case for the standard and
> absolute configurations. P4 (identity) implies P3 but is
> not required for this argument — only the idempotent-ground property
> is needed. A user-defined importance type with non-zero $\nu$ could
> satisfy P2 ($\nu \preceq a$ for all $a$) while failing P3 ($\nu \oplus \nu \neq \nu$).
> Such types are valid but lose the violation-free split guarantee;
> the §10.1 fallback handles them.

**Existing grandchildren through the existing sibling:** they previously had
uncle $g.\text{entry}(I)$ (when $p$ was a 2-node). Now they have uncles
$\{g.\text{entry}(I),\; s(\nu \oplus \nu)\}$. Their max uncle
is still at least $I$ — unchanged or increased. **No change to their
constraint.**

**Existing grandchildren through $g.\text{entry}$:** none — $g.\text{entry}$
is an entry (V-leaf, by V-I5), so it has no V-children.

**Summary.** Catalytic splits are violation-free when two conditions both
hold: (1) P2 — new entries cannot exceed any uncle ($\nu \preceq a$ for all $a \in I$);
(2) P3 — the structural node cannot exceed any uncle
($\nu \oplus \nu = \nu$). The standard and absolute configurations satisfy
both conditions by construction ($\nu = 0$). When either condition fails,
violations are possible and are caught by the unconditional push calls in
§10.1.

**When P2 fails:**
the new entries' importance $\nu$ may exceed existing entries'
importance (e.g., when uncles carry negative importance under the signed
configuration, or when $\nu$ is not the minimum under an elevated configuration).
The split is structurally correct, but V-I3 is **not** guaranteed. The
`push_side_effect_violations(p)` and `push_promoted_violations(p)` calls
in §10.1 enqueue any violations for Step 6's `rebalance()` pass.

---

## Chapter 11. V-Tree Rebalancing

When an observation causes an entry's importance to exceed every one of its uncles, the V-Tree must restructure to restore V-I3. This chapter specifies the rebalancing machinery: the violation work queue that mediates between detection and resolution (§11.1), the detection function and four resolution primitives (§§11.2–11.6), the dispatch logic that selects and composes them (§§11.7–11.10), and the correctness arguments that guarantee every violation is found and every rebalance terminates (§§11.11–11.13).

> _Organisational note._ §11.1 defines the **violation work queue** — the graph-level communication channel between violation detection and resolution. §§11.2–11.6 define the **primitives**: what a violation is and the four operations that resolve one. §§11.7–11.10 define the **orchestration**: the decision table, the rebalance loop, the resolve function, and the escalation mechanism that prevents cycles. §§11.11–11.13 establish **correctness**: exhaustive side-effect tracking, the complete catalogue of violation sources, and the termination proof.

---

### 11.1 Violation Work Queue

The **violation work queue** is a graph-level collection of V-node handles suspected of violating V-I3 (the max-uncle constraint). It is the communication channel between violation detection — which discovers potential violations during structural changes and importance updates — and the rebalance loop (§11.8), which resolves them.

#### 11.1.1 Declaration

The queue is a field of the graph object:

| Field             | Type                         | Initial value |
| ----------------- | ---------------------------- | ------------- |
| `violation_queue` | Collection of V-node handles | Empty         |

The element type is a handle (pointer, index, or arena key) to a V-node — either a V-Entry or a V-Structural node. Handles may reference nodes that have been destroyed since they were enqueued; the drain loop guards against this with `slot_occupied` (§11.8).

#### 11.1.2 Interface

| Operation                    | Signature          | Semantics                                               |
| ---------------------------- | ------------------ | ------------------------------------------------------- |
| `violation_queue.push(c)`    | V-node handle → () | Add `c` to the collection. Duplicates are permitted.    |
| `violation_queue.pop()`      | () → V-node handle | Remove and return one element. Precondition: non-empty. |
| `violation_queue.is_empty()` | () → bool          | True when no elements remain.                           |

**Duplicate tolerance.** The same V-node may be pushed multiple times — by different push functions, by the same push function in different contexts, or by overlapping ancestor walks. The drain loop (§11.8) filters duplicates and stale entries: `is_violated` (§11.2) is re-checked before acting, so redundant pushes are harmless no-ops at resolution time.

**Pop order.** Any pop order is correct. The `is_violated` re-check makes order irrelevant for correctness — every popped node is verified before resolution, filtering entries that were stale at pop time or resolved by a prior iteration. A simple list with last-in-first-out pop is sufficient. No priority queue is required.

A deepest-first heuristic — resolving violations at greater V-depth before shallower ones — prevents higher-level restructurings from disturbing lower-level corrections. This can reduce the number of stale queue entries but does not affect correctness. With LIFO pop and the ancestor-walk push order (Step 3 of §8.2, which pushes from entry to root), the shallowest violations are popped first — the opposite of deepest-first. Where ordering matters for performance, an explicit depth-keyed priority queue may be used.

#### 11.1.3 Lifecycle Invariant

$$\textbf{Q-I1 (Boundary Emptiness):}\quad \text{The violation queue is empty at the entry and exit of every public operation.}$$

Public operations are `observe()` (§8.2), `decay()` (§14.4), and standalone `check_evictions()` (§12.6). Q-I1 is maintained by the following protocol:

- **Population.** The queue is populated during the interior of a public operation by the push functions (§11.11) and by ancestor violation walks (§8.2 Step 3, §12.5 Step 4, §14.4 Phase 3).
- **Drainage.** The queue is drained by `rebalance()` (§11.8), which is called at every point where violations may have been pushed: Step 6 of `observe()`, Phase 3 of `check_evictions()`, and Phase 4 of `decay()`.
- **Transient non-emptiness.** Between population and drainage within the interior of a public operation, the queue may be non-empty. No external operation (query, sample, or concurrent observation) may execute during this window under the sequential model (§17.3).

Q-I1 follows from the drain protocol: every code path that pushes to the queue subsequently calls `rebalance()` before returning to the caller, and `rebalance()` drains the queue to empty — guaranteed by the termination proof (§11.13).

Within a single `observe()` call, two drain points exist: Step 6 drains violations from Steps 3 and 5; Phase 3 of `check_evictions()` (Step 7) drains violations from Phase 2 evictions. Between these two drain points, the queue is empty — Step 6 drained it, and Steps 6b–7 Phase 2 repopulate it before Phase 3 drains again. The same two-drain pattern appears in `decay()`: Phase 4 drains violations from Phase 3, and Phase 5's `check_evictions()` drains eviction-triggered violations. Q-I1 holds at the outer boundary in both cases.

#### 11.1.4 Population Discipline

The queue is populated by exactly two mechanisms:

**Ancestor violation walks.** A walk from a V-node to the V-root, checking `is_violated` (§11.2) at each level and pushing violators. Four sites use this mechanism:

| Site          | Trigger                                                              | Walk starts at                 |
| ------------- | -------------------------------------------------------------------- | ------------------------------ |
| §8.2 Step 3   | Importance increase from observation                                 | The receiving entry            |
| §10.1         | V-sum increase from non-idempotent split ($\nu \oplus \nu \neq \nu$) | The split parent `p`           |
| §12.5 Step 4  | Importance increase from eviction absorption                         | The absorbing G-parent's entry |
| §14.4 Phase 3 | Importance change from temporal scaling                              | Each affected entry            |

**Push functions (§11.11).** Context-specific functions that check bounded neighbourhoods for side-effect violations created by structural changes. Each push function is called immediately after the structural operation that may have created the violations. The push functions are:

| Function                                       | Context                       | Checks                                        |
| ---------------------------------------------- | ----------------------------- | --------------------------------------------- |
| `push_side_effect_violations` (§11.11.1)       | Contraction or promotion      | Grandchildren of restructured node            |
| `push_promoted_violations` (§11.11.1)          | Any child-set change          | Children of restructured node                 |
| `push_contraction_child_violations` (§11.11.1) | Contraction during resolve    | Children of contracted node, excluding target |
| `push_leaf_removal_violations` (§11.11.3)      | V-entry removal               | Siblings' children at each ancestor level     |
| `push_collapse_violations` (§11.11.3)          | 2-node collapse after removal | Children of surviving sibling                 |
| `push_remaining_sibling_violations` (§11.11.3) | 3→2 transition after removal  | Remaining siblings' children                  |
| `push_cousin_violations` (§11.11.3)            | 2-node collapse after removal | Cousins' children at grandparent level        |

No other code path may push to the queue. The population discipline ensures that every violation that can exist after a structural change or importance update is discovered and enqueued before the next `rebalance()` call.

#### 11.1.5 Notation Convention

Throughout this specification, bare `push c` in pseudocode is shorthand for `violation_queue.push(c)`. The target is always the graph's violation work queue; no other queue exists.

---

### 11.2 Violation Detection

```
function is_violated(c) → bool:
    p ← c.val_parent
    if p = null: return false
    g ← p.val_parent
    if g = null: return false
    max_uncle ← max { u.int : u ∈ siblings(p, g) }
    return c.int > max_uncle
```

A node violates V-I3 only when it is strictly heavier than **every** uncle:

- **2-node grandparent (one uncle):** $c > u$.
- **3-node grandparent (two uncles):** $c > u_1$ AND $c > u_2$.

Nodes at V-depth 0 or 1 have no grandparent and are unconstrained by V-I3. The null-grandparent guard handles this by returning false. This is an important architectural property: the shallowest entries are free to hold any importance, which is why the heaviest entries naturally sit at depth 1.

Violations are rare and meaningful. Three siblings of comparable importance coexist indefinitely under a 3-node parent — a violation requires beating _both_ uncles. The V-Tree restructures only when a node dramatically outgrows its entire neighbourhood, not on every minor importance fluctuation.

---

### 11.3 Contraction (3-Node → 2-Node)

**Purpose.** Reduce a 3-node to a 2-node by merging two of its children under a new structural node. Every promotion variant requires its parent (or grandparent) to be a 2-node; contraction is the preparatory step that achieves this.

**Rule.** Always isolate the child with **highest importance**. Merge the other two.

**Precondition.** $p$ is a structural node with $|p.\text{children}| = 3$.

```
function contract(p, isolate) → V-Structural:
    assert |p.children| = 3
    {a, b} ← p.children \ {isolate}

    s ← new V-Structural()
    s.int ← a.int + b.int
    s.children ← [a, b]
    a.val_parent ← s
    b.val_parent ← s
    s.val_parent ← p

    // Evictable flag: read the appropriate flag from each
    // child depending on its type
    if a is entry: fa ← a.is_evictable
    else: fa ← a.has_evictable
    if b is entry: fb ← b.is_evictable
    else: fb ← b.has_evictable
    s.has_evictable ← fa or fb

    p.children ← [isolate, s]
    propagate_evictable_flags(p)

    // V-I1: p.int = isolate.int + a.int + b.int
    //      = isolate.int + s.int — algebraic rearrangement,
    //      numerically unchanged. No propagation needed.
    // P5 required: the regrouping (h ⊕ a) ⊕ b → h ⊕ (a ⊕ b) is exact
    // only under associativity. Without P5, call:
    //   propagate_v_sums_from(p)

    return s   // the merged node
```

**V-I3 analysis.** Contraction changes uncle relationships at two levels. The analysis must be precise about which nodes are genuinely at risk.

_Children of $s$ (the merged pair $a$ and $b$)._ Before contraction, $a$ and $b$ were children of $p$ and each had uncles drawn from $p$'s other children — including `isolate`. After contraction, $a$ and $b$ are children of $s$ whose sole sibling under $s$ is each other; their uncle at the grandparent level is `isolate`. But their _grandchildren-level_ uncle context is different: children of $a$ now have uncle $b$ (instead of some subset of $p$'s children), and vice versa. Since `isolate` was the heaviest child, $a.\text{int} \leq \text{isolate.int}$ and $b.\text{int} \leq \text{isolate.int}$, so $a$ and $b$ themselves cannot be violated against `isolate`. $\checkmark$ However, **children of $a$ and children of $b$** face weakened uncles: before contraction, their max uncle included `isolate`; after, it is the other merged child only. **May create violations.**

_The merged node $s$ and the isolated node._ After contraction, $p$ is a 2-node under its own parent. Both `isolate` and $s$ face a new uncle context at $p$'s grandparent level. The isolated node's V-I3 status is **unchanged**: it was a child of $p$ with the same uncle set (siblings of $p$ at the grandparent), and its importance is unchanged. The merged node $s$ is new and has $s.\text{int} = a.\text{int} + b.\text{int}$; this aggregate may exceed uncles that neither $a$ nor $b$ individually exceeded. **$s$ may create a violation; `isolate`'s V-I3 status is unchanged.**

These side effects are bounded in count and caught by the push functions (§11.11).

**V-I1 preservation.** The parent's stored aggregate was computed under the old parenthesisation (e.g., $(h \oplus a) \oplus b$). After contraction, the children are $\{h, s\}$ with $s.\text{int} = a \oplus b$, and the new aggregate should be $h \oplus (a \oplus b)$. Under P5, these are equal and V-I1 holds without propagation. Without P5, the stored aggregate is stale and `propagate_v_sums_from(p)` must be called, at $O(h_V)$ cost.

**Key property.** When contraction is the first phase of `resolve` (§11.9), the merged node's aggregate importance may shield the triggering node, resolving the original violation outright. The dispatcher re-checks after each contraction.

---

### 11.4 Standard Promote (Explode a 2-Child Structural Node)

**Precondition.** $c$ is structural with exactly 2 children $\{c_1, c_2\}$. Parent $p$ is a 2-node.

**Effect.** Destroy $c$. Install $c_1$ and $c_2$ as direct children of $p$. $p$ becomes a 3-node.

```
function standard_promote(c):
    assert c is structural and |c.children| = 2
    p ← c.val_parent
    assert |p.children| = 2
    s ← sibling(c, p)
    {c₁, c₂} ← c.children

    p.children ← [c₁, c₂, s]
    c₁.val_parent ← p
    c₂.val_parent ← p

    destroy(c)
    propagate_evictable_flags(p)

    // V-I1: p.int was c.int + s.int = (c₁.int + c₂.int) + s.int.
    // Now p.int should be c₁.int + c₂.int + s.int — the same value.
    // Algebraic rearrangement; no propagation needed.
    // P5 required: the regrouping (c₁ ⊕ c₂) ⊕ s → c₁ ⊕ c₂ ⊕ s is exact
    // only under associativity. Without P5, call:
    //   propagate_v_sums_from(p)
```

**Why this helps.** $c$ was heavy. After explosion, $c_1$ and $c_2$ are individually lighter. The heavy mass is dispersed across the parent's children.

**V-I1 preservation.** The parent's stored aggregate was $(c_1 \oplus c_2) \oplus s$. After promotion, the children are $\{c_1, c_2, s\}$ and the recomputed aggregate is $c_1 \oplus c_2 \oplus s$ under the implementation's fixed parenthesisation. Under P5, these are equal and V-I1 holds without propagation. Without P5, the stored aggregate is stale and `propagate_v_sums_from(p)` must be called, at $O(h_V)$ cost.

**Side effects.** Children of $s$ previously had uncle $c$ with $c.\text{int} = c_1.\text{int} + c_2.\text{int}$. After promotion, their max uncle is $\max(c_1.\text{int}, c_2.\text{int}) \leq c.\text{int}$ (under non-negative importance). The shield weakened. When P1 fails (signed configuration), the shield may weaken or strengthen depending on the sign of the dispersed values. Caught by §11.11.

**Cycle risk.** Standard promotion can create an oscillation cycle with contraction. See §11.10.

---

### 11.5 Skip Promote (Elevate an Indivisible Node)

**Precondition.** $c$ is an entry (0 V-children) or a 3-child structural node. Parent $p$ is a 2-node. Grandparent $g$ is a 2-node.

**Effect.** Destroy $p$. Install $c$ and its sibling as direct children of $g$. $g$ becomes a 3-node. The parent is gone — scorched earth.

```
function skip_promote(c):
    p ← c.val_parent
    g ← p.val_parent
    assert |p.children| = 2
    assert |g.children| = 2
    s ← sibling(c, p)
    u ← sibling(p, g)

    g.children ← [c, s, u]
    c.val_parent ← g
    s.val_parent ← g

    destroy(p)
    propagate_evictable_flags(g)

    // V-I1: g.int was p.int + u.int = (c.int + s.int) + u.int.
    // Now g.int should be c.int + s.int + u.int — the same value.
    // Algebraic rearrangement; no propagation needed.
    // P5 required: the regrouping (c ⊕ s) ⊕ u → c ⊕ s ⊕ u is exact
    // only under associativity. Without P5, call:
    //   propagate_v_sums_from(g)
```

**Why this resolves the violation.** The violation was $c.\text{int} > u.\text{int}$ where $u$ was $c$'s uncle. After promotion, $c$ and $u$ are siblings under $g$. Siblings face no mutual constraint at their own level — the uncle relationship is dissolved.

**V-I1 preservation.** The grandparent's stored aggregate was $(c \oplus s) \oplus u$ (through the destroyed parent $p$). After promotion, the children are $\{c, s, u\}$ and the recomputed aggregate is $c \oplus s \oplus u$. Under P5, these are equal and V-I1 holds without propagation. Without P5, the stored aggregate is stale and `propagate_v_sums_from(g)` must be called, at $O(h_V)$ cost.

**Side effects.** Children of $u$ previously had uncle $p$ with $p.\text{int} = c.\text{int} + s.\text{int}$. After promotion, their max uncle $= \max(c.\text{int},\, s.\text{int})$. Under non-negative importance, $\max(c, s) \leq c + s = p.\text{int}$, so the shield weakened. When P1 fails (signed configuration), the shield may weaken or strengthen depending on the sign of $s.\text{int}$ — if $s.\text{int} < 0$, then $\max(c, s) = c > c + s = p.\text{int}$, and the shield actually strengthened. Caught by §11.11 regardless of configuration.

---

### 11.6 Legacy Promote (Elevate and Bequeath)

Legacy promotion is a normal uncle-constraint violation resolved by a different mechanism. A semi-internal G-node sits on the contour, receiving observations in its uncovered half. Its V-entry grows, eventually outweighing all its uncles. When `resolve()` (§11.9) reaches the skip-promote branch, it checks `is_semi_internal(c.gnode)` and `depth_V(c) ≤ D_evict`. If both hold, it dispatches to `legacy_promote`; otherwise it uses `skip_promote`. The depth check is an implementation optimisation: an heir created past $D_{\text{evict}}$ would be immediately eviction-eligible, triggering a futile allocate–evict round-trip. The V-Tree's rebalancing machinery is entirely generic; the semi-internal and depth checks are a late-bound dispatch.

The distinguishing feature is structural: **the promoted node leaves an heir in its vacated seat.** Skip promote destroys the parent — scorched earth, nothing remains. Legacy promote preserves the parent, depositing a new entry at importance $\nu$ in the exact position the promoted node vacated. The promoted node bequeaths its V-Tree seat as a legacy to the newly created G-node.

**Precondition.** $c$ is a V-entry. $c.\text{gnode}$ is semi-internal (exactly one G-child). Parent $p$ is a 2-node. Grandparent $g$ is a 2-node.

**Effect.** Lift $c$ to $g$. Create new G-node $N$ for the uncovered half. Insert $N.\text{entry}$ into $p$ at $c$'s former seat. $p$ survives as a 2-node. $g$ becomes a 3-node. $c.\text{entry}$ freezes.

```
function legacy_promote(c):
    assert c is entry
    assert is_semi_internal(c.gnode)
    p ← c.val_parent
    g ← p.val_parent
    assert |p.children| = 2
    assert |g.children| = 2
    s ← sibling(c, p)
    u ← sibling(p, g)
    gnode ← c.gnode

    // G-Tree: create the missing child
    exposed ← uncovered_range(gnode)
    new_child ← new G-Node(exposed, sum = 0, own = 0, importance = ν)
    if gnode.left = null:
        gnode.left ← new_child
    else:
        gnode.right ← new_child
    new_child.geo_parent ← gnode

    // V-Tree: create entry for new child
    ne ← new V-Entry(int = ν, gnode = new_child, is_exposed = true, is_evictable = true)
    new_child.entry ← ne

    // V-Tree: lift c to g, place ne in c's vacated seat
    replace c with ne in p.children
    ne.val_parent ← p

    g.children ← [c, p, u]
    c.val_parent ← g

    // Freeze: gnode now has 2 G-children, above the contour
    c.is_exposed ← false

    // Update importances. p lost c and gained ne:
    // p.int was c.int + s.int, now ne.int + s.int.
    p.int ← s.int + ne.int
    propagate_v_sums(p)
    propagate_evictable_flags(p)
    propagate_evictable_flags(g)
```

**Result (shown for the standard configuration where $\nu = 0$):**

```
Before:                          After:
    g (2-node)                       g (3-node)
    ├── p (2-node)                   ├── c (int = I, FROZEN)   ← rose
    │   ├── c (int = I)              ├── p (2-node, int = E + ν)
    │   └── s (int = E)              │   ├── s (int = E)       ← stayed
    └── u (int = U)                  │   └── ne (int = ν)      ← legacy: c's heir
                                     └── u (int = U)           ← stayed
```

Under the standard configuration where $\nu = 0$, $p.\text{int} = E$ and the diagram simplifies accordingly.

**Why this resolves the violation.** Identical to skip promote: $c$ and $u$ become siblings, dissolving the uncle relationship.

**Why the legacy creates correct structure.** $c.\text{entry}$ becomes uncle to $ne$ — the same relationship catalytic split (§10.2) produces. $ne$ starts at $\nu$, must earn promotion through competition. $c.\text{entry}$ is frozen because both G-children now intercept all observations. The mechanism is identical to catalytic split; only the trigger differs (competitive promotion vs. threshold + depth gate).

**V-I3 preservation.** When P2 holds ($\nu \preceq a$ for all $a$): $ne(\nu)$ is trivially satisfied against any uncle — $\nu$ is the minimum importance, so no uncle can be exceeded. When P2 fails, $ne$ may violate V-I3 — the caller must run `rebalance()`. $s$'s uncle set **improved** — gained $c$ as an uncle. Side effects at children of $u$ are the same class as skip promote, caught by §11.11.

**Invariant preservation:**

- **G-I1:** $\text{gnode.sum}$ unchanged. New child sum $= 0$. $\checkmark$
- **G-I4:** $c.\text{entry.int}$ references $\text{gnode.importance}$ (unchanged). $ne.\text{int}$ references $\text{new\_child.importance} = \nu$. (§4.2 reference semantics.) $\checkmark$
- **V-I1:** $p.\text{int} = s.\text{int} + ne.\text{int}$. $g.\text{int}$ recomputed via `propagate_v_sums`. $\checkmark$
- **V-I2:** $p$ is a 2-node. $g$ is a 3-node. $\checkmark$
- **V-I5:** $ne$ is an entry (V-leaf). $c$ remains an entry. $\checkmark$
- **V-I6:** $c.\text{is\_exposed}$ correctly set to false (now fully internal). $ne$ correctly set to true (exposed terminal). $\checkmark$
- **V-I6b:** $c.\text{is\_evictable}$ was already false (was semi-internal, still has dependents). $ne$ set to true (terminal, no dependents). $\checkmark$

---

### 11.7 Decision Table

| Condition on $c$                                                                      | Operation        | Rationale                                           |
| ------------------------------------------------------------------------------------- | ---------------- | --------------------------------------------------- |
| $c$ is structural with 2 children                                                     | Standard Promote | Disperses heavy aggregate; $p$ → 3-node             |
| $c$ is entry, backing semi-internal G-node, $\text{depth}_V(c) \leq D_{\text{evict}}$ | Legacy Promote   | Bequeaths seat to new child; $p$ survives as 2-node |
| $c$ is entry, backing semi-internal G-node, $\text{depth}_V(c) > D_{\text{evict}}$    | Skip Promote     | Heir would be immediately evictable; defer creation |
| $c$ is entry, backing terminal or internal G-node                                     | Skip Promote     | Cannot bequeath (nothing to create); rises bodily   |
| $c$ is structural with 3 children                                                     | Skip Promote     | Exploding would give $p$ four children; indivisible |

> _Note._ Escalation (§11.10) always dispatches to skip promote, even when the heaviest child `h` backs a semi-internal G-node. This breaks the oscillation cycle cleanly without G-Tree side effects. Legacy promotion is deferred — the semi-internal entry can earn it through normal competitive dynamics in a future rebalance.

---

### 11.8 The Rebalance Loop

The `rebalance()` function drains the violation work queue (§11.1), resolving every V-I3 violation that has been detected and enqueued.

```
function rebalance():
    while not violation_queue.is_empty():
        c ← violation_queue.pop()
        if not slot_occupied(c): continue   // destroyed by prior resolution
        if not is_violated(c): continue     // stale or already resolved
        resolve(c)
```

The `slot_occupied` guard handles V-node handles invalidated by destruction during prior resolutions in the same batch. The `is_violated` re-check (§11.2) filters duplicate and stale entries — see §11.1.2 for the duplicate tolerance and pop-order guarantees that make this correct.

---

### 11.9 The Resolve Dispatcher

By the time `resolve` is called, the reader has seen all four primitives (§§11.3–11.6). The dispatcher is a two-phase algorithm that normalises the neighbourhood, then selects the appropriate promotion.

```
function resolve(c):
    p ← c.val_parent
    g ← p.val_parent

    // ── Phase 1: Ensure p is a 2-node ──
    if |p.children| = 3:
        merged_p ← contract(p, isolate = heaviest_child(p))
        push_side_effect_violations(p)              // §11.11.1
        push_side_effect_violations(merged_p)
        push_contraction_child_violations(p, skip = c)
        if not is_violated(c): return               // contraction resolved it

    // ── Phase 2: Promote c ──

    // Re-read p (unchanged by Phase 1) and g (unchanged by Phase 1).
    // Phase 1 only restructured p's children; p and g are the same nodes.

    if c is structural and |c.children| = 2:
        standard_promote(c)
        push_side_effect_violations(p)              // §11.11.1
        push_promoted_violations(p)
        escalate_after_promote(p)                   // §11.10
    else:
        // Skip or legacy promote: requires g to be a 2-node.
        merged_g ← null
        if |g.children| = 3:
            merged_g ← contract(g, isolate = heaviest_child(g))
            push_side_effect_violations(g)
            push_side_effect_violations(merged_g)
            push_promoted_violations(g)
            if not is_violated(c): return           // contraction resolved it

        if c is entry and is_semi_internal(c.gnode) and depth_V(c) ≤ D_evict:
            legacy_promote(c)                       // §11.6
        else:
            skip_promote(c)                         // §11.5

        // Post-promotion side-effect checks.
        //
        // The push target is g — the original grandparent, which is
        // now the node under which the promoted c sits (as a 3-node
        // child). push_side_effect_violations(g) checks grandchildren
        // of g, which includes children of the node where the
        // promotion landed.
        push_side_effect_violations(g)
        push_promoted_violations(g)

        // When g-contraction introduced an intermediate node
        // (merged_g), the promotion operated at the merged_g level.
        // Children of merged_g's non-p child face a weakened uncle
        // context: their uncle changed from p (with int = c + s) to
        // the promoted node's dispersed components. These children
        // sit at depth 3 below g — beyond the reach of
        // push_side_effect_violations(g), which only covers depth 2.
        // An additional push on merged_g catches them.
        if merged_g ≠ null:
            push_side_effect_violations(merged_g)
```

**Phase 1** normalises the parent to a 2-node — a precondition for all promotion variants. The contraction may resolve the original violation by aggregating siblings into a heavier uncle; the dispatcher re-checks before proceeding to Phase 2.

**Phase 2** selects the promotion type via the decision table (§11.7). Standard promote disperses a 2-child structural aggregate. Skip and legacy promote both dissolve the uncle relationship by lifting $c$ to the grandparent level; legacy promote additionally bequeaths the vacated seat to a new entry.

**The g-contraction coverage issue.** When `g` is a 3-node, Phase 2 contracts it before promoting. Contraction creates an intermediate structural node (`merged_g`) between `g` and `p`. The subsequent promotion (skip or legacy) operates at the `merged_g` level: skip promote reads `p.val_parent` (which may now be `merged_g`), and the promotion restructures `merged_g` into a 3-node. After promotion, the children of `merged_g`'s non-`p` child — call it `u_merged` — face a changed uncle context: before promotion, children of `u_merged` had uncle `p` ($p.\text{int} = c.\text{int} + s.\text{int}$); after promotion, their uncle set contains `c` and `s` individually, with $\max(c, s) \leq c + s$ (under non-negative importance). The shield weakened.

These nodes sit at depth 3 below `g`. The standard push — `push_side_effect_violations(g)` — checks depth 2 only (grandchildren of `g`). The additional `push_side_effect_violations(merged_g)` catches depth 3 by checking grandchildren of `merged_g`, which are the children of `u_merged` and siblings.

Without this additional push, violations at this level persist undetected until the next observation triggers a fresh ancestor walk in the affected neighbourhood — violating the stated guarantee that `rebalance()` restores V-I3 everywhere.

---

### 11.10 Escalation After Standard Promote

Standard promotion makes $p$ a 3-node $\{c_1, c_2, s\}$ by exploding $c$. This disperses heavy importance — but can create a cycle.

**The direct cycle:**

```
1. standard_promote(c) → p becomes 3-node {c₁, c₂, s}
2. c₁ (heaviest) is violated against its uncle at grandparent g
3. resolve(c₁): Phase 1 contracts p → 2-node {c₁, merged}
4. c₁ still violated → standard_promote(c₁) → p becomes 3-node again
5. GOTO 2  ∞
```

**The indirect cycle:** A _child_ of $c_1$ is violated against its uncle (now $c_2$ or $s$, siblings under $p$). Resolving that child's violation contracts $p$, then standard_promote recreates the 3-node — the same oscillation one level deeper.

**The fix.** After standard_promote makes $p$ a 3-node, eagerly detect the cycle condition and take a different path — skip_promote — that advances the heavy node past the problematic parent structure entirely.

```
any_child_violated(h) → bool:
    If h is an entry: return false.
    Return true if any direct child c of h satisfies is_violated(c).
```

```
function escalate_after_promote(p):
    // p is a 3-node after standard_promote
    h ← heaviest_child(p)
    direct ← is_violated(h)
    indirect ← not direct and any_child_violated(h)
    if not direct and not indirect: return

    g ← p.val_parent
    if g = null: return

    // Undo the 3-node: contract p, isolating h
    merged_p ← contract(p, isolate = h)
    push_side_effect_violations(p)
    push_side_effect_violations(merged_p)
    push_contraction_child_violations(p, skip = h)

    // Re-check: contraction alone may resolve the trigger
    if direct:  needs_skip ← is_violated(h)
    else:       needs_skip ← any_child_violated(h)
    if not needs_skip: return

    // Ensure g is a 2-node
    merged_g ← null
    if |g.children| = 3:
        merged_g ← contract(g, isolate = heaviest_child(g))
        push_side_effect_violations(g)
        push_side_effect_violations(merged_g)
        push_promoted_violations(g)
        // Re-check after g's contraction
        if direct:  needs_skip ← is_violated(h)
        else:       needs_skip ← any_child_violated(h)
        if not needs_skip: return

    // Skip h past p to g, breaking the cycle
    skip_promote(h)
    push_side_effect_violations(g)
    push_promoted_violations(g)

    // Coverage for g-contraction intermediate level (§11.9 rationale)
    if merged_g ≠ null:
        push_side_effect_violations(merged_g)
```

The key insight: standard promotion followed by contraction is a no-op (the 3-node is created then undone). Escalation detects this before it becomes infinite and takes skip_promote instead, advancing $h$ one level higher.

> _Note._ Escalation always dispatches to skip promote, even when `h` backs a semi-internal G-node. Legacy promotion is deliberately suppressed in the escalation path: the cycle-breaking logic must be simple and side-effect-free with respect to the G-Tree. The semi-internal entry can earn legacy promotion through normal competitive dynamics in a future observation's rebalance.

---

### 11.11 Side-Effect Violations

Restructuring changes uncle relationships at multiple levels. The push functions enumerate all potentially affected nodes and add violators to the work queue. They arise in three contexts: rebalancing operations, observation propagation, and leaf removal.

#### 11.11.1 Rebalancing-Triggered Side Effects

Three push functions cover violations created by contraction and promotion.

**Grandchild-level** (`push_side_effect_violations`). After contraction or promotion changes a node's children, grandchildren face new uncle contexts. Called on the restructured node.

```
function push_side_effect_violations(node):
    for each child c of node:
        if c is structural:
            for each grandchild gc of c:
                if is_violated(gc): push gc
```

After contraction at $p$ creating merged node $s$: call on both $p$ and $s$. Grandchildren of $s$ — children of the merged pair — had their uncle reduced from isolate's importance to the other merged child's. After skip/legacy promote at $g$: call on $g$, and on the intermediate node if g-contraction occurred (§11.9).

**Child-level** (`push_promoted_violations`). After any operation that changes a node's children, the children themselves move into a new uncle context at the grandparent level. Distinct from the grandchild check — this looks one level down, not two.

```
function push_promoted_violations(node):
    for each child c of node:
        if is_violated(c): push c
```

**Contraction excluding target** (`push_contraction_child_violations`). During `resolve(c)`, Phase 1 contracts parent $p$. All of $p$'s children enter a new uncle context, but $c$ itself is about to be handled by Phase 2 — re-enqueuing it would cause duplicate processing. This variant skips $c`.

```
function push_contraction_child_violations(p, skip):
    for each child c of p:
        if c ≠ skip and is_violated(c): push c
```

#### 11.11.2 Observation-Triggered Side Effects

`propagate_v_sums` (Step 3b of §8.3.3) increases the importance of every structural ancestor of the observed entry. Each increased ancestor might now exceed its own uncle at the grandparent level. Step 3c of §8.3.3 checks for this by walking from the entry to the V-root:

```
check_id ← entry
while check_id ≠ null:
    if is_violated(check_id): push check_id
    check_id ← check_id.val_parent
```

This catches violations that an entry-only check would miss: a structural node at depth 4 might exceed its uncle at depth 3, even though the entry at depth 8 is not itself violated.

Cost: $O(h_V)$, matching `propagate_v_sums` itself.

#### 11.11.3 Leaf-Removal-Triggered Side Effects

Removing a V-entry (§9.2) creates violations through four mechanisms. All arise because `propagate_v_sums_from` decreases ancestor importances, weakening uncle shields.

**Ancestor walk** (`push_leaf_removal_violations`). At each ancestor level, the decreased node may be a weaker uncle to its siblings' children. Walk from the structural change point to the root:

```
function push_leaf_removal_violations(start):
    a ← start
    while a ≠ null:
        p ← a.val_parent
        if p = null: break
        for each sibling s of a under p:
            for each child c of s:
                if is_violated(c): push c
        a ← p
```

At each level, only siblings' children are checked — not the decreased ancestor's own children — because only siblings of the decreased ancestor see a weakened uncle. Per-level cost: at most $2 \times 3 = 6$ checks. Total: $O(h_V)$.

**Collapse: promoted node's children** (`push_collapse_violations`). When a 2-node collapses, the surviving sibling (`sole`) is re-parented to the grandparent. Children of `sole` now see entirely different uncles — the grandparent's other children, rather than the removed entry.

```
function push_collapse_violations(sole):
    for each child c of sole:
        if is_violated(c): push c
```

**3→2 transition: remaining siblings' children** (`push_remaining_sibling_violations`). When an entry is removed from a 3-node parent (making it a 2-node), the remaining siblings' children lose the removed entry as a potential uncle. If the removed entry was the heaviest uncle, `max_uncle` decreases.

```
function push_remaining_sibling_violations(parent, removed_id):
    for each child s of parent where s ≠ removed_id:
        for each child c of s:
            if is_violated(c): push c
```

> _Note._ At the §12.5 Step 8 call site, `vtree_remove_leaf` (Step 7) has already removed the evicted entry from `v_parent.children`, making the `removed_id` exclusion vacuous. That call site uses an inline loop instead of this function — a streamlined equivalent with the stronger precondition that the removed entry is already absent from the children list. The general-purpose definition with the explicit exclusion parameter is retained for any future call sites where the precondition does not hold.

**Collapse: cousins' children** (`push_cousin_violations`). When a 2-node collapses, the grandparent's other children ("cousins") see `sole` replace the collapsed parent as uncle. Since $\text{sole.int} < \text{parent.int}$ (the removed entry's importance is gone), the shield weakened.

```
function push_cousin_violations(sole, grandparent):
    for each child cousin of grandparent where cousin ≠ sole:
        for each child c of cousin:
            if is_violated(c): push c
```

#### 11.11.4 Cost Summary

| Context      | Functions                                                                                                                 | Per-event cost |
| ------------ | ------------------------------------------------------------------------------------------------------------------------- | -------------- |
| Rebalancing  | `push_side_effect_violations`, `push_promoted_violations`, `push_contraction_child_violations`                            | $O(1)$         |
| Observation  | Ancestor walk (§11.11.2)                                                                                                  | $O(h_V)$       |
| Leaf removal | `push_leaf_removal_violations`, `push_collapse_violations`, `push_remaining_sibling_violations`, `push_cousin_violations` | $O(h_V)$       |

The $O(h_V)$ costs match `propagate_v_sums` — the violation checks piggyback on propagation already being performed.

The $O(1)$ bound for rebalancing-triggered side effects counts as follows. After contraction at $p$ creating merged node $s$: `push_side_effect_violations` on $p$ checks at most $2 \times 3 = 6$ grandchildren; on $s$ checks at most $2 \times 3 = 6$ grandchildren. `push_contraction_child_violations(p, skip)` checks at most 1 child. `push_promoted_violations` checks at most 3 children. Each call to `is_violated` is $O(1)$. Total: bounded constant per restructuring event.

---

### 11.12 Violation Sources

The **root cause** of all violations is observation: an importance increase propagating upward through the V-Tree. But the _sites_ where violations manifest are varied, because `propagate_v_sums` increases every structural ancestor, and structural changes redistribute uncle relationships.

| #   | Source                                                        | Mechanism                                                                  | Detection                                                                                                                     |
| --- | ------------------------------------------------------------- | -------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| 1   | The observed entry                                            | Entry importance grew past max uncle                                       | Step 3 of §8.2 (ancestor walk)                                                                                                |
| 2   | Structural ancestors of the entry                             | `propagate_v_sums` increased ancestor past its uncle                       | Step 3 of §8.2 (ancestor walk)                                                                                                |
| 3   | Contraction side effects (merged-node grandchildren)          | Merged-node grandchildren face weaker uncle                                | `push_side_effect_violations` on merged node (§11.11.1)                                                                       |
| 4   | Promotion side effects (restructured node's grandchildren)    | Children of restructured node face new uncle context                       | `push_side_effect_violations` and `push_promoted_violations` (§11.11.1)                                                       |
| 5   | Split preprocessing (§10.1)                                   | Contraction during split creates effects of sources 3–4                    | Same as sources 3–4                                                                                                           |
| 5b  | Post-split violations (§10.1, catalytic and bootstrap splits) | New entries or structural node $s$ may exceed uncles when P2 or P3 fails   | `push_side_effect_violations` and `push_promoted_violations` at $p$; conditional ancestor walk when $\nu \oplus \nu \neq \nu$ |
| 6   | V-Tree leaf removal (ancestor walk)                           | Decreased ancestors are weaker uncles at every level                       | `push_leaf_removal_violations` (§11.11.3)                                                                                     |
| 7   | 2-node collapse: sole's children                              | Children of surviving sibling face entirely new uncles                     | `push_collapse_violations` (§11.11.3)                                                                                         |
| 8   | 3→2 transition: remaining siblings' children                  | Remaining children lose removed entry as uncle                             | `push_remaining_sibling_violations` (§11.11.3)                                                                                |
| 9   | 2-node collapse: cousins' children                            | Cousins' children see weaker replacement uncle                             | `push_cousin_violations` (§11.11.3)                                                                                           |
| 10  | g-contraction + promotion: intermediate-level grandchildren   | Promotion disperses uncle aggregate below the intermediate structural node | `push_side_effect_violations(merged_g)` in §11.9 and §11.10                                                                   |

Catalytic splits (§10.4) create **no violations** when P2 + P3 hold ($\nu \preceq a$ for all $a$ and $\nu \oplus \nu = \nu$) — adding a minimum-importance newcomer cannot weaken any node's shield. Under non-standard configurations, the post-split checks in §10.1 catch and enqueue any violations for Step 6's `rebalance()`.

---

### 11.13 Termination

**Claim.** The `rebalance()` loop terminates after finitely many iterations.

#### 11.13.1 Termination When P2 Holds ($\nu \preceq a$ for all $a$)

Define the weighted depth potential $\Phi = \sum_{\ell} \ell.\text{int} \cdot d(\ell)$ over all entries $\ell$, where $d(\ell)$ is $\ell$'s V-Tree depth. Importances are fixed during rebalancing (no observations arrive). Each complete resolution cycle moves heavy importance closer to the root:

- **Skip Promote:** Destroys one structural node, moving $c$ from depth $d$ to depth $d - 1$. $\Delta\Phi = -c.\text{int} < 0$.
- **Legacy Promote:** Creates a new entry at importance $\nu$ (contributing 0 to $\Phi$ since $\nu$ is the minimum [P2]) and moves $c$ from depth $d$ to depth $d - 1$. $\Delta\Phi = -c.\text{int} < 0$.
- **Standard Promote:** Destroys one structural node, dispersing the heavy aggregate into lighter pieces at the same or shallower depth. $\Delta\Phi \leq 0$. Escalation (§11.10) prevents cycles by detecting when dispersion fails to resolve the violation and switching to skip promote.
- **Contraction:** Creates one structural node, potentially increasing $\Phi$. Always paired with a subsequent promotion (or the contraction alone resolved the violation, with no $\Phi$ increase from the resolution itself). The combined contraction-plus-promotion cycle produces a net non-positive $\Delta\Phi$.

Since $\Phi \geq 0$ [P1: bounded below], importances are fixed, the entry set is finite, and each resolution cycle produces $\Delta\Phi \leq 0$ with $\Delta\Phi < 0$ when promotion occurs, the loop terminates. $\square$

#### 11.13.2 Termination When P2 Fails ($\nu$ is not the minimum)

When P2 fails (e.g., under an elevated configuration where $\nu > 0$), legacy promote creates entries with positive importance at depth $d$, contributing $\nu \cdot d > 0$ to $\Phi$. The net change $\Delta\Phi = -c.\text{int} + \nu \cdot d$ may be positive when $c.\text{int} < \nu \cdot d$. The $\Phi$-decrease argument does not directly apply.

**Alternative argument.** Termination is established by bounding the total number of operations:

1. **Entry creation is bounded.** Only legacy promote creates entries during rebalancing. By the Semi-Internal Consumption Lemma (§7.5.1), at most $S$ legacy promotions occur per rebalance (where $S$ is the semi-internal G-node count), because each consumes one semi-internal and no rebalancing primitive creates new semi-internals. The entry set grows by at most $S$.

2. **Promotions per entry are bounded.** Each promotion moves its target one level shallower (from depth $d$ to $d - 1$). Depth is bounded below by 0. Repeated promotions of the same node require it to be re-violated at successively shallower depths. Without new observations, a node at depth $d'$ after promotion is violated only if its importance exceeds all uncles at the new position — a condition involving fixed values. Each such violation is resolved by another promotion, further reducing depth. After at most $h_V$ promotions, the node reaches depth 0 or 1 where no grandparent exists and V-I3 is trivially satisfied.

3. **Total operations.** At most $n + S$ entries (original plus legacy-created), each promoted at most $h_V$ times. Each promotion involves at most 2 contractions ($O(1)$ each). Total operations: $O((n + S) \cdot h_V)$, which is finite.

$\square$

#### 11.13.3 Termination When P1 Fails

When P1 fails (no lower bound, e.g., the signed configuration), the Fibonacci bound does not apply. Termination relies on structural arguments only: the tree is finite, each promotion moves an entry strictly shallower, and no promotion creates new violations at shallower depths without also reducing the violating entry's depth. The total operation count is $O((n + S) \cdot h_V)$ by the same argument as §11.13.2.

> _Remark (practical convergence)._ Under all configurations, the typical rebalance resolves $O(1)$ violations per observation. The $O(n \cdot h_V)$ worst case requires every entry to cascade through every level — a configuration that the competitive mechanism itself tends to dissolve. Under proportional traffic ($w_i$-weighted observations), the expected rebalancing cost per observation is $O(H / \log_2\phi)$ (§18.8), which is $O(1)$ for concentrated distributions.

**Corollary (Q-I1).** Since `rebalance()` terminates after finitely many iterations and each iteration pops one element, the violation queue is empty when `rebalance()` returns — establishing Q-I1 (§11.1.3) at every drain point.

---

## Chapter 12. Contour Simplification

Contour refinement (§10) adds resolution. This chapter specifies the dual operation: how the contour loses resolution, and why this loss is controlled, conservative, and convergent. The mechanism is **eviction** — the removal of unprotected contour cells that have lost the V-Tree's competitive tournament. The effect is **simplification** — the contour coarsens from its tips inward, absorbing fine-scale energy into coarser cells, preserving total magnitude while surrendering spatial detail.

The chapter is organised in four parts. §§12.1–12.2 establish **measures and targeting**: a node-count measure with a plateau bound, and a design-case analysis of which nodes eviction reaches first. §§12.3–12.6 specify **the machinery**: eligibility criteria, absorption semantics, the eviction operation itself, and the three-phase scan that orchestrates it. §§12.7–12.8 describe **the dynamics**: bottom-up tidal contraction and the two independent simplification effects (direct node removal and implicit depth-variation narrowing). §§12.9–12.10 establish **correctness and cost**: structural invariant preservation, convergence to a fixed point with a finite-time bound, and the overhead analysis for semi-internal chains.

### 12.1 Node-Count Measure and Plateau Bound

The natural measure of the G-Tree's size is its node count:

$$\boxed{A = |G|}$$

where $|G|$ is the number of live G-nodes.

$$\textbf{Lemma 12.1 (Unit Decrease).}\quad \text{Every eviction decreases } |G| \text{ by exactly } 1.$$

**Proof.** Eviction destroys one G-node (§12.5). The root is permanently
exempt, so every eviction removes exactly one non-root node. $\square$

#### 12.1.1 Plateau–Node-Count Bound

$$\textbf{Proposition.}\quad P \leq |G|$$

**Proof.** Every plateau contains at least one contour cell (non-empty by
definition). Each contour cell corresponds to a unique G-node — the
receiver at that position. Distinct plateaus have disjoint contour cells
(P-I1, P-I3). Therefore $P \leq L \leq |G|$ where $L$ is the contour cell
count. (This result also appears in §5.6.6 via the contour-cell argument;
the proof here is restated for self-containment.) $\square$

The hard budget $G_{\max}$ from §7.5 gives an immediate corollary:

$$P \leq |G| \leq G_{\max}$$

No minimum-plateau-width dependence. No loose ratio. The bound is tight
in the degenerate case where every G-node is a terminal at a different
depth (maximally fragmented contour, $P = L = |G|$).

**Dynamic direction.** Under pure contraction (sustained decay, no new
splits), $|G|$ decreases by 1 per eviction. The ceiling on $P$ tightens
monotonically and unconditionally. Fewer nodes means fewer possible
plateaus — there is simply less material with which to be complex.

#### 12.1.2 Structural Cost of Deep Nodes

The following is a design observation, not a structural result.

Although every eviction costs exactly 1 node, not all nodes are equal in
their contribution to the G-Tree's structural footprint. A single terminal
at G-depth $d$ requires $d$ ancestor nodes on its root-to-leaf path. If
that terminal is the sole occupant of its branch at depths $> k$, its
removal lets the entire branch retract to depth $k$. The **structural
cost** of deep nodes lies not in their individual count (always 1) but in
the chain of ancestors they force the tree to maintain.

---

### 12.2 Why Eviction Targets Cold Contour Tips

Eviction targets cells that are **deep in the V-Tree** (past
$D_{\text{evict}}$) and **unprotected** (zero G-children). The V-Tree's
competitive mechanism tends to push low-importance entries toward greater
V-depth (the Fibonacci depth bound §18.1 upper-bounds depth by
$\log_\phi(1/w_i) + c$, but does not enforce strict weight-to-depth
monotonicity). So eviction selectively removes the coldest, most
insignificant cells.

> **Scope.** The argument in this section is a _design-case analysis_, not
> a structural invariant. The correlation between V-Tree depth (competitive
> insignificance) and G-Tree depth (spatial resolution) is strong under
> workloads with spatial locality and temporal decay, but is not enforced
> by any invariant. Under raw accumulation with no user-applied decay,
> frozen V-entries at abandoned deep regions retain historical intensities
> indefinitely — the correlation weakens for regions that were once
> strongly observed. Targeted annihilation ($\text{att} = 0$, §14.3) can clear such frozen entries immediately; the correlation is restored after the annihilated region re-accumulates. The formal results of this chapter (Unit Decrease,
> Plateau–Node-Count Bound, correctness of tip-only eviction) do not
> depend on this correlation. The description below characterises the
> _typical dynamic_ under the recommended operating regime (observation +
> decay), not a universal guarantee. Under raw accumulation, the
> correlation still holds for regions that were never strongly observed
> (their V-entries naturally sit deep), but weakens for regions with large
> frozen historical values that retain moderate V-positions long after
> traffic has moved elsewhere.

The correlation between V-Tree depth (competitive insignificance) and
G-Tree depth (spatial resolution) is **strong but not structural**:

| V-Tree depth | G-Tree depth | Typical case                            | What eviction removes                            |
| ------------ | ------------ | --------------------------------------- | ------------------------------------------------ |
| V-Deep       | G-Deep       | Cold region once refined, traffic moved | Deep contour tip — maximum structural retraction |
| V-Deep       | G-Shallow    | Large cold region, never refined much   | Shallow contour cell — minimal structural impact |
| V-Shallow    | G-Deep       | Hot region at fine resolution           | Not evicted — V-depth too shallow                |
| V-Shallow    | G-Shallow    | Active coarse region                    | Not evicted                                      |

The depth gates operate on V-Tree depth, not G-Tree depth. A cell at
G-depth 15 can still earn a split if its V-entry holds a shallow
tournament position — it has proven sustained competitive significance
despite spatial depth. Conversely, a G-Tree-shallow cell can sit deep in
the V-Tree if it is cold.

> _Note (inverse ordering along semi-internal chains)._ Along a
> semi-internal chain (§12.10), G-depth and V-depth are typically
> _inversely_ ordered. The most recently frozen entry — at the G-Tree
> tip — accumulated the most observations before freezing and therefore
> sits _shallowest_ in the V-Tree. Entries frozen earliest (at the
> chain's G-Tree base) have the smallest frozen intensities and sit
> _deepest_ in the V-Tree. Consequently, eviction (which targets
> V-deep entries) reaches the base entries first — but they are
> eviction-immune (they have dependents). The chain can only erode
> from the G-Tree tip inward, which is exactly the structural
> constraint: eviction requires zero G-children.

The typical case — cold regions at high G-depth being the primary eviction
targets — arises because reaching high G-depth requires many rounds of
refinement, each requiring competitive promotion, and regions that have
been abandoned after deep refinement are precisely those with both high
G-depth and low competitive standing.

Meanwhile, creation adds nodes at moderate G-depths (reaching extreme depth
requires a long chain of prior promotions), while eviction removes nodes
at the structural extremes (per the correlation described above). The net
effect is structural compression: the tree contracts from its deepest,
coldest branches while new structure builds in the interior.

> _Cross-reference._ The formal relationship between eviction and
> simplification — the three-link chain connecting Unit Decrease, the
> Plateau–Node-Count Bound, and the eviction-targeting dynamic — is
> established in §12.8.

---

### 12.3 Eviction Eligibility

A V-entry is eligible for eviction when three conditions hold simultaneously
(the threshold $D_{\text{evict}}$ is governed by D-I2, §7.2):

$$\text{evict}(v) \iff \text{depth}_V(v) > D_{\text{evict}} \;\wedge\; \neg\,\text{has\_dependents}(v.\text{gnode}) \;\wedge\; v.\text{gnode} \neq G_{\text{root}}$$

**The depth condition** selects the globally insignificant. Entries deep in
the V-Tree are the lightest, coldest competitors.

**The dependents condition** ensures only unprotected contour tips are
removed. A node with G-children is structurally load-bearing — it supports
finer-scale structure that depends on it. Only nodes with zero children sit
at the tips.

**The root exemption** guarantees V-I0 (the V-Tree always contains at
least one entry after initialisation) and ensures the sampling distribution
(§6.5) is always defined when total importance is positive. The root is
permanently exempt from eviction regardless of V-depth or dependents.

The first two conditions define _structural eligibility_ — the node is
globally insignificant and physically removable. The third is a _policy
override_ — the root is exempt even when structurally eligible.

Both structural conditions are necessary. Depth alone would remove
structurally load-bearing nodes. The dependents check alone would remove
significant contour cells. Together they select exactly the globally
insignificant, structurally exposed tips — subject to the root exemption.

**Semi-internal entries are eviction-immune** regardless of V-depth. They
have at least one G-child, so the dependents check (condition 2) fails.
They resolve naturally:

- **Hot path.** The uncovered half receives traffic, the entry grows,
  legacy promotion (§11.6) fires — the contour regrows at that point.
- **Cold path.** The surviving child's subtree erodes from its own tips
  inward — its terminal descendants are removed first. The parent
  eventually loses all dependents, joins the contour tips, and becomes
  eligible itself.

---

### 12.4 Absorption Semantics

When a contour tip is removed, its energy flows inward. The parent absorbs
everything.

```
Before eviction:
    Parent [0,4):  own=10, sum=40
    ├── Left  [0,2): own=18, sum=18 (terminal — contour tip)
    └── Right [2,4): own=12, sum=12 (terminal — contour tip)

After evicting Left:
    Parent [0,4):  own=28, sum=40 (semi-internal — partially on the contour)
    └── Right [2,4): own=12, sum=12 (terminal, still present)

After evicting Right:
    Parent [0,4):  own=40, sum=40 (terminal — fully on the contour, one level shallower)
```

**Status key:** exact = backed by a live terminal. est. = pro-rated
from the covering cell under a uniform-within-cell assumption.

| Query                   | Before | After Left Evicted                  | After Both Evicted                  |
| ----------------------- | ------ | ----------------------------------- | ----------------------------------- |
| Total energy in $[0,4)$ | 40     | 40                                  | 40                                  |
| Energy in $[0,2)$       | 18     | $28 \times \frac{1}{2} = 14$ (est.) | $40 \times \frac{2}{4} = 20$ (est.) |
| Energy in $[2,4)$       | 12     | 12 (**exact** — still live)         | $40 \times \frac{2}{4} = 20$ (est.) |

> _Note._ Sub-region values in the "Before" column show each terminal's
> direct measurement ($g.\text{own} = g.\text{sum}$), not the output of
> `range_sum`. These sum to 30, not 40 — the missing 10 is the parent's
> own pre-split accumulation ($g.\text{own} = 10$), which has no
> ground-truth spatial attribution to either half. `range_sum` (§5.5.2)
> would pro-rate this ancestor energy uniformly, giving
> $\text{range\_sum}([0,2)) = 18 + 5 = 23$ and
> $\text{range\_sum}([2,4)) = 12 + 5 = 17$, which do sum to 40. The
> table highlights the transition from known (terminal-backed) to
> estimated (pro-rated) attribution — the parent's undifferentiated
> energy is precisely the spatial information that eviction discards.

The intermediate state shows the asymmetry of partial eviction:
one sub-region retains full precision (the surviving child) while
the vacated sub-region is estimated from the semi-internal parent.
Resolution degrades one child at a time.

**The total energy is conserved.** Eviction moves the contour inward but
does not remove energy from the system. The parent inherits the full sum.
Magnitude is the integral. Spatial detail is the derivative. Absorption
preserves the integral.

**Both trees conserve total importance.** The G-Tree root's sum is unchanged
(§12.5 Step 1: absorption exactly compensates detachment — zero G-Tree
propagation cost). The V-Tree's total importance is also unchanged: the
the absorption operation (§8.7) ensures the parent gains exactly
what the removed entry carried.

> _Note on the plateau ordered map._ The plateau projection (§5.6.7)
> reports structural energy, which double-counts thatched regions by
> design (§5.6.3); its total generally exceeds the G-root sum. The
> conservation claim above applies to the G-Tree and V-Tree ground-truth
> totals, not the plateau projection.

Because $p.\text{sum}$ is algebraically unchanged (Design Note, §12.5 Step 1), eviction requires no G-Tree ancestor propagation — the G-side cost is $O(1)$, unlike `observe()` Step 4 which costs $O(d_{\text{geo}})$.

**Re-measurement is self-funding.** If the region becomes interesting again,
the parent's high importance attracts proportional sampling weight. Fresh
observations flow in. The contour can regrow at that point — but against a
hardened benchmark (§13.5).

---

### 12.5 The Eviction Operation

**Precondition.** $v$ is a V-entry with $\text{depth}_V(v) > D_{\text{evict}}$ and $\neg\,\text{has\_dependents}(v.\text{gnode})$.

```
function evict(v):
    g ← v.gnode
    if g = G_root: return   // root is permanently exempt
    p ← g.geo_parent        // non-null: root exemption guarantees g has a parent

    // Step 1: Parent absorbs value; detach g
    p.own ← p.own + g.sum
    if p.left = g: p.left ← null
    if p.right = g: p.right ← null

    // p.sum is algebraically unchanged: g.sum moved from child
    // contribution to p.own. See Design Note (Step 1 Invariant) below.

    // Step 2: Ghost fast path (normative).
    // When P4 holds (identity element): a ghost has importance ν, and
    // p.importance ⊕ ν = p.importance by definition. Absorption is a
    // no-op; Steps 3–4 would propagate a zero delta and find no
    // violations, costing O(h_V) for no effect. Skip them.
    // Steps 5–10 must still execute — the parent's exposure/evictable
    // state genuinely changes, the V-entry must be removed, and the
    // G-node deallocated.
    // Without this fast path, ghost evictions inflate total contraction
    // work from O(|G|₀ · h_V) to O(|G|₀ · h_V²).
    //
    // Property requirements for this fast path:
    //   P4 (identity): a ⊕ ν = a — absorption is a no-op.
    //   P2 without P4: ν ⪯ a for all a, but a ⊕ ν may differ from a — not safe.
    //   Neither P2 nor P4: g.importance may not be the minimum at all — not safe.
    // Under ordinary addition on [0,∞) (where P2 forces ν = 0, hence
    // P4 holds), the P2-without-P4 case does not arise.
    if P4 holds and g.importance = ν:
        goto Step 5

    // Step 3: Update parent's importance
    // Assert: p.entry ≠ null. The parent had dependents during the
    // eviction scan's snapshot phase (see §12.6 Phase 1), so it was
    // not in the candidate list. Phase 2 processes only the snapshot —
    // no new eviction candidates are created. Therefore p was never
    // evicted and its entry persists.
    assert p.entry ≠ null
    // Absorb the child's importance into the parent's importance
    // accumulator using the universal absorption rule (§8.7.1):
    //   p.importance ← p.importance ⊕ g.importance
    // See §8.7.2 for the aliasing optimisation under identity
    // projection (where p.importance = p.own after ledger absorption).
    // G-I4 is maintained (the V-entry references p.importance, which
    // is now correct). The child's V-entry is still live at this step
    // (removed at Step 7).
    p.importance ← p.importance ⊕ g.importance    // maintain G-I4 (§8.7)
    propagate_v_sums(p.entry)

    // Step 4: Ancestor-walk violation check (sources 1–2 of §11.12,
    // with p.entry as the pseudo-observed entry — absorption in Step 3
    // is structurally analogous to an observation at p.entry)
    //
    // Note: p.entry flags (is_exposed, is_evictable) are stale here;
    // Step 5 updates them. This walk depends only on importance via
    // is_violated(), not on flags, so staleness is harmless.
    check_id ← p.entry
    while check_id ≠ null:
        if is_violated(check_id): push check_id
        check_id ← check_id.val_parent

    // Step 5: Update parent's exposure and evictable flags
    // Precondition: Step 1 already nulled g's slot in p.
    // Getting this wrong silently produces an incorrect is_evictable flag.
    assert (p.left ≠ g) and (p.right ≠ g)
    p.entry.is_exposed ← true                    // removing a child always exposes range
    p.entry.is_evictable ← not has_dependents(p)  // maintain V-I6b
    propagate_evictable_flags(p.entry.val_parent)

    // Design note (Step 4/5 ordering): Step 4's ancestor-walk violation
    // check runs *before* Step 5 updates is_exposed and is_evictable.
    // This is deliberate. is_violated() (§11.2) depends only on
    // intensities — it reads .int fields on the node, its parent, and
    // its grandparent's children. It never reads is_exposed or
    // is_evictable. The flags are therefore stale but harmless during
    // Step 4. Performing Step 4 before Step 5 simplifies the ordering
    // (the intensity mutation in Step 3 and its violation consequences
    // in Step 4 are adjacent) without affecting correctness. Note that
    // violations pushed in Step 4 are not resolved until the trailing
    // rebalance (Phase 3 of check_evictions), which runs after all
    // individual evictions — including their Step 5 flag updates —
    // have completed. The flags are always current before resolution.

    // Step 6: Pre-capture V-Tree context for violation tracking.
    // vtree_remove_leaf may collapse a 2-node structural parent,
    // destroying it. Capture the correct change point and the
    // surviving sibling before removal.
    //
    // Edge case: if v_parent is the V-root (2-node), change_point = null.
    // After collapse, collapse_sibling becomes the new V-root.
    // Step 8's violation checks are safe: is_violated (§11.2) returns
    // false when the grandparent is null, so no spurious violations
    // are pushed for the new V-root's children.
    v_parent ← v.val_parent
    child_count ← |v_parent.children|
    if child_count = 2:
        collapse_sibling ← sibling of v under v_parent
        change_point ← v_parent.val_parent   // grandparent — v_parent will be destroyed
    else:
        collapse_sibling ← null
        change_point ← v_parent               // 3→2: parent survives

    // Step 7: Remove V-entry.
    // Transient V-I1 inflation: between Step 3 (parent's importance
    // increased) and this removal, the V-Tree's total importance is
    // transiently inflated by v.int. The ancestor-walk in Step 4
    // may push spurious violations from the inflated values. These
    // are harmless: the rebalance loop's re-check (§11.8) filters
    // them after this step corrects the sums. Safe under the
    // sequential model (§7) — no sampling or external observation
    // occurs during this transient window; see §12.5.1 for
    // concurrent considerations.
    //
    // The Two-Path Coverage Lemma (§12.5.1) proves that Steps 4 and 8
    // together discover every violation that persists after both
    // mutations complete. Duplicates are filtered by the rebalance
    // loop's re-check.
    vtree_remove_leaf(v)
    // g.entry already set to null by vtree_remove_leaf (§8.2).

    // Step 8: Enqueue violations from V-Tree structural changes
    // (§11.12 sources 6–9). Dispatch based on whether the parent
    // collapsed (2-node → destroyed) or shrank (3-node → 2-node).
    // When change_point = null (v_parent was the V-root, a 2-node),
    // collapse_sibling is the new V-root and no ancestor-walk is
    // needed — is_violated returns false when the grandparent is null.
    if change_point ≠ null:
        push_leaf_removal_violations(change_point)        // source 6: ancestor walk

    if child_count = 2 and collapse_sibling ≠ null:
        // Collapse case: v_parent was destroyed.
        assert slot_occupied(collapse_sibling)   // Invariant: Step 7 re-parents, never destroys, the sibling
        push_collapse_violations(collapse_sibling)         // source 7: sole's children
        if change_point ≠ null:
            push_cousin_violations(collapse_sibling, change_point)  // source 9: cousins' children
    else if child_count = 3 and v_parent ≠ null:
        // 3→2 case: v_parent survives as a 2-node.
        // Source 6 (ancestor walk) starts from v_parent = change_point,
        // covering the remaining children's changed uncle context at
        // grandparent level via propagate_v_sums_from's importance decrease.
        // Source 8 below covers the remaining children's children.
        // Precondition: vtree_remove_leaf (Step 7) already removed v
        // from v_parent.children. The exclusion parameter is unnecessary.
        assert v not in v_parent.children
        for each child s of v_parent:
            for each child c of s:
                if is_violated(c): push c              // source 8: remaining siblings' children

    // Step 9: Update plateau ordered map (§5.6.7 eviction maintenance)
    update_plateau_map_after_eviction(g, p)

    // Step 10: Deallocate
    destroy(v)          // deallocate V-entry arena slot
    destroy(g)          // deallocate G-node arena slot
```

> **Lemma 12.3 (Collapse Safety).** In the collapse case (2-node `v_parent`
> destroyed), `sole` (the surviving sibling) is re-parented to `change_point`
> (grandparent). Its new uncle context is the siblings of `grandparent`
> under `great-grandparent` — the same set that `v_parent` faced before
> removal.
>
> _Proof._ By V-I1: `sole.int ≤ v_parent.int` (sole was a child of v_parent).
> By V-I3 (before removal): `v_parent.int ≤ max{new uncles}`. These uncles'
> intensities are unchanged by Step 7's removal (they are not on the
> propagation path from `vtree_remove_leaf`). Step 3's absorption propagation
> may have increased some uncles (if they lie on Path 1), but stronger uncles
> only strengthen the shield — the inequality chain remains valid.
>
> Therefore `sole.int ≤ max{new uncles}`, and sole cannot be violated at its
> new position. No explicit check is required. $\square$

> **Design Note (Step 1 Invariant).** After absorption and detachment,
> $p.\text{sum}$ is algebraically unchanged:
>
> $$p.\text{sum}_{\text{before}} = p.\text{own} + g.\text{sum} + (\text{other\_child.sum or } 0)$$
> $$p.\text{sum}_{\text{after}} = (p.\text{own} + g.\text{sum}) + 0 + (\text{other\_child.sum or } 0)$$
>
> The $g.\text{sum}$ term moves from the child contribution to
> $p.\text{own}$; the detachment removes it from the child
> contribution. The cancellation is exact regardless of whether $p$
> has a surviving child. Because `sum` is a stored field (not
> recomputed on access), its stale value is already correct after the
> compensating mutations in Step 1. No explicit sum update or G-Tree
> propagation is needed.
>
> Under the sequential model (§8), the transient state between
> absorption and detachment is not observable. Under concurrent
> operation (§17.3), the two mutations must be performed under a
> single lock or as a single atomic transaction covering the G-parent
> and the evicted child. Note: §17.3 (Write-Set Locality) analyzes
> concurrent _observations_; concurrent eviction with observations is
> not specified. Implementations supporting concurrent eviction should
> extend §17.3's analysis to include eviction's write set (G-parent
> modification, V-entry removal, structural collapse).

#### 12.5.1 Two-Path Coverage Lemma

**Lemma (Two-Path Coverage).** Steps 4 and 8 of the eviction operation
together detect every V-I3 violation that persists after both the
importance increase (Step 3) and the structural removal (Step 7)
complete.

**Proof.** A violation at node $n$ requires either (a) $n$'s own
importance increased (making $n$ exceed its uncle) or (b) one of $n$'s
uncles' importances decreased (weakening $n$'s shield). Define two
paths:

- **Path 1** ($p.\text{entry} \to$ V-root): the ancestor chain whose
  importances increased by Step 3's `propagate_v_sums`. Step 4 walks
  this path, catching self-violations (source (a)).
- **Path 2** ($v$'s former V-parent $\to$ V-root): the ancestor chain
  whose importances decreased by Step 7's `vtree_remove_leaf`. Step 8's
  `push_leaf_removal_violations` walks this path; its companion
  functions (`push_collapse_violations`, `push_remaining_sibling_violations`,
  `push_cousin_violations`) check the immediate neighbours at each
  level, catching uncle-weakening violations (source (b)).

**Interaction.** A violation requires either (a) the node's own
importance increased past its uncle, or (b) the node's uncle's
importance decreased below the node. Consider a node $X$ that lies
on neither Path 1 nor Path 2:

- **(Case A)** _$X$'s importance is unchanged._ $X$ is not on Path 1 (whose
  ancestors had sums increased) and not on Path 2 (whose
  ancestors had sums decreased). $X$'s own importance is unaffected
  by either mutation.
- **(Case B)** _$X$'s uncle could be on Path 1 (increased)._ A stronger uncle
  cannot cause a violation — it raises $\max\{u.\text{int}\}$,
  making $X$ _less_ likely to violate.
- **(Case C)** _$X$'s uncle could be on Path 2 (decreased)._ Then $X$ is adjacent
  to Path 2. Step 8's push functions check siblings' children at
  every level of Path 2 — $X$ is covered.
- **(Case D)** _$X$'s uncle is on neither path._ Then the uncle's importance is
  unchanged, and $X$'s importance is unchanged. No violation.

_(An uncle on both paths has competing effects — increased by
Step 3, decreased by Step 7. Three cases: (1) net decrease → reduces
to Case C (uncle weakened, $X$ covered by push functions); (2) net
increase → reduces to Case B (uncle strengthened, no violation
possible); (3) net zero → uncle unchanged, reduces to Case D (no
violation). Under the sequential model (§8), no observation occurs
between Steps 3 and 7, so transient intermediate states are not
observable. The net effect at completion is what matters for violation
detection.)_

No off-path node escapes detection. False positives (from
transient inflation) are filtered by the rebalance loop's
re-check.

**Monotone-overestimate principle.** Between Steps 3 and 7, the V-Tree
total is transiently inflated by $v.\text{int}$. The inflation only
_increases_ ancestor sums, so an ancestor can only appear _more_
violated than its eventual correct state — never less. Transient
inflation produces false positives (filtered by the rebalance loop's
re-check) but never false negatives: no real violation escapes
detection. Step 4 may push spurious violations from the inflated
window; Step 8 catches violations that emerge only after deflation.
Together the two steps have complementary coverage.

Under the sequential model (§8), no sampling occurs during this
transient window — `observe()` completes atomically from the
caller's perspective. For concurrent operation, the transient
inflation must be handled by the locking strategy described in §17.3.

$\square$

**The contour tip is destroyed.** Its energy flows to the parent. No
subtree traversal, no orphans, no deferred cleanup. The root exemption
guarantees the evicted cell has $d \geq 1$. Node count decreases by exactly
1 (Lemma 12.1).

**The sibling survives.** The parent transitions state:

- **Had 2 children → semi-internal.** One child vacated; the sibling
  remains. The parent is now partially exposed, receiving observations in
  the vacated half.
- **Had 1 child → fully exposed.** Both children gone. The parent sits on
  the contour at one level shallower.

**The parent strengthens.** Its V-entry gains the absorbed value. This
strengthens the uncle shield protecting the surviving sibling — the
structures behind the new contour edge are better defended.

**Violation tracking.** Eviction creates violations from two sources:

(a) The parent's importance update may violate the parent and V-Tree
ancestors (§11.12 sources 1–2). The ancestor-walk in Step 4 enqueues
these explicitly.

(b) The V-Tree removal in Step 7 decreases ancestor importances, weakening
uncle shields at every level (§11.12 sources 6–9). Step 8 dispatches to
the appropriate push functions based on the V-parent's pre-removal child
count. In the **collapse case** (2-node parent destroyed), the change point
is the grandparent and the push functions operate on the surviving sibling
(`collapse_sibling`) and its new neighbourhood:
`push_leaf_removal_violations(grandparent)` for the ancestor walk (source
6), `push_collapse_violations(collapse_sibling)` for the sole's children
(source 7), and `push_cousin_violations(collapse_sibling, grandparent)` for
cousins' children (source 9). In the **3→2 case** (parent survives), the
change point is the parent itself:
`push_leaf_removal_violations(parent)` for the ancestor walk (source 6) and
`push_remaining_sibling_violations(parent, v)` for the remaining siblings'
children (source 8). The push functions are called after
`vtree_remove_leaf` completes — they are not called inside
`vtree_remove_leaf` itself, which is a pure structural operation.

Both walks are $O(h_V)$. Violations queue for the trailing rebalance in
`check_evictions` (see §12.6).

> _Design note._ The G-root is permanently exempt. It cannot be removed.
> This guarantees the V-Tree always has at least one entry after the first
> observation, ensuring that the sampling distribution (§6.5) is always
> defined when total importance is positive.

---

### 12.6 The Eviction Scan

The scan walks the V-Tree, pruning subtrees with no unprotected
descendants.

```
function check_evictions():
    // Phase 1: Snapshot eligible candidates (collect-then-evict).
    candidates ← scan_for_candidates(V_root, 0)

    // Phase 2: Evict each candidate, re-verifying eligibility.
    // Invariant: no rebalancing occurs between individual evictions.
    // evict() pushes violations to the queue but never drains it.
    // The V-Tree's structure during Phase 2 is modified only by
    // vtree_remove_leaf (collapses), never by promotion or contraction.
    // This guarantees Phase 2 creates no new G-nodes — only Phase 3 does.
    for v in candidates:
        // Guard ordering: slot_occupied must precede all field accesses
        if not slot_occupied(v): continue       // arena slot freed by prior eviction
        if not v.is_evictable: continue         // eligibility changed
        if depth_V(v) ≤ D_evict: continue       // depth reduced by prior collapse
        evict(v)

    // Phase 3: Trailing rebalance — resolve all eviction-triggered violations.
    // Entry intensities are fixed during this rebalance — only V-Tree structure
    // and G-node positions change, not observation counts. The termination proof
    // (§11.13) guarantees the trailing rebalance completes: when P2 holds,
    // the Φ-decrease argument (§11.13.1) applies directly; when P2 fails,
    // the entry-creation bound (§11.13.2) applies. In both cases, the
    // Semi-Internal Consumption Lemma (§7.5.1) independently bounds the number
    // of legacy promotions.
    //
    // Eviction's absorption step (§12.5 Step 3) strengthens the G-parent's
    // V-entry. The G-parent's V-Tree position is independent of its G-Tree
    // position — it can sit at a shallow V-depth. If the strengthened entry
    // backs a semi-internal G-node and depth_V(c) ≤ D_evict, the §11.9
    // dispatcher uses legacy promote, creating a new G-node. When the heir
    // would land past D_evict, skip promote is used instead, suppressing
    // the futile creation. Each eviction increases at most one V-entry
    // (the G-parent's). However, a legacy promotion changes V-Tree
    // structure, which can alter uncle contexts and trigger further
    // violations that may themselves resolve as legacy promotions. The
    // total number of legacy promotions L₂ in the trailing rebalance
    // satisfies L₂ ≤ S_post-evict (the Semi-Internal Consumption Lemma,
    // §7.5.1): each consumes one semi-internal G-node, and no rebalancing
    // primitive creates new semi-internals. Phase 2 evictions may create
    // up to E new semi-internals (internal → semi-internal transitions),
    // so L₂ ≤ S₀ − L₁ + E where S₀ is the semi-internal count at
    // observe() entry (§7) and L₁ is legacy promotions from Step 6 of
    // observe() (the first rebalance, not the Phase 3 trailing rebalance
    // of `check_evictions()`). This
    // accounting is scoped to a single observe() call; standalone
    // invocations of check_evictions (e.g., during dynamic D_evict
    // adjustment) have their own S₀ at entry. The net node change from
    // the eviction pass is Δ|G| = −E + L₂, bounded above by S₀ − L₁ ≤ S₀.
    // When S₀ = 0, L₂ ≤ E and the eviction pass is non-positive. The
    // §7.5.3 budget invariant |G| + S + 2 ≤ G_max absorbs this bound.
    // The budget invariant is frame-invariant: it holds at every call
    // boundary regardless of whether the caller is observe() or an
    // external maintenance trigger (§12.7).
    if any evictions occurred:
        rebalance()

function scan_for_candidates(v, depth) → list:
    if v = null: return []

    if v is entry:
        if depth > D_evict and v.is_evictable:
            // Root exemption: the G-root is permanently exempt from
            // eviction (§12.5 early return). Filtering it here avoids
            // wasted work discovering a candidate that will be rejected.
            if v.gnode = G_root: return []
            return [v]
        return []

    // v is structural — prune if no unprotected descendants
    if not v.has_evictable: return []

    result ← []
    for c in v.children:
        result ← result ∪ scan_for_candidates(c, depth + 1)
    return result
```

The two-phase collect-then-evict pattern avoids mutating the V-Tree during
traversal. Phase 1 snapshots candidates into a list; Phase 2 iterates the
snapshot, re-verifying each candidate before evicting (prior evictions in
the same batch may invalidate slots or change eligibility). Phase 3's
trailing rebalance resolves all V-I3 violations generated by the eviction
batch.

> _Implementation note._ Violations pushed by earlier evictions in Phase 2
> may become stale due to structural changes (collapses, re-parenting) from
> later evictions in the same batch. The rebalance loop's `is_violated`
> re-check (§11.8) and `slot_occupied` guard filter these — no correctness
> issue arises, but the queue may contain entries that are no-ops when
> resolved.

> _Implementation note._ A naïve inline-evict scan (evicting during DFS
> traversal) is unsound: if a 2-node structural parent collapses during
> `vtree_remove_leaf`, the scan's iterator over `v.children` holds a
> dangling reference. The collect-then-evict pattern eliminates this
> hazard.

> _Implementation note._ The `slot_occupied` check in Phase 2 is a
> defensive guard. In the current design, evicting one candidate cannot
> destroy another candidate's V-entry or backing G-node — only the
> evictee's own entry and G-node are removed. The check protects against
> future design changes where cascading removals might invalidate
> queued candidates.

> _Design note (one-tide lag)._ When Phase 2 evictions make a
> parent terminal (both children removed), the parent becomes
> eviction-eligible but was not in the Phase 1 snapshot. It
> survives until the next `check_evictions` call. This is
> consistent with the one-tide-per-observation model (§12.7).
> The parent's strengthened V-entry (from absorption) will tend
> to promote toward shallower V-depth via the competitive
> mechanism, moving it further from $D_{\text{evict}}$, not
> closer. Under the typical regime where absorption materially
> strengthens the parent relative to its neighbourhood, the lag
> is cosmetic. In a globally cold region where even the absorbed
> value is small relative to the parent's uncles, the parent may
> remain deep and become eviction-eligible the next tide —
> correct behaviour, since it reflects genuine insignificance.

> _Implementation note (sibling sparing)._ The most common source of
> cross-candidate interference is siblings under a 2-node V-parent: if
> candidates v₁ and v₂ are both children of a 2-node, evicting v₁
> triggers a collapse that re-parents v₂ to the grandparent, reducing
> v₂'s V-depth by 1. If this drops v₂ to or below D_evict, the Phase 2
> re-check spares it. This is the primary mechanism by which evictions
> rescue their siblings.
>
> _Worked example (both siblings evicted)._ Candidates $v_1 = [0,1)$ and
> $v_2 = [1,2)$ are both children of terminal G-parent $[0,2)$ and both
> past $D_{\text{evict}}$. Phase 2 processes $v_1$ first: G-parent becomes
> semi-internal, absorbs $v_1$'s value, V-structural parent collapses (if
> 2-node), $v_2$ may be re-parented. If $v_2$'s new V-depth still exceeds
> $D_{\text{evict}}$, Phase 2 evicts $v_2$ too: G-parent becomes terminal,
> absorbs $v_2$'s value, is now eviction-eligible but was not in the
> Phase 1 snapshot — survives until the next tide. The G-parent's
> strengthened V-entry (from double absorption) promotes toward shallower
> V-depth, typically moving it away from $D_{\text{evict}}$, not toward it.

The eviction order within Phase 2 is DFS from Phase 1. A prior eviction's
structural collapse may reduce a later candidate's V-depth below
$D_{\text{evict}}$, sparing it. This order dependence does not affect
correctness — every individual eviction is re-verified — and any spared
candidate is reconsidered in the next tide (§12.7). Note that the final
state after `check_evictions` is order-dependent: different traversal
orderings may spare different candidates, producing valid but potentially
non-unique results (e.g., if siblings $v_1$ and $v_2$ are both past
$D_{\text{evict}}$ under a 2-node V-parent, evicting $v_1$ first collapses
the parent and reduces $v_2$'s V-depth by 1, potentially sparing it;
evicting $v_2$ first would spare $v_1$ instead). This contrasts with the rebalance loop, whose final
invariant-satisfaction state (all violations resolved) is determined
regardless of resolution order — though the V-Tree's exact shape (which
rotations were applied) may differ between orderings; shape confluence is
not proven. No particular eviction ordering is preferred for correctness;
the DFS order from Phase 1 is a natural consequence of the scan traversal,
not a deliberate heuristic. An alternative ordering (e.g., deepest V-depth
first) could maximise structural collapse per tide but would require
sorting the candidate list, adding $O(E_t \log E_t)$ overhead for no
correctness benefit.

> _Implementation note (Canonical ordering for testing)._ For deterministic
> regression testing, a canonical Phase 2 ordering is recommended: sort
> candidates by `(depth_V descending, g.l ascending)` before evicting.
> Deepest-V-depth first maximizes structural collapse per tide (deeper
> entries are further from the eviction threshold, less likely to be spared
> by prior collapses). Within a depth, left-endpoint order provides spatial
> determinism. The sort costs $O(E_t \log E_t)$ and is recommended only for
> test builds; production builds may use the unsorted DFS order from
> Phase 1. (With cached V-depth — see the depth caching note below — the
> sort key is $O(1)$ per candidate, making the sort cost $O(E_t \log E_t)$
> with no hidden per-element walks.)

> _Cross-reference (§7.5.1)._ Legacy promotions may fire during the
> trailing rebalance of an eviction pass. Eviction's absorption step
> strengthens the G-parent's V-entry, which can sit at a shallow
> V-position (V-Tree parentage is independent of G-Tree parentage).
> If the strengthened entry backs a semi-internal G-node and
> $\text{depth}_V(c) \leq D_{\text{evict}}$, the §11.9 dispatcher
> uses legacy promote — creating a new G-node at a viable V-depth.
> When the heir would land past $D_{\text{evict}}$ (immediately
> eviction-eligible), the depth gate dispatches to skip promote
> instead, suppressing the futile creation. The total number of
> legacy promotions $L_2$ in the trailing rebalance satisfies
> $L_2 \leq S_{\text{post-evict}}$ by the Semi-Internal Consumption
> Lemma (§7.5.1): each consumes one semi-internal G-node, and no
> rebalancing primitive creates new semi-internals during the
> rebalance. The §7.5 budget invariant $|G| + S + 2 \leq G_{\max}$
> absorbs the maximum net node change from both the split (Step 5 of
> `observe()`) and all legacy promotions across both trailing rebalances.

> _Design note._ Phase 2 re-checks each candidate's V-Tree depth at
> eviction time — depths may have been reduced by structural collapses
> from prior Phase 2 evictions (see the sibling sparing note below) —
> but does not use hypothetical post-trailing-rebalance depths, since
> the trailing rebalance has not yet run. An eviction candidate might
> have promoted above $D_{\text{evict}}$ if the trailing rebalance had
> run first. This is an intentional design choice: contraction takes
> priority over promotion when both apply. The trailing rebalance in
> Phase 3 resolves any violations created by the eviction batch after
> all evictions are complete.

The `has_evictable` flag (V-I7) enables efficient pruning: subtrees
containing only protected entries are skipped in $O(1)$. The scan touches
only the exposed contour tips.

**Cost.** Phase 1 (the scan) costs $O(E_t + S_t)$ where $E_t$ is the
number of evictable entries past $D_{\text{evict}}$ and $S_t$ is the number
of structural nodes on paths to them (those not pruned by
`has_evictable`). Phase 2 (the evictions) costs
$O(E_t \cdot (h_V + \log P))$: each eviction performs absorption $O(1)$,
V-sum propagation and ancestor violation walks $O(h_V)$, a `depth_V`
re-check $O(h_V)$, and a plateau map update $O(\log P)$. Phase 3 (the
trailing rebalance) resolves all accumulated violations; its cost is bounded
by the $\Phi$-decrease argument (§11.13). The total cost is dominated by
Phase 2 in all practical cases. In the worst case the entire V-Tree is
traversed in Phase 1, but pruning typically reduces this to sub-linear.

**Worst-case per-call latency.** Under maximum eviction pressure (lowered
$D_{\text{evict}}$, most subtrees containing at least one evictable entry),
Phase 1 degenerates to $O(|V|)$ — a full V-Tree traversal. This occurs
precisely when the system is most constrained, making the cost spike
anti-correlated with headroom. For latency-sensitive implementations, an
**incremental scan** strategy is recommended: limit Phase 1 to $k$
candidates per `observe()` call, maintaining a resumption cursor across
calls. Each call evicts at most $k$ entries; the scan completes over
$\lceil E_t / k \rceil$ calls. This trades per-call latency
($O(k \cdot h_V)$) for contraction latency (full eviction takes multiple
calls). The correctness argument is unchanged — each individual eviction is
self-contained, and the trailing rebalance in Phase 3 resolves violations
from that call's batch.

**Depth caching.** The `depth_V(v)` re-check in Phase 2 requires walking
from $v$ to the V-root to count edges, costing $O(h_V)$ per candidate. For
$E_t$ candidates, total re-check cost is $O(E_t \cdot h_V)$, which can
dominate Phase 2 under heavy eviction. **Recommended:** cache V-Tree depth
as a field on each V-node, maintained incrementally during structural
changes (promotion, contraction, collapse). Insertion sets
`depth ← parent.depth + 1`. Promotion and collapse update the moved node
and propagate to descendants — but descendants of a promoted node all shift
by the same constant, so a lazy delta scheme (store depth relative to
parent) reduces maintenance to $O(1)$ per structural operation. Without
caching, each re-check costs $O(h_V)$, so the total Phase 2 cost is
$O(E_t \cdot h_V)$ for both eviction and re-checks combined. With cached
depth, re-checks drop to $O(1)$ each, reducing total Phase 2 cost to
$O(E_t \cdot h_V)$ for eviction proper (absorption, V-propagation) plus
$O(E_t)$ for re-checks — the re-check term becomes sub-dominant. Under the
relative-depth scheme, a node whose V-parent was destroyed in a collapse
must have its depth recomputed from the new parent (assigned during
`vtree_remove_leaf`'s re-parenting step). This is $O(1)$ per collapse —
the new parent's absolute depth is known, and the child's depth is
parent.depth + 1.

> _Implementation note._ The `has_evictable` flag prunes subtrees
> containing no terminal (zero-child) G-nodes, but does not encode
> V-Tree depth. A subtree with `has_evictable = true` may contain
> only evictable entries that are _not_ past $D_{\text{evict}}$ — the
> scan descends into such subtrees only to reject every candidate at
> the leaf level. A depth-aware variant (e.g.,
> `has_evictable_past_depth_k`) would tighten pruning at the cost
> of maintaining a more complex flag during structural changes. The
> current design favours simplicity; the $S_t$ term in the cost
> analysis accounts for these false-positive descents.

---

### 12.7 Bottom-Up Contraction

When an entire region cools, eviction proceeds inward in successive tides:

```
Tide 1:  Outermost contour tips past D_evict → removed.
         Parents become semi-internal.
         Parents absorb values → strengthen.

Tide 2:  Surviving siblings also past D_evict → removed.
         Parents lose last dependent, join the contour tips.

Tide 3:  Newly tip-exposed parents past D_evict → removed.
         Grandparents absorb. Contour retreats one level shallower.
         ...
```

Each tide is one `check_evictions` pass. Since §8 calls
`check_evictions()` unconditionally in every `observe()`, one tide (one
`check_evictions` pass, which may be a no-op) runs per observation — most
tides find nothing past $D_{\text{evict}}$ and complete with no evictions.

> _Tides are observation-driven._ If no observations arrive, no tides
> occur and the tree does not contract autonomously. For applications
> requiring contraction under quiescent conditions, external triggers can
> invoke `check_evictions()` independently — e.g., periodic maintenance
> timers, memory pressure callbacks, or explicit user calls. Standalone
> triggers should call `adjust_depth_gates()` (§7.4) before
> `check_evictions()` to ensure depth gates reflect current budget
> pressure. Such standalone invocations have their own $S_0$ at entry
> (§7.5.1) and function
> identically to observation-triggered tides. Standalone invocations of
> `check_evictions()` — including those triggered by `decay()` (Phase 5
> of §14.4) — function as tides without requiring new observations. A
> `decay()` call with $\text{att} < 1$ or $\text{att} = 0$ followed by
> its internal `check_evictions()` constitutes a tide.

Each eviction within a tide is atomic.
Individual evictions are **monotonically inward**: each eviction can only
bring parent nodes closer to the contour tips, never further. However, the
trailing rebalance (Phase 3 of `check_evictions`) may partially reverse
this through legacy promotion — creating new G-nodes that move parents
away from the tips — bounded by §7.1. Total unwinding time for a
region nested $k$ tides deep past $D_{\text{evict}}$ is $k$ tides, each
removing one layer of contour tips.

Under user-applied attenuation ($\text{att} < 1$, §14.3), this is gradual. Traffic stops, entries cool,
they drift past $D_{\text{evict}}$ — outermost first, then the next layer,
then the next. The tree contracts as smoothly as it expanded. Under annihilation ($\text{att} = 0$), the contraction is immediate: zeroed entries have ground importance (when P2 holds, $\nu$ is the minimum), are pushed to maximum V-depth by the trailing rebalance, and are evicted in a single `check_evictions` pass. Annihilation fast-forwards the tidal sequence — what would take $k$ tides under gradual attenuation completes in one. Between
tides, newly tip-exposed parents receive observations and may re-promote
above $D_{\text{evict}}$, arresting the contraction (a distinct mechanism
from the intra-tide reversal by trailing rebalance described above). The
tidal sequence
describes the trajectory under sustained cooling; any tide can be arrested
by renewed activity.

**Node count per tide.** Each tide removes at most the current set of
contour tips past $D_{\text{evict}}$. $|G|$ decreases by the number of
evictions in the tide. For a coherent region unwinding uniformly from
G-depth $d$ to G-depth $d - k$:

- Each tide removes one layer of contour tips from that region.
- The total nodes removed is the count of G-nodes in the unwound layers.

In practice, partial unwinding — one sibling evicted, the other surviving
and deepening — means different branches unwind at different rates.
The per-eviction guarantee (Lemma 12.1: $|G|$ decreases by 1) always holds; the
aggregate description is an idealization for coherent regions.

**Rapid re-expansion after absorption.** When eviction makes a parent
terminal (both children gone), the parent's own-value includes both its
pre-split accumulation and any value absorbed from evicted children. Two
regimes apply:

**Under raw accumulation (no decay):** Re-expansion is certain. The parent
split when $g.\text{sum} > \theta$ (§10.1), its own value was at least
$\theta$ at that time, and absorption only increases $g.\text{own}$. If
the parent's V-entry also holds a shallow enough position
($\text{depth}_V \leq D_{\text{create}}$), the very next observation
routing to the parent triggers `attempt_refine` and immediately re-splits
it. Eviction at depth $d$ can thus seed re-creation at depth $d - 1$ within
one observation.

**Under user-applied decay:** Re-expansion requires fresh observations. The
parent's own value may have decayed below $\theta$ between the original
split and the eviction. The parent sits on the contour, receiving
observations in its now-exposed range, until it re-crosses $\theta$ and
earns a shallow enough V-position. Re-expansion is possible but not
immediate.

In both regimes, benchmark compounding (§13.5) ensures the bar is higher:
the parent's frozen benchmark now includes all previously absorbed energy,
so children must accumulate past a hardened threshold. A semi-internal
parent (one child remaining) cannot re-split — `attempt_refine` returns
immediately because `has_dependents` is true. Re-expansion is possible only
when the parent is fully terminal.

> _Note._ Re-expansion can also fail immediately if the rebalance cascade
> triggered by the splitting observation pushes the new entries past
> $D_{\text{evict}}$, causing eviction within the same `observe()` call.
> This is the buffer oscillation phenomenon described in §7.3
> (implementation note on buffer width). The cycle is futile but
> self-limiting and invariant-preserving.

---

### 12.8 The Two Simplifications

Eviction produces two independent effects on the G-Tree's structural
complexity:

| Effect       | What changes            | Mechanism                                                                                    | Visible in                   |
| ------------ | ----------------------- | -------------------------------------------------------------------------------------------- | ---------------------------- |
| **Direct**   | Node count decreases    | A node is removed; its parent covers the gap at one shallower G-depth                        | Node count $\|G\|$           |
| **Implicit** | Depth variation narrows | Sustained eviction removes the nodes at maximum G-depth; the tree's depth profile compresses | Plateau count $P$, max depth |

**Direct simplification** is local. One node gone, its parent covers the
gap. Adjacent plateaus may merge if the parent's depth matches its
neighbor. The node count drops by 1. The plateau count $P$ changes by
at most $+2$ per eviction: when an interior cell of a plateau is evicted,
the parent at depth $d - 1$ creates two new depth transitions (one on each
side of the vacated range), splitting the original plateau in two. An
adjacent same-depth merge may cancel one or both transitions. Net
$\Delta P \in \{-2, -1, 0, +1, +2\}$.

> **Lemma 12.2 (Bounded plateau change).** A single eviction changes the plateau
> count by $\Delta P \in \{-2, -1, 0, +1, +2\}$.
>
> **Proof.** Let $g$ be the evicted terminal at G-depth $d$, with contour
> cell $[l, r_g)$. The cell has exactly two boundary coordinates: $l$ and $r_g$.
> (At domain boundaries, one coordinate is shared with the domain edge and
> contributes no transition; the argument still applies with one fewer
> boundary.) Eviction changes the contour depth at $[l, r_g)$ from $d$ to
> $d{-}1$ (the parent covers the vacated range at depth $d{-}1$).
>
> At each boundary coordinate $x \in \{l, r_g\}$, let $\delta$ be the contour
> depth on the far side of $x$ (unchanged by this eviction). The depth
> transition at $x$ changes as follows:
>
> | Far-side depth $\delta$                 | Before ($d$ vs $\delta$)    | After ($d{-}1$ vs $\delta$)      | $\Delta$ transitions |
> | --------------------------------------- | --------------------------- | -------------------------------- | -------------------- |
> | $\delta = d$                            | No transition (same depth)  | Transition ($d{-}1 \neq d$)      | $+1$                 |
> | $\delta = d{-}1$                        | Transition ($d \neq d{-}1$) | No transition (same depth)       | $-1$                 |
> | $\delta \neq d$ and $\delta \neq d{-}1$ | Transition                  | Transition (depth still differs) | $0$                  |
>
> Two boundary coordinates, each contributing $\Delta \in \{-1, 0, +1\}$.
> The nine combinations of $(\Delta_l,\, \Delta_r)$ map to five distinct
> $\Delta P$ values:
>
> | $(\Delta_l,\, \Delta_r)$                   | $\Delta P$ |
> | ------------------------------------------ | ---------- |
> | $(-1,\, -1)$                               | $-2$       |
> | $(-1,\, 0)$ or $(0,\, -1)$                 | $-1$       |
> | $(-1,\, +1)$ or $(0,\, 0)$ or $(+1,\, -1)$ | $0$        |
> | $(0,\, +1)$ or $(+1,\, 0)$                 | $+1$       |
> | $(+1,\, +1)$                               | $+2$       |
>
> All nine combinations are physically realisable. The total
> change satisfies:
>
> $$\Delta P = \sum_{x \in \{l,\,r_g\}} \Delta_x \;\in\; \{-2,\, -1,\, 0,\, +1,\, +2\}$$
>
> $\square$

**Corollary (Edge-Adjacent Evictions).** When the evicted cell is adjacent
to a domain boundary (left edge at $l = 0$ or right edge at $r = 2^N$),
the boundary coordinate contributes $\Delta = 0$ unconditionally — there
is no contour depth on the far side to transition to. The bound tightens
to $\Delta P \in \{-1, 0, +1\}$ for edge-adjacent evictions.

Note that $\Delta P = +2$ (eviction _increasing_ plateau count) can occur
when an interior cell is removed, breaking a uniform run into two separate
plateaus. This local increase is expected; the global simplification effect
(§12.8) is cumulative across many evictions, not guaranteed per-eviction.
The following examples illustrate the extremes of this bound.

> **Worked example ($\Delta P = +2$).** Consider a plateau of 4 cells at
> depth 3: $[0,1), [1,2), [2,3), [3,4)$. All share depth 3, so they form
> one plateau ($P_0$). Now evict the interior cell $[1,2)$. (This
> requires $[1,2)$ to sit deeper in the V-Tree than its same-depth
> siblings — plausible if $[1,2)$ received fewer observations,
> placing it at a lower competitive rank.) Its parent $[0,2)$ at
> depth 2 absorbs it and becomes semi-internal. The contour now reads:
> depth 3 at $[0,1)$, depth 2 at $[1,2)$, depth 3 at $[2,3)$ and
> $[3,4)$. This produces three plateaus where there was one:
> $P_A = [0,1)$ at depth 3, $P_B = [1,2)$ at depth 2, $P_C = [2,4)$
> at depth 3. Net $\Delta P = +2$.
>
> **Worked example ($\Delta P = -2$).** Pre-established configuration:
> G-node $[0,2)$ at depth 2 is semi-internal — its left child $[0,1)$
> was previously evicted in an earlier tide, so $[0,2)$ covers $[0,1)$
> on the contour at depth 2. Its surviving right child $[1,2)$ is
> terminal at depth 3. To the right, $[2,4)$ sits on the contour at
> depth 2. The contour before this eviction reads: depth 2 at $[0,1)$,
> depth 3 at $[1,2)$, depth 2 at $[2,4)$ — three plateaus ($P_L$,
> $P_M$, $P_R$). Now evict $[1,2)$. Parent $[0,2)$ absorbs it and
> becomes terminal at depth 2 (both children now gone). The contour
> reads: depth 2 at $[0,2)$ and depth 2 at $[2,4)$. These merge into
> one plateau at depth 2. Net $\Delta P = -2$ (three plateaus became
> one). Note: the semi-internal state of $[0,2)$ was the precondition
> that made this $-2$ outcome possible — it required a prior eviction
> to set up.

**Implicit simplification** is the cumulative consequence of sustained
node-count reduction. No single eviction guarantees a decrease in $P$ — in fact, a single eviction can locally
increase $P$ by splitting a plateau. The implicit effect is the sustained
consequence of $|G|$ decreasing monotonically.

The mechanism is structural. Each eviction removes one node. As the tree
shrinks, there are fewer nodes to support depth variation. The nodes at
the greatest G-depth — which created the most contour complexity by
forcing deep branches in the tree — are exactly those most likely to be
targeted for eviction (§12.2). As these deep nodes are removed, the tree's
depth profile compresses. Adjacent regions that previously sat at different
depths _may_ find themselves at the same depth when the deep nodes that
distinguished them are removed. If so, their plateaus merge, and the
contour smooths at locations potentially far from any individual eviction
site. Whether this occurs depends on the eviction order, which is governed
by the V-Tree's competitive ranking — a workload-dependent property
(§12.2).

**The formal chain.** Two proven results and one design-case argument
connect eviction to simplification:

1. **Node count decreases per eviction** (Lemma 12.1) — _proven,
   unconditional._ Every eviction removes exactly 1 node.
2. **Plateau count is bounded by node count** (proposition, §12.1.1) —
   _proven, unconditional._ $P \leq |G|$, so the capacity for structural
   complexity shrinks in lockstep with the node count.
3. **Eviction targets the deepest, coldest nodes** (§12.2) — _design-case
   analysis._ The V-Tree's significance ordering ensures that nodes at the
   greatest G-depth are typically those evicted first, under workloads with
   spatial locality and temporal decay. This is a dynamic property of the
   V-Tree ordering, not a structural invariant. See §12.2's scope note for
   the regime under which this holds.

The first two results are theorems. The third describes which nodes are
removed first — the answer depends on the V-Tree's competitive ranking,
which correlates with G-depth under the conditions described in §12.2. The
architectural guarantee is links 1–2: eviction provably shrinks the tree,
and a smaller tree provably supports fewer plateaus. Link 3 characterizes
_which_ nodes are removed first — the answer is workload-dependent but
architecturally favorable under the recommended regime.

**The effect under noisy input.** Under sustained diffuse spray, new nodes
are created as fast as old ones are removed:

- $|G|$ stays roughly constant — the direct effect washes out.
- The node distribution shifts: nodes at extreme G-depths are
  replaced by nodes at moderate G-depths. The tree's depth profile
  migrates from the extremes toward the center.
- $P$ tends to decrease — the surviving nodes have less depth variation
  and are more likely to share a depth, forming longer plateaus.

The tree is **not shrinking** — it is **regularizing**. Scattered deep
branches are consolidated into the moderate-depth interior, and the
surviving nodes form longer contiguous runs at similar depths. Adjacent
ranges that receive similar observation counts end up at similar depths.
The system consolidates regions of uniform activity — not by comparing
adjacent cells, but as a structural consequence of the node count being
drawn down (Lemma 12.1) while the domain extent holds steady.

The contour does not simplify because someone smoothed it. It simplifies
because the tree has less material with which to be complex — the node
count that could support depth variation is being provably drawn down
(Lemma 12.1), the capacity for high plateau count is being provably constrained
(proposition: $P \leq |G|$), and the nodes removed first are those at the
greatest depth (§12.2).

No stronger statement — that $P$ decreases monotonically under sustained
eviction — holds. Lemma 12.2's $\Delta P = +2$ worked example is a
constructive counterexample: evicting an interior cell of a uniform-depth
plateau _increases_ the plateau count. Per-eviction monotone decrease is
provably false. The Plateau–Node-Count Bound constrains the _capacity_ for
complexity; whether that capacity is realised depends on eviction ordering
(§12.2). The claim that $P$ tends to decrease under the recommended
operating regime is an empirical tendency, not a theorem.

---

### 12.9 Correctness of Tip-Only Eviction

#### 12.9.1 Structural Invariants

**No orphans.** Evicted nodes have no G-children. No V-entries are
stranded without backing G-nodes. The contour always traces a valid
boundary.

**Contour completeness.** After every eviction, every coordinate in
$[0, 2^N)$ is still covered by exactly one contour cell. The parent
absorbs the evicted range and either becomes semi-internal (covering
the vacated half) or terminal (covering the entire parent range).
There are no gaps. The contour remains a complete tiling of the domain.

**Tree connectedness.** The root always exists. Every other node's
parent is also present (eviction-immunity from having dependents).
Therefore the G-Tree is a connected rooted subtree — no orphaned
nodes.

**Sampling is always safe.** Every V-entry reached during sampling
backs a live G-node. The interior of the tree is intact.

**Typical ordering.** The coldest entries are typically at the
contour's deepest tips — they are removed first (§12.2: the
correlation between V-depth and G-depth is strong but not structural).
Structurally interior nodes are reached only after the tips above them
have been removed.

**Convergence.** Every eviction either removes a tip while a sibling
remains (parent stays interior) or removes the last sibling, promoting
the parent to the tips. Within Phase 2, individual evictions are
monotonically inward; the trailing rebalance (Phase 3) may partially
reverse this through legacy promotion (bounded by §7.5.1).

#### 12.9.2 Convergence to Fixed Point

Suppose no further observations arrive, $D_{\text{evict}}$ does not
increase, but `check_evictions()` continues to be invoked (e.g., by an
external maintenance trigger — see §12.7). The G-Tree contracts to a
**fixed point** where no eviction-eligible entries remain — but not
necessarily to the root.

Define potential $\Phi = |G| + S$ where $S$ is the semi-internal
count. Track $\Delta\Phi$ per event:

| Event                                   | $\Delta\lvert G\rvert$ | $\Delta S$ | $\Delta\Phi$ |
| --------------------------------------- | ---------------------- | ---------- | ------------ |
| Eviction, parent internal→semi-internal | $-1$                   | $+1$       | $0$          |
| Eviction, parent semi-internal→terminal | $-1$                   | $-1$       | $-2$         |
| Legacy promotion                        | $+1$                   | $-1$       | $0$          |

> _Note._ The table omits the case where the G-parent is already
> terminal before eviction. This case cannot occur: the evicted node
> was the parent's G-child, so the parent must have had at least one
> child (the evictee) and therefore cannot have been terminal. The
> three rows exhaust all possibilities.

$\Phi$ is weakly non-increasing. Starting
$\Phi_0 \leq 2|G_0| - 1$ (since $S \leq |G| - 1$ by §7.5.5), $\Phi$ can
never rise. However, $\Phi$-neutral events ($\Delta\Phi = 0$) —
evictions creating semi-internals and legacy promotions consuming
them — can alternate without reducing $\Phi$. Termination in finite
events is established by §12.9.3: the PIE budget (A) is finite, ghost
evictions produce no cascades (B), and recycled semi-internals cannot
self-excite (E), so the $\Phi$-neutral cycles must exhaust their fuel.
Intuitively: each $\Phi$-neutral cycle consumes one original node (via
PIE) and may temporarily replace it with a ghost (via LP + GE), but the
ghost is inert — it cannot seed further cycles. The stock of original
nodes is finite and strictly decreasing, so the cycles must terminate.
The system converges to a fixed point $\Phi_\infty$ where all
surviving terminal entries sit at V-depth $\leq D_{\text{evict}}$. At
this point, no further evictions occur under static conditions.

#### 12.9.3 Finite-Time Bound

We bound the total number of events during contraction to the fixed
point (no observations, static $D_{\text{evict}}$, repeated
`check_evictions()`). Classify every event into three types:

| Class                        | Symbol | $\Delta\lvert G\rvert$ | $\Delta S$   | Description                                                  |
| ---------------------------- | ------ | ---------------------- | ------------ | ------------------------------------------------------------ |
| Positive-importance eviction | PIE    | $-1$                   | $\pm 1$      | Evicts a node with non-minimum importance                    |
| Legacy promotion             | LP     | $+1$                   | $-1$         | Consumes a semi-internal, creates a ghost ($g.\text{sum}=0$) |
| Ghost eviction               | GE     | $-1$                   | $+1$ or $-1$ | Evicts a ghost; parent becomes semi-internal or terminal     |

The bound is derived in a single downward chain — each step uses only
prior steps.

**(A) PIE budget.** Each PIE removes a distinct original G-node
(Lemma 12.1); the root is exempt.

$$\text{PIE} \leq |G|_0 - 1$$

**(B) Ghost inertness (conditional on P4: $\nu \oplus a = a$).** A ghost
has $g.\text{sum} = 0$ and importance $\nu$. When P4 holds ($\nu$ is the
identity element): evicting the ghost means the parent absorbs zero —
no importance changes, `propagate_v_sums` propagates a zero delta,
and the ancestor violation walk finds nothing. The V-Tree structural
removal is equally inert: the ghost was never the max uncle for any
node (its importance is the bottom element under P2, which follows
from P4 in the standard configuration), so removing it cannot reduce
any node's max-uncle value. In the 2-node collapse case, the
V-structural parent is destroyed and the surviving sibling (`sole`)
is re-parented to the grandparent. This is still inert:
parent.int $=$ $\nu$ $\oplus$ sole.int $=$ sole.int (since $\nu$ is
the identity), so `sole` replaces parent as uncle to its cousins at
equal importance — no uncle weakens. (More generally: V-I3 held
pre-eviction, so every niece/nephew of a ground-importance entry
must itself have ground importance — the entire affected neighbourhood
is at ground, and collapse cannot weaken any shield.) **Ghost evictions
push zero violations to the queue and therefore trigger zero legacy
promotions.**

When P4 fails ($\nu \oplus a \neq a$ in general): ghost importance
$\nu$ may not act as an identity. Eviction absorption may change the
parent's importance, and ghost removal may reduce a node's max-uncle
value if the ghost served as the heaviest uncle. Ghost evictions are
**not** inert in this configuration — they may trigger violations
and legacy promotions. The ghost fast path (§12.5 Step 2) is
unavailable. The contraction bound degrades from $O(|G|_0 \cdot h_V)$
to $O(|G|_0 \cdot h_V^2)$: each ghost eviction may now trigger an
$O(h_V)$ violation cascade rather than terminating in $O(1)$.

**(C) Cascade depth per PIE.** A single PIE's absorption strengthens
one V-entry (the G-parent's). The resulting violation cascade traverses
at most $O(h_V)$ levels of the V-Tree, with $O(1)$ side-effect
violations per level (§11.11.2). Each resolution may be an LP.
Therefore:

$$\text{LP triggered per PIE} \leq O(h_V)$$

**(D) GE $\leq$ LP.** Each ghost is created by exactly one LP and
evicted at most once.

**(E) Ghost recycling is bounded by (C), not by pool size.** A ghost
eviction can restore a semi-internal (parent: internal →
semi-internal). This recycled semi-internal can be consumed by a
future LP — but only if a **fresh PIE** seeds a new violation cascade
that reaches it. Ghost evictions produce no violations (B), so recycled
slots cannot self-excite. Every LP, including those consuming recycled
semi-internals, traces back through the violation queue to a PIE.
The cascade-depth bound (C) limits total LP regardless of how many
times individual semi-internal slots are recycled: the $O(h_V)$ bound
is a property of the V-Tree walk structure, not of the pool state, so
a recycled slot consumed within a PIE's cascade counts against that
PIE's $O(h_V)$ budget exactly as a fresh one does.

> _This is the key insight._ The semi-internal pool size at any
> instant is bounded by $S_k \leq S_0 + |G|_0 - 1$ (each PIE creates
> at most one new semi-internal). But pool size bounds the stock, not
> the throughput. Recycling lets a slot of size 1 serve multiple LPs
> across successive tides. What actually bounds the **throughput** is
> the cascade depth: each PIE can drive at most $O(h_V)$ LPs through
> the violation queue, regardless of whether those LPs consume fresh
> or recycled semi-internals.

**(F) Total events.**

$$\text{Total events} = \text{PIE} + \text{LP} + \text{GE}$$

$$
\leq \underbrace{(|G|_0 - 1)}_{\text{(A)}}
     + \underbrace{O(|G|_0 \cdot h_V)}_{\text{(A)} \times \text{(C)}}
     + \underbrace{O(|G|_0 \cdot h_V)}_{\text{(D)}}
$$

$$\boxed{\text{Total events} = O(|G|_0 \cdot h_V) \quad \text{(when P4 holds)}}$$

**(G) Total work.**

| Event | Per-event cost                                                                                         | Count                           | Contribution                    |
| ----- | ------------------------------------------------------------------------------------------------------ | ------------------------------- | ------------------------------- |
| PIE   | $O(h_V)$ — propagation walks                                                                           | $O(\lvert G\rvert_0)$           | $O(\lvert G\rvert_0 \cdot h_V)$ |
| LP    | $O(1)$ — local restructuring (side-effect violations are separate resolutions within the same cascade) | $O(\lvert G\rvert_0 \cdot h_V)$ | $O(\lvert G\rvert_0 \cdot h_V)$ |
| GE    | $O(1)$ — zero-delta early termination (see below)                                                      | $O(\lvert G\rvert_0 \cdot h_V)$ | $O(\lvert G\rvert_0 \cdot h_V)$ |

$$\boxed{\text{Total work} = O(|G|_0 \cdot h_V) \quad \text{(when P4 holds)}}$$

Events and work are the same order because the numerous cheap events
(LP, GE) contribute no more total work than the fewer expensive events
(PIE).

> _When P4 fails (elevated configuration)._ When $\nu$
> is not the identity element, ghost inertness (B) does not hold. Ghost
> evictions may trigger $O(h_V)$ violation cascades, inflating per-GE
> cost from $O(1)$ to $O(h_V)$. Total work degrades to:
>
> $$\text{Total work} = O(|G|_0 \cdot h_V^2) \quad \text{(when P4 fails)}$$
>
> The event count is unchanged — the termination argument (A–E) does
> not depend on ghost inertness. Only the per-event cost changes.
> Conforming implementations must implement the ghost fast path
> (§12.5 Step 2) to achieve the P4-enabled bound.
>
> (The signed configuration has P4 ($0 + a = a$), so it achieves the
> $O(|G|_0 \cdot h_V)$ bound.)

**Ghost fast path (normative).** The $O(1)$ per-GE cost requires early
termination on zero absorption: when `evict` detects $g.\text{sum} = 0$,
it must bypass Steps 3–4 of §12.5 (importance update and
ancestor violation walk — both propagate a zero delta and find nothing,
but cost $O(h_V)$ each without the check). Step 1's detachment and
Step 5's flag updates must still execute — the parent's exposure and
evictable state genuinely changes. Steps 6–10 (V-Tree removal,
violation push, plateau update, deallocation) must also execute, but
`vtree_remove_leaf`'s sum propagation early-terminates at $O(1)$ on
the zero delta, and Step 8's violation push functions find nothing
(the removed entry had zero importance, so no uncle shield weakened).
Without the fast path, each GE walks to the V-root three times
accomplishing nothing, costing $O(h_V)$ per GE and inflating total
work to $O(|G|_0 \cdot h_V^2)$ — which is $O(|G|_0^3)$ under a
degenerate chain. **Conforming implementations must implement this
fast path to achieve the stated work bound.**

**(H) Regime table.**

| V-Tree shape     | $h_V$                        | Total events (P4 holds)                     | Total work (P4 holds)                       |
| ---------------- | ---------------------------- | ------------------------------------------- | ------------------------------------------- |
| Balanced         | $O(\log \lvert G\rvert_0)$   | $O(\lvert G\rvert_0 \log \lvert G\rvert_0)$ | $O(\lvert G\rvert_0 \log \lvert G\rvert_0)$ |
| Moderate skew    | $O(\sqrt{\lvert G\rvert_0})$ | $O(\lvert G\rvert_0^{3/2})$                 | $O(\lvert G\rvert_0^{3/2})$                 |
| Degenerate chain | $O(\lvert G\rvert_0)$        | $O(\lvert G\rvert_0^2)$                     | $O(\lvert G\rvert_0^2)$                     |

The degenerate chain is constructively tight: a V-Tree where every
entry has distinct importance forms a ladder of depth $|G|_0$.
Contracting tip-inward, the $k$-th PIE triggers a cascade of depth
$|G|_0 - k$, giving $\sum_{k=1}^{|G|_0} k = \Theta(|G|_0^2)$ total
LPs.

> _Note (self-correcting cost)._ Under raw accumulation (no decay),
> the quadratic contraction cost is matched by a _super_-quadratic
> re-expansion cost. Each PIE's absorption folds the evicted
> descendant's accumulated value into the parent, hardening the
> parent's benchmark (§13.5). To re-split back to depth $D$, every
> level $d$ must re-accumulate past its hardened benchmark $B_d$,
> giving a total observation cost of
> $\sum_{d=0}^{D} B_d \sim D^2\theta/2$. The degenerate chain that
> is expensive to contract is therefore _more_ expensive to
> reconstruct — the system demands stronger evidence each cycle
> before re-investing in the same structure. Under user-applied
> decay, the benchmarks attenuate between cycles, so the $D^2$
> formula is an upper bound rather than an asymptotic equivalence;
> the qualitative point — re-expansion costs at least as much as
> contraction — still holds.

> **Corollary (shallow-cascade estimate).** Under typical operation,
> most PIE absorptions create at most one direct violation at the
> parent; cascade effects rarely propagate past $O(1)$ levels (§7.5.2
> tightness remark). Setting cascade depth $= O(1)$, each PIE
> triggers at most $O(1)$ LPs, so LP $\leq$ PIE $\leq |G|_0 - 1$:
>
> $$\text{PIE} + 2\,\text{LP} \leq 3(|G|_0 - 1) = 3|G|_0 - 3$$
>
> The weaker form using the pool bound
> $\text{LP} \leq S_0 + |G|_0 - 1$ (which holds regardless of
> per-PIE cascade depth, provided total LP does not exceed the pool)
> gives:
>
> $$
> \text{PIE} + 2\,\text{LP} \leq (|G|_0 - 1) + 2(S_0 + |G|_0 - 1)
> = 3|G|_0 + 2S_0 - 3
> $$
>
> Applying $S_0 \leq |G|_0 - 1$ (§7.5.5): **Total $\leq 5|G|_0 - 5 \approx
> 5|G|_0$.** The $3|G|_0$ form is tighter; the $5|G|_0$ form is
> a safe universal bound. Neither should be cited as the rigorous
> worst-case bound, which is $O(|G|_0 \cdot h_V)$.

#### 12.9.4 Full Contraction to the Root

Full contraction requires one of:

1. **Progressive $D_{\text{evict}}$ tightening** (§7.4). Under
   sustained memory pressure, the system lowers $D_{\text{evict}}$
   until eventually every non-root entry is eligible. Convergence to
   root in finite time.

2. **Sustained attenuation.** User-applied attenuation ($\text{att} < 1$, §14.3) changes intensities,
   creating violations that reshuffle the V-Tree. Cold entries sink
   past $D_{\text{evict}}$ through competitive demotion rather than
   static depth. Convergence to root under continued attenuation.

3. **Annihilation.** User-applied annihilation ($\text{att} = 0$, §14.3) zeroes all importance values in the targeted subtree. The trailing rebalance pushes zeroed entries to maximum V-depth; `check_evictions()` removes those past $D_{\text{evict}}$. Under full-tree annihilation with sufficiently tight $D_{\text{evict}}$, convergence to root in $O(1)$ tides (one `decay()` call triggers one `check_evictions()` pass per §14.4 Phase 5; subsequent passes from external triggers or observations remove parent layers).

4. **Combined operation.** In practice, multiple mechanisms cooperate: attenuation weakens cold entries while memory pressure tightens thresholds; annihilation provides a fast reset when gradual attenuation is too slow.
   Full contraction is typical under the recommended operating regime
   but not guaranteed under the static $D_{\text{evict}}$ assumption.

The fixed-point guarantee (no eviction-eligible entries remain) is
unconditional. The root-convergence guarantee requires the additional
hypothesis of dynamic $D_{\text{evict}}$, sustained attenuation, or annihilation. $\square$

---

### 12.10 Semi-Internal Chain Overhead

In the full-binary case (every internal G-node has exactly 2 children), $L$
contour cells produce at most $L - 1$ interior nodes — total
$\leq 2L - 1$. Overhead bounded at ${\sim}2\times$.

Semi-internal chains break this bound. When one child is repeatedly evicted
while the other survives and splits deeper, a chain of one-child nodes
forms — a narrow branch extending outward, each node serving only its
single descendant:

| Scenario                        | Contour cells $L$ | Total G-nodes                                                       | Overhead per cell     |
| ------------------------------- | ----------------- | ------------------------------------------------------------------- | --------------------- |
| Balanced, depth $d$             | $2^d$             | $2^{d+1} - 1$                                                       | ${\sim}2\times$       |
| Mixed depths                    | $L$               | $\leq 2L - 1$                                                       | ${\sim}2\times$       |
| Asymmetric eviction             | $L$               | up to $L \cdot \bar{d}$ ($\bar{d}$ = average G-depth of live nodes) | ${\sim}\bar{d}\times$ |
| Worst case: single linear chain | 1                 | $N$                                                                 | ${\sim}N\times$       |

A semi-internal chain is a sequence of G-nodes, each with exactly one child,
extending from some branching ancestor down to a single terminal. Each
interior node in the chain occupies 1 unit of $|G|$ while serving only its
sole descendant. The chain's structural cost lies in forcing the tree to
maintain $k$ ancestors for a single contour cell — each ancestor is a node
that cannot be evicted because it has dependents.

The chain's $k$ interior nodes each contribute 1 to $|G|$. The total
structural cost is $k$ nodes. But each of those nodes forces the G-Tree to
maintain a branch of depth $k$ for a single active tip, preventing the tree
from contracting that branch.

**Dynamic $D_{\text{evict}}$ (§7.4) is the control.** Tightening
$D_{\text{evict}}$ erodes the chain from the tip. Each tide removes the
outermost node. The parent loses its sole child, joins the contour tips,
and is eligible next tide. Benchmark compounding (§13.5) ensures
re-extension costs more each cycle.

**Chain erosion time.** A semi-internal chain of depth $k$ can be fully
contracted by sustained eviction pressure. The time depends on whether
absorption-triggered promotions arrest the erosion:

| Regime                                | Erosion time                                            | Condition                                                                                                                                                                                                                                                                                                                                      |
| ------------------------------------- | ------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Best case                             | $k$ tides                                               | No absorption-triggered promotion arrests erosion. Each tide removes the outermost tip; the next tip is already past $D_{\text{evict}}$.                                                                                                                                                                                                       |
| Under user-applied attenuation        | $O(k \times \text{half-life})$ tides                    | Each successive parent attenuates between tides. Attenuation eventually overcomes absorbed intensity, ensuring the next tip crosses $D_{\text{evict}}$ via competitive demotion. Each level requires $O(\text{half-life})$ tides to weaken enough for demotion past $D_{\text{evict}}$.                                                        |
| Under annihilation                    | 1 tide + $O(k)$ evictions                               | `decay(chain_root, 0, 0)` zeroes all entries. The trailing rebalance pushes them to maximum V-depth. Phase 5’s `check_evictions` removes all unprotected tips. Subsequent tides erode parent layers. Total: $O(k)$ evictions across $O(k)$ tides — same as best case but triggered immediately rather than waiting for gradual cooling.        |
| Under raw accumulation on a hot chain | Unbounded without further $D_{\text{evict}}$ tightening | Absorption strengthens each successive parent. Promotion may push it above $D_{\text{evict}}$, arresting erosion indefinitely. This is correct behavior: the chain persists because the entries are genuinely significant. Full contraction requires either sufficient attenuation, annihilation, or further tightening of $D_{\text{evict}}$. |

> _Note._ A cold chain under raw accumulation (modest historical intensity,
> no active traffic) erodes at near-best-case speed: each successive
> parent's absorbed value is small, promotion above $D_{\text{evict}}$ is
> unlikely, and erosion proceeds at approximately one level per tide. The
> "best case" row effectively covers this scenario.

In all regimes, each individual eviction removes exactly 1 node
(Lemma 12.1). The question is not whether eviction works but how many
tides are needed for the full chain.

**V-Tree cost while the chain persists.** The $k$ semi-internal entries
along the chain are eviction-immune (each has a surviving G-child). In the
worst case, the $k$ chain entries form a chain of depth $k$ in the V-Tree —
each entry colder than the last. The Fibonacci bound then constrains $k$
via the coldest entry's weight fraction:

$$k \leq \log_\phi\!\left(\frac{1}{w_{\text{base}}}\right) + c$$

where $w_{\text{base}}$ is the weight fraction of the coldest (base) entry.
For proportional sampling, any entry whose V-Tree path passes through the
chain pays at most $k$ additional steps. The total probability mass of such
entries is $w_{\text{subtree}}$ — the chain's V-subtree weight fraction,
which includes both the chain entries themselves and everything below them
in the V-Tree. The total sampling cost inflation is therefore at most:

$$\Delta E[\text{cost}] \leq k \cdot w_{\text{subtree}}$$

The chain entries themselves are cold (low $w_i$). Entries below the chain
that happen to root in the same V-subtree are typically also cold (having
lost the competitive tournament to reach this neighbourhood), so the
typical case has $w_{\text{subtree}} \ll 1$ and the inflation is
negligible despite the $O(k)$ depth. Note that $w_{\text{subtree}}$ is
the true governing quantity — since V-Tree parentage is independent of
G-Tree parentage, an adversarial configuration could place a hot entry
in the chain's V-subtree, making $w_{\text{subtree}}$ non-negligible.
The bound is correct regardless; only the "typically small" qualifier
is a dynamic claim. The penalty dissolves as the chain is eroded: each
eviction removes the tip's V-entry, freeing one level of V-Tree depth.

> _Note (inverse ordering)._ Along a semi-internal chain, G-depth and
> V-depth typically exhibit an inverse relationship under sustained uniform
> traffic to the region. The most recently frozen entry (at the tip, deepest
> in the G-Tree) typically has the highest intensity — it accumulated the
> most observations before freezing — and therefore sits shallowest in the
> V-Tree. Conversely, entries frozen earliest (shallowest in the G-Tree,
> closest to the chain's base) have had their intensity diluted by subsequent
> splits and sit deepest in the V-Tree. This inverse ordering means the tip
> is the last chain entry to cross $D_{\text{evict}}$, and the base entries
> cross first — but they are eviction-immune (they have dependents). The
> chain can only erode from the tip inward, which is exactly the structural
> constraint (§12.9): eviction requires zero G-children. (Heterogeneous
> traffic patterns can produce exceptions where a mid-chain entry outweighs
> the tip; such configurations still obey the tip-only eviction constraint
> but may alter the V-Tree depth ordering.)

> _Cross-reference._ Path compression (§5.7) eliminates the per-node
> materialization cost of semi-internal chains. A chain of $k$ one-child
> nodes would be represented as a single compressed edge with explicit
> range annotation, reducing the materialized cost from $k$ nodes to
> $O(1)$. The overhead analysis in this section applies to the abstract
> tree; implementations using path compression do not pay the $k$-node
> penalty.

---

## Chapter 13. Governance: Dual Shielding

This chapter is the formal treatment of the governance mechanisms introduced in §1.5. Where §1.5 gives an intuitive overview, this chapter provides the precise definitions, state-machine semantics, and invariant proofs.

The section is organised in three parts. §§13.1–13.2 define **the protection mechanisms**: the three shields and the concrete state machine they produce. §§13.3–13.4 describe **the operational dynamics**: how observations flow through the dual-tree structure and what each tree contributes. §13.5 establishes **long-term behaviour**: how repeated expand–contract cycles harden the system's admission criteria.

---

### 13.1 The Three Shields

The two trees govern each other through three complementary protection mechanisms.

| Direction           | Shield                                                      | What it protects                                                                                                 | Mechanism            |
| ------------------- | ----------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- | -------------------- |
| V-Tree → downward   | Parent's entry is uncle to children's entries               | Children from competitive reshuffling — they cannot be displaced as long as the parent's entry is a strong uncle | Max-uncle constraint |
| G-Tree → upward     | Children intercept observations meant for parent's range    | Parent's V-entry from growing — it stays frozen as a fixed benchmark children can eventually surpass             | Observation routing  |
| G-Tree → structural | Only unprotected G-nodes (0 children) are eviction-eligible | Protected G-nodes from premature removal — they cannot be evicted while structurally load-bearing                | Dependents check     |

The root exemption (§12.3, §12.5) is not a fourth shield — it is a policy override that prevents the G-root from being destroyed regardless of eligibility. The three shields are mutual protection _mechanisms_; the root exemption is a _boundary condition_ that guarantees V-I0 (the V-Tree always contains at least one entry).

**Without the upward shield (routing interception):** The parent's V-entry would grow with every observation to its range, keeping pace with its children. The frozen benchmark would not exist. Children could never outgrow their uncle. The competitive mechanism would be dead.

**Without the downward shield (uncle constraint):** Children's positions would be unstable. Every intensity fluctuation would cause restructuring. The V-Tree would thrash rather than settle into a stable tournament.

**Without the structural shield (unprotected-only eviction):** Evicting an entry could destroy a G-node that supports other contour cells. The contour would tear — creating orphaned cells with no backing G-node. Instead, only unprotected G-nodes (zero children, no dependents) can be evicted. The contour contracts smoothly from the tips inward.

The three shields breathe in counterpoint — a rhythm made concrete by the G-node lifecycle that follows.

---

### 13.2 The G-Node Lifecycle

A G-node breathes through a full cycle. The three shields govern every transition.

```
    ON THE CONTOUR — fully exposed (no dependents)
        │                                                   ┄┄┄► DESTROYED
        │ catalytic split → 2 children created              (evicted: V-deep,
        ▼                                                    unprotected, §11.6)
    ABOVE THE CONTOUR (not exposed, has dependents, frozen benchmark)
        │                              ↑
        │ one child evicted            │
        │ (absorb, partially exposed)  │
        ▼                              │
    ON THE CONTOUR — partially exposed (has one dependent)
        │                              │
        ├─── other child evicted ──► ON THE CONTOUR — fully exposed
        │         (absorb,                 (no dependents, cycle restarts
        │          no dependents)           — or evicted if V-deep)
        │
        └─── legacy promotion ─────► ABOVE THE CONTOUR
               (entry outgrew              (not exposed, has dependents,
                all uncles)                 frozen benchmark, cycle continues)
```

The dashed exit arrow represents eviction: a terminal node past $D_{\text{evict}}$ with no dependents is destroyed. Eviction is a terminal event — the G-node and its V-entry are deallocated. The lifecycle is cyclic for nodes that remain competitively significant; eviction is the exit for nodes that do not.

**Compound transitions within a single tide.** The state machine correctly requires passage through each intermediate state, but transitions can compound within a single `check_evictions` pass. If both siblings of an internal node are evicted in the same Phase 2 batch (§12.6 worked example: "both siblings evicted"), the parent transitions Internal → Semi-internal → Terminal within one tide. Each individual transition is atomic and invariant-preserving; the compounding is a consequence of the batch processing model, not a violation of the sequential state machine.

**When children are created** (by refinement): the parent leaves the contour. The upward shield activates — children intercept all observations in the parent's range. The parent's V-entry freezes at its pre-split value. Children join the contour at zero importance and must earn their way up against a fixed bar. The structural shield activates — the parent now has dependents and cannot be evicted.

**When a child is evicted (§12.5, §12.6):** the parent absorbs that child's accumulated value, partially re-joins the contour for the vacated range, and starts receiving observations there again. Its V-entry strengthens — which strengthens the downward shield protecting the surviving child with a heavier uncle.

**When the parent earns legacy promotion:** the parent's V-entry outgrew all uncles through competitive accumulation. Legacy promotion (§11.6) bequeaths the vacated V-seat to a newly created G-node for the missing half. The parent goes back above the contour. The upward shield reactivates — both children now intercept all traffic. The entry freezes as a benchmark. The cycle continues.

**When both children are evicted (§12.8):** the parent absorbs both values, fully re-joins the contour, and has no dependents. It can refine again — or be evicted itself if it sits past $D_{\text{evict}}$.

The key arrow: a partially exposed node can create its missing child **through promotion alone** — no separate operation, no separate threshold. The V-Tree's competitive mechanism is the sole gate. The contour grows through competition.

---

### 13.3 The Governance Flow

```
Observation arrives at coordinate x               [G-Tree routes to contour cell]
    → receiver.own grows, receiver.entry grows     [single V-entry update]
    → V-I3 may be violated                         [cell outgrew frozen uncle]
    → rebalance: promotion                         [V-Tree restructures]
        → if semi-internal: legacy promote         [contour regrows]
        → else: standard or skip promote           [V-Tree only]

V-Tree identifies eviction candidate               [V-Tree ranks; G-Tree confirms]
    (too deep + unprotected + not root)
    → unprotected entry evicted                    [atomic removal]
    → backing G-node destroyed                     [contour coarsens]
    → parent absorbs value, may join contour       [absorption strengthens parent]
    → sibling's uncle shield strengthens           [downward shield]
    → trailing rebalance may trigger legacy promote [contour may regrow immediately]
    → parent may lose last dependent               [next-round candidate]
    → parent may earn legacy promotion later       [contour may regrow in future tide]
```

The eviction path involves all three structures: the V-Tree determines competitive insignificance (depth past $D_{\text{evict}}$), the G-Tree confirms structural eligibility (no dependents), and the G-Tree's spatial structure determines which node absorbs the evicted value and how the contour reshapes. Neither tree makes decisions alone — eviction requires the V-Tree's ranking, the G-Tree's structural attestation, and the policy override (root exemption) to agree before a node is destroyed.

The flow has a natural rhythm. Observations inject energy. The V-Tree's competitive mechanism routes that energy toward structural decisions — refinement for strong entries, eviction for weak ones, legacy promotion for entries that proved themselves in a partially exposed state. The G-Tree executes every structural decision immediately.

---

### 13.4 The Complementary Questions

The two rankings answer dual questions about the same underlying signal:

|                     | G-Tree (by $g.\text{sum}$)                                            | V-Tree (by $g.\text{importance}$)                                                    |
| ------------------- | --------------------------------------------------------------------- | ------------------------------------------------------------------------------------ |
| **Question**        | Where is activity concentrated **spatially**?                         | Where is importance concentrated — **and at what scale**?                            |
| **Direction**       | Top-down: coarse regions dominate because they _contain_ fine regions | Bottom-up: fine regions dominate because they _replaced_ coarse regions              |
| **Parent vs child** | Parent ≥ child always                                                 | Child > parent eventually                                                            |
| **Root**            | Maximum                                                               | Minimum among frozen entries (frozen earliest, at the most ancient value; see note)  |
| **Leaves**          | Minimum                                                               | Maximum among active entries (receiving observations, accumulating fresh importance) |

> _Note (Root as eventual minimum)._ The root's entry is frozen at the bootstrap split — the earliest freeze in the tree's history. Under sustained observation, every descendant that earns competitive promotion accumulates past the root's frozen value. The root becomes the minimum among frozen entries because it was frozen first with the least accumulated importance. This trajectory requires sustained observation: in a quiescent tree (no new observations), frozen entries retain their split-time values indefinitely and the root may not be the minimum if a descendant was frozen with less accumulated importance. Under user-applied decay (§14), all frozen entries attenuate, preserving relative ordering but shrinking absolute values.

> _Note (Regime dependence of the V-Tree's question)._ Under user-applied attenuation ($\text{att} < 1$, §14.3), the V-Tree's ranking reflects recency — "what matters now." Under user-applied amplification ($\text{att} > 1$) with depth selectivity ($q > 0$), the ranking reflects _reinforced_ fine-scale significance — entries at deeper G-depths grow faster, sharpening the importance landscape. Under annihilation ($\text{att} = 0$), the ranking in the targeted region collapses to minimum — all entries are equally insignificant until fresh observations re-seed competition. Under raw accumulation (the default), the ranking reflects scale-specific historical significance — "what mattered most at each level of refinement." The framing "where is importance concentrated, and at what scale" is regime-independent; the temporal character of the concentration depends on the user's choice of filter.

**G-Tree alone:** spatial queries work, but governance decisions (refine, evict) require a global ranking by current significance. The G-Tree's containment order is the wrong ranking for resource allocation — every parent outranks every child, regardless of which child is hottest.

**V-Tree alone:** governance and proportional sampling work, but "how important is this general area" requires aggregating scattered entries across an unrelated tournament structure. The V-Tree has no spatial awareness.

**Together:** the V-Tree governs and the G-Tree accounts. The V-Tree says where attention is flowing _now_ (under decay) or where it flowed _most intensely at each scale_ (under raw accumulation). The G-Tree says where attention has flowed _in total_. One is the rate. The other is the accumulated quantity. The derivative and the integral of the same underlying signal.

**Two concrete projections** make these dual questions queryable. The G-Tree projects to the plateau ordered map (§5.6.7): a spatially-ordered step function of thatched energy — the graph's learned shape. The V-Tree projects to the PEWEI (Progressive Entropic-Wavelet Exposure Image, companion document §§PEWEI M-1–13): a significance-ordered layer sequence — the graph's learned ranking. Neither subsumes the other; they are orthogonal views of the same dual-tree state.

---

### 13.5 Benchmark Compounding

Repeated expand–contract cycles at a given G-node harden its benchmark. Each absorption folds descendants' accumulated value into the node's own, raising the frozen bar that future children must exceed.

**First cycle.** The node accumulates $\theta$ observations, splits. Benchmark frozen at $\theta$. Children start at 0 and accumulate fresh observations.

**Children evicted, node absorbs.** If each child accumulated ${\sim}\theta$ before eviction, the node's own value is now ${\sim}3\theta$.

**Second cycle.** The node re-splits trivially (own $= 3\theta \gg \theta$, shallow V-depth). But its benchmark is now $3\theta$. Children face uncle $3\theta$ — three times the original bar.

**The pattern.** After $k$ full expand–contract cycles, the benchmark at depth $d$ grows approximately as:

$$B_d^{(k)} \;\approx\; B_d^{(k-1)} \cdot \alpha_k$$

where $\alpha_k > 1$ depends on how thoroughly descendants accumulated during cycle $k$ before contraction. Under a steady-state assumption (each cycle's descendants accumulate comparably), $\alpha_k \approx \alpha$ for all $k$ and the benchmark grows geometrically: $B_d^{(k)} \approx B_d^{(0)} \cdot \alpha^k$. In practice, $\alpha_k$ varies per cycle — early cycles with sparse observation may produce $\alpha_k$ close to 1, while later cycles with concentrated traffic produce larger $\alpha_k$. The qualitative trajectory (monotonic hardening under non-negative observations) is unconditional; the geometric rate is a steady-state idealisation.

**Per-path re-expansion cost.** To re-reach depth $D$ along a single root-to-leaf path after a full contraction, each of the $D$ levels must re-accumulate past its hardened benchmark. After one contraction cycle (where each level absorbed approximately $\theta$ from its single-level descendants), the benchmark at depth $d$ is approximately $(d + 1)\theta$. The total observation cost along the path is:

$$\sum_{d=0}^{D} B_d \;\sim\; \sum_{d=0}^{D} (d+1)\theta \;=\; \frac{(D+1)(D+2)}{2}\,\theta \;\sim\; \frac{D^2 \cdot \theta}{2}$$

This is the cost for a single root-to-leaf path. Re-expanding the entire subtree requires re-splitting at every node on every path — the total cost across all $2^D$ potential leaf paths is the sum of per-path costs, dominated by the deepest levels. The quadratic growth per path is the key result: each additional level of depth costs linearly more, not a constant increment.

After $k$ cycles, benchmarks grow as $B_d^{(k)} \approx (d+1)\theta \cdot \alpha^{k-1}$, making the per-path sum super-quadratic in $D$ for $k > 1$. The system demands exponentially more evidence in successive cycles before re-investing in the same structure.

**Interaction with temporal scaling.** The built-in temporal filter (§14.3) modulates benchmark compounding in three qualitatively distinct ways:

**Under attenuation** ($\text{att} < 1$): benchmarks attenuate between cycles. A benchmark $B$ decays to $\lambda^t B$ after $t$ scaling steps, weakening the bar that future children must clear. The compounding trajectory is modulated: $B_d^{(k)} \approx B_d^{(k-1)} \cdot \alpha_k \cdot \lambda^{t_k}$ where $t_k$ is the number of scaling steps during cycle $k$. When $\alpha_k \cdot \lambda^{t_k} > 1$, the benchmark still hardens despite attenuation (accumulation outpaces scaling). When $\alpha_k \cdot \lambda^{t_k} < 1$, the benchmark softens — the region becomes easier to re-expand, reflecting the system's judgment that the historical evidence has aged. The $D^2\theta/2$ formula is an upper bound under attenuation; the qualitative point — re-expansion costs at least as much as the re-accumulation needed to exceed attenuated benchmarks — still holds.

**Under amplification** ($\text{att} > 1$): benchmarks _strengthen_ between cycles. The frozen bar $B$ grows to $\lambda^t B$ with $\lambda > 1$, raising the threshold future children must exceed. Under uniform amplification ($q = 0$), children's importance grows at the same rate as the benchmark — the relative difficulty of promotion is unchanged. Under depth-selective amplification ($q > 0$), fine-scale entries (at deeper G-depth) amplify faster than coarse benchmarks (at shallower G-depth), making promotion _easier_ — the inverse of depth-selective attenuation's smoothing effect. The compounding formula $B_d^{(k)} \approx B_d^{(k-1)} \cdot \alpha_k \cdot \lambda^{t_k}$ still applies, with $\lambda > 1$ producing monotonically hardening benchmarks that do not soften. Repeated amplification without compensating attenuation makes the benchmark sequence divergent — values grow without bound (§14.2, representability note).

**Under annihilation** ($\text{att} = 0$): benchmarks are _reset_. $B \to 0$. The compounding history is erased. The region starts from scratch — future children face the split threshold $\theta$, not a hardened benchmark. This is the escape hatch from runaway compounding: a region that has undergone many expand–contract cycles with monotonically hardening benchmarks can be reset to its initial difficulty by a single annihilation call. Under detail flush ($\text{att} = 0$, $q = 1$), the subtree root's benchmark is _preserved_ while descendants are zeroed — the coarsest-scale measurement persists as a datum while fine-scale structure must be re-earned.

**This is the correct resource allocation.** A cell at depth $D$ requires $D$ ancestor nodes in the abstract tree (path compression §5.7 reduces the materialised count but not the logical depth). The quadratic-or-greater cost to reach it ensures this overhead is incurred only when justified by sustained, concentrated activity. The compounding is the system's long-term memory: it remembers that a region was previously explored and found wanting, and demands stronger evidence before re-investing.

**Self-regulation (updated).** Over-expansion → many unprotected entries at deep V-positions → temporal scaling → benchmarks weaken (attenuation), strengthen (amplification), or reset (annihilation) → future refinements adjusted accordingly. Under attenuation, the cycle tightens its own admission criteria with decreasing severity over time, allowing re-expansion when fresh evidence warrants it. Under amplification, the cycle tightens with _increasing_ severity — only strongly persistent structure survives. Under annihilation, the cycle is interrupted and restarted from the initial threshold.

---

## Chapter 14. Temporal Semantics

The architecture stores exact accumulated values by default. It imposes no
temporal model, runs no automatic decay, and dictates no filter. A user who
wants "what matters now" applies a temporal filter. A user who wants "most
significant ever" does nothing. The architecture adapts to whatever the
numbers say.

This chapter defines the **temporal filter contract** — the invariants any
filter must restore after modifying values (§14.1) — and provides one
**built-in implementation**: the three-parameter log-linear decay function
(§14.3) with its algorithm (§14.4). The built-in function supports
attenuation ($\text{att} < 1$), amplification ($\text{att} > 1$), and
annihilation ($\text{att} = 0$), enabling smoothing, sharpening, selective
boosting, and hard resets of the importance landscape. The contract is the
architecture's concern. The choice of filter — including the choice to
apply none — is the user's concern. The built-in decay is a provided
default with proven composability properties, not a mandatory component.

### 14.1 Temporal Filter Contract

A **temporal filter** is any operation that modifies the importance and
ledger accumulators of one or more G-nodes. The architecture requires that
after the filter completes, the following invariants have been restored:

| Invariant             | Restoration mechanism                                                                                                                                                                |
| --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| G-I1 (Summation)      | Recompute `g.sum` bottom-up from modified `g.own` values and children's sums                                                                                                         |
| V-I1 (V-Sums)         | Propagate V-sums from each affected entry to the V-root                                                                                                                              |
| V-I3 (Uncle)          | Detect violations, call `rebalance()`                                                                                                                                                |
| G-I4 (Importance Ref) | Modify importance accumulators in place — do not replace the referenced object. The V-entry holds a reference to `g.importance`; replacing the object breaks the reference silently. |
| V-I7 (Flags)          | If eviction follows, `check_evictions()` updates flags                                                                                                                               |
| Plateau map           | Recompute affected plateau sums and basis edges                                                                                                                                      |

V-I6 (`is_exposed`) and V-I6b (`is_evictable`) are unaffected by value
scaling — the filter does not add or remove G-children. More broadly,
`modulate` (§8.8.2) is a pure value operation: it preserves the entire
G-Tree topology, contour structure, and plateau map. No G-node is created,
destroyed, or reparented. Even under annihilation ($\lambda = 0$), all
nodes persist with zeroed accumulators — the contour still tiles the full
domain. If the filter triggers eviction (Phase 5 of §14.4), node removal
is performed by the eviction machinery, not by the filter itself; the
eviction machinery maintains these flags as specified in §12.5.

A filter that modifies values but does not restore these invariants leaves
the structure in a corrupt state. The filter need not restore invariants
incrementally at each node — it may modify all values first and then
restore invariants in a single batch pass. The sequential model (§17.3)
requires that no external operation (query, sample, observation) occur
between the start of the filter and the completion of invariant
restoration.

The architecture provides the incremental machinery — `propagate_v_sums`,
`is_violated`, `rebalance`, `check_evictions`, plateau map updates — as
composable building blocks. The filter is responsible for invoking them
in the correct order. §14.4 specifies the complete invocation sequence
for the built-in decay; other filters must follow an equivalent sequence.

> _Note (G-I4 and absolute projection)._ Under the standard configuration
> (identity projection), `g.own` and `g.importance` are the same
> accumulator (§8.6.2); a single in-place scaling suffices. Under absolute
> projection, they are independent fields — both must be scaled in place.
> A filter that creates a new importance object and assigns it to
> `g.importance` rather than mutating the existing accumulator would
> break G-I4: the V-entry's reference would point to the old (stale)
> object. The built-in decay avoids this by calling `modulate` on the
> existing accumulator.

### 14.2 Uniform Exponential Decay

The simplest temporal filter multiplies all accumulators by a constant
factor $\lambda \geq 0$:

- When $\lambda = 0$, values are annihilated (hard reset to zero).
- When $\lambda \in (0, 1)$, values attenuate (decay).
- When $\lambda = 1$, the operation is an identity.
- When $\lambda > 1$, values amplify.

$$g.\text{own} \leftarrow \text{modulate}(g.\text{own},\, \lambda)$$
$$g.\text{importance} \leftarrow \text{modulate}(g.\text{importance},\, \lambda)$$

for every G-node $g$ in the target subtree. The `modulate` operation is
the Temporal scaling capability of §8.8.2 — defined as multiplicative scaling
by a factor, which encompasses annihilation, attenuation, and amplification.
`modulate` is a pure value operation: it scales accumulators but does not
add, remove, or restructure any G-node. After modulation — even at
$\lambda = 0$ — every G-node, contour cell, and plateau boundary remains
in place. The G-Tree's spatial topology is invariant under modulation.
Node removal, if it occurs, is a downstream consequence of eviction
(§12.5), which may collect nodes whose zeroed importance places them past
$D_{\text{evict}}$ in V-depth.
For $\lambda = 0$, `modulate(v, 0)` must produce the
zero element of the type ($v \times 0 = \text{zero}_T$); this is a
natural requirement for any type that supports multiplicative scaling.

**G-sum restoration.** After scaling `g.own` at every node, `g.sum` must be
recomputed bottom-up (post-order traversal):

$$g.\text{sum} \leftarrow g.\text{own} + \sum_{c\, \in\, \text{children}(g)} c.\text{sum}$$

Direct scaling of `g.sum` by $\lambda$ — rather than bottom-up
recomputation — gives the correct result for floating-point types under
uniform $\lambda$ (scaling distributes over addition in exact arithmetic,
and the floating-point error is negligible). For integer types, it does
not: $\lfloor(a + b) \cdot \lambda\rfloor \neq \lfloor a \cdot
\lambda\rfloor + \lfloor b \cdot \lambda\rfloor$ in general, so direct
scaling of `sum` violates G-I1. For $\lambda = 0$ specifically, direct
scaling produces the correct result for both types ($0 = 0$), but
**bottom-up recomputation is always correct and is the normative
approach.**

**V-I3 preservation under uniform scaling (floating-point types).** If
$c.\text{int} \leq \max\{u.\text{int}\}$ before scaling, then for
$\lambda > 0$:

$$\lambda\, c.\text{int} \;\leq\; \lambda\, \max\{u.\text{int}\} \;=\; \max\{\lambda\, u.\text{int}\}$$

since $\lambda > 0$ preserves the `Ord` relationship. For $\lambda = 0$:

$$0 \cdot c.\text{int} = 0 \;\leq\; 0 = 0 \cdot \max\{u.\text{int}\}$$

which is trivially satisfied — all values collapse to zero, and $0 \leq 0$.

In both cases, after bottom-up V-sum recomputation, every structural node's
importance is exactly $\lambda$ times its pre-scaling value (by induction:
entry importances scale by $\lambda$; structural nodes' sums are computed
from scaled children). V-Tree shape unchanged. V-I3 preserved. **No
rebalancing is required for floating-point types under uniform scaling**,
modulo negligible floating-point rounding. This holds for annihilation
($\lambda = 0$), attenuation ($\lambda < 1$), and amplification
($\lambda > 1$).

> _This argument requires the same $\lambda$ for every node. The
> depth-selective decay of §14.3 ($q > 0$) applies different factors
> to different G-depths and does not preserve V-I3 in general; see
> §14.3._

**V-I3 under uniform scaling (integer types).** Per-entry scaling preserves
ordering: $a \geq b \implies \lfloor a \cdot \lambda \rfloor \geq
\lfloor b \cdot \lambda \rfloor$ (floor is monotone). However, V-I3
compares **structural aggregates**, not individual entries. After integer
truncation, a structural node's recomputed importance $\sum \lfloor
c_i.\text{int} \cdot \lambda \rfloor$ may differ from
$\lfloor (\sum c_i.\text{int}) \cdot \lambda \rfloor$. Two structural
siblings with equal pre-scaling importance may have different post-scaling
importances due to different truncation patterns in their subtrees.

> _Example (attenuation)._ Uncle (structural): 2 entries with importance
> 5 and 4. Aggregate $= 9$. Nephew (structural): 3 entries with importance
> 3, 3, 3. Aggregate $= 9$. V-I3 satisfied: $9 \leq 9$.
> After scaling by $\lambda = 0.7$: Uncle $= \lfloor 3.5 \rfloor +
> \lfloor 2.8 \rfloor = 3 + 2 = 5$. Nephew $= 3 \times \lfloor 2.1
> \rfloor = 3 \times 2 = 6$. Nephew $6 > 5$ uncle. **V-I3 violated.**

> _Example (amplification)._ Uncle (structural): 2 entries with importance
> 10 and 11. Aggregate $= 21$. Nephew (structural): 3 entries with
> importance 7, 7, 7. Aggregate $= 21$. V-I3 satisfied: $21 \leq 21$.
> After scaling by $\lambda = 10/7 \approx 1.43$: Uncle
> $= \lfloor 14.3 \rfloor + \lfloor 15.7 \rfloor = 14 + 15 = 29$.
> Nephew $= 3 \times \lfloor 10.0 \rfloor = 30$. Nephew $30 > 29$ uncle.
> **V-I3 violated.**

For $\lambda = 0$ with integer types: $\lfloor v \cdot 0 \rfloor = 0$ for
all $v$. All entries and aggregates become zero. $0 \leq 0$ everywhere.
**V-I3 trivially preserved.** The truncation issue does not arise at
$\lambda = 0$ — there is nothing to truncate.

**Integer types require trailing violation detection and rebalance for
$\lambda \in (0, 1) \cup (1, \infty)$.** At $\lambda = 0$ and $\lambda = 1$,
no violations can arise (annihilation or identity). The violations for other
$\lambda$ values are infrequent (they arise only when truncation patterns
differ across sibling subtrees) but cannot be statically excluded.

**P1 (bounded below) is preserved** when $\lambda \geq 0$: the bottom
element (typically 0) scales to itself ($\lambda \cdot 0 = 0$), remaining
the minimum of the scaled values. This holds for annihilation, attenuation,
and amplification.

**Sampling after annihilation.** Under full-tree uniform $\lambda = 0$, all
entry importances become zero (the ground element when P2 holds). The
V-root's aggregate equals the ground, and sampling (§6.5) returns $\perp$
— the honest answer that zero total importance means no distribution to
sample from. Sampling becomes defined again after the first
post-annihilation observation. Under subtree annihilation, entries outside the targeted
subtree retain their importance; sampling draws exclusively from them until
the annihilated region re-accumulates.

**Representability.** Under amplification ($\lambda > 1$), repeated
application causes geometric growth. For integer types, values eventually
overflow. For floating-point types, values eventually reach infinity. The
architecture does not guard against overflow from amplification — the user
is responsible for keeping importance values within the representable range
of the concrete type. This is symmetric with the non-guard on underflow
from attenuation (floating-point values approaching zero but never reaching
it; integer values truncating to zero, which is well-defined). Annihilation
($\lambda = 0$) produces no representability concerns — zero is always
representable.

### 14.3 Three-Parameter Built-In Decay

The built-in decay function extends uniform exponential decay with
depth-selective scaling. It is fully characterised by three parameters:

| Parameter    | Symbol       | Constraint          | Role                                      |
| :----------- | :----------- | :------------------ | :---------------------------------------- |
| Root         | `root`       |                     | Which G-subtree to scale (spatial target) |
| Scale factor | $\text{att}$ | $\text{att} \geq 0$ | Base scaling factor at the midpoint depth |
| Selectivity  | $q$          | $q \in [0, 1]$      | Depth-selectivity                         |

**Four regimes.** The $\text{att}$ parameter determines the direction of
temporal scaling:

| $\text{att}$ range     | Effect                                          | Use case                                |
| :--------------------- | :---------------------------------------------- | :-------------------------------------- |
| $\text{att} = 0$       | Annihilation — values reset to zero             | Hard reset, detail flush (with $q = 1$) |
| $\text{att} \in (0,1)$ | Attenuation — values shrink                     | Temporal decay: "forget old data"       |
| $\text{att} = 1$       | Identity — no effect (regardless of $q$; §14.4) | No-op; useful as a sentinel             |
| $\text{att} > 1$       | Amplification — values grow                     | Temporal boost, selective sharpening    |

The per-depth scaling factor within the target subtree is:

$$\lambda(d) \;=\; \text{att}^{\,1 \,+\, q\, \cdot\, (2\,d_{\text{local}}\,/\,D \;-\; 1)}$$

where $d_{\text{local}} = d_{\text{global}} - d_{\text{root}}$ is the depth
relative to the subtree root and $D = N - d_{\text{root}}$ is the local
depth range. The exponent is log-linear in local depth:

| Depth within subtree                | Factor             |
| :---------------------------------- | :----------------- |
| Root ($d_{\text{local}} = 0$)       | $\text{att}^{1-q}$ |
| Midpoint ($d_{\text{local}} = D/2$) | $\text{att}$       |
| Deepest ($d_{\text{local}} = D$)    | $\text{att}^{1+q}$ |

At $q = 0$ all subbands scale at the same rate (uniform exponential
scaling, §14.2). At $q > 0$, coarse and fine subbands scale at different
rates. At $q = 1$, the subtree root is unscaled ($\text{att}^0 = 1$) and
the deepest terminals scale at $\text{att}^2$.

**The $0^0$ convention.** When $\text{att} = 0$ and $q = 1$, the subtree
root's exponent is $1 - q = 0$, producing $0^0$. The specification adopts
the standard convention $0^0 = 1$ — the subtree root is preserved while
all descendants are annihilated. This convention is consistent with IEEE
754 (`pow(0.0, 0.0) = 1.0`) and with the mathematical limit
$\lim_{x \to 0^+} x^0 = 1$. Implementations that cannot guarantee this
behaviour from their power function must special-case the exponent-zero
branch (see §14.4, `scale_subtree`).

For $\text{att} = 0$ with exponent $> 0$: $0^{(\text{positive})} = 0$ —
the node is annihilated. The exponent $1 + q \cdot (2d_{\text{local}}/D -
1)$ equals zero only when $q = 1$ and $d_{\text{local}} = 0$.
Consequently:

| $\text{att}$ | $q$      | Root factor | All other depths | Character                                             |
| :----------- | :------- | :---------- | :--------------- | :---------------------------------------------------- |
| $0$          | $0$      | $0$         | $0$              | **Hard reset** — entire subtree zeroed                |
| $0$          | $(0, 1)$ | $0$         | $0$              | **Hard reset** — entire subtree zeroed                |
| $0$          | $1$      | $1$         | $0$              | **Detail flush** — root preserved, descendants zeroed |

The detail flush ($\text{att} = 0$, $q = 1$) is a structurally significant
operation: it preserves the subtree root's coarse-scale energy ($g.\text{own}$
and $g.\text{importance}$ unchanged) while zeroing all confirmed fine-scale
structure. The tree retains its spatial topology — all G-nodes, children,
and V-entries persist — but every descendant's accumulator is reset. Future
observations must re-earn every refinement from scratch against the
preserved coarse baseline. Combined with subsequent eviction (Phase 5 of
§14.4), detail flush followed by tidal contraction (§12.7) progressively
strips the subtree back to its root.

**Depth-selective regimes.** The combination of $\text{att}$ and $q$
produces qualitatively different operations:

| $\text{att}$ | $q$ | Root factor  | Deepest factor | Character                                            |
| :----------- | :-- | :----------- | :------------- | :--------------------------------------------------- |
| $0$          | $0$ | $0$          | $0$            | Hard reset                                           |
| $0$          | $1$ | $1$          | $0$            | Detail flush — root preserved, descendants zeroed    |
| $< 1$        | $0$ | $\text{att}$ | $\text{att}$   | Uniform decay                                        |
| $< 1$        | $1$ | $1$          | $\text{att}^2$ | Smoothing — fine detail fades, coarse persists       |
| $> 1$        | $0$ | $\text{att}$ | $\text{att}$   | Uniform boost                                        |
| $> 1$        | $1$ | $1$          | $\text{att}^2$ | Sharpening — fine detail amplified, coarse untouched |

**No mixing within a single call.** The exponent $1 + q \cdot
(2d_{\text{local}}/D - 1)$ ranges over $[1 - q,\; 1 + q] \subseteq [0, 2]$
for $q \in [0, 1]$. Since the exponent is always non-negative:

- $\text{att} = 0$: all $\lambda(d) \in \{0, 1\}$ — annihilation (with
  at most one preserved node when $q = 1$).
- $\text{att} \in (0, 1)$: all $\lambda(d) = \text{att}^{(\text{non-negative})}
  \in (0, 1]$ — pure attenuation across the subtree.
- $\text{att} > 1$: all $\lambda(d) = \text{att}^{(\text{non-negative})}
  \in [1, \infty)$ — pure amplification across the subtree.

No node is attenuated while another is amplified within the same call. This
separation enables the direction-dependent violation analysis in the
invariant restoration summary below.

When the target subtree is a single node ($D = 0$), the depth-selectivity
term is undefined. The convention is $\lambda = \text{att}$ (the midpoint
rate) — uniform scaling of the sole node. For $\text{att} = 0$, $D = 0$:
the sole node is annihilated.

**Q-invariance.** Composing $k$ decay ticks with parameters
$(\text{att}, q)$ is equivalent to a single tick with parameters
$(\text{att}^k, q)$:

$$\lambda(d)^k \;=\; \text{att}^{\,k\,(1 + q\,(2d/D - 1))} \;=\; (\text{att}^k)^{\,1 + q\,(2d/D - 1)}$$

The selectivity $q$ is invariant under repetition — it is a structural
property of the filter, not a magnitude. The filter bank therefore
commutes with time discretisation. This composability property is unique to
the log-linear profile; it fails for bell curves, step functions, or
arbitrary user closures.

Q-invariance holds across all regimes. Composing $k$ attenuation ticks at
$\text{att} < 1$ equals one tick at $\text{att}^k$ (stronger attenuation).
Composing $k$ amplification ticks at $\text{att} > 1$ equals one tick at
$\text{att}^k$ (stronger amplification). Composing an attenuation tick at
$a < 1$ with an amplification tick at $b > 1$ equals one tick at $a \cdot
b$, which attenuates if $ab < 1$ and amplifies if $ab > 1$. For
$\text{att} = 0$: $0^k = 0$ for $k \geq 1$, so composing any number of
annihilation ticks equals a single annihilation — idempotent. Composing
annihilation ($\text{att}_1 = 0$) followed by any operation
($\text{att}_2$) at the same $q$: $(\text{att}_1 \cdot \text{att}_2)^{1 +
q(\cdots)} = 0$ — annihilation absorbs subsequent operations (the zeroed
accumulator has nothing to scale).

Q-invariance holds for the _mathematical_ function. For integer types,
$k$ rounds of truncation produce different results than a single round at
$\text{att}^k$: $\lfloor \lfloor x \cdot \lambda \rfloor \cdot \lambda
\rfloor \neq \lfloor x \cdot \lambda^2 \rfloor$ in general. The
composability is exact for floating-point types and approximate for
integer types. At $\lambda = 0$, composability is exact for both types:
$\lfloor v \cdot 0 \rfloor = 0$ and $\lfloor 0 \cdot \lambda \rfloor = 0$
regardless of the second factor.

**Subtree targeting.** Because the G-Tree organises nodes by spatial
containment, decay can be targeted to any G-subtree. The walk visits every
G-node in the subtree and modifies its `own`, `importance`, and `sum`
fields. The corresponding V-entries — which may be scattered throughout the
V-Tree, since V-Tree parentage is independent of G-Tree parentage — see
their importances change, requiring V-sum propagation and violation
detection along each affected entry's V-ancestor path. Entries outside the
targeted G-subtree are undisturbed. This allows frequency-selective
temporal filtering: high-resolution subbands (deep plateaus) can decay
faster than coarse ones, specific spatial regions can be selectively
amplified while the rest of the tree remains unchanged, and regions can be
hard-reset via targeted annihilation.

**Carrier preservation.** Temporal modulation must preserve the carrier:
$\text{modulate}(i, \lambda) \in I$ for all $i \in I$. Under the standard
configuration, multiplicative scaling by $\lambda \in [0, 1]$ preserves
$\mathbb{R}_{\geq 0}$. Under the signed configuration, any real $\lambda$
preserves $\mathbb{R}$. If modulation is applied to a value at $\bot$,
the result should be $\bot$ (scaling zero by anything gives zero). When P4
holds, this is consistent with the ghost eviction fast path (§12.5 Step 2):
a ghost whose importance remains $\nu$ after modulation is still inert
under absorption.

> _Note (absorbed energy and depth-selective scaling)._ The `scale_subtree`
> function applies $\lambda(d)$ to `g.own` based on the node's _current_
> G-depth. For semi-internal nodes, `g.own` includes energy absorbed from
> evicted children (§12.5 Step 1) — energy that originally accumulated at
> a deeper G-depth. With $q > 0$, this absorbed energy is scaled at the
> parent's shallower depth (milder factor under attenuation, weaker
> amplification under boost) rather than its original deeper depth. This
> is an inherent consequence of absorption destroying depth context — once
> energy is folded into the parent, its original depth is lost. Under
> attenuation with $q > 0$, absorbed benchmarks are stickier (decay less
> than they "would have" at their original depth). Under amplification
> with $q > 0$, they are less responsive (amplify less than they "would
> have"). Under annihilation ($\text{att} = 0$) with $q < 1$, the issue
> does not arise — all nodes are zeroed regardless of depth. Under detail
> flush ($\text{att} = 0$, $q = 1$), only the subtree root is preserved;
> semi-internal nodes at deeper depths are zeroed, so the absorbed-energy
> bias affects only the root node's accumulator. For the noise-floor
> interpretation (§PEWEI M-6), this means the baseline is biased relative to
> what depth-proportional scaling would produce. The bias is conservative
> in both directions: under attenuation, baselines persist longer; under
> amplification, baselines grow more slowly than the fine-scale detail
> they calibrate.

**Non-uniform scaling and V-I3.** For $q > 0$, the per-depth scaling factor
varies with G-depth. Since V-Tree siblings and uncle–nephew pairs may back
G-nodes at different G-depths, they receive different scaling factors. The
uniform-scaling V-I3 preservation argument (§14.2) does not apply. The
direction of scaling determines which class of violations can arise:

> _Example (attenuation, $q > 0$)._ Uncle backs a G-node at G-depth 5:
> importance 80, strong depth-selective attenuation. Nephew backs a G-node
> at G-depth 2: importance 70, mild attenuation. Before decay:
> $70 \leq 80$, V-I3 satisfied. After decay: uncle $\to 56$,
> nephew $\to 63$. Now $63 > 56$: **V-I3 violated.** The uncle weakened
> more than the nephew because the uncle sits at a deeper G-depth with a
> stronger attenuation factor.

> _Example (amplification, $q > 0$)._ Entry backs a G-node at G-depth 5:
> importance 40, strong depth-selective amplification. Uncle backs a G-node
> at G-depth 1: importance 50, mild amplification. Before boost:
> $40 \leq 50$, V-I3 satisfied. After boost: entry $\to 72$,
> uncle $\to 55$. Now $72 > 55$: **V-I3 violated.** The entry amplified
> more than the uncle because the entry sits at a deeper G-depth with a
> stronger amplification factor.

> _Example (detail flush, subtree)._ Before: uncle inside subtree with
> importance 80. Nephew outside subtree with importance 70.
> $70 \leq 80$, V-I3 satisfied. After detail flush: uncle $\to 0$.
> $70 > 0$: **V-I3 violated.** Annihilation is the extreme case of
> attenuation-induced uncle weakening.

**Invariant restoration summary.** Two orthogonal questions determine what
Phase 3 of §14.4 must do: (1) whether a trailing rebalance is needed at
all, and (2) whether the violation detection must check nephew violations
in addition to self-violations.

_Whether trailing rebalance is needed:_

| Configuration                                      | V-I3 status              | Trailing rebalance |
| :------------------------------------------------- | :----------------------- | :----------------- |
| Float, $q = 0$, full-tree, any $\text{att} \geq 0$ | Preserved (§14.2)        | Not required       |
| Integer, $\text{att} = 0$, any $q$, any scope      | Preserved (all values 0) | Not required       |
| All other configurations                           | Violations possible      | **Required**       |

> _Note (integer annihilation)._ When $\text{att} = 0$ and $q < 1$, every
> node in the subtree is zeroed — including the subtree root, since its
> exponent $1 - q > 0$ yields $0^{(\text{positive})} = 0$. All entries and
> structural aggregates become zero. $0 \leq 0$ everywhere. V-I3 trivially
> holds regardless of integer truncation. When $\text{att} = 0$ and $q = 1$,
> the subtree root's exponent is $0$ (convention: $0^0 = 1$, root
> preserved) and all descendants are zeroed. V-I3: each descendant's
> importance is $0 \leq$ any uncle. The root sits at V-depth $\leq 1$ (no
> grandparent) — unconstrained. V-I3 holds. No integer truncation occurs at
> $\lambda \in \{0, 1\}$, so no rebalance is needed regardless of type.

_Whether nephew checks are needed during Phase 3 violation detection:_

Under **amplification** ($\text{att} > 1$) with non-negative importance
(P1 holds), every affected entry's importance is non-decreased ($\lambda \geq
1$ and $v.\text{int} \geq 0$ imply $\lambda \cdot v.\text{int} \geq
v.\text{int}$). Every structural aggregate on affected V-ancestor paths
is also non-decreased (sums of non-decreased terms). An uncle whose
importance increased or stayed the same cannot cause a nephew violation —
a stronger uncle makes V-I3 _easier_ to satisfy, not harder. The only
violation source is an entry growing past its uncle — a **self-violation**,
caught by the standard ancestor walk.

Under **attenuation or annihilation** ($\text{att} < 1$ or $\text{att} = 0$)
with subtree targeting, affected entries' importances decrease (or reach
zero). An affected entry that serves as uncle to a node _outside_ the
targeted G-subtree may weaken below that node's importance, creating a
**nephew violation**. The outside nephew is not on any affected entry's
ancestor walk path; the standard self-violation walk does not visit it.

Under **full-tree scaling** (any direction, any type), every entry is
affected and walked. The self-violation walk from each entry checks
`is_violated` at that entry's position, catching any violation regardless
of whether it was caused by the entry itself growing (amplification) or
its uncle weakening (attenuation). Nephew violations are subsumed by the
universal coverage of the self-walk.

Under **signed importance** (P1 fails) with amplification, negative
importance values become more negative — effectively _decreasing_ — which
can weaken uncles even under amplification. Nephew checks are required for
subtree scaling in this configuration.

| Scope     | Direction               | Float, P1 holds   | Float, P1 fails            | Integer           |
| :-------- | :---------------------- | :---------------- | :------------------------- | :---------------- |
| Full-tree | Any                     | Self-checks only  | Self-checks only           | Self-checks only  |
| Subtree   | $\text{att} > 1$        | Self-checks only  | **Nephew checks required** | Self-checks only  |
| Subtree   | $\text{att} \in (0, 1)$ | **Nephew checks** | **Nephew checks required** | **Nephew checks** |
| Subtree   | $\text{att} = 0$        | **Nephew checks** | **Nephew checks required** | **Nephew checks** |

> _Note (integer subtree amplification)._ Under non-negative integer
> arithmetic with $\lambda \geq 1$, each entry's importance satisfies
> $\lfloor v.\text{int} \cdot \lambda \rfloor \geq v.\text{int}$ (since
> $v.\text{int} \cdot \lambda \geq v.\text{int}$ for $v.\text{int} \geq 0$
> and $\lambda \geq 1$). Structural aggregates — sums of non-decreased
> values — are non-decreased. Inside uncles cannot weaken. Nephew
> violations from outside nephews are impossible. Self-checks suffice,
> matching the float P1-holds case. Integer truncation can still cause
> **self-violations** (different truncation patterns across same-level
> entries), which the self-walk catches.

> _Note (subtree annihilation nephew checks)._ While the subtree-internal
> state after annihilation is trivially V-I3-clean (all zeros), the
> boundary between the annihilated subtree and the rest of the V-Tree
> requires nephew checks: an outside nephew whose only strong uncle was an
> inside entry now faces uncle importance = 0. The nephew check in Phase 3
> (§14.4) detects these boundary violations. Under full-tree annihilation,
> no boundary exists and nephew checks are unnecessary.

### 14.4 The Decay Algorithm

```
function decay(root, att, q):
    // Identity short-circuit: att = 1 produces λ(d) = 1 for all d
    // regardless of q. No values change; no invariant restoration needed.
    if att = 1: return

    D ← N − depth_geo(root)
    sum_before ← root.sum

    // Phase 1: Scale values and recompute G-sums bottom-up
    scale_subtree(root, att, q, D, depth_geo(root))

    // Phase 2: Propagate G-sum change to ancestors above subtree root
    delta ← root.sum − sum_before
    g ← root.geo_parent
    while g ≠ null:
        g.sum ← g.sum + delta
        g ← g.geo_parent

    // Phase 3: Restore V-I1 and detect violations
    //
    // When no violations can arise, skip the V-Tree walk entirely.
    // Two cases: (a) full-tree uniform float scaling — V-I3
    // preserved by the §14.2 proof; (b) any annihilation where
    // all values collapse to zero — V-I3 trivially satisfied.
    //
    // For the remaining cases, two classes of violations may arise:
    // (a) Self-violations: an affected entry's importance changed past
    //     its uncle (or a structural ancestor's aggregate changed past
    //     its uncle). Detected by the standard ancestor walk.
    // (b) Nephew violations: an affected entry that serves as uncle to
    //     a node outside the affected set (or at a different G-depth
    //     under non-uniform scaling) may have weakened below that
    //     node's importance. Detected by checking siblings' children
    //     at each ancestor level.
    //
    // See §14.3 invariant restoration summary for the full analysis.

    is_full_tree ← (root = G_root)
    is_annihilation ← (att = 0 and (q < 1 or D = 0))
    is_detail_flush ← (att = 0 and q = 1 and D > 0)
    can_skip_rebalance ← (is_full_tree and q = 0 and is_float_type)
                          or (is_annihilation and not is_detail_flush)
    // Detail flush with subtree targeting can produce nephew violations
    // at the boundary (the preserved root's descendants in the V-Tree
    // outside the G-subtree are unaffected, but inside entries that
    // served as uncles are now zero). Full-tree detail flush: the root
    // entry is preserved; all other entries are zero; 0 ≤ 0, and the
    // root entry has no grandparent. V-I3 holds, can skip.
    if is_detail_flush and is_full_tree:
        can_skip_rebalance ← true

    if not can_skip_rebalance:
        is_subtree ← not is_full_tree
        needs_nephew_checks ← is_subtree and (att < 1 or att = 0
                               or not P1)

        for each G-node g in root's subtree where g.entry ≠ null:
            propagate_v_sums(g.entry)
            a ← g.entry
            while a ≠ null:
                if is_violated(a): push a                    // self-violation
                if needs_nephew_checks:
                    p ← a.val_parent
                    if p ≠ null:
                        for each sibling s of a under p:
                            for each child c of s:
                                if is_violated(c): push c    // nephew violation
                a ← a.val_parent
    else:
        // V-I1 must still be restored even when V-I3 is preserved.
        // Under full-tree uniform float scaling, structural aggregates
        // can be recomputed by a single bottom-up pass or per-entry
        // propagation. Under annihilation, all entries are zero and
        // structural aggregates must be recomputed to zero.
        for each G-node g in root's subtree where g.entry ≠ null:
            propagate_v_sums(g.entry)

    // Phase 4: Rebalance
    rebalance()                                    // §11.8

    // Phase 4b: Adjust depth gates
    adjust_depth_gates()                           // §7.4

    // Phase 5: Evict (decay may push entries past D_evict;
    //          annihilation produces zero-importance entries
    //          that are prime eviction candidates)
    check_evictions()                              // §12.6

    // Phase 6: Update plateau map
    update_plateau_map_after_decay(root)            // §5.6.7


function scale_subtree(g, att, q, D, d_root):
    // Post-order: children first, then self
    if g.left ≠ null:  scale_subtree(g.left, att, q, D, d_root)
    if g.right ≠ null: scale_subtree(g.right, att, q, D, d_root)

    d_local ← depth_geo(g) − d_root
    if D = 0:
        λ ← att                    // single-node subtree: uniform scaling
    else:
        exponent ← 1 + q × (2 × d_local / D − 1)
        if att = 0:
            // Explicit 0^exponent: avoid platform-dependent pow(0, 0).
            if exponent = 0:
                λ ← 1              // 0^0 = 1 by convention (§13.3)
            else:
                λ ← 0              // 0^(positive) = 0
        else:
            λ ← att ^ exponent
    // λ = 0 annihilates; λ = 1 preserves; 0 < λ < 1 attenuates; λ > 1 amplifies.

    g.own ← modulate(g.own, λ)
    g.importance ← modulate(g.importance, λ)
    // Under the standard configuration (identity projection), own and
    // importance are the same accumulator (§8.6.2); a single modulate
    // suffices.

    // Recompute sum from scaled own and children's (already scaled) sums
    g.sum ← g.own
    if g.left ≠ null:  g.sum ← g.sum + g.left.sum
    if g.right ≠ null: g.sum ← g.sum + g.right.sum
```

**Phase-by-phase rationale.**

_Phase 1_ performs the post-order walk, ensuring children are scaled before
parents. This allows bottom-up `sum` recomputation at each node from
already-correct children. G-I1 holds within the subtree after Phase 1
completes. Cost: $O(S)$ where $S$ is the subtree size. Under annihilation,
this reduces to zeroing every field in the subtree — an $O(S)$ memset-like
operation.

_Phase 2_ propagates the net change in the subtree root's sum to all
G-ancestors above the subtree root, restoring G-I1 globally. For full-tree
decay ($\text{root} = G_{\text{root}}$), there are no ancestors and this
phase is a no-op. Under full-tree annihilation ($\text{att} = 0$, $q < 1$),
$\text{delta} = 0 - \text{sum\_before} = -\text{sum\_before}$; but since
the root has no geo-parent, the loop body never executes. Under subtree
annihilation, all G-ancestors above the subtree root decrease by the
subtree's pre-decay sum. Cost: $O(d_{\text{root}})$.

_Phase 3_ restores V-I1 at every V-structural ancestor of every affected
entry, and detects V-I3 violations. Each affected entry's importance has
already been scaled in Phase 1; `propagate_v_sums` recomputes structural
aggregates along the V-ancestor path. The ancestor walk (matching §8.2
Step 3c) catches self-violations. When `needs_nephew_checks` is true, the
additional sibling-children check at each ancestor level catches nephew
violations from weakened uncles — the same pattern as
`push_leaf_removal_violations` (§11.11.3). The `can_skip_rebalance` guard
skips the violation walk when no violations can arise (the two proven
cases: full-tree uniform float scaling, and annihilation); V-I1 restoration
via `propagate_v_sums` still executes to recompute structural aggregates.
Cost: $O(S \cdot h_V)$ worst case — $S$ entries, each propagating $O(h_V)$
to the V-root. The nephew check adds at most $6$ `is_violated` calls per
ancestor level ($\leq 2$ siblings $\times$ $\leq 3$ children), a bounded
constant-factor increase. Many propagation paths overlap; see the
deduplication note below.

_Phase 4_ drains the violation queue. If Phase 3 skipped violation
detection (the `can_skip_rebalance` path), the queue is empty and
`rebalance()` returns immediately. The termination arguments of §11.13
apply unchanged — importances are fixed during rebalancing (no observations
arrive within the decay operation). Cost: bounded by the $\Phi$-decrease
argument (§11.13.1) under standard modes. In the worst case, every entry
in the subtree is violated and each promotion traverses $O(h_V)$ levels;
the $\Phi$-decrease argument bounds total work at $O(S \cdot h_V)$. Under
proportional traffic following the decay, expected cost is $O(1)$ per
entry (§18.8).

_Phase 5_ runs the eviction scan. Under attenuation, decay weakens entries,
potentially pushing them past $D_{\text{evict}}$ through competitive
demotion during Phase 4's rebalancing. Under annihilation, zeroed entries
have ground importance (when P2 holds) — they are prime candidates for
eviction. Phase 4's rebalancing pushes them to maximum V-depth (non-zero
outside entries promote past zeroed inside entries), and Phase 5 removes those past
$D_{\text{evict}}$. The ghost fast path (§12.5 Step 2) applies to every
zeroed entry, making each eviction $O(1)$. Under amplification, Phase 5 is
typically a no-op — amplification strengthens entries, pushing them toward
shallower V-depth, away from the eviction threshold. However, Phase 4's
rebalancing may displace bystander entries deeper as amplified entries
promote upward, indirectly creating eviction candidates. Phase 5 must
execute unconditionally regardless of scaling direction. The eviction pass
removes newly eligible unprotected contour cells, their parents absorb,
and the trailing rebalance within `check_evictions` resolves
eviction-triggered violations.

_Phase 6_ updates the plateau ordered map. Every plateau whose basis
elements intersect the decayed subtree has a changed sum. Phase 6
recomputes all affected plateau sums from their basis elements' current
`g.sum` values. For full-tree decay, all $P$ plateaus are affected. For
subtree decay, only plateaus whose basis elements overlap the targeted
G-subtree require updates. Phase 5's per-eviction updates
(§12.5 Step 9) handled structural changes (inserting or removing
plateaus); Phase 6 handles the residual energy changes from Phase 1 that
were not covered by Phase 5's structural updates. The recomputation is
idempotent — plateaus already updated by Phase 5 are correctly
recomputed. Cost: $O(P)$ for full-tree decay; $O(S + \log P)$ for subtree
decay (walk the subtree to identify affected basis elements, $O(\log P)$
per map operation).

**Cost summary.**

| Phase   | Full-tree ($S = \|G\|$) | Subtree ($S \ll \|G\|$) |
| ------- | ----------------------- | ----------------------- |
| Phase 1 | $O(\|G\|)$              | $O(S)$                  |
| Phase 2 | $O(1)$                  | $O(d_{\text{root}})$    |
| Phase 3 | $O(\|G\| \cdot h_V)$    | $O(S \cdot h_V)$        |
| Phase 4 | $O(\|G\| \cdot h_V)$    | $O(S \cdot h_V)$        |
| Phase 5 | $O(E_t \cdot h_V)$      | $O(E_t \cdot h_V)$      |
| Phase 6 | $O(P)$                  | $O(S + \log P)$         |

Total cost is dominated by Phases 3–4: $O(S \cdot h_V)$. The
G-Tree traversal is $O(S)$; the V-Tree propagation and rebalancing take
$O(S \cdot h_V)$. The phrase "decaying a subtree costs $O(S)$" applies
to the G-Tree portion only.

Under full-tree annihilation ($\text{att} = 0$, $q < 1$) or full-tree
detail flush ($\text{att} = 0$, $q = 1$), Phase 3 skips violation detection
and Phase 4's rebalance queue is empty. The dominant cost is Phase 1
($O(|G|)$) plus Phase 3's V-I1 restoration ($O(|G| \cdot h_V)$) plus
Phase 5's eviction scan. If the post-annihilation eviction removes most
entries, Phase 5's cost is $O(|G| \cdot h_V)$ (one eviction pass). The
ghost fast path ensures each individual eviction is $O(1)$, so
Phase 5's total is $O(E_t)$ for the evictions themselves plus $O(h_V)$ for
the trailing rebalance per eviction batch (typically trivial — zeroed
entries create no violations when evicted).

> _Implementation note (batched V-propagation)._ Phase 3's per-entry
> propagation visits the same V-structural nodes repeatedly when
> multiple affected entries share V-ancestors. An epoch-counter scheme
> avoids redundant work: before Phase 3, increment a global counter;
> during `propagate_v_sums`, skip any structural node whose counter
> matches the current epoch. For violation detection, track which nodes
> have been checked at each level. The asymptotic cost remains
> $O(S \cdot h_V)$ without deduplication; with epoch tracking, it
> reduces to $O(S + |V_{\text{affected}}|)$ where $|V_{\text{affected}}|$
> is the number of distinct V-structural nodes on affected paths —
> typically much smaller than $S \cdot h_V$ when many entries share
> V-ancestors. An alternative: scale all $S$ entries' importances in
> Phase 1, then perform a single bottom-up pass over the entire V-Tree,
> recomputing each structural node's aggregate from its children in
> $O(|V|)$. This is optimal for full-tree decay and wasteful for small
> subtree decay. A hybrid approach — mark affected entries as dirty,
> propagate only through structural ancestors of dirty entries — reduces
> redundant recomputation while avoiding a full V-Tree pass. The choice
> is an implementation concern; all three approaches produce the same
> result.

> _Budget invariant note._ Phase 4's rebalancing may trigger legacy
> promotions (§11.6), temporarily increasing $|G|$. Each legacy promotion
> creates one G-node ($+1$ to $|G|$) and consumes one semi-internal
> ($-1$ to $S$). The quantity $|G| + S$ is therefore invariant under
> legacy promotion. The budget invariant $|G| + S + 2 \leq G_{\max}$ (§7.5.3)
> holds throughout Phase 4 because the invariant held at entry to
> `decay()` and legacy promotions do not change $|G| + S$. Phase 5's
> evictions decrease $|G|$ while increasing $S$ by at most the same
> amount (each eviction: $-1$ to $|G|$, at most $+1$ to $S$), so
> $|G| + S$ is non-increasing. The budget invariant is maintained
> throughout the decay operation. Under annihilation followed by heavy
> eviction, $|G|$ decreases rapidly — the budget invariant becomes
> increasingly easy to satisfy.

**Transient invariant windows.** Under the sequential model (§17.3), no
external operation occurs during these windows.

| After phase | G-I1                                    | V-I1         | V-I3                   | Plateau map  |
| ----------- | --------------------------------------- | ------------ | ---------------------- | ------------ |
| Phase 1     | Within subtree: ✓. Ancestors: **stale** | **Stale**    | May be violated        | **Stale**    |
| Phase 2     | **Restored** everywhere                 | **Stale**    | May be violated        | **Stale**    |
| Phase 3     | ✓                                       | **Restored** | Violations queued      | **Stale**    |
| Phase 4     | ✓                                       | ✓            | **Restored**           | **Stale**    |
| Phase 5     | ✓                                       | ✓            | ✓ (trailing rebalance) | **Stale**    |
| Phase 6     | ✓                                       | ✓            | ✓                      | **Restored** |

**Notable window.** Between Phase 1 and Phase 2, G-ancestors above the
subtree root have stale sums. G-I1 is violated along the path from the
subtree root to the G-root. No G-Tree query should execute during this
window. Under annihilation, this staleness is maximal: every ancestor's
sum is overstated by the entire annihilated subtree's pre-decay sum.

Under amplification, the "violations queued" cell at Phase 3 typically
contains more self-violations (upward-growing entries) and fewer
nephew violations (no uncle weakening when P1 holds). Under attenuation or
annihilation with subtree targeting, the opposite pattern holds — nephew
violations dominate. Phase 4 handles both uniformly.

### 14.5 The G-V Graph as a Spatiotemporal Filter Bank

When decay is applied, the G-V Graph's state can be understood as a
three-stage spatiotemporal filter bank:

1. **Analysis.** The G-Tree's depth structure defines spatial subbands:
   each contour depth level corresponds to a spatial resolution.
   Observations route to individual cells via `route_to_receiver` (§5.2);
   G-sum propagation (§8.3.4) accounts for each observation at every coarser
   scale.

2. **Processing.** The V-Tree ranks entries within and across subbands by
   competitive importance. Temporal scaling adjusts subbands at
   depth-dependent rates (§14.3): under attenuation ($\text{att} < 1$),
   cold subbands are suppressed and hot subbands persist; under
   amplification ($\text{att} > 1$) with $q > 0$, fine-scale subbands are
   selectively boosted relative to coarse (sharpening); under annihilation
   ($\text{att} = 0$), subbands are hard-reset, and the detail flush
   variant ($q = 1$) selectively zeroes fine-scale structure while
   preserving the coarsest approximation. Together they implement a
   time-varying gain schedule per subband.

3. **Synthesis.** PEWEI reconstruction (companion document §§PEWEI M-9–10) reassembles the filtered subbands into a spatial intensity function, producing the system's output at the caller's chosen resolution.

### 14.6 Sliding Window

An alternative to exponential decay is a **sliding window** that subtracts
expired observations from accumulators, maintaining a running total over
the most recent $W$ time units. The sliding window interacts non-trivially
with the architecture. This section enumerates the challenges; a complete
specification is out of scope.

**Per-observation storage.** The window requires a time-ordered log of
observations $(x, \Delta, t)$. When an observation expires at time $t + W$,
its contribution must be reversed. Storage cost: $O(W \times R)$ where $R$
is the observation rate.

**Capability requirement.** Subtracting expired observations requires the
Subtraction capability on $T$ (§8.8.2). Integer types without general
subtraction (e.g., saturating unsigned integers) cannot support sliding
window decay.

**Routing expired observations.** When an observation expires, the reversal
must be routed to the **current** receiver at coordinate $x$, which may
differ from the original receiver if the G-Tree has restructured since the
observation. Routing uses `route_to_receiver(G_root, x)` (§5.2) at
expiration time.

**Evicted-node absorption chains.** If the original receiver has been
evicted since the observation, its value was absorbed by an ancestor
(§12.5). The expired observation's contribution now resides in the
absorbing ancestor's `own` — which may itself be frozen (if the ancestor
split again after absorbing). Reversing the contribution requires tracking
which ancestor currently holds each observation's energy. This bookkeeping
is non-trivial and application-specific; the architecture does not provide
it.

**Importance invertibility.** Under the standard configuration (identity
projection), aggregation is addition; reversal is subtraction. Under
absolute projection, $\pi(\Delta) = |\Delta|$, so aggregation is
$\text{current} + |\Delta|$; reversal requires subtracting the original
$|\Delta|$ (not $|\Delta|$ of the current value), so the original absolute
increment must be stored alongside the observation in the log.

**V-I3 violations.** Subtracting expired observations is non-uniform — each
node loses a different expired delta. V-I3 violations are expected and
require trailing `rebalance()`.

**Interaction with frozen entries.** An internal node's frozen `own` may
include observations that are now expiring. Subtracting them weakens the
frozen benchmark, potentially allowing children to outgrow the weakened
uncle and triggering competitive promotion. This is correct behavior: the
benchmark should soften as the evidence that created it ages out.

> _Practical alternative._ For many applications, periodic exponential
> decay (§14.3) approximates sliding window semantics with far less
> implementation complexity: no per-observation storage, no absorption
> chain tracking, and clean interaction with all architectural mechanisms.
> The approximation is exact in the limit of continuous decay with matching
> half-life. For applications that need a hard cutoff rather than
> exponential forgetting, the annihilation operation ($\text{att} = 0$)
> provides a targeted hard reset of any spatial region — conceptually the
> same as expiring all observations in that region simultaneously. The
> combination of periodic attenuation with occasional targeted annihilation
> covers many use cases that would otherwise require sliding window
> bookkeeping.

### 14.7 No Filter

Use raw accumulation. Do not call `decay()`. Entries reflect lifetime
significance — the full history of observation at each scale and location.
Under raw accumulation, frozen benchmarks harden monotonically (§13.5),
making re-expansion progressively more expensive with each expand–contract
cycle. The contour contracts only through dynamic $D_{\text{evict}}$
adjustment (§7.4) or external maintenance triggers (§12.7), not through
competitive demotion. This is the default.

**Amplification without compensating decay.** A user may call `decay()`
with $\text{att} > 1$ periodically without ever applying $\text{att} < 1$.
This produces geometric growth of all importance values. The V-Tree's
competitive ranking is preserved under uniform amplification (all entries
scale equally), but:

- The split threshold $\theta$ becomes relatively easier to exceed.
  Terminal entries below $\theta$ may cross it after amplification,
  causing the system to become progressively more "eager" to refine.
- Importance values grow without bound under repeated amplification.
  The budget mechanism (§7.4) prevents unbounded node growth (tightening
  $D_{\text{evict}}$ under memory pressure), but importance _values_ are
  unconstrained — overflow is the user's responsibility (§14.2).

Under depth-selective amplification ($q > 0$), fine-scale entries grow
faster than coarse, progressively sharpening the importance landscape.
Balanced use — alternating amplification and attenuation, or combining
amplification in one subtree with attenuation in another — is the expected
pattern for most applications. The architecture imposes no policy on how
`decay()` is called; the user chooses the combination that matches the
application's temporal semantics.

**Annihilation as a reset mechanism.** A user may call
`decay(root, 0, 0)` to hard-reset any G-subtree to zero. This is
equivalent to erasing all observations in the targeted region and starting
fresh. Under full-tree annihilation, the system returns to a state
equivalent to post-initialization (all accumulators at zero; the G-Tree's
spatial topology persists but carries no energy). The first subsequent
observation re-seeds the system. Under subtree annihilation, the targeted
region is erased while the rest of the tree continues operating —
unaffected entries retain their competitive positions. The detail flush
variant ($\text{att} = 0$, $q = 1$) preserves the subtree root's coarse
measurement while zeroing confirmed sub-scale structure, forcing the tree
to re-earn its spatial resolution from scratch.

> _Design note (the breathing cycle)._ Under exponential decay, entries
> whose traffic has subsided decay in importance, are pushed deeper in
> the V-Tree by newly hot entries via the uncle constraint, eventually
> cross $D_{\text{evict}}$ as terminal nodes, and are evicted. Their
> G-parents absorb the value (§12.5). When both children have been
> evicted, the parent becomes terminal and eligible for eviction itself —
> the tidal erosion of §12.7. Meanwhile, decay attenuates frozen
> benchmarks at internal nodes, softening the competitive bar that
> children must exceed for promotion (§13.5). Under amplification, the
> dual dynamic applies: frozen benchmarks strengthen, raising the bar for
> future children, while fine-scale entries grow faster ($q > 0$),
> potentially outpacing the strengthened benchmarks and triggering
> promotion. Under annihilation, the cycle is interrupted: zeroed entries
> lose all competitive standing immediately, sink to maximum V-depth, and
> are evicted in the subsequent tidal pass — a fast-forward contraction
> that bypasses the gradual erosion of normal decay. The balance between
> accumulation (new observations strengthening entries), attenuation
> (decay weakening them), amplification (selective boosting), and
> annihilation (targeted hard reset) governs the equilibrium tree depth:
> aggressive decay produces shallow, volatile trees; mild decay produces
> deep, stable trees; selective amplification can sharpen specific
> regions; and annihilation provides an escape hatch when gradual
> contraction is too slow. The tree breathes: expands under load,
> contracts layer by layer when traffic subsides, and can be selectively
> stimulated or reset by the user's choice of temporal filter. The triad
> of split/decay/eviction is the mechanism; the architecture provides
> split and eviction, and provides the built-in `decay()` method as the
> standard temporal filter. Users may alternatively apply any custom
> temporal filter by directly manipulating entry accumulators and
> following the temporal filter contract (§14.1).

---

## Chapter 15. Initialization

This chapter specifies the construction of a G-V Graph from its
configuration parameters. Initialization produces the minimal valid state:
a single G-node covering the full domain, a single V-entry serving as the
V-root, an empty violation queue, and a one-plateau contour map. Both
trees start as a single shared reference. The first observation that
pushes the root's importance above $\theta$ triggers the bootstrap split
(§10.1, §10.3).

### 15.1 Construction-Time Validation

The configuration parameters must satisfy all constraints from §3 and §7
before any state is allocated. Validation failures are hard errors —
rejected immediately.

| Parameter               | Constraint                                             | Source     |
| ----------------------- | ------------------------------------------------------ | ---------- |
| $N$                     | $N \geq 1$                                             | §3         |
| $N$                     | $N \leq \text{bit\_width}(C)$                          | §3         |
| $\theta$                | $\theta > \nu$ (via $\preceq$ on the carrier)          | §7.1, D-I4 |
| $D_{\text{create}}$     | $D_{\text{create}} \geq 0$                             | §7.1, D-I4 |
| $D_{\text{evict}}$      | $D_{\text{evict}} \geq 1$                              | §7.1, D-I4 |
| $\text{buffer}$         | $\text{buffer} \geq 1$                                 | §7.1       |
| $D_{\text{evict}}$      | $D_{\text{evict}} = D_{\text{create}} + \text{buffer}$ | §7.2, D-I3 |
| $\text{budget}$         | $\text{budget} > 0$                                    | §7.4       |
| $G_{\max}$              | $G_{\max} \geq 5$ and $G_{\max} > \text{budget}$       | §7.5.3     |
| $\alpha_{\text{relax}}$ | $\in (0, 1)$                                           | §7.4       |

> _Note ($\theta$ constraint)._ The constraint $\theta > \nu$ is
> stated canonically in §7.1 and D-I4. Under the standard configuration
> where $\nu = 0$, this reduces to $\theta > 0$. Under the elevated
> configuration (§8.6.5) where $\nu > \bot$, the stronger formulation
> is essential: if $\theta$ satisfied only $\theta > \bot$ while
> $\theta \leq \nu$, every freshly created entry would immediately
> satisfy the split guard, defeating the gate entirely.

> _Note (parameter primacy)._ The three parameters $D_{\text{create}}$,
> $D_{\text{evict}}$, and $\text{buffer}$ are related by
> $D_{\text{evict}} = D_{\text{create}} + \text{buffer}$. Only two of the
> three are independent. §7.1 defines $\text{buffer}$ as
> $D_{\text{evict}} - D_{\text{create}}$, treating $D_{\text{create}}$ and
> $D_{\text{evict}}$ as primary. The initialization accepts all three and
> asserts consistency — explicit redundancy with a consistency check.
> Implementations may alternatively accept two and derive the third;
> the invariant $D_{\text{evict}} = D_{\text{create}} + \text{buffer}$ is
> the normative requirement regardless of which parameters are user-facing.

```
function validate_parameters(N, C, θ, D_create, D_evict, buffer, budget, G_max, α_relax):
    assert N ≥ 1                                // §2: minimum domain size
    assert N ≤ bit_width(C)                     // §2: coordinate representability
    assert θ > ν                            // §7.1, D-I4
    assert D_create ≥ 0                         // §7.1, D-I4
    assert D_evict ≥ 1                          // §7.1, D-I4
    assert buffer ≥ 1                           // §7.1
    assert D_evict = D_create + buffer          // §7.2, D-I3
    assert budget > 0                           // §7.4
    assert G_max ≥ 5                            // §7.5.3: minimum for bootstrap split
    assert G_max > budget                       // §7.5: headroom above soft target
    assert 0 < α_relax < 1                     // §7.4: hysteresis fraction
```

For floating-point coordinate types, the additional concerns of §3.2
apply: the ordered map key type must provide a total-order comparator
(§3.2.7), and all entry points must reject NaN (§3.2.4). These are
structural properties of the coordinate type, verified at type
construction rather than at graph initialization.

### 15.2 The Algorithm

```
function initialize(N, C, θ, D_create, D_evict, buffer, budget, G_max, α_relax):
    validate_parameters(N, C, θ, D_create, D_evict, buffer, budget, G_max, α_relax)

    // ── G-Tree: single root covering the full domain ──
    root ← new G-Node(
        l           = 0,
        r           = 2^N,
        sum         = 0,
        own         = 0,
        importance  = ν,                  // §4.1: initialized to ground at creation
        left        = null,
        right       = null,
        entry       = null               // assigned below; see §3.1 convention
    )
    root.geo_parent ← null

    // ── V-Tree: single entry serving as V-root ──
    //
    // The V-root starts as an entry (§6.1). It transitions to a
    // structural root on the first split (§10.3, bootstrap split)
    // and never reverts.
    e ← new V-Entry(
        int          = root.importance,   // reference, not copy — G-I4 (§5.4)
        gnode        = root,
        is_exposed   = true,              // terminal: fully exposed — V-I6 (§6.2)
        is_evictable = true               // terminal: no dependents — V-I6b (§6.2)
                                          // Root exemption (§12.5) prevents actual
                                          // eviction; this flag reflects structural
                                          // state, not eviction eligibility.
    )
    root.entry ← e
    e.val_parent ← null

    G_root ← root
    V_root ← e

    // ── Counters and dynamic control (§7.4, §7.5.4) ──
    total_nodes ← 1                       // one live G-node
    S ← 0                                 // zero semi-internal G-nodes

    // Store configuration parameters as graph fields
    store θ, D_create, D_evict, buffer, budget, G_max, α_relax

    // ── Plateau map (§5.6.7) ──
    //
    // One plateau covering the full domain at depth 0. The root is
    // its sole basis element (§5.6.1). The plateau sum equals the
    // root's sum (zero). The basis edge is 0 — the domain origin.
    // The Plateau representation carries the fields described in
    // §5.6.7: thatched spatial extent (run), contour depth, basis
    // set, and total energy (sum).
    plateaus ← new OrderedMap()
    plateaus.insert(
        BasisEdge(0),
        Plateau(depth = 0, basis = {root}, sum = root.sum, run = [0, 2^N))
    )

    // ── Violation queue (§11.1) ──
    violation_queue ← empty    // §11.1.1: Collection of V-node handles, initially empty
```

> _Note (reference semantics of `v.int`)._ The V-entry's `int` field
> is a **reference** to `root.importance`, not an independent copy.
> When `accumulate_importance` updates `root.importance` during an
> observation (§8.3.3), the V-entry sees the new value immediately
> because it holds a reference to the same accumulator. This is the
> mechanism by which G-I4 is maintained without explicit synchronization.
> See §4.2 (Reference semantics) for the normative requirement.

> _Note (V-root as entry)._ The V-root is an entry — not a structural
> node — until the first split. This is the unique moment in the tree's
> lifecycle where the V-root is a leaf. §6.1 states: "The root may be
> either an entry (when only one entry exists) or structural. The root
> transitions from entry to structural on the first split (§10.3) and
> never reverts." Sampling (§6.5) from a single-entry V-root returns
> that entry directly (the `if v is entry: return v` base case) — unless
> the ground guard fires first (see §15.4). The uncle constraint
> (V-I3) is vacuously satisfied — a node at V-depth 0 has no
> grandparent.

> _Note (value space and projection)._ The value space
> $\mathcal{V} = (I, \oplus, \nu, \preceq)$ and projection $\pi$ are
> type-system parameters resolved at construction time. They are not stored
> as runtime fields. The properties P0–P5 are similarly resolved — either
> via compile-time trait checks (monomorphic) or via construction-time
> property flags (polymorphic). A single importance type
> (e.g., $\mathbb{R}_{\geq 0}$) may be paired with different projection
> functions (identity, absolute, or user-defined), producing different
> property profiles and different semantic relationships between the
> ledger and the importance signal (§8.6.1). The configuration does not
> affect initialization state — all configurations produce identical
> initial conditions with `importance = ν` — but must be resolved at
> construction time; it cannot change after the first `observe()` call
> without invalidating the importance accumulators' semantic relationship
> to the ledger. The worked example in §16 uses the standard
> configuration; §15.4 describes initial operational properties in
> configuration-neutral terms.

### 15.3 Invariant Verification

After initialization, every invariant defined in the specification holds.

**G-Tree invariants:**

- **G-I1 (Summation):** $\text{root.sum} = \text{root.own} +
  \sum_{\text{children}} c.\text{sum} = 0 + 0 = 0$. No children exist.
  $\checkmark$

- **G-I2 (Variable Fanout):** Root has zero children.
  $\checkmark$

- **G-I3 (Dyadic):** $[0, 2^N)$ has width $r - l = 2^N$, which equals
  $2^k$ for $k = N$. The interval is dyadic. $\checkmark$

- **G-I4 (Importance Reference):** $\text{root.entry} = e \neq \text{null}$,
  and $e.\text{int}$ references $\text{root.importance} = \text{New}()$.
  $\checkmark$

**V-Tree invariants:**

- **V-I0 (Non-emptiness):** One entry exists ($e$).
  $\checkmark$

- **V-I1 (Summation):** No structural nodes exist. Vacuously satisfied.
  $\checkmark$

- **V-I2 (Branching):** No structural nodes exist. Vacuously satisfied.
  $\checkmark$

- **V-I3 (Max-Uncle):** $e$ is at V-depth 0. No grandparent exists.
  `is_violated(e)` returns false (§11.2, null-grandparent guard).
  $\checkmark$

- **V-I4 (Unique Backing):** One V-entry ($e$) backs one G-node
  (root). $\checkmark$

- **V-I5 (Entry-Leaf):** $e$ is an entry and a leaf. No structural nodes
  exist. $\checkmark$

- **V-I6 (Exposed Flag):** $e.\text{is\_exposed} = \text{true}$.
  $\text{uncovered\_range}(\text{root}) = [0, 2^N) \neq \text{null}$.
  $\checkmark$

- **V-I6b (Evictable Flag):** $e.\text{is\_evictable} = \text{true}$.
  $\text{has\_dependents}(\text{root}) = \text{false}$ (zero children),
  so $\neg\,\text{has\_dependents}(\text{root}) = \text{true}$.
  $\checkmark$

- **V-I7 (Subtree Evictable Flag):** No structural nodes exist.
  Vacuously satisfied. $\checkmark$

**Depth gate invariants (§7.2):**

- **D-I1:** No splits have occurred. Vacuously satisfied. $\checkmark$

- **D-I2:** No evictions have occurred. Vacuously satisfied. $\checkmark$

- **D-I3:** $D_{\text{create}} < D_{\text{evict}}$ — enforced by
  parameter validation ($\text{buffer} \geq 1$). $\checkmark$

- **D-I4:** $\theta > \nu$, $D_{\text{evict}} \geq 1$,
  $D_{\text{create}} \geq 0$ — enforced by parameter validation.
  $\checkmark$

**Budget invariant (§7.5.3):**

$$|G| + S + 2 = 1 + 0 + 2 = 3 \leq G_{\max}$$

Holds because $G_{\max} \geq 5$ (§15.1). The minimum $G_{\max} \geq 5$ also
guarantees survival of the first bootstrap split, which creates 2 new
G-nodes (pushing $|G|$ to 3) with $S = 0$ (terminal → internal skips
semi-internal). After the bootstrap: $|G| + S + 2 = 3 + 0 + 2 = 5 \leq G_{\max}$.
$\checkmark$

> _Note (what $G_{\max}$ bounds)._ The hard ceiling $G_{\max}$ bounds the G-node count
> $|G|$, not the total node count across both trees. V-structural nodes
> are not counted against $G_{\max}$; they are bounded implicitly by $O(|G|)$
> through V-I2 (each structural node has 2 or 3 children, so the number
> of structural nodes is at most $|G| - 1$ when every V-leaf is an
> entry). The bootstrap split creates two V-structural nodes
> (`child_s` and `root_s` in §10.3) in addition to the two G-nodes —
> these do not affect the budget invariant.

**Plateau invariants (§5.6.4):**

- **P-I1 (Deterministic Tiling):** One plateau with key $a_0 = 0$.
  The contour depth function $\sigma \equiv 0$ has step set $\{0\}$.
  $\text{keys}(\text{plateaus}) = \{0\}$. The sole basis element (root)
  has $\text{tile}(\text{root}) = [0, 2^N)$. $\checkmark$

- **P-I2 (Minimal Deterministic Basis):** The root is the sole basis
  element and the unique maximal element whose contour falls within
  the plateau. $\checkmark$

- **P-I3 (Tile Disjointness):** One plateau — nothing to intersect.
  $\checkmark$

- **P-I4 (Thatch):** No semi-internal nodes. No thatching.
  $\checkmark$

- **P-I5 (Thatch Depth):** $\text{thatch\_depth}(x) = 1$ everywhere
  (only the owning plateau). $1 \leq 0 + 1 = 1$. $\checkmark$

**Counter consistency:**

- $\text{total\_nodes} = 1 = |G|$. $\checkmark$
- $S = 0$. Root is terminal (zero children), not semi-internal.
  $\checkmark$

### 15.4 Initial State Properties

The initialized graph has the following operational characteristics:

**Observation.** The root is the sole receiver for all coordinates in
$[0, 2^N)$. `route_to_receiver(G_root, x)` returns root for any valid
$x$ — both branches of the routing function (§5.2) fall through to
`return g` because both children are null.

**Splitting.** The first observation updates `root.importance` via
$\text{importance} \leftarrow \text{importance} \oplus \pi(\Delta)$
(§8.6). The resulting importance depends on the configuration: under
the standard configuration $(+, 0, \leq)$ with identity projection,
importance equals $\Delta$; under absolute projection, importance
equals $|\Delta|$; under the signed configuration, importance equals
$\Delta$ (which may be negative). If the resulting importance
exceeds $\theta$ and $\text{root.entry.val\_parent} = \text{null}$
(the bootstrap case in §10.1), `bootstrap_split` (§10.3) fires. The
bootstrap creates two G-children and two V-structural nodes,
transitioning the V-root from an entry to a structural node. V-I3
preservation after the bootstrap depends on the property profile
(see §10.4 for the property-conditional violation analysis).

If `root.importance` does not exceed $\theta$ after the first
observation (or after multiple observations), observations accumulate
at the root without structural change. The graph operates as a
single-cell accumulator until the split threshold is crossed. The
constraint $\theta > \nu$ (§15.1) guarantees that at least
one observation is required before the root can qualify for splitting.

**Sampling.** Sampling behaviour at initialization depends on the
value space's property profile:

| Property profile                          | `sample(V_root)` result                                                                                                        |
| ----------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------ |
| P2 holds ($\nu \preceq a$ for all $a$)    | Returns $\perp$ — ground guard fires; zero importance means no distribution to sample from (§6.5)                              |
| P1 holds, P2 fails ($\nu$ is not minimum) | Returns the root entry — ground guard does not fire; the entry has importance $\nu$, which is non-ground but pre-observational |
| P1 fails                                  | Undefined — sampling requires P1 (§6.5 precondition)                                                                           |

When P2 holds, sampling becomes defined after the first non-zero
observation. When P1 holds but P2 fails (elevated configuration),
sampling is technically defined at initialization (it returns the sole
entry), but the result carries pre-observational importance — the
caller should interpret it accordingly.

**Eviction.** The root is permanently exempt from eviction (§12.3, §12.5).
The `is_evictable` flag is true (the root is terminal with no
dependents — V-I6b is correctly maintained), but the root exemption
overrides the flag at every eviction check point: the
`scan_for_candidates` function (§12.6) explicitly filters
$v.\text{gnode} = G_{\text{root}}$, and `evict()` (§12.5) returns
immediately if $g = G_{\text{root}}$. No eviction can occur until the
tree has grown past the root.

**Plateau query.** `plateaus.range(..=BasisEdge(x)).next_back()` returns
the sole plateau for any coordinate $x \in [0, 2^N)$. The plateau
reports depth 0, sum 0, and run $[0, 2^N)$.

---

## Chapter 16. Worked Example

### 16.1 Configuration and Setup

> _Configuration._ This example uses the **standard configuration**:
> $\mathcal{V} = (\mathbb{R}_{\geq 0},\; +,\; 0,\; \leq)$ with identity
> projection $\pi(\Delta) = \Delta$. All properties P0–P5 hold. Ground
> is zero, aggregation is ordinary addition, and importance equals the
> raw observation value. Feature annotations [P$k$] mark the enabling
> property when a feature is invoked.

**Parameters.** $N = 3$, domain $[0, 8)$, split threshold $\theta = 5$,
$D_{\text{create}} = 3$, $D_{\text{evict}} = 6$.

---

### 16.2 Initialization

Single G-node covering $[0, 8)$, entry with importance 0, fully exposed,
serving as V-root.

```
G-Tree:  [0,8) sum=0, own=0      V-Tree:  root.entry(0, exposed=true)
```

---

### 16.3 Observation Phase

#### 16.3.1 Overview

Three observations exercise three distinct architectural mechanisms:

| Step | Observation    | Mechanism exercised                                |
| ---- | -------------- | -------------------------------------------------- |
| 1    | observe(3, 10) | Bootstrap split (§10.3)                            |
| 2    | observe(3, 15) | Catalytic split + competitive promotion            |
| 3    | observe(6, 8)  | Uncle shield — significant injection, no violation |

**Contour trajectory:**

| Step   | Event                | Contour state                                      | $P$ |
| ------ | -------------------- | -------------------------------------------------- | --- |
| Init   | —                    | 1 plateau, depth 0: `[0,8)`                        | 1   |
| Step 1 | Bootstrap split      | 1 plateau, depth 1: `[0,4) [4,8)`                  | 1   |
| Step 2 | Catalytic split of L | 2 plateaus: depth 2 `[0,2) [2,4)`, depth 1 `[4,8)` | 2   |
| Step 3 | Catalytic split of R | 1 plateau, depth 2: `[0,2) [2,4) [4,6) [6,8)`      | 1   |

---

#### 16.3.2 observe(3, 10) — Bootstrap Split

**Route:** G-root is terminal. Receiver $= [0, 8)$.

**Accumulate:** $[0,8).\text{own} = 10$. $\text{root.entry.int} = 10$.

**G-propagation:** $[0,8).\text{sum} = 10$.

**Split check:** $10 > \theta = 5$, range $8 > 1$, entry is V-root.
**Bootstrap split** (§10.3).

**Resulting V-Tree neighbourhood:**

```
SR (2-node, int=10)
├── root.entry (int=10, exposed=F) ← FROZEN
└── cs (2-node, int=0)
    ├── L.entry (int=0, exposed=T) [0,4)
    └── R.entry (int=0, exposed=T) [4,8)
```

root.entry is now **frozen** at 10: both G-children exist, so all future
observations to $[0, 8)$ route to children. Its exposed flag is false —
eviction-immune.

**V-I3 spot-check:** $L(0)$ uncle $\text{root}(10)$: $0 \leq 10$. $\checkmark$.
$R(0)$ uncle $\text{root}(10)$: $0 \leq 10$. $\checkmark$. No violations.
[P2: violation-free insertion — new entries at $\nu$ cannot exceed any uncle,
since $\nu \preceq a$ for all $a$.]

---

#### 16.3.3 observe(3, 15) — Catalytic Split and Promotion

**Route:** $[0,8) \to [0,4) = L$ (terminal). Receiver $= L$.

**Accumulate:** $L.\text{own} = 15$. $L.\text{entry.int} = 15$.

**V-sums:** $\text{cs.int} = 15 + 0 = 15$. $SR.\text{int} = 10 + 15 = 25$.
[P5: associativity — multi-level structural aggregation is correct regardless
of parenthesisation.]

**G-propagation:** $[0,4).\text{sum} = 15$. $[0,8).\text{sum} = 25$.

**Split check for $L$:** $15 > 5$, range $[0,4)$ width $= 4 > 1$. Parent cs
is 2-node. $\text{depth}_V(L.\text{entry}) = 2 \leq D_{\text{create}} = 3$.
$\checkmark$. **Catalytic split** (§10.2).

Create $\text{LL} = [0, 2)$ sum=0, $\text{LR} = [2, 4)$ sum=0. Create
structural $s_1$ holding LL.entry and LR.entry. Add $s_1$ as third child of
cs. $L$'s exposed flag flips to false.
[P3: $\nu \oplus \nu = \nu$ — new structural node's importance equals ground;
P2: new entries at importance $\nu \preceq$ all existing — catalytic split is
violation-free.]

**Post-split neighbourhood:**

```
cs (3-node, int=15)
├── L.entry (int=15, exposed=F)   ← FROZEN, eviction-immune
├── R.entry (int=0, exposed=T)
└── s₁ (2-node, int=0)
    ├── LL.entry (int=0, exposed=T)
    └── LR.entry (int=0, exposed=T)
```

**Violation detected:** $L(15)$ has parent cs, grandparent SR. Uncle $=
\text{root.entry}(10)$. $15 > 10$ → **VIOLATION.**

> _Timing note._ This violation is first detected during **Step 3c**
> (the ancestor violation walk after V-sum propagation), _before_ the
> catalytic split in Step 5. The post-split checks in §10.1
> (`push_promoted_violations`) would also find it, producing a duplicate
> entry in the violation queue (harmless per §11.1.2's idempotent
> dequeue). The consolidated presentation below resolves it after the
> split for narrative clarity.

This is the competitive mechanism at work: $L$ has accumulated more than the
frozen benchmark set by the root at split time.

**resolve($L$):**

_Phase 1:_ $p = \text{cs}$ is a 3-node. Contract cs, isolate heaviest child
$L(15)$. Merge $R(0)$ and $s_1(0)$ into $m_1(0)$.

```
cs (2-node, int=15)
├── L.entry (int=15, exposed=F)
└── m₁ (2-node, int=0, has_evictable=T)
    ├── R.entry (int=0, exposed=T)
    └── s₁ (2-node, int=0, has_evictable=T)
        ├── LL.entry (int=0, exposed=T)
        └── LR.entry (int=0, exposed=T)
```

Side-effect check: children of $m_1 = \{R(0), s_1(0)\}$, uncle $= L(15)$.
$0 \leq 15$. $\checkmark$. Grandchildren of $m_1$: children of $s_1 =
\{LL(0), LR(0)\}$, uncle $= R(0)$. $0 \leq 0$. $\checkmark$.

Still violated: $L(15) > \text{root}(10)$.

_Phase 2:_ $L$ is an entry (0 V-children). $g = SR$ is a 2-node.
**skip_promote($L$)** (§11.5). Destroy cs. SR becomes a 3-node.

**Post-promote neighbourhood:**

```
SR (3-node, int=25, has_evictable=T)
├── L.entry (int=15, exposed=F)
├── m₁ (2-node, int=0, has_evictable=T)
│   ├── R.entry (int=0, exposed=T)
│   └── s₁ (2-node, int=0, has_evictable=T)
│       ├── LL.entry (int=0, exposed=T)
│       └── LR.entry (int=0, exposed=T)
└── root.entry (int=10, exposed=F)
```

Side-effect check: children of $m_1$ and $\text{root.entry}$ — all have
uncles $\{L(15), \text{root}(10)\}$ or $\{L(15), m_1(0)\}$. All shielded by
$L(15)$. $\checkmark$. $L$ now at depth 1 with no grandparent — no
violation possible.

**No more violations.** One skip-promote resolved the violation. $L$ rose
from depth 2 to depth 1 by outgrowing the frozen root benchmark.

**State after §16.3.3:**

```
G-Tree:                              V-Tree:
    [0,8) sum=25, own=10                 SR (3-node, int=25, has_evictable=T)
    /            \                       ├── L.entry (15, exposed=F) d=1 — frozen
[0,4) s=15    [4,8) s=0                 ├── m₁ (2-node, int=0, has_evictable=T)
 own=15         own=0                    │   ├── R.entry (0, exposed=T) d=2
 /     \                                 │   └── s₁ (2-node, int=0, has_evictable=T)
[0,2) [2,4)                              │       ├── LL.entry (0, exposed=T) d=3
 s=0   s=0                               │       └── LR.entry (0, exposed=T) d=3
 own=0 own=0                             └── root.entry (10, exposed=F) d=1 — frozen
```

---

#### 16.3.4 observe(6, 8) — Uncle Shield

**Route:** $[0,8) \to [4,8) = R$ (terminal). Receiver $= R$.

**Accumulate:** $R.\text{own} = 8$. $R.\text{entry.int} = 8$.

**V-sums:** $m_1.\text{int} = 8 + 0 = 8$. $SR.\text{int} = 15 + 8 + 10 = 33$.

**G-propagation:** $[4,8).\text{sum} = 8$. $[0,8).\text{sum} = 33$.

**Split check for $R$:** $8 > 5$, range $[4,8)$ width $= 4 > 1$. Parent
$m_1$ is a 2-node. $\text{depth}_V(R.\text{entry}) = 2 \leq D_{\text{create}} = 3$.
$\checkmark$. **Catalytic split** (§10.2).

Create $\text{RL} = [4, 6)$ sum=0, $\text{RR} = [6, 8)$ sum=0. Create $s_2$
holding RL.entry and RR.entry. Add $s_2$ as third child of $m_1$. $R$'s
exposed flag flips to false.

**Post-split neighbourhood:**

```
m₁ (3-node, int=8, has_evictable=T)
├── R.entry (int=8, exposed=F)    ← FROZEN, eviction-immune
├── s₁ (2-node, int=0, has_evictable=T)
│   ├── LL.entry (int=0, exposed=T)
│   └── LR.entry (int=0, exposed=T)
└── s₂ (2-node, int=0, has_evictable=T)
    ├── RL.entry (int=0, exposed=T)
    └── RR.entry (int=0, exposed=T)
```

**V-I3 spot-check:** $R(8)$ has parent $m_1$, grandparent SR. Uncles $= \{L(15),
\text{root}(10)\}$. Max uncle $= 15$. $8 \leq 15$. $\checkmark$.
**No violation.** $R$ is shielded by uncle $L(15)$.

**The uncle shield in action.** $L(15)$ protects $R(8)$ from triggering a
violation despite $R$ being larger than root.entry(10). The max-uncle
formulation means a single strong uncle is sufficient.

---

#### 16.3.5 Resulting State

```
G-Tree:                                  V-Tree:
    [0,8) sum=33, own=10                     SR (3-node, int=33, has_evictable=T)
    /            \                           ├── L.entry (15, exposed=F) d=1 — frozen
[0,4) s=15    [4,8) s=8                     ├── m₁ (3-node, int=8, has_evictable=T)
 own=15         own=8                        │   ├── R.entry (8, exposed=F) d=2 — frozen
 /     \        /     \                      │   ├── s₁ (2-node, int=0, has_evictable=T)
[0,2) [2,4)  [4,6) [6,8)                    │   │   ├── LL.entry (0, exposed=T) d=3
 s=0   s=0    s=0   s=0                     │   │   └── LR.entry (0, exposed=T) d=3
 own=0 own=0  own=0 own=0                   │   └── s₂ (2-node, int=0, has_evictable=T)
                                             │       ├── RL.entry (0, exposed=T) d=3
                                             │       └── RR.entry (0, exposed=T) d=3
                                             └── root.entry (10, exposed=F) d=1 — frozen
```

---

#### 16.3.6 Invariant Verification

_G-I1:_ $[0,8).\text{sum} = 10 + 15 + 8 = 33$. $[0,4).\text{sum} = 15 + 0 + 0 = 15$. $[4,8).\text{sum} = 8 + 0 + 0 = 8$. $\checkmark$

_G-I4:_ root.entry.int $= 10 = [0,8).\text{own}$. L.entry.int $= 15 =
[0,4).\text{own}$. R.entry.int $= 8 = [4,8).\text{own}$. All zero entries
match. $\checkmark$ (Standard configuration [P0–P5]; under absolute
projection, `entry.int` tracks absolute accumulation — see §8.6.)

_V-I1:_ $s_1.\text{int} = 0 + 0 = 0$. $s_2.\text{int} = 0 + 0 = 0$. $m_1.\text{int} = 8 + 0 + 0 = 8$. $SR.\text{int} = 15 + 8 + 10 = 33$. $\checkmark$

_V-I3:_

| Node     | Depth | Parent | Grandparent | Uncles            | Max Uncle | Satisfied? |
| -------- | ----- | ------ | ----------- | ----------------- | --------- | ---------- |
| L(15)    | 1     | SR     | —           | —                 | —         | No check   |
| m₁(8)    | 1     | SR     | —           | —                 | —         | No check   |
| root(10) | 1     | SR     | —           | —                 | —         | No check   |
| R(8)     | 2     | m₁     | SR          | {L(15), root(10)} | 15        | 8 ≤ 15 ✓   |
| s₁(0)    | 2     | m₁     | SR          | {L(15), root(10)} | 15        | 0 ≤ 15 ✓   |
| s₂(0)    | 2     | m₁     | SR          | {L(15), root(10)} | 15        | 0 ≤ 15 ✓   |
| LL(0)    | 3     | s₁     | m₁          | {R(8), s₂(0)}     | 8         | 0 ≤ 8 ✓    |
| LR(0)    | 3     | s₁     | m₁          | {R(8), s₂(0)}     | 8         | 0 ≤ 8 ✓    |
| RL(0)    | 3     | s₂     | m₁          | {R(8), s₁(0)}     | 8         | 0 ≤ 8 ✓    |
| RR(0)    | 3     | s₂     | m₁          | {R(8), s₁(0)}     | 8         | 0 ≤ 8 ✓    |

All V-I3 satisfied. $\checkmark$

_V-I6:_ Exposed flags: root.entry(F, has 2 G-children), L.entry(F, has 2
G-children), R.entry(F, has 2 G-children), LL(T), LR(T), RL(T), RR(T).
$\checkmark$

_V-I7:_ SR.has_evictable = T. m₁.has_evictable = T. s₁.has_evictable = T.
s₂.has_evictable = T. $\checkmark$

_Conservation:_ V-entry sum $= 15 + 8 + 0 + 0 + 0 + 0 + 10 = 33 = \text{G-root sum}$.
$\checkmark$ — clean accounting, no double-counting (standard configuration
identity; under absolute projection, the total equals $\sum |\Delta_i|$).

---

#### 16.3.7 Remarks

**Remark 3.1 — Competitive mechanism fired.** In §16.3.3, $L(15)$ outgrew
the frozen root.entry$(10)$ and was promoted from depth 2 to depth 1 via
skip-promote. The child proved more significant than the parent's pre-split
benchmark.

**Remark 3.2 — Uncle shield held.** In §16.3.4, $R(8)$ injected significant
intensity but caused no violation because uncle $L(15)$ is stronger. The
V-Tree remained stable despite the injection — restructuring fires only when
a node outgrows its _entire_ neighbourhood.

**Remark 3.3 — Frozen entries are fixed, eviction-immune benchmarks.**
root.entry$(10)$, L.entry$(15)$, and R.entry$(8)$ are frozen at their
pre-split values. No subsequent observation can change them because their
G-children intercept all traffic. Their exposed flags are false — they
cannot be evicted while they have dependents.

**Remark 3.4 — Only unprotected entries are evictable.** Of the 7 entries,
only LL, LR, RL, and RR (all at depth 3, all fully exposed) would be
eviction candidates — but $3 \leq D_{\text{evict}} = 6$, so none are
evicted.

**Remark 3.5 — No information lost.** $[0,8).\text{sum} = 33$ reflects all
observations. Pre-split history lives in $g.\text{own}$ at each level.

**Remark 3.6 — Rebalancing economy.** Across three steps with one bootstrap
split and two catalytic splits, only §16.3.3 triggered rebalancing. Steps
§16.3.2 and §16.3.4 were absorbed by the uncle shield without any
restructuring.

**Remark 3.7 — Proportional sampling.** At $SR(33)$: choose $L(15)$ with
$p = 15/33 \approx 0.45$, $m_1(8)$ with $p \approx 0.24$, root$(10)$ with
$p \approx 0.30$. [P1: bounded below — proportional sampling well-defined;
non-negative ratios are meaningful.] The top two entries ($L$ and root,
total $25/33 \approx 0.76$) are both at depth 1, reached in one step. If
$m_1$ is chosen, one more step reaches $R(8)$ at depth 2. Expected cost
$\approx 1.24$ steps. Every entry reached is guaranteed live — no ghost
checks needed.

**Remark 3.8 — Steps 6b and 7 elided.** Each `observe()` call concludes
with Step 6b (depth adjustment) and Step 7 (`check_evictions`). With
$D_{\text{evict}} = 6$ and all entries at depth $\leq 3$, both are no-ops
throughout the observation phase.

---

### 16.4 Eviction Phase

#### 16.4.1 Overview

To exercise the eviction path, temporarily set $D_{\text{evict}} = 1$ and
$D_{\text{create}} = 0$ (maintaining D-I3: $D_{\text{create}} < D_{\text{evict}}$
with buffer $= 1 \geq 1$; no splits are attempted during the eviction
examples, so $D_{\text{create}} = 0$ has no operational effect).

With $D_{\text{evict}} = 1$, all four terminal entries (LL, LR, RL, RR)
satisfy the depth condition $3 > 1$ and appear in the Phase 1 snapshot. In
practice, `check_evictions` would process the entire batch in a single
Phase 2 sweep. Two evictions are traced individually to exercise both
structural cases of §12.5 Step 6:

| Eviction | Target | Structural case | V-parent transition                      |
| -------- | ------ | --------------- | ---------------------------------------- |
| §16.4.2  | LL     | 2-node collapse | $s_1$ destroyed, LR re-parented to $m_1$ |
| §16.4.3  | LR     | 3→2 transition  | $m_1$ shrinks from 3 to 2 children       |

Both are ghost evictions ($g.\text{importance} = \nu = 0$). Remark 4.7
(§16.4.6) addresses what this leaves unexercised.

The remaining candidates (RL, RR) exercise the same machinery and are
omitted for brevity.

**Contour trajectory (continued):**

| Step     | Event    | Contour state                                            | $P$ |
| -------- | -------- | -------------------------------------------------------- | --- |
| Evict LL | Collapse | 2 plateaus: depth 1 `[0,2)`, depth 2 `[2,4) [4,6) [6,8)` | 2   |
| Evict LR | 3→2      | 2 plateaus: depth 1 `[0,4)`, depth 2 `[4,6) [6,8)`       | 2   |

---

#### 16.4.2 Evict LL — 2-Node Collapse

Entry $LL(0)$ is at V-depth $3 > D_{\text{evict}} = 1$, is terminal (no
G-children), and its `is_evictable` flag is true. It qualifies.

**Step 1 (absorption).** Parent $[0,4)$: $\text{own} = 15 + 0 = 15$
(unchanged — LL had sum $= 0$). Detach LL from $[0,4).\text{left}$.

**Step 2 (ghost fast path).** $LL.\text{importance} = \nu = 0$ — skip
Steps 3–4. [P4: $\nu$ is identity —
$p.\text{importance} \oplus \nu = p.\text{importance}$; absorption is a
no-op.]

> _What would go wrong without the ghost fast path [P4]:_ Evicting LL would
> walk to the V-root three times (Step 3's `propagate_v_sums`, Step 4's
> ancestor violation walk, and `vtree_remove_leaf`'s propagation) —
> $O(h_V)$ work for zero effect at each walk. With P4, Steps 3–4 are
> skipped entirely, reducing ghost eviction to $O(1)$.

**Step 5 (flags).** $L.\text{entry.is\_exposed} \leftarrow \text{true}$
(was false, now semi-internal — uncovered range $[0,2)$ exposed).
$L.\text{entry.is\_evictable} \leftarrow \text{false}$ (still has G-child
$[2,4)$). Propagate `has_evictable` upward.

> _What would go wrong without the structural shield:_ If eviction could
> remove entries with dependents, evicting $[0,4)$ in the same pass would
> destroy a G-node whose child $[2,4)$ depends on it — tearing the contour.
> The dependents check prevents this. The one-tide lag (§12.6 design note)
> means $[0,4)$ was not in the Phase 1 snapshot and survives until the next
> `check_evictions` call.

**Step 6 (pre-capture).** $v\_\text{parent} = s_1$ (2-node).
$\text{child\_count} = 2$. $\text{collapse\_sibling} = LR.\text{entry}$.
$\text{change\_point} = m_1$.

**Step 7 (V-tree removal).** `vtree_remove_leaf(LL.entry)`. $s_1$ is a
2-node, collapses: $LR.\text{entry}$ is re-parented to $m_1$. $s_1$
destroyed.

**Post-removal V-Tree neighbourhood:**

```
SR (3-node, int=33)
├── L.entry (15, exposed=T, evictable=F) d=1 — UNFROZEN (semi-internal)
├── m₁ (3-node, int=8)
│   ├── R.entry (8, exposed=F) d=2 — frozen
│   ├── LR.entry (0, exposed=T, evictable=T) d=2
│   └── s₂ (2-node, int=0)
│       ├── RL.entry (0, exposed=T, evictable=T) d=3
│       └── RR.entry (0, exposed=T, evictable=T) d=3
└── root.entry (10, exposed=F) d=1 — frozen
```

**Step 8 (violation push).** Collapse case.

- `push_leaf_removal_violations(m₁)` (source 6): walks $m_1 \to SR$. At
  $m_1$: siblings $\{L.\text{entry}(15),\, \text{root.entry}(10)\}$ — both
  entries with no V-children, nothing to check. At $SR$: no parent, stop.

- `push_collapse_violations(LR.entry)` (source 7): $LR$ is an entry (no
  V-children). Nothing pushed.

- `push_cousin_violations(LR.entry, m₁)` (source 9): children of $m_1$'s
  children other than $LR.\text{entry}$. $R.\text{entry}(8)$: entry, no
  V-children → none. $s_2(0)$: children $\{RL(0),\, RR(0)\}$.
  `is_violated(RL)`: parent $s_2$, grandparent $m_1$, uncles
  $\{R(8),\, LR(0)\}$, max uncle $= 8$. $0 \leq 8$. $\checkmark$. Same
  for $RR$. No violations.

**Step 9 (plateau map).** Before eviction, all four terminals sat at
contour depth 2 — a single plateau covering $[0, 8)$ with sole basis
element $\{[0,8)\}$ (fully balanced tree, §5.6.1), $P.\text{sum} =
[0,8).\text{sum} = 33$. Evicting $[0,2)$ breaks
this uniformity: the contour now reads depth 1 at $[0,2)$ (covered by
semi-internal $[0,4)$) and depth 2 at $[2,4), [4,6), [6,8)$. Two plateaus
result:

- $P_1$: key $= 0$, depth 1, basis $= \{[0,4)\}$ (semi-internal;
  tile $= [0,2)$, span $= [0,4)$). $P_1.\text{sum} = [0,4).\text{sum} = 15$.
- $P_2$: key $= 2$, depth 2, basis $= \{[2,4),\, [4,8)\}$.
  $P_2.\text{sum} = 0 + 8 = 8$.

$P_1$'s basis element $[0,4)$ has span $[0,4)$ extending into $P_2$'s
territory at $[2,4)$ — one-hop thatching (§5.6.3, P-I4). [P-I1:
deterministic tiling — the two plateau keys $\{0, 2\}$ are exactly the
contour step coordinates where depth transitions.]

> _Plateau sum cross-check._ $P_1.\text{sum} + P_2.\text{sum} = 15 + 8 = 23
> \neq 33 = G\text{-root.sum}$. The difference (10) is $[0,8).\text{own}$ —
> the G-root's pre-split energy. Since $[0,8)$ spans both plateaus, it is
> not a basis element of either (P-I2 condition 1 fails — its contour
> includes both depth 1 and depth 2). Its `own` energy is not attributed to
> any plateau. This is expected: plateau sums account for structural energy
> within each plateau's basis, not total domain energy. The total-energy
> invariant ($\sum \text{V-entry importances} = G\text{-root.sum} = 33$) is
> maintained by the V-Tree; the plateau projection serves structural
> analysis (§5.6.3).

**Step 10 (deallocation).** Destroy LL's V-entry and G-node.

**Intermediate state and invariant verification:**

```
G-Tree:
    [0,8) sum=33, own=10
    /            \
[0,4) s=15    [4,8) s=8        ← [0,4) is now semi-internal
 own=15         own=8
    \           /     \         ← left child [0,2) gone
  [2,4)     [4,6) [6,8)
   s=0       s=0   s=0
   own=0     own=0 own=0
```

_G-I1:_ $[0,4).\text{sum} = 15 + 0 = 15$ (own=15, sole child
$[2,4).\text{sum}=0$). $[4,8).\text{sum} = 8 + 0 + 0 = 8$.
$[0,8).\text{sum} = 10 + 15 + 8 = 33$. $\checkmark$

_G-I4:_ L.entry.int $= 15 = [0,4).\text{own}$. R.entry.int $= 8 =
[4,8).\text{own}$. root.entry.int $= 10 = [0,8).\text{own}$. LR.entry.int
$= 0 = [2,4).\text{own}$. RL.entry.int $= 0 = [4,6).\text{own}$. RR.entry.int
$= 0 = [6,8).\text{own}$. $\checkmark$

_V-I1:_ $s_2.\text{int} = 0 + 0 = 0$. $m_1.\text{int} = 8 + 0 + 0 = 8$.
$SR.\text{int} = 15 + 8 + 10 = 33$. $\checkmark$

_V-I3:_

| Node     | Depth | Parent | Grandparent | Uncles            | Max Uncle | Satisfied? |
| -------- | ----- | ------ | ----------- | ----------------- | --------- | ---------- |
| L(15)    | 1     | SR     | —           | —                 | —         | No check   |
| m₁(8)    | 1     | SR     | —           | —                 | —         | No check   |
| root(10) | 1     | SR     | —           | —                 | —         | No check   |
| R(8)     | 2     | m₁     | SR          | {L(15), root(10)} | 15        | 8 ≤ 15 ✓   |
| LR(0)    | 2     | m₁     | SR          | {L(15), root(10)} | 15        | 0 ≤ 15 ✓   |
| s₂(0)    | 2     | m₁     | SR          | {L(15), root(10)} | 15        | 0 ≤ 15 ✓   |
| RL(0)    | 3     | s₂     | m₁          | {R(8), LR(0)}     | 8         | 0 ≤ 8 ✓    |
| RR(0)    | 3     | s₂     | m₁          | {R(8), LR(0)}     | 8         | 0 ≤ 8 ✓    |

All V-I3 satisfied. $\checkmark$

_V-I6:_ L.entry(T — semi-internal, uncovered range $[0,2)$). R.entry(F —
internal, has 2 G-children). root.entry(F — internal, has 2 G-children).
LR(T — terminal). RL(T — terminal). RR(T — terminal). $\checkmark$

_V-I6b:_ L.entry(F — has G-child $[2,4)$). R.entry(F — has 2 G-children).
root.entry(F — has 2 G-children). LR(T — no G-children). RL(T). RR(T).
$\checkmark$

_V-I7:_ $SR.\text{has\_evictable} = T$ ($m_1$ has evictable descendants).
$m_1.\text{has\_evictable} = T$ (LR is evictable; $s_2$ has evictable).
$s_2.\text{has\_evictable} = T$ (RL and RR are evictable). $\checkmark$

_Conservation:_ V-entry total $= 15 + 8 + 0 + 0 + 0 + 10 = 33 =$ G-root
sum. $\checkmark$

---

#### 16.4.3 Evict LR — 3→2 Transition

Continuing from the state after §16.4.2. Entry $LR(0)$ is at V-depth
$2 > D_{\text{evict}} = 1$, is terminal, and evictable.

> _Sibling sparing note._ With the original $D_{\text{evict}} = 2$, LR
> would be **spared**: the collapse-case eviction of LL reduced LR's
> V-depth from 3 to 2, and $2 > 2$ is false. This is the sibling-sparing
> mechanism of §12.6 — evicting one sibling rescues the other by collapsing
> their shared V-structural parent. With $D_{\text{evict}} = 1$, the rescue
> fails because $2 > 1$ still holds.

**Step 1 (absorption).** Parent $[0,4)$: $\text{own} = 15 + 0 = 15$
(unchanged — $[2,4).\text{sum} = 0$). Detach $[2,4)$ from
$[0,4).\text{right}$.

**Step 2 (ghost fast path).** $[2,4).\text{importance} = \nu = 0$ — skip
Steps 3–4. [P4: absorption of ghost importance is a no-op.]

**Step 5 (flags).** $L.\text{entry.is\_exposed}$ stays true (was already
semi-internal; now fully exposed — zero children, entire range $[0,4)$
uncovered). $L.\text{entry.is\_evictable} \leftarrow \text{true}$ ($[0,4)$
now has zero G-children). Despite `is_evictable = true`, $L$ is not
eviction-eligible because $\text{depth}_V = 1 \not> D_{\text{evict}} = 1$
(D-I2's depth condition is a separate guard). Propagate `has_evictable`
upward.

**Step 6 (pre-capture).** $v\_\text{parent} = m_1$ (3-node).
$\text{child\_count} = 3$. No collapse — 3→2 transition.
$\text{change\_point} = m_1$.

**Step 7 (V-tree removal).** `vtree_remove_leaf(LR.entry)`. $m_1$ shrinks
from 3-node to 2-node: children become $\{R.\text{entry},\, s_2\}$.

**Post-removal V-Tree neighbourhood:**

```
SR (3-node, int=33)
├── L.entry (15, exposed=T, evictable=T) d=1
├── m₁ (2-node, int=8)
│   ├── R.entry (8, exposed=F) d=2 — frozen
│   └── s₂ (2-node, int=0)
│       ├── RL.entry (0, exposed=T, evictable=T) d=3
│       └── RR.entry (0, exposed=T, evictable=T) d=3
└── root.entry (10, exposed=F) d=1 — frozen
```

**Step 8 (violation push).** 3→2 case.

- `push_leaf_removal_violations(m₁)` (source 6): walks $m_1 \to SR$. Same
  path as §16.4.2 — siblings are entries with no V-children. Nothing pushed.

- Inline 3→2 loop (source 8): remaining children of
  $m_1 = \{R.\text{entry},\, s_2\}$. $R.\text{entry}$: entry, no
  V-children → nothing. $s_2$: children $\{RL(0),\, RR(0)\}$.
  `is_violated(RL)`: parent $s_2$, grandparent $m_1$, uncles $= \{R(8)\}$,
  max uncle $= 8$. $0 \leq 8$. $\checkmark$. Same for $RR$.

The uncle set for $RL$ and $RR$ shrank from $\{R(8),\, LR(0)\}$ to
$\{R(8)\}$ — the max uncle is unchanged at $R(8)$. Removing $LR(0)$ (the
weaker uncle) cannot create violations. No violations pushed.

**Step 9 (plateau map).** $[2,4)$ was a basis element of $P_2$. After
eviction, $[0,4)$ covers $[2,4)$ at depth 1. Same accounting as §16.4.2 —
two plateaus remain:

- $P_1$: key $= 0$, depth 1, basis $= \{[0,4)\}$ (terminal;
  tile $= [0,4)$). $P_1.\text{sum} = 15$.
- $P_2$: key $= 4$, depth 2, basis $= \{[4,8)\}$ (balanced internal).
  $P_2.\text{sum} = 8$.

No thatching — both basis elements are non-semi-internal. $P_1 + P_2 = 23
\neq 33$; the gap of 10 is the G-root's unattributed `own`, as before.

**Step 10 (deallocation).** Destroy LR's V-entry and G-node.

**V-I3 spot-check:** $RL(0)$ at depth 3, parent $s_2$, grandparent $m_1$,
uncles $= \{R(8)\}$. $0 \leq 8$. $\checkmark$.

**Phase 3 (trailing rebalance).** The violation queue is empty after both
ghost evictions — neither Step 8 pushed any violations. `rebalance()`
returns immediately (no-op).

---

#### 16.4.4 Resulting State

```
G-Tree:                                  V-Tree:
    [0,8) sum=33, own=10                     SR (3-node, int=33)
    /            \                           ├── L.entry (15, exposed=T, evictable=T) d=1
[0,4) s=15    [4,8) s=8                     ├── m₁ (2-node, int=8)
 own=15         own=8                        │   ├── R.entry (8, exposed=F) d=2 — frozen
                /     \                      │   └── s₂ (2-node, int=0)
            [4,6) [6,8)                      │       ├── RL.entry (0, exposed=T) d=3
             s=0   s=0                       │       └── RR.entry (0, exposed=T) d=3
                                             └── root.entry (10, exposed=F) d=1 — frozen
```

---

#### 16.4.5 Invariant Verification

_G-I1:_ $[0,4).\text{sum} = \text{own} = 15$ (no children).
$[0,8).\text{sum} = 10 + 15 + 8 = 33$. $[4,8).\text{sum} = 8 + 0 + 0 = 8$.
$\checkmark$

_G-I4:_ L.entry.int $= 15 = [0,4).\text{own}$. R.entry.int $= 8 =
[4,8).\text{own}$. root.entry.int $= 10 = [0,8).\text{own}$. RL.entry.int
$= 0 = [4,6).\text{own}$. RR.entry.int $= 0 = [6,8).\text{own}$.
$\checkmark$

_V-I1:_ $s_2.\text{int} = 0 + 0 = 0$. $m_1.\text{int} = 8 + 0 = 8$.
$SR.\text{int} = 15 + 8 + 10 = 33$. $\checkmark$

_V-I3:_

| Node     | Depth | Parent | Grandparent | Uncles            | Max Uncle | Satisfied? |
| -------- | ----- | ------ | ----------- | ----------------- | --------- | ---------- |
| L(15)    | 1     | SR     | —           | —                 | —         | No check   |
| m₁(8)    | 1     | SR     | —           | —                 | —         | No check   |
| root(10) | 1     | SR     | —           | —                 | —         | No check   |
| R(8)     | 2     | m₁     | SR          | {L(15), root(10)} | 15        | 8 ≤ 15 ✓   |
| s₂(0)    | 2     | m₁     | SR          | {L(15), root(10)} | 15        | 0 ≤ 15 ✓   |
| RL(0)    | 3     | s₂     | m₁          | {R(8)}            | 8         | 0 ≤ 8 ✓    |
| RR(0)    | 3     | s₂     | m₁          | {R(8)}            | 8         | 0 ≤ 8 ✓    |

All V-I3 satisfied. $\checkmark$

_V-I6:_ L.entry(T — fully exposed, zero children).
R.entry(F — internal, has 2 G-children). root.entry(F — internal, has 2
G-children). RL(T — terminal). RR(T — terminal). $\checkmark$

_V-I6b:_ $L.\text{entry.is\_evictable} = \text{true}$.
$\neg\,\text{has\_dependents}([0,4)) = \text{true}$ (zero children).
$\checkmark$. (D-I2's depth condition $\text{depth}_V = 1 \not> 1$
separately prevents eviction.)

_V-I7:_ $SR.\text{has\_evictable} = T$ ($m_1$ has evictable descendants).
$m_1.\text{has\_evictable} = T$ ($s_2$ has evictable descendants).
$s_2.\text{has\_evictable} = T$ (RL and RR are evictable). $\checkmark$

_Conservation:_ V-entry total $= 15 + 8 + 0 + 0 + 10 = 33 =$ G-root sum.
$\checkmark$ (Standard configuration identity [P0–P5].)

---

#### 16.4.6 Remarks

**Remark 4.1 — Semi-internal transition.** After §16.4.2, the parent
$[0,4)$ absorbed LL's value (trivially 0 here) and became semi-internal —
it receives observations in $[0,2)$ while $[2,4)$ remains shielded by its
surviving child. After §16.4.3, $[0,4)$ became fully terminal.

**Remark 4.2 — Structural collapse.** In §16.4.2, the V-structural node
$s_1$ collapsed during removal, re-parenting $LR$ directly to $m_1$. V-I2
maintained throughout.

**Remark 4.3 — Contour retreat.** Two evictions removed two depth-2 cells
from the left half of the domain. The parent $[0,4)$ now serves the entire
$[0,4)$ range at depth 1 — one level shallower, with total energy
preserved. The right half's structure at depth 2 is untouched.

**Remark 4.4 — 3→2 vs. collapse.** In §16.4.2, the 2-node $s_1$ was
destroyed and its sole surviving child re-parented — scorched earth. In
§16.4.3, the 3-node $m_1$ shrank to a 2-node but survived. The uncle set
for $RL$ and $RR$ shrank, but the max uncle ($R$ at 8) was unchanged —
removing the weaker uncle $LR(0)$ cannot weaken the V-I3 shield.

**Remark 4.5 — Flag vs. eligibility.** After §16.4.3,
$L.\text{entry.is\_evictable} = \text{true}$ because $[0,4)$ has zero
G-children. But $L$ is not eviction-eligible: the depth gate
($1 \not> 1$) is a separate guard checked during the eviction scan, not
cached in this flag. The `is_evictable` flag caches one condition (§4.2);
eviction requires all three conditions of D-I2.

**Remark 4.6 — Sibling sparing is order-dependent.** With
$D_{\text{evict}} = 2$, evicting LL first would spare LR (depth drops from
3 to 2 via collapse); evicting LR first would spare LL instead. With
$D_{\text{evict}} = 1$, neither sibling is spared. The final state after
`check_evictions` may differ depending on eviction order — both outcomes
are valid (§12.6).

**Remark 4.7 — Ghost eviction coverage gap.** Both evictions evicted entries
with zero importance. The ghost fast path [P4] skipped Steps 3–4 of §12.5
in both cases, leaving two significant parts of the eviction machinery
unexercised:

- **Step 3** (importance absorption + V-sum propagation): when the evicted
  entry has non-zero importance, the parent's V-entry strengthens, and
  `propagate_v_sums` walks to the V-root updating structural aggregates.
- **Step 4** (ancestor-walk violation detection): the strengthened parent
  may exceed its own uncle, and the Two-Path Coverage Lemma (§12.5.1)
  guarantees that Steps 4 and 8 together detect every genuine violation.

A non-ghost eviction — where the evicted entry has accumulated positive
importance — would demonstrate the full absorption flow, the transient V-I1
inflation between Steps 3 and 7, and the interaction between
importance-increase violations (Step 4) and structural-removal violations
(Step 8). This would occur naturally if several observations were directed
at one of the terminal cells before the eviction pass, pushing its
importance above zero while it remains past $D_{\text{evict}}$.

---

## Chapter 17. Properties

### 17.1 Invariant Preservation Summary

| Operation                       | V-I3 (Uncle)                                                                                                                                                                                                                                                             | V-I5 (Entry-Leaf) | V-I6/7 (`is_exposed`/`has_evictable`) | G-I1 (Sums)  | G-I4 (Consistency) |
| ------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ----------------- | ------------------------------------- | ------------ | ------------------ |
| Observation (importance update) | May violate entry AND structural ancestors (§11.11.2)                                                                                                                                                                                                                    | $\checkmark$      | $\checkmark$                          | $\checkmark$ | $\checkmark$       |
| V-Tree insertion                | Preserved                                                                                                                                                                                                                                                                | $\checkmark$      | $\checkmark$                          | —            | $\checkmark$       |
| V-Tree leaf removal             | Importance decrease propagates to all ancestors, weakening uncle coverage at every level. Ancestor walk checks siblings' children (§11.11.3, source 6). Collapse and 3→2 transitions require additional checks (sources 7–9, §11.11.3).                                  | $\checkmark$      | $\checkmark$                          | —            | $\checkmark$       |
| Bootstrap split                 | Preserved                                                                                                                                                                                                                                                                | $\checkmark$      | $\checkmark$                          | $\checkmark$ | $\checkmark$       |
| Catalytic split                 | **Preserved** (P2 + P3); side-effects queued by §10.1 otherwise                                                                                                                                                                                                          | $\checkmark$      | $\checkmark$                          | $\checkmark$ | $\checkmark$       |
| Standard promote                | May create side-effects at grandchildren (§11.11.1) and children (§11.11.1); may require escalation (§11.10)                                                                                                                                                             | $\checkmark$      | $\checkmark$                          | —            | —                  |
| Skip promote                    | Resolves trigger; may create side-effects at grandchildren (§11.11.1) and children (§11.11.1)                                                                                                                                                                            | $\checkmark$      | $\checkmark$                          | —            | —                  |
| Legacy promote                  | Side-effects at $u$'s children (§11.11.1). Same class as skip promote.                                                                                                                                                                                                   | $\checkmark$      | $\checkmark$                          | $\checkmark$ | $\checkmark$       |
| Contraction                     | May create violations at merged-node grandchildren (§11.11.1), children (§11.11.1), and siblings of target (§11.11.1)                                                                                                                                                    | $\checkmark$      | $\checkmark$                          | —            | —                  |
| Eviction                        | Importance update may violate ancestors (source 1–2); leaf removal weakens uncle coverage at all ancestor levels (source 6, §11.11.3); collapse changes uncle context (sources 7, 9, §11.11.3); 3→2 reduces uncle set (source 8, §11.11.3). Trailing rebalance required. | $\checkmark$      | $\checkmark$                          | $\checkmark$ | $\checkmark$       |

> _Note._ G-I4 is a structural reference: the V-entry's importance is the
> G-node's importance accumulator, accessed through the opaque `Importance`
> interface. The invariant is maintained by updating `g.importance` at the
> single mutation site (`accumulate_importance` during observation,
> `accumulate_eviction` during eviction absorption). See §8.6, §8.7.

**Catalytic splits are violation-free** under the max-uncle constraint.
Observations are the root cause of all violations — both directly (entry
exceeds uncle) and indirectly (structural ancestors exceed their uncles
via propagated importance increases).

### 17.2 V-Tree Height

The uncle constraint permits a degenerate chain where $e_1.\text{int} \geq
e_2.\text{int} \geq \cdots$. Height may reach $O(L)$.

**This is not a bug.** It is the correct shape for a distribution where
every entry has distinct intensity and depth should reflect rank. Under
concentrated distributions, $h = O(\log L)$. Under uniform distributions,
$h = O(L)$ but proportional sampling still costs $O(1)$ per sample on
average since all entries have equal weight.

### 17.3 Write-Set Locality

Every observation's write set decomposes into two parts:

**Commutative writes.** G-Tree sum propagation (Step 4 of §8) and V-Tree
sum propagation (Step 3 of §8) perform addition along root-ward paths. Addition
is commutative and associative. Two observations incrementing a shared
ancestor node produce the correct result regardless of ordering, requiring
only atomic addition at each shared node.

**Local structural writes.** Violation detection (§11.2) reads a
constant-size neighborhood: the node, its parent, grandparent, and the
grandparent's children. Rebalancing (§11.3–11.6) writes to the same
bounded region — at most a grandparent and its descendants, roughly 10
nodes. A split (§10.2) writes to the splitting node's V-parent and creates
a constant number of new nodes. An eviction (§12.6) writes to the evicted
node's V-parent and G-parent.

**Consequence.** Two observations hitting different G-Tree leaves whose
V-entries do not share a grandparent have completely non-overlapping
structural write sets. Their violation checks and rebalancing operations
modify disjoint node sets and can proceed without coordination.

The rebalance loop (§11.8) is correct under any pop order — `is_violated` is re-checked before acting, filtering stale entries. A deepest-first heuristic can prevent higher-level restructurings from disturbing lower-level corrections, but is not required. Non-conflicting violations — those whose resolution neighbourhoods are disjoint — may be resolved simultaneously, analogous to concurrent B-tree splits that lock only the affected neighbourhood.

The uncle constraint's locality is the enabling property. A violation at
V-depth $d$ is resolved by touching nodes at depths $d$, $d{-}1$, and
$d{-}2$. The root is never involved. Subtrees on the opposite side of the
V-Tree are never disturbed. Compare to globally-balanced structures where
rotations may propagate to the root, serializing all concurrent
modifications.

**Contention points under concurrent operation:**

| Source                                    | Region                                | Mechanism                                |
| ----------------------------------------- | ------------------------------------- | ---------------------------------------- |
| Sum propagation at shared G-ancestors     | One node per shared ancestor          | Atomic addition                          |
| Sum propagation at shared V-ancestors     | One node per shared ancestor          | Atomic addition                          |
| Nearby V-Structural modifications         | Grandparent + descendants (~10 nodes) | Fine-grained locking or optimistic retry |
| Split/eviction overlapping with rebalance | One V-neighborhood                    | Fine-grained locking                     |

Expected contention under random observation patterns scales inversely with
tree size: in a V-Tree with $L$ entries, the probability that two random
observations trigger rebalancing in overlapping neighborhoods is $O(1/L)$.

---

## Chapter 18. Complexity Analysis

### 18.1 The Main Theorem: Fibonacci Depth Bound

> _Precondition._ This proof requires P1 (bounded below) — that is, the
> carrier $I$ has a known bottom element $\bot$ and all importance values
> satisfy $v.\text{int} \geq \bot$. The standard and absolute
> configurations satisfy this (P0–P5 hold, hence P1 holds); the signed
> configuration ($I = \mathbb{R}$) does not satisfy P1 — the Fibonacci
> bound does not apply in that configuration. P5 (associativity) is also
> required for the inductive step.

**Theorem.** In any V-Tree satisfying V-I3 and V-I2, an entry $v_i$ with
weight fraction $w_i = v_i.\text{int}\,/\,I_{\text{total}}$ has depth

$$d_i \;\leq\; \log_\phi\!\left(\frac{1}{w_i}\right) + 1$$

where $\phi = \frac{1+\sqrt{5}}{2} \approx 1.618$ is the golden ratio.

**Proof.** Let $v$ be a node at depth $d$. Denote the path from root to $v$
as $v_0, v_1, \ldots, v_d = v$ where $v_0$ is the V-Tree root and
$v_{k+1}$ is a child of $v_k$. Write $I_k = v_k.\text{int}$.

**Step 1: The Fibonacci recurrence.** For any $k \geq 2$, the triple
$(v_k,\, v_{k-1},\, v_{k-2})$ is child–parent–grandparent. V-I3 states
$v_k.\text{int} \leq \max\{u.\text{int} : u \in \text{siblings}(v_{k-1},\,
v_{k-2})\}$. If $v_k$ is not in violation, some uncle $u$ satisfies
$u.\text{int} \geq I_k$. Since $v_{k-2}$ has at least 2 children (V-I2),
$v_{k-1}$ has at least one sibling $u$ under $v_{k-2}$:

$$I_{k-2} \;=\; I_{k-1} \;+\; \sum_{\text{siblings}} u.\text{int} \;\geq\; I_{k-1} + I_k$$

The inequality uses the fact that additional siblings (in a 3-node) contribute non-negative importance. Under ordinary addition on $[0, \infty)$, P1 forces $m \geq 0$, so all importance values are $\geq 0$ and adding them cannot decrease the sum. This monotonicity step is specific to the additive case — it does not follow from P1 alone in the general algebraic framework.

This gives $I_{k-2} \geq I_{k-1} + I_k$ for all $k \geq 2$ — **the
Fibonacci recurrence** running backward from the leaf.

**Step 2: Unwinding the recurrence.** Define
$\alpha_k = I_k / I_d$ (normalised so $\alpha_d = 1$). Then:

- $\alpha_d = 1$
- $\alpha_{d-1} \geq 1$
- $\alpha_{k-2} \geq \alpha_{k-1} + \alpha_k$ for all $k \geq 2$

By induction, $\alpha_{d-j} \geq F_{j+1}$ where $F_n$ is the $n$-th
Fibonacci number ($F_1 = F_2 = 1$).

At the root: $\alpha_0 = I_0/I_d = I_{\text{total}}/v.\text{int} = 1/w_i$,
so:

$$\frac{1}{w_i} \;\geq\; F_{d+1}$$

**Step 3: Inverting the Fibonacci bound.** The standard bound $F_n \geq
\phi^{n-2}$ holds for all $n \geq 1$ (induction using $\phi^2 = \phi + 1$):

$$\frac{1}{w_i} \;\geq\; F_{d+1} \;\geq\; \phi^{d-1}$$

$$\boxed{\;d_i \;\leq\; \log_\phi\!\left(\frac{1}{w_i}\right) + 1\;}$$

$\blacksquare$

> _Remark (sharper constant)._ The bound $F_n \geq \phi^{n-2}$ is tight
> at $n = 2$ but loose for large $n$, where $F_n \sim \phi^n/\sqrt{5}$.
> Using Binet's formula, the Fibonacci chain construction (§18.3) achieves
> $d \approx \log_\phi(1/w_i) + \log_\phi\sqrt{5} - 1 \approx
> \log_\phi(1/w_i) + 0.67$, showing the additive constant could be
> tightened from $1$ to $\approx 0.67$. The leading coefficient is
> unaffected. Throughout this section, $c = 1$ is used — it follows from
> elementary induction with no appeal to Binet.

### 18.2 Expected Sampling Cost

**Corollary.**

$$E[\text{sampling}] \;=\; \sum_i w_i\, d_i \;\leq\; \frac{H}{\log_2 \phi} \;+\; 1$$

where $H = \sum_i w_i \log_2(1/w_i)$ is the Shannon entropy of the
intensity distribution and $1/\log_2\phi \approx 1.4404$.

**Proof.** Apply the depth bound entry-by-entry:

$$\sum_i w_i\, d_i \;\leq\; \sum_i w_i\!\left(\frac{\log_2(1/w_i)}{\log_2\phi} + 1\right) \;=\; \frac{H}{\log_2\phi} + 1$$

$\square$

The V-Tree is a **probability routing structure**, not a prefix code. At
each node, a child is chosen with probability proportional to its intensity.
There is no deterministic codeword for any entry. The expected traversal
cost scales with entropy because the uncle constraint ensures each
probabilistic step gains at least $\log_2\phi \approx 0.694$ bits of
information about which entry will be reached. The bound measures routing
efficiency, not code length.

### 18.3 Tightness: The Leading Coefficient Is Exact

The coefficient $1/\log_2\phi \approx 1.4404$ is achieved by a Fibonacci
chain construction: set $w_i \propto \phi^{-i}$ along a degenerate 2-node
chain. This chain satisfies all V-Tree invariants and every entry sits at
maximum permitted depth for its weight fraction. The uncle constraint is
satisfied with equality at every level: each node's intensity equals its
sole uncle's, and $I_{k-2} = I_{k-1} + I_k$ exactly.

### 18.4 The AVL Parallel

|                         | AVL Tree                                              | V-Tree                                            |
| ----------------------- | ----------------------------------------------------- | ------------------------------------------------- |
| **Balance constraint**  | Height-balance: children's heights differ by $\leq 1$ | Max-uncle: no grandchild outweighs heaviest uncle |
| **Nature**              | Local, structural                                     | Local, value-based                                |
| **Produces recurrence** | $N_h \geq N_{h-1} + N_{h-2} + 1$                      | $I_{k-2} \geq I_{k-1} + I_k$                      |
| **Growth rate**         | $\phi^h$                                              | $\phi^d$                                          |
| **Bound**               | Height $\leq \log_\phi n + O(1)$                      | Depth $\leq \log_\phi(1/w_i) + O(1)$              |
| **Overhead**            | $1/\log_2\phi \approx 1.44$                           | $1/\log_2\phi \approx 1.44$                       |

Both sacrifice $\approx 44\%$ overhead for the same reason: the local
constraint's tightest extremal configuration follows Fibonacci, and
$\log_2\phi \approx 0.694$ is the Fibonacci sequence's per-step entropy.

The parallel is structural, not superficial: both are local constraints
whose extremal analysis produces the same recurrence, yielding the same
constant, bounding different quantities (node count vs. weight fraction)
in the same way. But neither structure satisfies a Kraft inequality with
base $\phi$ — both overshoot, because both have minimum branching factor
2 while $\phi < 2$.

### 18.5 Information-Theoretic Framing

The V-Tree is a probability routing structure. Each step selects a child
with probability proportional to intensity. The efficiency question is:
how many steps are needed to identify the target entry?

**Lower bound.** Maximum information per step: $\log_2 3 \approx 1.585$
bits (at a 3-node, choosing among 3 children). To identify an entry from a
distribution with entropy $H$:

$$E[\text{sampling}] \;\geq\; \frac{H}{\log_2 3} \;\approx\; 0.631\, H$$

**Upper bound** (§18.2): $E[\text{sampling}] \leq 1.44\,H + 1$.

**Gap.**

$$\frac{\text{V-Tree worst case}}{\text{information-theoretic lower bound}} \;=\; \frac{1/\log_2\phi}{1/\log_2 3} \;=\; \frac{\log_2 3}{\log_2 \phi} \;=\; \log_\phi 3 \;\approx\; 2.28$$

The gap is the cost of maintaining the ranking with purely local operations
(the uncle constraint), versus a globally optimised structure (which
would require non-local restructuring on every update). The 2.28 factor
is the price of locality.

**Comparison with known proportional sampling schemes:**

| Scheme                        | Expected sampling cost  | Update cost           | Dynamic? |
| ----------------------------- | ----------------------- | --------------------- | -------- |
| Balanced segment tree         | $\Theta(\log L)$ always | $O(\log L)$           | Yes      |
| Static optimal tree (Huffman) | $\leq H + 1$            | $O(L)$ rebuild        | No       |
| Adaptive rank tree (Vitter)   | $\leq H + 1$ amortised  | $O(L)$ worst case     | Yes      |
| **V-Tree (uncle constraint)** | $\leq 1.44\, H + 1$     | $O(\log_\phi(1/w_i))$ | **Yes**  |

The V-Tree trades a 44% overhead on expected sampling cost for $O(\log_\phi(1/w_i))$ updates — polylogarithmic in the inverse weight fraction.
Under concentrated distributions, both sampling and update are $O(1)$.

### 18.6 Regime-Dependent Behaviour

| Regime                      | $H$         | $E[\text{sampling}]$ | Balanced tree    |
| --------------------------- | ----------- | -------------------- | ---------------- |
| Concentrated ($k$ hotspots) | $O(\log k)$ | $O(\log k)$          | $\Theta(\log L)$ |
| Uniform                     | $\log_2 L$  | $\leq 1.44\log_2 L$  | $\log_2 L$       |
| Zipf ($\alpha > 1$)         | $O(1)$      | $O(1)$               | $\Theta(\log L)$ |
| Geometric ($\beta^i$)       | $O(1)$      | $O(1)$               | $\Theta(\log L)$ |

### 18.7 The Two Roles: Code and Policy

The G-V Graph's coding-theoretic content separates cleanly across the two
trees.

**The G-Tree is a spatial code.** Its bottom contour satisfies a base-2
Kraft equality $\sum 2^{-d_i} = 1$ unconditionally — a structural
consequence of partitioning $[0, 2^N)$ into dyadic cells. The code
allocates precision toward distributional mass: deep cells (long codes)
where intensity is high, shallow cells (short codes) where it is low. This
is the rate-distortion dual of source coding.

**The V-Tree is the allocation policy.** It determines where the G-Tree
spends its precision budget, achieving entropy-sensitive governance at
$1.44\times$ overhead. The golden ratio lives here — in the efficiency
of the policy, not the structure of the code.

| Phase      | Structure      | Overhead     | Purpose                    | Coding role       |
| ---------- | -------------- | ------------ | -------------------------- | ----------------- |
| Attention  | V-Tree walk    | $1.44\times$ | Scale and region selection | Allocation policy |
| Resolution | G-Tree descent | $1.0\times$  | Cell-level refinement      | Description code  |

Spatial cost depends on $N$. Policy cost depends on $H$. These are
independent quantities.

### 18.8 Amortised Rebalancing Under Proportional Traffic

If observations arrive at entry $i$ with probability proportional to $w_i$:

$$E[\text{rebalance}] \;=\; \sum_i w_i \cdot O(d_i) \;=\; O\!\left(\frac{H}{\log_2 \phi}\right)$$

Under concentrated distributions, this is $O(1)$ amortised.

### 18.9 Cost Summary

| Operation                      | Cost                                                                                                                                                                       |
| ------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Observe (route + accumulate)   | $O(d_{\text{geo}})$                                                                                                                                                        |
| V-entry update per observation | $O(h_V)$ — single entry + V-sum propagation                                                                                                                                |
| Single promotion / contraction | $O(1)$ under P5; $O(h_V)$ without P5 (propagation required)                                                                                                                |
| Rebalance after injection      | $O(h_V)$ typical                                                                                                                                                           |
| Catalytic split                | $O(1)$ — violation-free [P2 + P3]; $O(h_V)$ if $\nu \oplus \nu \neq \nu$ (P3 fails)                                                                                        |
| Bootstrap split                | $O(1)$                                                                                                                                                                     |
| Eviction                       | $O(h_V + d_{\text{geo}} + \log P)$ — absorption $O(1)$, V-propagation + violation walks $O(h_V)$, plateau basis recalculation $O(d_{\text{geo}})$, plateau map $O(\log P)$ |
| Eviction scan                  | $O(E_t)$ where $E_t$ = unprotected entries past $D_{\text{evict}}$, pruned by flags                                                                                        |
| Range sum                      | $O(N)$                                                                                                                                                                     |
| Attention sample               | $O(1.44\,H + 1)$ expected                                                                                                                                                  |
| Point query (routing)          | $O(N)$ worst, $O(d_{\text{geo}})$ typical                                                                                                                                  |
| Point query (plateau)          | $O(\log P)$ where $P$ = plateau count                                                                                                                                      |

> _Note (P5 and amortised bounds)._ The amortised bounds assume P5. Without P5, each contraction or promotion incurs an additional $O(h_V)$ propagation. Since each rebalancing pass performs $O(1)$ rotations, the per-operation overhead is $O(h_V)$ — the same as existing propagation paths (e.g., after observation routing). The asymptotic amortised bounds are therefore unchanged, but the constant factor increases. Implementations targeting non-associative value spaces should account for this in their performance model.

### 18.10 Non-Additive Value Spaces

The property hierarchy P0–P5 is stated for general
$(I, \oplus, \nu, \preceq)$. Under ordinary addition, the hierarchy
collapses (§2.5) and admits essentially one fully-featured configuration.
Under non-additive operations, the properties genuinely separate:

- **Max-tropical semiring** $(\mathbb{R}_{\geq 0}, \max, 0, \leq)$: P0–P5
  all hold. Structural aggregation becomes max-aggregation. The Fibonacci
  bound's additive decomposition step fails; a replacement bound likely
  exists but is not developed here.

- **Multiplicative monoid** $(\mathbb{R}_{>0}, \times, 1, \leq)$: P0, P4,
  P5 hold. P1 holds if the carrier is $[1, \infty)$ (then P2 also holds).
  P3: $1 \times 1 = 1$. All properties hold on $[1, \infty)$ — another
  fully featured configuration. The Fibonacci proof would need
  multiplication-specific analysis.

These are noted as future directions. The current specification develops
the additive case only.

### 18.11 Recommended Test Configurations

The following non-standard configurations exercise specific property boundaries and serve as regression tests for implementations.

> **Shifted configuration** $([1, \infty),\; +,\; 1,\; \leq)$: Properties $\{\text{P0, P1, P2, P5}\}$. This configuration has P2 without P3 — ground is at the bottom but is not idempotent. Use as a regression test for:
>
> - Violation-free insertion: should work (P2 holds).
> - Ghost fast path: should **not** activate (P4 fails; $1 + a = a + 1 \neq a$).
> - Violation-free splits: structural node gets importance $1 + 1 = 2 \neq 1$. Check whether this creates violations.
> - Proportional sampling: should work (P1 holds).

> **Non-associative test configuration**: Define $a \oplus b = a + b + \min(a,\, b)$ on $[0, \infty)$ with $\nu = 0$ and $\preceq\; = \;\leq$. Properties: P0, P1, P2, P3, P4, but **not** P5. Verification: closure ($a, b \geq 0 \implies a + b + \min(a, b) \geq 0$), commutativity (symmetric), compatibility ($a \leq b \implies a + c + \min(a, c) \leq b + c + \min(b, c)$), P4 ($0 + a + \min(0, a) = a$). Non-associativity: $(1 \oplus 2) \oplus 3 = 4 \oplus 3 = 10$ but $1 \oplus (2 \oplus 3) = 1 \oplus 7 = 9$. Use as a regression test for:
>
> - Contraction/promotion: must trigger propagation (P5 fails).
> - Fibonacci depth bound: proof does not apply (P5 fails). Verify the bound is not assumed.

---

## Chapter 19. Spray Resistance

### 19.1 Defence Layers

| Layer                                  | Mechanism                                               | What it prevents                                           |
| -------------------------------------- | ------------------------------------------------------- | ---------------------------------------------------------- |
| Local intensity gate ($\theta$)        | Must exceed threshold to split                          | Low-intensity spray forcing splits                         |
| Global rank gate ($D_{\text{create}}$) | Must earn shallow V-position to split                   | Locally-intense but globally-insignificant nodes splitting |
| Parent arity gate                      | V-parent must be 2-node (preprocessing may deny split)  | Overcrowding at a V-level                                  |
| Eviction ($D_{\text{evict}}$)          | Unprotected contour cells past depth are evicted        | Accumulation of insignificant contour cells                |
| Structural immunity                    | Protected entries (have dependents) are eviction-immune | Premature removal of structurally load-bearing nodes       |

### 19.2 Behaviour Under Spray

Spray-generated entries are born adjacent to their G-parent with zero
intensity. Under diffuse spray, each entry carries a tiny fraction of global
intensity. The uncle constraint pushes them deep. Deep entries fail the
$D_{\text{create}}$ check and never split further. Unprotected contour cells
past $D_{\text{evict}}$ are evicted, their value absorbed by parents that
strengthen the uncle shield.

Protected contour cells (have dependents) whose children were created by
spray are eviction-immune while their children exist. But since their
unprotected children are evicted first, the protected entries
progressively lose children, become unprotected, and are evicted in
subsequent rounds. The contour coarsens from the spray-generated tips inward.

Semi-internal entries under spray: the uncovered half receives diffuse
observations, the entry's intensity grows slowly, and it remains deep
in the V-Tree because spray is globally insignificant. Legacy
promotion never fires for spray-generated semi-internals because they
never outgrow their uncles. The contour correctly refuses to regrow a
region that hasn't earned it.

### 19.3 Resource Bound

Under sustained spray of $R$ observations per batch at intensity $\Delta$,
with user-applied attenuation rate $\lambda \in (0, 1)$:

$$L_{\text{steady}} = \frac{R \cdot \Delta}{(1 - \lambda) \cdot \theta}$$

Independent of domain size, spray duration, and attack strategy. This formula assumes attenuation ($\lambda < 1$) as the contraction mechanism. Under annihilation ($\text{att} = 0$, §14.3), spray-generated nodes are immediately zeroed and evicted in the next tide — no steady-state accumulation occurs. Under amplification ($\text{att} > 1$), the formula does not apply; the dynamic $D_{\text{evict}}$ mechanism (§7.4) provides the hard ceiling.

**Recovery.** When the spray stops, user-applied attenuation continues. Unprotected
contour cells cool, sink past $D_{\text{evict}}$, and are evicted. Their
parents lose their last dependent, become unprotected, cool further, and are
evicted in turn. The tree returns to its pre-spray size through bottom-up
contraction (§12.7). Under annihilation, recovery is immediate: `decay(G_root, 0, 0)` zeroes the entire tree. The subsequent `check_evictions` pass removes all unprotected entries past $D_{\text{evict}}$. The tree returns to near-initialization state in a single call, rather than the gradual tidal erosion of attenuation-based recovery.

**The triad is necessary:**

| Mechanism                                      | Without it                                                                                                                                            |
| ---------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| Variable-depth leaves / depth gates            | Every unique address forces a leaf. G-Tree grows $O(N \cdot R \cdot T)$.                                                                              |
| Exponential attenuation (user) or annihilation | Intensity accumulates monotonically. Every entry eventually qualifies for splitting. (Amplification alone is not a defense — it worsens the problem.) |
| Eviction                                       | Cold entries persist after spray ends. Memory is never reclaimed.                                                                                     |

**Hard ceiling via dynamic $D_{\text{evict}}$ (§7.4).** The formula above
assumes user-applied attenuation as the sole contraction mechanism. Dynamic
$D_{\text{evict}}$ provides a strictly harder guarantee: when total node
count exceeds a budget, the system tightens $D_{\text{evict}}$ and evicts
unprotected contour tips immediately — independent of decay rate, spray
intensity, or attack strategy. The budget is a hard ceiling; the formula
above becomes a bound on steady-state node count _within_ that ceiling.
Annihilation ($\text{att} = 0$) provides an intermediate option between gradual attenuation and hard budget enforcement: the user can surgically zero a targeted region while leaving the rest of the tree intact, without adjusting $D_{\text{evict}}$ globally.

---

## Chapter 20. Structural Summary

```
   ┌──────────────────────────────────────────────────────────┐
   │              G-Tree (spatial, additively evolving)       │
   │                                                          │
   │              [0, 8)  sum=33, own=10                      │
   │             /            \                               │
   │         [0,4) s=15     [4,8) s=8                         │
   │          own=15          own=8                           │
   │          / \             / \                             │
   │      [0,2) [2,4)    [4,6) [6,8)                          │
   │       s=0   s=0      s=0   s=0                           │
   │       own=0 own=0    own=0 own=0                         │
   │                                                          │
   │  G-nodes have 0, 1, or 2 children.                       │
   │  g.own = direct accumulation. g.sum = own + children.    │
   │  Internal nodes are FROZEN — children intercept traffic. │
   │  Semi-internal nodes are PARTIALLY exposed.              │
   │                                                          │
   │  The bottom contour is the observation-receiving surface.│
   │  Fully exposed cells (no dependents) can be refined.     │
   │  Partially exposed cells (one dependent) can be restored │
   │  through competitive promotion — no separate gate needed.│
   │  Refinement adds resolution. Eviction removes it.        │
   │  Restoration regrows it — earned through competition.    │
   │  Only unprotected nodes (0 children) can be evicted.     │
   │                                                          │
   │  G-nodes serve as V-entries at multiple levels:          │
   │  contour and above-contour nodes all participate.        │
   └──────────────────────────────────────────────────────────┘
                 │
   ┌─────────────┼────────────────────────────────────────────┐
   │             ▼        V-Tree (tournament bracket)         │
   │                                                          │
   │           SR(33, 3-node, has_evictable=T)                │
   │           ├── L(15)  [0,4)  d=1 exposed=F — FROZEN       │
   │           ├── m₁(8, 3-node, has_evictable=T)             │
   │           │   ├── R(8) [4,8)  d=2 exposed=F — FROZEN     │
   │           │   ├── s₁(0, 2-node, has_evictable=T)         │
   │           │   │   ├── LL(0) [0,2) d=3 exposed=T          │
   │           │   │   └── LR(0) [2,4) d=3 exposed=T          │
   │           │   └── s₂(0, 2-node, has_evictable=T)         │
   │           │       ├── RL(0) [4,6) d=3 exposed=T          │
   │           │       └── RR(0) [6,8) d=3 exposed=T          │
   │           └── root(10) [0,8) d=1 exposed=F — FROZEN      │
   │                                                          │
   │  Entries (V-leaves) = G-nodes. Structural = scaffolding. │
   │  Heavy entries near root. Cold entries deep.             │
   │  Exposed flag: exposed cells participate in observation. │
   │  Evictable flag: prunes eviction scan efficiently.       │
   │  Legacy promotion: semi-internal entries that earn       │
   │  promotion create their missing child as a side-effect.  │
   │  No geometry — purely competitive ranking.               │
   │  Total V-intensity = G-root sum. No double-counting.     │
   │  Every entry is guaranteed live. No ghost checks needed. │
   └──────────────────────────────────────────────────────────┘
```

**The G-Tree** is a spatial index that grows where the V-Tree authorizes
resolution and shrinks when unprotected contour cells are evicted. Its
bottom contour is a step function of plateaus (§5.6) — the semantic output
unit for spatial queries. The live ordered map projection provides
$O(\log P)$ point queries, range iteration, and an $O(1)$
structural-complexity heartbeat. Its routing topology controls which
V-entries receive observations — the upward shield. Its structure controls
which entries may be evicted — the structural shield.

**The V-Tree** is a tournament where G-node entries compete for attention.
Heavy entries occupy shallow positions near the root. Light entries are
consolidated deep. It provides entropy-sensitive proportional sampling with
unconditionally safe traversal — every entry reached is live. Its uncle
constraint stabilises children's positions — the downward shield. When an
entry backs a semi-internal G-node and earns promotion, legacy
promotion creates the missing child as a side-effect.

**The two trees protect each other.** The G-Tree's spatial routing freezes
internal V-entries by intercepting their traffic. The G-Tree's structural
requirement ensures only unprotected nodes are evicted, so the contour
contracts cleanly from the tips. The V-Tree's uncle constraint shields
children from displacement while their parent's frozen benchmark stands.
Each tree's protection enables the other tree's dynamics.

**Geometry is the G-Tree's concern.** The V-Tree knows nothing about
coordinates.

**Competitive ranking is the V-Tree's concern.** The G-Tree makes no
structural decisions.

**Temporal semantics are the user's concern.** The architecture stores exact
accumulation and adapts to whatever the user's filter makes the numbers say.
The built-in temporal filter (§14) supports attenuation ("forget old data"), amplification ("sharpen fine detail"), and annihilation ("hard reset") through a single three-parameter function. The architecture imposes no policy on when or how these are combined.

**The design in one sentence.** The G-Tree maintains the _integral_ of
importance (spatial containment — parents ≥ children). The V-Tree maintains
the _derivative_ of importance (temporal resolution — children eventually >
parents). Every G-node lives in both trees simultaneously, and its two
rankings are the inverse of each other for any node that has delegated its
traffic to finer scales.

# Contour Ranges (Extension to the Dual-Tree Value-Stratified Index)

> _Convention._ Unqualified `§` references are to the main specification. References internal to this document use the prefix `§CR`.

---

## §CR.1 Definition

Plateaus $P_0, P_1, \ldots, P_{P-1}$ are indexed and totally ordered by the endpoint lattice $\mathcal{E} = \{a_0, a_1, \ldots, a_{P-1}\} \cup \{2^N\}$, where $a_j$ is the contour step coordinate at which plateau $P_j$ begins (P-I1). Each plateau $P_j$ occupies the contour tile $[a_j, a_{j+1})$.

A **contour range** is a contiguous sequence of plateaus:

$$\mathcal{R} = [a_s,\; a_{e+1}) \qquad 0 \leq s \leq e < P$$

selecting every plateau whose contour step coordinate falls in the half-open interval: $P_s, P_{s+1}, \ldots, P_e$. The spatial extent is:

$$\operatorname{span}(\mathcal{R}) = [a_s,\; a_{e+1})$$

No plateau is partially included. Every constituent plateau's full contour tile lies within the span.

**Terminal cases.**

- When $s = e$: the contour range contains a single plateau $P_s$. Its span is $[a_s, a_{s+1})$ — the plateau's contour tile.
- When $s = 0$ and $e = P - 1$: the contour range spans the full domain $[0, 2^N)$.
- The upper sentinel $a_{e+1} = 2^N$ when $e = P - 1$ (the last plateau).

The contour range inherits the plateau ordered map's indexing. Lookup, validation, and iteration use the same $O(\log P)$ mechanism that addresses individual plateaus.

---

## §CR.2 Basis Set

### §CR.2.1 Definition

The **basis set** of a contour range $\mathcal{R} = [a_s, a_{e+1})$ is the set of G-nodes returned by the segment-tree decomposition of §CR.8.1 applied to $(G_\text{root}, a_s, a_{e+1})$.

The segment-tree decomposition recurses through the G-Tree, selecting nodes by three mechanisms:

- **Fully-contained selection.** A G-node whose entire interval $[R.l, R.r)$ falls within $[a_s, a_{e+1})$ is selected directly. This applies to terminals, internals, and semi-internals alike — the node's internal structure is irrelevant.
- **Semi-internal early selection.** A semi-internal G-node that is not fully contained, but whose present-child half and absent-child half both overlap the range, is selected directly. This prevents the double-counting that would result from recursing into the present child (selecting descendants) and then also selecting the semi-internal for the absent child (whose `.sum` already includes those descendants). See §CR.8.1 for the algorithm and §CR.8.4 for the correctness argument.
- **Boundary selection.** A semi-internal G-node where only the absent child's territory overlaps the range (the present child's territory does not) is selected at the boundary.

The decomposition terminates at each selected node — no descendant of a selected node is separately selected. This is the minimality condition: the basis consists of the highest G-nodes whose selection-relevant territory falls within the span.

### §CR.2.2 Coverage

Each basis element $R$ has a **coverage** — the portion of $[a_s, a_{e+1})$ that $R$ is responsible for tiling:

| Selection path in the algorithm                                          | Coverage of $R$                                            |
| ------------------------------------------------------------------------ | ---------------------------------------------------------- |
| Fully-contained check (`start ≤ R.l` and `R.r ≤ end`)                    | $[R.l,\; R.r)$                                             |
| Semi-internal early selection (both halves overlap, not fully contained) | $[R.l,\; R.r) \;\cap\; [a_s,\; a_{e+1})$                   |
| Boundary selection (only absent half overlaps)                           | Uncovered half of $[R.l,\; R.r) \;\cap\; [a_s,\; a_{e+1})$ |

A fully-contained element covers its entire G-node interval. A semi-internal selected by early selection covers the intersection of its full interval with the range; the excess (if any) is boundary thatching (§CR.3). A boundary-selected semi-internal covers only its uncovered half that falls within the range; its surviving child's territory constitutes boundary thatching.

> _Note (context-dependent coverage)._ The same G-node can appear as a basis element for different contour ranges with different coverages. A semi-internal node fully contained in a wide range has coverage equal to its full interval. The same node appearing via early selection in a narrower range has coverage equal to its interval intersected with that range. The coverage is a property of the (node, range) pair, not of the node alone. §CR.14.7 demonstrates this with a concrete example.

### §CR.2.3 Derived Properties

> **Theorem (Complete Cover, CR-I2).** $\bigcup_{R \in \text{basis}} \text{coverage}(R) = [a_s, a_{e+1})$.
>
> _Proof._ Every point in the range is reached by exactly one recursive path that terminates at a basis element. The three selection mechanisms are exhaustive for any node whose interval overlaps the range: (1) the fully-contained check handles nodes entirely within the range; (2) the semi-internal early selection handles semi-internals straddling the range boundary where both halves overlap; (3) the original boundary and child-recursion logic handles all remaining cases (internal nodes recurse into both children, and semi-internals where only the absent half overlaps are selected directly). In all cases, the selected node's coverage includes the portion of the range that the recursive path was responsible for. No point in the range escapes all three checks. $\square$

> **Theorem (Disjointness, CR-I3).** The coverages of distinct basis elements are pairwise disjoint.
>
> _Proof._ Each recursive call partitions the parent's interval at the midpoint. Left and right subproblems receive disjoint sub-ranges. All three selection mechanisms terminate recursion upon selection (the fully-contained check returns, the semi-internal early selection returns, and the boundary selection returns), so no descendant of a selected node is separately selected. This ensures no ancestor–descendant pair coexists in the basis, which would be the only way for coverages to overlap given the partitioning structure. Distinct recursive paths therefore produce disjoint coverages. $\square$

> **Theorem (Minimality, CR-I4).** No proper ancestor of any basis element is also selected by the decomposition for the same range.
>
> _Proof._ The decomposition is a single top-down recursion. A node is selected only when recursion terminates at it. If an ancestor were also selected, recursion would have terminated at the ancestor and never reached the descendant — a contradiction. $\square$

**Single-plateau reduction.** When $s = e$, $\operatorname{span}(\mathcal{R}) = [a_s, a_{s+1})$. The segment-tree decomposition applied to this interval produces exactly $\operatorname{basis}(P_s)$ as defined in §5.6.1. The semi-internal early selection does not fire for single-plateau ranges because the midpoint of a semi-internal node is always a step coordinate (the depth changes between the present and absent sides), so a single-plateau range — which spans between consecutive step coordinates — cannot overlap both halves of a semi-internal simultaneously.

---

## §CR.3 Thatching

A contour range exhibits thatching at its boundaries in exactly the same way a plateau exhibits thatching at its boundaries. This is the same mechanism at both scales.

### §CR.3.1 What Thatching Is

A basis element's `.sum` accounts for energy across its full G-node interval $[R.l, R.r)$. When this interval extends beyond the basis element's coverage — beyond the contour range's span — the excess energy is **thatching**. The basis element's structural `.sum` is accepted in full; the excess is the price of using whole-node accounting without pro-ration.

### §CR.3.2 Boundary Thatching

A contour range can have thatching at its start ($a_s$) and at its end ($a_{e+1}$). A boundary or early-selected semi-internal basis element has:

- Its coverage inside $[a_s, a_{e+1})$ — this is why it qualifies as a basis element
- Territory extending outside $[a_s, a_{e+1})$ — this is the thatch
- Its `.sum` covering both — the in-range coverage and the out-of-range territory

Thatching can arise from two sources:

1. **Absent-child thatching** (original case). A semi-internal selected via boundary selection drags in its surviving child's `.sum`, which covers territory outside the range.
2. **Present-child thatching** (from early selection). A semi-internal selected via early selection may have part of its present child's territory outside the range. Its `.sum` includes that territory via G-I1.

At most one thatching semi-internal at each boundary. At most two total. This bound holds because the decomposition follows at most one path per boundary, and each path selects at most one semi-internal via early or boundary selection before terminating.

### §CR.3.3 Interior Thatching

Consider two adjacent plateaus $P_j$ and $P_{j+1}$, both inside $\mathcal{R}$, connected by a semi-internal G-node $R$. At the individual-plateau level, $R$ appears as a boundary basis element of one plateau, and the surviving child (or its descendants) appears in the other plateau's basis. The thatching at their shared boundary means $R.\operatorname{sum}$ includes energy also counted by $P_{j+1}$'s basis elements.

**When the semi-internal is fully contained in the range** — both halves fall within $[a_s, a_{e+1})$ — the segment-tree decomposition selects $R$ via the fully-contained check. Recursion terminates at $R$ without separately selecting its surviving child. The child is subsumed. G-I1 guarantees $R.\text{sum}$ accounts for all energy in $[R.l, R.r)$ exactly. The thatching that existed between the two plateaus vanishes: the energy is counted only once through $R.\operatorname{sum}$.

**When the semi-internal straddles the range boundary** — its full interval extends outside $[a_s, a_{e+1})$ — it is not fully contained. If both halves overlap the range, the semi-internal early selection (§CR.8.1) fires, selecting the node directly and preventing recursion into the present child. The interior thatching is resolved (no double-counting), but boundary thatching is introduced from the portion of the node's interval outside the range. If only the absent half overlaps, the node is selected via boundary selection as before.

More generally, consolidation may select an ancestor even higher than $R$ — any G-node whose full interval falls within the range and subsumes multiple per-plateau basis elements.

### §CR.3.4 Summary

| Location                                | Thatching | Mechanism                                                                                                 |
| --------------------------------------- | --------- | --------------------------------------------------------------------------------------------------------- |
| At start boundary ($a_s$)               | Present   | Boundary or early-selected semi-internal; surviving-child or present-child territory extends before range |
| At end boundary ($a_{e+1}$)             | Present   | Boundary or early-selected semi-internal; surviving-child or present-child territory extends past range   |
| Interior (between constituent plateaus) | Resolved  | Fully-contained selection or early selection subsumes semi-internals into whole-node accounting           |

A contour range cares about thatching at its start and end, in exactly the same way a plateau cares about thatching at its start and end.

---

## §CR.4 Basis Consolidation

The basis of a contour range may be strictly smaller than the union of its constituent plateaus' bases. When adjacent plateaus' basis elements are children (or deeper descendants) of a common G-node whose full interval falls within $[a_s, a_{e+1})$, the fully-contained selection picks the ancestor instead. Multiple per-plateau basis elements collapse into one.

This consolidation is what resolves interior thatching (§CR.3.3). It is also what makes multi-plateau energy accounting structurally different from summing individual plateau energies — consolidation introduces **ancestor `.own` energy** that no individual plateau captured (§CR.9).

**Extreme case.** If $[a_s, a_{e+1})$ exactly matches some G-node's interval:

$$\operatorname{basis}(\mathcal{R}) = \{R\}, \qquad E(\mathcal{R}) = R.\operatorname{sum}$$

All internal plateau structure is subsumed by a single node.

**Bound.** $|\operatorname{basis}(\mathcal{R})| \leq 2N$ (standard segment-tree decomposition of a dyadic range). In practice, consolidation across plateau boundaries typically yields:

$$|\operatorname{basis}(\mathcal{R})| \;\leq\; \sum_{j=s}^{e} |\operatorname{basis}(P_j)|$$

with strict inequality whenever consolidation occurs.

---

## §CR.5 First and Last Basis Elements

The first basis element of $\mathcal{R}$ (leftmost in spatial order) covers coordinate $a_s$. It is either the first plateau's first basis G-node, or an ancestor of it produced by consolidation. The last basis element (rightmost) covers the coordinate just before $a_{e+1}$. It is either the last plateau's last basis G-node, or an ancestor of it.

If the first basis element is a semi-internal whose territory extends before $a_s$ (selected via early selection or boundary selection), its `.sum` includes energy outside the range. This is start-boundary thatching.

If the last basis element is a semi-internal whose territory extends past $a_{e+1}$, its `.sum` includes energy outside the range. This is end-boundary thatching.

Consolidation only merges upward — it never splits. The spatial extent of the contour range is anchored at the extremal G-nodes of the first and last constituent plateaus, or at ancestors that contain them.

---

## §CR.6 Energy

The **energy** of a contour range is the sum of its basis elements' sums:

$$\boxed{E(\mathcal{R}) \;=\; \sum_{R \;\in\; \operatorname{basis}(\mathcal{R})} R.\operatorname{sum}}$$

This is the same definition as for a single plateau. One formula at both scales.

The energy includes boundary thatching at $a_s$ and $a_{e+1}$, because boundary semi-internals' `.sum` values include their out-of-range contributions. This is accepted by definition — the same acceptance that a single plateau makes at its boundaries.

The energy does not double-count interior thatching, because basis consolidation (§CR.4) and the semi-internal early selection (§CR.8.1) have already resolved it. However, consolidation introduces a second effect: **ancestor `.own` energy** — the pre-split accumulation of G-nodes that span multiple plateaus and are selected as basis elements only when the range is wide enough to fully contain them. This energy was invisible to any individual plateau's basis. The interplay between resolved thatching (energy removed) and ancestor `.own` absorption (energy added) governs the relationship between a contour range's energy and the sum of its constituent plateaus' energies (§CR.9).

---

## §CR.7 Invariants

$$\textbf{CR-I1 (Valid Endpoints):}\quad a_s, a_{e+1} \in \mathcal{E}, \;\; s \leq e, \;\; 0 \leq s < P, \;\; 0 \leq e < P$$

$$\textbf{CR-I2 (Complete Cover):}\quad \bigcup_{\operatorname{basis}(\mathcal{R})} \text{coverage}(R) \;=\; [a_s,\, a_{e+1})$$

$$\textbf{CR-I3 (Disjointness):}\quad \forall\, R \neq R' \in \operatorname{basis}(\mathcal{R}):\; \text{coverage}(R) \,\cap\, \text{coverage}(R') = \emptyset$$

$$\textbf{CR-I4 (Minimality):}\quad \text{No proper ancestor of any basis element is also in the basis for this range}$$

$$\textbf{CR-I5 (Energy):}\quad E(\mathcal{R}) = \sum_{\operatorname{basis}(\mathcal{R})} R.\operatorname{sum}$$

$$\textbf{CR-I6 (Determinism):}\quad \operatorname{basis}(\mathcal{R}) \text{ is uniquely determined by the G-Tree state and } [a_s,\, a_{e+1})$$

$$\textbf{CR-I7 (Consistency):}\quad s = e \implies \operatorname{basis}(\mathcal{R}) = \operatorname{basis}(P_s) \;\text{ and }\; E(\mathcal{R}) = E(P_s)$$

$$\textbf{CR-I8 (Boundary Thatch):}\quad \text{Basis elements whose } \texttt{.sum} \text{ covers territory outside the span occur only at the}$$
$$\text{start and end boundaries, at most one each}$$

> _Note (CR-I2 and CR-I3 use coverage)._ The tiling and disjointness invariants are stated in terms of coverage (§CR.2.2), not in terms of the G-node's full interval. For fully-contained basis elements, coverage equals the full interval. For boundary and early-selected semi-internals, coverage is the intersection of the relevant territory with the range. The proofs appear in §CR.2.3.

---

## §CR.8 Computation

### §CR.8.1 Algorithm

```
function contour_range_basis(g_root, a_s, a_{e+1}) → basis:
    basis ← {}
    decompose(g_root, a_s, a_{e+1}, basis)
    return basis

function decompose(g, start, end, basis):
    if g = null or g.r ≤ start or g.l ≥ end:
        return

    // Fully contained — select as basis element
    if start ≤ g.l and g.r ≤ end:
        basis ← basis ∪ {g}
        return

    m ← midpoint(g.l, g.r)

    // Semi-internal early selection guard.
    //
    // When a semi-internal node is not fully contained but both its
    // present-child half and absent-child half overlap the range,
    // select it directly and return. Without this guard, the algorithm
    // would recurse into the present child (selecting descendants) and
    // then also select the node for the absent child — producing an
    // ancestor–descendant pair in the basis. Since the node's .sum
    // already includes those descendants' .sum values (by G-I1), the
    // result would be double-counted energy and a violation of CR-I4.
    //
    // The guard trades fine-grained present-child decomposition for
    // correctness: the node's .sum covers its full interval, introducing
    // boundary thatching from territory outside the range. This is the
    // same trade-off that single plateaus make at their boundaries.
    left_absent  ← (g.left = null)
    right_absent ← (g.right = null)
    if left_absent ≠ right_absent:       // exactly one child absent: semi-internal
        left_overlaps  ← max(g.l, start) < min(m, end)
        right_overlaps ← max(m, start) < min(g.r, end)
        if left_overlaps and right_overlaps:
            basis ← basis ∪ {g}
            return

    // Standard recursion into children
    if g.left ≠ null:
        decompose(g.left, start, end, basis)
    else if max(g.l, start) < min(m, end):
        // g is semi-internal; only its absent left half overlaps the range.
        // The present (right) child's territory does not overlap.
        basis ← basis ∪ {g}
        return

    if g.right ≠ null:
        decompose(g.right, start, end, basis)
    else if max(m, start) < min(g.r, end):
        // g is semi-internal; only its absent right half overlaps the range.
        // The present (left) child's territory does not overlap.
        basis ← basis ∪ {g}
        return
```

### §CR.8.2 Cost

$O(N)$ — standard segment-tree decomposition. At most $2N$ basis elements.

### §CR.8.3 Energy Query

```
function contour_range_energy(g_root, a_s, a_{e+1}) → energy:
    basis ← contour_range_basis(g_root, a_s, a_{e+1})
    return sum(R.sum for R in basis)
```

$O(N)$ total.

### §CR.8.4 Correctness of the Semi-Internal Early Selection

**Claim.** The semi-internal early selection guard fires only when necessary and does not alter the result for any case that the original algorithm handled correctly.

**When the guard fires.** The guard requires three conditions simultaneously: (1) the node is semi-internal (exactly one child absent); (2) the node is not fully contained (the fully-contained check already returned); (3) both halves overlap the range. Under the original algorithm without the guard, condition (3) would cause the present-child recursion to select descendants, and the absent-child null check to select the node — producing an ancestor–descendant pair that violates CR-I4 and double-counts energy.

**When the guard does not fire.**

- _Internal nodes (both children present):_ `left_absent ≠ right_absent` is false. The guard is skipped.
- _Terminal nodes (both children absent):_ Same — the guard is skipped. The original null-check logic handles terminals correctly (at most one selection due to the `return` after each null-check addition).
- _Semi-internals where only one half overlaps:_ The conjunction `left_overlaps and right_overlaps` is false. The guard is skipped. The original logic handles these correctly: if only the absent half overlaps, the node is selected via boundary selection; if only the present half overlaps, descendants are selected and the absent-half check finds no overlap.

**Single-plateau ranges (CR-I7).** The guard cannot fire for single-plateau ranges. The midpoint of a semi-internal node is always a step coordinate in $\mathcal{E}$ (the contour depth changes between the present and absent halves). A single-plateau range $[a_j, a_{j+1})$ spans between consecutive step coordinates, so it cannot overlap both halves of a semi-internal whose midpoint lies at a step coordinate between $a_j$ and $a_{j+1}$ — by definition, that midpoint would be $a_j$ or $a_{j+1}$ itself, placing one half entirely outside the range. Therefore, the guard's firing condition is never satisfied, and single-plateau behavior is identical to the original algorithm.

> **Lemma (midpoint of a semi-internal is a step coordinate).** Let $g$ be a semi-internal G-node with interval $[l, r)$ and midpoint $m = (l + r) / 2$. Its present child covers one half and its absent child covers the other. On the present-child side, all contour cells are at depth $\geq d_g + 1$ (they are descendants of $g$'s surviving child). On the absent-child side, all contour cells are at depth $d_g$ (they are uncovered cells of $g$ itself, whose absent child was evicted). Therefore the contour depth changes at $m$: the present-child half has depth $\geq d_g + 1$ and the absent-child half has depth $d_g$. By definition (§12.1), a step coordinate is a point where contour depth changes, so $m \in \mathcal{E}$. $\square$

---

## §CR.9 Algebra

### §CR.9.1 Concatenation

For contour ranges $\mathcal{R}_1 = [a_s, a_m)$ and $\mathcal{R}_2 = [a_m, a_{e+1})$ sharing boundary $a_m \in \mathcal{E}$:

$$\mathcal{R}_1 \cdot \mathcal{R}_2 = [a_s,\; a_{e+1})$$

The concatenated range contains all plateaus from $P_s$ through $P_e$.

**Basis consolidation under concatenation.** G-nodes that were boundary thatching elements at $a_m$ in $\mathcal{R}_1$ or $\mathcal{R}_2$ individually may become interior to the concatenation. Consolidation absorbs them upward. Additionally, G-nodes whose full intervals straddle $a_m$ — not captured by either sub-range's basis individually — may now be fully contained and selected by the concatenation's decomposition.

$$|\operatorname{basis}(\mathcal{R}_1 \cdot \mathcal{R}_2)| \;\leq\; |\operatorname{basis}(\mathcal{R}_1)| + |\operatorname{basis}(\mathcal{R}_2)|$$

**Energy under concatenation.** Two effects compete at the join point $a_m$:

- **Consolidation gain** ($G \geq 0$, under non-negative $g.\text{own}$): G-nodes whose intervals straddle $a_m$ may be selected as basis elements of the concatenation, gaining their `.own` energy that neither sub-range captured. By G-I1, a node's `.sum` equals its `.own` plus children's `.sum` values. When the concatenation selects such a node and the sub-ranges had selected its children (or descendants), the difference is the intermediate `.own` values.

- **Thatching resolution** ($T \geq 0$, under non-negative $g.\text{own}$): boundary semi-internals at $a_m$ that thatched into the adjacent sub-range have their double-counted energy resolved. The surviving child's `.sum`, counted by the thatching semi-internal's `.sum` in one sub-range and separately by the child's own basis entry in the other sub-range, is now counted only once.

$$\boxed{E(\mathcal{R}_1 \cdot \mathcal{R}_2) \;=\; E(\mathcal{R}_1) \;+\; E(\mathcal{R}_2) \;+\; G(m) \;-\; T(m)}$$

The sign of $G - T$ is **indeterminate**: it depends on the relative magnitudes of ancestor `.own` accumulation and boundary thatching at $a_m$. Neither $E(\text{concat}) \leq E(\mathcal{R}_1) + E(\mathcal{R}_2)$ nor $E(\text{concat}) \geq E(\mathcal{R}_1) + E(\mathcal{R}_2)$ holds universally.

> _Intuition._ Consolidation gain is large when internal G-nodes above the join point have significant pre-split accumulation (large `.own`). Thatching resolution is large when semi-internal nodes at the join point have heavy surviving children. The two effects are structurally independent — a tree can have one without the other, both, or neither at any given boundary.

**Formal definitions of $G$ and $T$.** Let $B_1 = \operatorname{basis}(\mathcal{R}_1)$, $B_2 = \operatorname{basis}(\mathcal{R}_2)$, and $B_C = \operatorname{basis}(\mathcal{R}_1 \cdot \mathcal{R}_2)$.

- **Consolidation gain.** $G(m)$ is the sum of `.own` values at G-nodes whose intervals straddle $a_m$ and are selected by the concatenated decomposition $B_C$ but are not selected by either sub-decomposition $B_1$ or $B_2$:

$$G(m) = \sum_{\substack{g \in B_C \setminus (B_1 \cup B_2) \\ g.\text{interval straddles } a_m}} g.\text{own}$$

  These are ancestors that become fully contained only when the two sub-ranges are joined. Each such $g$ contributes its `.own` because $g.\text{sum}$ replaces the sum of its descendants' `.sum` values (which the sub-decompositions had selected individually), and the difference is $g.\text{own}$ plus any intermediate ancestors' `.own`, accumulated level by level via G-I1.

- **Thatching resolution.** $T(m)$ is the energy that was double-counted at $a_m$ by the two sub-decompositions due to boundary thatching from semi-internal nodes:

$$T(m) = \sum_{\substack{g \in B_1 \cup B_2 \\ g \text{ semi-internal,} \\ g.\text{interval straddles } a_m}} g.\text{surviving\_child.sum} \times \mathbb{1}[\text{child also contributes to the other sub-basis}]$$

  Specifically, if a semi-internal $g$ is selected in $B_1$ (or $B_2$) and its surviving child's territory extends past $a_m$ into $\mathcal{R}_2$ (or $\mathcal{R}_1$), then $g.\text{sum}$ includes the child's `.sum`, and the other sub-decomposition independently selects descendants covering that same territory. The thatching resolution $T$ is the sum of such double-counted child `.sum` values. After concatenation, the child is subsumed by $g$ (or by an ancestor of $g$), eliminating the double count.

### §CR.9.2 Splitting

At any interior plateau boundary $a_m \in \mathcal{E}$ with $s < m \leq e$:

$$\mathcal{R} = [a_s, a_m) \cdot [a_m, a_{e+1})$$

Splitting is the inverse of concatenation. Reading the concatenation identity in the opposite direction:

$$E([a_s, a_m)) + E([a_m, a_{e+1})) \;=\; E(\mathcal{R}) \;-\; G(m) \;+\; T(m)$$

Splitting loses consolidation gains (ancestor `.own` energy drops out when the ancestor's interval straddles the split point and no sub-range fully contains it) and creates boundary thatching at $a_m$ (energy that was counted once by consolidation is now counted in both sub-ranges). The sign of $-G + T$ is indeterminate by the same argument.

### §CR.9.3 Nesting

For $[a_i, a_k) \subseteq [a_s, a_{e+1})$ with all endpoints in $\mathcal{E}$:

> **Theorem (Nesting Monotonicity).** If $g.\text{own} \geq 0$ for every G-node $g$, then:
>
> $$E([a_i, a_k)) \;\leq\; E([a_s, a_{e+1}))$$

**Proof.** Let $B_I = \text{basis}([a_i, a_k))$ and $B_O = \text{basis}([a_s, a_{e+1}))$, both computed by the segment-tree decomposition of §CR.8.1.

Partition $B_I$ into two sets:

1. **Shared:** $b \in B_I \cap B_O$. The same node is selected by both decompositions, contributing $b.\text{sum}$ to both energies.

2. **Subsumed:** $b \in B_I \setminus B_O$. Since $b$'s interval overlaps $[a_s, a_{e+1}) \supseteq [a_i, a_k)$, the outer decomposition either selects $b$ (case 1) or selects a proper ancestor $a$ of $b$. In the latter case, define $D(a) = \{b \in B_I : b \text{ is a descendant of } a\}$. By G-I1 and the non-negative `.own` hypothesis, iterating from $a$ down to the $b$'s:

$$a.\text{sum} = a.\text{own} + \sum_{c} c.\text{sum} \geq \sum_{c} c.\text{sum} \geq \cdots \geq \sum_{b \in D(a)} b.\text{sum}$$

Each intermediate node adds a non-negative `.own` at each level. Therefore $a$'s contribution to $E(B_O)$ is at least the sum of its subsumed descendants' contributions to $E(B_I)$.

The critical step: CR-I4 guarantees that no ancestor–descendant pair coexists in either basis. Without CR-I4, the inner basis could contain both a node and its ancestor, making $\sum_{b \in B_I} b.\text{sum}$ exceed $a.\text{sum}$ and breaking the inequality. The semi-internal early selection guard (§CR.8.1) is what makes CR-I4 hold, and therefore what makes this proof valid.

For each $a \in B_O$:

- If $a \in B_I$ (shared): contributes $a.\text{sum}$ to both sides.
- If $a \notin B_I$ but $D(a) \neq \emptyset$: contributes $a.\text{sum} \geq \sum_{b \in D(a)} b.\text{sum}$.
- If $D(a) = \emptyset$ (covers territory only outside the inner range): contributes $a.\text{sum} \geq 0$ to the outer side, 0 to the inner side.

Summing: $E(B_O) \geq E(B_I)$. $\square$

**Precondition.** The nesting inequality requires $g.\text{own} \geq 0$ at every G-node, not merely P1 (non-negative importance). Under the **standard configuration** ($T = \mathbb{R}_{\geq 0}$), the condition holds by construction — all observations are non-negative and absorption preserves this. Under the **absolute configuration** ($T = \mathbb{R}$, $I = \mathbb{R}_{\geq 0}$), the ledger type is signed and `g.own` may be negative — the nesting inequality does **not** hold. Under the **signed configuration**, it likewise fails.

> _Counterexample (signed ledger)._ Under $T = \mathbb{R}$:
>
> ```
> [0,8) own=-20, sum=10
> ├── [0,4) terminal, own=15, sum=15
> └── [4,8) terminal, own=15, sum=15
> ```
>
> $E([0,4)) = 15 > 10 = E([0,8))$. Nesting violated.

### §CR.9.4 Energy Algebra Summary

| Operation     | Energy relation                                                  | Direction                                          |
| ------------- | ---------------------------------------------------------------- | -------------------------------------------------- |
| Concatenation | $E(\text{concat}) = E(\mathcal{R}_1) + E(\mathcal{R}_2) + G - T$ | Indeterminate ($G, T \geq 0$ under non-neg `.own`) |
| Splitting     | $E(\mathcal{R}_1) + E(\mathcal{R}_2) = E(\mathcal{R}) - G + T$   | Indeterminate (inverse of concatenation)           |
| Nesting       | $E(\text{inner}) \leq E(\text{outer})$                           | Non-negative (requires non-neg `.own`)             |

The nesting inequality is the sole directional result: widening a contour range can only increase energy. Concatenation and splitting have no universal direction because consolidation gain $G$ (ancestor `.own` energy gained) and thatching resolution $T$ (double-counted energy removed) have independent magnitudes.

---

## §CR.10 Properties

### §CR.10.1 Full-Domain Range

$$\operatorname{basis}([0, 2^N)) = \{G_{\text{root}}\}, \qquad E = G_{\text{root}}.\operatorname{sum}$$

All plateaus subsumed. All interior thatching resolved. No boundary thatching — the root has no exterior. The root's `.own` energy — invisible to any individual plateau whose basis does not include the root — is captured by the full-domain range.

### §CR.10.2 Single-Plateau Range (CR-I7)

$$\operatorname{basis}([a_j, a_{j+1})) = \operatorname{basis}(P_j), \qquad E = E(P_j)$$

By construction: the same algorithm applied to the same interval (§CR.8.4 proves the semi-internal early selection does not fire). The contour range's boundary thatching at $a_j$ and $a_{j+1}$ is exactly the plateau's boundary thatching.

### §CR.10.3 Mutation Sensitivity

$\mathcal{E}$ changes by $O(1)$ elements per G-Tree mutation (bounded plateau change, Lemma 12.2). A contour range $[a_s, a_{e+1})$ is **invalidated** if $a_s$ or $a_{e+1}$ is removed from $\mathcal{E}$, or if the plateau indices $s$ and $e$ shift due to insertion or deletion of plateaus before them.

Membership in $\mathcal{E}$ is testable in $O(\log P)$ via the plateau ordered map. After a mutation, re-anchoring a contour range to the current lattice requires at most two $O(\log P)$ lookups.

**Eviction and re-anchoring.** When a G-node is evicted, the plateau map changes (Lemma 12.2: $\Delta P \in \{-2, -1, 0, +1, +2\}$), which means $\mathcal{E}$ may lose or gain elements. A cached contour range $[a_s, a_{e+1})$ whose endpoints are no longer in $\mathcal{E}$ must be re-anchored. Operationally, each stale endpoint is snapped to the nearest surviving step coordinate via an $O(\log P)$ predecessor/successor query on the plateau map: $a_s$ snaps outward to $\operatorname{pred}_{\mathcal{E}}(a_s)$ (widening) or inward to $\operatorname{succ}_{\mathcal{E}}(a_s)$ (narrowing), and symmetrically for $a_{e+1}$. The choice depends on the caller's semantics — widening preserves coverage of all originally-selected plateaus, narrowing preserves lattice alignment at the cost of dropping boundary plateaus that merged.

**Temporal scaling (decay).** When `decay()` is applied to a subtree (§14), plateau `.sum` values change, which affects contour range energies. However, `decay()` does not alter the G-Tree's topology — no nodes are created or destroyed — so $\mathcal{E}$ is unchanged and cached contour range endpoints remain valid. The basis set is also unchanged (it depends only on tree structure). Only the energy $E(\mathcal{R}) = \sum_{R \in \text{basis}} R.\text{sum}$ must be recomputed, since the `.sum` values have been scaled. Cached energy values should not survive a decay operation.

### §CR.10.4 Cross-Plateau Energy

The difference between the contour range's energy and the sum of its constituent plateaus' individual energies:

$$E_{\times}(\mathcal{R}) = E(\mathcal{R}) - \sum_{j=s}^{e} E(P_j)$$

$E_\times$ aggregates the net of consolidation gains and resolved thatching across all interior plateau boundaries within $\mathcal{R}$. Its sign is **indeterminate** under non-negative $g.\text{own}$:

- $E_\times > 0$ when ancestor `.own` energy gained by consolidation exceeds resolved thatching.
- $E_\times < 0$ when resolved thatching exceeds gained ancestor energy.
- $E_\times = 0$ when the two effects cancel, or when there are no interior semi-internals and no spanning ancestors with non-zero `.own`.

The main specification (§16.3.6, plateau sum cross-check) demonstrates $E_\times > 0$: the G-root's `.own` $= 10$ is captured by the full-domain contour range but by no individual plateau.

---

## §CR.11 Summary Table

| Concept                 | Definition                                                                                                                                                      |
| ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **Contour range**       | $[a_s, a_{e+1})$ — contiguous plateaus $P_s$ through $P_e$, indexed by $\mathcal{E}$                                                                            |
| **Basis set**           | G-nodes returned by segment-tree decomposition with semi-internal early selection — same algorithm framework as single-plateau basis                            |
| **Coverage**            | Per-element portion of the span: full interval (fully contained), range intersection (early-selected semi-internal), or uncovered half (boundary semi-internal) |
| **Energy**              | $\sum_{\text{basis}} R.\operatorname{sum}$ — same formula as single-plateau energy                                                                              |
| **Start thatching**     | Boundary or early-selected semi-internal at $a_s$ whose territory extends before the range                                                                      |
| **End thatching**       | Boundary or early-selected semi-internal at $a_{e+1}$ whose territory extends past the range                                                                    |
| **Interior thatching**  | Resolved by fully-contained selection and early selection — does not appear in the contour range                                                                |
| **Consolidation gain**  | Ancestor `.own` energy gained when consolidation selects nodes spanning interior boundaries                                                                     |
| **First basis element** | First plateau's first G-node, or ancestor via consolidation                                                                                                     |
| **Last basis element**  | Last plateau's last G-node, or ancestor via consolidation                                                                                                       |
| **Basis size**          | $\leq 2N$; typically $\leq \sum_{j=s}^{e} \lvert\operatorname{basis}(P_j)\rvert$ with consolidation                                                             |
| **Computation cost**    | $O(N)$ — standard segment-tree decomposition                                                                                                                    |
| **Single-plateau case** | Identical by construction, including boundary thatching (early selection does not fire)                                                                         |

---

## §CR.12 Plateau Selection

### §CR.12.1 Problem

Given an arbitrary dyadic range $[l, r)$ where $0 \leq l < r \leq 2^N$, identify the contiguous run of plateaus that overlap it. Return their indices so a contour range can be constructed.

### §CR.12.2 Definition

$$\operatorname{select\_plateaus}(l, r) \;=\; (s,\; e+1)$$

where:

- $s$ is the index of the plateau containing $l$: the unique $s$ such that $a_s \leq l < a_{s+1}$
- $e$ is the index of the plateau containing $r - 1$: the unique $e$ such that $a_e \leq r - 1 < a_{e+1}$

The returned pair $(s, e+1)$ is a half-open plateau index range. The selected plateaus are $P_s, P_{s+1}, \ldots, P_e$ — every plateau that has any overlap with $[l, r)$.

The corresponding contour range is:

$$\mathcal{R} = [a_s,\; a_{e+1})$$

### §CR.12.3 Properties

**Completeness.** Every point in $[l, r)$ belongs to some selected plateau. No plateau overlapping $[l, r)$ is excluded.

**Contiguity.** The selected plateaus form a contiguous run. This is immediate: plateaus tile the domain without gaps, so any range that overlaps $P_s$ and $P_e$ must also overlap every plateau between them.

**Containment.** The contour range's span contains the input range:

$$[l, r) \;\subseteq\; [a_s,\; a_{e+1})$$

with equality if and only if $l$ and $r$ are both in $\mathcal{E}$.

**Widening.** When $l$ and $r$ are not lattice points, the contour range is strictly wider than the input range. The excess at the start is $[a_s, l)$ and at the end is $[r, a_{e+1})$. These are the partial-plateau regions at the boundaries.

**Single-plateau case.** When $l$ and $r - 1$ fall in the same plateau ($s = e$), the result is $(s, s+1)$ — a single-plateau contour range.

### §CR.12.4 Algorithm

```
function select_plateaus(l, r) → (s, e_next):
    // Find plateau containing l
    s ← plateau_map.floor_index(l)      // largest a_j ≤ l

    // Find plateau containing r - 1
    e ← plateau_map.floor_index(r - 1)  // largest a_j ≤ r - 1

    return (s, e + 1)
```

> _Note (floating-point coordinates)._ The `floor_index(r - 1)` call uses the predecessor of $r$ to identify the last included plateau. For integer coordinates, $r - 1$ is the last included coordinate — well-defined. For floating-point coordinates, $r - 1$ is not meaningful in the dyadic framework (§3.2.6). Floating-point implementations should use an exclusive-upper-bound variant: `floor_index_strict_less(r)`, returning the largest $a_j$ strictly less than $r$. Since contour range endpoints lie in $\mathcal{E}$ (which consists of G-node boundaries, all dyadic rationals), this is equivalent when $r \in \mathcal{E}$.

### §CR.12.5 Cost

$O(\log P)$ — two lookups in the plateau ordered map.

### §CR.12.6 Composition with Contour Range Construction

The full pipeline from dyadic range to contour range:

```
function contour_range_from_dyadic(g_root, l, r) → (basis, s, e_next):
    (s, e_next) ← select_plateaus(l, r)
    a_s ← plateau_map.key_at(s)
    a_e_next ← plateau_map.key_at(e_next)   // or 2^N if e_next = P
    basis ← contour_range_basis(g_root, a_s, a_e_next)
    return (basis, s, e_next)
```

$O(\log P + N)$ total: $O(\log P)$ for selection, $O(N)$ for basis computation.

---

## §CR.13 Exact Energy

### §CR.13.1 Problem

Given an arbitrary dyadic range $[l, r)$ where $0 \leq l < r \leq 2^N$, compute the energy spatially attributable to that range — with sub-cell precision, pro-rating G-nodes that straddle the boundaries.

This is the complement of `select_plateaus` (§CR.12). Selection snaps outward to plateau boundaries and uses the basis `.sum` total. Exact energy stays at the requested boundaries and pro-rates.

### §CR.13.2 Definition

The **exact energy** of a dyadic range $[l, r)$ is the G-Tree's recursive range sum:

$$\operatorname{exact\_energy}(l, r) \;=\; \operatorname{range\_sum}(G_{\text{root}},\; l,\; r)$$

where $\operatorname{range\_sum}$ (§5.5.2) recurses through the G-Tree, accumulating:

- $R.\operatorname{sum}$ for G-nodes fully contained in $[l, r)$
- $f_R \cdot R.\operatorname{own}$ for G-nodes straddling a boundary, where $f_R$ is the overlap fraction:

$$f_R \;=\; \frac{\min(R.r,\, r) - \max(R.l,\, l)}{R.r - R.l}$$

Since both G-node boundaries and the query boundaries are dyadic, $f_R$ is always a ratio of powers of two.

### §CR.13.3 Algorithm

```
function exact_energy(g_root, l, r) → energy:
    return range_sum(g_root, l, r)

function range_sum(g, l, r) → energy:
    if g = null or g.r ≤ l or g.l ≥ r:
        return 0

    // Fully contained
    if l ≤ g.l and g.r ≤ r:
        return g.sum

    // Partially overlapping: pro-rate own, recurse into children
    overlap ← min(g.r, r) - max(g.l, l)
    width ← g.r - g.l
    own_contribution ← g.own × (overlap / width)

    child_contribution ← 0
    if g.left ≠ null:
        child_contribution += range_sum(g.left, l, r)
    if g.right ≠ null:
        child_contribution += range_sum(g.right, l, r)

    return own_contribution + child_contribution
```

### §CR.13.4 Cost

$O(N)$ — standard segment-tree range query. At most $2N$ fully-contained nodes, at most $2N$ partially-overlapping ancestors on the two boundary paths.

### §CR.13.5 The Uniform-Within-Cell Assumption

The pro-rating $f_R \cdot R.\operatorname{own}$ assumes that a G-node's own-accumulation is uniformly distributed across its interval. This is a deliberate simplification. A G-node's `.own` is the sum of:

- Pre-split observations (deposited before the node's children existed)
- Absorbed energy (from evicted children)
- Post-eviction observations (routed to the node after a child was removed)

These components have no reason to be spatially uniform within $[R.l, R.r)$. The pro-rating is the best estimate available without sub-cell structure, but it is an estimate — not a measurement.

The contour range energy (§CR.6) avoids this assumption entirely by never pro-rating. It uses whole `.sum` values from basis elements, accepting thatching rather than estimating sub-cell distributions.

### §CR.13.6 Relationship to Contour Range Energy

Contour range energy and exact energy are **not generally ordered.** They differ by two independent effects:

- **Boundary thatching** ($B \geq 0$, under non-negative $g.\text{own}$): energy from boundary basis elements' `.sum` that covers territory outside $[a_s, a_{e+1})$. Included in contour range energy, excluded from exact energy.
- **Ancestor pro-ration** ($A \geq 0$, under non-negative $g.\text{own}$): energy from partial-overlap ancestors' `.own` that is pro-rated into $[a_s, a_{e+1})$ by `range_sum`. These ancestors are not basis elements of the contour range (they are not fully contained and were not selected by any selection path), so their `.own` contribution is included in exact energy but excluded from contour range energy.

$$\boxed{E(\mathcal{R}) \;=\; \operatorname{exact\_energy}(a_s, a_{e+1}) \;+\; B \;-\; A}$$

Both $B \geq 0$ and $A \geq 0$ under non-negative $g.\text{own}$. The sign of $B - A$ is indeterminate.

**Formal definitions.** Let $\text{basis} = \operatorname{basis}([a_s, a_{e+1}))$.

$$B = \sum_{\substack{R \in \text{basis} \\ R.\text{interval} \not\subseteq [a_s, a_{e+1})}} \operatorname{range\_sum}\big(R,\; R.\text{interval} \setminus [a_s, a_{e+1})\big)$$

That is, $B$ sums the energy that each boundary basis element attributes to territory outside the contour range. For a boundary semi-internal whose interval extends past one end of the range, this is the portion of its `.sum` (computed by `range_sum`) that falls in the overhang.

$$A = \sum_{\substack{g \notin \text{basis} \\ g.\text{interval} \cap [a_s, a_{e+1}) \neq \emptyset \\ g.\text{interval} \not\subseteq [a_s, a_{e+1})}} g.\text{own} \times \frac{|g.\text{interval} \cap [a_s, a_{e+1})|}{|g.\text{interval}|}$$

That is, $A$ sums the pro-rated `.own` contributions of non-basis ancestors that partially overlap the range. Each such ancestor $g$ has its `.own` uniformly distributed across its interval (the same assumption underlying `range_sum`), and the fraction overlapping $[a_s, a_{e+1})$ is included in exact energy but absent from contour range energy.

**Special cases:**

| Condition                                              | Relationship                            | Reason               |
| ------------------------------------------------------ | --------------------------------------- | -------------------- |
| No boundary thatching and no partial-overlap ancestors | $E(\mathcal{R}) = \text{exact\_energy}$ | $B = A = 0$          |
| Full domain                                            | $E(\mathcal{R}) = \text{exact\_energy}$ | No boundaries at all |
| No boundary thatching, positive partial-overlap `.own` | $E(\mathcal{R}) < \text{exact\_energy}$ | $B = 0$, $A > 0$     |
| Boundary thatching, no partial-overlap ancestors       | $E(\mathcal{R}) > \text{exact\_energy}$ | $B > 0$, $A = 0$     |

### §CR.13.7 When to Use Which

| Situation                   | Use                                     | Reason                                                                                                   |
| --------------------------- | --------------------------------------- | -------------------------------------------------------------------------------------------------------- |
| Comparing plateau groups    | $E(\mathcal{R})$ (contour range energy) | Consistent with individual plateau energy; one formula at all scales                                     |
| Structural decomposition    | $\operatorname{basis}(\mathcal{R})$     | Minimal G-node cover; thatching resolved interior                                                        |
| Precise spatial attribution | $\operatorname{exact\_energy}(l, r)$    | Pro-rates at boundaries; accounts for all ancestor `.own`; sub-cell precision (under uniform assumption) |
| Arbitrary coordinate query  | $\operatorname{exact\_energy}(l, r)$    | Does not require lattice-aligned endpoints                                                               |
| Total domain energy         | Either (they agree)                     | Full-domain range has no boundaries                                                                      |
| Upper bound on true energy  | Neither is a universal bound            | Each includes energy the other excludes                                                                  |

### §CR.13.8 Composition with Plateau Selection

The full pipeline offering both energy measures:

```
function query_range(g_root, l, r) → (s, e_next, basis, E_contour, E_exact):
    // Plateau-level: snap to lattice, compute basis and contour energy
    (s, e_next) ← select_plateaus(l, r)
    a_s ← plateau_map.key_at(s)
    a_e_next ← plateau_map.key_at(e_next)    // or 2^N if e_next = P
    basis ← contour_range_basis(g_root, a_s, a_e_next)
    E_contour ← sum(R.sum for R in basis)

    // Spatial-level: exact energy for the original range
    E_exact ← exact_energy(g_root, l, r)

    return (s, e_next, basis, E_contour, E_exact)
```

$O(\log P + N)$ total. The two energy queries share the same $O(N)$ tree traversal structure and could be fused into a single pass if desired.

---

## §CR.14 Worked Examples

### §CR.14.1 First Example: Setup

Domain $[0, 16)$ ($N = 4$). Standard configuration ($T = \mathbb{R}_{\geq 0}$, all $g.\text{own} \geq 0$). G-Tree state after several observations and one partial eviction:

```
[0,16) own=5, sum=50
├── [0,8)  semi-internal (surviving child [4,8)), own=12, sum=32
│   └── [4,8) internal, own=3, sum=20
│       ├── [4,6) terminal, own=9, sum=9
│       └── [6,8) terminal, own=8, sum=8
└── [8,16) internal, own=6, sum=13
    ├── [8,12) terminal, own=4, sum=4
    └── [12,16) terminal, own=3, sum=3
```

The semi-internal `[0,8)` lost its left child `[0,4)` in a prior eviction. Its uncovered half is `[0,4)`, and its surviving child is `[4,8)`.

**Verification of G-I1:**

- `[4,8).sum = 3 + 9 + 8 = 20` ✓
- `[0,8).sum = 12 + 20 = 32` ✓ (semi-internal: one child only)
- `[8,16).sum = 6 + 4 + 3 = 13` ✓
- `[0,16).sum = 5 + 32 + 13 = 50` ✓

**Contour:** `[0,4)` at depth 1 (uncovered half of `[0,8)`), `[4,6)` and `[6,8)` at depth 3 (terminals, children of `[4,8)` at depth 2), `[8,12)` and `[12,16)` at depth 2.

**Plateaus:**

| Plateau | Depth | Contour tile | Basis        | Energy |
| ------- | ----- | ------------ | ------------ | ------ |
| $P_0$   | 1     | $[0,4)$      | $\{[0,8)\}$  | 32     |
| $P_1$   | 3     | $[4,8)$      | $\{[4,8)\}$  | 20     |
| $P_2$   | 2     | $[8,16)$     | $\{[8,16)\}$ | 13     |

$\mathcal{E} = \{0, 4, 8, 16\}$. Three plateaus.

Note: $P_0$'s basis element `[0,8)` is semi-internal. It was selected as a boundary element — its uncovered half `[0,4)` matches $P_0$'s tile, but its surviving child `[4,8)` extends into $P_1$'s territory. This is **boundary thatching**: $P_0$'s energy (32) includes `[4,8).sum` = 20, which also appears as $P_1$'s energy.

Sum of individual plateau energies: $32 + 20 + 13 = 65$.

---

### §CR.14.2 Contour Range: Full Domain

$\mathcal{R} = [0, 16)$. Algorithm: `[0,16)` is fully contained ($0 \leq 0$ and $16 \leq 16$). Basis $= \{[0,16)\}$. Energy $= 50$.

**Cross-plateau energy:**

$$E_\times = E(\mathcal{R}) - \sum_{j} E(P_j) = 50 - 65 = -15$$

The deficit of 15 decomposes into two competing effects:

- **Consolidation gain** $G = 5$: `[0,16).own` — the root is selected by the concatenation but by neither sub-range.
- **Resolved thatching** $T = 20$: $P_0$'s boundary thatching counted `[4,8).sum` = 20 in both $P_0$ and $P_1$; the full-domain range counts it once.
- Net: $G - T = 5 - 20 = -15$. ✓

**Thatching dominates.** The resolved double-counting (20) far exceeds the gained ancestor `.own` (5).

---

### §CR.14.3 Contour Range: Left Half

$\mathcal{R} = [0, 8)$. Algorithm:

- `[0,16)` partially overlaps `[0,8)`.
- Left child `[0,8)`: $0 \leq 0$ and $8 \leq 8$ → fully contained → **basis element**.
- Right child `[8,16)`: `g.l = 8 ≥ end = 8` → out of range → skip.

Basis $= \{[0,8)\}$. Energy $= 32$.

This is a two-plateau range ($P_0$ and $P_1$). Sum of individual energies: $32 + 20 = 52$.

$$E_\times = 32 - 52 = -20$$

- Resolved thatching $T = 20$: `[0,8)` as a fully-contained element subsumes both $P_0$'s thatching semi-internal and $P_1$'s basis element `[4,8)`.
- Consolidation gain $G = 0$: `[0,8)` was already $P_0$'s basis element — no new ancestor `.own` is gained.
- Net: $0 - 20 = -20$. ✓

---

### §CR.14.4 Contour Range: Right Half

$\mathcal{R} = [8, 16)$. Algorithm: `[8,16)` fully contained → basis element.

Basis $= \{[8,16)\}$. Energy $= 13$.

Single-plateau range ($P_2$ only). $E_\times = 13 - 13 = 0$. No interior boundaries, no effects.

---

### §CR.14.5 Concatenation

Concatenate $\mathcal{R}_1 = [0, 8)$ and $\mathcal{R}_2 = [8, 16)$:

- $E(\mathcal{R}_1) = 32$
- $E(\mathcal{R}_2) = 13$
- $E(\mathcal{R}_1 \cdot \mathcal{R}_2) = E([0, 16)) = 50$

Checking:

$$E(\text{concat}) = E(\mathcal{R}_1) + E(\mathcal{R}_2) + G - T = 32 + 13 + 5 - 0 = 50 \;\checkmark$$

- Consolidation gain at join point 8: $G = 5$ (`[0,16).own`).
- Resolved thatching at join point 8: $T = 0$ (no semi-internal straddles the join).

**Consolidation gain dominates.** The concatenation is _more_ energetic than the sum of its parts because `[0,16).own` = 5 was invisible to both sub-ranges.

---

### §CR.14.6 Nesting

Inner $= [8, 16)$, outer $= [0, 16)$.

$$E(\text{inner}) = 13 \leq 50 = E(\text{outer}) \;\checkmark$$

Inner $= [0, 8)$, outer $= [0, 16)$.

$$E(\text{inner}) = 32 \leq 50 = E(\text{outer}) \;\checkmark$$

Inner $= [4, 8)$, outer $= [0, 8)$. (Both valid: $4, 8 \in \mathcal{E}$.)

$$E([4,8)) = 20 \leq 32 = E([0,8)) \;\checkmark$$

The gap of 12 is `[0,8).own` = 12. The wider range captures this because `[0,8)` is its basis element; the narrower range's basis element is `[4,8)` (a descendant).

---

### §CR.14.7 Context-Dependent Coverage

The G-node `[0,8)` (semi-internal) appears in the bases of three ranges computed above, with different roles in each:

| Range          | Selection path                                                    | Coverage of `[0,8)` | Role                                    |
| -------------- | ----------------------------------------------------------------- | ------------------- | --------------------------------------- |
| $P_0 = [0, 4)$ | Boundary selection (uncovered half `[0,4)` overlaps $P_0$'s tile) | $[0, 4)$            | Boundary element; thatches into `[4,8)` |
| $[0, 8)$       | Fully contained ($0 \leq 0$ and $8 \leq 8$)                       | $[0, 8)$            | Interior element; no thatching          |
| $[0, 16)$      | Not selected (subsumed by `[0,16)`)                               | N/A                 | Not a basis element                     |

In $P_0$'s basis, `[0,8)`'s coverage is `[0,4)` — only the uncovered half. In the contour range `[0,8)`'s basis, the same node's coverage is `[0,8)` — the full interval. The surviving child `[4,8)` is inside the range, so the fully-contained check fires.

---

### §CR.14.8 Exact Energy vs. Contour Range Energy

Exact energy for $[0, 8)$ (§CR.13):

```
range_sum([0,16), 0, 8):
    [0,16) partially overlaps. own_contribution = 5 × 8/16 = 2.5
    range_sum([0,8), 0, 8):
        Fully contained. Return 32.
    range_sum([8,16), 0, 8):
        g.l = 8 ≥ end = 8. Return 0.
    Return 2.5 + 32 + 0 = 34.5
```

$\text{exact\_energy}([0, 8)) = 34.5$. $E(\mathcal{R}) = 32$.

$$34.5 > 32 \implies \text{exact\_energy} > E(\mathcal{R})$$

Decomposition per §CR.13.6: $E = \text{exact} + B - A$.

- $B = 0$: no boundary thatching (the sole basis element `[0,8)` is fully contained — its entire interval is within $[0,8)$).
- $A = 2.5$: the pro-rated root `.own` ($5 \times 8/16$). The root `[0,16)` is not a basis element; its `.own` contribution appears in exact energy but not in contour range energy.
- Check: $32 = 34.5 + 0 - 2.5$. ✓

---

### §CR.14.9 Summary of Numerical Results (First Example)

| Range     | Basis                 | $E(\mathcal{R})$ | $\sum E(P_j)$ | $E_\times$ | Dominant effect      |
| --------- | --------------------- | ---------------- | ------------- | ---------- | -------------------- |
| $[0, 16)$ | $\{[0,16)\}$          | 50               | 65            | −15        | Thatching resolution |
| $[0, 8)$  | $\{[0,8)\}$           | 32               | 52            | −20        | Thatching resolution |
| $[8, 16)$ | $\{[8,16)\}$          | 13               | 13            | 0          | None                 |
| $[4, 8)$  | $\{[4,8)\}$           | 20               | 20            | 0          | None                 |
| $[4, 16)$ | $\{[4,8),\; [8,16)\}$ | 33               | 33            | 0          | None                 |

**Concatenation demonstrations:**

| $\mathcal{R}_1$ | $\mathcal{R}_2$ | $E(\mathcal{R}_1) + E(\mathcal{R}_2)$ | $E(\text{concat})$ | $G$ | $T$ | Direction     |
| --------------- | --------------- | ------------------------------------- | ------------------ | --- | --- | ------------- |
| $[0,8)$         | $[8,16)$        | 45                                    | 50                 | 5   | 0   | Gain > thatch |
| $[0,4)$         | $[4,8)$         | 52                                    | 32                 | 0   | 20  | Thatch > gain |
| $[4,8)$         | $[8,16)$        | 33                                    | 33                 | 0   | 0   | Neither       |

The three cases demonstrate all three regimes: gain-dominant, thatch-dominant, and balanced.

---

### §CR.14.10 Second Example: Semi-Internal Early Selection

This example demonstrates the semi-internal early selection guard (§CR.8.1) on a contour range where the range boundary falls inside the present-child territory of a semi-internal node.

**G-Tree state.** Domain $[0, 16)$ ($N = 4$).

```
Root [0,16) internal, own=10, sum=60
├── [0,8) semi-internal (LEFT child [0,4) present, right ABSENT), own=15, sum=45
│   └── [0,4) internal, own=10, sum=30
│       ├── [0,2) terminal, own=8, sum=8
│       └── [2,4) internal, own=6, sum=12
│           ├── [2,3) terminal, own=4, sum=4
│           └── [3,4) terminal, own=2, sum=2
└── [8,16) terminal, own=5, sum=5
```

The semi-internal `[0,8)` has its left child `[0,4)` present and its right child `[4,8)` absent. Its uncovered half is `[4,8)`. The present child's subtree has non-uniform depth — the contour changes at coordinate 2.

**Verification of G-I1:**

- `[2,3).sum = 4` ✓ `[3,4).sum = 2` ✓
- `[2,4).sum = 6 + 4 + 2 = 12` ✓
- `[0,2).sum = 8` ✓
- `[0,4).sum = 10 + 8 + 12 = 30` ✓
- `[0,8).sum = 15 + 30 = 45` ✓
- `[8,16).sum = 5` ✓
- `[0,16).sum = 10 + 45 + 5 = 60` ✓

**Contour:** `[0,2)` at depth 3, `[2,3)` and `[3,4)` at depth 4, `[4,8)` at depth 1 (uncovered by `[0,8)`), `[8,16)` at depth 1.

**Plateaus:**

| Plateau | Depth | Contour tile | Basis                 | Energy |
| ------- | ----- | ------------ | --------------------- | ------ |
| $P_0$   | 3     | $[0,2)$      | $\{[0,2)\}$           | 8      |
| $P_1$   | 4     | $[2,4)$      | $\{[2,4)\}$           | 12     |
| $P_2$   | 1     | $[4,16)$     | $\{[0,8),\; [8,16)\}$ | 50     |

$\mathcal{E} = \{0, 2, 4, 16\}$. Three plateaus. $P_2$'s basis element `[0,8)` is the semi-internal — selected because its uncovered half `[4,8)` falls in $P_2$'s tile. Its `.sum = 45` includes `[0,4).sum = 30`, which is thatching from $P_0$ and $P_1$ territory.

Sum of individual plateau energies: $8 + 12 + 50 = 70$.

---

### §CR.14.11 Contour Range [2, 16) — With Early Selection

The contour range $[2, 16)$ spans $P_1$ and $P_2$: $a_s = 2 \in \mathcal{E}$, $a_{e+1} = 16 \in \mathcal{E}$.

**Algorithm trace:**

```
decompose([0,16), 2, 16):
  Not fully contained (2 > 0). m = 8.
  Left child [0,8):
    Not fully contained (2 > 0). m = 4.
    Semi-internal check: left child [0,4) present, right child null.
      left_absent = false, right_absent = true → semi-internal.
      left_overlaps: max(0, 2) = 2 < min(4, 16) = 4? YES.
      right_overlaps: max(4, 2) = 4 < min(8, 16) = 8? YES.
    ★ Both overlap → early selection. ADD [0,8). Return.

  Right child [8,16):
    Fully contained (2 ≤ 8 and 16 ≤ 16). ADD [8,16). Return.
```

**Result:** basis $= \{[0,8),\; [8,16)\}$. Energy $= 45 + 5 = 50$.

**Boundary thatching:** `[0,8)` covers `[0,8)` but the range starts at 2. Territory $[0,2)$ is outside the range but included in `[0,8).sum`. This is start-boundary thatching — the same phenomenon as single-plateau thatching, now from early selection instead of boundary selection.

**Nesting:** $E([2,16)) = 50 \leq 60 = E([0,16))$. ✓

---

### §CR.14.12 What Would Go Wrong Without the Guard

Without the semi-internal early selection, the algorithm would proceed as follows on the same range $[2, 16)$:

```
decompose([0,8), 2, 16):
  Not fully contained. m = 4.
  [NO GUARD — proceed to standard recursion]
  Left child [0,4) exists → recurse:
    decompose([0,4), 2, 16):
      Not fully contained (2 > 0). m = 2.
      Left child [0,2): g.r = 2 ≤ start = 2. Return.
      Right child [2,4): fully contained (2 ≤ 2, 4 ≤ 16). ADD [2,4).
  Right child of [0,8): NULL.
    max(4, 2) = 4 < min(8, 16) = 8? YES. ADD [0,8). Return.
```

**Buggy result:** basis $= \{[2,4),\; [0,8),\; [8,16)\}$. Energy $= 12 + 45 + 5 = 62$.

**Double-counting:** `[0,8).sum = 45` includes `[0,4).sum = 30`, which includes `[2,4).sum = 12`. The energy from `[2,4)` is counted twice: once directly (12) and once as a component of `[0,8).sum`.

**CR-I4 violated:** `[0,8)` is a proper ancestor of `[2,4)`; both are in the basis.

**Nesting violated:** $E([2,16)) = 62 > 60 = E([0,16))$ — a sub-range has more energy than the full domain.

The early selection guard prevents this by selecting `[0,8)` at the semi-internal check and returning _before_ recursing into `[0,4)`.

---

### §CR.14.13 Contour Range [0, 8) — No Guard Needed

$\mathcal{R} = [0, 8)$. `[0,8)` is fully contained ($0 \leq 0$, $8 \leq 8$). The fully-contained check fires before the semi-internal guard is reached. Basis $= \{[0,8)\}$, energy $= 45$. Identical to the original algorithm.

---

### §CR.14.14 Energy Decomposition (Second Example)

For the contour range $[2, 16)$ with the corrected basis $\{[0,8),\; [8,16)\}$:

Exact energy:

```
range_sum([0,16), 2, 16):
  [0,16) partially overlaps. own share = 10 × 14/16 = 8.75.
  Left child [0,8):
    [0,8) partially overlaps. own share = 15 × 6/8 = 11.25.
    Left child [0,4):
      [0,4) partially overlaps. own share = 10 × 2/4 = 5.
      [0,2): g.r = 2 ≤ start = 2. Return 0.
      [2,4): fully contained. Return 12.
      Return 5 + 0 + 12 = 17.
    Right child: null. Return 0.
    Return 11.25 + 17 = 28.25.
  Right child [8,16): fully contained. Return 5.
  Return 8.75 + 28.25 + 5 = 42.
```

$\text{exact\_energy}([2,16)) = 42$. $E(\mathcal{R}) = 50$.

Decomposition per §CR.13.6:

$$E = \text{exact} + B - A$$

$B$ = boundary thatching from `[0,8)` covering territory $[0, 2)$ outside the range.

$$B = \text{range\_sum}([0,8), 0, 2) = 15 \times \tfrac{2}{8} + \text{range\_sum}([0,4), 0, 2)$$
$$= 3.75 + \bigl(10 \times \tfrac{2}{4} + 8\bigr) = 3.75 + 13 = 16.75$$

$A$ = ancestor pro-ration for `[0,16)` (not in basis, partially overlapping).

$$A = 10 \times 14/16 = 8.75$$

Check: $50 = 42 + 16.75 - 8.75 = 50$. ✓

---

### §CR.14.15 Summary of Numerical Results (Second Example)

| Range     | Basis                | $E(\mathcal{R})$ | $\text{exact}$ | $B$   | $A$  | Check         |
| --------- | -------------------- | ---------------- | -------------- | ----- | ---- | ------------- |
| $[0, 16)$ | $\{[0,16)\}$         | 60               | 60             | 0     | 0    | 60 = 60 ✓     |
| $[2, 16)$ | $\{[0,8),\;[8,16)\}$ | 50               | 42             | 16.75 | 8.75 | 50 = 42 + 8 ✓ |
| $[0, 8)$  | $\{[0,8)\}$          | 45               | 45             | 0     | 0    | 45 = 45 ✓     |

The second row demonstrates a case where $B > A$ (boundary thatching exceeds ancestor pro-ration), so $E(\mathcal{R}) > \text{exact\_energy}$.

**Nesting verification (second example):**

| Inner     | Outer     | $E(\text{inner})$ | $E(\text{outer})$ | ✓              |
| --------- | --------- | ----------------- | ----------------- | -------------- |
| $[2, 16)$ | $[0, 16)$ | 50                | 60                | $50 \leq 60$ ✓ |
| $[0, 8)$  | $[0, 16)$ | 45                | 60                | $45 \leq 60$ ✓ |

# The Progressive Entropic-Wavelet Exposure Image

## Output of the Dual-Tree Value-Stratified Index

> _Convention._ References to the main specification use the prefix `§IDEA` (e.g., §IDEA M-12.5). Unqualified `§` references are internal to this companion document.

---

## Chapter 1. What It Is

The Dual-Tree Value-Stratified Index (G-V Graph) is a living structure. It grows under observation, contracts under decay, and continuously restructures through competition. At any moment, the graph holds a state of knowledge — what has been measured, where structure has been confirmed, and what remains indistinguishable from noise.

The **Progressive Entropic-Wavelet Exposure Image** (PEWEI) is a static reading of that state. It is to the G-V Graph what a photograph is to a scene: a snapshot of a dynamic process, taken at a moment, encoding both the accumulated measurement and the history of the measurement process itself.

The PEWEI is extracted by a single breadth-first walk of the V-Tree (§9), reading fields that already exist on every node. Two lightweight derived quantities — refinement and a baseline ratio — are computed per phase transition node. The competitive ranking, the spatial decomposition, and the benchmark calibration were all performed by the same observations that built the graph. The PEWEI reads them out in significance order.

A consequence of this design is that the same observations serve both as the signal and as the calibration data. This is a structural property, not an incidental one: the frozen benchmarks that provide scale-adaptive baselines were deposited by the same measurements they calibrate. Whether this circularity is an advantage (no separate estimation pass, perfect scale-locality) or a limitation (no independent noise reference) depends on the application. §6 develops this trade-off.

---

## Chapter 2. Structure

A PEWEI consists of an ordered sequence of **layers**, each corresponding to a V-Tree depth. Within each layer, two populations of nodes appear:

**Phase transition nodes.** These are V-entries backed by G-nodes with confirmed sub-scale structure (internal or semi-internal — at least one G-child). Each carries:

| Field                  | Source         | Meaning                                                  |
| ---------------------- | -------------- | -------------------------------------------------------- |
| Region $[l, r)$        | G-node range   | The spatial domain of this transition                    |
| Baseline $B$           | $g.\text{own}$ | Direct accumulation at this node (see note below)        |
| Total $S$              | $g.\text{sum}$ | Total energy in this region including all refinement     |
| Refinement $R = S - B$ | Derived        | Energy attributed to confirmed sub-scale structure       |
| Baseline ratio $R / B$ | Derived        | Refinement relative to baseline ($B > 0$ guard required) |

**Terminal nodes.** These are V-entries backed by terminal G-nodes (zero children). Each carries:

| Field           | Source                        | Meaning                                           |
| --------------- | ----------------------------- | ------------------------------------------------- |
| Region $[l, r)$ | G-node range                  | The spatial domain of this measurement            |
| Intensity $I$   | $g.\text{own} = g.\text{sum}$ | Direct measurement at finest available resolution |

Phase transition nodes record **where and when the signal revealed internal structure**. Terminal nodes record **the current finest-resolution measurement**.

> **Baseline semantics depend on G-node state.** The baseline $B = g.\text{own}$ carries different provenance for different node states:
>
> | G-node state            | Baseline contains                                                                                        | Character                                 |
> | ----------------------- | -------------------------------------------------------------------------------------------------------- | ----------------------------------------- |
> | Internal (2 children)   | Pre-split accumulation only                                                                              | Frozen historical record                  |
> | Semi-internal (1 child) | Pre-split accumulation + absorbed evicted child's sum + post-eviction observations in the uncovered half | Hybrid of history and ongoing measurement |
>
> For fully internal nodes, $B$ is a clean pre-refinement measurement — the total energy observed before the region was subdivided. For semi-internal nodes, $B$ is contaminated by post-split events: it includes energy folded in from the evicted child (§IDEA M-12.5 Step 1) and fresh observations in the vacated half. The baseline ratio $R / B$ has correspondingly different interpretations in the two cases. §6 develops the consequences.
>
> The baseline ratio is undefined when $B = 0$. Under the standard configuration (P0–P5) with non-negative observations, $B > 0$ at split time (the node exceeded the split threshold $\theta > 0$). Under user-applied attenuation ($\text{att} < 1$, §IDEA M-14.3), $B$ may attenuate toward zero after splitting. Under amplification ($\text{att} > 1$), $B$ grows — the frozen baseline strengthens, raising the bar for the baseline ratio. Under annihilation ($\text{att} = 0$), $B$ is reset to zero; the baseline ratio becomes undefined until re-accumulation. The extraction algorithm (§9) guards against division by zero.

---

## Chapter 3. The Layer Ordering

The PEWEI emits one layer per V-Tree depth. The V-Tree's competitive mechanism places high-importance entries at shallow depths and low-importance entries deep. The PEWEI's layer ordering inherits this:

```
Layer 0 (V-depth 0):
    The V-root. Typically structural (after the first split),
    contributing no entries. In a single-entry tree (before
    the first split), the sole entry appears here.

Layer 1 (V-depth 1):
    The V-root's children. Dominant energy structures.
    Highest importance. Typically the first non-empty layer.

Layer 2 (V-depth 2):
    Secondary structures that proved themselves
    against Layer 1's competitive benchmarks.

Layer k (V-depth k):
    Progressively finer detail.
    Each entry earned its position by outgrowing
    its uncles through the competitive mechanism.

Final layers:
    The finest confirmed detail.
    Marginally significant.
    Deepest in the V-Tree tournament.
```

Early layers may be empty — containing only structural scaffolding whose entry-children appear at the next depth. This is normal. The first non-empty layer in a typical tree is layer 1.

The V-Tree depth bound (§IDEA M-18) guarantees that this ordering is near-optimal in the information-theoretic sense:

$$E[\text{cost to reach entry } i] \leq \frac{H}{\log_2 \varphi} + O(1) \approx 1.44\, H + O(1)$$

where $H$ is the Shannon entropy of the importance distribution. The ordering is within factor 1.44 of the entropy lower bound. This bound is on **expected sampling cost**, not on exact significance ordering: two entries at the same V-Tree depth may have different importances, and an entry at depth $k+1$ may outweigh one at depth $k$ without being in violation (its uncle is still heavier). The layer ordering is **approximately** significance-ordered, governed by the $1/\log_2\varphi$ overhead of the competitive mechanism.

---

## Chapter 4. Progressive Reconstruction

A PEWEI supports reconstruction at any truncation depth, but reconstruction is not naive summation. Because phase transition nodes' totals ($g.\text{sum}$) include their descendants' contributions, simply summing all visible nodes would double-count. A top-down reconstruction algorithm (§10) resolves this by distinguishing each transition's **baseline** ($g.\text{own}$, used as additive background when children are visible) from its **total** ($g.\text{sum}$, used as a lump substitute when children are truncated). The mechanism is developed fully in §10; the properties that result are:

- **Total energy is exact when the domain is covered.** The G-Tree's summation invariant (G-I1) guarantees that a parent's sum equals its own accumulation plus its children's sums. When the reconstruction algorithm substitutes a parent's total for truncated children, the energy is exactly preserved. This guarantee requires that the truncated PEWEI includes enough entries to cover the full domain — in practice, that the G-root's entry (or equivalent covering entries) is visible at the truncation depth. In typical trees, the G-root's entry sits at V-depth 1, so $\text{max\_layer} \geq 1$ suffices.

- **Each layer adds approximately the next-most-significant detail.** The V-Tree's competitive ordering ensures that shallow layers carry high-importance entries. Adding layer $k+1$ refines the estimate by resolving structure within regions that were previously represented by a parent's lump total. The approximation qualifier reflects the V-Tree's near-optimal (not exact) significance ordering.

- **The reconstruction is a near-optimal progressive approximation.** The first $k$ layers capture approximately the highest-energy structures, within the $1.44\times$ overhead of the competitive mechanism. This follows from the V-Tree depth bound: entries at shallow depth have high weight fractions, so early layers are biased toward maximum energy capture. "Near-optimal" means within the proven $1/\log_2\varphi$ factor of the information-theoretic bound — not that the first $k$ layers are the unique best $k$-term approximation.

```
Truncate at layer 0:
    Typically empty. Reconstruction outputs zero or
    uses whatever entries exist at V-depth 0.

Truncate at layer 1:
    Dominant structures visible. Correct total
    when the G-root's entry is included.
    No internal structure resolved.

Truncate at layer k:
    All detail validated through k levels of competition.
    Unresolved regions carry their parent's total,
    uniformly distributed — the best estimate
    available at the truncated resolution.

Full depth:
    Complete measurement at finest confirmed resolution.
    Everything below the finest frozen benchmark
    was adjudicated by the competitive mechanism.
```

**Reconstruction is additive, not replacement.** Each transition's baseline persists as uniform background beneath finer-scale detail added by its children. §10.2 develops why naive replacement is incorrect and demonstrates the energy accounting failure it produces.

---

## Chapter 5. The Wavelet Decomposition

The G-Tree is an adaptive multi-resolution decomposition of the domain $[0, 2^N)$. At each internal G-node, the structure admits a parallel — structural but not algebraically exact — to wavelet transforms:

| Standard Haar wavelet                         | G-Tree structural parallel                               |
| --------------------------------------------- | -------------------------------------------------------- |
| Scaling coefficient (coarse approximation)    | $g.\text{own}$ — the pre-refinement measurement          |
| Detail coefficient (fine correction)          | Children's sum asymmetry: $S_L - S_R$                    |
| Zero coefficient (no structure at this scale) | Evicted children — the tree found nothing and contracted |

The parallel is structural, not algebraic. A Haar node carries two values (scaling + detail) and reconstructs two children exactly. A G-Tree node carries three independent quantities: $g.\text{own}$, $S_L$, and $S_R$, linked by G-I1 ($g.\text{sum} = g.\text{own} + S_L + S_R$). Knowing $g.\text{own}$ and $S_L - S_R$ is not sufficient for reconstruction — the third degree of freedom ($S_L + S_R$, equivalently $g.\text{sum}$) is also needed. The G-Tree decomposition is richer than a wavelet transform: at each scale, it separates the pre-refinement energy ($g.\text{own}$) from the post-refinement energy ($S_L + S_R$), a distinction wavelets do not make.

The decomposition is **adaptive** in two ways:

1. **Spatially adaptive.** The G-Tree materialises nodes only where observations warranted refinement. Regions where the signal is smooth have coarse cells. Regions where the signal has structure have fine cells. The sparsity pattern was determined by the signal itself.

2. **Approximately significance-ordered.** The V-Tree ranks every coefficient by its competitive importance. Standard wavelet transforms produce all coefficients at all scales and then sort them. The G-V Graph produces coefficients as they are created and the V-Tree maintains an approximate significance ordering dynamically.

**Benchmark cascading.** Under typical operation, frozen benchmarks form an approximate cascade of quantisation contexts across scales (§IDEA M-10.2, §IDEA M-13.5):

```
Scale 0: Benchmark B₀ — pre-refinement energy at the coarsest level
    ↓
Scale 1: Benchmark B₁ — earned competitive promotion (typically
         by outgrowing B₀ as uncle, though the V-Tree topology
         may route through a different competitive path)
    ↓
Scale 2: Benchmark B₂ — earned promotion against its uncles
    ↓
    ...
```

Each benchmark was earned through the competitive mechanism — the entry accumulated enough importance to outgrow its uncles and trigger promotion, which authorised the split that froze the benchmark. The entry's uncle is **typically** the direct G-parent's frozen entry, but the relevant uncle depends on V-Tree topology, which is shaped by the full competitive history. The cascade is the common case under spatially coherent observation patterns, not a structural guarantee. The contexts are approximately self-calibrating — deposited by the same measurements they describe, at approximately the scale where calibration is needed.

Under attenuation (§IDEA M-14.3), benchmarks in the cascade attenuate at rates determined by their G-depth — deeper benchmarks fade faster under depth-selective decay ($q > 0$), causing the cascade to soften from the bottom up. Under amplification with $q > 0$, the cascade sharpens from the bottom up — deep benchmarks strengthen faster than shallow ones. Under annihilation, the cascade is flattened — all benchmarks reset to zero, and the tree must rebuild the entire cascade from new observations.

---

## Chapter 6. Pre-Refinement Baselines and Their Interpretation

The PEWEI embeds a pre-refinement energy datum at every scale and location. No separate estimation step produced these values — they are a structural consequence of the catalytic split mechanism. This section presents what the baselines are (§6.1), one natural interpretation as noise floors (§6.2), and the trade-offs of self-calibration (§6.3).

### 6.1 Frozen Baselines as Datums

A frozen baseline $B$ at region $[l, r)$ records the energy level that accumulated before the region was subdivided. For a fully internal G-node, this is exact: $B = g.\text{own}$ is the total of all observations received at this node while it was terminal, frozen at the moment both children were created (§IDEA M-10.2). For a semi-internal G-node, $B$ is hybrid (§2 note).

The baseline is a direct measurement. It is not an estimate, not a model parameter, and not derived from a noise model. It is the literal pre-refinement energy at this scale and location.

### 6.2 Noise-Floor Interpretation

Under an appropriate signal model, the baseline admits interpretation as a noise floor. The reasoning:

> Under the null hypothesis that the region $[l, r)$ is spatially uniform, each sub-region should contain energy proportional to its area. The frozen value $B$ is the total measurement consistent with this null hypothesis — it was accumulated before the system could distinguish left from right.
>
> Any child that exceeded $B$ (via competitive promotion, §IDEA M-11.5) has demonstrated structure beyond the undifferentiated baseline. Any child that failed to exceed $B$ was evicted (§IDEA M-12) — the tree adjudicated it as insufficiently distinguished from the baseline.

This interpretation is sound for **intensity-proportional noise regimes** — applications where the noise level scales with the signal (Poisson processes, photon counting, event counting). For such regimes, $B$ is both the expected background and the natural scale for noise variation.

For **additive noise regimes** (Gaussian noise independent of signal level), the baseline $B$ is the total pre-refinement energy, but the noise floor is determined by variance, not by mean intensity. In this regime, $B$ is a valid datum but not directly a noise floor without an additional variance estimate.

For each terminal node $T$ in the PEWEI, there exists a chain of ancestral phase transitions leading back to the root:

```
Phase transition F₁ at scale 0: baseline B₁
    Phase transition F₂ at scale 1: baseline B₂
        Phase transition F₃ at scale 2: baseline B₃
            Terminal T: intensity I

Relative intensity profile of T:
    Against scale 0: I / B₁   (intensity relative to global baseline)
    Against scale 1: I / B₂   (intensity relative to regional baseline)
    Against scale 2: I / B₃   (intensity relative to local baseline)
```

Each ratio is a scale-specific **relative intensity** measurement for the terminal node's value — how much this terminal's energy exceeds (or falls below) the pre-refinement baseline at each ancestral scale. These are not confidence intervals or p-values; they are deterministic intensity ratios. Converting them to statistical confidence requires a noise model the PEWEI does not supply.

### 6.3 Self-Calibration: Strengths and Limitations

The baselines are produced from the same observations they calibrate, without a separate estimation pass. This has concrete advantages:

- **Exact.** Each baseline is a direct measurement, not a statistical estimate.
- **Spatially adaptive.** Each region has its own baseline.
- **Scale-adaptive.** Each level of refinement has its own baseline.
- **Zero overhead.** No separate calibration pass, no auxiliary data structure.

It also has a structural limitation:

- **Circular.** The same data that determines signal structure also determines the calibration thresholds. The frozen benchmark at scale $k$ was the evidence that justified splitting at scale $k$; the children's energy is then measured against this same value. In standard denoising, noise is estimated from a holdout or from a statistically independent summary (e.g., median absolute deviation of fine-scale coefficients). The PEWEI has no such independence.

The circularity is mitigated by the scale separation that the cascading benchmarks provide — the benchmark at scale $k$ was frozen _before_ the children at scale $k+1$ were measured, so the benchmark is temporally independent of the children's specific values. But the benchmark was determined by the same underlying process generating the children's values. Whether this is sufficient depends on the application's tolerance for self-referential calibration.

---

## Chapter 7. Coding Structure

The G-V Graph distributes coding-theoretic content across both trees. The G-Tree is the code. The V-Tree is the significance map that governs its shape.

**The G-Tree is a dynamic spatial code.** Its bottom contour partitions $[0, 2^N)$ into dyadic cells whose widths sum to the domain:

$$\sum_{\text{cells}} 2^{-d_i} = 1$$

This base-2 Kraft equality holds unconditionally — a structural consequence of the contour being a complete partition. Every contour mutation (refinement, restoration, eviction) preserves it. The G-Tree is a valid, complete prefix code over the domain at all times.

**The allocation is rate-distortion, not source coding.** Source coding assigns short codewords to frequent symbols — compression. The G-Tree does the inverse: high-intensity regions receive deep cells (long codes, fine spatial resolution), while low-intensity regions receive shallow cells (short codes, coarse resolution). Precision is allocated _toward_ distributional mass, not away from it. The G-Tree is a description code — it represents a distribution, not compresses a message. This framing is conceptual; the specification does not formalise a rate-distortion objective or prove optimality in that sense.

**The V-Tree is the significance ordering.** It ranks the G-Tree's decomposition coefficients by competitive importance. The uncle constraint forces a minimum information gain of $\log_2\varphi \approx 0.694$ bits per sampling step, guaranteeing that each step of the V-Tree walk contributes meaningfully to identifying the target entry. The expected cost to reach an entry with weight fraction $w_i$ is at most $\log_\varphi(1/w_i) + O(1)$ steps; the expected sampling cost across the distribution is at most $1.44\,H + O(1)$.

| Coding-theoretic concept     | G-V Graph mechanism                                   | Owner   |
| ---------------------------- | ----------------------------------------------------- | ------- |
| Code structure               | Base-2 Kraft equality (contour partition)             | G-Tree  |
| Code length                  | G-Tree depth $d_i$ (spatial precision per cell)       | G-Tree  |
| Allocation direction         | Rate-distortion: precision toward mass                | G-Tree  |
| Decomposition coefficients   | $g.\text{own}$ at each scale and location             | G-nodes |
| Significance ordering        | V-Tree depth (competitive rank)                       | V-Tree  |
| Ordering efficiency          | $\leq 1.44\,H + O(1)$ expected cost                   | V-Tree  |
| Ordering overhead            | $1/\log_2 \varphi \approx 1.44\times$ Shannon entropy | V-Tree  |
| Progressive readout          | Breadth-first V-Tree walk                             | V-Tree  |
| Dynamic codebook maintenance | Competitive rebalancing (local operations)            | V-Tree  |

The progressive readout of the PEWEI is a traversal of the V-Tree's approximate significance ordering — high-importance coefficients first. The V-Tree determines which coefficients exist, where they approximately rank, and in what order they are emitted, while the G-Tree ensures the underlying spatial code is always complete and always valid. The golden ratio governs how efficiently the significance map adapts to changing data — the cost of maintaining the ranking, not the structure of the code it ranks.

---

## Chapter 8. Relation to Embedded Wavelet Coding

The PEWEI admits a conceptual mapping to the embedded wavelet coding paradigm. The parallels are genuine but not exact — several structural differences exist where the G-V Graph's mechanism is richer than the standard framework.

| Embedded coding concept                       | PEWEI parallel                                   | Structural difference                                                                                                                                                           |
| --------------------------------------------- | ------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Coefficient tree (parent-child across scales) | G-Tree (dyadic containment)                      | Same structure.                                                                                                                                                                 |
| Significance ordering across coefficients     | V-Tree depth (competitive ranking)               | Approximate ordering (within $1.44\times$), not exact.                                                                                                                          |
| Significant coefficient (exceeds threshold)   | Terminal node or promoted phase transition       | G-V threshold is competitive (uncle constraint), not a fixed magnitude test.                                                                                                    |
| Zerotree (no significant descendants)         | Evicted subtree — contracted, absorbed by parent | G-V decision uses V-Tree depth + structural immunity, not magnitude thresholding.                                                                                               |
| Significance threshold per pass               | Frozen benchmark $B$ at each phase transition    | **Major difference.** SPIHT uses a single global threshold halved each pass. G-V benchmarks are per-node, per-scale, and spatially adaptive. The G-V system is strictly richer. |
| Refinement pass (adding precision bits)       | Deeper V-Tree layers (adding finer-scale detail) | Comparable.                                                                                                                                                                     |
| Progressive bitstream                         | Breadth-first V-Tree walk                        | Comparable.                                                                                                                                                                     |

The structure that standard codecs like SPIHT build through explicit partitioning algorithms, the G-V Graph builds through competition. The significance ordering is maintained dynamically. Subtree-skipping decisions — which regions to leave coarse — are made by the eviction mechanism (§IDEA M-12), which contracts subtrees where observations failed to confirm structure. The PEWEI reads out the result.

The most significant structural difference is in the threshold mechanism. SPIHT's global-halving threshold is uniform across the domain and scale — every coefficient at every location faces the same bar. The G-V Graph's frozen benchmarks are **local**: each region has its own pre-refinement energy datum, and the competitive uncle constraint calibrates promotion thresholds via the specific V-Tree neighbourhood. This locality is both an advantage (spatially adaptive thresholding) and a cost (the ordering is approximate rather than exact).

---

## Chapter 9. Extraction

```
function extract_pewei(v_root) → PEWEI:
    pewei ← new PEWEI()
    queue ← [v_root]
    depth ← 0

    while queue is not empty:
        next_queue ← []
        layer ← new Layer(depth)

        for v in queue:
            if v is entry:
                g ← v.gnode
                if has_dependents(g):
                    // Phase transition node (internal or semi-internal)
                    layer.add_transition(
                        region = [g.l, g.r),
                        baseline = g.own,
                        total = g.sum,
                        refinement = g.sum − g.own,
                        baseline_ratio =
                            (g.sum − g.own) / g.own if g.own > 0
                            else undefined
                    )
                else:
                    // Terminal node (zero children)
                    layer.add_terminal(
                        region = [g.l, g.r),
                        intensity = g.own
                    )

            if v is structural:
                for c in v.children:
                    next_queue.append(c)

        if not layer.is_empty():
            pewei.add_layer(layer)
        // Empty layers (structural scaffolding only) are omitted
        // from the output. Layer indices in the PEWEI correspond
        // to V-Tree depths but may skip depths where no entries exist.

        queue ← next_queue
        depth ← depth + 1

    return pewei
```

Cost: $O(L + S)$ where $L$ is the number of V-entries and $S$ the number of V-Structural nodes. Every V-node visited exactly once. Every field read already exists on the node. Derived quantities (refinement and baseline ratio) are $O(1)$ per phase transition node.

Layers are indexed by V-Tree depth. The V-root is typically structural (after the first split), so depth 0 produces no entries — the first non-empty layer is typically depth 1. Empty layers are omitted from the output sequence to avoid confusing consumers with leading blanks.

---

## Chapter 10. Reconstruction From Truncated PEWEI

Given the first $k$ layers of a PEWEI, reconstruct the signal over $[0, 2^N)$.

### 10.1 The Additive Property

Reconstruction is **additive across scales, not replacement.** Each transition's baseline ($g.\text{own}$) is a uniform background layer over its region. Children add finer-scale structure on top of that background — they do not replace it. The baseline is real energy (observations received before the node split) that must persist through all finer scales.

G-I1 gives the accounting identity:

$$g.\text{sum} = g.\text{own} + \sum_{c \in \text{children}(g)} c.\text{sum}$$

For reconstruction this means:

- **Expanded** (G-children visible at the truncation depth): contribute baseline as pro-rated background to each child's region. Children contribute their own values recursively. Total = baseline + Σ children = $g.\text{sum}$ (by G-I1). ✓
- **Truncated** (G-children not visible): contribute total = $g.\text{sum}$ directly. Total = $g.\text{sum}$. ✓

Both cases yield the same total energy for the region. The pro-rating of baselines distributes ancestral energy uniformly within the region — a necessary approximation, since the ancestor's observations arrived before the region was spatially distinguished. This is the same uniform-within-cell assumption used by the G-Tree's range queries (§IDEA M-5.5.2).

### 10.2 Why Replacement Is Wrong

A flat-overwrite model (stamp total, let children replace) breaks energy conservation:

```
[0,8): own=10, sum=33
  [0,4): terminal, intensity=15
  [4,8): terminal, intensity=8
```

Replacement: stamp `[0,8) → 33`, then replace `[0,4) → 15`, `[4,8) → 8`. Grand total = 23. **Missing 10** — the root's baseline.

Additive: background 10 over `[0,8)`, pro-rated `10/2 = 5` per half, plus children. `[0,4) = 5 + 15 = 20`, `[4,8) = 5 + 8 = 13`. Grand total = 33. ✓

The root's pre-split energy was real measurement that does not vanish when children appear. It persists as a uniform background that children refine.

### 10.3 Algorithm

**Region lookup.** Before reconstruction, build a lookup table mapping each region to its PEWEI node (transition or terminal). Dyadic intervals are uniquely identified by their endpoints, so the mapping is unambiguous. The table supports:

- `get(region)` — return the PEWEI node at this region, or null if absent.
- `contains(region)` — membership test.
- `total(region)` — return the node's total energy ($g.\text{sum}$ for transitions, $g.\text{own}$ for terminals).

```
function reconstruct(pewei, max_layer) → list of spans over [0, 2^N):
    lookup ← build region → node map from pewei.layers[0..max_layer]
    spans ← []
    descend(lookup, root_region, accumulated=0, spans)
    return spans

function descend(lookup, region, accumulated, spans):
    // accumulated = sum of all ancestral baselines, pro-rated to this region.
    // This is the uniform background contributed by every ancestor
    // that has already been "opened" (expanded) by the reconstruction.

    node ← lookup.get(region)

    if node is None:
        // Gap: no PEWEI node covers this region at or above max_layer.
        // Output the accumulated ancestral background.
        spans.append(region, accumulated)
        return

    if node is terminal:
        spans.append(region, accumulated + node.intensity)
        return

    if node is transition:
        mid ← midpoint(region)
        left_region ← [region.l, mid)
        right_region ← [mid, region.r)

        // This node's baseline joins the ancestral background,
        // pro-rated equally to each half.
        total_bg ← accumulated + node.baseline
        half_bg ← total_bg / 2

        left_visible  ← lookup.contains(left_region)
        right_visible ← lookup.contains(right_region)

        if left_visible and right_visible:
            // Both children available — descend into each.
            descend(lookup, left_region,  half_bg, spans)
            descend(lookup, right_region, half_bg, spans)

        else if left_visible and not right_visible:
            // Left child known, right child truncated.
            // Right gets the remainder: total − baseline − left.total,
            // plus its share of ancestral background.
            descend(lookup, left_region, half_bg, spans)
            right_energy ← node.total − node.baseline − lookup.total(left_region)
            spans.append(right_region, half_bg + right_energy)

        else if right_visible and not left_visible:
            // Symmetric to above.
            left_energy ← node.total − node.baseline − lookup.total(right_region)
            spans.append(left_region, half_bg + left_energy)
            descend(lookup, right_region, half_bg, spans)

        else:
            // Fully truncated: no children visible at max_layer.
            // Use the transition's total, which includes everything below.
            spans.append(region, accumulated + node.total)
```

Cost: $O(V + K)$ where $V$ is the number of visible PEWEI nodes (those at layers $\leq$ max_layer) and $K$ is the number of output spans. Each PEWEI node is visited at most once via the lookup. Gap regions are visited at most once each.

### 10.4 Validity Conditions

**When is reconstruction valid?** The algorithm starts from `root_region` and descends. For total energy to be exactly conserved, the lookup must contain a PEWEI node at `root_region`. This node's total ($g.\text{sum}$ for the G-root) equals the entire domain's energy.

| Condition                                                                       | Consequence                                                                                                           |
| ------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------- |
| G-root's entry visible at max_layer                                             | Full energy conservation. Every coordinate covered.                                                                   |
| G-root's entry NOT visible, but descendant entries collectively tile the domain | **Not handled.** The descent hits the gap case at the root, outputs 0 for the entire domain, ignoring deeper entries. |
| G-root's entry NOT visible, partial coverage                                    | Energy underestimated. Missing regions output 0.                                                                      |

In typical trees, the G-root's entry sits at V-depth 1 (a child of the structural V-root), so max_layer ≥ 1 suffices for full conservation.

> **Design note.** The top-down algorithm cannot exploit a grandchild entry when its parent entry is missing. This is analogous to progressive coding: skipping a parent coefficient makes its children's context invalid. The PEWEI's progressive structure means you cannot skip the middle of the ancestry chain. This is a consequence of the additive property — the parent's baseline is part of the children's reconstruction — not a deficiency.

### 10.5 Properties at Valid Truncation Points

At each valid truncation point (G-root's entry visible):

- The total energy across the domain is exact (G-I1 conservation).
- The spatial distribution reflects all structure confirmed through the visible layers.
- Unresolved regions carry their parent's total, pro-rated uniformly — the best estimate available at the truncated resolution.
- Adding the next layer refines the estimate by splitting coarse spans into sub-regions with confirmed structure, while preserving ancestral baselines as background.
- Spatial distribution **within** a region is approximate: baselines are pro-rated uniformly, regardless of whether the original observations were spatially uniform within the region. For semi-internal nodes whose baseline includes absorbed and directed energy (§2 note), this pro-rating is a model assumption, not an exact decomposition.

---

## Chapter 11. Denoising From the PEWEI

Each phase transition node provides a potential denoising context for its descendants. This section is a **preliminary sketch** — the specific shrinkage function, threshold derivation, and cascade semantics require further development.

> _Stub sketch._ `apply_shrinkage(intensity, baseline)` applies a
> soft-threshold shrinkage estimator. For example, standard wavelet
> shrinkage:
>
> $$\text{shrink}(I, B) = \text{sign}(I - B) \cdot \max(|I - B| - \lambda,\, 0) + B$$
>
> where $\lambda$ is a threshold derived from the baseline $B$ **under
> a specific noise model.** The derivation of $\lambda$ from $B$ is
> the central open question — standard choices (universal threshold
> $\lambda = \sigma\sqrt{2 \log n}$, BayesShrink $\lambda = \sigma^2/\sigma_{\text{signal}}$)
> require a noise variance estimate, which the PEWEI does not directly
> supply. The baseline $B$ provides the mean pre-refinement energy; converting
> this to a variance requires a noise model (e.g., Poisson: $\sigma^2 = B$;
> Gaussian: $\sigma^2$ must be estimated separately).

```
function denoise(pewei, aggressive_threshold):
    for each terminal node T in pewei:
        ancestors ← chain of phase transition ancestors of T

        for F in ancestors (coarsest to finest):
            local_ratio ← T.intensity / F.baseline

            if local_ratio < aggressive_threshold:
                T.intensity ← apply_shrinkage(T.intensity, F.baseline)

    // Open question: should shrinkage at scale k feed into
    // shrinkage at scale k+1 (cascade filter), or should each
    // scale apply independently to the original intensity?
    // Cascade: order-dependent, progressive smoothing.
    // Independent: order-independent, each scale decides alone.
    // The choice affects denoising quality for nested transitions.
```

The key structural advantage: the baselines are spatially and scale-adaptive without a separate estimation pass. The key structural limitation: they are not statistically independent of the signal they calibrate (§6.3).

---

## Chapter 12. Properties

### 12.1 Total Energy Conservation

At every valid truncation depth (§10.4 — the G-root's entry is visible), the total energy across the domain is exactly preserved by the reconstruction algorithm. No observation is discarded by the progressive structure. Coarsening loses distributional detail but preserves magnitude.

Conservation is a property of the **reconstruction algorithm** (§10.3), not of naive summation over visible PEWEI nodes. The algorithm's careful distinction between baselines and totals is what maintains the invariant.

### 12.2 Scale-Adaptive Pre-Refinement Baselines

The phase transition nodes provide spatially-varying, scale-varying pre-refinement energy datums without a separate estimation step. Each datum is a direct measurement. Under appropriate noise models (§6.2), these datums serve as self-calibrating noise floors. The interpretation requires a noise model; the datums themselves are unconditional.

### 12.3 Near-Optimal Significance Ordering

The V-Tree depth bound guarantees the ordering is within factor $1.44$ of Shannon entropy. The first $k$ layers of the PEWEI approximately capture the highest-energy structures for $k$ levels of description. "Approximately" reflects the V-Tree's competitive mechanism: entries at the same V-depth may have different importances, and no single-entry exact significance ordering is guaranteed. The approximation quality is governed by the proven $1/\log_2\varphi$ overhead — tight (§IDEA M-18.3).

### 12.4 Adaptive Basis

The decomposition adapts to the signal. Regions of complexity have deep decomposition. Regions of uniformity have coarse representation. The sparsity pattern was determined by competitive observation, not by a fixed transform.

### 12.5 Dynamic Origin, Static Output

The PEWEI is a static snapshot of a dynamic process. The same G-V Graph extracted at different moments yields different PEWEIs — each reflecting the graph's state of knowledge at the time of extraction. Under sustained observation of a region without net contraction, more observations produce more detail, higher baseline ratios, and deeper phase transitions. Under attenuation or eviction, detail is actively removed and the PEWEI contracts. Under amplification, existing detail is reinforced — baseline ratios grow and phase transitions deepen without new observations. Under annihilation, detail is erased — the PEWEI collapses toward a single layer until re-accumulation. The image develops under the balance of observation and temporal scaling, like a photograph under varying exposure.

### 12.6 Completeness

The PEWEI contains:

| Component                          | Where in the PEWEI                           | Standard pipeline equivalent                        |
| ---------------------------------- | -------------------------------------------- | --------------------------------------------------- |
| Multi-resolution signal estimate   | G-Tree sums on all nodes                     | Wavelet coefficients                                |
| Approximate significance ordering  | V-Tree layer assignment                      | SPIHT partitioning output (approximately)           |
| Pre-refinement baselines per scale | Frozen baselines ($g.\text{own}$)            | Separate noise estimation pass (with caveats, §6.3) |
| Truncation quality                 | Near-optimal layer ordering ($1.44\times$)   | Rate-distortion optimisation (approximately)        |
| Sparsity structure                 | Terminal vs. phase transition classification | Zerotree / significance maps (conceptually)         |

All five components are read from existing fields. The extraction cost is one tree traversal. The qualifiers in the right column reflect the structural differences developed in §§5, 6, and 8 — the PEWEI's mechanisms are richer and more adaptive than their standard counterparts, but the correspondence is structural and approximate rather than algebraic and exact.

---

## Chapter 13. The Name

**Progressive.** Truncation at any valid depth yields a reconstruction with exact total energy. Each layer adds approximately the next-most-significant detail. The ordering has information-theoretic guarantees (within $1.44\times$ of Shannon entropy).

**Entropic.** The competitive ordering satisfies the $\varphi$-bounded depth inequality. V-Tree depth is the cost of probabilistic routing. The structure achieves near-optimal entropy coding maintained dynamically through purely local operations.

**Wavelet.** The G-Tree is an adaptive multi-resolution decomposition. Pre-refinement values serve as scaling-like coefficients. Children's sum asymmetries serve as detail-like coefficients. Pre-refinement baselines cascade approximately across scales as quantisation contexts. The parallel is structural — richer than a standard wavelet basis (§5) but recognisably in the same family.

**Exposure.** Every value is accumulated measurement. The frozen baselines record the energy level at which the competitive mechanism confirmed sub-scale structure — the threshold at which the region graduated from "undifferentiated" to "resolved." The image develops under progressive measurement as a photograph develops under light.

**Image.** A static reading of a living structure. A snapshot. A record of the tree's state of knowledge at one moment, encoding both what was measured and the history of how the measurement process resolved structure.

# Factored Adaptive Joint Indexing via Tensor-Product Z-Curve Composition

## Setup

Let $\mathcal{G}_x = (G_x, V_x)$ and $\mathcal{G}_y = (G_y, V_y)$ be independent G-V Graphs over $[0, 2^{N_x})$ and $[0, 2^{N_y})$. Let $P_x = \{P_x^0, \ldots, P_x^{p_x - 1}\}$, $P_y = \{P_y^0, \ldots, P_y^{p_y - 1}\}$ be their ordered plateau sets. Construct $\mathcal{G}_z = (G_z, V_z)$ over $[0, 2^{N_z})$ with $N_z = 4\lceil\log_2 M\rceil$ where $M$ is a conservative upper bound on $\max(p_x, p_y)$.

## Double-Referenced Ordinals

For element at position $k$ in a vocabulary of size $p$, define the **rank–co-rank pair**:

$$\hat{k} = (k,\; p - 1 - k)$$

The pair lies on the anti-diagonal $k + (p - 1 - k) = p - 1$. When the vocabulary mutates — insertion or deletion at any position $m$ — every surviving element shifts in at least one component:

| Mutation                    | Positions $< m$              | Positions $> m$              |
| --------------------------- | ---------------------------- | ---------------------------- |
| Insert at $m$ ($p \to p+1$) | Rank unchanged, co-rank $+1$ | Rank $+1$, co-rank unchanged |
| Delete at $m$ ($p \to p-1$) | Rank unchanged, co-rank $-1$ | Rank $-1$, co-rank unchanged |

**Total cascade property.** Every vocabulary change of size 1 shifts every surviving element's double ordinal by $\pm 1$ in exactly one component.

## Joint Index Map

Define the double-ordinal Z-address by 4-fold bit interleaving:

$$\zeta(\hat{k}, \hat{j}) = \bigoplus_{b\,\geq\, 0} \Bigl(k_b \cdot 2^{4b+3} \;+\; (p_x{-}1{-}k)_b \cdot 2^{4b+2} \;+\; j_b \cdot 2^{4b+1} \;+\; (p_y{-}1{-}j)_b \cdot 2^{4b}\Bigr)$$

where subscript $b$ denotes the $b$-th bit. The 4-fold interleave preserves **Z-locality under unit perturbation**: a $\pm 1$ change in any component flips a low-order bit in that component's lane, producing a bounded displacement in Z-space. Frozen entries at pre-mutation addresses and fresh entries at post-mutation addresses occupy proximate dyadic subtrees of $G_z$ and hence proximate $V_z$-tournament neighbourhoods.

## Observation Protocol

On arrival of $(x, y, \Delta)$:

$$\mathcal{G}_x.\text{observe}(x, \Delta) \;\longrightarrow\; k = \text{ord}_x(x),\quad p_x = |P_x|$$
$$\mathcal{G}_y.\text{observe}(y, \Delta) \;\longrightarrow\; j = \text{ord}_y(y),\quad p_y = |P_y|$$
$$\mathcal{G}_z.\text{observe}\!\bigl(\zeta\bigl((k,\, p_x{-}1{-}k),\; (j,\, p_y{-}1{-}j)\bigr),\; \Delta\bigr)$$

where $\text{ord}_x(x)$ returns the ordinal rank of the plateau containing $x$. Each tree is a standard unmodified G-V Graph. Only the routing is novel.

## Marginal Vocabulary Change as Total Benchmark Transfer

When $\mathcal{G}_x$ refines or contracts a plateau, $p_x$ changes by $\pm 1$. By the total cascade property, every double ordinal $\hat{k}$ shifts in one component. Every Z-address $\zeta(\hat{k}, \hat{j})$ moves. Entries in $\mathcal{G}_z$ at pre-mutation addresses cease receiving observations; their importance freezes as competitive benchmarks. Fresh entries accumulate at corrected addresses starting from $\nu$.

This is the catalytic split mechanism (§IDEA M-10.2) operating across trees. The marginals' structural decisions—which regions to refine, which to contract—become the joint tree's admission criteria through ordinal instability alone. No explicit inter-tree communication exists; the competitive mechanism is the transfer channel.

**Semantic coherence.** Under single ordinals, positions below the mutation point are unaffected—creating a joint tree containing entries indexed against mixed vocabulary versions. The rank–co-rank encoding eliminates this: every entry shifts, every entry freezes, and all live post-mutation entries reference the same vocabulary state. The $V_z$-tournament compares commensurable quantities.

**Self-prioritised healing.** Observations concentrate where marginals refined. Post-mutation observations route to corrected Z-addresses. The most actively observed pairings accumulate past their frozen benchmarks first. Cold pairings—already $V_z$-deep—self-evict through the standard mechanism. Healing cost is proportional to actual joint significance, not to cascade scope.

## Properties

| Property                         | Guarantee                                                                                        |
| -------------------------------- | ------------------------------------------------------------------------------------------------ | ------------- | -------------------------------------------------------------------------------- |
| Marginal queries                 | $O(N_x)$, $O(N_y)$ — direct single-tree operations                                               |
| Joint sampling                   | $O(1.44\, H_z + 1)$ expected, $H_z$ = entropy over materialised Z-cells                          |
| Routing depth in $\mathcal{G}_z$ | $O(\log \max(p_x, p_y))$ — logarithmic in vocabulary size                                        |
| Benchmark locality               | $\pm 1$ displacement per component $\Rightarrow$ Z-adjacent stale/fresh pairs                    |
| Vocabulary coherence             | Total cascade $\Rightarrow$ all live entries reference current vocabulary                        |
| Energy conservation              | Each tree independently maintains $\sum g.\text{sum}$ through all transitions                    |
| Sparsity                         | $                                                                                                | \mkern1mu G_z | \leq \min(n\_{\text{obs}},\; p_x \cdot p_y)$; only observed pairings materialise |
| Benchmark compounding            | Repeated vocabulary changes compound admission thresholds at rate $\Theta(\alpha^k)$             |
| $d$-dimensional extension        | $d$ marginals, one Z-tree over $2d$-fold rank–co-rank interleave, $N_z = 2d\lceil\log_2 M\rceil$ |

## System

$$\boxed{(x,y,\Delta)} \;\xrightarrow{\;\text{marginals}\;}\; \mathcal{G}_x \otimes \mathcal{G}_y \;\xrightarrow{\;\zeta(\text{rank–co-rank})\;}\; \mathcal{G}_z$$

Three unmodified 1-D G-V Graphs. The marginals learn per-axis feature vocabularies via adaptive plateaus. The joint tree discovers which feature pairings carry significant interaction through competitive ranking over the Z-linearised tensor product of double ordinals. Every marginal vocabulary mutation—refinement or contraction, on either axis—triggers a total cascade: all joint entries freeze as benchmarks, all pairings re-validate against the current vocabulary through native competitive dynamics. The rank–co-rank encoding guarantees totality of the cascade (semantic coherence), minimality of per-entry displacement ($\pm 1$ in one component), and locality of the resulting benchmark relationships (Z-adjacency). The instability of the double-ordinal index is the mechanism by which marginal learning governs joint learning.

# Multi-View Factored 2D Analysis via Hilbert–Z Composition

## Setup

Let $\Omega = [0, 2^{N_x}) \times [0, 2^{N_y})$ be a 2D observation domain. All G-V graphs are unmodified 1D instances per the base specification. Write $P_\mathcal{T}$ for the ordered plateau set of tree $\mathcal{T}$, $|\cdot|$ for its cardinality, and $\text{ord}_\mathcal{T}(x)$ for the ordinal of the plateau containing $x$. For vocabulary size $p$, define the double ordinal $\hat{k} = (k,\; p - 1 - k)$ and the 4-fold Z-interleave $\zeta(\hat{k}, \hat{j})$ as in the base factored construction.

## Architecture

Five 1D G-V graphs in two layers:

**Layer 1 — Three independent views of $\Omega$:**

| Tree            | Domain         | Input                      | Captures                       |
| --------------- | -------------- | -------------------------- | ------------------------------ |
| $\mathcal{G}_x$ | $[0, 2^{N_x})$ | $x$-coordinate             | Marginal $x$-structure         |
| $\mathcal{G}_y$ | $[0, 2^{N_y})$ | $y$-coordinate             | Marginal $y$-structure         |
| $\mathcal{G}_H$ | $[0, 2^{N_H})$ | $\eta(x,y)$, Hilbert curve | 2D spatial proximity structure |

**Layer 2 — Two interaction-discovery trees over Layer 1 vocabularies:**

| Tree              | Domain                                                 | Input                                                                    | Captures                                      |
| ----------------- | ------------------------------------------------------ | ------------------------------------------------------------------------ | --------------------------------------------- |
| $\mathcal{G}_z$   | $[0, 2^{N_z})$, $N_z = 4\lceil\log_2 M_{xy}\rceil$     | $\zeta\bigl(\widehat{\text{ord}_x(x)},\;\widehat{\text{ord}_y(y)}\bigr)$ | Significant axis-aligned feature interactions |
| $\mathcal{G}_\mu$ | $[0, 2^{N_\mu})$, $N_\mu = 4\lceil\log_2 M_{zH}\rceil$ | $\zeta\bigl(\widehat{\text{ord}_z(z)},\;\widehat{\text{ord}_H(h)}\bigr)$ | Cross-view correspondences                    |

where $M_{xy} \geq \max(|P_x|, |P_y|)$ and $M_{zH} \geq \max(|P_z|, |P_H|)$ are conservative bounds.

## Observation Protocol

On arrival of $(x, y, \Delta) \in \Omega \times \mathbb{R}_{>0}$:

$$\mathcal{G}_x.\text{observe}(x,\;\Delta) \;\longrightarrow\; k = \text{ord}_x(x)$$
$$\mathcal{G}_y.\text{observe}(y,\;\Delta) \;\longrightarrow\; j = \text{ord}_y(y)$$
$$\mathcal{G}_H.\text{observe}\bigl(\eta(x,y),\;\Delta\bigr) \;\longrightarrow\; h = \text{ord}_H\bigl(\eta(x,y)\bigr)$$
$$\mathcal{G}_z.\text{observe}\!\Bigl(\zeta\bigl(\hat{k},\,\hat{j}\bigr),\;\Delta\Bigr) \;\longrightarrow\; z = \text{ord}_z\!\Bigl(\zeta\bigl(\hat{k},\,\hat{j}\bigr)\Bigr)$$
$$\mathcal{G}_\mu.\text{observe}\!\Bigl(\zeta\bigl(\hat{z},\,\hat{h}\bigr),\;\Delta\Bigr)$$

## Complementarity Theorem (Informal)

The two Layer 1 spatial views have complementary representation costs for geometric primitives in $\Omega$:

| Structure in $\Omega$                | $\mathcal{G}_z$ plateau cells | $\mathcal{G}_H$ plateau clusters        |
| ------------------------------------ | ----------------------------- | --------------------------------------- | --- | ------------------------------- |
| Axis-aligned rectangle               | $O(1)$                        | $O(\text{perimeter}/\text{resolution})$ |
| Rank-1 separable $f(x)g(y)$          | $O(\text{rank})$              | $O(\text{support area})$                |
| Line at angle $\alpha \neq 0, \pi/2$ | $O(p \cdot                    | \sin 2\alpha                            | )$  | $O(1)$ up to quadrant crossings |
| Compact convex region                | $O(p_x + p_y)$                | $O(1)$                                  |

$\mathcal{G}_\mu$ discovers which $\mathcal{G}_z$-cells and $\mathcal{G}_H$-clusters co-occur under observation, providing: (i) **connectivity** — multiple $\mathcal{G}_z$-cells mapping to the same $\mathcal{G}_H$-cluster are fragments of one spatial feature; (ii) **decomposition** — each $\mathcal{G}_H$-cluster is annotated with its axis-aligned feature interactions; (iii) **consistency** — significance agreement across views increases confidence.

## Properties

All properties of the base factored construction (vocabulary coherence via total cascade, benchmark locality via Z-adjacency, energy conservation, sparsity, self-prioritised healing) hold at both composition layers. Additionally:

| Property                     | Guarantee                                              |
| ---------------------------- | ------------------------------------------------------ | ---------------- | ---------------------------- | --- | ----- | --- | --- |
| Marginal queries             | $O(N_x)$, $O(N_y)$ — direct single-tree access         |
| 2D spatial sampling          | $O(1.44\, H_H + 1)$ expected via $\mathcal{G}_H$       |
| Feature-interaction sampling | $O(1.44\, H_z + 1)$ expected via $\mathcal{G}_z$       |
| Cross-view sampling          | $O(1.44\, H_\mu + 1)$ expected via $\mathcal{G}_\mu$   |
| Cascade depth                | $\leq 2$ layers: marginal $\to$ interaction $\to$ meta |
| Materialised joint cells     | $                                                      | \mathcal{G}\_\mu | \leq \min(n\_{\text{obs}},\; | P_z | \cdot | P_H | )$  |
| Per-observation cost         | 5 independent G-V observe calls                        |

## System Diagram

$$\boxed{(x,y,\Delta)} \;\xrightarrow{\;\text{marginals + Hilbert}\;}\; \underbrace{\mathcal{G}_x \otimes \mathcal{G}_y}_{\text{axis features}} \;\Big\|\; \underbrace{\mathcal{G}_H}_{\text{spatial features}} \;\xrightarrow{\;\zeta(\text{r-cr})\;}\; \underbrace{\mathcal{G}_z}_{\text{interactions}} \;\Big\|\; \mathcal{G}_H \;\xrightarrow{\;\zeta(\text{r-cr})\;}\; \underbrace{\mathcal{G}_\mu}_{\text{cross-view}}$$

Five unmodified 1D G-V graphs. Two complementary spatial decompositions composed via the generic factored pattern. The meta-tree $\mathcal{G}_\mu$ bridges feature-interaction space and physical space through competitive plateau-pair indexing, discovering which axis-aligned interactions correspond to which contiguous spatial structures — and which don't.
