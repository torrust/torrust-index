// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`default_is_not_degraded`] | scenario | An assessment is clean until something during it actually goes wrong: the freshly-constructed context and the builder's untouched defaults both report no degradation. Degradation has to be earned by an event, so a host reading the flag on an ordinary assessment is not told to distrust a result that nothing was ever wrong with. |
//! | [`degraded_when_signals_sanitised`] | scenario | A single sanitised input signal at the first checkpoint is enough to mark the whole assessment degraded, with every other channel still clean. The flag is a disjunction rather than a threshold, so no amount of surrounding good health can dilute one substituted value into looking trustworthy. |
//! | [`degraded_when_signals_shape_mismatched`] | scenario | cites (´claim:scenario:any-single-degradation-channel-alone-marks-the-assessment-degraded´) |
//! | [`degraded_when_nan_sentinels`] | scenario | cites (´claim:scenario:any-single-degradation-channel-alone-marks-the-assessment-degraded´) |
//! | [`degraded_when_degraded_report_sentinels`] | scenario | cites (´claim:scenario:any-single-degradation-channel-alone-marks-the-assessment-degraded´) |
//! | [`degraded_when_features_sanitised`] | scenario | cites (´claim:scenario:any-single-degradation-channel-alone-marks-the-assessment-degraded´) |
//! | [`degraded_when_degraded_models`] | scenario | cites (´claim:scenario:any-single-degradation-channel-alone-marks-the-assessment-degraded´) |
//! | [`all_fields_degraded`] | scenario | Degradation channels compose rather than cancel: a context carrying sanitised signals, a shape mismatch, both kinds of sentinel fault, sanitised features and a fallen-back model reports degraded, exactly as each would alone. Nothing about an assessment going wrong in many ways at once can talk the flag back down to clean. |

//! Integration tests for `torrust_assayer::health::degradation`.
//!
//! Uses the [`DegradationSpec`] fluent builder from
//! [`torrust_assayer::testing`] to avoid restating the five clean
//! defaults on every test — each test flips just the field it cares
//! about.

use torrust_assayer::DegradationContext;
use torrust_assayer::testing::DegradationSpec;
use torrust_assayer::types::{ModelId, OutcomeAxisId};

/// An assessment is clean until something during it actually goes wrong: the
/// freshly-constructed context and the builder's untouched defaults both
/// report no degradation. Degradation has to be earned by an event, so a host
/// reading the flag on an ordinary assessment is not told to distrust a result
/// that nothing was ever wrong with.
///
/// ´claim:scenario:an-assessment-is-clean-until-a-degradation-event-is-recorded´
/// ´test:integration:default-is-not-degraded´
#[test]
fn default_is_not_degraded() {
    assert!(!DegradationContext::default().is_degraded());
    assert!(!DegradationSpec::new().build().is_degraded());
}

/// A single sanitised input signal at the first checkpoint is enough to mark
/// the whole assessment degraded, with every other channel still clean. The
/// flag is a disjunction rather than a threshold, so no amount of surrounding
/// good health can dilute one substituted value into looking trustworthy.
///
/// ´claim:scenario:any-single-degradation-channel-alone-marks-the-assessment-degraded´
/// ´test:integration:degraded-when-signals-sanitised´
#[test]
fn degraded_when_signals_sanitised() {
    assert!(DegradationSpec::new().signals_sanitised(1).build().is_degraded());
}

/// A signal whose value did not match its declared shape is zero-filled during
/// encoding, and that substitution is degradation in its own right — the
/// assessment is marked even though the count of NaN-sanitised signals stayed
/// at zero. A caller sending a categorical value where a scalar was declared
/// therefore cannot have the mistake pass silently.
///
/// (´claim:scenario:any-single-degradation-channel-alone-marks-the-assessment-degraded´)
/// ´test:integration:degraded-when-signals-shape-mismatched´
#[test]
fn degraded_when_signals_shape_mismatched() {
    assert!(DegradationSpec::new().signals_shape_mismatched(1).build().is_degraded());
}

/// Recording even one sentinel whose extraction had to be zeroed degrades the
/// assessment. The channel is a list rather than a counter, so the identity of
/// the failing sentinel survives to the report, and a non-empty list is
/// sufficient on its own regardless of which sentinel it names.
///
/// (´claim:scenario:any-single-degradation-channel-alone-marks-the-assessment-degraded´)
/// ´test:integration:degraded-when-nan-sentinels´
#[test]
fn degraded_when_nan_sentinels() {
    assert!(DegradationSpec::new().nan_sentinels(&[0]).build().is_degraded());
}

/// A sentinel whose cached batch report carried per-cell data-quality problems
/// degrades the assessment separately from one whose extraction went NaN. The
/// two lists are distinct channels, so a host can tell a broken extraction
/// apart from a usable-but-suspect cached report rather than seeing one
/// undifferentiated fault.
///
/// (´claim:scenario:any-single-degradation-channel-alone-marks-the-assessment-degraded´)
/// ´test:integration:degraded-when-degraded-report-sentinels´
#[test]
fn degraded_when_degraded_report_sentinels() {
    assert!(DegradationSpec::new().degraded_report_sentinels(&[1]).build().is_degraded());
}

/// Damage introduced downstream of the inputs still counts: features
/// sanitised after standardisation degrade the assessment even though every
/// signal arrived intact. Standardisation can manufacture a non-finite value
/// out of finite inputs, so the checkpoint after it needs its own channel or
/// that failure would be invisible.
///
/// (´claim:scenario:any-single-degradation-channel-alone-marks-the-assessment-degraded´)
/// ´test:integration:degraded-when-features-sanitised´
#[test]
fn degraded_when_features_sanitised() {
    assert!(DegradationSpec::new().features_sanitised(3).build().is_degraded());
}

/// A model that fell back to its prior after producing a non-finite estimate
/// degrades the assessment. The fallback keeps a number flowing so the
/// pipeline can finish, and this channel is what stops that graceful
/// substitution from being mistaken for a genuine model opinion.
///
/// (´claim:scenario:any-single-degradation-channel-alone-marks-the-assessment-degraded´)
/// ´test:integration:degraded-when-degraded-models´
#[test]
fn degraded_when_degraded_models() {
    assert!(
        DegradationSpec::new()
            .degraded_models([ModelId::Operational])
            .build()
            .is_degraded()
    );
}

/// Degradation channels compose rather than cancel: a context carrying
/// sanitised signals, a shape mismatch, both kinds of sentinel fault,
/// sanitised features and a fallen-back model reports degraded, exactly as
/// each would alone. Nothing about an assessment going wrong in many ways at
/// once can talk the flag back down to clean.
///
/// ´claim:scenario:degradation-channels-compose-rather-than-cancel-each-other´
/// ´test:integration:all-fields-degraded´
#[test]
fn all_fields_degraded() {
    let ctx = DegradationSpec::new()
        .signals_sanitised(2)
        .signals_shape_mismatched(1)
        .nan_sentinels(&[0])
        .degraded_report_sentinels(&[1])
        .features_sanitised(5)
        .degraded_models([ModelId::OutcomeAxis(OutcomeAxisId(0))])
        .build();
    assert!(ctx.is_degraded());
}
