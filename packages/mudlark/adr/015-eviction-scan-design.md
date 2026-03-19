# ADR-M-015: Eviction Scan Design

**Status:** Decided  
**Date:** 2026-02-25  
**Updated:** 2026-03-03  
**API:** [§API M-5.2](../docs/api.md#manual-eviction)
(`check_evictions`), [§API M-5.1](../docs/api.md#51-configv)
(`bounded_eviction`)  
**Surface:** 3 (Emulsion)

## Context

The eviction subsystem identifies entries that have sunk past the
depth threshold `D_evict` and are backed by terminal G-nodes
([ADR-M-013](013-eviction-eligibility.md)) and removes them via
absorption ([ADR-M-014](014-value-absorption.md)). Five interrelated
design choices must be resolved together:

1. **Trigger:** When does `check_evictions` fire?
2. **Ordering:** In what order are eligible entries evicted?
3. **Count:** How many evictions per pass?
4. **Rebalance timing:** Inline per-eviction or batched?
5. **Return value:** What does `check_evictions` report?

These choices interact: the trigger strategy constrains the cost
model, which constrains acceptable per-pass counts, which constrains
rebalance timing. This ADR resolves them as a unit.

**Spec references:** §IDEA M-8 (Step 7 — `check_evictions()`),
§IDEA M-12.6 (scan pseudocode / `scan_for_candidates`), §IDEA M-12.5 (tip
eviction operation / absorption), §IDEA M-12.5.1 (Two-Path Coverage
Lemma — violation correctness across intensity + structural
mutations), §IDEA M-12.7 (bottom-up contraction / tides), §IDEA M-12.8 (two
simplifications — direct + implicit), §IDEA M-12.9 (correctness,
convergence to fixed point, finite-time bound), §IDEA M-12.9.3(G) (ghost
fast path — normative $O(1)$ requirement), §IDEA M-7.4 (dynamic depth
control / `adjust_depth_gates()`).

**Related ADRs:**

- [ADR-M-013](013-eviction-eligibility.md): eligibility rule
  (`depth_V > D_evict ∧ is_evictable`)
- [ADR-M-014](014-value-absorption.md): absorption semantics
  (`parent.own += child.sum`)
- [ADR-M-017](017-dynamic-depth-control.md): dynamic depth control
  (`adjust_depth_gates` runs as Step 7, before eviction)
- [ADR-M-018](018-hard-budget-guarantee.md): introduces the dynamic
  soft limit (`soft_limit = budget − max(headroom, 2*(D_create − 1))`)
  that replaces raw `budget` as the eviction trigger
- [ADR-M-029](029-opportunistic-depth-caching.md): cached V-depth
  reduces Phase 2 re-verification from O(h_V) to O(1) per candidate
- [ADR-M-031](031-sorted-placement-normalize-elimination.md): sorted
  plateau placement; `normalize_plateaus` runs post-eviction

---

## Analysis

### The cost of a scan

The eviction scan is a DFS of the V-Tree. The raw cost is O(V-nodes)
in the worst case: every structural and entry node visited once.

Three pruning mechanisms reduce this:

1. **`has_evictable` flag (V-I7).** A structural node with
   `has_evictable == false` contains only frozen internal/semi-
   internal entries — none are evictable. The entire subtree is
   skipped in O(1). In a typical tree after Phase 2, most deep
   entries are frozen internals, so this prunes aggressively.

2. **Depth cutoff.** The scan carries a `depth` counter. If `depth`
   already exceeds `D_evict` and a structural node's subtree depth
   is at most 1, all children are entry-level checks — O(children)
   = O(3) per structural node.

3. **Early exit on soft-limit relief (if budget-triggered).** When
   eviction is triggered by `node_count > soft_limit` (ADR-M-018),
   the scan can stop after evicting the excess count. This bounds
   work to the excess, not the tree size.

**Empirical bound:** In the common case (budget-triggered, most
frozen entries pruned), the scan touches O(evictable_tips) nodes
plus O(log n) structural ancestors — comparable to a single
`rebalance()` resolution.

**Worst case:** A tree with no frozen internal entries
(`has_evictable == true` everywhere). Every V-node is visited.
This occurs only when all G-nodes are terminal (no splits, or
everything split and then evicted in all-but-one patterns). Even
then, the scan is O(n) with n ≤ budget.

### The cost of `observe()` today

For context, the current `observe()` pipeline (without eviction):

| Step                   | Cost                                   |
| ---------------------- | -------------------------------------- |
| 1. `route_to_receiver` | O(G-depth) pointer chasing             |
| 2. Accumulate          | O(1)                                   |
| 3. `recompute_g_sums`  | O(G-depth) × 3 cache lines/level       |
| 4. `propagate_v_sums`  | O(V-depth) pointer chasing             |
| 5. `attempt_split`     | O(V-depth) for insert + O(1) amortized |
| 6. `rebalance`         | O(violations) × restructuring cost     |

Steps 3–6 dominate. A typical `observe()` is O(G-depth + V-depth).
An eviction scan that touches O(evictable_tips) is in the same
ballpark as one or two `rebalance` resolutions.

### The `observe()` pipeline position

The spec (§IDEA M-8) places `check_evictions()` as Step 7 — called
unconditionally after rebalance. The implementation refines this
into two sub-steps:

- **Step 7:** `adjust_depth_gates()` (ADR-M-017) — adjusts `D_evict`
  and `D_create` based on `node_count` vs `soft_limit`.
- **Step 8:** Budget-guarded eviction (this ADR + ADR-M-018) — only
  fires when `node_count > soft_limit`.

The ordering is critical:

- Rebalance (Step 6) determines final V-depths. Eviction decisions
  based on pre-rebalance depths would be wrong (an entry might move
  above `D_evict` during rebalance).
- `adjust_depth_gates` (Step 7) must update `D_evict` _before_ the
  eviction scan uses it. Under memory pressure, tightening `D_evict`
  exposes additional entries to eviction.
- Eviction (Step 8) may create new V-I3 violations (via absorption
  intensity changes). These are resolved by a trailing `rebalance()`
  inside `check_evictions` itself (Phase 3).

> **Spec divergence.** The spec (§IDEA M-8) calls `check_evictions()`
> unconditionally on every `observe()`. The implementation makes this
> budget-guarded: the scan only runs when `node_count > soft_limit`
> (ADR-M-018). When `budget` is `None`, automatic eviction is skipped
> entirely; the user can call `check_evictions()` manually. This is
> a deliberate optimisation — see Q1 below.

Current `observe()` pipeline (from `observe.rs`):

```rust
pub fn observe<O: Observation<V>>(&mut self, coord: C, delta: O) {
    // 1. Route
    // 2. Accumulate (G-own + V-entry + propagate V-sums)
    // 3. Recompute G-sums (ADR-M-012)
    // 4. Violation check (ancestor walk)
    // 5. Attempt split
    // 6. Rebalance + handle legacy promotes (ADR-M-017)

    // 7. Dynamic depth control (§IDEA M-7.4, ADR-M-017)
    self.adjust_depth_gates();

    // 8. Budget-guarded eviction (ADR-M-015, ADR-M-018)
    if let Some(soft_limit) = self.soft_limit {
        if self.node_count as usize > soft_limit {
            if self.config.bounded_eviction {
                self.check_evictions_bounded(
                    self.node_count as usize - soft_limit,
                );
            } else {
                self.check_evictions();
            }
        }
    }

    // 9. Normalize plateaus (ADR-M-031)
    // 10. Repair P-I4
}
```

### Soft limit as the trigger (ADR-M-018)

`Config.budget: Option<usize>` is the user-visible hard ceiling.
ADR-M-018 introduces a **dynamic soft limit** (`soft_limit = budget −
max(headroom, 2*(D_create − 1))`) that triggers tightening and
eviction _before_ the hard wall is reached. `node_count: u32` is
maintained O(1) (incremented in `split.rs`, decremented in
`evict.rs`). The trigger check is:

```rust
if let Some(soft_limit) = self.soft_limit {
    if self.node_count as usize > soft_limit {
        // trigger eviction
    }
}
```

This is two comparisons — truly O(1) on the observe hot path.

---

## Q1: Trigger strategy

### Options

| Option                    | Per-observe cost              | When eviction fires | Latency profile             |
| ------------------------- | ----------------------------- | ------------------- | --------------------------- |
| **A. Every observe**      | O(scan) always                | Always              | Flat but high floor         |
| **B. Every K observes**   | O(scan/K) amortized           | Periodic            | Spiky every K               |
| **C. Soft-limit-guarded** | O(1) guard; O(scan) when over | Only under pressure | Spiky on threshold crossing |
| **D. Public-only**        | Zero in observe               | Never automatic     | User-controlled             |
| **E. C + D**              | O(1) guard; O(scan) when over | Automatic + manual  | Best tradeoff               |

### Detailed evaluation

**Option A (every observe)** matches the spec verbatim (§IDEA M-8 Step 7)
but imposes the scan cost on every observation regardless of whether
any entries are evictable. Even with `has_evictable` pruning, the
scan must at minimum visit the V-root and check its flag. For a
high-throughput use case doing millions of observations per second
with `D_evict = 30` (most entries well within bounds), this is wasted
work on every call.

However: Option A has a correctness advantage. If `D_evict` is
static (no dynamic adjustment), an entry can only cross `D_evict`
by being pushed deeper during `rebalance()`. This happens O(1) times
per observation (bounded by rebalance resolutions). So the "scan
finds nothing" case dominates, and the pruned scan is O(1) in
practice — the root's `has_evictable` is true, but no entries
exceed `D_evict`, so the scan visits O(V-depth) structural nodes
and finds nothing.

**Wait — that's not O(1).** Even a "finds nothing" scan is
O(V-depth) if `has_evictable` is true at every level (which it
typically is — there's usually at least one terminal entry somewhere
in each subtree). The scan must descend to the depth where entries
live, check depth vs. `D_evict`, and backtrack. For `D_evict = 20`,
this is ~20 node visits per observe for no benefit.

**Option C (soft-limit-guarded)** eliminates this. When `node_count
<= soft_limit`, the scan doesn't run. The only overhead is the
integer comparison. When the scan does fire (node_count just
exceeded soft_limit due to a split in step 5), it removes the excess
and stops. The next observation's split may push count over
soft_limit again, triggering another scan — but each scan removes
exactly the excess, so the amortized cost is O(evictions per split
cycle), which is bounded.

**Option D (public-only)** is maximally flexible but means the
library never enforces the budget automatically. The user must call
`check_evictions()` manually. This violates the principle of
budget-as-a-contract: if you set `budget = 50`, you expect the
library to respect it.

**Option E (C + D)** gives automatic enforcement plus user control.
The public `check_evictions()` method works independently of budget
— it always scans and evicts all eligible entries. The soft-limit
guard in `observe()` provides automatic enforcement. The user can
also call `check_evictions()` after `decay()` (Phase 4) or any
other bulk mutation that might push entries past `D_evict`.

**What about `budget: None`?** With no budget and static depth gates,
eviction only matters if entries are pushed past `D_evict` by
rebalancing. This is a slow, bounded process. The user who sets
`budget: None` is saying "I don't care about memory bounds." We
should still respect `D_evict` as a correctness threshold for
proportional sampling weight, but running an automatic scan on every
observe is wasteful.

Two sub-options for `budget: None`:

- **E1:** Skip automatic eviction entirely; user calls
  `check_evictions()` manually if they want it.
- **E2:** Still run the scan on every observe (option A behavior).

**E1** is better: the user who sets `budget: None` is opting out of
automatic lifecycle management. They get `check_evictions()` as a
public method if they want it.

### Recommendation: Option E with E1 for `budget: None`

```rust
// Inside observe(), after step 7 (adjust_depth_gates):
if let Some(soft_limit) = self.soft_limit {
    if self.node_count as usize > soft_limit {
        if self.config.bounded_eviction {
            self.check_evictions_bounded(
                self.node_count as usize - soft_limit,
            );
        } else {
            self.check_evictions();
        }
    }
}
```

Plus a public method:

```rust
/// Evict all eligible terminal entries past `D_evict`.
/// Returns the number of entries evicted.
pub fn check_evictions(&mut self) -> u32 { ... }
```

**Cost on hot path (under soft limit):** 1 branch on `Option`, 1
integer comparison. Both predict well. Effectively zero.

**Cost when triggered:** O(evictable_tips) for scan + eviction +
O(violations) for post-eviction rebalance. Bounded by excess over
soft limit.

---

## Q2: Scan ordering

### Options

| Option                                | Algorithm                                                           | Within-pass behavior                     | Cross-pass behavior                     |
| ------------------------------------- | ------------------------------------------------------------------- | ---------------------------------------- | --------------------------------------- |
| **A. DFS tree-order**                 | Recursive/iterative DFS from V-root                                 | Evicts entries in V-Tree traversal order | Newly eligible parents caught next pass |
| **B. Deepest-first**                  | Collect candidates, sort by `v_depth` desc, evict                   | Strict depth ordering                    | Same                                    |
| **C. DFS, skip new-terminal parents** | DFS but skip entries whose parent just became terminal in this pass | Natural bottom-up, no sort               | Slightly more complex loop              |

### Detailed evaluation

**Option B (deepest-first)** collects all eligible entries into a
`Vec`, sorts by `v_depth`, then evicts from deepest to shallowest.
This gives strict bottom-up ordering within a single pass — deepest
tips first, then their parents if they became terminal.

But: `v_depth()` is O(V-depth) per entry (parent-pointer walk to
root). For k candidates, collection is O(k × V-depth). Sort is
O(k log k). Then eviction mutates the tree, invalidating depths of
other candidates. Entries that were at depth 15 before eviction
might be at depth 12 after a sibling's removal triggers rebalancing.
The pre-computed depths become stale.

To be correct, Option B would need to re-verify depth after each
eviction, which defeats the purpose of sorting.

**Option C (DFS, skip new-terminal)** tries to get natural bottom-up
ordering by tracking which parents became terminal during this pass.
But the DFS visits children before parents in post-order (or parents
before children in pre-order). In pre-order: the parent is visited
first — it won't be terminal yet because its children haven't been
evicted. By the time its children are evicted, we've already passed
the parent. So C requires post-order DFS to work, which means
collecting the entire subtree traversal anyway.

**Option A (DFS tree-order)** is the simplest. It matches the spec's
pseudocode exactly (§IDEA M-12.6 `scan_for_candidates`):

```
function scan_for_candidates(v, depth) → list:
    if v = null: return []

    if v is entry:
        if depth > D_evict and v.is_evictable:
            if v.gnode = G_root: return []
            return [v]
        return []

    // v is structural — prune if no evictable descendants
    if not v.has_evictable: return []

    result ← []
    for c in v.children:
        result ← result ∪ scan_for_candidates(c, depth + 1)
    return result
```

When an entry is evicted mid-scan, its parent may become terminal.
But the parent is _above_ the current position in the DFS — it was
already visited and either wasn't eligible (depth ≤ D*evict) or was
already evicted. Either way, the parent is not re-visited in this
pass. If the parent is now terminal \_and* past `D_evict`, it will be
caught in the next `check_evictions` call.

This matches §IDEA M-12.7 (bottom-up contraction): "each round removes one
layer of terminal tips." Multiple rounds over successive `observe()`
calls peel the tree back layer by layer (tides).

**Key insight:** within a single DFS pass, siblings at the same
depth are independent. Evicting sibling A does not affect sibling
B's eligibility (B's depth is unchanged, B's terminal status is
unchanged). The only cross-entry effect is on the _parent_, which is
at a shallower depth and won't be evicted until a future pass.

### Recommendation: Option A

Single pre-order DFS pass. Simple, matches the spec, correct by
construction. Multi-round contraction handles cascading naturally.

---

## Q3: Eviction count per pass

### Options

| Option                 | Stop condition                    | Worst-case per pass | Behavior                        |
| ---------------------- | --------------------------------- | ------------------- | ------------------------------- |
| **A. Unbounded**       | Scan finishes                     | O(all eligible)     | Removes everything past D_evict |
| **B. Fixed cap M**     | M evictions                       | O(M)                | Predictable latency bound       |
| **C. Excess-relative** | evicted >= excess over soft_limit | O(excess)           | Stops when pressure relieved    |

### Detailed evaluation

**Option A (unbounded)** is the natural choice when there's no budget
(the user wants competitive eviction based purely on depth). Every
eligible entry is removed. The pass cost is proportional to the
number of evictions, each of which is O(1) for the eviction itself
plus O(V-depth) for V-sum propagation and violation pushing.

**Option B (fixed cap)** provides a hard latency bound but means the
tree may remain over-budget for multiple observations. With
`budget = 100` and `node_count = 150`, at cap M = 10, it takes 5
observations to converge. During those 5 observations, the tree is
50→40→30→20→10→0 nodes over budget. This is fine for soft budgets
but unacceptable for hard memory bounds.

**Option C (excess-relative)** is the Goldilocks option when a
budget exists. It evicts exactly enough to relieve pressure, then
stops. The eviction loop stops after `node_count − soft_limit`
evictions. The remaining candidates are never processed — pure
savings.

Implementation:

```rust
fn evict_candidates(&mut self, stop_at: Option<usize>) -> u32 {
    let candidates = evict::scan_for_candidates(self);
    let mut evicted: u32 = 0;
    for v_id in candidates {
        if let Some(limit) = stop_at {
            if evicted as usize >= limit { break; }
        }
        // ... re-verify and evict ...
    }
    // trailing rebalance
    evicted
}
```

**The two modes compose cleanly:**

| `budget`  | Behavior                                |
| --------- | --------------------------------------- |
| `Some(n)` | Evict until excess relieved, then stop  |
| `None`    | Evict all eligible (purely competitive) |

The public `check_evictions()` method always uses "evict all
eligible" mode regardless of budget — the user explicitly asked for
a full pass.

### Recommendation: Option C for budget-triggered, Option A for public API

The automatic eviction inside `observe()` (triggered by soft-limit
guard) uses excess-relative stopping **by default**. A config flag
`bounded_eviction` (default `true`) controls this: when `false`,
even budget-triggered eviction performs a full pass. The public
`check_evictions()` always uses unbounded eviction regardless of
budget or this flag.

---

## Q4: Rebalance timing

### Options

| Option                        | Mechanism                                                         | Per-eviction cost | Total cost       |
| ----------------------------- | ----------------------------------------------------------------- | ----------------- | ---------------- |
| **A. Per-eviction rebalance** | Call `rebalance()` after each `evict_tip()`                       | O(rebalance) × k  | O(k × rebalance) |
| **B. Batched violations**     | Push violations during evictions, single `rebalance()` after      | O(1) push         | O(k + rebalance) |
| **C. Second rebalance call**  | B, but structured as a second `rebalance()` at end of `observe()` | O(1) push         | O(k + rebalance) |

### Detailed evaluation

**Option A (per-eviction rebalance)** is the most conservative.
After each `evict_tip()` call, the V-Tree is fully re-balanced
before the next eviction. This means each subsequent eviction sees
correct V-depths, preventing incorrect eligibility decisions.

But: Q2 decided on DFS tree-order. The scan visits entries at their
current depth in the DFS, and evictions during the same pass don't
change the DFS's depth counter (it's computed from the recursion
depth, not from the V-Tree structure). So even if rebalancing moves
nodes around, the scan's depth counter is wrong for nodes below the
rebalanced region.

Actually, this is a problem for _all_ ordering options. Let's think
carefully:

The DFS carries a `depth` parameter incremented at each recursion
level. This counts the _structural_ V-Tree levels from the root.
After a rebalance reshuffles nodes, the structural depth of a node
may change (promoted higher, or pushed deeper). But the DFS's depth
counter reflects the _pre-rebalance_ tree shape.

**This is fine.** Here's why: the DFS traverses the tree's actual
edges. If rebalancing changes the tree between visiting sibling A
and sibling B, the child pointers may point to different nodes than
before. But `vtree_remove_leaf` doesn't move _other_ entries — it
only removes the evicted one and potentially collapses a structural
node. The remaining entries' parent pointers are updated, but their
position in the DFS is determined by the tree edges at the time of
traversal.

For safety: after each eviction, we could re-check the structural
node's children before continuing the DFS. But rebalance can change
the tree more dramatically (promotions move entries across subtrees).

**The safest approach:** collect eviction candidates during the DFS
scan (phase 1), then evict them all (phase 2), then rebalance
(phase 3). This avoids mutating the tree during traversal.

Wait — but evicting in phase 2 changes the tree (absorption, terminal
flags, V-sum propagation). If candidate A and candidate B are
siblings, evicting A changes B's context. Is B still valid?

Yes: B's eligibility depends on B's V-depth (unchanged by A's
eviction before rebalance) and B's `is_evictable` (unchanged —
B is a different G-node). The only thing A's eviction changes is
the _parent's_ state, which is not a candidate in this pass (per Q2,
Option A).

**Two-phase approach:**

```rust
fn check_evictions(&mut self) -> u32 {
    // Phase 1: collect eligible entry VNodeIds
    let candidates = evict::scan_for_candidates(self);

    // Phase 2: evict each candidate
    let mut count = 0;
    for v_id in candidates {
        if !self.vnodes.is_occupied(v_id.index()) { continue; }
        // Re-verify eligibility (may have been invalidated by prior eviction)
        if self.is_eviction_eligible(v_id) {
            evict::evict_tip(self, v_id);
            count += 1;
        }
    }

    // Phase 3: drain violations
    rebalance::rebalance(&mut self.vnodes, &mut self.violations);
    count
}
```

The re-verification guard in phase 2 handles the rare case where a
prior eviction invalidated a candidate (e.g., the node was destroyed
because both it and its sibling were candidates and the sibling was
evicted first, causing structural collapse that consumed this node).

**Option C (second rebalance call)** is the two-phase approach
integrated into the `observe()` pipeline:

```rust
// In observe(), after step 7:
if soft_limit_exceeded {
    self.check_evictions();  // includes internal rebalance
}
```

Or if `check_evictions` doesn't rebalance internally, the pipeline
does:

```rust
if soft_limit_exceeded {
    self.eviction_scan();  // pushes violations
    rebalance::rebalance(&mut self.vnodes, &mut self.violations);
}
```

### Recommendation: Two-phase collect-then-evict with single trailing rebalance

Phase 1 (scan) collects candidates into a `Vec<VNodeId>`. Phase 2
evicts each with re-verification. Violations accumulate during
phase 2. Phase 3 is a single `rebalance()` call to drain all
violations. This avoids mutating the tree during traversal, avoids
redundant rebalance passes, and handles invalidated candidates
gracefully.

The `Vec` allocation is bounded by the number of evictable entries,
which is bounded by the soft-limit excess (in budget-triggered mode)
or the number of terminal entries past `D_evict` (in manual mode).

---

## Q5: Return value

### Options

| Option        | Signature                                         | Caller value                          |
| ------------- | ------------------------------------------------- | ------------------------------------- |
| **A. Unit**   | `fn check_evictions(&mut self)`                   | None                                  |
| **B. Count**  | `fn check_evictions(&mut self) -> u32`            | Diagnostics, testing, budget feedback |
| **C. Report** | `fn check_evictions(&mut self) -> EvictionReport` | Monitoring, policy                    |

### Detailed evaluation

**Option A** matches `rebalance()` (which returns nothing). But
`rebalance()` is an internal function — users never call it.
`check_evictions()` is a public API method. Users calling it
manually will want to know whether it did anything.

**Option C** (`EvictionReport { evicted: u32, scanned: u32,
skipped_pruned: u32, ... }`) is rich but premature. We don't know
what monitoring information is useful until we have real usage.
Adding fields to a struct is a backward-compatible change; we can
start simple and grow.

**Option B** is the sweet spot. A `u32` count tells the caller:

- `0` → nothing was evictable (can stop calling)
- `> 0` → entries were removed (may want to call again for
  multi-round contraction per §IDEA M-12.7)

The count is also essential for testing: assert that exactly N
entries were evicted under known conditions.

### Recommendation: Option B

```rust
/// Scan the V-Tree and evict all eligible terminal entries
/// past `D_evict`. Returns the number of entries evicted.
pub fn check_evictions(&mut self) -> u32
```

---

## Decision

**Q1: Trigger** → **Option E** (soft-limit-guarded automatic +
public method). Automatic eviction fires inside `observe()` only
when `soft_limit` is `Some` and `node_count > soft_limit` (ADR-M-018).
No automatic eviction when `budget` is `None`. Public
`check_evictions()` is always available for manual use.

**Q2: Ordering** → **Option A** (single pre-order DFS pass).
Matches §IDEA M-12.6. Multi-round contraction (§IDEA M-12.7) handles
cascading across passes.

**Q3: Count** → **Option C/A hybrid**. Budget-triggered eviction
stops after evicting `node_count − soft_limit` entries (early exit)
if `Config.bounded_eviction` is `true` (the default). When `false`,
budget-triggered eviction still does a full pass. Public
`check_evictions()` always evicts all eligible (unbounded).

**Q4: Rebalance** → **Two-phase collect-then-evict** with a single
trailing `rebalance()`. Scan collects candidates into a `Vec`,
eviction loop processes them with re-verification, violations
accumulate and are drained in one pass.

**Q5: Return** → **Option B** (`u32` eviction count).

### The `observe()` pipeline, post-Phase 3

```rust
pub fn observe<O: Observation<V>>(&mut self, coord: C, delta: O) {
    let g_id = gtree::route_to_receiver(...);        // 1. Route
    // 2. Accumulate (G-own + V-entry + propagate V-sums)
    // 3. Recompute G-sums (ADR-M-012)
    // 4. Violation check (ancestor walk)
    split::attempt_split(self, g_id);                 // 5. Split
    let new = rebalance::rebalance(...);              // 6. Rebalance
    self.handle_legacy_promotes(&new);                // 6b. Legacy promotes

    // 7. Dynamic depth control (§IDEA M-7.4, ADR-M-017)
    self.adjust_depth_gates();

    // 8. Eviction (ADR-M-015, ADR-M-018)
    if let Some(soft_limit) = self.soft_limit {
        if self.node_count as usize > soft_limit {
            if self.config.bounded_eviction {
                self.check_evictions_bounded(
                    self.node_count as usize - soft_limit,
                );
            } else {
                self.check_evictions();
            }
        }
    }

    self.normalize_plateaus();                        // 9. ADR-M-031
    self.repair_p_i4();                               // 10. P-I4 repair
}
```

### The public `check_evictions()` method

```rust
/// Evict all eligible terminal entries past `D_evict`.
/// Always performs a full pass regardless of budget.
/// Returns the number of entries evicted.
pub fn check_evictions(&mut self) -> u32 {
    self.evict_candidates(None)
}

/// Budget-bounded eviction — stops after `stop_at` evictions.
/// Called automatically by `observe()` when over soft limit.
/// `stop_at` is computed as `node_count − soft_limit` (the excess).
pub(crate) fn check_evictions_bounded(&mut self, stop_at: usize) -> u32 {
    self.evict_candidates(Some(stop_at))
}

/// Shared implementation. When `stop_at` is `Some(n)`, the
/// eviction loop exits early once `n` evictions have been
/// performed.
fn evict_candidates(&mut self, stop_at: Option<usize>) -> u32 {
    let candidates = evict::scan_for_candidates(self);
    let mut evicted: u32 = 0;

    for v_id in candidates {
        // Budget-bounded early exit.
        if let Some(limit) = stop_at {
            if evicted as usize >= limit {
                break;
            }
        }

        // Re-verify eligibility: the slot may have been
        // invalidated by a prior eviction in this batch.
        if !self.vnodes.is_occupied(v_id.index()) { continue; }
        match &self.vnodes.get(v_id.index()).kind {
            VKind::Entry { gnode, is_evictable, .. } => {
                if !is_evictable { continue; }
                if *gnode == self.g_root { continue; }
                let depth = vtree::v_depth(&self.vnodes, v_id);
                if depth <= self.live_depth_evict { continue; }
            }
            VKind::Structural { .. } => continue,
        }

        evict::evict_tip(self, v_id);
        evicted += 1;
    }

    if evicted > 0 {
        let new = rebalance::rebalance(
            &mut self.vnodes,
            &mut self.gnodes,
            &mut self.violations,
            self.live_depth_evict,
        );
        self.handle_legacy_promotes(&new);
        self.repair_p_i4();
        self.normalize_plateaus();
        self.repair_p_i4();
    }
    evicted
}
```

## Consequences

- **Hot path cost (under soft limit):** O(1) — two comparisons,
  both well-predicted. Zero scan overhead when tree is within
  bounds.
- **Hot path cost (over soft limit):** O(excess + violations) —
  scan and evict only what's needed, then rebalance once.
- **No scan when `budget: None`:** users opting out of lifecycle
  management pay nothing. They can call `check_evictions()` manually
  when desired.
- **Two-phase scan avoids tree mutation during DFS:** the candidate
  list is a snapshot; eviction loop re-verifies before acting.
- **Single trailing rebalance:** amortizes violation resolution
  across all evictions in the pass.
- **Return value enables multi-round contraction:** caller can loop
  `while check_evictions() > 0 {}` for full contraction if desired
  (§IDEA M-12.7 tides).
- **Excess-count early exit is configurable:** automatic eviction
  from `observe()` defaults to bounded
  (`check_evictions_bounded(excess)`), doing minimal work. Setting
  `Config.bounded_eviction = false` makes it perform a full pass
  instead. The public `check_evictions()` always performs a full
  pass regardless.
- **Soft limit, not raw budget, is the trigger:** ADR-M-018's dynamic
  soft limit ensures that the hard budget `H` is never breached,
  even under transient overshoot from splits and gate convergence.
  The eviction scan fires at `soft_limit = H − headroom`, leaving
  room for the worst-case transient.
- **Ghost fast path is normative (§IDEA M-12.9.3(G)):** Ghost evictions
  ($g.\text{sum} = 0$) must skip Steps 3–4 of §IDEA M-12.5 (V-entry
  intensity update and ancestor violation walk). Without this,
  ghost evictions inflate total contraction work from
  $O(|G|_0 \cdot h_V)$ to $O(|G|_0 \cdot h_V^2)$. The
  `evict_tip` implementation must detect `g.sum == zero` and
  bypass propagation — see §IDEA M-12.5 Step 2. _(Not yet implemented;
  current `evict_tip` unconditionally executes Steps 3–4. The
  optimisation is deferred until contraction benchmarks confirm
  the quadratic inflation in practice.)_
- **Phase 2 depth re-verification benefits from caching
  ([ADR-M-029](029-opportunistic-depth-caching.md)):** each
  `v_depth()` call walks to the V-root at O(h_V). With cached
  depth, re-checks drop to O(1), making total Phase 2 cost
  dominated by eviction proper (absorption + V-propagation).
