// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! An online-learning dynamically resizing linear risk estimation model.
//!
//! The Assayer estimates risk from the measurement reports Sentinels produce,
//! and learns online from the labelled outcomes the host reports back. Its
//! dimension resizes as Sentinels register and deregister — a registration
//! extends every model but the fixed-dimension anchor, a deregistration
//! marginalises the same set — and what it produces is a risk estimate: the
//! Assayer measures, and the decision on the estimate is the host's.
//!
//! # Quick Start
//!
//! ```ignore
//! use torrust_assayer::{
//!     Assayer,
//!     AssayerConfig,
//!     ChallengeEstimate,
//!     ChannelPolicy,
//!     RequestContext,
//!     ResonanceConfig,
//!     derive_landscape,
//!     render_resonances,
//! };
//!
//! // Build. The command channel's capacity is host-set with no
//! // default: a deployment declares it or construction refuses.
//! let mut config = AssayerConfig::default();
//! config.infrastructure.command_channel_capacity = Some(64);
//! let builder = Assayer::builder(config).signal_schema(&[]);
//! let assayer = builder.build()?;
//!
//! // Assess
//! let assessments = assayer.assess(&[RequestContext::new(entity_key)]);
//! let assessment = &assessments[0];
//!
//! // Derive (reusing one assessment against two policies), then
//! // optionally render each landscape for display
//! // (´dec:landscape:rendering-optional´).
//! let login_policy = ChannelPolicy::default();
//! let api_policy = ChannelPolicy::default();
//! let challenge = ChallengeEstimate::default();
//! let login = derive_landscape(assessment, &login_policy, challenge);
//! let _api = derive_landscape(assessment, &api_policy, challenge);
//! let _login_profile = render_resonances(&login, &assessment.risk, &ResonanceConfig::default());
//!
//! // Label
//! assayer.label(label_data)?;
//! ```
//!
//! # Architecture Overview
//!
//! The Assayer maintains multiple Bayesian linear models that evolve over time:
//!
//! - **Operational model (V)** — The primary model used for live decisions
//! - **Sister model** — A slower-decaying copy for drift detection
//! - **Anchor model** — A fixed-dimension reference for baseline comparison
//! - **Outcome-axis models** — Per-axis models for different signal types
//!
//! All model updates occur on a dedicated model-owner thread, ensuring
//! serialised writes. Readers access model state through lock-free snapshots
//! via `arc-swap`.
//!
//! # Public API
//!
//! | Method | Signature | Purpose |
//! |--------|-----------|----------|
//! | [`Assayer::assess()`] | `&[RequestContext] → Vec<RiskAssessment>` | Core risk assessment |
//! | [`Assayer::request_labels()`] | `LabelBudget + LabelGuidanceParams → LabelRequests` | Core label guidance |
//! | [`derive_landscape()`] | `RiskAssessment + ChannelPolicy + ChallengeEstimate → DecisionLandscape` | Pure derivation |
//! | [`render_resonances()`] | `DecisionLandscape + RiskBasis + ResonanceConfig → ResonanceProfile` | Optional rendering layer |
//! | [`Assayer::label()`] | `LabelData → Result<LabelAck, LabelError>` | Submit ground-truth outcome |
//! | [`Assayer::receive_sentinel_report()`] | `(SentinelId, BatchReport<u128>) → Result<ReportAck, ReportError>` | Ingest Sentinel measurement |
//!
//! # Cross-References
//!
//! - Information crosses the boundary in one direction (´dec:ownership:feed-forward´)
//! - Learned state reaches readers only as a swapped snapshot (´dec:concurrency:snapshot-swap´)
//! - The model is dense and dynamically dimensioned (´dec:substrate:dense-dynamic´)
//! - One steward owns the working copy (´dec:concurrency:single-steward´)
//! - The steward is a dedicated named thread (´dec:concurrency:named-thread´)
//! - A periodic checkpoint and a self-contained journal (´dec:durability:checkpoint-journal´)
//! - Structural violations are hard; data-quality problems degrade (´dec:degradation:error-partition´)
//! - The assessment call takes a batch and cannot fail (´dec:surface:batch-infallible´)
//! - Label submission is asynchronous (´dec:surface:async-label´)
//! - Reception touches no model, graph, cache, or calibration state (´dec:surface:reception-isolation´)

#![warn(
    missing_docs,
    clippy::missing_docs_in_private_items,
    clippy::missing_errors_doc,
    clippy::missing_panics_doc
)]

// TODO ´todo:code:justify-or-retire-lint-suppressions´: justify or retire lint suppressions
// (´sec:assayer:clippy-exception-discipline´).

// TODO ´todo:code:retire-stale-layer-scaffolding´: retire stale layer scaffolding
// (´dec:assayer:spec-authoritative´).

// ═══════════════════════════════════════════════════════════════════════════════
// Foundation Modules (Layer 1–4) — All private except error and types
// ═══════════════════════════════════════════════════════════════════════════════

mod api;
mod assessment;
mod config;
pub mod error;
mod extraction;
mod feature;
mod guidance;
mod health;
mod identity;
mod ledger;
mod linalg;
pub mod metrics;
mod model;
mod numerics;
mod owner;
mod pending;
mod persistence;
mod report;
mod resonance;
mod risk;
#[cfg(feature = "serde")]
mod serde_codec;
mod signal;
mod snapshot;
pub mod types;

// Integration-test harness (see `tests/README.md`). Public only under
// the `test-support` feature the crate's own test builds enable, so both
// crate tests (`src/tests/`) and integration tests (`tests/`) share the
// same helpers while a host build seals the module: opacity enforced
// structurally rather than by review (´dec:surface:guidance-opacity´).
// The module itself always compiles — production reads its clock through
// `testing::Clock`.
#[cfg(any(test, feature = "test-support"))]
#[doc(hidden)]
pub mod testing;
#[cfg(not(any(test, feature = "test-support")))]
pub(crate) mod testing;

#[cfg(test)]
mod tests;

// ═══════════════════════════════════════════════════════════════════════════════
// Public Re-exports — Assessment and Derivation Output Types
// ═══════════════════════════════════════════════════════════════════════════════

pub use api::{AssayerBuilder, LabelAck, PreSeedEntry, PreSeedResult, ReportAck};
pub use assessment::{
    BatchInitObservation, DerivedReckoning, HealthSnapshot, OutcomePrediction, RequestContext, RiskAssessment, RiskBasis,
};
pub use config::types::{AssayerConfig, EligibilityPolicy, MonitoringConfig};
pub use extraction::SentinelAlarmSummary;
// The cold ramp's phase rides on the compact health every assessment carries
// and on the full report (´tab:monitoring:standardisation-transition´), so a
// host has to be able to name it and match on it. A public field whose type
// cannot be named is readable and not usable.
pub use feature::standardisation::StandardisationPhase;
pub use guidance::{LabelBudget, LabelCandidate, LabelCategory, LabelGuidanceParams, LabelRequests};
// The rebuild verdict rides the detailed precision health, and a host reading
// why a model declined its last rebuild has to be able to match on it for the
// same reason the ramp's phase is exported above: a public field whose type
// cannot be named is readable and not usable
// (´dec:posterior:measured-adoption´).
pub use health::{
    BlendStatistics, CompositeConvergenceStage, DegradationContext, DiscriminationMetrics, HealthSummary,
    PublishedRebuildVerdict, SystemHealthReport,
};
pub use owner::commands::{IdentityDimensionRegistration, LabelData, OutcomeAxisRegistration, SentinelRegistration};
pub use resonance::ambiguity::ProfileAmbiguity;
// `validate_channel_policy` is the host's half of the load-time check
// (´dec:derivation:ordered-actions´): exported beside the policy type it
// validates so a host-side channel registry can refuse a malformed
// declaration before any derivation runs.
pub use resonance::channel::{ChannelPolicy, RewardParameters, validate_channel_policy};
pub use resonance::derivation::{
    ChallengeEstimate, ResonanceConfig, ResonanceProfile, profile_ambiguity, render_resonances, tag_probabilities,
};
pub use resonance::landscape::{
    ActionCrossover, ActionRegime, DecisionFragility, DecisionLandscape, derive_landscape, fragility, optimal_action,
};
pub use resonance::tags::{Tag, TagResonance};
pub use risk::challenge::{
    ChallengeEffectivenessProvider, ChallengeEffectivenessState, ChallengeEffectivenessTracker, ChallengeEvidenceError,
    ChallengeHealthDetail, ChallengeHealthReport, ChallengeOverride, ChallengePosterior,
};
pub use signal::{Persistence, SignalDeclaration, SignalShape, SignalValue};
pub use types::{ChallengePosteriorInput, InvalidChallengePosterior};

// ═══════════════════════════════════════════════════════════════════════════════
// Integration-Test Support — behind the `test-support` feature
// ═══════════════════════════════════════════════════════════════════════════════

/// Numerics functions needed by integration tests.
///
/// Sealed behind `test-support` (´dec:surface:guidance-opacity´): the
/// integration tests enable the feature and a host does not.
#[cfg(any(test, feature = "test-support"))]
#[doc(hidden)]
pub mod numerics_export {
    pub use crate::numerics::{
        beta_quantile, decay_count, decay_factor, decay_factor_elapsed, decay_factor_since, half_life_days, half_life_hours,
        hours_to_decay_target, label_decay_factor, ln_gamma, normal_cdf, reg_inc_beta, regime_transition, stable_atanh,
        stable_logit, stable_sigmoid,
    };
}

/// Signal infrastructure needed by integration tests.
///
/// Sealed behind `test-support` (´dec:surface:guidance-opacity´): the
/// integration tests enable the feature and a host does not.
#[cfg(any(test, feature = "test-support"))]
#[doc(hidden)]
pub mod signal_export {
    pub use crate::signal::{SignalCache, SignalCacheHealth, SignalSchemaIndex, encode_signal};
}

// Re-export commonly used types at crate root
// ═══════════════════════════════════════════════════════════════════════════════
// Runtime
// ═══════════════════════════════════════════════════════════════════════════════
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU8, AtomicU64};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;
use std::time::Duration;

/// Assessment shared-state trait needed by integration tests.
///
/// Sealed behind `test-support` (´dec:surface:guidance-opacity´): the
/// integration tests enable the feature and a host does not.
#[cfg(any(test, feature = "test-support"))]
#[doc(hidden)]
pub use assessment::AssessmentSharedState;
use crossbeam_channel::{Receiver, Sender};
use dashmap::DashMap;
pub use error::{
    BuildError, ChannelError, LabelError, LabelPathStop, LifecycleError, ReportError, check_diagonal_nan, check_vec_nan,
    sanitise_f64,
};
use owner::commands::{ModelOwnerCommand, SequencedLabel};
use owner::thread::ModelOwner;
use snapshot::shared::SharedState;
use snapshot::working::WorkingCopy;
use tracing::{info, warn};
pub use types::{
    Action, AssessmentId, CellInterval, ChallengeResult, ChannelId, DimensionId, EntityKey, LedgerKey, MAX_DECAY_HOURS, ModelId,
    OutcomeAxisId, PersistentTimestamp, SentinelId, duration_to_hours, dyadic_ancestor_hi, dyadic_ancestor_lo, is_dyadic,
};

// ─── Restored engine state ───────────────────────────────────────────────────

/// State a checkpoint restore recovered beyond the working copy, carried
/// from `build_working_copy` into `build_inner` so each piece reaches the
/// infrastructure that owns it (´dec:durability:checkpoint-journal´).
#[cfg(feature = "serde")]
struct RestoredEngineState {
    /// Concordance tracker state.
    concordance: health::ConcordanceCheckpointState,
    /// Owner-held state (calibration buffer, trackers, counters).
    owner: persistence::checkpoint::OwnerCheckpointState,
    /// Per-Sentinel ledger state, attached to the outcome ledger.
    ledgers: Vec<(SentinelId, persistence::checkpoint::SentinelLedgerPayload)>,
    /// Per-dimension identity state.
    // TODO ´todo:code:reattach-restored-identity-state-when´: reattach restored identity state when
    // the host re-registers each dimension — the encode closure cannot
    // cross a checkpoint, so the graph and cell payloads recovered here
    // are dropped until a re-registration path can seed them. The graphs
    // belong to a dedicated owner (´dec:memory:graph-owner´).
    #[allow(dead_code)]
    identity: Vec<(DimensionId, persistence::checkpoint::IdentityDimensionPayload)>,
}

/// The durable inputs to construction, gathered into the one parameter
/// they have always been: the entries a restore left to replay, the
/// structural metadata the model owner is given, and the state
/// recovered beyond the working copy. The working-copy build produces
/// the three together and construction is their only consumer
/// (´dec:durability:checkpoint-journal´).
#[cfg(feature = "serde")]
struct DurableStart {
    /// Journal entries awaiting replay.
    replay_entries: Vec<persistence::journal::JournalEntry>,
    /// Structural metadata handed to the model owner.
    structural_metadata: persistence::checkpoint::StructuralMetadata,
    /// Checkpoint state recovered beyond the working copy, absent on a
    /// cold start.
    restored: Option<RestoredEngineState>,
}

// ─── Thread Handles ──────────────────────────────────────────────────────────

/// Handles for threads owned by the `Assayer`.
///
/// The structural layer's steward is a dedicated named thread
/// (´dec:concurrency:named-thread´), beside an optional checkpoint
/// scheduler; the identity maintenance thread serves the dedicated owner
/// of every identity graph (´dec:memory:graph-owner´).
struct AssayerThreads {
    /// Model-owner thread handle.
    model_owner: Option<JoinHandle<()>>,
    /// Checkpoint scheduler thread handle (´dec:durability:checkpoint-journal´).
    checkpoint: Option<JoinHandle<()>>,
    /// Ledger collection scheduler thread handle
    /// (´dec:memory:periodic-sweep´).
    ledger_gc: Option<JoinHandle<()>>,
    /// Identity maintenance thread handle.
    identity_maintenance: Option<JoinHandle<()>>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Assayer
// ═══════════════════════════════════════════════════════════════════════════════

/// An online-learning dynamically resizing linear risk estimation model.
///
/// The `Assayer` is the top-level entry point. It owns the model-owner
/// thread, communication channels, and the shared snapshot state.
///
/// # Construction
///
/// ```ignore
/// use torrust_assayer::{Assayer, AssayerConfig};
///
/// let mut config = AssayerConfig::default();
/// // Host-set with no default: declare the command channel's capacity.
/// config.infrastructure.command_channel_capacity = Some(64);
/// let builder = Assayer::builder(config).signal_schema(&[]);
/// let assayer = builder.build()?;
/// ```
///
/// # Public API
///
/// - [`Assayer::assess()`] — Core risk assessment: `&[RequestContext] → Vec<RiskAssessment>`
/// - [`Assayer::request_labels()`] — Core label guidance over live pending assessments
/// - [`derive_landscape()`] — Pure derivation from an explicit `RiskAssessment`
/// - [`Assayer::label()`] — Submit ground-truth outcome: `LabelData → Result<LabelAck, LabelError>`
/// - [`Assayer::receive_sentinel_report()`] — Ingest Sentinel report: `(SentinelId, BatchReport<u128>) → Result<ReportAck, ReportError>`
///
/// # Shutdown
///
/// Dropping the `Assayer` sends `Shutdown` to the model-owner thread
/// and joins it; the steward is a dedicated named thread
/// (´dec:concurrency:named-thread´).
pub struct Assayer {
    /// Shared state (published snapshot, accessed by readers).
    shared: Arc<SharedState>,
    /// The engine's source of time, in both domains.
    ///
    /// [`SystemClock`](testing::SystemClock) in production; tests
    /// inject a [`VirtualClock`](testing::VirtualClock) so staleness
    /// and decay intervals stop depending on how busy the machine is.
    clock: Arc<dyn testing::Clock>,
    /// Configuration snapshot.
    config: AssayerConfig,
    /// Sender for labels (bounded channel to model owner).
    label_tx: Sender<SequencedLabel>,
    /// Sender for commands (bounded channel to model owner).
    command_tx: Sender<ModelOwnerCommand>,
    /// Liveness signal for the model-owner thread.
    ///
    /// A bounded `()` channel whose sole Sender lives on the
    /// model-owner thread's stack. When that thread exits for any
    /// reason — normal `Shutdown`, unwinding panic, or abort — the
    /// Sender drops and this Receiver observes `Disconnected`. Used
    /// by `send_lifecycle_sync` to surface owner-thread death as
    /// [`LifecycleError::ModelOwnerShutdown`] instead of hanging on
    /// the per-submission completion channel forever.
    pub(crate) owner_alive_rx: Receiver<()>,
    /// Thread handles.
    threads: AssayerThreads,
    /// Monotonically increasing assessment ID counter.
    next_assessment_id: AtomicU64,
    /// Checkpoint scheduler control channel sender
    /// (´dec:durability:checkpoint-journal´).
    /// Dropping this signals the scheduler to shut down.
    checkpoint_control_tx: Option<Sender<()>>,
    /// Ledger collection scheduler control channel sender.
    /// Dropping this signals the scheduler to shut down.
    ledger_gc_control_tx: Option<Sender<()>>,

    // ─── Layer 2+ fields, the representation layer (´sec:representation:scope´) ───
    /// Per-Sentinel slot map (concurrent access via `DashMap`).
    sentinel_slots: DashMap<SentinelId, report::SentinelSlot>,
    /// Pending assessment buffer.
    pending_buffer: pending::PendingBuffer,
    /// Signal cache for entity-persistent features.
    signal_cache: Arc<signal::SignalCache>,
    /// Outcome ledger (per-Sentinel hierarchical outcome tracking).
    outcome_ledger: Arc<ledger::OutcomeLedger>,
    /// Report-ingestion settings, derived from the configuration once at
    /// construction. The absence threshold is eight bits wide where the
    /// deletion predicate reads it (´alg:ledger:entry-deletion´), so the
    /// configured count is converted here rather than narrowed at every
    /// report (´dec:construction:eager-validation´).
    ingestion: report::IngestionConfig,
    /// Identity dimension infrastructure (per-dimension shared state).
    identity_dimensions: Arc<RwLock<HashMap<DimensionId, Arc<identity::IdentityDimensionInfra>>>>,
    /// Identity maintenance command sender.
    identity_command_tx: Option<Sender<identity::MaintenanceCommand>>,

    // ─── Layer 5 fields, contracts and boundaries (´sec:contracts:scope´) ───
    /// Concordance tracker (rolling-window threshold estimation).
    concordance: Arc<health::ConcordanceTracker>,
    /// Current composite convergence stage (atomic for lock-free reads).
    convergence_stage: AtomicU8,
    /// Total assessments processed.
    total_assessments: AtomicU64,
    /// Blend statistics tracker, read by the cost-tiered queries
    /// (´dec:health:tiered-queries´).
    blend_stats: health::BlendStatisticsTracker,
    /// Assessment degradation counters.
    degradation_counters: health::AssessmentDegradationCounters,
    /// Valences repaired at the label boundary
    /// (´dec:surface:sanitise-not-reject´).
    label_valences_sanitised: AtomicU64,
    /// Per-axis outcomes dropped at the label boundary
    /// (´dec:surface:sanitise-not-reject´).
    label_outcomes_dropped: AtomicU64,
    /// Labels lost when the bounded label queue refuses a submission.
    label_queue_drops: AtomicU64,
    /// Health event receiver (´dec:health:bounded-events´).
    health_event_rx: crossbeam_channel::Receiver<health::HealthEvent>,
    /// Counter of health events dropped due to channel overflow, which is
    /// the counter the bounded channel drops with (´dec:health:bounded-events´).
    health_events_dropped: Arc<AtomicU64>,
    /// Counter incremented whenever a drift-reset health event is emitted.
    drift_reset_counter: Arc<health::DriftResetCounter>,
    /// Registered outcome axes (API-level TOCTOU guard).
    ///
    /// Provides atomic check-and-insert for `register_outcome_axis()`
    /// so that concurrent callers cannot both pass the duplicate
    /// check before the model owner publishes.
    registered_axes: Mutex<HashSet<OutcomeAxisId>>,
    /// Journal writer for crash-safe label replay
    /// (´dec:durability:checkpoint-journal´).
    /// `None` when persistence is disabled.
    #[cfg(feature = "serde")]
    journal: Option<Arc<Mutex<persistence::journal::JournalWriter>>>,
    /// Cold-ramp observations, on their own bounded queue so the ramp's
    /// traffic never crowds out a control command
    /// (´alg:standardisation:batch-initialisation´).
    observation_tx: crossbeam_channel::Sender<owner::commands::ColdRampObservation>,
}

impl std::fmt::Debug for Assayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guard = self.shared.published.load();
        f.debug_struct("Assayer")
            .field("instance_id", &self.config.instance_id)
            .field("published_version", &guard.version)
            .field(
                "next_assessment_id",
                &self.next_assessment_id.load(std::sync::atomic::Ordering::Relaxed),
            )
            .finish_non_exhaustive()
    }
}

#[allow(dead_code)] // L0: called by tests and L1+ public API
impl Assayer {
    /// Builds and starts an `Assayer` with the given configuration.
    ///
    /// Creates a cold-start working copy (or restores from checkpoint),
    /// publishes the initial snapshot, spawns the model-owner thread,
    /// and optionally spawns the checkpoint scheduler.
    ///
    /// Uses default (empty) signal schema, channels, and interaction
    /// templates. For the validated builder path, use
    /// [`build_from_builder`](Self::build_from_builder).
    ///
    /// # Errors
    ///
    /// Returns [`BuildError::PersistenceSetupFailed`] if the journal cannot be
    /// opened, and [`BuildError::CheckpointNotPositiveDefinite`] if a
    /// checkpoint carries a precision matrix the model may not hold.
    ///
    /// # Panics
    ///
    /// Panics if the model-owner thread cannot be spawned.
    pub(crate) fn build(config: AssayerConfig) -> Result<Self, BuildError> {
        let clock: Arc<dyn testing::Clock> = Arc::new(testing::SystemClock);

        #[cfg(feature = "serde")]
        let (working, replay_entries, restored) = Self::build_working_copy(&config, clock.now())?;
        #[cfg(not(feature = "serde"))]
        let (working, ()) = Self::build_working_copy(&config);

        let signal_schema = Arc::new(signal::SignalSchemaIndex::from_declarations(&[]).expect("empty schema"));

        #[cfg(feature = "serde")]
        return Self::build_inner(
            config,
            working,
            signal_schema,
            DurableStart {
                replay_entries,
                structural_metadata: persistence::checkpoint::StructuralMetadata::default(),
                restored,
            },
            clock,
        );
        #[cfg(not(feature = "serde"))]
        Self::build_inner(config, working, signal_schema, clock)
    }

    /// Core construction: creates infrastructure, spawns threads,
    /// assembles the `Assayer`.
    ///
    /// Both [`build`](Self::build) and [`build_from_builder`](Self::build_from_builder)
    /// delegate here after preparing the `WorkingCopy` and signal schema.
    ///
    /// # Errors
    ///
    /// Returns [`BuildError::PersistenceSetupFailed`] if the journal cannot be
    /// opened. Every fallible step runs before any thread is spawned.
    ///
    /// # Panics
    ///
    /// Panics if threads cannot be spawned.
    #[allow(clippy::too_many_lines)] // Justified: 9-phase build is inherently sequential
    fn build_inner(
        config: AssayerConfig,
        working: WorkingCopy,
        signal_schema: Arc<signal::SignalSchemaIndex>,
        #[cfg(feature = "serde")] durable: DurableStart,
        clock: Arc<dyn testing::Clock>,
    ) -> Result<Self, BuildError> {
        #[cfg(feature = "serde")]
        let DurableStart {
            replay_entries,
            structural_metadata,
            restored,
        } = durable;

        let initial_snapshot = working.to_snapshot_at(1, clock.now_monotonic());
        #[cfg(feature = "serde")]
        let restored_seq = working.last_processed_label_seq;
        let shared = Arc::new(SharedState::new(initial_snapshot));

        // Each restored piece reaches the infrastructure that owns it
        // below; the identity payloads are the one piece that cannot
        // reattach here (see `RestoredEngineState::identity`).
        #[cfg(feature = "serde")]
        let (restored_concordance_state, restored_owner_state, restored_ledger_state) = match restored {
            Some(state) => (Some(state.concordance), Some(state.owner), Some(state.ledgers)),
            None => (None, None, None),
        };

        // Opened before any thread is spawned: the journal is the fallible
        // half of construction (´dec:construction:eager-validation´), and a
        // failure after the spawn would unwind past a running model owner.
        #[cfg(feature = "serde")]
        let journal = match config.persistence.as_ref() {
            Some(p) => {
                // `open_after`, not `open`: the checkpoint's mark is a lower
                // bound on the next sequence, and numbering below it puts the
                // run's whole journal under the replay filter
                // (´cor:durability:replay-exactness´).
                let writer = persistence::journal::JournalWriter::open_after(&p.journal_path(), restored_seq)
                    .map_err(BuildError::PersistenceSetupFailed)?;
                Some(Arc::new(Mutex::new(writer)))
            }
            None => None,
        };

        // Command channel: host-set capacity with no default — a config
        // that never declared one refuses to build, so the capacity is a
        // decision made at every deployment (´tab:construction:parameters´),
        // (´cav:surface:no-local-figures´).
        let Some(command_channel_capacity) = config.infrastructure.command_channel_capacity else {
            return Err(BuildError::InvalidParameter {
                path: "infrastructure.command_channel_capacity",
                value: f64::NAN,
                reason: "must be set by the host; the command channel's capacity has no default",
            });
        };
        let (command_tx, command_rx) = crossbeam_channel::bounded::<ModelOwnerCommand>(command_channel_capacity);

        // The cold ramp gets its own queue rather than sharing the control
        // channel (´alg:standardisation:batch-initialisation´). Observations
        // are one per request for the length of the ramp and control commands
        // are rare and must not be dropped, so one bounded queue between them
        // would let the ramp's own traffic delay the registrations and
        // shutdowns it has no business delaying. The bound is the horizon
        // itself: a ramp never has more observations outstanding than it has
        // mass left to retire, so it is the mechanism's own figure and not a
        // second one to tune.
        let (observation_tx, observation_rx) =
            crossbeam_channel::bounded::<owner::commands::ColdRampObservation>(config.standardisation.n_init as usize);

        // Label channel: configurable capacity.
        let (label_tx, label_rx) = crossbeam_channel::bounded::<SequencedLabel>(config.infrastructure.label_channel_capacity);

        // Layer 2 infrastructure, the representation layer (´sec:representation:scope´):
        // pending buffer, signal cache, outcome ledger.
        let signal_cache = Arc::new(signal::SignalCache::new(
            config.infrastructure.signal_cache_capacity,
            signal_schema,
        ));
        let pending_buffer = pending::PendingBuffer::new(
            config.infrastructure.pending_buffer_capacity(),
            Duration::from_secs(config.infrastructure.expiry_horizon_secs),
        );
        // The restored per-Sentinel ledgers are attached to the runtime
        // outcome ledger before it is shared, so the state the checkpoint
        // carried is the state the restart serves
        // (´dec:durability:checkpoint-journal´).
        #[cfg(feature = "serde")]
        let outcome_ledger = {
            let mut outcome_ledger = ledger::OutcomeLedger::new();
            if let Some(ledger_state) = restored_ledger_state {
                let ledgers = ledger_state.into_iter().map(|(id, lp)| (id, lp.ledger)).collect();
                outcome_ledger.restore(ledger::OutcomeLedgerSnapshot { ledgers });
            }
            Arc::new(outcome_ledger)
        };
        #[cfg(not(feature = "serde"))]
        let outcome_ledger = Arc::new(ledger::OutcomeLedger::new());
        let identity_dimensions = Arc::new(RwLock::new(HashMap::new()));

        // Health event channel.
        let (event_tx, event_rx) = health::create_event_channel(config.infrastructure.health_event_capacity);
        let dropped_counter = Arc::new(AtomicU64::new(0));
        let drift_reset_counter = Arc::new(health::DriftResetCounter::new());

        let concordance_config = health::ConcordanceConfig {
            window_capacity: config.concordance.window_capacity,
            recalibration_interval: config.concordance.recalibration_interval as u64,
            calibration_percentile: config.concordance.percentile,
        };
        #[cfg(feature = "serde")]
        let concordance = Arc::new(match restored_concordance_state {
            Some(state) => health::ConcordanceTracker::from_checkpoint(concordance_config, state),
            None => health::ConcordanceTracker::new(concordance_config),
        });
        #[cfg(not(feature = "serde"))]
        let concordance = Arc::new(health::ConcordanceTracker::new(concordance_config));

        let config_arc = Arc::new(config.clone());

        // The identity graphs have a dedicated owner (´dec:memory:graph-owner´): spawn identity maintenance
        // before the model owner so the owner can hold its command sender
        // for the checkpoint's prepare-and-publish coordination.
        // The maintenance thread also drains the signal cache's deferred
        // writes: the assessment path enqueues, this thread applies
        // (´inv:runtime:enumerated-writes´).
        let (identity_handle, identity_cmd_tx) = identity::spawn_identity_maintenance_thread(
            &config.instance_id,
            command_tx.clone(),
            Arc::clone(&clock),
            Arc::clone(&signal_cache),
        );

        let owner = ModelOwner::new(
            working,
            Arc::clone(&shared),
            command_rx,
            observation_rx,
            label_rx,
            Arc::clone(&outcome_ledger),
            Arc::clone(&concordance),
            event_tx,
            Arc::clone(&dropped_counter),
            Arc::clone(&drift_reset_counter),
            Arc::clone(&identity_dimensions),
            Arc::clone(&config_arc),
            Arc::clone(&clock),
        )
        .with_persistence(config.persistence.clone())
        .with_identity_maintenance(identity_cmd_tx.clone());

        #[cfg(feature = "serde")]
        let owner = owner.with_structural_metadata(structural_metadata);

        // The owner truncates the journal at each checkpoint through the
        // same mutex `label()` appends under.
        #[cfg(feature = "serde")]
        let owner = owner.with_journal(journal.as_ref().map(Arc::clone));

        // The owner-held state a checkpoint captured beside the working
        // copy returns to the owner that holds it.
        #[cfg(feature = "serde")]
        let owner = match restored_owner_state {
            Some(state) => owner.with_restored_owner_state(state),
            None => owner,
        };

        // Liveness channel: `owner_alive_tx` lives on the model-owner
        // thread's stack and is dropped when the thread exits for any
        // reason (normal shutdown, unwinding panic, …). Callers block
        // on `owner_alive_rx` via `crossbeam_channel::Select` to detect
        // thread death without hanging on per-submission completion.
        let (owner_alive_tx, owner_alive_rx) = crossbeam_channel::bounded::<()>(1);

        let thread_name = format!("{}-model-owner", config.instance_id);
        let model_owner_handle = std::thread::Builder::new()
            .name(thread_name.clone())
            .spawn(move || {
                // `_alive` is kept on the stack solely for its `Drop`;
                // dropping disconnects `owner_alive_rx`.
                let _alive = owner_alive_tx;
                owner.run();
            })
            .unwrap_or_else(|e| panic!("failed to spawn model-owner thread {thread_name:?}: {e}"));

        // Spawn checkpoint scheduler if persistence is enabled.
        let (checkpoint_handle, checkpoint_control_tx) = if let Some(ref persistence_config) = config.persistence {
            let (ctl_tx, ctl_rx) = crossbeam_channel::bounded::<()>(0);
            let handle = persistence::scheduler::spawn_checkpoint_scheduler(
                &config.instance_id,
                persistence_config.checkpoint_interval,
                command_tx.clone(),
                ctl_rx,
            );
            (Some(handle), Some(ctl_tx))
        } else {
            (None, None)
        };

        // Ledger collection: a maintenance task sweeps each Sentinel's
        // Ledger on the fixed cadence, reading the floor and horizon
        // from the Ledger configuration (´alg:ledger:garbage-collection´).
        let (ledger_gc_control_tx, ledger_gc_control_rx) = crossbeam_channel::bounded::<()>(0);
        let ledger_gc_handle = ledger::spawn_ledger_gc_scheduler(
            &config.instance_id,
            ledger::SWEEP_INTERVAL,
            Arc::clone(&outcome_ledger),
            ledger::SweepConfig {
                floor: config.ledger.gc_floor,
                horizon: std::time::Duration::from_secs(u64::from(config.ledger.gc_horizon_days) * 24 * 3600),
            },
            Arc::clone(&clock),
            ledger_gc_control_rx,
        );

        info!(
            instance_id = %config.instance_id,
            persistence = config.persistence.is_some(),
            "Assayer started"
        );

        // Replay journal entries recovered from checkpoint (requires serde feature).
        // These are sent as Replay labels, processed on the model-owner
        // thread through the same 17-step pipeline the original ran
        // (´cor:durability:replay-exactness´).
        //
        // The journal is self-contained (´dec:durability:checkpoint-journal´), so
        // replay deduplicates by assessment_id to handle the case where a label
        // was journaled but try_send failed, and the host retried.
        #[cfg(feature = "serde")]
        if !replay_entries.is_empty() {
            let total_entries = replay_entries.len();
            let mut seen_ids: HashSet<AssessmentId> = HashSet::with_capacity(total_entries);
            let mut replayed = 0usize;
            let mut skipped_duplicate = 0usize;

            for entry in replay_entries {
                // Deduplicate by assessment_id (´dec:durability:checkpoint-journal´).
                if !seen_ids.insert(entry.label.assessment_id) {
                    skipped_duplicate += 1;
                    continue;
                }

                let sequenced = SequencedLabel {
                    seq: Some(entry.seq),
                    label: entry.label,
                    context: owner::commands::LabelContext::Replay(Box::new(entry.context)),
                    // A replayed label crossed a restart, so the instant it
                    // originally arrived belongs to a monotonic domain that no
                    // longer exists. It carries no arrival and contributes to
                    // no latency stage (´def:monitoring:feedback-latency´).
                    arrived_at: None,
                };
                if label_tx.send(sequenced).is_err() {
                    warn!("label channel closed during journal replay");
                    break;
                }
                replayed += 1;
            }
            info!(replayed, skipped_duplicate, total_entries, "journal replay complete");
        }

        // Derive the ingestion settings before moving config into Self. The
        // absence counter is eight bits wide (´alg:ledger:entry-deletion´), so
        // a threshold it cannot represent fails construction here rather than
        // wrapping at the first report (´dec:construction:eager-validation´).
        let ingestion = report::IngestionConfig {
            n_absent_threshold: u8::try_from(config.ledger.n_absent).map_err(|_| BuildError::InvalidParameter {
                path: "ledger.n_absent",
                value: f64::from(config.ledger.n_absent),
                reason: crate::config::types::N_ABSENT_RANGE_REASON,
            })?,
        };

        // Extract blend statistics config before moving config into Self.
        let blend_stats_config = health::BlendStatisticsConfig {
            window_capacity: config.blend_statistics.window_capacity,
            publish_interval: config.blend_statistics.publish_interval as u64,
        };

        Ok(Self {
            shared,
            clock,
            config,
            label_tx,
            command_tx,
            owner_alive_rx,
            threads: AssayerThreads {
                model_owner: Some(model_owner_handle),
                checkpoint: checkpoint_handle,
                ledger_gc: Some(ledger_gc_handle),
                identity_maintenance: Some(identity_handle),
            },
            next_assessment_id: AtomicU64::new(1),
            checkpoint_control_tx,
            ledger_gc_control_tx: Some(ledger_gc_control_tx),
            sentinel_slots: DashMap::new(),
            pending_buffer,
            signal_cache,
            outcome_ledger,
            ingestion,
            identity_dimensions,
            identity_command_tx: Some(identity_cmd_tx),
            concordance,
            convergence_stage: AtomicU8::new(0),
            total_assessments: AtomicU64::new(0),
            blend_stats: health::BlendStatisticsTracker::new(blend_stats_config),
            degradation_counters: health::AssessmentDegradationCounters::new(),
            label_valences_sanitised: AtomicU64::new(0),
            label_outcomes_dropped: AtomicU64::new(0),
            label_queue_drops: AtomicU64::new(0),
            health_event_rx: event_rx,
            health_events_dropped: dropped_counter,
            drift_reset_counter,
            registered_axes: Mutex::new(HashSet::new()),
            #[cfg(feature = "serde")]
            journal,
            observation_tx,
        })
    }

    // ─────────────────────────────────────────────────────────────────────
    // Crate-internal accessors
    //
    // Sealed at crate visibility: the four call surfaces the host uses at
    // runtime are the whole public API, and none of them will ever show
    // the steward's channel or the shared state
    // (´dec:surface:guidance-opacity´). A public command sender would let
    // a host reach every mutation those surfaces exist to mediate, and a
    // public identifier allocator would mint identifiers no pending entry
    // backs (´dec:surface:assessment-identifier´).
    // ─────────────────────────────────────────────────────────────────────

    /// Returns a reference to the shared state for snapshot loading.
    #[must_use]
    pub(crate) const fn shared(&self) -> &Arc<SharedState> {
        &self.shared
    }

    /// Returns the configuration.
    #[must_use]
    pub const fn config(&self) -> &AssayerConfig {
        &self.config
    }

    /// Returns the command sender, for the checkpoint scheduler
    /// (´dec:durability:checkpoint-journal´).
    #[must_use]
    pub(crate) const fn command_tx(&self) -> &Sender<ModelOwnerCommand> {
        &self.command_tx
    }

    /// Returns the label sender.
    #[must_use]
    #[allow(dead_code)] // L0: used later (´dec:surface:async-label´)
    pub(crate) const fn label_tx(&self) -> &Sender<SequencedLabel> {
        &self.label_tx
    }

    /// Allocates a new assessment ID (´dec:surface:assessment-identifier´).
    pub(crate) fn next_assessment_id(&self) -> AssessmentId {
        AssessmentId(self.next_assessment_id.fetch_add(1, std::sync::atomic::Ordering::Relaxed))
    }

    // ─────────────────────────────────────────────────────────────────────
    // Construction helpers
    // ─────────────────────────────────────────────────────────────────────

    /// Creates a working copy, attempting checkpoint restore if
    /// persistence is configured.
    ///
    /// Returns the working copy and any journal entries that need replay.
    /// Falls back to cold start where there is no state to resume from.
    ///
    /// # Errors
    ///
    /// Returns [`BuildError::CheckpointNotPositiveDefinite`] where a
    /// checkpoint was read and structurally agreed, and the precision matrix
    /// it carries is not one the model may hold. That refusal is not a cold
    /// start: it reaches the host so that the host chooses what to do with
    /// the artefact (´dec:posterior:cascade-never-fails´).
    ///
    /// [`BuildError::CheckpointNotPositiveDefinite`]: crate::error::BuildError::CheckpointNotPositiveDefinite
    #[cfg(feature = "serde")]
    fn build_working_copy(
        config: &AssayerConfig,
        now: types::PersistentTimestamp,
    ) -> Result<
        (
            WorkingCopy,
            Vec<persistence::journal::JournalEntry>,
            Option<RestoredEngineState>,
        ),
        BuildError,
    > {
        if let Some(ref persistence) = config.persistence
            && let Some(result) = persistence::recovery::attempt_restore(
                &persistence.checkpoint_path(),
                &persistence.journal_path(),
                config,
                &[],
                &[],
                now,
            )?
        {
            info!(replay_count = result.replay_entries.len(), "restored from checkpoint");
            let restored = RestoredEngineState {
                concordance: result.concordance_state,
                owner: result.owner_state,
                ledgers: result.ledger_state,
                identity: result.identity_state,
            };
            return Ok((result.working, result.replay_entries, Some(restored)));
        }

        Ok((
            WorkingCopy::cold_start_from_schema(
                0,
                Vec::new(),
                &[],
                &config.model,
                config.cholesky.n_recompute,
                config.standardisation.n_init,
            ),
            Vec::new(),
            None,
        ))
    }

    /// Creates a working copy (no persistence without serde feature).
    #[cfg(not(feature = "serde"))]
    fn build_working_copy(config: &AssayerConfig) -> (WorkingCopy, ()) {
        (
            WorkingCopy::cold_start_from_schema(
                0,
                Vec::new(),
                &[],
                &config.model,
                config.cholesky.n_recompute,
                config.standardisation.n_init,
            ),
            (),
        )
    }

    // ─────────────────────────────────────────────────────────────────────
    // Builder pattern
    // ─────────────────────────────────────────────────────────────────────

    /// Creates a new `AssayerBuilder` with the given configuration.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use torrust_assayer::{Assayer, AssayerConfig};
    ///
    /// let mut config = AssayerConfig::default();
    /// // Host-set with no default: declare the command channel's capacity.
    /// config.infrastructure.command_channel_capacity = Some(64);
    /// let builder = Assayer::builder(config).signal_schema(&[]);
    /// let assayer = builder.build()?;
    /// ```
    #[must_use]
    pub const fn builder(config: AssayerConfig) -> api::AssayerBuilder {
        api::AssayerBuilder::new(config)
    }

    /// Builds an Assayer from validated builder state.
    ///
    /// Called by `AssayerBuilder::build()` after validation phases 1–2.
    /// Executes phases 3–9 of the build process.
    ///
    /// The builder's validated signal declarations and interaction
    /// templates are threaded through to [`WorkingCopy::cold_start_from_schema`]
    /// so the `DimensionMap` reflects the correct `p_sig` and
    /// interaction block, which the builder validated eagerly
    /// (´dec:construction:eager-validation´). The same threading is what
    /// keeps the dimension structural — derived from the schema and the
    /// templates rather than carried in the configuration
    /// (´dec:construction:two-starts´).
    ///
    /// # Errors
    ///
    /// Returns [`BuildError::PersistenceSetupFailed`] if the journal cannot be
    /// opened, and [`BuildError::CheckpointNotPositiveDefinite`] if a
    /// checkpoint carries a precision matrix the model may not hold.
    pub(crate) fn build_from_builder(
        config: AssayerConfig,
        signal_declarations: &[signal::SignalDeclaration],
        interaction_templates: &[feature::interaction::InteractionTemplate],
        clock: Arc<dyn testing::Clock>,
    ) -> Result<Self, BuildError> {
        // Build the signal schema from the builder's validated declarations.
        // This was already validated in the builder's phase 1; unwrap is safe.
        let signal_schema = Arc::new(
            signal::SignalSchemaIndex::from_declarations(signal_declarations).expect("signal schema was validated by builder"),
        );

        let p_sig = signal_schema.total_width();

        // Construction either cold-starts or restores
        // (´dec:construction:two-starts´): attempt checkpoint restore, falling
        // back to cold start where there is nothing to resume from. A
        // checkpoint that was read and structurally agreed and then refused on
        // its numbers is not that case and fails the build instead
        // (´dec:posterior:cascade-never-fails´). The builder path has the
        // structural declarations needed for mismatch detection
        // (´dec:durability:structural-compatibility´).
        #[cfg(feature = "serde")]
        let (working, replay_entries, restored) = {
            let restored = if let Some(ref persistence) = config.persistence
                && let Some(result) = persistence::recovery::attempt_restore(
                    &persistence.checkpoint_path(),
                    &persistence.journal_path(),
                    &config,
                    signal_declarations,
                    interaction_templates,
                    clock.now(),
                )? {
                info!(
                    replay_count = result.replay_entries.len(),
                    p = result.working.dimension_map.p,
                    "restored from checkpoint (builder path)"
                );
                let state = RestoredEngineState {
                    concordance: result.concordance_state,
                    owner: result.owner_state,
                    ledgers: result.ledger_state,
                    identity: result.identity_state,
                };
                Some((result.working, result.replay_entries, state))
            } else {
                None
            };
            if let Some((working, entries, state)) = restored {
                (working, entries, Some(state))
            } else {
                let working = WorkingCopy::cold_start_from_schema(
                    p_sig,
                    interaction_templates.to_vec(),
                    &crate::signal::expand_signal_classes(signal_declarations),
                    &config.model,
                    config.cholesky.n_recompute,
                    config.standardisation.n_init,
                );
                info!(p = working.dimension_map.p, p_sig, "cold start dimension derived from schema");
                (working, Vec::new(), None)
            }
        };

        #[cfg(not(feature = "serde"))]
        let working = {
            let w = WorkingCopy::cold_start_from_schema(
                p_sig,
                interaction_templates.to_vec(),
                &crate::signal::expand_signal_classes(signal_declarations),
                &config.model,
                config.cholesky.n_recompute,
                config.standardisation.n_init,
            );
            info!(p = w.dimension_map.p, p_sig, "cold start dimension derived from schema");
            w
        };

        #[cfg(feature = "serde")]
        return {
            let structural_metadata = persistence::checkpoint::StructuralMetadata {
                signal_schema: signal_declarations.to_vec(),
                interaction_templates: interaction_templates.to_vec(),
            };
            Self::build_inner(
                config,
                working,
                signal_schema,
                DurableStart {
                    replay_entries,
                    structural_metadata,
                    restored,
                },
                clock,
            )
        };
        #[cfg(not(feature = "serde"))]
        Self::build_inner(config, working, signal_schema, clock)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Shared-State Implementation
// ═══════════════════════════════════════════════════════════════════════════════

// Every lock taken below recovers its guard through `PoisonError::into_inner`:
// this trait is the assessment path's shared state, the call is infallible
// (´dec:degradation:infallible-core´), and the state behind a poisoned lock is
// the last state known good that the posture requires be read rather than
// refused (´dec:degradation:retain-and-flag´).
impl assessment::AssessmentSharedState for Assayer {
    fn now_persistent(&self) -> types::PersistentTimestamp {
        self.clock.now()
    }

    fn concordance_thresholds(&self) -> [f64; types::SCORING_AXIS_COUNT] {
        self.concordance.current_thresholds()
    }

    fn observe_concordance(&self, per_axis_max_z: [f64; types::SCORING_AXIS_COUNT]) {
        self.concordance.observe(per_axis_max_z);
    }

    fn convergence_stage(&self) -> CompositeConvergenceStage {
        let raw = self.convergence_stage.load(std::sync::atomic::Ordering::Relaxed);
        // SAFETY: CompositeConvergenceStage is repr(u8) with variants 0–5.
        // Values outside that range fall back to ColdStart.
        match raw {
            1 => CompositeConvergenceStage::AnchorEmerging,
            2 => CompositeConvergenceStage::PreCalibration,
            3 => CompositeConvergenceStage::SisterConverging,
            4 => CompositeConvergenceStage::InteractionMaturing,
            5 => CompositeConvergenceStage::SteadyState,
            _ => CompositeConvergenceStage::ColdStart,
        }
    }

    fn uncertainty_inflation(&self) -> Option<f64> {
        self.shared
            .health
            .load()
            .discrimination
            .as_ref()
            .and_then(|d| d.uncertainty_inflation)
    }

    fn anchor_regime_frozen(&self, anchor_regime_records: f64) -> bool {
        let health = self.shared.health.load();
        health.platt_tracker.refits_completed > 0 && anchor_regime_records < f64::from(self.config.platt.n_cal_min)
    }

    fn drift_threshold(&self) -> f64 {
        self.config.monitoring.h_threshold
    }

    fn next_assessment_id(&self) -> AssessmentId {
        // One allocator: the inherent crate-internal method is the
        // implementation, so the two cannot drift apart.
        Self::next_assessment_id(self)
    }

    fn insert_pending(&self, pending: pending::PendingAssessment) {
        self.pending_buffer.insert(pending);
    }

    fn take_pending_buffer_evictions(&self) -> u64 {
        self.pending_buffer.take_evictions()
    }

    fn sentinel_report(&self, sentinel_id: SentinelId) -> Option<Arc<report::ReportIndex>> {
        self.sentinel_slots
            .get(&sentinel_id)
            .map(|slot| Arc::clone(&slot.report_index.load_full()))
    }

    fn sentinel_ledger(&self, sentinel_id: SentinelId) -> Option<Arc<std::sync::RwLock<ledger::SentinelLedger>>> {
        self.outcome_ledger.get_arc(sentinel_id)
    }

    fn extraction_config(&self) -> extraction::ExtractionConfig {
        extraction::ExtractionConfig {
            concordance_thresholds: self.concordance.current_thresholds(),
            // (´tab:config:extraction´)
            d_chain_norm: 16,
        }
    }

    fn ledger_immaturity_criteria(&self) -> ledger::ImmaturityCriteria {
        ledger::ImmaturityCriteria {
            // (´def:config:ledger-materiality-threshold´)
            materiality_threshold: self.config.monitoring.ledger_materiality_threshold,
            // (´def:config:attenuation-materiality-floor´)
            attenuation_floor: self.config.monitoring.attenuation_materiality_floor,
            // (´tab:ledger:entry-state´)
            lambda_l: self.config.ledger.lambda_l,
        }
    }

    fn outcome_axis_ids(&self) -> Vec<OutcomeAxisId> {
        let snapshot = self.shared.published.load();
        snapshot.outcome_models.keys().copied().collect()
    }

    fn gamma_t_ledger(&self) -> f64 {
        self.config.temporal.gamma_t_ledger
    }

    fn gamma_t_core(&self) -> f64 {
        self.config.temporal.gamma_t_core
    }

    fn feature_storage_precision(&self) -> pending::StoragePrecision {
        self.config.infrastructure.feature_storage_precision
    }

    fn registered_sentinels(&self) -> Vec<SentinelId> {
        self.sentinel_slots.iter().map(|entry| *entry.key()).collect()
    }

    fn registered_identity_dimensions(&self) -> Vec<DimensionId> {
        let guard = self
            .identity_dimensions
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.keys().copied().collect()
    }

    fn encode_for_dimension(&self, dim_id: DimensionId, entity: &types::EntityKey) -> Option<u128> {
        let guard = self
            .identity_dimensions
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.get(&dim_id).map(|infra| infra.dimension.encode_entity(entity))
    }

    fn find_active_cells(&self, dim_id: DimensionId, coord: u128) -> Vec<identity::CompetitiveCellId> {
        let guard = self
            .identity_dimensions
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard
            .get(&dim_id)
            .map(|infra| infra.find_active_cells(coord).to_vec())
            .unwrap_or_default()
    }

    fn record_active_indicator_count(&self, dim_id: DimensionId, active: usize) {
        let guard = self
            .identity_dimensions
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(infra) = guard.get(&dim_id) {
            infra.record_active_indicator_count(active);
        }
    }

    fn competitive_set_len(&self, dim_id: DimensionId) -> usize {
        let guard = self
            .identity_dimensions
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.get(&dim_id).map_or(0, |infra| infra.load_competitive_set().len())
    }

    fn graph_total_importance(&self, dim_id: DimensionId) -> f64 {
        let guard = self
            .identity_dimensions
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard
            .get(&dim_id)
            .map_or(0.0, |infra| infra.load_competitive_set().total_importance())
    }

    fn dimension_domain_bits(&self, dim_id: DimensionId) -> Option<u8> {
        let guard = self
            .identity_dimensions
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.get(&dim_id).map(|infra| infra.dimension.domain_bits)
    }

    fn cell_measurement_state(
        &self,
        dim_id: DimensionId,
        cell_id: &identity::CompetitiveCellId,
    ) -> Option<identity::MeasurementState> {
        let guard = self
            .identity_dimensions
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.get(&dim_id).and_then(|infra| {
            infra.with_cell_state(cell_id, |mutex| {
                let state = mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                state.measurement.clone()
            })
        })
    }

    fn cell_outcome_state(
        &self,
        dim_id: DimensionId,
        cell_id: &identity::CompetitiveCellId,
        now: &PersistentTimestamp,
    ) -> Option<identity::CellOutcomeState> {
        let guard = self
            .identity_dimensions
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.get(&dim_id).and_then(|infra| {
            infra.with_cell_state(cell_id, |mutex| {
                let state = mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                state.outcome.read_decayed(self.config.temporal.gamma_t_identity, now)
            })
        })
    }

    fn record_identity_observation(&self, dim_id: DimensionId, coord: u128) {
        let guard = self
            .identity_dimensions
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(infra) = guard.get(&dim_id) {
            // Uses IdentityDimensionInfra::observe() which is try_send.
            // Overflow increments a counter (degradation); observation is lost.
            // The maintenance loop applies Δ=1 internally (feed-forward).
            infra.observe(coord);
        }
    }

    fn update_cell_measurement(
        &self,
        dim_id: DimensionId,
        cell_id: &identity::CompetitiveCellId,
        sentinel_alarms: &[(SentinelId, f64)],
    ) {
        let guard = self
            .identity_dimensions
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some(infra) = guard.get(&dim_id) {
            infra.with_cell_state(cell_id, |mutex| {
                // Measurement smoothing complement (´tab:keyspace:decay-rates´).
                const ALPHA: f64 = 0.05;

                let mut state = mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                {
                    let meas = &mut state.measurement;
                    // Each entry averages its own Sentinel's composite alarm
                    // (´tab:keyspace:measurement-state´).
                    let before = meas.suspicion();
                    for &(sid, alarm) in sentinel_alarms {
                        let entry = meas.per_sentinel_alarm_ewma.entry(sid).or_insert(0.0);
                        *entry = (1.0 - ALPHA).mul_add(*entry, ALPHA * alarm);
                    }
                    // Volatility: squared change in the derived alarm average
                    // (´tab:keyspace:measurement-state´).
                    let delta = meas.suspicion() - before;
                    meas.volatility = (1.0 - ALPHA).mul_add(meas.volatility, ALPHA * delta * delta);
                    meas.step_count += 1;
                }
                drop(state);
            });
        }
    }

    fn lambda_prior(&self) -> f64 {
        self.config.model.lambda_prior
    }

    fn signal_cache_get_and_merge(
        &self,
        entity: &types::EntityKey,
        signals: &std::collections::HashMap<String, signal::SignalValue>,
    ) -> Vec<f64> {
        self.signal_cache.get_and_merge(entity, signals)
    }

    fn count_signal_shape_mismatches(&self, signals: &std::collections::HashMap<String, signal::SignalValue>) -> u32 {
        self.signal_cache.count_shape_mismatches(signals)
    }

    fn count_unknown_signals(&self, signals: &std::collections::HashMap<String, signal::SignalValue>) -> u32 {
        self.signal_cache.count_unknown_signals(signals)
    }

    fn offer_cold_observation(&self, phi_raw: &[f64], layout_generation: u64) -> assessment::BatchInitObservation {
        use assessment::BatchInitObservation;

        // Step 2 of (´alg:standardisation:batch-initialisation´): the offer
        // neither blocks nor mutates. What it hands over is the raw vector
        // and the layout it was assembled under; accumulating and publishing
        // are the steward's (´dec:concurrency:single-steward´), which is what
        // keeps an accepted observation off the snapshot the offering request
        // is answering from (´dec:ordering:score-before-evolve´).
        //
        // A channel too full to take it skips the observation and reports
        // that load condition to the caller rather than absorbing it
        // (´dec:concurrency:no-silent-drop´) (´cor:degradation:in-band-travel´).
        // What a refusal costs is pace, not a silent gap: it is one share of
        // prior mass retired later, and the host is told which.
        if self
            .observation_tx
            .try_send(owner::commands::ColdRampObservation {
                phi_raw: phi_raw.to_vec(),
                layout_generation,
            })
            .is_err()
        {
            return BatchInitObservation::SkippedContended;
        }
        BatchInitObservation::Observed
    }

    fn observe_sentinel_bootstrap(&self, sentinel_id: SentinelId, features: &crate::pending::StoredFeatures) {
        let Some(slot) = self.sentinel_slots.get(&sentinel_id) else {
            return;
        };
        // Fast-path: inactive after step 4 releases the accumulator.
        if !slot.is_bootstrap_active() {
            return;
        }
        // try_lock — never blocks the assessment path. A contended
        // observation is skipped; the next assessment picks it up.
        if let Ok(mut guard) = slot.bootstrap.try_lock()
            && let Some(acc) = guard.as_mut()
        {
            // Occupancy is prepended as 1.0: only a Sentinel that
            // reports in is observed (´alg:standardisation:sentinel-bootstrap´).
            let mut slot_values = Vec::with_capacity(features.len() + 1);
            slot_values.push(1.0);
            slot_values.extend(features.iter());
            acc.observe(&slot_values);

            if acc.is_complete() {
                if let Some(stats) = acc.complete() {
                    drop(
                        self.command_tx
                            .try_send(owner::commands::ModelOwnerCommand::BootstrapComplete { sentinel_id, stats }),
                    );
                }
                // Step 4: release the accumulator. Inline rather than
                // `clear_bootstrap()` — the mutex is already held.
                *guard = None;
                drop(guard);
                slot.bootstrap_active.store(false, std::sync::atomic::Ordering::Release);
            }
        }
    }
}

impl Drop for Assayer {
    /// Shuts down the Assayer cleanly.
    ///
    /// The threads spawned at build (´dec:construction:named-threads´) shut
    /// down in this order:
    /// 1. Checkpoint scheduler and Ledger collection scheduler
    /// 2. Identity maintenance, whose graphs have a dedicated owner
    ///    (´dec:memory:graph-owner´)
    /// 3. Model owner
    fn drop(&mut self) {
        // Step 1: Checkpoint scheduler.
        // Drop the control channel sender to signal shutdown.
        drop(self.checkpoint_control_tx.take());
        if let Some(handle) = self.threads.checkpoint.take()
            && let Err(e) = handle.join()
        {
            warn!("checkpoint scheduler thread panicked: {e:?}");
        }

        // Step 1b: Ledger collection scheduler, by the same mechanism.
        drop(self.ledger_gc_control_tx.take());
        if let Some(handle) = self.threads.ledger_gc.take()
            && let Err(e) = handle.join()
        {
            warn!("ledger GC scheduler thread panicked: {e:?}");
        }

        // Step 2: Identity maintenance.
        if let Some(ref tx) = self.identity_command_tx {
            drop(tx.send(identity::MaintenanceCommand::Shutdown));
        }
        drop(self.identity_command_tx.take());
        if let Some(handle) = self.threads.identity_maintenance.take()
            && let Err(e) = handle.join()
        {
            warn!("identity maintenance thread panicked: {e:?}");
        }

        // Step 3: Model owner.
        if self.command_tx.send(ModelOwnerCommand::Shutdown).is_err() {
            warn!("model-owner already disconnected during shutdown");
        }
        if let Some(handle) = self.threads.model_owner.take()
            && let Err(e) = handle.join()
        {
            warn!("model-owner thread panicked: {e:?}");
        }
    }
}
