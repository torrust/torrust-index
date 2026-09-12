// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`scalar_signal_population_split_converges_directionally`] | scenario | The engine learns a scalar request signal from the public assess-then-label loop alone: after five hundred alternating cycles teaching high scores as adverse and low scores as benign, held-out requests from the two populations separate by more than a tenth in posterior risk, and twenty samples drawn from each rank against one another with pairwise AUC above 0.65. Ordering, not just the gap between two points, is what a host needs if it is going to threshold on risk at all. |

//! Integration tests for learning convergence despite censoring and class
//! imbalance (´claim:scenario:labelled-cycles-teach-the-engine-to-rank-adverse-above-benign´).
//!
//! These are deliberately small public-API smokes. The long streaming tapes and
//! internal model-state probes stay with the Stage-3 harness work; this file
//! pins the directionally important behaviour that can already be observed from
//! `assess()`, `label()`, and host-side derivation.
//!
//! # Cross-References
//!
//! - (´chap:spec:convergence-and-resources´) — learning convergence and resource bounds
//! - (´chap:spec:label-pipeline´) — label pipeline orchestration
//! - (´chap:spec:assessment-interface´) — core assessment output

use torrust_assayer::testing::{
    LabelSpec, World, assert_finite, assert_health_clean, assert_reckoning_well_formed, cycle_request, scenario_with,
    score_verified_request_schema, with_score_verified,
};
use torrust_assayer::{ChannelPolicy, DerivedReckoning, RequestContext};

const INSTANCE: &str = "learning-convergence";
const SEED: u64 = 0x0000_0005_0000_0091_u64;
const TRAINING_CYCLES: usize = 500;
const MIN_POPULATION_GAP: f64 = 0.10;
const MIN_AUC: f64 = 0.65;

/// The engine learns a scalar request signal from the public assess-then-label
/// loop alone: after five hundred alternating cycles teaching high scores as
/// adverse and low scores as benign, held-out requests from the two
/// populations separate by more than a tenth in posterior risk, and twenty
/// samples drawn from each rank against one another with pairwise AUC above
/// 0.65. Ordering, not just the gap between two points, is what a host needs
/// if it is going to threshold on risk at all.
///
/// ´claim:scenario:labelled-cycles-teach-the-engine-to-rank-adverse-above-benign´
/// ´test:integration:scalar-signal-population-split-converges-directionally´
#[test]
fn scalar_signal_population_split_converges_directionally() {
    let scenario = scenario_with(INSTANCE, SEED, |builder| {
        builder
            .signal_schema(score_verified_request_schema())
            .channel("default", ChannelPolicy::default())
    })
    .expect("learning-convergence world should build");

    for cycle in 0..TRAINING_CYCLES {
        let adverse = cycle % 2 == 1;
        let request = if adverse {
            scored_request(&scenario, "training-adverse", 0.92, true)
        } else {
            scored_request(&scenario, "training-benign", 0.08, false)
        };

        cycle_request(&scenario, request, |assessment_id| {
            if adverse {
                LabelSpec::adverse(assessment_id)
            } else {
                LabelSpec::benign(assessment_id)
            }
        });
    }

    let benign_holdout = scored_risk(&scenario, "holdout-benign", 0.08, false);
    let adverse_holdout = scored_risk(&scenario, "holdout-adverse", 0.92, true);
    let population_gap = adverse_holdout.assessment.risk.p_bad - benign_holdout.assessment.risk.p_bad;

    assert!(
        population_gap > MIN_POPULATION_GAP,
        "500-label signal split should separate adverse and benign populations by > {MIN_POPULATION_GAP}; \
         benign={}, adverse={}, gap={population_gap}",
        benign_holdout.assessment.risk.p_bad,
        adverse_holdout.assessment.risk.p_bad,
    );

    let benign_risks: Vec<f64> = (0..20)
        .map(|sample| {
            let score = f64::from(sample).mul_add(0.012, 0.02);
            scored_risk(&scenario, &format!("auc-benign-{sample}"), score, false)
                .assessment
                .risk
                .p_bad
        })
        .collect();
    let adverse_risks: Vec<f64> = (0..20)
        .map(|sample| {
            let score = f64::from(sample).mul_add(0.012, 0.75);
            scored_risk(&scenario, &format!("auc-adverse-{sample}"), score, true)
                .assessment
                .risk
                .p_bad
        })
        .collect();
    let auc = pairwise_auc(&adverse_risks, &benign_risks);

    assert_finite(&[auc], "signal-split AUC");
    assert!(
        auc > MIN_AUC,
        "500-label signal split should produce pairwise AUC > {MIN_AUC}; got {auc} \
         (benign risks: {benign_risks:?}, adverse risks: {adverse_risks:?})",
    );
    assert_health_clean(&scenario);
}

fn scored_request(world: &World, entity: &str, score: f64, verified: bool) -> RequestContext {
    with_score_verified(world.request("default", entity), score, verified)
}

fn scored_risk(world: &World, entity: &str, score: f64, verified: bool) -> DerivedReckoning {
    let reckoning = world
        .derive_for_request(scored_request(world, entity, score, verified))
        .expect("scored assessment should derive");
    assert_reckoning_well_formed(&reckoning, entity);
    reckoning
}

fn pairwise_auc(adverse_risks: &[f64], benign_risks: &[f64]) -> f64 {
    assert!(!adverse_risks.is_empty(), "AUC needs adverse samples");
    assert!(!benign_risks.is_empty(), "AUC needs benign samples");

    let mut wins = 0.0_f64;
    let mut comparisons = 0.0_f64;
    for adverse_risk in adverse_risks {
        for benign_risk in benign_risks {
            if adverse_risk > benign_risk {
                wins += 1.0;
            } else if adverse_risk.to_bits() == benign_risk.to_bits() {
                wins += 0.5;
            }
            comparisons += 1.0;
        }
    }

    wins / comparisons
}
