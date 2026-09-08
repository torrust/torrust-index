// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`fresh_graph_has_one_node`] | routing | Before anything is observed the graph is a single cell spanning the whole coordinate domain. There is no partition worth choosing until traffic says where the boundaries should fall, so the sentinel starts with the one cell it can justify and refines outward from there. |
//! | [`fresh_graph_has_one_terminal`] | routing | cites (´claim:routing:a-fresh-graph-is-one-root-cell-covering-the-whole-domain-with-nothing-accumulated´) |
//! | [`fresh_graph_has_zero_total_sum`] | routing | cites (´claim:routing:a-fresh-graph-is-one-root-cell-covering-the-whole-domain-with-nothing-accumulated´) |
//! | [`ingest_feeds_graph_with_delta_one`] | routing | Every value in a batch contributes exactly one unit of importance, whichever range it falls in, so the graph's total after successive batches into unrelated ranges is simply how many values were handed over. Importance is a count of arrivals rather than a weight the caller can set, which is what lets the selector read it as evidence of where traffic is. |
//! | [`empty_ingest_does_not_observe`] | routing | A batch with nothing in it is not an event. The accumulated total stays where it was and no cell is created, so an interval in which nothing arrived neither adds evidence nor moves the partition. Quiet time is therefore invisible to the spatial layer rather than being recorded as an observation of emptiness. |
//! | [`duplicate_values_each_contribute`] | routing | cites (´claim:routing:every-ingested-value-adds-exactly-one-unit-of-importance-wherever-it-lands´) |
//! | [`graph_accumulates_across_batches`] | routing | cites (´claim:routing:every-ingested-value-adds-exactly-one-unit-of-importance-wherever-it-lands´) |
//! | [`lifetime_observations_tracks_total_sum`] | routing | The counter the sentinel keeps and the total the graph accumulates stay equal batch after batch. They are one quantity read from two layers: real observations are the only thing that increments either, and the synthetic data used to warm trackers is deliberately kept out of both. A host can therefore read whichever is nearer to hand without learning which layer maintains it. |
//! | [`concentrated_traffic_splits_nodes`] | routing | Traffic that keeps landing in one narrow range drives that range past the split threshold and the graph refines it, so the partition is bought with observations rather than configured up front. Where the traffic goes is where the resolution appears; the total meanwhile still counts exactly the values handed over, so refining a region does not manufacture evidence. |
//! | [`concentrated_traffic_grows_terminals`] | routing | cites (´claim:routing:concentrated-traffic-buys-resolution-by-splitting-the-range-it-lands-in´) |
//! | [`diverse_traffic_respects_budget`] | routing | Traffic spread thinly over many well-separated ranges asks the graph to refine everywhere at once, and the node budget is what keeps that from being unbounded: however many ranges are busy and however low the split threshold is set, the graph holds no more cells than the budget allows. The cost of modelling is a configured ceiling rather than a function of how widely an adversary chooses to scatter. |
//! | [`reset_restores_fresh_graph_state`] | routing | Reset discards the partition as well as the evidence: a graph that had split under load comes back as the single root cell of a fresh sentinel, with nothing accumulated and nothing to land in but the root. Structure is derived from observations, so once the observations are dropped there is no refinement left worth preserving, and a reset sentinel cannot be distinguished from a new one by what its graph holds. |

//! Graph routing — how an observed value reaches the cell that will
//! analyse it.
//!
//! Ingestion has one job at the spatial layer: hand every raw value to the
//! G-V Graph as a single unit of importance. Nothing is weighted, discounted
//! for repetition, or batched away, so the graph's accumulated total is a
//! count of arrivals and nothing else. That is what makes the sentinel's own
//! lifetime counter and the graph's total two readings of one number rather
//! than two tallies that could drift apart.
//!
//! Structure then follows traffic. A range that keeps receiving values crosses
//! the split threshold and is refined into finer cells, which is how the
//! sentinel comes to model where traffic actually is rather than a partition
//! chosen in advance. Refinement is not free, so it is bounded: the node
//! budget caps how many cells the graph will hold however many distinct
//! ranges the traffic touches. Spreading traffic thinly across the domain
//! therefore costs a configured ceiling of memory rather than unbounded
//! growth, which matters because that spread is something an adversary
//! chooses.
//!
//! Both ends of that lifecycle are observable. A fresh sentinel is one root
//! cell spanning the whole domain with nothing accumulated, and `reset()`
//! returns it to exactly that state — the learned partition is derived from
//! observations, so discarding the observations leaves nothing worth keeping
//! behind.

mod common;

use common::{cell_values, test_config};
use torrust_sentinel::{Sentinel128, SentinelConfig};

// ── Fresh state ─────────────────────────────────────────────

/// Before anything is observed the graph is a single cell spanning the whole
/// coordinate domain. There is no partition worth choosing until traffic says
/// where the boundaries should fall, so the sentinel starts with the one cell
/// it can justify and refines outward from there.
///
/// ´claim:routing:a-fresh-graph-is-one-root-cell-covering-the-whole-domain-with-nothing-accumulated´
/// ´test:integration:fresh-graph-has-one-node´
#[test]
fn fresh_graph_has_one_node() {
    let s = Sentinel128::new(test_config()).unwrap();
    assert_eq!(s.graph().node_count(), 1);
}

/// That single cell is also a leaf. Nothing has been split, so the one node
/// the graph holds is the one place an observation can land, and the count of
/// cells traffic can reach agrees with the count of nodes that exist.
///
/// (´claim:routing:a-fresh-graph-is-one-root-cell-covering-the-whole-domain-with-nothing-accumulated´)
/// ´test:integration:fresh-graph-has-one-terminal´
#[test]
fn fresh_graph_has_one_terminal() {
    let s = Sentinel128::new(test_config()).unwrap();
    assert_eq!(s.graph().terminal_count(), 1);
}

/// The other half of the fresh state: that root cell carries no accumulated
/// importance either. A cell exists because the domain has to be covered, not
/// because anything was seen in it, so structure and evidence start out
/// independent of one another.
///
/// (´claim:routing:a-fresh-graph-is-one-root-cell-covering-the-whole-domain-with-nothing-accumulated´)
/// ´test:integration:fresh-graph-has-zero-total-sum´
#[test]
fn fresh_graph_has_zero_total_sum() {
    let s = Sentinel128::new(test_config()).unwrap();
    assert_eq!(s.graph().total_sum(), 0);
}

// ── Basic routing ───────────────────────────────────────────

/// Every value in a batch contributes exactly one unit of importance,
/// whichever range it falls in, so the graph's total after successive batches
/// into unrelated ranges is simply how many values were handed over.
/// Importance is a count of arrivals rather than a weight the caller can set,
/// which is what lets the selector read it as evidence of where traffic is.
///
/// ´claim:routing:every-ingested-value-adds-exactly-one-unit-of-importance-wherever-it-lands´
/// ´test:integration:ingest-feeds-graph-with-delta-one´
#[test]
fn ingest_feeds_graph_with_delta_one() {
    let cfg = test_config();
    let mut s = Sentinel128::new(cfg).unwrap();

    // First batch: 5 values.
    s.ingest(&cell_values(0xA, 5));
    assert_eq!(s.graph().total_sum(), 5);

    // Second batch: 3 more values in a different range.
    s.ingest(&cell_values(0x3, 3));
    assert_eq!(s.graph().total_sum(), 8);
}

/// A batch with nothing in it is not an event. The accumulated total stays
/// where it was and no cell is created, so an interval in which nothing
/// arrived neither adds evidence nor moves the partition. Quiet time is
/// therefore invisible to the spatial layer rather than being recorded as an
/// observation of emptiness.
///
/// ´claim:routing:an-empty-batch-routes-nothing-and-leaves-the-graph-exactly-as-it-was´
/// ´test:integration:empty-ingest-does-not-observe´
#[test]
fn empty_ingest_does_not_observe() {
    let cfg = test_config();
    let mut s = Sentinel128::new(cfg).unwrap();

    s.ingest(&[]);
    assert_eq!(s.graph().total_sum(), 0);
    assert_eq!(s.graph().node_count(), 1); // still just the root
}

/// Repetition is traffic, not redundancy: one value handed over several times
/// in a batch counts several times over rather than collapsing into a single
/// arrival. The graph measures how often a region is visited, not how many
/// distinct values it has ever seen, which is why a flood from a single source
/// still moves the structure.
///
/// (´claim:routing:every-ingested-value-adds-exactly-one-unit-of-importance-wherever-it-lands´)
/// ´test:integration:duplicate-values-each-contribute´
#[test]
fn duplicate_values_each_contribute() {
    let cfg = test_config();
    let mut s = Sentinel128::new(cfg).unwrap();

    // Ingest three identical values — each should add 1 to total_sum.
    let v = cell_values(0xB, 1)[0];
    s.ingest(&[v, v, v]);
    assert_eq!(s.graph().total_sum(), 3);
}

// ── Accumulation ────────────────────────────────────────────

/// The same unit contribution survives batch boundaries: a long run of
/// batches scattered over many ranges leaves a total equal to everything ever
/// handed over. A batch is a delivery convenience, not an accounting period,
/// so the graph reports lifetime evidence rather than the most recent window.
///
/// (´claim:routing:every-ingested-value-adds-exactly-one-unit-of-importance-wherever-it-lands´)
/// ´test:integration:graph-accumulates-across-batches´
#[test]
fn graph_accumulates_across_batches() {
    let cfg = test_config();
    let mut s = Sentinel128::new(cfg).unwrap();

    for nibble in 0..10u128 {
        s.ingest(&cell_values(nibble, 100));
    }

    assert_eq!(s.graph().total_sum(), 1000);
}

/// The counter the sentinel keeps and the total the graph accumulates stay
/// equal batch after batch. They are one quantity read from two layers: real
/// observations are the only thing that increments either, and the synthetic
/// data used to warm trackers is deliberately kept out of both. A host can
/// therefore read whichever is nearer to hand without learning which layer
/// maintains it.
///
/// ´claim:routing:the-sentinels-lifetime-counter-and-the-graphs-total-are-one-quantity-read-twice´
/// ´test:integration:lifetime-observations-tracks-total-sum´
#[test]
fn lifetime_observations_tracks_total_sum() {
    let cfg = test_config();
    let mut s = Sentinel128::new(cfg).unwrap();

    s.ingest(&cell_values(0xC, 42));
    assert_eq!(s.lifetime_observations(), s.graph().total_sum());

    // Still holds after a second batch.
    s.ingest(&cell_values(0xD, 58));
    assert_eq!(s.lifetime_observations(), s.graph().total_sum());
    assert_eq!(s.lifetime_observations(), 100);
}

// ── Structural evolution ────────────────────────────────────

/// Traffic that keeps landing in one narrow range drives that range past the
/// split threshold and the graph refines it, so the partition is bought with
/// observations rather than configured up front. Where the traffic goes is
/// where the resolution appears; the total meanwhile still counts exactly the
/// values handed over, so refining a region does not manufacture evidence.
///
/// ´claim:routing:concentrated-traffic-buys-resolution-by-splitting-the-range-it-lands-in´
/// ´test:integration:concentrated-traffic-splits-nodes´
#[test]
fn concentrated_traffic_splits_nodes() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // All values in the same nibble range — should concentrate
    // into one cell and eventually split it.
    s.ingest(&cell_values(0xA, 50));

    assert_eq!(s.graph().total_sum(), 50);
    assert!(
        s.graph().node_count() > 1,
        "expected splits from concentrated traffic, got {} nodes",
        s.graph().node_count()
    );
}

/// Refinement creates new places for traffic to land, not merely new interior
/// structure above the old cell: after a split the graph has more than one
/// leaf. Subsequent values in that range are therefore separated from one
/// another instead of continuing to pile into a single undifferentiated cell.
///
/// (´claim:routing:concentrated-traffic-buys-resolution-by-splitting-the-range-it-lands-in´)
/// ´test:integration:concentrated-traffic-grows-terminals´
#[test]
fn concentrated_traffic_grows_terminals() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    s.ingest(&cell_values(0xA, 50));

    assert!(
        s.graph().terminal_count() > 1,
        "expected terminal_count > 1 after splits, got {}",
        s.graph().terminal_count()
    );
}

// ── Budget enforcement ──────────────────────────────────────

/// Traffic spread thinly over many well-separated ranges asks the graph to
/// refine everywhere at once, and the node budget is what keeps that from
/// being unbounded: however many ranges are busy and however low the split
/// threshold is set, the graph holds no more cells than the budget allows.
/// The cost of modelling is a configured ceiling rather than a function of how
/// widely an adversary chooses to scatter.
///
/// ´claim:routing:the-node-budget-caps-the-graph-however-widely-traffic-is-scattered´
/// ´test:integration:diverse-traffic-respects-budget´
#[test]
fn diverse_traffic_respects_budget() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 5,
        budget: 200,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Spread observations across many distinct nibble ranges.
    for nibble in 0..16u128 {
        let values: Vec<u128> = (0u128..500).map(|i| (nibble << 124) | (i << 100)).collect();
        s.ingest(&values);
    }

    assert!(
        s.graph().node_count() <= 200,
        "node count {} exceeds budget 200",
        s.graph().node_count()
    );
}

// ── Reset ───────────────────────────────────────────────────

/// Reset discards the partition as well as the evidence: a graph that had
/// split under load comes back as the single root cell of a fresh sentinel,
/// with nothing accumulated and nothing to land in but the root. Structure is
/// derived from observations, so once the observations are dropped there is no
/// refinement left worth preserving, and a reset sentinel cannot be
/// distinguished from a new one by what its graph holds.
///
/// ´claim:routing:reset-returns-the-graph-to-the-single-root-cell-of-a-fresh-sentinel´
/// ´test:integration:reset-restores-fresh-graph-state´
#[test]
fn reset_restores_fresh_graph_state() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Ingest enough to split.
    s.ingest(&cell_values(0xA, 100));
    assert!(s.graph().total_sum() > 0);
    assert!(s.graph().node_count() > 1);

    s.reset();

    assert_eq!(s.graph().total_sum(), 0);
    assert_eq!(s.graph().node_count(), 1);
    assert_eq!(s.graph().terminal_count(), 1);
}
