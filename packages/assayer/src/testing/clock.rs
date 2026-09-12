// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! The engine's source of time, in both domains.
//!
//! # Two domains, one handle
//!
//! The clock reports both timestamp domains that (´dec:clock:two-domains´)
//! keeps apart: [`Clock::now`] serves the persistent domain and
//! [`Clock::now_monotonic`] the intra-process one. A single handle
//! carries both so that time travel moves them together; the domains
//! stay separated by type, and no conversion between them exists.
//!
//! The monotonic reading is the one report staleness and model-decay
//! intervals are measured against, which makes it — not the persistent
//! reading — the source of cross-run divergence in reported-Sentinel
//! comparisons.

use std::time::{Instant, SystemTime};

use crate::types::PersistentTimestamp;

/// Source of "now" used throughout the Assayer engine.
///
/// The production default is [`SystemClock`]; tests substitute
/// [`VirtualClock`](super::VirtualClock) for full determinism.
pub trait Clock: Send + Sync {
    /// Returns the current time in the persistent domain.
    ///
    /// This is the domain that survives a restart and is the only one
    /// written to storage (´dec:clock:two-domains´).
    fn now(&self) -> PersistentTimestamp;

    /// Returns the current time in the intra-process monotonic domain.
    ///
    /// This is the domain report staleness and model-decay intervals
    /// are measured in. It is kept distinct from [`Self::now`] by type,
    /// per (´dec:clock:two-domains´); no conversion exists between them.
    fn now_monotonic(&self) -> Instant;
}

/// Wall-clock implementation used in production.
#[derive(Debug, Default, Clone, Copy)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> PersistentTimestamp {
        PersistentTimestamp::from_system_time(SystemTime::now())
    }

    fn now_monotonic(&self) -> Instant {
        Instant::now()
    }
}
