# ADR-M-002: V-Tree Node Representation

**Status:** Decided  
**Date:** 2026-02-24  
**Updated:** 2026-03-03  
**Amended by:** [ADR-M-029](029-opportunistic-depth-caching.md) (adds
`cached_depth` to `VNode`, occupies padding gap — preserves 64-byte
layout)  
**Spec:** §IDEA M-4.2 (V-Entry fields and flags), §IDEA M-4.3
(V-Structural fields and flags), §IDEA M-6.2 (V-I4–V-I7 invariants)  
**API:** [§API M-6](../docs/api.md#6-surface-3--emulsion) (`vnode`
module — `VNode<V>`, `VKind<V>`, `PackedChildren<V>`)  
**Surface:** 3 (Emulsion)

**Related ADRs:**

- [ADR-M-001](001-node-storage.md): arena topology — determines that
  `VNode<V>` lives in a single `Arena<VNode<V>>` with `VNodeId`
  handles
- [ADR-M-013](013-eviction-eligibility.md): eviction eligibility uses
  `is_evictable` on entries and `has_evictable` on structural nodes
- [ADR-M-016](016-semi-internal-state.md): semi-internal state is
  inferred from G-node children, not stored on V-nodes
- [ADR-M-027](027-observation-receiving-reframe.md): flag rename
  (`is_geo_terminal` → `is_evictable`, `has_geo_terminal` →
  `has_evictable`) and `is_exposed` / `is_evictable` split

## Context

The spec defines two V-Tree node types: V-Entry (leaf, backs a
G-node — §IDEA M-4.2) and V-Structural (internal, 2–3 children —
§IDEA M-4.3). These could be represented as separate types behind a
trait, or as variants of a single enum. The choice affects
pattern-matching ergonomics, arena design, and traversal code.

## Options Considered

1. **Separate types + trait:** `VEntry<V>` and `VStructural<V>` in
   separate arenas, unified by a `VNode` trait. Requires two arena
   pools and a `VNodeId` enum to distinguish which pool to index.

2. **Single enum:** `VNode<V>` with a `VKind<V>` enum. One arena,
   one handle type. Pattern match on `node.kind` at each use site.

## Decision

**Option 2 — Single `VNode<V>` struct with `VKind<V>` enum.**

```rust
struct VNode<V> {                  // 64 bytes for V = u64 — one cache line
    intensity: V,                  //  8 bytes: sum for structural, own for entry
    parent: Option<VNodeId>,       //  4 bytes
    cached_depth: AtomicU32,       //  4 bytes: padding gap (ADR-M-029)
    kind: VKind<V>,                // 48 bytes
}

enum VKind<V> {
    Entry {
        gnode: GNodeId,            // backing G-node (V-I4)
        is_exposed: bool,          // on contour? (V-I6)
        is_evictable: bool,        // zero G-children? (V-I6b)
    },
    Structural {
        children: PackedChildren<V>,
        has_evictable: bool,       // any descendant evictable? (V-I7)
    },
}
```

> **`cached_depth`:** Added by [ADR-M-029](029-opportunistic-depth-caching.md).
> Uses the 4-byte padding gap between `parent` (4 bytes, align 4) and
> `kind` (48 bytes, align 8) — zero memory overhead. `AtomicU32` for
> thread-safe lazy-invalidation caching; `DEPTH_STALE` (`u32::MAX`) marks
> stale entries.

### Entry flags (§IDEA M-4.2)

Two booleans on `VKind::Entry`, each serving a distinct purpose:

- **`is_exposed`** (V-I6): true iff the backing G-node has uncovered
  range — true for terminals AND semi-internals. Tracks contour
  membership — describes what the node **is** (exposed to
  observations), not what can be done to it. Governs observation
  routing. Changes when the G-node splits (flips to false when both
  children exist) and when children are removed (flips to true when
  uncovered range reappears).
- **`is_evictable`** (V-I6b): true iff the backing G-node has zero
  G-children. Caches `¬has_dependents(gnode)` to avoid pointer
  chasing into the G-Tree during eviction scans and flag propagation.
  Maintained by split (flips to false when a child is added) and
  eviction/restoration (flips to true when all children are removed).

### Structural flag (§IDEA M-4.3)

- **`has_evictable`** (V-I7): logical OR over descendant entries'
  `is_evictable` flags — standard bottom-up propagation via
  `propagate_evictable_flags` (§IDEA M-9.3). Enables efficient eviction
  scanning: subtrees where `has_evictable` is false contain nothing
  evictable and can be skipped entirely.

### PackedChildren: SoA layout for cache-friendly sampling

```rust
struct PackedChildren<V> {         // 40 bytes for V = u64
    intensities: [V; 3],          // 24 bytes — hot: one scan per sampling step
    ids: [Option<VNodeId>; 3],    // 12 bytes — cold: read only after winner chosen
    len: u8,                      //  1 byte  — 2 or 3
}
```

The struct-of-arrays (SoA) layout separates intensities from IDs.
The sampling decision at each V-level reads 24 bytes of contiguous
intensities — no interleaved pointer data, no cache pollution.

Child intensities are **cached** in the parent's `PackedChildren`
alongside the authoritative copy in each child's `.intensity` field.
The cached copy is updated during sum propagation (which already
visits the parent), so maintenance cost is zero.

## Consequences

- One arena (`Arena<VNode<V>>`) and one handle type (`VNodeId`) for
  all V-Tree nodes. Simpler than two pools + enum handle.
- Pattern matching on `node.kind` is ergonomic and exhaustive —
  the compiler enforces that both variants are handled.
- Sampling touches one cache line per V-level: 24 bytes of intensities
  - 4 bytes for the chosen child's ID.
- Contraction/promotion rewrites a 40-byte `PackedChildren` inline —
  O(1), bounded to 3–4 cache lines per restructuring operation.
- Entry is permanently an entry; structural is permanently structural
  (V-I5). No transmutation — the `VKind` is set at creation and
  never changes. Rebalancing creates and destroys only structural
  nodes; entries are rearranged but never transmuted.
- Three flags serve three orthogonal purposes (§§IDEA M-4.2–4.3):
  `is_exposed` (contour membership — V-I6), `is_evictable` (eviction
  eligibility — V-I6b), `has_evictable` (subtree eviction scan
  pruning — V-I7).
- `cached_depth` fits in the alignment gap at zero memory cost,
  converting O(h_V) depth queries to amortized O(1) — see
  [ADR-M-029](029-opportunistic-depth-caching.md).
