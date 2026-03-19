# ADR-M-031: Sorted Placement Order — `normalize_plateaus` Elimination

**Status:** Implemented  
**Date:** 2026-03-03  
**Phase:** 6 (Hardening)  
**Relates to:** [ADR-M-026](026-point-query-and-plateau-semantics.md) (plateau semantics),
[ADR-M-030](030-graph-module-decomposition.md) (graph module decomposition)  
**Spec:** §IDEA M-5.6 (plateaus), §IDEA M-5.6.1 (plateau basis),
§IDEA M-5.6.7 (live projection — incremental maintenance contract),
§IDEA M-8 (observation flow), §IDEA M-10 (contour refinement),
§IDEA M-11.6 (legacy promote), Chapter 12 (contour simplification)  
**API:** None (internal optimisation — no public surface change)  
**Surface:** 3 (Emulsion)

---

## Context

`normalize_plateaus()` ran unconditionally at the end of every
`observe()` call. It rebuilds the plateau basis from scratch: DFS
collection → sort → left-to-right sweep → `plateau_basis.rebuild()`
→ `consolidate_all_basis()`. Cost is **O(B log B)** where B is the
basis element count.

The spec's observation flow (§IDEA M-8) defines seven steps —
route, accumulate, V-entry update, G-sum propagation, contour
refinement, rebalance, and eviction. The normalize pass is not
among them. The spec's incremental maintenance contract (§IDEA M-5.6.7)
expects `observe()` to run in O(depth) for plateau maintenance,
with O(log P) per structural mutation. The O(B log B) normalize
is an implementation artefact that compensates for order-dependent
behaviour in `place_basis_element`.

The implementation's `observe()` pipeline (in `observe.rs`) extends
the spec's seven steps with three additional implementation steps:

| Impl step | Spec step | Description                                       |
| --------- | --------- | ------------------------------------------------- |
| 1         | 1         | Route to receiver                                 |
| 2a–2b     | 2–3       | Accumulate (G-node own + V-entry intensity)       |
| 3         | 4         | Recompute G-sums upward (ADR-M-012)                 |
| 4         | —         | Plateau sum maintenance (`plateau_after_observe`) |
| 5         | 5         | Attempt contour refinement                        |
| 6         | 6         | Rebalance (drain violation queue)                 |
| 7         | —         | Dynamic depth control (ADR-M-017)                   |
| 8         | 7         | Budget-guarded eviction                           |
| 9         | —         | Normalize plateau map **(this ADR)**              |
| 10        | —         | P-I4 thatch-hop repair                            |

Steps 4, 7, 9, and 10 are implementation additions not present in
the spec's §IDEA M-8 pseudocode.

### Root Cause

`place_basis_element` merges with **immediate left/right BTreeMap
neighbours** only (see `graph_plateau.rs`
`place_basis_element`). When multiple elements are placed in
non-spatial order (e.g. right-to-left, or interleaved), same-depth
adjacent plateaus may fail to merge — producing **non-contiguous
splits** that violate P-I2 (minimal basis, §IDEA M-5.6.1).

Example: three basis elements at coordinates 2, 6, 10, all at
depth 3. If placed in order [2, 10, 6], element 10 cannot see that
6 (not yet placed) would bridge it to 2's plateau. Two plateaus
result where one is correct.

### Callers that produce out-of-order placement

| Caller                          | Order issue                                                                                                                              |
| ------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| `plateau_after_bootstrap_split` | ✗ — 1–2 elements, always left-to-right                                                                                                   |
| `plateau_after_catalytic_split` | **Yes** — displaced siblings collected bottom-up in ancestor walk                                                                        |
| `plateau_after_legacy_promote`  | **Yes** — existing child placed before new child regardless of spatial position                                                          |
| `plateau_after_evict`           | **Yes** — parent placed first, may not be leftmost                                                                                       |
| `place_subtree_basis_elements`  | ✗ — recursive left-then-right DFS (now `#[allow(dead_code)]`; all callers migrated to `collect_subtree_basis_elements` + `place_sorted`) |
| `split_for_p_i4`                | ✗ — standalone insert into the BTreeMap, not via `place_basis_element`                                                                   |

## Decision

Reduce the cost of `normalize_plateaus()` on the `observe()` hot
path by combining two strategies:

1. **Sorted placement** — all callers of `place_basis_element` that
   place multiple elements now sort by `BasisEdge` (§IDEA M-5.6.5) first,
   so the existing left-neighbour merge is sufficient. This reduces
   the frequency of plateau map divergence.

2. **Dirty flag** — a `plateaus_dirty: bool` field on `GvGraph`
   gates `normalize_plateaus()` to an **O(1) early return** when no
   structural changes (splits, promotes, evictions) have occurred.
   Simple observations that only accumulate values skip normalize
   entirely.

`normalize_plateaus()` is retained at implementation step 9 because
cross-step interactions within a single `observe()` call (split →
rebalance → legacy promote) create ordering dependencies that
sorted placement of individual callers cannot fully resolve.
However, the dirty flag ensures it is a no-op for the common case
(observation without structural mutation).

This brings the implementation closer to the spec's incremental
maintenance contract (§IDEA M-5.6.7): O(depth) for non-structural
observations, O(B log B) only when structural changes actually
occur.

### Root plateau depth fix

During implementation, a pre-existing bug was discovered: the root
plateau was initialised with `depth: 0` in `GvGraph::new()`. For
non-power-of-2 domains (e.g. `u8` with N=8, `domain_max = 255`),
`gnode_depth_from_interval(0, 255, 8)` returns 1, not 0.
Previously, `normalize_plateaus()` running on the very first
`observe()` masked this by recomputing the correct depth. With the
dirty flag, simple observations no longer trigger normalize,
exposing the mismatch. Fixed by computing the correct depth at
init time via `gnode_depth_from_interval`.

## Implementation

### Phase 1: Infrastructure

#### 1a. `collect_subtree_basis_elements`

A read-only counterpart to `place_subtree_basis_elements` that
collects `(GNodeId, u32)` pairs into a `Vec` without placing.
Follows the same DFS logic as `build_plateaus()`: terminals and
semi-internals produce elements directly; internal nodes are
kept as single elements when their subtree contour is uniform
(via `uniform_contour_depth_of`), otherwise decomposed
recursively.

```rust
// graph_plateau.rs
#[cfg(feature = "dynamic-contour-tracking")]
pub(crate) fn collect_subtree_basis_elements(
    &self,
    gid: GNodeId,
    out: &mut Vec<(GNodeId, u32)>,
) {
    use crate::gnode::GState;
    let g = self.gnodes.get(gid.index());
    match g.state() {
        GState::Terminal | GState::SemiInternal => {
            let depth = crate::gtree::gnode_depth_from_interval(g.lo, g.hi, N);
            out.push((gid, depth));
        }
        GState::Internal => {
            if let Some(ud) = uniform_contour_depth_of(&self.gnodes, gid, N) {
                out.push((gid, ud));
            } else {
                if let Some(l) = g.left {
                    self.collect_subtree_basis_elements(l, out);
                }
                if let Some(r) = g.right {
                    self.collect_subtree_basis_elements(r, out);
                }
            }
        }
    }
}
```

This is a pure read of the G-tree — no plateau mutation. Can be
tested independently.

#### 1b. `place_sorted`

A thin wrapper that sorts a collected slice by `BasisEdge` (§IDEA M-5.6.5)
and places left-to-right:

```rust
#[cfg(feature = "dynamic-contour-tracking")]
pub(crate) fn place_sorted(&mut self, elements: &mut [(GNodeId, u32)]) {
    use crate::plateau::basis_edge_of;
    elements.sort_by(|a, b| {
        let a_key = basis_edge_of(self.gnodes.get(a.0.index()));
        let b_key = basis_edge_of(self.gnodes.get(b.0.index()));
        a_key.cmp(&b_key)
    });
    for &(gid, depth) in elements.iter() {
        self.place_basis_element(gid, depth);
    }
}
```

With sorted placement the existing left-neighbour merge in
`place_basis_element` is sufficient — no post-hoc normalize is
needed for that caller in isolation.

### Phase 2: Observe-path callers

#### 2a. `plateau_after_catalytic_split`

**Before:** displaced siblings placed via
`place_subtree_basis_elements` in ancestor-walk order, then target
placed last.

**After:** collect displaced subtrees + target into one vec, sort
by `BasisEdge`, place left-to-right.

```rust
// Step 3+4 combined — collect all, sort, place.
let mut to_place = Vec::new();
for &sib_id in &displaced {
    self.collect_subtree_basis_elements(sib_id, &mut to_place);
}
if left_depth == right_depth {
    to_place.push((g_id, left_depth));
} else {
    to_place.push((left_id, left_depth));
    to_place.push((right_id, right_depth));
}
self.place_sorted(&mut to_place);
```

**Elements affected:** typically 1–5 (one sibling per ancestor
level, plus the target).

#### 2b. `plateau_after_legacy_promote` → `plateau_after_legacy_promotes_batched`

**Before:** the per-promote `plateau_after_legacy_promote` placed
the existing child first, then the new child — which can be
right-before-left.

**After:** a **batched** variant,
`plateau_after_legacy_promotes_batched`, is called once from
`handle_legacy_promotes` with all new G-nodes. This prevents
cross-promote ordering issues when multiple legacy promotes occur
within one rebalance pass.

The batched method uses two phases:

1. **Remove** — iterate all promotes, remove each parent from the
   plateau basis, and collect displaced elements (existing child +
   new child) into one vec.
2. **Sort + place** — sort the entire vec by `BasisEdge` and place
   left-to-right in a single pass via `place_sorted`.

```rust
pub(crate) fn plateau_after_legacy_promotes_batched(
    &mut self,
    new_gnodes: &[GNodeId],
) {
    if new_gnodes.is_empty() { return; }
    self.plateaus_dirty = true;

    let mut to_place = Vec::new();

    // Phase 1: remove parents, collect displaced elements.
    for &new_gid in new_gnodes {
        let ng = self.gnodes.get(new_gid.index());
        let parent_id = ng.parent.expect("...");
        let child_depth = gnode_depth_from_interval(ng.lo, ng.hi, N);
        let pg = self.gnodes.get(parent_id.index());
        let existing_child_id =
            if pg.left == Some(new_gid) { pg.right } else { pg.left };

        if let Some(old_key) = self.plateau_basis.remove(parent_id) {
            self.fixup_plateau(old_key);
        }
        if let Some(ec_id) = existing_child_id {
            let ec = self.gnodes.get(ec_id.index());
            if ec.state() != GState::Internal
                && self.plateau_basis.plateau_key(ec_id).is_none()
            {
                let existing_depth =
                    gnode_depth_from_interval(ec.lo, ec.hi, N);
                to_place.push((ec_id, existing_depth));
            }
        }
        to_place.push((new_gid, child_depth));
    }

    // Phase 2: sort all collected elements, place left-to-right.
    self.place_sorted(&mut to_place);
}
```

**Rationale for batching:** the original per-promote approach
processed each legacy promote individually. When multiple promotes
occur in one rebalance pass (§IDEA M-11.8), a promote's placement can
interleave spatially with elements from prior promotes, producing
the same out-of-order placement problem that sorted placement was
designed to fix. Batching all promotes into a single sort+place
pass eliminates this cross-promote interaction.

The singular `plateau_after_legacy_promote` is retained as
`#[allow(dead_code)]` for potential future single-promote callers.

**Elements affected:** ~2 per promote × number of promotes.

#### 2c. Gate `normalize_plateaus()` behind dirty flag

**Original plan:** delete the normalize call and `repair_p_i4` at
implementation steps 9–10 entirely.

**Actual:** fully removing normalize from `observe()` was not
feasible. Cross-step interactions within a single `observe()` call
— specifically split (step 5) → rebalance (step 6) → legacy
promote — produce ordering dependencies that sorted placement of
individual callers cannot resolve in isolation. Four decay tests
exposed the divergence.

**Applied change:** normalize is **retained** at implementation
step 9 but **gated behind the `plateaus_dirty` flag** (Phase 4).
Structural mutation hooks (`plateau_after_bootstrap_split`,
`plateau_after_catalytic_split`,
`plateau_after_legacy_promotes_batched`) set `plateaus_dirty = true`
so the normalize runs only when structural changes actually
occurred. Simple observations (accumulate + propagate, no split or
promote) skip normalize entirely — **O(1)**.

`repair_p_i4()` is also retained at step 10, since normalization
can re-key basis elements, re-introducing thatch-hop violations
(P-I4, §IDEA M-5.6.3).

```rust
// observe.rs steps 9–10
// 9. [ADR-M-031] Normalize plateau map — gated behind dirty
//    flag so it's O(1) no-op for simple observations that
//    don't trigger structural changes (splits / promotes).
self.normalize_plateaus();

// 10. Re-run P-I4 repair — normalization may re-key basis
//     elements, re-introducing thatch-hop violations.
self.repair_p_i4();
```

### Phase 3: Eviction path

#### 3a. `plateau_after_evict`

**Before:** parent placed first (as spatial boundary), displaced
sorted by `.lo`, then a tiling repair in the old Step 6.

**After:** merge parent into the displaced vec, collect all basis
elements via `collect_subtree_basis_elements`, sort by `BasisEdge`,
place left-to-right. With sorted left-to-right placement, elements
are naturally placed into the correct plateaus — the old tiling
repair step is unnecessary.

```rust
// evict.rs — Step 4+5 combined (ADR-M-031 §3a).
let parent_depth = match parent_state_after {
    GState::Terminal | GState::SemiInternal =>
        gnode_depth_from_interval(parent_lo, parent_hi, N),
    GState::Internal => unreachable!("..."),
};
let mut to_place = Vec::new();
to_place.push((parent_id, parent_depth));
for &sib_id in &displaced {
    graph.collect_subtree_basis_elements(sib_id, &mut to_place);
}
graph.place_sorted(&mut to_place);
```

**Batch eviction:** multiple `evict_tip` calls in
`evict_candidates` each run `plateau_after_evict` independently.
The trailing `normalize_plateaus()` in `evict_candidates` is kept,
gated behind the dirty flag, to handle cross-eviction interaction.

#### 3b. `evict_candidates` trailing normalize

Kept behind the dirty flag (set when `evicted > 0`). Also runs
`repair_p_i4()` after normalize to fix any thatch-hop violations
introduced by re-keying.

### Phase 4: Dirty flag (primary gating mechanism)

Added a `plateaus_dirty: bool` field to `GvGraph`. This is the
primary mechanism that makes `normalize_plateaus()` O(1) in the
common case.

```rust
#[cfg(feature = "dynamic-contour-tracking")]
pub(crate) plateaus_dirty: bool,  // field on GvGraph
```

**Set in** (any path that may invalidate the plateau map):

- `plateau_after_bootstrap_split` — contour refinement (§IDEA M-10.3)
- `plateau_after_catalytic_split` — contour refinement (§IDEA M-10.2)
- `plateau_after_legacy_promotes_batched` — contour restoration (§IDEA M-11.6)
- `evict_candidates` — contour simplification (§IDEA M-12), when `evicted > 0`
- `decay_uniform` / `decay_selective` — before their normalize calls

**Checked in:**

- `normalize_plateaus` — early return if `!self.plateaus_dirty`,
  then clears the flag before proceeding

**Initialised:** `false` in `GvGraph::new()`.

This means:

- Simple observations (no split, no promote, no eviction):
  dirty flag stays `false` → normalize is O(1) no-op.
- Observations that trigger structural mutations:
  dirty flag set → normalize runs once at step 9,
  then flag cleared.

## Phasing and Rollout

All phases were implemented and landed together.

| Phase | Scope                                             | Status                                         |
| ----- | ------------------------------------------------- | ---------------------------------------------- |
| 1     | `collect_subtree_basis_elements` + `place_sorted` | ✅ Implemented                                 |
| 2a    | Catalytic split sorted placement                  | ✅ Implemented                                 |
| 2b    | Legacy promote → batched sorted placement         | ✅ Implemented (batched variant)               |
| 2c    | Gate normalize in `observe()` behind dirty flag   | ✅ Implemented (retained + gated, not removed) |
| 3a    | Evict sorted placement + old tiling step removal  | ✅ Implemented                                 |
| 3b    | Evict trailing normalize behind dirty flag        | ✅ Implemented (kept, gated)                   |
| 4     | Dirty flag on all structural mutation paths       | ✅ Implemented                                 |
| —     | Root plateau depth init fix                       | ✅ Implemented (discovered during testing)     |

## Verification

### Existing safety nets (retained)

- `debug_assert_plateau_mirror_consistency("POST-OBSERVE")` —
  compares dynamic mirror to `build_plateaus()` rebuild on every
  `observe()` in debug builds. Catches any divergence.
- `debug_check_plateau_sums` — verifies `plateau.sum == Σ basis.sum`
  after every placement.
- `check_p_i3_only` — inline P-I3 (tile disjointness, §IDEA M-5.6.4)
  check in eviction loop.
- Full invariant checker in `invariants.rs` — available for
  targeted testing.

### New verification

- Unit tests for `collect_subtree_basis_elements` — verify it
  produces the same set as the DFS in `build_plateaus`.
- Property test: for a random sequence of observations, assert
  that the dynamic mirror matches `build_plateaus()` at every
  step — this already exists via `debug_assert_plateau_mirror_consistency`
  but should be exercised with larger randomised test cases.
- Benchmark: compare `observe()` throughput before and after
  Phase 2c on a workload with B > 1000 basis elements.

## Complexity Analysis

### Before

Every `observe()` call:

- `plateau_after_observe`: O(depth_G) — sum propagation
- `plateau_after_*_split`: O(k) placements, k ∈ {0, 1, 2, …, ~5}
- `plateau_after_legacy_promote`: O(k) placements, k ∈ {0, 1, 2}
- `normalize_plateaus`: **O(B log B)** — unconditional
- `repair_p_i4`: O(B_semi + V × log P)
- `consolidate_all_basis`: O(B) — inside normalize

Total: **O(B log B)** dominated by normalize.

### After

Every `observe()` call:

- `plateau_after_observe`: O(depth_G) — unchanged
- `plateau_after_*_split`: O(k log k) — sorted placement, k ≤ ~5
- `plateau_after_legacy_promotes_batched`: O(m log m) — m = 2×promotes
- `normalize_plateaus`: **O(1) when not dirty** (common case);
  O(B log B) when structural changes occurred
- `repair_p_i4`: O(B_semi + V × log P) — unchanged

For the **common case** (no split, no promote, no eviction):
**O(depth_G)** — normalize is gated out, approaching the spec's
incremental maintenance target (§IDEA M-5.6.7).

For **structural-change observations** (split triggers rebalance):
**O(B log B)** — normalize still runs, but sorted placement
reduces the number of non-contiguous splits it needs to repair.

## Alternatives Considered

### A. Dirty flag only (partially adopted)

Add `plateaus_dirty: bool`, early-return in normalize. Simpler
(~10 lines changed) but the O(B log B) cost remains on every
observation that triggers a split — the "growth phase" where every
observe splits. Does not address the root cause.

**Verdict:** Originally dismissed as insufficient alone, but in
practice the dirty flag became the **primary gating mechanism**.
Sorted placement reduces the frequency of divergence, but
cross-step interactions within `observe()` mean normalize cannot be
fully eliminated. The combination of both strategies delivers the
key benefit: **O(1) for non-structural observations**.

### B. Right-merge fixup in `place_basis_element`

After each placement, check whether the new plateau's right
neighbour should have been merged. Eliminates the need for sorted
placement.

**Verdict:** Adds a second merging axis to reason about (left +
right). Risk of cascading merges (absorbing right may expose yet
another right neighbour). More complex to prove correct. Deferred.

### D. Lazy normalization

Defer normalize to `plateaus()` read accessor. Zero observe-path
overhead.

**Verdict:** Requires interior mutability (`RefCell` / `Cell`)
since `plateaus()` is `&self`. Breaks `Sync`. Internal readers
(`repair_p_i4`, debug assertions) would all need rework. Too
invasive.

### F. Order-independent merge (union-find)

Replace BTreeMap with union-find for membership tracking.

**Verdict:** Fundamental redesign. Loses O(log P) range queries
(§IDEA M-5.6.5 floor-key lookup). Disproportionate effort for the problem
size.

## References

- §IDEA M-5.6 — Plateau definition and the bottom contour
- §IDEA M-5.6.1 — Plateau basis (P-I2: minimal deterministic basis)
- §IDEA M-5.6.3 — Thatching (P-I4: thatch one-hop)
- §IDEA M-5.6.5 — Basis edge and floor-key lookup
- §IDEA M-5.6.7 — Live projection (incremental maintenance contract)
- §IDEA M-8 — Observation flow (spec pipeline, seven steps)
- §IDEA M-10.2 — Catalytic split
- §IDEA M-10.3 — Bootstrap split
- §IDEA M-11.6 — Legacy promote (contour restoration)
- §IDEA M-12 — Contour simplification (eviction)
- ADR-M-026 — Point query and plateau semantics (P-I1 through P-I5)
- ADR-M-030 — Graph module decomposition (file layout)
- `graph_plateau.rs` — `place_basis_element`, `collect_subtree_basis_elements`, `place_sorted`, `normalize_plateaus`
- `evict.rs` — `plateau_after_evict`
- `observe.rs` — `observe()` pipeline (implementation steps 1–10)
- `decay.rs` — `decay_uniform`, `decay_selective`
- `graph.rs` — `GvGraph` struct (dirty flag field), `handle_legacy_promotes`
