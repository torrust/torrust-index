# ADR-M-009: Trait Decomposition

**Status:** Decided (trait surface revised — see Addendum 2;
accumulator sub-traits added — see Addendum 3)  
**Date:** 2026-02-24  
**Updated:** 2026-03-04  
**Relates to:** [ADR-M-010](010-observation-generics.md) (observation
generics), [ADR-M-019](019-sampling-semantics.md) (sampling semantics),
[ADR-M-025](025-public-api-surface.md) (public API surface),
[ADR-M-026](026-point-query-and-plateau-semantics.md) (plateau semantics
/ thatching), [ADR-M-032](032-three-surface-model.md) (three-surface
model), [ADR-M-033](033-v-generic-importance-properties.md) (V generic bounds)  
**Spec:** §IDEA M-5.6 (plateaus), §IDEA M-6.5 (proportional sampling)  
**API:** [§API M-5.4](../docs/api.md#54-instruments) (instruments),
[§API M-5.3](../docs/api.md#rng) (Rng)  
**Surface:** 2 (Film)

## Context

The public API needs trait abstractions for generic code and
capability restriction at API boundaries. The question is how to
partition the methods across traits.

## Options Considered

- **C1:** Single `SpatialIndex` trait with everything.
- **C2:** Three traits: read, write, sample.
- **C3:** Fine-grained traits per operation (query, mutate, sample,
  iterate, extract).

## Decision

**C2 — Four capability traits: `SpatialRead` + `SpatialWrite` +
`TemporalDecay` + `WeightedSampler`.**

The original three-way split (read / write / sample) is extended to
a four-way split. `decay()` is extracted from `SpatialWrite` into
its own `TemporalDecay` trait (see Addendum 3), because decay
requires `V: Attenuatable` — a sub-trait that not all accumulators
need to implement. `SpatialWrite` now covers observation only,
requiring only `V: Accumulator`.

The _method surface_ of `SpatialRead` has been revised by
Addendum 2 to a thin 2-method trait (ADR-M-025 DC-025-1, ADR-M-026).

```rust
/// Immutable spatial read surface: plateau projection + point query.
pub trait SpatialRead {
    type Coord: Coordinate;
    type Accum: Accumulator;

    fn plateaus(&self) -> Cow<'_, BTreeMap<BasisEdge<Self::Coord>, Plateau<Self::Coord, Self::Accum>>>;
    fn get(&self, coord: Self::Coord) -> Cell<Self::Coord, Self::Accum>;
}

/// Mutation: observations.
pub trait SpatialWrite: SpatialRead {
    fn observe<O: Observation<Self::Accum>>(&mut self, coord: Self::Coord, delta: O);
}

/// Temporal decay: subband-adaptive attenuation.
/// Requires `Self::Accum: Attenuatable`.
pub trait TemporalDecay: SpatialRead {
    fn decay(&mut self, root: GNodeId, attenuation: f64, q: f64);
}

/// Weighted random sampling over entries by intensity.
/// Requires `Self::Accum: Weighable`.
pub trait WeightedSampler: SpatialRead {
    fn sample(&self, rng: &mut impl Rng) -> Option<Cell<Self::Coord, Self::Accum>>;
}
```

## Rationale

- **Read/write split** follows the standard Rust permission boundary
  (`Index`/`IndexMut` pattern). Handing out `&dyn SpatialRead` gives
  a truly immutable view — useful for concurrent readers or snapshot
  inspection.
- **`TemporalDecay`** is split from `SpatialWrite` because decay
  requires `V: Attenuatable` — a sub-trait of `Accumulator` that
  defines how multiplicative scaling works for the type. Not every
  accumulator needs to support decay (e.g. an ordinal ranking type
  has no meaningful attenuation), so the bound should not infect
  `SpatialWrite`.
- **`WeightedSampler`** is thin (one method) but conceptually distinct:
  sampling requires an RNG, has entropy-dependent cost, and requires
  `V: Weighable` — the ability to project `V` to `f64` for
  proportional weight calculation. Keeping it separate lets
  downstream code express "I need to read but never sample" without
  pulling in the `rand` dependency or requiring `Weighable`.
- The concrete type `GvGraph` implements all four when `V` satisfies
  the required sub-traits. No trait objects are required for normal
  use — the traits exist for generic code and for restricting
  capability at API boundaries.

## Object safety

`SpatialRead` is object-safe:
`&dyn SpatialRead<Coord = u64, Accum = u64>` compiles. This
enables trait-object mocking in test harnesses.

`SpatialWrite`, `TemporalDecay`, and `WeightedSampler` are **not**
object-safe — `SpatialWrite` due to `O: Observation<Self::Accum>`
on `observe()`, `WeightedSampler` due to `impl Rng` on `sample()`.
`TemporalDecay` could be made object-safe (no generics on `decay()`),
but is kept alongside the others as a static-dispatch capability
bound. (See §API M-5.4.)

## Consequences

- Four traits, each with a clear permission level and sub-trait
  requirement.
- `GvGraph` implements all four when `V` satisfies the bounds —
  no ceremony for direct use with built-in types.
- `&dyn SpatialRead` is a read-only capability handle.
- `SpatialWrite` requires only `V: Accumulator` — the minimal
  bound for observe + structural operations.
- `TemporalDecay` is gated on `V: Attenuatable` — types that
  cannot be meaningfully attenuated simply don't impl the trait.
- `WeightedSampler` is gated on `V: Weighable` — types that
  cannot project to `f64` for proportional weights don't impl the
  trait. Can be feature-gated behind `rand` if desired.
- `Accumulator` is additive on the observe hot-path (no `sub`
  needed). G-sum propagation uses recomputation instead of delta
  propagation — see [ADR-M-012](012-g-sum-recomputation.md).
  `Accumulator::sub` was later added for offline reconstruction
  (remainder energy) — see [ADR-M-023](023-pewei-reconstruction.md)
  DC-023-3.

---

## Addendum 1: Rng Trait Bound and `rand` Dependency (Phase 4)

**Status:** Decided  
**Date:** 2026-02-26  
**Phase:** 4 (Output)

### Context

`WeightedSampler::sample` is declared above with `rng: &mut impl Rng`.
Phase 4 must implement this, which requires choosing what `Rng` is
and whether the crate depends on the `rand` ecosystem.

### Q1: Rng trait definition (DC-009-1)

| Option | Definition                                                                | Notes                                                                                               |
| ------ | ------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| A      | Crate-local minimal trait: `trait Rng { fn next_f64(&mut self) -> f64; }` | Zero dependencies. Users impl for their own RNG or wrap `rand::Rng`.                                |
| B      | Re-export `rand_core::RngCore` as the bound                               | Proven API, ecosystem-standard. Adds `rand_core` as a required dependency (~30 KB).                 |
| C      | Use `rand::Rng` (the full trait) as the bound                             | Adds `rand` (~300 KB compile). Richer API (`gen_range`, etc.) but overkill for a single `f64` draw. |

**Considerations:**

- The sampling algorithm (§IDEA M-6.5) needs exactly one operation:
  draw a uniform `f64` in `[0, 1)` (or equivalently a `u64` and
  convert). Option A is sufficient.
- Option A means users with `rand` already in their dependency graph
  must write a 3-line adapter. A blanket impl
  `impl<T: rand_core::RngCore> Rng for T` in a feature-gated module
  eliminates this boilerplate.
- Option B (`rand_core`) is the lightest crate in the `rand`
  ecosystem and is unlikely to cause version conflicts since it has
  very few transitive dependencies.
- The crate's required dependencies at time of decision are `tracing`
  only. Adding even `rand_core` increases the surface. A minimal
  trait keeps the crate dependency-free.

### Q2: `rand` as a dependency (DC-009-2)

| Option | Dependency story                                                                                   | Notes                                                                         |
| ------ | -------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------- |
| A      | No `rand` dependency — crate-local trait. Provide a `rand` Cargo feature that adds a blanket impl. | Zero-cost default. Users who want `rand` interop opt in.                      |
| B      | `rand_core` as a required dependency.                                                              | Minimal, standard. Always interoperable with `rand` ecosystem.                |
| C      | `rand` as a required dependency.                                                                   | Heaviest. Provides `thread_rng()` convenience but this crate doesn't need it. |

**Considerations:**

- Option A + feature gate is the standard pattern for crates that
  want to interoperate with `rand` without mandating it (e.g.
  `uuid`, `petgraph`).
- If Option A is chosen for Q1, the feature gate adds:
  ```toml
  [features]
  rand = ["dep:rand_core"]
  [dependencies]
  rand_core = { version = "0.9", optional = true }
  ```
  and a blanket impl gated on `#[cfg(feature = "rand")]`.

### Decision

**DC-009-1: Option A — crate-local minimal trait.**

```rust
/// Minimal RNG trait for proportional sampling.
///
/// The sampling algorithm requires exactly one operation: draw a
/// uniform `f64` in `[0, 1)`. This trait captures that single
/// capability without pulling in external dependencies.
pub trait Rng {
    /// Return a uniformly distributed `f64` in `[0.0, 1.0)`.
    fn next_f64(&mut self) -> f64;
}
```

The crate's required dependencies at time of decision are `tracing`
only. A crate-local trait keeps the required dependency set minimal.
The sampling hot loop draws one `f64` per V-Tree level — no richer
RNG API is needed.

**DC-009-2: Option A — `rand` as an optional feature gate.**

```toml
[features]
rand = ["dep:rand_core"]

[dependencies]
rand_core = { version = "0.9", optional = true }
```

A blanket impl bridges the ecosystems when the feature is enabled:

```rust
#[cfg(feature = "rand")]
impl<T: rand_core::RngCore> Rng for T {
    fn next_f64(&mut self) -> f64 {
        // Standard conversion: u64 → [0, 1) with 53-bit mantissa.
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
}
```

This follows the established pattern used by `uuid`, `petgraph`, and
other ecosystem crates that interoperate with `rand` without mandating
it.

### Consequences

- Zero new required dependencies. The `Rng` trait is 4 lines.
- The `rand` feature is default-on (§API M-7.3). Users get
  `RngCore` interop automatically and can pass any `RngCore`
  implementor directly.
- Users who disable default features can implement the 1-method
  trait on any custom RNG (e.g. `fastrand`, `wyrand`, a
  deterministic seed).
- The `WeightedSampler` trait bound becomes
  `fn sample(&self, rng: &mut impl Rng) -> Option<Cell<...>>` using
  the crate-local `Rng`, not `rand::Rng`.

---

## Addendum 2: Trait Surface Revision (Phase 5, ADR-M-026)

**Status:** Decided  
**Date:** 2026-02-26  
**Phase:** 5

### Context

ADR-M-026 establishes the **plateau** as the semantic unit of the
G-Tree's output and the `BTreeMap<C, Plateau<C, V>>` projection
(with thatched energy attribution) as the primary analytical
interface. This obsoletes five methods from the original
`SpatialRead` surface:

| Original method | Why obsolete                                                                                                                                                                            |
| --------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `cells()`       | The plateau is the semantic unit, not the cell. The BTreeMap's contour DFS _is_ the cell traversal — it groups cells into plateaus. Exposing raw cells encourages sub-plateau thinking. |
| `range()`       | Absorbed by `map.range(a..b)` on the materialized BTreeMap. Thatched plateaus in the range are the correct answer.                                                                      |
| `len()`         | Plateau count = `map.len()`. Arena count = `node_count()` (already exists). No method on `GvGraph` needed.                                                                              |
| `is_empty()`    | A live GvGraph always has at least the root. Always false.                                                                                                                              |
| `values()`      | `map.values().map(\|p\| p.sum)` — one-liner on the BTreeMap.                                                                                                                            |

And adds one new method:

| New method   | Role                                                                                                                                                                                       |
| ------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `plateaus()` | Returns `&BTreeMap<BasisEdge<C>, Plateau<C, V>>` (or `Cow::Owned` without DCT). Live mirror of the G-Tree contour, maintained incrementally. $O(1)$ borrow. Primary analytical projection. |

### Revised `SpatialRead` surface

**Decided (DC-025-1 / DC-009-3): Option C — thin 2-method trait.**

```rust
pub trait SpatialRead {
    type Coord: Coordinate;
    type Accum: Accumulator;

    /// Plateau projection of the G-Tree contour.
    ///
    /// Returns a borrow of the live mirror (ADR-M-026). Keys are
    /// `BasisEdge<C>` and tile the domain (one per contour step
    /// position). Values carry thatched ranges — the backing node's
    /// true spatial footprint, which overlaps adjacent plateaus at
    /// semi-internal boundaries.
    ///
    /// With `dynamic-contour-tracking`: `Cow::Borrowed` — O(1).
    /// Without: `Cow::Owned` — O(G) rebuild.
    fn plateaus(&self) -> Cow<'_, BTreeMap<BasisEdge<Self::Coord>, Plateau<Self::Coord, Self::Accum>>>;

    /// Point query via route_to_receiver. O(depth), infallible.
    ///
    /// Returns the terminal or semi-internal receiver as a Cell
    /// with the *trimmed* half-interval (cell.start <= coord < cell.end).
    /// Out-of-domain coordinates are clamped (ADR-M-026 DC-026-4).
    /// NaN panics (float types).
    fn get(&self, coord: Self::Coord) -> Cell<Self::Coord, Self::Accum>;
}
```

Two methods. Object-safe, trivially mockable, captures exactly the
semantic boundary: "I am a readable spatial structure."

Five methods from the earlier 7-method revision are excluded:

| Excluded method  | Reason                                                                                                                                                                                |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `range_sum`      | Generic `R: RangeBounds` parameter breaks object safety. Stays as a concrete method on `GvGraph`; the ergonomic `RangeBounds` wrapper can't be cleanly split across trait + concrete. |
| `layers`         | Lifetime-bearing custom iterator type (`Layers<'a, C, V>`). Requires either a named associated type or boxing for `dyn` dispatch. Implementation-specific traversal.                  |
| `extract`        | Allocating $O(n)$ snapshot. V-Tree-specific — not every spatial structure would have significance-ordered layers.                                                                     |
| `terminal_count` | Arena-level metric. Leaks `GvGraph`'s resource model into the trait contract.                                                                                                         |
| `budget`         | Configuration accessor. Implementation-specific.                                                                                                                                      |

The BTreeMap returned by `plateaus()` already provides `range()`,
`values()`, `len()`, `iter()`, and `contains_key()` — all via
standard `BTreeMap` methods. `get()` provides the $O(\text{depth})$
point query that can't be derived from the BTreeMap.

### Cascade to `SpatialWrite`, `TemporalDecay`, and `WeightedSampler`

`SpatialWrite`, `TemporalDecay`, and `WeightedSampler` all require
`SpatialRead`. With DC-025-1 decided (thin 2-method trait), the
cascade is straightforward:

- `SpatialWrite` keeps `observe`. `decay` has moved to
  `TemporalDecay` (Addendum 3). Only the super-trait shrinks — from
  7 methods to 2.
- `TemporalDecay` requires `SpatialRead` + `Self::Accum: Attenuatable`.
- `WeightedSampler` requires `SpatialRead` + `Self::Accum: Weighable`.
- The full read surface (`range_sum`, `extract`, `layers`,
  `terminal_count`, `budget`) remains on `GvGraph` as concrete
  methods, accessible to any code holding a `GvGraph` reference
  but not part of the trait hierarchy.

### Consequences

- `SpatialRead` shrinks from 7 methods to 2: `plateaus` + `get`.
- Object-safe — `&dyn SpatialRead` works without boxing.
- No custom iterator types for `Cells`, `Values`, or `RangeIter`.
- `Layers<'a, C, V>` is the sole custom iterator struct
  (concrete method, not on the trait).
- The BTreeMap absorbs five methods' worth of functionality
  into standard library API surface.
- Five methods (`range_sum`, `extract`, `layers`,
  `terminal_count`, `budget`) stay as concrete methods on
  `GvGraph` — not on the trait.

---

## Addendum 3: Accumulator Sub-Traits and Capability Gating

**Status:** Decided  
**Date:** 2026-03-04  
**Relates to:** [ADR-M-033](033-v-generic-importance-properties.md) (V generic
bounds), [ADR-M-010](010-observation-generics.md) (observation generics),
[ADR-M-024](024-decay-semantics.md) (decay semantics)

### Context

The original `Accumulator` trait bundled six methods — including
`to_f64`, `from_f64`, and `prorate` — into a single trait bound on
`V`. This meant every `V` had to provide f64 conversion even if the
user only needed observe + structural operations. Analysis of every
`to_f64`/`from_f64` call site revealed four distinct semantic
operations, each with different user-customisation needs:

| Operation           | Call sites                                       | Semantic meaning                          |
| ------------------- | ------------------------------------------------ | ----------------------------------------- |
| Temporal decay      | `decay.rs`                                       | "scale this value by a decay factor"      |
| Proportional weight | `graph_query.rs` (sample), `pewei.rs`            | "project to f64 for probability weight"   |
| Spatial pro-rating  | `graph_query.rs` (range_sum)                     | "what fraction belongs to a sub-interval" |
| Debug/diagnostic    | `invariants.rs`, `diagnostic.rs`, `rebalance.rs` | "approximate f64 for human display"       |

These operations should not be assumed semantically identical for a
user's type. A saturating fixed-point type might attenuate by
snapping to the nearest representable level, while its weight
projection uses a different formula. A quantized histogram bin
might pro-rate by redistributing counts but attenuate by decaying
a confidence parameter.

### Decision (DC-009-4): Four sub-traits of `Accumulator`

Core `Accumulator` is stripped to the minimal bound needed by
`GvGraph`'s structural operations (observe, split, rebalance,
G-sum propagation, violation detection):

```rust
pub trait Accumulator: Copy + PartialOrd + Debug + Default + Send + Sync + 'static {
    fn zero() -> Self;
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
}
```

Implementations must satisfy five axioms — see
[ADR-M-006 §V](006-generic-parameters.md#v--accumulator) for the full
algebraic contract (ordered commutative monoid), including the three
hard implementation requirements (commutativity, minimum,
compatibility) and two user-side semantic contracts (identity,
associativity).

Four independent sub-traits extend it for optional capabilities:

```rust
/// Temporal decay: how this type responds to multiplicative attenuation.
pub trait Attenuatable: Accumulator {
    fn attenuate(self, factor: f64) -> Self;
}

/// Weight projection: f64 value for proportional sampling / PEWEI ratios.
pub trait Weighable: Accumulator {
    fn weight(self) -> f64;
}

/// Spatial pro-rating: fractional subdivision for range queries.
pub trait Proratable: Accumulator {
    fn prorate(self, portion: u64, total: u64) -> Self;
    fn scale_by(self, ratio: f64) -> Self;
}

/// Debug projection: approximate f64 for diagnostics and invariant checks.
pub trait Inspectable: Accumulator {
    fn to_f64_approx(self) -> f64;
}
```

### Trait-to-capability mapping

| Sub-trait      | Required by                            | Gated behind                                          |
| -------------- | -------------------------------------- | ----------------------------------------------------- |
| `Attenuatable` | `decay()`                              | `TemporalDecay` capability trait                      |
| `Weighable`    | `sample()`, PEWEI extraction           | `WeightedSampler` capability trait, `extract()`       |
| `Proratable`   | `range_sum()`                          | `range_sum()` impl block                              |
| `Inspectable`  | invariant checking, diagnostic display | `#[cfg(debug_assertions)]` paths, `invariants` module |

### Trait hierarchy

```
              Accumulator
           (zero, add, sub)
          /    |    \       \
 Attenuatable  |  Proratable Inspectable
   (decay)     |   (range)    (debug)
           Weighable
        (sample, pewei)
```

Four independent leaves. No sub-trait depends on another sub-trait.
A minimal `V` implements only `Accumulator` and gets observe +
structural ops. Each additional capability is opt-in.

### Blanket implementations

All four sub-traits have blanket implementations for the built-in
types (`u8`–`u128`, `f32`, `f64`):

- `Attenuatable`: `Self::from_f64(self.to_f64() * factor)` (using
  the type's own conversion, identical to today's decay path).
- `Weighable`: `self as f64` (direct cast).
- `Proratable`: `prorate` via `u128` intermediate for integers,
  native arithmetic for floats. `scale_by` via f64 round-trip.
- `Inspectable`: `self as f64` (same as `Weighable` for built-ins,
  but semantically distinct — one is for correctness, the other
  for humans).

Users implementing custom types choose which sub-traits to
implement based on which capabilities they need.

### Impact on `Observation<V>` blanket impl

The blanket `impl<V: Accumulator> Observation<V> for V` previously
used `V::from_f64(current.to_f64() * factor.to_f64())` for `scale`.
With `to_f64`/`from_f64` removed from `Accumulator`:

- `accumulate` still delegates to `Accumulator::add` — no change.
- `scale` now requires `V: Attenuatable` on the blanket impl, or
  the blanket `scale` impl is removed. Since ADR-M-024 established
  that `decay()` bypasses `Observation::scale` entirely, and the
  method is rarely exercised for unsigned accumulators, the blanket
  `scale` is gated behind `V: Attenuatable`.

### Impact on `SpatialWrite`

`decay()` moves from `SpatialWrite` to the new `TemporalDecay`
capability trait. `SpatialWrite` now contains only `observe()` and
requires only `V: Accumulator`.

### Impact on `Coordinate`

`Coordinate::to_f64` (used by `gtree.rs:58` for depth calculation)
is unaffected — it is on `Coordinate`, not `Accumulator`, and depth
computation is structural. This remains a required method on
`Coordinate`.

### Consequences

- **Minimal core bound.** `GvGraph<C, V, N>` requires only
  `V: Accumulator` — three methods, no f64.
- **User freedom.** A custom type can implement `Accumulator` alone
  and use observe + structural operations. Decay, sampling, range
  queries, and diagnostics each require their own opt-in sub-trait.
- **No semantic conflation.** `attenuate`, `weight`, `scale_by`, and
  `to_f64_approx` are distinct operations — a user defines each
  independently for their type.
- **Backward compatible for built-in types.** All seven built-in
  types implement all four sub-traits via blanket impls. Existing
  code using `u64`, `f64`, etc. sees no difference.
- **Compile-time capability gating.** If `V` doesn't impl
  `Attenuatable`, calling `decay()` is a compile error — not a
  runtime surprise.
