// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`observation_channel_send_receive`] | identity | A coordinate handed over by an assessment thread waits in the queue and comes back out unchanged on the maintenance side. Deferral is what keeps the assessment path off the graph: the assessing thread pays only the cost of an enqueue, and the coordinate it named is exactly the one the maintenance loop will later apply. |
//! | [`observation_channel_bounded`] | identity | The queue is bounded: it accepts exactly as many pending observations as its capacity allows and then refuses the next one instead of growing. A maintenance loop that falls behind therefore costs a fixed amount of memory rather than an unbounded backlog that would eventually take the host down with it. |
//! | [`observation_channel_overflow_counter`] | identity | Each refused observation is counted, one per attempt, so shedding load is recorded rather than silent. The identity layer's picture of traffic is only as complete as the observations that reached it, and the counter is what lets an operator tell a quiet dimension from a saturated one. |
//! | [`observation_channel_sender_clone`] | identity | A sender handed out to an assessment thread feeds the same queue the maintenance loop reads. There is one backlog per dimension however many threads are assessing, which is what lets the loop see a dimension's traffic as a single stream rather than reconciling per-thread queues. |
//! | [`observation_channel_default`] | identity | A dimension that names no capacity gets the standard one rather than some incidental value. The default is generous enough that ordinary bursts ride out a maintenance cycle without any observation being shed, so registering a dimension without tuning it is a safe thing to do. |
//! | [`drain_processes_all`] | identity | Draining a backlog yields every observation in it, each exactly once and in the order it was submitted, and leaves the queue empty. Order matters because the G-V graph splits as importance concentrates, so replaying a burst out of sequence would build a different structure from the one the traffic actually described. |

//! Observation deferral channel for identity dimensions.
//!
// Data structures only. A dedicated owner drains this channel into the graph,
// and the assessment path never touches the graph (´dec:memory:graph-owner´).
#![allow(dead_code)]
//!
//! This module provides the observation deferral mechanism
//! (´dec:memory:observation-surface´):
//!
//! - Per-dimension bounded channel accepts only `u128` coordinates
//! - `Δ=1` is hardcoded inside the maintenance loop (feed-forward enforcement)
//! - Overflow increments a counter; observation is lost
//!
//! # The Feed-Forward Boundary
//!
//! The API accepts only a `u128` coordinate. There is no path for callers
//! to specify a custom delta value. The maintenance loop calls `graph.observe()`
//! with `Δ=1` internally.
//!
//! # Cross-References
//!
//! - (´dec:memory:observation-surface´) — why the surface accepts a coordinate
//!   and nothing else
//! - (´dec:memory:overflow-degrades´) — why a full queue counts the loss
//!   instead of failing
//! - (´dec:ownership:feed-forward´) — the one direction information crosses
//!   the boundary
//! - (´alg:keyspace:observation-protocol´) — the protocol this defers

use std::sync::atomic::{AtomicU64, Ordering};

use crossbeam_channel::{Receiver, Sender, TrySendError};

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Default capacity for observation channels.
///
/// Every assessment may offer to this queue, so it is given the larger of the
/// two deferral depths — the cushion it buys is against a maintenance loop that
/// has fallen behind on the system's hottest path. A full queue drops the
/// observation and counts it rather than blocking the assessment
/// (´dec:concurrency:deferral-depth´).
///
/// ´const:assayer:deferred-observation-queue-depth´ (´alg:const:count´)
/// ´const:assayer:deferred-observation-queue-depth-count-100000´
pub const DEFAULT_OBSERVATION_CHANNEL_CAPACITY: usize = 100_000;

// ═══════════════════════════════════════════════════════════════════════════════
// Observation Channel
// ═══════════════════════════════════════════════════════════════════════════════

/// Observation deferral channel for a single identity dimension.
///
/// Provides a bounded channel that accepts `u128` coordinates from the
/// `assess()` path and delivers them to the maintenance loop. Overflow
/// observations are dropped and counted.
///
/// # Thread Safety
///
/// The sender handle is `Clone + Send + Sync`, allowing multiple `assess()`
/// threads to submit observations concurrently.
pub struct ObservationChannel {
    /// Sender for coordinates.
    tx: Sender<u128>,

    /// Receiver for coordinates (held by maintenance loop).
    rx: Receiver<u128>,

    /// Counter for overflow events.
    overflow_count: AtomicU64,

    /// Channel capacity.
    capacity: usize,
}

impl ObservationChannel {
    /// Creates a new observation channel with the default capacity.
    #[must_use]
    pub fn new() -> Self {
        Self::with_capacity(DEFAULT_OBSERVATION_CHANNEL_CAPACITY)
    }

    /// Creates a new observation channel with the given capacity.
    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        let (tx, rx) = crossbeam_channel::bounded(capacity);
        Self {
            tx,
            rx,
            overflow_count: AtomicU64::new(0),
            capacity,
        }
    }

    /// Attempts to send a coordinate observation.
    ///
    /// Returns `true` if the observation was enqueued, `false` if the
    /// channel was full (overflow). On overflow, the overflow counter
    /// is incremented.
    ///
    /// # The Feed-Forward Boundary
    ///
    /// Only the coordinate is accepted. The delta value `Δ=1` is applied
    /// by the maintenance loop when it calls `graph.observe()`.
    pub fn try_send(&self, coord: u128) -> bool {
        match self.tx.try_send(coord) {
            Ok(()) => true,
            Err(TrySendError::Full(_)) => {
                self.overflow_count.fetch_add(1, Ordering::Relaxed);
                false
            }
            Err(TrySendError::Disconnected(_)) => {
                // Channel disconnected — maintenance loop has shut down
                false
            }
        }
    }

    /// Returns a clone of the sender for use by assessment threads.
    #[must_use]
    pub fn sender(&self) -> Sender<u128> {
        self.tx.clone()
    }

    /// Returns a reference to the receiver (for the maintenance loop).
    #[must_use]
    pub const fn receiver(&self) -> &Receiver<u128> {
        &self.rx
    }

    /// Consumes self and returns the receiver.
    ///
    /// Use this when handing off the receiver to the maintenance loop.
    #[must_use]
    pub fn into_receiver(self) -> Receiver<u128> {
        self.rx
    }

    /// Returns the current overflow count.
    #[must_use]
    pub fn overflow_count(&self) -> u64 {
        self.overflow_count.load(Ordering::Relaxed)
    }

    /// Returns the channel capacity.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Returns `true` if the channel has no pending observations.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rx.is_empty()
    }

    /// Returns the number of pending observations.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rx.len()
    }
}

impl Default for ObservationChannel {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for ObservationChannel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ObservationChannel")
            .field("capacity", &self.capacity)
            .field("pending", &self.len())
            .field("overflow_count", &self.overflow_count())
            .field("tx", &"..")
            .field("rx", &"..")
            .finish()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// A coordinate handed over by an assessment thread waits in the queue and
    /// comes back out unchanged on the maintenance side. Deferral is what keeps
    /// the assessment path off the graph: the assessing thread pays only the
    /// cost of an enqueue, and the coordinate it named is exactly the one the
    /// maintenance loop will later apply.
    ///
    /// ´claim:identity:an-observation-handed-over-by-an-assessment-thread-reaches-the-maintenance-side-unchanged´
    /// ´test:unit:observation-channel-send-receive´
    #[test]
    fn observation_channel_send_receive() {
        let channel = ObservationChannel::with_capacity(10);

        // Send a coordinate
        assert!(channel.try_send(0x12345));
        assert_eq!(channel.len(), 1);

        // Receive it
        let coord = channel.receiver().try_recv().unwrap();
        assert_eq!(coord, 0x12345);
        assert!(channel.is_empty());
    }

    /// The queue is bounded: it accepts exactly as many pending observations as
    /// its capacity allows and then refuses the next one instead of growing.
    /// A maintenance loop that falls behind therefore costs a fixed amount of
    /// memory rather than an unbounded backlog that would eventually take the
    /// host down with it.
    ///
    /// ´claim:identity:the-observation-queue-is-bounded-and-refuses-work-past-capacity-rather-than-growing´
    /// ´test:unit:observation-channel-bounded´
    #[test]
    fn observation_channel_bounded() {
        let channel = ObservationChannel::with_capacity(3);

        // Fill the channel
        assert!(channel.try_send(1));
        assert!(channel.try_send(2));
        assert!(channel.try_send(3));

        // Next send should fail (channel full)
        assert!(!channel.try_send(4));

        // Verify all three were enqueued
        assert_eq!(channel.len(), 3);
    }

    /// Each refused observation is counted, one per attempt, so shedding load
    /// is recorded rather than silent. The identity layer's picture of traffic
    /// is only as complete as the observations that reached it, and the counter
    /// is what lets an operator tell a quiet dimension from a saturated one.
    ///
    /// ´claim:identity:every-refused-observation-is-counted-so-shed-load-is-visible-rather-than-silent´
    /// ´test:unit:observation-channel-overflow-counter´
    #[test]
    fn observation_channel_overflow_counter() {
        let channel = ObservationChannel::with_capacity(2);

        // Fill channel
        assert!(channel.try_send(1));
        assert!(channel.try_send(2));

        // Overflow
        assert!(!channel.try_send(3));
        assert!(!channel.try_send(4));
        assert!(!channel.try_send(5));

        // Should have 3 overflow events
        assert_eq!(channel.overflow_count(), 3);
    }

    /// A sender handed out to an assessment thread feeds the same queue the
    /// maintenance loop reads. There is one backlog per dimension however many
    /// threads are assessing, which is what lets the loop see a dimension's
    /// traffic as a single stream rather than reconciling per-thread queues.
    ///
    /// ´claim:identity:a-cloned-sender-feeds-the-same-queue-so-many-assessment-threads-share-one-backlog-per-dimension´
    /// ´test:unit:observation-channel-sender-clone´
    #[test]
    fn observation_channel_sender_clone() {
        let channel = ObservationChannel::with_capacity(10);

        // Get a cloned sender
        let sender = channel.sender();

        // Send via cloned sender
        sender.try_send(42).unwrap();

        // Receive via original channel
        let coord = channel.receiver().try_recv().unwrap();
        assert_eq!(coord, 42);
    }

    /// A dimension that names no capacity gets the standard one rather than
    /// some incidental value. The default is generous enough that ordinary
    /// bursts ride out a maintenance cycle without any observation being shed,
    /// so registering a dimension without tuning it is a safe thing to do.
    ///
    /// ´claim:identity:a-dimension-that-names-no-capacity-gets-the-standard-observation-queue-size´
    /// ´test:unit:observation-channel-default´
    #[test]
    fn observation_channel_default() {
        let channel = ObservationChannel::default();
        assert_eq!(channel.capacity(), DEFAULT_OBSERVATION_CHANNEL_CAPACITY);
    }

    /// Draining a backlog yields every observation in it, each exactly once and
    /// in the order it was submitted, and leaves the queue empty. Order matters
    /// because the G-V graph splits as importance concentrates, so replaying a
    /// burst out of sequence would build a different structure from the one the
    /// traffic actually described.
    ///
    /// ´claim:identity:draining-yields-every-queued-observation-once-and-in-arrival-order´
    /// ´test:unit:drain-processes-all´
    #[test]
    fn drain_processes_all() {
        let channel = ObservationChannel::with_capacity(100);

        // Send 50 observations
        for i in 0..50u128 {
            assert!(channel.try_send(i));
        }
        assert_eq!(channel.len(), 50);

        // Drain all via receiver
        let mut drained = Vec::new();
        while let Ok(coord) = channel.receiver().try_recv() {
            drained.push(coord);
        }

        assert_eq!(drained.len(), 50, "all 50 observations should be drained");
        assert!(channel.is_empty(), "channel should be empty after drain");

        // Verify ordering preserved
        for (i, &coord) in drained.iter().enumerate() {
            assert_eq!(coord, i as u128);
        }
    }
}
