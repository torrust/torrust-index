// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Mirror tests for Family 2 — Query benchmarks.

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::*;

use crate::init_tracing;
use crate::support::TestRng;

#[test]
fn mirror_query_get() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().spread(256, 6, 5_000);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 500);
    let _ = g.get(128);
    assert_invariants(&g);
}

#[test]
fn mirror_query_range_sum() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().spread(256, 6, 11_000);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 1_100);
    let _ = g.range_sum(32..192);
    assert_invariants(&g);
}

#[test]
fn mirror_query_sample() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().spread(256, 6, 9_000);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 900);
    let mut rng = TestRng(42);
    let _ = g.sample(&mut rng);
    assert_invariants(&g);
}

#[test]
fn mirror_query_sample_skewed() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().skewed(256, 20_000);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 2_000);
    let mut rng = TestRng(42);
    let _ = g.sample(&mut rng);
    assert_invariants(&g);
}

#[test]
fn mirror_query_sample_hotspot() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().hotspot(128, 10, 28_000);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 2_800);
    let mut rng = TestRng(42);
    let _ = g.sample(&mut rng);
    assert_invariants(&g);
}
