// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`convergence_rounds_table`] | convergence | How many noise rounds each axis needs, and what those rounds cost, can be re-derived on demand rather than taken on trust from the schedule that ships. The diagnostic asserts nothing: it exists so that a change in the scoring pipeline can be checked against the round budget the schedule was sized from. |
//! | [`axis_drift_investigation`] | convergence | When an axis refuses to settle, a per-round trace of its scores, baseline, spread, clip pressure and ceiling is available beside the theory it is supposed to follow. Diagnosing a convergence failure means finding where the empirical trajectory leaves the predicted one, and that needs the trajectory itself rather than a pass or a fail. |
//! | [`svd_timing_comparison`] | convergence | The cost of a warm-up run can be measured with the tracing layer installed and again without it, so the instrument's own overhead is separable from what it measures. A timing figure used to size the noise schedule would otherwise silently include the cost of having taken it. |

#![allow(clippy::print_stderr)]

//! On-demand **convergence diagnostics** (run with `--ignored`).
//!
//! These tests produce detailed diagnostic tables for convergence
//! analysis and SVD timing.  They have no assertions — they exist
//! to print human-readable tables when investigating performance
//! or tuning the noise schedule (`noise_schedule.rounds_for_depth()`).
//!
//! Ported from `convergence_benchmark.rs` tests that were pure
//! `eprintln!` diagnostic dumps with no assertions:
//!
//! - `noise_baselines_converge` → [`convergence_rounds_table`]
//! - `derived_noise_rounds_table` → (merged into `convergence_rounds_table`)
//! - `wall_clock_convergence_cost` → criterion `warmup_cost_detailed` group
//! - `svd_timing_diagnostic` → [`svd_timing_comparison`]
//! - (new) [`axis_drift_investigation`] — added for ADR-S-013 §1a
//!   convergence-failure triage
//!
//! They are instruments rather than judgements, which is why they assert
//! nothing and are excluded from ordinary runs. A pass or a fail answers
//! whether a bound still holds; sizing the noise schedule, or working out
//! why an axis will not settle, needs the trajectory itself — the round at
//! which each axis converged, the cost of getting there, and the
//! per-round march of scores, baselines, spreads and ceilings against the
//! theory they are supposed to follow. Keeping the measurement separate
//! from the assertion also keeps a slow diagnostic out of the gate.
//!
//! # Running
//!
//! Run a single diagnostic:
//!
//! ```sh
//! cargo test -p torrust-sentinel -- --ignored convergence_rounds_table --nocapture
//! cargo test -p torrust-sentinel -- --ignored axis_drift_investigation --nocapture
//! cargo test -p torrust-sentinel -- --ignored svd_timing_comparison --nocapture
//! ```
//!
//! Or all at once:
//!
//! ```sh
//! cargo test -p torrust-sentinel convergence_diagnostics -- --ignored --nocapture
//! ```
//!
//! # §-references
//!
//! - §ALGO S-4.2 Phase 3 — Latent distribution cold→warm
//! - §ALGO S-7.1.1 — EWMA outlier filter / clipping ceiling
//! - ADR-S-013 — Warm-up convergence benchmark
//! - ADR-M-028 — Span-native tracing

use std::time::Instant;

use rand::SeedableRng;
use rand::rngs::SmallRng;

use super::convergence_common::{
    AXIS_NAMES, as_slices, cfg_production, cfg_test, find_converged_round, generate_noise, trailing_cv,
};
use crate::maths::bench_tracing::SpanTiming;
use crate::sentinel::tracker::SubspaceTracker;

// ════════════════════════════════════════════════════════════
//  Convergence-rounds table
// ════════════════════════════════════════════════════════════

/// How many noise rounds each axis needs, and what those rounds cost, can be
/// re-derived on demand rather than taken on trust from the schedule that
/// ships. The diagnostic asserts nothing: it exists so that a change in the
/// scoring pipeline can be checked against the round budget the schedule was
/// sized from.
///
/// ´claim:convergence:the-round-budget-behind-the-noise-schedule-can-be-re-derived-on-demand´
/// ´test:crate:convergence-rounds-table´
#[test]
#[ignore = "on-demand diagnostic — run with --ignored --nocapture"]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::similar_names,
    clippy::too_many_lines
)]
fn convergence_rounds_table() {
    let dim = 128;
    let reference_rounds = 500;
    let window = 20;
    let tolerances = [0.01, 0.10, 0.20, 0.20]; // nov, disp, surp, coh
    let (timing, _guard) = SpanTiming::install();

    eprintln!("\n╔══════════════════════════════════════════════════════════╗");
    eprintln!("║  Convergence-rounds diagnostic (ADR-S-013 §3–§4)       ║");
    eprintln!("╚══════════════════════════════════════════════════════════╝");

    for (label, base_cfg) in [
        ("test (λ=0.95, b=4)", cfg_test()),
        ("production (λ=0.99, b=16)", cfg_production()),
    ] {
        let batch_size = base_cfg.noise_batch_size;
        let lambda = base_cfg.forgetting_factor;

        // ── Collect baseline traces ──────────────────────
        timing.reset();
        let mut tracker = SubspaceTracker::new(dim, &base_cfg, base_cfg.cusum_slow_decay);
        let mut rng = SmallRng::seed_from_u64(42);

        let mut baseline_traces: [Vec<f64>; 4] = [
            Vec::with_capacity(reference_rounds),
            Vec::with_capacity(reference_rounds),
            Vec::with_capacity(reference_rounds),
            Vec::with_capacity(reference_rounds),
        ];

        for _ in 0..reference_rounds {
            let noise = generate_noise(dim, batch_size, &mut rng);
            let report = tracker.observe(&as_slices(&noise), 0, true);

            baseline_traces[0].push(report.scores.novelty.baseline.mean);
            baseline_traces[1].push(report.scores.displacement.baseline.mean);
            baseline_traces[2].push(report.scores.surprise.baseline.mean);
            baseline_traces[3].push(report.scores.coherence.baseline.mean);
        }

        // ── SVD per-round cost ───────────────────────────
        let brand_per_round_us = timing.total_ns("svd_brand") as f64 / 1_000.0 / reference_rounds as f64;
        let naive_ns = timing.total_ns("svd_naive");
        let naive_per_round_us = if naive_ns > 0 {
            naive_ns as f64 / 1_000.0 / reference_rounds as f64
        } else {
            0.0
        };

        eprintln!("\n  [{label}] — {reference_rounds} rounds, d={dim}, b={batch_size}");
        eprintln!("  SVD per-round: Brand {brand_per_round_us:.1} µs");
        if naive_ns > 0 {
            let speedup = naive_per_round_us / brand_per_round_us;
            eprintln!("  SVD per-round: Naïve {naive_per_round_us:.1} µs  ({speedup:.2}× speedup)");
        }

        // ── Candidate rounds table ──────────────────────
        eprintln!();
        if naive_ns > 0 {
            eprintln!("  noise_rds | conv? | nov  | disp | surp | coh  | Brand (ms) | Naïve (ms)");
            eprintln!("  ----------|-------|------|------|------|------|------------|----------");
        } else {
            eprintln!("  noise_rds | conv? | nov  | disp | surp | coh  | Brand (ms)");
            eprintln!("  ----------|-------|------|------|------|------|----------");
        }

        for candidate in [5, 10, 20, 50, 65, 100, 150, 200, 400] {
            if candidate > reference_rounds {
                continue;
            }

            let mut all_converged = true;
            let mut axis_rounds = [0usize; 4];

            for i in 0..4 {
                let slice = &baseline_traces[i][..candidate];
                let conv = find_converged_round(slice, window.min(candidate / 2), tolerances[i]);
                axis_rounds[i] = conv;
                if conv >= candidate.saturating_sub(window) {
                    all_converged = false;
                }
            }

            let brand_cost_ms = candidate as f64 * brand_per_round_us / 1000.0;
            let status = if all_converged { "yes" } else { "NO " };
            if naive_ns > 0 {
                let naive_cost_ms = candidate as f64 * naive_per_round_us / 1000.0;
                eprintln!(
                    "  {candidate:9} | {status:5} | {:4} | {:4} | {:4} | {:4} | {brand_cost_ms:10.1} | {naive_cost_ms:10.1}",
                    axis_rounds[0], axis_rounds[1], axis_rounds[2], axis_rounds[3],
                );
            } else {
                eprintln!(
                    "  {candidate:9} | {status:5} | {:4} | {:4} | {:4} | {:4} | {brand_cost_ms:10.1}",
                    axis_rounds[0], axis_rounds[1], axis_rounds[2], axis_rounds[3],
                );
            }
        }

        // ── Derived recommended rounds ───────────────────
        let mut axis_convergence = [0usize; 4];
        for (i, trace) in baseline_traces.iter().enumerate() {
            axis_convergence[i] = find_converged_round(trace, window, tolerances[i]);
        }
        let worst = *axis_convergence.iter().max().unwrap();
        let theoretical_eta_05 = 0.05_f64.log(lambda).ceil() as usize;

        eprintln!();
        eprintln!("  axis         | converged | CV (last 100) | tolerance");
        eprintln!("  -------------|-----------|---------------|----------");
        for (i, name) in AXIS_NAMES.iter().enumerate() {
            let cv = trailing_cv(&baseline_traces[i], 100) * 100.0;
            eprintln!(
                "  {name:12} | {:9} | {cv:12.4}% | {:8}%",
                axis_convergence[i],
                tolerances[i] * 100.0
            );
        }

        let recommended = worst.max(theoretical_eta_05);
        let with_margin = (recommended as f64 * 1.5).ceil() as usize;
        let brand_per_round_ms = brand_per_round_us / 1000.0;

        eprintln!();
        eprintln!("  theoretical η < 0.05: {theoretical_eta_05} rounds");
        eprintln!("  worst-case axis:      {worst} rounds");
        eprintln!(
            "  recommended:          {recommended} rounds  ({:.1} ms Brand)",
            recommended as f64 * brand_per_round_ms
        );
        eprintln!(
            "  + 50% margin:         {with_margin} rounds  ({:.1} ms Brand)",
            with_margin as f64 * brand_per_round_ms
        );
        eprintln!(
            "  current default:      {} rounds  ({:.1} ms Brand)",
            base_cfg.noise_schedule.rounds_for_depth(0),
            f64::from(base_cfg.noise_schedule.rounds_for_depth(0)) * brand_per_round_ms
        );

        let ratio = recommended as f64 / f64::from(base_cfg.noise_schedule.rounds_for_depth(0));
        if ratio > 1.0 {
            eprintln!("  ⚠ current default is {ratio:.1}× too low!");
        } else {
            eprintln!("  ✓ current default is sufficient ({ratio:.1}× of needed)");
        }
    }
}

// ════════════════════════════════════════════════════════════
//  Per-axis drift investigation
// ════════════════════════════════════════════════════════════

/// When an axis refuses to settle, a per-round trace of its scores, baseline,
/// spread, clip pressure and ceiling is available beside the theory it is
/// supposed to follow. Diagnosing a convergence failure means finding where the
/// empirical trajectory leaves the predicted one, and that needs the trajectory
/// itself rather than a pass or a fail.
///
/// ´claim:convergence:a-per-round-trace-is-available-when-an-axis-refuses-to-settle´
/// ´test:crate:axis-drift-investigation´
#[test]
#[ignore = "on-demand diagnostic — run with --ignored --nocapture"]
#[allow(clippy::cast_precision_loss, clippy::too_many_lines)]
fn axis_drift_investigation() {
    let cfg = cfg_test();
    let dim = 128;
    let total_rounds = 500;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    // Collect full traces: [round][axis] for baseline means, score means,
    // baseline variances, and clip-pressure EWMAs.
    let mut bl_means: Vec<[f64; 4]> = Vec::with_capacity(total_rounds);
    let mut sc_means: Vec<[f64; 4]> = Vec::with_capacity(total_rounds);
    let mut bl_vars: Vec<[f64; 4]> = Vec::with_capacity(total_rounds);
    let mut clip_pressures: Vec<[f64; 4]> = Vec::with_capacity(total_rounds);
    let mut ranks: Vec<usize> = Vec::with_capacity(total_rounds);
    let mut etas: Vec<f64> = Vec::with_capacity(total_rounds);

    for _ in 0..total_rounds {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        let report = tracker.observe(&as_slices(&noise), 0, true);

        bl_means.push([
            report.scores.novelty.baseline.mean,
            report.scores.displacement.baseline.mean,
            report.scores.surprise.baseline.mean,
            report.scores.coherence.baseline.mean,
        ]);
        sc_means.push([
            report.scores.novelty.mean,
            report.scores.displacement.mean,
            report.scores.surprise.mean,
            report.scores.coherence.mean,
        ]);
        bl_vars.push([
            report.scores.novelty.baseline.variance,
            report.scores.displacement.baseline.variance,
            report.scores.surprise.baseline.variance,
            report.scores.coherence.baseline.variance,
        ]);
        clip_pressures.push([
            report.scores.novelty.clip_pressure,
            report.scores.displacement.clip_pressure,
            report.scores.surprise.clip_pressure,
            report.scores.coherence.clip_pressure,
        ]);
        ranks.push(report.rank);
        etas.push(report.maturity.noise_influence);
    }

    eprintln!("\n╔══════════════════════════════════════════════════════════════════════════════════╗");
    eprintln!(
        "║  Axis drift investigation (λ={:.2}, b={}, d={dim})                              ║",
        cfg.forgetting_factor, cfg.noise_batch_size
    );
    eprintln!("╚══════════════════════════════════════════════════════════════════════════════════╝");

    // ── Per-axis detailed table ──────────────────────────
    for (ax, name) in AXIS_NAMES.iter().enumerate() {
        eprintln!("\n  ─── {name} ───");
        eprintln!(
            "  round | rank | η        | ρ̄        | score_mean   | bl_mean      | bl_var       | clip_ceil    | Δbl/bl (%)"
        );
        eprintln!("  ------|------|----------|----------|--------------|--------------|--------------|--------------|----------");

        let mut prev_bl = f64::NAN;
        for r in 0..total_rounds {
            if r < 30 || r % 10 == 0 || r == total_rounds - 1 {
                let eta = etas[r];
                let cp = clip_pressures[r][ax];
                let p = eta.max(cp);
                let eff_clip = cfg.clip_sigmas * (1.0 + p / (1.0 - p + cfg.eps));
                let clip_ceil = eff_clip.mul_add(bl_vars[r][ax].sqrt(), bl_means[r][ax]);
                let delta_pct = if prev_bl.is_nan() || prev_bl.abs() < 1e-12 {
                    0.0
                } else {
                    (bl_means[r][ax] - prev_bl) / prev_bl.abs() * 100.0
                };
                eprintln!(
                    "  {:5} | {:4} | {:.6} | {:.6} | {:12.8} | {:12.8} | {:12.8} | {:12.4} | {:+8.4}",
                    r, ranks[r], eta, cp, sc_means[r][ax], bl_means[r][ax], bl_vars[r][ax], clip_ceil, delta_pct
                );
                prev_bl = bl_means[r][ax];
            }
        }

        // Stationarity test: compare first-half and second-half means
        // of the score means (not baselines) post rank stabilisation.
        let post_rank = 30; // well after rank reaches 2
        let mid = usize::midpoint(post_rank, total_rounds);
        let first_half_mean: f64 = sc_means[post_rank..mid].iter().map(|s| s[ax]).sum::<f64>() / (mid - post_rank) as f64;
        let second_half_mean: f64 = sc_means[mid..].iter().map(|s| s[ax]).sum::<f64>() / (total_rounds - mid) as f64;
        let score_drift_pct = if first_half_mean.abs() > 1e-12 {
            (second_half_mean - first_half_mean) / first_half_mean * 100.0
        } else {
            0.0
        };

        // Same for baselines
        let first_half_bl: f64 = bl_means[post_rank..mid].iter().map(|s| s[ax]).sum::<f64>() / (mid - post_rank) as f64;
        let second_half_bl: f64 = bl_means[mid..].iter().map(|s| s[ax]).sum::<f64>() / (total_rounds - mid) as f64;
        let bl_drift_pct = if first_half_bl.abs() > 1e-12 {
            (second_half_bl - first_half_bl) / first_half_bl * 100.0
        } else {
            0.0
        };

        let cv = trailing_cv(&bl_means.iter().map(|b| b[ax]).collect::<Vec<_>>(), 100) * 100.0;

        eprintln!();
        eprintln!("  score mean drift (half1 vs half2): {score_drift_pct:+.4}%");
        eprintln!("  baseline drift   (half1 vs half2): {bl_drift_pct:+.4}%");
        eprintln!("  baseline CV (last 100 rounds):     {cv:.4}%");
    }

    // ── find_settled_round analysis ──────────────────────
    // Reproduce the failing test's methodology and show which axis/round causes failure.
    eprintln!("\n  ─── find_settled_round breakdown (5% tol, reference = last round) ───");
    let reference = *bl_means.last().unwrap();
    for axes_count in [3, 4] {
        eprintln!("\n  checking {axes_count} axes:");
        for (i, means) in bl_means.iter().enumerate().rev() {
            let violations: Vec<String> = (0..axes_count)
                .filter_map(|a| {
                    let ref_val = reference[a];
                    if ref_val.abs() < 1e-12 {
                        None
                    } else {
                        let err = (means[a] - ref_val).abs() / ref_val.abs();
                        if err >= 0.05 {
                            Some(format!("{}={:.4}%", AXIS_NAMES[a], err * 100.0))
                        } else {
                            None
                        }
                    }
                })
                .collect();
            if !violations.is_empty() {
                eprintln!("  last violation at round {i}: {}", violations.join(", "));
                // Show 3 rounds around the violation
                let start = i.saturating_sub(2);
                let end = (i + 2).min(total_rounds - 1);
                for (offset, bl_mean) in bl_means[start..=end].iter().enumerate() {
                    let r = start + offset;
                    let errs: Vec<String> = (0..axes_count)
                        .map(|a| {
                            let ref_val = reference[a];
                            let err = if ref_val.abs() < 1e-12 {
                                0.0
                            } else {
                                (bl_mean[a] - ref_val) / ref_val * 100.0
                            };
                            format!("{}={:+.3}%", AXIS_NAMES[a], err)
                        })
                        .collect();
                    let marker = if r == i { " ← violation" } else { "" };
                    eprintln!("    round {:3}: {}{marker}", r, errs.join(", "));
                }
                break;
            }
        }
    }
}

// ════════════════════════════════════════════════════════════
//  SVD timing comparison
// ════════════════════════════════════════════════════════════

/// The cost of a warm-up run can be measured with the tracing layer installed
/// and again without it, so the instrument's own overhead is separable from
/// what it measures. A timing figure used to size the noise schedule would
/// otherwise silently include the cost of having taken it.
///
/// ´claim:convergence:warm-up-cost-can-be-measured-apart-from-the-cost-of-measuring-it´
/// ´test:crate:svd-timing-comparison´
#[test]
#[ignore = "on-demand diagnostic — run with --ignored --nocapture"]
#[allow(clippy::cast_precision_loss)]
fn svd_timing_comparison() {
    let dim = 128;
    let rounds = 500;
    let n_reps = 5;

    eprintln!("\n╔══════════════════════════════════════════════════════════╗");
    eprintln!("║  SVD timing diagnostic: SpanTiming vs raw Instant       ║");
    eprintln!("╚══════════════════════════════════════════════════════════╝");
    eprintln!("  cfg!(debug_assertions) = {}", cfg!(debug_assertions));

    for (label, base_cfg) in [
        ("test (λ=0.95, b=4)", cfg_test()),
        ("production (λ=0.99, b=16)", cfg_production()),
    ] {
        let batch_size = base_cfg.noise_batch_size;

        // Warm up: 2 full runs to stabilise CPU frequency.
        for _ in 0..2 {
            let mut t = SubspaceTracker::new(dim, &base_cfg, base_cfg.cusum_slow_decay);
            let mut rng = SmallRng::seed_from_u64(42);
            for _ in 0..rounds {
                let noise = generate_noise(dim, batch_size, &mut rng);
                t.observe(&as_slices(&noise), 0, true);
            }
        }

        // ── A: WITH subscriber (SpanTimingLayer) ────────
        let mut span_brand_sum = 0u128;
        let mut span_naive_sum = 0u128;
        let mut span_wall_sum = 0.0_f64;

        for _ in 0..n_reps {
            let (timing, guard) = SpanTiming::install();
            timing.reset();
            let wall_start = Instant::now();
            {
                let mut t = SubspaceTracker::new(dim, &base_cfg, base_cfg.cusum_slow_decay);
                let mut rng = SmallRng::seed_from_u64(42);
                for _ in 0..rounds {
                    let noise = generate_noise(dim, batch_size, &mut rng);
                    t.observe(&as_slices(&noise), 0, true);
                }
            }
            span_wall_sum = wall_start.elapsed().as_secs_f64().mul_add(1000.0, span_wall_sum);
            span_brand_sum += timing.total_ns("svd_brand");
            span_naive_sum += timing.total_ns("svd_naive");
            drop(guard);
        }

        let oracle_on = span_naive_sum > 0;
        let span_wall = span_wall_sum / f64::from(n_reps);
        let span_brand = span_brand_sum as f64 / f64::from(n_reps) / 1_000_000.0;
        let span_naive = span_naive_sum as f64 / f64::from(n_reps) / 1_000_000.0;

        // ── B: WITHOUT subscriber ───────────────────────
        let mut bare_sum = 0.0_f64;

        for _ in 0..n_reps {
            let wall_start = Instant::now();
            {
                let mut t = SubspaceTracker::new(dim, &base_cfg, base_cfg.cusum_slow_decay);
                let mut rng = SmallRng::seed_from_u64(42);
                for _ in 0..rounds {
                    let noise = generate_noise(dim, batch_size, &mut rng);
                    t.observe(&as_slices(&noise), 0, true);
                }
            }
            bare_sum = wall_start.elapsed().as_secs_f64().mul_add(1000.0, bare_sum);
        }

        let bare_wall = bare_sum / f64::from(n_reps);

        eprintln!("\n  [{label}] — {rounds} rounds, d={dim}, b={batch_size}, {n_reps} reps each");
        eprintln!("  oracle active: {oracle_on}");
        eprintln!("  WITH subscriber (avg of {n_reps}):");
        eprintln!("    wall-clock:     {span_wall:.2} ms");
        eprintln!("    span(brand):    {span_brand:.2} ms");
        if oracle_on {
            eprintln!("    span(naive):    {span_naive:.2} ms");
        }
        eprintln!("  WITHOUT subscriber (avg of {n_reps}):");
        eprintln!("    wall-clock:     {bare_wall:.2} ms");
        eprintln!("  span(brand) / bare = {:.2}×", span_brand / bare_wall);
        eprintln!("  sub-wall / bare = {:.2}×", span_wall / bare_wall);
    }
}
