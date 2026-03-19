# ADR-M-019: Sampling Semantics

**Status:** Decided  
**Date:** 2026-02-26  
**Phase:** 4 (Output)  
**Relates to:** [ADR-M-008](008-span-type.md) (view types — Cell
snapshot), [ADR-M-009](009-trait-decomposition.md) (trait decomposition —
WeightedSampler, Rng), [ADR-M-025](025-public-api-surface.md) (public
API surface)  
**Spec:** §IDEA M-6.5 (proportional sampling)  
**API:** [§API M-5.2](../docs/api.md#proportional-sampling)
(proportional sampling, under GvGraph),
[§API M-5.4](../docs/api.md#54-instruments) (WeightedSampler trait)  
**Surface:** 2 (Film)

## Context

Proportional sampling (§IDEA M-6.5) walks the V-Tree from root to
a V-entry (V-Tree leaf), choosing each child with probability
proportional to its cached intensity. The implementation needs to
decide what the `sample()` method returns, and how to handle the
edge case where total intensity is zero.

## Questions

### Q1: Return type (DC-019-1)

What does `sample()` return?

| Option | Type                 | Notes                                                            |
| ------ | -------------------- | ---------------------------------------------------------------- |
| A      | `Option<Cell<C, V>>` | Snapshot view (ADR-M-008 addendum: no lifetime, copied fields).    |
| B      | `Option<Span<C, V>>` | Owned copy. No lifetime. Already decided in ADR-M-008.             |
| C      | `Option<GNodeId>`    | Raw handle — caller dereferences via graph. Minimal API surface. |

**Considerations:**

- `Cell` is the richest return. Per ADR-M-008 addendum, `Cell` is a
  small `Copy` snapshot struct (no lifetime, no pointer chase) — so
  it is both user-friendly and composable.
- `Span` copies data out — no borrow, composable, but loses the
  ability to inspect V-Tree depth or node state.
- `GNodeId` is the most primitive; fine for internal use but poor for
  a public API.

### Q2: Empty-tree / zero-intensity behaviour (DC-019-2)

When `I_total == 0` (all entries have zero intensity), the probability
distribution is undefined. What should `sample()` do?

| Option | Behaviour                       | Rationale                                                                                         |
| ------ | ------------------------------- | ------------------------------------------------------------------------------------------------- |
| A      | Return `None`                   | Follows `std::collections` convention for empty containers. Clear, safe.                          |
| B      | Uniform random over all entries | Mathematically principled (limit of uniform-epsilon), but surprising if the tree has zero energy. |
| C      | Return the root entry           | Deterministic, always valid, but semantically arbitrary.                                          |

**Considerations:**

- Option A is simplest and follows Rust convention (`BinaryHeap::peek`
  returns `None` when empty).
- The tree always has at least one entry (the G-root), so "empty" in
  the strictly structural sense never happens — but zero-intensity is
  a valid state.
- `I_total == 0` can occur on a freshly constructed graph before any
  observations.

## Decision

**DC-019-1: Option A — `Option<Cell<C, V>>`.**

```rust
fn sample(&self, rng: &mut impl Rng) -> Option<Cell<C, V>>;
```

Sampling always lands on a V-entry (V-Tree leaf). The backing
G-node is typically terminal but may be semi-internal or internal
(with frozen intensity from before the split — §IDEA M-6.5 says
"if v is entry: return v" with no guard on G-node state).
`Cell.intensity` is always `g.own`: for terminals this equals
`g.sum`; for non-terminals it captures the frozen baseline — the
V-entry's competitive weight in the tournament. `Cell` is a `Copy`
snapshot (ADR-M-008 addendum) — no lifetime, no pointer chase, same
cost as returning `Span`. The `Cell` view type is shared with
`get()` (which returns `Cell<C, V>` infallibly) for a consistent
API surface.

The `WeightedSampler` trait (ADR-M-009) exposes the same method for
capability-bounded generic code:

```rust
pub trait WeightedSampler: SpatialRead {
    fn sample(&self, rng: &mut impl Rng) -> Option<Cell<Self::Coord, Self::Accum>>;
}
```

**DC-019-2: Option A — return `None` when `I_total == 0`.**

When total intensity is zero the probability distribution is
undefined (division by zero in `c.int / v.int`). Returning `None` is
the honest answer: no intensity means no distribution to sample from.

This follows the `std::collections` convention (`BinaryHeap::peek`
returns `None` when empty). The check is cheap — an early return on
the V-root handle followed by a single intensity comparison:

```rust
let v_root = self.v_root?;
let root_node = self.vnodes.get(v_root.index());
if root_node.intensity == V::zero() {
    return None;
}
```

The tree always has at least one entry (the G-root is created with
a V-entry at construction), so in practice `v_root` is always
`Some` and `None` is only returned for the zero-intensity state.

## Consequences

- `sample()` returns `Option<Cell<C, V>>` — same `Cell` view type
  as `get()` (which is infallible), composable, `Copy`, no lifetime.
- Callers use standard `Option` combinators: `if let Some(cell) = ...`
- Zero-intensity trees return `None` rather than panicking or
  returning an arbitrary entry.
- The sampling walk produces a `Cell` by snapshotting the entry's
  backing G-node's fields (`lo`, `hi`, `own`) plus `depth`
  (computed from the interval) at the end of the descent — one
  construction per sample, no allocation.
- The landing entry is typically a terminal G-node (where
  `own == sum`), but may be a semi-internal or internal node
  whose intensity was frozen at splitting time.
- `sample()` is also available via the `WeightedSampler` trait
  (ADR-M-009) for capability-bounded generic code.
