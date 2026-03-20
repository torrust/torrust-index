// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Budget enforcement, depth-gate adjustment, and eviction
//! orchestration for `GvGraph`.
//!
//! Extracted from `graph.rs` per ADR-M-030 Phase E.
//! See ADR-M-015 (eviction scan), ADR-M-017 (dynamic depth control),
//! ADR-M-018 (hard budget guarantee).

use crate::graph::GvGraph;
use crate::handle::GNodeId;
use crate::traits::{Accumulator, Coordinate, Inspectable};
use crate::{evict, rebalance, vtree};

impl<C: Coordinate, V: Accumulator + Inspectable, const N: u32> GvGraph<C, V, N> {
    /// Handle new G-nodes created by `legacy_promote` during rebalance.
    ///
    /// For each new G-node:
    /// 1. Increment `node_count`.
    /// 2. The parent G-node transitioned semi-internal → internal.
    ///    Remove it from the plateau basis (if present) and place the
    ///    new terminal child at the appropriate depth.
    ///
    /// ADR-M-031: all removals are performed first, then all collected
    /// elements are sorted by `BasisEdge` and placed left-to-right
    /// in a single batch.  This prevents cross-promote ordering
    /// issues when multiple legacy promotes occur in one `observe()`.
    pub(crate) fn handle_legacy_promotes(&mut self, new_gnodes: &[GNodeId]) {
        for &_new_gid in new_gnodes {
            self.node_count += 1;
            // Legacy promote creates a new terminal child and transitions
            // its parent from semi-internal → internal.  The parent was
            // not a terminal, so net terminal change = +1.
            self.terminal_count += 1;
        }
        self.plateau_after_legacy_promotes_batched(new_gnodes);
    }

    /// Adjust `D_evict` and `D_create` based on memory pressure (ADR-M-017).
    ///
    /// Called inside `observe()` after `rebalance()` and before
    /// `check_evictions()`. Only fires when `budget` is `Some`.
    ///
    /// - Over budget → tighten by 1 level (floor at `buffer + 1`).
    /// - Under `α_relax × budget` → relax by 1 level (no ceiling).
    /// - Inside the dead zone → no change.
    ///
    /// D-I3 is maintained at all times via the fixed buffer.
    pub(crate) fn adjust_depth_gates(&mut self) {
        let Some(budget) = self.config.budget else {
            return;
        };
        let count = self.node_count as usize;

        // Dynamic headroom: max(structural ceiling, convergence bound).
        // ADR-M-018: ensures S + 2(D_c − 1) ≤ H at every step.
        let convergence_bound = 2 * (self.live_depth_create as usize).saturating_sub(1);
        let required_headroom = self.headroom.max(convergence_bound);
        let soft_limit = budget - required_headroom;
        assert!(
            soft_limit >= 1,
            "soft_limit must be >= 1 (budget={budget}, headroom={required_headroom}, \
             D_c={}, buffer={})",
            self.live_depth_create,
            self.depth_buffer
        );
        self.soft_limit = Some(soft_limit);

        if count > soft_limit {
            // Tighten: lower D_evict by 1, floor at buffer + 1.
            let floor = self.depth_buffer + 1;
            if self.live_depth_evict > floor {
                self.live_depth_evict -= 1;
                self.live_depth_create = self.live_depth_evict - self.depth_buffer;
                tracing::debug!(
                    new_d_evict = self.live_depth_evict,
                    new_d_create = self.live_depth_create,
                    count,
                    soft_limit,
                    "depth gates tightened",
                );
            }
        } else {
            #[allow(clippy::cast_precision_loss)] // node counts fit in f64 mantissa
            let threshold = soft_limit as f64 * self.config.alpha_relax;
            #[allow(clippy::cast_precision_loss)]
            let count_f = count as f64;
            if count_f < threshold {
                // Relax: raise D_evict by 1 (no ceiling).
                self.live_depth_evict += 1;
                self.live_depth_create = self.live_depth_evict - self.depth_buffer;
                tracing::debug!(
                    new_d_evict = self.live_depth_evict,
                    new_d_create = self.live_depth_create,
                    count,
                    soft_limit,
                    "depth gates relaxed",
                );
            }
        }
    }

    // ── Eviction wiring (Step 7, ADR-M-015) ───────────────────────

    /// Evict all V-depth-eligible candidates without an eviction-count cap.
    ///
    /// Unlike the budget-triggered eviction at the end of [`observe()`](Self::observe)
    /// — which, when `bounded_eviction` is `true`,
    /// stops once the node count drops to the soft limit — this method
    /// processes every candidate whose V-depth exceeds the current `D_evict`
    /// gate. Returns the number of entries evicted.
    ///
    /// Eligible candidates are V-Tree entries whose depth exceeds
    /// the live `D_evict` gate.  In a fresh or lightly-populated
    /// graph no entries qualify, so the return value is 0.
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
    /// // No observations yet — nothing to evict.
    /// assert_eq!(g.check_evictions(), 0);
    ///
    /// // After some observations, eviction may still return 0 if
    /// // no entry exceeds the depth gate.
    /// g.observe(42, 10u64);
    /// let evicted = g.check_evictions();
    /// assert_eq!(evicted, 0);
    /// ```
    pub fn check_evictions(&mut self) -> u32 {
        let _span = tracing::debug_span!("check_evictions", d_evict = self.live_depth_evict).entered();
        self.evict_candidates(None)
    }

    /// Evict candidates up to a budget-derived limit (bounded mode).
    ///
    /// Like `check_evictions()` but stops after `stop_at` evictions.
    pub(crate) fn check_evictions_bounded(&mut self, stop_at: usize) -> u32 {
        self.evict_candidates(Some(stop_at))
    }

    /// Shared implementation for bounded and unbounded eviction.
    ///
    /// Two-phase collect-then-evict (ADR-M-015):
    /// 1. `scan_for_candidates()` — snapshot of eligible `VNodeId`s.
    /// 2. Iterate, re-verify eligibility (slot occupied, still
    ///    evictable, depth still exceeds `D_evict`, not G-root),
    ///    then call `evict_tip()`.
    /// 3. Trailing `rebalance()` if any evictions occurred.
    #[allow(clippy::too_many_lines)]
    fn evict_candidates(&mut self, stop_at: Option<usize>) -> u32 {
        let candidates = evict::scan_for_candidates(self);
        let _span = tracing::debug_span!("evict_batch", candidate_count = candidates.len(),).entered();
        let mut evicted: u32 = 0;

        for v_id in candidates {
            // Budget-bounded early exit.
            if let Some(limit) = stop_at
                && evicted as usize >= limit
            {
                break;
            }

            // Re-verify eligibility: the slot may have been
            // invalidated by a prior eviction in this batch.
            if !self.vnodes.is_occupied(v_id.index()) {
                continue;
            }
            match &self.vnodes.get(v_id.index()).kind {
                crate::vnode::VKind::Entry { gnode, is_evictable, .. } => {
                    if !is_evictable {
                        continue;
                    }
                    if *gnode == self.g_root {
                        continue;
                    }
                    // Depth may have changed due to V-tree mutations.
                    let depth = vtree::v_depth(&self.vnodes, v_id);
                    if depth <= self.live_depth_evict {
                        continue;
                    }
                }
                crate::vnode::VKind::Structural { .. } => continue,
            }

            evict::evict_tip(self, v_id);
            evicted += 1;

            // Post-eviction audit: find violations not in the queue.
            if tracing::enabled!(tracing::Level::DEBUG) {
                crate::diagnostic::audit_violations(&self.vnodes, &self.violations, "POST-EVICT");
            }

            // Per-eviction mirror check — pinpoints which eviction
            // in a batch caused a divergence (the batch-level
            // POST-EVICT check at the end only reports *that* the
            // batch diverged, not *which* eviction).
            #[cfg(feature = "dynamic-contour-tracking")]
            if cfg!(debug_assertions) {
                self.debug_assert_plateau_mirror_consistency(&format!("POST-EVICT-SINGLE-{evicted}"));
            }
        }

        // Trailing rebalance to resolve any absorption violations.
        if evicted > 0 {
            let new_gnodes = rebalance::rebalance(
                &mut self.vnodes,
                &mut self.gnodes,
                &mut self.violations,
                self.live_depth_evict,
            );
            if !new_gnodes.is_empty() {
                self.handle_legacy_promotes(&new_gnodes);
            }

            // Repair P-I4 thatch-hop violations introduced by
            // eviction plateau merges or legacy promote placements.
            self.repair_p_i4();

            // ADR-M-031 Phase 4: mark dirty for batch eviction
            // interaction and normalize if needed.
            #[cfg(feature = "dynamic-contour-tracking")]
            {
                self.plateaus_dirty = true;
            }
            self.normalize_plateaus();

            // Re-run P-I4 repair after normalization.
            self.repair_p_i4();
        }

        #[cfg(feature = "dynamic-contour-tracking")]
        if cfg!(debug_assertions) || tracing::enabled!(tracing::Level::DEBUG) {
            self.debug_assert_plateau_mirror_consistency("POST-EVICT");
        }

        evicted
    }
}
