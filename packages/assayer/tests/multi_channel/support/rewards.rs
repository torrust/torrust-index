// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use torrust_assayer::testing::{Scenario, WorldBuildError, WorldBuilder, assert_health_clean, scenario_with};
use torrust_assayer::types::Action;
use torrust_assayer::{RewardParameters, Tag};

use super::constants::{FOUR_ACTIONS, LOGIN_ACTION_TAGS, THREE_ACTIONS, TRANSACTION_ACTION_TAGS};
use super::derivation::assert_channel_pair_moves_only_action_layer;
use super::policy::{policy_with, policy_with_reward_perturbation};

/// One reward-parameter perturbation used by the reward-axis sweeps
/// (´claim:channel:reward-parameters-configure-the-action-layer-alone-and-cannot-reach-the-core´).
#[derive(Clone, Copy, Debug)]
pub struct RewardPerturbationCase<'a> {
    /// Human-readable parameter label.
    pub name: &'static str,
    /// Stable per-case seed salt.
    pub seed: u64,
    /// Shared channel action set for this perturbation.
    pub actions: &'a [Action],
    /// Expected public action tags for that action set.
    pub action_tags: &'a [Tag],
    /// Mutation applied to the perturbed channel's reward block.
    pub perturb: fn(&mut RewardParameters),
}

fn double_missed(reward: &mut RewardParameters) {
    reward.missed *= 2.0;
}

fn multiply_friction(reward: &mut RewardParameters) {
    reward.friction *= 5.0;
}

fn multiply_caught(reward: &mut RewardParameters) {
    reward.caught *= 3.0;
}

fn multiply_pass(reward: &mut RewardParameters) {
    reward.pass *= 4.0;
}

fn multiply_blocked(reward: &mut RewardParameters) {
    reward.blocked *= 3.0;
}

#[allow(clippy::missing_const_for_fn)] // Justified: stored as a runtime perturbation function pointer in the reward-axis catalogue.
fn lower_block_catches(reward: &mut RewardParameters) {
    reward.block_catches = 0.25;
}

fn double_beta_bad(reward: &mut RewardParameters) {
    reward.beta_bad *= 2.0;
}

fn triple_beta_good(reward: &mut RewardParameters) {
    reward.beta_good *= 3.0;
}

#[allow(clippy::missing_const_for_fn)] // Justified: stored as a runtime perturbation function pointer in the reward-axis catalogue.
fn raise_slow_severity(reward: &mut RewardParameters) {
    reward.slow_severity = 0.95;
}

/// Canonical reward-axis perturbations
/// (´claim:channel:reward-parameters-configure-the-action-layer-alone-and-cannot-reach-the-core´).
pub const REWARD_AXIS_SWEEP_CASES: [RewardPerturbationCase<'static>; 9] = [
    RewardPerturbationCase {
        name: "R_missed",
        seed: 0x7110_0001,
        actions: &THREE_ACTIONS,
        action_tags: &LOGIN_ACTION_TAGS,
        perturb: double_missed,
    },
    RewardPerturbationCase {
        name: "R_friction",
        seed: 0x7110_0002,
        actions: &THREE_ACTIONS,
        action_tags: &LOGIN_ACTION_TAGS,
        perturb: multiply_friction,
    },
    RewardPerturbationCase {
        name: "R_caught",
        seed: 0x7110_0003,
        actions: &THREE_ACTIONS,
        action_tags: &LOGIN_ACTION_TAGS,
        perturb: multiply_caught,
    },
    RewardPerturbationCase {
        name: "R_pass",
        seed: 0x7110_0004,
        actions: &THREE_ACTIONS,
        action_tags: &LOGIN_ACTION_TAGS,
        perturb: multiply_pass,
    },
    RewardPerturbationCase {
        name: "R_blocked",
        seed: 0x7110_0005,
        actions: &THREE_ACTIONS,
        action_tags: &LOGIN_ACTION_TAGS,
        perturb: multiply_blocked,
    },
    RewardPerturbationCase {
        name: "block_catches",
        seed: 0x7110_0006,
        actions: &THREE_ACTIONS,
        action_tags: &LOGIN_ACTION_TAGS,
        perturb: lower_block_catches,
    },
    RewardPerturbationCase {
        name: "beta_bad",
        seed: 0x7110_0007,
        actions: &THREE_ACTIONS,
        action_tags: &LOGIN_ACTION_TAGS,
        perturb: double_beta_bad,
    },
    RewardPerturbationCase {
        name: "beta_good",
        seed: 0x7110_0008,
        actions: &THREE_ACTIONS,
        action_tags: &LOGIN_ACTION_TAGS,
        perturb: triple_beta_good,
    },
    RewardPerturbationCase {
        name: "slow_severity",
        seed: 0x7110_0009,
        actions: &FOUR_ACTIONS,
        action_tags: &TRANSACTION_ACTION_TAGS,
        perturb: raise_slow_severity,
    },
];

/// Build a standard two-channel reward-perturbation scenario.
///
/// The resulting world declares:
///
/// - `baseline`: supplied action set + default rewards
/// - `perturbed`: same action set + one reward-block perturbation
///
/// This is the shared fixture for the tests that need to prove reward
/// parameters are decision-layer inputs rather than core-model inputs
/// (´claim:channel:reward-parameters-configure-the-action-layer-alone-and-cannot-reach-the-core´).
pub fn reward_perturbation_scenario<F>(
    instance_id: &str,
    seed: u64,
    actions: &[Action],
    perturb: F,
) -> Result<Scenario, WorldBuildError>
where
    F: FnOnce(&mut RewardParameters),
{
    reward_perturbation_scenario_with(instance_id, seed, actions, perturb, |builder| builder)
}

/// Build a reward-perturbation scenario with additional shared world setup.
///
/// Use this when both channels need the same signal schema or other builder
/// configuration before the baseline/perturbed channel pair is declared.
pub fn reward_perturbation_scenario_with<F, G>(
    instance_id: &str,
    seed: u64,
    actions: &[Action],
    perturb: F,
    configure: G,
) -> Result<Scenario, WorldBuildError>
where
    F: FnOnce(&mut RewardParameters),
    G: FnOnce(WorldBuilder) -> WorldBuilder,
{
    let baseline_policy = policy_with(actions.iter().copied(), RewardParameters::default());
    let perturbed_policy = policy_with_reward_perturbation(actions.iter().copied(), perturb);

    scenario_with(instance_id, seed, |builder| {
        configure(builder)
            .channel("baseline", baseline_policy)
            .channel("perturbed", perturbed_policy)
    })
}

/// Assert one named reward-axis perturbation leaves the core risk basis fixed.
#[track_caller]
pub fn assert_reward_perturbation_case_does_not_perturb_risk_basis(
    instance_id: &str,
    case: RewardPerturbationCase<'_>,
    entity: &str,
) {
    let world = reward_perturbation_scenario(instance_id, case.seed, case.actions, case.perturb).expect("world should build");
    assert_channel_pair_moves_only_action_layer(
        &world,
        "baseline",
        "perturbed",
        entity,
        case.action_tags,
        &format!("{} reward perturbation", case.name),
    );

    assert_health_clean(&world);
}
