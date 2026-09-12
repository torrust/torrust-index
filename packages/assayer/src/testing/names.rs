// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Name-based handles for test-owned engine entities.
//!
//! Scenarios in `tests/README.md` speak of "Sentinel A", "the
//! magnitude axis", or "the default channel". The harness translates
//! these human names to [`SentinelId`], [`OutcomeAxisId`], and
//! [`ChannelId`] so tests never handle raw integers.
//!
//! These newtypes are transparent wrappers over `&'static str` — the
//! expected form in tests is a string literal (`SentinelName("S1")`).

use crate::types::{ChannelId, DimensionId, OutcomeAxisId, SentinelId};

/// Test-facing name for a Sentinel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SentinelName(pub &'static str);

impl From<&'static str> for SentinelName {
    fn from(s: &'static str) -> Self {
        Self(s)
    }
}

/// Test-facing name for an outcome axis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AxisName(pub &'static str);

impl From<&'static str> for AxisName {
    fn from(s: &'static str) -> Self {
        Self(s)
    }
}

/// Test-facing name for a channel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChannelName(pub &'static str);

impl From<&'static str> for ChannelName {
    fn from(s: &'static str) -> Self {
        Self(s)
    }
}

/// Test-facing name for an identity dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IdentityName(pub &'static str);

impl From<&'static str> for IdentityName {
    fn from(s: &'static str) -> Self {
        Self(s)
    }
}

/// Small registry mapping names to engine IDs.
///
/// Owned by [`World`](super::World); not intended to be used directly.
#[derive(Debug, Default)]
pub(super) struct NameRegistry {
    /// Registered Sentinels, in registration order.
    pub sentinels: Vec<(SentinelName, SentinelId)>,
    /// Registered outcome axes, in registration order.
    pub axes: Vec<(AxisName, OutcomeAxisId)>,
    /// Declared channels, in declaration order.
    pub channels: Vec<(ChannelName, ChannelId)>,
    /// Registered identity dimensions, in registration order.
    pub identities: Vec<(IdentityName, DimensionId)>,
    /// Next Sentinel ID to hand out.
    pub next_sentinel: u32,
    /// Next outcome-axis ID to hand out.
    pub next_axis: u32,
    /// Next identity-dimension ID to hand out.
    pub next_identity: u32,
}

impl NameRegistry {
    /// Look up a Sentinel by name.
    pub fn sentinel(&self, name: SentinelName) -> Option<SentinelId> {
        self.sentinels.iter().find_map(|(n, id)| (*n == name).then_some(*id))
    }

    /// Look up an outcome axis by name.
    pub fn axis(&self, name: AxisName) -> Option<OutcomeAxisId> {
        self.axes.iter().find_map(|(n, id)| (*n == name).then_some(*id))
    }

    /// Look up a channel by name.
    pub fn channel(&self, name: ChannelName) -> Option<ChannelId> {
        self.channels.iter().find_map(|(n, id)| (*n == name).then_some(*id))
    }

    /// Look up an identity dimension by name.
    pub fn identity(&self, name: IdentityName) -> Option<DimensionId> {
        self.identities.iter().find_map(|(n, id)| (*n == name).then_some(*id))
    }

    /// Allocate the next Sentinel ID without recording a name.
    /// Used when the caller doesn't need name-based lookup.
    pub const fn alloc_sentinel_id(&mut self) -> SentinelId {
        let id = SentinelId(self.next_sentinel);
        self.next_sentinel += 1;
        id
    }

    /// Allocate the next identity-dimension ID without recording a name.
    pub const fn alloc_identity_id(&mut self) -> DimensionId {
        let id = DimensionId(self.next_identity);
        self.next_identity += 1;
        id
    }

    /// Allocate the next outcome-axis ID without recording a name.
    pub const fn alloc_axis_id(&mut self) -> OutcomeAxisId {
        let id = OutcomeAxisId(self.next_axis);
        self.next_axis += 1;
        id
    }
}
