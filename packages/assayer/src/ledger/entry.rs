// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`new_neutral_has_zero_ewmas`] | ledger | A cell that has just started being tracked carries no outcome history: every EWMA sits at zero and no axis rows exist yet. A cell must earn its reputation from observations rather than being born with one, so a freshly-appeared cell cannot be read as evidence of anything. |
//! | [`action_ordinal_values`] | ledger | Each action occupies a fixed slot in the per-action tally, in the declared order allow, challenge, slow, block. The counts are a bare array indexed by that ordinal, so the mapping is what keeps a persisted or snapshotted entry's tallies attributable to the actions that produced them. |
//! | [`materiality_evidence_counts_only_eligible_labels`] | ledger | The raw pair behind the true adverse rate is conditioned on eligibility on both sides: an ineligible adverse label moves the bad-rate average but neither the numerator nor the denominator of the raw rate, while an eligible adverse label moves both. The attenuation data is indexed in eligible labels, so a numerator counting labels the denominator never saw would divide two different populations against each other and report a rate belonging to neither. |
//! | [`the_arrival_window_reproduces_the_specified_attenuation`] | ledger | The stored arrival window, fed one eligible label per specified inter-arrival interval, yields the attenuation the convergence data tabulates at one, three and a half, ten and a hundred eligible labels a day. The window is only worth storing if it reproduces the table the theorem argues from, so the four tabulated rows are checked against the stored evidence rather than against a formula transcribed beside it. |
//! | [`the_arrival_window_decays_towards_silence`] | ledger | A window read after a long silence reports a lower load, a lower implied rate and a lower attenuation than the same window read at its last label, and an entry that has never seen an eligible label reports no rate at all. A cell that has stopped receiving evidence is not still receiving it, so the read-time decay has to carry the window down exactly as it carries the averages down. |

//! Ledger entry types for outcome tracking.
//!
//! This module provides the core data structures for outcome tracking:
//!
//! - [`LedgerEntry`] — The per-cell entry storing EWMAs and counts
//! - [`DecayedView`] — Read-only view of decayed EWMA values
//! - [`LedgerUpdate`] — Input for write-time updates
//! - [`ImmaturityCriteria`] — The thresholds the maturity predicate reads
//!
//! # EWMA Model
//!
//! The ledger uses a lazy-read/write-decay EWMA model. Decay is applied
//! at read time (pure, non-mutating) and at write time (before EWMA step).
//!
//! # Cross-References
//!
//! - (´tab:ledger:entry-state´) — the fields an entry carries
//! - (´alg:ledger:all-layers-update´) — the EWMA step applied to each
//! - (´def:ledger:time-decay´) — the time-indexed decay paid first
//! - (´conv:valence:sign´) — which direction counts as adverse
//! - (´thm:ledger:materiality´) — the two conditions the stored evidence
//!   answers, and why an average alone cannot answer them
//! - (´data:ledger:attenuation´) — the steady-state attenuation the arrival
//!   window reproduces

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::numerics::decay_factor_since;
use crate::types::{Action, OutcomeAxisId, PersistentTimestamp};

// ═══════════════════════════════════════════════════════════════════════════════
// Ledger Entry
// ═══════════════════════════════════════════════════════════════════════════════

/// Per-cell outcome ledger entry.
///
/// Stores EWMA statistics for outcome tracking at a single cell in the
/// dyadic hierarchy (´tab:ledger:entry-state´).
///
/// # Field Summary
///
/// | Field | Purpose |
/// |-------|---------|
/// | `total_assessments` | Total assessed decision events at this cell |
/// | `per_action_count` | Counts indexed by [`Action`] ordinal |
/// | `ewma_bad_rate` | EWMA of adverse outcome rate |
/// | `compressed_valence_ewma` | EWMA of `tanh(v / κᵥ)` |
/// | `raw_valence_ewma` | EWMA of raw valence |
/// | `per_axis` | Per-axis EWMAs (`axis_id`, compressed, raw) |
/// | `last_updated` | Timestamp for lazy decay |
/// | `recent_eligible_count` | For cell-set maintenance, and the raw rate's denominator |
/// | `adverse_eligible_count` | The raw rate's numerator |
/// | `eligible_arrival_load` | The recent eligible-label arrival window |
/// | `consecutive_absent` | Cycles absent from report |
///
/// # The materiality evidence
///
/// Three of those fields exist for one question the averages cannot answer.
/// A Ledger average is attenuated by elapsed-time decay before anything reads
/// it, so a low average is two situations at once: a cell whose true adverse
/// rate is ordinary, and a cell whose true rate is high but whose eligible
/// labels arrive too sparsely for the average to hold the excess
/// (´thm:ledger:materiality´). Telling them apart calls for the true rate and
/// the arrival rate as separate evidence, which is what the raw pair and the
/// arrival window are (´def:monitoring:ledger-value´).
///
/// The raw pair is undecayed on both sides and conditioned on eligibility on
/// both sides. Eligibility is the condition because the convergence and
/// attenuation data are indexed in eligible labels
/// (´data:ledger:attenuation´), so a rate measured over a wider population
/// would be attenuated by a factor computed for a narrower one.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct LedgerEntry {
    /// Total assessed decision events at this cell.
    pub total_assessments: u64,

    /// Per-action counts, indexed by [`Action`] ordinal.
    ///
    /// Order: Allow (0), Challenge (1), Slow (2), Block (3).
    pub per_action_count: [u64; 4],

    /// EWMA of adverse outcome rate (`is_positive == true`).
    ///
    /// Positive valence is adverse (´conv:valence:sign´); this EWMA tracks
    /// the rate of adverse outcomes (´alg:ledger:all-layers-update´).
    pub ewma_bad_rate: f64,

    /// EWMA of compressed valence: `tanh(v / κᵥ)`.
    pub compressed_valence_ewma: f64,

    /// EWMA of raw valence.
    pub raw_valence_ewma: f64,

    /// Per-axis EWMAs: `(axis_id, compressed, raw)`.
    ///
    /// Uses `Vec` rather than `HashMap` to minimise per-entry overhead.
    /// Linear search is adequate for the typical case (`m_s` ≤ 5), which is
    /// the per-entry shape the state table fixes (´tab:ledger:entry-state´).
    pub per_axis: Vec<(OutcomeAxisId, f64, f64)>,

    /// Timestamp of last update (for lazy decay).
    pub last_updated: PersistentTimestamp,

    /// Recent eligible assessment count (for cell-set maintenance).
    ///
    /// Cumulative and undecayed, clipped at its consumers rather than in
    /// storage (´tab:ledger:entry-state´). It is therefore also the
    /// denominator of the raw adverse rate.
    pub recent_eligible_count: u64,

    /// Eligible labels at this cell whose outcome was adverse.
    ///
    /// Cumulative and undecayed, over exactly the population
    /// `recent_eligible_count` counts. Their ratio is the cell's true adverse
    /// rate as observed, the quantity the materiality theorem's first
    /// condition is about (´thm:ledger:materiality´).
    ///
    /// Positive valence is adverse (´conv:valence:sign´).
    pub adverse_eligible_count: u64,

    /// Exponentially time-weighted count of recent eligible labels.
    ///
    /// The arrival-rate window. Each eligible label adds one; every elapsed
    /// interval multiplies the accumulation by the Ledger's own hourly decay,
    /// at write time and at read time alike (´def:ledger:time-decay´). The
    /// units are therefore eligible labels, weighted by how recently they
    /// arrived, and the window's shape is the Ledger's twenty-nine-day
    /// half-life rather than a second horizon of its own.
    ///
    /// Holding the window this way is what makes the attenuation a division
    /// rather than an inversion. A cell receiving one eligible label every
    /// `Δh` hours settles at `L = 1 / (1 - γ_t^Δh)`, so the per-interval decay
    /// the attenuation formula needs is `γ_t^Δh = 1 - 1/L` — recovered from
    /// the stored load alone, with no arrival interval to estimate and no
    /// second smoothing constant to choose (´data:ledger:attenuation´).
    pub eligible_arrival_load: f64,

    /// Consecutive report cycles where this cell was absent.
    ///
    /// Used by cell-set maintenance to trigger deletion.
    pub consecutive_absent: u8,
}

impl LedgerEntry {
    /// Creates a new neutral entry with zero EWMAs.
    ///
    /// Timestamp is set to the current time.
    #[must_use]
    pub fn new_neutral() -> Self {
        Self {
            total_assessments: 0,
            per_action_count: [0; 4],
            ewma_bad_rate: 0.0,
            compressed_valence_ewma: 0.0,
            raw_valence_ewma: 0.0,
            per_axis: Vec::new(),
            last_updated: PersistentTimestamp::now(),
            recent_eligible_count: 0,
            adverse_eligible_count: 0,
            eligible_arrival_load: 0.0,
            consecutive_absent: 0,
        }
    }

    /// Returns a decayed view of this entry's EWMAs.
    ///
    /// This is a pure function — the entry is not mutated.
    ///
    /// # Arguments
    ///
    /// * `gamma_t` — Hourly decay rate for time-based decay
    /// * `now` — Current timestamp for decay computation
    #[must_use]
    pub fn read_decayed(&self, gamma_t: f64, now: &PersistentTimestamp) -> DecayedView {
        let factor = decay_factor_since(gamma_t, &self.last_updated, now);

        DecayedView {
            bad_rate: self.ewma_bad_rate * factor,
            compressed_valence: self.compressed_valence_ewma * factor,
            raw_valence: self.raw_valence_ewma * factor,
            per_axis: self.per_axis.iter().map(|&(id, c, r)| (id, c * factor, r * factor)).collect(),
        }
    }

    /// Applies write-time decay, then performs an EWMA update.
    ///
    /// The decay is applied first (lazy write decay), then the EWMA step
    /// is performed using the provided update data.
    ///
    /// # Arguments
    ///
    /// * `gamma_t` — Hourly decay rate
    /// * `now` — Current timestamp
    /// * `update` — Update data
    /// * `lambda_l` — EWMA smoothing factor (`λ_L`)
    ///
    /// # EWMA Formula (´alg:ledger:all-layers-update´)
    ///
    /// ```text
    /// bar_b ← λ_L · bar_b + (1-λ_L) · is_positive
    /// bar_r_v ← λ_L · bar_r_v + (1-λ_L) · compressed_valence
    /// bar_v ← λ_L · bar_v + (1-λ_L) · raw_valence
    /// ```
    pub fn apply_write_decay_and_update(
        &mut self,
        gamma_t: f64,
        now: &PersistentTimestamp,
        update: &LedgerUpdate,
        lambda_l: f64,
    ) {
        // Step 1: Apply lazy decay
        let factor = decay_factor_since(gamma_t, &self.last_updated, now);
        self.ewma_bad_rate *= factor;
        self.compressed_valence_ewma *= factor;
        self.raw_valence_ewma *= factor;
        // The arrival window is decayed on the same clock and by the same
        // factor as the averages, which is what lets the load be read as an
        // inter-arrival decay (´def:ledger:time-decay´). Decaying in pieces at
        // every label and decaying once over the whole interval agree, because
        // the factor is multiplicative in elapsed time.
        self.eligible_arrival_load *= factor;
        for (_, c, r) in &mut self.per_axis {
            *c *= factor;
            *r *= factor;
        }

        // Step 2: EWMA update
        let alpha = 1.0 - lambda_l; // complement of smoothing factor
        // Positive valence = adverse = bad (´conv:valence:sign´).
        let is_bad = if update.is_positive { 1.0 } else { 0.0 };

        self.ewma_bad_rate = lambda_l.mul_add(self.ewma_bad_rate, alpha * is_bad);
        self.compressed_valence_ewma = lambda_l.mul_add(self.compressed_valence_ewma, alpha * update.compressed_valence);
        self.raw_valence_ewma = lambda_l.mul_add(self.raw_valence_ewma, alpha * update.raw_valence);

        // Per-axis updates
        for (&axis_id, &(compressed, raw)) in &update.axis_values {
            // Linear search — adequate for m_s ≤ 5 (´tab:ledger:entry-state´).
            if let Some((_, c, r)) = self.per_axis.iter_mut().find(|(id, _, _)| *id == axis_id) {
                *c = lambda_l.mul_add(*c, alpha * compressed);
                *r = lambda_l.mul_add(*r, alpha * raw);
            } else {
                // New axis — insert with initial EWMA step from zero.
                self.per_axis.push((axis_id, alpha * compressed, alpha * raw));
            }
        }

        // Step 3: Update counts
        self.total_assessments += 1;
        self.per_action_count[action_ordinal(update.action)] += 1;
        if update.is_eligible {
            self.recent_eligible_count += 1;
            self.eligible_arrival_load += 1.0;
            if update.is_positive {
                self.adverse_eligible_count += 1;
            }
        }

        // Step 4: Update timestamp and reset absence counter
        self.last_updated = *now;
        self.consecutive_absent = 0;
    }

    /// Returns the cell's observed true adverse rate, or `None` when no
    /// eligible label has reached it.
    ///
    /// Undecayed on both sides, so it is the rate the cell would show if the
    /// averages did not have to pay elapsed-time decay — the quantity the
    /// materiality theorem's first condition compares against the population
    /// (´thm:ledger:materiality´). Absence is reported rather than substituted:
    /// a cell with no eligible evidence has no rate, and a zero here would be
    /// a claim about the cell rather than a statement about the evidence.
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Justified: label counts stay far below f64's exact integer range.
    pub fn raw_adverse_rate(&self) -> Option<f64> {
        (self.recent_eligible_count > 0).then(|| self.adverse_eligible_count as f64 / self.recent_eligible_count as f64)
    }

    /// Returns the arrival window decayed to `now`, without mutating the entry.
    ///
    /// The read-time half of the two decay modes: the value persisting the
    /// decay would have produced (´def:ledger:time-decay´).
    #[must_use]
    pub fn decayed_eligible_load(&self, gamma_t: f64, now: &PersistentTimestamp) -> f64 {
        self.eligible_arrival_load * decay_factor_since(gamma_t, &self.last_updated, now)
    }

    /// Returns the steady-state attenuation the arrival window implies, or
    /// `None` when the window is empty.
    ///
    /// The attenuation of (´data:ledger:attenuation´) evaluated at this cell's
    /// own arrival rate. Writing the tabulated form
    /// `(1 - λ_L) / (1 - λ_L · γ_t^{24/r})` against the stored load `L`, whose
    /// steady state gives `γ_t^{24/r} = 1 - 1/L`, collapses it to
    ///
    /// ```text
    /// A = (1 - λ_L) / ((1 - λ_L) + λ_L / L)
    /// ```
    ///
    /// so the rate never has to be recovered to apply the factor. A cell whose
    /// window has decayed to a single label reads the floor `1 - λ_L`, which is
    /// what one isolated label leaves in an average, and a cell receiving
    /// labels faster than the decay erodes them approaches one.
    #[must_use]
    pub fn steady_state_attenuation(&self, gamma_t: f64, now: &PersistentTimestamp, lambda_l: f64) -> Option<f64> {
        let load = self.decayed_eligible_load(gamma_t, now);
        (load > 0.0).then(|| {
            let alpha = 1.0 - lambda_l;
            alpha / (alpha + lambda_l / load)
        })
    }

    /// Returns the eligible labels a day the arrival window implies, or `None`
    /// when the window is too thin to name a rate.
    ///
    /// The inverse reading of the same steady state, reported rather than
    /// consumed: `γ_t^{24/r} = 1 - 1/L` solved for `r`. A load at or below one
    /// names no rate, because a single label establishes no interval between
    /// arrivals to measure a rate over.
    #[must_use]
    pub fn eligible_arrival_rate_per_day(&self, gamma_t: f64, now: &PersistentTimestamp) -> Option<f64> {
        let load = self.decayed_eligible_load(gamma_t, now);
        if load <= 1.0 || gamma_t <= 0.0 || gamma_t >= 1.0 {
            return None;
        }
        Some(24.0 * gamma_t.ln() / (1.0 - 1.0 / load).ln())
    }

    /// Returns `true` where this entry's outcome features are present but not
    /// yet worth weighting (´def:runtime:alarm-summary´).
    ///
    /// The materiality theorem's two conditions, taken as a disjunction because
    /// either one alone leaves the features attenuated towards
    /// uninformativeness (´thm:ledger:materiality´). The first arm counts the
    /// cell's eligible labels against the materiality threshold. The second
    /// reads the steady-state attenuation this cell's own arrival window
    /// implies and compares it against the configured floor
    /// (´data:ledger:attenuation´). The arms are independent: a cell can hold
    /// plenty of labels whose arrivals are too sparse to preserve an excess,
    /// and a cell can be receiving densely while holding too few labels to
    /// have measured anything.
    ///
    /// An empty window names no attenuation, and what the second arm wants
    /// there is the limit rather than an absence: `A = α / (α + λ_L / L)` tends
    /// to zero as `L` does, so a cell with nothing left in its window reads as
    /// maximally attenuated and the arm holds. That is also what stops a cell
    /// whose window has decayed away from passing as mature on a count it
    /// accumulated long ago and has not added to since.
    #[must_use]
    pub fn is_immature(&self, criteria: &ImmaturityCriteria, gamma_t: f64, now: &PersistentTimestamp) -> bool {
        if self.recent_eligible_count < criteria.materiality_threshold {
            return true;
        }

        let attenuation = self.steady_state_attenuation(gamma_t, now, criteria.lambda_l).unwrap_or(0.0);
        attenuation < criteria.attenuation_floor
    }

    /// Returns `true` if all EWMAs are below the given floor.
    #[must_use]
    pub fn all_ewmas_below(&self, floor: f64) -> bool {
        self.ewma_bad_rate.abs() < floor
            && self.compressed_valence_ewma.abs() < floor
            && self.raw_valence_ewma.abs() < floor
            && self.per_axis.iter().all(|(_, c, r)| c.abs() < floor && r.abs() < floor)
    }
}

impl Default for LedgerEntry {
    fn default() -> Self {
        Self::new_neutral()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Immaturity Criteria
// ═══════════════════════════════════════════════════════════════════════════════

/// The thresholds the per-assessment immaturity predicate is taken against
/// (´def:runtime:alarm-summary´).
///
/// Two configured numbers and the Ledger's own averaging rate, gathered into
/// one carrier because the predicate reads all three or none. They are
/// restated here rather than declared: the eligible-label floor is
/// (´def:config:ledger-materiality-threshold´), the attenuation floor is
/// (´def:config:attenuation-materiality-floor´), and `λ_L` is the Ledger rate
/// the state table fixes (´tab:ledger:entry-state´), carried because it is
/// what turns a stored arrival window into an attenuation.
#[derive(Clone, Copy, Debug)]
pub struct ImmaturityCriteria {
    /// Eligible-label floor below which a cell is immature.
    pub materiality_threshold: u64,

    /// Steady-state attenuation below which a cell is immature.
    pub attenuation_floor: f64,

    /// Label-indexed Ledger averaging rate `λ_L`.
    pub lambda_l: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Decayed View
// ═══════════════════════════════════════════════════════════════════════════════

/// Read-only view of decayed EWMA values.
///
/// Produced by [`LedgerEntry::read_decayed()`]. Contains time-decayed
/// versions of all EWMA fields without mutating the source entry.
#[derive(Clone, Debug)]
pub struct DecayedView {
    /// Decayed adverse outcome rate.
    pub bad_rate: f64,

    /// Decayed compressed valence.
    pub compressed_valence: f64,

    /// Decayed raw valence.
    pub raw_valence: f64,

    /// Decayed per-axis EWMAs: `(axis_id, compressed, raw)`.
    pub per_axis: Vec<(OutcomeAxisId, f64, f64)>,
}

impl DecayedView {
    /// Creates a neutral `DecayedView` with all zeros.
    #[must_use]
    pub const fn neutral() -> Self {
        Self {
            bad_rate: 0.0,
            compressed_valence: 0.0,
            raw_valence: 0.0,
            per_axis: Vec::new(),
        }
    }
}

impl Default for DecayedView {
    fn default() -> Self {
        Self::neutral()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Ledger Update
// ═══════════════════════════════════════════════════════════════════════════════

/// Input data for a ledger write-time update.
///
/// Contains the outcome information from a single labelled assessment.
#[derive(Clone, Debug)]
pub struct LedgerUpdate {
    /// Whether the outcome is positive (`v > 0`).
    pub is_positive: bool,

    /// Compressed valence: `tanh(v / κᵥ)`.
    pub compressed_valence: f64,

    /// Raw valence value.
    pub raw_valence: f64,

    /// Per-axis values: `(compressed, raw)` keyed by axis ID.
    ///
    /// Uses `HashMap` for incoming updates (ephemeral, not stored).
    pub axis_values: std::collections::HashMap<OutcomeAxisId, (f64, f64)>,

    /// Action taken for this assessment.
    pub action: Action,

    /// Whether this assessment is eligible for cell-set maintenance.
    pub is_eligible: bool,
}

impl LedgerUpdate {
    /// Creates a new update from outcome values.
    #[must_use]
    pub fn new(is_positive: bool, compressed_valence: f64, raw_valence: f64, action: Action, is_eligible: bool) -> Self {
        Self {
            is_positive,
            compressed_valence,
            raw_valence,
            axis_values: std::collections::HashMap::new(),
            action,
            is_eligible,
        }
    }

    /// Adds a per-axis value to the update.
    pub fn with_axis(mut self, axis_id: OutcomeAxisId, compressed: f64, raw: f64) -> Self {
        self.axis_values.insert(axis_id, (compressed, raw));
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns the ordinal index for an [`Action`].
///
/// Used to index into `per_action_count`.
#[inline]
#[must_use]
const fn action_ordinal(action: Action) -> usize {
    match action {
        Action::Allow => 0,
        Action::Challenge => 1,
        Action::Slow => 2,
        Action::Block => 3,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A cell that has just started being tracked carries no outcome history:
    /// every EWMA sits at zero and no axis rows exist yet. A cell must earn its
    /// reputation from observations rather than being born with one, so a
    /// freshly-appeared cell cannot be read as evidence of anything.
    ///
    /// ´claim:ledger:a-fresh-entry-carries-no-outcome-history´
    /// ´test:unit:new-neutral-has-zero-ewmas´
    #[test]
    fn new_neutral_has_zero_ewmas() {
        let entry = LedgerEntry::new_neutral();
        assert!((entry.ewma_bad_rate - 0.0).abs() < f64::EPSILON);
        assert!((entry.compressed_valence_ewma - 0.0).abs() < f64::EPSILON);
        assert!((entry.raw_valence_ewma - 0.0).abs() < f64::EPSILON);
        assert_eq!(entry.per_axis, [] as [(OutcomeAxisId, f64, f64); 0]);
    }

    /// Each action occupies a fixed slot in the per-action tally, in the
    /// declared order allow, challenge, slow, block. The counts are a bare
    /// array indexed by that ordinal, so the mapping is what keeps a
    /// persisted or snapshotted entry's tallies attributable to the actions
    /// that produced them.
    ///
    /// ´claim:ledger:each-action-has-a-fixed-slot-in-the-per-action-tally´
    /// ´test:unit:action-ordinal-values´
    #[test]
    fn action_ordinal_values() {
        assert_eq!(action_ordinal(Action::Allow), 0);
        assert_eq!(action_ordinal(Action::Challenge), 1);
        assert_eq!(action_ordinal(Action::Slow), 2);
        assert_eq!(action_ordinal(Action::Block), 3);
    }

    /// The Ledger's own two rates, as the state table and the decay definition
    /// fix them (´tab:ledger:entry-state´), (´def:ledger:time-decay´).
    const LAMBDA_L: f64 = 0.999;
    const GAMMA_T_LEDGER: f64 = 0.999;

    /// An entry stamped at the epoch, so every elapsed interval below is the
    /// timestamp it is read at.
    fn entry_at_epoch() -> LedgerEntry {
        let mut entry = LedgerEntry::new_neutral();
        entry.last_updated = PersistentTimestamp::new(0, 0);
        entry
    }

    /// Hours of driving before a window is read as settled.
    ///
    /// The load approaches its steady state geometrically in elapsed time at
    /// the Ledger's own hourly rate, so twenty thousand hours leaves a residue
    /// of `0.999^20000`, nine orders of magnitude below the readings compared
    /// here — and it is the same residue at every arrival rate, which is what
    /// lets one figure serve all four tabulated rows.
    const SETTLING_HOURS: f64 = 20_000.0;

    /// Feeds one eligible label every `interval_hours` until the window has
    /// settled, returning the entry and the timestamp of the last arrival.
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // Justified: the driven horizons are small, positive and whole.
    fn drive_eligible_arrivals(interval_hours: f64) -> (LedgerEntry, PersistentTimestamp) {
        let mut entry = entry_at_epoch();
        let update = LedgerUpdate::new(true, 0.0, 0.0, Action::Allow, true);
        let mut at = PersistentTimestamp::new(0, 0);
        let arrivals = (SETTLING_HOURS / interval_hours).ceil() as u32;
        for step in 1..=arrivals {
            let seconds = (f64::from(step) * interval_hours * 3600.0).round() as i64;
            at = PersistentTimestamp::new(seconds, 0);
            entry.apply_write_decay_and_update(GAMMA_T_LEDGER, &at, &update, LAMBDA_L);
        }
        (entry, at)
    }

    /// The raw pair behind the true adverse rate is conditioned on eligibility
    /// on both sides: an ineligible adverse label moves the bad-rate average
    /// but neither the numerator nor the denominator of the raw rate, while an
    /// eligible adverse label moves both. The attenuation data is indexed in
    /// eligible labels, so a numerator counting labels the denominator never
    /// saw would divide two different populations against each other and
    /// report a rate belonging to neither.
    ///
    /// ´claim:ledger:the-raw-materiality-pair-counts-eligible-labels-on-both-sides´
    /// ´test:unit:materiality-evidence-counts-only-eligible-labels´
    #[test]
    fn materiality_evidence_counts_only_eligible_labels() {
        let mut entry = entry_at_epoch();
        let at = PersistentTimestamp::new(0, 0);

        // No eligible evidence at all: the rate is absent, not zero.
        assert_eq!(entry.raw_adverse_rate(), None);

        // An adverse but ineligible label moves the average and nothing else.
        let ineligible_adverse = LedgerUpdate::new(true, 0.0, 0.0, Action::Allow, false);
        entry.apply_write_decay_and_update(GAMMA_T_LEDGER, &at, &ineligible_adverse, LAMBDA_L);
        assert!(entry.ewma_bad_rate > 0.0, "the average takes every label");
        assert_eq!(entry.recent_eligible_count, 0);
        assert_eq!(entry.adverse_eligible_count, 0);
        assert_eq!(entry.raw_adverse_rate(), None, "an ineligible label is no eligible evidence");

        // Four eligible labels, one of them adverse: one in four.
        let eligible_adverse = LedgerUpdate::new(true, 0.0, 0.0, Action::Allow, true);
        let eligible_benign = LedgerUpdate::new(false, 0.0, 0.0, Action::Allow, true);
        entry.apply_write_decay_and_update(GAMMA_T_LEDGER, &at, &eligible_adverse, LAMBDA_L);
        for _ in 0..3 {
            entry.apply_write_decay_and_update(GAMMA_T_LEDGER, &at, &eligible_benign, LAMBDA_L);
        }
        assert_eq!(entry.recent_eligible_count, 4);
        assert_eq!(entry.adverse_eligible_count, 1);
        let rate = entry.raw_adverse_rate().expect("eligible evidence names a rate");
        assert!((rate - 0.25).abs() < 1e-12, "one adverse in four eligible: {rate}");

        // The window counts arrivals, not labels: four eligible, five total.
        assert!(
            (entry.eligible_arrival_load - 4.0).abs() < 1e-9,
            "the window took the four eligible arrivals: {}",
            entry.eligible_arrival_load
        );
    }

    /// The stored arrival window, fed one eligible label per specified
    /// inter-arrival interval, yields the attenuation the convergence data
    /// tabulates at one, three and a half, ten and a hundred eligible labels a
    /// day. The window is only worth storing if it reproduces the table the
    /// theorem argues from, so the four tabulated rows are checked against the
    /// stored evidence rather than against a formula transcribed beside it.
    ///
    /// ´claim:ledger:the-stored-arrival-window-reproduces-the-tabulated-steady-state-attenuation´
    /// ´test:unit:the-arrival-window-reproduces-the-specified-attenuation´
    #[test]
    fn the_arrival_window_reproduces_the_specified_attenuation() {
        // The four rows of (´data:ledger:attenuation´): labels a day, and the
        // attenuation the table gives for them.
        let tabulated = [(1.0_f64, 0.040_f64), (3.4, 0.124), (10.0, 0.294), (100.0, 0.806)];

        for (rate, expected) in tabulated {
            let interval_hours = 24.0 / rate;
            let (entry, at) = drive_eligible_arrivals(interval_hours);

            let attenuation = entry
                .steady_state_attenuation(GAMMA_T_LEDGER, &at, LAMBDA_L)
                .expect("a driven window names an attenuation");
            // The table states its attenuations to a tenth of a per cent, so
            // agreement is judged at the precision the table is written to.
            assert!(
                (attenuation - expected).abs() < 1e-3,
                "at {rate} labels a day the table gives {expected}, the window gives {attenuation}"
            );

            // And the window names the rate that produced it.
            let implied = entry
                .eligible_arrival_rate_per_day(GAMMA_T_LEDGER, &at)
                .expect("a driven window names a rate");
            assert!(
                (implied - rate).abs() < rate * 0.01,
                "the window implies {implied} labels a day against the {rate} it was driven at"
            );
        }
    }

    /// A window read after a long silence reports a lower load, a lower implied
    /// rate and a lower attenuation than the same window read at its last
    /// label, and an entry that has never seen an eligible label reports no
    /// rate at all. A cell that has stopped receiving evidence is not still
    /// receiving it, so the read-time decay has to carry the window down
    /// exactly as it carries the averages down.
    ///
    /// ´claim:ledger:the-arrival-window-falls-with-elapsed-silence-at-read-time´
    /// ´test:unit:the-arrival-window-decays-towards-silence´
    #[test]
    fn the_arrival_window_decays_towards_silence() {
        // Ten eligible labels a day, driven to steady state.
        let (entry, at) = drive_eligible_arrivals(2.4);

        let load_now = entry.decayed_eligible_load(GAMMA_T_LEDGER, &at);
        let rate_now = entry.eligible_arrival_rate_per_day(GAMMA_T_LEDGER, &at).expect("a rate");
        let attenuation_now = entry
            .steady_state_attenuation(GAMMA_T_LEDGER, &at, LAMBDA_L)
            .expect("an attenuation");

        // Sixty days of silence — twice the Ledger's own half-life over.
        let later = PersistentTimestamp::new(at.seconds + 60 * 24 * 3600, 0);
        let load_later = entry.decayed_eligible_load(GAMMA_T_LEDGER, &later);
        let rate_later = entry.eligible_arrival_rate_per_day(GAMMA_T_LEDGER, &later).expect("a rate");
        let attenuation_later = entry
            .steady_state_attenuation(GAMMA_T_LEDGER, &later, LAMBDA_L)
            .expect("an attenuation");

        assert!(
            load_later < load_now,
            "silence lowers the load: {load_later} against {load_now}"
        );
        assert!(
            rate_later < rate_now,
            "silence lowers the implied rate: {rate_later} against {rate_now}"
        );
        assert!(
            attenuation_later < attenuation_now,
            "silence lowers the attenuation: {attenuation_later} against {attenuation_now}"
        );

        // The read is pure: the stored load is where the write left it.
        assert!(
            (entry.eligible_arrival_load - load_now).abs() < 1e-9,
            "a read at the last-update instant is the stored value itself"
        );

        // A cell that has never had an eligible label has no window to read.
        let fresh = entry_at_epoch();
        assert_eq!(fresh.eligible_arrival_rate_per_day(GAMMA_T_LEDGER, &at), None);
        assert_eq!(fresh.steady_state_attenuation(GAMMA_T_LEDGER, &at, LAMBDA_L), None);
        assert_eq!(fresh.raw_adverse_rate(), None);
    }
}
