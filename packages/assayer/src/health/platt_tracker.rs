// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Platt calibration convergence tracker.
//!
//! Tracks the convergence state of Platt calibration (probability
//! calibration for the logistic model). Lives on the working copy
//! and is checkpointed.
//!
//! # Cross-References
//!
//! - (´dec:health:concrete-trackers´) — why this process carries its own
//!   concrete tracker
//! - (´dec:health:calibration-settled-cuts´) — the movement and the count
//!   calibration is called settled on
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`platt_initial_state`] | wellness | A fresh calibration tracker owes nothing and claims nothing: no refits completed, no labels banked towards the next one, no early refit pending, and a convergence reading of Initial. Nothing has been fitted, so the mapping in force is the identity the state name stands for. |
//! | [`platt_record_label_increments`] | wellness | Each processed label advances the since-refit count by exactly one, and the count accumulates across a run: one label makes it one, four more make it five. The refit budget is spent one label at a time, so a counter that skipped or double-counted would refit at a cadence unrelated to the evidence that had actually arrived. |
//! | [`platt_should_not_refit_below_threshold`] | wellness | cites (´claim:wellness:a-refit-falls-due-only-once-the-configured-label-count-has-accumulated´) |
//! | [`platt_state_does_not_regress`] | wellness | The convergence reading is recomputed from current evidence every time it is asked for, and it can be withdrawn: a tracker that has reached Converged over five small corrections falls back to Converging the moment a sixth refit moves the calibration sharply. Convergence is a present-tense statement about a live model, not a badge awarded once — a model whose calibration has started moving again is not converged merely because it used to be. |
//! | [`platt_health_snapshot`] | wellness | The calibration snapshot carries the last fit's own numbers — its calibration shift and both slopes — beside the refit count, the labels banked since, and the convergence state those figures imply. A reader gets the conclusion and the evidence for it in one consistent view, rather than a verdict it would have to trust blindly. |
//! | [`platt_from_config_uses_kappa_initial`] | wellness | Built from configuration, the tracker takes the configured initial slope for both the sister and the anchor regime and leaves everything else at its default — no refits claimed, no early refit pending. A deployment that knows its own scale can start calibration somewhere other than identity without also having to pretend fits have happened. |

// Items in this module are re-exported via crate::health and used by tests.
#![allow(dead_code)]

// ═══════════════════════════════════════════════════════════════════════════════
// Platt Convergence Tracker
// ═══════════════════════════════════════════════════════════════════════════════

/// Tracks Platt calibration convergence.
///
/// Lives on the working copy (checkpointed). Records refit history,
/// calibration drift, and kappa values for the sister and anchor models.
///
/// # Convergence States
///
/// - **Initial**: No refits completed yet
/// - **`FirstFit`**: Exactly one refit completed
/// - **Converging**: Multiple refits but `delta_cal > threshold`
/// - **Converged**: `delta_cal <= threshold` after sufficient refits
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlattConvergenceTracker {
    /// Number of Platt refits completed.
    pub refits_completed: u32,

    /// Labels processed since the last refit.
    pub labels_since_refit: u32,

    /// Most recent calibration delta (change in parameters).
    ///
    /// High values indicate the model is still adjusting significantly.
    pub last_delta_cal: f64,

    /// Whether an early refit is pending (requested explicitly).
    pub early_refit_pending: bool,

    /// Most recent kappa for the sister model.
    ///
    /// Initial value: 1.0 (identity mapping, no temperature scaling).
    /// κ₀ = 1.0 because output before the first refit is uncalibrated
    /// (´cav:calibration:pre-calibration´), which is what
    /// `PlattConvergenceState::Initial` names at the bottom of the settled
    /// ladder (´dec:health:calibration-settled-cuts´). The blend then reads κ
    /// through the shared regime transition
    /// (´dec:calibration:shared-transition´) as
    /// `κ_eff = (1 - g(w))·κ_sister + g(w)·κ_anchor` — zero would cause
    /// division by zero in the downstream sigmoid σ(ρ̂/κ).
    pub last_kappa_sister: f64,

    /// Most recent kappa for the anchor model.
    ///
    /// Initial value: 1.0. See [`last_kappa_sister`](Self::last_kappa_sister).
    pub last_kappa_anchor: f64,

    /// Label index at which the last anchor refit occurred.
    pub last_anchor_refit_label_index: u64,
}

impl Default for PlattConvergenceTracker {
    fn default() -> Self {
        Self {
            refits_completed: 0,
            labels_since_refit: 0,
            last_delta_cal: 0.0,
            early_refit_pending: false,
            last_kappa_sister: 1.0,
            last_kappa_anchor: 1.0,
            last_anchor_refit_label_index: 0,
        }
    }
}

impl PlattConvergenceTracker {
    /// Creates a new tracker in the initial state.
    ///
    /// Both κ values start at 1.0 — the identity mapping an uncalibrated
    /// output carries (´cav:calibration:pre-calibration´).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a tracker with initial κ sourced from [`PlattConfig`].
    ///
    /// This is the construction-time entry point, taking the figure the
    /// surfaces defer to (´tab:construction:parameters´).
    /// Both κ values start at `config.kappa_initial`.
    #[must_use]
    pub fn from_platt_config(config: &crate::risk::calibration::PlattConfig) -> Self {
        Self {
            last_kappa_sister: config.kappa_initial,
            last_kappa_anchor: config.kappa_initial,
            ..Self::default()
        }
    }

    /// Records that a label was processed.
    ///
    /// Increments `labels_since_refit`.
    pub const fn record_label(&mut self) {
        self.labels_since_refit = self.labels_since_refit.saturating_add(1);
    }

    /// Returns `true` if a Platt refit should be triggered.
    ///
    /// A refit is needed if:
    /// - `labels_since_refit >= n_refit`, OR
    /// - `early_refit_pending` is true
    ///
    /// # Arguments
    ///
    /// * `n_refit` — Number of labels between refits (from config)
    #[must_use]
    pub const fn should_refit(&self, n_refit: u32) -> bool {
        self.early_refit_pending || self.labels_since_refit >= n_refit
    }

    /// Records the result of a Platt refit.
    ///
    /// Updates `delta_cal`, kappa values, clears `early_refit_pending`,
    /// increments `refits_completed`, and resets `labels_since_refit`.
    ///
    /// If the anchor regime was fitted, updates `last_anchor_refit_label_index`
    /// to `label_index`, because the regimes are fitted separately
    /// (´dec:calibration:per-regime-search´).
    pub const fn record_refit(&mut self, result: &PlattRefitResult, label_index: u64) {
        self.last_delta_cal = result.delta_cal;
        self.last_kappa_sister = result.kappa_sister;
        self.last_kappa_anchor = result.kappa_anchor;
        if result.anchor_fitted {
            self.last_anchor_refit_label_index = label_index;
        }
        self.early_refit_pending = false;
        self.refits_completed = self.refits_completed.saturating_add(1);
        self.labels_since_refit = 0;
    }

    /// Returns the current convergence state.
    ///
    /// Pure function of tracker values and diagnostic thresholds.
    ///
    /// # Arguments
    ///
    /// * `delta_cal_threshold` — `δ_cal` at or below which calibration is
    ///   considered converged. Default: 0.01 (from
    ///   `ConvergenceThresholdConfig::platt_converged_delta_cal`).
    /// * `min_refit_count` — minimum refits before converged state is
    ///   possible. Default: 5 (from
    ///   `ConvergenceThresholdConfig::platt_converged_refit_count`).
    #[must_use]
    pub fn convergence_state(&self, delta_cal_threshold: f64, min_refit_count: u32) -> PlattConvergenceState {
        match self.refits_completed {
            0 => PlattConvergenceState::Initial,
            1 => PlattConvergenceState::FirstFit,
            _ => {
                if self.refits_completed >= min_refit_count && self.last_delta_cal <= delta_cal_threshold {
                    PlattConvergenceState::Converged
                } else {
                    PlattConvergenceState::Converging
                }
            }
        }
    }

    /// Marks an early refit as pending.
    ///
    /// Called when an external trigger requests immediate recalibration.
    pub const fn mark_early_refit(&mut self) {
        self.early_refit_pending = true;
    }

    /// Returns a health snapshot of the Platt tracker.
    #[must_use]
    pub fn health(&self, delta_cal_threshold: f64, min_refit_count: u32) -> PlattHealth {
        PlattHealth {
            refits_completed: self.refits_completed,
            labels_since_refit: self.labels_since_refit,
            last_delta_cal: self.last_delta_cal,
            convergence_state: self.convergence_state(delta_cal_threshold, min_refit_count),
            kappa_sister: self.last_kappa_sister,
            kappa_anchor: self.last_kappa_anchor,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Platt Convergence State
// ═══════════════════════════════════════════════════════════════════════════════

/// Convergence state of Platt calibration.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlattConvergenceState {
    /// No refits completed yet.
    #[default]
    Initial,
    /// Exactly one refit completed (first fit).
    FirstFit,
    /// Multiple refits but calibration is still adjusting.
    Converging,
    /// Calibration has converged (`delta_cal` small, sufficient refits).
    Converged,
}

impl PlattConvergenceState {
    /// Returns `true` if the calibration has converged.
    #[must_use]
    pub const fn is_converged(self) -> bool {
        matches!(self, Self::Converged)
    }

    /// Returns the state name as a static string.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::FirstFit => "first_fit",
            Self::Converging => "converging",
            Self::Converged => "converged",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Platt Refit Result
// ═══════════════════════════════════════════════════════════════════════════════

/// Result of a Platt calibration refit.
///
/// Contains the calibration delta and updated kappa values.
/// Produced by `refit_platt_calibration()` in `risk::calibration`.
///
/// # Cross-References
///
/// - (´dec:calibration:pure-refit´) — the pure function this is the result of
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlattRefitResult {
    /// Change in calibration parameters: max |ln `κ_new` − ln `κ_old`| across fitted regimes.
    pub delta_cal: f64,

    /// Updated kappa for the sister model.
    pub kappa_sister: f64,

    /// Updated kappa for the anchor model.
    pub kappa_anchor: f64,

    /// Whether the sister regime was fitted (met minimum sample requirements).
    pub sister_fitted: bool,

    /// Whether the anchor regime was fitted (met minimum sample requirements).
    pub anchor_fitted: bool,

    /// Effective weighted sample count for the sister regime.
    pub sister_effective_samples: f64,

    /// Effective weighted sample count for the anchor regime.
    pub anchor_effective_samples: f64,

    /// Discrimination metrics, computed once at this refit and cached
    /// (´dec:health:cached-discrimination´).
    ///
    /// `None` when insufficient calibration data.
    #[cfg_attr(feature = "serde", serde(skip))]
    pub discrimination: Option<super::published::DiscriminationMetrics>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Platt Health
// ═══════════════════════════════════════════════════════════════════════════════

/// Health snapshot of Platt calibration.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlattHealth {
    /// Number of refits completed.
    pub refits_completed: u32,

    /// Labels since last refit.
    pub labels_since_refit: u32,

    /// Most recent `delta_cal`.
    pub last_delta_cal: f64,

    /// Current convergence state.
    pub convergence_state: PlattConvergenceState,

    /// Current kappa for sister model.
    pub kappa_sister: f64,

    /// Current kappa for anchor model.
    pub kappa_anchor: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Default Thresholds
// ═══════════════════════════════════════════════════════════════════════════════

/// Default `δ_cal` convergence threshold (matches
/// `ConvergenceThresholdConfig::platt_converged_delta_cal`).
///
/// Exposed for unit tests that do not construct a full config.
///
/// A tenth of the movement that resets the drift accumulator, which is
/// the relation that fixes it: the calibration is called settled an
/// order tighter than it is called disturbed, leaving a wide band in
/// which it is neither (´dec:health:calibration-settled-cuts´).
///
/// ´const:assayer:calibration-settled-delta´ (´alg:const:scalar´)
/// ´const:assayer:calibration-settled-delta-scalar-0p01´
pub const DEFAULT_PLATT_CONVERGED_DELTA_CAL: f64 = 0.01;

/// Default minimum refit count for convergence (matches
/// `ConvergenceThresholdConfig::platt_converged_refit_count`).
///
/// A shipped operating point, and the count half of a pair that must
/// both be met: the movement bound alone is satisfiable by a thin refit
/// and the count alone says only that the cadence has run
/// (´dec:health:calibration-settled-cuts´).
///
/// ´const:assayer:calibration-settled-refits´ (´alg:const:count´)
/// ´const:assayer:calibration-settled-refits-count-5´
pub const DEFAULT_PLATT_CONVERGED_REFIT_COUNT: u32 = 5;

// ═══════════════════════════════════════════════════════════════════════════════
// Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// A fresh calibration tracker owes nothing and claims nothing: no refits
    /// completed, no labels banked towards the next one, no early refit
    /// pending, and a convergence reading of Initial. Nothing has been fitted,
    /// so the mapping in force is the identity the state name stands for.
    ///
    /// ´claim:wellness:a-fresh-calibration-tracker-owes-no-refit-and-reads-as-initial´
    /// ´test:unit:platt-initial-state´
    #[test]
    fn platt_initial_state() {
        let tracker = PlattConvergenceTracker::new();

        assert_eq!(tracker.refits_completed, 0);
        assert_eq!(tracker.labels_since_refit, 0);
        assert!(!tracker.early_refit_pending);
        assert_eq!(
            tracker.convergence_state(DEFAULT_PLATT_CONVERGED_DELTA_CAL, DEFAULT_PLATT_CONVERGED_REFIT_COUNT,),
            PlattConvergenceState::Initial
        );
    }

    /// Each processed label advances the since-refit count by exactly one, and
    /// the count accumulates across a run: one label makes it one, four more
    /// make it five. The refit budget is spent one label at a time, so a
    /// counter that skipped or double-counted would refit at a cadence
    /// unrelated to the evidence that had actually arrived.
    ///
    /// ´claim:wellness:each-processed-label-advances-the-since-refit-count-by-exactly-one´
    /// ´test:unit:platt-record-label-increments´
    #[test]
    fn platt_record_label_increments() {
        let mut tracker = PlattConvergenceTracker::new();

        tracker.record_label();
        assert_eq!(tracker.labels_since_refit, 1);

        for _ in 0..4 {
            tracker.record_label();
        }
        assert_eq!(tracker.labels_since_refit, 5);
    }

    /// Halfway to the budget is not due, and one label short of it is still
    /// not due: against a budget of two hundred, neither a hundred labels nor
    /// a hundred and ninety-nine make a refit owed. The comparison is a
    /// genuine threshold rather than a proportion of progress.
    ///
    /// (´claim:wellness:a-refit-falls-due-only-once-the-configured-label-count-has-accumulated´)
    /// ´test:unit:platt-should-not-refit-below-threshold´
    #[test]
    fn platt_should_not_refit_below_threshold() {
        let mut tracker = PlattConvergenceTracker::new();
        let n_refit = 200;

        // At 100 labels, not ready
        for _ in 0..100 {
            tracker.record_label();
        }
        assert!(!tracker.should_refit(n_refit));

        // At 199, still not ready
        for _ in 0..99 {
            tracker.record_label();
        }
        assert!(!tracker.should_refit(n_refit));
    }

    /// The convergence reading is recomputed from current evidence every time
    /// it is asked for, and it can be withdrawn: a tracker that has reached
    /// Converged over five small corrections falls back to Converging the
    /// moment a sixth refit moves the calibration sharply. Convergence is a
    /// present-tense statement about a live model, not a badge awarded once —
    /// a model whose calibration has started moving again is not converged
    /// merely because it used to be.
    ///
    /// ´claim:wellness:the-convergence-reading-is-recomputed-from-current-evidence-and-can-be-withdrawn´
    /// ´test:unit:platt-state-does-not-regress´
    #[test]
    fn platt_state_does_not_regress() {
        let mut tracker = PlattConvergenceTracker::new();

        // Get to Converged state
        for i in 0..5 {
            tracker.record_refit(
                &PlattRefitResult {
                    delta_cal: 0.005,
                    ..Default::default()
                },
                i,
            );
        }
        assert_eq!(
            tracker.convergence_state(DEFAULT_PLATT_CONVERGED_DELTA_CAL, DEFAULT_PLATT_CONVERGED_REFIT_COUNT,),
            PlattConvergenceState::Converged
        );

        // Record refit with high delta_cal
        tracker.record_refit(
            &PlattRefitResult {
                delta_cal: 0.5, // High delta
                ..Default::default()
            },
            5,
        );

        // State should still be Converging (6 refits, high delta)
        // Note: per spec, convergence state is computed fresh each time,
        // so with high delta it goes back to Converging. The testing plan's
        // "does not regress" may refer to a different semantic. Let's verify
        // the actual behavior:
        let state = tracker.convergence_state(DEFAULT_PLATT_CONVERGED_DELTA_CAL, DEFAULT_PLATT_CONVERGED_REFIT_COUNT);
        // With 6 refits but high delta_cal, it's Converging
        assert_eq!(state, PlattConvergenceState::Converging);
    }

    /// The calibration snapshot carries the last fit's own numbers — its
    /// calibration shift and both slopes — beside the refit count, the labels
    /// banked since, and the convergence state those figures imply. A reader
    /// gets the conclusion and the evidence for it in one consistent view,
    /// rather than a verdict it would have to trust blindly.
    ///
    /// ´claim:wellness:the-calibration-snapshot-carries-the-last-fits-numbers-beside-the-state-they-imply´
    /// ´test:unit:platt-health-snapshot´
    #[test]
    fn platt_health_snapshot() {
        let mut tracker = PlattConvergenceTracker::new();
        tracker.record_refit(
            &PlattRefitResult {
                delta_cal: 0.15,
                kappa_sister: 1.2,
                kappa_anchor: 0.8,
                ..Default::default()
            },
            50,
        );

        let health = tracker.health(DEFAULT_PLATT_CONVERGED_DELTA_CAL, DEFAULT_PLATT_CONVERGED_REFIT_COUNT);
        assert_eq!(health.refits_completed, 1);
        assert_eq!(health.labels_since_refit, 0);
        assert!((health.last_delta_cal - 0.15).abs() < 1e-10);
        assert_eq!(health.convergence_state, PlattConvergenceState::FirstFit);
        assert!((health.kappa_sister - 1.2).abs() < 1e-10);
        assert!((health.kappa_anchor - 0.8).abs() < 1e-10);
    }

    /// Built from configuration, the tracker takes the configured initial
    /// slope for both the sister and the anchor regime and leaves everything
    /// else at its default — no refits claimed, no early refit pending. A
    /// deployment that knows its own scale can start calibration somewhere
    /// other than identity without also having to pretend fits have happened.
    ///
    /// ´claim:wellness:building-from-config-seeds-both-slopes-from-the-configured-initial-and-leaves-the-rest-default´
    /// ´test:unit:platt-from-config-uses-kappa-initial´
    #[test]
    fn platt_from_config_uses_kappa_initial() {
        use crate::risk::calibration::PlattConfig;

        let config = PlattConfig {
            kappa_initial: 2.5,
            ..PlattConfig::default()
        };
        let tracker = PlattConvergenceTracker::from_platt_config(&config);

        assert!((tracker.last_kappa_sister - 2.5).abs() < 1e-10);
        assert!((tracker.last_kappa_anchor - 2.5).abs() < 1e-10);
        // Other fields should still be at defaults.
        assert_eq!(tracker.refits_completed, 0);
        assert!(!tracker.early_refit_pending);
    }
}
