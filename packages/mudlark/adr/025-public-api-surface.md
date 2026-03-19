# ADR-M-025: Public API Surface

**Status:** Decided (Implemented)  
**Date:** 2026-02-26  
**Phase:** 5  
**Relates to:** [ADR-M-008](008-span-type.md) (view types),
[ADR-M-009](009-trait-decomposition.md) (trait decomposition),
[ADR-M-020](020-range-query-design.md) (range query design),
[ADR-M-026](026-point-query-and-plateau-semantics.md) (plateau semantics /
point query / thatching),
[ADR-M-032](032-three-surface-model.md) (three-surface model)  
**Spec:** §IDEA M-5.6 (plateaus)  
**API:** [§API M-5.2](../docs/api.md#52-gvgraphc-v-n--the-negative)
(GvGraph methods),
[§API M-5.4](../docs/api.md#54-instruments) (capability traits),
[§API M-5.5](../docs/api.md#55-trait-implementations-on-gvgraph) (trait
implementations)  
**Surface:** 2 (Film)

## Context

Phase 4 shipped the core _engine_ — `observe`, `sample`,
`range_sum`, `extract`/`reconstruct`, `decay`, and a full set
of accessors — but the public API surface was incomplete. api.md
§§API M-5.2–5.5 and ADR-M-009 specified additional methods, iterators,
and `std` trait implementations that did not yet exist on `GvGraph`.

ADR-M-026 established that the **plateau** — not the terminal cell —
is the semantic unit of the G-Tree's output, and that the natural
projection is a `BTreeMap<BasisEdge<C>, Plateau<C, V>>` with thatched energy
attribution. This fundamentally reshaped the API surface: many
items specified in the pre-revision api.md became unnecessary,
redundant, or actively misleading. Others survived unchanged.
This ADR catalogued the full picture.

All 10 items decided here are now implemented. The remainder of
this document preserves the design rationale.

**Implemented (Phase 4) — pre-existing methods:**

| Method                        | Module       |
| ----------------------------- | ------------ |
| `new(config)`                 | `graph.rs`   |
| `observe(coord, delta)`       | `observe.rs` |
| `sample(rng) -> Option<Cell>` | `graph.rs`   |
| `range_sum(range) -> V`       | `graph.rs`   |
| `extract() -> Pewei`          | `graph.rs`   |
| `decay(root, att, q)`         | `decay.rs`   |
| `node_count() -> u32`         | `graph.rs`   |
| `check_evictions() -> u32`    | `graph.rs`   |

Phase 4 also shipped the full accessor surface (`config()`,
`budget()`, `g_root()`, `v_root()`, `total_sum()`,
`depth_evict()`, `depth_create()`, `depth_buffer()`,
`headroom()`, `soft_limit()`) and
the `Debug`, `Clone`, `Send + Sync` trait implementations on
`GvGraph`. These are documented in §API M-5.2 and §API M-5.5 but
were not Phase 5 additions.

### What ADR-M-026 absorbed

The BTreeMap projection replaced or absorbed five items from the
original api.md specification. (Section references below are to the
pre-revision api.md; these items no longer appear in the current
spec.)

| Original item                | Original ref   | Absorbed by                                                                                                                                                                                                                       |
| ---------------------------- | -------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cells() -> Cells<'_, C, V>` | §IDEA M-4.6 (pre-rev) | `map.values()` explodes plateaus back into cells — backwards. The contour DFS that builds the BTreeMap _is_ the cell traversal; the BTreeMap groups its output into plateaus. Exposing raw cells encourages sub-plateau thinking. |
| `values() -> Values<'_, V>`  | §IDEA M-4.6 (pre-rev) | `map.values().map(\|p\| p.sum)` — one-liner on the BTreeMap. No custom iterator type.                                                                                                                                             |
| `range(bounds) -> RangeIter` | §IDEA M-4.4 (pre-rev) | `map.range(a..b)` — thatched plateaus in the range. Standard BTreeMap API.                                                                                                                                                        |
| `len()`, `is_empty()`        | §IDEA M-4.7 (pre-rev) | `map.len()` for plateau count (structural complexity). Arena count is `node_count()` (exists). `is_empty()` on a live GvGraph is always false — root always exists.                                                               |
| `IntoIterator for &GvGraph`  | §IDEA M-7 (pre-rev)   | Iterate the BTreeMap: `for (_, p) in &map`. No trait impl on GvGraph needed.                                                                                                                                                      |

### What Phase 5 added

| #   | API                                                  | Source                          | Notes                                                                                                                                                                |
| --- | ---------------------------------------------------- | ------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1   | `plateaus() -> Cow<BTreeMap<BasisEdge<C>, Plateau>>` | ADR-M-026, §API M-5.2            | Primary analytical projection. Live mirror, $O(1)$ borrow (with DCT). Maintenance in `graph_plateau.rs`.                                                             |
| 2   | `get(coord) -> Cell`                                 | §API M-5.2, ADR-M-026 Q4         | Point query via `route_to_receiver`. Infallible with domain clamping (ADR-M-026 DC-026-4). $O(\text{depth})$, no allocation. Trimmed half-interval for semi-internals. |
| 3   | `layers() -> impl Iterator<Item = (usize, Node)>`    | §API M-5.2                     | V-Tree BFS yielding `(layer_index, Node)`. Lazy counterpart to `extract()`. `impl Iterator`, no public named struct.                                                 |
| 4   | `terminal_count()`                                   | §API M-5.2                     | Maintained counter. $O(1)$.                                                                                                                                          |
| 5   | `budget()`                                           | §API M-5.2                     | Wrapper around `config.budget`.                                                                                                                                      |
| 6   | `Extend<(C, O)>`                                     | §API M-5.5                     | Loop over `observe()`.                                                                                                                                               |
| 7   | `from_observations(config, iter)`                    | §API M-5.2                     | Named constructor: `new()` + `extend()`. Replaces `FromIterator` (no sensible `Config::default()`).                                                                  |
| 8   | `SpatialRead` impl (revised)                         | §API M-5.4, ADR-M-009 Addendum 2 | Thin 2-method trait — see Q1. Depends on #1, #2.                                                                                                                     |
| 9   | `SpatialWrite` impl                                  | §API M-5.4, ADR-M-009 Addendum 2 | Delegation. Depends on revised `SpatialRead`.                                                                                                                        |
| 10  | `WeightedSampler` impl                               | §API M-5.4, ADR-M-009 Addendum 2 | Delegation. Depends on revised `SpatialRead`.                                                                                                                        |

**Effort estimate (retrospective):**

| Category                 | Items                                                            | ~Lines   |
| ------------------------ | ---------------------------------------------------------------- | -------- |
| Medium (new traversal)   | `plateaus`, `layers`                                             | ~180     |
| Trivial / easy           | `get`, `terminal_count`, `budget`, `Extend`, `from_observations` | ~50      |
| Trait impls (delegation) | `SpatialRead`, `SpatialWrite`, `WeightedSampler`                 | ~60      |
| Tests (~2× impl)         | —                                                                | ~550     |
| **Total**                | **10 items**                                                     | **~840** |

Down from 15 items / ~1,000 lines in the pre-ADR-M-026 framing.
The reduction came from the BTreeMap absorbing five iterator/query
items.

## Questions

### Q1: `SpatialRead` trait — revise or drop? (DC-025-1)

ADR-M-009 originally defined `SpatialRead` with cell-centric methods:
`get`, `cells`, `values`, `range`, `len`, `terminal_count`, `budget`.
ADR-M-026 eliminated four of these (`cells`, `values`, `range`,
`len`) from GvGraph. The trait as originally designed no longer
fit the implementation.

| Option | Approach                   | Notes                                                                                                                                                                                                                            |
| ------ | -------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A      | Revise the trait           | Replace cell-centric methods with `fn plateaus(&self) -> Cow<BTreeMap<BasisEdge<C>, Plateau<C, V>>>` and keep `fn get(&self, coord: C) -> Cell<C, V>`. The trait becomes "spatial-read-via-projection."                          |
| B      | Drop the trait             | `GvGraph` exposes `plateaus()`, `get()`, `range_sum()`, `extract()`, `sample()` as concrete methods. No trait abstraction. There is only one implementor — the trait adds indirection without polymorphism.                      |
| C      | Thin trait, thick BTreeMap | The trait declares only `fn plateaus(&self) -> Cow<BTreeMap<BasisEdge<C>, Plateau>>` and `fn get(&self, coord) -> Cell`. All other read operations (`range`, `values`, `len`) are standard BTreeMap methods on the returned map. |

**Analysis:**

- A trait earns its keep when there are multiple implementors
  or when downstream code is generic over the trait. Neither
  condition holds today — `GvGraph` is the only spatial structure.
- Option B is honest: no trait, no ceremony. Methods are
  discoverable on the concrete type.
- Option C is the minimal abstraction: if a second spatial
  structure ever appears, it would also need to produce a
  `BTreeMap<BasisEdge<C>, Plateau>` and support `get()`. Two
  methods is a thin enough trait to justify its existence.
- If a trait is retained (A or C), `SpatialWrite` and
  `WeightedSampler` (ADR-M-009) cascade. `SpatialWrite` requires
  `SpatialRead` — if the read trait changes, the write trait
  surface changes too.

**Decision: Option C — thin 2-method trait (Shape 1).**

```rust
pub trait SpatialRead {
    type Coord: Coordinate;
    type Accum: Accumulator;

    fn plateaus(&self) -> Cow<'_, BTreeMap<BasisEdge<Self::Coord>, Plateau<Self::Coord, Self::Accum>>>;
    fn get(&self, coord: Self::Coord) -> Cell<Self::Coord, Self::Accum>;
}
```

Object-safe, trivially mockable, and captures exactly the
semantic boundary: "I am a readable spatial structure."
`range_sum`, `extract`, `layers`, `terminal_count`, and `budget`
stay as concrete methods on `GvGraph` — they are either not
object-safe (`range_sum`'s `RangeBounds` generic), allocating
(`extract`), implementation-specific (`terminal_count`, `budget`),
or carry lifetime-bearing iterator types (`layers`). None belong
on a minimal trait.

The BTreeMap returned by `plateaus()` gives consumers `range()`,
`values()`, `len()`, `iter()`, and `contains_key()` via std.
`get()` provides the $O(\text{depth})$ point query that can't be
derived from the BTreeMap.

`SpatialWrite: SpatialRead` and `WeightedSampler: SpatialRead`
retain their current method signatures (`observe`/`decay` and
`sample` respectively) — only the super-trait shrank. See
ADR-M-009 Addendum 2.

**Implementation:** `traits.rs` defines all three traits.
`graph.rs` provides the `impl` blocks for `GvGraph`.
`SpatialRead` is object-safe; `SpatialWrite` and
`WeightedSampler` are not (generic / `impl Trait` parameters).
See §API M-5.4.

### Q2: `layers()` semantics (DC-025-2)

`extract()` already produces an owned `Pewei` snapshot via V-Tree
BFS. `layers()` occupies a different niche — lazy, borrowed
iteration for consumers who don't need the full snapshot.

| Option | Approach                   | Yields                | Notes                                                                                                                                 |
| ------ | -------------------------- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------- |
| A      | Snapshot-at-yield          | `Node<C, V>`          | `Layers<'a, C, V>` holds `&'a GvGraph`. BFS over V-Tree, yielding `Node` copies (already `Copy`). No allocation beyond the BFS queue. |
| B      | Layer-grouped              | `(usize, Node<C, V>)` | Flat stream of `(layer_index, node)` pairs. Consumer groups if needed. Same traversal as A but includes the BFS depth.                |
| C      | Redundant with `extract()` | —                     | Returns owned snapshot. Identical to `extract()` — adds nothing.                                                                      |

**Analysis:**

- `Node` is `Copy` (~48 bytes). Yielding by value is trivially
  cheap — no lifetime or borrow complexity.
- Option B adds a layer index for free, which consumers need for
  progressive reconstruction and truncation analysis.
- Option C is `extract()` under a different name — no value added.
- `extract()` remains the preferred API for serialisation and
  reconstruction (owned, no lifetime). `layers()` is for
  inspection and analysis without allocation.

**Decision: Option B — layer-grouped `(usize, Node<C, V>)`.**

```rust
pub fn layers(&self) -> impl Iterator<Item = (usize, Node<C, V>)> + '_
```

The BFS queue already tracks `(VNodeId, bfs_depth)` — the layer
index is free. `Node::state` distinguishes Terminal from
Internal/SemiInternal (equivalent to `extract()`'s
Transition/Terminal split). `Node::refinement()` computes
`sum - own`. The layer index _is_ the `v_depth` field from
`extract()`.

Structural VNodes are skipped — only Entry VNodes (backing
G-nodes) are yielded, matching `extract()`'s behavior.

Since DC-025-1 keeps `layers()` off the trait, no named
`Layers<'a, C, V>` struct is needed in the public API —
`impl Iterator` suffices. The internal BFS struct is private.

**Implementation:** `graph.rs` defines the private `Layers`
struct and the public `layers()` method returning
`impl Iterator<Item = (usize, Node<C, V>)> + '_`.

### Q3: `terminal_count()` tracking (DC-025-3)

No stored counter existed — `node_count` tracks all G-nodes
(terminals + semi-internals + internals), not just terminals.

| Option | Approach                 | Notes                                                                                                                                                                     |
| ------ | ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| A      | Maintain a `u32` counter | Increment on split (creates 2 new terminals, parent may stop being terminal). Decrement on eviction. ~10 lines across `split.rs`, `evict.rs`, `observe.rs`. $O(1)$ query. |
| B      | Compute on demand        | Walk G-node arena, count nodes with `state() == Terminal`. $O(n)$ per call.                                                                                               |
| C      | Derive from node count   | For a binary tree: `terminals = (node_count + 1) / 2`... but semi-internal nodes (1 child) break the formula. Not viable.                                                 |

**Analysis:**

- Option A is correct and efficient. The counter logic mirrors
  `node_count` maintenance, which already exists.
- Option B is $O(n)$ per call. Acceptable if rarely called, but
  surprising for an accessor that presents as $O(1)$.
- `terminal_count` is an arena-level metric, independent of
  plateaus. A plateau may contain many terminals (all at the same
  depth). `map.len()` gives plateau count; `terminal_count()`
  gives the finer-grained arena count.

**Decision: Option A — maintained `u32` counter.**

A `terminal_count` field on `GvGraph`, initialized to 1 (root is
terminal at construction). Four mutation sites:

| Site                     | Delta   | Rationale                                                                                                                    |
| ------------------------ | ------- | ---------------------------------------------------------------------------------------------------------------------------- |
| `bootstrap_split`        | net +1  | +2 new terminals, −1 parent no longer terminal                                                                               |
| `catalytic_split`        | net +1  | +2 new terminals, −1 parent no longer terminal                                                                               |
| `evict_tip`              | −1 or 0 | −1 evicted node; +1 if parent becomes terminal (both children `None`)                                                        |
| `handle_legacy_promotes` | +1      | Legacy promotion creates a new terminal child; parent transitions semi-internal → internal (no net terminal loss for parent) |

The eviction conditional reuses the existing `is_geo_terminal`
check already in `evict.rs`. A `debug_assert_eq!` in
`assert_invariants` verifies the counter against an arena walk
(`check_terminal_count_consistency` in `invariants.rs`).

### Q4: `Extend` and `FromIterator` (DC-025-4)

| Item                   | Implementation                                        | Notes                                                                                                              |
| ---------------------- | ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| `Extend<(C, O)>`       | `for (c, o) in iter { self.observe(c, o); }`          | ~10 lines. Independent of read API.                                                                                |
| `FromIterator<(C, O)>` | `let mut g = GvGraph::new(config); g.extend(iter); g` | Requires a `Config` — `FromIterator` takes no extra arguments. Must use `Default` for `Config` or skip this trait. |

**Analysis:**

- `Extend` is clean and useful — batch observation from any
  iterator of `(coord, delta)` pairs.
- `FromIterator` is awkward: it must construct a `GvGraph` from
  only the iterator, but `GvGraph::new` requires a `Config`. If
  `Config` has a useful `Default`, this works. Otherwise
  `FromIterator` should be skipped in favor of an explicit
  `from_observations(config, iter)` constructor.
- `Config<V>` has no `Default` impl and cannot have a sensible
  one: `split_threshold: V` has no universal default — `V::zero()`
  causes every node to split immediately. `depth_create` and
  `depth_evict` are domain-dependent. A misleading `Default` is
  worse than none.
- Therefore `FromIterator` is not viable. A named constructor
  `from_observations(config, iter)` is the clean alternative —
  explicit config, discoverable, composable with `Extend`.

**Decision: `Extend` yes, `FromIterator` skip.**

```rust
impl<C, V, O, const N: u32> Extend<(C, O)> for GvGraph<C, V, N>
where
    C: Coordinate,
    V: Accumulator,
    O: Observation<V>,
{
    fn extend<I: IntoIterator<Item = (C, O)>>(&mut self, iter: I) {
        for (coord, delta) in iter {
            self.observe(coord, delta);
        }
    }
}
```

`FromIterator` is replaced by a named constructor:

```rust
pub fn from_observations<O, I>(config: Config<V>, iter: I) -> Self
where
    O: Observation<V>,
    I: IntoIterator<Item = (C, O)>,
```

This preserves the ergonomic "build from data" workflow without
requiring a fictitious `Config::default()`. Neither `Config` nor
`GvGraph` implement `Default`. See §API M-5.1 and §IDEA M-6.5.

### Q5: Scope and phasing (DC-025-5)

| Option | Scope                        | Notes                                                                              |
| ------ | ---------------------------- | ---------------------------------------------------------------------------------- |
| A      | Core projection only (#1–#2) | `plateaus()` + `get()`. Minimum viable read API. No traits.                        |
| B      | Full read surface (#1–#5)    | Adds `layers()`, `terminal_count()`, `budget()`. Completes all read methods.       |
| C      | Everything (#1–#10)          | Adds `Extend`, `from_observations`, trait impls. Complete API surface. ~840 lines. |

**Analysis:**

- `plateaus()` is the headline item — the rest are small.
  Shipping `plateaus()` + `get()` alone (A) gives users the full
  query surface via the BTreeMap, but leaves trait impls for later.
- Items #4–#7 are trivial or small. No reason to defer.
- Trait impls (#8–#10) depend on resolving Q1 (revise or drop
  `SpatialRead`). They can be a separate sub-phase if the trait
  question takes time to settle.

**Decision: Option C — everything (#1–#10), single phase.**

All design questions (Q1–Q4) were decided. No remaining blockers
justified deferring any items. ~840 lines (impl + tests) was a
moderate phase — comparable to a single Phase 4 step.

## Decision

**Decided (Implemented).**

| Q   | Decision                                        | Rationale                                                                                                                                                                                                      |
| --- | ----------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | **C (decided)**                                 | Thin 2-method trait: `plateaus()` + `get()`. Object-safe, mockable. All other read operations stay as concrete methods on `GvGraph`. See DC-025-1.                                                             |
| Q2  | **B (decided)**                                 | Layer-grouped `(usize, Node)`. `impl Iterator` return type — no public named struct needed (off-trait).                                                                                                        |
| Q3  | **A (decided)**                                 | Maintained `u32` counter. $O(1)$ query. 4 mutation sites: `bootstrap_split` (+1 net), `catalytic_split` (+1 net), `evict_tip` (−1 or 0), `handle_legacy_promotes` (+1). Debug-asserted in `assert_invariants`. |
| Q4  | **`Extend` yes; `FromIterator` skip (decided)** | `Extend` is clean. `FromIterator` replaced by `from_observations(config, iter)` named constructor — `Config<V>` has no sensible `Default`. See DC-025-4.                                                       |
| Q5  | **C (decided)**                                 | Everything (#1–#10), single phase. All design questions resolved — no blockers remained. ~840 lines. See DC-025-5.                                                                                             |

## Consequences

- **The API surface shrank from 15 items to 10.** The BTreeMap
  projection absorbed `cells()`, `values()`, `range()`, `len()`,
  `is_empty()`, and `IntoIterator`. What remains: `plateaus()`,
  `get()`, `layers()`, `terminal_count()`, `budget()`, `Extend`,
  `from_observations`, and three trait impls.

- **`plateaus()` is the primary new method.** It replaced five
  cell-centric methods with a single BTreeMap projection. Point
  query, range query, iteration, length, and serialization all
  come from `std::collections::BTreeMap`'s existing API. No
  custom iterator structs for the plateau surface.

- **`layers()` returns `impl Iterator<Item = (usize, Node)>`.**
  V-Tree BFS yielding `(layer_index, Node<C, V>)`. No public
  named iterator struct — `layers()` is off the trait (DC-025-1),
  so `impl Iterator` suffices. Internal BFS struct is private
  (`Layers` in `graph.rs`).

- **`SpatialRead` is a thin 2-method trait (DC-025-1).** `plateaus()`
  - `get()` — object-safe, mockable, captures the semantic boundary
    of "readable spatial structure." Five methods (`range_sum`,
    `extract`, `layers`, `terminal_count`, `budget`) stay as concrete
    methods on `GvGraph`. `SpatialWrite` and `WeightedSampler`
    cascade cleanly — their method signatures are unchanged, only the
    super-trait shrank. See §API M-5.4 and ADR-M-009 Addendum 2.

- **Two complementary read APIs.** `get(coord)` returns a `Cell`
  with the trimmed half-interval — where the coordinate routes,
  $O(\text{depth})$, infallible with domain clamping (ADR-M-026
  DC-026-4). `plateaus()` returns
  `Cow<'_, BTreeMap<BasisEdge<C>, Plateau>>` — the tree's learned
  structure, $O(1)$ borrow of the live mirror (ADR-M-026). They
  answer different questions and coexist.

- **Two complementary projection APIs.** `extract()` is the
  V-Tree's story — significance-ordered layers for serialization
  and reconstruction. `plateaus()` is the G-Tree's story —
  spatially-ordered contour for structural analysis and queries.
  Together they are the complete read surface of the dual-tree.
  See §API M-5.2 "Two complementary projections."

- **`terminal_count()` is the sole remaining arena-level
  accessor** (beyond `node_count()`). Maintained as a `u32`
  counter, updated in `split.rs`, `evict.rs`, and `graph.rs`
  (`handle_legacy_promotes`).

- **Tests migrated to the BTreeMap.** Instead of
  `cells().collect::<Vec<_>>()`, structural tests use
  `plateaus()` and assert on plateau count, depth, thatched
  ranges, and plateau sums. Arena-level tests via `gnodes()` remain
  for crate-internal assertions (`src/tests/`).

- The §§API M-5.2–5.5 core operations surface is complete.
  All 10 items from the "What Phase 5 added" table are
  implemented. The full public surface is documented in api.md.
