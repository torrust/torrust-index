// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`chain_single_root`] | extract | The root cell covers the whole coordinate space, so a report carrying nothing but the root still yields a chain — of exactly one entry, at depth zero — for any coordinate whatsoever. Every request can therefore be assessed against whatever the Sentinel does know, however coarse, and a young Sentinel that has not yet split is not a Sentinel that answers nothing. |
//! | [`chain_three_depths`] | extract | The walk returns the containing cells ordered deepest first, with the root last. Every view downstream is defined against that ordering — the cell view reads the front, the root view the back — so the order is part of the contract rather than an accident of how the index was iterated. |
//! | [`chain_gap_in_depths`] | extract | The chain is built from the cells the report actually carried, so a depth the report skipped is simply absent and the walk continues past it to the shallower levels. Sentinels report the cells they hold rather than a filled-in ladder of every level, and extraction must not stall at the first level it cannot find. |
//! | [`chain_many_depths`] | extract | cites (´claim:extract:the-ancestor-chain-is-ordered-deepest-first-with-the-root-last´) |
//! | [`chain_coord_in_d4_not_d8`] | extract | Membership is decided by containment and not by depth: a coordinate inside a reported depth-four cell but outside the depth-eight cell nested within it collects the shallower cell and skips the deeper one. The chain is the coordinate's own ancestry, so a sibling region's conditions never leak into a row about somewhere else. |
//! | [`chain_root_only_fallback`] | extract | cites (´claim:extract:a-cell-that-does-not-contain-the-coordinate-is-left-out-however-deep-it-was-reported´) |
//! | [`chain_boundary_lo`] | extract | A cell claims its own lower bound: a coordinate sitting exactly at the start of a reported cell is inside it. Cells tile the space without overlap, so each boundary must belong to exactly one side, and here it belongs to the cell that begins there. |
//! | [`chain_boundary_hi`] | extract | cites (´claim:extract:a-cell-spans-its-lower-bound-through-its-final-position-and-stops-before-the-next´) |
//! | [`chain_past_boundary`] | extract | cites (´claim:extract:a-cell-spans-its-lower-bound-through-its-final-position-and-stops-before-the-next´) |
//! | [`chain_contains_only_reported_cells`] | extract | cites (´claim:extract:the-chain-holds-only-cells-the-report-carried-so-an-unreported-depth-is-skipped´) |
//! | [`chain_empty_index`] | extract | An index that holds no report yields an empty chain and is answered without any walk at all — the absence is detected before the descent begins. A Sentinel that has never reported is a common state on a cold start, and it costs nothing rather than a fruitless sweep of every depth. |
//! | [`extract_occupancy_no_report`] | extract | A Sentinel that has never reported still produces a row of the full width, every slot zero, flagged as unoccupied, and an alarm summary that reads quiet. The model consumes rows in fixed positions, so silence has to be expressed as a well-formed row rather than as a missing one, and the flag is what stops those zeros being read as measured calm. |
//! | [`extract_occupancy_reporting`] | extract | A Sentinel holding even a single root-level cell is flagged occupied. The flag turns on the moment there is any real evidence to draw on, so the model can weigh a coarse but genuine reading differently from the absence of a reading, which the zeros alone could not tell it. |
//! | [`extract_feature_count`] | extract | The row is sixty slots that never move — chain z-scores, chain drift, structure, coordination, the three base ledger quantities and batch context — plus exactly two more for each spatial outcome axis the deployment tracks. Only the ledger tail varies, and it varies at the end, so a deployment that adds an outcome axis extends the row rather than renumbering the sixty slots a trained model already depends on. |
//! | [`extract_coord_not_in_report`] | extract | A coordinate far outside anything the Sentinel has split still extracts a complete, occupied row by resting on the root alone. Assessment is never refused for want of local detail: the answer degrades in precision as the evidence thins rather than disappearing at some coverage boundary. |
//! | [`extract_f32_precision`] | extract | Storing the row at single precision costs less than a thousandth in relative terms across values spanning ten orders of magnitude, and the error near zero stays far below the scale at which any feature is read. Rows are held in bulk, so halving their size is worth an error this far beneath the noise the features themselves carry. |
//! | [`extract_f32_nan_inf_propagation`] | extract | Non-finite values cross the downcast intact — a NaN stays a NaN, and each infinity keeps its sign — while ordinary values are untouched. The downcast is a change of storage and not a place where corruption is quietly laundered into a plausible-looking number, so a defect upstream stays detectable in the stored row. |
//! | [`extract_golden_report_full`] | extract | End to end from a hand-built three-level report, the blocks land in one fixed order: z-scores first, then chain drift from slot twenty-four, on through structure, coordination and the ledger, with batch context — the log of the containing cell's sample count — occupying the final slot. The alarm summary comes back alongside and agrees with the row's own peak. Absolute indices are the model's only names for its inputs, so the concatenation order is pinned against known values rather than left to the order the steps happen to run in. |
//! | [`property_no_nan_in_extraction`] | extract | Across chains of one, two, three and five entries, every z-score, drift and structure slot comes out finite. The degenerate lengths are where the divisions and folds could go wrong — a mean over one entry, a spread with no deviation to take — and a single non-finite slot would spread through standardisation into every feature the model reads. |
//! | [`property_gradient_is_cell_minus_root`] | extract | The gradient slot always equals its own row's cell slot minus its root slot, on every axis and at every chain length. The three are not computed from independent readings of the chain, so a model that learns on any two of them is reading the third consistently rather than picking up a discrepancy between separate traversals. |
//! | [`property_max_z_geq_cell_z`] | extract | The maximum view never falls below the cell view on any axis, whatever the chain's length. The deepest entry is one of the entries the maximum ranges over, so this is the ordering the layout promises: the maximum is an upper bound on the endpoint views and not a competing reading of the chain. |
//! | [`property_max_z_geq_root_z`] | extract | cites (´claim:extract:the-maximum-view-dominates-both-the-cell-and-root-views-on-every-axis´) |
//! | [`property_spread_nonnegative`] | extract | The spread view is never negative on any axis or chain length, including the single-entry case where it is exactly zero. It is a standard deviation, so its sign carries no information and a negative value could only mean the variance computation had gone wrong beneath it. |
//! | [`property_feature_count_matches_config`] | extract | cites (´claim:extract:the-row-is-sixty-fixed-slots-plus-two-more-for-each-spatial-outcome-axis´) |
//! | [`ledger_read_takes_the_deepest_covering_entry`] | extract | Where a region has been split, the outcome features come from the deepest tracked cell containing the coordinate rather than from the root above it: a request landing inside a cell with its own adverse history carries that history into the row, not the diluted average of the whole space. Depth is specificity, and the one thing the Ledger can tell a model that no current measurement can — that this particular neighbourhood turned out badly — survives only if the read reaches the neighbourhood's own entry. |
//! | [`ledger_read_falls_back_to_the_root`] | extract | A coordinate no tracked cell covers still reads: the walk falls back through the depths to the root, which contains every coordinate, and the row carries the root's history. The Ledger read cannot fail, so a request outside every split region is answered from coarser memory rather than refused or given a neutral placeholder that would read as measured innocence. |
//! | [`immaturity_raises_for_a_thin_deep_cell`] | extract | A deep cell holding fewer eligible labels than the materiality threshold raises the immaturity flag on its own account, while the root above it — holding plenty — does not. Lowering only the threshold, with the same evidence and the same floor, puts the flag down again, so what raised it was the count and not the arrival rate. A host discounting an immature reading needs the flag to be about the cell that answered the request, and a cell with too few labels has measured nothing to weight. |
//! | [`immaturity_raises_for_an_attenuation_starved_cell`] | extract | A deep cell holding enough eligible labels still raises the flag when its labels arrive too sparsely: a window fed once a year has decayed almost to a single arrival by the time the next comes, and the steady-state attenuation that window implies — about a tenth — stands below the configured floor. Lowering only the floor beneath that attenuation puts the flag down, so what raised it was the arrival rate and not the count. The root above it, holding a dense window under that same floor, stays mature, so the raise belongs to the cell that answered rather than to the Ledger. The two conditions are independent, and a cell whose true rate is high but whose evidence arrives too slowly to hold the excess in an average is exactly the case an average alone cannot report. |
//! | [`immaturity_stays_down_for_a_mature_deep_cell`] | extract | A deep cell clearing both conditions leaves the flag down even where the root above it would raise it: the request lands in a cell with labels enough and arrivals dense enough, and the thin root it is nested in has no say. The flag is a statement about the entry the read actually reached, so a mature neighbourhood inside a young Sentinel is not discounted for the Sentinel's youth. |

//! Per-Sentinel feature extraction from reports.
//!
//! This module extracts features from Sentinel reports for use in the Bayesian
//! model. Features are extracted from:
//!
//! - The ancestor chain of cells containing a coordinate
//! - Coordination contexts across analysis sets
//! - Ledger outcome history
//!
//! # Module Structure
//!
//! | Module | Purpose | Feature Count |
//! |--------|---------|---------------|
//! | [`chain`] | Chain z-score views | 24 |
//! | [`cusum`] | Chain CUSUM views | 12 |
//! | [`structure`] | Chain structural features | 8 |
//! | [`coordination`] | Coordination context features | 12 |
//! | [`ledger_features`] | Ledger outcome features | variable |
//! | [`alarm`] | Alarm summary computation | — |
//!
//! # Cross-References
//!
//! - (´chap:spec:structured-extraction´) — the per-Sentinel structured
//!   extraction this module is the implementation of
//! - (´dec:ordering:synchronous-reception´) — the reception that puts a batch
//!   report in the index this module reads

// Extraction functions are used in the assessment path. Allow dead code
// warnings until fully wired into the runtime.

pub mod alarm;
pub mod chain;
pub mod coordination;
pub mod cusum;
pub mod ledger_features;
pub mod structure;

pub use alarm::SentinelAlarmSummary;

use crate::ledger::{DecayedView, ImmaturityCriteria, LedgerEntry, SentinelLedger, routing};
use crate::pending::SentinelExtraction;
use crate::report::{ReportCellEntry, ReportIndex};
use crate::types::{LedgerKey, OutcomeAxisId, PersistentTimestamp, SCORING_AXIS_COUNT};

// ═══════════════════════════════════════════════════════════════════════════════
// Ancestor Chain Collection
// ═══════════════════════════════════════════════════════════════════════════════

/// Collects the ancestor chain of cells containing a coordinate.
///
/// Performs a depth-walk from the deepest cell containing `coord` up to the root.
/// Returns entries in order from deepest to shallowest (root last).
///
/// # Arguments
///
/// * `index` — The report index to search
/// * `coord` — The coordinate to find ancestors for
///
/// # Returns
///
/// A vector of `(depth, &ReportCellEntry)` pairs, deepest first. Returns an
/// empty vector if the index has no report.
///
/// # Cross-References
///
/// - (´alg:runtime:extraction-routing´) — the lookup this walk serves, which
///   takes the deepest reported cell covering the coordinate
#[must_use]
pub fn collect_ancestor_chain(index: &ReportIndex, coord: u128) -> Vec<(u8, &ReportCellEntry)> {
    if !index.has_report() {
        return Vec::new();
    }

    let mut chain = Vec::with_capacity(8); // Typical depth, avoids reallocs
    let max_depth = index.max_depth();

    // Walk from max_depth down to 0, collecting hits
    for depth in (0..=max_depth).rev() {
        let key = LedgerKey::from_coordinate(coord, depth);
        if let Some(entry) = index.cells().get(&key) {
            chain.push((depth, entry));
        }
    }

    chain
}

// ═══════════════════════════════════════════════════════════════════════════════
// f32 Downcast
// ═══════════════════════════════════════════════════════════════════════════════

/// Converts f64 features to f32 for memory-efficient storage.
///
/// NaN and Inf values are preserved. Values in the typical model range
/// `[-10, 10]` have quantisation error < 10⁻³.
///
/// # Cross-References
///
/// - (´rem:extraction:single-precision´) — why the extraction is stored in
///   single precision while the models accumulate in double
#[must_use]
#[allow(clippy::cast_possible_truncation)] // Justified: the downcast is intentional (´rem:extraction:single-precision´)
pub fn to_f32_features(features: &[f64]) -> Vec<f32> {
    features.iter().map(|&v| v as f32).collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Full Sentinel Extraction
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuration for Sentinel feature extraction.
#[derive(Clone, Debug)]
pub struct ExtractionConfig {
    /// Concordance thresholds per axis (N, D, S, C).
    pub concordance_thresholds: [f64; SCORING_AXIS_COUNT],
    /// Maximum chain depth for normalisation. Default: 16
    /// (´tab:config:extraction´).
    pub d_chain_norm: u8,
}

impl Default for ExtractionConfig {
    fn default() -> Self {
        Self {
            concordance_thresholds: [0.5; SCORING_AXIS_COUNT],
            d_chain_norm: 16,
        }
    }
}

/// Extracts all features for a single Sentinel at a given coordinate.
///
/// Combines chain z-scores (24), chain CUSUMs (12), chain structure (8),
/// coordination (12), ledger features (3 + 2 × number of spatial axes), and
/// batch context (1) into a single extraction.
///
/// # Arguments
///
/// * `index` — The Sentinel's current report index
/// * `ledger` — The Sentinel's outcome ledger (read-locked)
/// * `coord` — The coordinate to extract features for
/// * `gamma_t_ledger` — Ledger decay rate (hourly)
/// * `now` — Current persistent timestamp for ledger decay computation
/// * `batch_now` — Current monotonic instant for staleness computation
/// * `spatial_axis_ids` — The outcome axis IDs for per-axis ledger features
/// * `config` — Extraction configuration
/// * `maturity` — The thresholds the Ledger immaturity flag is taken against
/// * `hierarchical_asserted` — Whether hierarchical alarm is asserted
///
/// # Returns
///
/// A tuple of `(SentinelExtraction, SentinelAlarmSummary)`. The extraction
/// contains all features as f32; the alarm summary provides diagnostic data.
///
/// # Cross-References
///
/// - (´setup:extraction:from-batch-report´) — the six groups this function
///   appends, and the fixed order that is part of the contract
/// - (´def:extraction:slot´) — the width the six groups sum to
#[must_use]
#[allow(clippy::too_many_arguments)] // Extraction requires many inputs
pub fn extract_sentinel(
    index: &ReportIndex,
    ledger: &SentinelLedger,
    coord: u128,
    gamma_t_ledger: f64,
    now: &PersistentTimestamp,
    batch_now: &std::time::Instant,
    spatial_axis_ids: &[OutcomeAxisId],
    config: &ExtractionConfig,
    maturity: &ImmaturityCriteria,
    hierarchical_asserted: bool,
) -> (SentinelExtraction, SentinelAlarmSummary) {
    // Step 1: Collect ancestor chain
    let ancestor_chain = collect_ancestor_chain(index, coord);
    let has_report = !ancestor_chain.is_empty();

    // If no report, return empty extraction with alarm defaults
    if !has_report {
        let empty_features = vec![0.0f32; 24 + 12 + 8 + 12 + 3 + 2 * spatial_axis_ids.len() + 1];
        let extraction = SentinelExtraction::new(coord, empty_features, false);
        let alarm = SentinelAlarmSummary::default();
        return (extraction, alarm);
    }

    // Step 2: Extract chain z-scores (24 features)
    let chain_zscores = chain::extract_chain_zscores(&ancestor_chain);

    // Step 3: Extract chain CUSUMs (12 features)
    let chain_cusums = cusum::extract_chain_cusums(&ancestor_chain);

    // Step 4: Compute report staleness
    let staleness = compute_report_staleness(index, batch_now);

    // Step 5: Extract chain structure (8 features)
    let chain_structure = structure::extract_chain_structure(&ancestor_chain, config.d_chain_norm, staleness);

    // Step 6: Extract coordination features (12 features)
    let coordination_features = coordination::extract_coordination(
        index.coordination(),
        &config.concordance_thresholds,
        config.d_chain_norm,
        index.level_data().full_set_size,
    );

    // Step 7: Extract ledger features (3 + 2×m_s features)
    let ledger_entry = route_ledger_for_coord(ledger, coord);
    // The maturity predicate is about the entry the routing actually reached,
    // so it is taken from that entry rather than from the entry's existence
    // (´def:runtime:alarm-summary´). An absent entry — the rootless Ledger —
    // is immature for want of any evidence at all.
    let ledger_immature = ledger_entry.is_none_or(|entry| entry.is_immature(maturity, gamma_t_ledger, now));
    let decayed = ledger_entry.map_or_else(DecayedView::neutral, |entry| entry.read_decayed(gamma_t_ledger, now));
    let ledger_features = ledger_features::extract_ledger_features(&decayed, spatial_axis_ids);

    // Step 8: Extract batch context (1 feature)
    let batch_context = extract_batch_context(&ancestor_chain);

    // Step 9: Compute alarm summary
    let alarm = alarm::compute_alarm_summary(&ancestor_chain, index.coordination(), hierarchical_asserted, ledger_immature);

    // Step 10: Assemble all features
    let total_features = 24 + 12 + 8 + 12 + ledger_features.len() + 1;
    let mut all_features = Vec::with_capacity(total_features);

    // Append in order: chain z-scores, CUSUMs, structure, coordination, ledger, batch context
    all_features.extend(chain_zscores);
    all_features.extend(chain_cusums);
    all_features.extend(chain_structure);
    all_features.extend(coordination_features);
    all_features.extend(ledger_features);
    all_features.push(batch_context);

    // Step 11: Downcast to f32
    let features_f32 = to_f32_features(&all_features);
    let extraction = SentinelExtraction::new(coord, features_f32, true);

    (extraction, alarm)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Computes report staleness in seconds.
///
/// Returns the time since the report was received, capped at 1 year (`31_536_000` s).
/// Returns 0.0 if the report has no `received_at` timestamp.
///
/// # Cross-References
///
/// - (´def:extraction:report-staleness´) — the staleness this computes
fn compute_report_staleness(index: &ReportIndex, now: &std::time::Instant) -> f64 {
    index.received_at().map_or(0.0, |received| {
        let elapsed = now.saturating_duration_since(received).as_secs_f64();
        // Cap at 1 year (MAX_DECAY_HOURS * 3600)
        elapsed.min(crate::types::MAX_DECAY_HOURS * 3600.0)
    })
}

/// Extracts the batch throughput proxy from the containing reported cell.
///
/// The ancestor chain is ordered deepest first, so the first entry is the
/// specific cell containing the request coordinate.
#[allow(clippy::cast_precision_loss)] // Sample counts are a throughput proxy; exact integer precision is not required.
fn extract_batch_context(ancestor_chain: &[(u8, &ReportCellEntry)]) -> f64 {
    ancestor_chain
        .first()
        .map_or(0.0, |(_, entry)| (entry.sample_count as f64).ln_1p())
}

/// Routes a coordinate to the Ledger entry the assessment reads.
///
/// The deepest entry containing the coordinate, which is the routing the
/// specification states for Ledger features (´alg:runtime:extraction-routing´):
/// the narrowest cell that has been tracked is the one whose outcome history
/// describes this neighbourhood, and a broad ancestor answering in its place
/// would dilute a locally concentrated signal into the region's average. The
/// walk is the Ledger's own read routing (´dec:memory:depth-walk´), taken by
/// key so that a Ledger without its root reports absence rather than panicking
/// — the one state (´todo:code:remove-this-rootless-fallback-once´) still
/// admits on the assessment path.
///
/// Returns `None` only for that rootless Ledger. Wherever the root is in place
/// the lookup cannot fail, because the root contains every coordinate
/// (´def:ledger:root-semantics´).
fn route_ledger_for_coord(ledger: &SentinelLedger, coord: u128) -> Option<&LedgerEntry> {
    let key = routing::read_route_key(ledger, coord);
    ledger.get(&key)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    //! Tests for ancestor chain collection and full extraction orchestration.
    //!
    //! The chain walk decides which cells a coordinate's features are drawn
    //! from, and the orchestration decides how the resulting blocks are laid
    //! end to end into one fixed-width row. The property-style tests then
    //! re-derive relationships that must hold between slots — the gradient
    //! against its two endpoints, the maximum against the views it dominates —
    //! across chains of several lengths at once.

    #![allow(clippy::items_after_statements)]
    #![allow(clippy::unreadable_literal)]
    #![allow(clippy::cast_possible_truncation)]
    #![allow(clippy::cast_precision_loss)]
    #![allow(clippy::uninlined_format_args)]
    #![allow(clippy::approx_constant)]

    use std::collections::HashMap;

    use super::*;
    use crate::ledger::LedgerUpdate;
    use crate::report::{AxisScoreSet, AxisScoreSnapshot, ReportCellEntry, ReportIndex, ReportLevelData};
    use crate::types::{Action, LedgerKey};

    /// The Ledger's own hourly decay rate `γ_t,ledger`, as the decay definition
    /// fixes it (´def:ledger:time-decay´).
    const GAMMA_T_LEDGER: f64 = 0.999;

    /// The immaturity criteria a test states for itself.
    const fn criteria(materiality_threshold: u64, attenuation_floor: f64, lambda_l: f64) -> ImmaturityCriteria {
        ImmaturityCriteria {
            materiality_threshold,
            attenuation_floor,
            lambda_l,
        }
    }

    /// The shipped thresholds, for the extractions that are not about the flag.
    fn shipped_criteria() -> ImmaturityCriteria {
        let config = crate::config::types::AssayerConfig::default();
        criteria(
            config.monitoring.ledger_materiality_threshold,
            config.monitoring.attenuation_materiality_floor,
            config.ledger.lambda_l,
        )
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Ancestor Chain Walk Tests
    // ═══════════════════════════════════════════════════════════════════════════

    /// Creates a test entry with specified depth and `max_z` values.
    fn make_entry(depth: u8, n_z: f64, d_z: f64, s_z: f64, c_z: f64) -> ReportCellEntry {
        ReportCellEntry {
            depth,
            sample_count: 100,
            is_competitive: true,
            rank: 3,
            cap: 8,
            energy_ratio: 0.8,
            noise_influence: 0.1,
            scores: AxisScoreSet {
                novelty: AxisScoreSnapshot {
                    max_z: n_z,
                    ..Default::default()
                },
                displacement: AxisScoreSnapshot {
                    max_z: d_z,
                    ..Default::default()
                },
                surprise: AxisScoreSnapshot {
                    max_z: s_z,
                    ..Default::default()
                },
                coherence: AxisScoreSnapshot {
                    max_z: c_z,
                    ..Default::default()
                },
            },
            degraded: false,
        }
    }

    /// Builds a `ReportIndex` from cell entries.
    fn make_index(entries: Vec<(LedgerKey, ReportCellEntry)>) -> ReportIndex {
        let max_depth = entries.iter().map(|(k, _)| k.depth).max().unwrap_or(0);
        let cells: HashMap<LedgerKey, ReportCellEntry> = entries.into_iter().collect();
        ReportIndex::from_components(cells, vec![], ReportLevelData::default(), max_depth)
    }

    /// The root cell covers the whole coordinate space, so a report carrying
    /// nothing but the root still yields a chain — of exactly one entry, at
    /// depth zero — for any coordinate whatsoever. Every request can therefore
    /// be assessed against whatever the Sentinel does know, however coarse, and
    /// a young Sentinel that has not yet split is not a Sentinel that answers
    /// nothing.
    ///
    /// ´claim:extract:the-root-covers-every-coordinate-so-a-root-only-report-always-yields-a-chain-of-one´
    /// ´test:unit:chain-single-root´
    #[test]
    fn chain_single_root() {
        // Index with root only
        let root = make_entry(0, 0.5, 0.3, 1.0, 0.0);
        let index = make_index(vec![(LedgerKey::new(0, 0), root)]);

        // Any coordinate should return just the root
        let chain = collect_ancestor_chain(&index, 0x1234_5678_9ABC_DEF0);
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].0, 0); // depth 0
    }

    /// The walk returns the containing cells ordered deepest first, with the
    /// root last. Every view downstream is defined against that ordering — the
    /// cell view reads the front, the root view the back — so the order is part
    /// of the contract rather than an accident of how the index was iterated.
    ///
    /// ´claim:extract:the-ancestor-chain-is-ordered-deepest-first-with-the-root-last´
    /// ´test:unit:chain-three-depths´
    #[test]
    fn chain_three_depths() {
        // Index with entries at d=0, d=4, d=8
        let root = make_entry(0, 0.5, 0.3, 1.0, 0.0);
        let d4 = make_entry(4, 1.2, 0.7, 1.0, 0.4);
        let d8 = make_entry(8, 2.8, 1.5, 1.0, 0.9);

        // Depth-4 cell covers [0x1000..., 0x1FFF...]
        let d4_lo: u128 = 0x1000_0000_0000_0000_0000_0000_0000_0000;
        // Depth-8 cell covers [0x1200..., 0x12FF...]
        let d8_lo: u128 = 0x1200_0000_0000_0000_0000_0000_0000_0000;

        let index = make_index(vec![
            (LedgerKey::new(0, 0), root),
            (LedgerKey::new(d4_lo, 4), d4),
            (LedgerKey::new(d8_lo, 8), d8),
        ]);

        // Coordinate in the depth-8 cell
        let coord: u128 = 0x1234_5678_9ABC_DEF0_1234_5678_9ABC_DEF0;
        let chain = collect_ancestor_chain(&index, coord);

        // Should return 3 entries, deepest first
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0].0, 8); // deepest
        assert_eq!(chain[1].0, 4);
        assert_eq!(chain[2].0, 0); // root
    }

    /// The chain is built from the cells the report actually carried, so a
    /// depth the report skipped is simply absent and the walk continues past it
    /// to the shallower levels. Sentinels report the cells they hold rather than
    /// a filled-in ladder of every level, and extraction must not stall at the
    /// first level it cannot find.
    ///
    /// ´claim:extract:the-chain-holds-only-cells-the-report-carried-so-an-unreported-depth-is-skipped´
    /// ´test:unit:chain-gap-in-depths´
    #[test]
    fn chain_gap_in_depths() {
        // Index with entries at d=0 and d=8 (no d=4)
        let root = make_entry(0, 0.5, 0.3, 1.0, 0.0);
        let d8 = make_entry(8, 2.8, 1.5, 1.0, 0.9);

        let d8_lo: u128 = 0x1200_0000_0000_0000_0000_0000_0000_0000;

        let index = make_index(vec![(LedgerKey::new(0, 0), root), (LedgerKey::new(d8_lo, 8), d8)]);

        let coord: u128 = 0x1234_5678_9ABC_DEF0_1234_5678_9ABC_DEF0;
        let chain = collect_ancestor_chain(&index, coord);

        // Should return 2 entries, gap skipped
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].0, 8);
        assert_eq!(chain[1].0, 0);
    }

    /// A coordinate nested seven levels deep collects all seven containing
    /// cells, still deepest first and still ending at the root. The ordering
    /// holds at whatever depth a mature Sentinel has grown to, so the cell and
    /// root views keep meaning the same thing as the tree deepens beneath them.
    ///
    /// (´claim:extract:the-ancestor-chain-is-ordered-deepest-first-with-the-root-last´)
    /// ´test:unit:chain-many-depths´
    #[test]
    fn chain_many_depths() {
        // Index with entries at d=0,2,4,6,8,10,12
        // Use a coord that fits in the deepest cell
        // Each depth's lo is the dyadic ancestor of coord 0x1234...
        let root = make_entry(0, 0.5, 0.3, 1.0, 0.0);
        let mut cells: HashMap<LedgerKey, ReportCellEntry> = HashMap::new();
        cells.insert(LedgerKey::new(0, 0), root);

        // Build cells at increasing depths that all cover 0x1234...
        for d in [2u8, 4, 6, 8, 10, 12] {
            let lo = LedgerKey::from_coordinate(0x1234_5678_9ABC_DEF0_1234_5678_9ABC_DEF0, d).lo;
            cells.insert(LedgerKey::new(lo, d), make_entry(d, 1.0, 1.0, 1.0, 1.0));
        }

        let index = ReportIndex::from_components(cells, vec![], ReportLevelData::default(), 12);
        let coord: u128 = 0x1234_5678_9ABC_DEF0_1234_5678_9ABC_DEF0;
        let chain = collect_ancestor_chain(&index, coord);

        assert_eq!(chain.len(), 7);
        assert_eq!(chain[0].0, 12); // deepest first
        assert_eq!(chain[6].0, 0); // root last
    }

    /// Membership is decided by containment and not by depth: a coordinate
    /// inside a reported depth-four cell but outside the depth-eight cell
    /// nested within it collects the shallower cell and skips the deeper one.
    /// The chain is the coordinate's own ancestry, so a sibling region's
    /// conditions never leak into a row about somewhere else.
    ///
    /// ´claim:extract:a-cell-that-does-not-contain-the-coordinate-is-left-out-however-deep-it-was-reported´
    /// ´test:unit:chain-coord-in-d4-not-d8´
    #[test]
    fn chain_coord_in_d4_not_d8() {
        // Entries at d=0, d=4, d=8
        let root = make_entry(0, 0.5, 0.3, 1.0, 0.0);
        let d4 = make_entry(4, 1.2, 0.7, 1.0, 0.4);
        let d8 = make_entry(8, 2.8, 1.5, 1.0, 0.9);

        // d4 covers [0x1000..., 0x1FFF...]
        let d4_lo: u128 = 0x1000_0000_0000_0000_0000_0000_0000_0000;
        // d8 covers [0x1200..., 0x12FF...] — a subset of d4
        let d8_lo: u128 = 0x1200_0000_0000_0000_0000_0000_0000_0000;

        let index = make_index(vec![
            (LedgerKey::new(0, 0), root),
            (LedgerKey::new(d4_lo, 4), d4),
            (LedgerKey::new(d8_lo, 8), d8),
        ]);

        // Coordinate in d4 but NOT in d8 (e.g., 0x1100...)
        let coord: u128 = 0x1100_0000_0000_0000_0000_0000_0000_0000;
        let chain = collect_ancestor_chain(&index, coord);

        // Should return only d4 and root (not d8)
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].0, 4);
        assert_eq!(chain[1].0, 0);
    }

    /// A coordinate landing in a region the Sentinel has never split falls back
    /// to the root alone, because that is the only reported cell containing it.
    /// The fallback is not a special case but the containment rule reaching its
    /// limit, so a request into unexplored space is still assessed — against the
    /// coarsest evidence there is.
    ///
    /// (´claim:extract:a-cell-that-does-not-contain-the-coordinate-is-left-out-however-deep-it-was-reported´)
    /// ´test:unit:chain-root-only-fallback´
    #[test]
    fn chain_root_only_fallback() {
        // Entries at d=0 and d=8
        let root = make_entry(0, 0.5, 0.3, 1.0, 0.0);
        let d8 = make_entry(8, 2.8, 1.5, 1.0, 0.9);

        let d8_lo: u128 = 0x1200_0000_0000_0000_0000_0000_0000_0000;

        let index = make_index(vec![(LedgerKey::new(0, 0), root), (LedgerKey::new(d8_lo, 8), d8)]);

        // Coordinate completely outside d8 (e.g., 0xFF00...)
        let coord: u128 = 0xFF00_0000_0000_0000_0000_0000_0000_0000;
        let chain = collect_ancestor_chain(&index, coord);

        // Should return only root
        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].0, 0);
    }

    /// A cell claims its own lower bound: a coordinate sitting exactly at the
    /// start of a reported cell is inside it. Cells tile the space without
    /// overlap, so each boundary must belong to exactly one side, and here it
    /// belongs to the cell that begins there.
    ///
    /// ´claim:extract:a-cell-spans-its-lower-bound-through-its-final-position-and-stops-before-the-next´
    /// ´test:unit:chain-boundary-lo´
    #[test]
    fn chain_boundary_lo() {
        // Test coord exactly at cell.lo
        let root = make_entry(0, 0.5, 0.3, 1.0, 0.0);
        let d8 = make_entry(8, 2.8, 1.5, 1.0, 0.9);

        let d8_lo: u128 = 0x1200_0000_0000_0000_0000_0000_0000_0000;

        let index = make_index(vec![(LedgerKey::new(0, 0), root), (LedgerKey::new(d8_lo, 8), d8)]);

        // Coord exactly at d8_lo
        let chain = collect_ancestor_chain(&index, d8_lo);

        // Should include d8
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].0, 8);
        assert_eq!(chain[1].0, 0);
    }

    /// The far end of a cell belongs to it too: the last coordinate before the
    /// next cell's width begins still collects that cell. A cell's span is its
    /// full width and not one position short of it, so no coordinate falls
    /// through the crack between two adjacent cells.
    ///
    /// (´claim:extract:a-cell-spans-its-lower-bound-through-its-final-position-and-stops-before-the-next´)
    /// ´test:unit:chain-boundary-hi´
    #[test]
    fn chain_boundary_hi() {
        // Test coord at cell.lo + width - 1
        let root = make_entry(0, 0.5, 0.3, 1.0, 0.0);
        let d8 = make_entry(8, 2.8, 1.5, 1.0, 0.9);

        let d8_lo: u128 = 0x1200_0000_0000_0000_0000_0000_0000_0000;
        // Width at depth 8 is 2^(128-8) = 2^120
        let width: u128 = 1_u128 << 120;
        let d8_hi = d8_lo + width - 1;

        let index = make_index(vec![(LedgerKey::new(0, 0), root), (LedgerKey::new(d8_lo, 8), d8)]);

        // Coord at last position in d8
        let chain = collect_ancestor_chain(&index, d8_hi);

        // Should include d8
        assert_eq!(chain.len(), 2);
        assert_eq!(chain[0].0, 8);
    }

    /// One position past the end of a cell is outside it, and the chain drops
    /// back to the root. The cell's span closes exactly where the next begins,
    /// so a coordinate is never credited to a neighbouring region's evidence by
    /// an off-by-one in the walk.
    ///
    /// (´claim:extract:a-cell-spans-its-lower-bound-through-its-final-position-and-stops-before-the-next´)
    /// ´test:unit:chain-past-boundary´
    #[test]
    fn chain_past_boundary() {
        // Test coord at cell.lo + width (just past the cell)
        let root = make_entry(0, 0.5, 0.3, 1.0, 0.0);
        let d8 = make_entry(8, 2.8, 1.5, 1.0, 0.9);

        let d8_lo: u128 = 0x1200_0000_0000_0000_0000_0000_0000_0000;
        let width: u128 = 1_u128 << 120;
        let past_d8 = d8_lo + width; // Just past the cell

        let index = make_index(vec![(LedgerKey::new(0, 0), root), (LedgerKey::new(d8_lo, 8), d8)]);

        // Coord past d8 should only get root
        let chain = collect_ancestor_chain(&index, past_d8);

        assert_eq!(chain.len(), 1);
        assert_eq!(chain[0].0, 0);
    }

    /// Having a dyadic ancestor at some depth is not enough to appear in the
    /// chain: the coordinate has a well-defined depth-four ancestor, yet with no
    /// entry reported there the chain simply has no depth-four member. What
    /// enters the row is evidence the Sentinel actually gathered, never a
    /// position the geometry says could exist.
    ///
    /// (´claim:extract:the-chain-holds-only-cells-the-report-carried-so-an-unreported-depth-is-skipped´)
    /// ´test:unit:chain-contains-only-reported-cells´
    #[test]
    fn chain_contains_only_reported_cells() {
        // Index with entries at d=0 and d=8 only
        // Coord has a d=4 dyadic ancestor, but no entry exists for it
        let root = make_entry(0, 0.5, 0.3, 1.0, 0.0);
        let d8 = make_entry(8, 2.8, 1.5, 1.0, 0.9);

        let d8_lo: u128 = 0x1200_0000_0000_0000_0000_0000_0000_0000;

        let index = make_index(vec![(LedgerKey::new(0, 0), root), (LedgerKey::new(d8_lo, 8), d8)]);

        let coord: u128 = 0x1234_5678_9ABC_DEF0_1234_5678_9ABC_DEF0;
        let chain = collect_ancestor_chain(&index, coord);

        // Chain should NOT contain d=4, even though coord has a d=4 ancestor
        assert_eq!(chain.len(), 2);
        for (depth, _) in &chain {
            assert_ne!(*depth, 4);
        }
    }

    /// An index that holds no report yields an empty chain and is answered
    /// without any walk at all — the absence is detected before the descent
    /// begins. A Sentinel that has never reported is a common state on a cold
    /// start, and it costs nothing rather than a fruitless sweep of every depth.
    ///
    /// ´claim:extract:an-index-with-no-report-yields-an-empty-chain-without-walking-any-depth´
    /// ´test:unit:chain-empty-index´
    #[test]
    fn chain_empty_index() {
        let index = ReportIndex::empty();
        let chain = collect_ancestor_chain(&index, 0x1234_5678_9ABC_DEF0);
        assert!(chain.is_empty());
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Full Extraction Orchestration Tests
    // ═══════════════════════════════════════════════════════════════════════════

    /// A Sentinel that has never reported still produces a row of the full
    /// width, every slot zero, flagged as unoccupied, and an alarm summary that
    /// reads quiet. The model consumes rows in fixed positions, so silence has
    /// to be expressed as a well-formed row rather than as a missing one, and
    /// the flag is what stops those zeros being read as measured calm.
    ///
    /// ´claim:extract:a-sentinel-with-no-report-yields-a-full-width-all-zero-row-flagged-unoccupied´
    /// ´test:unit:extract-occupancy-no-report´
    #[test]
    fn extract_occupancy_no_report() {
        // Empty index → occupancy = false
        let index = ReportIndex::empty();
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();
        let now = PersistentTimestamp::now();
        let batch_now = std::time::Instant::now();
        let config = ExtractionConfig::default();

        let (extraction, alarm) = extract_sentinel(
            &index,
            &ledger,
            0x1234,
            0.01,
            &now,
            &batch_now,
            &[],
            &config,
            &shipped_criteria(),
            false,
        );

        assert!(!extraction.occupancy);
        assert_eq!(extraction.features.len(), 60);
        assert!(extraction.features.iter().all(|v| v == 0.0));
        assert_eq!(alarm.peak_z.to_bits(), 0.0f64.to_bits());
    }

    /// A Sentinel holding even a single root-level cell is flagged occupied.
    /// The flag turns on the moment there is any real evidence to draw on, so
    /// the model can weigh a coarse but genuine reading differently from the
    /// absence of a reading, which the zeros alone could not tell it.
    ///
    /// ´claim:extract:a-sentinel-holding-any-reported-cell-is-flagged-occupied´
    /// ´test:unit:extract-occupancy-reporting´
    #[test]
    fn extract_occupancy_reporting() {
        // Index with root → occupancy = true
        let root = make_entry(0, 0.5, 0.3, 1.0, 0.0);
        let index = make_index(vec![(LedgerKey::new(0, 0), root)]);
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();
        let now = PersistentTimestamp::now();
        let batch_now = std::time::Instant::now();
        let config = ExtractionConfig::default();

        let (extraction, _alarm) = extract_sentinel(
            &index,
            &ledger,
            0x1234,
            0.01,
            &now,
            &batch_now,
            &[],
            &config,
            &shipped_criteria(),
            false,
        );

        assert!(extraction.occupancy);
    }

    /// The row is sixty slots that never move — chain z-scores, chain drift,
    /// structure, coordination, the three base ledger quantities and batch
    /// context — plus exactly two more for each spatial outcome axis the
    /// deployment tracks. Only the ledger tail varies, and it varies at the
    /// end, so a deployment that adds an outcome axis extends the row rather
    /// than renumbering the sixty slots a trained model already depends on.
    ///
    /// ´claim:extract:the-row-is-sixty-fixed-slots-plus-two-more-for-each-spatial-outcome-axis´
    /// ´test:unit:extract-feature-count´
    #[test]
    fn extract_feature_count() {
        // Test that feature count = 24 + 12 + 8 + 12 + (3 + 2*m_s) + 1
        // m_s=0 → 60; m_s=1 → 62; m_s=3 → 66
        let root = make_entry(0, 0.5, 0.3, 1.0, 0.0);
        let index = make_index(vec![(LedgerKey::new(0, 0), root)]);
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();
        let now = PersistentTimestamp::now();
        let batch_now = std::time::Instant::now();
        let config = ExtractionConfig::default();

        // m_s = 0
        let (extraction, _) = extract_sentinel(
            &index,
            &ledger,
            0x1234,
            0.01,
            &now,
            &batch_now,
            &[],
            &config,
            &shipped_criteria(),
            false,
        );
        assert_eq!(extraction.features.len(), 24 + 12 + 8 + 12 + 3 + 1);

        // m_s = 1
        let axis1 = OutcomeAxisId(1);
        let (extraction, _) = extract_sentinel(
            &index,
            &ledger,
            0x1234,
            0.01,
            &now,
            &batch_now,
            &[axis1],
            &config,
            &shipped_criteria(),
            false,
        );
        assert_eq!(extraction.features.len(), 24 + 12 + 8 + 12 + 3 + 2 + 1);

        // m_s = 3
        let axes = [OutcomeAxisId(1), OutcomeAxisId(2), OutcomeAxisId(3)];
        let (extraction, _) = extract_sentinel(
            &index,
            &ledger,
            0x1234,
            0.01,
            &now,
            &batch_now,
            &axes,
            &config,
            &shipped_criteria(),
            false,
        );
        assert_eq!(extraction.features.len(), 24 + 12 + 8 + 12 + 3 + 6 + 1);
    }

    /// A coordinate far outside anything the Sentinel has split still extracts
    /// a complete, occupied row by resting on the root alone. Assessment is
    /// never refused for want of local detail: the answer degrades in precision
    /// as the evidence thins rather than disappearing at some coverage
    /// boundary.
    ///
    /// ´claim:extract:a-coordinate-covered-only-by-the-root-still-extracts-a-complete-occupied-row´
    /// ´test:unit:extract-coord-not-in-report´
    #[test]
    fn extract_coord_not_in_report() {
        // Report covers root only, coord routes to root
        let root = make_entry(0, 0.5, 0.3, 1.0, 0.0);
        let index = make_index(vec![(LedgerKey::new(0, 0), root)]);
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();
        let now = PersistentTimestamp::now();
        let batch_now = std::time::Instant::now();
        let config = ExtractionConfig::default();

        // Any coord will route to root since that's all we have
        let (extraction, _) = extract_sentinel(
            &index,
            &ledger,
            0xFFFF_FFFF_FFFF_FFFF,
            0.01,
            &now,
            &batch_now,
            &[],
            &config,
            &shipped_criteria(),
            false,
        );

        // Should still have occupancy (we have a report)
        assert!(extraction.occupancy);
        // Chain length = 1 (root only)
        // Structure features include chain length normalised
    }

    /// Storing the row at single precision costs less than a thousandth in
    /// relative terms across values spanning ten orders of magnitude, and the
    /// error near zero stays far below the scale at which any feature is read.
    /// Rows are held in bulk, so halving their size is worth an error this far
    /// beneath the noise the features themselves carry.
    ///
    /// ´claim:extract:the-single-precision-downcast-costs-less-than-a-thousandth-in-relative-error´
    /// ´test:unit:extract-f32-precision´
    #[test]
    fn extract_f32_precision() {
        // Test that f32 downcast preserves reasonable precision
        let test_values: Vec<f64> = vec![0.0, 1.0, -1.0, 0.123456789, 1e-10, 1e10, 3.141592653589793];

        let f32_values = to_f32_features(&test_values);
        let roundtrip: Vec<f64> = f32_values.iter().map(|&v| f64::from(v)).collect();

        for (orig, rt) in test_values.iter().zip(roundtrip.iter()) {
            if orig.abs() > 1e-6 {
                let rel_error = ((orig - rt) / orig).abs();
                assert!(rel_error < 1e-3, "f32 precision: {} → {} (rel_err={})", orig, rt, rel_error);
            } else {
                let abs_error = (orig - rt).abs();
                assert!(abs_error < 1e-6, "f32 precision: {} → {} (abs_err={})", orig, rt, abs_error);
            }
        }
    }

    /// Non-finite values cross the downcast intact — a NaN stays a NaN, and
    /// each infinity keeps its sign — while ordinary values are untouched. The
    /// downcast is a change of storage and not a place where corruption is
    /// quietly laundered into a plausible-looking number, so a defect upstream
    /// stays detectable in the stored row.
    ///
    /// ´claim:extract:non-finite-values-survive-the-downcast-rather-than-being-quietly-masked´
    /// ´test:unit:extract-f32-nan-inf-propagation´
    #[test]
    fn extract_f32_nan_inf_propagation() {
        // NaN propagates and each infinity keeps its sign, as the claim above
        // states (´claim:extract:non-finite-values-survive-the-downcast-rather-than-being-quietly-masked´).
        let special_values: Vec<f64> = vec![f64::NAN, f64::INFINITY, f64::NEG_INFINITY, 1.0, -1.0];
        let f32_values = to_f32_features(&special_values);

        assert!(f32_values[0].is_nan(), "NaN should propagate");
        assert!(
            f32_values[1].is_infinite() && f32_values[1].is_sign_positive(),
            "Inf should propagate"
        );
        assert!(
            f32_values[2].is_infinite() && f32_values[2].is_sign_negative(),
            "-Inf should propagate"
        );
        assert!((f32_values[3] - 1.0).abs() < 1e-6, "Normal values preserved");
        assert!((f32_values[4] + 1.0).abs() < 1e-6, "Normal values preserved");
    }

    /// End to end from a hand-built three-level report, the blocks land in one
    /// fixed order: z-scores first, then chain drift from slot twenty-four, on
    /// through structure, coordination and the ledger, with batch context — the
    /// log of the containing cell's sample count — occupying the final slot.
    /// The alarm summary comes back alongside and agrees with the row's own
    /// peak. Absolute indices are the model's only names for its inputs, so the
    /// concatenation order is pinned against known values rather than left to
    /// the order the steps happen to run in.
    ///
    /// ´claim:extract:the-row-concatenates-the-blocks-in-one-fixed-order-ending-in-batch-context´
    /// ´test:unit:extract-golden-report-full´
    #[test]
    #[allow(clippy::too_many_lines)]
    fn extract_golden_report_full() {
        // Build a golden report and verify extraction
        use crate::testing::{DEPTH_4_LO, DEPTH_8_LO, GOLDEN_COORD, scores};

        let root = ReportCellEntry {
            depth: 0,
            sample_count: 1000,
            is_competitive: true,
            rank: 4,
            cap: 8,
            energy_ratio: 0.85,
            noise_influence: 0.05,
            scores: AxisScoreSet {
                novelty: AxisScoreSnapshot {
                    max_z: scores::ROOT_MAX_Z[0],
                    cusum: scores::ROOT_CUSUM[0],
                    ..Default::default()
                },
                displacement: AxisScoreSnapshot {
                    max_z: scores::ROOT_MAX_Z[1],
                    cusum: scores::ROOT_CUSUM[1],
                    ..Default::default()
                },
                surprise: AxisScoreSnapshot {
                    max_z: scores::ROOT_MAX_Z[2],
                    cusum: scores::ROOT_CUSUM[2],
                    ..Default::default()
                },
                coherence: AxisScoreSnapshot {
                    max_z: scores::ROOT_MAX_Z[3],
                    cusum: scores::ROOT_CUSUM[3],
                    ..Default::default()
                },
            },
            degraded: false,
        };

        let d4 = ReportCellEntry {
            depth: 4,
            sample_count: 200,
            is_competitive: false,
            rank: 4,
            cap: 8,
            energy_ratio: 0.80,
            noise_influence: 0.08,
            scores: AxisScoreSet {
                novelty: AxisScoreSnapshot {
                    max_z: scores::D4_MAX_Z[0],
                    cusum: scores::D4_CUSUM[0],
                    ..Default::default()
                },
                displacement: AxisScoreSnapshot {
                    max_z: scores::D4_MAX_Z[1],
                    cusum: scores::D4_CUSUM[1],
                    ..Default::default()
                },
                surprise: AxisScoreSnapshot {
                    max_z: scores::D4_MAX_Z[2],
                    cusum: scores::D4_CUSUM[2],
                    ..Default::default()
                },
                coherence: AxisScoreSnapshot {
                    max_z: scores::D4_MAX_Z[3],
                    cusum: scores::D4_CUSUM[3],
                    ..Default::default()
                },
            },
            degraded: false,
        };

        let d8 = ReportCellEntry {
            depth: 8,
            sample_count: 50,
            is_competitive: true,
            rank: 3,
            cap: 8,
            energy_ratio: 0.72,
            noise_influence: 0.15,
            scores: AxisScoreSet {
                novelty: AxisScoreSnapshot {
                    max_z: scores::D8_MAX_Z[0],
                    cusum: scores::D8_CUSUM[0],
                    ..Default::default()
                },
                displacement: AxisScoreSnapshot {
                    max_z: scores::D8_MAX_Z[1],
                    cusum: scores::D8_CUSUM[1],
                    ..Default::default()
                },
                surprise: AxisScoreSnapshot {
                    max_z: scores::D8_MAX_Z[2],
                    cusum: scores::D8_CUSUM[2],
                    ..Default::default()
                },
                coherence: AxisScoreSnapshot {
                    max_z: scores::D8_MAX_Z[3],
                    cusum: scores::D8_CUSUM[3],
                    ..Default::default()
                },
            },
            degraded: false,
        };

        let mut cells = HashMap::new();
        cells.insert(LedgerKey::new(0, 0), root);
        cells.insert(LedgerKey::new(DEPTH_4_LO, 4), d4);
        cells.insert(LedgerKey::new(DEPTH_8_LO, 8), d8);

        let index = ReportIndex::from_components(cells, vec![], ReportLevelData::default(), 8);
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();
        let now = PersistentTimestamp::now();
        let batch_now = std::time::Instant::now();
        let config = ExtractionConfig::default();

        let (extraction, alarm) = extract_sentinel(
            &index,
            &ledger,
            GOLDEN_COORD,
            0.01,
            &now,
            &batch_now,
            &[],
            &config,
            &shipped_criteria(),
            false,
        );

        // Verify basic properties
        assert!(extraction.occupancy);
        assert_eq!(extraction.features.len(), 24 + 12 + 8 + 12 + 3 + 1);

        // Verify chain z-scores (indices 0–23)
        // Cell z-scores (0–3): [2.8, 1.5, 1.0, 0.9]
        const TOL: f32 = 1e-5;
        let expected_batch_context = (50.0_f32).ln_1p();
        assert!(
            (extraction.features.value(59) - f64::from(expected_batch_context)).abs() < f64::from(TOL),
            "Batch context"
        );
        assert!((extraction.features.value(0) - 2.8).abs() < f64::from(TOL), "Cell N z-score");
        assert!((extraction.features.value(1) - 1.5).abs() < f64::from(TOL), "Cell D z-score");
        assert!((extraction.features.value(2) - 1.0).abs() < f64::from(TOL), "Cell S z-score");
        assert!((extraction.features.value(3) - 0.9).abs() < f64::from(TOL), "Cell C z-score");

        // Root z-scores (4–7): [0.5, 0.3, 1.0, 0.0]
        assert!((extraction.features.value(4) - 0.5).abs() < f64::from(TOL), "Root N z-score");
        assert!((extraction.features.value(5) - 0.3).abs() < f64::from(TOL), "Root D z-score");

        // Chain CUSUMs (indices 24–35)
        // Cell CUSUMs (24–27): [0.7, 0.4, 0.5, 0.3]
        assert!((extraction.features.value(24) - 0.7).abs() < f64::from(TOL), "Cell N CUSUM");
        assert!((extraction.features.value(25) - 0.4).abs() < f64::from(TOL), "Cell D CUSUM");

        // Alarm should reflect the peak scores
        assert!(
            alarm.peak_z >= 2.8 - f64::from(TOL),
            "Peak z should be at least cell N z-score"
        );
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Property-Like Tests
    // ═══════════════════════════════════════════════════════════════════════════

    /// Generates deterministic test chains with varying parameters.
    fn make_test_chains() -> Vec<Vec<(u8, ReportCellEntry)>> {
        let mut chains = Vec::new();

        // Chain lengths: 1, 2, 3, 5
        for &len in &[1usize, 2, 3, 5] {
            let mut entries = Vec::new();
            for i in 0..len {
                let depth = (i * 4) as u8;
                // Vary z-scores: deeper cells tend to have higher z
                let z = f64::midpoint(i as f64, 1.0);
                entries.push((depth, make_entry(depth, z, z * 0.8, z * 0.5, z * 0.3)));
            }
            // Reverse to get deepest-first order
            entries.reverse();
            chains.push(entries);
        }

        chains
    }

    /// Across chains of one, two, three and five entries, every z-score, drift
    /// and structure slot comes out finite. The degenerate lengths are where
    /// the divisions and folds could go wrong — a mean over one entry, a spread
    /// with no deviation to take — and a single non-finite slot would spread
    /// through standardisation into every feature the model reads.
    ///
    /// ´claim:extract:chains-of-every-length-extract-to-finite-values-in-every-slot´
    /// ´test:unit:property-no-nan-in-extraction´
    #[test]
    fn property_no_nan_in_extraction() {
        // Test that valid inputs never produce NaN outputs
        for entries in make_test_chains() {
            let chain: Vec<(u8, &ReportCellEntry)> = entries.iter().map(|(d, e)| (*d, e)).collect();

            let zscores = chain::extract_chain_zscores(&chain);
            for (i, &v) in zscores.iter().enumerate() {
                assert!(v.is_finite(), "z-score[{}] is NaN/Inf for chain len {}", i, chain.len());
            }

            let cusums = cusum::extract_chain_cusums(&chain);
            for (i, &v) in cusums.iter().enumerate() {
                assert!(v.is_finite(), "CUSUM[{}] is NaN/Inf for chain len {}", i, chain.len());
            }

            let structure = structure::extract_chain_structure(&chain, 128, 0.0);
            for (i, &v) in structure.iter().enumerate() {
                assert!(v.is_finite(), "structure[{}] is NaN/Inf for chain len {}", i, chain.len());
            }
        }
    }

    /// The gradient slot always equals its own row's cell slot minus its root
    /// slot, on every axis and at every chain length. The three are not
    /// computed from independent readings of the chain, so a model that learns
    /// on any two of them is reading the third consistently rather than picking
    /// up a discrepancy between separate traversals.
    ///
    /// ´claim:extract:the-gradient-slot-is-always-exactly-the-cell-slot-minus-the-root-slot´
    /// ´test:unit:property-gradient-is-cell-minus-root´
    #[test]
    fn property_gradient_is_cell_minus_root() {
        // Gradient = Cell - Root for all axes
        for entries in make_test_chains() {
            let chain: Vec<(u8, &ReportCellEntry)> = entries.iter().map(|(d, e)| (*d, e)).collect();
            let zscores = chain::extract_chain_zscores(&chain);

            const TOL: f64 = 1e-14;
            for axis in 0..4 {
                let cell_z = zscores[axis];
                let root_z = zscores[4 + axis];
                let gradient = zscores[16 + axis];
                let expected = cell_z - root_z;
                assert!(
                    (gradient - expected).abs() < TOL,
                    "Gradient[{}] = {} but cell - root = {} for chain len {}",
                    axis,
                    gradient,
                    expected,
                    chain.len()
                );
            }
        }
    }

    /// The maximum view never falls below the cell view on any axis, whatever
    /// the chain's length. The deepest entry is one of the entries the maximum
    /// ranges over, so this is the ordering the layout promises: the maximum is
    /// an upper bound on the endpoint views and not a competing reading of the
    /// chain.
    ///
    /// ´claim:extract:the-maximum-view-dominates-both-the-cell-and-root-views-on-every-axis´
    /// ´test:unit:property-max-z-geq-cell-z´
    #[test]
    fn property_max_z_geq_cell_z() {
        // Max z >= Cell z for all axes
        for entries in make_test_chains() {
            let chain: Vec<(u8, &ReportCellEntry)> = entries.iter().map(|(d, e)| (*d, e)).collect();
            let zscores = chain::extract_chain_zscores(&chain);

            for axis in 0..4 {
                let cell_z = zscores[axis];
                let max_z = zscores[8 + axis];
                assert!(
                    max_z >= cell_z - 1e-14,
                    "Max[{}] = {} < Cell = {} for chain len {}",
                    axis,
                    max_z,
                    cell_z,
                    chain.len()
                );
            }
        }
    }

    /// The other endpoint is bounded the same way: the maximum view never falls
    /// below the root view either. Both ends of the ancestry are members of the
    /// set being maximised over, so no chain — including the single-entry chain
    /// where all three coincide — can put the maximum beneath one of them.
    ///
    /// (´claim:extract:the-maximum-view-dominates-both-the-cell-and-root-views-on-every-axis´)
    /// ´test:unit:property-max-z-geq-root-z´
    #[test]
    fn property_max_z_geq_root_z() {
        // Max z >= Root z for all axes
        for entries in make_test_chains() {
            let chain: Vec<(u8, &ReportCellEntry)> = entries.iter().map(|(d, e)| (*d, e)).collect();
            let zscores = chain::extract_chain_zscores(&chain);

            for axis in 0..4 {
                let root_z = zscores[4 + axis];
                let max_z = zscores[8 + axis];
                assert!(
                    max_z >= root_z - 1e-14,
                    "Max[{}] = {} < Root = {} for chain len {}",
                    axis,
                    max_z,
                    root_z,
                    chain.len()
                );
            }
        }
    }

    /// The spread view is never negative on any axis or chain length, including
    /// the single-entry case where it is exactly zero. It is a standard
    /// deviation, so its sign carries no information and a negative value could
    /// only mean the variance computation had gone wrong beneath it.
    ///
    /// ´claim:extract:the-spread-view-is-never-negative-on-any-axis-or-chain-length´
    /// ´test:unit:property-spread-nonnegative´
    #[test]
    fn property_spread_nonnegative() {
        // Spread >= 0 for all axes
        for entries in make_test_chains() {
            let chain: Vec<(u8, &ReportCellEntry)> = entries.iter().map(|(d, e)| (*d, e)).collect();
            let zscores = chain::extract_chain_zscores(&chain);

            for axis in 0..4 {
                let spread = zscores[20 + axis];
                assert!(
                    spread >= -1e-14,
                    "Spread[{}] = {} < 0 for chain len {}",
                    axis,
                    spread,
                    chain.len()
                );
            }
        }
    }

    /// Sweeping the spatial axis count from none through three, the row's width
    /// tracks the configuration at every step rather than only at the values
    /// spot-checked elsewhere. The width is a function of configuration alone,
    /// so an operator can predict the model's input dimension from the
    /// deployment's axis list without running an extraction to find out.
    ///
    /// (´claim:extract:the-row-is-sixty-fixed-slots-plus-two-more-for-each-spatial-outcome-axis´)
    /// ´test:unit:property-feature-count-matches-config´
    #[test]
    fn property_feature_count_matches_config() {
        // Feature count = 24 + 12 + 8 + 12 + (3 + 2*m_s) + 1
        let root = make_entry(0, 0.5, 0.3, 1.0, 0.0);
        let index = make_index(vec![(LedgerKey::new(0, 0), root)]);
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();
        let now = PersistentTimestamp::now();
        let batch_now = std::time::Instant::now();
        let config = ExtractionConfig::default();

        for m_s in 0..=3usize {
            let axes: Vec<OutcomeAxisId> = (1..=m_s).map(|i| OutcomeAxisId(i as u32)).collect();
            let (extraction, _) = extract_sentinel(
                &index,
                &ledger,
                0x1234,
                0.01,
                &now,
                &batch_now,
                &axes,
                &config,
                &shipped_criteria(),
                false,
            );

            let expected_len = 24 + 12 + 8 + 12 + 3 + 2 * m_s + 1;
            assert_eq!(
                extraction.features.len(),
                expected_len,
                "m_s={}: expected {} features, got {}",
                m_s,
                expected_len,
                extraction.features.len()
            );
        }
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // Ledger Routing and Maturity Tests
    // ═══════════════════════════════════════════════════════════════════════════

    /// The base ledger feature slot: the three whole-entry quantities follow the
    /// twenty-four z-scores, twelve accumulators, eight structure slots and
    /// twelve coordination slots (´def:extraction:slot´).
    const LEDGER_BASE: usize = 24 + 12 + 8 + 12;

    /// The root's diluted outcome history, in exactly representable fractions so
    /// the expectation survives the single-precision store unrounded.
    const ROOT_HISTORY: (f64, f64, f64) = (0.125, 0.25, 0.5);

    /// The deep cell's own, distinctly adverse, outcome history.
    const DEEP_HISTORY: (f64, f64, f64) = (0.875, 0.75, -0.5);

    /// The depth of the tracked cell inside the split region.
    const SPLIT_DEPTH: u8 = 8;

    /// A coordinate inside the tracked depth-eight cell at `lo = 0`.
    const COORD_IN_SPLIT: u128 = 0x0012_0000_0000_0000_0000_0000_0000_0000;

    /// A coordinate the tracked cell does not cover: its depth-eight ancestor
    /// begins at `0x0F00…` rather than at zero, so only the root contains it.
    const COORD_OUTSIDE_SPLIT: u128 = 0x0F00_0000_0000_0000_0000_0000_0000_0000;

    /// Builds an entry carrying the given history, stamped at `at` so a read
    /// taken at `at` pays no decay and the stored values are the read values.
    fn entry_with_history(history: (f64, f64, f64), at: PersistentTimestamp) -> LedgerEntry {
        let mut entry = LedgerEntry::new_neutral();
        entry.last_updated = at;
        entry.ewma_bad_rate = history.0;
        entry.compressed_valence_ewma = history.1;
        entry.raw_valence_ewma = history.2;
        entry
    }

    /// A Ledger for a split region: a diluted root, and a tracked depth-eight
    /// cell at `lo = 0` carrying a distinctly adverse history of its own.
    fn split_region_ledger(at: PersistentTimestamp) -> SentinelLedger {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();
        *ledger.root_mut() = entry_with_history(ROOT_HISTORY, at);
        ledger.insert(LedgerKey::new(0, SPLIT_DEPTH), entry_with_history(DEEP_HISTORY, at));
        ledger
    }

    /// A root-only report index, so every extraction below rests on a chain of
    /// one and the Ledger read is the only thing that varies.
    fn root_only_index() -> ReportIndex {
        make_index(vec![(LedgerKey::new(0, 0), make_entry(0, 0.5, 0.3, 1.0, 0.0))])
    }

    /// Reads the three base ledger slots out of an extraction.
    fn ledger_history(extraction: &SentinelExtraction) -> (f64, f64, f64) {
        (
            extraction.features.value(LEDGER_BASE),
            extraction.features.value(LEDGER_BASE + 1),
            extraction.features.value(LEDGER_BASE + 2),
        )
    }

    /// Drives `arrivals` eligible adverse labels into an entry, one every
    /// `interval_hours` from the epoch, and returns the last arrival's instant.
    ///
    /// The entry is restamped at the epoch first, so the driven history is the
    /// whole of its history however the entry was constructed.
    fn drive_eligible(entry: &mut LedgerEntry, arrivals: u32, interval_hours: f64, lambda_l: f64) -> PersistentTimestamp {
        entry.last_updated = PersistentTimestamp::new(0, 0);
        let update = LedgerUpdate::new(true, 0.0, 0.0, Action::Allow, true);
        let mut at = PersistentTimestamp::new(0, 0);
        for step in 1..=arrivals {
            let seconds = (f64::from(step) * interval_hours * 3600.0).round() as i64;
            at = PersistentTimestamp::new(seconds, 0);
            entry.apply_write_decay_and_update(GAMMA_T_LEDGER, &at, &update, lambda_l);
        }
        at
    }

    /// Extracts at `coord` against `ledger`, reading it at `at` under `criteria`.
    fn extract_at(
        ledger: &SentinelLedger,
        coord: u128,
        at: PersistentTimestamp,
        criteria: ImmaturityCriteria,
    ) -> (SentinelExtraction, SentinelAlarmSummary) {
        let index = root_only_index();
        let batch_now = std::time::Instant::now();
        let config = ExtractionConfig::default();
        extract_sentinel(
            &index,
            ledger,
            coord,
            GAMMA_T_LEDGER,
            &at,
            &batch_now,
            &[],
            &config,
            &criteria,
            false,
        )
    }

    /// Where a region has been split, the outcome features come from the
    /// deepest tracked cell containing the coordinate rather than from the root
    /// above it: a request landing inside a cell with its own adverse history
    /// carries that history into the row, not the diluted average of the whole
    /// space. Depth is specificity, and the one thing the Ledger can tell a
    /// model that no current measurement can — that this particular
    /// neighbourhood turned out badly — survives only if the read reaches the
    /// neighbourhood's own entry.
    ///
    /// ´claim:extract:ledger-features-come-from-the-deepest-tracked-cell-containing-the-coordinate´
    /// ´test:unit:ledger-read-takes-the-deepest-covering-entry´
    #[test]
    fn ledger_read_takes_the_deepest_covering_entry() {
        let at = PersistentTimestamp::new(1_000, 0);
        let ledger = split_region_ledger(at);

        let (extraction, _alarm) = extract_at(&ledger, COORD_IN_SPLIT, at, shipped_criteria());

        assert_eq!(
            ledger_history(&extraction),
            DEEP_HISTORY,
            "the row carries the deep cell's own history"
        );
        assert!(
            (ledger_history(&extraction).0 - ROOT_HISTORY.0).abs() > 0.5,
            "and not the root's, which the two histories are chosen far apart to distinguish"
        );
    }

    /// A coordinate no tracked cell covers still reads: the walk falls back
    /// through the depths to the root, which contains every coordinate, and the
    /// row carries the root's history. The Ledger read cannot fail, so a
    /// request outside every split region is answered from coarser memory
    /// rather than refused or given a neutral placeholder that would read as
    /// measured innocence.
    ///
    /// ´claim:extract:a-coordinate-no-tracked-cell-covers-reads-the-root-so-the-ledger-lookup-cannot-fail´
    /// ´test:unit:ledger-read-falls-back-to-the-root´
    #[test]
    fn ledger_read_falls_back_to_the_root() {
        let at = PersistentTimestamp::new(1_000, 0);
        let ledger = split_region_ledger(at);

        let (extraction, _alarm) = extract_at(&ledger, COORD_OUTSIDE_SPLIT, at, shipped_criteria());

        assert!(extraction.occupancy, "the extraction is complete rather than refused");
        assert_eq!(
            ledger_history(&extraction),
            ROOT_HISTORY,
            "the walk fell back past the depth-eight cell that does not contain the coordinate"
        );
    }

    /// A deep cell holding fewer eligible labels than the materiality threshold
    /// raises the immaturity flag on its own account, while the root above it —
    /// holding plenty — does not. Lowering only the threshold, with the same
    /// evidence and the same floor, puts the flag down again, so what raised it
    /// was the count and not the arrival rate. A host discounting an immature
    /// reading needs the flag to be about the cell that answered the request,
    /// and a cell with too few labels has measured nothing to weight.
    ///
    /// ´claim:extract:the-immaturity-flag-raises-where-the-covering-cell-holds-too-few-eligible-labels´
    /// ´test:unit:immaturity-raises-for-a-thin-deep-cell´
    #[test]
    fn immaturity_raises_for_a_thin_deep_cell() {
        const LAMBDA_L: f64 = 0.9;
        const FLOOR: f64 = 0.25;

        let mut deep = LedgerEntry::new_neutral();
        let at = drive_eligible(&mut deep, 10, 0.0, LAMBDA_L);

        let mut root = LedgerEntry::new_neutral();
        let root_at = drive_eligible(&mut root, 200, 0.0, LAMBDA_L);
        assert_eq!(root_at, at, "both cells are driven at one instant");

        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();
        *ledger.root_mut() = root;
        ledger.insert(LedgerKey::new(0, SPLIT_DEPTH), deep);

        // Ten labels against a hundred: the count arm holds at the deep cell
        // and not at the root, whose two hundred clear the same threshold.
        let (_, alarm) = extract_at(&ledger, COORD_IN_SPLIT, at, criteria(100, FLOOR, LAMBDA_L));
        assert!(alarm.ledger_immature, "ten eligible labels against a threshold of a hundred");

        // The same evidence under a threshold of five: nothing else changed,
        // and the flag is down, so the arrival rate never held.
        let (_, alarm) = extract_at(&ledger, COORD_IN_SPLIT, at, criteria(5, FLOOR, LAMBDA_L));
        assert!(!alarm.ledger_immature, "ten eligible labels against a threshold of five");
    }

    /// A deep cell holding enough eligible labels still raises the flag when
    /// its labels arrive too sparsely: a window fed once a year has decayed
    /// almost to a single arrival by the time the next comes, and the
    /// steady-state attenuation that window implies — about a tenth — stands
    /// below the configured floor. Lowering only the floor beneath that
    /// attenuation puts the flag down, so what raised it was the arrival rate
    /// and not the count. The root above it, holding a dense window under that
    /// same floor, stays mature, so the raise belongs to the cell that answered
    /// rather than to the Ledger. The two conditions are independent, and a cell
    /// whose true rate is high but whose evidence arrives too slowly to hold the
    /// excess in an average is exactly the case an average alone cannot report.
    ///
    /// ´claim:extract:the-immaturity-flag-raises-where-the-covering-cells-arrival-rate-implies-attenuation-below-the-floor´
    /// ´test:unit:immaturity-raises-for-an-attenuation-starved-cell´
    #[test]
    fn immaturity_raises_for_an_attenuation_starved_cell() {
        const LAMBDA_L: f64 = 0.9;
        const THRESHOLD: u64 = 5;
        // A year between arrivals, which is where the decay clamp puts the
        // longest interval the Ledger will honour (´def:ledger:time-decay´).
        const SPARSE_INTERVAL_HOURS: f64 = crate::types::MAX_DECAY_HOURS;

        let mut deep = LedgerEntry::new_neutral();
        let at = drive_eligible(&mut deep, 10, SPARSE_INTERVAL_HOURS, LAMBDA_L);
        let attenuation = deep
            .steady_state_attenuation(GAMMA_T_LEDGER, &at, LAMBDA_L)
            .expect("a driven window names an attenuation");
        assert!(
            (attenuation - 0.1).abs() < 0.01,
            "a window decayed to one arrival reads about a tenth: {attenuation}"
        );

        let mut root = LedgerEntry::new_neutral();
        root.last_updated = at;
        let update = LedgerUpdate::new(true, 0.0, 0.0, Action::Allow, true);
        for _ in 0..200 {
            root.apply_write_decay_and_update(GAMMA_T_LEDGER, &at, &update, LAMBDA_L);
        }

        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();
        *ledger.root_mut() = root;
        ledger.insert(LedgerKey::new(0, SPLIT_DEPTH), deep);

        // Ten labels clear the threshold of five, so only the rate arm can
        // raise the flag.
        let (_, alarm) = extract_at(&ledger, COORD_IN_SPLIT, at, criteria(THRESHOLD, 0.25, LAMBDA_L));
        assert!(
            alarm.ledger_immature,
            "an attenuation of a tenth against a floor of a quarter"
        );

        // The root's window is dense under the same floor, so the raise above
        // is the deep cell's own sparseness and not a condition of the Ledger.
        let (_, alarm) = extract_at(&ledger, COORD_OUTSIDE_SPLIT, at, criteria(THRESHOLD, 0.25, LAMBDA_L));
        assert!(
            !alarm.ledger_immature,
            "two hundred labels arriving at one instant clear the same floor"
        );

        // The same evidence under a floor the attenuation clears.
        let (_, alarm) = extract_at(&ledger, COORD_IN_SPLIT, at, criteria(THRESHOLD, 0.05, LAMBDA_L));
        assert!(
            !alarm.ledger_immature,
            "an attenuation of a tenth against a floor of a twentieth"
        );
    }

    /// A deep cell clearing both conditions leaves the flag down even where the
    /// root above it would raise it: the request lands in a cell with labels
    /// enough and arrivals dense enough, and the thin root it is nested in has
    /// no say. The flag is a statement about the entry the read actually
    /// reached, so a mature neighbourhood inside a young Sentinel is not
    /// discounted for the Sentinel's youth.
    ///
    /// ´claim:extract:the-immaturity-flag-stays-down-for-a-mature-covering-cell-whatever-the-root-holds´
    /// ´test:unit:immaturity-stays-down-for-a-mature-deep-cell´
    #[test]
    fn immaturity_stays_down_for_a_mature_deep_cell() {
        const LAMBDA_L: f64 = 0.9;
        const THRESHOLD: u64 = 5;
        const FLOOR: f64 = 0.25;

        let mut deep = LedgerEntry::new_neutral();
        let at = drive_eligible(&mut deep, 10, 0.0, LAMBDA_L);

        // A root thinner than the threshold, so a flag reading the root would
        // raise and a flag reading the deep cell will not.
        let mut root = LedgerEntry::new_neutral();
        drive_eligible(&mut root, 2, 0.0, LAMBDA_L);

        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();
        *ledger.root_mut() = root;
        ledger.insert(LedgerKey::new(0, SPLIT_DEPTH), deep);

        let (_, alarm) = extract_at(&ledger, COORD_IN_SPLIT, at, criteria(THRESHOLD, FLOOR, LAMBDA_L));
        assert!(!alarm.ledger_immature, "the deep cell clears both conditions");

        // The same Ledger read where no tracked cell covers the coordinate: the
        // walk reaches the thin root, and the flag raises on its two labels.
        let (_, alarm) = extract_at(&ledger, COORD_OUTSIDE_SPLIT, at, criteria(THRESHOLD, FLOOR, LAMBDA_L));
        assert!(
            alarm.ledger_immature,
            "the fallback root holds two labels against a threshold of five"
        );
    }
}
