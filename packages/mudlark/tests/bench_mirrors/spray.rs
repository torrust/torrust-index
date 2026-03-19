// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Mirror tests for Family 5 — Spray benchmarks.

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::*;

use crate::init_tracing;
use crate::support::TestRng;

#[test]
fn mirror_spray_cold_start() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().random_spray(42, 256, 6, 18_000);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 1_800);
}

#[test]
fn mirror_spray_steady_state() {
    let _t = init_tracing();

    let warmup = Plan::<u64, u64>::new().random_spray(42, 256, 6, 750);
    let spray = Plan::<u64, u64>::new().random_spray(99, 256, 6, 750);
    let mut g = run_checked::<u64, u64, 8>(budget_config(500), &warmup, 75);
    for &(coord, delta) in &spray.observations {
        g.observe(coord, delta);
    }
    assert_invariants(&g);
}

#[test]
fn mirror_spray_interleave() {
    let _t = init_tracing();

    let warmup = Plan::<u64, u64>::new().random_spray(42, 256, 6, 750);
    let spray = Plan::<u64, u64>::new().random_spray(99, 256, 6, 750);
    let mut g = run_checked::<u64, u64, 8>(budget_config(500), &warmup, 75);
    let mut rng = TestRng(123);
    for &(coord, delta) in &spray.observations {
        g.observe(coord, delta);
        let _ = g.sample(&mut rng);
    }
    assert_invariants(&g);
}

#[test]
fn mirror_spray_scaling_budget() {
    let _t = init_tracing();

    for budget in [200, 500, 2_000, 10_000] {
        let plan = Plan::<u64, u64>::new().random_spray(42, 256, 6, 450);
        let _g = run_checked::<u64, u64, 8>(budget_config(budget), &plan, 45);
    }
}
