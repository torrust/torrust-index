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
   via struct literals (`pub` fields).
4. **Emulsion ⇛ Film fields.** Emulsion code accesses `GvGraph` fields
   directly (`pub(crate)`), never through the public trait interface.

---

## 3. Crate root re-exports

```rust
// Surface 1 — Prints (view types users hold and inspect).
pub use contour_range::{BasisElement, ContourRange, ContourRangeEnergy};
pub use gnode::GState;
pub use handle::{GNodeId, VNodeId};
pub use pewei::{Layer, Pewei, Terminal, Transition};
pub use plateau::{BasisEdge, Plateau};
pub use view::{Cell, Node, Span};

// Surface 2 — Film (opaque operational types).
pub use graph::{Config, GvGraph};
pub use traits::{Accumulator, Attenuatable, Coordinate, Inspectable,
                 Observation, Proratable, Rng, SpatialRead,
                 SpatialWrite, TemporalDecay, Weighable,
                 WeightedSampler};
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

Snapshot of a terminal G-node (leaf cell). Returned by `sample()`,
`get()`.

```rust
pub struct Cell<C: Coordinate, V: Accumulator> {
    pub start: C,
    pub end: C,
    pub intensity: V,  // g.own (= g.sum for terminals)
    pub depth: u32,
}
```

Methods: `to_span() -> Span`, `width() -> C`, `is_final(n: u32) -> bool`.

#### `Node<C, V>` — `Copy`

Snapshot of any G-node — carries `own`, `sum`, and `state`.

```rust
pub struct Node<C: Coordinate, V: Accumulator> {
    pub start: C,
    pub end: C,
    pub own: V,
    pub sum: V,
    pub depth: u32,
    pub state: GState,
}
```

Methods: `to_span() -> Span` (uses `sum`), `to_cell() -> Option<Cell>`,
`is_terminal() -> bool`, `refinement() -> V` (`sum - own`), `width() -> C`.

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

Methods (require `V: Proratable`): `layer_count() -> usize`,
`node_count() -> usize`, `total_energy() -> V` ($O(n)$),
`reconstruct(max_layer: usize) -> Vec<Span<C, V>>` (energy-conserving
partitioning of the domain).

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

Methods: `width() -> C`, `to_span() -> Span`.

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
plus pre-computed energy fields (§CR.6, §CR.13).

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

### 4.4 Handles

Opaque references back into the graph.

```rust
pub struct GNodeId(/* NonZeroU32 */);
pub struct VNodeId(/* NonZeroU32 */);
```

Opaque typed arena indices. Used by `decay()` (`root` parameter),
`g_root()`, `v_root()`. `Copy`, `Debug`, `PartialEq`, `Eq`, `Hash`.

`from_index()` and `index()` are public — external consumers can
mint and inspect handles. (ADR-M-032 implementation checklist
acknowledges this as potential future tightening.)

### 4.5 Invariants

Every Print is `Debug` and carries no mutable reference into the
graph. Most are `Copy`; `Pewei`, `Layer`, and `ContourRange` are
`Clone`-only because they own `Vec`s. All Surface 1 types (except
handles) carry
`#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`.

---

## 5. Surface 2 — Film

The **opaque operational types** through which users expose, measure,
and attenuate the medium.

### 5.1 `Config<V>`

Film stock specification — fixed before loading.

```rust
#[derive(Debug, Clone)]
pub struct Config<V: Accumulator> {
    pub split_threshold: V,       // θ — minimum intensity to split
    pub depth_create: u32,        // D_create — max V-depth for split
    pub depth_evict: u32,         // D_evict — min V-depth for eviction
    pub budget: Option<usize>,    // hard node-count ceiling
    pub alpha_relax: f64,         // relaxation threshold ∈ (0, 1)
    pub bounded_eviction: bool,   // stop early once under budget
}
```

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

#### Observation (mutation)

Available when `V: Inspectable`.

```rust
pub fn observe<O: Observation<V>>(&mut self, coord: C, delta: O);
```

Pipeline: route → accumulate → propagate → violation check → split →
rebalance → depth control → budget-guarded eviction.

#### Point query

```rust
pub fn get(&self, coord: C) -> Cell<C, V>;
```

Infallible. Routes to the terminal or semi-internal receiver.
Out-of-domain coordinates are clamped. NaN panics. For semi-internal
nodes, returns a `Cell` with the trimmed half-interval.

Cost: $O(\text{depth})$.

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
sentinel `2^N`).

```rust
pub fn contour_range(
    &self,
    start: BasisEdge<C>,
    end: BasisEdge<C>,
) -> Option<ContourRange<C, V>>;
```

Full basis-set decomposition of a lattice-aligned half-open interval
`[start, end)` (§CR.2–§CR.6).  Endpoints must lie on the endpoint
lattice — the set of plateau basis edges plus the domain sentinel
`2^N`.  Returns `None` if either endpoint is not in the lattice or
`start >= end`.

The result contains the basis set (§CR.2) plus pre-computed energy
fields: contour range energy (§CR.6, `Σ basis.sum`), exact energy
(§CR.13, `range_sum`), plateau energy, and cross-plateau energy.

Cost: $O(N)$ — segment-tree decomposition plus one `range_sum()` pass.

```rust
pub fn contour_range_energy(
    &self,
    start: BasisEdge<C>,
    end: BasisEdge<C>,
) -> Option<ContourRangeEnergy<V>>;
```

Energy-only variant — same lattice-endpoint requirement, returns only
the scalar energy fields without allocating the basis set.
Delegates to `contour_range()` internally.

Cost: $O(N)$.

#### Plateau selection

Available when `V: Proratable + Inspectable`.

```rust
pub fn select_plateaus(&self, lo: C, hi: C) -> Option<(BasisEdge<C>, BasisEdge<C>)>;
```

Given arbitrary dyadic coordinates `[lo, hi)`, snaps outward to the
nearest lattice-aligned endpoints that span all overlapping plateaus
(§CR.12).  The returned pair is valid for `contour_range()` and
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

Expected cost: $O(1.44\, H + 1.67)$ where $H$ is the Shannon
entropy of the intensity distribution.

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

Cost: $O(L + S)$ — every V-node visited once.

#### Layer iteration

Available when `V: Inspectable`.

```rust
pub fn layers(&self) -> impl Iterator<Item = (usize, Node<C, V>)> + '_;
```

Lazy V-Tree BFS yielding `(layer_index, Node)`. Streaming counterpart
to `extract()` — same traversal order, same node classification, but
yields `Node` view types without allocating the full `Pewei` snapshot.

#### Temporal decay

Available when `V: Attenuatable + Inspectable`.

```rust
pub fn decay(&mut self, root: GNodeId, attenuation: f64, q: f64);
```

Subband-adaptive temporal decay. `root` specifies the G-subtree
(use `g_root()` for the entire tree).

- `attenuation` — base factor in `(0, ∞)`. Values in `(0, 1)` cause
  exponential decay, `1.0` is a no-op.
- `q` — selectivity in `[0, 1]`:
  - $q = 0$: uniform — all subbands decay at the same rate.
  - $q > 0$: selective — coarse persists, fine fades faster.
  - $q = 1$: maximum selectivity.

Per-depth factor:

$$\ln\lambda(d) = \ln(\text{att}) \cdot (1 + q \cdot (2d_\text{local}/D - 1))$$

Panics if `attenuation <= 0` or NaN, or `q` outside `[0, 1]` or NaN.
Does not trigger eviction. Invariants (G-I1, V-I1, V-I3) are restored
via trailing rebalance.

#### Manual eviction

Available when `V: Inspectable`.

```rust
pub fn check_evictions(&mut self) -> u32;
```

Evict all eligible candidates (unbounded). Returns the number evicted.
Trailing rebalance restores V-I3.

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
pub const fn v_root(&self) -> Option<VNodeId>;

// Dynamic depth control.
pub const fn depth_evict(&self) -> u32;
pub const fn depth_create(&self) -> u32;
pub const fn depth_buffer(&self) -> u32;       // D_evict − D_create (fixed)
pub const fn headroom(&self) -> usize;         // 3^(buffer+1)
pub const fn soft_limit(&self) -> Option<usize>;
```

#### Three complementary projections

`extract()`, `plateaus()`, and `contour_range()` are orthogonal
projections of the same dual-tree:

|              | `extract()` / PEWEI          | `plateaus()` / BTreeMap         | `contour_range()` / decomposition  |
| ------------ | ---------------------------- | ------------------------------- | ---------------------------------- |
| **Tree**     | V-Tree (significance)        | G-Tree (spatial)                | G-Tree (spatial)                   |
| **Ordering** | Significance (V-depth BFS)   | Spatial (left-to-right contour) | Spatial (segment-tree descent)     |
| **Question** | What's important?            | What shape did adaptation take? | What's the energy breakdown here?  |
| **Cost**     | $O(n)$, allocates            | $O(1)$ borrow                   | $O(N)$, allocates                  |
| **Path**     | Cold (checkpointing, export) | Hot (per-query)                 | Warm (analytical strip queries)    |

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
    fn scale(current: V, factor: Self) -> V { unimplemented!() }
}
```

`scale` has a default implementation that panics. Cross-type impls
(`f64` → uint, `f32` → uint) provide working `scale` methods.
Same-type `scale` is not used by the core engine — `decay()` uses
`Attenuatable::attenuate` directly (ADR-M-024).

Blanket impl: every `V: Accumulator` is `Observation<V>` (same-type,
via `Accumulator::add`). Cross-type impls provided for `f32`/`f64` →
integer types.

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

`SpatialRead` is object-safe. `SpatialWrite`, `TemporalDecay`, and
`WeightedSampler` are not (generic / `impl Trait` parameters) —
intended for static dispatch and capability bounding, not trait
objects.

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

**Not implemented:**

- `FromIterator` — would require `Config::default()`.
- `IntoIterator` — iterate via `plateaus()` instead.
- `Default` on `Config<V>` or `GvGraph` — domain-dependent parameters.

---

## 6. Surface 3 — Emulsion

`pub(crate)` **internal machinery**. Users never see or depend on
these. Listed here for completeness — not part of the public API.

| Module          | Contents                                                                             |
| --------------- | ------------------------------------------------------------------------------------ |
| `arena`         | `Arena<T>` — typed slab allocator                                                    |
| `gnode`         | `GNode<C, V>` — spatial node with all fields                                         |
| `vnode`         | `VNode<V>`, `VKind<V>`, `PackedChildren<V>`                                          |
| `gtree`         | G-Tree routing, propagation, recomputation                                           |
| `vtree`         | V-Tree insert, remove, depth, propagation                                            |
| `rebalance`     | Violation detection, promote, contract, resolve                                      |
| `split`         | `attempt_split` — subdivision logic                                                  |
| `evict`         | `evict_tip`, `scan_for_candidates`                                                   |
| `observe`       | `observe()` hot-path implementation                                                  |
| `decay`         | `decay()` implementation                                                             |
| `diagnostic`    | Consolidated audit functions (ADR-M-028)                                               |
| `graph_budget`  | Budget enforcement, depth-gate adjustment (ADR-M-030)                                  |
| `graph_extract` | PEWEI extraction, layer iteration (ADR-M-030)                                          |
| `graph_plateau` | Plateau mirror maintenance                                                           |
| `graph_query`   | Sampling, point query, range sum, contour range decomposition (ADR-M-030, ADR-M-037)  |
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

| Feature                    | Default | Effect                                                                                                            |
| -------------------------- | ------- | ----------------------------------------------------------------------------------------------------------------- |
| `dynamic-contour-tracking` | yes     | Live plateau mirror; `plateaus()` returns `Cow::Borrowed` in O(1).                                                |
| `serde`                    | yes     | `Serialize`/`Deserialize` on all Surface 1 snapshot types (view, PEWEI, plateau, `GState`) except opaque handles. |
| `rand`                     | yes     | Blanket `Rng` impl for all `rand_core::Rng` types.                                                            |

### 7.4 Test organisation

| Location                                | Access level    | Purpose                                                   |
| --------------------------------------- | --------------- | --------------------------------------------------------- |
| `#[cfg(test)] mod tests` inline         | Private fields  | Small, focused unit tests per module                      |
| `src/tests/` (`#[cfg(test)]` in lib.rs) | `pub(crate)`    | Cross-module crate-internal tests (arena/vtree/rebalance) |
| `tests/` (external integration tests)   | Public API only | Tests exercising the documented surface                   |
