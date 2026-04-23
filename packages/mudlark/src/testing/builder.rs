// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

// ── Fluent Builder ──────────────────────────────────────────────
//
//  `GraphCreator` wraps a `Plan` + `Config` and delegates every
//  observation-pattern method to `Plan` via the `delegate!` macro,
//  so adding a new pattern to `Plan` automatically surfaces here.

use std::fmt::Debug;

use super::config::{budget_config, default_config};
use super::plan::Plan;
use super::runner::{run, run_checked};
use crate::graph::{Config, GvGraph};
use crate::traits::{Accumulator, Coordinate, Inspectable};

/// Fluent builder that accumulates a [`Plan<C, V>`] and an
/// associated [`Config<V>`], then produces either the raw plan or a
/// fully-built graph.
pub struct GraphCreator<C: Copy + Debug, V: Accumulator + Inspectable> {
    config: Config<V>,
    plan: Plan<C, V>,
    check_invariants_every: Option<usize>,
}

// ── Delegation macro ────────────────────────────────────────────
//
// Each invocation generates a `#[must_use]` builder method that
// forwards to the identically-named method on `Plan`, threading
// `self` through.

/// Generate a builder method that delegates to `Plan::$method`.
macro_rules! delegate {
    // Simple: no extra generics, just typed params.
    (
        $(#[$meta:meta])*
        fn $method:ident ( $($param:ident : $ty:ty),* $(,)? )
    ) => {
        $(#[$meta])*
        #[must_use]
        pub fn $method(mut self, $($param: $ty),*) -> Self {
            self.plan = self.plan.$method($($param),*);
            self
        }
    };
}

impl<C: Coordinate, V: Accumulator + Inspectable> GraphCreator<C, V> {
    /// Start building with the given config.
    #[must_use]
    pub const fn new(config: Config<V>) -> Self {
        Self {
            config,
            plan: Plan::new(),
            check_invariants_every: None,
        }
    }

    // ── Observation phases (delegated to Plan) ──────────────

    delegate! {
        /// Single observation.
        fn observe(coord: C, delta: V)
    }
    delegate! {
        /// Repeat the same observation `n` times.
        fn observe_n(coord: C, delta: V, n: usize)
    }
    delegate! {
        /// Concentrated hotspot.
        fn hotspot(coord: C, delta: V, n: usize)
    }
    delegate! {
        /// Uniform spread across `[0, domain)`.
        fn spread(domain: u64, delta: V, n: usize)
    }
    delegate! {
        /// Monotone sweep with varying intensity.
        fn sweep(domain: u64, n: usize)
    }
    delegate! {
        /// Alternating extremes.
        fn zigzag(lo: C, hi: C, delta: V, n: usize)
    }
    delegate! {
        /// Left-biased zigzag with escalating intensity.
        fn adversarial_zigzag(lo: C, hi: C, base: V, step: V, n: usize)
    }
    delegate! {
        /// Power-law skew.
        fn skewed(domain: u64, n: usize)
    }
    delegate! {
        /// Spread phase followed by a hotspot burst.
        fn burst(
            domain: u64,
            spread_delta: V,
            n_spread: usize,
            hotspot: C,
            hot_delta: V,
            n_hot: usize,
        )
    }
    delegate! {
        /// Uniform random spray.
        fn random_spray(seed: u64, domain: u64, delta: V, n: usize)
    }
    delegate! {
        /// Oscillating hotspot.
        fn oscillating_hotspot(a: C, b: C, delta: V, burst: usize, cycles: usize)
    }

    // ── Targeted patterns ───────────────────────────────────

    delegate! {
        /// Interleaved multi-hotspot.
        fn interleaved_hotspots(
            start: u64,
            count: u64,
            spacing: u64,
            delta: V,
            burst: usize,
            cycles: usize,
        )
    }
    delegate! {
        /// Cousin rivalry between adjacent subtree regions.
        fn cousin_rivalry(
            a_start: u64,
            b_start: u64,
            width: u64,
            base: V,
            step: V,
            n: usize,
        )
    }

    /// Adversarial cousins: shared-grandparent targeted spikes.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn adversarial_cousins(
        mut self,
        a: C,
        b: C,
        setup_delta: V,
        n_setup: usize,
        spike_delta_a: V,
        spike_delta_b: V,
        n_spikes: usize,
    ) -> Self {
        self.plan = self
            .plan
            .adversarial_cousins(a, b, setup_delta, n_setup, spike_delta_a, spike_delta_b, n_spikes);
        self
    }

    delegate! {
        /// Phase-shifted oscillation between two close hotspots.
        fn phase_shifted_oscillation(
            a: C,
            b: C,
            high: V,
            low: V,
            burst_len: usize,
            cycles: usize,
        )
    }
    delegate! {
        /// Dense neighborhood burst around `center`.
        fn neighborhood_burst(center: u64, radius: u64, base: V, n: usize)
    }

    // ── Deep / structural patterns ──────────────────────────

    delegate! {
        /// Fractal spray at dyadic midpoints.
        fn fractal_spray(domain: u64, delta: V, depth: u32)
    }
    delegate! {
        /// Nested bursts at dyadic midpoints.
        fn nested_bursts(domain: u64, delta: V, levels: u32, burst_per_level: usize)
    }
    delegate! {
        /// Depth ladder: decreasing power-of-2 separations.
        fn depth_ladder(bits: u32, delta: V, reps: usize)
    }
    delegate! {
        /// Staircase: progressive sibling filling.
        fn staircase(step: u64, stairs: usize, reps_per_stair: usize, base: V)
    }
    delegate! {
        /// Cascade shift: hotspot then sibling transition.
        fn cascade_shift(a: C, b: C, delta: V, n_build: usize, n_shift: usize)
    }

    // ── Coverage / topology patterns ────────────────────────

    delegate! {
        /// Diamond: converge then diverge.
        fn diamond(domain: u64, delta: V, n: usize)
    }
    delegate! {
        /// Bit-flip walk: Gray code traversal.
        fn bit_flip_walk(domain: u64, delta: V, n: usize)
    }
    delegate! {
        /// Sibling flood: flood halves sequentially.
        fn sibling_flood(
            domain: u64,
            delta_first: V,
            delta_second: V,
            n_per_half: usize,
        )
    }
    delegate! {
        /// Mirrored growth: symmetric halves.
        fn mirrored_growth(domain: u64, delta: V, n: usize)
    }
    delegate! {
        /// Checkerboard: even coords then odd coords.
        fn checkerboard(domain: u64, delta_even: V, delta_odd: V, n: usize)
    }
    delegate! {
        /// Pincer: converging fronts.
        fn pincer(lo: u64, hi: u64, delta: V, burst_per_step: usize)
    }
    delegate! {
        /// Repulsion walk: diverging fronts.
        fn repulsion_walk(center: u64, delta: V, max_spread: u64, burst_per_step: usize)
    }
    delegate! {
        /// Sawtooth: escalating sweeps.
        fn sawtooth(width: u64, base: V, cycles: usize)
    }
    delegate! {
        /// Centroid drift: sliding hotspot.
        fn centroid_drift(start: u64, end: u64, delta: V, burst_len: usize)
    }
    delegate! {
        /// Pulse decay: decaying concentrated burst.
        fn pulse_decay(
            center: u64,
            domain: u64,
            base: V,
            rounds: usize,
            obs_per_round: usize,
        )
    }
    delegate! {
        /// Golden spiral: Fibonacci-hashed dispersal.
        fn golden_spiral(domain: u64, delta: V, n: usize)
    }

    /// Append a raw plan.
    #[must_use]
    pub fn then(mut self, other: Plan<C, V>) -> Self {
        self.plan = self.plan.then(other);
        self
    }

    // ── Configuration ───────────────────────────────────────

    /// Enable invariant checking every `k` observations during
    /// `build()`.  `k = 0` means check after every observation.
    #[must_use]
    pub const fn check_every(mut self, k: usize) -> Self {
        self.check_invariants_every = Some(if k == 0 { 1 } else { k });
        self
    }

    // ── Terminal operations ─────────────────────────────────

    /// Extract the serializable plan (without executing it).
    #[must_use]
    pub fn plan(self) -> Plan<C, V> {
        self.plan
    }

    /// Extract the config.
    #[must_use]
    pub const fn config(&self) -> &Config<V> {
        &self.config
    }

    /// Execute the plan and return the graph.
    ///
    /// If `check_every` was set, `assert_invariants` is called at
    /// the specified interval.
    #[must_use]
    pub fn build<const N: u32>(self) -> GvGraph<C, V, N> {
        match self.check_invariants_every {
            Some(k) => run_checked::<C, V, N>(self.config, &self.plan, k),
            None => run::<C, V, N>(self.config, &self.plan),
        }
    }
}

// ── u64-specific convenience constructors ───────────────────────

impl GraphCreator<u64, u64> {
    /// Start building with `default_config()`.
    #[must_use]
    pub const fn default_u64() -> Self {
        Self::new(default_config())
    }

    /// Start building with `budget_config(budget)`.
    #[must_use]
    pub const fn with_budget(budget: usize) -> Self {
        Self::new(budget_config(budget))
    }
}

impl GraphCreator<f64, f64> {
    /// Start building with `f64_default_config()`.
    #[must_use]
    pub const fn default_f64() -> Self {
        Self::new(super::config::f64_default_config())
    }

    /// Start building with `f64_deep_config()`.
    #[must_use]
    pub const fn deep_f64() -> Self {
        Self::new(super::config::f64_deep_config())
    }
}
