// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`request_context_builder`] | assess | A request begins as its entity and nothing else: the coordinate map and the signal map both start empty, and the entity's bytes come through unaltered. Nothing is inferred on the host's behalf, so an attribute absent from a request is absent from the assessment rather than silently defaulted to something the host never said. |
//! | [`request_context_with_signals`] | assess | Signals attach to a request by name and keep the value the host supplied — a bare number becomes a numeric signal of exactly that magnitude, negative values included. Nothing is rescaled or clamped at attachment time, so whatever encoding the schema later applies is applied to the host's own number and can be reasoned about from it. |
//! | [`request_context_with_sentinels`] | assess | One request can address several Sentinels at once, each under its own identifier, and each coordinate survives the full width of the hash space untruncated. An entity sits at a different place in every measurement space, so the coordinates must be kept apart and kept whole — a narrowed coordinate would point at the wrong cell in that Sentinel's tree. |
//! | [`request_context_combined`] | assess | Coordinates and signals live in separate stores, so a host may interleave the two kinds of builder call freely and end with both counts intact. Real call sites assemble a request from several sources in whatever order the surrounding code makes convenient; a builder where ordering mattered would drop attributes depending on how the caller happened to be written. |
//! | [`assessment_shared_state_trait_is_object_safe`] | assess | The shared-state interface can be named behind a trait object, which the compiler will only accept if every method on it is dispatchable dynamically. That keeps the interface within reach of a host that chooses its state implementation at run time or stores several behind one type, rather than confining it to callers that can monomorphise. |
//! | [`request_labels_public_api_returns_pending_candidate`] | assess | Asking for labels returns candidates drawn from assessments that are genuinely still pending: the assessment just made comes back by its own identifier, carrying a finite score. The call yields the requests directly rather than a fallible result, because having nothing worth labelling is an ordinary answer — an empty budget's worth of candidates — and not a failure the host must handle. |
//! | [`assess_batch_matches_forward_and_reverse_singletons`] | assess | Requests submitted together are assessed independently: each one's risk and resonance match what it would have received alone, whichever order the singletons were run in, and neither borrows the other's payload — the Sentinel alarm belongs to the request that supplied the coordinate, and the signal degradation counts stay with the request whose signals were malformed. Batching is therefore purely an efficiency choice for the host, with no bearing on what any individual request is told. |
//! | [`derived_output_types_are_send_sync`] | assess | Every type the public surface hands back can cross a thread boundary and be shared across threads — the whole assessment, the risk basis, the health snapshot, and the label-guidance types alike. A host assesses on its request threads and typically logs, decides, or reports elsewhere; an output tied to the thread that produced it would force a copy into some parallel type at every such handoff. |
//! | [`derived_output_types_are_clone`] | assess | Outputs are values a host can duplicate at will, not borrowed views onto engine state. Because a copy is independent of anything the engine holds, a host may retain an assessment for audit, hand a second copy to a decision path, and keep both long after the model has moved on. |
//! | [`derived_output_types_are_debug`] | assess | Every output type can be rendered for diagnostics without the host writing a formatter of its own. A risk estimate that cannot be printed cannot be put in a log line or an assertion message, and the moment a host most needs to see one is the moment something has gone wrong. |
//! | [`concurrent_assess_under_lifecycle_change_returns_valid_assessments`] | assess | Registering and deregistering a Sentinel is safe to do while the engine is serving traffic: a thousand assessments running against four threads while a fifth cycles a Sentinel in and out each come back with a probability that is finite and inside the unit interval, and the engine's health afterwards records no sanitisation, no model fallback, and no dropped health event. What the health may record is the honest trace of the contention itself — batch-init observations skipped on the contended warm-up lock, now reported in band rather than silent — and every degraded assessment must be accounted for by exactly that trace. Changing the measurement surface is an operational act a host performs on a live system, so it must not be payable in corrupted estimates for whatever requests happen to be in flight. |
//! | [`lifecycle_churn_around_cholesky_recompute_keeps_fast_path_valid`] | assess | The periodic Cholesky recompute is the riskiest moment in the learner's life — the factorisation the fast path reads is rebuilt underneath it — and it survives lifecycle churn crossing it. A label stream driven past the configured cadence while Sentinels register and deregister leaves recomputes actually observed, every racing assessment finite, no Schur correction skipped, no health event dropped, and the cascade terminus untripped. The recompute is not something a host can schedule for a quiet moment, so it has to be correct during a busy one. |
//! | [`standardisation_transition_reported_on_both_surfaces`] | assess | The standardisation transition is reported on both surfaces and each reading carries its coordinate join key. The compact reading riding an assessment is retrospective while the full report is current, so they may legitimately differ during an advance; whenever their versions agree, their phase and accepted count agree too. Before the first accepted observation both name the same `WaitingForInit` state at zero, and at the horizon both name the same `InService` state at the horizon's count. Each accepted advance publishes a later snapshot version, and a full-report version never names two different phase/count states. The full pair is read from the published snapshot rather than from the last label-time summary, which is what makes this true of a deployment that has not labelled anything yet — the whole of a cold start, and precisely when an operator is watching. |

//! Integration tests for `torrust_assayer` assess/derive public API types.
//!
//! # Cross-References
//!
//! - (´sec:assayer:testing-plan-standing´) — what the testing plan covers and where integration tests sit in it
//! - (´dec:ordering:assessment-function´) — assessment is one function, not a stage pipeline
//! - (´chap:spec:assessment-interface´) — core assessment output
//!
//! # Note
//!
//! Full assessment pipeline tests that require `ModelSnapshot` are located in
//! `src/tests/assess_pipeline.rs` (crate-level tests) since `ModelSnapshot`
//! is not publicly exported. These tests verify the public API surface.

use torrust_assayer::testing::{
    GOLDEN_COORD, LabelSpec, PURITY_DRIFT, Scenario, assert_core_and_resonance_profile_near_with_drift, assert_health_clean,
    assert_reckoning_well_formed, assert_signal_degradation_counts, attach_golden_reporting_sentinel, scenario, scenario_with,
    scenario_with_config, score_verified_request_schema, with_degraded_score_verified, with_standard_final_score_verified,
};
use torrust_assayer::types::{EntityKey, SentinelId};
use torrust_assayer::{
    Assayer, AssayerConfig, ChannelPolicy, LabelBudget, LabelGuidanceParams, RequestContext, SentinelRegistration, SignalValue,
    StandardisationPhase,
};

/// Build a minimal [`Scenario`] with a single named `"default"` channel.
///
/// Assess-integration tests only need a channel name resolvable by
/// [`World::request`]; no signal schema or sentinels are required.
fn build_world() -> Scenario {
    scenario("assess-integration", 0x0042)
}

/// Build the rich public fixture used by the batch-independence witness, with
/// its cold standardisation ramp already at the horizon.
///
/// Settling is what makes the witness's comparison a comparison of the batch
/// path. Below the horizon the published standardisation moments are a mixture
/// of the class priors and the accepted sample
/// (´alg:standardisation:batch-initialisation´): every accepted observation
/// retires one share of prior mass and publishes a fresh snapshot, and the
/// steward that accepts them runs on its own thread
/// (´dec:concurrency:single-steward´), deliberately kept off the snapshot the
/// offering request is being answered from
/// (´dec:ordering:score-before-evolve´). Two requests made while the ramp is
/// running are therefore answered in coordinate systems zero or one advance
/// apart, and which of the two happens is settled by how the threads were
/// scheduled — that is, by how much real time passed between the calls and how
/// loaded the machine was. One advance is one hundredth of the horizon, so it
/// moves the compared statistic by about a percent, two orders of magnitude
/// above the drift budget; and nothing the test controls bounds how many
/// advances fit between two calls. A comparison taken there reads the
/// machine's scheduling rather than the batch path
/// (´inv:guarantee:evidence-authority´).
///
/// At the horizon the prior mass is gone, the ramp accepts nothing further, and
/// an in-service snapshot stops the assessment offering at all. No snapshot can
/// then be published while the compared requests are made, all three worlds
/// stand on the same empirical moments, and the comparison is exact to
/// floating-point order again.
///
/// The ramp is settled with a request that addresses no Sentinel, for two
/// reasons. One repeated request makes the horizon moments the moments of a
/// single constant vector, so they do not depend on how many offers the bounded
/// observation channel happened to take in each world — the settled state is
/// the same whatever the acceptance pattern was. And the per-Sentinel bootstrap
/// accumulator is fed only by requests that supply that Sentinel's coordinate
/// (´alg:standardisation:sentinel-bootstrap´), so leaving it untouched here
/// keeps it far below its own hundred-observation completion — this witness
/// offers it one — and so removes the only other publication that could land
/// mid-comparison.
fn build_batch_independence_world(instance_id: &str) -> Scenario {
    let mut scenario = scenario_with(instance_id, 0xBADC_0FFE_0000_0090_u64, |builder| {
        builder
            .signal_schema(score_verified_request_schema())
            .channel("default", ChannelPolicy::default())
    })
    .expect("batch-independence world should build");

    attach_golden_reporting_sentinel(&mut scenario.world, "S1");
    scenario.settle_cold_ramp_with(&[scenario.request("default", "cold-ramp")]);
    scenario
}

/// Build the two named requests whose payloads are intentionally distinct.
fn batch_independence_requests(scenario: &Scenario) -> Vec<RequestContext> {
    vec![
        with_standard_final_score_verified(scenario.request_with_sentinel("default", "alice", "S1", GOLDEN_COORD)),
        with_degraded_score_verified(scenario.request("default", "bob")),
    ]
}

// ═══════════════════════════════════════════════════════════════════════════════
// RequestContext Tests
// ═══════════════════════════════════════════════════════════════════════════════

// Direct-constructor test: verifies the raw public API shape of
// `RequestContext::new` without going through the harness.
/// A request begins as its entity and nothing else: the coordinate map and the
/// signal map both start empty, and the entity's bytes come through unaltered.
/// Nothing is inferred on the host's behalf, so an attribute absent from a
/// request is absent from the assessment rather than silently defaulted to
/// something the host never said.
///
/// ´claim:assess:a-freshly-built-request-carries-only-its-entity-with-nothing-inferred-on-the-hosts-behalf´
/// ´test:integration:request-context-builder´
#[test]
fn request_context_builder() {
    let entity = EntityKey::new(vec![1, 2, 3, 4, 5]);
    let req = RequestContext::new(entity);

    assert_eq!(req.entity.as_bytes(), &[1, 2, 3, 4, 5]);
    assert!(req.sentinel_coordinates.is_empty());
    assert!(req.signals.is_empty());
}

/// Signals attach to a request by name and keep the value the host supplied —
/// a bare number becomes a numeric signal of exactly that magnitude, negative
/// values included. Nothing is rescaled or clamped at attachment time, so
/// whatever encoding the schema later applies is applied to the host's own
/// number and can be reasoned about from it.
///
/// ´claim:assess:signals-attach-by-name-and-keep-the-value-the-host-supplied-unaltered´
/// ´test:integration:request-context-with-signals´
#[test]
fn request_context_with_signals() {
    let world = build_world();
    let req = world
        .request("default", "alice")
        .with_signal("velocity", 100.0)
        .with_signal("age_hours", 24.5)
        .with_signal("score", -0.5);

    assert_eq!(req.signals.len(), 3);
    assert_eq!(req.signals.get("velocity"), Some(&SignalValue::Numeric(100.0)));
    assert_eq!(req.signals.get("age_hours"), Some(&SignalValue::Numeric(24.5)));
    assert_eq!(req.signals.get("score"), Some(&SignalValue::Numeric(-0.5)));
}

/// One request can address several Sentinels at once, each under its own
/// identifier, and each coordinate survives the full width of the hash space
/// untruncated. An entity sits at a different place in every measurement
/// space, so the coordinates must be kept apart and kept whole — a narrowed
/// coordinate would point at the wrong cell in that Sentinel's tree.
///
/// ´claim:assess:each-sentinel-coordinate-is-kept-whole-and-separate-under-its-own-sentinel-id´
/// ´test:integration:request-context-with-sentinels´
#[test]
fn request_context_with_sentinels() {
    let coord1: u128 = 0x1234_5678_9ABC_DEF0_1234_5678_9ABC_DEF0;
    let coord2: u128 = 0xAAAA_BBBB_CCCC_DDDD_EEEE_FFFF_0000_1111;

    let world = build_world();
    let req = world
        .request("default", "bob")
        .with_sentinel(SentinelId(1), coord1)
        .with_sentinel(SentinelId(2), coord2);

    assert_eq!(req.sentinel_coordinates.len(), 2);
    assert_eq!(req.sentinel_coordinates.get(&SentinelId(1)), Some(&coord1));
    assert_eq!(req.sentinel_coordinates.get(&SentinelId(2)), Some(&coord2));
}

/// Coordinates and signals live in separate stores, so a host may interleave
/// the two kinds of builder call freely and end with both counts intact.
/// Real call sites assemble a request from several sources in whatever order
/// the surrounding code makes convenient; a builder where ordering mattered
/// would drop attributes depending on how the caller happened to be written.
///
/// ´claim:assess:coordinates-and-signals-are-stored-apart-so-builder-calls-may-be-interleaved-freely´
/// ´test:integration:request-context-combined´
#[test]
fn request_context_combined() {
    let world = build_world();
    let req = world
        .request("default", "carol")
        .with_sentinel(SentinelId(1), 0x1234)
        .with_signal("velocity", 100.0)
        .with_sentinel(SentinelId(2), 0x5678)
        .with_signal("score", 0.95);

    assert_eq!(req.sentinel_coordinates.len(), 2);
    assert_eq!(req.signals.len(), 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// AssessmentSharedState Trait Tests
// ═══════════════════════════════════════════════════════════════════════════════

// Note: Full AssessmentSharedState implementation tests are in crate-level tests
// since they require access to internal types like ModelSnapshot and PendingAssessment.
// The trait is designed for internal use and testing within the crate.

/// The shared-state interface can be named behind a trait object, which the
/// compiler will only accept if every method on it is dispatchable
/// dynamically. That keeps the interface within reach of a host that chooses
/// its state implementation at run time or stores several behind one type,
/// rather than confining it to callers that can monomorphise.
///
/// ´claim:assess:the-shared-state-interface-stays-object-safe-so-it-can-be-held-behind-a-trait-object´
/// ´test:integration:assessment-shared-state-trait-is-object-safe´
#[test]
fn assessment_shared_state_trait_is_object_safe() {
    use torrust_assayer::AssessmentSharedState;

    // Mentioning `&dyn AssessmentSharedState` in a signature forces the compiler
    // to check object safety at type-check time. The function body is
    // irrelevant — if the trait were not object-safe, this test would fail
    // to compile.
    fn _accept_trait_object(_: &dyn AssessmentSharedState) {}
}

// ═══════════════════════════════════════════════════════════════════════════════
// Label Guidance Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Asking for labels returns candidates drawn from assessments that are
/// genuinely still pending: the assessment just made comes back by its own
/// identifier, carrying a finite score. The call yields the requests directly
/// rather than a fallible result, because having nothing worth labelling is an
/// ordinary answer — an empty budget's worth of candidates — and not a failure
/// the host must handle.
///
/// ´claim:assess:label-guidance-names-genuinely-pending-assessments-by-identifier-and-scores-each-finitely´
/// ´test:integration:request-labels-public-api-returns-pending-candidate´
#[test]
fn request_labels_public_api_returns_pending_candidate() {
    let world = build_world();
    let assessment = world.assess(world.request("default", "guide-me"));

    let requests = world.assayer().request_labels(
        LabelBudget {
            risk_informative: 1,
            investigation_candidates: 0,
            starvation_relief: 0,
        },
        LabelGuidanceParams::default(),
    );

    assert_eq!(requests.risk_informative.len(), 1);
    assert_eq!(requests.risk_informative[0].assessment_id, assessment.id);
    assert!(requests.risk_informative[0].score.is_finite());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Batch Independence
// ═══════════════════════════════════════════════════════════════════════════════

/// Requests submitted together are assessed independently: each one's risk and
/// resonance match what it would have received alone, whichever order the
/// singletons were run in, and neither borrows the other's payload — the
/// Sentinel alarm belongs to the request that supplied the coordinate, and the
/// signal degradation counts stay with the request whose signals were
/// malformed. Batching is therefore purely an efficiency choice for the host,
/// with no bearing on what any individual request is told.
///
/// ´claim:assess:batching-requests-is-an-efficiency-choice-that-cannot-change-what-any-request-is-told´
/// ´test:integration:assess-batch-matches-forward-and-reverse-singletons´
#[test]
fn assess_batch_matches_forward_and_reverse_singletons() {
    let batch_world = build_batch_independence_world("assess-batch-independence-batch");
    let forward_world = build_batch_independence_world("assess-batch-independence-forward");
    let reverse_world = build_batch_independence_world("assess-batch-independence-reverse");

    let batch_requests = batch_independence_requests(&batch_world);
    let batch_reckonings: Vec<_> = batch_world
        .derive_for_requests(&batch_requests)
        .into_iter()
        .map(|result| result.expect("batch derivation should succeed"))
        .collect();
    assert_eq!(batch_reckonings.len(), 2, "batch should preserve request count");

    assert_eq!(
        batch_reckonings[0].assessment.per_sentinel.len(),
        1,
        "alice request should carry the live Sentinel payload",
    );
    assert!(
        batch_reckonings[1].assessment.per_sentinel.is_empty(),
        "bob request should not inherit alice's Sentinel alarm payload",
    );
    assert_signal_degradation_counts(&batch_reckonings[0], 0, 0, "alice batch payload");
    assert_signal_degradation_counts(&batch_reckonings[1], 1, 1, "bob batch payload");

    let mut forward_requests = batch_independence_requests(&forward_world);
    let forward_alice = forward_world
        .derive_for_request(forward_requests.remove(0))
        .expect("forward alice derivation should succeed");
    let forward_bob = forward_world
        .derive_for_request(forward_requests.remove(0))
        .expect("forward bob derivation should succeed");

    let mut reverse_requests = batch_independence_requests(&reverse_world);
    let reverse_bob_request = reverse_requests.pop().expect("bob request should exist");
    let reverse_bob = reverse_world
        .derive_for_request(reverse_bob_request)
        .expect("reverse bob derivation should succeed");
    let reverse_alice_request = reverse_requests.pop().expect("alice request should exist");
    let reverse_alice = reverse_world
        .derive_for_request(reverse_alice_request)
        .expect("reverse alice derivation should succeed");

    // All three worlds were handed back with their ramps at the horizon, so
    // nothing publishes between the batch call and either singleton call and
    // the two passes read the same immutable snapshot. What is left to compare
    // is arithmetic, and the budget below is the suite's purity budget. The
    // wider budget this site used to borrow was headroom over that arithmetic
    // rather than a measurement of anything, and measurement removed it: run
    // loaded at sixteen concurrent instances of this binary at sixteen test
    // threads, every one of these four comparisons holds at the purity scale.
    assert_core_and_resonance_profile_near_with_drift(
        &batch_reckonings[0],
        &forward_alice,
        PURITY_DRIFT,
        "batch alice vs forward singleton",
    );
    assert_core_and_resonance_profile_near_with_drift(
        &batch_reckonings[1],
        &forward_bob,
        PURITY_DRIFT,
        "batch bob vs forward singleton",
    );
    assert_core_and_resonance_profile_near_with_drift(
        &batch_reckonings[0],
        &reverse_alice,
        PURITY_DRIFT,
        "batch alice vs reverse singleton",
    );
    assert_core_and_resonance_profile_near_with_drift(
        &batch_reckonings[1],
        &reverse_bob,
        PURITY_DRIFT,
        "batch bob vs reverse singleton",
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Derived Output Type Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Every type the public surface hands back can cross a thread boundary and be
/// shared across threads — the whole assessment, the risk basis, the health
/// snapshot, and the label-guidance types alike. A host assesses on its
/// request threads and typically logs, decides, or reports elsewhere; an
/// output tied to the thread that produced it would force a copy into some
/// parallel type at every such handoff.
///
/// ´claim:assess:every-public-output-type-can-be-moved-and-shared-across-threads´
/// ´test:integration:derived-output-types-are-send-sync´
#[test]
fn derived_output_types_are_send_sync() {
    use torrust_assayer::{
        DecisionLandscape, DerivedReckoning, HealthSnapshot, LabelBudget, LabelCandidate, LabelCategory, LabelGuidanceParams,
        LabelRequests, ResonanceProfile, RiskBasis, SentinelAlarmSummary,
    };

    fn assert_send_sync<T: Send + Sync>() {}

    assert_send_sync::<DerivedReckoning>();
    assert_send_sync::<RiskBasis>();
    assert_send_sync::<DecisionLandscape>();
    assert_send_sync::<ResonanceProfile>();
    assert_send_sync::<SentinelAlarmSummary>();
    assert_send_sync::<HealthSnapshot>();
    assert_send_sync::<LabelBudget>();
    assert_send_sync::<LabelGuidanceParams>();
    assert_send_sync::<LabelRequests>();
    assert_send_sync::<LabelCandidate>();
    assert_send_sync::<LabelCategory>();
}

/// Outputs are values a host can duplicate at will, not borrowed views onto
/// engine state. Because a copy is independent of anything the engine holds, a
/// host may retain an assessment for audit, hand a second copy to a decision
/// path, and keep both long after the model has moved on.
///
/// ´claim:assess:public-outputs-are-owned-values-a-host-can-duplicate-and-retain-independently´
/// ´test:integration:derived-output-types-are-clone´
#[test]
fn derived_output_types_are_clone() {
    use torrust_assayer::{
        DecisionLandscape, HealthSnapshot, LabelBudget, LabelCandidate, LabelCategory, LabelGuidanceParams, LabelRequests,
        ResonanceProfile, RiskBasis, SentinelAlarmSummary,
    };

    fn assert_clone<T: Clone>() {}

    assert_clone::<RiskBasis>();
    assert_clone::<DecisionLandscape>();
    assert_clone::<ResonanceProfile>();
    assert_clone::<SentinelAlarmSummary>();
    assert_clone::<HealthSnapshot>();
    assert_clone::<LabelBudget>();
    assert_clone::<LabelGuidanceParams>();
    assert_clone::<LabelRequests>();
    assert_clone::<LabelCandidate>();
    assert_clone::<LabelCategory>();
}

/// Every output type can be rendered for diagnostics without the host writing
/// a formatter of its own. A risk estimate that cannot be printed cannot be
/// put in a log line or an assertion message, and the moment a host most needs
/// to see one is the moment something has gone wrong.
///
/// ´claim:assess:every-public-output-type-can-be-rendered-for-diagnostics-as-it-stands´
/// ´test:integration:derived-output-types-are-debug´
#[test]
fn derived_output_types_are_debug() {
    use torrust_assayer::{
        DecisionLandscape, HealthSnapshot, LabelBudget, LabelCandidate, LabelCategory, LabelGuidanceParams, LabelRequests,
        ResonanceProfile, RiskBasis, SentinelAlarmSummary,
    };

    fn assert_debug<T: std::fmt::Debug>() {}

    assert_debug::<RiskBasis>();
    assert_debug::<DecisionLandscape>();
    assert_debug::<ResonanceProfile>();
    assert_debug::<SentinelAlarmSummary>();
    assert_debug::<HealthSnapshot>();
    assert_debug::<LabelBudget>();
    assert_debug::<LabelGuidanceParams>();
    assert_debug::<LabelRequests>();
    assert_debug::<LabelCandidate>();
    assert_debug::<LabelCategory>();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Concurrent assess
// ═══════════════════════════════════════════════════════════════════════════════

/// Registering and deregistering a Sentinel is safe to do while the engine is
/// serving traffic: a thousand assessments running against four threads while
/// a fifth cycles a Sentinel in and out each come back with a probability that
/// is finite and inside the unit interval, and the engine's health afterwards
/// records no sanitisation, no model fallback, and no dropped health event.
/// What the health may record is the honest trace of the contention itself —
/// batch-init observations skipped on the contended warm-up lock, now
/// reported in band rather than silent — and every degraded assessment must
/// be accounted for by exactly that trace. Changing the measurement surface
/// is an operational act a host performs on a live system, so it must not be
/// payable in corrupted estimates for whatever requests happen to be in
/// flight.
///
/// ´claim:assess:sentinel-registration-churn-racing-live-traffic-leaves-every-estimate-in-range-and-health-clean´
/// ´test:integration:concurrent-assess-under-lifecycle-change-returns-valid-assessments´
#[test]
fn concurrent_assess_under_lifecycle_change_returns_valid_assessments() {
    // The wider concurrency property
    // (´claim:assess:concurrent-assessments-neither-collide-on-an-identifier-nor-lose-a-pending-record´)
    // has 10,000
    // assessments can be in flight when a Sentinel's report changes
    // mid-way without producing torn reads, NaN, or panics. The
    // crate-level [`assess_pipeline`](../../src/tests/assess_pipeline.rs)
    // already pins the bare assessment-side concurrency invariant
    // (4 threads × 100 calls). This integration variant verifies the
    // *public* `assess()` surface tolerates a Sentinel
    // register/deregister handshake racing the assess stream — a
    // weaker but more realistic reflection of what a host application
    // experiences.
    //
    // Shape:
    //   * 4 reader threads × 250 calls each = 1,000 concurrent
    //     assessments on the same engine.
    //   * One lifecycle thread alternates `register_sentinel` /
    //     `deregister_sentinel` 8 times, exercising both the
    //     snapshot-publish path (read-side) and the lifecycle
    //     event channel (write-side) concurrently.
    //
    // The assertion floor: every `assess()` returns `Ok` with a
    // finite `p_bad`, no thread panics, the final post-join
    // derivation still succeeds, and health remains clean — no
    // sanitisation, no fallbacks, no events dropped — with any
    // degraded-assessment tally accounted for by the reported
    // batch-init load conditions alone.
    use std::thread;

    use torrust_assayer::SentinelRegistration;
    use torrust_assayer::types::SentinelId;

    let world = build_world();
    let engine = world.assayer();

    // Pre-register a baseline Sentinel so the snapshot readers see
    // a non-degenerate map for the entire run; the racing lifecycle
    // thread cycles a *separate* Sentinel id.
    engine
        .register_sentinel(SentinelRegistration {
            id: SentinelId(100),
            name: "baseline".to_owned(),
        })
        .expect("baseline register");

    thread::scope(|s| {
        // Reader threads: 4 × 250 assessments each.
        for tid in 0..4 {
            s.spawn(move || {
                for i in 0..250_u32 {
                    let entity = EntityKey::new(format!("t{tid}-i{i}").into_bytes());
                    let req = RequestContext::new(entity);
                    let out = engine.assess(&[req]);
                    let r = out.into_iter().next().expect("batch of 1 yields 1 assessment");
                    assert!(r.risk.p_bad.is_finite(), "t{tid} round {i}: p_bad must be finite");
                    assert!(
                        (0.0..=1.0).contains(&r.risk.p_bad),
                        "t{tid} round {i}: p_bad {} out of range",
                        r.risk.p_bad,
                    );
                }
            });
        }

        // Lifecycle thread: cycle a churn Sentinel in and out 8
        // times concurrently with the readers. `register_sentinel`
        // and `deregister_sentinel` both take `&self`, so we can
        // race them against the reader fleet on the same engine.
        s.spawn(move || {
            let id = SentinelId(200);
            for cycle in 0..8 {
                engine
                    .register_sentinel(SentinelRegistration {
                        id,
                        name: format!("churn-{cycle}"),
                    })
                    .unwrap_or_else(|e| panic!("churn cycle {cycle} register: {e:?}"));
                engine
                    .deregister_sentinel(id)
                    .unwrap_or_else(|e| panic!("churn cycle {cycle} deregister: {e:?}"));
            }
        });
    });

    // Post-join smoke check: the engine still assesses and derives normally,
    // and the health snapshot reports no degradation events.
    let post = world.derive_default("post").expect("post-concurrent assess/derive");
    assert!(post.assessment.risk.p_bad.is_finite(), "post-concurrent p_bad");
    let h = engine.health_summary();
    assert_eq!(h.degradation.signals_sanitised, 0, "no signal sanitisation");
    assert_eq!(h.degradation.features_sanitised, 0, "no feature sanitisation");
    assert_eq!(h.degradation.sentinel_slots_zeroed, 0, "no slots zeroed");
    assert_eq!(h.degradation.model_fallbacks, 0, "no model fallbacks");
    // A command channel too full to take a cold-ramp observation is a load
    // condition reported in band; with every other counter at zero, each
    // degraded assessment must be accounted for by that trace alone.
    assert!(
        h.degradation.total_degraded <= h.degradation.batch_init_observations_skipped,
        "every degraded assessment is a refused cold-ramp observation: {} degraded, {} refused",
        h.degradation.total_degraded,
        h.degradation.batch_init_observations_skipped,
    );
    assert_eq!(h.health_events_dropped, 0, "no health events dropped");
    assert!(!h.any_cascade_terminus, "no cascade terminus tripped");
}

/// The periodic Cholesky recompute is the riskiest moment in the learner's
/// life — the factorisation the fast path reads is rebuilt underneath it — and
/// it survives lifecycle churn crossing it. A label stream driven past the
/// configured cadence while Sentinels register and deregister leaves recomputes
/// actually observed, every racing assessment finite, no Schur correction
/// skipped, no health event dropped, and the cascade terminus untripped. The
/// recompute is not something a host can schedule for a quiet moment, so it
/// has to be correct during a busy one.
///
/// ´claim:assess:crossing-the-cholesky-recompute-cadence-under-lifecycle-churn-skips-no-schur-correction-and-drops-no-health-event´
/// ´test:integration:lifecycle-churn-around-cholesky-recompute-keeps-fast-path-valid´
#[test]
fn lifecycle_churn_around_cholesky_recompute_keeps_fast_path_valid() {
    use std::thread;

    const RECOMPUTE_CADENCE: u32 = 100;
    const WARMUP_LABELS: usize = 99;
    const RACING_LABELS: usize = 256;
    const LIFECYCLE_CYCLES: usize = 12;

    let mut config = AssayerConfig {
        instance_id: "assess-lifecycle-recompute".to_owned(),
        // The capacity is host-set with no default; the fixture declares it.
        infrastructure: torrust_assayer::testing::test_infrastructure(),
        ..Default::default()
    };
    config.cholesky.n_recompute = RECOMPUTE_CADENCE;

    let world = scenario_with_config(config, 0x0042_0089, |builder| {
        builder.channel("default", ChannelPolicy::default())
    })
    .expect("recompute concurrency world should build");
    let engine = world.assayer();

    for i in 0..WARMUP_LABELS {
        submit_assessment_label(engine, &format!("warmup-{i}"), i % 2 == 0);
    }
    world.flush_labels().expect("warmup labels should flush");

    thread::scope(|scope| {
        scope.spawn(move || {
            for i in 0..RACING_LABELS {
                submit_assessment_label(engine, &format!("race-label-{i}"), i % 2 == 1);
            }
        });

        scope.spawn(move || {
            for i in 0..LIFECYCLE_CYCLES {
                let sentinel_id = SentinelId(10_000 + u32::try_from(i).expect("cycle id fits in u32"));
                engine
                    .register_sentinel(SentinelRegistration {
                        id: sentinel_id,
                        name: format!("recompute-churn-{i}"),
                    })
                    .unwrap_or_else(|err| panic!("register churn Sentinel {i}: {err:?}"));

                let assessment = engine.assess(&[RequestContext::new(EntityKey::new(format!("race-assess-{i}").into_bytes()))]);
                assert_single_assessment_finite(&assessment, &format!("race assess {i}"));

                engine
                    .deregister_sentinel(sentinel_id)
                    .unwrap_or_else(|err| panic!("deregister churn Sentinel {i}: {err:?}"));
            }
        });
    });

    world.flush_labels().expect("racing labels should flush");

    let post = world.derive_default("post-recompute-churn").expect("post-race assess/derive");
    assert_reckoning_well_formed(&post, "post recompute lifecycle churn");

    let report = engine.full_health_report();
    let total_recomputes: u64 = report.precision.values().map(|precision| precision.cholesky_recomputes).sum();
    assert!(
        total_recomputes > 0,
        "the racing label stream should cross the configured Cholesky recompute cadence",
    );
    assert_eq!(
        report.health_events_dropped, 0,
        "health events should not drop under recompute/lifecycle churn"
    );
    assert_eq!(
        report.schur_corrections_skipped, 0,
        "lifecycle churn should not skip Schur corrections"
    );
    assert!(
        !report.cascade_terminus_active,
        "Cholesky cascade terminus must stay inactive"
    );
    assert_health_clean(&world);
}

fn submit_assessment_label(engine: &Assayer, entity: &str, adverse: bool) {
    let assessments = engine.assess(&[RequestContext::new(EntityKey::new(entity.to_string().into_bytes()))]);
    assert_single_assessment_finite(&assessments, entity);
    let assessment = assessments.into_iter().next().expect("single assessment result");
    let label = if adverse {
        LabelSpec::adverse(assessment.id)
    } else {
        LabelSpec::benign(assessment.id)
    };
    engine
        .label(label.build())
        .unwrap_or_else(|err| panic!("label {entity}: {err:?}"));
}

fn assert_single_assessment_finite(assessments: &[torrust_assayer::RiskAssessment], label: &str) {
    assert_eq!(
        assessments.len(),
        1,
        "{label}: single-request batch should return one assessment"
    );
    let risk = assessments[0].risk.p_bad;
    assert!(risk.is_finite(), "{label}: p_bad should be finite");
    assert!((0.0..=1.0).contains(&risk), "{label}: p_bad {risk} out of range");
}

/// The standardisation transition is reported on both surfaces and each
/// reading carries its coordinate join key. The compact reading riding an
/// assessment is retrospective while the full report is current, so they may
/// legitimately differ during an advance; whenever their versions agree,
/// their phase and accepted count agree too. Before the first accepted
/// observation both name the same `WaitingForInit` state at zero, and at the
/// horizon both name the same `InService` state at the horizon's count. Each
/// accepted advance publishes a later snapshot version, and a full-report
/// version never names two different phase/count states.
///
/// The full pair is read from the published snapshot rather than from the last
/// label-time summary, which is what makes this true of a deployment that has
/// not labelled anything yet — the whole of a cold start, and precisely when
/// an operator is watching.
///
/// ´claim:assess:the-standardisation-transition-is-reported-on-both-surfaces-and-they-agree-at-rest´
/// ´test:integration:standardisation-transition-reported-on-both-surfaces´
#[test]
fn standardisation_transition_reported_on_both_surfaces() {
    let world = scenario("standardisation-transition-surfaces", 0x57A4_D123);
    let engine = world.assayer();

    // Before anything is assessed at all, the full report says waiting at zero.
    // It is read first because it is the *current* reading: the moment one
    // assessment has offered its vector the steward may apply it, and the
    // report would then correctly say `Transitioning` while the assessment
    // that caused it correctly still says `WaitingForInit`. That disagreement
    // is the design and is asserted below; here the point is the starting
    // state, so nothing has happened yet.
    let cold_report = engine.full_health_report();
    assert_eq!(cold_report.standardisation.phase, StandardisationPhase::WaitingForInit);
    assert_eq!(cold_report.standardisation.accepted_observations, 0);

    // The first assessment is scored in the class-prior coordinate system,
    // whatever its own offer goes on to cause.
    let first = world.derive_default("alice").expect("first assess");
    assert_eq!(
        first.assessment.health.standardisation_phase,
        StandardisationPhase::WaitingForInit,
        "the first assessment is scored in the class-prior coordinate system",
    );
    assert_eq!(first.assessment.health.standardisation_observations, 0);
    assert_eq!(
        cold_report.standardisation.snapshot_version, first.assessment.health.snapshot_version,
        "both initial readings name the same published coordinate state",
    );

    // Walk to the horizon. Along the way the compact pair never runs ahead of
    // the full one: a result cannot have been scored in a coordinate state
    // later than the one the engine has reached.
    let mut versions: Vec<u64> = vec![first.assessment.health.snapshot_version];
    let mut counts: Vec<usize> = vec![0];
    let mut report_states = std::collections::HashMap::from([(
        cold_report.standardisation.snapshot_version,
        (
            cold_report.standardisation.phase,
            cold_report.standardisation.accepted_observations,
        ),
    )]);
    for _ in 0..4_000 {
        let r = world.derive_default("alice").expect("assess");
        let compact = &r.assessment.health;
        let full = engine.full_health_report();
        let full_state = (full.standardisation.phase, full.standardisation.accepted_observations);

        if let Some(previous) = report_states.insert(full.standardisation.snapshot_version, full_state) {
            assert_eq!(
                previous, full_state,
                "one full-report snapshot version must name one phase/count state",
            );
        }
        if full.standardisation.snapshot_version == compact.snapshot_version {
            assert_eq!(
                full.standardisation.phase, compact.standardisation_phase,
                "equal versions must name the same phase on both surfaces",
            );
            assert_eq!(
                full.standardisation.accepted_observations, compact.standardisation_observations,
                "equal versions must name the same accepted count on both surfaces",
            );
        }

        assert!(
            compact.standardisation_observations <= full.standardisation.accepted_observations,
            "the compact pair is retrospective and must not run ahead of the current one: {} vs {}",
            compact.standardisation_observations,
            full.standardisation.accepted_observations,
        );

        if compact.standardisation_observations > *counts.last().expect("seeded") {
            counts.push(compact.standardisation_observations);
            versions.push(compact.snapshot_version);
        }
        if compact.standardisation_phase.is_in_service() {
            break;
        }
    }

    // Every advance published a snapshot, so the version rose with the count.
    assert!(counts.len() > 1, "the ramp must have advanced for this to decide anything");
    for w in versions.windows(2) {
        assert!(
            w[1] > w[0],
            "each accepted advance publishes a new snapshot version: {} then {}",
            w[0],
            w[1],
        );
    }

    // At rest at the horizon the two surfaces agree exactly.
    let settled = world.derive_default("alice").expect("settled assess");
    let settled_report = engine.full_health_report();
    assert_eq!(
        settled.assessment.health.standardisation_phase,
        StandardisationPhase::InService,
    );
    assert_eq!(
        settled_report.standardisation.phase, settled.assessment.health.standardisation_phase,
        "at rest the two surfaces report the same phase",
    );
    assert_eq!(
        settled_report.standardisation.accepted_observations, settled.assessment.health.standardisation_observations,
        "at rest the two surfaces report the same accepted count",
    );
    assert_eq!(
        settled_report.standardisation.snapshot_version, settled.assessment.health.snapshot_version,
        "at rest the two surfaces use the same version to name that phase/count state",
    );
}
