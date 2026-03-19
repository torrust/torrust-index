// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

// ── Plan Runners ────────────────────────────────────────────────
//
//  Each runner applies a `Plan` to a fresh `GvGraph`, optionally
//  checking invariants or budget constraints at a configurable
//  interval.

use std::fmt::Display;

use super::plan::Plan;
use crate::graph::{Config, GvGraph};
use crate::invariants::{assert_invariants, check_all_invariants};
use crate::traits::{Accumulator, Coordinate, Inspectable};

/// Invariant violations keyed by 1-based observation step.
pub type SoftErrors = Vec<(usize, Vec<String>)>;

// ── Unchecked run ───────────────────────────────────────────────

/// Apply a plan to a fresh graph.
#[must_use]
pub fn run<C, V, const N: u32>(config: Config<V>, plan: &Plan<C, V>) -> GvGraph<C, V, N>
where
    C: Coordinate,
    V: Accumulator + Inspectable,
{
    let mut g = GvGraph::new(config);
    for &(coord, delta) in &plan.observations {
        g.observe(coord, delta);
    }
    g
}

// ── Invariant-checked run ───────────────────────────────────────

/// Apply a plan, asserting invariants every `check_every`
/// observations.
///
/// # Panics
///
/// Panics if any invariant is violated at a check-point.
#[must_use]
pub fn run_checked<C, V, const N: u32>(config: Config<V>, plan: &Plan<C, V>, check_every: usize) -> GvGraph<C, V, N>
where
    C: Coordinate,
    V: Accumulator + Inspectable,
{
    let k = check_every.max(1);
    let mut g = GvGraph::new(config);
    for (i, &(coord, delta)) in plan.observations.iter().enumerate() {
        g.observe(coord, delta);
        if (i + 1) % k == 0 {
            assert_invariants(&g);
        }
    }
    g
}

// ── Soft-checked run ────────────────────────────────────────────

/// Apply a plan, collecting invariant violations every `check_every`
/// observations **without** panicking.
///
/// Returns `(graph, errors)` where `errors` is a list of
/// `(step, violations)` pairs.  The caller can dump diagnostics
/// before asserting.
#[must_use]
pub fn run_soft_checked<C, V, const N: u32>(
    config: Config<V>,
    plan: &Plan<C, V>,
    check_every: usize,
) -> (GvGraph<C, V, N>, SoftErrors)
where
    C: Coordinate,
    V: Accumulator + Inspectable,
{
    let k = check_every.max(1);
    let mut g = GvGraph::new(config);
    let mut all_errors = Vec::new();
    for (i, &(coord, delta)) in plan.observations.iter().enumerate() {
        g.observe(coord, delta);
        if (i + 1) % k == 0 {
            let errs = check_all_invariants(&g);
            if !errs.is_empty() {
                all_errors.push((i + 1, errs));
            }
        }
    }
    (g, all_errors)
}

// ── Budget-checked run ──────────────────────────────────────────

/// Apply a plan, asserting `node_count <= budget` on every
/// observation **and** invariants every `check_every` steps.
///
/// # Panics
///
/// Panics on budget violation or invariant violation.
#[must_use]
pub fn run_budget_checked<C, V, const N: u32>(
    config: Config<V>,
    plan: &Plan<C, V>,
    budget: usize,
    check_every: usize,
) -> GvGraph<C, V, N>
where
    C: Coordinate + Display,
    V: Accumulator + Inspectable + Display,
{
    let k = check_every.max(1);
    let mut g = GvGraph::new(config);
    for (i, &(coord, delta)) in plan.observations.iter().enumerate() {
        g.observe(coord, delta);
        assert!(
            g.node_count() as usize <= budget,
            "hard budget violated at observation {i} (coord={coord}, \
             delta={delta}): node_count={}, budget={budget}",
            g.node_count()
        );
        if (i + 1) % k == 0 {
            assert_invariants(&g);
        }
    }
    g
}
