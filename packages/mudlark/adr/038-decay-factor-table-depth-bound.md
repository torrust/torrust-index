# ADR-M-038: Decay Factor Table Depth Bound

**Status:** Implemented  
**Date:** 2026-03-20  
**Phase:** 4 (Output)  
**Relates to:** [ADR-M-024](024-decay-semantics.md) (decay semantics),
[ADR-M-017](017-dynamic-depth-control.md) (dynamic depth control)  
**Spec:** §IDEA M-14.5 (filter bank), §IDEA M-3.2 (coordinate domains)  
**API:** [§API M-5.2](../docs/api.md#temporal-decay) (decay)  
**Surface:** 2 (Film)

---

## Context

### The bug

`decay_selective()` panics with an index-out-of-bounds error when
applied to trees containing G-nodes at G-tree depths beyond N.

```
index out of bounds: the len is 5 but the index is 5
  at packages/mudlark/src/decay.rs (factor table access)
```

The selective path precomputes a per-depth factor table whose size
is `N - d_root + 1`. During the DFS walk, each node looks up its
factor via `factors[d_local]` where `d_local = gnode_depth(gid) -
d_root`. If any node has G-tree depth greater than N, the index
exceeds the table length and the program panics.

**Not related to infinite attenuation.** Finite attenuation with
`q > 0` triggers the same panic on any f64 graph whose structure
extends beyond depth N.

### Why nodes exist beyond depth N

The G-tree depth of a node is computed from its interval width:

$$d = N - \lfloor\log_2(\text{hi} - \text{lo})\rfloor$$

For integer coordinates (`u8`–`u128`), a unit-width cell has
`width = 1`, so `log₂(1) = 0` and max depth is N. The midpoint
of a unit cell equals `lo`, which fails the `midpoint > lo` guard
in `attempt_split`, preventing further subdivision. Integer trees
cannot exceed depth N.

For floating-point coordinates (`f32`, `f64`), subdivision can
continue past depth N: `midpoint(0.0, 1.0) = 0.5 > 0.0`, so a
width-1 cell at depth N splits into two width-0.5 cells at depth
N+1. The only gate is the V-tree depth check:

```rust
// split.rs
if v_depth(&graph.vnodes, entry_id) > graph.live_depth_create {
    return;
}
```

This is a V-tree depth gate, not a G-tree depth gate. V-tree
depth can diverge from G-tree depth due to structural 2–3 node
wrapping. With `depth_evict = 8` and `N = 4`, G-nodes at depths
5, 6, and 7 readily exist.

`Coordinate::is_final()` does return `depth >= N` for floats,
but **nothing in `attempt_split` calls `is_final`**. Its only
callers are in the eviction path and diagnostic routines.

### The incorrect assumption in ADR-M-024

ADR-M-024 §DC-024-4 states:

> **Implementation:** precompute a table of D+1 factors. Cost: D+1
> calls to `exp`, amortised over the subtree walk. For N ≤ 64,
> the table fits on the stack.

"D" here is `N - d_root`, assuming the maximum G-tree depth
relative to the subtree root is bounded by N. The per-depth
factor formula:

$$\ln\lambda(d) = \ln(\text{att}) \cdot \left(1 + q \cdot \left(\frac{2d_{\text{local}}}{D} - 1\right)\right)$$

uses `D = N - d_root` as the depth range denominator. This produces
exactly the right midpoint-pivot, rolloff shape, and composition
properties — but only when the tree lives within `[0, N]`.

### Scope

- **Selective path only.** The uniform path (`q = 0`) does not use
  a factor table.
- **Float coordinates only.** Integer coordinates cap naturally at
  depth N (a unit-width cell cannot subdivide further).
- **Pre-existing.** Present since the selective path was introduced;
  masked by test suites that used integer coordinates or `q = 0`.

---

## Questions

### Q1: Where should the depth range come from? (DC-038-1)

The current code uses `N` as the conceptual maximum G-tree depth.
If the actual tree extends deeper, the factor table is too short.

| Option | Depth range                        | Pros                                                                          | Cons                                                                                                                                 |
| ------ | ---------------------------------- | ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------ |
| **A**  | Scan the subtree to find max depth | Always correct for any tree shape.                                            | Extra O(S) pass, though the DFS already visits every node anyway.                                                                    |
| **B**  | Use `live_depth_evict - d_root`    | No extra pass; conservative upper bound.                                      | Over-allocates: up to `depth_evict` entries, most unused. Shifts the factor curve midpoint away from the actual structural midpoint. |
| **C**  | Use `N + buffer - d_root`          | Slightly tighter than B. Still O(1).                                          | `depth_buffer = depth_evict - depth_create` is a V-tree concept; mixing V and G measures is confusing.                               |
| **D**  | Clamp `d_local` at `depth_range`   | Zero-cost. No table resize. Nodes deeper than N get the maximum-depth factor. | Changes the mathematical semantics: deepest nodes share the same factor instead of interpolating further.                            |

### Q2: What should the factor curve mean beyond depth N? (DC-038-2)

The selective factor curve ($\ln\lambda(d)$, linear in depth) is
defined for the depth range `[0, D]`. Nodes at depth `D + k` for
`k > 0` fall outside this range. What factor should they receive?

| Option | Factor beyond D                                    | Interpretation                                                                                                              |
| ------ | -------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| **A**  | Extrapolate: extend the log-linear curve naturally | The selectivity slope continues. Deeper = more aggressive treatment. Preserves the "constant dB-per-octave" property.       |
| **B**  | Clamp at the depth-D factor: `att^(1+q)`           | The deepest nodes all receive the same treatment. Simpler to reason about. The curve is "flat beyond the nominal boundary." |
| **C**  | Use a separate factor formula beyond N             | Maximum flexibility. Harder to explain, harder to compose.                                                                  |

### Q3: Should splitting be capped at G-tree depth N? (DC-038-3)

An alternative fix: prevent the tree from exceeding depth N in the
first place by adding an `is_final` check to `attempt_split`.

| Option | Who changes           | Pros                                                        | Cons                                                                                            |
| ------ | --------------------- | ----------------------------------------------------------- | ----------------------------------------------------------------------------------------------- |
| **A**  | Fix `decay` only      | Decay adapts to whatever tree exists. No change to observe. | The G-tree depth exceeding N is surprising; other code may have the same assumption.            |
| **B**  | Cap splitting at N    | Eliminates the entire class of "depth > N" surprises.       | Reduces resolution for f64 trees. The V-tree depth gate allows deeper structure on purpose.     |
| **C**  | Both: cap + fix decay | Belt and suspenders.                                        | Redundant once splitting is capped. Increases code complexity for a case that no longer occurs. |

---

## Decision

### DC-038-1: Use actual max depth from a subtree scan

**Option A.** The DFS already visits every node. We restructure
the implementation into two passes:

1. **Pass 1 (DFS):** collect nodes in pre-order _and_ record the
   maximum G-tree depth encountered.
2. **Compute factor table:** `max_depth - d_root + 1` entries.
3. **Pass 2 (scale):** iterate the collected nodes, look up factors,
   scale `g.own`.
4. **Bottom-up sum recompute:** same as before.

The extra cost is one `max()` comparison per node during the
initial DFS — negligible next to the `attenuate` call per node
that follows.

### DC-038-2: Extrapolate the log-linear curve (option A)

The factor curve extends naturally beyond depth N. This preserves:

- **Constant dB-per-octave rolloff.** Depth N+1 sees dB gain
  = `(1 + q * (2(N+1-d_root)/D - 1)) × ln(att)` with the original
  `D = max_depth - d_root`.
- **Composition invariance.** Q remains idempotent under repeated
  application regardless of the depth range.
- **Midpoint at actual structural midpoint.** The pivot is at
  `d_local = max_depth/2`, centered on the tree that actually
  exists, not on the nominal N.

Clamping (option B) would create a discontinuity at depth N
where the dB slope abruptly flatlines. Extrapolation is the
natural continuation of the same log-linear family.

Note: using the _actual_ max depth as `D` rather than `N` means
the factor curve adapts to the tree's current structure. Two
otherwise-identical trees with different `depth_evict` values (and
hence different actual depths) will see slightly different factor
distributions for the same `(att, q)` call. This is intentional —
the selectivity curve should fit the tree that exists, not a
theoretical maximum.

### DC-038-3: Fix decay only (option A)

Splitting beyond depth N is intentional for floats — the V-tree
depth gate is the correct resolution limiter, and deeper structure
is valuable in high-resolution f64 domains. Adding an `is_final`
cap would artificially reduce resolution. Decay was the only
consumer that assumed depth ≤ N.

Other code paths that touch G-tree depth (invariant checks, PEWEI
extraction, contour range) do not use N as a depth bound — they
either derive depth from the interval or walk the tree
structurally. A codebase audit found no other `N - d_root`
patterns outside `decay_selective`.

---

## Consequences

- `decay_selective()` now performs two DFS passes: a read-only
  pass to collect nodes and find max depth, then a mutating pass
  to apply factors. The overhead is one extra traversal of the
  subtree — `O(S)` additional reads.

- The factor table adapts to the tree's actual depth, not `N`.
  The midpoint of the selectivity curve is at the structural
  midpoint of the subtree, giving more faithful frequency-axis
  alignment for f64 trees that extend beyond N.

- The fix is confined to `decay_selective`. No changes to
  `attempt_split`, `observe`, or any other module.

- ADR-M-024 §DC-024-4's "D+1 factors" is corrected to
  "$D_{\text{actual}} + 1$ factors, where $D_{\text{actual}}$ is
  the maximum G-tree depth within the target subtree minus
  $d_{\text{root}}$."

- Incidentally fixes infinite-attenuation integration tests that
  use f64 graphs with `depth_evict > N`, since the factor table
  is now correctly sized.

### Secondary fix: debug assertions with infinite values

Three `debug_assert!` checks in `normalize_plateaus`
(`graph_plateau.rs`) use `(a - b).abs() < 1e-9`, which fails when
both `a` and `b` are `∞` (producing NaN). Debug-only issue. The
fix: add `a == b ||` before the floating-point comparison so that
`∞ == ∞` short-circuits before the subtraction. Applied alongside
the primary fix.
