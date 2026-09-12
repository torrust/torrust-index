// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`cold_start_dimensions`] | snapshot | The operational and sister models are built to the feature width they were asked for, while the anchor keeps its own fixed, much smaller width. The anchor lives in a deliberately chosen projection of the feature space rather than the whole of it, which is what lets it converge on local evidence long before a full-width model could. |
//! | [`cold_start_defaults`] | snapshot | A cold-start model starts uncommitted in every direction it could be committed: all three calibration factors sit at unity, both base rates at even odds, and no label has yet been consumed. Every one of those is a quantity learned from outcomes, so beginning anywhere other than neutral would amount to a belief the system was never given evidence for. |
//! | [`cold_start_outcome_models_empty`] | snapshot | No outcome axis exists until a host declares one. What counts as an adverse outcome is a property of the host's domain rather than of the model, so the system opens with no axes at all instead of a guess at what the host cares to predict. |
//! | [`cold_start_optional_fields_present`] | snapshot | Every optional part of a snapshot is present from the first publication, holding its neutral value rather than being absent: drift state is an empty map, and the feature statistics are a full-length vector of zero means and unit variances sized to the layout they standardise. A reader therefore never has to distinguish "not yet learned" from "missing" — standardising against these values is the identity, so the same code path serves a day-old model and a minute-old one. |
//! | [`to_snapshot_version`] | snapshot | A snapshot is stamped with the version its publisher assigns, carried through unaltered. Versioning belongs to the publisher because only it knows the order in which its own publications happened; the snapshot's job is to remember which one it is so a reader can say what it read. |
//! | [`to_snapshot_version_increments`] | snapshot | Two publications of the same working copy are still distinguishable, and they differ in the direction time runs. Publishing does not consume or alter the working copy, so the same state can be published repeatedly; the version is what lets a reader tell a re-publication apart from the one it already holds. |
//! | [`to_snapshot_carries_timestamp`] | snapshot | A snapshot records the instant it was published, taken at publication and not inherited from whenever the working copy last changed. Readers inflate their uncertainty by how stale the model they are reading has become, so that instant is the origin of the ageing they apply; a timestamp from any other moment would make an old snapshot look fresh. |
//! | [`to_snapshot_parameters`] | snapshot | Publishing carries all three models across at once — operational, sister and anchor — each with its own mean vector and its own width. Readers blend the three against one another, so they must come from a single publication or the blend would weigh models learned at different moments against each other. |
//! | [`to_snapshot_scalars`] | snapshot | The calibration scalars the writer has learned — the per-model Platt factors and both base rates — cross into the snapshot as they stand, not as defaults. Turning a log-odds score into a probability is exactly what these numbers do, so a reader given the parameters but not the calibration would report confidently miscalibrated probabilities from correct arithmetic. |
//! | [`to_snapshot_dimension_map`] | snapshot | The width of the feature layout is decided by what is registered — the bias and the aggregate block at cold start — and not by the width the model happened to be constructed with. Asking for a wide model does not conjure features to fill it: the layout grows only when a Sentinel, a dimension, or a signal schema actually adds something to measure. |
//! | [`roundtrip_mu_identical`] | snapshot | A working copy rebuilt from a snapshot holds the same mean vectors it started with, for all three models, element for element. The means are the learned beliefs themselves; a restore that perturbed them would silently discard training every time a process restarted from a checkpoint. |
//! | [`roundtrip_sigma_identical`] | snapshot | The covariances survive the round trip to bit identity, despite passing through a flat column-major representation on the way out and a reconstruction on the way back. Covariance is how much the model doubts itself, and it also weights the blend; drift introduced by the storage format alone would move estimates without any evidence having arrived. |
//! | [`roundtrip_b_reconstructed`] | snapshot | A snapshot stores covariance alone, and restoring rebuilds the precision matrix from it as a genuine inverse: the product of the two comes back to the identity for every model. Only one of the pair need be persisted since each determines the other, but the two must agree once restored — updates are applied in precision space and read out in covariance space, so a mismatched pair would corrupt learning from the first label onwards. |
//! | [`from_snapshot_resets_last_label_time`] | snapshot | Restoring restarts the decay clock at the moment of restore rather than resuming whatever the clock read when the snapshot was taken. Elapsed time is what the model decays its confidence against, and a process may have been down for an unknown stretch; carrying the old mark forward would let the first label after a restart be dated against an instant from a previous run. |
//! | [`arcswap_load_returns_guard`] | snapshot | Reading the model is an unconditional load that hands back the snapshot currently published — no lock is taken and no waiting is possible. Every assessment begins with this read, so making it wait on the writer would put training latency directly into the request path. |
//! | [`arcswap_store_load_new_version`] | snapshot | Publishing replaces what subsequent readers get: a load after a store returns the newly published snapshot, not the one it superseded. This is how learning reaches the request path at all — the writer never mutates what readers hold, it substitutes the whole thing. |
//! | [`arcswap_two_snapshots_alive`] | snapshot | A reader that already holds a snapshot goes on reading that same snapshot after the writer publishes a newer one, while a fresh load sees the new version: the two coexist for as long as anyone still holds the older. An assessment in progress therefore reads one coherent model from start to finish, and the writer never has to wait for readers to finish before it can publish. |
//! | [`model_snapshot_send_sync`] | snapshot | A published snapshot can be sent between threads and shared by many at once. That is what makes one published model serviceable by an entire fleet of request threads from a single shared reference, instead of each thread needing a copy or a turn at a lock. |
//! | [`arcswap_guard_drops_cleanly`] | snapshot | Letting go of a snapshot leaves the mechanism intact: later loads still succeed, and a publication made after the release is visible to them. Readers acquire and release constantly — once per assessment — so a release that damaged the shared cell, or a stale reference that kept an old version authoritative, would break the model within the first few requests. |
//! | [`concurrent_readers_and_writer`] | snapshot | Under a thousand publications racing four continuously reading threads, no reader ever sees the version go backwards, and the last publication is what remains once the dust settles. Time only runs forwards for a reader: it may miss versions when the writer outpaces it, but it can never be handed an older model than one it has already seen, so a host cannot observe the system apparently unlearning. |
//! | [`snapshot_has_dimension_map`] | snapshot | A registered Sentinel earns its own slot in the published feature layout, and the layout carries one entry per Sentinel. The slot is where that Sentinel's extracted features live in the assembled vector, so a reader can attribute a coefficient to the Sentinel that produced the feature it weights. |
//! | [`snapshot_dimension_map_serde`] | snapshot | The published layout survives serialisation whole: total width, the per-Sentinel slots, the per-dimension ranges, the cross-dimension block and the competitive-cell range all come back as they went in. A restored model interprets feature positions by this map, so a layout that changed shape in storage would silently pair every learned coefficient with the wrong feature. |
//! | [`snapshot_with_outcome_models`] | snapshot | A registered outcome axis is published with more than its parameters: its own calibration factor and its spatial flag travel with it. Readers need both to use the axis at all — the factor to put the prediction back on the host's scale, and the flag to know whether the axis participates in per-Sentinel spatial extraction. |
//! | [`snapshot_working_copy_gains_dim_map`] | snapshot | cites (´claim:snapshot:the-feature-layout-width-follows-what-is-registered-not-the-models-parameter-width´) |
//! | [`snapshot_with_full_dim_map_serde`] | snapshot | cites (´claim:snapshot:the-feature-layout-survives-serialisation-so-a-restored-model-reads-features-at-the-same-offsets´) |
//! | [`working_copy_dim_map_consistent`] | snapshot | The cold-start layout is a definite arrangement, not merely a width: the bias occupies the first position and the fifteen cross-Sentinel aggregates follow it. Every later registration appends beyond that block, so these fixed positions are what let the same feature keep the same index as the layout grows. |
//! | [`snapshot_version_survives_format_extension`] | snapshot | A checkpoint payload round-trips the fields the format has grown, not only the ones it started with: the feature layout, the per-Sentinel ledger and identity state, and the label sequence mark all come back. The payload is how a process resumes across a restart, so a field added later but dropped in transit would be invisible in every test except one that restores and looks for it. |
//! | [`roundtrip_floor_masses_identical`] | snapshot | A model reverted from a snapshot comes back carrying the floor masses it had when the snapshot was published, bit for bit and in both shapes. The masses are the one thing about the covariance that the covariance cannot say: they record how much of the precision behind it was borrowed from the floors rather than earned from labels, and nothing in Σ distinguishes the two. A revert that dropped them would hand back models reading as though every unit of that precision had been earned — the direction that under-reports uncertainty, on the one path taken precisely because something has already gone wrong. The masses planted here are non-zero and distinct per model and per coordinate, so a restore that reset them or crossed them between models fails rather than agreeing by accident. |

//! Crate-level tests for the snapshot module (´dec:concurrency:snapshot-swap´).
//!
//! The writer keeps a mutable working copy; readers never touch it. What
//! readers see is a published snapshot — an immutable projection of the
//! working copy, stamped with a version and a publication instant — swapped in
//! atomically. A reader holding a snapshot keeps reading it unchanged while
//! the writer publishes the next, so no assessment ever blocks on training and
//! none ever observes a half-written model.

#![allow(clippy::float_cmp, clippy::suboptimal_flops)]

use std::sync::Arc;

use crate::linalg::convert::{col_to_vec, mat_to_vec};
use crate::model::bayesian::BayesianLinearModel;
use crate::snapshot::published::ModelSnapshot;
use crate::snapshot::shared::SharedState;
use crate::snapshot::working::{ModelConfig, WorkingCopy};
use crate::testing::{DEFAULT_TOLERANCES, assert_near};

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Default model config for tests.
fn test_config() -> ModelConfig {
    ModelConfig::default()
}

/// Compute Frobenius norm of ‖AB − I‖ to check BΣ ≈ I.
fn bs_minus_i_frobenius(model: &BayesianLinearModel) -> f64 {
    let p = model.dim();
    let b = model.precision().as_inner();
    let s = model.covariance().as_inner();
    let mut sum = 0.0;
    for i in 0..p {
        for j in 0..p {
            let mut bs_ij = 0.0;
            for k in 0..p {
                bs_ij += b[(i, k)] * s[(k, j)];
            }
            let expected = if i == j { 1.0 } else { 0.0 };
            let diff = bs_ij - expected;
            sum += diff * diff;
        }
    }
    sum.sqrt()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Cold start & basic construction
// ═══════════════════════════════════════════════════════════════════════════════

/// The operational and sister models are built to the feature width they were
/// asked for, while the anchor keeps its own fixed, much smaller width. The
/// anchor lives in a deliberately chosen projection of the feature space
/// rather than the whole of it, which is what lets it converge on local
/// evidence long before a full-width model could.
///
/// ´claim:snapshot:the-anchor-model-has-its-own-fixed-width-independent-of-the-feature-dimension´
/// ´test:crate:cold-start-dimensions´
#[test]
fn cold_start_dimensions() {
    let wc = WorkingCopy::cold_start(100, &test_config(), 1000);
    assert_eq!(wc.operational.dim(), 100);
    assert_eq!(wc.sister.dim(), 100);
    assert_eq!(wc.anchor.dim(), 15);
}

/// A cold-start model starts uncommitted in every direction it could be
/// committed: all three calibration factors sit at unity, both base rates at
/// even odds, and no label has yet been consumed. Every one of those is a
/// quantity learned from outcomes, so beginning anywhere other than neutral
/// would amount to a belief the system was never given evidence for.
///
/// ´claim:snapshot:a-cold-start-model-begins-uncalibrated-at-even-odds-with-no-labels-consumed´
/// ´test:crate:cold-start-defaults´
#[test]
fn cold_start_defaults() {
    let wc = WorkingCopy::cold_start(100, &test_config(), 1000);
    assert_eq!(wc.kappa_sister, 1.0);
    assert_eq!(wc.kappa_anchor, 1.0);
    assert_eq!(wc.kappa_v, 1.0);
    assert_eq!(wc.p_positive_global, 0.5);
    assert_eq!(wc.p_positive_eligible, 0.5);
    assert_eq!(wc.last_processed_label_seq, 0);
}

/// No outcome axis exists until a host declares one. What counts as an adverse
/// outcome is a property of the host's domain rather than of the model, so the
/// system opens with no axes at all instead of a guess at what the host cares
/// to predict.
///
/// ´claim:snapshot:outcome-axes-exist-only-once-a-host-declares-them´
/// ´test:crate:cold-start-outcome-models-empty´
#[test]
fn cold_start_outcome_models_empty() {
    let wc = WorkingCopy::cold_start(100, &test_config(), 1000);
    assert!(wc.outcome_models.is_empty());
}

/// Every optional part of a snapshot is present from the first publication,
/// holding its neutral value rather than being absent: drift state is an empty
/// map, and the feature statistics are a full-length vector of zero means and
/// unit variances sized to the layout they standardise. A reader therefore
/// never has to distinguish "not yet learned" from "missing" — standardising
/// against these values is the identity, so the same code path serves a
/// day-old model and a minute-old one.
///
/// ´claim:snapshot:a-cold-start-snapshot-carries-neutral-standardisation-sized-to-the-layout-rather-than-absent-fields´
/// ´test:crate:cold-start-optional-fields-present´
#[test]
fn cold_start_optional_fields_present() {
    let wc = WorkingCopy::cold_start(50, &test_config(), 1000);
    let snap = wc.to_snapshot(1);

    // Calibration buffer is default.
    let _ = &snap.calibration_buffer; // type is CalibrationBufferSnapshot

    // Per-model drift state — present but empty.
    assert!(snap.drift.is_empty());

    // Feature statistics — initialised to neutral standardisation.
    assert_eq!(snap.feature_means.len(), snap.dimension_map.p);
    assert!(snap.feature_means.iter().all(|&m| m == 0.0));
    assert_eq!(snap.feature_variances.len(), snap.dimension_map.p);
    assert!(snap.feature_variances.iter().all(|&v| v == 1.0));

    // Outcome models — present but empty.
    assert!(snap.outcome_models.is_empty());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Snapshot projection
// ═══════════════════════════════════════════════════════════════════════════════

/// A snapshot is stamped with the version its publisher assigns, carried
/// through unaltered. Versioning belongs to the publisher because only it
/// knows the order in which its own publications happened; the snapshot's job
/// is to remember which one it is so a reader can say what it read.
///
/// ´claim:snapshot:a-snapshot-is-stamped-with-the-version-its-publisher-assigns´
/// ´test:crate:to-snapshot-version´
#[test]
fn to_snapshot_version() {
    let wc = WorkingCopy::cold_start(50, &test_config(), 1000);
    let snap = wc.to_snapshot(42);
    assert_eq!(snap.version, 42);
}

/// Two publications of the same working copy are still distinguishable, and
/// they differ in the direction time runs. Publishing does not consume or
/// alter the working copy, so the same state can be published repeatedly; the
/// version is what lets a reader tell a re-publication apart from the one it
/// already holds.
///
/// ´claim:snapshot:successive-publications-of-one-working-copy-remain-distinguishable-and-ordered´
/// ´test:crate:to-snapshot-version-increments´
#[test]
fn to_snapshot_version_increments() {
    let wc = WorkingCopy::cold_start(50, &test_config(), 1000);
    let snap_a = wc.to_snapshot(1);
    let snap_b = wc.to_snapshot(2);
    assert_ne!(snap_a.version, snap_b.version);
    assert_eq!(snap_b.version, snap_a.version + 1);
}

/// A snapshot records the instant it was published, taken at publication and
/// not inherited from whenever the working copy last changed. Readers inflate
/// their uncertainty by how stale the model they are reading has become, so
/// that instant is the origin of the ageing they apply; a timestamp from any
/// other moment would make an old snapshot look fresh.
///
/// ´claim:snapshot:a-snapshot-records-the-instant-it-was-published-so-readers-can-age-its-uncertainty´
/// ´test:crate:to-snapshot-carries-timestamp´
#[test]
fn to_snapshot_carries_timestamp() {
    let before = std::time::Instant::now();
    let wc = WorkingCopy::cold_start(50, &test_config(), 1000);
    let snap = wc.to_snapshot(1);
    let after = std::time::Instant::now();

    // publish_timestamp should be between `before` and `after`.
    assert!(snap.publish_timestamp >= before);
    assert!(snap.publish_timestamp <= after);
}

/// Publishing carries all three models across at once — operational, sister
/// and anchor — each with its own mean vector and its own width. Readers blend
/// the three against one another, so they must come from a single publication
/// or the blend would weigh models learned at different moments against each
/// other.
///
/// ´claim:snapshot:one-publication-carries-all-three-models-parameters-and-widths-together´
/// ´test:crate:to-snapshot-parameters´
#[test]
fn to_snapshot_parameters() {
    let wc = WorkingCopy::cold_start(50, &test_config(), 1000);
    let snap = wc.to_snapshot(1);

    // μ should be zero for all cold-start models.
    assert!(snap.operational.mu.iter().all(|&v| v == 0.0));
    assert!(snap.sister.mu.iter().all(|&v| v == 0.0));
    assert!(snap.anchor.mu.iter().all(|&v| v == 0.0));

    // Dimensions should match.
    assert_eq!(snap.operational.p, 50);
    assert_eq!(snap.sister.p, 50);
    assert_eq!(snap.anchor.p, 15);
}

/// The calibration scalars the writer has learned — the per-model Platt
/// factors and both base rates — cross into the snapshot as they stand, not as
/// defaults. Turning a log-odds score into a probability is exactly what these
/// numbers do, so a reader given the parameters but not the calibration would
/// report confidently miscalibrated probabilities from correct arithmetic.
///
/// ´claim:snapshot:calibration-scalars-cross-into-the-snapshot-so-readers-calibrate-with-the-writers-learned-values´
/// ´test:crate:to-snapshot-scalars´
#[test]
fn to_snapshot_scalars() {
    let mut wc = WorkingCopy::cold_start(50, &test_config(), 1000);
    wc.p_positive_global = 0.3;
    wc.p_positive_eligible = 0.7;
    wc.kappa_v = 2.5;
    wc.kappa_sister = 3.0;
    wc.kappa_anchor = 1.5;

    let snap = wc.to_snapshot(1);
    assert_eq!(snap.p_positive_global, 0.3);
    assert_eq!(snap.p_positive_eligible, 0.7);
    assert_eq!(snap.kappa_v, 2.5);
    assert_eq!(snap.kappa_sister, 3.0);
    assert_eq!(snap.kappa_anchor, 1.5);
}

/// The width of the feature layout is decided by what is registered — the bias
/// and the aggregate block at cold start — and not by the width the model
/// happened to be constructed with. Asking for a wide model does not conjure
/// features to fill it: the layout grows only when a Sentinel, a dimension, or
/// a signal schema actually adds something to measure.
///
/// ´claim:snapshot:the-feature-layout-width-follows-what-is-registered-not-the-models-parameter-width´
/// ´test:crate:to-snapshot-dimension-map´
#[test]
fn to_snapshot_dimension_map() {
    let wc = WorkingCopy::cold_start(73, &test_config(), 1000);
    let snap = wc.to_snapshot(1);
    // DimensionMap.p reflects structural layout (cold start: bias(1) + agg(15) = 16),
    // not the BLM operational dimension.
    assert_eq!(snap.dimension_map.p, 16);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Snapshot round-trip
// ═══════════════════════════════════════════════════════════════════════════════

/// A working copy rebuilt from a snapshot holds the same mean vectors it
/// started with, for all three models, element for element. The means are the
/// learned beliefs themselves; a restore that perturbed them would silently
/// discard training every time a process restarted from a checkpoint.
///
/// ´claim:snapshot:restoring-from-a-snapshot-returns-every-models-mean-vector-unchanged´
/// ´test:crate:roundtrip-mu-identical´
#[test]
fn roundtrip_mu_identical() {
    let wc = WorkingCopy::cold_start(50, &test_config(), 1000);
    let snap = wc.to_snapshot(1);
    let config = test_config();

    let restored = WorkingCopy::from_snapshot(&snap, &config, 1000, 100).unwrap();

    // μ should be bit-identical (zero → zero).
    let orig_mu = col_to_vec(wc.operational.mu());
    let rest_mu = col_to_vec(restored.operational.mu());
    assert_eq!(orig_mu, rest_mu);

    let orig_mu_s = col_to_vec(wc.sister.mu());
    let rest_mu_s = col_to_vec(restored.sister.mu());
    assert_eq!(orig_mu_s, rest_mu_s);

    let orig_mu_a = col_to_vec(wc.anchor.mu());
    let rest_mu_a = col_to_vec(restored.anchor.mu());
    assert_eq!(orig_mu_a, rest_mu_a);
}

/// The covariances survive the round trip to bit identity, despite passing
/// through a flat column-major representation on the way out and a
/// reconstruction on the way back. Covariance is how much the model doubts
/// itself, and it also weights the blend; drift introduced by the storage
/// format alone would move estimates without any evidence having arrived.
///
/// ´claim:snapshot:the-storage-representation-round-trips-covariance-without-perturbing-it´
/// ´test:crate:roundtrip-sigma-identical´
#[test]
fn roundtrip_sigma_identical() {
    let wc = WorkingCopy::cold_start(50, &test_config(), 1000);
    let snap = wc.to_snapshot(1);
    let config = test_config();

    let restored = WorkingCopy::from_snapshot(&snap, &config, 1000, 100).unwrap();

    // Σ should be bit-identical (through col-major → from-params path).
    let orig_sigma = mat_to_vec(wc.operational.covariance().as_inner());
    let rest_sigma = mat_to_vec(restored.operational.covariance().as_inner());
    for (a, b) in orig_sigma.iter().zip(rest_sigma.iter()) {
        assert_near(*a, *b, DEFAULT_TOLERANCES.bit_identical, "Σ (operational)");
    }

    let orig_sigma_a = mat_to_vec(wc.anchor.covariance().as_inner());
    let rest_sigma_a = mat_to_vec(restored.anchor.covariance().as_inner());
    for (a, b) in orig_sigma_a.iter().zip(rest_sigma_a.iter()) {
        assert_near(*a, *b, DEFAULT_TOLERANCES.bit_identical, "Σ (anchor)");
    }
}

/// A snapshot stores covariance alone, and restoring rebuilds the precision
/// matrix from it as a genuine inverse: the product of the two comes back to
/// the identity for every model. Only one of the pair need be persisted since
/// each determines the other, but the two must agree once restored — updates
/// are applied in precision space and read out in covariance space, so a
/// mismatched pair would corrupt learning from the first label onwards.
///
/// ´claim:snapshot:restoring-rebuilds-the-precision-matrix-as-a-genuine-inverse-of-the-restored-covariance´
/// ´test:crate:roundtrip-b-reconstructed´
#[test]
fn roundtrip_b_reconstructed() {
    let wc = WorkingCopy::cold_start(50, &test_config(), 1000);
    let snap = wc.to_snapshot(1);
    let config = test_config();

    let restored = WorkingCopy::from_snapshot(&snap, &config, 1000, 100).unwrap();

    // ‖BΣ − I‖_F should be very small for all three models, and each is held
    // to the specification's threshold at its own width rather than to one
    // flat ceiling — the anchor is far narrower than the other two, so a
    // shared figure would hold it to the loosest of the three
    // (´def:monitoring:synchronisation-error´).
    for (model, name) in [
        (&restored.operational, "operational"),
        (&restored.sister, "sister"),
        (&restored.anchor, "anchor"),
    ] {
        assert_near(
            bs_minus_i_frobenius(model),
            0.0,
            DEFAULT_TOLERANCES.precision_sync_ceiling(model.dim()),
            &format!("‖BΣ − I‖_F ({name})"),
        );
    }
}

/// Restoring restarts the decay clock at the moment of restore rather than
/// resuming whatever the clock read when the snapshot was taken. Elapsed time
/// is what the model decays its confidence against, and a process may have
/// been down for an unknown stretch; carrying the old mark forward would let
/// the first label after a restart be dated against an instant from a previous
/// run.
///
/// ´claim:snapshot:restoring-restarts-the-decay-clock-at-the-moment-of-restore´
/// ´test:crate:from-snapshot-resets-last-label-time´
#[test]
fn from_snapshot_resets_last_label_time() {
    let wc = WorkingCopy::cold_start(50, &test_config(), 1000);
    let snap = wc.to_snapshot(1);
    let config = test_config();

    let before = std::time::Instant::now();
    let restored = WorkingCopy::from_snapshot(&snap, &config, 1000, 100).unwrap();
    let after = std::time::Instant::now();

    // last_label_time should be set to approximately Instant::now().
    assert!(restored.last_label_time >= before);
    assert!(restored.last_label_time <= after);
}

// ═══════════════════════════════════════════════════════════════════════════════
// ArcSwap concurrency
// ═══════════════════════════════════════════════════════════════════════════════

/// Reading the model is an unconditional load that hands back the snapshot
/// currently published — no lock is taken and no waiting is possible. Every
/// assessment begins with this read, so making it wait on the writer would put
/// training latency directly into the request path.
///
/// ´claim:snapshot:reading-the-model-is-an-unconditional-load-that-cannot-block-on-the-writer´
/// ´test:crate:arcswap-load-returns-guard´
#[test]
fn arcswap_load_returns_guard() {
    let wc = WorkingCopy::cold_start(10, &test_config(), 1000);
    let snap = wc.to_snapshot(1);
    let shared = SharedState::new(snap);

    let guard = shared.published.load();
    assert_eq!(guard.version, 1);
}

/// Publishing replaces what subsequent readers get: a load after a store
/// returns the newly published snapshot, not the one it superseded. This is
/// how learning reaches the request path at all — the writer never mutates
/// what readers hold, it substitutes the whole thing.
///
/// ´claim:snapshot:publishing-substitutes-the-whole-model-so-later-readers-load-the-new-one´
/// ´test:crate:arcswap-store-load-new-version´
#[test]
fn arcswap_store_load_new_version() {
    let wc = WorkingCopy::cold_start(10, &test_config(), 1000);
    let snap1 = wc.to_snapshot(1);
    let shared = SharedState::new(snap1);

    let snap2 = wc.to_snapshot(2);
    shared.published.store(Arc::new(snap2));

    let guard = shared.published.load();
    assert_eq!(guard.version, 2);
}

/// A reader that already holds a snapshot goes on reading that same snapshot
/// after the writer publishes a newer one, while a fresh load sees the new
/// version: the two coexist for as long as anyone still holds the older. An
/// assessment in progress therefore reads one coherent model from start to
/// finish, and the writer never has to wait for readers to finish before it
/// can publish.
///
/// ´claim:snapshot:a-reader-keeps-reading-the-snapshot-it-holds-while-the-writer-publishes-the-next´
/// ´test:crate:arcswap-two-snapshots-alive´
#[test]
fn arcswap_two_snapshots_alive() {
    let wc = WorkingCopy::cold_start(10, &test_config(), 1000);
    let snap1 = wc.to_snapshot(1);
    let shared = SharedState::new(snap1);

    // Load a guard to keep version 1 alive.
    let guard_v1 = shared.published.load();
    assert_eq!(guard_v1.version, 1);

    // Store version 2.
    let snap2 = wc.to_snapshot(2);
    shared.published.store(Arc::new(snap2));

    // guard_v1 still sees version 1.
    assert_eq!(guard_v1.version, 1);

    // New load sees version 2.
    let guard_v2 = shared.published.load();
    assert_eq!(guard_v2.version, 2);

    // Both are alive — Arc::strong_count confirms at least 2
    // references exist (one from ArcSwap internals, one from the guard).
    // We simply verify that both guards return correct versions,
    // which proves two snapshots coexist.
}

/// A published snapshot can be sent between threads and shared by many at
/// once. That is what makes one published model serviceable by an entire fleet
/// of request threads from a single shared reference, instead of each thread
/// needing a copy or a turn at a lock.
///
/// ´claim:snapshot:a-published-snapshot-can-be-shared-by-any-number-of-threads-at-once´
/// ´test:crate:model-snapshot-send-sync´
#[test]
fn model_snapshot_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ModelSnapshot>();
}

/// Letting go of a snapshot leaves the mechanism intact: later loads still
/// succeed, and a publication made after the release is visible to them.
/// Readers acquire and release constantly — once per assessment — so a release
/// that damaged the shared cell, or a stale reference that kept an old version
/// authoritative, would break the model within the first few requests.
///
/// ´claim:snapshot:releasing-a-snapshot-disturbs-neither-later-loads-nor-later-publications´
/// ´test:crate:arcswap-guard-drops-cleanly´
#[test]
fn arcswap_guard_drops_cleanly() {
    let wc = WorkingCopy::cold_start(10, &test_config(), 1000);
    let snap = wc.to_snapshot(1);
    let shared = SharedState::new(snap);

    // Load a guard and drop it.
    {
        let guard = shared.published.load();
        assert_eq!(guard.version, 1);
    }
    // Dropped — subsequent loads should still work.
    let guard2 = shared.published.load();
    assert_eq!(guard2.version, 1);

    // Store a new version after the first guard was dropped.
    let snap2 = wc.to_snapshot(2);
    shared.published.store(Arc::new(snap2));
    let guard3 = shared.published.load();
    assert_eq!(guard3.version, 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Concurrency smoke test
// ═══════════════════════════════════════════════════════════════════════════════

/// Under a thousand publications racing four continuously reading threads, no
/// reader ever sees the version go backwards, and the last publication is what
/// remains once the dust settles. Time only runs forwards for a reader: it may
/// miss versions when the writer outpaces it, but it can never be handed an
/// older model than one it has already seen, so a host cannot observe the
/// system apparently unlearning.
///
/// ´claim:snapshot:readers-may-skip-versions-but-never-see-the-published-model-go-backwards´
/// ´test:crate:concurrent-readers-and-writer´
#[test]
fn concurrent_readers_and_writer() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;

    const NUM_READERS: usize = 4;
    const NUM_WRITES: u64 = 1_000;

    let config = test_config();
    let wc = WorkingCopy::cold_start(10, &config, 1000);
    let snap = wc.to_snapshot(0);
    let shared = Arc::new(SharedState::new(snap));
    let done = Arc::new(AtomicBool::new(false));

    // Spawn reader threads.
    let readers: Vec<_> = (0..NUM_READERS)
        .map(|_| {
            let shared = Arc::clone(&shared);
            let done = Arc::clone(&done);
            thread::spawn(move || {
                let mut last_version = 0_u64;
                while !done.load(Ordering::Relaxed) {
                    let guard = shared.published.load();
                    let v = guard.version;
                    // Versions should be monotonically non-decreasing.
                    assert!(v >= last_version, "version went backwards: {last_version} → {v}");
                    last_version = v;
                }
                last_version
            })
        })
        .collect();

    // Writer: store 1,000 incremented snapshots.
    for i in 1..=NUM_WRITES {
        let new_snap = wc.to_snapshot(i);
        shared.published.store(Arc::new(new_snap));
    }

    done.store(true, Ordering::Relaxed);

    // All readers should have seen at least version 0 and at most NUM_WRITES.
    for handle in readers {
        let last = handle.join().unwrap();
        assert!(last <= NUM_WRITES, "reader saw version {last} > {NUM_WRITES}");
    }

    // Final load should see the last version.
    let final_guard = shared.published.load();
    assert_eq!(final_guard.version, NUM_WRITES);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Snapshot Extension
// ═══════════════════════════════════════════════════════════════════════════════

/// A registered Sentinel earns its own slot in the published feature layout,
/// and the layout carries one entry per Sentinel. The slot is where that
/// Sentinel's extracted features live in the assembled vector, so a reader can
/// attribute a coefficient to the Sentinel that produced the feature it
/// weights.
///
/// ´claim:snapshot:each-registered-sentinel-earns-its-own-slot-in-the-published-feature-layout´
/// ´test:crate:snapshot-has-dimension-map´
#[test]
fn snapshot_has_dimension_map() {
    use indexmap::IndexMap;

    use crate::feature::dimension_map::DimensionMap;
    use crate::testing::dimensions::sentinels;

    // Build a working copy with a dimension map that includes 1 sentinel.
    let mut wc = WorkingCopy::cold_start(77, &test_config(), 1000);
    wc.dimension_map = DimensionMap::rebuild_no_interactions(0, &sentinels(&[1]), &[], &IndexMap::new(), &[]);

    let snap = wc.to_snapshot(1);
    assert!(snap.dimension_map.p > 0, "dimension_map.p should be > 0");
    assert_eq!(snap.dimension_map.sentinel_slots.len(), 1, "1 sentinel slot");
}

/// The published layout survives serialisation whole: total width, the
/// per-Sentinel slots, the per-dimension ranges, the cross-dimension block and
/// the competitive-cell range all come back as they went in. A restored model
/// interprets feature positions by this map, so a layout that changed shape in
/// storage would silently pair every learned coefficient with the wrong
/// feature.
///
/// ´claim:snapshot:the-feature-layout-survives-serialisation-so-a-restored-model-reads-features-at-the-same-offsets´
/// ´test:crate:snapshot-dimension-map-serde´
#[cfg(feature = "serde")]
#[test]
fn snapshot_dimension_map_serde() {
    use crate::feature::dimension_map::DimensionMap;
    use crate::testing::dimensions::{cells, dims, make_cells, n_axes, sentinels};

    // Build a rich dimension map.
    let dm = DimensionMap::rebuild_no_interactions(
        3,
        &sentinels(&[10, 20]),
        &dims(&[1]),
        &cells(&[(1, &make_cells(5))]),
        &n_axes(1),
    );

    let mut wc = WorkingCopy::cold_start(dm.p, &test_config(), 1000);
    wc.dimension_map = dm.clone();

    let snap = wc.to_snapshot(1);

    // Round-trip through the serde codec (simulating checkpoint serde of the snapshot's DimensionMap).
    let bytes = crate::serde_codec::serialize(&snap.dimension_map).expect("serialize");
    let restored: DimensionMap = crate::serde_codec::deserialize(&bytes).expect("deserialize");

    assert_eq!(restored.p, dm.p);
    assert_eq!(restored.sentinel_slots.len(), 2);
    assert_eq!(restored.id_dim_ranges.len(), 1);
    assert!(restored.id_cross_dim_range.is_some());
    assert_eq!(restored.competitive_range.len(), 5);
    // TODO(2026-05-22) ´todo:test:replace-legacy-projection-index-equality-serde´: replace legacy projection-index equality serde
    // checks with canonical `AnchorProjection` serde coverage, so the index
    // comes from the one resolver (´dec:vector:sole-resolver´).
    assert_eq!(restored.anchor_projection_indices, dm.anchor_projection_indices,);
}

/// A registered outcome axis is published with more than its parameters: its
/// own calibration factor and its spatial flag travel with it. Readers need
/// both to use the axis at all — the factor to put the prediction back on the
/// host's scale, and the flag to know whether the axis participates in
/// per-Sentinel spatial extraction.
///
/// ´claim:snapshot:a-published-outcome-axis-carries-its-calibration-and-its-spatial-flag-alongside-its-parameters´
/// ´test:crate:snapshot-with-outcome-models´
#[test]
fn snapshot_with_outcome_models() {
    use crate::model::bayesian::BayesianLinearModel;
    use crate::snapshot::working::WorkingAxisModel;
    use crate::types::OutcomeAxisId;

    let mut wc = WorkingCopy::cold_start(50, &test_config(), 1000);
    wc.outcome_models.insert(
        OutcomeAxisId(1),
        WorkingAxisModel {
            name: "axis_1".to_owned(),
            description: "axis one description".to_owned(),
            model: BayesianLinearModel::new(50, 0.1, 1000),
            kappa_a: 1.5,
            gamma: 0.9998,
            spatial: true,
            eligibility: crate::types::OutcomeEligibility::default(),
        },
    );

    let snap = wc.to_snapshot(1);
    assert_eq!(snap.outcome_models.len(), 1, "should have 1 outcome model");
    assert!(snap.outcome_models.contains_key(&OutcomeAxisId(1)));
    assert_eq!(snap.outcome_models[&OutcomeAxisId(1)].kappa_a, 1.5);
    assert!(snap.outcome_models[&OutcomeAxisId(1)].spatial);
}

/// The separation holds on the writer's side as well as in what it publishes:
/// a working copy built at one model width still lays its features out at the
/// structural width, and the two coexist without one being derived from the
/// other. The layout is rebuilt when registrations change, whereas the model's
/// width is fixed at construction — they answer to different events and so
/// cannot be one number.
///
/// (´claim:snapshot:the-feature-layout-width-follows-what-is-registered-not-the-models-parameter-width´)
/// ´test:crate:snapshot-working-copy-gains-dim-map´
#[test]
fn snapshot_working_copy_gains_dim_map() {
    let wc = WorkingCopy::cold_start(50, &test_config(), 1000);
    // At cold start, dimension map is bias(1) + agg(15) = 16.
    assert_eq!(wc.dimension_map.p, 16);
    // The BLM operational dimension is independent.
    assert_eq!(wc.operational.dim(), 50);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Snapshot + DimensionMap Integration
// ═══════════════════════════════════════════════════════════════════════════════

/// The layout still round-trips intact at the scale a real deployment reaches
/// — eight Sentinels, two identity dimensions and twenty competitive cells
/// between them — with the per-Sentinel feature width and every anchor
/// projection index preserved element by element. Serialisation fidelity has
/// to hold at production shape, not merely for the small configurations that
/// happen to be convenient to construct.
///
/// (´claim:snapshot:the-feature-layout-survives-serialisation-so-a-restored-model-reads-features-at-the-same-offsets´)
/// ´test:crate:snapshot-with-full-dim-map-serde´
#[cfg(feature = "serde")]
#[test]
fn snapshot_with_full_dim_map_serde() {
    use crate::feature::dimension_map::DimensionMap;
    use crate::testing::dimensions::{cells, dims, make_cells, n_axes, sentinels};

    // Reference-ish config: 8 sentinels, 2 dims, 10 cells each, m_s=1.
    let cs = make_cells(10);
    let dm = DimensionMap::rebuild_no_interactions(
        0,
        &sentinels(&[10, 11, 12, 13, 14, 15, 16, 17]),
        &dims(&[1, 2]),
        &cells(&[(1, &cs), (2, &cs)]),
        &n_axes(1),
    );

    let mut wc = WorkingCopy::cold_start(dm.p, &test_config(), 1000);
    wc.dimension_map = dm.clone();

    let snap = wc.to_snapshot(1);

    // Snapshot → serde → restore DimensionMap
    let bytes = crate::serde_codec::serialize(&snap.dimension_map).expect("serialize");
    let restored: DimensionMap = crate::serde_codec::deserialize(&bytes).expect("deserialize");

    assert_eq!(restored.p, dm.p);
    assert_eq!(restored.sentinel_slots.len(), 8);
    assert_eq!(restored.id_dim_ranges.len(), 2);
    assert!(restored.id_cross_dim_range.is_some());
    assert_eq!(restored.competitive_range.len(), 20);
    assert_eq!(restored.sentinel_feature_width, dm.sentinel_feature_width);
    // TODO(2026-05-22) ´todo:test:replace-legacy-projection-index-element-wise´: replace legacy projection-index element-wise
    // serde checks with canonical `AnchorProjection` serde coverage, so the
    // index comes from the one resolver (´dec:vector:sole-resolver´).
    for (i, idx) in restored.anchor_projection_indices.iter().enumerate() {
        assert_eq!(idx, &dm.anchor_projection_indices[i], "anchor[{i}] mismatch");
    }
}

/// The cold-start layout is a definite arrangement, not merely a width: the
/// bias occupies the first position and the fifteen cross-Sentinel aggregates
/// follow it. Every later registration appends beyond that block, so these
/// fixed positions are what let the same feature keep the same index as the
/// layout grows.
///
/// ´claim:snapshot:the-layout-opens-with-the-bias-then-the-fifteen-aggregates-and-grows-only-after-them´
/// ´test:crate:working-copy-dim-map-consistent´
#[test]
fn working_copy_dim_map_consistent() {
    // Cold-start: dim_map.p reflects the structural layout, not the BLM dim.
    let wc = WorkingCopy::cold_start(77, &test_config(), 1000);
    assert_eq!(wc.dimension_map.p, 16, "cold-start structural p = bias + agg");
    // The dimension_map.p field is accessible and consistent.
    assert_eq!(wc.dimension_map.bias_idx, 0);
    assert_eq!(wc.dimension_map.agg_range.len(), 15);
}

/// A checkpoint payload round-trips the fields the format has grown, not only
/// the ones it started with: the feature layout, the per-Sentinel ledger and
/// identity state, and the label sequence mark all come back. The payload is
/// how a process resumes across a restart, so a field added later but dropped
/// in transit would be invisible in every test except one that restores and
/// looks for it.
///
/// ´claim:snapshot:a-checkpoint-payload-round-trips-every-field-the-format-has-grown-not-only-its-original-ones´
/// ´test:crate:snapshot-version-survives-format-extension´
#[cfg(feature = "serde")]
#[test]
fn snapshot_version_survives_format_extension() {
    let wc = WorkingCopy::cold_start(50, &test_config(), 1000);

    // Round-trip through checkpoint payload, which carries the Layer 2 fields
    // whole (´dec:durability:checkpoint-journal´).
    let payload = wc.to_checkpoint_payload(crate::types::PersistentTimestamp::now(), Vec::new(), Vec::new());
    let bytes = crate::serde_codec::serialize(&payload).expect("serialize payload");
    let restored: crate::persistence::checkpoint::CheckpointPayload =
        crate::serde_codec::deserialize(&bytes).expect("deserialize payload");

    // Layer 2 fields present and correct.
    assert_eq!(restored.dimension_map.p, wc.dimension_map.p);
    assert!(restored.ledger_state.is_empty());
    assert!(restored.identity_state.is_empty());
    assert_eq!(restored.last_processed_label_seq, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// What a revert carries besides the parameters
// ═══════════════════════════════════════════════════════════════════════════════

/// A model reverted from a snapshot comes back carrying the floor masses it
/// had when the snapshot was published, bit for bit and in both shapes. The
/// masses are the one thing about the covariance that the covariance cannot
/// say: they record how much of the precision behind it was borrowed from the
/// floors rather than earned from labels, and nothing in Σ distinguishes the
/// two. A revert that dropped them would hand back models reading as though
/// every unit of that precision had been earned — the direction that
/// under-reports uncertainty, on the one path taken precisely because
/// something has already gone wrong. The masses planted here are non-zero and
/// distinct per model and per coordinate, so a restore that reset them or
/// crossed them between models fails rather than agreeing by accident.
///
/// ´claim:snapshot:a-revert-carries-the-floor-masses-the-snapshot-published´
/// ´test:crate:roundtrip-floor-masses-identical´
#[test]
fn roundtrip_floor_masses_identical() {
    use crate::model::parameters::FloorMass;
    use crate::types::ModelId;

    let config = test_config();

    #[allow(clippy::cast_precision_loss)] // Justified: tiny test index
    fn planted(model: &BayesianLinearModel, spectral: f64, base: f64) -> FloorMass {
        FloorMass::new(spectral, (0..model.dim()).map(|i| base + i as f64 * 0.25).collect())
    }

    let mut wc = WorkingCopy::cold_start(12, &config, 1000);

    let install = |model: &BayesianLinearModel, mass: &FloorMass, id: ModelId| {
        BayesianLinearModel::from_parameters(&model.to_parameters(), mass, config.lambda_prior, id, 1000)
            .expect("a prior-initialised covariance inverts")
    };

    let operational_mass = planted(&wc.operational, 0.125, 1.5);
    let sister_mass = planted(&wc.sister, 0.25, 2.5);
    let anchor_mass = planted(&wc.anchor, 0.5, 3.5);
    let operational = install(&wc.operational, &operational_mass, ModelId::Operational);
    let sister = install(&wc.sister, &sister_mass, ModelId::Sister);
    let anchor = install(&wc.anchor, &anchor_mass, ModelId::Anchor);
    wc.operational = operational;
    wc.sister = sister;
    wc.anchor = anchor;

    // Not vacuous: a model reading zero would agree with a restore that reset.
    assert!(
        wc.operational.spectral_floor_mass() > 0.0 && wc.operational.clamp_mass().iter().all(|&c| c > 0.0),
        "the planted masses must be non-zero, or the assertion below proves nothing"
    );

    let snapshot = wc.to_snapshot(1);
    assert_eq!(
        snapshot.floor_masses.operational.spectral.to_bits(),
        wc.operational.spectral_floor_mass().to_bits(),
        "the snapshot publishes what the model held"
    );

    let restored = WorkingCopy::from_snapshot(&snapshot, &config, 1000, 100).expect("the revert succeeds");

    for (name, before, after) in [
        ("operational", &wc.operational, &restored.operational),
        ("sister", &wc.sister, &restored.sister),
        ("anchor", &wc.anchor, &restored.anchor),
    ] {
        assert_eq!(
            after.spectral_floor_mass().to_bits(),
            before.spectral_floor_mass().to_bits(),
            "the {name} model's identity-shaped mass crosses the revert unchanged"
        );
        assert_eq!(
            after.clamp_mass().len(),
            before.clamp_mass().len(),
            "the {name} model's coordinate-shaped mass keeps its width"
        );
        for (i, (a, b)) in after.clamp_mass().iter().zip(before.clamp_mass()).enumerate() {
            assert_eq!(
                a.to_bits(),
                b.to_bits(),
                "the {name} model's coordinate-shaped mass crosses the revert unchanged at {i}"
            );
        }
    }
}
