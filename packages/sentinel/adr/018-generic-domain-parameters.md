# ADR-S-018: Generic Domain Parameters · `rec:sentinel:generic-coordinate-accumulator-and-domain-width`

**Status:** Accepted **Date:** 2026-03-13 **Spec:** §ALGO S-2.1 (domain), §ALGO S-2.4 (what sentinel owns), §ALGO S-3.2 (suffix encoding), §ALGO S-13.3 (G-V Graph config) **Supersedes:** [ADR-S-003](003-mudlark-integration.md) Part A (type parameters) **Relates to:** [ADR-S-003](003-mudlark-integration.md) (mudlark integration — Part B on Cargo features is unchanged), [ADR-S-002](002-feed-forward-invariant.md) (feed-forward invariant), mudlark [ADR-M-006](../../mudlark/adr/006-generic-parameters.md) (generic parameters), mudlark [ADR-M-009](../../mudlark/adr/009-trait-decomposition.md) (trait decomposition)

## Context · `sec:sentinel:domainparams-context`

ADR-S-003 Part A hardcodes `GvGraph<u128, u64, 128>`: the sentinel owns a single concrete instantiation of the G-V Graph with 128-bit coordinates, `u64` counts, and full 128-bit domain width.

Two developments challenge this:

1. **Some hosts require N = 64.** A host that encodes observations as `u64` cannot use a 128-bit domain without wasting 64 constant suffix dimensions, a full SVD rank slot (25% of `max_rank = 4`), and doubling per-tracker memory.

2. **Different hosts may want different accumulator semantics.** The sentinel exposes `graph()` to the host. With a hardcoded `V = u64`, the host receives `&GvGraph<u128, u64, 128>` and can call any method `u64` supports. But a host that wants a custom accumulator — weighted observations, saturating counters, a newtype with domain-specific overflow policy — cannot use the sentinel at all. The sentinel should not be the bottleneck on what the host can do with the graph it carries.

Mudlark is already fully generic: `GvGraph<C: Coordinate, V:
Accumulator, const N: u32>` works today for any conforming types. The sentinel is the only layer that locks the parameters.

## Decision · `sec:sentinel:domainparams-decision`

**`SpectralSentinel` becomes generic over `C`, `V`, and `N`, mirroring mudlark's `GvGraph<C, V, N>` triple.**

```rust
pub struct SpectralSentinel<C, V, const N: u32>
where
    C: Coordinate + CentredBitSource,
    V: Inspectable + Attenuatable,
{
    graph: GvGraph<C, V, N>,
    // ...
}
```

### The three parameters · `sec:sentinel:domainparams-three-parameters`

| Parameter | Role in sentinel | Bound | Rationale |
|-----------|-----------------|-------|-----------|
| `C` | Coordinate type — spatial addressing, interval bounds, observation values | `Coordinate + CentredBitSource` | Mudlark routing + sentinel bit-extraction |
| `V` | Accumulator type — importance accounting in the G-V Graph | `Inspectable + Attenuatable` | Sentinel calls `observe()`, `layers()`, `plateaus()`, `decay()` |
| `N` | Domain bit-width — tree height, suffix dimensions | `u32` (const generic) | Controls `CentredBits` vector length and max depth |

### Type aliases for common instantiations · `sec:sentinel:domainparams-type-aliases`

```rust
pub type Sentinel128 = SpectralSentinel<u128, u64, 128>;
pub type Sentinel64  = SpectralSentinel<u64, u64, 64>;
```

Existing consumers use `Sentinel128` or `Sentinel64` as appropriate for their domain width.

---

## Part A — Coordinate Parameter `C` · `sec:sentinel:domainparams-coordinate-parameter`

### `CentredBitSource`: sentinel's bridge trait · `sec:sentinel:domainparams-centred-bit-source`

The sentinel must convert coordinate values to centred bit vectors for subspace analysis (§ALGO S-3.2). This requires bit-level access that `Coordinate` does not provide. Rather than modify mudlark's trait surface, the sentinel defines a sealed internal trait:

```rust
pub trait CentredBitSource: Coordinate {
    fn to_centred_bits(&self, n: u32) -> CentredBits;
}

impl CentredBitSource for u64 {
    fn to_centred_bits(&self, n: u32) -> CentredBits {
        // Extract n bits: bit i → if (value >> (n-1-i)) & 1 { +0.5 } else { -0.5 }
    }
}

impl CentredBitSource for u128 {
    fn to_centred_bits(&self, n: u32) -> CentredBits {
        // Same, with 128-bit shifts.
    }
}
```

This trait is public — external consumers implementing a custom coordinate type must provide an impl to use it with the sentinel.

### `CentredBits`: fixed-size array with runtime length · `sec:sentinel:domainparams-centred-bits`

```rust
pub struct CentredBits {
    pub bits: [f64; 128],   // max-size stack buffer
    pub len: usize,         // actual width = N
}
```

The fixed array avoids heap allocation on the hot path. At N = 64, 64 trailing slots are unused — 512 bytes wasted per instance, but `CentredBits` is short-lived (created per observation, consumed immediately by the tracker). The `suffix()` method respects `len`: `&self.bits[depth..self.len]`.

### Sites that change · `sec:sentinel:domainparams-coordinate-change-sites`

Every `u128` in production code becomes `C`:

- `CellState { start: C, end: C }` — interval bounds
- `ingest(&[C])` — observation input
- `CentredBits::from_u128(v)` → `C::to_centred_bits(N)`
- `u128::MAX` → `C::domain_max(N)`
- `128 - depth` → `N as usize - depth as usize`

---

## Part B — Accumulator Parameter `V` · `sec:sentinel:domainparams-accumulator-parameter`

### Sentinel's relationship with `V` · `sec:sentinel:domainparams-accumulator-relationship`

The sentinel is a **pass-through** for accumulator values. It:

1. **Writes** unit deltas into the graph: `graph.observe(coord, delta)`
2. **Reads** importance back: `total_sum()`, `gnode_info().own`, `gnode_info().sum`
3. **Compares** importance: sorting by `PartialOrd` in analysis set selection
4. **Stores** importance in internal types: `AnalysisEntry`, `WarmingCell`
5. **Exposes** the graph: `graph() → &GvGraph<C, V, N>` — the host may call any method their `V` supports

The sentinel never performs V-arithmetic itself (no `add`, `sub`, `zero` in sentinel logic). It shuttles V values between mudlark and its own bookkeeping/report types.

### Bound: `V: Inspectable + Attenuatable` · `sec:sentinel:domainparams-accumulator-bound`

This is the minimum that makes the sentinel's mudlark API calls compile:

| Sentinel operation | Mudlark method | Required bound |
|---|---|---|
| Feed observations | `observe()` | `Inspectable` |
| Walk V-tree | `layers()` | `Inspectable` |
| Read contour | `plateaus()` | `Inspectable` |
| Temporal decay | `decay()` | `Inspectable + Attenuatable` |
| Read totals | `total_sum()`, `gnode_info()` | `Accumulator` (implied by `Inspectable`) |

The sentinel does **not** bound on `Weighable` or `Proratable`. But through `graph()`, the host can call `sample()` or `range_sum()` if their `V` satisfies those bounds — the sentinel carries `V` without constraining it beyond its own needs.

### Why not lock `V = u64`? · `sec:sentinel:domainparams-no-u64-lock`

Locking `V = u64` is simpler (no type parameter on ~10 internal types) but closes the door:

- A host using weighted observations (`observe(coord, weight)`) must use a different accumulator. The sentinel's Δ = 1 policy (ADR-S-002) governs *sentinel-initiated* observations, but a host wrapping the sentinel could feed different deltas for its own purposes.
- A host wanting `graph().sample()` with a custom weighting scheme needs `V: Weighable` — which `u64` satisfies, but a custom newtype would not unless the sentinel propagates `V`.
- Mudlark's `Config<V>` already requires `V` for `split_threshold`. The sentinel's config would need a conversion boundary rather than directly storing `V`.

The type parameter tax is real (~10 types gain `<V>`) but justified: the sentinel should not be the bottleneck on what the host can do with the spatial substrate it carries.

### Unit delta construction · `sec:sentinel:domainparams-unit-delta`

The sentinel observes with `Δ = 1` (ADR-S-002). With generic `V`, the unit delta is constructed via `Inspectable::from_f64(1.0)` and cached at construction time:

```rust
let unit_delta: V = V::from_f64(1.0);
// ...
self.graph.observe(value, unit_delta);
```

For all built-in types this is exact. For custom types, `from_f64(1.0)` must return the type's unit increment — this is an invariant of any sensible `Inspectable` implementation.

### `SentinelConfig` carries `V` · `sec:sentinel:domainparams-config-accumulator`

```rust
pub struct SentinelConfig<V: Accumulator> {
    pub split_threshold: V,
    // ...~30 other fields unchanged (f64, u32, usize, bool)
}
```

One field out of ~30 carries `V`. The cost is that consumers write `SentinelConfig<u64>` instead of `SentinelConfig`. This is acceptable — it matches mudlark's `Config<V>` pattern and makes the type-level contract explicit.

---

## Part C — Domain Width `N` · `sec:sentinel:domainparams-domain-width`

### Propagation · `sec:sentinel:domainparams-width-propagation`

`N` propagates through `GvGraph<C, V, N>` and into every width computation:

- Root cell width: `N`
- Suffix width: `N - depth`
- `CentredBits` vector length: `N`
- Max depth: `N` (fits in `u8` for N ≤ 255)

### `SubspaceTracker`: already generic · `sec:sentinel:domainparams-tracker-generic`

The core SVD engine takes `dim: usize` at runtime construction. It contains no hardcoded `128`. No changes needed.

### Config: domain-independent · `sec:sentinel:domainparams-domain-independent-config`

`SentinelConfig` has no N-dependent fields. All thresholds are `f64` or `u32`. The noise schedule uses depth tiers, not absolute widths. N propagates only through the type system, not through config values.

---

## Part D — Report Type Strategy · `sec:sentinel:domainparams-report-strategy`

### Coordinate in reports: generic `C` · `sec:sentinel:domainparams-generic-report-coordinates`

Report types that carry interval bounds become generic over `C`:

```rust
pub struct CellReport<C>       { pub start: C, pub end: C, … }
pub struct CoordinationReport<C> { pub start: C, pub end: C, … }
pub struct MemberScore<C>      { pub cell_start: C, pub cell_end: C, … }
pub struct CellInspection<C>   { pub start: C, pub end: C, … }
```

### Accumulator in reports: erased to `f64` · `sec:sentinel:domainparams-erased-report-accumulator`

Importance values in reports are diagnostic — the host reads them for health monitoring, not for correctness-critical computation. Rather than propagating `V` through all report types (which would make `BatchReport<C, V>`), importance is erased at the report boundary via `Inspectable::to_f64_approx()`:

```rust
pub struct ContourSnapshot {
    pub total_importance: f64,   // V::to_f64_approx()
    // ...
}

pub struct AnalysisSetSummary {
    pub importance_range: (f64, f64),
    // ...
}
```

This keeps `BatchReport<C>` — one generic parameter, not two. The precision loss (`u64` values above $2^{53}$ lose LSBs) is acceptable for diagnostic fields. A host needing exact importance can read `graph().total_sum()` directly, which returns `V`.

**Exception:** `AnalysisEntry` (internal type) retains `V` for importance because the analysis set sorts by exact importance values:

```rust
pub struct AnalysisEntry<C, V: Accumulator> {
    pub importance: V,
    pub start: C,
    pub end: C,
    // ...
}
```

This type is `pub(crate)` — it does not surface in the public API.

---

## Type Propagation Summary · `sec:sentinel:domainparams-propagation-summary`

| Type | Parameters | Public? |
|------|-----------|---------|
| `SpectralSentinel<C, V, N>` | `C, V, N` | Yes |
| `SentinelConfig<V>` | `V` | Yes |
| `BatchReport<C>` | `C` | Yes |
| `CellReport<C>` | `C` | Yes |
| `CoordinationReport<C>` | `C` | Yes |
| `MemberScore<C>` | `C` | Yes |
| `CellInspection<C>` | `C` | Yes |
| `ContourSnapshot` | — | Yes |
| `AnalysisSetSummary` | — | Yes |
| `HealthReport` | — | Yes |
| `CellState<C>` | `C` | No (`pub(super)`) |
| `AnalysisEntry<C, V>` | `C, V` | No (`pub(crate)`) |
| `AnalysisSet<C, V>` | `C, V` | No (`pub(crate)`) |
| `WarmingCell<C, V>` | `C, V` | No (`pub(crate)`) |
| `StagingArea<C, V>` | `C, V` | No (`pub(crate)`) |
| `CentredBits` | — | No |
| `SubspaceTracker` | — | No |

V appears in 5 types (1 public: `SentinelConfig`; 4 internal).

---

## Alternatives Considered · `sec:sentinel:domainparams-alternatives`

| Alternative | Pros | Cons |
|-------------|------|------|
| **Lock `V = u64`, generic `C` and `N` only** | Fewer types carry V; simpler config | Closes door on custom accumulators; host cannot use `graph()` with custom V; diverges from mudlark pattern |
| **Runtime `domain_bits: u32` instead of `const N`** | No const-generic propagation; tests keep `u128` | Wastes 8 bytes per coord at N=64; no compile-time enforcement; slight runtime overhead |
| **`V` generic in reports (not erased)** | Exact importance in reports | `BatchReport<C, V>` — two generics on every consumer; ~5 more types carry V for diagnostic-only fields |
| **`CentredBits` as `Vec<f64>`** | Exact sizing | Heap allocation per observation per cell on the hot path; dwarfed by SVD cost but unnecessary |
| **Add `bit(index) → bool` to mudlark's `Coordinate`** | Clean bit extraction | Changes mudlark's trait surface for sentinel's benefit; `CentredBitSource` in sentinel is less invasive |
| **`GvGraph<C, V, N>` exposed via trait object / type-erased wrapper** | Host doesn't see `V` | Loses monomorphisation; runtime dispatch on hot path; `GvGraph` is not object-safe |

## Consequences · `sec:sentinel:domainparams-consequences`

- `SpectralSentinel<C, V, N>` replaces the concrete `SpectralSentinel`. Type aliases `Sentinel128` and `Sentinel64` provide ergonomic shorthand.

- ADR-S-003 Part A is superseded. Part B (Cargo features) is unchanged.

- The sentinel's minimum bound on V (`Inspectable + Attenuatable`) is strictly less than "all four sub-traits". A host with a custom V that only implements these two bounds can use the sentinel; it just cannot call `graph().sample()` or `graph().range_sum()` — the compiler enforces this per-method, not per-struct.

- `SentinelConfig<V>` carries V for `split_threshold`. All other config fields remain concrete. Consumers write `SentinelConfig<u64>` — one extra token.

- Report types carry only `<C>`, not `<V>`. Importance values are `f64` in reports. Hosts needing exact V read `graph().total_sum()` or `graph().gnode_info().own` directly.

- `CentredBitSource` is a public sentinel trait. Adding support for a new coordinate type (e.g. `u32`, `f64`) requires an impl in the downstream crate — a one-function addition.

- Test code migrates gradually: existing `u128` tests can use `Sentinel128`; new `u64` tests use `Sentinel64`. No test needs both instantiations simultaneously.

- The compiler catches every missed migration site: a `u128` where `C` is expected is a type error, not a runtime bug.
