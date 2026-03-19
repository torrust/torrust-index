// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! The `observe()` hot-path: accumulate an observation, propagate
//! sums, split if warranted, and rebalance.
//!
//! Implements §IDEA M-8 (observation flow). This is the primary
//! public mutation entry-point for the `GvGraph`.

use crate::graph::GvGraph;
use crate::traits::{Accumulator, Coordinate, Inspectable, Observation};
use crate::{gtree, rebalance, split, vtree};

impl<C: Coordinate, V: Accumulator + Inspectable, const N: u32> GvGraph<C, V, N> {
    /// Record an observation of `delta` at coordinate `coord`.
    ///
    /// Pipeline (§IDEA M-8, extended by implementation — see
    /// ADR-M-031 for the full mapping between spec and impl steps):
    ///
    ///  1.    Route to receiver.
    ///  2. a–b. Accumulate into G-node own value, V-entry intensity;
    ///     propagate V-sums; enqueue violations.
    ///  3.    Recompute G-sums upward (ADR-M-012).
    ///  4.    Plateau sum maintenance (`plateau_after_observe`).
    ///  5.    Attempt contour refinement (split).
    ///  6.    Drain violation queue / rebalance (V-I3).
    ///  7.    Dynamic depth control (§IDEA M-7.1, ADR-M-017).
    ///  8.    Budget-guarded eviction (ADR-M-015, ADR-M-018).
    ///  9.    Normalize plateau map — dirty-gated (ADR-M-031).
    /// 10.    P-I4 thatch-hop repair.
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
    /// g.observe(42, 10u64);
    /// assert_eq!(g.total_sum(), 10);
    ///
    /// g.observe(42, 5u64);
    /// assert_eq!(g.total_sum(), 15);
    /// ```
    pub fn observe<O: Observation<V>>(&mut self, coord: C, delta: O) {
        // 1. Route to receiver.
        let g_id = gtree::route_to_receiver(&self.gnodes, self.g_root, coord);
        let _span = tracing::debug_span!("observe", g = g_id.index()).entered();

        // 2a. Accumulate into G-node own value.
        let g = self.gnodes.get_mut(g_id.index());
        g.own = O::accumulate(g.own, delta);

        // 2b. Accumulate into V-entry intensity and propagate V-sums.
        if let Some(entry_id) = self.gnodes.get(g_id.index()).entry {
            let v = self.vnodes.get_mut(entry_id.index());
            v.intensity = O::accumulate(v.intensity, delta);
            let new_intensity = v.intensity;

            vtree::update_parent_cached_intensity(&mut self.vnodes, entry_id, new_intensity);
            vtree::propagate_v_sums(&mut self.vnodes, entry_id);

            // 2b (cont). Violation check — enqueue entry and all
            //    structural ancestors whose intensity increased via
            //    propagation.  The spec's step 6 (rebalance) finds
            //    violated nodes; our queue must push eagerly.
            //    Cost: O(depth_v), matching propagate_v_sums.
            let mut check_id = Some(entry_id);
            while let Some(id) = check_id {
                if rebalance::is_violated(&self.vnodes, id) {
                    tracing::debug!(
                        violated = %rebalance::Nd(&self.vnodes, id),
                        entry = entry_id.index(),
                        "enqueuing violated ancestor",
                    );
                    self.violations.push(id);
                }
                check_id = self.vnodes.get(id.index()).parent;
            }
        }

        // 3. Recompute G-sums from receiver to root (ADR-M-012).
        gtree::recompute_g_sums(&mut self.gnodes, g_id);

        // 4. Plateau sum maintenance (no-op without `plateau` feature).
        self.plateau_after_observe::<O>(g_id, delta);

        // 5. Attempt split.
        split::attempt_split(self, g_id);

        // Post-split audit: check for untracked violations.
        if tracing::enabled!(tracing::Level::DEBUG) {
            crate::diagnostic::audit_violations(&self.vnodes, &self.violations, "POST-SPLIT");
        }

        #[cfg(feature = "dynamic-contour-tracking")]
        if cfg!(debug_assertions) || tracing::enabled!(tracing::Level::DEBUG) {
            self.debug_assert_plateau_mirror_consistency("POST-SPLIT");
        }

        // 6. Drain violation queue.
        let new_gnodes = rebalance::rebalance(
            &mut self.vnodes,
            &mut self.gnodes,
            &mut self.violations,
            self.live_depth_evict,
        );
        if !new_gnodes.is_empty() {
            self.handle_legacy_promotes(&new_gnodes);
            self.repair_p_i4();
        }

        #[cfg(feature = "dynamic-contour-tracking")]
        if cfg!(debug_assertions) || tracing::enabled!(tracing::Level::DEBUG) {
            self.debug_assert_plateau_mirror_consistency("POST-REBALANCE");
        }

        // 7. Dynamic depth control (§IDEA M-7.1, ADR-M-017).
        self.adjust_depth_gates();

        // 8. Budget-guarded eviction (ADR-M-015, ADR-M-018).
        if let Some(soft_limit) = self.soft_limit
            && self.node_count as usize > soft_limit
        {
            if self.config.bounded_eviction {
                self.check_evictions_bounded(self.node_count as usize - soft_limit);
            } else {
                self.check_evictions();
            }
        }

        // 9. [ADR-M-031] Normalize plateau map — gated behind dirty
        //    flag so it's O(1) no-op for simple observations that
        //    don't trigger structural changes (splits / promotes).
        //    Sorted placement (Phases 2a/2b) reduces frequency;
        //    remaining cross-step interactions within observe()
        //    still need normalize when structural changes occur.
        self.normalize_plateaus();

        #[cfg(feature = "dynamic-contour-tracking")]
        if cfg!(debug_assertions) || tracing::enabled!(tracing::Level::DEBUG) {
            self.debug_check_plateau_sums("POST-NORMALIZE");
        }

        // 10. Re-run P-I4 repair — normalization may re-key basis
        //     elements, re-introducing thatch-hop violations.
        self.repair_p_i4();

        // Post-observe audit: catch violations missed by any pipeline
        // stage.
        //
        // The O(n) scan always runs in debug builds (ensures the assert
        // fires during `cargo test`) and optionally in release when a
        // tracing subscriber is active at DEBUG.
        if cfg!(debug_assertions) || tracing::enabled!(tracing::Level::DEBUG) {
            let remaining = crate::diagnostic::audit_violations(&self.vnodes, &self.violations, "POST-OBSERVE");
            debug_assert!(remaining.is_empty(), "POST-OBSERVE: residual violations: {remaining:?}");
        }

        #[cfg(feature = "dynamic-contour-tracking")]
        if cfg!(debug_assertions) || tracing::enabled!(tracing::Level::DEBUG) {
            self.debug_assert_plateau_mirror_consistency("POST-OBSERVE");
        }
    }
}

#[cfg(test)]
mod tests {}
