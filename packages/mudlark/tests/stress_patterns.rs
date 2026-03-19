// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Stress tests for diverse observation patterns.
//!
//! Exercises all 22 new observation patterns across multiple configs
//! and domain sizes.  Every test uses `run_checked` with
//! `check_every = 1` so invariants are validated after **every**
//! observation — maximising the chance of catching transient
//! violations (including source 10).

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{
    Plan, aggressive_config, cascade_config, deep_config, low_threshold_config, run_checked, wide_shallow_config,
};

mod support;
use support::init_tracing;

// =====================================================================
// Group A — Targeted patterns
// =====================================================================

// ── interleaved_hotspots ────────────────────────────────────────

#[test]
fn interleaved_hotspots_n4_cascade() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().interleaved_hotspots(0, 4, 2, 10, 5, 12);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn interleaved_hotspots_n8_aggressive() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().interleaved_hotspots(0, 8, 16, 8, 6, 20);
    let g = run_checked::<u64, u64, 8>(aggressive_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn interleaved_hotspots_tight_spacing() {
    let _t = init_tracing();
    // Spacing=1: coords 0,1,2,3 — maximum ancestor sharing.
    let plan = Plan::<u64, u64>::new().interleaved_hotspots(0, 4, 1, 15, 8, 16);
    let g = run_checked::<u64, u64, 4>(low_threshold_config(), &plan, 1);
    assert_invariants(&g);
}

// ── cousin_rivalry ──────────────────────────────────────────────

#[test]
fn cousin_rivalry_n4_low_threshold() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().cousin_rivalry(0, 8, 4, 3, 2, 60);
    let g = run_checked::<u64, u64, 4>(low_threshold_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn cousin_rivalry_n8_deep() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().cousin_rivalry(0, 128, 64, 5, 3, 120);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn cousin_rivalry_narrow_regions() {
    let _t = init_tracing();
    // Narrow width=2 regions at coords {0,1} vs {4,5}
    let plan = Plan::<u64, u64>::new().cousin_rivalry(0, 4, 2, 5, 4, 80);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

// ── adversarial_cousins ─────────────────────────────────────────

#[test]
fn adversarial_cousins_n4_siblings() {
    let _t = init_tracing();
    // Coords 1 and 3: share parent at depth 3 in N=4.
    let plan = Plan::<u64, u64>::new().adversarial_cousins(1, 3, 3, 10, 50, 30, 8);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn adversarial_cousins_n4_far() {
    let _t = init_tracing();
    // Coords 2 and 10: share root-level ancestor in N=4.
    let plan = Plan::<u64, u64>::new().adversarial_cousins(2, 10, 4, 12, 60, 40, 10);
    let g = run_checked::<u64, u64, 4>(low_threshold_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn adversarial_cousins_n8_deep() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().adversarial_cousins(32, 96, 5, 20, 80, 60, 15);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn adversarial_cousins_asymmetric_spikes() {
    let _t = init_tracing();
    // Very asymmetric: one side gets 100, other gets 5.
    let plan = Plan::<u64, u64>::new().adversarial_cousins(0, 8, 3, 15, 100, 5, 12);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

// ── phase_shifted_oscillation ───────────────────────────────────

#[test]
fn phase_shifted_oscillation_n4() {
    let _t = init_tracing();
    // Coords 3 and 4: adjacent, cross a subtree boundary.
    let plan = Plan::<u64, u64>::new().phase_shifted_oscillation(3, 4, 20, 2, 8, 6);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn phase_shifted_oscillation_n8_close() {
    let _t = init_tracing();
    // Coords 127 and 128: straddle the root split.
    let plan = Plan::<u64, u64>::new().phase_shifted_oscillation(127, 128, 30, 3, 10, 8);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// ── neighborhood_burst ──────────────────────────────────────────

#[test]
fn neighborhood_burst_n4_center() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().neighborhood_burst(8, 3, 10, 50);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn neighborhood_burst_n8_wide() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().neighborhood_burst(128, 16, 8, 100);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn neighborhood_burst_edge() {
    let _t = init_tracing();
    // Near domain edge: center=2, radius=5 → lo saturates to 0.
    let plan = Plan::<u64, u64>::new().neighborhood_burst(2, 5, 12, 40);
    let g = run_checked::<u64, u64, 4>(low_threshold_config(), &plan, 1);
    assert_invariants(&g);
}

// =====================================================================
// Group B — Deep / structural patterns
// =====================================================================

// ── fractal_spray ───────────────────────────────────────────────

#[test]
fn fractal_spray_n4_full() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().fractal_spray(16, 10, 4);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn fractal_spray_n8_deep() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().fractal_spray(256, 8, 8);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// ── nested_bursts ───────────────────────────────────────────────

#[test]
fn nested_bursts_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().nested_bursts(16, 10, 4, 5);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn nested_bursts_n8_aggressive() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().nested_bursts(256, 8, 6, 4);
    let g = run_checked::<u64, u64, 8>(aggressive_config(), &plan, 1);
    assert_invariants(&g);
}

// ── depth_ladder ────────────────────────────────────────────────

#[test]
fn depth_ladder_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().depth_ladder(4, 10, 8);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn depth_ladder_n8() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().depth_ladder(8, 8, 5);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// ── staircase ───────────────────────────────────────────────────

#[test]
fn staircase_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().staircase(2, 6, 5, 5);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn staircase_n8_wide() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().staircase(16, 8, 4, 6);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// ── cascade_shift ───────────────────────────────────────────────

#[test]
fn cascade_shift_n4_siblings() {
    let _t = init_tracing();
    // Coords 1 and 2: adjacent siblings.
    let plan = Plan::<u64, u64>::new().cascade_shift(1, 2, 15, 20, 30);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn cascade_shift_n8_halves() {
    let _t = init_tracing();
    // Root-level shift: 64 → 192.
    let plan = Plan::<u64, u64>::new().cascade_shift(64, 192, 10, 30, 40);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// =====================================================================
// Group C — Coverage / topology patterns
// =====================================================================

// ── diamond ─────────────────────────────────────────────────────

#[test]
fn diamond_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().diamond(16, 8, 60);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn diamond_n8() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().diamond(256, 6, 200);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// ── bit_flip_walk ───────────────────────────────────────────────

#[test]
fn bit_flip_walk_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().bit_flip_walk(16, 10, 48);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn bit_flip_walk_n8() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().bit_flip_walk(256, 6, 256);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// ── sibling_flood ───────────────────────────────────────────────

#[test]
fn sibling_flood_n4_equal() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().sibling_flood(16, 10, 10, 30);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn sibling_flood_n8_asymmetric() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().sibling_flood(256, 5, 20, 40);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// ── mirrored_growth ─────────────────────────────────────────────

#[test]
fn mirrored_growth_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().mirrored_growth(16, 8, 40);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn mirrored_growth_n8() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().mirrored_growth(256, 6, 80);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// ── checkerboard ────────────────────────────────────────────────

#[test]
fn checkerboard_n4_equal() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().checkerboard(16, 10, 10, 60);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn checkerboard_n8_asymmetric() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().checkerboard(256, 5, 15, 100);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// ── pincer ──────────────────────────────────────────────────────

#[test]
fn pincer_n4_full_domain() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().pincer(0, 15, 10, 3);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn pincer_n8_narrow() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().pincer(100, 155, 8, 4);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// ── repulsion_walk ──────────────────────────────────────────────

#[test]
fn repulsion_walk_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().repulsion_walk(8, 10, 7, 4);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn repulsion_walk_n8() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().repulsion_walk(128, 8, 30, 3);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// ── sawtooth ────────────────────────────────────────────────────

#[test]
fn sawtooth_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().sawtooth(16, 3, 5);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn sawtooth_n8() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().sawtooth(64, 2, 4);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// ── centroid_drift ──────────────────────────────────────────────

#[test]
fn centroid_drift_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().centroid_drift(0, 15, 10, 5);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn centroid_drift_n8() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().centroid_drift(50, 200, 6, 3);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// ── pulse_decay ─────────────────────────────────────────────────

#[test]
fn pulse_decay_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().pulse_decay(8, 16, 50, 8, 6);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn pulse_decay_n8() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().pulse_decay(128, 256, 80, 12, 8);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// ── golden_spiral ───────────────────────────────────────────────

#[test]
fn golden_spiral_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().golden_spiral(16, 8, 60);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn golden_spiral_n8() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().golden_spiral(256, 6, 200);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// =====================================================================
// Combination / multi-phase stress tests
// =====================================================================

#[test]
fn multi_topology_stress_n4() {
    use torrust_mudlark::testing::plan_multi_topology_stress;
    let _t = init_tracing();
    let plan = plan_multi_topology_stress(16);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn multi_topology_stress_n8() {
    use torrust_mudlark::testing::plan_multi_topology_stress;
    let _t = init_tracing();
    let plan = plan_multi_topology_stress(256);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

/// Chain **all** targeted patterns in sequence.
#[test]
fn all_targeted_patterns_combined_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new()
        .interleaved_hotspots(0, 4, 2, 10, 4, 8)
        .cousin_rivalry(0, 8, 4, 3, 2, 40)
        .adversarial_cousins(1, 3, 3, 8, 50, 30, 6)
        .phase_shifted_oscillation(3, 4, 20, 2, 6, 4)
        .neighborhood_burst(8, 3, 10, 30);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

/// Chain **all** structural patterns in sequence.
#[test]
fn all_structural_patterns_combined_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new()
        .fractal_spray(16, 10, 4)
        .nested_bursts(16, 8, 3, 4)
        .depth_ladder(4, 10, 5)
        .staircase(2, 5, 3, 5)
        .cascade_shift(1, 14, 10, 15, 20);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

/// Chain **all** coverage patterns in sequence.
#[test]
fn all_coverage_patterns_combined_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new()
        .diamond(16, 6, 30)
        .bit_flip_walk(16, 8, 32)
        .sibling_flood(16, 8, 12, 15)
        .mirrored_growth(16, 6, 20)
        .checkerboard(16, 8, 10, 40)
        .pincer(0, 15, 8, 2)
        .repulsion_walk(8, 8, 7, 3)
        .sawtooth(16, 3, 3)
        .centroid_drift(0, 15, 8, 3)
        .pulse_decay(8, 16, 40, 6, 4)
        .golden_spiral(16, 6, 30);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

/// The ultimate stress test: every single pattern chained, N=8,
/// aggressive config.
#[test]
fn everything_combined_n8_aggressive() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new()
        // Targeted
        .interleaved_hotspots(0, 8, 16, 8, 4, 10)
        .cousin_rivalry(0, 128, 32, 4, 2, 60)
        .adversarial_cousins(32, 96, 4, 10, 60, 40, 8)
        .phase_shifted_oscillation(63, 64, 20, 3, 6, 4)
        .neighborhood_burst(128, 8, 8, 40)
        // Structural
        .fractal_spray(256, 8, 6)
        .nested_bursts(256, 6, 4, 3)
        .depth_ladder(8, 6, 3)
        .staircase(16, 6, 3, 4)
        .cascade_shift(64, 192, 8, 15, 25)
        // Coverage
        .diamond(256, 5, 60)
        .bit_flip_walk(256, 5, 80)
        .sibling_flood(256, 5, 10, 20)
        .mirrored_growth(256, 4, 30)
        .checkerboard(256, 5, 8, 60)
        .pincer(50, 205, 6, 2)
        .repulsion_walk(128, 6, 20, 2)
        .sawtooth(32, 2, 3)
        .centroid_drift(100, 155, 5, 2)
        .pulse_decay(128, 256, 30, 8, 4)
        .golden_spiral(256, 5, 60);
    let g = run_checked::<u64, u64, 8>(aggressive_config(), &plan, 1);
    assert_invariants(&g);
}

// =====================================================================
// Cross-config tests — same pattern under many configs
// =====================================================================

/// Run a pattern across all available configs.
fn run_across_configs<const N: u32>(plan: &Plan<u64, u64>) {
    use torrust_mudlark::testing::default_config;

    let configs: Vec<(&str, torrust_mudlark::Config<u64>)> = vec![
        ("default", default_config()),
        ("low_threshold", low_threshold_config()),
        ("cascade", cascade_config()),
        ("deep", deep_config()),
        ("aggressive", aggressive_config()),
        ("wide_shallow", wide_shallow_config()),
    ];
    for (_name, config) in configs {
        let g = run_checked::<u64, u64, N>(config, plan, 1);
        assert_invariants(&g);
    }
}

#[test]
fn cousin_rivalry_all_configs_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().cousin_rivalry(0, 8, 4, 5, 3, 40);
    run_across_configs::<4>(&plan);
}

#[test]
fn adversarial_cousins_all_configs_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().adversarial_cousins(1, 3, 3, 8, 40, 25, 6);
    run_across_configs::<4>(&plan);
}

#[test]
fn fractal_spray_all_configs_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().fractal_spray(16, 8, 4);
    run_across_configs::<4>(&plan);
}

#[test]
fn diamond_all_configs_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().diamond(16, 8, 40);
    run_across_configs::<4>(&plan);
}

#[test]
fn golden_spiral_all_configs_n8() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().golden_spiral(256, 6, 100);
    run_across_configs::<8>(&plan);
}

// =====================================================================
// Energy conservation spot-checks
// =====================================================================

#[test]
fn energy_conserved_interleaved_hotspots() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().interleaved_hotspots(0, 4, 2, 10, 5, 8);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    let expected: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), expected, "energy conservation violated");
}

#[test]
fn energy_conserved_fractal_spray() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().fractal_spray(16, 10, 4);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    let expected: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), expected, "energy conservation violated");
}

#[test]
fn energy_conserved_pincer() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().pincer(0, 15, 8, 3);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    let expected: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), expected, "energy conservation violated");
}

#[test]
fn energy_conserved_everything_combined() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new()
        .interleaved_hotspots(0, 4, 2, 8, 3, 6)
        .cousin_rivalry(0, 8, 4, 3, 2, 20)
        .fractal_spray(16, 8, 4)
        .diamond(16, 6, 20)
        .checkerboard(16, 8, 10, 30)
        .golden_spiral(16, 6, 20);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    let expected: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), expected, "energy conservation violated");
}
