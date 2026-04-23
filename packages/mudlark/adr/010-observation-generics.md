# ADR-M-010: Observation Type Generics

**Status:** Decided  
**Date:** 2026-02-24  
**Updated:** 2026-03-04  
**Relates to:** [ADR-M-006](006-generic-parameters.md) (generic parameters),
[ADR-M-009](009-trait-decomposition.md) (`SpatialWrite::observe` signature,
accumulator sub-traits),
[ADR-M-011](011-overflow-narrowing.md) (overflow & narrowing semantics),
[ADR-M-024](024-decay-semantics.md) (decay semantics),
[ADR-M-033](033-v-generic-importance-properties.md) (V generic bounds)  
**Spec:** §IDEA M-8 (observation flow)  
**API:** [§API M-5.3](../docs/api.md#observationv) (`Observation` trait),
[§API M-5.2](../docs/api.md#observation-mutation) (`observe` signature)  
**Surface:** 2 (Film)

## Context

The spec's observation flow (§IDEA M-8) uses an untyped intensity
increment $\Delta$:

```
receiver.own ← receiver.own + Δ
```

At the Rust level the stored accumulator is `V: Accumulator`. The
question is whether the external value must be the same type as `V`,
or whether cross-type arithmetic should be supported — e.g. passing
an `f32` delta to a tree storing `u16` counts.

## Decision

`observe()` is generic over `O: Observation<V>`, decoupling
arithmetic precision from storage type.

```rust
pub trait Observation<V: Accumulator>: Copy + Debug + Send + Sync {
    /// Additive update: current + delta, in Self's precision, stored as V.
    fn accumulate(current: V, delta: Self) -> V;

    /// Multiplicative scaling: current × factor, in Self's precision, stored as V.
    fn scale(current: V, factor: Self) -> V;
}
```

`Send + Sync` are required for thread-safe use of `GvGraph` (ADR-M-007).

### Blanket impl (same-type)

Every `V: Accumulator` is automatically `Observation<V>`:

- `accumulate` delegates to `Accumulator::add`.
- `scale` requires `V: Attenuatable` (ADR-M-009 Addendum 3). When
  the bound is satisfied, the blanket impl delegates to
  `V::attenuate(current, factor.to_f64())`. When `V` does not impl
  `Attenuatable`, the blanket `scale` is unavailable — users must
  provide an explicit `Observation<V>` impl if they need
  multiplicative semantics.

Same-type usage requires zero ceremony — no wrapping or conversion.
`f64 → f64` and `f32 → f32` are handled by this blanket impl.

### Cross-type implementations

| `O` → `V`        | `accumulate`                      | `scale`                            |
| ---------------- | --------------------------------- | ---------------------------------- |
| `f64 → u8..u128` | `(current as f64 + delta) as V`   | `(current as f64 * factor) as V`   |
| `f64 → f32`      | `(current as f64 + delta) as f32` | `(current as f64 * factor) as f32` |
| `f32 → u8..u32`  | `(current as f32 + delta) as V`   | `(current as f32 * factor) as V`   |

`f32` cross-type impls stop at `u32` because `f32` has only 24 bits
of mantissa — insufficient to represent `u64`/`u128` values without
loss. `f64` covers the full integer range.

All cross-type impls use Rust's `as` cast: truncation toward zero,
saturating at type bounds (see ADR-M-011 for the full narrowing
policy).

### `scale` and decay

`scale` is defined on the trait but **`decay()` does not use it**.
ADR-M-024 established that decay uses concrete `f64` parameters and
scales values via `V::attenuate(factor)` — the `Attenuatable`
sub-trait's method (ADR-M-009 Addendum 3). This is semantically the
same operation as `scale` for built-in types, but decay bypasses
`Observation<V>` entirely because its parameters (`attenuation`,
`q`) are inherently `f64` quantities, not generic observation types.

Importantly, `attenuate` and `scale` should **not** be assumed
semantic identities for user-defined types. `attenuate` means
"this value fades over time"; `scale` means "multiplicatively
transform during observation processing." A user type may implement
them differently (e.g. different rounding, saturation, or
quantisation behaviour).

`scale` remains on the trait for:

1. The plateau sum maintenance path
   (`plateau_after_observe`), which converts `O → V` via
   `O::accumulate(V::zero(), delta)` to delta-propagate sums.
2. User-defined observation types that need custom multiplicative
   semantics (e.g. saturating or rounding scaling).

## Examples

```rust
// Given GvGraph<u64, u16, 32>:
tree.observe(42_u64, 1.0_f32);    // f32 arithmetic, stored as u16
tree.observe(42_u64, 5_u16);      // u16 arithmetic (same-type, blanket impl)
tree.observe(42_u64, 3.7_f64);    // f64 arithmetic, truncated to u16

// decay() uses V::attenuate() via the Attenuatable sub-trait,
// not Observation<V> (see ADR-M-024, ADR-M-009 Addendum 3):
tree.decay(tree.g_root(), 0.9, 0.0);  // uniform decay at 0.9×
```

## Rationale

- Storage stays compact (`u16`) while the API boundary works at
  higher precision (`f32`, `f64`).
- The blanket impl means same-type usage has zero ceremony.
- Custom observation types can implement rounding, clamping, or
  saturating semantics (see ADR-M-011).
- The `O: Observation<V>` bound makes `SpatialWrite` non-object-safe
  (ADR-M-009 §Object safety) — this is by design, since the trait is
  for static dispatch and capability bounding, not trait objects.

## Consequences

- `observe` accepts any `O` implementing `Observation<V>`.
  `from_observations` and `Extend<(C, O)>` propagate the same bound.
- `decay` uses `V::attenuate()` (the `Attenuatable` sub-trait) for
  multiplicative scaling — it is not generic over `O` (see
  [ADR-M-024](024-decay-semantics.md),
  [ADR-M-009](009-trait-decomposition.md) Addendum 3).
- `attenuate` and `scale` are not assumed to be semantic identities.
  User types may implement them with different behaviour.
- Same-type calls work without explicit wrapping.
- Cross-type narrowing uses truncation by default (see ADR-M-011).
  Users who need different policies implement the trait on newtype
  wrappers.
