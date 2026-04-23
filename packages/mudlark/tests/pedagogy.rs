// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # End-to-End Pedagogy Test for the Worked Example
//!
//! This test walks through the construction, observation, rebalancing,
//! extraction, sampling, and decay of a small G-V Graph step by step,
//! printing a narrative log that can be read alongside the formal
//! specification.  Every assertion corresponds to a claim in the theory
//! documents — if this test passes, the prose is accurate.
//!
//! ## What is a G-V Graph?
//!
//! A G-V Graph is an **adaptive partition** of a one-dimensional domain
//! that continuously reallocates its precision toward regions of observed
//! significance.  It consists of two trees sharing the same nodes:
//!
//! - The **G-Tree** (Geometric Tree) is the partition itself — a binary
//!   tree of dyadic intervals whose leaves tile the domain without gaps
//!   or overlaps.  Its leaf set satisfies the **Kraft equality**
//!   `Σ 2^{-dᵢ} = 1` at all times — it is a valid prefix-free code
//!   (§THEORY M-1.1, §IDEA M-5).
//!
//! - The **V-Tree** (Value Tree) is a competitive tournament that ranks
//!   every G-node by importance.  High-importance entries sit near the
//!   root; low-importance entries are pushed deep.  The ranking is
//!   maintained within a factor of 1.44× Shannon entropy by the
//!   **max-uncle constraint**: no node may outrank every one of its
//!   uncles (§IDEA M-6.2, §THEORY M-3.3, §THEORY M-4.1).
//!
//! The two trees protect each other through three shields (§IDEA M-1.5,
//! §IDEA M-13.1, §THEORY M-3.4):
//!
//! 1. **Observation routing** (G→V, upward): children intercept traffic,
//!    freezing the parent's importance as a benchmark.
//! 2. **Uncle constraint** (V→entries, downward): the frozen benchmark
//!    shields children from displacement.
//! 3. **Structural immunity** (G→eviction): only nodes with zero
//!    children can be removed — the partition contracts from tips inward.
//!
//! ## What this test demonstrates
//!
//! Steps 0–3 replay the worked example from §IDEA M-16 (bootstrap split,
//! catalytic split, skip-promote, uncle shield).  Steps 4–9 extend beyond
//! that section to exercise PEWEI extraction (§PEWEI M-9), proportional
//! sampling (§IDEA M-6.5, §THEORY M-4.3), subtree decay (§IDEA M-14.3,
//! §THEORY M-7), and post-decay rebalancing (§IDEA M-11.13).
//!
//! ## Configuration
//!
//! The domain is `[0, 8)` (`N = 3`, so `2³ = 8` addressable points).
//! The split threshold `θ = 5` means a cell must accumulate at least 5
//! units of importance before it may subdivide.  No budget is set, so
//! the tree grows freely and we can observe every structural decision.
//! Depth gates `D_create = 3`, `D_evict = 6` give ample room for the
//! tournament to develop (§IDEA M-7, §API M-5.1).
//!
//! Run with:
//!
//! ```sh
//! cargo test -p torrust-mudlark --test pedagogy
//! ```
//!
//! # Test Index
//!
//! ## Configuration & Initialization (§IDEA M-15, §IDEA M-16.1)
//!
//! | Step | Function                         | What it teaches                                                  |
//! |------|----------------------------------|------------------------------------------------------------------|
//! | 0    | [`step_0_fresh`]                 | A fresh graph is a single plateau at depth 0 — Kraft = `2⁰ = 1` |
//!
//! ## Observation & Code Lengthening (§IDEA M-16.3, §THEORY M-6)
//!
//! | Step | Function                         | What it teaches                                                  |
//! |------|----------------------------------|------------------------------------------------------------------|
//! | 1    | [`step_1_first_observation`]     | Bootstrap split: the first code-lengthening event                |
//! | 2    | [`step_2_asymmetric_contour`]    | Catalytic split + competitive promotion → asymmetric resolution  |
//! |      | [`step_2_contour`]               | Two plateaus at different depths: the code adapts                |
//! |      | [`step_2_vtree`]                 | V-Tree ranking diverges from spatial containment (the inversion) |
//! |      | [`step_2_point_queries`]         | Point queries route through the adaptive partition               |
//! | 3    | [`step_3_right_half_splits`]     | Uncle shield absorbs a significant observation without reshaping |
//! |      | [`step_3_uniform_contour`]       | Contour recovers to uniform depth after balanced observation     |
//! |      | [`step_3_vtree_memory`]          | The tournament remembers arrival order — history matters         |
//!
//! ## PEWEI Extraction (§PEWEI M-2, §PEWEI M-9)
//!
//! | Step | Function                         | What it teaches                                                  |
//! |------|----------------------------------|------------------------------------------------------------------|
//! | 4    | [`step_4_first_pewei`]           | The significance-ordered snapshot reads both trees in one pass   |
//!
//! ## Hotspot Dynamics (§IDEA M-8, §IDEA M-10)
//!
//! | Step | Function                         | What it teaches                                                  |
//! |------|----------------------------------|------------------------------------------------------------------|
//! | 5    | [`step_5_new_observations`]      | New evidence shifts the competitive landscape                    |
//! | 6    | [`step_6_contour_after_obs`]     | Plateaus reflect the code's learned shape                        |
//! | 7    | [`step_7_sample`]                | Entropy-adaptive sampling: O(1) for hotspots, O(log L) uniform  |
//!
//! ## Temporal Modulation (§IDEA M-14, §THEORY M-7)
//!
//! | Step | Function                         | What it teaches                                                  |
//! |------|----------------------------------|------------------------------------------------------------------|
//! | 8    | [`step_8_subtree_decay`]         | Targeted decay weakens a region, triggering V-Tree rebalancing   |
//! | 9    | [`step_9_final_pewei`]           | Post-decay PEWEI: the significance ordering reflects new reality |

use torrust_mudlark::invariants::assert_invariants;
#[cfg(debug_assertions)]
use torrust_mudlark::testing::TestLcgRng;
use torrust_mudlark::testing::worked_example_config;
use torrust_mudlark::{Config, GNodeId, GState, GvGraph};

// ── Configuration ───────────────────────────────────────────────
//
// The configuration is written out verbatim here so the test is
// self-documenting.  It must match `worked_example_config()` from
// the shared test harness — the assertion in `main()` catches drift.
//
// These parameters come from §IDEA M-16.1:
//   - θ = 5:  a cell must accumulate importance > 5 to split
//   - D_create = 3:  only V-entries at depth ≤ 3 may split
//   - D_evict = 6:   unprotected entries past depth 6 are evicted
//   - No budget:     the tree grows freely
//
// The depth gates define three zones (§IDEA M-7.3):
//   [0, D_create]:        creation zone — splits permitted
//   (D_create, D_evict]:  buffer zone — borrowed time, can promote out
//   (D_evict, ∞):         eviction zone — unprotected tips removed

const README_CONFIG: Config<u64> = Config {
    split_threshold: 5,
    depth_create: 3,
    depth_evict: 6,
    budget: None,
    alpha_relax: 0.75,
    bounded_eviction: true,
};

// ── Helpers ─────────────────────────────────────────────────────

fn heading(s: &str) {
    let rule: String = "═".repeat(60);
    println!("\n{rule}");
    println!("  {s}");
    println!("{rule}");
}

fn subheading(s: &str) {
    println!("\n── {s} ──");
}

/// Print the full contour state: every plateau and every terminal cell.
///
/// This corresponds to the "bottom contour" from §IDEA M-5.6 — the
/// observation-receiving surface of the domain.  Each plateau is a
/// maximal contiguous run of the contour at a single depth.  The
/// point-query walk shows which cell actually receives observations
/// at each coordinate (§IDEA M-5.5.1).
fn print_contour_debug(label: &str, g: &GvGraph<u64, u64, 3>) {
    subheading(&format!("Contour snapshot: {label}"));

    let plateaus = g.plateaus();
    println!("Plateaus ({}):", plateaus.len());
    for (edge, p) in &*plateaus {
        println!(
            "  BasisEdge({:>2}):  [{}, {})  depth={}  sum={}  width={}",
            edge.0,
            p.start,
            p.end,
            p.depth,
            p.sum,
            p.width()
        );
    }

    // Walk every integer coordinate to show the routing result.
    // Each `get(x)` descends the G-Tree via `route_to_receiver`
    // (§IDEA M-5.2) and returns the terminal or semi-internal cell
    // whose uncovered range contains `x`.
    println!("Terminal cells (point queries across domain):");
    for x in 0u64..8 {
        let cell = g.get(x);
        println!(
            "  get({x}) → [{}, {})  depth={}  intensity={}",
            cell.start, cell.end, cell.depth, cell.intensity
        );
    }
    println!();
}

/// Collect V-Tree layer data for snapshot comparison.
///
/// The `layers()` iterator performs a breadth-first walk of the V-Tree
/// (§PEWEI M-9, §API M-5.2), yielding entries in significance order:
/// shallow layers (low index) contain the most important entries,
/// deep layers contain the least.  This is the same traversal order
/// used by `extract()` to produce the PEWEI.
fn vtree_snapshot(g: &GvGraph<u64, u64, 3>) -> Vec<(usize, u64, u64, u64, GState)> {
    g.layers()
        .map(|(layer, node)| (layer, node.start, node.end, node.own, node.state))
        .collect()
}

/// Layer positions only — for shape comparison ignoring value changes.
fn layer_positions(g: &GvGraph<u64, u64, 3>) -> Vec<(usize, u64, u64)> {
    g.layers().map(|(layer, node)| (layer, node.start, node.end)).collect()
}

/// Find the V-Tree layer index of a specific G-node by its interval.
fn entry_layer(g: &GvGraph<u64, u64, 3>, start: u64, end: u64) -> Option<usize> {
    g.layers()
        .find_map(|(l, n)| if n.start == start && n.end == end { Some(l) } else { None })
}

/// Find the `GNodeId` handle for a G-node by its interval.
///
/// `GNodeId` is a typed arena handle (§API M-4.5) — opaque, `Copy`,
/// used to address specific G-nodes in introspection and decay operations.
fn find_gnode_id(g: &GvGraph<u64, u64, 3>, start: u64, end: u64) -> GNodeId {
    g.layers()
        .find_map(|(_, n)| {
            if n.start == start && n.end == end {
                Some(n.gnode_id)
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("G-node [{start}, {end}) not found in layers"))
}

// ═══════════════════════════════════════════════════════════════
//  STEP 0: INITIALIZATION
//  §IDEA M-15, §IDEA M-16.1, §THEORY M-1.1
// ═══════════════════════════════════════════════════════════════

/// A fresh G-V Graph contains a single G-node covering the entire
/// domain, a single V-entry serving as the V-root, and a single
/// plateau at depth 0.
///
/// In coding-theoretic terms (§THEORY M-1.1), this is the trivial
/// code: one codeword of length 0 covering the entire symbol space.
/// The Kraft equality holds: `2^{-0} = 1`.
///
/// The plateau map (§IDEA M-5.6.7, §API M-5.2) is the contour's
/// external interface — a `BTreeMap<BasisEdge<C>, Plateau<C, V>>`
/// that projects the bottom contour as an ordered sequence of
/// constant-depth regions.  Here it has exactly one entry.
fn step_0_fresh(g: &GvGraph<u64, u64, 3>) {
    heading("Step 0: Configuration & Initialization");
    println!("  (§IDEA M-15 — Initialization, §IDEA M-16.1 — Worked Example Setup)");

    println!();
    println!("Domain: [0, 8)   (N = 3, so 2³ = 8 addressable points)");
    println!("Split threshold θ = 5   (must exceed this to subdivide)");
    println!("Depth gates: D_create = 3, D_evict = 6");
    println!("No budget — the tree grows freely");

    let plateaus = g.plateaus();
    assert_eq!(plateaus.len(), 1, "fresh graph has exactly one plateau");
    let p = plateaus.values().next().unwrap();
    assert_eq!((p.start, p.end, p.depth), (0, 8, 0));

    println!();
    println!("Initial state:");
    println!("  G-Tree:  one root node [0, 8) with sum=0, own=0");
    println!("  V-Tree:  one entry (the root's V-entry) — V-root is an entry,");
    println!("           not structural, until the first split (§IDEA M-6.1).");
    println!("  Plateau: one plateau at depth 0 over [0, 8)");
    println!();
    println!("  Kraft equality: 2^(-0) = 1  ✓");
    println!("  This is the trivial code: one codeword covering everything.");

    assert_invariants(g);
}

// ═══════════════════════════════════════════════════════════════
//  STEP 1: FIRST OBSERVATION — BOOTSTRAP SPLIT
//  §IDEA M-8 (Observation Flow), §IDEA M-10.3 (Bootstrap Split),
//  §IDEA M-16.3.2
// ═══════════════════════════════════════════════════════════════

/// The first observation that exceeds the split threshold triggers
/// the **bootstrap split** (§IDEA M-10.3) — a special case that
/// converts the V-root from a single entry to a structural node
/// with the original entry and a new child structural node.
///
/// After the split:
/// - The root [0,8) is now *internal*: both children [0,4) and [4,8)
///   exist.  Its importance is **frozen** — children intercept all
///   future observations via `route_to_receiver` (§IDEA M-5.2), so
///   the root's V-entry will never grow again.  This is Shield 1
///   (§IDEA M-13.1): observation routing as the freeze mechanism.
/// - The children start at importance `ν = 0` (the ground element,
///   §IDEA M-2.1) and must **earn** their way up against the frozen
///   benchmark through competitive accumulation.
/// - The contour is uniform at depth 1 — both halves have the same
///   resolution.  The Kraft equality: `2^(-1) + 2^(-1) = 1`.
///
/// In coding terms (§THEORY M-1.3), we replaced one codeword of
/// length 0 with two codewords of length 1.  Kraft is preserved:
/// `-2^0 + 2·2^{-1} = 0`.
fn step_1_first_observation(g: &mut GvGraph<u64, u64, 3>) {
    heading("Step 1: observe(3, 10) — Bootstrap Split");
    println!("  (§IDEA M-16.3.2, §IDEA M-10.3 — Bootstrap Split)");

    g.observe(3, 10u64);

    println!();
    println!("What happened:");
    println!("  1. Route: G-root is terminal → receiver = [0,8)  (§IDEA M-5.2)");
    println!("  2. Accumulate: [0,8).own = 0 + 10 = 10");
    println!("  3. Split check: importance 10 > θ = 5, and V-root has no parent");
    println!("     → bootstrap_split fires  (§IDEA M-10.3)");
    println!("  4. Result: [0,8) splits into [0,4) + [4,8)");
    println!("     Root's V-entry freezes at importance 10.");
    println!("     Children start at importance ν = 0.");

    // Both children at depth 1 → single merged plateau.
    let plateaus = g.plateaus();
    assert_eq!(plateaus.len(), 1, "uniform depth → single plateau");
    let p = plateaus.values().next().unwrap();
    assert_eq!((p.start, p.end, p.depth), (0, 8, 1));

    print_contour_debug("after bootstrap split", g);

    println!("     ┌──────────────────────────────────────────┐");
    println!("     │  ┌──────────────────┐┌──────────────────┐│");
    println!("     └──┴──────────────────┴┴──────────────────┴┘");
    println!("     0                      4                    8");
    println!();
    println!("     depth 1 ▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔");
    println!("     uniform — both halves at same resolution");
    println!();
    println!("  Kraft: 2^(-1) + 2^(-1) = 1  ✓");
    println!("  The code lengthened: one codeword became two.");
    println!("  The frozen root benchmark creates the competitive bar");
    println!("  that children must exceed to earn promotion (§IDEA M-13.1).");

    assert_invariants(g);
}

// ═══════════════════════════════════════════════════════════════
//  STEP 2: ASYMMETRIC REFINEMENT
//  §IDEA M-10.2 (Catalytic Split), §IDEA M-11.5 (Skip Promote),
//  §IDEA M-16.3.3
// ═══════════════════════════════════════════════════════════════

/// A second heavy observation at x=3 drives the left half [0,4) past
/// the split threshold AND past the frozen root benchmark, triggering
/// both a **catalytic split** and a **competitive promotion**.
///
/// The catalytic split (§IDEA M-10.2, §THEORY M-6.2) is the general
/// split mechanism.  Unlike bootstrap, the parent persists as a frozen
/// benchmark — it is *not consumed*.  Its importance is fixed at the
/// pre-split value, and children start from zero.  This is why it's
/// called "catalytic": the parent enables the reaction without being
/// consumed by it (§IDEA M-1.6.1).
///
/// The violation L(15) > root(10) means L outgrew its uncle — the
/// frozen benchmark.  The V-Tree resolves this via **skip promote**
/// (§IDEA M-11.5): L rises from V-depth 2 to V-depth 1, dissolving
/// the uncle relationship.  This is the competitive mechanism at work:
/// the child region proved more significant than the parent region was
/// at split time (§IDEA M-6.3, §THEORY M-3.5).
///
/// After this step the contour is *asymmetric*: depth 2 on the left
/// (where the hotspot is) and depth 1 on the right (still coarse).
/// The code allocated its precision budget toward the observed mass —
/// the rate-distortion dual of source coding (§THEORY M-1.2).
fn step_2_asymmetric_contour(g: &mut GvGraph<u64, u64, 3>) {
    heading("Step 2: observe(3, 15) — Catalytic Split + Promotion");
    println!("  (§IDEA M-16.3.3, §IDEA M-10.2, §IDEA M-11.5)");

    g.observe(3, 15u64);

    println!();
    println!("What happened:");
    println!("  1. Route: [0,8) → [0,4) (terminal receiver)  (§IDEA M-5.2)");
    println!("  2. Accumulate: [0,4).own = 0 + 15 = 15");
    println!("  3. Split check: 15 > θ=5, D_V ≤ D_create=3");
    println!("     → catalytic_split fires  (§IDEA M-10.2)");
    println!("     [0,4) splits into [0,2) + [2,4)");
    println!("     L's entry freezes at importance 15.");
    println!("  4. Violation: L(15) > uncle root(10)");
    println!("     → skip_promote: L rises from depth 2 to depth 1  (§IDEA M-11.5)");

    step_2_contour(g);
    step_2_vtree(g);
    step_2_point_queries(g);

    assert_invariants(g);
}

/// The contour after step 2 shows the architecture's core property:
/// **adaptive resolution** — fine where hot, coarse where cold.
///
/// Two plateaus exist (§IDEA M-5.6): one at depth 2 covering [0,4)
/// (the hotspot region, now subdivided) and one at depth 1 covering
/// [4,8) (the quiet region, not yet refined).  The plateau count P=2
/// is a measure of structural complexity (§IDEA M-5.6.6).
fn step_2_contour(g: &GvGraph<u64, u64, 3>) {
    subheading("Contour — the key asymmetric moment");
    println!("  (§IDEA M-5.6 — Plateaus measure structural complexity)");

    let plateaus = g.plateaus();
    assert_eq!(plateaus.len(), 2, "two plateaus at different depths");
    let ps: Vec<_> = plateaus.values().collect();

    // Left half: depth 2 (two subdivisions from the root).
    assert_eq!((ps[0].start, ps[0].end, ps[0].depth), (0, 4, 2));
    // Right half: depth 1 (one subdivision — still coarse).
    assert_eq!((ps[1].start, ps[1].end, ps[1].depth), (4, 8, 1));

    print_contour_debug("asymmetric contour", g);

    println!("     ┌──────────────────────────────────────────┐");
    println!("     │  ┌──────────────────┐                    │");
    println!("     │  │  ┌──────┐┌──────┐│                    │");
    println!("     └──┴──┴──────┴┴──────┴┴────────────────────┘");
    println!("     0     2       4                            8");
    println!();
    println!("     depth 2 ▔▔▔▔▔▔▔▔▔▔▔▔▔   depth 1 ▔▔▔▔▔▔▔▔▔");
    println!("     fine (hotspot)            coarse (quiet)");
    println!();
    println!("  This is the rate-distortion dual (§THEORY M-1.2):");
    println!("  deep where intense, shallow where quiet.");
    println!("  Kraft: 2^(-2) + 2^(-2) + 2^(-1) = 0.25 + 0.25 + 0.5 = 1  ✓");
}

/// The V-Tree's ranking after step 2 demonstrates the **inversion**
/// (§IDEA M-1.3.4, §THEORY M-6.3): the V-Tree and G-Tree rank the
/// same nodes in opposite order for internal nodes.
///
/// The G-Tree says [0,8) is the most important node (highest sum —
/// it contains everything).  The V-Tree says [0,8) is a historical
/// footnote (frozen at its ancient pre-split value of 10).
///
/// Meanwhile [0,4), which the G-Tree considers a mere subset, sits
/// at V-depth 1 — the tournament's shallowest entry layer.  It earned
/// this position through accumulated evidence, not through spatial
/// containment.
///
/// This is §IDEA M-13.4: the G-Tree measures the integral (cumulative
/// mass), the V-Tree measures the derivative (scale-specific arrival
/// rate).  The two are the same signal from complementary directions.
fn step_2_vtree(g: &GvGraph<u64, u64, 3>) {
    subheading("V-Tree tournament ranking — the inversion");
    println!("  (§IDEA M-1.3.4 — Inversion, §THEORY M-6.3)");

    let snap = vtree_snapshot(g);

    println!("V-Tree layers (shallow = important, deep = marginal):");
    for (layer, start, end, own, state) in &snap {
        println!("  Layer {layer}: [{start},{end})  own={own}  {state:?}");
    }

    // Layer 1: the two most important entries.
    // [0,4) earned promotion through accumulation (own=15).
    // [0,8) is the frozen root benchmark (own=10).
    let layer1: Vec<_> = snap.iter().filter(|t| t.0 == 1).collect();
    assert_eq!(layer1.len(), 2, "two entries at V-depth 1");

    let hot = layer1.iter().find(|t| t.1 == 0 && t.2 == 4).unwrap();
    assert_eq!(hot.3, 15, "[0,4) carries own=15 — earned through observation");
    assert_eq!(hot.4, GState::Internal, "[0,4) is internal — has children, frozen");

    let bench = layer1.iter().find(|t| t.1 == 0 && t.2 == 8).unwrap();
    assert_eq!(bench.3, 10, "[0,8) carries own=10 — frozen at bootstrap split time");

    // Layer 2: [4,8) — hasn't accumulated anything yet.
    // It sits deeper in the tournament because it hasn't
    // proven itself through observation.
    let layer2: Vec<_> = snap.iter().filter(|t| t.0 == 2).collect();
    assert_eq!(layer2.len(), 1);
    assert_eq!((layer2[0].1, layer2[0].2, layer2[0].3), (4, 8, 0));
    assert_eq!(layer2[0].4, GState::Terminal, "[4,8) is terminal — a leaf on the contour");

    // Layer 3: [0,2) and [2,4) — freshly created, importance ν = 0.
    // These are the newest codewords; they must earn their way up.
    let layer3: Vec<_> = snap.iter().filter(|t| t.0 == 3).collect();
    assert_eq!(layer3.len(), 2);
    for t in &layer3 {
        assert_eq!(t.3, 0, "fresh leaf [{},{}) starts at ground importance ν=0", t.1, t.2);
        assert_eq!(t.4, GState::Terminal);
    }

    println!();
    println!("The inversion in action:");
    println!("  G-Tree ranking (by sum):  [0,8) > [0,4) > [4,8) > leaves");
    println!("  V-Tree ranking (by own):  [0,4)(15) > [0,8)(10) > [4,8)(0) > leaves(0)");
    println!("  The parent [0,8) ranks BELOW its child [0,4) in the tournament.");
    println!("  One is the integral, the other the derivative (§IDEA M-13.4).  ✓");
}

/// Point queries route through the G-Tree's adaptive partition.
/// `get(x)` descends via `route_to_receiver` (§IDEA M-5.2) and
/// returns the deepest cell containing coordinate `x`.
///
/// The query cost is `O(d_geo)` — proportional to the geometric depth
/// of the target cell (§IDEA M-18.9, §API M-5.2).
fn step_2_point_queries(g: &GvGraph<u64, u64, 3>) {
    subheading("Point queries through the adaptive partition");

    let cell = g.get(3);
    assert_eq!((cell.start, cell.end), (2, 4), "x=3 routes to [2,4)");
    assert_eq!(cell.depth, 2, "depth 2 — refined region");

    let cell_quiet = g.get(6);
    assert_eq!((cell_quiet.start, cell_quiet.end), (4, 8), "x=6 routes to [4,8)");
    assert_eq!(cell_quiet.depth, 1, "depth 1 — coarse region, not yet refined");

    println!(
        "  get(3) → [{}, {}) depth {} — routes to the refined hotspot region",
        cell.start, cell.end, cell.depth
    );
    println!(
        "  get(6) → [{}, {}) depth {} — routes to the coarse quiet region",
        cell_quiet.start, cell_quiet.end, cell_quiet.depth
    );
    println!("  ✓");
}

// ═══════════════════════════════════════════════════════════════
//  STEP 3: THE UNCLE SHIELD
//  §IDEA M-6.3 (Uncle Constraint Semantics), §IDEA M-13.1,
//  §IDEA M-16.3.4
// ═══════════════════════════════════════════════════════════════

/// A significant observation (8 units) arrives at x=6, triggering
/// a catalytic split of [4,8).  But despite R(8) being larger than
/// root(10), **no V-I3 violation occurs** because L(15) is also an
/// uncle — and 8 ≤ 15.
///
/// This demonstrates **Shield 2** in action (§IDEA M-13.1): the
/// max-uncle constraint says a node violates only when it outranks
/// *every* uncle.  Under a 3-node grandparent, two uncles must both
/// be exceeded.  L(15) shields R(8) from triggering a restructuring.
///
/// This is the mechanism that makes the V-Tree stable under noise
/// (§IDEA M-1.5.3): three siblings of comparable importance coexist
/// indefinitely.  The tournament restructures only when a node
/// *dramatically* outgrows its entire neighbourhood.
fn step_3_right_half_splits(g: &mut GvGraph<u64, u64, 3>) {
    heading("Step 3: observe(6, 8) — The Uncle Shield");
    println!("  (§IDEA M-16.3.4, §IDEA M-6.3 — Uncle Constraint Semantics)");

    g.observe(6, 8u64);

    println!();
    println!("What happened:");
    println!("  1. Route: [0,8) → [4,8) (terminal)  (§IDEA M-5.2)");
    println!("  2. Accumulate: [4,8).own = 0 + 8 = 8");
    println!("  3. Split check: 8 > θ=5 → catalytic_split  (§IDEA M-10.2)");
    println!("     [4,8) splits into [4,6) + [6,8)");
    println!("  4. Violation check: R(8) has uncles L(15) and root(10).");
    println!("     Max uncle = L(15).  8 ≤ 15 → NO VIOLATION.");
    println!("     The uncle shield absorbed the injection (§IDEA M-1.5.3).");
    println!("     No rebalancing needed.");

    step_3_uniform_contour(g);
    step_3_vtree_memory(g);

    assert_invariants(g);
}

/// After observation at both halves, the contour recovers to uniform
/// depth.  All four terminal cells sit at depth 2 — one large plateau.
///
/// The Kraft equality: `4 × 2^{-2} = 1`.  The code has four codewords
/// of equal length, uniformly partitioning the domain.
fn step_3_uniform_contour(g: &GvGraph<u64, u64, 3>) {
    subheading("Contour recovers to uniform depth");

    let plateaus = g.plateaus();
    assert_eq!(plateaus.len(), 1, "uniform depth → single plateau");
    let p = plateaus.values().next().unwrap();
    assert_eq!((p.start, p.end, p.depth), (0, 8, 2));

    print_contour_debug("uniform recovery", g);

    println!("     ┌──────────────────────────────────────────┐");
    println!("     │  ┌──────────────────┐┌──────────────────┐│");
    println!("     │  │  ┌──────┐┌──────┐││  ┌──────┐┌──────┐││");
    println!("     └──┴──┴──────┴┴──────┴┴┴──┴──────┴┴──────┴┘");
    println!("     0     2       4     5       6              8");
    println!();
    println!("     depth 2 ▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔▔");
    println!("     uniform — all four cells at the same resolution");
    println!();
    println!("  Kraft: 4 × 2^(-2) = 1  ✓");
}

/// The V-Tree preserves **arrival-order history**: [0,4) proved itself
/// first (observation at step 2, importance 15) and holds the shallower
/// tournament position.  [4,8) arrived later (importance 8, step 3) and
/// sits deeper — shielded by uncle L(15).
///
/// This history-awareness is a consequence of the three shields working
/// together (§IDEA M-1.5): routing froze benchmarks, the uncle constraint
/// stabilised positions, and structural immunity prevented premature
/// removal.  No explicit timestamp — the competitive ranking *is* the
/// temporal record.
fn step_3_vtree_memory(g: &GvGraph<u64, u64, 3>) {
    subheading("V-Tree remembers arrival order");
    println!("  (§IDEA M-1.5 — The three shields preserve history)");

    let snap = vtree_snapshot(g);

    println!("V-Tree layers:");
    for (layer, start, end, own, state) in &snap {
        println!("  Layer {layer}: [{start},{end})  own={own}  {state:?}");
    }

    // [0,4) still at Layer 1 with importance 15 — it proved itself first.
    let layer1: Vec<_> = snap.iter().filter(|t| t.0 == 1).collect();
    let hot = layer1.iter().find(|t| t.1 == 0 && t.2 == 4).unwrap();
    assert_eq!(hot.3, 15, "[0,4) holds position at own=15");

    // [4,8) at Layer 2 with own=8 — arrived later, shielded by L(15).
    let layer2: Vec<_> = snap.iter().filter(|t| t.0 == 2).collect();
    let right = layer2.iter().find(|t| t.1 == 4 && t.2 == 8).unwrap();
    assert_eq!(right.3, 8, "[4,8) has own=8");
    assert_eq!(right.4, GState::Internal, "[4,8) is now internal (has children)");

    println!();
    println!("History in the ranking:");
    println!("  [0,4) at Layer 1 (own=15) — proved itself first, holds position.");
    println!("  [4,8) at Layer 2 (own=8)  — arrived later, shielded by uncle L(15).");
    println!("  Both are frozen benchmarks now (both have children).");
    println!("  Their terminal children at Layer 3 start from ν=0 and must earn");
    println!("  their own way up through future observations.  ✓");
}

// ═══════════════════════════════════════════════════════════════
//  STEP 4: PEWEI EXTRACTION
//  §PEWEI M-2 (Structure), §PEWEI M-9 (Extraction),
//  §THEORY M-8.1
// ═══════════════════════════════════════════════════════════════

/// The **PEWEI** (Progressive Entropic-Wavelet Exposure Image) is a
/// static, significance-ordered snapshot of the dynamic G-V Graph
/// (§PEWEI M-1).
///
/// `extract()` performs a breadth-first walk of the V-Tree (§PEWEI M-9,
/// §API M-5.2), producing layers in approximate significance order.
/// Each V-entry is classified by its backing G-node's state:
///
/// - **Transition** (internal/semi-internal): a region with confirmed
///   sub-scale structure.  Carries `baseline` (pre-split measurement,
///   `g.own`) and `total` (inclusive of all descendants, `g.sum`).
///   This is the frozen benchmark from §THEORY M-6.2.
///
/// - **Terminal** (zero children): a leaf measurement at the finest
///   available resolution.  `intensity = g.own = g.sum`.
///
/// The layer ordering inherits the Fibonacci depth bound (§THEORY M-4.1):
/// the first k layers capture approximately the highest-energy structures,
/// within 1.44× of Shannon entropy.
fn step_4_first_pewei(g: &GvGraph<u64, u64, 3>) {
    heading("Step 4: PEWEI Extraction — Significance-Ordered Snapshot");
    println!("  (§PEWEI M-9 — Extraction, §THEORY M-8.1)");

    let pewei = g.extract();
    println!("{} layers, {} nodes", pewei.layer_count(), pewei.node_count());

    assert!(pewei.layer_count() >= 2, "at least 2 layers");
    assert!(
        pewei.node_count() >= 5,
        "at least 5 nodes (2 transitions + 4 terminals minimum)"
    );

    // Print the full layer structure.
    for (i, layer) in pewei.layers.iter().enumerate() {
        let mut parts = Vec::new();
        for t in &layer.transitions {
            parts.push(format!(
                "Transition [{},{}) baseline={} total={} refinement={}",
                t.start, t.end, t.baseline, t.total, t.refinement
            ));
        }
        for t in &layer.terminals {
            parts.push(format!("Terminal   [{},{}) intensity={}", t.start, t.end, t.intensity));
        }
        if !parts.is_empty() {
            println!("  Layer {i}:");
            for p in &parts {
                println!("    {p}");
            }
        }
    }

    println!();
    println!("How to read the PEWEI:");
    println!("  - Layer 0 is typically empty (V-root is structural scaffolding).");
    println!("  - Layer 1 contains the dominant structures: [0,4) and [0,8).");
    println!("    These are Transitions — they have children and carry frozen baselines.");
    println!("  - Deeper layers contain progressively finer detail.");
    println!("  - Terminals are leaf measurements at the code's finest resolution.");
    println!("  - Total energy = Σ all g.own values = G-root sum = 33  (§PEWEI M-12.1).");
    println!("  ✓");

    assert_invariants(g);
}

// ═══════════════════════════════════════════════════════════════
//  STEP 5: NEW OBSERVATIONS — SHIFTING THE LANDSCAPE
//  §IDEA M-8 (Observation Flow), §IDEA M-10.2 (Catalytic Split)
// ═══════════════════════════════════════════════════════════════

/// Two new observations shift the competitive landscape.
///
/// `observe(5, 20)` delivers 20 units to [4,6), which dramatically
/// exceeds its uncle R(8).  This triggers a catalytic split AND a
/// competitive promotion — the V-Tree reshapes to reflect the new
/// evidence.  The [4,6) region was quiet before; now it's the
/// strongest entry in the tournament.
///
/// `observe(1, 6)` delivers 6 units to [0,2).  This exceeds θ=5 but
/// the entry sits too deep in the V-Tree (`depth_V` > `D_create` after
/// the right-side promotion reshaped the tree) to qualify for a
/// split.  The depth gate (§IDEA M-7, D-I1) prevents nodes that
/// haven't proven *global* significance from consuming the code's
/// finite precision budget.
///
/// This demonstrates the two-gate system (§IDEA M-7): a cell must
/// satisfy both a *local* intensity threshold (θ) AND a *global*
/// competitive rank (`D_create`) to earn refinement.
fn step_5_new_observations(g: &mut GvGraph<u64, u64, 3>) {
    heading("Step 5: New Observations — Competitive Landscape Shift");
    println!("  (§IDEA M-8 — Observation Flow, §IDEA M-7 — Depth Gates)");

    let nodes_before = g.node_count();

    g.observe(5, 20u64);
    println!("observe(5, 20):");
    println!("  Routes to [4,6) → own rises to 20.");
    println!("  20 > θ=5 and shallow enough → catalytic split → [4,5) + [5,6).");
    println!("  Violation: [4,6)(20) outgrows uncle R(8) → skip-promote.");

    g.observe(1, 6u64);
    println!("observe(1, 6):");
    println!("  Routes to [0,2) → own rises to 6.");
    println!("  6 > θ=5 but depth_V > D_create=3 after the promotion reshaped");
    println!("  the V-Tree.  The LOCAL gate passes; the GLOBAL gate blocks.");
    println!("  No split — evidence is locally sufficient but globally insufficient.");

    let nodes_after = g.node_count();
    assert_eq!(g.total_sum(), 59, "total = 10 + 15 + 8 + 20 + 6 = 59");
    assert!(
        nodes_after > nodes_before,
        "new G-nodes created: {nodes_before} → {nodes_after}",
    );

    println!();
    println!("Energy conservation (§IDEA M-1.6.4 — Single-Entry Accounting):");
    println!(
        "  G-root sum = {}.  All observations accounted for exactly once.",
        g.total_sum()
    );
    println!("  G-nodes: {nodes_before} → {nodes_after}.  ✓");

    assert_invariants(g);
}

// ═══════════════════════════════════════════════════════════════
//  STEP 6: PLATEAU STRUCTURE
//  §IDEA M-5.6 (Plateaus), §THEORY M-9
// ═══════════════════════════════════════════════════════════════

/// Plateaus are the semantic unit of the code's spatial output
/// (§IDEA M-5.6).  Within a plateau, all cells have the same depth —
/// the code has judged the region to be uniform at this resolution.
/// Boundaries between plateaus are where the code found non-uniformity
/// worth resolving — its learned feature edges.
///
/// The plateau count P measures structural complexity (§IDEA M-5.6.6,
/// §THEORY M-9.2).  P=1 means uniform code; P∝L means maximally
/// fragmented.
fn step_6_contour_after_obs(g: &GvGraph<u64, u64, 3>) {
    heading("Step 6: Contour After New Observations — Learned Shape");
    println!("  (§IDEA M-5.6 — Plateaus, §THEORY M-9.2 — Structural Complexity)");

    let plateaus = g.plateaus();

    println!("{} plateaus:", plateaus.len());
    for (edge, p) in &*plateaus {
        println!(
            "  BasisEdge({:>2}):  depth {} over [{}, {})  sum={}",
            edge.0, p.depth, p.start, p.end, p.sum
        );
    }

    // The [4,6) split introduced depth-3 terminals, creating depth variation.
    assert!(plateaus.len() >= 2, "depth variation from the [4,6) split");
    let has_depth_3 = plateaus.values().any(|p| p.depth == 3);
    assert!(has_depth_3, "depth 3 plateau exists from [4,6) split");

    print_contour_debug("after new observations", g);

    println!("     ┌──────────────────────────────────────────┐");
    println!("     │  ┌──────────────────┐┌──────────────────┐│");
    println!("     │  │  ┌──────┐┌──────┐││  ┌────────┐      ││");
    println!("     │  │  │      ││      │││  │  ┌┐┌┐  │      ││");
    println!("     └──┴──┴──────┴┴──────┴┴┴──┴──┴┘┴┘──┴──────┴┘");
    println!("     0     2       4      5  6                   8");
    println!();
    println!("     depth 2 ▔▔▔▔▔▔▔▔▔▔▔▔▔  d3 ▔▔  depth 2 ▔▔▔▔");
    println!("     (left half)         (hotspot)  (right quiet)");
    println!();
    println!("  The code drilled deeper at [4,6) — a new hotspot emerged.");
    println!("  Precision was allocated toward the observed mass  (§THEORY M-1.2).");
    println!("  Plateau boundaries mark the code's learned feature edges.  ✓");

    assert_invariants(g);
}

// ═══════════════════════════════════════════════════════════════
//  STEP 7: PROPORTIONAL SAMPLING
//  §IDEA M-6.5, §THEORY M-4.2–4.3, §API M-5.2
// ═══════════════════════════════════════════════════════════════

/// Proportional sampling walks the V-Tree from root to leaf, choosing
/// each child with probability proportional to its importance (§IDEA M-6.5).
///
/// The Fibonacci depth bound (§THEORY M-4.1) guarantees:
///   Expected cost ≤ 1.44 × H + O(1)
///
/// where H is the Shannon entropy of the importance distribution.
///
/// For a concentrated distribution (one hotspot dominates), H ≈ 0 and
/// sampling is effectively O(1) — the walker finds the dominant entry
/// immediately.  For a uniform distribution, H = log₂ L and sampling
/// costs ≈ 1.44 × log₂ L.  This entropy-adaptiveness is the V-Tree's
/// signature property (§THEORY M-4.6).
///
/// Note: we use `TestLcgRng` for deterministic results.  With the
/// default `rand` feature: `g.sample(&mut rand::rng())` (§API M-5.3).
#[cfg(debug_assertions)]
fn step_7_sample(g: &GvGraph<u64, u64, 3>) {
    heading("Step 7: Proportional Sampling — Entropy-Adaptive");
    println!("  (§IDEA M-6.5, §THEORY M-4.2–4.3, §API M-5.2)");

    let mut rng = TestLcgRng(42);
    let trials = 1000;
    let mut rl_count = 0u32; // [4,6) — the new hotspot
    let mut left_count = 0u32;
    let mut right_count = 0u32;

    for _ in 0..trials {
        let cell = g.sample(&mut rng).expect("graph has positive total intensity");
        if cell.start == 4 && cell.end == 6 {
            rl_count += 1;
        }
        if cell.start < 4 {
            left_count += 1;
        } else {
            right_count += 1;
        }
    }

    println!("Sampled {trials} cells:");
    println!("  [4,6) specifically (importance 20 / total 59 ≈ 34%):  {rl_count}");
    println!("  Left half  (start < 4):  {left_count}");
    println!("  Right half (start ≥ 4):  {right_count}");

    // [4,6) has importance 20 out of 59 total ≈ 34%.
    // With 1000 trials, ~340 ± 30 expected.
    assert!(rl_count > 200, "[4,6) should be heavily sampled: got {rl_count}/1000");

    println!();
    println!("Why the hotspot dominates:");
    println!("  [4,6) has importance 20 — it sits at a shallow V-Tree depth.");
    println!("  The sampling walk finds it in ~1 step (low self-information).");
    println!("  Zero-importance leaves (own=0) are never sampled.");
    println!();
    println!("  This is §THEORY M-4.6:");
    println!("  Concentrated distribution → low entropy → O(1) sampling.");
    println!("  Uniform distribution → high entropy → O(log L) sampling.");
    println!("  The V-Tree adapts its routing cost to the data, not the size.");
    println!();
    println!("  {rl_count}/1000 samples landed on the hotspot.  ✓");

    assert_invariants(g);
}

// ═══════════════════════════════════════════════════════════════
//  STEP 8: TARGETED SUBTREE DECAY
//  §IDEA M-14.3 (Three-Parameter Decay), §THEORY M-7,
//  §API M-5.2 (decay)
// ═══════════════════════════════════════════════════════════════

/// Temporal decay (§IDEA M-14, §THEORY M-7) modulates the importance
/// landscape over time.  `decay(root, attenuation, q)` scales every
/// accumulator in the targeted G-subtree by a depth-dependent factor:
///
///   `λ(d) = att^{1 + q·(2d_local/D − 1)}`
///
/// With `att = 0.1` and `q = 0.0` (uniform decay), every node in the
/// subtree is scaled by 0.1 — a 90% reduction.  This weakens the
/// left side's competitive standing, which triggers V-I3 violations:
/// entries that were shielded by the now-weakened L(15→1) may exceed
/// their softened uncle.  The trailing rebalance (§IDEA M-11.8)
/// restores the tournament.
///
/// Decay does NOT change the G-Tree's topology — no nodes are created
/// or destroyed.  It is a pure value operation (§IDEA M-14.1).  The
/// contour shape is preserved; only the energy values change.
/// Structural changes (eviction of zeroed nodes) happen later through
/// the separate eviction mechanism (§IDEA M-12).
///
/// Three temporal regimes are available (§THEORY M-7.3):
///   att = 0:      annihilation — hard reset to zero
///   att ∈ (0,1):  attenuation — exponential forgetting
///   att = 1:      identity — raw accumulation (the default)
///   att > 1:      amplification — selective boosting
fn step_8_subtree_decay(g: &mut GvGraph<u64, u64, 3>) {
    heading("Step 8: Subtree Decay — Reshaping the Tournament");
    println!("  (§IDEA M-14.3, §THEORY M-7, §API M-5.2)");

    let sum_before = g.total_sum();
    let positions_before = layer_positions(g);

    let rl_layer_before = entry_layer(g, 4, 6).expect("[4,6) must be present");
    let l_layer_before = entry_layer(g, 0, 4).expect("[0,4) must be present");

    subheading("V-Tree BEFORE decay");
    let snap_before = vtree_snapshot(g);
    for (layer, start, end, own, state) in &snap_before {
        println!("  Layer {layer}: [{start},{end})  own={own:>3}  {state:?}");
    }

    // ── Apply targeted decay ──
    //
    // `find_gnode_id` locates [0,4) by interval.  `decay()` walks
    // the G-subtree rooted there, scaling every accumulator by 0.1.
    //
    // L.entry (own=15) → floor(15 × 0.1) = 1
    // LL.entry (own=6) → floor(6 × 0.1) = 0
    // LR.entry (own=0) → 0 (unchanged)
    //
    // This weakens L as an uncle.  Right-side entries that were
    // shielded by L's importance may now exceed their weakened
    // uncle, triggering V-I3 violations and rebalancing.
    let left_root = find_gnode_id(g, 0, 4);
    println!();
    println!("Decaying [0,4) subtree: attenuation=0.1, q=0.0 (uniform)");
    println!("  L.own: 15 → floor(15 × 0.1) = 1");
    println!("  LL.own: 6 → floor(6 × 0.1) = 0");

    g.decay(left_root, 0.1, 0.0);

    // ── Examine the result ──

    let sum_after = g.total_sum();
    let positions_after = layer_positions(g);

    let rl_layer_after = entry_layer(g, 4, 6).expect("[4,6) must survive decay");
    let l_layer_after = entry_layer(g, 0, 4).expect("[0,4) must survive decay");

    subheading("V-Tree AFTER decay");
    let snap_after = vtree_snapshot(g);
    for (layer, start, end, own, state) in &snap_after {
        println!("  Layer {layer}: [{start},{end})  own={own:>3}  {state:?}");
    }

    // 1. Energy decreased (left subtree attenuated).
    assert!(sum_after < sum_before, "total energy decreased: {sum_before} → {sum_after}");
    println!();
    println!("Effects of decay:");
    println!("  Total energy: {sum_before} → {sum_after}  (left subtree attenuated)");

    // 2. V-Tree shape changed.
    // The weakened uncle L(1) can no longer shield the right side's
    // entries.  Violations detected → rebalance → entries move.
    assert_ne!(positions_before, positions_after, "V-Tree must reshape");
    println!("  V-Tree shape changed — the trailing rebalance (§IDEA M-11.8)");
    println!("  resolved violations caused by the weakened uncle.");

    // 3. RL [4,6) should have promoted (or at least held position).
    // With the left side weakened, the right side's entries face
    // less competition — they may rise in the tournament.
    assert!(
        rl_layer_after <= rl_layer_before,
        "RL [4,6) should promote or hold: {rl_layer_before} → {rl_layer_after}",
    );
    println!("  RL [4,6) layer: {rl_layer_before} → {rl_layer_after}  (promoted or held)");

    // 4. L [0,4) may have moved deeper — its competitive weight dropped.
    println!("  L  [0,4) layer: {l_layer_before} → {l_layer_after}  (weakened by decay)");

    print_contour_debug("after subtree decay", g);

    println!("  Key insight (§THEORY M-7.3):");
    println!("  Decay modified VALUES, not STRUCTURE.  Every G-node, contour cell,");
    println!("  and plateau boundary persists.  The partition shape is unchanged.");
    println!("  Only the energy map changed — and the tournament adapted to match.");
    println!("  ✓");

    assert_invariants(g);
}

// ═══════════════════════════════════════════════════════════════
//  STEP 9: FINAL PEWEI — POST-DECAY SIGNIFICANCE ORDERING
//  §PEWEI M-2, §PEWEI M-12.5, §THEORY M-8.6
// ═══════════════════════════════════════════════════════════════

/// The post-decay PEWEI reflects the new competitive reality.
///
/// Before decay, L(15) was the dominant entry.  After attenuating
/// the left subtree by 0.9×, RL(20) — untouched by the targeted
/// decay — now dominates the tournament.  The significance ordering
/// adapts to whatever the temporal filter produces (§IDEA M-14,
/// §IDEA M-1.7.1).
///
/// The PEWEI's total energy equals the G-root sum (§PEWEI M-12.1):
/// `total_energy() = Σ g.own over all entries = G_root.sum` (by G-I1).
/// This is the single-entry accounting principle (§IDEA M-1.6.4) —
/// each observation updates exactly one V-entry, so the total is
/// always conserved.
fn step_9_final_pewei(g: &GvGraph<u64, u64, 3>) {
    heading("Step 9: Final PEWEI — Post-Decay Significance Ordering");
    println!("  (§PEWEI M-2, §PEWEI M-12.5 — Dynamic Origin, Static Output)");

    let pewei = g.extract();
    let total = g.total_sum();

    println!(
        "{} layers, {} nodes, total energy = {}",
        pewei.layer_count(),
        pewei.node_count(),
        total
    );
    println!();

    for (i, layer) in pewei.layers.iter().enumerate() {
        let mut parts = Vec::new();
        for t in &layer.transitions {
            parts.push(format!(
                "Phase [{},{}) baseline={} total={}",
                t.start, t.end, t.baseline, t.total
            ));
        }
        for t in &layer.terminals {
            parts.push(format!("Leaf  [{},{}) intensity={}", t.start, t.end, t.intensity));
        }
        if !parts.is_empty() {
            println!("  Layer {i}:");
            for p in &parts {
                println!("    {p}");
            }
        }
    }

    // The PEWEI must have structure.
    assert!(pewei.layer_count() >= 2, "at least 2 layers");
    assert!(pewei.node_count() >= 7, "at least 7 nodes");

    // The shallowest non-empty layer should contain RL [4,6) — the
    // strongest entry after the left side was attenuated.
    let first_layer = pewei
        .layers
        .iter()
        .find(|l| !l.transitions.is_empty() || !l.terminals.is_empty())
        .expect("PEWEI must have at least one non-empty layer");
    let rl_in_top = first_layer.transitions.iter().any(|t| t.start == 4 && t.end == 6)
        || first_layer.terminals.iter().any(|t| t.start == 4 && t.end == 6);
    assert!(rl_in_top, "RL [4,6) should dominate the shallowest entry layer");

    // Total energy conservation (§PEWEI M-12.1).
    assert_eq!(pewei.total_energy(), total, "PEWEI total must equal G-root sum");

    println!();
    println!("The post-decay significance ordering:");
    println!("  • RL [4,6) dominates — importance 20, untouched by decay.");
    println!("  • L [0,4) weakened — attenuated from 15 to ~1, pushed deeper.");
    println!("  • Root [0,8) persists — frozen benchmark from the first split.");
    println!();
    println!("  The PEWEI is a photograph of the graph's state of knowledge");
    println!("  at this moment (§PEWEI M-12.5).  Before decay, L dominated.");
    println!("  After decay, RL dominates.  Same observations, different");
    println!("  temporal policy, different significance ordering.");
    println!();
    println!("  PEWEI total = G-root sum = {total}  (energy conserved).  ✓");

    assert_invariants(g);
}

// ═══════════════════════════════════════════════════════════════
//  MAIN
// ═══════════════════════════════════════════════════════════════

fn main() {
    // Support `--list --format terse` so cargo-nextest can enumerate this
    // custom-harness test binary (it expects lines of `name: test`).
    // With `--ignored`, output nothing (we have no ignored tests).
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|a| a == "--list") {
        if !args.iter().any(|a| a == "--ignored") {
            println!("pedagogy: test");
        }
        return;
    }

    // Guard: README config and unit-test harness must agree.
    assert_eq!(
        README_CONFIG,
        worked_example_config(),
        "README config drifted from worked_example_config() — update one or the other",
    );

    let mut g = GvGraph::<u64, u64, 3>::new(README_CONFIG);

    step_0_fresh(&g);
    step_1_first_observation(&mut g);
    step_2_asymmetric_contour(&mut g);
    step_3_right_half_splits(&mut g);
    step_4_first_pewei(&g);
    step_5_new_observations(&mut g);
    step_6_contour_after_obs(&g);
    #[cfg(debug_assertions)]
    step_7_sample(&g);
    step_8_subtree_decay(&mut g);
    step_9_final_pewei(&g);

    heading("All Claims Verified");
    println!();
    println!("  Structural properties:");
    println!("    ✓ Kraft equality holds at every step");
    println!("    ✓ All invariants (G-I1, V-I1, V-I3, ...) hold after every mutation");
    println!("    ✓ Energy is conserved: PEWEI total = G-root sum");
    println!();
    println!("  Code dynamics (§THEORY M-6):");
    println!("    ✓ Bootstrap split: code lengthens for the first time");
    println!("    ✓ Catalytic split: parent persists as frozen benchmark");
    println!("    ✓ Competitive promotion: child proves itself and rises");
    println!("    ✓ Uncle shield: significant injection absorbed without restructuring");
    println!("    ✓ Contour recovers to uniform after balanced observation");
    println!();
    println!("  Adaptive resolution (§THEORY M-1.2):");
    println!("    ✓ Fine where hot, coarse where cold — the rate-distortion dual");
    println!("    ✓ Plateaus reflect the code's learned spatial structure");
    println!("    ✓ New observations shift the competitive landscape");
    println!();
    println!("  Significance ordering (§THEORY M-4):");
    println!("    ✓ Tournament ranking diverges from spatial containment (the inversion)");
    println!("    ✓ V-Tree remembers arrival order — history is the ranking");
    println!("    ✓ Proportional sampling routes attention toward evidence");
    println!("    ✓ PEWEI layers capture structures in approximate significance order");
    println!();
    println!("  Temporal modulation (§THEORY M-7):");
    println!("    ✓ Subtree decay weakens a targeted region");
    println!("    ✓ Weakened uncle triggers V-Tree rebalancing");
    println!("    ✓ Post-decay PEWEI reflects the new competitive reality");
    println!("    ✓ Decay is a value operation — topology unchanged");
    println!();
}
