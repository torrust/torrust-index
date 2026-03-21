# ADR-M-001: Node Storage Strategy

**Status:** Decided  
**Date:** 2026-02-24  
**Relates to:** [ADR-M-002](002-vtree-node-enum.md) (V-Tree node
representation — determines the two-arena topology),
[ADR-M-029](029-opportunistic-depth-caching.md) (adds `cached_depth`
to `VNode`, occupies padding gap — preserves 64-byte layout)  
**Spec:** §IDEA M-4.1–§IDEA M-4.3 (node types), §IDEA M-4.4 (destruction and
liveness)  
**API:** [§API M-6](../docs/api.md#6-surface-3--emulsion) (Emulsion
module table), [§API M-4.4](../docs/api.md#44-handles) (handle
types)  
**Surface:** 3 (Emulsion)

## Context

The GvGraph contains two arenas of nodes — `Arena<GNode<C, V>>` and
`Arena<VNode<V>>` — with cross-references between them. The spec
defines three logical node types (G-Node, V-Entry, V-Structural);
[ADR-M-002](002-vtree-node-enum.md) unifies the two V-Tree types into
a single `VNode<V>` with a `VKind<V>` discriminant, so only two
arena pools are needed. Nodes are created (splits, rebalancing) and
destroyed (eviction, structural collapse). The storage strategy
affects cache locality, borrow-checker ergonomics, allocation cost,
and invalidation safety.

**Constraint:** std-only — no external dependencies.

## Options Considered

| Option                | Mechanism                                     | Cache | Safety     | Reuse      |
| --------------------- | --------------------------------------------- | ----- | ---------- | ---------- |
| A. Vec + free list    | `Vec<T>`, newtype index handles               | ★★★   | None       | ✓ freelist |
| B. Generational Vec   | `Vec<Slot<T>>`, generation counter on handles | ★★★   | Debug-only | ✓ freelist |
| C. HashMap            | `HashMap<u64, T>`, monotonic IDs              | ★     | Always     | ✓ auto     |
| D. Vec\<Option\<T\>\> | `Vec<Option<T>>`, no free list                | ★★½   | Always     | ✗ grows    |

## Decision

**Option B — Generational `Vec` arenas**, refined with:

- **`NonZeroU32` niche-optimized handles** — `Option<GNodeId>` and
  `Option<VNodeId>` are exactly 4 bytes (no discriminant overhead).
- **Separate `Vec<u64>` bitset** for occupancy tracking instead of
  per-slot generation counters, keeping node structs cache-line sized.
- **`debug_assert!`** on occupancy for stale-handle detection — catches
  use-after-free during development, zero-cost in release.
- **Free list** (`Vec<u32>`) for O(1) slot reuse.

Nodes require `T: Default` for `std::mem::take` on deallocation
(avoids unsafe `MaybeUninit`).

### Arena\<T\> summary

```rust
struct Arena<T: Default> {
    slots: Vec<T>,          // dense storage, T is cache-line sized
    occupied: Vec<u64>,     // bitset: 1 bit per slot, 64 slots per u64
    free: Vec<u32>,         // free list for O(1) reuse
    count: u32,             // live entry count
}
```

**Hot path (get/get_mut):** single `Vec` index — `debug_assert` on
occupancy, compiles to bare array access in release.

**Alloc:** pop from free list or push to vec — O(1).

**Dealloc:** `std::mem::take` + push index to free list — O(1).

### Handle types

```rust
use std::num::NonZeroU32;

struct GNodeId(NonZeroU32);   // 4 bytes, Option = 4 bytes
struct VNodeId(NonZeroU32);   // 4 bytes, Option = 4 bytes
```

Type-safe: `GNodeId` cannot index the V-node arena (compile error).
Opaque outside the crate — `from_index()` and `index()` are
currently `pub` (§API M-4.4 notes this as potential future
tightening).

### Liveness guard — `is_occupied`

The bitset provides an O(1) `is_occupied(index)` check that returns
whether a slot currently holds a live object. This implements the
`slot_occupied` liveness guard described in §IDEA M-4.4: stale
handles may persist in work queues — the rebalance loop (§IDEA M-11.8) and
the eviction scan (§IDEA M-12.6, Phase 2 re-verification) both check
`is_occupied` before accessing a queued node. With bitset-tracked
allocation, this is a single word-read + bit-test — no generation
counter comparison needed.

### Two-arena topology

The decision produces two arena instances in `GvGraph`:

| Arena                        | Stored type   | Node size               | Spec types covered                                                                                        |
| ---------------------------- | ------------- | ----------------------- | --------------------------------------------------------------------------------------------------------- |
| `gnodes: Arena<GNode<C, V>>` | `GNode<C, V>` | 48 bytes (`<u64, u64>`) | G-Node (§IDEA M-4.1)                                                                                      |
| `vnodes: Arena<VNode<V>>`    | `VNode<V>`    | 64 bytes (`<u64>`)      | V-Entry (§IDEA M-4.2) + V-Structural (§IDEA M-4.3) via `VKind` enum ([ADR-M-002](002-vtree-node-enum.md)) |

Both types fit in a single 64-byte cache line.

## Consequences

- Cache-line-sized nodes: `GNode<u64, u64>` (48 bytes) and
  `VNode<u64>` (64 bytes) each fit in one 64-byte cache line.
- No external crate dependency (slotmap, thunderdome, etc.).
- Stale handle bugs caught in debug builds via `debug_assert`.
- Free list prevents `None`-hole fragmentation from expand/contract
  cycles.
- `Vec<u64>` bitset adds ~1 bit per slot overhead (cold, never on
  hot path). Doubles as the O(1) liveness guard for §IDEA M-4.4
  `slot_occupied` semantics.
- Two arenas with one handle type each — simpler than three pools
  with a union handle (see [ADR-M-002](002-vtree-node-enum.md)).

## std types used

`Vec<T>`, `Vec<u64>`, `Vec<u32>`, `NonZeroU32`, `Option`.
