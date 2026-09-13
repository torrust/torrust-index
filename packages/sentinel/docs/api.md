# Sentinel — Public API Reference · `spec:sentinel:api-reference`

The definitive specification for the public surface of the `torrust-sentinel` crate. Everything listed here is intentionally public. Everything else is `pub(crate)` or private — implementation detail, subject to change without notice.

Modelled on `packages/mudlark/docs/api.md`.

---

## §1. Design Principles · `sec:sentinel:api-design-principles`

1. **Measure, don't decide.** All outputs are raw statistical quantities. The sentinel never emits threat levels, recommended actions, or policy decisions. The host reads the reports and applies its own policy.

2. **Feed-forward invariant (ADR-S-002).** The G-V Graph receives only `observe(v, 1u64)` per raw input value. Anomaly scores never flow back into the spatial layer's importance signal. The spatial layer sees pure volume counting — Δ=1 per observation. This prevents the anomaly detector from influencing its own spatial structure.

3. **Host controls temporal policy.** The sentinel never calls `decay()` automatically. The host schedules decay: when, how aggressively, and with what selectivity (§ALGO S-13.4).

4. **Adapt, don't control.** The spatial structure evolves autonomously under observation and decay. The host controls resource ceilings, temporal policy, and analysis budgets — not the structure itself.

> _Design note (why volume-only importance)._ If anomaly scores boosted importance, the affected cell would earn finer resolution, changing its statistical model, changing its scores, changing its importance — an unstable feedback loop. With Δ=1, an adversary cannot influence the spatial structure except through observation volume, which is precisely what the spatial layer is designed to handle.

---

## §2. Three-Layer Architecture · `sec:sentinel:api-three-layer-architecture`

```
Layer 1: Spatial Index (mudlark GvGraph)
         Adaptive spatial partitioning of [0, 2^N)
         Pure volume tracking (Δ = 1 per observation)
         Competitive ranking by observation volume
              │
              │ Significance ranking → top-K selection
              ▼
Layer 2: Analysis Selector
         Selects significant cells for statistical analysis
         Closes selection under spatial ancestry
              │
              │ Suffix bit vectors at every ancestor depth
              ▼
Layer 3: Analysis Engine (subspace trackers, coordination)
         Per-cell subspace models at selected and ancestor cells
         Hierarchical coordination across related cells
              │
              ▼
         BatchReport → host
```

### §2.1 Layer Responsibilities · `sec:sentinel:api-layer-responsibilities`

| Concern                                               | Owner                         |
| ----------------------------------------------------- | ----------------------------- |
| Spatial partitioning of $[0, 2^N)$                    | Spatial Layer (Layer 1)       |
| Competitive significance ranking                      | Spatial Layer — Value Tree    |
| Spatial lifecycle (split, evict, absorb, restore)     | Spatial Layer                 |
| Spatial memory (temporal decay, contour evolution)    | Spatial Layer + host policy   |
| Investment commitment (which cells receive trackers)  | Analysis Selector (Layer 2)   |
| Production selection (which invested cells score)     | Analysis Selector (Layer 2)   |
| Ancestor closure (spatial ancestry of targets)        | Analysis Selector (Layer 2)   |
| Statistical modelling within each cell                | Analysis Engine (Layer 3)     |
| Anomaly scoring (four axes)                           | Analysis Engine               |
| Drift detection (CUSUM accumulators)                  | Analysis Engine               |
| Cross-cell coordination detection                     | Analysis Engine — coordination |
| Interpretation and response                           | Host (external)               |

See §ALGO S-1.4–1.5 for the full architecture specification.

---

## §3. Crate Root Re-exports · `sec:sentinel:api-crate-root-reexports`

```rust
// Type aliases (convenience).
pub type Sentinel128 = SpectralSentinel<u128, u64, 128>;
pub type Sentinel64 = SpectralSentinel<u64, u64, 64>;

// Re-exported from mudlark for decay_subtree() handles.
pub use torrust_mudlark::GNodeId;

// Modules are crate-private.
pub(crate) mod analysis_set;
pub(crate) mod config;
pub(crate) mod ewma;
pub(crate) mod maths;
pub(crate) mod observation;
pub(crate) mod report;
pub(crate) mod sentinel;

// Surface 1 — report/view types.
pub use report::{
    AnalysisSetSummary, AnomalyScores, AxisBaselineSnapshots,
    BaselineSnapshot, BatchReport, CellInspection, CellReport,
    ClipPressureDistribution, ContourSnapshot, CoordinationHealth,
    CoordinationReport, CusumSnapshot, GeometryDistribution,
    HealthReport, MaturityDistribution, MemberScore,
    RankDistribution, SampleScore, ScoreDistribution,
    ScoringGeometry, TrackerMaturity,
};

// Surface 2 — operational types.
pub use analysis_set::{AnalysisEntry, AnalysisSet};
pub use config::{
    ConfigError, ConfigErrors, ConfigWarning, NoiseSchedule,
    SentinelConfig,
};
pub use maths::SvdStrategy;
pub use observation::{CentredBitSource, CentredBits};
pub use sentinel::SpectralSentinel;

// Not re-exported: MIN_TRACKER_DIM (pub(crate) const).
```

Hidden compatibility/testing affordances may also be re-exported with `#[doc(hidden)]`; they are not part of the stable public surface.

The type aliases wrap `SpectralSentinel<C, V, N>` for the most common domain widths:

| Alias         | $C$      | $V$    | $N$  | Use case                     |
| ------------- | -------- | ------ | ---- | ---------------------------- |
| `Sentinel128` | `u128`   | `u64`  | 128  | IPv6-class data (default)    |
| `Sentinel64`  | `u64`    | `u64`  | 64   | 64-bit domains               |

---

## §4. Surface 1 — Report Types · `sec:sentinel:api-report-types`

Lightweight, read-only **view types** — detached from the sentinel. Returned by `ingest()`. All carry `Debug`, `Clone`, `serde` (with feature).

These types carry raw statistical measurements — never opinions or recommended actions. The host reads these reports and applies its own policy.

### §4.1 Batch-level Reports · `sec:sentinel:api-batch-level-reports`

#### `BatchReport<C>` — `Clone` · `sec:sentinel:api-batch-report`

Complete statistical output from one `ingest()` call.

```rust
pub struct BatchReport<C: Copy + Debug> {
    pub cell_reports: Vec<CellReport<C>>,
    pub ancestor_reports: Vec<CellReport<C>>,
    pub coordination_reports: Vec<CoordinationReport<C>>,
    pub contour: ContourSnapshot,
    pub health: HealthReport,
    pub analysis_set_summary: AnalysisSetSummary,
    pub oldest_observation_age_micros: Option<u64>,
}
```

| Field                           | Content                                                  |
| ------------------------------- | -------------------------------------------------------- |
| `cell_reports`                  | Per-cell reports for competitive cells ($\mathcal{A}$)   |
| `ancestor_reports`              | Per-cell reports for ancestor-only cells                 |
| `coordination_reports`          | Cross-cell coordination analysis (§ALGO S-9.4)           |
| `contour`                       | Snapshot of G-V Graph spatial structure                  |
| `health`                        | Operational health snapshot                              |
| `analysis_set_summary`          | Summary of investment/producing sets                     |
| `oldest_observation_age_micros` | Age of the batch's oldest observation at report emission, in microseconds; absent where there is none |

All vector fields are ordered by `GNodeId` for deterministic output (ADR-S-005).

`oldest_observation_age_micros` is a duration on the sentinel's own monotonic clock — stamped as the batch arrives, read off as the report is assembled — and never a wall-clock instant, so it carries no cross-machine skew. A batch arrives whole, so the single figure bounds every observation in it. `None` is reported both for a batch that carried no observations and for a payload written before the field existed: neither carries a measurement, and a zero would claim one. The field deserialises with `#[serde(default)]`, so older payloads read back unchanged. How long the host held the observations before handing them over is not included and is deliberately unmeasured (`sec:sentinel:algorithm-output-observation-age`).

### §4.2 Cell-level Reports · `sec:sentinel:api-cell-level-reports`

#### `CellReport<C>` — `Clone` · `sec:sentinel:api-cell-report`

Statistics for a single analysis cell after processing one batch.

```rust
pub struct CellReport<C: Copy + Debug> {
    pub gnode_id: GNodeId,
    pub start: C,
    pub end: C,
    pub depth: u32,
    pub analysis_width: usize,
    pub is_competitive: bool,
    pub sample_count: usize,
    pub rank: usize,
    pub energy_ratio: f64,
    pub top_singular_value: f64,
    pub scores: AnomalyScores,
    pub maturity: TrackerMaturity,
    pub geometry: ScoringGeometry,
    pub per_sample: Option<Vec<SampleScore>>,
}
```

| Field               | Content                                              |
| ------------------- | ---------------------------------------------------- |
| `gnode_id`          | Arena handle of the backing G-node                   |
| `start`, `end`      | Dyadic interval `[start, end)`, except at the top of the domain: the cell whose bound is the domain maximum owns that maximum, because a coordinate width filling the coordinate type leaves no value above it to be excluded |
| `depth`             | G-tree depth of this cell                            |
| `analysis_width`    | Suffix width: `N - depth`                            |
| `is_competitive`    | `true` if competitively selected                     |
| `sample_count`      | Observations in this batch routed to this cell       |
| `rank`              | Current rank of the learned subspace                 |
| `energy_ratio`      | Fraction of variance captured by current rank        |
| `top_singular_value`| Largest singular value                               |
| `scores`            | Anomaly scores along all four axes                   |
| `maturity`          | Tracker maturity (real vs noise observations)        |
| `geometry`          | Geometric scoring properties                         |
| `per_sample`        | Per-sample scores (if `per_sample_scores` enabled)   |

### §4.3 Coordination Reports · `sec:sentinel:api-coordination-reports`

#### `CoordinationReport<C>` — `Clone` · `sec:sentinel:api-coordination-report`

Coordination analysis at a single G-tree internal node (§ALGO S-9.1).

The coordination tracker operates at $w = 4$, consuming running-mean-centred cell-score matrices as observations. The group consists of all competitive cells in this node's subtree that reported scores in this batch.

```rust
pub struct CoordinationReport<C: Copy + Debug> {
    pub gnode_id: GNodeId,
    pub start: C,
    pub end: C,
    pub depth: u32,
    pub cells_reporting: usize,
    pub rank: usize,
    pub energy_ratio: f64,
    pub top_singular_value: f64,
    pub scores: AnomalyScores,
    pub maturity: TrackerMaturity,
    pub geometry: ScoringGeometry,
    pub per_member: Option<Vec<MemberScore<C>>>,
}
```

The four anomaly axes have **second-order meaning** at coordination level:

| Meta-axis    | Detects                                          |
| ------------ | ------------------------------------------------ |
| Novelty      | A cell-score pattern the model has never seen    |
| Displacement | The overall score landscape has shifted          |
| Surprise     | A specific scoring axis is system-wide anomalous |
| Coherence    | An unusual combination of axis elevations        |

### §4.4 Anomaly Scores · `sec:sentinel:api-anomaly-scores`

#### `AnomalyScores` — `Clone` · `sec:sentinel:api-anomaly-scores-type`

The four anomaly-score axes for a batch of observations.

```rust
pub struct AnomalyScores {
    pub novelty: ScoreDistribution,
    pub displacement: ScoreDistribution,
    pub surprise: ScoreDistribution,
    pub coherence: ScoreDistribution,
}
```

| Axis         | Metric                                      | Intuition                                          |
| ------------ | ------------------------------------------- | -------------------------------------------------- |
| Novelty      | Residual energy / DOF: `‖X − X̂‖² / (dim−k)`| "How much of this is foreign?"                     |
| Displacement | `‖z‖² / (k + ‖z‖²)`, bounded in `[0, 1)`    | "How far is this from the centroid?"               |
| Surprise     | Mahalanobis / rank: `Σⱼ ((zⱼ−μⱼ)/σⱼ)² / k`  | "The shape is familiar, but magnitude is wild"     |
| Coherence    | Cross-correlation deviation                 | "Normal individually, unusual combination"         |

All axes share the same polarity: **higher values indicate greater anomalous departure**. This ensures uniform z-score interpretation.

> **Why not projection energy / "normality"?** Under the sentinel's centred binary encoding, every observation has the same L2 norm (`dim / 4`). Projection energy is therefore a perfect affine function of residual energy — it carries zero independent information. See §ALGO S-Appendix B.

#### `ScoreDistribution` — `Copy` · `sec:sentinel:api-score-distribution`

Summary statistics for a vector of anomaly scores.

```rust
pub struct ScoreDistribution {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub max_z_score: f64,
    pub mean_z_score: f64,
    pub baseline: BaselineSnapshot,
    pub cusum: CusumSnapshot,
    pub clip_pressure: f64,
}
```

### §4.5 Baseline and CUSUM Snapshots · `sec:sentinel:api-baseline-and-cusum-snapshots`

#### `BaselineSnapshot` — `Copy` · `sec:sentinel:api-baseline-snapshot`

Frozen snapshot of an EWMA baseline at the time of scoring.

```rust
pub struct BaselineSnapshot {
    pub mean: f64,
    pub variance: f64,
}
```

#### `CusumSnapshot` — `Copy` · `sec:sentinel:api-cusum-snapshot`

Frozen snapshot of a CUSUM accumulator at scoring time.

```rust
pub struct CusumSnapshot {
    pub accumulator: f64,
    pub slow_baseline: BaselineSnapshot,
    pub steps_since_reset: u64,
}
```

### §4.6 Maturity and Geometry · `sec:sentinel:api-maturity-and-geometry`

#### `TrackerMaturity` — `Copy` · `sec:sentinel:api-tracker-maturity`

How much experience a tracker has, and how much of that is noise.

```rust
pub struct TrackerMaturity {
    pub real_observations: u64,
    pub noise_observations: u64,
    pub noise_influence: f64,
}
```

Methods: `cold() -> Self`, `total_observations() -> u64`.

#### `ScoringGeometry` — `Copy` · `sec:sentinel:api-scoring-geometry`

Geometric properties of the tracker's scoring state.

```rust
pub struct ScoringGeometry {
    pub dim: usize,
    pub cap: usize,
    pub residual_dof: usize,
}
```

Methods: `is_novelty_saturated() -> bool`, `is_novelty_saturable() -> bool`.

### §4.7 Per-sample Scores · `sec:sentinel:api-per-sample-scores`

#### `SampleScore` — `Copy` · `sec:sentinel:api-sample-score`

Raw anomaly scores for a single observation.

```rust
pub struct SampleScore {
    pub novelty: f64,
    pub displacement: f64,
    pub surprise: f64,
    pub coherence: f64,
    pub novelty_z: f64,
    pub displacement_z: f64,
    pub surprise_z: f64,
    pub coherence_z: f64,
}
```

#### `MemberScore<C>` — `Copy` · `sec:sentinel:api-member-score`

Per-member scores at coordination level (identifies contributing cell).

### §4.8 Health and Summary Reports · `sec:sentinel:api-health-and-summary-reports`

#### `HealthReport` — `Clone` · `sec:sentinel:api-health-report`

Operational health snapshot of the entire sentinel.

```rust
pub struct HealthReport {
    pub total_g_nodes: usize,
    pub semi_internal_count: usize,
    pub active_trackers: usize,
    pub active_competitive_trackers: usize,
    pub active_ancestor_trackers: usize,
    pub active_coordination_contexts: usize,
    pub investment_set_size: usize,
    pub warming_trackers: usize,
    pub warming_competitive_targets: usize,
    pub lifetime_observations: u64,
    pub cells_tracked: usize,
    pub rank_distribution: RankDistribution,
    pub maturity_distribution: MaturityDistribution,
    pub geometry_distribution: GeometryDistribution,
    pub coordination_health: CoordinationHealth,
    pub clip_pressure_distribution: ClipPressureDistribution,
}
```

#### `AnalysisSetSummary` — `Copy` · `sec:sentinel:api-analysis-set-summary`

Summary of the analysis set at report time.

```rust
pub struct AnalysisSetSummary {
    pub competitive_size: usize,
    pub full_size: usize,
    pub investment_set_size: usize,
    pub depth_range: (u32, u32),
    pub importance_range: (f64, f64),
    pub v_depth_range: (usize, usize),
    pub degenerate_cells_skipped: usize,
}
```

#### `ContourSnapshot` — `Copy` · `sec:sentinel:api-contour-snapshot`

Snapshot of the G-V Graph's spatial contour at report time.

```rust
pub struct ContourSnapshot {
    pub plateau_count: usize,
    pub cell_count: usize,
    pub total_importance: f64,
    pub splits_since_last_report: u32,
    pub net_removals_since_last_report: u32,
}
```

### §4.9 Distribution Summaries · `sec:sentinel:api-distribution-summaries`

#### `RankDistribution` — `Copy` · `sec:sentinel:api-rank-distribution`

```rust
pub struct RankDistribution {
    pub min: usize,
    pub max: usize,
    pub mean: f64,
}
```

#### `MaturityDistribution` — `Copy` · `sec:sentinel:api-maturity-distribution`

```rust
pub struct MaturityDistribution {
    pub max_noise_influence: f64,
    pub min_noise_influence: f64,
    pub mean_noise_influence: f64,
    pub cold_trackers: usize,
}
```

#### `GeometryDistribution` — `Copy` · `sec:sentinel:api-geometry-distribution`

```rust
pub struct GeometryDistribution {
    pub novelty_saturated: usize,
    pub novelty_saturable: usize,
    pub coherence_inactive: usize,
}
```

#### `ClipPressureDistribution` — `Copy` · `sec:sentinel:api-clip-pressure-distribution`

```rust
pub struct ClipPressureDistribution {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
}
```

#### `CoordinationHealth` — `Copy` · `sec:sentinel:api-coordination-health`

Health snapshot of the hierarchical coordination tier.

```rust
pub struct CoordinationHealth {
    pub active_contexts: usize,
    pub capacity: usize,
    pub rank_distribution: RankDistribution,
    pub maturity_distribution: MaturityDistribution,
    pub dim: usize,
    pub geometry_distribution: GeometryDistribution,
}
```

### §4.10 Inspection Types · `sec:sentinel:api-inspection-types`

#### `CellInspection<C>` — `Clone` · `sec:sentinel:api-cell-inspection`

Detailed snapshot of a cell's tracker state. Returned by `SpectralSentinel::inspect_cell()`.

```rust
pub struct CellInspection<C: Copy + Debug> {
    pub gnode_id: GNodeId,
    pub start: C,
    pub end: C,
    pub depth: u32,
    pub analysis_width: usize,
    pub is_competitive: bool,
    pub rank: usize,
    pub energy_ratio: f64,
    pub top_singular_value: f64,
    pub maturity: TrackerMaturity,
    pub geometry: ScoringGeometry,
    pub baselines: AxisBaselineSnapshots,
}
```

#### `AxisBaselineSnapshots` — `Copy` · `sec:sentinel:api-axis-baseline-snapshots`

Per-axis EWMA baseline snapshots for all four scoring axes.

```rust
pub struct AxisBaselineSnapshots {
    pub novelty: BaselineSnapshot,
    pub displacement: BaselineSnapshot,
    pub surprise: BaselineSnapshot,
    pub coherence: BaselineSnapshot,
}
```

---

## §5. Surface 2 — Operational Types · `sec:sentinel:api-operational-types`

### §5.1 `SentinelConfig<V>` — `Clone` · `sec:sentinel:api-sentinel-config`

Measurement parameters for the sentinel. Every field controls *how* the sentinel observes and learns, never *what it thinks* about what it sees.

```rust
pub struct SentinelConfig<V: Accumulator> {
    // ── Subspace parameters ─────────────────────────────
    pub max_rank: usize,              // Default: 16
    pub forgetting_factor: f64,       // Default: 0.99, in (0.0, 1.0)
    pub rank_update_interval: u64,    // Default: 100
    pub energy_threshold: f64,        // Default: 0.90, in (0.0, 1.0)
    pub eps: f64,                     // Default: 1e-6

    // ── CUSUM parameters ────────────────────────────────
    pub cusum_slow_decay: f64,        // Default: 0.999
    pub cusum_coord_slow_decay: f64,  // Default: 0.999
    pub cusum_allowance_sigmas: f64,  // Default: 0.5

    // ── Clip parameters ─────────────────────────────────
    pub clip_sigmas: f64,             // Default: 3.0
    pub clip_pressure_decay: f64,     // Default: 0.95

    // ── Analysis selection ──────────────────────────────
    pub analysis_k: usize,            // Default: 1024
    pub analysis_depth_cutoff: usize, // Default: 6

    // ── G-V Graph passthrough ───────────────────────────
    pub split_threshold: V,           // Default: 100
    pub d_create: u32,                // Default: 3
    pub d_evict: u32,                 // Default: 6
    pub budget: usize,                // Default: 100_000

    // ── Noise injection ─────────────────────────────────
    pub noise_schedule: NoiseSchedule,
    pub noise_batch_size: usize,      // Default: 16
    pub noise_seed: Option<u64>,      // Default: Some(42)
    pub background_warming: bool,     // Default: false

    // ── Output control ──────────────────────────────────
    pub per_sample_scores: bool,      // Default: false

    // ── SVD strategy ────────────────────────────────────
    pub svd_strategy: SvdStrategy,    // Default: Brand
}
```

#### Key field groups · `sec:sentinel:api-sentinel-config-field-groups`

**Subspace parameters** (§ALGO S-4):
- `max_rank`: Maximum basis vectors any tracker can use.
- `forgetting_factor` (λ): Exponential decay rate. Half-life ≈ `ln(2) / ln(1/λ)`.
- `rank_update_interval`: Reassess rank every N observations.
- `energy_threshold` (τ): Cumulative energy threshold for rank adaptation.

**CUSUM parameters** (§ALGO S-6):
- `cusum_slow_decay` (λ_s): Slow EWMA decay for drift detection reference.
- `cusum_coord_slow_decay`: Same for coordination tier.
- `cusum_allowance_sigmas` (κ_σ): Noise allowance in slow-baseline σ units.

**G-V Graph passthrough** (§ALGO S-13.3):
- `split_threshold`: Observations before cell subdivision.
- `d_create`: Maximum V-Tree depth for new splits.
- `d_evict`: Minimum V-Tree depth for eviction eligibility.
- `budget`: Hard ceiling on live G-nodes.

**Noise injection** (§ALGO S-11, ADR-S-015):
- `noise_schedule`: Depth-tiered warm-up schedule.
- `background_warming`: When `true`, warm-up runs on a background thread.

Methods: `validate() -> Result<(), ConfigErrors>`, `Default`.

### §5.2 `NoiseSchedule` — `Clone` · `sec:sentinel:api-noise-schedule`

Depth-tiered noise injection schedule.

```rust
pub enum NoiseSchedule {
    Geometric { root: u32, decay: f64, min: u32 },
    Explicit(Vec<u32>),
}
```

| Variant    | Behaviour                                        |
| ---------- | ------------------------------------------------ |
| `Geometric`| `root × decay^depth`, floored to `min`           |
| `Explicit` | Per-depth vector; depths beyond end use last     |

Methods: `geometric(root, decay, min) -> Self`, `rounds_for_depth(depth) -> u32`, `is_disabled() -> bool`, `max_rounds() -> u32`, `Default`.

Default: `Geometric { root: 450, decay: 0.5, min: 50 }`.

### §5.3 `SpectralSentinel<C, V, N>` — Main Orchestrator · `sec:sentinel:api-spectral-sentinel`

The main orchestrator. Generic over coordinate `C`, accumulator `V`, domain width `N`.

```rust
pub struct SpectralSentinel<C, V, const N: u32>
where
    C: Coordinate + CentredBitSource,
    V: Inspectable + Attenuatable,
{
    // ... internal state ...
}
```

#### Method Index · `sec:sentinel:api-spectral-sentinel-method-index`

| Method                    | Category      | Cost                  | Description                                 |
| ------------------------- | ------------- | --------------------- | ------------------------------------------- |
| `new`                     | Construction  | $O(w)$ noise          | Validated config → warmed root tracker      |
| `ingest`                  | Observation   | $O(n × \text{cells})$ | Batch processing, returns `BatchReport`     |
| `decay`                   | Temporal      | $O(\|G\|)$            | Global importance attenuation               |
| `decay_subtree`           | Temporal      | $O(\|G_{sub}\|)$      | Subtree importance attenuation              |
| `reset`                   | Lifecycle     | $O(w)$                | Re-initialise to fresh state                |
| `health`                  | Diagnostic    | $O(\|cells\|)$        | Operational health snapshot                 |
| `config`                  | Accessor      | $O(1)$                | Read-only config reference                  |
| `graph`                   | Accessor      | $O(1)$                | Read-only G-V Graph reference               |
| `analysis_set`            | Accessor      | $O(1)$                | Current analysis set                        |
| `cells_tracked`           | Accessor      | $O(1)$                | Number of active trackers                   |
| `lifetime_observations`   | Accessor      | $O(1)$                | Total real observations                     |
| `degenerate_cells_skipped`| Accessor      | $O(1)$                | Cells excluded for width < 2                |
| `cell_gnodes`             | Accessor      | $O(\|cells\|)$        | List all tracked cell handles               |
| `inspect_cell`            | Diagnostic    | $O(1)$                | Detailed cell state snapshot                |

#### `new(config: SentinelConfig<V>) -> Result<Self, ConfigErrors>` · `sec:sentinel:api-spectral-sentinel-new`

Validates the configuration and creates the root tracker. The root tracker is automatically warmed with synthetic noise (§ALGO S-11.2). No other cells are created until the first `ingest()` call triggers analysis set computation.

#### `ingest(&mut self, values: &[C]) -> BatchReport<C>` · `sec:sentinel:api-spectral-sentinel-ingest`

Process a batch of raw coordinate observations and return a full statistical report. Each value is:
1. Fed to the G-V Graph with Δ=1 (feed-forward invariant).
2. Routed to every analysis cell whose interval contains it.
3. Encoded as centred bits and scored against learned subspaces.

An empty input slice produces an empty report.

#### `decay(&mut self, attenuation: f64, q: f64)` · `sec:sentinel:api-spectral-sentinel-decay`

Apply spatial decay to the entire G-V Graph.

Parameters:
- `attenuation` — base decay factor at midpoint depth.
  - `(0, 1)`: Cold cells lose standing.
  - `> 1.0`: Hot cells reinforced (amplification).
  - `1.0`: No-op.
- `q` — depth selectivity in `[0.0, 1.0]`.
  - `0.0`: Uniform — all depths decay equally.
  - `> 0.0`: Selective — fine structure fades faster.

Panics if `attenuation < 0.0`, `q` out of range, or `NaN`.

#### `decay_subtree(&mut self, root: GNodeId, attenuation: f64, q: f64)` · `sec:sentinel:api-spectral-sentinel-decay-subtree`

Apply spatial decay to a subtree of the G-V Graph.

Use cases (§ALGO S-10.3):
- **Regime change**: Attenuate a subtree that experienced a traffic shift.
- **Suspected poisoning**: `decay_subtree(root, 0.0₊, 1.0)` is a detail flush.
- **Hot reinforcement**: `decay_subtree(root, 1.5, 0.0)` amplifies a hot subtree.

Panics if `root` is stale or parameters out of range.

#### `reset(&mut self)` · `sec:sentinel:api-spectral-sentinel-reset`

Reset the sentinel to its freshly-constructed state. Drops all cell trackers and their learned subspaces, re-initialises the G-V Graph, and zeroes all counters. Configuration is preserved.

### §5.4 `AnalysisSet<C, V>` and `AnalysisEntry<C, V>` · `sec:sentinel:api-analysis-set-and-entry`

Analysis set management (§ALGO S-8.1–8.3).

#### `AnalysisEntry<C, V>` — `Copy` · `sec:sentinel:api-analysis-entry`

A cell selected for analysis by the selector.

```rust
pub struct AnalysisEntry<C: Coordinate, V: Accumulator> {
    pub gnode: GNodeId,
    pub depth: u32,
    pub v_depth: usize,
    pub importance: V,
    pub start: C,
    pub end: C,
    pub is_competitive: bool,
}
```

#### `AnalysisSet<C, V>` — `Clone` · `sec:sentinel:api-analysis-set`

The current analysis set — competitive targets $\mathcal{T}$ closed under G-tree ancestry to form the investment set $\mathcal{I}$.

```rust
pub struct AnalysisSet<C: Coordinate, V: Accumulator> {
    competitive: Vec<AnalysisEntry<C, V>>,
    full: Vec<AnalysisEntry<C, V>>,
}
```

Methods:
- `recompute<const N: u32>(graph, k, depth_cutoff) -> Self`
- `competitive() -> &[AnalysisEntry]` — ordered by importance descending
- `full() -> &[AnalysisEntry]` — ordered by `GNodeId`
- `contains(gnode: GNodeId) -> bool`
- `is_competitive(gnode: GNodeId) -> bool`
- `competitive_count() -> usize`
- `total_count() -> usize`
- `summary() -> AnalysisSetSummary`

#### Selection algorithm (§ALGO S-8.1) · `sec:sentinel:api-analysis-set-selection-algorithm`

1. Scan V-Tree for entries with `v_depth ≤ depth_cutoff` and `width ≥ 2`.
2. Sort by importance (descending), then by `start` (ascending) for determinism.
3. Take top-K (the competitive targets $\mathcal{T}$).
4. Close under G-tree ancestry to form $\mathcal{I}$.

The root is always an ancestor, never competitive (§ALGO S-4.7).

---

## §6. Surface 3 — Internal Machinery · `sec:sentinel:api-internal-machinery`

`pub(crate)` modules not part of the public API:

| Module                    | Contents                                              |
| ------------------------- | ----------------------------------------------------- |
| `sentinel::tracker`       | `SubspaceTracker` — SVD-based subspace model          |
| `sentinel::cusum`         | CUSUM accumulator for drift detection                 |
| `sentinel::staging`       | Deferred warm-up staging area (ADR-S-019)             |
| `sentinel::warming_thread`| Background noise injection thread (§ALGO S-18.2)      |
| `maths`                   | SVD (naive + Brand), matrix ops, Gamma distribution   |
| `ewma`                    | `EwmaStats` — exponential moving average with clipping|
| `observation`             | `CentredBits`, `CentredBitSource` — suffix encoding   |

These modules implement the internal machinery. Their interfaces may change without notice. Only types re-exported from the crate root are part of the public API.

---

## §7. Cross-Cutting Concerns · `sec:sentinel:api-cross-cutting-concerns`

### §7.1 Error Handling · `sec:sentinel:api-error-handling`

- `SentinelConfig::validate() -> Result<(), ConfigErrors>` — validates all configuration parameters.
- `SpectralSentinel::new()` propagates validation errors.
- Panics for stale handles (documented per-method).
- `ConfigErrors` is a `Vec<ConfigError>` carrying all validation failures.

#### `ConfigError` variants · `sec:sentinel:api-config-error-variants`

| Variant                        | Constraint violated                           |
| ------------------------------ | --------------------------------------------- |
| `MaxRankZero`                  | `max_rank` must be ≥ 1                        |
| `ForgettingFactorOutOfRange`   | `forgetting_factor` must be in `(0.0, 1.0)`   |
| `RankUpdateIntervalZero`       | `rank_update_interval` must be ≥ 1            |
| `AnalysisKZero`                | `analysis_k` must be ≥ 1                      |
| `EnergyThresholdOutOfRange`    | `energy_threshold` must be in `(0.0, 1.0)`    |
| `CusumSlowDecayOutOfRange`     | `cusum_slow_decay` must be in `(0.0, 1.0)`    |
| `CusumSlowDecayTooLow`         | `cusum_slow_decay` must be > `forgetting_factor` |
| `DEvictNotGreaterThanDCreate`  | `d_evict` must be > `d_create`                |
| `BudgetTooSmall`               | `budget` must exceed mudlark's headroom       |

### §7.2 Thread Safety · `sec:sentinel:api-thread-safety`

- `SpectralSentinel` is **not** `Sync` due to internal `Mutex<StagingArea>`.
- `BatchReport` and all report types are `Send + Sync`.
- Background warming thread (when `background_warming = true`) runs independently without blocking `ingest()`.

### §7.3 Feature Gates · `sec:sentinel:api-feature-gates`

| Feature | Default | Effect                                      |
| ------- | ------- | ------------------------------------------- |
| `serde` | off     | `Serialize`/`Deserialize` on config + reports |

### §7.4 Serde · `sec:sentinel:api-serde`

All Surface 1 report types carry conditional serde derives. Config types (`SentinelConfig`, `NoiseSchedule`) likewise. Round-trip stability is maintained.

Serde bounds:
- `BatchReport<C>`, `CellReport<C>`, etc.: `C: Serialize + DeserializeOwned`
- `SentinelConfig<V>`: `V: Serialize + DeserializeOwned`

### §7.5 Determinism (ADR-S-005) · `sec:sentinel:api-determinism`

Output *ordering* is deterministic given the same inputs and configuration. Output *values* are deterministic as well with `background_warming` disabled and on a fixed build — one target and one set of dependency versions:
- `BTreeMap` iteration order for cell/coordination maps.
- Tie-breaking by `start` (ascending) in competitive selection.
- Fixed `noise_seed` for reproducible warm-up, within that scope.

Under background warming the same seed and the same traffic still give the same graph, the same investment set and the same ascending-handle report order, but neither the baselines a tracker starts from nor the ingest cycle on which it first scores: the warming worker draws from its own generator and takes whichever staged cell leads on volume when it looks. A caller diffing two runs against each other holds the flag off, or compares converged state rather than cycle-by-cycle output.

---

## §8. Cross-References · `sec:sentinel:api-cross-references`

| Target                | Format               | Example                      |
| --------------------- | -------------------- | ---------------------------- |
| algorithm.md sections | `§ALGO S-N.M`        | `§ALGO S-9.1`                |
| mudlark api.md        | `§API M-N`           | `§API M-4.1` (Cell)          |
| mudlark idea.md       | `§IDEA M-N`          | `§IDEA M-5.5`                |
| sentinel ADRs         | `ADR-S-NNN`          | `ADR-S-002` (feed-forward)   |
| mudlark ADRs          | `ADR-M-NNN`          | `ADR-M-040` (GNodeId)        |
