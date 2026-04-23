// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Correctness tests for **[`Plan`] builder methods**.
//!
//! Every observation plan used in the test suite is assembled from the
//! combinators on [`Plan`].  These tests verify each builder in
//! isolation — correct length, coordinate placement, delta values,
//! determinism, and chaining semantics — so that higher-level graph
//! tests can rely on the plans being well-formed.
//!
//! # Test index
//!
//! ## Core methods
//!
//! | Test | Focus |
//! |------|-------|
//! | [`new_plan_is_empty`] | freshly constructed plan has zero observations |
//! | [`observe_appends_single_observation`] | `observe` pushes exactly one `(coord, delta)` pair |
//! | [`observe_n_appends_n_observations`] | `observe_n` repeats the same pair *n* times |
//! | [`observe_n_zero_is_noop`] | `observe_n` with count 0 leaves the plan unchanged |
//! | [`hotspot_delegates_to_observe_n`] | `hotspot` produces identical output to `observe_n` |
//! | [`then_concatenates_observations`] | `then` appends observations from a second plan |
//! | [`then_empty_is_identity`] | concatenating an empty plan is an identity operation |
//! | [`default_is_empty`] | `Default` impl yields an empty plan |
//! | [`plan_equality`] | structural equality and inequality of plans |
//!
//! ## Basic patterns
//!
//! | Test | Focus |
//! |------|-------|
//! | [`spread_count_and_coord_cycling`] | `spread` cycles coords modulo domain |
//! | [`sweep_count_and_varying_intensity`] | `sweep` cycles coords with escalating delta |
//! | [`zigzag_alternates_lo_hi`] | `zigzag` alternates between lo and hi coords |
//! | [`adversarial_zigzag_escalates_lo_side`] | lo-side delta escalates while hi stays at base |
//! | [`skewed_lower_quarter_gets_higher_intensity`] | lower quarter averages higher intensity than upper |
//! | [`burst_spread_then_hotspot`] | `burst` emits a spread phase followed by a hotspot phase |
//!
//! ## Random / stochastic
//!
//! | Test | Focus |
//! |------|-------|
//! | [`random_spray_produces_expected_count`] | output length matches requested count |
//! | [`random_spray_is_deterministic`] | same seed produces identical plans |
//! | [`random_spray_different_seeds_differ`] | different seeds produce different plans |
//! | [`random_spray_coords_within_domain`] | all coords fall within `[0, domain)` |
//!
//! ## Oscillation / hotspot
//!
//! | Test | Focus |
//! |------|-------|
//! | [`oscillating_hotspot_count`] | total observations = burst × cycles |
//! | [`oscillating_hotspot_alternates_targets`] | target coord alternates each cycle |
//! | [`oscillating_hotspot_zero_cycles_is_empty`] | zero cycles yields an empty plan |
//! | [`interleaved_hotspots_count_and_cycling`] | *k* hotspots cycle round-robin across cycles |
//! | [`phase_shifted_oscillation_count_and_symmetry`] | two targets swap high/low roles each cycle |
//!
//! ## Targeted / adversarial
//!
//! | Test | Focus |
//! |------|-------|
//! | [`cousin_rivalry_count_and_alternation`] | observations alternate between two disjoint regions |
//! | [`adversarial_cousins_setup_then_spikes`] | gentle setup phase followed by high-delta spikes |
//! | [`neighborhood_burst_coords_within_radius`] | all coords lie within `center ± radius` |
//!
//! ## Deep / structural
//!
//! | Test | Focus |
//! |------|-------|
//! | [`fractal_spray_count_by_depth`] | total points = `2^depth − 1` (geometric sum) |
//! | [`nested_bursts_count_by_levels`] | same as `fractal_spray` scaled by `burst_per_level` |
//! | [`depth_ladder_count`] | `(bits + 1) × reps` observations with decreasing powers of 2 |
//! | [`staircase_count_and_escalation`] | stair count grows linearly, delta scales per stair |
//! | [`cascade_shift_build_then_transition`] | build phase at one coord, then partial shift to another |
//!
//! ## Coverage / topology
//!
//! | Test | Focus |
//! |------|-------|
//! | [`diamond_count`] | correct observation count for diamond pattern |
//! | [`bit_flip_walk_uses_gray_code`] | coords follow the Gray-code sequence |
//! | [`sibling_flood_fills_halves_sequentially`] | lower half filled first, then upper half |
//! | [`mirrored_growth_is_symmetric`] | every pair sums to `domain − 1` |
//! | [`checkerboard_even_then_odd`] | even coords emitted first, odd coords second |
//! | [`pincer_converges`] | left and right pointers converge toward the centre |
//! | [`repulsion_walk_diverges_from_center`] | paired coords walk outward from centre |
//! | [`sawtooth_escalating_cycles`] | delta scales linearly with cycle number |
//! | [`centroid_drift_slides_across_domain`] | centroid slides from `start` to `end` |
//! | [`pulse_decay_intensity_decreases`] | delta decreases each round (`initial / round`) |
//! | [`golden_spiral_coords_within_domain`] | coords in `[0, domain)` with high distinct coverage |
//!
//! ## Chaining
//!
//! | Test | Focus |
//! |------|-------|
//! | [`multi_builder_chain`] | three builders chained; total length is additive |
//! | [`chained_plans_preserve_order`] | observation order matches builder invocation order |

use crate::testing::Plan;

// ── Helpers ─────────────────────────────────────────────────────

/// Shorthand: `Plan<u64, u64>`.
type P = Plan<u64, u64>;

fn plan() -> P {
    P::new()
}

// ═══════════════════════════════════════════════════════════════
// § Core methods
// ═══════════════════════════════════════════════════════════════

#[test]
fn new_plan_is_empty() {
    let p = plan();
    assert!(p.is_empty());
    assert_eq!(p.len(), 0);
}

#[test]
fn observe_appends_single_observation() {
    let p = plan().observe(42, 7);
    assert_eq!(p.len(), 1);
    assert_eq!(p.observations[0], (42, 7));
}

#[test]
fn observe_n_appends_n_observations() {
    let p = plan().observe_n(10, 3, 5);
    assert_eq!(p.len(), 5);
    assert!(p.observations.iter().all(|&obs| obs == (10, 3)));
}

#[test]
fn observe_n_zero_is_noop() {
    let p = plan().observe_n(10, 3, 0);
    assert!(p.is_empty());
}

#[test]
fn hotspot_delegates_to_observe_n() {
    let via_hotspot = plan().hotspot(5, 8, 4);
    let via_observe_n = plan().observe_n(5, 8, 4);
    assert_eq!(via_hotspot, via_observe_n);
}

#[test]
fn then_concatenates_observations() {
    let a = plan().observe(1, 10).observe(2, 20);
    let b = plan().observe(3, 30);
    let combined = a.then(b);
    assert_eq!(combined.len(), 3);
    assert_eq!(combined.observations[0], (1, 10));
    assert_eq!(combined.observations[1], (2, 20));
    assert_eq!(combined.observations[2], (3, 30));
}

#[test]
fn then_empty_is_identity() {
    let p = plan().observe(1, 1);
    let combined = p.clone().then(plan());
    assert_eq!(combined, p);

    let combined2 = plan().then(p.clone());
    assert_eq!(combined2, p);
}

#[test]
fn default_is_empty() {
    let p = P::default();
    assert!(p.is_empty());
    assert_eq!(p, plan());
}

#[test]
fn plan_equality() {
    let a = plan().observe(1, 2).observe(3, 4);
    let b = plan().observe(1, 2).observe(3, 4);
    let c = plan().observe(1, 2).observe(3, 5);
    assert_eq!(a, b);
    assert_ne!(a, c);
}

// ═══════════════════════════════════════════════════════════════
// § Basic patterns
// ═══════════════════════════════════════════════════════════════

#[test]
fn spread_count_and_coord_cycling() {
    let p = plan().spread(4, 1, 10);
    assert_eq!(p.len(), 10);

    // Coords cycle: 0, 1, 2, 3, 0, 1, 2, 3, 0, 1
    for (i, &(coord, delta)) in p.observations.iter().enumerate() {
        assert_eq!(coord, (i as u64) % 4, "coord at index {i}");
        assert_eq!(delta, 1, "delta at index {i}");
    }
}

#[test]
fn sweep_count_and_varying_intensity() {
    let p = plan().sweep(8, 14);
    assert_eq!(p.len(), 14);

    // Delta = (i % 7) + 1, cycling 1..=7
    for (i, &(coord, delta)) in p.observations.iter().enumerate() {
        assert_eq!(coord, (i as u64) % 8);
        assert_eq!(delta, (i as u64) % 7 + 1);
    }
}

#[test]
fn zigzag_alternates_lo_hi() {
    let p = plan().zigzag(0, 255, 5, 6);
    assert_eq!(p.len(), 6);

    for (i, &(coord, delta)) in p.observations.iter().enumerate() {
        let expected_coord = if i % 2 == 0 { 0 } else { 255 };
        assert_eq!(coord, expected_coord, "coord at index {i}");
        assert_eq!(delta, 5);
    }
}

#[test]
fn adversarial_zigzag_escalates_lo_side() {
    let p = plan().adversarial_zigzag(0, 100, 5, 3, 6);
    assert_eq!(p.len(), 6);

    // Even indices → coord 0, delta = base + i * step = 5 + i*3
    // Odd indices  → coord 100, delta = base = 5
    for (i, &(coord, delta)) in p.observations.iter().enumerate() {
        if i % 2 == 0 {
            assert_eq!(coord, 0, "even index {i} should target lo");
            assert_eq!(delta, 5 + (i as u64) * 3, "escalating delta at {i}");
        } else {
            assert_eq!(coord, 100, "odd index {i} should target hi");
            assert_eq!(delta, 5, "hi side stays at base");
        }
    }
}

#[test]
fn skewed_lower_quarter_gets_higher_intensity() {
    let domain = 16;
    let p = plan().skewed(domain, 32);
    assert_eq!(p.len(), 32);

    let quarter = domain / 4;
    let (mut lo_sum, mut hi_sum) = (0u64, 0u64);
    let (mut lo_count, mut hi_count) = (0usize, 0usize);

    for &(coord, delta) in &p.observations {
        if coord < quarter {
            lo_sum += delta;
            lo_count += 1;
        } else {
            hi_sum += delta;
            hi_count += 1;
        }
    }

    // Lower quarter should average much higher intensity
    assert!(lo_count > 0 && hi_count > 0);
    #[allow(clippy::cast_precision_loss)]
    let lo_avg = lo_sum as f64 / lo_count as f64;
    #[allow(clippy::cast_precision_loss)]
    let hi_avg = hi_sum as f64 / hi_count as f64;
    assert!(lo_avg > hi_avg, "lo_avg {lo_avg} should exceed hi_avg {hi_avg}");
}

#[test]
fn burst_spread_then_hotspot() {
    let p = plan().burst(8, 1, 10, 0, 50, 5);
    assert_eq!(p.len(), 15);

    // First 10: spread across domain 8
    for (i, &(coord, delta)) in p.observations[..10].iter().enumerate() {
        assert_eq!(coord, (i as u64) % 8);
        assert_eq!(delta, 1);
    }

    // Last 5: hotspot at coord 0
    for &(coord, delta) in &p.observations[10..] {
        assert_eq!(coord, 0);
        assert_eq!(delta, 50);
    }
}

// ═══════════════════════════════════════════════════════════════
// § Random / stochastic
// ═══════════════════════════════════════════════════════════════

#[test]
fn random_spray_produces_expected_count() {
    let p = plan().random_spray(42, 256, 6, 100);
    assert_eq!(p.len(), 100);
}

#[test]
fn random_spray_is_deterministic() {
    let a = plan().random_spray(42, 256, 6, 100);
    let b = plan().random_spray(42, 256, 6, 100);
    assert_eq!(a, b);
}

#[test]
fn random_spray_different_seeds_differ() {
    let a = plan().random_spray(42, 256, 6, 100);
    let b = plan().random_spray(99, 256, 6, 100);
    assert_ne!(a, b);
}

#[test]
fn random_spray_coords_within_domain() {
    let domain = 256;
    let p = plan().random_spray(42, domain, 6, 200);
    for &(coord, _) in &p.observations {
        assert!(coord < domain, "coord {coord} out of domain {domain}");
    }
}

// ═══════════════════════════════════════════════════════════════
// § Oscillation / hotspot
// ═══════════════════════════════════════════════════════════════

#[test]
fn oscillating_hotspot_count() {
    let p = plan().oscillating_hotspot(0, 255, 10, 5, 4);
    assert_eq!(p.len(), 20); // 5 burst × 4 cycles
}

#[test]
fn oscillating_hotspot_alternates_targets() {
    let p = plan().oscillating_hotspot(0, 255, 10, 3, 4);
    // cycle 0 → coord 0, cycle 1 → coord 255, ...
    for cycle in 0..4 {
        let expected = if cycle % 2 == 0 { 0 } else { 255 };
        for j in 0..3 {
            assert_eq!(p.observations[cycle * 3 + j].0, expected, "cycle {cycle} obs {j}");
        }
    }
}

#[test]
fn oscillating_hotspot_zero_cycles_is_empty() {
    let p = plan().oscillating_hotspot(0, 255, 10, 5, 0);
    assert!(p.is_empty());
}

#[test]
fn interleaved_hotspots_count_and_cycling() {
    // 3 hotspots at coords 0, 10, 20 — 4 burst each — 6 cycles
    let p = plan().interleaved_hotspots(0, 3, 10, 5, 4, 6);
    assert_eq!(p.len(), 24); // 4 burst × 6 cycles

    // Cycle 0 → coord 0, cycle 1 → coord 10, cycle 2 → coord 20,
    // cycle 3 → coord 0, ...
    for cycle in 0..6 {
        let expected_coord = (cycle as u64 % 3) * 10;
        assert_eq!(p.observations[cycle * 4].0, expected_coord, "cycle {cycle}");
    }
}

#[test]
fn phase_shifted_oscillation_count_and_symmetry() {
    let p = plan().phase_shifted_oscillation(10, 20, 50, 2, 3, 4);
    // Each cycle: burst_len * 2 observations (one a, one b per step)
    assert_eq!(p.len(), 3 * 2 * 4); // burst_len × 2 × cycles = 24

    // Cycle 0: a gets high=50, b gets low=2
    assert_eq!(p.observations[0], (10, 50)); // a, high
    assert_eq!(p.observations[1], (20, 2)); // b, low
    // Cycle 1: swapped
    let cycle1_start = 3 * 2; // 6
    assert_eq!(p.observations[cycle1_start], (10, 2)); // a, low
    assert_eq!(p.observations[cycle1_start + 1], (20, 50)); // b, high
}

// ═══════════════════════════════════════════════════════════════
// § Targeted / adversarial
// ═══════════════════════════════════════════════════════════════

#[test]
fn cousin_rivalry_count_and_alternation() {
    let p = plan().cousin_rivalry(0, 128, 64, 5, 3, 10);
    assert_eq!(p.len(), 10);

    // Even indices → region a (coords in [0, 64))
    // Odd indices  → region b (coords in [128, 192))
    for (i, &(coord, _)) in p.observations.iter().enumerate() {
        if i % 2 == 0 {
            assert!(coord < 64, "even obs {i}: coord {coord} not in [0,64)");
        } else {
            assert!((128..192).contains(&coord), "odd obs {i}: coord {coord} not in [128,192)");
        }
    }
}

#[test]
fn adversarial_cousins_setup_then_spikes() {
    let p = plan().adversarial_cousins(10, 20, 1, 3, 50, 30, 2);
    // Setup: 3 pairs (a, b) = 6 observations
    // Spikes: 2 pairs (a, b) = 4 observations
    assert_eq!(p.len(), 10);

    // Setup phase: gentle delta=1 at both coords
    for i in 0..6 {
        let expected_coord = if i % 2 == 0 { 10 } else { 20 };
        assert_eq!(p.observations[i].0, expected_coord, "setup obs {i}");
        assert_eq!(p.observations[i].1, 1, "setup delta at {i}");
    }

    // Spike phase: alternating high deltas
    assert_eq!(p.observations[6], (10, 50));
    assert_eq!(p.observations[7], (20, 30));
    assert_eq!(p.observations[8], (10, 50));
    assert_eq!(p.observations[9], (20, 30));
}

#[test]
fn neighborhood_burst_coords_within_radius() {
    let center = 100;
    let radius = 5;
    let p = plan().neighborhood_burst(center, radius, 10, 30);
    assert_eq!(p.len(), 30);

    let lo = center - radius;
    let hi = center + radius;
    for &(coord, _) in &p.observations {
        assert!((lo..=hi).contains(&coord), "coord {coord} outside [{lo}, {hi}]");
    }
}

// ═══════════════════════════════════════════════════════════════
// § Deep / structural
// ═══════════════════════════════════════════════════════════════

#[test]
fn fractal_spray_count_by_depth() {
    // Depth d produces 2^d points at level d.
    // Total = sum(2^0, 2^1, ..., 2^(depth-1)) = 2^depth - 1
    let p = plan().fractal_spray(256, 5, 4);
    assert_eq!(p.len(), 15); // 1 + 2 + 4 + 8
}

#[test]
fn nested_bursts_count_by_levels() {
    // Same point counts as fractal_spray, but each × burst_per_level
    let p = plan().nested_bursts(256, 5, 4, 3);
    assert_eq!(p.len(), 15 * 3); // (2^4 - 1) × 3 = 45
}

#[test]
fn depth_ladder_count() {
    // Each rep: 1 (coord 0) + bits coords = (bits + 1) per rep
    let bits = 6;
    let reps = 3;
    let p = plan().depth_ladder(bits, 5, reps);
    assert_eq!(p.len(), (bits as usize + 1) * reps);

    // First obs of each rep is coord 0, then decreasing powers of 2
    assert_eq!(p.observations[0].0, 0);
    assert_eq!(p.observations[1].0, 1 << 5); // 2^(bits-1) = 32
    assert_eq!(p.observations[2].0, 1 << 4); // 16
    assert_eq!(p.observations[3].0, 1 << 3); // 8
}

#[test]
fn staircase_count_and_escalation() {
    // stairs=3, reps_per_stair=2, step=10
    // Stair 1: 1 coord × 2 reps = 2
    // Stair 2: 2 coords × 2 reps = 4
    // Stair 3: 3 coords × 2 reps = 6
    // Total = 12
    let p = plan().staircase(10, 3, 2, 5);
    assert_eq!(p.len(), 12);

    // Stair 1: delta = base×1 = 5
    assert_eq!(p.observations[0].1, 5);
    // Stair 2: delta = base×2 = 10
    assert_eq!(p.observations[2].1, 10);
    // Stair 3: delta = base×3 = 15
    assert_eq!(p.observations[6].1, 15);
}

#[test]
fn cascade_shift_build_then_transition() {
    let p = plan().cascade_shift(10, 20, 5, 4, 6);
    // Build phase: 4 at coord 10
    // Shift phase: 6 obs, every 3rd (indices 0,3) at coord 10, rest at 20
    assert_eq!(p.len(), 10);

    // Build phase
    for i in 0..4 {
        assert_eq!(p.observations[i], (10, 5), "build phase obs {i}");
    }

    // Shift phase: i%3==0 → coord 10 (lingering), else → coord 20
    let shift = &p.observations[4..];
    assert_eq!(shift[0].0, 10); // i=0: lingering
    assert_eq!(shift[1].0, 20); // i=1: new target
    assert_eq!(shift[2].0, 20); // i=2: new target
    assert_eq!(shift[3].0, 10); // i=3: lingering
    assert_eq!(shift[4].0, 20); // i=4: new target
    assert_eq!(shift[5].0, 20); // i=5: new target
}

// ═══════════════════════════════════════════════════════════════
// § Coverage / topology
// ═══════════════════════════════════════════════════════════════

#[test]
fn diamond_count() {
    let p = plan().diamond(64, 5, 20);
    assert_eq!(p.len(), 20);
}

#[test]
fn bit_flip_walk_uses_gray_code() {
    let p = plan().bit_flip_walk(16, 5, 8);
    assert_eq!(p.len(), 8);

    // Gray code sequence: 0, 1, 3, 2, 7, 5, 4, 6 (for raw 0..8, mod 16)
    let expected: Vec<u64> = (0..8u64).map(|i| i ^ (i >> 1)).collect();
    for (i, &(coord, _)) in p.observations.iter().enumerate() {
        assert_eq!(coord, expected[i], "Gray code mismatch at index {i}");
    }
}

#[test]
fn sibling_flood_fills_halves_sequentially() {
    let domain = 16;
    let p = plan().sibling_flood(domain, 10, 20, 5);
    assert_eq!(p.len(), 10); // 5 per half

    // First half: coords in [0, 8)
    for &(coord, delta) in &p.observations[..5] {
        assert!(coord < domain / 2, "first half coord {coord}");
        assert_eq!(delta, 10);
    }
    // Second half: coords in [8, 16)
    for &(coord, delta) in &p.observations[5..] {
        assert!(coord >= domain / 2, "second half coord {coord}");
        assert_eq!(delta, 20);
    }
}

#[test]
fn mirrored_growth_is_symmetric() {
    let domain = 32;
    let p = plan().mirrored_growth(domain, 5, 6);
    // Each iteration emits 2 observations → 12 total
    assert_eq!(p.len(), 12);

    // Pairs: (x, domain - 1 - x)
    for pair in p.observations.chunks(2) {
        let (a, b) = (pair[0].0, pair[1].0);
        assert_eq!(a + b, domain - 1, "mirror pair ({a}, {b})");
    }
}

#[test]
fn checkerboard_even_then_odd() {
    let domain = 16;
    let p = plan().checkerboard(domain, 10, 20, 8);
    assert_eq!(p.len(), 8);

    let half = 4;
    // First half: even coords
    for &(coord, delta) in &p.observations[..half] {
        assert_eq!(coord % 2, 0, "first phase should be even coords");
        assert_eq!(delta, 10);
    }
    // Second half: odd coords
    for &(coord, delta) in &p.observations[half..] {
        assert_eq!(coord % 2, 1, "second phase should be odd coords");
        assert_eq!(delta, 20);
    }
}

#[test]
fn pincer_converges() {
    let p = plan().pincer(0, 4, 5, 1);
    // Steps: (0,4), (1,3), (2,2) = 3 steps
    // Step 0: 2 obs (left≠right), step 1: 2, step 2: 1 (left==right)
    assert_eq!(p.len(), 5);

    let coords: Vec<u64> = p.observations.iter().map(|&(c, _)| c).collect();
    assert_eq!(coords, vec![0, 4, 1, 3, 2]);
}

#[test]
fn repulsion_walk_diverges_from_center() {
    let p = plan().repulsion_walk(10, 5, 3, 1);
    // offset 0: a=10, b=10 (same) → 1 obs
    // offset 1: a=9, b=11 → 2 obs
    // offset 2: a=8, b=12 → 2 obs
    // offset 3: a=7, b=13 → 2 obs
    assert_eq!(p.len(), 7);

    // First obs is center
    assert_eq!(p.observations[0].0, 10);
    // Subsequent pairs diverge
    assert_eq!(p.observations[1].0, 9);
    assert_eq!(p.observations[2].0, 11);
    assert_eq!(p.observations[3].0, 8);
    assert_eq!(p.observations[4].0, 12);
}

#[test]
fn sawtooth_escalating_cycles() {
    let width = 4;
    let p = plan().sawtooth(width, 3, 3);
    // 3 cycles × 4 coords = 12
    assert_eq!(p.len(), 12);

    // Cycle 0: delta = 3×1 = 3
    // Cycle 1: delta = 3×2 = 6
    // Cycle 2: delta = 3×3 = 9
    for (i, &(_, delta)) in p.observations.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let cycle = i / width as usize;
        let expected = 3 * (cycle as u64 + 1);
        assert_eq!(delta, expected, "cycle {cycle} obs {i}");
    }
}

#[test]
fn centroid_drift_slides_across_domain() {
    let p = plan().centroid_drift(5, 8, 10, 2);
    // Coords 5, 6, 7, 8 — each with burst_len=2 → 8 total
    assert_eq!(p.len(), 8);

    // Coords appear in sequence
    let coords: Vec<u64> = p.observations.iter().map(|&(c, _)| c).collect();
    assert_eq!(coords, vec![5, 5, 6, 6, 7, 7, 8, 8]);
}

#[test]
fn pulse_decay_intensity_decreases() {
    let p = plan().pulse_decay(10, 32, 60, 3, 4);
    // 3 rounds × 4 obs = 12
    assert_eq!(p.len(), 12);

    // Round 0: delta = 60/1 = 60
    // Round 1: delta = 60/2 = 30
    // Round 2: delta = 60/3 = 20
    assert_eq!(p.observations[0].1, 60);
    assert_eq!(p.observations[4].1, 30);
    assert_eq!(p.observations[8].1, 20);
}

#[test]
fn golden_spiral_coords_within_domain() {
    // Use a prime domain so gcd(step, domain) = 1, giving full coverage.
    let domain = 67;
    let p = plan().golden_spiral(domain, 5, 80);
    assert_eq!(p.len(), 80);

    for &(coord, _) in &p.observations {
        assert!(coord < domain, "coord {coord} out of domain {domain}");
    }

    // With a prime domain the golden-ratio step is coprime to it,
    // so the sequence visits every residue before repeating.
    let mut unique: Vec<u64> = p.observations.iter().map(|&(c, _)| c).collect();
    unique.sort_unstable();
    unique.dedup();
    #[allow(clippy::cast_possible_truncation)]
    let half = (domain / 2) as usize;
    assert!(
        unique.len() > half,
        "golden spiral should cover many distinct coords, got {}",
        unique.len()
    );
}

// ═══════════════════════════════════════════════════════════════
// § Chaining
// ═══════════════════════════════════════════════════════════════

#[test]
fn multi_builder_chain() {
    let p = plan()
        .spread(64, 1, 10)
        .random_spray(7, 128, 2, 20)
        .oscillating_hotspot(0, 63, 5, 4, 3);
    assert_eq!(p.len(), 10 + 20 + 12);
}

#[test]
fn chained_plans_preserve_order() {
    let p = plan()
        .observe(1, 100)
        .spread(4, 1, 3) // coords: 0, 1, 2
        .observe(99, 999);

    assert_eq!(p.len(), 5);
    assert_eq!(p.observations[0], (1, 100)); // observe
    assert_eq!(p.observations[1], (0, 1)); // spread[0]
    assert_eq!(p.observations[2], (1, 1)); // spread[1]
    assert_eq!(p.observations[3], (2, 1)); // spread[2]
    assert_eq!(p.observations[4], (99, 999)); // observe
}
