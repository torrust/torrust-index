// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Mirror tests for Family 6 — Pathological benchmarks.

use torrust_mudlark::testing::*;

use crate::init_tracing;

#[test]
fn mirror_pathological_left_deep() {
    let _t = init_tracing();

    let plan = plan_left_deep(6, 14_000);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 1_400);
}

#[test]
fn mirror_pathological_right_deep() {
    let _t = init_tracing();

    let plan = plan_right_deep(256, 6, 12_000);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 1_200);
}

#[test]
fn mirror_pathological_adversarial_zigzag() {
    let _t = init_tracing();

    let plan = plan_adversarial(256, 8_500);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 850);
}

#[test]
fn mirror_pathological_budget_burst() {
    let _t = init_tracing();

    let plan = plan_budget_burst(256, 1_000, 200);
    let _g = run_checked::<u64, u64, 8>(budget_config(500), &plan, 120);
}

#[test]
fn mirror_pathological_growth_pressure_relax() {
    let _t = init_tracing();

    let plan = plan_growth_pressure_relax(256, 2_200);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 770);
}

#[test]
fn mirror_pathological_single_hotspot() {
    let _t = init_tracing();

    let plan = plan_single_hotspot(128, 6, 8_000);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 800);
}

#[test]
fn mirror_pathological_oscillating_hotspot() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().oscillating_hotspot(0, 255, 10, 440, 20);
    let _g = run_checked::<u64, u64, 8>(budget_config(500), &plan, 440);
}

#[test]
fn mirror_pathological_tight_budget_spray() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().random_spray(42, 256, 6, 26_000);
    let _g = run_checked::<u64, u64, 8>(tight_budget_config(), &plan, 2_600);
}

#[test]
fn mirror_pathological_cousin_rivalry() {
    let _t = init_tracing();

    let plan = plan_cousin_rivalry(2_000);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 200);
}

#[test]
fn mirror_pathological_phase_shifted() {
    let _t = init_tracing();

    let plan = plan_phase_shifted(32, 96, 4_000);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 400);
}

#[test]
fn mirror_pathological_fractal_fill() {
    let _t = init_tracing();

    let plan = plan_fractal_fill(256, 7);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 10);
}

#[test]
fn mirror_pathological_diamond() {
    let _t = init_tracing();

    let plan = plan_diamond(256, 5_000);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 500);
}

#[test]
fn mirror_pathological_gray_code() {
    let _t = init_tracing();

    let plan = plan_gray_code(256, 5_000);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 500);
}

#[test]
fn mirror_pathological_centroid_drift() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().centroid_drift(0, 200, 8, 20);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 200);
}

#[test]
fn mirror_pathological_golden_spiral() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().golden_spiral(256, 6, 10_000);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 1_000);
}

#[test]
fn mirror_pathological_pincer() {
    let _t = init_tracing();

    let plan = Plan::<u64, u64>::new().pincer(0, 255, 6, 10);
    let _g = run_checked::<u64, u64, 8>(default_config(), &plan, 100);
}
