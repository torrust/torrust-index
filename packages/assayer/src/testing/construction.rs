// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! The construction figures the harness declares.
//!
//! A deployment sets its own; these are the harness's, declared here because
//! nothing outside the harness declares them (´dec:assayer:harness-declares´)
//! and registered as such (´tab:assayer:harness-construction-figures´).

/// The harness's declared command-channel capacity.
///
/// The capacity is host-set with no default (´tab:construction:parameters´),
/// which is why a figure must stand here at all; what fixes this figure is the
/// harness's own declaration (´tab:assayer:harness-construction-figures´). It is
/// the one the retired literal shipped, kept so the behaviour under test is the
/// behaviour the suite has always exercised, and a deployment declares its own
/// rather than inheriting it (´dec:assayer:harness-declares´).
///
/// ´const:assayer:harness-command-channel-depth´ (´alg:const:count´)
/// ´const:assayer:harness-command-channel-depth-count-64´
pub const TEST_COMMAND_CHANNEL_CAPACITY: usize = 64;

/// An `InfrastructureConfig` for fixtures: the defaults, plus the harness's declared
/// command-channel capacity — the one field a default cannot supply.
#[must_use]
pub fn test_infrastructure() -> crate::config::types::InfrastructureConfig {
    crate::config::types::InfrastructureConfig {
        command_channel_capacity: Some(TEST_COMMAND_CHANNEL_CAPACITY),
        ..Default::default()
    }
}
