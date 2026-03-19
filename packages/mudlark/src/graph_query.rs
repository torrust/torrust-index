// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Read-side queries on `GvGraph`: sampling, point lookup,
//! range accumulation, and contour range decomposition.
//!
//! Extracted from `graph.rs` per ADR-M-030 Phase C.
//! Contour range queries added per ADR-M-037 Phase 3.

#[cfg(debug_assertions)]
use crate::contour_range::debug_assert_contour_range_invariants;
use crate::contour_range::{BasisElement, ContourRange, ContourRangeEnergy, compute_plateau_energy, validate_endpoints};
use crate::gnode::GNode;
use crate::graph::GvGraph;
use crate::gtree::gnode_depth_from_interval;
use crate::handle::{GNodeId, VNodeId};
use crate::plateau::BasisEdge;
use crate::traits::{Accumulator, Coordinate, Inspectable, Proratable, Weighable};

impl<C: Coordinate, V: Accumulator + Weighable, const N: u32> GvGraph<C, V, N> {
    // ── Sampling (Phase 4, Step 1 — ADR-M-019) ───────────────────

    /// Sample a terminal cell with probability proportional to its
    /// intensity.
    ///
    /// Performs a weighted random walk from the V-Tree root to a
    /// V-Entry, choosing each child with probability proportional to
    /// its cached intensity (§IDEA M-6.5, ADR-M-019).
    ///
    /// Returns `None` if total intensity is zero (DC-019-2).
    ///
    /// Expected cost: $O(1.44\, H + 1.67)$ where $H$ is the Shannon
    /// entropy of the intensity distribution.
    ///
    /// With the `rand` feature (enabled by default), any
    /// [`rand_core::Rng`] type satisfies the [`Rng`](crate::Rng)
    /// bound via a blanket impl — e.g. `g.sample(&mut rand::rng())`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph, Rng};
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// # struct SimpleRng(u64);
    /// # impl Rng for SimpleRng {
    /// #     fn next_f64(&mut self) -> f64 {
    /// #         self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
    /// #         (self.0 >> 11) as f64 / (1u64 << 53) as f64
    /// #     }
    /// # }
    /// # let mut rng = SimpleRng(12345);
    /// // Empty graph — total intensity is zero.
    /// assert!(g.sample(&mut rng).is_none());
    ///
    /// g.observe(100, 20u64);
    /// let cell = g.sample(&mut rng).unwrap();
    /// assert!(cell.start <= 100 && 100 < cell.end);
    /// ```
    #[must_use]
    pub fn sample(&self, rng: &mut impl crate::traits::Rng) -> Option<crate::view::Cell<C, V>> {
        use crate::vnode::VKind;

        let v_root = self.v_root?;
        let root_node = self.vnodes.get(v_root.index());
        if root_node.intensity == V::zero() {
            return None;
        }

        let mut current = v_root;
        loop {
            let vnode = self.vnodes.get(current.index());
            match &vnode.kind {
                VKind::Entry { gnode, .. } => {
                    let g = self.gnodes.get(gnode.index());
                    return Some(crate::view::Cell {
                        start: g.lo,
                        end: g.hi,
                        intensity: g.own,
                        depth: crate::gtree::gnode_depth_from_interval(g.lo, g.hi, N),
                    });
                }
                VKind::Structural { children, .. } => {
                    current = Self::sample_child(children, rng);
                }
            }
        }
    }

    /// Weighted child selection via cumulative-sum scan over
    /// `PackedChildren::intensities`.
    ///
    /// Picks a child with probability `child.intensity / total`.
    /// The hot loop touches only the contiguous `intensities` array
    /// (24 bytes for 3×`u64`) — SoA-optimised for this path.
    fn sample_child(children: &crate::vnode::PackedChildren<V>, rng: &mut impl crate::traits::Rng) -> VNodeId {
        let total: f64 = children.intensities[..children.len()].iter().map(|v| v.weight()).sum();
        debug_assert!(total > 0.0, "sample_child: zero-total children");

        let threshold = rng.next_f64() * total;
        let mut cumulative = 0.0_f64;
        for i in 0..children.len() {
            cumulative += children.intensities[i].weight();
            if threshold < cumulative {
                return children.ids[i].expect("child ID within len");
            }
        }
        // Floating-point rounding safeguard: if threshold rounds to
        // exactly `total`, fall through to the last child.
        children.ids[children.len() - 1].expect("last child ID")
    }

    // ── Point Query (Phase 5, Step 3 — ADR-M-026) ────────────────
}

impl<C: Coordinate, V: Accumulator, const N: u32> GvGraph<C, V, N> {
    /// Infallible point query: which cell receives observations at
    /// `coord`?
    ///
    /// Routes through the G-Tree via `route_to_receiver` and returns
    /// a [`Cell`] snapshot. For semi-internal nodes, the returned
    /// interval is the *trimmed* uncovered half — the effective
    /// region that accumulates directly, not the arena node's full
    /// `[lo, hi)`.
    ///
    /// Out-of-domain coordinates are clamped to `[0, 2^N)`,
    /// matching `range_sum()`'s behaviour.
    ///
    /// Cost: $O(\text{depth})$ — iterative G-Tree descent.
    ///
    /// # Panics
    ///
    /// Panics if `coord` is NaN (float coordinate types).
    ///
    /// # Examples
    ///
    /// ```
    /// use torrust_mudlark::{GvGraph, Config};
    /// let cfg = Config {
    ///     split_threshold: 100u64,
    ///     depth_create: 3,
    ///     depth_evict: 6,
    ///     budget: None,
    ///     alpha_relax: 0.75,
    ///     bounded_eviction: true,
    /// };
    /// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// g.observe(42, 10u64);
    /// let cell = g.get(42);
    /// assert!(cell.start <= 42 && 42 < cell.end);
    /// assert_eq!(cell.intensity, 10);
    /// ```
    ///
    /// [`Cell`]: crate::view::Cell
    #[must_use]
    pub fn get(&self, coord: C) -> crate::view::Cell<C, V> {
        // ── NaN guard (float types) ─────────────────────────────
        assert!(!coord.is_nan(), "get(): coordinate is NaN");

        // ── Domain clamping (DC-026-4) ──────────────────────────
        let clamped = Self::clamp_to_domain(coord);

        // ── Route to receiver ───────────────────────────────────
        let g_id = crate::gtree::route_to_receiver(&self.gnodes, self.g_root, clamped);
        let g = self.gnodes.get(g_id.index());

        // ── Trim half-interval for semi-internals ───────────────
        let (start, end) = Self::trimmed_interval(g, clamped);

        #[cfg(debug_assertions)]
        {
            // The returned cell's interval contains the (clamped) coord.
            debug_assert!(
                start <= clamped && clamped < end || (clamped == C::domain_max(N) && start < end),
                "get(): returned cell [{start:?}, {end:?}) does not \
                 contain clamped coord {clamped:?}",
            );
        }

        crate::view::Cell {
            start,
            end,
            intensity: g.own,
            depth: crate::gtree::gnode_depth_from_interval(start, end, N),
        }
    }

    /// Clamp a coordinate to the domain `[0, 2^N)`.
    ///
    /// Out-of-range coordinates are projected inward:
    /// - `coord < 0` (signed/float) → `C::zero()`
    /// - `coord >= domain_max(N)` → `domain_max(N)` (routing handles
    ///   this by always going right, landing in the rightmost terminal)
    /// - otherwise → `coord`
    ///
    /// Safe because `route_to_receiver` uses `x < mid` / else, so
    /// `x == domain_max` always routes right, and the returned `Cell`
    /// draws its interval from the arena node's `[lo, hi)` which is
    /// always inside the domain.
    #[inline]
    fn clamp_to_domain(coord: C) -> C {
        let lo = C::zero();
        let hi = C::domain_max(N);
        if coord < lo {
            lo
        } else if coord >= hi {
            hi
        } else {
            coord
        }
    }

    /// Trim a G-node's interval to the effective half for semi-internals.
    ///
    /// - **Terminal / Internal:** returns `(g.lo, g.hi)`.
    /// - **Semi-internal (left child):** uncovered right half `[mid, hi)`.
    /// - **Semi-internal (right child):** uncovered left half `[lo, mid)`.
    ///
    /// Routing guarantees `coord` falls in the uncovered half for
    /// semi-internals; `debug_assert` validates this.
    #[inline]
    fn trimmed_interval(g: &GNode<C, V>, coord: C) -> (C, C) {
        use crate::gnode::GState;
        match g.state() {
            GState::Terminal | GState::Internal => (g.lo, g.hi),
            GState::SemiInternal => {
                let mid = C::midpoint(g.lo, g.hi);
                if g.left.is_some() {
                    // Left child covers [lo, mid);
                    // uncovered half is [mid, hi).
                    debug_assert!(
                        coord >= mid,
                        "get(): coord {coord:?} in covered half \
                         [lo={:?}, mid={mid:?}) of semi-internal",
                        g.lo,
                    );
                    (mid, g.hi)
                } else {
                    // Right child covers [mid, hi);
                    // uncovered half is [lo, mid).
                    debug_assert!(
                        coord < mid,
                        "get(): coord {coord:?} in covered half \
                         [mid={mid:?}, hi={:?}) of semi-internal",
                        g.hi,
                    );
                    (g.lo, mid)
                }
            }
        }
    }

    // ── Range Query (Phase 4, Step 2 — ADR-M-020) ────────────────
}

impl<C: Coordinate, V: Accumulator + Proratable, const N: u32> GvGraph<C, V, N> {
    /// Sum of accumulated values in a range, with pro-rated partial
    /// overlaps.
    ///
    /// Accepts any `RangeBounds<C>` (`a..b`, `a..=b`, `..b`, `a..`,
    /// `..`). Bounds outside the domain are clamped. Empty ranges
    /// return `V::zero()`.
    ///
    /// Cost: $O(N)$ — recursive descent over the G-Tree.
    ///
    /// # Panics
    ///
    /// Panics if either bound is NaN (float coordinate types).
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// g.observe(10, 3u64);
    /// g.observe(200, 7u64);
    ///
    /// // Full domain.
    /// assert_eq!(g.range_sum(..), 10);
    ///
    /// // Sub-range containing only the first observation.
    /// assert!(g.range_sum(0..128) >= 3);
    /// ```
    #[must_use]
    pub fn range_sum<R: std::ops::RangeBounds<C>>(&self, range: R) -> V {
        use std::ops::Bound;

        // ── Resolve to half-open [lo, hi) ───────────────────────
        let lo = match range.start_bound() {
            Bound::Included(&x) => {
                assert!(!x.is_nan(), "range_sum: start bound is NaN");
                x
            }
            Bound::Excluded(&x) => {
                assert!(!x.is_nan(), "range_sum: start bound is NaN");
                x.next_value()
            }
            Bound::Unbounded => C::zero(),
        };
        let hi = match range.end_bound() {
            Bound::Included(&x) => {
                assert!(!x.is_nan(), "range_sum: end bound is NaN");
                x.next_value()
            }
            Bound::Excluded(&x) => {
                assert!(!x.is_nan(), "range_sum: end bound is NaN");
                x
            }
            Bound::Unbounded => C::domain_max(N),
        };

        // ── Clamp to domain ─────────────────────────────────────
        let domain_lo = C::zero();
        let domain_hi = C::domain_max(N);

        let lo = if lo < domain_lo { domain_lo } else { lo };
        let hi = if hi > domain_hi { domain_hi } else { hi };

        // Empty range after clamping.
        if lo >= hi {
            return V::zero();
        }

        self.range_sum_inner(self.g_root, lo, hi)
    }

    /// Recursive G-Tree descent for range sum.
    ///
    /// `query_lo..query_hi` is the half-open query range (already
    /// clamped). `gid` is the current G-node.
    fn range_sum_inner(&self, gid: crate::handle::GNodeId, query_lo: C, query_hi: C) -> V {
        let g = self.gnodes.get(gid.index());
        let node_lo = g.lo;
        let node_hi = g.hi;

        // Fully disjoint.
        if query_lo >= node_hi || query_hi <= node_lo {
            return V::zero();
        }

        // Fully contained.
        if query_lo <= node_lo && query_hi >= node_hi {
            return g.sum;
        }

        // ── Partial overlap: pro-rate own, recurse children ─────
        let overlap_lo = if query_lo > node_lo { query_lo } else { node_lo };
        let overlap_hi = if query_hi < node_hi { query_hi } else { node_hi };

        // Pro-rate g.own by overlap/width using f64 ratio arithmetic.
        let node_width = C::width(node_lo, node_hi).to_f64();
        let overlap_width = C::width(overlap_lo, overlap_hi).to_f64();
        let own_prorated = g.own.scale_by(overlap_width / node_width);

        // Recurse into children.
        let left_sum = g
            .left
            .map_or_else(V::zero, |left_id| self.range_sum_inner(left_id, query_lo, query_hi));
        let right_sum = g
            .right
            .map_or_else(V::zero, |right_id| self.range_sum_inner(right_id, query_lo, query_hi));

        V::add(own_prorated, V::add(left_sum, right_sum))
    }
}

// ── Contour Range Query (Phase 3 — ADR-M-037, revised Phase 2) ────

impl<C: Coordinate, V: Accumulator + Proratable + Inspectable, const N: u32> GvGraph<C, V, N> {
    /// Full contour-range decomposition (§CR.2–§CR.6).
    ///
    /// Returns the basis set (minimal G-node cover of the range) and
    /// pre-computed energy fields.  Endpoints must lie on the
    /// endpoint lattice (CR-I1).
    ///
    /// Cost: O(N) — standard segment-tree decomposition plus one
    /// additional O(N) pass via `range_sum()` for the exact energy.
    ///
    /// Returns `None` if either endpoint is not in the endpoint lattice
    /// or `start >= end`.
    ///
    /// # Examples
    ///
    /// ```
    /// use torrust_mudlark::{Config, GvGraph, BasisEdge, ContourRange};
    ///
    /// let cfg = Config {
    ///     split_threshold: 2u64,
    ///     depth_create: 3,
    ///     depth_evict: 6,
    ///     budget: None,
    ///     alpha_relax: 0.75,
    ///     bounded_eviction: true,
    /// };
    /// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// for _ in 0..10 {
    ///     g.observe(16, 5u64);
    ///     g.observe(48, 5u64);
    /// }
    ///
    /// let plateaus = g.plateaus();
    /// let first = *plateaus.keys().next().unwrap();
    /// let end = BasisEdge(256u64);  // domain sentinel = 2^N
    ///
    /// let cr = g.contour_range(first, end).unwrap();
    /// assert!(!cr.basis.is_empty());
    /// assert!(cr.basis.iter().filter(|b| b.is_boundary_thatch).count() <= 2);
    /// // Full-domain: energy == exact_energy (no boundary thatching).
    /// assert_eq!(cr.energy, cr.exact_energy);
    /// ```
    #[must_use]
    pub fn contour_range(&self, start: BasisEdge<C>, end: BasisEdge<C>) -> Option<ContourRange<C, V>> {
        // 1. Obtain plateau map.
        let plateaus = self.plateaus();

        // 2. Validate endpoints (CR-I1).
        validate_endpoints(&plateaus, start, end, C::domain_max(N))?;

        // 3. Compute plateau energy.
        let (plateau_energy, plateau_count) = compute_plateau_energy(&plateaus, start, end);

        // 4. Collect basis set.
        let mut basis = Vec::new();
        self.decompose_basis(self.g_root, start.0, end.0, &mut basis);

        // 5. Compute energy fields.
        //
        // §CR.6: contour range energy = Σ basis[i].sum
        let energy = basis.iter().fold(V::zero(), |acc, b| V::add(acc, b.sum));
        //
        // §CR.13: exact energy = range_sum(start..end)
        let exact_energy = self.range_sum(start.0..end.0);
        //
        // §CR.10.4: cross-plateau energy
        let cross_plateau_energy = V::sub(energy, plateau_energy);

        // 6. Assemble result.
        let result = ContourRange {
            start: start.0,
            end: end.0,
            basis,
            energy,
            exact_energy,
            plateau_energy,
            cross_plateau_energy,
            plateau_count,
        };

        // 7. Debug-build invariant assertions.
        #[cfg(debug_assertions)]
        debug_assert_contour_range_invariants(&result);

        Some(result)
    }

    /// Energy-only contour-range query (§CR.6, §CR.13).
    ///
    /// Returns the energy scalars without exposing the decomposition
    /// vectors.  Same lattice-endpoint requirement as
    /// [`contour_range()`](Self::contour_range).
    ///
    /// Cost: O(N).
    ///
    /// Returns `None` if either endpoint is not in the endpoint lattice
    /// or `start >= end`.
    ///
    /// # Examples
    ///
    /// ```
    /// use torrust_mudlark::{Config, GvGraph, BasisEdge, ContourRangeEnergy};
    ///
    /// let cfg = Config {
    ///     split_threshold: 2u64,
    ///     depth_create: 3,
    ///     depth_evict: 6,
    ///     budget: None,
    ///     alpha_relax: 0.75,
    ///     bounded_eviction: true,
    /// };
    /// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// for _ in 0..10 {
    ///     g.observe(16, 5u64);
    ///     g.observe(48, 5u64);
    /// }
    ///
    /// let plateaus = g.plateaus();
    /// let first = *plateaus.keys().next().unwrap();
    /// let end = BasisEdge(256u64);  // domain sentinel = 2^N
    ///
    /// let e = g.contour_range_energy(first, end).unwrap();
    /// // Full-domain: energy == exact_energy (no boundary thatching).
    /// assert_eq!(e.energy, e.exact_energy);
    /// assert!(e.energy > 0);
    /// ```
    #[must_use]
    pub fn contour_range_energy(&self, start: BasisEdge<C>, end: BasisEdge<C>) -> Option<ContourRangeEnergy<V>> {
        let cr = self.contour_range(start, end)?;
        Some(ContourRangeEnergy {
            energy: cr.energy,
            exact_energy: cr.exact_energy,
            plateau_energy: cr.plateau_energy,
            cross_plateau_energy: cr.cross_plateau_energy,
            plateau_count: cr.plateau_count,
        })
    }

    /// Plateau selection (§CR.12): given an arbitrary dyadic range
    /// `[lo, hi)`, find the contiguous run of overlapping plateaus
    /// and return their lattice-aligned endpoints.
    ///
    /// The returned pair `(start, end)` are valid [`BasisEdge`]s for
    /// [`contour_range()`](Self::contour_range) and
    /// [`contour_range_energy()`](Self::contour_range_energy).
    ///
    /// Returns `None` if:
    /// - The plateau map is empty (no observations).
    /// - `lo >= hi`.
    /// - No plateau overlaps `[lo, hi)`.
    ///
    /// Cost: $O(\log P)$ — two lookups in the plateau ordered map.
    ///
    /// # Examples
    ///
    /// ```
    /// use torrust_mudlark::{Config, GvGraph, BasisEdge};
    ///
    /// let cfg = Config {
    ///     split_threshold: 2u64,
    ///     depth_create: 3,
    ///     depth_evict: 6,
    ///     budget: None,
    ///     alpha_relax: 0.75,
    ///     bounded_eviction: true,
    /// };
    /// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// for _ in 0..10 {
    ///     g.observe(16, 5u64);
    ///     g.observe(48, 5u64);
    /// }
    ///
    /// // Arbitrary coordinates → lattice-aligned contour range endpoints.
    /// if let Some((start, end)) = g.select_plateaus(10, 60) {
    ///     let cr = g.contour_range(start, end).unwrap();
    ///     assert!(cr.basis.len() >= 1);
    /// }
    /// ```
    #[must_use]
    pub fn select_plateaus(&self, lo: C, hi: C) -> Option<(BasisEdge<C>, BasisEdge<C>)> {
        use std::ops::Bound;

        if lo >= hi {
            return None;
        }

        let plateaus = self.plateaus();
        if plateaus.is_empty() {
            return None;
        }

        // §CR.12: find the largest a_j ≤ lo  (floor index)
        let start_entry = plateaus.range(..=BasisEdge(lo)).next_back().map(|(k, _)| *k)?;

        // §CR.12: find the last plateau whose basis edge is < hi.
        let last_plateau_key = plateaus.range(..BasisEdge(hi)).next_back().map(|(k, _)| *k)?;

        // The end endpoint is the next key after last_plateau_key,
        // or the domain sentinel 2^N.
        let end = plateaus
            .range((Bound::Excluded(last_plateau_key), Bound::Unbounded))
            .next()
            .map_or_else(|| BasisEdge(C::domain_max(N)), |(k, _)| *k);

        // Sanity: start < end (should always hold given the above logic).
        if start_entry >= end {
            return None;
        }

        Some((start_entry, end))
    }

    /// Recursive G-Tree descent for basis set collection (§CR.8.1).
    ///
    /// Collects the minimal G-node cover of `[query_lo, query_hi)`.
    /// Semi-internal nodes with an uncovered half overlapping the
    /// range are flagged as boundary thatching elements (§CR.3.2).
    fn decompose_basis(&self, gid: GNodeId, query_lo: C, query_hi: C, basis: &mut Vec<BasisElement<C, V>>) {
        let g = self.gnodes.get(gid.index());

        // 1. Disjoint — no overlap with query range.
        if query_lo >= g.hi || query_hi <= g.lo {
            return;
        }

        // 2. Fully contained — this is a basis element (not thatching).
        if query_lo <= g.lo && query_hi >= g.hi {
            basis.push(BasisElement {
                gnode_id: gid,
                start: g.lo,
                end: g.hi,
                own: g.own,
                sum: g.sum,
                depth: gnode_depth_from_interval(g.lo, g.hi, N),
                is_boundary_thatch: false,
            });
            return;
        }

        // 3. Partial overlap.
        let mid = C::midpoint(g.lo, g.hi);

        // ── Semi-internal early selection guard (§CR.8.1, §CR.8.4) ──
        //
        // When a semi-internal node is not fully contained but both
        // its present-child half and absent-child half overlap the
        // range, select it directly and return.  Without this guard
        // the algorithm would recurse into the present child (selecting
        // descendants) and then also select the node for the absent
        // child — producing an ancestor–descendant pair that violates
        // CR-I4 and double-counts energy.
        //
        // The guard cannot fire for single-plateau ranges (§CR.8.4).
        let left_absent = g.left.is_none();
        let right_absent = g.right.is_none();
        if left_absent != right_absent {
            // Exactly one child absent ⇒ semi-internal.
            // Check if both halves overlap the query range.
            let l_lo = if query_lo > g.lo { query_lo } else { g.lo };
            let l_hi = if query_hi < mid { query_hi } else { mid };
            let r_lo = if query_lo > mid { query_lo } else { mid };
            let r_hi = if query_hi < g.hi { query_hi } else { g.hi };
            if l_lo < l_hi && r_lo < r_hi {
                // Both halves overlap — select the node directly.
                // Coverage: g's interval clipped to the query range (§CR.2.2).
                basis.push(BasisElement {
                    gnode_id: gid,
                    start: l_lo, // = max(g.lo, query_lo)
                    end: r_hi,   // = min(g.hi, query_hi)
                    own: g.own,
                    sum: g.sum,
                    depth: gnode_depth_from_interval(g.lo, g.hi, N),
                    is_boundary_thatch: true,
                });
                return;
            }
        }

        // ── Left child ──────────────────────────────────────────
        if let Some(left_id) = g.left {
            self.decompose_basis(left_id, query_lo, query_hi, basis);
        } else {
            // No left child — semi-internal.  Uncovered half is [g.lo, mid).
            // If this overlaps the query, it's a boundary thatching element.
            let tile_lo = if query_lo > g.lo { query_lo } else { g.lo };
            let tile_hi = if query_hi < mid { query_hi } else { mid };
            if tile_lo < tile_hi {
                basis.push(BasisElement {
                    gnode_id: gid,
                    start: tile_lo,
                    end: tile_hi,
                    own: g.own,
                    sum: g.sum,
                    depth: gnode_depth_from_interval(g.lo, g.hi, N),
                    is_boundary_thatch: true,
                });
                return; // §CR.3.2: node used as thatching, don't also check right
            }
        }

        // ── Right child ─────────────────────────────────────────
        if let Some(right_id) = g.right {
            self.decompose_basis(right_id, query_lo, query_hi, basis);
        } else {
            // No right child — semi-internal.  Uncovered half is [mid, g.hi).
            let tile_lo = if query_lo > mid { query_lo } else { mid };
            let tile_hi = if query_hi < g.hi { query_hi } else { g.hi };
            if tile_lo < tile_hi {
                // Defensive: the return in the left-else branch prevents
                // reaching here for nodes where both children are null AND
                // was already pushed as boundary thatch.
                debug_assert!(
                    basis.last().is_none_or(|b| b.gnode_id != gid),
                    "double push for gnode {gid:?}",
                );
                basis.push(BasisElement {
                    gnode_id: gid,
                    start: tile_lo,
                    end: tile_hi,
                    own: g.own,
                    sum: g.sum,
                    depth: gnode_depth_from_interval(g.lo, g.hi, N),
                    is_boundary_thatch: true,
                });
            }
        }
    }
}
