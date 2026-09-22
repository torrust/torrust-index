// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Error types for all Assayer public entry points.
//!
//! This module defines the five public error enums and three NaN
//! checkpoint utility functions.  All error types are `Send + Sync`,
//! carry `#[non_exhaustive]`, and implement `std::error::Error`.
//!
//! # Error Types
//!
//! | Type | Variants | Entry point |
//! |------|----------|-------------|
//! | [`LabelError`] | 6 | `label()` |
//! | [`ReportError`] | 4 | `receive_sentinel_report()` |
//! | [`LifecycleError`] | 7 | `register()` / `deregister()` |
//! | [`BuildError`] | 7 | `Assayer::builder()` |
//! | [`ChannelError`] | 6 | host channel policy validation |
//!
//! # NaN Checkpoint Utilities
//!
//! - [`sanitise_f64`] — Replace NaN/Inf with a default value.
//! - [`check_vec_nan`] — Find indices of NaN/Inf in a slice.
//! - [`check_diagonal_nan`] — Find NaN/Inf on a matrix diagonal.
//!
//! # Cross-References
//!
//! - Structural violations are hard; data-quality problems degrade
//!   (´dec:degradation:error-partition´)
//! - Channel validation against an ordered action set
//!   (´dec:derivation:ordered-actions´)
//! - The builder validates eagerly (´dec:construction:eager-validation´)
//! - Lifecycle is six methods (´dec:construction:six-methods´)
//! - The error type catalogue this partition fixes
//!   (´dec:degradation:error-partition´)

use std::{fmt, io};

use crate::types::{CellInterval, DimensionId, ModelId, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// LabelError — label submission failures
// ═══════════════════════════════════════════════════════════════════════════════

/// Why the label path stopped, recorded once at the moment it stopped.
///
/// The label path stops when a numeric checkpoint has judged the working copy
/// corrupt, has reverted the label, and has then failed to rebuild the working
/// copy from the last published snapshot. This is what that failure was: the
/// model whose reconstruction was refused, the pivot the factorisation stopped
/// at, the model's dimension, and how far into the label stream the engine had
/// got.
///
/// The cause is a fact about the owner's state rather than about any one label,
/// which is why every submission after the stop carries the same one.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LabelPathStop {
    /// The model whose reconstruction from the snapshot was refused.
    pub model: ModelId,
    /// The pivot at which the factorisation stopped.
    pub pivot: usize,
    /// The dimension of the model being reconstructed.
    pub p: usize,
    /// The count of labels the engine had taken up when the revert failed,
    /// which is where a reader looks in the journal for the last label the
    /// models could have been carrying.
    pub labels_taken_up: u64,
}

impl fmt::Display for LabelPathStop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "rebuilding model {:?} (p={}) from the last published snapshot was refused at pivot {}, at label {}",
            self.model, self.p, self.pivot, self.labels_taken_up
        )
    }
}

/// Errors from the label submission path.
///
/// Structural violations are hard while data-quality problems degrade
/// (´dec:degradation:error-partition´), and the journal is the other party
/// to a submission (´dec:durability:checkpoint-journal´).
#[derive(Debug)]
#[non_exhaustive]
pub enum LabelError {
    /// Assessment ID not found in pending buffer
    /// (expired, already labelled, or never existed).
    AssessmentNotFound,

    /// Journal write failed. The pending buffer entry was
    /// consumed (step 1) but the label was NOT journaled and
    /// NOT enqueued. **The label is lost.** The host must
    /// assess the same request again and label the new assessment.
    ///
    /// Do NOT retry with the same `assessment_id` — the pending
    /// entry is gone and the call will return `AssessmentNotFound`.
    JournalFailed(io::Error),

    /// Label is journaled for crash recovery but the label
    /// processing channel is full. Do NOT retry — the label
    /// will be recovered on restart.
    JournaledButNotEnqueued,

    /// The label processing channel is full and journaling is
    /// disabled. The label is lost. Retry may succeed if the
    /// channel drains.
    ChannelFull,

    /// Model owner has shut down.
    ModelOwnerShutdown,

    /// The label path has stopped and no further label will be applied.
    ///
    /// A numeric checkpoint judged the working copy corrupt, reverted the
    /// label, and could not rebuild the working copy from the last published
    /// snapshot. The engine holds no state it is willing to learn from, so it
    /// stops learning rather than continuing with the state it just judged
    /// corrupt.
    ///
    /// **The label is not consumed and not journaled.** The refusal is taken
    /// before the pending entry is removed, so nothing is lost here and
    /// nothing is written that would replay: retrying is pointless, but it
    /// costs the assessment nothing beyond its own expiry. This is the one
    /// place the shape differs from [`Self::ModelOwnerShutdown`], which is
    /// reached at the enqueue after the entry has been consumed and, under
    /// persistence, after the journal has taken the label.
    ///
    /// This is a state of the owner and not of the label, so it is permanent
    /// for the life of the process and every later call returns it with the
    /// same cause. The engine keeps answering assessments from the last
    /// published snapshot throughout, and the command path — lifecycle,
    /// checkpoint, shutdown — keeps working, which is what makes the host's
    /// two actions available: **take a checkpoint and restart**, or **restore
    /// an earlier checkpoint**. Labels journaled before the stop replay on
    /// restart under the recovery already defined
    /// (´dec:durability:checkpoint-journal´). Replay is deterministic, so a
    /// journal replayed in full onto the checkpoint it was written against
    /// reaches the state that stopped the path and stops it again; the exits
    /// are an earlier checkpoint, or a restart that does not replay the
    /// journal suffix past `labels_taken_up`. Choosing between them is the
    /// host's, and the flag `label_path_stopped` on the health surface is how
    /// it learns the outcome either way.
    LabelPathStopped(LabelPathStop),
}

impl fmt::Display for LabelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AssessmentNotFound => {
                write!(f, "assessment ID not found in pending buffer")
            }
            Self::JournalFailed(e) => {
                write!(f, "journal write failed: {e}")
            }
            Self::JournaledButNotEnqueued => {
                write!(f, "label journaled but not enqueued (channel full); will recover on restart")
            }
            Self::ChannelFull => {
                write!(f, "label processing channel is full")
            }
            Self::ModelOwnerShutdown => {
                write!(f, "model owner has shut down")
            }
            Self::LabelPathStopped(stop) => {
                write!(f, "the label path has stopped: {stop}")
            }
        }
    }
}

impl std::error::Error for LabelError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::JournalFailed(e) => Some(e),
            _ => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ReportError — Sentinel report ingestion failures
// ═══════════════════════════════════════════════════════════════════════════════

/// Errors from Sentinel report ingestion.
///
/// Structural contract violations (´pre:architecture:report-validation´) are hard errors:
/// the report is rejected and the adapter layer should investigate.
/// Data quality issues (NaN in scores) are handled as per-cell
/// degradation — the report is accepted with affected cells zeroed.
///
/// The partition between the two is the error model
/// (´dec:degradation:error-partition´).
#[derive(Debug)]
#[non_exhaustive]
pub enum ReportError {
    /// The report contains no root cell (depth 0).
    /// Violates the rule that the root cell is present in every batch
    /// report (´pre:architecture:report-validation´).
    MissingRootCell,

    /// Two cells share the same `(lo, depth)` key.
    ///
    /// Under the dyadic invariant (´pre:architecture:report-validation´),
    /// any same-depth collision implies two interval-identical cells — so a
    /// single `CellInterval` fully describes the violation, which the
    /// structural phase is what rejects (´dec:ordering:two-phase-validation´).
    /// `CellInterval::hi` is inclusive.
    DuplicateCell {
        /// The duplicated cell interval. Its `hi` bound is inclusive.
        cell: CellInterval,
    },

    /// A cell interval is not dyadic at its claimed depth.
    /// `CellInterval::hi` is inclusive.
    /// Rejected in the structural phase (´dec:ordering:two-phase-validation´).
    NonDyadicInterval {
        /// The non-dyadic cell interval. Its `hi` bound is inclusive.
        cell: CellInterval,
    },

    /// Unknown Sentinel ID — not registered.
    UnknownSentinel {
        /// The unrecognised Sentinel ID.
        id: SentinelId,
    },
}

impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingRootCell => {
                write!(f, "report contains no root cell (depth 0)")
            }
            Self::DuplicateCell { cell } => {
                write!(
                    f,
                    "duplicate cell interval: [{:#x}, {:#x}] depth {}",
                    cell.lo, cell.hi, cell.depth,
                )
            }
            Self::NonDyadicInterval { cell } => {
                write!(
                    f,
                    "non-dyadic cell interval: [{:#x}, {:#x}] depth {}",
                    cell.lo, cell.hi, cell.depth,
                )
            }
            Self::UnknownSentinel { id } => {
                write!(f, "unknown sentinel ID: {id:?}")
            }
        }
    }
}

impl std::error::Error for ReportError {}

// ═══════════════════════════════════════════════════════════════════════════════
// LifecycleError — lifecycle registration/deregistration failures
// ═══════════════════════════════════════════════════════════════════════════════

/// Errors from lifecycle registration/deregistration methods.
///
/// These are structural failures at the API boundary. Model-level
/// issues (Schur fallback, Cholesky regularisation) are NOT errors
/// — they are reported through the bounded health event channel
/// (´dec:health:bounded-events´).
///
/// Lifecycle is six methods (´dec:construction:six-methods´).
#[derive(Debug)]
#[non_exhaustive]
pub enum LifecycleError {
    /// A Sentinel, axis, or dimension with this ID is already registered.
    DuplicateId {
        /// The entity type (`"Sentinel"`, `"OutcomeAxis"`, `"IdentityDimension"`).
        entity_type: &'static str,
        /// String representation of the duplicate ID.
        id: String,
    },

    /// The Sentinel, axis, or dimension ID was not found for deregistration.
    NotFound {
        /// The entity type (`"Sentinel"`, `"OutcomeAxis"`, `"IdentityDimension"`).
        entity_type: &'static str,
        /// String representation of the missing ID.
        id: String,
    },

    /// The model owner thread has exited (panic or shutdown).
    ModelOwnerShutdown,

    /// The identity maintenance thread has exited.
    IdentityMaintenanceShutdown,

    /// The command channel is full. Its capacity is host-set at
    /// construction, with no default (´tab:construction:parameters´).
    CommandChannelFull,

    /// The registration name is empty.
    ///
    /// All entity registrations require a non-empty name.
    EmptyName {
        /// The entity type (`"Sentinel"`, `"OutcomeAxis"`, `"IdentityDimension"`).
        entity_type: &'static str,
    },

    /// The identity dimension's domain bit width is outside `1..=128`.
    InvalidDomainBits {
        /// The identity dimension ID being registered.
        id: DimensionId,
        /// The invalid domain bit width.
        domain_bits: u8,
    },
}

impl fmt::Display for LifecycleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DuplicateId { entity_type, id } => {
                write!(f, "duplicate {entity_type} ID: {id}")
            }
            Self::NotFound { entity_type, id } => {
                write!(f, "{entity_type} not found: {id}")
            }
            Self::ModelOwnerShutdown => {
                write!(f, "model owner has shut down")
            }
            Self::IdentityMaintenanceShutdown => {
                write!(f, "identity maintenance thread has shut down")
            }
            Self::CommandChannelFull => {
                write!(f, "lifecycle command channel is full")
            }
            Self::EmptyName { entity_type } => {
                write!(f, "{entity_type} registration name must not be empty")
            }
            Self::InvalidDomainBits { id, domain_bits } => {
                write!(
                    f,
                    "identity dimension {id:?} has invalid domain_bits={domain_bits}; expected 1..=128"
                )
            }
        }
    }
}

impl std::error::Error for LifecycleError {}

// ═══════════════════════════════════════════════════════════════════════════════
// BuildError — construction failures
// ═══════════════════════════════════════════════════════════════════════════════

/// Errors from `AssayerBuilder::build()`.
///
/// The builder validates eagerly (´dec:construction:eager-validation´).
#[derive(Debug)]
#[non_exhaustive]
pub enum BuildError {
    /// No signal schema declared. Call `signal_schema()` before `build()`.
    NoSignalSchema,

    /// Signal schema contains duplicate names.
    DuplicateSignalName {
        /// The duplicated signal name.
        name: String,
    },

    /// A signal declaration's shape parameters are structurally invalid.
    ///
    /// A structural contract violation is refused on the fallible
    /// construction surface, where the caller can fix it, rather than
    /// admitted and surfaced on the infallible assessment call
    /// (´dec:degradation:error-partition´).
    InvalidSignalShape {
        /// The declaring signal's name.
        name: String,
        /// Description of the constraint violated.
        reason: &'static str,
    },

    /// An interaction template references a feature position that
    /// exceeds the fixed-block range.
    InvalidInteractionTemplate {
        /// Index of the template in the interaction list.
        template_index: usize,
        /// Description of what is invalid.
        reason: String,
    },

    /// A numerical parameter is out of its valid range.
    InvalidParameter {
        /// Dot-path to the parameter (e.g. `"model.gamma_operational"`).
        path: &'static str,
        /// The invalid value.
        value: f64,
        /// Description of the constraint violated.
        reason: &'static str,
    },

    /// Persistence directory could not be created or is not writable.
    PersistenceSetupFailed(io::Error),

    /// Persistence was configured in a build that does not compile the
    /// checkpoint and journal codecs.
    ///
    /// Both codecs are compiled behind the `serde` feature, so an instance
    /// built without it holds no writer for either artefact. Accepting the
    /// configuration would hand the host an instance that looks durable and
    /// persists nothing, with the first evidence of that a restart with no
    /// state to restore; refusing it at construction is the same eager
    /// verdict every other unhonourable configuration gets
    /// (´dec:construction:eager-validation´).
    PersistenceUnsupported,

    /// A checkpoint's stored precision matrix is not positive definite, so
    /// the restore it was read for is refused
    /// (´dec:posterior:cascade-never-fails´).
    ///
    /// The verdict is a plain Cholesky attempt taken at the restore itself,
    /// because nothing else takes one: the checkpoint carries `B` so that no
    /// inverse has to be recomputed to rebuild the model
    /// (´dec:durability:checkpoint-journal´), and the storage boundary's only
    /// numerical verdict is symmetry. A matrix that is symmetric to the bit
    /// and indefinite by a whole eigenvalue therefore reaches this point
    /// unexamined.
    ///
    /// This is the error partition applied rather than extended: the artefact
    /// does not carry what the contract admits, which is the defect of
    /// whoever wrote it and one that can be acted on
    /// (´dec:degradation:error-partition´). A host holding this error has
    /// three real actions — restore an earlier checkpoint, rebuild from the
    /// journal, or start clean — and construction fails so that it takes one
    /// of them knowingly, rather than the instance cold-starting and
    /// discarding every learned parameter without saying so.
    CheckpointNotPositiveDefinite {
        /// Which model's stored precision matrix was refused.
        model: ModelId,
        /// The index of the first pivot the factorisation could not take.
        /// It names a coordinate of the stored matrix, which is where a
        /// reader inspecting the artefact should look first.
        pivot: usize,
    },

    /// The journal beside a restored checkpoint could not be read, so the
    /// labels acknowledged since that checkpoint cannot be replayed
    /// (´dec:durability:checkpoint-journal´).
    ///
    /// The reader already absorbs the two failures that are not losses. An
    /// absent journal reads as no entries, which is what a deployment that
    /// has never journalled looks like; a record torn by a crash during its
    /// own append is dropped from the tail, which is the one label the
    /// protocol allows to be lost. A journal of another format version is a
    /// third case with its own answer, and it is a discard by design rather
    /// than an error: an upgraded binary meeting the previous deployment's
    /// journal cold-starts instead of refusing to run.
    ///
    /// What reaches this variant is the remainder — an unreadable file, or a
    /// record corrupt before the tail, which makes every record after it
    /// unfindable. Those records are the only copy of every label
    /// acknowledged since the checkpoint was taken. Continuing with an empty
    /// replay set would discard them while reporting a successful start,
    /// which is the shape the error partition refuses
    /// (´dec:degradation:error-partition´): the host has real actions here —
    /// an earlier checkpoint, a repaired journal, a deliberate cold start —
    /// and can take one only if construction says so.
    JournalUnreadable {
        /// What the journal reader refused on, as it reported it, including
        /// the byte offset where a corruption was detected. The artefact is
        /// what an operator has to go and inspect, and the offset says where.
        reason: String,
    },

    /// A checkpoint's stored model state disagrees with the width it declares
    /// for itself, so the restore it was read for is refused
    /// (´dec:durability:structural-compatibility´).
    ///
    /// Nothing upstream of the reconstruction establishes this. The reader's
    /// checksum is an integrity check: it proves the bytes are the bytes that
    /// were written, and a state written inconsistent passes it exactly as a
    /// consistent one does. The structural check taken before the restore
    /// compares the payload against the configuration and the builder's
    /// declarations, which is a relation between the artefact and this build
    /// rather than a property of the artefact. Between the two, a state
    /// declaring one width and carrying a mean, a covariance or a precision
    /// matrix of another is admitted.
    ///
    /// What it is admitted into is a conversion that asserts. Widening the
    /// flat covariance back into a matrix takes the declared width twice and
    /// requires the entry count to be its square, in every build rather than
    /// only in a checked one, so a disagreement there ends the process at
    /// startup. That is the wrong answer to a defective artefact, for the
    /// reason the refusal beside this one already gives: the host has real
    /// actions available — an earlier checkpoint, a journal rebuild, a clean
    /// start — and can only take one of them if construction fails rather
    /// than aborts (´dec:degradation:error-partition´).
    ///
    /// The refusal names the model and both disagreeing extents, because the
    /// artefact is what a reader has to go and inspect, and those two numbers
    /// say where in it to look.
    CheckpointDimensionMismatch {
        /// Which model's stored state disagrees with itself.
        model: ModelId,
        /// The extent that disagrees, named as the stored state carries it.
        quantity: &'static str,
        /// How many of them the state carries.
        found: usize,
        /// The width the state declares. A mean and a precision row count are
        /// that many; a covariance entry count is its square.
        declared: usize,
    },
}

impl fmt::Display for BuildError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoSignalSchema => {
                write!(f, "no signal schema declared")
            }
            Self::DuplicateSignalName { name } => {
                write!(f, "duplicate signal name: {name:?}")
            }
            Self::InvalidSignalShape { name, reason } => {
                write!(f, "invalid signal shape for {name:?}: {reason}")
            }
            Self::InvalidInteractionTemplate { template_index, reason } => {
                write!(f, "invalid interaction template at index {template_index}: {reason}")
            }
            Self::InvalidParameter { path, value, reason } => {
                write!(f, "invalid parameter {path} = {value}: {reason}")
            }
            Self::PersistenceSetupFailed(e) => {
                write!(f, "persistence setup failed: {e}")
            }
            Self::PersistenceUnsupported => {
                write!(
                    f,
                    "persistence was configured, but this build compiles no checkpoint or journal codec: both need the `serde` feature"
                )
            }
            Self::JournalUnreadable { reason } => {
                write!(f, "journal beside the restored checkpoint is unreadable: {reason}")
            }
            Self::CheckpointNotPositiveDefinite { model, pivot } => {
                write!(
                    f,
                    "checkpoint refused: model {model:?} carries a precision matrix that is not positive definite (first refused pivot {pivot})"
                )
            }
            Self::CheckpointDimensionMismatch {
                model,
                quantity,
                found,
                declared,
            } => {
                write!(
                    f,
                    "checkpoint refused: model {model:?} declares width {declared} but carries {found} {quantity}"
                )
            }
        }
    }
}

impl std::error::Error for BuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::PersistenceSetupFailed(e) => Some(e),
            _ => None,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ChannelError — channel policy validation failures
// ═══════════════════════════════════════════════════════════════════════════════

/// Errors from channel policy validation at registration time.
///
/// The transform is generic over an ordered action set, and this is the
/// validation of that set (´dec:derivation:ordered-actions´).
#[derive(Debug)]
#[non_exhaustive]
pub enum ChannelError {
    /// Fewer than 2 actions declared.
    TooFewActions,

    /// Actions are not in strictly ascending order of severity.
    ActionsNotOrdered,

    /// One or more reward parameters are non-positive.
    NonPositiveReward,

    /// Slow severity is not in the open interval $(0, 1)$.
    InvalidSlowSeverity,

    /// Block-catches parameter is not in $[0, 1]$.
    InvalidBlockCatches,

    /// One or more sensitivity exponents are non-positive.
    NonPositiveSensitivity,

    /// Challenge-effectiveness decay rate `γ_{q,t}` is not in $(0, 1)$.
    InvalidGammaQt,

    /// A policy parameter holds a value that is not finite.
    ///
    /// Every other variant here reports a bound that was broken, and a bound
    /// is a pair of ordered comparisons. A value that is not a number
    /// satisfies both ends of every such pair, so it breaks no bound and
    /// needs a refusal of its own — one that names the field, since the
    /// arithmetic downstream gives no indication of which of a dozen policy
    /// numbers arrived unusable.
    NonFiniteParameter {
        /// The policy field that carried the value.
        field: &'static str,
        /// The value the field held.
        value: f64,
    },
}

impl fmt::Display for ChannelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooFewActions => {
                write!(f, "channel requires at least 2 actions")
            }
            Self::ActionsNotOrdered => {
                write!(f, "actions must be in strictly ascending order of severity")
            }
            Self::NonPositiveReward => {
                write!(f, "all reward parameters must be positive")
            }
            Self::InvalidSlowSeverity => {
                write!(f, "slow_severity must be in (0, 1)")
            }
            Self::InvalidBlockCatches => {
                write!(f, "block_catches must be in [0, 1]")
            }
            Self::NonPositiveSensitivity => {
                write!(f, "sensitivity exponents must be positive")
            }
            Self::InvalidGammaQt => {
                write!(f, "gamma_q_t must be in (0, 1)")
            }
            Self::NonFiniteParameter { field, value } => {
                write!(f, "{field} must be a finite value, got {value}")
            }
        }
    }
}

impl std::error::Error for ChannelError {}

// ═══════════════════════════════════════════════════════════════════════════════
// NaN Checkpoint Utilities
// ═══════════════════════════════════════════════════════════════════════════════

/// Sanitises a single `f64` value, replacing NaN and infinite values with
/// a default.
///
/// Returns a tuple of `(sanitised_value, was_replaced)`.
///
/// Used by the numeric checkpoints (´tab:runtime:numeric-checkpoints´),
/// whose placement is definition rather than decision
/// (´cav:degradation:checkpoint-placement´).
///
/// # Examples
///
/// ```
/// use torrust_assayer::error::sanitise_f64;
///
/// assert_eq!(sanitise_f64(2.72, 0.0), (2.72, false));
/// assert_eq!(sanitise_f64(f64::NAN, 0.0), (0.0, true));
/// assert_eq!(sanitise_f64(f64::INFINITY, 0.0), (0.0, true));
/// assert_eq!(sanitise_f64(f64::NEG_INFINITY, 0.0), (0.0, true));
/// ```
#[must_use]
#[inline]
pub const fn sanitise_f64(val: f64, default: f64) -> (f64, bool) {
    if val.is_finite() { (val, false) } else { (default, true) }
}

/// Returns the indices of NaN or infinite elements in a slice.
///
/// A spot-check convenience for hosts and test harnesses. The
/// production checkpoints CP5–CP6 are placed by
/// (´tab:runtime:numeric-checkpoints´) and take their readings with
/// their own guards on the label path.
///
/// # Examples
///
/// ```
/// use torrust_assayer::error::check_vec_nan;
///
/// let data = [1.0, f64::NAN, 3.0, f64::INFINITY, 5.0];
/// assert_eq!(check_vec_nan(&data), vec![1, 3]);
///
/// let clean = [1.0, 2.0, 3.0];
/// assert!(check_vec_nan(&clean).is_empty());
/// ```
#[must_use]
pub fn check_vec_nan(slice: &[f64]) -> Vec<usize> {
    slice
        .iter()
        .enumerate()
        .filter(|(_, v)| !v.is_finite())
        .map(|(i, _)| i)
        .collect()
}

/// Returns the indices of diagonal elements that are NaN or infinite.
///
/// Accepts a slice of diagonal values. When `SymmetricMatrix`
/// is available, pass its diagonal slice here.
///
/// A spot-check convenience for hosts and test harnesses, like
/// [`check_vec_nan`]; the production checkpoints are placed by
/// (´tab:runtime:numeric-checkpoints´).
///
/// # Examples
///
/// ```
/// use torrust_assayer::error::check_diagonal_nan;
///
/// let diag = [1.0, 2.0, 3.0];
/// assert!(check_diagonal_nan(&diag).is_empty());
///
/// let diag = [1.0, f64::NAN, 3.0];
/// assert_eq!(check_diagonal_nan(&diag), vec![1]);
/// ```
#[must_use]
pub fn check_diagonal_nan(diagonal: &[f64]) -> Vec<usize> {
    check_vec_nan(diagonal)
}
