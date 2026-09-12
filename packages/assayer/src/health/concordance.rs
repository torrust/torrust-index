// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Concordance tracking and threshold recalibration.
//!
//! The [`ConcordanceTracker`] maintains a rolling window of per-axis
//! max z-scores and recalibrates concordance thresholds at regular
//! intervals.
//!
//! # Design
//!
//! - Lives on shared Assayer state (not the working copy)
//! - Window protected by Mutex (cheap: ~20ns acquisition)
//! - Thresholds served via `ArcSwap` (lock-free: ~15ns)
//! - Recalibration synchronous (~50μs), batched every N observations
//! - Checkpoint snapshots clone the bounded window under the same Mutex
//!
//! # Cross-References
//!
//! - (´dec:health:concrete-trackers´) — why this process carries its own
//!   concrete tracker
//! - (´alg:feature:concordance-recalibration´) — the threshold adaptation this
//!   tracker performs
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`per_entity_concordance_flags_and_evicts_least_recent`] | wellness | Five disagreements trail an aggregate AUC of four fifths by more than the configured deficit and are flagged. The same fixture then fills a two-entity map, refreshes the flagged entity, and proves that the other entity is the one least-recently-observed eviction removes (´alg:monitoring:per-entity-concordance´). |
//! | [`per_entity_concordance_honours_configured_gate_and_deficit`] | wellness | The configured gate and deficit are exact: an entity stays unflagged one label below the gate, reaches the gate without being flagged when its concordance equals aggregate AUC minus the deficit, and flags when the configured deficit moves just below that gap. |
//! | [`circular_buffer_push_and_len`] | wellness | The window reports the observations actually written to it, not the storage it reserved: a buffer sized for five reads as empty until the first push and as three after three. Recalibration divides by the reported length, so counting unwritten slots would dilute every percentile with placeholder zeroes during the fill. |
//! | [`circular_buffer_wraps_at_capacity`] | wellness | A full window evicts its oldest entry to make room and keeps yielding what remains in oldest-to-newest order across the wrap point: after a fourth push into a buffer of three, the first observation is gone and the survivors iterate in the sequence they arrived. The physical write position wraps, but the observation order a reader sees does not. |
//! | [`circular_buffer_zero_capacity`] | wellness | A window configured with no capacity accepts pushes and keeps nothing, rather than dividing by zero or indexing past its empty storage. A degenerate configuration should disable the statistic it feeds, not bring down the Assayer that merely observes. |
//! | [`percentile_empty_slice`] | wellness | A percentile taken over no observations answers zero rather than panicking or returning a NaN. Recalibration reaches this case whenever an axis contributed nothing finite, and a NaN threshold would silently disable every comparison made against it thereafter. |
//! | [`percentile_single_element`] | wellness | A lone observation is every percentile of itself — the median, the bottom and the top all return it. With one sample there is no spread to interpolate across, and any answer other than the value itself would be invented rather than measured. |
//! | [`percentile_median`] | wellness | When the requested percentile lands exactly on a rank, the observation at that rank is returned unaltered: the midpoint of five evenly spaced values is the middle one. An exact hit must not be nudged by the interpolation machinery that serves the inexact cases. |
//! | [`percentile_80th`] | wellness | A percentile falling between two ranks is interpolated between its neighbours rather than rounded to one of them: the eightieth percentile of ten evenly spaced values sits between the eighth and ninth, not on either. Interpolation keeps the calibrated threshold a smooth function of the window, so one arriving observation shifts it a little instead of stepping it. |
//! | [`percentile_clamped`] | wellness | A percentile outside the sensible range is clamped to the ends of the sample rather than read as an index: a negative request returns the smallest observation and one above a hundred returns the largest. The percentile is configuration, so a mistyped one degrades to an extreme threshold instead of reaching outside the window. |

// Items in this module are re-exported via crate::health and used by tests.
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use arc_swap::ArcSwap;

use crate::types::{EntityKey, SCORING_AXIS_COUNT};

/// Maximum number of entities retained for concordance reporting.
pub const PER_ENTITY_CONCORDANCE_CAPACITY: usize = 10_000;

// ═══════════════════════════════════════════════════════════════════════════════
// Concordance Configuration
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuration for the concordance tracker.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConcordanceConfig {
    /// Capacity of the rolling window (default: 5,000).
    /// TODO ´todo:code:default-should-become-10-000-per´: default should become 10,000 per-Sentinel
    /// sub-score entries after the aggregate observation API is widened. This
    /// is separate from the shipped label-outcome concordance map.
    pub window_capacity: usize,

    /// Recalibration interval in observations (default: 1,000).
    pub recalibration_interval: u64,

    /// Percentile for threshold calibration (default: 80).
    pub calibration_percentile: f64,
}

impl Default for ConcordanceConfig {
    fn default() -> Self {
        Self {
            window_capacity: DEFAULT_WINDOW_CAPACITY,
            recalibration_interval: DEFAULT_RECALIBRATION_INTERVAL,
            calibration_percentile: DEFAULT_CALIBRATION_PERCENTILE,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Concordance Tracker
// ═══════════════════════════════════════════════════════════════════════════════

/// Tracks per-axis max z-scores and recalibrates concordance thresholds.
///
/// TODO ´todo:code:this-currently-stores-one-aggregate-observation´: this currently stores one aggregate observation per
/// assessment. One `[N, D, S, C]` entry per reporting Sentinel and a
/// 10,000-entry sub-score window remains a separate aggregate-calibration
/// refinement.
///
/// Maintains a circular buffer of recent observations. At regular intervals
/// (every `recalibration_interval` observations), computes the 80th
/// percentile per axis and updates the thresholds.
///
/// # Thread Safety
///
/// - `observe()` acquires the window Mutex (~20ns)
/// - `current_thresholds()` is lock-free via `ArcSwap` (~15ns)
/// - Recalibration is synchronous during `observe()` (~50μs)
pub struct ConcordanceTracker {
    /// Rolling window of per-axis max z-scores.
    window: Mutex<CircularBuffer>,

    /// Current concordance thresholds (one per axis).
    thresholds: ArcSwap<[f64; SCORING_AXIS_COUNT]>,

    /// Total observations (for recalibration interval).
    /// TODO ´todo:code:split-this-into-an-assessment-counter´: split this into an assessment counter for the
    /// recalibration interval and sub-score entry count in the window.
    count: AtomicU64,

    /// Configuration parameters.
    config: ConcordanceConfig,

    /// Number of recalibrations completed.
    calibrations_completed: AtomicU64,
}

/// Serializable checkpoint state for [`ConcordanceTracker`].
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConcordanceCheckpointState {
    /// Rolling observations in oldest-to-newest order.
    pub window: Vec<[f64; SCORING_AXIS_COUNT]>,

    /// Current concordance thresholds (one per axis).
    pub thresholds: [f64; SCORING_AXIS_COUNT],

    /// Total observations since creation.
    pub count: u64,

    /// Number of recalibrations completed.
    pub calibrations_completed: u64,
}

impl Default for ConcordanceCheckpointState {
    fn default() -> Self {
        Self {
            window: Vec::new(),
            thresholds: INITIAL_THRESHOLDS,
            count: 0,
            calibrations_completed: 0,
        }
    }
}

impl ConcordanceTracker {
    /// Creates a new concordance tracker with the given configuration.
    #[must_use]
    pub fn new(config: ConcordanceConfig) -> Self {
        Self {
            window: Mutex::new(CircularBuffer::new(config.window_capacity)),
            thresholds: ArcSwap::new(Arc::new(INITIAL_THRESHOLDS)),
            count: AtomicU64::new(0),
            config,
            calibrations_completed: AtomicU64::new(0),
        }
    }

    /// Creates a new concordance tracker with default configuration.
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(ConcordanceConfig::default())
    }

    /// Restores a concordance tracker from checkpoint state.
    #[must_use]
    pub fn from_checkpoint(config: ConcordanceConfig, state: ConcordanceCheckpointState) -> Self {
        Self {
            window: Mutex::new(CircularBuffer::from_observations(config.window_capacity, state.window)),
            thresholds: ArcSwap::new(Arc::new(state.thresholds)),
            count: AtomicU64::new(state.count),
            config,
            calibrations_completed: AtomicU64::new(state.calibrations_completed),
        }
    }

    /// Observes a new set of per-axis max z-scores.
    ///
    /// TODO ´todo:code:change-this-signature-to-accept-f64´: change this signature to accept `&[[f64; 4]]`,
    /// push every reporting Sentinel's sub-score entry, and increment the
    /// recalibration assessment counter once per call. This does not affect
    /// the label-outcome concordance map.
    ///
    /// Pushes the observation to the circular buffer and triggers
    /// recalibration if the count is a multiple of `recalibration_interval`.
    ///
    /// # Arguments
    ///
    /// * `per_axis_max_z` — Maximum z-score observed per axis (4 axes)
    ///
    /// # Panics
    ///
    /// Panics if the internal Mutex is poisoned.
    pub fn observe(&self, per_axis_max_z: [f64; SCORING_AXIS_COUNT]) {
        // Push to window under Mutex
        {
            let mut window = self.window.lock().expect("concordance window lock poisoned");
            window.push(per_axis_max_z);
        }

        // Increment count
        let new_count = self.count.fetch_add(1, Ordering::Relaxed) + 1;

        // Check if recalibration is needed
        if new_count.is_multiple_of(self.config.recalibration_interval) {
            self.recalibrate();
        }
    }

    /// Returns the current concordance thresholds.
    ///
    /// Lock-free read via `ArcSwap` (~15ns).
    #[must_use]
    pub fn current_thresholds(&self) -> [f64; SCORING_AXIS_COUNT] {
        **self.thresholds.load()
    }

    /// Returns a health snapshot of the concordance tracker.
    ///
    /// # Panics
    ///
    /// Panics if the internal Mutex is poisoned.
    #[must_use]
    pub fn health(&self) -> ConcordanceHealth {
        let window = self.window.lock().expect("concordance window lock poisoned");
        ConcordanceHealth {
            window_size: window.len(),
            window_capacity: self.config.window_capacity,
            calibrations_completed: self.calibrations_completed.load(Ordering::Relaxed),
            current_thresholds: self.current_thresholds(),
            total_observations: self.count.load(Ordering::Relaxed),
            per_entity: HashMap::new(),
            per_entity_capacity: 0,
            per_entity_evictions: 0,
        }
    }

    /// Returns a serializable checkpoint snapshot of the tracker.
    ///
    /// # Panics
    ///
    /// Panics if the internal Mutex is poisoned.
    #[must_use]
    pub fn checkpoint_state(&self) -> ConcordanceCheckpointState {
        let window = self.window.lock().expect("concordance window lock poisoned");
        ConcordanceCheckpointState {
            window: window.iter().copied().collect(),
            thresholds: self.current_thresholds(),
            count: self.count.load(Ordering::Relaxed),
            calibrations_completed: self.calibrations_completed.load(Ordering::Relaxed),
        }
    }

    /// Performs threshold recalibration.
    ///
    /// Computes the configured percentile per axis from the window
    /// and updates the thresholds via `ArcSwap`.
    fn recalibrate(&self) {
        let window = self.window.lock().expect("concordance window lock poisoned");

        if window.is_empty() {
            return;
        }

        let mut new_thresholds = [0.0; 4];

        for axis in 0..4 {
            let mut values: Vec<f64> = window.iter().map(|obs| obs[axis]).filter(|v| v.is_finite()).collect();
            values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

            new_thresholds[axis] = percentile(&values, self.config.calibration_percentile).max(THRESHOLD_FLOOR);
        }

        drop(window); // Release lock before ArcSwap store

        self.thresholds.store(Arc::new(new_thresholds));
        // TODO ´todo:code:emit-convergenceevent-concordancefirstcalibration´: emit `ConvergenceEvent::ConcordanceFirstCalibration`
        // on the first successful empirical threshold calibration once the
        // tracker owns or can reach the one bounded event channel
        // (´dec:health:bounded-events´).
        self.calibrations_completed.fetch_add(1, Ordering::Relaxed);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Concordance Health
// ═══════════════════════════════════════════════════════════════════════════════

/// Health snapshot of the concordance tracker.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConcordanceHealth {
    /// Current number of observations in the window.
    pub window_size: usize,

    /// Maximum capacity of the window.
    pub window_capacity: usize,

    /// Number of recalibrations completed.
    pub calibrations_completed: u64,

    /// Current concordance thresholds (one per axis).
    pub current_thresholds: [f64; SCORING_AXIS_COUNT],

    /// Total observations since creation.
    pub total_observations: u64,

    /// Per-entity outcome/risk-sign concordance.
    pub per_entity: HashMap<EntityKey, EntityConcordanceHealth>,

    /// Maximum number of entities retained by the per-entity map.
    pub per_entity_capacity: usize,

    /// Lifetime entities evicted from the bounded map.
    pub per_entity_evictions: u64,
}

/// Reported concordance for one entity.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntityConcordanceHealth {
    /// Labels observed for the entity while it was retained.
    pub label_count: u64,
    /// Fraction whose outcome sign agreed with the assessment-time risk sign.
    pub concordance: f64,
    /// Whether concordance trails aggregate rank discrimination by the
    /// configured deficit after the minimum label count.
    pub flagged: bool,
}

/// Bounded label-time state behind per-entity concordance health.
///
/// Existing entities update in constant time. A new entity pays a linear scan
/// only when a full map must evict the least recently observed entry; this
/// keeps the hot repeat-entity path cheap without maintaining a second linked
/// allocation per key.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PerEntityConcordanceTracker {
    /// Retained state keyed by entity.
    entries: HashMap<EntityKey, PerEntityConcordanceEntry>,
    /// Monotone recency sequence assigned on every observation.
    sequence: u64,
    /// Lifetime least-recently-observed eviction count.
    evictions: u64,
    /// Maximum number of retained entities.
    capacity: usize,
}

/// Running concordance state for one retained entity.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
struct PerEntityConcordanceEntry {
    /// Labels observed while this entity remained retained.
    label_count: u64,
    /// Labels whose outcome and frozen risk signs agreed.
    agreements: u64,
    /// Sequence of this entity's latest observation.
    last_observed: u64,
}

impl Default for PerEntityConcordanceTracker {
    fn default() -> Self {
        Self::new(PER_ENTITY_CONCORDANCE_CAPACITY)
    }
}

impl PerEntityConcordanceTracker {
    /// Creates a bounded tracker with least-recently-observed eviction.
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self {
            entries: HashMap::new(),
            sequence: 0,
            evictions: 0,
            capacity,
        }
    }

    /// Records whether one label's outcome sign agreed with its frozen risk sign.
    pub fn observe(&mut self, entity: &EntityKey, agrees: bool) {
        self.sequence = self.sequence.saturating_add(1);
        if self.capacity == 0 {
            return;
        }
        if !self.entries.contains_key(entity)
            && self.entries.len() == self.capacity
            && let Some(oldest) = self
                .entries
                .iter()
                .min_by_key(|(_, entry)| entry.last_observed)
                .map(|(key, _)| key.clone())
        {
            self.entries.remove(&oldest);
            self.evictions = self.evictions.saturating_add(1);
        }
        let entry = self.entries.entry(entity.clone()).or_insert(PerEntityConcordanceEntry {
            label_count: 0,
            agreements: 0,
            last_observed: self.sequence,
        });
        entry.label_count = entry.label_count.saturating_add(1);
        entry.agreements = entry.agreements.saturating_add(u64::from(agrees));
        entry.last_observed = self.sequence;
    }

    /// Projects the bounded state against the current aggregate discrimination.
    #[must_use]
    pub fn health(&self, aggregate_auc: Option<f64>) -> HashMap<EntityKey, EntityConcordanceHealth> {
        let monitoring = crate::config::types::MonitoringConfig::default();
        self.health_with_thresholds(
            aggregate_auc,
            monitoring.min_concordance_labels,
            monitoring.concordance_deficit_threshold,
        )
    }

    /// Projects the bounded state against configured concordance thresholds.
    #[must_use]
    pub(crate) fn health_with_thresholds(
        &self,
        aggregate_auc: Option<f64>,
        min_labels: u64,
        deficit_threshold: f64,
    ) -> HashMap<EntityKey, EntityConcordanceHealth> {
        self.entries
            .iter()
            .map(|(entity, entry)| {
                #[allow(clippy::cast_precision_loss)] // Justified: the public statistic is a floating ratio of monotone counters.
                let concordance = entry.agreements as f64 / entry.label_count as f64;
                let flagged =
                    entry.label_count >= min_labels && aggregate_auc.is_some_and(|auc| concordance < auc - deficit_threshold);
                (
                    entity.clone(),
                    EntityConcordanceHealth {
                        label_count: entry.label_count,
                        concordance,
                        flagged,
                    },
                )
            })
            .collect()
    }

    /// Configured entity capacity.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Lifetime eviction count.
    #[must_use]
    pub const fn evictions(&self) -> u64 {
        self.evictions
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Circular Buffer
// ═══════════════════════════════════════════════════════════════════════════════

/// Fixed-capacity circular buffer for observations.
///
/// When full, the oldest entry is evicted on push.
struct CircularBuffer {
    /// Storage for observations.
    buffer: Vec<[f64; SCORING_AXIS_COUNT]>,

    /// Write index (wraps at capacity).
    write_index: usize,

    /// Number of valid entries (capped at capacity).
    len: usize,

    /// Maximum capacity.
    capacity: usize,
}

impl CircularBuffer {
    /// Creates a new circular buffer with the given capacity.
    fn new(capacity: usize) -> Self {
        Self {
            buffer: vec![[0.0; 4]; capacity],
            write_index: 0,
            len: 0,
            capacity,
        }
    }

    /// Creates a circular buffer from observations in oldest-to-newest order.
    fn from_observations(capacity: usize, observations: Vec<[f64; SCORING_AXIS_COUNT]>) -> Self {
        let mut buffer = Self::new(capacity);
        let keep_from = observations.len().saturating_sub(capacity);
        for observation in observations.into_iter().skip(keep_from) {
            buffer.push(observation);
        }
        buffer
    }

    /// Pushes an observation, evicting the oldest if full.
    fn push(&mut self, observation: [f64; SCORING_AXIS_COUNT]) {
        if self.capacity == 0 {
            return;
        }

        self.buffer[self.write_index] = observation;
        self.write_index = (self.write_index + 1) % self.capacity;
        self.len = self.len.saturating_add(1).min(self.capacity);
    }

    /// Returns the number of valid entries.
    const fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the buffer is empty.
    const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Iterates over all valid entries.
    fn iter(&self) -> impl Iterator<Item = &[f64; SCORING_AXIS_COUNT]> {
        let start = if self.len < self.capacity { 0 } else { self.write_index };

        (0..self.len).map(move |i| &self.buffer[(start + i) % self.capacity])
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Computes the nth percentile of a sorted slice.
///
/// Uses linear interpolation between nearest ranks.
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }

    if sorted.len() == 1 {
        return sorted[0];
    }

    let p = p.clamp(0.0, 100.0) / 100.0;

    #[allow(clippy::cast_precision_loss)]
    let rank = p * (sorted.len() - 1) as f64;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let lower = rank.floor() as usize;
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let upper = rank.ceil() as usize;
    let frac = rank - rank.floor();

    if lower == upper || upper >= sorted.len() {
        sorted[lower.min(sorted.len() - 1)]
    } else {
        sorted[upper].mul_add(frac, sorted[lower] * (1.0 - frac))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Configuration Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Default window capacity.
///
/// TODO ´todo:code:update-to-10-000-after-switching´: update to `10_000` after switching from aggregate
/// observations to per-Sentinel sub-score entries.
///
/// ´const:assayer:concordance-window-depth´ (´alg:const:count´)
/// ´const:assayer:concordance-window-depth-count-5000´
const DEFAULT_WINDOW_CAPACITY: usize = 5_000;

/// Default recalibration interval (observations): the thresholds are
/// recomputed every thousand assessments
/// (´alg:feature:concordance-recalibration´).
///
/// ´const:assayer:concordance-recalibration-cadence´ (´alg:const:count´)
/// ´const:assayer:concordance-recalibration-cadence-count-1000´
const DEFAULT_RECALIBRATION_INTERVAL: u64 = 1_000;

/// Default calibration percentile: each axis takes the eightieth
/// percentile of its window (´alg:feature:concordance-recalibration´).
///
/// ´const:assayer:concordance-threshold-percentile´ (´alg:const:scalar´)
/// ´const:assayer:concordance-threshold-percentile-scalar-80p0´
const DEFAULT_CALIBRATION_PERCENTILE: f64 = 80.0;

/// Initial concordance thresholds: one half until the first
/// recalibration (´alg:warmup:concordance-bootstrap´).
///
/// ´const:assayer:concordance-initial-thresholds´ (´alg:const:form´)
/// ´const:assayer:concordance-initial-thresholds-form-xd5eb677e´
const INITIAL_THRESHOLDS: [f64; SCORING_AXIS_COUNT] = [0.5; SCORING_AXIS_COUNT];

/// Floor for recalibrated thresholds.
///
/// Prevents degenerate near-zero thresholds when all observed scores
/// cluster near zero (e.g. during early convergence). Without the floor,
/// `score > θ` fires on any positive signal, defeating calibration.
/// It floors the percentile the recalibration computes
/// (´alg:feature:concordance-recalibration´): the smallest threshold that
/// still discriminates, binding only while the window is degenerate —
/// a statistical guard rather than an arithmetic one
/// (´tab:degradation:guard-magnitudes´).
///
/// ´const:assayer:concordance-threshold-floor´ (´alg:const:scalar´)
/// ´const:assayer:concordance-threshold-floor-scalar-0p01´
const THRESHOLD_FLOOR: f64 = 0.01;

// ═══════════════════════════════════════════════════════════════════════════════
// Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// Five disagreements trail an aggregate AUC of four fifths by more than
    /// the configured deficit and are flagged. The same fixture then fills a
    /// two-entity map, refreshes the flagged entity, and proves that the other
    /// entity is the one least-recently-observed eviction removes
    /// (´alg:monitoring:per-entity-concordance´).
    ///
    /// ´claim:wellness:per-entity-concordance-flags-at-the-gate-and-evicts-the-least-recently-observed´
    /// ´test:unit:per-entity-concordance-flags-and-evicts-least-recent´
    #[test]
    fn per_entity_concordance_flags_and_evicts_least_recent() {
        let first = EntityKey::new(b"first".to_vec());
        let second = EntityKey::new(b"second".to_vec());
        let third = EntityKey::new(b"third".to_vec());
        let mut tracker = PerEntityConcordanceTracker::new(2);

        for _ in 0..5 {
            tracker.observe(&first, false);
        }
        tracker.observe(&second, true);
        tracker.observe(&first, false);
        tracker.observe(&third, true);

        let health = tracker.health(Some(0.8));
        let first_health = &health[&first];
        assert_eq!(first_health.label_count, 6);
        assert_eq!(first_health.concordance.to_bits(), 0.0f64.to_bits());
        assert!(first_health.flagged);
        assert!(!health.contains_key(&second), "least recently observed entity is evicted");
        assert!(health.contains_key(&third));
        assert_eq!(tracker.evictions(), 1);
    }

    /// The configured gate and deficit are exact: an entity stays unflagged
    /// one label below the gate, reaches the gate without being flagged when
    /// its concordance equals aggregate AUC minus the deficit, and flags when
    /// the configured deficit moves just below that gap.
    ///
    /// ´claim:wellness:per-entity-concordance-honours-the-configured-gate-and-strict-deficit´
    /// ´test:unit:per-entity-concordance-honours-configured-gate-and-deficit´
    #[test]
    fn per_entity_concordance_honours_configured_gate_and_deficit() {
        let entity = EntityKey::new(b"boundary".to_vec());
        let mut tracker = PerEntityConcordanceTracker::new(1);
        tracker.observe(&entity, true);
        tracker.observe(&entity, false);
        tracker.observe(&entity, true);

        let below_gate = tracker.health_with_thresholds(Some(0.75), 4, 0.25);
        assert!(!below_gate[&entity].flagged);

        tracker.observe(&entity, false);
        let at_boundary = tracker.health_with_thresholds(Some(0.75), 4, 0.25);
        assert_eq!(at_boundary[&entity].concordance.to_bits(), 0.5f64.to_bits());
        assert!(
            !at_boundary[&entity].flagged,
            "equality does not trail the configured deficit"
        );

        let beyond_boundary = tracker.health_with_thresholds(Some(0.75), 4, 0.249);
        assert!(beyond_boundary[&entity].flagged);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Circular Buffer Tests
    // ─────────────────────────────────────────────────────────────────────────

    /// The window reports the observations actually written to it, not the
    /// storage it reserved: a buffer sized for five reads as empty until the
    /// first push and as three after three. Recalibration divides by the
    /// reported length, so counting unwritten slots would dilute every
    /// percentile with placeholder zeroes during the fill.
    ///
    /// ´claim:wellness:the-window-counts-observations-written-not-storage-reserved´
    /// ´test:unit:circular-buffer-push-and-len´
    #[test]
    fn circular_buffer_push_and_len() {
        let mut buf = CircularBuffer::new(5);
        assert_eq!(buf.len(), 0);
        assert!(buf.is_empty());

        buf.push([1.0; 4]);
        assert_eq!(buf.len(), 1);

        buf.push([2.0; 4]);
        buf.push([3.0; 4]);
        assert_eq!(buf.len(), 3);
    }

    /// A full window evicts its oldest entry to make room and keeps yielding
    /// what remains in oldest-to-newest order across the wrap point: after a
    /// fourth push into a buffer of three, the first observation is gone and
    /// the survivors iterate in the sequence they arrived. The physical write
    /// position wraps, but the observation order a reader sees does not.
    ///
    /// ´claim:wellness:a-full-window-evicts-its-oldest-entry-and-still-iterates-in-arrival-order-across-the-wrap´
    /// ´test:unit:circular-buffer-wraps-at-capacity´
    #[test]
    fn circular_buffer_wraps_at_capacity() {
        let mut buf = CircularBuffer::new(3);

        buf.push([1.0; 4]);
        buf.push([2.0; 4]);
        buf.push([3.0; 4]);
        assert_eq!(buf.len(), 3);

        // Push beyond capacity
        buf.push([4.0; 4]);
        assert_eq!(buf.len(), 3); // Still 3

        // Verify oldest (1.0) is evicted
        let values: Vec<_> = buf.iter().collect();
        assert_eq!(values.len(), 3);
        assert!((values[0][0] - 2.0).abs() < f64::EPSILON);
        assert!((values[1][0] - 3.0).abs() < f64::EPSILON);
        assert!((values[2][0] - 4.0).abs() < f64::EPSILON);
    }

    /// A window configured with no capacity accepts pushes and keeps nothing,
    /// rather than dividing by zero or indexing past its empty storage. A
    /// degenerate configuration should disable the statistic it feeds, not
    /// bring down the Assayer that merely observes.
    ///
    /// ´claim:wellness:a-zero-capacity-window-quietly-keeps-nothing-instead-of-failing´
    /// ´test:unit:circular-buffer-zero-capacity´
    #[test]
    fn circular_buffer_zero_capacity() {
        let mut buf = CircularBuffer::new(0);
        buf.push([1.0; 4]);
        assert_eq!(buf.len(), 0);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Percentile Tests
    // ─────────────────────────────────────────────────────────────────────────

    /// A percentile taken over no observations answers zero rather than
    /// panicking or returning a NaN. Recalibration reaches this case whenever
    /// an axis contributed nothing finite, and a NaN threshold would silently
    /// disable every comparison made against it thereafter.
    ///
    /// ´claim:wellness:a-percentile-over-no-observations-answers-zero-rather-than-panicking-or-returning-nan´
    /// ´test:unit:percentile-empty-slice´
    #[test]
    fn percentile_empty_slice() {
        assert!((percentile(&[], 50.0) - 0.0).abs() < f64::EPSILON);
    }

    /// A lone observation is every percentile of itself — the median, the
    /// bottom and the top all return it. With one sample there is no spread to
    /// interpolate across, and any answer other than the value itself would be
    /// invented rather than measured.
    ///
    /// ´claim:wellness:a-lone-observation-is-every-percentile-of-itself´
    /// ´test:unit:percentile-single-element´
    #[test]
    fn percentile_single_element() {
        assert!((percentile(&[5.0], 50.0) - 5.0).abs() < f64::EPSILON);
        assert!((percentile(&[5.0], 0.0) - 5.0).abs() < f64::EPSILON);
        assert!((percentile(&[5.0], 100.0) - 5.0).abs() < f64::EPSILON);
    }

    /// When the requested percentile lands exactly on a rank, the observation
    /// at that rank is returned unaltered: the midpoint of five evenly spaced
    /// values is the middle one. An exact hit must not be nudged by the
    /// interpolation machinery that serves the inexact cases.
    ///
    /// ´claim:wellness:a-percentile-landing-exactly-on-a-rank-returns-that-observation-unaltered´
    /// ´test:unit:percentile-median´
    #[test]
    fn percentile_median() {
        let sorted = [1.0, 2.0, 3.0, 4.0, 5.0];
        let p50 = percentile(&sorted, 50.0);
        assert!((p50 - 3.0).abs() < 1e-10);
    }

    /// A percentile falling between two ranks is interpolated between its
    /// neighbours rather than rounded to one of them: the eightieth percentile
    /// of ten evenly spaced values sits between the eighth and ninth, not on
    /// either. Interpolation keeps the calibrated threshold a smooth function
    /// of the window, so one arriving observation shifts it a little instead
    /// of stepping it.
    ///
    /// ´claim:wellness:a-percentile-between-ranks-is-interpolated-rather-than-rounded-to-a-neighbour´
    /// ´test:unit:percentile-80th´
    #[test]
    fn percentile_80th() {
        // For [0, 1, 2, 3, 4, 5, 6, 7, 8, 9], 80th percentile ≈ 7.2
        let sorted: Vec<f64> = (0..10).map(f64::from).collect();
        let p80 = percentile(&sorted, 80.0);
        assert!((p80 - 7.2).abs() < 0.1);
    }

    /// A percentile outside the sensible range is clamped to the ends of the
    /// sample rather than read as an index: a negative request returns the
    /// smallest observation and one above a hundred returns the largest. The
    /// percentile is configuration, so a mistyped one degrades to an extreme
    /// threshold instead of reaching outside the window.
    ///
    /// ´claim:wellness:a-percentile-outside-the-sensible-range-clamps-to-the-ends-of-the-sample´
    /// ´test:unit:percentile-clamped´
    #[test]
    fn percentile_clamped() {
        let sorted = [1.0, 2.0, 3.0];
        // Negative percentile clamped to 0
        assert!((percentile(&sorted, -10.0) - 1.0).abs() < f64::EPSILON);
        // Over 100 clamped to 100
        assert!((percentile(&sorted, 150.0) - 3.0).abs() < f64::EPSILON);
    }
}
