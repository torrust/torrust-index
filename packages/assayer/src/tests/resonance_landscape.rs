// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`low_risk_landscape_matches_the_worked_tables`] | resonance | The low-risk worked landscape comes back figure for figure: the shared term at 1.178 with its 0.168 uncertainty, three crossovers at the tabulated logits and postures, every variance at 0.0816, and each credible interval matching the quantile push-through the specification tabulates to two decimals (´ex:landscape:low-risk´). The worked chapter is the numeric acceptance oracle for the whole derivation, so an implementation that reproduces its tables has the decomposition, the covariance and the intervals right at once. |
//! | [`regime_table_is_risk_independent`] | resonance | Recomputing the landscape at the high-risk and medium-risk bases leaves every offset, every width and every width variance bit-for-bit identical to the low-risk case: only the shared term moved, and all three crossovers moved with it by exactly that difference (´thm:landscape:rigid-translation´). Regime structure belongs to the policy and the posterior, so no risk update can change what the regimes are, only where they sit. |
//! | [`three_action_landscape_matches_the_worked_tables`] | resonance | Withdrawing Slow re-derives the Challenge-to-Block transition from the surviving pair's own cost tables rather than summing the two steps it replaced: the offset lands at +0.204, Challenge's regime widens from a 0.204 sliver to 1.052, and the width's standard deviation stays at 0.462 because it depends only on the difference of adjacent sensitivities (´ex:landscape:three-action´). The landscape keeps one crossover per surviving adjacent pair, so shortening the menu changes values and never structure. |
//! | [`landscape_shape_is_stable`] | resonance | A landscape for J declared actions always carries exactly J−1 crossovers and J regimes, the two boundary regimes carrying no width, whatever the evidence says about dominance (´sig:landscape:output´). Shape stability is what makes a landscape safe to store, replay and difference across a policy comparison — a Companion update can move every number and can never add or remove a record. |
//! | [`derivation_replays_bit_for_bit`] | resonance | Deriving twice from one argument triple gives two landscapes whose every scalar agrees to the bit, shared term through interval endpoints. The transform holds no state and reads no clock (´pf:landscape:purity´), so a host that stored the assessment, the policy and the posterior stored the complete determinant of the landscape. |
//! | [`forfeiture_sends_the_crossover_to_infinity`] | resonance | Below full block-catching, a posterior mean past the forfeiture threshold drives the Slow-to-Block adverse differential non-positive and the crossover to positive infinity, with the interval's upper endpoint following it (´thm:landscape:catching-forfeiture´): blocking has stopped paying at any posture, and the landscape reports the fact structurally instead of clamping it to a large number. |
//! | [`optimal_action_reads_the_envelope`] | resonance | The optimal-action utility walks the low-risk worked landscape regime by regime — Allow at Normal, Challenge just past its lower crossover, Slow and Block in theirs — and hands over within a hair's breadth of the crossover, the tie itself breaking toward the more permissive action so the reading is left-continuous in the posture (´alg:landscape:optimal-action´). The utility reads the landscape and a posture and nothing else, which is what makes one landscape answer every operating point a host cares to probe (´sig:landscape:utilities´). |
//! | [`envelope_skips_a_dominated_action`] | resonance | At a challenge estimate low enough to invert Challenge's crossovers, a posture inside the inverted span answers Allow from the cost-curve envelope where walking the crossover vector in order would have returned the dominated action (´alg:landscape:optimal-action´). The bracketing shortcut fails structurally under inversion — the crossovers bracket nothing (´thm:landscape:dominance´) — and the envelope needs no special case to exclude an action that never attains the minimum. |
//! | [`fragility_matches_the_worked_readings`] | resonance | Fragility at the worked chapter's probes returns the chapter's own figures (´ex:landscape:low-risk´), (´ex:landscape:high-risk´): near-zero flip at Normal on the low-risk landscape with the boundary 4.1 standard deviations away, flip about 0.72 with a challenge share about 0.65 at posture 0.6, and the high-risk landscape's modal Slow at Normal. The two shares split the bounding variance by evidence source (´def:fragility:definition´), which is the reading no single component can produce. |
//! | [`fragility_uses_envelope_boundaries_under_inversion`] | resonance | Inside an inverted region the fragility reading is taken against the crossings that bound the optimal regime from the envelope — here a compound Allow-to-Slow crossing no single crossover record carries — rather than the raw adjacent records (´def:fragility:definition´). The modal action, the flip probability and the shares all stay well-formed where the naive bracketing reading has nothing coherent to offer. |
//! | [`utilities_are_total_over_the_posture_axis`] | resonance | Across a grid of postures spanning the open unit interval and landscapes spanning healthy, inverted and forfeited regimes, both utilities always answer (´sig:landscape:utilities´): the optimal action is one of the declared set, the flip probability stays inside the unit interval, and the two shares are non-negative and sum to one wherever a boundary exists. Totality is the utilities' contract — every posture has an optimal action and a fragility. |
//! | [`conjugate_counts_cross_the_boundary_whole`] | resonance | The Companion's pseudo-counts arrive in the landscape as themselves: after one observed challenge failure on uniform priors the tracker holds Beta(2, 1), the carrier the harness path hands the derivation is the conjugate variant with exactly those counts, and the credible intervals equal the ones computed from Beta(2, 1) quantiles directly (´def:landscape:credible-intervals´). The conjugate structure survives the crossing instead of being flattened to two moments one call before the consumers that need it (´sig:companion:posterior´). |
//! | [`moment_pair_crosses_at_matched_fidelity`] | resonance | A bare mean-and-variance pair from outside the crate is admitted, not refused: a pair carrying a Beta's own moments derives the same landscape as that Beta's conjugate variant to float tolerance, because moment matching within the Beta family is lossless, while a pair no Beta can carry falls back to the uniform prior's quantiles rather than to an invented shape (´def:companion:prior´). The two variants are the specified boundary (´sig:companion:posterior´) — exactness where structure crossed, first-order fidelity where only moments did. |
//! | [`dominance_reports_the_forfeiture_mass`] | resonance | At partial block-catching the forfeiture threshold sits at the tabulated 0.706 (´thm:landscape:catching-forfeiture´), and the landscape reports Block dominated with the uniform prior's mass above it — 0.294 — while a confident posterior past the threshold pushes the probability toward one. The trigger the theorem is written for is evidential, not point-estimate (´thm:landscape:dominance´): the probability moves smoothly with the posterior where the boolean of the old surface could only flip. |
//! | [`reported_domination_drives_the_display_treatment`] | resonance | A posterior whose mean keeps Challenge's point-estimate regime open but whose mass sits mostly below the domination threshold now marks the rendered tag dominated through the first trigger — the reported probability read against the display threshold (´alg:rendering:dominated-treatment´) — while a sharper posterior at the same mean leaves the tag live. The host sees an open evidential question where the inversion trigger alone saw a settled fact, which is precisely the case the corpus wants reported with both regime and probability (´thm:landscape:dominance´). |
//!
//! Crate tests for the decision landscape (´sig:landscape:output´)
//! against the worked chapter (´chap:spec:worked-landscapes´).

use crate::assessment::RiskBasis;
use crate::numerics::stable_logit;
use crate::resonance::channel::{ChannelPolicy, RewardParameters};
use crate::resonance::landscape::{DecisionLandscape, derive_landscape_from_basis};
use crate::types::{Action, ChallengeEstimate};

/// A risk basis holding exactly the triple the derivation reads
/// (´schema:risk:basis´); every other field is inert here by
/// outcome-prediction neutrality (´def:landscape:outcome-neutrality´).
fn basis(p_bad: f64, sigma_eff: f64, kappa_eff: f64) -> RiskBasis {
    RiskBasis {
        p_bad,
        uncertainty: p_bad * (1.0 - p_bad) * sigma_eff / kappa_eff,
        // The derivation reads the basis and never the models, so a basis
        // assembled here borrowed nothing by construction.
        borrowed_share: 0.0,
        anchor_weight: 0.0,
        rho_eff: stable_logit(p_bad) * kappa_eff,
        sigma_eff,
        kappa_eff,
        p_bad_sister: p_bad,
        p_bad_operational: p_bad,
        intervention_effectiveness: 0.0,
        anchor_converged: false,
        n_sentinels_reporting: 0,
        sister_regime_calibration_records: 0.0,
        anchor_regime_calibration_records: 0.0,
    }
}

/// The uniform prior as a moment pair: mean one half, variance one
/// twelfth (´def:companion:prior´).
fn uniform_prior() -> ChallengeEstimate {
    ChallengeEstimate::default()
}

fn low_risk_landscape() -> DecisionLandscape {
    derive_landscape_from_basis(&basis(0.05, 0.42, 1.0), &ChannelPolicy::default(), uniform_prior())
}

/// The low-risk worked landscape comes back figure for figure: the shared
/// term at 1.178 with its 0.168 uncertainty, three crossovers at the
/// tabulated logits and postures, every variance at 0.0816, and each
/// credible interval matching the quantile push-through the specification
/// tabulates to two decimals (´ex:landscape:low-risk´). The worked chapter
/// is the numeric acceptance oracle for the whole derivation, so an
/// implementation that reproduces its tables has the decomposition, the
/// covariance and the intervals right at once.
///
/// ´claim:resonance:the-landscape-reproduces-the-low-risk-worked-tables´
/// ´test:crate:low-risk-landscape-matches-the-worked-tables´
#[test]
fn low_risk_landscape_matches_the_worked_tables() {
    let l = low_risk_landscape();

    // Evidence group (´eq:landscape:rigid-decomposition´).
    assert!((l.u - 1.178).abs() < 5e-4, "shared term: got {}", l.u);
    assert!((l.sigma_u - 0.168).abs() < 5e-4, "shared uncertainty: got {}", l.sigma_u);
    assert!((l.beta_sum - 2.5).abs() < 1e-12);
    assert!((l.posterior.q_c() - 0.5).abs() < 1e-12);

    // Crossover locations, both scales, and variances.
    let expect = [
        (0.330, 0.582, -0.848, -0.8),
        (0.534, 0.630, -0.644, 0.8),
        (1.462, 0.812, 0.284, 0.8),
    ];
    assert_eq!(l.crossovers.len(), 3);
    for (c, &(logit, posture, offset, sens)) in l.crossovers.iter().zip(expect.iter()) {
        assert!((c.logit - logit).abs() < 1e-3, "logit: got {} want {logit}", c.logit);
        assert!(
            (c.posture - posture).abs() < 1e-3,
            "posture: got {} want {posture}",
            c.posture
        );
        assert!((c.offset - offset).abs() < 1e-3, "offset: got {} want {offset}", c.offset);
        assert!(
            (c.sensitivity - sens).abs() < 1e-3,
            "sensitivity: got {} want {sens}",
            c.sensitivity
        );
        assert!((c.variance - 0.0816).abs() < 1e-4, "variance: got {}", c.variance);
    }

    // Credible intervals (´def:landscape:credible-intervals´), logit and
    // posture scales, to the two decimals the chapter states.
    let intervals = [(-0.27, 1.86), (-0.06, 2.06), (0.87, 2.99)];
    let interval_postures = [(0.43, 0.87), (0.48, 0.89), (0.70, 0.95)];
    for ((c, &(lo, hi)), &(plo, phi)) in l.crossovers.iter().zip(intervals.iter()).zip(interval_postures.iter()) {
        assert!(
            (c.interval_logit.0 - lo).abs() < 5e-3,
            "interval lo: got {}",
            c.interval_logit.0
        );
        assert!(
            (c.interval_logit.1 - hi).abs() < 5e-3,
            "interval hi: got {}",
            c.interval_logit.1
        );
        // The chapter's posture endpoints are the logistic images of its
        // two-decimal logit endpoints, so they carry that rounding.
        assert!((c.interval_posture.0 - plo).abs() < 8e-3);
        assert!((c.interval_posture.1 - phi).abs() < 8e-3);
    }

    // Regimes (´prop:landscape:width-variance´): Challenge 0.204 wide with
    // standard deviation 0.462, Slow 0.928 wide and exactly certain.
    assert_eq!(l.regimes.len(), 4);
    assert!(l.regimes[0].width.is_none(), "Allow is a boundary regime");
    let challenge = &l.regimes[1];
    assert!((challenge.width.unwrap() - 0.204).abs() < 1e-3);
    assert!((challenge.width_variance.unwrap().sqrt() - 0.462).abs() < 1e-3);
    let slow = &l.regimes[2];
    assert!((slow.width.unwrap() - 0.928).abs() < 1e-3);
    assert!(slow.width_variance.unwrap().abs() < 1e-15, "the Slow width is exactly known");
    assert!(l.regimes[3].width.is_none(), "Block is a boundary regime");

    // The dominance column (´thm:landscape:dominance´): Challenge at the
    // cold-start 0.375, everything else at zero.
    let dominance: Vec<f64> = l.regimes.iter().map(|r| r.domination_probability).collect();
    assert!(dominance[0].abs() < 1e-12, "Allow: got {}", dominance[0]);
    assert!((dominance[1] - 0.375).abs() < 1e-9, "Challenge: got {}", dominance[1]);
    assert!(dominance[2].abs() < 1e-12, "Slow: got {}", dominance[2]);
    assert!(dominance[3].abs() < 1e-12, "Block at full catching: got {}", dominance[3]);
}

/// Recomputing the landscape at the high-risk and medium-risk bases leaves
/// every offset, every width and every width variance bit-for-bit identical
/// to the low-risk case: only the shared term moved, and all three
/// crossovers moved with it by exactly that difference
/// (´thm:landscape:rigid-translation´). Regime structure belongs to the
/// policy and the posterior, so no risk update can change what the regimes
/// are, only where they sit.
///
/// ´claim:resonance:the-regime-table-is-independent-of-the-risk-basis´
/// ´test:crate:regime-table-is-risk-independent´
#[test]
fn regime_table_is_risk_independent() {
    let low = low_risk_landscape();
    let high = derive_landscape_from_basis(&basis(0.72, 0.74, 1.0), &ChannelPolicy::default(), uniform_prior());
    let medium = derive_landscape_from_basis(&basis(0.35, 0.88, 1.0), &ChannelPolicy::default(), uniform_prior());

    // The worked figures for the two shifted landscapes
    // (´ex:landscape:high-risk´), (´ex:landscape:medium-risk´).
    assert!((high.u - (-0.378)).abs() < 5e-4, "high-risk shared term: got {}", high.u);
    assert!((high.crossovers[0].logit - (-1.226)).abs() < 1e-3);
    assert!((high.crossovers[2].logit - (-0.094)).abs() < 1e-3);
    assert!((high.crossovers[0].variance - 0.141).abs() < 5e-4);
    assert!((medium.u - 0.248).abs() < 5e-4);
    assert!((medium.crossovers[1].logit - (-0.396)).abs() < 1e-3);
    assert!((medium.crossovers[0].variance - 0.177).abs() < 5e-4);

    for shifted in [&high, &medium] {
        let du = shifted.u - low.u;
        for (a, b) in low.crossovers.iter().zip(shifted.crossovers.iter()) {
            assert_eq!(a.offset.to_bits(), b.offset.to_bits(), "offsets are risk-free");
            assert_eq!(a.sensitivity.to_bits(), b.sensitivity.to_bits());
            assert_eq!(a.delta_good.to_bits(), b.delta_good.to_bits());
            assert_eq!(a.delta_bad.to_bits(), b.delta_bad.to_bits());
            assert!(
                ((b.logit - a.logit) - du).abs() < 1e-12,
                "rigid translation by the shared term"
            );
        }
        for (a, b) in low.regimes.iter().zip(shifted.regimes.iter()) {
            assert_eq!(a.width.map(f64::to_bits), b.width.map(f64::to_bits));
            assert_eq!(a.width_variance.map(f64::to_bits), b.width_variance.map(f64::to_bits));
        }
    }
}

/// Withdrawing Slow re-derives the Challenge-to-Block transition from the
/// surviving pair's own cost tables rather than summing the two steps it
/// replaced: the offset lands at +0.204, Challenge's regime widens from a
/// 0.204 sliver to 1.052, and the width's standard deviation stays at 0.462
/// because it depends only on the difference of adjacent sensitivities
/// (´ex:landscape:three-action´). The landscape keeps one crossover per
/// surviving adjacent pair, so shortening the menu changes values and never
/// structure.
///
/// ´claim:resonance:the-three-action-landscape-matches-its-worked-tables´
/// ´test:crate:three-action-landscape-matches-the-worked-tables´
#[test]
fn three_action_landscape_matches_the_worked_tables() {
    let policy = ChannelPolicy {
        actions: vec![Action::Allow, Action::Challenge, Action::Block],
        reward: RewardParameters::default(),
        ..ChannelPolicy::default()
    };
    let l = derive_landscape_from_basis(&basis(0.05, 0.42, 1.0), &policy, uniform_prior());

    assert_eq!(l.crossovers.len(), 2);
    assert_eq!(l.regimes.len(), 3);

    // Allow-to-Challenge is untouched; Challenge-to-Block is derived
    // from the three-action differentials 2.5 and 1.5
    // (´def:landscape:reward-differentials´).
    assert!((l.crossovers[0].logit - 0.330).abs() < 1e-3);
    assert!((l.crossovers[1].offset - 0.204).abs() < 1e-3);
    assert!((l.crossovers[1].logit - 1.382).abs() < 1e-3);
    assert!((l.crossovers[1].posture - 0.799).abs() < 1e-3);
    assert!((l.crossovers[1].delta_good - 2.5).abs() < 1e-10);
    assert!((l.crossovers[1].delta_bad - 1.5).abs() < 1e-10);
    // Pinned to the recomputed push-through, not the chapter's row. The
    // chapter tabulates [0.44, 2.51] for this transition, and that pair
    // fails recomputation against the definition it cites
    // (´def:landscape:credible-intervals´): evaluating the offset at the
    // uniform prior's 2.5 and 97.5 per cent quantiles gives offsets
    // −0.063 and +1.403, hence [0.786, 2.910] — which also equals the
    // Allow-to-Challenge interval rigidly shifted by the Challenge width,
    // as the shared-term decomposition requires. Every other interval row
    // of the worked chapter recomputes exactly; this one is recorded as a
    // corpus figure defect in wave ten's landing note.
    assert!((l.crossovers[1].interval_logit.0 - 0.786).abs() < 5e-3);
    assert!((l.crossovers[1].interval_logit.1 - 2.910).abs() < 5e-3);

    let challenge = &l.regimes[1];
    assert!((challenge.width.unwrap() - 1.052).abs() < 1e-3);
    assert!((challenge.width_variance.unwrap().sqrt() - 0.462).abs() < 1e-3);
    // Dominance falls from the four-action 0.375 to the worked 0.067
    // (´thm:landscape:dominance´): very likely viable in the three-action
    // channel and close to a coin flip in the four-action one.
    assert!(
        (challenge.domination_probability - 0.067).abs() < 5e-4,
        "got {}",
        challenge.domination_probability
    );
}

/// A landscape for J declared actions always carries exactly J−1 crossovers
/// and J regimes, the two boundary regimes carrying no width, whatever the
/// evidence says about dominance (´sig:landscape:output´). Shape stability
/// is what makes a landscape safe to store, replay and difference across a
/// policy comparison — a Companion update can move every number and can
/// never add or remove a record.
///
/// ´claim:resonance:landscape-shape-is-a-function-of-the-declaration-alone´
/// ´test:crate:landscape-shape-is-stable´
#[test]
fn landscape_shape_is_stable() {
    let two_action = ChannelPolicy {
        actions: vec![Action::Allow, Action::Block],
        reward: RewardParameters::default(),
        ..ChannelPolicy::default()
    };
    // A posterior mean deep in Challenge's dominated region
    // (´thm:landscape:dominance´) and one far out of it.
    for estimate in [
        ChallengeEstimate::new(0.05, 0.01).unwrap(),
        ChallengeEstimate::new(0.95, 0.001).unwrap(),
    ] {
        for (policy, n) in [(ChannelPolicy::default(), 4), (two_action.clone(), 2)] {
            let l = derive_landscape_from_basis(&basis(0.35, 0.88, 1.0), &policy, estimate);
            assert_eq!(l.crossovers.len(), n - 1);
            assert_eq!(l.regimes.len(), n);
            assert!(l.regimes.first().unwrap().width.is_none());
            assert!(l.regimes.last().unwrap().width.is_none());
            for r in &l.regimes[1..n - 1] {
                assert!(r.width.is_some(), "interior regimes carry their width");
            }
        }
    }
}

/// Deriving twice from one argument triple gives two landscapes whose every
/// scalar agrees to the bit, shared term through interval endpoints. The
/// transform holds no state and reads no clock (´pf:landscape:purity´), so
/// a host that stored the assessment, the policy and the posterior stored
/// the complete determinant of the landscape.
///
/// ´claim:resonance:one-argument-triple-determines-the-landscape-to-the-bit´
/// ´test:crate:derivation-replays-bit-for-bit´
#[test]
fn derivation_replays_bit_for_bit() {
    let b = basis(0.35, 0.88, 1.0);
    let estimate = ChallengeEstimate::new(0.62, 0.03).unwrap();
    let first = derive_landscape_from_basis(&b, &ChannelPolicy::default(), estimate);
    let second = derive_landscape_from_basis(&b, &ChannelPolicy::default(), estimate);

    assert_eq!(first.u.to_bits(), second.u.to_bits());
    assert_eq!(first.sigma_u.to_bits(), second.sigma_u.to_bits());
    for (a, b) in first.crossovers.iter().zip(second.crossovers.iter()) {
        assert_eq!(a.logit.to_bits(), b.logit.to_bits());
        assert_eq!(a.variance.to_bits(), b.variance.to_bits());
        assert_eq!(a.interval_logit.0.to_bits(), b.interval_logit.0.to_bits());
        assert_eq!(a.interval_logit.1.to_bits(), b.interval_logit.1.to_bits());
    }
}

/// Below full block-catching, a posterior mean past the forfeiture
/// threshold drives the Slow-to-Block adverse differential non-positive and
/// the crossover to positive infinity, with the interval's upper endpoint
/// following it (´thm:landscape:catching-forfeiture´): blocking has stopped
/// paying at any posture, and the landscape reports the fact structurally
/// instead of clamping it to a large number.
///
/// ´claim:resonance:forfeiture-is-reported-as-an-infinite-crossover´
/// ´test:crate:forfeiture-sends-the-crossover-to-infinity´
#[test]
fn forfeiture_sends_the_crossover_to_infinity() {
    // β_c = 0 puts the forfeiture threshold at 0.545
    // (´thm:landscape:catching-forfeiture´); a confident posterior above
    // it drives Δ_bad below zero for Slow-to-Block.
    let policy = ChannelPolicy {
        reward: RewardParameters {
            block_catches: 0.0,
            ..RewardParameters::default()
        },
        ..ChannelPolicy::default()
    };
    let estimate = ChallengeEstimate::new(0.8, 0.005).unwrap();
    let l = derive_landscape_from_basis(&basis(0.35, 0.88, 1.0), &policy, estimate);

    let last = l.crossovers.last().unwrap();
    assert!(
        last.logit.is_infinite() && last.logit > 0.0,
        "crossover at +∞, got {}",
        last.logit
    );
    assert!((last.posture - 1.0).abs() < 1e-15);
    assert!(last.delta_bad <= 0.0, "the differential the theorem names has vanished");
    assert!(
        last.interval_logit.1.is_infinite(),
        "the interval's far endpoint follows the crossover"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// The Landscape Utilities (´sig:landscape:utilities´)
// ═══════════════════════════════════════════════════════════════════════════════

/// The optimal-action utility walks the low-risk worked landscape regime by
/// regime — Allow at Normal, Challenge just past its lower crossover, Slow
/// and Block in theirs — and hands over within a hair's breadth of the
/// crossover, the tie itself breaking toward the more permissive action so
/// the reading is left-continuous in the posture
/// (´alg:landscape:optimal-action´). The utility reads the landscape and a
/// posture and nothing else, which is what makes one landscape answer every
/// operating point a host cares to probe (´sig:landscape:utilities´).
///
/// ´claim:resonance:the-optimal-action-utility-reads-the-envelope-at-a-posture´
/// ´test:crate:optimal-action-reads-the-envelope´
#[test]
fn optimal_action_reads_the_envelope() {
    use crate::resonance::landscape::optimal_action;
    let l = low_risk_landscape();

    // The worked regimes: crossovers at postures 0.582, 0.630, 0.812.
    assert_eq!(optimal_action(&l, 0.30), Action::Allow);
    assert_eq!(optimal_action(&l, 0.60), Action::Challenge);
    assert_eq!(optimal_action(&l, 0.70), Action::Slow);
    assert_eq!(optimal_action(&l, 0.90), Action::Block);

    // Left-continuity at the handover: a hair below the first crossover
    // the answer is still Allow, a hair above it is Challenge. The exact
    // tie breaks toward the more permissive action by the envelope's
    // strict comparison; the posture-scale roundtrip cannot reproduce the
    // tie to the bit, so the property is pinned as the bracket around it.
    let crossover_posture = l.crossovers[0].posture;
    assert_eq!(optimal_action(&l, crossover_posture - 1e-9), Action::Allow);
    assert_eq!(optimal_action(&l, crossover_posture + 1e-9), Action::Challenge);
}

/// At a challenge estimate low enough to invert Challenge's crossovers, a
/// posture inside the inverted span answers Allow from the cost-curve
/// envelope where walking the crossover vector in order would have returned
/// the dominated action (´alg:landscape:optimal-action´). The bracketing
/// shortcut fails structurally under inversion — the crossovers bracket
/// nothing (´thm:landscape:dominance´) — and the envelope needs no special
/// case to exclude an action that never attains the minimum.
///
/// ´claim:resonance:the-envelope-skips-a-dominated-action-inside-its-inverted-span´
/// ´test:crate:envelope-skips-a-dominated-action´
#[test]
fn envelope_skips_a_dominated_action() {
    use crate::numerics::stable_sigmoid;
    use crate::resonance::landscape::optimal_action;

    // q̂_c = 0.2 sits below the four-action Challenge threshold of 0.375
    // (´thm:landscape:dominance´), inverting its two crossovers.
    let estimate = ChallengeEstimate::new(0.2, 0.01).unwrap();
    let l = derive_landscape_from_basis(&basis(0.35, 0.88, 1.0), &ChannelPolicy::default(), estimate);
    let lower = l.crossovers[0].logit;
    let upper = l.crossovers[1].logit;
    assert!(upper < lower, "Challenge's bounding crossovers are inverted at this estimate");

    // A posture between the inverted pair: the naive walk counts one
    // crossover below and answers Challenge; the envelope answers Allow.
    let inside = stable_sigmoid(f64::midpoint(lower, upper));
    assert_eq!(optimal_action(&l, inside), Action::Allow);
}

/// Fragility at the worked chapter's probes returns the chapter's own
/// figures (´ex:landscape:low-risk´), (´ex:landscape:high-risk´): near-zero
/// flip at Normal on the low-risk landscape with the boundary 4.1 standard
/// deviations away, flip about 0.72 with a challenge share about 0.65 at
/// posture 0.6, and the high-risk landscape's modal Slow at Normal. The two
/// shares split the bounding variance by evidence source
/// (´def:fragility:definition´), which is the reading no single component
/// can produce.
///
/// ´claim:resonance:fragility-reproduces-the-worked-chapters-readings´
/// ´test:crate:fragility-matches-the-worked-readings´
#[test]
fn fragility_matches_the_worked_readings() {
    use crate::resonance::landscape::fragility;

    let low = low_risk_landscape();

    // Normal posture: modal Allow, flip about zero — the boundary is 4.1
    // standard deviations away.
    let at_normal = fragility(&low, 0.30);
    assert_eq!(at_normal.modal_action, Action::Allow);
    assert!(at_normal.flip_probability < 1e-3, "got {}", at_normal.flip_probability);

    // Posture 0.6: modal Challenge, flip ≈ 0.72, challenge share ≈ 0.65.
    let at_elevated = fragility(&low, 0.60);
    assert_eq!(at_elevated.modal_action, Action::Challenge);
    assert!(
        (at_elevated.flip_probability - 0.72).abs() < 0.01,
        "flip: got {}",
        at_elevated.flip_probability
    );
    assert!(
        (at_elevated.challenge_share - 0.65).abs() < 0.01,
        "challenge share: got {}",
        at_elevated.challenge_share
    );
    assert!((at_elevated.risk_share + at_elevated.challenge_share - 1.0).abs() < 1e-12);

    // High-risk landscape at Normal: modal Slow between the worked
    // crossovers at −1.022 and −0.094. The chapter says "about 0.31";
    // the Gaussian-marginal default the fragility definition prescribes
    // gives 0.343, and the difference is the approximation the chapter's
    // own prose acknowledges by hedging — recorded as a wave-ten reading.
    let high = derive_landscape_from_basis(&basis(0.72, 0.74, 1.0), &ChannelPolicy::default(), uniform_prior());
    let at_high = fragility(&high, 0.30);
    assert_eq!(at_high.modal_action, Action::Slow);
    assert!(
        (at_high.flip_probability - 0.343).abs() < 0.01,
        "got {}",
        at_high.flip_probability
    );
}

/// Inside an inverted region the fragility reading is taken against the
/// crossings that bound the optimal regime from the envelope — here a
/// compound Allow-to-Slow crossing no single crossover record carries —
/// rather than the raw adjacent records (´def:fragility:definition´). The
/// modal action, the flip probability and the shares all stay well-formed
/// where the naive bracketing reading has nothing coherent to offer.
///
/// ´claim:resonance:fragility-reads-the-envelopes-own-boundaries-under-inversion´
/// ´test:crate:fragility-uses-envelope-boundaries-under-inversion´
#[test]
fn fragility_uses_envelope_boundaries_under_inversion() {
    use crate::numerics::stable_sigmoid;
    use crate::resonance::landscape::fragility;

    let estimate = ChallengeEstimate::new(0.2, 0.01).unwrap();
    let l = derive_landscape_from_basis(&basis(0.35, 0.88, 1.0), &ChannelPolicy::default(), estimate);
    let inside = stable_sigmoid(f64::midpoint(l.crossovers[0].logit, l.crossovers[1].logit));

    let f = fragility(&l, inside);
    assert_eq!(f.modal_action, Action::Allow, "the envelope's answer, not the bracketing one");
    assert!(f.flip_probability >= 0.0 && f.flip_probability <= 1.0);
    assert!(f.risk_share >= 0.0 && f.challenge_share >= 0.0);
    assert!((f.risk_share + f.challenge_share - 1.0).abs() < 1e-12);
    // Allow's envelope regime here is bounded above by the compound
    // Allow-to-Slow crossing, which sits below the raw Allow-to-Challenge
    // record: the posture is nearer the handover than the records suggest,
    // so the flip probability is substantial.
    assert!(f.flip_probability > 0.2, "got {}", f.flip_probability);
}

/// Across a grid of postures spanning the open unit interval and landscapes
/// spanning healthy, inverted and forfeited regimes, both utilities always
/// answer (´sig:landscape:utilities´): the optimal action is one of the
/// declared set, the flip probability stays inside the unit interval, and
/// the two shares are non-negative and sum to one wherever a boundary
/// exists. Totality is the utilities' contract — every posture has an
/// optimal action and a fragility.
///
/// ´claim:resonance:both-utilities-are-total-over-the-open-posture-axis´
/// ´test:crate:utilities-are-total-over-the-posture-axis´
#[test]
fn utilities_are_total_over_the_posture_axis() {
    use crate::resonance::landscape::{fragility, optimal_action};

    let forfeiting = ChannelPolicy {
        reward: RewardParameters {
            block_catches: 0.0,
            ..RewardParameters::default()
        },
        ..ChannelPolicy::default()
    };
    let landscapes = [
        low_risk_landscape(),
        derive_landscape_from_basis(
            &basis(0.35, 0.88, 1.0),
            &ChannelPolicy::default(),
            ChallengeEstimate::new(0.2, 0.01).unwrap(),
        ),
        derive_landscape_from_basis(
            &basis(0.35, 0.88, 1.0),
            &forfeiting,
            ChallengeEstimate::new(0.8, 0.005).unwrap(),
        ),
    ];
    for l in &landscapes {
        let declared: Vec<Action> = l.regimes.iter().map(|r| r.action).collect();
        for i in 1..100 {
            let posture = f64::from(i) / 100.0;
            let action = optimal_action(l, posture);
            assert!(declared.contains(&action));
            let f = fragility(l, posture);
            assert_eq!(f.modal_action, action, "the two utilities agree on the modal action");
            assert!(f.flip_probability >= 0.0 && f.flip_probability <= 1.0);
            assert!(f.risk_share >= 0.0 && f.challenge_share >= 0.0);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// The Posterior Crossing (´sig:companion:posterior´)
// ═══════════════════════════════════════════════════════════════════════════════

/// The Companion's pseudo-counts arrive in the landscape as themselves:
/// after one observed challenge failure on uniform priors the tracker holds
/// Beta(2, 1), the carrier the harness path hands the derivation is the
/// conjugate variant with exactly those counts, and the credible intervals
/// equal the ones computed from Beta(2, 1) quantiles directly
/// (´def:landscape:credible-intervals´). The conjugate structure survives
/// the crossing instead of being flattened to two moments one call before
/// the consumers that need it (´sig:companion:posterior´).
///
/// ´claim:resonance:the-conjugate-pseudo-counts-cross-the-derivation-boundary-whole´
/// ´test:crate:conjugate-counts-cross-the-boundary-whole´
#[test]
fn conjugate_counts_cross_the_boundary_whole() {
    use crate::numerics::beta_quantile;
    use crate::risk::challenge::{ChallengeEffectivenessProvider, ChallengeEffectivenessState, ChallengeEffectivenessTracker};
    use crate::types::{ChallengePosteriorInput, ChallengeResult, ChannelId, PersistentTimestamp};

    let mut tracker = ChallengeEffectivenessTracker::new();
    let channel = ChannelId(7);
    let t0 = PersistentTimestamp::new(1_000_000, 0);
    tracker.insert_channel(channel, ChallengeEffectivenessState::new(1.0, 1.0, 0.9998));
    tracker.update(channel, ChallengeResult::Fail, &t0);

    let carrier: ChallengePosteriorInput = tracker.challenge_posterior(channel, &t0).into();
    let ChallengePosteriorInput::Conjugate { alpha, beta } = carrier else {
        panic!("the tracker's posterior crosses as the conjugate variant");
    };
    assert!((alpha - 2.0).abs() < 1e-12, "α: got {alpha}");
    assert!((beta - 1.0).abs() < 1e-12, "β: got {beta}");

    let l = derive_landscape_from_basis(&basis(0.05, 0.42, 1.0), &ChannelPolicy::default(), carrier);

    // The intervals are the push-through at Beta(2, 1)'s own quantiles:
    // for Allow-to-Challenge, offset b(q) = 0.4·ln(0.06/q) evaluated at
    // the exact 2.5 and 97.5 per cent quantiles of Beta(2, 1).
    let q_lo = beta_quantile(2.0, 1.0, 0.025);
    let q_hi = beta_quantile(2.0, 1.0, 0.975);
    let b_at = |q: f64| 0.4 * (0.06_f64 / q).ln();
    let expected_lo = 1.96_f64.mul_add(-l.sigma_u, l.u + b_at(q_hi).min(b_at(q_lo)));
    let expected_hi = 1.96_f64.mul_add(l.sigma_u, l.u + b_at(q_hi).max(b_at(q_lo)));
    assert!((l.crossovers[0].interval_logit.0 - expected_lo).abs() < 1e-9);
    assert!((l.crossovers[0].interval_logit.1 - expected_hi).abs() < 1e-9);
}

/// A bare mean-and-variance pair from outside the crate is admitted, not
/// refused: a pair carrying a Beta's own moments derives the same landscape
/// as that Beta's conjugate variant to float tolerance, because moment
/// matching within the Beta family is lossless, while a pair no Beta can
/// carry falls back to the uniform prior's quantiles rather than to an
/// invented shape (´def:companion:prior´). The two variants are the
/// specified boundary (´sig:companion:posterior´) — exactness where
/// structure crossed, first-order fidelity where only moments did.
///
/// ´claim:resonance:a-moment-only-pair-is-admitted-at-matched-fidelity´
/// ´test:crate:moment-pair-crosses-at-matched-fidelity´
#[test]
fn moment_pair_crosses_at_matched_fidelity() {
    use crate::types::ChallengePosteriorInput;

    let b = basis(0.05, 0.42, 1.0);
    let policy = ChannelPolicy::default();

    // Beta(3, 2): mean 0.6, variance 0.04. The matched pair recovers it.
    let conjugate = ChallengePosteriorInput::conjugate(3.0, 2.0).unwrap();
    let moments = ChallengeEstimate::new(0.6, 0.04).unwrap();
    let from_counts = derive_landscape_from_basis(&b, &policy, conjugate);
    let from_moments = derive_landscape_from_basis(&b, &policy, moments);
    for (a, m) in from_counts.crossovers.iter().zip(from_moments.crossovers.iter()) {
        assert!((a.logit - m.logit).abs() < 1e-9);
        assert!((a.interval_logit.0 - m.interval_logit.0).abs() < 1e-9);
        assert!((a.interval_logit.1 - m.interval_logit.1).abs() < 1e-9);
    }

    // A pair no Beta carries: variance at the Bernoulli ceiling. The
    // quantile shape falls back to the uniform prior instead of a
    // fabricated fit.
    let unmatched = ChallengeEstimate::new(0.5, 0.3).unwrap();
    let carrier: ChallengePosteriorInput = unmatched.into();
    assert_eq!(carrier.beta_shape(), (1.0, 1.0));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dominance as a Probability (´thm:landscape:dominance´)
// ═══════════════════════════════════════════════════════════════════════════════

/// At partial block-catching the forfeiture threshold sits at the tabulated
/// 0.706 (´thm:landscape:catching-forfeiture´), and the landscape reports
/// Block dominated with the uniform prior's mass above it — 0.294 — while a
/// confident posterior past the threshold pushes the probability toward
/// one. The trigger the theorem is written for is evidential, not
/// point-estimate (´thm:landscape:dominance´): the probability moves
/// smoothly with the posterior where the boolean of the old surface could
/// only flip.
///
/// ´claim:resonance:the-forfeiture-mass-is-reported-as-blocks-domination-probability´
/// ´test:crate:dominance-reports-the-forfeiture-mass´
#[test]
fn dominance_reports_the_forfeiture_mass() {
    let policy = ChannelPolicy {
        reward: RewardParameters {
            block_catches: 0.5,
            ..RewardParameters::default()
        },
        ..ChannelPolicy::default()
    };

    // Uniform prior: the mass above 0.706 is 0.294.
    let l = derive_landscape_from_basis(&basis(0.35, 0.88, 1.0), &policy, uniform_prior());
    let block = l.regimes.last().unwrap();
    assert!(
        (block.domination_probability - (1.0 - 0.706)).abs() < 1e-3,
        "got {}",
        block.domination_probability
    );

    // A confident posterior above the threshold: most of its mass is in
    // the forfeited region.
    let confident = crate::types::ChallengePosteriorInput::conjugate(80.0, 20.0).unwrap();
    let l = derive_landscape_from_basis(&basis(0.35, 0.88, 1.0), &policy, confident);
    assert!(
        l.regimes.last().unwrap().domination_probability > 0.95,
        "got {}",
        l.regimes.last().unwrap().domination_probability
    );
}

/// A posterior whose mean keeps Challenge's point-estimate regime open but
/// whose mass sits mostly below the domination threshold now marks the
/// rendered tag dominated through the first trigger — the reported
/// probability read against the display threshold
/// (´alg:rendering:dominated-treatment´) — while a sharper posterior at the
/// same mean leaves the tag live. The host sees an open evidential question
/// where the inversion trigger alone saw a settled fact, which is precisely
/// the case the corpus wants reported with both regime and probability
/// (´thm:landscape:dominance´).
///
/// ´claim:resonance:the-reported-domination-probability-drives-the-first-display-trigger´
/// ´test:crate:reported-domination-drives-the-display-treatment´
#[test]
fn reported_domination_drives_the_display_treatment() {
    use crate::numerics::reg_inc_beta;
    use crate::resonance::derivation::{ResonanceConfig, render_resonances};
    use crate::resonance::tags::Tag;
    use crate::types::ChallengePosteriorInput;

    let b = basis(0.35, 0.88, 1.0);
    let policy = ChannelPolicy::default();
    let config = ResonanceConfig::default();

    // Beta(1, 1.6): mean 0.3846 — above the 0.375 threshold, so the
    // point-estimate width is positive and the inversion trigger stays
    // quiet — with mass 0.529 below it.
    let broad = ChallengePosteriorInput::conjugate(1.0, 1.6).unwrap();
    assert!(broad.q_c() > 0.375);
    assert!(reg_inc_beta(1.0, 1.6, 0.375) > 0.5);
    let l = derive_landscape_from_basis(&b, &policy, broad);
    let challenge = &l.regimes[1];
    assert!(challenge.width.unwrap() > 0.0, "the point-estimate regime is open");
    assert!(
        challenge.domination_probability > 0.5,
        "got {}",
        challenge.domination_probability
    );

    let profile = render_resonances(&l, &b, &config);
    let tag = profile.tags.iter().find(|t| t.tag == Tag::Challenge).unwrap();
    assert!(tag.dominated, "the first trigger marks the tag");
    assert!((tag.magnitude - config.epsilon_mono).abs() < 1e-18, "dominated magnitude");
    assert!((tag.q - config.q_min).abs() < 1e-12, "dominated bandwidth");

    // Ten times the evidence at the same mean: the mass below the
    // threshold falls under one half and the tag stays live.
    let sharp = ChallengePosteriorInput::conjugate(10.0, 16.0).unwrap();
    assert!((sharp.q_c() - broad.q_c()).abs() < 1e-12);
    assert!(reg_inc_beta(10.0, 16.0, 0.375) < 0.5);
    let l = derive_landscape_from_basis(&b, &policy, sharp);
    assert!(l.regimes[1].domination_probability < 0.5);
    let profile = render_resonances(&l, &b, &config);
    let tag = profile.tags.iter().find(|t| t.tag == Tag::Challenge).unwrap();
    assert!(!tag.dominated, "same point estimate, different evidence, different display");
}
