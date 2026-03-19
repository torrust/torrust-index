# ADR-M-008: Span Type

**Status:** Decided  
**Date:** 2026-02-24  
**Updated:** 2026-03-03  
**Relates to:** [ADR-M-016](016-semi-internal-state.md) (GState enum),
[ADR-M-021](021-pewei-output-representation.md) (PEWEI output),
[ADR-M-025](025-public-api-surface.md) (public API surface — uses
Cell / Node in method returns),
[ADR-M-032](032-three-surface-model.md) (three-surface model)  
**Spec:** §IDEA M-4.1 (G-Node fields), §IDEA M-5.3 (accounting: own vs
sum), §PEWEI M-2 (layer populations — transition vs. terminal)  
**API:** [§API M-4.1](../docs/api.md#41-spot-readings) (Span, Cell,
Node — spot readings)  
**Surface:** 1 (Prints)

## Context

Queries and iteration need a lightweight type representing a dyadic
interval paired with its accumulated intensity. This appears in point
queries, range queries, iteration, and Pewei extraction. The question
is whether to use an enum (distinguishing terminal vs. internal) or a
single generic struct.

## Decision

Single generic struct `Span<C, V>` — no enum variants.

```rust
/// Owned dyadic interval + single intensity. Purely geometric —
/// no node identity or tree state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Span<C: Coordinate, V: Accumulator> {
    pub start: C,        // l
    pub end: C,          // r  (half-open [start, end))
    pub intensity: V,    // accumulated value
    pub depth: u32,      // G-Tree depth
}
```

Methods: `width() -> C`.

Finality checking (`is_final`) lives on `Cell`, not `Span` — a `Span`
is a bare interval + intensity with no opinion on termination. For
integer coordinates, `Cell::is_final()` reduces to `width() == 1`;
for float coordinates it checks `depth >= N`.

## Rationale

- An enum would distinguish terminal from internal nodes at the type
  level, but `Span` is a **value type** — it represents a measurement,
  not a node. The caller decides whether they care about node state
  (via `Cell` or `Node` view types instead).
- Generic over both `C` and `V` — the same span type works across
  all tree instantiations.
- `depth` field is always available — cheap to carry (4 bytes) and
  needed for float coordinate finality checks.

## Consequences

- One type, not an enum — simpler API surface.
- `Cell` and `Node` view types delegate to `Span` for the interval +
  intensity, adding their own metadata (V-depth, terminal status, etc.).

---

## Addendum: View Types — Cell and Node (Phase 4)

**Status:** Decided  
**Date:** 2026-02-26  
**Phase:** 4 (Output)

### Context

The decision above established `Span<C, V>` as an owned value type.
The Consequences note mentions `Cell` and `Node` view types but does
not define them. Output methods — `sample` (Phase 4), `get()` and
`layers()` (Phase 5, ADR-M-025) — return views from the graph. Two
questions:

### Q1: View type shape (DC-008-1)

What should borrowed view types look like?

| Option | Design                                                                                        | Notes                                                                                                                                     |
| ------ | --------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- |
| A      | `Cell<'a, C, V>` borrows `&'a GvGraph` + `GNodeId` — deref to fields lazily                   | Zero-copy, minimal struct (pointer + handle). But every field access goes through a pointer chase.                                        |
| B      | `Cell<'a, C, V>` contains `start: C, end: C, intensity: V, depth: u32` — snapshot at creation | Copied on construction. Self-contained. No borrow escape hazards. Can outlive the graph if `'a` is removed (but then it's just a `Span`). |
| C      | `Cell<'a, C, V> = &'a Span<C, V>` — `Cell` is a type alias for a reference                    | Simplest. No new struct. But can't add Cell-specific methods without a newtype.                                                           |
| D      | `Cell<'a, C, V>` borrows individual fields: `start: &'a C, end: &'a C, intensity: &'a V`      | Fine-grained borrows. Unusual in Rust — typically borrow-of-struct patterns use a single reference.                                       |

**Considerations:**

- `C` and `V` are `Copy` types (integers, floats). Copying them into
  a small struct (Option B) is cheaper than a pointer chase (Option A)
  for small values.
- `Cell` should support `cell.to_span() -> Span<C, V>` for ownership
  transfer regardless of option chosen.
- `Node` (an internal G-Tree node) could follow the same pattern as
  Cell, adding `children: (GNodeId, GNodeId)` for traversal. Or it
  can be deferred — most API surfaces only expose terminal cells.
- If `Cell` is just a `Span` with a lifetime, Option C is simplest
  and avoids type proliferation.

### Q2: When to introduce Node (DC-008-2)

`Node` is a view-type for internal G-nodes (non-terminal). It's
useful for traversal, inspection, and debugging but not needed for
the core Phase 4 API (`sample`, `cells()`, `range_sum`).

| Option | Timing                                             | Notes                                                                                 |
| ------ | -------------------------------------------------- | ------------------------------------------------------------------------------------- |
| A      | Phase 4: introduce both `Cell` and `Node` together | Complete view-type API. `layers()` may want `Node` for internal nodes.                |
| B      | Phase 4: `Cell` only; `Node` in Phase 5            | Keeps Phase 4 lean. `layers()` can yield `Span` for internal nodes.                   |
| C      | Never: use `Span` everywhere, no `Node` type       | Maximally simple. Callers can't distinguish terminal from internal at the type level. |

### Decision

**DC-008-1: Option B — snapshot struct.**

`Cell` and `Node` are small `Copy` structs whose fields are copied
from the arena at construction time. Both `C: Coordinate` and
`V: Accumulator` require `Copy`, so snapshotting 4–6 fields
(~24–40 bytes for `<u64, u64>`) is cheaper than a pointer chase
through the arena on every access.

```rust
/// Snapshot of a terminal G-node (leaf cell).
///
/// Fields are copied at construction — no pointer chase.
/// Convertible to the owned `Span` via `to_span()`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cell<C: Coordinate, V: Accumulator> {
    pub start: C,        // l
    pub end: C,          // r  (half-open [start, end))
    pub intensity: V,    // g.own (= g.sum for terminals)
    pub depth: u32,      // G-Tree depth
}

/// Snapshot of any G-node (terminal, semi-internal, or internal).
///
/// Fields are copied at construction — no pointer chase.
/// Carries both `own` and `sum` plus the categorical state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Node<C: Coordinate, V: Accumulator> {
    pub start: C,        // l
    pub end: C,          // r  (half-open [start, end))
    pub own: V,          // g.own — direct accumulation / baseline
    pub sum: V,          // g.sum — total including children
    pub depth: u32,      // G-Tree depth
    pub state: GState,   // Terminal / SemiInternal / Internal
}
```

Neither type carries a lifetime parameter nor a graph reference.
They are _produced_ from a reference to the graph (snapshot at
construction), not held by one.

Convenience methods on both types:

- `Cell::to_span() -> Span<C, V>` — trivial field copy.
- `Cell::width() -> C` — `end - start`.
- `Cell::is_final(n: u32) -> bool` — integer: `width() == 1`;
  float: `depth == n`.
- `Node::to_span() -> Span<C, V>` — uses `self.sum` as the span
  intensity. For terminals, `sum == own` so this is lossless. For
  internal/semi-internal nodes, `sum` captures the total energy
  over the region — the natural value for a `Span`.
- `Node::to_cell() -> Option<Cell<C, V>>` — succeeds only when
  `state == Terminal`.
- `Node::is_terminal() -> bool`.
- `Node::refinement() -> V` — `sum - own` (derived; only meaningful
  for internal / semi-internal nodes).
- `Node::width() -> C` — `end - start`.

**DC-008-2: Option A — introduce both `Cell` and `Node` in Phase 4.**

Rationale:

1. PEWEI extraction (deliverable #3) classifies every V-entry as
   either a **phase-transition node** (internal G-node) or a
   **terminal node** (leaf G-node). The phase-transition nodes carry
   `own` (baseline), `sum` (total), and derived `refinement` / SNR —
   exactly the fields on `Node`. Introducing `Node` now gives
   `extract()` and `layers()` a typed representation instead of
   falling back to `Span` (which only carries a single intensity).

2. The cost is small: `Node` is 6 fields, no lifetime, no graph
   reference. It follows the identical snapshot pattern as `Cell`.

3. Having both types ready avoids a Phase 5 refactor where every
   consumer of `layers()` / PEWEI output would need to migrate from
   `Span` to `Node`.

### Consequences

- Two new public types (`Cell`, `Node`) plus the existing `Span`.
  Clear responsibility split:
  - `Cell` — terminal G-nodes (leaf measurement).
  - `Node` — any G-node (carries both `own` and `sum`).
  - `Span` — owned value type (interval + single intensity).

- `Cell` is a strict subset of `Node` (every terminal can be viewed
  as a `Node`). Provide `Node::to_cell() -> Option<Cell>` that
  succeeds only when `state == Terminal`.

- No lifetime parameter means `Cell` and `Node` are `Send + Sync`
  automatically and can be stored in collections without borrowing
  the graph.

- `layers()` yields `(usize, Node)` pairs (layer index + snapshot),
  giving callers type-level access to `state`, `own`, `sum`, and
  `refinement()` without downcasting or matching on an enum.

- PEWEI output types (`Transition`, `Terminal`) can be
  constructed directly from `Node` and `Cell` snapshots respectively.
