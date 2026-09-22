// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`finds_existing_signal`] | signal | Looking a signal up by name yields its offset and its width, and consecutive one-wide signals sit at consecutive offsets. This is the lookup the encoding path uses to write a value straight into the right column without building a whole vector first. |
//! | [`returns_none_for_unknown`] | signal | A name the schema does not declare yields nothing at all rather than a plausible-looking offset, so callers can tell an undeclared signal apart from one declared at position zero. |
//! | [`schema_refuses_malformed_shape_parameters`] | signal | Every shape parameter the encoding cannot honour is refused at construction, naming the declaring signal: a clipping interval with reversed or non-finite bounds, a divisor, maximum or period of zero, and a hashed or vector width of zero. Each of these previously passed the constructor and surfaced on the assessment path — the reversed interval as a panic in the clamping helper, the zero maximum as a quotient that is not a number — on the one call the corpus promises is infallible. |
//! | [`schema_admits_well_formed_boundary_shapes`] | signal | cites (´claim:signal:a-shape-parameter-the-encoding-cannot-honour-is-refused-at-construction´) |
//! | [`ordinal_arm_sanitises_nan_quotient`] | signal | The ordinal arm sanitises its quotient as its five siblings do: a zero value against a declared maximum whose quotient is not a number encodes to zero rather than passing NaN through the clamp into the signal block, the standardisation pass and every interaction formed over the position. The malformed maximum itself is refused at construction; this pins the belt the module's own contract claims for values that reach the encoder by other routes. |
//! | [`builder_refuses_malformed_signal_declaration`] | signal | cites (´claim:signal:a-shape-parameter-the-encoding-cannot-honour-is-refused-at-construction´) |

//! Crate-level tests for `signal::SignalSchemaIndex` internal API.
//!
//! These tests cover `pub(crate)` methods not exposed in the public API.
//! Declaration fixtures come from [`crate::testing::signals`].

use crate::signal::{Persistence, SignalSchemaIndex};
use crate::testing::signals::scalar_decl;

mod get {
    use super::*;

    /// Looking a signal up by name yields its offset and its width, and
    /// consecutive one-wide signals sit at consecutive offsets. This is the
    /// lookup the encoding path uses to write a value straight into the right
    /// column without building a whole vector first.
    ///
    /// ´claim:signal:a-lookup-by-name-yields-the-signals-offset-and-width´
    /// ´test:crate:finds-existing-signal´
    #[test]
    fn finds_existing_signal() {
        let decls = vec![scalar_decl("a", Persistence::Entity), scalar_decl("b", Persistence::Entity)];
        let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();

        assert_eq!(schema.get("a"), Some((0, 1)));
        assert_eq!(schema.get("b"), Some((1, 1)));
    }

    /// A name the schema does not declare yields nothing at all rather than a
    /// plausible-looking offset, so callers can tell an undeclared signal
    /// apart from one declared at position zero.
    ///
    /// ´claim:signal:a-lookup-for-an-undeclared-name-yields-nothing´
    /// ´test:crate:returns-none-for-unknown´
    #[test]
    fn returns_none_for_unknown() {
        let decls = vec![scalar_decl("a", Persistence::Entity)];
        let schema = SignalSchemaIndex::from_declarations(&decls).unwrap();
        assert_eq!(schema.get("unknown"), None);
    }

    /// Every shape parameter the encoding cannot honour is refused at
    /// construction, naming the declaring signal: a clipping interval with
    /// reversed or non-finite bounds, a divisor, maximum or period of zero,
    /// and a hashed or vector width of zero. Each of these previously
    /// passed the constructor and surfaced on the assessment path — the
    /// reversed interval as a panic in the clamping helper, the zero
    /// maximum as a quotient that is not a number — on the one call the
    /// corpus promises is infallible.
    ///
    /// ´claim:signal:a-shape-parameter-the-encoding-cannot-honour-is-refused-at-construction´
    /// ´test:crate:schema-refuses-malformed-shape-parameters´
    #[test]
    fn schema_refuses_malformed_shape_parameters() {
        use crate::error::BuildError;
        use crate::signal::{SignalDeclaration, SignalShape};

        let refused = |shape: SignalShape| {
            let decl = SignalDeclaration::new("s", shape, Persistence::Request);
            matches!(
                SignalSchemaIndex::from_declarations(&[decl]),
                Err(BuildError::InvalidSignalShape { .. })
            )
        };

        // Reversed and non-finite clipping intervals, scalar and vector.
        assert!(refused(SignalShape::Scalar { clip: (1.0, 0.0) }));
        assert!(refused(SignalShape::Scalar { clip: (f64::NAN, 1.0) }));
        assert!(refused(SignalShape::Scalar {
            clip: (0.0, f64::INFINITY)
        }));
        assert!(refused(SignalShape::Vector {
            len: 3,
            clip: (2.0, -2.0)
        }));

        // Zero divisor, maximum and period.
        assert!(refused(SignalShape::LogScaled { divisor: 0.0 }));
        assert!(refused(SignalShape::Ordinal { max: 0.0 }));
        assert!(refused(SignalShape::Cyclic { period: 0.0 }));

        // Zero hashed and vector widths.
        assert!(refused(SignalShape::HashedCategorical { width: 0 }));
        assert!(refused(SignalShape::Vector {
            len: 0,
            clip: (0.0, 1.0)
        }));
    }

    /// The refusal admits what it should: well-formed parameters build,
    /// including the boundary cases the constraint does not name — an
    /// equal-bounds interval (a constant clamp is ordered), a negative
    /// divisor and a negative maximum (non-zero, so the quotient is a
    /// number). The check refuses the malformations the assessment path
    /// cannot survive and nothing beside them.
    ///
    /// (´claim:signal:a-shape-parameter-the-encoding-cannot-honour-is-refused-at-construction´)
    /// ´test:crate:schema-admits-well-formed-boundary-shapes´
    #[test]
    fn schema_admits_well_formed_boundary_shapes() {
        use crate::signal::{SignalDeclaration, SignalShape};

        let admitted = |shape: SignalShape| {
            let decl = SignalDeclaration::new("s", shape, Persistence::Request);
            SignalSchemaIndex::from_declarations(&[decl]).is_ok()
        };

        assert!(admitted(SignalShape::Scalar { clip: (0.5, 0.5) }));
        assert!(admitted(SignalShape::LogScaled { divisor: -2.0 }));
        assert!(admitted(SignalShape::Ordinal { max: -4.0 }));
        assert!(admitted(SignalShape::Cyclic { period: -24.0 }));
        assert!(admitted(SignalShape::HashedCategorical { width: 1 }));
        assert!(admitted(SignalShape::Vector {
            len: 1,
            clip: (-1.0, 1.0)
        }));
    }

    /// The ordinal arm sanitises its quotient as its five siblings do: a
    /// zero value against a declared maximum whose quotient is not a number
    /// encodes to zero rather than passing NaN through the clamp into the
    /// signal block, the standardisation pass and every interaction formed
    /// over the position. The malformed maximum itself is refused at
    /// construction; this pins the belt the module's own contract claims
    /// for values that reach the encoder by other routes.
    ///
    /// ´claim:signal:the-ordinal-arm-sanitises-a-quotient-that-is-not-a-number´
    /// ´test:crate:ordinal-arm-sanitises-nan-quotient´
    #[test]
    fn ordinal_arm_sanitises_nan_quotient() {
        use crate::signal::{SignalShape, SignalValue, encode_signal};

        // 0 / 0 is NaN, and NaN passes f64::clamp unchanged; the arm
        // must replace it with zero per the module contract.
        let encoded = encode_signal(&SignalValue::Numeric(0.0), &SignalShape::Ordinal { max: 0.0 });
        assert_eq!(encoded, vec![0.0]);

        // The neighbouring well-formed reads are untouched.
        let encoded = encode_signal(&SignalValue::Numeric(2.0), &SignalShape::Ordinal { max: 4.0 });
        assert_eq!(encoded, vec![0.5]);
    }

    /// The two shape parameters that reach the assessment path are refused
    /// at the builder, so the host is told at build time — the surface that
    /// is fallible by design — instead of the defect surfacing later as a
    /// panic or a poisoned position on the infallible assessment call.
    ///
    /// (´claim:signal:a-shape-parameter-the-encoding-cannot-honour-is-refused-at-construction´)
    /// ´test:crate:builder-refuses-malformed-signal-declaration´
    #[test]
    fn builder_refuses_malformed_signal_declaration() {
        use crate::Assayer;
        use crate::config::types::AssayerConfig;
        use crate::error::BuildError;
        use crate::signal::{SignalDeclaration, SignalShape};

        let decls = vec![SignalDeclaration::new(
            "reversed",
            SignalShape::Scalar { clip: (1.0, 0.0) },
            Persistence::Entity,
        )];
        let result = Assayer::builder(AssayerConfig::default()).signal_schema(&decls).build();
        assert!(matches!(result, Err(BuildError::InvalidSignalShape { .. })));
    }
}
