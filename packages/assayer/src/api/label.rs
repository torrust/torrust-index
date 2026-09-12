// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`sanitise_nan_valence`] | labelling | A valence that is not a number is repaired to zero at the boundary rather than refused, and the repair is counted: the sanitisation record beside the payload reports one valence repaired. The label still reaches the model owner, carrying no signal instead of poisoning every scalar downstream — and because zero is also an ordinary valence, the count is the only trace the repair leaves (´dec:surface:sanitise-not-reject´). |
//! | [`sanitise_inf_valence`] | labelling | cites (´claim:labelling:a-non-finite-valence-is-repaired-to-zero-and-the-repair-is-counted´) |
//! | [`sanitise_nan_outcome`] | labelling | Per-axis outcomes are repaired individually and the drops are counted: the axis whose value is not finite leaves the map, its finite sibling stays, and the sanitisation record reports exactly one outcome dropped. One broken axis therefore costs the host that axis only — and visibly, since a dropped axis leaves no other trace on the label (´dec:surface:sanitise-not-reject´). |
//! | [`sanitise_normal_values_untouched`] | labelling | Repair is confined to what is actually broken: an ordinary valence and an ordinary outcome come out of the boundary bit-for-bit as they went in, and the sanitisation record reports nothing repaired. The sanitiser is a filter on non-finite values, not a transform every label pays for — and a count that rose on clean labels would make the diagnostic it feeds meaningless (´dec:surface:sanitise-not-reject´). |

//! Public `label()` method on `Assayer` and `LabelAck` response type.
//!
//! # Cross-References
//!
//! - Label submission is asynchronous (´dec:surface:async-label´)
//! - A periodic checkpoint and a self-contained journal (´dec:durability:checkpoint-journal´)
//! - Anomalous values are sanitised at the boundary rather than rejected
//!   (´dec:surface:sanitise-not-reject´)

use crossbeam_channel::TrySendError;

use crate::Assayer;
use crate::error::LabelError;
use crate::owner::commands::{LabelContext, LabelData, SequencedLabel};
use crate::types::AssessmentId;

// ═══════════════════════════════════════════════════════════════════════════════
// LabelAck
// ═══════════════════════════════════════════════════════════════════════════════

/// Acknowledgement returned by `Assayer::label()`.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct LabelAck {
    // TODO ´todo:code:reconcile-whether-the-acknowledgement´: Reconcile whether the acknowledgement
    // should expose the journal sequence number. The self-contained journal's
    // examples return `seq` (´dec:durability:checkpoint-journal´); the public
    // label contract currently returns the per-assessment identifier
    // (´dec:surface:assessment-identifier´).
    /// The assessment ID that was labelled.
    pub assessment_id: AssessmentId,
}

// ═══════════════════════════════════════════════════════════════════════════════
// NaN Sanitisation
// ═══════════════════════════════════════════════════════════════════════════════

/// What the label boundary repaired, returned beside the repaired
/// payload so the repairs are counted rather than silent
/// (´dec:surface:sanitise-not-reject´).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct LabelSanitisation {
    /// Non-finite valences repaired to zero (0 or 1 per label).
    valences_sanitised: u32,
    /// Non-finite per-axis outcomes dropped from the map.
    outcomes_dropped: u32,
}

/// Sanitises a [`LabelData`] at the API boundary
/// (´dec:surface:sanitise-not-reject´), returning what it repaired beside
/// the repaired payload.
///
/// Honours the checkpoint philosophy that partitions the two kinds of
/// failure (´dec:degradation:error-partition´): data-quality issues are
/// repaired and counted, not rejected. Hard errors are reserved
/// for structural failures (pending buffer miss, I/O failure, channel
/// failure, thread death).
///
/// - Valence: NaN/±Inf → `0.0` (treated as benign / no signal). The
///   `+Inf` case is a deliberate semantic flip, and part of the same
///   sanitisation decision (´dec:surface:sanitise-not-reject´).
/// - Outcome values: NaN/±Inf entries removed from the map. Other
///   entries pass through unchanged.
fn sanitise_label_data(mut data: LabelData) -> (LabelData, LabelSanitisation) {
    let mut repaired = LabelSanitisation::default();

    // Valence: NaN/Inf → 0.0
    if !data.valence.is_finite() {
        data.valence = 0.0;
        repaired.valences_sanitised = 1;
    }

    // Outcome values: remove NaN/Inf entries
    let before = data.outcomes.len();
    data.outcomes.retain(|_, v| v.is_finite());
    repaired.outcomes_dropped = u32::try_from(before - data.outcomes.len()).unwrap_or(u32::MAX);

    (data, repaired)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Assayer::label()
// ═══════════════════════════════════════════════════════════════════════════════

impl Assayer {
    /// Submits a label (ground-truth outcome) for a previous assessment.
    ///
    /// The label is matched to its assessment via the pending buffer,
    /// optionally journaled for crash recovery, and enqueued for model
    /// update on the model-owner thread.
    ///
    /// # NaN Sanitisation
    ///
    /// - Valence NaN/Inf → 0.0
    /// - Outcome values with NaN/Inf are removed from the map
    /// # Panics
    ///
    /// Panics if the journal mutex is poisoned (indicates a prior panic
    /// in the journal writer).
    ///
    /// # Errors
    ///
    /// - [`LabelError::AssessmentNotFound`] — assessment ID not in pending buffer
    /// - [`LabelError::JournalFailed`] — journal write failed (safe to retry)
    /// - [`LabelError::JournaledButNotEnqueued`] — journaled but channel full
    /// - [`LabelError::ChannelFull`] — channel full, no journal
    /// - [`LabelError::ModelOwnerShutdown`] — model owner disconnected
    /// - [`LabelError::LabelPathStopped`] — a failed revert stopped the label
    ///   path; nothing was consumed and nothing was journaled
    ///
    /// # Cross-References
    ///
    /// - Label submission is asynchronous (´dec:surface:async-label´)
    /// - A periodic checkpoint and a self-contained journal (´dec:durability:checkpoint-journal´)
    pub fn label(&self, data: LabelData) -> Result<LabelAck, LabelError> {
        let assessment_id = data.assessment_id;

        // Step 0: a stopped label path refuses before anything is consumed.
        //
        // The check stands ahead of the pending buffer's removal rather than
        // beside the enqueue, where the shutdown refusal is taken. Both are
        // states of the owner rather than of the label, but the shutdown is
        // discovered at the channel and can only be discovered there, while
        // this one is known before the call does any work — and taking it here
        // is what keeps the refusal free of consequences: the pending entry
        // survives to its own expiry, the journal takes nothing, and the
        // journal therefore does not grow with labels that replay into an
        // engine which will not apply them (´dec:durability:checkpoint-journal´).
        if let Some(stop) = self.shared.label_path_stop.load_full() {
            return Err(LabelError::LabelPathStopped((*stop).clone()));
        }

        // The host's feedback arrives here and nowhere else, so this is where
        // the second stage of the feedback latency ends and the third begins
        // (´def:monitoring:feedback-latency´). It is stamped before the
        // buffer removal rather than after the enqueue so that the work this
        // boundary does — sanitisation, and a journal write that calls
        // `sync_all` — falls inside the queue stage it belongs to instead of
        // vanishing between two stages.
        let arrived_at = self.clock.now_monotonic();

        // Step 1: Remove from pending buffer (irreversible)
        let pending = self
            .pending_buffer
            .remove(data.assessment_id)
            .ok_or(LabelError::AssessmentNotFound)?;

        // Step 2: Sanitise inputs, and count what the boundary repaired
        // (´dec:surface:sanitise-not-reject´) — the counts join the
        // label-integrity health the report assembles.
        let (sanitised, repaired) = sanitise_label_data(data);
        if repaired.valences_sanitised > 0 {
            self.label_valences_sanitised
                .fetch_add(u64::from(repaired.valences_sanitised), std::sync::atomic::Ordering::Relaxed);
        }
        if repaired.outcomes_dropped > 0 {
            self.label_outcomes_dropped
                .fetch_add(u64::from(repaired.outcomes_dropped), std::sync::atomic::Ordering::Relaxed);
        }

        // Steps 3–5: Journal append + channel enqueue.
        //
        // The journal write and channel send are performed under a single
        // journal-mutex critical section when persistence is enabled. This
        // guarantees that journal sequence order matches channel delivery
        // order: thread A cannot append seq=N and then lose the race to
        // thread B which appends seq=N+1 but reaches `try_send` first.
        //
        // Without this ordering guarantee, out-of-order channel delivery
        // combined with the model owner's `last_processed_label_seq`
        // high-water mark could cause a `JournaledButNotEnqueued` label
        // to be silently skipped on replay, against the guarantee that a
        // recorded assessment replays exactly (´cor:durability:replay-exactness´).
        //
        // The `try_send` call is wait-free (~50 ns); the journal
        // `write()+sync_all()` already dominates and already serialises
        // callers, so holding the lock across `try_send` adds negligible
        // contention.
        #[cfg(feature = "serde")]
        if let Some(ref journal) = self.journal {
            let entry = crate::persistence::journal::JournalEntry::unsequenced(sanitised.clone(), pending.to_context());
            let mut writer = journal.lock().expect("journal lock poisoned");
            let assigned_seq = writer.append(&entry).map_err(LabelError::JournalFailed)?;

            let sequenced = SequencedLabel {
                seq: Some(assigned_seq),
                label: sanitised,
                context: LabelContext::Full(Box::new(pending)),
                arrived_at: Some(arrived_at),
            };

            // Send while still holding `writer` so journal order = channel order.
            let send_result = self.label_tx.try_send(sequenced);
            drop(writer);

            return match send_result {
                Ok(()) => Ok(LabelAck { assessment_id }),
                Err(TrySendError::Full(_)) => {
                    self.label_queue_drops.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                    Err(LabelError::JournaledButNotEnqueued)
                }
                Err(TrySendError::Disconnected(_)) => Err(LabelError::ModelOwnerShutdown),
            };
        }

        // No persistence: no journal, no ordering concern between callers.
        let sequenced = SequencedLabel {
            seq: None,
            label: sanitised,
            context: LabelContext::Full(Box::new(pending)),
            arrived_at: Some(arrived_at),
        };

        match self.label_tx.try_send(sequenced) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => {
                self.label_queue_drops.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                return Err(LabelError::ChannelFull);
            }
            Err(TrySendError::Disconnected(_)) => return Err(LabelError::ModelOwnerShutdown),
        }

        Ok(LabelAck { assessment_id })
    }
}

#[cfg(test)]
mod tests {

    use std::collections::HashMap;

    use super::*;
    use crate::types::{Action, OutcomeAxisId};

    fn make_label(valence: f64) -> LabelData {
        LabelData {
            assessment_id: AssessmentId(1),
            action_taken: Action::Allow,
            valence,
            outcomes: HashMap::new(),
            ground_truth: true,
        }
    }

    /// A valence that is not a number is repaired to zero at the boundary rather
    /// than refused, and the repair is counted: the sanitisation record beside
    /// the payload reports one valence repaired. The label still reaches the
    /// model owner, carrying no signal instead of poisoning every scalar
    /// downstream — and because zero is also an ordinary valence, the count is
    /// the only trace the repair leaves (´dec:surface:sanitise-not-reject´).
    ///
    /// ´claim:labelling:a-non-finite-valence-is-repaired-to-zero-and-the-repair-is-counted´
    /// ´test:unit:sanitise-nan-valence´
    #[test]
    fn sanitise_nan_valence() {
        let data = make_label(f64::NAN);
        let (result, repaired) = sanitise_label_data(data);
        assert_eq!(result.valence.to_bits(), 0.0f64.to_bits());
        assert_eq!(repaired.valences_sanitised, 1);
        assert_eq!(repaired.outcomes_dropped, 0);
    }

    /// Infinity is treated exactly as a NaN is: the finiteness test, not a
    /// NaN test, is what the boundary applies, so an unbounded valence collapses
    /// to zero as well. Nothing that could saturate an exponential moving average
    /// gets past the door.
    ///
    /// (´claim:labelling:a-non-finite-valence-is-repaired-to-zero-and-the-repair-is-counted´)
    /// ´test:unit:sanitise-inf-valence´
    #[test]
    fn sanitise_inf_valence() {
        let data = make_label(f64::INFINITY);
        let (result, repaired) = sanitise_label_data(data);
        assert_eq!(result.valence.to_bits(), 0.0f64.to_bits());
        assert_eq!(repaired.valences_sanitised, 1);
    }

    /// Per-axis outcomes are repaired individually and the drops are counted:
    /// the axis whose value is not finite leaves the map, its finite sibling
    /// stays, and the sanitisation record reports exactly one outcome
    /// dropped. One broken axis therefore costs the host that axis only —
    /// and visibly, since a dropped axis leaves no other trace on the label
    /// (´dec:surface:sanitise-not-reject´).
    ///
    /// ´claim:labelling:a-non-finite-outcome-drops-only-its-own-axis-and-the-drop-is-counted´
    /// ´test:unit:sanitise-nan-outcome´
    #[test]
    fn sanitise_nan_outcome() {
        let mut data = make_label(1.0);
        data.outcomes.insert(OutcomeAxisId(1), f64::NAN);
        data.outcomes.insert(OutcomeAxisId(2), 0.8);

        let (result, repaired) = sanitise_label_data(data);
        assert_eq!(result.outcomes.len(), 1);
        assert!(result.outcomes.contains_key(&OutcomeAxisId(2)));
        assert_eq!(repaired.outcomes_dropped, 1);
        assert_eq!(repaired.valences_sanitised, 0);
    }

    /// Repair is confined to what is actually broken: an ordinary valence and
    /// an ordinary outcome come out of the boundary bit-for-bit as they went
    /// in, and the sanitisation record reports nothing repaired. The
    /// sanitiser is a filter on non-finite values, not a transform every
    /// label pays for — and a count that rose on clean labels would make the
    /// diagnostic it feeds meaningless (´dec:surface:sanitise-not-reject´).
    ///
    /// ´claim:labelling:sanitisation-leaves-finite-label-values-exactly-as-submitted´
    /// ´test:unit:sanitise-normal-values-untouched´
    #[test]
    fn sanitise_normal_values_untouched() {
        let mut data = make_label(0.7);
        data.outcomes.insert(OutcomeAxisId(1), 0.3);

        let (result, repaired) = sanitise_label_data(data);
        assert!((result.valence - 0.7).abs() < 1e-9);
        assert_eq!(result.outcomes.len(), 1);
        assert_eq!(repaired, LabelSanitisation::default());
    }
}
