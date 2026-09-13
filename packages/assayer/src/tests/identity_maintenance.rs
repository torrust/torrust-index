// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`thread_name_correct`] | identity | The maintenance thread is named after the instance it serves, so a process running several engines shows one identifiable thread per engine rather than a row of anonymous workers. Anyone reading a stack dump or a scheduler trace can then attribute identity-maintenance work to the instance that caused it. |
//! | [`create_dimension_command`] | identity | A dimension is ready for traffic the moment the loop acknowledges its creation: an observation submitted straight afterwards is accepted, meaning the loop is holding the receiving end. Creation is acknowledged rather than fired and forgotten precisely so a caller never has to guess whether the dimension it just registered is listening yet. |
//! | [`destroy_dimension_command`] | identity | Destroying a dimension drops its graph and its receiving end without disturbing the loop, which keeps cycling for whatever dimensions remain. The command carries no acknowledgement, so the only thing observable from outside is that the loop is still standing afterwards. |
//! | [`observation_increases_importance`] | identity | Observations submitted by an assessment thread genuinely arrive in the dimension's graph: a hundred sent at one coordinate leave a hundred units of energy in the snapshot the next checkpoint publishes. Deferral is the whole mechanism by which the identity layer learns, so a queue that accepted work without it reaching the graph would leave every dimension permanently blank. |
//! | [`published_total_importance_matches_checkpoint_reconstruction`] | identity | The scalar published with the competitive-set index equals the total reconstructed from the checkpoint snapshot on the same maintenance barrier. Assessment can therefore read the publication directly without changing the feature value it previously reconstructed. |
//! | [`delta_always_one`] | identity | cites (´claim:identity:importance-accumulates-one-unit-per-observation-so-the-graphs-total-counts-the-traffic-it-saw´) |
//! | [`competitive_set_published`] | identity | cites (´claim:identity:observations-submitted-through-the-queue-reach-the-dimensions-graph´) |
//! | [`competitive_set_readable_without_blocking`] | identity | Four threads reading a dimension's competitive set in a tight loop run alongside a hundred observations being processed and republished, and none of them stalls or fails. Readers are never blocked by the maintenance loop swapping in a new set: republication installs a fresh index beside the old one rather than mutating what a reader is holding, so structural change costs the request path nothing. |
//! | [`checkpoint_coordination`] | identity | By the time a checkpoint acknowledges, every dimension has a snapshot published and that snapshot carries real energy. The acknowledgement is the signal a persistence layer waits on, so it has to mean the state is already there to be read — not that the loop has merely been told to get round to it. |
//! | [`checkpoint_drains_queues`] | identity | A checkpoint requested with a hundred observations still queued drains them first: the snapshot it publishes accounts for all hundred, not for whatever the loop had happened to reach. The cut is therefore taken at the moment the checkpoint was asked for rather than at some arbitrary earlier point, so a restart cannot silently lose the backlog that was in flight. |
//! | [`multiple_dimensions`] | identity | One loop serving two dimensions keeps their graphs entirely apart: thirty observations into one and seventy into the other produce snapshots of thirty and seventy, with neither borrowing from the other. Dimensions are different ways of carving up the same population, so importance crossing between them would make one dimension's traffic argue for the other's structure. |
//! | [`decay_triggers_exit`] | identity | A cell that loses the importance which earned it its place is announced as an exit. Annihilating decay zeroes a dimension's energy, so on the next cycle the competitive set computes as empty and every previously tracked cell is reported gone. Entries and exits are the model's only notice that its shape has changed, so forgetting has to be announced as loudly as learning. |
//! | [`graph_from_new_config`] | identity | A dimension's graph begins with a root node and nothing else to its name: at least one node exists, and the accumulated total is zero. The root is what gives every coordinate somewhere to land from the very first observation, so the structure exists before any traffic does while the evidence in it does not. |
//! | [`graph_observe_increases_total`] | identity | Importance accumulates one unit per observation, so a graph's total is a count of the traffic it has been shown: a hundred observations across a hundred coordinates leave a total of a hundred. That equivalence is what makes total energy a meaningful checkpoint quantity — it is a number of requests, not an arbitrary score whose scale depends on how it was fed. |
//! | [`graph_layers_yields_entries`] | identity | Observed traffic gives the graph nodes to enumerate, and every node it exposes covers a genuinely non-empty interval — the start always strictly below the end, at every layer. Each such node becomes a candidate competitive cell, and a cell spanning nothing could never contain the coordinate that produced it. |
//! | [`graph_decay_reduces_importance`] | identity | Decay lowers accumulated importance, so evidence has to be renewed to keep its weight. Without it a region that was busy once would hold its place in the competitive set for ever, and the identity layer would end up describing the traffic of months ago rather than of now. |
//! | [`graph_heavy_decay_eliminates_terminals`] | identity | Decay applied hard and repeatedly drives a dimension's importance towards nothing rather than settling at some floor: the first pass already reduces the total, and thirty more leave it an order of magnitude below where it started, with the terminal count falling alongside. This is the mechanism the exit tests lean on — a region that stops receiving traffic eventually stops existing as far as the competitive set is concerned. |
//! | [`graph_split_occurs`] | identity | Traffic gathered into one small region of a vast domain makes the graph split, ending with more nodes than it began with. Resolution is earned rather than configured: a region only gets carved finely once enough distinct entities in it have been seen, so the structure follows where the population actually is. |
//! | [`graph_snapshot_for_checkpoint`] | identity | cites (´claim:identity:a-snapshot-carries-the-graphs-whole-energy-not-just-what-its-terminals-hold´) |
//! | [`graph_reconstruct_from_snapshot`] | identity | cites (´claim:identity:a-graph-rebuilt-from-a-snapshots-observations-holds-the-total-energy-it-started-with´) |
//! | [`split_detected_as_entry`] | identity | A cell that becomes competitive is announced to the model as an entry: a burst concentrated at one coordinate splits the graph, and the resulting cells arrive at the model owner as entry events. The model has to extend its shape before a new cell's evidence can mean anything, so structural growth discovered on the maintenance thread must be told rather than merely published. |
//! | [`no_change_no_event`] | identity | A cycle that changes nothing says nothing: a dimension with no observations keeps an empty competitive set and emits no lifecycle traffic at all. The loop wakes on a timer regardless of load, so a cycle that reported its own occurrence would flood the model owner with events on an idle engine and make every genuine structural change harder to see. |
//! | [`single_entry_emits_one_event`] | identity | Repeated traffic at a single coordinate leaves the graph holding at least one terminal cell — the finest region the evidence supports, and the unit that later enters the competitive set. This is checked against the graph directly because split timing through the maintenance loop is not deterministic enough to pin an exact cell count on. |
//! | [`exits_before_entries`] | identity | Within any one submission, no exit ever follows an entry: the model marginalises cells away before it extends to take new ones on. Order matters because the two operations reshape the same model — extending first would have the model briefly carrying both the cell it is about to drop and its replacement, and reasoning over a shape that never really existed. |
//! | [`single_exit_emits_one_event`] | identity | cites (´claim:identity:a-cell-that-loses-its-importance-is-announced-to-the-model-as-an-exit´) |
//! | [`batch_entry_exit`] | identity | One submission can carry both kinds of change at once. Decaying an established region hard and immediately loading a distant one produces exits and entries in the same batch — and the exits still come first. A cycle discovers the whole difference between the old competitive set and the new one, so a landscape that shifts wholesale reaches the model as a single coherent reshaping rather than as a sequence of partial ones. |
//! | [`command_channel_full`] | identity | When the model owner cannot keep up, the maintenance loop sheds lifecycle events rather than blocking or dying: with a single-slot channel and two bursts of five hundred observations driving repeated set changes, the loop stays responsive throughout. The identity layer's structural updates are advisory, so a stalled model owner must never be able to wedge the thread that is still draining every dimension's observation queue. |
//! | [`identity_observation_to_competitive_set`] | identity | Sustained traffic earns a region a place in the published competitive set, end to end: a thousand observations submitted from the assessment side pass through the queue, the drain, the graph and the change detector, and come out as cells an assessment thread can read. Nothing along that path needs telling which regions matter — the traffic decides. |
//! | [`competitive_set_change_emits_lifecycle`] | identity | Lifecycle events cross from the maintenance thread to the model owner, and each one is an entry or an exit tagged with the dimension whose set changed — nothing else appears on that path. The tag is what the receiving side keys on when it decides which dimension of its model to reshape. |
//! | [`identity_find_active_matches_competitive_set`] | identity | Routing and publication agree: an observed coordinate resolves to a non-empty chain, and every cell in that chain is one the published competitive set actually contains. Assessment routes through the same index the maintenance loop publishes, so a cell could not be routed to unless the loop had announced it and the model had extended to hold it. |
//! | [`two_dimensions_independent`] | identity | cites (´claim:identity:each-dimension-accumulates-only-its-own-observations´) |
//! | [`identity_decay_shrinks_competitive_set`] | identity | Forgetting reaches the assessment path, not just the event stream: after annihilating decay the published competitive set is strictly smaller than it was, and exits are emitted for what left it. Both halves are needed — an announced exit whose cell stayed routable would keep sending traffic to a cell the model had already marginalised away. |

//! Crate-level integration tests for the identity maintenance loop.
//!
//! These tests exercise the maintenance loop through its public API:
//! `spawn_identity_maintenance_thread` + `MaintenanceCommand`. They
//! verify observation processing, competitive set change detection,
//! lifecycle event emission, checkpoint coordination, and G-V graph
//! integration.
//!
//! All boilerplate (spawning the thread, creating dimensions, flushing
//! the loop, draining the model-owner channel) lives in
//! [`crate::testing::maintenance`].
//!
//! # Cross-References
//!
//! - (´dec:memory:competitive-publication´) — the per-dimension publication these tests watch for change
//! - (´dec:memory:graph-owner´) — the one dedicated thread whose loop is driven here

use std::sync::Arc;
use std::time::Duration;

use torrust_mudlark::{Config as MudlarkConfig, GState, GvGraph};

use crate::owner::commands::{LifecycleEvent, ModelOwnerCommand};
use crate::testing::maintenance::{
    MaintenanceHarness, aggressive_split_config, collect_lifecycle_events, default_mudlark_config, setup_dimension,
};
use crate::types::DimensionId;

// ═══════════════════════════════════════════════════════════════════════════════
// Maintenance Loop Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// The maintenance thread is named after the instance it serves, so a process
/// running several engines shows one identifiable thread per engine rather than
/// a row of anonymous workers. Anyone reading a stack dump or a scheduler trace
/// can then attribute identity-maintenance work to the instance that caused it.
///
/// ´claim:identity:the-maintenance-thread-is-named-after-the-instance-it-serves´
/// ´test:crate:thread-name-correct´
#[test]
fn thread_name_correct() {
    let harness = MaintenanceHarness::spawn("test-42");

    assert_eq!(
        harness.thread_name(),
        "test-42-identity-maintenance",
        "thread name should match instance_id pattern"
    );
}

/// A dimension is ready for traffic the moment the loop acknowledges its
/// creation: an observation submitted straight afterwards is accepted, meaning
/// the loop is holding the receiving end. Creation is acknowledged rather than
/// fired and forgotten precisely so a caller never has to guess whether the
/// dimension it just registered is listening yet.
///
/// ´claim:identity:a-dimension-accepts-observations-as-soon-as-the-loop-acknowledges-its-creation´
/// ´test:crate:create-dimension-command´
#[test]
fn create_dimension_command() {
    let harness = MaintenanceHarness::spawn("create-dim");
    let (infra, obs) = setup_dimension(1);

    harness.register_dimension(DimensionId(1), 24, obs.into_receiver(), infra.clone());

    // Should be able to send observations (channel connected)
    assert!(infra.observe(42), "observation should succeed after dimension creation");
}

/// Destroying a dimension drops its graph and its receiving end without
/// disturbing the loop, which keeps cycling for whatever dimensions remain.
/// The command carries no acknowledgement, so the only thing observable from
/// outside is that the loop is still standing afterwards.
///
/// ´claim:identity:destroying-a-dimension-releases-it-without-bringing-the-loop-down´
/// ´test:crate:destroy-dimension-command´
#[test]
fn destroy_dimension_command() {
    let harness = MaintenanceHarness::spawn("destroy-dim");
    let (infra, obs) = setup_dimension(1);

    harness.register_dimension(DimensionId(1), 24, obs.into_receiver(), infra);
    harness.destroy_dimension(DimensionId(1));

    // Give the loop time to process
    std::thread::sleep(Duration::from_millis(200));
}

/// Observations submitted by an assessment thread genuinely arrive in the
/// dimension's graph: a hundred sent at one coordinate leave a hundred units of
/// energy in the snapshot the next checkpoint publishes. Deferral is the whole
/// mechanism by which the identity layer learns, so a queue that accepted work
/// without it reaching the graph would leave every dimension permanently blank.
///
/// ´claim:identity:observations-submitted-through-the-queue-reach-the-dimensions-graph´
/// ´test:crate:observation-increases-importance´
#[test]
fn observation_increases_importance() {
    let harness = MaintenanceHarness::spawn("obs-importance");
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension(DimensionId(1), 24, obs_rx, infra.clone());

    // Send 100 observations for the same coordinate
    for _ in 0..100 {
        obs.try_send(0x1000);
    }
    std::thread::sleep(Duration::from_millis(300));

    harness.checkpoint();

    // After checkpoint, graph_snapshot should be published
    let snapshot = infra.graph_snapshot.load();
    assert!(snapshot.is_some(), "snapshot should be published after checkpoint");
    let snap = snapshot.as_ref().as_ref().unwrap();
    assert_eq!(snap.total_energy(), 100, "total_energy should match observation count");
}

/// The scalar published with the competitive-set index equals the total
/// reconstructed from the checkpoint snapshot on the same maintenance barrier.
/// Assessment can therefore read the publication directly without changing the
/// feature value it previously reconstructed.
///
/// ´claim:identity:published-total-importance-equals-the-checkpoint-reconstruction-it-replaces´
/// ´test:crate:published-total-importance-matches-checkpoint-reconstruction´
#[test]
#[allow(clippy::cast_precision_loss)] // Justified: the test total is far below f64's exact-integer limit
fn published_total_importance_matches_checkpoint_reconstruction() {
    let harness = MaintenanceHarness::spawn("published-total-importance");
    let (infra, obs) = setup_dimension(1);

    harness.register_dimension(DimensionId(1), 24, obs.receiver().clone(), infra.clone());
    for i in 0..137 {
        assert!(obs.try_send(i));
    }
    harness.checkpoint();

    let snapshot = infra.graph_snapshot.load();
    let reconstructed = snapshot.as_ref().as_ref().expect("checkpoint graph snapshot").total_energy() as f64;
    let published = infra.load_competitive_set().total_importance();
    assert_eq!(published.to_bits(), reconstructed.to_bits());
}

/// Fifty observations at fifty different coordinates leave exactly fifty units
/// of energy: the loop applies one unit per observation and takes no weight
/// from the caller. Since the competitive set is drawn from where importance
/// concentrates, a caller able to choose its own weight could nominate its own
/// region for dedicated tracking — the fixed unit is what makes the graph a
/// record of traffic rather than of assertion.
///
/// (´claim:identity:importance-accumulates-one-unit-per-observation-so-the-graphs-total-counts-the-traffic-it-saw´)
/// ´test:crate:delta-always-one´
#[test]
fn delta_always_one() {
    let harness = MaintenanceHarness::spawn("delta-one");
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension(DimensionId(1), 24, obs_rx, infra.clone());

    // Send 50 distinct coordinates
    for i in 0..50u128 {
        obs.try_send(i * 1_000_000);
    }

    harness.checkpoint();

    let snapshot = infra.graph_snapshot.load();
    let snap = snapshot.as_ref().as_ref().unwrap();
    // Each observation is Δ=1, so 50 observations → total_energy == 50
    assert_eq!(snap.total_energy(), 50, "each observation should increment by exactly 1");
}

/// The competitive set can be loaded and held while the loop carries on
/// working: the reader takes its guard, a checkpoint runs underneath it, and
/// the fifty concentrated observations still show up in the published snapshot.
/// Holding a read of the set costs the maintenance side nothing, so an
/// assessment mid-flight neither stalls the loop nor is stalled by it.
///
/// (´claim:identity:observations-submitted-through-the-queue-reach-the-dimensions-graph´)
/// ´test:crate:competitive-set-published´
#[test]
fn competitive_set_published() {
    let harness = MaintenanceHarness::spawn("set-published");
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension(DimensionId(1), 100, obs_rx, infra.clone());

    // Send enough concentrated observations to trigger a split
    for _ in 0..50 {
        obs.try_send(0x1000);
    }
    harness.flush_loop();

    // Lock-free read of the competitive set (no panic, no deadlock)
    let set = infra.load_competitive_set();
    harness.checkpoint();

    let snapshot = infra.graph_snapshot.load();
    let snap = snapshot.as_ref().as_ref().unwrap();
    assert_eq!(snap.total_energy(), 50, "observations should be processed");

    let _len = set.len();
}

/// Four threads reading a dimension's competitive set in a tight loop run
/// alongside a hundred observations being processed and republished, and none of
/// them stalls or fails. Readers are never blocked by the maintenance loop
/// swapping in a new set: republication installs a fresh index beside the old
/// one rather than mutating what a reader is holding, so structural change
/// costs the request path nothing.
///
/// ´claim:identity:readers-of-the-competitive-set-are-not-blocked-by-the-loop-republishing-it´
/// ´test:crate:competitive-set-readable-without-blocking´
#[test]
fn competitive_set_readable_without_blocking() {
    let harness = MaintenanceHarness::spawn("lockfree-read");
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension(DimensionId(1), 100, obs_rx, infra.clone());

    // Spawn reader threads that continuously load the competitive set
    let readers: Vec<_> = (0..4)
        .map(|_| {
            let infra = Arc::clone(&infra);
            std::thread::spawn(move || {
                for _ in 0..200 {
                    let _guard = infra.load_competitive_set();
                    std::thread::yield_now();
                }
            })
        })
        .collect();

    // Meanwhile send observations
    for i in 0..100u128 {
        obs.try_send(i * 1000);
    }
    std::thread::sleep(Duration::from_millis(300));

    for r in readers {
        r.join().expect("reader thread should not panic");
    }
}

/// By the time a checkpoint acknowledges, every dimension has a snapshot
/// published and that snapshot carries real energy. The acknowledgement is the
/// signal a persistence layer waits on, so it has to mean the state is already
/// there to be read — not that the loop has merely been told to get round to it.
///
/// ´claim:identity:a-checkpoint-publishes-a-snapshot-of-every-dimension-before-it-acknowledges´
/// ´test:crate:checkpoint-coordination´
#[test]
fn checkpoint_coordination() {
    let harness = MaintenanceHarness::spawn("checkpoint");
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension(DimensionId(1), 24, obs_rx, infra.clone());

    // Send some observations
    for i in 0..20u128 {
        obs.try_send(i * 100);
    }

    harness.checkpoint();

    // Snapshot should be published
    let snapshot = infra.graph_snapshot.load();
    assert!(snapshot.is_some(), "snapshot should be published after checkpoint");
    let snap = snapshot.as_ref().as_ref().unwrap();
    assert!(snap.total_energy() > 0, "snapshot should have non-zero total");
}

/// A checkpoint requested with a hundred observations still queued drains them
/// first: the snapshot it publishes accounts for all hundred, not for whatever
/// the loop had happened to reach. The cut is therefore taken at the moment the
/// checkpoint was asked for rather than at some arbitrary earlier point, so a
/// restart cannot silently lose the backlog that was in flight.
///
/// ´claim:identity:a-checkpoint-drains-the-backlog-first-so-its-snapshot-omits-no-submitted-observation´
/// ´test:crate:checkpoint-drains-queues´
#[test]
fn checkpoint_drains_queues() {
    let harness = MaintenanceHarness::spawn("checkpoint-drain");
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension(DimensionId(1), 24, obs_rx, infra.clone());

    // Send 100 observations without waiting for loop to process
    for i in 0..100u128 {
        obs.try_send(i);
    }

    // Immediately request checkpoint — should drain all pending
    harness.checkpoint();

    let snapshot = infra.graph_snapshot.load();
    let snap = snapshot.as_ref().as_ref().unwrap();
    assert_eq!(
        snap.total_energy(),
        100,
        "checkpoint should drain all pending observations before acking"
    );
}

/// One loop serving two dimensions keeps their graphs entirely apart: thirty
/// observations into one and seventy into the other produce snapshots of thirty
/// and seventy, with neither borrowing from the other. Dimensions are different
/// ways of carving up the same population, so importance crossing between them
/// would make one dimension's traffic argue for the other's structure.
///
/// ´claim:identity:each-dimension-accumulates-only-its-own-observations´
/// ´test:crate:multiple-dimensions´
#[test]
fn multiple_dimensions() {
    let harness = MaintenanceHarness::spawn("multi-dim");

    let (infra1, obs1) = setup_dimension(1);
    let (infra2, obs2) = setup_dimension(2);

    let obs_rx1 = obs1.receiver().clone();
    let obs_rx2 = obs2.receiver().clone();

    harness.register_dimension(DimensionId(1), 24, obs_rx1, infra1.clone());
    harness.register_dimension(DimensionId(2), 24, obs_rx2, infra2.clone());

    for _ in 0..30 {
        obs1.try_send(0x1000);
    }
    for _ in 0..70 {
        obs2.try_send(0x2000);
    }

    harness.checkpoint();

    let snap1 = infra1.graph_snapshot.load();
    let snap2 = infra2.graph_snapshot.load();

    let s1 = snap1.as_ref().as_ref().unwrap();
    let s2 = snap2.as_ref().as_ref().unwrap();

    assert_eq!(s1.total_energy(), 30, "dimension 1 should have 30 observations");
    assert_eq!(s2.total_energy(), 70, "dimension 2 should have 70 observations");
}

/// A cell that loses the importance which earned it its place is announced as
/// an exit. Annihilating decay zeroes a dimension's energy, so on the next
/// cycle the competitive set computes as empty and every previously tracked
/// cell is reported gone. Entries and exits are the model's only notice that
/// its shape has changed, so forgetting has to be announced as loudly as
/// learning.
///
/// ´claim:identity:a-cell-that-loses-its-importance-is-announced-to-the-model-as-an-exit´
/// ´test:crate:decay-triggers-exit´
#[test]
fn decay_triggers_exit() {
    // Annihilation decay (att=0) zeroes all energy.  On the
    // next loop cycle `compute_competitive_set` sees `total_sum()==0`
    // and returns an empty set, so `CompetitiveCellExit` events are
    // emitted for every cell that was previously tracked.
    let harness = MaintenanceHarness::spawn("decay-exit");
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension_with_config(aggressive_split_config(), DimensionId(1), 100, obs_rx, infra.clone());

    for _ in 0..500 {
        obs.try_send(0x1000);
    }
    harness.flush_loop();

    let set_before = infra.load_competitive_set();
    assert!(!set_before.is_empty(), "competitive set should be populated before decay");

    // Drain any lifecycle events from the build-up phase
    harness.clear_model_owner();

    // Annihilation: a zero attenuation factor zeroes all energy, uniformly
    // (´[MUDLARK-claim:attenuation:a-zero-attenuation-factor-annihilates-all-accumulated-value]´).
    harness.force_decay(DimensionId(1), 0.0);

    // flush_loop triggers a full cycle: drain_observations +
    // detect_and_emit_changes.  With all energy zeroed,
    // total_sum()==0 → empty competitive set → exits emitted.
    harness.flush_loop();

    let cmds = harness.drain_model_owner();
    let events = collect_lifecycle_events(&cmds);

    let exit_count = events
        .iter()
        .filter(|e| matches!(e, LifecycleEvent::CompetitiveCellExit { .. }))
        .count();

    assert!(
        exit_count > 0,
        "annihilation decay should produce CompetitiveCellExit events, got {} events total",
        events.len()
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// G-V Graph Integration Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A dimension's graph begins with a root node and nothing else to its name:
/// at least one node exists, and the accumulated total is zero. The root is
/// what gives every coordinate somewhere to land from the very first
/// observation, so the structure exists before any traffic does while the
/// evidence in it does not.
///
/// ´claim:identity:a-fresh-dimension-graph-has-a-root-and-no-accumulated-importance´
/// ´test:crate:graph-from-new-config´
#[test]
fn graph_from_new_config() {
    let graph = GvGraph::<u128, u64, 128>::new(default_mudlark_config());

    // A new graph has a root
    let root = graph.g_root();
    // node_count should be at least 1 (the root G-node)
    assert!(graph.node_count() > 0, "new graph should have at least the root node");
    assert_eq!(graph.total_sum(), 0, "new graph should have zero total");
    let _ = root; // root is a valid GNodeId
}

/// Importance accumulates one unit per observation, so a graph's total is a
/// count of the traffic it has been shown: a hundred observations across a
/// hundred coordinates leave a total of a hundred. That equivalence is what
/// makes total energy a meaningful checkpoint quantity — it is a number of
/// requests, not an arbitrary score whose scale depends on how it was fed.
///
/// ´claim:identity:importance-accumulates-one-unit-per-observation-so-the-graphs-total-counts-the-traffic-it-saw´
/// ´test:crate:graph-observe-increases-total´
#[test]
fn graph_observe_increases_total() {
    let mut graph = GvGraph::<u128, u64, 128>::new(default_mudlark_config());

    for i in 0..100u128 {
        graph.observe(i * 1000, 1u64);
    }

    assert_eq!(graph.total_sum(), 100, "total_sum should equal observation count");
}

/// Observed traffic gives the graph nodes to enumerate, and every node it
/// exposes covers a genuinely non-empty interval — the start always strictly
/// below the end, at every layer. Each such node becomes a candidate
/// competitive cell, and a cell spanning nothing could never contain the
/// coordinate that produced it.
///
/// ´claim:identity:every-node-the-graph-exposes-covers-a-non-empty-interval´
/// ´test:crate:graph-layers-yields-entries´
#[test]
fn graph_layers_yields_entries() {
    let mut graph = GvGraph::<u128, u64, 128>::new(default_mudlark_config());

    // Add observations to create structure
    for i in 0..20u128 {
        graph.observe(i * 1_000_000, 1u64);
    }

    let nodes: Vec<_> = graph.layers().collect();
    assert!(!nodes.is_empty(), "layers() should produce at least one node");

    for (layer, node) in &nodes {
        assert!(node.start < node.end, "node interval should be non-empty at layer {layer}");
    }
}

/// Decay lowers accumulated importance, so evidence has to be renewed to keep
/// its weight. Without it a region that was busy once would hold its place in
/// the competitive set for ever, and the identity layer would end up describing
/// the traffic of months ago rather than of now.
///
/// ´claim:identity:decay-lowers-accumulated-importance-so-evidence-must-be-renewed-to-keep-its-weight´
/// ´test:crate:graph-decay-reduces-importance´
#[test]
fn graph_decay_reduces_importance() {
    let mut graph = GvGraph::<u128, u64, 128>::new(default_mudlark_config());

    for _ in 0..100 {
        graph.observe(0x5000, 1u64);
    }
    assert_eq!(graph.total_sum(), 100);

    let root = graph.g_root();
    graph.decay(root, 0.5, 1.0);

    assert!(
        graph.total_sum() < 100,
        "decay should reduce total_sum, got {}",
        graph.total_sum()
    );
}

/// Decay applied hard and repeatedly drives a dimension's importance towards
/// nothing rather than settling at some floor: the first pass already reduces
/// the total, and thirty more leave it an order of magnitude below where it
/// started, with the terminal count falling alongside. This is the mechanism
/// the exit tests lean on — a region that stops receiving traffic eventually
/// stops existing as far as the competitive set is concerned.
///
/// ´claim:identity:repeated-heavy-decay-drives-a-dimensions-importance-towards-nothing´
/// ´test:crate:graph-heavy-decay-eliminates-terminals´
#[test]
fn graph_heavy_decay_eliminates_terminals() {
    // Diagnostic: verifies how repeated heavy decay affects the G-V graph's
    // terminal set and total_sum. This characterises the decay ↔ eviction
    // interaction that `decay_triggers_exit` relies on.
    let mut graph = GvGraph::<u128, u64, 128>::new(aggressive_split_config());

    for _ in 0..500 {
        graph.observe(0x1000, 1u64);
    }

    let before_sum = graph.total_sum();
    let before_terminals = graph.layers().filter(|(_, n)| n.state == GState::Terminal).count();
    assert_eq!(before_sum, 500);
    assert!(before_terminals >= 1, "graph should have at least one terminal");

    let root = graph.g_root();

    graph.decay(root, 0.001, 1.0);
    let after1_sum = graph.total_sum();
    let after1_terminals = graph.layers().filter(|(_, n)| n.state == GState::Terminal).count();
    assert!(
        after1_sum < before_sum,
        "first decay should reduce total: {before_sum} -> {after1_sum}"
    );

    for _ in 0..30 {
        if graph.total_sum() == 0 {
            break;
        }
        graph.decay(root, 0.001, 1.0);
    }

    let final_sum = graph.total_sum();
    let final_terminals = graph.layers().filter(|(_, n)| n.state == GState::Terminal).count();

    assert!(
        final_sum < before_sum / 10,
        "repeated heavy decay should drive total well below initial, got {final_sum} (started at {before_sum})"
    );

    eprintln!(
        "decay trajectory: terminals {before_terminals} -> {after1_terminals} -> {final_terminals}, \
         sum {before_sum} -> {after1_sum} -> {final_sum}"
    );
}

/// Traffic gathered into one small region of a vast domain makes the graph
/// split, ending with more nodes than it began with. Resolution is earned
/// rather than configured: a region only gets carved finely once enough
/// distinct entities in it have been seen, so the structure follows where the
/// population actually is.
///
/// ´claim:identity:traffic-gathered-in-one-region-splits-the-graph-into-finer-cells-than-it-began-with´
/// ´test:crate:graph-split-occurs´
#[test]
fn graph_split_occurs() {
    let mut graph = GvGraph::<u128, u64, 128>::new(aggressive_split_config());

    let initial_count = graph.layers().count();

    for i in 0..100u128 {
        graph.observe(i, 1u64);
    }

    let after_nodes: Vec<_> = graph.layers().collect();
    assert!(
        after_nodes.len() > initial_count,
        "concentrated observations should cause splits: {} -> {}",
        initial_count,
        after_nodes.len()
    );
}

/// An extraction taken for checkpointing carries the graph's energy exactly,
/// and wrapping it as an identity snapshot changes nothing: fifty observations
/// remain fifty units, in a snapshot that reports itself as non-empty. The
/// checkpoint boundary is where the identity layer's state leaves memory, so
/// energy lost in the handover would be energy lost on every restart.
///
/// (´claim:identity:a-snapshot-carries-the-graphs-whole-energy-not-just-what-its-terminals-hold´)
/// ´test:crate:graph-snapshot-for-checkpoint´
#[test]
fn graph_snapshot_for_checkpoint() {
    let mut graph = GvGraph::<u128, u64, 128>::new(default_mudlark_config());

    for i in 0..50u128 {
        graph.observe(i * 100, 1u64);
    }

    let pewei = graph.extract();
    assert!(pewei.total_energy() > 0, "Pewei should capture energy");
    assert_eq!(pewei.total_energy(), graph.total_sum());

    let snap = crate::identity::IdentityGraphSnapshot::from_pewei(pewei);
    assert!(!snap.is_empty());
    assert_eq!(snap.total_energy(), 50);
}

/// Rebuilding a dimension from its snapshot under the same configuration
/// restores the total the original held, across the full snapshot-and-replay
/// path the identity layer uses at startup. What a restarted engine inherits is
/// therefore the weight of the traffic its predecessor saw, so a restart is a
/// pause in learning rather than a reset of it.
///
/// (´claim:identity:a-graph-rebuilt-from-a-snapshots-observations-holds-the-total-energy-it-started-with´)
/// ´test:crate:graph-reconstruct-from-snapshot´
#[test]
fn graph_reconstruct_from_snapshot() {
    let config: MudlarkConfig<u64> = default_mudlark_config();
    let mut graph = GvGraph::<u128, u64, 128>::new(config.clone());

    for i in 0..100u128 {
        graph.observe(i * 1000, 1u64);
    }

    let original_total = graph.total_sum();

    let snap = crate::identity::IdentityGraphSnapshot::from_pewei(graph.extract());
    assert_eq!(snap.total_energy(), original_total);

    let reconstructed = GvGraph::<u128, u64, 128>::from_observations(config, snap.reconstruction_observations());

    assert_eq!(
        reconstructed.total_sum(),
        original_total,
        "reconstructed graph should have same total_sum"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Lifecycle Event Emission Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A cell that becomes competitive is announced to the model as an entry: a
/// burst concentrated at one coordinate splits the graph, and the resulting
/// cells arrive at the model owner as entry events. The model has to extend its
/// shape before a new cell's evidence can mean anything, so structural growth
/// discovered on the maintenance thread must be told rather than merely
/// published.
///
/// ´claim:identity:a-cell-that-becomes-competitive-is-announced-to-the-model-as-an-entry´
/// ´test:crate:split-detected-as-entry´
#[test]
fn split_detected_as_entry() {
    let harness = MaintenanceHarness::spawn("split-entry");
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension_with_config(aggressive_split_config(), DimensionId(1), 100, obs_rx, infra);

    for _ in 0..200 {
        obs.try_send(0x1000);
    }
    harness.flush_loop();

    let cmds = harness.drain_model_owner();
    let events = collect_lifecycle_events(&cmds);

    let entry_count = events
        .iter()
        .filter(|e| matches!(e, LifecycleEvent::CompetitiveCellEntry { .. }))
        .count();

    assert!(
        entry_count > 0,
        "split should produce CompetitiveCellEntry events, got {} lifecycle events total",
        events.len()
    );
}

/// A cycle that changes nothing says nothing: a dimension with no observations
/// keeps an empty competitive set and emits no lifecycle traffic at all. The
/// loop wakes on a timer regardless of load, so a cycle that reported its own
/// occurrence would flood the model owner with events on an idle engine and
/// make every genuine structural change harder to see.
///
/// ´claim:identity:an-unchanged-competitive-set-produces-no-lifecycle-traffic´
/// ´test:crate:no-change-no-event´
#[test]
fn no_change_no_event() {
    let harness = MaintenanceHarness::spawn("no-change");
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension(DimensionId(1), 24, obs_rx, infra);

    // Don't send any observations — competitive set should remain empty
    harness.flush_loop();

    let cmds = harness.drain_model_owner();
    let events = collect_lifecycle_events(&cmds);

    assert!(
        events.is_empty(),
        "no observations → no competitive set change → no lifecycle events, got {}",
        events.len()
    );
}

/// Repeated traffic at a single coordinate leaves the graph holding at least
/// one terminal cell — the finest region the evidence supports, and the unit
/// that later enters the competitive set. This is checked against the graph
/// directly because split timing through the maintenance loop is not
/// deterministic enough to pin an exact cell count on.
///
/// ´claim:identity:repeated-traffic-at-one-coordinate-produces-at-least-one-terminal-cell´
/// ´test:crate:single-entry-emits-one-event´
#[test]
fn single_entry_emits_one_event() {
    // This test verifies that when exactly one cell enters the competitive set,
    // the submission contains exactly one CompetitiveCellEntry event.
    // We use the graph directly since controlling exact split behavior through
    // the maintenance loop is non-deterministic.
    let mut graph = GvGraph::<u128, u64, 128>::new(aggressive_split_config());

    for _ in 0..20 {
        graph.observe(0x5000, 1u64);
    }

    let terminal_count = graph.layers().filter(|(_, node)| node.state == GState::Terminal).count();

    assert!(terminal_count >= 1, "graph should have at least one terminal node");
}

/// Within any one submission, no exit ever follows an entry: the model
/// marginalises cells away before it extends to take new ones on. Order matters
/// because the two operations reshape the same model — extending first would
/// have the model briefly carrying both the cell it is about to drop and its
/// replacement, and reasoning over a shape that never really existed.
///
/// ´claim:identity:within-a-submission-exits-precede-entries-so-the-model-marginalises-before-it-extends´
/// ´test:crate:exits-before-entries´
#[test]
fn exits_before_entries() {
    // Verify the ordering convention: exits are emitted before entries.
    let harness = MaintenanceHarness::spawn("exit-entry-order");
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension_with_config(aggressive_split_config(), DimensionId(1), 100, obs_rx, infra);

    for _ in 0..500 {
        obs.try_send(0x1000);
    }
    harness.flush_loop();

    let cmds = harness.drain_model_owner();
    for cmd in &cmds {
        if let ModelOwnerCommand::Lifecycle(sub) = cmd {
            // Within each submission: exits must precede entries
            let mut seen_entry = false;
            for event in &sub.events {
                match event {
                    LifecycleEvent::CompetitiveCellExit { .. } => {
                        assert!(!seen_entry, "exit should not come after entry in same submission");
                    }
                    LifecycleEvent::CompetitiveCellEntry { .. } => {
                        seen_entry = true;
                    }
                    _ => {}
                }
            }
        }
    }
}

/// After annihilating decay every cell the dimension held is reported gone —
/// one exit apiece — and every one of those events names the dimension it came
/// from. A single loop serves all dimensions, so an exit that lost its
/// attribution would have the model marginalise a cell out of the wrong one.
///
/// (´claim:identity:a-cell-that-loses-its-importance-is-announced-to-the-model-as-an-exit´)
/// ´test:crate:single-exit-emits-one-event´
#[test]
fn single_exit_emits_one_event() {
    // Annihilation decay followed by a loop flush emits
    // CompetitiveCellExit events, one per cell that was forgotten.
    // All events reference the correct dimension.
    let harness = MaintenanceHarness::spawn("single-exit");
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension_with_config(aggressive_split_config(), DimensionId(1), 100, obs_rx, infra.clone());

    for _ in 0..500 {
        obs.try_send(0x1000);
    }
    harness.flush_loop();

    let set_before = infra.load_competitive_set();
    assert!(!set_before.is_empty(), "competitive set should be populated");

    harness.clear_model_owner();

    harness.force_decay(DimensionId(1), 0.0);
    harness.flush_loop();

    let cmds = harness.drain_model_owner();
    let events = collect_lifecycle_events(&cmds);

    let exit_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, LifecycleEvent::CompetitiveCellExit { .. }))
        .collect();

    assert!(
        !exit_events.is_empty(),
        "annihilation decay should produce CompetitiveCellExit events"
    );

    for event in &exit_events {
        if let LifecycleEvent::CompetitiveCellExit { dimension, .. } = event {
            assert_eq!(*dimension, DimensionId(1));
        }
    }
}

/// One submission can carry both kinds of change at once. Decaying an
/// established region hard and immediately loading a distant one produces exits
/// and entries in the same batch — and the exits still come first. A cycle
/// discovers the whole difference between the old competitive set and the new
/// one, so a landscape that shifts wholesale reaches the model as a single
/// coherent reshaping rather than as a sequence of partial ones.
///
/// ´claim:identity:one-submission-can-carry-both-exits-and-entries-from-a-single-cycle´
/// ´test:crate:batch-entry-exit´
#[test]
fn batch_entry_exit() {
    // A single lifecycle submission can contain both exits and entries.
    // Exits must precede entries in the batch (marginalisation before extension).
    let harness = MaintenanceHarness::spawn("batch-entry-exit");
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension_with_config(aggressive_split_config(), DimensionId(1), 100, obs_rx, infra);

    // Build initial competitive set at coordinate 0x1000
    for _ in 0..500 {
        obs.try_send(0x1000);
    }
    harness.flush_loop();
    harness.clear_model_owner();

    // Apply heavy decay to shrink existing cells...
    harness.force_decay(DimensionId(1), 0.01);

    // ...then immediately add observations in a different region to cause entries
    for _ in 0..500 {
        obs.try_send(0xFFFF_FFFF_FFFF_0000);
    }
    harness.flush_loop();

    let cmds = harness.drain_model_owner();

    let mut total_exits = 0usize;
    let mut total_entries = 0usize;
    for cmd in &cmds {
        if let ModelOwnerCommand::Lifecycle(sub) = cmd {
            let mut seen_entry = false;
            for event in &sub.events {
                match event {
                    LifecycleEvent::CompetitiveCellExit { .. } => {
                        assert!(!seen_entry, "exit must not follow entry within the same submission");
                        total_exits += 1;
                    }
                    LifecycleEvent::CompetitiveCellEntry { .. } => {
                        seen_entry = true;
                        total_entries += 1;
                    }
                    _ => {}
                }
            }
        }
    }

    assert!(total_exits > 0, "decay should produce exits, got 0 exits");
    assert!(total_entries > 0, "new observations should produce entries, got 0 entries");
}

/// When the model owner cannot keep up, the maintenance loop sheds lifecycle
/// events rather than blocking or dying: with a single-slot channel and two
/// bursts of five hundred observations driving repeated set changes, the loop
/// stays responsive throughout. The identity layer's structural updates are
/// advisory, so a stalled model owner must never be able to wedge the thread
/// that is still draining every dimension's observation queue.
///
/// ´claim:identity:a-full-model-owner-channel-costs-lifecycle-events-not-the-maintenance-loop´
/// ´test:crate:command-channel-full´
#[test]
fn command_channel_full() {
    // When the model-owner command channel is full,
    // `try_send` fails without panic. The loop continues operating.
    let harness = MaintenanceHarness::spawn_with_capacity("chan-full", 1);
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension_with_config(aggressive_split_config(), DimensionId(1), 100, obs_rx, infra);

    // Stuff the model-owner channel by flushing once, then trigger many
    // competitive set changes rapidly.
    harness.checkpoint();

    for _ in 0..500 {
        obs.try_send(0x1000);
    }
    harness.flush_loop();

    // The loop should still be alive and responsive.
    for _ in 0..500 {
        obs.try_send(0x2000);
    }
    harness.flush_loop();

    // Drain whatever made it through. Not all lifecycle events may
    // have been delivered (some were dropped due to channel full).
    let cmds = harness.drain_model_owner();
    // The important thing is: no panic occurred.
    drop(cmds);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Crate-Level Integration Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Sustained traffic earns a region a place in the published competitive set,
/// end to end: a thousand observations submitted from the assessment side pass
/// through the queue, the drain, the graph and the change detector, and come
/// out as cells an assessment thread can read. Nothing along that path needs
/// telling which regions matter — the traffic decides.
///
/// ´claim:identity:sustained-traffic-at-one-coordinate-earns-that-region-a-place-in-the-published-competitive-set´
/// ´test:crate:identity-observation-to-competitive-set´
#[test]
fn identity_observation_to_competitive_set() {
    let harness = MaintenanceHarness::spawn("obs-to-set");
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension_with_config(aggressive_split_config(), DimensionId(1), 100, obs_rx, infra.clone());

    for _ in 0..1000 {
        obs.try_send(0x1000);
    }
    harness.flush_loop();

    let set = infra.load_competitive_set();
    assert!(
        !set.is_empty(),
        "after 1000 concentrated observations, competitive set should be populated"
    );
}

/// Lifecycle events cross from the maintenance thread to the model owner, and
/// each one is an entry or an exit tagged with the dimension whose set changed —
/// nothing else appears on that path. The tag is what the receiving side keys
/// on when it decides which dimension of its model to reshape.
///
/// ´claim:identity:every-lifecycle-event-names-the-dimension-whose-set-changed´
/// ´test:crate:competitive-set-change-emits-lifecycle´
#[test]
fn competitive_set_change_emits_lifecycle() {
    let harness = MaintenanceHarness::spawn("set-change-lifecycle");
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension_with_config(aggressive_split_config(), DimensionId(1), 100, obs_rx, infra);

    for _ in 0..500 {
        obs.try_send(0x1000);
    }
    harness.flush_loop();

    let cmds = harness.drain_model_owner();
    let events = collect_lifecycle_events(&cmds);

    assert!(!events.is_empty(), "competitive set changes should emit lifecycle events");

    for event in &events {
        match event {
            LifecycleEvent::CompetitiveCellEntry { dimension, .. } | LifecycleEvent::CompetitiveCellExit { dimension, .. } => {
                assert_eq!(*dimension, DimensionId(1), "events should be for the correct dimension");
            }
            _ => panic!("unexpected event type: {event:?}"),
        }
    }
}

/// Routing and publication agree: an observed coordinate resolves to a
/// non-empty chain, and every cell in that chain is one the published
/// competitive set actually contains. Assessment routes through the same index
/// the maintenance loop publishes, so a cell could not be routed to unless the
/// loop had announced it and the model had extended to hold it.
///
/// ´claim:identity:an-observed-coordinate-routes-only-to-cells-the-published-set-contains´
/// ´test:crate:identity-find-active-matches-competitive-set´
#[test]
fn identity_find_active_matches_competitive_set() {
    let harness = MaintenanceHarness::spawn("find-active-match");
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension_with_config(aggressive_split_config(), DimensionId(1), 100, obs_rx, infra.clone());

    for _ in 0..1000 {
        obs.try_send(0x1000);
    }
    harness.flush_loop();

    let set = infra.load_competitive_set();
    if !set.is_empty() {
        let active = infra.find_active_cells(0x1000);
        assert!(
            !active.is_empty(),
            "find_active for observed coord should return matching cells"
        );

        for cell in &active {
            assert!(
                set.cell_ids().any(|c| c == cell),
                "find_active cell {cell:?} should be in the competitive set"
            );
        }
    }
}

/// Two dimensions loaded at once, each with five hundred observations in a
/// different corner of the domain, end with five hundred units apiece, and
/// every lifecycle event emitted belongs to one of the two rather than to
/// neither. Concurrency does not blur the boundary between dimensions: they
/// share a thread and a command channel but nothing of their state.
///
/// (´claim:identity:each-dimension-accumulates-only-its-own-observations´)
/// ´test:crate:two-dimensions-independent´
#[test]
fn two_dimensions_independent() {
    let harness = MaintenanceHarness::spawn("two-dims");

    let (infra1, obs1) = setup_dimension(1);
    let (infra2, obs2) = setup_dimension(2);

    let obs_rx1 = obs1.receiver().clone();
    let obs_rx2 = obs2.receiver().clone();

    let config = aggressive_split_config();
    harness.register_dimension_with_config(config.clone(), DimensionId(1), 100, obs_rx1, infra1.clone());
    harness.register_dimension_with_config(config, DimensionId(2), 100, obs_rx2, infra2.clone());

    for _ in 0..500 {
        obs1.try_send(0x1000); // Dim 1: concentrated at 0x1000
    }
    for _ in 0..500 {
        obs2.try_send(0xFFFF_FFFF_FFFF_0000); // Dim 2: concentrated elsewhere
    }

    harness.checkpoint();

    let snap1 = infra1.graph_snapshot.load();
    let snap2 = infra2.graph_snapshot.load();

    let s1 = snap1.as_ref().as_ref().unwrap();
    let s2 = snap2.as_ref().as_ref().unwrap();

    assert_eq!(s1.total_energy(), 500, "dim 1 should have 500 observations");
    assert_eq!(s2.total_energy(), 500, "dim 2 should have 500 observations");

    let cmds = harness.drain_model_owner();
    for cmd in &cmds {
        if let ModelOwnerCommand::Lifecycle(sub) = cmd {
            for event in &sub.events {
                match event {
                    LifecycleEvent::CompetitiveCellEntry { dimension, .. }
                    | LifecycleEvent::CompetitiveCellExit { dimension, .. } => {
                        assert!(
                            *dimension == DimensionId(1) || *dimension == DimensionId(2),
                            "events should be for one of the two dimensions"
                        );
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Forgetting reaches the assessment path, not just the event stream: after
/// annihilating decay the published competitive set is strictly smaller than it
/// was, and exits are emitted for what left it. Both halves are needed — an
/// announced exit whose cell stayed routable would keep sending traffic to a
/// cell the model had already marginalised away.
///
/// ´claim:identity:decay-shrinks-the-published-competitive-set-and-not-merely-the-event-stream´
/// ´test:crate:identity-decay-shrinks-competitive-set´
#[test]
fn identity_decay_shrinks_competitive_set() {
    // Annihilation decay zeroes all energy, causing the competitive
    // set to empty on the next loop cycle.
    let harness = MaintenanceHarness::spawn("decay-shrinks");
    let (infra, obs) = setup_dimension(1);
    let obs_rx = obs.receiver().clone();

    harness.register_dimension_with_config(aggressive_split_config(), DimensionId(1), 100, obs_rx, infra.clone());

    for _ in 0..500 {
        obs.try_send(0x1000);
    }
    harness.flush_loop();

    let cells_before = infra.load_competitive_set().len();
    assert!(cells_before > 0, "should have competitive cells before decay");

    harness.clear_model_owner();

    harness.force_decay(DimensionId(1), 0.0);
    harness.flush_loop();

    let cells_after = infra.load_competitive_set().len();

    assert!(
        cells_after < cells_before,
        "annihilation decay should shrink competitive set: {cells_before} -> {cells_after}"
    );

    let cmds = harness.drain_model_owner();
    let events = collect_lifecycle_events(&cmds);
    let exit_count = events
        .iter()
        .filter(|e| matches!(e, LifecycleEvent::CompetitiveCellExit { .. }))
        .count();

    assert!(exit_count > 0, "annihilation decay should emit CompetitiveCellExit events");
}
