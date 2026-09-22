// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`empty_chain_returns_defaults`] | extract | A Sentinel with no chain still yields a summary, and every measured indicator in it reads zero — peaks, coordination peak and composite alike — while the caller's own flags survive into the result. An operations dashboard therefore shows a quiet Sentinel rather than a gap, and the two facts the caller already knew are not lost just because there was nothing to measure. |
//! | [`peak_z_from_cell`] | extract | The summary's peaks are drawn from the deepest entry alone, taking the largest z-score and the largest accumulated drift across its four axes, and the reported depth and maturity come from that same entry. An operator reading the summary is reading conditions where the observation actually landed, not an average smeared over its whole ancestry. |
//! | [`composite_from_z`] | extract | The composite puts the two peaks onto a common scale — each divided by its own saturation constant — and reports whichever is larger. A z-score of four out of five therefore beats a drift of half out of two, so a single loud batch can raise the composite on its own. The two kinds of evidence are not summed: one alarm is one alarm however many ways it shows up. |
//! | [`composite_from_cusum`] | extract | cites (´claim:extract:the-composite-is-the-larger-of-the-z-and-drift-peaks-each-scaled-by-its-own-constant´) |
//! | [`composite_saturates_at_1`] | extract | However far a peak runs past its saturation constant the composite stops at one. Beyond the constant the reading is already as alarming as the summary can express, and an unbounded figure would let one extreme Sentinel dominate any dashboard ranking or threshold that compares composites across the fleet. |
//! | [`peak_coord_z`] | extract | The coordination peak ranges over every coordination context and every axis within them, reporting the loudest, and it is kept in its own field rather than merged into the cell peak. Alarm confined to one cell and alarm shared across a whole analysis set mean very different things, so the summary reports them side by side instead of collapsing the distinction. |
//! | [`flags_passthrough`] | extract | The hierarchical-alarm and ledger-immaturity flags are relayed exactly as the caller stated them, on a populated chain as much as on an empty one; nothing in the entries can set or clear them. Both facts are decided elsewhere — by the hierarchical alarm machinery and by the ledger — so the summary carries them for the reader's convenience without claiming the authority to second-guess either. |

//! Sentinel alarm summary computation.
//!
//! Computes a diagnostic summary of alarm indicators from extracted features.
//!
//! # Fields
//!
//! | Field | Description |
//! |-------|-------------|
//! | `peak_z` | Maximum z-score across cell-level scores and axes |
//! | `peak_cusum` | Maximum CUSUM across cell-level scores and axes |
//! | `peak_coord_z` | Maximum z-score from coordination contexts |
//! | `composite` | Max of normalised `peak_z` and `peak_cusum` |
//! | `chain_depth` | Depth of the deepest cell in the chain |
//! | `maturity` | Cell maturity (`1 - noise_influence`) |
//! | `hierarchical_asserted` | Whether hierarchical alarm is asserted |
//! | `ledger_immature` | Whether the covering ledger entry is immature |
//!
//! # Cross-References
//!
//! - (´def:runtime:alarm-summary´) — the per-Sentinel alarm summary this
//!   module computes, and the composite scale its two peaks are divided onto

use crate::report::{CoordinationEntry, ReportCellEntry};

// ═══════════════════════════════════════════════════════════════════════════════
// Alarm Summary Type
// ═══════════════════════════════════════════════════════════════════════════════

/// Diagnostic alarm summary for a single Sentinel.
///
/// Provides a concise view of alarm indicators for observability dashboards
/// and operational alerts.
#[derive(Clone, Debug, Default)]
pub struct SentinelAlarmSummary {
    /// Maximum z-score across cell-level scores and axes.
    pub peak_z: f64,
    /// Maximum CUSUM across cell-level scores and axes.
    pub peak_cusum: f64,
    /// Maximum z-score from coordination contexts.
    pub peak_coord_z: f64,
    /// Composite alarm indicator: max of normalised `peak_z` and `peak_cusum`.
    pub composite: f64,
    /// Depth of the deepest cell in the chain.
    pub chain_depth: usize,
    /// Cell maturity (`1 - noise_influence`).
    pub maturity: f64,
    /// Whether hierarchical alarm is asserted.
    pub hierarchical_asserted: bool,
    /// Whether the Ledger entry covering the request is immature: it holds
    /// fewer eligible labels than the materiality threshold, or its arrival
    /// window implies a steady-state attenuation below the configured floor
    /// (´def:runtime:alarm-summary´), (´thm:ledger:materiality´).
    pub ledger_immature: bool,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Alarm Summary Computation
// ═══════════════════════════════════════════════════════════════════════════════

/// Normalisation constant for z-scores in composite computation.
///
/// Z-scores above this value are considered saturated at 1.0. The composite
/// puts the peak z-score and the peak accumulator onto a common scale by
/// dividing each by its own saturation constant
/// (´def:runtime:alarm-summary´), which fixes that the division happens and
/// not the divisor this constant supplies. The divisor is fixed at
/// (´dec:health:alarm-composite-scale´): under a normal reference an
/// observation at or beyond five standard deviations lies in a two-sided
/// tail of about six in ten million, which is as alarming as this
/// composite has any need to distinguish.
///
/// ´const:assayer:alarm-z-saturation´ (´alg:const:scalar´)
/// ´const:assayer:alarm-z-saturation-scalar-5p0´
const Z_NORM: f64 = 5.0;

/// Normalisation constant for CUSUMs in composite computation.
///
/// CUSUMs above this value are considered saturated at 1.0. It is the
/// accumulator's half of the common scale the composite divides onto
/// (´def:runtime:alarm-summary´), which fixes that the division happens and
/// not the divisor this constant supplies. It is a shipped operating
/// point fixed at (´dec:health:alarm-composite-scale´), and what fixes it
/// is its ratio to the z-score half rather than its own magnitude: two
/// standard deviations of accumulated departure is a sustained drift
/// where a z of two is one unremarkable observation, so the pair makes
/// sustained drift saturate the composite sooner than instantaneous
/// extremity does.
///
/// ´const:assayer:alarm-accumulator-saturation´ (´alg:const:scalar´)
/// ´const:assayer:alarm-accumulator-saturation-scalar-2p0´
const CUSUM_NORM: f64 = 2.0;

/// Computes the alarm summary for a Sentinel.
///
/// # Arguments
///
/// * `chain` — Ancestor chain entries, deepest first.
/// * `coordination` — Coordination context entries.
/// * `hierarchical_asserted` — Whether hierarchical alarm is asserted.
/// * `ledger_immature` — Whether the covering ledger entry is immature.
///
/// # Returns
///
/// A `SentinelAlarmSummary` with computed alarm indicators.
///
/// # Cross-References
///
/// - (´def:runtime:alarm-summary´) — the summary this function computes
#[must_use]
pub fn compute_alarm_summary(
    chain: &[(u8, &ReportCellEntry)],
    coordination: &[CoordinationEntry],
    hierarchical_asserted: bool,
    ledger_immature: bool,
) -> SentinelAlarmSummary {
    if chain.is_empty() {
        return SentinelAlarmSummary {
            hierarchical_asserted,
            ledger_immature,
            ..Default::default()
        };
    }

    // Get the cell (deepest entry)
    let (cell_depth, cell) = chain[0];

    // peak_z: max across cell-level z-scores and axes
    let peak_z = cell.scores.max_z_per_axis().into_iter().fold(f64::NEG_INFINITY, f64::max);
    let peak_z = if peak_z.is_finite() { peak_z } else { 0.0 };

    // peak_cusum: max across cell-level CUSUMs and axes
    let peak_cusum = cell.scores.cusum_per_axis().into_iter().fold(f64::NEG_INFINITY, f64::max);
    let peak_cusum = if peak_cusum.is_finite() { peak_cusum } else { 0.0 };

    // peak_coord_z: max z-score from coordination contexts
    let peak_coord_z = coordination
        .iter()
        .flat_map(|e| e.scores.max_z_per_axis())
        .fold(f64::NEG_INFINITY, f64::max);
    let peak_coord_z = if peak_coord_z.is_finite() { peak_coord_z } else { 0.0 };

    // composite: max of normalised peak_z and peak_cusum
    let norm_z = (peak_z / Z_NORM).clamp(0.0, 1.0);
    let norm_cusum = (peak_cusum / CUSUM_NORM).clamp(0.0, 1.0);
    let composite = norm_z.max(norm_cusum);

    // maturity: 1 - noise_influence for cell
    let maturity = 1.0 - cell.noise_influence;

    SentinelAlarmSummary {
        peak_z,
        peak_cusum,
        peak_coord_z,
        composite,
        chain_depth: usize::from(cell_depth),
        maturity,
        hierarchical_asserted,
        ledger_immature,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {

    use super::*;
    use crate::report::{AxisScoreSet, AxisScoreSnapshot, CoordinationEntry, ReportCellEntry};

    fn make_entry(depth: u8, n_z: f64, cusum: f64, noise_influence: f64) -> ReportCellEntry {
        ReportCellEntry {
            depth,
            sample_count: 100,
            is_competitive: true,
            rank: 1,
            cap: 10,
            energy_ratio: 1.0,
            noise_influence,
            scores: AxisScoreSet {
                novelty: AxisScoreSnapshot {
                    max_z: n_z,
                    cusum,
                    ..Default::default()
                },
                displacement: AxisScoreSnapshot::default(),
                surprise: AxisScoreSnapshot::default(),
                coherence: AxisScoreSnapshot::default(),
            },
            degraded: false,
        }
    }

    fn make_coord_entry(n_z: f64) -> CoordinationEntry {
        CoordinationEntry {
            depth: 0,
            cells_reporting: 5,
            scores: AxisScoreSet {
                novelty: AxisScoreSnapshot {
                    max_z: n_z,
                    ..Default::default()
                },
                displacement: AxisScoreSnapshot::default(),
                surprise: AxisScoreSnapshot::default(),
                coherence: AxisScoreSnapshot::default(),
            },
        }
    }

    /// A Sentinel with no chain still yields a summary, and every measured
    /// indicator in it reads zero — peaks, coordination peak and composite
    /// alike — while the caller's own flags survive into the result. An
    /// operations dashboard therefore shows a quiet Sentinel rather than a gap,
    /// and the two facts the caller already knew are not lost just because
    /// there was nothing to measure.
    ///
    /// ´claim:extract:a-sentinel-with-no-chain-summarises-as-quiet-rather-than-as-a-missing-summary´
    /// ´test:unit:empty-chain-returns-defaults´
    #[test]
    fn empty_chain_returns_defaults() {
        let chain: Vec<(u8, &ReportCellEntry)> = vec![];
        let coord: Vec<CoordinationEntry> = vec![];
        let summary = compute_alarm_summary(&chain, &coord, true, true);

        assert_eq!(summary.peak_z.to_bits(), 0.0f64.to_bits());
        assert_eq!(summary.peak_cusum.to_bits(), 0.0f64.to_bits());
        assert_eq!(summary.peak_coord_z.to_bits(), 0.0f64.to_bits());
        assert_eq!(summary.composite.to_bits(), 0.0f64.to_bits());
        assert!(summary.hierarchical_asserted);
        assert!(summary.ledger_immature);
    }

    /// The summary's peaks are drawn from the deepest entry alone, taking the
    /// largest z-score and the largest accumulated drift across its four axes,
    /// and the reported depth and maturity come from that same entry. An
    /// operator reading the summary is reading conditions where the observation
    /// actually landed, not an average smeared over its whole ancestry.
    ///
    /// ´claim:extract:the-alarm-peaks-depth-and-maturity-all-describe-the-deepest-cell´
    /// ´test:unit:peak-z-from-cell´
    #[test]
    fn peak_z_from_cell() {
        let entry = make_entry(8, 3.0, 0.5, 0.2);
        let chain = vec![(8, &entry)];
        let coord: Vec<CoordinationEntry> = vec![];
        let summary = compute_alarm_summary(&chain, &coord, false, false);

        assert!((summary.peak_z - 3.0).abs() < 1e-10);
        assert!((summary.peak_cusum - 0.5).abs() < 1e-10);
        assert_eq!(summary.chain_depth, 8);
        assert!((summary.maturity - 0.8).abs() < 1e-10); // 1 - 0.2
    }

    /// The composite puts the two peaks onto a common scale — each divided by
    /// its own saturation constant — and reports whichever is larger. A z-score
    /// of four out of five therefore beats a drift of half out of two, so a
    /// single loud batch can raise the composite on its own. The two kinds of
    /// evidence are not summed: one alarm is one alarm however many ways it
    /// shows up.
    ///
    /// ´claim:extract:the-composite-is-the-larger-of-the-z-and-drift-peaks-each-scaled-by-its-own-constant´
    /// ´test:unit:composite-from-z´
    #[test]
    fn composite_from_z() {
        // peak_z = 4.0, peak_cusum = 0.5
        // norm_z = 4.0 / 5.0 = 0.8
        // norm_cusum = 0.5 / 2.0 = 0.25
        // composite = max(0.8, 0.25) = 0.8
        let entry = make_entry(8, 4.0, 0.5, 0.0);
        let chain = vec![(8, &entry)];
        let summary = compute_alarm_summary(&chain, &[], false, false);

        assert!((summary.composite - 0.8).abs() < 1e-10);
    }

    /// Accumulated drift can carry the composite on its own: with a modest
    /// z-score and drift well past half of its saturation constant, the drift
    /// side is what the composite reports. Because the two constants differ,
    /// the comparison is between fractions of alarm rather than between raw
    /// magnitudes, and a persistent shift is not shouted down by whichever
    /// quantity happens to live on the larger scale.
    ///
    /// (´claim:extract:the-composite-is-the-larger-of-the-z-and-drift-peaks-each-scaled-by-its-own-constant´)
    /// ´test:unit:composite-from-cusum´
    #[test]
    fn composite_from_cusum() {
        // peak_z = 1.0, peak_cusum = 1.5
        // norm_z = 1.0 / 5.0 = 0.2
        // norm_cusum = 1.5 / 2.0 = 0.75
        // composite = max(0.2, 0.75) = 0.75
        let entry = make_entry(8, 1.0, 1.5, 0.0);
        let chain = vec![(8, &entry)];
        let summary = compute_alarm_summary(&chain, &[], false, false);

        assert!((summary.composite - 0.75).abs() < 1e-10);
    }

    /// However far a peak runs past its saturation constant the composite stops
    /// at one. Beyond the constant the reading is already as alarming as the
    /// summary can express, and an unbounded figure would let one extreme
    /// Sentinel dominate any dashboard ranking or threshold that compares
    /// composites across the fleet.
    ///
    /// ´claim:extract:the-composite-saturates-at-one-however-far-a-peak-runs-past-its-constant´
    /// ´test:unit:composite-saturates-at-1´
    #[test]
    fn composite_saturates_at_1() {
        // peak_z = 10.0 → norm_z = 2.0 → clamped to 1.0
        let entry = make_entry(8, 10.0, 0.0, 0.0);
        let chain = vec![(8, &entry)];
        let summary = compute_alarm_summary(&chain, &[], false, false);

        assert!((summary.composite - 1.0).abs() < 1e-10);
    }

    /// The coordination peak ranges over every coordination context and every
    /// axis within them, reporting the loudest, and it is kept in its own field
    /// rather than merged into the cell peak. Alarm confined to one cell and
    /// alarm shared across a whole analysis set mean very different things, so
    /// the summary reports them side by side instead of collapsing the
    /// distinction.
    ///
    /// ´claim:extract:the-coordination-peak-is-the-loudest-z-over-every-context-and-is-reported-separately´
    /// ´test:unit:peak-coord-z´
    #[test]
    fn peak_coord_z() {
        let entry = make_entry(8, 1.0, 0.0, 0.0);
        let chain = vec![(8, &entry)];
        let coord = vec![make_coord_entry(2.5), make_coord_entry(3.5)];
        let summary = compute_alarm_summary(&chain, &coord, false, false);

        assert!((summary.peak_coord_z - 3.5).abs() < 1e-10);
    }

    /// The hierarchical-alarm and ledger-immaturity flags are relayed exactly as
    /// the caller stated them, on a populated chain as much as on an empty one;
    /// nothing in the entries can set or clear them. Both facts are decided
    /// elsewhere — by the hierarchical alarm machinery and by the ledger — so
    /// the summary carries them for the reader's convenience without claiming
    /// the authority to second-guess either.
    ///
    /// ´claim:extract:the-hierarchical-and-immaturity-flags-are-relayed-unchanged-rather-than-inferred´
    /// ´test:unit:flags-passthrough´
    #[test]
    fn flags_passthrough() {
        let entry = make_entry(8, 1.0, 0.0, 0.5);
        let chain = vec![(8, &entry)];
        let summary = compute_alarm_summary(&chain, &[], true, true);

        assert!(summary.hierarchical_asserted);
        assert!(summary.ledger_immature);
    }
}
