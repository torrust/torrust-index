// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! The coverage matrix — which combinations of structure and state the
//! sentinel is exercised against (§ALGO S-17.6).
//!
//! A disturbance can differ along two axes at once, and the two are
//! independent. It has an *extent* — confined to one range, spread over some
//! of them, or present everywhere — and it has an *onset*, either arriving all
//! at once or creeping in over a run of batches. Crossing the two gives the
//! modalities a deployed sentinel has to survive, and each of them stresses a
//! different part of the design: extent decides at which level of the chain
//! the disturbance is visible, while onset decides whether a single batch
//! comparison suffices or whether the evidence has to be carried forward and
//! summed.
//!
//! The matrix matters because the failure modes are complementary. A design
//! tuned only to per-cell comparison catches the sharp local case and is blind
//! to a slow shift that each batch alone could pass off as ordinary; a design
//! that only watches the whole domain misses what one range is doing. Covering
//! the grid is what says the two mechanisms compose rather than each covering
//! for the other's blind spot on the examples that happened to be written.
//!
//! Every case here asks the same question of the sentinel: is the measurement
//! after the disturbance higher than what this same sentinel produced under
//! steady traffic moments before? None asks whether a threshold was crossed.
//! What counts as alarming depends on the deployment and belongs to the host;
//! the sentinel's obligation is that the number move in the right direction
//! and by an amount the host can reason about (ADR-S-001).
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`sudden_single_cell_z_score`] | coverage | The sharpest corner of the matrix: a single range switching abruptly to structurally different values pushes the highest per-cell score above what the same sentinel measured on the preceding ordinary batch. One batch is enough here, because the disturbance is a departure from learned structure rather than a change in how often the range is visited. |
//! | [`gradual_single_cell_cusum`] | coverage | Persistence is itself evidence. Where the anomalous traffic keeps arriving batch after batch, the accumulating statistic climbs above where it stood on the first such batch, so a disturbance that never grows louder still grows more certain. The comparison is taken only after the slow baseline has been allowed to settle, which is what separates genuine accumulation from the transient left over from warm-up. |
//! | [`sudden_partial_per_cell`] | coverage | Normal traffic elsewhere does not dilute an anomaly. With several ranges live and only some of them turning anomalous, the highest per-cell score still rises above the steady-state batch — the untouched ranges contribute their own unremarkable readings and nothing else. Because scoring is per-cell rather than an average over the domain, an attacker gains nothing by keeping most of the traffic ordinary. |
//! | [`gradual_partial_cusum`] | coverage | The two evasions combined — a drift rather than a jump, and in only part of the traffic — still accumulate: with one range going anomalous batch after batch while another stays ordinary, the accumulated statistic ends above where the first anomalous batch left it. Being quiet and being partial are not additive protections, because accumulation happens per cell and the ordinary range accumulates nothing to average it away. |
//! | [`sudden_system_wide_root_catches`] | coverage | The case a per-cell view is least equipped for: every range changes at once, so no cell is unusual relative to its neighbours, and the reading that moves is the root's. Coverage of this corner is what the coarse end of the chain exists for — an attack that shifts the whole population is caught by the model whose region is the whole population. |
//! | [`gradual_system_wide_root_cusum`] | coverage | The quietest corner of the matrix, and the one a self-adjusting baseline is most at risk of absorbing: a shift across all ranges that simply keeps happening. The accumulated statistic keeps climbing past its first-batch value rather than settling, so persistence still tells even where extent leaves no cell looking unusual against its neighbours and no batch looks unusual against the last. |

mod common;

use common::{
    ScenarioBuilder, anomalous_values, assert_invariants, cell_values, integration_config, max_cusum, max_novelty_z,
    root_novelty_z,
};
use torrust_sentinel::SentinelConfig;

// ── 1. sudden_single_cell_z_score ──────────────────────────

/// The sharpest corner of the matrix: a single range switching abruptly to
/// structurally different values pushes the highest per-cell score above what
/// the same sentinel measured on the preceding ordinary batch. One batch is
/// enough here, because the disturbance is a departure from learned structure
/// rather than a change in how often the range is visited.
///
/// ´claim:coverage:a-sudden-change-in-one-range-shows-up-in-a-single-batch-comparison´
/// ´test:integration:sudden-single-cell-z-score´
#[test]
fn sudden_single_cell_z_score() {
    let mut s = ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            split_threshold: 10,
            ..integration_config()
        })
        .seed_range(0xA, 20)
        .warm_batches(5)
        .build();

    // Steady-state reference.
    let normal = s.ingest(&cell_values(0xA, 8));
    assert_invariants(&s, &normal);

    // Anomalous batch — structurally different data.
    let anomaly = s.ingest(&anomalous_values(0xA, 8));
    assert_invariants(&s, &anomaly);

    let normal_z = max_novelty_z(&normal);
    let anomaly_z = max_novelty_z(&anomaly);

    assert!(
        anomaly_z > normal_z,
        "sudden single-cell anomaly should produce higher z-score: \
         anomaly={anomaly_z:.4}, normal={normal_z:.4}"
    );
}

// ── 2. gradual_single_cell_cusum ───────────────────────────

/// Persistence is itself evidence. Where the anomalous traffic keeps arriving
/// batch after batch, the accumulating statistic climbs above where it stood
/// on the first such batch, so a disturbance that never grows louder still
/// grows more certain. The comparison is taken only after the slow baseline
/// has been allowed to settle, which is what separates genuine accumulation
/// from the transient left over from warm-up.
///
/// ´claim:coverage:a-disturbance-that-persists-keeps-accumulating-even-when-it-never-grows-louder´
/// ´test:integration:gradual-single-cell-cusum´
#[test]
fn gradual_single_cell_cusum() {
    let cfg = SentinelConfig::<u64> {
        max_rank: 1,
        cusum_slow_decay: 0.96,
        cusum_coord_slow_decay: 0.96,
        split_threshold: 10,
        ..integration_config()
    };
    let mut s = ScenarioBuilder::new().config(cfg).seed_range(0xA, 20).warm_batches(8).build();

    // Stabilise slow EWMA (λ_s=0.96, h_s≈17) before measuring.
    for _ in 0..20 {
        s.ingest(&cell_values(0xA, 8));
    }

    // Record CUSUM from first anomaly batch as baseline.
    let first_report = s.ingest(&anomalous_values(0xA, 8));
    assert_invariants(&s, &first_report);
    let cusum_first = max_cusum(&first_report);

    // Gradual anomaly: 4 more batches of anomalous traffic.
    let mut cusum_after = cusum_first;
    for _ in 0..4 {
        let report = s.ingest(&anomalous_values(0xA, 8));
        cusum_after = max_cusum(&report);
    }

    assert!(
        cusum_after > cusum_first,
        "CUSUM should accumulate under gradual anomaly: \
         first={cusum_first:.4}, last={cusum_after:.4}"
    );
}

// ── 3. sudden_partial_per_cell ─────────────────────────────

/// Normal traffic elsewhere does not dilute an anomaly. With several ranges
/// live and only some of them turning anomalous, the highest per-cell score
/// still rises above the steady-state batch — the untouched ranges contribute
/// their own unremarkable readings and nothing else. Because scoring is
/// per-cell rather than an average over the domain, an attacker gains nothing
/// by keeping most of the traffic ordinary.
///
/// ´claim:coverage:normal-traffic-elsewhere-does-not-mask-an-anomaly-in-some-ranges´
/// ´test:integration:sudden-partial-per-cell´
#[test]
fn sudden_partial_per_cell() {
    let mut s = ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            split_threshold: 10,
            ..integration_config()
        })
        .seed_range(0xA, 20)
        .seed_range(0xB, 20)
        .seed_range(0xC, 20)
        .seed_range(0xD, 20)
        .warm_batches(5)
        .build();

    // Steady-state reference across all ranges.
    let normal = s.ingest(
        &[
            cell_values(0xA, 8),
            cell_values(0xB, 8),
            cell_values(0xC, 8),
            cell_values(0xD, 8),
        ]
        .concat(),
    );
    assert_invariants(&s, &normal);

    // Partial anomaly: anomalous traffic only to ranges A and B.
    let partial = s.ingest(
        &[
            anomalous_values(0xA, 8),
            anomalous_values(0xB, 8),
            cell_values(0xC, 8),
            cell_values(0xD, 8),
        ]
        .concat(),
    );
    assert_invariants(&s, &partial);

    let normal_z = max_novelty_z(&normal);
    let partial_z = max_novelty_z(&partial);

    assert!(
        partial_z > normal_z,
        "partial anomaly should produce elevated z-scores: \
         partial={partial_z:.4}, normal={normal_z:.4}"
    );
}

// ── 4. gradual_partial_cusum ───────────────────────────────

/// The two evasions combined — a drift rather than a jump, and in only part of
/// the traffic — still accumulate: with one range going anomalous batch after
/// batch while another stays ordinary, the accumulated statistic ends above
/// where the first anomalous batch left it. Being quiet and being partial are
/// not additive protections, because accumulation happens per cell and the
/// ordinary range accumulates nothing to average it away.
///
/// ´claim:coverage:being-both-gradual-and-partial-does-not-stop-the-evidence-accumulating´
/// ´test:integration:gradual-partial-cusum´
#[test]
fn gradual_partial_cusum() {
    let cfg = SentinelConfig::<u64> {
        cusum_slow_decay: 0.96,
        cusum_coord_slow_decay: 0.96,
        split_threshold: 10,
        ..integration_config()
    };
    let mut s = ScenarioBuilder::new()
        .config(cfg)
        .seed_range(0xA, 20)
        .seed_range(0xB, 20)
        .warm_batches(8)
        .build();

    // Stabilise: let slow EWMA (λ_s=0.96, h_s≈17) catch up so CUSUM
    // from the warm-up transient settles before we measure.
    for _ in 0..20 {
        s.ingest(&[cell_values(0xA, 8), cell_values(0xB, 8)].concat());
    }

    // Record CUSUM from first anomaly batch as baseline.
    let first_anomaly = s.ingest(&[anomalous_values(0xA, 8), cell_values(0xB, 8)].concat());
    assert_invariants(&s, &first_anomaly);
    let cusum_first = max_cusum(&first_anomaly);

    // Gradual partial: anomalous traffic to range A only, normal to B.
    let mut cusum_after = cusum_first;
    for _ in 0..4 {
        let report = s.ingest(&[anomalous_values(0xA, 8), cell_values(0xB, 8)].concat());
        cusum_after = max_cusum(&report);
    }

    assert!(
        cusum_after > cusum_first,
        "gradual partial anomaly should accumulate CUSUM: \
         first={cusum_first:.4}, last={cusum_after:.4}"
    );
}

// ── 5. sudden_system_wide_root_catches ─────────────────────

/// The case a per-cell view is least equipped for: every range changes at
/// once, so no cell is unusual relative to its neighbours, and the reading
/// that moves is the root's. Coverage of this corner is what the coarse end of
/// the chain exists for — an attack that shifts the whole population is caught
/// by the model whose region is the whole population.
///
/// ´claim:coverage:a-change-in-every-range-at-once-is-caught-at-the-root´
/// ´test:integration:sudden-system-wide-root-catches´
#[test]
fn sudden_system_wide_root_catches() {
    let mut s = ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            split_threshold: 10,
            ..integration_config()
        })
        .seed_range(0xA, 20)
        .seed_range(0xB, 20)
        .seed_range(0xC, 20)
        .warm_batches(5)
        .build();

    // Normal reference.
    let normal = s.ingest(&[cell_values(0xA, 8), cell_values(0xB, 8), cell_values(0xC, 8)].concat());
    assert_invariants(&s, &normal);

    // System-wide anomaly: all ranges anomalous.
    let anomaly = s.ingest(&[anomalous_values(0xA, 8), anomalous_values(0xB, 8), anomalous_values(0xC, 8)].concat());
    assert_invariants(&s, &anomaly);

    let normal_root_z = root_novelty_z(&normal);
    let anomaly_root_z = root_novelty_z(&anomaly);

    assert!(
        anomaly_root_z > normal_root_z,
        "system-wide anomaly should elevate root z-score: \
         anomaly={anomaly_root_z:.4}, normal={normal_root_z:.4}"
    );
}

// ── 6. gradual_system_wide_root_cusum ──────────────────────

/// The quietest corner of the matrix, and the one a self-adjusting baseline is
/// most at risk of absorbing: a shift across all ranges that simply keeps
/// happening. The accumulated statistic keeps climbing past its first-batch
/// value rather than settling, so persistence still tells even where extent
/// leaves no cell looking unusual against its neighbours and no batch looks
/// unusual against the last.
///
/// ´claim:coverage:a-slow-shift-across-everything-accumulates-instead-of-becoming-the-new-normal´
/// ´test:integration:gradual-system-wide-root-cusum´
#[test]
fn gradual_system_wide_root_cusum() {
    let cfg = SentinelConfig::<u64> {
        cusum_slow_decay: 0.96,
        cusum_coord_slow_decay: 0.96,
        split_threshold: 10,
        ..integration_config()
    };
    let mut s = ScenarioBuilder::new()
        .config(cfg)
        .seed_range(0xA, 20)
        .seed_range(0xB, 20)
        .warm_batches(8)
        .build();

    // Stabilise slow EWMA before measuring.
    for _ in 0..20 {
        s.ingest(&[cell_values(0xA, 8), cell_values(0xB, 8)].concat());
    }

    // Record CUSUM from first anomaly batch.
    let first_report = s.ingest(&[anomalous_values(0xA, 8), anomalous_values(0xB, 8)].concat());
    assert_invariants(&s, &first_report);
    let cusum_first = max_cusum(&first_report);

    // Gradual system-wide: all ranges anomalous for 4 more batches.
    let mut cusum_after = cusum_first;
    for _ in 0..4 {
        let report = s.ingest(&[anomalous_values(0xA, 8), anomalous_values(0xB, 8)].concat());
        cusum_after = max_cusum(&report);
    }

    assert!(
        cusum_after > cusum_first,
        "gradual system-wide anomaly should accumulate CUSUM: \
         first={cusum_first:.4}, last={cusum_after:.4}"
    );
}
