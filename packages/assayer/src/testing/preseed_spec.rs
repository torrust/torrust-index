// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Pre-seed entry builders for scenario tests.
//!
//! Mirrors [`super::LabelSpec`] for the [`PreSeedEntry`] surface.
//! Scenarios usually care about one knob (valence, action,
//! ground-truth) so this fluent builder fills in the rest with
//! sensible defaults:
//!
//! ```ignore
//! let entry = PreSeedSpec::adverse(channel, entity).ground_truth().build();
//! ```

use std::collections::HashMap;

use crate::api::PreSeedEntry;
use crate::types::{Action, ChannelId, EntityKey, OutcomeAxisId};

/// Fluent builder for [`PreSeedEntry`].
#[derive(Clone, Debug)]
pub struct PreSeedSpec {
    /// Underlying entry, mutated by the builder methods.
    entry: PreSeedEntry,
}

impl PreSeedSpec {
    /// Start a new spec for `entity` on `channel`.
    ///
    /// Defaults: `Allow` action, valence 0.0 (benign), no outcomes,
    /// not ground-truth.
    #[must_use]
    pub fn new(channel: ChannelId, entity: EntityKey) -> Self {
        Self {
            entry: PreSeedEntry {
                entity,
                channel,
                action_taken: Action::Allow,
                valence: 0.0,
                outcomes: HashMap::new(),
                ground_truth: false,
            },
        }
    }

    /// Shorthand for a clearly-adverse entry: valence = 1.0.
    #[must_use]
    pub fn adverse(channel: ChannelId, entity: EntityKey) -> Self {
        Self::new(channel, entity).valence(1.0)
    }

    /// Shorthand for a clearly-benign entry: valence = -1.0.
    #[must_use]
    pub fn benign(channel: ChannelId, entity: EntityKey) -> Self {
        Self::new(channel, entity).valence(-1.0)
    }

    /// Set the host action taken.
    #[must_use]
    pub const fn action(mut self, action: Action) -> Self {
        self.entry.action_taken = action;
        self
    }

    /// Set the valence (positive = adverse).
    #[must_use]
    pub const fn valence(mut self, v: f64) -> Self {
        self.entry.valence = v;
        self
    }

    /// Mark this entry as ground-truth.
    #[must_use]
    pub const fn ground_truth(mut self) -> Self {
        self.entry.ground_truth = true;
        self
    }

    /// Record an outcome-axis value.
    #[must_use]
    pub fn outcome(mut self, axis: OutcomeAxisId, value: f64) -> Self {
        self.entry.outcomes.insert(axis, value);
        self
    }

    /// Consume the builder and produce the underlying [`PreSeedEntry`].
    #[must_use]
    pub fn build(self) -> PreSeedEntry {
        self.entry
    }
}

impl From<PreSeedSpec> for PreSeedEntry {
    fn from(spec: PreSeedSpec) -> Self {
        spec.build()
    }
}
