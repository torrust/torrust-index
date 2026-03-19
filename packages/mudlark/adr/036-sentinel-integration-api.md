# ADR-M-036: Sentinel Integration API

**Status:** Implemented
**Date:** 2026-03-08
**Decided:** 2026-03-08
**Implemented:** 2026-03-08
**Relates to:** [ADR-M-032](032-three-surface-model.md) (three-surface model),
[ADR-M-025](025-public-api-surface.md) (public API surface),
[ADR-M-030](030-graph-module-decomposition.md) (graph module decomposition),
[ADR-M-019](019-sampling-semantics.md) (sampling semantics)
**Spec:** §IDEA M-5.2 (routing), §IDEA M-6.5 (sampling), §IDEA M-8 (observation
pipeline), §IDEA M-14 (temporal decay), §IDEA M-18.1 (Fibonacci depth bound)
**API:** [§API M-5.2](../docs/api.md#52-gvgraphc-v-n--the-negative) (GvGraph
operations)
**Surface:** 1 + 2 (Prints + Film)

---

## Context

The `torrust-sentinel` package (Spectral Sentinel — hierarchical online
subspace anomaly detector) specifies a three-layer architecture (see
`packages/sentinel/docs/algorithm.md`):

```
Layer 1: G-V Graph  ──  adaptive spatial partitioning, Δ = 1 always
Layer 2: Analysis Selector  ──  reads V-Tree ranking, picks top-K cells
Layer 3: Analysis Engine  ──  SubspaceTrackers, scoring, CUSUM, coordination
```

The sentinel's specification requires two capabilities that mudlark's
current public API does not provide:

1. **G-node identity on `Node` view types.** The analysis selector
   (Layer 2) iterates V-Tree entries via `layers()` and needs a stable
   handle to key its tracker map, check ancestry relationships for
   multi-scale observation delivery, and identify coordination contexts.

2. **G-node structural queries.** The hierarchical coordination tier
   (Layer 3) walks from analysed cells upward through the G-tree to
   find internal nodes whose left and right subtrees both contain
   reporting cells. This requires read access to a G-node's interval,
   children, and parent.

### What exists today

The current public API on `GvGraph` exposes:

| Method | Surface | Returns | Sentinel use |
|--------|---------|---------|--------------|
| `observe(coord, delta)` | 2 (Film) | — | Volume accounting (Δ = 1) |
| `decay(root, att, q)` | 2 (Film) | — | Spatial memory |
| `get(coord)` | 1+2 | `Cell` | Point query |
| `sample(rng)` | 1+2 | `Option<Cell>` | — |
| `range_sum(range)` | 1+2 | `V` | — |
| `extract()` | 1+2 | `Pewei` | Spatial portrait |
| `layers()` | 1+2 | `impl Iterator<Item = (usize, Node)>` | **Analysis selector** |
| `plateaus()` | 1+2 | `Cow<'_, BTreeMap<BasisEdge, Plateau>>` | Contour snapshot |
| `g_root()` | 1 | `GNodeId` | Decay root, coordination root |
| `node_count()` | 1 | `u32` | Health reporting |
| `terminal_count()` | 1 | `u32` | Health reporting |
| `total_sum()` | 1 | `V` | Health reporting |

The gap: `layers()` yields `(usize, Node)` where `Node` has `start`,
`end`, `own`, `sum`, `depth`, `state` — but no `GNodeId`. And there
is no public way to query a G-node's children or parent given a handle.

### Design constraints

1. **ADR-M-032 crossing rules.** New API must respect the three-surface
   model. Surface 1 types are `Copy`/`Clone` view types with no `&mut`
   back into the graph. Surface 2 types are opaque operational types.
   Surface 3 machinery stays `pub(crate)`.

2. **`GNodeId` is already Surface 1.** It is a public `Copy` handle
   re-exported from the crate root (ADR-M-032 §S1, Handles table). Adding
   it to `Node` does not promote any internal type.

3. **Minimal surface principle** (§API M-1). Expose only what the sentinel
   needs, not the full internal structure. The sentinel should not be
   able to mutate tree structure or bypass the observation pipeline.

4. **Read-only.** All new API is `&self` — no new mutation paths beyond
   the existing `observe()` and `decay()`.

---

## Decision

### D1 — Add `gnode_id` field to `Node<C, V>`

`Node` gains a `GNodeId` field:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Node<C: Coordinate, V: Accumulator> {
    pub start: C,
    pub end: C,
    pub own: V,
    pub sum: V,
    pub depth: u32,
    pub state: GState,
    pub gnode_id: GNodeId,  // NEW — arena handle of the backing G-node
}
```

**Rationale:**

- `GNodeId` is already public Surface 1 (`Copy`, opaque, `NonZeroU32`
  index). Adding it to `Node` doesn't expose any Surface 3 internals.
- The sentinel uses it to key its per-cell tracker map
  (`BTreeMap<GNodeId, CellState>`), avoiding reverse-lookups from
  `(start, depth)`.
- `layers()` constructs `Node` in the `Layers` iterator where the
  `GNodeId` is already in hand (via `VKind::Entry { gnode, .. }`),
  so the change is non-invasive in the emulsion.
- The alternative — a parallel `layers_with_ids()` — adds a second
  iterator covering the same traversal with the same logic and an
  unnecessary API decision point for callers.

**Impact on `extract()`:** The `Pewei` / `Layer` / `Terminal` /
`Transition` types do **not** gain `GNodeId`. They are self-contained
snapshots (contact prints) that must remain valid after the graph has
mutated. Arena handles into a mutated graph are dangling references
in spirit, even if they're index-based. Keeping handles off the
extraction types preserves the "contact print" invariant from ADR-M-032.

### D2 — Add `gnode_info()` read-only structural query

A new method on `GvGraph`:

```rust
impl<C: Coordinate, V: Accumulator, const N: u32> GvGraph<C, V, N> {
    /// Read-only snapshot of a G-node's spatial and structural state.
    ///
    /// Returns `None` if `id` does not refer to a live G-node.
    ///
    /// The returned [`GNodeInfo`] is a `Copy` snapshot — it does not
    /// borrow the graph. Handles in `left`, `right`, and `parent`
    /// are live at the time of the call but may become stale after
    /// subsequent `observe()` or `decay()` calls.
    ///
    /// # Cost
    ///
    /// $O(1)$ — single arena lookup plus depth computation.
    pub fn gnode_info(&self, id: GNodeId) -> Option<GNodeInfo<C, V>>;
}
```

With a new Surface 1 view type:

```rust
/// Snapshot of a G-node's spatial extent, accumulation, and
/// tree-structural neighbours.
///
/// Returned by [`GvGraph::gnode_info()`]. All fields are values
/// captured at call time — the snapshot does not borrow the graph.
///
/// The `left`, `right`, and `parent` handles are live at snapshot
/// time but may become stale after graph mutations (`observe()`,
/// `decay()`). They are intended for single-pass tree walks within
/// a read-only borrow of the graph, not for long-lived caching.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GNodeInfo<C: Coordinate, V: Accumulator> {
    /// Lower bound of the dyadic range (inclusive).
    pub start: C,
    /// Upper bound of the dyadic range (exclusive).
    pub end: C,
    /// Direct accumulation at this node.
    pub own: V,
    /// Total value: `own + children.sum`.
    pub sum: V,
    /// G-Tree depth of this node.
    pub depth: u32,
    /// Categorical state at snapshot time.
    pub state: GState,
    /// Left child G-node, if any.
    pub left: Option<GNodeId>,
    /// Right child G-node, if any.
    pub right: Option<GNodeId>,
    /// Parent G-node. `None` for the root.
    pub parent: Option<GNodeId>,
}
```

**Rationale:**

- The sentinel's hierarchical coordination (algorithm.md §9) requires
  a bottom-up walk from each analysed cell to the G-root, collecting
  coordination contexts at every internal node whose left and right
  subtrees both contain reporting cells.
- `GNodeInfo` is a `Copy` snapshot (Surface 1 Print) — it exposes
  the same fields as `Node` plus `left`, `right`, `parent` handles.
  All handles are existing Surface 1 types.
- The method is `&self` and $O(1)$. It adds no mutation path.
- Naming follows the existing pattern: `gnode_depth` (internal) →
  `gnode_info` (public). "Info" communicates read-only structural
  snapshot.

**Why not expose `GNode` directly?** `GNode<C, V>` is Surface 3
(Emulsion). It contains arena indices, entry pointers, and invariant
fields that downstream code must not depend on. `GNodeInfo` is a
curated projection — the same narrowing principle as `Cell` projecting
from the `GNode` in `get()`.

### D3 — Add `is_ancestor_of()` containment check

```rust
impl<C: Coordinate, V: Accumulator, const N: u32> GvGraph<C, V, N> {
    /// Whether `ancestor` is a proper ancestor of `descendant` in
    /// the G-Tree (i.e., `descendant` is in `ancestor`'s subtree
    /// and `ancestor ≠ descendant`).
    ///
    /// Uses interval containment: `ancestor` is an ancestor of
    /// `descendant` iff `ancestor.lo <= descendant.lo` and
    /// `ancestor.hi >= descendant.hi` and the intervals are not
    /// equal.
    ///
    /// Returns `false` if either handle is not live.
    ///
    /// # Cost
    ///
    /// $O(1)$ — two arena lookups plus interval comparison.
    pub fn is_ancestor_of(&self, ancestor: GNodeId, descendant: GNodeId) -> bool;
}
```

**Rationale:**

- The sentinel's multi-scale delivery (algorithm.md §4.3) delivers
  each observation to every analysis ancestor on its routing path.
  It needs to test whether a cell's interval contains a given value,
  which it can do with `start <= value < end` from the `Node` or
  `GNodeInfo` snapshot. But for coordination, it also needs to test
  cell-to-cell ancestry.
- In a dyadic tree, ancestry is equivalent to strict interval
  containment, so this is a pure arithmetic check — $O(1)$, no
  tree walk.
- The sentinel *could* perform this check itself using `start`/`end`
  from `GNodeInfo`, but providing it as a method on `GvGraph` is
  more ergonomic and self-documenting. It also insulates the caller
  from the subtlety that interval endpoints must be compared with
  `<=` / `>=` (not `<` / `>`), and handles the stale-handle case.

---

## Alternatives considered

### A1 — Use `(start, depth)` as the tracker key instead of `GNodeId`

In a dyadic tree, `(start, depth)` uniquely identifies a G-node.
The sentinel could use this pair instead of `GNodeId`.

**Rejected** because:
- Every tracker map lookup becomes a composite-key comparison instead
  of a single `u32` comparison.
- Multi-scale delivery requires testing "is cell A an ancestor of
  cell B?" — with handles this is `is_ancestor_of(a, b)`; with
  `(start, depth)` the caller must implement interval arithmetic.
- The `GNodeId` is already public and `Copy`. Not exposing it on
  `Node` forces callers into a strictly worse API for no benefit.

### A2 — Add a separate `layers_with_ids()` iterator

Keep `Node` as-is and add a parallel iterator that yields
`(usize, GNodeId, Node)`.

**Rejected** because:
- Two iterators over the same traversal with the same logic are an
  unnecessary maintenance and documentation burden.
- `GNodeId` on `Node` is a strict superset — callers who don't need
  it simply ignore the field. There is no cost (it's one `u32`).
- `Node` already carries `GState` (which is similarly a structural
  concern), so adding `GNodeId` is consistent.

### A3 — Expose raw `GNode<C, V>` references

Make `GNode` public and add `pub fn gnode(&self, id: GNodeId) -> &GNode`
that returns a borrow.

**Rejected** because:
- `GNode` is Surface 3 (Emulsion). Exposing it violates ADR-M-032's
  crossing rules — Film methods must not expose Emulsion types in
  their public signatures.
- A borrowed `&GNode` holds a lifetime into the arena, preventing
  concurrent graph mutation. The sentinel needs to iterate nodes
  and then mutate (call `observe()`). A `Copy` snapshot avoids this.
- `GNode` contains `entry: Option<VNodeId>` and
  `parent: Option<GNodeId>` which are invariant-laden fields that
  downstream code should not reason about.

### A4 — Full tree-walking iterator (DFS/BFS from arbitrary root)

Add `pub fn walk(&self, root: GNodeId) -> impl Iterator<Item = GNodeInfo>`.

**Rejected for now** because:
- The sentinel's coordination walk is application-specific: it cares
  about which nodes have *analysed cells* in both subtrees, not
  about visiting every node. A generic walk would visit many
  irrelevant nodes.
- `gnode_info()` + the sentinel's own recursive logic is more
  expressive (the sentinel decides the walk order and pruning) and
  adds less to mudlark's API surface.
- If a generic walk proves useful later, it can be added without
  breaking changes.

### A5 — Change notification callbacks

Have `observe()` and `decay()` emit structural change events (split,
evict, rebalance) so the sentinel can maintain the analysis set
incrementally instead of recomputing via `layers()`.

**Deferred.** This is an optimisation for high node-count scenarios.
The current `layers()` BFS is $O(V)$ where $V$ is the V-node count.
Profiling should determine whether this is a bottleneck before adding
callback infrastructure. If needed, a future ADR can introduce a
`ChangeLog` mechanism without altering the API established here.

---

## Implementation

### File changes in `packages/mudlark/`

| File | Change | Surface |
|------|--------|---------|
| `src/view.rs` | Add `gnode_id: GNodeId` field to `Node` | 1 |
| `src/graph_extract.rs` | Set `gnode_id` in `Layers::next()` | 3 |
| `src/graph.rs` | Add `GNodeInfo` struct, `gnode_info()`, `is_ancestor_of()` | 1 + 2 |
| `src/lib.rs` | Re-export `GNodeInfo` | 1 |
| `src/handle.rs` | Add `serde` derives to `GNodeId` and `VNodeId`; expand doc table | 1 |
| `src/tests/view.rs` | Add `gnode_id` to 12 internal `Node` struct literals | (test-only) |
| `tests/sentinel_api.rs` | New integration test file — 14 tests | (test-only) |

### D1 — `Node.gnode_id`

In `view.rs`, add the field to the struct definition. In
`graph_extract.rs`, the `Layers` iterator already has `*gnode` in
scope:

```rust
// graph_extract.rs — Layers::next()
VKind::Entry { gnode, .. } => {
    let g = self.graph.gnodes.get(gnode.index());
    let g_depth = self.graph.gnode_depth(*gnode);
    let node = crate::view::Node {
        start: g.lo,
        end: g.hi,
        own: g.own,
        sum: g.sum,
        depth: g_depth,
        state: g.state(),
        gnode_id: *gnode,  // NEW
    };
    return Some((bfs_depth, node));
}
```

### D2 — `gnode_info()`

In `graph.rs`, add:

```rust
impl<C: Coordinate, V: Accumulator, const N: u32> GvGraph<C, V, N> {
    pub fn gnode_info(&self, id: GNodeId) -> Option<GNodeInfo<C, V>> {
        if !self.gnodes.is_occupied(id.index()) {
            return None;
        }
        let g = self.gnodes.get(id.index());
        Some(GNodeInfo {
            start: g.lo,
            end: g.hi,
            own: g.own,
            sum: g.sum,
            depth: crate::gtree::gnode_depth_from_interval(g.lo, g.hi, N),
            state: g.state(),
            left: g.left,
            right: g.right,
            parent: g.parent,
        })
    }
}
```

### D3 — `is_ancestor_of()`

In `graph.rs`, add:

```rust
impl<C: Coordinate, V: Accumulator, const N: u32> GvGraph<C, V, N> {
    pub fn is_ancestor_of(&self, ancestor: GNodeId, descendant: GNodeId) -> bool {
        if !self.gnodes.is_occupied(ancestor.index())
            || !self.gnodes.is_occupied(descendant.index())
        {
            return false;
        }
        let a = self.gnodes.get(ancestor.index());
        let d = self.gnodes.get(descendant.index());
        a.lo <= d.lo && a.hi >= d.hi && (a.lo != d.lo || a.hi != d.hi)
    }
}
```

### Re-exports

In `lib.rs`, add `GNodeInfo` to the Surface 1 re-exports:

```rust
pub use graph::GNodeInfo;
```

---

## Testing

| Test | Validates |
|------|-----------|
| `node_carries_gnode_id` | `layers()` yields `Node` with correct `gnode_id` matching the G-tree topology |
| `node_gnode_id_empty_tree` | Root-only tree: `layers()` yields one node whose `gnode_id == g_root()` |
| `gnode_info_live_node` | `gnode_info()` returns `Some` for live nodes with correct fields |
| `gnode_info_dead_handle` | `gnode_info()` returns `None` for handles that have been evicted |
| `gnode_info_root_has_no_parent` | `gnode_info(g_root()).parent` is `None` |
| `gnode_info_children_consistent` | `left`/`right` handles round-trip: `gnode_info(info.left.unwrap()).parent == Some(id)` |
| `gnode_info_sum_equals_own_plus_children` | `info.sum == info.own + Σ child.sum` for every node |
| `is_ancestor_of_root_ancestors_all` | `is_ancestor_of(g_root(), any_other)` is true |
| `is_ancestor_of_self_is_false` | `is_ancestor_of(x, x)` is false (proper ancestor) |
| `is_ancestor_of_stale_handles` | Both stale-ancestor and stale-descendant return false |
| `is_ancestor_of_sibling_is_false` | Sibling nodes are not ancestors of each other |
| `is_ancestor_of_parent_child` | Direct parent→child is true, child→parent is false |
| `is_ancestor_of_transitive` | Leaf-to-root path: every higher node is ancestor of every lower node |
| `range_tree_gnode_ids_consistent` | `range_tree` preset: every `layers()` gnode_id round-trips through `gnode_info()` |

All tests go in `tests/sentinel_api.rs` (integration tests on the
public API surface), per [§API M-7.4](../docs/api.md#74-test-organisation).

---

## Implementation notes

Recorded during implementation (2026-03-08).

### Serde derives on handles

`GNodeId` and `VNodeId` in `handle.rs` gained
`#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]`.
This was not anticipated in the original decision but was required
because:

- `Node<C, V>` already had the serde derive and now contains a
  `GNodeId` field. Without the derive on the handle, compilation
  fails when the `serde` feature is enabled.
- `GNodeInfo<C, V>` contains `Option<GNodeId>` fields (`left`,
  `right`, `parent`), triggering the same requirement.
- `VNodeId` was given the same treatment for consistency — both
  handle types share the same derive set.

This is a minor Surface 1 expansion: downstream code with `serde`
enabled can now serialize/deserialize handles. The round-trip is
index-based (`NonZeroU32`), consistent with the existing
`from_index()` / `index()` API.

### handle.rs documentation update

The `GNodeId` doc table in `handle.rs` was expanded with three new
rows (`Node::gnode_id`, `GvGraph::gnode_info`, `GvGraph::is_ancestor_of`)
and corresponding link reference definitions were added so rustdoc
resolves them correctly.

### Internal test updates (view.rs)

The 12 `Node` struct literal sites in `src/tests/view.rs` required
the new `gnode_id` field. These use `GNodeId::from_index(0)` as a
placeholder since the unit tests exercise view-type behaviour, not
G-tree identity. The ADR noted that "struct literal construction in
downstream code would break, but `Node` is only constructed inside
the crate" — the internal test module is that "inside the crate"
code.

### Budget in stale-handle tests

Stale-handle tests (`gnode_info_dead_handle`, `is_ancestor_of_stale_handles`)
use `budget: Some(100)`. The initial attempt with `budget: Some(20)`
failed because ADR-M-018 requires headroom > 81 for `depth_create = 3`
(§PEWEI M-headroom formula). 100 is the smallest round number that
exceeds the headroom bound while still being tight enough to trigger
evictions.

### `HashSet` instead of `BTreeSet`

Test code collects `GNodeId` values into `HashSet<GNodeId>` rather
than `BTreeSet`, because `GNodeId` implements `Hash + Eq` but not
`Ord`. This is correct — arena handles have no meaningful ordering.

---

## Consequences

- **Sentinel can depend on mudlark** for adaptive spatial partitioning,
  using `layers()` for analysis selection, `gnode_info()` for
  hierarchical coordination walks, and `is_ancestor_of()` for
  multi-scale delivery.

- **Surface 1 gains one type** (`GNodeInfo`) and **one field**
  (`Node.gnode_id`). Both are read-only `Copy` snapshots consistent
  with existing Surface 1 conventions.

- **Surface 2 gains two methods** (`gnode_info()`, `is_ancestor_of()`),
  both `&self`, both $O(1)$.

- **No Surface 3 changes** are exposed. The implementation is thin
  delegation from the Film layer to existing Emulsion machinery.

- **`extract()` / `Pewei` are unaffected.** Contact prints remain
  handle-free, preserving the snapshot independence invariant.

- **No breaking changes** to existing callers. The new `gnode_id` field
  on `Node` is additive (struct literal construction in downstream code
  would break, but `Node` is only constructed inside the crate).

- **Serde feature expanded.** When the `serde` feature is enabled,
  `GNodeId`, `VNodeId`, `Node`, and `GNodeInfo` are all serializable.
  This was a necessary consequence of adding `GNodeId` to types that
  already had serde derives.

- **Future path:** If profiling shows `layers()` is too expensive for
  analysis set recomputation at high node counts, §A5 (change
  notifications) can be revisited. The API established here remains
  valid — notifications would be an optimisation, not a replacement.
