# ADR-M-026: Point Query and Plateau Semantics

**Status:** Decided  
**Date:** 2026-02-27  
**Phase:** 5  
**Relates to:** [ADR-M-008](008-span-type.md) (view types),
[ADR-M-009](009-trait-decomposition.md) (`SpatialRead`),
[ADR-M-016](016-semi-internal-state.md) (semi-internal nodes),
[ADR-M-025](025-public-api-surface.md) (public API surface),
[ADR-M-032](032-three-surface-model.md) (three-surface model)  
**Spec:** §IDEA M-5.6 (plateaus), §IDEA M-5.2 (routing), §IDEA M-5.5 (queries)  
**API:** [§API M-5.2](../docs/api.md#point-query) (point query),
[§API M-5.2](../docs/api.md#plateau-projection) (plateau projection),
[§API M-4.3](../docs/api.md#43-contour-map) (contour map types)  
**Surface:** 1 (Prints)

## Context

§API M-5.2 specifies a point query:

```rust
fn get(&self, coord: C) -> Cell<C, V>;
```

This requires answering: _what does "the cell at coordinate X" mean
in a structure with semi-internal nodes?_

Answering this question properly requires establishing the right
conceptual model for the G-Tree's output. That model is governed by
five invariants — **P-I1** through **P-I5** — which together
determine how the G-Tree's spatial structure partitions into
plateaus, how the basis is minimal, how energy is attributed,
how the ordered map is keyed, and how thatch depth is bounded.

### The G-Tree as a shape

Imagine the G-Tree drawn as a filled shape: the domain `[0, 2^N)`
on the x-axis, depth on the y-axis (increasing downward). The tree
fills in wherever it has refined. Deeper regions are where the tree
invested more resolution; shallower regions are where it didn't.

Now trace the **bottom contour** — the edge along the underside of
the shape, from left to right. This contour is a step function: a
series of horizontal runs (plateaus) at varying depths, connected
by vertical drops where the depth changes.

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

A **fully balanced** tree is a solid filled rectangle. The bottom
edge is a single flat line at maximum depth. **One plateau.** The
input was uniform — the tree found no reason to refine unevenly.
It doesn't matter that there are `2^d` individual terminal arena
nodes; they're all at the same depth, so they merge into one
continuous surface. The shape has no features.

A **degenerate** tree (every node semi-internal, only left children)
is a staircase. Each level introduces a new depth, a new plateau.
`d` plateaus for depth `d`.

### Plateaus, not terminals, are the semantic unit

A plateau is a maximal contiguous run of the contour at a single
depth. It is the G-Tree's answer to: _"over this region, how hard
did I look, and what did I find?"_

Within a plateau, all terminals are at the same depth, same
interval width. Whatever intensity variation exists between them is
_below the tree's discrimination threshold_ — if it were
significant, the tree would have split unevenly and introduced a
depth change, breaking the plateau in two. The tree has already
rendered its verdict: "this region is uniform at this resolution."
Sub-plateau variation across individual terminals is noise the tree
chose not to model.

The **boundaries between plateaus** are where the tree found
non-uniformity worth resolving. They are the tree's learned
_feature edges_ in the observed distribution. The number of
plateaus is the tree's estimate of the **spatial complexity** of
the input.

### The five plateau invariants

The spec (§IDEA M-5.6.4) defines **five invariants** that
govern the relationship between plateaus, their basis elements,
their ordered map keys, and the depth of thatch stacking. Every
design choice in this ADR flows from preserving these five
invariants under all G-Tree mutations.

Each plateau maintains a **basis**: the minimal set of G-nodes
whose `sum` values capture the plateau's energy. A G-node `R` is
a basis element of plateau `P` if:

1. `R`'s contour contribution falls entirely within `P` — either
   `R`'s subtree produces only contour cells belonging to `P`, or
   `R` is a semi-internal whose uncovered half belongs to `P`.
2. No ancestor of `R` also satisfies condition 1 for `P`.
3. `R` belongs to exactly one plateau's basis.

The plateau's total energy is:

```
P.sum = Σ R.sum  for R in basis(P)
```

Each basis element has a **tile** (contour contribution) and a
**span** (full G-node interval):

| Basis element type                              | `tile(R)`                 | `span(R)`                |
| ----------------------------------------------- | ------------------------- | ------------------------ |
| Terminal `[l, r)`                               | `[l, r)`                  | `[l, r)`                 |
| Balanced internal `[l, r)`                      | `[l, r)`                  | `[l, r)`                 |
| Semi-internal `[l, r)`, child on left `[l, m)`  | `[m, r)` — uncovered half | `[l, r)` — full interval |
| Semi-internal `[l, r)`, child on right `[m, r)` | `[l, m)` — uncovered half | `[l, r)` — full interval |

The **edge** of `R` is `edge(R) = min(tile(R))` — the leftmost
coordinate of the tile. The **run** of a plateau is the union of
its basis elements' spans: `[min R.l, max R.r)`, which may exceed
the tile range at semi-internal boundaries (thatching).

The five governing invariants are:

> **P-I1 (Deterministic Tiling).** Let `σ` be the contour depth
> function and `0 = a_0 < a_1 < ... < a_{P-1}` its step
> coordinates (where depth changes or the domain starts). Then:
>
> (i) `keys(plateaus) = {a_0, ..., a_{P-1}}`
>
> (ii) For each plateau `P_i`:
> `⋃ { tile(R) : R ∈ basis(P_i) } = [a_i, a_{i+1})`
>
> (iii) `P_i.run ⊇ [a_i, a_{i+1})`, with equality iff
> `basis(P_i)` contains no semi-internal element.
>
> The ordered map key for plateau `P` is the **basis edge**:
>
> ```
> key(P) = min { edge(R) : R ∈ basis(P) } = a_i
> ```
>
> Basis edge keys tile the domain `[0, 2^N)` left-to-right with
> no gaps and no duplicates. Consecutive keys partition the
> domain into contour runs.

> **P-I2 (Minimal Deterministic Basis).** Each basis element is
> maximally consolidated:
>
> ```
> ∀ R ∈ basis(P):
>   (1) R's contour ⊆ P
>   (2) ∄ A ⊃ R s.t. A's contour ⊆ P
>   (3) R ∈ basis(P) only
> ```
>
> The basis is the **minimal** set: no element can be replaced by
> an ancestor that also satisfies condition (1). A fully balanced
> tree of depth `d` has a single basis element — the root — not
> `2^d` terminals or `2^(d-1)` one-level-balanced internals.

> **P-I3 (Tile Disjointness).** The tiles of basis elements from
> different plateaus are pairwise disjoint:
>
> ```
> ∀ P ≠ P',
>   ∀ R ∈ basis(P),
>   ∀ R' ∈ basis(P'):
>     tile(R) ∩ tile(R') = ∅
> ```
>
> This is a corollary of P-I1(ii) — distinct plateaus own disjoint
> tile ranges, and each plateau's tiles partition its range exactly.
> It is stated separately as a checkable cross-invariant.
>
> Note that tiles, not spans, are disjoint. Semi-internal basis
> elements' spans extend into the child plateau's territory — that
> overlap is thatching, governed by P-I4.

> **P-I4 (Thatch — one-hop).** Every semi-internal basis element
> thatches the plateau that its direct G-child belongs to:
>
> ```
> g ∈ basis(P), g semi-internal, child(g) ∈ basis(P')
>     ⟹ P' ≠ P ∧ P' is unique
> ```
>
> The thatch relationship is one-hop: `g` thatches the plateau of
> its _immediate_ child node, not all transitively contained
> plateaus. Transitive spatial overlap from nested semi-internals
> is a consequence of stacked one-hop thatches. The stacking depth
> is bounded by P-I5.

> **P-I5 (Thatch Depth).** The number of plateaus whose basis
> elements' spans include any coordinate `x` is bounded:
>
> ```
> thatch_depth(x) ≤ d_geo(x) + 1
> ```
>
> where `d_geo(x)` is the G-Tree depth of the contour cell at `x`.
> The owning plateau always contributes 1. Each semi-internal
> ancestor along the root-to-contour path can contribute at most
> one additional thatch layer, and there are at most `d_geo(x)`
> such ancestors.

Together, the five invariants provide the structural, keying, and
bounding guarantees:

| Property                                   | Governed by                        |
| ------------------------------------------ | ---------------------------------- |
| **Ordered map keys**                       | P-I1 (deterministic tiling)        |
| **Basis minimality**                       | P-I2 (minimal deterministic basis) |
| **Tile disjointness** across plateaus      | P-I3 (tile disjointness)           |
| **Semi-internal** (partial) basis elements | P-I4 (one-hop thatch)              |
| **Thatch stacking**                        | P-I5 (depth + 1 bound)             |

### Evolution of P-I3: from coverage to tiles

An earlier P-I3 formulation ("Basis Completeness") defined
disjointness over a node's full spatial coverage (span), requiring
exclusion of semi-internal elements — whose spans inherently extend
into the thatched child's territory. Consider:

```
[0,8) semi-internal (left child [0,4) survives)
  └─ [0,4) semi-internal (right child [2,4) survives)
       └─ [2,4) terminal

Contour:  [0,2) d1    [2,4) d2    [4,8) d0
          P_A         P_B         P_C
```

`basis(P_B) = {[2,4)}`, `basis(P_C) = {[0,8)}`.
`span([2,4)) ∩ span([0,8)) = [2,4) ≠ ∅` — a span overlap caused
by thatching, not a structural error.

The tile-based formulation (§IDEA M-5.6.2) resolves this cleanly:
`tile([2,4)) = [2,4)`, `tile([0,8)) = [4,8)` (uncovered half).
`tile([2,4)) ∩ tile([0,8)) = ∅`. No exclusion needed. All basis
elements — including semi-internals — participate in the
disjointness check because tiles represent only the contour
contribution, not the thatched extent. Span overlap is thatching,
governed by P-I4.

### Why P-I4 uses one-hop instead of transitive containment

The original P-I4 formulation used `∃!` (exactly one) over
transitive spatial containment:

```
g ∈ basis(P), g semi-internal
    ⟹ ∃! P' ≠ P : span(g) ⊃ span(basis(P'))
```

This breaks with nested semi-internals:

```
[0,8) semi-int (left child only)
  └─ [0,4) semi-int (right child only)
       └─ [2,4) terminal

Contour:  [0,2) d1    [2,4) d2    [4,8) d0
          P_A         P_B         P_C
```

`[0,8) ∈ basis(P_C)`. Its span `[0,8)` spatially contains
both `basis(P_A) = {[0,4)}` and `basis(P_B) = {[2,4)}`. That's
two plateaus — violating `∃!`.

The fix: define thatch via the direct G-child, which is always
unique. `[0,8)`'s child is `[0,4)`. `[0,4) ∈ basis(P_A)`. So
`[0,8)` thatches P_A — one plateau. `[0,4)`'s child is `[2,4)`.
`[2,4) ∈ basis(P_B)`. So `[0,4)` thatches P_B — one plateau.
Transitive spatial overlap is a consequence of stacked one-hop
thatches.

### Basis element classification

Three cases govern which nodes serve as basis elements. Each case
maps to the invariants:

**Terminals** are basis elements of their plateau, unless a higher
balanced ancestor subsumes them. Their tiles are pairwise disjoint
from other plateaus' tiles — P-I3 applies.
They never produce thatch.

**Semi-internal nodes** are always basis elements. Their uncovered
half (tile) belongs to exactly one plateau, and their `sum` — which
includes the child subtree's energy — becomes part of that
plateau's total. These are **partial-basis** elements: their
spans overlap their direct child's plateau — P-I4 applies.
They are the sole source of thatching.

**Balanced internal nodes** are basis elements when their entire
subtree falls within a single plateau. One basis element replaces
potentially many terminals. When a balanced internal node's
children span different plateaus, it cannot be a basis element of
either — its tile would span both, violating P-I3.

**Example — fully balanced tree:**

If the entire tree is fully balanced, there is one plateau. Its
sole basis element is the root node of the entire G-Tree. One
BTreeMap entry, one basis element, sum = `root.sum`. The `2^d`
terminals don't appear individually — the balanced root subsumes
them all. All five invariants are vacuously satisfied (one
plateau, no semi-internals, no thatch, basis edge = 0).

**Example — semi-internal `[0, 8)` with left child `[0, 4)`:**

```
G-Tree:  [0, 8) semi-internal, own=5
           └─ [0, 4) terminal, own=3

BTreeMap:
  BasisEdge(0): P1 { start: 0, end: 4,  depth: 1, sum: 3,
                     basis: [[0,4)] }
  BasisEdge(4): P2 { start: 0, end: 8,  depth: 0, sum: 8,
                     basis: [[0,8)] }
```

_(The `basis` annotation shows backing G-nodes for exposition;
see Q1 — the `Plateau` struct externalizes basis bookkeeping to
the internal `PlateauBasis` structure.)_

- P-I1: P1's basis edge = `edge([0,4)) = 0`. P2's basis edge =
  `edge([0,8) semi-int, child on left) = 4`. Keys tile
  `[0, 8)`: `[0, 4)` then `[4, 8)`. ✓
- P-I2: P1's sole basis element is the terminal `[0,4)` — no
  ancestor has contour entirely within P1, so it cannot be
  replaced. P2's sole basis element is `[0,8)` — the root, so
  no ancestor exists; P-I2 is vacuously satisfied. ✓
- P-I3: P1's sole basis element is `[0, 4)`, a terminal with
  `tile([0,4)) = [0,4)`. P2's sole basis element is `[0,8)`,
  semi-internal with `tile([0,8)) = [4,8)`.
  `tile([0,4)) ∩ tile([0,8)) = ∅`. ✓
- P2's basis element is `[0, 8)` — semi-internal. P-I4: child of
  `[0,8)` is `[0,4)`. `[0,4) ∈ basis(P1)`. P1 is unique. ✓
- P-I5: At coord 2: covered by P1 and P2. `thatch_depth(2) = 2`.
  `d_geo(2) = 1`. `2 ≤ 1 + 1 = 2`. ✓ (tight).
  At coord 6: covered by P2 only. `thatch_depth(6) = 1`.
  `d_geo(6) = 0`. `1 ≤ 0 + 1 = 1`. ✓ (tight).

**Example — bilateral thatching:**

```
[0,16) internal
  ├─ [0,8)  semi-int (right child only)  ← partial-basis of P_L
  │    └─ [4,8) terminal
  └─ [8,16) semi-int (left child only)   ← partial-basis of P_R
       └─ [8,12) terminal

Contour:  [0,4) d1    [4,12) d2    [12,16) d1
          P_L         P_M          P_R

BTreeMap:
  BasisEdge(0):  P_L { depth: 1 }   basis: [[0,8)]
  BasisEdge(4):  P_M { depth: 2 }   basis: [[4,8), [8,12)]
  BasisEdge(12): P_R { depth: 1 }   basis: [[8,16)]
```

P_M is thatched from both sides: by P_L's semi-internal `[0,8)`
(P-I4: child `[4,8) ∈ basis(P_M)`) and by P_R's semi-internal
`[8,16)` (P-I4: child `[8,12) ∈ basis(P_M)`). Each thatch is
independent and one-hop.

P_M's own basis elements `[4,8)` and `[8,12)` are terminals —
P-I3 ensures their tiles are disjoint from other plateaus' tiles.

At coord 6: covered by P_L (via `[0,8)`) and P_M (via `[4,8)`).
`thatch_depth = 2`, `d_geo = 2`. `2 ≤ 2 + 1 = 3`. ✓

**Example — transitive thatch stacking:**

```
[0,8) semi-int (left child only)        basis of P_C
  └─ [0,4) semi-int (right child only)  basis of P_A
       └─ [2,4) terminal                basis of P_B

Contour:  [0,2) d1    [2,4) d2    [4,8) d0
          P_A         P_B         P_C

BTreeMap:
  BasisEdge(0): P_A { depth: 1 }  basis: [[0,4)]
  BasisEdge(2): P_B { depth: 2 }  basis: [[2,4)]
  BasisEdge(4): P_C { depth: 0 }  basis: [[0,8)]
```

P-I4 checks:

- `[0,8) ∈ basis(P_C)`, child = `[0,4)`, `[0,4) ∈ basis(P_A)`. ✓
- `[0,4) ∈ basis(P_A)`, child = `[2,4)`, `[2,4) ∈ basis(P_B)`. ✓

Each thatch is one-hop. At coord 3: covered by P_A (via `[0,4)`),
P_B (via `[2,4)`), and P_C (via `[0,8)`). `thatch_depth = 3`,
`d_geo = 2`. `3 ≤ 2 + 1 = 3`. ✓ (tight — P-I5 is achieved).

### Thatching: multi-counting by design

Thatching is the direct consequence of **P-I4**. A semi-internal
basis element's span subsumes its direct child's plateau. That
subsumption is the multi-counting.

```
BTreeMap key    Plateau value
────────────    ──────────────────────────────────────────
BasisEdge(0)    { start: 0, end: 4,  depth: 1, sum: 3 }
BasisEdge(4)    { start: 0, end: 8,  depth: 0, sum: 8 }
                          ↑
                   parent's TRUE range, not [4,8).
                   sum=8 includes child's sum=3.
```

**The multi-counting is correct and by design.** The parent really
did accumulate energy over `[0, 8)` — including the `[0, 4)` half
now covered by the child:

- **Pre-split energy:** observations that arrived at `[0, 8)`
  before the child existed.
- **Post-split routing:** observations in `[4, 8)` that route
  to the parent after the child exists.
- **Absorbed energy:** if a right child was evicted
  (`parent.own += child.sum`), that energy from `[4, 8)` is
  folded into the parent's `own`.

Every layer of thatch is real energy. The BTreeMap's basis edge
keys (P-I1) tile the domain — consecutive keys partition
`[0, 2^N)` with no gaps. But the `start..end` fields inside each
plateau value reflect the backing basis elements' true spatial
footprint, which overlaps at every semi-internal boundary like
overlapping layers of thatch.

**P-I4 guarantees exactly one thatch layer per semi-internal
boundary (one-hop).** Stacking of multiple layers occurs only
through nested semi-internals, where each semi-internal
independently satisfies P-I4. **P-I5 bounds the total stacking
depth** at any coordinate to `d_geo(x) + 1`.

Where there is no thatching (a fully balanced tree, no
partial-basis elements), every plateau's range matches its key
range exactly — thickness 1 everywhere. Thicker thatch only occurs
through semi-internal nesting, which is where the interesting
structure lives.

### The actively-maintained BTreeMap

The BTreeMap is not built on demand. It is a **live mirror** of
the G-Tree's contour, stored as a field of `GvGraph` and updated
incrementally as the G-Tree mutates.

```rust
pub struct GvGraph<C: Coordinate, V: Accumulator, const N: u32> {
    // ... existing fields ...
    gnodes: Arena<GNode<C, V>>,
    vnodes: Arena<VNode<V>>,
    g_root: GNodeId,
    // ...

    /// Live plateau mirror of the G-Tree contour.
    /// Keyed by BasisEdge — the leftmost coordinate where the
    /// plateau's basis starts producing terminal-depth contour.
    plateaus: BTreeMap<BasisEdge<C>, Plateau<C, V>>,
}
```

`plateaus()` returns `Cow<'_, BTreeMap<BasisEdge<C>, Plateau<C, V>>>`.
With `dynamic-contour-tracking` (default): `Cow::Borrowed` — `O(1)`
borrow of the live mirror. Without: `Cow::Owned` — `O(G)` rebuild.

This inverts the previous "snapshot on demand" model:

|                       | Snapshot model                            | Active mirror (`dynamic-contour-tracking`) |
| --------------------- | ----------------------------------------- | ------------------------------------------ |
| **`plateaus()`**      | `O(n)` DFS, allocate, return `Cow::Owned` | `O(1)` borrow, return `Cow::Borrowed`      |
| **After `observe()`** | Stale until rebuilt                       | Already updated                            |
| **Memory**            | Allocated per call                        | One BTreeMap, always live                  |
| **Cross-thread**      | Owned — send freely                       | `.clone()` for snapshot                    |

The standard BTreeMap query surface is available directly on the
returned reference:

- **Point query:** `plateaus.range(..=BasisEdge(coord)).next_back()`
  — floor-key pattern. `O(log p)` where `p` is the plateau count.
- **Range query:** `plateaus.range(BasisEdge(a)..BasisEdge(b))` —
  all steps in `[a, b)`.
- **Iteration:** `for (_, p) in plateaus` — steps in spatial
  order. Left to right.
- **Length:** `plateaus.len()` — plateau count = structural
  complexity. `O(1)`.

### Incremental maintenance and invariant preservation

Every G-Tree mutation must preserve all five plateau invariants.
The incremental maintenance rules are derived from these invariants.

**`observe(coord, delta)`:**

After `route_to_receiver` and g-sum propagation:

1. Walk the propagation path (receiver → root).
2. For each node on the path: if it is a basis element, its
   `sum` just changed. Recompute its plateau's cached sum:
   `plateau.sum = Σ R.sum for R in basis(P)`.
3. Typically 1–2 basis elements lie on any propagation path.
   Cost: `O(depth)` for propagation + `O(1)` per affected
   plateau.

**Invariant preservation:** `observe` changes no basis assignments,
no plateau boundaries, and no basis edge keys. Only `sum` values
change. All five invariants are structural (about tiling, keying,
and depth relationships — not magnitudes) and are trivially
preserved. ✓

**`split(terminal)`:**

A terminal at depth `d` splits into two children at depth `d+1`.
The terminal was a basis element of some plateau `P`.

- Both children are at the same depth → `P` deepens but stays
  one plateau. The now-internal node's subtree is entirely within
  `P`. It replaces the terminal as `P`'s basis element. `P`'s
  depth updates to `d+1`. BTreeMap: one entry updated, `O(log p)`.
- If the split creates a depth boundary with neighbors (adjacent
  contour cells at a different depth): `P` may split into
  multiple entries. BTreeMap: `O(1)` insertions and removals,
  `O(log p)` each.

**P-I3 preservation:** The splitting terminal's tile was disjoint
from other plateaus' tiles (P-I3). Its replacement (internal node
or individual children) tiles the same region. New children at
the same depth within the same plateau do not create cross-plateau
tile overlap. ✓

**P-I4 preservation:** If the split creates a semi-internal node
(one child evicted before the other), that semi-internal becomes a
partial-basis element of its uncovered-half plateau. Its child
belongs to exactly one other plateau — one-hop ✓.

**P-I1 preservation:** New basis elements have well-defined
`edge()` values. Basis edge keys are recomputed for affected
plateaus. The tiling property is maintained because splits
introduce new contour depth transitions, each marked by a new
basis edge key. ✓

**P-I2 preservation:** The terminal was a minimal basis element
(no ancestor's contour was contained within `P`). After the
split: if both children stay in `P`, the now-balanced internal
node replaces the terminal — its parent's contour extends beyond
`P`, so the internal node is still minimal. If the split creates
new plateaus, each child is a terminal — trivially minimal (no
smaller element exists). ✓

**P-I5 preservation:** A split increases `d_geo` at the split
region by 1. If the parent becomes semi-internal, it adds one
thatch layer. `thatch_depth` increases by at most 1, and so
does the `d_geo + 1` bound. ✓

**`evict(child)`:**

A child is evicted, parent absorbs `child.sum`. The parent may
become terminal or semi-internal with a new contour shape.

- If adjacent plateaus now share the same depth: they merge. The
  combined plateau gets the union of basis elements. BTreeMap:
  `O(1)` removals, `O(log p)` each.

**P-I3 preservation:** Merging plateaus unions their basis
elements. Elements that were individually tile-disjoint from each
other's plateaus are now in the same plateau — no cross-plateau
tile overlap introduced. If a semi-internal becomes terminal (both
children evicted), it loses its partial-basis status and becomes
a regular basis element. Its tile is now contained within its
plateau — P-I3's tile disjointness holds. ✓

**P-I4 preservation:** If a child plateau is destroyed by
eviction, the parent semi-internal that thatched it (per P-I4)
either: (a) acquires a new child plateau (remaining sibling), in
which case the one-hop relationship re-targets, or (b) becomes
terminal (both children gone), removing it from P-I4's scope
entirely. In case (a), the surviving child's plateau is the
unique thatched plateau — one-hop remains satisfied. ✓

**P-I1 preservation:** Eviction may merge adjacent same-depth
plateaus, which removes intermediate basis edge keys. The merged
plateau's basis edge is the minimum of the constituent basis
edges. The tiling property is maintained because merged plateaus
fill the gap left by the removed key. ✓

**P-I2 preservation:** Eviction may merge adjacent same-depth
plateaus. Each surviving basis element was already minimal in
its original plateau. The merged plateau's domain is larger, so
no previously-invalid ancestor becomes valid — ancestors that
spanned beyond the original plateau still span beyond the merged
one. If a semi-internal becomes terminal (both children evicted),
it replaces its old partial-basis entry — as a terminal it is
trivially minimal. ✓

**P-I5 preservation:** Eviction removes a contour node, reducing
`d_geo` at the affected coordinates. If a semi-internal's child
is removed, one thatch layer disappears. Both sides of the bound
decrease together. ✓

**`decay(factor)`:**

Decay reduces all `own` values and recomputes g-sums. Every
basis element's `sum` potentially changes. Recompute all plateau
sums. Cost: `O(n)` for decay + `O(p)` for plateau updates. Since
`p ≤ n`, the `O(p)` is dominated.

**Invariant preservation:** Decay changes magnitudes, not
structure. No basis assignments change. No plateau boundaries
move. No basis edge keys change. All five invariants are trivially
preserved. ✓

**`legacy_promote(semi_internal)`:**

A semi-internal node earns competitive promotion (V-I3 violation
triggers restoration of its missing child). The semi-internal was
a partial-basis element of some plateau `P`. Its missing child is
restored as a new terminal.

1. Remove the semi-internal from its plateau's basis. If this was
   the sole basis element, remove the plateau.
2. The new child becomes a terminal basis element — joins an
   adjacent same-depth plateau or creates a new one.
3. The previously semi-internal node is now internal. If its
   subtree is balanced (both children at the same depth), it
   becomes a balanced internal basis element. Otherwise it is not
   a basis element.
4. The thatch relationship that `P` held over the surviving child's
   plateau dissolves — the one-hop link (P-I4) no longer exists.

**P-I1 preservation:** Restored child creates a new basis edge
key or joins an existing plateau. Tile union covers the child's
range. ✓

**P-I3 preservation:** The restored terminal's tile is the new
child's range — disjoint from other plateaus' tiles. ✓

**P-I4 preservation:** The semi-internal that thatched the
surviving child's plateau becomes internal — exiting P-I4's scope.
If it becomes a balanced internal basis element, it is
non-thatching (P-I3 governs instead). ✓

**P-I5 preservation:** Restoration adds a contour node (increasing
`d_geo` by 1) and removes one thatch layer (semi-internal →
internal). Net effect on thatch depth: zero or decrease. ✓

**Initialization:**

`GvGraph::new()` creates the root node (a terminal covering the
full domain). The BTreeMap starts with one entry:
`BasisEdge(0) → Plateau { start: 0, end: 2^N, depth: 0, sum: 0 }`.
All five invariants are vacuously satisfied (one plateau, no
semi-internal nodes, no thatch, `thatch_depth = 1 ≤ 0 + 1`).

The floor-key pattern on the stored BTreeMap is the
point-query interface:

```rust
let plateaus = graph.plateaus();
let (_, plateau) = plateaus
    .range(..=BasisEdge(coord))
    .next_back()
    .unwrap();
```

## Questions

### Q1: Plateau struct — public surface (DC-026-1)

**Decided.**

```rust
/// Snapshot of one plateau on the G-Tree's bottom contour.
///
/// A small `Copy` view type — consistent with `Cell`, `Node`, and
/// `Span` (ADR-M-008). Basis bookkeeping is maintained internally
/// by `GvGraph` and does not appear on this struct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plateau<C: Coordinate, V: Accumulator> {
    /// The basis edge — leftmost coordinate where this plateau's
    /// basis starts producing terminal-depth contour (P-I1).
    pub basis_edge: BasisEdge<C>,
    /// Thatched range start (min of basis elements' lo).
    pub start: C,
    /// Thatched range end (max of basis elements' hi).
    pub end: C,
    /// Contour depth of this plateau.
    pub depth: u32,
    /// Total energy: `Σ R.sum` over this plateau's basis elements.
    ///
    /// At semi-internal boundaries, this includes child subtree
    /// energy (thatching). See §Thatching above for the
    /// multi-counting semantics.
    pub sum: V,
}
```

| Question               | Decision               | Rationale                                                                                                                                |
| ---------------------- | ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `sum` vs `own`?        | **`sum`**              | It's the sum of the basis elements' `sum` values. Not a single node's `own`.                                                             |
| `basis` on the struct? | **No — externalized.** | Basis tracking is maintenance machinery. It lives in a private `PlateauBasis` structure inside `GvGraph`. This lets `Plateau` be `Copy`. |
| `SmallVec` dependency? | **No.**                | Unnecessary. The crate stays at three dependencies.                                                                                      |
| `Copy`?                | **Yes.**               | ~36 bytes for `<u64, u64>` (extra 8 for `basis_edge`). Consistent with `Cell`, `Node`, `Span`.                                           |
| `basis_edge` field?    | **Yes.**               | The plateau carries its own key (P-I1). Enables reconstruction of the ordered map from a collection of plateaus.                         |

**The `start`/`end` fields encode the thatched extent (P-I4).**
For a plateau whose basis includes a semi-internal element,
`start..end` reflects the semi-internal's full span. For
plateaus with only non-semi-internal basis elements, `start`
and `end` match the basis edge key range exactly (P-I3 tile
disjointness).

**Basis bookkeeping (internal):**

```rust
/// Internal bookkeeping for plateau ↔ basis associations.
/// Not part of the public API.
///
/// Maintains the basis assignments that satisfy P-I1 through P-I5.
/// Updated atomically by split/evict/decay code paths.
pub(crate) struct PlateauBasis<C: Coordinate> {
    /// Forward: basis edge → basis element GNodeIds.
    forward: BTreeMap<BasisEdge<C>, HashSet<GNodeId>>,
    /// Back: basis element → basis edge.
    back: HashMap<GNodeId, BasisEdge<C>>,
}
```

No `C: Ord` required — `BasisEdge<C>` provides `Ord` via
`Coordinate::total_cmp` (see Q7).

### Q2: `get(coord)` — retain or remove? (DC-026-2)

**Decided: Option A — keep `get()` returning `Cell`.**

| Option | API                           | Cost       | Notes                                                                                                |
| ------ | ----------------------------- | ---------- | ---------------------------------------------------------------------------------------------------- |
| A      | Keep `get()` returning `Cell` | `O(depth)` | Uses `route_to_receiver`. Returns the individual terminal. Trimmed half-interval for semi-internals. |
| B      | Remove — use BTreeMap only    | `O(log p)` | Floor-key on stored mirror. Returns the plateau, not the terminal.                                   |

**Rationale:**

`get()` and the BTreeMap floor-key answer different questions at
different abstraction levels:

|                 | `get(coord)`                             | `plateaus().range(..=BasisEdge(coord)).next_back()`     |
| --------------- | ---------------------------------------- | ------------------------------------------------------- |
| **Question**    | Where does this observation route?       | What structural region contains this coordinate?        |
| **Returns**     | `Cell` — individual terminal/receiver    | `&Plateau` — contour summary (possibly many terminals)  |
| **Interval**    | Trimmed: effective half-interval only    | Thatched: basis element's full spatial footprint (P-I4) |
| **Value**       | `own` — direct accumulation at this node | `sum` — thatched total including child energy           |
| **Granularity** | Per-terminal (sub-plateau)               | Per-plateau                                             |

`sample()` already returns `Cell` — removing `get()` would create
an asymmetry. Implementation cost is ~5 lines on top of
`route_to_receiver`.

**Semi-internal trimming contract:** For a semi-internal `[0, 8)`
with left child `[0, 4)`, `get(6)` returns
`Cell { start: 4, end: 8, ... }` — the _effective_ uncovered
half-interval, not the arena node's full `[lo, hi)`.

### Q3: Plateau count and size metrics (DC-026-3)

**Decided: three complementary metrics, each with an obvious home.**

| Metric         | Source             | Cost   | Meaning                                             |
| -------------- | ------------------ | ------ | --------------------------------------------------- |
| Plateau count  | `plateaus().len()` | `O(1)` | Structural complexity — how many depth transitions. |
| G-node count   | `node_count()`     | `O(1)` | Arena resource usage — total live nodes.            |
| Terminal count | `terminal_count()` | `O(1)` | Resolution — how many leaf regions.                 |

The relationship `plateaus ≤ terminals ≤ nodes`
holds always. A tree where plateaus ≈ terminals has maximal
structural variation. A tree where plateaus ≪ terminals is mostly
uniform.

### Q4: `get()` return type (DC-026-4)

**Decided: Option C — infallible `Cell<C, V>` with domain clamping.**

| Option | Behaviour                                       | Notes                                  |
| ------ | ----------------------------------------------- | -------------------------------------- |
| A      | `Option<Cell<C, V>>` — `None` for out-of-domain | `None` is unreachable for valid usage. |
| B      | `Cell<C, V>` — panic for out-of-domain          | Hostile for a library.                 |
| **C**  | **`Cell<C, V>` — clamp to domain**              | Matches `range_sum`'s clamping.        |

```rust
pub fn get(&self, coord: C) -> Cell<C, V>
```

### Q5: Back-pointer mechanism (DC-026-5)

**Decided: Option B — `HashMap<GNodeId, BasisEdge<C>>` inside
`PlateauBasis`.**

How does the `observe` propagation path find which basis elements
are affected?

| Option | Mechanism                       | GNode size         | Lookup               |
| ------ | ------------------------------- | ------------------ | -------------------- |
| A      | Flag on GNode                   | 48 → 64 bytes      | `O(1)` field read    |
| **B**  | **`HashMap` in `PlateauBasis`** | **48 (unchanged)** | **`O(1)` amortized** |
| C      | Walk up and probe BTreeMap      | 48 (unchanged)     | `O(log p)` per step  |

**Rationale:**

The back-pointer lives alongside the forward map in `PlateauBasis`
— both directions in one struct, updated atomically by the same
code paths that maintain P-I1 through P-I5:

```rust
pub(crate) struct PlateauBasis<C: Coordinate> {
    forward: BTreeMap<BasisEdge<C>, HashSet<GNodeId>>,
    back: HashMap<GNodeId, BasisEdge<C>>,
}
```

Option A puts derived-structure bookkeeping on the arena node —
rejected for the same reasons as ADR-M-016. It also grows GNode from
48 to 64 bytes, crossing a cache line boundary on the `observe`
hot path.

### Q6: Relationship to `extract()` / PEWEI (DC-026-6)

**Characterization — no decision required.**

`extract()` and `plateaus()` are orthogonal projections of the
same dual-tree, each destroying the other's ordering:

|                 | `extract()` / PEWEI           | `plateaus()` / BTreeMap          |
| --------------- | ----------------------------- | -------------------------------- |
| **Tree**        | V-Tree (value tournament)     | G-Tree (spatial routing)         |
| **Ordering**    | Significance (V-depth BFS)    | Spatial (left-to-right contour)  |
| **Question**    | What's important?             | What shape did adaptation take?  |
| **Granularity** | Per-node                      | Per-plateau                      |
| **Cost**        | `O(n)`, allocates             | `O(1)` borrow                    |
| **Path**        | Cold (checkpointing, export)  | Hot (per-observation, per-query) |
| **Governs**     | V-Tree invariants (V-I1–V-I7) | Plateau invariants (P-I1–P-I5)   |

Neither subsumes the other.

### Q7: `Ord` requirement on coordinate type (DC-026-7)

**Decided: `BasisEdge<C>` newtype + `Coordinate::total_cmp`.
Float coordinates supported. No `Ord` on `GvGraph`.**

**Discovery:** `BTreeMap<C, _>` requires `C: Ord`. The
`Coordinate` trait requires only `PartialOrd` (to accommodate
`f32`/`f64`). Adding `C: Ord` to `GvGraph` would exclude float
coordinates entirely.

**Solution: `BasisEdge<C>` newtype.**

Add `total_cmp` to the `Coordinate` trait:

```rust
// In Coordinate trait:
/// Total ordering for use as a BTreeMap key.
///
/// Integers: delegates to Ord::cmp (zero overhead).
/// Floats: uses IEEE 754 totalOrder via f{32,64}::total_cmp.
fn total_cmp(&self, other: &Self) -> std::cmp::Ordering;
```

Then define a zero-cost newtype:

```rust
/// The leftmost coordinate where a plateau's basis starts
/// producing terminal-depth contour.
///
/// A zero-cost newtype providing Ord for any Coordinate by
/// delegating to Coordinate::total_cmp. Used as the key type
/// for the plateau BTreeMap and PlateauBasis forward map.
#[derive(Debug, Clone, Copy)]
pub struct BasisEdge<C: Coordinate>(pub C);

impl<C: Coordinate> Ord for BasisEdge<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}
impl<C: Coordinate> PartialOrd for BasisEdge<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<C: Coordinate> PartialEq for BasisEdge<C> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl<C: Coordinate> Eq for BasisEdge<C> {}
```

| Structure              | Before                       | After                                      |
| ---------------------- | ---------------------------- | ------------------------------------------ |
| `GvGraph` field        | `BTreeMap<C, Plateau<C, V>>` | `BTreeMap<BasisEdge<C>, Plateau<C, V>>`    |
| `PlateauBasis` forward | `BTreeMap<C, Vec<GNodeId>>`  | `BTreeMap<BasisEdge<C>, HashSet<GNodeId>>` |
| `PlateauBasis` back    | `HashMap<GNodeId, C>`        | `HashMap<GNodeId, BasisEdge<C>>`           |
| `GvGraph` bound        | `C: Coordinate + Ord`        | `C: Coordinate` (no `Ord`)                 |

**Why this is correct:**

- **No specialization** — int vs float difference is just normal
  trait method dispatch through `Coordinate::total_cmp`.
- **No precision loss** — integers use native `Ord::cmp`; floats
  use native `total_cmp` (stable since Rust 1.62). No `to_f64()`
  conversion. NaN sorts deterministically (after +∞).
- **Newtype pattern** — `BasisEdge` is a zero-cost wrapper adding
  `Ord` to any `Coordinate`. Same pattern as `Reverse<T>`,
  `OrderedFloat<T>`.
- **`GvGraph` stays `C: Coordinate`** — no `Ord` bound, no float
  exclusion. `GvGraph<f64, f64, 8>` compiles.
- **Semantic naming** — `BasisEdge` names what the key _means_
  (where terminal-depth contour begins — P-I1), not just what
  it _is_.

This supersedes the original Q7 decision ("floats excluded").

## Decision

**All questions resolved.**

| Q   | Resolution                                                                                 | Rationale                                                                                         |
| --- | ------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------- |
| Q1  | `Plateau` is `Copy`: `basis_edge`, `start`, `end`, `depth`, `sum`. No `basis` on struct.   | View type consistency. `basis_edge` carries P-I1 key. Bookkeeping externalized to `PlateauBasis`. |
| Q2  | Keep `get()` returning `Cell` alongside BTreeMap.                                          | Two abstractions, two questions. Routing truth (trimmed) vs structural truth (thatched).          |
| Q3  | Three metrics: `plateaus().len()`, `node_count()`, `terminal_count()`.                     | Each `O(1)`.                                                                                      |
| Q4  | Infallible `Cell<C, V>` with domain clamping. NaN panics.                                  | Consistent with `range_sum` and `observe`.                                                        |
| Q5  | `HashMap<GNodeId, BasisEdge<C>>` inside `PlateauBasis`.                                    | GNode stays 48 bytes. `O(1)` amortized.                                                           |
| Q6  | Complementary projections — no unification.                                                | `extract()` = V-Tree (cold). `plateaus()` = G-Tree (hot).                                         |
| Q7  | `BasisEdge<C>` newtype + `Coordinate::total_cmp`. No `Ord` on `GvGraph`. Floats supported. | Zero-cost `Ord` wrapper. Semantic key naming (P-I1). No float exclusion.                          |

## Consequences

- **The BTreeMap is an actively-maintained mirror** of the G-Tree
  contour, stored as a field of `GvGraph`. `plateaus()` returns
  `Cow<'_, BTreeMap<BasisEdge<C>, Plateau<C, V>>>` — a zero-cost
  borrow with `dynamic-contour-tracking` (default), or an `O(G)`
  rebuild without.

- **P-I1 through P-I5 are the governing invariants.** Every
  mutation to the BTreeMap mirror and `PlateauBasis` must preserve
  all five. P-I1 ensures basis edge keys tile the domain. P-I2
  ensures the basis is minimal (no ancestor could replace an
  element). P-I3 ensures basis elements' tiles are disjoint across
  plateaus. P-I4 ensures semi-internal basis elements
  each thatch exactly one plateau (one-hop). P-I5 bounds thatch
  stacking at `d_geo(x) + 1`.

- **`Plateau<C, V>` is a `Copy` view type** alongside `Cell`,
  `Node`, and `Span` (ADR-M-008). Five public fields: `basis_edge`,
  `start`, `end`, `depth`, `sum`. ~36 bytes for `<u64, u64>`.
  The `start`/`end` fields encode the thatched extent (P-I4).
  The `basis_edge` field carries the ordered map key (P-I1).

- **`BasisEdge<C>` is a public zero-cost newtype** providing `Ord`
  for any `Coordinate` via `Coordinate::total_cmp`. It appears
  as the key type of `plateaus()` and is needed by callers for
  floor-key lookups. Semantic name derived from P-I1.

- **`PlateauBasis<C>` is a private bookkeeping struct** inside
  `GvGraph`. Forward map (`BTreeMap<BasisEdge<C>, HashSet<GNodeId>>`)
  and back-pointer map (`HashMap<GNodeId, BasisEdge<C>>`).
  Updated atomically by `split`/`evict`/`decay`. GNode stays at
  48 bytes.

- **`GvGraph` requires only `C: Coordinate`** — no `Ord` bound.
  Float coordinate types compile. The `Ord` requirement is
  encapsulated inside `BasisEdge<C>`.

- **Incremental maintenance preserves P-I1 through P-I5.**
  `observe()` changes magnitudes only — all five trivially
  preserved. `split()` and `evict()` change basis assignments
  — each operation maintains tiling (P-I1), minimality (P-I2),
  disjointness (P-I3), one-hop thatch (P-I4), and depth bound
  (P-I5) by construction. `decay()` changes magnitudes only.

- **Two point-query answers at different abstraction levels.**
  `get(coord)` returns a `Cell` with the trimmed half-interval.
  `plateaus().range(..=BasisEdge(coord)).next_back()` returns a
  `&Plateau` with the thatched run (P-I4) and basis edge (P-I1).

- **Three complementary metrics.** `plateaus().len()` = structural
  complexity. `node_count()` = arena resource usage.
  `terminal_count()` = resolution. All `O(1)`.

- **Two complementary projections.** `extract()` is the V-Tree's
  significance-ordered story (cold path, V-I1–V-I7). `plateaus()`
  is the G-Tree's spatially-ordered story (hot path, P-I1–P-I5).

- **Structural lifecycle.** `observe()` can create new plateau
  boundaries (splits introduce depth transitions). `decay()` can
  remove them (eviction smooths the contour). `plateaus().len()`
  is an `O(1)` heartbeat of the tree's structural complexity:
  rising = learning new structure, falling = forgetting old
  structure. All five invariants hold continuously through these
  transitions.

- **ADR-M-025 impact:** The live mirror replaces snapshot-based
  `plateaus()`. `SpatialRead::plateaus()` returns
  `Cow<'_, BTreeMap<BasisEdge<C>, Plateau<...>>>`.

- **ADR-M-009 impact:** `SpatialRead` shrinks further. `plateaus()`
  returns `Cow::Borrowed` with `dynamic-contour-tracking`,
  `Cow::Owned` without. `get()` is infallible.
