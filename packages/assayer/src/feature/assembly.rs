// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![allow(dead_code)]

//! Feature vector assembly from extracted features.
//!
//! Assembles the full feature vector φ from individual components:
//! - Aggregate features (15)
//! - Identity features (8 + 3 outcome-axis features per dimension, plus an 8-feature cross-dimension block)
//! - Signal features
//! - Sentinel slot features (occupancy + extraction)
//! - Interaction features (computed from base features)
//! - Competitive cell indicators
//!
//! # Assembly Order
//!
//! 1. Fill base blocks (bias, aggregates, identity, signals, Sentinel slots)
//! 2. Fill competitive indicators
//! 3. Compute interactions from unstandardised base values
//! 4. Standardise the full vector
//!
//! # Tests
//!
//! Crate-level tests: `src/tests/assembly.rs`
//!
//! # Cross-References
//!
//! - (´dec:vector:unstandardised-bases´) — assessment-time assembly:
//!   interactions from raw operands, then one standardisation pass over the
//!   whole vector
//! - (´dec:vector:label-time-assembly´) — label-time assembly: what was fixed
//!   is restored, what has since moved is re-derived
//! - (´def:feature:vector-structure´) — the eight block types and the fixed
//!   order this module fills

use std::collections::HashMap;

use crate::feature::aggregate::SentinelSubScores;
use crate::feature::dimension_map::{
    DimensionMap, IndexRange, SENTINEL_OCCUPANCY_WIDTH, SENTINEL_Q_BASE, SENTINEL_SPATIAL_FEATURES_PER_AXIS,
};
use crate::feature::interaction::compute_interactions;
use crate::feature::permutation::EXTRACTION_AXIS_PAIR_OFFSET;
use crate::feature::standardisation::{StandardisationConfig, standardise_in_place};
use crate::identity::CompetitiveCellId;
use crate::pending::{SentinelExtraction, StoredFeatures};
use crate::types::{DimensionId, OutcomeAxisId, SCORING_AXIS_COUNT, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Helper Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Identity aggregate features for one dimension.
///
/// 3 features per dimension: `coverage_depth`, `active_indicator_fraction`,
/// `total_dimension_importance`.
#[derive(Clone, Debug, Default)]
pub struct IdentityAggregateFeatures {
    /// Coverage depth: fraction of coordinate space with active cells.
    pub coverage_depth: f64,
    /// Active indicator fraction: fraction of registered cells that are active.
    pub active_indicator_fraction: f64,
    /// Total dimension importance: sum of cell importances.
    pub total_dimension_importance: f64,
}

impl IdentityAggregateFeatures {
    /// Creates identity aggregates with the specified values.
    #[must_use]
    pub const fn new(coverage_depth: f64, active_indicator_fraction: f64, total_dimension_importance: f64) -> Self {
        Self {
            coverage_depth,
            active_indicator_fraction,
            total_dimension_importance,
        }
    }

    /// Returns the features as a slice for filling into φ.
    #[must_use]
    pub const fn as_slice(&self) -> [f64; 3] {
        [
            self.coverage_depth,
            self.active_indicator_fraction,
            self.total_dimension_importance,
        ]
    }
}

/// Per-dimension measurement features.
///
/// 5 features for each identity dimension.
#[derive(Clone, Debug, Default)]
pub struct IdentityMeasurementFeatures {
    /// Maximum alarm across active cells.
    pub max_alarm: f64,
    /// Mean alarm across active cells.
    pub mean_alarm: f64,
    /// Maximum suspicion across active cells.
    pub max_suspicion: f64,
    /// Volatility of the deepest active cell.
    pub volatility_deepest: f64,
    /// Binary indicator: 1.0 if any competitive cell is active in the dimension.
    pub has_competitive_cell: f64,
}

impl IdentityMeasurementFeatures {
    /// Creates measurement features with the specified values.
    #[must_use]
    pub const fn new(
        max_alarm: f64,
        mean_alarm: f64,
        max_suspicion: f64,
        volatility_deepest: f64,
        has_competitive_cell: f64,
    ) -> Self {
        Self {
            max_alarm,
            mean_alarm,
            max_suspicion,
            volatility_deepest,
            has_competitive_cell,
        }
    }

    /// Returns the features as a slice for filling into φ.
    #[must_use]
    pub const fn as_slice(&self) -> [f64; 5] {
        [
            self.max_alarm,
            self.mean_alarm,
            self.max_suspicion,
            self.volatility_deepest,
            self.has_competitive_cell,
        ]
    }
}

/// Complete per-dimension identity feature inputs for assessment assembly.
#[derive(Clone, Debug, Default)]
pub struct IdentityDimensionFeatures {
    /// Structural aggregate features for this dimension.
    pub aggregate: IdentityAggregateFeatures,
    /// Measurement features for this dimension.
    pub measurement: IdentityMeasurementFeatures,
    /// Per-axis outcome features for this dimension.
    pub axis_features: HashMap<OutcomeAxisId, IdentityAxisFeatures>,
}

impl IdentityDimensionFeatures {
    /// Creates a complete per-dimension identity feature set.
    #[must_use]
    pub const fn new(
        aggregate: IdentityAggregateFeatures,
        measurement: IdentityMeasurementFeatures,
        axis_features: HashMap<OutcomeAxisId, IdentityAxisFeatures>,
    ) -> Self {
        Self {
            aggregate,
            measurement,
            axis_features,
        }
    }

    /// Returns the fixed 8-feature prefix for this dimension.
    #[must_use]
    pub const fn fixed_prefix(&self) -> [f64; 8] {
        let aggregate = self.aggregate.as_slice();
        let measurement = self.measurement.as_slice();
        [
            aggregate[0],
            aggregate[1],
            aggregate[2],
            measurement[0],
            measurement[1],
            measurement[2],
            measurement[3],
            measurement[4],
        ]
    }
}

/// Cross-dimension aggregate identity features.
#[derive(Clone, Debug, Default)]
pub struct CrossDimensionAggregateFeatures {
    /// Maximum suspicion across all active identity cells.
    pub max_suspicion: f64,
    /// Maximum alarm across all active identity cells.
    pub max_alarm: f64,
    /// Maximum volatility across dimensions.
    pub max_volatility: f64,
    /// Maximum adverse rate across active identity cells.
    pub max_adverse_rate: f64,
    /// Maximum absolute compressed valence across active identity cells.
    pub max_abs_compressed_valence: f64,
    /// Maximum absolute raw valence across active identity cells.
    pub max_abs_raw_valence: f64,
    /// Binary indicator: any competitive cell is active in any dimension.
    pub has_any_competitive_cell: f64,
    /// Maximum coverage depth across dimensions.
    pub max_coverage_depth: f64,
}

impl CrossDimensionAggregateFeatures {
    /// Returns the cross-dimension block as an ordered feature array.
    #[must_use]
    pub const fn as_slice(&self) -> [f64; 8] {
        [
            self.max_suspicion,
            self.max_alarm,
            self.max_volatility,
            self.max_adverse_rate,
            self.max_abs_compressed_valence,
            self.max_abs_raw_valence,
            self.has_any_competitive_cell,
            self.max_coverage_depth,
        ]
    }
}

/// Per-axis identity outcome features for one dimension.
///
/// 3 features per registered outcome axis per identity dimension: mean
/// compressed EWMA, mean raw EWMA, and compressed-EWMA stability across
/// active competitive cells in that dimension.
#[derive(Clone, Debug, Default)]
pub struct IdentityAxisFeatures {
    /// Mean compressed outcome-axis EWMA across active cells.
    pub mean_compressed_axis_ewma: f64,
    /// Mean raw outcome-axis EWMA across active cells.
    pub mean_raw_axis_ewma: f64,
    /// Standard deviation of compressed outcome-axis EWMAs across active cells.
    pub axis_stability: f64,
}

impl IdentityAxisFeatures {
    /// Creates per-axis identity features with the specified values.
    #[must_use]
    pub const fn new(mean_compressed_axis_ewma: f64, mean_raw_axis_ewma: f64, axis_stability: f64) -> Self {
        Self {
            mean_compressed_axis_ewma,
            mean_raw_axis_ewma,
            axis_stability,
        }
    }

    /// Returns the features as a slice for filling into φ.
    #[must_use]
    pub const fn as_slice(&self) -> [f64; 3] {
        [self.mean_compressed_axis_ewma, self.mean_raw_axis_ewma, self.axis_stability]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Block Filling Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Fills a range in φ from a slice of values.
///
/// # Panics
///
/// Debug panics if the slice length doesn't match the range length.
#[inline]
fn fill_range(phi: &mut [f64], start: usize, values: &[f64]) {
    debug_assert!(start + values.len() <= phi.len(), "fill_range out of bounds");
    phi[start..start + values.len()].copy_from_slice(values);
}

/// Marks a contiguous range as covered in the debug coverage tracker.
#[cfg(debug_assertions)]
#[inline]
fn mark_covered(coverage: &mut [bool], start: usize, end: usize) {
    coverage[start..end].fill(true);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Core Assessment Assembly
// ═══════════════════════════════════════════════════════════════════════════════

/// Assembles the raw (unstandardised) feature vector φ for one assessment.
///
/// Contains all blocks (bias, aggregates, identity, signals, Sentinel slots,
/// competitive indicators, interactions) but does NOT apply standardisation.
/// This is the vector the cold ramp's offer step is offered
/// (´alg:standardisation:batch-initialisation´).
///
/// # Arguments
///
/// * `dim_map` — The dimension map defining feature layout
/// * `sentinel_extractions` — Per-Sentinel extracted features (keyed by ID)
/// * `aggregate_features` — The 15 aggregate features
/// * `identity_features` — Per-dimension identity feature blocks
/// * `cross_dimension_features` — Cross-dimension identity aggregate features
/// * `signal_features` — Signal features block
/// * `active_cells` — Active competitive cells per dimension
///
/// # Returns
///
/// The assembled but unstandardised feature vector φ.
///
/// # Cross-References
///
/// - (´dec:vector:unstandardised-bases´) — why the interactions here are
///   products of raw operands and nothing is standardised twice
/// - (´alg:standardisation:batch-initialisation´) — the offer step this raw
///   vector serves, between the interactions and the standardisation
#[must_use]
pub fn assemble_assessment_raw(
    dim_map: &DimensionMap,
    sentinel_extractions: &HashMap<SentinelId, SentinelExtraction>,
    aggregate_features: &[f64; 15],
    identity_features: &HashMap<DimensionId, IdentityDimensionFeatures>,
    cross_dimension_features: &CrossDimensionAggregateFeatures,
    signal_features: &[f64],
    active_cells: &HashMap<DimensionId, Vec<CompetitiveCellId>>,
) -> Vec<f64> {
    let p = dim_map.p;
    let mut phi = vec![0.0_f64; p];

    #[cfg(debug_assertions)]
    let mut coverage = vec![false; p];

    // ─── Block 1: Bias ───
    phi[dim_map.bias_idx] = 1.0;
    #[cfg(debug_assertions)]
    {
        coverage[dim_map.bias_idx] = true;
    }

    // ─── Block 2: Aggregates (15 features) ───
    fill_range(&mut phi, dim_map.agg_range.start, aggregate_features);
    #[cfg(debug_assertions)]
    mark_covered(&mut coverage, dim_map.agg_range.start, dim_map.agg_range.end);

    // ─── Block 3: Per-dimension identity features ───
    for (dim_id, range) in &dim_map.id_dim_ranges {
        if let Some(features) = identity_features.get(dim_id) {
            fill_range(&mut phi, range.start, &features.fixed_prefix());
            for (axis_id, axis_features) in &features.axis_features {
                if let Some(offset) = dim_map.identity_axis_offset(*axis_id) {
                    fill_range(&mut phi, range.start + offset, &axis_features.as_slice());
                }
            }
        }
        // Absent dimension → zeros (already initialised)
        #[cfg(debug_assertions)]
        mark_covered(&mut coverage, range.start, range.end);
    }

    // ─── Block 4: Cross-dimension identity aggregate features ───
    if let Some(ref range) = dim_map.id_cross_dim_range {
        fill_range(&mut phi, range.start, &cross_dimension_features.as_slice());
        #[cfg(debug_assertions)]
        mark_covered(&mut coverage, range.start, range.end);
    }

    // ─── Block 5: Signal features ───
    if !dim_map.sig_range.is_empty() {
        debug_assert_eq!(
            signal_features.len(),
            dim_map.sig_range.len(),
            "Signal features length mismatch"
        );
        fill_range(&mut phi, dim_map.sig_range.start, signal_features);
        #[cfg(debug_assertions)]
        mark_covered(&mut coverage, dim_map.sig_range.start, dim_map.sig_range.end);
    }

    // ─── Block 6: Sentinel slots (occupancy + features) ───
    for (sentinel_id, slot_range) in &dim_map.sentinel_slots {
        if let Some(extraction) = sentinel_extractions.get(sentinel_id) {
            // Occupancy indicator
            phi[slot_range.start] = if extraction.occupancy { 1.0 } else { 0.0 };
            // Features (upcast to f64 at the stored precision)
            for (i, f) in extraction.features.iter().enumerate() {
                phi[slot_range.start + 1 + i] = f;
            }
        }
        // Absent Sentinel → zeros (occupancy = 0, features = 0)
        #[cfg(debug_assertions)]
        mark_covered(&mut coverage, slot_range.start, slot_range.end);
    }

    // ─── Block 7: Competitive indicators ───
    for (dim_id, cell_map) in &dim_map.competitive_indices {
        let active = active_cells.get(dim_id);
        for (cell_id, &idx) in cell_map {
            let is_active = active.is_some_and(|cells| cells.contains(cell_id));
            phi[idx] = if is_active { 1.0 } else { 0.0 };
            #[cfg(debug_assertions)]
            {
                coverage[idx] = true;
            }
        }
    }

    // ─── Block 8: Interaction features (computed from unstandardised base) ───
    compute_interactions(&mut phi, dim_map);
    #[cfg(debug_assertions)]
    for &idx in dim_map.interaction_indices.values() {
        coverage[idx] = true;
    }

    // ─── Debug: verify write coverage ───
    #[cfg(debug_assertions)]
    {
        let uncovered_nonzero: Vec<usize> = coverage
            .iter()
            .enumerate()
            .filter(|(i, c)| !*c && phi[*i] != 0.0)
            .map(|(i, _)| i)
            .collect();
        debug_assert!(
            uncovered_nonzero.is_empty(),
            "Uncovered non-zero φ positions: {uncovered_nonzero:?}"
        );
    }

    phi
}

/// Assembles and standardises φ for one assessment request.
///
/// Convenience wrapper around [`assemble_assessment_raw`] followed by
/// [`standardise_in_place`]. Callers that need the raw φ for batch
/// init observation (´alg:standardisation:batch-initialisation´) should call
/// `assemble_assessment_raw`
/// directly.
///
/// # Cross-References
///
/// - (´dec:vector:unstandardised-bases´) — the single standardisation pass this
///   wrapper adds after the raw assembly
#[must_use]
#[allow(clippy::too_many_arguments)] // Justified: each parameter is a distinct data source for one block of φ
pub fn assemble_and_standardise_assessment(
    dim_map: &DimensionMap,
    sentinel_extractions: &HashMap<SentinelId, SentinelExtraction>,
    aggregate_features: &[f64; 15],
    identity_features: &HashMap<DimensionId, IdentityDimensionFeatures>,
    cross_dimension_features: &CrossDimensionAggregateFeatures,
    signal_features: &[f64],
    active_cells: &HashMap<DimensionId, Vec<CompetitiveCellId>>,
    means: &[f64],
    variances: &[f64],
    config: &StandardisationConfig,
) -> Vec<f64> {
    let mut phi = assemble_assessment_raw(
        dim_map,
        sentinel_extractions,
        aggregate_features,
        identity_features,
        cross_dimension_features,
        signal_features,
        active_cells,
    );
    standardise_in_place(&mut phi, means, variances, config.epsilon);
    phi
}

/// Extracts sub-scores from Sentinel extractions for aggregate computation.
///
/// Used to convert `SentinelExtraction` to `SentinelSubScores` for
/// `compute_aggregates()`.
///
/// # Arguments
///
/// * `extractions` — Per-Sentinel extracted features
///
/// # Returns
///
/// A vector of (`SentinelId`, `SentinelSubScores`) for reporting Sentinels.
#[must_use]
pub fn extract_sub_scores(extractions: &HashMap<SentinelId, SentinelExtraction>) -> Vec<(SentinelId, SentinelSubScores)> {
    extractions
        .iter()
        .filter(|(_, ext)| ext.occupancy)
        .map(|(&sid, ext)| {
            // Extract cell-level z-scores (indices 0-3) and CUSUMs (indices derived from chain)
            // Feature layout per extraction:
            //   [0-3]: Cell z-scores (N, D, S, C)
            //   [24-27]: Cell CUSUMs (N, D, S, C) (after z-score views)
            // The four reads stay written out rather than driven off the arity: at a
            // different arity they must fail to compile, because the CUSUM offset
            // below would need moving too and a loop would silently not move it.
            let z_scores = if ext.features.len() >= SCORING_AXIS_COUNT {
                [
                    ext.features.value(0),
                    ext.features.value(1),
                    ext.features.value(2),
                    ext.features.value(3),
                ]
            } else {
                [0.0; SCORING_AXIS_COUNT]
            };

            // CUSUMs are at indices after z-scores (24 z-scores + 12 CUSUMs layout)
            // Cell CUSUMs are at indices 24-27 (first 4 of the 12 CUSUM features)
            let cusums = if ext.features.len() >= 28 {
                [
                    ext.features.value(24),
                    ext.features.value(25),
                    ext.features.value(26),
                    ext.features.value(27),
                ]
            } else {
                [0.0; SCORING_AXIS_COUNT]
            };

            (sid, SentinelSubScores::from_values(z_scores, cusums))
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Label-Time Assembly
// ═══════════════════════════════════════════════════════════════════════════════

/// The frozen half of a label-time reconstruction: what the pending entry
/// stored at assessment time (´def:runtime:pending-entry´).
///
/// Sources one, three and four of (´alg:runtime:reconstruction´) read
/// these from storage — the signal features and per-dimension identity
/// base features, the per-axis identity features, and the extractions
/// with the two Sentinel lists that tell an active-but-silent Sentinel
/// from one registered after the assessment. No call can supply any of
/// them from state the label path holds.
#[derive(Debug)]
pub struct FrozenPendingView<'a> {
    /// Per-Sentinel extractions stored at assessment time.
    pub extractions: &'a HashMap<SentinelId, SentinelExtraction>,
    /// Sentinels registered at assessment time.
    pub active_sentinels: &'a [SentinelId],
    /// Sentinels actually reporting at assessment time.
    pub reporting_sentinels: &'a [SentinelId],
    /// Signal features stored at assessment time (upcast to f64).
    pub signals: &'a [f64],
    /// Frozen per-dimension identity base features: the five measurement
    /// values behind each dimension block's structural triple.
    pub base_features: &'a HashMap<DimensionId, StoredFeatures>,
    /// Frozen per-dimension, per-axis identity features.
    pub axis_features: &'a HashMap<DimensionId, HashMap<OutcomeAxisId, StoredFeatures>>,
    /// The spatially enabled outcome axes the stored extractions were laid
    /// out against, in layout order (´def:extraction:slot´).
    ///
    /// The extractions above are flat vectors whose outcome-memory pairs carry
    /// no identity of their own, so this list is the only thing that says which
    /// axis owns the pair at a given rank. Source four matches it against the
    /// current layout's own ordering rather than assuming the two agree.
    pub spatial_axis_ids: &'a [OutcomeAxisId],
}

/// The current half of a label-time reconstruction: what sources two,
/// five and six of (´alg:runtime:reconstruction´) re-derive from the
/// working copy at the moment of labelling.
#[derive(Debug)]
pub struct CurrentLabelState<'a> {
    /// Aggregate features recomputed from the stored extractions over
    /// the currently registered Sentinels.
    pub aggregate_features: &'a [f64; 15],
    /// Per-dimension structural triples re-derived from the current
    /// competitive set.
    pub identity_aggregates: &'a HashMap<DimensionId, IdentityAggregateFeatures>,
    /// Cross-dimension aggregates re-derived from current cell state.
    pub cross_dimension_features: &'a CrossDimensionAggregateFeatures,
    /// Current active competitive cells for the re-encoded coordinate.
    pub active_cells: &'a HashMap<DimensionId, Vec<CompetitiveCellId>>,
}

/// Places one frozen extraction into its Sentinel's slot, group by group and
/// pair by axis.
///
/// The extraction is six groups in a fixed order and the per-axis
/// outcome-memory pairs are the fifth, not the last: the batch context feature
/// follows them (´def:extraction:slot´). A prefix copy preserves the width and
/// loses the alignment (´entry:assayer:wl-feature-frozen-slot-truncation´) — on
/// a registration it stopped two short, so the batch context was read at the
/// position the new pair now owns while its own position took zero; on a
/// deregistration it dropped the batch context and half a pair off the end.
///
/// Placed by group instead: the fixed run ahead of the pairs copies as itself,
/// and the batch context is placed last on both sides. The pairs between them
/// are matched by axis identity rather than by rank. Each stored pair is
/// carried to the position the current layout gives *that axis*, which the map
/// answers directly (`DimensionMap::slot_axis_offset`); a stored pair whose
/// axis is no longer spatial has no such position and is dropped; a currently
/// spatial axis the assessment never held is named by nothing stored and keeps
/// its zeros.
///
/// Rank matching was correct only while every lifecycle event happened at the
/// end of the tail. Retire an axis from the middle and the ranks behind it all
/// shift by one, so every survivor would read its neighbour's memory and the
/// retired axis's memory would be donated to whichever axis inherited its rank
/// — including one registered after the retirement, which never held that
/// evidence at all. Identity matching has no such case: a position is found by
/// asking who owns it, and an axis that owns nothing is told so.
fn place_frozen_extraction(
    phi: &mut [f64],
    slot_range: &IndexRange,
    extraction: &SentinelExtraction,
    stored_spatial_axes: &[OutcomeAxisId],
    dim_map: &DimensionMap,
) {
    let features_start = slot_range.start + SENTINEL_OCCUPANCY_WIDTH;
    let slot_feature_len = slot_range.end - features_start;
    let stored_len = extraction.features.len();

    let fixed_len = EXTRACTION_AXIS_PAIR_OFFSET.min(stored_len).min(slot_feature_len);
    for i in 0..fixed_len {
        phi[features_start + i] = extraction.features.value(i);
    }

    // How many pairs the stored vector physically holds, and how many the
    // entry names. They agree on every entry this build writes; taking the
    // smaller means a disagreement reads short rather than off the end.
    let stored_pairs = stored_len.saturating_sub(SENTINEL_Q_BASE) / SENTINEL_SPATIAL_FEATURES_PER_AXIS;
    for (rank, &axis_id) in stored_spatial_axes.iter().take(stored_pairs).enumerate() {
        // No offset for this axis means it is no longer spatial. Its pair is
        // dropped rather than handed to whoever holds its old rank.
        let Some(slot_offset) = dim_map.slot_axis_offset(axis_id) else {
            continue;
        };
        let target = slot_range.start + slot_offset;
        if target + SENTINEL_SPATIAL_FEATURES_PER_AXIS > slot_range.end {
            continue;
        }
        let source = EXTRACTION_AXIS_PAIR_OFFSET + rank * SENTINEL_SPATIAL_FEATURES_PER_AXIS;
        for half in 0..SENTINEL_SPATIAL_FEATURES_PER_AXIS {
            phi[target + half] = extraction.features.value(source + half);
        }
    }

    let current_pairs = slot_feature_len.saturating_sub(SENTINEL_Q_BASE) / SENTINEL_SPATIAL_FEATURES_PER_AXIS;
    let stored_batch = EXTRACTION_AXIS_PAIR_OFFSET + stored_pairs * SENTINEL_SPATIAL_FEATURES_PER_AXIS;
    let current_batch = EXTRACTION_AXIS_PAIR_OFFSET + current_pairs * SENTINEL_SPATIAL_FEATURES_PER_AXIS;
    if stored_batch < stored_len && current_batch < slot_feature_len {
        phi[features_start + current_batch] = extraction.features.value(stored_batch);
    }
}

/// Reconstructs the raw (unstandardised) φ for label-time model update.
///
/// Fills all six sources of (´alg:runtime:reconstruction´), every index
/// claimed by exactly one:
///
/// 1. The bias, the frozen signal block, and the frozen per-dimension
///    identity base features.
/// 2. Per currently registered dimension: the structural triple and the
///    competitive indicators from the current set, with the
///    cross-dimension aggregates beside them.
/// 3. The frozen per-axis identity features where the axis existed at
///    assessment time, zeros where it was registered afterwards.
/// 4. One slot per currently registered Sentinel: the stored extraction
///    where it was reporting, occupancy with zero features where it was
///    active but silent, zeros where it was registered after the
///    assessment — and no slot at all for one no longer registered.
/// 5. The aggregate block, recomputed by the caller from the stored
///    extractions.
/// 6. Interactions, recomputed from the reconstructed base.
///
/// # Cross-References
///
/// - (´dec:vector:label-time-assembly´) — the rebuild this performs, restoring
///   the frozen blocks and re-deriving the rest
#[must_use]
pub fn assemble_phi_for_label_raw(
    frozen: &FrozenPendingView<'_>,
    current: &CurrentLabelState<'_>,
    dim_map: &DimensionMap,
) -> Vec<f64> {
    let p = dim_map.p;
    let mut phi = vec![0.0_f64; p];

    #[cfg(debug_assertions)]
    let mut coverage = vec![false; p];

    // ─── Block 1: Bias ───
    phi[dim_map.bias_idx] = 1.0;
    #[cfg(debug_assertions)]
    {
        coverage[dim_map.bias_idx] = true;
    }

    // ─── Block 2: Aggregates (RE-DERIVED, source 5) ───
    fill_range(&mut phi, dim_map.agg_range.start, current.aggregate_features);
    #[cfg(debug_assertions)]
    mark_covered(&mut coverage, dim_map.agg_range.start, dim_map.agg_range.end);

    // ─── Block 3: Per-dimension identity features ───
    // The structural triple is re-derived from the current competitive
    // set (source 2); the five measurement values behind it and the
    // per-axis features are frozen from the pending entry (sources 1
    // and 3).
    for (dim_id, range) in &dim_map.id_dim_ranges {
        if let Some(agg) = current.identity_aggregates.get(dim_id) {
            fill_range(&mut phi, range.start, &agg.as_slice());
        }
        if let Some(base) = frozen.base_features.get(dim_id) {
            let base_start = range.start + 3;
            let copy_len = base.len().min(5).min(range.end.saturating_sub(base_start));
            for (i, v) in base.iter().take(copy_len).enumerate() {
                phi[base_start + i] = v;
            }
        }
        if let Some(per_axis) = frozen.axis_features.get(dim_id) {
            for (axis_id, values) in per_axis {
                // An axis no longer registered has no offset and is not
                // placed; one registered after the assessment has no
                // stored values and its positions stay zero (source 3).
                if let Some(offset) = dim_map.identity_axis_offset(*axis_id) {
                    let axis_start = range.start + offset;
                    let copy_len = values.len().min(3).min(range.end.saturating_sub(axis_start));
                    for (i, v) in values.iter().take(copy_len).enumerate() {
                        phi[axis_start + i] = v;
                    }
                }
            }
        }
        // Absent dimension → zeros (already initialised).
        #[cfg(debug_assertions)]
        mark_covered(&mut coverage, range.start, range.end);
    }

    // ─── Block 4: Cross-dimension identity aggregates (RE-DERIVED, source 2) ───
    if let Some(ref range) = dim_map.id_cross_dim_range {
        fill_range(&mut phi, range.start, &current.cross_dimension_features.as_slice());
        #[cfg(debug_assertions)]
        mark_covered(&mut coverage, range.start, range.end);
    }

    // ─── Block 5: Signal features (FROZEN, source 1) ───
    if !dim_map.sig_range.is_empty() {
        debug_assert_eq!(
            frozen.signals.len(),
            dim_map.sig_range.len(),
            "Signal features length mismatch"
        );
        fill_range(&mut phi, dim_map.sig_range.start, frozen.signals);
        #[cfg(debug_assertions)]
        mark_covered(&mut coverage, dim_map.sig_range.start, dim_map.sig_range.end);
    }

    // ─── Block 6: Sentinel slots (FROZEN, source 4, three ways) ───
    for (sentinel_id, slot_range) in &dim_map.sentinel_slots {
        let was_reporting = frozen.reporting_sentinels.contains(sentinel_id);
        let was_active = frozen.active_sentinels.contains(sentinel_id);
        if let Some(extraction) = frozen.extractions.get(sentinel_id).filter(|_| was_reporting) {
            // Reporting at assessment time: the stored extraction.
            phi[slot_range.start] = if extraction.occupancy { 1.0 } else { 0.0 };

            place_frozen_extraction(&mut phi, slot_range, extraction, frozen.spatial_axis_ids, dim_map);
        } else if was_active {
            // Active but silent at assessment time: occupancy, zero
            // features.
            phi[slot_range.start] = 1.0;
        }
        // Registered after the assessment → zeros.
        #[cfg(debug_assertions)]
        mark_covered(&mut coverage, slot_range.start, slot_range.end);
    }

    // ─── Block 7: Competitive indicators (RE-DERIVED, source 2) ───
    for (dim_id, cell_map) in &dim_map.competitive_indices {
        let active = current.active_cells.get(dim_id);
        for (cell_id, &idx) in cell_map {
            let is_active = active.is_some_and(|cells| cells.contains(cell_id));
            phi[idx] = if is_active { 1.0 } else { 0.0 };
            #[cfg(debug_assertions)]
            {
                coverage[idx] = true;
            }
        }
    }

    // ─── Block 8: Interaction features (RE-DERIVED, source 6) ───
    compute_interactions(&mut phi, dim_map);
    #[cfg(debug_assertions)]
    for &idx in dim_map.interaction_indices.values() {
        coverage[idx] = true;
    }

    // ─── Debug: verify write coverage ───
    #[cfg(debug_assertions)]
    {
        let uncovered_nonzero: Vec<usize> = coverage
            .iter()
            .enumerate()
            .filter(|(i, c)| !*c && phi[*i] != 0.0)
            .map(|(i, _)| i)
            .collect();
        debug_assert!(
            uncovered_nonzero.is_empty(),
            "Label-time: uncovered non-zero φ positions: {uncovered_nonzero:?}"
        );
    }

    phi
}

/// Reconstructs and standardises φ for label-time model update.
///
/// [`assemble_phi_for_label_raw`] followed by standardisation against the
/// current working-copy statistics.
///
/// This is a distinct function from `assemble_and_standardise_assessment`
/// (´dec:vector:label-time-assembly´). Key differences:
/// - A per-Sentinel dimension-mismatch guard for stored features, whether the
///   stored row is narrower than the slot
///   (´test:crate:label-narrower-stored-row-blanks-only-the-new-axis-pair´) or
///   wider than it
///   (´test:crate:label-wider-stored-row-loses-only-the-retired-axis-pair´)
/// - No CP3 NaN audit: a non-finite frozen feature survives rather than being
///   sanitised (´test:crate:label-no-cp3-nan-survives´)
///
/// # Returns
///
/// The reconstructed and standardised feature vector φ̂.
///
/// # Cross-References
///
/// - (´dec:vector:label-time-assembly´) — the label-time reconstruction this
///   standardises against the current working copy
#[must_use]
pub fn assemble_phi_for_label(
    frozen: &FrozenPendingView<'_>,
    current: &CurrentLabelState<'_>,
    dim_map: &DimensionMap,
    means: &[f64],
    variances: &[f64],
    config: &StandardisationConfig,
) -> Vec<f64> {
    let mut phi = assemble_phi_for_label_raw(frozen, current, dim_map);

    // ─── Standardisation (working-copy statistics, no CP3 sanitisation) ───
    // A non-finite frozen feature is left to survive, on purpose
    // (´test:crate:label-no-cp3-nan-survives´).
    standardise_in_place(&mut phi, means, variances, config.epsilon);

    phi
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════
