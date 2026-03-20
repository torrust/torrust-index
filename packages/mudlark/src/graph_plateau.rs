// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Plateau tracking `impl GvGraph` methods.
//!
//! This module contains all `dynamic-contour-tracking` feature code
//! and the non-feature `build_plateaus` fallback.  Separated from
//! `graph.rs` per ADR-M-030 Phase B to isolate the ~1,000 lines of
//! plateau bookkeeping from the core struct definition.
//!
//! All methods are `impl GvGraph<C, V, N>` — Rust resolves them
//! identically to methods defined in `graph.rs`.

use std::borrow::Cow;
use std::collections::BTreeMap;

use crate::graph::{GvGraph, uniform_contour_depth_of};
use crate::handle::GNodeId;
#[cfg(feature = "dynamic-contour-tracking")]
use crate::plateau::PlateauBasis;
use crate::plateau::{BasisEdge, Plateau};
use crate::traits::{Accumulator, Coordinate, Inspectable};

impl<C: Coordinate, V: Accumulator + Inspectable, const N: u32> GvGraph<C, V, N> {
    // ── Plateau accessor ────────────────────────────────────────

    /// The plateau mirror of the G-Tree bottom contour.
    ///
    /// Keys are `BasisEdge<C>` (P-I1) and tile the domain `[0, 2^N)`.
    /// Standard `BTreeMap` queries work directly on the returned map:
    ///
    /// - **Floor-key:** `plateaus.range(..=BasisEdge(x)).next_back()`
    /// - **Range:** `plateaus.range(BasisEdge(a)..BasisEdge(b))`
    /// - **Count:** `plateaus.len()`
    ///
    /// With `dynamic-contour-tracking` (default): O(1) borrow of the
    /// actively-maintained mirror.
    ///
    /// Without `dynamic-contour-tracking`: O(G) on-demand build from
    /// the current G-tree state. Caller owns the returned map.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph, BasisEdge};
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// // Fresh graph has exactly one plateau covering the full domain.
    /// let p = g.plateaus();
    /// assert_eq!(p.len(), 1);
    ///
    /// // After enough observations the contour refines.
    /// for i in 0..10 {
    ///     g.observe(i * 25, 10u64);
    /// }
    /// let p = g.plateaus();
    /// assert!(p.len() >= 1);
    ///
    /// // Floor-key lookup: find the plateau containing coordinate 42.
    /// let hit = p.range(..=BasisEdge(42u64)).next_back();
    /// assert!(hit.is_some());
    /// ```
    #[must_use]
    #[allow(clippy::missing_const_for_fn)] // Cow::Borrowed is not const-compatible
    pub fn plateaus(&self) -> Cow<'_, BTreeMap<BasisEdge<C>, Plateau<C, V>>> {
        #[cfg(feature = "dynamic-contour-tracking")]
        {
            self.debug_assert_plateau_mirror_consistency("plateaus()");
            Cow::Borrowed(&self.plateaus)
        }
        #[cfg(not(feature = "dynamic-contour-tracking"))]
        {
            Cow::Owned(self.build_plateaus())
        }
    }

    // ── Mirror consistency check ────────────────────────────────

    /// Assert that the dynamically-maintained plateau mirror matches
    /// a freshly-built version from the G-tree state.
    ///
    /// Only compiled in debug builds with `dynamic-contour-tracking`.
    /// The `label` argument is included in any panic message to help
    /// identify which mutation path caused divergence.
    #[cfg(feature = "dynamic-contour-tracking")]
    pub(crate) fn debug_assert_plateau_mirror_consistency(&self, label: &str) {
        // Always run in debug builds (cargo test); optionally in
        // release when a tracing subscriber is active at DEBUG.
        if !cfg!(debug_assertions) && !tracing::enabled!(tracing::Level::DEBUG) {
            return;
        }

        let rebuilt = self.build_plateaus();
        if self.plateaus != rebuilt {
            // Find differing keys
            let dyn_keys: std::collections::BTreeSet<_> = self.plateaus.keys().collect();
            let stat_keys: std::collections::BTreeSet<_> = rebuilt.keys().collect();
            let only_dynamic: Vec<_> = dyn_keys.difference(&stat_keys).collect();
            let only_static: Vec<_> = stat_keys.difference(&dyn_keys).collect();
            let both: Vec<_> = dyn_keys.intersection(&stat_keys).collect();
            let differing: Vec<_> = both.iter().filter(|&&k| self.plateaus.get(k) != rebuilt.get(k)).collect();

            tracing::error!(
                label,
                only_dynamic = ?only_dynamic,
                only_static = ?only_static,
                differing = ?differing,
                "PLATEAU DIVERGENCE DETECTED",
            );

            for &&key in &only_dynamic {
                let p = &self.plateaus[key];
                let basis = self.plateau_basis.basis_elements(key);
                tracing::error!(
                    key = ?key,
                    depth = p.depth,
                    start = ?p.start,
                    end = ?p.end,
                    sum = ?p.sum,
                    basis = ?basis.iter().map(|g| g.index()).collect::<Vec<_>>(),
                    "DYNAMIC-ONLY plateau",
                );
                for &gid in basis {
                    if self.gnodes.is_occupied(gid.index()) {
                        let g = self.gnodes.get(gid.index());
                        tracing::error!(
                            gid = gid.index(),
                            state = ?g.state(),
                            lo = ?g.lo,
                            hi = ?g.hi,
                            sum = ?g.sum,
                            "  basis element",
                        );
                    }
                }
            }

            for &&key in &only_static {
                let p = &rebuilt[key];
                tracing::error!(
                    key = ?key,
                    depth = p.depth,
                    start = ?p.start,
                    end = ?p.end,
                    sum = ?p.sum,
                    "STATIC-ONLY plateau",
                );
            }

            for &&&key in &differing {
                let dyn_p = &self.plateaus[key];
                let stat_p = &rebuilt[key];
                let basis = self.plateau_basis.basis_elements(key);
                tracing::error!(
                    key = ?key,
                    dyn_depth = dyn_p.depth,
                    dyn_start = ?dyn_p.start,
                    dyn_end = ?dyn_p.end,
                    dyn_sum = ?dyn_p.sum,
                    stat_depth = stat_p.depth,
                    stat_start = ?stat_p.start,
                    stat_end = ?stat_p.end,
                    stat_sum = ?stat_p.sum,
                    basis = ?basis.iter().map(|g| g.index()).collect::<Vec<_>>(),
                    "DIFFERS",
                );
                for &gid in basis {
                    if self.gnodes.is_occupied(gid.index()) {
                        let g = self.gnodes.get(gid.index());
                        let ud = uniform_contour_depth_of(&self.gnodes, gid, N);
                        tracing::error!(
                            gid = gid.index(),
                            state = ?g.state(),
                            lo = ?g.lo,
                            hi = ?g.hi,
                            sum = ?g.sum,
                            uniform_depth = ?ud,
                            "  basis element",
                        );
                    }
                }
            }

            tracing::error!(dump = %crate::invariants::dump_gtree::<C, V, N>(self), "G-TREE DUMP");

            panic!(
                "{label}: dynamic-contour-tracking mirror diverged from static rebuild\n\
                 left (dynamic): {:#?}\n\
                 right (static): {:#?}",
                self.plateaus, rebuilt
            );
        }
    }

    // ── Build plateaus (unconditional) ──────────────────────────

    /// Build the plateau contour from the current G-tree state.
    ///
    /// Produces an identical `BTreeMap` to the actively-maintained
    /// mirror by finding the **deterministic basis** for each plateau
    /// per §IDEA M-5.6.1:
    ///
    /// 1. **Terminal** nodes are basis elements of their plateau.
    /// 2. **Semi-internal** nodes are *always* basis elements; their
    ///    child subtree also contributes its own basis elements.
    /// 3. **One-level balanced internals** — internal nodes whose
    ///    both children are terminals at the same depth — are single
    ///    basis elements (matching `plateau_after_bootstrap_split`).
    ///    Deeper balanced subtrees are decomposed into their one-level
    ///    balanced children.
    ///
    /// After collecting basis elements, a left-to-right merge pass
    /// fuses adjacent same-depth entries — matching the semantics of
    /// `place_basis_element`.
    ///
    /// Cost: O(G + B log B) where G = total G-nodes, B = basis size.
    ///
    /// # Panics
    ///
    /// Panics if an `Internal` G-node is missing a child pointer.
    #[doc(hidden)]
    #[must_use]
    pub fn build_plateaus(&self) -> BTreeMap<BasisEdge<C>, Plateau<C, V>> {
        use crate::gnode::GState;
        use crate::gtree::gnode_depth_from_interval;
        use crate::plateau::basis_edge_of;

        // 1. DFS collecting basis elements.
        //    One-level balanced internals (both children terminals at
        //    the same depth) are used as single basis elements.
        //    All other internals are recursed through.
        let mut basis: Vec<(BasisEdge<C>, u32, C, C, V)> = Vec::new();
        let mut stack = vec![self.g_root];
        while let Some(gid) = stack.pop() {
            let g = self.gnodes.get(gid.index());
            match g.state() {
                GState::Terminal => {
                    let depth = gnode_depth_from_interval(g.lo, g.hi, N);
                    basis.push((BasisEdge(g.lo), depth, g.lo, g.hi, g.sum));
                }
                GState::SemiInternal => {
                    // Semi-internal is always a basis element.
                    let depth = gnode_depth_from_interval(g.lo, g.hi, N);
                    basis.push((basis_edge_of(g), depth, g.lo, g.hi, g.sum));
                    // Recurse into child subtree for its own basis elements.
                    if let Some(left) = g.left {
                        stack.push(left);
                    }
                    if let Some(right) = g.right {
                        stack.push(right);
                    }
                }
                GState::Internal => {
                    // Uniform-depth subtree: all leaves at the same
                    // depth → single basis element (P-I2 minimal).
                    // This matches consolidate_all_basis behaviour
                    // which merges multi-level balanced subtrees.
                    if let Some(ud) = uniform_contour_depth_of(&self.gnodes, gid, N) {
                        basis.push((basis_edge_of(g), ud, g.lo, g.hi, g.sum));
                    } else {
                        // Non-uniform: decompose into children.
                        if let Some(left) = g.left {
                            stack.push(left);
                        }
                        if let Some(right) = g.right {
                            stack.push(right);
                        }
                    }
                }
            }
        }

        // 2. Sort by basis edge (left-to-right spatial order).
        basis.sort_by_key(|a| a.0);

        // 3. Left-to-right sweep: merge adjacent same-depth entries.
        //    Matches `place_basis_element`'s left-neighbour merge rule.
        let mut result: BTreeMap<BasisEdge<C>, Plateau<C, V>> = BTreeMap::new();
        for (be, depth, lo, hi, sum) in basis {
            let merged = if let Some((_, prev)) = result.iter_mut().next_back() {
                if prev.depth == depth {
                    if hi.total_cmp(&prev.end) == std::cmp::Ordering::Greater {
                        prev.end = hi;
                    }
                    if lo.total_cmp(&prev.start) == std::cmp::Ordering::Less {
                        prev.start = lo;
                    }
                    prev.sum = V::add(prev.sum, sum);
                    true
                } else {
                    false
                }
            } else {
                false
            };
            if !merged {
                result.insert(
                    be,
                    Plateau {
                        basis_edge: be,
                        start: lo,
                        end: hi,
                        depth,
                        sum,
                    },
                );
            }
        }

        result
    }

    // ── Basis accessors ─────────────────────────────────────────

    /// Crate-internal access to the plateau basis bookkeeping.
    #[cfg(feature = "dynamic-contour-tracking")]
    #[must_use]
    #[inline]
    pub(crate) const fn plateau_basis(&self) -> &PlateauBasis<C> {
        &self.plateau_basis
    }

    /// Diagnostic: for each plateau, return the `BasisEdge` key and
    /// the list of `(GNodeId_index, lo, hi, state, g_depth)` tuples
    /// for its basis elements.
    ///
    /// This exposes `pub(crate)` basis bookkeeping for integration
    /// tests that need to diagnose plateau invariant violations.
    #[doc(hidden)]
    #[cfg(feature = "dynamic-contour-tracking")]
    #[must_use]
    #[allow(clippy::type_complexity)]
    pub fn debug_plateau_basis(&self) -> Vec<(BasisEdge<C>, Vec<(usize, C, C, &'static str, u32)>)> {
        let mut result = Vec::new();
        for &key in self.plateaus.keys() {
            let elements = self.plateau_basis.basis_elements(&key);
            let infos: Vec<_> = elements
                .iter()
                .map(|&gid| {
                    let g = self.gnodes.get(gid.index());
                    let state_str = match g.state() {
                        crate::gnode::GState::Terminal => "Terminal",
                        crate::gnode::GState::Internal => "Internal",
                        crate::gnode::GState::SemiInternal => "SemiInternal",
                    };
                    let g_depth = crate::gtree::gnode_depth_from_interval(g.lo, g.hi, N);
                    (gid.index(), g.lo, g.hi, state_str, g_depth)
                })
                .collect();
            result.push((key, infos));
        }
        result
    }

    // ── Recompute + placement ───────────────────────────────────

    /// Recompute `start`, `end`, `sum`, `depth` for a plateau from
    /// its current basis elements.
    ///
    /// Called after basis-membership changes. Does NOT re-key the
    /// `BTreeMap` entry — the caller must handle key changes.
    ///
    /// No-op if the plateau has no basis elements (caller should
    /// remove the `BTreeMap` entry instead).
    #[cfg(feature = "dynamic-contour-tracking")]
    pub(crate) fn recompute_plateau(&mut self, key: &BasisEdge<C>) {
        let elements = self.plateau_basis.basis_elements(key);
        if elements.is_empty() {
            return;
        }

        let first = *elements.iter().next().unwrap();
        let mut min_lo = self.gnodes.get(first.index()).lo;
        let mut max_hi = self.gnodes.get(first.index()).hi;
        let mut sum = V::zero();
        let mut depth = 0u32;

        for &gid in elements {
            let g = self.gnodes.get(gid.index());
            if g.lo.total_cmp(&min_lo) == std::cmp::Ordering::Less {
                min_lo = g.lo;
            }
            if g.hi.total_cmp(&max_hi) == std::cmp::Ordering::Greater {
                max_hi = g.hi;
            }
            sum = V::add(sum, g.sum);

            // Depth: contour depth contributed by this basis element.
            // Terminal / Semi-internal: own depth.
            // Internal (balanced): actual uniform contour depth of
            //   the subtree — may be deeper than own_depth + 1 after
            //   consolidation merges multi-level balanced subtrees.
            let d = match g.state() {
                crate::gnode::GState::Terminal | crate::gnode::GState::SemiInternal => {
                    crate::gtree::gnode_depth_from_interval(g.lo, g.hi, N)
                }
                crate::gnode::GState::Internal => uniform_contour_depth_of(&self.gnodes, gid, N)
                    .unwrap_or_else(|| crate::gtree::gnode_depth_from_interval(g.lo, g.hi, N) + 1),
            };
            depth = depth.max(d);
        }

        if let Some(p) = self.plateaus.get_mut(key) {
            p.start = min_lo;
            p.end = max_hi;
            p.sum = sum;
            p.depth = depth;
        }
    }

    /// Place a G-node as a new basis element into the correct
    /// plateau, merging with same-depth neighbours when possible.
    ///
    /// | Left match | Right match | Action                          |
    /// |:---:|:---:|:---|
    /// | ✓ | ✓ | Merge element + right into left   |
    /// | ✓ | ✗ | Insert into left, recompute       |
    /// | ✗ | ✓ | Re-key right to our key, merge    |
    /// | ✗ | ✗ | Create a brand-new plateau        |
    ///
    /// Called by split / evict plateau maintenance (Step 2).
    #[cfg(feature = "dynamic-contour-tracking")]
    pub(crate) fn place_basis_element(&mut self, gnode: GNodeId, depth: u32) {
        use crate::plateau::{BasisEdge, Plateau, basis_edge_of};

        let g = self.gnodes.get(gnode.index());
        let key = basis_edge_of(g);
        let lo = g.lo;
        let hi = g.hi;
        let sum = g.sum;

        // Left neighbour at the same depth AND spatially adjacent.
        // Adjacent means: left_plateau.end >= our lo (they touch or overlap).
        // Also ensure no intervening plateau key exists between the
        // left plateau's key and ours — an intervening plateau at a
        // different depth (e.g. a SemiInternal's child) means the
        // merge would skip over it, producing a plateau that
        // `build_plateaus`'s strict sorted sweep would never create.
        let left_key = self
            .plateaus
            .range(..key)
            .next_back()
            .filter(|(_, p)| p.depth == depth && p.end.total_cmp(&lo) != std::cmp::Ordering::Less)
            .map(|(&k, _)| k)
            .filter(|lk| {
                // Check for intervening plateaus between lk and key.
                self.plateaus
                    .range((std::ops::Bound::Excluded(*lk), std::ops::Bound::Excluded(key)))
                    .next()
                    .is_none()
            });

        // Right neighbour at the same depth AND spatially adjacent.
        // Adjacent means: right_plateau.start <= our hi (they touch or overlap).
        // Same intervening-plateau guard as above.
        let right_key = self
            .plateaus
            .range(BasisEdge(hi)..)
            .next()
            .filter(|(_, p)| p.depth == depth && p.start.total_cmp(&hi) != std::cmp::Ordering::Greater)
            .map(|(&k, _)| k)
            .filter(|rk| {
                // Check for intervening plateaus between key and rk.
                self.plateaus
                    .range((std::ops::Bound::Excluded(key), std::ops::Bound::Excluded(*rk)))
                    .next()
                    .is_none()
            });

        match (left_key, right_key) {
            (Some(lk), Some(rk)) => {
                // Merge: element joins left, right absorbed into left.
                self.plateau_basis.insert(lk, gnode);
                let rights: Vec<_> = self.plateau_basis.basis_elements(&rk).iter().copied().collect();
                for rid in rights {
                    self.plateau_basis.remove(rid);
                    self.plateau_basis.insert(lk, rid);
                }
                self.plateaus.remove(&rk);
                self.recompute_plateau(&lk);
                tracing::trace!(gnode = gnode.index(), ?lk, ?rk, "place_basis_element: merge-both");
            }
            (Some(lk), None) => {
                self.plateau_basis.insert(lk, gnode);
                self.recompute_plateau(&lk);
                tracing::trace!(gnode = gnode.index(), ?lk, "place_basis_element: insert-left");
            }
            (None, Some(rk)) => {
                // Our element is left of the right neighbour → re-key.
                let rights: Vec<_> = self.plateau_basis.basis_elements(&rk).iter().copied().collect();
                for rid in rights {
                    self.plateau_basis.remove(rid);
                    self.plateau_basis.insert(key, rid);
                }
                self.plateau_basis.insert(key, gnode);
                self.plateaus.remove(&rk);
                self.plateaus.insert(
                    key,
                    Plateau {
                        basis_edge: key,
                        start: lo,
                        end: lo, // recomputed below
                        depth,
                        sum,
                    },
                );
                self.recompute_plateau(&key);
                tracing::trace!(gnode = gnode.index(), ?rk, "place_basis_element: rekey-right");
            }
            (None, None) => {
                self.plateau_basis.insert(key, gnode);
                if let std::collections::btree_map::Entry::Vacant(e) = self.plateaus.entry(key) {
                    e.insert(Plateau {
                        basis_edge: key,
                        start: lo,
                        end: hi,
                        depth,
                        sum,
                    });
                } else {
                    // A plateau already exists at this key but at a
                    // different depth (the left/right neighbour checks
                    // skipped it because of the depth mismatch).
                    // Join the existing plateau and recompute.
                    self.recompute_plateau(&key);
                }
                tracing::trace!(gnode = gnode.index(), "place_basis_element: new-plateau");
            }
        }

        // Consolidate: if gnode's tree-structural sibling is also a
        // basis element of the same plateau, merge both into the
        // parent — maintaining the minimal basis (§IDEA M-5.6.1 rule 2).
        self.consolidate_basis_up(gnode);

        // If the placed node is semi-internal, it may violate P-I4
        // (thatch one-hop).  Queue it for the drain in repair_p_i4.
        // (consolidation never removes semi-internals, so this is safe.)
        if self.gnodes.get(gnode.index()).state() == crate::gnode::GState::SemiInternal {
            let final_key = self.plateau_basis.plateau_key(gnode).expect("just placed");
            self.pending_p_i4.push((gnode, final_key));
        }
    }

    /// Collect `(GNodeId, u32)` pairs for a subtree's basis elements
    /// without placing them.  Read-only counterpart to
    /// `place_subtree_basis_elements`.
    ///
    /// # `SemiInternal` non-recursion
    ///
    /// For `SemiInternal` nodes this emits the SI itself but does
    /// **not** recurse into its child.  This is correct by
    /// construction: the SI at depth *d* and its child at depth
    /// *d + 1* always land in **different** plateaus (the merge
    /// rule only fuses adjacent same-depth entries).  In every
    /// production call-site the SI's child is already an
    /// independent basis element in another plateau, so recursing
    /// would double-count it and trigger a
    /// `PlateauBasis::insert` panic.
    ///
    /// Four structural invariants guarantee the child is always
    /// independently tracked:
    ///
    /// 1. `uniform_contour_depth_of` returns `None` for any
    ///    subtree containing an SI, so no ancestor can mask the
    ///    child as part of a uniform basis element.
    /// 2. `consolidate_basis_up` explicitly breaks at SI children,
    ///    preventing bottom-up consolidation from absorbing them.
    /// 3. The depth-difference invariant (child = parent + 1)
    ///    means the two are never merge-eligible.
    /// 4. `normalize_plateaus` (called at the end of every
    ///    `observe`) rebuilds from scratch with a `seen` set,
    ///    correcting any transient inconsistency.
    ///
    /// ADR-M-031 Phase 1a: enables callers to collect elements, sort by
    /// `BasisEdge`, and place left-to-right via `place_sorted`.
    #[cfg(feature = "dynamic-contour-tracking")]
    pub(crate) fn collect_subtree_basis_elements(&self, gid: GNodeId, out: &mut Vec<(GNodeId, u32)>) {
        use crate::gnode::GState;
        let g = self.gnodes.get(gid.index());
        match g.state() {
            GState::Terminal | GState::SemiInternal => {
                let depth = crate::gtree::gnode_depth_from_interval(g.lo, g.hi, N);
                out.push((gid, depth));
            }
            GState::Internal => {
                if let Some(ud) = uniform_contour_depth_of(&self.gnodes, gid, N) {
                    out.push((gid, ud));
                } else {
                    if let Some(l) = g.left {
                        self.collect_subtree_basis_elements(l, out);
                    }
                    if let Some(r) = g.right {
                        self.collect_subtree_basis_elements(r, out);
                    }
                }
            }
        }
    }

    /// Sort collected `(GNodeId, u32)` pairs by `BasisEdge` and place
    /// left-to-right.  With sorted placement the existing
    /// left-neighbour merge in `place_basis_element` is sufficient to
    /// produce the correct partition — no post-hoc normalize is needed.
    ///
    /// ADR-M-031 Phase 1b.
    #[cfg(feature = "dynamic-contour-tracking")]
    pub(crate) fn place_sorted(&mut self, elements: &mut [(GNodeId, u32)]) {
        use crate::plateau::basis_edge_of;
        elements.sort_by(|a, b| {
            let a_key = basis_edge_of(self.gnodes.get(a.0.index()));
            let b_key = basis_edge_of(self.gnodes.get(b.0.index()));
            a_key.cmp(&b_key)
        });
        for &(gid, depth) in elements.iter() {
            self.place_basis_element(gid, depth);
        }
    }

    /// Recursively place a subtree's basis elements.
    ///
    /// For Terminal/SemiInternal nodes, places them directly.
    /// For Internal nodes, checks if the subtree is uniform:
    /// - If uniform: places the node as a single basis element.
    /// - If non-uniform: recursively decomposes into children.
    ///
    /// This matches the logic in `build_plateaus()` and ensures
    /// Internal nodes with non-uniform subtrees are properly split.
    ///
    /// Note: after ADR-M-031, all callers use `collect_subtree_basis_elements`
    /// + `place_sorted` instead. Retained for potential future use.
    #[cfg(feature = "dynamic-contour-tracking")]
    #[allow(dead_code)]
    pub(crate) fn place_subtree_basis_elements(&mut self, gid: GNodeId) {
        use crate::gnode::GState;

        let g = self.gnodes.get(gid.index());
        match g.state() {
            GState::Terminal | GState::SemiInternal => {
                let depth = crate::gtree::gnode_depth_from_interval(g.lo, g.hi, N);
                self.place_basis_element(gid, depth);
            }
            GState::Internal => {
                // Check if subtree is uniform
                if let Some(ud) = uniform_contour_depth_of(&self.gnodes, gid, N) {
                    // Uniform subtree: place as single basis element
                    self.place_basis_element(gid, ud);
                } else {
                    // Non-uniform: decompose into children
                    let (left, right) = {
                        let g = self.gnodes.get(gid.index());
                        (g.left, g.right)
                    };
                    if let Some(l) = left {
                        self.place_subtree_basis_elements(l);
                    }
                    if let Some(r) = right {
                        self.place_subtree_basis_elements(r);
                    }
                }
            }
        }
    }

    /// No-op stub when plateau tracking is compiled out.
    #[cfg(not(feature = "dynamic-contour-tracking"))]
    #[inline(always)]
    #[allow(dead_code, clippy::unused_self, clippy::needless_pass_by_ref_mut)]
    pub(crate) const fn place_subtree_basis_elements(&mut self, _gid: GNodeId) {}

    // ── Consolidation + normalization ───────────────────────────

    /// Walk up from `gid`, consolidating pairs of sibling basis
    /// elements into their parent when the parent's entire subtree
    /// has uniform contour depth.
    ///
    /// This maintains the **minimal** basis per §IDEA M-5.6.1 rule 2:
    /// "No ancestor of R also satisfies condition 1 for P."
    ///
    /// The consolidation is safe because we verify
    /// `uniform_contour_depth_of(parent)` before merging.  Two
    /// children can be in the same plateau (via `own_depth + 1`)
    /// *without* the parent's full subtree being uniform — e.g. when
    /// a child is Internal with deeper descendants that feed other
    /// plateaus.  The uniform check catches this.
    #[cfg(feature = "dynamic-contour-tracking")]
    #[allow(clippy::too_many_lines)]
    fn consolidate_basis_up(&mut self, mut gid: GNodeId) {
        use crate::gnode::GState;

        loop {
            let Some(parent_id) = self.gnodes.get(gid.index()).parent else {
                tracing::trace!(from = gid.index(), "consolidate_basis_up: stop — no parent");
                break;
            };
            if self.gnodes.get(parent_id.index()).state() != GState::Internal {
                tracing::trace!(
                    from = gid.index(),
                    parent = parent_id.index(),
                    parent_state = ?self.gnodes.get(parent_id.index()).state(),
                    "consolidate_basis_up: stop — parent not Internal",
                );
                break;
            }
            let (left, right) = {
                let pg = self.gnodes.get(parent_id.index());
                match (pg.left, pg.right) {
                    (Some(l), Some(r)) => (l, r),
                    _ => break,
                }
            };

            // Semi-internals break uniformity — cannot consolidate.
            if self.gnodes.get(left.index()).state() == GState::SemiInternal
                || self.gnodes.get(right.index()).state() == GState::SemiInternal
            {
                tracing::trace!(
                    from = gid.index(),
                    parent = parent_id.index(),
                    "consolidate_basis_up: stop — semi-internal child"
                );
                break;
            }

            // Both children must be basis elements in the same plateau.
            let Some(left_key) = self.plateau_basis.plateau_key(left) else {
                tracing::trace!(
                    from = gid.index(),
                    parent = parent_id.index(),
                    left = left.index(),
                    "consolidate_basis_up: stop — left not in basis"
                );
                break;
            };
            let Some(right_key) = self.plateau_basis.plateau_key(right) else {
                tracing::trace!(
                    from = gid.index(),
                    parent = parent_id.index(),
                    right = right.index(),
                    "consolidate_basis_up: stop — right not in basis"
                );
                break;
            };
            if left_key != right_key {
                tracing::trace!(
                    from = gid.index(),
                    parent = parent_id.index(),
                    ?left_key,
                    ?right_key,
                    "consolidate_basis_up: stop — children in different plateaus"
                );
                break;
            }

            // Guard: parent's subtree must actually be uniform depth.
            let Some(uniform_depth) = uniform_contour_depth_of(&self.gnodes, parent_id, N) else {
                tracing::trace!(
                    from = gid.index(),
                    parent = parent_id.index(),
                    "consolidate_basis_up: stop — parent subtree not uniform"
                );
                break;
            };

            // Extra sanity: the uniform depth must match the plateau.
            let Some(p) = self.plateaus.get(&left_key) else { break };
            if p.depth != uniform_depth {
                tracing::trace!(
                    from = gid.index(),
                    parent = parent_id.index(),
                    plateau_depth = p.depth,
                    uniform_depth,
                    "consolidate_basis_up: stop — depth mismatch"
                );
                break;
            }

            // Consolidate: replace both children with the parent.
            tracing::debug!(
                left = left.index(),
                right = right.index(),
                parent = parent_id.index(),
                ?left_key,
                uniform_depth,
                "consolidate_basis_up: MERGING siblings into parent",
            );
            let key = left_key;
            self.plateau_basis.remove(left);
            self.plateau_basis.remove(right);
            self.plateau_basis.insert(key, parent_id);
            self.recompute_plateau(&key);

            gid = parent_id;
        }
    }

    /// Rebuild `self.plateaus` and `self.plateau_basis` key-assignments
    /// from the existing basis elements, using the **standard merge**
    /// (adjacent same-depth only).
    ///
    /// The incremental `place_basis_element` path can produce
    /// non-contiguous plateaus when entries are inserted in a
    /// temporal order that differs from `BasisEdge` order.  This
    /// normalisation pass — cheap, O(B log B) — corrects that by
    /// re-grouping the existing basis elements left-to-right, exactly
    /// matching the logic used by `build_plateaus()`.
    ///
    /// ADR-M-031: gated behind `plateaus_dirty` flag.  After sorted
    /// placement was introduced for observe-path callers, this
    /// function is only needed for batch-eviction and decay paths.
    /// Early-returns O(1) when the flag is not set.
    #[cfg(feature = "dynamic-contour-tracking")]
    #[allow(clippy::too_many_lines, clippy::float_cmp)]
    pub(crate) fn normalize_plateaus(&mut self) {
        use crate::gnode::GState;
        use crate::gtree::gnode_depth_from_interval;
        use crate::plateau::{BasisEdge, Plateau, basis_edge_of};

        // ADR-M-031 Phase 4: early return when not dirty.
        if !self.plateaus_dirty {
            return;
        }
        self.plateaus_dirty = false;

        let _span = tracing::debug_span!("normalize_plateaus").entered();

        // ── Step 0: capture old total for conservation check ────
        #[cfg(debug_assertions)]
        let old_total: f64 = {
            let build = self.build_plateaus();
            build.values().map(|p| p.sum.to_f64_approx()).sum()
        };

        // ── Step 1: DFS collection ──────────────────────────────
        //    Internal nodes with non-uniform contour depth are
        //    decomposed into their descendant terminals / balanced
        //    internals (mirroring build_plateaus's DFS logic).
        //
        //    A `seen` set prevents double-collection when a
        //    SemiInternal's DFS recurses into a node that is also
        //    a top-level old basis element.
        let mut elems: Vec<(GNodeId, BasisEdge<C>, u32, C, C, V)> = Vec::new();
        let mut seen = std::collections::HashSet::new();
        let basis_ids: Vec<GNodeId> = self.plateau_basis.back_map().keys().copied().collect();
        for gid in basis_ids {
            let mut stack = vec![gid];
            while let Some(nid) = stack.pop() {
                if !seen.insert(nid) {
                    continue; // already collected by an ancestor's DFS
                }
                let g = self.gnodes.get(nid.index());
                match g.state() {
                    GState::Terminal => {
                        let depth = gnode_depth_from_interval(g.lo, g.hi, N);
                        elems.push((nid, basis_edge_of(g), depth, g.lo, g.hi, g.sum));
                    }
                    GState::SemiInternal => {
                        let depth = gnode_depth_from_interval(g.lo, g.hi, N);
                        elems.push((nid, basis_edge_of(g), depth, g.lo, g.hi, g.sum));
                        // Recurse into child subtree for its own
                        // basis elements.
                        if let Some(left) = g.left {
                            stack.push(left);
                        }
                        if let Some(right) = g.right {
                            stack.push(right);
                        }
                    }
                    GState::Internal => {
                        if let Some(ud) = uniform_contour_depth_of(&self.gnodes, nid, N) {
                            // Uniform subtree: keep as single basis
                            // element.
                            elems.push((nid, basis_edge_of(g), ud, g.lo, g.hi, g.sum));
                        } else {
                            // Non-uniform subtree: decompose into
                            // children (like build_plateaus).
                            if let Some(left) = g.left {
                                stack.push(left);
                            }
                            if let Some(right) = g.right {
                                stack.push(right);
                            }
                        }
                    }
                }
            }
        }
        elems.sort_by_key(|e| e.1);

        // ── Step 1 assert: no duplicate GNodeIds ────────────────
        #[cfg(debug_assertions)]
        {
            let mut ids: Vec<usize> = elems.iter().map(|e| e.0.index()).collect();
            ids.sort_unstable();
            for w in ids.windows(2) {
                debug_assert_ne!(
                    w[0], w[1],
                    "normalize_plateaus step 1: duplicate GNodeId({}) in DFS collection",
                    w[0],
                );
            }
        }

        // ── Step 2: left-to-right sweep ─────────────────────────
        let mut new_plateaus: std::collections::BTreeMap<BasisEdge<C>, Plateau<C, V>> = std::collections::BTreeMap::new();
        let mut assignments: Vec<(BasisEdge<C>, GNodeId)> = Vec::with_capacity(elems.len());

        for (gid, be, depth, lo, hi, sum) in &elems {
            let merge_key = new_plateaus
                .range(..*be)
                .next_back()
                .filter(|(_, p)| p.depth == *depth)
                .map(|(&k, _)| k);

            let key = if let Some(mk) = merge_key {
                let p = new_plateaus.get_mut(&mk).unwrap();
                if hi.total_cmp(&p.end) == std::cmp::Ordering::Greater {
                    p.end = *hi;
                }
                if lo.total_cmp(&p.start) == std::cmp::Ordering::Less {
                    p.start = *lo;
                }
                p.sum = V::add(p.sum, *sum);
                mk
            } else {
                new_plateaus.insert(
                    *be,
                    Plateau {
                        basis_edge: *be,
                        start: *lo,
                        end: *hi,
                        depth: *depth,
                        sum: *sum,
                    },
                );
                *be
            };
            assignments.push((key, *gid));
        }

        // ── Step 2 assert: sweep sum == element sum per key ─────
        #[cfg(debug_assertions)]
        {
            // Group collected element sums by assigned plateau key.
            let mut key_sums: std::collections::BTreeMap<BasisEdge<C>, f64> = std::collections::BTreeMap::new();
            for (i, (key, _gid)) in assignments.iter().enumerate() {
                *key_sums.entry(*key).or_default() += elems[i].5.to_f64_approx();
            }
            for (key, elem_sum) in &key_sums {
                let sweep_sum = new_plateaus[key].sum.to_f64_approx();
                debug_assert!(
                    sweep_sum == *elem_sum || (sweep_sum - elem_sum).abs() < 1e-9,
                    "normalize_plateaus step 2: sweep sum {sweep_sum} != \
                     element sum {elem_sum} for plateau {key:?}",
                );
            }
        }

        // ── Step 3: swap maps ───────────────────────────────────
        self.plateaus = new_plateaus;
        self.plateau_basis.rebuild(assignments);

        // ── Step 3 assert: every plateau sum == Σ basis.sum ─────
        #[cfg(debug_assertions)]
        self.debug_check_plateau_sums("normalize step 3 (post-rebuild)");

        // ── Step 4: consolidation pass (P-I2 minimality) ────────
        self.consolidate_all_basis();

        // ── Step 4 assert: sums still consistent after merges ───
        #[cfg(debug_assertions)]
        self.debug_check_plateau_sums("normalize step 4 (post-consolidate)");

        // ── Conservation: total plateau energy unchanged ─────────
        #[cfg(debug_assertions)]
        {
            let new_total: f64 = {
                let build = self.build_plateaus();
                build.values().map(|p| p.sum.to_f64_approx()).sum()
            };
            debug_assert!(
                old_total == new_total || (old_total - new_total).abs() < 1e-9,
                "normalize_plateaus: total plateau energy changed: \
                 old={old_total}, new={new_total}, delta={}",
                new_total - old_total,
            );
        }
    }

    /// No-op stub when plateau tracking is compiled out.
    #[cfg(not(feature = "dynamic-contour-tracking"))]
    #[inline(always)]
    #[allow(clippy::unused_self, clippy::needless_pass_by_ref_mut)]
    pub(crate) const fn normalize_plateaus(&mut self) {}

    /// Sweep every basis element and attempt to consolidate upward
    /// (promote sibling pairs to their parent).  Enforces P-I2
    /// (minimal deterministic basis) after any pipeline stage that
    /// may have introduced new fine-grained basis elements.
    #[cfg(feature = "dynamic-contour-tracking")]
    pub(crate) fn consolidate_all_basis(&mut self) {
        let basis_snapshot: Vec<GNodeId> = self.plateau_basis.back_map().keys().copied().collect();
        let count = basis_snapshot.len();
        let mut merged = 0u32;
        for gid in basis_snapshot {
            // The element may have been consumed by a prior iteration's
            // consolidation (its sibling was processed first).
            if self.plateau_basis.plateau_key(gid).is_some() {
                let before = self.plateau_basis.basis_count();
                self.consolidate_basis_up(gid);
                let after = self.plateau_basis.basis_count();
                if after < before {
                    #[allow(clippy::cast_possible_truncation)]
                    {
                        merged += (before - after) as u32;
                    }
                }
            }
        }
        if merged > 0 {
            tracing::debug!(
                basis_before = count,
                basis_after = self.plateau_basis.basis_count(),
                merged,
                "consolidate_all_basis: completed",
            );
        }
    }

    /// No-op stub when plateau tracking is compiled out.
    #[cfg(not(feature = "dynamic-contour-tracking"))]
    #[inline(always)]
    #[allow(dead_code, clippy::unused_self, clippy::needless_pass_by_ref_mut)]
    pub(crate) const fn consolidate_all_basis(&mut self) {}

    // ── Diagnostics ─────────────────────────────────────────────

    /// Debug-only: assert that every plateau's tracked sum matches
    /// recomputed `Σ basis_elements.sum`. Panics with a labelled
    /// message identifying the offending plateau.
    #[cfg(feature = "dynamic-contour-tracking")]
    #[allow(clippy::float_cmp)]
    pub(crate) fn debug_check_plateau_sums(&self, label: &str) {
        // Always run in debug builds (cargo test); optionally in
        // release when a tracing subscriber is active at DEBUG.
        if !cfg!(debug_assertions) && !tracing::enabled!(tracing::Level::DEBUG) {
            return;
        }

        for (&key, plateau) in &self.plateaus {
            let elems: Vec<_> = self
                .plateau_basis
                .basis_elements(&key)
                .iter()
                .map(|&gid| {
                    let g = self.gnodes.get(gid.index());
                    (gid.index(), g.sum.to_f64_approx(), format!("{:?}", g.state()))
                })
                .collect();
            let expected: f64 = elems.iter().map(|e| e.1).sum();
            let actual = plateau.sum.to_f64_approx();
            assert!(
                expected == actual || (expected - actual).abs() < 1e-9,
                "{label}: plateau sum mismatch at key {key:?}\n\
                 tracked={actual}, recomputed={expected}\n\
                 basis_elements={elems:?}",
            );
        }
    }

    // ── Fixup + repair ──────────────────────────────────────────

    /// Repair a plateau after one of its basis elements was removed.
    ///
    /// - Empty → remove the `BTreeMap` entry.
    /// - Non-contiguous remainder → evacuate all elements and
    ///   re-place them via `place_sorted` so each contiguous
    ///   group forms its own plateau (or merges with a neighbour).
    /// - Min `BasisEdge` changed → re-key and recompute.
    /// - Otherwise → just recompute.
    #[cfg(feature = "dynamic-contour-tracking")]
    pub(crate) fn fixup_plateau(&mut self, old_key: crate::plateau::BasisEdge<C>) {
        use crate::plateau::{BasisEdge, Plateau, basis_edge_of};

        // Snapshot element IDs up-front so we can mutate `self` below.
        let element_ids: Vec<GNodeId> = self.plateau_basis.basis_elements(&old_key).iter().copied().collect();

        if element_ids.is_empty() {
            self.plateaus.remove(&old_key);
            return;
        }

        // ── Non-contiguity guard ────────────────────────────────
        //
        // When a basis element is removed from a merged plateau the
        // remaining members may become non-contiguous.  A simple
        // bounding-box recompute (`recompute_plateau`) would produce
        // a bogus `end` that spans the gap, causing later
        // `place_basis_element` left-neighbour merges to
        // incorrectly absorb unrelated elements.
        //
        // Detect this by sorting the remaining elements by `lo` and
        // checking that each element's `lo` equals its predecessor's
        // `hi` (i.e. they tile without gaps).  When a gap is found,
        // evacuate every element and re-place the whole set via
        // `place_sorted` — this is O(k log P) where k is the
        // (typically small) number of co-basis members.
        if element_ids.len() > 1 {
            let mut intervals: Vec<(GNodeId, C, C)> = element_ids
                .iter()
                .map(|&gid| {
                    let g = self.gnodes.get(gid.index());
                    (gid, g.lo, g.hi)
                })
                .collect();
            intervals.sort_by(|a, b| a.1.total_cmp(&b.1));

            let contiguous = intervals
                .windows(2)
                .all(|w| w[0].2.total_cmp(&w[1].1) == std::cmp::Ordering::Equal);

            if !contiguous {
                tracing::debug!(
                    ?old_key,
                    n_elements = element_ids.len(),
                    "fixup_plateau: non-contiguous remainder, evacuating"
                );
                let mut displaced: Vec<(GNodeId, u32)> = Vec::with_capacity(element_ids.len());
                for &gid in &element_ids {
                    self.plateau_basis.remove(gid);
                    let g = self.gnodes.get(gid.index());
                    let d = match g.state() {
                        crate::gnode::GState::Terminal | crate::gnode::GState::SemiInternal => {
                            crate::gtree::gnode_depth_from_interval(g.lo, g.hi, N)
                        }
                        crate::gnode::GState::Internal => crate::graph::uniform_contour_depth_of(&self.gnodes, gid, N)
                            .unwrap_or_else(|| crate::gtree::gnode_depth_from_interval(g.lo, g.hi, N) + 1),
                    };
                    displaced.push((gid, d));
                }
                self.plateaus.remove(&old_key);
                self.place_sorted(&mut displaced);
                return;
            }
        }

        // ── Contiguous remainder — normal recompute / re-key ────
        let min_key: BasisEdge<C> = element_ids
            .iter()
            .map(|&gid| basis_edge_of(self.gnodes.get(gid.index())))
            .min()
            .unwrap();

        if min_key == old_key {
            self.recompute_plateau(&old_key);
        } else {
            // Re-key: move all elements to the new key.
            for &eid in &element_ids {
                self.plateau_basis.remove(eid);
            }
            self.plateaus.remove(&old_key);
            for &eid in &element_ids {
                self.plateau_basis.insert(min_key, eid);
            }
            self.plateaus.insert(
                min_key,
                Plateau {
                    basis_edge: min_key,
                    start: C::zero(),
                    end: C::zero(),
                    depth: 0,
                    sum: V::zero(),
                },
            );
            self.recompute_plateau(&min_key);
        }
    }

    /// Repair P-I4 thatch-one-hop violations.
    ///
    /// Two-phase approach:
    /// 1. **Collect** — merge the targeted work-list (from
    ///    `place_basis_element`) with a single scan of all
    ///    semi-internal basis elements.  The scan catches violations
    ///    introduced by tile boundary changes (fixup re-keying,
    ///    plateau removal) that `place_basis_element` cannot detect.
    /// 2. **Drain** — check each candidate at O(log P) and fix via
    ///    `split_for_p_i4`.  New candidates pushed by
    ///    `place_basis_element` during fixes are naturally drained.
    ///
    /// Total: O(`B_semi` + V × log P) where `B_semi` is the number of
    /// semi-internal basis elements and V is the violation count.
    /// Compared to the old O(V × `B_total`) fix-point loop, this
    /// replaces V full scans with one.
    ///
    /// Convergence: each fix adds exactly one new `BTreeMap` key
    /// (monotonically increasing plateau count, bounded by node
    /// count).
    ///
    /// Called after eviction batches and legacy promotes — any
    /// operation that can rearrange plateau boundaries.
    #[cfg(feature = "dynamic-contour-tracking")]
    pub(crate) fn repair_p_i4(&mut self) {
        use crate::gnode::GState;
        use crate::plateau::BasisEdge;

        let span = tracing::debug_span!(
            "repair_p_i4",
            pending = self.pending_p_i4.len(),
            candidates = tracing::field::Empty,
        )
        .entered();

        // Phase 1: one-time scan of all semi-internal basis elements
        // to catch violations from tile boundary changes not tracked
        // by the place_basis_element work-list.
        let mut candidates: Vec<(GNodeId, BasisEdge<C>)> = Vec::new();
        for (&key, elements) in self.plateau_basis.iter() {
            for &gid in elements {
                if self.gnodes.is_occupied(gid.index()) && self.gnodes.get(gid.index()).state() == GState::SemiInternal {
                    candidates.push((gid, key));
                }
            }
        }
        self.pending_p_i4.extend(candidates);
        span.record("candidates", self.pending_p_i4.len());

        // Phase 2: drain all candidates.  Each check is O(log P).
        // split_for_p_i4 may push new candidates via
        // pending_p_i4 — the while-let naturally drains them.
        while let Some((gid, pk)) = self.pending_p_i4.pop() {
            // Guard: a prior fix may have moved this node to a
            // different plateau or changed its state.
            if self.plateau_basis.plateau_key(gid) != Some(pk) {
                continue;
            }
            if !self.gnodes.is_occupied(gid.index()) {
                continue;
            }
            let g = self.gnodes.get(gid.index());
            if g.state() != GState::SemiInternal {
                continue;
            }

            let Some(child_id) = g.left.or(g.right) else {
                continue;
            };
            let child_lo = self.gnodes.get(child_id.index()).lo;
            let child_pk = self.plateaus.range(..=BasisEdge(child_lo)).next_back().map(|(&k, _)| k);

            if child_pk == Some(pk) {
                self.split_for_p_i4(pk, child_id);
            }
        }
    }

    /// No-op stub when plateau tracking is compiled out.
    #[cfg(not(feature = "dynamic-contour-tracking"))]
    #[inline(always)]
    #[allow(clippy::unused_self, clippy::needless_pass_by_ref_mut)]
    pub(crate) const fn repair_p_i4(&mut self) {}

    /// Create a tile boundary to resolve a P-I4 violation.
    ///
    /// Finds a descendant of `child_id` whose `basis_edge` falls
    /// strictly after `parent_pk`.  If that descendant is already a
    /// basis element of a merged plateau, extracts it and
    /// re-inserts it as a standalone so the `BTreeMap` key sequence
    /// includes a key between the parent and the child's `lo`.
    #[cfg(feature = "dynamic-contour-tracking")]
    fn split_for_p_i4(&mut self, parent_pk: crate::plateau::BasisEdge<C>, child_id: GNodeId) {
        use crate::gnode::GState;
        use crate::plateau::{Plateau, basis_edge_of};

        // Find the node to use as a tile boundary. Walk leftmost
        // down the child subtree until we find a node whose
        // basis_edge > parent_pk.
        let boundary_id = self.find_boundary_node(child_id, parent_pk);
        let Some(boundary_id) = boundary_id else { return };

        let bg = self.gnodes.get(boundary_id.index());
        let boundary_key = basis_edge_of(bg);
        let boundary_depth = match bg.state() {
            GState::Terminal | GState::SemiInternal => crate::gtree::gnode_depth_from_interval(bg.lo, bg.hi, N),
            GState::Internal => crate::gtree::gnode_depth_from_interval(bg.lo, bg.hi, N) + 1,
        };
        let b_lo = bg.lo;
        let b_hi = bg.hi;
        let b_sum = bg.sum;

        // If the boundary node is already a basis element, extract it
        // from its current plateau.
        if let Some(old_pk) = self.plateau_basis.remove(boundary_id) {
            self.fixup_plateau(old_pk);
        }

        // Insert as a standalone plateau at boundary_key.
        // This creates a BTreeMap entry that splits the parent's tile.
        self.plateau_basis.insert(boundary_key, boundary_id);
        if let std::collections::btree_map::Entry::Vacant(e) = self.plateaus.entry(boundary_key) {
            e.insert(Plateau {
                basis_edge: boundary_key,
                start: b_lo,
                end: b_hi,
                depth: boundary_depth,
                sum: b_sum,
            });
        }
        self.recompute_plateau(&boundary_key);
    }

    /// Walk the subtree rooted at `gid` to find a node whose
    /// `basis_edge > parent_pk`.  Prefers the leftmost such node
    /// (to split as close to the parent as possible).
    #[cfg(feature = "dynamic-contour-tracking")]
    fn find_boundary_node(&self, gid: GNodeId, parent_pk: crate::plateau::BasisEdge<C>) -> Option<GNodeId> {
        use crate::plateau::basis_edge_of;

        let g = self.gnodes.get(gid.index());
        let key = basis_edge_of(g);
        if key > parent_pk {
            return Some(gid);
        }
        // The node's basis_edge <= parent_pk. Try children.
        if let Some(left) = g.left
            && let Some(found) = self.find_boundary_node(left, parent_pk)
        {
            return Some(found);
        }
        if let Some(right) = g.right
            && let Some(found) = self.find_boundary_node(right, parent_pk)
        {
            return Some(found);
        }
        None
    }

    // ── Plateau mutation hooks ──────────────────────────────────
    //
    // Each hook encapsulates the plateau bookkeeping for one
    // mutation path. When the `plateau` feature is disabled, the
    // compiler emits a no-op stub and dead-code-eliminates the
    // call site.

    /// Post-observe: delta-propagate plateau sums along the
    /// receiver → root path.
    #[cfg(feature = "dynamic-contour-tracking")]
    pub(crate) fn plateau_after_observe<O: crate::traits::Observation<V>>(&mut self, g_id: GNodeId, delta: O) {
        let value_v: V = O::accumulate(V::zero(), delta);
        let mut cur = Some(g_id);
        while let Some(id) = cur {
            if let Some(key) = self.plateau_basis.plateau_key(id)
                && let Some(p) = self.plateaus.get_mut(&key)
            {
                p.sum = V::add(p.sum, value_v);
            }
            cur = self.gnodes.get(id.index()).parent;
        }

        #[cfg(debug_assertions)]
        {
            let mut cur = Some(g_id);
            while let Some(id) = cur {
                if let Some(key) = self.plateau_basis.plateau_key(id) {
                    let elems: Vec<_> = self
                        .plateau_basis
                        .basis_elements(&key)
                        .iter()
                        .map(|&r| {
                            let g = self.gnodes.get(r.index());
                            (r.index(), g.sum.to_f64_approx(), format!("{:?}", g.state()))
                        })
                        .collect();
                    let expected: V = self
                        .plateau_basis
                        .basis_elements(&key)
                        .iter()
                        .map(|&r| self.gnodes.get(r.index()).sum)
                        .fold(V::zero(), V::add);
                    assert_eq!(
                        self.plateaus[&key].sum,
                        expected,
                        "POST-OBSERVE: plateau sum drift at key {key:?}\n\
                         delta={}, g_id=G({}), cur_node=G({})\n\
                         basis_elements={elems:?}",
                        value_v.to_f64_approx(),
                        g_id.index(),
                        id.index(),
                    );
                }
                cur = self.gnodes.get(id.index()).parent;
            }
        }
    }

    /// No-op stub when plateau tracking is compiled out.
    #[cfg(not(feature = "dynamic-contour-tracking"))]
    #[inline(always)]
    #[allow(clippy::unused_self, clippy::needless_pass_by_ref_mut)]
    pub(crate) const fn plateau_after_observe<O: crate::traits::Observation<V>>(&mut self, _g_id: GNodeId, _delta: O) {}

    /// Post-bootstrap-split: parent was terminal → now balanced
    /// internal. Remove old basis element and re-place at child depth.
    #[cfg(feature = "dynamic-contour-tracking")]
    pub(crate) fn plateau_after_bootstrap_split(&mut self, g_id: GNodeId, left_id: GNodeId) {
        let _span = tracing::debug_span!("plateau_after_bootstrap_split", g_id = g_id.index(), left = left_id.index(),).entered();

        // ADR-M-031: mark dirty for trailing normalize in observe().
        self.plateaus_dirty = true;

        let old_key = self
            .plateau_basis
            .remove(g_id)
            .expect("bootstrap_split: g_id must be a basis element");
        self.fixup_plateau(old_key);

        let right_id = self
            .gnodes
            .get(g_id.index())
            .right
            .expect("bootstrap_split: g_id must have a right child");

        let left_depth =
            crate::gtree::gnode_depth_from_interval(self.gnodes.get(left_id.index()).lo, self.gnodes.get(left_id.index()).hi, N);
        let right_depth = crate::gtree::gnode_depth_from_interval(
            self.gnodes.get(right_id.index()).lo,
            self.gnodes.get(right_id.index()).hi,
            N,
        );

        if left_depth == right_depth {
            // Power-of-2 domain: parent subtree is uniform, place
            // parent as a single basis element.
            self.place_basis_element(g_id, left_depth);
        } else {
            // Non-power-of-2 domain (e.g. u8/N=8, domain_max=255):
            // children have different depths, so the parent subtree
            // is not uniform. Place each child individually.
            tracing::debug!(
                g = g_id.index(),
                left = left_id.index(),
                right = right_id.index(),
                left_depth,
                right_depth,
                "bootstrap_split: unequal child depths, placing children separately"
            );
            self.place_basis_element(left_id, left_depth);
            self.place_basis_element(right_id, right_depth);
        }
    }

    /// No-op stub when plateau tracking is compiled out.
    #[cfg(not(feature = "dynamic-contour-tracking"))]
    #[inline(always)]
    #[allow(clippy::unused_self, clippy::needless_pass_by_ref_mut)]
    pub(crate) const fn plateau_after_bootstrap_split(&mut self, _g_id: GNodeId, _left_id: GNodeId) {}

    /// Post-catalytic-split: `g_id` was terminal → now balanced internal.
    /// Decompose covering ancestor (if any), place displaced siblings,
    /// then place `g_id` at the new child depth.
    #[cfg(feature = "dynamic-contour-tracking")]
    pub(crate) fn plateau_after_catalytic_split(&mut self, g_id: GNodeId, left_id: GNodeId) {
        let _span = tracing::debug_span!("plateau_after_catalytic_split", g_id = g_id.index(), left = left_id.index(),).entered();

        // ADR-M-031: mark dirty for trailing normalize in observe().
        self.plateaus_dirty = true;

        let right_id = self
            .gnodes
            .get(g_id.index())
            .right
            .expect("catalytic_split: g_id must have a right child");

        let left_depth =
            crate::gtree::gnode_depth_from_interval(self.gnodes.get(left_id.index()).lo, self.gnodes.get(left_id.index()).hi, N);
        let right_depth = crate::gtree::gnode_depth_from_interval(
            self.gnodes.get(right_id.index()).lo,
            self.gnodes.get(right_id.index()).hi,
            N,
        );

        // Step 1 — Find and remove the covering basis element.
        // For displaced siblings, we just collect IDs (not pre-computed depths)
        // because Internal nodes may need decomposition.
        //
        // After removing the covering element we also evacuate all
        // **remaining** co-basis elements of the same plateau.  Those
        // elements were contiguous while the removed node bridged
        // regions, but they may become non-contiguous after the
        // removal, producing a bogus bounding-box `end` that tricks
        // the left-neighbour merge in `place_basis_element`.
        // Re-placing every co-member via `place_sorted` guarantees
        // the rebuilt plateaus are correctly partitioned.
        let (old_key, displaced) = if let Some(key) = self.plateau_basis.remove(g_id) {
            // g_id was itself a basis element.  Evacuate all remaining
            // basis elements of the same plateau so they are re-placed.
            let co_members: Vec<GNodeId> = self.plateau_basis.basis_elements(&key).iter().copied().collect();
            for &m in &co_members {
                self.plateau_basis.remove(m);
            }
            (key, co_members)
        } else {
            let mut path: Vec<GNodeId> = vec![g_id];
            let mut parent_opt = self.gnodes.get(g_id.index()).parent;
            let mut result = None;

            while let Some(p_id) = parent_opt {
                if let Some(key) = self.plateau_basis.remove(p_id) {
                    let mut displaced: Vec<GNodeId> = Vec::new();
                    for &path_node in &path {
                        let par = self
                            .gnodes
                            .get(path_node.index())
                            .parent
                            .expect("path node must have a parent");
                        let pg = self.gnodes.get(par.index());
                        let sibling = if pg.left == Some(path_node) { pg.right } else { pg.left };
                        if let Some(sib_id) = sibling {
                            displaced.push(sib_id);
                        }
                    }
                    // Also evacuate remaining co-basis elements of the
                    // same plateau (they may become non-contiguous after
                    // the ancestor's removal).
                    let co_members: Vec<GNodeId> = self.plateau_basis.basis_elements(&key).iter().copied().collect();
                    for &m in &co_members {
                        self.plateau_basis.remove(m);
                    }
                    displaced.extend(co_members);
                    result = Some((key, displaced));
                    break;
                }
                path.push(p_id);
                parent_opt = self.gnodes.get(p_id.index()).parent;
            }

            result.expect("catalytic_split: no basis element covers g_id")
        };

        // Step 2 — Clean up the old plateau.
        self.fixup_plateau(old_key);

        // Step 3+4 combined — collect all displaced + target into one
        // vec, sort by BasisEdge, place left-to-right (ADR-M-031 §2a).
        let mut to_place = Vec::new();
        for &sib_id in &displaced {
            self.collect_subtree_basis_elements(sib_id, &mut to_place);
        }
        if left_depth == right_depth {
            to_place.push((g_id, left_depth));
        } else {
            to_place.push((left_id, left_depth));
            to_place.push((right_id, right_depth));
        }
        self.place_sorted(&mut to_place);
    }

    /// No-op stub when plateau tracking is compiled out.
    #[cfg(not(feature = "dynamic-contour-tracking"))]
    #[inline(always)]
    #[allow(clippy::unused_self, clippy::needless_pass_by_ref_mut)]
    pub(crate) const fn plateau_after_catalytic_split(&mut self, _g_id: GNodeId, _left_id: GNodeId) {}

    /// Post-legacy-promote: parent transitioned semi-internal →
    /// internal. Remove parent from basis, place existing child and
    /// new child at their contour depths (sorted, ADR-M-031 §2b).
    /// Note: after ADR-M-031, `handle_legacy_promotes` uses
    /// `plateau_after_legacy_promotes_batched` instead. Retained for
    /// potential future single-promote callers.
    #[cfg(feature = "dynamic-contour-tracking")]
    #[allow(dead_code)]
    pub(crate) fn plateau_after_legacy_promote(&mut self, new_gid: GNodeId) {
        use crate::gnode::GState;

        let ng = self.gnodes.get(new_gid.index());
        let parent_id = ng.parent.expect("legacy_promote child must have a parent");
        let child_depth = crate::gtree::gnode_depth_from_interval(ng.lo, ng.hi, N);

        let pg = self.gnodes.get(parent_id.index());
        let existing_child_id = if pg.left == Some(new_gid) { pg.right } else { pg.left };

        if let Some(old_key) = self.plateau_basis.remove(parent_id) {
            self.fixup_plateau(old_key);
        }

        // Collect both elements, sort, place left-to-right (ADR-M-031 §2b).
        let mut to_place = Vec::new();
        if let Some(ec_id) = existing_child_id {
            let ec = self.gnodes.get(ec_id.index());
            if ec.state() != GState::Internal && self.plateau_basis.plateau_key(ec_id).is_none() {
                let existing_depth = crate::gtree::gnode_depth_from_interval(ec.lo, ec.hi, N);
                to_place.push((ec_id, existing_depth));
            }
        }
        to_place.push((new_gid, child_depth));
        self.place_sorted(&mut to_place);
    }

    /// No-op stub when plateau tracking is compiled out.
    #[cfg(not(feature = "dynamic-contour-tracking"))]
    #[inline(always)]
    #[allow(dead_code, clippy::unused_self, clippy::needless_pass_by_ref_mut)]
    pub(crate) const fn plateau_after_legacy_promote(&mut self, _new_gid: GNodeId) {}

    /// Batched legacy-promote plateau maintenance (ADR-M-031).
    ///
    /// Processes all promotes in two phases:
    /// 1. **Remove** each parent from the plateau basis and collect
    ///    displaced elements (existing child + new child) into one vec.
    /// 2. **Sort** all collected elements by `BasisEdge` and place
    ///    left-to-right in a single pass.
    ///
    /// This prevents cross-promote ordering issues when multiple
    /// legacy promotes occur within one `observe()` or `decay()` call.
    #[cfg(feature = "dynamic-contour-tracking")]
    pub(crate) fn plateau_after_legacy_promotes_batched(&mut self, new_gnodes: &[GNodeId]) {
        use crate::gnode::GState;

        if new_gnodes.is_empty() {
            return;
        }

        let _span = tracing::debug_span!("plateau_after_legacy_promotes_batched", count = new_gnodes.len(),).entered();

        // ADR-M-031: mark dirty for trailing normalize in observe().
        self.plateaus_dirty = true;

        let mut to_place = Vec::new();

        // Phase 1: remove parents from basis, collect displaced elements.
        for &new_gid in new_gnodes {
            let ng = self.gnodes.get(new_gid.index());
            let parent_id = ng.parent.expect("legacy_promote child must have a parent");
            let child_depth = crate::gtree::gnode_depth_from_interval(ng.lo, ng.hi, N);

            let pg = self.gnodes.get(parent_id.index());
            let existing_child_id = if pg.left == Some(new_gid) { pg.right } else { pg.left };

            if let Some(old_key) = self.plateau_basis.remove(parent_id) {
                self.fixup_plateau(old_key);
            }

            if let Some(ec_id) = existing_child_id {
                let ec = self.gnodes.get(ec_id.index());
                if ec.state() != GState::Internal && self.plateau_basis.plateau_key(ec_id).is_none() {
                    let existing_depth = crate::gtree::gnode_depth_from_interval(ec.lo, ec.hi, N);
                    to_place.push((ec_id, existing_depth));
                }
            }
            to_place.push((new_gid, child_depth));
        }

        // Phase 2: sort all collected elements and place left-to-right.
        self.place_sorted(&mut to_place);
    }

    /// No-op stub when plateau tracking is compiled out.
    #[cfg(not(feature = "dynamic-contour-tracking"))]
    #[inline(always)]
    #[allow(clippy::unused_self, clippy::needless_pass_by_ref_mut)]
    pub(crate) const fn plateau_after_legacy_promotes_batched(&mut self, _new_gnodes: &[GNodeId]) {}

    /// Post-decay: recompute plateau sums from authoritative g.sum
    /// values. Decay changes magnitudes only — no structural changes.
    #[cfg(feature = "dynamic-contour-tracking")]
    #[cfg_attr(not(debug_assertions), allow(unused_variables))]
    pub(crate) fn plateau_recompute_sums(&mut self, label: &str) {
        for (&key, plateau) in &mut self.plateaus {
            plateau.sum = self
                .plateau_basis
                .basis_elements(&key)
                .iter()
                .map(|&r| self.gnodes.get(r.index()).sum)
                .fold(V::zero(), V::add);
        }

        #[cfg(debug_assertions)]
        for (&key, plateau) in &self.plateaus {
            let expected: V = self
                .plateau_basis
                .basis_elements(&key)
                .iter()
                .map(|&r| self.gnodes.get(r.index()).sum)
                .fold(V::zero(), V::add);
            assert_eq!(plateau.sum, expected, "POST-{label}: plateau sum drift at key {key:?}");
        }
    }

    /// No-op stub when plateau tracking is compiled out.
    #[cfg(not(feature = "dynamic-contour-tracking"))]
    #[inline(always)]
    #[allow(clippy::unused_self, clippy::needless_pass_by_ref_mut)]
    pub(crate) const fn plateau_recompute_sums(&mut self, _label: &str) {}

    // ── Plateau Selection (§CR.12 — ADR-M-037) ──────────────────

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
}
