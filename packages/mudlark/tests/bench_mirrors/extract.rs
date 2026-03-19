// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Mirror tests for Family 3 — Extract benchmarks.

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::*;

use crate::init_tracing;

#[test]
fn mirror_extract_full() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().spread(256, 6, 7_500);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 750);
    drop(g.extract());
    assert_invariants(&g);
}

#[test]
fn mirror_extract_layers() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().spread(256, 6, 8_000);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 800);
    let _ = g.layers().count();
    assert_invariants(&g);
}

#[test]
fn mirror_extract_plateaus() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().spread(256, 6, 5_000);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 500);
    drop(g.plateaus());
    assert_invariants(&g);
}

#[test]
fn mirror_extract_full_skewed() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().skewed(256, 7_500);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 750);
    drop(g.extract());
    assert_invariants(&g);
}

#[test]
fn mirror_extract_full_hotspot() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().hotspot(128, 6, 7_500);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 750);
    drop(g.extract());
    assert_invariants(&g);
}
