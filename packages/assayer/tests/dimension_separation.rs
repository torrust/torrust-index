// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`the_association_reading_separates_a_keyed_dimension_from_a_null_one`] | wellness | Two identity dimensions are registered on one engine and read off one publication. Each request arrives under an entity naming two coordinates — one drawn from the half of the pool the outcome names, one drawn independently of it — and each dimension reads one of them, so the two blocks differ in exactly one thing and are scored by one model against one loss. The keyed block's association stands clear of the alert band at every point of a four-point schedule while the null block's sits beside the floor a coordinate telling the outcome nothing earns, and the keyed reading goes on climbing across a third dimension's registration rebuilding the layout underneath both — the event that made the frozen-window bench unable to ask the question at all. That is the per-dimension separation the definition left open, established on the shipped prequential producers rather than on a bench. The contribution and weight-mass readings are taken at the same points and printed beside it, and neither is asserted on. Both are functions of the operational weights, and this harness does not hold the weights' scale fixed between runs: the competitive geometry that feeds them forms on the maintenance loop's own wall-clock cadence, so how much of it exists when the model learns differs with the load on the machine. Across repeated runs of this file the two blocks' contribution figures crossed each other in both directions and their ratio spanned two orders of magnitude, and the weight-mass figures did the same over a narrower range. The association reading is the one that survives that variation, and it survives it because it is model-free: it is a correlation calibrated against a floor that depends on the window and on nothing else (´def:monitoring:slot-association´), so it is invariant to the scale of the weights the two other readings are functions of. What the schedule establishes and what it declines to establish are therefore the screening reading and the confirming one respectively, which is the division of labour the host duty already assigns them (´req:keyspace:host-duties´). The schedule begins from a settled competitive geometry rather than from a fixed label count, because that geometry forms on the same wall-clock cadence: a machine that drives labels faster than the loop splits reaches the first checkpoint with no cells at all, and two blocks with no cells read identically for want of a population rather than for want of a separation. |

//! The per-dimension association reading driven against a null dimension
//! (´def:monitoring:slot-association´), with the two readings it stands
//! beside printed at the same points
//! (´def:monitoring:slot-contribution´), (´def:monitoring:dimension-informativeness´).
//!
//! The per-Sentinel separation of these readings was established on a bench
//! whose windows were frozen, and an identity layout rebuild collapses such a
//! window to a handful of observations. This harness drives the shipped
//! producers instead, which accumulate prequentially, and puts a rebuild in
//! the middle of the schedule rather than avoiding one.
//!
//! Both dimensions register the same way, are read off the same publication,
//! and see the same requests. What differs is which coordinate of the request
//! each one reads: every entity names a keyed coordinate — the half of the
//! pool its outcome names, and a position inside that half that turns over
//! with the request index — and a null coordinate drawn the same way from a
//! half-selector the outcome stream does not share. Putting the difference in
//! the coordinate rather than in the entity is what lets everything else the
//! entity feeds cancel exactly.
//!
//! The encoding places a coordinate's index in the domain's top four bits.
//! The competitive mechanism partitions the domain by dyadic prefix, so where
//! in the coordinate an entity's identity sits decides at what depth the
//! mechanism can tell two entities apart; an encoding placing identity in the
//! low bits of a wide domain gives every entity the same prefix at every
//! depth the cutoff allows, and the block then reports one undivided
//! population however the traffic is drawn.
//!
//! # Cross-References
//!
//! - (´chap:spec:health-monitoring´) — the monitoring surface these readings
//!   are published on
//! - (´def:monitoring:dimension-informativeness´) — the weight-mass reading
//!   they stand beside, printed here and asserted on nowhere
//! - (´req:keyspace:host-duties´) — the duty that reads the association
//!   figure this file establishes

#![allow(
    // A measurement test's numbers are worth reading even when it passes: the
    // assertion says a separation exists and the table says how large it is.
    clippy::print_stdout,
)]

use std::collections::BTreeMap;

use torrust_assayer::testing::{LabelSpec, Scenario, World, cycle_request, fail_fast_on_model_owner_panic, scenario_with};
use torrust_assayer::types::{DimensionId, EntityKey, IdentityBudget, OutcomeAxisId};
use torrust_assayer::{ChannelPolicy, IdentityDimensionRegistration};

/// Instance id, so a world names itself in any diagnostic it emits.
const INSTANCE: &str = "dimension-separation";

/// The world's seed. Fixed, because reproducibility is the harness contract.
const SEED: u64 = 0x0000_0005_0000_0113_u64;

/// The one declared channel every stream runs on.
const CHANNEL: &str = "default";

/// The dimension reading the coordinate the outcome names.
const KEYED_NAME: &str = "account";

/// The dimension reading the coordinate drawn independently of the outcome.
const NULL_NAME: &str = "shadow";

/// The dimension registered mid-schedule, whose registration is the rebuild.
const LATE_NAME: &str = "tenant";

/// The keyed dimension's identifier.
///
/// The harness allocates its own identifiers because the registrations are
/// made through the public surface rather than through
/// `World::register_identity`, whose encoding this measurement cannot use.
const KEYED_DIM: DimensionId = DimensionId(1);

/// The null dimension's identifier.
const NULL_DIM: DimensionId = DimensionId(2);

/// The late dimension's identifier.
const LATE_DIM: DimensionId = DimensionId(3);

/// The outcome axis whose per-cell history the dimension block carries.
///
/// Without a registered axis the block is its eight structural and
/// measurement positions and nothing else, and none of those is written from
/// an outcome (´tab:keyspace:dimension-features´). A dimension asked to carry
/// the outcome with no axis registered is a block asked to report a quantity
/// its layout gives it no position for.
const AXIS: &str = "magnitude";

/// Coordinates per outcome half. The pool is twice this.
const POOL_HALF: u64 = 8;

/// Labels driven between two readings.
const LEG: u64 = 250;

/// Labels driven per round of the settling loop that precedes the schedule.
const SETTLE_BURST: u64 = 100;

/// Rounds the settling loop will run before giving up.
const SETTLE_ROUNDS: u32 = 14;

/// The dyadic depth both competitive sets must reach before the schedule
/// starts.
///
/// The competitive geometry forms on the maintenance loop's own wall-clock
/// cadence, so how much of it exists at a given label count depends on how
/// fast the host drove the labels. A schedule that started at a fixed label
/// count would read a formed geometry on a loaded machine and an empty one on
/// an idle machine, and the two would not be the same measurement. Waiting
/// for a stated structural state instead makes the first reading mean the
/// same thing on both. Sixteen coordinates separate at depth four; five is
/// that plus the level which says the tree has gone past them.
const SETTLE_DEPTH: u8 = 5;

/// How many legs the schedule runs.
const LEGS: usize = 4;

/// The leg after which the late dimension is registered.
///
/// Two legs of evidence stand behind the reading the rebuild lands on, and
/// two legs follow it, so the schedule reads both dimensions on each side of
/// the event rather than only after it.
const REBUILD_AFTER_LEG: usize = 2;

/// The salt the outcome stream is mixed under.
///
/// The outcomes are drawn from their own avalanche rather than alternated: an
/// alternating stream is predictable from its own recent history, which would
/// hand both blocks a real predictor having nothing to do with either one's
/// coordinate.
const CLASS_SALT: u64 = 0xb7e1_5162_8aed_2a6b;

/// The salt the null coordinate's half-selector is mixed under.
const NULL_SALT: u64 = 0x243f_6a88_85a3_08d3;

/// The alert band's lower edge, in multiples of the null floor
/// (´def:monitoring:slot-association´).
const ASSOCIATION_ALERT_EDGE: f64 = 3.0;

/// The multiple a separation has to clear to be a separation and not a margin.
const SEPARATION_MULTIPLE: f64 = 3.0;

/// How far the two blocks' coverage fractions may stand apart and still be
/// one figure. They are equal by construction; the tolerance is against the
/// rounding of a quotient of counts and nothing else.
const COVERAGE_TOLERANCE: f64 = 0.05;

/// The two-decimal field at `offset` of an entity name, read as a coordinate
/// in the domain's top four bits.
///
/// Reads rather than parses: a byte the name does not carry is read as zero,
/// so a malformed name yields a coordinate instead of a panic on the label
/// path.
fn field_at(bytes: &[u8], offset: usize) -> u128 {
    let tens = bytes.get(offset).copied().unwrap_or(b'0').wrapping_sub(b'0');
    let ones = bytes.get(offset + 1).copied().unwrap_or(b'0').wrapping_sub(b'0');
    ((u128::from(tens) * 10 + u128::from(ones)) & 0xF) << 124
}

/// The keyed dimension's encoding: the entity's first field.
fn keyed_encoder(entity: &EntityKey) -> u128 {
    field_at(entity.as_bytes(), 1)
}

/// The null dimension's encoding: the entity's second field.
fn null_encoder(entity: &EntityKey) -> u128 {
    field_at(entity.as_bytes(), 4)
}

/// The late dimension's encoding: every entity lands on one coordinate.
///
/// Its registration is what rebuilds the layout, and that is the whole of its
/// job here. A late dimension encoding either of the two live coordinates
/// would carry information one of the blocks under measurement already
/// carries, and the contribution reading is a removal cost: two blocks that
/// duplicate each other each cost nothing to remove alone
/// (´def:monitoring:slot-contribution´). Giving the rebuild a structureless
/// dimension keeps the event a rebuild rather than a second experiment.
const fn flat_encoder(_: &EntityKey) -> u128 {
    0
}

/// The entity one request arrives under, naming both coordinates.
///
/// Six bytes: `k` and the keyed coordinate's two decimal digits, then `n` and
/// the null coordinate's. The two encoders read the two fields at their fixed
/// offsets.
fn entity_name(index: u64, adverse: bool) -> String {
    let keyed = (if adverse { POOL_HALF } else { 0 }) + index % POOL_HALF;
    let null = (if mixed_bit(index, NULL_SALT) { POOL_HALF } else { 0 }) + index % POOL_HALF;
    format!("k{keyed:02}n{null:02}")
}

/// What one dimension's block reads on the published health report.
struct Reading {
    /// The block's association with the outcome, in multiples of the floor.
    association: Option<f64>,
    /// What removing the block from the score would cost.
    contribution: Option<f64>,
    /// The weight mass over the block, the harness's expected-negative control.
    weight_mass: Option<f64>,
    /// The weight mass over the dimension's competitive indicators.
    cell_weight_mass: Option<f64>,
    /// The share of assessed traffic that fell in a competitive cell.
    coverage: Option<f64>,
    /// The competitive cells the dimension holds, by dyadic depth.
    depths: BTreeMap<u8, usize>,
}

/// Reads one dimension's block off the published health report.
fn read_dimension(world: &World, dimension: DimensionId) -> Reading {
    let report = world.assayer().full_health_report();
    let health = &report.identity_dimensions[&dimension];
    Reading {
        association: health.association,
        contribution: health.contribution,
        weight_mass: health.informativeness,
        cell_weight_mass: health.cell_weight_mass,
        coverage: health.competitive_coverage_fraction,
        depths: health.competitive_cell_depth_distribution.clone(),
    }
}

/// Registers one identity dimension through the public surface.
#[track_caller]
fn register_dimension(world: &World, id: DimensionId, name: &str, encode: fn(&EntityKey) -> u128) {
    world
        .assayer()
        .register_identity_dimension(IdentityDimensionRegistration {
            id,
            name: name.to_owned(),
            description: "two-block separation harness".to_owned(),
            coordinate_semantics: "the leading nibble names the coordinate".to_owned(),
            domain_bits: 128,
            depth_cutoff: 10,
            budget: IdentityBudget::for_depth_cutoff(10),
            encode,
        })
        .unwrap_or_else(|error| panic!("register {name}: {error:?}"));
    world.flush_labels().expect("publish the registration");
}

/// Builds one world with a channel and the outcome axis, and no dimension yet.
fn build_world() -> (Scenario, OutcomeAxisId) {
    let mut scenario = scenario_with(INSTANCE, SEED, |builder| builder.channel(CHANNEL, ChannelPolicy::default()))
        .expect("the world should build");
    let axis = scenario.register_axis(AXIS, false).expect("register the outcome axis");
    (scenario, axis)
}

/// Drives one stretch of one world's stream.
fn drive(world: &World, axis: OutcomeAxisId, from: u64, cycles: u64) {
    for index in from..from + cycles {
        let adverse = mixed_bit(index, CLASS_SALT);
        let request = world.request(CHANNEL, &entity_name(index, adverse));
        cycle_request(world, request, |assessment_id| {
            let spec = if adverse {
                LabelSpec::adverse(assessment_id)
            } else {
                LabelSpec::benign(assessment_id)
            };
            spec.outcome(axis, if adverse { 1.0 } else { -1.0 })
        });
    }
}

/// The deepest dyadic depth a competitive set has reached.
fn deepest(depths: &BTreeMap<u8, usize>) -> u8 {
    depths.keys().next_back().copied().unwrap_or(0)
}

/// Drives the stream until both competitive sets have formed, and answers
/// with the index the measured schedule starts at.
///
/// # Panics
///
/// Panics if the geometry has not formed within the settling budget. A world
/// whose competitive sets never form is a world in which neither block has a
/// population to be keyed or null over, and reading the schedule against it
/// would report the absence of geometry as the absence of a separation.
fn settle_geometry(world: &World, axis: OutcomeAxisId) -> u64 {
    let mut index = 0;
    for _ in 0..SETTLE_ROUNDS {
        drive(world, axis, index, SETTLE_BURST);
        index += SETTLE_BURST;
        let keyed = deepest(&read_dimension(world, KEYED_DIM).depths);
        let null = deepest(&read_dimension(world, NULL_DIM).depths);
        if keyed >= SETTLE_DEPTH && null >= SETTLE_DEPTH {
            println!("geometry settled after {index} labels at depths keyed={keyed} null={null}");
            return index;
        }
    }
    panic!("the competitive geometry did not reach depth {SETTLE_DEPTH} within {index} labels");
}

/// Renders one optional reading for the table.
fn show(value: Option<f64>) -> String {
    value.map_or_else(|| "absent".to_owned(), |v| format!("{v:.4}"))
}

/// Prints one schedule point's two blocks side by side.
fn print_point(labels: u64, rebuilt: bool, keyed: &Reading, null: &Reading) {
    let rows: [(&str, String, String); 5] = [
        ("association ", show(keyed.association), show(null.association)),
        ("contribution", show(keyed.contribution), show(null.contribution)),
        ("weight mass ", show(keyed.weight_mass), show(null.weight_mass)),
        ("cell mass   ", show(keyed.cell_weight_mass), show(null.cell_weight_mass)),
        ("coverage    ", show(keyed.coverage), show(null.coverage)),
    ];
    for (name, left, right) in rows {
        println!("{labels:6}  {rebuilt:7}  {name}  {left:>10}  {right:>10}");
    }
    println!(
        "{labels:6}  {rebuilt:7}  cells         keyed={:?} null={:?}",
        keyed.depths, null.depths
    );
}

/// Two identity dimensions are registered on one engine and read off one
/// publication. Each request arrives under an entity naming two coordinates —
/// one drawn from the half of the pool the outcome names, one drawn
/// independently of it — and each dimension reads one of them, so the two
/// blocks differ in exactly one thing and are scored by one model against one
/// loss.
///
/// The keyed block's association stands clear of the alert band at every
/// point of a four-point schedule while the null block's sits beside the
/// floor a coordinate telling the outcome nothing earns, and the keyed
/// reading goes on climbing across a third dimension's registration
/// rebuilding the layout underneath both — the event that made the
/// frozen-window bench unable to ask the question at all. That is the
/// per-dimension separation the definition left open, established on the
/// shipped prequential producers rather than on a bench.
///
/// The contribution and weight-mass readings are taken at the same points and
/// printed beside it, and neither is asserted on. Both are functions of the
/// operational weights, and this harness does not hold the weights' scale
/// fixed between runs: the competitive geometry that feeds them forms on the
/// maintenance loop's own wall-clock cadence, so how much of it exists when
/// the model learns differs with the load on the machine. Across repeated
/// runs of this file the two blocks' contribution figures crossed each other
/// in both directions and their ratio spanned two orders of magnitude, and
/// the weight-mass figures did the same over a narrower range. The
/// association reading is the one that survives that variation, and it
/// survives it because it is model-free: it is a correlation calibrated
/// against a floor that depends on the window and on nothing else
/// (´def:monitoring:slot-association´), so it is invariant to the scale of
/// the weights the two other readings are functions of. What the schedule
/// establishes and what it declines to establish are therefore the screening
/// reading and the confirming one respectively, which is the division of
/// labour the host duty already assigns them (´req:keyspace:host-duties´).
///
/// The schedule begins from a settled competitive geometry rather than from a
/// fixed label count, because that geometry forms on the same wall-clock
/// cadence: a machine that drives labels faster than the loop splits reaches
/// the first checkpoint with no cells at all, and two blocks with no cells
/// read identically for want of a population rather than for want of a
/// separation.
///
/// ´claim:wellness:the-per-dimension-association-reading-separates-a-keyed-dimension-from-a-null-one´
/// ´test:integration:the-association-reading-separates-a-keyed-dimension-from-a-null-one´
#[test]
fn the_association_reading_separates_a_keyed_dimension_from_a_null_one() {
    // Every wait this schedule takes on the label path is unbounded, so an
    // owner that stops answering would leave the run parked rather than
    // failing it. Arming the hook is what makes the owner's own message the
    // verdict.
    fail_fast_on_model_owner_panic();
    let (world, axis) = build_world();
    register_dimension(&world, KEYED_DIM, KEYED_NAME, keyed_encoder);
    register_dimension(&world, NULL_DIM, NULL_NAME, null_encoder);

    let start = settle_geometry(&world, axis);

    let mut points: Vec<(u64, bool, Reading, Reading)> = Vec::with_capacity(LEGS);
    println!("labels  rebuilt  reading            keyed        null");
    for leg in 0..LEGS {
        let from = start + leg as u64 * LEG;
        drive(&world, axis, from, LEG);
        let keyed = read_dimension(&world, KEYED_DIM);
        let null = read_dimension(&world, NULL_DIM);
        print_point(from + LEG, leg >= REBUILD_AFTER_LEG, &keyed, &null);
        points.push((from + LEG, leg >= REBUILD_AFTER_LEG, keyed, null));
        if leg + 1 == REBUILD_AFTER_LEG {
            register_dimension(&world, LATE_DIM, LATE_NAME, flat_encoder);
        }
    }

    for (labels, _, keyed, null) in &points {
        let keyed_association = keyed.association.unwrap_or_else(|| panic!("keyed association at {labels}"));
        let null_association = null.association.unwrap_or_else(|| panic!("null association at {labels}"));
        assert!(
            null_association < ASSOCIATION_ALERT_EDGE,
            "the null block sits inside the alert band at {labels} labels: {null_association}"
        );
        assert!(
            keyed_association > ASSOCIATION_ALERT_EDGE,
            "the keyed block stands clear of the alert band at {labels} labels: {keyed_association}"
        );
        assert!(
            keyed_association > null_association * SEPARATION_MULTIPLE,
            "the association separation is a multiple at {labels} labels: {keyed_association} against {null_association}"
        );

        // Both readings are also required present at every point, including
        // the two after the rebuild. Absence there would say the producers
        // had lost the ability to accumulate rather than the evidence they
        // had, which is the distinction the prequential accumulation exists
        // to keep (´def:monitoring:slot-contribution´).
        assert!(
            keyed.contribution.is_some() && null.contribution.is_some(),
            "both blocks publish a contribution at {labels} labels: {:?} against {:?}",
            keyed.contribution,
            null.contribution
        );

        // The geometry control. Both dimensions partition sixteen coordinates
        // the same way and see the same requests, so the share of traffic
        // falling in a competitive cell is one figure and not two. A harness
        // whose two blocks had drifted apart in coverage would be comparing
        // two populations as well as two keyings, and the separation would be
        // a reading of the difference between the populations.
        let keyed_coverage = keyed.coverage.unwrap_or_else(|| panic!("keyed coverage at {labels}"));
        let null_coverage = null.coverage.unwrap_or_else(|| panic!("null coverage at {labels}"));
        assert!(
            (keyed_coverage - null_coverage).abs() < COVERAGE_TOLERANCE,
            "the two blocks cover the same traffic at {labels} labels: {keyed_coverage} against {null_coverage}"
        );
    }

    // The rebuild costs the reading nothing. The keyed block's association is
    // higher at the first point after the layout was rebuilt underneath it
    // than at the last point before, so the reading did not merely resume: it
    // went on climbing through the event, which a frozen window could not
    // have done because the window itself would have collapsed.
    let before = points[REBUILD_AFTER_LEG - 1]
        .2
        .association
        .expect("the keyed association at the last point before the rebuild");
    let after = points[REBUILD_AFTER_LEG]
        .2
        .association
        .expect("the keyed association at the first point after the rebuild");
    assert!(
        after > before,
        "the keyed reading climbs through the rebuild: {after} against {before}"
    );
}

/// One bit of an avalanche over the request index under the given salt.
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
