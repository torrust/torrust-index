// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Signal-schema fixture builders.
//!
//! Small constructors for [`SignalDeclaration`] payloads used by
//! signal-schema tests. Centralised here so the same shapes do not
//! drift across `src/tests/` files.

use crate::assessment::RequestContext;
use crate::signal::{Persistence, SignalDeclaration, SignalShape};

/// `SignalDeclaration` with [`SignalShape::Scalar`] clipped to `0.0..=1.0`.
#[must_use]
pub fn scalar_decl(name: &str, persistence: Persistence) -> SignalDeclaration {
    SignalDeclaration::new(name, SignalShape::Scalar { clip: (0.0, 1.0) }, persistence)
}

/// `SignalDeclaration` with [`SignalShape::Binary`].
#[must_use]
pub fn binary_decl(name: &str, persistence: Persistence) -> SignalDeclaration {
    SignalDeclaration::new(name, SignalShape::Binary, persistence)
}

/// Standard request-scoped signal schema used by the signal-rich probes of the
/// core/decision seam (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
///
/// The pair is deliberately tiny: one clipped scalar and one binary feature.
/// It is enough to exercise request-signal assembly without making every test
/// declare ad-hoc schema boilerplate.
#[must_use]
pub fn score_verified_request_schema() -> Vec<SignalDeclaration> {
    vec![
        scalar_decl("score", Persistence::Request),
        binary_decl("verified", Persistence::Request),
    ]
}

/// Attach the standard score/verified request-scoped signal payload.
#[must_use]
pub fn with_score_verified(request: RequestContext, score: f64, verified: bool) -> RequestContext {
    let verified_value = if verified { 1.0 } else { 0.0 };
    request.with_signal("score", score).with_signal("verified", verified_value)
}

/// Attach the standard score/verified training payload for cycle-based probes.
///
/// Rich seam fixtures
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´)
/// use this when they need a non-constant request-signal stream that is still
/// identical across matched channels or worlds.
#[must_use]
pub fn with_standard_training_score_verified(request: RequestContext, cycle: usize) -> RequestContext {
    with_score_verified(
        request,
        if cycle.is_multiple_of(2) { 0.84 } else { 0.26 },
        cycle.is_multiple_of(3),
    )
}

/// Attach the standard final score/verified payload for the rich seam probes
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
#[must_use]
pub fn with_standard_final_score_verified(request: RequestContext) -> RequestContext {
    with_score_verified(request, 0.68, true)
}

/// Attach a deliberately degraded score/verified payload.
///
/// The schema from [`score_verified_request_schema`] declares `score` as a
/// numeric scalar and `verified` as a binary numeric signal. This payload
/// therefore exercises both public signal-degradation counters: one
/// non-finite numeric value and one shape mismatch. Tests use it to prove
/// that channel policy can change only decision-layer resonance, not the
/// sanitised core feature path or its inline health snapshot.
#[must_use]
pub fn with_degraded_score_verified(request: RequestContext) -> RequestContext {
    request
        .with_signal("score", f64::NAN)
        .with_signal("verified", "shape-mismatch")
}
