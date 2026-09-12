// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`parameters_round_trip_serde`] | bayes | A parameter set survives a trip through the serialised form intact: the dimension, the mean, and the flattened covariance all come back bit for bit, with every value still finite on the way out as on the way in. The serialised form is how a posterior crosses a restart, so anything the encoding rounds off is learning permanently lost. |
//! | [`parameters_from_zero_dim`] | bayes | The empty parameter set is a coherent zero-dimensional description rather than a placeholder to be avoided: no coordinates, no mean entries, no covariance entries. A model that has not yet been given any features can still be represented and stored, so callers need no separate absent case. |

//! Crate-level tests for `model::parameters::ModelParameters`.

use crate::model::parameters::ModelParameters;
use crate::testing::assert_finite;

/// A parameter set survives a trip through the serialised form intact: the
/// dimension, the mean, and the flattened covariance all come back bit for
/// bit, with every value still finite on the way out as on the way in. The
/// serialised form is how a posterior crosses a restart, so anything the
/// encoding rounds off is learning permanently lost.
///
/// ´claim:bayes:serialising-and-deserialising-a-parameter-set-preserves-every-value-exactly´
/// ´test:crate:parameters-round-trip-serde´
#[test]
#[cfg(feature = "serde")]
fn parameters_round_trip_serde() {
    let params = ModelParameters {
        mu: vec![1.0, 2.0, 3.0],
        covariance_data: vec![1.0, 0.0, 0.0, 0.0, 2.0, 0.0, 0.0, 0.0, 3.0],
        p: 3,
    };

    // Sanity-check the fixture with the shared finite-values assertion
    // before we round-trip it, so any NaN/Inf slip-up fails with the
    // same diagnostic format as the rest of the test suite.
    assert_finite(&params.mu, "ModelParameters::mu (fixture)");
    assert_finite(&params.covariance_data, "ModelParameters::covariance_data (fixture)");

    let json = serde_json::to_string(&params).unwrap();
    let restored: ModelParameters = serde_json::from_str(&json).unwrap();

    assert_eq!(restored.p, params.p);
    assert_eq!(restored.mu, params.mu);
    assert_eq!(restored.covariance_data, params.covariance_data);

    assert_finite(&restored.mu, "ModelParameters::mu (restored)");
    assert_finite(&restored.covariance_data, "ModelParameters::covariance_data (restored)");
}

/// The empty parameter set is a coherent zero-dimensional description rather
/// than a placeholder to be avoided: no coordinates, no mean entries, no
/// covariance entries. A model that has not yet been given any features can
/// still be represented and stored, so callers need no separate absent case.
///
/// ´claim:bayes:the-empty-parameter-set-describes-a-zero-dimensional-model-with-no-entries´
/// ´test:crate:parameters-from-zero-dim´
#[test]
fn parameters_from_zero_dim() {
    let params = ModelParameters::empty();
    assert_eq!(params.p, 0);
    assert_eq!(params.mu, [] as [f64; 0]);
    assert_eq!(params.covariance_data, [] as [f64; 0]);

    // Vacuously finite, but exercising the helper keeps the invariant
    // wired in for future non-empty cases.
    assert_finite(&params.mu, "ModelParameters::empty().mu");
    assert_finite(&params.covariance_data, "ModelParameters::empty().covariance_data");
}
