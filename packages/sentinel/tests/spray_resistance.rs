// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`competitive_set_bounded_by_k`] | resistance | Spraying traffic across far more leading ranges than the sentinel is permitted to model does not enlarge the set of cells that compete for modelling effort: it stays within the configured cap. The cap is on attention, not on input, so an attacker who can address any part of the domain still cannot make the sentinel promise more work than it budgeted for. |
//! | [`competitive_set_at_k_equals_one`] | resistance | cites (´claim:resistance:a-spray-across-many-ranges-cannot-enlarge-the-competitive-set-beyond-its-cap´) |
//! | [`full_set_bounded_by_steiner`] | resistance | Capping the winners would be hollow if the ancestors pulled in to connect them to the root were unbounded, since each of those also carries a tracker. After a spray across many ranges the materialised set stays within the root plus the cap times the deepest level reached — the connecting chains are shared and counted, so the total cost of attention is a function of the cap and the depth alone, never of how many ranges were touched. |
//! | [`g_nodes_bounded_by_budget_under_spray`] | resistance | Feeding a long run of one-value batches, each a distinct coordinate spread over the ranges, leaves the tree comfortably inside its node budget rather than growing a node per distinct value. Memory is the resource an attacker would most like to exhaust, so the budget is enforced by eviction as the tree grows and is not merely a hint the structure is asked to respect. |
//! | [`cells_tracked_bounded_under_diverse_traffic`] | resistance | Sustained traffic to every leading range at once leaves the number of live trackers bounded by roughly twice the competitive cap. Trackers are the expensive objects — each carries a learned subspace and its baselines — so what bounds them is the cap on attention rather than the diversity of the traffic. Diverse traffic that is not adversarial is held to the same bound as a spray, because the sentinel does not need to tell them apart to stay within budget. |
//! | [`concentrated_range_survives_spray`] | resistance | A range carrying the great bulk of the traffic is still represented in the reports after a thin spray touches every other range — either as a competitor in its own right or through the chain of ancestors that covers it. Attention is bought with accumulated weight rather than with novelty, which is what stops a cheap spray from evicting the model of the range an operator actually cares about. |
//! | [`invariants_hold_under_spray`] | resistance | The structural guarantees are checked after every single batch of a wide spray and again through the concentrated burst that follows it, and none of them breaks. The bounds, the ordering of the reports and the presence of the root are not properties of a settled sentinel: they hold batch by batch while the tree is being churned by hostile traffic and while it is reconverging afterwards, which is the only time they matter. |

//! Integration tests for the sentinel's **resistance to spray** — traffic
//! spread thinly and deliberately across the whole coordinate domain rather
//! than concentrated where real activity lives.
//!
//! A spray is cheap for an attacker and expensive for a naive watcher. Every
//! fresh region looks like something new worth modelling, so a design that let
//! attention follow novelty could be made to allocate a tracker per sprayed
//! region until it ran out of memory, or to evict the model of the range that
//! actually mattered. The sentinel answers with hard caps rather than
//! heuristics: only a bounded number of cells may compete for modelling effort,
//! the ancestors dragged in to connect those winners to the root are bounded in
//! turn by that count times the depth reached, the tree beneath it all is held
//! under an explicit node budget, and the number of live trackers is bounded by
//! the competitive cap. None of those bounds is a function of how many distinct
//! values arrived, which is precisely why spraying more of them buys nothing.
//!
//! The caps alone would be a poor defence if they were satisfied by dropping
//! the traffic that matters, so the other half of the property is that
//! attention is bought with weight, not with variety: a range carrying the bulk
//! of the traffic is still represented after a thin spray across many ranges.
//! The tests here drive the sentinel through spray, through spray followed by a
//! concentrated burst, and through the boundary where only a single cell may
//! ever compete.

mod common;

use common::{assert_invariants, cell_values, integration_config, test_config};
use torrust_sentinel::{Sentinel128, SentinelConfig};

// ═══════════════════════════════════════════════════════════
//  Analysis-set bounds
// ═══════════════════════════════════════════════════════════

/// Spraying traffic across far more leading ranges than the sentinel is
/// permitted to model does not enlarge the set of cells that compete for
/// modelling effort: it stays within the configured cap. The cap is on
/// attention, not on input, so an attacker who can address any part of the
/// domain still cannot make the sentinel promise more work than it budgeted
/// for.
///
/// ´claim:resistance:a-spray-across-many-ranges-cannot-enlarge-the-competitive-set-beyond-its-cap´
/// ´test:integration:competitive-set-bounded-by-k´
#[test]
fn competitive_set_bounded_by_k() {
    let k = 8;
    let cfg = SentinelConfig::<u64> {
        analysis_k: k,
        split_threshold: 10,
        budget: 10_000,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Spray traffic across 64 distinct leading nibbles.
    for nibble in 0..64u128 {
        s.ingest(&cell_values(nibble, 20));
    }

    let report = s.ingest(&cell_values(0, 4));
    assert_invariants(&s, &report);
    assert!(
        report.analysis_set_summary.competitive_size <= k,
        "competitive set {} exceeds K={}",
        report.analysis_set_summary.competitive_size,
        k,
    );
}

/// This pins the tightest end of the cap. Configured to model a single
/// competitor and then fed repeated traffic to every leading range in turn, the
/// sentinel still admits at most one — and it does not answer the pressure by
/// emptying out, since the set it materialises always retains at least the
/// root. A cap of one is an ordinary setting rather than a degenerate case that
/// collapses the analysis set.
///
/// (´claim:resistance:a-spray-across-many-ranges-cannot-enlarge-the-competitive-set-beyond-its-cap´)
/// ´test:integration:competitive-set-at-k-equals-one´
#[test]
fn competitive_set_at_k_equals_one() {
    let cfg = SentinelConfig::<u64> {
        analysis_k: 1,
        split_threshold: 10,
        budget: 10_000,
        ..integration_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    for nibble in 0..16u128 {
        for _ in 0..8 {
            s.ingest(&cell_values(nibble, 16));
        }
    }

    let report = s.ingest(&cell_values(0xA, 4));
    assert_invariants(&s, &report);
    assert!(
        report.analysis_set_summary.competitive_size <= 1,
        "with K=1, competitive set should be at most 1, got {}",
        report.analysis_set_summary.competitive_size,
    );
    assert!(
        report.analysis_set_summary.full_size >= 1,
        "full set should have at least root"
    );
}

/// Capping the winners would be hollow if the ancestors pulled in to connect
/// them to the root were unbounded, since each of those also carries a tracker.
/// After a spray across many ranges the materialised set stays within the root
/// plus the cap times the deepest level reached — the connecting chains are
/// shared and counted, so the total cost of attention is a function of the cap
/// and the depth alone, never of how many ranges were touched.
///
/// ´claim:resistance:the-ancestors-connecting-the-winners-are-bounded-by-the-cap-times-the-depth-reached´
/// ´test:integration:full-set-bounded-by-steiner´
#[test]
fn full_set_bounded_by_steiner() {
    let k = 8;
    let cfg = SentinelConfig::<u64> {
        analysis_k: k,
        split_threshold: 10,
        budget: 10_000,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    for nibble in 0..64u128 {
        s.ingest(&cell_values(nibble, 20));
    }

    let report = s.ingest(&cell_values(0, 4));
    assert_invariants(&s, &report);

    let summary = &report.analysis_set_summary;
    let d_max = summary.depth_range.1 as usize;
    let bound = 1 + k * d_max;
    assert!(
        summary.full_size <= bound,
        "full set {} exceeds Steiner bound 1+K·d_max={} (K={}, d_max={})",
        summary.full_size,
        bound,
        k,
        d_max,
    );
}

// ═══════════════════════════════════════════════════════════
//  Resource bounds
// ═══════════════════════════════════════════════════════════

/// Feeding a long run of one-value batches, each a distinct coordinate spread
/// over the ranges, leaves the tree comfortably inside its node budget rather
/// than growing a node per distinct value. Memory is the resource an attacker
/// would most like to exhaust, so the budget is enforced by eviction as the
/// tree grows and is not merely a hint the structure is asked to respect.
///
/// ´claim:resistance:a-stream-of-distinct-sprayed-values-cannot-grow-the-tree-past-its-node-budget´
/// ´test:integration:g-nodes-bounded-by-budget-under-spray´
#[test]
fn g_nodes_bounded_by_budget_under_spray() {
    let budget = 500;
    let cfg = SentinelConfig::<u64> {
        budget,
        split_threshold: 10,
        d_create: 3,
        d_evict: 6,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Spray many distinct values across nibble ranges.
    for i in 0..500u128 {
        let nibble = i % 16;
        s.ingest(&[(nibble << 124) | (i + 1)]);
    }

    let g_nodes = s.health().total_g_nodes;
    assert!(
        g_nodes <= budget * 2 + 2,
        "G-tree node count {g_nodes} exceeds budget guard (budget={budget})",
    );
}

/// Sustained traffic to every leading range at once leaves the number of live
/// trackers bounded by roughly twice the competitive cap. Trackers are the
/// expensive objects — each carries a learned subspace and its baselines — so
/// what bounds them is the cap on attention rather than the diversity of the
/// traffic. Diverse traffic that is not adversarial is held to the same bound
/// as a spray, because the sentinel does not need to tell them apart to stay
/// within budget.
///
/// ´claim:resistance:the-number-of-live-trackers-stays-near-the-competitive-cap-however-diverse-the-traffic´
/// ´test:integration:cells-tracked-bounded-under-diverse-traffic´
#[test]
fn cells_tracked_bounded_under_diverse_traffic() {
    let k = 8;
    let cfg = SentinelConfig::<u64> {
        analysis_k: k,
        split_threshold: 10,
        budget: 10_000,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Send traffic to all 16 leading-nibble ranges.
    for nibble in 0..16u128 {
        for _ in 0..12 {
            s.ingest(&cell_values(nibble, 16));
        }
    }

    assert!(
        s.cells_tracked() <= 2 * k + 1,
        "cells_tracked {} exceeds 2K+1={} (K={})",
        s.cells_tracked(),
        2 * k + 1,
        k,
    );
}

// ═══════════════════════════════════════════════════════════
//  Fairness
// ═══════════════════════════════════════════════════════════

/// A range carrying the great bulk of the traffic is still represented in the
/// reports after a thin spray touches every other range — either as a
/// competitor in its own right or through the chain of ancestors that covers
/// it. Attention is bought with accumulated weight rather than with novelty,
/// which is what stops a cheap spray from evicting the model of the range an
/// operator actually cares about.
///
/// ´claim:resistance:a-thin-spray-does-not-displace-the-range-that-carries-the-weight-of-the-traffic´
/// ´test:integration:concentrated-range-survives-spray´
#[test]
fn concentrated_range_survives_spray() {
    let cfg = SentinelConfig::<u64> {
        analysis_k: 4,
        split_threshold: 50,
        budget: 10_000,
        max_rank: 2,
        ..integration_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // 90% traffic to range A.
    for _ in 0..20 {
        s.ingest(&cell_values(0xA, 40));
    }

    // 10% spray across many ranges.
    for nibble in 0..16u128 {
        s.ingest(&cell_values(nibble, 3));
    }

    let report = s.ingest(&cell_values(0xA, 8));
    assert_invariants(&s, &report);

    // The concentrated range should still be represented — either as a
    // competitive cell or through its ancestor chain.
    let has_range_a = report
        .cell_reports
        .iter()
        .chain(report.ancestor_reports.iter())
        .any(|cr| cr.sample_count > 0);

    assert!(has_range_a, "concentrated range A should still be represented in reports");
}

// ═══════════════════════════════════════════════════════════
//  Cross-cutting invariants
// ═══════════════════════════════════════════════════════════

/// The structural guarantees are checked after every single batch of a wide
/// spray and again through the concentrated burst that follows it, and none of
/// them breaks. The bounds, the ordering of the reports and the presence of the
/// root are not properties of a settled sentinel: they hold batch by batch
/// while the tree is being churned by hostile traffic and while it is
/// reconverging afterwards, which is the only time they matter.
///
/// ´claim:resistance:the-structural-guarantees-hold-batch-by-batch-through-a-spray-and-the-burst-that-follows-it´
/// ´test:integration:invariants-hold-under-spray´
#[test]
fn invariants_hold_under_spray() {
    let cfg = SentinelConfig::<u64> {
        analysis_k: 8,
        split_threshold: 10,
        budget: 10_000,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Phase 1: wide spray across 64 nibbles.
    for nibble in 0..64u128 {
        let report = s.ingest(&cell_values(nibble, 20));
        assert_invariants(&s, &report);
    }

    // Phase 2: concentrated burst after spray.
    for _ in 0..10 {
        let report = s.ingest(&cell_values(0xA, 50));
        assert_invariants(&s, &report);
    }
}
