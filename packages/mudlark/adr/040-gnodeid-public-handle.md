# ADR-M-040: GNodeId — Generational Public Handle

**Status:** Accepted  
**Date:** 2026-03-27  
**Relates to:** [ADR-M-001](001-node-storage.md) (arena storage with
free-list reuse), [ADR-M-032](032-three-surface-model.md) (three-surface
model, handle test), [ADR-M-036](036-sentinel-integration-api.md)
(sentinel integration — `Node.gnode_id`, `BasisElement.gnode_id`)  
**Spec:** §API M-4.5 (handles)  
**Surface:** 1 (Prints) + 3 (Emulsion)

---

## Context

### The stale-handle problem

The G-node arena (`Arena<GNode<C, V>>`) reuses slots via a free list.
When a node is evicted, its slot index returns to the free list and
may be allocated to an entirely different node. A `GSlotPointer`
obtained before the eviction now silently refers to the new occupant —
a logical use-after-free.

Today the only guard is the occupancy bitmap: `gnode_info()` and
`is_ancestor_of()` return `None`/`false` for freed slots. But the
bitmap cannot detect the **ABA problem** — when the slot is freed
*and then reallocated*, the bitmap shows "occupied" and the stale
handle passes validation, silently binding to a semantically
unrelated node.

This is not hypothetical. The sentinel (ADR-M-036) stores
`GSlotPointer` values as map keys (`BTreeMap<GSlotPointer, CellState>`)
across observation cycles. An evict-then-split sequence within a
single cycle can recycle a slot, causing the sentinel to attach old
tracker state to a new, unrelated cell. The bug is silent: no panic,
no `None`, just wrong analysis.

### Naming

`GSlotPointer` exposes implementation vocabulary — "slot" and
"pointer" — that belongs to the arena (Surface 3). Public-facing
code should name the concept it identifies (a G-node), not the
mechanism by which it is located (a slot pointer). The rename to
`GNodeId` improves both clarity and grep-ability without changing
any semantics.

### Scope of exposure

`GSlotPointer` currently appears in the public API in these positions:

| Position                           | Produces / Consumes |
| ---------------------------------- | ------------------- |
| `GvGraph::g_root()`                | produces            |
| `Node.gnode_id`                    | produces            |
| `Node.parent`                      | produces            |
| `BasisElement.gnode_id`            | produces            |
| `TemporalDecay::decay(root, …)`   | consumes            |
| `GvGraph::gnode_info(id)`          | consumes            |
| `GvGraph::gnode_children(id)`      | consumes            |
| `GvGraph::is_ancestor_of(a, b)`   | consumes            |

The `#[doc(hidden)]` methods `from_index()` and `index()` additionally
permit round-tripping raw indices.

---

## Decision

### D1 — Introduce `GNodeId` as the public handle type

A new `GNodeId` type replaces `GSlotPointer` on all public API
surfaces. It carries both the slot index and a **generation counter**
(slot iteration count), so stale handles are detectable.

```rust
/// Opaque identity token for a G-node.
///
/// Carries sufficient information to detect use of a stale handle
/// after the underlying arena slot has been freed and reallocated.
/// Any consuming method will panic with a diagnostic message if
/// the generation does not match the slot's current generation.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GNodeId {
    index: NonZeroU32,      // 1-indexed arena slot (same encoding as today)
    generation: u32,        // slot iteration count at creation time
}
```

**Size:** 8 bytes. `Option<GNodeId>` is also 8 bytes
(niche-optimised via the `NonZeroU32` index field). This is a cost
over today's 4-byte `GSlotPointer` / 4-byte `Option<GSlotPointer>`,
but `GNodeId` appears in snapshot types (`Node`, `BasisElement`) that
are already 40+ bytes, so the relative impact is small. The hot
internal paths continue to use the 4-byte `GSlotPointer` (see D3).

**Trait impls:** `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`,
`PartialOrd`, `Ord`, `Hash`, conditional `serde`. Same programme as
the current `GSlotPointer` public surface.

### D2 — Arena stores per-slot generation counters

`Arena<T>` gains a parallel `Vec<u32>` of generation counters. On
`alloc()`, the counter for the slot is read (defaulting to 0 for
newly grown slots) and included in the returned handle. On
`dealloc()`, the counter is incremented. A subsequent `alloc()` into
the same slot reads the incremented value, producing a `GNodeId` with
a higher generation than any previously issued handle for that slot.

```rust
pub struct Arena<T: Default> {
    slots: Vec<T>,
    occupied: Vec<u64>,
    free: Vec<u32>,
    count: u32,
    generations: Vec<u32>,  // NEW — one counter per slot
}
```

The generation `Vec` is cold data — touched only on alloc/dealloc,
never on the read/write hot path. Cache impact: negligible (same
access pattern as the existing `occupied` bitset).

**Overflow:** `u32` gives ~4 billion reuses per slot. At a
hypothetical worst case of 1 million evict-realloc cycles per second
on a single slot, saturation takes ~71 minutes. This is far
beyond any realistic workload. If the generation counter wraps
(which should not happen in practice), it wraps silently via
`wrapping_add(1)`. The probability of an ABA collision from a
wrapped counter is astronomically low.

### D3 — `GSlotPointer` becomes `pub(crate)` internal handle

`GSlotPointer` continues to exist as the generation-free, 4-byte
internal handle for intra-arena operations where the generation check
is unnecessary overhead (e.g., tree walks within a single
`observe()` call where no eviction can interleave). It is demoted
from `pub` to `pub(crate)`.

`GNodeId` provides a method to extract its `GSlotPointer` for
internal use:

```rust
impl GNodeId {
    /// Internal: extract the raw slot pointer (no generation).
    pub(crate) fn slot(self) -> GSlotPointer { … }
}
```

### D4 — Consuming methods validate the generation

Every public method that accepts a `GNodeId` extracts the slot index
and checks the generation against the arena's current generation for
that slot. On mismatch:

- **Debug builds:** panic with a diagnostic message including the
  handle's generation, the slot's current generation, and the slot
  index.
- **Release builds:** panic. Stale-handle use is a logic error,
  not a recoverable condition. Silent misbehaviour (the status quo)
  is strictly worse.

Methods affected: `decay()`, `gnode_info()`, `gnode_children()`,
`is_ancestor_of()`.

For `gnode_info()` and `is_ancestor_of()`, which currently return
`Option`/`bool` for freed slots, the behaviour tightens. A freed slot
with matching generation returns `None`/`false` (the node was freed
but the handle is not stale — it correctly identifies a node that no
longer exists). A generation mismatch panics.

### D5 — `from_index()` / `index()` become generation-aware

The existing `#[doc(hidden)]` `from_index()`/`index()` pair on
`GSlotPointer` served serialisation and diagnostics. On `GNodeId`:

- `index()` returns the 0-based slot index (same as today).
- `generation()` returns the generation counter.
- `from_parts(index: usize, generation: u32)` reconstitutes a handle.
  This replaces `from_index()` and makes the generation an explicit
  input — callers cannot accidentally omit it.

All three are `#[doc(hidden)]` and carry the same serialisation
caveat as today: a reconstituted handle is only valid against the
graph instance that produced it.

### D6 — Clean break: `GSlotPointer` becomes `pub(crate)` immediately

The rename `GSlotPointer` → `GNodeId` on all public surfaces is a
breaking change. `GSlotPointer` is demoted to `pub(crate)` in the
same commit — no deprecated type alias, no transition period. All
external code must switch to `GNodeId` at once.

**Rationale:** mudlark has no downstream consumers outside the
torrust-index workspace. A deprecated alias adds code and cognitive
overhead for zero practical benefit. A clean break keeps the public
API surface minimal and avoids a state where two names for
conceptually different types (one generational, one not) coexist in
the public namespace.

Serde representation: `GNodeId` serialises as a struct with `index`
and `generation` fields (or a two-element tuple — to be decided at
implementation time). This is a breaking change to the serialised
format, which is acceptable because handle serde was explicitly
documented as unstable (§API M-4.5 serde note).

---

## Consequences

### Positive

- **Stale-handle detection.** The ABA problem is eliminated. Any use
  of a recycled slot with an old handle panics immediately with a
  diagnostic message rather than silently operating on the wrong node.

- **Sentinel correctness.** `BTreeMap<GNodeId, CellState>` in the
  sentinel cannot silently bind old tracker state to a new node.
  Either the handle still refers to the original node, or the
  consuming method panics, forcing the sentinel to clean up its
  tracker map.

- **Name clarity.** `GNodeId` names the concept ("G-node identity"),
  not the mechanism ("slot pointer"). Public API reads better;
  internal code retains `GSlotPointer` where it belongs.

- **Internal hot paths unchanged.** `GSlotPointer` remains 4 bytes
  for all intra-crate tree walks. The 8-byte `GNodeId` exists only
  at the API boundary.

### Negative

- **Snapshot type growth.** `Node` gains 4 bytes (generation in
  `gnode_id`) and 4 bytes (generation in `parent`). `BasisElement`
  gains 4 bytes. This is ~10–20% growth on types that are already
  40+ bytes and are not stored in bulk.

- **Option size growth.** `Option<GNodeId>` is 8 bytes
  (niche-optimised via `NonZeroU32`) vs today's 4-byte
  `Option<GSlotPointer>`. Affects `Node.parent` in the public
  projection. The internal fields continue to use
  `Option<GSlotPointer>` (4 bytes, niche-optimised).

- **Arena generation `Vec`.** One `u32` per slot. For a typical
  graph with ~1,000 slots, this is 4 KB of cold memory.

- **Breaking change.** Public type rename + serde format change.
  Clean break — no transition alias.

### Neutral

- **`from_parts()`** replaces `from_index()`. Slightly more ceremony
  for diagnostic tooling, but prevents the dangerous pattern of
  reconstituting a handle without a generation.

---

## Alternatives Considered

### A1 — Generation counter inside `GSlotPointer` (no separate type)

Pack index + generation into a single `u64` (or keep `NonZeroU32`
for the index and add a `u16` generation). This avoids having two
handle types but either doubles the handle size everywhere (including
internal hot paths) or limits the generation counter to 16 bits
(65 K reuses before wrap). Rejected because the two-type design
(D3) keeps internal paths lean while providing full 32-bit
generations at the boundary.

### A2 — Epoch-based invalidation

Maintain a global epoch counter on `GvGraph`, bumped on every mutation.
Handles carry the epoch. Any handle from a previous epoch is
rejected. This is simpler than per-slot generations but
**over-invalidates**: a handle obtained before an unrelated mutation
(e.g., an `observe()` that doesn't evict anything) would be rejected
even though the slot is unchanged. This makes handles almost useless
across `observe()` calls, which is exactly how the sentinel needs
them.

### A3 — Do nothing; document the hazard

Document that `GSlotPointer` may alias after eviction and that callers
must re-obtain handles after any mutation. This is the status quo for
the ABA case. It shifts the burden to every consumer, is easy to get
wrong silently, and produces bugs that are extremely difficult to
diagnose. Rejected.

### A4 — Never reuse slots (monotonic allocation)

Remove the free list; never dealloc. Stale handles would point to
default-initialised slots that fail the occupancy check. This
eliminates ABA but wastes memory proportionally to the total number
of nodes ever created, which for long-running processes with
aggressive eviction can be unbounded. The free-list design exists
specifically to bound memory usage. Rejected.
