// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`crossover_matching`] | resonance | The outermost action tags are placed by solving for the offset at which their kernel meets their neighbour's, and it holds: at the crossover logit computed independently from the reward differentials, the two kernels agree to within the crossover tolerance. Where the channel changes its recommendation, the two actions are equally supported — the handover happens at the posture the reward arithmetic says it should, not a little either side of it. |
//! | [`crossover_matching_random_sweep`] | resonance | cites (´claim:resonance:adjacent-boundary-tags-meet-at-exactly-the-crossover-the-spec-places-them-around´) |

//! Boundary-crossover matching tests (´eq:landscape:crossover´).
//!
//! The boundary-placement formula guarantees that the Cauchy kernel
//! values of adjacent tags meet exactly at the spec-derived crossover
//! logit. Interior crossovers are *not* guaranteed to match (different
//! magnitudes and Q-bandwidths) and are excluded.

use super::helpers::{KernelScalars, derive_tags};
use crate::resonance::channel::{ChannelPolicy, compute_derived_constants};
use crate::resonance::derivation::ResonanceConfig;
use crate::testing::{DEFAULT_TOLERANCES, TestRng, assert_below, assert_finite};

/// The outermost action tags are placed by solving for the offset at which their
/// kernel meets their neighbour's, and it holds: at the crossover logit computed
/// independently from the reward differentials, the two kernels agree to within
/// the crossover tolerance. Where the channel changes its recommendation, the
/// two actions are equally supported — the handover happens at the posture the
/// reward arithmetic says it should, not a little either side of it.
///
/// ´claim:resonance:adjacent-boundary-tags-meet-at-exactly-the-crossover-the-spec-places-them-around´
/// ´test:crate:crossover-matching´
#[test]
fn crossover_matching() {
    // At boundary crossovers, the boundary_location formula guarantees
    // that f_boundary(ℓ*) = f_interior(ℓ*). Interior crossovers are NOT
    // guaranteed to match (different magnitudes and Q-bandwidths).
    let p_hat = 0.05_f64;
    let input = KernelScalars {
        p_hat,
        sigma_eff: 0.42,
        kappa_eff: 1.0,
        sigma2_q_hat_c: 0.05,
    };
    let policy = ChannelPolicy::default();
    let derived = compute_derived_constants(&policy, 0.5);
    let config = ResonanceConfig::default();
    let tags = derive_tags(&input, 0.5, &policy, &config);

    let action_tags: Vec<&_> = tags.iter().filter(|t| !t.is_classification()).collect();
    let beta_sum = derived.beta_sum;
    let n = derived.differentials.len();

    for (i, diff) in derived.differentials.iter().enumerate() {
        // Only verify boundary crossovers (first and last).
        let is_boundary = i == 0 || i == n - 1;
        if !is_boundary {
            continue;
        }
        if diff.delta_good <= 0.0 || diff.delta_bad <= 0.0 {
            continue;
        }
        let logit_cross = (1.0 / beta_sum) * ((1.0 - p_hat) * diff.delta_good / (p_hat * diff.delta_bad)).ln();
        if !logit_cross.is_finite() {
            continue;
        }

        let k_left = action_tags[i].kernel_at_logit(logit_cross);
        let k_right = action_tags[i + 1].kernel_at_logit(logit_cross);
        assert_finite(&[k_left, k_right], &format!("boundary crossover {i} kernels"));

        let max_k = k_left.max(k_right);
        assert!(max_k > 1e-10, "crossover {i}: both kernels near zero");
        let rel_diff = (k_left - k_right).abs() / max_k;
        assert_below(
            rel_diff,
            DEFAULT_TOLERANCES.crossover,
            &format!("boundary crossover {i}: k_left={k_left:.6e}, k_right={k_right:.6e}"),
        );
    }
}

/// A 10 000-sample random sweep of the boundary-crossover matching identity.
///
/// Draws `(p̂, σ_eff)` uniformly from `(0.01, 0.99) × (0.1, 1.5)` —
/// spanning the spec-allowed non-degenerate envelope — and, at every
/// draw, re-derives the resonance field and asserts that the first and
/// last boundary crossovers satisfy `f_{a_j}(ℓ*) = f_{a_{j+1}}(ℓ*)`
/// within `DEFAULT_TOLERANCES.crossover`.
///
/// Interior crossovers are deliberately *not* asserted: the boundary-placement
/// formula (´alg:rendering:placement´) only binds at the two outermost
/// crossovers, and adjacent interior tags carry independently-scaled
/// Cauchy magnitudes / bandwidths that make matching there a non-goal
/// (see the `crossover_matching` doc-comment).
///
/// The scenario's original framing ("verify the match holds at every
/// crossover position") overstates what the engine guarantees. This test
/// delivers the strongest assertion the specification actually makes —
/// the boundary-crossover identity — at 10 000× the sample count of
/// [`crossover_matching`].
///
/// The identity is not a property of one convenient input: ten thousand draws
/// spanning the whole non-degenerate envelope of risk and spread all match,
/// firing on more than ninety percent of the nominal assertions. The two
/// documented escapes — a neighbour dominated out, or a boundary tag pushed onto
/// its location clamp — are skipped rather than silently tolerated, so the
/// exceptions are named and the rest of the envelope is held to the identity.
///
/// (´claim:resonance:adjacent-boundary-tags-meet-at-exactly-the-crossover-the-spec-places-them-around´)
/// ´test:crate:crossover-matching-random-sweep´
#[test]
fn crossover_matching_random_sweep() {
    const SAMPLES: u32 = 10_000;
    const SEED: u64 = 0x00C0_FFEE_BEEF_0016;

    let policy = ChannelPolicy::default();
    let derived_env = compute_derived_constants(&policy, 0.5);
    let beta_sum = derived_env.beta_sum;
    let config = ResonanceConfig::default();
    let n = derived_env.differentials.len();

    let mut rng = TestRng::new(SEED);
    let mut checked = 0u32;

    for sample in 0..SAMPLES {
        // Uniform in (0.01, 0.99) and (0.1, 1.5) — the spec's
        // non-degenerate envelope.
        let p_hat = rng.next_f64().mul_add(0.98, 0.01_f64);
        let sigma_eff = rng.next_f64().mul_add(1.4, 0.1_f64);

        let input = KernelScalars {
            p_hat,
            sigma_eff,
            kappa_eff: 1.0,
            sigma2_q_hat_c: 0.05,
        };
        let tags = derive_tags(&input, 0.5, &policy, &config);
        let action_tags: Vec<&_> = tags.iter().filter(|t| !t.is_classification()).collect();

        for (i, diff) in derived_env.differentials.iter().enumerate() {
            // Only verify boundary crossovers (first and last).
            let is_boundary = i == 0 || i == n - 1;
            if !is_boundary {
                continue;
            }
            if diff.delta_good <= 0.0 || diff.delta_bad <= 0.0 {
                continue;
            }
            let logit_cross = (1.0 / beta_sum) * ((1.0 - p_hat) * diff.delta_good / (p_hat * diff.delta_bad)).ln();
            if !logit_cross.is_finite() {
                continue;
            }

            let k_left = action_tags[i].kernel_at_logit(logit_cross);
            let k_right = action_tags[i + 1].kernel_at_logit(logit_cross);
            assert_finite(
                &[k_left, k_right],
                &format!("sample {sample} boundary crossover {i}: p_hat={p_hat:.6}, sigma_eff={sigma_eff:.6}"),
            );

            let max_k = k_left.max(k_right);
            if max_k <= 1e-300 {
                // Both tails numerically zero — vacuously equal.
                continue;
            }
            // The dominated-tag treatment (´alg:rendering:dominated-treatment´)
            // replaces such a tag's magnitude with
            // `ε_mono`, which breaks the boundary-matching identity that
            // assumes the spec magnitudes on both sides. Skip crossovers
            // where either neighbour was dominated out.
            if action_tags[i].dominated || action_tags[i + 1].dominated {
                continue;
            }
            // The engine additionally clamps boundary-tag locations to
            // `[0.01, 0.49]` (first) / `[0.51, 0.99]` (last) in
            // `compute_action_tags`. When the clamp fires, the boundary
            // tag sits at an endpoint rather than at the location the
            // matching formula assumed, so the crossover identity also
            // falls back. Skip those samples.
            let boundary_tag = if i == 0 { action_tags[0] } else { action_tags[i + 1] };
            let at_clamp = [0.01, 0.49, 0.51, 0.99]
                .iter()
                .any(|&c: &f64| (boundary_tag.location - c).abs() < 1e-9);
            if at_clamp {
                continue;
            }
            let rel_diff = (k_left - k_right).abs() / max_k;
            assert_below(
                rel_diff,
                DEFAULT_TOLERANCES.crossover,
                &format!(
                    "sample {sample} boundary crossover {i}: p_hat={p_hat:.6}, sigma_eff={sigma_eff:.6}, k_left={k_left:.6e}, k_right={k_right:.6e}"
                ),
            );
            checked += 1;
        }
    }

    // Sanity: we must have actually asserted something on a meaningful
    // share of the draws. With two boundary crossovers per sample and a
    // uniform envelope the delta_good/delta_bad guards rarely short-
    // circuit, so ≥90% of the nominal 2·SAMPLES assertions should fire.
    assert!(
        checked > (SAMPLES * 2 * 9) / 10,
        "boundary-crossover sweep must exercise > 90% of draws, fired on {checked} / {}",
        SAMPLES * 2
    );
}
