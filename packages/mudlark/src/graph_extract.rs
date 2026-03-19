// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! PEWEI extraction, layer iteration, and convenience
//! construction for `GvGraph`.
//!
//! Extracted from `graph.rs` per ADR-M-030 Phase D.

use crate::graph::{Config, GvGraph};
use crate::handle::VNodeId;
use crate::traits::{Accumulator, Coordinate, Inspectable};

impl<C: Coordinate, V: Accumulator + Inspectable, const N: u32> GvGraph<C, V, N> {
    // ── PEWEI Extraction (Phase 4, Step 3 — ADR-M-021) ───────────

    /// Extract a PEWEI snapshot: a significance-ordered, layered
    /// representation of the current G-V Tree state.
    ///
    /// Performs a breadth-first walk of the V-Tree. Each V-entry is
    /// classified by the state of its backing G-node:
    ///
    /// - **Terminal** (`GState::Terminal`) → [`Terminal`](crate::pewei::Terminal).
    /// - **Internal / Semi-internal** → [`Transition`](crate::pewei::Transition)
    ///   (DC-021-5: semi-internal nodes are transitions).
    ///
    /// V-structural nodes are pure scaffolding and are not emitted —
    /// their children are enqueued for the next BFS depth.
    ///
    /// An all-zero tree produces a single layer with one zero-intensity
    /// terminal (DC-021-4: the root V-entry always exists physically).
    ///
    /// Cost: $O(L + S)$ — every V-node visited once.
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
    /// let pewei = g.extract();
    /// assert!(pewei.layer_count() >= 1);
    /// assert!(pewei.total_energy() >= 10);
    ///
    /// // Layers are significance-ordered: iterate from dominant to fine.
    /// for layer in &pewei.layers {
    ///     for t in &layer.terminals {
    ///         assert!(t.start < t.end);
    ///     }
    /// }
    /// ```
    #[must_use]
    pub fn extract(&self) -> crate::pewei::Pewei<C, V> {
        use std::collections::VecDeque;

        use crate::gnode::GState;
        use crate::pewei::{Layer, Pewei, Terminal, Transition};
        use crate::vnode::VKind;

        let domain_start = C::zero();
        let domain_end = C::domain_max(N);

        let Some(v_root) = self.v_root else {
            return Pewei {
                domain_start,
                domain_end,
                layers: vec![],
            };
        };

        // BFS queue: (VNodeId, bfs_depth).
        let mut queue = VecDeque::new();
        queue.push_back((v_root, 0u32));

        let mut layers: Vec<Layer<C, V>> = Vec::new();

        while let Some((vid, bfs_depth)) = queue.pop_front() {
            let vnode = self.vnodes.get(vid.index());

            match &vnode.kind {
                VKind::Entry { gnode, .. } => {
                    let g = self.gnodes.get(gnode.index());
                    let g_depth = self.gnode_depth(*gnode);

                    // Ensure the layer exists.
                    while layers.len() <= bfs_depth as usize {
                        layers.push(Layer {
                            transitions: Vec::new(),
                            terminals: Vec::new(),
                        });
                    }
                    let layer = &mut layers[bfs_depth as usize];

                    match g.state() {
                        GState::Terminal => {
                            layer.terminals.push(Terminal {
                                start: g.lo,
                                end: g.hi,
                                intensity: g.own,
                                depth: g_depth,
                                v_depth: bfs_depth,
                            });
                        }
                        GState::SemiInternal | GState::Internal => {
                            layer.transitions.push(Transition {
                                start: g.lo,
                                end: g.hi,
                                baseline: g.own,
                                total: g.sum,
                                refinement: V::sub(g.sum, g.own),
                                depth: g_depth,
                                v_depth: bfs_depth,
                            });
                        }
                    }
                }
                VKind::Structural { children, .. } => {
                    for i in 0..children.len() {
                        let (child_id, _intensity) = children.get(i);
                        queue.push_back((child_id, bfs_depth + 1));
                    }
                }
            }
        }

        Pewei {
            domain_start,
            domain_end,
            layers,
        }
    }

    /// Lazy V-Tree BFS iterator yielding `(layer_index, Node<C, V>)`.
    ///
    /// Streaming counterpart to [`extract()`](Self::extract): same
    /// traversal order, same node classification (structural V-nodes
    /// skipped), but yields [`Node`](crate::view::Node) view types
    /// without building the full owned `Pewei` snapshot.
    ///
    /// Layer index is the V-Tree BFS depth — nodes at layer 0 are
    /// the most significant. Callers can distinguish terminals from
    /// transitions via [`Node::state`](crate::view::Node::state) or
    /// [`Node::to_cell()`](crate::view::Node::to_cell).
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// # let cfg = Config {
    /// #     split_threshold: 5u64, depth_create: 3, depth_evict: 6,
    /// #     budget: None, alpha_relax: 0.75, bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// # g.observe(42, 10u64);
    /// // Each yielded node covers a valid interval.
    /// let nodes: Vec<_> = g.layers().collect();
    /// assert!(!nodes.is_empty());
    /// for (layer, node) in &nodes {
    ///     assert!(node.start < node.end);
    /// }
    /// ```
    pub fn layers(&self) -> impl Iterator<Item = (usize, crate::view::Node<C, V>)> + '_ {
        let mut queue = std::collections::VecDeque::new();
        if let Some(v_root) = self.v_root {
            queue.push_back((v_root, 0usize));
        }
        Layers { graph: self, queue }
    }

    /// Construct a `GvGraph` and populate it with observations from
    /// an iterator.
    ///
    /// Equivalent to:
    /// ```ignore
    /// let mut g = GvGraph::new(config);
    /// g.extend(iter);
    /// ```
    ///
    /// This replaces `FromIterator`, which would require a fictitious
    /// `Config::default()` — `Config<V>` has no sensible `Default`.
    ///
    /// # Examples
    ///
    /// ```
    /// use torrust_mudlark::{Config, GvGraph};
    /// let cfg = Config {
    ///     split_threshold: 5u64,
    ///     depth_create: 3,
    ///     depth_evict: 6,
    ///     budget: None,
    ///     alpha_relax: 0.75,
    ///     bounded_eviction: true,
    /// };
    /// let observations = vec![(10u64, 1u64), (20, 2), (30, 3)];
    /// let g = GvGraph::<u64, u64, 8>::from_observations(cfg, observations);
    /// assert_eq!(g.total_sum(), 6);
    /// ```
    #[must_use]
    pub fn from_observations<O, I>(config: Config<V>, iter: I) -> Self
    where
        O: crate::traits::Observation<V>,
        I: IntoIterator<Item = (C, O)>,
    {
        let mut graph = Self::new(config);
        graph.extend(iter);
        graph
    }
}

/// Bulk-insert observations from an iterator
///
/// Each `(coordinate, observation)` pair is fed to
/// [`GvGraph::observe`] in order.  This is the idiomatic way to
/// populate a graph from a collection or stream of data.
///
/// # Examples
///
/// ```
/// # use torrust_mudlark::{Config, GvGraph};
/// # let cfg = Config {
/// #     split_threshold: 5u64, depth_create: 3, depth_evict: 6,
/// #     budget: None, alpha_relax: 0.75, bounded_eviction: true,
/// # };
/// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
///
/// let data = vec![(10u64, 1u64), (20, 2), (30, 3)];
/// g.extend(data);
///
/// assert_eq!(g.total_sum(), 6);
/// ```
impl<C, V, O, const N: u32> Extend<(C, O)> for GvGraph<C, V, N>
where
    C: Coordinate,
    V: Accumulator + Inspectable,
    O: crate::traits::Observation<V>,
{
    fn extend<I: IntoIterator<Item = (C, O)>>(&mut self, iter: I) {
        for (coord, delta) in iter {
            self.observe(coord, delta);
        }
    }
}

/// BFS iterator over V-Tree entries, yielding `(layer_index, Node)`.
///
/// Created by [`GvGraph::layers()`]. Structural V-nodes are traversed
/// but not yielded — only V-Entry nodes (backing G-nodes) produce
/// output, matching `extract()`'s behavior.
struct Layers<'a, C: Coordinate, V: Accumulator, const N: u32> {
    graph: &'a GvGraph<C, V, N>,
    queue: std::collections::VecDeque<(VNodeId, usize)>,
}

impl<C: Coordinate, V: Accumulator, const N: u32> Iterator for Layers<'_, C, V, N> {
    type Item = (usize, crate::view::Node<C, V>);

    fn next(&mut self) -> Option<Self::Item> {
        use crate::vnode::VKind;

        loop {
            let (vid, bfs_depth) = self.queue.pop_front()?;
            let vnode = self.graph.vnodes.get(vid.index());

            match &vnode.kind {
                VKind::Structural { children, .. } => {
                    for i in 0..children.len() {
                        let (child_id, _) = children.get(i);
                        self.queue.push_back((child_id, bfs_depth + 1));
                    }
                    // Skip structural nodes — continue loop.
                }
                VKind::Entry { gnode, .. } => {
                    let g = self.graph.gnodes.get(gnode.index());
                    let g_depth = self.graph.gnode_depth(*gnode);
                    let node = crate::view::Node {
                        start: g.lo,
                        end: g.hi,
                        own: g.own,
                        sum: g.sum,
                        depth: g_depth,
                        state: g.state(),
                        gnode_id: *gnode,
                    };
                    return Some((bfs_depth, node));
                }
            }
        }
    }
}
