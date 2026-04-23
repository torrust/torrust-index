// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Stress tests for diverse observation patterns.
//!
//! Exercises all 21 observation patterns from `Plan`'s Targeted,
//! Structural, and Coverage sections across multiple configs and
//! domain sizes.  Every test uses `run_checked` with
//! `check_every = 1` so invariants are validated after **every**
//! observation — maximising the chance of catching transient
//! violations (including source 10).
//!
//! # Test index
//!
//! ## Group A — Targeted patterns
//!
//! | Test | Focus |
//! |------|-------|
//! | [`interleaved_hotspots_n4_cascade`] | N=4 cascade config, spacing=2 |
//! | [`interleaved_hotspots_n8_aggressive`] | N=8 aggressive config, spacing=16 |
//! | [`interleaved_hotspots_tight_spacing`] | spacing=1 — maximum ancestor sharing |
//! | [`cousin_rivalry_n4_low_threshold`] | N=4 low-threshold config |
//! | [`cousin_rivalry_n8_deep`] | N=8 deep config, wide regions |
//! | [`cousin_rivalry_narrow_regions`] | narrow width=2 regions at {0,1} vs {4,5} |
//! | [`adversarial_cousins_n4_siblings`] | siblings sharing parent at depth 3 |
//! | [`adversarial_cousins_n4_far`] | root-level ancestor sharing (2 vs 10) |
//! | [`adversarial_cousins_n8_deep`] | N=8 deep config |
//! | [`adversarial_cousins_asymmetric_spikes`] | very asymmetric energy (100 vs 5) |
//! | [`phase_shifted_oscillation_n4`] | adjacent coords crossing subtree boundary |
//! | [`phase_shifted_oscillation_n8_close`] | straddling the root split (127 vs 128) |
//! | [`phase_shifted_oscillation_n4_aggressive`] | immediate splitting, out-of-phase neighbours |
//! | [`neighborhood_burst_n4_center`] | centred burst, N=4 cascade |
//! | [`neighborhood_burst_n8_wide`] | wide radius=16, N=8 deep |
//! | [`neighborhood_burst_edge`] | near domain edge — lo saturates to 0 |
//!
//! ## Group B — Deep / structural patterns
//!
//! | Test | Focus |
//! |------|-------|
//! | [`fractal_spray_n4_full`] | full domain N=4 cascade |
//! | [`fractal_spray_n8_deep`] | N=8 deep config |
//! | [`nested_bursts_n4`] | N=4 cascade |
//! | [`nested_bursts_n8_aggressive`] | N=8 aggressive config |
//! | [`depth_ladder_n4`] | depth-spine walk, N=4 cascade |
//! | [`depth_ladder_n8`] | depth-spine walk, N=8 deep |
//! | [`depth_ladder_n4_low_threshold`] | low threshold → frequent splits down the spine |
//! | [`staircase_n4`] | step=2, N=4 cascade |
//! | [`staircase_n8_wide`] | wide step=16, N=8 deep |
//! | [`cascade_shift_n4_siblings`] | adjacent siblings (1→2) |
//! | [`cascade_shift_n8_halves`] | root-level shift (64→192) |
//!
//! ## Group C — Coverage / topology patterns
//!
//! | Test | Focus |
//! |------|-------|
//! | [`diamond_n4`] | diamond topology, N=4 cascade |
//! | [`diamond_n8`] | diamond topology, N=8 deep |
//! | [`bit_flip_walk_n4`] | Hamming-distance walk, N=4 |
//! | [`bit_flip_walk_n8`] | Hamming-distance walk, N=8 |
//! | [`sibling_flood_n4_equal`] | equal-energy siblings, N=4 |
//! | [`sibling_flood_n8_asymmetric`] | asymmetric energy siblings, N=8 |
//! | [`mirrored_growth_n4`] | symmetric growth, N=4 cascade |
//! | [`mirrored_growth_n8`] | symmetric growth, N=8 deep |
//! | [`mirrored_growth_n4_wide_shallow`] | symmetric growth, wide-shallow config |
//! | [`checkerboard_n4_equal`] | equal-energy alternating, N=4 |
//! | [`checkerboard_n8_asymmetric`] | asymmetric alternating, N=8 |
//! | [`checkerboard_n4_aggressive`] | immediate splitting on even/odd coords |
//! | [`pincer_n4_full_domain`] | converging fronts, full domain 0–15 |
//! | [`pincer_n8_narrow`] | converging fronts, narrow range 100–155 |
//! | [`pincer_n4_aggressive`] | converging fronts with aggressive splitting |
//! | [`repulsion_walk_n4`] | repulsion walk, N=4 cascade |
//! | [`repulsion_walk_n8`] | repulsion walk, N=8 deep |
//! | [`sawtooth_n4`] | escalating sweeps, N=4 cascade |
//! | [`sawtooth_n8`] | escalating sweeps, N=8 deep |
//! | [`sawtooth_n4_aggressive`] | escalating sweeps, aggressive (θ=1) |
//! | [`centroid_drift_n4`] | drifting centroid, full domain |
//! | [`centroid_drift_n8`] | drifting centroid, 50–200 |
//! | [`pulse_decay_n4`] | pulse-decay pattern, N=4 cascade |
//! | [`pulse_decay_n8`] | pulse-decay pattern, N=8 deep |
//! | [`golden_spiral_n4`] | golden-ratio spiral, N=4 |
//! | [`golden_spiral_n8`] | golden-ratio spiral, N=8 |
//!
//! ## Combination / multi-phase
//!
//! | Test | Focus |
//! |------|-------|
//! | [`multi_topology_stress_n4`] | pre-built multi-topology plan, N=4 cascade |
//! | [`multi_topology_stress_n8`] | pre-built multi-topology plan, N=8 deep |
//! | [`all_targeted_patterns_combined_n4`] | all 5 targeted patterns chained, N=4 |
//! | [`all_structural_patterns_combined_n4`] | all 5 structural patterns chained, N=4 |
//! | [`all_coverage_patterns_combined_n4`] | all 11 coverage patterns chained, N=4 |
//! | [`everything_combined_n8_aggressive`] | every pattern chained, N=8 aggressive |
//!
//! ## Cross-config (pattern × 6 configs)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`interleaved_hotspots_all_configs_n4`] | interleaved hotspots across 6 configs |
//! | [`cousin_rivalry_all_configs_n4`] | cousin rivalry across 6 configs |
//! | [`adversarial_cousins_all_configs_n4`] | adversarial cousins across 6 configs |
//! | [`neighborhood_burst_all_configs_n4`] | neighbourhood burst across 6 configs |
//! | [`fractal_spray_all_configs_n4`] | fractal spray across 6 configs |
//! | [`diamond_all_configs_n4`] | diamond across 6 configs |
//! | [`checkerboard_all_configs_n4`] | checkerboard across 6 configs |
//! | [`sawtooth_all_configs_n4`] | sawtooth across 6 configs |
//! | [`golden_spiral_all_configs_n8`] | golden spiral across 6 configs, N=8 |
//!
//! ## Energy conservation
//!
//! | Test | Focus |
//! |------|-------|
//! | [`energy_conserved_interleaved_hotspots`] | `total_sum` == Σ observations |
//! | [`energy_conserved_fractal_spray`] | `total_sum` == Σ observations |
//! | [`energy_conserved_pincer`] | `total_sum` == Σ observations |
//! | [`energy_conserved_depth_ladder`] | `total_sum` == Σ observations |
//! | [`energy_conserved_sawtooth`] | `total_sum` == Σ observations |
//! | [`energy_conserved_mirrored_growth`] | `total_sum` == Σ observations |
//! | [`energy_conserved_everything_combined`] | `total_sum` == Σ observations, 6 patterns chained |

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

#[test]
fn phase_shifted_oscillation_n4_aggressive() {
    let _t = init_tracing();
    // Immediate splitting with out-of-phase neighbors.
    let plan = Plan::<u64, u64>::new().phase_shifted_oscillation(3, 4, 25, 2, 10, 8);
    let g = run_checked::<u64, u64, 4>(aggressive_config(), &plan, 1);
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

#[test]
fn depth_ladder_n4_low_threshold() {
    let _t = init_tracing();
    // Low threshold → frequent splits walking down the spine.
    let plan = Plan::<u64, u64>::new().depth_ladder(4, 6, 10);
    let g = run_checked::<u64, u64, 4>(low_threshold_config(), &plan, 1);
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

#[test]
fn mirrored_growth_n4_wide_shallow() {
    let _t = init_tracing();
    // Symmetric growth in a wide, shallow tree.
    let plan = Plan::<u64, u64>::new().mirrored_growth(16, 10, 50);
    let g = run_checked::<u64, u64, 4>(wide_shallow_config(), &plan, 1);
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

#[test]
fn checkerboard_n4_aggressive() {
    let _t = init_tracing();
    // Immediate splitting on adjacent even/odd coords.
    let plan = Plan::<u64, u64>::new().checkerboard(16, 8, 12, 80);
    let g = run_checked::<u64, u64, 4>(aggressive_config(), &plan, 1);
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

#[test]
fn pincer_n4_aggressive() {
    let _t = init_tracing();
    // Converging fronts with immediate splitting.
    let plan = Plan::<u64, u64>::new().pincer(0, 15, 8, 4);
    let g = run_checked::<u64, u64, 4>(aggressive_config(), &plan, 1);
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

#[test]
fn sawtooth_n4_aggressive() {
    let _t = init_tracing();
    // Escalating sweeps with θ=1.
    let plan = Plan::<u64, u64>::new().sawtooth(16, 3, 6);
    let g = run_checked::<u64, u64, 4>(aggressive_config(), &plan, 1);
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
    for (name, config) in configs {
        let _span = tracing::info_span!("config", name).entered();
        let g = run_checked::<u64, u64, N>(config, plan, 1);
        assert_invariants(&g);
    }
}

#[test]
fn interleaved_hotspots_all_configs_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().interleaved_hotspots(0, 4, 2, 8, 4, 10);
    run_across_configs::<4>(&plan);
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
fn neighborhood_burst_all_configs_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().neighborhood_burst(8, 3, 8, 40);
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
fn checkerboard_all_configs_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().checkerboard(16, 8, 10, 50);
    run_across_configs::<4>(&plan);
}

#[test]
fn sawtooth_all_configs_n4() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().sawtooth(16, 3, 4);
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
fn energy_conserved_depth_ladder() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().depth_ladder(4, 10, 8);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    let expected: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), expected, "energy conservation violated");
}

#[test]
fn energy_conserved_sawtooth() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().sawtooth(16, 3, 5);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    let expected: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), expected, "energy conservation violated");
}

#[test]
fn energy_conserved_mirrored_growth() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new().mirrored_growth(16, 8, 40);
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
