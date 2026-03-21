# Mudlark

> Named after the Magpie-lark — a small Australian bird whose mated pairs
> like to duet together.

An adaptive spatial index that reallocates resolution toward observed
significance, backed by two interlocking trees — one spatial, one
significance-ordered. Together they deliver near-entropy-optimal
proportional sampling (within factor 1.44 of Shannon entropy), adaptive
resolution, and temporal decay. Formally: a Dual-Tree Value-Stratified
Index (φ-Bounded Geometric-Value Graph, or `GvGraph`).

See [Architecture in brief](#architecture-in-brief) below for the
conceptual model, or jump straight to [Quick start](#quick-start) for
code.

## Choose `GvGraph`

When your data has **spatial locality and temporal non-stationarity** —
that is, hot regions shift over time and some regions deserve much finer
resolution than others.

If your distribution is static and known in advance, a pre-built segment
tree will be simpler. For uniform distributions with no hotspots, a flat
array suffices. For multi-dimensional data, see the composition
architecture in [docs/theory.md](docs/theory.md). The V-Tree's advantage
is online adaptation without reconfiguration.

## Stability

This crate follows [Semantic Versioning](https://semver.org/). The
public API surface documented in [docs/api.md](docs/api.md) — every
type, trait, and method re-exported from the crate root — is covered by
semver guarantees from 1.0.0 onwards.

Internal machinery (Surface 3 / `pub(crate)`) is not part of the
public API and may change in any release.

**MSRV:** 1.80 (tested in CI)

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

> Every claim below is asserted with full invariant checks in the
> [pedagogy integration test](tests/pedagogy.rs). For the complete
> step-by-step invariant verification, see §IDEA M-16.

### Create the index

```rust
use torrust_mudlark::{Config, GvGraph};

let config = Config {
    split_threshold: 5u64,  // θ — minimum intensity to trigger a split
    depth_create: 3,        // max V-Tree depth for split authorization
    depth_evict: 6,         // min V-Tree depth for eviction eligibility
    budget: None,           // no hard node-count ceiling
    alpha_relax: 0.75,      // depth-gate hysteresis
    bounded_eviction: true, // stop eviction scan early once under budget
};
//                          ↓ N = 3: domain is [0, 2³) = [0, 8)
let mut g = GvGraph::<u64, u64, 3>::new(config);
```

`GvGraph` also implements `Extend<(C, O)>` for bulk-loading and `Clone`
for checkpoint-and-fork workflows. `from_observations(config, iter)`
combines construction and bulk-load in one call.

### Feed data — the tree adapts

Three observations are enough to see the full adaptive resolution cycle:

```rust
# use torrust_mudlark::{Config, GvGraph};
# let config = Config { split_threshold: 5u64, depth_create: 3, depth_evict: 6,
#     budget: None, alpha_relax: 0.75, bounded_eviction: true };
# let mut g = GvGraph::<u64, u64, 3>::new(config);
g.observe(3, 10u64);  // exceeds θ → bootstrap split: [0,4) and [4,8)
g.observe(3, 15u64);  // left half splits again; L(15) outgrows root(10) → promoted
g.observe(6, 8u64);   // right half splits; R(8) ≤ uncle L(15) → no rebalancing
```

After the second observation the contour is **asymmetric** — fine where
the signal concentrates, coarse where it doesn't:

```text
     ┌──────────────────────────────────────────┐
     │  ┌──────────────────┐                    │
     │  │  ┌──────┐┌──────┐│                    │
     └──┴──┴──────┴┴──────┴┴────────────────────┘
     0     2       4                            8

     depth 2 ▔▔▔▔▔▔▔▔▔▔▔▔▔   depth 1 ▔▔▔▔▔▔▔▔▔
     fine (hotspot)            coarse (quiet)
```

The third observation balances the contour back to uniform depth 2 — but
the V-Tree tournament _remembers_ that `[0,4)` proved itself first. It
sits at tournament layer 1; `[4,8)` sits deeper, shielded by its uncle.

### Read it back

```rust
# use torrust_mudlark::{Config, GvGraph};
# let config = Config { split_threshold: 5u64, depth_create: 3, depth_evict: 6,
#     budget: None, alpha_relax: 0.75, bounded_eviction: true };
# let mut g = GvGraph::<u64, u64, 3>::new(config);
# g.observe(3, 10u64);
# g.observe(3, 15u64);
# g.observe(6, 8u64);
// Point query — routes to the contour cell covering the coordinate
let cell = g.get(3);
assert_eq!((cell.start, cell.end, cell.depth), (2, 4, 2));

// Plateaus — the learned spatial shape, O(1) with default features
let plateaus = g.plateaus();
assert_eq!(plateaus.len(), 1);  // uniform after the third observation

// Range sum — pro-rated aggregate over any interval
assert_eq!(g.range_sum(..), 33);        // full domain
assert_eq!(g.range_sum(0u64..4), 20);   // left half only

// Significance-ordered snapshot (PEWEI) — energy is conserved
let pewei = g.extract();
assert_eq!(pewei.total_energy(), g.total_sum());
```

**Proportional sampling** draws from the V-Tree tournament at expected
cost 𝒪(1.44 H + 1) where H is the Shannon entropy of the weight
distribution. A balanced segment tree always pays Θ(log n) per sample
regardless of how concentrated the weights are. The V-Tree's cost
tracks entropy, not tree size — so for concentrated distributions
(Zipf, geometric, a handful of hotspots) sampling is 𝒪(1). Under
fixed concentration, the advantage over Θ(log n) grows without bound
as the index grows:

```rust
# use torrust_mudlark::{Config, GvGraph};
# let config = Config { split_threshold: 5u64, depth_create: 3, depth_evict: 6,
#     budget: None, alpha_relax: 0.75, bounded_eviction: true };
# let mut g = GvGraph::<u64, u64, 3>::new(config);
# g.observe(3, 10u64);
let cell = g.sample(&mut rand::rng()).unwrap();
```

> With the default `rand` feature, any `rand_core::RngCore` type works
> directly. Without it, implement `torrust_mudlark::Rng` on your own type.

### Targeted decay — selective forgetting

New observations can shift the competitive landscape, and decay can
reshape it further:

```rust
# use torrust_mudlark::{Config, GvGraph};
# let config = Config { split_threshold: 5u64, depth_create: 3, depth_evict: 6,
#     budget: None, alpha_relax: 0.75, bounded_eviction: true };
# let mut g = GvGraph::<u64, u64, 3>::new(config);
# g.observe(3, 10u64);
# g.observe(3, 15u64);
# g.observe(6, 8u64);
g.observe(5, 20u64);  // new hotspot on the right — [4,6) splits, V-Tree reshuffles
g.observe(1, 6u64);   // modest hit on the left — accumulates quietly

// Find [0,4) via V-Tree BFS, then decay that subtree by 90%
let left = g.layers()
    .find_map(|(_, n)| (n.start == 0 && n.end == 4).then_some(n.gnode_id))
    .unwrap();
g.decay(left, 0.1, 0.0);  // attenuation=0.1, uniform across depths (q=0)

// Right side now dominates the tournament; energy is still conserved
let pewei = g.extract();
assert_eq!(pewei.total_energy(), g.total_sum());
```

Decay doesn't destroy spatial structure — the same contour cells
persist. It reshapes the _competitive ranking_, so sampling and the
PEWEI reflect the shifted significance landscape. Global decay,
depth-selective decay (the `q` parameter), and annihilation (`att = 0`)
are all supported — see [docs/api.md](docs/api.md) for the full
semantics.

## Advanced usage

> Every claim below is asserted with full invariant checks in the
> [advanced pedagogy test](tests/pedagogy_advanced.rs). For the
> complete step-by-step narrative, see the source of that test.

The quick start covers building and reading a single graph. This
section covers construction alternatives, structural introspection,
the contour range decomposition, progressive reconstruction, and
budget-constrained operation.

All examples below build on the five canonical observations — the
three from the quick start plus two more that create a depth-3
hotspot at `[4,6)`:

```rust
# use torrust_mudlark::{Config, GvGraph};
let config = Config { split_threshold: 5u64, depth_create: 3, depth_evict: 6,
    budget: None, alpha_relax: 0.75, bounded_eviction: true };
// from_observations is new() + extend() in a single call
let g = GvGraph::<u64, u64, 3>::from_observations(config,
    [(3, 10u64), (3, 15), (6, 8), (5, 20), (1, 6)]);
assert_eq!(g.total_sum(), 59);
assert_eq!(g.plateaus().len(), 3);  // depth-2 / depth-3 / depth-2
```

Each element runs the full observe pipeline, so order matters
structurally (the same five observations in a different order may
produce a different V-Tree shape — different internal tournament
rankings — even though `total_sum()` and `get()` answers are the
same).

### Checkpoint and fork

`Clone` produces an independent deep copy. Mutating the fork doesn't
affect the original — the negative is duplicated, not aliased:

```rust
# use torrust_mudlark::{Config, GvGraph};
# let config = Config { split_threshold: 5u64, depth_create: 3, depth_evict: 6,
#     budget: None, alpha_relax: 0.75, bounded_eviction: true };
# let g = GvGraph::<u64, u64, 3>::from_observations(config,
#     [(3, 10u64), (3, 15), (6, 8), (5, 20), (1, 6)]);
let mut fork = g.clone();
fork.observe(7, 100u64);
assert_eq!(fork.total_sum(), 159);
assert_eq!(g.total_sum(), 59);  // original unchanged
```

### Inspecting tree structure

`gnode_info()` returns a snapshot of any G-node by its arena handle.
Handles come from `g_root()`, `Node.gnode_id`, and `Node.parent` —
they remain valid until the backing node is evicted:

```rust
use torrust_mudlark::{Config, GState, GvGraph};
# let config = Config { split_threshold: 5u64, depth_create: 3, depth_evict: 6,
#     budget: None, alpha_relax: 0.75, bounded_eviction: true };
# let g = GvGraph::<u64, u64, 3>::from_observations(config,
#     [(3, 10u64), (3, 15), (6, 8), (5, 20), (1, 6)]);

// The root carries own=10 (frozen at bootstrap split) and sum=59 (everything)
let root = g.gnode_info(g.g_root()).unwrap();
assert_eq!((root.own, root.sum, root.state), (10, 59, GState::Internal));

// Find [0,4) via the V-Tree BFS and inspect its lineage
let left = g.layers()
    .find_map(|(_, n)| (n.start == 0 && n.end == 4).then_some(n))
    .unwrap();
assert_eq!(left.own, 15);   // frozen at catalytic split time
assert_eq!(left.sum, 21);   // 15 + children: [0,2) has own=6, [2,4) has own=0
assert!(g.is_ancestor_of(g.g_root(), left.gnode_id));  // O(1) — interval containment
assert!(left.parent == Some(g.g_root()));               // stable provenance
```

The `own` / `sum` split is the G-Tree's accounting identity
`sum = own + Σ children.sum` (G-I1). For frozen internal nodes, `own`
is the pre-subdivision measurement — the benchmark that children must
outgrow through competitive accumulation.

### Contour range decomposition

`contour_range()` decomposes a lattice-aligned interval into a minimal
basis set of G-nodes. Where `range_sum()` gives a pro-rated scalar
total, `contour_range()` returns the structural decomposition with
explicit energy accounting — basis elements, boundary thatching, and
the relationship between the decomposition's energy and the sum of
individual plateau energies:

```rust
# use torrust_mudlark::{Config, GvGraph};
# let config = Config { split_threshold: 5u64, depth_create: 3, depth_evict: 6,
#     budget: None, alpha_relax: 0.75, bounded_eviction: true };
# let g = GvGraph::<u64, u64, 3>::from_observations(config,
#     [(3, 10u64), (3, 15), (6, 8), (5, 20), (1, 6)]);

// Snap arbitrary coordinates to the endpoint lattice — (2,6) widens to (0,6)
let (lo, hi) = g.select_plateaus(2, 6).unwrap();

// Decompose into a basis set with energy fields
let cr = g.contour_range(lo, hi).unwrap();
assert_eq!(cr.basis.len(), 2);     // [0,4) and [4,6) — the minimal cover
assert_eq!(cr.energy, 41);         // Σ basis.sum: 21 + 20
assert_eq!(cr.plateau_count, 2);

// The algebraic identity always holds
assert_eq!(cr.energy, cr.plateau_energy + cr.cross_plateau_energy);

// exact_energy = range_sum over the same interval (pro-rated at boundaries)
assert_eq!(cr.exact_energy, g.range_sum(0u64..6));
```

The two energy measures (`energy` vs `exact_energy`) diverge at range
boundaries: `energy` accepts whole-node `.sum` values (possibly
thatching into territory outside the range), while `exact_energy`
pro-rates ancestor `.own` at partial overlaps. Neither is universally
larger — see §CR M-13.6 for the full analysis.

### Progressive reconstruction

`Pewei::reconstruct()` converts a truncated PEWEI back into a spatial
intensity function. Reconstruction is **additive**: each phase
transition's baseline persists as uniform background beneath finer
detail — it is supplemented, not overwritten. The output tiles the
domain with no gaps:

```rust
# use torrust_mudlark::{Config, GvGraph};
# let config = Config { split_threshold: 5u64, depth_create: 3, depth_evict: 6,
#     budget: None, alpha_relax: 0.75, bounded_eviction: true };
# let g = GvGraph::<u64, u64, 3>::from_observations(config,
#     [(3, 10u64), (3, 15), (6, 8), (5, 20), (1, 6)]);
let pewei = g.extract();
let all = pewei.layer_count().saturating_sub(1);

let full   = pewei.reconstruct(all);  // every layer visible
let coarse = pewei.reconstruct(2);    // first 3 layers only

// Both tile [0, 8) perfectly — no gaps, no overlaps
assert_eq!(full.first().unwrap().start, 0u64);
assert_eq!(full.last().unwrap().end, 8u64);
for w in full.windows(2) { assert_eq!(w[0].end, w[1].start); }

// Coarser truncation → fewer spans, energy still conserved
assert!(coarse.len() <= full.len());
```

Each additional layer refines the estimate by splitting coarse spans
into sub-regions with confirmed structure. Truncating earlier produces
a coarser but valid reconstruction — unresolved regions carry their
parent's total, uniformly distributed.

### Budget-constrained operation

Setting a `budget` activates the hard node-count ceiling. The tree
automatically evicts cold contour tips during `observe()`, and
`check_evictions()` sweeps manually. Eviction absorbs — mass flows
inward to parents, never disappears:

```rust
use torrust_mudlark::{Config, GvGraph};

let config = Config {
    split_threshold: 5u64, depth_create: 2, depth_evict: 3,
    budget: Some(10), alpha_relax: 0.75, bounded_eviction: true,
};
let mut g = GvGraph::<u64, u64, 3>::from_observations(config,
    [(3, 10u64), (3, 15), (6, 8), (5, 20), (1, 6)]);

// Budget caps node count — eviction absorbed mass into parents
assert!(g.node_count() <= 10);  // within budget; tips were evicted
assert_eq!(g.total_sum(), 59);  // energy conserved — mass flows to parents, never lost

// Manual sweep removes any remaining eligible candidates
let evicted = g.check_evictions();
assert_eq!(g.total_sum(), 59);  // still conserved
```

The tree contracts from its finest tips inward (Shield 3 — structural
immunity). The coarsest scales survive; fine-grained detail is
sacrificed first. When pressure eases, the depth gates relax and the
tree can re-expand where the V-Tree authorises it.

## Common patterns

Three recipes that arise in most applications of `GvGraph`.

### Periodic decay loop

The most common lifecycle: ingest a batch of observations, then apply a
uniform decay so stale regions fade between ticks.

```rust
# use torrust_mudlark::{Config, GvGraph};
# let config = Config { split_threshold: 5u64, depth_create: 3, depth_evict: 6,
#     budget: None, alpha_relax: 0.75, bounded_eviction: true };
# let mut g = GvGraph::<u64, u64, 3>::new(config);
# let batches: Vec<Vec<(u64, u64)>> = vec![vec![(3, 10)], vec![(5, 8)]];
for batch in &batches {
    for &(coord, delta) in batch {
        g.observe(coord, delta);
    }
    g.decay(g.g_root(), 0.95, 0.0); // 5 % uniform decay per tick
}
```

The `q` parameter controls depth-selectivity: `q = 0` decays every node
equally; raising `q` toward 1.0 decays deeper (finer) nodes more
aggressively.

### Plateau iteration for export

Walking the spatial contour to emit the learned shape — e.g. for
serialisation, logging, or feeding a downstream system:

```rust
# use torrust_mudlark::{Config, GvGraph};
# let config = Config { split_threshold: 5u64, depth_create: 3, depth_evict: 6,
#     budget: None, alpha_relax: 0.75, bounded_eviction: true };
# let mut g = GvGraph::<u64, u64, 3>::new(config);
# g.observe(3, 10u64);
# g.observe(3, 15u64);
# g.observe(6, 8u64);
# fn emit(_s: u64, _e: u64, _d: u32, _v: u64) {}
for plateau in g.plateaus().values() {
    emit(plateau.start, plateau.end, plateau.depth, plateau.sum);
}
```

`plateaus()` is 𝒪(1) with the default `dynamic-contour-tracking`
feature — it returns a borrowed `BTreeMap`, so the iteration itself is
𝒪(P) where P is the number of plateaus (contiguous depth-uniform
regions).

### Liveness check before subtree decay

When decaying an arbitrary subtree root — one obtained from an earlier
`layers()` walk or stored externally — the handle may have been
invalidated by a subsequent eviction. Guard with `gnode_info()`:

```rust
# use torrust_mudlark::{Config, GvGraph};
# let config = Config { split_threshold: 5u64, depth_create: 3, depth_evict: 6,
#     budget: None, alpha_relax: 0.75, bounded_eviction: true };
# let mut g = GvGraph::<u64, u64, 3>::new(config);
# g.observe(3, 10u64);
# g.observe(3, 15u64);
# g.observe(6, 8u64);
# let subtree_root = g.g_root(); // stand-in for a previously stored handle
if g.gnode_info(subtree_root).is_some() {
    g.decay(subtree_root, 0.90, 0.0);
}
```

`decay()` asserts that its root handle is live — calling it with a stale
handle will panic. The `gnode_info()` guard is cheap (an arena bounds
check) and makes the pattern safe in the presence of budget-driven
evictions.

## Architecture in brief

**G-Tree (Geometric Tree)** — a binary tree over dyadic intervals of
`[0, 2^N)`. Every materialised node stores the accumulated value for its
range. The G-Tree answers spatial queries and maintains a **contour** — the
set of leaf cells that currently tile the domain: a step function whose
depth at each coordinate reflects how finely the tree has resolved that
region. Observations always land on contour cells; refinement deepens
them, eviction coarsens them.

**V-Tree (Value Tree)** — a dynamic tournament bracket (branching factor
2 or 3) governed by the max-uncle constraint. High-intensity entries rise
near the root for efficient sampling; low-intensity entries consolidate
deeper.

**The two trees protect each other.** The V-Tree decides what earns
spatial resolution; the G-Tree's routing controls which entries receive
observations. Together they provide proportional sampling within factor
1.44 of Shannon entropy.

`GvGraph` is `Send + Sync`. Reads can proceed concurrently; writes take
`&mut self` and require external synchronisation (e.g. `RwLock`).

The architecture is a dynamic prefix-free code (G-Tree) governed by a
near-entropy-optimal significance ordering (V-Tree) —
[docs/theory.md](docs/theory.md) develops this foundation from first
principles.

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

| Type                    | Surface | Description                                                          |
| ----------------------- | ------- | -------------------------------------------------------------------- |
| `GvGraph<C, V, N>`      | 2       | The live dual-tree index                                             |
| `Config<V>`             | 2       | Index configuration (no `Default` — parameters are domain-dependent) |
| `Cell<C, V>`            | 1       | Snapshot of a terminal G-node                                        |
| `Span<C, V>`            | 1       | Dyadic interval + intensity                                          |
| `Node<C, V>`            | 1       | Snapshot of any G-node (own, sum, state)                             |
| `GState`                | 1       | Terminal / SemiInternal / Internal discriminant                      |
| `Pewei<C, V>`           | 1       | Full significance-ordered extraction                                 |
| `Layer<C, V>`           | 1       | One BFS depth-level of the V-Tree                                    |
| `Transition<C, V>`      | 1       | Phase-transition node — region with finer spatial structure          |
| `Terminal<C, V>`        | 1       | Leaf node — no further subdivision                                   |
| `Plateau<C, V>`         | 1       | One contiguous region of uniform contour depth                       |
| `BasisEdge<C>`          | 1       | `Ord`-providing newtype for plateau `BTreeMap` keys                  |
| `ContourRange<C, V>`    | 1       | Full contour-range decomposition of a lattice-aligned interval       |
| `ContourRangeEnergy<V>` | 1       | Energy-only result of a contour range query                          |
| `BasisElement<C, V>`    | 1       | One element of the minimal G-node cover of a contour range           |
| `GNodeId`               | 1       | Opaque arena handle                                                  |

### Key operations on `GvGraph`

| Method                             | Description                                                                                                                                           |
| ---------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| `new(config)`                      | Construct with a single root G-node covering `[0, 2^N)`                                                                                               |
| `from_observations(config, iter)`  | `new` + `extend` from `(coord, delta)` pairs                                                                                                          |
| `observe(coord, delta)`            | Route → accumulate → split → rebalance → evict                                                                                                        |
| `gnode_info(id)`                   | Snapshot of any G-node by arena handle                                                                                                                |
| `is_ancestor_of(ancestor, desc)`   | Test G-Tree ancestry between two node ids                                                                                                             |
| `get(coord)`                       | Infallible point query — 𝒪(d_geo), at most 𝒪(N)                                                                                                       |
| `plateaus()`                       | Spatial contour projection — 𝒪(1) borrowed (requires `V: Inspectable`)                                                                                |
| `range_sum(range)`                 | Recursive G-Tree range sum — 𝒪(N) (requires `V: Proratable`)                                                                                          |
| `contour_range(start, end)`        | Contour range decomposition — 𝒪(N) (requires `V: Proratable + Inspectable`)                                                                           |
| `contour_range_energy(start, end)` | Energy-only contour range query — 𝒪(N) (requires `V: Proratable + Inspectable`)                                                                       |
| `select_plateaus(lo, hi)`          | Plateau selection — snaps arbitrary coords to lattice endpoints — 𝒪(log P) (requires `V: Inspectable`)                                                |
| `sample(rng)`                      | Proportional sampling — 𝒪(1.44 H + 1) expected (requires `V: Weighable`)                                                                              |
| `extract()`                        | PEWEI extraction — 𝒪(n) where n = L + S (requires `V: Inspectable`)                                                                                   |
| `layers()`                         | Streaming V-Tree BFS yielding `(layer_index, Node)` — lazy, 𝒪(n) total where n = L (V-entries) + S (V-structural nodes) (requires `V: Inspectable`)   |
| `decay(root, attenuation, q)`      | Subband-adaptive temporal decay (requires `V: Attenuatable + Inspectable`)                                                                            |
| `check_evictions()`                | Manual eviction sweep (requires `V: Inspectable`)                                                                                                     |
| Accessors (11)                     | `node_count`, `terminal_count`, `budget`, `total_sum`, `config`, `g_root`, `depth_evict`, `depth_create`, `depth_buffer`, `headroom`, `soft_limit`    |

### Traits

#### Chemistry contracts

| Trait                    | Purpose                                                                                                                                                                                                                                                |
| ------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `Coordinate`             | Spatial coordinate — provided for `u8`..`u128`, `f32`, `f64`                                                                                                                                                                                           |
| `Accumulator`            | Core intensity/value type — non-negative, starting from zero. Provided for `u8`..`u128`, `f32`, `f64`                                                                                                                                                  |
| `Attenuatable`           | Multiplicative decay — required by `decay()` / `TemporalDecay`                                                                                                                                                                                         |
| `Weighable`              | Weight projection to `f64` — required by `sample()` / `WeightedSampler`                                                                                                                                                                                |
| `Proratable`             | Fractional subdivision — required by `range_sum()`, `contour_range()`, `contour_range_energy()`, and `Pewei::reconstruct()`                                                                                                                            |
| `Inspectable`            | `f64` projection — required by most operations including `observe()`; used for invariant checking, debug output, and plateau assertions (`observe`, `plateaus`, `extract`, `layers`, `contour_range`, `from_observations`, `check_evictions`, `decay`) |
| `Observation<V>`         | Accumulation rule (blanket impl for same-type; cross-type for floats)                                                                                                                                                                                  |
| `ScalableObservation<V>` | Multiplicative scaling for cross-type observation (e.g. `f64 → u16`); provided for downstream consumers — not used by the core engine                                                                                                                  |
| `Rng`                    | Minimal RNG — blanket impl for `rand_core::Rng` with `rand` feature                                                                                                                                                                                    |

**All built-in numeric types (`u8`–`u128`, `f32`, `f64`) implement
`Accumulator` and all four sub-traits (`Attenuatable`, `Weighable`,
`Proratable`, `Inspectable`).** Custom types need to implement only
the traits required by the operations they use.

**Why `Inspectable` is required by mutation operations like `observe()`.**
The trait impls on `GvGraph` (e.g. `SpatialWrite`) carry the union of
their methods' bounds. `SpatialRead` groups `get()` (needs only
`Accumulator`) with `plateaus()` (needs `Inspectable` for diagnostic
assertions in the plateau mirror), so the impl block must require
`Inspectable` for both — and `SpatialWrite: SpatialRead` inherits that
bound. This is a Rust language constraint, not a design choice. See
[Inherent vs. trait bounds](docs/api.md#inherent-vs-trait-bounds) in the
API doc for the full discussion.

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

The instruments partition the API by permission level — accepting
`&dyn SpatialRead` grants read-only spatial access without exposing
mutation, decay, or sampling.

## Features

| Feature                    | Default | Effect                                                            |
| -------------------------- | ------- | ----------------------------------------------------------------- |
| `dynamic-contour-tracking` | yes     | Live plateau mirror; `plateaus()` returns `Cow::Borrowed` in 𝒪(1) |
| `serde`                    | no      | `Serialize`/`Deserialize` on all Surface 1 snapshot types         |
| `rand`                     | yes     | Blanket `Rng` impl for all `rand_core::Rng` types                 |

## Performance

143 Criterion benchmarks across six families validate the theoretical
complexity claims. The table below shows representative latencies from
a single development machine (u64 configuration) — **absolute numbers
will vary by hardware; the complexity classes and relative ratios are
the meaningful guarantees:**

| Operation           | Typical latency | Complexity                              |
| ------------------- | --------------- | --------------------------------------- |
| `plateaus()`        | ~3 ns           | 𝒪(1) — reference borrow                 |
| `get(x)`            | ~15–20 ns       | 𝒪(d_geo), at most 𝒪(N) — G-Tree descent |
| `sample()` hotspot  | ~10 ns          | 𝒪(1.44 H + 1) — near-zero entropy       |
| `sample()` uniform  | ~130 ns         | 𝒪(1.44 H + 1) — max entropy             |
| `range_sum()`       | ~3–80 ns        | 𝒪(N) — width-dependent                  |
| `observe()` cold    | ~150–500 ns/obs | 𝒪(d_geo) amortised                      |
| `observe()` steady  | ~1.8 µs/obs     | Budget pressure + eviction              |
| `decay()` selective | ~6–8 µs         | Subband-adaptive, sub-linear scaling    |
| `check_evictions()` | ~58–59 µs       | Budget enforcement sweep                |

The entropy-adaptive sampling claim is confirmed: hotspot distributions
sample roughly **12× faster** than uniform. Fifteen adversarial patterns
(spine, fractal fill, gray code, pincer, and others) confirm
sub-quadratic throughput under worst-case access sequences.

See [docs/performance.md](docs/performance.md) for the full analysis
with scaling exponents, per-family breakdowns, f64 comparison, and a
claims scorecard.

## Limitations

- **1D only** — multi-dimensional domains require the Z-curve composition
  described in [docs/theory.md](docs/theory.md); it is not built in.
- **f64 mutation cost.** `<f64, f64, 52>` is 3–13× slower than
  `<u64, u64, 8>` due to the deeper G-Tree. Users choosing f64
  coordinates should set `N` judiciously.
- **Single-writer.** Mutating methods take `&mut self` — concurrent
  writes need external synchronisation.
- **No signed accumulators.** `i8`–`i128` are not supported; the
  Standard configuration requires non-negative values starting from zero.

## Documentation

> Full documentation including the formal spec, theory, and ADRs is on
> [GitHub](https://github.com/torrust/torrust-index/tree/main/packages/mudlark/docs).
> The relative links below work in the repository but may not resolve on
> crates.io.

| Document                                     | Contents                                                                     |
| -------------------------------------------- | ---------------------------------------------------------------------------- |
| [docs/idea.md](docs/idea.md)                 | Formal specification                                                         |
| [docs/api.md](docs/api.md)                   | Public API reference                                                         |
| [docs/architecture.md](docs/architecture.md) | Implementation architecture                                                  |
| [docs/performance.md](docs/performance.md)   | Benchmark results and performance analysis                                   |
| [docs/testing.md](docs/testing.md)           | Test suite breakdown and running guide                                       |
| [docs/theory.md](docs/theory.md)             | Coding-theoretic foundation — the architecture derived from first principles |
| [adr/](adr/)                                 | 38 architecture decision records                                             |
| [CHANGELOG.md](CHANGELOG.md)                 | Release history                                                              |

### Design records

Architectural decisions are recorded in [adr/](adr/). Notable entries:

- [ADR-M-032](adr/032-three-surface-model.md) — Three-surface visibility model
- [ADR-M-019](adr/019-sampling-semantics.md) — Sampling semantics
- [ADR-M-024](adr/024-decay-semantics.md) — Decay semantics
- [ADR-M-025](adr/025-public-api-surface.md) — Public API surface
- [ADR-M-020](adr/020-range-query-design.md) — Range query design
- [ADR-M-021](adr/021-pewei-output-representation.md) — PEWEI output representation
- [ADR-M-037](adr/037-contour-range-queries.md) — Contour range queries

### Cross-references

Doc-comments and documentation use `§`-prefixed tags to cite specific
sections of the design documents listed above. The prefix names the
document; the `M-` qualifier identifies the Mudlark package; the
number identifies the section.

| Tag           | Document                                               |
| ------------- | ------------------------------------------------------ |
| `§IDEA M-N`   | [docs/idea.md](docs/idea.md) §N — formal specification |
| `§CR M-N`     | [docs/idea.md](docs/idea.md) appendix §N — Contour Range          |
| `§PEWEI M-N`  | [docs/idea.md](docs/idea.md) appendix §N — PEWEI                  |
| `§THEORY M-N` | [docs/theory.md](docs/theory.md) §N                    |
| `§API M-N`    | [docs/api.md](docs/api.md) §N                          |
| `§ARCH M-N`   | [docs/architecture.md](docs/architecture.md) §N        |
| `§PERF M-N`   | [docs/performance.md](docs/performance.md) §N          |
| `§TEST M-N`   | [docs/testing.md](docs/testing.md) §N                  |
| `ADR-M-NNN`   | Architecture decision record in [adr/](adr/)           |

For example, `§IDEA M-9.3` in a doc-comment points to §9.3 of the
formal specification. `§§` denotes a range (e.g. `§§IDEA M-12.2–12.5`).
The full authoring conventions are in [AGENTS.md](../../AGENTS.md).

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

1315 tests across unit, integration, and doc-test suites (651 unit,
543 integration, 121 doc-tests).

The library emits structured diagnostics via `tracing` at `DEBUG` and
`TRACE` levels during rebalancing and eviction. These are silent by
default; attach a `tracing` subscriber to see them.
See [docs/testing.md](docs/testing.md) for the full breakdown,
feature-gated variation, and benchmark details.

### Benchmarks

Run all benchmarks:

```sh
cargo bench --package torrust-mudlark
```

Run a specific family:

```sh
cargo bench --package torrust-mudlark -- observe/
cargo bench --package torrust-mudlark -- spray/steady_state
```

Smoke-test (compile + single iteration, no timing):

```sh
cargo bench --package torrust-mudlark -- --test
```

(Criterion flag: compile and run once, no timing.)

Reports at `target/criterion/report/index.html`.

Save/compare baselines:

```sh
cargo bench --package torrust-mudlark -- --save-baseline my-baseline
cargo bench --package torrust-mudlark -- --baseline my-baseline
```

See [adr/035-benchmarking-framework.md](adr/035-benchmarking-framework.md) for details.

## License

Copyright (c) 2026 Torrust project contributors.

Licensed under AGPL-3.0-only with the
[Torrust-Mudlark Linking Exception](LINKING-EXCEPTION). In brief: you
may combine this library (unmodified) with your own code through the
documented public API ([Surfaces 1 and 2](docs/api.md)) without AGPL obligations on
your code. Modifications or use of internal APIs (Surface 3) are
subject to the full AGPL.

The full license text is in the repository root
([LICENSE](../../LICENSE)) and at
<https://www.gnu.org/licenses/agpl-3.0.html>. See
[LINKING-EXCEPTION](LINKING-EXCEPTION) for the complete exception text
and the Legacy Exception caveat.
