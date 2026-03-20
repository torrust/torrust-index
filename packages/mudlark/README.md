# Mudlark

> Named after the Magpie-lark — a small Australian bird whose mated pairs
> like to duet together.

A **Dual-Tree Value-Stratified Index** (φ-Bounded Geometric-Value Graph,
or `GvGraph`) that maintains a spatial index whose logical hierarchy
adapts to value distribution. Two interlocking trees — one spatial, one
significance-ordered — protect each other to deliver entropy-optimal
proportional sampling, adaptive resolution, and temporal decay.

## Stability

This crate follows [Semantic Versioning](https://semver.org/). The
public API surface documented in [docs/api.md](docs/api.md) — every
type, trait, and method re-exported from the crate root — is covered by
semver guarantees from 1.0.0 onwards.

Internal machinery (Surface 3 / `pub(crate)`) is not part of the
public API and may change in any release.

**MSRV:** 1.80

See [CHANGELOG.md](CHANGELOG.md) for the release history.

## Installation

Add the crate to your project:

```sh
cargo add torrust-mudlark
```

Or add it manually to your `Cargo.toml`:

```toml
[dependencies]
torrust-mudlark = "1"
```

The two default features (`dynamic-contour-tracking` and `rand`) are
enabled automatically. The `serde` feature is opt-in:

```toml
[dependencies]
torrust-mudlark = { version = "1", features = ["serde"] }
```

Users who need a minimal build should use `default-features = false`,
which disables both default features simultaneously:

```toml
[dependencies]
torrust-mudlark = { version = "1", default-features = false }
```

## Quick start

```rust
use torrust_mudlark::{Config, GvGraph};

// Configure the index.
let config = Config {
    split_threshold: 10u64,
    depth_create: 2,
    depth_evict: 5,
    budget: Some(128),
    alpha_relax: 0.5,
    bounded_eviction: true,
};

// Create and populate.
let mut graph = GvGraph::<u64, u64, 16>::new(config);
graph.observe(100, 5u64);
graph.observe(200, 3u64);

// Point query.
let cell = graph.get(100);
println!("{cell:?}");

// Plateau projection (O(1) with default features).
let plateaus = graph.plateaus();
for (_, p) in &*plateaus {
    println!("depth {} over [{}, {})", p.depth, p.start, p.end);
}

// PEWEI extraction.
let pewei = graph.extract();
println!("{} layers, {} nodes", pewei.layer_count(), pewei.node_count());
```

## Core ideas

**G-Tree (Geometric Tree)** — a binary tree over dyadic intervals of
`[0, 2^N)`. Every materialised node stores the accumulated value for its
range. The G-Tree answers spatial queries and maintains a contour — the
observation-receiving surface of the domain.

**V-Tree (Value Tree)** — a dynamic tournament bracket (branching factor
2 or 3) governed by the max-uncle constraint. High-intensity entries rise
near the root for efficient sampling; low-intensity entries consolidate
deeper.

**The two trees protect each other.** The V-Tree decides what earns
spatial resolution; the G-Tree's routing controls which entries receive
observations. Together they provide proportional sampling within factor
1.44 of Shannon entropy.

**PEWEI** (Progressive Entropic-Wavelet Exposure Image) — the extracted
significance-ordered snapshot of the live index.

## Public API surface

The crate follows a **three-surface visibility model** (see
[ADR-M-032](adr/032-three-surface-model.md)):

| Surface | Name         | What it exposes                                       |
| ------- | ------------ | ----------------------------------------------------- |
| 1       | **Prints**   | Lightweight `Copy`/`Clone` view types users hold      |
| 2       | **Film**     | Opaque operational types users interact through       |
| 3       | **Emulsion** | `pub(crate)` internal machinery — not part of the API |

All public types are re-exported flat from the crate root — one
canonical path per type.

### Key types

| Type                 | Surface | Description                                                          |
| -------------------- | ------- | -------------------------------------------------------------------- |
| `GvGraph<C, V, N>`   | 2       | The live dual-tree index                                             |
| `Config<V>`          | 2       | Index configuration (no `Default` — parameters are domain-dependent) |
| `Cell<C, V>`         | 1       | Snapshot of a terminal G-node                                        |
| `Span<C, V>`         | 1       | Dyadic interval + intensity                                          |
| `Node<C, V>`         | 1       | Snapshot of any G-node (own, sum, state)                             |
| `GState`             | 1       | Terminal / SemiInternal / Internal discriminant                      |
| `Pewei<C, V>`        | 1       | Full significance-ordered extraction                                 |
| `Layer<C, V>`        | 1       | One BFS depth-level of the V-Tree                                    |
| `Transition<C, V>`   | 1       | Phase-transition node — region with finer spatial structure          |
| `Terminal<C, V>`     | 1       | Leaf node — no further subdivision                                   |
| `Plateau<C, V>`      | 1       | One contiguous region of uniform contour depth                       |
| `BasisEdge<C>`       | 1       | `Ord`-providing newtype for plateau `BTreeMap` keys                  |
| `ContourRange<C, V>` | 1       | Full contour-range decomposition of a lattice-aligned interval       |
| `ContourRangeEnergy<V>` | 1    | Energy-only result of a contour range query                          |
| `BasisElement<C, V>`    | 1    | One element of the minimal G-node cover of a contour range           |
| `GNodeId`            | 1       | Opaque arena handle                                                  |

### Key operations on `GvGraph`

| Method                            | Description                                                                    |
| --------------------------------- | ------------------------------------------------------------------------------ |
| `new(config)`                     | Construct with a single root G-node covering `[0, 2^N)`                        |
| `from_observations(config, iter)` | `new` + `extend` from `(coord, delta)` pairs                                   |
| `observe(coord, delta)`           | Route → accumulate → split → rebalance → evict                                 |
| `get(coord)`                      | Infallible point query — $O(\text{depth})$                                     |
| `plateaus()`                      | Spatial contour projection — $O(1)$ borrowed (requires `V: Inspectable`)       |
| `range_sum(range)`                | Recursive G-Tree range sum — $O(N)$ (requires `V: Proratable`)                 |
| `contour_range(start, end)`       | Contour range decomposition — $O(N)$ (requires `V: Proratable + Inspectable`)  |
| `contour_range_energy(start, end)` | Energy-only contour range query — $O(N)$ (requires `V: Proratable + Inspectable`) |
| `select_plateaus(lo, hi)`          | Plateau selection — snaps arbitrary coords to lattice endpoints — $O(\log P)$ (requires `V: Inspectable`) |
| `sample(rng)`                     | Proportional sampling — $O(1.44\,H)$ expected (requires `V: Weighable`) |
| `extract()`                       | PEWEI extraction — $O(n)$ (requires `V: Inspectable`)                          |
| `layers()`                        | Streaming V-Tree BFS yielding `(layer_index, Node)` — lazy, $O(V)$ BFS queue (requires `V: Inspectable`) |
| `decay(root, attenuation, q)`     | Subband-adaptive temporal decay (requires `V: Attenuatable + Inspectable`)     |
| `check_evictions()`               | Manual eviction sweep (requires `V: Inspectable`)                              |

### Traits

#### Chemistry contracts

| Trait            | Purpose                                                                                                                                                  |
| ---------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Coordinate`     | Spatial coordinate — provided for `u8`..`u128`, `f32`, `f64`                                                                                             |
| `Accumulator`    | Core intensity/value type — non-negative, starting from zero. Provided for `u8`..`u128`, `f32`, `f64`                                                    |
| `Attenuatable`   | Multiplicative decay — required by `decay()` / `TemporalDecay`                                                                                           |
| `Weighable`      | Weight projection to `f64` — required by `sample()` / `WeightedSampler`                                                                                  |
| `Proratable`     | Fractional subdivision — required by `range_sum()`, `contour_range()`, `contour_range_energy()`, and `Pewei::reconstruct()`         |
| `Inspectable`    | Diagnostic `f64` projection — baseline for most operations (`observe`, `plateaus`, `extract`, `layers`, `contour_range`, `from_observations`, `check_evictions`, `decay`) |
| `Observation<V>` | Accumulation rule (blanket impl for same-type; cross-type for floats)                                                                                    |
| `Rng`            | Minimal RNG — blanket impl for `rand_core::Rng` with `rand` feature                                                                                  |

**Why `Accumulator` requires non-negative values starting from zero.**
The spec's importance interface is deliberately general — it defines a
menu of algebraic properties (P0–P5) that a value type _may_ satisfy,
and each property unlocks specific features. A signed type like `i64`
could support governance but would lose proportional sampling and
efficient eviction. A type whose "empty" value isn't zero could still
work but would need extra bookkeeping after every split.

This implementation chooses to support only the
[Standard configuration](docs/idea.md#166-recommended-configurations):
values are non-negative, zero is both "empty" and the smallest possible
value, and addition is the usual kind. That single choice gives every
`GvGraph` the full feature set — violation-free splits, a fast path for
evicting empty nodes, well-defined sampling weights, and a logarithmic
depth guarantee — with no runtime branching or fallback paths. All
built-in numeric types (`u8`–`u128`, `f32`, `f64`) satisfy this
automatically. Custom types just need to uphold the same contract:
`zero()` is the identity for `add()` and no value is less than `zero()`.

This is why signed integer types (`i8`–`i128`) are not provided as
`Accumulator` implementations: their zero is not the smallest value, so
they cannot satisfy the Standard configuration. Supporting them would
require fallback paths for every feature they break.

#### Instruments

| Trait             | Purpose                                                                           |
| ----------------- | --------------------------------------------------------------------------------- |
| `SpatialRead`     | Read-only spatial queries (`plateaus`, `get`) — object-safe                       |
| `SpatialWrite`    | Mutation (`observe`)                                                              |
| `TemporalDecay`   | Decay (`decay`) — split from `SpatialWrite` because it requires `V: Attenuatable` |
| `WeightedSampler` | Proportional sampling (`sample`)                                                  |

## Features

| Feature                    | Default | Effect                                                              |
| -------------------------- | ------- | ------------------------------------------------------------------- |
| `dynamic-contour-tracking` | yes     | Live plateau mirror; `plateaus()` returns `Cow::Borrowed` in $O(1)$ |
| `serde`                    | no      | `Serialize`/`Deserialize` on all Surface 1 snapshot types           |
| `rand`                     | yes     | Blanket `Rng` impl for all `rand_core::Rng` types               |

## Performance

143 Criterion benchmarks across six families validate the theoretical
complexity claims. Headline numbers on the u64 configuration:

| Operation           | Latency            | Complexity                           |
| ------------------- | ------------------ | ------------------------------------ |
| `plateaus()`        | **2.6–2.9 ns**     | $O(1)$ — reference borrow            |
| `get(x)`            | **14–22 ns**       | $O(\log P)$ — plateau floor-key      |
| `sample()` hotspot  | **10 ns**          | $O(1.44\,H)$ — near-zero entropy     |
| `sample()` uniform  | **126 ns**         | $O(1.44\,H)$ — max entropy           |
| `range_sum()`       | **3–77 ns**        | $O(N)$ — width-dependent             |
| `observe()` cold    | **142–493 ns/obs** | $O(d_\text{geo})$ amortised          |
| `observe()` steady  | **1.8 µs/obs**     | Budget pressure + eviction           |
| `decay()` selective | **5.5–8.1 µs**     | Subband-adaptive, sub-linear scaling |
| `check_evictions()` | **58–59 µs**       | Budget enforcement sweep             |

The entropy-adaptive sampling claim is confirmed: hotspot distributions
sample **12× faster** than uniform. Fifteen adversarial patterns
(spine, fractal fill, gray code, pincer, and others) confirm
sub-quadratic throughput under worst-case access sequences.

See [docs/performance.md](docs/performance.md) for the full analysis
with scaling exponents, per-family breakdowns, f64 comparison, and a
claims scorecard.

## Documentation

| Document                                             | Contents                                    |
| ---------------------------------------------------- | ------------------------------------------- |
| [docs/idea.md](docs/idea.md)                         | Formal specification                        |
| [docs/api.md](docs/api.md)                           | Public API reference                        |
| [docs/architecture.md](docs/architecture.md)         | Implementation architecture                 |
| [docs/performance.md](docs/performance.md)           | Benchmark results and performance analysis  |
| [docs/testing.md](docs/testing.md)                   | Test suite breakdown and running guide      |
| [docs/theory.md](docs/theory.md)                     | Theoretical background                      |
| [adr/](adr/)                                         | 37 architecture decision records            |
| [CHANGELOG.md](CHANGELOG.md)                         | Release history                             |

## Design records

Architectural decisions are recorded in [adr/](adr/). Notable entries:

- [ADR-M-032](adr/032-three-surface-model.md) — Three-surface visibility model
- [ADR-M-019](adr/019-sampling-semantics.md) — Sampling semantics
- [ADR-M-024](adr/024-decay-semantics.md) — Decay semantics
- [ADR-M-025](adr/025-public-api-surface.md) — Public API surface
- [ADR-M-020](adr/020-range-query-design.md) — Range query design
- [ADR-M-021](adr/021-pewei-output-representation.md) — PEWEI output representation
- [ADR-M-037](adr/037-contour-range-queries.md) — Contour range queries

## Development

### Building and testing

```sh
# Unit + integration tests (debug, optimised build):
CARGO_PROFILE_DEV_OPT_LEVEL=3 cargo test -p torrust-mudlark --all-targets --all-features

# Doc-tests:
cargo test -p torrust-mudlark --all-features --doc

# Release mode:
cargo test -p torrust-mudlark --all-targets --all-features --release

# No-default-features spot-check:
CARGO_PROFILE_DEV_OPT_LEVEL=3 cargo test -p torrust-mudlark --all-targets --no-default-features

# Clippy and docs:
cargo clippy -p torrust-mudlark --all-targets --all-features
cargo doc -p torrust-mudlark --all-features --no-deps
```

868 tests across unit, integration, and doc-test suites (346 unit,
415 integration, 107 doc-tests).
See [docs/testing.md](docs/testing.md) for the full breakdown,
feature-gated variation, and benchmark details.

### Benchmarks

Run all benchmarks:

    cargo bench --package torrust-mudlark

Run a specific family:

    cargo bench --package torrust-mudlark -- observe/
    cargo bench --package torrust-mudlark -- spray/steady_state

Smoke-test (compile + single iteration, no timing):

    cargo bench --package torrust-mudlark -- --test

Reports at `target/criterion/report/index.html`.

Save/compare baselines:

    cargo bench --package torrust-mudlark -- --save-baseline my-baseline
    cargo bench --package torrust-mudlark -- --baseline my-baseline

See [adr/035-benchmarking-framework.md](adr/035-benchmarking-framework.md) for details.

### Cross-reference conventions

Cross-references use `§` (section sign):

| Reference  | Meaning                                           | Examples              |
| ---------- | ------------------------------------------------- | --------------------- |
| `§N`       | Spec section N ([docs/idea.md](docs/idea.md))     | `§IDEA M-9.3`, `§§IDEA M-12.2–12.5` |
| `§IDEA M-N`  | Spec section N (explicit form for src/)           | `§IDEA M-9.3`           |
| `§PEWEI M-N` | PEWEI appendix section N                          | `§PEWEI M-2`            |
| `§API M-N`   | [api.md](docs/api.md) section N                   | `§API M-3.3`            |
| `§ARCH M-N`  | [architecture.md](docs/architecture.md) section N | `§ARCH M-2`             |
| `§PERF N`  | [performance.md](docs/performance.md) section N   | `§PERF 9`             |
| `§TEST M-N`  | [testing.md](docs/testing.md) section N           | `§TEST M-4`             |
| `ADR-M-NNN`  | Mudlark Architecture Decision Record by number      | `ADR-M-024`           |
| `ADR-S-NNN`  | Sentinel Architecture Decision Record by number     | `ADR-S-005`           |

Use `§§` for ranges: `§§IDEA M-12.2–12.5`.

## License

Copyright (c) 2026 Torrust project contributors.

This program is free software: you can redistribute it and/or modify it
under the terms of the **GNU Affero General Public License v3.0 only**
(AGPL-3.0-only) as published by the Free Software Foundation.

The full license text is available in the repository root
([LICENSE](../../LICENSE)) and at
<https://www.gnu.org/licenses/agpl-3.0.html>.

### Public API and linking

The **public API surface** of this crate — the types, traits, and
function signatures exposed through Surface 1 (Prints) and Surface 2
(Film) as documented in [docs/api.md](docs/api.md) — constitutes an
**interface** in the sense of §AGPL_3 0 and §AGPL_3 13. Using these public types
and traits from your own code (i.e. calling methods on `GvGraph`,
implementing `Coordinate` or `Accumulator`, consuming `Cell`/`Span`/
`Pewei` values) does **not** create a derivative work of the library and
therefore does **not** trigger the AGPL's copyleft obligations on your
code.

In concrete terms: you may depend on `torrust-mudlark` as a library,
call its public API, and distribute your application under any license
you choose — provided you do not modify _this_ crate's source code. If
you modify the crate itself, the AGPL applies to those modifications in
the usual way.
