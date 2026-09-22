// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Published model snapshot (read-only, shared via `ArcSwap`).
//!
//! `ModelSnapshot` is the unit of data published to readers. It captures
//! the full Core state needed for `assess()` calls without holding any mutable
//! model state. The precision matrix B is excluded — only μ and Σ are
//! published (´dec:retention:precision-excluded´).
//!
//! Some aggregate fields are patched by the model-owner label pipeline before
//! publication; per-axis lifecycle metadata is projected directly from the
//! working copy.
//!
//! The three core models' floor masses travel with the parameters, because a
//! reader holding only μ and Σ cannot recover how much of the precision behind
//! them the model borrowed from its own floors
//! (´dec:posterior:spectral-floor´).
//!
//! # Cross-References
//!
//! - (´dec:concurrency:snapshot-swap´) — lock-free publication by one swap
//! - (´dec:retention:precision-excluded´) — snapshot composition and the B-exclusion
//! - (´def:publication:model-snapshot´) — the full field catalogue a snapshot carries

use std::collections::HashMap;
use std::time::Instant;

use indexmap::IndexMap;

use crate::feature::dimension_map::DimensionMap;
use crate::model::bayesian::BayesianLinearModel;
use crate::model::parameters::{FloorMass, ModelParameters};
use crate::types::{ModelId, OutcomeAxisId};

// ═══════════════════════════════════════════════════════════════════════════════
// Ancillary Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Per-axis model state snapshot.
///
/// Contains model parameters and per-axis metadata for outcome-axis models.
/// Populated from `WorkingAxisModel` during snapshot projection.
#[derive(Clone, Debug)]
pub struct AxisModelState {
    /// Human-readable axis name.
    pub name: String,
    /// Serialisable parameters (μ, Σ).
    pub parameters: ModelParameters,
    /// Condition number estimate for this axis model.
    pub kappa_a: f64,
    /// Decay rate for this axis model.
    pub gamma_a: f64,
    /// Whether this axis contributes per-Sentinel spatial ledger features.
    pub spatial: bool,
    /// Per-axis eligibility mode (´req:host:eligibility-policy´).
    pub eligibility: crate::types::OutcomeEligibility,
}

/// TODO ´todo:code:retire-legacy-dimensionmapsnapshot´: retire legacy `DimensionMapSnapshot`
/// (´dec:assayer:spec-authoritative´).
#[derive(Clone, Debug)]
pub struct DimensionMapSnapshot {
    /// Current operational model dimensionality.
    pub p: usize,
}

// TODO ´todo:code:retire-the-snapshot-driftstate-re-export´: retire the snapshot `DriftState` re-export
// (´dec:assayer:spec-authoritative´).
pub use crate::health::DriftState;

/// The floor masses the three core models carry, published beside their
/// parameters (´dec:posterior:spectral-floor´).
///
/// The bulk argument that keeps the precision matrix out of a snapshot does not
/// reach these (´dec:retention:precision-excluded´). `B` is `p²` per model and
/// would double a published snapshot to serve a path that never reads it; the
/// masses are `p + 1` per model, five orders smaller at the reference width,
/// and the assessment path reads them on every request. What they buy is the
/// one thing the covariance alone cannot say: which part of the confidence
/// behind it was borrowed.
///
/// Only the three core models carry them. The outcome-axis models are absent
/// because nothing yet tempers an axis prediction by them, and a field
/// published before it is read is a field whose meaning nothing holds to
/// account.
#[derive(Clone, Debug, Default)]
pub struct CoreFloorMasses {
    /// The operational model's two masses.
    pub operational: FloorMass,
    /// The sister model's two masses.
    pub sister: FloorMass,
    /// The anchor model's two masses.
    pub anchor: FloorMass,
}

impl CoreFloorMasses {
    /// Projects the three live models' tracked masses into the published form.
    #[must_use]
    pub fn from_models(operational: &BayesianLinearModel, sister: &BayesianLinearModel, anchor: &BayesianLinearModel) -> Self {
        Self {
            operational: FloorMass::new(operational.spectral_floor_mass(), operational.clamp_mass().to_vec()),
            sister: FloorMass::new(sister.spectral_floor_mass(), sister.clamp_mass().to_vec()),
            anchor: FloorMass::new(anchor.spectral_floor_mass(), anchor.clamp_mass().to_vec()),
        }
    }
}

/// Calibration buffer snapshot.
///
/// Carries the regimes' weighted sample counts from the `CalibrationBuffer`
/// (´constr:platt:buffer´) so that the assessment path can populate the three
/// calibration-related fields on `RiskBasis` without touching the
/// model-owner thread.
///
/// # Cross-References
///
/// - (´constr:platt:buffer´) — what the calibration buffer holds, and how weighted
/// - (´schema:risk:basis´) — the fields on `RiskBasis` these counts populate
#[derive(Clone, Debug, Default)]
pub struct CalibrationBufferSnapshot {
    /// Total entries in the calibration buffer.
    pub total_entries: usize,
    /// Weighted sister-regime sample count: each record's sister share
    /// times its importance weight, summed (´constr:platt:buffer´).
    pub sister_regime_records: f64,
    /// Weighted anchor-regime sample count: each record's anchor share
    /// times its importance weight, summed (´constr:platt:buffer´).
    pub anchor_regime_records: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// ModelSnapshot
// ═══════════════════════════════════════════════════════════════════════════════

/// Published model snapshot.
///
/// This is the immutable snapshot shared with all readers via `ArcSwap`.
/// It contains everything needed for `assess()` calls: model parameters,
/// calibration state, and dimensional metadata. The precision matrix B
/// is excluded (´dec:retention:precision-excluded´).
///
/// At ~9 MB per snapshot for p=638 (three models × p²×8 bytes for Σ
/// plus ancillary fields), two snapshots may briefly coexist during
/// swap — ~18 MB transient, because the snapshot is built whole and
/// installed with one swap (´dec:retention:monolithic-snapshot´).
///
/// # Send + Sync
///
/// Required by `ArcSwap<ModelSnapshot>`. Satisfied because all fields
/// are either `Copy`, `Clone` containers of `Send + Sync` types, or
/// `Instant` (which is `Send + Sync`).
#[derive(Clone, Debug)]
pub struct ModelSnapshot {
    /// Monotonically increasing version counter.
    pub version: u64,

    /// Wall-clock instant when this snapshot was published.
    pub publish_timestamp: Instant,

    // ─── Core model parameters ───────────────────────────────────────
    /// Operational model (V) parameters: μ and Σ.
    pub operational: ModelParameters,

    /// Sister model parameters: μ and Σ.
    pub sister: ModelParameters,

    /// Anchor model parameters: μ and Σ (fixed at `p_a` = 15).
    pub anchor: ModelParameters,

    /// What each of the three carries in floor mass, so that a reader of the
    /// covariance can tell borrowed precision from earned
    /// (´dec:posterior:spectral-floor´).
    pub floor_masses: CoreFloorMasses,

    // ─── Outcome-axis models ─────────────────────────────────────────
    /// Per-axis model state. Present, empty at cold start.
    /// Populated in Layer 3, by the label call that mutates them
    /// (´dec:ordering:label-function´).
    pub outcome_models: IndexMap<OutcomeAxisId, AxisModelState>,

    // ─── Dimensional metadata ────────────────────────────────────────
    /// Dimension map — structural tracking of feature-vector index ranges.
    ///
    /// Replaced the Layer 1 `DimensionMapSnapshot` placeholder with full
    /// `DimensionMap` in Layer 2, whose block order it tracks
    /// (´dec:vector:block-order´).
    pub dimension_map: DimensionMap,

    // ─── Feature statistics ──────────────────────────────────────────
    /// Running mean of features. Present, empty at cold start.
    pub feature_means: Vec<f64>,

    /// Running variance of features. Present, empty at cold start.
    pub feature_variances: Vec<f64>,

    /// Where the cold prior-mass ramp stood when this snapshot was published
    /// (´tab:monitoring:standardisation-transition´).
    ///
    /// This is the coordinate state a result scored against this snapshot was
    /// computed in, not where the ramp has since reached. Every accepted
    /// advance publishes a snapshot, so the version beside it is the
    /// coordinate version and no second version namespace is needed
    /// (´dec:health:standardisation-phase-reported´).
    pub standardisation_phase: crate::feature::standardisation::StandardisationPhase,

    /// Accepted cold-ramp observations this snapshot represents, from zero
    /// through `N_init`. Maturity is this against the horizon and is not
    /// stored beside it (´dec:vector:prior-mass-ramp´).
    pub standardisation_observations: usize,

    /// Which layout the feature statistics describe.
    ///
    /// An assessment carries this back with the raw vector it offers, so the
    /// steward can refuse an observation assembled under a layout it has
    /// since renumbered (´req:standardisation:lifecycle-entries´).
    pub layout_generation: u64,

    // ─── Global calibration scalars ──────────────────────────────────
    /// Global positive-class prior (for uncalibrated blend).
    pub p_positive_global: f64,

    /// Eligible positive-class prior (accounting for challenge flow).
    pub p_positive_eligible: f64,

    // ─── Platt calibration and compression scalars ────────────────────
    /// Valence compression scale `κ_v` (´def:risk:compression-scale´).
    /// Assessment reads: for compressed valence features in φ.
    pub kappa_v: f64,

    /// Sister-regime Platt temperature parameter (´def:platt:regimes´).
    /// Assessment reads: for σ(ρ̂/κ) probability mapping.
    pub kappa_sister: f64,

    /// Anchor-regime Platt temperature parameter (´def:platt:regimes´).
    pub kappa_anchor: f64,

    // ─── Calibration, Layer 5 snapshot export (´constr:platt:buffer´) ──
    /// Calibration buffer snapshot. Reserved for Layer 5.
    pub calibration_buffer: CalibrationBufferSnapshot,

    // ─── Drift (´alg:monitoring:drift-cusums´) ─────────────────────────
    /// Per-model drift state. Present, empty at cold start.
    pub drift: HashMap<ModelId, DriftState>,
}

// Compile-time assertion that `ModelSnapshot` is `Send + Sync`.
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ModelSnapshot>();
};

impl ModelSnapshot {
    /// Creates a cold-start snapshot with default values.
    ///
    /// All models carry the given parameters. Optional collections are empty/default.
    /// The dimension map is built from an empty configuration (no sentinels,
    /// no identity dimensions, no signals).
    #[must_use]
    pub fn cold_start(
        version: u64,
        operational: ModelParameters,
        sister: ModelParameters,
        anchor: ModelParameters,
        p: usize,
    ) -> Self {
        // Build a cold-start dimension map. At cold start there are no
        // sentinels, dimensions, signals, or outcome axes.
        let dimension_map = DimensionMap::rebuild_no_interactions(
            0,                // p_sig
            &IndexMap::new(), // sentinels
            &[],              // identity_dimensions
            &IndexMap::new(), // competitive_cells
            &[],              // spatial_axis_ids
        );
        // Note: dimension_map.p may differ from `p` at cold start.
        // The `p` parameter is retained for backward-compatible model
        // dimension; Layer 3 aligns them (´dec:vector:block-order´).
        let _ = p;

        let dim_p = dimension_map.p;

        Self {
            version,
            publish_timestamp: Instant::now(),
            operational,
            sister,
            anchor,
            // A model at its construction prior has borrowed nothing: no
            // rebuild has applied a spectral floor and no clamp has bound.
            floor_masses: CoreFloorMasses::default(),
            outcome_models: IndexMap::new(),
            dimension_map,
            feature_means: vec![0.0; dim_p],
            feature_variances: vec![1.0; dim_p],
            standardisation_phase: crate::feature::standardisation::StandardisationPhase::WaitingForInit,
            standardisation_observations: 0,
            layout_generation: 0,
            p_positive_global: 0.5,
            p_positive_eligible: 0.5,
            kappa_v: 1.0,
            kappa_sister: 1.0,
            kappa_anchor: 1.0,
            calibration_buffer: CalibrationBufferSnapshot::default(),
            drift: HashMap::new(),
        }
    }
}
