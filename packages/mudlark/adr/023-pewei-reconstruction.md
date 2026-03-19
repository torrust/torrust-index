# ADR-M-023: PEWEI Reconstruction

**Status:** Decided  
**Date:** 2026-02-26  
**Phase:** 4 (Output)  
**Relates to:** [ADR-M-008](008-span-type.md) (Span type),
[ADR-M-012](012-g-sum-recomputation.md) (G-sum recomputation),
[ADR-M-021](021-pewei-output-representation.md) (output representation),
[ADR-M-022](022-pewei-serialisation.md) (serialisation)  
**API:** [§API M-4.2](../docs/api.md#42-contact-print)
(`Pewei::reconstruct`),
[§API M-5.3](../docs/api.md#accumulator)
(`Accumulator::sub`)  
**Surface:** 1 + 3 (Prints + Emulsion)

## Context

Given a `Pewei<C, V>` truncated at layer $k$, `reconstruct()`
(§PEWEI M-10) produces a signal estimate over $[0, 2^N)$.
Each truncation point preserves total energy exactly (G-I1).
Deeper layers refine the estimate by adding confirmed sub-scale
structure on top of coarser baselines.

Three sub-decisions:

1. What type does `reconstruct()` return?
2. What algorithm produces the output?
3. How does the algorithm obtain the remainder energy for the
   one-visible-one-invisible case?

### Background: the additive property

The G-Tree is an adaptive Haar wavelet decomposition (§PEWEI M-5).
Each phase-transition node's `baseline` (= `g.own`) is a
**scaling coefficient**: the energy accumulated at that scale
_before_ children were created. Children's contributions are the
**detail coefficients**: they add finer-scale structure on top.

G-I1 gives the accounting identity:

$$g.\text{sum} = g.\text{own} + \sum_{c \in \text{children}(g)} c.\text{sum}$$

Reconstruction is **additive across scales, not replacement.** A
transition's `baseline` is a uniform background that persists even
when children refine the region — it is _supplemented_, not
_replaced_. Using `total` (= `g.sum`) is correct only when no
children are visible (truncated). See §PEWEI M-10.1–§PEWEI M-10.2
for the full argument and counter-example.

### Baseline pro-ration

When descending from a region $[l, r)$ to its two halves, each
child inherits a **proportional share** of the accumulated
background, not the full amount. For dyadic splits this is
`prorate(1, 2)` — half the background goes to each child.

Example: root `[0,8)` has `baseline=10`, accumulated `0`.
`total_bg = 10`. Each half gets `prorate(1,2) = 5`.
Left child `intensity=15` → span intensity `5 + 15 = 20`.
Right child `intensity=8` → span intensity `5 + 8 = 13`.
Grand total `20 + 13 = 33 = root.total`. ✓

Without pro-ration, each child would receive the full `10`,
inflating the total to `10 + 15 + 10 + 8 = 43`. ✗

### The one-visible-one-invisible case

When a transition is expanded (at least one G-child visible in the
PEWEI at ≤ `max_layer`) but only _one_ child is visible, the
invisible sibling's energy can be computed from G-I1:

$$\text{remainder} = \text{total} - \text{baseline} - \text{visible\_child.total}$$

This requires **subtraction** on the value type. The `Accumulator`
trait (at the time of this decision) provided `add`, `prorate`,
`to_f64`/`from_f64`, but not `sub` — ADR-M-012 deliberately avoided
it on the observe hot-path. However, reconstruction operates on the
extracted `Pewei<C, V>` (offline, not the live graph), and remainder
computation is the natural inverse of G-I1.

### Domain bounds on `Pewei`

The reconstruction algorithm starts at the root region $[0, 2^N)$.
The current `Pewei<C, V>` struct (ADR-M-021) stores only
`layers: Vec<Layer<C, V>>` — it has no record of the domain.
Inferring the root from the widest region in layer 0 is fragile
and fails for empty PEWEIs. The domain should be stored explicitly.

## Questions

### Q1: Reconstruction output type (DC-023-1)

| Option | Type                   | Notes                                                                                                                                                                                |
| ------ | ---------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| A      | `Vec<Span<C, V>>`      | Ordered, non-overlapping spans covering $[0, 2^N)$. Each span carries the **fully-summed intensity** (all ancestral baselines + node value). Simple, composable, iterable, testable. |
| B      | `impl Fn(C) -> V`      | Point-query closure. Lazy. But opaque — can't iterate, inspect, serialise, or compare.                                                                                               |
| C      | `Signal<C, V>` wrapper | Methods for `query(coord)`, `iter()`, `total_energy()`. Premature if it's just a `Vec<Span>` with accessors.                                                                         |
| D      | `Vec<(C, C, V)>`       | Bare tuple. Re-invents `Span` without methods.                                                                                                                                       |

### Q2: Reconstruction algorithm (DC-023-2)

| Option | Strategy                                                                                                                                                                                                                                                                                         | Notes                                                                                                                      |
| ------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------- |
| A      | **Top-down recursive.** Build `(g_depth, start) → node` lookup. Recurse from root, carrying pro-rated accumulated baseline and G-depth. Expanded → pro-rate baseline, recurse into children at `g_depth + 1`. Truncated → emit `accumulated + total`. Terminal → emit `accumulated + intensity`. | Natural for the additive model. Correct by construction. $O(L + T)$ — each visible node and each output span visited once. |
| B      | **Layer-order additive.** Process layers coarsest-to-finest, tracking an accumulated-background map keyed by region.                                                                                                                                                                             | Equivalent to A but unrolled. More bookkeeping (explicit background map ≈ the recursion stack of A).                       |
| C      | **Build mini G-Tree.** Materialise a tree from PEWEI nodes, then walk it.                                                                                                                                                                                                                        | Correct but over-built. A depth-bucketed lookup suffices.                                                                  |

### Q3: Remainder subtraction (DC-023-3)

How to compute `total − baseline − visible_child.total` for the
one-visible-one-invisible case.

| Option | Approach                                                 | Trait change?   | Notes                                                                                                              |
| ------ | -------------------------------------------------------- | --------------- | ------------------------------------------------------------------------------------------------------------------ |
| A      | Add `fn sub(self, other: Self) -> Self` to `Accumulator` | Yes — all impls | Exact for all types. Natural inverse of `add`. Useful beyond reconstruction (future: delta queries, differencing). |
| B      | Via `to_f64()` / `from_f64()` round-trip                 | No              | Exact for values < $2^{53}$. Introduces float rounding for large integers.                                         |
| C      | Treat partially-expanded as fully truncated              | No              | Preserves total energy. Loses sub-region detail for one transition at the truncation boundary.                     |

## Decision

### DC-023-1: Option A — `Vec<Span<C, V>>`

```rust
impl<C: Coordinate, V: Accumulator> Pewei<C, V> {
    pub fn reconstruct(&self, max_layer: usize) -> Vec<Span<C, V>>;
}
```

`Span<C, V>` already exists (ADR-M-008). Each output span carries the
fully-summed intensity at that region — all ancestral baselines
pro-rated and folded in. The vec covers $[0, 2^N)$ with no gaps and
no overlaps: a partition of the domain. Callers need no knowledge of
the wavelet structure.

Options B–D rejected:

- B is opaque (can't iterate, serialise, or test).
- C is a premature wrapper — add it later if real API pressure
  emerges.
- D re-invents `Span` without its methods.

### DC-023-2: Option A — top-down recursive with pro-rated baseline accumulation

The algorithm follows §PEWEI M-10.3:

```rust
fn reconstruct(&self, max_layer: usize) -> Vec<Span<C, V>> {
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
    let mut output = Vec::new();
    descend::<C, V>(
        &lookup, visible,
        self.domain_start, self.domain_end, 0,
        V::zero(), &mut output,
    );
    output
}

fn descend<C: Coordinate, V: Accumulator>(
    lookup: &RegionLookup<C>,
    layers: &[Layer<C, V>],
    start: C,
    end: C,
    g_depth: usize,
    accumulated: V,           // ancestral background, pro-rated to this region
    output: &mut Vec<Span<C, V>>,
) {
    match lookup.get(start, g_depth) {
        None => {
            // Gap — no PEWEI node here. Emit background only.
            output.push(Span {
                start,
                end,
                intensity: accumulated,
                depth: 0,
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

            let left_ref  = lookup.get(start, child_depth);
            let right_ref = lookup.get(mid, child_depth);

            match (left_ref, right_ref) {
                (Some(_), Some(_)) => {
                    descend(lookup, layers, start, mid, child_depth,
                            half_bg, output);
                    descend(lookup, layers, mid, end, child_depth,
                            half_bg, output);
                }
                (Some(left_nr), None) => {
                    descend(lookup, layers, start, mid, child_depth,
                            half_bg, output);
                    let left_total = node_total(layers, left_nr);
                    let remainder = V::sub(
                        V::sub(total, baseline),
                        left_total,
                    );
                    output.push(Span {
                        start: mid,
                        end,
                        intensity: V::add(half_bg, remainder),
                        depth,
                    });
                }
                (None, Some(right_nr)) => {
                    let right_total = node_total(layers, right_nr);
                    let remainder = V::sub(
                        V::sub(total, baseline),
                        right_total,
                    );
                    output.push(Span {
                        start,
                        end: mid,
                        intensity: V::add(half_bg, remainder),
                        depth,
                    });
                    descend(lookup, layers, mid, end, child_depth,
                            half_bg, output);
                }
                (None, None) => {
                    // Fully truncated — emit total
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
```

**Energy conservation proof:** At every transition:

- **Both visible:** background `total_bg` is split into
  `half_bg + half_bg`. Children's recursive totals add up to
  `refinement = total − baseline` (by G-I1). Grand total for
  region = `total_bg + refinement = accumulated + total`. ✓
- **One visible, one not:** visible child recurses with `half_bg`.
  Invisible sibling gets `half_bg + remainder` where
  `remainder = refinement − visible_child.total`. By G-I1,
  `visible.total + remainder = refinement`. Grand total =
  `total_bg + refinement = accumulated + total`. ✓
- **None visible:** emit `accumulated + total` directly. ✓

All three cases yield `accumulated + total` for the region.

**Recursion depth:** bounded by the maximum G-Tree depth in the
PEWEI (≤ $N$). For typical usage, depths are 10–30.

**`RegionLookup`:** a depth-bucketed lookup table keyed by
`(g_depth, start)`. Within a given G-depth, dyadic intervals never
share a start coordinate, so the pair is a unique key (§PEWEI M-10.3). Built in $O(n \log n)$ from the visible layers
(one sort per depth bucket). Each entry is an index-based reference
(`NodeRef`) into the layers slice — no field values are copied
into the table. The implementation uses a Structure-of-Arrays
layout: each depth bucket stores `starts` and `nodes` as separate
parallel arrays, with binary search on the `starts` array only.
At GB-scale PEWEIs this keeps the search working set ~3× smaller
than an interleaved layout. The `node_total` helper resolves a
`NodeRef` to the node's total energy (transitions: `total` field;
terminals: `intensity` field) for remainder computation.

### DC-023-3: Option A — add `Accumulator::sub`

```rust
pub trait Accumulator: Copy + PartialOrd + Debug + Default + Send + Sync + 'static {
    // ... existing methods ...

    /// Same-type subtraction. Used by PEWEI reconstruction
    /// (remainder energy) and future delta/differencing operations.
    ///
    /// Callers must ensure `self >= other` for unsigned types.
    /// Default overflow behavior matches `add`: panic in debug,
    /// wrap in release.
    #[must_use]
    fn sub(self, other: Self) -> Self;
}
```

Blanket implementations for all existing types:

```rust
// Unsigned integers:
fn sub(self, other: Self) -> Self { self - other }

// f32, f64:
fn sub(self, other: Self) -> Self { self - other }
```

Trivial: one line per impl, mirrors `add` exactly. All current
`Accumulator` types (`u8`–`u128`, `f32`, `f64`) support native
subtraction.

Options B and C rejected:

- B (`to_f64` round-trip) is lossy for `u64` values > $2^{53}$
  and for `u128`. Introduces unnecessary float arithmetic into an
  integer-typed pipeline.
- C (treat as truncated) loses sub-region detail. While the total
  energy is preserved, it forfeits information that was already
  paid for in the PEWEI. The one-visible-one-invisible case is
  common at truncation boundaries and matters for rendering.

**Relationship to ADR-M-012:** ADR-M-012 decided against subtraction on
the _observe hot-path_ because recomputation from children is
self-healing under float drift. That argument does not apply to
reconstruction — which is offline, operates on an immutable
snapshot, and needs the natural inverse of G-I1 exactly once per
partially-expanded transition. Adding `sub` to the trait has zero
impact on the observe path.

### DC-023-4: Store domain bounds on `Pewei`

```rust
pub struct Pewei<C: Coordinate, V: Accumulator> {
    pub domain_start: C,    // C::zero()
    pub domain_end: C,      // C::domain_max(N)
    pub layers: Vec<Layer<C, V>>,
}
```

Free to populate during `extract()` — just `C::zero()` and
`C::domain_max(N)`. Makes the snapshot self-contained for
reconstruction, serialisation (ADR-M-022), and round-tripping
without needing the original `GvGraph` or its `N` const generic.

### Integer rounding note

For integer `Accumulator` types, `prorate(1, 2)` performs
truncating division: `11.prorate(1, 2) = 5`. Both halves
receive 5, losing 1 unit of background per odd baseline per
level. Over a reconstruction of depth $d$, each baseline can
lose up to 1 unit, for a cumulative worst-case loss of $d$
units across the entire domain.

This is acceptable:

- For `u64` values, $d \leq 64$. A 64-unit rounding error on
  a total that reached split threshold is negligible.
- Reconstruction from a truncated PEWEI is already an
  approximation (detail below the truncation point is lost).
  Integer rounding adds noise far below that truncation error.
- Float types (`f32`, `f64`) do not truncate — they round to
  nearest, which is self-cancelling in expectation.

Document the bound in a doc-comment on `reconstruct()`. No
mitigation needed.

## Consequences

- `reconstruct()` returns `Vec<Span<C, V>>` — reuses ADR-M-008,
  zero new types, fully composable output.
- Top-down recursive algorithm follows the additive wavelet
  model. Correct by construction. $O(n)$ in visible nodes.
- `Accumulator::sub` is added — one line per impl, mirrors `add`.
  Unlocks remainder computation for the one-visible-one-invisible
  case and future delta/differencing operations.
- `Pewei` gains `domain_start`/`domain_end` — self-contained
  snapshot, no dependency on `GvGraph` or `N` for reconstruction.
- Integer rounding from `prorate(1, 2)` is bounded by tree depth
  and documented. No mitigation required.
- The implementation uses a depth-bucketed `RegionLookup` keyed
  by `(g_depth, start)` with a Structure-of-Arrays layout, built
  once per `reconstruct()` call. Cost: $O(n \log n)$ build
  (sort per depth bucket) + $O(n)$ traversal.
