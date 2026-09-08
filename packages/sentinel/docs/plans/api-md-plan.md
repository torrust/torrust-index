# Plan: Sentinel Public API Reference (`api.md`) · `plan:sentinel:apiplan-public-api-reference`

Create a public API reference document for `torrust-sentinel` modelled on `packages/mudlark/docs/api.md`.

---

## 1. Document Structure · `sec:sentinel:apiplan-document-structure`

Mirror mudlark's seven-section layout, adapted for sentinel's domain:

| §   | Title                           | Content                                                                |
| --- | ------------------------------- | ---------------------------------------------------------------------- |
| 1   | Design Principles               | "Measure, don't decide", feed-forward invariant, host policy control   |
| 2   | Three-Layer Architecture        | Spatial Layer (mudlark), Analysis Selector, Analysis Engine            |
| 3   | Crate Root Re-exports           | Type aliases, re-exported mudlark types, public modules                |
| 4   | Surface 1 — Report Types        | Batch/cell/coordination reports, scores, maturity, geometry, snapshots |
| 5   | Surface 2 — Operational Types   | `SpectralSentinel`, `SentinelConfig`, `NoiseSchedule`                  |
| 6   | Surface 3 — Internal Machinery  | `pub(crate)` modules (tracker, staging, cusum, warming_thread, etc.)   |
| 7   | Cross-Cutting Concerns          | Error handling, thread safety, feature gates, serde                    |

---

## 2. Section Breakdown · `sec:sentinel:apiplan-section-breakdown`

### §1 Design Principles · `sec:sentinel:apiplan-design-principles`

- **Measure, don't decide.** All outputs are raw statistical quantities; no threat levels, no recommended actions.
- **Feed-forward invariant (ADR-S-002).** The G-V Graph receives only `observe(v, 1u64)` — anomaly scores never flow back into importance.
- **Host controls temporal policy.** The sentinel never calls `decay()` automatically; the host schedules it.

Reference: §ALGO S-1.3.

### §2 Three-Layer Architecture · `sec:sentinel:apiplan-three-layer-architecture`

Reproduce the ASCII diagram and table from algorithm.md §1.4–§1.5:

```
Layer 1: Spatial Index (mudlark GvGraph)
         │
         │ Significance ranking → top-K selection
         ▼
Layer 2: Analysis Selector
         │
         │ Suffix bit vectors at every ancestor depth
         ▼
Layer 3: Analysis Engine (subspace trackers, coordination)
         │
         ▼
         BatchReport → host
```

Responsibility table (summarised from §ALGO S-1.5).

### §3 Crate Root Re-exports · `sec:sentinel:apiplan-crate-root-reexports`

Document the `lib.rs` re-exports:

```rust
// Type aliases (convenience).
pub type Sentinel128 = SpectralSentinel<u128, u64, 128>;
pub type Sentinel64 = SpectralSentinel<u64, u64, 64>;

// Re-exported from mudlark.
pub use torrust_mudlark::GNodeId;

// Public modules.
pub mod analysis_set;
pub mod config;
pub mod ewma;
pub mod maths;
pub mod observation;
pub mod report;
pub mod sentinel;
```

### §4 Surface 1 — Report Types (module `report`) · `sec:sentinel:apiplan-report-types`

Lightweight, read-only **view types** — detached from the sentinel. Returned by `ingest()`. All carry `Debug`, `Clone`, `serde` (with feature).

| Type                        | Role                                          | Copy? |
| --------------------------- | --------------------------------------------- | ----- |
| `BatchReport<C>`            | Complete output from one `ingest()` call      | No    |
| `CellReport<C>`             | Per-cell statistics                           | No    |
| `CoordinationReport<C>`     | Cross-cell coordination analysis              | No    |
| `AnomalyScores`             | Four-axis score bundle                        | No    |
| `ScoreDistribution`         | Per-axis summary (min/max/mean/z-scores)      | Copy  |
| `BaselineSnapshot`          | EWMA baseline at scoring time                 | Copy  |
| `CusumSnapshot`             | CUSUM accumulator snapshot                    | Copy  |
| `TrackerMaturity`           | Real/noise observation counts, noise_influence | Copy  |
| `ScoringGeometry`           | dim, cap, residual_dof, saturation predicates | Copy  |
| `SampleScore`               | Per-sample raw scores (if enabled)            | Copy  |
| `MemberScore<C>`            | Per-member scores at coordination level       | Copy  |
| `ContourSnapshot`           | G-V Graph structure snapshot                  | Copy  |
| `HealthReport`              | Operational health summary                    | No    |
| `AnalysisSetSummary`        | Analysis set composition                      | Copy  |
| `RankDistribution`          | Rank min/max/mean across trackers             | Copy  |
| `MaturityDistribution`      | Noise influence distribution                  | Copy  |
| `GeometryDistribution`      | Saturation counts                             | Copy  |
| `CoordinationHealth`        | Coordination tier summary                     | Copy  |
| `ClipPressureDistribution`  | Clip pressure min/max/mean                    | Copy  |
| `AxisBaselineSnapshots`     | Per-axis baseline snapshots                   | Copy  |
| `CellInspection<C>`         | Detailed cell state (if enabled)              | No    |

#### 4.1 `AnomalyScores` axes · `sec:sentinel:apiplan-anomaly-score-axes`

| Axis         | Metric                                      | Intuition                                 |
| ------------ | ------------------------------------------- | ----------------------------------------- |
| Novelty      | Residual energy / DOF                       | "How much of this is foreign?"            |
| Displacement | `‖z‖² / (k + ‖z‖²)`                         | "How far is this from the centroid?"      |
| Surprise     | Mahalanobis / rank                          | "The shape is familiar, but magnitude wild" |
| Coherence    | Cross-correlation deviation                 | "Normal individually, unusual combination" |

All axes share polarity: **higher = more anomalous**.

### §5 Surface 2 — Operational Types · `sec:sentinel:apiplan-operational-types`

#### 5.1 `SentinelConfig<V>` · `sec:sentinel:apiplan-sentinel-config`

Configuration struct controlling measurement parameters. Fields documented with defaults, ranges, and algorithm.md cross-references.

Key field groups:
- Subspace parameters (`max_rank`, `forgetting_factor`, `rank_update_interval`, `energy_threshold`)
- CUSUM parameters (`cusum_slow_decay`, `cusum_coord_slow_decay`, `cusum_allowance_sigmas`)
- Clip parameters (`clip_sigmas`, `clip_pressure_decay`)
- Analysis selection (`analysis_k`, `analysis_depth_cutoff`)
- G-V Graph passthrough (`split_threshold`, `d_create`, `d_evict`, `budget`)
- Noise injection (`noise_schedule`, `noise_seed`, `noise_batch_size`)
- Output control (`per_sample_scores`)

Methods: `validate() -> Result<(), ConfigErrors>`, `Default` impl.

#### 5.2 `NoiseSchedule` · `sec:sentinel:apiplan-noise-schedule`

Depth-tiered noise injection schedule.

```rust
pub enum NoiseSchedule {
    Geometric { root: u32, decay: f64, min: u32 },
    Explicit(Vec<u32>),
}
```

Methods: `geometric()`, `rounds_for_depth()`, `is_disabled()`, `max_rounds()`, `Default`.

#### 5.3 `SpectralSentinel<C, V, N>` · `sec:sentinel:apiplan-spectral-sentinel`

The main orchestrator. Generic over coordinate `C`, accumulator `V`, domain width `N`.

**Method index:**

| Method                   | Category      | Cost                | One-liner                                     |
| ------------------------ | ------------- | ------------------- | --------------------------------------------- |
| `new`                    | Construction  | $O(w)$ noise        | Validated config → warmed root tracker        |
| `ingest`                 | Observation   | $O(n × \text{obs})$ | Batch processing, returns `BatchReport`       |
| `decay`                  | Temporal      | $O(\|G\|)$          | Global importance attenuation                 |
| `decay_subtree`          | Temporal      | $O(\|G_{sub}\|)$    | Subtree importance attenuation                |
| `reset`                  | Lifecycle     | $O(w)$              | Re-initialise to fresh state                  |
| `health`                 | Diagnostic    | $O(\|cells\|)$      | Operational health snapshot                   |
| `config`                 | Accessor      | $O(1)$              | Read-only config                              |
| `graph`                  | Accessor      | $O(1)$              | Read-only graph                               |
| `analysis_set`           | Accessor      | $O(1)$              | Current analysis set                          |
| `cells_tracked`          | Accessor      | $O(1)$              | Number of active trackers                     |
| `lifetime_observations`  | Accessor      | $O(1)$              | Total real observations                       |
| `degenerate_cells_skipped` | Accessor    | $O(1)$              | Cells excluded for width < 2                  |
| `cell_gnodes`            | Accessor      | $O(\|cells\|)$      | List all tracked cell handles                 |

#### 5.4 `AnalysisSet<C, V>` and `AnalysisEntry<C, V>` · `sec:sentinel:apiplan-analysis-set-and-entry`

Analysis set management (module `analysis_set`). Document:
- Competitive selection (top-K by importance)
- Ancestor closure
- `AnalysisEntry` fields
- `AnalysisSet` methods: `recompute()`, `competitive()`, `full()`, `contains()`, `is_competitive()`, `summary()`

### §6 Surface 3 — Internal Machinery · `sec:sentinel:apiplan-internal-machinery`

`pub(crate)` modules not part of the public API:

| Module              | Contents                                         |
| ------------------- | ------------------------------------------------ |
| `sentinel::tracker` | `SubspaceTracker` — SVD-based subspace model     |
| `sentinel::cusum`   | CUSUM accumulator                                |
| `sentinel::staging` | Deferred warm-up staging area (ADR-S-019)        |
| `sentinel::warming_thread` | Background noise injection (§ALGO S-18.2) |
| `maths`             | SVD, matrix ops, Gamma distribution              |
| `ewma`              | `EwmaStats` — exponential moving average         |
| `observation`       | `CentredBits`, `CentredBitSource`                |

### §7 Cross-Cutting Concerns · `sec:sentinel:apiplan-cross-cutting-concerns`

#### 7.1 Error Handling · `sec:sentinel:apiplan-error-handling`

- `SentinelConfig::validate()` returns `Result<(), ConfigErrors>`.
- `SpectralSentinel::new()` propagates validation errors.
- Panics for stale handles (documented per-method).

#### 7.2 Thread Safety · `sec:sentinel:apiplan-thread-safety`

- `SpectralSentinel` is **not** `Sync` due to internal `Mutex<StagingArea>`.
- `BatchReport` and all report types are `Send + Sync`.
- Background warming thread (when enabled) runs independently.

#### 7.3 Feature Gates · `sec:sentinel:apiplan-feature-gates`

| Feature | Default | Effect                                  |
| ------- | ------- | --------------------------------------- |
| `serde` | off     | `Serialize`/`Deserialize` on all types  |

#### 7.4 Serde · `sec:sentinel:apiplan-serde`

All Surface 1 types carry conditional serde derives. `SentinelConfig` and `NoiseSchedule` likewise. Round-trip stability documented.

---

## 3. Cross-References · `sec:sentinel:apiplan-cross-references`

| Target                | Format               | Example                              |
| --------------------- | -------------------- | ------------------------------------ |
| algorithm.md sections | `§ALGO S-N.M`        | `§ALGO S-9.1`                        |
| mudlark api.md        | `§API M-N`           | `§API M-4.1` (Cell)                  |
| mudlark idea.md       | `§IDEA M-N`          | `§IDEA M-5.5`                        |
| ADRs                  | `ADR-S-NNN`          | `ADR-S-002` (feed-forward)           |
| mudlark ADRs          | `ADR-M-NNN`          | `ADR-M-040` (GNodeId)                |

---

## 4. Tasks · `sec:sentinel:apiplan-tasks`

1. [ ] Draft §1–§2 (principles, architecture) — lift from algorithm.md.
2. [ ] Draft §3 (re-exports) — survey lib.rs.
3. [ ] Draft §4 (report types) — enumerate report.rs structs with field tables.
4. [ ] Draft §5 (operational types) — config.rs, sentinel/mod.rs public methods.
5. [ ] Draft §6 (internal) — one-liner per pub(crate) module.
6. [ ] Draft §7 (cross-cutting) — error handling, serde, thread safety.
7. [ ] Add cross-reference anchors `§SPEC S-N` for external citation.
8. [ ] Review for consistency with mudlark api.md style:
   - Method index tables with cost annotations.
   - Rust code blocks for struct/enum definitions.
   - Design notes in blockquotes.
9. [ ] Ensure all public items are documented.
10. [ ] Final review against Rust doc comments for accuracy.

---

## 5. Open Questions · `sec:sentinel:apiplan-open-questions`

1. **Module visibility:** Should `analysis_set` types be in §4 (report-like) or §5 (operational)? Currently proposed for §5 since they're mutable and tied to sentinel lifecycle.

2. **Coordination tier depth:** How much detail on `CoordinationReport` and the 4D meta-axes? algorithm.md §9 has the full story.

3. **Tracker internals exposure:** The doc currently marks `SubspaceTracker` as internal. If any methods become `pub`, they'd move to §5.

---

## 6. Estimated Scope · `sec:sentinel:apiplan-estimated-scope`

| Section | Lines (est.) |
| ------- | ------------ |
| §1–§2   | 100          |
| §3      | 50           |
| §4      | 400          |
| §5      | 500          |
| §6      | 50           |
| §7      | 100          |
| **Total** | **~1200**  |

Comparable to mudlark api.md (~1600 lines), accounting for sentinel's smaller public surface.
