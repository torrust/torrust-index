# ADR-M-027: Exposed / Evictable Flag Reframe and Legacy Promotion

**Status:** Decided  
**Date:** 2026-02-28  
**Supersedes:** Partially amends ADR-M-013, ADR-M-016  
**Spec references:** §IDEA M-1.6.2 (dual shielding), §IDEA M-4.1 (node
states), §IDEA M-4.2 (V-Entry `is_exposed`, `is_evictable`), §IDEA M-4.3
(V-Structural `has_evictable`), §IDEA M-9.3 (evictable flag propagation),
§IDEA M-10.4 (split violation-freedom), §IDEA M-11.6 (legacy promotion), §IDEA M-11.7
(decision table), §IDEA M-11.9 (resolve dispatcher), §IDEA M-12.3 (eviction
eligibility), §IDEA M-13.2 (G-node lifecycle), §IDEA M-13.3 (governance flow),
§IDEA M-13.5 (benchmark compounding)  
**Surface:** 3 (Emulsion)

## Context

The spec (idea.md) defines two orthogonal properties for G-nodes:

| Property                            | Terminal (0 children) | Semi-internal (1 child) | Internal (2 children) |
| ----------------------------------- | --------------------- | ----------------------- | --------------------- |
| **Exposed** (has uncovered range)   | Yes — entire range    | Yes — uncovered half    | No — fully shielded   |
| **Has dependents** (has G-children) | No                    | Yes                     | Yes                   |

These govern different concerns:

- **Exposed** governs observation routing (does the node
  accumulate?), V-entry liveness (is the entry growing or frozen?),
  and contour membership.
- **Has dependents** governs eviction safety (can the node be
  removed without orphaning descendants?).

The spec names the corresponding V-Tree flags `is_exposed` (§IDEA M-4.2)
and `has_evictable` (§IDEA M-4.3). The codebase currently uses
`is_geo_terminal` and `has_geo_terminal` — names that conflate
"exposed to observations" with "zero children", which are the same
thing only for terminal nodes.

### The semi-internal problem

A semi-internal node is exposed AND has dependents. The current
`is_geo_terminal` flag is false for semi-internals, which is
correct for eviction safety but wrong for contour membership. This
conflation creates a stuck scenario: a hot region that lost one
child to eviction cannot regrow that child. The only recovery path
is for the surviving sibling to also be evicted, collapsing the
parent back to terminal, then re-splitting from scratch — destroying
the surviving sibling's accumulated V-Tree position.

### The spec's solution: legacy promotion

The spec resolves this not by broadening the split guard, but
through a completely different mechanism: **legacy promotion**
(§IDEA M-11.6). Splitting remains terminal-only (fully exposed, no
dependents). Instead, a semi-internal node regrows its missing child
through the V-Tree's competitive rebalancing:

1. A semi-internal node receives observations in its uncovered half.
2. Its V-entry intensity grows.
3. Eventually it outweighs every uncle — a V-I3 violation.
4. The rebalance dispatcher (§IDEA M-11.9) reaches the skip-promote branch,
   checks `is_semi_internal(c.gnode)` and `depth_V(c) ≤ D_evict`,
   and dispatches to `legacy_promote`.
5. Legacy promote lifts the entry to the grandparent level AND
   creates the missing G-child, depositing a new zero-intensity
   entry in the vacated V-seat.

**The depth gate is an implementation optimisation** (§IDEA M-11.6): an
heir created past $D_{\text{evict}}$ would be immediately
eviction-eligible, triggering a futile allocate–evict round-trip to
the same end state. Suppressing the creation avoids the transient
node churn. When `depth_V(c) > D_evict`, the dispatcher falls
through to `skip_promote` instead — the semi-internal node rises
bodily without creating an heir. The competitive mechanism remains
the primary gate; the depth check is a cost filter.

### Current flag usage in the codebase

| Current name               | Used by                                            | Purpose                                                    |
| -------------------------- | -------------------------------------------------- | ---------------------------------------------------------- |
| `is_geo_terminal`          | `evict.rs` (eligibility)                           | Only evict nodes with no dependents                        |
| `is_geo_terminal`          | `split.rs` (flag update)                           | Parent entry goes non-exposed, non-evictable after split   |
| `is_geo_terminal`          | `evict.rs` (parent state update)                   | Track whether parent becomes terminal after child eviction |
| `has_geo_terminal`         | `evict.rs` (scan pruning)                          | Skip V-subtrees with nothing evictable                     |
| `has_geo_terminal`         | `rebalance.rs` (contraction)                       | Propagate structural flags after restructuring             |
| `propagate_terminal_flags` | `vtree.rs`, `split.rs`, `evict.rs`, `rebalance.rs` | Bottom-up flag propagation                                 |

---

## Analysis

### The two flags serve different audiences

The spec (§IDEA M-4.3) explains the asymmetry clearly:

> `is_exposed` tracks whether the entry receives observations
> (contour membership — governs refinement and restoration
> eligibility). `is_evictable` on entries caches whether the backing
> G-node has zero children (governs eviction eligibility).
> `has_evictable` on structural nodes tracks whether anything in the
> subtree is unprotected and can be safely deallocated (governs
> eviction scanning). These serve different purposes.

**`is_exposed`** answers: "Is this entry's backing G-node on the
contour — does it receive observations?" True for terminals AND
semi-internals. False for internals (both children present).

**`has_evictable`** answers: "Does this V-subtree contain any entry
whose backing G-node has zero children and can be safely removed?"
This is the eviction scan's pruning flag.

Note: the spec explicitly explains why there is **no flag for legacy
promotion** (§IDEA M-4.3): legacy promotion has no scan — it piggybacks on
V-I3 violation resolution, which naturally visits exactly the entry
that just promoted. A single pointer chase into `c.gnode` determines
semi-internal status. No subtree aggregate needed.

### Legacy promotion mechanics

From §IDEA M-11.6:

```
function legacy_promote(c):
    p ← c.val_parent
    g ← p.val_parent
    s ← sibling(c, p)
    u ← sibling(p, g)
    gnode ← c.gnode

    // G-Tree: create the missing child
    exposed ← uncovered_range(gnode)
    new_child ← new G-Node(exposed, sum = 0, own = 0)
    attach new_child to gnode's empty slot

    // V-Tree: create entry for new child (the heir)
    ne ← new V-Entry(int = 0, gnode = new_child,
                      is_exposed = true, is_evictable = true)

    // V-Tree: lift c to g, place ne in c's vacated seat
    replace c with ne in p.children
    ne.val_parent ← p

    g.children ← [c, p, u]
    c.val_parent ← g

    // Freeze: gnode now has 2 G-children, above the contour
    c.is_exposed ← false
```

**Dispatch condition (§IDEA M-11.7, §IDEA M-11.9).** The resolve dispatcher
reaches this function only when `c` is an entry backing a
semi-internal G-node AND `depth_V(c) ≤ D_evict`. When
`depth_V(c) > D_evict`, the dispatcher uses `skip_promote` instead
— the heir would be immediately eviction-eligible, so its creation
is suppressed.

The result is identical to what catalytic split produces — the
parent's entry becomes uncle to its children's entries — but the
trigger is competitive promotion, not a split threshold. The new
child starts at zero intensity and must earn its way up. V-I3 is
trivially preserved: adding a zero-intensity entry cannot weaken
any uncle shield (same argument as §IDEA M-10.4).

### The G-node lifecycle with legacy promotion

From §IDEA M-13.2:

```
    ON THE CONTOUR — fully exposed (no dependents)
        │
        │ catalytic split → 2 children created
        ▼
    ABOVE THE CONTOUR (not exposed, has dependents, frozen benchmark)
        │                              ↑
        │ one child evicted            │
        │ (absorb, partially exposed)  │
        ▼                              │
    ON THE CONTOUR — partially exposed (has one dependent)
        │                              │
        ├─── other child evicted ──► ON THE CONTOUR — fully exposed
        │         (absorb,                 (no dependents, cycle restarts)
        │          no dependents)
        │
        └─── legacy promotion ─────► ABOVE THE CONTOUR
               (entry outgrew              (not exposed, has dependents,
                all uncles)                 frozen benchmark, cycle continues)
```

The stuck scenario is resolved: a semi-internal node regrows its
evicted half through competitive promotion — no separate operation,
no separate threshold.

### Impact on each flag

| Current name               | New name                    | Semantic change                                                                           |
| -------------------------- | --------------------------- | ----------------------------------------------------------------------------------------- |
| `is_geo_terminal`          | `is_exposed`                | **Broadened (§IDEA M-4.2):** true for terminal AND semi-internal (any node with uncovered range) |
| (new)                      | `is_evictable`              | **New (§IDEA M-4.2):** caches `¬has_dependents(gnode)`, true for terminal only                   |
| `has_geo_terminal`         | `has_evictable`             | **Renamed (§IDEA M-4.3):** true iff any descendant entry backs a 0-child G-node                  |
| `propagate_terminal_flags` | `propagate_evictable_flags` | Renamed to match                                                                          |

The critical change is `is_exposed`. Currently `is_geo_terminal` is
true only for 0-child nodes. The spec's `is_exposed` is true for
any node on the contour (terminal or semi-internal). This flag
change is a prerequisite for legacy promotion: the rebalance
dispatcher needs to know whether an entry is exposed (to determine
if the V-I3 violation should trigger legacy promote vs. skip
promote).

### Why `has_evictable` does not simply mirror `is_exposed`

`is_exposed` is true for terminals AND semi-internals. But
semi-internals are NOT evictable (they have dependents). So
`has_evictable` must check the stricter condition: entry backs a
G-node with zero children. The propagation formula (§IDEA M-9.3) reads
`is_evictable` from entry children — the cached terminal check.
Semi-internal entries are exposed but not evictable (they have a
surviving child). The eviction scan resolves final eligibility at
the leaf with `¬has_dependents(v.gnode)`.

This is the same check as the current `has_geo_terminal`, but the
flag's **name** and **documentation** are updated to reflect its
actual purpose (eviction scan pruning), not the node state it
happens to correlate with in the terminal-only world.

### The `is_evictable` cache (§IDEA M-4.2)

The spec defines `is_evictable: bool` on V-Entry (§IDEA M-4.2) — a cached
answer to `¬has_dependents(v.gnode)`. It is a spec-defined field
that serves as an implementation optimization: propagation and the
eviction scan read a local bool instead of chasing into gnodes.

`is_evictable` is true iff the backing G-node has zero children
(terminal only). Three mutation sites maintain it: catalytic split,
eviction, and legacy promotion. A debug invariant (V-I6b) validates
it at every checkpoint. Zero size cost — Entry is the smaller enum
variant.

---

## Options Considered

### Option A — Status quo (no renames, no legacy promotion)

Keep `is_geo_terminal` / `has_geo_terminal`. Semi-internal nodes
remain stuck. Recovery requires full contraction before re-expansion.

**Pros:** No code changes.  
**Cons:** Stuck semi-internal scenario persists. Flag names conflate
two orthogonal properties. Blocks implementation of legacy promotion.

### Option B — Rename flags only (no legacy promotion yet)

Rename `is_geo_terminal` → `is_exposed`, `has_geo_terminal` →
`has_evictable`, `propagate_terminal_flags` →
`propagate_evictable_flags`. Broaden `is_exposed` to be true for
semi-internals. Do not implement legacy promotion yet.

**Pros:** Aligns naming with spec. Separates the two concerns.
Unblocks future legacy promotion implementation.  
**Cons:** The stuck scenario persists until legacy promotion is
implemented. The `is_exposed` broadening is inert without a consumer.

### Option C — Full reframe: rename flags + implement legacy promotion (proposed)

Rename flags per Option B. Implement `legacy_promote` in the
rebalance dispatcher (§IDEA M-11.9). The split guard remains terminal-only.
The rebalance dispatcher checks `is_semi_internal(c.gnode)` and
`depth_V(c) ≤ D_evict` at the skip-promote branch and dispatches
to `legacy_promote` when both hold.

**Pros:**

- Aligns with the spec completely.
- Cleanly separates "exposed" (contour membership) from "evictable"
  (structural safety).
- Semi-internal nodes regrow their missing child through competition.
- Surviving sibling's V-position is preserved.
- The depth gate (§IDEA M-11.6) suppresses futile creations — heirs that
  would be immediately eviction-eligible are never created.
- Benchmark compounding (§IDEA M-13.5) is preserved: the parent absorbed
  the evicted child's value, so the new child faces a hardened bar.
- V-I3 is trivially preserved (zero-intensity insertion).

**Cons:**

- New code path in `rebalance.rs` for legacy promotion.
- V-Tree insertion logic needs position-aware placement (new entry
  goes into the vacated V-seat under the same parent).
- Requires `is_exposed` flag propagation changes in eviction (parent
  state transitions from internal → semi-internal and semi-internal
  → terminal both change `is_exposed`).

---

## Decision

**Option C — Full reframe with legacy promotion.**

### Rationale

1. **Spec alignment.** The spec (§§IDEA M-4.2,4.3, 11.6, 13.2)
   defines the flag names, the legacy promotion mechanism, and the
   lifecycle transitions. This ADR aligns the codebase with the spec.

2. **Correct lifecycle.** The stuck semi-internal scenario is a real
   limitation. A region that loses one child to a transient traffic
   dip should regrow that child when traffic returns, without
   destroying the sibling. The sibling earned its V-Tree position
   through competition — forcing its destruction is information
   destruction.

3. **Competition as the primary gate.** Legacy promotion fires when
   the entry outgrows every uncle — a V-I3 violation. The depth
   gate (`depth_V(c) ≤ D_evict`, §IDEA M-11.6) is a cost filter: it
   suppresses heir creation when the heir would be immediately
   eviction-eligible. The V-Tree's competitive mechanism drives the
   decision; the depth check avoids futile work.

4. **Benchmark compounding is preserved.** When the child was
   evicted, its `sum` was absorbed into the parent's `own`
   (ADR-M-014). The parent's hardened benchmark means the new child
   must prove significance against a higher bar. The
   self-regulation mechanism (§IDEA M-13.5) is unaffected.

5. **V-I3 is trivially preserved.** The new entry starts at zero
   intensity. Adding a zero-intensity node cannot weaken any uncle
   shield (§IDEA M-10.4 argument applies identically).

6. **Clean flag semantics.** `is_exposed` answers "does this entry
   accumulate?" `has_evictable` answers "can this subtree contract?"
   Each flag's purpose is self-documenting. The spec (§IDEA M-4.3) explains
   exactly why no third flag is needed for legacy promotion.

---

## Consequences

### Code changes

- **`vnode.rs`:** Rename `is_geo_terminal` → `is_exposed` in
  `VKind::Entry`. Rename `has_geo_terminal` → `has_evictable` in
  `VKind::Structural`. Add `is_evictable: bool` to `VKind::Entry`
  as a cache of `¬has_dependents(gnode)` (see Analysis). Update all
  doc comments. `is_exposed` is now true for terminal AND
  semi-internal G-nodes. **Size impact: zero** — Entry is the
  smaller variant; adding 1 byte does not change the enum's
  footprint.

- **`vtree.rs`:** Rename `propagate_terminal_flags` →
  `propagate_evictable_flags`. Propagation reads `is_evictable`
  from entry children (the cached bool, §IDEA M-9.3). No signature
  change — the function still takes `(vnodes, start)` only.

- **`gnode.rs`:** The helper methods `uncovered_range()`,
  `has_dependents()`, and `is_semi_internal()` decided in
  [ADR-M-016](016-semi-internal-state.md) are unchanged. No new
  stored fields — all three are computed from `left`/`right`.

- **`split.rs`:** Split guard unchanged — only terminal (fully
  exposed, zero children) nodes can split via catalytic split.
  The `is_exposed` flag flip after split is the same: parent goes
  from exposed to not-exposed when both children are created.

- **`rebalance.rs`:** In the resolve dispatcher (§IDEA M-11.9), at the
  skip-promote branch: check `is_semi_internal(c.gnode)` and
  `depth_V(c) ≤ D_evict`. If both hold, dispatch to
  `legacy_promote`. If either fails, dispatch to `skip_promote`
  as before. Add `legacy_promote` function per §IDEA M-11.6.
  **Implementation note:** after `legacy_promote`, the dispatcher
  also pushes side-effect violations at `p` (not just `g`), because
  replacing `c` (high intensity) with `ne` (zero) inside `p`
  weakens uncle shields for `s`'s children — grandchildren of `p`
  that `push_side_effect_violations(g)` would miss.

- **`evict.rs`:** Eligibility check reads `is_evictable` (the
  cached bool). Scan pruning reads `has_evictable`. After eviction,
  update parent's `is_exposed` from `g.uncovered_range().is_some()`
  and `is_evictable` from `g.is_terminal()`. Propagate
  `has_evictable` up the V-Tree.

- **`invariants.rs`:** Update V-I6 to verify
  `is_exposed == (uncovered_range(g) != None)`. Add V-I6b
  (debug-only) to verify `is_evictable == g.is_terminal()`. Update
  V-I7 to verify `has_evictable` propagation.

- **`observe.rs`:** No change — observation routing via
  `gtree::route_to_receiver` already handles semi-internal nodes
  correctly (stops at the exposed uncovered half).

### Invariant amendments

| Invariant | Change                                                                           |
| --------- | -------------------------------------------------------------------------------- |
| V-I6      | `is_exposed = (uncovered_range(g) ≠ null)` — true for terminal AND semi-internal |
| V-I6b     | **New:** `is_evictable = g.is_terminal()` — validates the cache                  |
| V-I7      | Renamed: `has_evictable` propagation (reads `is_evictable` from entries)         |
| G-I2      | Unchanged (fanout 0, 1, or 2)                                                    |
| D-I1      | Unchanged: `split(g) ⟹ ¬has_dependents(g) ∧ depth_V ≤ D_create` (terminal-only)  |

### Spec sections implemented

| Section | What it describes                                                           |
| ------- | --------------------------------------------------------------------------- |
| §IDEA M-4.1    | Three node states, `uncovered_range()`, exposed/dependents table            |
| §IDEA M-4.2    | V-Entry `is_exposed` and `is_evictable` flags                               |
| §IDEA M-4.3    | V-Structural `has_evictable` flag, why no legacy-promotion flag             |
| §IDEA M-11.6   | `legacy_promote` function                                                   |
| §IDEA M-11.7   | Decision table: entry + semi-internal + depth ≤ D_evict → legacy promote    |
| §IDEA M-11.9   | Resolve dispatcher: `is_semi_internal` + depth check at skip-promote branch |
| §IDEA M-12.3   | Eviction eligibility: depth + ¬has_dependents                               |
| §IDEA M-13.2   | Full G-node lifecycle with legacy promotion arrow                           |

### What does NOT change

- **Eviction safety.** Only 0-child G-nodes can be evicted. The
  structural guarantee is unchanged.
- **Observation routing.** `route_to_receiver` already handles
  semi-internal nodes. No change.
- **Catalytic split.** The existing two-child split is untouched.
  Splitting remains terminal-only.
- **Bootstrap split.** Unchanged — only fires when V-root is a
  lone entry (always terminal at that point).
- **Standard promote / skip promote.** Unchanged. Legacy promote is
  a new dispatch target alongside skip promote, not a replacement.
- **Contraction.** Unchanged.
- **Value absorption.** ADR-M-014 is unchanged.
- **48-byte `GNode` target.** No new stored fields on `GNode`
  (ADR-M-016). `uncovered_range()`, `has_dependents()`, and
  `is_semi_internal()` are computed from `left`/`right`.
- **64-byte `VNode` target.** `is_exposed` is the same field as
  `is_geo_terminal`, broadened in semantics. The `is_evictable`
  cache adds 1 byte to Entry, but Entry is the smaller variant —
  the enum footprint is unchanged.
