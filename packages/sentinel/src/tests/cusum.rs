// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`starts_at_zero`] | cusum | A fresh accumulator holds no evidence and has taken no steps. Drift is something that must be accumulated from observations, so a newly created axis starts owing the host nothing to explain. |
//! | [`accumulates_under_sustained_elevation`] | cusum | Evidence builds when batch means stay above the slow reference: an accumulator settled by a long run of ordinary batches grows once the scores are consistently elevated. Because each batch adds its remaining gap to the running sum, sustained elevation compounds — which is the point, since it separates a persistent shift from a single loud batch. |
//! | [`steps_since_reset_increments`] | cusum | Every update advances the step count by exactly one, whatever the batch contained and whether or not the gap contributed anything. The count is how long evidence has been gathering, so a host can read an accumulator value against the number of chances it had to grow rather than against nothing. |
//! | [`clamps_at_zero_when_below_baseline`] | cusum | A run of batches below the reference leaves the accumulator at zero rather than driving it negative. Quiet time banks no credit: the sum cannot go into debt during a lull and then have to be repaid before a genuine rise registers. Evidence of drift is always built from the present run, never netted against the past. |
//! | [`allowance_absorbs_noise`] | cusum | The allowance is a dead band that ordinary variation does not cross: against a slow baseline with real spread, a generous allowance leaves slightly elevated batches accumulating essentially nothing. Because the band is scaled by the baseline's own deviation rather than being an absolute score, a noisy cell tolerates more before it counts as drifting than a quiet one does. |
//! | [`resets_to_zero`] | cusum | A reset discards the accumulated evidence and the count of steps that built it together. Neither outlives the other, so a host acknowledging a regime change is not left reading a fresh sum against a stale step count. |
//! | [`reset_preserves_slow_baseline`] | cusum | What a reset does not touch is the slow baseline: its mean and spread come through unchanged. Acknowledging drift clears the evidence, not the reference the evidence was measured against — otherwise every acknowledgement would throw away a long-memory baseline that takes many batches to rebuild, and the axis would be blind while it re-converged. |
//! | [`reset_cold_clears_everything`] | cusum | Clearing goes further than resetting: the evidence, the step count and the slow baseline all return to their freshly-constructed state, the baseline back to its placeholders rather than to whatever it had drifted to. This is the operation for an axis that has ceased to exist — coherence when the rank falls too low, say — where keeping a reference learned under a geometry that no longer holds would be worse than having none. |
//! | [`seed_slow_from_aligns_baselines`] | cusum | The slow reference can be started from a fast baseline that has already converged, taking its mean and spread exactly. After noise is injected the two would otherwise disagree for a very long time — the slow baseline's memory is far too long to catch up — and every batch in between would register a gap that reflects the mismatch rather than any real drift. Seeding closes that gap in one step. |
//! | [`update_filtered_matches_update_no_clip`] | cusum | The pre-filtered entry point differs from the ordinary one only in who applies the outlier filter: given a clip wide enough that none would be rejected, both reach the same accumulated value. Moving the filter out to a shared pipeline therefore changes where clipping happens and not what drift means. |
//! | [`snapshot_reports_slow_baseline`] | cusum | The snapshot carries the slow reference itself, and that reference tracks the scores it was fed: after a run of batches at one level the reported mean sits near it. A host reading a report can therefore see what the drift was measured against, not merely how much of it accumulated. |

//! Tests for [`CusumAccumulator`](crate::sentinel::cusum::CusumAccumulator) —
//! the one-sided drift accumulator each scoring axis owns.
//!
//! A single elevated batch is not drift. The accumulator answers a different
//! question from the baseline: not whether this batch is unusual, but whether
//! a run of batches has been consistently above the reference for long
//! enough to be evidence rather than noise. It measures each batch's mean
//! against a slow baseline, subtracts a noise allowance scaled to that
//! baseline's own spread, and adds what remains to a running sum.
//!
//! Two decisions shape it. The sum is clamped at zero, so a period below the
//! reference banks no credit against a later rise — evidence has to be built
//! afresh rather than offset. And the gap is measured before the slow
//! baseline sees the batch, so a batch is always scored against the state
//! that preceded it and can never partly explain itself away.
//!
//! Resetting and clearing are deliberately different operations. A reset
//! discards the accumulated evidence but keeps what the slow baseline
//! learned; clearing returns both to their freshly-constructed state, which
//! is what an axis that has ceased to exist requires.

use crate::sentinel::cusum::*;

// ─── Construction ───────────────────────────────────────────

/// A fresh accumulator holds no evidence and has taken no steps. Drift is
/// something that must be accumulated from observations, so a newly created
/// axis starts owing the host nothing to explain.
///
/// ´claim:cusum:a-fresh-accumulator-holds-no-evidence-and-has-taken-no-steps´
/// ´test:crate:starts-at-zero´
#[test]
fn starts_at_zero() {
    let c = CusumAccumulator::new(0.999);
    let snap = c.snapshot();
    assert!((snap.accumulator).abs() < f64::EPSILON);
    assert_eq!(snap.steps_since_reset, 0);
}

// ─── Core accumulation ─────────────────────────────────────

/// Evidence builds when batch means stay above the slow reference: an
/// accumulator settled by a long run of ordinary batches grows once the
/// scores are consistently elevated. Because each batch adds its remaining
/// gap to the running sum, sustained elevation compounds — which is the
/// point, since it separates a persistent shift from a single loud batch.
///
/// ´claim:cusum:sustained-elevation-above-the-slow-reference-accumulates-as-evidence´
/// ´test:crate:accumulates-under-sustained-elevation´
#[test]
fn accumulates_under_sustained_elevation() {
    let mut c = CusumAccumulator::new(0.999);
    // Warm the slow baseline with normal-ish scores.
    for _ in 0..20 {
        c.update(&[1.0, 1.0, 1.0], 1.0, 0.5, 1e-6, 3.0);
    }
    let before = c.snapshot().accumulator;

    // Now feed consistently elevated scores.
    for _ in 0..10 {
        c.update(&[5.0, 5.0, 5.0], 5.0, 0.5, 1e-6, 3.0);
    }
    assert!(
        c.snapshot().accumulator > before,
        "accumulator should grow under sustained elevation"
    );
}

/// Every update advances the step count by exactly one, whatever the batch
/// contained and whether or not the gap contributed anything. The count is
/// how long evidence has been gathering, so a host can read an accumulator
/// value against the number of chances it had to grow rather than against
/// nothing.
///
/// ´claim:cusum:every-update-advances-the-step-count-by-one-whatever-the-batch-contributed´
/// ´test:crate:steps-since-reset-increments´
#[test]
fn steps_since_reset_increments() {
    let mut c = CusumAccumulator::new(0.999);
    for i in 1..=5 {
        c.update(&[1.0], 1.0, 0.5, 1e-6, 3.0);
        assert_eq!(c.snapshot().steps_since_reset, i);
    }
}

// ─── Clamping ───────────────────────────────────────────────

/// A run of batches below the reference leaves the accumulator at zero rather
/// than driving it negative. Quiet time banks no credit: the sum cannot go
/// into debt during a lull and then have to be repaid before a genuine rise
/// registers. Evidence of drift is always built from the present run, never
/// netted against the past.
///
/// ´claim:cusum:a-quiet-run-banks-no-credit-because-the-sum-is-clamped-at-zero´
/// ´test:crate:clamps-at-zero-when-below-baseline´
#[test]
fn clamps_at_zero_when_below_baseline() {
    let mut c = CusumAccumulator::new(0.999);
    // Warm with high values.
    for _ in 0..20 {
        c.update(&[10.0, 10.0], 10.0, 0.5, 1e-6, 3.0);
    }
    c.reset();

    // Feed low values — gap is negative, accumulator stays at zero.
    for _ in 0..10 {
        c.update(&[0.1, 0.1], 0.1, 0.5, 1e-6, 3.0);
    }
    assert!(
        (c.snapshot().accumulator).abs() < f64::EPSILON,
        "accumulator should not go below zero"
    );
}

// ─── Allowance ──────────────────────────────────────────────

/// The allowance is a dead band that ordinary variation does not cross:
/// against a slow baseline with real spread, a generous allowance leaves
/// slightly elevated batches accumulating essentially nothing. Because the
/// band is scaled by the baseline's own deviation rather than being an
/// absolute score, a noisy cell tolerates more before it counts as drifting
/// than a quiet one does.
///
/// ´claim:cusum:the-allowance-is-a-dead-band-scaled-to-the-baselines-own-spread´
/// ´test:crate:allowance-absorbs-noise´
#[test]
fn allowance_absorbs_noise() {
    // With a large allowance, small deviations should not accumulate
    // when the slow baseline has meaningful variance.
    let mut c = CusumAccumulator::new(0.999);

    // Warm with varied data so the slow baseline has real variance.
    for _ in 0..20 {
        c.update(&[0.5, 1.0, 1.5], 1.0, 2.0, 1e-6, 3.0);
    }
    c.reset();

    // Feed slightly elevated scores — allowance should absorb them.
    for _ in 0..10 {
        c.update(&[1.1, 1.2, 1.3], 1.2, 2.0, 1e-6, 3.0);
    }
    assert!(
        c.snapshot().accumulator < 0.1,
        "generous allowance should absorb small deviations, got {}",
        c.snapshot().accumulator,
    );
}

// ─── Reset ──────────────────────────────────────────────────

/// A reset discards the accumulated evidence and the count of steps that
/// built it together. Neither outlives the other, so a host acknowledging a
/// regime change is not left reading a fresh sum against a stale step count.
///
/// ´claim:cusum:a-reset-discards-the-evidence-and-the-count-that-built-it-together´
/// ´test:crate:resets-to-zero´
#[test]
fn resets_to_zero() {
    let mut c = CusumAccumulator::new(0.999);
    c.update(&[5.0, 5.0], 5.0, 0.0, 1e-6, 3.0);
    c.update(&[5.0, 5.0], 5.0, 0.0, 1e-6, 3.0);
    assert!(c.snapshot().accumulator > 0.0);

    c.reset();
    assert!((c.snapshot().accumulator).abs() < f64::EPSILON);
    assert_eq!(c.snapshot().steps_since_reset, 0);
}

/// What a reset does not touch is the slow baseline: its mean and spread come
/// through unchanged. Acknowledging drift clears the evidence, not the
/// reference the evidence was measured against — otherwise every
/// acknowledgement would throw away a long-memory baseline that takes many
/// batches to rebuild, and the axis would be blind while it re-converged.
///
/// ´claim:cusum:a-reset-clears-the-evidence-without-discarding-the-reference-it-was-measured-against´
/// ´test:crate:reset-preserves-slow-baseline´
#[test]
fn reset_preserves_slow_baseline() {
    let mut c = CusumAccumulator::new(0.999);
    for _ in 0..20 {
        c.update(&[5.0, 5.0], 5.0, 0.5, 1e-6, 3.0);
    }
    let baseline_before = c.snapshot().slow_baseline;

    c.reset();

    let baseline_after = c.snapshot().slow_baseline;
    assert!(
        (baseline_before.mean - baseline_after.mean).abs() < f64::EPSILON,
        "reset() should preserve the slow baseline mean"
    );
    assert!(
        (baseline_before.variance - baseline_after.variance).abs() < f64::EPSILON,
        "reset() should preserve the slow baseline variance"
    );
}

/// Clearing goes further than resetting: the evidence, the step count and the
/// slow baseline all return to their freshly-constructed state, the baseline
/// back to its placeholders rather than to whatever it had drifted to. This
/// is the operation for an axis that has ceased to exist — coherence when the
/// rank falls too low, say — where keeping a reference learned under a
/// geometry that no longer holds would be worse than having none.
///
/// ´claim:cusum:clearing-returns-the-evidence-the-count-and-the-reference-all-to-their-constructed-state´
/// ´test:crate:reset-cold-clears-everything´
#[test]
fn reset_cold_clears_everything() {
    let mut c = CusumAccumulator::new(0.999);
    for _ in 0..20 {
        c.update(&[5.0, 5.0], 5.0, 0.0, 1e-6, 3.0);
    }
    assert!(c.snapshot().accumulator > 0.0);
    // Slow baseline should have drifted away from the cold defaults.
    assert!((c.snapshot().slow_baseline.mean - 1.0).abs() > 0.1);

    c.reset_cold();

    let snap = c.snapshot();
    assert!((snap.accumulator).abs() < f64::EPSILON);
    assert_eq!(snap.steps_since_reset, 0);
    // Cold defaults: mean = 1.0, variance = 1.0.
    assert!(
        (snap.slow_baseline.mean - 1.0).abs() < f64::EPSILON,
        "reset_cold should restore cold-default mean"
    );
    assert!(
        (snap.slow_baseline.variance - 1.0).abs() < f64::EPSILON,
        "reset_cold should restore cold-default variance"
    );
}

// ─── Seeding ────────────────────────────────────────────────

/// The slow reference can be started from a fast baseline that has already
/// converged, taking its mean and spread exactly. After noise is injected the
/// two would otherwise disagree for a very long time — the slow baseline's
/// memory is far too long to catch up — and every batch in between would
/// register a gap that reflects the mismatch rather than any real drift.
/// Seeding closes that gap in one step.
///
/// ´claim:cusum:the-slow-reference-can-be-seeded-from-a-converged-fast-baseline-so-the-two-start-in-agreement´
/// ´test:crate:seed-slow-from-aligns-baselines´
#[test]
fn seed_slow_from_aligns_baselines() {
    use crate::ewma::EwmaStats;

    let mut fast = EwmaStats::new(0.95);
    for _ in 0..20 {
        fast.update(&[3.0, 3.5, 2.5], 3.0);
    }

    let mut c = CusumAccumulator::new(0.999);
    c.seed_slow_from(&fast);

    let snap = c.snapshot();
    assert!(
        (snap.slow_baseline.mean - fast.mean()).abs() < f64::EPSILON,
        "seed_slow_from should copy mean from fast EWMA"
    );
    assert!(
        (snap.slow_baseline.variance - fast.variance()).abs() < f64::EPSILON,
        "seed_slow_from should copy variance from fast EWMA"
    );
}

// ─── Filtered update ────────────────────────────────────────

/// The pre-filtered entry point differs from the ordinary one only in who
/// applies the outlier filter: given a clip wide enough that none would be
/// rejected, both reach the same accumulated value. Moving the filter out to
/// a shared pipeline therefore changes where clipping happens and not what
/// drift means.
///
/// ´claim:cusum:the-pre-filtered-path-differs-from-the-ordinary-one-only-in-who-applies-the-filter´
/// ´test:crate:update-filtered-matches-update-no-clip´
#[test]
fn update_filtered_matches_update_no_clip() {
    let mut a = CusumAccumulator::new(0.999);
    let mut b = CusumAccumulator::new(0.999);
    let scores = &[1.0, 1.1, 0.9, 1.05, 0.95];
    #[allow(clippy::cast_precision_loss)]
    let mean = scores.iter().sum::<f64>() / scores.len() as f64;

    a.update(scores, mean, 0.5, 1e-6, 100.0);
    b.update_filtered(scores, mean, 0.5, 1e-6);

    assert!((a.snapshot().accumulator - b.snapshot().accumulator).abs() < 1e-12);
}

// ─── Snapshot ───────────────────────────────────────────────

/// The snapshot carries the slow reference itself, and that reference tracks
/// the scores it was fed: after a run of batches at one level the reported
/// mean sits near it. A host reading a report can therefore see what the
/// drift was measured against, not merely how much of it accumulated.
///
/// ´claim:cusum:a-snapshot-carries-the-slow-reference-and-that-reference-tracks-the-scores-it-was-fed´
/// ´test:crate:snapshot-reports-slow-baseline´
#[test]
fn snapshot_reports_slow_baseline() {
    let mut c = CusumAccumulator::new(0.999);
    for _ in 0..20 {
        c.update(&[4.0, 4.0], 4.0, 0.5, 1e-6, 3.0);
    }

    let snap = c.snapshot();
    // After warming, baseline mean should be near 4.0.
    assert!(
        (snap.slow_baseline.mean - 4.0).abs() < 0.5,
        "slow baseline mean should track input; got {}",
        snap.slow_baseline.mean,
    );
}
