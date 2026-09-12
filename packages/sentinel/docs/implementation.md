# Sentinel — Implementation Guide · `guide:sentinel:implementation-guide`

> **Cross-reference label:** `§IMPL` — e.g. `§IMPL S-3.1` refers to §3.1 of this document. See AGENTS.md for all conventions.

This document describes how the Spectral Sentinel is implemented. It is an evergreen companion to the algorithm specification ([algorithm.md](algorithm.md)) — the spec says _what_; this document says _where_ and _how_.

Architecture Decision Records live in [`../adr/`](../adr/).

---

## Table of Contents · `sec:sentinel:implementation-table-of-contents`

1. [Source Layout](#1-source-layout)
2. [Architecture Overview](#2-architecture-overview)
3. [The Core Loop](#3-the-core-loop)
4. [Scoring Axes](#4-scoring-axes)
5. [Baseline Tracking](#5-baseline-tracking)
6. [Analysis Selector](#6-analysis-selector)
7. [Hierarchical Coordination](#7-hierarchical-coordination)
8. [Noise Injection and Warm-Up](#8-noise-injection-and-warm-up)
9. [SVD Strategy](#9-svd-strategy)
10. [Configuration Reference](#10-configuration-reference)
11. [Report Types](#11-report-types)
12. [Design Decisions](#12-design-decisions)
13. [Cross-Reference Conventions](#13-cross-reference-conventions)

---

## 1. Source Layout · `sec:sentinel:implementation-source-layout`

| File                                                                | Role                                                                       |
| ------------------------------------------------------------------- | -------------------------------------------------------------------------- |
| [src/lib.rs](../src/lib.rs)                                         | Crate root, public module structure                                        |
| [src/config.rs](../src/config.rs)                                   | `SentinelConfig`, `NoiseSchedule`, validation, `ConfigError`, `ConfigWarning` |
| [src/ewma.rs](../src/ewma.rs)                                       | `EwmaStats` — EWMA mean/variance with outlier clip                         |
| [src/observation.rs](../src/observation.rs)                         | `CentredBits`, suffix extraction                                           |
| [src/analysis_set.rs](../src/analysis_set.rs)                       | `AnalysisEntry`, `AnalysisSet`, `recompute()` — Layer 2                    |
| [src/report.rs](../src/report.rs)                                   | All report/snapshot types                                                  |
| [src/sentinel/mod.rs](../src/sentinel/mod.rs)                       | `SpectralSentinel` orchestrator                                            |
| [src/sentinel/tracker.rs](../src/sentinel/tracker.rs)               | `SubspaceTracker` — core SVD engine                                        |
| [src/sentinel/cusum.rs](../src/sentinel/cusum.rs)                   | `CusumAccumulator` — one-sided Page's test                                 |
| [src/sentinel/staging.rs](../src/sentinel/staging.rs)               | `WarmingCell`, `StagingArea` — deferred warm-up (§ALGO S-18.2)             |
| [src/sentinel/warming_thread.rs](../src/sentinel/warming_thread.rs) | Background warming thread (§ALGO S-18.2 Step 3)                            |
| [src/maths/mod.rs](../src/maths/mod.rs)                             | SVD strategy dispatch, `SvdStrategy` enum                                  |
| [src/maths/brand_svd.rs](../src/maths/brand_svd.rs)                 | Brand's incremental SVD ([ADR-S-016](../adr/016-brand-incremental-svd.md)) |
| [src/maths/naive_svd.rs](../src/maths/naive_svd.rs)                 | Dense thin SVD baseline                                                    |
| [src/maths/bench_tracing.rs](../src/maths/bench_tracing.rs)         | Lightweight span-timing layer for convergence benchmarks                   |

### 1.1 Dependencies · `sec:sentinel:implementation-source-layout-dependencies`

| Crate                 | Purpose                                                       |
| --------------------- | ------------------------------------------------------------- |
| `torrust-mudlark`     | G-V Graph (Layer 1 spatial substrate)                         |
| `faer`                | Linear algebra (SVD, matrix operations)                       |
| `rand` / `rand_distr` | Noise generation, Gamma distribution for coordination warming |
| `serde` (optional)    | Serialisation of config and report types                      |
| `tracing`             | Structured diagnostics                                        |

### 1.2 Test Layout · `sec:sentinel:implementation-test-layout`

Unit tests live in `src/tests/` (crate-level) and integration tests in `tests/` (package-level). Shared helpers live in `tests/common/`.

#### Integration tests (`tests/`) · `sec:sentinel:implementation-integration-tests`

| File                                 | Coverage                                                                    |
| ------------------------------------ | --------------------------------------------------------------------------- |
| `tests/integration.rs`               | End-to-end `ingest()` with realistic value streams                          |
| `tests/invariants.rs`                | Structural invariants: feed-forward, constant-norm, rank bounds             |
| `tests/hierarchical_coordination.rs` | Coordination tree assembly and scoring                                      |
| `tests/deferred_warmup.rs`           | Staging area, background warming, promotion                                 |
| `tests/spray_resistance.rs`          | Budget enforcement under adversarial spray                                  |
| `tests/determinism.rs`               | Reproducibility given fixed seed                                            |
| `tests/serde_roundtrip.rs`           | Config/report serialisation round-trips                                     |
| `tests/ancestor_chain.rs`            | Multi-scale ancestor chain properties (§ALGO S-4.4–4.9)                     |
| `tests/api.rs`                       | Public API contract tests for `SpectralSentinel`                            |
| `tests/clip_pressure.rs`             | Clip-pressure EWMA integration (§ALGO S-6.4)                                |
| `tests/coverage_matrix.rs`           | Six attack modalities from the coverage matrix (§ALGO S-17.6)               |
| `tests/edge_cases.rs`                | Edge-case and boundary-condition tests                                      |
| `tests/graph_routing.rs`             | Graph routing: `ingest()` feeds the G-V Graph                               |
| `tests/health.rs`                    | `HealthReport` behavioural tests                                            |
| `tests/noise.rs`                     | Automatic noise injection lifecycle (§ALGO S-11)                            |
| `tests/report_structure.rs`          | `BatchReport` structure and field contracts                                 |
| `tests/sentinel_u64.rs`              | 64-bit sentinel (`Sentinel64`) end-to-end path                              |
| `tests/spatial_decay.rs`             | Spatial decay via `decay()` / `decay_subtree()`                             |
| `tests/suffix_analysis.rs`           | Suffix analysis (§ALGO S-3.2)                                               |
| `tests/warm_up.rs`                   | Four-stage warm-up sequence (§ALGO S-11.6)                                  |
| `tests/common/`                      | Shared builders, assertions, config presets, generators                     |

#### Crate tests (`src/tests/`) · `sec:sentinel:implementation-crate-tests`

| File                                     | Coverage                                                                                            |
| ---------------------------------------- | --------------------------------------------------------------------------------------------------- |
| `src/tests/analysis_set.rs`              | `AnalysisSet` selection pipeline (§ALGO S-8.1–8.3)                                                  |
| `src/tests/config.rs`                    | Config validation, defaults, `NoiseSchedule` helpers                                                |
| `src/tests/convergence_fixes.rs`         | Regression tests for warm-up convergence ([ADR-S-013](../adr/013-warm-up-convergence-benchmark.md)) |
| `src/tests/convergence_clipping.rs`      | Clipping stability audit (fast EWMA clip, graduated exemption, slow EWMA/CUSUM clip)                |
| `src/tests/convergence_common.rs`        | Shared infrastructure for convergence tests                                                         |
| `src/tests/convergence_diagnostics.rs`   | On-demand convergence diagnostics (run with `--ignored`)                                            |
| `src/tests/convergence_eta.rs`           | Noise influence η tracking and maturity counters                                                    |
| `src/tests/convergence_ewma.rs`          | Pure EWMA convergence properties (isolated from tracker)                                            |
| `src/tests/convergence_noise.rs`         | Tracker-level noise-baseline convergence                                                            |
| `src/tests/cusum.rs`                     | `CusumAccumulator` tests                                                                           |
| `src/tests/ewma.rs`                      | `EwmaStats` unit tests                                                                             |
| `src/tests/observation.rs`               | Centred-bit representation tests                                                                    |
| `src/tests/report.rs`                    | Report types (construction, field contracts)                                                        |
| `src/tests/tracker.rs`                   | `SubspaceTracker` (construction, scoring, rank, CUSUM, clip-pressure)                               |
| `src/tests/variance_formula.rs`          | EWMA-mean-centred variance formula ([ADR-S-021](../adr/021-ewma-mean-centred-variance.md))          |

---

## 2. Architecture Overview · `sec:sentinel:implementation-architecture-overview`

The sentinel implements the three-layer architecture (§ALGO S-1.1):

```
Layer 1: GvGraph<C, V, N>            ← torrust-mudlark
         Adaptive spatial partitioning of [0, 2^N)
         Competitive ranking by observation volume
              │
              │  V-Tree depth ≤ cutoff → top-K selection
              ▼
Layer 2: AnalysisSet                  ← analysis_set.rs
         Picks competitive cells, closes under G-ancestry
              │
              │  suffix bit vectors at every ancestor depth
              ▼
Layer 3: SubspaceTracker fleet        ← sentinel/tracker.rs
         + Hierarchical coordination    ← sentinel/mod.rs
              │
              ▼
         BatchReport<C> → host
```

### 2.1 The Spatial Substrate (Layer 1) · `sec:sentinel:implementation-architecture-spatial-substrate`

The sentinel owns a `GvGraph<C, V, N>` ([ADR-S-018](../adr/018-generic-domain-parameters.md)):

- **`C`** coordinate type — spatial addressing (e.g. `u128`, `u64`).
- **`V`** accumulator — observation counts ($\Delta = 1$ per value).
- **`N`** bit-width — domain resolution.

The default instantiation (`Sentinel128`) uses `GvGraph<u128, u64, 128>`; `Sentinel64` uses `GvGraph<u64, u64, 64>`.

The graph is mutated only through `observe(coord, 1)` during `ingest()` and through `decay()` / `decay_subtree()` when the host requests temporal decay. Anomaly scores never feed back into the graph's importance signal — this is the **feed-forward invariant** ([ADR-S-002](../adr/002-feed-forward-invariant.md)).

### 2.2 The Analysis Selector (Layer 2) · `sec:sentinel:implementation-architecture-analysis-selector`

`AnalysisSet::recompute()` runs after every G-V Graph observation pass and selects up to $K$ **competitive targets** ($\mathcal{T}$) from the V-Tree by importance, filtered by a depth cutoff $L$ (§ALGO S-8.1). Ties break by G-node interval left endpoint for deterministic, spatially stable ordering.

The V-Tree scan uses `graph.layers_to(depth_cutoff)` (ADR-M-041), which limits the BFS to V-depths $\leq L$ and avoids expanding structural nodes below the cutoff — saving exponential work on deep trees compared to a full `layers()` plus post-hoc filter.

The competitive targets are then closed under G-tree ancestry (§ALGO S-8.2), forming the **investment set** ($\mathcal{I}$) — the set of all cells with allocated trackers. Every member has a tracker regardless of online status, guaranteeing a complete chain from every competitive target to the root.

The investment set is reconciled against the live `cells` map and the `StagingArea`: entering cells are created and enqueued for warm-up, exiting cells are destroyed (eager removal, §ALGO S-8.5). The online subset of the investment set forms the **producing sets**: $\mathcal{A}$ (online competitive targets) and $\mathcal{A}^*$ (all online members). The full recompute runs from scratch each time ([ADR-S-006](../adr/006-analysis-set-recomputation.md), [ADR-S-019](../adr/019-investment-set-terminology-and-reporting.md)).

### 2.3 The Analysis Engine (Layer 3) · `sec:sentinel:implementation-architecture-analysis-engine`

One `SubspaceTracker` per cell in the investment set $\mathcal{I}$. Each tracker analyses the **suffix** bits `[d, N)` with width $w = N - d$ (§ALGO S-3.2). Cells with $w < 2$ are excluded ([ADR-S-011](../adr/011-degenerate-cell-dimension-guard.md)). Only online trackers (the producing full set $\mathcal{A}^*$) score observations; warming trackers receive only synthetic noise.

The root tracker at depth 0 ($w = N$) is permanent — never destroyed (§ALGO S-8.4).

### 2.4 Determinism · `sec:sentinel:implementation-architecture-determinism`

All collection types use `BTreeMap` for deterministic iteration order ([ADR-S-005](../adr/005-deterministic-order-and-thread-safety.md)). Given a fixed `noise_seed`, the sentinel is fully reproducible with `background_warming` disabled and on a fixed build — one target and one set of dependency versions; the generator behind the noise is chosen for speed and is portable across neither. Under background warming the same seed and the same traffic still give the same graph, the same investment set and the same ascending-handle report order, but neither the baselines a tracker starts from nor the ingest cycle on which it first scores: the warming worker draws from its own generator and takes whichever staged cell leads on volume when it looks. `SpectralSentinel<C, V, N>` is `Send + Sync`.

---

## 3. The Core Loop · `sec:sentinel:implementation-core-loop`

`SubspaceTracker::observe()` implements the five-phase core loop (§ALGO S-4.2) with a **score-before-evolve** invariant: scores are computed against the current basis, then the basis is updated.

| Phase | Operation                                                                                                                                               | §ALGO S-     |
| ----- | ------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------ |
| 1     | **Project** — $Z = X U$, $\hat{X} = Z U^\top$, $R = X - \hat{X}$                                                                                        | §4.2 Phase 1 |
| 2     | **Evolve subspace** — combined matrix $M$, thin SVD, update $U$, $\sigma$                                                                               | §4.2 Phase 2 |
| 3     | **Evolve latent distribution** — EWMA-mean-centred update of $\mu^{(z)}$, $\nu^{(z)}$, $\Gamma$ ([ADR-S-021](../adr/021-ewma-mean-centred-variance.md)) | §4.2 Phase 3 |
| 4     | **Score** — compute all four axis scores, update EWMA baselines, CUSUM                                                                                  | §4.2 Phase 4 |
| 5     | **Adapt rank** — energy-threshold selection with +1 buffer, ±1 oscillation guard                                                                        | §4.2 Phase 5 |

Safety guards:

- **SVD failure** — falls back to identity if SVD does not converge.
- **Identical observations** — handled without numerical degeneracy.
- **Coherence lifecycle** — undefined at $k < 2$; baselines destroyed on rank drop (§ALGO S-6.4).
- **Latent cold→warm initialisation** — on the first batch, `lat_mean`, `lat_var`, and `cross_corr` are seeded directly from data ([ADR-S-013](../adr/013-warm-up-convergence-benchmark.md)).
- **EWMA-mean-centred variance** — variance centres on the EWMA mean rather than the batch mean, eliminating batch-size-dependent bias ([ADR-S-021](../adr/021-ewma-mean-centred-variance.md)).

---

## 4. Scoring Axes · `sec:sentinel:implementation-scoring-axes`

All four axes satisfy the polarity invariant: **higher = more anomalous** (§ALGO S-6).

| Axis         | Formula                              | Bounds        | §ALGO S- |
| ------------ | ------------------------------------ | ------------- | -------- |
| Novelty      | $\|r\|^2 / (d - k)$                  | $[0, \infty)$ | §6.1     |
| Displacement | $\|z\|^2 / (k + \|z\|^2)$            | $[0, 1)$      | §6.2     |
| Surprise     | diagonal Mahalanobis $/\, k$         | $[0, \infty)$ | §6.3     |
| Coherence    | pairwise cross-correlation deviation | $[0, \infty)$ | §6.4     |

Each raw score is transformed into a z-score via:

$$z = \frac{s - \bar{s}}{\sqrt{\bar{v}} + \varepsilon}$$

where $\bar{s}$ and $\bar{v}$ are the fast EWMA mean and variance (§ALGO S-7.1.2). The variance floor is $10^{-4}$.

---

## 5. Baseline Tracking · `sec:sentinel:implementation-baseline-tracking`

### 5.1 Fast EWMA (§ALGO S-7.1) · `sec:sentinel:implementation-fast-ewma`

Per-axis `EwmaStats` with configurable `clip_sigmas`. Observations beyond $\bar{s} + n_\sigma^{\text{eff}} \sqrt{\bar{v}}$ are clipped (upper-tail only) to prevent baseline poisoning.

**Graduated clip-exemption** ([ADR-S-013](../adr/013-warm-up-convergence-benchmark.md)): during warm-up the effective clip width scales with the noise-influence fraction $\eta$:

$$n_\sigma^{\text{eff}} = n_\sigma + n_\sigma \cdot \frac{\eta}{1 - \eta + \varepsilon}$$

This widens the ceiling while baselines are immature, eliminating the clipping-ceiling positive feedback loop that previously caused 5–10× slower convergence.

### 5.2 Slow EWMA (§ALGO S-7.2) · `sec:sentinel:implementation-slow-ewma`

A secondary EWMA with decay factor `cusum_slow_decay` (default 0.999, half-life ≈ 693 steps) provides the reference baseline for CUSUM drift detection.

### 5.3 CUSUM Drift Detection (§ALGO S-7.3) · `sec:sentinel:implementation-cusum-drift-detection`

`CusumAccumulator` implements one-sided Page's test with:

- Compute-before-update ordering.
- Noise allowance: $\kappa_\sigma \cdot \sqrt{v_{\text{slow}}}$.
- Reset after noise injection (§ALGO S-7.4).
- Slow EWMA seeded from fast EWMA at noise→real transition ([ADR-S-013](../adr/013-warm-up-convergence-benchmark.md)).

---

## 6. Analysis Selector · `sec:sentinel:implementation-analysis-selector`

`AnalysisSet` in `analysis_set.rs` implements the Layer 2 selection pipeline (§ALGO S-8):

1. **Enumerate** all V-entries with V-depth $\leq L$.
2. **Rank** by importance, take top $K$ — the **competitive targets** $\mathcal{T}$ (§ALGO S-8.1).
3. **Break ties** by interval start (deterministic, spatially stable).
4. **Close** under G-tree ancestry by walking parent pointers — forming the **investment set** $\mathcal{I}$ (§ALGO S-8.2).

The root tracker is permanent and never participates in competitive selection (§ALGO S-8.4). Reconciliation after each observation pass reconciles the investment set: entering cells are created and enqueued for warm-up, exiting cells are eagerly removed (§ALGO S-8.5, [ADR-S-019](../adr/019-investment-set-terminology-and-reporting.md)).

---

## 7. Hierarchical Coordination · `sec:sentinel:implementation-hierarchical-coordination`

The sentinel detects coordinated anomalies across cells using hierarchical G-tree coordination (§ALGO S-9).

### 7.1 Coordination Contexts · `sec:sentinel:implementation-coordination-contexts`

One `CoordContext` per internal G-node whose left and right subtrees both contribute competitive cells. Each context owns:

- A 4-dimensional `SubspaceTracker` (one dimension per scoring axis).
- A running-mean centring reference $\mu^{(\text{in})}$ for de-meaning the input signal before feeding (§ALGO S-9.3).

### 7.2 Bottom-Up Assembly (§ALGO S-9.4) · `sec:sentinel:implementation-coordination-bottom-up-assembly`

After cell scoring, the coordination tier assembles score vectors bottom-up through the G-tree:

1. Leaf contributions: competitive cells emit their 4D centred score vector.
2. Internal nodes: when both subtrees contribute, the assembled matrix is fed to the coordination tracker.
3. The walk is pruned — only nodes reachable through subtrees with competitive cells are visited.

### 7.3 Lifecycle · `sec:sentinel:implementation-coordination-lifecycle`

`CoordContext` instances are created on first fire and pruned when their subtree loses all competitive cells. There is no manual API — lifecycle is fully automatic.

### 7.4 Coordination Warm-Up (§ALGO S-9.8) · `sec:sentinel:implementation-coordination-warmup`

Coordination contexts are warmed with Gamma-sampled synthetic score vectors. This happens during the chained coordination warming phase (§ALGO S-11.4) whenever a new tracker is warmed — noise scores flow through the coordination tree just as real scores do.

---

## 8. Noise Injection and Warm-Up · `sec:sentinel:implementation-noise-injection-and-warmup`

### 8.1 Noise Generation (§ALGO S-11.1) · `sec:sentinel:implementation-noise-generation`

Synthetic noise vectors are uniform $\pm 0.5$ centred bit vectors matching the `CentredBits` encoding. A persistent `SmallRng` seeded from `noise_seed` (or system entropy) generates all noise sequences.

### 8.2 Depth-Tiered Schedule ([ADR-S-015](../adr/015-cell-creation-performance.md)) · `sec:sentinel:implementation-depth-tiered-noise-schedule`

`NoiseSchedule` replaces the earlier flat `noise_rounds` parameter. Deeper cells are narrower and need fewer rounds to converge:

- **Geometric** (default): `root` rounds at depth 0, decaying by `decay` per depth level, clamped to `min`. Default: `Geometric { root: 450, decay: 0.5, min: 50 }`.
- **Explicit**: a lookup table of per-depth round counts.

### 8.3 Automatic Injection · `sec:sentinel:implementation-automatic-noise-injection`

There is no manual noise API — the sentinel owns the injection lifecycle entirely ([ADR-S-007](../adr/007-automatic-noise-injection.md)). Every newly created tracker is warmed before it receives real observations. The root tracker is warmed at construction.

### 8.4 Deferred Cell Warm-Up (§ALGO S-11.6, §ALGO S-18.2) · `sec:sentinel:implementation-deferred-cell-warmup`

Cell warm-up is decoupled from the `ingest()` hot path to bound per-call work variance:

1. **Enqueue** — `reconcile_analysis_set()` creates a `CellState` and enqueues it into the `StagingArea` with the target round count from `NoiseSchedule`. The cell holds an **investment slot** in $\mathcal{I}$ but not a production slot in $\mathcal{A}$.
2. **Warm** — the background thread (or synchronous drain) picks the highest-priority cell by g.sum (§ALGO S-11.6.2) and injects one noise batch at a time. The g.sum ordering ensures ancestors come online before descendants.
3. **Promote** — completed cells are moved to the ready queue and transferred into the live `cells` map at the start of the next `ingest()` call, entering the producing set.

The staging area lives behind `Arc<Mutex<StagingArea>>` for sharing with the background warming thread.

### 8.5 Background Warming Thread (§ALGO S-18.2 Step 3) · `sec:sentinel:implementation-background-warming-thread`

When `config.background_warming` is `true`, a dedicated thread runs the warm-up loop:

- Owns its own `SmallRng` seeded from `noise_seed + 1`.
- Takes cells from the staging area via `take_highest_priority()`, injects noise _without holding the lock_, and returns them via `finish_warming()`.
- Sleeps on a condvar when there is nothing to warm.
- The main thread notifies the condvar after enqueueing new cells.
- Clean shutdown via `WarmingThreadHandle::shutdown()`.

When `background_warming` is `false` (default), warm-up runs synchronously inside `reconcile_analysis_set()`. This mode is deterministic and used by the test suite.

### 8.6 Convergence Fixes ([ADR-S-013](../adr/013-warm-up-convergence-benchmark.md)) · `sec:sentinel:implementation-warmup-convergence-fixes`

Three interacting fixes address a 5–10× convergence gap between theoretical and empirical EWMA baseline convergence:

| Fix                          | Mechanism                                                                                   | Impact                                   |
| ---------------------------- | ------------------------------------------------------------------------------------------- | ---------------------------------------- |
| Graduated clip-exemption     | $n_\sigma^{\text{eff}}$ scales with $\eta$                                                  | Surprise convergence: 1000+ → 65 rounds  |
| Latent cold→warm init        | First-batch seeding of `lat_mean`, `lat_var`, `cross_corr`                                  | Rise factor: 5.9× → 1.26×                |
| Slow-from-fast CUSUM seeding | Copy fast EWMA into slow at noise→real transition                                           | False drift: 198 → 5.7                   |
| EWMA-mean-centred variance   | Centre on EWMA mean, not batch mean ([ADR-S-021](../adr/021-ewma-mean-centred-variance.md)) | Worst-case convergence: 289 → 282 rounds |

Regression tests in `src/tests/convergence_fixes.rs` and `src/tests/variance_formula.rs`.

### 8.7 Maturity Tracking (§ALGO S-11.5) · `sec:sentinel:implementation-warmup-maturity-tracking`

Each tracker maintains a noise-influence fraction $\eta \in [0, 1]$ that decays toward 0 as real observations replace synthetic noise. Batch-vectorised via $\lambda^n$.

---

## 9. SVD Strategy · `sec:sentinel:implementation-svd-strategy`

Configurable via `SentinelConfig::svd_strategy` ([ADR-S-016](../adr/016-brand-incremental-svd.md)):

| Strategy          | Algorithm                                                                                      | Complexity                   | Notes                               |
| ----------------- | ---------------------------------------------------------------------------------------------- | ---------------------------- | ----------------------------------- |
| `Naive`           | Dense thin SVD of $M \in \mathbb{R}^{w \times (k+b)}$                                          | $O(w \cdot (k+b)^2)$         | Simple, numerically stable baseline |
| `Brand` (default) | Incremental SVD (Brand 2006) — projects onto current basis, SVDs a $(k+b) \times (k+b)$ kernel | $O(w \cdot (k+b) + (k+b)^3)$ | ~2–3× faster; default               |

In **debug builds**, both algorithms run regardless of the setting and their outputs are compared — a continuous oracle test.

---

## 10. Configuration Reference · `sec:sentinel:implementation-configuration-reference`

All parameters from the algorithm spec (§ALGO S-13) are represented in `SentinelConfig<V>` (generic over the accumulator type for `split_threshold`; see [ADR-S-018](../adr/018-generic-domain-parameters.md)). Defaults match the spec.

### 10.1 Analysis Engine (§ALGO S-13.1) · `sec:sentinel:implementation-configuration-analysis-engine`

| Config field             | Default | Description                                            |
| ------------------------ | ------- | ------------------------------------------------------ |
| `max_rank`               | 16      | Maximum subspace rank per tracker                      |
| `forgetting_factor`      | 0.99    | Exponential decay factor $\lambda$                     |
| `rank_update_interval`   | 100     | Steps between rank re-evaluations                      |
| `energy_threshold`       | 0.90    | Cumulative energy fraction for rank selection          |
| `eps`                    | 1e-6    | Numerical stability constant                           |
| `clip_sigmas`            | 3.0     | EWMA outlier clip width ($n_\sigma$)                   |
| `clip_pressure_decay`    | 0.95    | Clip-pressure EWMA decay ($\lambda_\rho$, §ALGO S-6.4) |
| `cusum_slow_decay`       | 0.999   | Slow EWMA decay for per-tracker CUSUM                  |
| `cusum_coord_slow_decay` | 0.999   | Slow EWMA decay for coordination CUSUM                 |
| `cusum_allowance_sigmas` | 0.5     | CUSUM noise allowance ($\kappa_\sigma$)                |
| `per_sample_scores`      | false   | Include per-observation scores in reports              |
| `svd_strategy`           | Brand   | SVD algorithm selection                                |

### 10.2 Analysis Selector (§ALGO S-13.2) · `sec:sentinel:implementation-configuration-analysis-selector`

| Config field            | Default | Description                     |
| ----------------------- | ------- | ------------------------------- |
| `analysis_k`            | 1024    | Maximum competitive cells ($K$) |
| `analysis_depth_cutoff` | 6       | V-Tree depth cutoff ($L$)       |

### 10.3 G-V Graph (§ALGO S-13.3) · `sec:sentinel:implementation-configuration-gv-graph`

| Config field      | Default | Description                                          |
| ----------------- | ------- | ---------------------------------------------------- |
| `split_threshold` | 100     | Minimum volume before subdivision                    |
| `d_create`        | 3       | Maximum V-depth for new splits ($D_{\text{create}}$) |
| `d_evict`         | 6       | Minimum V-depth for eviction ($D_{\text{evict}}$)    |
| `budget`          | 100,000 | Hard ceiling on live G-nodes ($G_{\max}$)            |

### 10.4 Noise Injection (§ALGO S-13.5) · `sec:sentinel:implementation-configuration-noise-injection`

| Config field         | Default                                        | Description                                      |
| -------------------- | ---------------------------------------------- | ------------------------------------------------ |
| `noise_schedule`     | `Geometric { root: 450, decay: 0.5, min: 50 }` | Depth-tiered round counts                        |
| `noise_batch_size`   | 16                                             | Samples per noise round                          |
| `noise_seed`         | Some(42)                                       | Deterministic RNG seed (`None` = system entropy) |
| `background_warming` | false                                          | Warm cells on a background thread                |

### 10.5 Validation · `sec:sentinel:implementation-configuration-validation`

`SentinelConfig::validate()` checks all constraints (ranges, inter-parameter relationships, mudlark headroom) before construction. Invalid configs produce `ConfigErrors` — the sentinel never panics on bad config ([ADR-S-004](../adr/004-config-validation-over-panic.md)).

---

## 11. Report Types · `sec:sentinel:implementation-report-types`

`SpectralSentinel::ingest()` returns a `BatchReport<C>` containing everything the host needs to assess the batch.

### 11.1 Top-Level Report · `sec:sentinel:implementation-top-level-report`

| Field                  | Type                         | Content                                                 |
| ---------------------- | ---------------------------- | ------------------------------------------------------- |
| `cell_reports`         | `Vec<CellReport<C>>`         | Competitive cells only                                                                |
| `ancestor_reports`     | `Vec<CellReport<C>>`         | Non-competitive ancestors + root                                                      |
| `coordination_reports` | `Vec<CoordinationReport<C>>` | Per-G-node hierarchical coordination                                                  |
| `contour`              | `ContourSnapshot`            | Plateau count, cell count, total importance                                           |
| `health`               | `HealthReport`               | Fleet-wide operational summary                                                        |
| `analysis_set_summary` | `AnalysisSetSummary`         | Competitive/full/investment-set sizes, depth/importance/V-depth ranges, degenerate count |
| `oldest_observation_age_micros` | `Option<u64>`       | Age of the batch's oldest observation at report emission, in microseconds on the sentinel's own monotonic clock; absent where there is none |

The age is stamped in `ingest()` at the call boundary, immediately after the empty-input early return, and read back as the last field of the report literal so that it covers the whole of the call's work. `empty_report()` reports it absent. The reading saturates at `u64::MAX` rather than wrapping (`sec:sentinel:algorithm-output-observation-age`).

### 11.2 Cell-Level Reports · `sec:sentinel:implementation-cell-level-reports`

| Type                | Content                                                                                                          |
| ------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `CellReport<C>`     | Wraps a `TrackerReport` with cell identity (GNode, depth, interval, competitive flag)                            |
| `TrackerReport`     | `AnomalyScores`, four `ScoreDistribution` axes, `TrackerMaturity`, `ScoringGeometry`, optional per-sample scores |
| `AnomalyScores`     | Batch-mean raw score and z-score for each of the four axes                                                       |
| `ScoreDistribution` | `BaselineSnapshot` (EWMA mean/var) + `CusumSnapshot` (accumulator, slow mean/var) + `clip_pressure` (ρ̄)         |
| `TrackerMaturity`   | Noise influence $\eta$, total observations, noise observation count                                              |
| `ScoringGeometry`   | Novelty saturation ratio, coherence activity flag ([ADR-S-008](../adr/008-scoring-geometry-extension.md))        |

### 11.3 Coordination Reports · `sec:sentinel:implementation-coordination-reports`

| Type                    | Content                                                                                                        |
| ----------------------- | -------------------------------------------------------------------------------------------------------------- |
| `CoordinationReport<C>` | G-node identity, `TrackerReport` from the 4D coordination tracker, `Option<Vec<MemberScore<C>>>` per contributing cell (requires `per_sample_scores`) |
| `MemberScore<C>`        | Cell identity (start/end/depth) + 8 score fields (4 raw + 4 z-scores)                                          |

### 11.4 System Health · `sec:sentinel:implementation-system-health`

| Type                | Content                                                                                                                                                    |
| ------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `HealthReport`      | Node counts, tracker counts, investment set size, warming counts, `RankDistribution`, `MaturityDistribution`, `GeometryDistribution`, `CoordinationHealth`, `ClipPressureDistribution` |
| `CellInspection<C>` | Deep dive into a single cell via `inspect_cell(gnode)`, includes `AxisBaselineSnapshots`                                                                                               |
| `ContourSnapshot`   | Plateau count, terminal cell count, total importance                                                                                                                                   |

---

## 12. Design Decisions · `sec:sentinel:implementation-design-decisions`

| ADR                                                                 | Title                                  | Summary                                                     |
| ------------------------------------------------------------------- | -------------------------------------- | ----------------------------------------------------------- |
| [ADR-S-001](../adr/001-measures-not-opinions.md)                    | Measures Not Opinions                  | Sentinel emits raw statistics, never policy                 |
| [ADR-S-002](../adr/002-feed-forward-invariant.md)                   | Feed-Forward Invariant                 | Only `observe(v, 1u64)` — scores never feed back            |
| [ADR-S-003](../adr/003-mudlark-integration.md)                      | Mudlark Integration                    | Cargo features; type params superseded by ADR-S-018         |
| [ADR-S-004](../adr/004-config-validation-over-panic.md)             | Config Validation Over Panic           | Pre-validate all constraints; return `ConfigErrors`         |
| [ADR-S-005](../adr/005-deterministic-order-and-thread-safety.md)    | Deterministic Order and Thread Safety  | `BTreeMap`, `Send + Sync`                                   |
| [ADR-S-006](../adr/006-analysis-set-recomputation.md)               | Analysis Set Recomputation             | Full recompute per `ingest()`; $O(n)$ scan                  |
| [ADR-S-007](../adr/007-automatic-noise-injection.md)                | Automatic Noise Injection              | No manual API; sentinel owns lifecycle                      |
| [ADR-S-008](../adr/008-scoring-geometry-extension.md)               | Scoring Geometry Extension             | `ScoringGeometry` for structural observability              |
| [ADR-S-009](../adr/009-decay-does-not-invalidate-analysis-set.md)   | Decay Does Not Invalidate Analysis Set | Decay is host-driven, analysis set reconciled at `ingest()` |
| [ADR-S-010](../adr/010-linear-routing-over-g-tree-descent.md)       | Linear Routing Over G-Tree Descent     | Route by interval containment, not tree traversal           |
| [ADR-S-011](../adr/011-degenerate-cell-dimension-guard.md)          | Degenerate Cell Dimension Guard        | Exclude cells with $w < 2$                                  |
| [ADR-S-012](../adr/012-test-duration-budget.md)                     | Test Duration Budget                   | Time-box test suite                                         |
| [ADR-S-013](../adr/013-warm-up-convergence-benchmark.md)            | Warm-Up Convergence Benchmark          | Three convergence fixes, 7 regression tests                 |
| [ADR-S-014](../adr/014-subspace-tracker-visibility.md)              | Subspace Tracker Visibility            | `pub(crate)` visibility for tracker internals               |
| [ADR-S-015](../adr/015-cell-creation-performance.md)                | Cell Creation Performance              | Depth-tiered `NoiseSchedule`                                |
| [ADR-S-016](../adr/016-brand-incremental-svd.md)                    | Brand's Incremental SVD                | ~2–3× faster subspace evolution                             |
| [ADR-S-017](../adr/017-deferred-cell-warm-up.md)                    | Deferred Cell Warm-Up                  | Staging area + background thread                            |
| [ADR-S-018](../adr/018-generic-domain-parameters.md)                | Generic Domain Parameters              | `C`, `V`, `N` type parameters mirror mudlark                |
| [ADR-S-019](../adr/019-investment-set-terminology-and-reporting.md) | Investment-Set Terminology & Reporting | Investment/producing sets, new health fields                |
| [ADR-S-020](../adr/020-clip-pressure-ewma.md)                       | Clip-Pressure EWMA                     | Per-axis clip-pressure tracking (§ALGO S-6.4)               |
| [ADR-S-021](../adr/021-ewma-mean-centred-variance.md)               | EWMA-Mean-Centred Latent Variance      | Centre on EWMA mean to eliminate batch-size bias            |

---

## 13. Cross-Reference Conventions · `sec:sentinel:implementation-cross-reference-conventions`

Source comments use `§ALGO S-N` to reference sections of [algorithm.md](algorithm.md). Within this document, `§IMPL S-N` references sections here. See AGENTS.md for the full cross-reference system.

When adding new code comments, always use the fully qualified form (`§ALGO S-4.2`, not bare `§4.2`) since the algorithm spec is a separate document.
