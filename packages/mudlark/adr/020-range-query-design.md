# ADR-M-020: Range Query Design

**Status:** Decided  
**Date:** 2026-02-26  
**Phase:** 4 (Output)  
**Relates to:** [ADR-M-006](006-generic-parameters.md) (generic
parameters), [ADR-M-025](025-public-api-surface.md) (public API surface),
[ADR-M-026](026-point-query-and-plateau-semantics.md) (plateau semantics)  
**Spec:** §IDEA M-5.5 (Queries)  
**API:** [§API M-5.2](../docs/api.md#range-sum) (range sum)  
**Surface:** 2 (Film)

## Context

Range sum queries (§IDEA M-5.5) perform a segment-tree decomposition
over the G-Tree. Pro-rates `g.own` at partial overlaps under a
uniform-within-cell assumption. Cost: $O(N)$.

The Rust API needs to decide how range bounds are expressed and how
edge cases with float coordinates are handled.

## Questions

### Q1: Range bounds type (DC-020-1)

How should the range be specified?

| Option | Signature                                               | Notes                                                                                    |
| ------ | ------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| A      | `fn range_sum<R: RangeBounds<C>>(&self, range: R) -> V` | Idiomatic Rust. Supports `a..b`, `a..=b`, `..b`, `a..`, `..`. Matches `BTreeMap::range`. |
| B      | `fn range_sum(&self, lo: C, hi: C) -> V`                | Simple. No generics. Always half-open `[lo, hi)`.                                        |

**Considerations:**

- Option A follows `std::collections` convention and is more flexible
  (unbounded ranges, inclusive end).
- Option A requires converting `RangeBounds` to concrete `(lo, hi)`
  values internally. For `Unbounded`, use `C::zero()` and
  `C::domain_max(N)`.
- `RangeBounds<C>` requires `C: PartialOrd` (already a `Coordinate`
  supertrait), but _not_ `Ord`. This is fine for `PartialOrd`-only
  types like `f64` since the concrete comparisons in the
  implementation use `partial_cmp` anyway.

### Q2: Float coordinate edge cases (DC-020-2)

When `C` is `f32` or `f64`, range inputs can include NaN, infinity,
or negative values. How should these be handled?

| Option | Behaviour                                                                            | Rationale                                                                                                         |
| ------ | ------------------------------------------------------------------------------------ | ----------------------------------------------------------------------------------------------------------------- |
| A      | Panic on NaN inputs; treat ±∞ as domain bounds                                       | Strict, follows the "programmer error → panic" convention (§API M-7.1). Infinities have a natural interpretation. |
| B      | Return `V::zero()` for degenerate ranges (NaN, empty)                                | Lenient, never panics, but may hide bugs.                                                                         |
| C      | Require `Coordinate: Ord` for `range_sum`                                            | Excludes floats entirely from `range_sum`. Too restrictive — floats are a first-class coordinate type.            |
| D      | Accept but clamp: NaN → panic, negative → clamp to 0, overflow → clamp to domain max | Defensive with clear error for truly invalid input.                                                               |

**Considerations:**

- The existing `Coordinate` trait uses `PartialOrd`, not `Ord`, to
  support floats. The routing logic already uses `x < mid` comparisons
  which are well-defined for non-NaN floats.
- NaN is unambiguously a programmer error (no meaningful range
  contains NaN). Panicking is appropriate.
- Values outside `[0, 2^N)` could be clamped to domain boundaries or
  treated as errors. Clamping is more forgiving for callers doing
  arithmetic near boundaries.

## Decision

**DC-020-1: Option A — `RangeBounds<C>`.**

```rust
fn range_sum<R: RangeBounds<C>>(&self, range: R) -> V;
```

Follows the `std::collections::BTreeMap::range` convention. Supports
`a..b`, `a..=b`, `..b`, `a..`, and `..` syntaxes.

Internal bound resolution converts to half-open `[lo, hi)` form:

| Bound               | Resolution                                                           |
| ------------------- | -------------------------------------------------------------------- |
| `Unbounded` start   | `C::zero()`                                                          |
| `Unbounded` end     | `C::domain_max(N)`                                                   |
| `Included(x)` start | `x`                                                                  |
| `Excluded(x)` start | `x.next_value()` (integers: `x + 1`; floats: **panics** — see below) |
| `Included(x)` end   | `x.next_value()` (integers: `x + 1`; floats: **panics** — see below) |
| `Excluded(x)` end   | `x` (already half-open)                                              |

**Float coordinate restriction:** `Coordinate::next_value()` panics
for `f32`/`f64` because there is no single "next" float that
preserves the caller's intent across all scales. For float
coordinates, only bound forms that do not invoke `next_value` are
supported:

| Syntax  | Start bound | End bound   | Float-safe?     |
| ------- | ----------- | ----------- | --------------- |
| `a..b`  | `Included`  | `Excluded`  | **Yes**         |
| `a..`   | `Included`  | `Unbounded` | **Yes**         |
| `..b`   | `Unbounded` | `Excluded`  | **Yes**         |
| `..`    | `Unbounded` | `Unbounded` | **Yes**         |
| `a..=b` | `Included`  | `Included`  | **No** — panics |

This matches the `Coordinate::next_value()` contract: float
implementations panic, so callers must use bound forms that avoid
`next_value` — the natural half-open `a..b` syntax.

**DC-020-2: Option D — NaN panics, out-of-range clamps.**

| Input                                   | Behaviour                                                    |
| --------------------------------------- | ------------------------------------------------------------ |
| NaN (either bound)                      | **Panic.** Programmer error, same convention as §API M-7.1. |
| Negative / below `C::zero()`            | Clamp to `C::zero()`.                                        |
| Above `C::domain_max(N)`                | Clamp to `C::domain_max(N)`.                                 |
| ±∞                                      | Clamp to domain bounds (falls out of the above rules).       |
| Empty range after clamping (`lo >= hi`) | Return `V::zero()`.                                          |

Rationale:

- Reads and writes have different strictness requirements. `observe()`
  targets a specific cell — out-of-domain is meaningful to reject.
  `range_sum()` integrates over a region — the domain exterior
  contributes zero energy by definition, so clamping is semantically
  exact.
- Both options agree that `±∞ → domain_max` is correct. Clamping
  finite out-of-range values is the same principle at smaller scale.
- Matches `BTreeMap::range` spirit: querying beyond stored keys
  returns what's there, not a panic.
- Frees callers from manual `max(0, center - radius)` clamping near
  domain edges.

**Pro-rate arithmetic:**

The implementation computes partial-overlap pro-rating uniformly via
f64 ratio arithmetic for all coordinate types:

```rust
let node_width = C::width(node_lo, node_hi).to_f64();
let overlap_width = C::width(overlap_lo, overlap_hi).to_f64();
let own_prorated = V::from_f64(g.own.to_f64() * (overlap_width / node_width));
```

This single code path handles both integer and float coordinates
without branching. For integer accumulators, `V::from_f64` truncates
toward zero (via `as` cast), matching the truncation semantics of
`Accumulator::prorate`. The `prorate` trait method exists for other
contexts but is not used by `range_sum`.

## Consequences

- `range_sum` accepts all `RangeBounds<C>` types, matching Rust
  convention and enabling expressive range syntax.
- For float coordinates, only half-open and unbounded forms are
  supported (`a..b`, `a..`, `..b`, `..`). Inclusive-end `a..=b`
  panics because `Coordinate::next_value()` is undefined for floats.
- NaN inputs panic immediately with a clear message.
- Out-of-domain bounds are silently clamped — callers need not
  pre-clamp coordinate arithmetic near boundaries.
- Empty ranges (after clamping) return `V::zero()` — no special case.
- Pro-rating uses f64 ratio arithmetic for all coordinate types.
  Integer accumulators receive truncation-toward-zero semantics via
  `V::from_f64`.
