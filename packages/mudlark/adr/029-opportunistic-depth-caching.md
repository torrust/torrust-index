# ADR-M-029: Opportunistic V-Tree Depth Caching

**Status:** Decided  
**Date:** 2026-03-02  
**Relates to:** [ADR-M-001](001-node-storage.md) (arena layout — 64-byte
`VNode` preserved), [ADR-M-002](002-vtree-node-enum.md) (VNode padding
gap), [ADR-M-015](015-eviction-scan-design.md) (eviction scan Phase 2
re-verification)  
**Spec:** §IDEA M-7 (Depth Gates), §IDEA M-10.1 (split gate), §IDEA M-11.9 (legacy
promotion gate), §IDEA M-12.6 (eviction re-verification, depth caching
recommendation)  
**API:** [§API M-6](../docs/api.md#6-surface-3--emulsion) (vtree
module: `v_depth`, `invalidate_depth_subtree`)  
**Surface:** 3 (Emulsion)

## Context

The V-Tree depth of a node is queried in three hot paths:

1. **Split gate (§IDEA M-10.1):** `v_depth(entry) <= D_create`
2. **Eviction re-verification (§IDEA M-12.6):** `v_depth(v) > D_evict`
3. **Legacy promotion dispatcher (§IDEA M-11.9):** `v_depth(c) <= D_evict`

Without caching, depth is computed by walking from the node to the
V-root, counting edges — O(h_V) per query, where h_V is V-Tree
height (~log₂ E).

**Primary cost concern.** During Phase 2 of eviction (§IDEA M-12.6), each
candidate is re-verified with `v_depth()`. For E_t candidates, total
re-check cost is O(E_t × h_V). Under sustained memory pressure with
large eviction batches, this dominates Phase 2 cost. The spec
recommends caching V-Tree depth to reduce re-checks to O(1) each.

**Tension.** Naïve absolute depth caching requires O(subtree)
maintenance on structural operations (contract, promote, collapse).
The goal is to save O(h_V) per query without paying O(subtree) per
structural change in the common case.

---

## Decision

**Opportunistic caching with lazy invalidation and path warming.**

Store a cached depth on each V-node using a sentinel value:

```rust
/// Sentinel indicating cached depth is stale (needs recompute).
const DEPTH_STALE: u32 = u32::MAX;

pub struct VNode<V> {
    pub intensity: V,
    pub parent: Option<VNodeId>,
    pub cached_depth: AtomicU32,  // DEPTH_STALE = needs recompute
    pub kind: VKind<V>,
}
```

### Query: Path-warming read

```rust
pub fn v_depth<V: Accumulator>(vnodes: &Arena<VNode<V>>, id: VNodeId) -> u32 {
    let node = vnodes.get(id.index());
    let cached = node.cached_depth.load(Ordering::Relaxed);

    // Fast path: cached and valid
    if cached != DEPTH_STALE {
        return cached;
    }

    // Slow path: recurse to parent, cache on return
    let depth = node.parent.map_or(0, |p| v_depth(vnodes, p) + 1);
    node.cached_depth.store(depth, Ordering::Relaxed);
    depth
}
```

On cache miss, the recursive call caches all stale ancestors on the
path to the first cached node (or root). A single query at depth d
with all-stale ancestors pays O(d) but warms the entire path.

A `debug_assert_eq!` in the implementation verifies cached values
against a full uncached walk, catching invariant violations in debug
builds.

### Maintenance: Classify by cost

| Operation                           | Depth Impact           | Strategy                                           |
| ----------------------------------- | ---------------------- | -------------------------------------------------- |
| `GvGraph::new` (§IDEA M-9.1 Case 1) | Root entry             | **Set** `0`                                        |
| `bootstrap_split` (§IDEA M-10.3)    | Create tree structure  | **Set** 5 nodes (root=0, L1=1, L2=2)               |
| `catalytic_split` (§IDEA M-10.2)    | Attach 3-node subtree  | **Set** 3 nodes (s=p+1, entries=p+2)               |
| `vtree_remove_leaf` 3→2             | No depth change        | **No-op**                                          |
| `vtree_remove_leaf` collapse        | Subtree moves up       | **Invalidate** subtree                             |
| `contract` (§IDEA M-11.3)           | Children pushed down   | **Set** merged=p+1; **Invalidate** both subtrees   |
| `standard_promote` (§IDEA M-11.4)   | Two children move up   | **Invalidate** both subtrees                       |
| `skip_promote` (§IDEA M-11.5)       | c and s move up        | **Invalidate** both subtrees                       |
| `legacy_promote` (§IDEA M-11.6)     | c rises, ne takes seat | **Set** c.depth−1 and ne=c_old_depth (both leaves) |

**O(1) operations** explicitly set the new depth value.
**O(subtree) operations** invalidate to `DEPTH_STALE` and let queries
repair lazily. Contraction is mixed: the new merged node is set O(1),
while both child subtrees are invalidated.

> _Implementation note._ The spec's `vtree_insert` (§IDEA M-9.1) is not a
> standalone function in the implementation. Entry creation is inlined
> into `GvGraph::new` (Case 1), `bootstrap_split` (§IDEA M-10.3), and
> `catalytic_split` (§IDEA M-10.2), each of which sets depths explicitly for
> all newly created nodes.

### Invalidation: Early-exit propagation

```rust
fn invalidate_depth_subtree<V: Accumulator>(vnodes: &Arena<VNode<V>>, root: VNodeId) {
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        let node = vnodes.get(id.index());

        // Already stale → subtree also stale (invariant)
        if node.cached_depth.load(Ordering::Relaxed) == DEPTH_STALE {
            continue;
        }

        node.cached_depth.store(DEPTH_STALE, Ordering::Relaxed);

        if let VKind::Structural { children, .. } = &node.kind {
            for i in 0..children.len() {
                stack.push(children.get(i).0);
            }
        }
    }
}
```

The early-exit is sound because of the **ancestor invariant**: if a
node has a valid cached depth, all its ancestors also have valid
caches. This is maintained because:

- Invalidation propagates downward (parent → children)
- Caching propagates upward (child → parent via recursion)

---

## Rationale

### Why not naïve absolute depth?

Naïve caching pays O(subtree) per structural operation. During a
rebalance cascade, a node may move +1 then −1 (contract then promote),
paying O(subtree) twice for net-zero change.

With opportunistic caching:

- First invalidation pays O(subtree)
- Second invalidation sees `DEPTH_STALE` and exits in O(1)
- Net cost for cascade: O(subtree) once, not O(moves × subtree)

### Why not relative depth (lazy deltas)?

The spec (§IDEA M-12.6) suggests a lazy delta scheme — storing depth relative
to the parent — to reduce maintenance to O(1) per structural
operation, since descendants of a moved node all shift by the same
constant. However, computing absolute depth from relative offsets
still requires an O(h_V) walk at query time. A full lazy-delta
approach with deferred propagation avoids the walk but requires
tracking pending deltas and produces temporarily stale absolute depths
during a cascade — similar bookkeeping to the sentinel approach,
without the early-exit benefit during back-to-back invalidations.

The sentinel approach is simpler: one field, one sentinel value, one
invalidation function. In the (2,3)-tree structure where cascades are
the norm, the early-exit amortization is the dominant benefit.

### Why not batched propagation?

Deferring updates until rebalance completes avoids redundant work
but requires tracking moved nodes in a set and produces stale
depths during the cascade. The opportunistic approach achieves the
same cascade amortization without tracking overhead.

### Eviction re-verification locality

The eviction scan (`scan_for_candidates`) is a DFS traversal that
tracks depth inline as a recursive parameter — it does not call
`v_depth()` and incurs no cached-depth overhead. The scan returns
candidates in DFS order.

The cached depth benefits the **re-verification step** in
`evict_candidates()`, where each candidate's V-Tree depth is
rechecked via `v_depth()` (depths may have shifted due to structural
collapses from prior evictions in the same batch). When iterating
candidates in the scan's DFS order:

- First candidate in a subtree warms the path to root: O(h_V)
- Sibling candidates share cached ancestors: O(1) per sibling
- Amortized cost for n candidates in DFS order: O(n + h_V)

Note: intervening `evict_tip()` calls may invalidate cached depths
via structural collapses (`vtree_remove_leaf` collapse →
`invalidate_depth_subtree`), partially reducing the locality benefit.
The amortized bound holds for the non-invalidated common case.

### Memory cost

**Key finding: zero memory overhead with sentinel encoding.**

The `VNode<u64>` layout has a 4-byte padding gap between `parent`
(4 bytes, niche-optimized `Option<VNodeId>`) and `kind` (48 bytes,
align 8). The `AtomicU32` cached_depth field fills this gap exactly:

```
Offset  Field             Size   Notes
──────────────────────────────────────────────
0       intensity         8      u64
8       parent            4      Option<VNodeId> (niche-optimized)
12      cached_depth      4      AtomicU32 (fills padding gap)
16      kind              48     VKind<u64> (align 8)
──────────────────────────────────────────────
Total                     64     One cache line
```

Verified by static assertion: `assert!(size_of::<VNode<u64>>() == 64)`.

**Representation choice:**

| Type                       | Size    | Works?                                 |
| -------------------------- | ------- | -------------------------------------- |
| `Cell<Option<u32>>`        | 8 bytes | NO — exceeds 64-byte cache line        |
| `Cell<Option<NonZeroU32>>` | 4 bytes | NO — cannot represent depth 0 (V-root) |
| `AtomicU32` with sentinel  | 4 bytes | YES — fits in padding gap, `Sync`-safe |

**Why not `Option<NonZeroU32>`?** Although niche-optimized to 4 bytes,
`NonZeroU32` cannot represent zero. The V-root has depth 0, so we'd
need to store `depth + 1` and subtract on read — adding complexity
and off-by-one risk. The sentinel approach (`u32::MAX` = stale) is
simpler and leaves `0..u32::MAX - 1` as valid depth values, far
exceeding any realistic V-Tree height.
