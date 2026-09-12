// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Command and label types for the model-owner thread.
//!
//! This module defines the two channel payloads:
//!
//! - [`ModelOwnerCommand`] on the *command channel* (bounded 64)
//! - [`SequencedLabel`] on the *label channel* (bounded, configurable)
//!
//! Commands preempt labels: the model-owner thread drains all pending
//! commands before processing the next label
//! (´dec:concurrency:channel-preemption´).
//!
//! # Type Index
//!
//! ## Command Types
//!
//! | Type | Variants | What the payload answers to |
//! |------|----------|-----------------------------|
//! | [`ModelOwnerCommand`] | 6 | (´dec:concurrency:channel-preemption´); includes test-only `TestBlock` |
//! | [`ColdRampObservation`] | — | (´alg:standardisation:batch-initialisation´), on its own channel |
//! | [`LifecycleSubmission`] | — | (´dec:construction:compound-batch´) |
//! | [`LifecycleEvent`] | 8 | (´dec:construction:six-methods´), plus competitive cell entry and exit |
//! | [`CheckpointRequest`] | — | (´dec:durability:checkpoint-journal´) |
//! | [`ObservationBarrierRequest`] | — | (´alg:standardisation:batch-initialisation´), the queue a checkpoint does not drain |
//! | [`InitStats`] | — | (´alg:standardisation:sentinel-bootstrap´) (re-export from `feature::bootstrap`) |
//!
//! ## Lifecycle Result Types
//!
//! | Type | What the payload answers to |
//! |------|-----------------------------|
//! | [`LifecycleResult`] | (´dec:construction:compound-batch´) |
//! | [`EventResult`] | (´dec:construction:compound-batch´) |
//! | [`EventOutcome`] | (´dec:numerics:marginalisation-correction´) |
//!
//! ## Label Types
//!
//! | Type | What the payload answers to |
//! |------|-----------------------------|
//! | [`SequencedLabel`] | (´dec:durability:checkpoint-journal´) |
//! | [`LabelData`] | (´tab:host:label-reporting´) |
//! | [`LabelContext`] | (´dec:retention:pending-map´) live, (´dec:retention:journal-subset´) on replay |
//! | [`PendingAssessment`] | (´def:runtime:pending-entry´) (re-export) |
//! | [`PendingContext`] | (´dec:retention:journal-subset´) (re-export) |
//!
//! ## Registration Payloads
//!
//! | Type | Populated in |
//! |------|--------------|
//! | [`SentinelRegistration`] | Layer 2 (data structures, (´schema:registry:sentinel-record´)) |
//! | [`OutcomeAxisRegistration`] | Layer 3 (mutations, (´schema:registry:axis-record´)) |
//! | [`IdentityDimensionRegistration`] | Layer 2 (data structures, (´schema:keyspace:dimension-record´)) |
//! | [`CompetitiveCellId`] | Layer 2 (re-export of `identity::CompetitiveCellId`) |
//!
//! # Cross-References
//!
//! - (´dec:concurrency:single-steward´) — the working copy these commands reach
//! - (´dec:concurrency:channel-preemption´) — commands drain before the next label
//! - (´dec:concurrency:named-thread´) — the thread the payloads are addressed to
//! - (´dec:durability:checkpoint-journal´) — the journal that assigns the sequence number
//! - (´alg:standardisation:sentinel-bootstrap´) — the bootstrap whose completion is a command
//! - (´dec:surface:distinct-action-types´) — what the host did, as against what a derivation proposed
//! - (´tab:host:label-reporting´) — the five fields a label carries
//! - (´dec:construction:compound-batch´) — lifecycle events arrive together, gather into order-free chains and publish once per chain

use std::collections::HashMap;
use std::io;

use crate::error::LifecycleError;
pub use crate::feature::bootstrap::InitStats;
use crate::types::{Action, AssessmentId, DimensionId, EntityKey, OutcomeAxisId, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Model Owner Commands
// ═══════════════════════════════════════════════════════════════════════════════

/// Commands sent to the model-owner thread via the command channel
/// (´dec:concurrency:named-thread´).
///
/// Production commands are drained before any label is processed
/// (´dec:concurrency:channel-preemption´). The `TestBlock` variant is a
/// test-harness barrier and is not part of the runtime contract.
#[derive(Debug)]
#[allow(dead_code)] // L0 structural: some variants constructed in L1+
pub enum ModelOwnerCommand {
    /// A batch of lifecycle events (register/deregister sentinels,
    /// axes, dimensions, cells).
    Lifecycle(LifecycleSubmission),

    /// Request an immediate checkpoint.
    Checkpoint(CheckpointRequest),

    /// Acknowledge once the cold-ramp observation queue has been applied.
    ObservationBarrier(ObservationBarrierRequest),

    /// Shut down the model-owner thread.
    Shutdown,

    /// Host-initiated drift accumulator reset
    /// (´tab:monitoring:drift-resets´): every model's accumulators
    /// reset; the smoothed diagnostics survive.
    ResetDriftAccumulators,

    /// A single sentinel's bootstrap is complete (Layer 2+,
    /// (´alg:standardisation:sentinel-bootstrap´)).
    BootstrapComplete {
        /// The sentinel that finished bootstrapping.
        sentinel_id: SentinelId,
        /// Summary statistics from the bootstrap run.
        stats: InitStats,
    },

    /// Test-harness barrier that keeps the owner thread parked until released.
    TestBlock {
        /// Signals that the owner thread has entered the barrier.
        entered: crossbeam_channel::Sender<()>,
        /// Releases the owner thread when the sender side provides a token.
        release: crossbeam_channel::Receiver<()>,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// Checkpoint
// ═══════════════════════════════════════════════════════════════════════════════

/// Request for an atomic checkpoint write.
///
/// If `completion` is `Some`, a result is sent once the checkpoint
/// finishes (or fails) (´dec:durability:checkpoint-journal´).
#[derive(Debug)]
pub struct CheckpointRequest {
    /// Optional completion channel for acknowledgement.
    pub completion: Option<crossbeam_channel::Sender<Result<(), io::Error>>>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Observation barrier
// ═══════════════════════════════════════════════════════════════════════════════

/// Request for an acknowledgement that the observation queue has been drained
/// and everything taken from it applied.
///
/// The checkpoint request above is not this barrier and cannot be turned into
/// one by ordering. Observations ride their own channel, so where a command
/// sits in the command queue says nothing about where an observation sits in
/// the observation queue; the two are related only by what the steward chooses
/// to drain, and the steward drains the whole command queue before it touches
/// an observation (´dec:concurrency:channel-preemption´). A caller that needs
/// the ramp's arrivals applied therefore has to ask for exactly that, and this
/// is the asking.
///
/// The acknowledgement is unconditional rather than optional, because a
/// barrier nobody waits on is not a barrier: the whole effect of this command
/// is the wait it ends (´alg:standardisation:batch-initialisation´).
#[derive(Debug)]
pub struct ObservationBarrierRequest {
    /// Completion channel signalled once the queue is empty and every
    /// observation taken from it has been applied.
    pub completion: crossbeam_channel::Sender<()>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Init Stats — re-exported from `crate::feature::bootstrap`
// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// Lifecycle Types
// ═══════════════════════════════════════════════════════════════════════════════

/// A batch of lifecycle events with optional completion notification.
///
/// Events are gathered, in order, into maximal chains of order-free operations;
/// each chain applies as one plan and publishes one snapshot, and the
/// completion channel (if present) receives one `LifecycleResult` for the
/// entire batch once the last chain has published
/// (´dec:construction:compound-batch´).
#[derive(Debug)]
pub struct LifecycleSubmission {
    /// Ordered batch of lifecycle events.
    pub events: Vec<LifecycleEvent>,
    /// Optional completion channel for the batch result.
    pub completion: Option<crossbeam_channel::Sender<LifecycleResult>>,
}

/// Individual lifecycle event.
///
/// Six of the ten variants are the registration and deregistration
/// operations the lifecycle surface exposes
/// (´dec:construction:six-methods´); two carry competitive cell entry and
/// exit, and two are the hibernating readings of the Sentinel and axis
/// removals (´alg:registry:hibernation´).
///
/// The hibernating removals are separate variants rather than a flag on the
/// destructive ones because they are separate decisions with separate
/// consequences, and the surface says which was taken. Deregistration is
/// destructive by default and stays so
/// (´dec:construction:destructive-deregistration´): what the hibernating
/// variant adds is a copy of the entity's own block kept against its
/// identifier, and the marginalisation it performs on the surviving models is
/// the same marginalisation the destructive variant performs.
#[derive(Debug)]
pub enum LifecycleEvent {
    /// Register a new sentinel.
    RegisterSentinel(SentinelRegistration),
    /// Remove a sentinel (may trigger marginalisation).
    DeregisterSentinel(SentinelId),
    /// Remove a sentinel, keeping its own parameter block against its
    /// identifier for a later registration (´alg:registry:hibernation´).
    HibernateSentinel(SentinelId),
    /// Register a new outcome axis.
    RegisterOutcomeAxis(OutcomeAxisRegistration),
    /// Remove an outcome axis.
    DeregisterOutcomeAxis(OutcomeAxisId),
    /// Remove an outcome axis, keeping its own parameter block against its
    /// identifier under the same protocol Sentinels hibernate under
    /// (´rem:registry:axis-hibernation´).
    HibernateOutcomeAxis(OutcomeAxisId),
    /// Register a new identity dimension.
    ///
    /// Carries only the field the model owner needs (dimension ID).
    /// The full `IdentityDimensionRegistration`
    /// (including `encode`) is consumed by `api/lifecycle.rs` and
    /// stored in `IdentityDimensionInfra`; it is never forwarded to
    /// the model-owner thread.
    RegisterIdentityDimension {
        /// The identity dimension being registered.
        id: DimensionId,
    },
    /// Remove an identity dimension.
    DeregisterIdentityDimension(DimensionId),
    /// An entity entered a competitive cell.
    CompetitiveCellEntry {
        /// The identity dimension.
        dimension: DimensionId,
        /// The cell entered.
        cell: CompetitiveCellId,
    },
    /// An entity exited a competitive cell.
    CompetitiveCellExit {
        /// The identity dimension.
        dimension: DimensionId,
        /// The cell exited.
        cell: CompetitiveCellId,
    },
}

// ─── Lifecycle result types ──────────────────────────────────────────────────

/// Result of processing a lifecycle submission.
pub type LifecycleResult = Result<Vec<EventResult>, LifecycleError>;

/// Per-event result within a lifecycle batch.
#[derive(Clone, Debug)]
#[allow(dead_code)] // L0: fields read by model owner in L1+
pub struct EventResult {
    /// Index of the event within the submission's `events` Vec.
    pub event_index: usize,
    /// Which of the submission's order-free chains this event was applied in.
    ///
    /// Results sharing a chain index were applied by one plan against one entry
    /// layout and published in one snapshot, so the diagnostics they carry are
    /// one computation reported to each of them rather than several
    /// (´dec:construction:compound-batch´). A consumer that must count
    /// applications rather than operations counts distinct chain indices.
    pub chain_index: usize,
    /// The outcome for this event.
    pub outcome: EventOutcome,
    /// Aggregate model diagnostics for the chain application this event was
    /// part of, where that application removed dimensions.
    pub marginalisation: Option<crate::health::MarginalisationHealth>,
}

/// Outcome of a single lifecycle event.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventOutcome {
    /// Event processed successfully.
    Success,
    /// Event succeeded but the conditioning guard declined the
    /// correction and fell back to the uncorrected block — information
    /// loss, not an error (´dec:numerics:marginalisation-correction´).
    SchurFallback,
}

// ─── Registration types ──────────────────────────────────────────────────────

/// Sentinel registration data.
#[derive(Clone, Debug)]
pub struct SentinelRegistration {
    /// The sentinel being registered.
    pub id: SentinelId,
    /// Human-readable name (must be unique across registered Sentinels).
    pub name: String,
}

/// Outcome axis registration data.
#[derive(Clone, Debug)]
pub struct OutcomeAxisRegistration {
    /// The outcome axis being registered.
    pub id: OutcomeAxisId,
    /// Human-readable name.
    pub name: String,
    /// What the axis measures, in the host's own words
    /// (´schema:registry:axis-record´).
    pub description: String,
    /// Which labels this axis receives.
    pub eligibility: crate::types::OutcomeEligibility,
    /// Initial κ value for this axis.
    pub initial_kappa: f64,
    /// Decay rate γ (per label).
    pub gamma: f64,
    /// Whether spatial features are enabled.
    pub spatial_features: crate::types::SpatialFeaturePolicy,
}

/// Identity dimension registration data.
#[derive(Clone, Debug)]
pub struct IdentityDimensionRegistration {
    /// The identity dimension being registered.
    pub id: DimensionId,
    /// Human-readable name.
    pub name: String,
    /// What this key space is, in the host's own words.
    pub description: String,
    /// What coordinates mean, how they are encoded, and whether shared
    /// prefixes assert shared context.
    pub coordinate_semantics: String,
    /// Domain bit-width (typically 128).
    pub domain_bits: u8,
    /// Maximum competitive cell depth.
    pub depth_cutoff: u8,
    /// Resource budget for this dimension.
    pub budget: crate::types::IdentityBudget,
    /// Entity-to-coordinate encoding function.
    pub encode: fn(&EntityKey) -> u128,
}

/// Re-export of [`crate::identity::CompetitiveCellId`].
pub use crate::identity::CompetitiveCellId;

// ═══════════════════════════════════════════════════════════════════════════════
// Label Types
// ═══════════════════════════════════════════════════════════════════════════════

/// A label with optional sequence number, sent on the label channel.
///
/// The `seq` field is assigned by the journal writer (if persistence is
/// enabled) (´dec:durability:checkpoint-journal´); `None` during
/// cold-start or when journaling is disabled.
#[derive(Debug)]
pub struct SequencedLabel {
    /// Journal sequence number (assigned by the persistence layer).
    pub seq: Option<u64>,
    /// The label data from the caller.
    pub label: LabelData,
    /// Assessment context (full or replay).
    pub context: LabelContext,
    /// When this label reached the public label boundary, on the injected
    /// monotonic clock (´def:monitoring:feedback-latency´).
    ///
    /// The label boundary and the model owner are different threads with a
    /// channel between them, so the wait on that channel is measurable only
    /// if the arrival travels with the label. It rides here rather than on
    /// [`LabelData`] because it is not something the host reported: it is
    /// what this process observed about the host's report.
    ///
    /// `None` for a label replayed from the journal, whose original arrival
    /// was an instant in a monotonic domain the restart ended.
    pub arrived_at: Option<std::time::Instant>,
}

/// Public label input from the caller.
///
/// Reports the outcome of a previously assessed request. The host
/// provides the outcome (valence), what it did (`action_taken`), and
/// any secondary outcome axis values.
///
/// # Valence sign convention (´conv:valence:sign´)
///
/// **Positive valence = adverse outcome** (fraud, violation, harm).
/// **Non-positive valence = benign or neutral outcome.**
///
/// A host that provides positive valence for benign outcomes will
/// obtain **systematically inverted risk estimates**. The convention
/// must be established before the first label and remain consistent
/// throughout the deployment lifetime.
///
/// # Cross-References
///
/// - (´tab:host:label-reporting´) — the five fields and what each carries
/// - (´conv:valence:sign´) — the valence sign convention
/// - (´tab:eligibility:training´) — which labels train which model
/// - (´cav:limitation:investigation-pipeline´) — `ground_truth` trusted unconditionally
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LabelData {
    /// Links this label to a pending assessment.
    ///
    /// Must match an [`AssessmentId`] from a previous `assess()` call
    /// whose pending buffer entry has not expired or been evicted.
    /// If not found, `label()` returns
    /// [`LabelError::AssessmentNotFound`](crate::error::LabelError::AssessmentNotFound).
    pub assessment_id: AssessmentId,

    /// What the host actually did with this request.
    ///
    /// Need not match any action from the host's derivation policy: the
    /// host is free to take any action, including actions not indicated
    /// by derivation (´tab:host:label-reporting´). The system learns from
    /// what the host did, not from what the system determined.
    ///
    /// Affects training eligibility (´tab:eligibility:training´): `Allow` and `Slow`
    /// are eligible with observed outcomes; `Challenge` and `Block` require
    /// independently verified ground truth for sister/anchor training.
    pub action_taken: Action,

    /// Outcome magnitude. **Positive = adverse.**
    ///
    /// The risk model uses only the sign (´def:risk:target´):
    ///
    /// - `valence > 0` → risk target `+1`
    /// - `valence ≤ 0` → risk target `-1`
    ///
    /// The magnitude enters the system through features (entity and
    /// Ledger valence EWMAs and compressed valence) but **not** through
    /// the risk training target.
    ///
    /// NaN/Inf values are sanitised to `0.0` at the API boundary
    /// (treated as benign).
    pub valence: f64,

    /// Per-axis outcome values for registered outcome axes.
    ///
    /// Axes not present in this map are not updated — their existing
    /// Ledger and identity cell features are preserved
    /// (´tab:host:label-reporting´).
    /// Axis IDs not currently registered are silently ignored by the
    /// model owner.
    ///
    /// NaN/Inf values are removed at the API boundary before
    /// enqueueing.
    pub outcomes: HashMap<OutcomeAxisId, f64>,

    /// Host's assertion that the outcome was independently
    /// investigated.
    ///
    /// When `true`, the label is treated as unconfounded regardless of
    /// `action_taken` — eligible for sister, anchor, and outcome axis
    /// model training (´tab:eligibility:training´). The Assayer trusts this
    /// flag unconditionally; it has no mechanism to verify it
    /// (´cav:limitation:investigation-pipeline´). A compromised investigation pipeline that
    /// systematically mislabels outcomes will corrupt the sister model.
    pub ground_truth: bool,
}

/// Context carried with a label through the channel.
///
/// `Full` carries the pending assessment saved at `assess()` time.
/// `Replay` carries a stripped version reconstructed from the journal.
#[derive(Debug)]
#[allow(dead_code)] // L0: variants constructed in tests/L1+
pub enum LabelContext {
    /// Full context from a live assessment.
    Full(Box<PendingAssessment>),
    /// Self-contained context from journal replay.
    Replay(Box<PendingContext>),
}

/// Re-export of [`crate::pending::PendingAssessment`].
///
/// Full in-memory pending assessment entry saved at `assess()` time.
/// Carries sentinel extractions, identity coordinates, signal features,
/// risk basis, degradation context, etc.
///
/// # Cross-References
///
/// - (´def:runtime:pending-entry´) — what a pending entry carries
pub use crate::pending::PendingAssessment;
/// Re-export of [`crate::pending::PendingContext`].
///
/// Journal-stripped version of `PendingAssessment` for persistence and
/// replay. Omits `risk_basis` (recomputed from model) and `degradation`.
///
/// # Cross-References
///
/// - (´dec:retention:journal-subset´) — what the journal keeps and what it omits
pub use crate::pending::PendingContext;

// ═══════════════════════════════════════════════════════════════════════════════
// Cold ramp observations
// ═══════════════════════════════════════════════════════════════════════════════

/// One raw assessment-time vector offered to the cold prior-mass ramp
/// step 2 of (´alg:standardisation:batch-initialisation´).
///
/// Carried on its own channel rather than as a control command, because it is
/// neither. A control command is rare, is issued by the host, and must not be
/// dropped; an observation is one per request for the length of the ramp, is
/// issued by the assessment path, and is best-effort by specification. Sharing
/// one bounded queue between them would let the ramp's own traffic crowd out
/// the registrations and shutdowns it has no business delaying — the volumes
/// differ by the horizon itself.
///
/// The channel is bounded at that horizon, `N_init`: a ramp never has more
/// observations outstanding than it has mass left to retire, so the bound is
/// the mechanism's own figure and not a second one to tune.
#[derive(Clone, Debug)]
pub struct ColdRampObservation {
    /// The raw, unstandardised feature vector.
    pub phi_raw: Vec<f64>,
    /// The layout it was assembled under. The steward refuses it if the
    /// working copy has since renumbered its positions
    /// (´req:standardisation:lifecycle-entries´).
    pub layout_generation: u64,
}
