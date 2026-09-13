# ADR-S-003: Mudlark Integration · `rec:sentinel:mudlark-default-types-and-feature-gating`

**Status:** Decided (Part A superseded by [ADR-S-018](018-generic-domain-parameters.md)) **Date:** 2026-03-09 **Spec:** §ALGO S-2.1 (domain $[0, 2^{128})$), §ALGO S-2.4 (what sentinel owns), §ALGO S-13.3 (G-V Graph config) **Relates to:** [ADR-S-002](002-feed-forward-invariant.md) (Δ = 1 invariant), [ADR-S-004](004-config-validation-over-panic.md) (config validation), mudlark [ADR-M-006](../../mudlark/adr/006-generic-parameters.md) (generic parameters)

## Context · `sec:sentinel:mudlark-context`

The sentinel must own a `GvGraph` instance from mudlark. Two groups of decisions arise:

1. **Type-level parameterisation** — the three generic parameters `C`, `V`, `N` and the `Config<V>` fields.
2. **Cargo feature gating** — which of mudlark's optional features the sentinel enables, and how they relate to sentinel's own features.

These are a single integration surface and are decided together.

---

## Part A — Type Parameters: `GvGraph<u128, u64, 128>` · `sec:sentinel:mudlark-part-a-type-parameters`

> **Superseded by [ADR-S-018](018-generic-domain-parameters.md).** The sentinel is now generic: `SpectralSentinel<C, V, N>` with type aliases `Sentinel128` and `Sentinel64`. The rationale below is retained for historical context; the concrete parameters described here remain the defaults via `Sentinel128`.

| Parameter | Value | Rationale |
|-----------|-------|-----------|
| `C = u128` | The sentinel analyses IPv6 addresses and similar 128-bit identifiers. The full `u128` domain covers $[0, 2^{128})$ without truncation. |
| `V = u64` | The sentinel observes with Δ = 1 per input value (ADR-S-002). An unsigned integer counter is the natural accumulator. `u64` provides a ceiling of $1.8 \times 10^{19}$ observations — far beyond any practical lifetime. |
| `N = 128` | Full domain resolution. Every bit position is available for spatial refinement. |

### `split_threshold` as `u64` · `sec:sentinel:mudlark-split-threshold`

The spec lists `split_threshold` as `f64`, but since `V = u64`, mudlark's `Config<V>` requires `split_threshold: u64`. The sentinel accordingly stores `split_threshold: u64` in `SentinelConfig` with default `100`.

This is type-honest: the threshold is "number of observations a cell must accumulate before subdividing". A fractional count is meaningless.

### Hardcoded mudlark-internal knobs · `sec:sentinel:mudlark-internal-knobs`

Two `Config<V>` fields are not exposed in `SentinelConfig`:

| Field | Value | Rationale |
|-------|-------|-----------|
| `alpha_relax` | `0.75` | Mudlark's recommended default. Controls depth-gate relaxation timing. Not performance-sensitive for the sentinel's use case. |
| `bounded_eviction` | `true` | Mudlark's recommended default. Prevents over-eviction. Always desirable. |

These can be promoted to `SentinelConfig` later if profiling reveals sensitivity.

---

## Part B — Cargo Feature Gating · `sec:sentinel:mudlark-part-b-feature-gating`

Mudlark exposes three Cargo features:

| Feature                     | Default? | What it provides |
|-----------------------------|----------|-----------------|
| `dynamic-contour-tracking`  | Yes      | `plateaus()`, contour queries |
| `rand`                      | Yes      | `WeightedSampler` (randomised sampling) |
| `serde`                     | Yes      | `Serialize`/`Deserialize` on all public types |

### Decisions · `sec:sentinel:mudlark-feature-decisions`

1. **Disable mudlark's default features** (`default-features = false`).

2. **Always enable `dynamic-contour-tracking`.** The analysis selector (§ALGO S-4) and report structure (§ALGO S-14) need `plateaus()` and contour queries.

3. **Do not enable `rand`.** Mudlark's `rand` feature provides `WeightedSampler`, which the sentinel does not use. The sentinel has its own `rand` dependency for noise generation.

4. **Gate `torrust-mudlark/serde` behind sentinel's `serde` feature:**

   ```toml
   [features]
   serde = ["dep:serde", "torrust-mudlark/serde"]
   ```

### Resulting `Cargo.toml` snippet · `sec:sentinel:mudlark-cargo-snippet`

```toml
[features]
serde = ["dep:serde", "torrust-mudlark/serde"]

[dependencies]
torrust-mudlark = { path = "../mudlark", default-features = false, features = ["dynamic-contour-tracking"] }
```

---

## Alternatives Considered · `sec:sentinel:mudlark-alternatives`

| Alternative | Pros | Cons |
|-------------|------|------|
| `V = f64` | Matches spec table literally | Lossy for a counter; `f64` loses precision above $2^{53}$ |
| `C = u64`, `N = 64` | Smaller coordinates | Loses half the address space; useless for IPv6 |
| Expose `alpha_relax` | Full tunability | Config surface without demonstrated need |
| `split_threshold: f64` in sentinel, cast `as u64` | Matches spec | Lossy cast, misleading type |
| Accept mudlark's default features | Simpler `Cargo.toml` | Pulls in `rand_core` unconditionally; forces serde on all builds |
| Gate `dynamic-contour-tracking` | Allows "headless" build | Every use of the G-V Graph needs contour queries |

## Consequences · `sec:sentinel:mudlark-consequences`

- `GvGraph<u128, u64, 128>` appears in the sentinel's struct definition and all downstream code.
- `u128` arithmetic is well-optimized on 64-bit platforms (two-wide operations). Benchmark the observe hot-path if performance concerns arise.
- `u64` accumulator means `decay()` operates on integer attenuation via mudlark's `Attenuatable` trait for `u64` (floor-rounding).
- `cargo check --no-default-features` builds with only `dynamic-contour-tracking` in mudlark — no serde, no rand.
- `cargo check --all-features` enables mudlark's `serde` via the transitive feature gate.

### Re-export policy · `sec:sentinel:mudlark-reexport-policy`

Only `GNodeId` is re-exported from mudlark to the sentinel's public API. Users needing other mudlark types (e.g., `GNodeInfo`, `Plateau`) must depend on `torrust-mudlark` directly. This minimises version-coupling — the sentinel can upgrade its internal mudlark dependency without breaking callers who don't use mudlark types in their API surface.
