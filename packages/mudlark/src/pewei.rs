// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! PEWEI extraction snapshot types.
//!
//! A [`Pewei`] is an owned, serialisable **significance-ordered
//! snapshot** of a [`GvGraph`](crate::graph::GvGraph). Calling
//! [`GvGraph::extract()`](crate::graph::GvGraph::extract) captures
//! the graph's spatial intensity model as a portable, self-contained
//! value — no lifetimes, no arena handles, ready for inspection or
//! serialisation.
//!
//! ## Mental model
//!
//! The crate uses a silver halide film analogy
//! ([ADR-M-032](https://github.com/torrust/torrust-index/blob/develop/packages/mudlark/adr/032-three-surface-model.md)):
//! a [`GvGraph`](crate::graph::GvGraph) is **undeveloped film** —
//! continuously exposed and regressing, never fixed. A `Pewei` is
//! the **contact print**: a stable, detached developed image made
//! from that negative at a point in time. The negative keeps
//! changing; the print does not.
//!
//! The four types in this module map to the print:
//!
//! | Type           | Print analogy                                        |
//! |----------------|------------------------------------------------------|
//! | [`Pewei`]      | The contact print itself — complete developed image.  |
//! | [`Layer`]      | One dye-coupler layer (coarse → fine detail).         |
//! | [`Transition`] | Crystal-aggregate boundary (macro → micro density).   |
//! | [`Terminal`]   | Fully resolved grain — single crystal, single density.|
//!
//! ## Structure
//!
//! The snapshot is a sequence of **layers** ordered from most to
//! least significant. Layer 0 holds the dominant energy structures;
//! each subsequent layer adds progressively finer spatial detail.
//! Within each layer, two populations appear:
//!
//! | Type           | Description                                          |
//! |----------------|------------------------------------------------------|
//! | [`Pewei`]      | Top-level snapshot: domain bounds + layers.           |
//! | [`Layer`]      | One depth-level of the significance hierarchy.        |
//! | [`Transition`] | Region where sub-scale structure was confirmed.       |
//! | [`Terminal`]   | Leaf region: finest-resolution measurement.           |
//!
//! ## Usage
//!
//! ```
//! # use torrust_mudlark::{Config, GvGraph};
//! # let mut g = GvGraph::<u64, u64, 8>::new(Config {
//! #     split_threshold: 5u64, depth_create: 3, depth_evict: 6,
//! #     budget: None, alpha_relax: 0.75, bounded_eviction: true,
//! # });
//! // Expose the film — accumulate spatial observations.
//! g.observe(42, 10u64);
//! g.observe(200, 5u64);
//!
//! // Make a contact print — extract the significance snapshot.
//! let pewei = g.extract();
//!
//! // Walk the significance hierarchy layer by layer.
//! for layer in &pewei.layers {
//!     for tr in &layer.transitions {
//!         // Regions with confirmed sub-structure.
//!         let _ = (tr.start, tr.end, tr.baseline, tr.refinement);
//!     }
//!     for t in &layer.terminals {
//!         // Leaf regions — finest-resolution measurements.
//!         let _ = (t.start, t.end, t.intensity);
//!     }
//! }
//!
//! // Reconstruct a signal estimate from all layers.
//! let spans = pewei.reconstruct(pewei.layer_count().saturating_sub(1));
//! assert_eq!(spans.first().unwrap().start, 0);
//! assert_eq!(spans.last().unwrap().end, 256);
//!
//! // Truncate to layer 0 only for a coarser, more compact view.
//! let coarse = pewei.reconstruct(0);
//! assert!(coarse.len() <= spans.len());
//! ```
//!
//! ## Reconstruction
//!
//! [`Pewei::reconstruct()`] produces a `Vec<Span<C, V>>` partitioning
//! the domain into non-overlapping intervals, each carrying the
//! fully-summed intensity at that region (all ancestral baselines
//! pro-rated and folded in). See ADR-M-023 for the algorithm and
//! energy-conservation proof.
//!
//! ### Implementation notes
//!
//! The internal lookup table uses a Structure-of-Arrays (`SoA`) layout:
//! each depth bucket stores `starts` and `nodes` as separate parallel
//! arrays. Binary search runs on the `starts` array only — at 8 bytes
//! per element for `u64`, the search working set is ~3× smaller than
//! an interleaved `(start, NodeRef)` layout, reducing bandwidth
//! consumed during the ~19 comparison steps that dominate at GB-scale
//! PEWEIs (each step is a likely cache miss). The `nodes` array is
//! touched exactly once, after the search lands.
//!
//! Introduced in Phase 4, Steps 3–4 (ADR-M-021, ADR-M-022, ADR-M-023).

use crate::traits::{Accumulator, Coordinate, Proratable, Weighable};
use crate::view::Span;

// ── Pewei ────────────────────────────────────────────────────────────

/// Progressive Entropic-Wavelet Exposure Image — the extracted
/// significance-ordered snapshot of a [`GvGraph`](crate::graph::GvGraph)
///
/// A PEWEI captures the graph's spatial intensity model as an
/// immutable, portable value. The layers are ordered by decreasing
/// significance: layer 0 holds the dominant energy structures,
/// deeper layers add progressively finer spatial detail.
///
/// *Photographic analogy:* the **contact print** — a complete,
/// self-describing image made from the ever-changing negative
/// ([`GvGraph`](crate::GvGraph)). The negative keeps accumulating
/// and decaying; the print is a stable, detached record.
///
/// Typical usage:
/// - **Inspect layers** to understand where energy is concentrated
///   and at what spatial resolution.
/// - **Reconstruct** a signal estimate via
///   [`reconstruct()`](Self::reconstruct), truncating at any layer
///   for progressive detail.
/// - **Serialize** (with the `serde` feature) for storage or
///   network transfer.
///
/// Produced by [`GvGraph::extract()`](crate::graph::GvGraph::extract).
/// The snapshot is self-contained: no references to the original
/// graph, no lifetimes, no arena handles. `domain_start` and
/// `domain_end` are stored so that reconstruction and serialisation
/// do not require the original `GvGraph` or its `N` const generic.
///
/// *In the [three-surface model](https://github.com/torrust/torrust-index/blob/develop/packages/mudlark/adr/032-three-surface-model.md),
/// a `Pewei` is the **contact print** — a complete developed image
/// detached from the negative. Like a real contact print, it can be
/// inspected, copied, and stored independently of the original film.*
///
/// # Examples
///
/// ```
/// # use torrust_mudlark::{Config, GvGraph, Pewei};
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
/// g.observe(200, 5u64);
/// let pewei: Pewei<u64, u64> = g.extract();
/// assert_eq!(pewei.domain_start, 0);
/// assert_eq!(pewei.domain_end, 256);
/// assert!(!pewei.layers.is_empty());
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pewei<C: Coordinate, V: Accumulator> {
    /// Lower bound of the domain (inclusive). Always `C::zero()`.
    pub domain_start: C,
    /// Upper bound of the domain (exclusive). Always `C::domain_max(N)`.
    pub domain_end: C,
    /// Layers in decreasing significance order (BFS depth 0 first).
    pub layers: Vec<Layer<C, V>>,
}

impl<C: Coordinate, V: Accumulator + Proratable> Pewei<C, V> {
    /// Number of layers.
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
    /// # g.observe(42, 10u64);
    /// let pewei = g.extract();
    /// assert!(pewei.layer_count() >= 1);
    /// ```
    #[inline]
    #[must_use]
    pub const fn layer_count(&self) -> usize {
        self.layers.len()
    }

    /// Total number of nodes (transitions + terminals) across all layers.
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
    /// # g.observe(42, 10u64);
    /// let pewei = g.extract();
    /// assert!(pewei.node_count() >= 1);
    /// ```
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.layers.iter().map(|l| l.transitions.len() + l.terminals.len()).sum()
    }

    /// Total energy: sum of all terminal intensities plus all
    /// transition baselines. Equals the G-root sum when extracted
    /// from a consistent tree. Returns `V::zero()` for an empty
    /// snapshot.
    ///
    /// This walks all layers — $O(n)$ where $n$ is `node_count()`.
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
    /// g.observe(200, 5u64);
    /// let pewei = g.extract();
    /// assert_eq!(pewei.total_energy(), 15);
    /// ```
    #[must_use]
    pub fn total_energy(&self) -> V {
        let mut acc = V::zero();
        for layer in &self.layers {
            for t in &layer.transitions {
                acc = V::add(acc, t.baseline);
            }
            for t in &layer.terminals {
                acc = V::add(acc, t.intensity);
            }
        }
        acc
    }

    /// Reconstruct a signal estimate from the first `max_layer + 1`
    /// layers of this PEWEI.
    ///
    /// Returns a `Vec<Span<C, V>>` that partitions
    /// `[domain_start, domain_end)` with no gaps and no overlaps.
    /// Each span's intensity includes all ancestral baselines,
    /// pro-rated to the span's width (ADR-M-023, DC-023-2).
    ///
    /// # Energy conservation
    ///
    /// The sum of all output span intensities equals the total energy
    /// of the visible layers — equivalent to the G-root sum when
    /// `max_layer >= layer_count() - 1`.
    ///
    /// # Integer rounding
    ///
    /// For integer `Accumulator` types, `prorate(1, 2)` truncates:
    /// `11.prorate(1, 2) = 5`. Worst-case cumulative loss is bounded
    /// by tree depth ($\le N$ units). Negligible relative to
    /// truncation error (ADR-M-023 §Integer rounding note).
    ///
    /// # Panics
    ///
    /// Panics if any coordinate in the PEWEI is NaN. This is
    /// structurally impossible for snapshots produced by
    /// [`GvGraph::extract()`](crate::graph::GvGraph::extract) — all
    /// coordinates arise from dyadic arithmetic on valid inputs.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph, Span};
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
    /// g.observe(200, 5u64);
    /// let pewei = g.extract();
    /// let spans = pewei.reconstruct(pewei.layer_count().saturating_sub(1));
    ///
    /// // Spans partition the full domain with no gaps.
    /// assert_eq!(spans.first().unwrap().start, 0);
    /// assert_eq!(spans.last().unwrap().end, 256);
    /// for w in spans.windows(2) {
    ///     assert_eq!(w[0].end, w[1].start);
    /// }
    ///
    /// // Energy conservation: span intensities sum to total energy.
    /// let sum: u64 = spans.iter().map(|s| s.intensity).sum();
    /// // Integer prorate truncation may lose at most N units.
    /// assert!(pewei.total_energy().abs_diff(sum) <= 8);
    /// ```
    #[must_use]
    pub fn reconstruct(&self, max_layer: usize) -> Vec<Span<C, V>> {
        if self.layers.is_empty() {
            return vec![Span {
                start: self.domain_start,
                end: self.domain_end,
                intensity: V::zero(),
                depth: 0,
            }];
        }

        let max_layer = max_layer.min(self.layers.len() - 1);
        let visible = &self.layers[..=max_layer];
        let lookup = RegionLookup::build(visible);

        // Pre-allocate: every terminal produces one span, every
        // transition produces at most one span (the invisible
        // sibling) plus passes through. The terminal count is a
        // tight lower bound; +1 accounts for the root if it appears
        // as a single truncated span.
        let estimated = visible.iter().map(|l| l.terminals.len() + l.transitions.len()).sum::<usize>() + 1;
        let mut output = Vec::with_capacity(estimated);

        descend(
            &lookup,
            visible,
            self.domain_start,
            self.domain_end,
            0,
            V::zero(),
            &mut output,
        );
        output
    }
}

// ── RegionLookup (private) ───────────────────────────────────────────

/// A reference into the `layers` slice — either a transition or a
/// terminal, identified by `(layer_index, item_index)`.
///
/// Stored in a parallel `nodes` array, only touched once after the
/// binary search on the `starts` array has landed.
#[derive(Debug, Clone, Copy)]
enum NodeRef {
    Transition { layer: u32, idx: u32 },
    Terminal { layer: u32, idx: u32 },
}

/// Structure-of-Arrays depth bucket for bandwidth-efficient lookup.
///
/// Binary search runs on `starts` only (8 bytes per element for
/// `u64`). The `nodes` array is parallel and indexed by the search
/// result — touched exactly once, after the search lands.
///
/// At GB-scale PEWEIs the search working set is ~3× smaller than
/// an interleaved `(start, NodeRef)` layout, keeping more of the
/// searched data in cache.
struct DepthBucket<C: Coordinate> {
    starts: Vec<C>,
    nodes: Vec<NodeRef>,
}

/// Depth-bucketed lookup table for PEWEI reconstruction.
///
/// Indexed by G-Tree depth. Each bucket's `starts` array is sorted,
/// enabling binary search. Within a given G-depth, `start` is unique
/// (dyadic intervals at the same scale never share a start position),
/// so `(depth, start)` is a unique key.
///
/// Entries are index-based references into the original `layers`
/// slice — no field values are copied into the table.
struct RegionLookup<C: Coordinate> {
    by_depth: Vec<DepthBucket<C>>,
}

impl<C: Coordinate> RegionLookup<C> {
    /// Build the lookup from visible layers.
    ///
    /// Cost: $O(n \log n)$ — one pass to collect, then sort each
    /// depth bucket.
    fn build<V: Accumulator>(layers: &[Layer<C, V>]) -> Self {
        let max_depth = layers
            .iter()
            .flat_map(|l| {
                l.transitions
                    .iter()
                    .map(|t| t.depth)
                    .chain(l.terminals.iter().map(|t| t.depth))
            })
            .max()
            .unwrap_or(0) as usize;

        // Collect (start, NodeRef) pairs per depth, then split into
        // parallel arrays after sorting.
        let mut pairs: Vec<Vec<(C, NodeRef)>> = (0..=max_depth).map(|_| Vec::new()).collect();

        #[allow(clippy::cast_possible_truncation)] // layer/item indices ≤ u32::MAX for any realistic PEWEI
        for (li, layer) in layers.iter().enumerate() {
            for (ti, t) in layer.transitions.iter().enumerate() {
                pairs[t.depth as usize].push((
                    t.start,
                    NodeRef::Transition {
                        layer: li as u32,
                        idx: ti as u32,
                    },
                ));
            }
            for (ti, t) in layer.terminals.iter().enumerate() {
                pairs[t.depth as usize].push((
                    t.start,
                    NodeRef::Terminal {
                        layer: li as u32,
                        idx: ti as u32,
                    },
                ));
            }
        }

        // Sort each bucket by `start`, then split into SoA.
        // Safety: coordinates from `extract()` are dyadic arithmetic
        // on valid inputs — NaN is structurally unreachable.
        let by_depth = pairs
            .into_iter()
            .map(|mut bucket| {
                bucket.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());
                let (starts, nodes) = bucket.into_iter().unzip();
                DepthBucket { starts, nodes }
            })
            .collect();

        Self { by_depth }
    }

    /// Look up the node reference at `(g_depth, start)`.
    ///
    /// Binary search touches only the `starts` array. The `nodes`
    /// entry is read once at the final index.
    ///
    /// Returns `None` if no PEWEI node exists at that position
    /// (gap region — background only).
    fn get(&self, start: C, g_depth: usize) -> Option<NodeRef> {
        let bucket = self.by_depth.get(g_depth)?;
        // Safety: same NaN argument as `build()`.
        let idx = bucket.starts.binary_search_by(|s| s.partial_cmp(&start).unwrap()).ok()?;
        Some(bucket.nodes[idx])
    }
}

// ── descend (private) ────────────────────────────────────────────────

/// Resolve a `NodeRef` to the total energy of that node.
///
/// Terminals: `intensity`. Transitions: `total`.
fn node_total<C: Coordinate, V: Accumulator>(layers: &[Layer<C, V>], nr: NodeRef) -> V {
    match nr {
        NodeRef::Terminal { layer, idx } => layers[layer as usize].terminals[idx as usize].intensity,
        NodeRef::Transition { layer, idx } => layers[layer as usize].transitions[idx as usize].total,
    }
}

/// Top-down recursive descent for PEWEI reconstruction (ADR-M-023
/// DC-023-2).
///
/// `accumulated` carries the ancestral background pro-rated to the
/// current region. At every node the energy accounting identity
/// holds: the total intensity emitted for a region equals
/// `accumulated + node.total` (or just `accumulated` for gaps).
fn descend<C: Coordinate, V: Accumulator + Proratable>(
    lookup: &RegionLookup<C>,
    layers: &[Layer<C, V>],
    start: C,
    end: C,
    g_depth: usize,
    accumulated: V,
    output: &mut Vec<Span<C, V>>,
) {
    match lookup.get(start, g_depth) {
        None => {
            // Gap — no PEWEI node here. Emit background only.
            // Use the current G-depth so consumers see the resolution
            // at which the gap was discovered, consistent with the
            // truncated-child spans inside the Transition arm.
            #[allow(clippy::cast_possible_truncation)]
            output.push(Span {
                start,
                end,
                intensity: accumulated,
                depth: g_depth as u32,
            });
        }
        Some(NodeRef::Terminal { layer, idx }) => {
            let t = &layers[layer as usize].terminals[idx as usize];
            output.push(Span {
                start,
                end,
                intensity: V::add(accumulated, t.intensity),
                depth: t.depth,
            });
        }
        Some(NodeRef::Transition { layer, idx }) => {
            let tr = &layers[layer as usize].transitions[idx as usize];
            let baseline = tr.baseline;
            let total = tr.total;
            let depth = tr.depth;

            let mid = C::midpoint(start, end);
            let total_bg = V::add(accumulated, baseline);
            let half_bg = total_bg.prorate(1, 2);
            let child_depth = g_depth + 1;

            let left_ref = lookup.get(start, child_depth);
            let right_ref = lookup.get(mid, child_depth);

            match (left_ref, right_ref) {
                (Some(_), Some(_)) => {
                    // Both children visible — recurse into each half.
                    descend(lookup, layers, start, mid, child_depth, half_bg, output);
                    descend(lookup, layers, mid, end, child_depth, half_bg, output);
                }
                (Some(left_nr), None) => {
                    // Left visible, right invisible.
                    descend(lookup, layers, start, mid, child_depth, half_bg, output);
                    let left_total = node_total(layers, left_nr);
                    let remainder = V::sub(V::sub(total, baseline), left_total);
                    output.push(Span {
                        start: mid,
                        end,
                        intensity: V::add(half_bg, remainder),
                        depth,
                    });
                }
                (None, Some(right_nr)) => {
                    // Left invisible, right visible.
                    let right_total = node_total(layers, right_nr);
                    let remainder = V::sub(V::sub(total, baseline), right_total);
                    output.push(Span {
                        start,
                        end: mid,
                        intensity: V::add(half_bg, remainder),
                        depth,
                    });
                    descend(lookup, layers, mid, end, child_depth, half_bg, output);
                }
                (None, None) => {
                    // Fully truncated — emit total.
                    output.push(Span {
                        start,
                        end,
                        intensity: V::add(accumulated, total),
                        depth,
                    });
                }
            }
        }
    }
}

// ── Layer ────────────────────────────────────────────────────────────

/// One depth-level of the significance hierarchy
///
/// Layer 0 contains the dominant energy structures — the most
/// significant regions the index has identified. Each subsequent
/// layer adds finer spatial detail that was validated against the
/// layers above.
///
/// *Photographic analogy:* a **dye-coupler layer** — coarse
/// densities first, fine detail overlaid progressively
/// (cf. Kodachrome's multi-layer process).
///
/// Within each layer, two populations appear:
///
/// - **`transitions`**: regions where sub-scale structure was
///   confirmed — the index found meaningful spatial variation
///   inside. Each carries a `baseline` (pre-subdivision energy)
///   and `refinement` (energy from finer scales).
/// - **`terminals`**: leaf regions with no further subdivision —
///   the finest-resolution measurements available.
///
/// Nodes within a layer are ordered by BFS encounter order,
/// mirroring the V-Tree's competitive ranking.
///
/// # Examples
///
/// ```
/// # use torrust_mudlark::{Config, GvGraph, Layer};
/// # let cfg = Config {
/// #     split_threshold: 5u64,
/// #     depth_create: 3,
/// #     depth_evict: 6,
/// #     budget: None,
/// #     alpha_relax: 0.75,
/// #     bounded_eviction: true,
/// # };
/// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
/// # g.observe(42, 10u64);
/// let pewei = g.extract();
///
/// // Layers are ordered by significance (most significant first).
/// assert!(!pewei.layers.is_empty());
///
/// // Each layer has two populations: transitions and terminals.
/// for layer in &pewei.layers {
///     // Transitions: regions where the index found structure inside.
///     for tr in &layer.transitions {
///         assert!(tr.total >= tr.baseline);
///     }
///     // Terminals: leaf regions with no further subdivision.
///     for t in &layer.terminals {
///         assert!(t.start < t.end);
///     }
/// }
///
/// // At least one terminal exists across all layers.
/// assert!(pewei.layers.iter().any(|l| !l.terminals.is_empty()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Layer<C: Coordinate, V: Accumulator> {
    /// Phase-transition nodes at this V-Tree depth.
    pub transitions: Vec<Transition<C, V>>,
    /// Terminal (leaf) nodes at this V-Tree depth.
    pub terminals: Vec<Terminal<C, V>>,
}

// ── Transition ───────────────────────────────────────────────────────

/// A region where the index confirmed sub-scale spatial structure
///
/// *Photographic analogy:* the **crystal-aggregate boundary** —
/// where macro-density gives way to resolved micro-structure.
///
/// When accumulated observations in a region exceed the split
/// threshold, the index subdivides and creates child nodes to
/// capture finer detail. The transition node records both the
/// **baseline** (energy accumulated before subdivision) and the
/// **total** (baseline + all descendant contributions). The
/// difference, `refinement = total - baseline`, measures how much
/// energy the finer scales contributed.
///
/// During [`Pewei::reconstruct()`], the baseline is pro-rated as
/// uniform background across the region while children layer
/// finer structure on top.
///
/// Regions where only one half has been further subdivided
/// (semi-internal) are also classified as transitions: one half
/// is covered by a child while the other remains on this node.
///
/// *Analogy: a **crystal-aggregate boundary** — the edge within the
/// emulsion where macro-density (`baseline`) gives way to resolved
/// micro-structure (`refinement`).*
///
/// # Examples
///
/// ```
/// # use torrust_mudlark::{Config, GvGraph, Transition};
/// # let cfg = Config {
/// #     split_threshold: 5u64,
/// #     depth_create: 3,
/// #     depth_evict: 6,
/// #     budget: None,
/// #     alpha_relax: 0.75,
/// #     bounded_eviction: true,
/// # };
/// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
/// // Concentrate enough energy to trigger splits and create transitions.
/// for _ in 0..20 {
///     g.observe(42, 10u64);
/// }
/// let pewei = g.extract();
/// let transitions: Vec<&Transition<u64, u64>> =
///     pewei.layers.iter().flat_map(|l| &l.transitions).collect();
/// if let Some(tr) = transitions.first() {
///     assert!(tr.start < tr.end);
///     assert!(tr.total >= tr.baseline);
///     assert_eq!(tr.refinement, tr.total - tr.baseline);
/// }
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Transition<C: Coordinate, V: Accumulator> {
    /// Lower bound of the dyadic range (inclusive).
    pub start: C,
    /// Upper bound of the dyadic range (exclusive).
    pub end: C,
    /// Baseline energy: `g.own` — energy before structure was confirmed.
    pub baseline: V,
    /// Total energy: `g.sum` — baseline + all descendant contributions.
    pub total: V,
    /// Refinement energy: `total - baseline`. Stored for convenience
    /// (DC-021-2 Option C).
    pub refinement: V,
    /// G-Tree depth of this node.
    pub depth: u32,
    /// V-Tree depth at extraction time (= BFS depth).
    pub v_depth: u32,
}

impl<C: Coordinate, V: Accumulator + Weighable> Transition<C, V> {
    /// Local signal-to-noise ratio: `refinement / baseline`
    ///
    /// Measures confidence that the sub-scale structure in this
    /// region is meaningful. A high ratio indicates strong spatial
    /// variation relative to the background; a low ratio means the
    /// refinement is small compared to the pre-subdivision energy.
    ///
    /// Returns `None` when baseline is zero (no direct accumulation
    /// at this node, so the ratio is undefined).
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
    /// # for _ in 0..20 { g.observe(42, 10u64); }
    /// let pewei = g.extract();
    /// for tr in pewei.layers.iter().flat_map(|l| &l.transitions) {
    ///     match tr.snr() {
    ///         Some(ratio) => assert!(ratio >= 0.0),
    ///         None => { /* baseline is zero */ }
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn snr(&self) -> Option<f64> {
        let b = self.baseline.weight();
        if b == 0.0 {
            return None;
        }
        Some(self.refinement.weight() / b)
    }

    /// Width of this node's interval: `end - start`.
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
    /// # for _ in 0..20 { g.observe(42, 10u64); }
    /// let pewei = g.extract();
    /// for tr in pewei.layers.iter().flat_map(|l| &l.transitions) {
    ///     assert_eq!(tr.width(), tr.end - tr.start);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn width(&self) -> C {
        C::width(self.start, self.end)
    }
}

// ── Terminal ─────────────────────────────────────────────────────────

/// A terminal (leaf) node: a region at the finest resolution the
/// index has resolved
///
/// *Photographic analogy:* a **fully resolved grain** — the
/// smallest developable unit with no internal structure.
///
/// Terminals represent the leaf-level measurements of the spatial
/// index — regions with no further subdivision. Each terminal's
/// `intensity` is the total accumulated energy from all observations
/// routed to this region.
///
/// In a [`Pewei`], terminals in early layers carry the most
/// significant leaf measurements; terminals in deeper layers
/// represent progressively finer or less significant regions.
///
/// *Analogy: a **fully resolved grain** — the smallest developable
/// unit in the emulsion. No internal structure; a single crystal,
/// a single density reading.*
///
/// # Examples
///
/// ```
/// # use torrust_mudlark::{Config, GvGraph, Terminal};
/// # let cfg = Config {
/// #     split_threshold: 5u64,
/// #     depth_create: 3,
/// #     depth_evict: 6,
/// #     budget: None,
/// #     alpha_relax: 0.75,
/// #     bounded_eviction: true,
/// # };
/// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
/// # g.observe(42, 10u64);
/// let pewei = g.extract();
/// let terminals: Vec<&Terminal<u64, u64>> =
///     pewei.layers.iter().flat_map(|l| &l.terminals).collect();
/// assert!(!terminals.is_empty());
///
/// let t = terminals[0];
/// assert!(t.start < t.end);
/// // Each terminal carries its spatial extent and accumulated energy.
/// let _range = t.start..t.end;        // dyadic interval
/// let _energy = t.intensity;           // total routed energy (zero is valid)
/// let _resolution = t.depth;           // G-Tree depth (spatial resolution)
/// let _significance = t.v_depth;       // V-Tree depth (layer ordering)
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Terminal<C: Coordinate, V: Accumulator> {
    /// Lower bound of the dyadic range (inclusive).
    pub start: C,
    /// Upper bound of the dyadic range (exclusive).
    pub end: C,
    /// Intensity: `g.own` (= `g.sum` for terminals).
    pub intensity: V,
    /// G-Tree depth of this node.
    pub depth: u32,
    /// V-Tree depth at extraction time (= BFS depth).
    pub v_depth: u32,
}

impl<C: Coordinate, V: Accumulator> Terminal<C, V> {
    /// Width of this node's interval: `end - start`.
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
    /// # g.observe(42, 10u64);
    /// let pewei = g.extract();
    /// for t in pewei.layers.iter().flat_map(|l| &l.terminals) {
    ///     assert_eq!(t.width(), t.end - t.start);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn width(&self) -> C {
        C::width(self.start, self.end)
    }
}

#[cfg(test)]
mod tests {}
