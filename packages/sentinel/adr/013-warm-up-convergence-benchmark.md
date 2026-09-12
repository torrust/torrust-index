# ADR-S-013: Warm-Up Convergence Benchmark · `rec:sentinel:benchmark-convergence-and-fix-three-root-causes`

**Status:** Accepted (core fixes implemented; config and spec updates remain) **Date:** 2026-03-10 **Spec:** §ALGO S-11.5 (maturity tracking), §ALGO S-11.6 (system-level warm-up), §ALGO S-7.1.1 (EWMA baseline tracking and outlier clipping), §ALGO S-5.2 Phase 3 (latent distribution) **Relates to:** [ADR-S-007](007-automatic-noise-injection.md) (automatic noise injection), [ADR-S-012](012-test-duration-budget.md) (test duration budget), [ADR-S-001](001-measures-not-opinions.md) (measures not opinions), [ADR-S-014](014-subspace-tracker-visibility.md) (subspace tracker visibility), [ADR-S-015](015-cell-creation-performance.md) (cell creation performance), [ADR-S-016](016-brand-incremental-svd.md) (Brand's incremental SVD)

**Findings:** Four rounds of investigation (preliminary → secondary → code audit → tertiary synthesis) plus post-fix quaternary validation and recommendations.  All six findings documents have been retired; their essential content is captured in this ADR and in the spec updates to §ALGO S-5.2, §ALGO S-7.1.1, §ALGO S-7.4, §ALGO S-11.5, and §ALGO S-11.6.

## Context · `sec:sentinel:warmbench-context`

The sentinel's warm-up tests (`warm_up.rs`) and many other test suites use **arbitrary iteration counts** — 50, 100, or even 500 warm-up batches — with no empirical basis for why those numbers were chosen.  ADR-S-012 identifies the `warm_up` suite as the worst offender (135.6 s for `phase4_steady_state_maturity`, which asserts `noise_influence < 0.5` after 100 warm-up batches).

The maturity model (§ALGO S-11.5) is a closed-form exponential decay:

$$\eta_n = \lambda^n$$

where $\eta$ is the noise influence fraction and $\lambda$ is the forgetting factor.  For a given $\lambda$, the number of real batches required to reach any target $\eta^*$ is exactly:

$$n^* = \left\lceil \frac{\ln \eta^*}{\ln \lambda} \right\rceil$$

At `integration_config()` values ($\lambda = 0.95$):

| Target $\eta^*$ | Required batches $n^*$ |
|-----------------:|-----------------------:|
| 0.50             | 14                     |
| 0.10             | 45                     |
| 0.05             | 59                     |
| 0.01             | 90                     |

Yet `phase4_steady_state_maturity` uses **100 warm-up batches** (each with 20 seed values) merely to assert $\eta < 0.5$ — a condition that is theoretically met after **14 batches**.

### The missing piece (original hypothesis) · `sec:sentinel:warmbench-missing-piece-hypothesis`

The original proposal hypothesised that EWMA baselines converge at roughly the same rate as $\eta$, giving `n_settled ≤ 30` (real data) and `r_noise ≤ 20` (noise injection).  This was based on the assumption that the simple exponential model $\eta_n = \lambda^n$ would approximate baseline convergence.

**This hypothesis was wrong by an order of magnitude.**

### What the investigation discovered · `sec:sentinel:warmbench-investigation-findings`

Four rounds of empirical investigation — 40+ targeted tests, a line-by-line code audit, and multi-seed robustness analysis — revealed that EWMA baseline convergence is governed by three interacting mechanisms that the simple exponential model cannot capture:

1. **A clipping-ceiling positive feedback loop** (~85% of the convergence gap).  The EWMA outlier clip ceiling is computed from the EWMA's own nascent statistics.  The first batch produces near-zero variance → ultra-tight ceiling (0.28) → ~31% of valid scores are clipped every round → EWMA mean stays artificially low → ceiling stays tight.  This self-reinforcing loop kept surprise baselines drifting for **1000+ rounds** before the fix.

2. **A latent variance cold-start cascade** (~6% of the gap). `SubspaceTracker::new()` initialised `lat_var` at 1.0 when the true steady-state value is ~0.19 (5.3× mismatch).  The surprise score $= z^2 / \nu$ was non-stationary for ~60 rounds as $\nu$ decayed, creating a serial cascade: lat_var EWMA → surprise scores → surprise baseline EWMA.

3. **Stochastic batch variance** (~6% of the gap).  At batch_size=4, per-batch score variance has CV ≈ 0.43, causing the EWMA to overshoot and undershoot around its target.

Additionally, the **convergence metric itself was broken**. The original `find_settled_round()` function (5% tolerance against the last-round reference) was unfalsifiable for 3 of 4 axes: surprise, coherence, and displacement baselines wander with 8–17% CV even at true steady state, so the metric measured *trace length*, not convergence.

### No bug — a design trap · `sec:sentinel:warmbench-design-trap`

A line-by-line code audit confirmed that the implementation faithfully follows §ALGO S-7.1.1.  The slow convergence was a *design consequence* of applying attack-resistant outlier clipping from round 1 using nascent statistics, not a coding error.

## Decision · `sec:sentinel:warmbench-decision`

**Add a convergence benchmark and characterisation test suite** that empirically measures baseline convergence, **and fix the three root causes** that make convergence 5–10× slower than the theoretical model predicts.

### Implemented fixes · `sec:sentinel:warmbench-implemented-fixes`

#### Fix 1: Graduated clip-exemption (§ALGO S-7.1.1) · `sec:sentinel:warmbench-graduated-clip-exemption`

The effective clip width now scales with noise influence $\eta$:

$$n_\sigma^{\text{eff}} = n_\sigma + \frac{n_\sigma \cdot \eta}{1 - \eta + \varepsilon}$$

| $\eta$ | $n_\sigma^{\text{eff}}$ (at $n_\sigma = 3$) | Behaviour |
|-------:|--------------------------------------------:|-----------|
| 1.0    | ~3,000,003 (capped by $\varepsilon$)        | No clipping (cold) |
| 0.99   | 300                                         | Very wide ceiling |
| 0.50   | 6.0                                         | Moderately relaxed |
| 0.01   | 3.03                                        | Near-production |
| 0.0    | 3.00                                        | Production (exact) |

Monotonicity is confirmed: $n_\sigma^{\text{eff}}$ is strictly decreasing as $\eta$ decreases.  At $\eta = 0$ the effective clip equals the configured `clip_sigmas` exactly.

**Result:** Surprise convergence improved from **1000+ rounds to ~65 rounds** at $\lambda = 0.95$, $b = 4$ — a 15× improvement.

**Code:** `tracker.rs` Phase 4 (`observe()`), +13 lines.

#### Fix 2: Latent cold→warm initialisation (§ALGO S-5.2 Phase 3) · `sec:sentinel:warmbench-latent-cold-warm-initialisation`

On the first batch (`step == 0`), `lat_mean`, `lat_var`, and `cross_corr` are seeded directly from data rather than blended with the initial defaults:

```rust
if self.step == 0 {
    self.lat_mean[j] = col_mean;
    self.lat_var[j]  = col_var.max(eps);
} else {
    self.lat_mean[j] = lam.mul_add(self.lat_mean[j], alpha * col_mean);
    self.lat_var[j]  = lam.mul_add(self.lat_var[j], alpha * col_var.max(eps));
}
```

**Result:** Surprise rise factor improved from **5.9× to 1.26×**.

**Code:** `tracker.rs` Phase 3 (`evolve_latent()`), +12 lines.

#### Fix 3: CUSUM fast-slow gap seeding · `sec:sentinel:warmbench-cusum-gap-seeding`

After noise injection completes, the slow EWMA ($\lambda_s = 0.999$, half-life 693 steps) is seeded from the fast EWMA's converged values, and CUSUM accumulators are reset:

```rust
// In inject_noise_into_cell(), after noise rounds complete:
cell.tracker.seed_cusum_slow_from_baselines();
cell.tracker.reset_cusum();
```

**Result:** False CUSUM drift reduced from **198 to 5.7** (97% reduction).

**Code:** `ewma.rs` `seed_from()`, `cusum.rs` `seed_slow_from()`, `tracker.rs` `seed_cusum_slow_from_baselines()`, `sentinel/mod.rs` `inject_noise_into_cell()`.

### Convergence test suite · `sec:sentinel:warmbench-convergence-test-suite`

> **Note (2026-03-11):** The five original files listed below have been consolidated into a cleaner structure.  The new layout is:
>
> - `src/tests/convergence_common.rs` — shared configs, noise generation, metrics
> - `src/tests/convergence_ewma.rs` — pure EWMA property tests (5 tests)
> - `src/tests/convergence_eta.rs` — η tracking & maturity tests (4 tests)
> - `src/tests/convergence_noise.rs` — tracker baseline convergence (9 tests)
> - `src/tests/convergence_fixes.rs` — ADR-S-013 fix validation (7 tests)
> - `src/tests/convergence_diagnostics.rs` — on-demand `#[ignore]` diagnostic tables
> - `benches/sentinel.rs` — `warmup_cost_detailed` criterion group
>
> The original files have been deleted.

~~Five~~ source files ~~implement~~ implemented the benchmark and characterisation tests:

- `src/convergence_benchmark.rs` (765 lines) — the benchmark with timing and convergence trace recording.
- `src/convergence_tests.rs` (1226 lines) — `pub(crate)` unit tests with direct `SubspaceTracker` access for per-axis diagnostics.
- `src/convergence_investigation.rs` (837 lines) — Round 1 targeted experiments isolating each root cause.
- `src/convergence_investigation_2.rs` — Round 2 experiments plus the `audit_surprise_pipeline` tracer (1000-round round-by-round trace).
- `src/convergence_investigation_3.rs` — Post-fix validation (12 tests, all pass): Q1–Q8 answering specific convergence questions, multi-seed robustness, and production-config validation.

### Convergence metric: windowed-mean comparison · `sec:sentinel:warmbench-windowed-mean-metric`

The original `find_settled_round()` (single-point 5% tolerance) was replaced by a **windowed-mean comparison** that absorbs per-round noise:

```rust
fn find_converged_round(
    baselines: &[f64],
    window: usize,    // e.g. 20 = 1/α at λ = 0.95
    tolerance: f64,
) -> usize
```

Convergence is declared when the rolling mean over a window of $W$ rounds is within tolerance of the reference window at the end of the trace.  Per-axis tolerances are mandatory:

| Axis | Tolerance | Empirical CV ($b = 4$) | Empirical CV ($b = 16$) |
|------|:---------:|:----------------------:|:-----------------------:|
| Novelty | 1% | 0.13% | 0.06% |
| Displacement | 10% | 5.9% | 3.0% |
| Surprise | 20% | 9.0% | 3.6% |
| Coherence | 20% | 19.9% | 11.5% |

## Empirical Convergence Results · `sec:sentinel:warmbench-empirical-results`

### Post-fix convergence times · `sec:sentinel:warmbench-post-fix-times`

**Test config ($\lambda = 0.95$, $b = 4$, seed = 42):**

| Axis | Converged (rounds) | CV (last 100) |
|------|-------------------:|:-------------:|
| Novelty | 21 | 0.07% |
| Displacement | 404 | 3.37% |
| Surprise | **65** | 6.46% |
| Coherence | **406** | 10.01% |

**Test config ($\lambda = 0.95$, $b = 16$, seed = 42):**

| Axis | Converged (rounds) | CV (last 100) |
|------|-------------------:|:-------------:|
| Novelty | 21 | 0.04% |
| Displacement | **21** | 2.08% |
| Surprise | **70** | 2.53% |
| Coherence | **173** | 8.68% |

**Production config ($\lambda = 0.99$, $b = 16$, seed = 42):**

| Axis | Converged (rounds) | CV (last 200) |
|------|-------------------:|:-------------:|
| Novelty | 101 | 0.02% |
| Displacement | 101 | 0.83% |
| Surprise | **398** | 0.94% |
| Coherence | 315 | 2.36% |

### η-convergence matches theory exactly · `sec:sentinel:warmbench-eta-theory-match`

The maturity metric $\eta_n = \lambda^n$ matches the theoretical prediction to machine epsilon: max $|\eta - \lambda^{bn}| = 2.78 \times 10^{-17}$.  The exponential model is trivially correct for η but **not** for EWMA baselines.

### Per-axis convergence character · `sec:sentinel:warmbench-per-axis-character`

| Axis | Character | Dominant bottleneck |
|------|-----------|---------------------|
| **Novelty** | Instant (round 21). CV = 0.07–0.13%. | None — constant-norm score is inherently stable. |
| **Displacement** | Fast at $b \geq 16$ (21 rounds); bimodal at $b = 4$ (21–475). | Stochastic subspace evolution at small batch sizes. |
| **Surprise** | 65 rounds ($\lambda = 0.95$); 398 rounds ($\lambda = 0.99$). | Cascaded lat_var → score EWMA; stochastic batch variance. The clipping feedback loop is eliminated. |
| **Coherence** | Consistently slowest: 406 ($b = 4$), 173 ($b = 16$), 315 (production). | Rank-gating delay ($k < 2$ → score = 0) plus `cross_corr` convergence time. |

### Multi-seed robustness ($\lambda = 0.95$, $b = 4$, 10 seeds) · `sec:sentinel:warmbench-multi-seed-robustness`

| Axis | Min | Max | Range | Mean |
|------|----:|----:|------:|-----:|
| Novelty | 21 | 21 | 0 | 21.0 |
| Displacement | 21 | 475 | 454 | 260.0 |
| Surprise | 61 | 392 | 331 | 131.8 |
| Coherence | 365 | 477 | 112 | 411.1 |

Novelty is deterministic.  Coherence is consistently slow but seed-stable (range 112).  Surprise and displacement have high seed variance at $b = 4$, driven by stochastic subspace evolution.  At $b = 16$, displacement becomes deterministic (21 across all seeds) and surprise variance decreases substantially.

## Revised Iteration-Count Guidance · `sec:sentinel:warmbench-iteration-guidance`

The original ADR proposed `n_settled ≤ 30` and `r_noise ≤ 20`. These targets were off by >10×.  Revised guidance:

| Config | Worst-case axis | Convergence (rounds) |
|--------|-----------------|---------------------:|
| $\lambda = 0.95$, $b = 4$ | Coherence | 406 |
| $\lambda = 0.95$, $b = 16$ | Coherence | 173 |
| $\lambda = 0.99$, $b = 16$ | Surprise | 398 |

**`noise_rounds` must be at least as large as the worst-case convergence time** for baselines to be settled before real traffic arrives.  The current defaults are insufficient:

| Config | Current `noise_rounds` | Required minimum | Shortfall |
|--------|:----------------------:|:----------------:|:---------:|
| Test ($\lambda = 0.95$, $b = 4$) | 5 | ≥65 (surprise) | 13× |
| Production ($\lambda = 0.99$, $b = 16$) | 50 | ≥400 | 8× |

### Derived iteration-count table · `sec:sentinel:warmbench-iteration-count-table`

| Test need | Iterations | Justification |
|-----------|:----------:|---------------|
| Surprise baseline settled ($\lambda = 0.95$) | 65 | Empirical windowed-mean convergence |
| Coherence baseline settled ($b = 4$) | 406 | Rank-gating delay + EWMA convergence |
| Coherence baseline settled ($b = 16$) | 173 | Reduced by CLT at larger batch size |
| Production worst-case ($\lambda = 0.99$) | 398 | Surprise at slower forgetting rate |
| η < 0.05 | 59 | Exact: $\lceil \ln(0.05) / \ln(\lambda) \rceil$ |

## Investigation History · `sec:sentinel:warmbench-investigation-history`

The path from hypothesis to validated fixes spanned four rounds:

**Round 1 (Preliminary):** Identified the 5.1× convergence gap. Attributed it primarily to the `lat_var` cold-start cascade (deterministic model predicted 114 rounds).  Correct on mechanisms, wrong on dominance — the cascade accounts for only ~6% of the gap.

**Round 2 (Secondary):** Discovered the convergence metric was fundamentally broken (round-300 reference was 56% wrong for surprise; baselines wander 15.5% CV at true steady state).  The 5% tolerance criterion was unfalsifiable for 3 of 4 axes. Incorrectly attributed 89% of the gap to score-level variance (conflated the measurement problem with the convergence problem).

**Round 3 (Code Audit):** Line-by-line audit confirmed **no code bug** — the implementation faithfully follows §ALGO S-7.1.1. Discovered the **clipping-ceiling positive feedback loop**: the true dominant cause (~85% of the gap, adding 800+ rounds to surprise convergence).  Cold→warm variance ≈ $10^{-4}$ → ceiling ≈ 0.28 → perpetual clipping → slow mean/variance drift.

**Round 4 (Quaternary — Post-fix Validation):** Implemented both fixes.  Surprise convergence improved from 1000+ to 65 rounds. Confirmed the CUSUM fast-slow gap as severe (198 false drift). Discovered coherence as the true production bottleneck (406 rounds at $b = 4$) and displacement bimodality across seeds.

### What each round got right and wrong · `sec:sentinel:warmbench-round-assessment`

| Finding | Preliminary | Secondary | Audit | Tertiary |
|---------|:-----------:|:---------:|:-----:|:--------:|
| lat_var cold-start exists | ✓ | ✓ | ✓ | ✓ |
| lat_var cascade is PRIMARY | **✗** | corrected | — | rank 2 |
| Criterion is broken for 3/4 axes | — | ✓ | ✓ | ✓ |
| Clipping "adds ≤ 5 rounds" | — | **✗** | corrected | dominant |
| Score-level variance is 89% of gap | — | **✗** | corrected | ~3% |
| Clipping-ceiling feedback loop | — | — | ✓ | ✓ |
| No code bug | — | — | ✓ | ✓ |

## Downstream Consequences · `sec:sentinel:warmbench-downstream-consequences`

This investigation triggered three further ADRs:

- **ADR-S-014** — Subspace Tracker Visibility.  Convergence tests need direct `SubspaceTracker` access.  Decided to keep the tracker `pub(crate)` and enrich `CellInspection` with baseline snapshots rather than exposing the tracker publicly.

- **ADR-S-015** — Cell Creation Performance.  The finding that `noise_rounds` must increase to ≥400 raised hot-path cost concerns: `inject_noise_into_cell()` runs on every new cell in the analysis set, not just at construction.  Benchmarked the cost and motivated ADR-S-016.

- **ADR-S-016** — Brand's Incremental SVD.  Replaced dense thin SVD in `evolve_subspace()` with Brand's incremental algorithm, reducing per-round SVD cost and cell creation times by ~1.9–2.9×.  This makes the higher `noise_rounds` affordable on the hot path.

## Remaining Work · `sec:sentinel:warmbench-remaining-work`

| Item | Priority | Status |
|------|----------|--------|
| Increase `noise_rounds` default (50 → ≥400 at $\lambda = 0.99$) | **HIGH** | TODO |
| Promote windowed-mean convergence metric to production test suite | Medium | Validated in quaternary tests |
| ~~Update §ALGO S-7.1.1 with graduated clip-exemption formula~~ | ~~Medium~~ | Done (2026-03-11) |
| ~~Update §ALGO S-5.2 Phase 3 with cold→warm initialisation~~ | ~~Medium~~ | Done (2026-03-11) |
| ~~Update §ALGO S-7.4 with slow-from-fast CUSUM seeding~~ | ~~Medium~~ | Done (2026-03-11) |
| ~~Update §ALGO S-11.5–11.6 with empirical convergence data~~ | ~~Medium~~ | Done (2026-03-11) |
| Address coherence as the production bottleneck (rank-gating delay) | Medium | Confirmed structural |
| Per-axis test tolerances and displaced bimodality metric | Low | Characterised |
| ~~Mark superseded findings documents~~ | ~~Low~~ | Done — files retired, content absorbed into spec (2026-03-11) |
| ~~Update `convergence_tests.rs` module doc~~ | Low | Done — file removed in consolidation (2026-03-11) |

## Alternatives Considered · `sec:sentinel:warmbench-alternatives`

- **Just reduce iteration counts by the theoretical formula.** The formula for $\eta$ is trivially exact, but EWMA baseline convergence depends on the score distribution, outlier clipping, and batch-mean aggregation.  The investigation proved this approach would be off by 5–10×.

- **Remove clipping outright during warm-up.**  The graduated clip-exemption was chosen over binary on/off because it provides proportional attack resistance at all maturity levels.  At $\eta = 0.01$ the clip is 3.03σ — negligibly wider than production.

- **Accept longer noise injection without fixing the feedback loop.**  Would require `noise_rounds > 1000` at $b = 4$. Infeasible on the hot path (cell creation during live traffic).

- **Property-based testing across random configs.**  Worth doing eventually, but the immediate need was concrete empirical bounds for known configurations.

## Consequences · `sec:sentinel:warmbench-consequences`

- **Empirically justified iteration counts** replace arbitrary constants.  The investigation provides precise per-axis, per-config convergence times rather than order-of-magnitude estimates.

- **Three code fixes** eliminate the dominant convergence bottlenecks: the clipping feedback loop (Fix 1), the lat_var cold-start cascade (Fix 2), and the CUSUM fast-slow gap (Fix 3).

- **Surprise convergence improved 15×** (1000+ → 65 rounds at $\lambda = 0.95$), exceeding the original prediction of ~120 rounds.

- **The bottleneck shifted** from surprise (fixed) to coherence (structural: rank-gating delay + cross-correlation convergence).  Coherence at $b = 4$ takes ~406 rounds — this is the true system convergence time.

- **`noise_rounds` defaults are insufficient.**  Production needs ≥400 at $\lambda = 0.99$.  ADR-S-015 and ADR-S-016 ensure this increase is affordable on the hot path.

- **The simple exponential model is only valid for η.**  EWMA baselines converge at a rate determined by cascaded EWMA interactions, clipping policy, batch size, and axis-specific score distributions.  The spec (§ALGO S-11.5–11.6) must be updated to reflect this.

- **Regression detection.**  The convergence test suite catches changes to EWMA update rules, clipping behaviour, or cold start logic that would shift convergence times.
