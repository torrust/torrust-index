# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2026-03-11

Initial stable release. The public API surface documented in
[docs/api.md](docs/api.md) is now covered by semver guarantees.

### Added

- **Dual-tree index (`GvGraph<C, V, N>`)** — adaptive-resolution spatial
  index backed by interlocking G-Tree (geometric) and V-Tree (value)
  structures.
- **Observation pipeline** — `observe(coord, delta)` with automatic
  split, rebalance, and eviction.
- **Batch construction** — `GvGraph::from_observations(config, iter)`.
- **`Extend<(C, O)>`** — incremental batch insertion via
  `g.extend([(coord, delta), …])`.
- **Point query** — `get(coord)` returning `Cell<C, V>` snapshots.
- **Plateau projection** — `plateaus()` in O(1) with the
  `dynamic-contour-tracking` feature.
- **Range sum** — `range_sum(range)` for G-Tree recursive range
  queries (requires `V: Proratable`).
- **Contour range queries** — `contour_range(start, end)` and
  `contour_range_energy(start, end)` decomposing lattice-aligned
  intervals into interior, boundary, and straddling elements
  (requires `V: Proratable + Inspectable`).
- **Plateau selection** — `select_plateaus(lo, hi)` snapping
  arbitrary coordinates to lattice-aligned contour range endpoints
  (requires `V: Proratable + Inspectable`).
- **Proportional sampling** — `sample(rng)` with O(1.44 H + 1.67)
  expected cost (inherent: `V: Weighable`; via `WeightedSampler`
  trait: `V: Weighable + Inspectable`).
- **PEWEI extraction** — `extract()` producing significance-ordered
  `Pewei<C, V>` snapshots with layer iteration and reconstruction.
- **Streaming V-Tree BFS** — `layers()` yielding `(layer_index, Node)`
  lazily.
- **Temporal decay** — `decay(root, attenuation, q)` with
  subband-adaptive scaling (requires `V: Attenuatable +
  Inspectable`).
- **Budget enforcement** — configurable hard budget with
  `check_evictions()` and automatic eviction during observation.
- **Accessors** — `node_count()`, `terminal_count()`, `budget()`,
  `total_sum()`, `config()`, `g_root()`, `v_root()`,
  `depth_evict()`, `depth_create()`, `depth_buffer()`,
  `headroom()`, `soft_limit()` — all O(1).
- **Three-surface visibility model** (ADR-M-032) — Prints (Surface 1),
  Film (Surface 2), Emulsion (Surface 3) with flat crate-root
  re-exports.
- **Chemistry contract traits** — `Coordinate`, `Accumulator`,
  `Attenuatable`, `Weighable`, `Proratable`, `Inspectable`,
  `Observation<V>`, `Rng`.
- **Instrument traits** — `SpatialRead`, `SpatialWrite`,
  `TemporalDecay`, `WeightedSampler`.
- **Built-in implementations** for `u8`–`u128`, `f32`, `f64`.
- **Feature flags** — `dynamic-contour-tracking` (default), `serde`
  (default), `rand` (default).
- **Serde support** — `Serialize`/`Deserialize` on all Surface 1
  snapshot types behind the `serde` feature.
- **rand integration** — blanket `Rng` impl for `rand_core::Rng`
  behind the `rand` feature.
- **Invariant checker** — `assert_invariants()` exposed as a
  `#[doc(hidden)]` testing affordance.
- **Shared test infrastructure** — config presets, plan runner, fluent
  builder, and RNG stubs in the `testing` module.
- **143 Criterion benchmarks** across six families (observe, query,
  extract, lifecycle, spray, pathological).
- **833 tests** (346 unit, 379 integration, 108 doc-tests).
- **37 architecture decision records** in `adr/`.
- **Full documentation** — `idea.md` (formal spec), `api.md`,
  `architecture.md`, `performance.md`, `testing.md`, `theory.md`.

[1.0.0]: https://github.com/torrust/torrust-index/releases/tag/mudlark-v1.0.0
