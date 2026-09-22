// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`identity_register_deregister_round_trip`] | identity | An identity dimension can be brought up and taken down by name, and the registry agrees either side of the cycle: unknown before, resolvable to its identifier while live, unknown again afterwards, with health still clean. Asking to remove a name that was never registered is a scoped not-found error rather than a panic, so a caller confused about its own state is corrected instead of taking the engine down. |
//! | [`identity_register_allows_assess_and_derive`] | identity | Registering an identity dimension extends the model without disturbing what it produces: assessment still yields a well-formed reckoning, and both the sister and operational sub-predictors underneath the headline risk stay finite. Extension adds a dimension to the model's shape, so a dimension that arrived badly would surface as a NaN in a sub-predictor long before the combined figure looked wrong. |
//! | [`multiple_identities_can_measure_concurrently`] | identity | Two identity dimensions live together and assessment works with both; drop one and assessment still works with the survivor. Removing a dimension marginalises it out of the model rather than invalidating what the remaining dimensions learned, so the engine keeps answering across the change. |
//! | [`identity_churn_preserves_finite_assess_and_derive`] | identity | The same identity name can be registered, exercised with a labelled benign cycle, and deregistered a dozen times over without any round failing and with health clean at the end. Each cycle runs the extension and marginalisation cascade in full, so residue left behind by one round — a stale dimension slot, an orphaned cell, an event for a dimension that has gone — would accumulate and surface as a later round's failure. |
//! | [`multiple_identity_dimensions_coexist`] | identity | Three distinct identity dimensions can be live at once — all visible in the registry, all folded into a well-formed assessment — and can then be dismantled one at a time with a live assessment between each removal. The teardown order is the reverse of registration, so the cascade is exercised against a dimension that was not the most recently added. A race once made this scenario unrunnable: the maintenance loop could announce a cell entering the competitive set for a dimension already deregistered, and the model owner counted a dimension that no longer existed. Entries for unknown dimensions are now dropped on arrival, matching how exits were already treated. |
//! | [`multiple_domain_bits_widths_coexist`] | identity | Dimensions declared at 32, 64 and 128 bits of domain width coexist on one engine: each registers, all three are live for the same assessment, and each can be torn down without poisoning the rest. Coordinates are left-justified into the internal domain, so a narrow width names the same dyadic intervals as the wide one rather than a squeezed coordinate space of its own — which is why a heterogeneous set of widths needs no special handling in the model's dimension map. |
//! | [`identity_registration_rejects_domain_bits_zero`] | identity | A dimension declaring a zero-width domain is refused outright, with an error naming both the offending identifier and the width it asked for, and health stays clean afterwards. Rejection happens before any lifecycle effect, so a nonsensical registration leaves nothing half-created behind it — there is no coordinate space to partition and no way to encode an entity into one. |
//! | [`identity_registration_rejects_domain_bits_above_128`] | identity | cites (´claim:identity:a-domain-width-outside-the-supported-range-is-refused-before-any-lifecycle-effect´) |
//! | [`identity_audit_metadata_is_reported_without_disturbing_assessment`] | identity | The host's description and coordinate semantics survive registration into public health diagnostics, while an assessment through the dimension remains well formed. The two strings are therefore carried audit context rather than inputs to Core behaviour. |
//! | [`identity_observation_overflow_is_reported_and_assessments_stay_finite`] | identity | A dimension given a queue of one slot is overwhelmed by a burst of five hundred assessments, yet every one of those assessments returns a finite risk strictly inside the open unit interval, and a derivation afterwards is still well formed. The dropped observations are not hidden: the dimension's entry in the public health report shows a non-zero dropped count, so an operator can see that this dimension's competitive geometry is being starved even though nothing in the request path failed. |

//! Integration tests for the identity layer
//! (´chap:spec:key-space-identity´).
//!
//! These scenarios exercise the identity-dimension lifecycle and the
//! assess/derive path through a registered identity dimension, driven from
//! the public `World` harness surface. Each scenario is clock-
//! independent: no virtual-time advance is needed to observe the
//! invariant under test.
//!
//! # Cross-References
//!
//! - (´chap:spec:registries-and-lifecycle´) — structure changes without destroying what was learned
//! - (´alg:axis:lifecycle´) — the companion register/deregister round-trip for outcome axes
//! - (´chap:spec:key-space-identity´) — Sentinels of differing domain widths at the same midpoint
//! - (´dec:construction:eager-validation´) — invalid `domain_bits` is refused at the builder
//! - (´dec:memory:overflow-degrades´) — identity observation-queue overflow degrades and is counted
//! - (´dec:memory:coordinate-depth-key´) — the identity layer's coordinate-and-depth keying
//! - (´dec:construction:blocking-deregistration´) — identity deregistration blocks

use torrust_assayer::IdentityDimensionRegistration;
use torrust_assayer::error::LifecycleError;
use torrust_assayer::testing::{assert_health_clean, assert_reckoning_well_formed, scenario};
use torrust_assayer::types::{DimensionId, EntityKey, IdentityBudget};

const INSTANCE: &str = "identity-layer";
const SEED: u64 = 0x1DEA_BEEF;

const fn zero_identity_encoder(_: &EntityKey) -> u128 {
    0
}

fn identity_registration_with_domain_bits(domain_bits: u8) -> IdentityDimensionRegistration {
    IdentityDimensionRegistration {
        id: DimensionId(9000),
        name: format!("invalid-domain-bits-{domain_bits}"),
        description: "invalid-width test hierarchy".to_owned(),
        coordinate_semantics: "prefixes identify test groups".to_owned(),
        domain_bits,
        depth_cutoff: 10,
        budget: IdentityBudget::for_depth_cutoff(10),
        encode: zero_identity_encoder,
    }
}

fn identity_registration_with_capacity(id: DimensionId, capacity: usize) -> IdentityDimensionRegistration {
    IdentityDimensionRegistration {
        id,
        name: format!("tiny-observation-queue-{}", id.0),
        description: "observation-overflow test hierarchy".to_owned(),
        coordinate_semantics: "prefixes identify test groups".to_owned(),
        domain_bits: 128,
        depth_cutoff: 10,
        budget: IdentityBudget {
            observation_capacity: capacity,
            ..IdentityBudget::for_depth_cutoff(10)
        },
        encode: zero_identity_encoder,
    }
}

/// An identity dimension can be brought up and taken down by name, and the
/// registry agrees either side of the cycle: unknown before, resolvable to its
/// identifier while live, unknown again afterwards, with health still clean.
/// Asking to remove a name that was never registered is a scoped not-found
/// error rather than a panic, so a caller confused about its own state is
/// corrected instead of taking the engine down.
///
/// ´claim:identity:a-dimension-can-be-registered-and-deregistered-by-name-with-the-registry-agreeing-either-side´
/// ´test:integration:identity-register-deregister-round-trip´
#[test]
fn identity_register_deregister_round_trip() {
    let mut s = scenario(INSTANCE, SEED);

    // Fresh world → no identities registered.
    assert_eq!(s.identity("ip-hash"), None);

    let id = s.register_identity("ip-hash").expect("register identity");
    assert_eq!(s.identity("ip-hash"), Some(id));

    s.deregister_identity("ip-hash").expect("deregister identity");
    assert_eq!(s.identity("ip-hash"), None);

    // Deregistering an unknown name is a scoped NotFound, not a panic.
    let err = s.deregister_identity("ghost").unwrap_err();
    assert!(matches!(err, LifecycleError::NotFound { .. }), "unexpected error: {err:?}");

    assert_health_clean(&s);
}

/// Registering an identity dimension extends the model without disturbing what
/// it produces: assessment still yields a well-formed reckoning, and both the
/// sister and operational sub-predictors underneath the headline risk stay
/// finite. Extension adds a dimension to the model's shape, so a dimension that
/// arrived badly would surface as a NaN in a sub-predictor long before the
/// combined figure looked wrong.
///
/// ´claim:identity:a-registered-dimension-leaves-assessment-well-formed-down-to-its-sub-predictors´
/// ´test:integration:identity-register-allows-assess-and-derive´
#[test]
fn identity_register_allows_assess_and_derive() {
    let mut s = scenario(INSTANCE, SEED);
    s.register_identity("ip-hash").expect("register identity");

    let r = s.derive_default("alice").expect("assess with identity dimension registered");

    assert_reckoning_well_formed(&r, "p_bad");
    assert!(
        r.assessment.risk.p_bad_sister.is_finite() && r.assessment.risk.p_bad_operational.is_finite(),
        "sub-predictors should be finite: sister={}, operational={}",
        r.assessment.risk.p_bad_sister,
        r.assessment.risk.p_bad_operational,
    );
    assert_health_clean(&s);
}

/// Two identity dimensions live together and assessment works with both; drop
/// one and assessment still works with the survivor. Removing a dimension
/// marginalises it out of the model rather than invalidating what the remaining
/// dimensions learned, so the engine keeps answering across the change.
///
/// ´claim:identity:dropping-one-dimension-leaves-the-others-answering-as-before´
/// ´test:integration:multiple-identities-can-measure-concurrently´
#[test]
fn multiple_identities_can_measure_concurrently() {
    let mut s = scenario(INSTANCE, SEED);

    s.register_identity("ip-hash").expect("first identity registers");
    s.register_identity("ua-hash").expect("second identity registers");

    let r = s.derive_default("alice").expect("assess with multiple identities");
    assert_reckoning_well_formed(&r, "p_bad with multiple identities");

    s.deregister_identity("ip-hash").expect("deregister first identity");
    let r = s.derive_default("bob").expect("assess after deregistering one identity");
    assert_reckoning_well_formed(&r, "p_bad after dropping one identity");

    assert_health_clean(&s);
}

/// The same identity name can be registered, exercised with a labelled benign
/// cycle, and deregistered a dozen times over without any round failing and
/// with health clean at the end. Each cycle runs the extension and
/// marginalisation cascade in full, so residue left behind by one round — a
/// stale dimension slot, an orphaned cell, an event for a dimension that has
/// gone — would accumulate and surface as a later round's failure.
///
/// ´claim:identity:repeated-churn-of-one-dimension-name-leaves-no-residue-behind´
/// ´test:integration:identity-churn-preserves-finite-assess-and-derive´
#[test]
fn identity_churn_preserves_finite_assess_and_derive() {
    let mut s = scenario(INSTANCE, SEED);

    for round in 0..12 {
        s.register_identity("ephemeral")
            .unwrap_or_else(|e| panic!("round {round}: register failed: {e:?}"));

        let r = s.cycle_benign("alice");
        assert_reckoning_well_formed(&r, &format!("round {round}: p_bad"));

        s.deregister_identity("ephemeral")
            .unwrap_or_else(|e| panic!("round {round}: deregister failed: {e:?}"));
    }

    assert_health_clean(&s);
}

/// Three distinct identity dimensions can be live at once — all visible in the
/// registry, all folded into a well-formed assessment — and can then be
/// dismantled one at a time with a live assessment between each removal. The
/// teardown order is the reverse of registration, so the cascade is exercised
/// against a dimension that was not the most recently added.
///
/// A race once made this scenario unrunnable: the maintenance loop could
/// announce a cell entering the competitive set for a dimension already
/// deregistered, and the model owner counted a dimension that no longer
/// existed. Entries for unknown dimensions are now dropped on arrival, matching
/// how exits were already treated.
///
/// ´claim:identity:several-dimensions-live-together-and-can-be-dismantled-one-at-a-time´
/// ´test:integration:multiple-identity-dimensions-coexist´
#[test]
fn multiple_identity_dimensions_coexist() {
    let mut s = scenario(INSTANCE, SEED);

    // NOTE: intentionally no Sentinel is registered here. Combining a
    // Sentinel with ≥2 identity dimensions is covered (as it becomes
    // available) by separate lifecycle slices
    // (´chap:spec:registries-and-lifecycle´); this scenario exercises
    // the pure identity-layer cascade.
    //
    // Historical note: this test was previously `#[ignore]`-gated
    // because the identity-maintenance loop could emit
    // `CompetitiveCellEntry` events for a dimension *after* it had
    // been deregistered, tripping a `debug_assert_eq!` in
    // `owner::lifecycle::rebuild_dimension_map` (DimensionMap.p
    // drifted by 1 vs model_p). The race is now handled by a guard
    // in `handle_competitive_cell_entry` that drops cell entries for
    // unknown identity dimensions, symmetric to the existing guard
    // in `handle_competitive_cell_exit`.

    let ids = ["ip-hash", "ua-hash", "asn"];
    for name in ids {
        s.register_identity(name).unwrap_or_else(|e| panic!("register {name}: {e:?}"));
    }

    // All three must be visible in the name registry.
    for name in ids {
        assert!(s.identity(name).is_some(), "identity {name} should be registered");
    }

    let r = s.derive_default("alice").expect("assess with three identities registered");
    assert_reckoning_well_formed(&r, "p_bad with three identities");

    // Deregister in reverse order; a live assess/derive between each
    // deregistration exercises the marginalisation / extension
    // cascade at every step.
    for name in ids.iter().rev() {
        s.deregister_identity(*name)
            .unwrap_or_else(|e| panic!("deregister {name}: {e:?}"));
        let r = s
            .derive_default("bob")
            .unwrap_or_else(|e| panic!("assess after dropping {name}: {e:?}"));
        assert_reckoning_well_formed(&r, &format!("p_bad after dropping {name}"));
    }

    assert_health_clean(&s);
}

/// Dimensions declared at 32, 64 and 128 bits of domain width coexist on one
/// engine: each registers, all three are live for the same assessment, and each
/// can be torn down without poisoning the rest. Coordinates are left-justified
/// into the internal domain, so a narrow width names the same dyadic intervals
/// as the wide one rather than a squeezed coordinate space of its own — which is
/// why a heterogeneous set of widths needs no special handling in the model's
/// dimension map.
///
/// ´claim:identity:dimensions-of-different-domain-widths-coexist-on-one-engine´
/// ´test:integration:multiple-domain-bits-widths-coexist´
#[test]
#[allow(clippy::items_after_statements, clippy::type_complexity, clippy::cast_possible_truncation)]
fn multiple_domain_bits_widths_coexist() {
    // The spec permits any `domain_bits` width up to 128
    // (´chap:spec:key-space-identity´).
    // The wider property reports observations against four
    // Sentinels at widths 8, 32, 64, and 128 and asserts the engine
    // routes each into the same internal left-justified midpoint.
    // Sentinels do not currently expose a `domain_bits` knob through
    // the public API, but identity dimensions do — and the dyadic-
    // coordinate routing logic is shared. This slice registers three
    // identity dimensions at widths 32, 64, and 128 on the same
    // engine and asserts:
    //
    //   1. each registration succeeds (the engine accepts narrow
    //      widths alongside the default);
    //   2. all three coexist in the name registry concurrently;
    //   3. `assess()` produces a finite, in-range risk assessment with
    //      every width live (the model's dimension map handles the
    //      heterogeneous extension shape);
    //   4. the health snapshot stays clean across the whole cycle;
    //   5. each width can be torn down independently afterwards
    //      without poisoning the others.
    //
    // The width-32/64 dimensions go through the public surface
    // directly because [`World::register_identity`] hard-codes 128
    // (matching the harness's conventional encoding); narrower
    // widths require an explicit `IdentityDimensionRegistration`.
    let s = scenario(INSTANCE, SEED);

    // Three distinct named encoders: `encode` is a `fn` pointer, not
    // a closure, so it cannot capture the per-width mask. The masks
    // are encoded directly in each function body. Each takes the
    // first eight bytes of the entity key as a little-endian `u64`,
    // then masks to the dimension's domain width — the engine left-
    // justifies the coordinate internally
    // (´chap:spec:derivation-interface´), so a value
    // masked to 32 or 64 bits routes to the same dyadic interval at
    // every depth as the corresponding 128-bit value.
    fn read_u64_le(entity: &EntityKey) -> u128 {
        let bytes = entity.as_bytes();
        let mut buf = [0u8; 8];
        let n = bytes.len().min(8);
        buf[..n].copy_from_slice(&bytes[..n]);
        u128::from(u64::from_le_bytes(buf))
    }
    fn encode_32(entity: &EntityKey) -> u128 {
        read_u64_le(entity) & ((1u128 << 32) - 1)
    }
    fn encode_64(entity: &EntityKey) -> u128 {
        read_u64_le(entity) & u128::from(u64::MAX)
    }
    fn encode_128(entity: &EntityKey) -> u128 {
        read_u64_le(entity)
    }

    let widths: [(&str, u8, fn(&EntityKey) -> u128); 3] = [
        ("ip-32", 32, encode_32),
        ("ip-64", 64, encode_64),
        ("ip-128", 128, encode_128),
    ];

    let assayer = s.assayer();
    for (i, (name, bits, encode)) in widths.iter().enumerate() {
        assayer
            .register_identity_dimension(IdentityDimensionRegistration {
                id: DimensionId(i as u32),
                name: (*name).to_owned(),
                description: format!("{name} test hierarchy"),
                coordinate_semantics: "prefixes identify test groups".to_owned(),
                domain_bits: *bits,
                depth_cutoff: 10,
                budget: IdentityBudget::for_depth_cutoff(10),
                encode: *encode,
            })
            .unwrap_or_else(|e| panic!("register {name} (domain_bits={bits}): {e:?}"));
    }

    // All three widths live concurrently — the dimension map carries
    // each one independently.
    let r = s
        .derive_default("alice")
        .expect("assess with three different domain_bits widths registered");
    assert_reckoning_well_formed(&r, "p_bad with widths {32, 64, 128}");

    // Tear them down one at a time; every assess() in between must
    // remain well-formed, confirming the marginalisation cascade is
    // width-agnostic.
    for (i, (name, _, _)) in widths.iter().enumerate() {
        assayer
            .deregister_identity_dimension(DimensionId(i as u32))
            .unwrap_or_else(|e| panic!("deregister {name}: {e:?}"));
        let r = s
            .derive_default("bob")
            .unwrap_or_else(|e| panic!("assess after dropping {name}: {e:?}"));
        assert_reckoning_well_formed(&r, &format!("p_bad after dropping {name}"));
    }

    assert_health_clean(&s);
}

/// A dimension declaring a zero-width domain is refused outright, with an error
/// naming both the offending identifier and the width it asked for, and health
/// stays clean afterwards. Rejection happens before any lifecycle effect, so a
/// nonsensical registration leaves nothing half-created behind it — there is no
/// coordinate space to partition and no way to encode an entity into one.
///
/// ´claim:identity:a-domain-width-outside-the-supported-range-is-refused-before-any-lifecycle-effect´
/// ´test:integration:identity-registration-rejects-domain-bits-zero´
#[test]
fn identity_registration_rejects_domain_bits_zero() {
    let s = scenario(INSTANCE, SEED);

    let err = s
        .assayer()
        .register_identity_dimension(identity_registration_with_domain_bits(0))
        .unwrap_err();

    assert!(
        matches!(
            err,
            LifecycleError::InvalidDomainBits {
                id: DimensionId(9000),
                domain_bits: 0,
            }
        ),
        "unexpected error: {err:?}",
    );
    assert_health_clean(&s);
}

/// The upper end is guarded as firmly as the lower: a width one bit past the
/// engine's 128-bit domain is refused with the same error, again before
/// anything is created and again leaving health clean. The coordinate type
/// simply cannot hold more, so accepting the request would mean silently
/// truncating the entity space a caller thought it had asked for.
///
/// (´claim:identity:a-domain-width-outside-the-supported-range-is-refused-before-any-lifecycle-effect´)
/// ´test:integration:identity-registration-rejects-domain-bits-above-128´
#[test]
fn identity_registration_rejects_domain_bits_above_128() {
    let s = scenario(INSTANCE, SEED);

    let err = s
        .assayer()
        .register_identity_dimension(identity_registration_with_domain_bits(129))
        .unwrap_err();

    assert!(
        matches!(
            err,
            LifecycleError::InvalidDomainBits {
                id: DimensionId(9000),
                domain_bits: 129,
            }
        ),
        "unexpected error: {err:?}",
    );
    assert_health_clean(&s);
}

/// The host's description and coordinate semantics survive registration into
/// public health diagnostics, while an assessment through the dimension
/// remains well formed. The two strings are therefore carried audit context
/// rather than inputs to Core behaviour.
///
/// ´claim:identity:dimension-audit-metadata-is-reported-and-does-not-drive-core-behaviour´
/// ´test:integration:identity-audit-metadata-is-reported-without-disturbing-assessment´
#[test]
fn identity_audit_metadata_is_reported_without_disturbing_assessment() {
    let s = scenario(INSTANCE, SEED);
    let dim_id = DimensionId(9050);
    let mut registration = identity_registration_with_capacity(dim_id, 64);
    registration.description = "tenant hierarchy used for abuse isolation".to_owned();
    registration.coordinate_semantics =
        "tenant identifier in the high bits; shared prefixes assert shared administration".to_owned();

    s.assayer()
        .register_identity_dimension(registration)
        .expect("register identity dimension with audit metadata");
    s.flush_labels().expect("flush identity lifecycle registration");

    let assessment = s.core_assess(s.request("default", "metadata-probe"));
    assert!(assessment.risk.p_bad.is_finite());

    let report = s.assayer().full_health_report();
    let identity = report
        .identity_dimensions
        .get(&dim_id)
        .expect("registered identity dimension should appear in full health report");
    assert_eq!(identity.name, "tiny-observation-queue-9050");
    assert_eq!(identity.description, "tenant hierarchy used for abuse isolation");
    assert_eq!(
        identity.coordinate_semantics,
        "tenant identifier in the high bits; shared prefixes assert shared administration"
    );
}

/// A dimension given a queue of one slot is overwhelmed by a burst of five
/// hundred assessments, yet every one of those assessments returns a finite
/// risk strictly inside the open unit interval, and a derivation afterwards is
/// still well formed. The dropped observations are not hidden: the dimension's
/// entry in the public health report shows a non-zero dropped count, so an
/// operator can see that this dimension's competitive geometry is being starved
/// even though nothing in the request path failed.
///
/// ´claim:identity:a-starved-observation-queue-is-reported-through-public-health-while-assessments-stay-well-formed´
/// ´test:integration:identity-observation-overflow-is-reported-and-assessments-stay-finite´
#[test]
fn identity_observation_overflow_is_reported_and_assessments_stay_finite() {
    let s = scenario(INSTANCE, SEED);
    let dim_id = DimensionId(9100);

    s.assayer()
        .register_identity_dimension(identity_registration_with_capacity(dim_id, 1))
        .expect("register identity with capacity-1 observation queue");
    s.flush_labels().expect("flush identity lifecycle registration");

    for i in 0..512 {
        let entity = format!("overflow-{i}");
        let assessment = s.core_assess(s.request("default", &entity));
        assert!(assessment.risk.p_bad.is_finite(), "assessment {i}: p_bad should be finite");
        assert!(
            assessment.risk.p_bad > 0.0 && assessment.risk.p_bad < 1.0,
            "assessment {i}: p_bad should be in (0, 1), got {}",
            assessment.risk.p_bad,
        );
    }

    let report = s.assayer().full_health_report();
    let identity = report
        .identity_dimensions
        .get(&dim_id)
        .expect("registered identity dimension should appear in full health report");
    assert!(
        identity.observations_dropped > 0,
        "capacity-1 queue should expose dropped observations in health report",
    );

    let r = s.derive_default("post-overflow").expect("derive after overflow burst");
    assert_reckoning_well_formed(&r, "p_bad after identity observation overflow");
}
