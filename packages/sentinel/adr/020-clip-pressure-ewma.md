# ADR-S-020: Clip-Pressure EWMA · `rec:sentinel:clip-pressure-ewma`

**Status:** Accepted **Date:** 2026-03-18 **Spec:** §ALGO S-6.1.1 (baseline update pipeline), §ALGO S-6.4 (clip-pressure dynamics), §ALGO S-13.1 ($\lambda_\rho$), §ALGO S-14.4 (per-cell clip-pressure reporting), §ALGO S-14.11 (fleet-wide clip-pressure distribution) **Supersedes:** The effective-clip formula in ADR-S-013 §1a (graduated clip-exemption using η alone) **Relates to:** [ADR-S-013](013-warm-up-convergence-benchmark.md) (warm-up convergence benchmark), [ADR-S-007](007-automatic-noise-injection.md) (automatic noise injection)

## Context · `sec:sentinel:clippressure-context`

The algorithm specification was amended to introduce a **clip-pressure EWMA** ($\bar{\rho}$) — a per-axis exponentially-weighted moving average of the batch clip ratio $\rho_t = (\text{clipped samples}) / (\text{total samples})$.

### Problem · `sec:sentinel:clippressure-problem`

The previous design (ADR-S-013 §1a) modulated the effective clip ceiling solely via the noise-influence parameter $\eta$:

$$
n_\sigma^{\text{eff}} = n_\sigma + n_\sigma \cdot \frac{\eta}{1 - \eta + \varepsilon}
$$

This works well during warm-up (where $\eta$ is large and organic clipping would create a positive feedback loop) but has a blind spot: **sustained high clipping in production** ($\eta \approx 0$) after a genuine regime change leaves the ceiling at $n_\sigma$ regardless of contamination level, and the baseline slowly poisons.

### Solution in the spec · `sec:sentinel:clippressure-spec-solution`

A new per-axis state $\bar{\rho}_t$ tracks the fraction of clipped samples via an EWMA with decay $\lambda_\rho$ (default 0.95, half-life ≈ 14 batches).  The effective-clip formula is now unified:

$$
p = \max(\eta,\; \bar{\rho}), \qquad
n_\sigma^{\text{eff}} = n_\sigma \cdot \left(1 + \frac{p}{1 - p + \varepsilon}\right)
$$

When $\bar{\rho}$ is low (clean traffic), this collapses to the production ceiling.  When $\bar{\rho}$ is high (contamination), the ceiling widens automatically — precisely the same safety valve that $\eta$ provides during warm-up, but now reactive to ongoing conditions.

The spec also consolidates clipping into a **single shared filter** computed against the fast EWMA's ceiling, applied once per batch.  The slow EWMA (CUSUM reference) and the fast EWMA both receive the same retained sample set, rather than each computing an independent clip filter.

### What already works · `sec:sentinel:clippressure-already-works`

- **Pre-clip raw batch mean for CUSUM**: `update_axis()` computes the raw `mean` from all scores and passes it to `cusum.update()` as `batch_mean` — this is already the pre-clip mean required by §6.1.1 step 5.

- **Formula shape**: The current formula `n + n·η/(1-η+ε)` is algebraically equivalent to `n·(1 + η/(1-η+ε))`, so the multiplicative form in the new spec is the same shape — just with $p$ replacing $\eta$.

- **Coherence rank-drop reset**: `adapt_rank()` already calls `coherence_bl.reset_cold()` when rank drops below 2 — adding $\bar{\rho}$ reset is a one-line extension.

### What needs to change · `sec:sentinel:clippressure-needs-change`

The implementation has **11 gaps** between the current code and the amended spec:

1. **New per-axis state.** `AxisBaseline` needs a `clip_pressure: f64` field ($\bar{\rho}$), initialised to 0.0.

2. **New config parameter.** `SentinelConfig` needs `clip_pressure_decay: f64` ($\lambda_\rho$) with default 0.95 and validation `0 < λ_ρ < 1`.

3. **Unified modulation formula.** The effective-clip computation in `observe()` must change from using $\eta$ alone to $p = \max(\eta, \bar{\rho})$ per axis.

4. **Single shared clip filter.** Clipping must move from inside `EwmaStats::update()` (per-EWMA independent) to the caller (`update_axis()`), computed once against the fast EWMA's ceiling and applied to both EWMAs.

5. **Clip ratio tracking.** The clip filter must report the clip ratio $\rho_t = 1 - |\text{retained}| / |\text{total}|$ back to the caller, so the caller can update $\bar{\rho}$.

6. **CUSUM slow EWMA receives pre-filtered samples.** Since clipping is externalised, `CusumAccumulator::update()` must accept pre-filtered samples instead of re-clipping internally.

7. **Report: `ScoreDistribution::clip_pressure`** (§14.4). New `f64` field in `[0, 1]`.

8. **Report: `HealthReport` clip-pressure distribution** (§14.11). New summary field (min/max/mean across active trackers).

9. **Coherence rank-drop reset.** `adapt_rank()` must also zero the coherence axis's $\bar{\rho}$.

10. **Warm-up completion reset** (§11.4). When noise influence crosses the warm-up threshold, all four axes' $\bar{\rho}$ must be zeroed.

11. **Baseline memory accounting.** Per-axis baseline size grows from 7 to 8 floats ($4 \times 8 = 32$ total), matching §4.3.

## Decision · `sec:sentinel:clippressure-decision`

### 1. Add clip-pressure state to `AxisBaseline` · `sec:sentinel:clippressure-axis-baseline-state`

```rust
struct AxisBaseline {
    fast: EwmaStats,
    cusum: CusumAccumulator,
    /// Clip-pressure EWMA: ρ̄ ∈ [0, 1].
    clip_pressure: f64,
}
```

Initialised to `0.0`.  `reset_cold()` zeros it.

### 2. Add config parameter · `sec:sentinel:clippressure-config-parameter`

```rust
/// Clip-pressure EWMA decay factor (λ_ρ).
///
/// Controls how quickly the clip-pressure estimate responds to
/// changing contamination levels.  Higher values = longer memory.
///
/// Default: `0.95` (half-life ≈ 14 batches).
/// Must be in (0, 1).
pub clip_pressure_decay: f64,
```

Default `0.95`.  Validated alongside `clip_sigmas`.

### 3. Externalise clipping from `EwmaStats` · `sec:sentinel:clippressure-externalise-clipping`

`EwmaStats::update()` currently computes its own clip filter internally.  The method signature changes to accept pre-filtered values and skip its internal filter:

**Option A — split into two methods:**
- `update_raw(values)` — accepts pre-filtered values, updates mean/variance unconditionally (no internal clipping).
- `update(values, clip_sigmas)` — preserved for any callers that still need self-contained clipping (backward compat).

**Option B — move clip logic to caller entirely:**
- `update()` drops its `clip_sigmas` parameter and always trusts the caller to have filtered.

We choose **Option A** for minimal disruption.  The test suite's direct `EwmaStats::update()` calls continue to work.  The hot path (`update_axis`) calls the new `update_raw()`.

### 4. Single shared clip filter in `update_axis()` · `sec:sentinel:clippressure-shared-clip-filter`

`update_axis()` gains the responsibility of:

1. Computing the clip ceiling from the **fast EWMA**'s current mean and variance: `ceiling = n_σ_eff · √var + mean`.
2. Filtering: `retained = scores.filter(|&v| v < ceiling)`.
3. Computing `ρ_t = 1 − retained.len() / scores.len()`.
4. Updating $\bar{\rho}$: `ρ̄ = λ_ρ · ρ̄ + (1 − λ_ρ) · ρ_t`.
5. Passing `&retained` to both `fast.update_raw()` and `cusum.update_filtered()`.

### 5. Adjust `CusumAccumulator` to accept pre-filtered samples · `sec:sentinel:clippressure-cusum-prefiltered`

`CusumAccumulator::update()` changes to accept pre-filtered samples (no internal re-clipping by the slow EWMA):

```rust
pub fn update_filtered(
    &mut self,
    filtered: &[f64],
    raw_batch_mean: f64,
    allowance_sigmas: f64,
    eps: f64,
) {
    // Gap uses raw_batch_mean (pre-clip), as before.
    let gap = raw_batch_mean - self.slow.mean() - allowance;
    self.accumulator = (self.accumulator + gap).max(0.0);
    // Slow EWMA receives the already-filtered samples.
    self.slow.update_raw(filtered);
    self.steps_since_reset += 1;
}
```

The old `update()` can be deprecated or removed.

### 6. Per-axis effective-clip computation · `sec:sentinel:clippressure-effective-clip`

The effective clip is now **per-axis** (each axis has its own $\bar{\rho}$).  `observe()` computes four separate effective-clip values, one per `update_axis()` call:

```rust
let p = eta.max(bl.clip_pressure);
let effective_clip = clip_sigmas * (1.0 + p / (1.0 - p + eps));
```

This replaces the single `effective_clip` variable computed from $\eta$ alone.

### 7. Reporting · `sec:sentinel:clippressure-reporting`

`ScoreDistribution` gains:

```rust
/// Current clip-pressure EWMA for this axis: ρ̄ ∈ [0, 1].
pub clip_pressure: f64,
```

`HealthReport` gains a `ClipPressureDistribution`:

```rust
pub clip_pressure_distribution: ClipPressureDistribution,
```

with `min`, `max`, and `mean` across active tracker axes.

### 8. Lifecycle resets · `sec:sentinel:clippressure-lifecycle-resets`

- `AxisBaseline::reset_cold()` — zeros `clip_pressure`.
- `adapt_rank()` — when coherence is destroyed, `reset_cold()` already covers the new field.
- Warm-up completion — when $\eta$ crosses the threshold, zero all four axes' `clip_pressure` to prevent warm-up contamination echoing into production scoring.

### 9. Accepted deviations · `sec:sentinel:clippressure-accepted-deviations`

- **Cold-path bypass is preserved.** When an EWMA is cold (first update), all values are accepted even under the new shared filter.  This matches the spec's "skip entirely on first update" (§6.1.1 Design Note 1).

- **Formula equivalence.** The implementation may use either the additive form `n + n·p/(1-p+ε)` or the multiplicative form `n·(1 + p/(1-p+ε))` — they are algebraically identical.  The choice is left to readability preference.

## Consequences · `sec:sentinel:clippressure-consequences`

- The effective-clip ceiling becomes **self-correcting**: it widens automatically under sustained contamination (high $\bar{\rho}$) and tightens to $n_\sigma$ once the attack subsides.

- `EwmaStats::update()` retains backward compatibility.  The new `update_raw()` method is used only by the baseline pipeline.

- Four new `f64` fields (one per axis) increase per-tracker memory by 32 bytes — negligible at sentinel scale.

- One new config parameter (`clip_pressure_decay`).  Hosts that don't set it get the default $\lambda_\rho = 0.95$.

- ADR-S-013 §1a's graduated clip-exemption formula is superseded: the unified $\max(\eta, \bar{\rho})$ formula subsumes the pure-η behaviour as a special case (when $\bar{\rho} = 0$, $p = \eta$ exactly).
