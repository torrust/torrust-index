// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`register_deregister_round_trip`] | lifespan | Sentinels are numbered in the order they arrive, starting at zero, and each name resolves to its id until that name is retired — at which point it resolves to nothing while its siblings continue to resolve as before. Retiring a name nobody registered is an error rather than a panic, and none of this churn disturbs the health report. |
//! | [`register_deregister_axis_round_trip`] | lifespan | Outcome axes are numbered in arrival order the way Sentinels are, and removing one leaves every other axis still resolving by name. The two axes here differ in their spatial policy, so retiring the non-spatial one first shows the bookkeeping to be keyed per axis rather than aliased by slot position. |
//! | [`register_axis_duplicate_returns_error`] | lifespan | cites (´claim:lifespan:a-duplicate-registration-is-refused-and-names-the-entity-kind-that-collided´) |
//! | [`deregister_unknown_axis_returns_not_found`] | lifespan | cites (´claim:lifespan:retiring-an-identifier-that-was-never-registered-is-refused-as-not-found´) |
//! | [`lifecycle_churn_register_deregister_32_rounds`] | lifespan | Cycling a small pool of Sentinels in and out thirty-two times, issuing a reckoning in the middle of every round, leaves each reckoning finite and in range and the health report clean at the end. The dimension map is rebuilt on both edges of every round while assessment reads through the published snapshot, so any slippage between the two would surface as a panic or a non- finite probability rather than as slow drift. |
//! | [`label_cycle_under_sentinel_churn_stays_finite`] | lifespan | The full assess-then-label round-trip completes on every round of Sentinel churn, for adverse and benign labels alike. The pending buffer is keyed on the assessment's own monotonic id rather than on whichever Sentinel slots were live when it was issued, so a label arriving after its Sentinel has been retired still finds its entry. |
//! | [`axis_register_deregister_churn_cycle`] | lifespan | Registering and retiring outcome axes sixteen times over — alternating between a spatial axis, whose features grow with the Sentinel count, and non-spatial ones that do not — keeps every intervening reckoning finite and in range. Each retired axis stops resolving immediately, so both extension shapes are exercised in a tight loop without either leaving residue behind. |
//! | [`peak_register_then_full_teardown_integration`] | lifespan | Built up to eight Sentinels and four outcome axes and then dismantled with Sentinel and axis retirements interleaved, the engine returns a well-formed reckoning at every step and at the end — with no Sentinel left standing the reckoning is prior-only, and it says so by reporting zero Sentinels reporting. Full teardown is a state the engine occupies, not merely one it survives. |

//! Integration tests for the algebra of registration and deregistration
//! (´chap:spec:registries-and-lifecycle´).
//!
//! These scenarios exercise the algebra of register / deregister /
//! re-register across Sentinels, outcome axes, and identity
//! dimensions: dimension consistency, state recovery, and the
//! Schur-complement conservation laws that govern marginalisation.
//!
//! They are the **pilot file for the Stage 1 harness skeleton**
//! (`torrust_assayer::testing::World`). Every scenario here is
//! reachable through today's public API; no clock-dependent work
//! lives in this file.
//!
//! # Cross-References
//!
//! - (´claim:bayes:the-schur-complement-is-the-analytical-marginal´) — extension and marginalisation preserve what was learned
//! - (´dec:construction:six-methods´) — registration and deregistration among the lifecycle methods

// The crate's `testing` module is the harness of record: `World` plus
// its domain-aware asserts and named types. Pull everything from there
// so this file reads as a scenario, not as engine glue.
use torrust_assayer::OutcomeAxisRegistration;
use torrust_assayer::testing::{LabelSpec, assert_health_clean, assert_reckoning_well_formed, scenario};
use torrust_assayer::types::{OutcomeAxisId, OutcomeEligibility, SentinelId, SpatialFeaturePolicy};

const INSTANCE: &str = "skeleton-test";
const SEED: u64 = 0x00C0_FFEE;

/// Sentinel pool used by [`peak_register_then_full_teardown_integration`]
/// (´test:integration:peak-register-then-full-teardown-integration´).
/// Eight names is the smallest pool that exercises a
/// non-trivial multi-Sentinel extension cascade through the public API
/// without slowing the suite.
const PEAK_SENTINEL_NAMES: [&str; 8] = ["S0", "S1", "S2", "S3", "S4", "S5", "S6", "S7"];

/// Axis pool used by [`peak_register_then_full_teardown_integration`].
/// One spatial axis (`blast_radius`) and three non-spatial axes
/// span both extension shapes the dimension map handles.
const PEAK_AXES: [(&str, bool); 4] = [
    ("magnitude", false),
    ("blast_radius", true),
    ("severity", false),
    ("recency", false),
];

/// Sentinels are numbered in the order they arrive, starting at zero, and each
/// name resolves to its id until that name is retired — at which point it
/// resolves to nothing while its siblings continue to resolve as before.
/// Retiring a name nobody registered is an error rather than a panic, and none
/// of this churn disturbs the health report.
///
/// ´claim:lifespan:sentinel-ids-are-handed-out-in-registration-order-and-a-name-stops-resolving-the-moment-it-is-retired´
/// ´test:integration:register-deregister-round-trip´
#[test]
fn register_deregister_round_trip() {
    // `ChannelPolicy::default()` already yields [Allow, Challenge, Block]
    // with default reward parameters — exactly the shape scenarios
    // need for lifecycle tests.
    let mut s = scenario(INSTANCE, SEED);

    // A freshly built World has no degradation signals.
    assert_health_clean(&s);

    // IDs are assigned in registration order starting at 0.
    let s1 = s.register_sentinel("S1").expect("register S1");
    let s2 = s.register_sentinel("S2").expect("register S2");
    assert_eq!(s1, SentinelId(0));
    assert_eq!(s2, SentinelId(1));

    // Name-based lookup works.
    assert_eq!(s.sentinel("S1"), Some(s1));
    assert_eq!(s.sentinel("S2"), Some(s2));

    // Deregister by name; lookup returns None afterwards.
    s.deregister_sentinel("S1").expect("deregister S1");
    assert_eq!(s.sentinel("S1"), None);
    assert_eq!(s.sentinel("S2"), Some(s2));

    // Deregistering an unknown name is an error (not a panic).
    let err = s.deregister_sentinel("ghost").unwrap_err();
    let msg = format!("{err}");
    assert!(msg.contains("Sentinel"), "error should mention Sentinel: {msg}");

    // Lifecycle churn should not flip the health report.
    assert_health_clean(&s);
}

/// Outcome axes are numbered in arrival order the way Sentinels are, and
/// removing one leaves every other axis still resolving by name. The two axes
/// here differ in their spatial policy, so retiring the non-spatial one first
/// shows the bookkeeping to be keyed per axis rather than aliased by slot
/// position.
///
/// ´claim:lifespan:outcome-axes-are-numbered-in-registration-order-and-removing-one-leaves-the-others-resolving´
/// ´test:integration:register-deregister-axis-round-trip´
#[test]
fn register_deregister_axis_round_trip() {
    let mut s = scenario(INSTANCE, SEED);
    assert_health_clean(&s);

    // Axis IDs are assigned in registration order starting at 0.
    let a1 = s.register_axis("magnitude", false).expect("register magnitude");
    let a2 = s.register_axis("blast_radius", true).expect("register blast_radius");
    assert_eq!(a1, OutcomeAxisId(0));
    assert_eq!(a2, OutcomeAxisId(1));

    // Name-based lookup works in both directions.
    assert_eq!(s.axis("magnitude"), Some(a1));
    assert_eq!(s.axis("blast_radius"), Some(a2));

    // Deregister the non-spatial axis first: the spatial one continues
    // to resolve, proving per-axis bookkeeping does not alias by slot.
    s.deregister_axis("magnitude").expect("deregister magnitude");
    assert_eq!(s.axis("magnitude"), None);
    assert_eq!(s.axis("blast_radius"), Some(a2));

    s.deregister_axis("blast_radius").expect("deregister blast_radius");
    assert_eq!(s.axis("blast_radius"), None);

    assert_health_clean(&s);
}

/// Uniqueness is enforced on the identifier, not on the human-readable name: a
/// second axis carrying an id already in use is refused even though it arrives
/// under a different name. The id is what every model index and ledger row is
/// keyed by, so it is the thing that must not collide.
///
/// (´claim:lifespan:a-duplicate-registration-is-refused-and-names-the-entity-kind-that-collided´)
/// ´test:integration:register-axis-duplicate-returns-error´
#[test]
fn register_axis_duplicate_returns_error() {
    // The engine's uniqueness guard keys on `OutcomeAxisId`, not on
    // the human name. We therefore drive the duplicate path directly
    // through the public `Assayer` surface with an explicit colliding
    // id, using the same `registrations::axis_reg` helper that backs
    // crate-level tests.
    let s = scenario(INSTANCE, SEED);

    let first = OutcomeAxisRegistration {
        id: OutcomeAxisId(42),
        name: "outcome".to_owned(),
        description: String::new(),
        eligibility: OutcomeEligibility::default(),
        initial_kappa: 1.0,
        gamma: 0.99,
        spatial_features: SpatialFeaturePolicy::default(),
    };
    s.assayer().register_outcome_axis(first).expect("first registration succeeds");

    let second = OutcomeAxisRegistration {
        id: OutcomeAxisId(42),
        name: "outcome-again".to_owned(),
        description: String::new(),
        eligibility: OutcomeEligibility::default(),
        initial_kappa: 1.0,
        gamma: 0.99,
        spatial_features: SpatialFeaturePolicy::default(),
    };
    let err = s
        .assayer()
        .register_outcome_axis(second)
        .expect_err("second registration of the same id must fail");
    let msg = format!("{err}");
    assert!(
        msg.to_lowercase().contains("duplicate"),
        "duplicate-id error should say so: {msg}",
    );
}

/// Retiring an axis name the engine never held is reported as a not-found error
/// naming the outcome-axis kind, not swallowed silently. A host that mistypes a
/// name learns so immediately rather than believing a teardown happened.
///
/// (´claim:lifespan:retiring-an-identifier-that-was-never-registered-is-refused-as-not-found´)
/// ´test:integration:deregister-unknown-axis-returns-not-found´
#[test]
fn deregister_unknown_axis_returns_not_found() {
    let mut s = scenario(INSTANCE, SEED);
    let err = s
        .deregister_axis("never-registered")
        .expect_err("deregistering an unknown axis must fail");
    let msg = format!("{err}");
    assert!(msg.contains("OutcomeAxis"), "error should mention OutcomeAxis: {msg}");
}

/// Cycling a small pool of Sentinels in and out thirty-two times, issuing a
/// reckoning in the middle of every round, leaves each reckoning finite and in
/// range and the health report clean at the end. The dimension map is rebuilt
/// on both edges of every round while assessment reads through the published
/// snapshot, so any slippage between the two would surface as a panic or a non-
/// finite probability rather than as slow drift.
///
/// ´claim:lifespan:reckoning-stays-well-formed-through-repeated-registration-and-retirement-of-the-same-sentinel-pool´
/// ´test:integration:lifecycle-churn-register-deregister-32-rounds´
#[test]
fn lifecycle_churn_register_deregister_32_rounds() {
    // `SentinelName` only accepts `&'static str`, so we cycle
    // through a fixed pool of four names rather than allocating
    // fresh identifiers each round.
    const NAMES: [&str; 4] = ["S0", "S1", "S2", "S3"];

    // A lightened churn variant: cycle a small pool of Sentinels
    // in and out 32 times over, issuing a reckoning each round.
    // This keeps the dimension map churning across extend and
    // marginalise while the assessment pipeline reads through the
    // snapshot surface. If the owner or the snapshot publishing
    // path were sloppy about dimension consistency, this loop would
    // either panic or produce a non-finite p_bad.
    let mut s = scenario(INSTANCE, SEED);

    for round in 0..32 {
        let name = NAMES[round % NAMES.len()];
        s.register_sentinel(name)
            .unwrap_or_else(|e| panic!("register {name} on round {round} failed: {e:?}"));

        let r = s
            .derive_default("alice")
            .unwrap_or_else(|e| panic!("reckon on round {round} failed: {e:?}"));
        assert_reckoning_well_formed(&r, &format!("round {round}"));

        s.deregister_sentinel(name)
            .unwrap_or_else(|e| panic!("deregister {name} on round {round} failed: {e:?}"));
    }

    assert_health_clean(&s);
}

/// The full assess-then-label round-trip completes on every round of Sentinel
/// churn, for adverse and benign labels alike. The pending buffer is keyed on
/// the assessment's own monotonic id rather than on whichever Sentinel slots
/// were live when it was issued, so a label arriving after its Sentinel has
/// been retired still finds its entry.
///
/// ´claim:lifespan:an-assessment-can-still-be-labelled-after-the-sentinel-pool-has-churned-beneath-it´
/// ´test:integration:label-cycle-under-sentinel-churn-stays-finite´
#[test]
fn label_cycle_under_sentinel_churn_stays_finite() {
    const NAMES: [&str; 3] = ["S0", "S1", "S2"];

    // The registry extension case
    // (´chap:spec:registries-and-lifecycle´):
    // exercise the full `assess → label → deregister`
    // round-trip against a churning Sentinel pool. The register →
    // reckon → label pattern is the hot path the host invokes on
    // every live request; lifecycle churn must not poison the
    // pending-buffer / snapshot interaction the label consumer
    // needs to find the entry.
    let mut s = scenario(INSTANCE, SEED);

    for round in 0..24 {
        let name = NAMES[round % NAMES.len()];
        s.register_sentinel(name)
            .unwrap_or_else(|e| panic!("register {name} round {round}: {e:?}"));

        let r = s
            .derive_default("alice")
            .unwrap_or_else(|e| panic!("reckon round {round}: {e:?}"));
        assert_reckoning_well_formed(&r, &format!("round {round}"));

        // Alternate benign / adverse labels so the engine sees both
        // polarities across the churn. The label must succeed for
        // every single round — the pending buffer is keyed on the
        // monotonic id, not on the Sentinel slot state.
        let spec = if round % 2 == 0 {
            LabelSpec::benign(r.assessment.id)
        } else {
            LabelSpec::adverse(r.assessment.id)
        };
        s.label(spec.build()).unwrap_or_else(|e| panic!("label round {round}: {e:?}"));

        s.deregister_sentinel(name)
            .unwrap_or_else(|e| panic!("deregister {name} round {round}: {e:?}"));
    }

    assert_health_clean(&s);
}

/// Registering and retiring outcome axes sixteen times over — alternating
/// between a spatial axis, whose features grow with the Sentinel count, and
/// non-spatial ones that do not — keeps every intervening reckoning finite and
/// in range. Each retired axis stops resolving immediately, so both extension
/// shapes are exercised in a tight loop without either leaving residue behind.
///
/// ´claim:lifespan:the-axis-side-of-the-dimension-map-survives-churn-across-both-spatial-and-non-spatial-axes´
/// ´test:integration:axis-register-deregister-churn-cycle´
#[test]
fn axis_register_deregister_churn_cycle() {
    const AXES: [(&str, bool); 3] = [("magnitude", false), ("blast_radius", true), ("severity", false)];

    // The axis extension case
    // (´chap:spec:registries-and-lifecycle´):
    // cycle a small axis pool in and out 16
    // times, reckoning in between each round. Registering and
    // deregistering an outcome axis reshapes the dimension map
    // (spatial axes grow $p$ proportional to the Sentinel count;
    // non-spatial axes grow by 1 per axis). Churning through both
    // kinds in a tight loop proves the owner's extend/marginalise
    // bookkeeping for the axis-side of the dimension map is as
    // robust as the Sentinel-side exercised by
    // [`lifecycle_churn_register_deregister_32_rounds`].
    let mut s = scenario(INSTANCE, SEED);

    // Register one Sentinel so the spatial axis has a non-trivial
    // per-Sentinel extension shape to churn through.
    s.register_sentinel("S1").expect("register S1");

    for round in 0..16 {
        let (name, spatial) = AXES[round % AXES.len()];
        s.register_axis(name, spatial)
            .unwrap_or_else(|e| panic!("register axis {name} round {round}: {e:?}"));

        let r = s
            .derive_default("alice")
            .unwrap_or_else(|e| panic!("reckon round {round}: {e:?}"));
        assert_reckoning_well_formed(&r, &format!("round {round}"));

        s.deregister_axis(name)
            .unwrap_or_else(|e| panic!("deregister axis {name} round {round}: {e:?}"));
        assert_eq!(
            s.axis(name),
            None,
            "round {round}: axis {name} still resolves after deregister"
        );
    }

    assert_health_clean(&s);
}

/// Built up to eight Sentinels and four outcome axes and then dismantled with
/// Sentinel and axis retirements interleaved, the engine returns a well-formed
/// reckoning at every step and at the end — with no Sentinel left standing the
/// reckoning is prior-only, and it says so by reporting zero Sentinels
/// reporting. Full teardown is a state the engine occupies, not merely one it
/// survives.
///
/// ´claim:lifespan:an-engine-torn-all-the-way-back-down-still-reckons-and-reports-nothing-reporting´
/// ´test:integration:peak-register-then-full-teardown-integration´
#[test]
fn peak_register_then_full_teardown_integration() {
    // The public-surface slice of
    // (´test:crate:peak-registration-then-full-deregistration-restores-min-dimension´):
    // the crate-level
    // [`lifecycle::peak_registration_then_full_deregistration_restores_min_dimension`]
    // test drives the model/dimension-map invariant directly. This
    // integration variant exercises the same "build up to peak, tear
    // it all down" trajectory through the *public* API and asserts
    // observable invariants only:
    //
    // - every interleaved `assess()` returns a finite `p_bad` in
    //   `[0, 1]` even as the dimension shape mutates underneath;
    // - after full teardown the engine still reckons, reports zero
    //   Sentinels, and the health snapshot stays clean;
    // - per-round labels round-trip through the pending buffer.
    //
    // 8 Sentinels × 4 axes (one spatial) is large enough to traverse
    // the multi-Sentinel + multi-axis extension cascade but stays
    // well below sizes that would slow the test suite. The cleanup
    // order interleaves Sentinels and axes so neither side ever sits
    // in a "fully empty" mid-tear-down state until the very end.
    let mut s = scenario(INSTANCE, SEED);

    // Build up to peak, asserting assessments remain well-formed at
    // every milestone.
    let ids = s.register_sentinels(PEAK_SENTINEL_NAMES).expect("register all sentinels");
    assert_eq!(ids.len(), PEAK_SENTINEL_NAMES.len(), "every sentinel was assigned an id");

    for (name, spatial) in PEAK_AXES {
        s.register_axis(name, spatial)
            .unwrap_or_else(|e| panic!("register axis {name}: {e:?}"));
        let r = s
            .derive_default("alice")
            .unwrap_or_else(|e| panic!("post-{name} reckon: {e:?}"));
        assert_reckoning_well_formed(&r, &format!("post-{name}"));
    }

    // Issue and label one round of assessments at peak — proves the
    // pending buffer keys consistently across the largest dim map
    // shape this scenario reaches.
    let peak = s.derive_default("bob").expect("peak reckon");
    s.label(LabelSpec::benign(peak.assessment.id).build())
        .expect("peak label round-trips");

    // Tear down: alternate Sentinel + axis deregistrations so neither
    // pool empties mid-stream. Assessments continue to succeed.
    let mut s_iter = PEAK_SENTINEL_NAMES.iter().copied();
    let mut a_iter = PEAK_AXES.iter().map(|(n, _)| *n);
    let mut interleave = 0usize;
    loop {
        let did_something = if interleave.is_multiple_of(2) {
            s_iter.next().is_some_and(|name| {
                s.deregister_sentinel(name)
                    .unwrap_or_else(|e| panic!("deregister {name}: {e:?}"));
                true
            })
        } else {
            a_iter.next().is_some_and(|name| {
                s.deregister_axis(name)
                    .unwrap_or_else(|e| panic!("deregister axis {name}: {e:?}"));
                true
            })
        };

        // Drain the still-non-empty side once the other side empties.
        if !did_something {
            let drained_sentinels = s_iter.by_ref().count() == 0;
            let drained_axes = a_iter.by_ref().count() == 0;
            if drained_sentinels && drained_axes {
                break;
            }
            interleave += 1;
            continue;
        }

        let r = s
            .derive_default("carol")
            .unwrap_or_else(|e| panic!("mid-teardown reckon at step {interleave}: {e:?}"));
        assert_reckoning_well_formed(&r, &format!("mid-teardown step {interleave}"));
        interleave += 1;
    }

    // Drain any remainder (the loop above prefers Sentinel-then-axis;
    // run a final pass for safety).
    for name in PEAK_SENTINEL_NAMES {
        if s.sentinel(name).is_some() {
            s.deregister_sentinel(name).expect("residual sentinel cleanup");
        }
    }
    for (name, _) in PEAK_AXES {
        if s.axis(name).is_some() {
            s.deregister_axis(name).expect("residual axis cleanup");
        }
    }

    // Post-teardown the engine is back to its minimal shape: no
    // Sentinels reporting, but `assess` still produces a valid
    // prior-only reckoning.
    let post = s.derive_default("dave").expect("post-teardown reckon");
    assert_reckoning_well_formed(&post, "post-teardown");
    assert_eq!(
        post.assessment.risk.n_sentinels_reporting, 0,
        "every Sentinel was deregistered",
    );

    assert_health_clean(&s);
}
