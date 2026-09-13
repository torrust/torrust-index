// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! The deterministic clock a scenario drives time with.
//!
//! It is a harness item and it is gated as one, so a sealed build compiles no
//! virtual time at all. The trait it implements stays beside the production
//! clock in [`super::clock`], because that is what the engine reads its time
//! through in every build.
//!
//! # Contract
//!
//! [`Clock::now`] must be **monotonically non-decreasing** within a single
//! test. [`VirtualClock::travel_to`] panics on backward jumps to make the
//! invariant unmissable.
//!
//! A [`VirtualClock`] instance belongs to one [`World`](super::World). It is
//! never a process-wide or thread-local singleton: integration tests run many
//! worlds in one process, and a shared clock would make them observe each
//! other.

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::clock::Clock;
use crate::types::PersistentTimestamp;

/// Deterministic, test-controlled clock.
///
/// Gated with the harness it belongs to: a sealed build has no caller for
/// virtual time, and compiling it there would be compiling something dead.
///
/// Internally stores nanoseconds since the Unix epoch in an
/// [`AtomicU64`] so multiple threads can read concurrently without
/// locking. Tests drive it via [`Self::travel_to`] and [`Self::advance`].
///
/// # Example
///
/// ```ignore
/// use std::sync::Arc;
/// use std::time::Duration;
/// use torrust_assayer::testing::{Clock, VirtualClock};
///
/// let clock = Arc::new(VirtualClock::epoch());
/// let t0 = clock.now();
/// clock.advance(Duration::from_secs(60 * 60 * 24));
/// let t1 = clock.now();
/// assert!(t1.seconds() > t0.seconds());
/// ```
#[derive(Debug)]
pub struct VirtualClock {
    /// Nanoseconds since the Unix epoch.
    nanos: AtomicU64,
    /// Reading of [`Self::nanos`] at construction.
    ///
    /// The monotonic domain is expressed as an offset from this, so
    /// that time travel moves both domains by the same amount.
    base_nanos: u64,
    /// Real [`Instant`] captured once at construction.
    ///
    /// [`Instant`] is opaque and cannot be synthesised, so the virtual
    /// monotonic reading is built as `base_instant + offset` rather
    /// than from an absolute value.
    base_instant: Instant,
}

impl VirtualClock {
    /// Fixed epoch used by the default test clock.
    ///
    /// 2026-01-01T00:00:00Z — a round, stable point that is comfortably
    /// inside the `PersistentTimestamp` valid range and won't overflow
    /// when scenarios project months into the future.
    ///
    /// The properties the harness needs are stated above; the instant that
    /// carries them is the harness's own choice among many that would, declared
    /// as such rather than derived (´tab:assayer:harness-construction-figures´).
    ///
    /// ´const:assayer:harness-clock-epoch´ (´alg:const:seconds´)
    /// ´const:assayer:harness-clock-epoch-seconds-1767225600´
    pub const EPOCH_SECS: u64 = 1_767_225_600;

    /// Constructs a clock anchored at [`Self::EPOCH_SECS`].
    #[must_use]
    pub fn epoch() -> Self {
        Self::at_secs(Self::EPOCH_SECS)
    }

    /// Constructs a clock at the given whole-second offset from the
    /// Unix epoch.
    #[must_use]
    pub fn at_secs(secs: u64) -> Self {
        let nanos = secs.saturating_mul(1_000_000_000);
        Self {
            nanos: AtomicU64::new(nanos),
            base_nanos: nanos,
            base_instant: Instant::now(),
        }
    }

    /// Jumps the clock forward to an absolute point (nanoseconds from
    /// the Unix epoch).
    ///
    /// # Panics
    ///
    /// Panics if `target_nanos` is earlier than the current reading —
    /// backward travel violates the monotonicity contract.
    pub fn travel_to_nanos(&self, target_nanos: u64) {
        let current = self.nanos.load(Ordering::Acquire);
        assert!(
            target_nanos >= current,
            "VirtualClock: backward travel forbidden ({target_nanos} < {current})",
        );
        self.nanos.store(target_nanos, Ordering::Release);
    }

    /// Jumps the clock forward to an absolute [`SystemTime`].
    ///
    /// # Panics
    ///
    /// Panics if `target` is before the Unix epoch, or earlier than
    /// the current clock reading.
    pub fn travel_to(&self, target: SystemTime) {
        let dur = target
            .duration_since(UNIX_EPOCH)
            .expect("VirtualClock: target must be >= UNIX_EPOCH");
        let nanos = u64::try_from(dur.as_nanos()).expect("VirtualClock: target does not fit in u64 nanoseconds");
        self.travel_to_nanos(nanos);
    }

    /// Advances the clock by `delta`.
    ///
    /// Saturating at `u64::MAX` nanoseconds.
    pub fn advance(&self, delta: Duration) {
        let delta_nanos = u64::try_from(delta.as_nanos()).unwrap_or(u64::MAX);
        self.nanos.fetch_add(delta_nanos, Ordering::AcqRel);
    }
}

impl Default for VirtualClock {
    fn default() -> Self {
        Self::epoch()
    }
}

impl Clock for VirtualClock {
    fn now(&self) -> PersistentTimestamp {
        let nanos = self.nanos.load(Ordering::Acquire);
        let secs = nanos / 1_000_000_000;
        let sub = u32::try_from(nanos % 1_000_000_000).unwrap_or(0);
        let system_time = UNIX_EPOCH + Duration::new(secs, sub);
        PersistentTimestamp::from_system_time(system_time)
    }

    fn now_monotonic(&self) -> Instant {
        let elapsed = self.nanos.load(Ordering::Acquire).saturating_sub(self.base_nanos);
        self.base_instant + Duration::from_nanos(elapsed)
    }
}

impl Clock for Arc<VirtualClock> {
    fn now(&self) -> PersistentTimestamp {
        (**self).now()
    }

    fn now_monotonic(&self) -> Instant {
        (**self).now_monotonic()
    }
}
