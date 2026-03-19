# ADR-M-013: Eviction Eligibility & Scan Pruning

**Status:** Decided  
**Date:** 2026-02-25  
**Updated:** 2026-03-03  
**Amended by:** [ADR-M-027](027-observation-receiving-reframe.md) (flag
rename: `is_geo_terminal` → `is_exposed` / `is_evictable`,
`has_geo_terminal` → `has_evictable`)  
**API:** [§API M-5.2](../docs/api.md#manual-eviction)
(`check_evictions`), [§API M-5.2](../docs/api.md#accessors)
(size metrics)  
**Surface:** 3 (Emulsion)

**Related ADRs:**

- [ADR-M-014](014-value-absorption.md): absorption semantics
  (`parent.own += child.sum`)
- [ADR-M-015](015-eviction-scan-design.md): scan trigger, ordering,
  batching, trailing rebalance
- [ADR-M-016](016-semi-internal-state.md): semi-internal state
  representation (computed from `left`/`right`, no stored field)
- [ADR-M-017](017-dynamic-depth-control.md): dynamic `D_evict`
  adjustment under budget pressure
- [ADR-M-018](018-hard-budget-guarantee.md): dynamic soft limit
  (`soft_limit = budget − headroom`) as eviction trigger
- [ADR-M-027](027-observation-receiving-reframe.md): `is_exposed` vs
  `is_evictable` flag split — semi-internal nodes are exposed but
  not evictable

## Context

When the tree exceeds its memory budget, it must selectively
reclaim memory. We need clear rules for **which** entries may be
evicted, and an efficient strategy for **finding** them.

The tree contains three kinds of G-nodes (§IDEA M-4.1):

- **Terminal:** zero G-children. Its entry describes a leaf region.
- **Semi-internal:** one G-child. One half of its region is covered
  by that child; the other half is described by its own entry.
- **Internal:** two G-children. Both halves are covered by children;
  its own entry is a frozen benchmark from the moment of splitting.

Evicting a non-terminal G-node would orphan its children in the
G-Tree, requiring cascade deletion or re-parenting — both expensive
and complex. The spec avoids this entirely.

**Spec references:** §IDEA M-4.1 (node state table — terminal-only
reasoning), §IDEA M-4.2 (`is_exposed` flag on V-Entry, `is_evictable`
cache), §IDEA M-4.3 (`has_evictable` flag on V-Structural), §IDEA M-6.2 (V-I6,
V-I6b, V-I7 invariants), §IDEA M-7.2 (D-I2 — the formal eligibility
invariant), §IDEA M-7.4 (`adjust_depth_gates`), §IDEA M-9.3 (evictable flag
propagation), §IDEA M-12.3 (eligibility), §IDEA M-12.5 (the eviction operation),
§IDEA M-12.6 (the eviction scan).

## Decision

### Eligibility rule

A V-entry is eligible for eviction when **all three** conditions hold
(D-I2, §IDEA M-7.2; elaborated in §IDEA M-12.3):

$$\text{evict}(v) \iff \text{depth}_V(v) > D_{\text{evict}} \;\wedge\; \neg\,\text{has\_dependents}(v.\text{gnode}) \;\wedge\; v.\text{gnode} \neq G_{\text{root}}$$

The implementation checks `v.is_evictable` — a cached form of
`¬has_dependents(v.gnode)` stored on V-Entry (V-I6b, §IDEA M-4.2).

- **Depth condition (`depth_V(v) > D_evict`):** the entry has sunk
  deep in the V-Tree's competitive tournament, meaning its intensity
  is globally insignificant relative to the rest of the tree.
  `D_evict` is dynamically adjusted under budget pressure
  ([ADR-M-017](017-dynamic-depth-control.md)); the effective trigger
  uses `soft_limit = budget − headroom` rather than the raw budget
  ([ADR-M-018](018-hard-budget-guarantee.md)).
- **Dependents condition (`¬has_dependents`, cached as
  `is_evictable`):** the backing G-node has zero G-children (§IDEA M-4.2,
  V-I6b). Removing it cannot orphan any descendant.

All three conditions are necessary. Depth alone would remove
structurally load-bearing nodes. The dependents check alone would
remove significant contour cells. The root exemption guarantees the
V-Tree is never empty (V-I0). Together they select exactly the
globally insignificant, structurally exposed, non-root tips
(§IDEA M-12.3).

### Terminal-only eviction (no cascade)

Only terminal G-nodes are eviction-eligible. This is the fundamental
structural guarantee:

- **No orphans.** A terminal node has no G-children. Removing it
  leaves the G-Tree well-formed.
- **No deferred cleanup.** There are no ghost entries to sweep.
- **Sampling safety.** The V-Tree and G-Tree remain consistent at
  all times — every span in the V-Tree maps to a live G-node.

Internal and semi-internal G-nodes are **eviction-immune** regardless
of their V-depth (§IDEA M-12.3). Their entries may sit past `D_evict`
indefinitely as frozen benchmarks. This is harmless: each frozen
entry occupies O(1) space and carries negligible sampling weight (it
is deep in the V-Tree, so it is rarely sampled).

Semi-internal entries resolve naturally (§IDEA M-12.3):

- **Hot path.** The uncovered half receives traffic, the entry
  grows, legacy promotion (§IDEA M-11.6) fires — the contour regrows at
  that point. The rebalance dispatcher (§IDEA M-11.9) checks
  `is_semi_internal(c.gnode)` and `depth_V(c) ≤ D_evict` to
  dispatch to `legacy_promote` vs `skip_promote`
  ([ADR-M-027](027-observation-receiving-reframe.md)).
- **Cold path.** The surviving child's subtree erodes from its own
  tips inward. The parent eventually loses all dependents, joins
  the contour tips, and becomes eligible itself.

### Flag semantics: `is_exposed` vs `is_evictable`

Two orthogonal V-Entry flags track distinct properties
([ADR-M-027](027-observation-receiving-reframe.md)):

| Flag           | True for                 | Governs                                         | Invariant |
| -------------- | ------------------------ | ----------------------------------------------- | --------- |
| `is_exposed`   | Terminal + semi-internal | Contour membership, observation routing         | V-I6      |
| `is_evictable` | Terminal only            | Eviction eligibility (caches `¬has_dependents`) | V-I6b     |

Semi-internal entries are exposed (they receive observations in
the uncovered half) but **not evictable** (they have a surviving
child). This distinction is why `has_evictable` on V-Structural
reads `c.is_evictable` rather than `c.is_exposed` (§IDEA M-4.3, §IDEA M-9.3).

### G-root exemption

The G-root's entry is **always exempt** from eviction. Under typical
operation it is the heaviest entry (near the V-root). Even in
degenerate cases, it is a single entry — O(1) space.

Enforcement: the scan filters the G-root from candidates, and
`evict_tip` asserts the precondition:

```rust
// scan_dfs: skip the G-root during collection
if depth > graph.live_depth_evict && *is_evictable && *gnode != graph.g_root {
    candidates.push(v_id);
}

// evict_tip: hard assert (belt-and-suspenders)
assert_ne!(gnode_id, graph.g_root, "evict_tip: cannot evict the G-root");
```

The G-root's entry should rarely (if ever) sink past `D_evict`,
but the double guard prevents pathological cases.

### Scan pruning via `has_evictable`

The `has_evictable` flag on V-Structural nodes (V-I7, §IDEA M-6.2;
maintained via bottom-up propagation, §IDEA M-9.3) enables O(1) pruning of
V-subtrees during the eviction scan:

- **`has_evictable == false`:** this V-subtree contains only
  frozen internal/semi-internal entries. No evictable terminals.
  Skip the entire subtree.
- **`has_evictable == true`:** at least one terminal entry exists
  in this subtree. Descend and check.

The propagation function (`propagate_evictable_flags`, §IDEA M-9.3)
walks from a changed entry up to the V-root, reading
`c.is_evictable` on entry children and `c.has_evictable` on
structural children. It piggy-backs on operations that already
traverse the V-Tree for sum propagation, and early-terminates
when the flag is unchanged — O(1) typical, O(h_V) worst case.

This transforms the scan from O(V-nodes) to O(E_t + S_t) in
the common case, where E_t is evictable entries past `D_evict` and
S_t is the structural nodes on paths to them (§IDEA M-12.6, Cost analysis).
In a cooled-down region (no terminal entries), the scanner never
descends.

## Consequences

- Eviction is structurally safe: the G-Tree is always well-formed
  after eviction.
- Semi-internal and internal G-nodes are immune — no complex
  cascade logic, no re-parenting.
- Semi-internal entries resolve through either the hot path (legacy
  promotion, §IDEA M-11.6) or the cold path (tip erosion), both of which
  converge without special-casing.
- The pruned scan is efficient: proportional to the number of
  evictable entries, not the total tree size.
- The G-root exemption is explicit and cheap (one comparison).
- The `has_evictable` flag, already maintained during core
  V-Tree operations (split, eviction, legacy promotion), now pays
  for itself as the pruning key for eviction scans.
- The `is_exposed` / `is_evictable` flag split
  ([ADR-M-027](027-observation-receiving-reframe.md)) cleanly
  separates contour membership from eviction safety, enabling
  legacy promotion to rely on `is_exposed` without conflating the
  two concerns.
