// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Analysis set: V-Tree competitive selection (§ALGO S-8.1–8.3).
//!
//! The competitive targets $\mathcal{T}$ are the top-$K$ V-entries
//! by importance with V-depth ≤ $L$, closed under G-tree ancestry
//! to form the investment set $\mathcal{I}$. The set is recomputed
//! from scratch after every observation pass (ADR-S-006).
//! The selector deliberately does not filter by G-Tree state: terminal,
//! semi-internal, and internal V-entries can all be selected when they
//! satisfy the V-depth, importance, and analysis-width criteria.
//!
//! The producing sets ($\mathcal{A}$, $\mathcal{A}^*$) are the
//! online subsets of $\mathcal{I}$ — derived by the orchestrator
//! from the investment set and tracker online status (ADR-S-019).

use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use torrust_mudlark::{Accumulator, Coordinate, GNodeId, GvGraph, Inspectable};

use crate::report::AnalysisSetSummary;

/// A cell selected for analysis by the selector (§ALGO S-8.1).
///
/// Competitive entries are the top-$K$ V-Tree entries by importance
/// (the competitive targets $\mathcal{T}$). Ancestor entries close
/// the targets under G-tree ancestry to form the investment set
/// $\mathcal{I}$ (§ALGO S-8.2).
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(clippy::derive_partial_eq_without_eq)] // V: Accumulator includes f64 which has no Eq
pub struct AnalysisEntry<C: Coordinate, V: Accumulator> {
    /// Arena handle of the backing G-node.
    pub gnode: GNodeId,
    /// G-Tree depth of this cell.
    pub depth: u32,
    /// V-Tree depth of this cell's V-entry (for competitive cells)
    /// or `0` for ancestor-only cells.
    pub v_depth: usize,
    /// Own importance of this cell (direct accumulation).
    pub importance: V,
    /// Lower bound of the dyadic interval (inclusive).
    pub start: C,
    /// Upper bound of the dyadic interval, exclusive everywhere except at the
    /// top of the domain: the entry whose bound is the domain maximum owns that
    /// maximum, because a coordinate width filling the coordinate type leaves
    /// no value above it to be excluded.
    pub end: C,
    /// Whether this entry is competitively selected (vs ancestor-only).
    pub is_competitive: bool,
}

/// The current analysis set — competitive targets $\mathcal{T}$
/// closed under G-tree ancestry to form the investment set
/// $\mathcal{I}$ (§ALGO S-8.1–8.2).
///
/// Recomputed after every observation pass (ADR-S-006).
#[derive(Debug, Clone)]
pub struct AnalysisSet<C: Coordinate, V: Accumulator> {
    /// Competitively selected cells, ordered by importance (descending),
    /// ties broken by interval start (ascending) for determinism.
    competitive: Vec<AnalysisEntry<C, V>>,

    /// All cells: competitive + ancestors (deduplicated).
    /// The investment set $\mathcal{I}$: the competitive targets closed under
    /// G-tree ancestry, with no filter on whether a cell is online yet. The
    /// producing sets $\mathcal{A}$ and $\mathcal{A}^*$ are its online
    /// subsets, which this type cannot see and the orchestrator derives.
    /// Ordered by `GNodeId` for deterministic iteration (ADR-S-005).
    full: Vec<AnalysisEntry<C, V>>,
}

impl<C: Coordinate, V: Inspectable> AnalysisSet<C, V> {
    /// Recompute the analysis set from the V-Tree.
    ///
    /// Scans `graph.layers_to(depth_cutoff)` (ADR-M-041) to collect
    /// V-entries with `v_depth ≤ depth_cutoff`, takes top `k` by
    /// importance (the competitive targets $\mathcal{T}$,
    /// §ALGO S-8.1; ties broken by `start`), then closes under
    /// G-tree ancestry to form the investment set $\mathcal{I}$
    /// (§ALGO S-8.2).
    ///
    /// The depth-limited BFS avoids expanding V-structural nodes
    /// below the cutoff, saving `O(2^(D−K))` queue work on balanced
    /// trees (ADR-M-041 §Performance).
    ///
    /// `O(n_K)` in V-entries at depth ≤ K + `O(K · max_depth)` for
    /// ancestor closure.
    ///
    /// # Panics
    ///
    /// Panics if the G-root node is not live in the graph.
    #[must_use]
    pub fn recompute<const N: u32>(graph: &GvGraph<C, V, N>, k: usize, depth_cutoff: usize) -> Self {
        // ── Step 1: Collect candidates from V-Tree ──────────────
        // Eligibility: V-depth ≤ cutoff AND analysis width w ≥ 2
        // (§ALGO S-8.1).  The depth limit is enforced by the BFS
        // itself (ADR-M-041), not a post-hoc filter.
        let mut candidates: Vec<AnalysisEntry<C, V>> = graph
            .layers_to(depth_cutoff)
            .filter(|(_, node)| {
                // w = N - depth ≥ MIN_TRACKER_DIM (§ALGO S-8.1, §ALGO S-4.1).
                N.saturating_sub(node.depth) as usize >= crate::MIN_TRACKER_DIM
            })
            .map(|(v_depth, node)| AnalysisEntry {
                gnode: node.gnode_id,
                depth: node.depth,
                v_depth,
                importance: node.own,
                start: node.start,
                end: node.end,
                is_competitive: true,
            })
            .collect();

        // ── Step 2: Sort by importance (desc), then start (asc) ─
        candidates.sort_by(|a, b| {
            b.importance
                .partial_cmp(&a.importance)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.start.partial_cmp(&b.start).unwrap_or(Ordering::Equal))
        });

        // ── Step 3: Take top K ──────────────────────────────────
        //
        // The root is always an ancestor and never competitive (§ALGO S-4.7),
        // so it is dropped before the cut rather than after it. Dropped
        // afterwards it consumes a slot it can never use: the root's own
        // intensity is what it accumulated before its first split, and the
        // split freezes that figure while both children start from zero, so
        // the root outranks every real candidate until one of them passes a
        // total the root is no longer adding to. At a capacity of one — a
        // configuration the validation accepts — that leaves the competitive
        // set permanently empty, and no descendant can ever become
        // competitive. Removing it first spends every slot on an entry that
        // can actually be selected.
        let g_root = graph.g_root();
        candidates.retain(|e| e.gnode != g_root);
        candidates.truncate(k);

        let competitive: Vec<AnalysisEntry<C, V>> = candidates;

        // ── Step 4: Ancestor closure (§ALGO S-4.2) ────────────────
        // Walk G-tree parents for each competitive entry. Collect
        // all ancestor GNodeIds not already in the competitive set.
        let mut full_set: BTreeMap<GNodeId, AnalysisEntry<C, V>> = BTreeMap::new();

        // Insert competitive entries.
        for entry in &competitive {
            full_set.insert(entry.gnode, *entry);
        }

        // Walk ancestors.
        for entry in &competitive {
            let mut current = entry.gnode;
            while let Some(info) = graph.gnode_info(current) {
                let Some(parent_id) = info.parent else {
                    break; // reached the root
                };
                if full_set.contains_key(&parent_id) {
                    break; // already tracked (shared ancestor)
                }
                let Some(parent_info) = graph.gnode_info(parent_id) else {
                    break;
                };
                full_set.insert(
                    parent_id,
                    AnalysisEntry {
                        gnode: parent_id,
                        depth: parent_info.depth,
                        v_depth: 0, // not meaningful for ancestors
                        importance: parent_info.own,
                        start: parent_info.start,
                        end: parent_info.end,
                        is_competitive: false,
                    },
                );
                current = parent_id;
            }
        }

        // Ensure the root is always present (§ALGO S-4.7).
        full_set.entry(g_root).or_insert_with(|| {
            let info = graph.gnode_info(g_root).expect("G-root must be live");
            AnalysisEntry {
                gnode: g_root,
                depth: info.depth,
                v_depth: 0,
                importance: info.own,
                start: info.start,
                end: info.end,
                is_competitive: false,
            }
        });

        let full: Vec<AnalysisEntry<C, V>> = full_set.into_values().collect();

        Self { competitive, full }
    }
}

impl<C: Coordinate, V: Accumulator> AnalysisSet<C, V> {
    /// The competitively selected entries.
    #[must_use]
    pub fn competitive(&self) -> &[AnalysisEntry<C, V>] {
        &self.competitive
    }

    /// The full analysis set (competitive + ancestors).
    #[must_use]
    pub fn full(&self) -> &[AnalysisEntry<C, V>] {
        &self.full
    }

    /// Number of competitive entries.
    #[must_use]
    pub const fn competitive_count(&self) -> usize {
        self.competitive.len()
    }

    /// Total entries (competitive + ancestors).
    #[must_use]
    pub const fn total_count(&self) -> usize {
        self.full.len()
    }

    /// Whether a given `GNodeId` is in the full analysis set.
    #[must_use]
    pub fn contains(&self, gnode: GNodeId) -> bool {
        self.full.iter().any(|e| e.gnode == gnode)
    }

    /// Whether a given `GNodeId` is competitively selected.
    #[must_use]
    pub fn is_competitive(&self, gnode: GNodeId) -> bool {
        self.competitive.iter().any(|e| e.gnode == gnode)
    }
}

impl<C: Coordinate, V: Inspectable> AnalysisSet<C, V> {
    /// Build a summary snapshot of the current analysis set — the whole
    /// selection, whether or not each cell has a tracker yet.
    ///
    /// See [`summary_online`](Self::summary_online) for the reading over the
    /// cells that are online, which is the one the batch report carries.
    #[must_use]
    pub fn summary(&self) -> AnalysisSetSummary {
        self.summarise(|_| true)
    }

    /// Build a summary snapshot of the producing sets — the reading the batch
    /// report carries.
    ///
    /// `online` names the cells that currently have a tracker. Every figure is
    /// taken over the selection intersected with it, because the producing
    /// sets are the online ones — every figure but the investment count, which
    /// is documented as the whole investment and is reported as the whole
    /// selection here too. Filtering that one would report an investment with
    /// the warming cells removed, and those are exactly the part of it that
    /// has been paid for and has not yet produced anything.
    /// [`summary`](Self::summary) reads the whole selection throughout, which
    /// is the investment set: it includes cells still warming in staging,
    /// whose depths and importances would widen these ranges with cells no
    /// observation has yet reached. The two readings are separate methods
    /// because the difference between them is exactly what a caller has to
    /// choose.
    #[must_use]
    pub fn summary_online(&self, online: &BTreeSet<GNodeId>) -> AnalysisSetSummary {
        self.summarise(|gnode| online.contains(&gnode))
    }

    /// Shared body of the two summaries, over whichever entries are included.
    fn summarise(&self, included: impl Fn(GNodeId) -> bool) -> AnalysisSetSummary {
        let competitive_included = || self.competitive.iter().filter(|e| included(e.gnode));
        let full_included = || self.full.iter().filter(|e| included(e.gnode));

        let competitive_size = competitive_included().count();
        let full_size = full_included().count();

        let depth_range = if full_size == 0 {
            (0, 0)
        } else {
            let mut min_d = u32::MAX;
            let mut max_d = 0u32;
            for entry in full_included() {
                min_d = min_d.min(entry.depth);
                max_d = max_d.max(entry.depth);
            }
            (min_d, max_d)
        };

        let (importance_range, v_depth_range) = if competitive_size == 0 {
            ((0.0, 0.0), (0, 0))
        } else {
            let mut min_imp = f64::INFINITY;
            let mut max_imp = f64::NEG_INFINITY;
            let mut min_vd = usize::MAX;
            let mut max_vd = 0usize;
            for entry in competitive_included() {
                let imp = entry.importance.to_f64_approx();
                min_imp = min_imp.min(imp);
                max_imp = max_imp.max(imp);
                min_vd = min_vd.min(entry.v_depth);
                max_vd = max_vd.max(entry.v_depth);
            }
            ((min_imp, max_imp), (min_vd, max_vd))
        };

        AnalysisSetSummary {
            competitive_size,
            full_size,
            // The investment set is the whole selection, whether or not a cell
            // is online yet, so this figure is taken before the filter rather
            // than after it: filtered, it would report an investment that
            // excluded every cell still being warmed, which is precisely the
            // part of the investment that has been paid for and not yet
            // returned. The orchestrator replaces it with the tracker
            // population it can see directly.
            investment_set_size: self.full.len(),
            depth_range,
            importance_range,
            v_depth_range,
            degenerate_cells_skipped: 0,
        }
    }
}
