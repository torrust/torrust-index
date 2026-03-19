# ADR-M-030: Graph Module Decomposition

**Status:** Complete  
**Date:** 2026-03-02 (updated 2026-03-04)  
**Phase:** 6 (Hardening)  
**Relates to:** [ADR-M-025](025-public-api-surface.md) (public API surface),
[ADR-M-026](026-point-query-and-plateau-semantics.md) (plateau semantics),
[ADR-M-009](009-trait-decomposition.md) (trait decomposition),
[ADR-M-031](031-sorted-placement-normalize-elimination.md) (sorted placement),
[ADR-M-032](032-three-surface-model.md) (three-surface model)  
**Spec:** §IDEA M-7 (depth gates), §IDEA M-15 (initialization)  
**API:** [§API M-6](../docs/api.md#6-surface-3--emulsion) (Surface 3 — Emulsion),
[§API M-2](../docs/api.md#2-three-surface-visibility-model)
(three-surface model)  
**Surface:** 3 (Emulsion)

## Context

`graph.rs` was the largest file in the crate and contained multiple
unrelated concerns. This ADR tracks its incremental decomposition
into focused sibling modules.

### Pre-decomposition state

Before any extraction work, `graph.rs` was ~4,589 lines containing
nine interleaved concerns, embedded tests, and scattered
`#[cfg(feature)]` conditional blocks.

### Current state (post Phase A–F — complete)

| File                      | Lines | Role                                                              |
| ------------------------- | ----- | ----------------------------------------------------------------- |
| `graph.rs`                | 475   | Struct, config, init, accessors                                   |
| `graph_query.rs`          | 311   | Sampling, point query, range query                                |
| `graph_extract.rs`        | 225   | PEWEI extraction, layer iterator, `Extend`, `from_observations`   |
| `graph_budget.rs`         | 251   | Budget enforcement, depth-gate adjustment, eviction orchestration |
| `graph_traits.rs`         | 45    | `SpatialRead`, `SpatialWrite`, `WeightedSampler`                  |
| `graph_plateau.rs`        | 1,530 | All `dynamic-contour-tracking` plateau code                       |
| `src/tests/graph.rs`      | 420   | Crate-internal tests needing `pub(crate)` access (24 tests)       |
| `src/tests/graph_init.rs` | 45    | Crate-internal initialization tests (2 tests)                     |
| `tests/graph_*.rs`        | 1,283 | External integration tests (6 files)                              |

> The inline `#[cfg(test)] mod tests {}` in `graph.rs` is now
> empty. The 420 lines of tests that required `pub(crate)` access
> were migrated to `src/tests/graph.rs`, and 2 initialization tests
> to `src/tests/graph_init.rs`, following the three-tier test
> organisation in [§API M-7.4](../docs/api.md#74-test-organisation):
> inline unit tests, `src/tests/` for crate-internal cross-module
> tests, and `tests/` for public-API integration tests.

### Metrics

| Metric                                  | Pre-decomposition | Post A+B | Post A–F (final) |
| --------------------------------------- | ----------------- | -------- | ---------------- |
| `graph.rs` total lines                  | ~4,589            | 1,257    | 475              |
| Sibling modules                         | 0                 | 1        | 5                |
| `#[cfg(feature)]` in `graph.rs`         | 28                | 12       | 10               |
| `#[cfg(feature)]` in `graph_plateau.rs` | —                 | 23       | 23               |
| `pub(crate)` methods in `graph.rs`      | 30+               | 6        | 2                |

### Comparison with other modules

| Module          | Lines | Responsibility                  |
| --------------- | ----- | ------------------------------- |
| `rebalance.rs`  | 1,458 | V-Tree rebalancing              |
| `invariants.rs` | 1,412 | Post-mutation invariant checker |
| `evict.rs`      | 802   | Eviction logic                  |
| `split.rs`      | 344   | Bootstrap + catalytic splits    |
| `decay.rs`      | 271   | Temporal decay                  |
| `observe.rs`    | 165   | Observation hot-path            |

At 1,257 lines `graph.rs` is no longer the largest file. Further
decomposition is optional but would improve navigability by
separating the remaining four concerns (§ Decision, Phases C–F).

### Remaining concerns in graph.rs

All concerns have been extracted. `graph.rs` now contains only:

| #   | Concern                            | Lines |
| --- | ---------------------------------- | ----- |
| 1   | Config + struct + init + accessors | ~475  |

Budget wiring, queries, extraction, and trait impls have been moved
to their respective sibling modules.

### Problems (residual)

1. **Four distinct concerns remain.** Config/init, budget wiring,
   read-side queries, and extraction/iteration are unrelated but
   share one file.

2. **Feature-conditional fields.** 12 `#[cfg(feature)]` blocks
   remain in `graph.rs` — struct field definitions, constructor
   initialization, and import lines that must stay with the struct.

3. **`evict_candidates` size.** The private `evict_candidates()`
   at L550–689 is ~140 lines, mostly debug-assertion scaffolding.
   It orchestrates `evict::scan_for_candidates` and
   `evict::evict_tip` with re-verification and trailing rebalance.

## Decision

**Decompose `graph.rs` into focused sibling modules**, following the
crate's established flat-module convention (`gnode.rs`, `vnode.rs`,
`gtree.rs`, `vtree.rs`). The `GvGraph` struct remains the central
type with `impl` blocks distributed across modules via Rust's
multi-file `impl` pattern.

### Module structure

```
src/
├── graph.rs            # GvGraph struct, Config, new(), accessors (475 lines ✅)
├── graph_plateau.rs    # Plateau tracking: basis, normalization, repair (1,530 lines ✅)
├── graph_query.rs      # sample(), get(), range_sum() (311 lines ✅)
├── graph_extract.rs    # extract(), layers(), Layers iterator, from_observations, Extend (225 lines ✅)
├── graph_budget.rs     # adjust_depth_gates(), evict wiring, handle_legacy_promotes (251 lines ✅)
└── graph_traits.rs     # SpatialRead, SpatialWrite, WeightedSampler (45 lines ✅)

src/tests/
├── graph.rs            # Crate-internal tests (pub(crate) access, 420 lines, 24 tests)
├── graph_init.rs       # Crate-internal initialization tests (45 lines, 2 tests)

tests/                  # External integration tests (public API only)
├── graph_sample.rs     # Sampling tests (204 lines)
├── graph_range.rs      # Range query tests (191 lines)
├── graph_point.rs      # Point query tests (222 lines)
├── graph_extract.rs    # Extraction tests (175 lines)
├── graph_layers.rs     # Layers iterator tests (395 lines)
└── graph_terminal.rs   # terminal_count tests (96 lines)
```

### Module responsibilities

#### graph.rs (475 lines, ✅ complete)

Retains:

- `Config<V>` struct and `validate()`
- `GvGraph<C, V, N>` struct definition (all fields)
- `new()` constructor
- Public accessors: `node_count`, `terminal_count`, `config`,
  `budget`, `g_root`, `v_root`, `total_sum`, `depth_evict`,
  `depth_create`, `depth_buffer`, `headroom`, `soft_limit`
- `pub(crate)` accessors: `gnodes`, `vnodes`
- `uniform_contour_depth_of` free function (used by
  `graph_plateau.rs` and `invariants.rs`)
- `gnode_depth()` helper (captures const-generic `N`)

> **Wiring.** Sibling modules are declared in `lib.rs`
> (e.g. `pub(crate) mod graph_plateau;`), matching the existing flat style.
> Each imports via `use crate::graph::GvGraph`.
>
> ```rust
> // lib.rs (actual excerpt)
> pub(crate) mod graph;
> pub(crate) mod graph_budget;
> pub(crate) mod graph_extract;
> pub(crate) mod graph_plateau;
> pub(crate) mod graph_query;
> pub(crate) mod graph_traits;
> ```

#### graph_plateau.rs (1,530 lines, ✅ complete)

All `dynamic-contour-tracking` feature code. Listed by visibility:

**Public methods:**

- `plateaus()` → `Cow<BTreeMap<BasisEdge<C>, Plateau<C, V>>>`
- `build_plateaus()` → `BTreeMap` (feature-gated fallback builder)
- `debug_plateau_basis()` → diagnostic dump

**`pub(crate)` methods:**

- `plateau_basis()` — basis bookkeeping accessor
- `recompute_plateau()` — recompute a single plateau's sum
- `place_basis_element()` — insert one basis element
- `collect_subtree_basis_elements()` — DFS collection (ADR-M-031)
- `place_sorted()` — left-to-right sorted placement (ADR-M-031)
- `place_subtree_basis_elements()` — collect + place subtree
- `normalize_plateaus()` — full basis rebuild (dirty-flag gated, ADR-M-031)
- `consolidate_all_basis()` — merge adjacent same-depth plateaus
- `spot_check_p_i2()` — P-I2 sampling diagnostic
- `debug_check_plateau_sums()` — sum consistency check
- `debug_assert_plateau_mirror_consistency()` — full mirror audit
- `fixup_plateau()` — fix a plateau after its key moved
- `repair_p_i4()` — drain `pending_p_i4` queue, fix thatch one-hop
- `plateau_after_observe()` — mutation hook: observation
- `plateau_after_bootstrap_split()` — mutation hook: bootstrap split
- `plateau_after_catalytic_split()` — mutation hook: catalytic split
- `plateau_after_legacy_promote()` — mutation hook: single promote
- `plateau_after_legacy_promotes_batched()` — mutation hook: batch
- `plateau_recompute_sums()` — bulk sum rewrite after decay

**Private methods:**

- `consolidate_basis_up()` — walk ancestors to merge basis
- `split_for_p_i4()` — split a plateau to satisfy P-I4
- `find_boundary_node()` — locate depth-boundary G-node

Each `pub(crate)` method has a `#[cfg(not(feature))]` no-op stub
immediately below it, keeping the conditional pairs co-located
within this one file.

#### graph_query.rs (311 lines, ✅ complete)

Read-side queries:

- `sample()` + `sample_child()` private helper
- `get()` + `clamp_to_domain()`, `trimmed_interval()` private helpers
- `range_sum()` + `range_sum_inner()` private helper

No feature conditionals.

#### graph_extract.rs (225 lines, ✅ complete)

PEWEI extraction and construction helpers:

- `extract()` → `Pewei<C, V>`
- `layers()` → `impl Iterator<Item = (usize, Node<C, V>)>`
- `Layers` iterator struct and `Iterator` impl
- `from_observations()` constructor
- `impl Extend<(C, O)> for GvGraph` (moved here, coupled to
  `from_observations`)

No feature conditionals.

#### graph_budget.rs (251 lines, ✅ complete)

Budget enforcement and eviction orchestration:

- `handle_legacy_promotes()` — wire legacy-promoted G-nodes
  into plateau tracking
- `adjust_depth_gates()` — dynamic D_create / D_evict (ADR-M-017)
- `check_evictions()` — public unbounded eviction (§API M-5.2)
- `check_evictions_bounded()` — `pub(crate)` bounded eviction
- `evict_candidates()` — private two-phase collect-then-evict

Note: The actual eviction logic (`evict_tip`, `scan_for_candidates`)
lives in `evict.rs`. This module provides the `GvGraph`-level
wiring with trailing rebalance and plateau repair.

Contains 2 `#[cfg(feature)]` blocks (the `plateaus_dirty = true`
assignment in `evict_candidates` and a `debug_assertions`
pre-eviction snapshot).

#### graph_traits.rs (45 lines, ✅ complete)

Trait implementations (ADR-M-009, §API M-5.5):

- `impl SpatialRead for GvGraph`
- `impl SpatialWrite for GvGraph`
- `impl WeightedSampler for GvGraph`

`impl Extend` moved to `graph_extract.rs` (coupled to
`from_observations`). Thin delegation only.

### File impact summary

| File            | Change             | Reason                                                   |
| --------------- | ------------------ | -------------------------------------------------------- |
| `graph.rs`      | **Major refactor** | Decomposition target                                     |
| `lib.rs`        | **Minor**          | Add `mod` declarations for new sibling modules           |
| `observe.rs`    | **None**           | Already imports `GvGraph`, no change                     |
| `split.rs`      | **Minor**          | May need to import from `graph_plateau` for hooks        |
| `evict.rs`      | **None**           | Already self-contained                                   |
| `decay.rs`      | **Minor**          | Imports `plateau_recompute_sums` from `graph_plateau`    |
| `invariants.rs` | **Minor**          | Imports `uniform_contour_depth_of` — stays in `graph.rs` |
| `rebalance.rs`  | **None**           | No direct `graph.rs` dependency                          |
| `plateau.rs`    | **None**           | Types only, no cross-module methods                      |

### Migration strategy

1. **Phase A: Extract tests** ✅ Complete
   - 1,283 lines moved to 6 external integration test files in
     `tests/graph_*.rs` (public API surface only):
     `graph_sample.rs` (204), `graph_range.rs` (191),
     `graph_point.rs` (222), `graph_extract.rs` (175),
     `graph_layers.rs` (395), `graph_terminal.rs` (96).
   - 420 lines of tests requiring `pub(crate)` access moved to
     `src/tests/graph.rs` (crate-internal, 24 tests).
     These tests mutate private fields (`node_count`,
     `live_depth_evict`, `live_depth_create`, `violations`) or
     call `pub(crate)` methods (`adjust_depth_gates`,
     `check_evictions_bounded`, `gnode_depth`).
   - 45 lines of initialization tests moved to
     `src/tests/graph_init.rs` (crate-internal, 2 tests).
   - Follows the three-tier test model (§API M-7.4).

2. **Phase B: Extract `graph_plateau.rs`** ✅ Complete (1,530 lines)
   - All `dynamic-contour-tracking` code isolated.
   - 23 `#[cfg(feature)]` blocks in `graph_plateau.rs`;
     12 remain in `graph.rs` (struct fields + constructor init +
     imports).
   - Wired as sibling module via `lib.rs`
     (`pub(crate) mod graph_plateau;`).
   - `graph.rs` reduced from ~4,589 → 1,257 lines (combined
     effect of Phase A test extraction + Phase B plateau
     extraction).

3. **Phase C: Extract `graph_query.rs`** (311 lines) — ✅ Complete
   - Self-contained read operations, no feature conditionals.
   - Moved `sample`, `get`, `range_sum` and private helpers.

4. **Phase D: Extract `graph_extract.rs`** (225 lines) — ✅ Complete
   - `Layers` iterator, `from_observations`, and `Extend` impl.
   - Moved `extract`, `layers`, `Layers` struct + `Iterator` impl.

5. **Phase E: Extract `graph_budget.rs`** (251 lines) — ✅ Complete
   - Eviction wiring + depth gate adjustment.
   - Moved `handle_legacy_promotes`, `adjust_depth_gates`,
     `check_evictions`, `check_evictions_bounded`, `evict_candidates`.

6. **Phase F: Extract `graph_traits.rs`** (45 lines) — ✅ Complete
   - `SpatialRead`, `SpatialWrite`, `WeightedSampler` impls.
   - `Extend` moved to `graph_extract.rs` instead.

Each phase is independently testable via
`cargo test --workspace --all-targets --all-features`.

## Consequences

### Positive

- **Improved navigability.** Each file has a single concern.
- **Feature isolation.** `dynamic-contour-tracking` code lives in
  one place; disabling the feature doesn't leave stubs littered
  throughout the main file.
- **Smaller diffs.** Changes to plateau logic touch only
  `graph_plateau.rs`.
- **Parallel development.** Multiple contributors can work on
  different modules without merge conflicts.
- **Test organization.** Three-tier model (§API M-7.4): inline
  unit tests, `src/tests/` for crate-internal, `tests/` for
  integration.

### Negative

- **More files.** Up to 6 source modules instead of 1. Navigation
  requires knowing which module owns which method.
- **Cross-module `impl` blocks.** Rust allows `impl GvGraph` in
  multiple files, but IDE "go to definition" may land in the wrong
  file initially.
- **Import churn.** Internal callers may need updated `use` paths
  during migration.

### Neutral

- **No public API change.** `lib.rs` re-exports remain identical.
  All decomposed modules are Surface 3 (Emulsion).
- **No performance change.** Module boundaries are compile-time only.
- **Line count unchanged.** Total code volume is preserved; only
  organization changes.

## Alternatives Considered

### A. Leave graph.rs as-is

**Rejected** (for Phase B). At ~4,589 lines pre-decomposition, the
file was 27% of the crate. Post-Phase-B, at 1,257 lines, this
alternative becomes more tenable for the remaining concerns — further
extraction is beneficial but not urgent.

### B. Move plateau code into plateau.rs

**Partially accepted.** Types (`Plateau`, `BasisEdge`,
`PlateauBasis`) live in `plateau.rs`. The _mutation hooks_
(`plateau_after_observe`, etc.) are `impl GvGraph` methods requiring
`&mut self` access to arenas. These belong in `graph_plateau.rs`
which imports from both `graph` and `plateau`.

### C. Use inline modules (`mod plateau { ... }` inside graph.rs)

**Rejected.** Inline modules don't reduce file length — they add
indentation. The goal is physical separation for editor navigability
and git diff clarity.

### D. Create a graph/ directory with mod.rs

**Considered but deferred.** A `graph/mod.rs` with submodules is
cleaner for deep hierarchies. For 5–6 siblings, flat `graph_*.rs`
files are simpler and match the existing crate style. If future
growth warrants, `graph_*.rs` files can be moved into a `graph/`
directory with a one-line `mod.rs`.

## Open Questions

### Q1: Should `uniform_contour_depth_of` move to gtree.rs? (DC-030-1)

The function operates on `Arena<GNode>` without touching V-nodes.
Logically it's a G-Tree query. Moving it to `gtree.rs` would reduce
`graph.rs` imports in `invariants.rs`.

Used by:

- `graph_plateau.rs` (basis computation)
- `invariants.rs` (P-I2 checks)

**Decision:** Move to `gtree.rs` and re-export from `graph.rs` for
backward compatibility.

**Status:** Not yet implemented — still defined in `graph.rs` at
L97.

### Q2: Should tests/ use a shared helper module? (DC-030-2)

The `src/tests/graph.rs` tests define repeated helpers:

- `default_config()`
- `budget_config(budget)`
- `build_range_tree()`
- `build_evictable_graph()`
- `TestLcgRng`, `FixedRng`, `SeqRng`

These could live in `src/tests/common.rs` or a shared helper.

**Decision:** Extract shared fixtures when the pattern recurs in
further graph test files.

**Status:** Not yet implemented.

### Q3: Should `gnode_depth()` stay in graph.rs or move? (DC-030-3)

The method is a 3-line wrapper around
`gtree::gnode_depth_from_interval` that captures the const-generic
`N`. Used by queries, extraction, and budget code.

**Decision:** Keep in `graph.rs` as a shared accessor — it depends
on `self.gnodes` and the const-generic `N`, and is needed by
multiple future sibling modules.

**Status:** ✅ Implemented as decided — remains in `graph.rs` at
L462.

## References

- §IDEA M-5.6: Plateaus and bottom contour
- §IDEA M-7: Depth gates
- §IDEA M-15: Initialization
- §API M-2: Three-surface visibility model (ADR-M-032)
- §API M-7.4: Test organisation (three-tier model)
- §API M-5.5: Trait implementations on GvGraph
- §API M-6: Surface 3 — Emulsion
- ADR-M-025: Public API surface
- ADR-M-026: Plateau semantics and point query
- ADR-M-031: Sorted placement / normalize elimination
- ADR-M-032: Three-surface visibility model
- Rust by Example: [Splitting modules](https://doc.rust-lang.org/rust-by-example/mod/split.html)
