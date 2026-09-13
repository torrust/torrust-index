// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`decay_factor_zero_time`] | numerics | cites (´claim:numerics:a-zero-elapsed-interval-decays-by-exactly-unity´) |
//! | [`decay_factor_one_hour`] | numerics | One hour of elapsed time decays by exactly one factor of the rate, which is what makes the rate readable as a per-hour retention. The hour is the unit the whole configuration is expressed in — half-lives, forgetting factors, budgets — so the identity at one hour is what gives those numbers meaning. |
//! | [`decay_factor_29_day_half_life`] | numerics | cites (´claim:numerics:the-per-hour-rate-determines-the-half-life´) |
//! | [`decay_factor_290_day_half_life`] | numerics | cites (´claim:numerics:the-per-hour-rate-determines-the-half-life´) |
//! | [`decay_factor_always_positive`] | numerics | cites (´claim:numerics:the-decay-factor-always-lands-in-the-half-open-unit-interval´) |
//! | [`decay_factor_since_forward`] | numerics | cites (´claim:numerics:an-hour-of-elapsed-time-decays-by-exactly-one-factor-of-the-rate´) |
//! | [`integration_decay_factor_since_backward_clock`] | numerics | cites (´claim:numerics:a-backward-clock-decays-nothing-at-all´) |
//! | [`decay_factor_since_same_time`] | numerics | cites (´claim:numerics:a-zero-elapsed-interval-decays-by-exactly-unity´) |
//! | [`decay_factor_elapsed_zero`] | numerics | cites (´claim:numerics:a-zero-elapsed-interval-decays-by-exactly-unity´) |
//! | [`decay_factor_elapsed_one_hour`] | numerics | cites (´claim:numerics:an-hour-of-elapsed-time-decays-by-exactly-one-factor-of-the-rate´) |
//! | [`decay_composition_property`] | numerics | cites (´claim:numerics:decay-composes-over-consecutive-intervals´) |
//! | [`decay_factor_in_range`] | numerics | cites (´claim:numerics:the-decay-factor-always-lands-in-the-half-open-unit-interval´) |
//! | [`half_life_consistency`] | numerics | The half-life the system reports for a rate is the time at which that rate actually halves a value — decaying for exactly that long lands on one half. The half-life is derived by inverting the decay, so this closes the loop and makes the reported figure a measurement rather than a claim. |
//! | [`half_life_days_conversion`] | numerics | The half-life reported in days is the same quantity as the one reported in hours, divided by twenty-four. Both units are offered for readability, and they must never be able to disagree about how long a memory lasts. |
//! | [`half_life_specification_values`] | numerics | cites (´claim:numerics:the-per-hour-rate-determines-the-half-life´) |
//! | [`decay_count_scales_by_the_factor`] | numerics | Decaying a count is simply scaling it by the factor — a hundred at half strength is fifty. Counts age by the same arithmetic as anything else, so no separate ageing rule can drift out of step with the decay itself. |
//! | [`hours_to_decay_target_inverts_the_decay`] | numerics | Asking how long it takes to fall to a given fraction and then decaying for that long lands on that fraction. The inverse is what lets a policy be stated as a retention target rather than as a number of hours, and it is only useful if it inverts exactly. |
//! | [`decay_factor_gamma_one`] | numerics | A rate of one never forgets: no interval, however long, moves the factor off unity. This is the setting that turns decay off entirely, and it has to be exact rather than merely close — a factor a hair below one applied over a year of hours would erode a value substantially. |
//! | [`decay_factor_gamma_zero`] | numerics | A rate of zero forgets everything the moment any time passes, yet still leaves an empty interval untouched. The degenerate rate keeps the zero-interval identity rather than collapsing it, so the two edge rules do not contradict each other at their intersection. |
//! | [`decay_factor_negative_time_panics`] | numerics | A negative interval is refused outright in a debug build rather than silently producing a factor above one. Unlike a backward clock, which the timestamp entry point handles as a fact of life, a negative hour count reaching the raw arithmetic means a caller computed a difference the wrong way round, and that is worth failing loudly on. |
//! | [`decay_factor_identity_half_life`] | numerics | cites (´claim:numerics:the-per-hour-rate-determines-the-half-life´) |
//! | [`decay_monotonicity_in_time`] | numerics | Waiting longer never retains more: across intervals from nothing up to the clamp, each longer wait leaves a factor no larger than the shorter one before it. Age is meant to weaken evidence uniformly, and a fold anywhere in that curve would make some ages inexplicably worth more than younger ones. |
//! | [`decay_monotonicity_in_rate`] | numerics | Over a fixed interval, a rate nearer one retains more, all the way up the scale from a half to unity. This is what makes the rate a legible dial: turning it up lengthens memory, without a range where the relationship reverses and a longer-memory setting forgets faster. |
//! | [`decay_composition_property_many`] | numerics | cites (´claim:numerics:decay-composes-over-consecutive-intervals´) |
//! | [`decay_range_property_many`] | numerics | cites (´claim:numerics:the-decay-factor-always-lands-in-the-half-open-unit-interval´) |
//! | [`decay_boundary_property`] | numerics | cites (´claim:numerics:a-rate-of-one-never-decays-however-long-the-interval´) |
//! | [`decay_idempotence_property`] | numerics | cites (´claim:numerics:a-zero-elapsed-interval-decays-by-exactly-unity´) |

#![allow(clippy::suboptimal_flops)]
//! Integration tests for `torrust_assayer::numerics`.
//!
//! Uses the shared [`torrust_assayer::testing`] harness for deterministic
//! RNG ([`TestRng`](torrust_assayer::testing::TestRng)) and domain-aware
//! assertion helpers ([`assert_near`](torrust_assayer::testing::assert_near),
//! [`assert_positive`](torrust_assayer::testing::assert_positive),
//! [`assert_in_unit_interval`](torrust_assayer::testing::assert_in_unit_interval)).

use std::time::Duration;

use torrust_assayer::numerics_export::{
    decay_count, decay_factor, decay_factor_elapsed, decay_factor_since, half_life_days, half_life_hours, hours_to_decay_target,
};
use torrust_assayer::testing::{TestRng, assert_in_unit_interval, assert_near, assert_positive};
use torrust_assayer::types::{MAX_DECAY_HOURS, PersistentTimestamp};

/// Exact-equality tolerance (matches `DEFAULT_TOLERANCES.bit_identical`).
const EXACT: f64 = 0.0;
/// Tight float tolerance for analytically exact identities.
const TIGHT: f64 = 1e-15;
/// Numerical tolerance for compositions of floating-point decays.
const COMPOSITION: f64 = 1e-14;
/// Tolerance for derived/transcendental inversions (half-life, log-based).
const ANALYTIC: f64 = 1e-10;

// ─────────────────────────────────────────────────────────────────────────────
// Core decay function tests
// ─────────────────────────────────────────────────────────────────────────────

/// Seen from outside the crate, a zero elapsed interval still decays by
/// exactly unity. This is the published surface other packages call, so the
/// guarantee has to hold there and not merely in the module's own tests.
///
/// (´claim:numerics:a-zero-elapsed-interval-decays-by-exactly-unity´)
/// ´test:integration:decay-factor-zero-time´
#[test]
fn decay_factor_zero_time() {
    assert_near(decay_factor(0.999, 0.0), 1.0, TIGHT, "decay_factor(γ=0.999, dt=0)");
}

/// One hour of elapsed time decays by exactly one factor of the rate, which is
/// what makes the rate readable as a per-hour retention. The hour is the unit
/// the whole configuration is expressed in — half-lives, forgetting factors,
/// budgets — so the identity at one hour is what gives those numbers meaning.
///
/// ´claim:numerics:an-hour-of-elapsed-time-decays-by-exactly-one-factor-of-the-rate´
/// ´test:integration:decay-factor-one-hour´
#[test]
fn decay_factor_one_hour() {
    assert_near(decay_factor(0.999, 1.0), 0.999, TIGHT, "decay_factor(γ=0.999, dt=1h)");
}

/// The rate used for the shorter-memory configuration halves a value in about
/// twenty-nine days. Operators reason in days of memory rather than in
/// per-hour retentions, and this is where the one is anchored to the other.
///
/// (´claim:numerics:the-per-hour-rate-determines-the-half-life´)
/// ´test:integration:decay-factor-29-day-half-life´
#[test]
fn decay_factor_29_day_half_life() {
    let factor = decay_factor(0.999, 29.0 * 24.0);
    assert_near(factor, 0.5, 0.01, "29-day decay with γ=0.999");
}

/// Moving the rate one decimal place closer to unity lengthens the memory by
/// roughly a factor of ten, to some two hundred and ninety days. The
/// sensitivity is steep, which is why the long-memory configuration is
/// expressed as a rate rather than nudged by hand.
///
/// (´claim:numerics:the-per-hour-rate-determines-the-half-life´)
/// ´test:integration:decay-factor-290-day-half-life´
#[test]
fn decay_factor_290_day_half_life() {
    let factor = decay_factor(0.9999, 290.0 * 24.0);
    assert_near(factor, 0.5, 0.01, "290-day decay with γ=0.9999");
}

/// Even at the longest interval the clamp permits, the factor is still inside
/// the unit interval and still above zero. The clamp bound is the worst case
/// the arithmetic will ever be asked for, so it is the case worth naming.
///
/// (´claim:numerics:the-decay-factor-always-lands-in-the-half-open-unit-interval´)
/// ´test:integration:decay-factor-always-positive´
#[test]
fn decay_factor_always_positive() {
    let factor = decay_factor(0.999, MAX_DECAY_HOURS);
    assert_in_unit_interval(factor, "decay_factor(γ=0.999, dt=MAX_DECAY_HOURS)");
}

// ─────────────────────────────────────────────────────────────────────────────
// Persistent timestamp decay tests
// ─────────────────────────────────────────────────────────────────────────────

/// An hour expressed as a pair of timestamps decays by the same single factor
/// of the rate as an hour expressed any other way. The timestamp form has to
/// convert seconds to hours to get there, and a mistake in that conversion
/// would rescale every memory in the system at once.
///
/// (´claim:numerics:an-hour-of-elapsed-time-decays-by-exactly-one-factor-of-the-rate´)
/// ´test:integration:decay-factor-since-forward´
#[test]
fn decay_factor_since_forward() {
    let t0 = PersistentTimestamp::new(0, 0);
    let t1 = PersistentTimestamp::new(3600, 0);

    let factor = decay_factor_since(0.999, &t0, &t1);
    assert_near(factor, 0.999, ANALYTIC, "decay_factor_since forward 1h");
}

/// The backward-clock guarantee is part of the published surface too: a later
/// timestamp offered as the earlier one yields no decay rather than an error
/// or an amplification. Callers outside the crate are exactly the ones holding
/// timestamps from other hosts.
///
/// (´claim:numerics:a-backward-clock-decays-nothing-at-all´)
/// ´test:integration:integration-decay-factor-since-backward-clock´
#[test]
fn integration_decay_factor_since_backward_clock() {
    let t0 = PersistentTimestamp::new(3600, 0);
    let t1 = PersistentTimestamp::new(0, 0);

    let factor = decay_factor_since(0.999, &t0, &t1);
    assert_near(factor, 1.0, TIGHT, "decay_factor_since backward clock");
}

/// A timestamp compared against itself decays by unity, so re-reading a value
/// without the clock having moved leaves it exactly as it was.
///
/// (´claim:numerics:a-zero-elapsed-interval-decays-by-exactly-unity´)
/// ´test:integration:decay-factor-since-same-time´
#[test]
fn decay_factor_since_same_time() {
    let t = PersistentTimestamp::new(12345, 0);
    let factor = decay_factor_since(0.999, &t, &t);
    assert_near(factor, 1.0, TIGHT, "decay_factor_since same time");
}

// ─────────────────────────────────────────────────────────────────────────────
// Duration decay tests
// ─────────────────────────────────────────────────────────────────────────────

/// The duration-shaped entry point agrees with the others on an empty
/// interval, returning unity from the published surface as it does within.
///
/// (´claim:numerics:a-zero-elapsed-interval-decays-by-exactly-unity´)
/// ´test:integration:decay-factor-elapsed-zero´
#[test]
fn decay_factor_elapsed_zero() {
    let factor = decay_factor_elapsed(0.999, Duration::ZERO);
    assert_near(factor, 1.0, TIGHT, "decay_factor_elapsed zero duration");
}

/// An hour handed over as a duration of thirty-six hundred seconds decays by
/// one factor of the rate, matching what the timestamp pair and the bare hour
/// count give. The three entry points are interchangeable in meaning, which is
/// what lets a caller use whichever shape of elapsed time it happens to hold.
///
/// (´claim:numerics:an-hour-of-elapsed-time-decays-by-exactly-one-factor-of-the-rate´)
/// ´test:integration:decay-factor-elapsed-one-hour´
#[test]
fn decay_factor_elapsed_one_hour() {
    let factor = decay_factor_elapsed(0.999, Duration::from_secs(3600));
    assert_near(factor, 0.999, ANALYTIC, "decay_factor_elapsed one hour");
}

// ─────────────────────────────────────────────────────────────────────────────
// Composition property tests
// ─────────────────────────────────────────────────────────────────────────────

/// Ten hours followed by fifteen leaves a value exactly where twenty-five
/// hours in one step would have left it. The worked case pins the property
/// concretely, at intervals a reader can check by hand.
///
/// (´claim:numerics:decay-composes-over-consecutive-intervals´)
/// ´test:integration:decay-composition-property´
#[test]
fn decay_composition_property() {
    let gamma = 0.999;
    let a = 10.0;
    let b = 15.0;

    let composed = decay_factor(gamma, a) * decay_factor(gamma, b);
    let direct = decay_factor(gamma, a + b);

    assert_near(composed, direct, COMPOSITION, "f(a)·f(b) vs f(a+b)");
}

/// The range holds across deliberately awkward combinations — the shortest and
/// longest intervals, and rates from a half up to nearly one. Configuration is
/// not always drawn from the recommended band, so the bound must survive
/// values chosen carelessly as well as values chosen well.
///
/// (´claim:numerics:the-decay-factor-always-lands-in-the-half-open-unit-interval´)
/// ´test:integration:decay-factor-in-range´
#[test]
fn decay_factor_in_range() {
    let test_cases = [
        (0.999, 0.0),
        (0.999, 1.0),
        (0.999, 100.0),
        (0.9999, 1000.0),
        (0.5, 10.0),
        (0.99, MAX_DECAY_HOURS),
    ];

    for (gamma, dt) in test_cases {
        let factor = decay_factor(gamma, dt);
        assert_positive(factor, &format!("decay_factor({gamma}, {dt})"));
        assert!(factor <= 1.0, "decay_factor({gamma}, {dt}) = {factor} > 1");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Half-life computation tests
// ─────────────────────────────────────────────────────────────────────────────

/// The half-life the system reports for a rate is the time at which that rate
/// actually halves a value — decaying for exactly that long lands on one half.
/// The half-life is derived by inverting the decay, so this closes the loop
/// and makes the reported figure a measurement rather than a claim.
///
/// ´claim:numerics:the-per-hour-rate-determines-the-half-life´
/// ´test:integration:half-life-consistency´
#[test]
fn half_life_consistency() {
    let gamma = 0.999;
    let hl = half_life_hours(gamma);

    let factor = decay_factor(gamma, hl);
    assert_near(factor, 0.5, ANALYTIC, "decay at reported half-life");
}

/// The half-life reported in days is the same quantity as the one reported in
/// hours, divided by twenty-four. Both units are offered for readability, and
/// they must never be able to disagree about how long a memory lasts.
///
/// ´claim:numerics:the-half-life-agrees-whether-reported-in-hours-or-in-days´
/// ´test:integration:half-life-days-conversion´
#[test]
fn half_life_days_conversion() {
    let gamma = 0.999;
    let hl_hours = half_life_hours(gamma);
    let hl_days = half_life_days(gamma);

    assert_near(hl_hours / 24.0, hl_days, ANALYTIC, "hours/24 vs days");
}

/// The two rates the specification names come out at the memory lengths the
/// specification promises for them — roughly a month and roughly ten months.
/// The published tables are what operators tune against, so the code and the
/// tables have to be pinned to each other somewhere.
///
/// (´claim:numerics:the-per-hour-rate-determines-the-half-life´)
/// ´test:integration:half-life-specification-values´
#[test]
fn half_life_specification_values() {
    assert_near(half_life_days(0.999), 29.0, 1.0, "γ=0.999 half-life (days)");
    assert_near(half_life_days(0.9999), 290.0, 10.0, "γ=0.9999 half-life (days)");
}

// ─────────────────────────────────────────────────────────────────────────────
// Edge case tests
// ─────────────────────────────────────────────────────────────────────────────

/// Decaying a count is simply scaling it by the factor — a hundred at half
/// strength is fifty. Counts age by the same arithmetic as anything else, so
/// no separate ageing rule can drift out of step with the decay itself.
///
/// ´claim:numerics:decaying-a-count-scales-it-by-the-factor´
/// ´test:integration:decay-count-scales-by-the-factor´
#[test]
fn decay_count_scales_by_the_factor() {
    let count = decay_count(100.0, 0.5);
    assert_near(count, 50.0, ANALYTIC, "decay_count(100, 0.5)");
}

/// Asking how long it takes to fall to a given fraction and then decaying for
/// that long lands on that fraction. The inverse is what lets a policy be
/// stated as a retention target rather than as a number of hours, and it is
/// only useful if it inverts exactly.
///
/// ´claim:numerics:the-hours-to-a-target-fraction-invert-the-decay´
/// ´test:integration:hours-to-decay-target-inverts-the-decay´
#[test]
fn hours_to_decay_target_inverts_the_decay() {
    let gamma = 0.999;
    let target = 0.25;

    let hours = hours_to_decay_target(gamma, target);
    let factor = decay_factor(gamma, hours);

    assert_near(factor, target, ANALYTIC, "hours_to_decay_target round-trip");
}

/// A rate of one never forgets: no interval, however long, moves the factor
/// off unity. This is the setting that turns decay off entirely, and it has to
/// be exact rather than merely close — a factor a hair below one applied over
/// a year of hours would erode a value substantially.
///
/// ´claim:numerics:a-rate-of-one-never-decays-however-long-the-interval´
/// ´test:integration:decay-factor-gamma-one´
#[test]
fn decay_factor_gamma_one() {
    assert_near(decay_factor(1.0, 0.0), 1.0, TIGHT, "γ=1, dt=0");
    assert_near(decay_factor(1.0, 100.0), 1.0, TIGHT, "γ=1, dt=100");
    assert_near(decay_factor(1.0, MAX_DECAY_HOURS), 1.0, TIGHT, "γ=1, dt=MAX");
}

/// A rate of zero forgets everything the moment any time passes, yet still
/// leaves an empty interval untouched. The degenerate rate keeps the
/// zero-interval identity rather than collapsing it, so the two edge rules do
/// not contradict each other at their intersection.
///
/// ´claim:numerics:a-rate-of-zero-forgets-everything-once-any-time-has-passed-but-not-before´
/// ´test:integration:decay-factor-gamma-zero´
#[test]
fn decay_factor_gamma_zero() {
    assert_near(decay_factor(0.0, 0.0), 1.0, TIGHT, "γ=0, dt=0");
    assert_near(decay_factor(0.0, 1.0), 0.0, EXACT, "γ=0, dt=1");
    assert_near(decay_factor(0.0, 100.0), 0.0, EXACT, "γ=0, dt=100");
}

/// A negative interval is refused outright in a debug build rather than
/// silently producing a factor above one. Unlike a backward clock, which the
/// timestamp entry point handles as a fact of life, a negative hour count
/// reaching the raw arithmetic means a caller computed a difference the wrong
/// way round, and that is worth failing loudly on.
///
/// ´claim:numerics:a-negative-elapsed-interval-is-a-caller-error-caught-in-debug-builds´
/// ´test:integration:decay-factor-negative-time-panics´
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "dt_hours must be non-negative")]
fn decay_factor_negative_time_panics() {
    let _ = decay_factor(0.999, -1.0);
}

/// The rate used for identity-scale memory halves in about a fortnight,
/// markedly shorter than the general-purpose rates. Identity evidence is meant
/// to be the fastest-moving of the memories, and its half-life is where that
/// intent is written down in a form that can fail.
///
/// (´claim:numerics:the-per-hour-rate-determines-the-half-life´)
/// ´test:integration:decay-factor-identity-half-life´
#[test]
fn decay_factor_identity_half_life() {
    assert_near(half_life_days(0.998), 14.4, 1.0, "γ=0.998 half-life (days)");
}

// ─────────────────────────────────────────────────────────────────────────────
// Property-based tests with deterministic pseudo-randomness
// ─────────────────────────────────────────────────────────────────────────────

/// Waiting longer never retains more: across intervals from nothing up to the
/// clamp, each longer wait leaves a factor no larger than the shorter one
/// before it. Age is meant to weaken evidence uniformly, and a fold anywhere
/// in that curve would make some ages inexplicably worth more than younger
/// ones.
///
/// ´claim:numerics:decay-never-increases-as-the-interval-lengthens´
/// ´test:integration:decay-monotonicity-in-time´
#[test]
fn decay_monotonicity_in_time() {
    let gamma = 0.999;
    let dts = [0.0, 1.0, 10.0, 100.0, 1000.0, MAX_DECAY_HOURS];

    for window in dts.windows(2) {
        let f1 = decay_factor(gamma, window[0]);
        let f2 = decay_factor(gamma, window[1]);
        assert!(
            f2 <= f1,
            "decay should decrease over time: f({}) = {} > f({}) = {}",
            window[0],
            f1,
            window[1],
            f2
        );
    }
}

/// Over a fixed interval, a rate nearer one retains more, all the way up the
/// scale from a half to unity. This is what makes the rate a legible dial:
/// turning it up lengthens memory, without a range where the relationship
/// reverses and a longer-memory setting forgets faster.
///
/// ´claim:numerics:a-higher-rate-retains-more-over-the-same-interval´
/// ´test:integration:decay-monotonicity-in-rate´
#[test]
fn decay_monotonicity_in_rate() {
    let dt = 100.0;
    let gammas = [0.5, 0.9, 0.99, 0.999, 0.9999, 1.0];

    for window in gammas.windows(2) {
        let f1 = decay_factor(window[0], dt);
        let f2 = decay_factor(window[1], dt);
        assert!(
            f2 >= f1,
            "decay factor should increase with γ: f({}) = {} > f({}) = {}",
            window[0],
            f1,
            window[1],
            f2
        );
    }
}

/// Composition survives sampling: across a hundred rates spread over the whole
/// upper half of the range and arbitrary splits of an interval, decaying twice
/// matches decaying once. The worked case could have been a coincidence of its
/// particular numbers; this shows it is not.
///
/// (´claim:numerics:decay-composes-over-consecutive-intervals´)
/// ´test:integration:decay-composition-property-many´
#[test]
fn decay_composition_property_many() {
    let mut rng = TestRng::new(42);

    for _ in 0..100 {
        let gamma = 0.5 + rng.next_f64() * 0.5;
        let a = rng.next_f64() * MAX_DECAY_HOURS / 2.0;
        let b = rng.next_f64() * MAX_DECAY_HOURS / 2.0;

        let composed = decay_factor(gamma, a) * decay_factor(gamma, b);
        let direct = decay_factor(gamma, a + b);

        assert_near(
            composed,
            direct,
            COMPOSITION,
            &format!("composition for γ={gamma}, a={a}, b={b}"),
        );
    }
}

/// Across a hundred sampled rates from the band the system actually
/// configures, paired with intervals anywhere up to the clamp, the factor
/// stays positive and never exceeds one.
///
/// (´claim:numerics:the-decay-factor-always-lands-in-the-half-open-unit-interval´)
/// ´test:integration:decay-range-property-many´
#[test]
fn decay_range_property_many() {
    let mut rng = TestRng::new(12345);

    for _ in 0..100 {
        let gamma = 0.99 + rng.next_f64() * 0.01;
        let dt = rng.next_f64() * MAX_DECAY_HOURS;

        let factor = decay_factor(gamma, dt);
        assert_positive(factor, &format!("decay_factor({gamma}, {dt})"));
        assert!(factor <= 1.0, "decay_factor({gamma}, {dt}) = {factor} > 1");
    }
}

/// A rate of one holds at unity for a hundred sampled intervals across the
/// whole permitted range, not merely at the round durations. Turning decay off
/// has to be genuinely off everywhere, since the exponentiation would
/// otherwise be free to drift by a rounding step at awkward inputs.
///
/// (´claim:numerics:a-rate-of-one-never-decays-however-long-the-interval´)
/// ´test:integration:decay-boundary-property´
#[test]
fn decay_boundary_property() {
    let mut rng = TestRng::new(99999);

    for _ in 0..100 {
        let dt = rng.next_f64() * MAX_DECAY_HOURS;
        let factor = decay_factor(1.0, dt);
        assert_near(factor, 1.0, TIGHT, &format!("γ=1.0, dt={dt}"));
    }
}

/// An empty interval leaves a value alone whatever the rate — sampled across
/// the whole span from nearly-immediate forgetting to nearly none. Decay is
/// applied opportunistically, so a repeat application within the same instant
/// must be a no-op at every configured rate, not only at the sensible ones.
///
/// (´claim:numerics:a-zero-elapsed-interval-decays-by-exactly-unity´)
/// ´test:integration:decay-idempotence-property´
#[test]
fn decay_idempotence_property() {
    let mut rng = TestRng::new(77777);

    for _ in 0..100 {
        let gamma = 0.1 + rng.next_f64() * 0.9;
        let factor = decay_factor(gamma, 0.0);
        assert_near(factor, 1.0, TIGHT, &format!("γ={gamma}, dt=0"));
    }
}
