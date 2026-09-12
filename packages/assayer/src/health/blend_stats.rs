// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Blend weight statistics tracker: one concrete tracker carried by this
//! process (´dec:health:concrete-trackers´).
//!
//! Tracks the distribution of blend weights `w` from the assessment
//! pipeline (step 6). Publishes percentile statistics at regular
//! intervals via `ArcSwap`.
//!
//! # Design
//!
//! Same pattern as `ConcordanceTracker`: window under Mutex, published
//! statistics via `ArcSwap`, counter-triggered recomputation.
//!
//! # Cross-References
//!
//! - (´dec:health:independent-publication´) — why the published statistics
//!   swap on their own rather than with the model snapshot
//! - (´inv:monitoring:report-only´) — the statistics are read by a monitoring
//!   surface that reports and never acts on them

#![allow(dead_code)]

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;

// ═══════════════════════════════════════════════════════════════════════════════
// Configuration
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuration for blend statistics tracking.
#[derive(Clone, Debug)]
pub struct BlendStatisticsConfig {
    /// Capacity of the rolling window. Default: 5,000.
    pub window_capacity: usize,

    /// Number of assessments between statistics recomputations.
    /// Default: 1,000.
    pub publish_interval: u64,
}

impl Default for BlendStatisticsConfig {
    fn default() -> Self {
        Self {
            window_capacity: 5_000,
            publish_interval: 1_000,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Published Statistics
// ═══════════════════════════════════════════════════════════════════════════════

/// Published blend weight statistics.
///
/// Recomputed every `publish_interval` assessments (~50 μs cost).
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct BlendStatistics {
    /// Mean blend weight.
    pub mean: f64,
    /// 10th percentile.
    pub p10: f64,
    /// 50th percentile (median).
    pub p50: f64,
    /// 90th percentile.
    pub p90: f64,
    /// 99th percentile.
    pub p99: f64,
    /// Fraction of assessments with `w > 0.3` (anchor-dominated).
    pub fraction_anchor_dominated: f64,
    /// EWMA estimate of steady-state blend weight.
    pub w_infinity_estimate: f64,
    /// `true` when `w_infinity_estimate < 0.15` — the system has
    /// converged and the anchor's contribution has fallen to the small
    /// residual the steady state expects (´tab:warmup:stages´).
    pub steady_state: bool,
    /// Current window size (may be less than capacity during warm-up).
    pub window_size: usize,
}

impl Default for BlendStatistics {
    fn default() -> Self {
        Self {
            mean: 0.0,
            p10: 0.0,
            p50: 0.0,
            p90: 0.0,
            p99: 0.0,
            fraction_anchor_dominated: 0.0,
            w_infinity_estimate: 0.0,
            steady_state: false,
            window_size: 0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Blend Statistics Tracker
// ═══════════════════════════════════════════════════════════════════════════════

/// Tracks blend weight distribution and publishes percentiles.
///
/// Updated at assessment step 6 with the blend weight `w`.
/// Recomputes statistics every `publish_interval` observations.
pub struct BlendStatisticsTracker {
    /// Rolling window of blend weights.
    window: Mutex<BlendWindow>,
    /// Published statistics (lock-free reads via `ArcSwap`).
    published: ArcSwap<BlendStatistics>,
    /// Total observations.
    counter: AtomicU64,
    /// EWMA of blend weight (γ = 0.999).
    ///
    /// `None` until the first observation, eliminating cold-start
    /// bias toward zero.  Initialised to the first observed `w`.
    w_ewma: Mutex<Option<f64>>,
    /// Configuration.
    config: BlendStatisticsConfig,
}

/// Fixed-capacity circular buffer for blend weights.
struct BlendWindow {
    /// Buffer storage.
    data: Vec<f64>,
    /// Write position (modular).
    write_pos: usize,
    /// Number of elements stored (up to capacity).
    len: usize,
    /// Capacity.
    capacity: usize,
}

impl BlendWindow {
    /// Creates a new window with the given capacity.
    fn new(capacity: usize) -> Self {
        Self {
            data: vec![0.0; capacity],
            write_pos: 0,
            len: 0,
            capacity,
        }
    }

    /// Pushes a new value, evicting the oldest if full.
    fn push(&mut self, value: f64) {
        self.data[self.write_pos] = value;
        self.write_pos = (self.write_pos + 1) % self.capacity;
        if self.len < self.capacity {
            self.len += 1;
        }
    }

    /// Returns the current length.
    const fn len(&self) -> usize {
        self.len
    }

    /// Returns an iterator over current elements (oldest first).
    fn iter(&self) -> impl Iterator<Item = f64> + '_ {
        let start = if self.len < self.capacity { 0 } else { self.write_pos };
        (0..self.len).map(move |i| self.data[(start + i) % self.capacity])
    }
}

impl BlendStatisticsTracker {
    /// Creates a new tracker with the given configuration.
    #[must_use]
    pub fn new(config: BlendStatisticsConfig) -> Self {
        let capacity = config.window_capacity;
        Self {
            window: Mutex::new(BlendWindow::new(capacity)),
            published: ArcSwap::from_pointee(BlendStatistics::default()),
            counter: AtomicU64::new(0),
            w_ewma: Mutex::new(None),
            config,
        }
    }

    /// Creates a tracker with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(BlendStatisticsConfig::default())
    }

    /// Observes a blend weight from assessment step 6.
    ///
    /// Pushes to the window and triggers recomputation at intervals.
    pub fn observe(&self, w: f64) {
        // Update EWMA (first observation seeds the value)
        {
            let mut ewma = self.w_ewma.lock().expect("w_ewma lock poisoned");
            *ewma = Some(ewma.map_or(w, |prev| 0.001f64.mul_add(w, 0.999 * prev)));
        }

        // Push to window
        {
            let mut window = self.window.lock().expect("blend window lock poisoned");
            window.push(w);
        }

        // Increment and check interval
        let new_count = self.counter.fetch_add(1, Ordering::Relaxed) + 1;
        if new_count.is_multiple_of(self.config.publish_interval) {
            self.recompute();
        }
    }

    /// Returns the current published statistics.
    #[must_use]
    pub fn current(&self) -> Arc<BlendStatistics> {
        self.published.load_full()
    }

    /// Returns the total number of observations.
    #[must_use]
    pub fn total_observations(&self) -> u64 {
        self.counter.load(Ordering::Relaxed)
    }

    /// Recomputes percentile statistics from the window.
    fn recompute(&self) {
        let window = self.window.lock().expect("blend window lock poisoned");
        let len = window.len();
        if len == 0 {
            return;
        }

        let mut sorted: Vec<f64> = window.iter().collect();
        drop(window); // Release lock before computation

        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        #[allow(clippy::cast_precision_loss)]
        let mean = sorted.iter().sum::<f64>() / len as f64;

        #[allow(clippy::cast_precision_loss)]
        let fraction_anchor_dominated = sorted.iter().filter(|&&w| w > 0.3).count() as f64 / len as f64;

        let w_infinity_estimate = self.w_ewma.lock().expect("w_ewma lock poisoned").unwrap_or(0.0);

        let stats = BlendStatistics {
            mean,
            p10: percentile(&sorted, 10.0),
            p50: percentile(&sorted, 50.0),
            p90: percentile(&sorted, 90.0),
            p99: percentile(&sorted, 99.0),
            fraction_anchor_dominated,
            w_infinity_estimate,
            steady_state: w_infinity_estimate < 0.15,
            window_size: len,
        };

        self.published.store(Arc::new(stats));
    }
}

/// Computes a percentile from a sorted array.
///
/// Uses linear interpolation.
fn percentile(sorted: &[f64], pct: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    if sorted.len() == 1 {
        return sorted[0];
    }

    #[allow(clippy::cast_precision_loss)]
    let rank = pct / 100.0 * (sorted.len() - 1) as f64;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lower = rank.floor() as usize;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let upper = rank.ceil() as usize;
    let frac = rank - rank.floor();

    if upper >= sorted.len() {
        sorted[sorted.len() - 1]
    } else {
        (1.0 - frac).mul_add(sorted[lower], frac * sorted[upper])
    }
}

impl std::fmt::Debug for BlendStatisticsTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let stats = self.published.load();
        f.debug_struct("BlendStatisticsTracker")
            .field("total_observations", &self.counter.load(Ordering::Relaxed))
            .field("mean", &stats.mean)
            .field("p50", &stats.p50)
            .finish_non_exhaustive()
    }
}
