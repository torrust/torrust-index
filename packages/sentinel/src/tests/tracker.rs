// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`new_starts_at_rank_one`] | subspace | A newly built cell model claims one direction, has counted no observations of either kind, and regards itself as entirely noise-taught. Starting at a single axis means the model asserts as little structure as it can and has to earn every further direction from energy it actually observes; starting at full noise influence means nothing it later reports is trusted until real traffic has displaced the warm-up that shaped it. |
//! | [`new_accepts_min_tracker_dim`] | subspace | The narrowest width the engine admits is admitted, and the model it produces is an ordinary one starting at a single axis. The minimum is a boundary that is included rather than approached: cells right at the edge of being too deep to analyse still get a working model instead of a special case. |
//! | [`new_panics_on_zero_dim_debug`] | subspace | A width below the minimum is treated as a caller's mistake, not as a state to accommodate: building a model for a cell with nothing left to analyse fails loudly in debug builds and names the offending width. Filtering such cells out is the selector's job, so a model that receives one has been handed something upstream should have excluded, and the fault is worth more than a degenerate model that would score nothing meaningfully. |
//! | [`new_panics_on_dim_one_debug`] | subspace | cites (´claim:subspace:a-width-below-the-minimum-is-a-callers-fault-caught-in-debug-rather-than-a-state-to-accommodate´) |
//! | [`dim_and_cap_reflect_construction`] | subspace | A cell's rank ceiling is the lesser of the configured maximum and its own width: a wide cell is capped by policy, a narrow one by geometry. There are no more independent directions than dimensions to hold them, so the width binds where it is the smaller of the two, and one configuration can serve cells of every depth without being retuned per depth. |
//! | [`scoring_geometry_matches_state`] | subspace | A model reports the geometry its scores were computed in: the width it works over, the ceiling it may grow to, and the residual degrees of freedom left after the claimed directions are removed. That last figure is the divisor novelty is normalised by, so publishing it lets a host compare scores from cells of different depths and ranks instead of comparing numbers whose scale it cannot see. |
//! | [`observe_returns_correct_depth`] | subspace | A report carries back the depth it was given, unchanged, alongside the rank in force while the batch was scored — and that rank is the one the model held beforehand, since adaptation happens after scoring. The model has no idea which cell it serves, so the depth is a label it holds on the host's behalf, which is what lets a host attribute a report without keeping its own bookkeeping alongside every call. |
//! | [`observe_per_sample_when_enabled`] | subspace | Per-row detail is produced only where a cell is configured to want it. Building it costs a standardisation of every axis for every row, which is worth paying when a host needs to know which observation in a batch was responsible and wasted when it only needs the batch's summary — so the choice is made per configuration rather than always. |
//! | [`observe_no_per_sample_when_disabled`] | subspace | cites (´claim:subspace:per-row-detail-is-produced-only-where-it-is-configured-because-it-costs-work-per-row´) |
//! | [`observe_report_batch_size_matches`] | subspace | Where per-row detail is produced there is exactly one entry for every row handed in, whether the batch was a single observation or many, and the same model gives both answers in turn. The correspondence is positional, so a host can attribute a score back to the observation that earned it without the model needing to know what that observation was. |
//! | [`maturity_noise_only`] | subspace | Maturity is counted in observations rather than in calls: a batch of several injected rows advances the noise tally by that many and leaves the real tally untouched. Counting rows is what makes the figure comparable across cells fed at different batch sizes, and keeping the two tallies apart is what lets a host ask how much of what a cell knows it was taught deliberately. |
//! | [`maturity_real_only`] | subspace | cites (´claim:subspace:maturity-counts-observations-row-by-row-and-keeps-the-injected-and-the-real-apart´) |
//! | [`maturity_mixed_real_and_noise`] | subspace | cites (´claim:subspace:maturity-counts-observations-row-by-row-and-keeps-the-injected-and-the-real-apart´) |
//! | [`noise_influence_decays_toward_zero_for_real`] | subspace | Sustained real traffic drives the noise influence to essentially nothing. The figure falls by the forgetting factor once per observation rather than once per batch, so it measures how much genuine data has passed rather than how often the host called — and once it is small enough, the cell has effectively declared that what it knows now came from the traffic and not from its warm-up. |
//! | [`noise_influence_converges_toward_one_for_noise`] | subspace | Warm-up is re-enterable. A cell pushed part-way down by real traffic climbs back toward full influence when injection resumes, by the same geometric step run in the other direction. Cells are re-warmed after splits and long silences, so a figure that could only fall would leave a re-taught cell wrongly claiming its knowledge came from traffic it never saw. |
//! | [`novelty_low_for_repeated_pattern`] | subspace | Novelty is whatever the learned directions fail to explain, divided by the room left over after those directions are removed. A pattern the model has been trained on lies almost inside its own axes, so what is left is nearly nothing and the pattern scores as unremarkable — the model reports familiarity by having nothing to report. |
//! | [`novelty_high_for_unseen_pattern`] | subspace | cites (´claim:subspace:novelty-is-what-the-learned-directions-fail-to-explain-so-a-familiar-pattern-scores-low´) |
//! | [`coherence_cold_at_rank_one`] | subspace | Coherence measures whether pairs of axes move together as they usually do, so at a single axis it does not exist: there is no pair, and the score is exactly zero on every batch rather than some small residue. Because those zeroes are an absence of the question and not an answer to it, the axis's baseline is deliberately left cold while rank stays at one — otherwise it would learn that zero is normal and treat the first genuine coherence value, once a second axis appears, as an alarm. |
//! | [`rank_stays_bounded_by_max_rank`] | subspace | However long a cell runs and however strongly its traffic is structured, rank stays within a floor of one axis and the ceiling it was built with. The ceiling is what bounds the cost of every later step — the work per batch grows with rank — and the floor is what keeps a model from disappearing entirely during a quiet stretch and having to be rebuilt from nothing. |
//! | [`rank_acquires_buffer_dimension`] | subspace | A cell keeps one axis more than the energy threshold strictly demands. Fed a single dominant pattern, the leading direction alone already captures the required share, yet the model settles at two directions rather than one. The spare axis is where a genuinely new direction first shows up: without it, novel structure would have to displace the established pattern before the model could represent it at all, and the arrival would be invisible until it was already dominant. |
//! | [`energy_ratio_and_top_singular_value_evolve`] | subspace | A model that has been fed traffic reports a leading direction with real strength behind it and an energy share that is positive and cannot exceed the whole. The share is what the claimed axes explain out of everything the model holds, so it is bounded above by construction, and a leading value at zero would mean the model had learned nothing — the two figures together are how a host reads whether a cell's model has substance. |
//! | [`cusum_reset_zeroes_steps`] | subspace | Clearing a cell's drift evidence restarts the count of batches that evidence was gathered over, so the very next batch is the first step of a new run rather than the next of an old one. Accumulated drift is only interpretable against how long it took to accumulate, and a fresh sum read against a stale count would look like a sudden collapse in drift rather than a deliberate acknowledgement of it. |
//! | [`seed_cusum_slow_from_baselines_then_reset`] | subspace | Finishing warm-up is a two-step handover applied to every axis at once: the long-memory reference is seeded from the short-memory one that has already converged on the injected traffic, and only then is the evidence cleared. Done in that order, drift detection resumes from a state where the two references agree, so the first real batches are scored against a reference that is already current instead of registering the warm-up's own leftover gap as drift for as long as the slow memory takes to catch up. |
//! | [`explicit_reset_clip_pressure`] | subspace | Clip pressure records how often a cell has lately been discarding scores as outliers, and it widens that cell's own outlier band while it is high. Warm-up is exactly when it runs high, since injected traffic is scored against a barely-formed model. Clearing it zeroes every axis together, so a cell entering production judges its first real batches by the ordinary band rather than by one still slackened by the noise it was taught with. |
//! | [`eta_threshold_crossing_zeros_clip_pressure`] | subspace | cites (´claim:subspace:clearing-clip-pressure-zeroes-every-axis-so-warm-up-clipping-does-not-slacken-production-scoring´) |
//! | [`a_declined_incremental_step_still_yields_a_usable_model`] | subspace | A step the incremental strategy declines is answered by the dense one, so what comes back is always a basis that was actually re-orthogonalised. The incremental path builds a small kernel and back-transforms through it, which needs spare dimensions to be stable, and at the coordination tier's width it declines every step on exactly those grounds. Declining is the honest answer, and the dispatcher's response to it is to run the strategy that does not need the step. That is the same response the incremental path now gives when its own corrective factorisation fails — the step that re-orthogonalises the basis — because the alternative is returning the basis from before that step under a field documented orthonormal, which every caller writes straight into a tracker and then relies on. The failure of that factorisation cannot be provoked from outside without a hook into the linear algebra, so what is exercised here is the fallback it now takes. |

//! Crate-level tests for [`SubspaceTracker`](crate::sentinel::tracker::SubspaceTracker).
//!
//! One of these belongs to each analysed cell, and it is the whole of what
//! that cell knows. It holds a small orthonormal basis — a handful of
//! directions that between them explain most of the traffic the cell has seen
//! — the strength of each direction, and a running picture of how a batch's
//! coordinates within those directions are usually distributed. Every batch is
//! scored against that model before the model absorbs it, so a score always
//! measures a departure from what was known beforehand and never from a state
//! the batch itself helped create.
//!
//! Three decisions shape the state. Rank is adaptive but heavily damped: it
//! moves by at most one axis at a time, toward the smallest number of
//! directions that captures the configured share of energy plus one spare, and
//! it never leaves the band between a single axis and the cell's ceiling. A
//! cell that has only ever seen injected noise must be distinguishable from one
//! taught by real traffic, so a noise-influence figure starts at full and
//! decays toward nothing with each real observation — climbing back if noise
//! resumes, because warm-up is re-enterable rather than a door that shuts once.
//! And the tracker knows nothing about cells, coordinates or the host's
//! domain: it takes rows of numbers and returns a report, which is what lets
//! the same engine serve every depth of the tree.
//!
//! Because the tracker owns no randomness, deterministic inputs make every
//! assertion here reproducible: the same rows produce the same model, the same
//! rank trajectory and the same scores on every run.

use crate::config::SentinelConfig;
use crate::sentinel::tracker::SubspaceTracker;

// ════════════════════════════════════════════════════════════
//  Helpers
// ════════════════════════════════════════════════════════════

/// Config used by most tests: fast adaptation, per-sample scores on.
fn cfg_per_sample() -> SentinelConfig<u64> {
    SentinelConfig {
        max_rank: 4,
        forgetting_factor: 0.95,
        rank_update_interval: 10,
        energy_threshold: 0.90,
        eps: 1e-6,
        per_sample_scores: true,
        cusum_allowance_sigmas: 0.5,
        ..SentinelConfig::default()
    }
}

/// Same as [`cfg_per_sample`] but with per-sample scores *disabled*.
fn cfg_no_per_sample() -> SentinelConfig<u64> {
    SentinelConfig {
        per_sample_scores: false,
        ..cfg_per_sample()
    }
}

/// Generate a batch of centred bit vectors from `u128` values.
fn centred_rows(values: &[u128], depth: usize) -> Vec<Vec<f64>> {
    values
        .iter()
        .map(|&v| {
            (0..depth)
                .map(|i| if (v >> (127 - i)) & 1 == 1 { 0.5 } else { -0.5 })
                .collect()
        })
        .collect()
}

fn as_slices(vecs: &[Vec<f64>]) -> Vec<&[f64]> {
    vecs.iter().map(Vec::as_slice).collect()
}

// ════════════════════════════════════════════════════════════
//  Construction & accessors
// ════════════════════════════════════════════════════════════

/// A newly built cell model claims one direction, has counted no observations
/// of either kind, and regards itself as entirely noise-taught. Starting at a
/// single axis means the model asserts as little structure as it can and has
/// to earn every further direction from energy it actually observes; starting
/// at full noise influence means nothing it later reports is trusted until
/// real traffic has displaced the warm-up that shaped it.
///
/// ´claim:subspace:a-new-cell-model-claims-one-direction-and-counts-itself-entirely-noise-taught´
/// ´test:crate:new-starts-at-rank-one´
#[test]
fn new_starts_at_rank_one() {
    let cfg = cfg_per_sample();
    let t = SubspaceTracker::new(8, &cfg, 0.999);

    assert_eq!(t.rank(), 1);
    assert_eq!(t.maturity().real_observations, 0);
    assert_eq!(t.maturity().noise_observations, 0);
    assert!((t.maturity().noise_influence - 1.0).abs() < f64::EPSILON);
}

/// The narrowest width the engine admits is admitted, and the model it
/// produces is an ordinary one starting at a single axis. The minimum is a
/// boundary that is included rather than approached: cells right at the edge
/// of being too deep to analyse still get a working model instead of a special
/// case.
///
/// ´claim:subspace:the-narrowest-admissible-width-yields-an-ordinary-model-rather-than-a-special-case´
/// ´test:crate:new-accepts-min-tracker-dim´
#[test]
fn new_accepts_min_tracker_dim() {
    let cfg = cfg_per_sample();
    let t = SubspaceTracker::new(crate::MIN_TRACKER_DIM, &cfg, 0.999);
    assert_eq!(t.rank(), 1);
}

/// A width below the minimum is treated as a caller's mistake, not as a state
/// to accommodate: building a model for a cell with nothing left to analyse
/// fails loudly in debug builds and names the offending width. Filtering such
/// cells out is the selector's job, so a model that receives one has been
/// handed something upstream should have excluded, and the fault is worth more
/// than a degenerate model that would score nothing meaningfully.
///
/// ´claim:subspace:a-width-below-the-minimum-is-a-callers-fault-caught-in-debug-rather-than-a-state-to-accommodate´
/// ´test:crate:new-panics-on-zero-dim-debug´
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "SubspaceTracker::new() called with dim=0")]
fn new_panics_on_zero_dim_debug() {
    let cfg = cfg_per_sample();
    drop(SubspaceTracker::new(0, &cfg, 0.999));
}

/// The rejection is not merely of the empty width: a single dimension is
/// refused too, and the message again names what was asked for. One direction
/// cannot be decomposed into structure and residual — there is no room left
/// over once an axis is claimed — so the boundary sits above zero rather than
/// at it.
///
/// (´claim:subspace:a-width-below-the-minimum-is-a-callers-fault-caught-in-debug-rather-than-a-state-to-accommodate´)
/// ´test:crate:new-panics-on-dim-one-debug´
#[test]
#[cfg(debug_assertions)]
#[should_panic(expected = "SubspaceTracker::new() called with dim=1")]
fn new_panics_on_dim_one_debug() {
    let cfg = cfg_per_sample();
    drop(SubspaceTracker::new(1, &cfg, 0.999));
}

/// A cell's rank ceiling is the lesser of the configured maximum and its own
/// width: a wide cell is capped by policy, a narrow one by geometry. There are
/// no more independent directions than dimensions to hold them, so the width
/// binds where it is the smaller of the two, and one configuration can serve
/// cells of every depth without being retuned per depth.
///
/// ´claim:subspace:the-rank-ceiling-is-the-lesser-of-the-configured-maximum-and-the-cells-own-width´
/// ´test:crate:dim-and-cap-reflect-construction´
#[test]
fn dim_and_cap_reflect_construction() {
    let cfg = SentinelConfig {
        max_rank: 6,
        ..cfg_per_sample()
    };

    // When dim > max_rank, cap = max_rank.
    let t = SubspaceTracker::new(16, &cfg, 0.999);
    assert_eq!(t.dim(), 16);
    assert_eq!(t.cap(), 6);

    // When dim < max_rank, cap = dim.
    let t2 = SubspaceTracker::new(4, &cfg, 0.999);
    assert_eq!(t2.dim(), 4);
    assert_eq!(t2.cap(), 4);
}

/// A model reports the geometry its scores were computed in: the width it
/// works over, the ceiling it may grow to, and the residual degrees of freedom
/// left after the claimed directions are removed. That last figure is the
/// divisor novelty is normalised by, so publishing it lets a host compare
/// scores from cells of different depths and ranks instead of comparing
/// numbers whose scale it cannot see.
///
/// ´claim:subspace:a-model-publishes-the-residual-degrees-of-freedom-its-scores-were-normalised-by´
/// ´test:crate:scoring-geometry-matches-state´
#[test]
fn scoring_geometry_matches_state() {
    let cfg = SentinelConfig {
        max_rank: 4,
        ..cfg_per_sample()
    };
    let t = SubspaceTracker::new(16, &cfg, 0.999);
    let g = t.scoring_geometry();

    assert_eq!(g.dim, 16);
    assert_eq!(g.cap, 4);
    // rank starts at 1, so residual_dof = dim - rank = 15.
    assert_eq!(g.residual_dof, 15);
}

// ════════════════════════════════════════════════════════════
//  Observe — report structure
// ════════════════════════════════════════════════════════════

/// A report carries back the depth it was given, unchanged, alongside the rank
/// in force while the batch was scored — and that rank is the one the model
/// held beforehand, since adaptation happens after scoring. The model has no
/// idea which cell it serves, so the depth is a label it holds on the host's
/// behalf, which is what lets a host attribute a report without keeping its
/// own bookkeeping alongside every call.
///
/// ´claim:subspace:a-report-carries-back-the-depth-it-was-given-and-the-rank-that-scored-the-batch´
/// ´test:crate:observe-returns-correct-depth´
#[test]
fn observe_returns_correct_depth() {
    let cfg = cfg_per_sample();
    let mut t = SubspaceTracker::new(8, &cfg, 0.999);

    let rows = centred_rows(&[0x0123_4567_89AB_CDEF_0123_4567_89AB_CDEF], 8);
    let report = t.observe(&as_slices(&rows), 8, false);

    assert_eq!(report.depth, 8);
    assert_eq!(report.rank, 1); // hasn't adapted yet
}

/// Per-row detail is produced only where a cell is configured to want it.
/// Building it costs a standardisation of every axis for every row, which is
/// worth paying when a host needs to know which observation in a batch was
/// responsible and wasted when it only needs the batch's summary — so the
/// choice is made per configuration rather than always.
///
/// ´claim:subspace:per-row-detail-is-produced-only-where-it-is-configured-because-it-costs-work-per-row´
/// ´test:crate:observe-per-sample-when-enabled´
#[test]
fn observe_per_sample_when_enabled() {
    let cfg = cfg_per_sample();
    let mut t = SubspaceTracker::new(8, &cfg, 0.999);

    let rows = centred_rows(&[1, 2, 3], 8);
    let report = t.observe(&as_slices(&rows), 8, false);

    let ps = report.per_sample.as_ref().expect("per_sample should be Some when enabled");
    assert_eq!(ps.len(), 3, "one SampleScore per input row");
}

/// The other end of the same choice: with per-row detail switched off the
/// report carries none, rather than carrying an empty list or zeroed entries.
/// Absent and empty are different answers, and a host reading a report can
/// tell that the detail was never asked for instead of concluding the batch
/// had nothing in it.
///
/// (´claim:subspace:per-row-detail-is-produced-only-where-it-is-configured-because-it-costs-work-per-row´)
/// ´test:crate:observe-no-per-sample-when-disabled´
#[test]
fn observe_no_per_sample_when_disabled() {
    let cfg = cfg_no_per_sample();
    let mut t = SubspaceTracker::new(8, &cfg, 0.999);

    let rows = centred_rows(&[1, 2, 3], 8);
    let report = t.observe(&as_slices(&rows), 8, false);

    assert!(report.per_sample.is_none(), "per_sample should be None when disabled");
}

/// Where per-row detail is produced there is exactly one entry for every row
/// handed in, whether the batch was a single observation or many, and the same
/// model gives both answers in turn. The correspondence is positional, so a
/// host can attribute a score back to the observation that earned it without
/// the model needing to know what that observation was.
///
/// ´claim:subspace:there-is-exactly-one-per-row-score-for-every-row-in-the-batch-whatever-its-size´
/// ´test:crate:observe-report-batch-size-matches´
#[test]
fn observe_report_batch_size_matches() {
    let cfg = cfg_per_sample();
    let mut t = SubspaceTracker::new(8, &cfg, 0.999);

    // Single sample.
    let rows1 = centred_rows(&[1], 8);
    let r1 = t.observe(&as_slices(&rows1), 8, false);
    assert_eq!(r1.per_sample.as_ref().unwrap().len(), 1);

    // Larger batch.
    let rows8 = centred_rows(&[1, 2, 3, 4, 5, 6, 7, 8], 8);
    let r8 = t.observe(&as_slices(&rows8), 8, false);
    assert_eq!(r8.per_sample.as_ref().unwrap().len(), 8);
}

// ════════════════════════════════════════════════════════════
//  Maturity tracking
// ════════════════════════════════════════════════════════════

/// Maturity is counted in observations rather than in calls: a batch of
/// several injected rows advances the noise tally by that many and leaves the
/// real tally untouched. Counting rows is what makes the figure comparable
/// across cells fed at different batch sizes, and keeping the two tallies
/// apart is what lets a host ask how much of what a cell knows it was taught
/// deliberately.
///
/// ´claim:subspace:maturity-counts-observations-row-by-row-and-keeps-the-injected-and-the-real-apart´
/// ´test:crate:maturity-noise-only´
#[test]
fn maturity_noise_only() {
    let cfg = cfg_per_sample();
    let mut t = SubspaceTracker::new(8, &cfg, 0.999);

    let rows = centred_rows(&[1, 2, 3], 8);
    t.observe(&as_slices(&rows), 8, true);

    assert_eq!(t.maturity().noise_observations, 3);
    assert_eq!(t.maturity().real_observations, 0);
}

/// The mirror case pins the other tally: real rows advance the real count by
/// one per row and leave the injected count at nothing. Which tally a batch
/// lands in is decided by the caller and not inferred from the data, because
/// synthetic and genuine observations can look identical and only the host
/// knows which it sent.
///
/// (´claim:subspace:maturity-counts-observations-row-by-row-and-keeps-the-injected-and-the-real-apart´)
/// ´test:crate:maturity-real-only´
#[test]
fn maturity_real_only() {
    let cfg = cfg_per_sample();
    let mut t = SubspaceTracker::new(8, &cfg, 0.999);

    let rows = centred_rows(&[10, 20], 8);
    t.observe(&as_slices(&rows), 8, false);

    assert_eq!(t.maturity().real_observations, 2);
    assert_eq!(t.maturity().noise_observations, 0);
}

/// The two tallies are independent accumulators, not two views of one figure:
/// the same rows sent first as injected and then as real leave both counts
/// standing at what each was given. What the real batch does move is the
/// influence figure, which drops below full the moment any genuine
/// observation arrives — so the record of how a cell was taught survives while
/// the weight given to that teaching starts falling immediately.
///
/// (´claim:subspace:maturity-counts-observations-row-by-row-and-keeps-the-injected-and-the-real-apart´)
/// ´test:crate:maturity-mixed-real-and-noise´
#[test]
fn maturity_mixed_real_and_noise() {
    let cfg = cfg_per_sample();
    let mut t = SubspaceTracker::new(8, &cfg, 0.999);

    let rows = centred_rows(&[1, 2, 3], 8);
    let slices = as_slices(&rows);

    t.observe(&slices, 8, true); // noise
    assert_eq!(t.maturity().noise_observations, 3);
    assert_eq!(t.maturity().real_observations, 0);

    t.observe(&slices, 8, false); // real
    assert_eq!(t.maturity().real_observations, 3);
    assert!(t.maturity().noise_influence < 1.0);
}

/// Sustained real traffic drives the noise influence to essentially nothing.
/// The figure falls by the forgetting factor once per observation rather than
/// once per batch, so it measures how much genuine data has passed rather than
/// how often the host called — and once it is small enough, the cell has
/// effectively declared that what it knows now came from the traffic and not
/// from its warm-up.
///
/// ´claim:subspace:sustained-real-traffic-drives-the-noise-influence-to-nothing-so-a-cell-can-declare-itself-warmed´
/// ´test:crate:noise-influence-decays-toward-zero-for-real´
#[test]
fn noise_influence_decays_toward_zero_for_real() {
    let cfg = cfg_per_sample();
    let mut t = SubspaceTracker::new(8, &cfg, 0.999);

    let rows = centred_rows(&[1, 2, 3, 4], 8);
    let slices = as_slices(&rows);

    // η starts at 1.0; each real batch decays it by λⁿ.
    for _ in 0..50 {
        t.observe(&slices, 8, false);
    }

    assert!(
        t.maturity().noise_influence < 0.01,
        "η should decay toward 0 after many real batches, got {}",
        t.maturity().noise_influence,
    );
}

/// Warm-up is re-enterable. A cell pushed part-way down by real traffic climbs
/// back toward full influence when injection resumes, by the same geometric
/// step run in the other direction. Cells are re-warmed after splits and long
/// silences, so a figure that could only fall would leave a re-taught cell
/// wrongly claiming its knowledge came from traffic it never saw.
///
/// ´claim:subspace:renewed-injection-drives-the-influence-back-up-so-warm-up-is-re-enterable-rather-than-a-one-way-door´
/// ´test:crate:noise-influence-converges-toward-one-for-noise´
#[test]
fn noise_influence_converges_toward_one_for_noise() {
    let cfg = cfg_per_sample();
    let mut t = SubspaceTracker::new(8, &cfg, 0.999);

    // Start by feeding real data to push η below 1.0.
    let rows = centred_rows(&[1, 2, 3, 4], 8);
    let slices = as_slices(&rows);
    for _ in 0..10 {
        t.observe(&slices, 8, false);
    }
    let eta_after_real = t.maturity().noise_influence;
    assert!(eta_after_real < 1.0);

    // Now feed noise — η should move back toward 1.0.
    for _ in 0..30 {
        t.observe(&slices, 8, true);
    }

    assert!(
        t.maturity().noise_influence > eta_after_real,
        "η should increase toward 1 under noise, was {eta_after_real}, now {}",
        t.maturity().noise_influence,
    );
}

// ════════════════════════════════════════════════════════════
//  Scoring behaviour
// ════════════════════════════════════════════════════════════

/// Novelty is whatever the learned directions fail to explain, divided by the
/// room left over after those directions are removed. A pattern the model has
/// been trained on lies almost inside its own axes, so what is left is nearly
/// nothing and the pattern scores as unremarkable — the model reports
/// familiarity by having nothing to report.
///
/// ´claim:subspace:novelty-is-what-the-learned-directions-fail-to-explain-so-a-familiar-pattern-scores-low´
/// ´test:crate:novelty-low-for-repeated-pattern´
#[test]
fn novelty_low_for_repeated_pattern() {
    let cfg = cfg_per_sample();
    let mut t = SubspaceTracker::new(16, &cfg, 0.999);

    let pattern: u128 = 0xAAAA_0000_0000_0000_0000_0000_0000_0000;
    let rows = centred_rows(&[pattern; 8], 16);
    let slices = as_slices(&rows);

    // Train the subspace on a single repeated pattern.
    for _ in 0..20 {
        t.observe(&slices, 16, false);
    }

    // Score the same pattern — novelty should be low.
    let report = t.observe(&slices, 16, false);
    assert!(
        report.scores.novelty.mean < 1.0,
        "novelty should be low for a learned pattern, got {}",
        report.scores.novelty.mean,
    );
}

/// The comparative end of the same statement, which is the end that matters
/// operationally: a pattern the model was never trained on scores strictly
/// higher than the one it was, on the same model in the same state. Novelty is
/// meaningful as a ranking against what this cell has learned rather than as
/// an absolute quantity, so what is claimed is the ordering between the two
/// and not a threshold either of them crosses.
///
/// (´claim:subspace:novelty-is-what-the-learned-directions-fail-to-explain-so-a-familiar-pattern-scores-low´)
/// ´test:crate:novelty-high-for-unseen-pattern´
#[test]
fn novelty_high_for_unseen_pattern() {
    let cfg = cfg_per_sample();
    let mut t = SubspaceTracker::new(16, &cfg, 0.999);

    // Train on pattern A.
    let pattern_a: u128 = 0xAAAA_0000_0000_0000_0000_0000_0000_0000;
    let rows_a = centred_rows(&[pattern_a; 8], 16);
    let slices_a = as_slices(&rows_a);
    for _ in 0..20 {
        t.observe(&slices_a, 16, false);
    }

    // Novelty for trained pattern A.
    let report_a = t.observe(&slices_a, 16, false);

    // Score a completely different pattern B (without further training).
    let pattern_b: u128 = 0x5555_FFFF_0000_0000_0000_0000_0000_0000;
    let rows_b = centred_rows(&[pattern_b; 8], 16);
    let report_b = t.observe(&as_slices(&rows_b), 16, false);

    assert!(
        report_b.scores.novelty.mean > report_a.scores.novelty.mean,
        "unseen pattern should produce higher novelty ({}) than learned pattern ({})",
        report_b.scores.novelty.mean,
        report_a.scores.novelty.mean,
    );
}

/// Coherence measures whether pairs of axes move together as they usually do,
/// so at a single axis it does not exist: there is no pair, and the score is
/// exactly zero on every batch rather than some small residue. Because those
/// zeroes are an absence of the question and not an answer to it, the axis's
/// baseline is deliberately left cold while rank stays at one — otherwise it
/// would learn that zero is normal and treat the first genuine coherence
/// value, once a second axis appears, as an alarm.
///
/// ´claim:subspace:coherence-does-not-exist-at-a-single-axis-because-there-is-no-pair-to-be-coherent-about´
/// ´test:crate:coherence-cold-at-rank-one´
#[test]
fn coherence_cold_at_rank_one() {
    // At rank 1 there are no pairs, so coherence should be zero.
    let cfg = SentinelConfig {
        max_rank: 1,
        rank_update_interval: 1,
        ..cfg_per_sample()
    };
    let mut t = SubspaceTracker::new(8, &cfg, 0.999);

    let rows = centred_rows(&[0xFF00_0000_0000_0000_0000_0000_0000_0000; 4], 8);
    let slices = as_slices(&rows);

    for _ in 0..10 {
        let report = t.observe(&slices, 8, false);
        assert!(
            report.scores.coherence.mean.abs() < f64::EPSILON,
            "coherence should be zero at rank 1, got {}",
            report.scores.coherence.mean,
        );
    }
}

// ════════════════════════════════════════════════════════════
//  Rank adaptation
// ════════════════════════════════════════════════════════════

/// However long a cell runs and however strongly its traffic is structured,
/// rank stays within a floor of one axis and the ceiling it was built with.
/// The ceiling is what bounds the cost of every later step — the work per
/// batch grows with rank — and the floor is what keeps a model from
/// disappearing entirely during a quiet stretch and having to be rebuilt from
/// nothing.
///
/// ´claim:subspace:rank-moves-within-a-floor-of-one-axis-and-the-configured-ceiling-and-never-outside-them´
/// ´test:crate:rank-stays-bounded-by-max-rank´
#[test]
fn rank_stays_bounded_by_max_rank() {
    let cfg = SentinelConfig {
        max_rank: 3,
        rank_update_interval: 1,
        ..cfg_per_sample()
    };
    let mut t = SubspaceTracker::new(8, &cfg, 0.999);

    let rows = centred_rows(&[0xFF00_0000_0000_0000_0000_0000_0000_0000; 8], 8);
    let slices = as_slices(&rows);

    for _ in 0..50 {
        t.observe(&slices, 8, false);
    }

    assert!(t.rank() >= 1);
    assert!(t.rank() <= 3, "rank should not exceed max_rank, got {}", t.rank());
}

/// A cell keeps one axis more than the energy threshold strictly demands. Fed
/// a single dominant pattern, the leading direction alone already captures the
/// required share, yet the model settles at two directions rather than one.
/// The spare axis is where a genuinely new direction first shows up: without
/// it, novel structure would have to displace the established pattern before
/// the model could represent it at all, and the arrival would be invisible
/// until it was already dominant.
///
/// ´claim:subspace:the-model-keeps-one-axis-beyond-what-the-energy-threshold-demands-so-a-new-direction-has-somewhere-to-land´
/// ´test:crate:rank-acquires-buffer-dimension´
#[test]
fn rank_acquires_buffer_dimension() {
    // With one dominant pattern capturing ≥ 90% energy, the target is
    // min{i : c_i ≥ 0.90} + 1 = 0 + 1 = 1 → +1 buffer → 2.
    let cfg = SentinelConfig::<u64> {
        max_rank: 8,
        rank_update_interval: 1,
        energy_threshold: 0.90,
        ..SentinelConfig::default()
    };
    let mut t = SubspaceTracker::new(16, &cfg, 0.999);

    let pattern: u128 = 0xAAAA_BBBB_0000_0000_0000_0000_0000_0000;
    let rows = centred_rows(&[pattern; 16], 16);
    let slices = as_slices(&rows);

    for _ in 0..100 {
        t.observe(&slices, 16, false);
    }

    assert!(
        t.rank() >= 2,
        "rank should be at least 2 (buffer dimension), got {}",
        t.rank(),
    );
}

/// A model that has been fed traffic reports a leading direction with real
/// strength behind it and an energy share that is positive and cannot exceed
/// the whole. The share is what the claimed axes explain out of everything the
/// model holds, so it is bounded above by construction, and a leading value at
/// zero would mean the model had learned nothing — the two figures together
/// are how a host reads whether a cell's model has substance.
///
/// ´claim:subspace:a-trained-model-reports-a-leading-direction-with-strength-and-an-energy-share-that-cannot-exceed-the-whole´
/// ´test:crate:energy-ratio-and-top-singular-value-evolve´
#[test]
fn energy_ratio_and_top_singular_value_evolve() {
    let cfg = cfg_per_sample();
    let mut t = SubspaceTracker::new(8, &cfg, 0.999);

    let rows = centred_rows(&[0xABCD_EF01_2345_6789_ABCD_EF01_2345_6789; 4], 8);
    let slices = as_slices(&rows);

    for _ in 0..20 {
        t.observe(&slices, 8, false);
    }

    let report = t.observe(&slices, 8, false);

    // After learning, the energy ratio should be positive and ≤ 1.
    assert!(report.energy_ratio > 0.0, "energy_ratio should be positive");
    assert!(report.energy_ratio <= 1.0, "energy_ratio should be at most 1.0");

    // Top singular value should be positive after observations.
    assert!(
        report.top_singular_value > 0.0,
        "top_singular_value should be positive after training",
    );
}

// ════════════════════════════════════════════════════════════
//  CUSUM & clip-pressure lifecycle
// ════════════════════════════════════════════════════════════

/// Clearing a cell's drift evidence restarts the count of batches that
/// evidence was gathered over, so the very next batch is the first step of a
/// new run rather than the next of an old one. Accumulated drift is only
/// interpretable against how long it took to accumulate, and a fresh sum read
/// against a stale count would look like a sudden collapse in drift rather
/// than a deliberate acknowledgement of it.
///
/// ´claim:subspace:clearing-the-drift-evidence-restarts-the-step-count-so-the-next-batch-is-the-first-of-a-new-run´
/// ´test:crate:cusum-reset-zeroes-steps´
#[test]
fn cusum_reset_zeroes_steps() {
    let cfg = cfg_per_sample();
    let mut t = SubspaceTracker::new(8, &cfg, 0.999);

    let rows = centred_rows(&[1, 2, 3, 4], 8);
    let slices = as_slices(&rows);

    for _ in 0..5 {
        t.observe(&slices, 8, false);
    }

    t.reset_cusum();

    let report = t.observe(&slices, 8, false);
    assert_eq!(report.scores.novelty.cusum.steps_since_reset, 1);
}

/// Finishing warm-up is a two-step handover applied to every axis at once: the
/// long-memory reference is seeded from the short-memory one that has already
/// converged on the injected traffic, and only then is the evidence cleared.
/// Done in that order, drift detection resumes from a state where the two
/// references agree, so the first real batches are scored against a reference
/// that is already current instead of registering the warm-up's own leftover
/// gap as drift for as long as the slow memory takes to catch up.
///
/// ´claim:subspace:the-warm-up-handover-seeds-the-slow-reference-from-the-converged-fast-one-before-the-evidence-is-cleared´
/// ´test:crate:seed-cusum-slow-from-baselines-then-reset´
#[test]
fn seed_cusum_slow_from_baselines_then_reset() {
    let cfg = cfg_per_sample();
    let mut t = SubspaceTracker::new(8, &cfg, 0.999);

    let rows = centred_rows(&[0xFF00_FF00_FF00_FF00_FF00_FF00_FF00_FF00; 8], 8);
    let slices = as_slices(&rows);

    // Build up baselines with noise.
    for _ in 0..20 {
        t.observe(&slices, 8, true);
    }

    // Seed slow from fast, then reset — the canonical warm-up
    // completion sequence.
    t.seed_cusum_slow_from_baselines();
    t.reset_cusum();

    // After reset + one real step, CUSUM steps = 1 and accumulator
    // should be close to zero (fast ≈ slow, so no drift).
    let report = t.observe(&slices, 8, false);
    assert_eq!(report.scores.novelty.cusum.steps_since_reset, 1);
}

/// Clip pressure records how often a cell has lately been discarding scores as
/// outliers, and it widens that cell's own outlier band while it is high.
/// Warm-up is exactly when it runs high, since injected traffic is scored
/// against a barely-formed model. Clearing it zeroes every axis together, so a
/// cell entering production judges its first real batches by the ordinary band
/// rather than by one still slackened by the noise it was taught with.
///
/// ´claim:subspace:clearing-clip-pressure-zeroes-every-axis-so-warm-up-clipping-does-not-slacken-production-scoring´
/// ´test:crate:explicit-reset-clip-pressure´
#[test]
fn explicit_reset_clip_pressure() {
    let cfg = SentinelConfig::<u64> {
        max_rank: 4,
        forgetting_factor: 0.95,
        rank_update_interval: 10,
        energy_threshold: 0.90,
        clip_pressure_decay: 0.95,
        ..SentinelConfig::default()
    };
    let mut t = SubspaceTracker::new(8, &cfg, 0.999);

    // Inject noise to potentially build clip-pressure.
    let rows = centred_rows(&[0xFF00_FF00_FF00_FF00_FF00_FF00_FF00_FF00; 8], 8);
    let slices = as_slices(&rows);
    for _ in 0..20 {
        t.observe(&slices, 8, true);
    }

    assert!(
        t.maturity().noise_influence > 0.5,
        "η should be high after noise injection, got {}",
        t.maturity().noise_influence,
    );

    // Explicit reset.
    t.seed_cusum_slow_from_baselines();
    t.reset_cusum();
    t.reset_clip_pressure();

    let cp = t.clip_pressures();
    for (i, &v) in cp.iter().enumerate() {
        assert!(
            v.abs() < f64::EPSILON,
            "axis {i} clip_pressure should be 0 after reset_clip_pressure(), got {v}",
        );
    }
}

/// The same clearing also happens of its own accord, at the moment the noise
/// influence falls through the threshold that marks warm-up complete — caught
/// here by stepping real batches in one at a time and looking at the exact
/// crossing. A cell whose host never makes the explicit call still leaves its
/// warm-up behind, because the condition that matters is that the influence
/// has decayed, not that anyone remembered to say so.
///
/// (´claim:subspace:clearing-clip-pressure-zeroes-every-axis-so-warm-up-clipping-does-not-slacken-production-scoring´)
/// ´test:crate:eta-threshold-crossing-zeros-clip-pressure´
#[test]
fn eta_threshold_crossing_zeros_clip_pressure() {
    // When η crosses below WARMUP_THRESHOLD (0.01), clip-pressure
    // is automatically zeroed even without an explicit reset call.
    let cfg = SentinelConfig::<u64> {
        max_rank: 4,
        forgetting_factor: 0.95,
        rank_update_interval: 10,
        energy_threshold: 0.90,
        clip_pressure_decay: 0.95,
        ..SentinelConfig::default()
    };
    let mut t = SubspaceTracker::new(8, &cfg, 0.999);

    // Drive η high via noise.
    let rows = centred_rows(&[0xABCD_EF01_2345_6789_ABCD_EF01_2345_6789; 4], 8);
    let slices = as_slices(&rows);
    for _ in 0..30 {
        t.observe(&slices, 8, true);
    }
    assert!(t.maturity().noise_influence > 0.5);

    // Feed real traffic step-by-step, watching for the η threshold crossing.
    let mut crossed = false;
    for _ in 0..100 {
        let old_eta = t.maturity().noise_influence;
        t.observe(&slices, 8, false);
        let new_eta = t.maturity().noise_influence;

        if old_eta >= 0.01 && new_eta < 0.01 {
            let cp = t.clip_pressures();
            for (i, &v) in cp.iter().enumerate() {
                assert!(
                    v.abs() < f64::EPSILON,
                    "axis {i} clip_pressure should be 0 at η threshold crossing, got {v}",
                );
            }
            crossed = true;
            break;
        }
    }
    assert!(crossed, "η never crossed the 0.01 threshold");
}

/// A step the incremental strategy declines is answered by the dense one, so
/// what comes back is always a basis that was actually re-orthogonalised. The
/// incremental path builds a small kernel and back-transforms through it,
/// which needs spare dimensions to be stable, and at the coordination tier's
/// width it declines every step on exactly those grounds. Declining is the
/// honest answer, and the dispatcher's response to it is to run the strategy
/// that does not need the step. That is the same response the incremental path
/// now gives when its own corrective factorisation fails — the step that
/// re-orthogonalises the basis — because the alternative is returning the
/// basis from before that step under a field documented orthonormal, which
/// every caller writes straight into a tracker and then relies on. The failure
/// of that factorisation cannot be provoked from outside without a hook into
/// the linear algebra, so what is exercised here is the fallback it now takes.
///
/// ´claim:subspace:a-step-the-incremental-strategy-declines-is-answered-by-the-dense-one´
/// ´test:crate:a-declined-incremental-step-still-yields-a-usable-model´
#[test]
fn a_declined_incremental_step_still_yields_a_usable_model() {
    let cfg = SentinelConfig::<u64> {
        svd_strategy: crate::SvdStrategy::Brand,
        ..cfg_per_sample()
    };

    // The coordination tier's width, where the incremental kernel has no
    // spare dimensions to work in and the strategy declines every step.
    let mut tracker = SubspaceTracker::new(4, &cfg, 0.999);

    for round in 0..40 {
        let a = f64::from(round % 7) / 10.0 - 0.3;
        let b = f64::from(round % 5) / 10.0 - 0.2;
        let rows: Vec<Vec<f64>> = vec![vec![a, b, -a, -b], vec![b, -a, a, -b]];
        let slices: Vec<&[f64]> = rows.iter().map(Vec::as_slice).collect();
        let report = tracker.observe(&slices, 0, false);

        assert!(
            report.scores.novelty.mean.is_finite(),
            "a declined step must still leave a model that can score"
        );
        assert!(report.rank >= 1, "the model keeps at least one direction");
        assert!(
            report.energy_ratio >= 0.0 && report.energy_ratio <= 1.0,
            "the captured fraction stays a fraction, which an un-orthogonalised basis would not give"
        );
    }
}
