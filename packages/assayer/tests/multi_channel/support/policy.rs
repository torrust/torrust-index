// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use torrust_assayer::types::Action;
use torrust_assayer::{ChannelPolicy, RewardParameters};

/// Build a [`ChannelPolicy`] with the supplied action set and the
/// default reward parameters.
///
/// The integration suite frequently spins up channels that differ
/// only in their action set; centralising the boilerplate here keeps
/// every test file from duplicating the same three-line `ChannelPolicy
/// { actions: …, reward: RewardParameters::default() }` literal.
#[must_use]
pub fn policy_with_actions<I>(actions: I) -> ChannelPolicy
where
    I: IntoIterator<Item = Action>,
{
    policy_with(actions, RewardParameters::default())
}

/// Build a [`ChannelPolicy`] with an explicit action set and reward
/// block.
///
/// Companion to [`policy_with_actions`] for tests that vary the
/// reward parameters — reward-axis independence of the risk basis
/// (´claim:channel:reward-parameters-configure-the-action-layer-alone-and-cannot-reach-the-core´).
#[must_use]
pub fn policy_with<I>(actions: I, reward: RewardParameters) -> ChannelPolicy
where
    I: IntoIterator<Item = Action>,
{
    ChannelPolicy {
        actions: actions.into_iter().collect(),
        reward,
        ..ChannelPolicy::default()
    }
}

/// Build a [`ChannelPolicy`] by applying a perturbation to the default
/// reward block.
///
/// This is the common reward-perturbation shape
/// (´claim:channel:reward-parameters-configure-the-action-layer-alone-and-cannot-reach-the-core´):
/// two channels share an action set, one channel keeps default rewards, and the
/// other changes exactly one decision-layer knob. Keeping that construction here
/// makes individual tests name the perturbation instead of restating policy
/// boilerplate.
#[must_use]
pub fn policy_with_reward_perturbation<I, F>(actions: I, perturb: F) -> ChannelPolicy
where
    I: IntoIterator<Item = Action>,
    F: FnOnce(&mut RewardParameters),
{
    let mut reward = RewardParameters::default();
    perturb(&mut reward);
    policy_with(actions, reward)
}

/// Reward block used by the policy-matrix probes
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
///
/// The perturbation deliberately moves several independent decision-layer axes
/// at once. Tests pair it with an identical action set to prove the action
/// layer is live while the core assessment remains unchanged.
#[must_use]
pub fn high_decision_cost_reward() -> RewardParameters {
    let mut reward = RewardParameters::default();
    reward.missed *= 2.0;
    reward.friction *= 4.0;
    reward.caught *= 1.5;
    reward
}
