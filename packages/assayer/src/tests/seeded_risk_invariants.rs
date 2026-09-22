// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`challenge_conjugacy_seeded_sweep`] | risk | cites (´claim:risk:a-challenge-outcome-adds-one-count-to-its-own-side-and-leaves-the-other-untouched´) |
//! | [`blend_variance_non_negative_seeded_sweep`] | risk | cites (´claim:risk:the-three-term-mixture-variance-is-non-negative-across-the-whole-input-range´) |
//! | [`blend_component_diagnostics_seeded_sweep`] | risk | cites (´claim:risk:the-reported-component-variances-recompose-into-the-blended-variance-and-respect-the-subspace-ordering´) |
//! | [`blend_mixture_identity_seeded_sweep`] | risk | cites (´claim:risk:the-blended-variance-is-exactly-the-variance-of-the-two-component-mixture´) |

//! Seeded packs for the Companion's conjugate update and the risk blend's subspace and variance identities.

use crate::model::parameters::{FloorMass, ModelParameters};
use crate::risk::blend::{BlendResult, compute_blend};
use crate::risk::challenge::ChallengeEffectivenessState;
use crate::testing::run_seeded_sweep;
use crate::types::{ChallengeResult, PersistentTimestamp};

#[derive(Debug)]
struct ChallengeConjugacyCase {
    alpha: f64,
    beta: f64,
    failed: bool,
}

#[derive(Debug)]
struct BlendCase {
    sister_mean: Vec<f64>,
    anchor_mean: Vec<f64>,
    sister_diagonal: Vec<f64>,
    anchor_diagonal: Vec<f64>,
    shared_features: Vec<f64>,
    excluded_features: Vec<f64>,
    replacement_excluded_features: Vec<f64>,
    time_correction: f64,
}

fn diagonal_parameters(mean: &[f64], diagonal: &[f64]) -> ModelParameters {
    let width = mean.len();
    let mut covariance_data = vec![0.0; width * width];
    for (index, &value) in diagonal.iter().enumerate() {
        covariance_data[index * width + index] = value;
    }
    ModelParameters {
        mu: mean.to_vec(),
        covariance_data,
        p: width,
    }
}

fn close(actual: f64, expected: f64, tolerance: f64, quantity: &str) -> Result<(), String> {
    let error = (actual - expected).abs();
    if error > tolerance {
        Err(format!(
            "{quantity} differs: actual={actual}, expected={expected}, error={error}"
        ))
    } else {
        Ok(())
    }
}

fn evaluate_blend(case: &BlendCase, replace_excluded: bool) -> BlendResult {
    const FULL_WIDTH: usize = 6;
    const SHARED_WIDTH: usize = 3;

    let sister = diagonal_parameters(&case.sister_mean, &case.sister_diagonal);
    let anchor = diagonal_parameters(&case.anchor_mean, &case.anchor_diagonal);
    let indices = [Some(0), Some(1), Some(2)];
    let excluded = if replace_excluded {
        &case.replacement_excluded_features
    } else {
        &case.excluded_features
    };
    let mut features = case.shared_features.clone();
    features.extend_from_slice(excluded);
    compute_blend(
        &sister,
        &anchor,
        &FloorMass::new(0.0, vec![0.0; FULL_WIDTH]),
        &FloorMass::new(0.0, vec![0.0; SHARED_WIDTH]),
        &features,
        &case.shared_features,
        &indices,
        case.time_correction,
        1.0,
        1.0,
    )
}

fn draw_blend_case(rng: &mut crate::testing::TestRng) -> BlendCase {
    const FULL_WIDTH: usize = 6;
    const SHARED_WIDTH: usize = 3;

    BlendCase {
        sister_mean: (0..FULL_WIDTH).map(|_| rng.next_f64().mul_add(2.0, -1.0)).collect(),
        anchor_mean: (0..SHARED_WIDTH).map(|_| rng.next_f64().mul_add(2.0, -1.0)).collect(),
        sister_diagonal: (0..FULL_WIDTH).map(|_| rng.next_f64().mul_add(2.9, 0.1)).collect(),
        anchor_diagonal: (0..SHARED_WIDTH).map(|_| rng.next_f64().mul_add(1.9, 0.1)).collect(),
        shared_features: (0..SHARED_WIDTH).map(|_| rng.next_f64().mul_add(2.0, -1.0)).collect(),
        excluded_features: (0..FULL_WIDTH - SHARED_WIDTH)
            .map(|_| rng.next_f64().mul_add(2.0, -1.0))
            .collect(),
        replacement_excluded_features: (0..FULL_WIDTH - SHARED_WIDTH)
            .map(|_| rng.next_f64().mul_add(2.0, -1.0))
            .collect(),
        time_correction: rng.next_f64().mul_add(2.0, 1.0),
    }
}

fn blend_variance_oracles(case: &BlendCase) -> (f64, f64, f64) {
    let mut features = case.shared_features.clone();
    features.extend_from_slice(&case.excluded_features);
    let inherited = features
        .iter()
        .zip(&case.sister_diagonal)
        .map(|(&feature, &variance)| variance * feature * feature)
        .sum::<f64>()
        * case.time_correction;
    let inherited_shared = case
        .shared_features
        .iter()
        .zip(&case.sister_diagonal)
        .map(|(&feature, &variance)| variance * feature * feature)
        .sum::<f64>()
        * case.time_correction;
    let anchor = case
        .shared_features
        .iter()
        .zip(&case.anchor_diagonal)
        .map(|(&feature, &variance)| variance * feature * feature)
        .sum::<f64>()
        * case.time_correction;
    (inherited, inherited_shared, anchor)
}

/// One observed outcome adds one exact pseudo-count to its own side of any valid Beta state and leaves the opposite side bit-identical.
///
/// The generator draws 459 positive alpha-beta pairs from `[0.1, 20.1)` and both challenge outcomes. Each update uses the state's own timestamp, isolating conjugate count arithmetic from elapsed-time decay. The count gives more than a ninety-nine per cent chance of entering any defect region occupying at least one per cent of this distribution.
///
/// (´inv:guarantee:conjugacy´)
/// (´claim:risk:a-challenge-outcome-adds-one-count-to-its-own-side-and-leaves-the-other-untouched´)
/// ´test:crate:challenge-conjugacy-seeded-sweep´
#[test]
fn challenge_conjugacy_seeded_sweep() {
    const CASES: usize = 459;
    const SEED: u64 = 0xC0A0_6A7E_B37A_0459;

    run_seeded_sweep(
        SEED,
        CASES,
        |rng| ChallengeConjugacyCase {
            alpha: rng.next_f64().mul_add(20.0, 0.1),
            beta: rng.next_f64().mul_add(20.0, 0.1),
            failed: rng.coin(),
        },
        |case| {
            let mut state =
                ChallengeEffectivenessState::new(case.alpha, case.beta, 0.9998, PersistentTimestamp::new(1_700_000_000, 0));
            let now = state.t_last;
            let result = if case.failed {
                ChallengeResult::Fail
            } else {
                ChallengeResult::Pass
            };
            state.update(result, &now);
            let expected_alpha = case.alpha + if case.failed { 1.0 } else { 0.0 };
            let expected_beta = case.beta + if case.failed { 0.0 } else { 1.0 };
            if state.alpha.to_bits() != expected_alpha.to_bits() || state.beta.to_bits() != expected_beta.to_bits() {
                return Err(format!(
                    "conjugate counts are alpha={}, beta={}, expected alpha={expected_alpha}, beta={expected_beta}",
                    state.alpha, state.beta
                ));
            }
            Ok(())
        },
    );
}

/// The effective three-term mixture variance remains non-negative across the generated input space.
///
/// The generator draws 459 complete six-coordinate sister and three-coordinate anchor means, positive diagonal covariances, shared and excluded feature vectors over `[−1, 1)`, an independent replacement excluded vector, and a time correction from `[1, 3)`. Diagonal matrices make every oracle an explicit sum that imports no blend implementation. The count gives more than a ninety-nine per cent chance of entering any defect region occupying at least one per cent of this distribution.
///
/// (´inv:guarantee:blend-variance´)
/// (´claim:risk:the-three-term-mixture-variance-is-non-negative-across-the-whole-input-range´)
/// ´test:crate:blend-variance-non-negative-seeded-sweep´
#[test]
fn blend_variance_non_negative_seeded_sweep() {
    const CASES: usize = 459;
    const SEED: u64 = 0xB1E0_D5AB_A0A0_0459;

    run_seeded_sweep(SEED, CASES, draw_blend_case, |case| {
        let result = evaluate_blend(case, false);
        if result.variance_effective < 0.0 {
            return Err(format!("mixture variance is negative: {}", result.variance_effective));
        }
        Ok(())
    });
}

/// The reported component variances equal independent diagonal quadratic forms, recompose into the effective variance, and keep the shared sister form no greater than the full form; excluded-coordinate changes cannot move the subspace weight.
///
/// The generator draws 459 complete six-coordinate sister and three-coordinate anchor means, positive diagonal covariances, shared and excluded feature vectors over `[−1, 1)`, an independent replacement excluded vector, and a time correction from `[1, 3)`. Diagonal matrices make every oracle an explicit sum that imports no blend implementation. The count gives more than a ninety-nine per cent chance of entering any defect region occupying at least one per cent of this distribution.
///
/// (´inv:guarantee:blend-subspace´)
/// (´claim:risk:the-reported-component-variances-recompose-into-the-blended-variance-and-respect-the-subspace-ordering´)
/// ´test:crate:blend-component-diagnostics-seeded-sweep´
#[test]
fn blend_component_diagnostics_seeded_sweep() {
    const CASES: usize = 459;
    const SEED: u64 = 0xB1E0_D5AB_5AB5_0459;

    run_seeded_sweep(SEED, CASES, draw_blend_case, |case| {
        let result = evaluate_blend(case, false);
        let replaced = evaluate_blend(case, true);
        if result.anchor_weight.to_bits() != replaced.anchor_weight.to_bits() {
            return Err(format!(
                "excluded coordinates moved anchor weight from {} to {}",
                result.anchor_weight, replaced.anchor_weight
            ));
        }
        let (inherited, inherited_shared, anchor) = blend_variance_oracles(case);
        close(result.variance_inherited, inherited, 1e-12, "inherited variance")?;
        close(
            result.variance_inherited_shared,
            inherited_shared,
            1e-12,
            "shared inherited variance",
        )?;
        close(result.variance_anchor, anchor, 1e-12, "anchor variance")?;
        if result.variance_inherited_shared > result.variance_inherited + 1e-12 {
            return Err(format!(
                "shared variance {} exceeds full variance {}",
                result.variance_inherited_shared, result.variance_inherited
            ));
        }
        let disagreement = result.rho_inherited - result.rho_anchor;
        let weight = result.anchor_weight;
        let base = (1.0 - weight).mul_add(inherited, weight * anchor);
        let expected = (weight * (1.0 - weight) * disagreement).mul_add(disagreement, base);
        close(result.variance_effective, expected, 1e-12, "recomposed variance")
    });
}

/// The effective variance equals the independent second-moment identity for the two component distribution.
///
/// The generator draws 459 complete six-coordinate sister and three-coordinate anchor means, positive diagonal covariances, shared and excluded feature vectors over `[−1, 1)`, an independent replacement excluded vector, and a time correction from `[1, 3)`. Diagonal matrices make every oracle an explicit sum that imports no blend implementation. The count gives more than a ninety-nine per cent chance of entering any defect region occupying at least one per cent of this distribution.
///
/// (´inv:guarantee:blend-variance´)
/// (´claim:risk:the-blended-variance-is-exactly-the-variance-of-the-two-component-mixture´)
/// ´test:crate:blend-mixture-identity-seeded-sweep´
#[test]
fn blend_mixture_identity_seeded_sweep() {
    const CASES: usize = 459;
    const SEED: u64 = 0xB1E0_D5AB_5AFE_0459;

    run_seeded_sweep(SEED, CASES, draw_blend_case, |case| {
        let result = evaluate_blend(case, false);
        let (inherited, _, anchor) = blend_variance_oracles(case);
        let disagreement = result.rho_inherited - result.rho_anchor;
        let weight = result.anchor_weight;
        let base = (1.0 - weight).mul_add(inherited, weight * anchor);
        let expected = (weight * (1.0 - weight) * disagreement).mul_add(disagreement, base);
        close(result.variance_effective, expected, 1e-12, "mixture variance")
    });
}
