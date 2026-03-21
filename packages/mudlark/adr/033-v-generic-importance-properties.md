# ADR-M-033: V Generic Bounds and the Importance Property Hierarchy

**Status:** Decided  
**Date:** 2026-03-04  
**Updated:** 2026-03-06  
**Relates to:** [ADR-M-006](006-generic-parameters.md) (generic
parameters), [ADR-M-009](009-trait-decomposition.md) (trait
decomposition, accumulator sub-traits),
[ADR-M-010](010-observation-generics.md) (observation generics),
[ADR-M-019](019-sampling-semantics.md) (sampling semantics)  
**Spec:** §IDEA M-2 (Importance Interface — property hierarchy,
feature table, collapse theorem), §IDEA M-8.6 (accumulation
configurations), §IDEA M-8.8 (capability decomposition), §IDEA M-10.4
(violation-free catalytic splits), §IDEA M-12.9.3 (finite-time bound)

---

## Context

`GvGraph<C, V, N>` is generic over three parameters
([ADR-M-006](006-generic-parameters.md)): the coordinate type `C`, the
accumulation type `V`, and the domain bit-width `N`.

The spec defines the importance interface as a value space
$\mathcal{V} = (I,\; \oplus,\; \nu,\; \preceq)$ with a base axiom
set **P0** (§IDEA M-2.2) and five optional properties P1–P5 (§IDEA M-2.3).
Features are enabled by individual properties, not by membership in
a named tier:

| Property | Statement                                                                        | Enables                                                                    |
| -------- | -------------------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| **P0**   | Closure + Commutativity + Compatibility + Totality + Well-typing                 | Governance: compare, aggregate, rebalance, promote, evict                  |
| **P1**   | Bounded below: $\exists\, \bot \in I : \forall\, x,\; \bot \preceq x$            | Proportional sampling                                                      |
| **P2**   | Grounded: $\forall\, a \in I : \nu \preceq a$ (implies P1)                       | Violation-free insertion                                                   |
| **P3**   | Idempotent ground: $\nu \oplus \nu = \nu$                                        | Violation-free structural nodes after split                                |
| **P4**   | Identity element: $\forall\, a \in I : \nu \oplus a = a$ (implies P3)            | Ghost eviction fast path, sum-propagation early termination                |
| **P5**   | Associativity: $\forall\, a,b,c : (a \oplus b) \oplus c = a \oplus (b \oplus c)$ | Propagation-free structural rearrangement; Fibonacci depth bound (with P1) |

The spec identifies four named configurations (§IDEA M-8.6.1), each with a
different property profile:

| Configuration | Properties     | Lost features                                       |
| ------------- | -------------- | --------------------------------------------------- |
| **Standard**  | P0–P5          | None                                                |
| **Absolute**  | P0–P5          | None (different projection)                         |
| **Signed**    | P0, P3, P4, P5 | Sampling, Fibonacci bound, violation-free insertion |
| **Elevated**  | P0, P1, P5     | Ghost fast path, violation-free splits              |

Under ordinary addition on $[0, \infty)$ with $\nu = 0$, the five
properties collapse into mutual equivalence — the additive collapse
theorem (§IDEA M-2.5.1). This is the Standard configuration and the common
case for all built-in numeric types.

The question is: what property profile should `V` carry in
`GvGraph<C, V, N>`, and must the implementation support all four
configurations?

## Decision

**`V` is generic over types satisfying the Standard configuration
(P0–P5).** The core `Accumulator` trait encodes the Standard
property profile, and `GvGraph` requires `V: Accumulator`:

```rust
pub trait Accumulator: Copy + PartialOrd + Debug + Default + Send + Sync + 'static {
    fn zero() -> Self;             // additive identity (P4) AND minimum (P2)
    fn add(self, other: Self) -> Self;
    fn sub(self, other: Self) -> Self;
}
```

This is the **minimal** bound on `V`. No f64 conversion is required
at this level.

### Properties encoded by `Accumulator`

The trait's doc-contract requires `V` to satisfy every property in
the Standard profile:

| Property             | How encoded                                            | Category   |
| -------------------- | ------------------------------------------------------ | ---------- |
| **P0** Closure       | Type system (`add` returns `Self`)                     | Structural |
| **P0** Commutativity | Doc-contract: `add(a, b) == add(b, a)`                 | Structural |
| **P0** Compatibility | Doc-contract: `a <= b` ⟹ `add(a, c) <= add(b, c)`      | Structural |
| **P0** Totality      | `PartialOrd` bound                                     | Structural |
| **P0** Well-typing   | `zero()` returns `Self`                                | Structural |
| **P2** Grounded      | Doc-contract: `zero() <= v` for all `v`                | Structural |
| **P4** Identity      | Doc-contract: `add(a, zero()) == a`                    | Semantic   |
| **P5** Associativity | Doc-contract: `add(a, add(b, c)) == add(add(a, b), c)` | Semantic   |

P2 implies P1 (bounded below); P4 implies P3 (idempotent ground).
Therefore, requiring P2 + P4 + P5 yields the full Standard
configuration (P0–P5). See
[ADR-M-006 §V](006-generic-parameters.md#v--accumulator) for the
algebraic contract, axiom severity classification, and per-axiom
justification.

### What `V: Accumulator` guarantees

Because `V` is generic only over Standard-compatible types, every
`GvGraph<C, V, N>` unconditionally has:

1. **Violation-free splits** [P2 + P3]. Catalytic splits and
   `vtree_insert` never create V-I3 violations — `zero()` is the
   bottom element and $\nu \oplus \nu = \nu$ (§IDEA M-10.4).

2. **Ghost fast path** [P4]. A V-node with `importance == V::zero()`
   is a ghost; eviction absorption is a no-op ($\nu \oplus a = a$),
   skipping Steps 3–4 of §IDEA M-12.5. Per-ghost eviction cost is $O(1)$
   instead of $O(h_V)$ (§IDEA M-12.9.3(B)).

3. **Fibonacci depth bound** [P1 + P5]. V-Tree height is bounded by
   $\log_\varphi n$ where $\varphi$ is the golden ratio (§IDEA M-18.1).

4. **Proportional sampling** [P1]. Non-negative ratios provide
   well-defined probability weights (§IDEA M-6.5).

5. **Propagation-free rearrangement** [P5]. Contraction and promotion
   do not require sum-propagation calls (§§IDEA M-11.3–11.5).

6. **Observe + structural operations.** `observe()`, split,
   rebalance, G-sum propagation, and violation detection require only
   `Accumulator` — no f64 conversion.

### Extended capabilities via sub-traits

Additional capabilities are provided by four independent sub-traits
([ADR-M-009](009-trait-decomposition.md) Addendum 3):

```rust
/// Temporal scaling: multiplicative modulation (§IDEA M-8.8.2, §IDEA M-14).
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
    fn from_f64(v: f64) -> Self;
}
```

| Sub-trait      | Required for                           | Capability trait                 |
| -------------- | -------------------------------------- | -------------------------------- |
| `Attenuatable` | `decay()`                              | `TemporalDecay`                  |
| `Weighable`    | `sample()`, PEWEI extraction           | `WeightedSampler`                |
| `Proratable`   | `range_sum()`                          | Concrete method on `GvGraph`     |
| `Inspectable`  | Invariant checking, diagnostic display | `#[cfg(debug_assertions)]` paths |

A `V` that implements only `Accumulator` can still be observed,
queried (point query, plateau projection), split, and rebalanced.
Decay, sampling, range queries, and diagnostics each require their
own opt-in sub-trait. These sub-traits correspond to the spec's
extended capabilities (§IDEA M-8.8.2): temporal scaling, weighted sampling,
range pro-ration, and diagnostics.

### What is excluded

The implementation does not support configurations where the Standard
property profile (P0–P5) fails:

- **Signed value spaces** ($I = \mathbb{R}$) where P1 and P2 fail.
  `zero()` is not bounded below; sampling is undefined; the Fibonacci
  depth bound does not hold; violation-free insertion is not
  guaranteed. (Spec §IDEA M-8.6.4 defines the signed configuration with
  properties {P0, P3, P4, P5}.)
- **Elevated value spaces** ($\nu > 0$) where P2, P3, P4 fail.
  Splits may create violations; the ghost fast path is unavailable;
  contraction cost degrades from $O(|G|_0 \cdot h_V)$ to
  $O(|G|_0 \cdot h_V^2)$ (§IDEA M-12.9.3). (Spec §IDEA M-8.6.5 defines the
  elevated configuration with properties {P0, P1, P5}.)
- **Types where `Default::default()` differs from the additive
  identity** — P4 is violated.
- **Types without `PartialOrd`** — P0 Totality is violated.

No runtime property-selection, no conditional code paths, no
degraded fallback paths.

### Blanket implementations

All traits have blanket implementations for the built-in types:

```rust
// Core Accumulator: u8–u128, f32, f64
impl Accumulator for u8  { .. }  // zero() = 0, add = +, sub = -
impl Accumulator for u16 { .. }
// ...
impl Accumulator for f64 { .. }  // zero() = 0.0, add = +, sub = -

// Sub-traits: all seven built-in types implement all four.
impl Attenuatable for u64 { fn attenuate(self, f: f64) -> Self { (self as f64 * f) as Self } }
impl Weighable    for u64 { fn weight(self) -> f64 { self as f64 } }
impl Proratable   for u64 { .. }  // u128 intermediate arithmetic
impl Inspectable  for u64 { fn to_f64_approx(self) -> f64 { self as f64 } }
// ... same pattern for all built-in types
```

All provided implementations satisfy P0–P5 by construction: `zero()`
is always `0` / `0.0`, which is both the additive identity and the
minimum of the unsigned / non-negative range. Under the additive
collapse theorem (§IDEA M-2.5.1), this uniquely determines the Standard
configuration $(\mathbb{R}_{\geq 0},\; +,\; 0,\; \leq)$.

A user may implement `Accumulator` for a custom newtype (e.g. a
saturating or fixed-point type) provided it upholds the full Standard
property profile. They then choose which sub-traits to implement
based on which capabilities they need.

## Consequences

- **Single code path.** Every internal algorithm assumes the Standard
  property profile (P0–P5). No branching on configuration, no
  degraded fallback paths.
- **Minimal core bound.** `GvGraph<C, V, N>` requires only
  `V: Accumulator` — three methods, no f64. This is the widest
  accept set consistent with the Standard property profile.
- **Compile-time capability gating.** If `V` doesn't impl
  `Attenuatable`, calling `decay()` doesn't compile. If `V` doesn't
  impl `Weighable`, `sample()` doesn't compile. No runtime surprise.
- **No semantic conflation.** `attenuate`, `weight`, `scale_by`, and
  `to_f64_approx` are defined independently. A user type may
  attenuate by snapping to the nearest representable level while
  projecting weight via a different formula.
- **Backward compatible.** All seven built-in types implement all
  four sub-traits. Existing code using `u64`, `f64`, etc. sees no
  difference.
- **Debug-mode enforcement.** The constraint `v >= V::zero()` (P2)
  is a doc-contract, not a type-level proof. A user who implements
  `Accumulator` for a type violating P2 will see silent corruption
  in release builds. `observe()` contains a `debug_assert!` that
  fires when the post-accumulation value drops below `V::zero()`,
  catching misuse in debug builds and `cargo test`. See
  `tests/negative_f64.rs` for integration tests that document the
  failure modes when the guard is bypassed (release builds).
- **Smaller test surface.** Only the Standard and Absolute
  configurations need testing (they share the same value space;
  different projections). No adversarial signed-importance or
  elevated-ground scenarios.
- **Future expansion.** If Signed or Elevated configuration support
  is ever needed (e.g. for signed financial P&L or biased initial
  importance), it would require either: (a) property-conditional code
  paths — the spec already specifies fallback behaviour for each
  missing property (§IDEA M-10.4: violation push on P2/P3 failure; §IDEA M-12.9.3:
  degraded contraction bound on P4 failure; §§IDEA M-11.3–11.5: explicit
  sum-propagation on P5 failure), or (b) a separate trait hierarchy
  or distinct type. The property-based decomposition in the spec
  (§IDEA M-2.3) provides the architectural roadmap; the current
  implementation optimises for the common case by requiring the full
  Standard profile.
