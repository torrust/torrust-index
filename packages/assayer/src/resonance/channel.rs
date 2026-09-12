// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`channel_policy_refuses_every_non_finite_parameter`] | resonance | Every floating-point field a channel policy owns is refused when it is not finite, for a value that is not a number and for either infinity, and the refusal names the field that carried it. The reserved neutral-zone width is in the table with the rest: it is accepted today and read by nothing, but a policy that stores a value the derivation cannot use is a policy that will fail the moment the field is honoured. The eleven cases are enumerated from the policy's own fields rather than from the constraints below them, so a field no constraint mentions fails here instead of passing unexamined. |
//! | [`default_4action_differentials`] | resonance | A channel's differentials are adjacent differences of the per-action cost model, one per escalation step: four actions yield three. At default rewards the benign cost of escalating rises modestly through Allow→ Challenge→Slow and then jumps to 2.44 at Slow→Block, while the adversary's loss falls away — which is exactly why Block sits at the restrictive end and needs a strong posture to justify. |
//! | [`three_action_differentials`] | resonance | cites (´claim:resonance:the-reward-differentials-are-adjacent-differences-of-the-per-action-cost-model´) |
//! | [`d_delta_bad_d_q_analytical`] | resonance | Alongside each differential the channel carries its exact derivative with respect to challenge effectiveness, computed in closed form rather than by finite difference: 5.0 for Allow→Challenge and negative for the steps beyond it, since escalating past a challenge gives back some of what challenging had caught. The Q-bandwidth of every crossover is scaled by this sensitivity, so an approximation here would blur every placement. |
//! | [`neutral_zone_is_reserved_and_inert`] | resonance | The neutral-zone width is accepted on a policy for forward compatibility and reaches nothing: moving it from its default to 0.49 leaves the action list, the posture-sensitivity sum, and every differential bit-for-bit unchanged. A host may set the field today without any hidden effect on the channel it gets, which is what makes it safe to reserve rather than silently half-honour. |

//! Channel policy, reward parameters, and derived constants for derivation.
//!
//! # Cross-References
//!
//! - (´dec:derivation:per-call-constants´) — why the constants this module
//!   computes are derived on every call and never cached
//! - (´sig:landscape:derivation-function´) — the derivation the policy is an
//!   input to
//! - (´chap:spec:channel-policy´) — the channel policy chapter
//! - (´tab:landscape:action-costs´) — the per-action cost model
//! - (´data:channel:challenge-interaction´) — how the challenge estimate moves
//!   the crossover, which is why it enters the constants

use super::tags::Tag;
use crate::error::ChannelError;
pub use crate::types::Action;

/// Default reserved neutral-zone width (´def:channel:neutral-zone´).
///
/// ´const:assayer:indifference-band-width´ (´alg:const:scalar´)
/// ´const:assayer:indifference-band-width-scalar-0p1´
const DEFAULT_NEUTRAL_ZONE: f64 = 0.1;

// ═══════════════════════════════════════════════════════════════════════════════
// Action → Tag Mapping
// ═══════════════════════════════════════════════════════════════════════════════

/// Converts an [`Action`] to its corresponding resonance [`Tag`].
#[must_use]
pub const fn action_to_tag(action: Action) -> Tag {
    match action {
        Action::Allow => Tag::Allow,
        Action::Challenge => Tag::Challenge,
        Action::Slow => Tag::Slow,
        Action::Block => Tag::Block,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Reward Parameters
// ═══════════════════════════════════════════════════════════════════════════════

/// Extended reward parameters for a channel (´def:landscape:extended-rewards´).
///
/// The host declares them per channel (´tab:channel:reward-parameters´).
#[derive(Clone, Debug)]
pub struct RewardParameters {
    /// `R_p`: opportunity cost of full denial.
    pub pass: f64,
    /// `R_f`: cost of challenge to benign cases.
    pub friction: f64,
    /// `R_m`: cost when adverse outcome is not prevented.
    pub missed: f64,
    /// `R_c`: value of identifying adverse source.
    pub caught: f64,
    /// `R_b`: cost of denial to benign cases.
    pub blocked: f64,
    /// `α_s ∈ (0, 1)`: throttle severity.
    pub slow_severity: f64,
    /// `β_c ∈ [0, 1]`: fraction of challenge catching retained under Block.
    pub block_catches: f64,
    /// `β_b`: bad posture-sensitivity exponent.
    pub beta_bad: f64,
    /// `β_g`: good posture-sensitivity exponent.
    pub beta_good: f64,
    /// `γ_{q,t}`: challenge-effectiveness time-decay (hourly). Default: 0.9998.
    /// Carried in the reward parameters but owned by the Companion
    /// (´def:companion:challenge-decay´).
    pub gamma_q_t: f64,
}

impl Default for RewardParameters {
    fn default() -> Self {
        Self {
            pass: 1.0,
            friction: 0.3,
            missed: 3.0,
            caught: 2.0,
            blocked: 1.5,
            slow_severity: 0.2,
            block_catches: 1.0,
            beta_bad: 1.5,
            beta_good: 1.0,
            gamma_q_t: 0.9998,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Channel Policy
// ═══════════════════════════════════════════════════════════════════════════════

/// Host-supplied policy controlling derivation for one channel.
///
/// The Core never stores or reads this policy. Hosts pass it explicitly to
/// `derive_reckoning()` for each derivation (´sig:landscape:derivation-function´).
/// The reserved `neutral_zone` field is accepted for forward compatibility but
/// is inert in the current derivation (´def:channel:neutral-zone´).
#[derive(Clone, Debug)]
pub struct ChannelPolicy {
    /// Ordered action set (least → most restrictive).
    pub actions: Vec<Action>,
    /// Reward parameters for this channel.
    pub reward: RewardParameters,
    /// Reserved Suspicious-shaping width.
    ///
    /// Accepted for forward compatibility; not read by the current derivation.
    pub neutral_zone: f64,
}

impl Default for ChannelPolicy {
    fn default() -> Self {
        Self {
            actions: vec![Action::Allow, Action::Challenge, Action::Slow, Action::Block],
            reward: RewardParameters::default(),
            neutral_zone: DEFAULT_NEUTRAL_ZONE,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Derived Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// A reward differential between adjacent actions.
#[derive(Clone, Debug)]
pub struct RewardDifferential {
    /// `Δ_good`: additional benign-class cost.
    pub delta_good: f64,
    /// `Δ_bad`: adversary benefit from escalation.
    pub delta_bad: f64,
    /// `∂Δ_bad / ∂q̂_c`: sensitivity of `Δ_bad` to `q̂_c`.
    /// Used for per-crossover Q-bandwidth computation (´def:rendering:bandwidths´).
    pub d_delta_bad_d_q: f64,
}

/// Constants derived from channel policy and challenge effectiveness.
///
/// Computed by [`compute_derived_constants`] when deriving a reckoning.
#[derive(Clone, Debug)]
pub struct ChannelDerivedConstants {
    /// Ordered action list.
    pub actions: Vec<Action>,
    /// `β_b + β_g` (posture-sensitivity sum).
    // TODO ´todo:code:the-adr-pseudocode-now-stores-inv´: The ADR pseudocode now stores `inv_beta_sum` directly.
    // Either switch this field to the reciprocal or update the ADR to document
    // the implementation's `beta_sum` representation as the canonical API.
    pub beta_sum: f64,
    /// Per-transition reward differentials (`n_actions - 1` entries).
    pub differentials: Vec<RewardDifferential>,
}

/// Computes [`ChannelDerivedConstants`] from a [`ChannelPolicy`] at the
/// given challenge effectiveness estimate `q̂_c`.
#[must_use]
pub fn compute_derived_constants(policy: &ChannelPolicy, q_hat_c: f64) -> ChannelDerivedConstants {
    let actions = &policy.actions;
    let n = actions.len();
    // A hard assertion, not a debug one: a malformed action set is the
    // caller's defect and fails hard rather than degrading, in every
    // build profile (´dec:derivation:ordered-actions´).
    assert!(n >= 2, "action set must have at least 2 actions");

    let r = &policy.reward;
    let beta_sum = r.beta_bad + r.beta_good;

    // Per-action benign and adverse costs (´tab:landscape:action-costs´)
    let benign_cost = |a: Action| -> f64 {
        match a {
            Action::Allow => 0.0,
            Action::Challenge => r.friction,
            Action::Slow => r.friction.mul_add(1.0 + r.slow_severity, 0.0),
            Action::Block => r.block_catches.mul_add(r.friction, r.blocked + r.pass),
        }
    };

    let adverse_cost = |a: Action, q: f64| -> f64 {
        match a {
            Action::Allow => r.missed,
            Action::Challenge => (1.0 - q).mul_add(r.missed, -(q * r.caught)),
            Action::Slow => (1.0 - q).mul_add((1.0 - r.slow_severity) * r.missed, -(q * r.caught)),
            Action::Block => -(r.block_catches * q * r.caught),
        }
    };

    // Analytical derivative of adverse cost w.r.t. q̂_c
    // (´dec:derivation:per-call-constants´).
    // Constant per action — does not depend on q̂_c.
    let adverse_cost_dqc = |a: Action| -> f64 {
        match a {
            Action::Allow => 0.0,
            Action::Challenge => -(r.missed + r.caught),
            Action::Slow => -((1.0_f64 - r.slow_severity).mul_add(r.missed, r.caught)),
            Action::Block => -(r.block_catches * r.caught),
        }
    };

    // Compute differentials and their q̂_c derivatives
    // (´def:landscape:reward-differentials´)
    let mut differentials = Vec::with_capacity(n - 1);

    for i in 0..n - 1 {
        let delta_good = benign_cost(actions[i + 1]) - benign_cost(actions[i]);
        let delta_bad = adverse_cost(actions[i], q_hat_c) - adverse_cost(actions[i + 1], q_hat_c);
        let d_delta_bad_d_q = adverse_cost_dqc(actions[i]) - adverse_cost_dqc(actions[i + 1]);

        differentials.push(RewardDifferential {
            delta_good,
            delta_bad,
            d_delta_bad_d_q,
        });
    }

    ChannelDerivedConstants {
        actions: actions.clone(),
        beta_sum,
        differentials,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Channel Policy Validation (´dec:derivation:ordered-actions´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Validates a channel policy, returning `Err` on any constraint violation.
///
/// Called eagerly by host-side channel registries — malformed
/// channel policies fail at declaration time, not at `build()`.
///
/// # Errors
///
/// Returns `ChannelError` if the policy violates any constraint:
/// - Fewer than 2 actions
/// - Actions not strictly ordered by severity
/// - Any floating-point parameter that is not finite, including the reserved
///   neutral-zone width, named by its field
/// - Non-positive reward parameters
/// - `slow_severity` outside (0, 1)
/// - `block_catches` outside [0, 1]
/// - Non-positive sensitivity exponents
/// - `gamma_q_t` outside (0, 1)
pub fn validate_channel_policy(policy: &ChannelPolicy) -> Result<(), ChannelError> {
    // At least 2 actions required.
    if policy.actions.len() < 2 {
        return Err(ChannelError::TooFewActions);
    }

    // Actions must be strictly ordered (ascending severity).
    // types::Action derives Ord with the canonical restrictiveness ordering.
    for window in policy.actions.windows(2) {
        if window[0] >= window[1] {
            return Err(ChannelError::ActionsNotOrdered);
        }
    }

    let r = &policy.reward;

    // Every constraint below is an ordered comparison, and every ordered
    // comparison with a value that is not a number is false: such a value
    // satisfies both ends of an interval at once, so it breaks no bound and
    // would be reported as a valid policy. It then reaches
    // `compute_derived_constants`, where it turns the posture-sensitivity sum
    // and every reward differential into a value that is not a number — a
    // channel whose whole rendered field is unusable, with nothing in the
    // registration that objected. Finiteness is therefore tested first, for
    // every floating-point field the policy owns including the reserved
    // width, and the refusal names the field and what it held.
    for (field, value) in [
        ("reward.pass", r.pass),
        ("reward.friction", r.friction),
        ("reward.missed", r.missed),
        ("reward.caught", r.caught),
        ("reward.blocked", r.blocked),
        ("reward.slow_severity", r.slow_severity),
        ("reward.block_catches", r.block_catches),
        ("reward.beta_bad", r.beta_bad),
        ("reward.beta_good", r.beta_good),
        ("reward.gamma_q_t", r.gamma_q_t),
        ("neutral_zone", policy.neutral_zone),
    ] {
        if !value.is_finite() {
            return Err(ChannelError::NonFiniteParameter { field, value });
        }
    }

    // All reward parameters must be positive.
    if r.pass <= 0.0 || r.friction <= 0.0 || r.missed <= 0.0 || r.caught <= 0.0 || r.blocked <= 0.0 {
        return Err(ChannelError::NonPositiveReward);
    }

    // Slow severity must be in (0, 1).
    if r.slow_severity <= 0.0 || r.slow_severity >= 1.0 {
        return Err(ChannelError::InvalidSlowSeverity);
    }

    // Block catches must be in [0, 1].
    if !(0.0..=1.0).contains(&r.block_catches) {
        return Err(ChannelError::InvalidBlockCatches);
    }

    // Sensitivity exponents must be positive.
    if r.beta_bad <= 0.0 || r.beta_good <= 0.0 {
        return Err(ChannelError::NonPositiveSensitivity);
    }

    // Challenge-effectiveness decay rate must be in (0, 1)
    // (´def:companion:challenge-decay´).
    if r.gamma_q_t <= 0.0 || r.gamma_q_t >= 1.0 {
        return Err(ChannelError::InvalidGammaQt);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every floating-point field a channel policy owns is refused when it is
    /// not finite, for a value that is not a number and for either infinity,
    /// and the refusal names the field that carried it. The reserved
    /// neutral-zone width is in the table with the rest: it is accepted today
    /// and read by nothing, but a policy that stores a value the derivation
    /// cannot use is a policy that will fail the moment the field is honoured.
    ///
    /// The eleven cases are enumerated from the policy's own fields rather
    /// than from the constraints below them, so a field no constraint mentions
    /// fails here instead of passing unexamined.
    ///
    /// ´claim:resonance:every-channel-policy-float-is-refused-when-it-is-not-finite-and-the-refusal-names-it´
    /// ´test:unit:channel-policy-refuses-every-non-finite-parameter´
    #[test]
    fn channel_policy_refuses_every_non_finite_parameter() {
        /// One row of the enumeration: the policy field the refusal has to
        /// name, and the setter that plants a value in it.
        type PolicyFloatField = (&'static str, fn(&mut ChannelPolicy, f64));

        let fields: [PolicyFloatField; 11] = [
            ("reward.pass", |p, v| p.reward.pass = v),
            ("reward.friction", |p, v| p.reward.friction = v),
            ("reward.missed", |p, v| p.reward.missed = v),
            ("reward.caught", |p, v| p.reward.caught = v),
            ("reward.blocked", |p, v| p.reward.blocked = v),
            ("reward.slow_severity", |p, v| p.reward.slow_severity = v),
            ("reward.block_catches", |p, v| p.reward.block_catches = v),
            ("reward.beta_bad", |p, v| p.reward.beta_bad = v),
            ("reward.beta_good", |p, v| p.reward.beta_good = v),
            ("reward.gamma_q_t", |p, v| p.reward.gamma_q_t = v),
            ("neutral_zone", |p, v| p.neutral_zone = v),
        ];

        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for (field, set) in fields {
                let mut policy = ChannelPolicy::default();
                set(&mut policy, value);
                match validate_channel_policy(&policy) {
                    Err(ChannelError::NonFiniteParameter {
                        field: named,
                        value: held,
                    }) => {
                        assert_eq!(named, field, "the refusal names the field that carried the value");
                        assert!(!held.is_finite(), "the refusal carries the value the field held");
                    }
                    other => panic!("expected a non-finite refusal at {field}, got {other:?}"),
                }
            }
        }
    }

    /// A channel's differentials are adjacent differences of the per-action cost
    /// model, one per escalation step: four actions yield three. At default
    /// rewards the benign cost of escalating rises modestly through Allow→
    /// Challenge→Slow and then jumps to 2.44 at Slow→Block, while the adversary's
    /// loss falls away — which is exactly why Block sits at the restrictive end
    /// and needs a strong posture to justify.
    ///
    /// ´claim:resonance:the-reward-differentials-are-adjacent-differences-of-the-per-action-cost-model´
    /// ´test:unit:default-4action-differentials´
    #[test]
    fn default_4action_differentials() {
        let policy = ChannelPolicy::default();
        let dc = compute_derived_constants(&policy, 0.5);

        assert_eq!(dc.actions.len(), 4);
        assert_eq!(dc.differentials.len(), 3);

        // A→C: Δ_good = 0.3, Δ_bad = 2.5
        assert!((dc.differentials[0].delta_good - 0.30).abs() < 1e-10);
        assert!((dc.differentials[0].delta_bad - 2.50).abs() < 1e-10);

        // C→S: Δ_good = 0.06, Δ_bad = 0.3
        assert!((dc.differentials[1].delta_good - 0.06).abs() < 1e-10);
        assert!((dc.differentials[1].delta_bad - 0.30).abs() < 1e-10);

        // S→B: Δ_good = 2.44, Δ_bad = 1.2
        assert!((dc.differentials[2].delta_good - 2.44).abs() < 1e-10);
        assert!((dc.differentials[2].delta_bad - 1.20).abs() < 1e-10);
    }

    /// Dropping an action from the middle of the ladder does not perturb the
    /// steps that did not touch it: Allow→Challenge keeps the same pair of
    /// differentials it had with four actions, while the two steps that ran
    /// through Slow fuse into a single Challenge→Block step whose costs are their
    /// composition. Each differential depends only on the two actions it spans,
    /// so a host may shorten its menu without re-deriving the rest.
    ///
    /// (´claim:resonance:the-reward-differentials-are-adjacent-differences-of-the-per-action-cost-model´)
    /// ´test:unit:three-action-differentials´
    #[test]
    fn three_action_differentials() {
        let policy = ChannelPolicy {
            actions: vec![Action::Allow, Action::Challenge, Action::Block],
            reward: RewardParameters::default(),
            ..ChannelPolicy::default()
        };
        let dc = compute_derived_constants(&policy, 0.5);

        assert_eq!(dc.actions.len(), 3);
        assert_eq!(dc.differentials.len(), 2);

        // A→C: same as 4-action
        assert!((dc.differentials[0].delta_good - 0.30).abs() < 1e-10);
        assert!((dc.differentials[0].delta_bad - 2.50).abs() < 1e-10);

        // C→B: Δ_good = 2.5, Δ_bad = 1.5
        assert!((dc.differentials[1].delta_good - 2.50).abs() < 1e-10);
        assert!((dc.differentials[1].delta_bad - 1.50).abs() < 1e-10);
    }

    /// Alongside each differential the channel carries its exact derivative with
    /// respect to challenge effectiveness, computed in closed form rather than by
    /// finite difference: 5.0 for Allow→Challenge and negative for the steps
    /// beyond it, since escalating past a challenge gives back some of what
    /// challenging had caught. The Q-bandwidth of every crossover is scaled by
    /// this sensitivity, so an approximation here would blur every placement.
    ///
    /// ´claim:resonance:the-sensitivity-of-each-differential-to-challenge-effectiveness-is-carried-in-closed-form´
    /// ´test:unit:d-delta-bad-d-q-analytical´
    #[test]
    fn d_delta_bad_d_q_analytical() {
        let policy = ChannelPolicy::default();
        let dc = compute_derived_constants(&policy, 0.5);

        // A→C: ∂Δ_bad/∂q = R_m + R_c = 5.0
        assert!(
            (dc.differentials[0].d_delta_bad_d_q - 5.0).abs() < 1e-10,
            "A→C: got {}",
            dc.differentials[0].d_delta_bad_d_q
        );

        // C→S: ∂Δ_bad/∂q = -α_s·R_m = -0.6
        assert!(
            (dc.differentials[1].d_delta_bad_d_q - (-0.6)).abs() < 1e-10,
            "C→S: got {}",
            dc.differentials[1].d_delta_bad_d_q
        );

        // S→B at β_c=1: ∂Δ_bad/∂q = -(1-α_s)·R_m = -2.4
        assert!(
            (dc.differentials[2].d_delta_bad_d_q - (-2.4)).abs() < 1e-10,
            "S→B: got {}",
            dc.differentials[2].d_delta_bad_d_q
        );
    }

    /// The neutral-zone width is accepted on a policy for forward compatibility
    /// and reaches nothing: moving it from its default to 0.49 leaves the action
    /// list, the posture-sensitivity sum, and every differential bit-for-bit
    /// unchanged. A host may set the field today without any hidden effect on
    /// the channel it gets, which is what makes it safe to reserve rather than
    /// silently half-honour.
    ///
    /// ´claim:resonance:the-reserved-neutral-zone-is-accepted-but-reaches-nothing-in-the-derived-constants´
    /// ´test:unit:neutral-zone-is-reserved-and-inert´
    #[test]
    fn neutral_zone_is_reserved_and_inert() {
        let baseline = ChannelPolicy::default();
        let mut shifted = baseline.clone();
        shifted.neutral_zone = 0.49;

        let baseline_constants = compute_derived_constants(&baseline, 0.5);
        let shifted_constants = compute_derived_constants(&shifted, 0.5);

        assert_eq!(baseline_constants.actions, shifted_constants.actions);
        assert_eq!(baseline_constants.differentials.len(), shifted_constants.differentials.len());
        assert!((baseline_constants.beta_sum - shifted_constants.beta_sum).abs() < f64::EPSILON);
        for (left, right) in baseline_constants
            .differentials
            .iter()
            .zip(shifted_constants.differentials.iter())
        {
            assert!((left.delta_good - right.delta_good).abs() < f64::EPSILON);
            assert!((left.delta_bad - right.delta_bad).abs() < f64::EPSILON);
            assert!((left.d_delta_bad_d_q - right.d_delta_bad_d_q).abs() < f64::EPSILON);
        }
    }
}
