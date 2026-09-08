# ADR-S-020 Implementation Plan: Clip-Pressure EWMA · `plan:sentinel:clip-pressure-ewma-implementation-plan`

Detailed, file-by-file implementation plan for the 11 gaps identified in [ADR-S-020](020-clip-pressure-ewma.md).

---

## Phase 0 — Preparation · `sec:sentinel:clipplan-phase0-preparation`

Before writing any code:

1. **Read §ALGO S-6.1.1, §ALGO S-6.4, §ALGO S-13.1, §ALGO S-14.4, §ALGO S-14.11** end-to-end to internalise the spec language.
2. **Snapshot the existing test suite** — run `CARGO_PROFILE_DEV_OPT_LEVEL=3 cargo test --package torrust-sentinel --all-targets --all-features` and record baseline counts/timings. The warm-up convergence benchmark (ADR-S-013) is especially important: the new formula must converge no slower at η ≈ 1 and no wider at η ≈ 0.
3. Create a **feature branch** `feat/clip-pressure-ewma`.

---

## Phase 1 — New Config Parameter · `sec:sentinel:clipplan-phase1-config-parameter`

**File: `src/config.rs`**

### Step 1.1 — Add field to `SentinelConfig` · `sec:sentinel:clipplan-step-1-1-config-field`

Insert `clip_pressure_decay` after `clip_sigmas`:

```rust
/// Clip-pressure EWMA decay factor (λ_ρ).
///
/// Controls how quickly the per-axis clip-pressure estimate adapts
/// to changing contamination levels.  Higher values = longer memory.
///
/// Half-life ≈ ln(2) / ln(1/λ_ρ):
/// - `0.95` = ~14 batches (default, §ALGO S-13.1)
/// - `0.99` = ~69 batches
///
/// Must be in `(0.0, 1.0)`.
///
/// Default: `0.95`
pub clip_pressure_decay: f64,
```

### Step 1.2 — Default impl · `sec:sentinel:clipplan-step-1-2-default-impl`

In `impl Default for SentinelConfig`:

```rust
clip_pressure_decay: 0.95,
```

### Step 1.3 — Validation · `sec:sentinel:clipplan-step-1-3-validation`

Add a new `ConfigError` variant:

```rust
/// `clip_pressure_decay` must be in `(0.0, 1.0)`.
ClipPressureDecayOutOfRange(f64),
```

Add a corresponding arm in `Display for ConfigError`:

```rust
Self::ClipPressureDecayOutOfRange(v) =>
    write!(f, "clip_pressure_decay must be in (0.0, 1.0), got {v}"),
```

Add the validation check inside `validate()`, near the `clip_sigmas` check:

```rust
if self.clip_pressure_decay <= 0.0 || self.clip_pressure_decay >= 1.0 {
    errors.push(ConfigError::ClipPressureDecayOutOfRange(self.clip_pressure_decay));
}
```

### Step 1.4 — Test · `sec:sentinel:clipplan-step-1-4-test`

In `src/tests/config.rs`, add a test `rejects_clip_pressure_decay_out_of_range` paralleling `rejects_non_positive_clip_sigmas`:

```rust
#[test]
fn rejects_clip_pressure_decay_out_of_range() {
    let cfg = SentinelConfig::<f64> {
        clip_pressure_decay: 0.0,
        ..Default::default()
    };
    assert!(cfg.validate().is_err());

    let cfg = SentinelConfig::<f64> {
        clip_pressure_decay: 1.0,
        ..Default::default()
    };
    assert!(cfg.validate().is_err());

    let cfg = SentinelConfig::<f64> {
        clip_pressure_decay: 0.95,
        ..Default::default()
    };
    assert!(cfg.validate().is_ok());
}
```

### Step 1.5 — Propagate to `SubspaceTracker` · `sec:sentinel:clipplan-step-1-5-propagate`

In `src/sentinel/tracker.rs`, add a field:

```rust
clip_pressure_decay: f64,
```

Initialise it from `cfg.clip_pressure_decay` in `SubspaceTracker::new()`.

### Checkpoint · `sec:sentinel:clipplan-phase1-checkpoint`

`cargo test --package torrust-sentinel --all-targets --all-features` — all existing tests pass, new config test passes.

---

## Phase 2 — Per-Axis Clip-Pressure State · `sec:sentinel:clipplan-phase2-axis-state`

**File: `src/sentinel/tracker.rs`**

### Step 2.1 — Add field to `AxisBaseline` · `sec:sentinel:clipplan-step-2-1-baseline-field`

```rust
struct AxisBaseline {
    fast: EwmaStats,
    cusum: CusumAccumulator,
    /// Clip-pressure EWMA: ρ̄ ∈ [0, 1]  (§ALGO S-6.4).
    clip_pressure: f64,
}
```

### Step 2.2 — Initialise to `0.0` · `sec:sentinel:clipplan-step-2-2-initialise`

In `AxisBaseline::new()`:

```rust
Self {
    fast: EwmaStats::new(fast_decay),
    cusum: CusumAccumulator::new(slow_decay),
    clip_pressure: 0.0,
}
```

### Step 2.3 — Reset: `reset_cold()` zeros `clip_pressure` · `sec:sentinel:clipplan-step-2-3-reset-cold`

```rust
fn reset_cold(&mut self) {
    self.fast.reset_cold();
    self.cusum.reset_cold();
    self.clip_pressure = 0.0;
}
```

### Checkpoint · `sec:sentinel:clipplan-phase2-checkpoint`

Compiles, all tests pass.  The field exists but is unused — no behavioural change yet.

---

## Phase 3 — Externalise Clipping from `EwmaStats` · `sec:sentinel:clipplan-phase3-externalise-clipping`

**File: `src/ewma.rs`**

### Step 3.1 — Add `update_raw()` method · `sec:sentinel:clipplan-step-3-1-update-raw`

This method accepts **pre-filtered** values — the caller has already applied the clip filter.  It performs the same EWMA update as `update()` but skips the internal ceiling computation:

```rust
/// Update the baseline with pre-filtered values.
///
/// The caller is responsible for outlier rejection.  This method
/// unconditionally incorporates all values (including the cold→warm
/// path).  Used by the baseline pipeline (§ALGO S-6.1.1) where
/// clipping is externalised to `update_axis()`.
pub fn update_raw(&mut self, normals: &[f64]) {
    if normals.is_empty() {
        return;
    }

    #[allow(clippy::cast_precision_loss)]
    let new_mean = normals.iter().sum::<f64>() / normals.len() as f64;

    if !self.warm {
        self.mean = new_mean;
        if normals.len() > 1 {
            let var = mean_squared_deviation(normals, new_mean);
            self.variance = var.max(1e-4);
        }
        self.warm = true;
        return;
    }

    let alpha = 1.0 - self.decay;
    self.mean = self.decay.mul_add(self.mean, alpha * new_mean);

    if normals.len() > 1 {
        let var = mean_squared_deviation(normals, new_mean).max(1e-4);
        self.variance = self.decay.mul_add(self.variance, alpha * var);
    }
}
```

> **Why keep `update()` around?**  Direct callers in the test suite (unit tests of `EwmaStats` itself) rely on the self-contained `update(values, clip_sigmas)` signature.  Removing it is unnecessary churn.

### Step 3.2 — Add `ceiling()` helper · `sec:sentinel:clipplan-step-3-2-ceiling-helper`

Expose the clip ceiling so the caller can compute it once:

```rust
/// Compute the upper-tail clip ceiling: `mean + clip_sigmas · √variance`.
///
/// Returns `f64::INFINITY` when the baseline is cold (no meaningful
/// ceiling can be defined — matches the cold-path bypass in `update()`).
#[must_use]
pub fn ceiling(&self, clip_sigmas: f64) -> f64 {
    if self.warm {
        clip_sigmas.mul_add(self.variance.sqrt(), self.mean)
    } else {
        f64::INFINITY
    }
}
```

### Step 3.3 — Tests · `sec:sentinel:clipplan-step-3-3-tests`

Add a unit test verifying `update_raw()` produces the same result as `update()` when all values are within the clip ceiling:

```rust
#[test]
fn update_raw_matches_update_when_no_clipping() {
    let mut a = EwmaStats::new(0.95);
    let mut b = EwmaStats::new(0.95);
    let values = &[1.0, 1.2, 0.8, 1.1, 0.9];

    a.update(values, 100.0); // clip_sigmas so high nothing is clipped
    b.update_raw(values);

    assert!((a.mean() - b.mean()).abs() < 1e-12);
    assert!((a.variance() - b.variance()).abs() < 1e-12);
}
```

### Checkpoint · `sec:sentinel:clipplan-phase3-checkpoint`

Compiles, all tests pass.  No behavioural change to the hot path yet.

---

## Phase 4 — Adjust `CusumAccumulator` for Pre-Filtered Samples · `sec:sentinel:clipplan-phase4-cusum-prefiltered`

**File: `src/sentinel/cusum.rs`**

### Step 4.1 — Add `update_filtered()` method · `sec:sentinel:clipplan-step-4-1-update-filtered`

```rust
/// Update the accumulator with **pre-filtered** samples.
///
/// Identical to [`update`](Self::update) except the slow EWMA
/// receives pre-filtered values via `update_raw()` instead of
/// applying its own clip filter.  The CUSUM gap still uses
/// `raw_batch_mean` (the pre-clip mean of the full batch).
///
/// Used by the shared-filter pipeline (§ALGO S-6.1.1 step 5).
pub fn update_filtered(
    &mut self,
    filtered: &[f64],
    raw_batch_mean: f64,
    allowance_sigmas: f64,
    eps: f64,
) {
    let slow_mean = self.slow.mean();
    let slow_std = (self.slow.variance() + eps).sqrt();
    let allowance = allowance_sigmas * slow_std;

    let gap = raw_batch_mean - slow_mean - allowance;
    self.accumulator = (self.accumulator + gap).max(0.0);

    self.slow.update_raw(filtered);
    self.steps_since_reset += 1;
}
```

> **`raw_batch_mean`** is the mean of the *full, unclipped* batch — this is already computed in `update_axis()` as the variable `mean` and passed to `cusum.update()` today. No new computation needed.

### Step 4.2 — Test · `sec:sentinel:clipplan-step-4-2-test`

Add a test verifying `update_filtered()` produces the same result as `update()` when no samples are clipped:

```rust
#[test]
fn update_filtered_matches_update_no_clip() {
    let mut a = CusumAccumulator::new(0.999);
    let mut b = CusumAccumulator::new(0.999);
    let scores = &[1.0, 1.1, 0.9, 1.05, 0.95];
    let mean = scores.iter().sum::<f64>() / scores.len() as f64;

    a.update(scores, mean, 0.5, 1e-6, 100.0);
    b.update_filtered(scores, mean, 0.5, 1e-6);

    assert!((a.snapshot().accumulator - b.snapshot().accumulator).abs() < 1e-12);
}
```

### Checkpoint · `sec:sentinel:clipplan-phase4-checkpoint`

Compiles, all tests pass.

---

## Phase 5 — Unified Clip + Clip-Pressure in `update_axis()` · `sec:sentinel:clipplan-phase5-unified-clip`

**File: `src/sentinel/tracker.rs`**

This is the **core change**.  The existing `update_axis()` delegates clipping to `EwmaStats::update()` and `CusumAccumulator::update()`, each applying their own independent filter.  After this step, `update_axis()` computes a **single shared clip filter** from the fast EWMA's ceiling, and both EWMAs receive the same retained set.

### Step 5.1 — Change the caller: `observe()` · `sec:sentinel:clipplan-step-5-1-observe-caller`

Replace the single `effective_clip` variable with per-axis computation inside `update_axis()`.  Pass `clip_pressure_decay` and `clip_sigmas`
+ `eta` as inputs:

```rust
// In observe(), Phase 4 — replace the `effective_clip` block:
let allowance = self.cusum_allowance_sigmas;
let eta = self.noise_influence;
let clip_sigmas = self.clip_sigmas;
let eps = self.eps;
let cp_decay = self.clip_pressure_decay;

let novelty_dist = Self::update_axis(
    &mut self.novelty_bl, &nov_scores, eps, allowance,
    clip_sigmas, eta, cp_decay, true,
);
let displacement_dist = Self::update_axis(
    &mut self.displacement_bl, &disp_scores, eps, allowance,
    clip_sigmas, eta, cp_decay, true,
);
let surprise_dist = Self::update_axis(
    &mut self.surprise_bl, &surp_scores, eps, allowance,
    clip_sigmas, eta, cp_decay, true,
);
let coherence_dist = Self::update_axis(
    &mut self.coherence_bl, &coh_scores, eps, allowance,
    clip_sigmas, eta, cp_decay, k >= 2,
);
```

### Step 5.2 — Rewrite `update_axis()` · `sec:sentinel:clipplan-step-5-2-rewrite-update-axis`

```rust
/// Update one scoring axis: shared clip filter → fast EWMA → CUSUM.
///
/// §ALGO S-6.1.1 pipeline with clip-pressure EWMA (§ALGO S-6.4).
fn update_axis(
    bl: &mut AxisBaseline,
    scores: &[f64],
    eps: f64,
    cusum_allowance: f64,
    clip_sigmas: f64,
    eta: f64,
    clip_pressure_decay: f64,
    evolve: bool,
) -> ScoreDistribution {
    // ── Raw batch statistics (pre-clip) ─────────────
    let (min, max, sum) = scores
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY, 0.0_f64), |(mn, mx, s), &v| {
            (mn.min(v), mx.max(v), s + v)
        });

    #[allow(clippy::cast_precision_loss)]
    let mean = sum / scores.len() as f64;

    // Z-scores computed *before* updating the fast baseline.
    let max_z = bl.fast.z_score(max, eps);
    let mean_z = bl.fast.z_score(mean, eps);
    let baseline = bl.fast.snapshot();

    if evolve {
        // ── Per-axis effective clip (§ALGO S-6.4) ───
        //
        //   p = max(η, ρ̄)
        //   n_σ_eff = n_σ · (1 + p / (1 − p + ε))
        //
        let p = eta.max(bl.clip_pressure);
        let effective_clip = clip_sigmas * (1.0 + p / (1.0 - p + eps));

        // ── Single shared clip filter ───────────────
        // Computed against the fast EWMA's current baseline.
        // Cold-path bypass: ceiling() returns +∞ when cold.
        let ceiling = bl.fast.ceiling(effective_clip);

        let retained: Vec<f64> = scores.iter().copied().filter(|&v| v < ceiling).collect();

        // ── Update clip-pressure EWMA ───────────────
        //   ρ_t = 1 − |retained| / |total|
        //   ρ̄ = λ_ρ · ρ̄ + (1 − λ_ρ) · ρ_t
        #[allow(clippy::cast_precision_loss)]
        let rho_t = 1.0 - (retained.len() as f64 / scores.len() as f64);
        let alpha = 1.0 - clip_pressure_decay;
        bl.clip_pressure = clip_pressure_decay.mul_add(bl.clip_pressure, alpha * rho_t);

        // ── Fast EWMA: receives retained samples ────
        if retained.is_empty() {
            // All outliers — learn nothing this round.
            // clip_pressure was still updated above (it saw 100% clipping).
        } else {
            bl.fast.update_raw(&retained);
        }

        // ── CUSUM: receives retained samples, raw batch mean ──
        if !retained.is_empty() {
            bl.cusum.update_filtered(&retained, mean, cusum_allowance, eps);
        }
    }
    let cusum = bl.cusum.snapshot();

    ScoreDistribution {
        min,
        max,
        mean,
        max_z_score: max_z,
        mean_z_score: mean_z,
        baseline,
        cusum,
        clip_pressure: bl.clip_pressure,  // NEW — added in Phase 6
    }
}
```

### Step 5.3 — Verify formula equivalence · `sec:sentinel:clipplan-step-5-3-formula-equivalence`

The old formula was:

```
n_σ_eff = n_σ + n_σ · η / (1 − η + ε)
        = n_σ · (1 + η / (1 − η + ε))
```

The new formula is identical when `ρ̄ = 0` (because `p = max(η, 0) = η`). This means **at ρ̄ = 0 the behaviour is bit-for-bit identical** to the old code for the η term.

### Checkpoint · `sec:sentinel:clipplan-phase5-checkpoint`

At this point the convergence benchmark (ADR-S-013) should produce results negligibly different from before, since `clip_pressure` starts at 0 and no contamination is present in clean-traffic tests.

Run:
```
CARGO_PROFILE_DEV_OPT_LEVEL=3 cargo test --package torrust-sentinel convergence --all-features
```

---

## Phase 6 — Reporting: `ScoreDistribution::clip_pressure` · `sec:sentinel:clipplan-phase6-score-distribution`

**File: `src/report.rs`**

### Step 6.1 — Add field to `ScoreDistribution` · `sec:sentinel:clipplan-step-6-1-distribution-field`

```rust
pub struct ScoreDistribution {
    pub min: f64,
    pub max: f64,
    pub mean: f64,
    pub max_z_score: f64,
    pub mean_z_score: f64,
    pub baseline: BaselineSnapshot,
    pub cusum: CusumSnapshot,
    /// Current clip-pressure EWMA for this axis: ρ̄ ∈ [0, 1]  (§ALGO S-14.4).
    pub clip_pressure: f64,
}
```

### Step 6.2 — Fix all construction sites · `sec:sentinel:clipplan-step-6-2-construction-sites`

Every place that constructs a `ScoreDistribution` must now include `clip_pressure`.  The only production site is `update_axis()` (done in Phase 5).  Search for test/mock construction sites:

```
grep -rn "ScoreDistribution {" packages/sentinel/
```

Each mock/test site should set `clip_pressure: 0.0` (neutral).

### Checkpoint · `sec:sentinel:clipplan-phase6-checkpoint`

Compiles, all tests pass with the new field.

---

## Phase 7 — Reporting: `HealthReport` Clip-Pressure Distribution · `sec:sentinel:clipplan-phase7-health-report`

**File: `src/report.rs`**

### Step 7.1 — Add `ClipPressureDistribution` struct · `sec:sentinel:clipplan-step-7-1-distribution-struct`

```rust
/// Summary of clip-pressure EWMA values across active trackers (§ALGO S-14.11).
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClipPressureDistribution {
    /// Minimum clip-pressure EWMA across active tracker axes.
    pub min: f64,

    /// Maximum clip-pressure EWMA across active tracker axes.
    pub max: f64,

    /// Mean clip-pressure EWMA across active tracker axes.
    pub mean: f64,
}
```

### Step 7.2 — Add field to `HealthReport` · `sec:sentinel:clipplan-step-7-2-health-report-field`

```rust
/// Clip-pressure distribution across active trackers (§ALGO S-14.11).
pub clip_pressure_distribution: ClipPressureDistribution,
```

### Step 7.3 — Expose clip-pressure from `SubspaceTracker` · `sec:sentinel:clipplan-step-7-3-expose-from-tracker`

**File: `src/sentinel/tracker.rs`**

Add a helper to extract the four per-axis `clip_pressure` values:

```rust
/// Per-axis clip-pressure EWMA values [novelty, displacement, surprise, coherence].
pub(crate) fn clip_pressures(&self) -> [f64; 4] {
    [
        self.novelty_bl.clip_pressure,
        self.displacement_bl.clip_pressure,
        self.surprise_bl.clip_pressure,
        self.coherence_bl.clip_pressure,
    ]
}
```

### Step 7.4 — Populate in `health()` · `sec:sentinel:clipplan-step-7-4-populate-health`

**File: `src/sentinel/mod.rs`**

In the `health()` method, alongside the existing rank/maturity/geometry loops, accumulate clip-pressure min/max/sum across all 4 axes of all active trackers:

```rust
let mut cp_min = f64::INFINITY;
let mut cp_max = f64::NEG_INFINITY;
let mut cp_sum = 0.0_f64;
let mut cp_count = 0_u64;

for cell in self.cells.values() {
    for cp in cell.tracker.clip_pressures() {
        cp_min = cp_min.min(cp);
        cp_max = cp_max.max(cp);
        cp_sum += cp;
        cp_count += 1;
    }
}
```

Then in the `HealthReport` struct literal:

```rust
clip_pressure_distribution: ClipPressureDistribution {
    min: if cp_count > 0 { cp_min } else { 0.0 },
    max: if cp_count > 0 { cp_max } else { 0.0 },
    mean: if cp_count > 0 { cp_sum / cp_count as f64 } else { 0.0 },
},
```

Do the same for the early-return zero-trackers branch.

### Step 7.5 — Coordination `health()` too · `sec:sentinel:clipplan-step-7-5-coordination-health`

If the coordination tier's `HealthReport` / `CoordinationHealth` should also report clip-pressure, repeat the same pattern.  Check whether §14.11 mandates it — if not, skip for now.

### Checkpoint · `sec:sentinel:clipplan-phase7-checkpoint`

`cargo test --package torrust-sentinel --all-targets --all-features` passes.

---

## Phase 8 — Lifecycle Resets · `sec:sentinel:clipplan-phase8-lifecycle-resets`

### Step 8.1 — Coherence rank-drop reset (already covered) · `sec:sentinel:clipplan-step-8-1-rank-drop-reset`

`adapt_rank()` calls `self.coherence_bl.reset_cold()` when rank drops below 2.  Since Phase 2 added `clip_pressure = 0.0` to `reset_cold()`, **this gap is already closed**.  Verify with a quick read of `adapt_rank()`.

### Step 8.2 — Warm-up completion reset (§ALGO S-11.4) · `sec:sentinel:clipplan-step-8-2-warm-up-reset`

**File: `src/sentinel/tracker.rs`**

The spec says: when noise influence crosses the warm-up threshold (η goes below some value), zero all four axes' `clip_pressure`.

Currently **there is no explicit warm-up-completion callback** in `SubspaceTracker`.  The transition from warming → production happens implicitly as η decays.  Two options:

**Option A — Threshold check inside `update_maturity()`:**

Add a check after updating `noise_influence`: if it just crossed below a threshold (e.g. 0.01), zero clip-pressure on all four axes.

```rust
fn update_maturity(&mut self, batch_size: usize, is_noise: bool) {
    let old_eta = self.noise_influence;

    // ... existing η update ...

    // §ALGO S-11.4: when η crosses the warm-up threshold, zero
    // clip-pressure to prevent warm-up contamination echoing into
    // production scoring.
    const WARMUP_THRESHOLD: f64 = 0.01;
    if old_eta >= WARMUP_THRESHOLD && self.noise_influence < WARMUP_THRESHOLD {
        self.novelty_bl.clip_pressure = 0.0;
        self.displacement_bl.clip_pressure = 0.0;
        self.surprise_bl.clip_pressure = 0.0;
        self.coherence_bl.clip_pressure = 0.0;
    }
}
```

**Option B — Reset at noise injection completion:**

The warm-up pipeline already calls `seed_cusum_slow_from_baselines()` then `reset_cusum()` after noise injection completes (in `warm_inline()` / the staging pipeline).  Add `reset_clip_pressure()` alongside:

```rust
pub fn reset_clip_pressure(&mut self) {
    self.novelty_bl.clip_pressure = 0.0;
    self.displacement_bl.clip_pressure = 0.0;
    self.surprise_bl.clip_pressure = 0.0;
    self.coherence_bl.clip_pressure = 0.0;
}
```

Called from `warm_inline()` in `sentinel/mod.rs` after `cell.tracker.reset_cusum()`.

**Recommendation:** Use **both**.  Option B handles the explicit noise-injection completion path.  Option A catches edge cases where η decays through real-traffic dilution alone (e.g. if noise was partially skipped).

The warm-up threshold constant (`0.01`) should either:
- Be defined as `const WARMUP_THRESHOLD: f64 = 0.01` in `tracker.rs`, or
- Be configurable (a future config field).  For now, a constant is fine — the spec doesn't parameterise it.

### Checkpoint · `sec:sentinel:clipplan-phase8-checkpoint`

Write a test that injects noise, transitions to real traffic, and verifies `clip_pressure` is 0 after transition.

---

## Phase 9 — Memory Accounting Verification · `sec:sentinel:clipplan-phase9-memory-accounting`

### Step 9.1 — Count the fields · `sec:sentinel:clipplan-step-9-1-count-fields`

Each `AxisBaseline` now contains:
- `fast: EwmaStats` → 3 fields: `mean`, `variance`, `decay` (+ `warm` bool, packed) = effectively 3 f64s for accounting purposes (ignoring the bool/decay since they're config, not per-axis learned state) → For spec purposes: **2 floats** (mean, variance)
- `cusum: CusumAccumulator` → slow EWMA (2 floats: mean, variance) + accumulator (1 float) + steps (1 u64) → For spec purposes: **5 floats** (slow mean, slow variance, accumulator, steps counter, decay — but decay is config not state) → Actually counting learned state only: slow mean + slow variance + accumulator = **3 floats**
- `clip_pressure: f64` → **1 float**

Per-axis learned-state floats: fast\_mean(1) + fast\_var(1) + slow\_mean(1) + slow\_var(1) + cusum\_acc(1) + clip\_pressure(1) = **6 floats**.

Wait — let me re-read the ADR: "Per-axis baseline size grows from 7 to 8 floats". Let me re-count the *current* state:
- fast mean, fast variance = 2
- slow mean, slow variance = 2
- cusum accumulator = 1
- cusum steps_since_reset (u64, counts as 1) = 1
- warm (bool, but pad to 1) = ~1

That's 7 depending on how you count.  With `clip_pressure` = **8**. 4 axes × 8 = **32 floats**.

### Step 9.2 — Update any doc comments · `sec:sentinel:clipplan-step-9-2-doc-comments`

If `SubspaceTracker` or `AxisBaseline` has doc comments referencing memory accounting, update them.  Check `docs/algorithm.md` §4.3 if it's in-repo.

### No code change needed here — just verification. · `sec:sentinel:clipplan-phase9-no-code-change`

---

## Phase 10 — Update Convergence Diagnostics · `sec:sentinel:clipplan-phase10-convergence-diagnostics`

**File: `src/tests/convergence_diagnostics.rs`**

The convergence diagnostic test currently computes `eff_clip` as:

```rust
let eff_clip = cfg.clip_sigmas + cfg.clip_sigmas * eta / (1.0 - eta + cfg.eps);
```

Update to the new formula:

```rust
let p = eta.max(clip_pressure);
let eff_clip = cfg.clip_sigmas * (1.0 + p / (1.0 - p + cfg.eps));
```

Since `clip_pressure` is per-axis, the diagnostic will need to read it from the tracker report (via `ScoreDistribution::clip_pressure`).

Also update the column header in the diagnostic table to include `ρ̄`.

---

## Phase 11 — Integration Test: Contamination Self-Correction · `sec:sentinel:clipplan-phase11-contamination-test`

**File:** **`tests/` (new test file or extend `tests/spray_resistance.rs`)**

This is the **acceptance test** that validates the core value proposition: sustained contamination in production (η ≈ 0) should widen the clip ceiling automatically.

### Step 11.1 — Test outline · `sec:sentinel:clipplan-step-11-1-test-outline`

```rust
#[test]
fn clip_pressure_widens_ceiling_under_contamination() {
    // 1. Build a sentinel, inject noise, let it converge.
    // 2. Feed 200+ batches of clean traffic (scores ~ Normal(1.0, 0.1)).
    // 3. Assert clip_pressure ≈ 0 on all axes.
    // 4. Feed 50 batches of contaminated traffic (scores ~ Normal(1.0, 0.1)
    //    with 30% of samples replaced by 10.0 — well above 3σ ceiling).
    // 5. Assert clip_pressure > 0.2 on at least one axis.
    // 6. Assert the effective clip ceiling is wider than n_σ.
    // 7. Resume 200 batches of clean traffic.
    // 8. Assert clip_pressure decays back toward 0.
}
```

### Step 11.2 — Test: η-only behaviour preserved · `sec:sentinel:clipplan-step-11-2-eta-only-preserved`

```rust
#[test]
fn clip_pressure_zero_under_clean_traffic() {
    // Feed clean traffic with no contamination.
    // Assert clip_pressure stays ≈ 0 on all axes.
    // Assert effective-clip behaviour matches the old η-only formula.
}
```

### Step 11.3 — Test: warm-up reset clears clip-pressure · `sec:sentinel:clipplan-step-11-3-warm-up-reset-test`

```rust
#[test]
fn warm_up_completion_resets_clip_pressure() {
    // Inject noise (which may cause high clipping during warm-up).
    // Transition to real traffic.
    // Assert all clip_pressure values are 0 after transition.
}
```

---

## Phase 12 — Cross-Cutting Fixups · `sec:sentinel:clipplan-phase12-cross-cutting`

### Step 12.1 — Serde roundtrip · `sec:sentinel:clipplan-step-12-1-serde-roundtrip`

If `ScoreDistribution` is `Serialize/Deserialize`, the new field is automatically included.  Run `tests/serde_roundtrip.rs` to confirm.

### Step 12.2 — `AxisBaselineSnapshots` · `sec:sentinel:clipplan-step-12-2-baseline-snapshots`

In `src/report.rs`, `AxisBaselineSnapshots` is used for convergence tests.  Consider adding per-axis `clip_pressure` fields if tests need them:

```rust
pub struct AxisBaselineSnapshots {
    pub novelty: BaselineSnapshot,
    pub displacement: BaselineSnapshot,
    pub surprise: BaselineSnapshot,
    pub coherence: BaselineSnapshot,
    pub novelty_clip_pressure: f64,
    pub displacement_clip_pressure: f64,
    pub surprise_clip_pressure: f64,
    pub coherence_clip_pressure: f64,
}
```

Update `axis_baseline_snapshots()` in `tracker.rs` correspondingly.

Alternatively, provide a separate `clip_pressures()` method (done in Phase 7.3) and keep `AxisBaselineSnapshots` unchanged.  Prefer the separate method unless tests need both in a single struct.

### Step 12.3 — Grep for hardcoded clip formula · `sec:sentinel:clipplan-step-12-3-hardcoded-formula`

```bash
grep -rn 'clip_sigmas.*eta\|η.*clip' packages/sentinel/src/
```

Ensure no stale copies of the old `η/(1-η+ε)` formula remain in production code.  Test code / diagnostic comments referencing the old formula should be updated or annotated.

### Step 12.4 — Doc tests · `sec:sentinel:clipplan-step-12-4-doc-tests`

```bash
cargo test --package torrust-sentinel --doc --all-features
```

### Step 12.5 — No-default-features build · `sec:sentinel:clipplan-step-12-5-no-default-features`

```bash
cargo check --package torrust-sentinel --no-default-features
cargo test --package torrust-sentinel --no-default-features
```

---

## Phase 13 — Final Validation · `sec:sentinel:clipplan-phase13-final-validation`

### Step 13.1 — Full test suite (debug, optimised) · `sec:sentinel:clipplan-step-13-1-full-test-suite`

```bash
CARGO_PROFILE_DEV_OPT_LEVEL=3 cargo test --package torrust-sentinel --all-targets --all-features
```

### Step 13.2 — Release mode · `sec:sentinel:clipplan-step-13-2-release-mode`

```bash
cargo test --package torrust-sentinel --all-targets --all-features --release
```

### Step 13.3 — Clippy · `sec:sentinel:clipplan-step-13-3-clippy`

```bash
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

### Step 13.4 — Doc build · `sec:sentinel:clipplan-step-13-4-doc-build`

```bash
cargo doc --package torrust-sentinel --all-features --no-deps
```

### Step 13.5 — Benchmark comparison · `sec:sentinel:clipplan-step-13-5-benchmark`

Run the convergence benchmark before and after, compare round counts. The warm-up convergence should be ≤ the old round count (formula is identical when ρ̄ = 0).

---

## Execution Order Summary · `sec:sentinel:clipplan-execution-order`

| Phase | Gap(s) | Files touched | Risk |
|-------|--------|---------------|------|
| 1 | 2 | `config.rs`, `tests/config.rs` | Low — additive |
| 2 | 1 | `tracker.rs` | Low — unused field |
| 3 | 4 (partial) | `ewma.rs` | Low — new method, old preserved |
| 4 | 6 (partial) | `cusum.rs` | Low — new method, old preserved |
| 5 | 3, 4, 5, 6 | `tracker.rs` | **High** — core behavioural change |
| 6 | 7 | `report.rs` | Medium — struct change, many construction sites |
| 7 | 8 | `report.rs`, `mod.rs` | Medium — new reporting pipeline |
| 8 | 9, 10 | `tracker.rs`, `mod.rs` | Medium — lifecycle logic |
| 9 | 11 | docs only | Low — verification |
| 10 | — | `tests/convergence_diagnostics.rs` | Low — test update |
| 11 | — | `tests/` | Low — new tests |
| 12 | — | various | Low — fixups |
| 13 | — | — | Low — final validation |

**Phase 5 is the critical path.**  Everything before it is additive and safe.  Everything after it is reporting/testing.  If Phase 5 needs to be reverted, Phases 1–4 can remain in the codebase harmlessly.
