// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Owned, read-only projections of test-support state.
//!
//! Every projection follows the same contract: its [`World`](super::World) entry point crosses the queue barrier the reading depends on, takes one source view, copies only retained fields into an owned value, exposes no route back to mutable engine state, and obtains any observation time from the scenario clock. Published precision remains outside this surface (´dec:harness:probe-contract´) (´cav:retention:probe-boundary´).

use std::collections::HashMap;

use crate::feature::standardisation::StandardisationPhase;
use crate::model::parameters::{FloorMass, ModelParameters};
use crate::pending::{PendingAssessment, StoredFeatures};
use crate::snapshot::published::ModelSnapshot;
use crate::types::{DimensionId, ModelId, PersistentTimestamp, SentinelId};

/// One named model block copied from a single published snapshot.
///
/// The mean, packed covariance, and floor masses are owned copies. The precision matrix is deliberately absent, and no field borrows or retains a guard from the engine. `observed_at` is read from the scenario clock.
#[derive(Clone, Debug, PartialEq)]
pub struct PublishedModelBlock {
    /// Model selected by the caller.
    pub model: ModelId,
    /// Version of the one snapshot that supplied every published field.
    pub published_version: u64,
    /// Layout generation of that same snapshot.
    pub layout_generation: u64,
    /// Scenario-owned persistent time at which the projection was taken.
    pub observed_at: PersistentTimestamp,
    /// Published model dimension.
    pub dimension: usize,
    /// Published mean vector.
    pub mean: Vec<f64>,
    /// Published covariance in column-major order.
    pub covariance: Vec<f64>,
    /// Published identity-shaped floor mass, absent for outcome-axis models.
    pub spectral_floor_mass: Option<f64>,
    /// Published coordinate-shaped floor masses, absent for outcome-axis models.
    pub clamp_floor_mass: Option<Vec<f64>>,
}

impl PublishedModelBlock {
    pub(super) fn from_snapshot(snapshot: &ModelSnapshot, model: ModelId, observed_at: PersistentTimestamp) -> Option<Self> {
        let (parameters, floor_mass) = match &model {
            ModelId::Operational => (&snapshot.operational, Some(&snapshot.floor_masses.operational)),
            ModelId::Sister => (&snapshot.sister, Some(&snapshot.floor_masses.sister)),
            ModelId::Anchor => (&snapshot.anchor, Some(&snapshot.floor_masses.anchor)),
            ModelId::OutcomeAxis(axis) => (&snapshot.outcome_models.get(axis)?.parameters, None),
        };
        Some(Self::from_parameters(
            model,
            snapshot.version,
            snapshot.layout_generation,
            observed_at,
            parameters,
            floor_mass,
        ))
    }

    fn from_parameters(
        model: ModelId,
        published_version: u64,
        layout_generation: u64,
        observed_at: PersistentTimestamp,
        parameters: &ModelParameters,
        floor_mass: Option<&FloorMass>,
    ) -> Self {
        Self {
            model,
            published_version,
            layout_generation,
            observed_at,
            dimension: parameters.p,
            mean: parameters.mu.clone(),
            covariance: parameters.covariance_data.clone(),
            spectral_floor_mass: floor_mass.map(|mass| mass.spectral),
            clamp_floor_mass: floor_mass.map(|mass| mass.clamp.clone()),
        }
    }
}

/// Standardisation moments for one named Sentinel slot in one publication.
///
/// The slot range, moments, phase, and count all come from the same snapshot. No precision or model state is exposed.
#[derive(Clone, Debug, PartialEq)]
pub struct PublishedSlotMoments {
    /// Sentinel whose published slot was selected.
    pub sentinel: SentinelId,
    /// Version of the one snapshot that supplied every field.
    pub published_version: u64,
    /// Layout generation of that same snapshot.
    pub layout_generation: u64,
    /// Published means across the complete Sentinel slot.
    pub means: Vec<f64>,
    /// Published variances across the complete Sentinel slot.
    pub variances: Vec<f64>,
    /// Published cold-ramp phase.
    pub phase: StandardisationPhase,
    /// Published accepted cold-ramp observation count.
    pub observation_count: usize,
}

impl PublishedSlotMoments {
    pub(super) fn from_snapshot(snapshot: &ModelSnapshot, sentinel: SentinelId) -> Option<Self> {
        let range = snapshot.dimension_map.sentinel_slots.get(&sentinel)?.to_range();
        Some(Self {
            sentinel,
            published_version: snapshot.version,
            layout_generation: snapshot.layout_generation,
            means: snapshot.feature_means.get(range.clone())?.to_vec(),
            variances: snapshot.feature_variances.get(range)?.to_vec(),
            phase: snapshot.standardisation_phase,
            observation_count: snapshot.standardisation_observations,
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
enum RetainedSignalFeatures {
    Single(Vec<f32>),
    Double(Vec<f64>),
}

/// Request-scoped input state copied from one live pending entry.
///
/// This view contains only the raw signal block and active competitive-cell counts needed to identify the request input. It retains the buffer's actual storage width. The identifier, both timestamps, degradation context, report origin, model state, dimension map, cache, graph, and every mutation route are excluded.
#[derive(Clone, Debug, PartialEq)]
pub struct PendingEntryView {
    signal_features: RetainedSignalFeatures,
    /// Number of active competitive cells retained for each identity dimension.
    pub active_cell_counts: HashMap<DimensionId, usize>,
}

impl PendingEntryView {
    pub(super) fn from_pending(pending: &PendingAssessment) -> Self {
        let signal_features = match &pending.signal_features {
            StoredFeatures::Single(values) => RetainedSignalFeatures::Single(values.clone()),
            StoredFeatures::Double(values) => RetainedSignalFeatures::Double(values.clone()),
        };
        let active_cell_counts = pending
            .identity_active_cells
            .iter()
            .map(|(&dimension, cells)| (dimension, cells.len()))
            .collect();
        Self {
            signal_features,
            active_cell_counts,
        }
    }

    /// Returns the retained signal values when this entry uses single precision.
    #[must_use]
    pub fn single_precision_signals(&self) -> Option<&[f32]> {
        match &self.signal_features {
            RetainedSignalFeatures::Single(values) => Some(values),
            RetainedSignalFeatures::Double(_) => None,
        }
    }

    /// Returns the retained signal values when this entry uses double precision.
    #[must_use]
    pub fn double_precision_signals(&self) -> Option<&[f64]> {
        match &self.signal_features {
            RetainedSignalFeatures::Single(_) => None,
            RetainedSignalFeatures::Double(values) => Some(values),
        }
    }
}
