# Mudlark — Public API Reference

The definitive specification for the public surface of the `torrust-mudlark` crate.
Everything listed here is intentionally public. Everything else is
`pub(crate)` or private — implementation detail, subject to change
without notice.

Modelled on `std::collections` conventions where reasonable.

---

## 1. Design principles

1. **Familiar to Rust users.** Naming, method signatures, and iterator
   patterns mirror `std::collections::{BTreeMap, HashMap, BinaryHeap}`.
2. **Zero-cost abstractions.** Lightweight `Copy` snapshot view types,
   not cloned graphs.
3. **Minimal surface.** Expose the spec's operations; hide the
   dual-tree internals.
4. **Composable queries.** The plateau `BTreeMap` exposes standard
   `BTreeMap` iteration and range methods via `std`.
5. **No `Default` where it lies.** Neither `Config<V>` nor `GvGraph`
   implements `Default` — `split_threshold`, `depth_create`, and
   `depth_evict` are domain-dependent.

---

## 2. Three-surface visibility model

> See [ADR-M-032](../adr/032-three-surface-model.md) for the governing
> rationale and the silver halide analogy.

The crate's API is organised into three **surfaces**. All modules
are `pub(crate)` — types are re-exported flat from the crate root,
giving each public type exactly one canonical path.

| Surface | Name     | Rust visibility | Role                                             |
| ------- | -------- | --------------- | ------------------------------------------------ |
| 1       | Prints   | `pub(crate)`    | Lightweight view types users hold and inspect.   |
| 2       | Film     | `pub(crate)`    | Opaque operational types users interact through. |
| 3       | Emulsion | `pub(crate)`    | Internal machinery. Not part of the public API.  |

### Crossing rules

1. **Print → Film.** Film methods _return_ Prints. A Print never holds
   a `&mut` back into the Film.
2. **Film → Emulsion.** Every Film method is a thin façade that
   delegates to Emulsion functions. Public signatures mention only
   Surface 1 and 2 types.
3. **Emulsion → Print.** Emulsion code _constructs_ Prints internally
   via struct literals (`pub` fields). The `parent` field is read from
   the G-node's immutable parent pointer, set once at split time.
4. **Emulsion ⇛ Film fields.** Emulsion code accesses `GvGraph` fields
   directly (`pub(crate)`), never through the public trait interface.

---

## 3. Crate root re-exports

```rust
// Surface 1 — Prints (view types users hold and inspect).
pub use contour_range::{BasisElement, ContourRange, ContourRangeEnergy};
pub use gnode::GState;
pub use handle::GNodeId;
pub use pewei::{Layer, Pewei, Terminal, Transition};
pub use plateau::{BasisEdge, Plateau};
pub use view::{Cell, Node, Span};

// Surface 2 — Film (opaque operational types).
pub use graph::{Config, GvGraph};
pub use traits::{Accumulator, Attenuatable, Coordinate, Inspectable,
                 Observation, Proratable, Rng, ScalableObservation,
                 SpatialRead, SpatialWrite, TemporalDecay, Weighable,
                 WeightedSampler};

// #[doc(hidden)] — available but not part of the primary API surface.
// pub use graph::GNodeChildren;
```

---

## 4. Surface 1 — Prints

Lightweight, read-only **view types** — stable, self-contained records
detached from the negative. None borrow the graph. Most are `Copy`;
`Pewei`, `Layer`, and `ContourRange` are `Clone`-only because they own
`Vec`s. All carry `Debug`.

### 4.1 Spot readings

Single-region measurements from the negative.

#### `Span<C, V>` — `Copy`

Owned dyadic interval + single intensity. Purely geometric — no node
identity or tree state.

```rust
pub struct Span<C: Coordinate, V: Accumulator> {
    pub start: C,       // [start, end)
    pub end: C,
    pub intensity: V,
    pub depth: u32,
}
```

Methods: `width() -> C`.

#### `Cell<C, V>` — `Copy`

Snapshot of a single G-node's directly-accumulated energy.
Returned by `get()` and `sample()`.

The typical case is a **contour cell** — a terminal G-node or the
uncovered half of a semi-internal G-node. However, `sample()` can
also land on an **internal** G-node whose V-entry still carries
frozen pre-split intensity (see [ADR-M-019] and §IDEA M-6.5).

| G-node state   | Interval                | `intensity`        |
| -------------- | ----------------------- | ------------------ |
| Terminal       | full `[lo, hi)`         | `g.own` (= `g.sum`) |
| Semi-internal  | uncovered half only     | `g.own`            |
| Internal       | full `[lo, hi)`         | `g.own` (frozen)   |

For semi-internals the interval is narrowed to the uncovered half
and `intensity` is the node's direct accumulation (pre-split +
absorbed + post-eviction observations routed to the vacated half;
see §IDEA M-5.5.1). For internals, both halves have children so
no narrowing occurs; `intensity` is the frozen baseline from before
the split.

`get()` routes through the G-Tree, so it always reaches a terminal
or semi-internal node — never an internal one. `sample()` walks
the V-Tree, where an internal G-node's V-entry persists with
its frozen `g.own` weight.

[ADR-M-019]: ../adr/019-sampling-semantics.md

```rust
pub struct Cell<C: Coordinate, V: Accumulator> {
    pub start: C,
    pub end: C,
    pub intensity: V,  // g.own
    pub depth: u32,
}
```

> **Thatching vs. sampling — no double-counting.** Plateau thatching
> (§IDEA M-5.6.3) multi-counts energy across overlapping plateaus
> via `g.sum`. `Cell` carries `g.own`, not `g.sum`. The V-Tree's
> importance accounting propagates `g.own` values with clean
> summation (V-I1) — no thatching overlap enters the sampling
> distribution. A `Cell` from `sample()` or `get()` reports the
> contour cell's direct accumulation, not the plateau's thatched
> energy.

Methods: `to_span() -> Span`, `width() -> C`, `is_final(n: u32) -> bool`.

> **Naming seam: `Cell.intensity` vs `Node.own` / `Node.sum`.**
>
> The recommendation to add cross-referencing doc comments at the
> `Cell.intensity` ↔ `Node.own`/`Node.sum` boundary (rather than
> renaming fields or adding `sum` to `Cell`) is the right call, but
> deserves a more thorough defence than "the naming divergence is the
> abstraction boundary working as intended." Here is the full picture.
>
> **Three vocabularies, not two.** Surface 1 types actually use
> _three_ naming conventions for the same underlying G-node accumulators:
>
> | Vocabulary        | Types using it             | `g.own` name | `g.sum` name      |
> | ----------------- | -------------------------- | ------------ | ----------------- |
> | Measurement       | `Cell`, `Span`, `Terminal` | `intensity`  | (absent or equal) |
> | Tree-accounting   | `Node`, `BasisElement`     | `own`        | `sum`             |
> | Signal-processing | `Transition`               | `baseline`   | `total`           |
>
> Each vocabulary serves its consumer. A `Cell` from `get()` or
> `sample()` is a _reading_ — you point at a spot and get back a
> number; "intensity" is the natural name. A `Node` from `layers()`
> is a _ledger entry_ — you see the full double-entry bookkeeping of
> a tree node; `own`/`sum` are the natural names. A `Transition`
> from `extract()` is a _phase boundary_ in the significance
> hierarchy — "baseline" (pre-subdivision energy) and "total"
> (inclusive of refinement) describe the spectral decomposition.
> These are not inconsistencies; they are domain-appropriate names
> at three different abstraction levels.
>
> **Why the doc-comment-only approach is correct.**
>
> 1. _Adding `sum` to `Cell` would be spatially misleading._ For
>    terminal cells, `sum == own == intensity` — the field is
>    redundant. For semi-internal contour cells (the uncovered half),
>    `g.sum` covers `[g.lo, g.hi)` — the _full_ node range including
>    the surviving child's territory — while `Cell.start..Cell.end`
>    covers only the uncovered half. Exposing `sum` on a struct whose
>    spatial extent doesn't match what `sum` aggregates would be a
>    footgun disguised as helpfulness. The existing `Transition` type
>    gets away with carrying `total` (= `g.sum`) because its
>    `start..end` _is_ the full `[g.lo, g.hi)` and its entire purpose
>    is to describe the full-node energy decomposition.
> 2. _Renaming `intensity` → `own` would infect measurement types
>    with tree internals._ `Cell`, `Span`, and `Terminal` are
>    "prints" — lightweight, detached measurements that don't know
>    about the tree. The name `own` implies a complementary `sum`
>    (a partnership the type cannot fulfil), and would make
>    `Cell::to_span()` a semantic no-op that just renames `own` back
>    to `intensity` — a signal to API consumers that the naming is
>    incoherent, not that they're crossing an abstraction boundary.
> 3. _A shared vocabulary would flatten two useful levels._ The
>    `Cell` ↔ `Node` boundary is the Surface 1 ↔ Surface 1 analogue
>    of the Print ↔ Emulsion boundary — both expose fields of the
>    same underlying G-node, but one is a user-facing measurement and
>    the other is a tree-aware diagnostic snapshot. Forcing them into
>    one naming convention would either make measurements speak
>    tree-internal language (bad for `get()`/`sample()` consumers who
>    don't care about the tree) or strip the accounting distinction
>    from `Node` (bad for `layers()` consumers who need `own`/`sum`
>    decomposition).
> 4. _Adding `gnode_id` to `Cell` violates its design._ `Cell` is a
>    handle-free, graph-detached measurement. Adding an arena handle
>    would couple it back to the graph — defeating the "no borrow,
>    no identity" guarantee that makes `Cell` safe to store, send,
>    and serialize without worrying about graph lifetime.
>
> **What the doc comments should actually say.** The proposal's three
> comments are good but could be tightened:
>
> - `Cell.intensity`: cross-reference to `Node::own` is important,
>   but should also note the Pewei parallel (`Terminal::intensity`
>   and `Transition::baseline` are the same underlying value at
>   different abstraction levels). The existing comment already
>   mentions `g.own` and `g.sum` — the gap is the inter-type
>   cross-reference, not the raw explanation.
> - `Node::to_cell()`: the existing rustdoc and this api.md spec
>   already explain the `Terminal`-only restriction and the
>   semi-internal trimming gap (see the paragraph below the `Node`
>   methods list). The proposed doc comment is a _tightening_ that
>   brings the inline rustdoc up to the level of this spec document —
>   worth doing, since the rustdoc is the first thing a consumer reads.
> - `Node::to_span()`: the warning about divergent `Span.intensity`
>   values between the `Cell` and `Node` paths is genuinely missing
>   from both the rustdoc and this spec. This is the highest-value
>   change of the three — it catches the most likely misunderstanding
>   (assuming `cell.to_span().intensity == node.to_span().intensity`
>   for the same G-node, which is only true for terminals).
>
> **One concern with the proposal.** The recommended `Cell.intensity`
> comment says "for semi-internal contour cells, `Node::sum` includes
> the surviving child's energy over a wider interval than this cell
> covers." This is accurate but buries the lede. The _primary_
> consumer confusion isn't "what does `Node::sum` cover?" (since
> `Node::sum` describes itself on `Node`); it's "I called `get(42)`
> and got `intensity = 7`, then found the same G-node via `layers()`
> with `own = 7` and `sum = 42` — which is the 'real' answer?" The
> doc comment should lead with the equivalence (`intensity` = `own`)
> and then explain _why_ `sum` is deliberately absent — which the
> existing code comment already does but without naming `Node`.
>
> **Bottom line.** Three doc comments, zero API changes is the right
> answer. The naming divergence is load-bearing — it encodes three
> distinct perspectives on the same underlying data. The only real
> gap is navigability: a consumer reading `Cell.intensity` gets no
> signpost to `Node.own`, and a consumer comparing `to_span()` on
> both types gets no warning that "intensity" means different things
> on the two paths. The proposed comments close exactly that gap.
> Implement all three changes; the `Node::to_span()` divergence
> warning is the highest-priority one.

#### `Node<C, V>` — `Copy`

Snapshot of any G-node — carries `own`, `sum`, `state`, and lineage.

```rust
pub struct Node<C: Coordinate, V: Accumulator> {
    pub start: C,
    pub end: C,
    pub own: V,
    pub sum: V,
    pub depth: u32,
    pub state: GState,
    pub gnode_id: GNodeId,   // ADR-M-036 D1
    pub parent: Option<GNodeId>,  // stable provenance (ADR-M-032)
}
```

Methods: `to_span() -> Span` (uses `sum`), `to_cell() -> Option<Cell>`,
`is_terminal() -> bool`, `refinement() -> V` (`sum - own`), `width() -> C`,
`is_root() -> bool` (`parent.is_none()`).

`to_cell()` returns `Some` only for `Terminal` nodes; `SemiInternal`
and `Internal` nodes return `None`. (Semi-internal cells are produced
by `get()` and `sample()`, which trim the interval to the uncovered
half — `Node::to_cell()` does not perform that trimming.)

The `gnode_id` field lets callers use nodes as map keys
(`BTreeMap<GNodeId, CellState>`) and pass them to
`GvGraph::gnode_info()` or `GvGraph::is_ancestor_of()`.

The `parent` field is **stable provenance** — it records which G-node
was split to produce this one. `None` at the root. Determined at
creation; immutable for the node's lifetime. Shield 3 (structural
immunity) guarantees the parent outlives the child. Callers use it
for upward traversal, decay scoping, and lineage queries.

#### `GState` — `Copy`

```rust
pub enum GState {
    Terminal,       // zero G-children
    SemiInternal,   // one G-child
    Internal,       // two G-children
}
```

Derived from child pointers — no stored field, zero storage overhead.

### 4.2 Contact print

The full developed image, detached from the negative.

#### `Pewei<C, V>` — `Clone`

Full significance-ordered extraction. Self-contained: no references
back to the graph.

```rust
pub struct Pewei<C: Coordinate, V: Accumulator> {
    pub domain_start: C,
    pub domain_end: C,
    pub layers: Vec<Layer<C, V>>,
}
```

Methods: `layer_count() -> usize` ($O(1)$),
`node_count() -> usize` ($O(L)$ where $L$ = `layer_count()`),
`total_energy() -> V` ($O(n)$ where $n$ = `node_count()`).

> **Design note — `total_energy()` is $O(n)$, not $O(1)$.**
> Most `Pewei` accessors are $O(1)$; `total_energy()` (and
> `node_count()`) walk every layer. Since `Pewei` is immutable after
> extraction, a cached `total_energy` field computed once during
> `extract()` — which already pays the $O(L + S)$ BFS cost and has
> the G-root `.sum` at hand — would make readback $O(1)$ for free.
>
> We deliberately leave the cache out for now:
>
> 1. **No hot caller.** `total_energy()` is used in assertions and
>    diagnostics, never in a tight loop. The actual hot path
>    (`reconstruct`) does its own internal traversal and never calls
>    `total_energy()`.
> 2. **Public fields.** `Pewei`'s fields are `pub` and the struct is
>    constructible by tests and by serde `Deserialize`. A cached
>    field must either be kept in sync (error-prone with public
>    construction) or hidden behind a constructor that recomputes it
>    — adding API surface for no current consumer.
> 3. **Derived `Eq`.** The struct derives `PartialEq` + `Eq`. A
>    cached energy field would either participate in equality
>    (redundant — it's a function of `layers`) or require a manual
>    `PartialEq` impl to exclude it. Both are worse than the status
>    quo for a struct this simple.
> 4. **$n$ is small.** Bounded eviction (`budget`) caps node count at
>    the configured budget (often low hundreds). Even without a
>    budget, the G-Tree depth is ≤ $N$ (a const generic, typically
>    8–16), so node count is bounded by $2^N - 1$. The walk is
>    sequential, cache-friendly, and branch-free.
>
> If a future consumer needs repeated $O(1)$ energy queries — e.g. a
> normalisation pass that divides every span intensity by total energy
> — the right fix is to call `total_energy()` once, bind the result,
> and pass it through, not to complicate the struct.

Methods (require `V: Proratable`):
`reconstruct(max_layer: usize) -> Vec<Span<C, V>>` (energy-conserving
partitioning of the domain; cost: $O(n \log n + K)$ where
$n$ = visible PEWEI nodes at layers $\leq$ `max_layer` and
$K$ = output span count).

> **Reconstruction is additive, not replacement.** Each output
> `Span` carries the **fully-summed** intensity at that region:
> all ancestral baselines pro-rated and folded in, plus the node's
> own contribution. When a transition node is expanded (its children
> are visible at the truncation depth), the parent's `baseline`
> persists as a uniform background beneath the children's
> refinements — it is _supplemented_, not _overwritten_.
>
> A consumer that stamps the parent's `total` and then overwrites
> with children's values loses the baseline energy and breaks
> conservation. You don't need to do this: `reconstruct()` handles
> the additive bookkeeping internally. Each span's `intensity` is
> the final answer for that region — just read and use it.
>
> See §PEWEI M-10 and [ADR-M-023](../adr/023-pewei-reconstruction.md)
> for the full argument.

#### `Layer<C, V>` — `Clone`

One BFS depth-level of the V-Tree.

```rust
pub struct Layer<C: Coordinate, V: Accumulator> {
    pub transitions: Vec<Transition<C, V>>,
    pub terminals: Vec<Terminal<C, V>>,
}
```

#### `Transition<C, V>` — `Copy`

Phase-transition node — a region with confirmed finer spatial structure.

```rust
pub struct Transition<C: Coordinate, V: Accumulator> {
    pub start: C,
    pub end: C,
    pub baseline: V,     // g.own
    pub total: V,        // g.sum
    pub refinement: V,   // total − baseline
    pub depth: u32,      // G-Tree depth
    pub v_depth: u32,    // V-Tree depth at extraction time
}
```

Methods (require `V: Weighable`): `snr() -> Option<f64>`
(`refinement / baseline`, `None` when baseline is zero), `width() -> C`.

> **`Transition.refinement` — stored vs. derived (§THEORY M-8.2).**
>
> The theory defines refinement as _derived_: $R = S - B$. The struct
> stores it as a field. This is an intentional denormalisation
> (DC-021-2 Option C) for ergonomics and self-documenting serialisation.
>
> **`baseline` and `total` are authoritative.** The PEWEI reconstruction
> code (`descend`) reads only those two fields; it never touches
> `refinement`. The sole consumer of the stored `refinement` is
> `snr()`, which trusts the field without re-deriving. If the invariant
> `refinement == total − baseline` is violated, `snr()` silently
> returns an incorrect ratio, but reconstruction is unaffected.
>
> **Consistency is a caller obligation.** The struct has all-`pub`
> fields and (with the `serde` feature) a derived `Deserialize` impl,
> so nothing prevents constructing an inconsistent value via struct
> literal or deserialisation. This is the same contract as, say,
> `std::ops::Range` — the library documents the invariant rather than
> enforcing it at the type level. The trade-off is justified by three
> facts:
>
> 1. `Transition` is `Copy` — once constructed it cannot be mutated
>    field-by-field, so inconsistency can only arise at creation time,
>    never by incremental drift.
> 2. The only production construction site (`graph_extract`) computes
>    the field correctly via `V::sub(g.sum, g.own)`.
> 3. Adding a private-fields-plus-constructor pattern would sacrifice
>    destructuring, pattern matching, and the lightweight data-record
>    ergonomics that make the PEWEI output pleasant to consume.
>
> Compare the live-tree path: `NodeView::refinement()` is a _method_
> that derives the same quantity on the fly. The difference is
> appropriate — `NodeView` borrows into a mutable graph where values
> change between calls, so caching would be unsound. `Transition` is
> a frozen snapshot where caching is safe.
>
> **Practical guidance for downstream code.** If you construct
> `Transition` values by hand (tests, synthetic benchmarks, custom
> deserialisation), always set `refinement` to `V::sub(total, baseline)`.
> A `debug_assert_eq!` in test harnesses is cheap insurance.

#### `Terminal<C, V>` — `Copy`

Leaf node — no further subdivision exists.

```rust
pub struct Terminal<C: Coordinate, V: Accumulator> {
    pub start: C,
    pub end: C,
    pub intensity: V,
    pub depth: u32,
    pub v_depth: u32,
}
```

Methods: `width() -> C`.

### 4.3 Densitometry

Isodensity zones across the G-Tree bottom contour, and the
structural decomposition of lattice-aligned strip queries through
those zones.

#### `BasisEdge<C>` — `Copy`

```rust
pub struct BasisEdge<C: Coordinate>(pub C);
```

`Ord`-providing newtype for `BTreeMap` keys. Delegates to
`Coordinate::total_cmp` for total ordering (including floats).

#### `Plateau<C, V>` — `Copy`

One plateau — a contiguous region of uniform contour depth.

```rust
pub struct Plateau<C: Coordinate, V: Accumulator> {
    pub basis_edge: BasisEdge<C>,
    pub start: C,       // thatched range start
    pub end: C,         // thatched range end
    pub depth: u32,     // contour depth
    pub sum: V,         // Σ R.sum over basis elements
}
```

Methods: `width() -> C`, `to_span() -> Span` (uses `sum` as intensity).

> **Duplicate `BasisEdge` in `Plateau`.**
> The `basis_edge` field always equals the `BTreeMap` key it is stored
> under (enforced by the key-consistency invariant in
> `check_plateau_btreemap_key_consistency`). The redundancy is
> deliberate: `Plateau` is a Surface 1 _Print_ — a `Copy`, detachable
> record that may be pulled out of the map via `.values()`, passed
> across API boundaries, or round-tripped through `serde` without its
> key. Embedding `basis_edge` keeps the value self-describing in every
> context, consistent with how `Node` carries its own `gnode_id` and
> `parent`. In practice `BasisEdge` is a single-field newtype wrapping
> `C`, so the overhead is one `C` per plateau — negligible relative
> to the BTree node cost.

A strip query through these zones decomposes into a **basis set**
(ADR-M-037, §CR.2–§CR.6):

#### `BasisElement<C, V>` — `Copy`

One element of the minimal G-node cover of a contour range.
At most two elements per range are boundary thatching semi-internals,
flagged by `is_boundary_thatch`.

```rust
pub struct BasisElement<C: Coordinate, V: Accumulator> {
    pub gnode_id: GNodeId,
    pub start: C,
    pub end: C,
    pub own: V,
    pub sum: V,
    pub depth: u32,
    pub is_boundary_thatch: bool,
}
```

#### `ContourRange<C, V>` — `Clone`

Full basis-set decomposition of a lattice-aligned half-open interval,
plus pre-computed energy fields (§CR.6, §CR.10.4, §CR.13).

```rust
pub struct ContourRange<C: Coordinate, V: Accumulator> {
    pub start: C,
    pub end: C,
    pub basis: Vec<BasisElement<C, V>>,
    pub energy: V,
    pub exact_energy: V,
    pub plateau_energy: V,
    pub cross_plateau_energy: V,
    pub plateau_count: usize,
}
```

| Field                  | Definition                                                                                                                                                                                                                                                                                                           |
| ---------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `start`                | Range start (inclusive), on the endpoint lattice                                                                                                                                                                                                                                                                     |
| `end`                  | Range end (exclusive), on the endpoint lattice                                                                                                                                                                                                                                                                       |
| `basis`                | Basis set: minimal G-node cover of `[start, end)` (§CR.2)                                                                                                                                                                                                                                                            |
| `energy`               | Contour range energy (§CR.6): `Σ basis[i].sum`. Includes boundary thatching; does not double-count interior thatching (resolved by basis consolidation)                                                                                                                                                              |
| `exact_energy`         | Exact energy (§CR.13): `range_sum(start..end)`. Pro-rates boundary G-nodes under the uniform-within-cell assumption. Not generally ordered relative to `energy` (§CR.13.6)                                                                                                                                           |
| `plateau_energy`       | Sum of individual plateau energies within the range (§CR.10.4): `Σ P_j.sum` for every plateau whose basis edge lies in `[start, end)`. Each plateau's `.sum` is the energy of its own leaf-level basis, computed independently — it does not include ancestor `.own` energy or thatching from neighbouring plateaus  |
| `cross_plateau_energy` | Cross-plateau energy (§CR.10.4): `energy − plateau_energy`. Captures the net effect of basis consolidation (ancestor `.own` energy gained when the multi-plateau decomposition selects spanning nodes) minus interior thatching resolved by that consolidation. Sign is **indeterminate** under non-negative `g.own` |
| `plateau_count`        | Number of plateaus whose basis edge lies in `[start, end)` — i.e. `plateaus.range(start..end).count()`. Every plateau fully contained in the range is counted; boundary semantics follow the half-open interval (start-inclusive, end-exclusive)                                                                     |

The identity `energy = plateau_energy + cross_plateau_energy` holds
by construction.

#### `ContourRangeEnergy<V>` — `Copy`

Energy-only result — lightweight alternative to `ContourRange` that
carries only the scalar energy fields, no basis set.

```rust
pub struct ContourRangeEnergy<V: Accumulator> {
    pub energy: V,
    pub exact_energy: V,
    pub plateau_energy: V,
    pub cross_plateau_energy: V,
    pub plateau_count: usize,
}
```

Field semantics are identical to `ContourRange` — see the table above.

### 4.4 Structural snapshot

`gnode_info()` returns the same `Node<C, V>` type used by `layers()`,
containing spatial extent, accumulation, depth, state, the
`gnode_id` handle, and the stable `parent` handle. The mutable
refinement topology — which children currently exist below a node —
is available through the `#[doc(hidden)]` method `gnode_children()`
returning a `GNodeChildren`.

#### `GNodeChildren` — `Copy`, `#[doc(hidden)]`

```rust
pub struct GNodeChildren {
    pub left: Option<GNodeId>,
    pub right: Option<GNodeId>,
}
```

`Debug`, `Clone`, `Copy`, `PartialEq`, `Eq`.

These fields reflect the graph's **current refinement state** and
may change after any mutation (split, eviction, legacy promotion).
They are `#[doc(hidden)]` because they fail the snapshot test:
mutable topology, not stable provenance.

### 4.5 Handles

Opaque references back into the graph.

```rust
pub struct GNodeId(/* NonZeroU32 */);
```

Opaque typed arena index. Used by `decay()` (`root` parameter),
`g_root()`, `gnode_info()`, `gnode_children()`, `is_ancestor_of()`.
Also appears in `Node.gnode_id`, `Node.parent`, and
`BasisElement.gnode_id`.

`GNodeId`: `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`, `PartialOrd`,
`Ord`, `Hash`.

`GNodeId` carries
`#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`.

`from_index()` and `index()` are `#[doc(hidden)]` — available for
serialization and diagnostics but not part of the primary API surface.

`VNodeId` is `pub(crate)` — it has no public consuming method
(ADR-M-032 handle test). `v_root()` is likewise `pub(crate)`.
Neither is re-exported from the crate root.

> **Serde on handles — resolution of §4.5 vs §4.6/§7.3.**
>
> The code agrees with §4.5: both `GNodeId` and `VNodeId` carry the
> conditional serde derives. §4.6 ("all Surface 1 types _except
> handles_") and §7.3 ("except opaque handles") were wrong and have
> been corrected.
>
> Handles _should_ have serde. The safety concern — that deserializing
> an arbitrary index could mint a handle that bypasses validation — is
> real but already mitigated: every consuming method (`gnode_info`,
> `is_ancestor_of`, `decay`) validates the handle against the arena's
> occupancy bitmap and returns `None` or panics on stale/fabricated
> indices. The handle is an opaque index, not a capability token.
> Refusing to serialize it would force callers to use the
> `#[doc(hidden)]` `from_index()`/`index()` pair manually — strictly
> worse ergonomics for the same trust boundary.
>
> Additionally, `GNodeId` appears as a field inside `Node` and
> `BasisElement`, both of which are serde-enabled. Excluding the
> handle from serde would break serialization of those types entirely
> (the derive would fail to compile because a field's type would not
> implement `Serialize`/`Deserialize`).
>
> The appropriate mental model: handles serialize their opaque index,
> not a meaningful identity. A deserialized `GNodeId` is only valid
> against the graph instance that produced it — the same constraint
> that applies to handles obtained at runtime. Serde adds no new
> attack surface beyond what `from_index()` already permits.

### 4.6 Invariants

Every Print is `Debug` and carries no mutable reference into the
graph. Most are `Copy`; `Pewei`, `Layer`, and `ContourRange` are
`Clone`-only because they own `Vec`s. All Surface 1 types — including
handles — carry
`#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`.
Handles serialize their opaque arena index; a deserialized handle
is only valid against the graph instance that produced it (see
§4.5 resolution note).

---

## 5. Surface 2 — Film

The **opaque operational types** through which users expose, measure,
and attenuate the medium.

### 5.1 `Config<V>`

Film stock specification — fixed before loading.

```rust
#[derive(Debug, Clone, PartialEq)]
pub struct Config<V: Accumulator> {
    pub split_threshold: V,       // θ — minimum intensity to split
    pub depth_create: u32,        // D_create — max V-depth for split
    pub depth_evict: u32,         // D_evict — min V-depth for eviction
    pub budget: Option<usize>,    // hard node-count ceiling
    pub alpha_relax: f64,         // α_relax — depth-gate hysteresis ∈ (0, 1)
    pub bounded_eviction: bool,   // stop early once under budget
}
```

`alpha_relax` controls the **hysteresis band** for dynamic depth-gate
adjustment (§IDEA M-7.4). When a `budget` is set, the system tightens
both gates in lockstep (lowering `D_evict` and `D_create` by 1)
whenever the node count exceeds the soft limit, but only _relaxes_ them
(raising both by 1) once the count drops below
`soft_limit × alpha_relax`. The buffer `D_evict − D_create` is invariant
(ADR-M-017). The gap between those two thresholds is a dead
zone where the depth gates hold steady, preventing oscillation. Values
near 0 make the system very reluctant to relax — once tightened it stays
coarse until the tree is nearly empty; values near 1 narrow the dead
zone so the system re-expands almost immediately after shedding nodes,
maximising detail retention at the cost of more frequent gate changes.
A typical default is 0.75.

`Config` records the **initial** depth gates. After construction,
dynamic adjustments modify the live values on `GvGraph` itself —
accessible via `depth_create()` and `depth_evict()` — while
`config().depth_create` and `config().depth_evict` retain their
construction-time values.

No `Default` impl. Validated at construction:

- `depth_create < depth_evict` (D-I3).
- `depth_create >= 1`.
- `alpha_relax ∈ (0.0, 1.0)`.
- When `budget` is `Some`: budget > max(3^(buffer+1), 2\*(D_create−1)).

### 5.2 `GvGraph<C, V, N>` — the negative

The live dual-tree index. Continuously exposed and regressing.

```rust
pub struct GvGraph<C: Coordinate, V: Accumulator, const N: u32> { /* opaque */ }
```

- `C` — coordinate type (`u8`..`u128`, `f32`, `f64`).
- `V` — accumulation / intensity type (`u8`..`u128`, `f32`, `f64`).
- `N` — domain bit-width. The domain is `[0, 2^N)`.

All fields are `pub(crate)`. The struct is opaque to external consumers.

**Method index** — grouped by category, with trait-bound and
asymptotic cost at a glance.

| Method                        | Bound         | Category           | Cost                                                           | One-liner                                                                                                        |
| ----------------------------- | ------------- | ------------------ | -------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `new`                         | —             | Construction       | $O(1)$                                                         | Single root covering $[0, 2^N)$                                                                                  |
| `from_observations`           | `Insp`        | Construction       | $n \times$ observe                                             | `new` + `extend`                                                                                                 |
| `observe`                     | `Insp`        | Spatial mutation   | $O(d + h_V)$ amort.^[1]^                                       | Route → accumulate → split → rebalance                                                                           |
| `decay`                       | `Att + Insp`  | Temporal transform | $O(\lvert G_{\text{sub}}\rvert + \lvert V\rvert + K h_V)$^[2]^ | Subband-adaptive scaling of G-subtree                                                                            |
| `check_evictions`             | `Insp`        | Maintenance        | $O(E_t + E \cdot h_V)$^[3]^                                    | Evict all eligible tips                                                                                          |
| `get`                         | —             | Point query        | $O(d)$                                                         | Cell at coordinate                                                                                               |
| `plateaus`                    | `Insp`        | Projection         | $O(1)$ / $O(G)$^[4]^                                           | Bottom-contour `BTreeMap`                                                                                        |
| `range_sum`                   | `Pror`        | Projection         | $O(N)$                                                         | Pro-rated sum over arbitrary range                                                                               |
| `contour_range`               | `Pror + Insp` | Projection         | $O(N)$                                                         | Basis decomposition of lattice interval                                                                          |
| `contour_range_energy`        | `Pror + Insp` | Projection         | $O(N)$                                                         | Energy-only variant                                                                                              |
| `select_plateaus`             | `Insp`        | Projection support | $O(\log P)$                                                    | Snap coordinates to lattice endpoints                                                                            |
| `sample`                      | `Weigh`       | Stochastic         | $\leq 1.44H + 1$ exp.                                          | Weighted random cell                                                                                             |
| `extract`                     | `Insp`        | Snapshot           | $O(\lvert V\rvert)$                                            | Full PEWEI                                                                                                       |
| `layers`                      | `Insp`        | Iteration          | $O(\lvert V\rvert)$ total                                      | Lazy BFS yielding `(layer, Node)`                                                                                |
| `gnode_info`                  | —             | Introspection      | $O(1)$                                                         | `Node` snapshot by handle                                                                                        |
| `is_ancestor_of`              | —             | Introspection      | $O(1)$                                                         | Proper ancestor test (interval containment)                                                                      |
| `gnode_children`^[5]^         | —             | Introspection      | $O(1)$                                                         | Current child handles                                                                                            |
| `build_plateaus`^[5]^         | `Insp`        | Diagnostic         | $O(G + B\log B)$                                               | Full plateau rebuild via DFS                                                                                     |
| `debug_plateau_basis`^[5][7]^ | `Insp`        | Diagnostic         | $O(B)$                                                         | Per-plateau basis bookkeeping                                                                                    |
| Accessors (11)                | —             | Accessor           | $O(1)$                                                         | `node_count`, `terminal_count`, `budget`, `total_sum`, `config`, `g_root`, depth gates, `headroom`, `soft_limit` |

1. Core cost per call; budget-guarded eviction, when triggered, adds amortised $O(h_V)$ per eviction (§PERF M-3, §PERF M-7.1).
2. $K$ = V-I3 violations created by scaling. Uniform decay on floats ($q = 0$): $K = 0$, cost simplifies to $O(\lvert G\rvert)$. Non-uniform or integer types: $K$ may be non-zero (§THEORY M-7.4).
3. Per-eviction cost is $O(h_V + d_{\text{geo}} + \log P)$, giving full per-call cost $O(E_t + E \cdot (h_V + d_{\text{geo}} + \log P))$ where $E_t$ = scan pool and $E \leq E_t$ = actually evicted. Convergence (repeated calls): $O(\lvert G_0\rvert \cdot h_V)$ with P4 (§THEORY M-A.7).
4. $O(1)$ with `dynamic-contour-tracking` (default); $O(G + B\log B)$ rebuild without.
5. `#[doc(hidden)]` — available but not part of the stable surface.
6. Requires `dynamic-contour-tracking` feature (enabled by default).
7. **Bound abbreviations:** `Insp` = `Inspectable`, `Att` = `Attenuatable`, `Pror` = `Proratable`, `Weigh` = `Weighable`. All methods additionally require the struct-level `V: Accumulator` bound.

#### Construction

```rust
pub fn new(config: Config<V>) -> Self;
```

Requires only `V: Accumulator`. Initialises both trees with a single
shared G-node covering `[0, 2^N)`. Panics (compile-time) if
`N > C::BITS`, or at runtime if config validation fails.

Available when `V: Inspectable`:

```rust
pub fn from_observations<O, I>(config: Config<V>, iter: I) -> Self
where
    O: Observation<V>,
    I: IntoIterator<Item = (C, O)>;
```

`from_observations` is `new` + `extend` — replaces `FromIterator`
(which would require a fictitious `Config::default()`).
Because each element runs the full `observe()` pipeline, the
iteration order of the `IntoIterator` matters structurally — see
[`Extend<(C, O)>` semantics](#extendc-o-semantics) for the full
discussion.

Cost: $n \times$ `observe` cost (see below).

#### Observation (mutation)

Available when `V: Inspectable`.

```rust
pub fn observe<O: Observation<V>>(&mut self, coord: C, delta: O);
```

Pipeline: route → accumulate → propagate → violation check → split →
rebalance → depth control → budget-guarded eviction.

Cost: $O(d_{\text{geo}} + h_V)$ per call — G-Tree routing
($O(d_{\text{geo}})$) plus V-Tree propagation & rebalance ($O(h_V)$).
This is the **core observation cost** without eviction; budget-guarded
eviction (step 8), when triggered, adds amortised overhead that can
dominate under sustained budget pressure
(see §PERF M-3, §PERF M-7.1).

#### Point query

```rust
pub fn get(&self, coord: C) -> Cell<C, V>;
```

Infallible. Routes to the terminal or semi-internal receiver.
Out-of-domain coordinates are clamped. NaN panics. For semi-internal
nodes, the returned `Cell` interval is the **uncovered half** — the
sub-interval where the missing child would be, not the child's half.
The `intensity` is the node's `g.own`: all energy that was ever
accumulated directly on this node (pre-split observations spanning
the full interval + post-eviction observations routed to the
vacated half; see §IDEA M-5.5.1).

_Example._ A semi-internal node covers `[4, 8)` with a left child
occupying `[4, 6)`. The right half `[6, 8)` has no child —
`get(7)` returns `Cell { start: 6, end: 8, intensity: g.own, .. }`.
Conversely, `get(5)` routes through the left child and never stops
at this node.

Cost: $O(\text{depth})$ — equivalently $O(d_{\text{geo}})$, at most $O(N)$.

> **Routing vs. plateau point queries.** The theory document
> (§IDEA M-5.6.7, §IDEA M-18.9) defines two complementary point
> queries. `get()` is the **routing point query**: a G-Tree descent
> returning a `Cell` — observation-level truth at $O(d_{\text{geo}})$.
> The **plateau point query** is a floor-key lookup on the `BTreeMap`
> returned by `plateaus()`:
> `plateaus.range(..=BasisEdge(coord)).next_back()` — structural
> truth returning a `Plateau` at $O(\log P)$. The two have different
> costs, return types, and semantics; see §IDEA M-5.6.7 for the full
> distinction.

#### Plateau projection

Available when `V: Inspectable`.

```rust
pub fn plateaus(&self) -> Cow<'_, BTreeMap<BasisEdge<C>, Plateau<C, V>>>;
```

Returns the live `BTreeMap` mirror of the G-Tree's bottom contour.
With `dynamic-contour-tracking` (default): `Cow::Borrowed` — $O(1)$.
Without: `Cow::Owned` — $O(G)$ rebuild.

The standard `BTreeMap` query surface is available directly:

- **Range query:** `plateaus.range(a..b)`
- **Iteration:** `for (_, p) in &*plateaus`
- **Length:** `plateaus.len()`
- **Floor-key lookup:** `plateaus.range(..=BasisEdge(coord)).next_back()`

#### Range sum

Available when `V: Proratable`.

```rust
pub fn range_sum<R: RangeBounds<C>>(&self, range: R) -> V;
```

Recursive G-Tree descent with pro-rated `g.own` at partial overlaps.
Accepts `a..b`, `a..=b`, `..b`, `a..`, `..`. Out-of-domain bounds are
clamped. Empty ranges return `V::zero()`. NaN bounds panic.

Cost: $O(N)$.

#### Contour range decomposition

Available when `V: Proratable + Inspectable`.

Builds on the plateau projection — endpoints must lie on the
endpoint lattice (the set of plateau basis edges plus the domain
sentinel $2^N$).

$0$ is **always** in the endpoint lattice. The G-root covers
$[0, 2^N)$ and can never be evicted; the leftmost basis element in
the DFS therefore always has `BasisEdge(0)`. A freshly constructed
graph has a single root plateau at `BasisEdge(0)`, and no sequence
of splits or evictions can remove it: splits only add children (whose
leftmost terminal still starts at $0$), and the merge pass in
`build_plateaus` preserves `BasisEdge(0)` as the first key since
there is never a left neighbour to absorb it into. Consequently,
`contour_range(BasisEdge(0), BasisEdge(domain_max))` always
succeeds (full-domain query).

```rust
pub fn contour_range(
    &self,
    start: BasisEdge<C>,
    end: BasisEdge<C>,
) -> Option<ContourRange<C, V>>;
```

Full basis-set decomposition of a lattice-aligned half-open interval
`[start, end)` (§CR.2–§CR.6). Endpoints must lie on the endpoint
lattice — the set of plateau basis edges plus the domain sentinel
`2^N`. Returns `None` if either endpoint is not in the lattice or
`start >= end`.

The result contains the basis set (§CR.2) plus pre-computed energy
fields: contour range energy (§CR.6, `Σ basis.sum`), exact energy
(§CR.13, `range_sum`), plateau energy (`Σ P_j.sum` over constituent
plateaus, §CR.10.4), and cross-plateau energy (`energy − plateau_energy`,
§CR.10.4). See the `ContourRange` field table above for full definitions.

Cost: $O(N)$ — segment-tree decomposition plus one `range_sum()` pass.

```rust
pub fn contour_range_energy(
    &self,
    start: BasisEdge<C>,
    end: BasisEdge<C>,
) -> Option<ContourRangeEnergy<V>>;
```

Energy-only variant — same lattice-endpoint requirement, same $O(N)$
cost. Returns only the scalar energy fields; the basis set is
computed but not exposed. (A dedicated fast path that skips the
allocation is a natural future optimisation.)

Cost: $O(N)$.

#### Plateau selection

Available when `V: Inspectable`.

```rust
pub fn select_plateaus(&self, lo: C, hi: C) -> Option<(BasisEdge<C>, BasisEdge<C>)>;
```

Given arbitrary dyadic coordinates `[lo, hi)`, snaps outward to the
nearest lattice-aligned endpoints that span all overlapping plateaus
(§CR.12). The returned pair is valid for `contour_range()` and
`contour_range_energy()`.

Returns `None` if the plateau map is empty, `lo >= hi`, or no plateau
overlaps `[lo, hi)`.

Cost: $O(\log P)$ — two `BTreeMap` lookups.

#### Proportional sampling

Available when `V: Weighable`.

```rust
pub fn sample(&self, rng: &mut impl Rng) -> Option<Cell<C, V>>;
```

Weighted random walk from the V-Tree root. Returns `None` if total
intensity is zero.

The landing V-entry is typically a terminal G-node, but may be
semi-internal or internal (with frozen pre-split intensity — see
[ADR-M-019](../adr/019-sampling-semantics.md) and §IDEA M-6.5).

- **Terminal:** `Cell` covers the full `[lo, hi)` range;
  `intensity = g.own = g.sum`.
- **Semi-internal:** `Cell` is narrowed to the uncovered half with
  `intensity = g.own`, exactly as `get()` would return. There is a
  deliberate spatial mismatch between the entry's competitive weight
  (which spans the full node range) and the output interval (uncovered
  half only) — see [ADR-M-016](../adr/016-semi-internal-state.md) and
  §IDEA M-5.5.1 for the rationale.
- **Internal:** `Cell` covers the full `[lo, hi)` range;
  `intensity = g.own` (frozen baseline from before the split).
  Both halves have children, so no narrowing occurs.

Expected cost: $\leq 1.44\,H + O(1)$ node visits, where $H$ is the
Shannon entropy of the weight distribution. See §THEORY M-4.3 for
the tight bound and §IDEA M-18.1 for the Fibonacci overhead analysis.

#### PEWEI extraction

Available when `V: Inspectable`.

```rust
pub fn extract(&self) -> Pewei<C, V>;
```

BFS walk of the V-Tree producing a significance-ordered, layered
snapshot. V-entries backed by terminals become `Terminal` values;
entries backed by internal or semi-internal G-nodes become
`Transition` values. V-structural nodes are scaffolding and are not
emitted.

Cost: $O(L + S)$ where $L$ = V-entries, $S$ = V-structural nodes — every V-node visited once.

#### Layer iteration

Available when `V: Inspectable`.

```rust
pub fn layers(&self) -> impl Iterator<Item = (usize, Node<C, V>)> + '_;
```

Lazy V-Tree BFS yielding `(layer_index, Node)`. Streaming counterpart
to `extract()` — same traversal order, same node classification, but
yields `Node` view types without allocating the full `Pewei` snapshot.

Cost: $O(L + S)$ total (same as `extract()`; $L$ = V-entries,
$S$ = V-structural nodes), amortised across `next()` calls.

#### Temporal decay

Available when `V: Attenuatable + Inspectable`.

```rust
pub fn decay(&mut self, root: GNodeId, attenuation: f64, q: f64);
```

Subband-adaptive temporal decay. `root` specifies the G-subtree
(use `g_root()` for the entire tree).

- `attenuation` — base factor in `[0, ∞]`. Values in `(0, 1)` cause
  exponential decay, `1.0` is a no-op, values above `1.0` amplify,
  `0.0` is annihilation (§THEORY M-7.3), `∞` is infinite
  amplification (mirror of annihilation).
- `q` — selectivity in `[0, 1]`:
  - $q = 0$: uniform — all subbands decay at the same rate.
  - $q > 0$: selective — coarse persists, fine fades faster.
  - $q = 1$: maximum selectivity.

Per-depth factor (for $\text{att} > 0$):

$$\ln\lambda(d) = \ln(\text{att}) \cdot (1 + q \cdot (2d_\text{local}/D - 1))$$

**Annihilation** ($\text{att} = 0$): the per-depth factor is
$0^{\text{exponent}}$, using the convention $0^0 = 1$. At $q < 1$
every exponent is positive, so the entire subtree is zeroed. At
$q = 1$ the subtree root's exponent is $1 - q = 0$, giving
$0^0 = 1$ — the root is preserved while all descendants are zeroed.
This is the _detail flush_ (§THEORY M-7.3): the coarsest codeword
survives; all refinement must be re-earned.

**Infinite amplification** ($\text{att} = \infty$): the per-depth
factor is $\infty^{\text{exponent}}$, using the convention
$\infty^0 = 1$. Mirror of annihilation: at $q < 1$ every exponent is
positive, so the entire subtree receives $\infty$. At $q = 1$ the
subtree root's exponent is $1 - q = 0$, giving $\infty^0 = 1$ — the
root is preserved while all descendants receive $\infty$. The actual
effect on accumulator values is determined by the `V: Attenuatable`
implementation (e.g. built-in integers saturate via `as` cast).

Panics if `root` is not live (freed by eviction), `attenuation` is
negative or NaN, or `q` outside `[0, 1]` or NaN. Does not trigger
eviction. Invariants (G-I1, V-I1, V-I3) are restored via trailing
rebalance.

> **Stale-handle panic — rationale and caveats.**
>
> `decay()` panics on a stale `root`; `gnode_info()` and
> `is_ancestor_of()` return `None` / `false`. The asymmetry is
> intentional. `gnode_info()` is a _query_ — asking "does this node
> still exist?" is a reasonable question with a reasonable answer.
> `decay()` is a _mutation_ that walks a subtree, recomputes sums,
> and fixes V-I3 violations across the graph. Decaying a freed
> subtree is structurally incoherent — there is no meaningful
> partial result or graceful degradation, so the alternatives are
> `panic!` or `Result`. Since the caller chose which subtree to
> decay, a dead root signals a contract violation, and the panic
> message names the exact stale index to aid diagnosis.
>
> **Staleness is not always a logic error.** A `GNodeId` obtained
> from `gnode_info()`, `layers()`, or `BasisElement.gnode_id` can
> become stale after any mutating call — budget-guarded eviction
> inside `observe()` (step 8) or an explicit `check_evictions()`
> can remove the backing G-node while the caller still holds the
> handle. This is normal operation, not a bug in the caller's logic.
>
> Certain handles are **structurally safe**:
>
> - `g_root()` — the root covers $[0, 2^N)$ and can never be
>   evicted. This is the common case for global decay.
> - `Node.parent` — Shield 3 (structural immunity) guarantees the
>   parent is eviction-immune while any child exists, so a parent
>   handle obtained from a live child is safe until that child is
>   itself evicted.
>
> Handles to arbitrary deep leaves or subtree roots obtained via
> introspection carry **no structural guarantee** across mutation
> boundaries. The recommended defensive pattern:
>
> ```rust
> // Liveness check before subtree decay.
> if graph.gnode_info(subtree_root).is_some() {
>     graph.decay(subtree_root, att, q);
> }
> ```
>
> The check and the decay are not atomic — in principle a concurrent
> mutation could evict the node between the two calls. However,
> mutating methods take `&mut self`, so within a single exclusive
> borrow the check-then-decay sequence is safe — no concurrent
> mutation can interleave. In architectures where the graph is
> behind a `Mutex` or `RwLock`, the write-guard must span both calls.

Cost: $O(|G_{\text{subtree}}| + |V| + K \cdot h_V)$ — DFS walk
to scale own-values ($|G_{\text{subtree}}|$), bottom-up G-sum
recomputation, V-sum recomputation ($|V|$), and trailing rebalance
for $K$ violations. For global uniform decay on float types, $K = 0$
and the cost simplifies to $O(|G|)$; under non-uniform scaling or
integer types, $K$ may be non-zero (§THEORY M-7.4).

#### Manual eviction

Available when `V: Inspectable`.

```rust
pub fn check_evictions(&mut self) -> u32;
```

Evict all eligible candidates (unbounded). Returns the number evicted.
Trailing rebalance restores V-I3.

Cost per call: $O(E_t + E \cdot (h_V + d_{\text{geo}} + \log P))$
where $E_t$ is the scan pool (entries past $D_{\text{evict}}$) and
$E \leq E_t$ is the number actually evicted. Total convergence cost
(repeated calls until quiescent): $O(|G_0| \cdot h_V)$ with P4
(§THEORY M-A.7); $O(|G_0| \cdot h_V^2)$ without P4.

#### G-node introspection (ADR-M-036)

Available always (only requires `V: Accumulator`).

```rust
pub fn gnode_info(&self, id: GNodeId) -> Option<Node<C, V>>;
```

Returns a `Node` snapshot of the specified G-node, or `None` if
the handle is not live (freed by eviction). All fields are captured at
call time — the result does not borrow the graph. The `parent` field
is the node's stable provenance; for mutable child linkage, see
`gnode_children()`.

Cost: $O(1)$ — single arena lookup plus depth computation.

```rust
#[doc(hidden)]
pub fn gnode_children(&self, id: GNodeId) -> Option<GNodeChildren>;
```

Returns the current child handles of the specified G-node, or `None`
if the handle is not live. The returned values reflect the graph's
refinement state at call time and may be invalidated by any subsequent
mutation.

Cost: $O(1)$ — single arena lookup.

```rust
pub fn is_ancestor_of(&self, ancestor: GNodeId, descendant: GNodeId) -> bool;
```

Whether `ancestor` is a **proper** ancestor of `descendant` in the
G-Tree (strict interval containment; a node is not its own ancestor).
Returns `false` if either handle is not live.

Cost: $O(1)$ — two arena lookups plus interval comparison.

#### Plateau rebuild (`#[doc(hidden)]`)

Available when `V: Inspectable`.

```rust
#[doc(hidden)]
pub fn build_plateaus(&self) -> BTreeMap<BasisEdge<C>, Plateau<C, V>>;
```

Rebuilds the full plateau map from scratch via a DFS walk of the
G-Tree. Returns an owned `BTreeMap`. This is the fallback used by
`plateaus()` when `dynamic-contour-tracking` is disabled, and is
also useful for diagnostics and invariant checking against the live
mirror.

Cost: $O(G + B \log B)$ where $G$ = total G-nodes, $B$ = basis size.

```rust
#[doc(hidden)]
#[cfg(feature = "dynamic-contour-tracking")]
pub fn debug_plateau_basis(
    &self,
) -> Vec<(BasisEdge<C>, Vec<(usize, C, C, &'static str, u32)>)>;
```

Exposes the internal per-plateau basis bookkeeping for integration
tests that need to diagnose plateau invariant violations.
Only available with `dynamic-contour-tracking`.

#### Accessors

```rust
// Size and capacity — all O(1) maintained counters.
pub const fn node_count(&self) -> u32;         // total G-nodes
pub const fn terminal_count(&self) -> u32;     // leaf cells only
pub const fn budget(&self) -> Option<usize>;   // shorthand for config().budget
pub fn total_sum(&self) -> V;                  // G-root sum (aggregate of all observations)

// Configuration.
pub const fn config(&self) -> &Config<V>;

// Tree handles.
pub const fn g_root(&self) -> GNodeId;
pub(crate) const fn v_root(&self) -> Option<VNodeId>;
```

```rust
// Dynamic depth control.
pub const fn depth_evict(&self) -> u32;
pub const fn depth_create(&self) -> u32;
pub const fn depth_buffer(&self) -> u32;       // D_evict − D_create (fixed)
pub const fn headroom(&self) -> usize;         // 3^(buffer+1)
pub const fn soft_limit(&self) -> Option<usize>; // budget − max(headroom, 2(D_c−1)); dynamic (ADR-M-018)
```

`depth_evict` and `depth_create` return the **current** dynamic gate
values, which start at their `Config` counterparts and may diverge
after budget-driven depth-gate adjustment (§IDEA M-7.4).
`depth_buffer` is invariant — fixed at construction.

#### Three deterministic read decompositions

`extract()`, `plateaus()`, and `contour_range()` are orthogonal
read projections of the same dual-tree (distinct from the
theory-derived triple in §THEORY M-14.2 / ADR-M-032, which lists
plateaus, PEWEI, and proportional sampling — a stochastic operation
that doesn't fit a structural-decomposition table):

|              | `extract()` / PEWEI          | `plateaus()` / BTreeMap         | `contour_range()` / decomposition |
| ------------ | ---------------------------- | ------------------------------- | --------------------------------- |
| **Tree**     | V-Tree (significance)        | G-Tree (spatial)                | G-Tree (spatial)                  |
| **Ordering** | Significance (V-depth BFS)   | Spatial (left-to-right contour) | Spatial (segment-tree descent)    |
| **Question** | What's important?            | What shape did adaptation take? | What's the energy breakdown here? |
| **Cost**     | $O(n)$, allocates            | $O(1)$ borrow                   | $O(N)$, allocates                 |
| **Path**     | Cold (checkpointing, export) | Hot (per-query)                 | Warm (analytical strip queries)   |

No single projection subsumes the others. Together they form the
complete read surface of the dual-tree.

### 5.3 Chemistry contracts

Trait bounds that constrain what coordinate spaces, intensity types,
and observation types are compatible.

#### `Coordinate`

```rust
pub trait Coordinate: Copy + PartialOrd + Debug + Default + Send + Sync + 'static {
    const BITS: u32;
    fn zero() -> Self;
    fn domain_max(n: u32) -> Self;
    fn midpoint(a: Self, b: Self) -> Self;
    fn width(start: Self, end: Self) -> Self;
    fn is_final(start: Self, end: Self, depth: u32, n: u32) -> bool;
    fn from_u64(v: u64) -> Self;
    fn next_value(self) -> Self;
    fn to_f64(self) -> f64;
    fn is_nan(self) -> bool;
    fn total_cmp(&self, other: &Self) -> std::cmp::Ordering;
}
```

Provided for `u8`, `u16`, `u32`, `u64`, `u128`, `f32`, `f64`.
Integer finality: `end − start == 1`. Float finality: `depth == n`.

#### `Accumulator`

The core bound on `GvGraph<C, V, N>`. Encodes the [Standard property
profile](idea.md#27-recommended-configurations) (P0–P5, ADR-M-033):
`zero()` is both the additive identity (P4) and the minimum (P2) of
the type's non-negative range.

```rust
pub trait Accumulator: Copy + PartialOrd + Debug + Default + Send + Sync + 'static {
    fn zero() -> Self;
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
}
```

Provided for `u8`..`u128`, `f32`, `f64`. Same-type only — cross-type
arithmetic is handled by `Observation<V>`.

Additional capabilities are gated behind independent sub-traits
(ADR-M-009 Addendum 3, ADR-M-033). All built-in types implement all
four sub-traits.

#### `Attenuatable`

Multiplicative decay — required by `decay()` / `TemporalDecay`.

```rust
pub trait Attenuatable: Accumulator {
    fn attenuate(self, factor: f64) -> Self;
}
```

#### `Weighable`

Weight projection to `f64` — required by `sample()` /
`WeightedSampler`, PEWEI `Transition::snr()`, and sampling
internals.

```rust
pub trait Weighable: Accumulator {
    fn weight(self) -> f64;
}
```

#### `Proratable`

Fractional subdivision — required by `range_sum()`,
`contour_range()`, `contour_range_energy()`, and
`Pewei::reconstruct()`.

```rust
pub trait Proratable: Accumulator {
    fn prorate(self, portion: u64, total: u64) -> Self;
    fn scale_by(self, ratio: f64) -> Self;
}
```

#### `Inspectable`

Diagnostic `f64` projection — required by `observe()`, `extract()`,
`layers()`, `check_evictions()`, `contour_range()`,
`contour_range_energy()`, and invariant checking. Semantically
distinct from `Weighable::weight` — this is for human-readable output
and debug assertions, not correctness-critical sampling weights.

```rust
pub trait Inspectable: Accumulator {
    fn to_f64_approx(self) -> f64;
    fn from_f64(v: f64) -> Self;
}
```

#### `Observation<V>`

```rust
pub trait Observation<V: Accumulator>: Copy + Debug + Send + Sync {
    fn accumulate(current: V, delta: Self) -> V;
}
```

Blanket impl: every `V: Accumulator` is `Observation<V>` (same-type,
via `Accumulator::add`). Cross-type impls provided for `f32`/`f64` →
integer types.

#### `ScalableObservation<V>`

```rust
pub trait ScalableObservation<V: Accumulator>: Observation<V> {
    fn scale(current: V, factor: Self) -> V;
}
```

Sub-trait of `Observation<V>` for observation types that can
multiplicatively scale an accumulator. Not used by the core engine —
`decay()` uses `Attenuatable::attenuate` directly (ADR-M-024).
Cross-type impls (`f64` → uint, `f32` → uint, `f64` → `f32`) provide
working `scale` methods. Same-type observations do not implement
`ScalableObservation` — use `Attenuatable::attenuate` instead.

#### `Rng`

```rust
pub trait Rng {
    fn next_f64(&mut self) -> f64;  // uniform in [0, 1)
}
```

Crate-local minimal RNG trait. When the `rand` feature is enabled, a
blanket impl covers all `rand_core::Rng` types.

### 5.4 Instruments

Four traits partition the API by permission level (ADR-M-009
Addendum 3):

```rust
pub trait SpatialRead {
    type Coord: Coordinate;
    type Accum: Accumulator;

    fn plateaus(&self) -> Cow<'_, BTreeMap<BasisEdge<Self::Coord>, Plateau<Self::Coord, Self::Accum>>>;
    fn get(&self, coord: Self::Coord) -> Cell<Self::Coord, Self::Accum>;
}

pub trait SpatialWrite: SpatialRead {
    fn observe<O: Observation<Self::Accum>>(&mut self, coord: Self::Coord, delta: O);
}

pub trait TemporalDecay: SpatialRead {
    fn decay(&mut self, root: GNodeId, attenuation: f64, q: f64);
}

pub trait WeightedSampler: SpatialRead {
    fn sample(&self, rng: &mut impl Rng) -> Option<Cell<Self::Coord, Self::Accum>>;
}
```

`decay()` was split from `SpatialWrite` into its own `TemporalDecay`
trait (ADR-M-009 Addendum 3) because it requires `V: Attenuatable` — a
sub-trait that not all accumulators implement.

`SpatialRead` and `TemporalDecay` are object-safe — every method
takes only concrete types. `SpatialWrite` and `WeightedSampler` are
not (generic / `impl Trait` method parameters) — intended for static
dispatch and capability bounding, not trait objects.

### 5.5 Trait implementations on `GvGraph`

| Trait             | Bounds on `V`                              | Behaviour                                             |
| ----------------- | ------------------------------------------ | ----------------------------------------------------- |
| `Debug`           | `Accumulator`                              | Derived                                               |
| `Clone`           | `Accumulator`                              | Deep clone of both trees                              |
| `Extend<(C, O)>`  | `Accumulator + Inspectable`                | `(coord, delta)` pairs — `O: Observation<V>`          |
| `Send + Sync`     | `Accumulator`                              | By construction — no interior mutability, no `unsafe` |
| `SpatialRead`     | `Accumulator + Inspectable`                | Delegates to concrete methods                         |
| `SpatialWrite`    | `Accumulator + Inspectable`                | Delegates to concrete methods                         |
| `TemporalDecay`   | `Accumulator + Attenuatable + Inspectable` | Delegates to concrete methods                         |
| `WeightedSampler` | `Accumulator + Inspectable + Weighable`    | Delegates to concrete methods                         |

#### Inherent vs. trait bounds

The concrete (inherent) methods on `GvGraph` carry **minimal**
bounds — `get()` needs only `Accumulator`, `sample()` needs
`Accumulator + Weighable` — while the trait impls carry the
**union** of their methods' bounds. This creates a visible gap:

| Method      | Inherent bound              | Trait impl bound                                            | Extra         |
| ----------- | --------------------------- | ----------------------------------------------------------- | ------------- |
| `get()`     | `Accumulator`               | `Accumulator + Inspectable` (`SpatialRead`)                 | `Inspectable` |
| `sample()`  | `Accumulator + Weighable`   | `Accumulator + Inspectable + Weighable` (`WeightedSampler`) | `Inspectable` |
| `observe()` | `Accumulator + Inspectable` | `Accumulator + Inspectable` (`SpatialWrite`)                | —             |

The gap is a Rust language constraint, not a design choice.
A trait impl block must satisfy every method in the trait
simultaneously; since `SpatialRead` groups `get()` (needs only
`Accumulator`) with `plateaus()` (needs `Inspectable` for
diagnostic assertions in the plateau mirror), the impl must carry
`Inspectable` for both. Splitting `SpatialRead` into `PointQuery`

- `PlateauRead` would narrow the bounds but fragment the capability
  model — one read capability is cleaner than two micro-traits.

**Practical impact is small.** `Inspectable` is implemented by every
built-in type (`u8`–`u128`, `f32`, `f64`), and any graph that calls
`observe()` already requires it. The gap only surfaces for custom
accumulator types used through read-only or sample-only trait bounds.

**Escape hatch.** Callers who need only `get()` or `sample()` and
want to avoid pulling in `Inspectable` should call the inherent
methods directly on `GvGraph<C, V, N>` rather than going through
trait bounds. This preserves the minimal per-method bounds shown
in the method index.

> **Implementation note.** The `Inspectable` requirement on
> `plateaus()` is itself an implementation-level restriction — the
> dynamic-contour-tracking mirror's debug-assertion path calls
> `to_f64_approx()` for consistency checks and tracing output; the
> core accessor (`Cow::Borrowed(&self.plateaus)`) does not need it.
> A future refactoring that conditions the diagnostic path on a
> separate bound could close this gap at source, making
> `SpatialRead`'s impl require only `Accumulator`.

#### `Extend<(C, O)>` semantics

`Extend` is exactly equivalent to calling `observe()` for each
`(coord, delta)` pair in iterator order — the implementation is a
plain `for` loop with no deferred or batched structural work. Each
observation runs the full `observe()` pipeline (accumulate →
propagate → split → rebalance → evict → plateau normalise), so the
graph's invariants hold after every element, not just at the end.

No batch optimisation is feasible here: every `observe()` may trigger
splits that change routing boundaries, rebalances that reshape the
V-Tree, or evictions that remove nodes entirely. A deferred-work
scheme would need to route subsequent observations through stale
receivers — violating §IDEA M-8's per-observation guarantees. The
cost is therefore $O(n)$ calls to `observe()` with their full
per-call amortised cost (§IDEA M-8.3), not a cheaper bulk insert.

**Ordering matters structurally, not algebraically.** Because
`Accumulator::add` is commutative (P0), aggregate values like
`total_sum()` are permutation-invariant — the same multiset of
observations always produces the same total. Internal _structure_,
however, is order-dependent: the sequence determines when thresholds
are crossed, which nodes split first, which rebalance rotations
fire, and which eviction candidates are spared (see §IDEA M-12.4
on sibling-sparing order dependence). Two permutations of the same
data yield observationally equivalent graphs (same plateaus, same
`get()` answers) in the common case, but the V-Tree's shape may
differ — an intentional consequence of the online, adaptive design.

**Not implemented:**

- `FromIterator` — would require `Config::default()`.
- `IntoIterator` — iterate via `plateaus()` instead.
- `Default` on `Config<V>` or `GvGraph` — domain-dependent parameters.

---

## 6. Surface 3 — Emulsion

`pub(crate)` **internal machinery**. Users never see or depend on
these. Listed here for completeness — not part of the public API.

| Module          | Contents                                                                               |
| --------------- | -------------------------------------------------------------------------------------- |
| `arena`         | `Arena<T>` — typed slab allocator                                                      |
| `gnode`         | `GNode<C, V>` — spatial node with all fields                                           |
| `vnode`         | `VNode<V>`, `VKind<V>`, `PackedChildren<V>`                                            |
| `gtree`         | G-Tree routing, propagation, recomputation                                             |
| `vtree`         | V-Tree insert, remove, depth, propagation                                              |
| `rebalance`     | Violation detection, promote, contract, resolve                                        |
| `split`         | `attempt_split` — subdivision logic                                                    |
| `evict`         | `evict_tip`, `scan_for_candidates`                                                     |
| `observe`       | `observe()` hot-path implementation                                                    |
| `decay`         | `decay()` implementation                                                               |
| `diagnostic`    | Consolidated audit functions (ADR-M-028)                                               |
| `graph_budget`  | Budget enforcement, depth-gate adjustment (ADR-M-030)                                  |
| `graph_extract` | PEWEI extraction, layer iteration (ADR-M-030)                                          |
| `graph_plateau` | Plateau mirror maintenance                                                             |
| `graph_query`   | Sampling, point query, range sum, contour range decomposition (ADR-M-030, ADR-M-037)   |
| `graph_traits`  | `SpatialRead` / `SpatialWrite` / `TemporalDecay` / `WeightedSampler` impls (ADR-M-030) |

Two modules are **`pub`** + `#[doc(hidden)]` as testing affordances
(ADR-M-032) but are not part of the stable public API:

| Module       | Contents                                               |
| ------------ | ------------------------------------------------------ |
| `invariants` | `assert_invariants`, `dump_gtree`, diagnostic fns      |
| `testing`    | Config presets, plan runner, fluent builder, RNG stubs |

---

## 7. Cross-cutting concerns

### 7.1 Error handling

Follows `std::collections`: panics for programmer errors (NaN
coordinates, invalid config), domain clamping for out-of-range
coordinates, `Option` for empty-tree queries (`sample` returns
`None` when total intensity is zero). No `Result` types.

### 7.2 Thread safety

`GvGraph` is `Send + Sync` — the sole interior-mutable field is
`VNode::cached_depth: AtomicU32` (ADR-M-029), which is `pub(crate)`
and not exposed through any public API; no `unsafe`.
Concurrent writes require external `RwLock` or `Mutex`.

### 7.3 Feature gates

| Feature                    | Default | Effect                                                                                                                            |
| -------------------------- | ------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `dynamic-contour-tracking` | yes     | Live plateau mirror; `plateaus()` returns `Cow::Borrowed` in O(1).                                                                |
| `serde`                    | no      | `Serialize`/`Deserialize` on all Surface 1 types (view, PEWEI, plateau, `GState`, handles). Handles serialize their opaque index. |
| `rand`                     | yes     | Blanket `Rng` impl for all `rand_core::Rng` types.                                                                                |

> **`rand` is default-on — rationale.**
>
> Most crates that interoperate with `rand` gate the integration behind
> a feature. The reason it is nonetheless default-on here is narrower
> and arguably stronger:
>
> 1. **Sampling is a first-class operation, not an add-on.**
>    `WeightedSampler::sample()` is one of the four instrument traits
>    (alongside `SpatialRead`, `SpatialWrite`, `TemporalDecay`). The
>    V-Tree exists precisely to maintain O(log n) proportional
>    sampling weights. A `GvGraph` without `sample()` is a spatial
>    index that throws away the structure that makes it interesting.
>    Gating the _ability to call_ `sample()` behind a feature would be
>    misleading — the method is always present; the feature only
>    controls whether `rand` ecosystem types satisfy its `Rng`
>    argument _automatically_.
> 2. **The alternative is worse ergonomics, not a smaller API.**
>    Without `rand`, callers must write a manual `impl Rng` wrapper
>    around their `rand::rngs::StdRng` (or whatever they use). The
>    crate's own integration tests do exactly this — three hand-rolled
>    stubs in `tests/support/` — because integration tests cannot rely
>    on default features. That boilerplate is acceptable in a test
>    harness; it is a paper-cut for application code that just wants
>    `g.sample(&mut rand::rng())` to work.
> 3. **The dependency is featherweight.** `rand_core` 0.10 is
>    `#![no_std]`-compatible, has zero transitive dependencies, no
>    proc macros, and compiles in under a second. It is already in
>    the dependency graph of virtually every Rust project that does
>    anything stochastic. The marginal cost of pulling it in is
>    effectively zero.
>
> The case _against_ default-on is real but thin: a hypothetical
> embedded consumer who needs `observe` + `get` + `plateaus` but not
> `sample`, and who cannot tolerate even `rand_core` in the dependency
> tree. That consumer already needs `default-features = false` to
> drop `dynamic-contour-tracking`; `rand` comes off for free in the
> same gesture.

> **`serde` is opt-in — rationale.**
>
> This follows the ecosystem convention (serde itself recommends
> opt-in to avoid pulling `syn`/`proc-macro2` for users who don't
> need serialization).
>
> Consumers who need serialisation (e.g. `torrust-index`) enable the
> feature explicitly: `features = ["serde"]`. This keeps the default
> dependency footprint minimal while preserving full support for
> `Serialize`/`Deserialize` on all Surface 1 snapshot types when
> opted in.

### 7.4 Test organisation

| Location                                | Access level    | Purpose                                                   |
| --------------------------------------- | --------------- | --------------------------------------------------------- |
| `#[cfg(test)] mod tests` inline         | Private fields  | Small, focused unit tests per module                      |
| `src/tests/` (`#[cfg(test)]` in lib.rs) | `pub(crate)`    | Cross-module crate-internal tests (arena/vtree/rebalance) |
| `tests/` (external integration tests)   | Public API only | Tests exercising the documented surface                   |
