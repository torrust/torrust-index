// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

// ── Serializable Plan ───────────────────────────────────────────

use std::fmt::Debug;

use crate::traits::{Accumulator, Coordinate, Inspectable};

/// A replayable, serializable observation plan.
///
/// Holds a sequence of `(coordinate, delta)` pairs.  The `Config` is
/// supplied separately at execution time so the same plan can be
/// replayed under different configurations.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(
        serialize = "C: serde::Serialize, V: serde::Serialize",
        deserialize = "C: serde::Deserialize<'de>, V: serde::Deserialize<'de>"
    ))
)]
pub struct Plan<C, V> {
    pub observations: Vec<(C, V)>,
}

// ── Core methods (only need Copy) ───────────────────────────────

impl<C: Copy + Debug, V: Copy + Debug> Plan<C, V> {
    /// Empty plan.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            observations: Vec::new(),
        }
    }

    /// Single observation.
    #[must_use]
    pub fn observe(mut self, coord: C, delta: V) -> Self {
        self.observations.push((coord, delta));
        self
    }

    /// Repeat the same observation `n` times.
    #[must_use]
    pub fn observe_n(mut self, coord: C, delta: V, n: usize) -> Self {
        self.observations.extend(std::iter::repeat_n((coord, delta), n));
        self
    }

    /// Concentrated hotspot: `n` observations at a single `coord`.
    #[must_use]
    pub fn hotspot(self, coord: C, delta: V, n: usize) -> Self {
        self.observe_n(coord, delta, n)
    }

    /// Append another plan's observations after this one.
    #[must_use]
    pub fn then(mut self, other: Self) -> Self {
        self.observations.extend(other.observations);
        self
    }

    /// Number of observations in the plan.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.observations.len()
    }

    /// Whether the plan has zero observations.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }
}

impl<C: Copy + Debug + Default, V: Copy + Debug> Default for Plan<C, V> {
    fn default() -> Self {
        Self::new()
    }
}

// ── Convenience builders (need Coordinate + Accumulator) ────────

impl<C: Coordinate, V: Accumulator + Inspectable> Plan<C, V> {
    /// Uniform spread: `n` observations cycling through coordinates
    /// `0, 1, 2, …` (mod `domain`), all with the same `delta`.
    #[must_use]
    pub fn spread(mut self, domain: u64, delta: V, n: usize) -> Self {
        for i in 0..n {
            let coord = C::from_u64(i as u64 % domain);
            self.observations.push((coord, delta));
        }
        self
    }

    /// Monotone sweep: coordinates `0, 1, 2, …` (mod `domain`), with
    /// varying intensity `(i % 7) + 1`.
    #[must_use]
    pub fn sweep(mut self, domain: u64, n: usize) -> Self {
        for i in 0..n {
            let coord = C::from_u64(i as u64 % domain);
            #[allow(clippy::cast_precision_loss)]
            let delta = V::from_f64((i as u64 % 7 + 1) as f64);
            self.observations.push((coord, delta));
        }
        self
    }

    /// Alternating extremes: `lo`, `hi`, `lo`, `hi`, …
    #[must_use]
    pub fn zigzag(mut self, lo: C, hi: C, delta: V, n: usize) -> Self {
        for i in 0..n {
            let coord = if i % 2 == 0 { lo } else { hi };
            self.observations.push((coord, delta));
        }
        self
    }

    /// Left-biased zigzag with **escalating** intensity.
    /// Alternates between `lo` and `hi`, but the `lo` side gets
    /// `base + i * step` intensity while `hi` stays at `base`.
    /// Forces asymmetric growth and repeated contraction
    /// preprocessing.
    #[must_use]
    pub fn adversarial_zigzag(mut self, lo: C, hi: C, base: V, step: V, n: usize) -> Self {
        for i in 0..n {
            if i % 2 == 0 {
                #[allow(clippy::cast_precision_loss)]
                let delta = V::from_f64((i as f64).mul_add(step.to_f64_approx(), base.to_f64_approx()));
                self.observations.push((lo, delta));
            } else {
                self.observations.push((hi, base));
            }
        }
        self
    }

    /// Power-law skew: lower coordinates get disproportionate
    /// intensity.  `coord = i % domain`, `delta = if coord < domain/4
    /// { 10 + i%7 } else { 1 + i%3 }`.
    #[must_use]
    pub fn skewed(mut self, domain: u64, n: usize) -> Self {
        let quarter = domain / 4;
        for i in 0..n {
            let raw = i as u64 % domain;
            let coord = C::from_u64(raw);
            let delta = if raw < quarter {
                #[allow(clippy::cast_precision_loss)]
                V::from_f64((10 + (i as u64 % 7)) as f64)
            } else {
                #[allow(clippy::cast_precision_loss)]
                V::from_f64((1 + (i as u64 % 3)) as f64)
            };
            self.observations.push((coord, delta));
        }
        self
    }

    /// Burst: `n_spread` uniform observations across `domain`,
    /// followed by `n_hot` concentrated observations at `hotspot`.
    /// Exercises growth → pressure → eviction → relaxation.
    #[must_use]
    pub fn burst(mut self, domain: u64, spread_delta: V, n_spread: usize, hotspot: C, hot_delta: V, n_hot: usize) -> Self {
        for i in 0..n_spread {
            let coord = C::from_u64(i as u64 % domain);
            self.observations.push((coord, spread_delta));
        }
        self.observations.extend(std::iter::repeat_n((hotspot, hot_delta), n_hot));
        self
    }

    /// Uniform random spray: `n` observations at pseudo-random
    /// coordinates within `[0, domain)`, all with the same `delta`.
    ///
    /// Deterministic: same `seed` → identical plan.
    /// Uses an inline LCG (no external RNG dependency).
    #[must_use]
    pub fn random_spray(mut self, seed: u64, domain: u64, delta: V, n: usize) -> Self {
        let mut state = seed;
        for _ in 0..n {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let coord = C::from_u64(state % domain);
            self.observations.push((coord, delta));
        }
        self
    }

    /// Oscillating hotspot: every `burst` observations the target
    /// switches between `a` and `b`.  Forces repeated deep refinement
    /// followed by abandonment and eviction.
    #[must_use]
    pub fn oscillating_hotspot(mut self, a: C, b: C, delta: V, burst: usize, cycles: usize) -> Self {
        for cycle in 0..cycles {
            let target = if cycle % 2 == 0 { a } else { b };
            self.observations.extend(std::iter::repeat_n((target, delta), burst));
        }
        self
    }

    // ── Targeted patterns ─────────────────────────────
    //
    // These patterns create overlapping violation paths — two or more
    // concurrent violations in the same V-Tree neighborhood at
    // different depths sharing a grandparent.  This is the topology
    // required to trigger violation source 10 (g-contraction +
    // promotion on intermediate-level grandchildren).

    /// Interleaved multi-hotspot: `cycles` rounds, each round emits
    /// `burst` observations at the next of `count` coordinates
    /// starting at `start` with `spacing` apart.
    ///
    /// Unlike [`oscillating_hotspot`](Self::oscillating_hotspot) (2
    /// far-apart points), this targets `count` spatially close points
    /// whose V-Tree subtrees share ancestors, creating overlapping
    /// violation neighborhoods.
    #[must_use]
    pub fn interleaved_hotspots(mut self, start: u64, count: u64, spacing: u64, delta: V, burst: usize, cycles: usize) -> Self {
        for cycle in 0..cycles {
            let coord = C::from_u64(start + (cycle as u64 % count) * spacing);
            self.observations.extend(std::iter::repeat_n((coord, delta), burst));
        }
        self
    }

    /// Cousin rivalry: alternate observations between two adjacent
    /// subtree regions `[a_start, a_start+width)` and
    /// `[b_start, b_start+width)` with escalating intensity.
    ///
    /// The two regions should share a common ancestor (e.g., left and
    /// right children of an internal node), creating concurrent
    /// violations in adjacent subtrees — the precondition for source
    /// 10.
    #[must_use]
    pub fn cousin_rivalry(mut self, a_start: u64, b_start: u64, width: u64, base: V, step: V, n: usize) -> Self {
        for i in 0..n {
            let offset = i as u64 % width;
            #[allow(clippy::cast_precision_loss)]
            let (region_start, intensity_mult) = if i % 2 == 0 {
                (a_start, i as f64)
            } else {
                (b_start, (i as f64) * 0.7)
            };
            let coord = C::from_u64(region_start + offset);
            #[allow(clippy::cast_precision_loss)]
            let delta = V::from_f64(intensity_mult.mul_add(step.to_f64_approx(), base.to_f64_approx()));
            self.observations.push((coord, delta));
        }
        self
    }

    /// Adversarial cousins: targeted observations at two coordinates
    /// that share a grandparent, with carefully chosen intensities to
    /// create violations at two different depths.
    ///
    /// Emits `n_setup` gentle interleaved observations at both `a`
    /// and `b`, then `n_spikes` interleaved high-intensity
    /// observations designed to create concurrent violations.
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
        // Gentle buildup at both coordinates
        for _ in 0..n_setup {
            self.observations.push((a, setup_delta));
            self.observations.push((b, setup_delta));
        }
        // Interleaved spikes
        for _ in 0..n_spikes {
            self.observations.push((a, spike_delta_a));
            self.observations.push((b, spike_delta_b));
        }
        self
    }

    /// Phase-shifted oscillation: two spatially close hotspots (`a`,
    /// `b`) oscillate out of phase.  When `a` is in a high-intensity
    /// burst, `b` gets a trickle, and vice versa.
    ///
    /// Unlike [`oscillating_hotspot`](Self::oscillating_hotspot) which
    /// fully alternates, here both points always receive *some*
    /// observations, maintaining concurrent growth in the same V-Tree
    /// neighborhood.
    #[must_use]
    pub fn phase_shifted_oscillation(mut self, a: C, b: C, high: V, low: V, burst_len: usize, cycles: usize) -> Self {
        for cycle in 0..cycles {
            let (a_delta, b_delta) = if cycle % 2 == 0 { (high, low) } else { (low, high) };
            for _ in 0..burst_len {
                self.observations.push((a, a_delta));
                self.observations.push((b, b_delta));
            }
        }
        self
    }

    /// Dense neighborhood burst: `n` observations within
    /// `[center − radius, center + radius] ∩ [0, domain)` with
    /// intensity proportional to proximity to `center`.
    ///
    /// Creates a dense cluster of violations in a small spatial
    /// region, forcing multiple concurrent violations with shared
    /// ancestors.
    #[must_use]
    pub fn neighborhood_burst(mut self, center: u64, radius: u64, base: V, n: usize) -> Self {
        let lo = center.saturating_sub(radius);
        let hi = center + radius;
        let width = hi - lo + 1;
        for i in 0..n {
            let offset = i as u64 % width;
            let coord = C::from_u64(lo + offset);
            let dist = (lo + offset).abs_diff(center);
            #[allow(clippy::cast_precision_loss)]
            let delta = V::from_f64(base.to_f64_approx() * (1.0 + (radius as f64 - dist as f64)));
            self.observations.push((coord, delta));
        }
        self
    }

    // ── Deep / structural patterns ──────────────────────────────
    //
    // Exercise V-Tree depth, multi-level splitting, and restructuring
    // at various scales.

    /// Fractal spray: one observation at each dyadic midpoint of
    /// the domain, level by level.
    ///
    /// Level 0: `domain/2`.
    /// Level 1: `domain/4`, `3·domain/4`.
    /// Level k: `(2j+1) · domain / 2^(k+1)` for `j = 0..2^k`.
    ///
    /// Builds a balanced V-Tree top-down, creating splits at every
    /// level.
    #[must_use]
    pub fn fractal_spray(mut self, domain: u64, delta: V, depth: u32) -> Self {
        for d in 0..depth {
            let denom = 1u64 << (d + 1);
            let count = 1u64 << d;
            for k in 0..count {
                let coord = (2 * k + 1) * domain / denom;
                if coord < domain {
                    self.observations.push((C::from_u64(coord), delta));
                }
            }
        }
        self
    }

    /// Nested bursts: multi-scale fractal bursts at progressively
    /// finer spatial resolution.
    ///
    /// Same dyadic midpoint sequence as
    /// [`fractal_spray`](Self::fractal_spray), but each point gets
    /// `burst_per_level` repeated observations, creating sustained
    /// violations at every V-Tree depth simultaneously.
    #[must_use]
    pub fn nested_bursts(mut self, domain: u64, delta: V, levels: u32, burst_per_level: usize) -> Self {
        for level in 0..levels {
            let denom = 1u64 << (level + 1);
            let count = 1u64 << level;
            for j in 0..count {
                let coord = (2 * j + 1) * domain / denom;
                if coord < domain {
                    self.observations
                        .extend(std::iter::repeat_n((C::from_u64(coord), delta), burst_per_level));
                }
            }
        }
        self
    }

    /// Depth ladder: observations at coordinates separated by
    /// decreasing powers of 2: `0, 2^(bits−1), 2^(bits−2), …, 1`.
    ///
    /// Each successive observation shares a progressively deeper
    /// ancestor with coordinate 0, walking down the V-Tree spine.
    /// Repeated `reps` times to build intensity at every depth.
    #[must_use]
    pub fn depth_ladder(mut self, bits: u32, delta: V, reps: usize) -> Self {
        for _ in 0..reps {
            self.observations.push((C::from_u64(0), delta));
            for b in (0..bits).rev() {
                self.observations.push((C::from_u64(1u64 << b), delta));
            }
        }
        self
    }

    /// Staircase: progressively fill wider coordinate ranges with
    /// escalating intensity.
    ///
    /// Stair `k` (of `stairs`): observe coordinates
    /// `0, step, 2·step, …, (k−1)·step`, each `reps_per_stair`
    /// times, at intensity `base × k`.  Forces sibling subtrees to
    /// fill evenly before the tree grows vertically.
    #[must_use]
    pub fn staircase(mut self, step: u64, stairs: usize, reps_per_stair: usize, base: V) -> Self {
        for stair in 1..=stairs {
            #[allow(clippy::cast_precision_loss)]
            let delta = V::from_f64(base.to_f64_approx() * (stair as f64));
            for coord_idx in 0..stair {
                let coord = C::from_u64(coord_idx as u64 * step);
                self.observations.extend(std::iter::repeat_n((coord, delta), reps_per_stair));
            }
        }
        self
    }

    /// Cascade shift: build a hotspot at `a`, then gradually shift
    /// activity to its sibling coordinate `b`.
    ///
    /// The transition interleaves lingering `a` observations (every
    /// 3rd step) with dominant `b` observations, forcing the old
    /// subtree to merge/contract while the new one splits — creating
    /// violations on both sides of their shared parent.
    #[must_use]
    pub fn cascade_shift(mut self, a: C, b: C, delta: V, n_build: usize, n_shift: usize) -> Self {
        // Pure buildup at a
        self.observations.extend(std::iter::repeat_n((a, delta), n_build));
        // Transition: gradually shift weight from a to b
        for i in 0..n_shift {
            if i % 3 == 0 {
                self.observations.push((a, delta)); // lingering
            } else {
                self.observations.push((b, delta));
            }
        }
        self
    }

    // ── Coverage / topology patterns ────────────────────────────
    //
    // Diverse traversal strategies that exercise the V-Tree from
    // different angles.

    /// Diamond: converge from domain extremes toward the center, then
    /// diverge back out.
    ///
    /// Phase 1 (converge): alternating `0, domain−1, 1, domain−2, …`
    /// Phase 2 (diverge): center outward in both directions.
    ///
    /// Exercises tree restructuring in both contraction and expansion
    /// directions.
    #[must_use]
    pub fn diamond(mut self, domain: u64, delta: V, n: usize) -> Self {
        let half = n / 2;
        // Converge: outside → center
        for i in 0..half {
            let offset = i as u64 % (domain / 2);
            if i % 2 == 0 {
                self.observations.push((C::from_u64(offset), delta));
            } else {
                self.observations.push((C::from_u64(domain - 1 - offset), delta));
            }
        }
        // Diverge: center → outside
        let center = domain / 2;
        for i in 0..(n - half) {
            let offset = i as u64 % (domain / 2);
            if i % 2 == 0 {
                let coord = (center + offset).min(domain - 1);
                self.observations.push((C::from_u64(coord), delta));
            } else {
                self.observations
                    .push((C::from_u64(center.saturating_sub(offset + 1)), delta));
            }
        }
        self
    }

    /// Bit-flip walk: visit coordinates in Gray code order.
    ///
    /// Adjacent coordinates in a Gray code sequence differ by exactly
    /// one bit, meaning each step crosses exactly one sibling boundary
    /// in the V-Tree.  Systematically exercises every ancestor
    /// relationship.
    #[must_use]
    pub fn bit_flip_walk(mut self, domain: u64, delta: V, n: usize) -> Self {
        for i in 0..n {
            let raw = i as u64 % domain;
            // Gray code: G(n) = n ^ (n >> 1)
            let gray = raw ^ (raw >> 1);
            self.observations.push((C::from_u64(gray % domain), delta));
        }
        self
    }

    /// Sibling flood: flood one half of the domain, then abruptly
    /// flood the other half with (potentially different) intensity.
    ///
    /// The halves share the root as their common ancestor.  The
    /// abrupt shift forces massive restructuring at the shallowest
    /// tree level.
    #[must_use]
    pub fn sibling_flood(mut self, domain: u64, delta_first: V, delta_second: V, n_per_half: usize) -> Self {
        let half = domain / 2;
        // Flood lower half
        for i in 0..n_per_half {
            self.observations.push((C::from_u64(i as u64 % half), delta_first));
        }
        // Flood upper half
        for i in 0..n_per_half {
            self.observations.push((C::from_u64(half + (i as u64 % half)), delta_second));
        }
        self
    }

    /// Mirrored growth: simultaneous symmetric observations at `x`
    /// and `domain − 1 − x` for `x = 0, 1, 2, …`.
    ///
    /// Forces balanced growth in both halves of the V-Tree, creating
    /// symmetric violation patterns.
    #[must_use]
    pub fn mirrored_growth(mut self, domain: u64, delta: V, n: usize) -> Self {
        for i in 0..n {
            let x = i as u64 % (domain / 2);
            self.observations.push((C::from_u64(x), delta));
            self.observations.push((C::from_u64(domain - 1 - x), delta));
        }
        self
    }

    /// Checkerboard: all even coordinates first, then all odd
    /// coordinates.
    ///
    /// Phase 1 creates violations at even-child boundaries; phase 2
    /// injects adjacent violations that share parents with phase 1
    /// nodes, potentially triggering source 10 topology.
    #[must_use]
    pub fn checkerboard(mut self, domain: u64, delta_even: V, delta_odd: V, n: usize) -> Self {
        let half = n / 2;
        // Even coords
        for i in 0..half {
            let coord = (i as u64 * 2) % domain;
            self.observations.push((C::from_u64(coord), delta_even));
        }
        // Odd coords
        for i in 0..(n - half) {
            let coord = (i as u64 * 2 + 1) % domain;
            self.observations.push((C::from_u64(coord), delta_odd));
        }
        self
    }

    /// Pincer: two fronts approaching each other from opposite ends
    /// of `[lo, hi]`.
    ///
    /// Step k: observe at `lo + k` and `hi − k` with `burst_per_step`
    /// repetitions.  Creates converging violation paths that
    /// eventually share the same ancestor at every depth.
    #[must_use]
    pub fn pincer(mut self, lo: u64, hi: u64, delta: V, burst_per_step: usize) -> Self {
        let mut left = lo;
        let mut right = hi;
        while left <= right {
            for _ in 0..burst_per_step {
                self.observations.push((C::from_u64(left), delta));
                if left != right {
                    self.observations.push((C::from_u64(right), delta));
                }
            }
            left += 1;
            if right == 0 {
                break;
            }
            right -= 1;
        }
        self
    }

    /// Repulsion walk: two hotspots start at the same coordinate and
    /// gradually drift apart, one step at a time.
    ///
    /// Creates initially shared then diverging violation paths,
    /// exercising the transition from single-spine to cousin
    /// topology.
    #[must_use]
    pub fn repulsion_walk(mut self, center: u64, delta: V, max_spread: u64, burst_per_step: usize) -> Self {
        for offset in 0..=max_spread {
            let a = center.saturating_sub(offset);
            let b = center + offset;
            for _ in 0..burst_per_step {
                self.observations.push((C::from_u64(a), delta));
                if a != b {
                    self.observations.push((C::from_u64(b), delta));
                }
            }
        }
        self
    }

    /// Sawtooth: repeated linear sweeps through `[0, width)` with
    /// intensity escalating each cycle.
    ///
    /// Creates periodic pressure waves that force the V-Tree to
    /// adapt to ever-increasing load.
    #[must_use]
    pub fn sawtooth(mut self, width: u64, base: V, cycles: usize) -> Self {
        for k in 0..cycles {
            #[allow(clippy::cast_precision_loss)]
            let delta = V::from_f64(base.to_f64_approx() * ((k + 1) as f64));
            for x in 0..width {
                self.observations.push((C::from_u64(x), delta));
            }
        }
        self
    }

    /// Centroid drift: a hotspot that slides across the domain from
    /// `start` to `end`, spending `burst_len` observations at each
    /// coordinate.
    ///
    /// Creates a moving front of violations that exercises different
    /// V-Tree neighborhoods in sequence, with lingering effects from
    /// previous positions.
    #[must_use]
    pub fn centroid_drift(mut self, start: u64, end: u64, delta: V, burst_len: usize) -> Self {
        let (lo, hi) = if start <= end { (start, end) } else { (end, start) };
        for coord in lo..=hi {
            self.observations
                .extend(std::iter::repeat_n((C::from_u64(coord), delta), burst_len));
        }
        self
    }

    /// Pulse decay: concentrated initial burst that decays in both
    /// intensity and spatial extent over successive rounds.
    ///
    /// Round k: intensity = `base / (k + 1)`, coordinates cycle
    /// through `[center − k, center + k] ∩ [0, domain)`.
    #[must_use]
    pub fn pulse_decay(mut self, center: u64, domain: u64, base: V, rounds: usize, obs_per_round: usize) -> Self {
        for k in 0..rounds {
            #[allow(clippy::cast_precision_loss)]
            let delta = V::from_f64(base.to_f64_approx() / (k as f64 + 1.0));
            let lo = center.saturating_sub(k as u64);
            let hi = (center + k as u64).min(domain - 1);
            let width = hi - lo + 1;
            for j in 0..obs_per_round {
                let coord = C::from_u64(lo + (j as u64 % width));
                self.observations.push((coord, delta));
            }
        }
        self
    }

    /// Golden spiral: Fibonacci-hashed coordinates that provide
    /// maximally dispersed, quasi-uniform coverage of the domain.
    ///
    /// Uses the golden ratio to generate a low-discrepancy sequence
    /// — each new observation lands as far as possible from all
    /// previous ones.  Deterministic.
    #[must_use]
    pub fn golden_spiral(mut self, domain: u64, delta: V, n: usize) -> Self {
        // Golden-ratio fractional step: floor(domain × (√5 − 1) / 2)
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation, clippy::cast_precision_loss)]
        let step = ((domain as f64) * 0.618_033_988_749_895).ceil() as u64;
        let step = step.max(1);
        let mut coord_raw = 0u64;
        for _ in 0..n {
            self.observations.push((C::from_u64(coord_raw % domain), delta));
            coord_raw = coord_raw.wrapping_add(step);
        }
        self
    }
}
