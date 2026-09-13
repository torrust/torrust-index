// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Composite convergence stage computation.
//!
//! This module provides the [`CompositeConvergenceStage`] enum and the
//! [`compute_composite_stage`] function that combines Core convergence stages
//! into a single overall convergence indicator. Host-owned Companion challenge
//! sufficiency can be carried in composed health reports, but it is not Core
//! state and does not gate the composite stage.
//!
//! # Conservative Gating
//!
//! The composite stage is the **minimum** of all critical process stages.
//! Concordance and Companion challenge effectiveness are NOT gating — they
//! affect output quality but not structural validity.
//!
//! # Cross-References
//!
//! - (´dec:health:on-demand-composite´) — the composite is computed on demand
//!   and may regress
//! - (´dec:challenge:sufficiency-floor-owned-here´) — Companion sufficiency is
//!   the host's figure, not Core state
//! - (´dec:health:non-exhaustive-report´) — why the composed report type stays
//!   open to new fields

// Items in this module are re-exported via crate::health and used by tests.
#![allow(dead_code)]

use super::identity_tracker::IdentityConvergenceTracker;
use super::platt_tracker::PlattConvergenceTracker;
use crate::types::PersistentTimestamp;

// ═══════════════════════════════════════════════════════════════════════════════
// Composite Convergence Stage
// ═══════════════════════════════════════════════════════════════════════════════

/// Overall convergence stage of the Assayer system.
///
/// Represents the minimum convergence level across all critical processes.
/// This is a diagnostic summary and may regress after lifecycle events that
/// extend or marginalise model dimensions.
///
/// # Stage Semantics
///
/// | Stage | Description | Typical Labels |
/// |-------|-------------|----------------|
/// | `ColdStart` | No actionable risk discrimination | <30 eligible |
/// | `AnchorEmerging` | Anchor model active; sister not yet fit | 30–79 eligible |
/// | `PreCalibration` | Directional but Platt not yet fitted | 80+ eligible |
/// | `SisterConverging` | Platt actively refitting | varies |
/// | `InteractionMaturing` | Identity dimensions stabilising | varies |
/// | `SteadyState` | All processes converged | ~500+ |
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
#[repr(u8)]
pub enum CompositeConvergenceStage {
    /// No model updates yet; decisions use prior-only estimates.
    #[default]
    ColdStart = 0,
    /// Anchor model is active; sister model not yet fitted.
    AnchorEmerging = 1,
    /// First Platt calibration fit completed; sister model emerging.
    PreCalibration = 2,
    /// Platt calibration is actively refitting; calibration converging.
    SisterConverging = 3,
    /// Identity dimensions are forming and stabilising.
    InteractionMaturing = 4,
    /// All critical processes have converged.
    SteadyState = 5,
}

impl CompositeConvergenceStage {
    /// Returns `true` if this stage indicates the system is fully operational.
    #[must_use]
    pub const fn is_converged(self) -> bool {
        matches!(self, Self::SteadyState)
    }

    /// Returns `true` if the system has completed at least the first Platt fit.
    #[must_use]
    pub const fn has_platt_fit(self) -> bool {
        matches!(
            self,
            Self::PreCalibration | Self::SisterConverging | Self::InteractionMaturing | Self::SteadyState
        )
    }

    /// Returns the stage name as a static string.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ColdStart => "cold_start",
            Self::AnchorEmerging => "anchor_emerging",
            Self::PreCalibration => "pre_calibration",
            Self::SisterConverging => "sister_converging",
            Self::InteractionMaturing => "interaction_maturing",
            Self::SteadyState => "steady_state",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Configuration Thresholds
// ═══════════════════════════════════════════════════════════════════════════════

/// Minimum labels required to exit `ColdStart` stage.
/// TODO ´todo:code:replace-with-configurable-cold-start-labels´: replace with configurable `cold_start_labels`
/// and evaluate against eligible labels, not total labels
/// (´tab:warmup:stages´).
///
/// ´const:assayer:cold-start-exit-labels´ (´alg:const:count´)
/// ´const:assayer:cold-start-exit-labels-count-1´
const MIN_LABELS_FOR_ANCHOR: u64 = 1;

/// Minimum labels required to enter `PreCalibration` stage.
/// TODO ´todo:code:replace-with-configurable´: replace with configurable
/// `anchor_emergence_labels` (default 80 eligible labels).
///
/// ´const:assayer:sister-emergence-labels´ (´alg:const:count´)
/// ´const:assayer:sister-emergence-labels-count-30´
const MIN_LABELS_FOR_SISTER: u64 = 30;

/// Minimum Platt refits for calibration to be considered converging.
///
/// The lower of the ladder's two counts, and it exists so that a host
/// watching the stage sees it move before the fit has finished, rather
/// than waiting at the stage below through every refit and then jumping
/// (´dec:health:calibration-settled-cuts´).
///
/// ´const:assayer:converging-stage-refits´ (´alg:const:count´)
/// ´const:assayer:converging-stage-refits-count-2´
const MIN_REFITS_FOR_CONVERGING: u32 = 2;

// Platt convergence thresholds are passed as parameters from
// `ConvergenceThresholdConfig`, one of the figures the surfaces defer to
// (´tab:construction:parameters´). See
// `platt_tracker::DEFAULT_PLATT_CONVERGED_*` for test defaults.

// ═══════════════════════════════════════════════════════════════════════════════
// Composite Stage Computation
// ═══════════════════════════════════════════════════════════════════════════════

/// Computes the composite convergence stage.
///
/// Uses conservative gating: the composite equals the minimum of all
/// critical process stages. Concordance and Companion challenge effectiveness
/// are NOT gating factors.
///
/// Companion sufficiency used to be taken here as a deliberately unused
/// argument, and it was removed rather than left standing as a parameter
/// Core refused to read. The figure behind it
/// is the host's and lives with the tracker that reads it
/// (´dec:challenge:sufficiency-floor-owned-here´).
///
/// # Arguments
///
/// * `total_labels` — Total labels processed since startup.
///   TODO ´todo:code:rename-to-total-eligible-labels-and´: rename to `total_eligible_labels` and pass the
///   checkpointed eligible-label clock from the working copy. Stage
///   boundaries 1–3 are defined by eligible labels (´tab:warmup:stages´), so
///   total labels can advance the composite too early.
/// * `platt` — Platt calibration convergence tracker
/// * `identity_trackers` — Per-dimension identity convergence trackers
///
/// # Returns
///
/// The composite convergence stage (minimum of all critical stages).
///
/// # Stage Transitions
///
/// - `ColdStart` → `AnchorEmerging`: `total_labels >= 1`
/// - `AnchorEmerging` → `PreCalibration`: `total_labels >= 30` and first Platt fit
/// - `PreCalibration` → `SisterConverging`: `refits >= 2`
/// - `SisterConverging` → `InteractionMaturing`: `delta_cal <= threshold` and `refits >= min`
/// - `InteractionMaturing` → `SteadyState`: all identity trackers stable
#[must_use]
pub fn compute_composite_stage(
    total_labels: u64,
    platt: &PlattConvergenceTracker,
    identity_trackers: &[&IdentityConvergenceTracker],
    platt_converged_delta_cal: f64,
    platt_converged_refit_count: u32,
    now: &PersistentTimestamp,
) -> CompositeConvergenceStage {
    // TODO ´todo:code:rename-total-labels-to-total-eligible´: rename `total_labels` to `total_eligible_labels`.
    // Stage 0: No labels yet
    if total_labels < MIN_LABELS_FOR_ANCHOR {
        return CompositeConvergenceStage::ColdStart;
    }

    // Stage 1: Anchor dominated (no Platt fit yet)
    if platt.refits_completed == 0 || total_labels < MIN_LABELS_FOR_SISTER {
        return CompositeConvergenceStage::AnchorEmerging;
    }

    // Stage 2: Sister emerging (first Platt fit completed)
    if platt.refits_completed < MIN_REFITS_FOR_CONVERGING {
        return CompositeConvergenceStage::PreCalibration;
    }

    // Stage 3: Calibration converging (multiple refits but not yet converged)
    let platt_converged =
        platt.refits_completed >= platt_converged_refit_count && platt.last_delta_cal <= platt_converged_delta_cal;

    if !platt_converged {
        return CompositeConvergenceStage::SisterConverging;
    }

    // Stage 4: Identity forming (Platt converged, checking identity stability)
    // Uses the per-dimension convergence_stage() which includes the
    // maturity check — a Maturing dimension is not yet Stable.
    let all_identity_stable = identity_trackers.is_empty()
        || identity_trackers
            .iter()
            .all(|tracker| tracker.convergence_stage(now).is_stable());

    if !all_identity_stable {
        return CompositeConvergenceStage::InteractionMaturing;
    }

    // Stage 5: Fully converged
    CompositeConvergenceStage::SteadyState
}

// ═══════════════════════════════════════════════════════════════════════════════
// Convergence Health
// ═══════════════════════════════════════════════════════════════════════════════

/// Aggregated health snapshot of all convergence processes.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConvergenceHealth {
    /// Current composite convergence stage.
    pub composite_stage: CompositeConvergenceStage,
    /// Total labels processed.
    pub total_labels: u64,
    /// Number of Platt refits completed.
    pub platt_refits: u32,
    /// Most recent Platt `delta_cal` value.
    pub platt_delta_cal: f64,
    /// Number of identity dimensions.
    pub identity_dimensions: usize,
    /// Number of stable identity dimensions.
    pub stable_identity_dimensions: usize,
    /// Whether host-owned Companion challenge evidence is sufficient.
    /// TODO ´todo:code:remove-from-core-convergence-health-companion´: remove from Core convergence health; Companion
    /// readiness belongs to the host-owned Companion health report, where the
    /// figure lives (´dec:challenge:sufficiency-floor-owned-here´).
    pub challenge_sufficient: bool,
}
