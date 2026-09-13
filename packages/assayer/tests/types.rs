// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`id_trait_bounds`] | types | Every identifier the crate hands out can be cloned, compared, hashed and carried across threads, and the numeric ones are Copy besides. These are the bounds that let identifiers be used as map keys and be shared into worker threads without ceremony at each call site. |
//! | [`action_ordering`] | types | Actions are ordered by severity — allow below challenge, challenge below slow, slow below block — so combining two decisions is a matter of taking the larger. Policy code escalates by comparison rather than by a hand- written table that could disagree with itself. |
//! | [`persistent_timestamp_hours_since_forward`] | types | An elapsed interval between two persistent timestamps is reported in hours: an hour's worth of seconds reads as one. Hours are the unit the decay half- lives are written in, so the conversion belongs to the timestamp rather than to each caller. |
//! | [`persistent_timestamp_hours_since_backward`] | types | Asking for the interval from a later moment to an earlier one gives zero, not a negative number. A clock that steps backwards — an upstream correction, a restored checkpoint — cannot un-decay anything or drive a decay factor above one. |
//! | [`persistent_timestamp_hours_since_clamped`] | types | An elapsed measure longer than the decay horizon is reported as the horizon itself: a two-year gap reads as one year's worth of hours. Past that point the exponential has decayed to nothing anyway, and the cap keeps the arithmetic away from the magnitudes where it stops being meaningful. |
//! | [`dyadic_ancestor_lo_depth_16`] | types | A dyadic lower bound keeps the coordinate's leading bits down to the requested depth and zeroes everything below them. Every coordinate sharing that prefix maps to the same bound, which is what makes the bound an address for a whole region rather than for a point. |
//! | [`dyadic_ancestor_lo_depth_zero`] | types | Depth zero names the whole coordinate space, so its lower bound is zero whichever coordinate was asked about: at the root there is only one region to be in. |
//! | [`dyadic_ancestor_lo_depth_128`] | types | At the deepest level each coordinate is its own region, so the lower bound is the coordinate itself and no bits are discarded. |
//! | [`is_dyadic_valid`] | types | An interval counts as dyadic at a depth when it is aligned to that depth's cell size and spans exactly one such cell: at depth one both halves of the space qualify. Routing structures lean on this so a key cannot claim a depth its extent does not support. |
//! | [`is_dyadic_depth_zero`] | types | At depth zero only the entire coordinate space qualifies; an interval one short of the maximum does not, however nearly it covers everything. The root is a specific interval, not a description of a large one. |
//! | [`is_dyadic_depth_128`] | types | At the deepest level only a degenerate interval qualifies — one whose bounds coincide. A two-point span claiming that depth is rejected. |
//! | [`is_dyadic_rejects_misaligned`] | types | An interval whose lower bound is not a multiple of its depth's cell size is rejected even when its extent looks plausible. Alignment is what makes the hierarchy nest, so a shifted interval is not merely unusual but unaddressable. |
//! | [`is_dyadic_rejects_wrong_width`] | types | An interval correctly aligned but two short of its depth's cell width is rejected too. Both the start and the extent have to match the claimed depth; getting one of them right is not enough. |
//! | [`entity_key_from_vec`] | types | An entity key preserves exactly the bytes it was built from, so a host's identifier reaches the cache unchanged and the same identifier always keys the same record. |
//! | [`entity_key_from_slice`] | types | cites (´claim:types:an-entity-key-preserves-the-bytes-it-was-built-from´) |
//! | [`entity_key_clone_cheap`] | types | Cloning an entity key shares the underlying bytes instead of copying them — after a clone both handles report two references to one allocation. Keys are cloned on every cache write, so sharing is what keeps that path from copying an identifier per request. |
//! | [`sentinel_id_copy_clone`] | types | An identifier is a Copy type: passing it on leaves the original usable and the copy equal to it, so identifiers can be scattered through call sites without borrow plumbing. |
//! | [`sentinel_id_hash_eq`] | types | Equal identifiers hash alike, so a set holds one entry per identity rather than one per insertion. A registry keyed by identifier therefore counts identities, not the times they were mentioned. |
//! | [`model_id_variants_debug`] | types | Every model identity renders differently in diagnostics: the operational, sister and anchor models and a per-axis one are all distinguishable in a log line, so a report naming a model is unambiguous about which it means. |
//! | [`model_id_axis_hash`] | types | Model identities differing only in which outcome axis they name hash apart, so per-axis models keyed in one map do not collapse into a single entry. |
//! | [`assessment_id_is_u64`] | types | An assessment identifier carries the full range of a sixty-four-bit counter and converts back to it exactly at both ends. The counter is never expected to wrap, so nothing narrows it on the way out. |
//! | [`timestamp_now_positive`] | types | The current time is a moment after the epoch: the clamped elapsed reader — the only interval a host can take — reports a positive gap from the epoch to now. With the fields sealed, positivity of the reading is the whole of what the surface promises a host about `now()`; the field-level invariants are pinned in the crate suite, where the fields are reachable. |
//! | [`timestamp_nanos_invariant_panics`] | types | In debug builds that same overflowing nanosecond count trips the invariant at once, so the bug is found where it is written rather than surviving into a checkpoint that some later reader has to forgive. |
//! | [`hours_since_zero_gap`] | types | A timestamp is zero hours from itself, so a decay applied at the very moment of the last update leaves the value it decays untouched. |
//! | [`hours_since_fractional`] | types | cites (´claim:types:an-elapsed-interval-between-two-timestamps-is-reported-in-hours´) |
//! | [`hours_since_exactly_max`] | types | A gap of exactly one year lands on the decay horizon itself, so the horizon is a value the arithmetic reaches and not only one the cap imposes. The two agree at the boundary rather than meeting at a step. |
//! | [`hours_since_just_under_max`] | types | A gap short of a year is reported in full: three hundred and sixty-four days read as their own hours, below the horizon. The cap bites only beyond the horizon and does not flatten the long intervals just short of it. |
//! | [`hours_since_nanos_precision`] | types | A single nanosecond registers as a positive interval, vanishingly small but not zero. The measurement is not truncated to whole seconds on its way to hours, so events within one second are still separated by their gap. |
//! | [`dyadic_ancestor_lo_depth_4`] | types | cites (´claim:types:a-dyadic-lower-bound-keeps-the-leading-depth-bits-and-zeroes-the-rest´) |
//! | [`dyadic_ancestor_lo_depth_8`] | types | cites (´claim:types:a-dyadic-lower-bound-keeps-the-leading-depth-bits-and-zeroes-the-rest´) |
//! | [`cell_interval_display`] | types | A cell interval renders both its bounds and its depth in diagnostics, so an interval printed into a log can be read back as the region it describes rather than as an opaque handle. |
//! | [`cell_interval_width`] | types | A cell interval's width counts its upper bound as included: a span from zero to two hundred and fifty-five is two hundred and fifty-six wide, and an interval whose bounds coincide is one wide rather than zero. The convention deliberately departs from half-open notation, and the arithmetic saturates rather than overflowing when the interval covers the entire space. |
//! | [`dyadic_ancestor_hi_depth_zero`] | types | Depth zero's upper bound is the largest representable coordinate, so the root region genuinely spans everything rather than merely most of it. |
//! | [`dyadic_ancestor_hi_depth_128`] | types | At the deepest level the upper bound coincides with the lower one: a single point has no extent to report. |
//! | [`dyadic_ancestor_hi_depth_1`] | types | The two halves at depth one abut exactly: the lower half ends one below the midpoint where the upper half begins, and the upper half ends at the largest coordinate. No coordinate falls between them and none falls outside them. |
//! | [`duration_to_hours_zero`] | types | A zero duration converts to zero hours, so a decay computed over no elapsed time is the identity. |
//! | [`duration_to_hours_one_hour`] | types | A duration converts to hours by its seconds — an hour's worth reads as one — giving the same unit as a difference of timestamps, so both routes feed the same decay arithmetic. |
//! | [`duration_to_hours_clamped`] | types | cites (´claim:types:an-elapsed-measure-beyond-the-decay-horizon-is-reported-as-the-horizon´) |

#![allow(clippy::missing_const_for_fn)]

//! Integration tests for `torrust_assayer::types`.

use std::hash::Hash;
use std::sync::Arc;
use std::time::Duration;

use torrust_assayer::testing::{DEFAULT_TOLERANCES, assert_near, assert_positive};
use torrust_assayer::types::{
    Action, AssessmentId, CellInterval, ChannelId, DimensionId, EntityKey, MAX_DECAY_HOURS, ModelId, OutcomeAxisId,
    PersistentTimestamp, SentinelId, duration_to_hours, dyadic_ancestor_hi, dyadic_ancestor_lo, is_dyadic,
};

// ─────────────────────────────────────────────────────────────────────────────
// Trait bound verification
// ─────────────────────────────────────────────────────────────────────────────

fn assert_id_bounds<T>()
where
    T: Clone + std::fmt::Debug + PartialEq + Eq + Hash + Send + Sync,
{
}

fn assert_copy_id_bounds<T>()
where
    T: Copy + Clone + std::fmt::Debug + PartialEq + Eq + Hash + Send + Sync,
{
}

/// Every identifier the crate hands out can be cloned, compared, hashed and
/// carried across threads, and the numeric ones are Copy besides. These are
/// the bounds that let identifiers be used as map keys and be shared into
/// worker threads without ceremony at each call site.
///
/// ´claim:types:identifiers-can-be-cloned-compared-hashed-and-carried-across-threads´
/// ´test:integration:id-trait-bounds´
#[test]
fn id_trait_bounds() {
    assert_copy_id_bounds::<SentinelId>();
    assert_copy_id_bounds::<DimensionId>();
    assert_copy_id_bounds::<OutcomeAxisId>();
    assert_copy_id_bounds::<ChannelId>();
    assert_id_bounds::<ModelId>();
    assert_id_bounds::<EntityKey>();
}

/// Actions are ordered by severity — allow below challenge, challenge below
/// slow, slow below block — so combining two decisions is a matter of taking
/// the larger. Policy code escalates by comparison rather than by a hand-
/// written table that could disagree with itself.
///
/// ´claim:types:actions-are-ordered-by-severity-so-escalation-is-a-comparison´
/// ´test:integration:action-ordering´
#[test]
fn action_ordering() {
    assert!(Action::Allow < Action::Challenge);
    assert!(Action::Challenge < Action::Slow);
    assert!(Action::Slow < Action::Block);
}

// ─────────────────────────────────────────────────────────────────────────────
// PersistentTimestamp tests
// ─────────────────────────────────────────────────────────────────────────────

/// An elapsed interval between two persistent timestamps is reported in hours:
/// an hour's worth of seconds reads as one. Hours are the unit the decay half-
/// lives are written in, so the conversion belongs to the timestamp rather
/// than to each caller.
///
/// ´claim:types:an-elapsed-interval-between-two-timestamps-is-reported-in-hours´
/// ´test:integration:persistent-timestamp-hours-since-forward´
#[test]
fn persistent_timestamp_hours_since_forward() {
    let earlier = PersistentTimestamp::new(0, 0);
    let later = PersistentTimestamp::new(3600, 0);
    assert_near(later.hours_since(&earlier), 1.0, DEFAULT_TOLERANCES.default, "3600s gap → 1h");
}

/// Asking for the interval from a later moment to an earlier one gives zero,
/// not a negative number. A clock that steps backwards — an upstream
/// correction, a restored checkpoint — cannot un-decay anything or drive a
/// decay factor above one.
///
/// ´claim:types:an-interval-measured-backwards-reports-zero-hours´
/// ´test:integration:persistent-timestamp-hours-since-backward´
#[test]
fn persistent_timestamp_hours_since_backward() {
    let earlier = PersistentTimestamp::new(3600, 0);
    let later = PersistentTimestamp::new(0, 0);
    assert_near(
        later.hours_since(&earlier),
        0.0,
        DEFAULT_TOLERANCES.default,
        "backward gap clamped to 0",
    );
}

/// An elapsed measure longer than the decay horizon is reported as the horizon
/// itself: a two-year gap reads as one year's worth of hours. Past that point
/// the exponential has decayed to nothing anyway, and the cap keeps the
/// arithmetic away from the magnitudes where it stops being meaningful.
///
/// ´claim:types:an-elapsed-measure-beyond-the-decay-horizon-is-reported-as-the-horizon´
/// ´test:integration:persistent-timestamp-hours-since-clamped´
#[test]
fn persistent_timestamp_hours_since_clamped() {
    let earlier = PersistentTimestamp::new(0, 0);
    let later = PersistentTimestamp::new(2 * 365 * 24 * 3600, 0);
    assert_near(
        later.hours_since(&earlier),
        MAX_DECAY_HOURS,
        DEFAULT_TOLERANCES.default,
        "2-year gap clamped to MAX_DECAY_HOURS",
    );
}

// The system-time round trip and the extreme-conversion probe moved to
// the crate suite: `to_system_time` sealed with the fields, being the
// second route to the unbounded interval the clamp withholds
// (´cor:clock:clamp-unbypassable´), so the conversion is no longer part
// of the surface a host reads.

// ─────────────────────────────────────────────────────────────────────────────
// Dyadic interval tests
// ─────────────────────────────────────────────────────────────────────────────

/// A dyadic lower bound keeps the coordinate's leading bits down to the
/// requested depth and zeroes everything below them. Every coordinate sharing
/// that prefix maps to the same bound, which is what makes the bound an
/// address for a whole region rather than for a point.
///
/// ´claim:types:a-dyadic-lower-bound-keeps-the-leading-depth-bits-and-zeroes-the-rest´
/// ´test:integration:dyadic-ancestor-lo-depth-16´
#[test]
fn dyadic_ancestor_lo_depth_16() {
    let coord = 0x1234_5678_9ABC_DEF0_1234_5678_9ABC_DEF0_u128;
    let lo = dyadic_ancestor_lo(coord, 16);
    assert_eq!(lo, 0x1234_0000_0000_0000_0000_0000_0000_0000_u128);
}

/// Depth zero names the whole coordinate space, so its lower bound is zero
/// whichever coordinate was asked about: at the root there is only one region
/// to be in.
///
/// ´claim:types:depth-zero-names-the-whole-space-so-every-coordinates-lower-bound-is-zero´
/// ´test:integration:dyadic-ancestor-lo-depth-zero´
#[test]
fn dyadic_ancestor_lo_depth_zero() {
    let coord = 0x1234_5678_u128;
    assert_eq!(dyadic_ancestor_lo(coord, 0), 0);
}

/// At the deepest level each coordinate is its own region, so the lower bound
/// is the coordinate itself and no bits are discarded.
///
/// ´claim:types:the-deepest-level-names-a-single-point-so-the-lower-bound-is-the-coordinate-itself´
/// ´test:integration:dyadic-ancestor-lo-depth-128´
#[test]
fn dyadic_ancestor_lo_depth_128() {
    let coord = 0x1234_5678_u128;
    assert_eq!(dyadic_ancestor_lo(coord, 128), coord);
}

/// An interval counts as dyadic at a depth when it is aligned to that depth's
/// cell size and spans exactly one such cell: at depth one both halves of the
/// space qualify. Routing structures lean on this so a key cannot claim a
/// depth its extent does not support.
///
/// ´claim:types:an-interval-aligned-to-its-depth-and-spanning-exactly-one-cell-is-dyadic´
/// ´test:integration:is-dyadic-valid´
#[test]
fn is_dyadic_valid() {
    let mid = 1_u128 << 127;
    assert!(is_dyadic(0, mid - 1, 1));
    assert!(is_dyadic(mid, u128::MAX, 1));
}

/// At depth zero only the entire coordinate space qualifies; an interval one
/// short of the maximum does not, however nearly it covers everything. The
/// root is a specific interval, not a description of a large one.
///
/// ´claim:types:only-the-entire-space-is-dyadic-at-depth-zero´
/// ´test:integration:is-dyadic-depth-zero´
#[test]
fn is_dyadic_depth_zero() {
    assert!(is_dyadic(0, u128::MAX, 0));
    assert!(!is_dyadic(0, u128::MAX - 1, 0));
}

/// At the deepest level only a degenerate interval qualifies — one whose
/// bounds coincide. A two-point span claiming that depth is rejected.
///
/// ´claim:types:only-a-single-point-interval-is-dyadic-at-the-deepest-level´
/// ´test:integration:is-dyadic-depth-128´
#[test]
fn is_dyadic_depth_128() {
    assert!(is_dyadic(42, 42, 128));
    assert!(!is_dyadic(42, 43, 128));
}

/// An interval whose lower bound is not a multiple of its depth's cell size is
/// rejected even when its extent looks plausible. Alignment is what makes the
/// hierarchy nest, so a shifted interval is not merely unusual but
/// unaddressable.
///
/// ´claim:types:an-interval-whose-lower-bound-is-misaligned-for-its-depth-is-not-dyadic´
/// ´test:integration:is-dyadic-rejects-misaligned´
#[test]
fn is_dyadic_rejects_misaligned() {
    assert!(!is_dyadic(1, 1_u128 << 127, 1));
}

/// An interval correctly aligned but two short of its depth's cell width is
/// rejected too. Both the start and the extent have to match the claimed
/// depth; getting one of them right is not enough.
///
/// ´claim:types:an-aligned-interval-of-the-wrong-width-for-its-depth-is-still-not-dyadic´
/// ´test:integration:is-dyadic-rejects-wrong-width´
#[test]
fn is_dyadic_rejects_wrong_width() {
    assert!(!is_dyadic(0, (1_u128 << 127) - 2, 1));
}

// ─────────────────────────────────────────────────────────────────────────────
// EntityKey tests
// ─────────────────────────────────────────────────────────────────────────────

/// An entity key preserves exactly the bytes it was built from, so a host's
/// identifier reaches the cache unchanged and the same identifier always keys
/// the same record.
///
/// ´claim:types:an-entity-key-preserves-the-bytes-it-was-built-from´
/// ´test:integration:entity-key-from-vec´
#[test]
fn entity_key_from_vec() {
    let key = EntityKey::from(vec![1, 2, 3, 4]);
    assert_eq!(key.as_bytes(), &[1, 2, 3, 4]);
}

/// Building from a borrowed slice gives the same key as building from an owned
/// vector: the conversion copies the bytes rather than reinterpreting them, so
/// a caller need not own its identifier to look one up.
///
/// (´claim:types:an-entity-key-preserves-the-bytes-it-was-built-from´)
/// ´test:integration:entity-key-from-slice´
#[test]
fn entity_key_from_slice() {
    let key = EntityKey::from([5, 6, 7].as_slice());
    assert_eq!(key.as_bytes(), &[5, 6, 7]);
}

/// Cloning an entity key shares the underlying bytes instead of copying them —
/// after a clone both handles report two references to one allocation. Keys
/// are cloned on every cache write, so sharing is what keeps that path from
/// copying an identifier per request.
///
/// ´claim:types:cloning-an-entity-key-shares-the-bytes-rather-than-copying-them´
/// ´test:integration:entity-key-clone-cheap´
#[test]
fn entity_key_clone_cheap() {
    let key = EntityKey::from(vec![1, 2, 3]);
    assert_eq!(Arc::strong_count(&key.0), 1);
    let cloned = key.clone();
    assert_eq!(Arc::strong_count(&key.0), 2);
    assert_eq!(Arc::strong_count(&cloned.0), 2);
}

// ─────────────────────────────────────────────────────────────────────────────
// Additional Newtype ID tests
// ─────────────────────────────────────────────────────────────────────────────

/// An identifier is a Copy type: passing it on leaves the original usable and
/// the copy equal to it, so identifiers can be scattered through call sites
/// without borrow plumbing.
///
/// ´claim:types:an-identifier-is-a-copy-type-so-passing-it-on-leaves-the-original-usable´
/// ´test:integration:sentinel-id-copy-clone´
#[test]
fn sentinel_id_copy_clone() {
    let id = SentinelId(42);
    let copied = id;
    #[allow(clippy::clone_on_copy)]
    let cloned = id.clone();
    assert_eq!(id, copied);
    assert_eq!(id, cloned);
}

/// Equal identifiers hash alike, so a set holds one entry per identity rather
/// than one per insertion. A registry keyed by identifier therefore counts
/// identities, not the times they were mentioned.
///
/// ´claim:types:equal-identifiers-hash-alike-so-a-set-holds-one-entry-per-identity´
/// ´test:integration:sentinel-id-hash-eq´
#[test]
fn sentinel_id_hash_eq() {
    use std::collections::HashSet;
    let mut set = HashSet::new();
    set.insert(SentinelId(42));
    set.insert(SentinelId(42));
    assert_eq!(set.len(), 1, "Two equal SentinelIds should hash to the same bucket");
}

/// Every model identity renders differently in diagnostics: the operational,
/// sister and anchor models and a per-axis one are all distinguishable in a
/// log line, so a report naming a model is unambiguous about which it means.
///
/// ´claim:types:every-model-identity-renders-distinguishably-in-diagnostics´
/// ´test:integration:model-id-variants-debug´
#[test]
fn model_id_variants_debug() {
    let operational = format!("{:?}", ModelId::Operational);
    let sister = format!("{:?}", ModelId::Sister);
    let anchor = format!("{:?}", ModelId::Anchor);
    let axis = format!("{:?}", ModelId::OutcomeAxis(OutcomeAxisId(1)));

    assert_ne!(operational, sister);
    assert_ne!(operational, anchor);
    assert_ne!(operational, axis);
    assert_ne!(sister, anchor);
    assert_ne!(sister, axis);
    assert_ne!(anchor, axis);
}

/// Model identities differing only in which outcome axis they name hash apart,
/// so per-axis models keyed in one map do not collapse into a single entry.
///
/// ´claim:types:model-identities-differing-only-in-their-axis-hash-apart´
/// ´test:integration:model-id-axis-hash´
#[test]
fn model_id_axis_hash() {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::Hasher;

    fn compute_hash<T: Hash>(value: &T) -> u64 {
        let mut hasher = DefaultHasher::new();
        value.hash(&mut hasher);
        hasher.finish()
    }

    let axis1 = ModelId::OutcomeAxis(OutcomeAxisId(1));
    let axis2 = ModelId::OutcomeAxis(OutcomeAxisId(2));

    assert_ne!(compute_hash(&axis1), compute_hash(&axis2));
}

/// An assessment identifier carries the full range of a sixty-four-bit counter
/// and converts back to it exactly at both ends. The counter is never expected
/// to wrap, so nothing narrows it on the way out.
///
/// ´claim:types:an-assessment-identifier-carries-the-full-range-of-a-sixty-four-bit-counter´
/// ´test:integration:assessment-id-is-u64´
#[test]
fn assessment_id_is_u64() {
    let min = AssessmentId(0);
    let max = AssessmentId(u64::MAX);
    assert_eq!(u64::from(min), 0_u64);
    assert_eq!(u64::from(max), u64::MAX);
}

// ─────────────────────────────────────────────────────────────────────────────
// Additional PersistentTimestamp tests
// ─────────────────────────────────────────────────────────────────────────────

/// The current time is a moment after the epoch: the clamped elapsed
/// reader — the only interval a host can take — reports a positive gap
/// from the epoch to now. With the fields sealed, positivity of the
/// reading is the whole of what the surface promises a host about
/// `now()`; the field-level invariants are pinned in the crate suite,
/// where the fields are reachable.
///
/// ´claim:types:the-current-time-reads-as-a-positive-elapsed-gap-from-the-epoch´
/// ´test:integration:timestamp-now-positive´
#[test]
fn timestamp_now_positive() {
    let epoch = PersistentTimestamp::new(0, 0);
    let ts = PersistentTimestamp::now();
    assert_positive(ts.hours_since(&epoch), "hours from the epoch to now()");
}

// The release-profile nanosecond-normalisation pin moved to the crate
// suite with the sealed fields: the surplus carrying into the seconds is
// a field-level fact no public reader exposes.

/// In debug builds that same overflowing nanosecond count trips the invariant
/// at once, so the bug is found where it is written rather than surviving into
/// a checkpoint that some later reader has to forgive.
///
/// ´claim:types:debug-builds-trip-the-invariant-on-nanoseconds-that-reach-a-full-second´
/// ´test:integration:timestamp-nanos-invariant-panics´
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "nanos must be < 1_000_000_000")]
fn timestamp_nanos_invariant_panics() {
    let _ = PersistentTimestamp::new(100, 1_500_000_000);
}

/// A timestamp is zero hours from itself, so a decay applied at the very
/// moment of the last update leaves the value it decays untouched.
///
/// ´claim:types:a-timestamp-is-zero-hours-from-itself´
/// ´test:integration:hours-since-zero-gap´
#[test]
fn hours_since_zero_gap() {
    let t = PersistentTimestamp::new(12345, 0);
    assert_near(t.hours_since(&t), 0.0, DEFAULT_TOLERANCES.default, "hours_since self");
}

/// The unit is genuinely hours and not a rounded count of them: half an hour's
/// worth of seconds reads as a half.
///
/// (´claim:types:an-elapsed-interval-between-two-timestamps-is-reported-in-hours´)
/// ´test:integration:hours-since-fractional´
#[test]
fn hours_since_fractional() {
    let earlier = PersistentTimestamp::new(0, 0);
    let later = PersistentTimestamp::new(1800, 0);
    assert_near(
        later.hours_since(&earlier),
        0.5,
        DEFAULT_TOLERANCES.default,
        "30 minutes → 0.5h",
    );
}

/// A gap of exactly one year lands on the decay horizon itself, so the horizon
/// is a value the arithmetic reaches and not only one the cap imposes. The two
/// agree at the boundary rather than meeting at a step.
///
/// ´claim:types:a-gap-of-exactly-one-year-lands-on-the-decay-horizon-itself´
/// ´test:integration:hours-since-exactly-max´
#[test]
fn hours_since_exactly_max() {
    let earlier = PersistentTimestamp::new(0, 0);
    let later = PersistentTimestamp::new(365 * 24 * 3600, 0);
    assert_near(
        later.hours_since(&earlier),
        MAX_DECAY_HOURS,
        DEFAULT_TOLERANCES.default,
        "1 year → MAX_DECAY_HOURS",
    );
}

/// A gap short of a year is reported in full: three hundred and sixty-four
/// days read as their own hours, below the horizon. The cap bites only beyond
/// the horizon and does not flatten the long intervals just short of it.
///
/// ´claim:types:an-interval-short-of-the-horizon-is-reported-in-full´
/// ´test:integration:hours-since-just-under-max´
#[test]
fn hours_since_just_under_max() {
    let earlier = PersistentTimestamp::new(0, 0);
    let later = PersistentTimestamp::new(364 * 24 * 3600, 0);
    let hours = later.hours_since(&earlier);
    assert!(hours < MAX_DECAY_HOURS, "364 days should be < MAX_DECAY_HOURS");
    assert_near(hours, 364.0 * 24.0, DEFAULT_TOLERANCES.default, "364 days → 8736h");
}

/// A single nanosecond registers as a positive interval, vanishingly small but
/// not zero. The measurement is not truncated to whole seconds on its way to
/// hours, so events within one second are still separated by their gap.
///
/// ´claim:types:a-single-nanosecond-registers-as-a-positive-but-vanishing-interval´
/// ´test:integration:hours-since-nanos-precision´
#[test]
fn hours_since_nanos_precision() {
    let earlier = PersistentTimestamp::new(0, 0);
    let later = PersistentTimestamp::new(0, 1);
    let hours = later.hours_since(&earlier);
    assert_positive(hours, "1 ns gap");
    assert!(hours < 1e-10, "1 nanosecond gap should be tiny, got {hours}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Additional dyadic interval tests
// ─────────────────────────────────────────────────────────────────────────────

/// Depth four keeps a single leading hexadecimal digit of the coordinate. What
/// is retained is fixed by the depth alone, not by where the coordinate's own
/// structure happens to fall.
///
/// (´claim:types:a-dyadic-lower-bound-keeps-the-leading-depth-bits-and-zeroes-the-rest´)
/// ´test:integration:dyadic-ancestor-lo-depth-4´
#[test]
fn dyadic_ancestor_lo_depth_4() {
    let coord = 0xABCD_EF01_2345_6789_ABCD_EF01_2345_6789_u128;
    let lo = dyadic_ancestor_lo(coord, 4);
    assert_eq!(lo, 0xA000_0000_0000_0000_0000_0000_0000_0000_u128);
}

/// Depth eight keeps two leading digits, including when those bits are all
/// set: a coordinate high in the space is truncated exactly as one low in it
/// is.
///
/// (´claim:types:a-dyadic-lower-bound-keeps-the-leading-depth-bits-and-zeroes-the-rest´)
/// ´test:integration:dyadic-ancestor-lo-depth-8´
#[test]
fn dyadic_ancestor_lo_depth_8() {
    let coord = 0xFF12_3456_7890_ABCD_EF12_3456_7890_ABCD_u128;
    let lo = dyadic_ancestor_lo(coord, 8);
    assert_eq!(lo, 0xFF00_0000_0000_0000_0000_0000_0000_0000_u128);
}

/// A cell interval renders both its bounds and its depth in diagnostics, so an
/// interval printed into a log can be read back as the region it describes
/// rather than as an opaque handle.
///
/// ´claim:types:a-cell-interval-renders-its-bounds-and-depth-in-diagnostics´
/// ´test:integration:cell-interval-display´
#[test]
fn cell_interval_display() {
    let cell = CellInterval::new(0, 255, 120);
    let debug = format!("{cell:?}");
    assert!(debug.contains("CellInterval"));
    assert!(debug.contains("lo:"));
    assert!(debug.contains("hi:"));
    assert!(debug.contains("depth:"));
}

/// A cell interval's width counts its upper bound as included: a span from
/// zero to two hundred and fifty-five is two hundred and fifty-six wide, and
/// an interval whose bounds coincide is one wide rather than zero. The
/// convention deliberately departs from half-open notation, and the arithmetic
/// saturates rather than overflowing when the interval covers the entire
/// space.
///
/// ´claim:types:a-cell-intervals-width-counts-its-upper-bound-as-included´
/// ´test:integration:cell-interval-width´
#[test]
fn cell_interval_width() {
    let cell = CellInterval::new(0, 255, 120);
    assert_eq!(cell.width(), 256);

    let point = CellInterval::new(42, 42, 128);
    assert_eq!(point.width(), 1);

    let full = CellInterval::new(0, u128::MAX, 0);
    assert_eq!(full.width(), u128::MAX);
}

// ─────────────────────────────────────────────────────────────────────────────
// dyadic_ancestor_hi tests
// ─────────────────────────────────────────────────────────────────────────────

/// Depth zero's upper bound is the largest representable coordinate, so the
/// root region genuinely spans everything rather than merely most of it.
///
/// ´claim:types:depth-zero-reaches-up-to-the-largest-coordinate´
/// ´test:integration:dyadic-ancestor-hi-depth-zero´
#[test]
fn dyadic_ancestor_hi_depth_zero() {
    let coord = 0x1234_5678_u128;
    assert_eq!(dyadic_ancestor_hi(coord, 0), u128::MAX);
}

/// At the deepest level the upper bound coincides with the lower one: a single
/// point has no extent to report.
///
/// ´claim:types:the-deepest-level-has-an-upper-bound-equal-to-the-coordinate-itself´
/// ´test:integration:dyadic-ancestor-hi-depth-128´
#[test]
fn dyadic_ancestor_hi_depth_128() {
    let coord = 0x1234_5678_u128;
    assert_eq!(dyadic_ancestor_hi(coord, 128), coord);
}

/// The two halves at depth one abut exactly: the lower half ends one below the
/// midpoint where the upper half begins, and the upper half ends at the
/// largest coordinate. No coordinate falls between them and none falls outside
/// them.
///
/// ´claim:types:the-two-halves-at-depth-one-abut-with-no-gap-and-no-overlap´
/// ´test:integration:dyadic-ancestor-hi-depth-1´
#[test]
fn dyadic_ancestor_hi_depth_1() {
    let lo_coord = 0_u128;
    let hi_coord = 1_u128 << 127;

    assert_eq!(dyadic_ancestor_hi(lo_coord, 1), (1_u128 << 127) - 1);
    assert_eq!(dyadic_ancestor_hi(hi_coord, 1), u128::MAX);
}

// ─────────────────────────────────────────────────────────────────────────────
// Duration utility tests
// ─────────────────────────────────────────────────────────────────────────────

/// A zero duration converts to zero hours, so a decay computed over no elapsed
/// time is the identity.
///
/// ´claim:types:a-zero-duration-converts-to-zero-hours´
/// ´test:integration:duration-to-hours-zero´
#[test]
fn duration_to_hours_zero() {
    assert_near(
        duration_to_hours(Duration::ZERO),
        0.0,
        DEFAULT_TOLERANCES.default,
        "zero duration",
    );
}

/// A duration converts to hours by its seconds — an hour's worth reads as one
/// — giving the same unit as a difference of timestamps, so both routes feed
/// the same decay arithmetic.
///
/// ´claim:types:a-duration-converts-to-hours-by-its-seconds´
/// ´test:integration:duration-to-hours-one-hour´
#[test]
fn duration_to_hours_one_hour() {
    let one_hour = Duration::from_secs(3600);
    assert_near(duration_to_hours(one_hour), 1.0, DEFAULT_TOLERANCES.default, "3600s → 1h");
}

/// The horizon caps a measured duration exactly as it caps a gap between two
/// timestamps: two years reads as the horizon. Both ways into the decay
/// arithmetic are bounded, so neither can be used to reach magnitudes the
/// other refuses.
///
/// (´claim:types:an-elapsed-measure-beyond-the-decay-horizon-is-reported-as-the-horizon´)
/// ´test:integration:duration-to-hours-clamped´
#[test]
fn duration_to_hours_clamped() {
    let two_years = Duration::from_secs(2 * 365 * 24 * 3600);
    assert_near(
        duration_to_hours(two_years),
        MAX_DECAY_HOURS,
        DEFAULT_TOLERANCES.default,
        "2 years clamped",
    );
}
