// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Identity dimension convergence tracker.
//!
//! Tracks the stability of competitive sets within an identity dimension.
//! Each dimension has its own tracker. Lives on the working copy and is
//! checkpointed.
//!
//! # Cross-References
//!
//! - (´dec:health:concrete-trackers´) — why this process carries its own
//!   concrete tracker
//! - (´dec:health:identity-stability-cuts´) — the two coordinates a dimension's
//!   stability is read on
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`identity_initial_state`] | wellness | A tracker that has seen nothing starts optimistic on overlap and quiet on churn — perfect similarity, zero change rate, no cells, no entries or exits — and reports Initial rather than any settled stage. The counters are what distinguish "no churn observed" from "no observation yet", so the stage is decided by them rather than by the flattering starting metrics. |
//! | [`identity_stage_forming_few_cells`] | wellness | A dimension whose first cells have only just arrived reads as unsettled — Volatile or Stabilising — and never as mature or stable. The very event that populates a competitive set is a total change to it, so the moment of formation is the least stable moment a dimension has. |
//! | [`identity_stage_stabilising`] | wellness | A run of observations reporting the same set carries a dimension out of volatility: after ten unchanged observations following its formation, the stage has climbed to Stabilising or beyond. Stability is earned by repetition, and the smoothing is slack enough that a single quiet observation does not buy it while ten do. |
//! | [`identity_stage_converged_stable`] | wellness | Stable is reached when both conditions hold at once: the set-level metrics are settled — high overlap, low churn, changes actually recorded — and enough time has passed since the dimension was first seen. The metrics describe the competitive set, the elapsed time stands in for the labels the cell-indicator weights needed, and only together do they mean the dimension can be relied upon. |
//! | [`identity_stage_maturing`] | wellness | cites (´claim:wellness:calm-metrics-on-a-newly-registered-identity-set-do-not-count-as-maturity´) |
//! | [`identity_maturity_gate_follows_the_eligible_label_rate`] | wellness | The maturity gate clears the arrival time the convergence budget gives when the budget is divided the way the specification divides it — by the rate at which *eligible* labels arrive, not by the raw label rate. The two divisions differ by the eligibility fraction alone, and the raw one lands the gate below the arrival it stands in for, which would report a dimension settled while the cell-indicator weights behind it were still short of their labels. The gate is the whole day above the eligible arrival rather than an arbitrary pad, so this pins the rounding as well as the rate. |
//! | [`maturity_gate_covers_the_smaller_two_tabulated_deployments`] | wellness | One constant stands against an arrival that is a property of the deployment, so the gate covers some of the specification's tabulated sizes and not others, and which ones is a fact worth pinning rather than restating. It covers the minimal and the standard deployments. It does not cover the standard-plus, and the margin there is the part prose gets wrong: eight and four fifths hours, under a day, which is small enough to read as covered and is not. It does not cover the large, by a little over four days. The gate itself is unchanged by this — the shortfall costs nothing because the stage is a reading and gates no path — and what this test refuses is a record that says the eleven days reach further than they do. |
//! | [`compute_jaccard_empty_sets`] | wellness | Two empty sets count as perfectly overlapping rather than as an undefined ratio. A dimension with no competitive cells has not changed when it still has none, and answering with a division by zero — or with zero similarity — would report churn where there was none to report. |
//! | [`compute_jaccard_identical_sets`] | wellness | A set compared with itself overlaps perfectly. This is the reading a quiescent dimension produces on every observation, so it has to be exactly one — anything short of it would decay the smoothed overlap of a set that never changes and eventually mark it volatile. |
//! | [`compute_jaccard_disjoint_sets`] | wellness | Sets sharing no member at all overlap not at all — the measure bottoms out at zero, not at some floor. Wholesale replacement of a competitive set is the strongest churn signal there is, and it has to register at full strength for the volatility thresholds to mean anything. |
//! | [`compute_jaccard_half_overlap`] | wellness | Partial overlap is measured against the union of both sets rather than against either one alone: two sets of two sharing a single member score a third, not a half. Dividing by the union means a set that grows while keeping its old members is scored as having changed, which is exactly what churn in a competitive set means. |
//! | [`identity_jaccard_half_changed`] | wellness | Smoothing lets the tracker register a large change without lurching to it: swapping half of a ten-cell set produces a raw overlap of a third, and the smoothed measure moves down from perfect while staying well above that raw reading. One disruptive observation should tilt the judgement, not decide it — otherwise a single reorganisation would declare a long-settled dimension volatile. |
//! | [`identity_change_rate_decays`] | wellness | Churn is forgiven with quiet: after an initial burst that populates the set, twenty observations reporting no change leave the smoothed change rate below where the burst left it. Past turbulence must be able to age out, or a dimension that had one bad day could never afterwards be judged stable. |

// Items in this module are re-exported via crate::health and used by tests.

use std::collections::HashSet;

use crate::identity::CompetitiveCellId;
use crate::types::PersistentTimestamp;

// ═══════════════════════════════════════════════════════════════════════════════
// Identity Convergence Tracker
// ═══════════════════════════════════════════════════════════════════════════════

/// Tracks convergence of a single identity dimension.
///
/// Monitors the stability of the competitive cell set via:
/// - Change rate EWMA (entries + exits per update)
/// - Jaccard similarity EWMA (overlap with previous set)
///
/// Lives on the working copy (checkpointed).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentityConvergenceTracker {
    /// EWMA of change rate (entries + exits normalized).
    ///
    /// High values indicate rapid competitive set changes.
    pub change_rate_ewma: f64,

    /// EWMA of Jaccard similarity with previous set.
    ///
    /// High values indicate stable competitive sets.
    pub jaccard_ewma: f64,

    /// Current number of cells in the competitive set.
    pub current_cell_count: usize,

    /// Cell IDs from the previous update (for Jaccard computation).
    pub previous_cell_ids: HashSet<CompetitiveCellId>,

    /// Total entries since tracking began.
    pub total_entries: u64,

    /// Total exits since tracking began.
    pub total_exits: u64,

    /// Timestamp when tracking began.
    pub first_seen: PersistentTimestamp,
}

impl IdentityConvergenceTracker {
    /// Creates a new tracker in the initial state, first seen at the
    /// given moment of the injected clock.
    // TODO ´todo:code:start-pessimistic-and-count-change-events´: start pessimistic and count change events —
    // `change_rate_ewma` at 1.0, `jaccard_ewma` at 0.0, and a
    // `total_change_events` counter so quiescence ticks can move a
    // quiet dimension toward Stable on the cuts it is read against
    // (´dec:health:identity-stability-cuts´).
    #[must_use]
    pub fn new(first_seen: PersistentTimestamp) -> Self {
        Self {
            change_rate_ewma: 0.0,
            jaccard_ewma: 1.0, // Assume stable initially
            current_cell_count: 0,
            previous_cell_ids: HashSet::new(),
            total_entries: 0,
            total_exits: 0,
            first_seen,
        }
    }

    /// Records a change event (entries and exits).
    ///
    /// Updates the EWMA values and counters.
    ///
    /// # Arguments
    ///
    /// * `entries` — Cell IDs that entered the competitive set
    /// * `exits` — Cell IDs that exited the competitive set
    /// * `current_set` — The current complete competitive set
    pub fn record_change_event(
        &mut self,
        entries: &[CompetitiveCellId],
        exits: &[CompetitiveCellId],
        current_set: &[CompetitiveCellId],
    ) {
        // TODO ´todo:code:increment-total-change-events-here-and´: increment `total_change_events` here and add a
        // `tick(current_set)` call path for zero-change quiescence updates.
        // Update counters
        self.total_entries = self.total_entries.saturating_add(entries.len() as u64);
        self.total_exits = self.total_exits.saturating_add(exits.len() as u64);

        // Compute change rate (normalized by current + previous size)
        let n_changes = entries.len() + exits.len();
        let denominator = current_set.len().saturating_add(self.previous_cell_ids.len()).max(1);

        #[allow(clippy::cast_precision_loss)]
        let change_rate = n_changes as f64 / denominator as f64;

        // Compute Jaccard similarity
        let current_ids: HashSet<CompetitiveCellId> = current_set.iter().copied().collect();
        let jaccard = compute_jaccard(&self.previous_cell_ids, &current_ids);

        // Update EWMAs
        self.change_rate_ewma = (1.0 - EWMA_ALPHA).mul_add(self.change_rate_ewma, EWMA_ALPHA * change_rate);
        self.jaccard_ewma = (1.0 - EWMA_ALPHA).mul_add(self.jaccard_ewma, EWMA_ALPHA * jaccard);

        // Update state
        self.current_cell_count = current_set.len();
        self.previous_cell_ids = current_ids;
    }

    /// Returns the diagnostic convergence stage at the given moment,
    /// a pure function of the tracker's state and the injected present.
    ///
    /// Stages are diagnostic only — the Assayer operates identically
    /// regardless of stage.
    #[must_use]
    pub fn convergence_stage(&self, now: &PersistentTimestamp) -> IdentityConvergenceStage {
        // Very early: no changes recorded yet
        if self.total_entries == 0 && self.total_exits == 0 {
            return IdentityConvergenceStage::Initial;
        }

        // Volatile: low Jaccard or high change rate
        if self.jaccard_ewma < VOLATILE_JACCARD_THRESHOLD || self.change_rate_ewma > VOLATILE_CHANGE_RATE_THRESHOLD {
            return IdentityConvergenceStage::Volatile;
        }

        // Not yet stable enough on set-level metrics
        if self.jaccard_ewma < STABLE_JACCARD_THRESHOLD || self.change_rate_ewma > STABLE_CHANGE_RATE_THRESHOLD {
            return IdentityConvergenceStage::Stabilising;
        }

        // Set metrics are stable — check temporal maturity.
        // Even with a stable competitive set, model weights for the
        // cell indicators need label accumulation to converge. Use
        // hours since registration as a proxy
        // (´cav:health:identity-maturity-arithmetic´), gated at
        // the eligible-label arrival time the budget gives
        // (´bound:resource:convergence-budget´).
        let hours_since_first = now.hours_since(&self.first_seen);
        if hours_since_first < MATURITY_HOURS_THRESHOLD {
            return IdentityConvergenceStage::Maturing;
        }

        IdentityConvergenceStage::Stable
    }

    /// Returns a health snapshot of the identity tracker at the given moment.
    #[must_use]
    pub fn health(&self, now: &PersistentTimestamp) -> IdentityConvergenceHealth {
        IdentityConvergenceHealth {
            change_rate_ewma: self.change_rate_ewma,
            jaccard_ewma: self.jaccard_ewma,
            current_cell_count: self.current_cell_count,
            total_entries: self.total_entries,
            total_exits: self.total_exits,
            convergence_stage: self.convergence_stage(now),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Identity Convergence Stage
// ═══════════════════════════════════════════════════════════════════════════════

/// Diagnostic convergence stage for an identity dimension.
///
/// These stages are purely diagnostic — they do not affect Assayer
/// operation, because convergence is reported and never enforced
/// (´dec:health:reports-never-gates´).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IdentityConvergenceStage {
    /// No changes recorded yet.
    #[default]
    Initial,
    /// High change rate or low Jaccard similarity.
    Volatile,
    /// Between volatile and stable thresholds.
    Stabilising,
    /// Set metrics stable, but insufficient time for model weights
    /// to converge. The competitive set is spatially settled, yet
    /// the Bayesian cell-indicator weights are still prior-dominated.
    Maturing,
    /// Low change rate, high Jaccard similarity, and sufficient
    /// maturity time elapsed.
    Stable,
}

impl IdentityConvergenceStage {
    /// Returns `true` if the dimension is stable.
    #[must_use]
    pub const fn is_stable(self) -> bool {
        matches!(self, Self::Stable)
    }

    /// Returns the stage name as a static string.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::Volatile => "volatile",
            Self::Stabilising => "stabilising",
            Self::Maturing => "maturing",
            Self::Stable => "stable",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Identity Convergence Health
// ═══════════════════════════════════════════════════════════════════════════════

/// Health snapshot for an identity dimension.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentityConvergenceHealth {
    /// Current change rate EWMA.
    pub change_rate_ewma: f64,

    /// Current Jaccard similarity EWMA.
    pub jaccard_ewma: f64,

    /// Current number of cells in competitive set.
    pub current_cell_count: usize,

    /// Total entries since tracking began.
    pub total_entries: u64,

    /// Total exits since tracking began.
    pub total_exits: u64,

    /// Diagnostic convergence stage.
    pub convergence_stage: IdentityConvergenceStage,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Computes the Jaccard similarity between two sets.
///
/// Returns 1.0 if both sets are empty.
fn compute_jaccard(a: &HashSet<CompetitiveCellId>, b: &HashSet<CompetitiveCellId>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }

    let intersection = a.intersection(b).count();
    let union = a.union(b).count();

    if union == 0 {
        return 1.0;
    }

    #[allow(clippy::cast_precision_loss)]
    {
        intersection as f64 / union as f64
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Configuration Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// EWMA smoothing factor (α).
///
/// Higher values give more weight to recent observations. At this
/// weight the mean age of the evidence behind a reading is nine change
/// events, so the tracker reports roughly what the last ten did — long
/// enough that a cascade of splits is not read as volatility, short
/// enough to notice a dimension that has genuinely moved
/// (´tab:health:identity-stability-cuts´).
///
/// ´const:assayer:churn-smoothing-weight´ (´alg:const:scalar´)
/// ´const:assayer:churn-smoothing-weight-scalar-0p1´
const EWMA_ALPHA: f64 = 0.1;

/// Jaccard threshold for stable classification.
///
/// Derived from the stable churn ceiling rather than chosen: it is the
/// overlap a constant-size replacement at that ceiling produces, taken
/// to one decimal (´tab:health:identity-stability-cuts´).
///
/// ´const:assayer:stable-overlap-threshold´ (´alg:const:scalar´)
/// ´const:assayer:stable-overlap-threshold-scalar-0p9´
const STABLE_JACCARD_THRESHOLD: f64 = 0.9;

/// Change rate threshold for stable classification.
///
/// One event in twenty, and six times below the volatile floor — the
/// factor being the width of the band a dimension crosses to change
/// its stage, wide enough that one sitting near a cut does not
/// oscillate (´tab:health:identity-stability-cuts´).
///
/// ´const:assayer:stable-churn-ceiling´ (´alg:const:scalar´)
/// ´const:assayer:stable-churn-ceiling-scalar-0p05´
const STABLE_CHANGE_RATE_THRESHOLD: f64 = 0.05;

/// Jaccard threshold below which is considered volatile.
///
/// Derived from the volatile churn floor on the same footing as the
/// stable overlap above (´tab:health:identity-stability-cuts´).
///
/// ´const:assayer:volatile-overlap-threshold´ (´alg:const:scalar´)
/// ´const:assayer:volatile-overlap-threshold-scalar-0p5´
const VOLATILE_JACCARD_THRESHOLD: f64 = 0.5;

/// Change rate threshold above which is considered volatile.
///
/// The shipped operating point the rest of the ladder derives from:
/// roughly a third of the set turning over per change event is movement
/// no reader should call settling
/// (´tab:health:identity-stability-cuts´).
///
/// ´const:assayer:volatile-churn-floor´ (´alg:const:scalar´)
/// ´const:assayer:volatile-churn-floor-scalar-0p3´
const VOLATILE_CHANGE_RATE_THRESHOLD: f64 = 0.3;

/// Minimum hours since registration before the dimension can reach
/// `Stable`. Even with a spatially-settled competitive set, the
/// Bayesian model weights for cell indicators need label accumulation
/// to converge, and this age gate stands in for a label count the
/// tracker does not see (´cav:health:identity-maturity-arithmetic´).
///
/// Eleven days, taken as the round figure above the arrival time the
/// specification's convergence budget gives: twice the standard
/// deployment's parameter count is one thousand two hundred and
/// seventy-six eligible labels, and the budget is divided by the rate
/// at which *eligible* labels arrive — a hundred and twenty a day at
/// the reference rates — for ten and two thirds days
/// (´bound:resource:convergence-budget´),
/// (´tab:health:identity-stability-cuts´).
///
/// ´const:assayer:dimension-maturity-age´ (´alg:const:scalar´)
/// ´const:assayer:dimension-maturity-age-scalar-264p0´
const MATURITY_HOURS_THRESHOLD: f64 = 264.0;

// ═══════════════════════════════════════════════════════════════════════════════
// Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn make_cell(lo: u128, depth: u8) -> CompetitiveCellId {
        CompetitiveCellId::new(lo, depth)
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Tracker Tests
    // ─────────────────────────────────────────────────────────────────────────

    /// A tracker that has seen nothing starts optimistic on overlap and quiet
    /// on churn — perfect similarity, zero change rate, no cells, no entries
    /// or exits — and reports Initial rather than any settled stage. The
    /// counters are what distinguish "no churn observed" from "no observation
    /// yet", so the stage is decided by them rather than by the flattering
    /// starting metrics.
    ///
    /// ´claim:wellness:a-fresh-identity-tracker-reports-initial-rather-than-letting-its-starting-metrics-speak´
    /// ´test:unit:identity-initial-state´
    #[test]
    fn identity_initial_state() {
        let now = PersistentTimestamp::new(0, 0);
        let tracker = IdentityConvergenceTracker::new(now);

        assert!((tracker.change_rate_ewma - 0.0).abs() < 1e-10);
        assert!((tracker.jaccard_ewma - 1.0).abs() < 1e-10);
        assert_eq!(tracker.current_cell_count, 0);
        assert!(tracker.previous_cell_ids.is_empty());
        assert_eq!(tracker.total_entries, 0);
        assert_eq!(tracker.total_exits, 0);
        assert_eq!(tracker.convergence_stage(&now), IdentityConvergenceStage::Initial);
    }

    /// A dimension whose first cells have only just arrived reads as unsettled
    /// — Volatile or Stabilising — and never as mature or stable. The very
    /// event that populates a competitive set is a total change to it, so the
    /// moment of formation is the least stable moment a dimension has.
    ///
    /// ´claim:wellness:a-dimension-whose-cells-have-only-just-arrived-reads-as-unsettled´
    /// ´test:unit:identity-stage-forming-few-cells´
    #[test]
    fn identity_stage_forming_few_cells() {
        let now = PersistentTimestamp::new(0, 0);
        let mut tracker = IdentityConvergenceTracker::new(now);

        // Add a few cells
        let cells = vec![make_cell(0, 4), make_cell(100, 4)];
        tracker.record_change_event(&cells, &[], &cells);

        // With only 2 cells and recent changes, likely Volatile or Stabilising
        let stage = tracker.convergence_stage(&now);
        assert!(
            matches!(
                stage,
                IdentityConvergenceStage::Volatile | IdentityConvergenceStage::Stabilising
            ),
            "Expected Volatile or Stabilising, got {stage:?}"
        );
    }

    /// A run of observations reporting the same set carries a dimension out of
    /// volatility: after ten unchanged observations following its formation,
    /// the stage has climbed to Stabilising or beyond. Stability is earned by
    /// repetition, and the smoothing is slack enough that a single quiet
    /// observation does not buy it while ten do.
    ///
    /// ´claim:wellness:repeated-unchanged-observations-carry-a-dimension-out-of-volatility´
    /// ´test:unit:identity-stage-stabilising´
    #[test]
    fn identity_stage_stabilising() {
        let now = PersistentTimestamp::new(0, 0);
        let mut tracker = IdentityConvergenceTracker::new(now);

        let cells: Vec<_> = (0..10).map(|i| make_cell(i * 100, 4)).collect();

        // First observation
        tracker.record_change_event(&cells, &[], &cells);

        // Several stable observations (same set)
        for _ in 0..10 {
            tracker.record_change_event(&[], &[], &cells);
        }

        // Should be stabilising, maturing, or stable at this point
        let stage = tracker.convergence_stage(&now);
        assert!(
            matches!(
                stage,
                IdentityConvergenceStage::Stabilising | IdentityConvergenceStage::Maturing | IdentityConvergenceStage::Stable
            ),
            "Expected Stabilising, Maturing, or Stable, got {stage:?}"
        );
    }

    /// Stable is reached when both conditions hold at once: the set-level
    /// metrics are settled — high overlap, low churn, changes actually
    /// recorded — and enough time has passed since the dimension was first
    /// seen. The metrics describe the competitive set, the elapsed time stands
    /// in for the labels the cell-indicator weights needed, and only together
    /// do they mean the dimension can be relied upon.
    ///
    /// ´claim:wellness:stable-requires-settled-set-metrics-and-elapsed-existence-together´
    /// ´test:unit:identity-stage-converged-stable´
    #[test]
    fn identity_stage_converged_stable() {
        let mut tracker = IdentityConvergenceTracker::new(PersistentTimestamp::new(0, 0));

        // Force stable values directly
        tracker.jaccard_ewma = 0.95;
        tracker.change_rate_ewma = 0.01;
        tracker.total_entries = 10;

        // Read far enough after first_seen to pass the maturity gate.
        let now = PersistentTimestamp::new(400 * 3600, 0);
        assert_eq!(tracker.convergence_stage(&now), IdentityConvergenceStage::Stable);
    }

    /// Set metrics that would otherwise qualify as stable read as Maturing
    /// while the dimension is newly registered: the maturity gate is what the
    /// stage falls back to when everything except elapsed time is satisfied.
    /// It is a distinct stage rather than a demotion to Stabilising, so an
    /// operator can see that the only thing outstanding is time.
    ///
    /// (´claim:wellness:calm-metrics-on-a-newly-registered-identity-set-do-not-count-as-maturity´)
    /// ´test:unit:identity-stage-maturing´
    #[test]
    fn identity_stage_maturing() {
        let now = PersistentTimestamp::new(0, 0);
        let mut tracker = IdentityConvergenceTracker::new(now);

        // Stable set-level metrics but registered just now
        tracker.jaccard_ewma = 0.95;
        tracker.change_rate_ewma = 0.01;
        tracker.total_entries = 10;
        // first_seen is the reading instant — maturity gate should fire

        assert_eq!(tracker.convergence_stage(&now), IdentityConvergenceStage::Maturing);
    }

    /// The maturity gate clears the arrival time the convergence budget
    /// gives when the budget is divided the way the specification divides
    /// it — by the rate at which *eligible* labels arrive, not by the raw
    /// label rate. The two divisions differ by the eligibility fraction
    /// alone, and the raw one lands the gate below the arrival it stands in
    /// for, which would report a dimension settled while the cell-indicator
    /// weights behind it were still short of their labels. The gate is the
    /// whole day above the eligible arrival rather than an arbitrary pad, so
    /// this pins the rounding as well as the rate.
    ///
    /// ´claim:wellness:the-identity-maturity-gate-clears-the-eligible-label-arrival-and-not-the-raw-rate´
    /// ´test:unit:identity-maturity-gate-follows-the-eligible-label-rate´
    #[test]
    fn identity_maturity_gate_follows_the_eligible_label_rate() {
        // The specification's own figures for the standard deployment
        // (´bound:resource:convergence-budget´).
        let parameters = 638.0_f64;
        let labels_per_day = 200.0_f64;
        let eligible_fraction = 0.6_f64;
        let hours_per_day = 24.0_f64;

        let budget = 2.0 * parameters;
        let eligible_arrival_hours = budget / (labels_per_day * eligible_fraction) * hours_per_day;

        assert!(
            MATURITY_HOURS_THRESHOLD >= eligible_arrival_hours,
            "the gate must cover the eligible-label arrival it stands in for: \
             gate {MATURITY_HOURS_THRESHOLD} h against arrival {eligible_arrival_hours} h",
        );
        assert!(
            MATURITY_HOURS_THRESHOLD - eligible_arrival_hours < hours_per_day,
            "the gate is the whole day above the arrival, not a pad: \
             gate {MATURITY_HOURS_THRESHOLD} h against arrival {eligible_arrival_hours} h",
        );

        // Dividing by the raw label rate is the step the gate must not take.
        let raw_rate_hours = budget / labels_per_day * hours_per_day;
        assert!(
            MATURITY_HOURS_THRESHOLD > raw_rate_hours,
            "the gate must not sit at the raw-rate figure {raw_rate_hours} h",
        );

        // And the ladder reads the gate the way the arithmetic says it does.
        let mut tracker = IdentityConvergenceTracker::new(PersistentTimestamp::new(0, 0));
        tracker.jaccard_ewma = 0.95;
        tracker.change_rate_ewma = 0.01;
        tracker.total_entries = 10;

        #[allow(clippy::cast_possible_truncation)]
        let raw_rate_secs = (raw_rate_hours * 3600.0) as i64;
        assert_eq!(
            tracker.convergence_stage(&PersistentTimestamp::new(raw_rate_secs, 0)),
            IdentityConvergenceStage::Maturing,
            "a dimension at the raw-rate age has not yet reached the gate",
        );

        #[allow(clippy::cast_possible_truncation)]
        let gate_secs = (MATURITY_HOURS_THRESHOLD * 3600.0) as i64;
        assert_eq!(
            tracker.convergence_stage(&PersistentTimestamp::new(gate_secs, 0)),
            IdentityConvergenceStage::Stable,
            "a dimension at the gate has reached it",
        );
    }

    /// One constant stands against an arrival that is a property of the
    /// deployment, so the gate covers some of the specification's tabulated
    /// sizes and not others, and which ones is a fact worth pinning rather
    /// than restating. It covers the minimal and the standard deployments.
    /// It does not cover the standard-plus, and the margin there is the part
    /// prose gets wrong: eight and four fifths hours, under a day, which is
    /// small enough to read as covered and is not. It does not cover the
    /// large, by a little over four days. The gate itself is unchanged by
    /// this — the shortfall costs nothing because the stage is a reading and
    /// gates no path — and what this test refuses is a record that says the
    /// eleven days reach further than they do.
    ///
    /// ´claim:wellness:the-maturity-gate-covers-the-minimal-and-standard-deployments-and-falls-short-of-the-larger-two´
    /// ´test:unit:maturity-gate-covers-the-smaller-two-tabulated-deployments´
    #[test]
    fn maturity_gate_covers_the_smaller_two_tabulated_deployments() {
        // The deployment table of (´bound:resource:convergence-budget´),
        // as parameter counts; the eligible rate is the same throughout.
        let eligible_per_day = 200.0_f64 * 0.6_f64;
        let hours_per_day = 24.0_f64;
        let arrival_hours = |parameters: f64| 2.0 * parameters / eligible_per_day * hours_per_day;

        let minimal = arrival_hours(286.0);
        let standard = arrival_hours(638.0);
        let standard_plus = arrival_hours(682.0);
        let large = arrival_hours(910.0);

        assert!(
            MATURITY_HOURS_THRESHOLD >= minimal && MATURITY_HOURS_THRESHOLD >= standard,
            "the gate covers the minimal ({minimal} h) and the standard ({standard} h)",
        );

        let standard_plus_shortfall = standard_plus - MATURITY_HOURS_THRESHOLD;
        assert!(
            standard_plus_shortfall > 8.0 && standard_plus_shortfall < 9.0,
            "the standard-plus falls short by about nine hours, not none: {standard_plus_shortfall} h",
        );

        let large_shortfall = large - MATURITY_HOURS_THRESHOLD;
        assert!(
            large_shortfall > 4.0 * hours_per_day && large_shortfall < 5.0 * hours_per_day,
            "the large falls short by about four days: {large_shortfall} h",
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Jaccard Tests
    // ─────────────────────────────────────────────────────────────────────────

    /// Two empty sets count as perfectly overlapping rather than as an
    /// undefined ratio. A dimension with no competitive cells has not changed
    /// when it still has none, and answering with a division by zero — or with
    /// zero similarity — would report churn where there was none to report.
    ///
    /// ´claim:wellness:two-empty-sets-count-as-perfectly-overlapping-rather-than-undefined´
    /// ´test:unit:compute-jaccard-empty-sets´
    #[test]
    fn compute_jaccard_empty_sets() {
        let a: HashSet<CompetitiveCellId> = HashSet::new();
        let b: HashSet<CompetitiveCellId> = HashSet::new();
        assert!((compute_jaccard(&a, &b) - 1.0).abs() < 1e-10);
    }

    /// A set compared with itself overlaps perfectly. This is the reading a
    /// quiescent dimension produces on every observation, so it has to be
    /// exactly one — anything short of it would decay the smoothed overlap of
    /// a set that never changes and eventually mark it volatile.
    ///
    /// ´claim:wellness:a-set-compared-with-itself-overlaps-perfectly´
    /// ´test:unit:compute-jaccard-identical-sets´
    #[test]
    fn compute_jaccard_identical_sets() {
        let set: HashSet<_> = vec![make_cell(0, 4), make_cell(100, 4)].into_iter().collect();
        assert!((compute_jaccard(&set, &set) - 1.0).abs() < 1e-10);
    }

    /// Sets sharing no member at all overlap not at all — the measure bottoms
    /// out at zero, not at some floor. Wholesale replacement of a competitive
    /// set is the strongest churn signal there is, and it has to register at
    /// full strength for the volatility thresholds to mean anything.
    ///
    /// ´claim:wellness:sets-sharing-no-member-overlap-not-at-all´
    /// ´test:unit:compute-jaccard-disjoint-sets´
    #[test]
    fn compute_jaccard_disjoint_sets() {
        let a: HashSet<_> = vec![make_cell(0, 4), make_cell(100, 4)].into_iter().collect();
        let b: HashSet<_> = vec![make_cell(200, 4), make_cell(300, 4)].into_iter().collect();
        assert!((compute_jaccard(&a, &b) - 0.0).abs() < 1e-10);
    }

    /// Partial overlap is measured against the union of both sets rather than
    /// against either one alone: two sets of two sharing a single member score
    /// a third, not a half. Dividing by the union means a set that grows while
    /// keeping its old members is scored as having changed, which is exactly
    /// what churn in a competitive set means.
    ///
    /// ´claim:wellness:partial-overlap-is-measured-against-the-union-not-against-either-set-alone´
    /// ´test:unit:compute-jaccard-half-overlap´
    #[test]
    fn compute_jaccard_half_overlap() {
        // A = {0, 100}, B = {100, 200}
        // Intersection = {100}, Union = {0, 100, 200}
        // Jaccard = 1/3 ≈ 0.333
        let a: HashSet<_> = vec![make_cell(0, 4), make_cell(100, 4)].into_iter().collect();
        let b: HashSet<_> = vec![make_cell(100, 4), make_cell(200, 4)].into_iter().collect();
        let j = compute_jaccard(&a, &b);
        assert!((j - 1.0 / 3.0).abs() < 1e-10, "Expected 0.333, got {j}");
    }

    /// Smoothing lets the tracker register a large change without lurching to
    /// it: swapping half of a ten-cell set produces a raw overlap of a third,
    /// and the smoothed measure moves down from perfect while staying well
    /// above that raw reading. One disruptive observation should tilt the
    /// judgement, not decide it — otherwise a single reorganisation would
    /// declare a long-settled dimension volatile.
    ///
    /// ´claim:wellness:smoothing-lets-a-large-change-tilt-the-overlap-measure-without-the-measure-lurching-to-it´
    /// ´test:unit:identity-jaccard-half-changed´
    #[test]
    fn identity_jaccard_half_changed() {
        let mut tracker = IdentityConvergenceTracker::new(PersistentTimestamp::new(0, 0));

        // Initial: 10 cells
        let initial_cells: Vec<_> = (0..10).map(|i| make_cell(i * 100, 4)).collect();
        tracker.record_change_event(&initial_cells, &[], &initial_cells);

        // Change: keep 5, add 5 new
        let exits: Vec<_> = (0..5).map(|i| make_cell(i * 100, 4)).collect();
        let entries: Vec<_> = (10..15).map(|i| make_cell(i * 100, 4)).collect();
        let new_cells: Vec<_> = (5..15).map(|i| make_cell(i * 100, 4)).collect();

        tracker.record_change_event(&entries, &exits, &new_cells);

        // Jaccard for old={0..10} vs new={5..15}:
        // Intersection = {5..10} = 5 cells
        // Union = {0..15} = 15 cells
        // Jaccard = 5/15 = 0.333
        // After EWMA: jaccard_ewma = 0.9 * 1.0 + 0.1 * 0.333 ≈ 0.933
        // Actually wait, the EWMA formula is: (1-α)*old + α*new
        // where α = 0.1, so: 0.9 * 1.0 + 0.1 * 0.333 ≈ 0.933
        assert!(tracker.jaccard_ewma < 1.0, "Jaccard should decrease");
        assert!(
            tracker.jaccard_ewma > 0.5,
            "Jaccard EWMA should not drop too fast due to smoothing"
        );
    }

    /// Churn is forgiven with quiet: after an initial burst that populates the
    /// set, twenty observations reporting no change leave the smoothed change
    /// rate below where the burst left it. Past turbulence must be able to age
    /// out, or a dimension that had one bad day could never afterwards be
    /// judged stable.
    ///
    /// ´claim:wellness:a-run-of-unchanged-observations-decays-the-change-rate-back-down´
    /// ´test:unit:identity-change-rate-decays´
    #[test]
    fn identity_change_rate_decays() {
        let mut tracker = IdentityConvergenceTracker::new(PersistentTimestamp::new(0, 0));

        let cells: Vec<_> = (0..5).map(|i| make_cell(i * 100, 4)).collect();

        // Initial high-change event
        tracker.record_change_event(&cells, &[], &cells);
        let rate_after_initial = tracker.change_rate_ewma;

        // Multiple no-change events should decay the rate
        for _ in 0..20 {
            tracker.record_change_event(&[], &[], &cells);
        }

        assert!(
            tracker.change_rate_ewma < rate_after_initial,
            "Change rate should decay: {} < {}",
            tracker.change_rate_ewma,
            rate_after_initial
        );
    }
}
