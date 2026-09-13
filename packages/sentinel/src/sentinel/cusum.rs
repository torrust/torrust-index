// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! CUSUM (Cumulative Sum) drift accumulator.
//!
//! Detects sustained upward drift of batch mean scores away from a
//! slow EWMA reference.  This is a one-sided Page's test: the
//! accumulator grows when the fast signal consistently exceeds the
//! slow baseline by more than a noise allowance, and resets to zero
//! when the deviation reverses.
//!
//! See `docs/algorithm.md` §ALGO S-7.3 for the full specification.
//!
//! Pure `f64` arithmetic — no `faer` dependency.

use crate::ewma::EwmaStats;
use crate::report::{BaselineSnapshot, CusumSnapshot};

/// One-sided CUSUM accumulator with a slow EWMA reference.
///
/// Each scoring axis owns one of these.  It pairs a slow EWMA
/// baseline (longer memory than the fast baseline in [`EwmaStats`])
/// with a cumulative sum that builds evidence of sustained drift.
///
/// The sentinel reports the raw accumulator value; the host decides
/// what level of accumulated drift warrants action.
#[derive(Debug, Clone)]
pub struct CusumAccumulator {
    /// Slow EWMA baseline — the reference the CUSUM measures drift from.
    slow: EwmaStats,

    /// The cumulative sum.  Non-negative (clamped at zero).
    accumulator: f64,

    /// Batches since the last reset (including post-noise reset).
    steps_since_reset: u64,
}

impl CusumAccumulator {
    /// Create a new CUSUM accumulator with the given slow decay factor.
    #[must_use]
    pub const fn new(slow_decay: f64) -> Self {
        Self {
            slow: EwmaStats::new(slow_decay),
            accumulator: 0.0,
            steps_since_reset: 0,
        }
    }

    /// Update the accumulator with a batch of per-sample scores.
    ///
    /// 1. Computes the gap: `batch_mean − slow_mean − κ·√slow_var`.
    /// 2. Accumulates: `S = max(0, S + gap)`.
    /// 3. Feeds the scores to the slow EWMA baseline.
    ///
    /// The gap is computed *before* updating the slow baseline so the
    /// reference reflects the prior state — matching the principle
    /// that scoring precedes evolution.
    ///
    /// `allowance_sigmas` is `κ_σ` from config — the noise tolerance
    /// in units of slow-baseline standard deviation.
    #[cfg(test)]
    pub fn update(&mut self, scores: &[f64], batch_mean: f64, allowance_sigmas: f64, eps: f64, clip_sigmas: f64) {
        let slow_mean = self.slow.mean();
        let slow_std = (self.slow.variance() + eps).sqrt();
        let allowance = allowance_sigmas * slow_std;

        let gap = batch_mean - slow_mean - allowance;
        self.accumulator = (self.accumulator + gap).max(0.0);

        // Now update the slow baseline with this batch.
        self.slow.update(scores, clip_sigmas);

        self.steps_since_reset += 1;
    }

    /// Reset the accumulator to zero.
    ///
    /// Called after noise injection (§ALGO S-7.4) and optionally by the host
    /// after acknowledging a regime change.
    pub const fn reset(&mut self) {
        self.accumulator = 0.0;
        self.steps_since_reset = 0;
    }

    /// Destroy all state — return to the freshly-constructed state.
    ///
    /// Resets the accumulator *and* the slow EWMA baseline to cold.
    /// Used when the scoring axis this accumulator tracks ceases to
    /// exist (e.g. coherence when rank drops below 2).
    pub const fn reset_cold(&mut self) {
        self.slow.reset_cold();
        self.accumulator = 0.0;
        self.steps_since_reset = 0;
    }

    /// Seed the slow EWMA from an external fast EWMA baseline.
    ///
    /// Closes the fast-slow gap after noise injection (ADR-S-013
    /// §6b, Option C).  The slow EWMA's mean and variance are set
    /// to the fast EWMA's current values so the CUSUM starts with
    /// the two baselines in agreement — eliminating the monotonic
    /// false-drift accumulation caused by the slow EWMA's 693-step
    /// half-life being unable to catch up to the fast EWMA.
    ///
    /// Should be called **after** noise injection completes and
    /// **before** `reset()`.
    pub const fn seed_slow_from(&mut self, fast: &EwmaStats) {
        self.slow.seed_from(fast);
    }

    /// Update the accumulator with **pre-filtered** samples.
    ///
    /// Identical to [`update`](Self::update) except the slow EWMA
    /// receives pre-filtered values via `update_raw()` instead of
    /// applying its own clip filter.  The CUSUM gap still uses
    /// `raw_batch_mean` (the pre-clip mean of the full batch).
    ///
    /// Used by the shared-filter pipeline (§ALGO S-6.1.1 step 5).
    pub fn update_filtered(&mut self, filtered: &[f64], raw_batch_mean: f64, allowance_sigmas: f64, eps: f64) {
        let slow_mean = self.slow.mean();
        let slow_std = (self.slow.variance() + eps).sqrt();
        let allowance = allowance_sigmas * slow_std;

        let gap = raw_batch_mean - slow_mean - allowance;
        self.accumulator = (self.accumulator + gap).max(0.0);

        self.slow.update_raw(filtered);
        self.steps_since_reset += 1;
    }

    /// Snapshot for inclusion in reports.
    #[must_use]
    pub const fn snapshot(&self) -> CusumSnapshot {
        CusumSnapshot {
            accumulator: self.accumulator,
            slow_baseline: BaselineSnapshot {
                mean: self.slow.mean(),
                variance: self.slow.variance(),
            },
            steps_since_reset: self.steps_since_reset,
        }
    }
}
