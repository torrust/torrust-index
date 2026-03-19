# ADR-M-006: Generic Parameters

**Status:** Decided  
**Date:** 2026-02-24  
**Updated:** 2026-03-04  
**Relates to:** [ADR-M-009](009-trait-decomposition.md) (trait
decomposition, accumulator sub-traits),
[ADR-M-010](010-observation-generics.md) (Observation generics),
[ADR-M-011](011-overflow-narrowing.md) (overflow & narrowing semantics),
[ADR-M-015](015-eviction-scan-design.md) (eviction scan design),
[ADR-M-017](017-dynamic-depth-control.md) (dynamic depth control),
[ADR-M-018](018-hard-budget-guarantee.md) (hard budget guarantee),
[ADR-M-024](024-decay-semantics.md) (decay semantics),
[ADR-M-025](025-public-api-surface.md) (public API surface),
[ADR-M-033](033-v-generic-importance-properties.md) (V generic bounds)  
**Spec:** §IDEA M-2 (Importance Interface — property hierarchy),
§IDEA M-3 (Domain), §IDEA M-4.1 (G-Node), §IDEA M-8 (observation flow),
§IDEA M-8.6 (accumulation configurations), §IDEA M-10.4 (violation-free catalytic
splits), §IDEA M-12.9.3 (finite-time bound)  
**API:** [§API M-5.2](../docs/api.md#52-gvgraphc-v-n--the-negative)
(GvGraph), [§API M-5.1](../docs/api.md#51-configv) (Config),
[§API M-5.3](../docs/api.md#53-chemistry-contracts) (traits)  
**Surface:** 2 (Film)

---

## Context

The primary type needs generic parameters for domain coordinates,
intensity values, and domain bit-width (§IDEA M-3: domain is
$[0, 2^N)$ with $N \geq 1$; §IDEA M-4.1: each G-node carries typed
`sum`/`own` fields). The mutation method `observe()` additionally
accepts an observation type that may differ from the stored
accumulator (§IDEA M-8: untyped $\Delta$ increment). The questions
are:

1. Which parameters are type-level generics, which are const generics?
2. What algebraic contracts must each type satisfy?
3. How does observation handle cross-type arithmetic?

## Decision

```rust
pub struct GvGraph<C: Coordinate, V: Accumulator, const N: u32> { /* ... */ }
```

Three struct-level parameters (`C`, `V`, `N`) and one method-level
generic (`O: Observation<V>` on `observe()`). Each has a distinct
role and distinct contracts.

---

### `C` — Coordinate

The spatial domain type. Lets the same tree work over integer and
floating-point domains. Every G-node stores its dyadic half-open
interval as `lo: C, hi: C`.

#### Trait surface

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

Eleven methods, grouped by purpose:

| Group | Methods | Role |
| ----- | ------- | ---- |
| **Domain construction** | `BITS`, `zero()`, `domain_max(n)` | Define the spatial extent $[0, 2^N)$ and validate `N <= C::BITS` at compile time |
| **Dyadic arithmetic** | `midpoint(a, b)`, `width(start, end)`, `is_final(...)` | Binary bisection, interval measurement, recursion termination |
| **Ordering** | `PartialOrd` (super-trait), `total_cmp(...)` | Routing comparisons (`x < mid`), `BTreeMap` key ordering via `BasisEdge<C>` |
| **Conversions** | `to_f64()`, `from_u64(v)`, `next_value()` | Depth computation, test harnesses, `RangeBounds` resolution |
| **Safety** | `is_nan()` | Panic guard for float coordinates in `get()` and `range_sum()` |

#### Two families: integers vs floats

Blanket implementations are provided for `u8`, `u16`, `u32`, `u64`,
`u128`, `f32`, and `f64`. The two families have subtly different
semantics:

| Property | Integer (`u8`–`u128`) | Float (`f32`, `f64`) |
| -------- | --------------------- | -------------------- |
| `domain_max(n)` | `1 << n` (or `MAX` when `n == BITS`) | `2.0.powi(n)` |
| `midpoint(a, b)` | `a + (b - a) / 2` (overflow-safe) | `a + (b - a) / 2.0` |
| `is_final(...)` | `end - start == 1` (unit cell) | `depth == n` (depth-gated) |
| `next_value()` | `self + 1` | **panics** — not meaningful in the dyadic context |
| `total_cmp(...)` | `Ord::cmp` (zero overhead) | IEEE 754 `total_cmp` (NaN sorts after +∞) |
| NaN | Impossible | Must be guarded (`is_nan()` checks) |
| Finality | Natural — subdivision terminates at width 1 | Artificial — `depth == n` is the recursion bound |

`total_cmp` exists separately from `PartialOrd` because `f64` does
not implement `Ord` (NaN is incomparable). The plateau `BTreeMap`
keys (`BasisEdge<C>`) delegate to `total_cmp` for a deterministic
total order over all `C` values.

#### Structural contracts

The G-Tree's correctness depends on six implicit properties of `C`.
These are not expressed as separate trait methods — they are
consequences of a correct `Coordinate` implementation:

1. **Dyadic closure** — `midpoint(a, b)` must return a value
   strictly between `a` and `b` when the interval is divisible.
   The split guard checks `mid > lo`; if this fails, the tree
   cannot subdivide.

2. **Width-depth coherence** — `width(lo, hi).to_f64().log2()` must
   yield an integer for dyadic intervals. Depth computation
   (`N − log₂(width)`) depends on this. For integer types the
   property holds exactly (power-of-two widths). For floats,
   `f64::log2` returns exact integers for $2^k$ inputs.

3. **Total ordering** — `total_cmp` must be a total order
   (reflexive, antisymmetric, transitive, total). Required by
   `BTreeMap` invariants for plateau keys.

4. **Domain spanning** — `[zero(), domain_max(N))` must cover the
   valid coordinate range. The root G-node is initialized with
   these bounds. All descendant intervals are subsets by
   construction.

5. **Midpoint determinism** — `midpoint(a, b)` must be deterministic
   and pure. The G-Tree does not store midpoints — it recomputes
   them from `(lo, hi)` on every routing step. Non-deterministic
   midpoints would cause routing to arrive at inconsistent
   locations.

6. **Non-negative domain** — the domain starts at `zero()`. The spec
   defines it as $[0, 2^N)$. Signed types (e.g. `i64`) are not
   provided — negative coordinates have no meaning in the dyadic
   framework.

#### Where `C` appears in the system

`C` flows through every spatial operation:

- **G-node intervals** — `g.lo: C`, `g.hi: C` on every arena node.
- **Observation routing** — `C::midpoint(g.lo, g.hi)` on the hot
  path (`route_to_receiver`).
- **Split** — child intervals `[lo, mid)` and `[mid, hi)`.
- **Depth computation** — `C::width(lo, hi).to_f64()` → `log₂`.
- **Point query clamping** — `C::zero()`, `C::domain_max(N)`.
- **Range query pro-rating** — `C::width()` ratios for partial
  overlaps.
- **Plateau keys** — `BasisEdge<C>` wrapping `C::total_cmp` for
  `BTreeMap` ordering.
- **View types** — `Cell<C, V>`, `Span<C, V>`, `Node<C, V>` all
  carry `start: C, end: C`.

`C` does **not** participate in the V-Tree, decay parameters,
observation arithmetic, or any `V`-space computation. The coordinate
space and value space are fully independent.

---

### `V` — Accumulator

The intensity / accumulation type. Lets the tree store exact integer
counts or floating-point intensities. `V` is the type of every
`g.own`, `g.sum`, and V-entry intensity field.

#### Core trait surface

```rust
pub trait Accumulator: Copy + PartialOrd + Debug + Default + Send + Sync + 'static {
    fn zero() -> Self;
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
}
```

Three methods. This is the **minimal** bound on `V` — no f64
conversion is required at this level. `GvGraph` requires only
`V: Accumulator` for construction, observation, split, rebalance,
G-sum propagation, and violation detection.

#### Algebraic contract

`(V, add, zero(), <=)` must form an **ordered commutative monoid**.
Concretely, every implementation must satisfy five axioms:

| Axiom | Statement | Used by |
| ----- | --------- | ------- |
| **Identity** | `add(a, zero()) == a` | G-sum uses `zero()` for missing children |
| **Associativity** | `add(a, add(b, c)) == add(add(a, b), c)` | Nested sum recomputation |
| **Commutativity** | `add(a, b) == add(b, a)` | Sibling merge order is arbitrary |
| **Minimum** | `zero() <= v` for all `v` | Ghost fast path, violation-free splits |
| **Compatibility** | `a <= b` implies `add(a, c) <= add(b, c)` for all `c >= zero()` | V-Tree tournament coherence |

All five axioms are **jointly required** for a fully coherent
system, but they split into two categories: three are **hard
implementation requirements** (the implementation produces
incorrect internal state without them), and two are **user-side
semantic contracts** (the implementation works mechanically, but
the user's "sum" loses its expected meaning).

##### Hard implementation requirements

**Commutativity** is a hard requirement because rebalance merges
sibling V-nodes with `add(a_int, b_int)` where the order of `a` and
`b` depends on rotation direction — an internal implementation
detail the user cannot control or predict. If
`add(a, b) != add(b, a)`, the same logical merge produces different
values depending on whether the implementation chooses a left or
right rotation.

**Minimum** is a hard requirement because catalytic splits initialise
new V-entries at `zero()` and the ghost fast path checks
`importance == zero()`. Without a guaranteed bottom element, these
constructions produce V-I3 violations (§IDEA M-10.4, §IDEA M-12.9.3).

**Compatibility** is a hard requirement because it is the unique
axiom that binds `PartialOrd` to `add`. Without it, the ordering
and the addition are independent structures that happen to coexist
on the same type. The V-Tree uses `PartialOrd` for governance
(uncle comparisons) and `add` for aggregation (structural sums).
Compatibility guarantees these are coherent: if $a$ is less
important than $b$, then $a + c$ is less important than $b + c$.
Addition preserves relative standing. This is what makes the
tournament semantics work — when the V-Tree aggregates children into
a structural node and compares that aggregate against an uncle, it
relies on the aggregate faithfully representing the competitive
strength of its contents.

##### User-side semantic contracts

**Identity** — if `add(x, zero()) != x`, sums will drift on
eviction (the implementation absorbs a child's sum into
`parent.own`, unlinks the child, then recomputes with `zero()` for
the missing slot). The implementation does not crash — it faithfully
tracks the user's non-conserving arithmetic. Energy creation or
destruction is the user's concern.

**Associativity** — the implementation always nests G-sums as
`add(own, add(left, right))` and V-structural intensities as a left
fold over children. These conventions are deterministic and
consistent across all code paths. If associativity does not hold,
the result is still well-defined — the user simply needs to
understand the nesting convention. The semantic meaning of "sum" is
the user's concern.

Note: `debug_assert!` checks in `invariants.rs` may fire if
identity or associativity are violated, because the checker
recomputes sums and compares against stored values. These are
debug-only — release builds proceed without checking.

#### The grounded-identity invariant

The critical structural invariant is:

> **For all `v: V`, `v >= V::zero()`** — i.e. `zero()` is the
> minimum of the type's non-negative range and the additive identity.

In the spec's property hierarchy (§IDEA M-2.3.1), this is encoded
by properties P2 (grounded: $\nu \preceq a$ for all $a$) and P4
(identity: $\nu \oplus a = a$). Without P2, catalytic splits may
create V-I3 violations and sampling probabilities are undefined.
Without P4, the ghost fast path is unavailable.

`GvGraph` requires the full [Standard property profile](../docs/idea.md#271-standard-non-negative-identity)
(§IDEA M-2.7.1, P0–P5) exclusively. The invariant is **not** expressed as a separate
`HasMinimum` trait or a feature flag. Instead it is encoded
structurally: blanket implementations are provided only for types
where `zero()` is provably the minimum (`u8`–`u128`, `f32`, `f64`),
and the `Accumulator` doc-contract specifies that any user-supplied
implementation must uphold `zero() == minimum`. See
[ADR-M-033](033-v-generic-importance-properties.md) for the full
property analysis and exclusion rationale.

#### What `V: Accumulator` guarantees

Because `V` is generic only over Standard-compatible types (P0–P5),
every `GvGraph<C, V, N>` unconditionally has:

1. **Violation-free splits.** Catalytic splits and `vtree_insert`
   never produce V-I3 violations — `zero()` is the bottom element
   by construction (§IDEA M-10.4).

2. **Ghost fast path** [P4]. A V-node with `importance == V::zero()`
   is a ghost; eviction absorption is a no-op ($\nu \oplus a = a$),
   skipping Steps 3–4 of §IDEA M-12.5. Per-ghost eviction cost is $O(1)$
   instead of $O(h_V)$ (§IDEA M-12.9.3(B)).

3. **Observe + structural operations.** `observe()`, split,
   rebalance, G-sum propagation, and violation detection all require
   only `Accumulator` — no f64 conversion.

#### Sub-trait hierarchy

Four independent sub-traits extend `Accumulator` for optional
capabilities. See [ADR-M-009](009-trait-decomposition.md) Addendum 3
for the full design rationale, and
[ADR-M-033](033-v-generic-importance-properties.md) for the blanket
implementation details.

```
              Accumulator
           (zero, add, sub)
          /    |    \       \
 Attenuatable  |  Proratable Inspectable
   (decay)     |   (range)    (debug)
           Weighable
        (sample, pewei)
```

| Sub-trait | Methods | Required by | Gated behind |
| --------- | ------- | ----------- | ------------ |
| `Attenuatable` | `attenuate(self, factor: f64) -> Self` | `decay()` | `TemporalDecay` capability trait |
| `Weighable` | `weight(self) -> f64` | `sample()`, PEWEI extraction | `WeightedSampler` capability trait |
| `Proratable` | `prorate(self, portion, total) -> Self`, `scale_by(self, ratio) -> Self` | `range_sum()` | Concrete method on `GvGraph` |
| `Inspectable` | `to_f64_approx(self) -> f64` | Invariant checking, diagnostics | `#[cfg(debug_assertions)]` paths |

Four independent leaves — no sub-trait depends on another sub-trait.
A minimal `V` implements only `Accumulator` and gets observe +
structural ops. Each additional capability is opt-in.

#### Provided implementations

All traits have blanket implementations for the built-in types
(`u8`–`u128`, `f32`, `f64`):

```rust
// Core Accumulator: u8–u128, f32, f64
impl Accumulator for u64 { .. }  // zero() = 0, add = +, sub = -
impl Accumulator for f64 { .. }  // zero() = 0.0, add = +, sub = -

// Sub-traits: all seven built-in types implement all four.
impl Attenuatable for u64 { fn attenuate(self, f: f64) -> Self { (self as f64 * f) as Self } }
impl Weighable    for u64 { fn weight(self) -> f64 { self as f64 } }
impl Proratable   for u64 { .. }  // u128 intermediate arithmetic
impl Inspectable  for u64 { fn to_f64_approx(self) -> f64 { self as f64 } }
```

A user may implement `Accumulator` for a custom newtype (e.g. a
saturating or fixed-point type) provided it upholds the algebraic
contract and the [Standard property profile](../docs/idea.md#271-standard-non-negative-identity)
(§IDEA M-2.7.1, P0–P5). They then choose which sub-traits to
implement based on which capabilities they need.

#### What is excluded

`V` does not range over types that would require degraded property
profiles (Signed, Elevated configurations — see §IDEA M-8.6):

- **Signed importance types** (e.g. `i32`, `i64`) where `zero()` is
  not the minimum.
- **Types where `Default::default()` differs from the infimum** of
  the representable range.
- **Types without `PartialOrd`** — ordering is required for
  threshold comparison and violation detection.

No runtime property-selection, no conditional code paths, no
fallback rebalance after splits.

---

### `N` — Domain Bit-Width

The domain extent as a `const` generic `u32`. The domain is
$[0, 2^N)$ in coordinate space, with $N \geq 1$ (§IDEA M-3).

#### Compile-time validation

```rust
const { assert!(N <= C::BITS, "N must be <= C::BITS") };
```

Evaluated at monomorphization. `GvGraph<u32, u64, 48>` fails to
compile because `u32::BITS == 32 < 48`. The domain must be
representable in the coordinate type.

#### Relationship to `C`

- `N` determines the root interval: `[C::zero(), C::domain_max(N))`.
- When `N == C::BITS` (e.g. `GvGraph<u64, _, 64>`), `domain_max`
  returns `C::MAX` because $2^{64}$ would overflow `u64`.
- G-Tree depth at any node is $N - \log_2(\text{width})$, making `N`
  the maximum possible depth and the recursion bound for float
  `is_final`.
- `N` is a const generic on the struct — not a runtime `Config`
  field. This enables zero-cost bit-width arithmetic throughout.

---

### `O` — Observation Type

A method-level generic on `observe()`, not a struct-level parameter.
Decouples arithmetic precision from storage type: a tree with
`V = u16` can accept observations in `f64`, performing the math at
float precision and truncating back to `u16` for storage.

See [ADR-M-010](010-observation-generics.md) for the full design
rationale and [ADR-M-011](011-overflow-narrowing.md) for
overflow/narrowing semantics.

```rust
pub fn observe<O: Observation<V>>(&mut self, coord: C, delta: O);
```

#### Trait surface

```rust
pub trait Observation<V: Accumulator>: Copy + Debug + Send + Sync {
    fn accumulate(current: V, delta: Self) -> V;
    fn scale(current: V, factor: Self) -> V;
}
```

Two methods. `Send + Sync` are required for thread-safe use of
`GvGraph` ([ADR-M-007](007-thread-safety.md)).

**`accumulate(current, delta)`** — the additive hot path.
Called on every observation to update `g.own` and the V-entry
intensity. Also used as the `O → V` conversion bridge via
`accumulate(V::zero(), delta)` (plateau sum maintenance).

**`scale(current, factor)`** — multiplicative transform. Defined on
the trait but **not called by internal machinery**. `decay()` uses
`V::attenuate()` instead (see below). The method exists for
user-defined observation types that need custom multiplicative
semantics.

#### Blanket impl: same-type

Every `V: Accumulator` is automatically `Observation<V>`:

- `accumulate` delegates to `Accumulator::add`.
- `scale` round-trips through `f64`: `V::from_f64(current.to_f64() * factor.to_f64())`.

Same-type usage requires zero ceremony — no wrapping or conversion.

#### Cross-type implementations

| `O` → `V` | `accumulate` | `scale` |
| ---------- | ------------ | ------- |
| `f64 → u8..u128` | `(current as f64 + delta) as V` | `(current as f64 * factor) as V` |
| `f64 → f32` | `(current as f64 + delta) as f32` | `(current as f64 * factor) as f32` |
| `f32 → u8..u32` | `(current as f32 + delta) as V` | `(current as f32 * factor) as V` |

`f32` cross-type impls stop at `u32` because `f32` has only 24 bits
of mantissa — insufficient to represent `u64`/`u128` values without
loss. All cross-type impls use Rust's `as` cast: truncation toward
zero, saturating at type bounds (see ADR-M-011).

#### No formal axioms

Unlike `Accumulator`, `Observation<V>` has **no algebraic axioms**.
It is a mapping contract, not an algebraic structure:

- `accumulate` must be deterministic (same inputs → same output).
- `accumulate(V::zero(), delta)` must produce a well-defined `V` —
  this is the `O → V` conversion path used by plateau sum
  maintenance. If this produces garbage, plateau sums drift.
- `scale` is semantically independent from
  `Attenuatable::attenuate`. `attenuate` means "this value fades
  over time"; `scale` means "multiplicatively transform during
  observation processing." A user type may implement them
  differently.

#### Where `O` appears

`O` flows through five API entry points:

| Entry point | Signature |
| ----------- | --------- |
| `observe()` | `fn observe<O: Observation<V>>(&mut self, coord: C, delta: O)` |
| `SpatialWrite::observe()` | Same signature via capability trait |
| `Extend<(C, O)>` | `fn extend<I: IntoIterator<Item = (C, O)>>(&mut self, iter: I)` |
| `from_observations()` | `fn from_observations<O, I>(config, iter) -> Self` |
| `plateau_after_observe()` | Internal: `fn plateau_after_observe<O: Observation<V>>(&mut self, g_id, delta: O)` |

The `O: Observation<V>` bound on `observe()` makes `SpatialWrite`
non-object-safe — this is by design, since the trait is for static
dispatch and capability bounding, not trait objects
([ADR-M-009](009-trait-decomposition.md)).

---

### Decay parameters

`decay()` uses concrete `f64` parameters — it is **not** generic
over `O: Observation<V>`. See [ADR-M-024](024-decay-semantics.md) for
the full design rationale.

```rust
pub fn decay(&mut self, root: GNodeId, attenuation: f64, q: f64);
```

The original design made `decay` generic over `O: Observation<V>`.
ADR-M-024 superseded this: decay is a subband-adaptive temporal filter
whose parameters (`attenuation`, `q`) are inherently floating-point
quantities. Using concrete `f64` avoids forcing callers to supply a
type parameter for a fundamentally continuous operation.

`decay()` requires `V: Attenuatable` — a sub-trait that defines how
multiplicative scaling is applied to the type. This bound is enforced
on the `TemporalDecay` capability trait (split from `SpatialWrite`),
not on `GvGraph` itself. A `V` that doesn't support attenuation can
still be used for observe, query, and structural operations. See
[ADR-M-009](009-trait-decomposition.md) Addendum 3.

---

### `Config<V>`

`N` is the const generic on the type — no runtime `domain_bits`
field.

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

`alpha_relax` and `bounded_eviction` were introduced by
[ADR-M-017](017-dynamic-depth-control.md) (dynamic depth control) and
[ADR-M-018](018-hard-budget-guarantee.md) /
[ADR-M-015](015-eviction-scan-design.md) (hard budget / early-stop
eviction) respectively. See [§API M-5.1](../docs/api.md#51-configv)
for validation rules.

## Examples

```rust
// Construction — type parameters determine the domain and value space.
let tree: GvGraph<u64, u64, 32> = GvGraph::new(config);     // 32-bit coords, integer counts
let tree: GvGraph<u128, f64, 128> = GvGraph::new(config);    // u128 coords, float intensity
let tree: GvGraph<f64, f64, 52> = GvGraph::new(config);      // f64 coords, float intensity
let tree: GvGraph<u32, u32, 16> = GvGraph::new(config);      // 16-bit coords, u32 counts

// Same-type observation (blanket impl, zero ceremony):
tree.observe(42_u64, 10_u64);

// Cross-type observation (f32 arithmetic, stored as u16):
let tree: GvGraph<u64, u16, 32> = GvGraph::new(config);
tree.observe(42_u64, 1.0_f32);    // f32 precision, truncated to u16
tree.observe(42_u64, 3.7_f64);    // f64 precision, truncated to u16

// Decay uses concrete f64 — not generic over O:
tree.decay(tree.g_root(), 0.9, 0.0);  // uniform decay at 0.9×
```

## Consequences

- **Compile-time domain validation:** `const { assert!(N <= C::BITS) }`
  in `new()` — invalid combinations are rejected at monomorphization.
- **Zero-cost bit-width arithmetic:** no runtime branching on domain
  size.
- **Cross-type observation:** `GvGraph<u64, u16, 32>` can accept
  `f32` deltas via `Observation<u16> for f32`.
- **Type-safe configuration:** `Config<V>` uses `V` for the split
  threshold — no implicit conversions.
- **Iterator integration:** `Extend<(C, O)>` and `from_observations`
  propagate the `O: Observation<V>` bound through the iterator
  interface ([ADR-M-025](025-public-api-surface.md)).
- **Associated types on capability traits:** `SpatialRead`,
  `SpatialWrite`, `TemporalDecay`, and `WeightedSampler` expose `C`
  and `V` as associated types rather than repeating the generic
  parameters ([ADR-M-009](009-trait-decomposition.md)).
- **Minimal core bound:** `GvGraph<C, V, N>` requires only
  `V: Accumulator` — three methods, no f64. This is the widest
  possible accept set for user-defined types.
- **Compile-time capability gating:** If `V` doesn't impl
  `Attenuatable`, calling `decay()` doesn't compile. If `V` doesn't
  impl `Weighable`, `sample()` doesn't compile. No runtime surprise.
- **No semantic conflation:** `attenuate`, `weight`, `scale_by`, and
  `to_f64_approx` are defined independently. A user type may
  attenuate by snapping to the nearest representable level while
  projecting weight via a different formula.
- **Backward compatible:** All seven built-in types implement all
  four sub-traits. Existing code using `u64`, `f64`, etc. sees no
  difference.
- **Single code path:** Every internal algorithm assumes `V::zero()`
  is the bottom element. No branching on configuration, no degraded
  fallback paths.
- **No runtime enforcement:** The constraint `v >= V::zero()` is a
  doc-contract, not a type-level proof. A user who implements
  `Accumulator` for a signed type and violates the invariant will
  see silent corruption. A `debug_assert!(delta >= V::zero())` in
  `observe()` would catch misuse in debug builds; this is not yet
  implemented but is a natural follow-up.
- **Coordinate independence:** `C` and `V` never interact
  algebraically. The only bridge is `C::width().to_f64()` as a
  denominator in range-sum pro-rating.
- **Observation is a mapping, not an algebra:** `O: Observation<V>`
  has no formal axioms. Users who need different narrowing policies
  implement the trait on newtype wrappers (see ADR-M-011).
