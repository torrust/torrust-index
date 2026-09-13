// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`sentinel_extraction_clone`] | pending | An extraction copies by value: a clone carries the same coordinate, the same feature vector and the same occupancy flag, and the original is left intact beside it. Extractions are handed to both the pending buffer and the journal path, so a copy that shared or consumed the features would couple two lifetimes that must stay independent. |
//! | [`pending_risk_basis_default`] | pending | A default risk basis carries zeros in every scalar — probability, uncertainty and anchor weight alike. An entry constructed before the model has said anything therefore claims no prior belief rather than inheriting whatever residue happened to sit in memory. |
//! | [`pending_assessment_to_context_preserves_core_fields`] | pending | Converting an in-memory entry to its journal context keeps everything replay depends on: the entity key, the merged signal features, the per-Sentinel extractions, the per-dimension coordinates and active cells, and the assessment-time scalars — the risk basis value for value and the per-axis predictions — all arrive unchanged. The entry that reaches the journal is a whole submission depending on nothing but itself (´dec:durability:checkpoint-journal´), so a field quietly lost in the conversion would be a step a replayed label cannot run. |
//! | [`to_context_strips_transient_fields`] | pending | What the journal context omits is only what is owned elsewhere: the type has no `id` or `timestamp` field (the journal entry carries its own), no `degradation` field (no label step reads it), and no `report_origin` field (an instant in a monotonic domain the restart ends), so the omission is enforced by the type system rather than by a copying routine remembering. The model-derived scalars are not among the omissions — they cross with the entry, because a replayed label must apply what the original applied rather than recompute against a restored model (´cor:durability:replay-exactness´). |
//! | [`pending_context_serde_roundtrip`] | pending | A journal context survives a round trip through JSON with its feature values bit-for-bit intact, negatives and fractions included. The stored features are the only record of what the model saw, so replay after a restart is only faithful if serialisation neither rounds them nor widens them on the way back. |
//! | [`pending_context_reconstruction_fields_round_trip`] | pending | The four reconstruction fields cross the journal boundary in both directions with their values equal: the Sentinel lists and the two frozen identity feature blocks survive serialisation and deserialisation exactly, and the restored context serialises back to a context equal to itself. These are the inputs reconstruction sources one, three and four read from storage (´alg:runtime:reconstruction´), and no call at label time can supply them from state the label path holds — so a field lost or rounded at this boundary is a reconstruction block silently zeroed or skewed after every restart. |
//! | [`storage_precision_double_keeps_what_single_quantises`] | pending | The storage precision is a real choice with a measurable difference: a value single precision cannot represent survives double-precision storage bit-for-bit and comes back from single-precision storage as its nearest single — exactly the quantisation the corpus argues is five orders of magnitude below the restandardisation error (´def:runtime:storage-precision´). The default is single, as the parameter table fixes it (´tab:config:pending-buffer´). A deployment that doubts the argument can now falsify it instead of assuming it. |
//! | [`double_precision_context_serde_roundtrip`] | pending | A double-precision context crosses the journal boundary with its values bit-for-bit intact, in both directions. The double variant exists to retain what single storage would quantise, so a journal pass that rounded it back down would silently reduce the configuration to its default (´def:runtime:storage-precision´). |
//! | [`sentinel_extraction_memory_size`] | pending | An extraction at the reference feature width stays inside a few hundred bytes, features included. The buffer holds one of these per Sentinel for every assessment still awaiting its label, so the per-entry footprint is what decides how deep the pending horizon can be before memory rather than policy sets the limit. |

//! Pending buffer entry types.
//!
//! This module provides the entry types for the pending buffer:
//!
//! - [`SentinelExtraction`] — Per-Sentinel extracted features
//! - [`PendingRiskBasis`] — Risk scalars retained for labels and guidance
//! - [`PendingAssessment`] — Full in-memory entry
//! - [`PendingContext`] — Self-contained journal version for persistence
//!
//! # Cross-References
//!
//! - (´dec:retention:pending-map´) — how an assessment awaiting its label is held
//! - (´def:runtime:pending-entry´) — the field set an entry carries
//! - (´def:runtime:storage-precision´) — the width the stored features are held at

use std::collections::HashMap;
use std::time::Instant;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::health::DegradationContext;
use crate::identity::CompetitiveCellId;
use crate::types::{AssessmentId, DimensionId, EntityKey, OutcomeAxisId, PersistentTimestamp, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Storage Precision
// ═══════════════════════════════════════════════════════════════════════════════

/// Precision at which the pending buffer stores feature values.
///
/// (´def:runtime:storage-precision´) fixes single as the default and makes
/// the width configurable; (´tab:config:pending-buffer´) tabulates it with
/// the constraint single-or-double, which this type enforces by
/// construction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum StoragePrecision {
    /// Single precision: quantisation five orders of magnitude below the
    /// restandardisation error already present
    /// (´def:runtime:storage-precision´).
    #[default]
    Single,
    /// Double precision, for a deployment that wants to falsify the
    /// single-precision argument rather than assume it.
    Double,
}

/// A stored feature vector at the configured storage precision.
///
/// The pending entry's feature vectors are declared against this type, so
/// the tabulated precision parameter (´tab:config:pending-buffer´) reaches
/// them by construction. Values upcast to double at label time whatever
/// the stored width (´def:runtime:storage-precision´).
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum StoredFeatures {
    /// Values quantised to single precision.
    Single(Vec<f32>),
    /// Values kept at double precision.
    Double(Vec<f64>),
}

impl StoredFeatures {
    /// Stores double-precision values at the configured precision.
    #[must_use]
    pub fn store(values: &[f64], precision: StoragePrecision) -> Self {
        match precision {
            #[allow(clippy::cast_possible_truncation)] // The quantisation is the point of the variant.
            StoragePrecision::Single => Self::Single(values.iter().map(|&v| v as f32).collect()),
            StoragePrecision::Double => Self::Double(values.to_vec()),
        }
    }

    /// Re-stores this vector at the given precision.
    ///
    /// Widening a single-precision vector recovers no information — an
    /// upcast value carries only the precision it was stored at, which is
    /// the extraction contract's own caveat
    /// (´rem:extraction:single-precision´).
    #[must_use]
    pub fn at_precision(&self, precision: StoragePrecision) -> Self {
        match (self, precision) {
            (Self::Single(_), StoragePrecision::Single) | (Self::Double(_), StoragePrecision::Double) => self.clone(),
            (Self::Single(v), StoragePrecision::Double) => Self::Double(v.iter().map(|&f| f64::from(f)).collect()),
            #[allow(clippy::cast_possible_truncation)] // The quantisation is the point of the variant.
            (Self::Double(v), StoragePrecision::Single) => Self::Single(v.iter().map(|&f| f as f32).collect()),
        }
    }

    /// Number of stored values.
    #[must_use]
    pub const fn len(&self) -> usize {
        match self {
            Self::Single(v) => v.len(),
            Self::Double(v) => v.len(),
        }
    }

    /// Whether the vector is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// The value at `index`, upcast to double.
    ///
    /// # Panics
    ///
    /// Panics if `index` is out of bounds, as slice indexing would.
    #[must_use]
    pub fn value(&self, index: usize) -> f64 {
        match self {
            Self::Single(v) => f64::from(v[index]),
            Self::Double(v) => v[index],
        }
    }

    /// The value at `index`, upcast to double, or `None` out of bounds.
    #[must_use]
    pub fn get(&self, index: usize) -> Option<f64> {
        match self {
            Self::Single(v) => v.get(index).copied().map(f64::from),
            Self::Double(v) => v.get(index).copied(),
        }
    }

    /// Iterates the stored values, upcast to double.
    pub fn iter(&self) -> Box<dyn Iterator<Item = f64> + '_> {
        match self {
            Self::Single(v) => Box::new(v.iter().copied().map(f64::from)),
            Self::Double(v) => Box::new(v.iter().copied()),
        }
    }
}

impl Default for StoredFeatures {
    fn default() -> Self {
        Self::Single(Vec::new())
    }
}

impl From<Vec<f32>> for StoredFeatures {
    fn from(values: Vec<f32>) -> Self {
        Self::Single(values)
    }
}

impl From<Vec<f64>> for StoredFeatures {
    fn from(values: Vec<f64>) -> Self {
        Self::Double(values)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Sentinel Extraction
// ═══════════════════════════════════════════════════════════════════════════════

/// Per-Sentinel extracted features stored in the pending buffer.
///
/// Extraction computes and hands over single-precision values
/// (´rem:extraction:single-precision´); the stored width follows the
/// buffer's configured precision (´def:runtime:storage-precision´), and
/// label-time reconstruction upcasts to double whatever the width.
///
/// # Invariants
///
/// - `features` length matches the Sentinel's feature width (q).
/// - `coordinate` is the hash coordinate used to route into the Sentinel's G-V graph.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SentinelExtraction {
    /// Hash coordinate in this Sentinel's 128-bit domain.
    pub coordinate: u128,
    /// Extracted features (q features, at the stored precision).
    pub features: StoredFeatures,
    /// Whether this Sentinel had a report with valid cells for this coordinate.
    pub occupancy: bool,
}

impl SentinelExtraction {
    /// Creates a new `SentinelExtraction` from extraction's own
    /// single-precision values (´rem:extraction:single-precision´).
    #[must_use]
    pub const fn new(coordinate: u128, features: Vec<f32>, occupancy: bool) -> Self {
        Self {
            coordinate,
            features: StoredFeatures::Single(features),
            occupancy,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pending Risk Basis
// ═══════════════════════════════════════════════════════════════════════════════

/// Risk scalars retained with a pending assessment.
///
/// This is the pending-state subset of the public `RiskBasis`: the
/// probability, probability-space uncertainty, anchor blend weight, and
/// effective linear predictor needed after the host later submits a label or
/// asks the Core for label guidance. The journal context carries these
/// values as the original run computed them, so a replayed label applies
/// what the original applied (´cor:durability:replay-exactness´).
///
/// # Cross-References
///
/// - (´dec:retention:pending-map´) — the map this subset is retained in
/// - (´alg:monitoring:drift-cusums´) — the drift residual `rho_effective` feeds
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PendingRiskBasis {
    /// Prior probability of bad outcome (from model prediction).
    pub p_bad: f64,
    /// Probability-space uncertainty `σ_p̂` retained for label guidance.
    pub uncertainty: f64,
    /// Anchor model weight used to dampen base rate drift.
    pub anchor_weight: f64,
    /// Blended linear predictor `ρ̂_eff` from the blend computation.
    ///
    /// Stored for drift accumulator residual at label time (´alg:monitoring:drift-cusums´):
    /// `residual = r_ρ − ρ̂_eff`.
    pub rho_effective: f64,
}

impl PendingRiskBasis {
    /// Creates a new `PendingRiskBasis` with the given values.
    #[must_use]
    pub const fn new(p_bad: f64, uncertainty: f64, anchor_weight: f64, rho_effective: f64) -> Self {
        Self {
            p_bad,
            uncertainty,
            anchor_weight,
            rho_effective,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Report Origin
// ═══════════════════════════════════════════════════════════════════════════════

/// Where the oldest Sentinel evidence behind one assessment came from.
///
/// An assessment reads the current report index of every Sentinel that
/// supplied a coordinate, and those indexes arrived at different moments. The
/// one that arrived earliest is the one whose evidence has waited longest, so
/// it is the one the feedback latency is measured against
/// (´def:monitoring:feedback-latency´): a decomposition that anchored to the
/// freshest contributor would report the shortest journey rather than the
/// journey the assessment actually completed.
///
/// The pair is carried together because neither half is a latency on its own.
/// The age was measured on the Sentinel's clock and stops at emission; the
/// arrival was measured on this process's clock and starts at reception. What
/// separates them — the host's forwarding delay — is measured by nothing and
/// is left as an explicitly unmeasured residual, which is why the stage these
/// two bound is reported as a lower bound.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ReportOrigin {
    /// When that report reached the Assayer, on the injected monotonic clock.
    pub received_at: Instant,
    /// How old its oldest observation already was when the Sentinel emitted
    /// it, in the producer's own microseconds; `None` when it carried no age.
    pub oldest_observation_age_micros: Option<u64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pending Assessment
// ═══════════════════════════════════════════════════════════════════════════════

/// Full in-memory pending assessment entry.
///
/// Contains all context needed to process a label when it arrives. Stored in the
/// [`PendingBuffer`](super::PendingBuffer) indexed by `AssessmentId`.
///
/// # Fields
///
/// - `id` — Unique assessment identifier (monotonically increasing)
/// - `timestamp` — When the assessment occurred (for expiry)
/// - `persistent_timestamp` — Wall-clock timestamp for public guidance output
/// - `entity` — Entity key for signal cache lookup
/// - `sentinel_extractions` — Per-Sentinel extracted features
/// - `active_sentinels` — Sentinels registered at assessment time
/// - `reporting_sentinels` — Sentinels actually reporting at assessment time
/// - `identity_coordinates` — Per-dimension hash coordinates
/// - `identity_active_cells` — Per-dimension competitive cells that contained the entity
/// - `entity_base_features` — Frozen per-dimension identity base features
/// - `entity_axis_features` — Frozen per-dimension, per-axis identity features
/// - `signal_features` — Merged signal features (f32)
/// - `risk_basis` — Risk basis data for the model update (´dec:posterior:three-step-update´)
/// - `degradation` — Degradation context from checkpoints CP1–CP4
/// - `report_origin` — Where the oldest Sentinel evidence this assessment read came from
///
/// The field set is the twelve of (´def:runtime:pending-entry´) plus the
/// additions that definition records as unspecified.
///
/// # Cross-References
///
/// - (´dec:retention:pending-map´) — the map this entry is held in
/// - (´def:runtime:pending-entry´) — the field set this type realises
#[derive(Clone, Debug)]
pub struct PendingAssessment {
    /// Unique assessment identifier.
    pub id: AssessmentId,
    /// Timestamp when the assessment occurred (for expiry computation).
    pub timestamp: Instant,
    /// Wall-clock timestamp when the assessment occurred.
    pub persistent_timestamp: PersistentTimestamp,
    /// Entity key (for signal cache and identity lookup).
    pub entity: EntityKey,
    /// Per-Sentinel extracted features.
    pub sentinel_extractions: HashMap<SentinelId, SentinelExtraction>,
    /// Sentinels registered at assessment time (´def:runtime:pending-entry´).
    ///
    /// With `reporting_sentinels`, lets reconstruction source four tell a
    /// Sentinel that was active but silent (occupancy, zero features) from
    /// one registered after the assessment (´alg:runtime:reconstruction´).
    pub active_sentinels: Vec<SentinelId>,
    /// Sentinels that were actually reporting at assessment time
    /// (´def:runtime:pending-entry´).
    pub reporting_sentinels: Vec<SentinelId>,
    /// Per-dimension hash coordinates.
    pub identity_coordinates: HashMap<DimensionId, u128>,
    /// Per-dimension competitive cells that contained this entity.
    pub identity_active_cells: HashMap<DimensionId, Vec<CompetitiveCellId>>,
    /// Frozen per-dimension identity base features
    /// (´def:runtime:pending-entry´): the five measurement values of the
    /// dimension block, in `IdentityMeasurementFeatures::as_slice()` order.
    /// Reconstruction source one reads them from storage; the structural
    /// triple ahead of them is re-derived from current state instead
    /// (´alg:runtime:reconstruction´).
    pub entity_base_features: HashMap<DimensionId, StoredFeatures>,
    /// Frozen per-dimension, per-axis identity features
    /// (´def:runtime:pending-entry´): the three per-axis values, in
    /// `IdentityAxisFeatures::as_slice()` order. Reconstruction source
    /// three places them where the axis existed at assessment time and
    /// zeros where it was registered afterwards
    /// (´alg:runtime:reconstruction´).
    pub entity_axis_features: HashMap<DimensionId, HashMap<OutcomeAxisId, StoredFeatures>>,
    /// The spatially enabled outcome axes, in the order the stored
    /// extractions were laid out against (´def:extraction:slot´).
    ///
    /// One list for the whole entry rather than one per Sentinel: a single
    /// assessment reads the axis set once and extracts every Sentinel
    /// against it, so a per-Sentinel copy would be the same list repeated.
    ///
    /// It is stored because it exists nowhere else. Each extraction is a flat
    /// vector whose outcome-memory pairs are laid out in this order and carry
    /// no identity of their own, so nothing already in the entry says which
    /// axis owns the pair at a given rank: the two axis-keyed fields beside it
    /// are unordered maps and both range over every outcome axis rather than
    /// the spatial subset the slot tail is built from. Without the list a
    /// reader can only assume the ranks still mean what they meant, which
    /// stops being true the moment an axis is retired from anywhere but the
    /// end (´entry:assayer:wl-feature-frozen-slot-truncation´).
    ///
    /// Empty means the extractions carry no pairs — the pre-seed path, whose
    /// synthesised extractions are featureless, is the case that says so.
    pub spatial_axis_ids: Vec<OutcomeAxisId>,
    /// Merged signal features (entity-persistent + request-scoped), at
    /// the configured storage precision (´def:runtime:storage-precision´).
    pub signal_features: StoredFeatures,
    /// Risk basis data for the model update (´dec:posterior:three-step-update´).
    pub risk_basis: PendingRiskBasis,
    /// Per-axis predicted point estimates at assessment time (´dec:health:cached-discrimination´).
    ///
    /// Combined with the label's `outcomes` at step 14 to populate
    /// `CalibrationEntry.axis_data` for discrimination metrics.
    pub outcome_predictions: HashMap<OutcomeAxisId, f64>,
    /// Degradation context from checkpoints CP1–CP4.
    pub degradation: DegradationContext,
    /// Where the oldest Sentinel evidence this assessment read came from
    /// (´def:monitoring:feedback-latency´).
    ///
    /// `None` when no Sentinel contributed an extraction, so there was no
    /// Sentinel observation for this assessment to be downstream of.
    pub report_origin: Option<ReportOrigin>,
}

impl PendingAssessment {
    /// Converts this entry to a [`PendingContext`] for journal persistence.
    ///
    /// - Drops `id` and `timestamp`: the journal entry carries its own
    ///   (´dec:retention:journal-subset´)
    /// - Drops `degradation` (not needed for replay)
    /// - Drops `report_origin`: it is an `Instant` in this process's
    ///   monotonic domain and means nothing after a restart, which is the
    ///   same reason `timestamp` is dropped
    /// - Keeps everything a replayed label's seventeen steps read,
    ///   the risk basis and outcome predictions included
    ///   (´dec:durability:checkpoint-journal´)
    #[must_use]
    pub fn to_context(&self) -> PendingContext {
        PendingContext {
            entity: self.entity.clone(),
            sentinel_extractions: self.sentinel_extractions.clone(),
            active_sentinels: self.active_sentinels.clone(),
            reporting_sentinels: self.reporting_sentinels.clone(),
            identity_coordinates: self.identity_coordinates.clone(),
            identity_active_cells: self.identity_active_cells.clone(),
            entity_base_features: self.entity_base_features.clone(),
            entity_axis_features: self.entity_axis_features.clone(),
            spatial_axis_ids: self.spatial_axis_ids.clone(),
            signal_features: self.signal_features.clone(),
            risk_basis: self.risk_basis.clone(),
            outcome_predictions: self.outcome_predictions.clone(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pending Context
// ═══════════════════════════════════════════════════════════════════════════════

/// Journal-persisted version of `PendingAssessment`, self-contained for
/// replay.
///
/// This type is serialised to the journal and used for replay after crash
/// recovery. Each entry carries enough to replay without the state that
/// produced it (´dec:durability:checkpoint-journal´): the assessment-time
/// risk basis and outcome predictions cross with the features, so a
/// replayed label applies what the original applied
/// (´cor:durability:replay-exactness´). What it omits is owned elsewhere:
/// `id` and `timestamp` belong to the journal entry itself, and
/// `degradation` is not read by any label step.
///
/// # Serde Support
///
/// Requires the `serde` feature. Uses default serde derive with JSON-compatible
/// output.
///
/// # Cross-References
///
/// - (´dec:retention:journal-subset´) — what the journal omits, and why
/// - (´def:runtime:pending-entry´) — the field set this context mirrors
/// - (´dec:durability:checkpoint-journal´) — the self-containment the payload must carry
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PendingContext {
    /// Entity key (for signal cache and identity lookup).
    pub entity: EntityKey,
    /// Per-Sentinel extracted features.
    pub sentinel_extractions: HashMap<SentinelId, SentinelExtraction>,
    /// Sentinels registered at assessment time (´def:runtime:pending-entry´).
    pub active_sentinels: Vec<SentinelId>,
    /// Sentinels actually reporting at assessment time
    /// (´def:runtime:pending-entry´).
    pub reporting_sentinels: Vec<SentinelId>,
    /// Per-dimension hash coordinates.
    pub identity_coordinates: HashMap<DimensionId, u128>,
    /// Per-dimension competitive cells that contained this entity.
    pub identity_active_cells: HashMap<DimensionId, Vec<CompetitiveCellId>>,
    /// Frozen per-dimension identity base features — the five measurement
    /// values read by reconstruction source one
    /// (´alg:runtime:reconstruction´).
    pub entity_base_features: HashMap<DimensionId, StoredFeatures>,
    /// Frozen per-dimension, per-axis identity features read by
    /// reconstruction source three (´alg:runtime:reconstruction´).
    pub entity_axis_features: HashMap<DimensionId, HashMap<OutcomeAxisId, StoredFeatures>>,
    /// The spatially enabled outcome axes the stored extractions were laid
    /// out against, in layout order (´def:extraction:slot´).
    ///
    /// Replay is exactly the case that needs it. A restart is a gap across
    /// which the spatial set may have changed, and reconstruction source four
    /// must still put each stored outcome-memory pair at its own axis's
    /// current position; the ranks alone cannot say which axis that is, and
    /// the journal is where a replayed entry's evidence comes from
    /// (´dec:durability:checkpoint-journal´).
    pub spatial_axis_ids: Vec<OutcomeAxisId>,
    /// Merged signal features (entity-persistent + request-scoped), at
    /// the configured storage precision (´def:runtime:storage-precision´).
    pub signal_features: StoredFeatures,
    /// Risk basis as the original run computed it.
    pub risk_basis: PendingRiskBasis,
    /// Per-axis predicted point estimates at assessment time.
    pub outcome_predictions: HashMap<OutcomeAxisId, f64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// An extraction copies by value: a clone carries the same coordinate,
    /// the same feature vector and the same occupancy flag, and the original
    /// is left intact beside it. Extractions are handed to both the pending
    /// buffer and the journal path, so a copy that shared or consumed the
    /// features would couple two lifetimes that must stay independent.
    ///
    /// ´claim:pending:an-extraction-clones-by-value-and-leaves-the-original-intact´
    /// ´test:unit:sentinel-extraction-clone´
    #[test]
    fn sentinel_extraction_clone() {
        let extraction = SentinelExtraction::new(12345, vec![0.5f32, 1.0, -0.3], true);
        let cloned = extraction.clone();
        assert_eq!(extraction.coordinate, 12345);
        assert_eq!(cloned.coordinate, 12345);
        assert_eq!(cloned.features, StoredFeatures::Single(vec![0.5f32, 1.0, -0.3]));
        assert!(cloned.occupancy);
    }

    /// A default risk basis carries zeros in every scalar — probability,
    /// uncertainty and anchor weight alike. An entry constructed before the
    /// model has said anything therefore claims no prior belief rather than
    /// inheriting whatever residue happened to sit in memory.
    ///
    /// ´claim:pending:a-default-risk-basis-asserts-no-prior-belief´
    /// ´test:unit:pending-risk-basis-default´
    #[test]
    fn pending_risk_basis_default() {
        let basis = PendingRiskBasis::default();
        assert!(basis.p_bad.abs() < f64::EPSILON);
        assert!(basis.uncertainty.abs() < f64::EPSILON);
        assert!(basis.anchor_weight.abs() < f64::EPSILON);
    }

    /// Converting an in-memory entry to its journal context keeps everything
    /// replay depends on: the entity key, the merged signal features, the
    /// per-Sentinel extractions, the per-dimension coordinates and active
    /// cells, and the assessment-time scalars — the risk basis value for
    /// value and the per-axis predictions — all arrive unchanged. The entry
    /// that reaches the journal is a whole submission depending on nothing
    /// but itself (´dec:durability:checkpoint-journal´), so a field quietly
    /// lost in the conversion would be a step a replayed label cannot run.
    ///
    /// ´claim:pending:the-journal-context-preserves-everything-replay-depends-on´
    /// ´test:unit:pending-assessment-to-context-preserves-core-fields´
    #[test]
    fn pending_assessment_to_context_preserves_core_fields() {
        let assessment = create_test_assessment();
        let context = assessment.to_context();
        assert_eq!(context.entity, assessment.entity);
        assert_eq!(context.signal_features, assessment.signal_features);
        assert_eq!(context.sentinel_extractions.len(), assessment.sentinel_extractions.len());
        assert_eq!(context.active_sentinels, assessment.active_sentinels);
        assert_eq!(context.reporting_sentinels, assessment.reporting_sentinels);
        assert_eq!(context.identity_coordinates, assessment.identity_coordinates);
        assert_eq!(context.identity_active_cells.len(), assessment.identity_active_cells.len());
        assert_eq!(context.entity_base_features, assessment.entity_base_features);
        assert_eq!(context.entity_axis_features, assessment.entity_axis_features);
        assert!((context.risk_basis.p_bad - assessment.risk_basis.p_bad).abs() < f64::EPSILON);
        assert!((context.risk_basis.uncertainty - assessment.risk_basis.uncertainty).abs() < f64::EPSILON);
        assert!((context.risk_basis.anchor_weight - assessment.risk_basis.anchor_weight).abs() < f64::EPSILON);
        assert!((context.risk_basis.rho_effective - assessment.risk_basis.rho_effective).abs() < f64::EPSILON);
        assert_eq!(context.outcome_predictions, assessment.outcome_predictions);
    }

    /// What the journal context omits is only what is owned elsewhere: the
    /// type has no `id` or `timestamp` field (the journal entry carries its
    /// own), no `degradation` field (no label step reads it), and no
    /// `report_origin` field (an instant in a monotonic domain the restart
    /// ends), so the omission is enforced by the type system rather than by a
    /// copying routine remembering. The model-derived scalars are not among the
    /// omissions — they cross with the entry, because a replayed label must
    /// apply what the original applied rather than recompute against a
    /// restored model (´cor:durability:replay-exactness´).
    ///
    /// ´claim:pending:transient-fields-are-stripped-by-the-type-not-by-convention´
    /// ´test:unit:to-context-strips-transient-fields´
    #[test]
    fn to_context_strips_transient_fields() {
        let assessment = create_test_assessment();
        let context = assessment.to_context();
        // The retained half is accessible, scalars included:
        assert!(!context.entity.0.is_empty());
        assert!(!context.sentinel_extractions.is_empty());
        assert!(!context.identity_coordinates.is_empty());
        assert!(!context.identity_active_cells.is_empty());
        assert!(!context.signal_features.is_empty());
        assert!(context.risk_basis.p_bad > 0.0, "the risk basis crosses into the journal");
        // If PendingContext had the omitted fields, the following would compile:
        // let _ = context.id;              // ERROR: no field (journal has its own)
        // let _ = context.timestamp;       // ERROR: no field (journal has its own)
        // let _ = context.degradation;     // ERROR: no field (no label step reads it)
        // let _ = context.report_origin;   // ERROR: no field (monotonic, ends at restart)
        // This test passes by virtue of compiling — the type system enforces the omission.
    }

    /// A journal context survives a round trip through JSON with its feature
    /// values bit-for-bit intact, negatives and fractions included. The stored
    /// features are the only record of what the model saw, so replay after a
    /// restart is only faithful if serialisation neither rounds them nor
    /// widens them on the way back.
    ///
    /// ´claim:pending:a-context-round-trips-through-serialisation-with-its-features-exact´
    /// ´test:unit:pending-context-serde-roundtrip´
    #[cfg(feature = "serde")]
    #[test]
    fn pending_context_serde_roundtrip() {
        let assessment = create_test_assessment();
        let context = assessment.to_context();
        let json = serde_json::to_string(&context).expect("serialise");
        let restored: PendingContext = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(restored.signal_features, context.signal_features);
        // Verify f32 features survived
        assert_eq!(restored.signal_features, StoredFeatures::Single(vec![0.5f32, 1.0, -0.3]));
    }

    /// The four reconstruction fields cross the journal boundary in both
    /// directions with their values equal: the Sentinel lists and the two
    /// frozen identity feature blocks survive serialisation and
    /// deserialisation exactly, and the restored context serialises back to
    /// a context equal to itself. These are the inputs reconstruction
    /// sources one, three and four read from storage
    /// (´alg:runtime:reconstruction´), and no call at label time can supply
    /// them from state the label path holds — so a field lost or rounded at
    /// this boundary is a reconstruction block silently zeroed or skewed
    /// after every restart.
    ///
    /// ´claim:pending:the-four-reconstruction-fields-cross-the-journal-boundary-both-ways-exactly´
    /// ´test:unit:pending-context-reconstruction-fields-round-trip´
    #[cfg(feature = "serde")]
    #[test]
    fn pending_context_reconstruction_fields_round_trip() {
        let assessment = create_test_assessment();
        let context = assessment.to_context();

        // Outward: in-memory context → journal bytes → restored context.
        let json = serde_json::to_string(&context).expect("serialise");
        let restored: PendingContext = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(restored.active_sentinels, context.active_sentinels);
        assert_eq!(restored.reporting_sentinels, context.reporting_sentinels);
        assert_eq!(restored.entity_base_features, context.entity_base_features);
        assert_eq!(restored.entity_axis_features, context.entity_axis_features);

        // Return: the restored context crosses again and lands on itself.
        let json_again = serde_json::to_string(&restored).expect("serialise restored");
        let restored_again: PendingContext = serde_json::from_str(&json_again).expect("deserialise restored");
        assert_eq!(restored_again.active_sentinels, restored.active_sentinels);
        assert_eq!(restored_again.reporting_sentinels, restored.reporting_sentinels);
        assert_eq!(restored_again.entity_base_features, restored.entity_base_features);
        assert_eq!(restored_again.entity_axis_features, restored.entity_axis_features);
    }

    /// The storage precision is a real choice with a measurable difference:
    /// a value single precision cannot represent survives double-precision
    /// storage bit-for-bit and comes back from single-precision storage as
    /// its nearest single — exactly the quantisation the corpus argues is
    /// five orders of magnitude below the restandardisation error
    /// (´def:runtime:storage-precision´). The default is single, as the
    /// parameter table fixes it (´tab:config:pending-buffer´). A deployment
    /// that doubts the argument can now falsify it instead of assuming it.
    ///
    /// ´claim:pending:double-storage-keeps-what-single-storage-quantises´
    /// ´test:unit:storage-precision-double-keeps-what-single-quantises´
    #[test]
    fn storage_precision_double_keeps_what_single_quantises() {
        assert_eq!(StoragePrecision::default(), StoragePrecision::Single, "tabulated default");

        let value = 0.1_f64; // not representable in single precision
        let double = StoredFeatures::store(&[value], StoragePrecision::Double);
        let single = StoredFeatures::store(&[value], StoragePrecision::Single);

        assert!(
            (double.value(0) - value).abs() < f64::EPSILON,
            "double keeps the value exactly"
        );
        assert!((single.value(0) - value).abs() > 0.0, "single quantises it");
        assert!(
            (single.value(0) - f64::from(0.1_f32)).abs() < f64::EPSILON,
            "the quantised value is the nearest single"
        );
    }

    /// A double-precision context crosses the journal boundary with its
    /// values bit-for-bit intact, in both directions. The double variant
    /// exists to retain what single storage would quantise, so a journal
    /// pass that rounded it back down would silently reduce the
    /// configuration to its default (´def:runtime:storage-precision´).
    ///
    /// ´claim:pending:a-double-precision-context-round-trips-bit-for-bit´
    /// ´test:unit:double-precision-context-serde-roundtrip´
    #[cfg(feature = "serde")]
    #[test]
    fn double_precision_context_serde_roundtrip() {
        let mut assessment = create_test_assessment();
        assessment.signal_features = StoredFeatures::store(&[0.1, 0.2, -0.3], StoragePrecision::Double);
        let context = assessment.to_context();

        let json = serde_json::to_string(&context).expect("serialise");
        let restored: PendingContext = serde_json::from_str(&json).expect("deserialise");
        assert_eq!(restored.signal_features, context.signal_features);
        assert!((restored.signal_features.value(0) - 0.1_f64).abs() < f64::EPSILON);

        let json_again = serde_json::to_string(&restored).expect("serialise restored");
        let restored_again: PendingContext = serde_json::from_str(&json_again).expect("deserialise restored");
        assert_eq!(restored_again.signal_features, restored.signal_features);
    }

    /// An extraction at the reference feature width stays inside a few hundred
    /// bytes, features included. The buffer holds one of these per Sentinel
    /// for every assessment still awaiting its label, so the per-entry
    /// footprint is what decides how deep the pending horizon can be before
    /// memory rather than policy sets the limit.
    ///
    /// ´claim:pending:an-extraction-at-reference-width-stays-within-its-memory-budget´
    /// ´test:unit:sentinel-extraction-memory-size´
    #[test]
    fn sentinel_extraction_memory_size() {
        // Reference q = 62 features at m_s=1
        let features: Vec<f32> = vec![0.0f32; 62];
        let extraction = SentinelExtraction::new(0, features, true);
        // Approximate size: u128 (16) + Vec header (24) + 62*4 (248) + bool (1) = ~289 bytes
        // The spec says ~252 bytes, but this is close enough for a reference check
        let size = std::mem::size_of_val(&extraction) + extraction.features.len() * std::mem::size_of::<f32>();
        // Allow some slack for alignment and Vec overhead
        assert!(
            size < 400,
            "SentinelExtraction with 62 features should be < 400 bytes, got {size}"
        );
    }

    /// Creates a test `PendingAssessment` with sample data.
    fn create_test_assessment() -> PendingAssessment {
        let mut sentinel_extractions = HashMap::new();
        sentinel_extractions.insert(SentinelId(1), SentinelExtraction::new(42, vec![1.0f32, 2.0], true));

        let mut identity_coordinates = HashMap::new();
        identity_coordinates.insert(DimensionId(1), 123u128);

        let mut identity_active_cells = HashMap::new();
        identity_active_cells.insert(DimensionId(1), vec![CompetitiveCellId::new(0, 8)]);

        PendingAssessment {
            spatial_axis_ids: Vec::new(),
            id: AssessmentId(42),
            timestamp: Instant::now(),
            persistent_timestamp: PersistentTimestamp::now(),
            entity: EntityKey::new(vec![1, 2, 3]),
            sentinel_extractions,
            active_sentinels: vec![SentinelId(1), SentinelId(2)],
            reporting_sentinels: vec![SentinelId(1)],
            identity_coordinates,
            identity_active_cells,
            entity_base_features: HashMap::from([(DimensionId(1), vec![0.9f32, 0.4, 0.6, 0.05, 1.0].into())]),
            entity_axis_features: HashMap::from([(
                DimensionId(1),
                HashMap::from([(OutcomeAxisId(2), vec![0.3f32, -0.2, 0.1].into())]),
            )]),
            signal_features: vec![0.5f32, 1.0, -0.3].into(),
            risk_basis: PendingRiskBasis::new(0.1, 0.3, 0.2, 0.4),
            outcome_predictions: HashMap::from([(OutcomeAxisId(2), 0.7)]),
            degradation: DegradationContext::default(),
            report_origin: Some(ReportOrigin {
                received_at: Instant::now(),
                oldest_observation_age_micros: Some(1_500),
            }),
        }
    }
}
