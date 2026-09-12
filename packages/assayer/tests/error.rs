// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`all_errors_are_send_sync`] | errors | Every error the assayer can hand back crosses a thread boundary. The model owner, the identity maintenance thread and the caller are different threads, so a failure that could not be sent between them would have to be flattened into a string at the boundary and would lose its variant on the way. |
//! | [`all_errors_implement_std_error`] | errors | All five of the assayer's error families are standard errors, so a host can put any of them into its own error type, its logging, or a boxed chain without the assayer having to know which framework it landed in. |
//! | [`label_error_display`] | errors | A label failure says which stage of the label path gave way — the pending buffer, the journal write, the enqueue after a durable append, the channel, or the owner having gone — and the journal case carries the underlying I/O complaint in its own text. The five outcomes call for different responses from the host, from retry to give up, so the message has to distinguish them rather than say only that labelling failed. |
//! | [`report_error_display`] | errors | A rejected sentinel report names the structural violation and the thing that violated it: the missing root, the duplicated or non-dyadic interval, or the unrecognised sentinel by its identifier. These faults are the adapter layer's to fix, and it needs the offending cell to find them in a report of thousands. |
//! | [`error_lifecycle_error_display`] | errors | Lifecycle refusals are reported in terms of the entity the host named: the kind of entity and its identifier appear in the duplicate, not-found, empty-name and bad-domain-width messages, the last of which also states the range it expected. One registration sweep may cover sentinels, axes and dimensions together, so a message that omitted which one failed would leave the host to guess among them. |
//! | [`build_error_display`] | errors | A refused construction points at the part of the builder configuration that was wrong: the duplicated signal name, the index of the offending interaction template and why, the dotted path of the out-of-range parameter together with the value it was given. Construction happens once at start-up, so the message is the only diagnosis the operator gets before the process gives up. |
//! | [`channel_error_display`] | errors | A rejected channel policy names the setting at fault by the field the host wrote and states the constraint it broke — at least two actions, strictly ascending severity, positive rewards and sensitivities, the two rates inside their intervals, and finiteness, which is the one refusal that carries the offending field and value because a value that is not a number breaks no bound and so cannot be named by one. The host can go straight to that field rather than re-deriving which of a dozen policy numbers the validator objected to. |
//! | [`label_error_journal_has_source`] | errors | A failed journal write keeps the underlying I/O error reachable as its source rather than only summarising it in text. A host that wants the operating system's own error kind — to tell a full disk from a permissions problem, say — can walk the chain and find it. |
//! | [`build_error_persistence_has_source`] | errors | cites (´claim:errors:an-assayer-error-wrapping-an-io-failure-keeps-it-reachable-as-the-source´) |
//! | [`label_channel_overflow_without_journal_consumes_pending`] | errors | A label that cannot be enqueued has already spent its pending assessment: the full-channel refusal comes back once, and submitting the very same label again is answered with the assessment being unknown rather than with the same backpressure. Without journalling that first refusal is the label's end, and the second answer is what tells the host so — a retry loop would otherwise spin against an entry that no longer exists. |
//! | [`label_channel_overflow_with_journal_reports_journaled_not_enqueued`] | errors | With journalling configured the same backpressure means something different: the label is refused as journalled-but-not-enqueued, and the journal on disk has genuinely grown. The label is not lost, merely deferred to restart, and the distinct variant is how the host knows not to resubmit it. |
//! | [`label_reports_model_owner_shutdown_after_owner_exit`] | errors | A model owner that has gone is reported as exactly that, not as a full channel or a lost label. The two look alike from the sending side — nothing can be enqueued either way — but one clears when the queue drains and the other never will, so a host that cannot tell them apart would retry forever. |
//! | [`label_reports_model_owner_shutdown_with_journal_preserves_durable_record`] | errors | The durable append happens before the live enqueue is attempted, and a failed enqueue rewrites nothing: after the owner has gone, the journal has grown by the label while the last successful checkpoint is byte-for-byte what it was, and retrying the consumed assessment appends no duplicate. Ordering the write first is what makes the label survive the failure; leaving the checkpoint alone is what stops a failed operation from being mistaken for a completed one. |
//! | [`label_reports_model_owner_shutdown_journal_replays_on_restart`] | errors | The promise the durable append makes is kept: an instance rebuilt from the same directories replays the label written during the failed enqueue, counts it among its processed labels, and truncates the journal once the replay has been checkpointed. Durability that were never replayed would be a file on disk rather than a label, and a journal never truncated would replay it again at every restart. |
//! | [`sanitise_f64_nan`] | errors | A value that is not a number is replaced by the caller's chosen default and the substitution is reported alongside it. The checkpoint does not merely clean the value: it says that it did, so a NaN entering the model is a health event rather than a silently plausible zero. |
//! | [`sanitise_f64_positive_infinity`] | errors | cites (´claim:errors:a-non-finite-value-is-replaced-by-the-callers-default-and-the-substitution-is-reported´) |
//! | [`sanitise_f64_negative_infinity`] | errors | cites (´claim:errors:a-non-finite-value-is-replaced-by-the-callers-default-and-the-substitution-is-reported´) |
//! | [`sanitise_f64_normal_value`] | errors | An ordinary finite value passes through bit-for-bit and is not flagged. The checkpoint is a filter on the pathological cases only — if it perturbed healthy values it would be a source of drift rather than a guard against it. |
//! | [`sanitise_f64_zero`] | errors | cites (´claim:errors:a-finite-value-passes-through-the-checkpoint-unchanged-and-unflagged´) |
//! | [`sanitise_f64_negative_zero`] | errors | cites (´claim:errors:a-finite-value-passes-through-the-checkpoint-unchanged-and-unflagged´) |
//! | [`sanitise_f64_custom_default`] | errors | cites (´claim:errors:a-non-finite-value-is-replaced-by-the-callers-default-and-the-substitution-is-reported´) |
//! | [`sanitise_f64_subnormal`] | errors | cites (´claim:errors:a-finite-value-passes-through-the-checkpoint-unchanged-and-unflagged´) |
//! | [`check_vec_nan_empty`] | errors | Scanning a slice with nothing wrong in it yields no indices at all, and the empty slice is the degenerate instance of that. A spot-check over state that does not exist yet reports health rather than failing, so a cold model needs no special case around the checkpoints. |
//! | [`check_vec_nan_all_clean`] | errors | cites (´claim:errors:a-slice-holding-no-non-finite-values-yields-an-empty-index-list´) |
//! | [`check_vec_nan_mixed`] | errors | The scan reports the position of every non-finite element, in ascending order, and passes over the healthy ones between them — NaN and both infinities alike. Positions are what make the finding actionable: they name which features or which axes went bad, not merely that something did. |
//! | [`check_vec_nan_all_bad`] | errors | cites (´claim:errors:the-scan-reports-the-ascending-positions-of-every-non-finite-element´) |
//! | [`check_vec_nan_single_middle`] | errors | cites (´claim:errors:the-scan-reports-the-ascending-positions-of-every-non-finite-element´) |
//! | [`check_diagonal_nan_clean`] | errors | Checking a covariance diagonal answers exactly as checking the same numbers as a plain slice would: a healthy diagonal reports no positions. The diagonal entry point exists to say what is being spot-checked, not to apply a different standard of health to a matrix than to a vector. |
//! | [`check_diagonal_nan_with_nan`] | errors | cites (´claim:errors:scanning-a-matrix-diagonal-answers-exactly-as-scanning-the-same-slice-would´) |

//! Integration tests for `torrust_assayer::error`.
//!
//! The "clean" cases among the NaN utilities double-check their outputs through
//! [`torrust_assayer::testing::assert_finite`] — the harness helper
//! that wraps `check_vec_nan` — so the test-facing contract and the
//! primitive stay in lockstep.

use std::io;

use torrust_assayer::error::{
    BuildError, ChannelError, LabelError, LifecycleError, ReportError, check_diagonal_nan, check_vec_nan, sanitise_f64,
};
use torrust_assayer::testing::{LabelSpec, World, assert_finite};
use torrust_assayer::types::{CellInterval, DimensionId, SentinelId};
use torrust_assayer::{AssayerConfig, ChannelPolicy, DerivedReckoning};

// ─────────────────────────────────────────────────────────────────────────────
// Send + Sync bounds
// ─────────────────────────────────────────────────────────────────────────────

/// Every error the assayer can hand back crosses a thread boundary. The model
/// owner, the identity maintenance thread and the caller are different threads,
/// so a failure that could not be sent between them would have to be flattened
/// into a string at the boundary and would lose its variant on the way.
///
/// ´claim:errors:every-assayer-error-type-can-cross-a-thread-boundary´
/// ´test:integration:all-errors-are-send-sync´
#[test]
fn all_errors_are_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<LabelError>();
    assert_send_sync::<ReportError>();
    assert_send_sync::<LifecycleError>();
    assert_send_sync::<BuildError>();
    assert_send_sync::<ChannelError>();
}

/// All five of the assayer's error families are standard errors, so a host can
/// put any of them into its own error type, its logging, or a boxed chain
/// without the assayer having to know which framework it landed in.
///
/// ´claim:errors:every-assayer-error-type-implements-the-standard-error-trait´
/// ´test:integration:all-errors-implement-std-error´
#[test]
fn all_errors_implement_std_error() {
    fn assert_error<T: std::error::Error>() {}
    assert_error::<LabelError>();
    assert_error::<ReportError>();
    assert_error::<LifecycleError>();
    assert_error::<BuildError>();
    assert_error::<ChannelError>();
}

// ─────────────────────────────────────────────────────────────────────────────
// Display tests
// ─────────────────────────────────────────────────────────────────────────────

/// A label failure says which stage of the label path gave way — the pending
/// buffer, the journal write, the enqueue after a durable append, the channel,
/// or the owner having gone — and the journal case carries the underlying I/O
/// complaint in its own text. The five outcomes call for different responses
/// from the host, from retry to give up, so the message has to distinguish them
/// rather than say only that labelling failed.
///
/// ´claim:errors:a-label-failure-message-says-which-stage-of-the-label-path-gave-way´
/// ´test:integration:label-error-display´
#[test]
fn label_error_display() {
    let e = LabelError::AssessmentNotFound;
    assert!(e.to_string().contains("not found"));

    let e = LabelError::JournalFailed(io::Error::other("disk full"));
    let msg = e.to_string();
    assert!(msg.contains("journal"));
    assert!(msg.contains("disk full"));

    let e = LabelError::JournaledButNotEnqueued;
    assert!(e.to_string().contains("journaled"));

    let e = LabelError::ChannelFull;
    assert!(e.to_string().contains("full"));

    let e = LabelError::ModelOwnerShutdown;
    assert!(e.to_string().contains("shut down"));
}

/// A rejected sentinel report names the structural violation and the thing that
/// violated it: the missing root, the duplicated or non-dyadic interval, or the
/// unrecognised sentinel by its identifier. These faults are the adapter layer's
/// to fix, and it needs the offending cell to find them in a report of thousands.
///
/// ´claim:errors:a-report-rejection-names-the-structural-violation-and-the-cell-or-sentinel-that-caused-it´
/// ´test:integration:report-error-display´
#[test]
fn report_error_display() {
    let e = ReportError::MissingRootCell;
    assert!(e.to_string().contains("root cell"));

    let e = ReportError::DuplicateCell {
        cell: CellInterval::new(0, 0xFF, 8),
    };
    assert!(e.to_string().contains("duplicate"));

    let e = ReportError::NonDyadicInterval {
        cell: CellInterval::new(3, 10, 4),
    };
    assert!(e.to_string().contains("non-dyadic"));

    let e = ReportError::UnknownSentinel { id: SentinelId(42) };
    let msg = e.to_string();
    assert!(msg.contains("unknown sentinel"));
    assert!(msg.contains("42"));
}

/// Lifecycle refusals are reported in terms of the entity the host named: the
/// kind of entity and its identifier appear in the duplicate, not-found, empty-name
/// and bad-domain-width messages, the last of which also states the range it
/// expected. One registration sweep may cover sentinels, axes and dimensions
/// together, so a message that omitted which one failed would leave the host to
/// guess among them.
///
/// ´claim:errors:a-lifecycle-refusal-names-the-entity-type-and-the-identifier-it-concerns´
/// ´test:integration:error-lifecycle-error-display´
#[test]
fn error_lifecycle_error_display() {
    let e = LifecycleError::DuplicateId {
        entity_type: "Sentinel",
        id: "s-1".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("duplicate"));
    assert!(msg.contains("Sentinel"));
    assert!(msg.contains("s-1"));

    let e = LifecycleError::NotFound {
        entity_type: "OutcomeAxis",
        id: "a-5".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains("not found"));
    assert!(msg.contains("a-5"));

    let e = LifecycleError::ModelOwnerShutdown;
    assert!(e.to_string().contains("model owner"));

    let e = LifecycleError::IdentityMaintenanceShutdown;
    assert!(e.to_string().contains("identity maintenance"));

    let e = LifecycleError::CommandChannelFull;
    assert!(e.to_string().contains("full"));

    let e = LifecycleError::EmptyName { entity_type: "Sentinel" };
    assert!(e.to_string().contains("empty"));
    assert!(e.to_string().contains("Sentinel"));

    let e = LifecycleError::InvalidDomainBits {
        id: DimensionId(7),
        domain_bits: 0,
    };
    let msg = e.to_string();
    assert!(msg.contains("domain_bits"));
    assert!(msg.contains('7'));
    assert!(msg.contains('0'));
}

/// A refused construction points at the part of the builder configuration that
/// was wrong: the duplicated signal name, the index of the offending interaction
/// template and why, the dotted path of the out-of-range parameter together with
/// the value it was given. Construction happens once at start-up, so the message
/// is the only diagnosis the operator gets before the process gives up.
///
/// ´claim:errors:a-build-refusal-points-at-the-offending-name-index-or-parameter-value´
/// ´test:integration:build-error-display´
#[test]
fn build_error_display() {
    let e = BuildError::NoSignalSchema;
    assert!(e.to_string().contains("signal schema"));

    let e = BuildError::DuplicateSignalName { name: "ip_score".into() };
    assert!(e.to_string().contains("ip_score"));

    let e = BuildError::InvalidInteractionTemplate {
        template_index: 3,
        reason: "position exceeds block size".into(),
    };
    let msg = e.to_string();
    assert!(msg.contains('3'));
    assert!(msg.contains("position exceeds"));

    let e = BuildError::InvalidParameter {
        path: "model.gamma_operational",
        value: -0.5,
        reason: "must be in (0, 1)",
    };
    let msg = e.to_string();
    assert!(msg.contains("gamma_operational"));
    assert!(msg.contains("-0.5"));

    let e = BuildError::PersistenceSetupFailed(io::Error::new(io::ErrorKind::PermissionDenied, "access denied"));
    assert!(e.to_string().contains("persistence"));
}

/// A rejected channel policy names the setting at fault by the field the host
/// wrote and states the constraint it broke — at least two actions, strictly
/// ascending severity, positive rewards and sensitivities, the two rates
/// inside their intervals, and finiteness, which is the one refusal that
/// carries the offending field and value because a value that is not a number
/// breaks no bound and so cannot be named by one. The host can go straight to
/// that field rather than re-deriving which of a dozen policy numbers the
/// validator objected to.
///
/// ´claim:errors:a-channel-policy-rejection-names-the-setting-at-fault-and-the-constraint-it-broke´
/// ´test:integration:channel-error-display´
#[test]
fn channel_error_display() {
    let e = ChannelError::TooFewActions;
    assert!(e.to_string().contains("2 actions"));

    let e = ChannelError::ActionsNotOrdered;
    assert!(e.to_string().contains("ascending"));

    let e = ChannelError::NonPositiveReward;
    assert!(e.to_string().contains("positive"));

    let e = ChannelError::InvalidSlowSeverity;
    assert!(e.to_string().contains("slow_severity"));

    let e = ChannelError::InvalidBlockCatches;
    assert!(e.to_string().contains("block_catches"));

    let e = ChannelError::NonPositiveSensitivity;
    assert!(e.to_string().contains("sensitivity"));

    let e = ChannelError::InvalidGammaQt;
    assert!(e.to_string().contains("gamma_q_t"));

    let e = ChannelError::NonFiniteParameter {
        field: "reward.beta_bad",
        value: f64::NAN,
    };
    let rendered = e.to_string();
    assert!(rendered.contains("reward.beta_bad"), "{rendered}");
    assert!(rendered.contains("NaN"), "{rendered}");
}

// ─────────────────────────────────────────────────────────────────────────────
// Error source chaining
// ─────────────────────────────────────────────────────────────────────────────

/// A failed journal write keeps the underlying I/O error reachable as its source
/// rather than only summarising it in text. A host that wants the operating
/// system's own error kind — to tell a full disk from a permissions problem, say —
/// can walk the chain and find it.
///
/// ´claim:errors:an-assayer-error-wrapping-an-io-failure-keeps-it-reachable-as-the-source´
/// ´test:integration:label-error-journal-has-source´
#[test]
fn label_error_journal_has_source() {
    let inner = io::Error::other("disk full");
    let e = LabelError::JournalFailed(inner);
    assert!(std::error::Error::source(&e).is_some());
}

/// The same holds at the other end of the lifecycle: a persistence directory the
/// builder could not prepare carries the I/O refusal underneath it, so the
/// operator can see that the path was denied rather than merely absent.
///
/// (´claim:errors:an-assayer-error-wrapping-an-io-failure-keeps-it-reachable-as-the-source´)
/// ´test:integration:build-error-persistence-has-source´
#[test]
fn build_error_persistence_has_source() {
    let inner = io::Error::new(io::ErrorKind::PermissionDenied, "denied");
    let e = BuildError::PersistenceSetupFailed(inner);
    assert!(std::error::Error::source(&e).is_some());
}

// ─────────────────────────────────────────────────────────────────────────────
// Runtime failure surfaces
// ─────────────────────────────────────────────────────────────────────────────

/// The label channel's tabulated floor: the smallest capacity the
/// builder admits (´tab:config:concurrency´). Overflow tests park the
/// steward and submit one label more than this.
const LABEL_CHANNEL_FLOOR: usize = 100;

fn floor_label_channel_world(instance_id: &str, seed: u64) -> World {
    let mut config = AssayerConfig {
        instance_id: instance_id.to_owned(),
        // The capacity is host-set with no default; the fixture declares it.
        infrastructure: torrust_assayer::testing::test_infrastructure(),
        ..Default::default()
    };
    config.infrastructure.label_channel_capacity = LABEL_CHANNEL_FLOOR;

    World::builder(config)
        .channel("default", ChannelPolicy::default())
        .seed(seed)
        .build()
        .expect("floor label-channel world should build")
}

/// Fill the label channel to its floor capacity with the steward parked,
/// returning one further derived assessment whose label must overflow.
fn fill_label_channel(world: &World) -> DerivedReckoning {
    for i in 0..LABEL_CHANNEL_FLOOR {
        let filler = world.derive_default(&format!("fill-{i}")).expect("derive filler assessment");
        let ack = world
            .label(LabelSpec::adverse(filler.assessment.id).ground_truth().build())
            .expect("filler label should enqueue while the channel has room");
        assert_eq!(ack.assessment_id, filler.assessment.id);
    }
    world.derive_default("overflow").expect("derive overflowing assessment")
}

/// A label that cannot be enqueued has already spent its pending assessment: the
/// full-channel refusal comes back once, and submitting the very same label again
/// is answered with the assessment being unknown rather than with the same
/// backpressure. Without journalling that first refusal is the label's end, and
/// the second answer is what tells the host so — a retry loop would otherwise
/// spin against an entry that no longer exists.
///
/// ´claim:errors:a-label-that-cannot-be-enqueued-has-already-consumed-its-pending-assessment-so-a-retry-finds-nothing´
/// ´test:integration:label-channel-overflow-without-journal-consumes-pending´
#[test]
fn label_channel_overflow_without_journal_consumes_pending() {
    let world = floor_label_channel_world("label-overflow-no-journal", 0x5754_4147_4532_0086);

    let owner_block = world.block_model_owner_for_test();

    let overflowing = fill_label_channel(&world);
    let second_label = LabelSpec::benign(overflowing.assessment.id).ground_truth().build();

    let err = world
        .label(second_label.clone())
        .expect_err("the label past the declared capacity should observe a full no-journal channel");
    assert!(matches!(err, LabelError::ChannelFull));

    let retry = world
        .label(second_label)
        .expect_err("full-channel failure consumes the pending entry before returning");
    assert!(matches!(retry, LabelError::AssessmentNotFound));

    drop(owner_block);
    world.flush_labels().expect("queued label should flush after owner release");
}

/// With journalling configured the same backpressure means something different:
/// the label is refused as journalled-but-not-enqueued, and the journal on disk
/// has genuinely grown. The label is not lost, merely deferred to restart, and
/// the distinct variant is how the host knows not to resubmit it.
///
/// ´claim:errors:journalling-turns-a-full-label-channel-from-a-lost-label-into-a-durable-one-awaiting-restart´
/// ´test:integration:label-channel-overflow-with-journal-reports-journaled-not-enqueued´
#[cfg(feature = "serde")]
#[test]
fn label_channel_overflow_with_journal_reports_journaled_not_enqueued() {
    let temp_dir = tempfile::tempdir().expect("temp persistence dir");
    let mut config = AssayerConfig {
        instance_id: "label-overflow-journal".to_owned(),
        // The capacity is host-set with no default; the fixture declares it.
        infrastructure: torrust_assayer::testing::test_infrastructure(),
        ..Default::default()
    };
    config.infrastructure.label_channel_capacity = LABEL_CHANNEL_FLOOR;

    let world = World::builder(config)
        .persistence_dirs(temp_dir.path(), temp_dir.path())
        .channel("default", ChannelPolicy::default())
        .seed(0x5754_4147_4532_0186)
        .build()
        .expect("journaled floor label-channel world should build");

    let owner_block = world.block_model_owner_for_test();

    let overflowing = fill_label_channel(&world);
    let second_label = LabelSpec::benign(overflowing.assessment.id).ground_truth().build();

    let err = world
        .label(second_label.clone())
        .expect_err("the journaled label past the declared capacity should fail after durable append");
    assert!(matches!(err, LabelError::JournaledButNotEnqueued));

    let retry = world
        .label(second_label)
        .expect_err("journaled-not-enqueued label consumes the live pending entry");
    assert!(matches!(retry, LabelError::AssessmentNotFound));

    let journal_path = temp_dir.path().join("journal.bin");
    assert!(
        journal_path.metadata().expect("journal should exist").len() > 0,
        "journaled-but-not-enqueued label should be durable"
    );

    drop(owner_block);
}

/// A model owner that has gone is reported as exactly that, not as a full channel
/// or a lost label. The two look alike from the sending side — nothing can be
/// enqueued either way — but one clears when the queue drains and the other never
/// will, so a host that cannot tell them apart would retry forever.
///
/// ´claim:errors:a-departed-model-owner-is-reported-to-the-labeller-as-its-own-distinct-failure´
/// ´test:integration:label-reports-model-owner-shutdown-after-owner-exit´
#[test]
fn label_reports_model_owner_shutdown_after_owner_exit() {
    let mut world = World::cold("label-owner-shutdown", 0x5754_4147_4532_0087);
    let reckoning = world.derive_default("alice").expect("derive default reckoning");
    let label = LabelSpec::adverse(reckoning.assessment.id).ground_truth().build();

    world.shut_down_model_owner_for_test();

    let err = world
        .label(label.clone())
        .expect_err("label should observe disconnected model owner");
    assert!(matches!(err, LabelError::ModelOwnerShutdown));

    let retry = world
        .label(label)
        .expect_err("pending entry is consumed before the enqueue failure is reported");
    assert!(matches!(retry, LabelError::AssessmentNotFound));
}

/// The durable append happens before the live enqueue is attempted, and a failed
/// enqueue rewrites nothing: after the owner has gone, the journal has grown by
/// the label while the last successful checkpoint is byte-for-byte what it was,
/// and retrying the consumed assessment appends no duplicate. Ordering the write
/// first is what makes the label survive the failure; leaving the checkpoint alone
/// is what stops a failed operation from being mistaken for a completed one.
///
/// ´claim:errors:a-label-is-made-durable-before-the-live-enqueue-is-attempted-and-a-failed-enqueue-rewrites-nothing´
/// ´test:integration:label-reports-model-owner-shutdown-with-journal-preserves-durable-record´
#[cfg(feature = "serde")]
#[test]
fn label_reports_model_owner_shutdown_with_journal_preserves_durable_record() {
    use std::fs;

    let temp_dir = tempfile::tempdir().expect("temp persistence dir");
    let config = AssayerConfig {
        instance_id: "label-owner-shutdown-journal".to_owned(),
        // The capacity is host-set with no default; the fixture declares it.
        infrastructure: torrust_assayer::testing::test_infrastructure(),
        ..Default::default()
    };
    let mut world = World::builder(config)
        .persistence_dirs(temp_dir.path(), temp_dir.path())
        .channel("default", ChannelPolicy::default())
        .seed(0x5754_4147_4532_0187)
        .build()
        .expect("journaled owner-shutdown world should build");

    world.flush_labels().expect("initial checkpoint should succeed");
    let checkpoint_path = temp_dir.path().join("checkpoint.bin");
    let journal_path = temp_dir.path().join("journal.bin");
    let checkpoint_before = fs::read(&checkpoint_path).expect("initial checkpoint should exist");
    let journal_before = fs::read(&journal_path).expect("initial journal should exist");
    assert_eq!(
        journal_before.len(),
        8,
        "initial checkpoint should truncate the journal to its format header"
    );

    let reckoning = world.derive_default("alice").expect("derive default reckoning");
    let label = LabelSpec::adverse(reckoning.assessment.id).ground_truth().build();

    world.shut_down_model_owner_for_test();

    let err = world
        .label(label.clone())
        .expect_err("label should observe disconnected model owner after journaling");
    assert!(matches!(err, LabelError::ModelOwnerShutdown));

    let checkpoint_after = fs::read(&checkpoint_path).expect("checkpoint should remain readable after shutdown failure");
    let journal_after = fs::read(&journal_path).expect("journal should remain readable after shutdown failure");
    assert_eq!(
        checkpoint_after, checkpoint_before,
        "failed live enqueue must not rewrite the last successful checkpoint",
    );
    assert!(
        journal_after.len() > journal_before.len(),
        "journal append should be durable before the disconnected send is reported",
    );

    let retry = world
        .label(label)
        .expect_err("pending entry is consumed even when the durable label cannot be live-enqueued");
    assert!(matches!(retry, LabelError::AssessmentNotFound));
    let journal_after_retry = fs::read(&journal_path).expect("journal should remain readable after retry");
    assert_eq!(
        journal_after_retry, journal_after,
        "retrying the consumed assessment id must not duplicate the durable journal entry",
    );
}

/// The promise the durable append makes is kept: an instance rebuilt from the
/// same directories replays the label written during the failed enqueue, counts it
/// among its processed labels, and truncates the journal once the replay has been
/// checkpointed. Durability that were never replayed would be a file on disk
/// rather than a label, and a journal never truncated would replay it again at
/// every restart.
///
/// ´claim:errors:a-label-journalled-during-a-failed-enqueue-is-replayed-and-then-cleared-on-restart´
/// ´test:integration:label-reports-model-owner-shutdown-journal-replays-on-restart´
#[cfg(feature = "serde")]
#[test]
fn label_reports_model_owner_shutdown_journal_replays_on_restart() {
    use std::fs;

    let temp_dir = tempfile::tempdir().expect("temp persistence dir");
    let checkpoint_path = temp_dir.path().join("checkpoint.bin");
    let journal_path = temp_dir.path().join("journal.bin");
    let instance_id = "label-owner-shutdown-replay";

    {
        let config = AssayerConfig {
            instance_id: instance_id.to_owned(),
            // The capacity is host-set with no default; the fixture declares it.
            infrastructure: torrust_assayer::testing::test_infrastructure(),
            ..Default::default()
        };
        let mut world = World::builder(config)
            .persistence_dirs(temp_dir.path(), temp_dir.path())
            .channel("default", ChannelPolicy::default())
            .seed(0x5754_4147_4532_0287)
            .build()
            .expect("journaled owner-shutdown world should build");

        world.flush_labels().expect("initial checkpoint should succeed");
        assert!(checkpoint_path.exists(), "initial checkpoint should exist");
        assert_eq!(
            fs::read(&journal_path).expect("initial journal should exist").len(),
            8,
            "initial checkpoint should truncate the journal to its format header",
        );

        let reckoning = world.derive_default("alice").expect("derive default reckoning");
        let label = LabelSpec::adverse(reckoning.assessment.id).ground_truth().build();

        world.shut_down_model_owner_for_test();

        let err = world
            .label(label)
            .expect_err("label should observe disconnected model owner after journaling");
        assert!(matches!(err, LabelError::ModelOwnerShutdown));
        assert!(
            journal_path.metadata().expect("shutdown journal should exist").len() > 8,
            "shutdown-written label should remain durable past the format header for restart replay",
        );
    }

    let config = AssayerConfig {
        instance_id: instance_id.to_owned(),
        // The capacity is host-set with no default; the fixture declares it.
        infrastructure: torrust_assayer::testing::test_infrastructure(),
        ..Default::default()
    };
    let restored = World::builder(config)
        .persistence_dirs(temp_dir.path(), temp_dir.path())
        .channel("default", ChannelPolicy::default())
        .seed(0x5754_4147_4532_0387)
        .build()
        .expect("restored world should build from checkpoint and journal");

    restored
        .flush_labels()
        .expect("shutdown-written journal entry should replay and checkpoint");

    let health = restored.assayer().health_summary();
    assert_eq!(health.total_labels, 1, "restored owner should process the replayed label");
    assert_eq!(
        fs::read(&journal_path)
            .expect("journal should remain readable after replay")
            .len(),
        8,
        "successful replay checkpoint should truncate the consumed journal entry",
    );
}

// ─────────────────────────────────────────────────────────────────────────────
// NaN utilities
// ─────────────────────────────────────────────────────────────────────────────

/// A value that is not a number is replaced by the caller's chosen default and
/// the substitution is reported alongside it. The checkpoint does not merely
/// clean the value: it says that it did, so a NaN entering the model is a health
/// event rather than a silently plausible zero.
///
/// ´claim:errors:a-non-finite-value-is-replaced-by-the-callers-default-and-the-substitution-is-reported´
/// ´test:integration:sanitise-f64-nan´
#[test]
fn sanitise_f64_nan() {
    assert_eq!(sanitise_f64(f64::NAN, 0.0), (0.0, true));
}

/// Overflow is scrubbed on the same terms as NaN: a positive infinity is
/// substituted and flagged. The checkpoint's test is finiteness rather than
/// not-a-number, so a variance that ran away to infinity is caught too.
///
/// (´claim:errors:a-non-finite-value-is-replaced-by-the-callers-default-and-the-substitution-is-reported´)
/// ´test:integration:sanitise-f64-positive-infinity´
#[test]
fn sanitise_f64_positive_infinity() {
    assert_eq!(sanitise_f64(f64::INFINITY, 0.0), (0.0, true));
}

/// The negative infinity is treated identically, so the scrub is symmetric: a
/// score that has run away downwards is no more acceptable than one that ran away
/// upwards.
///
/// (´claim:errors:a-non-finite-value-is-replaced-by-the-callers-default-and-the-substitution-is-reported´)
/// ´test:integration:sanitise-f64-negative-infinity´
#[test]
fn sanitise_f64_negative_infinity() {
    assert_eq!(sanitise_f64(f64::NEG_INFINITY, 0.0), (0.0, true));
}

/// An ordinary finite value passes through bit-for-bit and is not flagged. The
/// checkpoint is a filter on the pathological cases only — if it perturbed healthy
/// values it would be a source of drift rather than a guard against it.
///
/// ´claim:errors:a-finite-value-passes-through-the-checkpoint-unchanged-and-unflagged´
/// ´test:integration:sanitise-f64-normal-value´
#[test]
fn sanitise_f64_normal_value() {
    let (val, replaced) = sanitise_f64(2.72, 0.0);
    assert_eq!((val, replaced), (2.72, false));
    assert_finite(&[val], "sanitised normal value");
}

/// Zero survives even where the default offered was something else: the value is
/// kept and no substitution is reported. Zero is a real measurement — an axis
/// contributing nothing — and confusing it with the fallback would hide exactly
/// the case the flag exists to expose.
///
/// (´claim:errors:a-finite-value-passes-through-the-checkpoint-unchanged-and-unflagged´)
/// ´test:integration:sanitise-f64-zero´
#[test]
fn sanitise_f64_zero() {
    let (val, replaced) = sanitise_f64(0.0, 1.0);
    assert_eq!((val, replaced), (0.0, false));
    assert_finite(&[val], "sanitised zero");
}

/// Negative zero is finite and is left alone too, unflagged and with its sign bit
/// intact. The sign bit on a zero is an artefact of how it was arrived at, not a
/// defect, so the checkpoint has no business rewriting it, and the assertion is
/// made on the bit pattern because numeric equality cannot tell the two zeros
/// apart and so cannot see a rewrite at all.
///
/// (´claim:errors:a-finite-value-passes-through-the-checkpoint-unchanged-and-unflagged´)
/// ´test:integration:sanitise-f64-negative-zero´
#[test]
fn sanitise_f64_negative_zero() {
    let (val, replaced) = sanitise_f64(-0.0, 1.0);
    assert!(!replaced);
    assert_eq!(val.to_bits(), (-0.0f64).to_bits());
    assert_finite(&[val], "sanitised negative zero");
}

/// The replacement is whatever the caller nominated, not a fixed zero: a NaN
/// checked against a default of forty-two comes back as forty-two. Different
/// checkpoints guard quantities with different neutral values — a variance and a
/// score do not share one — so the fallback belongs to the call site.
///
/// (´claim:errors:a-non-finite-value-is-replaced-by-the-callers-default-and-the-substitution-is-reported´)
/// ´test:integration:sanitise-f64-custom-default´
#[test]
fn sanitise_f64_custom_default() {
    let (val, replaced) = sanitise_f64(f64::NAN, 42.0);
    assert_eq!((val, replaced), (42.0, true));
    assert_finite(&[val], "sanitised NaN → custom default");
}

/// A subnormal — smaller than the smallest normal value but still not zero —
/// is finite, and so is kept exactly as it is. Underflow toward the denormal range
/// is what a converged variance looks like, and rounding it away at the checkpoint
/// would destroy precisely the state the model worked hardest to reach.
///
/// (´claim:errors:a-finite-value-passes-through-the-checkpoint-unchanged-and-unflagged´)
/// ´test:integration:sanitise-f64-subnormal´
#[test]
fn sanitise_f64_subnormal() {
    let subnormal = f64::MIN_POSITIVE / 2.0;
    assert!(subnormal != 0.0, "subnormal should be non-zero");
    let (val, replaced) = sanitise_f64(subnormal, 0.0);
    assert!(!replaced);
    assert_eq!(val.to_bits(), subnormal.to_bits());
    assert_finite(&[val], "sanitised subnormal");
}

/// Scanning a slice with nothing wrong in it yields no indices at all, and the
/// empty slice is the degenerate instance of that. A spot-check over state that
/// does not exist yet reports health rather than failing, so a cold model needs no
/// special case around the checkpoints.
///
/// ´claim:errors:a-slice-holding-no-non-finite-values-yields-an-empty-index-list´
/// ´test:integration:check-vec-nan-empty´
#[test]
fn check_vec_nan_empty() {
    assert_eq!(check_vec_nan(&[]), [] as [usize; 0]);
    assert_finite(&[], "empty slice");
}

/// A populated slice of healthy values reports the same emptiness. Silence from
/// the scan means health, not that it declined to look, which is what lets a host
/// treat a non-empty result as an unambiguous alarm.
///
/// (´claim:errors:a-slice-holding-no-non-finite-values-yields-an-empty-index-list´)
/// ´test:integration:check-vec-nan-all-clean´
#[test]
fn check_vec_nan_all_clean() {
    let clean = [1.0, 2.0, 3.0];
    assert_eq!(check_vec_nan(&clean), [] as [usize; 0]);
    assert_finite(&clean, "clean slice");
}

/// The scan reports the position of every non-finite element, in ascending order,
/// and passes over the healthy ones between them — NaN and both infinities alike.
/// Positions are what make the finding actionable: they name which features or
/// which axes went bad, not merely that something did.
///
/// ´claim:errors:the-scan-reports-the-ascending-positions-of-every-non-finite-element´
/// ´test:integration:check-vec-nan-mixed´
#[test]
fn check_vec_nan_mixed() {
    let data = [1.0, f64::NAN, 3.0, f64::INFINITY, 5.0, f64::NEG_INFINITY];
    assert_eq!(check_vec_nan(&data), vec![1, 3, 5]);
}

/// When nothing in the slice is finite, every position is reported. The scan does
/// not stop at the first fault, so a wholesale collapse is visible as such rather
/// than as a single index that suggests an isolated one.
///
/// (´claim:errors:the-scan-reports-the-ascending-positions-of-every-non-finite-element´)
/// ´test:integration:check-vec-nan-all-bad´
#[test]
fn check_vec_nan_all_bad() {
    let data = [f64::NAN, f64::INFINITY, f64::NEG_INFINITY];
    assert_eq!(check_vec_nan(&data), vec![0, 1, 2]);
}

/// A lone fault surrounded by healthy values is found and reported at its own
/// index. Neither end of the slice is privileged, so a scan is a genuine sweep and
/// not a check of the boundaries.
///
/// (´claim:errors:the-scan-reports-the-ascending-positions-of-every-non-finite-element´)
/// ´test:integration:check-vec-nan-single-middle´
#[test]
fn check_vec_nan_single_middle() {
    assert_eq!(check_vec_nan(&[1.0, f64::NAN, 3.0]), vec![1]);
}

// ─────────────────────────────────────────────────────────────────────────────
// Diagonal NaN utilities
// ─────────────────────────────────────────────────────────────────────────────

/// Checking a covariance diagonal answers exactly as checking the same numbers as
/// a plain slice would: a healthy diagonal reports no positions. The diagonal
/// entry point exists to say what is being spot-checked, not to apply a different
/// standard of health to a matrix than to a vector.
///
/// ´claim:errors:scanning-a-matrix-diagonal-answers-exactly-as-scanning-the-same-slice-would´
/// ´test:integration:check-diagonal-nan-clean´
#[test]
fn check_diagonal_nan_clean() {
    let diag = [1.0, 2.0, 3.0];
    assert_eq!(check_diagonal_nan(&diag), [] as [usize; 0]);
    assert_finite(&diag, "clean diagonal");
}

/// A NaN sitting on the diagonal is reported at its position, which names the
/// feature whose variance has gone bad. That is the cheapest useful spot-check on
/// a covariance matrix: a diagonal entry is a variance, and a variance that is not
/// a number will spread through every quadratic form the matrix takes part in.
///
/// (´claim:errors:scanning-a-matrix-diagonal-answers-exactly-as-scanning-the-same-slice-would´)
/// ´test:integration:check-diagonal-nan-with-nan´
#[test]
fn check_diagonal_nan_with_nan() {
    let diag = [1.0, f64::NAN, 3.0];
    assert_eq!(check_diagonal_nan(&diag), vec![1]);
}
