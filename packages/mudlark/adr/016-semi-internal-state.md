# ADR-M-016: Semi-Internal State Representation

**Status:** Decided  
**Date:** 2026-02-25  
**Updated:** 2026-03-03  
**Amended by:** [ADR-M-027](027-observation-receiving-reframe.md) (flag
rename: `is_geo_terminal` → `is_exposed` / `is_evictable`)  
**API:** [§API M-4.1](../docs/api.md#41-spot-readings) (GState enum
under Spot readings)  
**Surface:** 1 (Prints)

## Context

A G-node exists in one of three states:

| State             | Left child             | Right child | Behaviour                                                        |
| ----------------- | ---------------------- | ----------- | ---------------------------------------------------------------- |
| **Terminal**      | `None`                 | `None`      | Leaf region — receives observations directly                     |
| **Semi-internal** | one `Some`, one `None` |             | One half routes to the child; the other accumulates on own entry |
| **Internal**      | `Some`                 | `Some`      | Both halves are covered by children                              |

Semi-internal nodes arise in eviction when an internal node has one
child evicted (§IDEA M-12.6); in split construction they appear
transiently when the first child is linked before the second.

The codebase infers node state entirely from `Option<GNodeId>`
checks on the `left` / `right` fields. This ADR decides whether to
add an explicit stored state field.

**Spec reference:** §IDEA M-4.1 (semi-internal definition, node
state table), §IDEA M-4.2 (`is_exposed`, `is_evictable` flags on V-Entry),
§IDEA M-4.3 (`has_evictable` flag on V-Structural), §IDEA M-12.3 (eviction
eligibility uses `has_dependents`), §IDEA M-12.4 (absorption produces
semi-internal state when one child is evicted), §IDEA M-12.5 Step 5 (flag
updates use `is_exposed`, `is_evictable`), §IDEA M-12.10 (semi-internal
chain analysis validates inferred state for chains of one-child
nodes).

---

## Analysis

### Q1 — How is state used?

Call sites fall into two categories:

**A. Individual binary questions** — the majority of call sites ask
one binary question ("has left child?", "is terminal?") and act
accordingly. These are well-served by the `Option` fields and the
convenience methods `is_terminal()` / `has_dependents()`.

| Module                      | Pattern                                                         | Purpose                                                                |
| --------------------------- | --------------------------------------------------------------- | ---------------------------------------------------------------------- |
| `gtree::route_to_receiver`  | `if let Some(left)` / `else if let Some(right)` / `else return` | Iterative descent — handles all 3 states uniformly via Option chaining |
| `gtree::recompute_g_sums`   | `g.left.map_or_else(V::zero, …)`                                | G-I1 recomputation — treats absent child as zero                       |
| `split::attempt_split`      | `g.left.is_some() \|\| g.right.is_some()`                       | Guard: only split terminal nodes                                       |
| `evict::evict_tip`          | `p.uncovered_range().is_some()`, `p.is_terminal()`              | Parent exposure/evictable state after child removal                    |
| `rebalance::legacy_promote` | `gn.is_semi_internal()`, `gn.left.is_none()`                    | Detect semi-internal, find missing side                                |
| `invariants::check_v_i6`    | `g.uncovered_range().is_some()`                                 | V-I6: verify `is_exposed` flag                                         |
| `invariants::check_v_i6b`   | `g.is_terminal()`                                               | V-I6b: verify `is_evictable` flag                                      |
| `invariants::check_g_i1`    | `g.left.map_or(…)`                                              | G-I1: sum = own + children                                             |
| `invariants::check_g_i2`    | `[g.left.is_some(), g.right.is_some()]`                         | G-I2: fanout ≤ 2                                                       |

**B. Three-way categorical matches** — several sites do switch on
the full `GState` via `g.state()`:

| Module                              | Pattern                                      | Purpose                                                          |
| ----------------------------------- | -------------------------------------------- | ---------------------------------------------------------------- |
| `graph.rs` (layers)                 | `match g.state()`                            | Classify G-node as Terminal / Transition for PEWEI extraction    |
| `graph.rs` (invariant display)      | `match g.state()`                            | Diagnostic label in error messages                               |
| `graph.rs` (contour depth)          | `match g.state()`                            | Uniform contour depth computation                                |
| `graph_plateau.rs` (basis elements) | `match g.state()`                            | Subtree basis element collection — different traversal per state |
| `graph_plateau.rs` (contour depth)  | `match g.state()`                            | Contour depth dispatch for plateau placement                     |
| `plateau.rs` (basis edge)           | `match g.state()`                            | Plateau basis walk — different edge computation per state        |
| `evict.rs` (plateau maintenance)    | `parent_state_after == GState::SemiInternal` | Plateau decomposition branching after eviction                   |
| `evict.rs` (contour depth)          | `match parent_state_after`                   | Contour depth dispatch for sorted placement                      |
| `evict.rs` (tests)                  | `pg.state()`                                 | Assert parent state after eviction                               |
| `invariants.rs`                     | `g.state()`                                  | Contour depth, state labels in diagnostics                       |

**Key observation:** Category B sites use the _computed_ `state()`
method — they never read a stored field. The `match` exhaustiveness
comes from the `GState` enum returned by the method, not from a
stored discriminant.

### Q2 — Does eviction need a stored enum?

Eviction eligibility ([ADR-M-013](013-eviction-eligibility.md)) checks:

1. `depth_V > D_evict` — a V-Tree property.
2. `is_evictable` — a V-Entry flag caching `g.is_terminal()`
   ([ADR-M-027](027-observation-receiving-reframe.md)).

The eviction scan traverses the **V-Tree**, not the G-Tree,
pruned by `has_evictable` flags on V-Structural nodes (V-I7).
Absorption ([ADR-M-014](014-value-absorption.md)) operates on the
parent's fields directly. No eviction code path benefits from a
stored G-node state enum.

Plateau maintenance after eviction snapshots
`(pg.state(), pg.lo, pg.hi)` before deallocation, then branches
on `GState::SemiInternal` to determine whether the surviving child
needs its own basis element. This is a computed read, not a stored
field — the snapshot captures the transient result.

Legacy promotion ([ADR-M-027](027-observation-receiving-reframe.md))
checks `is_semi_internal()` — a single pointer chase into the
backing G-node's child pointers. No scan, no flag, no stored
discriminant needed.

### Q3 — What is the size impact?

Empirically measured via `std::mem::size_of`:

| Variant                      |   `GNode<u64, u64>` |   `GNode<u32, u32>` |
| ---------------------------- | ------------------: | ------------------: |
| **Current** (no state field) |        **48 bytes** |        **32 bytes** |
| **+ 1-byte enum**            | **56 bytes** (+17%) | **36 bytes** (+13%) |
| **+ bool**                   | **56 bytes** (+17%) | **36 bytes** (+13%) |

Adding any stored field — even a single byte — triggers alignment
padding that pushes `GNode<u64, u64>` from exactly one 64-byte
cache line (48 ≤ 64) into a two-cache-line footprint on many
architectures, or at minimum wastes 8 bytes per node. This directly
violates the design target in [ADR-M-001](001-node-storage.md)
("target size: 48 bytes").

### Q4 — Readability concern

The strongest argument for a stored enum is readability. However:

1. Several functions already handle all three states cleanly via
   `match g.state()`, proving the computed helper is sufficient.
2. Most call sites only care about a **single** binary question
   ("has left child?", "is terminal?"), where a three-variant enum
   is overkill.
3. The computed `state()` method provides exhaustive matching at zero
   storage cost. Additional convenience methods (`is_terminal()`,
   `has_dependents()`, `is_semi_internal()`, `uncovered_range()`)
   cover every common binary question.

---

## Options Evaluated

### Option A — Inferred state (status quo + helper methods)

Keep `left: Option<GNodeId>` and `right: Option<GNodeId>` as the
sole source of truth. Provide zero-cost computed helpers:

```rust
/// The three categorical states of a G-node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum GState {
    /// Zero G-children. Leaf region — receives observations.
    Terminal,
    /// One G-child. One half routes to the child; the other
    /// accumulates on the node's own entry.
    SemiInternal,
    /// Two G-children. Both halves are covered.
    Internal,
}

impl<C, V> GNode<C, V> {
    /// Derive the categorical state from child pointers.
    #[inline]
    #[must_use]
    pub const fn state(&self) -> GState {
        match (self.left, self.right) {
            (None, None)       => GState::Terminal,
            (Some(_), Some(_)) => GState::Internal,
            _                  => GState::SemiInternal,
        }
    }

    /// True iff this node has zero G-children.
    #[inline]
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    /// True iff this node has at least one G-child.
    /// Inverse of `is_terminal()`.
    #[inline]
    #[must_use]
    pub const fn has_dependents(&self) -> bool {
        self.left.is_some() || self.right.is_some()
    }

    /// True iff exactly one G-child is present.
    #[inline]
    #[must_use]
    pub const fn is_semi_internal(&self) -> bool {
        matches!(self.state(), GState::SemiInternal)
    }

    /// Returns the uncovered half-interval, or `None` for internal
    /// nodes. Maps directly to §IDEA M-4.1 `uncovered_range(g)`.
    #[must_use]
    pub fn uncovered_range(&self) -> Option<(C, C)>
    where
        C: Coordinate,
    {
        match (self.left, self.right) {
            (None, None)       => Some((self.lo, self.hi)),
            (Some(_), None)    => { let mid = C::midpoint(self.lo, self.hi); Some((mid, self.hi)) }
            (None, Some(_))    => { let mid = C::midpoint(self.lo, self.hi); Some((self.lo, mid)) }
            (Some(_), Some(_)) => None,
        }
    }
}
```

**Pros:**

- Zero storage overhead — 48-byte target preserved
  ([ADR-M-001](001-node-storage.md)).
- Children fields remain the single source of truth — no
  synchronization invariant to maintain or violate.
- `state()` provides full three-way exhaustive matching for the
  sites that need it.
- `is_terminal()`, `has_dependents()`, `is_semi_internal()`, and
  `uncovered_range()` cover every common binary question.
- `GState` is a Surface 1 (Prints) type — serde-gated,
  `Copy`, returned in `Node<C, V>` snapshots.

**Cons:**

- Two Option checks per `state()` call instead of one enum read
  (negligible — both fit in a single loaded cache line).

### Option B — Stored `GState` enum field

Add `state: GState` to `GNode`.

**Pros:**

- Single field read to determine state.
- Exhaustiveness checking by compiler on `match`.

**Cons:**

- **Breaks 48-byte target** — grows to 56 bytes on `<u64, u64>`.
- Every `split`, `eviction`, `legacy_promote`, and future `merge`
  operation must update the field. Forgetting creates a silent
  invariant violation.
- Redundant with child pointers — the same information exists in
  two places, violating DRY and creating a new invariant to test.
- Would need an additional invariant check (e.g., "G-I9: state
  matches children") with associated test and maintenance cost.

### Option C — Replace `Option` children with indexed enum

Represent children as:

```rust
enum GChildren {
    Terminal,
    Left(GNodeId),
    Right(GNodeId),
    Both(GNodeId, GNodeId),
}
```

**Pros:**

- State and children unified — impossible to be inconsistent.
- Exhaustive matching.

**Cons:**

- **Breaks 48-byte target** — `GChildren` is 12 bytes
  (discriminant + max variant) vs. 8 bytes for 2 × `Option<GNodeId>`.
  Total becomes 52, rounded to 56 with alignment.
- Every child access requires destructuring instead of a direct
  field read. `route_to_receiver` becomes more verbose, not less.
- Niche optimisation for `Option<NonZeroU32>` is lost inside the
  enum variants, so the handles no longer pack for free.

---

## Decision

**Option A — Inferred state with computed helper methods.**

Do not add a stored state field to `GNode`. The `GState` enum
and computed methods live in `gnode.rs`.

### Rationale

1. **Cache-line preservation.** The 48-byte size of
   `GNode<u64, u64>` is a deliberate design target
   ([ADR-M-001](001-node-storage.md)). Adding a stored field pushes
   the struct to 56 bytes — a 17% increase that wastes 8 bytes per
   node and may cross a cache-line boundary on some architectures.

2. **Single source of truth.** The `left` / `right` Option fields
   already encode state perfectly. A stored enum duplicates this
   information and creates a new synchronization invariant. Every
   mutation site (split, eviction, legacy promotion) would need to
   update the field — a class of bug that is entirely preventable
   by not storing the field.

3. **No consumer needs a stored enum.** Eviction eligibility is
   cached in V-Entry flags (`is_evictable`, `is_exposed` —
   [ADR-M-027](027-observation-receiving-reframe.md)) and never
   reads G-node categorical state as a discriminant. Legacy
   promotion checks `is_semi_internal()` — a trivial computed
   method. The PEWEI extractor, plateau walker, and plateau
   maintenance code all use `g.state()` successfully.

4. **Computed helpers are sufficient.** The `state()` method
   provides identical readability for three-way matches. The
   convenience methods (`is_terminal()`, `has_dependents()`,
   `is_semi_internal()`, `uncovered_range()`) cover every common
   binary question. All are `#[inline]` / `const fn` and trivially
   cheap.

---

## Consequences

- **`gnode.rs`:** `GState` enum and five computed methods
  (`state()`, `is_terminal()`, `has_dependents()`,
  `is_semi_internal()`, `uncovered_range()`). No stored field.
  `GState` is serde-gated and `Copy` — a Surface 1 type.
- **`view.rs`:** `Node<C, V>` carries `state: GState` — a snapshot
  value computed at extraction time, not a live stored field.
- **`split.rs`:** Terminal guard uses `g.left.is_some() ||
g.right.is_some()` (equivalently `g.has_dependents()`). Both
  forms are clear.
- **`evict.rs`:** Parent state after absorption checked via
  `p.uncovered_range().is_some()` and `p.is_terminal()` (binary
  questions). Plateau maintenance snapshots `pg.state()` before
  deallocation and branches on `GState::SemiInternal` to determine
  surviving-child basis membership. Contour depth dispatch uses
  `match parent_state_after` for sorted placement. No stored state
  to update.
- **`rebalance.rs`:** Legacy promotion checks
  `g.is_semi_internal()` and `g.left.is_none()` to find the
  missing side. No stored state needed.
- **`invariants.rs`:** V-I6 uses `g.uncovered_range().is_some()`;
  V-I6b uses `g.is_terminal()`. Diagnostic messages use
  `g.state()` for labelling. Contour depth validation uses
  `match g.state()`.
- **`graph.rs`:** Uses `match g.state()` for PEWEI layer
  classification (Terminal / Transition) and uniform contour depth
  computation.
- **`graph_plateau.rs`:** Uses `match g.state()` for subtree basis
  element collection (different traversal per state), contour depth
  dispatch for plateau placement, and sorted placement helpers.
- **`plateau.rs`:** Uses `match g.state()` for basis edge
  computation — different edge per state.
- **No new invariant needed.** Since no state is stored, there is
  nothing to fall out of sync.
- **Size tests unchanged.** The `gnode_size` test asserts 48 bytes
  for `GNode<u64, u64>`.
- **Validated by §IDEA M-12.10 (semi-internal chain overhead).** Chapter
  12's analysis of semi-internal chains — sequences of one-child
  G-nodes — relies on the inferred-state predicates defined in §IDEA
  4.1. No chain operation requires or would benefit from a stored
  state discriminant, confirming the inferred approach handles the
  most structurally complex G-Tree configurations.
