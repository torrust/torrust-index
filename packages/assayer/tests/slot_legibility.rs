// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`association_separates_a_keyed_sentinel_from_a_hash_fed_one`] | wellness | Coordinates drawn from a cryptographic hash carry no exploitable structure, and the system says so. Driven side by side against a Sentinel routed by the outcome itself, on one labelled stream and one publication, the hash-fed Sentinel's slot reads beside the correlation a coordinate telling the outcome nothing earns — inside the alert band — while the keyed Sentinel's stands clear of that band by a multiple. A measurement surface contributing nothing has to be legible as such, or an operator reading a healthy-looking report cannot tell a working Sentinel from a noise generator. The weight-mass reading taken off the same publication is not that legibility and the test reports it alongside: it gives the hash-fed slot a figure of the same order as the keyed one's, because weight settles at noise scale on coordinates that predict nothing and a converged model has no residual left to shrink it with (´def:monitoring:encoding-effectiveness´). |
//! | [`contribution_separates_a_sentinel_the_score_leans_on_from_a_hash_fed_one`] | wellness | Removing the hash-fed Sentinel's slot from the score costs a small fraction of what removing the keyed one's costs, each measured as the loss the removal would actually incur. The reading is the removal cost itself rather than a proxy for it, which is what makes it the one a retirement decision can be taken on: the number an operator reads is the number the deregistration would produce. |
//! | [`per_dimension_readings_survive_a_layout_rebuild`] | wellness | A registered identity dimension publishes both readings over its own block, and goes on publishing them after a later registration has rebuilt the layout underneath it. The producers accumulate at label time rather than over a frozen window of stored requests, so a rebuild costs a block the evidence it had gathered and never its ability to gather more — which is the whole reason the readings are taken where the label is. A frozen-window reading of the same quantities collapses to a handful of observations under exactly this event. |

//! The two legibility readings, driven end to end through the public surface
//! (´def:monitoring:slot-association´), (´def:monitoring:slot-contribution´).
//!
//! Each test drives one labelled stream through Sentinels whose coordinate
//! streams differ in exactly one thing — whether the region a request lands in
//! is named by the outcome or by an avalanche hash of the request index — and
//! reads the published health report. Nothing here reaches inside the package:
//! the readings come off `full_health_report` exactly as a host would take
//! them.
//!
//! # Cross-References
//!
//! - (´chap:spec:health-monitoring´) — the monitoring surface these readings
//!   are published on
//! - (´def:monitoring:encoding-effectiveness´) — the weight-mass reading they
//!   stand beside, and which does not separate these streams

#![allow(
    // A measurement test's numbers are worth reading even when it passes: the
    // assertion says a separation exists and the table says how large it is.
    clippy::print_stdout,
)]

use torrust_assayer::ChannelPolicy;
use torrust_assayer::testing::{LabelSpec, World, cycle_request, golden_report, make_cell_report, scenario_with};
use torrust_assayer::types::SCORING_AXIS_COUNT;

/// One reading per scoring axis, which is the shape every score fixture takes.
type AxisReadings = [f64; SCORING_AXIS_COUNT];

/// Instance id, so the world names itself in any diagnostic it emits.
const INSTANCE: &str = "slot-legibility";

/// The world's seed. Fixed, because reproducibility is the harness contract.
const SEED: u64 = 0x0000_0005_0000_0113_u64;

/// The one declared channel every stream runs on.
const CHANNEL: &str = "default";

/// Labels driven before the per-Sentinel readings are taken.
///
/// Long enough that the decayed window has passed its evidence floor several
/// times over and the operational weights have learned something to ablate,
/// short enough that the suite is not a benchmark.
const TRAINING_CYCLES: u64 = 1_500;

/// The salt the structureless coordinate stream is mixed under.
const COORDINATE_SALT: u64 = 0x243f_6a88_85a3_08d3;

/// The salt the outcome stream is mixed under.
///
/// The outcomes are drawn from their own avalanche rather than alternated: an
/// alternating stream is predictable from its own recent history, and every
/// Sentinel's ledger carries that history, which would hand both Sentinels a
/// real predictor having nothing to do with either one's coordinates.
const CLASS_SALT: u64 = 0xb7e1_5162_8aed_2a6b;

/// The alert band's lower edge, in multiples of the null floor
/// (´def:monitoring:slot-association´).
const ASSOCIATION_ALERT_EDGE: f64 = 3.0;

/// The measurement surface one Sentinel presents: two disjoint depth-four
/// regions with contrasting readings, and a root above them.
///
/// The two Sentinels are given separate fixtures rather than one shared
/// report. A report copied between them would put literally equal columns in
/// both slots, and what the control has to hold fixed is the *shape* of the
/// surface rather than its numbers: same region count, same depth, same
/// occupancy on every request, same split of the traffic.
struct Surface {
    /// The Sentinel's harness name.
    name: &'static str,
    /// The region a request lands in when the surface is read as quiet.
    quiet_region: u128,
    /// The region a request lands in when the surface is read as loud.
    loud_region: u128,
    /// The root's peaks and accumulators.
    root: (AxisReadings, AxisReadings),
    /// The quiet region's peaks and accumulators.
    quiet: (AxisReadings, AxisReadings),
    /// The loud region's peaks and accumulators.
    loud: (AxisReadings, AxisReadings),
    /// The root's sample count. Each region carries half of it.
    samples: usize,
    /// The energy ratio the regions report.
    energy_ratio: f64,
    /// The noise influence the regions report.
    noise_influence: f64,
}

impl Surface {
    /// The coordinate a request lands on, in whichever region it is routed to.
    ///
    /// The low bits vary with `fill` so the stream is not one repeated point,
    /// and they sit far below the depth-four prefix, so they never move a
    /// request out of its region.
    fn coordinate(&self, loud: bool, fill: u64) -> u128 {
        let region = if loud { self.loud_region } else { self.quiet_region };
        region | u128::from(fill)
    }
}

/// The keyed surface: prefixes `0x3` and `0xB`, read loud when the outcome is
/// adverse.
const KEYED: Surface = Surface {
    name: "keyed",
    quiet_region: 0x3000_0000_0000_0000_0000_0000_0000_0000,
    loud_region: 0xB000_0000_0000_0000_0000_0000_0000_0000,
    root: ([0.5, 0.3, 1.0, 0.0], [0.2, 0.1, 0.4, 0.0]),
    quiet: ([0.2, 0.1, 0.2, 0.1], [0.1, 0.1, 0.1, 0.1]),
    loud: ([4.0, 3.0, 3.5, 2.5], [1.5, 1.2, 1.4, 1.0]),
    samples: 4_000,
    energy_ratio: 0.72,
    noise_influence: 0.15,
};

/// The hash-fed surface: prefixes `0x5` and `0xD`, read loud when an avalanche
/// hash of the request index says so.
///
/// Its readings are the keyed surface's ordinary variation and not a weakened
/// version of them: a surface reporting flatter or quieter cells would have
/// less to say for reasons other than the one under test.
const HASH_FED: Surface = Surface {
    name: "hash-fed",
    quiet_region: 0x5000_0000_0000_0000_0000_0000_0000_0000,
    loud_region: 0xD000_0000_0000_0000_0000_0000_0000_0000,
    root: ([0.45, 0.35, 0.9, 0.05], [0.25, 0.15, 0.35, 0.05]),
    quiet: ([0.25, 0.15, 0.25, 0.05], [0.15, 0.05, 0.15, 0.05]),
    loud: ([3.6, 2.7, 3.2, 2.2], [1.4, 1.1, 1.3, 0.9]),
    samples: 3_800,
    energy_ratio: 0.69,
    noise_influence: 0.18,
};

/// What one driven stream leaves on the health surface for the two Sentinels.
struct SentinelPair {
    /// The keyed Sentinel's association, contribution and weight mass.
    keyed: (Option<f64>, Option<f64>, Option<f64>),
    /// The hash-fed Sentinel's three readings, in the same order.
    hash_fed: (Option<f64>, Option<f64>, Option<f64>),
}

/// Drives the two-Sentinel stream and reads both slots off the health report.
fn drive_two_sentinels() -> SentinelPair {
    let mut scenario = scenario_with(INSTANCE, SEED, |builder| builder.channel(CHANNEL, ChannelPolicy::default()))
        .expect("the two-Sentinel world should build");

    let keyed = scenario.register_sentinel(KEYED.name).expect("register the keyed Sentinel");
    let hash_fed = scenario
        .register_sentinel(HASH_FED.name)
        .expect("register the hash-fed Sentinel");
    scenario
        .flush_labels()
        .expect("publish both registrations before report ingestion");
    ingest_surface(&scenario, &KEYED);
    ingest_surface(&scenario, &HASH_FED);

    for index in 0..TRAINING_CYCLES {
        let adverse = mixed_bit(index, CLASS_SALT);
        let request = scenario
            .request(CHANNEL, &format!("obs-{index}"))
            .with_sentinel(keyed, KEYED.coordinate(adverse, index))
            .with_sentinel(hash_fed, HASH_FED.coordinate(mixed_bit(index, COORDINATE_SALT), index));
        cycle_request(&scenario, request, |assessment_id| {
            if adverse {
                LabelSpec::adverse(assessment_id)
            } else {
                LabelSpec::benign(assessment_id)
            }
        });
    }

    let report = scenario.assayer().full_health_report();
    let read = |id| {
        let health = &report.sentinels[&id];
        (health.association, health.contribution, health.informativeness)
    };
    SentinelPair {
        keyed: read(keyed),
        hash_fed: read(hash_fed),
    }
}

/// Coordinates drawn from a cryptographic hash carry no exploitable structure,
/// and the system says so. Driven side by side against a Sentinel routed by the
/// outcome itself, on one labelled stream and one publication, the hash-fed
/// Sentinel's slot reads beside the correlation a coordinate telling the
/// outcome nothing earns — inside the alert band — while the keyed Sentinel's
/// stands clear of that band by a multiple.
///
/// A measurement surface contributing nothing has to be legible as such, or an
/// operator reading a healthy-looking report cannot tell a working Sentinel
/// from a noise generator. The weight-mass reading taken off the same
/// publication is not that legibility and the test reports it alongside: it
/// gives the hash-fed slot a figure of the same order as the keyed one's,
/// because weight settles at noise scale on coordinates that predict nothing
/// and a converged model has no residual left to shrink it with
/// (´def:monitoring:encoding-effectiveness´).
///
/// ´claim:wellness:structureless-coordinates-drive-encoding-effectiveness-to-nothing´
/// ´test:integration:association-separates-a-keyed-sentinel-from-a-hash-fed-one´
#[test]
fn association_separates_a_keyed_sentinel_from_a_hash_fed_one() {
    let pair = drive_two_sentinels();

    let keyed = pair.keyed.0.expect("the keyed slot has carried a window of labels");
    let hash_fed = pair.hash_fed.0.expect("the hash-fed slot has carried a window of labels");
    println!(
        "association: keyed={keyed:.4} hash_fed={hash_fed:.4} | weight mass: keyed={:?} hash_fed={:?}",
        pair.keyed.2, pair.hash_fed.2
    );

    assert!(
        hash_fed < ASSOCIATION_ALERT_EDGE,
        "the hash-fed slot sits inside the alert band: {hash_fed}"
    );
    assert!(
        keyed > ASSOCIATION_ALERT_EDGE,
        "the keyed slot stands clear of the alert band: {keyed}"
    );
    assert!(
        keyed > hash_fed * ASSOCIATION_ALERT_EDGE,
        "the separation is a multiple and not a margin: {keyed} against {hash_fed}"
    );
}

/// Removing the hash-fed Sentinel's slot from the score costs a small fraction
/// of what removing the keyed one's costs, each measured as the loss the
/// removal would actually incur. The reading is the removal cost itself rather
/// than a proxy for it, which is what makes it the one a retirement decision
/// can be taken on: the number an operator reads is the number the
/// deregistration would produce.
///
/// ´claim:wellness:the-contribution-reading-separates-a-slot-the-score-leans-on-from-a-hash-fed-one´
/// ´test:integration:contribution-separates-a-sentinel-the-score-leans-on-from-a-hash-fed-one´
#[test]
fn contribution_separates_a_sentinel_the_score_leans_on_from_a_hash_fed_one() {
    let pair = drive_two_sentinels();

    let keyed = pair.keyed.1.expect("the keyed slot has carried a window of labels");
    let hash_fed = pair.hash_fed.1.expect("the hash-fed slot has carried a window of labels");
    println!("contribution: keyed={keyed:.6} hash_fed={hash_fed:.6}");

    assert!(keyed > 0.0, "removing the keyed slot costs the score loss: {keyed}");
    assert!(
        keyed > hash_fed * 3.0,
        "removing the hash-fed slot costs a fraction of what removing the keyed one costs: {keyed} against {hash_fed}"
    );
}

/// The identity dimension's harness name.
const DIMENSION: &str = "account";

/// The dimension registered after the stream is under way, whose registration
/// is what rebuilds the layout.
const LATE_DIMENSION: &str = "tenant";

/// The entity pool the identity stream draws from.
const ENTITY_POOL: u64 = 8;

/// Labels driven before and again after the rebuild.
const DIMENSION_CYCLES: u64 = 700;

/// A registered identity dimension publishes both readings over its own block,
/// and goes on publishing them after a later registration has rebuilt the
/// layout underneath it. The producers accumulate at label time rather than
/// over a frozen window of stored requests, so a rebuild costs a block the
/// evidence it had gathered and never its ability to gather more — which is
/// the whole reason the readings are taken where the label is. A frozen-window
/// reading of the same quantities collapses to a handful of observations under
/// exactly this event.
///
/// ´claim:wellness:a-dimensions-readings-survive-the-layout-rebuild-a-later-registration-causes´
/// ´test:integration:per-dimension-readings-survive-a-layout-rebuild´
#[test]
fn per_dimension_readings_survive_a_layout_rebuild() {
    let mut scenario = scenario_with(INSTANCE, SEED, |builder| builder.channel(CHANNEL, ChannelPolicy::default()))
        .expect("the identity world should build");
    let companion = scenario
        .register_sentinel(KEYED.name)
        .expect("register the companion Sentinel");
    scenario.flush_labels().expect("publish the registration");
    ingest_surface(&scenario, &KEYED);
    let dimension = scenario.register_identity(DIMENSION).expect("register the dimension");
    scenario.flush_labels().expect("publish the dimension registration");

    drive_identity_stream(&scenario, companion, 0, DIMENSION_CYCLES);
    let before = scenario.assayer().full_health_report().identity_dimensions[&dimension].association;

    // The rebuild: a second dimension's registration extends every
    // full-dimension model and rewrites the layout the first dimension's block
    // is addressed in.
    let late = scenario
        .register_identity(LATE_DIMENSION)
        .expect("register the late dimension");
    scenario.flush_labels().expect("publish the late registration");

    drive_identity_stream(&scenario, companion, DIMENSION_CYCLES, DIMENSION_CYCLES);
    let report = scenario.assayer().full_health_report();
    let after = &report.identity_dimensions[&dimension];
    let late_health = &report.identity_dimensions[&late];
    println!(
        "dimension readings: before={before:?} after={:?}/{:?} late={:?}/{:?}",
        after.association, after.contribution, late_health.association, late_health.contribution
    );

    assert!(before.is_some(), "the dimension read before the rebuild: {before:?}");
    assert!(
        after.association.is_some(),
        "the dimension reads again after the rebuild: {:?}",
        after.association
    );
    assert!(
        after.contribution.is_some(),
        "the contribution reading survives the rebuild too: {:?}",
        after.contribution
    );
    assert!(
        late_health.association.is_some(),
        "the dimension registered into the rebuilt layout accumulates from its own start: {:?}",
        late_health.association
    );
}

/// Drives one stretch of the identity stream, with the entity a request
/// arrives under drawn from the half of the pool its outcome names.
fn drive_identity_stream(world: &World, companion: torrust_assayer::types::SentinelId, from: u64, cycles: u64) {
    for index in from..from + cycles {
        let adverse = mixed_bit(index, CLASS_SALT);
        let slot = u64::from(adverse) * ENTITY_POOL + index % ENTITY_POOL;
        let request = world
            .request(CHANNEL, &format!("acct-{slot}"))
            .with_sentinel(companion, KEYED.coordinate(adverse, index));
        cycle_request(world, request, |assessment_id| {
            if adverse {
                LabelSpec::adverse(assessment_id)
            } else {
                LabelSpec::benign(assessment_id)
            }
        });
    }
}

/// Ingests one surface's report for the Sentinel already registered under its
/// name.
#[track_caller]
fn ingest_surface(world: &World, surface: &Surface) {
    let name = surface.name;
    let half = surface.samples / 2;
    let mut report = golden_report();
    report.cell_reports = vec![
        make_cell_report(0, 0, surface.samples, 4, 8, 0.85, 0.05, surface.root.0, surface.root.1, true),
        make_cell_report(
            surface.quiet_region,
            4,
            half,
            3,
            8,
            surface.energy_ratio,
            surface.noise_influence,
            surface.quiet.0,
            surface.quiet.1,
            true,
        ),
        make_cell_report(
            surface.loud_region,
            4,
            half,
            3,
            8,
            surface.energy_ratio,
            surface.noise_influence,
            surface.loud.0,
            surface.loud.1,
            true,
        ),
    ];
    report.ancestor_reports = Vec::new();
    report.contour.cell_count = 3;
    report.analysis_set_summary.competitive_size = 3;
    report.analysis_set_summary.full_size = 3;
    report.analysis_set_summary.investment_set_size = 3;
    report.analysis_set_summary.depth_range = (0, 4);

    let ack = world
        .receive_report(name, report)
        .unwrap_or_else(|error| panic!("{name} report should be accepted: {error:?}"));
    assert_eq!(ack.cells_in_report, 3, "{name}: the report carries three cells");
    assert_eq!(ack.degraded_cells, 0, "{name}: no cell should degrade");
}

/// One bit of an avalanche over the request index under the given salt.
///
/// Stands in for the cryptographic hash the claim speaks of: what the test
/// needs of it is that the index tells nothing about the bit, that two salts
/// give two streams telling nothing about each other, and that the same index
/// yields the same bit on every run.
const fn mixed_bit(index: u64, salt: u64) -> bool {
    mixed_word(index, salt) >> 40 & 1 == 1
}

/// The avalanche of one request index under one salt.
const fn mixed_word(index: u64, salt: u64) -> u64 {
    avalanche(avalanche(index).wrapping_add(salt))
}

/// A fixed-point avalanche over a 64-bit word.
const fn avalanche(mut word: u64) -> u64 {
    word ^= word >> 30;
    word = word.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    word ^= word >> 27;
    word = word.wrapping_mul(0x94d0_49bb_1331_11eb);
    word ^= word >> 31;
    word
}
