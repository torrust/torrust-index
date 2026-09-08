# Spectral Sentinel · `guide:sentinel:overview`

Hierarchical online subspace anomaly detection for positionally structured observation streams.

Spectral Sentinel combines Mudlark's adaptive spatial index with low-rank statistical trackers. Mudlark ranks spatial entries by observation volume; Spectral Sentinel selects significant V-Tree entries, closes them under G-tree ancestry, and scores incoming batches against learned subspace models for that selected structure. Reports contain measurements only: scores, baselines, drift accumulators, maturity, structural summaries, and health snapshots.

**Spectral Sentinel measures; the host decides.**

See the architecture in brief (`sec:sentinel:readme-architecture-in-brief`) below for the conceptual model, or jump straight to the quick start (`sec:sentinel:readme-quick-start`) for code.

## Choose Spectral Sentinel · `sec:sentinel:readme-choose-spectral-sentinel`

Use Spectral Sentinel when your stream has **hierarchical positional structure**: leading bits define coarse membership and successive bits refine it. IPv6-like address spaces, network-prefix encodings, and other dyadic coordinate domains are natural fits.

If the values are pseudo-random, the model has no meaningful positional structure to learn. Cryptographic hashes, UUIDs, random nonces, and uniform identifiers will still be processed, but the measurements will not be useful. If the distribution is static and known in advance, a fixed index or offline model will usually be simpler. If you only need adaptive spatial aggregation or proportional sampling, use [`torrust-mudlark`](../mudlark/README.md) directly.

Spectral Sentinel's advantage is online multi-scale measurement: it lets Mudlark adapt spatial structure as traffic shifts, models suffix-bit structure at competitively selected regions, and emits raw statistical readouts without embedding host policy.

## Stability · `sec:sentinel:readme-stability`

This crate follows [Semantic Versioning](https://semver.org/). The public API surface documented in [docs/api.md](docs/api.md) — every type, trait, and method re-exported from the crate root — is covered by semver guarantees from 1.0.0 onwards.

Internal machinery (`pub(crate)` modules, hidden test affordances, EWMA state, tracker internals, staging, and SVD plumbing) is not part of the public API and may change in any release.

**MSRV:** 1.89 (workspace setting under ADR-T-011)

## Installation · `sec:sentinel:readme-installation`

Add the crate to your project:

```sh
cargo add torrust-sentinel
```

Or add it manually to your `Cargo.toml`:

```toml
[dependencies]
torrust-sentinel = "0.1"
```

The base build has no default feature flags. The `serde` feature is opt-in and enables serialisation for configuration, SVD strategy, and report snapshot types:

```toml
[dependencies]
torrust-sentinel = { version = "0.1", features = ["serde"] }
```

## Quick start · `sec:sentinel:readme-quick-start`

> Every claim below is asserted with full invariant checks in the [pedagogy integration test](tests/pedagogy.rs). For the inspection surface, see the [advanced pedagogy test](tests/pedagogy_advanced.rs).

### Create the sentinel · `sec:sentinel:readme-create-the-sentinel`

`Sentinel128` is the convenience alias for `SpectralSentinel<u128, u64, 128>`: a full 128-bit coordinate domain with `u64` volume counters. New trackers are automatically warmed with synthetic noise before real observations are scored.

```rust
use torrust_sentinel::{NoiseSchedule, Sentinel128, SentinelConfig};

let config = SentinelConfig::<u64> {
    analysis_k: 8,                    // small analysis budget for a demo
    split_threshold: 4,               // split quickly so examples show structure
    noise_schedule: NoiseSchedule::Explicit(vec![4]),
    noise_batch_size: 4,
    noise_seed: Some(2026),
    ..SentinelConfig::default()
};

let mut sentinel = Sentinel128::new(config).unwrap();
assert_eq!(sentinel.lifetime_observations(), 0);
```

The default configuration is tuned for longer-lived streams. The compact noise schedule above keeps examples and doctests fast; production callers should choose warm-up settings from the recommendations in [docs/algorithm.md](docs/algorithm.md) Appendix A.

### Feed data — spatial structure adapts · `sec:sentinel:readme-feed-data-spatial-structure-adapts`

Every raw value increments Mudlark's spatial substrate by exactly one unit. Scores never feed back into spatial importance, so concentrated traffic can reshape the contour but anomalous-looking scores cannot promote themselves.

```rust
# use torrust_sentinel::{NoiseSchedule, Sentinel128, SentinelConfig};
# let config = SentinelConfig::<u64> {
#     analysis_k: 8, split_threshold: 4,
#     noise_schedule: NoiseSchedule::Explicit(vec![4]), noise_batch_size: 4,
#     noise_seed: Some(2026), ..SentinelConfig::default()
# };
# let mut sentinel = Sentinel128::new(config).unwrap();
let values: Vec<u128> = vec![
    0xF000_0000_0000_0000_0000_0000_0000_0001,
    0xF000_0000_0000_0000_0000_0000_0000_0002,
    0xF000_0000_0000_0000_0000_0000_0000_0003,
    0x1000_0000_0000_0000_0000_0000_0000_0004,
];

let report = sentinel.ingest(&values);
assert_eq!(sentinel.lifetime_observations(), values.len() as u64);
assert!(report.ancestor_reports.iter().any(|cell| cell.depth == 0));
```

A non-empty batch always reports the root as an ancestor context. As the G-V Graph splits, `cell_reports` contains competitive analysis cells and `ancestor_reports` contains the multi-scale chain back to the root.

### Read measurements back · `sec:sentinel:readme-read-measurements-back`

Each cell report carries the same four raw scoring axes. Higher values mean greater departure from the learned baseline; Spectral Sentinel does not turn those values into severity levels or actions.

```rust
# use torrust_sentinel::{NoiseSchedule, Sentinel128, SentinelConfig};
# let config = SentinelConfig::<u64> {
#     analysis_k: 8, split_threshold: 4,
#     noise_schedule: NoiseSchedule::Explicit(vec![4]), noise_batch_size: 4,
#     noise_seed: Some(2026), ..SentinelConfig::default()
# };
# let mut sentinel = Sentinel128::new(config).unwrap();
# let values: Vec<u128> = vec![
#     0xF000_0000_0000_0000_0000_0000_0000_0001,
#     0xF000_0000_0000_0000_0000_0000_0000_0002,
#     0xF000_0000_0000_0000_0000_0000_0000_0003,
#     0x1000_0000_0000_0000_0000_0000_0000_0004,
# ];
# let report = sentinel.ingest(&values);
for cell in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
    let scores = &cell.scores;
    println!(
        "depth {} [{:#034x}, {:#034x}) novelty z={:.2} displacement z={:.2}",
        cell.depth,
        cell.start,
        cell.end,
        scores.novelty.max_z_score,
        scores.displacement.max_z_score,
    );
}
```

The host decides how to interpret the measurements. A dashboard might plot z-scores and maturity, a forensic workflow might enable per-sample payloads, and an automated system might compare score distributions against a domain-specific policy.

### Apply host-controlled decay · `sec:sentinel:readme-apply-host-controlled-decay`

Temporal policy is external. Spectral Sentinel never decays the graph on its own; the host decides when old spatial importance should fade.

```rust
# use torrust_sentinel::{NoiseSchedule, Sentinel128, SentinelConfig};
# let config = SentinelConfig::<u64> {
#     analysis_k: 8, split_threshold: 4,
#     noise_schedule: NoiseSchedule::Explicit(vec![4]), noise_batch_size: 4,
#     noise_seed: Some(2026), ..SentinelConfig::default()
# };
# let mut sentinel = Sentinel128::new(config).unwrap();
# let values: Vec<u128> = vec![
#     0xF000_0000_0000_0000_0000_0000_0000_0001,
#     0xF000_0000_0000_0000_0000_0000_0000_0002,
#     0xF000_0000_0000_0000_0000_0000_0000_0003,
#     0x1000_0000_0000_0000_0000_0000_0000_0004,
# ];
# let _report = sentinel.ingest(&values);
let before = sentinel.graph().total_sum();
sentinel.decay(0.5, 0.0); // uniform 50% attenuation across the graph
let after = sentinel.graph().total_sum();
assert!(after <= before);
```

Decay changes spatial importance and future competitive selection. It does not mutate tracker subspaces, baselines, CUSUM state, or lifetime observation counts.

## Advanced usage · `sec:sentinel:readme-advanced-usage`

The quick start covers construction, ingestion, reporting, and decay. This section covers inspection payloads, alternative domains, configuration, and operational patterns.

### Inspect trackers directly · `sec:sentinel:readme-inspect-trackers-directly`

`cell_gnodes()` lists the live analysis trackers. Use `inspect_cell()` when a host needs a tracker snapshot outside the batch report lifecycle: current rank, energy ratio, maturity, geometry flags, and baseline snapshots.

```rust
# use torrust_sentinel::{NoiseSchedule, Sentinel128, SentinelConfig};
# let config = SentinelConfig::<u64> {
#     analysis_k: 8, split_threshold: 4,
#     noise_schedule: NoiseSchedule::Explicit(vec![4]), noise_batch_size: 4,
#     noise_seed: Some(2026), ..SentinelConfig::default()
# };
# let mut sentinel = Sentinel128::new(config).unwrap();
# let _report = sentinel.ingest(&[1_u128, 2, 3, 4]);
let root = sentinel
    .cell_gnodes()
    .into_iter()
    .find_map(|id| sentinel.inspect_cell(id).filter(|cell| cell.depth == 0))
    .unwrap();

assert_eq!(root.analysis_width, 128);
assert!(root.maturity.total_observations() > 0);
```

G-node handles come from the analysis set, reports, inspections, and the underlying `graph()` view. Handles can become stale after later graph restructuring, so guard externally stored handles with a fresh graph lookup before targeted subtree decay.

### Enable per-sample payloads · `sec:sentinel:readme-enable-per-sample-payloads`

Set `per_sample_scores` when the host needs observation-level detail. The batch-level summaries remain available; each cell report also contains one `SampleScore` per routed observation.

```rust
# use torrust_sentinel::{NoiseSchedule, Sentinel128, SentinelConfig};
let config = SentinelConfig::<u64> {
    per_sample_scores: true,
    analysis_k: 8,
    split_threshold: 4,
    noise_schedule: NoiseSchedule::Explicit(vec![4]),
    noise_batch_size: 4,
    noise_seed: Some(2026),
    ..SentinelConfig::default()
};
let mut sentinel = Sentinel128::new(config).unwrap();
let report = sentinel.ingest(&[1_u128, 2, 3, 4]);

let root = report.ancestor_reports.iter().find(|cell| cell.depth == 0).unwrap();
let samples = root.per_sample.as_ref().unwrap();
assert_eq!(samples.len(), root.sample_count);
```

Per-sample payloads are useful for forensics and expensive for large batches. Keep them disabled in hot paths unless the host needs that resolution.

### Use 64-bit or custom-width domains · `sec:sentinel:readme-use-64-bit-or-custom-width-domains`

`Sentinel64` is the convenience alias for `SpectralSentinel<u64, u64,
64>`. The generic engine can also be instantiated with another bit-width when the coordinate type can supply centred bits over that domain.

```rust
use torrust_sentinel::{NoiseSchedule, Sentinel64, SentinelConfig};

let config = SentinelConfig::<u64> {
    noise_schedule: NoiseSchedule::Explicit(vec![2]),
    noise_batch_size: 4,
    noise_seed: Some(7),
    ..SentinelConfig::default()
};

let mut sentinel = Sentinel64::new(config).unwrap();
let report = sentinel.ingest(&[0xF000_0000_0000_0001_u64, 0xF000_0000_0000_0002]);
assert!(report.health.active_trackers > 0);
```

```rust
use torrust_sentinel::{NoiseSchedule, SentinelConfig, SpectralSentinel};

type Sentinel16 = SpectralSentinel<u64, u64, 16>;

let config = SentinelConfig::<u64> {
    noise_schedule: NoiseSchedule::Explicit(vec![2]),
    noise_batch_size: 4,
    noise_seed: Some(11),
    ..SentinelConfig::default()
};

let mut sentinel = Sentinel16::new(config).unwrap();
let report = sentinel.ingest(&[0xF001_u64, 0xF002, 0x1003]);
assert!(report.contour.total_importance >= 3.0);
```

The domain is `[0, 2^N)`. Hosts are responsible for ensuring coordinates fit the chosen width and carry meaningful positional structure.

### Periodic ingest and decay loop · `sec:sentinel:readme-periodic-ingest-and-decay-loop`

A common lifecycle is: ingest a batch, read measurements, then apply a host-selected decay so stale spatial importance fades between ticks.

```rust
# use torrust_sentinel::{NoiseSchedule, Sentinel128, SentinelConfig};
# let config = SentinelConfig::<u64> {
#     analysis_k: 8, split_threshold: 4,
#     noise_schedule: NoiseSchedule::Explicit(vec![4]), noise_batch_size: 4,
#     noise_seed: Some(2026), ..SentinelConfig::default()
# };
# let mut sentinel = Sentinel128::new(config).unwrap();
let batches: Vec<Vec<u128>> = vec![
    vec![0xA000_0000_0000_0000_0000_0000_0000_0001, 0xA000_0000_0000_0000_0000_0000_0000_0002],
    vec![0xB000_0000_0000_0000_0000_0000_0000_0001, 0xA000_0000_0000_0000_0000_0000_0000_0003],
];

for batch in &batches {
    let report = sentinel.ingest(batch);
    let _max_novelty_z = report
        .cell_reports
        .iter()
        .chain(report.ancestor_reports.iter())
        .map(|cell| cell.scores.novelty.max_z_score)
        .fold(0.0_f64, f64::max);

    sentinel.decay(0.95, 0.0);
}
```

The `q` parameter controls depth selectivity: `q = 0.0` decays every spatial depth equally; raising `q` toward `1.0` makes fine structure fade more aggressively than coarse structure.

### Treat warm trackers as preliminary · `sec:sentinel:readme-treat-warm-trackers-as-preliminary`

Every tracker reports a `noise_influence` value. It decays as real observations replace synthetic warm-up as the basis of the learned baseline. Hosts can use it to suppress early conclusions without hiding the raw measurements.

```rust
# use torrust_sentinel::{NoiseSchedule, Sentinel128, SentinelConfig};
# let config = SentinelConfig::<u64> {
#     analysis_k: 8, split_threshold: 4,
#     noise_schedule: NoiseSchedule::Explicit(vec![4]), noise_batch_size: 4,
#     noise_seed: Some(2026), ..SentinelConfig::default()
# };
# let mut sentinel = Sentinel128::new(config).unwrap();
# let report = sentinel.ingest(&[1_u128, 2, 3, 4]);
for cell in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
    if cell.maturity.noise_influence > 0.5 {
        continue;
    }

    let _usable_mean_z = cell.scores.novelty.mean_z_score;
}
```

This is a host policy choice. Spectral Sentinel exposes maturity; it does not suppress or reinterpret scores on the host's behalf.

## Configuration · `sec:sentinel:readme-configuration`

`SentinelConfig` validates every invariant up front. Invalid parameters return all detected errors at once; warnings report valid combinations that are likely to produce poor warm-up quality.

```rust
use torrust_sentinel::{NoiseSchedule, SentinelConfig, SvdStrategy};

let config = SentinelConfig::<u64> {
    max_rank: 16,                 // rank ceiling per tracker
    forgetting_factor: 0.99,      // fast EWMA memory
    rank_update_interval: 100,    // tracker steps between rank checks
    energy_threshold: 0.90,       // variance target for rank adaptation
    eps: 1e-6,                    // numerical stability
    cusum_slow_decay: 0.999,      // slow baseline for per-cell drift
    cusum_coord_slow_decay: 0.999,// slow baseline for coordination drift
    cusum_allowance_sigmas: 0.5,  // CUSUM noise allowance
    clip_sigmas: 3.0,             // upper-tail baseline clip width
    clip_pressure_decay: 0.95,    // clip-pressure EWMA memory
    per_sample_scores: false,     // include per-observation detail
    analysis_k: 1024,             // competitive analysis budget
    analysis_depth_cutoff: 6,     // V-Tree eligibility cutoff
    split_threshold: 100,         // G-V Graph split sensitivity
    d_create: 3,                  // max V-depth for new splits
    d_evict: 6,                   // min V-depth for eviction
    budget: 100_000,              // hard live G-node ceiling
    noise_schedule: NoiseSchedule::default(),
    noise_batch_size: 16,
    noise_seed: Some(42),
    background_warming: false,
    svd_strategy: SvdStrategy::Brand,
};

config.validate().unwrap();
let _warnings = config.warnings();
```

Key tuning knobs:

| Parameter | Effect |
| --------- | ------ |
| `analysis_k` | Resource ceiling for competitive analysis cells. Total live cell trackers are bounded by the competitive cells plus their shared ancestors. |
| `analysis_depth_cutoff` | V-Tree depth eligibility. Lower values restrict analysis to entries that have risen closer to the tournament root. |
| `forgetting_factor` | Fast statistical memory. Lower values adapt faster; higher values remember longer. |
| `max_rank` | Model expressiveness ceiling. Higher values can model richer suffix structure at greater memory and SVD cost. |
| `energy_threshold` | Variance target for automatic rank adaptation. Higher values tend to grow rank. |
| `split_threshold` | Spatial split sensitivity. Lower values refine the G-V Graph faster. |
| `d_create` / `d_evict` | Mudlark depth gates. They control tree growth, eviction eligibility, and budget behaviour. |
| `budget` | Hard ceiling on live G-nodes in the spatial substrate. |
| `noise_schedule` | Depth-tiered warm-up schedule: geometric by default, or explicit per-depth counts. |
| `background_warming` | Moves new-cell warm-up off the `ingest()` hot path at the cost of delayed participation. |
| `svd_strategy` | `Brand` incremental SVD by default, with `Naive` available as a dense baseline. |

## Automatic warm-up · `sec:sentinel:readme-automatic-warm-up`

Every newly created tracker is warmed with synthetic noise before it receives real observations. There is no manual noise-injection API; the Spectral Sentinel owns the warm-up lifecycle.

| Tracker | Warm-up behaviour |
| ------- | ----------------- |
| Root tracker | Warmed during `SpectralSentinel::new()`. |
| New analysis cells | Enqueued into the staging area and warmed before promotion. |
| Coordination contexts | Warmed when cross-cell coordination first activates. |

`NoiseSchedule` supports two forms:

| Variant | Meaning |
| ------- | ------- |
| `Geometric { root, decay, min }` | `rounds(depth) = max(min, root × decay^depth)`. This is the default schedule. |
| `Explicit(Vec<u32>)` | Per-depth round counts. The last entry repeats for deeper cells; an empty vector disables noise. |

Synchronous warm-up (`background_warming: false`) is deterministic and used heavily by tests. Background warm-up (`true`) is intended for production paths where cell creation should not introduce a latency spike.

## Architecture in brief · `sec:sentinel:readme-architecture-in-brief`

Spectral Sentinel implements a three-layer feed-forward architecture backed by the Mudlark G-V Graph.

```text
Layer 1: G-V Graph
         Adaptive spatial partitioning of [0, 2^N)
         Pure volume tracking: Δ = 1 per observation
         Competitive ranking by observation volume
              │
              │ V-Tree depth ≤ cutoff → top-K selection
              ▼
Layer 2: Analysis Selector
         Selects competitive cells
         Closes selection under G-tree ancestry
              │
              │ suffix bit vectors at each ancestor depth
              ▼
Layer 3: Analysis Engine
         Per-cell low-rank subspace trackers
         Hierarchical coordination over cross-cell score patterns
              │
              ▼
         BatchReport<C> → host
```

### Spatial substrate · `sec:sentinel:readme-spatial-substrate`

Spectral Sentinel owns a `GvGraph<C, V, N>`. During `ingest()`, each raw coordinate is observed by the graph with `Δ = 1`; that is the feed-forward invariant from [ADR-S-002](adr/002-feed-forward-invariant.md). Anomaly scores flow outward to reports and never back into Mudlark's importance signal.

The host may call `decay()` or `decay_subtree()` to reshape spatial importance over time. Decay affects the graph's competitive ranking, not the statistical state inside trackers.

### Analysis selector · `sec:sentinel:readme-analysis-selector`

After each observation pass, `AnalysisSet` selects the top competitive V-Tree entries within `analysis_depth_cutoff`, takes at most `analysis_k`, and closes the result under G-tree ancestry. The root is permanent context, never a competitive target.

The selected cells form the investment set: cells that own trackers or are warming toward ownership. Online members produce reports; warming members are visible in health and summary counts.

### Analysis engine · `sec:sentinel:readme-analysis-engine`

Each analysis cell owns a `SubspaceTracker` over the suffix bits `[d, N)` where `d` is the G-tree depth. The tracker processes batches in a strict score-before-evolve order:

1. Score the batch against the prior model.
2. Evolve the subspace with streaming thin SVD.
3. Evolve latent mean, variance, and second-moment state.
4. Update fast baselines, slow CUSUM references, and clip pressure.
5. Adapt rank toward the configured energy threshold.

Coordination trackers sit at internal G-tree contexts and model patterns among competitive-cell score vectors. They detect cross-cell score shapes that no single cell needs to treat as special.

## Scoring axes · `sec:sentinel:readme-scoring-axes`

All axes share the same polarity: higher values indicate greater anomalous departure from the learned baseline.

| Axis | Measures | Range |
| ---- | -------- | ----- |
| `novelty` | Residual energy outside the learned subspace | `[0, ∞)` |
| `displacement` | Distance from the cell's latent centroid | `[0, 1)` |
| `surprise` | Per-dimension magnitude deviation | `[0, ∞)` |
| `coherence` | Unusual pairwise co-activation patterns | `[0, ∞)` |

Together they decompose the covariance structure of centred suffix-bit vectors without assembling or inverting a dense covariance matrix. `ScoreDistribution` reports min, max, mean, z-scores, fast baseline, slow CUSUM reference, accumulator state, and clip pressure for each axis.

At the coordination tier the same four axes have second-order meaning: novelty measures unseen cross-cell score patterns, displacement measures a shifted score landscape, surprise measures a system-wide axis elevation, and coherence measures unusual combinations of axis elevation.

## Report structure · `sec:sentinel:readme-report-structure`

`ingest()` returns a `BatchReport<C>`:

```text
BatchReport
├── cell_reports: [CellReport]                  competitive cells
├── ancestor_reports: [CellReport]              ancestor-only cells, including root
├── coordination_reports: [CoordinationReport]  cross-cell score-pattern models
├── contour: ContourSnapshot                    spatial contour summary
├── health: HealthReport                        operational health snapshot
├── analysis_set_summary: AnalysisSetSummary    investment and producing-set summary
└── oldest_observation_age_micros: Option<u64>  age of the batch's oldest observation at emission
```

The age is a duration on the sentinel's own monotonic clock — stamped as the batch arrives, read off as the report is assembled — so it carries no wall-clock instant and no cross-machine skew. A batch arrives whole, so the one figure bounds every observation in it. It is absent both for a batch that carried no observations and for a payload written before the field existed, because neither holds a measurement and a zero would claim one. How long the host held the observations before handing them over is not included and is deliberately unmeasured.

A `CellReport` includes interval bounds, depth, suffix width, sample count, rank, energy ratio, top singular value, four-axis scores, tracker maturity, scoring geometry, and optional per-sample scores.

A `CoordinationReport` includes the same tracker facts for a coordination context, plus the number of competitive cells that contributed and optional per-member score records.

## Public API surface · `sec:sentinel:readme-public-api-surface`

Spectral Sentinel uses the same three-surface visibility model as Mudlark. Public symbols are re-exported flat from the crate root; modules remain private, so downstream code has one canonical import path.

| Surface | Name | What it exposes |
| ------- | ---- | --------------- |
| 1 | **Readouts** | Detached report and snapshot types users hold after an observation cycle. |
| 2 | **Engine** | Opaque operational types users configure and drive. |
| 3 | **Internals** | `pub(crate)` implementation machinery; not part of the API. |

### Key types · `sec:sentinel:readme-key-types`

`GNodeId` is listed with the engine surface because hosts pass it back to target subtree decay. Reports also carry it as a readout identifier.

| Type | Surface | Description |
| ---- | ------- | ----------- |
| `SpectralSentinel<C, V, N>` | 2 | Live Spectral Sentinel engine generic over coordinate, accumulator, and bit-width. |
| `Sentinel128` | 2 | Alias for `SpectralSentinel<u128, u64, 128>`. |
| `Sentinel64` | 2 | Alias for `SpectralSentinel<u64, u64, 64>`. |
| `SentinelConfig<V>` | 2 | Measurement, resource, warm-up, and Mudlark substrate configuration. |
| `NoiseSchedule` | 2 | Depth-tiered tracker warm-up schedule. |
| `SvdStrategy` | 2 | `Brand` incremental SVD or `Naive` dense SVD. |
| `CentredBitSource` | 2 | Coordinate-to-centred-bits capability used by the engine. |
| `GNodeId` | 2 | Re-exported Mudlark arena handle for reports and targeted subtree decay. |
| `BatchReport<C>` | 1 | Complete output from one `ingest()` call. |
| `CellReport<C>` | 1 | Per-cell score, rank, maturity, and geometry readout. |
| `CoordinationReport<C>` | 1 | Cross-cell score-pattern readout. |
| `AnalysisSet<C, V>` | 1 | Current competitive targets plus G-tree ancestors. |
| `AnalysisEntry<C, V>` | 1 | One selected cell in the analysis set. |
| `HealthReport` | 1 | Tracker, coordination, maturity, geometry, and clip-pressure health. |
| `CellInspection<C>` | 1 | Direct snapshot of one live cell tracker. |
| `AnomalyScores` | 1 | Four-axis score distributions. |
| `ScoreDistribution` | 1 | Raw score summary plus z-scores, baseline, CUSUM, and clip pressure. |
| `TrackerMaturity` | 1 | Real/noise observation counts and noise influence. |
| `ScoringGeometry` | 1 | Structural flags for novelty saturation and coherence activity. |

### Key operations · `sec:sentinel:readme-key-operations`

| Method | Description |
| ------ | ----------- |
| `SpectralSentinel::new(config)` | Validate configuration, create graph, create and warm the root tracker. |
| `ingest(&[C])` | Process one batch and return `BatchReport<C>`. |
| `health()` | Snapshot active trackers, ranks, maturity, geometry, coordination, and clip pressure. |
| `cell_gnodes()` | List live analysis-cell handles. |
| `inspect_cell(gnode)` | Snapshot a specific cell tracker if it is currently live. |
| `cells_tracked()` | Count live cell trackers in the analysis set. |
| `lifetime_observations()` | Count real observations processed since construction or reset. |
| `degenerate_cells_skipped()` | Count cells excluded because suffix width is too small for tracking. |
| `config()` | Read-only access to the validated configuration. |
| `analysis_set()` | Read-only access to the current analysis set. |
| `graph()` | Read-only access to the Mudlark spatial substrate. |
| `decay(attenuation, q)` | Apply host-controlled temporal decay to the full graph. |
| `decay_subtree(gnode, attenuation, q)` | Apply host-controlled temporal decay to one G-subtree. |
| `reset()` | Clear learned state and recreate the fresh warmed root. |

## Features · `sec:sentinel:readme-features`

| Feature | Default | Effect |
| ------- | ------- | ------ |
| `serde` | no | `Serialize`/`Deserialize` for configuration, SVD strategy, and report snapshot types. Also enables `torrust-mudlark/serde`. |

## Resource model · `sec:sentinel:readme-resource-model`

`analysis_k` is the primary analysis-tier budget. The competitive set is bounded by that value, and the full tracker set is the competitive cells plus the shared ancestor chain back to the root. The default configuration keeps this bounded by roughly `2 × analysis_k` in the common Steiner-tree case.

The Mudlark `budget` field is the hard ceiling on live spatial nodes. `split_threshold`, `d_create`, and `d_evict` determine how quickly the spatial substrate refines and how it contracts under pressure.

Warm-up work is usually the largest cold-start cost. Use `background_warming: true` when avoiding `ingest()` latency spikes matters more than immediate participation by brand-new cells.

Criterion benchmarks live in [benches/sentinel.rs](benches/sentinel.rs):

```sh
cargo bench -p torrust-sentinel
```

## Limitations · `sec:sentinel:readme-limitations`

- **Structured coordinates required.** Pseudo-random bit strings do not contain learnable suffix structure for this model.
- **One-dimensional domain.** Spectral Sentinel expects one-dimensional coordinates. Multi-dimensional data needs an external encoding, such as a space-filling curve composition.
- **Measurements only.** Spectral Sentinel has no policy thresholds, threat levels, labels, or actions. Hosts must interpret reports in context.
- **Single-writer mutation.** Mutating methods take `&mut self`; concurrent writers need external synchronisation.
- **Warm-up tradeoff.** Synchronous warm-up is deterministic but can add latency when cells are created. Background warm-up avoids that spike but delays scoring for new cells.
- **Timing equalisation out of scope.** Deferred warm-up is implemented; timing-protection padding and equalisation from §ALGO S-18.5 are outside this crate.

## Documentation · `sec:sentinel:readme-documentation`

> The relative links below work in the repository but may not resolve on crates.io.

| Document | Contents |
| -------- | -------- |
| [docs/algorithm.md](docs/algorithm.md) | Formal algorithm specification. |
| [docs/api.md](docs/api.md) | Public API reference. |
| [docs/implementation.md](docs/implementation.md) | Source layout, implementation guide, and test-suite map. |
| [adr/](adr/) | Architecture decision records. |
| [tests/pedagogy.rs](tests/pedagogy.rs) | End-to-end narrative test for construction, ingest, scoring, decay, and reset. |
| [tests/pedagogy_advanced.rs](tests/pedagogy_advanced.rs) | Inspection-oriented narrative test for readouts, geometry, coordination, and temporal separation. |

### Design records · `sec:sentinel:readme-design-records`

Architectural decisions are recorded in [adr/](adr/). Notable entries:

- [ADR-S-001](adr/001-measures-not-opinions.md) — Measures, not opinions
- [ADR-S-002](adr/002-feed-forward-invariant.md) — Feed-forward invariant
- [ADR-S-007](adr/007-automatic-noise-injection.md) — Automatic noise injection
- [ADR-S-015](adr/015-cell-creation-performance.md) — Cell creation performance
- [ADR-S-016](adr/016-brand-incremental-svd.md) — Brand incremental SVD
- [ADR-S-017](adr/017-deferred-cell-warm-up.md) — Deferred cell warm-up
- [ADR-S-018](adr/018-generic-domain-parameters.md) — Generic domain parameters
- [ADR-S-019](adr/019-investment-set-terminology-and-reporting.md) — Investment-set terminology and reporting
- [ADR-S-020](adr/020-clip-pressure-ewma.md) — Clip-pressure EWMA
- [ADR-S-021](adr/021-ewma-mean-centred-variance.md) — EWMA mean-centred variance

### Cross-references · `sec:sentinel:readme-cross-references`

Doc-comments and documentation use `§`-prefixed tags to cite specific sections of the design documents. The `S-` qualifier identifies this Sentinel package; ADRs use their own `ADR-S-NNN` form.

| Tag | Document |
| --- | -------- |
| `§ALGO S-N` | [docs/algorithm.md](docs/algorithm.md) §N |
| `§API S-N` | [docs/api.md](docs/api.md) §N |
| `§IMPL S-N` | [docs/implementation.md](docs/implementation.md) §N |
| `ADR-S-NNN` | Architecture decision record in [adr/](adr/) |

For example, `§ALGO S-8.2` refers to analysis-set ancestry closure in the algorithm specification. `§§` denotes a range, such as `§§ALGO S-4.2–4.5`. The full authoring conventions are in [AGENTS.md](../../AGENTS.md).

## The area register · `sec:sentinel:area-register`

Every claim this package mints names an area, and this register says what each area is for. The requirement it answers is that a package minting claims carries one register in its own prose, an entry per area, whose head prose states the stake — what is lost if claims of this area fail.

The stake is the one thing about an area that no census can compute and no individual claim states, because a claim says what the system does rather than why anyone should care that it does it. The vocabulary was not fixed in advance; it is what the statements turned out to want, censused after they were written and curated into the entries below. An unregistered area is a report line rather than a finding, so a claim minted in a new one is admitted and counted, and this register is what catches up.

Three of the entries carry a caveat the register cannot settle. The convergence area is a mixture: about half its claims are about the engine, and the other half define the instruments the convergence suite measures with, which are claims about the yardstick rather than about the sentinel. The analysis-width arithmetic is minted four times over — at the model, at the coordinate alias, at the report surface, and as a standing guarantee — which is defensible, each stating the fact at a different boundary, and still leaves a reader asking which is the source. And the width area is a misnomer: both its claims are about domain parametrisation rather than about width, and the entry below is written to the claims rather than to the name.

**Section (ancestry)** · `sec:sentinel:area-ancestry`

If this fails: a disturbance can be noticed but not located. Delivery is by containment rather than ownership — nothing is consumed by the deepest cell that matched — so an ancestor credited with less than everything beneath it holds a baseline for a volume no cell ever saw. A chain that narrows toward the root, or that skips the intervening depths, stops being a sequence of scales at all. What goes with it is the comparison itself: the root grades an anomaly by how much of the domain it reaches, and nothing else tells a host whether an incident is local or system-wide.

**Section (bits)** · `sec:sentinel:area-bits`

If this fails: every measurement above this boundary describes a value nobody sent. A bit enters as minus or plus a half, most significant first, because that centring is what the tracker assumes and that ordering is what lets a cell take its working view by dropping the entries routing already fixed. Get the ordering wrong and a cell analyses another region's bits; cap the width wrongly and the tail fills with structure never observed. A squared norm fixed at a quarter of the width is what makes magnitude carry no information, so a residual means departure from learned structure.

**Section (clipping)** · `sec:sentinel:area-clipping`

If this fails: the defence against poisoning becomes the thing that poisons. A clipped baseline has a second, biased fixed point where tight clipping keeps rejecting the evidence that would loosen it — an axis clips itself into a baseline it then refuses all evidence against, and it looks settled while doing so. The graduated exemption widens the basin of the correct one and must narrow smoothly, or a batch is judged by a far tighter rule than the one before it. The truncation must also cost nothing: a spread shrunk by the missing tail leaves the baseline correctly centred and miscalibrated for every score divided by it.

**Section (config)** · `sec:sentinel:area-config`

If this fails: an incoherent sentinel runs instead of refusing to start. The refusals are not tidiness — a capacity of nothing leaves the tracker no basis to hold, a clip width of nothing rejects every observation the baseline learns from, a negative allowance manufactures the drift it absorbs, and a drift reference decaying no slower than the baseline follows a gradual shift rather than exposing it. The schedules carry the same weight: what each depth is warmed with, and what counts as warming switched off. Advice and refusal stay separate channels, so a configuration that merely converges too slowly still runs and still says so.

**Section (convergence)** · `sec:sentinel:area-convergence`

If this fails: warm-up is sized from numbers nobody can trust. A baseline closing on a level at the rate the forgetting factor dictates is what makes warm-up length computable rather than discovered, and a spread lagging its own mean leaves every score miscalibrated after the level looks settled. The area also holds the instruments — the unweighted window average, settling dated from the round after the last violation, stationarity as an early block against a late one, jitter as a fraction of its level. A yardstick with a memory of its own would judge a long-memory baseline by an equally sluggish standard, and every bound derived here would be measuring the ruler.

**Section (coordination)** · `sec:sentinel:area-coordination`

If this fails: a pattern spread across sibling cells stays invisible, because no cell sees it alone. A context fires only where both subtrees contribute, so a node firing on one branch reports a group that does not exist, and a context carried forward after a subtree falls silent describes an earlier batch as this one. Membership nests with the tree — an ancestor measures a superset of its descendant — which is what lets one pattern be read at two scales. The tier works in four dimensions whatever the cells beneath it analyse, and that fixed geometry is the only reason a second tier is affordable.

**Section (coverage)** · `sec:sentinel:area-coverage`

If this fails: an attacker picks the corner that is not covered. Sudden against gradual, confined against system-wide: each corner is answered by a different mechanism — one batch comparison, accumulation for the disturbance that never grows louder, per-cell scoring for the one hiding behind ordinary traffic, and the coarse end of the chain for the shift that moves everything at once and leaves no cell unusual against its neighbours. Lose a corner and being quiet and being partial become additive protections.

**Section (cusum)** · `sec:sentinel:area-cusum`

If this fails: a persistent shift is indistinguishable from one loud batch. Evidence is built from the present run only — the sum is clamped at rest, so a lull banks no credit a later rise must first repay — and the dead band is scaled to the baseline's own spread, so a noisy axis tolerates more before it counts as drifting. The reference gives all of it meaning: a reset that discarded it would leave the axis blind while a long memory rebuilt itself, and one not seeded from the converged fast baseline registers its own lag as drift until that lag closes.

**Section (decay)** · `sec:sentinel:area-decay`

If this fails: the host loses temporal policy while appearing to keep it. Decay rescales spatial standing and stops there — the models built are not the same asset as the standing that justified building them — so a decay reaching the trackers or the observation record discards learning the host only meant to reprioritise. Importance is held in integers, so repeated attenuation arrives at nothing rather than leaving a residue that outranks a genuinely new cell. A subtree decay must spend its whole effect inside its target, or one region cannot re-form without punishing the rest.

**Section (determinism)** · `sec:sentinel:area-determinism`

If this fails: no difference between two runs can be attributed. Reports come out in ascending handle order rather than in whatever order a traversal produced, so two readouts compare position by position; without that, a reader diffing them sees churn that means nothing. The seed is the only randomness in the engine and is genuinely visible in the scores, since warming noise shapes the subspace a tracker starts from — which is why reproducibility has to be stated in terms of the seed and not of the data alone.

**Section (edge)** · `sec:sentinel:area-edge`

If this fails: the corners of the input space bring the run down instead of being reported. The extremes of the domain are ordinary observations because the encoding centres every bit; a stream with no variation still counts in full and simply teaches no new direction; a cell driven too narrow to model is declined and counted rather than handed a tracker that could form no basis. Idling on empty batches must leave the engine exactly where it was, so the burst that follows is scored as though the quiet had never happened.

**Section (engine)** · `sec:sentinel:area-engine`

If this fails: the public surface stops describing the thing behind it. A configuration reads back as given because construction validates without rewriting, so the accessor is the authority on how this sentinel behaves; a handle from another sentinel inspects to nothing rather than to whichever local cell sits at that index. The root tracker is permanent, which gives the tracked count a floor and every ancestor chain somewhere to terminate. The counter accumulates observations and not batches, and the handle list and the tracked count are two views of one map — each a place where two records could quietly disagree.

**Section (ewma)** · `sec:sentinel:area-ewma`

If this fails: the notion of normal is either unearned or unmovable. The first batch is adopted outright rather than blended with a placeholder that was never an observation, and warmth is never manufactured by seeding from a cold source. A batch of one carries no spread, so taking its deviation as zero would make every later value an outlier. The ceiling refuses exactly what an attacker would use to walk normal upward — but while cold there is nothing to clip against, and without a floor on the spread a quiet stretch closes the ceiling onto the mean and freezes it there.

**Section (health)** · `sec:sentinel:area-health`

If this fails: an operator watches a picture of a different engine. Health is a readout of present state rather than a summary of the last batch, so it can be polled on the host's own schedule and still describe the whole run. The distinctions it draws are the ones a host acts on: real observations kept apart from synthetic seeding, coldness measured as exposure to the world rather than whether a model is populated, an axis structurally unavailable told apart from one merely quiet. The tracker counts are one partition reported three ways, so disagreement among them is an accounting fault.

**Section (invariant)** · `sec:sentinel:area-invariant`

If this fails: the guarantees were only ever properties of a settled system. They are asserted batch by batch through unstructured traffic and through a collapse and the regrowth after it, because the transient is where a bound slips. What they protect is the readability of everything else: scores run one way, so a host may threshold without knowing which axis produced a number; a drift accumulator never falls below rest, so a rise starts from zero; and no axis returns a value that is not a number, since such a value compares false against every threshold and would disarm the alerting silently.

**Section (noise)** · `sec:sentinel:area-noise`

If this fails: a cell cannot say how much of what it knows it invented. The synthetic share gates clip width and decides when warm-up is over, so it has to follow its closed form exactly — a batch of decay computed as one power, never accumulated a sample at a time. It falls only under real data and rises only under injection, without reversal either way, which lets a threshold crossing mean the same thing whenever it happens. The injected traffic must be shaped and centred like the real encoding, or a model is warmed on something it will never be asked to judge.

**Section (pressure)** · `sec:sentinel:area-pressure`

If this fails: refusal becomes permanent and nobody sees it happen. The rejection rate is both symptom and control — it reports how hard an axis is working to keep contamination out of its baseline, and widens that axis's own ceiling while it is high, so a lasting shift in the traffic can eventually be learned rather than clipped away for ever. It must settle under traffic that keeps its shape rather than ratcheting, or a rise carries no news; it must age out, or one past episode leaves a forever-loose ceiling; and what warm-up accrued must be discarded at the crossing.

**Section (readout)** · `sec:sentinel:area-readout`

If this fails: a host reads numbers it cannot interpret or compare. The report partitions cells by how they earned their place, so the competitive list is the sentinel's own investment decisions and nothing else; the analysis width travels beside the scores, because readings from different depths are not commensurable and a bare number invites the comparison anyway. Degeneracy is stated rather than hidden — novelty saturable told apart from saturated, cells too narrow to track counted rather than dropped, churn scoped to the interval since the last report. An empty ingest returns the shape a busy one does.

**Section (resistance)** · `sec:sentinel:area-resistance`

If this fails: an attacker who can address any part of the domain decides what the sentinel spends. The cap is on attention rather than on input, and capping the winners is hollow unless the ancestors connecting them are bounded too, since each carries a tracker — so the total cost follows the cap and the depth reached, never how many ranges were touched. The node budget is enforced by eviction rather than hinted at, and attention is bought with accumulated weight, which is what stops a thin spray from evicting the range an operator actually cares about.

**Section (routing)** · `sec:sentinel:area-routing`

If this fails: the selector reads something other than where the traffic is. Every value adds exactly one unit wherever it lands — importance is a count of arrivals, not a weight a caller can set — so the graph total and the sentinel's own counter stay one quantity read from two layers, with synthetic warming kept out of both. An empty batch is not an event: quiet time neither adds evidence nor moves the partition. Resolution is bought with observations, and the node budget is what keeps refining everywhere at once from being unbounded.

**Section (schedule)** · `sec:sentinel:area-schedule`

If this fails: a cell scores its first real batch against nothing, and the arrival of a new region is reported as an anomaly in it. Warming is not a construction-time favour to the root — cells born mid-run are warmed as they enter the analysis set, and contexts are warmed on activation from the baselines of the cells they watch, since a context scores score vectors rather than coordinates. Warming rounds must never enter the observed count, and the drift accumulators must be wound back at hand-over, or drift chased during warm-up is charged to the first real request.

**Section (selection)** · `sec:sentinel:area-selection`

If this fails: the sentinel invests its budget somewhere other than where the evidence is. Ranking is by accumulated importance alone, with ties settled by a property of the coordinate domain rather than by whatever order a walk produced — the foundation every downstream score's reproducibility rests on. The root is in the full set unconditionally so closure terminates, and is never competitive, because it accumulates everything by construction and would crowd out the cells whose behaviour is informative. The summary counts competition and closure apart, so the price of ancestry stays visible.

**Section (serde)** · `sec:sentinel:area-serde`

If this fails: the readout cannot leave the process that produced it. The nesting is deep and generic over the coordinate type, so a batch report surviving with its lists and counts intact is what makes handing measurements to another host possible at all. Components travel on their own as well, since a host forwarding structural state to one consumer and scores to another should not have to ship the whole readout to either. Optional detail must cross as present or absent — detail lost in transit looks like a host that never asked for it.

**Section (staging)** · `sec:sentinel:area-staging`

If this fails: a half-built model reaches a report, or an evicted one comes back. A cell counts as present from the moment it is enqueued, which is what stops the reconciler enqueuing it twice; presence is answered across every state it can occupy, including while a background thread holds it, because the expensive injection happens outside the lock. Promotion moves ownership rather than copying it, so no cell is promoted twice however often the queue is drained. A cell evicted in flight is discarded on return — cheaper than admitting one the selector has already rejected.

**Section (subspace)** · `sec:sentinel:area-subspace`

If this fails: the per-cell model asserts structure it never earned, or publishes scores whose scale nobody can see. A new model claims a single direction and counts itself entirely noise-taught, so every further direction is earned from energy it actually observed. The residual degrees of freedom it publishes are the divisor novelty was normalised by — without them a host compares numbers across depths and ranks blind. The spare axis beyond what the energy threshold demands is where a genuinely new direction first lands; without it, novel structure stays invisible until it is already dominant.

**Section (suffix)** · `sec:sentinel:area-suffix`

If this fails: a cell models bits that carry no information and reports scores nobody can compare. The bits its depth stands for were fixed by routing and are identical for every value reaching it, so a cell's width is a consequence of position rather than a setting anyone can get wrong, and its geometry reports that same width rather than a second number that happens to agree. The ceiling on learned structure is whichever is smaller, the configured maximum or the cell's own space — the budget is never a promise of capacity.

**Section (svd)** · `sec:sentinel:area-svd`

If this fails: the cheap path stops being the same answer and becomes an approximation nobody audited. Everything downstream assumes the returned axes are orthonormal — coordinates are a projection onto them and novelty is what the projection leaves over, which is a decomposition only if they are — and assumes descending order, since rank adaptation walks the values accumulating energy and takes a position as the rank. Agreement must hold over a whole streaming run with divergence growing no faster than the steps taken, and both paths must explain unseen data equally well.

**Section (variance)** · `sec:sentinel:area-variance`

If this fails: the divisor every score rests on stops meaning anything. The spread is measured against the centre a batch arrived to find, before the mean moves, or a batch partly explains its own deviation away. Centring on the running mean is what lets a single-row batch contribute a real squared departure instead of finding no scatter within itself, driving the divisor to its floor and turning ordinary traffic into scores orders of magnitude too large. A score of about one has to mean as expected everywhere, or no threshold can be set once and applied across cells.

**Section (warmup)** · `sec:sentinel:area-warmup`

If this fails: a cell enters service on a model it never built, and says nothing about it. Real batches displace the synthetic seed geometrically, so maturity arrives within a predictable number of batches and only ever improves — a figure that could rebound would be useless as progress. A freshly promoted cell is mostly synthetic and publishes that fact, which is what lets a host discount its scores. Deferring the work through a staging area must change when a cell becomes live and nothing else, and a region under construction is still watched by the ancestor chain above it.

**Section (width)** · `sec:sentinel:area-width`

If this fails: the engine is usable at one domain size only. Width is a property of the type rather than of the configuration, so the same settings must serve either alias and the same arithmetic must hold for both — the width a cell analyses is read from the sentinel's own parameter rather than assumed. A host working a narrower coordinate space would otherwise keep a second set of settings, or discover at run time that a value it sent was shifted out of range.

## Development · `sec:sentinel:readme-development`

### Building and testing · `sec:sentinel:readme-building-and-testing`

```sh
# Unit + integration tests (debug, optimised dev profile):
cargo test -p torrust-sentinel --all-targets --all-features

# Doc-tests, including this README:
cargo test -p torrust-sentinel --all-features --doc

# Release mode:
cargo test -p torrust-sentinel --all-targets --all-features --release

# No-default-features spot-check:
cargo test -p torrust-sentinel --all-targets --no-default-features

# Clippy and docs:
cargo clippy -p torrust-sentinel --all-targets --all-features
cargo doc -p torrust-sentinel --all-features --no-deps
```

The README is included from [src/lib.rs](src/lib.rs) under `#[cfg(doctest)]`, so Rust code blocks here compile and run as part of `cargo test --doc`.

Structured diagnostics use `tracing`. They are silent by default; attach a subscriber when you need debug or trace-level detail from tracker and SVD internals.
