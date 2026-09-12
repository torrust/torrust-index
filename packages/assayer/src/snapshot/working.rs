// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Mutable working copy maintained by the model-owner thread.
//!
//! `WorkingCopy` holds the live `BayesianLinearModel` instances that
//! receive rank-1 updates on each label. It projects into a
//! `ModelSnapshot` for publication and can be reconstructed from one
//! for crash recovery.
//!
//! # Cross-References
//!
//! - (´dec:concurrency:snapshot-swap´) — lock-free publication by one swap
//! - (´dec:substrate:dense-dynamic´) — the dynamic-dimension model type held here
//! - (´dec:concurrency:single-steward´) — the one thread that may mutate this copy
//! - (´dec:retention:precision-excluded´) — snapshot composition and the B-exclusion
//! - (´dec:posterior:recomputation-trigger´) — what the Cholesky tracking fields count towards

use std::collections::HashMap;
use std::time::Instant;

use indexmap::IndexMap;

use super::published::{AxisModelState, CalibrationBufferSnapshot, ModelSnapshot};
pub use crate::config::types::{ModelConfig, P_A};
use crate::feature::bootstrap::ColdRamp;
use crate::feature::dimension_map::DimensionMap;
use crate::feature::interaction::InteractionTemplate;
use crate::feature::permutation::LayoutPermutation;
use crate::feature::standardisation::{FeatureClass, assign_feature_classes, init_from_priors};
use crate::health::DriftState;
use crate::model::bayesian::{BayesianLinearModel, RevertError};
use crate::model::marginalise::{MarginalisationErrorLedger, SchurConfig};
use crate::types::{ModelId, OutcomeAxisId};

// ═══════════════════════════════════════════════════════════════════════════════
// WorkingAxisModel
// ═══════════════════════════════════════════════════════════════════════════════

/// Per-axis working model.
///
/// Contains the Bayesian linear model and per-axis metadata.
/// Populated from Layer 3, by the label call that mutates them
/// (´dec:ordering:label-function´).
#[derive(Debug)]
pub struct WorkingAxisModel {
    /// Human-readable axis name.
    pub name: String,
    /// What the axis measures, in the host's own words
    /// (´schema:registry:axis-record´).
    pub description: String,
    /// The Bayesian linear model for this axis.
    pub model: BayesianLinearModel,
    /// Condition number estimate.
    pub kappa_a: f64,
    /// Per-axis label-indexed forgetting rate (´tab:risk:forgetting-rates´).
    ///
    /// Default: `GAMMA_LABEL_SISTER` (0.9998). Configurable at
    /// registration via `OutcomeAxisRegistration::gamma`.
    pub gamma: f64,
    /// Whether spatial features are enabled for this axis (´disc:registry:spatial-policy´).
    ///
    /// When `true`, each registered Sentinel gains 2 extraction features
    /// (compressed + raw axis EWMA) for this axis.
    pub spatial: bool,
    /// Which labels this axis receives (´req:host:eligibility-policy´).
    ///
    /// `EligibleOnly` means the axis updates only on globally eligible
    /// labels; `AllLabels` means all labels update this axis regardless
    /// of the global eligibility gate.
    pub eligibility: crate::types::OutcomeEligibility,
}

// ═══════════════════════════════════════════════════════════════════════════════
// WorkingCopy
// ═══════════════════════════════════════════════════════════════════════════════

/// Mutable working copy of all model state.
///
/// Maintained exclusively by the model-owner thread. Projects into
/// `ModelSnapshot` for `ArcSwap` publication. Reconstructed from
/// a snapshot (plus `cholesky_inverse`) on crash recovery.
#[derive(Debug)]
pub struct WorkingCopy {
    // ─── Core models ─────────────────────────────────────────────────
    /// The primary operational model (V).
    pub operational: BayesianLinearModel,

    /// The sister model for drift detection.
    pub sister: BayesianLinearModel,

    /// The anchor model with fixed dimensionality `p_a`.
    pub anchor: BayesianLinearModel,

    // ─── Outcome-axis models ─────────────────────────────────────────
    /// Per-axis working models. Present, empty at cold start.
    /// Populated in Layer 3, by the label call that mutates them
    /// (´dec:ordering:label-function´).
    pub outcome_models: IndexMap<OutcomeAxisId, WorkingAxisModel>,

    // ─── Platt calibration and compression scalars ────────────────────
    /// Sister-regime Platt temperature parameter (´def:platt:regimes´).
    pub kappa_sister: f64,

    /// Anchor-regime Platt temperature parameter (´def:platt:regimes´).
    pub kappa_anchor: f64,

    /// Global positive-class prior.
    pub p_positive_global: f64,

    /// Eligible positive-class prior.
    pub p_positive_eligible: f64,

    /// Valence compression scale `κ_v` (´def:risk:compression-scale´).
    pub kappa_v: f64,

    // ─── Sequencing ──────────────────────────────────────────────────
    /// Sequence number of the last processed label, for journal
    /// coordination (´dec:durability:checkpoint-journal´).
    pub last_processed_label_seq: u64,

    // TODO ´todo:code:add-total-eligible-labels-u64-to´: add `total_eligible_labels: u64` to the
    // working copy, initialise it at cold start, increment it after
    // label eligibility succeeds, and include it in checkpoint payloads.
    // This counter is the composite convergence clock for stages 1–3
    // (´dec:operational:convergence-diagnostic´).
    /// Instant of the last processed label (for time-indexed decay).
    pub last_label_time: Instant,

    // ─── Dimension map ─────────────────────────────────────────────────────
    /// Cached dimension map for index-range tracking.
    ///
    /// Rebuilt when entity registrations change (sentinel, dimension,
    /// competitive cell lifecycle events). Cloned into the snapshot
    /// on publication.
    pub dimension_map: DimensionMap,

    // ─── Drift state (´alg:monitoring:drift-cusums´) ─────────────────
    /// Per-model drift accumulators.
    ///
    /// Updated at label step 16 with CUSUM and EWMA tracking.
    /// Keys: `ModelId::Operational`, `ModelId::Sister`, etc.
    pub drift_state: HashMap<ModelId, DriftState>,

    // ─── Standardisation statistics ──────────────────────────────────
    /// Running feature means, indexed against `dimension_map`.
    pub feature_means: Vec<f64>,

    /// Running feature variances, indexed against `dimension_map`.
    pub feature_variances: Vec<f64>,

    /// Per-feature class assignments (for standardisation extend/compact).
    ///
    /// Derived from the map and the signal schema at construction and
    /// recomputed after every rebuild, with the means and variances
    /// initialised from the resulting classes' priors
    /// (´req:standardisation:class-assignment´).
    pub feature_classes: Vec<FeatureClass>,

    /// The cold prior-mass ramp (´alg:standardisation:batch-initialisation´).
    ///
    /// The steward owns it, and it is the sole author of `feature_means` and
    /// `feature_variances` until it reaches its horizon
    /// (´dec:concurrency:single-steward´). It stays after completion because
    /// its count and phase are reported for the life of the instance, not only
    /// while they are changing (´dec:health:standardisation-phase-reported´).
    pub cold_ramp: ColdRamp,

    /// Which layout the standardisation vectors currently describe.
    ///
    /// Advanced once per lifecycle rebuild. An observation assembled under an
    /// earlier generation describes positions this one has renumbered, so the
    /// steward refuses it rather than mixing it in
    /// (´req:standardisation:lifecycle-entries´). Width alone would not do:
    /// a rebuild may permute positions without changing how many there are.
    pub layout_generation: u64,

    // ─── Marginalisation approximation ───────────────────────────────
    /// What repeated marginalisation has cost the models, folded across every
    /// lifecycle event this process has served
    /// (´alg:gaussian:regularised-schur´).
    ///
    /// A single event's error says how much precision that removal retained
    /// over the exact marginal; it cannot say whether the models are drifting.
    /// The corpus states no law composing the events
    /// (´inv:guarantee:structural-exactness´), so this is a monitor rather
    /// than a bound, and it is what a host reads to decide that a recompute
    /// has become worth its cost.
    ///
    /// The whole-state checkpoint carries the ledger with the models it
    /// describes, so a restart preserves the deployment's accounting rather
    /// than presenting earlier approximation as though it had not happened
    /// (´dec:durability:checkpoint-journal´).
    pub marginalisation_errors: MarginalisationErrorLedger,

    // ─── Hibernation archive (´alg:registry:hibernation´) ────────────
    /// The parameter blocks hibernating deregistrations have kept, keyed by
    /// the identifier each was deregistered under.
    ///
    /// It lives here because this is the Assayer's own custody: the steward
    /// is the archive's only writer (´dec:concurrency:single-steward´), and
    /// the whole-state checkpoint carries it across a restart with the models
    /// it describes (´dec:durability:checkpoint-journal´). It is deliberately
    /// not projected into the published snapshot — no assessment or label
    /// reads a block belonging to an entity that is not registered.
    pub hibernation: crate::model::hibernation::HibernationArchive,
}

impl WorkingCopy {
    // ─────────────────────────────────────────────────────────────────────
    // Construction
    // ─────────────────────────────────────────────────────────────────────

    /// Creates a cold-start working copy with isotropic priors.
    ///
    /// The operational and sister models start at `max(p, dimension_map.p)`
    /// so the model dimension is always at least as large as the minimum
    /// `DimensionMap` layout (1 bias + 15 aggregates = 16).
    /// Anchor starts at [`P_A`] (15) with the same `lambda_prior`
    /// (´dec:substrate:anchor-invariant´).
    ///
    /// `n_recompute` is the Cholesky recomputation cadence from
    /// `CholeskyConfig::n_recompute`, default 1000
    /// (´dec:posterior:recomputation-trigger´).
    #[must_use]
    pub fn cold_start(p: usize, config: &ModelConfig, n_recompute: u32) -> Self {
        let dimension_map = DimensionMap::rebuild_no_interactions(
            0,                          // p_sig
            &indexmap::IndexMap::new(), // sentinels
            &[],                        // identity_dimensions
            &indexmap::IndexMap::new(), // competitive_cells
            &[],                        // spatial_axis_ids
        );

        // Ensure model dimension is at least the DimensionMap minimum.
        let actual_p = p.max(dimension_map.p);

        // Every position is a z-score here, whose prior is the neutral pair
        // this constructor has always published, so taking the moments from
        // the classes changes no value — it only removes the second place
        // that knew what a z-score's moments are.
        let feature_classes = vec![FeatureClass::ZScore; dimension_map.p];
        let (feature_means, feature_variances) = init_from_priors(&feature_classes);
        let n_init = crate::feature::standardisation::StandardisationConfig::default().n_init as usize;

        Self {
            operational: BayesianLinearModel::new(actual_p, config.lambda_prior, n_recompute),
            sister: BayesianLinearModel::new(actual_p, config.lambda_prior, n_recompute),
            anchor: BayesianLinearModel::new(P_A, config.lambda_prior, n_recompute),
            outcome_models: IndexMap::new(),
            kappa_sister: 1.0,
            kappa_anchor: 1.0,
            p_positive_global: 0.5,
            p_positive_eligible: 0.5,
            kappa_v: 1.0,
            last_processed_label_seq: 0,
            last_label_time: Instant::now(),
            cold_ramp: ColdRamp::new(feature_means.clone(), feature_variances.clone(), n_init, 0),
            layout_generation: 0,
            marginalisation_errors: MarginalisationErrorLedger::default(),
            hibernation: crate::model::hibernation::HibernationArchive::default(),
            // Sized to the layout they standardise, which may be narrower
            // than the legacy `p` parameter's model dimension.
            feature_means,
            feature_variances,
            feature_classes,
            dimension_map,
            drift_state: HashMap::new(),
        }
    }

    /// Creates a cold-start working copy with dimension derived from schema.
    ///
    /// Unlike [`cold_start`](Self::cold_start), this constructor derives
    /// the model dimension `p` from the structural declarations: signal
    /// count `p_sig` and interaction templates. The resulting
    /// `DimensionMap` includes the interaction block (Block 7).
    ///
    /// At cold start with no Sentinels, no axes, no identity dimensions:
    ///
    /// $$p = 1 + 15 + p_\text{sig} + n_\text{interactions}$$
    ///
    /// where $n_\text{interactions}$ comes from Type-4 (always-present)
    /// templates only (other template types require Sentinels or
    /// identity dimensions to contribute features).
    ///
    /// # Cross-References
    ///
    /// - (´cav:construction:schema-fixity´) — the construction-time declarations the dimension is derived from
    /// - (´dec:vector:block-order´) — the feature vector's block structure
    #[must_use]
    pub fn cold_start_from_schema(
        p_sig: usize,
        interaction_templates: Vec<InteractionTemplate>,
        signal_classes: &[FeatureClass],
        config: &ModelConfig,
        n_recompute: u32,
        n_init: u32,
    ) -> Self {
        let dimension_map = DimensionMap::rebuild(
            p_sig,
            &indexmap::IndexMap::new(), // sentinels: empty
            &[],                        // identity_dimensions: empty
            &indexmap::IndexMap::new(), // competitive_cells: empty
            &[],                        // spatial_axis_ids
            interaction_templates,
        );
        let p = dimension_map.p;

        // Every position's class from the three authorities, rather than the
        // generic standardising class assigned to all of them at once
        // (´entry:assayer:wl-feature-cold-start-classes´).
        let feature_classes = assign_feature_classes(&dimension_map, signal_classes);

        // Step 1 of (´alg:standardisation:batch-initialisation´): cold
        // construction publishes the classes' own prior moments, the phase
        // `WaitingForInit` and a count of zero. The priors were held back
        // while the discontinuity they exposed was still there — a cold
        // instance moved its whole coordinate system in one step at the
        // hundredth assessment — and the ramp is what removed it. The
        // uncertainty now walks that same distance a hundredth at a time,
        // from about 2.52 down to about 0.79.
        let (feature_means, feature_variances) = init_from_priors(&feature_classes);

        Self {
            operational: BayesianLinearModel::new(p, config.lambda_prior, n_recompute),
            sister: BayesianLinearModel::new(p, config.lambda_prior, n_recompute),
            anchor: BayesianLinearModel::new(P_A, config.lambda_prior, n_recompute),
            outcome_models: IndexMap::new(),
            kappa_sister: 1.0,
            kappa_anchor: 1.0,
            p_positive_global: config.p_plus_init,
            p_positive_eligible: config.p_plus_init,
            kappa_v: 1.0,
            last_processed_label_seq: 0,
            last_label_time: Instant::now(),
            cold_ramp: ColdRamp::new(feature_means.clone(), feature_variances.clone(), n_init as usize, 0),
            layout_generation: 0,
            marginalisation_errors: MarginalisationErrorLedger::default(),
            hibernation: crate::model::hibernation::HibernationArchive::default(),
            feature_means,
            feature_variances,
            feature_classes,
            dimension_map,
            drift_state: HashMap::new(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Cold ramp (´alg:standardisation:batch-initialisation´)
    // ─────────────────────────────────────────────────────────────────────

    /// Takes one accepted cold observation and republishes the moments.
    ///
    /// Returns `true` when this observation was the one that reached the
    /// horizon, so the caller emits the completion event exactly once.
    ///
    /// The observation is refused, and nothing advances, when it was
    /// assembled under a layout this working copy has since renumbered, or
    /// when the ramp has already reached its horizon. Neither refusal is
    /// silent to the caller: it is the return of `None`
    /// (´dec:concurrency:no-silent-drop´).
    pub fn accept_cold_observation(&mut self, phi_raw: &[f64], layout_generation: u64, v_floor: f64) -> Option<bool> {
        if layout_generation != self.layout_generation || self.cold_ramp.is_complete() {
            return None;
        }

        self.cold_ramp.observe(phi_raw);
        let (means, variances) = self.cold_ramp.publish(&self.feature_classes, v_floor);
        self.feature_means = means;
        self.feature_variances = variances;

        Some(self.cold_ramp.is_complete())
    }

    /// Rebases the ramp on what a lifecycle event has just published.
    ///
    /// The layout generation advances here and nowhere else, so an
    /// observation assembled before the change is refused on arrival rather
    /// than mixed into positions it does not describe
    /// (´req:standardisation:lifecycle-entries´).
    pub fn rebase_cold_ramp(&mut self) {
        self.layout_generation = self.layout_generation.wrapping_add(1);
        let generation = self.layout_generation;
        self.cold_ramp
            .rebase(&self.feature_means, &self.feature_variances, generation);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Snapshot projection
    // ─────────────────────────────────────────────────────────────────────

    /// Projects this working copy into an immutable `ModelSnapshot`.
    ///
    /// Clones μ and Σ for all models. B is excluded
    /// (´dec:retention:precision-excluded´).
    #[must_use]
    pub fn to_snapshot(&self, version: u64) -> ModelSnapshot {
        self.to_snapshot_at(version, Instant::now())
    }

    /// Projects this working copy into an immutable `ModelSnapshot` stamped
    /// with the supplied monotonic publication instant.
    ///
    /// Engine publication passes its injected clock reading here. The
    /// convenience [`Self::to_snapshot`] remains for isolated construction
    /// tests that do not own an engine clock.
    #[must_use]
    pub fn to_snapshot_at(&self, version: u64, publish_timestamp: Instant) -> ModelSnapshot {
        let outcome_snapshots = self
            .outcome_models
            .iter()
            .map(|(&axis_id, wam)| {
                let state = AxisModelState {
                    name: wam.name.clone(),
                    parameters: wam.model.to_parameters(),
                    kappa_a: wam.kappa_a,
                    gamma_a: wam.gamma,
                    spatial: wam.spatial,
                    eligibility: wam.eligibility,
                };
                (axis_id, state)
            })
            .collect();

        ModelSnapshot {
            version,
            publish_timestamp,
            operational: self.operational.to_parameters(),
            sister: self.sister.to_parameters(),
            anchor: self.anchor.to_parameters(),
            floor_masses: super::published::CoreFloorMasses::from_models(&self.operational, &self.sister, &self.anchor),
            outcome_models: outcome_snapshots,
            dimension_map: self.dimension_map.clone(),
            feature_means: self.feature_means.clone(),
            feature_variances: self.feature_variances.clone(),
            // The coordinate state this snapshot *is*, so an assessment that
            // acquires it can report the phase and count its own result was
            // scored in rather than wherever the ramp has since reached
            // (´tab:monitoring:standardisation-transition´).
            standardisation_phase: self.cold_ramp.phase(),
            standardisation_observations: self.cold_ramp.accepted(),
            layout_generation: self.layout_generation,
            p_positive_global: self.p_positive_global,
            p_positive_eligible: self.p_positive_eligible,
            kappa_v: self.kappa_v,
            kappa_sister: self.kappa_sister,
            kappa_anchor: self.kappa_anchor,
            // TODO ´todo:code:project-the-live-calibration-buffer-summary´: project the live calibration buffer summary
            // here instead of defaulting and patching after construction
            // (´constr:platt:buffer´).
            calibration_buffer: CalibrationBufferSnapshot::default(),
            drift: self.drift_state.clone(),
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Snapshot restoration
    // ─────────────────────────────────────────────────────────────────────

    /// Reconstructs a working copy from a `ModelSnapshot`.
    ///
    /// B is recomputed as Σ⁻¹ via `cholesky_inverse` for each model.
    /// The cadence-tracking fields the recomputation trigger reads
    /// (´dec:posterior:recomputation-trigger´) are reset to defaults, and
    /// `labels_since_recompute` is set to 0 (fresh recompute).
    ///
    /// The three core models come back carrying the floor masses the snapshot
    /// published beside their parameters (´dec:posterior:spectral-floor´).
    /// They are not derivable from Σ, so a revert that dropped them would
    /// hand back models reading as though every unit of precision behind that
    /// covariance had been earned from labels — the direction that
    /// under-reports uncertainty, on a path taken precisely because something
    /// has already gone wrong. The outcome axes carry none because the
    /// snapshot publishes none for them, which is a statement about what was
    /// published rather than about what those models held.
    ///
    /// # Errors
    ///
    /// Returns `RevertError` if any model's Cholesky inverse fails.
    pub fn from_snapshot(
        snapshot: &ModelSnapshot,
        config: &ModelConfig,
        n_recompute: u32,
        horizon: usize,
    ) -> Result<Self, RevertError> {
        let operational = BayesianLinearModel::from_parameters(
            &snapshot.operational,
            &snapshot.floor_masses.operational,
            config.lambda_prior,
            ModelId::Operational,
            n_recompute,
        )?;

        let sister = BayesianLinearModel::from_parameters(
            &snapshot.sister,
            &snapshot.floor_masses.sister,
            config.lambda_prior,
            ModelId::Sister,
            n_recompute,
        )?;

        let anchor = BayesianLinearModel::from_parameters(
            &snapshot.anchor,
            &snapshot.floor_masses.anchor,
            config.lambda_prior,
            ModelId::Anchor,
            n_recompute,
        )?;

        // An outcome axis's masses are not among what a snapshot publishes,
        // so there is nothing to restore and nothing is invented: the axis
        // comes back reading no borrowed precision, which is exactly what the
        // snapshot says about it.
        let unpublished_mass = crate::model::parameters::FloorMass::default();

        let mut outcome_models = IndexMap::new();
        for (&axis_id, axis_state) in &snapshot.outcome_models {
            let model = BayesianLinearModel::from_parameters(
                &axis_state.parameters,
                &unpublished_mass,
                config.lambda_prior,
                ModelId::OutcomeAxis(axis_id),
                n_recompute,
            )?;
            outcome_models.insert(
                axis_id,
                WorkingAxisModel {
                    name: axis_state.name.clone(),
                    // The snapshot carries no description
                    // (´def:publication:model-snapshot´); the CP5/CP6
                    // revert preserves the live one across this call.
                    description: String::new(),
                    model,
                    kappa_a: axis_state.kappa_a,
                    gamma: axis_state.gamma_a,
                    spatial: axis_state.spatial,
                    eligibility: axis_state.eligibility,
                },
            );
        }

        Ok(Self {
            operational,
            sister,
            anchor,
            outcome_models,
            kappa_sister: snapshot.kappa_sister,
            kappa_anchor: snapshot.kappa_anchor,
            p_positive_global: snapshot.p_positive_global,
            p_positive_eligible: snapshot.p_positive_eligible,
            kappa_v: snapshot.kappa_v,
            last_processed_label_seq: 0,
            last_label_time: Instant::now(),
            feature_means: snapshot.feature_means.clone(),
            feature_variances: snapshot.feature_variances.clone(),
            // The snapshot carries no classes (´def:publication:model-snapshot´);
            // the CP5/CP6 revert preserves the live vector across this call.
            feature_classes: vec![FeatureClass::ZScore; snapshot.dimension_map.p],
            // Nor does it carry the ramp's base and sample. What it does carry
            // is the pair that was published, the phase and the count, and a
            // ramp rebuilt from those publishes and reports the same things —
            // which is what a revert to this snapshot means. The CP5/CP6
            // revert preserves the live ramp across this call in the same way
            // it preserves the classes.
            cold_ramp: ColdRamp::from_published(
                &snapshot.feature_means,
                &snapshot.feature_variances,
                snapshot.standardisation_observations,
                horizon,
                snapshot.layout_generation,
            ),
            layout_generation: snapshot.layout_generation,
            marginalisation_errors: MarginalisationErrorLedger::default(),
            // Nor the hibernation archive, on the same terms as the classes:
            // the snapshot is what an assessment reads, and no assessment
            // reads the archive (´alg:registry:hibernation´). The CP5/CP6
            // revert preserves the live archive across this call.
            hibernation: crate::model::hibernation::HibernationArchive::default(),
            dimension_map: snapshot.dimension_map.clone(),
            drift_state: snapshot.drift.clone(),
        })
    }

    // ─────────────────────────────────────────────────────────────────────
    // Checkpoint persistence (´dec:durability:checkpoint-journal´)
    // ─────────────────────────────────────────────────────────────────────

    /// Builds a `CheckpointPayload` from the current working copy.
    ///
    /// Captures full model state — μ, B, Σ, and the recomputation
    /// tracking fields (´dec:posterior:recomputation-trigger´) — plus
    /// calibration scalars, sequencing, and Layer 2 state (ledger,
    /// identity, dimension map).
    ///
    /// # Arguments
    ///
    /// * `timestamp` — Checkpoint timestamp for time-decay on restore
    /// * `ledger_state` — Pre-captured per-Sentinel ledger state
    /// * `identity_state` — Pre-captured per-dimension identity state
    ///
    /// Structural metadata fields (´cav:construction:schema-fixity´) are populated as empty.
    /// Use [`to_checkpoint_payload_with_structural`](Self::to_checkpoint_payload_with_structural)
    /// to include signal schema / interaction templates
    /// for mismatch detection on restore.
    #[cfg(feature = "serde")]
    #[must_use]
    pub fn to_checkpoint_payload(
        &self,
        timestamp: crate::types::PersistentTimestamp,
        ledger_state: Vec<(
            crate::types::SentinelId,
            crate::persistence::checkpoint::SentinelLedgerPayload,
        )>,
        identity_state: Vec<(
            crate::types::DimensionId,
            crate::persistence::checkpoint::IdentityDimensionPayload,
        )>,
    ) -> crate::persistence::checkpoint::CheckpointPayload {
        self.to_checkpoint_payload_with_structural(
            timestamp,
            ledger_state,
            identity_state,
            crate::persistence::checkpoint::StructuralMetadata::default(),
        )
    }

    /// Builds a `CheckpointPayload` including structural metadata for
    /// mismatch detection on restore (´cav:construction:schema-fixity´).
    ///
    /// Identical to [`to_checkpoint_payload`](Self::to_checkpoint_payload)
    /// but additionally captures the construction-time signal schema,
    /// and interaction templates.
    #[cfg(feature = "serde")]
    #[must_use]
    pub fn to_checkpoint_payload_with_structural(
        &self,
        timestamp: crate::types::PersistentTimestamp,
        ledger_state: Vec<(
            crate::types::SentinelId,
            crate::persistence::checkpoint::SentinelLedgerPayload,
        )>,
        identity_state: Vec<(
            crate::types::DimensionId,
            crate::persistence::checkpoint::IdentityDimensionPayload,
        )>,
        structural: crate::persistence::checkpoint::StructuralMetadata,
    ) -> crate::persistence::checkpoint::CheckpointPayload {
        use crate::persistence::checkpoint::{CheckpointAxisState, CheckpointPayload};

        let outcome_models = self
            .outcome_models
            .iter()
            .map(|(&axis_id, wam)| {
                (
                    axis_id.0,
                    CheckpointAxisState {
                        model: wam.model.to_checkpoint_state(),
                        kappa_a: wam.kappa_a,
                        gamma: wam.gamma,
                        spatial: wam.spatial,
                        name: wam.name.clone(),
                        description: wam.description.clone(),
                        eligibility: wam.eligibility,
                    },
                )
            })
            .collect();

        CheckpointPayload {
            operational: self.operational.to_checkpoint_state(),
            sister: self.sister.to_checkpoint_state(),
            anchor: self.anchor.to_checkpoint_state(),
            outcome_models,
            kappa_sister: self.kappa_sister,
            kappa_anchor: self.kappa_anchor,
            p_positive_global: self.p_positive_global,
            p_positive_eligible: self.p_positive_eligible,
            kappa_v: self.kappa_v,
            last_processed_label_seq: self.last_processed_label_seq,
            checkpoint_timestamp: timestamp,
            ledger_state,
            identity_state,
            dimension_map: self.dimension_map.clone(),
            signal_schema: structural.signal_schema,
            interaction_templates: structural.interaction_templates,
            concordance_state: crate::health::ConcordanceCheckpointState::default(),
            marginalisation_errors: self.marginalisation_errors.clone(),
            // The archive crosses the restart with the models it describes.
            // It has to: it is keyed by identifiers the host will register
            // again, and an archive that did not survive would silently turn
            // every restore attempt after a restart into an extension at the
            // prior (´alg:registry:hibernation´).
            hibernation: self.hibernation.clone(),
            drift_state: self
                .drift_state
                .iter()
                .map(|(id, drift)| (id.clone(), drift.clone()))
                .collect(),
            feature_means: self.feature_means.clone(),
            feature_variances: self.feature_variances.clone(),
            feature_classes: self.feature_classes.clone(),
            // Step 7 of (´alg:standardisation:batch-initialisation´): the ramp
            // travels whole — the base it mixes against, the sample it has
            // accepted and the count that positions it — so a resumed ramp
            // publishes what an uninterrupted one would have published at the
            // same count, rather than one run's sample against another run's
            // assumptions (´dec:durability:ramp-resumption´).
            cold_ramp: crate::persistence::checkpoint::ColdRampPayload {
                phase: self.cold_ramp.phase(),
                accepted: self.cold_ramp.accepted(),
                base_means: self.cold_ramp.base_means().to_vec(),
                base_variances: self.cold_ramp.base_variances().to_vec(),
                running_mean: self.cold_ramp.accumulator().running_mean.clone(),
                running_m2: self.cold_ramp.accumulator().running_m2.clone(),
                layout_generation: self.layout_generation,
            },
            owner_state: crate::persistence::checkpoint::OwnerCheckpointState::default(),
        }
    }

    /// Restores a `WorkingCopy` from a `CheckpointPayload`.
    ///
    /// Unlike `from_snapshot`, this does *not* require `cholesky_inverse`
    /// because the precision matrix B is included in the checkpoint.
    ///
    /// `horizon` is the `N_init` the restoring instance is configured with,
    /// not one carried in the payload: a restored ramp recomputes its position
    /// against the horizon in force, so a count at or past a shortened horizon
    /// completes on arrival and publishes the empirical moments, and the
    /// one-time change in mixture weight is disclosed by this restore's own
    /// coordinate-version change (´dec:durability:ramp-resumption´).
    ///
    /// `lambda_prior` arrives the same way and for the same reason: the three
    /// core models take the prior precision the restoring instance is
    /// configured with, which is what their Schur regularisation is derived
    /// against (´alg:gaussian:regularised-schur´). An outcome axis takes its
    /// own instead, because a registration sets it per axis and the payload
    /// already carries it.
    ///
    /// # Errors
    ///
    /// Returns [`BuildError::CheckpointDimensionMismatch`] naming the first
    /// model whose stored state disagrees with the width it declares for
    /// itself, and [`BuildError::CheckpointNotPositiveDefinite`] naming the
    /// first model whose stored precision matrix does not factor. The restore
    /// is refused whole rather than per model: the models are one posterior
    /// read under three heads and a working copy assembled from some restored
    /// parameters and some fresh priors is a state no host asked for and none
    /// can reason about.
    ///
    /// [`BuildError::CheckpointNotPositiveDefinite`]: crate::error::BuildError::CheckpointNotPositiveDefinite
    /// [`BuildError::CheckpointDimensionMismatch`]: crate::error::BuildError::CheckpointDimensionMismatch
    #[cfg(feature = "serde")]
    pub fn from_checkpoint_payload(
        payload: &crate::persistence::checkpoint::CheckpointPayload,
        horizon: usize,
        lambda_prior: f64,
    ) -> Result<Self, crate::error::BuildError> {
        use crate::persistence::checkpoint::CheckpointModelState;

        fn restore_model(
            state: &CheckpointModelState,
            prior_precision: f64,
            model: ModelId,
        ) -> Result<BayesianLinearModel, crate::error::BuildError> {
            use crate::linalg::convert::{vec_to_col, vec_to_mat};
            use crate::linalg::symmetric::SymmetricMatrix;

            check_extents(state, &model)?;

            let mu = vec_to_col(&state.parameters.mu);
            let covariance_mat = vec_to_mat(&state.parameters.covariance_data, state.parameters.p, state.parameters.p);
            let covariance = SymmetricMatrix::from_computation(covariance_mat);

            BayesianLinearModel::from_checkpoint(
                mu,
                state.precision.clone(),
                covariance,
                prior_precision,
                model,
                // TODO(2026-05-23) ´todo:code:restore-conservatively-by´: Restore conservatively by
                // resetting labels_since_recompute and
                // consecutive_clean_recomputes to 0, while preserving
                // n_recompute_effective and health diagnostics, against what
                // the recomputation trigger reads
                // (´dec:posterior:recomputation-trigger´).
                state.labels_since_recompute,
                state.n_recompute_effective,
                state.consecutive_clean_recomputes,
                state.last_diagonal_ratio,
                state.last_sync_error,
                state.sync_error_shortenings,
                state.total_recomputes,
                state.measurements,
                state.floored_rebuilds,
                state.cascade_terminus_events,
                state.last_dimensions_at_floor,
                state.alarms,
                state.baseline,
                state.spectral_floor_mass,
                state.clamp_mass.clone(),
            )
        }

        let operational = restore_model(&payload.operational, lambda_prior, ModelId::Operational)?;
        let sister = restore_model(&payload.sister, lambda_prior, ModelId::Sister)?;
        let anchor = restore_model(&payload.anchor, lambda_prior, ModelId::Anchor)?;

        let mut outcome_models = IndexMap::new();
        for &(axis_id_val, ref axis_state) in &payload.outcome_models {
            let axis_id = crate::types::OutcomeAxisId(axis_id_val);
            let model = restore_model(&axis_state.model, axis_state.kappa_a, ModelId::OutcomeAxis(axis_id))?;
            outcome_models.insert(
                axis_id,
                WorkingAxisModel {
                    name: axis_state.name.clone(),
                    description: axis_state.description.clone(),
                    model,
                    kappa_a: axis_state.kappa_a,
                    gamma: axis_state.gamma,
                    spatial: axis_state.spatial,
                    eligibility: axis_state.eligibility,
                },
            );
        }

        Ok(Self {
            operational,
            sister,
            anchor,
            outcome_models,
            kappa_sister: payload.kappa_sister,
            kappa_anchor: payload.kappa_anchor,
            p_positive_global: payload.p_positive_global,
            p_positive_eligible: payload.p_positive_eligible,
            kappa_v: payload.kappa_v,
            last_processed_label_seq: payload.last_processed_label_seq,
            last_label_time: Instant::now(),
            feature_means: payload.feature_means.clone(),
            feature_variances: payload.feature_variances.clone(),
            feature_classes: payload.feature_classes.clone(),
            cold_ramp: ColdRamp::from_parts(
                payload.cold_ramp.base_means.clone(),
                payload.cold_ramp.base_variances.clone(),
                payload.cold_ramp.running_mean.clone(),
                payload.cold_ramp.running_m2.clone(),
                payload.cold_ramp.accepted,
                horizon,
                payload.cold_ramp.layout_generation,
            ),
            layout_generation: payload.cold_ramp.layout_generation,
            marginalisation_errors: payload.marginalisation_errors.clone(),
            hibernation: payload.hibernation.clone(),
            dimension_map: payload.dimension_map.clone(),
            drift_state: payload
                .drift_state
                .iter()
                .map(|(id, drift)| (id.clone(), drift.clone()))
                .collect(),
        })
    }

    /// Applies bulk time-decay to all models.
    ///
    /// Used by recovery to account for time elapsed since the checkpoint.
    #[cfg(feature = "serde")]
    pub fn apply_time_decay(&mut self, decay_factor: f64) {
        self.operational.apply_time_decay(decay_factor);
        self.sister.apply_time_decay(decay_factor);
        self.anchor.apply_time_decay(decay_factor);
        for wam in self.outcome_models.values_mut() {
            wam.model.apply_time_decay(decay_factor);
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Hibernation (´alg:registry:hibernation´)
    // ─────────────────────────────────────────────────────────────────────

    /// Copies out the self-structure an entity's own positions carry, across
    /// every full-dimension model and the standardisation vectors.
    ///
    /// Called before the marginalisation that removes those positions, which
    /// is the only order in which the block exists to be taken: the Schur
    /// complement folds the departing block into the survivors and then drops
    /// it (´alg:registry:sentinel-deregistration´).
    ///
    /// The positions are taken in the order given, and the layout handed in
    /// is the record's statement of the arrangement they were archived under.
    /// The anchor contributes nothing: it is a fixed-dimension projection that
    /// no lifecycle event touches (´dec:substrate:anchor-invariant´).
    pub(crate) fn archive_self_structure(
        &self,
        positions: &[usize],
        layout: crate::model::hibernation::HibernationLayout,
        now: &crate::types::PersistentTimestamp,
    ) -> crate::model::hibernation::HibernationRecord {
        use crate::model::hibernation::{HIBERNATION_RECORD_VERSION, HibernationRecord};

        let mut blocks = Vec::with_capacity(2 + self.outcome_models.len());
        blocks.push((ModelId::Operational, self.operational.self_structure_block(positions)));
        blocks.push((ModelId::Sister, self.sister.self_structure_block(positions)));
        for (&axis_id, wam) in &self.outcome_models {
            blocks.push((ModelId::OutcomeAxis(axis_id), wam.model.self_structure_block(positions)));
        }

        HibernationRecord {
            version: HIBERNATION_RECORD_VERSION,
            archived_at: *now,
            archived_label_seq: self.last_processed_label_seq,
            layout,
            feature_means: positions.iter().map(|&i| self.feature_means[i]).collect(),
            feature_variances: positions.iter().map(|&i| self.feature_variances[i]).collect(),
            blocks,
        }
    }

    /// Writes an admitted record over the contiguous positions an extension
    /// has just created, model by model.
    ///
    /// Each model ages its own block on its own label-indexed rate — the
    /// operational rate, the sister rate, and each axis's own — against the
    /// one time-indexed core rate, which is the pairing the forgetting-rate
    /// table already draws (´tab:risk:forgetting-rates´). A model whose aged
    /// block would sink below the replenishment floor keeps the extension's
    /// prior instead, and so does a model the record never held a block for.
    ///
    /// This is where the replenishment floor is decided, and it is the only
    /// place it could be decided. Admission asks its questions of the record as
    /// a whole and answers with one refusal covering all of it; the floor is
    /// asked of one model's block under that model's own rate, so the same
    /// archived block stands above the floor for a model that forgets slowly
    /// and sinks below it for one that forgets fast
    /// (´req:gaussian:prior-replenishment-floor´). A record-level verdict would
    /// have to be wrong about one of them. The answer's shape follows from
    /// that: a count rather than a refusal, with the models that took their
    /// block and the models left at the prior reported separately, and
    /// `RestoreReport::at_prior` is the record of what the floor decided.
    ///
    /// The standardisation entries follow the models rather than leading
    /// them: they are written only where at least one model took its block,
    /// so a returning entity is never standardised against moments no
    /// surviving parameter was learned under.
    pub(crate) fn restore_self_structure(
        &mut self,
        record: &crate::model::hibernation::HibernationRecord,
        start: usize,
        now: &crate::types::PersistentTimestamp,
        model_config: &ModelConfig,
        gamma_time: f64,
    ) -> crate::model::hibernation::RestoreReport {
        use crate::model::hibernation::{RestoreReport, survives_floor};

        let label_seq = self.last_processed_label_seq;
        let floor = model_config.lambda_floor;
        let mut report = RestoreReport::default();

        let apply = |model: &mut BayesianLinearModel, id: &ModelId, gamma_label: f64, report: &mut RestoreReport| {
            let decay = record.decay(now, label_seq, gamma_time, gamma_label);
            match record.block(id) {
                Some(block) if survives_floor(block, decay, floor) => {
                    model.restore_self_structure_block(start, block, decay);
                    report.restored += 1;
                }
                _ => report.at_prior += 1,
            }
        };

        apply(
            &mut self.operational,
            &ModelId::Operational,
            model_config.gamma_opr,
            &mut report,
        );
        apply(&mut self.sister, &ModelId::Sister, model_config.gamma_inh, &mut report);
        for (&axis_id, wam) in &mut self.outcome_models {
            let decay = record.decay(now, label_seq, gamma_time, wam.gamma);
            match record.block(&ModelId::OutcomeAxis(axis_id)) {
                Some(block) if survives_floor(block, decay, floor) => {
                    wam.model.restore_self_structure_block(start, block, decay);
                    report.restored += 1;
                }
                _ => report.at_prior += 1,
            }
        }

        if report.any_restored() {
            for (offset, (&mean, &variance)) in record.feature_means.iter().zip(record.feature_variances.iter()).enumerate() {
                self.feature_means[start + offset] = mean;
                self.feature_variances[start + offset] = variance;
            }
        }

        self.warn_if_anchor_dimension_changed("restore_self_structure");
        report
    }

    // ─────────────────────────────────────────────────────────────────────
    // Lifecycle helpers (´inv:guarantee:axis-lifecycle´)
    // ─────────────────────────────────────────────────────────────────────

    /// Emits a warning if the fixed anchor model dimension changes.
    fn warn_if_anchor_dimension_changed(&self, operation: &'static str) {
        let anchor_dim = self.anchor.dim();
        debug_assert_eq!(anchor_dim, P_A, "anchor dimension changed during {operation}");
        if anchor_dim != P_A {
            tracing::warn!(operation, anchor_dim, expected = P_A, "Anchor dimension invariant violated");
        }
    }

    /// Extends every full-dimension model by `r` dimensions.
    ///
    /// "Full-dimension" means every model that tracks the complete
    /// feature vector `φ`: operational, sister, and each registered
    /// outcome-axis prediction model. The anchor model is **not**
    /// extended — its dimension is fixed at [`P_A`]
    /// (´dec:substrate:anchor-invariant´).
    ///
    /// Every outcome-axis prediction model undergoes exact extension on
    /// each lifecycle event (´inv:guarantee:structural-exactness´).
    pub(crate) fn extend_all_full_dim(&mut self, r: usize, prior_precision: f64) {
        if r == 0 {
            return;
        }
        self.operational.extend(r, prior_precision);
        self.sister.extend(r, prior_precision);
        for wam in self.outcome_models.values_mut() {
            wam.model.extend(r, prior_precision);
        }
        self.warn_if_anchor_dimension_changed("extend_all_full_dim");
    }

    /// Rearranges every full-dimension model and the standardisation vectors
    /// into the rebuilt map's canonical index order.
    ///
    /// The same model set `extend_all_full_dim` extends, plus the three
    /// standardisation vectors, because the invariant that binds them requires
    /// one index semantics across all of them
    /// (´inv:dimension:version-consistency´). The anchor is a fixed-dimension
    /// projection and is not rearranged; its projection indices are recomputed
    /// by the rebuild that produced this permutation.
    pub(crate) fn permute_all_full_dim(&mut self, permutation: &LayoutPermutation) {
        if permutation.is_identity() {
            return;
        }
        let source = permutation.as_slice();
        self.operational.permute(source);
        self.sister.permute(source);
        for wam in self.outcome_models.values_mut() {
            wam.model.permute(source);
        }
        permutation.apply_to(&mut self.feature_means);
        permutation.apply_to(&mut self.feature_variances);
        permutation.apply_to(&mut self.feature_classes);
        self.warn_if_anchor_dimension_changed("permute_all_full_dim");
    }

    /// Marginalises every full-dimension model by `remove_indices`.
    ///
    /// Returns whether any model fell back to `B_kk` and the aggregate
    /// diagnostics for this removal. Every model is marginalised: nothing
    /// upstream is handed an error from a factorisation
    /// (´dec:posterior:cascade-never-fails´), so a removal always completes
    /// across the whole model set.
    ///
    /// Iteration order: operational, sister, then each outcome-axis
    /// model in `IndexMap` order. The anchor is **not** marginalised
    /// (fixed dimension).
    ///
    /// TODO(2026-05-23) ´todo:code:add-a-companion-helper-for´: Add a companion helper for
    /// competitive-cell low-rank removal that applies the `Σ_kk` extraction
    /// path (´dec:posterior:half-solve´) across all full-dimension models,
    /// exactly as a lifecycle event must (´inv:guarantee:structural-exactness´).
    pub(crate) fn marginalise_all_full_dim(
        &mut self,
        remove_indices: &[usize],
        schur_config: &SchurConfig,
    ) -> (bool, MarginalisationErrorLedger) {
        // Each model's diagnostics are folded into the ledger before the
        // fallback flag is taken from them. The flag alone was all this joint
        // used to carry out, and it is what a host could read no drift from:
        // it says that some model refused its correction, and nothing about
        // how much approximation the models that accepted theirs committed
        // (´alg:gaussian:regularised-schur´).
        let mut any_fallback = false;
        let mut event_errors = MarginalisationErrorLedger::default();

        let operational = self
            .operational
            .marginalise(remove_indices, schur_config, ModelId::Operational);
        self.marginalisation_errors.record(&operational);
        event_errors.record(&operational);
        any_fallback |= operational.fell_back_to_bkk;

        let sister = self.sister.marginalise(remove_indices, schur_config, ModelId::Sister);
        self.marginalisation_errors.record(&sister);
        event_errors.record(&sister);
        any_fallback |= sister.fell_back_to_bkk;

        for (&axis_id, wam) in &mut self.outcome_models {
            let axis = wam
                .model
                .marginalise(remove_indices, schur_config, ModelId::OutcomeAxis(axis_id));
            self.marginalisation_errors.record(&axis);
            event_errors.record(&axis);
            any_fallback |= axis.fell_back_to_bkk;
        }

        self.warn_if_anchor_dimension_changed("marginalise_all_full_dim");
        (any_fallback, event_errors)
    }
}

/// Refuses a stored model state that disagrees with the width it declares for
/// itself, before anything is reconstructed from it.
///
/// Every restore reaches a model through one reconstruction, and that
/// reconstruction's second step is the conversion widening the flat covariance
/// back into a matrix on the declared width. The conversion asserts in every
/// build rather than only in a checked one, so the extents have to be
/// established ahead of it, and the reconstruction is the one place they can be
/// established for every model at once. The three fixed models and every
/// registered outcome axis reach it by the same call, so a single guard covers
/// all of them and no axis added later can arrive by a route that skips it.
///
/// The clamp mass is the one extent with a second admissible reading. A state
/// written before the coordinate-shaped mass existed carries none, and an
/// absent one is widened to zeros because nothing was accumulated, which is
/// what nothing accumulated looks like. A present one of another width is not
/// that case: it is a measurement that disagrees with the model it was taken
/// on, and widening it would discard a reading in silence.
#[cfg(feature = "serde")]
fn check_extents(
    state: &crate::persistence::checkpoint::CheckpointModelState,
    model: &ModelId,
) -> Result<(), crate::error::BuildError> {
    let declared = state.parameters.p;
    let disagrees = |quantity: &'static str, found: usize| crate::error::BuildError::CheckpointDimensionMismatch {
        model: model.clone(),
        quantity,
        found,
        declared,
    };

    if state.parameters.mu.len() != declared {
        return Err(disagrees("mean entries", state.parameters.mu.len()));
    }

    if state.precision.dim() != declared {
        return Err(disagrees("precision rows", state.precision.dim()));
    }

    // The covariance count has to be the declared width squared, and the square
    // is reached by division rather than by multiplying: a width large enough
    // for the product to wrap is exactly a width no stored count can match, and
    // taking the quotient reaches that verdict instead of computing a wrapped
    // one to compare against.
    let entries = state.parameters.covariance_data.len();
    let squares = if declared == 0 {
        entries == 0
    } else {
        entries.is_multiple_of(declared) && entries / declared == declared
    };
    if !squares {
        return Err(disagrees("covariance entries", entries));
    }

    if !state.clamp_mass.is_empty() && state.clamp_mass.len() != declared {
        return Err(disagrees("clamp-mass entries", state.clamp_mass.len()));
    }

    Ok(())
}
