# ADR-M-011: Overflow and Narrowing Semantics

**Status:** Decided  
**Date:** 2026-02-24  
**Relates to:** [ADR-M-010](010-observation-generics.md) (Observation
generics — cross-type table), [ADR-M-012](012-g-sum-recomputation.md)
(G-sum recomputation — no subtraction on hot-path),
[ADR-M-023](023-pewei-reconstruction.md) (PEWEI reconstruction — added
`sub` to `Accumulator`),
[ADR-M-024](024-decay-semantics.md) (decay — uses `to_f64`/`from_f64`
directly, not `Observation::scale`)  
**Spec:** §IDEA M-5.3 (accounting — G-I1), §IDEA M-8 (observation flow),
§PEWEI M-10 (reconstruction)  
**API:** [§API M-5.3](../docs/api.md#accumulator) (Accumulator),
[§API M-5.3](../docs/api.md#observationv) (Observation)  
**Surface:** 2 (Film)

## Context

1. **Integer overflow in `Accumulator::{add, sub}`:** What happens
   when `u64 + u64` exceeds `u64::MAX`, or `u64 - u64` underflows?
2. **Cross-type narrowing in `Observation`:** What happens when
   `f64 → u16` produces a value outside `u16`'s range?
3. **Lossy conversion in `Accumulator::from_f64`:** What happens when
   `f64 → u16` produces a fractional or out-of-range value?

## Decision

**The trait defines the operation; the type defines the policy.**

### Overflow

Provided `Accumulator` impls for integer types use Rust's default
`+` and `-` operators: **panic in debug builds, wrap in release
builds.** This is what every `+` / `-` does in Rust — no surprises.

### Narrowing

Provided `Observation` cross-type impls use Rust's `as` cast:
**truncation.** This is what `as` does in Rust — zero-cost, predictable.

Provided `Accumulator::from_f64` impls likewise use `as`: truncation
toward zero, saturating at type bounds.

### Customization

Users who need different semantics implement the traits on newtype
wrappers. The tree's internal machinery calls trait methods —
`Accumulator::{add, sub, to_f64, from_f64, prorate}` and
`Observation::{accumulate, scale}` — but is agnostic to the
overflow, underflow, and narrowing policy each method embodies.

| Method       | Used by                                                                                                                                                                            | Policy concern                     |
| ------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------- |
| `add`        | Blanket `Observation<V>` (same-type), G-sum recomputation (ADR-M-012), V-sum propagation, eviction absorption (ADR-M-014), PEWEI reconstruction (ADR-M-023), range-sum combination | Overflow                           |
| `sub`        | PEWEI extraction (`Transition.refinement`), `Node::refinement()`, PEWEI reconstruction remainder (ADR-M-023)                                                                       | Underflow                          |
| `to_f64`     | Sampling weights, V-I3 comparison, range-sum pro-ration, decay (ADR-M-024), `Transition::snr()`                                                                                    | Precision loss                     |
| `from_f64`   | Blanket `Observation::scale`, range-sum pro-ration, decay (ADR-M-024)                                                                                                              | Narrowing / truncation             |
| `prorate`    | PEWEI reconstruction baseline pro-ration (ADR-M-023)                                                                                                                               | Intermediate overflow / truncation |
| `accumulate` | Observation hot-path (§IDEA M-8)                                                                                                                                                   | Cross-type narrowing               |
| `scale`      | Contract method for custom `Observation` impls; not called internally — `decay()` uses `to_f64`/`from_f64` directly (ADR-M-024)                                                    | Cross-type narrowing               |

## Examples

### Saturating accumulator

```rust
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default)]
pub struct Saturating<T>(pub T);

impl Accumulator for Saturating<u64> {
    fn zero() -> Self { Saturating(0) }
    fn add(self, other: Self) -> Self {
        Saturating(self.0.saturating_add(other.0))
    }
    fn sub(self, other: Self) -> Self {
        Saturating(self.0.saturating_sub(other.0))
    }
    fn to_f64(self) -> f64 { self.0 as f64 }
    fn from_f64(val: f64) -> Self { Saturating(val as u64) }
    fn prorate(self, portion: u64, total: u64) -> Self {
        if total == 0 { return Self::zero(); }
        Saturating((self.0 as u128 * portion as u128 / total as u128) as u64)
    }
}

let mut graph: GvGraph<u64, Saturating<u64>, 32> = GvGraph::new(config);
graph.observe(42, Saturating(10));
```

### Wrapping accumulator

```rust
#[derive(Clone, Copy, PartialEq, PartialOrd, Debug, Default)]
pub struct Wrapping<T>(pub T);

impl Accumulator for Wrapping<u64> {
    fn zero() -> Self { Wrapping(0) }
    fn add(self, other: Self) -> Self {
        Wrapping(self.0.wrapping_add(other.0))
    }
    fn sub(self, other: Self) -> Self {
        Wrapping(self.0.wrapping_sub(other.0))
    }
    fn to_f64(self) -> f64 { self.0 as f64 }
    fn from_f64(val: f64) -> Self { Wrapping(val as u64) }
    fn prorate(self, portion: u64, total: u64) -> Self {
        if total == 0 { return Self::zero(); }
        Wrapping((self.0 as u128 * portion as u128 / total as u128) as u64)
    }
}
```

### Rounding observation

```rust
#[derive(Clone, Copy, Debug)]
pub struct Rounded(pub f64);

impl Observation<u32> for Rounded {
    fn accumulate(current: u32, delta: Self) -> u32 {
        (current as f64 + delta.0).round() as u32
    }
    fn scale(current: u32, factor: Self) -> u32 {
        (current as f64 * factor.0).round() as u32
    }
}

graph.observe(42, Rounded(3.7));  // stores 4, not 3
```

### Clamping observation

```rust
#[derive(Clone, Copy, Debug)]
pub struct Clamped(pub f64);

impl Observation<u16> for Clamped {
    fn accumulate(current: u16, delta: Self) -> u16 {
        (current as f64 + delta.0).clamp(0.0, u16::MAX as f64) as u16
    }
    fn scale(current: u16, factor: Self) -> u16 {
        (current as f64 * factor.0).clamp(0.0, u16::MAX as f64) as u16
    }
}
```

## Consequences

- Default behavior matches Rust conventions — no surprises.
- Customization requires ~12 lines per newtype wrapper (6 methods).
- The tree internals never need to know about overflow/narrowing policy.
- Users can mix policies: `Saturating<u64>` accumulator with `Rounded`
  observations on the same tree.
- `sub` exists on `Accumulator` (added for PEWEI reconstruction,
  [ADR-M-023](023-pewei-reconstruction.md)) but is not on the observe
  hot-path — G-sum propagation uses recomputation from children
  ([ADR-M-012](012-g-sum-recomputation.md)), so custom `add` semantics
  are not undermined by a mismatched `sub`.
- `Observation::scale` is defined in the trait but not called by
  internal machinery — `decay()` uses `Accumulator::to_f64` /
  `Accumulator::from_f64` directly
  ([ADR-M-024](024-decay-semantics.md)). The method exists as a
  contract for custom observation types that need control over
  multiplicative narrowing.
