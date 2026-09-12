// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Crash recovery: checkpoint restore + journal replay.
//!
//! Recovery follows this protocol (´dec:durability:checkpoint-journal´):
//!
//! 1. Load checkpoint → validate structural compatibility.
//! 2. Apply bulk time-indexed decay from checkpoint timestamp to now.
//! 3. Reset `last_updated` fields.
//! 4. Return the working copy + journal entries for replay.
//!
//! If any step fails, return `None` — the caller falls back to cold start.
//!
//! # Replay Invariant
//!
//! After restore-decay, replayed labels see Δt ≈ microseconds
//! (processing time), not the full downtime gap
//! (´dec:durability:decay-once´). This is because the bulk decay on
//! restore already accounts for the elapsed time.
//!
//! # Cross-References
//!
//! - (´dec:durability:checkpoint-journal´) — the checkpoint and journal replay proceeds from
//! - (´dec:durability:decay-once´) — elapsed decay applied exactly once on restore
//! - (´cav:construction:schema-fixity´) — the construction-time declarations a restore is checked against
//! - (´dec:durability:structural-compatibility´) — what a structural mismatch does

use std::path::Path;

use tracing::{info, warn};

use super::checkpoint::{self, CheckpointPayload, IdentityDimensionPayload, SentinelLedgerPayload};
use super::journal::{self, JournalEntry};
use crate::config::types::AssayerConfig;
use crate::feature::interaction::InteractionTemplate;
use crate::numerics::decay_factor;
use crate::signal::SignalDeclaration;
use crate::snapshot::working::WorkingCopy;
use crate::types::{DimensionId, PersistentTimestamp, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Recovery Result
// ═══════════════════════════════════════════════════════════════════════════════

/// Successful recovery output.
#[allow(dead_code)]
pub struct RecoveryResult {
    /// The restored and time-decayed working copy.
    pub working: WorkingCopy,
    /// Journal entries with `seq > working.last_processed_label_seq`
    /// that need to be replayed.
    pub replay_entries: Vec<JournalEntry>,
    /// Restored per-Sentinel ledger state (time-decayed).
    pub ledger_state: Vec<(SentinelId, SentinelLedgerPayload)>,
    /// Restored per-dimension identity state (cell outcomes time-decayed).
    pub identity_state: Vec<(DimensionId, IdentityDimensionPayload)>,
    /// Restored concordance tracker state.
    pub concordance_state: crate::health::ConcordanceCheckpointState,
    /// Restored owner-held state (calibration buffer, trackers, counters).
    pub owner_state: super::checkpoint::OwnerCheckpointState,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Attempt Restore
// ═══════════════════════════════════════════════════════════════════════════════

/// Attempts to restore from checkpoint and journal.
///
/// Returns `Ok(Some(RecoveryResult))` where the state was restored, and
/// `Ok(None)` where there is no state to restore from and the caller should
/// cold-start.
///
/// # Two dispositions, and why they are not one
///
/// A checkpoint that cannot be read, or one whose structure does not match
/// the instance being built, yields `Ok(None)` and the caller cold-starts.
/// That is the disposition those two cases have always had and it is the
/// right one for them: an absent or unreadable file is a deployment with no
/// state to resume, and a structural mismatch is a legitimate change to the
/// schema or the templates that the restore is explicitly not asked to
/// migrate across (´dec:durability:structural-compatibility´). In both, cold
/// starting is what the host was going to get anyway.
///
/// A checkpoint whose stored precision matrix is not positive definite is a
/// different kind of thing, and it fails the call. The file was read, its
/// structure agreed, and what it carries is a matrix the model's invariant
/// does not admit — corruption rather than a configuration change. Cold
/// starting on it would discard every learned parameter silently, at the one
/// moment a host could still act: an earlier checkpoint may be intact, the
/// journal may be replayable, and the host is the only party that knows which
/// of those it has. So the refusal travels rather than being absorbed
/// (´dec:posterior:cascade-never-fails´), (´dec:degradation:error-partition´).
///
/// The journal beside the checkpoint is read on the same principle. A torn
/// tail and an absent file are absorbed, and a journal of another format
/// version is discarded by design so that an upgrade is not an outage; an
/// unreadable file or a corruption before the tail is a loss of the only copy
/// of the labels acknowledged since the capture, and fails the call for the
/// reason the indefinite checkpoint does.
///
/// # Errors
///
/// Returns [`BuildError::CheckpointNotPositiveDefinite`] where the payload's
/// precision matrix for some model does not factor, and
/// [`BuildError::JournalUnreadable`] where the journal fails to read for a
/// reason the protocol does not absorb.
///
/// [`BuildError::JournalUnreadable`]: crate::error::BuildError::JournalUnreadable
///
/// # Protocol
///
/// 1. Read and validate the checkpoint file.
/// 2. Validate structural compatibility with the current config and
///    the builder's structural declarations (´cav:construction:schema-fixity´).
/// 3. Reconstruct the `WorkingCopy` from the checkpoint payload, which takes
///    a definiteness verdict on each restored precision matrix.
/// 4. Apply bulk per-component time-decay (´dec:durability:decay-once´) for the
///    time elapsed between the checkpoint's timestamp and `now`, which the
///    caller reads from the engine's clock so that both ends of the interval
///    are in one time domain (´dec:clock:two-domains´).
/// 5. Read journal entries and filter to those needing replay. A journal that
///    cannot be read fails the restore, apart from the tolerated torn tail and
///    the by-design discard of another format version.
///
/// [`BuildError::CheckpointNotPositiveDefinite`]: crate::error::BuildError::CheckpointNotPositiveDefinite
pub fn attempt_restore(
    checkpoint_path: &Path,
    journal_path: &Path,
    config: &AssayerConfig,
    signal_schema: &[SignalDeclaration],
    interaction_templates: &[InteractionTemplate],
    now: PersistentTimestamp,
) -> Result<Option<RecoveryResult>, crate::error::BuildError> {
    // Step 1: Read checkpoint.
    let payload = match checkpoint::read_checkpoint(checkpoint_path) {
        Ok(p) => {
            info!(seq = p.last_processed_label_seq, "checkpoint loaded successfully");
            p
        }
        Err(e) => {
            warn!(%e, "checkpoint restore failed; falling back to cold start");
            return Ok(None);
        }
    };

    // Step 2: Structural compatibility check (´dec:durability:structural-compatibility´).
    if let Err(reason) = validate_structural_compatibility(&payload, config, signal_schema, interaction_templates) {
        warn!(
            reason = %reason,
            "checkpoint structurally incompatible; falling back to cold start"
        );
        return Ok(None);
    }

    // Step 3: Reconstruct working copy, taking the definiteness verdict on
    // every restored precision matrix. A refusal here is not a cold start.
    let concordance_state = payload.concordance_state.clone();
    let mut working =
        WorkingCopy::from_checkpoint_payload(&payload, config.standardisation.n_init as usize, config.model.lambda_prior)
            .inspect_err(|e| {
                warn!(%e, "checkpoint refused; construction fails rather than discarding the checkpointed state");
            })?;
    let owner_state = payload.owner_state;

    // Step 4: Apply bulk per-component time-decay (´dec:durability:decay-once´).
    // The present comes from the caller, which holds the engine's clock. The
    // checkpoint's own timestamp was written from that same clock, so taking a
    // wall reading here would subtract two different time domains: a
    // checkpoint written under an injected clock would restore with the gap
    // between virtual time and wall time, clamped at the elapsed ceiling, and
    // every stored average would be aged by a downtime that never happened
    // (´dec:clock:two-domains´).
    let temporal = &config.temporal;
    let dt_hours = now.hours_since(&payload.checkpoint_timestamp);

    if dt_hours > 0.0 {
        // 4a. Core models: γ_t = gamma_t_core
        let model_factor = decay_factor(temporal.gamma_t_core, dt_hours);
        info!(dt_hours, model_factor, "applying bulk time-decay for downtime");
        working.apply_time_decay(model_factor);
    }

    // 4b. Ledger EWMAs: γ_{t,L} = gamma_t_ledger
    let mut ledger_state = payload.ledger_state;
    if dt_hours > 0.0 {
        let ledger_factor = decay_factor(temporal.gamma_t_ledger, dt_hours);
        for (_sentinel_id, lp) in &mut ledger_state {
            for entry in lp.ledger.entries_mut().values_mut() {
                entry.ewma_bad_rate *= ledger_factor;
                entry.compressed_valence_ewma *= ledger_factor;
                entry.raw_valence_ewma *= ledger_factor;
                for (_, c, r) in &mut entry.per_axis {
                    *c *= ledger_factor;
                    *r *= ledger_factor;
                }
                entry.last_updated = now;
            }
        }
    }

    // 4c. Identity cell outcome EWMAs: γ_{t,id} = gamma_t_identity
    let mut identity_state = payload.identity_state;
    if dt_hours > 0.0 {
        let identity_factor = decay_factor(temporal.gamma_t_identity, dt_hours);
        for (_dim_id, ip) in &mut identity_state {
            for (_cell_id, cell) in &mut ip.cell_outcome_state {
                cell.adverse_rate_ewma *= identity_factor;
                cell.compressed_valence_ewma *= identity_factor;
                cell.raw_valence_ewma *= identity_factor;
                for v in cell.per_axis_compressed.values_mut() {
                    *v *= identity_factor;
                }
                for v in cell.per_axis_raw.values_mut() {
                    *v *= identity_factor;
                }
                cell.last_updated = now;
            }
        }
    }

    // Step 5: Read and filter journal entries.
    //
    // A journal that cannot be read is a build failure and not a quiet cold
    // start, because the entries past the checkpoint's mark are the only copy
    // of every label acknowledged since the capture. The reader has already
    // absorbed the two failures that are not losses: an absent journal reads
    // as no entries, and a record torn by a crash during its own append is
    // dropped from the tail, which is the single label the append protocol
    // allows to be lost. What is left — an unreadable file, or a corruption
    // before the tail that makes every record after it unfindable — is a loss,
    // and construction says so rather than reporting a successful start over a
    // model that is missing evidence it was told was durable
    // (´dec:degradation:error-partition´).
    //
    // A journal of another format version is the third case and keeps its own
    // answer, which is a discard by design: an upgraded binary meeting the
    // previous deployment's journal cold-starts rather than refusing to run,
    // and a headerless file is that same previous layout
    // (´dec:durability:checkpoint-journal´). Refusing the build there would
    // turn a supported upgrade into an outage.
    let journal_entries = match journal::read_journal(journal_path) {
        Ok(entries) => {
            info!(total = entries.len(), "journal entries read");
            entries
        }
        Err(e @ journal::JournalError::VersionMismatch { .. }) => {
            warn!(%e, "journal of another format version; discarded by design, proceeding without replay");
            Vec::new()
        }
        Err(e) => {
            warn!(%e, "journal unreadable; construction fails rather than dropping the labels it holds");
            return Err(crate::error::BuildError::JournalUnreadable { reason: e.to_string() });
        }
    };

    let last_seq = working.last_processed_label_seq;
    let replay_entries: Vec<JournalEntry> = journal_entries.into_iter().filter(|entry| entry.seq > last_seq).collect();

    info!(
        replay_count = replay_entries.len(),
        last_checkpoint_seq = last_seq,
        "recovery complete"
    );

    Ok(Some(RecoveryResult {
        working,
        replay_entries,
        ledger_state,
        identity_state,
        concordance_state,
        owner_state,
    }))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Structural Compatibility
// ═══════════════════════════════════════════════════════════════════════════════

/// Validates that the checkpoint is structurally compatible with the
/// current configuration and builder declarations.
///
/// A checkpoint is structurally compatible iff the construction-time
/// immutable properties match (´dec:durability:structural-compatibility´):
///
/// | Property              | Match criterion                     |
/// | --------------------- | ----------------------------------- |
/// | Signal schema         | Same names, same shapes, same order |
/// | Interaction templates | Same count, same types              |
/// | Anchor dimension      | Same `P_A`                          |
///
/// Numerical parameter changes across restarts are permitted.
/// Runtime-registered entities (Sentinels, axes, dimensions) are NOT
/// checked — they are restored from the checkpoint.
fn validate_structural_compatibility(
    payload: &CheckpointPayload,
    _config: &AssayerConfig,
    signal_schema: &[SignalDeclaration],
    interaction_templates: &[InteractionTemplate],
) -> Result<(), String> {
    // Check anchor dimension — must match the fixed P_A constant (´dec:substrate:anchor-invariant´).
    let checkpoint_anchor_p = payload.anchor.parameters.p;
    let config_anchor_p = crate::config::types::P_A;
    if checkpoint_anchor_p != config_anchor_p {
        return Err(format!(
            "anchor dimension mismatch: checkpoint has p_a={checkpoint_anchor_p}, config has p_a={config_anchor_p}"
        ));
    }

    // Signal schema: same count, same names, same shapes, same order.
    // Only checked when the checkpoint carries schema metadata (v4+).
    if !payload.signal_schema.is_empty() || !signal_schema.is_empty() {
        if payload.signal_schema.len() != signal_schema.len() {
            return Err(format!(
                "signal schema count mismatch: checkpoint has {}, current has {}",
                payload.signal_schema.len(),
                signal_schema.len(),
            ));
        }
        for (i, (ck, cur)) in payload.signal_schema.iter().zip(signal_schema).enumerate() {
            if ck.name != cur.name || ck.shape != cur.shape {
                return Err(format!(
                    "signal schema mismatch at position {i}: checkpoint has {:?}/{:?}, current has {:?}/{:?}",
                    ck.name, ck.shape, cur.name, cur.shape,
                ));
            }
        }
    }

    // Interaction templates: same count, same types.
    // Only checked when the checkpoint carries template metadata (v4+).
    if !payload.interaction_templates.is_empty() || !interaction_templates.is_empty() {
        if payload.interaction_templates.len() != interaction_templates.len() {
            return Err(format!(
                "interaction template count mismatch: checkpoint has {}, current has {}",
                payload.interaction_templates.len(),
                interaction_templates.len(),
            ));
        }
        for (i, (ck, cur)) in payload.interaction_templates.iter().zip(interaction_templates).enumerate() {
            if ck != cur {
                return Err(format!(
                    "interaction template mismatch at position {i}: checkpoint has {ck:?}, current has {cur:?}",
                ));
            }
        }
    }

    Ok(())
}
