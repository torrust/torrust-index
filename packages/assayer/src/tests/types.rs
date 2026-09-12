// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`system_time_round_trip_post_epoch`] | types | A moment after the epoch is stored as plain seconds and nanoseconds since it, and restores to exactly the moment it came from. Nothing is rounded on the way in, so a checkpointed instant is the same instant when it is read back. |
//! | [`system_time_round_trip_pre_epoch`] | types | A moment before the epoch borrows a second: half a second early is stored as minus one second plus five hundred million nanoseconds, and still restores exactly. The invariant that nanoseconds lie inside a second is therefore kept without giving up pre-epoch times altogether. |
//! | [`system_time_round_trip_exact_epoch`] | types | The epoch itself is the zero of the encoding, both fields empty, so the boundary between the forward and the borrow-a-second representations is a single unambiguous point rather than a place where the two disagree. |
//! | [`hours_since_one_hour`] | types | cites (´claim:types:an-elapsed-interval-between-two-timestamps-is-reported-in-hours´) |
//! | [`hours_since_fractional_nanos`] | types | Nanoseconds inside the second carry through into the hour figure: eighteen hundred and a half seconds reads as its exact fraction of an hour rather than as the whole-second part alone. The interval is accumulated in nanoseconds and converted once, at the end. |
//! | [`hours_since_backward_clamps_to_zero`] | types | cites (´claim:types:an-interval-measured-backwards-reports-zero-hours´) |
//! | [`hours_since_enormous_clamps_to_max`] | types | cites (´claim:types:an-elapsed-measure-beyond-the-decay-horizon-is-reported-as-the-horizon´) |
//! | [`hours_since_identical_is_zero`] | types | cites (´claim:types:a-timestamp-is-zero-hours-from-itself´) |
//! | [`new_panics_on_invalid_nanos`] | types | cites (´claim:types:debug-builds-trip-the-invariant-on-nanoseconds-that-reach-a-full-second´) |
//! | [`to_system_time_extreme_does_not_panic`] | types | Extreme second counts at both ends of the signed range convert without panicking; where the platform's own time type cannot represent the moment, the conversion simply yields nothing. An absurd or corrupted persisted timestamp cannot take the process down when it is read. |
//! | [`timestamp_nanos_normalised_release`] | types | In release builds an overflowing nanosecond count is normalised — the surplus carried into the seconds — rather than bringing the host down. A corrupted persisted timestamp costs a wrong moment, which the decay clamps already bound, and not an outage. Pinned here rather than in the public suite because the sealed fields are what the normalisation is visible on. |
//! | [`timestamp_carry_saturates_at_the_representable_maximum`] | types | A carry with nowhere left to go saturates at the greatest representable instant instead of wrapping to the earliest one. The release profile carries no overflow checks, so the carry that normalisation performs used to wrap the second count: the latest moment the type can express became the earliest, and because the ordering is derived from the second count first, every comparison against it inverted. That is the one failure this domain cannot absorb — decay and staleness are read from the ordering rather than from the magnitude — so the saturated value is pinned together with its place in the order: at the maximum, and above every instant below it. Pinned beside its sibling for the same reason, that the sealed fields are what saturation is visible on. |
//! | [`challenge_estimate_constructor_refuses_malformed_pairs`] | types | A challenge estimate refuses a malformed pair at construction — a mean that is not a number, a mean outside the unit interval, or a negative variance — and admits an in-domain pair, echoing it back through its readers. Refusal at construction is what lets the derivation take the estimate as given: the audit found a slightly negative mean being silently clamped to zero per call, which turned Challenge into a dominated action for a reason that had nothing to do with its rewards. |

//! Crate-level tests for the types module.

use std::time::{Duration, UNIX_EPOCH};

use crate::testing::{DEFAULT_TOLERANCES, assert_near};
use crate::types::{MAX_DECAY_HOURS, PersistentTimestamp};

// ═══════════════════════════════════════════════════════════════════════════════
// SystemTime Round-Trip
// ═══════════════════════════════════════════════════════════════════════════════

/// A moment after the epoch is stored as plain seconds and nanoseconds since
/// it, and restores to exactly the moment it came from. Nothing is rounded on
/// the way in, so a checkpointed instant is the same instant when it is read
/// back.
///
/// ´claim:types:a-post-epoch-moment-is-stored-as-plain-seconds-and-nanoseconds-and-restores-exactly´
/// ´test:crate:system-time-round-trip-post-epoch´
#[test]
fn system_time_round_trip_post_epoch() {
    let original = UNIX_EPOCH + Duration::new(1_700_000_000, 123_456_789);
    let pts = PersistentTimestamp::from_system_time(original);

    assert_eq!(pts.seconds, 1_700_000_000);
    assert_eq!(pts.nanos, 123_456_789);

    let restored = pts.to_system_time().expect("post-epoch should convert");
    assert_eq!(restored, original);
}

/// A moment before the epoch borrows a second: half a second early is stored
/// as minus one second plus five hundred million nanoseconds, and still
/// restores exactly. The invariant that nanoseconds lie inside a second is
/// therefore kept without giving up pre-epoch times altogether.
///
/// ´claim:types:a-pre-epoch-moment-borrows-a-second-to-keep-nanoseconds-non-negative-and-still-restores-exactly´
/// ´test:crate:system-time-round-trip-pre-epoch´
#[test]
fn system_time_round_trip_pre_epoch() {
    // 0.5 seconds before epoch
    let original = UNIX_EPOCH - Duration::new(0, 500_000_000);
    let pts = PersistentTimestamp::from_system_time(original);

    assert_eq!(pts.seconds, -1);
    assert_eq!(pts.nanos, 500_000_000);

    let restored = pts.to_system_time().expect("pre-epoch should convert");
    assert_eq!(restored, original);
}

/// The epoch itself is the zero of the encoding, both fields empty, so the
/// boundary between the forward and the borrow-a-second representations is a
/// single unambiguous point rather than a place where the two disagree.
///
/// ´claim:types:the-epoch-itself-is-the-zero-of-the-encoding´
/// ´test:crate:system-time-round-trip-exact-epoch´
#[test]
fn system_time_round_trip_exact_epoch() {
    let pts = PersistentTimestamp::from_system_time(UNIX_EPOCH);
    assert_eq!(pts.seconds, 0);
    assert_eq!(pts.nanos, 0);

    let restored = pts.to_system_time().expect("epoch should convert");
    assert_eq!(restored, UNIX_EPOCH);
}

// ═══════════════════════════════════════════════════════════════════════════════
// hours_since
// ═══════════════════════════════════════════════════════════════════════════════

/// An hour's worth of seconds separating two timestamps reads as exactly one
/// hour, the unit the decay half-lives are expressed in.
///
/// (´claim:types:an-elapsed-interval-between-two-timestamps-is-reported-in-hours´)
/// ´test:crate:hours-since-one-hour´
#[test]
fn hours_since_one_hour() {
    let t0 = PersistentTimestamp::new(1_000_000, 0);
    let t1 = PersistentTimestamp::new(1_003_600, 0);
    let hours = t1.hours_since(&t0);
    assert_near(hours, 1.0, DEFAULT_TOLERANCES.default, "hours_since_one_hour");
}

/// Nanoseconds inside the second carry through into the hour figure: eighteen
/// hundred and a half seconds reads as its exact fraction of an hour rather
/// than as the whole-second part alone. The interval is accumulated in
/// nanoseconds and converted once, at the end.
///
/// ´claim:types:sub-second-precision-carries-through-into-the-hour-figure´
/// ´test:crate:hours-since-fractional-nanos´
#[test]
fn hours_since_fractional_nanos() {
    let t0 = PersistentTimestamp::new(0, 0);
    let t1 = PersistentTimestamp::new(1800, 500_000_000); // 1800.5 seconds
    let hours = t1.hours_since(&t0);
    let expected = 1800.5 / 3600.0;
    assert_near(hours, expected, DEFAULT_TOLERANCES.default, "hours_since_fractional_nanos");
}

/// Reversing the two operands gives zero rather than the negation of the
/// forward answer, so it is the direction of the comparison that decides and
/// not the sign of the difference.
///
/// (´claim:types:an-interval-measured-backwards-reports-zero-hours´)
/// ´test:crate:hours-since-backward-clamps-to-zero´
#[test]
fn hours_since_backward_clamps_to_zero() {
    let later = PersistentTimestamp::new(2000, 0);
    let earlier = PersistentTimestamp::new(1000, 0);
    // Calling hours_since with self=earlier, other=later → backward
    assert_near(
        earlier.hours_since(&later),
        0.0,
        DEFAULT_TOLERANCES.bit_identical,
        "hours_since_backward_clamps_to_zero",
    );
}

/// A ten-year gap is reported as the horizon, the same answer a two-year gap
/// gives: past the cap, further age carries no further information.
///
/// (´claim:types:an-elapsed-measure-beyond-the-decay-horizon-is-reported-as-the-horizon´)
/// ´test:crate:hours-since-enormous-clamps-to-max´
#[test]
fn hours_since_enormous_clamps_to_max() {
    let t0 = PersistentTimestamp::new(0, 0);
    let t1 = PersistentTimestamp::new(10 * 365 * 24 * 3600, 0); // ~10 years
    let hours = t1.hours_since(&t0);
    assert_near(
        hours,
        MAX_DECAY_HOURS,
        DEFAULT_TOLERANCES.bit_identical,
        "hours_since_enormous_clamps_to_max",
    );
}

/// A timestamp compared with itself gives exactly zero even when its
/// nanosecond field sits at the top of its range, so the identity does not
/// depend on the sub-second part being empty.
///
/// (´claim:types:a-timestamp-is-zero-hours-from-itself´)
/// ´test:crate:hours-since-identical-is-zero´
#[test]
fn hours_since_identical_is_zero() {
    let t = PersistentTimestamp::new(1_700_000_000, 999_999_999);
    assert_near(
        t.hours_since(&t),
        0.0,
        DEFAULT_TOLERANCES.bit_identical,
        "hours_since_identical_is_zero",
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Construction Edge Cases
// ═══════════════════════════════════════════════════════════════════════════════

/// Exactly one billion nanoseconds is already one too many — the invariant is
/// strict rather than a guard against gross overflow — and a debug build
/// refuses the construction outright.
///
/// (´claim:types:debug-builds-trip-the-invariant-on-nanoseconds-that-reach-a-full-second´)
/// ´test:crate:new-panics-on-invalid-nanos´
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "nanos must be < 1_000_000_000")]
fn new_panics_on_invalid_nanos() {
    let _ = PersistentTimestamp::new(0, 1_000_000_000);
}

/// Extreme second counts at both ends of the signed range convert without
/// panicking; where the platform's own time type cannot represent the moment,
/// the conversion simply yields nothing. An absurd or corrupted persisted
/// timestamp cannot take the process down when it is read.
///
/// ´claim:types:extreme-second-counts-convert-without-panicking-yielding-nothing-when-unrepresentable´
/// ´test:crate:to-system-time-extreme-does-not-panic´
#[test]
fn to_system_time_extreme_does_not_panic() {
    let _ = PersistentTimestamp::new(i64::MAX, 999_999_999).to_system_time();
    let _ = PersistentTimestamp::new(i64::MIN + 1, 0).to_system_time();
    let _ = PersistentTimestamp::new(0, 0).to_system_time();
}

/// In release builds an overflowing nanosecond count is normalised — the
/// surplus carried into the seconds — rather than bringing the host down. A
/// corrupted persisted timestamp costs a wrong moment, which the decay clamps
/// already bound, and not an outage. Pinned here rather than in the public
/// suite because the sealed fields are what the normalisation is visible on.
///
/// ´claim:types:release-builds-normalise-overflowing-nanoseconds-instead-of-crashing-the-host´
/// ´test:crate:timestamp-nanos-normalised-release´
#[test]
#[cfg(not(debug_assertions))]
fn timestamp_nanos_normalised_release() {
    let ts = PersistentTimestamp::new(100, 1_500_000_000);
    assert!(ts.nanos < 1_000_000_000);
    assert_eq!(ts.seconds, 101, "the surplus second carries into the seconds");
}

/// A carry with nowhere left to go saturates at the greatest representable
/// instant instead of wrapping to the earliest one. The release profile
/// carries no overflow checks, so the carry that normalisation performs used
/// to wrap the second count: the latest moment the type can express became the
/// earliest, and because the ordering is derived from the second count first,
/// every comparison against it inverted. That is the one failure this domain
/// cannot absorb — decay and staleness are read from the ordering rather than
/// from the magnitude — so the saturated value is pinned together with its
/// place in the order: at the maximum, and above every instant below it.
/// Pinned beside its sibling for the same reason, that the sealed fields are
/// what saturation is visible on.
///
/// ´claim:types:a-nanosecond-carry-past-the-representable-maximum-saturates-and-keeps-the-ordering´
/// ´test:crate:timestamp-carry-saturates-at-the-representable-maximum´
#[test]
#[cfg(not(debug_assertions))]
fn timestamp_carry_saturates_at_the_representable_maximum() {
    let saturated = PersistentTimestamp::new(i64::MAX, 1_000_000_000);
    assert_eq!(
        saturated.seconds,
        i64::MAX,
        "a carry past the representable range holds at the maximum second count"
    );
    assert_eq!(
        saturated.nanos, 999_999_999,
        "saturation takes the whole instant to the maximum, not the second count alone"
    );

    let predecessor = PersistentTimestamp::new(i64::MAX, 999_999_999);
    assert!(
        saturated >= predecessor,
        "an input at or beyond the representable range may not order below an earlier one"
    );
    assert!(
        saturated > PersistentTimestamp::new(i64::MAX - 1, 999_999_999),
        "the saturated instant still orders above everything strictly below it"
    );
}

/// A challenge estimate refuses a malformed pair at construction — a mean
/// that is not a number, a mean outside the unit interval, or a negative
/// variance — and admits an in-domain pair, echoing it back through its
/// readers. Refusal at construction is what lets the derivation take the
/// estimate as given: the audit found a slightly negative mean being
/// silently clamped to zero per call, which turned Challenge into a
/// dominated action for a reason that had nothing to do with its rewards.
///
/// ´claim:types:a-malformed-challenge-posterior-is-refused-where-the-caller-can-fix-it´
/// ´test:crate:challenge-estimate-constructor-refuses-malformed-pairs´
#[test]
fn challenge_estimate_constructor_refuses_malformed_pairs() {
    use crate::types::ChallengeEstimate;

    // Refused: the exact shape the audit measured being clamped.
    assert!(ChallengeEstimate::new(-0.01, 0.02).is_err());
    // Refused: non-finite mean, out-of-range mean, negative or
    // non-finite variance.
    assert!(ChallengeEstimate::new(f64::NAN, 0.02).is_err());
    assert!(ChallengeEstimate::new(1.5, 0.02).is_err());
    assert!(ChallengeEstimate::new(0.4, -1.0).is_err());
    assert!(ChallengeEstimate::new(0.4, f64::INFINITY).is_err());

    // Admitted: an in-domain pair, echoed exactly.
    let ok = ChallengeEstimate::new(0.4, 0.02).expect("in-domain pair admitted");
    assert!((ok.q_c() - 0.4).abs() < f64::EPSILON);
    assert!((ok.variance() - 0.02).abs() < f64::EPSILON);

    // The interval is closed: both endpoints are declarable.
    assert!(ChallengeEstimate::new(0.0, 0.0).is_ok());
    assert!(ChallengeEstimate::new(1.0, 0.0).is_ok());
}
