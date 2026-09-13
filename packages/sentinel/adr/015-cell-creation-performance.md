# ADR-S-015: Cell Creation Performance · `rec:sentinel:depth-tiered-noise-schedule-bounds-creation-cost`

**Status:** Implemented (§1 `NoiseSchedule`) **Date:** 2026-03-10 **Spec:** §ALGO S-8.2 (analysis set reconciliation), §ALGO S-11.2 (automatic noise injection) **Relates to:** [ADR-S-007](007-automatic-noise-injection.md) (automatic noise injection), [ADR-S-013](013-warm-up-convergence-benchmark.md) (warm-up convergence benchmark), [ADR-S-006](006-analysis-set-recomputation.md) (analysis set recomputation), [ADR-S-012](012-test-duration-budget.md) (test duration budget), [ADR-S-016](016-brand-incremental-svd.md) (Brand's incremental SVD)

## Context · `sec:sentinel:noiseperf-context`

Every time a new cell enters the analysis set — whether at sentinel construction, after `reset()`, or during live `reconcile_analysis_set()` — `inject_noise_into_cell()` runs `noise_rounds` batches of synthetic observations through a fresh `SubspaceTracker`.  This is the mechanism described in ADR-S-007.

The critical realisation is that **cell creation is not a one-time startup cost**.  The G-V graph splits cells as traffic arrives. `reconcile_analysis_set()` runs on every `ingest()` call, and when the analysis set changes, new cells are created — each paying the full noise injection cost.

### Call sites · `sec:sentinel:noiseperf-call-sites`

`inject_noise_into_cell()` is called from three places:

1. **`SpectralSentinel::new()`** — root cell at construction. One-time cost, acceptable.

2. **`SpectralSentinel::reset()`** — root cell after reset. Infrequent, acceptable.

3. **`reconcile_analysis_set()`** — new cells entering the analysis set during live traffic.  **This is the hot-path concern.** Multiple cells may enter in a single `ingest()` call if the graph has split since the last reconciliation.

### Measured costs · `sec:sentinel:noiseperf-measured-costs`

Criterion benchmarks (release profile, optimised) on the sentinel crate, 2026-03-10:

| `noise_rounds` | Config | Construction time | Per-round marginal |
|----------------:|--------|------------------:|-------------------:|
| 10  | bench (rank=4, k=16, b=4)            |   38 ms |  ~2.5 ms |
| 50  | bench                                |  319 ms |    ~7 ms |
| 100 | bench                                |  161 ms |  ~2.5 ms |
| 200 | bench                                | 1.28 s  |    ~5 ms |
| 400 | bench                                | 3.27 s  |    ~7 ms |
| 10  | realistic (rank=16, k=1024, b=16)    |   99 ms |      — |
| 50  | realistic                            |  332 ms |    ~6 ms |
| 100 | realistic                            |  164 ms |    ~2 ms |
| 200 | realistic                            | 3.72 s  |   ~17 ms |
| 400 | realistic                            | 4.92 s  |   ~10 ms |

> **Update (post ADR-S-016):** Brand's incremental SVD reduces construction times by **~1.9×** at b=4 and **~2.9×** at b=16. The decision analysis below uses Brand's-adjusted figures.

Per-round `ingest()` cost on a warmed sentinel (Criterion):

| Config | batch_size | Per-ingest call |
|--------|----------:|--------------:|
| bench        |  4 |   232 µs |
| bench        | 16 |  6.6 ms |
| realistic    |  4 |  4.4 ms |
| realistic    | 16 | 14.7 ms |

> With Brand's SVD (ADR-S-016) these drop to ~122 µs (bench b=4), ~2.3 ms (bench b=16), ~2.3 ms (realistic b=4), ~5.1 ms (realistic b=16).

### Convergence data · `sec:sentinel:noiseperf-convergence-data`

From the convergence benchmark suite (formerly `convergence_benchmark.rs`, now `convergence_diagnostics.rs` / criterion `warmup_cost_detailed`, optimised debug):

**Noise-only convergence (per-axis, rounds needed):**

| noise_rounds | Novelty | Surprise | Displacement (b=16) | Coherence | All converged? |
|---:|----|----|----|----|----|
|   5 | 3  | 3  | 3  | 0  | NO |
|  50 | 21 | 30 | 21 | 0  | NO |
|  65 | 21 | 43 | 21 | 0  | YES (prod) |
| 100 | 21 | 67 | 21 | 0  | YES |
| 200 | 21 | 67 | 33 | 157 | YES |
| 400 | 21 | 188 | 24 | 273 | YES |

**Real-data phase convergence (after 200 noise rounds):**

| Config | Worst axis | Converged at round | Wall-clock |
|--------|-----------|---:|---:|
| test (λ=0.95, b=4) | coherence | 233 | ~2.0 s |
| production (λ=0.99, b=16) | coherence | 212 | ~5.1 s |

**η (noise influence) matches theory exactly:** max $|\eta - \lambda^{bn}| = 2.78 \times 10^{-17}$.

## Problem · `sec:sentinel:noiseperf-problem`

The current defaults are:

| Config | `noise_rounds` | Per-cell cost (pre-Brand) | Per-cell cost (Brand) |
|--------|---:|---:|---:|
| Test | 5 | ~38 ms | ~13 ms |
| Production | 50 | ~332 ms | ~115 ms |

The test default of 5 is **13× too low** for surprise convergence (needs ≥65).  The production default of 50 is **6× too low** for η < 0.05 (needs ≥299 at λ = 0.99).

However, raising `noise_rounds` to 300–400 is still **expensive on the hot path** even with Brand's incremental SVD (ADR-S-016). If the analysis set gains 5 new cells during a single `ingest()`:

| `noise_rounds` | Per-cell (Brand) | 5 cells | Impact on `ingest()` |
|---:|---:|---:|---|
| 50 (current) | ~115 ms | ~575 ms | Tolerable |
| 100 | ~57 ms | ~283 ms | **Well within budget** |
| 200 | ~1.28 s | ~6.4 s | Blocking stall |
| 300 | ~860 ms | ~4.3 s | Significant stall |
| 400 | ~1.7 s | ~8.5 s | Blocking stall |

At `analysis_k = 1024` and `analysis_depth_cutoff = 6`, the analysis set can contain up to 1024 cells.  Graph growth from zero to steady state may create hundreds of cells over time.  Each split that creates a new cell in the analysis set triggers noise injection.

## Decision · `sec:sentinel:noiseperf-decision`

### 1. Depth-tiered `noise_schedule` · `sec:sentinel:noiseperf-depth-tiered-schedule`

Replace the single `noise_rounds` scalar with a **`noise_schedule` enum** that determines per-depth noise rounds.  Two variants:

```rust
enum NoiseSchedule {
    /// Geometric decay: r(d) = max(min, floor(root * decay^d))
    Geometric { root: u32, decay: f64, min: u32 },

    /// Explicit per-depth array; last element repeats for all
    /// deeper layers.
    Explicit(Vec<u32>),  // non-empty, validated at construction
}
```

Resolution at injection time:

```rust
fn rounds_for_depth(&self, depth: usize) -> u32 {
    match self {
        Geometric { root, decay, min } =>
            (*min).max((*root as f64 * decay.powi(depth as i32)) as u32),
        Explicit(v) =>
            v[depth.min(v.len() - 1)],
    }
}
```

#### Default: `Geometric { root: 400, decay: 0.5, min: 100 }` · `sec:sentinel:noiseperf-default-geometric`

$$r(d) = \max\!\bigl(100,\; \lfloor 400 \cdot 0.5^{\,d} \rfloor\bigr)$$

Materialised:

| Depth | Formula | Rounds |
|---:|---:|---:|
| 0 (root) | $400 \cdot 0.5^0$ | **400** |
| 1 | $400 \cdot 0.5^1$ | **200** |
| 2 | $400 \cdot 0.5^2 = 100$ | **100** |
| 3+ | $\max(100, \ldots)$ | **100** (floor) |

This is equivalent to `Explicit([400, 200, 100])`.

#### Rationale per tier · `sec:sentinel:noiseperf-rationale-per-tier`

**Depth 0 (root) — 400 rounds.**  Created exactly once at `SpectralSentinel::new()` or `reset()`, never on the hot path. 400 rounds guarantees **all axes converge**, including coherence:

| Metric | At 400 rounds |
|--------|---:|
| Per-cell cost (realistic, Brand) | **~1.7 s** |
| Surprise converged? | **Yes** (188 of 400) |
| Displacement converged? | **Yes** (24 of 400) |
| Coherence converged? | **Yes** (273 of 400) |
| η at transition (λ=0.99, b=16) | 0.99^6400 ≈ 0 |
| η at transition (λ=0.95, b=4) | 0.95^1600 ≈ 0 |

The root sees *all* traffic before the graph splits, so full convergence here eliminates graduated exemptions and transition artefacts for the most-observed cell.

**Depth 1 — 200 rounds.**  First-level splits are infrequent (typically a handful over the sentinel's lifetime) and each covers a large fraction of the address space.  200 rounds costs ~1.28 s per cell (Brand), which is acceptable for an uncommon event.  Surprise, displacement, and partial coherence all converge.

**Depth 2+ — 100 rounds (floor).**  These cells are created on the `reconcile_analysis_set()` hot path and may arrive in bursts. 100 rounds costs ~57 ms per cell (Brand), keeping a 5-cell burst at ~283 ms:

| Metric | At 100 rounds |
|--------|---:|
| Per-cell cost (realistic, Brand) | **~57 ms** |
| 5 cells in one ingest | **~283 ms** |
| Surprise converged? | **Yes** (67 of 100) |
| η at transition (λ=0.99, b=16) | 0.99^1600 ≈ 1.2 × 10⁻⁷ |
| η at transition (λ=0.95, b=4) | 0.95^400 ≈ 7.7 × 10⁻¹⁰ |
| Coherence converged? | No — finishes during real traffic |

100 rounds is sufficient for deep cells because:

- **Surprise** (the hardest non-gated axis) converges by round 67.
- **η** is already negligible (10⁻⁷ or below) — the maturity model correctly reports the cell as fully warmed.
- **The graduated clip-exemption** (ADR-S-013 §1) prevents false scoring while baselines are still converging.
- **The CUSUM slow-from-fast seeding** (ADR-S-013 §6) eliminates false drift at the noise→real transition.
- **Coherence** is gated by `rank_update_interval` anyway — it doesn't produce scores until $k \geq 2$, which takes `rank_update_interval` steps regardless of noise injection length.

#### User configuration · `sec:sentinel:noiseperf-user-configuration`

Operators choose whichever variant is most natural:

```toml
# Default — geometric decay (3 parameters)
[sentinel.noise_schedule]
type = "geometric"
root = 400
decay = 0.5
min = 100

# Same result, explicit array
[sentinel.noise_schedule]
type = "explicit"
rounds = [400, 200, 100]

# Converge everything at every depth (slow hot path)
[sentinel.noise_schedule]
type = "explicit"
rounds = [400]

# Slower decay — more rounds at shallow depths
[sentinel.noise_schedule]
type = "geometric"
root = 400
decay = 0.7
min = 100
# → [400, 280, 196, 137, 100, 100, ...]
```

**`Geometric`** is compact and self-documenting: the operator states intent (root quality, decay rate, floor) and the schedule scales automatically to any graph depth.  **`Explicit`** gives full control when specific per-layer tuning is needed; the last element repeats for all deeper layers.

### 2. Document the per-cell cost in the spec · `sec:sentinel:noiseperf-per-cell-cost`

§ALGO S-11.2 should note that noise injection runs per cell, not once globally.  The cost model is:

$$T_{\text{noise}} = \sum_{\text{cells } c} r(d_c) \times t_{\text{round}}$$

where $r(d_c)$ is the noise-schedule value for cell $c$ at depth $d_c$, and $t_{\text{round}}$ is the per-round observe cost (config and hardware dependent, ~2–17 ms in benchmarks).

### 3. Consider future optimisations (not in this ADR) · `sec:sentinel:noiseperf-future-optimisations`

These are documented here for future reference but are **not** being implemented now:

**a. Async / background noise injection.**  Move noise injection off the `ingest()` hot path.  New cells would be created with a "warming" flag and injected in a background task.  Scores from warming cells would be suppressed (η = 1.0) until injection completes.  This eliminates the hot-path stall entirely but adds concurrency complexity.

**b. Adaptive schedule.**  Auto-tune `decay` or the explicit array based on observed cell-creation frequency per depth.  If depth-1 splits are rare, increase their rounds; if they burst, reduce.  Requires runtime telemetry that doesn't yet exist.

**c. Shared noise cache.**  Pre-generate the noise batch matrix once and reuse it across cells (adjusting for dimension).  Saves RNG and allocation cost but not the `observe()` cost, which dominates.

**d. Lazy injection.**  Defer noise injection until the cell actually receives real traffic.  Cells that enter the analysis set but never see observations (common during rapid graph churn) would skip injection entirely.  Risk: first real batch hits unconverged baselines.

## Consequences · `sec:sentinel:noiseperf-consequences`

- **Root cell starts fully converged** on all axes (400 rounds, ~1.7 s one-time cost).  No graduated exemptions or transition artefacts needed for the root.

- **Depth-1 cells get 200 rounds** (~1.28 s each), converging surprise and displacement fully.  These splits are infrequent.

- **Depth-2+ cell creation cost is bounded to ~57 ms** at realistic settings (with Brand's SVD, ADR-S-016).  A 5-cell creation burst stalls `ingest()` for ~283 ms, well within budget for a batch-oriented system.

- **Surprise baselines are converged** at all depths before the cell sees real traffic.

- **Coherence baselines are NOT converged** on depth-2+ cells at creation.  They finish converging during the real-data phase, gated by η.  This is acceptable because coherence scoring requires $k \geq 2$, and the coherence EWMA cold→warms from the first real coherence score (ADR-S-013 §2b).

- **Test suite performance improves.**  Increasing test `noise_rounds` from 5 to 100 adds ~57 ms per cell construction (with Brand's SVD), but eliminates the ~500+ rounds of "compensatory warm-up" that many tests currently run.  Net effect depends on the test, but the convergence benchmark suite (now in criterion `warmup_cost_detailed` and `convergence_diagnostics.rs`) shows 100 rounds finishes well under 1 second total for all four tests.

- **The `noise_schedule` parameter is fully user-configurable.** Operators supply an array `[root, layer_1, ..., floor]` where the last element repeats for all deeper layers.  The geometric default `[400, 200, 100]` is a measured compromise.  A single-element `[400]` converges everything everywhere; a single-element `[100]` minimises hot-path cost at the expense of root convergence.

## Appendix: Convergence vs Cost Tradeoff Table · `sec:sentinel:noiseperf-cost-tradeoff`

| `noise_rounds` | Per-cell, Brand (ms) | Surprise | Displ. | Coherence | η (λ=0.99,b=16) | Recommended? |
|---:|---:|:---:|:---:|:---:|:---:|:---:|
| 5 | ~13 | NO | NO | NO | 0.923 | NO — current test default, too low |
| 50 | ~115 | NO | YES | NO | 0.449 | NO — current prod default, too low |
| 65 | ~138 | MARGINAL | YES | NO | 0.353 | MARGINAL |
| **100** | **~57** | **YES** | **YES** | **NO** | **1.2×10⁻⁷** | **YES — recommended default** |
| 200 | ~1283 | YES | YES | PARTIAL | ~0 | NO — too expensive per cell |
| 400 | ~1697 | YES | YES | YES | ~0 | NO — expensive on hot path |
