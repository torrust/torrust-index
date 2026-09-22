// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Pre-seeding support for bootstrapping the Assayer from historical data.
//!
//! The `pre_seed()` method feeds historical label data through the normal
//! label pipeline, allowing the model to learn from past observations
//! before the system goes live.
//!
//! # Cross-References
//!
//! - Pre-seeding is a post-construction step (´dec:construction:post-seeding´)
//! - Preloaded outcomes travel the ordinary label path rather than a shortcut
//!   (´alg:host:pre-seeding´)

use std::collections::HashMap;

use crossbeam_channel::TrySendError;

use crate::Assayer;
use crate::error::LabelError;
use crate::owner::commands::{LabelData, ModelOwnerCommand};
use crate::types::{Action, AssessmentId, ChannelId, EntityKey, OutcomeAxisId};

// ═══════════════════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════════════════

/// A single pre-seed entry containing minimal label data.
///
/// Pre-seed entries carry enough information to construct a synthetic
/// `PendingAssessment` and feed the label through the normal pipeline.
///
/// The synthetic entry is built against the declared schema and the
/// registered Sentinels (´alg:host:pre-seeding´): each registered
/// Sentinel holds an occupied, zero-featured slot — active and silent —
/// the entry's key is encoded on every registered identity dimension
/// with the current competitive state frozen beside it, and the signal
/// block is encoded from the schema at the entity's cached persistent
/// values.
#[derive(Clone, Debug)]
pub struct PreSeedEntry {
    /// Entity key for identity and signal feature computation.
    pub entity: EntityKey,
    // TODO(2026-05-17) ´todo:code:remove-this-field-core-pre-seeding´: Remove this field; Core pre-seeding
    // has no channel dimension after channel policy moved to derivation
    // (´dec:surface:no-channel-input´).
    /// Channel to associate with this pre-seed label.
    pub channel: ChannelId,
    /// The action that was actually taken.
    pub action_taken: Action,
    /// Observed valence (positive = spam).
    pub valence: f64,
    /// Per-axis outcome values.
    pub outcomes: HashMap<OutcomeAxisId, f64>,
    /// Whether this is ground truth.
    pub ground_truth: bool,
}

/// Result of a `pre_seed()` call.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct PreSeedResult {
    /// Number of entries successfully processed.
    pub processed: usize,
    /// Number of entries that failed (submitted but rejected by
    /// the label pipeline). Currently always 0 because failures
    /// are propagated as `LabelError`.
    pub failed: usize,
    /// Global P+ estimate after all pre-seed labels have been
    /// processed. Reads the published snapshot, so reflects the
    /// state at the latest model-owner publication.
    pub final_p_positive: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Assayer::pre_seed()
// ═══════════════════════════════════════════════════════════════════════════════

impl Assayer {
    /// Pre-seeds the model with historical label data.
    ///
    /// For each entry, a synthetic `PendingAssessment` is synthesised
    /// against the declared schema and the registered Sentinels — see
    /// [`PreSeedEntry`] for what the entry carries — at prior-level
    /// risk, then fed through the normal `label()` path. After all
    /// entries are submitted, the method blocks until the model-owner
    /// has processed them all.
    ///
    /// # Errors
    ///
    /// Returns `LabelError` if any label submission fails. Entries
    /// processed before the failure are not rolled back — they have
    /// already been submitted to the label channel and will be
    /// applied by the model owner asynchronously.  The caller
    /// cannot determine exactly how many were applied at the time
    /// the error is returned; `flush_label_channel()` is **not**
    /// called on the error path.
    ///
    /// # Cross-References
    ///
    /// - Pre-seeding is a post-construction step (´dec:construction:post-seeding´)
    pub fn pre_seed(&self, entries: &[PreSeedEntry]) -> Result<PreSeedResult, LabelError> {
        let mut processed = 0;

        // The layout the synthesis freezes against: the published
        // dimension map's signal width and the registered axes.
        let (sig_len, outcome_axis_ids) = {
            let snapshot = self.shared.published.load();
            let axis_ids: Vec<crate::types::OutcomeAxisId> = snapshot.outcome_models.keys().copied().collect();
            (snapshot.dimension_map.sig_range.len(), axis_ids)
        };

        for entry in entries {
            // Allocate an assessment ID for this pre-seed entry
            let assessment_id = self.next_assessment_id();

            // Synthesise the entry against the declared schema and the
            // registered Sentinels (´alg:host:pre-seeding´).
            let pending =
                crate::assessment::synthesise_preseed_pending(assessment_id, &entry.entity, self, sig_len, &outcome_axis_ids);
            self.pending_buffer.insert(pending);

            // Feed through normal label pipeline
            let label_data = entry_to_label_data(assessment_id, entry);
            self.label(label_data)?;
            processed += 1;
        }

        // Block until model owner processes all entries
        self.flush_label_channel()?;

        // Read p_positive from the snapshot published after flushing.
        let final_p_positive = self.shared.published.load().p_positive_global;

        Ok(PreSeedResult {
            processed,
            failed: 0,
            final_p_positive,
        })
    }

    /// Sends a synchronisation marker through the command channel
    /// and waits for the model owner to acknowledge it.
    ///
    /// # What the acknowledgement covers
    ///
    /// Two queues, and only two. Every command submitted before this call —
    /// lifecycle submissions included — has been applied, because the command
    /// channel is first-in-first-out and the marker rides it. Every label
    /// submitted before this call has been applied, because the marker's own
    /// handler drains the label channel before it acknowledges.
    ///
    /// # What it does not cover
    ///
    /// Cold-ramp observations. They ride a third channel of their own, which
    /// the steward reaches only after it has emptied the command queue
    /// (´dec:concurrency:channel-preemption´) and which the marker's handler
    /// does not touch. So the marker is reached, and answered, with
    /// observations still queued: this
    /// call orders nothing about the standardisation ramp, and a caller that
    /// reads the accepted count or the phase straight afterwards is reading
    /// whatever the steward happened to have got through. The name says
    /// labels because labels are what it flushes.
    ///
    /// [`flush_observation_channel`](Self::flush_observation_channel) is the
    /// barrier for that queue. The two are separate calls because they are
    /// separate queues, and no ordering of one marker against the other
    /// would make either cover both.
    pub(crate) fn flush_label_channel(&self) -> Result<(), LabelError> {
        let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);

        // Use a checkpoint request as a synchronisation barrier.
        // Checkpoints are processed after all pending commands and labels
        // have been drained, because structure preempts observation
        // (´dec:concurrency:channel-preemption´).
        let request = crate::owner::commands::CheckpointRequest {
            completion: Some(ack_tx),
        };

        self.command_tx
            .try_send(ModelOwnerCommand::Checkpoint(request))
            .map_err(|e| match e {
                TrySendError::Full(_) => LabelError::ChannelFull,
                TrySendError::Disconnected(_) => LabelError::ModelOwnerShutdown,
            })?;

        // Block until checkpoint completes (label channel drained first)
        let _checkpoint_result = ack_rx.recv().map_err(|_| LabelError::ModelOwnerShutdown)?;

        Ok(())
    }

    /// Sends an observation barrier through the command channel and waits for
    /// the model owner to acknowledge it.
    ///
    /// The sibling of [`flush_label_channel`](Self::flush_label_channel) for
    /// the queue that call leaves alone: on return, every cold-ramp
    /// observation offered before this call has been applied and its advance
    /// published, so the accepted count and the standardisation phase read
    /// afterwards are the ones those offers add up to
    /// (´alg:standardisation:batch-initialisation´).
    ///
    /// What it does not do is make the ramp advance. An offer refused by a
    /// full queue was never queued and there is nothing here to wait for; an
    /// arrival under a superseded layout is still refused by the steward.
    /// The barrier settles the queue, and the queue is all it settles.
    ///
    /// # Errors
    ///
    /// Returns [`LabelError::ChannelFull`] if the command channel cannot take
    /// the barrier, and [`LabelError::ModelOwnerShutdown`] if the model owner
    /// is gone — either before the barrier is sent, or after, in which case
    /// the acknowledgement's sender is dropped with the owner and the wait
    /// ends in that error rather than hanging.
    /// The barrier is the harness's, and it is gated with the harness: the
    /// crate's own tests and the integration-test world are its only callers,
    /// and a host reaches the same settlement through the public surface.
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) fn flush_observation_channel(&self) -> Result<(), LabelError> {
        let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);

        let request = crate::owner::commands::ObservationBarrierRequest { completion: ack_tx };

        self.command_tx
            .try_send(ModelOwnerCommand::ObservationBarrier(request))
            .map_err(|e| match e {
                TrySendError::Full(_) => LabelError::ChannelFull,
                TrySendError::Disconnected(_) => LabelError::ModelOwnerShutdown,
            })?;

        ack_rx.recv().map_err(|_| LabelError::ModelOwnerShutdown)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Internal Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Converts a `PreSeedEntry` to `LabelData` for the label pipeline.
fn entry_to_label_data(assessment_id: AssessmentId, entry: &PreSeedEntry) -> LabelData {
    LabelData {
        assessment_id,
        action_taken: entry.action_taken,
        valence: entry.valence,
        outcomes: entry.outcomes.clone(),
        ground_truth: entry.ground_truth,
    }
}
