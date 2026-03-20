// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! The `GvGraph` — primary dual-tree value-stratified index.
//!
//! This module contains the top-level struct, configuration, and
//! initialization logic. See §IDEA M-15 for the initialization
//! specification.

#[cfg(feature = "dynamic-contour-tracking")]
use std::collections::BTreeMap;
use std::sync::atomic::AtomicU32;

use crate::arena::Arena;
use crate::gnode::GNode;
use crate::handle::{GNodeId, VNodeId};
#[cfg(feature = "dynamic-contour-tracking")]
use crate::plateau::{BasisEdge, Plateau, PlateauBasis};
use crate::traits::{Accumulator, Coordinate};
use crate::view::Node;
use crate::vnode::VNode;

// ── Surface 1 view type (ADR-M-036 D2) ────────────────────────────

/// Current child handles of a G-node (left, right).
///
/// Returned by [`GvGraph::gnode_children()`]. All handles are live at
/// snapshot time but may become stale after graph mutations
/// (`observe()`, `decay()`). Intended for single-pass tree walks
/// within a read-only borrow of the graph.
///
/// For the stable parent pointer, see [`Node::parent`](crate::Node::parent)
/// (returned by [`GvGraph::gnode_info()`]).
#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GNodeChildren {
    /// Left child G-node, if any.
    pub left: Option<GNodeId>,
    /// Right child G-node, if any.
    pub right: Option<GNodeId>,
}

/// Configure how a [`GvGraph`] grows, splits, and prunes
///
/// Groups three concerns into a single value object:
///
/// - **Resolution:** [`split_threshold`](Self::split_threshold) sets
///   the minimum accumulated intensity a cell must exceed before the
///   index subdivides it into two finer spatial cells.  Lower values
///   produce finer resolution at the cost of more nodes.
/// - **Depth gates:** [`depth_create`](Self::depth_create) and
///   [`depth_evict`](Self::depth_evict) bound where in the V-Tree
///   nodes may be created or removed.  The gap between them (the
///   *buffer zone*) gives mid-depth entries time to compete before
///   eviction.
/// - **Memory budget:** [`budget`](Self::budget),
///   [`alpha_relax`](Self::alpha_relax), and
///   [`bounded_eviction`](Self::bounded_eviction) enable optional
///   dynamic depth control that keeps the live node count within a
///   hard ceiling.
///
/// All fields are public.  There is no `Default` implementation —
/// every parameter is domain-dependent.  `N` (the domain bit-width)
/// is a const generic on [`GvGraph`] itself, not a field here.
///
/// # Silver halide analogy
///
/// In the crate's photographic metaphor (see crate-level docs and
/// ADR-M-032), `Config` is the **film stock specification** — ISO
/// rating, grain-size distribution, dynamic range.  You choose it
/// at the store; it is baked into the emulsion at manufacture.
/// Once the roll is loaded ([`GvGraph::new`]), these base
/// parameters are fixed.  Runtime depth-gate adjustments live on
/// the graph itself (see [`GvGraph::depth_create`],
/// [`GvGraph::depth_evict`]).
///
/// | Field | Photographic equivalent |
/// |-------|------------------------|
/// | [`split_threshold`](Self::split_threshold) | Minimum exposure to form a developable latent image center — controls grain acuity |
/// | [`depth_create`](Self::depth_create) | Maximum crystal layers the emulsion can grow during manufacture |
/// | [`depth_evict`](Self::depth_evict) | Depth at which underdeveloped crystals dissolve — Ostwald ripening threshold |
/// | [`budget`](Self::budget) | Frame count on the roll — hard physical limit on how many grains fit |
///
/// # Validation
///
/// Validated at construction time ([`GvGraph::new`]) — panics on
/// violation:
/// - `depth_create < depth_evict` (the buffer zone must be ≥ 1).
/// - `depth_create >= 1` (the root must be able to subdivide).
/// - `alpha_relax ∈ (0.0, 1.0)`.
/// - When `budget` is `Some`:
///   `budget > max(3^(buffer+1), 2×(depth_create−1))`.
///
/// # Examples
///
/// Unbounded configuration — the graph grows without limit:
///
/// ```
/// use torrust_mudlark::{Config, GvGraph};
///
/// // Domain [0, 2^8) = [0, 256).  A cell must accumulate more than
/// // 5 before it subdivides; depth gates allow 3 levels of V-Tree
/// // creation and a 3-level buffer zone before eviction.
/// let cfg = Config {
///     split_threshold: 5u64,
///     depth_create: 3,
///     depth_evict: 6,
///     budget: None,
///     alpha_relax: 0.75,
///     bounded_eviction: true,
/// };
///
/// // Fields are public — inspect before handing to the graph.
/// assert!(cfg.depth_create < cfg.depth_evict);
/// assert!(cfg.budget.is_none());
///
/// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
/// assert_eq!(g.node_count(), 1); // single root node
///
/// // Once intensity exceeds the split threshold the root subdivides.
/// g.observe(100, 6u64);          // 6 > 5 → split triggered
/// assert!(g.node_count() > 1);
/// ```
///
/// Budgeted configuration — the graph respects a hard node ceiling:
///
/// ```
/// use torrust_mudlark::{Config, GvGraph};
///
/// let cfg = Config {
///     split_threshold: 10u64,
///     depth_create: 2,
///     depth_evict: 5,
///     budget: Some(128),
///     alpha_relax: 0.5,
///     bounded_eviction: true,
/// };
///
/// let g = GvGraph::<u64, u64, 16>::new(cfg);
///
/// // With a budget, dynamic depth control is active: the graph
/// // maintains headroom and a soft limit it uses to trigger
/// // evictions and depth-gate adjustments automatically.
/// assert_eq!(g.budget(), Some(128));
/// assert!(g.soft_limit().is_some());
/// assert!(g.headroom() > 0);
/// ```
#[derive(Debug, Clone)]
pub struct Config<V: Accumulator> {
    /// Minimum accumulated intensity a cell must exceed before
    /// subdividing ($\theta$ in the formal spec).
    ///
    /// Lower values produce finer spatial resolution at the cost of
    /// more nodes; higher values keep the tree coarser.
    pub split_threshold: V,
    /// Initial maximum V-Tree depth at which new splits are allowed.
    ///
    /// Controls how deep the tree can grow: entries deeper than this
    /// in the V-Tree cannot create children.  Provides the initial
    /// value for [`GvGraph::depth_create`]; dynamic depth control
    /// adjusts the live gate on the graph when a
    /// [`budget`](Self::budget) is set and node pressure changes.
    pub depth_create: u32,
    /// Minimum V-Tree depth at which entries become eligible for eviction
    ///
    /// Must be strictly greater than
    /// [`depth_create`](Self::depth_create) — the gap is the
    /// *buffer zone*.  Entries in the buffer zone are too deep to
    /// create children but not yet deep enough to be evicted, giving
    /// them time to accumulate observations and promote upward.
    pub depth_evict: u32,
    /// Optional hard ceiling on live G-node count
    ///
    /// When `Some(n)`, enables dynamic depth control: the graph
    /// adjusts [`depth_create`](Self::depth_create) and
    /// [`depth_evict`](Self::depth_evict) at runtime to keep the
    /// node count under `n`.  When `None`, the tree grows without
    /// limit.
    pub budget: Option<usize>,
    /// Relaxation threshold as a fraction of
    /// [`soft_limit`](crate::GvGraph::soft_limit)
    ///
    /// When the live node count drops below `soft_limit × alpha_relax`
    /// the depth gates widen, allowing the tree to grow again.
    /// Must be in `(0.0, 1.0)`.  Ignored when
    /// [`budget`](Self::budget) is `None`.
    pub alpha_relax: f64,
    /// Whether budget-triggered eviction should stop early
    ///
    /// When `true`, eviction halts as soon as the node count drops
    /// to the soft limit.  When `false`, every eviction pass runs
    /// to completion regardless of the current count.
    pub bounded_eviction: bool,
}

impl<V: Accumulator> Config<V> {
    /// Validate the configuration.
    ///
    /// Checks:
    /// - D-I3: `D_create < D_evict`.
    /// - `D_create ≥ 1` (root must be able to subdivide).
    /// - `alpha_relax ∈ (0.0, 1.0)`.
    fn validate(&self) {
        assert!(
            self.depth_create < self.depth_evict,
            "Config: D_create ({}) must be < D_evict ({}) (idea.md D-I3)",
            self.depth_create,
            self.depth_evict
        );
        assert!(
            self.depth_create >= 1,
            "Config: D_create ({}) must be >= 1",
            self.depth_create
        );
        assert!(
            self.alpha_relax > 0.0 && self.alpha_relax < 1.0,
            "Config: alpha_relax ({}) must be in (0.0, 1.0)",
            self.alpha_relax
        );
        if let Some(budget) = self.budget {
            let buffer = self.depth_evict - self.depth_create;
            let headroom = 3usize.pow(buffer + 1);
            let convergence = 2 * (self.depth_create as usize).saturating_sub(1);
            let required = headroom.max(convergence);
            assert!(
                budget > required,
                "Config: budget ({budget}) must be > max(3^(buffer+1), 2*(D_c-1)) \
                 = {required} (ADR-M-018: budget must exceed required headroom \
                 for hard ceiling guarantee)"
            );
        }
    }
}

// ── Uniform contour depth helper ──────────────────────────────────

/// Check whether a G-node's subtree has uniform contour depth.
///
/// Returns `Some(d)` if *every* contour cell (terminal or uncovered
/// semi-internal half) in the subtree is at depth `d`.
/// Returns `None` if depths vary (including any semi-internal).
///
/// Free function so it can be called from `&mut self` methods
/// without borrowing the whole `GvGraph` — only needs
/// `&Arena<GNode<C,V>>`.
#[allow(dead_code)] // Used by recompute_plateau, invariant checks, and normalize_plateaus.
pub fn uniform_contour_depth_of<C: Coordinate, V: Accumulator>(gnodes: &Arena<GNode<C, V>>, gid: GNodeId, n: u32) -> Option<u32> {
    use crate::gnode::GState;
    use crate::gtree::gnode_depth_from_interval;
    let g = gnodes.get(gid.index());
    match g.state() {
        GState::Terminal => Some(gnode_depth_from_interval(g.lo, g.hi, n)),
        GState::SemiInternal => None,
        GState::Internal => {
            let ld = g.left.and_then(|l| uniform_contour_depth_of(gnodes, l, n))?;
            let rd = g.right.and_then(|r| uniform_contour_depth_of(gnodes, r, n))?;
            if ld == rd { Some(ld) } else { None }
        }
    }
}

/// A Dual-Tree Value-Stratified Index (Kraft–McMillan Geometric-Value
/// Graph).
///
/// Maintains a spatial index over `[0, 2^N)` whose logical hierarchy
/// adapts to value distribution. Consists of two trees sharing a
/// common set of G-nodes:
///
/// - **G-Tree:** spatial routing, range queries, sum propagation.
/// - **V-Tree:** tournament bracket, proportional sampling, competitive
///   rebalancing.
///
/// # Operational model — undeveloped film, never fixed
///
/// `GvGraph` is **continuously exposed and regressing** — there is
/// no develop / fix / wash cycle.  The three core mutations map to
/// the silver halide lifecycle:
///
/// | Method | Phase | Silver halide equivalent |
/// |--------|-------|--------------------------|
/// | [`observe()`](Self::observe) | **Expose** | Photon strike — accumulate intensity at a spatial cell; subdivide when the cell exceeds the split threshold |
/// | [`decay()`](Self::decay) | **Regress** | Latent image regression — attenuate intensity; coarse structure persists, fine detail fades first (requires `V:` [`Attenuatable`](crate::Attenuatable)) |
/// | [`extract()`](Self::extract) | **Print** | Contact print — produce a detached, significance-ordered [`Pewei`](crate::Pewei) snapshot the caller owns |
///
/// Read-only operations leave the negative unchanged:
///
/// | Method | Analogy | Cost |
/// |--------|---------|------|
/// | [`get()`](Self::get) | Spot densitometer | $O(\text{depth})$ |
/// | [`plateaus()`](Self::plateaus) | Isodensity contour map | $O(1)$ borrowed |
/// | [`sample()`](Self::sample) | Random grain probe | $O(1.44\,H)$ expected (requires `V:` [`Weighable`](crate::Weighable)) |
///
/// See the crate-level documentation and ADR-M-032 for the full
/// analogy, including where it breaks: adaptive subdivision,
/// non-destructive readout, and unbounded accumulation have no
/// photographic counterpart.
///
/// # Type Parameters
///
/// - `C: Coordinate` — coordinate type (e.g. `u64`, `f64`).
/// - `V: Accumulator` — intensity type (e.g. `u64`, `f64`).
/// - `N: u32` — domain bit-width. The domain is `[0, 2^N)`.
///
/// # Examples
///
/// Create, observe, query, extract — the canonical lifecycle:
///
/// ```
/// use torrust_mudlark::{Config, GvGraph};
///
/// // Configure the index over [0, 256).
/// let cfg = Config {
///     split_threshold: 5u64,
///     depth_create: 3,
///     depth_evict: 6,
///     budget: None,
///     alpha_relax: 0.75,
///     bounded_eviction: true,
/// };
///
/// // Construct — one root node covers the full domain.
/// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
/// assert_eq!(g.node_count(), 1);
///
/// // Observe: the G-Tree refines as intensity accumulates.
/// for i in 0..20 {
///     g.observe(i * 12, 1u64);
/// }
/// assert!(g.node_count() > 1, "splits have refined the tree");
///
/// // Point query — which cell owns coordinate 42?
/// let cell = g.get(42);
/// assert!(cell.start <= 42 && 42 < cell.end);
///
/// // Spatial contour projection.
/// let plateaus = g.plateaus();
/// assert!(plateaus.len() >= 1);
///
/// // PEWEI extraction (significance-ordered snapshot).
/// let pewei = g.extract();
/// assert!(pewei.layer_count() >= 1);
/// assert_eq!(g.total_sum(), 20);
/// ```
///
/// Full expose → regress → print lifecycle:
///
/// ```
/// use torrust_mudlark::{Config, GvGraph};
///
/// let cfg = Config {
///     split_threshold: 5u64,
///     depth_create: 3,
///     depth_evict: 6,
///     budget: None,
///     alpha_relax: 0.75,
///     bounded_eviction: true,
/// };
/// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
///
/// // ① Expose — accumulate observations across the domain.
/// for i in 0..50 {
///     g.observe(i * 5, 2u64);
/// }
/// let after_expose = g.total_sum();
/// assert_eq!(after_expose, 100);
///
/// // ② Regress — uniform half-life decay over the full tree.
/// let root = g.g_root();
/// g.decay(root, 0.5, 0.0);
/// assert!(g.total_sum() < after_expose, "decay reduced intensity");
///
/// // ③ Print — detached significance-ordered snapshot.
/// let pewei = g.extract();
/// assert!(pewei.layer_count() >= 1);
/// ```
#[derive(Debug, Clone)]
pub struct GvGraph<C: Coordinate, V: Accumulator, const N: u32> {
    /// G-node arena (spatial nodes).
    pub(crate) gnodes: Arena<GNode<C, V>>,
    /// V-node arena (tournament nodes).
    pub(crate) vnodes: Arena<VNode<V>>,
    /// G-Tree root — always exists after `new()`.
    pub(crate) g_root: GNodeId,
    /// V-Tree root — entry or structural; None only if tree is empty
    /// (should not happen after initialization).
    pub(crate) v_root: Option<VNodeId>,
    /// Runtime configuration.
    pub(crate) config: Config<V>,
    /// Violation work queue — scoped to one mutation batch (ADR-M-003).
    /// Built during propagation, drained by `rebalance()`, empty on
    /// return.
    pub(crate) violations: Vec<VNodeId>,
    /// Live G-node count, for dynamic depth control (§IDEA M-7.4).
    pub(crate) node_count: u32,
    /// Live terminal G-node count (nodes with zero G-children).
    /// Maintained incrementally by split (+1 net) and evict (−1 or 0).
    pub(crate) terminal_count: u32,
    /// Live `D_evict` — adjusted by `adjust_depth_gates()` (ADR-M-017).
    /// Initialized to `config.depth_evict`.
    pub(crate) live_depth_evict: u32,
    /// Live `D_create` — adjusted by `adjust_depth_gates()` (ADR-M-017).
    /// Initialized to `config.depth_create`.
    pub(crate) live_depth_create: u32,
    /// Buffer zone width: `config.depth_evict − config.depth_create`.
    /// Computed once at construction; preserved across all adjustments.
    pub(crate) depth_buffer: u32,
    /// Headroom: `3^(buffer+1)`. Maximum entries immune to eviction
    /// at the depth floor (ADR-M-018).
    pub(crate) headroom: usize,
    /// Internal soft limit: `budget − max(headroom, 2*(D_c−1))`.
    /// Recomputed dynamically every `adjust_depth_gates()` call so
    /// that the invariant `S + 2*(D_c−1) ≤ H` always holds, where
    /// `H = budget` (ADR-M-018).  Panics if the value falls below 1.
    pub(crate) soft_limit: Option<usize>,

    /// Live plateau mirror of the G-Tree contour (ADR-M-026).
    ///
    /// Keyed by `BasisEdge` (P-I1 tiling). Maintained incrementally
    /// by observe, split, evict, and decay. Always up to date.
    #[cfg(feature = "dynamic-contour-tracking")]
    pub(crate) plateaus: BTreeMap<BasisEdge<C>, Plateau<C, V>>,

    /// Work-list of semi-internal basis elements that may violate
    /// P-I4 (thatch one-hop).  Populated by `place_basis_element`;
    /// drained by `repair_p_i4`.  Mirrors the `violations` queue
    /// pattern used for V-I3.
    #[cfg(feature = "dynamic-contour-tracking")]
    pub(crate) pending_p_i4: Vec<(GNodeId, BasisEdge<C>)>,

    /// Bidirectional basis ↔ plateau edge bookkeeping.
    ///
    /// Tracks which G-nodes are basis elements and which plateau
    /// they belong to. Invariant: `plateaus.keys()` and
    /// `plateau_basis.forward.keys()` are identical sets.
    #[cfg(feature = "dynamic-contour-tracking")]
    pub(crate) plateau_basis: PlateauBasis<C>,

    /// Dirty flag for `normalize_plateaus()` (ADR-M-031 Phase 4).
    ///
    /// **Set by** any path that may invalidate the plateau map:
    /// - `plateau_after_bootstrap_split` — contour refinement (§IDEA M-10.3)
    /// - `plateau_after_catalytic_split` — contour refinement (§IDEA M-10.2)
    /// - `plateau_after_legacy_promotes_batched` — contour restoration (§IDEA M-11.6)
    /// - `evict_candidates` — contour simplification (§IDEA M-12), when `evicted > 0`
    /// - `decay_uniform` / `decay_selective` — before their normalize calls
    ///
    /// **Checked by** `normalize_plateaus` — early return when `false`,
    /// then cleared before proceeding.
    #[cfg(feature = "dynamic-contour-tracking")]
    pub(crate) plateaus_dirty: bool,
}

impl<C: Coordinate, V: Accumulator, const N: u32> GvGraph<C, V, N> {
    /// Create a new `GvGraph` with the given configuration.
    ///
    /// Initializes both trees with a single shared G-node covering
    /// the full domain `[0, 2^N)` (§IDEA M-15).
    ///
    /// # Panics
    ///
    /// - If `N > C::BITS` (compile-time domain validation).
    /// - If `config.depth_create >= config.depth_evict` (D-I3).
    ///
    /// # Examples
    ///
    /// ```
    /// use torrust_mudlark::{GvGraph, Config};
    /// let cfg = Config {
    ///     split_threshold: 5u64,
    ///     depth_create: 3,
    ///     depth_evict: 6,
    ///     budget: None,
    ///     alpha_relax: 0.75,
    ///     bounded_eviction: true,
    /// };
    /// let g = GvGraph::<u64, u64, 8>::new(cfg);
    /// assert_eq!(g.node_count(), 1);
    /// assert_eq!(g.terminal_count(), 1);
    /// ```
    #[must_use]
    pub fn new(config: Config<V>) -> Self {
        // Compile-time domain validation (§IDEA M-15.1).
        const { assert!(N <= C::BITS, "N must be <= C::BITS") };

        config.validate();

        let mut gnodes = Arena::new();
        let mut vnodes = Arena::new();

        // Create the root G-node covering [0, 2^N).
        let root_gnode = GNode {
            lo: C::zero(),
            hi: C::domain_max(N),
            sum: V::zero(),
            own: V::zero(),
            left: None,
            right: None,
            parent: None,
            entry: None,
        };
        let g_root = GNodeId::from_index(gnodes.alloc(root_gnode));

        // Create the root V-entry for the G-root.
        let root_entry = VNode {
            intensity: V::zero(),
            parent: None,
            cached_depth: AtomicU32::new(0), // Root entry is at depth 0
            kind: crate::vnode::VKind::Entry {
                gnode: g_root,
                is_exposed: true,
                is_evictable: true,
            },
        };
        let v_root_id = VNodeId::from_index(vnodes.alloc(root_entry));
        gnodes.get_mut(g_root.index()).entry = Some(v_root_id);

        let live_depth_evict = config.depth_evict;
        let live_depth_create = config.depth_create;
        let depth_buffer = config.depth_evict - config.depth_create;
        let headroom = 3usize.pow(depth_buffer + 1);
        let convergence_bound = 2 * (live_depth_create as usize).saturating_sub(1);
        let required_headroom = headroom.max(convergence_bound);
        let soft_limit = config.budget.map(|b| {
            let s = b - required_headroom;
            assert!(s >= 1, "soft_limit must be >= 1 (budget={b}, headroom={required_headroom})");
            s
        });

        // Plateau mirror: root G-node is the sole terminal, forming
        // one plateau covering the full domain.
        #[cfg(feature = "dynamic-contour-tracking")]
        let (plateaus, plateau_basis) = {
            use crate::gtree::gnode_depth_from_interval;
            let root_key = BasisEdge(C::zero());
            let root_depth = gnode_depth_from_interval(C::zero(), C::domain_max(N), N);
            let mut pb = PlateauBasis::new();
            pb.insert(root_key, g_root);
            let mut map = BTreeMap::new();
            map.insert(
                root_key,
                Plateau {
                    basis_edge: root_key,
                    start: C::zero(),
                    end: C::domain_max(N),
                    depth: root_depth,
                    sum: V::zero(),
                },
            );
            (map, pb)
        };

        Self {
            gnodes,
            vnodes,
            g_root,
            v_root: Some(v_root_id),
            config,
            violations: Vec::new(),
            node_count: 1,
            terminal_count: 1,
            live_depth_evict,
            live_depth_create,
            depth_buffer,
            headroom,
            soft_limit,
            #[cfg(feature = "dynamic-contour-tracking")]
            plateaus,
            #[cfg(feature = "dynamic-contour-tracking")]
            pending_p_i4: Vec::new(),
            #[cfg(feature = "dynamic-contour-tracking")]
            plateau_basis,
            #[cfg(feature = "dynamic-contour-tracking")]
            plateaus_dirty: false,
        }
    }

    /// Total number of live G-nodes.
    ///
    /// Starts at 1 (the root) and increases by 1 for each split.
    /// Decreases by 1 for each eviction. O(1).
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
    /// // A fresh graph has exactly one node (the root).
    /// assert_eq!(g.node_count(), 1);
    ///
    /// // After enough observations to trigger a split the count grows.
    /// for _ in 0..10 {
    ///     g.observe(42, 1u64);
    /// }
    /// assert!(g.node_count() > 1);
    /// ```
    #[must_use]
    #[inline]
    pub const fn node_count(&self) -> u32 {
        self.node_count
    }

    /// Number of terminal G-nodes (leaves with zero G-children).
    ///
    /// Maintained incrementally: +1 net per split, −1 per eviction
    /// (unless the parent becomes terminal, in which case net 0).
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
    /// // The root is the sole terminal in a fresh graph.
    /// assert_eq!(g.terminal_count(), 1);
    ///
    /// // Splits increase the terminal count.
    /// for _ in 0..10 {
    ///     g.observe(42, 1u64);
    /// }
    /// assert!(g.terminal_count() >= 2);
    /// ```
    #[must_use]
    #[inline]
    pub const fn terminal_count(&self) -> u32 {
        self.terminal_count
    }

    // Plateau tracking methods (plateaus, build_plateaus, plateau_basis,
    // debug_plateau_basis, recompute_plateau, place_basis_element,
    // place_subtree_basis_elements, consolidate_basis_up, normalize_plateaus,
    // consolidate_all_basis, debug_check_plateau_sums,
    // fixup_plateau, repair_p_i4, split_for_p_i4, find_boundary_node,
    // plateau_after_observe, plateau_after_bootstrap_split,
    // plateau_after_catalytic_split, plateau_after_legacy_promote,
    // plateau_recompute_sums) live in graph_plateau.rs — see ADR-M-030 Phase B.

    /// Reference to the current configuration.
    ///
    /// Returns the [`Config`] supplied at construction time.
    /// Useful for inspecting thresholds and depth gates from
    /// generic code that holds only a `&GvGraph`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// # let config = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let g = GvGraph::<u64, u64, 8>::new(config);
    /// assert_eq!(g.config().split_threshold, 5u64);
    /// assert_eq!(g.config().depth_create, 3);
    /// assert_eq!(g.config().depth_evict, 6);
    /// ```
    #[must_use]
    #[inline]
    pub const fn config(&self) -> &Config<V> {
        &self.config
    }

    /// The configured hard node-count ceiling, or `None` if unbounded.
    ///
    /// Shorthand for `self.config().budget`. Useful in generic code
    /// that only needs to inspect the budget without borrowing the
    /// full `Config`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// // Unbounded graph.
    /// # let config = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let g = GvGraph::<u64, u64, 8>::new(config);
    /// assert_eq!(g.budget(), None);
    ///
    /// // Budgeted graph.
    /// # let config2 = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: Some(500),
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let g2 = GvGraph::<u64, u64, 8>::new(config2);
    /// assert_eq!(g2.budget(), Some(500));
    /// ```
    #[must_use]
    #[inline]
    pub const fn budget(&self) -> Option<usize> {
        self.config.budget
    }

    // Budget enforcement, depth-gate adjustment, eviction orchestration,
    // and handle_legacy_promotes live in graph_budget.rs — see ADR-M-030
    // Phase E.

    /// The G-Tree root handle (always valid).
    ///
    /// Returns the [`GNodeId`] of the root G-node, which covers the
    /// entire domain `[0, 2^N)`. This handle never changes over the
    /// lifetime of the graph.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph, GNodeId};
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let g = GvGraph::<u64, u64, 8>::new(cfg);
    /// let root = g.g_root();
    /// assert_eq!(root, GNodeId::from_index(0));
    /// ```
    #[must_use]
    #[inline]
    pub const fn g_root(&self) -> GNodeId {
        self.g_root
    }

    /// The V-Tree root handle (diagnostic / testing affordance).
    ///
    /// Always `Some` after construction — the root V-entry is
    /// created alongside the root G-node by [`GvGraph::new`].
    ///
    /// `VNodeId` has no public consuming methods — this accessor
    /// exists for diagnostic and crate-level testing only
    /// (ADR-M-032 surface assignment test, handle test).
    #[must_use]
    #[inline]
    pub(crate) const fn v_root(&self) -> Option<VNodeId> {
        self.v_root
    }

    /// Total accumulated value across the entire domain.
    ///
    /// Equivalent to the G-root's `sum` field — the aggregate of
    /// every observation ever applied to the graph.  O(1).
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
    /// assert_eq!(g.total_sum(), 0u64);
    ///
    /// g.observe(10, 3u64);
    /// g.observe(20, 7u64);
    /// assert_eq!(g.total_sum(), 10u64);
    /// ```
    // NOTE: not `const fn` because `Arena::get` indexes into a `Vec`,
    // and `Vec::as_slice()` only became const in Rust 1.86.  Promote
    // to `const fn` once the workspace MSRV reaches 1.86+.
    #[must_use]
    #[inline]
    pub fn total_sum(&self) -> V {
        self.gnodes.get(self.g_root.index()).sum
    }

    /// Read-only access to the G-node arena.
    ///
    /// `pub(crate)` — Surface 3 (Emulsion). External consumers use
    /// the public query surface (`get`, `range_sum`, `plateaus`,
    /// `sample`, `extract`, `layers`) instead.
    #[must_use]
    #[inline]
    pub(crate) const fn gnodes(&self) -> &Arena<GNode<C, V>> {
        &self.gnodes
    }

    /// Read-only access to the V-node arena.
    ///
    /// `pub(crate)` — Surface 3 (Emulsion).
    #[must_use]
    #[inline]
    pub(crate) const fn vnodes(&self) -> &Arena<VNode<V>> {
        &self.vnodes
    }

    /// Whether there are unresolved violations in the work queue.
    ///
    /// After any public mutation ([`observe`](Self::observe),
    /// [`decay`](Self::decay), [`check_evictions`](Self::check_evictions))
    /// returns, the violation queue is always fully drained — so
    /// this returns `false` in normal use.  It exists for diagnostics
    /// and invariant assertions.
    ///
    /// `pub(crate)` — Surface 3 (Emulsion). Test-only.
    #[cfg(test)]
    #[must_use]
    #[inline]
    pub(crate) const fn has_pending_violations(&self) -> bool {
        !self.violations.is_empty()
    }

    /// Current (dynamic) `D_evict` depth gate
    ///
    /// Entries deeper than `D_evict` in the V-Tree are eligible for
    /// eviction.  Starts at [`Config::depth_evict`] and adjusts
    /// dynamically when a budget is set (ADR-M-017).
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
    /// # let g = GvGraph::<u64, u64, 8>::new(cfg);
    /// assert_eq!(g.depth_evict(), 6);
    /// ```
    #[must_use]
    #[inline]
    pub const fn depth_evict(&self) -> u32 {
        self.live_depth_evict
    }

    /// Current (dynamic) `D_create` depth gate
    ///
    /// Splits are only authorized when the target entry's V-depth
    /// is at most `D_create`.  Starts at [`Config::depth_create`]
    /// and moves in lockstep with `D_evict` to preserve the buffer
    /// zone (ADR-M-017).
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
    /// # let g = GvGraph::<u64, u64, 8>::new(cfg);
    /// assert_eq!(g.depth_create(), 3);
    /// assert!(g.depth_create() < g.depth_evict());
    /// ```
    #[must_use]
    #[inline]
    pub const fn depth_create(&self) -> u32 {
        self.live_depth_create
    }

    /// Buffer zone width (`D_evict − D_create`), fixed at construction
    ///
    /// The buffer zone separates the creation and eviction gates so
    /// that newly-split entries have room to prove their significance
    /// before becoming eviction-eligible.
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
    /// # let g = GvGraph::<u64, u64, 8>::new(cfg);
    /// // buffer = D_evict − D_create = 6 − 3 = 3.
    /// assert_eq!(g.depth_buffer(), 3);
    /// assert_eq!(g.depth_buffer(), g.depth_evict() - g.depth_create());
    /// ```
    #[must_use]
    #[inline]
    pub const fn depth_buffer(&self) -> u32 {
        self.depth_buffer
    }

    /// Headroom: $3^{(\text{buffer}+1)}$ — worst-case entries immune
    /// to eviction at the depth floor (ADR-M-018)
    ///
    /// With V-Tree branching factor up to 3, at most
    /// $3^{(\text{buffer}+1)}$ entries can reside between the root
    /// and `D_evict`.  These entries are not evictable, so any
    /// configured budget must exceed this value.
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
    /// # let g = GvGraph::<u64, u64, 8>::new(cfg);
    /// // buffer = 3, so headroom = 3^(3+1) = 81.
    /// assert_eq!(g.headroom(), 81);
    /// assert_eq!(g.headroom(), 3usize.pow(g.depth_buffer() + 1));
    /// ```
    #[must_use]
    #[inline]
    pub const fn headroom(&self) -> usize {
        self.headroom
    }

    /// Internal soft limit, or `None` if no budget
    ///
    #[allow(clippy::doc_markdown)]
    /// $S = \text{budget} - \max(\text{headroom},\; 2(D_c - 1))$
    ///
    /// When `node_count` exceeds the soft limit, depth-gate
    /// tightening and eviction are triggered.  Recomputed
    /// dynamically as `D_create` adjusts (ADR-M-018).
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// // Unbounded — no soft limit.
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let g = GvGraph::<u64, u64, 8>::new(cfg);
    /// assert_eq!(g.soft_limit(), None);
    ///
    /// // Budgeted — soft limit is budget minus headroom.
    /// # let cfg2 = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: Some(500),
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let g2 = GvGraph::<u64, u64, 8>::new(cfg2);
    /// let s = g2.soft_limit().unwrap();
    /// assert!(s < 500);
    /// assert_eq!(s, 500 - g2.headroom()); // headroom = 81 > 2*(3−1)
    /// ```
    #[must_use]
    #[inline]
    pub const fn soft_limit(&self) -> Option<usize> {
        self.soft_limit
    }

    // ── Shared helpers (Phase 4) ────────────────────────────────

    /// Compute the G-Tree depth of a node by its interval width.
    ///
    /// Thin wrapper around [`gtree::gnode_depth_from_interval`] that
    /// captures the const-generic `N`.
    #[must_use]
    #[inline]
    #[allow(dead_code)] // used by Phase 4 steps 3 and 5 (extraction, decay)
    pub(crate) fn gnode_depth(&self, gid: GNodeId) -> u32 {
        let g = self.gnodes.get(gid.index());
        crate::gtree::gnode_depth_from_interval(g.lo, g.hi, N)
    }

    // Sampling, point query, and range query methods live in
    // graph_query.rs — see ADR-M-030 Phase C.

    // PEWEI extraction, layer iteration, from_observations, and Extend
    // live in graph_extract.rs — see ADR-M-030 Phase D.

    // ── Sentinel integration API (ADR-M-036) ──────────────────────

    /// Read-only snapshot of a G-node's spatial extent and accumulation.
    ///
    /// Returns `None` if `id` does not refer to a live G-node.
    ///
    /// The returned [`Node`] is a `Copy` snapshot — it does not
    /// borrow the graph. For mutable child linkage (left, right), see
    /// [`gnode_children()`](Self::gnode_children).
    ///
    /// # Cost
    ///
    /// $O(1)$ — single arena lookup plus depth computation.
    ///
    /// # Examples
    ///
    /// ```
    /// use torrust_mudlark::{Config, GvGraph, GState};
    ///
    /// let cfg = Config {
    ///     split_threshold: 5u64,
    ///     depth_create: 3,
    ///     depth_evict: 6,
    ///     budget: None,
    ///     alpha_relax: 0.75,
    ///     bounded_eviction: true,
    /// };
    /// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// g.observe(42, 10u64);
    ///
    /// // Root is always live.
    /// let info = g.gnode_info(g.g_root()).unwrap();
    /// assert_eq!(info.start, 0);
    /// assert_eq!(info.end, 256);
    /// ```
    #[must_use]
    pub fn gnode_info(&self, id: GNodeId) -> Option<Node<C, V>> {
        if !self.gnodes.is_occupied(id.index()) {
            return None;
        }
        let g = self.gnodes.get(id.index());
        Some(Node {
            start: g.lo,
            end: g.hi,
            own: g.own,
            sum: g.sum,
            depth: crate::gtree::gnode_depth_from_interval(g.lo, g.hi, N),
            state: g.state(),
            gnode_id: id,
            parent: g.parent,
        })
    }

    /// Current child handles of a G-node (left, right).
    ///
    /// Returns `None` if `id` does not refer to a live G-node.
    /// Handles are live at snapshot time but may become stale after
    /// `observe()` or `decay()`.
    ///
    /// For the stable parent pointer, see
    /// [`gnode_info()`](Self::gnode_info) → [`Node::parent`](crate::Node::parent).
    ///
    /// # Cost
    ///
    /// $O(1)$ — single arena lookup.
    ///
    /// # Examples
    ///
    /// Walk from the root to collect all live G-node handles:
    ///
    /// ```
    /// use torrust_mudlark::{Config, GvGraph};
    ///
    /// let cfg = Config {
    ///     split_threshold: 5u64,
    ///     depth_create: 3,
    ///     depth_evict: 6,
    ///     budget: None,
    ///     alpha_relax: 0.75,
    ///     bounded_eviction: true,
    /// };
    /// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// g.observe(42, 10u64);
    ///
    /// // DFS walk collecting all G-node handles.
    /// let mut stack = vec![g.g_root()];
    /// let mut visited = Vec::new();
    /// while let Some(id) = stack.pop() {
    ///     visited.push(id);
    ///     if let Some(ch) = g.gnode_children(id) {
    ///         stack.extend(ch.left);
    ///         stack.extend(ch.right);
    ///     }
    /// }
    /// // Root split ⇒ at least three nodes.
    /// assert!(visited.len() >= 3);
    /// ```
    #[doc(hidden)]
    #[must_use]
    pub fn gnode_children(&self, id: GNodeId) -> Option<GNodeChildren> {
        if !self.gnodes.is_occupied(id.index()) {
            return None;
        }
        let g = self.gnodes.get(id.index());
        Some(GNodeChildren {
            left: g.left,
            right: g.right,
        })
    }

    /// Whether `ancestor` is a proper ancestor of `descendant` in
    /// the G-Tree (i.e., `descendant` is in `ancestor`'s subtree
    /// and `ancestor ≠ descendant`).
    ///
    /// Uses interval containment: `ancestor` is an ancestor of
    /// `descendant` iff `ancestor.lo <= descendant.lo` and
    /// `ancestor.hi >= descendant.hi` and the intervals are not
    /// equal.
    ///
    /// Returns `false` if either handle is not live.
    ///
    /// # Cost
    ///
    /// $O(1)$ — two arena lookups plus interval comparison.
    ///
    /// # Examples
    ///
    /// ```
    /// use torrust_mudlark::{Config, GvGraph};
    ///
    /// let cfg = Config {
    ///     split_threshold: 5u64,
    ///     depth_create: 3,
    ///     depth_evict: 6,
    ///     budget: None,
    ///     alpha_relax: 0.75,
    ///     bounded_eviction: true,
    /// };
    /// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// g.observe(42, 10u64);
    ///
    /// let root = g.g_root();
    /// // Root is ancestor of every other node.
    /// for (_layer, node) in g.layers() {
    ///     if node.gnode_id != root {
    ///         assert!(g.is_ancestor_of(root, node.gnode_id));
    ///     }
    /// }
    /// // A node is not its own proper ancestor.
    /// assert!(!g.is_ancestor_of(root, root));
    /// ```
    #[must_use]
    pub fn is_ancestor_of(&self, ancestor: GNodeId, descendant: GNodeId) -> bool {
        if !self.gnodes.is_occupied(ancestor.index()) || !self.gnodes.is_occupied(descendant.index()) {
            return false;
        }
        let a = self.gnodes.get(ancestor.index());
        let d = self.gnodes.get(descendant.index());
        a.lo <= d.lo && a.hi >= d.hi && (a.lo != d.lo || a.hi != d.hi)
    }
}

// SpatialRead, SpatialWrite, and WeightedSampler trait impls live in
// graph_traits.rs — see ADR-M-030 Phase F.
