// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`public_derive_reckoning_bit_identical`] | scenario | Derivation is a pure function of the assessment, the channel policy and the challenge estimate, and rendering of the landscape, the risk basis and the display configuration: a thousand two-call runs on one held assessment reproduce the whole landscape — shared term, crossovers, intervals, regime table — and every rendered tag's kind, location, magnitude, q and domination flag, compared as raw bits rather than within a tolerance. The first such run also matches what the engine derived internally, so a host that re-derives a stored assessment under the same policy gets back exactly the landscape and profile the engine would have (´pf:landscape:purity´). |
//! | [`repeated_assessment_advances_no_outcome_learned_state`] | scenario | Repeated assessment advances no outcome-learned state. Across a thousand assessments of one request on a world nothing has ever labelled, no label is counted, no calibration refit happens, the calibration delta does not move and the Platt convergence state stands where it started — while the cold standardisation ramp, which is observation-authorised, advances the whole way to its horizon beside them. The contrast is the point: this is not a claim that assessment leaves nothing behind, which is false and which the ramp is deliberately falsifying, but that what it leaves behind is observational. Anything an outcome taught moves on the label path and nowhere else, so a host reading two different values for one subject knows the difference is a coordinate system it can name from the version rather than learning it cannot see. |
//! | [`cold_ramp_trajectory_stays_inside_the_falsifier`] | scenario | The cold ramp discloses where it stands and spreads the whole prior-to-empirical movement across its horizon, so that no single accepted observation moves the uncertainty by more than a tenth of that total. This is the ruling's own falsifier and the reason the ramp was adopted: under the completion gate the coordinate system moved its entire distance between two adjacent requests, and results either side of that step were results in different coordinate systems that nothing on the wire distinguished. Three things are held together here because they are one property. Every assessment reports the phase and the accepted count of the snapshot it was actually scored in, so a reader can place a result on the trajectory. The count rises monotonically and the uncertainty falls monotonically with it, from the class priors to the empirical moments. And the largest single step is far inside a tenth of the total — the studies put it at the first observation, at about one part in fifty-five. The endpoints are the ones the ramp inherited rather than new ones: the ramp redistributes the old cliff and moves neither end of it. |
//! | [`identical_inputs_tag_shape_stable`] | scenario | The shape of a reckoning is exact, and so are its numbers, when the thing repeated is the derivation rather than the assessment: across a hundred derivations of one held assessment the sequence of tag kinds is identical element for element and every location, magnitude and q is identical bit for bit. A reckoning also never comes back empty. This scenario used to repeat the *assessment* and allow its parameters to wander inside a drift band, which was the wrong comparison twice over. It compared results computed in coordinate systems that an accepted cold observation is entitled to move between, so the band was standing in for a legitimate coordinate change rather than for floating-point noise; and by tolerating movement it could not see the one thing derivation actually promises, which is exactness. Holding the assessment fixed removes the coordinate change from the comparison and lets the promise be asserted as the promise is written (´inv:guarantee:evidence-authority´). |
//! | [`identical_inputs_across_entities_well_formed`] | scenario | Purity is a promise about identical inputs, not about all inputs: two distinct entity keys feed the identity layer differently and are each only held to producing a well-formed, finite, in-range assessment. On a cold world with no identity dimensions registered they may legitimately agree, so demanding that different entities differ would be asserting a coincidence of the current model state rather than a property of the engine. |

//! Integration tests for the resonance derivation
//! (´inv:guarantee:derivation-purity´).
//!
//! Exercises the purity / statelessness guarantee of the reckoning
//! pipeline. The bit-identical witness
//! (´test:integration:public-derive-reckoning-bit-identical´) holds one
//! assessment and repeats the pure landscape and rendering transforms,
//! pinning every public number as raw bits. Assessment-facing scenarios use
//! the injected virtual clock, so elapsed host time cannot enter their
//! expectations.
//!
//! These scenarios are label-free. They distinguish the observation-authorised
//! cold ramp from outcome-learned state, so any movement they permit is named
//! as coordinate movement rather than hidden inside a time allowance.
//!
//! # Cross-References
//!
//! - (´test:integration:public-derive-reckoning-bit-identical´) — bit-identical output under a thousand repetitions
//! - (´inv:guarantee:derivation-purity´) — the derivation purity guarantee
//! - (´chap:spec:derivation-interface´) — the `RiskAssessment` inputs to derivation

use std::collections::BTreeMap;

use torrust_assayer::testing::{assert_health_clean, assert_reckoning_well_formed, scenario};
use torrust_assayer::{
    ChallengeEstimate, ChannelPolicy, DecisionLandscape, ResonanceConfig, ResonanceProfile, derive_landscape, render_resonances,
};

const INSTANCE: &str = "derive-purity";
const SEED: u64 = 0x5EED_D00D;

fn derivation_fingerprint(profile: &ResonanceProfile) -> Vec<(String, u64, u64, u64, bool)> {
    profile
        .tags
        .iter()
        .map(|tag| {
            (
                format!("{:?}", tag.tag),
                tag.location.to_bits(),
                tag.magnitude.to_bits(),
                tag.q.to_bits(),
                tag.dominated,
            )
        })
        .collect()
}

fn landscape_fingerprint(landscape: &DecisionLandscape) -> Vec<u64> {
    let mut bits = vec![
        landscape.u.to_bits(),
        landscape.sigma_u.to_bits(),
        landscape.beta_sum.to_bits(),
    ];
    for c in &landscape.crossovers {
        bits.extend([
            c.logit.to_bits(),
            c.posture.to_bits(),
            c.offset.to_bits(),
            c.sensitivity.to_bits(),
            c.variance.to_bits(),
            c.interval_logit.0.to_bits(),
            c.interval_logit.1.to_bits(),
        ]);
    }
    for r in &landscape.regimes {
        bits.push(r.width.map_or(0, f64::to_bits));
        bits.push(r.width_variance.map_or(0, f64::to_bits));
    }
    bits
}

/// Derivation is a pure function of the assessment, the channel policy and
/// the challenge estimate, and rendering of the landscape, the risk basis and
/// the display configuration: a thousand two-call runs on one held assessment
/// reproduce the whole landscape — shared term, crossovers, intervals, regime
/// table — and every rendered tag's kind, location, magnitude, q and
/// domination flag, compared as raw bits rather than within a tolerance. The
/// first such run also matches what the engine derived internally, so a host
/// that re-derives a stored assessment under the same policy gets back exactly
/// the landscape and profile the engine would have (´pf:landscape:purity´).
///
/// ´claim:scenario:derivation-from-a-held-assessment-is-bit-identical-across-repetitions´
/// ´test:integration:public-derive-reckoning-bit-identical´
#[test]
fn public_derive_reckoning_bit_identical() {
    let s = scenario(INSTANCE, SEED);
    let assessed = s.derive_default("direct").expect("baseline assessment");
    let policy = ChannelPolicy::default();
    let challenge = ChallengeEstimate::default();
    let config = ResonanceConfig::default();

    let baseline_landscape = derive_landscape(&assessed.assessment, &policy, challenge);
    let baseline = render_resonances(&baseline_landscape, &assessed.assessment.risk, &config);
    assert_eq!(
        landscape_fingerprint(&baseline_landscape),
        landscape_fingerprint(&assessed.landscape)
    );
    assert_eq!(derivation_fingerprint(&baseline), derivation_fingerprint(&assessed.profile));

    for i in 1..=1_000 {
        let landscape = derive_landscape(&assessed.assessment, &policy, challenge);
        assert_eq!(
            landscape_fingerprint(&landscape),
            landscape_fingerprint(&baseline_landscape),
            "direct derivation #{i}: landscape payload changed",
        );
        let profile = render_resonances(&landscape, &assessed.assessment.risk, &config);
        assert_eq!(
            derivation_fingerprint(&profile),
            derivation_fingerprint(&baseline),
            "direct derivation #{i}: tag payload changed",
        );
    }
}

/// Repeated assessment advances no outcome-learned state. Across a thousand
/// assessments of one request on a world nothing has ever labelled, no label is
/// counted, no calibration refit happens, the calibration delta does not move
/// and the Platt convergence state stands where it started — while the cold
/// standardisation ramp, which is observation-authorised, advances the whole way
/// to its horizon beside them. The contrast is the point: this is not a claim
/// that assessment leaves nothing behind, which is false and which the ramp is
/// deliberately falsifying, but that what it leaves behind is observational.
/// Anything an outcome taught moves on the label path and nowhere else, so a
/// host reading two different values for one subject knows the difference is a
/// coordinate system it can name from the version rather than learning it
/// cannot see.
///
/// ´claim:scenario:repeated-assessment-advances-observational-geometry-and-no-outcome-learned-state´
/// ´test:integration:repeated-assessment-advances-no-outcome-learned-state´
#[test]
fn repeated_assessment_advances_no_outcome_learned_state() {
    let s = scenario(INSTANCE, SEED);

    let before = s.assayer().health_summary();
    assert_eq!(before.total_labels, 0, "the world starts unlabelled");

    for _ in 0..1_000 {
        let _reckoning = s.derive_default("alice").expect("assess should succeed");
    }

    let after = s.assayer().health_summary();

    // Nothing an outcome taught has moved.
    assert_eq!(after.total_labels, 0, "an assessment must not count as a label");
    assert_eq!(after.eligible_labels, 0, "an assessment must not become base-rate evidence");
    assert_eq!(
        after.platt_refits_completed, before.platt_refits_completed,
        "an assessment must not refit the calibration",
    );
    assert_eq!(
        after.last_delta_cal.to_bits(),
        before.last_delta_cal.to_bits(),
        "an assessment must not move the calibration delta"
    );
    assert_eq!(
        after.platt_state, before.platt_state,
        "an assessment must not advance the calibration's convergence state",
    );

    // And the observational half did move, which is what stops the assertions
    // above from passing on an engine where nothing happens at all.
    assert!(after.total_assessments >= 1_000, "the assessments were actually served");
    let health = s.derive_default("alice").expect("assess").assessment.health;
    assert!(
        health.standardisation_observations > 0,
        "the cold ramp must have advanced, or this scenario proves only that an idle engine is idle",
    );

    assert_health_clean(&s);
}

/// The cold ramp discloses where it stands and spreads the whole
/// prior-to-empirical movement across its horizon, so that no single accepted
/// observation moves the uncertainty by more than a tenth of that total. This
/// is the ruling's own falsifier and the reason the ramp was adopted: under the
/// completion gate the coordinate system moved its entire distance between two
/// adjacent requests, and results either side of that step were results in
/// different coordinate systems that nothing on the wire distinguished.
///
/// Three things are held together here because they are one property. Every
/// assessment reports the phase and the accepted count of the snapshot it was
/// actually scored in, so a reader can place a result on the trajectory. The
/// count rises monotonically and the uncertainty falls monotonically with it,
/// from the class priors to the empirical moments. And the largest single step
/// is far inside a tenth of the total — the studies put it at the first
/// observation, at about one part in fifty-five.
///
/// The endpoints are the ones the ramp inherited rather than new ones: the ramp
/// redistributes the old cliff and moves neither end of it.
///
/// ´claim:scenario:the-cold-ramp-discloses-its-trajectory-and-no-single-observation-moves-the-uncertainty-by-more-than-a-tenth-of-the-total´
/// ´test:integration:cold-ramp-trajectory-stays-inside-the-falsifier´
#[test]
fn cold_ramp_trajectory_stays_inside_the_falsifier() {
    /// The uncertainty a cold instance publishes against the class priors,
    /// before any accepted observation.
    const PRIOR_ENDPOINT: f64 = 2.519_920_533_763_746_5;
    /// The uncertainty at the horizon, against the empirical moments.
    const EMPIRICAL_ENDPOINT: f64 = 0.790_569_415_042_094_9;
    /// Requests to serve while walking the ramp. Generous, because a refused
    /// observation advances no count and the walk is allowed to cost pace.
    const REQUESTS: usize = 4_000;
    /// A pause between requests, long enough for the steward to publish the
    /// advance the previous request offered.
    ///
    /// The assessment path does not wait for the steward and must not, so an
    /// unpaced loop outruns it: the steward drains a burst of observations and
    /// publishes a burst of snapshots, and the walk samples only a few of them.
    /// That would not make the ramp less gradual — every one of those snapshots
    /// was published, and each retired exactly one share — but it would leave
    /// this scenario measuring intervals rather than steps, which is not what
    /// the bound below is about. Pausing samples the trajectory that was
    /// actually published rather than a subsequence of it.
    const PACING: std::time::Duration = std::time::Duration::from_millis(2);

    let s = scenario(INSTANCE, SEED);

    // Walk the ramp, recording the uncertainty each assessment was scored at
    // against the accepted count of the snapshot it acquired. The count is the
    // snapshot's own, so this is the trajectory as a host would reconstruct it
    // from the compact health it was handed — not from anything internal.
    let mut trajectory: BTreeMap<usize, f64> = BTreeMap::new();
    let mut horizon: Option<usize> = None;
    for _ in 0..REQUESTS {
        let r = s.derive_default("alice").expect("assess should succeed");
        let health = &r.assessment.health;
        trajectory
            .entry(health.standardisation_observations)
            .or_insert(r.assessment.risk.uncertainty);
        if health.standardisation_phase.is_in_service() {
            horizon = Some(health.standardisation_observations);
            break;
        }
        std::thread::sleep(PACING);
    }

    let horizon = horizon.expect("the ramp must reach its horizon within the request budget");
    assert_eq!(horizon, 100, "the configured horizon is one hundred accepted observations");

    let first = *trajectory.get(&0).expect("the first assessment is scored at count zero");
    let last = *trajectory.get(&horizon).expect("an assessment is scored at the horizon");

    // The endpoints the ramp inherited, unchanged.
    assert!(
        (first - PRIOR_ENDPOINT).abs() < 1e-9,
        "the prior endpoint moved: expected {PRIOR_ENDPOINT}, got {first}",
    );
    assert!(
        (last - EMPIRICAL_ENDPOINT).abs() < 1e-9,
        "the empirical endpoint moved: expected {EMPIRICAL_ENDPOINT}, got {last}",
    );

    // The walk has to be dense enough for "largest single step" to mean
    // anything: a trajectory of two points would pass the bound below while
    // measuring the cliff the ramp exists to remove.
    let adjacent: Vec<(usize, f64)> = trajectory.iter().map(|(&k, &v)| (k, v)).collect();
    let steps: Vec<(usize, f64)> = adjacent
        .windows(2)
        .filter(|w| w[1].0 == w[0].0 + 1)
        .map(|w| (w[1].0, (w[1].1 - w[0].1).abs()))
        .collect();
    // Count one has to be among them, because it carries the largest step and
    // the pin below names it. A walk that missed it would still be measuring
    // the ramp, but it would not be measuring the step this scenario is about.
    assert!(
        trajectory.contains_key(&1),
        "the first accepted observation's snapshot was never sampled; the walk is not keeping up with the ramp",
    );
    assert!(
        steps.len() >= 40,
        "only {} single-observation steps were observed of a hundred-observation ramp; \
         the trajectory is too sparse for the bound to decide anything",
        steps.len(),
    );

    // Monotone: each accepted observation retires prior mass and never returns
    // any, so the uncertainty falls the whole way and never turns back.
    for w in adjacent.windows(2) {
        assert!(
            w[1].1 <= w[0].1 + 1e-12,
            "the uncertainty rose between counts {} and {}: {} then {}",
            w[0].0,
            w[1].0,
            w[0].1,
            w[1].1,
        );
    }

    // The falsifier itself.
    let total = (first - last).abs();
    let bound = total / 10.0;
    let (worst_at, worst) = steps
        .iter()
        .copied()
        .fold((0_usize, 0.0_f64), |acc, s| if s.1 > acc.1 { s } else { acc });
    assert!(
        worst > 0.0,
        "no step moved at all, so the bound below would pass on a ramp that never ran",
    );
    assert!(
        worst <= bound,
        "the ruling is falsified: the accepted observation at count {worst_at} moved the uncertainty by \
         {worst}, more than a tenth ({bound}) of the {total} total prior-to-empirical movement",
    );

    // Where the largest step falls, and how large it is, are the studies' own
    // prediction and are pinned here. Clearing the tenth is the ruling's
    // criterion; landing at the first observation at about one part in
    // fifty-five is what says the ramp is gradual in the shape the studies
    // computed, rather than merely gradual enough. A ramp that cleared the
    // bound while moving in a few large steps somewhere in the middle would
    // pass the assertion above and fail this one.
    assert_eq!(
        worst_at, 1,
        "the largest step should be the first accepted observation's, where the most prior mass stands",
    );
    let share = worst / total;
    assert!(
        (share - 0.017_995_4).abs() < 1e-6,
        "the largest step was {share} of the total movement, where the standard-library audit of this \
         fixture predicted about 0.0179954",
    );
}

/// The shape of a reckoning is exact, and so are its numbers, when the thing
/// repeated is the derivation rather than the assessment: across a hundred
/// derivations of one held assessment the sequence of tag kinds is identical
/// element for element and every location, magnitude and q is identical bit for
/// bit. A reckoning also never comes back empty.
///
/// This scenario used to repeat the *assessment* and allow its parameters to
/// wander inside a drift band, which was the wrong comparison twice over. It
/// compared results computed in coordinate systems that an accepted cold
/// observation is entitled to move between, so the band was standing in for a
/// legitimate coordinate change rather than for floating-point noise; and by
/// tolerating movement it could not see the one thing derivation actually
/// promises, which is exactness. Holding the assessment fixed removes the
/// coordinate change from the comparison and lets the promise be asserted as
/// the promise is written (´inv:guarantee:evidence-authority´).
///
/// ´claim:scenario:the-tag-kind-sequence-and-every-tag-parameter-are-exact-across-derivations-of-one-held-assessment´
/// ´test:integration:identical-inputs-tag-shape-stable´
#[test]
fn identical_inputs_tag_shape_stable() {
    let s = scenario(INSTANCE, SEED);
    let held = s.derive_default("bob").expect("first reckoning");
    let policy = ChannelPolicy::default();
    let challenge = ChallengeEstimate::default();
    let config = ResonanceConfig::default();

    let derive = |a: &torrust_assayer::RiskAssessment| -> ResonanceProfile {
        render_resonances(&derive_landscape(a, &policy, challenge), &a.risk, &config)
    };
    let kinds = |r: &ResonanceProfile| -> Vec<String> { r.tags.iter().map(|t| format!("{:?}", t.tag)).collect() };
    let params = |r: &ResonanceProfile| -> Vec<[u64; 3]> {
        r.tags
            .iter()
            .map(|t| [t.location.to_bits(), t.magnitude.to_bits(), t.q.to_bits()])
            .collect()
    };

    let baseline = derive(&held.assessment);
    let baseline_kinds = kinds(&baseline);
    let baseline_params = params(&baseline);
    assert!(!baseline_kinds.is_empty(), "a reckoning should emit at least one tag");

    for i in 1..=100 {
        let r = derive(&held.assessment);
        assert_eq!(kinds(&r), baseline_kinds, "derivation #{i}: tag kinds diverged");
        assert_eq!(
            params(&r),
            baseline_params,
            "derivation #{i}: a tag parameter diverged, and derivation of one held assessment is exact",
        );
    }
}

/// Purity is a promise about identical inputs, not about all inputs: two
/// distinct entity keys feed the identity layer differently and are each only
/// held to producing a well-formed, finite, in-range assessment. On a cold
/// world with no identity dimensions registered they may legitimately agree,
/// so demanding that different entities differ would be asserting a coincidence
/// of the current model state rather than a property of the engine.
///
/// ´claim:scenario:purity-binds-identical-inputs-and-distinct-entities-owe-only-well-formedness´
/// ´test:integration:identical-inputs-across-entities-well-formed´
#[test]
fn identical_inputs_across_entities_well_formed() {
    // This is the negative control for the bit-identical witness
    // (´test:integration:public-derive-reckoning-bit-identical´): purity is a contract
    // *relative to the inputs*. Distinct entity keys feed into the
    // identity layer and MAY produce different assessments — this test
    // only asserts that both sides remain finite and in range, not
    // that they are unequal (a cold engine with no identity
    // dimensions registered may legitimately produce identical
    // assessments for distinct entities, separation being a property a
    // registered dimension's encoder supplies
    // (´claim:identity:a-dimensions-encoder-is-stable-per-entity-and-separates-distinct-entities´)).
    let s = scenario(INSTANCE, SEED);
    let a = s.derive_default("alice").expect("alice");
    let b = s.derive_default("zebediah").expect("zeb");

    assert_reckoning_well_formed(&a, "alice p_bad");
    assert_reckoning_well_formed(&b, "zebediah p_bad");
}
