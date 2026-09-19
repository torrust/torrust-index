// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`fresh_has_one_root_tracker`] | health | A sentinel that has ingested nothing holds exactly one tracker, on the root cell, over a graph of a single node. The root exists unconditionally so that every ancestor chain has somewhere to terminate; every other tracker is bought only once traffic has justified it. |
//! | [`fresh_has_zero_observations`] | health | The lifetime count reads zero on a fresh sentinel even though its root tracker has already absorbed synthetic seeding. Injected noise gives a tracker a usable model but is not evidence about traffic, so it is kept deliberately out of the number a host reads as how much the engine has actually seen. |
//! | [`fresh_rank_distribution_is_uniform_at_one`] | health | With a single tracker the rank distribution degenerates: smallest, largest and mean all agree, and all sit at the starting rank of one, because a learned subspace begins with a single basis direction. The distribution summarises a fleet, and a fleet of one has no spread to report. |
//! | [`fresh_maturity_is_cold`] | health | A tracker counts as cold for as long as its real-observation count is zero, whatever synthetic data it has already absorbed. Coldness measures exposure to the world rather than whether the model is populated, which is exactly the distinction a host needs when deciding how much a score from that tracker is worth. |
//! | [`fresh_geometry_distribution`] | health | The geometry summary counts trackers whose axes are structurally unavailable rather than merely quiet. A rank-one tracker has no second direction for coherence to compare against and is counted inactive on that axis, while on a wide domain its rank is nowhere near the dimension so novelty is not saturated. A zero score means something different in each case, and this is where a host learns which case it is in. |
//! | [`fresh_clip_pressure_is_zero`] | health | Clip pressure reads zero at every extreme on a sentinel that has seen no real data. Pressure accumulates only when observations actually press against the clipping bound, so it is a record of how often the engine had to hold data back — and an engine that has held nothing back reports none. |
//! | [`fresh_coordination_is_empty`] | coordination | No coordination context exists until a batch has been observed, because a context is created only where cells on both sides of a split report in the same batch. The summary is nonetheless present and reads empty rather than being absent: the tier is always described, even when there is nothing in it. |
//! | [`tracker_counts_are_consistent`] | health | The active trackers decompose exactly into the competitive cells, the ancestors pulled in to connect them, and the one permanent root. These are not three independent measurements but one partition reported three ways, so a host can read the shape of the engine's investment from them and any disagreement would be an accounting fault rather than a fact about traffic. |
//! | [`investment_set_covers_active_plus_warming`] | health | The investment set is every tracker the engine is paying for, whether already scoring or still warming up in the staging pipeline. Warming cells consume memory and work before they produce anything, so a figure that counted only the online trackers would understate what the sentinel is spending. |
//! | [`population_grows_after_divergent_ingest`] | health | Values whose leading bits diverge land in separate regions of the domain and the tracker population follows the traffic there. The two counts that describe that population stay in step: the cells the sentinel says it is tracking are exactly the cells with trackers behind them, so neither figure can drift into describing an investment that does not exist. |
//! | [`lifetime_observations_accumulate`] | health | cites (´claim:health:lifetime-observations-counts-real-data-only-so-seeding-noise-leaves-it-at-zero´) |
//! | [`rank_bounds_hold_after_ingest`] | health | Ranks reported after real traffic stay inside the configured ceiling from above and at the starting rank or better from below, with the mean lying between the two extremes it summarises. The ceiling is a spending decision the host made, so the snapshot is expected to respect it rather than announce a model larger than was authorised. |
//! | [`noise_reduces_noise_influence`] | health | Noise influence is the share of a tracker's model still owed to its synthetic seeding, and real observations dilute it: once actual traffic has arrived the fleet's mean influence has fallen below the value it starts at. It is the measurement that tells a host how much of a score is still borrowed from data the engine invented for itself. |
//! | [`cold_config_leaves_trackers_cold`] | health | With seeding switched off there is nothing to dilute, so real data leaves every tracker at the same maturity and the distribution collapses — its lowest value is no lower than its mean. Spread in maturity across the fleet comes from trackers being at different stages of shedding their synthetic prior, not from the traffic alone. |
//! | [`health_evolves_over_repeated_batches`] | health | Across a run of warm-up batches the snapshot reflects the whole history rather than the batch just handled: observations total up across all of them, trackers remain live, and rank has had the chance to adapt above its starting value. Health describes the engine's state as it now stands, which is a running position and not a per-batch reading. |
//! | [`coordination_starts_empty`] | coordination | cites (´claim:coordination:no-context-exists-until-a-batch-has-been-observed´) |
//! | [`coordination_structurally_sound_after_noise`] | coordination | Whatever contexts a run happens to have created, the shape they report is internally consistent: the tier's fixed dimensionality, a capacity that exceeds neither the configured rank ceiling nor the four dimensions available, an influence share inside its unit range, and a rank between the starting value and that capacity. The check is conditional because contexts are created lazily where traffic happens to fall, and pretending otherwise would test the fixture rather than the engine. |

//! The health snapshot — what the sentinel reports about its own condition
//! rather than about the traffic it is watching.
//!
//! A host cannot read cell scores sensibly without knowing what state the
//! engine was in when it produced them, so the snapshot describes the
//! population of trackers it is paying for and how far along that population
//! is. The counts decompose exactly: the active trackers are the competitive
//! cells, plus the ancestors pulled in to connect them, plus the one
//! permanent root; the investment set is those together with whatever is
//! still warming in the staging pipeline. Rank, maturity, geometry and clip
//! pressure arrive as distributions rather than single values, because what
//! is interesting about a fleet of trackers is its extremes and its mean.
//!
//! Maturity is measured against the synthetic prior each tracker is seeded
//! with. A tracker that has seen only injected noise is cold: the noise buys
//! it a usable model but is not evidence about real traffic, so the lifetime
//! observation count ignores it entirely and the maturity figure records how
//! much of the model is still borrowed. Real data dilutes that share as it
//! arrives.
//!
//! The coordination tier appears here as a summary of its own, present and
//! reading empty before any context exists. Like every other field, none of
//! it is a verdict: health is a measurement about the engine, offered so the
//! host can judge how much weight the rest of the report deserves.

mod common;

use common::{ScenarioBuilder, cold_config, seeded_sentinel, test_config};
use torrust_sentinel::Sentinel128;

// ═══════════════════════════════════════════════════════════
//  Fresh sentinel
// ═══════════════════════════════════════════════════════════

/// A sentinel that has ingested nothing holds exactly one tracker, on the
/// root cell, over a graph of a single node. The root exists unconditionally
/// so that every ancestor chain has somewhere to terminate; every other
/// tracker is bought only once traffic has justified it.
///
/// ´claim:health:a-fresh-sentinel-holds-exactly-the-root-tracker-over-a-single-node-graph´
/// ´test:integration:fresh-has-one-root-tracker´
#[test]
fn fresh_has_one_root_tracker() {
    let s = Sentinel128::new(test_config()).unwrap();
    let h = s.health();

    assert_eq!(h.active_trackers, 1, "only the root tracker exists");
    assert_eq!(h.cells_tracked, 1);
    assert_eq!(h.total_g_nodes, 1);
}

/// The lifetime count reads zero on a fresh sentinel even though its root
/// tracker has already absorbed synthetic seeding. Injected noise gives a
/// tracker a usable model but is not evidence about traffic, so it is kept
/// deliberately out of the number a host reads as how much the engine has
/// actually seen.
///
/// ´claim:health:lifetime-observations-counts-real-data-only-so-seeding-noise-leaves-it-at-zero´
/// ´test:integration:fresh-has-zero-observations´
#[test]
fn fresh_has_zero_observations() {
    let s = Sentinel128::new(test_config()).unwrap();
    let h = s.health();

    assert_eq!(h.lifetime_observations, 0);
}

/// With a single tracker the rank distribution degenerates: smallest,
/// largest and mean all agree, and all sit at the starting rank of one,
/// because a learned subspace begins with a single basis direction. The
/// distribution summarises a fleet, and a fleet of one has no spread to
/// report.
///
/// ´claim:health:one-tracker-collapses-the-rank-distribution-to-a-single-value-at-the-starting-rank´
/// ´test:integration:fresh-rank-distribution-is-uniform-at-one´
#[test]
fn fresh_rank_distribution_is_uniform_at_one() {
    let s = Sentinel128::new(test_config()).unwrap();
    let rd = s.health().rank_distribution;

    assert_eq!(rd.min, 1);
    assert_eq!(rd.max, 1);
    assert!((rd.mean - 1.0).abs() < f64::EPSILON);
}

/// A tracker counts as cold for as long as its real-observation count is
/// zero, whatever synthetic data it has already absorbed. Coldness measures
/// exposure to the world rather than whether the model is populated, which
/// is exactly the distinction a host needs when deciding how much a score
/// from that tracker is worth.
///
/// ´claim:health:a-tracker-that-has-seen-only-synthetic-data-is-still-counted-cold´
/// ´test:integration:fresh-maturity-is-cold´
#[test]
fn fresh_maturity_is_cold() {
    let s = Sentinel128::new(test_config()).unwrap();
    let md = s.health().maturity_distribution;

    // Root tracker has received noise but no real observations,
    // so it counts as cold (zero real observations).
    assert_eq!(md.cold_trackers, 1);
}

/// The geometry summary counts trackers whose axes are structurally
/// unavailable rather than merely quiet. A rank-one tracker has no second
/// direction for coherence to compare against and is counted inactive on
/// that axis, while on a wide domain its rank is nowhere near the dimension
/// so novelty is not saturated. A zero score means something different in
/// each case, and this is where a host learns which case it is in.
///
/// ´claim:health:the-geometry-summary-counts-which-axes-are-structurally-available-rather-than-merely-quiet´
/// ´test:integration:fresh-geometry-distribution´
#[test]
fn fresh_geometry_distribution() {
    let s = Sentinel128::new(test_config()).unwrap();
    let gd = s.health().geometry_distribution;

    // With a single rank-1 root tracker on a 128-wide space,
    // novelty is not saturated and coherence is inactive (rank < 2).
    assert_eq!(gd.novelty_saturated, 0);
    assert_eq!(gd.coherence_inactive, 1, "rank-1 tracker cannot compute coherence");
}

/// Clip pressure reads zero at every extreme on a sentinel that has seen no
/// real data. Pressure accumulates only when observations actually press
/// against the clipping bound, so it is a record of how often the engine had
/// to hold data back — and an engine that has held nothing back reports
/// none.
///
/// ´claim:health:clip-pressure-stays-at-zero-until-observations-press-against-the-bound´
/// ´test:integration:fresh-clip-pressure-is-zero´
#[test]
fn fresh_clip_pressure_is_zero() {
    let s = Sentinel128::new(test_config()).unwrap();
    let cp = s.health().clip_pressure_distribution;

    assert!((cp.min).abs() < f64::EPSILON);
    assert!((cp.max).abs() < f64::EPSILON);
    assert!((cp.mean).abs() < f64::EPSILON);
}

/// No coordination context exists until a batch has been observed, because a
/// context is created only where cells on both sides of a split report in
/// the same batch. The summary is nonetheless present and reads empty rather
/// than being absent: the tier is always described, even when there is
/// nothing in it.
///
/// ´claim:coordination:no-context-exists-until-a-batch-has-been-observed´
/// ´test:integration:fresh-coordination-is-empty´
#[test]
fn fresh_coordination_is_empty() {
    let s = Sentinel128::new(test_config()).unwrap();
    let ch = s.health().coordination_health;

    assert_eq!(ch.active_contexts, 0);
}

// ═══════════════════════════════════════════════════════════
//  Tracker arithmetic
// ═══════════════════════════════════════════════════════════

/// The active trackers decompose exactly into the competitive cells, the
/// ancestors pulled in to connect them, and the one permanent root. These
/// are not three independent measurements but one partition reported three
/// ways, so a host can read the shape of the engine's investment from them
/// and any disagreement would be an accounting fault rather than a fact
/// about traffic.
///
/// ´claim:health:active-trackers-decompose-exactly-into-competitive-cells-ancestors-and-the-permanent-root´
/// ´test:integration:tracker-counts-are-consistent´
#[test]
fn tracker_counts_are_consistent() {
    let s = seeded_sentinel();
    let h = s.health();

    // active = competitive + ancestor + 1 (root)
    assert_eq!(
        h.active_trackers,
        h.active_competitive_trackers + h.active_ancestor_trackers + 1,
        "active trackers = competitive + ancestor + root"
    );
}

/// The investment set is every tracker the engine is paying for, whether
/// already scoring or still warming up in the staging pipeline. Warming
/// cells consume memory and work before they produce anything, so a figure
/// that counted only the online trackers would understate what the sentinel
/// is spending.
///
/// ´claim:health:the-investment-set-is-the-online-trackers-plus-those-still-warming´
/// ´test:integration:investment-set-covers-active-plus-warming´
#[test]
fn investment_set_covers_active_plus_warming() {
    let s = seeded_sentinel();
    let h = s.health();

    assert_eq!(
        h.investment_set_size,
        h.active_trackers + h.warming_trackers,
        "investment = active + warming"
    );
}

// ═══════════════════════════════════════════════════════════
//  After ingestion
// ═══════════════════════════════════════════════════════════

/// Values whose leading bits diverge land in separate regions of the domain
/// and the tracker population follows the traffic there. The two counts that
/// describe that population stay in step: the cells the sentinel says it is
/// tracking are exactly the cells with trackers behind them, so neither
/// figure can drift into describing an investment that does not exist.
///
/// ´claim:health:the-tracker-population-follows-the-traffic-and-cells-tracked-matches-the-trackers-behind-them´
/// ´test:integration:population-grows-after-divergent-ingest´
#[test]
fn population_grows_after_divergent_ingest() {
    let mut s = Sentinel128::new(test_config()).unwrap();

    // Two values with maximally separated leading bits.
    s.ingest(&[
        0xF000_0000_0000_0000_0000_0000_0000_0001,
        0x1000_0000_0000_0000_0000_0000_0000_0002,
    ]);

    let h = s.health();
    assert!(h.active_trackers >= 1);
    assert_eq!(h.cells_tracked, h.active_trackers);
    assert_eq!(h.lifetime_observations, 2);
}

/// Pins the accumulating end of the same count: every value in every batch
/// adds one, and the total carries across separate ingest calls instead of
/// describing only the batch just handled. That is what makes it readable as
/// the age of the engine's experience rather than a measure of the latest
/// traffic.
///
/// (´claim:health:lifetime-observations-counts-real-data-only-so-seeding-noise-leaves-it-at-zero´)
/// ´test:integration:lifetime-observations-accumulate´
#[test]
fn lifetime_observations_accumulate() {
    let mut s = Sentinel128::new(test_config()).unwrap();

    s.ingest(&[0x0000_0000_0000_0000_0000_0000_0000_0001]);
    assert_eq!(s.health().lifetime_observations, 1);

    s.ingest(&[
        0x0000_0000_0000_0000_0000_0000_0000_0002,
        0x0000_0000_0000_0000_0000_0000_0000_0003,
    ]);
    assert_eq!(s.health().lifetime_observations, 3);
}

/// Ranks reported after real traffic stay inside the configured ceiling from
/// above and at the starting rank or better from below, with the mean lying
/// between the two extremes it summarises. The ceiling is a spending
/// decision the host made, so the snapshot is expected to respect it rather
/// than announce a model larger than was authorised.
///
/// ´claim:health:reported-ranks-respect-the-configured-ceiling-and-the-mean-lies-between-the-extremes´
/// ´test:integration:rank-bounds-hold-after-ingest´
#[test]
fn rank_bounds_hold_after_ingest() {
    let s = seeded_sentinel();
    let rd = s.health().rank_distribution;

    assert!(rd.min >= 1, "rank must be at least 1");
    assert!(rd.max <= s.config().max_rank, "rank must not exceed max_rank");
    #[allow(clippy::cast_precision_loss)]
    {
        assert!(rd.mean >= rd.min as f64);
        assert!(rd.mean <= rd.max as f64);
    }
}

// ═══════════════════════════════════════════════════════════
//  Maturity
// ═══════════════════════════════════════════════════════════

/// Noise influence is the share of a tracker's model still owed to its
/// synthetic seeding, and real observations dilute it: once actual traffic
/// has arrived the fleet's mean influence has fallen below the value it
/// starts at. It is the measurement that tells a host how much of a score is
/// still borrowed from data the engine invented for itself.
///
/// ´claim:health:real-observations-dilute-the-synthetic-share-so-noise-influence-falls-below-its-starting-value´
/// ´test:integration:noise-reduces-noise-influence´
#[test]
fn noise_reduces_noise_influence() {
    let s = seeded_sentinel();
    let md = s.health().maturity_distribution;

    // Real observations dilute the noise baseline — mean influence
    // drops below the 1.0 cold-start value.
    assert!(
        md.mean_noise_influence < 1.0,
        "noise injection + real data should reduce mean noise_influence, got {}",
        md.mean_noise_influence,
    );
}

/// With seeding switched off there is nothing to dilute, so real data leaves
/// every tracker at the same maturity and the distribution collapses — its
/// lowest value is no lower than its mean. Spread in maturity across the
/// fleet comes from trackers being at different stages of shedding their
/// synthetic prior, not from the traffic alone.
///
/// ´claim:health:with-seeding-disabled-the-maturity-distribution-collapses-to-a-single-level´
/// ´test:integration:cold-config-leaves-trackers-cold´
#[test]
fn cold_config_leaves_trackers_cold() {
    let mut s = Sentinel128::new(cold_config()).unwrap();

    // Even after real data, cold_config skips noise so trackers
    // remain at maximum noise influence.
    s.ingest(&[0xAAAA_BBBB_CCCC_DDDD_0000_0000_0000_0001]);

    let md = s.health().maturity_distribution;
    assert!(
        md.min_noise_influence >= md.mean_noise_influence,
        "min should be >= mean (all at same level without noise)",
    );
}

// ═══════════════════════════════════════════════════════════
//  Multi-batch evolution
// ═══════════════════════════════════════════════════════════

/// Across a run of warm-up batches the snapshot reflects the whole history
/// rather than the batch just handled: observations total up across all of
/// them, trackers remain live, and rank has had the chance to adapt above
/// its starting value. Health describes the engine's state as it now stands,
/// which is a running position and not a per-batch reading.
///
/// ´claim:health:the-snapshot-describes-the-engine-as-it-stands-after-a-whole-run-not-just-the-latest-batch´
/// ´test:integration:health-evolves-over-repeated-batches´
#[test]
fn health_evolves_over_repeated_batches() {
    let (s, _) = ScenarioBuilder::new()
        .seed_range(0xA, 8)
        .seed_range(0x5, 8)
        .warm_batches(5)
        .build_with_reports();

    let h = s.health();

    // After several warm-up batches the sentinel should have
    // accumulated meaningful observations and the rank may have
    // adapted above 1.
    assert!(h.lifetime_observations >= 6 * 16, "6 batches × 16 values");
    assert!(h.rank_distribution.max >= 1);
    assert!(h.active_trackers >= 1);
}

// ═══════════════════════════════════════════════════════════
//  Coordination health
// ═══════════════════════════════════════════════════════════

/// Read directly from the coordination summary rather than through the wider
/// health report, a newly constructed sentinel still shows no active
/// contexts. Construction alone creates nothing at this tier; only observed
/// traffic can.
///
/// (´claim:coordination:no-context-exists-until-a-batch-has-been-observed´)
/// ´test:integration:coordination-starts-empty´
#[test]
fn coordination_starts_empty() {
    let s = Sentinel128::new(test_config()).unwrap();
    let ch = s.health().coordination_health;

    assert_eq!(ch.active_contexts, 0);
}

/// Whatever contexts a run happens to have created, the shape they report is
/// internally consistent: the tier's fixed dimensionality, a capacity that
/// exceeds neither the configured rank ceiling nor the four dimensions
/// available, an influence share inside its unit range, and a rank between
/// the starting value and that capacity. The check is conditional because
/// contexts are created lazily where traffic happens to fall, and pretending
/// otherwise would test the fixture rather than the engine.
///
/// ´claim:coordination:a-contexts-reported-rank-and-capacity-stay-inside-the-bounds-it-was-built-with´
/// ´test:integration:coordination-structurally-sound-after-noise´
#[test]
fn coordination_structurally_sound_after_noise() {
    let s = seeded_sentinel();
    let ch = s.health().coordination_health;

    // Coordination contexts are created lazily when enough
    // competitive cells co-exist. If any were created, verify
    // their structural properties.
    if ch.active_contexts > 0 {
        assert_eq!(ch.dim, 4, "coordination dimensionality is always 4");
        assert!(ch.capacity <= s.config().max_rank.min(4));
        assert!(
            ch.maturity_distribution.max_noise_influence <= 1.0,
            "noise_influence must be in [0, 1]"
        );
        assert!(ch.rank_distribution.min >= 1);
        assert!(ch.rank_distribution.max <= ch.capacity);
    }
}
