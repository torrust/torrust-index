// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! `decay()` — subband-adaptive temporal filter on the G-V Graph.
//!
//! Implements §IDEA M-14, §API M-6.8, ADR-M-024.
//!
//! The G-V Tree is a spatiotemporal filter bank with three stages:
//! analysis (G-Tree splitting), processing (decay), and synthesis
//! (PEWEI reconstruction). `decay()` is the processing stage.
//!
//! Three parameters control the temporal filter shape:
//! - **root** — which G-subtree to decay.
//! - **attenuation** — base decay factor at the midpoint depth.
//! - **q** — selectivity in `[0, 1]`. 0 = uniform, >0 = selective.
//!
//! The per-depth factor within the target subtree:
//!
//! $$\ln\lambda(d) = \ln(\text{att}) \cdot (1 + q \cdot (2d_\text{local}/D - 1))$$

use crate::graph::GvGraph;
use crate::handle::GNodeId;
use crate::traits::{Accumulator, Attenuatable, Coordinate, Inspectable};
use crate::{gtree, rebalance, vtree};

impl<C: Coordinate, V: Accumulator + Attenuatable + Inspectable, const N: u32> GvGraph<C, V, N> {
    /// Apply temporal decay to a G-subtree.
    ///
    /// - `root`: G-node whose subtree is decayed. Use `g_root()` for
    ///   the entire tree.
    /// - `attenuation`: base decay factor, applied at the midpoint
    ///   depth of the subtree. Values in `(0, 1)` cause exponential
    ///   decay. Values above `1.0` amplify.
    /// - `q`: selectivity in `[0, 1]`.
    ///   - `0.0` = uniform: all subbands decay at the same rate.
    ///   - `> 0.0` = selective: coarse persists, fine fades faster.
    ///   - `1.0` = maximum: subtree root undecayed, deepest terminals
    ///     decay at `attenuation²`.
    ///
    /// # Panics
    ///
    /// - `attenuation <= 0.0` or `attenuation.is_nan()`.
    /// - `q < 0.0`, `q > 1.0`, or `q.is_nan()`.
    ///
    /// # Cost
    ///
    /// - Uniform ($q = 0$): $O(S + V + K \log K)$ with trailing
    ///   rebalance ($K$ typically zero for float types).
    /// - Selective ($q > 0$): $O(S + V + K \log K)$ with per-depth
    ///   factors and trailing rebalance.
    ///
    /// # Invariants
    ///
    /// G-I1 (summation) and V-I1 (structural sum) are restored.
    /// V-I3 (max-uncle) is restored via rebalance when needed.
    /// `decay()` does not trigger eviction (DC-024-3).
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
    /// g.observe(42, 100u64);
    /// let before = g.total_sum();
    ///
    /// // Uniform half-life decay (q = 0).
    /// let root = g.g_root();
    /// g.decay(root, 0.5, 0.0);
    /// assert!(g.total_sum() < before);
    /// ```
    pub fn decay(&mut self, root: GNodeId, attenuation: f64, q: f64) {
        let _span = tracing::debug_span!("decay", root = root.index(), attenuation, q,).entered();

        // ── Validation ──────────────────────────────────────────
        assert!(
            self.gnodes.is_occupied(root.index()),
            "decay: root handle (index {}) does not refer to a live G-node",
            root.index()
        );
        assert!(
            attenuation > 0.0 && !attenuation.is_nan(),
            "decay: attenuation must be > 0 and not NaN, got {attenuation}"
        );
        assert!(
            (0.0..=1.0).contains(&q) && !q.is_nan(),
            "decay: q must be in [0.0, 1.0] and not NaN, got {q}"
        );

        // No-op: factor of exactly 1.0 changes nothing.
        #[allow(clippy::float_cmp)] // Intentional exact comparison for the no-op case.
        if attenuation == 1.0 {
            return;
        }

        let is_global = root == self.g_root;

        let _span = tracing::debug_span!("decay", root = root.index(), attenuation, q,).entered();

        if q == 0.0 {
            self.decay_uniform(root, attenuation, is_global);
        } else {
            self.decay_selective(root, attenuation, q, is_global);
        }
    }

    // ── Uniform path (q = 0) ────────────────────────────────────

    /// Single-pass scale: λ(a + b) = λa + λb, so scaling both `own`
    /// and `sum` by the same factor preserves G-I1 within the subtree.
    fn decay_uniform(&mut self, root: GNodeId, att: f64, is_global: bool) {
        let _span = tracing::debug_span!("decay_uniform", root = root.index(), att, is_global,).entered();

        // 1. DFS walk: scale g.own and collect nodes in pre-order.
        //    Do NOT scale g.sum directly — integer truncation means
        //    floor(sum*λ) ≠ floor(own*λ) + floor(child.sum*λ).
        let mut order = Vec::new();
        let mut stack = vec![root];
        while let Some(gid) = stack.pop() {
            order.push(gid);

            let g = self.gnodes.get_mut(gid.index());
            g.own = g.own.attenuate(att);

            // Sync V-entry intensity.
            if let Some(v_id) = g.entry {
                let own = g.own;
                self.vnodes.get_mut(v_id.index()).intensity = own;
            }

            // Push children.
            let g = self.gnodes.get(gid.index());
            if let Some(left) = g.left {
                stack.push(left);
            }
            if let Some(right) = g.right {
                stack.push(right);
            }
        }

        // 2. Recompute g.sum bottom-up within subtree.
        gtree::recompute_g_sums_subtree(&mut self.gnodes, &order);

        // 3. Fix G-sums above the subtree.
        if !is_global && let Some(parent) = self.gnodes.get(root.index()).parent {
            gtree::recompute_g_sums(&mut self.gnodes, parent);
        }

        // 3. Recompute all V-structural intensities bottom-up.
        if let Some(v_root) = self.v_root {
            vtree::recompute_all_v_intensities(&mut self.vnodes, v_root);
        }

        // 4. V-I3 handling.
        //    Even for global uniform, integer truncation can cause
        //    cumulative drift that breaks V-I3 after repeated decays.
        //    Always detect and rebalance — find_violated_nodes is O(V)
        //    and returns empty when no violations exist.
        self.violations = rebalance::find_violated_nodes(&self.vnodes);
        let new_gnodes = rebalance::rebalance(
            &mut self.vnodes,
            &mut self.gnodes,
            &mut self.violations,
            self.live_depth_evict,
        );
        if !new_gnodes.is_empty() {
            self.handle_legacy_promotes(&new_gnodes);
        }

        // 5. Plateau sum recomputation (no-op without `plateau` feature).
        self.plateau_recompute_sums("DECAY-UNIFORM");

        // 6. Normalize plateaus after sum changes.
        // ADR-M-031 Phase 4: mark dirty so normalize runs.
        #[cfg(feature = "dynamic-contour-tracking")]
        {
            self.plateaus_dirty = true;
        }
        self.normalize_plateaus();

        // 7. Re-run P-I4 repair after normalization.
        self.repair_p_i4();

        #[cfg(feature = "dynamic-contour-tracking")]
        if cfg!(debug_assertions) || tracing::enabled!(tracing::Level::DEBUG) {
            self.debug_assert_plateau_mirror_consistency("POST-DECAY-UNIFORM");
        }
    }

    // ── Selective path (q > 0) ──────────────────────────────────

    /// Per-depth factor table, bottom-up G-sum recompute, trailing
    /// rebalance.
    fn decay_selective(&mut self, root: GNodeId, att: f64, q: f64, is_global: bool) {
        let _span = tracing::debug_span!("decay_selective", root = root.index(), att, q, is_global,).entered();

        let d_root = self.gnode_depth(root);
        let depth_range = N - d_root;

        // 1. Precompute per-depth factor table.
        //    factors[d_local] = exp(ln(att) * (1 + q * (2*d_local/D - 1)))
        let ln_att = att.ln();
        let factors: Vec<f64> = (0..=depth_range)
            .map(|d_local| {
                let t = if depth_range == 0 {
                    0.0
                } else {
                    2.0 * f64::from(d_local) / f64::from(depth_range) - 1.0
                };
                (ln_att * q.mul_add(t, 1.0)).exp()
            })
            .collect();

        // 2. DFS walk: collect nodes in pre-order for bottom-up
        //    reversal, scale g.own per depth.
        let mut order = Vec::new();
        let mut stack = vec![root];
        while let Some(gid) = stack.pop() {
            order.push(gid);

            let g = self.gnodes.get(gid.index());
            let d_local = self.gnode_depth(gid) - d_root;
            let factor = factors[d_local as usize];

            // Scale own.
            let new_own = g.own.attenuate(factor);
            // Drop immutable borrow before mutable access.
            let g = self.gnodes.get_mut(gid.index());
            g.own = new_own;

            // Push children (read from immutable borrow).
            let g = self.gnodes.get(gid.index());
            if let Some(left) = g.left {
                stack.push(left);
            }
            if let Some(right) = g.right {
                stack.push(right);
            }
        }

        // 3. Recompute g.sum bottom-up within subtree.
        gtree::recompute_g_sums_subtree(&mut self.gnodes, &order);

        // 4. Propagate G-sums above the subtree.
        if !is_global && let Some(parent) = self.gnodes.get(root.index()).parent {
            gtree::recompute_g_sums(&mut self.gnodes, parent);
        }

        // 5. Sync V-entry intensities from g.own.
        for &gid in &order {
            let g = self.gnodes.get(gid.index());
            if let Some(v_id) = g.entry {
                let own = g.own;
                self.vnodes.get_mut(v_id.index()).intensity = own;
            }
        }

        // 6. Recompute all V-structural intensities bottom-up.
        if let Some(v_root) = self.v_root {
            vtree::recompute_all_v_intensities(&mut self.vnodes, v_root);
        }

        // 7. Detect V-I3 violations and rebalance.
        //    Selective decay changes relative ordering within the subtree,
        //    so rebalance is always needed (even for global).
        let _ = is_global; // Both paths rebalance for selective.
        self.violations = rebalance::find_violated_nodes(&self.vnodes);
        let new_gnodes = rebalance::rebalance(
            &mut self.vnodes,
            &mut self.gnodes,
            &mut self.violations,
            self.live_depth_evict,
        );
        if !new_gnodes.is_empty() {
            self.handle_legacy_promotes(&new_gnodes);
        }

        // 8. Plateau sum recomputation (no-op without `plateau` feature).
        self.plateau_recompute_sums("DECAY-SELECTIVE");

        // 9. Normalize plateaus after sum changes.
        // ADR-M-031 Phase 4: mark dirty so normalize runs.
        #[cfg(feature = "dynamic-contour-tracking")]
        {
            self.plateaus_dirty = true;
        }
        self.normalize_plateaus();

        // 10. Re-run P-I4 repair after normalization.
        self.repair_p_i4();

        #[cfg(feature = "dynamic-contour-tracking")]
        if cfg!(debug_assertions) || tracing::enabled!(tracing::Level::DEBUG) {
            self.debug_assert_plateau_mirror_consistency("POST-DECAY-SELECTIVE");
        }
    }
}

// ── Tests ───────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {}
