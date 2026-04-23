# ADR-M-014: Value Absorption Semantics

**Status:** Decided  
**Date:** 2026-02-25  
**Updated:** 2026-03-03  
**API:** [§API M-5.2](../docs/api.md#manual-eviction)
(`check_evictions` — absorption on eviction)  
**Surface:** 3 (Emulsion)

**Spec references:** §IDEA M-12.4 (absorption semantics), §IDEA M-12.5
(the eviction operation — Steps 1–10), §IDEA M-12.5.1 (Two-Path Coverage
Lemma), §IDEA M-12.9.3(G) (ghost fast path — normative O(1) requirement),
§IDEA M-8 (observation flow — energy conservation identity).

**Related ADRs:**

- [ADR-M-003](003-violation-tracking.md): violation tracking strategy
  (sources 1–2, ancestor-walk pattern)
- [ADR-M-012](012-g-sum-recomputation.md): G-sum recomputation —
  `propagate_g_sums` retained as utility but absorption does not
  need it (see Consequences)
- [ADR-M-013](013-eviction-eligibility.md): eligibility rule
  (`depth_V > D_evict ∧ is_evictable`)
- [ADR-M-015](015-eviction-scan-design.md): scan trigger, ordering,
  batching, trailing rebalance
- [ADR-M-016](016-semi-internal-state.md): semi-internal state
  representation (parent transitions after absorption)
- [ADR-M-027](027-observation-receiving-reframe.md): `is_exposed` /
  `is_evictable` flag split — semi-internal nodes are exposed but
  not evictable

## Context

Eviction removes a terminal G-node from the tree. The evicted
child's `sum` (equal to `own` for a terminal) represents real
observation energy. Discarding it would violate energy
conservation. Re-distributing it across multiple nodes would add
complexity. The spec prescribes a single clean operation: the
parent absorbs everything (§IDEA M-12.4).

Two G-Tree invariants are at stake:

- **G-I1 (Summation):** `g.sum = g.own + Σ children.sum` for every
  G-node (§IDEA M-5.4).
- **G-I4 (Entry Consistency):** `g.entry ≠ null ⟹ g.entry.int =
g.own` (§IDEA M-5.4).

And a global accounting identity:

- **Energy conservation:** the G-root's `sum` equals the total of
  all observations ever made; the V-Tree's total intensity equals
  the G-root's `sum` (§IDEA M-8, §IDEA M-12.4).

## Decision

### Absorption rule (§IDEA M-12.4, §IDEA M-12.5 Step 1)

When terminal G-child `c` is evicted, its G-parent `p` absorbs the
child's energy:

```
p.own += c.sum
```

Then `c` is detached (`p.left = null` or `p.right = null`) and
later deallocated (Step 10 of §IDEA M-12.5). The parent transitions
state (§IDEA M-12.4):

- **Had 2 children → semi-internal.** One child vacated; the
  sibling remains. The parent is now partially exposed, receiving
  observations in the vacated half.
- **Had 1 child → fully exposed (terminal).** Both children gone.
  The parent sits on the contour at one level shallower.

### G-sum is invariant under terminal absorption

**No G-sum propagation is needed after absorbing a terminal child.**

Proof (from §IDEA M-12.5 Design Note, Step 1 Invariant):

```
Before:  p.sum = p.own_old + c.sum + sibling.sum
After:   p.own_new = p.own_old + c.sum        (absorption)
         c removed: p.sum = p.own_new + 0 + sibling.sum
                         = (p.own_old + c.sum) + sibling.sum
                         = p.own_old + c.sum + sibling.sum
                         = old p.sum  ✓
```

When `c` is the only child (no sibling), `sibling.sum = 0` and the
identity holds trivially. When both children are evicted in the same
pass, each absorption independently preserves `p.sum`.

Because `sum` is a stored field (not recomputed on access), its
stale value is already correct after the compensating mutations in
Step 1. No explicit sum update or G-Tree propagation is needed —
the G-side cost is O(1).

**Both trees conserve total intensity** (§IDEA M-12.4). The G-Tree root's
sum is unchanged. The V-Tree's total intensity is also unchanged:
the parent's entry gains `g.sum` while the evicted entry (also
carrying `g.sum`, since `g` is terminal) is removed. The two trees'
totals remain equal after eviction, preserving the accounting
identity from §IDEA M-8.

**Implementation:** `debug_assert!` that `p.sum` is unchanged after
absorption, then recompute from children for bit-level exactness.
No call to `propagate_g_sums` or `recompute_g_sums`.

### Ghost fast path (§IDEA M-12.5 Step 2, normative — §IDEA M-12.9.3(G))

When `c.sum = 0` (a ghost node), absorption adds zero to `p.own`.
Steps 3–4 would propagate a zero delta and find no violations,
costing O(h_V) for no effect. The spec mandates skipping them
(§IDEA M-12.5 Step 2). Steps 5–10 must still execute — the parent's
exposure/evictable state genuinely changes, the V-entry must be
removed, and the G-node deallocated.

**Conforming implementations must implement this fast path** to
achieve the O(|G|₀ · h_V) contraction work bound (§IDEA M-12.9.3(G)).
Without it, ghost evictions inflate total contraction work from
O(|G|₀ · h_V) to O(|G|₀ · h_V²).

### V-entry intensity update (§IDEA M-12.5 Step 3)

After absorption, the parent's V-entry intensity must be updated to
maintain G-I4:

```rust
let p_entry_id = gnodes.get(parent_id).entry
    .expect("parent must have V-entry (has dependents)");
let p_own = gnodes.get(parent_id).own;
vnodes.get_mut(p_entry_id).intensity = p_own;
vtree::update_parent_cached_intensity(&mut vnodes, p_entry_id, p_own);
vtree::propagate_v_sums(&mut vnodes, p_entry_id);
```

The `expect` on `p.entry` is safe: the parent had dependents during
Phase 1's scan (it contained `c`), so it was not in the candidate
list and was never evicted. Its entry persists (see §IDEA M-12.5 Step 3
assertion rationale).

### Violation tracking after absorption (§IDEA M-12.5 Steps 4, 6–8)

Eviction creates violations from **two** sources:

**(a) Absorption intensity increase (Step 4).** The intensity
increase on `p`'s V-entry may cause max-uncle violations (V-I3),
exactly as an observation does. The same ancestor-walk pattern from
[ADR-M-003](003-violation-tracking.md) (sources 1–2) is performed:

```rust
let mut check_id = Some(p_entry_id);
while let Some(id) = check_id {
    if is_violated(id) { violations.push(id); }
    check_id = vnodes.get(id.index()).parent;
}
```

**(b) V-Tree structural removal (Steps 6–8).** Removing the V-entry
via `vtree_remove_leaf` (Step 7) decreases ancestor intensities,
weakening uncle shields. The spec defines four push functions
dispatched by the V-parent's pre-removal child count:

| Child count | Case                          | Push functions                                           | §IDEA M-11.12 source |
| ----------- | ----------------------------- | -------------------------------------------------------- | -------------------- |
| any         | Ancestor intensity decrease   | `push_leaf_removal_violations(change_point)`             | 6                    |
| 2           | Collapse (V-parent destroyed) | `push_collapse_violations(collapse_sibling)`             | 7                    |
| 2           | Cousins' children             | `push_cousin_violations(collapse_sibling, change_point)` | 9                    |
| 3           | 3→2 transition                | `push_remaining_sibling_violations(v_parent, v_id)`      | 8                    |

Step 6 pre-captures the V-Tree context (change point, surviving
sibling) before `vtree_remove_leaf` potentially destroys the
V-parent in the collapse case.

The Two-Path Coverage Lemma (§IDEA M-12.5.1) proves that Steps 4 and 8
together discover every V-I3 violation that persists after both the
intensity increase (Step 3) and the structural removal (Step 7)
complete. Duplicates are filtered by the rebalance loop's re-check
(§IDEA M-11.8).

All violations queue for the trailing rebalance in `check_evictions`
(Phase 3 of the two-phase collect-then-evict pattern,
[ADR-M-015](015-eviction-scan-design.md)).

### Flag updates (§IDEA M-12.5 Step 5)

After detaching child `c`, the parent's V-entry flags are updated
to maintain V-I6 and V-I6b:

- **`is_exposed ← true`** (V-I6): removing a child always exposes
  range (the parent is now on the contour for at least the vacated
  sub-region).
- **`is_evictable ← ¬has_dependents(p)`** (V-I6b): if the parent's
  remaining child was `c`'s sibling and it too was evicted (or
  was already gone), the parent is now terminal and evictable.
  Otherwise, it still has a child and is not evictable.

Then `propagate_evictable_flags` walks up the V-Tree from the
parent's V-entry, maintaining V-I7 (`has_evictable` on structural
nodes).

**Step 4/5 ordering (design note from §IDEA M-12.5).** Step 4's violation
walk runs _before_ Step 5 updates `is_exposed` and `is_evictable`.
This is deliberate: `is_violated()` (§IDEA M-11.2) depends only on
intensities — it never reads the flags. The flags are stale but
harmless during Step 4. Performing Step 4 before Step 5 simplifies
the ordering (the intensity mutation in Step 3 and its violation
consequences in Step 4 are adjacent) without affecting correctness.
Violations pushed in Step 4 are not resolved until the trailing
rebalance (Phase 3), which runs after all individual evictions —
including their Step 5 flag updates — have completed.

### V-entry removal (§IDEA M-12.5 Step 7)

`vtree_remove_leaf(v)` removes the evicted entry from the V-Tree.
In the 2-node case this collapses the structural parent (destroying
it and re-parenting the surviving sibling). In the 3→2 case the
structural parent survives as a 2-node. The function also nulls
`g.entry` on the backing G-node (§IDEA M-9.2).

### Plateau maintenance (§IDEA M-12.5 Step 9)

After structural changes, the plateau ordered map (§IDEA M-5.6.7) must be
updated. The evicted terminal's basis element (if any) is removed,
and the parent is placed in the plateau map with its new contour
state. The implementation handles three cases: parent was already a
basis element, an ancestor's tile covered the parent, or the parent
was descendant-covered. Displaced sibling subtrees are collected
and re-placed via sorted left-to-right insertion
([ADR-M-031](031-sorted-placement-normalize-elimination.md)).

### Deallocation (§IDEA M-12.5 Step 10)

The evicted G-node's arena slot is deallocated and `node_count` is
decremented by 1. `terminal_count` is decremented by 1 for the
evicted terminal; if the parent is now terminal (both children
gone), `terminal_count` is incremented by 1.

## Options Considered

### A. Absorption into parent (chosen)

Parent takes the child's energy. Simple, local, preserves `p.sum`
exactly. O(1) on the G-side.

### B. Distribute to nearby entries

Split the evicted energy among siblings or ancestors proportionally.
More "fair" but adds complexity and changes multiple nodes'
energies. Violates the clean-accounting property.

## Consequences

- Eviction is O(1) on the G-Tree. V-Tree cost is O(h_V) for sum
  propagation and violation walks.
- The `propagate_g_sums` utility from [ADR-M-012](012-g-sum-recomputation.md)
  is retained but unused by absorption. ADR-M-012's Consequences
  section documents this relationship.
- The ghost fast path (§IDEA M-12.9.3(G)) is normative: implementations
  must skip Steps 3–4 when `g.sum = 0`.
- Semi-internal parents re-enter the observation routing topology
  for the vacated half ([ADR-M-027](027-observation-receiving-reframe.md)).
  Legacy promotion (§IDEA M-11.6) can regrow the contour without
  re-splitting from scratch.
