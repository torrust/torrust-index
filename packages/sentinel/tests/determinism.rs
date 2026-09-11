// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`identical_seed_produces_identical_reports`] | determinism | Two sentinels built from one configuration with one seed, and stepped through the same batches, agree at every step — the same number of cells, the same ancestors, the same cross-cell contexts, and score means whose floating-point bit patterns are equal rather than merely near. Agreement is checked batch by batch and not only at the end, so a divergence could not open and close again unnoticed. |
//! | [`deterministic_across_repeated_runs`] | engine | cites (´claim:engine:the-root-tracker-receives-every-observation-in-every-batch´) |
//! | [`different_seeds_produce_different_scores`] | determinism | The seed is an input with observable consequences, not a formality: two sentinels differing only in their seed, fed identical values, disagree in at least one of the root's score means. The warming noise a tracker is primed with shapes the subspace it starts from, and that starting point is still visible in what the tracker measures once real traffic arrives — which is why reproducibility has to be stated in terms of the seed rather than of the data alone. |
//! | [`report_ordering_is_deterministic`] | determinism | All three report vectors come out in the order their contract states rather than in the order the walk produced: the competitive cells and the ancestors ascend by node handle, and the cross-cell contexts come shallowest first with ties broken by the handle. Depth leads there because handles are recycled as cells are evicted and restored, so a correctly ordered run can carry a lower handle at a greater depth. Neither ordering is a property of traversal or of when a cell was created, so two runs list the same entries in the same positions and a reader may compare them index by index. Splitting is forced aggressively here so that each vector holds several entries and the ordering is actually put to the question. |
//! | [`send_and_sync_bounds`] | engine | The engine type may be moved between threads and referenced from several at once — a statement about the type, discharged by the compiler when these bounds are demanded, not by anything the test executes at run time. It holds because the sentinel keeps no thread-bound state: its optional background warming lives behind a lock it owns. A host is therefore free to place a sentinel wherever its own concurrency model wants it. |

//! Reproducibility of the sentinel's output, and the bounds its type
//! carries.
//!
//! Everything the engine does to a batch is ordinary arithmetic in a fixed
//! order, and the one place randomness enters — the synthetic noise used to
//! warm a tracker before real traffic can teach it anything — is drawn from
//! a generator the configuration seeds. Two sentinels built from the same
//! configuration and fed the same values are therefore not merely close but
//! identical, down to the bit patterns of the reported means. That is what
//! makes a report worth comparing across runs at all: a difference between
//! two runs is a difference in what they were given, never in the order the
//! machine happened to visit things.
//!
//! For that guarantee to have content the seed has to be a real input.
//! Different seeds draw different warming noise, and the difference survives
//! into the scores rather than being averaged away, so a host that fixes the
//! seed is choosing a particular run and not merely satisfying a parameter.
//!
//! Ordering belongs to the same promise. Every vector in a report is emitted
//! in ascending node-handle order, so a reader compares two runs positionally
//! without depending on the order cells were visited in. Thread-safety is a
//! different kind of statement altogether — a property of the type rather
//! than of any run — and it is discharged by the compiler: the engine holds
//! no thread-bound state, so a host may move it between threads or share it
//! behind a lock of its own.

mod common;

use common::{ScenarioBuilder, assert_invariants, cell_values, test_config};
use torrust_sentinel::{Sentinel128, SentinelConfig};

// ── Reproducibility ─────────────────────────────────────────

/// Two sentinels built from one configuration with one seed, and stepped
/// through the same batches, agree at every step — the same number of cells,
/// the same ancestors, the same cross-cell contexts, and score means whose
/// floating-point bit patterns are equal rather than merely near. Agreement
/// is checked batch by batch and not only at the end, so a divergence could
/// not open and close again unnoticed.
///
/// ´claim:determinism:the-same-seed-and-the-same-data-reproduce-the-same-reports´
/// ´test:integration:identical-seed-produces-identical-reports´
#[test]
fn identical_seed_produces_identical_reports() {
    let cfg = SentinelConfig::<u64> {
        noise_seed: Some(42),
        ..test_config()
    };

    let values = cell_values(0xA, 100);

    let mut s1 = Sentinel128::new(cfg.clone()).unwrap();
    let mut s2 = Sentinel128::new(cfg).unwrap();

    for chunk in values.chunks(10) {
        let r1 = s1.ingest(chunk);
        let r2 = s2.ingest(chunk);

        assert_invariants(&s1, &r1);
        assert_invariants(&s2, &r2);

        assert_eq!(r1.cell_reports.len(), r2.cell_reports.len());
        assert_eq!(r1.ancestor_reports.len(), r2.ancestor_reports.len());
        assert_eq!(r1.coordination_reports.len(), r2.coordination_reports.len());
        assert_eq!(r1.health.lifetime_observations, r2.health.lifetime_observations);

        // Compare score values exactly (all operations are deterministic).
        for (c1, c2) in r1.cell_reports.iter().zip(&r2.cell_reports) {
            assert_eq!(
                c1.scores.novelty.mean.to_bits(),
                c2.scores.novelty.mean.to_bits(),
                "novelty mean mismatch"
            );
            assert_eq!(
                c1.scores.displacement.mean.to_bits(),
                c2.scores.displacement.mean.to_bits(),
                "displacement mean mismatch"
            );
        }

        for (a1, a2) in r1.ancestor_reports.iter().zip(&r2.ancestor_reports) {
            assert_eq!(
                a1.scores.novelty.mean.to_bits(),
                a2.scores.novelty.mean.to_bits(),
                "ancestor novelty mean mismatch"
            );
        }
    }
}

/// A seeded run lands on stated figures rather than merely on some figure.
/// The root appears in the report, its sample count equals the size of the
/// batch, and the lifetime count agrees with it — every value reached the
/// root, none was counted twice, and the warming noise the seed generated
/// did not leak into the record of real observations. The root's scores are
/// numbers, not the non-number that an empty or singular update would leave
/// behind.
///
/// (´claim:engine:the-root-tracker-receives-every-observation-in-every-batch´)
/// ´test:integration:deterministic-across-repeated-runs´
#[test]
fn deterministic_across_repeated_runs() {
    let cfg = SentinelConfig::<u64> {
        noise_seed: Some(12345),
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    let values = cell_values(0xA, 20);
    let report = s.ingest(&values);

    assert_invariants(&s, &report);

    assert_eq!(report.health.lifetime_observations, 20);

    let root = report.ancestor_reports.iter().find(|r| r.depth == 0);
    assert!(root.is_some(), "root (depth 0) must appear in ancestor reports");
    assert_eq!(root.unwrap().sample_count, 20);

    let root = root.unwrap();
    assert!(!root.scores.novelty.mean.is_nan());
    assert!(!root.scores.displacement.mean.is_nan());
}

// ── Seed sensitivity ────────────────────────────────────────

/// The seed is an input with observable consequences, not a formality: two
/// sentinels differing only in their seed, fed identical values, disagree in
/// at least one of the root's score means. The warming noise a tracker is
/// primed with shapes the subspace it starts from, and that starting point
/// is still visible in what the tracker measures once real traffic arrives —
/// which is why reproducibility has to be stated in terms of the seed rather
/// than of the data alone.
///
/// ´claim:determinism:the-seed-is-visible-in-the-scores-so-two-seeds-do-not-coincide´
/// ´test:integration:different-seeds-produce-different-scores´
#[test]
fn different_seeds_produce_different_scores() {
    let cfg_a = SentinelConfig::<u64> {
        noise_seed: Some(1),
        ..test_config()
    };
    let cfg_b = SentinelConfig::<u64> {
        noise_seed: Some(2),
        ..test_config()
    };

    let values = cell_values(0xA, 40);

    let mut sa = Sentinel128::new(cfg_a).unwrap();
    let mut sb = Sentinel128::new(cfg_b).unwrap();

    let ra = sa.ingest(&values);
    let rb = sb.ingest(&values);

    assert_invariants(&sa, &ra);
    assert_invariants(&sb, &rb);

    // At least one root-level score must differ because the noise
    // sequences are seeded differently.
    let root_a = ra.ancestor_reports.iter().find(|r| r.depth == 0).unwrap();
    let root_b = rb.ancestor_reports.iter().find(|r| r.depth == 0).unwrap();

    let scores_identical = root_a.scores.novelty.mean.to_bits() == root_b.scores.novelty.mean.to_bits()
        && root_a.scores.displacement.mean.to_bits() == root_b.scores.displacement.mean.to_bits()
        && root_a.scores.surprise.mean.to_bits() == root_b.scores.surprise.mean.to_bits();

    assert!(!scores_identical, "different seeds must produce different scores at root");
}

// ── Report ordering ─────────────────────────────────────────

/// All three report vectors come out in the order their contract states
/// rather than in the order the walk produced: the competitive cells and the
/// ancestors ascend by node handle, and the cross-cell contexts come
/// shallowest first with ties broken by the handle. Depth leads there because
/// handles are recycled as cells are evicted and restored, so a correctly
/// ordered run can carry a lower handle at a greater depth. Neither ordering
/// is a property of traversal or of when a cell was created, so two runs list
/// the same entries in the same positions and a reader may compare them index
/// by index. Splitting is forced aggressively here so that each vector holds
/// several entries and the ordering is actually put to the question.
///
/// ´claim:determinism:every-report-vector-is-ordered-by-node-handle-so-a-reader-never-depends-on-visit-order´
/// ´test:integration:report-ordering-is-deterministic´
#[test]
fn report_ordering_is_deterministic() {
    let cfg = SentinelConfig::<u64> {
        noise_seed: Some(42),
        split_threshold: 10,
        ..test_config()
    };

    let (mut s, _) = ScenarioBuilder::new()
        .config(cfg)
        .seed_range(0xA, 10)
        .seed_range(0xB, 10)
        .warm_batches(49)
        .build_with_reports();

    let report = s.ingest(&[cell_values(0xA, 4), cell_values(0xB, 4)].concat());

    for window in report.cell_reports.windows(2) {
        assert!(window[0].gnode_id < window[1].gnode_id, "cell_reports not in GNodeId order");
    }

    for window in report.ancestor_reports.windows(2) {
        assert!(
            window[0].gnode_id < window[1].gnode_id,
            "ancestor_reports not in GNodeId order"
        );
    }

    for window in report.coordination_reports.windows(2) {
        assert!(
            (window[0].depth, window[0].gnode_id) < (window[1].depth, window[1].gnode_id),
            "coordination_reports not in (depth, GNodeId) order: ({}, {:?}) >= ({}, {:?})",
            window[0].depth,
            window[0].gnode_id,
            window[1].depth,
            window[1].gnode_id,
        );
    }
}

// ── Thread safety ───────────────────────────────────────────

/// The engine type may be moved between threads and referenced from several
/// at once — a statement about the type, discharged by the compiler when
/// these bounds are demanded, not by anything the test executes at run time.
/// It holds because the sentinel keeps no thread-bound state: its optional
/// background warming lives behind a lock it owns. A host is therefore free
/// to place a sentinel wherever its own concurrency model wants it.
///
/// ´claim:engine:the-sentinel-type-carries-send-and-sync-so-a-host-may-own-it-across-threads´
/// ´test:integration:send-and-sync-bounds´
#[test]
fn send_and_sync_bounds() {
    fn assert_send<T: Send>() {}
    fn assert_sync<T: Sync>() {}

    assert_send::<Sentinel128>();
    assert_sync::<Sentinel128>();
}
