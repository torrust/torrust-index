# Mudlark — Implementation Architecture

How the formal specification (`idea.md`) maps to Rust modules,
types, and operations.

---

## 1. Governing model: Three surfaces (ADR-M-032)

Every public symbol belongs to exactly one of three surfaces.
The compiler enforces the boundary: Surface 3 modules are
`pub(crate)`, so downstream crates cannot depend on internal
machinery.

| Surface | Name     | Rust visibility                                   | Role                                                         |
| ------- | -------- | ------------------------------------------------- | ------------------------------------------------------------ |
| 1       | Prints   | `pub(crate)` modules, types re-exported from root | Lightweight `Copy`/`Clone` view types users hold and inspect |
| 2       | Film     | `pub(crate)` modules, types re-exported from root | Opaque operational types users interact through              |
| 3       | Emulsion | `pub(crate)`                                      | Internal machinery — not part of the public API              |

### Crossing rules

1. **Print → Film.** Film methods _return_ Prints. A Print never
   holds a `&mut` back into the Film.
2. **Film → Emulsion.** Every Film method is a thin façade
   delegating to Emulsion functions. Public signatures mention
   only Surface 1 and 2 types.
3. **Emulsion → Print.** Emulsion code _constructs_ Prints
   internally via struct literals (`pub` fields).
4. **Emulsion ⇛ Film fields.** Emulsion code accesses `GvGraph`
   fields directly (`pub(crate)`), never through the public trait
   interface.

### Exceptions

Two modules are `pub` + `#[doc(hidden)]` as testing affordances
(ADR-M-032):

- `invariants` — `assert_invariants()` for integration tests.
- `testing` — shared test infrastructure (config presets, plan
  runner, fluent builder, RNG stubs).

---

## 2. Crate layout

### Source modules

```
src/
├── lib.rs              # Crate root: mod declarations, flat re-exports
│
│   ── Surface 1 — Prints ──────────────────────────├── contour_range.rs    # BasisElement, ContourRange, ContourRangeEnergy;
│                       #   validate_endpoints, compute_plateau_energy (ADR-M-037)├── gnode.rs            # GState (pub), GNode (pub(crate)): Surface 1 + 3
├── handle.rs           # GNodeId, VNodeId: NonZeroU32 niche-optimized handles
├── pewei.rs            # Pewei, Layer, Transition, Terminal: PEWEI extraction
├── plateau.rs          # BasisEdge, Plateau, PlateauBasis: contour types
├── view.rs             # Span, Cell, Node: snapshot view types
│
│   ── Surface 2 — Film ────────────────────────────
├── graph.rs            # GvGraph struct, Config, GNodeInfo, new(), accessors
├── traits/             # Chemistry contracts and instrument traits (ADR-M-009, ADR-M-033)
│   ├── mod.rs          # Re-exports, module-level docs
│   ├── accumulator.rs  # Accumulator trait (ordered commutative monoid)
│   ├── attenuatable.rs # Attenuatable sub-trait (decay scaling)
│   ├── coordinate.rs   # Coordinate trait (spatial domain type)
│   ├── inspectable.rs  # Inspectable sub-trait (diagnostic f64 projection)
│   ├── observation.rs  # Observation trait (photon → accumulator delta)
│   ├── proratable.rs   # Proratable sub-trait (fractional range queries)
│   ├── rng.rs          # Rng trait + optional rand_core blanket impl
│   ├── spatial_read.rs # SpatialRead instrument trait
│   ├── spatial_write.rs# SpatialWrite instrument trait
│   ├── temporal_decay.rs# TemporalDecay instrument trait
│   ├── weighable.rs    # Weighable sub-trait (weight projection)
│   └── weighted_sampler.rs # WeightedSampler instrument trait
│
│   ── Surface 3 — Emulsion ────────────────────────
├── arena.rs            # Arena<T>: Vec-backed slab allocator, bitset occupancy
├── decay.rs            # Subband-adaptive temporal decay (§IDEA M-14, ADR-M-024)
├── diagnostic.rs       # Consolidated audit & tracing helpers (ADR-M-028)
├── evict.rs            # Eviction: tip removal and candidate scanning (§IDEA M-12)
├── graph_budget.rs     # Budget enforcement, depth-gate adjustment, eviction
│                       #   orchestration (ADR-M-015, ADR-M-017, ADR-M-018) [Phase E]
├── graph_extract.rs    # PEWEI extraction, layer iteration, from_observations
│                       #   (ADR-M-021, ADR-M-022) [Phase D]
├── graph_plateau.rs    # Plateau tracking impl methods on GvGraph (ADR-M-026, ADR-M-031)
├── graph_query.rs      # Read queries: sample, get, range_sum, contour_range,
│                       #   contour_range_energy, select_plateaus
│                       #   (ADR-M-019, ADR-M-020, ADR-M-037) [Phase C]
├── graph_traits.rs     # Trait impls: SpatialRead, SpatialWrite, TemporalDecay,
│                       #   WeightedSampler (ADR-M-009) [Phase F]
├── gtree.rs            # G-Tree: spatial routing, sum propagation (§IDEA M-5)
├── observe.rs          # Observation hot-path: 10-step pipeline (§IDEA M-8)
├── rebalance.rs        # V-Tree rebalancing: contract, promote, resolve (§IDEA M-11)
├── split.rs            # Bootstrap and catalytic splits (§IDEA M-10)
├── vnode.rs            # VNode, VKind, PackedChildren: tournament nodes
├── vtree.rs            # V-Tree: insert, remove, depth, propagation (§IDEA M-9)
│
│   ── Testing affordances ─────────────────────────
├── invariants.rs       # Post-mutation invariant checker (pub + #[doc(hidden)])
├── testing/            # Shared test infrastructure (pub + #[doc(hidden)])
│   ├── mod.rs          # Re-exports
│   ├── builder.rs      # GraphCreator: fluent builder for test graphs
│   ├── config.rs       # default_config(), budget_config() helpers
│   ├── plan.rs         # Plan<C,V>: serializable observation sequence
│   ├── presets.rs      # Degenerate plan presets (left-deep, zigzag, etc.)
│   ├── rng.rs          # TestLcgRng, FixedRng, SeqRng stubs
│   └── runner.rs       # run(), run_checked(), run_soft() executors
│
│   ── Crate-internal tests ────────────────────────
└── tests/              # #[cfg(test)] cross-module tests (pub(crate) access)
    ├── mod.rs
    ├── arena.rs         ├── decay.rs          ├── decay_f64_depth.rs
    ├── diagnostic.rs    ├── evict.rs          ├── gnode.rs
    ├── graph.rs         ├── graph_init.rs     ├── gtree.rs
    ├── handle.rs        ├── observe.rs        ├── pewei.rs
    ├── plan_builders.rs ├── plateau.rs        ├── rebalance.rs
    ├── rebalance_stress.rs                    ├── semi_internal_plateau.rs
    ├── spiked_vtree.rs  ├── split.rs          ├── structural.rs
    ├── traits.rs        ├── view.rs           ├── vnode.rs
    ├── vtree.rs         └── worked_example.rs
```

### Integration tests (public API only)

```
tests/
├── bench_mirrors.rs        # Invariant-checked debug-mode mirrors of benchmarks
├── bench_mirrors/          # Per-family benchmark mirror modules (ADR-M-035)
│   ├── extract.rs          │   ├── lifecycle.rs
│   ├── observe.rs          │   ├── pathological.rs
│   ├── query.rs            │   └── spray.rs
├── budget.rs               ├── buffer_oscillation.rs
├── cascade.rs              ├── contour_range.rs
├── cross_type.rs           ├── eviction.rs
├── eviction_debug.rs       ├── eviction_p_i2.rs
├── graph_extract.rs        ├── graph_layers.rs
├── graph_point.rs          ├── graph_range.rs
├── graph_sample.rs         ├── graph_terminal.rs
├── hex_binary_tree_mapping.rs
├── plateau.rs              ├── sentinel_api.rs
├── stress_patterns.rs
└── support/
    └── mod.rs              # Shared integration test helpers
```

### Feature gates

| Feature                    | Default | Effect                                                              |
| -------------------------- | ------- | ------------------------------------------------------------------- |
| `dynamic-contour-tracking` | yes     | Live plateau mirror; `plateaus()` returns `Cow::Borrowed` in $O(1)$ |
| `serde`                    | yes     | `Serialize`/`Deserialize` on all Surface 1 snapshot types           |
| `rand`                     | yes     | Blanket `Rng` impl for all `rand_core::Rng` types               |

### Benchmarks (ADR-M-035)

```
benches/
├── bench_rng.rs        # Shared deterministic RNG for benchmark reproducibility
├── extract.rs          # Family 3 — PEWEI extraction
├── lifecycle.rs        # Family 4 — decay + eviction lifecycle
├── observe.rs          # Family 1 — observation hot-path
├── pathological.rs     # Family 6 — adversarial / degenerate patterns
├── query.rs            # Family 2 — sample, range_sum, point query
└── spray.rs            # Family 5 — random-spray bulk insertion
```

All benchmarks use Criterion. Each has an invariant-checked mirror
in `tests/bench_mirrors/` to catch correctness regressions in debug
mode (see ADR-M-035).

### Dependencies

| Crate           | Kind                       | Purpose                               |
| --------------- | -------------------------- | ------------------------------------- |
| `rand_core` 0.9 | Optional (`rand` feature)  | RNG trait bridge                      |
| `serde` 1       | Optional (`serde` feature) | Serialization derives                 |
| `tracing` 0     | Required                   | Structured diagnostic instrumentation |

---

## 3. Node representation

### 3.1 Storage strategy (ADR-M-001)

`Vec`-backed arenas with `NonZeroU32` niche-optimized handles.
Free list for $O(1)$ slot reuse. Separate `Vec<u64>` bitset for
occupancy tracking. `debug_assert!` guards catch stale handles
in development; compiles to bare array access in release. std-only.

### 3.2 Handle types

`GNodeId(NonZeroU32)` and `VNodeId(NonZeroU32)` — 4 bytes each.
`Option<GNodeId>` and `Option<VNodeId>` are also 4 bytes (niche
optimization). Type-safe: `GNodeId` cannot index the V-node arena.

### 3.3 GNode — spatial node (ADR-M-002)

48 bytes for `GNode<u64, u64>`. Dyadic range `[lo, hi)`,
`sum`/`own` values, G-children (`left`, `right`), `parent`,
and cross-tree V-entry link (`entry`).

`GState` is derived from child pointers at zero storage cost
(ADR-M-016): `Terminal` (0 children), `SemiInternal` (1 child),
`Internal` (2 children).

`GNodeInfo<C, V>` (Surface 1, ADR-M-036) is the read-only snapshot
of a G-node's structural state — range, own/sum, depth, state,
child IDs, and parent ID. Obtained via `GvGraph::gnode_info()`.

### 3.4 VNode — tournament node (ADR-M-002, ADR-M-029)

64 bytes for `VNode<u64>` (one cache line). Single struct with
`VKind<V>` enum discriminating Entry from Structural.

The `cached_depth: AtomicU32` field (ADR-M-029) occupies the 4-byte
padding gap between `parent` and `kind` — zero memory overhead.
`DEPTH_STALE` (= `u32::MAX`) indicates the cache needs recompute.
Uses `AtomicU32` for thread safety on the read path.

### 3.5 PackedChildren — SoA layout

`PackedChildren<V>` uses struct-of-arrays layout for cache-friendly
sampling: 3 intensities contiguous (24 bytes, hot) + 3 IDs
(12 bytes, cold) + len byte. The sampling decision reads the hot
intensities in one scan, then follows a single child ID.

### 3.6 VKind — entry vs. structural

- **Entry:** Backed by a G-node. Carries `is_exposed` (has uncovered
  range — true for terminal and semi-internal, ADR-M-027) and
  `is_evictable` (zero G-children — terminal only, ADR-M-027).
- **Structural:** Pure scaffolding with `PackedChildren<V>` (2 or 3
  children) and `has_evictable` flag for eviction scan pruning.

### 3.7 Top-level struct

```rust
pub struct GvGraph<C: Coordinate, V: Accumulator, const N: u32> {
    gnodes: Arena<GNode<C, V>>,
    vnodes: Arena<VNode<V>>,
    g_root: GNodeId,
    v_root: Option<VNodeId>,
    config: Config<V>,
    violations: Vec<VNodeId>,         // scoped to one mutation batch (ADR-M-003)
    node_count: u32,
    terminal_count: u32,              // maintained leaf counter (ADR-M-025)
    live_depth_evict: u32,            // dynamic (ADR-M-017)
    live_depth_create: u32,           // dynamic (ADR-M-017)
    depth_buffer: u32,                // D_evict − D_create, fixed at construction
    headroom: usize,                  // 3^(buffer+1) (ADR-M-018)
    soft_limit: Option<usize>,        // budget − headroom (ADR-M-018)

    // ── cfg(feature = "dynamic-contour-tracking") ──
    plateaus: BTreeMap<BasisEdge<C>, Plateau<C, V>>,   // live contour mirror (ADR-M-026)
    pending_p_i4: Vec<(GNodeId, BasisEdge<C>)>,        // thatch-hop repair queue
    plateau_basis: PlateauBasis<C>,                     // bidirectional basis bookkeeping
    plateaus_dirty: bool,                               // normalize gate (ADR-M-031)
}
```

---

## 4. The G-Tree (§IDEA M-5)

### 4.1 Routing — `gtree::route_to_receiver`

Iterative descent returning the terminal or semi-internal receiver.
Handles 0/1/2 children uniformly. Cost: $O(\text{depth})$.

### 4.2 Sum propagation — `gtree::recompute_g_sums` (ADR-M-012)

Recomputes `sum = own + left.sum + right.sum` from receiver to root.
Avoids subtraction on `Accumulator`. Self-healing under float drift.

`recompute_g_sums_subtree(gnodes, preorder)` — bulk variant for
post-decay fixup. Iterates in reverse (post-order) so children are
correct when the parent is visited. Cost: $O(|S|)$.

### 4.3 Range queries and contour range decomposition

**`range_sum`** (ADR-M-020) — Recursive G-Tree descent with pro-rated
`g.own` at partial overlaps using `f64` ratio arithmetic. Accepts any
`RangeBounds<C>`. NaN panics, out-of-range clamping, empty range
returns `V::zero()`. Cost: $O(N)$. Requires `V: Proratable`.

**`contour_range` / `contour_range_energy`** (ADR-M-037, revised) —
Decomposes a coordinate range $[s, e)$ into a **basis set**: the
minimal G-node cover whose effective tiles are pairwise disjoint and
contiguously tile the range (§CR.2).  At most two basis elements are
*boundary thatching* semi-internals whose `.sum` leaks energy outside
the range (§CR.3.2).  `contour_range` returns the full
`ContourRange<C, V>` (basis set + energy fields);
`contour_range_energy` returns only the scalar `ContourRangeEnergy<V>`.
Both share the same $O(N)$ G-Tree walk and require
`V: Proratable + Inspectable`.

**`select_plateaus`** (§CR.12) — Given arbitrary dyadic coordinates
`[lo, hi)`, snaps outward to the nearest lattice-aligned endpoints
that span all overlapping plateaus.  Returns a `(BasisEdge, BasisEdge)`
pair valid for `contour_range()`.  Cost: $O(\log P)$.  Requires
`V: Proratable + Inspectable`.

### 4.4 Depth computation — `gtree::gnode_depth_from_interval`

Computes G-Tree depth from dyadic interval width:
$\text{depth} = N - \log_2(\text{width})$. Free function usable from
`&mut self` contexts. Thin wrapper `GvGraph::gnode_depth()` captures
the const-generic `N`.

---

## 5. The V-Tree (§IDEA M-6, §IDEA M-9, §IDEA M-11)

### 5.1 Insertion — `vtree::vtree_insert`

Zero-intensity buddy-insert: descend to lightest entry, wrap in
structural node. Three cases: empty tree, single-entry root, general.
No rebalancing required.

### 5.2 Removal — `vtree::vtree_remove_leaf`

Leaf removal with structural collapse when a 2-node parent loses a
child. Handles 3→2 node contraction and root-entry removal.

### 5.3 Rebalancing — `rebalance::rebalance` (ADR-M-003)

Eager violation tracking with `Vec<VNodeId>` work queue. Ten
violation sources (§IDEA M-11.12). The `rebalance` loop drains
the queue with stale/resolved guards. Always empty when a mutation
batch returns.

Operations:

- `contract` — 3→2 node, isolate heaviest child.
- `standard_promote` — explode 2-child structural into parent.
- `skip_promote` — elevate entry/3-node past parent to grandparent.

Structured `tracing` spans and events instrument the resolve loop
at `DEBUG`/`TRACE`/`WARN`/`ERROR` levels.

### 5.4 Proportional sampling — `GvGraph::sample` (ADR-M-019)

Weighted random walk from V-root to leaf entry, choosing each child
with probability proportional to its cached intensity via
cumulative-sum scan over `PackedChildren::intensities`. Returns
`Option<Cell<C, V>>` — `None` when total intensity is zero.
Expected cost: $O(1.44\, H + 1.67)$ where $H$ is the Shannon
entropy. Requires `V: Weighable`.

### 5.5 Evictable flag propagation — `vtree::propagate_evictable_flags`

Bottom-up OR propagation of `is_evictable` / `has_evictable`.
Early-terminates when flag is unchanged.

### 5.6 Exposed / evictable flag semantics (ADR-M-027)

Two orthogonal V-entry properties govern different concerns:

| Flag           | True when                                              | Governs                                                   |
| -------------- | ------------------------------------------------------ | --------------------------------------------------------- |
| `is_exposed`   | G-node has uncovered range (terminal or semi-internal) | Observation routing, V-entry liveness, contour membership |
| `is_evictable` | G-node has zero children (terminal only)               | Eviction safety — no dependents to orphan                 |

The two-flag model (ADR-M-027) supersedes the single-flag approach
of ADR-M-013 and ADR-M-016.

### 5.7 Opportunistic depth caching (ADR-M-029)

V-Tree depth is queried in three hot paths: split gate, eviction
re-verification, and legacy promotion. Without caching, each query
walks to the V-root at $O(h_V)$. The `cached_depth: AtomicU32`
field on `VNode` caches the result; `DEPTH_STALE` invalidation
propagates to affected subtrees after structural mutations. Fits in
the existing 4-byte padding gap — zero size overhead.

---

## 6. The observation pipeline (§IDEA M-8) — `observe.rs`

The primary mutation entry-point. Ten steps (extended from the
spec's seven by plateau maintenance, normalization, and P-I4
repair):

```text
observe(coord, delta):
  1.  Route to receiver              — gtree::route_to_receiver
  2a. Accumulate g.own += Δ          — GNode direct mutation
  2b. Update V-entry intensity       — VNode + vtree::propagate_v_sums
      Violation check                — ancestor walk, push queue
  3.  Recompute G-sums upward        — gtree::recompute_g_sums (ADR-M-012)
  4.  Plateau sum maintenance        — plateau_after_observe (no-op without feature)
  5.  Attempt contour refinement     — split::attempt_split
      Post-split audit               — diagnostic::audit_violations (debug)
  6.  Drain violation queue          — rebalance::rebalance
      Handle legacy promotes         — handle_legacy_promotes + repair_p_i4
  7.  Dynamic depth control          — adjust_depth_gates (ADR-M-017)
  8.  Budget-guarded eviction        — check_evictions[_bounded] (ADR-M-015, ADR-M-018)
  9.  Normalize plateau map          — normalize_plateaus (dirty-gated, ADR-M-031)
 10.  P-I4 thatch-hop repair         — repair_p_i4
```

Steps 2b and 3 are independent. Step 6 guarantees V-I3 on return.
Steps 4, 9, and 10 are no-ops when `dynamic-contour-tracking` is
disabled. Step 9 is further gated behind a `plateaus_dirty` flag
so simple observations that don't trigger structural changes
(splits / promotes) skip the $O(B \log B)$ rebuild (ADR-M-031).

P-I2 spot-checks are interspersed between steps 5–9 in debug
builds and when a `tracing` subscriber is active at `DEBUG` level.

---

## 7. Splitting (§IDEA M-10) — `split.rs`

- **Bootstrap split** (§IDEA M-10.3): First split when V-Tree is a single
  entry. Creates root structural + child structural + 2 entries.
- **Catalytic split** (§IDEA M-10.2): General case. Parent persists as frozen
  benchmark. Adds 3rd child to V-parent (2→3 node). Violation-free.
- **Preprocessing**: If V-parent is a 3-node, contract first (isolate
  heaviest). Depth gate checked after contraction.
- **Depth gate**: `v_depth(entry) <= D_create` required for split.

---

## 8. Plateau subsystem (ADR-M-026, ADR-M-031) — `plateau.rs` + `graph_plateau.rs`

The plateau subsystem maintains a live BTreeMap mirror of the
G-Tree's bottom contour — contiguous regions of uniform terminal
depth, tiled by basis elements.

### 8.1 Types — `plateau.rs`

| Type              | Surface | Size   | Purpose                                                                         |
| ----------------- | ------- | ------ | ------------------------------------------------------------------------------- |
| `BasisEdge<C>`    | 1       | `Copy` | `Ord`-providing newtype for BTreeMap keys; delegates to `Coordinate::total_cmp` |
| `Plateau<C, V>`   | 1       | `Copy` | One plateau: thatched range, depth, sum                                         |
| `PlateauBasis<C>` | 3       | —      | Bidirectional basis ↔ edge bookkeeping (forward + reverse maps)                 |

### 8.2 Invariants

| ID   | Name           | Description                                                                                   |
| ---- | -------------- | --------------------------------------------------------------------------------------------- |
| P-I1 | Tiling         | Basis edges tile the domain `[0, 2^N)` — no gaps, no overlaps                                 |
| P-I2 | Depth          | Every basis element's contour depth matches its plateau                                       |
| P-I3 | Sum            | `plateau.sum = Σ g.sum` over basis elements                                                   |
| P-I4 | Thatch one-hop | Semi-internal basis element's uncovered half is at most one depth step from the plateau depth |

### 8.3 Incremental maintenance — `graph_plateau.rs`

At ~1,570 lines, this is the largest module. It provides `impl GvGraph`
methods for incremental plateau maintenance across every mutation path:

**Mutation hooks** (called from `observe.rs`, `split.rs`, `evict.rs`, `decay.rs`):

- `plateau_after_observe` — sum update on the receiving plateau
- `plateau_after_bootstrap_split` — contour refinement for first split
- `plateau_after_catalytic_split` — contour refinement for general split
- `handle_legacy_promotes` — contour restoration after rebalance (via `graph_budget.rs`)
- `plateau_recompute_sums` — bulk sum rewrite after decay

**Structural operations:**

- `place_basis_element` / `place_sorted` — insert basis elements (ADR-M-031)
- `collect_subtree_basis_elements` / `place_subtree_basis_elements` — DFS collection
- `normalize_plateaus` — full basis rebuild (dirty-flag gated)
- `consolidate_all_basis` — merge adjacent same-depth plateaus
- `repair_p_i4` — drain `pending_p_i4` queue, fix thatch one-hop

Each `pub(crate)` method has a `#[cfg(not(feature))]` no-op stub
keeping the conditional pairs co-located within this one file.

### 8.4 Sorted placement optimization (ADR-M-031)

The spec's incremental maintenance contract (§IDEA M-5.6.7) expects
$O(\text{depth})$ plateau maintenance per observation. The
`normalize_plateaus` pass is $O(B \log B)$. ADR-M-031 introduces
sorted placement order so that basis elements are inserted
left-to-right, reducing the frequency of full normalizations. A
`plateaus_dirty` flag gates the normalize call — simple observations
that don't trigger structural changes skip it entirely.

### 8.5 Dynamic-contour-tracking vs. on-demand build

With `dynamic-contour-tracking` (default): `plateaus()` returns
`Cow::Borrowed` — $O(1)$ borrow of the actively-maintained mirror.

Without the feature: `plateaus()` returns `Cow::Owned` — $O(G)$
on-demand build from the current G-tree state. No plateau fields
exist on the struct. All maintenance hooks compile to no-ops.

---

## 9. Eviction (§IDEA M-12) — `evict.rs`

- **Eligibility (ADR-M-013, ADR-M-027):** `is_evictable` (terminal only)
  AND `v_depth > D_evict` AND not G-root.
- **Scan:** `scan_for_candidates()` — pre-order DFS with
  `has_evictable` pruning.
- **Tip eviction (ADR-M-014):** `evict_tip()` — absorbs `child.sum`
  into `parent.own`, unlinks, propagates V-sums and evictable flags,
  removes V-entry, deallocates G-node. Energy conservation verified
  by `debug_assert`.
- **Two-phase pattern:** collect candidates, then re-verify and evict.
  Bounded mode stops after a budget-derived limit.
- **Wiring:** `observe()` triggers eviction automatically when
  `node_count > soft_limit` (ADR-M-018). Orchestration lives in
  `graph_budget.rs`.

---

## 10. Depth gates and budget (§IDEA M-7) — `graph.rs` + `graph_budget.rs`

- `D_create` — maximum V-depth for split authorization.
- `D_evict` — minimum V-depth for eviction eligibility.
- **Dynamic adjustment (ADR-M-017):** `adjust_depth_gates()` tightens
  when `node_count > soft_limit`, relaxes when
  `node_count < α_relax × soft_limit`. Buffer `D_evict − D_create`
  is fixed at construction. Floor at `buffer + 1`.
- **Hard budget guarantee (ADR-M-018):** `soft_limit = budget −
max(headroom, 2*(D_c−1))` where `headroom = 3^(buffer+1)`.
  Recomputed dynamically on every `adjust_depth_gates()` call.
  Budget is never exceeded.

---

## 11. PEWEI extraction and reconstruction — `pewei.rs` + `graph_extract.rs`

### 11.1 Extraction — `GvGraph::extract` (ADR-M-021, ADR-M-022)

BFS walk of the V-Tree producing an ordered sequence of layers.
Each V-entry is classified by backing G-node state:

- `Terminal` — leaf G-node (no G-children).
- `Transition` — internal or semi-internal G-node. Stores
  `baseline` (g.own), `total` (g.sum), `refinement` (total −
  baseline). `snr()` returns `Option<f64>`.

Empty / zero-intensity trees emit 1 layer with 1 zero-intensity
terminal. All types serde-gated. Implementation lives in
`graph_extract.rs` (extracted per ADR-M-030 Phase D).

### 11.2 Reconstruction — `Pewei::reconstruct` (ADR-M-023)

Top-down recursive descent producing `Vec<Span<C, V>>` from a
truncated `Pewei`. Visible layers contribute spans; invisible
remainder energy is pro-rated into the parent transition's span.
Uses a bandwidth-optimised `RegionLookup` with SoA depth-bucketed
layout.

### 11.3 Convenience construction — `GvGraph::from_observations`

`graph_extract.rs` also provides `GvGraph::from_observations()` —
a convenience constructor that builds a graph from an iterator of
`(coord, delta)` pairs, applying each as an observation.

---

## 12. Temporal decay — `decay.rs` (ADR-M-024)

`decay(root, attenuation, q)` — subband-adaptive temporal filter.

Per-depth factor:

$$\ln\lambda(d) = \ln(\text{att}) \cdot (1 + q \cdot (2d_{\text{local}}/D - 1))$$

Two internal paths:

- **Uniform** ($q = 0$): single factor applied to every `g.own`.
- **Selective** ($q > 0$): per-depth factor table from the
  log-linear envelope.

Both paths: scale `g.own` → recompute `g.sum` bottom-up via
`gtree::recompute_g_sums_subtree` → fix ancestor sums → sync
V-entry intensities → `vtree::recompute_all_v_intensities` →
detect and repair V-I3 violations through rebalance.

Integer truncation means `floor(sum × λ) ≠ floor(own × λ) +
floor(child.sum × λ)`, so `g.sum` is never scaled directly —
always recomputed from children. Even global uniform decay runs
rebalance (cumulative drift breaks V-I3 after repeated rounds).

Requires `V: Attenuatable`.

---

## 13. Traits and configuration

### 13.1 Trait hierarchy (ADR-M-009, ADR-M-033)

`V` is generic over types satisfying the Standard property profile (§IDEA M-2.7)
(P0–P5). The core `Accumulator` trait encodes the structural
contract — an ordered commutative monoid where `zero()` is both
the additive identity and the minimum:

```rust
pub trait Accumulator: Copy + PartialOrd + Debug + Default + Send + Sync + 'static {
    fn zero() -> Self;
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
}
```

Four independent sub-traits gate additional capabilities
(ADR-M-009 Addendum 3, ADR-M-033):

| Sub-trait      | Method(s)                      | Required for                                           |
| -------------- | ------------------------------ | ------------------------------------------------------ |
| `Attenuatable` | `attenuate(self, f64) -> Self` | `decay()` (`TemporalDecay` trait)                      |
| `Weighable`    | `weight(self) -> f64`          | `sample()` (`WeightedSampler` trait), PEWEI extraction |
| `Proratable`   | `prorate(…)`, `scale_by(…)`    | `range_sum()`, exact energy in `contour_range()` / `contour_range_energy()` |
| `Inspectable`  | `to_f64_approx(self) -> f64`   | Invariant checking, diagnostic display, `contour_range()` zero-pruning |

All seven built-in numeric types (`u8`–`u128`, `f32`, `f64`)
implement `Accumulator` and all four sub-traits. A user-defined
type needs only `Accumulator`; sub-traits are opt-in per
capability needed.

`Inspectable` is required by `SpatialRead`, `SpatialWrite`,
`TemporalDecay`, and `WeightedSampler` trait impls on `GvGraph`
(see `graph_traits.rs`).

### 13.2 Configuration — `Config<V>`

`N` is a const generic on the `GvGraph` type (ADR-M-006). Runtime
configuration via `Config<V>`:

```rust
pub struct Config<V: Accumulator> {
    pub split_threshold: V,       // θ
    pub depth_create: u32,        // D_create
    pub depth_evict: u32,         // D_evict
    pub budget: Option<usize>,    // hard node-count ceiling (§IDEA M-7.5)
    pub alpha_relax: f64,         // relaxation threshold (ADR-M-017)
    pub bounded_eviction: bool,   // stop early once under budget (ADR-M-015)
}
```

Validated at construction: `D_create < D_evict` (D-I3), `D_create ≥ 1`,
`alpha_relax ∈ (0.0, 1.0)`, and when `budget` is `Some`, the budget
must exceed `max(3^(buffer+1), 2*(D_create−1))` (ADR-M-018).

### 13.3 Thread safety (ADR-M-007)

`GvGraph` is `Send + Sync` by construction — no interior mutability
(beyond `AtomicU32` for depth caching, which is `Send + Sync`),
no `unsafe`. Traits require `Send + Sync` as supertraits.

---

## 14. Diagnostic infrastructure — `diagnostic.rs` (ADR-M-028)

Consolidated audit and tracing helpers extracted from duplicated
logic across `evict.rs`, `split.rs`, and `rebalance.rs`. All output
uses structured `tracing` events gated by `tracing::enabled!()`.

Key functions:

- `audit_violations` — $O(n)$ full-tree scan for violated V-nodes
  not present in the work queue. Each missed node logged at `ERROR`.
- `audit_plateau_consistency` — validates plateau tiling and depth
  invariants against the live G-Tree state.
- `diagnose_missed_violation` — detailed diagnostic dump for a
  known-violated node including parent/grandparent/uncle context.
- `Gn` / `Pl` — `Display` helpers for G-node and plateau formatting
  in tracing output.

The `observe()` pipeline invokes `audit_violations` after step 5
(post-split) when a debug subscriber is active.

---

## 15. Testing infrastructure

### 15.1 Test organisation (§API M-7.4)

Three tiers, matching standard Rust convention:

| Location                        | Access          | Purpose                                 |
| ------------------------------- | --------------- | --------------------------------------- |
| `#[cfg(test)] mod tests` inline | Private fields  | Small, focused unit tests per module    |
| `src/tests/` (`#[cfg(test)]`)   | `pub(crate)`    | Cross-module crate-internal tests       |
| `tests/` (integration)          | Public API only | Tests exercising the documented surface |

### 15.2 `testing/` module

Shared infrastructure used by both `src/tests/` and `tests/`:

- **`GraphCreator`** — fluent builder: `.observe()`, `.hotspot()`,
  `.spread()`, `.sweep()`, `.zigzag()`, `.skewed()`, `.burst()`,
  `.check_every(k)`, `.build::<N>()`.
- **`Plan<C, V>`** — serializable observation sequence. Serde-gated.
  Can be stored, replayed, and shared between test files.
- **`run()` / `run_checked()` / `run_soft()`** — executors that
  apply a plan to a fresh graph, optionally asserting invariants at
  configurable intervals.
- **Presets** — degenerate plans (`plan_left_deep`, `plan_adversarial`,
  `plan_budget_burst`, etc.) for stress testing.
- **RNG stubs** — `TestLcgRng`, `FixedRng`, `SeqRng` for
  deterministic sampling tests.

### 15.3 `invariants.rs` — post-mutation checker

Validates every structural invariant from idea.md (G-I1, V-I1,
V-I3, D-I3, P-I1..P-I4, etc.) as a post-condition. Collects all
violations before panicking with a comprehensive diagnostic.
Unconditionally compiled (not `#[cfg(test)]`) so integration tests
can call `assert_invariants()`.

---

## 16. `GvGraph` impl distribution (ADR-M-030)

`GvGraph` methods are split across six focused modules. The core
module (`graph.rs`) holds the struct definition, `Config`,
`GNodeInfo`, `new()`, and accessors. Each satellite module contains
one cohesive concern as `impl GvGraph` blocks:

| Module              | Concern                                                              | Key ADRs      |
| ------------------- | -------------------------------------------------------------------- | ------------- |
| `graph.rs`          | Struct, `Config`, `GNodeInfo`, `new()`, core accessors               | ADR-M-005, -006 |
| `graph_plateau.rs`  | Plateau tracking: incremental contour maintenance, normalization     | ADR-M-026, -031 |
| `graph_query.rs`    | Read queries: `sample`, `get`, `range_sum`, `contour_range`, `contour_range_energy`, `select_plateaus` | ADR-M-019, -020, -037 |
| `graph_extract.rs`  | PEWEI extraction, layer iteration, `from_observations`               | ADR-M-021, -022 |
| `graph_budget.rs`   | Budget enforcement, depth-gate adjustment, eviction orchestration    | ADR-M-015, -017, -018 |
| `graph_traits.rs`   | Trait impls: `SpatialRead`, `SpatialWrite`, `TemporalDecay`, `WeightedSampler` | ADR-M-009 |

Corresponding tests live in `src/tests/graph.rs`,
`src/tests/graph_init.rs`, and `tests/graph_*.rs`.

---

## 17. ADR index

| ADR                                                         | Topic                                    | Status                                           |
| ----------------------------------------------------------- | ---------------------------------------- | ------------------------------------------------ |
| [001](../adr/001-node-storage.md)                           | Node storage                             | Decided — implemented                            |
| [002](../adr/002-vtree-node-enum.md)                        | V-Tree node repr                         | Decided — implemented                            |
| [003](../adr/003-violation-tracking.md)                     | Violation tracking                       | Decided — implemented                            |
| [005](../adr/005-primary-type-name.md)                      | Primary type name                        | Decided — implemented                            |
| [006](../adr/006-generic-parameters.md)                     | Generic parameters                       | Decided — implemented                            |
| [007](../adr/007-thread-safety.md)                          | Thread safety                            | Decided — implemented                            |
| [008](../adr/008-span-type.md)                              | Span type / view types                   | Decided — implemented                            |
| [009](../adr/009-trait-decomposition.md)                    | Trait decomposition / Rng                | Decided — implemented                            |
| [010](../adr/010-observation-generics.md)                   | Observation generics                     | Decided — implemented                            |
| [011](../adr/011-overflow-narrowing.md)                     | Overflow & narrowing                     | Decided — implemented                            |
| [012](../adr/012-g-sum-recomputation.md)                    | G-sum recomputation                      | Decided — implemented                            |
| [013](../adr/013-eviction-eligibility.md)                   | Eviction eligibility                     | Decided — implemented (amended by ADR-M-027)       |
| [014](../adr/014-value-absorption.md)                       | Value absorption                         | Decided — implemented                            |
| [015](../adr/015-eviction-scan-design.md)                   | Eviction scan design                     | Decided — implemented                            |
| [016](../adr/016-semi-internal-state.md)                    | Semi-internal state                      | Decided — implemented (amended by ADR-M-027)       |
| [017](../adr/017-dynamic-depth-control.md)                  | Dynamic depth control                    | Decided — implemented                            |
| [018](../adr/018-hard-budget-guarantee.md)                  | Hard budget guarantee                    | Decided — implemented                            |
| [019](../adr/019-sampling-semantics.md)                     | Sampling semantics                       | Decided — implemented                            |
| [020](../adr/020-range-query-design.md)                     | Range query design                       | Decided — implemented                            |
| [021](../adr/021-pewei-output-representation.md)            | PEWEI output representation              | Decided — implemented                            |
| [022](../adr/022-pewei-serialisation.md)                    | PEWEI serialisation                      | Decided — implemented                            |
| [023](../adr/023-pewei-reconstruction.md)                   | PEWEI reconstruction                     | Decided — implemented                            |
| [024](../adr/024-decay-semantics.md)                        | Decay semantics                          | Decided — implemented                            |
| [025](../adr/025-public-api-surface.md)                     | Public API surface                       | Decided — implemented                            |
| [026](../adr/026-point-query-and-plateau-semantics.md)      | Plateau semantics / point query          | Decided — implemented                            |
| [027](../adr/027-observation-receiving-reframe.md)          | Exposed / evictable flag reframe         | Decided — implemented                            |
| [028](../adr/028-span-native-tracing.md)                    | Span-native tracing                      | Decided — implemented                            |
| [029](../adr/029-opportunistic-depth-caching.md)            | Opportunistic depth caching              | Decided — implemented                            |
| [030](../adr/030-graph-module-decomposition.md)             | Graph module decomposition               | Decided — implemented                            |
| [031](../adr/031-sorted-placement-normalize-elimination.md) | Sorted placement / normalize elimination | Decided — implemented                            |
| [032](../adr/032-three-surface-model.md)                    | Three-surface visibility model           | Decided — implemented                            |
| [033](../adr/033-v-generic-importance-properties.md)        | V generic importance properties          | Decided — implemented                            |
| [034](../adr/034-doc-comment-and-doctest-policy.md)         | Doc-comment and doc-test policy           | Decided — implemented                            |
| [035](../adr/035-benchmarking-framework.md)                 | Benchmarking framework                   | Decided — implemented                            |
| [036](../adr/036-sentinel-integration-api.md)               | Sentinel integration API                 | Decided — implemented                            |
| [037](../adr/037-contour-range-queries.md)                  | Contour range queries                    | Decided — implemented                            |

---

## 18. Cross-reference to companion documents

| Document                         | Scope                                           |
| -------------------------------- | ----------------------------------------------- |
| [idea.md](idea.md)               | Formal specification (the "what")               |
| [api.md](api.md)                 | Public API reference (Surfaces 1 + 2)           |
| [performance.md](performance.md) | Benchmark results and performance profile       |
| [testing.md](testing.md)         | Test suite breakdown, running tests, benchmarks |
| This document                    | Implementation architecture (the "how")         |
| `adr/` directory                 | Individual design decisions                     |
