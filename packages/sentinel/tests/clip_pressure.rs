// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`fresh_sentinel_has_zero_clip_pressure`] | pressure | A sentinel that has been shown nothing reports no rejection at all — not a nominal starting level, but zero across the smallest, largest and average axis alike. Rejection is a measurement of traffic, so with no traffic there is nothing to measure, and the ceiling starts as wide as it can be rather than pre-loaded against the first arrivals. |
//! | [`clip_pressure_distribution_min_le_mean_le_max`] | pressure | The summary the sentinel publishes is a genuine summary of the per-axis values behind it: after a long clean run followed by a contaminated burst, where the axes have been driven apart, the smallest value still sits at or below the average and the average at or below the largest. An operator watching only the peak can therefore trust that no axis is being rejected harder than the number they are watching. |
//! | [`per_axis_clip_pressure_in_cell_reports`] | pressure | Rejection is visible at the grain it actually happens: every batch report carries a separate, finite, non-negative figure for each of the four scoring axes of every cell reported. Each axis keeps its own ceiling and its own running rejection rate, so a caller can see which axis is under strain rather than only that something is. |
//! | [`clip_pressure_stable_under_clean_traffic`] | pressure | Traffic that keeps its shape does not ratchet the rejection rate upward. Over a long clean run the second stretch of batches sits no higher than the first, and the average across axes stays well under half. A modest steady rejection is expected — a ceiling a few deviations out will always trim the occasional sample — but it settles at a level rather than climbing, which is what lets a rise be read as news. |
//! | [`contamination_elevates_clip_pressure`] | pressure | Mixing a substantial minority of structurally novel values into an otherwise settled stream drives the peak rejection rate clearly above where clean traffic had left it. The rate is thus an observable symptom of contamination in its own right: the ceiling is doing its job of keeping those samples out of the baseline, and the pressure is the sentinel saying how hard it is having to work at it. |
//! | [`clip_pressure_decays_after_contamination`] | pressure | When the contamination stops, the rejection rate comes back down: after a long stretch of clean traffic the peak sits below where the contaminated phase left it. The measure is a rolling one with a finite memory, so an episode ages out instead of marking the sentinel permanently — a system that never forgot would treat one past attack as grounds for a forever-loose ceiling. |
//! | [`effective_ceiling_widens_under_pressure`] | pressure | The rejection rate is not merely reported, it feeds back into the ceiling. Recomputing the documented widening rule from the pressure a contaminated run actually leaves behind gives a ceiling meaningfully above the configured one. Refusal is therefore self-limiting by construction: the harder an axis has been rejecting, the more room it grants itself, so a genuine and lasting shift in the traffic can eventually be learned rather than clipped away for ever. |
//! | [`faster_decay_recovers_sooner`] | pressure | How long the rejection rate remembers is a configured choice with an observable consequence: given identical contamination and an identical recovery window, the sentinel with the shorter memory ends up at a lower pressure than the one with the longer. The setting therefore trades how quickly a ceiling snaps back after an episode against how steadily it holds through a noisy one. |
//! | [`warm_up_completion_resets_clip_pressure`] | pressure | Rejection accrued while a tracker still leaned on synthetic history is discarded at the moment that reliance falls away: shortly after the crossing, the peak rate across the sentinel is still low, having had only a handful of real batches in which to rebuild. Whatever the warming rounds caused the ceiling to refuse is therefore not allowed to widen the ceiling that production traffic will be judged against. |

//! Integration tests for **clip pressure** — the running measure of how much of
//! the incoming traffic each scoring axis is currently refusing to learn from.
//!
//! A baseline that learned from every sample would be led anywhere an attacker
//! wanted it to go, so each axis keeps a ceiling and drops the samples above it
//! before updating. Only the upper tail is filtered: anomaly scores are
//! non-negative and right-skewed, an attacker inflates them and never deflates
//! them, and a lower bound would throw away the honestly low scores of a quiet
//! period. Clip pressure is the fraction of samples that ceiling rejected,
//! smoothed over recent batches, and it is reported per axis and summarised
//! across the whole sentinel.
//!
//! The quantity exists because refusing to learn is only ever a temporary
//! answer. Persistent rejection means the traffic has genuinely moved and the
//! ceiling is now in the wrong place; so the ceiling is widened in proportion to
//! the pressure, which lets the sentinel eventually follow a real shift instead
//! of scoring forever against a world that no longer exists. The smoothing
//! constant sets how long that memory is, and rejection accumulated while a
//! tracker was still being warmed is discarded outright when warming ends, so
//! synthetic history cannot loosen the ceiling that real traffic will be judged
//! against.
//!
//! Most tests here start from a cold configuration, with no warming rounds, so
//! that the baselines they exercise were built from the score distributions the
//! test itself fed in; the one test concerned with the warming handover is the
//! exception.

mod common;

use common::{cell_values, cold_config, test_config};
use torrust_sentinel::{Sentinel128, SentinelConfig};

// ═══════════════════════════════════════════════════════════
//  Helpers
// ═══════════════════════════════════════════════════════════

/// Maximum `clip_pressure` from a health report across all
/// active tracker axes.
fn max_clip_pressure(s: &Sentinel128) -> f64 {
    s.health().clip_pressure_distribution.max
}

/// Mean `clip_pressure` from a health report.
fn mean_clip_pressure(s: &Sentinel128) -> f64 {
    s.health().clip_pressure_distribution.mean
}

/// Generate values routed to the same cell (leading nibble `nibble`)
/// but with moderate structural variety in the middle bits.
///
/// Unlike [`cell_values()`] (sequential low bits only), this injects
/// variation across a wider bit range, producing per-batch score
/// variance that the EWMA can track meaningfully.
fn diverse_cell_values(nibble: u128, count: usize, batch_id: usize) -> Vec<u128> {
    (0..count)
        .map(|i| {
            // Vary bits 16–31 based on batch_id, and bits 0–15 based on i.
            let mid = ((batch_id as u128 * 7 + 3) % 0xFFFF) << 16;
            let low = (i as u128) | ((i as u128 * 13 + batch_id as u128) % 0xFFFF);
            (nibble << 124) | mid | low
        })
        .collect()
}

/// Generate contaminated values: same leading nibble for correct
/// routing, but with a dense middle-bit block that produces elevated
/// anomaly scores (structurally novel relative to [`diverse_cell_values()`]).
fn contaminated_values(nibble: u128, count: usize, offset: usize) -> Vec<u128> {
    (0..count)
        .map(|i| {
            // Set bits 32–63, creating structural novelty relative to
            // the normal diverse_cell_values pattern (which only varies 0–31).
            let dense = 0x0000_0000_FFFF_FFFF_0000_0000_0000_0000_u128;
            (nibble << 124) | dense | ((offset + i) as u128)
        })
        .collect()
}

/// Common cold-start config for clip-pressure tests: root-only cell,
/// deterministic decay.
fn clip_cold_config(decay: f64) -> SentinelConfig<u64> {
    SentinelConfig::<u64> {
        clip_pressure_decay: decay,
        clip_sigmas: 3.0,
        split_threshold: 100_000,
        ..cold_config()
    }
}

/// Warm up a sentinel with `n` batches of diverse clean traffic on
/// the given nibble, returning the sentinel.
fn warmed_sentinel(cfg: SentinelConfig<u64>, nibble: u128, batches: usize) -> Sentinel128 {
    let mut s = Sentinel128::new(cfg).unwrap();
    for batch_id in 0..batches {
        s.ingest(&diverse_cell_values(nibble, 16, batch_id));
    }
    s
}

// ═══════════════════════════════════════════════════════════
//  Initial state
// ═══════════════════════════════════════════════════════════

/// A sentinel that has been shown nothing reports no rejection at all — not a
/// nominal starting level, but zero across the smallest, largest and average
/// axis alike. Rejection is a measurement of traffic, so with no traffic there
/// is nothing to measure, and the ceiling starts as wide as it can be rather
/// than pre-loaded against the first arrivals.
///
/// ´claim:pressure:a-sentinel-that-has-seen-no-traffic-reports-no-rejection-at-all´
/// ´test:integration:fresh-sentinel-has-zero-clip-pressure´
#[test]
fn fresh_sentinel_has_zero_clip_pressure() {
    let s = Sentinel128::new(cold_config()).unwrap();
    let cp = s.health().clip_pressure_distribution;

    assert!(
        cp.min == 0.0 && cp.max == 0.0 && cp.mean == 0.0,
        "fresh sentinel should have all-zero clip_pressure, got min={}, max={}, mean={}",
        cp.min,
        cp.max,
        cp.mean,
    );
}

// ═══════════════════════════════════════════════════════════
//  Distribution invariants
// ═══════════════════════════════════════════════════════════

/// The summary the sentinel publishes is a genuine summary of the per-axis
/// values behind it: after a long clean run followed by a contaminated burst,
/// where the axes have been driven apart, the smallest value still sits at or
/// below the average and the average at or below the largest. An operator
/// watching only the peak can therefore trust that no axis is being rejected
/// harder than the number they are watching.
///
/// ´claim:pressure:the-published-summary-brackets-its-own-average-between-the-least-and-most-pressed-axis´
/// ´test:integration:clip-pressure-distribution-min-le-mean-le-max´
#[test]
fn clip_pressure_distribution_min_le_mean_le_max() {
    // Ingest enough traffic that clip_pressure is non-trivially
    // exercised, then verify the distribution ordering invariant.
    let mut s = warmed_sentinel(clip_cold_config(0.95), 0xA, 50);

    // Inject a burst of contamination so min ≠ max is more likely
    // when multiple axes are active.
    for batch_id in 50..60 {
        let mut batch = diverse_cell_values(0xA, 10, batch_id);
        batch.extend(contaminated_values(0xA, 6, batch_id));
        s.ingest(&batch);
    }

    let cp = s.health().clip_pressure_distribution;

    assert!(
        cp.min <= cp.mean && cp.mean <= cp.max,
        "invariant violated: min={} ≤ mean={} ≤ max={}",
        cp.min,
        cp.mean,
        cp.max,
    );
}

// ═══════════════════════════════════════════════════════════
//  Per-axis visibility
// ═══════════════════════════════════════════════════════════

/// Rejection is visible at the grain it actually happens: every batch report
/// carries a separate, finite, non-negative figure for each of the four scoring
/// axes of every cell reported. Each axis keeps its own ceiling and its own
/// running rejection rate, so a caller can see which axis is under strain
/// rather than only that something is.
///
/// ´claim:pressure:every-scoring-axis-of-every-reported-cell-carries-its-own-finite-rejection-rate´
/// ´test:integration:per-axis-clip-pressure-in-cell-reports´
#[test]
fn per_axis_clip_pressure_in_cell_reports() {
    // After ingestion, the batch report's cell_reports should
    // carry finite, non-negative clip_pressure on every axis.
    let mut s = warmed_sentinel(clip_cold_config(0.95), 0xA, 50);

    let report = s.ingest(&diverse_cell_values(0xA, 16, 50));

    for cr in &report.cell_reports {
        for (name, cp) in [
            ("novelty", cr.scores.novelty.clip_pressure),
            ("displacement", cr.scores.displacement.clip_pressure),
            ("surprise", cr.scores.surprise.clip_pressure),
            ("coherence", cr.scores.coherence.clip_pressure),
        ] {
            assert!(
                cp.is_finite() && cp >= 0.0,
                "cell gnode {:?} axis {name}: clip_pressure should be finite and ≥ 0, got {cp}",
                cr.gnode_id,
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════
//  Clean-traffic equilibrium (§11.2)
// ═══════════════════════════════════════════════════════════

/// Traffic that keeps its shape does not ratchet the rejection rate upward. Over
/// a long clean run the second stretch of batches sits no higher than the first,
/// and the average across axes stays well under half. A modest steady rejection
/// is expected — a ceiling a few deviations out will always trim the occasional
/// sample — but it settles at a level rather than climbing, which is what lets
/// a rise be read as news.
///
/// ´claim:pressure:traffic-that-keeps-its-shape-holds-the-rejection-rate-at-a-modest-level-instead-of-climbing´
/// ´test:integration:clip-pressure-stable-under-clean-traffic´
#[test]
fn clip_pressure_stable_under_clean_traffic() {
    let mut s = warmed_sentinel(clip_cold_config(0.95), 0xA, 200);

    // Collect clip_pressure over the next 100 batches.
    let mut cp_values = Vec::with_capacity(100);
    for batch_id in 200..300 {
        s.ingest(&diverse_cell_values(0xA, 16, batch_id));
        cp_values.push(max_clip_pressure(&s));
    }

    // No upward trend: second half should not exceed first by much.
    let first_half_mean: f64 = cp_values[..50].iter().sum::<f64>() / 50.0;
    let second_half_mean: f64 = cp_values[50..].iter().sum::<f64>() / 50.0;

    assert!(
        second_half_mean <= first_half_mean + 0.05,
        "clip_pressure should not trend upward under clean traffic: \
         first_half={first_half_mean:.4}, second_half={second_half_mean:.4}"
    );

    // Mean should confirm stability.
    let mean_cp = mean_clip_pressure(&s);
    assert!(
        mean_cp < 0.5,
        "mean clip_pressure should be moderate under clean traffic, got {mean_cp:.4}"
    );
}

// ═══════════════════════════════════════════════════════════
//  Contamination dynamics (§11.1)
// ═══════════════════════════════════════════════════════════

/// Mixing a substantial minority of structurally novel values into an otherwise
/// settled stream drives the peak rejection rate clearly above where clean
/// traffic had left it. The rate is thus an observable symptom of contamination
/// in its own right: the ceiling is doing its job of keeping those samples out
/// of the baseline, and the pressure is the sentinel saying how hard it is
/// having to work at it.
///
/// ´claim:pressure:sustained-contamination-drives-the-rejection-rate-above-its-clean-level´
/// ´test:integration:contamination-elevates-clip-pressure´
#[test]
fn contamination_elevates_clip_pressure() {
    let mut s = warmed_sentinel(clip_cold_config(0.95), 0xA, 200);
    let cp_baseline = max_clip_pressure(&s);

    // 80 batches of mixed traffic: 60% normal + 40% outliers.
    for batch_id in 200..280 {
        let mut batch = diverse_cell_values(0xA, 10, batch_id);
        batch.extend(contaminated_values(0xA, 6, batch_id));
        s.ingest(&batch);
    }

    let cp_after = max_clip_pressure(&s);

    assert!(
        cp_after > cp_baseline + 0.05,
        "contamination should elevate clip_pressure: \
         baseline={cp_baseline:.4}, after={cp_after:.4}"
    );
}

/// When the contamination stops, the rejection rate comes back down: after a
/// long stretch of clean traffic the peak sits below where the contaminated
/// phase left it. The measure is a rolling one with a finite memory, so an
/// episode ages out instead of marking the sentinel permanently — a system that
/// never forgot would treat one past attack as grounds for a forever-loose
/// ceiling.
///
/// ´claim:pressure:the-rejection-rate-falls-back-once-the-contamination-stops´
/// ´test:integration:clip-pressure-decays-after-contamination´
#[test]
fn clip_pressure_decays_after_contamination() {
    let mut s = warmed_sentinel(clip_cold_config(0.95), 0xA, 200);

    // Contamination phase.
    for batch_id in 200..280 {
        let mut batch = diverse_cell_values(0xA, 10, batch_id);
        batch.extend(contaminated_values(0xA, 6, batch_id));
        s.ingest(&batch);
    }
    let cp_contaminated = max_clip_pressure(&s);

    // Recovery: 200 batches of clean traffic.
    for batch_id in 280..480 {
        s.ingest(&diverse_cell_values(0xA, 16, batch_id));
    }
    let cp_recovered = max_clip_pressure(&s);

    assert!(
        cp_recovered < cp_contaminated,
        "clip_pressure should decrease after contamination ends: \
         contaminated={cp_contaminated:.4}, recovered={cp_recovered:.4}"
    );
}

/// The rejection rate is not merely reported, it feeds back into the ceiling.
/// Recomputing the documented widening rule from the pressure a contaminated run
/// actually leaves behind gives a ceiling meaningfully above the configured one.
/// Refusal is therefore self-limiting by construction: the harder an axis has
/// been rejecting, the more room it grants itself, so a genuine and lasting
/// shift in the traffic can eventually be learned rather than clipped away for
/// ever.
///
/// ´claim:pressure:a-raised-rejection-rate-widens-the-ceiling-so-refusal-cannot-become-permanent´
/// ´test:integration:effective-ceiling-widens-under-pressure´
#[test]
fn effective_ceiling_widens_under_pressure() {
    let mut s = warmed_sentinel(clip_cold_config(0.95), 0xA, 200);

    // Contamination phase to elevate pressure.
    for batch_id in 200..280 {
        let mut batch = diverse_cell_values(0xA, 10, batch_id);
        batch.extend(contaminated_values(0xA, 6, batch_id));
        s.ingest(&batch);
    }

    let p = max_clip_pressure(&s);
    let clip_sigmas = 3.0_f64;
    let eps = 1e-6_f64;

    // §ALGO S-14.4 effective-clip formula: n_σ · (1 + ρ̄ / (1 − ρ̄ + ε))
    let effective_clip = clip_sigmas * (1.0 + p / (1.0 - p + eps));

    assert!(
        effective_clip > clip_sigmas * 1.01,
        "effective clip {effective_clip:.4} should exceed base clip_sigmas {clip_sigmas}"
    );
}

// ═══════════════════════════════════════════════════════════
//  Decay-rate effect
// ═══════════════════════════════════════════════════════════

/// How long the rejection rate remembers is a configured choice with an
/// observable consequence: given identical contamination and an identical
/// recovery window, the sentinel with the shorter memory ends up at a lower
/// pressure than the one with the longer. The setting therefore trades how
/// quickly a ceiling snaps back after an episode against how steadily it holds
/// through a noisy one.
///
/// ´claim:pressure:a-shorter-memory-brings-the-rejection-rate-back-down-within-a-shorter-recovery-window´
/// ´test:integration:faster-decay-recovers-sooner´
#[test]
fn faster_decay_recovers_sooner() {
    // Two sentinels, identical contamination, different λ_ρ.
    // Lower λ_ρ → faster decay → lower pressure after recovery.
    let fast_decay = 0.85;
    let slow_decay = 0.98;

    let contaminate_and_recover = |decay: f64| -> f64 {
        let mut s = warmed_sentinel(clip_cold_config(decay), 0xA, 200);

        for batch_id in 200..280 {
            let mut batch = diverse_cell_values(0xA, 10, batch_id);
            batch.extend(contaminated_values(0xA, 6, batch_id));
            s.ingest(&batch);
        }

        // Fixed recovery window: 100 batches of clean traffic.
        for batch_id in 280..380 {
            s.ingest(&diverse_cell_values(0xA, 16, batch_id));
        }

        max_clip_pressure(&s)
    };

    let cp_fast = contaminate_and_recover(fast_decay);
    let cp_slow = contaminate_and_recover(slow_decay);

    assert!(
        cp_fast < cp_slow,
        "faster decay (λ_ρ={fast_decay}) should recover to lower pressure \
         than slow decay (λ_ρ={slow_decay}): fast={cp_fast:.4}, slow={cp_slow:.4}"
    );
}

// ═══════════════════════════════════════════════════════════
//  Warm-up reset (§11.3)
// ═══════════════════════════════════════════════════════════

/// Rejection accrued while a tracker still leaned on synthetic history is
/// discarded at the moment that reliance falls away: shortly after the
/// crossing, the peak rate across the sentinel is still low, having had only a
/// handful of real batches in which to rebuild. Whatever the warming rounds
/// caused the ceiling to refuse is therefore not allowed to widen the ceiling
/// that production traffic will be judged against.
///
/// ´claim:pressure:the-rejection-rate-is-zeroed-when-a-tracker-stops-leaning-on-synthetic-history´
/// ´test:integration:warm-up-completion-resets-clip-pressure´
#[test]
fn warm_up_completion_resets_clip_pressure() {
    // With noise injection, clip_pressure may rise during warm-up.
    // After η crosses the warm-up threshold (0.01), the
    // update_maturity() callback zeros clip_pressure on all axes.
    //
    // With λ=0.90 and batch_size=8, η < 0.01 requires ~44 batches
    // (ln(0.01) / ln(0.90) ≈ 43.7). We run up to 100 batches.
    let cfg = SentinelConfig::<u64> {
        clip_pressure_decay: 0.95,
        clip_sigmas: 3.0,
        split_threshold: 100_000,
        forgetting_factor: 0.90,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Seed traffic to create the root cell.
    s.ingest(&cell_values(0xA, 16));

    // Feed real traffic until η crosses below 0.01.
    let root = s.graph().g_root();
    let mut crossed = false;
    let mut post_cross_batches = 0_usize;

    for batch_id in 0..100 {
        s.ingest(&diverse_cell_values(0xA, 16, batch_id));
        let insp = s.inspect_cell(root).unwrap();
        if !crossed && insp.maturity.noise_influence < 0.01 {
            crossed = true;
        }
        if crossed {
            post_cross_batches += 1;
            if post_cross_batches == 5 {
                // Clip_pressure was zeroed at crossing, then accumulated
                // for only 5 batches. With λ_ρ=0.95, even if every
                // batch clips 100%, ρ̄ ≤ 1 - 0.95^5 ≈ 0.23.
                let h = s.health();
                assert!(
                    h.clip_pressure_distribution.max < 0.30,
                    "clip_pressure should be low shortly after warm-up \
                     reset, got max={:.4} (5 batches post-crossing)",
                    h.clip_pressure_distribution.max
                );
                break;
            }
        }
    }

    assert!(crossed, "η should have crossed below 0.01 within 100 batches");
}
