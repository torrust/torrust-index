// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Atomic checkpoint writer and reader.
//!
//! Checkpoints capture the full working-copy state in a single file with:
//!
//! | Offset | Size | Content |
//! |--------|------|---------|
//! | 0 | 4 | Magic bytes `ASAY` |
//! | 4 | 4 | Format version (little-endian `u32`) |
//! | 8 | 4 | CRC32 of payload (little-endian `u32`) |
//! | 12 | variable | Postcard-serialised `CheckpointPayload` |
//!
//! ## Atomic Write Protocol
//!
//! 1. Write to `{path}.new`
//! 2. `sync_all()` the temporary file
//! 3. `rename()` to `{path}`
//! 4. Sync the directory holding `{path}`, so the entry the rename
//!    published is durable and not merely visible
//!
//! Crash at any point leaves either the old or the new file — never
//! a partial write, and never a file whose contents survived while the
//! name still points at its predecessor.
//!
//! ## Format Version Policy
//!
//! Version mismatch on restore always results in cold start. No
//! migration path. The checkpoint format is internal and unstable.
//!
//! # Cross-References
//!
//! - (´dec:durability:checkpoint-journal´) — the periodic whole-state checkpoint
//! - (´dec:durability:file-framing´) — the on-disk framing this module writes

use std::io::{self, Write};
use std::path::Path;
use std::{fmt, fs};

use serde::{Deserialize, Serialize};

use crate::feature::dimension_map::DimensionMap;
use crate::feature::interaction::InteractionTemplate;
use crate::identity::{CellOutcomeState, CompetitiveCellId, IdentityGraphSnapshot};
use crate::ledger::sentinel_ledger::SentinelLedger;
use crate::linalg::symmetric::SymmetricMatrix;
use crate::model::parameters::ModelParameters;
use crate::signal::SignalDeclaration;
use crate::types::{DimensionId, PersistentTimestamp, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Magic bytes identifying an Assayer checkpoint file.
///
/// The four opening bytes of the header: the package's three-byte prefix and
/// the byte that names the role, which is what refuses a journal offered as a
/// checkpoint before its generation is read (´dec:durability:file-framing´).
///
/// ´const:assayer:checkpoint-file-signature´ (´alg:const:form´)
/// ´const:assayer:checkpoint-file-signature-form-x4a92c817´
const CHECKPOINT_MAGIC: [u8; 4] = *b"ASAY";

/// Current format version. Incremented on any layout change.
/// Layer 2 bumped 1 → 2 for ledger + identity state.
/// Layer 5 bumped 2 → 3 for enriched registration types + health summary fields.
/// Layer 5 bumped 3 → 4 for structural metadata (´cav:construction:schema-fixity´).
/// Layer 5 bumped 4 → 5 for outcome-axis spatial metadata.
/// The second work package of the retired plan bumped 5 → 6 for
/// channel-free core construction metadata.
/// Review follow-up bumped 6 → 7 for `ConcordanceTracker` state.
/// Bincode 2 migration bumped 7 → 8 for the wire-format change.
/// The repair campaign bumped 8 → 9 for the whole-state payload
/// (´dec:durability:checkpoint-journal´): per-axis metadata,
/// standardisation statistics, drift state, and owner-held state.
/// The repair campaign bumped 9 → 10 for the empirical-coverage
/// fields on the persisted discrimination metrics
/// (´alg:monitoring:empirical-coverage´).
/// The owner-layout repair bumped 10 → 11 because the persisted model
/// parameters and standardisation vectors moved to canonical feature order
/// (´entry:assayer:wl-owner-layout-misalignment´).
/// The ramp stewardship work bumped 11 → 12 for cold-ramp progress added
/// to the whole-state payload (´dec:durability:ramp-resumption´).
/// The marginalisation repair bumped 12 → 13 for the marginalisation-error
/// ledger joining the whole-state payload (´dec:durability:checkpoint-journal´).
/// The materiality work bumped 13 → 14 for the Ledger entry's materiality
/// evidence: the raw adverse-eligible count and the eligible arrival window
/// (´tab:ledger:entry-state´), (´thm:ledger:materiality´).
/// The hibernation work bumped 14 → 15 for the hibernation archive, which
/// crosses the restart with the models it describes (´alg:registry:hibernation´).
/// The legibility work bumped 15 → 16 for the per-block legibility evidence the
/// health counters carry (´def:monitoring:slot-association´),
/// (´def:monitoring:slot-contribution´).
/// The recomputation work bumped 16 → 17 for the spatial-axis identities the
/// dimension map records against each Sentinel slot's outcome-memory pairs
/// (´def:extraction:slot´), (´entry:assayer:wl-feature-frozen-slot-truncation´).
/// The recomputation work bumped 17 → 18 for the rebuild's baseline record and
/// the two shares of the precision matrix the prior is holding up
/// (´dec:posterior:recomputation-trigger´), (´dec:posterior:spectral-floor´).
/// The measured-adoption work bumped 18 → 19 for the baseline's split into the
/// readings every visit takes and the record only a rebuild writes, and for the
/// count of visits that rebuilt nothing (´dec:posterior:measured-adoption´).
/// This change bumps 19 → 20 for the codec change: the payload is
/// postcard-encoded. Both encodings write integers as varints and `f64` as
/// eight fixed little-endian bytes, and they agree byte for byte on integers
/// below 128, so a checkpoint of the previous generation would not reliably
/// fail to decode; the generation is what refuses it.
///
/// That earlier bump is what a map field costs, and it is the whole of the
/// compatibility answer. The codec is not self-describing and the format policy
/// stated at the head of this module is that a version mismatch cold-starts
/// rather than migrating, so a checkpoint written before the identities existed
/// is refused as a whole. Reading one instead would be worse than refusing it:
/// the payload's model parameters and standardisation vectors are indexed
/// against the map they were saved with, and a map decoded without its slot
/// identities cannot say which axis owns which pair, which is the exact
/// misreading this generation exists to end.
///
/// The identities cross the restart rather than being recomputed on the way in
/// because the map is what the saved vectors are indexed against. Rebuilding it
/// from the restored outcome models would produce the identities the instance
/// has now, not the ones the saved parameters were laid out under, and the two
/// differ precisely when an axis was retired between the save and the restore —
/// the case that would otherwise pass unnoticed.
///
/// The version exists so that a restore can refuse a checkpoint whose
/// structure does not match the instance being built
/// (´dec:durability:structural-compatibility´); the bump is the design
/// rather than an accident, so the citations that dangle when it moves
/// are the migration's own checklist. That checklist is the bump being
/// made rather than the lines above it: each of those records a bump
/// that has landed and is not restated afterwards, so where an agent's
/// own name has since been retired the line describes that agent
/// rather than writing the name.
///
/// ´const:assayer:checkpoint-layout-generation´ (´alg:const:version´)
/// ´const:assayer:checkpoint-layout-generation-version-20´
const CHECKPOINT_FORMAT_VERSION: u32 = 20;

/// Size of the header (magic + version + CRC32).
///
/// Three four-byte fields — signature, generation, and the CRC32 over the
/// payload that only the checkpoint carries — so the width is the sum of the
/// layout rather than a figure of its own (´cor:durability:header-width´). It is
/// read here as the offset at which the payload begins.
///
/// ´const:assayer:checkpoint-header-width´ (´alg:const:count´)
/// ´const:assayer:checkpoint-header-width-count-12´
const HEADER_SIZE: usize = 12;

// ═══════════════════════════════════════════════════════════════════════════════
// Error Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Error reading or validating a checkpoint.
#[derive(Debug)]
pub enum CheckpointError {
    /// I/O error reading/writing the file.
    Io(io::Error),
    /// File does not start with the expected magic bytes.
    InvalidMagic {
        /// The bytes found.
        found: [u8; 4],
    },
    /// Format version in the file does not match the current version.
    VersionMismatch {
        /// Version found in the file.
        found: u32,
        /// Version expected by this binary.
        expected: u32,
    },
    /// CRC32 of the payload does not match the header.
    IntegrityFailed {
        /// CRC32 recorded in the header.
        expected: u32,
        /// CRC32 computed from the payload.
        actual: u32,
    },
    /// Postcard deserialisation failed.
    Deserialise(String),
    /// The checkpoint is structurally incompatible with the current config.
    #[allow(dead_code)] // used by recovery module (´dec:durability:structural-compatibility´)
    StructuralMismatch {
        /// Human-readable description of the mismatch.
        reason: String,
    },
}

impl fmt::Display for CheckpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "checkpoint I/O error: {e}"),
            Self::InvalidMagic { found } => write!(f, "invalid checkpoint magic: {found:02x?}"),
            Self::VersionMismatch { found, expected } => {
                write!(f, "checkpoint version mismatch: found {found}, expected {expected}")
            }
            Self::IntegrityFailed { expected, actual } => {
                write!(
                    f,
                    "checkpoint CRC32 mismatch: header {expected:#010x}, computed {actual:#010x}"
                )
            }
            Self::Deserialise(msg) => write!(f, "checkpoint deserialisation failed: {msg}"),
            Self::StructuralMismatch { reason } => {
                write!(f, "checkpoint structural mismatch: {reason}")
            }
        }
    }
}

impl std::error::Error for CheckpointError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for CheckpointError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Checkpoint Payload
// ═══════════════════════════════════════════════════════════════════════════════

/// Serialisable model state for a single `BayesianLinearModel`.
///
/// Captures everything needed to reconstruct the model without
/// `cholesky_inverse`: μ, B, Σ, plus the tracking fields the
/// recomputation trigger reads (´dec:posterior:recomputation-trigger´).
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckpointModelState {
    /// Serialisable model parameters (μ, Σ column-major flat, p).
    pub parameters: ModelParameters,
    /// Precision matrix B — column-major flat layout.
    pub precision: SymmetricMatrix,
    /// Labels processed since last Cholesky recomputation.
    pub labels_since_recompute: u32,
    /// Effective recompute interval (´dec:posterior:recomputation-trigger´).
    pub n_recompute_effective: u32,
    /// Consecutive clean (unregularised) recomputes (´dec:posterior:recomputation-trigger´).
    pub consecutive_clean_recomputes: u32,
    /// The cheap diagonal ratio last read, a lower bound on the true condition
    /// number (´def:monitoring:condition-number´).
    pub last_diagonal_ratio: f64,
    /// Last synchronisation error ‖BΣ − I‖_F (´dec:posterior:recomputation-trigger´).
    pub last_sync_error: f64,
    /// Clean high-error recomputations that shortened the interval
    /// (´dec:posterior:adaptive-cadence´).
    #[serde(default)]
    pub sync_error_shortenings: u64,
    /// Total Cholesky recomputes, an aggregate counter reported by health
    /// (´dec:health:concrete-trackers´).
    #[serde(default)]
    pub total_recomputes: u64,
    /// Visits that measured and rebuilt nothing
    /// (´dec:posterior:recomputation-trigger´).
    ///
    /// A checkpoint written before the cadence measured before rebuilding
    /// carries none, and restores as nothing. That reading is honest rather
    /// than lossy: under the layout that wrote it every visit was a rebuild,
    /// so the count of visits that rebuilt nothing was zero.
    #[serde(default)]
    pub measurements: u64,
    /// Of which repaired the spectrum, an aggregate counter of the spectral
    /// floor's work (´dec:posterior:spectral-floor´).
    ///
    /// The slot is the one the retired repair cascade's count occupied. The
    /// payload's encoding is positional rather than named, so the rename
    /// moves no byte and the layout generation does not turn on it
    /// (´dec:durability:structural-compatibility´); what the slot means has
    /// moved with the repair it counts, and the meaning it keeps — this model's
    /// precision needed help — is the one it always had.
    #[serde(default)]
    pub floored_rebuilds: u64,
    /// Of which hit the cascade terminus (´dec:posterior:repair-cascade´).
    #[serde(default)]
    pub cascade_terminus_events: u64,
    /// Dimensions at the replenishment floor (´req:gaussian:prior-replenishment-floor´).
    #[serde(default)]
    pub last_dimensions_at_floor: usize,
    /// Rebuilds that were needed and left the model no better, or that the
    /// factorisation refused — the alarm
    /// (´dec:posterior:measured-adoption´).
    ///
    /// The slot is the one the declined-rebuild count occupied, and the
    /// encoding is positional, so the rename moves no byte on its own. What
    /// moved is the meaning, and it moved because the flow around it did: a
    /// rebuild now runs only where a measurement found drift worth acting on,
    /// so a decline is a condition rather than a rate
    /// (´rep:assayer:declined-rebuild-cadence´).
    #[serde(default)]
    pub alarms: u64,
    /// The record the last visit wrote, and the baseline every arm of the
    /// per-label check is relative to (´dec:posterior:recomputation-trigger´).
    ///
    /// A checkpoint written before the baseline existed carries none. Such a
    /// checkpoint restores with the baseline absent, which is the same state a
    /// fresh model is in: the per-label check falls back to its counter, the
    /// update applies no spectral increment, and the first visit after the
    /// restore establishes the record. Nothing is invented to fill the gap,
    /// because a baseline is a measurement and there is none to report.
    #[serde(default)]
    pub baseline: Option<crate::model::recompute::RecomputeBaseline>,
    /// The identity-shaped share of the precision matrix the spectral floor
    /// has put there (´dec:posterior:spectral-floor´). Absent in a checkpoint
    /// written before it existed, and restored as nothing.
    #[serde(default)]
    pub spectral_floor_mass: f64,
    /// The coordinate-shaped share the replenishment clamp has put there, one
    /// entry per coordinate (´req:gaussian:prior-replenishment-floor´). Absent
    /// in a checkpoint written before it existed, and restored as nothing —
    /// which the restore widens to the model's own width.
    #[serde(default)]
    pub clamp_mass: Vec<f64>,
}

/// Per-axis model state for checkpoint.
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckpointAxisState {
    /// The axis model state.
    pub model: CheckpointModelState,
    /// Condition number estimate for this axis.
    pub kappa_a: f64,
    /// Per-axis label-indexed forgetting rate (´tab:risk:forgetting-rates´).
    ///
    /// Default: 0.9998 (`GAMMA_LABEL_SISTER`).
    #[serde(default = "default_gamma_axis")]
    pub gamma: f64,
    /// Whether this axis contributes per-Sentinel spatial ledger features.
    pub spatial: bool,
    /// Human-readable axis name, unique across registered axes
    /// (´schema:registry:axis-record´).
    pub name: String,
    /// What the axis measures, in the host's own words
    /// (´schema:registry:axis-record´).
    pub description: String,
    /// Which labels this axis trains on (´def:axis:training-target´).
    pub eligibility: crate::types::OutcomeEligibility,
}

/// Backward-compatible default for `CheckpointAxisState::gamma`.
const fn default_gamma_axis() -> f64 {
    0.9998
}

/// The cold prior-mass ramp's progress, carried whole
/// (´dec:durability:ramp-resumption´).
///
/// The base the shares are retired against travels with the sample being
/// mixed into it, so a resumed ramp publishes what an uninterrupted one would
/// have published at the same count. A checkpoint that carried the published
/// moments and nothing behind them would leave a restore only two answers,
/// both wrong: restart the ramp, or treat a partial transition as finished.
///
/// The horizon is deliberately absent. It is read from the configuration in
/// force at restore, so there is never a second horizon authority to disagree
/// with it (´alg:standardisation:batch-initialisation´).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ColdRampPayload {
    /// The phase at the moment of writing. Recomputed against the configured
    /// horizon on restore; carried so a reader of the file can see it.
    pub phase: crate::feature::standardisation::StandardisationPhase,
    /// Accepted observations so far — the whole of the ramp's position.
    pub accepted: usize,
    /// The moments each retired share is taken against.
    pub base_means: Vec<f64>,
    /// The base variances, alongside `base_means`.
    pub base_variances: Vec<f64>,
    /// Running mean of the accepted sample.
    pub running_mean: Vec<f64>,
    /// Running second moment of the accepted sample.
    pub running_m2: Vec<f64>,
    /// The layout the base and the sample were gathered under.
    pub layout_generation: u64,
}

/// State held by the model owner beside the working copy, captured
/// whole into every checkpoint (´dec:durability:checkpoint-journal´).
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct OwnerCheckpointState {
    /// Calibration buffer, rows and order intact (´constr:platt:buffer´).
    pub calibration_buffer: crate::risk::calibration::CalibrationBuffer,
    /// Platt calibration convergence tracker.
    pub platt_tracker: crate::health::PlattConvergenceTracker,
    /// Per-dimension identity convergence trackers.
    pub identity_trackers: Vec<(DimensionId, crate::health::IdentityConvergenceTracker)>,
    /// The model owner's monotonic label counter.
    pub label_count: u64,
    /// Health counters for the published summary.
    pub health_counters: crate::owner::label_path::HealthCounters,
}

/// Full checkpoint payload.
///
/// Contains all state needed to reconstruct a `WorkingCopy` without
/// any Cholesky recomputation.
#[derive(Debug, Serialize, Deserialize)]
pub struct CheckpointPayload {
    /// Operational model (V) state.
    pub operational: CheckpointModelState,
    /// Sister model state.
    pub sister: CheckpointModelState,
    /// Anchor model state.
    pub anchor: CheckpointModelState,
    /// Per-axis outcome model states, keyed by axis ID value.
    pub outcome_models: Vec<(u32, CheckpointAxisState)>,

    // ─── Calibration scalars ─────────────────────────────────────────
    /// Condition number estimate for sister model.
    pub kappa_sister: f64,
    /// Condition number estimate for anchor model.
    pub kappa_anchor: f64,
    /// Global positive-class prior.
    pub p_positive_global: f64,
    /// Eligible positive-class prior.
    pub p_positive_eligible: f64,
    /// Condition number estimate for operational model.
    pub kappa_v: f64,

    // ─── Sequencing ──────────────────────────────────────────────────
    /// Last processed label sequence number.
    pub last_processed_label_seq: u64,
    /// Timestamp of the checkpoint (for time-decay on restore).
    pub checkpoint_timestamp: PersistentTimestamp,

    // ─── Layer 2+ data structures the whole-state checkpoint carries
    // ─── (´dec:durability:checkpoint-journal´) ─────────────────────────
    /// Per-Sentinel outcome ledger state.
    pub ledger_state: Vec<(SentinelId, SentinelLedgerPayload)>,
    /// Per-dimension identity state (G-V graph, competitive set, cell outcomes).
    pub identity_state: Vec<(DimensionId, IdentityDimensionPayload)>,
    /// Structural dimension map.
    pub dimension_map: DimensionMap,

    // ─── Structural metadata (´cav:construction:schema-fixity´) ──────
    /// Signal schema at construction time. Used for structural mismatch
    /// detection on restore. Empty for checkpoints written before v4.
    #[serde(default)]
    pub signal_schema: Vec<SignalDeclaration>,
    /// Interaction templates at construction time. Used for structural
    /// mismatch detection on restore. Empty for pre-v4 checkpoints.
    #[serde(default)]
    pub interaction_templates: Vec<InteractionTemplate>,

    // ─── Health/convergence state (´dec:health:concrete-trackers´) ───
    /// Concordance tracker state captured from the Assayer at checkpoint time.
    pub concordance_state: crate::health::ConcordanceCheckpointState,

    /// Approximation accounting accumulated across lifecycle marginalisations.
    /// The counters belong to the deployment and therefore cross the same
    /// whole-state boundary as the models they describe
    /// (´dec:durability:checkpoint-journal´).
    pub marginalisation_errors: crate::model::marginalise::MarginalisationErrorLedger,

    // ─── Hibernation archive (´alg:registry:hibernation´) ────────────
    /// The parameter blocks hibernating deregistrations kept, keyed by the
    /// identifier each was deregistered under.
    ///
    /// The archive is the Assayer's own custody and never leaves it, so its
    /// persistence is this payload and nothing else. A restart that dropped
    /// it would leave the archive's expiry the only thing a host could reason
    /// about while the records themselves had already gone.
    pub hibernation: crate::model::hibernation::HibernationArchive,

    // ─── Drift state (´tab:monitoring:drift-resets´) ─────────────────
    /// Per-model drift accumulators. A restart is not among the table's
    /// discard triggers, so the evidence crosses the checkpoint.
    pub drift_state: Vec<(crate::types::ModelId, crate::health::DriftState)>,

    // ─── Standardisation statistics (´req:standardisation:timing´) ───
    /// Running feature means, indexed against `dimension_map`.
    pub feature_means: Vec<f64>,
    /// Running feature variances, indexed against `dimension_map`.
    pub feature_variances: Vec<f64>,
    /// Per-feature class assignments.
    pub feature_classes: Vec<crate::feature::standardisation::FeatureClass>,
    /// The cold prior-mass ramp's progress
    /// step 7 of (´alg:standardisation:batch-initialisation´).
    pub cold_ramp: ColdRampPayload,

    // ─── Owner-held state beside the working copy ────────────────────
    /// Calibration buffer, trackers and counters the model owner holds
    /// outside the `WorkingCopy`. Filled at the write site; default from
    /// the working-copy conversion alone.
    pub owner_state: OwnerCheckpointState,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Layer 2 Checkpoint Payload Types — data structures carried whole
// (´dec:durability:checkpoint-journal´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Construction-time structural metadata captured in every checkpoint
/// for mismatch detection on restore (´cav:construction:schema-fixity´).
///
/// These properties determine the feature vector layout and must
/// match between the checkpoint and the rebuilding Assayer.
#[derive(Debug, Clone, Default)]
pub struct StructuralMetadata {
    /// Signal schema declared at construction time.
    pub signal_schema: Vec<SignalDeclaration>,
    /// Interaction templates declared at construction time.
    pub interaction_templates: Vec<InteractionTemplate>,
}

/// Serialisable per-Sentinel ledger state for checkpoint persistence.
#[derive(Debug, Serialize, Deserialize)]
pub struct SentinelLedgerPayload {
    /// The serialised Sentinel ledger.
    pub ledger: SentinelLedger,
}

/// Serialisable per-dimension identity state for checkpoint persistence.
#[derive(Debug, Serialize, Deserialize)]
pub struct IdentityDimensionPayload {
    /// G-V graph snapshot (terminal + transition observations).
    pub graph_snapshot: IdentityGraphSnapshot,
    /// Competitive cells at checkpoint time.
    pub competitive_cells: Vec<CompetitiveCellId>,
    /// Per-cell outcome state.
    pub cell_outcome_state: Vec<(CompetitiveCellId, CellOutcomeState)>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Writer
// ═══════════════════════════════════════════════════════════════════════════════

/// Writes an atomic checkpoint to `path`.
///
/// Protocol: write to `{path}.new` → `sync_all()` → `rename()` → sync the
/// directory the rename published the name in.
///
/// The fourth step is what lets a caller act on the return value. Syncing the
/// temporary file makes its *contents* durable; the name is an entry in the
/// parent directory, and until that directory is synced the entry is visible
/// but not persisted. The model owner truncates the journal as soon as this
/// returns (´dec:durability:checkpoint-journal´), so a success that promised
/// only visibility would let a power loss keep the truncation and lose the
/// rename — leaving the labels this checkpoint absorbed in neither file.
/// Which platforms carry the fourth step, and what the others do instead, is
/// stated on [`rename_durably`](super::rename_durably).
///
/// Power loss is not an event a test can stage, so the guarantee is argued
/// here rather than witnessed; the round-trip tests cover the protocol's other
/// half, that what is written reads back.
///
/// # Errors
///
/// Returns `io::Error` on any filesystem failure.
pub fn write_checkpoint(path: &Path, payload: &CheckpointPayload) -> Result<(), io::Error> {
    let tmp_path = path.with_extension("new");

    // Serialise payload via postcard.
    let payload_bytes = crate::serde_codec::serialize(payload).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

    // Compute CRC32 of payload.
    let crc = crc32fast::hash(&payload_bytes);

    // Build the complete file contents.
    let mut buf = Vec::with_capacity(HEADER_SIZE + payload_bytes.len());
    buf.extend_from_slice(&CHECKPOINT_MAGIC);
    buf.extend_from_slice(&CHECKPOINT_FORMAT_VERSION.to_le_bytes());
    buf.extend_from_slice(&crc.to_le_bytes());
    buf.extend_from_slice(&payload_bytes);

    // Write atomically: temp file → sync → rename → sync the directory.
    {
        let mut file = fs::File::create(&tmp_path)?;
        file.write_all(&buf)?;
        file.sync_all()?;
    }

    super::rename_durably(&tmp_path, path)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Reader
// ═══════════════════════════════════════════════════════════════════════════════

/// Reads and validates a checkpoint from `path`.
///
/// Validates: magic bytes, format version, CRC32, then deserialises.
///
/// # Errors
///
/// Returns `CheckpointError` on any validation or deserialisation failure.
pub fn read_checkpoint(path: &Path) -> Result<CheckpointPayload, CheckpointError> {
    let data = fs::read(path)?;

    if data.len() < HEADER_SIZE {
        return Err(CheckpointError::Io(io::Error::new(
            io::ErrorKind::UnexpectedEof,
            format!("checkpoint file too small: {} bytes", data.len()),
        )));
    }

    // Validate magic.
    let mut magic = [0u8; 4];
    magic.copy_from_slice(&data[0..4]);
    if magic != CHECKPOINT_MAGIC {
        return Err(CheckpointError::InvalidMagic { found: magic });
    }

    // Validate format version.
    let version = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);
    if version != CHECKPOINT_FORMAT_VERSION {
        return Err(CheckpointError::VersionMismatch {
            found: version,
            expected: CHECKPOINT_FORMAT_VERSION,
        });
    }

    // Validate CRC32.
    let expected_crc = u32::from_le_bytes([data[8], data[9], data[10], data[11]]);
    let payload_bytes = &data[HEADER_SIZE..];
    let actual_crc = crc32fast::hash(payload_bytes);
    if expected_crc != actual_crc {
        return Err(CheckpointError::IntegrityFailed {
            expected: expected_crc,
            actual: actual_crc,
        });
    }

    // Deserialise.
    crate::serde_codec::deserialize(payload_bytes).map_err(CheckpointError::Deserialise)
}
