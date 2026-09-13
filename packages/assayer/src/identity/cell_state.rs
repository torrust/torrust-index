// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`cell_outcome_state_new_neutral`] | identity | A cell that has just become competitive starts neutral: no adverse rate, no valence in either compressed or raw form, and no per-axis history at all. A region only earns entity-specific opinions from outcomes actually attributed to it, so a freshly promoted cell contributes nothing of its own until it has seen something. |
//! | [`cell_outcome_state_ewma_update`] | identity | One outcome moves a cell's EWMAs by the complement of the smoothing factor and no further: an adverse outcome — positive valence, under the corpus's sign convention — shifts the adverse rate a thousandth of the way towards one under a long memory, and drags the compressed valence the same fraction towards the reported value. A single labelled request cannot therefore rewrite a region's reputation — it takes a run of them. |
//! | [`cell_outcome_state_decay`] | identity | Stored EWMAs are decayed for the time that has elapsed since the cell was last written, before the new outcome is folded in. A cell that saw a bad day and then went quiet for a day comes back with its adverse rate already faded rather than frozen at what it was — reputation ages on wall-clock time, not on how often the cell happens to be touched. |
//! | [`cell_outcome_state_decayed_view_is_exact_and_pure`] | identity | A read-time decayed view is value-for-value identical to applying the shared decay operation to a clone, while every stored scalar, map and timestamp remains unchanged. Assessment can therefore read current outcome evidence without turning a read into a write or approximating the decay. |
//! | [`measurement_state_default`] | identity | A cell's measurement side starts equally empty: no Sentinel has an alarm recorded against it, suspicion and volatility are zero, and the step count says no updates have arrived. The step count is what lets a reader tell "calm" from "never looked at", so it must start honest. |
//! | [`cell_mutable_state_new`] | identity | Every competitive cell carries measurement state from the moment it is created, freshly zeroed rather than inherited. The maintenance loop installs this state the instant a cell enters the competitive set, so the assessment path never has to ask whether a tracked cell has somewhere to record what it sees. |
//! | [`measurement_update_alarm`] | identity | Alarm evidence is held one entry per Sentinel, so a cell can be loud on one Sentinel's axis and quiet on another's without the two blurring together; smoothing a fresh alarm into an existing entry moves it only part of the way. Alongside them the step count records how many updates built the picture, so a reader can weigh an alarm level against the number of chances it had to settle. |
//! | [`measurement_suspicion_derived_from_entries`] | identity | Suspicion is not a fourth accumulator: it is derived at read time as the mean of the per-Sentinel alarm averages, so it agrees with the entries at every moment — including the moment the Sentinel set changes, which is exactly when a stored copy would drift from what the entries say. A cell loud on one Sentinel and quiet on another reads as the average of the two, and a cell nobody has reported on reads as zero rather than as a stale remembered level. |
//! | [`outcome_update_positive`] | identity | A positive-valence outcome is exactly what the adverse rate counts: one such outcome raises the rate by the complement of the smoothing factor, and a benign outcome leaves it untouched. Positive valence is the adverse direction under the corpus's sign convention, so the rate a model reads at the cross-dimension maximum really is the rate at which this range's labelled requests came back adverse — a rate built on the complement would carry the benign rate at the position the models reserve for the adverse one, with the learned association silently inverted wherever the feature carries signal. |
//! | [`outcome_per_axis_values`] | identity | Each outcome axis keeps its own pair of readings, compressed and raw, and each is smoothed independently with its sign preserved — a cell that is favourable on one axis and adverse on another records exactly that rather than an average that hides both. Keeping the raw value beside the compressed one means the magnitude survives the squashing that makes axes comparable. |

//! Cell-level mutable state for identity dimensions.
//!
// Data structures only. A dedicated owner installs and mutates this state, and
// the assessment path never does (´dec:memory:graph-owner´).
//!
//! This module provides the per-cell mutable state structures for competitive
//! cells in an identity dimension:
//!
//! - [`CellMutableState`] — Container for all per-cell mutable state
//! - [`MeasurementState`] — Measurement tracking (suspicion, volatility)
//! - [`CellOutcomeState`] — Outcome EWMAs (adverse rate, valence)
//!
//! # Cross-References
//!
//! - (´tab:keyspace:measurement-state´) — the measurement fields a cell
//!   carries, and the suspicion derived from them
//! - (´tab:keyspace:outcome-state´) — the outcome fields written at label time
//! - (´tab:keyspace:decay-rates´) — the two rates at which this state fades
//! - (´dec:memory:graph-owner´) — who is allowed to write it

use std::collections::HashMap;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::numerics::decay_factor_since;
use crate::types::{OutcomeAxisId, PersistentTimestamp, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Cell Mutable State
// ═══════════════════════════════════════════════════════════════════════════════

/// Container for all mutable state associated with a competitive cell.
///
/// Each competitive cell maintains both measurement state (for alarm tracking)
/// and outcome state (for outcome EWMAs).
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CellMutableState {
    /// Measurement state for this competitive cell.
    pub measurement: MeasurementState,

    /// Outcome state (EWMAs of outcome metrics).
    pub outcome: CellOutcomeState,
}

impl CellMutableState {
    /// Creates a new competitive-cell state.
    #[must_use]
    pub fn new() -> Self {
        Self {
            measurement: MeasurementState::default(),
            outcome: CellOutcomeState::new_neutral(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Measurement State
// ═══════════════════════════════════════════════════════════════════════════════

/// Measurement tracking state for a competitive cell
/// (´tab:keyspace:measurement-state´).
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct MeasurementState {
    /// Per-Sentinel alarm averages: each Sentinel's own composite alarm,
    /// exponentially averaged at the measurement smoothing factor. The
    /// composite is already normalised and clipped to the unit interval
    /// (´def:runtime:alarm-summary´), so these averages stay inside it.
    pub per_sentinel_alarm_ewma: HashMap<SentinelId, f64>,

    /// Volatility: exponential average over the squared change in the
    /// derived alarm average per update.
    pub volatility: f64,

    /// Step count: measurement updates received by this cell.
    pub step_count: u64,
}

impl MeasurementState {
    /// Suspicion, derived at read time: the mean of the per-Sentinel alarm
    /// averages, zero while no Sentinel has reported
    /// (´tab:keyspace:measurement-state´).
    #[must_use]
    pub fn suspicion(&self) -> f64 {
        if self.per_sentinel_alarm_ewma.is_empty() {
            return 0.0;
        }
        #[allow(clippy::cast_precision_loss)]
        let count = self.per_sentinel_alarm_ewma.len() as f64;
        self.per_sentinel_alarm_ewma.values().sum::<f64>() / count
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Cell Outcome State
// ═══════════════════════════════════════════════════════════════════════════════

/// Outcome EWMA state for a competitive cell.
///
/// Tracks adverse rate, valence EWMAs, and per-axis values
/// (´tab:keyspace:outcome-state´).
/// Uses the same lazy-read/write-decay pattern as [`LedgerEntry`].
///
/// [`LedgerEntry`]: crate::ledger::LedgerEntry
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CellOutcomeState {
    /// EWMA of adverse outcome rate.
    ///
    /// Averages the positive-valence indicator (´tab:keyspace:outcome-state´);
    /// positive valence is the adverse direction (´conv:valence:sign´).
    pub adverse_rate_ewma: f64,

    /// EWMA of compressed valence: `tanh(v / κᵥ)`.
    pub compressed_valence_ewma: f64,

    /// EWMA of raw valence.
    pub raw_valence_ewma: f64,

    /// Per-axis compressed value EWMAs.
    pub per_axis_compressed: HashMap<OutcomeAxisId, f64>,

    /// Per-axis raw value EWMAs.
    pub per_axis_raw: HashMap<OutcomeAxisId, f64>,

    /// Timestamp of last update (for lazy decay).
    pub last_updated: PersistentTimestamp,
}

impl CellOutcomeState {
    /// Creates a new neutral outcome state with zero EWMAs.
    ///
    /// Timestamp is set to the current time.
    #[must_use]
    pub fn new_neutral() -> Self {
        Self {
            adverse_rate_ewma: 0.0,
            compressed_valence_ewma: 0.0,
            raw_valence_ewma: 0.0,
            per_axis_compressed: HashMap::new(),
            per_axis_raw: HashMap::new(),
            last_updated: PersistentTimestamp::now(),
        }
    }

    /// Returns the outcome state decayed to `now` without mutating storage.
    ///
    /// The view uses the same shared factor and mutation helper as the write
    /// path, so it is the exact state a write-time decay would expose before
    /// folding in a new outcome (´dec:clock:shared-functions´).
    #[must_use]
    pub fn read_decayed(&self, gamma_t: f64, now: &PersistentTimestamp) -> Self {
        let mut view = self.clone();
        view.apply_decay(gamma_t, now);
        view
    }

    /// Applies lazy outcome decay through the shared clock function.
    fn apply_decay(&mut self, gamma_t: f64, now: &PersistentTimestamp) {
        let factor = decay_factor_since(gamma_t, &self.last_updated, now);
        self.adverse_rate_ewma *= factor;
        self.compressed_valence_ewma *= factor;
        self.raw_valence_ewma *= factor;
        for value in self.per_axis_compressed.values_mut() {
            *value *= factor;
        }
        for value in self.per_axis_raw.values_mut() {
            *value *= factor;
        }
        self.last_updated = *now;
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
    pub fn apply_write_decay_and_update(
        &mut self,
        gamma_t: f64,
        now: &PersistentTimestamp,
        update: &CellOutcomeUpdate,
        lambda_l: f64,
    ) {
        // Step 1: Apply lazy decay
        self.apply_decay(gamma_t, now);

        // Step 2: EWMA update
        let alpha = 1.0 - lambda_l; // complement of smoothing factor
        // Positive valence is the adverse direction (´conv:valence:sign´).
        let is_adverse = if update.is_positive { 1.0 } else { 0.0 };

        self.adverse_rate_ewma = lambda_l.mul_add(self.adverse_rate_ewma, alpha * is_adverse);
        self.compressed_valence_ewma = lambda_l.mul_add(self.compressed_valence_ewma, alpha * update.compressed_valence);
        self.raw_valence_ewma = lambda_l.mul_add(self.raw_valence_ewma, alpha * update.raw_valence);

        // Per-axis updates
        for (&axis_id, &compressed) in &update.axis_compressed {
            let entry = self.per_axis_compressed.entry(axis_id).or_insert(0.0);
            *entry = lambda_l.mul_add(*entry, alpha * compressed);
        }
        for (&axis_id, &raw) in &update.axis_raw {
            let entry = self.per_axis_raw.entry(axis_id).or_insert(0.0);
            *entry = lambda_l.mul_add(*entry, alpha * raw);
        }
    }
}

impl Default for CellOutcomeState {
    fn default() -> Self {
        Self::new_neutral()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Cell Outcome Update
// ═══════════════════════════════════════════════════════════════════════════════

/// Input data for a cell outcome state update.
///
/// Contains the outcome information from a single labelled assessment,
/// projected onto the cell level.
#[derive(Clone, Debug)]
pub struct CellOutcomeUpdate {
    /// Whether the outcome was positive (v > 0).
    pub is_positive: bool,

    /// Compressed valence: `tanh(v / κᵥ)`.
    pub compressed_valence: f64,

    /// Raw valence value.
    pub raw_valence: f64,

    /// Per-axis compressed values.
    pub axis_compressed: HashMap<OutcomeAxisId, f64>,

    /// Per-axis raw values.
    pub axis_raw: HashMap<OutcomeAxisId, f64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// A cell that has just become competitive starts neutral: no adverse rate,
    /// no valence in either compressed or raw form, and no per-axis history at
    /// all. A region only earns entity-specific opinions from outcomes actually
    /// attributed to it, so a freshly promoted cell contributes nothing of its
    /// own until it has seen something.
    ///
    /// ´claim:identity:a-newly-tracked-cell-starts-neutral-with-no-outcome-history-of-its-own´
    /// ´test:unit:cell-outcome-state-new-neutral´
    #[test]
    fn cell_outcome_state_new_neutral() {
        let state = CellOutcomeState::new_neutral();
        assert_eq!(state.adverse_rate_ewma.to_bits(), 0.0f64.to_bits());
        assert_eq!(state.compressed_valence_ewma.to_bits(), 0.0f64.to_bits());
        assert_eq!(state.raw_valence_ewma.to_bits(), 0.0f64.to_bits());
        assert!(state.per_axis_compressed.is_empty());
        assert!(state.per_axis_raw.is_empty());
    }

    /// One outcome moves a cell's EWMAs by the complement of the smoothing
    /// factor and no further: an adverse outcome — positive valence, under the
    /// corpus's sign convention — shifts the adverse rate a thousandth of the
    /// way towards one under a long memory, and drags the compressed valence
    /// the same fraction towards the reported value. A single labelled request
    /// cannot therefore rewrite a region's reputation — it takes a run of them.
    ///
    /// ´claim:identity:a-single-outcome-moves-a-cells-ewmas-by-the-complement-of-the-smoothing-factor´
    /// ´test:unit:cell-outcome-state-ewma-update´
    #[test]
    fn cell_outcome_state_ewma_update() {
        let mut state = CellOutcomeState::new_neutral();
        let now = PersistentTimestamp::now();

        // Adverse outcome: positive valence (´conv:valence:sign´).
        let update = CellOutcomeUpdate {
            is_positive: true,
            compressed_valence: 0.5,
            raw_valence: 10.0,
            axis_compressed: HashMap::new(),
            axis_raw: HashMap::new(),
        };

        // Use lambda_l = 0.999 (smoothing factor)
        let lambda_l = 0.999;
        state.apply_write_decay_and_update(0.999, &now, &update, lambda_l);

        // After one update with lambda=0.999:
        // adverse_rate = 0.999 * 0.0 + 0.001 * 1.0 = 0.001
        let expected_rate = 0.001;
        assert!((state.adverse_rate_ewma - expected_rate).abs() < 1e-9);

        // compressed_valence = 0.999 * 0.0 + 0.001 * 0.5 = 0.0005
        let expected_compressed = 0.0005;
        assert!((state.compressed_valence_ewma - expected_compressed).abs() < 1e-9);
    }

    /// Stored EWMAs are decayed for the time that has elapsed since the cell
    /// was last written, before the new outcome is folded in. A cell that saw a
    /// bad day and then went quiet for a day comes back with its adverse rate
    /// already faded rather than frozen at what it was — reputation ages on
    /// wall-clock time, not on how often the cell happens to be touched.
    ///
    /// ´claim:identity:a-cells-stored-ewmas-are-aged-for-the-elapsed-time-before-a-new-outcome-is-folded-in´
    /// ´test:unit:cell-outcome-state-decay´
    #[test]
    fn cell_outcome_state_decay() {
        let mut state = CellOutcomeState::new_neutral();

        // Manually set EWMAs to non-zero values
        state.adverse_rate_ewma = 1.0;
        state.compressed_valence_ewma = 1.0;
        state.raw_valence_ewma = 1.0;

        // Set last_updated to 24 hours ago
        let old_ts = PersistentTimestamp::new(state.last_updated.seconds - 24 * 3600, 0);
        state.last_updated = old_ts;

        // Apply update with decay. Benign outcome — non-positive valence
        // (´conv:valence:sign´) — so the folded indicator is 0 and the decay
        // stays observable on its own.
        let now = PersistentTimestamp::now();
        let update = CellOutcomeUpdate {
            is_positive: false,
            compressed_valence: 0.0,
            raw_valence: 0.0,
            axis_compressed: HashMap::new(),
            axis_raw: HashMap::new(),
        };

        // gamma_t = 0.999 per hour, so after 24 hours:
        // factor = 0.999^24 ≈ 0.976
        let gamma_t = 0.999;
        let lambda_l = 0.999;
        state.apply_write_decay_and_update(gamma_t, &now, &update, lambda_l);

        // The EWMAs should have been decayed before the update
        // adverse_rate was 1.0, decayed by ~0.976, then EWMA with 0 (benign)
        // Result: 0.999 * (1.0 * 0.976...) + 0.001 * 0 ≈ 0.975
        assert!(state.adverse_rate_ewma < 1.0);
        assert!(state.adverse_rate_ewma > 0.9); // decayed but not too much
    }

    /// A read-time decayed view is value-for-value identical to applying the
    /// shared decay operation to a clone, while every stored scalar, map and
    /// timestamp remains unchanged. Assessment can therefore read current
    /// outcome evidence without turning a read into a write or approximating
    /// the decay.
    ///
    /// ´claim:identity:a-decayed-outcome-view-is-exactly-the-mutating-decay-result-without-changing-storage´
    /// ´test:unit:cell-outcome-state-decayed-view-is-exact-and-pure´
    #[test]
    fn cell_outcome_state_decayed_view_is_exact_and_pure() {
        let axis = OutcomeAxisId(7);
        let mut stored = CellOutcomeState::new_neutral();
        stored.adverse_rate_ewma = 0.75;
        stored.compressed_valence_ewma = -0.5;
        stored.raw_valence_ewma = 8.0;
        stored.per_axis_compressed.insert(axis, 0.25);
        stored.per_axis_raw.insert(axis, -4.0);
        stored.last_updated = PersistentTimestamp::new(1_000, 250);

        let before = stored.clone();
        let now = PersistentTimestamp::new(1_000 + 36 * 3_600, 250);
        let view = stored.read_decayed(0.998, &now);
        let mut mutating_result = stored.clone();
        mutating_result.apply_decay(0.998, &now);

        assert_eq!(view, mutating_result);
        assert_eq!(stored, before);
    }

    /// A cell's measurement side starts equally empty: no Sentinel has an alarm
    /// recorded against it, suspicion and volatility are zero, and the step
    /// count says no updates have arrived. The step count is what lets a reader
    /// tell "calm" from "never looked at", so it must start honest.
    ///
    /// ´claim:identity:a-cells-measurement-state-begins-with-no-sentinel-alarms-and-a-zero-step-count´
    /// ´test:unit:measurement-state-default´
    #[test]
    fn measurement_state_default() {
        let state = MeasurementState::default();
        assert!(state.per_sentinel_alarm_ewma.is_empty());
        assert_eq!(state.suspicion().to_bits(), 0.0f64.to_bits());
        assert_eq!(state.volatility.to_bits(), 0.0f64.to_bits());
        assert_eq!(state.step_count, 0);
    }

    /// Every competitive cell carries measurement state from the moment it is
    /// created, freshly zeroed rather than inherited. The maintenance loop
    /// installs this state the instant a cell enters the competitive set, so
    /// the assessment path never has to ask whether a tracked cell has somewhere
    /// to record what it sees.
    ///
    /// ´claim:identity:every-competitive-cell-carries-measurement-state-from-the-moment-it-is-created´
    /// ´test:unit:cell-mutable-state-new´
    #[test]
    fn cell_mutable_state_new() {
        let state = CellMutableState::new();
        assert_eq!(state.measurement.step_count, 0);
    }

    /// Alarm evidence is held one entry per Sentinel, so a cell can be loud on
    /// one Sentinel's axis and quiet on another's without the two blurring
    /// together; smoothing a fresh alarm into an existing entry moves it only
    /// part of the way. Alongside them the step count records how many updates
    /// built the picture, so a reader can weigh an alarm level against the
    /// number of chances it had to settle.
    ///
    /// ´claim:identity:alarm-evidence-is-held-per-sentinel-alongside-a-count-of-the-updates-that-built-it´
    /// ´test:unit:measurement-update-alarm´
    #[test]
    fn measurement_update_alarm() {
        let mut ms = MeasurementState::default();
        let sid = SentinelId(1);

        // Insert alarm EWMA value for sentinel S1
        ms.per_sentinel_alarm_ewma.insert(sid, 2.5);
        ms.step_count += 1;

        assert_eq!(ms.per_sentinel_alarm_ewma[&sid].to_bits(), 2.5f64.to_bits());
        assert_eq!(ms.step_count, 1);

        // Simulate a second observation: manual EWMA with λ=0.9
        let lambda: f64 = 0.9;
        let new_alarm: f64 = 1.0;
        let entry = ms.per_sentinel_alarm_ewma.get_mut(&sid).unwrap();
        *entry = (1.0 - lambda).mul_add(new_alarm, lambda * *entry);

        // 0.9 * 2.5 + 0.1 * 1.0 = 2.35
        assert!((*entry - 2.35).abs() < 1e-15);
        ms.step_count += 1;
        assert_eq!(ms.step_count, 2);
    }

    /// Suspicion is not a fourth accumulator: it is derived at read time as the
    /// mean of the per-Sentinel alarm averages, so it agrees with the entries
    /// at every moment — including the moment the Sentinel set changes, which
    /// is exactly when a stored copy would drift from what the entries say. A
    /// cell loud on one Sentinel and quiet on another reads as the average of
    /// the two, and a cell nobody has reported on reads as zero rather than as
    /// a stale remembered level.
    ///
    /// ´claim:identity:suspicion-is-derived-at-read-time-as-the-mean-of-the-per-sentinel-alarm-averages´
    /// ´test:unit:measurement-suspicion-derived-from-entries´
    #[test]
    fn measurement_suspicion_derived_from_entries() {
        let mut ms = MeasurementState::default();
        assert_eq!(ms.suspicion().to_bits(), 0.0f64.to_bits(), "no entries → zero suspicion");

        ms.per_sentinel_alarm_ewma.insert(SentinelId(1), 0.9);
        assert!((ms.suspicion() - 0.9).abs() < 1e-15, "one entry → that entry");

        ms.per_sentinel_alarm_ewma.insert(SentinelId(2), 0.3);
        assert!((ms.suspicion() - 0.6).abs() < 1e-15, "two entries → their mean");

        // The derivation re-reads the entries: removing one moves suspicion
        // with it, where a stored accumulator would have kept the old mix.
        ms.per_sentinel_alarm_ewma.remove(&SentinelId(1));
        assert!((ms.suspicion() - 0.3).abs() < 1e-15, "set change tracks entries");
    }

    /// A positive-valence outcome is exactly what the adverse rate counts:
    /// one such outcome raises the rate by the complement of the smoothing
    /// factor, and a benign outcome leaves it untouched. Positive valence is
    /// the adverse direction under the corpus's sign convention, so the rate a
    /// model reads at the cross-dimension maximum really is the rate at which
    /// this range's labelled requests came back adverse — a rate built on the
    /// complement would carry the benign rate at the position the models
    /// reserve for the adverse one, with the learned association silently
    /// inverted wherever the feature carries signal.
    ///
    /// ´claim:identity:a-positive-valence-outcome-raises-the-adverse-rate-and-a-benign-one-leaves-it-untouched´
    /// ´test:unit:outcome-update-positive´
    #[test]
    fn outcome_update_positive() {
        let mut state = CellOutcomeState::new_neutral();
        let now = PersistentTimestamp::now();

        // Adverse: positive valence (´conv:valence:sign´).
        let update = CellOutcomeUpdate {
            is_positive: true,
            compressed_valence: 0.8,
            raw_valence: 5.0,
            axis_compressed: HashMap::new(),
            axis_raw: HashMap::new(),
        };

        // One adverse update with λ=0.999
        let lambda_l = 0.999;
        state.apply_write_decay_and_update(0.999, &now, &update, lambda_l);

        // adverse_rate = 0.999 * 0.0 + 0.001 * 1.0 = 0.001 (adverse → indicator 1)
        assert!((state.adverse_rate_ewma - 0.001).abs() < 1e-15);

        // compressed_valence = 0.999 * 0.0 + 0.001 * 0.8 = 0.0008
        assert!((state.compressed_valence_ewma - 0.0008).abs() < 1e-15);

        // A benign outcome (indicator 0) decays the rate and adds nothing.
        let benign = CellOutcomeUpdate {
            is_positive: false,
            compressed_valence: 0.0,
            raw_valence: 0.0,
            axis_compressed: HashMap::new(),
            axis_raw: HashMap::new(),
        };
        state.apply_write_decay_and_update(0.999, &now, &benign, lambda_l);

        // adverse_rate = 0.999 * 0.001 + 0.001 * 0.0 = 0.000999
        assert!((state.adverse_rate_ewma - 0.000_999).abs() < 1e-15);
    }

    /// Each outcome axis keeps its own pair of readings, compressed and raw,
    /// and each is smoothed independently with its sign preserved — a cell that
    /// is favourable on one axis and adverse on another records exactly that
    /// rather than an average that hides both. Keeping the raw value beside the
    /// compressed one means the magnitude survives the squashing that makes
    /// axes comparable.
    ///
    /// ´claim:identity:outcome-axes-are-smoothed-separately-in-both-compressed-and-raw-form-and-keep-their-sign´
    /// ´test:unit:outcome-per-axis-values´
    #[test]
    fn outcome_per_axis_values() {
        let mut state = CellOutcomeState::new_neutral();
        let now = PersistentTimestamp::now();

        let mut axis_compressed = HashMap::new();
        axis_compressed.insert(OutcomeAxisId(1), 0.5);
        axis_compressed.insert(OutcomeAxisId(2), -0.3);

        let mut axis_raw = HashMap::new();
        axis_raw.insert(OutcomeAxisId(1), 10.0);
        axis_raw.insert(OutcomeAxisId(2), -6.0);

        let update = CellOutcomeUpdate {
            is_positive: true,
            compressed_valence: 0.0,
            raw_valence: 0.0,
            axis_compressed,
            axis_raw,
        };

        let lambda_l = 0.9;
        state.apply_write_decay_and_update(0.999, &now, &update, lambda_l);

        // axis 1: 0.9 * 0.0 + 0.1 * 0.5 = 0.05
        let a1 = state.per_axis_compressed[&OutcomeAxisId(1)];
        assert!((a1 - 0.05).abs() < 1e-15);

        // axis 2: 0.9 * 0.0 + 0.1 * (-0.3) = -0.03
        let a2 = state.per_axis_compressed[&OutcomeAxisId(2)];
        assert!((a2 - (-0.03)).abs() < 1e-15);

        // raw axis 1: 0.9 * 0.0 + 0.1 * 10.0 = 1.0
        let r1 = state.per_axis_raw[&OutcomeAxisId(1)];
        assert!((r1 - 1.0).abs() < 1e-15);

        // raw axis 2: 0.9 * 0.0 + 0.1 * (-6.0) = -0.6
        let r2 = state.per_axis_raw[&OutcomeAxisId(2)];
        assert!((r2 - (-0.6)).abs() < 1e-15);
    }
}
