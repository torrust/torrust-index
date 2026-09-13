// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`rolling_mean_empty`] | convergence | Averaging an empty window yields zero rather than a division by zero, so a metric may ask for the mean of a stretch that turned out to hold nothing and still get an answer it can carry forward. |
//! | [`rolling_mean_single`] | convergence | cites (´claim:convergence:the-window-average-is-the-plain-unweighted-mean-of-what-it-covers´) |
//! | [`rolling_mean_known`] | convergence | The window average is the plain unweighted mean of the values it covers, with no decay of its own. The yardstick is deliberately unlike the baselines it measures: an instrument with a memory would judge a long-memory baseline by an equally sluggish standard. |
//! | [`rolling_mean_negative`] | convergence | cites (´claim:convergence:the-window-average-is-the-plain-unweighted-mean-of-what-it-covers´) |
//! | [`settled_all_within_tolerance`] | convergence | cites (´claim:convergence:settling-is-dated-from-the-round-after-the-last-violation´) |
//! | [`settled_never`] | convergence | A trace still violating at its final round is not settled at all, and the answer is an absence rather than a round number. Settling is a claim about the whole remainder of a run, so it cannot be asserted while the run is still moving. |
//! | [`settled_after_specific_round`] | convergence | Settling is dated from the round after the last violation, not from the first round that happened to fall inside tolerance. The search walks backwards from the end, so a trace that strays and returns is credited only from its return. |
//! | [`settled_near_zero_reference_skipped`] | convergence | An axis whose reference level is effectively zero is passed over rather than failed. Tolerance is relative, so a near-zero reference would make every deviation enormous; an axis that never activated is treated as having nothing to say instead of as permanently unconverged. |
//! | [`settled_single_trace`] | convergence | cites (´claim:convergence:settling-is-dated-from-the-round-after-the-last-violation´) |
//! | [`settled_subset_of_axes`] | convergence | Only the axes actually asked about can hold settling back, so a wildly mismatched axis outside the requested set is invisible. The axes mature at very different rates, and a caller may need to know when the ones it depends on have settled without waiting on one it does not use. |
//! | [`converged_too_few_values`] | convergence | A trace too short to hold both a window and a separate reference window yields no verdict better than its own length. With no room to compare an early stretch against a late one, the metric reports that convergence has not been demonstrated rather than guessing that it has. |
//! | [`converged_already_stable`] | convergence | A trace that never departs from its final level is converged from its first round. The reference is the trace's own tail, so this metric answers when a run reached where it ended up — not whether that destination was the right one. |
//! | [`converged_after_transient`] | convergence | A run that starts far from its eventual level is dated as converged after the transient and well before the end: the backward walk stops at the last window whose mean departed from the tail reference. Comparing windows rather than single rounds is what separates a genuine departure from ordinary jitter about a settled level. |
//! | [`converged_near_zero_reference`] | convergence | A trace whose final level is effectively zero is reported as converged from the start rather than divided by. As with the settling metric, an inactive channel is excluded from the judgement instead of poisoning it. |
//! | [`block_mean_late_near_zero`] | convergence | An axis whose late block is effectively zero yields no verdict at all rather than a ratio against nothing. An axis that never activated has no steady state to be stationary about, and saying so is more honest than reporting an enormous relative error. |
//! | [`block_mean_identical_blocks`] | convergence | Stationarity is measured as the relative gap between an early block mean and a late one, so two blocks drawn from the same settled stretch differ by nothing. Averaging over blocks is what lets the test tell a baseline still moving from one merely jittering in place. |
//! | [`block_mean_known_error`] | convergence | cites (´claim:convergence:stationarity-is-the-relative-gap-between-an-early-block-mean-and-a-late-one´) |
//! | [`trailing_cv_too_few`] | convergence | Asking for the jitter of a window longer than the trace yields not a number, rather than a figure computed from whatever happened to be available. A short trace is not a quiet one, and the metric refuses to let the two be confused. |
//! | [`trailing_cv_constant`] | convergence | Jitter is reported as a fraction of the level it sits on, so a flat tail has none whatever that level happens to be. Normalising by the mean is what makes the figure comparable across axes whose scores differ by orders of magnitude. |
//! | [`trailing_cv_known`] | convergence | cites (´claim:convergence:jitter-is-reported-as-a-fraction-of-the-level-it-sits-on´) |
//! | [`generate_noise_shape_and_values`] | noise | Injected noise arrives in exactly the shape the tracker expects, every entry plus or minus a half. Synthetic warm-up traffic therefore carries the same centring the real encoding produces, so a model warmed on noise is warmed on the same kind of thing it will later be asked to judge. |
//! | [`as_slices_preserves_data`] | noise | Handing a generated batch to the tracker borrows it rather than transforming it: the same values arrive in the same order. Nothing is rescaled on the way in, so what a run does is attributable to the noise that was generated for it. |
//! | [`cfg_test_is_valid`] | convergence | The configuration convergence is measured under passes the same validation any production configuration must. Results measured here are therefore statements about a legal sentinel rather than about a corner of the parameter space the crate would refuse to construct. |
//! | [`cfg_b16_overrides_batch_size`] | convergence | The larger-batch variant differs from the standard one in batch size alone and is likewise valid, so a comparison between the two isolates the effect of batch size. Convergence claims made at two batch sizes are then about one system observed differently, not about two systems. |
//! | [`cfg_production_overrides`] | convergence | The production-like variant changes three things together — a longer memory, larger batches and a slower rank cadence — and remains valid. These are the settings the shipped noise schedule is sized from, so they are exercised as a set rather than one at a time. |

//! Shared ground for the convergence tests: the configurations they run
//! under, the synthetic traffic they run on, and the metrics by which a
//! run is judged to have settled. The traffic is centred half-magnitude
//! noise — the same shape and centring the real encoding produces — so a
//! model warmed here is warmed on the kind of thing it will later judge.
//!
//! The metrics exist because "converged" is not obvious for an
//! exponentially-weighted estimator. Such an estimator never stops moving:
//! its output keeps jittering around the correct level for as long as the
//! input is stochastic, and how much it jitters differs by more than a
//! hundredfold across the four scoring axes. A criterion that waits for
//! the movement to stop would therefore never fire on the noisiest axis,
//! however correct that axis had become. Two answers live here. The
//! settling metric walks backwards from the end of a trace and dates
//! settling from the round after the last violation, so a stray excursion
//! cannot be forgiven by later good behaviour. The block metrics compare
//! an average over an early stretch against an average over a late one,
//! which averages the irreducible jitter down far enough that a baseline
//! still moving can be told from one merely fluctuating in place.
//!
//! Both families decline to judge a channel whose reference level is
//! effectively zero, rather than reporting it as broken. Tolerances are
//! relative, so they are meaningless there, and an axis that never
//! activated has no steady state it could have reached.
//!
//! The configurations come as a small family — a fast standard one, the
//! same at a larger batch size, and one resembling production's longer
//! memory — so that a convergence bound can be established at more than
//! one point and the effect of each setting seen separately. Each is
//! validated in its own right, because a bound measured under a
//! configuration the crate would refuse to build would be worth nothing.
//!
//! # §-references
//!
//! - §ALGO S-4.2 Phase 3 — Latent distribution cold→warm
//! - §ALGO S-7.1.1 — EWMA outlier filter / clipping ceiling
//! - §ALGO S-11.5 — Maturity tracking (noise influence η)
//! - ADR-S-013 — Warm-up convergence benchmark

use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

use crate::config::SentinelConfig;
use crate::sentinel::tracker::SubspaceTracker;

// ════════════════════════════════════════════════════════════
//  Configs
// ════════════════════════════════════════════════════════════

/// Standard test config (λ = 0.95, b = 4).
pub(super) fn cfg_test() -> SentinelConfig<u64> {
    SentinelConfig {
        max_rank: 2,
        forgetting_factor: 0.95,
        rank_update_interval: 5,
        analysis_k: 16,
        analysis_depth_cutoff: 6,
        energy_threshold: 0.90,
        eps: 1e-6,
        per_sample_scores: false,
        cusum_allowance_sigmas: 0.5,
        cusum_slow_decay: 0.999,
        cusum_coord_slow_decay: 0.999,
        clip_sigmas: 3.0,
        clip_pressure_decay: 0.95,
        split_threshold: 100,
        d_create: 3,
        d_evict: 6,
        budget: 100_000,
        noise_schedule: crate::config::NoiseSchedule::Explicit(vec![5]),
        noise_batch_size: 4,
        noise_seed: Some(42),
        background_warming: false,
        svd_strategy: crate::maths::SvdStrategy::Brand,
    }
}

/// Test config with larger batch size (λ = 0.95, b = 16).
pub(super) fn cfg_b16() -> SentinelConfig<u64> {
    SentinelConfig {
        noise_batch_size: 16,
        ..cfg_test()
    }
}

/// Production-like config (λ = 0.99, b = 16).
pub(super) fn cfg_production() -> SentinelConfig<u64> {
    SentinelConfig {
        forgetting_factor: 0.99,
        rank_update_interval: 100,
        noise_batch_size: 16,
        noise_schedule: crate::config::NoiseSchedule::Explicit(vec![50]),
        ..cfg_test()
    }
}

// ════════════════════════════════════════════════════════════
//  Noise generation
// ════════════════════════════════════════════════════════════

/// Generate a batch of random ±0.5 noise vectors.
pub(super) fn generate_noise(dim: usize, batch_size: usize, rng: &mut SmallRng) -> Vec<Vec<f64>> {
    (0..batch_size)
        .map(|_| (0..dim).map(|_| if rng.random_bool(0.5) { 0.5 } else { -0.5 }).collect())
        .collect()
}

/// Convert owned vectors to a slice-of-slices for `tracker.observe()`.
pub(super) fn as_slices(vecs: &[Vec<f64>]) -> Vec<&[f64]> {
    vecs.iter().map(Vec::as_slice).collect()
}

// ════════════════════════════════════════════════════════════
//  Axis labels
// ════════════════════════════════════════════════════════════

pub(super) const AXIS_NAMES: [&str; 4] = ["novelty", "displacement", "surprise", "coherence"];

// ════════════════════════════════════════════════════════════
//  Convergence metrics
// ════════════════════════════════════════════════════════════

/// Multi-axis backward-walk convergence: finds the first round from
/// which ALL of the first `axes` channels stay within `tolerance`
/// of `reference` permanently.
pub(super) fn find_settled_round(traces: &[[f64; 4]], reference: &[f64; 4], tolerance: f64, axes: usize) -> Option<usize> {
    let mut last_violation = None;
    for (i, means) in traces.iter().enumerate().rev() {
        let all_ok = (0..axes).all(|a| {
            let ref_val = reference[a];
            if ref_val.abs() < 1e-12 {
                true
            } else {
                (means[a] - ref_val).abs() < tolerance * ref_val.abs()
            }
        });
        if !all_ok {
            last_violation = Some(i);
            break;
        }
    }
    match last_violation {
        Some(v) if v + 1 < traces.len() => Some(v + 1),
        None => Some(0),
        _ => None,
    }
}

/// Windowed-mean convergence metric (ADR-S-013 §3a).
///
/// Compares a rolling mean of a window-sized block against the
/// reference (rolling mean of the final `window` values). Returns
/// the first round where the metric permanently stays within
/// `tolerance` of the reference.
pub(super) fn find_converged_round(baselines: &[f64], window: usize, tolerance: f64) -> usize {
    let n = baselines.len();
    if n < 2 * window {
        return n;
    }
    let reference = rolling_mean(&baselines[n - window..]);
    if reference.abs() < 1e-12 {
        return 0;
    }
    for i in (window..n - window).rev() {
        let local = rolling_mean(&baselines[i.saturating_sub(window)..i]);
        if ((local - reference) / reference).abs() > tolerance {
            return i + 1;
        }
    }
    0
}

/// Rolling mean of a slice.
#[allow(clippy::cast_precision_loss)]
pub(super) fn rolling_mean(slice: &[f64]) -> f64 {
    if slice.is_empty() {
        return 0.0;
    }
    slice.iter().sum::<f64>() / slice.len() as f64
}

/// Block-mean stationarity test for a single axis.
///
/// Compares the mean of traces in `[early_start, early_end)` against
/// `[late_start, late_end)`.  Returns `Some(relative_error)` if the
/// late-block mean is non-negligible, or `None` if the late-block mean
/// is effectively zero (axis inactive).
///
/// The theoretical basis (§`noise_convergence.md` §1–§2): after the EWMA
/// transient decays (λ^t < ε), the expected baseline mean equals the
/// score distribution's expectation.  Two widely-separated block means
/// from the same stationary process should agree within the statistical
/// uncertainty of the block-mean estimator, which is of order
/// `CV_EWMA × √((1+λ)/(1−λ) / block_len)`.  The per-axis tolerances
/// passed by callers are set to ≥ 3× this quantity.
pub(super) fn block_mean_relative_error(
    traces: &[[f64; 4]],
    axis: usize,
    early_start: usize,
    early_end: usize,
    late_start: usize,
    late_end: usize,
) -> Option<f64> {
    let early_mean = rolling_mean(&traces[early_start..early_end].iter().map(|t| t[axis]).collect::<Vec<_>>());
    let late_mean = rolling_mean(&traces[late_start..late_end].iter().map(|t| t[axis]).collect::<Vec<_>>());
    if late_mean.abs() < 1e-12 {
        return None; // axis inactive
    }
    Some((early_mean - late_mean).abs() / late_mean.abs())
}

/// Coefficient of variation of the last `window` values.
#[allow(clippy::cast_precision_loss)]
pub(super) fn trailing_cv(baselines: &[f64], window: usize) -> f64 {
    let n = baselines.len();
    if n < window {
        return f64::NAN;
    }
    let tail = &baselines[n - window..];
    let mean = rolling_mean(tail);
    if mean.abs() < 1e-12 {
        return 0.0;
    }
    let var = tail.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / tail.len() as f64;
    var.sqrt() / mean
}

// ════════════════════════════════════════════════════════════
//  Trace collection
// ════════════════════════════════════════════════════════════

/// Per-round snapshot of baseline means, score means, and CUSUM state.
#[derive(Clone)]
#[allow(dead_code)] // Fields are for convergence test consumers.
pub(super) struct RoundTrace {
    pub baseline_means: [f64; 4],
    pub score_means: [f64; 4],
    pub cusum_accumulators: [f64; 4],
    pub slow_baseline_means: [f64; 4],
    pub noise_influence: f64,
}

/// Run `total_rounds` of noise injection and collect per-round traces.
pub(super) fn run_noise_trace(cfg: &SentinelConfig<u64>, dim: usize, total_rounds: usize, seed: u64) -> Vec<RoundTrace> {
    let mut tracker = SubspaceTracker::new(dim, cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(seed);
    let mut traces = Vec::with_capacity(total_rounds);

    for _ in 0..total_rounds {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        let report = tracker.observe(&as_slices(&noise), 0, true);

        traces.push(RoundTrace {
            baseline_means: [
                report.scores.novelty.baseline.mean,
                report.scores.displacement.baseline.mean,
                report.scores.surprise.baseline.mean,
                report.scores.coherence.baseline.mean,
            ],
            score_means: [
                report.scores.novelty.mean,
                report.scores.displacement.mean,
                report.scores.surprise.mean,
                report.scores.coherence.mean,
            ],
            cusum_accumulators: [
                report.scores.novelty.cusum.accumulator,
                report.scores.displacement.cusum.accumulator,
                report.scores.surprise.cusum.accumulator,
                report.scores.coherence.cusum.accumulator,
            ],
            slow_baseline_means: [
                report.scores.novelty.cusum.slow_baseline.mean,
                report.scores.displacement.cusum.slow_baseline.mean,
                report.scores.surprise.cusum.slow_baseline.mean,
                report.scores.coherence.cusum.slow_baseline.mean,
            ],
            noise_influence: report.maturity.noise_influence,
        });
    }

    traces
}

// ════════════════════════════════════════════════════════════
//  Unit tests
// ════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    // ── rolling_mean ────────────────────────────────────────

    /// Averaging an empty window yields zero rather than a division by zero, so
    /// a metric may ask for the mean of a stretch that turned out to hold
    /// nothing and still get an answer it can carry forward.
    ///
    /// ´claim:convergence:an-empty-window-averages-to-zero-rather-than-failing´
    /// ´test:crate:rolling-mean-empty´
    #[test]
    fn rolling_mean_empty() {
        assert!(rolling_mean(&[]).abs() < f64::EPSILON);
    }

    /// A window of one is that one value, since averaging cannot smooth what it
    /// has seen only once. This pins the degenerate end of the same rule the
    /// multi-value case fixes.
    ///
    /// (´claim:convergence:the-window-average-is-the-plain-unweighted-mean-of-what-it-covers´)
    /// ´test:crate:rolling-mean-single´
    #[test]
    fn rolling_mean_single() {
        assert!((rolling_mean(&[7.0]) - 7.0).abs() < 1e-15);
    }

    /// The window average is the plain unweighted mean of the values it covers,
    /// with no decay of its own. The yardstick is deliberately unlike the
    /// baselines it measures: an instrument with a memory would judge a
    /// long-memory baseline by an equally sluggish standard.
    ///
    /// ´claim:convergence:the-window-average-is-the-plain-unweighted-mean-of-what-it-covers´
    /// ´test:crate:rolling-mean-known´
    #[test]
    fn rolling_mean_known() {
        // (1 + 2 + 3 + 4) / 4 = 2.5
        assert!((rolling_mean(&[1.0, 2.0, 3.0, 4.0]) - 2.5).abs() < 1e-15);
    }

    /// Values below zero are averaged as they stand rather than by magnitude,
    /// so a mean can itself come out negative. Convergence metrics compare
    /// signed levels, and folding the sign away would make a trace that
    /// overshoots look like one that undershoots.
    ///
    /// (´claim:convergence:the-window-average-is-the-plain-unweighted-mean-of-what-it-covers´)
    /// ´test:crate:rolling-mean-negative´
    #[test]
    fn rolling_mean_negative() {
        // (-3 + 1) / 2 = -1.0
        assert!((rolling_mean(&[-3.0, 1.0]) - (-1.0)).abs() < 1e-15);
    }

    // ── find_settled_round ──────────────────────────────────

    /// A trace that never leaves tolerance has no last violation at all, so it
    /// counts as settled from its very first round.
    ///
    /// (´claim:convergence:settling-is-dated-from-the-round-after-the-last-violation´)
    /// ´test:crate:settled-all-within-tolerance´
    #[test]
    fn settled_all_within_tolerance() {
        // All rounds identical to reference → settled from round 0.
        let traces = vec![[1.0, 2.0, 3.0, 4.0]; 10];
        let reference = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(find_settled_round(&traces, &reference, 0.05, 4), Some(0));
    }

    /// A trace still violating at its final round is not settled at all, and
    /// the answer is an absence rather than a round number. Settling is a claim
    /// about the whole remainder of a run, so it cannot be asserted while the
    /// run is still moving.
    ///
    /// ´claim:convergence:a-trace-still-violating-at-its-last-round-is-never-settled´
    /// ´test:crate:settled-never´
    #[test]
    fn settled_never() {
        // Last round is a violation → cannot settle.
        let mut traces = vec![[1.0, 2.0, 3.0, 4.0]; 10];
        traces[9] = [100.0, 2.0, 3.0, 4.0]; // violates axis 0
        assert_eq!(find_settled_round(&traces, &[1.0, 2.0, 3.0, 4.0], 0.05, 4), None);
    }

    /// Settling is dated from the round after the last violation, not from the
    /// first round that happened to fall inside tolerance. The search walks
    /// backwards from the end, so a trace that strays and returns is credited
    /// only from its return.
    ///
    /// ´claim:convergence:settling-is-dated-from-the-round-after-the-last-violation´
    /// ´test:crate:settled-after-specific-round´
    #[test]
    fn settled_after_specific_round() {
        // Violation at round 3, then stable from round 4 onward.
        let mut traces = vec![[1.0, 2.0, 3.0, 4.0]; 10];
        traces[3] = [100.0, 2.0, 3.0, 4.0];
        assert_eq!(find_settled_round(&traces, &[1.0, 2.0, 3.0, 4.0], 0.05, 4), Some(4));
    }

    /// An axis whose reference level is effectively zero is passed over rather
    /// than failed. Tolerance is relative, so a near-zero reference would make
    /// every deviation enormous; an axis that never activated is treated as
    /// having nothing to say instead of as permanently unconverged.
    ///
    /// ´claim:convergence:an-axis-with-a-near-zero-reference-is-passed-over-rather-than-failed´
    /// ´test:crate:settled-near-zero-reference-skipped´
    #[test]
    fn settled_near_zero_reference_skipped() {
        // Reference near zero → axis is always OK (skipped).
        let traces = vec![[999.0, 0.0, 0.0, 0.0]; 5];
        let reference = [999.0, 0.0, 0.0, 0.0];
        assert_eq!(find_settled_round(&traces, &reference, 0.01, 4), Some(0));
    }

    /// A trace of a single round is settled when that round is within
    /// tolerance: the backward walk finds nothing, so the degenerate case falls
    /// out of the same rule rather than needing one of its own.
    ///
    /// (´claim:convergence:settling-is-dated-from-the-round-after-the-last-violation´)
    /// ´test:crate:settled-single-trace´
    #[test]
    fn settled_single_trace() {
        let traces = vec![[1.0, 2.0, 3.0, 4.0]];
        let reference = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(find_settled_round(&traces, &reference, 0.05, 4), Some(0));
    }

    /// Only the axes actually asked about can hold settling back, so a wildly
    /// mismatched axis outside the requested set is invisible. The axes mature
    /// at very different rates, and a caller may need to know when the ones it
    /// depends on have settled without waiting on one it does not use.
    ///
    /// ´claim:convergence:only-the-axes-asked-about-can-hold-settling-back´
    /// ´test:crate:settled-subset-of-axes´
    #[test]
    fn settled_subset_of_axes() {
        // Only check first 2 axes; axis 2 huge mismatch is ignored.
        let traces = vec![[1.0, 2.0, 999.0, 999.0]; 5];
        let reference = [1.0, 2.0, 3.0, 4.0];
        assert_eq!(find_settled_round(&traces, &reference, 0.05, 2), Some(0));
    }

    // ── find_converged_round ────────────────────────────────

    /// A trace too short to hold both a window and a separate reference window
    /// yields no verdict better than its own length. With no room to compare an
    /// early stretch against a late one, the metric reports that convergence
    /// has not been demonstrated rather than guessing that it has.
    ///
    /// ´claim:convergence:a-trace-too-short-for-two-windows-is-reported-as-not-yet-converged´
    /// ´test:crate:converged-too-few-values´
    #[test]
    fn converged_too_few_values() {
        // n < 2*window → returns n.
        let data = vec![1.0; 5];
        assert_eq!(find_converged_round(&data, 10, 0.05), 5);
    }

    /// A trace that never departs from its final level is converged from its
    /// first round. The reference is the trace's own tail, so this metric
    /// answers when a run reached where it ended up — not whether that
    /// destination was the right one.
    ///
    /// ´claim:convergence:convergence-is-measured-against-the-traces-own-tail´
    /// ´test:crate:converged-already-stable´
    #[test]
    fn converged_already_stable() {
        // Constant input → converged from round 0.
        let data = vec![3.0; 100];
        assert_eq!(find_converged_round(&data, 10, 0.05), 0);
    }

    /// A run that starts far from its eventual level is dated as converged
    /// after the transient and well before the end: the backward walk stops at
    /// the last window whose mean departed from the tail reference. Comparing
    /// windows rather than single rounds is what separates a genuine departure
    /// from ordinary jitter about a settled level.
    ///
    /// ´claim:convergence:the-converged-round-is-the-one-after-the-last-window-that-departed-from-the-final-level´
    /// ´test:crate:converged-after-transient´
    #[test]
    fn converged_after_transient() {
        // Big values then settling to 1.0 — must converge after transient.
        let mut data = vec![100.0; 20];
        data.extend(vec![1.0; 80]);
        let round = find_converged_round(&data, 10, 0.05);
        // Must be after the transient region but before the end.
        assert!(round > 10, "should detect transient, got {round}");
        assert!(round < 50, "should converge well before end, got {round}");
    }

    /// A trace whose final level is effectively zero is reported as converged
    /// from the start rather than divided by. As with the settling metric, an
    /// inactive channel is excluded from the judgement instead of poisoning it.
    ///
    /// ´claim:convergence:a-trace-with-a-near-zero-final-level-is-treated-as-converged-rather-than-divided-by´
    /// ´test:crate:converged-near-zero-reference´
    #[test]
    fn converged_near_zero_reference() {
        // Near-zero final mean → returns 0.
        let data = vec![0.0; 40];
        assert_eq!(find_converged_round(&data, 10, 0.05), 0);
    }

    // ── block_mean_relative_error ───────────────────────────

    /// An axis whose late block is effectively zero yields no verdict at all
    /// rather than a ratio against nothing. An axis that never activated has no
    /// steady state to be stationary about, and saying so is more honest than
    /// reporting an enormous relative error.
    ///
    /// ´claim:convergence:an-axis-with-no-late-signal-yields-no-stationarity-verdict-at-all´
    /// ´test:crate:block-mean-late-near-zero´
    #[test]
    fn block_mean_late_near_zero() {
        // Late block mean near zero → axis inactive → None.
        let traces = vec![[1.0, 0.0, 0.0, 0.0]; 20];
        assert!(block_mean_relative_error(&traces, 1, 0, 10, 10, 20).is_none());
    }

    /// Stationarity is measured as the relative gap between an early block mean
    /// and a late one, so two blocks drawn from the same settled stretch differ
    /// by nothing. Averaging over blocks is what lets the test tell a baseline
    /// still moving from one merely jittering in place.
    ///
    /// ´claim:convergence:stationarity-is-the-relative-gap-between-an-early-block-mean-and-a-late-one´
    /// ´test:crate:block-mean-identical-blocks´
    #[test]
    fn block_mean_identical_blocks() {
        // Identical blocks → error is 0.0.
        let traces = vec![[5.0, 5.0, 5.0, 5.0]; 20];
        let err = block_mean_relative_error(&traces, 0, 0, 10, 10, 20).unwrap();
        assert!(err < 1e-15, "expected ~0, got {err}");
    }

    /// The gap is normalised by the late block — the one taken as the reference
    /// — so an early level at half the late one reports as a half. This pins
    /// the scale of the figure that the per-axis tolerances are set against.
    ///
    /// (´claim:convergence:stationarity-is-the-relative-gap-between-an-early-block-mean-and-a-late-one´)
    /// ´test:crate:block-mean-known-error´
    #[test]
    fn block_mean_known_error() {
        // Early block mean = 2.0, late block mean = 4.0 → error = 0.5.
        let mut traces = Vec::new();
        for _ in 0..10 {
            traces.push([2.0, 0.0, 0.0, 0.0]);
        }
        for _ in 0..10 {
            traces.push([4.0, 0.0, 0.0, 0.0]);
        }
        let err = block_mean_relative_error(&traces, 0, 0, 10, 10, 20).unwrap();
        assert!((err - 0.5).abs() < 1e-15, "expected 0.5, got {err}");
    }

    // ── trailing_cv ─────────────────────────────────────────

    /// Asking for the jitter of a window longer than the trace yields not a
    /// number, rather than a figure computed from whatever happened to be
    /// available. A short trace is not a quiet one, and the metric refuses to
    /// let the two be confused.
    ///
    /// ´claim:convergence:a-window-longer-than-the-trace-yields-no-jitter-figure-at-all´
    /// ´test:crate:trailing-cv-too-few´
    #[test]
    fn trailing_cv_too_few() {
        let data = vec![1.0; 3];
        assert!(trailing_cv(&data, 10).is_nan());
    }

    /// Jitter is reported as a fraction of the level it sits on, so a flat tail
    /// has none whatever that level happens to be. Normalising by the mean is
    /// what makes the figure comparable across axes whose scores differ by
    /// orders of magnitude.
    ///
    /// ´claim:convergence:jitter-is-reported-as-a-fraction-of-the-level-it-sits-on´
    /// ´test:crate:trailing-cv-constant´
    #[test]
    fn trailing_cv_constant() {
        let data = vec![5.0; 20];
        assert!((trailing_cv(&data, 10)).abs() < 1e-15);
    }

    /// A spread measured against a mean several times larger reports as that
    /// ratio and not as the deviation itself, which pins the normalisation the
    /// statement asserts.
    ///
    /// (´claim:convergence:jitter-is-reported-as-a-fraction-of-the-level-it-sits-on´)
    /// ´test:crate:trailing-cv-known´
    #[test]
    fn trailing_cv_known() {
        // std([1,2,3,4,5]) / mean([1,2,3,4,5])
        // mean = 3.0, var = (4+1+0+1+4)/5 = 2.0, std = √2
        // CV = √2 / 3 ≈ 0.4714
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let cv = trailing_cv(&data, 5);
        let expected = 2.0_f64.sqrt() / 3.0;
        assert!((cv - expected).abs() < 1e-10, "expected {expected}, got {cv}");
    }

    // ── generate_noise / as_slices ──────────────────────────

    /// Injected noise arrives in exactly the shape the tracker expects, every
    /// entry plus or minus a half. Synthetic warm-up traffic therefore carries
    /// the same centring the real encoding produces, so a model warmed on noise
    /// is warmed on the same kind of thing it will later be asked to judge.
    ///
    /// ´claim:noise:injected-noise-is-shaped-and-centred-exactly-like-real-observations´
    /// ´test:crate:generate-noise-shape-and-values´
    #[test]
    fn generate_noise_shape_and_values() {
        let mut rng = SmallRng::seed_from_u64(42);
        let noise = generate_noise(8, 5, &mut rng);
        assert_eq!(noise.len(), 5);
        for row in &noise {
            assert_eq!(row.len(), 8);
            for &v in row {
                assert!(
                    (v - 0.5).abs() < f64::EPSILON || (v + 0.5).abs() < f64::EPSILON,
                    "unexpected value {v}"
                );
            }
        }
    }

    /// Handing a generated batch to the tracker borrows it rather than
    /// transforming it: the same values arrive in the same order. Nothing is
    /// rescaled on the way in, so what a run does is attributable to the noise
    /// that was generated for it.
    ///
    /// ´claim:noise:handing-a-generated-batch-to-the-tracker-borrows-it-without-altering-it´
    /// ´test:crate:as-slices-preserves-data´
    #[test]
    fn as_slices_preserves_data() {
        let vecs = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
        let slices = as_slices(&vecs);
        assert_eq!(slices.len(), 2);
        assert_eq!(slices[0], &[1.0, 2.0]);
        assert_eq!(slices[1], &[3.0, 4.0]);
    }

    // ── config constructors ─────────────────────────────────

    /// The configuration convergence is measured under passes the same
    /// validation any production configuration must. Results measured here are
    /// therefore statements about a legal sentinel rather than about a corner
    /// of the parameter space the crate would refuse to construct.
    ///
    /// ´claim:convergence:the-configuration-convergence-is-measured-under-is-one-the-crate-would-accept´
    /// ´test:crate:cfg-test-is-valid´
    #[test]
    fn cfg_test_is_valid() {
        let cfg = cfg_test();
        assert!(cfg.validate().is_ok(), "cfg_test() must pass validation");
    }

    /// The larger-batch variant differs from the standard one in batch size
    /// alone and is likewise valid, so a comparison between the two isolates
    /// the effect of batch size. Convergence claims made at two batch sizes are
    /// then about one system observed differently, not about two systems.
    ///
    /// ´claim:convergence:the-larger-batch-variant-differs-in-batch-size-alone´
    /// ´test:crate:cfg-b16-overrides-batch-size´
    #[test]
    fn cfg_b16_overrides_batch_size() {
        let cfg = cfg_b16();
        assert_eq!(cfg.noise_batch_size, 16);
        assert!(cfg.validate().is_ok(), "cfg_b16() must pass validation");
    }

    /// The production-like variant changes three things together — a longer
    /// memory, larger batches and a slower rank cadence — and remains valid.
    /// These are the settings the shipped noise schedule is sized from, so they
    /// are exercised as a set rather than one at a time.
    ///
    /// ´claim:convergence:the-production-like-variant-changes-memory-batch-size-and-rank-cadence-together´
    /// ´test:crate:cfg-production-overrides´
    #[test]
    fn cfg_production_overrides() {
        let cfg = cfg_production();
        assert!((cfg.forgetting_factor - 0.99).abs() < 1e-15);
        assert_eq!(cfg.noise_batch_size, 16);
        assert_eq!(cfg.rank_update_interval, 100);
        assert!(cfg.validate().is_ok(), "cfg_production() must pass validation");
    }
}
