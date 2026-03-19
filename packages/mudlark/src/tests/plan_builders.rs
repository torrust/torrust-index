// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use crate::testing::Plan;

#[test]
fn random_spray_produces_expected_count() {
    let plan = Plan::<u64, u64>::new().random_spray(42, 256, 6, 100);
    assert_eq!(plan.len(), 100);
}

#[test]
fn random_spray_is_deterministic() {
    let a = Plan::<u64, u64>::new().random_spray(42, 256, 6, 100);
    let b = Plan::<u64, u64>::new().random_spray(42, 256, 6, 100);
    assert_eq!(a, b);
}

#[test]
fn random_spray_different_seeds_differ() {
    let a = Plan::<u64, u64>::new().random_spray(42, 256, 6, 100);
    let b = Plan::<u64, u64>::new().random_spray(99, 256, 6, 100);
    assert_ne!(a, b);
}

#[test]
fn random_spray_coords_within_domain() {
    let plan = Plan::<u64, u64>::new().random_spray(42, 256, 6, 200);
    for &(coord, _) in &plan.observations {
        assert!(coord < 256, "coord {coord} out of domain 256");
    }
}

#[test]
fn oscillating_hotspot_produces_expected_count() {
    let plan = Plan::<u64, u64>::new().oscillating_hotspot(0, 255, 10, 5, 4);
    assert_eq!(plan.len(), 20); // 5 burst × 4 cycles
}

#[test]
fn oscillating_hotspot_alternates_targets() {
    let plan = Plan::<u64, u64>::new().oscillating_hotspot(0, 255, 10, 3, 4);
    // cycle 0 → coord 0, cycle 1 → coord 255, cycle 2 → coord 0, cycle 3 → coord 255
    assert_eq!(plan.observations[0].0, 0);
    assert_eq!(plan.observations[3].0, 255);
    assert_eq!(plan.observations[6].0, 0);
    assert_eq!(plan.observations[9].0, 255);
}

#[test]
fn oscillating_hotspot_zero_cycles_is_empty() {
    let plan = Plan::<u64, u64>::new().oscillating_hotspot(0, 255, 10, 5, 0);
    assert!(plan.is_empty());
}

#[test]
fn random_spray_chains_with_other_builders() {
    let plan = Plan::<u64, u64>::new()
        .spread(64, 1, 10)
        .random_spray(7, 128, 2, 20)
        .oscillating_hotspot(0, 63, 5, 4, 3);
    assert_eq!(plan.len(), 10 + 20 + 12);
}
