// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Health and convergence event types.
//!
//! This module provides event types for tracking convergence progress
//! and system health. Events are emitted to a bounded channel; overflow
//! drops events with counter increment (no panic).
//!
//! # Cross-References
//!
//! - (´dec:health:bounded-events´) — one bounded channel that drops with a
//!   counter rather than blocking
//! - (´inv:monitoring:report-only´) — health monitoring reports and never acts

// Items in this module are re-exported via crate::health and used by tests.
#![allow(dead_code)]

use std::sync::atomic::{AtomicU64, Ordering};

use crossbeam_channel::{Sender, TrySendError};

use super::composite::CompositeConvergenceStage;
use super::counters::DriftResetCounter;
use crate::types::{ChannelId, DimensionId, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Health Events
// ═══════════════════════════════════════════════════════════════════════════════

/// Top-level health event category.
///
/// Events are pushed via bounded channel with `try_send`. On overflow,
/// the event is dropped and a counter is incremented.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum HealthEvent {
    /// Convergence-related event (warm-up tracking).
    Convergence(ConvergenceEvent),
    /// Precision/numerical health event.
    Precision(PrecisionHealthEvent),
    /// Lifecycle health event (entity state changes).
    Lifecycle(LifecycleHealthEvent),
}

// ═══════════════════════════════════════════════════════════════════════════════
// Convergence Events
// ═══════════════════════════════════════════════════════════════════════════════

/// Convergence progress event.
///
/// Emitted as the system warms up and models converge, at the milestones the
/// warm-up names (´tab:warmup:milestones´).
///
/// TODO ´todo:code:align-this-enum-with-the-revised´: align this enum with the revised seven Core convergence
/// events by adding `ConcordanceFirstCalibration { thresholds: [f64; 4] }`
/// and moving Companion challenge readiness out of Core convergence events.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ConvergenceEvent {
    /// Batch initialisation completed (`DimensionMap` built, models initialised).
    BatchInitComplete,
    /// A Sentinel's bootstrap phase completed (sufficient observations).
    SentinelBootstrapComplete {
        /// The Sentinel that completed bootstrap.
        sentinel_id: SentinelId,
    },
    /// First Platt calibration fit completed.
    PlattFirstFit {
        /// Fitted kappa for the sister model.
        kappa_sister: f64,
        /// Fitted kappa for the anchor model.
        kappa_anchor: f64,
        /// Calibration quality metric (max |ln `κ_new` − ln `κ_old`|).
        delta_cal: f64,
    },
    /// Platt calibration has converged (`delta_cal` ≤ threshold).
    PlattConverged {
        /// Total Platt refits completed at convergence.
        refits_completed: u32,
        /// Final `delta_cal` at convergence.
        delta_cal: f64,
    },
    /// An identity dimension has stabilised (low change rate).
    IdentityStabilised {
        /// The dimension that stabilised.
        dimension_id: DimensionId,
    },
    /// Host-owned Companion challenge effectiveness for a channel is now usable.
    /// TODO ´todo:code:move-this-out-of-core-convergenceevent´: move this out of Core `ConvergenceEvent`; challenge
    /// effectiveness is host-owned Companion health and does not gate Core
    /// (´dec:challenge:sufficiency-floor-owned-here´).
    ChallengeEffectivenessUsable {
        /// The channel with usable challenge data.
        channel_id: ChannelId,
    },
    /// Composite convergence stage changed.
    CompositeStageChanged {
        /// Previous stage.
        from: CompositeConvergenceStage,
        /// New stage.
        to: CompositeConvergenceStage,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// Precision Health Events
// ═══════════════════════════════════════════════════════════════════════════════

/// Precision and numerical health event.
///
/// Emitted when numerical issues (NaN, Inf) are detected and sanitised,
/// or when Cholesky recomputation encounters non-clean outcomes.
///
/// The variants are what one factorisation can answer
/// (´dec:posterior:repair-cascade´): `CascadeTerminus` and
/// `SyncErrorShortened`.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PrecisionHealthEvent {
    /// Features were sanitised due to NaN/Inf values.
    FeaturesSanitised {
        /// Number of features sanitised.
        count: u32,
    },
    /// A model fell back to prior due to NaN in parameters.
    ModelFallbackToPrior {
        /// Description of the affected model.
        model_description: String,
    },
    /// Cholesky recomputation cascade terminus.
    ///
    /// The factorisation was refused and nothing was offered in its place.
    /// The existing SM-maintained Σ is retained. Severe health warning.
    CascadeTerminus {
        /// Which model encountered the failure.
        model_id: crate::types::ModelId,
        /// Synchronisation error (´def:monitoring:synchronisation-error´) measured before the failed recomputation.
        sync_error: f64,
    },
    /// A rebuild that was needed shortened this model's effective interval
    /// (´def:monitoring:synchronisation-error´).
    ///
    /// The measurement found drift over the model's threshold and above the
    /// resolution of its own reading, the rebuild removed it, and the cadence
    /// will visit sooner (´dec:posterior:adaptive-cadence´).
    SyncErrorShortened {
        /// Which model was rebuilt.
        model_id: crate::types::ModelId,
        /// The residual that called for the rebuild.
        sync_error: f64,
        /// Dimension-scaled threshold the residual exceeded.
        threshold: f64,
    },
    /// A rebuild that was needed left the model no better, or the
    /// factorisation refused it outright — the alarm
    /// (´dec:posterior:measured-adoption´).
    ///
    /// This is the event the restructured cadence exists to be able to raise.
    /// Under the arrangement it replaces a rebuild ran at the cadence whether
    /// or not one was called for, so a healthy model declined nine rebuilds in
    /// ten and there was no reading a decline could be published as
    /// (´rep:assayer:declined-rebuild-cadence´). A rebuild now runs only where
    /// a measurement found drift worth acting on, so a decline is a model
    /// whose drift is real and whose fresh inverse is worse than the pair it
    /// holds. The pair is kept and the interval goes to its floor.
    RebuildAlarm {
        /// Which model raised it.
        model_id: crate::types::ModelId,
        /// The residual that called for the rebuild.
        sync_error_residual: f64,
        /// The whole reading that residual was taken out of.
        sync_error_before: f64,
        /// The drift the rebuild measured after itself, and what the model
        /// would have taken on had the result been adopted. Equal to the
        /// reading before where the factorisation offered nothing.
        sync_error_after: f64,
        /// The pivot at which the factorisation was refused, absent where it
        /// succeeded and the measurement declined its answer.
        refused_pivot: Option<usize>,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// Lifecycle Health Events
// ═══════════════════════════════════════════════════════════════════════════════

/// Lifecycle health event.
///
/// Emitted on entity competitive-set transitions and operationally
/// significant drift resets.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LifecycleHealthEvent {
    /// A lifecycle removal completed its model marginalisations.
    MarginalisationCompleted {
        /// Aggregate diagnostics across the models changed by the event.
        diagnostics: super::summary::MarginalisationHealth,
    },
    /// Entities entered the competitive set.
    EntitiesEntered {
        /// Dimension where entries occurred.
        dimension_id: DimensionId,
        /// Number of entities that entered.
        count: usize,
    },
    /// Entities exited the competitive set.
    EntitiesExited {
        /// Dimension where exits occurred.
        dimension_id: DimensionId,
        /// Number of entities that exited.
        count: usize,
    },
    /// Drift accumulators were reset (´tab:monitoring:drift-resets´):
    /// per-model on an automatic threshold crossing, or per affected
    /// regime on a large Platt `δ_cal`.
    DriftReset {
        /// The `δ_cal` that triggered the reset.
        delta_cal: f64,
        /// Number of models whose drift state was reset.
        models_reset: usize,
        /// `true` when `reset_all()` was called (CUSUMs AND EWMAs
        /// zeroed).  `false` for a CUSUM-only auto-reset that
        /// preserves the MAR/sign EWMAs.
        full_reset: bool,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// Event Emission
// ═══════════════════════════════════════════════════════════════════════════════

/// Attempts to send a health event, incrementing the drop counter on overflow.
///
/// Uses `try_send` to avoid blocking. If the channel is full, the event
/// is dropped and `dropped_counter` is incremented.
pub fn emit_health_event(event: HealthEvent, event_tx: &Sender<HealthEvent>, dropped_counter: &AtomicU64) {
    match event_tx.try_send(event) {
        Err(TrySendError::Full(_)) => {
            dropped_counter.fetch_add(1, Ordering::Relaxed);
        }
        // A delivered event and a disconnected consumer alike leave the
        // counter where it stands: only a full queue is an overflow.
        Ok(()) | Err(TrySendError::Disconnected(_)) => {}
    }
}

/// Records and emits one drift-reset event.
pub fn emit_drift_reset_event(
    delta_cal: f64,
    models_reset: usize,
    full_reset: bool,
    event_tx: &Sender<HealthEvent>,
    dropped_counter: &AtomicU64,
    reset_counter: &DriftResetCounter,
) {
    reset_counter.increment();
    emit_health_event(
        HealthEvent::Lifecycle(LifecycleHealthEvent::DriftReset {
            delta_cal,
            models_reset,
            full_reset,
        }),
        event_tx,
        dropped_counter,
    );
}

/// Checks if the convergence stage changed and emits a `CompositeStageChanged` event.
///
/// Only emits if `old_stage != new_stage`.
pub fn check_convergence_event(
    old_stage: CompositeConvergenceStage,
    new_stage: CompositeConvergenceStage,
    event_tx: &Sender<HealthEvent>,
    dropped_counter: &AtomicU64,
) {
    if old_stage != new_stage {
        let event = HealthEvent::Convergence(ConvergenceEvent::CompositeStageChanged {
            from: old_stage,
            to: new_stage,
        });
        emit_health_event(event, event_tx, dropped_counter);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Event Channel Configuration
// ═══════════════════════════════════════════════════════════════════════════════

/// Default capacity for the health event channel.
///
/// Events arise from state changes rather than from traffic, so this queue is
/// given the smaller of the two deferral depths: a full queue discards the event
/// and counts it rather than blocking whatever raised it
/// (´dec:concurrency:deferral-depth´). The count, not an argument, is what would
/// justify moving the figure.
///
/// ´const:assayer:health-event-queue-depth´ (´alg:const:count´)
/// ´const:assayer:health-event-queue-depth-count-1000´
pub const DEFAULT_EVENT_CHANNEL_CAPACITY: usize = 1_000;

/// Creates a bounded health event channel with the given capacity.
///
/// Returns `(sender, receiver)` pair. The sender is cloneable.
#[must_use]
pub fn create_event_channel(capacity: usize) -> (Sender<HealthEvent>, crossbeam_channel::Receiver<HealthEvent>) {
    crossbeam_channel::bounded(capacity)
}
