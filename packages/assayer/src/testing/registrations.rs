// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Lifecycle-event registration builders.
//!
//! Small constructors for the `*Registration` payloads that lifecycle
//! tests submit to the engine. Centralised here so the same shapes do
//! not drift across `src/tests/` files.

use crate::owner::commands::{OutcomeAxisRegistration, SentinelRegistration};
use crate::types::{OutcomeAxisId, OutcomeEligibility, SentinelId, SpatialFeaturePolicy};

/// `SentinelRegistration` with a generated `test-sentinel-{id}` name.
#[must_use]
pub fn sentinel_reg(id: SentinelId) -> SentinelRegistration {
    SentinelRegistration {
        id,
        name: format!("test-sentinel-{}", id.0),
    }
}

/// `OutcomeAxisRegistration` with default eligibility and
/// [`SpatialFeaturePolicy::default()`].
#[must_use]
pub fn axis_reg(id: OutcomeAxisId) -> OutcomeAxisRegistration {
    OutcomeAxisRegistration {
        id,
        name: format!("test-axis-{}", id.0),
        description: format!("test axis {} description", id.0),
        eligibility: OutcomeEligibility::default(),
        initial_kappa: 1.0,
        gamma: 0.99,
        spatial_features: SpatialFeaturePolicy::default(),
    }
}

/// `OutcomeAxisRegistration` with [`SpatialFeaturePolicy::Enabled`],
/// i.e. the axis contributes per-Sentinel features.
#[must_use]
pub fn spatial_axis_reg(id: OutcomeAxisId) -> OutcomeAxisRegistration {
    OutcomeAxisRegistration {
        id,
        name: format!("test-spatial-axis-{}", id.0),
        description: format!("test spatial axis {} description", id.0),
        eligibility: OutcomeEligibility::default(),
        initial_kappa: 1.0,
        gamma: 0.99,
        spatial_features: SpatialFeaturePolicy::Enabled,
    }
}
