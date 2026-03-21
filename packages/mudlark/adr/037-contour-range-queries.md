# ADR-M-037: Contour Range Queries (Revised)

**Status:** Implemented  
**Date:** 2026-03-12  
**Supersedes:** ADR-M-037 (2026-03-11, implemented)  
**Phase:** 4 (Output)  
**Relates to:** [ADR-M-020](020-range-query-design.md) (range query design),
[ADR-M-026](026-point-query-and-plateau-semantics.md) (plateau semantics),
[ADR-M-032](032-three-surface-model.md) (three-surface model),
[ADR-M-036](036-sentinel-integration-api.md) (sentinel integration API)  
**Spec:** §CR.1–§CR.14 (Contour Ranges extension in idea.md),
§IDEA M-5.5 (queries), §IDEA M-5.6 (plateaus)  
**API:** [§API M-5.2, range sum](../docs/api.md#range-sum),
[§API M-5.2, contour range](../docs/api.md#contour-range-decomposition),
[§API M-5.2, plateau selection](../docs/api.md#plateau-selection)  
**Surface:** 1 + 2 (Prints + Film)

---

## Context

### The original ADR and the spec rewrite

ADR-M-037 (2026-03-11) introduced `contour_range()` and
`contour_range_energy()` based on an earlier draft of the Contour
Ranges extension (§CR.1–§CR.9). That draft defined contour ranges
in terms of a **three-set partition** of G-nodes: interior
($\mathcal{I}$), boundary ($\mathcal{B}$), and straddling ancestors
($\mathcal{S}$). The implementation shipped five public types
(`ContourRange`, `ContourRangeEnergy`, `InteriorElement`,
`BoundaryElement`, `StraddlingAncestor`) and a decomposition identity
tying the three sets to the energy scalar.

A pre-1.0 audit found the original definition lacking. The spec was
rewritten (§CR.1–§CR.13) with a significantly simpler model:

- A contour range is defined by its **basis set** — the same minimal
  G-node cover used for single plateaus, applied to the wider
  interval (§CR.2).
- **Thatching** (§CR.3) is the central structural phenomenon, not a
  three-set partition. Basis elements that are semi-internal at the
  range boundary have `.sum` that leaks energy outside the range.
  Interior thatching between constituent plateaus is resolved by
  **basis consolidation** (§CR.4).
- The contour range **energy** is simply $\sum_{\text{basis}}
  R.\text{sum}$ (§CR.6) — the same formula used for single plateaus.
- The three-set partition ($\mathcal{I}$/$\mathcal{B}$/$\mathcal{S}$)
  no longer appears. Straddling ancestors with pro-rated fractions
  are the province of `range_sum` (§CR.13, exact energy), not of
  contour ranges.
- Two new concepts were added: **Plateau Selection** (§CR.12) bridges
  arbitrary dyadic coordinates to lattice-aligned contour ranges, and
  **Exact Energy** (§CR.13) formalises the pro-rated `range_sum` as a
  complementary measure.

The existing implementation must be revised to align with the new
spec. This ADR proposes six decisions that reshape the public API.

### What exists today (implemented from original ADR)

| Method                             | Returns                                                  | Notes                                 |
| ---------------------------------- | -------------------------------------------------------- | ------------------------------------- |
| `contour_range(start, end)`        | `ContourRange<C, V>` with three `Vec`s + 6 energy fields | Three-set partition                   |
| `contour_range_energy(start, end)` | `ContourRangeEnergy<V>` (`Copy`, 6 scalars)              | Delegates to full decomposition       |
| `range_sum(range)`                 | `V` (scalar)                                             | Arbitrary `RangeBounds<C>`, pro-rates |
| `plateaus()`                       | `BTreeMap<BasisEdge, Plateau>`                           | Full contour map                      |

Public types from original ADR: `InteriorElement`, `BoundaryElement`,
`StraddlingAncestor`, `ContourRange`, `ContourRangeEnergy`.

### What the spec now says

| Spec section | Concept                                                   | API implication                                        |
| ------------ | --------------------------------------------------------- | ------------------------------------------------------ |
| §CR.2        | Basis set — single flat set of G-nodes                    | Replace three `Vec`s with one `Vec<BasisElement>`      |
| §CR.3        | Thatching — ≤2 boundary semi-internals whose `.sum` leaks | Mark basis elements, don't separate them               |
| §CR.6        | Energy = $\sum_{\text{basis}} R.\text{sum}$               | Not `range_sum`; different value when thatching exists |
| §CR.12       | Plateau selection: arbitrary coords → lattice endpoints   | New method needed                                      |
| §CR.13       | Exact energy = `range_sum(l, r)` with pro-rating          | Complementary to contour range energy                  |

### Divergences between implementation and spec

1. **Three-set partition is gone.** The spec defines one basis set;
   the implementation has three `Vec`s.

2. **`energy` field is wrong.** The implementation computes
   `self.range_sum(start..end)` — the exact energy (§CR.13) — and
   stores it as `energy`. The spec's contour range energy (§CR.6)
   is $\sum_{\text{basis}} R.\text{sum}$, which differs when
   boundary thatching or ancestor pro-ration exists (§CR.13.6).
   The two are **not generally ordered** — neither
   `exact_energy ≤ energy` nor `exact_energy ≥ energy` holds
   universally.

3. **Straddling ancestors don't belong.** Pro-rated fractional
   contributions are the mechanism of `range_sum` (§CR.13), not of
   the basis set (§CR.2).

4. **No plateau selection.** The spec adds §CR.12 as the bridge from
   arbitrary coordinates to lattice-aligned contour ranges. The API
   has no equivalent.

5. **Stale §CR references.** Every `§CR.N` reference in the original
   ADR points to renumbered or removed sections.

---

## Questions

### Q1: Return type shape (DC-037-1, revised)

The spec defines one basis set, not three disjoint sets. What
should `contour_range()` return?

| Option | Shape                                                                        | Notes                                                                                 |
| ------ | ---------------------------------------------------------------------------- | ------------------------------------------------------------------------------------- |
| A      | **Single `Vec<BasisElement>`** with an `is_boundary_thatch` flag per element | One flat list. Flag marks the ≤2 boundary thatchers. Simple, aligns with §CR.2–§CR.3. |
| B      | **Two lists:** `Vec<BasisElement>` + `Vec<BoundaryThatch>` (at most 2)       | Separates thatching elements. Slightly richer boundary info.                          |
| C      | **`Vec<GNodeId>` only** — caller uses `gnode_info()` for detail              | Minimal. Requires per-element round-trips.                                            |
| D      | **Keep three `Vec`s** (status quo)                                           | Preserves original API. Contradicts spec.                                             |

**Considerations:**

- The spec's §CR.3 defines thatching as a property of certain basis
  elements, not a separate set. A boolean flag on each element
  captures this naturally.
- Boundary thatching elements are rare (at most 2) and callers need
  to identify them to understand energy leakage. The flag avoids a
  second lookup.
- Option C loses the snapshot property — callers must re-traverse to
  get `own`/`sum`/`depth`. The basis set is bounded ($\leq 2N$), so
  the `Vec` allocation is modest.
- Option D contradicts the revised spec.

### Q2: Energy definition (DC-037-2, revised)

The implementation currently sets `energy = range_sum(start..end)`.
The spec defines two different quantities:

- **Contour range energy** (§CR.6): $E(\mathcal{R}) = \sum_{\text{basis}} R.\text{sum}$
- **Exact energy** (§CR.13): $\text{exact\_energy}(a_s, a_{e+1}) = \text{range\_sum}(a_s, a_{e+1})$

The two measures are **not generally ordered** (§CR.13.6):
$E(\mathcal{R}) = \text{exact\_energy} + B - A$, where $B \geq 0$
(boundary thatching) and $A \geq 0$ (ancestor pro-ration) under P1.
The sign of $B - A$ is indeterminate.

How should the `ContourRange` struct represent energy?

| Option | Fields                                               | Notes                                                                                                         |
| ------ | ---------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| A      | `energy` (= $\sum_{\text{basis}} R.\text{sum}$) only | Matches §CR.6. Caller uses `range_sum()` separately if exact energy needed.                                   |
| B      | **Both**: `energy` (§CR.6) + `exact_energy` (§CR.13) | Both measures in one result. Makes the gap ($B - A$) inspectable. One extra $O(N)$ traversal (or fused pass). |
| C      | `energy` (= `range_sum`, status quo)                 | Contradicts spec naming. Confusing.                                                                           |

**Considerations:**

- Providing both makes the relationship between the two measures
  explicit. Callers can inspect the gap $B - A$ (§CR.13.6):
  boundary thatching leakage minus ancestor pro-ration.
- The extra `range_sum` call is $O(N)$ and shares the same tree
  traversal structure. It could be fused into the basis walk, but
  even as a separate call the cost is marginal.
- Option C uses the spec's name for the wrong value — a source of
  bugs for anyone reading both spec and code.

### Q3: Energy scalars (DC-037-3, revised)

The original ADR defined five energy fields plus `plateau_count`.
The spec defines:

- $E(\mathcal{R}) = \sum_{\text{basis}} R.\text{sum}$ (§CR.6)
- $E_\times = E - \sum P.\text{sum}$ (§CR.10.4, cross-plateau energy)
- `exact_energy` (§CR.13, if included per Q2)

Which energy scalars should the struct carry?

| Current field          | Spec status                        | Recommendation                                            |
| ---------------------- | ---------------------------------- | --------------------------------------------------------- |
| `energy`               | Misaligned (currently `range_sum`) | **Redefine** to §CR.6: $\sum_{\text{basis}} R.\text{sum}$ |
| `exact_energy`         | §CR.13 (new)                       | **Add** — the pro-rated `range_sum` result                |
| `interior_energy`      | Not in spec                        | **Drop** — artifact of three-set partition                |
| `thatched_energy`      | Not in spec                        | **Drop** — artifact of three-set partition                |
| `plateau_energy`       | §CR.10.4                           | **Keep** — needed for $E_\times$                          |
| `cross_plateau_energy` | §CR.10.4                           | **Keep** — $E - \sum P.\text{sum}$                        |
| `plateau_count`        | Useful metadata                    | **Keep**                                                  |

| Option | Scalars                                                                             | Notes                                                    |
| ------ | ----------------------------------------------------------------------------------- | -------------------------------------------------------- |
| A      | `energy`, `exact_energy`, `plateau_energy`, `cross_plateau_energy`, `plateau_count` | Aligns with spec. Drops two obsolete fields.             |
| B      | All of A plus `interior_energy`, `thatched_energy`                                  | Backwards-compatible. Carries dead weight.               |
| C      | `energy`, `plateau_energy`, `cross_plateau_energy`, `plateau_count` only            | Omits exact energy — caller uses `range_sum` separately. |

### Q4: Plateau selection method (DC-037-4, new)

The spec adds §CR.12: given arbitrary dyadic coordinates $(l, r)$,
find the contiguous run of plateaus that overlap and return valid
contour range endpoints.

Should a public method be added?

| Option | Method                                                                                | Notes                                                                                      |
| ------ | ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| A      | `pub fn select_plateaus(&self, lo: C, hi: C) -> Option<(BasisEdge<C>, BasisEdge<C>)>` | Direct bridge from arbitrary coords to lattice endpoints. $O(\log P)$.                     |
| B      | No method — callers walk `plateaus()` manually                                        | Boilerplate. Error-prone.                                                                  |
| C      | Accept `RangeBounds<C>` in `contour_range` directly, snap internally                  | Violates CR-I1; silently changes the queried range. Already rejected by original ADR (A2). |

**Considerations:**

- The pattern "I have coordinates, I want a contour range" is the
  natural entry point for most callers. Without `select_plateaus`,
  every caller reimplements the floor-index lookup.
- Option A composes cleanly: `select_plateaus` returns endpoints,
  `contour_range` consumes them. The type system enforces that
  contour ranges use lattice-aligned endpoints.
- §CR.12.5: cost is $O(\log P)$ — two lookups in the plateau ordered
  map.

### Q5: Keep `ContourRangeEnergy`? (DC-037-5, revised)

The original ADR provided a `Copy` energy-only struct. With the
simplified scalar set (Q3), should it be retained?

| Option | Approach                                                                     | Notes                                                                                                                     |
| ------ | ---------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------- |
| A      | **Keep**, updated to new scalar set                                          | Clean API for callers who want only scalars. No `Vec` allocation. Can have a dedicated $O(N)$ pass that skips collection. |
| B      | **Drop** — callers ignore the `basis` field on `ContourRange`                | One fewer type. The `Vec` allocation is bounded ($\leq 2N$), so savings are small.                                        |
| C      | **Replace** with a method that returns `(V, V)` tuple (energy, exact_energy) | Minimal. Loses plateau/cross-plateau breakdown.                                                                           |

**Considerations:**

- The `basis` `Vec` is typically small, but the API contract "I only
  want scalars" is valuable for downstream code that shouldn't depend
  on structural details.
- A dedicated energy-only code path can skip `Vec` allocation and
  `BasisElement` construction entirely — useful for hot-path callers
  (e.g. sentinel monitoring).
- Current implementation delegates to `contour_range()` and discards
  the `Vec`s. Even if the dedicated path is deferred, retaining the
  type preserves the option.

### Q6: Endpoint validation (DC-037-6, unchanged)

How should endpoints be validated against the lattice $\mathcal{E}$?

This question is carried forward from the original ADR unchanged.

| Option | Behaviour                                            | Notes                                                               |
| ------ | ---------------------------------------------------- | ------------------------------------------------------------------- |
| A      | **Panic** if either endpoint is not in $\mathcal{E}$ | Strict. Matches `observe()` NaN panic convention (§API M-7.1).      |
| B      | **Return `None`** for invalid endpoints              | Lenient. Matches `gnode_info()` returning `None` for stale handles. |
| C      | **Snap to nearest lattice point**                    | Magical. Changes the caller's intent.                               |

**Considerations (unchanged):**

- The endpoint lattice changes with every mutation ($O(1)$ elements
  per mutation, §CR.10.3). Staleness is a timing issue, not a
  programmer error.
- Panicking (A) is appropriate for hard errors (like NaN), not for
  stale handles.
- Snapping (C) silently changes the queried range.

---

## Decisions

### DC-037-1 (revised): Option A — Single `Vec<BasisElement>` with thatch flag

The three-set partition is replaced by a single basis set (§CR.2).
Each element carries an `is_boundary_thatch` flag indicating whether
it is a boundary thatching semi-internal (§CR.3.2).

This eliminates `InteriorElement`, `BoundaryElement`, and
`StraddlingAncestor` as public types. One new type `BasisElement`
replaces all three.

Rationale:

- Directly models the spec's basis set definition.
- The flag captures the only classification the spec makes within
  the basis: boundary thatch vs not.
- Straddling ancestors with pro-rated fractions belong to `range_sum`
  (§CR.13), not to the contour range decomposition.

### DC-037-2 (revised): Option B — Both `energy` and `exact_energy`

The `ContourRange` struct carries two energy measures:

- `energy`: the spec's contour range energy (§CR.6),
  $\sum_{\text{basis}} R.\text{sum}$.
- `exact_energy`: the spec's exact energy (§CR.13),
  `range_sum(start..end)`.

The two are **not generally ordered** (§CR.13.6). The
relationship $E = \text{exact} + B - A$ decomposes into boundary
thatching $B \geq 0$ and ancestor pro-ration $A \geq 0$; the sign
of $B - A$ is indeterminate. The gap is inspectable.

Rationale:

- The old `energy` field was actually `range_sum` (exact energy).
  Callers who depended on that value get it as `exact_energy`.
- The spec's energy (§CR.6) is the structurally meaningful measure —
  consistent with single-plateau energy, consolidation-aware.
- Providing both makes the gap ($B - A$) inspectable without
  additional API calls, exposing the interplay of boundary
  thatching and ancestor pro-ration.

### DC-037-3 (revised): Option A — Aligned scalar set

Five energy scalars plus `plateau_count`:

| Field                  | Definition                         | Source   |
| ---------------------- | ---------------------------------- | -------- |
| `energy`               | $\sum_{\text{basis}} R.\text{sum}$ | §CR.6    |
| `exact_energy`         | `range_sum(start..end)`            | §CR.13   |
| `plateau_energy`       | $\sum_{j=s}^{e} E(P_j)$            | §CR.10.4 |
| `cross_plateau_energy` | `energy - plateau_energy`          | §CR.10.4 |
| `plateau_count`        | Number of constituent plateaus     | §CR.1    |

Dropped: `interior_energy`, `thatched_energy`. These were artifacts
of the three-set partition and have no counterpart in the revised
spec.

### DC-037-4 (new): Option A — Add `select_plateaus`

New public method on `GvGraph`:

```rust
/// Plateau selection (§CR.12): given an arbitrary dyadic range
/// `[lo, hi)`, find the contiguous run of overlapping plateaus
/// and return their lattice-aligned endpoints.
///
/// The returned pair `(start, end)` are valid `BasisEdge`s for
/// `contour_range()` and `contour_range_energy()`.
///
/// Returns `None` if the plateau map is empty (no observations)
/// or if `lo >= hi`.
///
/// Cost: O(log P) — two lookups in the plateau ordered map.
pub fn select_plateaus(
    &self,
    lo: C,
    hi: C,
) -> Option<(BasisEdge<C>, BasisEdge<C>)>;
```

Rationale:

- §CR.12 is the natural entry point: callers have coordinates, not
  `BasisEdge` values.
- The method composes with `contour_range()`:
  `select_plateaus(l, r)` → `contour_range(start, end)`.
- Cost is $O(\log P)$ — negligible alongside the $O(N)$
  decomposition.

### DC-037-5 (revised): Option A — Keep `ContourRangeEnergy`, updated

The `ContourRangeEnergy` struct is retained with the new scalar set
from DC-037-3. It remains a `Copy` type.

```rust
pub fn contour_range_energy(
    &self,
    start: BasisEdge<C>,
    end: BasisEdge<C>,
) -> Option<ContourRangeEnergy<V>>;
```

The implementation may initially delegate to `contour_range()` and
discard the `Vec`, with a dedicated non-allocating path added later
if profiling warrants.

### DC-037-6 (unchanged): Option B — Return `None` for invalid endpoints

Unchanged from the original ADR. Stale endpoints are a timing
issue; returning `None` lets callers retry after re-reading the
plateau map.

---

## Types

### `BasisElement<C, V>` — Single basis element (Surface 1, Print)

Replaces `InteriorElement`, `BoundaryElement`, and
`StraddlingAncestor`.

```rust
/// A basis element of a contour range decomposition (§CR.2).
///
/// Part of the minimal G-node cover of `[start, end)`.  Every
/// basis element's effective tile is pairwise disjoint from every
/// other's (CR-I3) and their union covers the range (CR-I2).
///
/// At most two basis elements in any contour range are boundary
/// thatching semi-internals (§CR.3.2), flagged by
/// `is_boundary_thatch`.  Their `.sum` includes energy from
/// territory outside the range — via the surviving child (boundary
/// selection) or the present child (early selection, §CR.8.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BasisElement<C: Coordinate, V: Accumulator> {
    /// Arena handle of the backing G-node.
    pub gnode_id: GNodeId,
    /// Coverage tile start (inclusive) — the portion of the range
    /// this element is responsible for (§CR.2.2).
    ///
    /// For fully-contained elements: `g.lo`.
    /// For boundary/early-selected semi-internals: clipped to the
    /// query range intersection.
    ///
    /// The invariants CR-I2 (complete cover) and CR-I3 (disjointness)
    /// are stated in terms of these coverage tiles, not the backing
    /// G-node's full interval.
    pub start: C,
    /// Coverage tile end (exclusive) — see `start`.
    pub end: C,
    /// Direct accumulation (`g.own`).
    pub own: V,
    /// Total accumulation (`g.sum`).
    pub sum: V,
    /// G-Tree depth.
    pub depth: u32,
    /// `true` if this is a boundary thatching semi-internal (§CR.3.2).
    ///
    /// At most two per contour range — one at each boundary.  When
    /// `true`, this element's `.sum` includes energy from outside
    /// `[range.start, range.end)`.  The out-of-range energy enters
    /// via the surviving child (boundary selection) or via the
    /// present child's territory beyond the range (early selection,
    /// §CR.8.1).
    pub is_boundary_thatch: bool,
}
```

### `ContourRange<C, V>` — Full decomposition (Surface 1, Print)

```rust
/// A contour-range decomposition of the G-Tree over a lattice-aligned
/// half-open interval (§CR.2–§CR.6).
///
/// Contains the basis set (§CR.2) and pre-computed energy fields
/// (§CR.6, §CR.13).  Obtain one by calling
/// [`GvGraph::contour_range()`](crate::GvGraph::contour_range).
///
/// Not `Copy` — contains a `Vec`.  Derives `Clone`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContourRange<C: Coordinate, V: Accumulator> {
    /// Range start (inclusive).  Lies on the endpoint lattice.
    pub start: C,
    /// Range end (exclusive).  Lies on the endpoint lattice.
    pub end: C,

    // ── Basis set (§CR.2) ───────────────────────────────────────

    /// The basis set: minimal G-node cover of `[start, end)`,
    /// in spatial order (sorted by `start`).
    ///
    /// Size: `≤ 2N`.  Elements with `is_boundary_thatch == true`
    /// number at most 2 (§CR.3.2, CR-I8).
    pub basis: Vec<BasisElement<C, V>>,

    // ── Energy (§CR.6, §CR.13) ──────────────────────────────────

    /// Contour range energy (§CR.6): `Σ basis[i].sum`.
    ///
    /// Includes boundary thatching — semi-internal basis elements'
    /// `.sum` values leak energy outside the range.  This matches
    /// the single-plateau energy definition.
    pub energy: V,

    /// Exact energy (§CR.13): `range_sum(start..end)`.
    ///
    /// Pro-rates G-nodes at the boundary under the uniform-within-
    /// cell assumption.  **Not generally ordered** relative to
    /// `energy` (§CR.13.6): the gap is $B - A$ where $B$ is
    /// boundary thatching leakage and $A$ is ancestor pro-ration,
    /// both non-negative under P1 but with indeterminate net sign.
    pub exact_energy: V,

    /// Sum of individual plateau energies within the range (§CR.10.4).
    pub plateau_energy: V,

    /// Cross-plateau energy (§CR.10.4): `energy - plateau_energy`.
    ///
    /// Sign is **indeterminate** under P1 (§CR.10.4).  Positive
    /// when ancestor `.own` energy gained by consolidation exceeds
    /// resolved thatching; negative when resolved thatching
    /// dominates; zero when the two cancel or no interior effects
    /// exist.
    pub cross_plateau_energy: V,

    /// Number of constituent plateaus.
    pub plateau_count: usize,
}
```

### `ContourRangeEnergy<V>` — Energy-only result (Surface 1, Print)

```rust
/// Energy-only result of a contour range query (§CR.6, §CR.13).
///
/// Lightweight `Copy` alternative to [`ContourRange`] that carries
/// only the scalar energy fields, not the basis set.
// NOTE: `Eq` is derived here (unlike `ContourRange`, which derives
// only `PartialEq`) because this struct contains no `Vec`.  For
// float accumulators (`f32`, `f64`), Rust's orphan rules prevent
// `Eq` from being generated — the derive silently becomes a no-op
// when `V: !Eq`.  The asymmetry is intentional.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContourRangeEnergy<V: Accumulator> {
    /// Contour range energy (§CR.6): `Σ basis[i].sum`.
    pub energy: V,
    /// Exact energy (§CR.13): `range_sum(start..end)`.
    pub exact_energy: V,
    /// Sum of individual plateau energies.
    pub plateau_energy: V,
    /// Cross-plateau energy (§CR.10.4): `energy - plateau_energy`.
    pub cross_plateau_energy: V,
    /// Number of constituent plateaus.
    pub plateau_count: usize,
}
```

---

## Algorithm

### Basis decomposition (§CR.8.1)

The decomposition is a recursive G-Tree descent. It collects the
basis set directly — no three-way classification, no pro-ration.

```
function contour_range_basis(g_root, a_s, a_{e+1}) → basis:
    basis ← []
    decompose(g_root, a_s, a_{e+1}, basis)
    return basis

function decompose(g, start, end, basis):
    if g = null or g.r ≤ start or g.l ≥ end:
        return                              // disjoint

    // Fully contained — basis element
    if start ≤ g.l and g.r ≤ end:
        basis.push(g, is_boundary_thatch=false)
        return

    m ← midpoint(g.l, g.r)

    // Semi-internal early selection guard (§CR.8.1, §CR.8.4).
    // When a semi-internal's present and absent halves BOTH overlap
    // the range, select the node directly.  Without this, recursion
    // into the present child would select descendants whose .sum is
    // already included in this node's .sum (G-I1), violating CR-I4.
    left_absent  ← (g.left = null)
    right_absent ← (g.right = null)
    if left_absent ≠ right_absent:              // exactly one absent
        left_overlaps  ← max(g.l, start) < min(m, end)
        right_overlaps ← max(m, start) < min(g.r, end)
        if left_overlaps and right_overlaps:
            basis.push(g, is_boundary_thatch=true)
            return

    // Left child: recurse if present, else semi-internal detection
    if g.left ≠ null:
        decompose(g.left, start, end, basis)
    else if max(g.l, start) < min(m, end):
        // Semi-internal: uncovered left half overlaps range.
        // This is a basis element with boundary thatching.
        basis.push(g, is_boundary_thatch=true)
        return

    // Right child: recurse if present, else semi-internal detection
    if g.right ≠ null:
        decompose(g.right, start, end, basis)
    else if max(m, start) < min(g.r, end):
        // Defensive: g ∉ basis is always true here because the left
        // semi-internal branch returns after pushing.  The only node
        // type that could reach both else-if arms is a terminal (both
        // children null), but the return prevents it.  Assert retained
        // for safety against future refactors that remove the return.
        debug_assert(g ∉ basis)
        basis.push(g, is_boundary_thatch=true)
```

**Key differences from the original algorithm:**

1. **No straddling ancestor classification.** G-nodes that partially
   overlap but have children are traversed through those children.
   Only semi-internal nodes with uncovered halves in-range become
   basis elements (with thatching). Fully-terminal partial-overlap
   nodes are not basis elements of a lattice-aligned contour range —
   their boundaries are already lattice-aligned and handled by the
   recursion.

2. **Semi-internal early selection guard (§CR.8.1, §CR.8.4).**
   When a semi-internal node is not fully contained but both its
   present-child half and absent-child half overlap the range, the
   algorithm selects it directly and returns — preventing recursion
   into the present child. Without this guard, the present-child
   recursion would select descendants that are also subsumed by the
   semi-internal's `.sum` (via G-I1), producing an ancestor–descendant
   pair in the basis that violates CR-I4 and double-counts energy.
   The guard fires only for multi-plateau ranges (§CR.8.4 proves it
   cannot fire for single-plateau ranges, preserving CR-I7).
   Early-selected nodes are marked `is_boundary_thatch=true` because
   their `.sum` covers territory outside the range.

### Energy computation

```
energy       = Σ basis[i].sum               // §CR.6
exact_energy = range_sum(g_root, a_s, a_{e+1})  // §CR.13
```

Both are $O(N)$. The `range_sum` call reuses the existing
implementation unchanged (ADR-M-020).

### Plateau selection (§CR.12)

```
function select_plateaus(l, r) → (BasisEdge(a_s), BasisEdge(a_{e+1})):
    s ← plateau_map.floor_index(l)          // largest a_j ≤ l
    e ← plateau_map.floor_index(r - 1)      // largest a_j ≤ r - 1
    a_s ← plateau_map.key_at(s)
    a_{e+1} ← plateau_map.key_at(e + 1)     // or 2^N if e+1 = P
    return (BasisEdge(a_s), BasisEdge(a_{e+1}))
```

$O(\log P)$.

> _Note (floating-point coordinates)._ `floor_index(r - 1)` assumes
> integer coordinates where $r - 1$ is the last included point.
> For floating-point `C`, the subtraction is not meaningful in the
> dyadic framework (§3.2.6). Floating-point implementations should
> use an exclusive-upper-bound variant `floor_index_strict_less(r)`
> returning the largest $a_j$ strictly less than $r$. See §CR.12.4
> for the full discussion.

### Shared core with `range_sum_inner`

The decision to keep separate recursive functions (not a generic
visitor) is carried forward. The basis decomposition diverges from
`range_sum_inner` even more now: no pro-ration arithmetic at all,
just basis collection and thatch flagging. The two code paths share
only the recursion skeleton.

---

## Invariant verification (debug builds)

Updated to match the revised spec invariants (§CR.7):

| Invariant                   | Check                                                                               |
| --------------------------- | ----------------------------------------------------------------------------------- |
| **CR-I2 (Complete Cover)**  | Union of basis effective tiles = `[start, end)`                                     |
| **CR-I3 (Disjointness)**    | Pairwise non-overlapping effective tiles                                            |
| **CR-I4 (Minimality)**      | No proper ancestor of any basis element also qualifies                              |
| **CR-I6 (Determinism)**     | Two calls on unmutated graph produce identical `basis` and energy fields            |
| **CR-I8 (Boundary Thatch)** | `basis.iter().filter(\|b\| b.is_boundary_thatch).count() ≤ 2`                       |
| **Energy (§CR.6)**          | `energy == basis.iter().map(\|b\| b.sum).sum()`                                     |
| **Energy gap (§CR.13.6)**   | `energy == exact_energy + B - A` (structural identity; both $B, A \geq 0$ under P1) |
| **Cross-plateau**           | `cross_plateau_energy == energy - plateau_energy`                                   |
| **CR-I7 (Consistency)**     | Single-plateau case: basis matches plateau's basis                                  |

Removed: the _old_ CR-I6 (decomposition identity with the three-set
partition) — no longer applicable. The revised spec reuses the
CR-I6 label for **Determinism** (the basis is uniquely determined by
the G-Tree state and the range). This is verified by the
`determinism` test and is included in the table above.

The new energy identity (`energy = Σ basis.sum`) is trivially true
by construction (CR-I5).

CR-I4 (minimality) can be checked in debug builds by verifying that
no basis element's parent also has its interval within `[start, end)`.
This requires parent-lookup capability, already available via
`GNodeInfo::parent`.

---

## Testing

Updated test matrix. Tests marked ★ are new or substantially
changed.

| Test                               | Validates                                                                                                                              |
| ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------- |
| `single_plateau_range`             | Range spanning one plateau: basis matches plateau's basis elements (CR-I7).                                                            |
| `full_domain_range`                | `[0, 2^N)`: single basis element (G-root), `energy == total_sum()`, `exact_energy == energy`.                                          |
| ★ `basis_is_flat_list`             | `basis` is a single `Vec`, not three separate collections. All elements are `BasisElement`.                                            |
| ★ `boundary_thatch_flag`           | At most 2 basis elements have `is_boundary_thatch == true`. They are semi-internal nodes at the range boundaries.                      |
| `two_plateau_concatenation`        | Concatenation energy consistency: each sub-range energy ≤ combined energy (nesting); discrepancy bounded (§CR.9.1).                    |
| `split_and_nesting`                | Each sub-range energy ≤ full range energy (nesting monotonicity, §CR.9.3).                                                             |
| `nesting_monotonicity`             | Wider range has ≥ energy of inner range.                                                                                               |
| ★ `energy_exact_gap`               | `energy == exact_energy + B - A`: verify the structural identity (§CR.13.6) where $B$ = boundary thatching, $A$ = ancestor pro-ration. |
| ★ `energy_equals_basis_sum`        | `energy == basis.iter().map(\|b\| b.sum).sum()` — trivially true by construction but verifies the assembly.                            |
| `invalid_endpoint_returns_none`    | Non-lattice endpoints produce `None`.                                                                                                  |
| `stale_endpoint_returns_none`      | Mutation invalidates previous endpoints → `None`.                                                                                      |
| `cross_plateau_energy_consistency` | `cross_plateau_energy == energy - plateau_energy` (§CR.10.4); sign is indeterminate.                                                   |
| `energy_only_matches_full`         | `contour_range_energy()` scalars match `contour_range()` scalars.                                                                      |
| `determinism`                      | Two calls on unmutated graph produce identical results.                                                                                |
| `empty_range_rejected`             | `start >= end` returns `None`.                                                                                                         |
| ★ `select_plateaus_basic`          | Arbitrary coordinates snap outward to lattice endpoints. Returned endpoints are valid for `contour_range()`.                           |
| ★ `select_plateaus_aligned`        | Lattice-aligned input returns unchanged endpoints.                                                                                     |
| ★ `select_plateaus_single_plateau` | Both coords in same plateau → single-plateau contour range.                                                                            |
| ★ `select_plateaus_full_domain`    | `select_plateaus(0, 2^N)` → full domain contour range.                                                                                 |
| ★ `select_plateaus_compose`        | `select_plateaus(l, r)` → `contour_range(start, end)` round-trip succeeds.                                                             |
| ★ `no_straddling_ancestors`        | Contour range has no pro-rated fractional elements — all contributions are whole `.sum` values.                                        |

---

## Migration

### Types removed (pre-1.0 breaking change)

| Type                       | Replacement                                               |
| -------------------------- | --------------------------------------------------------- |
| `InteriorElement<C, V>`    | `BasisElement<C, V>` with `is_boundary_thatch == false`   |
| `BoundaryElement<C, V>`    | `BasisElement<C, V>` with `is_boundary_thatch == true`    |
| `StraddlingAncestor<C, V>` | Removed entirely. No equivalent in contour range context. |

### Fields removed on `ContourRange`

| Field             | Migration                                             |
| ----------------- | ----------------------------------------------------- |
| `interior`        | `cr.basis.iter().filter(\|b\| !b.is_boundary_thatch)` |
| `boundary`        | `cr.basis.iter().filter(\|b\| b.is_boundary_thatch)`  |
| `straddling`      | Removed. Use `range_sum()` for pro-rated energy.      |
| `interior_energy` | Removed. Compute from basis if needed.                |
| `thatched_energy` | Removed. `energy` (§CR.6) subsumes this concept.      |

### Fields renamed/redefined on `ContourRange`

| Field    | Old meaning                            | New meaning                                                      |
| -------- | -------------------------------------- | ---------------------------------------------------------------- |
| `energy` | `range_sum(start..end)` (exact energy) | $\sum_{\text{basis}} R.\text{sum}$ (contour range energy, §CR.6) |

### Fields added

| Field          | Meaning                                                  |
| -------------- | -------------------------------------------------------- |
| `exact_energy` | `range_sum(start..end)` (§CR.13) — what old `energy` was |
| `basis`        | `Vec<BasisElement<C, V>>` — the flat basis set           |

### Re-exports in `lib.rs`

Old:

```rust
pub use contour_range::{
    BoundaryElement, ContourRange, ContourRangeEnergy,
    InteriorElement, StraddlingAncestor,
};
```

New:

```rust
pub use contour_range::{BasisElement, ContourRange, ContourRangeEnergy};
```

---

## Implementation

### File changes in `packages/mudlark/`

| File                     | Change                                                                                                                                                                                                                                   | Surface     |
| ------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------- |
| `src/contour_range.rs`   | Replace five types with three (`BasisElement`, `ContourRange`, `ContourRangeEnergy`). Update `validate_endpoints`, `compute_plateau_energy`. Update debug invariant assertions.                                                          | 1           |
| `src/graph_query.rs`     | Rewrite `decompose_inner` to collect `Vec<BasisElement>` (no three-way classification). Add semi-internal early selection guard (§CR.8.1). Add `select_plateaus()`. Compute `energy` as `Σ basis.sum`, `exact_energy` via `range_sum()`. | 2           |
| `src/lib.rs`             | Update re-exports: remove `InteriorElement`, `BoundaryElement`, `StraddlingAncestor`; add `BasisElement`.                                                                                                                                | 1           |
| `tests/contour_range.rs` | Rewrite tests to use `.basis` and `.is_boundary_thatch`. Add new tests for `select_plateaus`, energy ordering, thatch flag.                                                                                                              | (test-only) |

### Trait bounds

Methods require `V: Accumulator + Proratable + Inspectable`:

- `Proratable` for `range_sum()` (exact energy computation).
- `Inspectable` for plateau access via `self.plateaus()`.
- `Accumulator` for `V::zero()`, `V::add()`.

`select_plateaus` requires only `V: Accumulator + Inspectable` (no
pro-ration). Note: api.md currently lists `V: Proratable +
Inspectable` for `select_plateaus` — this is an over-strict bound
that should be corrected to `V: Inspectable` (plateau access is all
that is needed).

---

## Alternatives considered

### A1 — Keep three-set partition (status quo)

Retain `InteriorElement`, `BoundaryElement`, `StraddlingAncestor`.

**Rejected** because the revised spec (§CR.1–§CR.13) defines a
single basis set, not a three-set partition. Keeping the old model
creates a permanent divergence between spec and implementation,
making the spec's invariants and algebra untestable.

### A2 — Accept arbitrary `RangeBounds<C>` in `contour_range`

Let `contour_range` accept `a..b` for any `a`, `b` and snap or clamp
to lattice points internally.

**Rejected** (carried forward from original ADR). Contour ranges
are defined only for lattice-aligned endpoints (CR-I1). Silently
snapping violates the decomposition identity. `select_plateaus`
(DC-037-4) provides the explicit snapping step.

### A3 — Omit `exact_energy` from the struct

Return only `energy` (§CR.6) and let callers call `range_sum()`
separately.

**Rejected** because the structural identity
$E = \text{exact} + B - A$ (§CR.13.6) is a key diagnostic — it
exposes the interplay of boundary thatching and ancestor
pro-ration. The cost of computing both in the same call is
marginal.

### A4 — Return `Vec<GNodeId>` instead of `Vec<BasisElement>`

Let callers use `gnode_info()` to get details.

**Rejected** because it requires $O(|\text{basis}|)$ additional
lookups, breaks the snapshot property, and loses the
`is_boundary_thatch` classification that the caller would have to
recompute.

### A5 — Add `is_lattice_point` convenience method

**Deferred** (carried forward). Callers can check via
`plateaus().contains_key(&BasisEdge(coord))`.

---

## Consequences

- **Surface 1 drops three types, gains one:** `InteriorElement`,
  `BoundaryElement`, `StraddlingAncestor` → `BasisElement`.
  `ContourRange` and `ContourRangeEnergy` are retained with updated
  fields. Net Surface 1 type count: 3 (was 5).

- **Surface 2 gains one method:** `select_plateaus()`.
  `contour_range()` and `contour_range_energy()` retain their
  signatures but their return types change.

- **Energy semantics are corrected.** `energy` now means
  $\sum_{\text{basis}} R.\text{sum}$ (§CR.6), not `range_sum`.
  Old callers who wanted `range_sum` use `exact_energy`.

- **`range_sum` is unaffected.** The existing scalar range query
  retains its current signature, performance, and trait bounds.

- **Spec alignment.** All §CR cross-references now point to the
  correct sections in the revised spec (§CR.1–§CR.13).

- **Pre-1.0 breaking change.** This removes three public types and
  restructures two public structs. No external consumers use these
  types (sentinel does not import them). The migration is contained
  within `packages/mudlark/`.

- **Future path: algebraic operations.** Concatenation (§CR.9.1) and
  splitting (§CR.9.2) remain deferred. The simpler `ContourRange`
  struct makes future algebra methods easier to add — they operate on
  the basis set directly rather than juggling three sets.
