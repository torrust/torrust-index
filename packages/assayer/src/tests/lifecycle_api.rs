// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`register_sentinel_succeeds`] | lifespan | A Sentinel can be registered against an engine that is already built and serving, with no pause or restart in between. Structure is meant to change while the system is live, so the ordinary case has to be the cheap one. |
//! | [`register_sentinel_duplicate_error`] | lifespan | A second registration under an id that is already live is refused, and the refusal names the kind of entity that collided rather than only reporting that something did. The first registration is left untouched, so a caller retrying blindly cannot destroy the slot it meant to create. |
//! | [`deregister_sentinel_succeeds`] | lifespan | Retiring a registered Sentinel succeeds against a running engine, the mirror of registration. Hosts lose Sentinels for ordinary reasons — a probe is decommissioned, a machine goes away — and that must not be an outage. |
//! | [`deregister_sentinel_not_found`] | lifespan | Retiring an id the engine never held is an error naming both the kind and the missing id, not a silent success. A caller that believed it tore something down when it did not would leave the real entity live and unattended. |
//! | [`register_sentinel_then_assess_and_derive_sees_zero_features`] | lifespan | An assessment issued between a Sentinel's registration and its first report still produces a finite probability: the new slot contributes zeros rather than a gap. Registration is visible to assessment immediately, so the window before the first report has to be a well-defined state rather than an error. |
//! | [`deregister_sentinel_then_assess_and_derive_excludes`] | lifespan | cites (´claim:lifespan:an-assessment-with-no-sentinel-registered-flags-itself-rather-than-refusing-to-answer´) |
//! | [`report_rejected_after_deregistration`] | lifespan | A Sentinel that has been retired can no longer file reports; its next report is rejected as coming from an unknown Sentinel. Slot removal is synchronous with deregistration, so there is no window in which a departed Sentinel keeps writing into state nothing will read. |
//! | [`report_accepted_after_registration`] | lifespan | A report filed immediately after registration returns is accepted, without waiting for the model to be widened. Registration makes the slot visible synchronously while the model extension proceeds behind it, so a Sentinel need not sleep before it starts speaking. |
//! | [`register_outcome_axis_succeeds`] | lifespan | A new outcome axis is accepted by an engine already in service. Axes represent the consequences a host has learned to care about, and those arrive over the lifetime of a deployment rather than all at build time. |
//! | [`register_outcome_axis_duplicate_error`] | lifespan | cites (´claim:lifespan:a-duplicate-registration-is-refused-and-names-the-entity-kind-that-collided´) |
//! | [`deregister_outcome_axis_not_found`] | lifespan | cites (´claim:lifespan:retiring-an-identifier-that-was-never-registered-is-refused-as-not-found´) |
//! | [`deregister_outcome_axis_succeeds`] | lifespan | An axis whose registration has been processed and published can then be retired again, completing the pair. The flush in between means the teardown path acts on an axis the model actually holds rather than on a command still in flight. |
//! | [`register_identity_dimension_succeeds`] | lifespan | An identity dimension registers successfully against a running engine. Unlike the other registrations this one blocks until the identity- maintenance thread acknowledges the new dimension, so a caller holding an accepted result knows the machinery behind the dimension exists rather than merely having been asked for. |
//! | [`register_identity_dimension_duplicate_error`] | lifespan | cites (´claim:lifespan:a-duplicate-registration-is-refused-and-names-the-entity-kind-that-collided´) |
//! | [`register_second_identity_dimension_succeeds`] | lifespan | A second identity dimension registers alongside the first rather than displacing it. Entities are identified along several independent axes at once — address, agent, account — and the engine is built to hold them together. |
//! | [`deregister_identity_dimension_succeeds`] | lifespan | Retiring an identity dimension succeeds on a running engine. The call waits for the model owner to marginalise the dimension's features and publish a snapshot without them before the infrastructure is torn down, so no assessment can read through to machinery that has already gone. |
//! | [`deregister_identity_dimension_not_found`] | lifespan | cites (´claim:lifespan:retiring-an-identifier-that-was-never-registered-is-refused-as-not-found´) |
//! | [`pre_seed_empty_entries`] | lifespan | Pre-seeding an empty batch succeeds and reports nothing processed. A host assembling its history from a source that turned out to be empty gets an ordinary answer rather than having to guard the call. |
//! | [`pre_seed_processes_entries`] | lifespan | A pre-seed batch reports exactly as many entries processed as it was given. That count is the host's only receipt for work done inside the engine, so it has to be a tally rather than an estimate. |
//! | [`pre_seed_before_sentinel_registration`] | lifespan | Pre-seeding succeeds with no Sentinel registered at all: the synthesis reads the registry and finds nothing to fill a slot for, so the entry carries no slots rather than fabricated ones. Historical labels usually predate the sensing apparatus that will later be pointed at the same traffic, so seeding must not depend on that apparatus already standing. |
//! | [`pre_seed_entry_synthesised_against_schema_and_registry`] | lifespan | The synthetic pre-seed entry is built against the declared schema and the registered Sentinels, as the pre-seeding record's simplified arrangement requires of the synthesis that reads them: every registered Sentinel holds an occupied slot with no features — a Sentinel active and silent, which is what reconstruction source four reads — the entry's own key is encoded on each registered identity dimension, and the signal block spans the declared schema's width. The audit measured every one of these blocks empty, so a pre-seeded label trained on a bias and nothing else while the type's own documentation asserted the computation that did not happen. |
//! | [`lifecycle_error_display`] | lifespan | Each lifecycle failure renders to a non-empty message — collisions, missing entities, both shutdown flavours, channel exhaustion and empty names alike. These errors surface in operator logs at the moment a structural change fails, which is exactly when a bare variant name would be least useful. |

//! Lifecycle orchestration acceptance tests.
//!
//! Tests the six public lifecycle methods (`register_sentinel`,
//! `deregister_sentinel`, `register_outcome_axis`,
//! `deregister_outcome_axis`, `register_identity_dimension`,
//! `deregister_identity_dimension`) and `pre_seed()` through the
//! full public API on a running `Assayer`.
//!
//! These tests drive the engine through the
//! [`World`](crate::testing::World) harness; verbs that the harness
//! does not yet wrap (outcome-axis and identity-dimension lifecycle,
//! pre-seeding, label-channel flush, raw reports) go through
//! [`World::assayer`](crate::testing::World::assayer) as a narrow
//! escape hatch.
//!
//! # Cross-References
//!
//! - (´dec:construction:six-methods´) — the six registration and deregistration methods driven here

#![allow(clippy::items_after_statements)]

use super::helpers::test_report;
use crate::error::{LifecycleError, ReportError};
use crate::owner::commands::{IdentityDimensionRegistration, OutcomeAxisRegistration, SentinelRegistration};
use crate::resonance::channel::{Action, ChannelPolicy, RewardParameters};
use crate::testing::{PreSeedSpec, World};
use crate::types::{ChannelId, DimensionId, IdentityBudget, OutcomeAxisId, OutcomeEligibility, SentinelId, SpatialFeaturePolicy};

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Default channel name used by every test in this module.
const CHANNEL: &str = "test";

fn default_policy() -> ChannelPolicy {
    ChannelPolicy {
        actions: vec![Action::Allow, Action::Challenge, Action::Block],
        reward: RewardParameters::default(),
        ..ChannelPolicy::default()
    }
}

/// Build a minimal [`World`] with one channel (assigned `ChannelId(0)`).
fn build_test_world() -> World {
    let config = super::helpers::test_config_with_id("lifecycle-test");
    World::builder(config)
        .channel(CHANNEL, default_policy())
        .seed(0x11CE_C7CE_5EED_1234)
        .build()
        .expect("test world build should succeed")
}

fn test_sentinel_reg(id: u32, name: &str) -> SentinelRegistration {
    SentinelRegistration {
        id: SentinelId(id),
        name: name.to_owned(),
    }
}

fn test_axis_reg(id: u32, name: &str) -> OutcomeAxisRegistration {
    OutcomeAxisRegistration {
        id: OutcomeAxisId(id),
        name: name.to_owned(),
        description: String::new(),
        eligibility: OutcomeEligibility::AllLabels,
        initial_kappa: 1.0,
        gamma: 0.999,
        spatial_features: SpatialFeaturePolicy::Disabled,
    }
}

fn test_identity_reg(id: u32, name: &str) -> IdentityDimensionRegistration {
    IdentityDimensionRegistration {
        id: DimensionId(id),
        name: name.to_owned(),
        description: "lifecycle API test hierarchy".to_owned(),
        coordinate_semantics: "the leading bytes select a prefix group".to_owned(),
        domain_bits: 128,
        depth_cutoff: 10,
        budget: IdentityBudget::for_depth_cutoff(10),
        encode: |entity| {
            let bytes = entity.as_bytes();
            if bytes.len() >= 8 {
                u128::from(u64::from_le_bytes(bytes[..8].try_into().unwrap_or_default()))
            } else {
                0
            }
        },
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Sentinel Lifecycle
// ═══════════════════════════════════════════════════════════════════════════════

/// A Sentinel can be registered against an engine that is already built and
/// serving, with no pause or restart in between. Structure is meant to change
/// while the system is live, so the ordinary case has to be the cheap one.
///
/// ´claim:lifespan:registering-a-sentinel-on-a-running-engine-succeeds-without-quiescing-it´
/// ´test:crate:register-sentinel-succeeds´
#[test]
fn register_sentinel_succeeds() {
    let mut world = build_test_world();
    let result = world.register_sentinel("sentinel-A");
    assert!(result.is_ok(), "register_sentinel should succeed: {result:?}");
}

/// A second registration under an id that is already live is refused, and the
/// refusal names the kind of entity that collided rather than only reporting
/// that something did. The first registration is left untouched, so a caller
/// retrying blindly cannot destroy the slot it meant to create.
///
/// ´claim:lifespan:a-duplicate-registration-is-refused-and-names-the-entity-kind-that-collided´
/// ´test:crate:register-sentinel-duplicate-error´
#[test]
fn register_sentinel_duplicate_error() {
    // The `World::register_sentinel` verb allocates a fresh ID per call,
    // so `DuplicateId` can only be triggered through the raw assayer
    // surface. Use the escape hatch for this boundary case.
    let world = build_test_world();
    world.assayer().register_sentinel(test_sentinel_reg(1, "sentinel-A")).unwrap();

    let result = world.assayer().register_sentinel(test_sentinel_reg(1, "sentinel-A-dup"));
    assert!(
        matches!(
            &result,
            Err(LifecycleError::DuplicateId {
                entity_type: "Sentinel",
                ..
            })
        ),
        "expected DuplicateId, got: {result:?}"
    );
}

/// Retiring a registered Sentinel succeeds against a running engine, the mirror
/// of registration. Hosts lose Sentinels for ordinary reasons — a probe is
/// decommissioned, a machine goes away — and that must not be an outage.
///
/// ´claim:lifespan:a-live-sentinel-can-be-retired-by-name-while-the-engine-runs´
/// ´test:crate:deregister-sentinel-succeeds´
#[test]
fn deregister_sentinel_succeeds() {
    let mut world = build_test_world();
    world.register_sentinel("sentinel-A").unwrap();

    let result = world.deregister_sentinel("sentinel-A");
    assert!(result.is_ok(), "deregister_sentinel should succeed: {result:?}");
}

/// Retiring an id the engine never held is an error naming both the kind and
/// the missing id, not a silent success. A caller that believed it tore
/// something down when it did not would leave the real entity live and
/// unattended.
///
/// ´claim:lifespan:retiring-an-identifier-that-was-never-registered-is-refused-as-not-found´
/// ´test:crate:deregister-sentinel-not-found´
#[test]
fn deregister_sentinel_not_found() {
    // Bypass the name registry to probe the engine's `NotFound` path.
    let world = build_test_world();
    let result = world.assayer().deregister_sentinel(SentinelId(999));
    assert!(
        matches!(
            &result,
            Err(LifecycleError::NotFound {
                entity_type: "Sentinel",
                ..
            })
        ),
        "expected NotFound, got: {result:?}"
    );
}

/// An assessment issued between a Sentinel's registration and its first report
/// still produces a finite probability: the new slot contributes zeros rather
/// than a gap. Registration is visible to assessment immediately, so the window
/// before the first report has to be a well-defined state rather than an error.
///
/// ´claim:lifespan:a-sentinel-that-has-reported-nothing-yet-contributes-zeros-rather-than-blocking-assessment´
/// ´test:crate:register-sentinel-then-assess-and-derive-sees-zero-features´
#[test]
fn register_sentinel_then_assess_and_derive_sees_zero_features() {
    let mut world = build_test_world();
    world.register_sentinel("sentinel-A").unwrap();

    let reckoning = world.derive_for_request(world.request(CHANNEL, "e1")).unwrap();
    // With a newly registered Sentinel (no report), features are zero
    assert!(reckoning.assessment.risk.p_bad.is_finite(), "p_bad should be finite");
}

/// Once the only registered Sentinel is retired, the next assessment carries
/// the zero-Sentinels flag again: the engine still answers, but says the answer
/// rests on no Sentinel evidence. The flag tracks the present registration
/// state, so it comes back on when the fleet empties as readily as it went off
/// when the fleet filled.
///
/// (´claim:lifespan:an-assessment-with-no-sentinel-registered-flags-itself-rather-than-refusing-to-answer´)
/// ´test:crate:deregister-sentinel-then-assess-and-derive-excludes´
#[test]
fn deregister_sentinel_then_assess_and_derive_excludes() {
    let mut world = build_test_world();
    world.register_sentinel("sentinel-A").unwrap();
    world.deregister_sentinel("sentinel-A").unwrap();

    let reckoning = world.derive_for_request(world.request(CHANNEL, "e1")).unwrap();

    // After deregistration, the Sentinel is excluded from assessments.
    assert!(
        reckoning.assessment.health.zero_sentinels,
        "should have zero sentinels after deregistration"
    );
}

/// A Sentinel that has been retired can no longer file reports; its next report
/// is rejected as coming from an unknown Sentinel. Slot removal is synchronous
/// with deregistration, so there is no window in which a departed Sentinel
/// keeps writing into state nothing will read.
///
/// ´claim:lifespan:a-report-from-a-retired-sentinel-is-rejected-as-coming-from-an-unknown-sentinel´
/// ´test:crate:report-rejected-after-deregistration´
#[test]
fn report_rejected_after_deregistration() {
    let mut world = build_test_world();
    let id = world.register_sentinel("sentinel-A").unwrap();
    world.deregister_sentinel("sentinel-A").unwrap();

    let report = test_report(&[]);
    let result = world.assayer().receive_sentinel_report(id, report);
    assert!(
        matches!(&result, Err(ReportError::UnknownSentinel { .. })),
        "expected UnknownSentinel after deregistration, got: {result:?}"
    );
}

/// A report filed immediately after registration returns is accepted, without
/// waiting for the model to be widened. Registration makes the slot visible
/// synchronously while the model extension proceeds behind it, so a Sentinel
/// need not sleep before it starts speaking.
///
/// ´claim:lifespan:reports-are-accepted-from-the-moment-registration-returns´
/// ´test:crate:report-accepted-after-registration´
#[test]
fn report_accepted_after_registration() {
    let mut world = build_test_world();
    let id = world.register_sentinel("sentinel-A").unwrap();

    let report = test_report(&[]);
    let result = world.assayer().receive_sentinel_report(id, report);
    assert!(result.is_ok(), "report should be accepted after registration: {result:?}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Outcome Axis Lifecycle
// ═══════════════════════════════════════════════════════════════════════════════

/// A new outcome axis is accepted by an engine already in service. Axes
/// represent the consequences a host has learned to care about, and those
/// arrive over the lifetime of a deployment rather than all at build time.
///
/// ´claim:lifespan:an-outcome-axis-can-be-added-to-a-running-engine´
/// ´test:crate:register-outcome-axis-succeeds´
#[test]
fn register_outcome_axis_succeeds() {
    let world = build_test_world();
    let result = world.assayer().register_outcome_axis(test_axis_reg(1, "fraud"));
    assert!(result.is_ok(), "register_outcome_axis should succeed: {result:?}");
}

/// The axis uniqueness guard holds even once the first registration has been
/// flushed all the way through the model owner and published: a second axis on
/// the same id is still refused. The guard is not merely a race window on an
/// in-flight command, it is the standing state of the engine.
///
/// (´claim:lifespan:a-duplicate-registration-is-refused-and-names-the-entity-kind-that-collided´)
/// ´test:crate:register-outcome-axis-duplicate-error´
#[test]
fn register_outcome_axis_duplicate_error() {
    let world = build_test_world();
    world.assayer().register_outcome_axis(test_axis_reg(1, "fraud")).unwrap();

    // Flush to ensure the model owner processes the registration and
    // publishes a new snapshot containing the axis.
    world.assayer().flush_label_channel().unwrap();

    let result = world.assayer().register_outcome_axis(test_axis_reg(1, "fraud-dup"));
    assert!(
        matches!(
            &result,
            Err(LifecycleError::DuplicateId {
                entity_type: "OutcomeAxis",
                ..
            })
        ),
        "expected DuplicateId, got: {result:?}"
    );
}

/// An axis id the engine never held cannot be retired, and the refusal names
/// the outcome-axis kind. Axis ids key per-axis model state and ledger rows, so
/// a mistaken teardown has to be reported rather than absorbed.
///
/// (´claim:lifespan:retiring-an-identifier-that-was-never-registered-is-refused-as-not-found´)
/// ´test:crate:deregister-outcome-axis-not-found´
#[test]
fn deregister_outcome_axis_not_found() {
    let world = build_test_world();
    let result = world.assayer().deregister_outcome_axis(OutcomeAxisId(999));
    assert!(
        matches!(
            &result,
            Err(LifecycleError::NotFound {
                entity_type: "OutcomeAxis",
                ..
            })
        ),
        "expected NotFound, got: {result:?}"
    );
}

/// An axis whose registration has been processed and published can then be
/// retired again, completing the pair. The flush in between means the teardown
/// path acts on an axis the model actually holds rather than on a command still
/// in flight.
///
/// ´claim:lifespan:an-axis-that-the-model-owner-has-published-can-then-be-retired´
/// ´test:crate:deregister-outcome-axis-succeeds´
#[test]
fn deregister_outcome_axis_succeeds() {
    let world = build_test_world();
    world.assayer().register_outcome_axis(test_axis_reg(1, "fraud")).unwrap();

    // Flush to ensure the model owner processes and publishes the axis.
    world.assayer().flush_label_channel().unwrap();

    let result = world.assayer().deregister_outcome_axis(OutcomeAxisId(1));
    assert!(result.is_ok(), "deregister_outcome_axis should succeed: {result:?}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Identity Dimension Lifecycle
// ═══════════════════════════════════════════════════════════════════════════════

/// An identity dimension registers successfully against a running engine.
/// Unlike the other registrations this one blocks until the identity-
/// maintenance thread acknowledges the new dimension, so a caller holding an
/// accepted result knows the machinery behind the dimension exists rather than
/// merely having been asked for.
///
/// ´claim:lifespan:registering-an-identity-dimension-returns-only-once-the-maintenance-thread-has-acknowledged-it´
/// ´test:crate:register-identity-dimension-succeeds´
#[test]
fn register_identity_dimension_succeeds() {
    let world = build_test_world();
    let result = world.assayer().register_identity_dimension(test_identity_reg(1, "ip-hash"));
    assert!(result.is_ok(), "register_identity_dimension should succeed: {result:?}");
}

/// Identity dimensions are subject to the same uniqueness guard as the other
/// entity kinds, and the collision is reported against the identity-dimension
/// kind. Its duplicate check is taken twice — once cheaply, once again under
/// the write lock after the blocking acknowledgement — because that wait leaves
/// room for another caller to slip in.
///
/// (´claim:lifespan:a-duplicate-registration-is-refused-and-names-the-entity-kind-that-collided´)
/// ´test:crate:register-identity-dimension-duplicate-error´
#[test]
fn register_identity_dimension_duplicate_error() {
    let world = build_test_world();
    world
        .assayer()
        .register_identity_dimension(test_identity_reg(1, "ip-hash"))
        .unwrap();

    let result = world
        .assayer()
        .register_identity_dimension(test_identity_reg(1, "ip-hash-dup"));
    assert!(
        matches!(
            &result,
            Err(LifecycleError::DuplicateId {
                entity_type: "IdentityDimension",
                ..
            })
        ),
        "expected DuplicateId, got: {result:?}"
    );
}

/// A second identity dimension registers alongside the first rather than
/// displacing it. Entities are identified along several independent axes at
/// once — address, agent, account — and the engine is built to hold them
/// together.
///
/// ´claim:lifespan:more-than-one-identity-dimension-can-be-live-at-once´
/// ´test:crate:register-second-identity-dimension-succeeds´
#[test]
fn register_second_identity_dimension_succeeds() {
    let world = build_test_world();
    world
        .assayer()
        .register_identity_dimension(test_identity_reg(1, "ip-hash"))
        .unwrap();

    let result = world.assayer().register_identity_dimension(test_identity_reg(2, "ua-hash"));
    assert!(result.is_ok(), "second identity dimension should succeed: {result:?}");
}

/// Retiring an identity dimension succeeds on a running engine. The call waits
/// for the model owner to marginalise the dimension's features and publish a
/// snapshot without them before the infrastructure is torn down, so no
/// assessment can read through to machinery that has already gone.
///
/// ´claim:lifespan:an-identity-dimension-is-retired-only-after-the-model-has-stopped-referencing-it´
/// ´test:crate:deregister-identity-dimension-succeeds´
#[test]
fn deregister_identity_dimension_succeeds() {
    let world = build_test_world();
    world
        .assayer()
        .register_identity_dimension(test_identity_reg(1, "ip-hash"))
        .unwrap();

    let result = world.assayer().deregister_identity_dimension(DimensionId(1));
    assert!(result.is_ok(), "deregister_identity_dimension should succeed: {result:?}");
}

/// A dimension id that was never registered cannot be retired, and the refusal
/// names the identity-dimension kind. The check happens before any command is
/// sent, so a mistaken teardown costs the model owner nothing.
///
/// (´claim:lifespan:retiring-an-identifier-that-was-never-registered-is-refused-as-not-found´)
/// ´test:crate:deregister-identity-dimension-not-found´
#[test]
fn deregister_identity_dimension_not_found() {
    let world = build_test_world();
    let result = world.assayer().deregister_identity_dimension(DimensionId(999));
    assert!(
        matches!(
            &result,
            Err(LifecycleError::NotFound {
                entity_type: "IdentityDimension",
                ..
            })
        ),
        "expected NotFound, got: {result:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pre-seeding
// ═══════════════════════════════════════════════════════════════════════════════

/// Pre-seeding an empty batch succeeds and reports nothing processed. A host
/// assembling its history from a source that turned out to be empty gets an
/// ordinary answer rather than having to guard the call.
///
/// ´claim:lifespan:pre-seeding-with-nothing-to-seed-succeeds-and-reports-nothing-processed´
/// ´test:crate:pre-seed-empty-entries´
#[test]
fn pre_seed_empty_entries() {
    let world = build_test_world();
    let result = world.assayer().pre_seed(&[]);
    assert!(result.is_ok(), "pre_seed with empty entries should succeed: {result:?}");
    assert_eq!(result.unwrap().processed, 0);
}

/// A pre-seed batch reports exactly as many entries processed as it was given.
/// That count is the host's only receipt for work done inside the engine, so it
/// has to be a tally rather than an estimate.
///
/// ´claim:lifespan:every-pre-seeded-entry-is-accounted-for-in-the-processed-count´
/// ´test:crate:pre-seed-processes-entries´
#[test]
fn pre_seed_processes_entries() {
    let world = build_test_world();

    let entries: Vec<_> = (0..10)
        .map(|i| {
            let entity = World::entity(&format!("e{i}"));
            let mut spec = PreSeedSpec::new(ChannelId(0), entity).ground_truth();
            if i % 5 == 0 {
                spec = spec.valence(1.0);
            }
            spec.build()
        })
        .collect();

    let result = world.assayer().pre_seed(&entries);
    assert!(result.is_ok(), "pre_seed should succeed: {result:?}");
    assert_eq!(result.unwrap().processed, 10);
}

/// Pre-seeding succeeds with no Sentinel registered at all: the synthesis
/// reads the registry and finds nothing to fill a slot for, so the entry
/// carries no slots rather than fabricated ones. Historical labels usually
/// predate the sensing apparatus that will later be pointed at the same
/// traffic, so seeding must not depend on that apparatus already standing.
///
/// ´claim:lifespan:history-can-be-loaded-before-any-sentinel-exists´
/// ´test:crate:pre-seed-before-sentinel-registration´
#[test]
fn pre_seed_before_sentinel_registration() {
    let world = build_test_world();

    // Pre-seed without any Sentinels registered → zero Sentinel features
    let entries = vec![
        PreSeedSpec::new(ChannelId(0), World::entity("e1"))
            .valence(1.0)
            .ground_truth()
            .build(),
    ];

    let result = world.assayer().pre_seed(&entries);
    assert!(
        result.is_ok(),
        "pre_seed before sentinel registration should succeed: {result:?}"
    );
    assert_eq!(result.unwrap().processed, 1);
}

/// The synthetic pre-seed entry is built against the declared schema and
/// the registered Sentinels, as the pre-seeding record's simplified
/// arrangement requires of the synthesis that reads them: every
/// registered Sentinel holds an occupied slot with no features — a
/// Sentinel active and silent, which is what reconstruction source four
/// reads — the entry's own key is encoded on each registered identity
/// dimension, and the signal block spans the declared schema's width. The
/// audit measured every one of these blocks empty, so a pre-seeded label
/// trained on a bias and nothing else while the type's own documentation
/// asserted the computation that did not happen.
///
/// ´claim:lifespan:the-synthetic-pre-seed-entry-is-built-against-the-schema-and-the-registry´
/// ´test:crate:pre-seed-entry-synthesised-against-schema-and-registry´
#[test]
fn pre_seed_entry_synthesised_against_schema_and_registry() {
    use crate::signal::{Persistence, SignalDeclaration, SignalShape};

    // A world with one declared signal, one registered Sentinel and one
    // registered identity dimension.
    let config = super::helpers::test_config_with_id("preseed-synthesis");
    let world = World::builder(config)
        .channel(CHANNEL, default_policy())
        .signal_schema(vec![SignalDeclaration::new(
            "reputation",
            SignalShape::Scalar { clip: (0.0, 1.0) },
            Persistence::Entity,
        )])
        .seed(0x11CE_C7CE_5EED_5EED)
        .build()
        .expect("test world build should succeed");
    world.assayer().register_sentinel(test_sentinel_reg(7, "s7")).unwrap();
    world
        .assayer()
        .register_identity_dimension(test_identity_reg(3, "ip-hash"))
        .unwrap();

    let assayer = world.assayer();
    let entity = World::entity("seeded");
    let snapshot = assayer.shared().published.load();
    let sig_len = snapshot.dimension_map.sig_range.len();
    assert_eq!(sig_len, 1, "the declared scalar signal occupies one position");

    let pending = crate::assessment::synthesise_preseed_pending(crate::types::AssessmentId(41), &entity, assayer, sig_len, &[]);

    // An occupied, zero-featured slot per registered Sentinel: active
    // and silent.
    assert_eq!(pending.active_sentinels, vec![SentinelId(7)]);
    let slot = pending.sentinel_extractions.get(&SentinelId(7)).expect("slot present");
    assert!(slot.occupancy, "the slot is occupied");
    assert_eq!(slot.features.len(), 0, "the slot's features are zero-width: silent");
    assert!(pending.reporting_sentinels.is_empty(), "nothing reported at seed time");

    // The entry's own key on the registered identity dimension.
    use crate::assessment::AssessmentSharedState as _;
    let dim = crate::types::DimensionId(3);
    let expected_coord = assayer.encode_for_dimension(dim, &entity).expect("dimension registered");
    assert_eq!(pending.identity_coordinates.get(&dim), Some(&expected_coord));

    // The signal block spans the declared schema.
    assert_eq!(pending.signal_features.len(), sig_len);
}

// ═══════════════════════════════════════════════════════════════════════════════
// LifecycleError Display
// ═══════════════════════════════════════════════════════════════════════════════

/// Each lifecycle failure renders to a non-empty message — collisions, missing
/// entities, both shutdown flavours, channel exhaustion and empty names alike.
/// These errors surface in operator logs at the moment a structural change
/// fails, which is exactly when a bare variant name would be least useful.
///
/// ´claim:lifespan:every-lifecycle-failure-carries-a-human-readable-message´
/// ´test:crate:lifecycle-error-display´
#[test]
fn lifecycle_error_display() {
    let errors: Vec<LifecycleError> = vec![
        LifecycleError::DuplicateId {
            entity_type: "Sentinel",
            id: "42".to_owned(),
        },
        LifecycleError::NotFound {
            entity_type: "OutcomeAxis",
            id: "7".to_owned(),
        },
        LifecycleError::ModelOwnerShutdown,
        LifecycleError::IdentityMaintenanceShutdown,
        LifecycleError::CommandChannelFull,
        LifecycleError::EmptyName { entity_type: "Sentinel" },
    ];

    for err in &errors {
        let msg = format!("{err}");
        assert!(!msg.is_empty(), "Display for {err:?} should be non-empty");
    }
}
