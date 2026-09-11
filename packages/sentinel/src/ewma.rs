// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Exponentially-weighted moving average (EWMA) statistics.
//!
//! Tracks a running mean and variance that exponentially decay old
//! observations. Used by the subspace tracker to maintain baselines
//! of "normal" anomaly scores.
//!
//! Outlier-resistant: observations beyond `clip_sigmas`·σ from the
//! current mean are rejected before updating, preventing an attacker
//! from poisoning the baseline with a single burst.
//!
//! No `faer` dependency — this is pure `f64` arithmetic.

use crate::report::BaselineSnapshot;

/// Exponentially-weighted running mean and variance.
///
/// After `update()` with a batch of values, the baseline reflects
/// a smoothed estimate of the central tendency and spread, biased
/// toward recent observations by the decay factor `λ`.
#[derive(Debug, Clone)]
pub struct EwmaStats {
    /// Decay factor (λ). Each old value's contribution shrinks by
    /// this factor per update. Higher = longer memory.
    decay: f64,

    /// Running weighted mean.
    mean: f64,

    /// Running weighted variance.
    variance: f64,

    /// Whether at least one real update has occurred.
    warm: bool,
}

impl EwmaStats {
    /// Create a new EWMA tracker with the given decay factor.
    ///
    /// Starts "cold" with `mean = 1.0`, `variance = 1.0` — deliberately
    /// wide to avoid extreme z-scores before the first real data arrives.
    #[must_use]
    pub const fn new(decay: f64) -> Self {
        Self {
            decay,
            mean: 1.0,
            variance: 1.0,
            warm: false,
        }
    }

    /// Current mean.
    #[must_use]
    pub const fn mean(&self) -> f64 {
        self.mean
    }

    /// Current variance.
    #[must_use]
    pub const fn variance(&self) -> f64 {
        self.variance
    }

    /// Whether the baseline has seen at least one update.
    #[must_use]
    pub const fn is_warm(&self) -> bool {
        self.warm
    }

    /// Return to the cold state — as if freshly constructed.
    ///
    /// Restores the placeholder mean and variance and marks the
    /// tracker as cold.  The next [`update`](Self::update) will
    /// enter the cold→warm initialisation path.
    pub const fn reset_cold(&mut self) {
        self.mean = 1.0;
        self.variance = 1.0;
        self.warm = false;
    }

    /// Snapshot of the current baseline for inclusion in reports.
    #[must_use]
    pub const fn snapshot(&self) -> BaselineSnapshot {
        BaselineSnapshot {
            mean: self.mean,
            variance: self.variance,
        }
    }

    /// Seed this EWMA's mean and variance from another EWMA.
    ///
    /// Used to close the fast-slow gap after noise injection
    /// (ADR-S-013 §6b, Option C): the slow EWMA in the CUSUM
    /// accumulator is seeded from the fast EWMA's converged
    /// baseline so the two start in agreement.
    ///
    /// Marks the receiver as warm if the source is warm.
    pub const fn seed_from(&mut self, source: &Self) {
        self.mean = source.mean;
        self.variance = source.variance;
        if source.warm {
            self.warm = true;
        }
    }

    /// Compute the z-score of a value against the current baseline.
    ///
    /// Returns `(value - mean) / (sqrt(variance) + eps)`.
    ///
    /// The stability constant sits outside the root rather than inside it, so
    /// it floors the deviation the score is divided by rather than the
    /// variance. The two readings differ exactly where the constant exists to
    /// matter — a baseline whose variance is small beside it — and the
    /// outside form is the one the package's own definition of this score
    /// states. Flooring the standard deviation also keeps the constant in the
    /// units of the quantity it guards, where flooring the variance would
    /// make its effect on the divisor depend on its own square root.
    ///
    /// The caller supplies `eps` (typically
    /// [`SentinelConfig::eps`](crate::config::SentinelConfig::eps))
    /// so that every component of the sentinel shares a single
    /// stability constant.
    #[must_use]
    pub fn z_score(&self, value: f64, eps: f64) -> f64 {
        (value - self.mean) / (self.variance.sqrt() + eps)
    }

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

    /// Update the baseline with a batch of new values.
    ///
    /// Values beyond `mean + clip_sigmas·√variance` are rejected
    /// (outlier resistance). If all values are outliers, the baseline
    /// is unchanged.
    ///
    /// Only the **upper** tail is clipped. Anomaly scores are
    /// non-negative and right-skewed — an attacker inflates them,
    /// never deflates them. A lower-tail bound would wrongly reject
    /// legitimate low scores during quiet periods.
    ///
    /// The filter is **skipped entirely on the first update** (while
    /// the tracker is still cold). The initial `mean = 1.0` /
    /// `variance = 1.0` are placeholders, not a real baseline — you
    /// can't define "outlier" without one.
    pub fn update(&mut self, values: &[f64], clip_sigmas: f64) {
        if values.is_empty() {
            return;
        }

        // When cold, accept everything — no real baseline to filter against.
        // Upper-tail only: anomaly scores are right-skewed.
        let normals: Vec<f64> = if self.warm {
            let ceiling = clip_sigmas.mul_add(self.variance.sqrt(), self.mean);
            let filtered: Vec<f64> = values.iter().copied().filter(|&v| v < ceiling).collect();
            if filtered.is_empty() {
                return; // all outliers — learn nothing
            }
            filtered
        } else {
            values.to_vec()
        };

        #[allow(clippy::cast_precision_loss)] // batch len ≪ 2^52
        let new_mean = normals.iter().sum::<f64>() / normals.len() as f64;

        if !self.warm {
            self.mean = new_mean;
            if normals.len() > 1 {
                let var = mean_squared_deviation(&normals, new_mean);
                self.variance = var.max(1e-4);
            }
            self.warm = true;
            return;
        }

        let alpha = 1.0 - self.decay;
        self.mean = self.decay.mul_add(self.mean, alpha * new_mean);

        if normals.len() > 1 {
            let var = mean_squared_deviation(&normals, new_mean).max(1e-4);
            self.variance = self.decay.mul_add(self.variance, alpha * var);
        }
    }
}

/// Mean squared deviation from the given mean (÷N, no Bessel's correction).
fn mean_squared_deviation(values: &[f64], mean: f64) -> f64 {
    #[allow(clippy::cast_precision_loss)] // batch len ≪ 2^52
    let n = values.len() as f64;
    values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n
}
