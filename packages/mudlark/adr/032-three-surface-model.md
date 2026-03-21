# ADR-M-032: Three-Surface Visibility Model

**Status:** Decided
**Date:** 2026-03-03
**Relates to:** [ADR-M-025](025-public-api-surface.md) (public API surface),
[ADR-M-030](030-graph-module-decomposition.md) (graph module decomposition)
**API:** [§API M-2](../docs/api.md#2-three-surface-visibility-model)
**Surface:** 1 + 2 + 3 (all)

---

## Context

Every ADR in this repository makes visibility decisions — `pub`, `pub(crate)`,
or private — yet no single document states the governing principle behind those
choices. The result is consistent practice but under-theorised rationale:
reviewers must reverse-engineer the policy from 30+ individual decisions.

We need a concise mental model that:

1. Classifies every public symbol into exactly one of a small number of
   _surfaces_.
2. Gives each surface a memorable name so ADR prose can reference it.
3. Makes the _crossing rules_ between surfaces explicit.
4. Provides **testable criteria** for deciding which surface a symbol
   belongs on — so classification is principled, not ad hoc.

---

## Decision

The crate's API is organised into **three surfaces**. A symbol's
surface is determined by three assignment tests (§ tests below). Items
that fail all three tests belong behind `#[doc(hidden)]` or a
feature gate. The three surfaces are named by analogy with silver
halide photography (§ naming rationale below).

---

## Naming rationale — silver halide film

The crate's operational cycle — expose, regress, measure — corresponds
to the physics of **undeveloped silver halide film** undergoing
**latent image regression**.

In a silver halide emulsion:

1. A **photon strike** liberates an electron in an AgBr crystal, reducing
   Ag⁺ to Ag⁰. Multiple strikes at the same grain build a cluster of
   metallic silver atoms — the **latent image center**.
2. Sub-critical clusters are thermodynamically unstable. Given time (or
   heat) the silver atoms **disperse back** into the halide lattice.
   The grain re-sensitizes and can be exposed again. This is **latent
   image regression**.
3. A **contact print** can be made from the negative at any moment — a
   stable, detached record. The negative itself keeps changing.

`GvGraph` is undeveloped film that is **never fixed**: it is
continuously exposed and regressing. There is no develop/fix/wash
cycle. This single observation anchors the three-surface model and
names the surfaces:

| Surface | Name         | Photographic role                               | Rust role                                            |
| ------- | ------------ | ----------------------------------------------- | ---------------------------------------------------- |
| 1       | **Prints**   | Contact prints — stable records from a negative | Lightweight view types users hold and inspect        |
| 2       | **Film**     | Loaded film — opaque active medium              | Opaque operational types users interact through      |
| 3       | **Emulsion** | Crystal microstructure in gelatin               | `pub(crate)` internal machinery; not part of the API |

All modules are `pub(crate)` — types are re-exported flat from the
crate root, giving each public type exactly one canonical path.

---

## Crossing rules

1. **Print → Film.** Film methods _return_ Prints. A Print never holds
   a `&mut` back into the Film.

2. **Film → Emulsion.** Every Film method is a thin façade that
   delegates to Emulsion functions. The public signature mentions only
   Print and Film types.

3. **Emulsion → Print.** Emulsion code _constructs_ Prints internally
   via struct literals (e.g.
   `Node { start, end, own, sum, depth, state, gnode_id, parent }`)
   — possible because Print fields are `pub`. The `parent` field is
   read from the G-node's immutable parent pointer, set once at
   split time.

4. **Emulsion ⇛ Film fields.** Emulsion code accesses `GvGraph` fields
   directly (they are `pub(crate)`), never through the public trait
   interface.

---

## Surface assignment tests

These three tests govern which surface a `pub` item belongs on.

### Primary test

A `pub` item belongs on Surface 1 or 2 if and only if it serves at
least one of:

- The **three projections** (plateaus, PEWEI, sampling;
  §THEORY M-14.2 — distinct from the API's deterministic-read
  triple which substitutes `contour_range` for sampling) **and
  supporting queries** (`range_sum`, `gnode_info`, `is_ancestor_of`)
  — the code's read surface.
- The **three mutations** (observe, decay, evict) — the code's write
  surface.
- **Construction** (new, from_observations, Config) — the code's
  setup surface.

Items serving only diagnostic, serialisation, or testing use cases
belong behind `#[doc(hidden)]` or a feature gate, following the
precedent set by `invariants` / `testing`.

**Rationale.** The three projections and three mutations are the
crate's complete operational interface (§ theory Ch. 14.2). Every
other public API entry is a supporting type for one of these six
operations. Anything that serves none of them is an internal affordance
that has leaked to the public surface.

### Handle test

A handle type on Surface 1 must have at least one public **producing**
method and at least one public **consuming** method. A handle with
only producers or only consumers is a dead-end API — it implies a
capability the crate does not expose — and belongs on Surface 3.

| Handle      | Producing methods                                    | Consuming methods                             | Verdict                 |
| ----------- | ---------------------------------------------------- | --------------------------------------------- | ----------------------- |
| `GNodeId`   | `g_root()`, `Node.gnode_id`, `BasisElement.gnode_id` | `gnode_info()`, `is_ancestor_of()`, `decay()` | **Pass**                |
| `BasisEdge` | `plateaus()`, `select_plateaus()`                    | `contour_range()`, `contour_range_energy()`   | **Pass**                |
| `VNodeId`   | `v_root()`                                           | _(none)_                                      | **Fail** — `pub(crate)` |

### Snapshot test

A Surface 1 Print's fields must describe the snapshot's **own content**,
**self-identity**, or **stable provenance** — not the graph's current
topology. The distinction turns on **when the relationship is
determined** and **whether it can change**:

- **Self-identity handle** (`Node.gnode_id`): answers "which entity
  does this snapshot describe?" Determined at creation. Immutable for
  the lifetime of the arena slot. Even if the node is later evicted,
  the handle truthfully records which node the snapshot _was_ of.

- **Stable provenance** (`Node.parent`): answers "which entity was
  split to produce this one?" Determined at creation — the parent is
  the codeword of length $d-1$ whose refinement produced this
  codeword of length $d$. Immutable: no G-Tree mutation reassigns
  parentage. Shield 3 (structural immunity) guarantees the parent
  cannot be evicted while the child exists. The parent handle is the
  serial number of the negative frame from which this print was
  enlarged — it identifies lineage, not current adjacency.

- **Mutable topology** (`left` / `right` in child linkage): answers
  "what refinements currently exist below this entity?" Changes
  through splits, evictions, and legacy promotions. These describe
  the code's current refinement state, not the snapshot's content.

Test: **is the relationship determined at the entity's creation and
immutable for its lifetime (Print), or does it reflect the graph's
evolving refinement state (Emulsion)?**

| Field      | Determined                        | Changes after creation | Verdict                     |
| ---------- | --------------------------------- | ---------------------- | --------------------------- |
| `gnode_id` | Creation                          | Never                  | Self-identity — Print       |
| `parent`   | Creation (split)                  | Never                  | Stable provenance — Print   |
| `left`     | Split / eviction / legacy promote | Yes — continuously     | Mutable topology — Emulsion |
| `right`    | Split / eviction / legacy promote | Yes — continuously     | Mutable topology — Emulsion |

### Application

| Finding                             | Which test    | Current state                                                                    |
| ----------------------------------- | ------------- | -------------------------------------------------------------------------------- |
| `from_index()`/`index()` on handles | Primary test  | `#[doc(hidden)]` — serialisation uses `serde`, not raw indices                   |
| `VNodeId` / `v_root()`              | Handle test   | Both `pub(crate)` — `VNodeId` has no public consumer                             |
| `build_plateaus()`                  | Primary test  | `#[doc(hidden)]` — serves diagnostics, not projections                           |
| `debug_plateau_basis()`             | Primary test  | `#[doc(hidden)]` — serves testing only                                           |
| `ScalableObservation`               | Primary test  | Sub-trait of `Observation` with a required `scale` method (§2.4)                 |
| `Node.parent`                       | Snapshot test | Stable provenance — on `Node` as `parent: Option<GNodeId>` (`None` at root)      |
| `GNodeChildren.left`/`.right`       | Snapshot test | Mutable topology — `#[doc(hidden)]` `gnode_children()` returning `GNodeChildren` |

`check_evictions()` **passes** the primary test — it serves the
eviction mutation, with manual rather than automatic timing.

---

## Surface 1 — Prints

Lightweight, read-only **view types** that users hold, inspect, and
store. They are the **contact prints** — stable, self-contained records
detached from the negative. None borrow the graph: no lifetimes, no
`&`-references back into the `GvGraph`. Some are trivially `Copy`
(scalar-sized), others own heap data and are `Clone`-only.

### 1.1 Spot readings

Single-region measurements from the negative.

#### `Span<C, V>` — `Copy`

Owned dyadic interval + single intensity. Purely geometric — no node
identity or tree state.

```rust
pub struct Span<C: Coordinate, V: Accumulator> {
    pub start: C,
    pub end: C,
    pub intensity: V,
    pub depth: u32,
}
```

Methods: `width() -> C`.

_Analog (load-bearing):_ **Densitometer strip reading.** A section of
the negative read as one density value over a spatial extent. The
simplest possible measurement.

#### `Cell<C, V>` — `Copy`

Snapshot of a contour cell — either a terminal G-node (leaf) or the
uncovered half of a semi-internal G-node. Returned by `sample()`,
`get()`.

```rust
pub struct Cell<C: Coordinate, V: Accumulator> {
    pub start: C,
    pub end: C,
    pub intensity: V,
    pub depth: u32,
}
```

Methods: `to_span() -> Span`, `width() -> C`, `is_final(n: u32) -> bool`.

_Analog (load-bearing):_ **Single-grain loupe reading.** The
finest-resolution measurement at a specific point — one grain, one
density.

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
    pub gnode_id: GNodeId,
    pub parent: Option<GNodeId>,
}
```

Methods: `to_span() -> Span` (uses `sum`), `to_cell() -> Option<Cell>`,
`is_terminal() -> bool`, `refinement() -> V` (`sum - own`), `width() -> C`,
`is_root() -> bool` (`parent.is_none()`).

The `gnode_id` field is a **self-identity handle** (§ snapshot test) —
it records which G-node this snapshot was taken from.

The `parent` field is **stable provenance** (§ snapshot test) — it
records which G-node was split to produce this one. `None` at the
root. Determined at creation; immutable for the node's lifetime.
Shield 3 (structural immunity) guarantees the parent outlives the
child. Callers use it for upward traversal, decay scoping, and
lineage queries.

_Analog (load-bearing):_ **Zone reading.** An area measurement
reporting both local density (`own`) and aggregate density over all
sub-grains (`sum`). The `state` tells you whether the zone has
sub-structure. The `parent` handle is the **frame number of the
source negative** — which exposure this enlargement was printed from.
A contact print always records its source. The source negative cannot
be discarded while prints derived from it are still in catalogue
(Shield 3).

#### `GState` — `Copy`

```rust
pub enum GState {
    Terminal,
    SemiInternal,
    Internal,
}
```

Derived from child-pointer presence — zero storage overhead. The three
variants correspond to a G-node's child count (0, 1, 2).

_Analog (illustrative only):_ The ADR maps these to "crystal structure
class" (isolated grain, partially-clustered aggregate, fully-subdivided
aggregate). The `Terminal` and `Internal` cases hand-wave adequately,
but **`SemiInternal` has no crystallographic meaning** — crystals do not
have exactly one sub-crystal. The state exists because binary trees can
temporarily have one child during split/evict sequences. It is a
_tree_ property, not a _material_ property. The "where it breaks"
section (§ analogy scope) addresses this.

### 1.2 Contact print

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

Methods (require `V: Proratable`): `layer_count() -> usize` ($O(1)$),
`node_count() -> usize` ($O(L)$ where $L$ = `layer_count()`),
`total_energy() -> V` ($O(n)$ where $n$ = `node_count()`),
`reconstruct(max_layer: usize) -> Vec<Span<C, V>>` (energy-conserving
partitioning of the domain).

_Analog (load-bearing):_ **The contact print itself.** Complete
developed image detached from the negative. Domain bounds baked in so
the print is self-describing. Layers are multi-pass printing — coarse
densities first, fine detail overlaid progressively (cf. dye-coupler
layers in Kodachrome).

#### `Layer<C, V>` — `Clone`

One BFS depth-level of the V-Tree.

```rust
pub struct Layer<C: Coordinate, V: Accumulator> {
    pub transitions: Vec<Transition<C, V>>,
    pub terminals: Vec<Terminal<C, V>>,
}
```

_Analog (load-bearing):_ **One dye-coupler layer.** Each layer captures
one spatial-frequency band, from coarse to fine.

#### `Transition<C, V>` — `Copy`

Phase-transition node — a region where finer spatial structure was
confirmed.

```rust
pub struct Transition<C: Coordinate, V: Accumulator> {
    pub start: C,
    pub end: C,
    pub baseline: V,
    pub total: V,
    pub refinement: V,
    pub depth: u32,
    pub v_depth: u32,
}
```

Methods (require `V: Weighable`): `snr() -> Option<f64>`, `width() -> C`.

_Analog (load-bearing):_ **Crystal-aggregate boundary.** Where
macro-density (`baseline`) gives way to resolved micro-structure
(`refinement`).

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

_Analog (load-bearing):_ **Fully resolved grain.** The smallest
developable unit — a single crystal, a single density.

### 1.3 Densitometry

Isodensity zones across the G-Tree bottom contour, and the structural
decomposition of lattice-aligned strip queries through those zones.

**Terminological note:** The ADR labels plateaus "isodensity zones"
by photographic analogy. The defining property is **uniform tree
depth** (uniform spatial resolution), not uniform intensity. Different
cells within a plateau can carry different intensities. In a real
emulsion, uniform grain size _correlates with_ but does not _equal_
uniform density. The label is illustrative; the specification is
depth-based.

#### `BasisEdge<C>` — `Copy`

```rust
pub struct BasisEdge<C: Coordinate>(pub C);
```

`Ord`-providing newtype for `BTreeMap` keys. Delegates to
`Coordinate::total_cmp` for total ordering.

_Analog (illustrative):_ **Contour boundary.** The coordinate where
the contour's terminal depth transitions from one zone to the next.

#### `Plateau<C, V>` — `Copy`

One plateau — a contiguous region of uniform contour depth.

```rust
pub struct Plateau<C: Coordinate, V: Accumulator> {
    pub basis_edge: BasisEdge<C>,
    pub start: C,
    pub end: C,
    pub depth: u32,
    pub sum: V,
}
```

Methods: `width() -> C`, `to_span() -> Span` (uses `sum` as intensity).

_Analog (illustrative):_ **Uniform-resolution zone.** A region where
all cells sit at the same depth — uniform spatial precision, one
structural stratum. "Isodensity" is approximate: resolution correlates
with, but does not determine, density.

#### `BasisElement<C, V>` — `Copy`

One element of the minimal G-node cover of a contour range.

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

_Analog (illustrative):_ **Grain cluster in the measurement strip.**
At most two boundary elements overlap the strip edges — flagged by
`is_boundary_thatch`. The "scattered-light leakage" phrasing in
the photographic analog is decorative; the flag marks a segment-tree
partial-overlap artefact, not an optical phenomenon.

#### `ContourRange<C, V>` — `Clone`

Full basis-set decomposition plus pre-computed energy fields
(§CR.6, §CR.13).

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

_Analog (illustrative):_ **Densitometry report.** The complete
breakdown of a strip reading into basis contributions and energy
fields.

#### `ContourRangeEnergy<V>` — `Copy`

Energy-only result — lightweight alternative without the basis set.

```rust
pub struct ContourRangeEnergy<V: Accumulator> {
    pub energy: V,
    pub exact_energy: V,
    pub plateau_energy: V,
    pub cross_plateau_energy: V,
    pub plateau_count: usize,
}
```

_Analog (illustrative):_ **Summary density reading.** Just the
numbers, without per-element attribution.

### 1.4 Handles

Opaque references back into the graph.

```rust
pub struct GNodeId(/* NonZeroU32 */);
```

Opaque typed arena index. `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`,
`PartialOrd`, `Ord`, `Hash`.
`#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`.

_Analog (illustrative):_ **Grain serial number.** A lab notation for
referring back to a specific crystal without exposing its position
or chemistry.

**`VNodeId`** is `pub(crate)` — it has no public consuming method
(§ handle test). `v_root()` is likewise `pub(crate)`: the accessor
cannot be more visible than the type it returns. Both serve only
the diagnostic module and crate-level tests.

**`from_index()` / `index()`** on handles are `#[doc(hidden)]`
(§ primary test: serialisation affordance, not a projection or
mutation). External serialisation uses `serde`, not raw arena indices.
Minting an arbitrary handle bypasses all graph validation; the opacity
contract depends on handles being produced exclusively by the graph.

### 1.5 Print invariants

Every Print is `Debug` and carries no mutable reference into the
graph. Most are `Copy`; `Pewei`, `Layer`, and `ContourRange` are
`Clone`-only because they own `Vec`s. All Surface 1 types — including
handles — carry
`#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]`.
Handles serialize their opaque arena index; a deserialized handle is
only valid against the graph instance that produced it.

---

## Surface 2 — Film

The **opaque operational types** through which users expose, measure,
and attenuate the medium — the film you load, the instruments you
meter with, and the chemistry contracts that constrain compatibility.

### 2.1 Film stock — `Config<V>`

Manufacturing specification — fixed before loading.

```rust
#[derive(Debug, Clone)]
pub struct Config<V: Accumulator> {
    pub split_threshold: V,
    pub depth_create: u32,
    pub depth_evict: u32,
    pub budget: Option<usize>,
    pub alpha_relax: f64,
    pub bounded_eviction: bool,
}
```

No `Default` impl. Validated at construction (D-I3, range checks).

_Analog (illustrative):_ **Film stock specification.** ISO rating,
grain-size distribution, reciprocity — chosen at the store, baked
at manufacture. The mapping holds for threshold and depth parameters.
`budget` and `bounded_eviction` are computational resource constraints
with no physical counterpart; film does not have a "maximum number of
grains."

### 2.2 The negative — `GvGraph<C, V, N>`

The live dual-tree index. Continuously exposed and regressing.

```rust
pub struct GvGraph<C: Coordinate, V: Accumulator, const N: u32> { /* opaque */ }
```

All fields are `pub(crate)`. The struct is opaque to external
consumers. `Clone` performs a deep copy of both trees.

_Analog (load-bearing):_ **Film loaded in the camera.** The camera
back is closed — you cannot see or touch the emulsion directly. You
interact only through the instruments.

_Analog breaks:_ `Clone` lets you duplicate the graph. You cannot
duplicate a piece of partially-exposed film. This is a
Rust-capability/physical-object mismatch — not actionable, but worth
noting that the "unique physical medium" aspect of the metaphor does
not hold.

### 2.3 Core operation mapping

| Operation                           | Fidelity         | Silver halide analog                                                                                                                                                                                                                                                                         |
| ----------------------------------- | ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `observe(coord, delta)`             | **Load-bearing** | Photon strike. `delta` accumulation = multi-hit cluster growth on a grain.                                                                                                                                                                                                                   |
| `decay(att, q)` where att ∈ (0,1)   | **Load-bearing** | Latent image regression. Sub-critical silver clusters thermally disperse; the grain re-sensitizes. `q` maps to cluster-size threshold — small clusters regress first, large ones persist.                                                                                                    |
| `decay(att, q)` where att > 1       | **Breaks**       | Amplification — artificially increasing latent image density. Real thermal processes only destroy latent image, never create it. There is no physical process for reinforcing latent image centers after exposure.                                                                           |
| `decay(att=0, q=1)`                 | **Illustrative** | Extreme heating destroying all latent image in a region. Plausible direction, but the $0^0=1$ convention (root preserved, descendants zeroed; §IDEA M-7.3) has no thermal analog.                                                                                                            |
| `extract()` → `Pewei`               | **Load-bearing** | Contact print from the negative — stable, detached, owning. The **never-fixed** property explains why there is no "finalize" step.                                                                                                                                                           |
| `plateaus()`                        | **Illustrative** | Isodensity contour map. The defining property is uniform _depth_ (resolution), not uniform intensity. Depth correlates with density but does not determine it.                                                                                                                               |
| `contour_range(s, e)`               | **Illustrative** | Microdensitometer strip reading. The decomposition maps well; `is_boundary_thatch` as "scattered-light leakage" is decorative — it is a segment-tree partial-overlap artefact.                                                                                                               |
| `select_plateaus(l, h)`             | **Illustrative** | Aperture selection — snapping to contour boundaries.                                                                                                                                                                                                                                         |
| `sample(rng)`                       | **Illustrative** | The _probability mechanism_ maps well: selection probability ∝ accumulated weight ↔ developer encounter probability ∝ cluster size. The _process outcome_ inverts: real development irreversibly consumes the grain; `sample()` is `&self` — perfectly non-destructive. See §analogy breaks. |
| `get(coord)`                        | **Load-bearing** | Spot densitometer reading at a point.                                                                                                                                                                                                                                                        |
| `range_sum(start, end)`             | **Illustrative** | Integrating densitometer — measures total density across a strip without decomposing into individual grain contributions.                                                                                                                                                                    |
| `from_observations(iter)`           | **Illustrative** | Bulk exposure from a pre-captured light-field — loading a pre-patterned emulsion rather than exposing grain by grain.                                                                                                                                                                        |
| `layers()`                          | **Illustrative** | Iterating dye-coupler layers of a contact print — reading the image band by band from coarsest to finest frequency.                                                                                                                                                                          |
| `check_evictions()`                 | **Illustrative** | Emulsion quality control — inspecting for loose crystals below the adhesion threshold. Not theory-motivated (the theory treats eviction as an automatic step in the observation pipeline), but fills a real operational need.                                                                |
| `gnode_info()` / `is_ancestor_of()` | **Illustrative** | Examining the negative under a microscope — structural inspection of individual grains and their spatial relationships.                                                                                                                                                                      |
| `Extend<(C, O)>`                    | **Illustrative** | Multiple exposure — batch observation.                                                                                                                                                                                                                                                       |

### 2.4 Chemistry contracts

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

Provided for `u8`–`u128`, `f32`, `f64`.

_Analog (load-bearing):_ **Film plane geometry.** The coordinate system
of the emulsion surface.

#### `Accumulator`

The core bound on `GvGraph<C, V, N>`. Encodes the standard property
profile (P0–P5, §theory 3.2).

```rust
pub trait Accumulator: Copy + PartialOrd + Debug + Default + Send + Sync + 'static {
    fn zero() -> Self;
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
}
```

Provided for `u8`–`u128`, `f32`, `f64`.

_Analog (load-bearing):_ **Emulsion chemistry.** AgBr, AgCl, AgI —
different crystals have different sensitivity profiles and combination
rules. The `Accumulator` impl _is_ the choice of crystal.

Additional capabilities are gated behind independent sub-traits.
All built-in types implement all four:

| Sub-trait      | What it gates                                                                  | Fidelity         | Analog                                                                                                                                                                  |
| -------------- | ------------------------------------------------------------------------------ | ---------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Attenuatable` | `decay()` / `TemporalDecay`                                                    | **Illustrative** | **Thermal sensitivity** — how a crystal responds to heat-driven regression. Holds for attenuation; breaks for amplification (§ analogy breaks).                         |
| `Weighable`    | `sample()` / `WeightedSampler`, `Transition::snr`                              | **Illustrative** | **Developability measure** — how exposed a grain is, expressed as a single number. The measure maps well; the non-destructive sampling process does not.                |
| `Proratable`   | `range_sum()`, `contour_range()`, `Pewei::reconstruct`                         | **Decorative**   | "Partial-grain densitometry" names the operation but does not illuminate it. Nobody in a darkroom thinks about fractional arithmetic as a distinct chemical capability. |
| `Inspectable`  | `observe()`, `extract()`, `layers()`, `check_evictions()`, `assert_invariants` | **Decorative**   | "Lab densitometer calibration" — naming only. The trait is an `f64` round-trip capability, not a physical process.                                                      |

#### `Observation<V>`

```rust
pub trait Observation<V: Accumulator>: Copy + Debug + Send + Sync {
    fn accumulate(current: V, delta: Self) -> V;
}
```

_Analog (illustrative):_ **Photon characteristics** — wavelength,
energy, spectral interaction. The `accumulate` method maps well.

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

_Analog:_ **None.** Photons expose emulsions; they do not scale
existing accumulation. The trait exists for cross-type arithmetic,
not for any operation in the photographic model.

`ScalableObservation` follows the same split-trait pattern used for
`Accumulator` sub-traits (`Attenuatable`, `Weighable`, etc.).
`Observation` itself carries only `accumulate` — no default-panic
hazard.

#### `Rng`

```rust
pub trait Rng {
    fn next_f64(&mut self) -> f64;
}
```

_Analog (illustrative):_ **Developer diffusion.** Brownian motion
through the gelatin; encounter probability ∝ cluster size. The
probability distribution maps correctly. The process-level analog
inverts: developer encounters cause irreversible grain reduction
(destructive), while `sample()` is `&self` (non-destructive). The
`Rng` models the stochastic selection mechanism, not the development
outcome.

### 2.5 Instruments

Four traits partition the API by access level:

```rust
pub trait SpatialRead {
    type Coord: Coordinate;
    type Accum: Accumulator;
    fn plateaus(&self) -> Cow<'_, BTreeMap<BasisEdge<Self::Coord>,
                                          Plateau<Self::Coord, Self::Accum>>>;
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
trait (ADR-M-009 Addendum 3) because it requires `V: Attenuatable`.
`SpatialRead` and `TemporalDecay` are object-safe — every method
takes only concrete types. `SpatialWrite` and `WeightedSampler` are
not (generic / `impl Trait` method parameters).

| Trait             | Analog             |
| ----------------- | ------------------ |
| `SpatialRead`     | Densitometer       |
| `SpatialWrite`    | Incident light     |
| `TemporalDecay`   | Thermal regressor  |
| `WeightedSampler` | Random grain probe |

### 2.6 Extended concept mapping

| Concept                               | Fidelity         | Analog                                                                                                |
| ------------------------------------- | ---------------- | ----------------------------------------------------------------------------------------------------- |
| `GvGraph`                             | **Load-bearing** | Undeveloped film — perpetually latent, never fixed                                                    |
| `Config` (threshold, depth gates)     | **Illustrative** | Film stock specification. `budget` / `bounded_eviction` have no physical counterpart.                 |
| `Accumulator` trait                   | **Load-bearing** | Emulsion chemistry — different crystals, different combination rules                                  |
| `Observation` / `ScalableObservation` | **Illustrative** | Photon characteristics. `accumulate` maps well; `ScalableObservation::scale` has no photon analog.    |
| `Coordinate`                          | **Load-bearing** | Position on the film plane                                                                            |
| G-node spatial cells                  | **Illustrative** | Individual grains accumulating silver                                                                 |
| `GState`                              | **Illustrative** | Crystal structure class. `SemiInternal` (one child) has no crystallographic meaning — see §breaks.    |
| `Arena<T>` (slab allocator)           | **Illustrative** | Gelatin matrix — the transparent substrate holding crystals in position                               |
| View types (`Span`, `Cell`, `Node`)   | **Load-bearing** | Loupe readings — lightweight, informational, detached                                                 |
| Contour range decomposition           | **Illustrative** | Microdensitometer strip analysis                                                                      |
| Subtree-scoped `decay(root, ...)`     | **Illustrative** | Localised regression via masked heating (IR laser through a mask)                                     |
| G-Tree depth = resolution             | **Load-bearing** | Grain size = acuity (Velvia 50 vs. HP5 at 3200)                                                       |
| `rebalance` / `promote` / `contract`  | **Illustrative** | Ostwald ripening during emulsion manufacture                                                          |
| `GvGraph: Clone`                      | **Breaks**       | You cannot duplicate a piece of partially-exposed film. Rust capability with no physical counterpart. |

---

## Surface 3 — Emulsion

`pub(crate)` **internal machinery** — the crystal microstructure in
gelatin. Users never see or depend on these.

| Module          | Contents                                                                                      |
| --------------- | --------------------------------------------------------------------------------------------- |
| `arena`         | `Arena<T>` — typed slab allocator (gelatin matrix)                                            |
| `gnode`         | `GNode<C, V>` — full spatial node (grain)                                                     |
| `vnode`         | `VNode<V>`, `VKind<V>`, `PackedChildren<V>`                                                   |
| `gtree`         | G-Tree routing, propagation, recomputation                                                    |
| `vtree`         | V-Tree insert, remove, depth, propagation                                                     |
| `rebalance`     | Violation detection, promote, contract, resolve                                               |
| `split`         | `attempt_split` — subdivision logic                                                           |
| `evict`         | `evict_tip`, `scan_for_candidates`                                                            |
| `observe`       | `observe()` hot-path implementation                                                           |
| `decay`         | `decay()` implementation                                                                      |
| `diagnostic`    | Consolidated audit and tracing helpers (ADR-M-028)                                            |
| `graph_budget`  | Budget enforcement, depth-gate adjustment, eviction (ADR-M-030 §E)                            |
| `graph_extract` | PEWEI extraction and layer iteration (ADR-M-030 §D)                                           |
| `graph_plateau` | Plateau mirror maintenance                                                                    |
| `graph_query`   | Read-side queries: sampling, point lookup, range sum, contour range (ADR-M-030 §C, ADR-M-037) |
| `graph_traits`  | Trait impls on `GvGraph`: `SpatialRead`, `SpatialWrite`, etc. (ADR-M-030 §F)                  |

Two modules are **`pub`** + `#[doc(hidden)]` as testing affordances
but are **not** part of the stable public API:

| Module       | Contents                                               |
| ------------ | ------------------------------------------------------ |
| `invariants` | `assert_invariants`, `dump_gtree`, diagnostic fns      |
| `testing`    | Config presets, plan runner, fluent builder, RNG stubs |

The following methods on `GvGraph` are `#[doc(hidden)]` testing/diagnostic
affordances — public for integration-test access, not part of the stable
surface:

| Method                  | Purpose                                  | Why `#[doc(hidden)]`                                     |
| ----------------------- | ---------------------------------------- | -------------------------------------------------------- |
| `build_plateaus()`      | Rebuild plateau map from scratch via DFS | Serves invariant checking, not user queries              |
| `debug_plateau_basis()` | Expose per-plateau basis bookkeeping     | Integration-test diagnostics only                        |
| `gnode_children()`      | Return current child handles of a G-node | Mutable topology, not snapshot content (§ snapshot test) |
| `v_root()`              | Return V-Tree root handle                | Returns `VNodeId` (`pub(crate)`); no public consumer     |

---

## Analogy scope and integrity

The silver halide metaphor is **load-bearing** for the expose / regress
/ print cycle and the three-surface boundary model. It becomes
**illustrative** for densitometry and state naming, **decorative** for
sub-trait naming (labels without insight), and **breaks** where the
operational model exceeds what physics permits.

### Load-bearing (the analogy does real design work)

The following mappings illuminate design decisions — removing them
would make the ADR's reasoning harder to follow:

- **`observe()` ↔ photon strike.** Accumulation directionality,
  multi-hit cluster growth.
- **`decay(att < 1)` ↔ thermal regression.** Subband-adaptive
  attenuation; small clusters regress first.
- **`extract()` ↔ contact print.** Stable, detached, owning snapshot
  from a continuously-changing medium. The "never fixed" observation
  explains why there is no finalise step.
- **Prints / Film / Emulsion naming.** The three-surface boundaries
  follow the physical boundaries: prints are detached records, the
  camera back is closed (opaque), and nobody outside the lab handles
  gelatin.
- **Crossing rules.** "A Print never holds a `&mut` back into the
  Film" is simultaneously a Rust ownership statement and a physical
  truth about contact prints. The analogy does structural work here.

### Illustrative (useful mental image, not precisely correct)

- **`sample()` / `Rng` ↔ developer diffusion.** The probability
  mechanism maps correctly (selection ∝ weight ↔ encounter ∝ cluster
  size). The process outcome inverts: development is irreversibly
  destructive; `sample()` is `&self`.
- **Plateaus ↔ isodensity zones.** The defining property is uniform
  _depth_, not uniform _density_. "Isodensity" is a useful shorthand
  that conflates resolution with density.
- **`GState` ↔ crystal structure class.** `Terminal` and `Internal`
  hand-wave adequately. `SemiInternal` has no crystallographic
  meaning — crystals do not have exactly one sub-crystal. The state
  is a tree property (child-pointer count), not a material property.
- **`Config.budget` / `bounded_eviction`.** Computational resource
  constraints with no physical counterpart. Film does not have a
  "maximum grain count."

### Decorative (naming only, no illumination)

These name the operations but do not illuminate them. They are
algebraic capabilities dressed in photographic language for uniform
coverage. Compare with `Attenuatable` ↔ thermal sensitivity, which
genuinely illuminates.

- **`Proratable` ↔ partial-grain densitometry.** Fractional arithmetic
  is not a distinct chemical capability.
- **`Inspectable` ↔ lab densitometer calibration.** An `f64` round-trip
  capability, not a physical process.

### Where the analogy breaks

The following concepts have no valid silver halide analog. The metaphor
should not be extended to cover them.

| Concept                                                          | Why it breaks                                                                                                                                                                                                                                                               |
| ---------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Adaptive spatial subdivision (`split`)                           | Film grain is fixed at manufacture. You cannot grow finer grain where you expose more. This is the single deepest break — there is no photographic analog for exposure-driven resolution increase.                                                                          |
| Non-destructive readout (`extract`, `plateaus`, `get`, `sample`) | You cannot probe a latent image without additional exposure. `sample()` is the sharpest case: its analog (development) is the _most_ destructive step in photography, yet the operation is `&self`. This is CCD/CMOS sensor behaviour, not film.                            |
| Amplification via `decay(att > 1)`                               | Real latent image regression is strictly lossy — thermal energy only disperses silver atoms, never concentrates them. The API's amplification regime (reinforcement of surviving codewords) has no thermal analog. The attenuation mapping covers half the parameter range. |
| Deterministic observation                                        | Real photon→latent-image conversion is quantum-probabilistic (~1–3% quantum efficiency). `observe()` is deterministic.                                                                                                                                                      |
| Unbounded accumulation                                           | Real grains saturate (solarisation reversal at extreme overexposure). PEWEI accumulators can grow without bound.                                                                                                                                                            |
| Hierarchical tree structure                                      | Film is a ~monolayer of randomly scattered grains on a 2D plane. The G-Tree/V-Tree hierarchy has no physical counterpart.                                                                                                                                                   |
| `GvGraph: Clone`                                                 | You cannot duplicate a piece of partially-exposed film. The "unique physical medium" aspect of the metaphor does not hold.                                                                                                                                                  |
| `ScalableObservation::scale`                                     | Photons expose emulsions; they do not scale existing accumulation. The method exists for cross-type arithmetic, not for any photographic process.                                                                                                                           |

---

## ADR tagging convention

Every existing ADR receives a `**Surface:**` metadata line indicating
which surface it primarily governs:

- `Surface: 1 (Prints)` — defines or constrains a view type.
- `Surface: 2 (Film)` — defines or constrains the operational interface.
- `Surface: 3 (Emulsion)` — defines or constrains internal machinery.

ADRs that cross surfaces use the primary surface and note the secondary
in their body text.

---

## Consequences

- **§API M-2** is restructured around the three surfaces instead of a
  flat public/internal split.
- **lib.rs** re-exports are grouped by surface with explanatory comments.
- ADR reviewers can immediately see which surface a decision affects.
- Any ad-hoc visibility punch-lists are retired — the surface tags and
  §API M-2 tables are the single source of truth.
- The **three assignment tests** (primary, handle, snapshot) provide
  mechanical criteria for surface classification. New public items are
  evaluated against all three tests before assignment.
- The analogy section explicitly marks each mapping's **fidelity**
  (load-bearing, illustrative, decorative, breaks), so future ADR
  authors know which parts of the metaphor to lean on and which to
  avoid.
- **`VNodeId`** and **`v_root()`** are both `pub(crate)`
  (§ handle test — the accessor cannot be more visible than the
  type it returns).
- **`gnode_info()`** returns `Node` — the same Print type that
  `layers()` and PEWEI extraction produce. `GNodeInfo` is removed
  entirely: `Node` serves Surface 1, and Emulsion code reads `GNode`
  fields directly through `pub(crate)` access (crossing rule 4).
  `gnode_info()` is the only method that accepts a `GNodeId` and
  returns a `Node` for a potentially non-terminal G-node — terminal,
  semi-internal, or internal. The returned `Node`'s `gnode_id` field
  equals the `id` the caller passed in; this redundancy is intentional,
  maintaining consistency with every other `Node`-producing path.
- **`Node<C, V>`** gains `parent: Option<GNodeId>` — stable
  provenance, passing the refined snapshot test. The `parent`
  field is determined at creation (split time) and immutable.
  Shield 3 guarantees the parent outlives the child.
- **`GNodeLinks`** is replaced by **`GNodeChildren`** (fields: `left`,
  `right` only), remaining `#[doc(hidden)]`.
- **`gnode_links()`** is renamed **`gnode_children()`**.
- The **snapshot test** distinguishes stable provenance
  (immutable, creation-time) from mutable topology (evolving
  through mutations). The coding-theoretic grounding: a codeword's
  parent is part of its definition (refinement of a shorter
  codeword); a codeword's children are part of the code's current
  state.
- Tree-upward traversal is available on the primary API surface
  without `#[doc(hidden)]` methods.
- Child linkage (`left`/`right`) is available through the
  `#[doc(hidden)]` method `gnode_children()` on `GvGraph`, which
  returns a `GNodeChildren` struct. This is public for
  integration-test access but not part of the stable surface.
- **`from_index()` / `index()`** on handles are `#[doc(hidden)]`;
  external serialisation uses `serde`.
- **`build_plateaus()`**, **`debug_plateau_basis()`**, and
  **`gnode_children()`** are `#[doc(hidden)]`.
- **`ScalableObservation<V>: Observation<V>`** is a separate sub-trait
  with a required `scale` method, following the split-trait pattern
  used for `Accumulator`. `Observation` itself carries only
  `accumulate`.
