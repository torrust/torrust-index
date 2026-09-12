// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`a_dense_competitive_schedule_leaves_every_precision_matrix_factorisable`] | wellness | A schedule that grows the competitive geometry until the model's feature width is in the thousands drives the periodic factorisation past the width at which it used to report the precision matrix as severely non-symmetric-positive-definite, and no model reports a cascade terminus. Ignored by default and run on demand with `cargo test -p torrust-assayer --test precision_definiteness -- --ignored`, because the per-label cost is quadratic in a width the schedule is itself growing and the run takes about two and a half minutes on the shared server: a guard left in the default suite would charge that to every chain twice over, and to every lane that follows, for a verdict nobody is waiting on between releases. |

//! The periodic factorisation's verdict on the precision matrix under a
//! schedule dense enough to grow the feature width into the thousands
//! (´dec:posterior:repair-cascade´).
//!
//! # What this harness is for
//!
//! The precision matrix is built out of operations that each preserve positive
//! definiteness — a scaling by a positive factor, a diagonal raised to a floor,
//! and the addition of a non-negatively weighted outer product — so the
//! periodic factorisation that inverts it should never fail. It did. A scan
//! driving two identity dimensions at a competitive cutoff of ten past three
//! thousand labels reported the matrix severely non-symmetric-positive-definite
//! on every periodic attempt from some point onward, and the effective weight
//! computed against the covariance that could then no longer be refreshed went
//! negative.
//!
//! This harness is that scan cut down to the minimum that reaches the failure,
//! and it is bounded in the two directions the scan was not: every stretch
//! carries its own deadline, and every checkpoint prints as it is taken, so a
//! run stopped anywhere is evidence up to where it stopped.
//!
//! # What a checkpoint prints
//!
//! Each checkpoint prints one aggregate row and then one `model=` line per
//! model. The aggregate row carries the extremes — the largest condition
//! estimate, the largest of each of the three synchronisation readings, and the
//! shortest effective interval — which is what a scan for a failure wants and
//! is not what a question about one model can be answered from: a maximum and a
//! minimum taken over four models name no model between them. The per-model
//! lines carry that model's own measured reading, the part of it the
//! replenishment clamp put there, the residual left when that part is taken
//! out, and the interval the cadence has left it at, so a row showing an
//! interval at the floor also shows whose interval it is and what the cadence
//! was acting on when it arrived there
//! (´def:monitoring:synchronisation-error´), (´dec:posterior:adaptive-cadence´).
//! Each per-model line closes with the three dispositions a visit can take —
//! the visits that measured and rebuilt nothing, the rebuilds paid for, and
//! the alarms — and the aggregate row carries the largest of the last two.
//! They are disjoint and their ratio is the reading: a model accumulating
//! measurements with its rebuild count flat is one the cadence is visiting
//! rather than rebuilding, and any alarm at all is a model whose drift was
//! real and whose fresh inverse did not improve on the pair it holds
//! (´dec:posterior:recomputation-trigger´),
//! (´dec:posterior:measured-adoption´). The line also carries `atres`, the
//! measurement's verdict on whether its own residual stood at or below the
//! rounding level of the reading it came out of
//! (´def:monitoring:synchronisation-error´).
//!
//! Beside the declines each line carries what the model's last rebuild
//! measured after itself and what the measurement then did with it, and the
//! aggregate row carries the largest after-drift. A decline has two possible
//! causes and the count alone names neither: the fresh inverse can be no
//! better than the pair the model already held, or it can be better and still
//! over the model's width-scaled threshold (´dec:posterior:measured-adoption´).
//! The `after` column read against the `sync` column beside it — the drift
//! standing before that same rebuild — is what separates them, and the
//! `adopt` column says which rebuild the pair was taken at. The verdict is
//! printed only where the factorisation refused, because a refusal is a third
//! reading again: a rebuild that offered nothing to adopt.
//!
//! # The geometry, and why it is the one that fails
//!
//! Both encodings place the coordinate index in the domain's top four bits and
//! zero in every bit below, so two entities of one coordinate are one point and
//! no dyadic depth separates them past the fourth. The registration
//! nevertheless carries a competitive cutoff of ten, which lets the mechanism
//! partition one population into a full part and an empty one for six levels
//! past the last informative split. Each of those levels adds a cell and each
//! cell adds a position to the score, so the width grows without the geometry
//! carrying any more information than it did at depth four. That is the shape
//! the failure was found in and it is reproduced rather than repaired here: a
//! registration is free to ask for a cutoff deeper than its encoding supports,
//! and the engine owes it a factorisable precision matrix either way.
//!
//! # Cross-References
//!
//! - (´dec:posterior:repair-cascade´) — the one attempted factorisation whose
//!   refusal is the terminus this harness watches for
//! - (´alg:update:sherman-morrison´) — the update whose leverage is read
//!   against the covariance the cascade's terminus stops refreshing

use std::collections::BTreeMap;
use std::io::Write;
use std::time::{Duration, Instant};

use torrust_assayer::testing::{
    LabelSpec, Progress, Scenario, World, advanced, cycle_request, fail_fast_on_model_owner_panic, finished, scenario_with,
    watch_progress,
};
use torrust_assayer::types::{DimensionId, EntityKey, IdentityBudget, OutcomeAxisId};
use torrust_assayer::{ChannelPolicy, IdentityDimensionRegistration, PublishedRebuildVerdict};

/// Instance id, so a world names itself in any diagnostic it emits.
const INSTANCE: &str = "precision-definiteness";

/// The world's seed. Fixed, because reproducibility is the harness contract.
const SEED: u64 = 0x0000_0005_0000_0113_u64;

/// The one declared channel every stream runs on.
const CHANNEL: &str = "default";

/// The first dimension's identifier.
const FIRST_DIM: DimensionId = DimensionId(1);

/// The second dimension's identifier.
const SECOND_DIM: DimensionId = DimensionId(2);

/// The outcome axis whose per-cell history a dimension block carries.
const AXIS: &str = "magnitude";

/// Coordinates per outcome half. The pool is twice this.
const POOL_HALF: u64 = 8;

/// The competitive cutoff the registrations ask for.
///
/// Six levels past the last depth the encoding can separate, which is what
/// grows the width: every competitive cell contributes one indicator feature
/// (´def:keyspace:competitive-indicators´).
const DEPTH_CUTOFF: u8 = 10;

/// Labels driven between two checkpoints.
const STRETCH: u64 = 200;

/// The most stretches the schedule drives.
///
/// The failure arrived at a feature width of about a thousand and fifty, some
/// twenty-nine hundred labels in; this reaches about twelve hundred positions,
/// which is past it with margin, and stops. The margin is not made larger
/// because the per-label cost is quadratic in a width the schedule grows, so
/// every further stretch costs several times what the last one did while the
/// verdict it can return is the one already in hand.
const MAX_STRETCHES: usize = 17;

/// The fewest competitive cells the run must reach for its verdict to mean
/// anything.
///
/// The failure needed the geometry, so a run that stopped at its wall before
/// growing it has not exercised what it is guarding and says so rather than
/// passing quietly.
const REQUIRED_CELLS: usize = 1000;

/// The wall the schedule will not drive past.
///
/// Roughly three times what the schedule above costs when the box is the
/// harness's own, so only a badly contended machine reaches it — which is a
/// condition to report rather than a verdict on the engine.
const DEADLINE_SECONDS: f64 = 900.0;

/// The wall one stretch will not drive past on its own.
///
/// The per-label cost is quadratic in a width the schedule grows, so a stretch
/// late in the run costs many times what an early one did. A budget checked
/// only between stretches would let one stretch overrun the whole run's.
const STRETCH_DEADLINE_SECONDS: f64 = 300.0;

/// How often the driving loop checks the wall.
const DEADLINE_STRIDE: u64 = 25;

/// How often the liveness watcher says where the harness is.
const HEARTBEAT_SECONDS: u64 = 20;

/// How long the harness may make no progress at all before it is a failure.
///
/// Every wait on the label path is unbounded: a flush sends a checkpoint
/// command to the model owner and blocks on an acknowledgement with no
/// timeout. A watcher outside that loop is what turns a harness that has
/// stopped advancing into a harness that says so.
const STALL_SECONDS: u64 = 120;

/// The salt the outcome stream is mixed under.
const CLASS_SALT: u64 = 0xb7e1_5162_8aed_2a6b;

/// The salt the second coordinate's half-selector is mixed under.
const SECOND_SALT: u64 = 0x243f_6a88_85a3_08d3;

/// The two-decimal field at `offset` of an entity name, read as a coordinate
/// in the domain's top four bits.
fn field_at(bytes: &[u8], offset: usize) -> u128 {
    let tens = bytes.get(offset).copied().unwrap_or(b'0').wrapping_sub(b'0');
    let ones = bytes.get(offset + 1).copied().unwrap_or(b'0').wrapping_sub(b'0');
    ((u128::from(tens) * 10 + u128::from(ones)) & 0xF) << 124
}

/// The first dimension's encoding: the entity's first field.
fn first_encoder(entity: &EntityKey) -> u128 {
    field_at(entity.as_bytes(), 1)
}

/// The second dimension's encoding: the entity's second field.
fn second_encoder(entity: &EntityKey) -> u128 {
    field_at(entity.as_bytes(), 4)
}

/// The entity one request arrives under, naming both coordinates.
fn entity_name(index: u64, adverse: bool) -> String {
    let first = (if adverse { POOL_HALF } else { 0 }) + index % POOL_HALF;
    let second = (if mixed_bit(index, SECOND_SALT) { POOL_HALF } else { 0 }) + index % POOL_HALF;
    format!("k{first:02}n{second:02}")
}

/// What one model looked like at one checkpoint.
///
/// The three synchronisation readings travel together because separately they
/// mislead: the measured reading is large on a model leaning on its prior and
/// says nothing about drift, and the residual alone hides how much of the
/// distance a recomputation cannot close
/// (´def:monitoring:synchronisation-error´).
struct ModelRow {
    /// The model's identifier, formatted so the rows sort.
    model: String,
    /// The whole reading the last recomputation measured before itself.
    sync_error: f64,
    /// The part of that reading the replenishment clamp put there
    /// (´req:gaussian:prior-replenishment-floor´).
    prior_induced: f64,
    /// What is left once that part is taken out, and the quantity the cadence
    /// acts on (´dec:posterior:adaptive-cadence´).
    residual: f64,
    /// Dimensions of this model sitting at the prior floor.
    at_floor: usize,
    /// This model's effective interval after any adaptive shortening.
    interval: u32,
    /// Rebuilds this model has paid for.
    recomputes: u64,
    /// Visits at which this model measured and rebuilt nothing
    /// (´dec:posterior:recomputation-trigger´).
    ///
    /// It rides beside the count above because the two are only meaningful
    /// together: they are the cadence's two dispositions, and their ratio is
    /// what says whether the model is being visited or rebuilt.
    measurements: u64,
    /// Rebuilds that were needed and left this model no better, or that the
    /// factorisation refused — the alarm
    /// (´dec:posterior:measured-adoption´).
    ///
    /// A non-zero reading is a condition to look at, not a rate to expect. A
    /// rebuild runs only where a measurement found drift over the threshold
    /// and above the resolution of its own reading, so a decline is a model
    /// whose drift is real and whose fresh inverse is worse than the pair it
    /// holds (´rep:assayer:declined-rebuild-cadence´).
    alarms: u64,
    /// Whether the last measurement's residual stood at or below the
    /// resolution of the reading it was taken out of
    /// (´def:monitoring:synchronisation-error´).
    at_resolution: bool,
    /// The drift the model's last rebuild measured after itself, absent until
    /// one has run (´dec:posterior:measured-adoption´).
    ///
    /// Read against the reading above, which stands before that same rebuild:
    /// the two are what say whether a decline was a fresh inverse that failed
    /// to improve on the pair the model held, or one that improved on it and
    /// was still over the model's threshold. The column above counts the
    /// declines and cannot say which of the two produced them.
    after: Option<f64>,
    /// What the model's last rebuild's factorisation answered, absent until one
    /// has run (´dec:posterior:measured-adoption´).
    verdict: Option<PublishedRebuildVerdict>,
    /// Whether that rebuild was adopted, absent until one has run.
    adopted: Option<bool>,
}

/// What the periodic factorisation and the geometry looked like at one
/// checkpoint.
struct Checkpoint {
    /// Competitive cells across both dimensions.
    cells: usize,
    /// The deepest dyadic depth either set has reached.
    depth: u8,
    /// The largest per-model condition-number estimate.
    kappa: f64,
    /// The largest per-model synchronisation error.
    sync_error: f64,
    /// The largest per-model prior-induced component of that reading
    /// (´req:gaussian:prior-replenishment-floor´).
    prior_induced: f64,
    /// The largest per-model residual: the reading less that component, and
    /// what the cadence acts on (´dec:posterior:adaptive-cadence´).
    residual: f64,
    /// The largest per-model drift measured after a rebuild, zero where no
    /// model has rebuilt (´dec:posterior:measured-adoption´).
    ///
    /// A maximum, like the readings beside it, and read like them: it names
    /// the worst reading any rebuild left behind and not the reading of any
    /// one model, which the per-model rows below carry.
    after: f64,
    /// Cascade terminus events across every model: the failure this watches
    /// for (´dec:posterior:repair-cascade´).
    terminus: u64,
    /// Rebuilds at which the spectral floor repaired the precision matrix,
    /// across every model (´dec:posterior:spectral-floor´).
    floored: u64,
    /// The largest per-model alarm count: a maximum rather than a sum, because
    /// the question the row answers is whether any one model is raising the
    /// alarm, and a sum over four models answers it for none of them
    /// (´dec:posterior:measured-adoption´).
    alarms: u64,
    /// The largest per-model count of visits that rebuilt nothing, taken as a
    /// maximum for the same reason (´dec:posterior:recomputation-trigger´).
    measurements: u64,
    /// The largest per-model count of dimensions sitting at the prior floor.
    at_floor: usize,
    /// The shortest effective recomputation interval across the models.
    interval: u32,
    /// Marginalisation events folded in so far.
    marginalisations: u64,
    /// Schur corrections discarded so far.
    schur_skipped: u64,
    /// Labels reverted at CP5: the surface a refused update reports on
    /// (´tab:runtime:numeric-checkpoints´).
    cp5: u64,
    /// One entry per model, sorted by name, so a row that shows an interval at
    /// the floor also shows whose it is.
    models: Vec<ModelRow>,
}

/// The competitive cells a set holds.
fn cells(depths: &BTreeMap<u8, usize>) -> usize {
    depths.values().sum()
}

/// The deepest dyadic depth a competitive set has reached.
fn deepest(depths: &BTreeMap<u8, usize>) -> u8 {
    depths.keys().next_back().copied().unwrap_or(0)
}

/// Reads one checkpoint off one published health report.
fn take_checkpoint(world: &World) -> Checkpoint {
    let report = world.assayer().full_health_report();
    let mut cell_total = 0;
    let mut depth = 0;
    for dimension in [FIRST_DIM, SECOND_DIM] {
        let health = &report.identity_dimensions[&dimension];
        cell_total += cells(&health.competitive_cell_depth_distribution);
        depth = depth.max(deepest(&health.competitive_cell_depth_distribution));
    }
    let mut kappa = 0.0_f64;
    let mut sync_error = 0.0_f64;
    let mut prior_induced = 0.0_f64;
    let mut residual = 0.0_f64;
    let mut after = 0.0_f64;
    let mut terminus = 0;
    let mut floored = 0;
    let mut alarms = 0;
    let mut measurements = 0;
    let mut at_floor = 0;
    let mut interval = u32::MAX;
    for detail in report.precision.values() {
        kappa = kappa.max(detail.diagonal_ratio);
        sync_error = sync_error.max(detail.last_sync_error);
        prior_induced = prior_induced.max(detail.last_prior_induced_sync_error);
        residual = residual.max(detail.last_sync_error_residual);
        // A model with no rebuild behind it contributes nothing, which is the
        // identity for a maximum rather than a claim that its last rebuild
        // measured zero (´dec:health:tiered-queries´).
        after = after.max(detail.last_sync_error_after.unwrap_or(0.0));
        terminus += detail.cascade_terminus_count;
        floored += detail.floored_rebuilds;
        alarms = alarms.max(detail.alarms);
        measurements = measurements.max(detail.measurements);
        at_floor = at_floor.max(detail.dimensions_at_floor);
        interval = interval.min(detail.n_recompute_effective);
    }
    let mut models: Vec<ModelRow> = report
        .precision
        .iter()
        .map(|(id, detail)| ModelRow {
            model: format!("{id:?}"),
            sync_error: detail.last_sync_error,
            prior_induced: detail.last_prior_induced_sync_error,
            residual: detail.last_sync_error_residual,
            at_floor: detail.dimensions_at_floor,
            interval: detail.n_recompute_effective,
            recomputes: detail.cholesky_recomputes,
            measurements: detail.measurements,
            alarms: detail.alarms,
            at_resolution: detail.last_measurement_at_resolution,
            after: detail.last_sync_error_after,
            verdict: detail.last_rebuild_verdict,
            adopted: detail.last_rebuild_adopted,
        })
        .collect();
    models.sort_by(|left, right| left.model.cmp(&right.model));
    let summary = world.assayer().health_summary();
    Checkpoint {
        cells: cell_total,
        depth,
        kappa,
        sync_error,
        prior_induced,
        residual,
        after,
        terminus,
        floored,
        alarms,
        measurements,
        at_floor,
        interval,
        marginalisations: report.marginalisation.events,
        schur_skipped: report.schur_corrections_skipped,
        cp5: summary.cp5_reverts,
        models,
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
            description: "dense competitive geometry harness".to_owned(),
            coordinate_semantics: "the leading nibble names the coordinate".to_owned(),
            domain_bits: 128,
            depth_cutoff: DEPTH_CUTOFF,
            budget: IdentityBudget::for_depth_cutoff(DEPTH_CUTOFF),
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

/// Prints one line and flushes it, so a run stopped anywhere is evidence up to
/// where it stopped.
#[allow(clippy::print_stdout)] // Justified: this harness's table is its evidence, and evidence a reader never sees is none — every other line here reaches stdout through this one function.
fn say(line: &str) {
    println!("{line}");
    std::io::stdout().flush().ok();
}

/// Drives one stretch, stopping early if either wall falls inside it.
fn drive(world: &World, axis: OutcomeAxisId, from: u64, started: &Instant, progress: &Progress) -> u64 {
    let stretch_started = Instant::now();
    let mut driven = 0;
    for index in from..from + STRETCH {
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
        driven += 1;
        advanced(progress);
        if driven % DEADLINE_STRIDE == 0
            && (started.elapsed().as_secs_f64() > DEADLINE_SECONDS
                || stretch_started.elapsed().as_secs_f64() > STRETCH_DEADLINE_SECONDS)
        {
            break;
        }
    }
    driven
}

/// One model's drift after its last rebuild, or a placeholder where it has not
/// rebuilt, at the width the row's other readings print at.
fn after_column(after: Option<f64>) -> String {
    after.map_or_else(|| format!("{:>11}", "absent"), |value| format!("{value:11.3e}"))
}

/// What the model did with its last rebuild: adopted, declined, or no rebuild
/// yet. A placeholder rather than a false, because a model that has not
/// rebuilt has not declined anything (´dec:posterior:measured-adoption´).
const fn adopt_column(adopted: Option<bool>) -> &'static str {
    match adopted {
        Some(true) => "yes",
        Some(false) => "no ",
        None => "-- ",
    }
}

/// The verdict, printed only where the factorisation did not answer cleanly.
///
/// A Refused rebuild is a different reading from a rebuild whose answer the
/// measurement declined, and the column exists to keep the two apart; printing
/// Clean on every line would add a column that never varies over a run in
/// which no factorisation fails (´dec:posterior:measured-adoption´).
const fn verdict_note(verdict: Option<PublishedRebuildVerdict>) -> &'static str {
    match verdict {
        Some(PublishedRebuildVerdict::Refused) => " verdict=Refused",
        _ => "",
    }
}

/// Prints one checkpoint's whole row, and one line per model beneath it.
///
/// The row's four synchronisation figures are maxima taken separately, so they
/// need not come from one model and the row does not claim they do; the lines
/// beneath it are where one model's readings stand together.
fn print_row(labels: u64, started: &Instant, stretch_seconds: f64, point: &Checkpoint) {
    say(&format!(
        "{labels:6} {:8.1}s (+{stretch_seconds:6.1}s) cells {:5} depth {:2} | kappa {:11.3e} sync {:11.3e} prior {:11.3e} resid {:11.3e} after {:11.3e} floor {:5} interval {:5} | floored {:4} meas {:4} alarms {:4} terminus {:4} | marg {:5} schur-skipped {:5} cp5 {:5}",
        started.elapsed().as_secs_f64(),
        point.cells,
        point.depth,
        point.kappa,
        point.sync_error,
        point.prior_induced,
        point.residual,
        point.after,
        point.at_floor,
        point.interval,
        point.floored,
        point.measurements,
        point.alarms,
        point.terminus,
        point.marginalisations,
        point.schur_skipped,
        point.cp5,
    ));
    for model in &point.models {
        say(&format!(
            "{labels:6}   model={:14} sync={:11.3e} prior={:11.3e} resid={:11.3e} atres={:5} after={} adopt={} floor={:5} interval={:5} meas={:5} recomp={:5} alarms={:5}{}",
            model.model,
            model.sync_error,
            model.prior_induced,
            model.residual,
            model.at_resolution,
            after_column(model.after),
            adopt_column(model.adopted),
            model.at_floor,
            model.interval,
            model.measurements,
            model.recomputes,
            model.alarms,
            verdict_note(model.verdict),
        ));
    }
}

/// A schedule that grows the competitive geometry until the model's feature width is in the thousands drives the periodic factorisation past the width at which it used to report the precision matrix as severely non-symmetric-positive-definite, and no model reports a cascade terminus. Ignored by default and run on demand with `cargo test -p torrust-assayer --test precision_definiteness -- --ignored`, because the per-label cost is quadratic in a width the schedule is itself growing and the run takes about two and a half minutes on the shared server: a guard left in the default suite would charge that to every chain twice over, and to every lane that follows, for a verdict nobody is waiting on between releases.
///
/// ´claim:wellness:a-dense-competitive-schedule-leaves-every-precision-matrix-factorisable´
/// ´test:integration:a-dense-competitive-schedule-leaves-every-precision-matrix-factorisable´
#[test]
#[ignore = "two and a half minutes on the shared server: run with --ignored, and in the release sweeps"]
fn a_dense_competitive_schedule_leaves_every_precision_matrix_factorisable() {
    let started = Instant::now();
    fail_fast_on_model_owner_panic();
    let progress = watch_progress(Duration::from_secs(STALL_SECONDS), Duration::from_secs(HEARTBEAT_SECONDS));
    let (world, axis) = build_world();
    register_dimension(&world, FIRST_DIM, "account", first_encoder);
    register_dimension(&world, SECOND_DIM, "shadow", second_encoder);

    let mut at = 0;
    let mut worst = 0;
    let mut cells_reached = 0;
    for _ in 0..MAX_STRETCHES {
        let stretch_started = Instant::now();
        let driven = drive(&world, axis, at, &started, &progress);
        at += driven;
        let point = take_checkpoint(&world);
        print_row(at, &started, stretch_started.elapsed().as_secs_f64(), &point);
        worst = worst.max(point.terminus);
        cells_reached = cells_reached.max(point.cells);
        if driven < STRETCH || started.elapsed().as_secs_f64() > DEADLINE_SECONDS {
            say(&format!("the schedule stopped at its wall after {at} labels"));
            break;
        }
    }
    finished(&progress);
    say(&format!(
        "the schedule finished at {at} labels in {:.1}s",
        started.elapsed().as_secs_f64()
    ));

    assert_eq!(
        worst, 0,
        "the periodic factorisation reported the precision matrix severely non-SPD {worst} times over {at} labels"
    );
    assert!(
        cells_reached >= REQUIRED_CELLS,
        "the schedule reached {cells_reached} competitive cells against the {REQUIRED_CELLS} the failure needed, \
         so its verdict is about the wall it stopped at and not about the engine"
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
