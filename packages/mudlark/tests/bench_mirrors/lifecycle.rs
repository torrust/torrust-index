// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Mirror tests for Family 4 — Lifecycle benchmarks (decay + eviction).

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::*;

use crate::init_tracing;

#[test]
fn mirror_lifecycle_decay() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().spread(256, 6, 8_000);
    let mut g = run_checked::<u64, u64, 8>(default_config(), &plan, 800);
    let root = g.g_root();
    g.decay(root, 0.95, 0.5);
    assert_invariants(&g);
}

#[test]
fn mirror_lifecycle_eviction() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().spread(256, 6, 900);
    let mut g = run_checked::<u64, u64, 8>(budget_config(500), &plan, 90);
    g.check_evictions();
    assert_invariants(&g);
}

#[test]
fn mirror_lifecycle_decay_evict() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().spread(256, 6, 1_000);
    let mut g = run_checked::<u64, u64, 8>(budget_config(500), &plan, 100);
    let root = g.g_root();
    g.decay(root, 0.95, 0.5);
    g.check_evictions();
    assert_invariants(&g);
}

#[test]
fn mirror_lifecycle_decay_selective() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().spread(256, 6, 2_000);
    for q in [0.0, 0.25, 0.5, 0.75, 1.0] {
        let mut g = run_checked::<u64, u64, 8>(default_config(), &plan, 200);
        let root = g.g_root();
        g.decay(root, 0.95, q);
        assert_invariants(&g);
    }
}

#[test]
fn mirror_lifecycle_decay_skewed() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().skewed(256, 8_000);
    let mut g = run_checked::<u64, u64, 8>(default_config(), &plan, 800);
    let root = g.g_root();
    g.decay(root, 0.95, 0.5);
    assert_invariants(&g);
}

#[test]
fn mirror_lifecycle_eviction_skewed() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().skewed(256, 900);
    let mut g = run_checked::<u64, u64, 8>(budget_config(500), &plan, 90);
    g.check_evictions();
    assert_invariants(&g);
}
