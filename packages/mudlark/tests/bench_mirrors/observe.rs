// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Mirror tests for Family 1 — Observe benchmarks.

use torrust_mudlark::testing::*;

use crate::init_tracing;

#[test]
fn mirror_observe_spread() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().spread(256, 6, 8_000);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 800);
}

#[test]
fn mirror_observe_spread_budgeted() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().spread(256, 6, 1_400);
    let _g = run_checked::<u64, u64, 8>(budget_config(500), &plan, 140);
}

#[test]
fn mirror_observe_skewed() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().skewed(256, 12_000);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 1_200);
}

#[test]
fn mirror_observe_deep() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().sweep(256, 3_000);
    let _g = run_checked::<u64, u64, 8>(deep_config(), &plan, 300);
}

#[test]
fn mirror_observe_zigzag() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().zigzag(0, 255, 6, 8_000);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 800);
}

#[test]
fn mirror_observe_diamond() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().diamond(256, 6, 8_000);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 800);
}

#[test]
fn mirror_observe_golden_spiral() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().golden_spiral(256, 6, 8_000);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 800);
}
