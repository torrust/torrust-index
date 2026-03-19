# ADR-M-012: G-Sum Propagation via Recomputation

**Status:** Decided  
**Date:** 2026-02-24  
**Relates to:** [ADR-M-010](010-observation-generics.md) (cross-type
observations — the motivating problem),
[ADR-M-014](014-value-absorption.md) (absorption bypasses
G-sum propagation entirely),
[ADR-M-023](023-pewei-reconstruction.md) (`Accumulator::sub` added
for reconstruction, not observe),
[ADR-M-024](024-decay-semantics.md) (decay uses recomputation)  
**Spec:** §IDEA M-5.4 (G-I1 summation invariant), §IDEA M-8 Step 4
(G-Tree sum propagation)  
**API:** [§API M-5.3](../docs/api.md#accumulator) (Accumulator
trait), [§API M-5.2](../docs/api.md#observation-mutation)
(observe pipeline)  
**Surface:** 3 (Emulsion)

## Context

Mutating the receiver's `own` value requires updating G-I1
(§IDEA M-5.4) from the receiver to the root. The spec's Step 4
(§IDEA M-8) propagates a known `V`-typed delta upward:

```
g.sum ← g.sum + Δ
```

For cross-type observations (`O: Observation<V>`, e.g. `f64 → u16`),
the V-space delta is not the raw observation — it is the truncated
result of `O::accumulate(current, delta) - current`. Computing this
would require introducing subtraction into the observe hot path.
The spec's observe flow is purely additive — all propagation uses
`+Δ`, never `-`. Adding subtraction solely for cross-type delta
extraction would violate this principle.

Five options were evaluated.

## Options Considered

### A. Add `Accumulator::sub`

A new `fn sub(self, other: Self) -> Self` method on the trait.

- Pro: Clean, trait-uniform, mirrors `add`.
- Con: Unsigned underflow semantics (panic/wrap) must be documented
  and correctly implemented by every custom `Accumulator`. The formal
  spec never subtracts accumulators on the observe path — all flows
  are additive. Permanent API commitment.

### B. Add `Sub<Output = Self>` as supertrait

Require `std::ops::Sub` on `Accumulator` and use native `-` in
`observe`.

- Pro: Zero new methods. Every primitive already satisfies it.
- Con: Supertrait infection — every custom `Accumulator` must also
  implement `Sub`, with semantics that may not be meaningful for
  the type. Exposes subtraction everywhere, not just the one call
  site. Same underflow issue as Option A.

### C. Recompute sums from children

Replace delta propagation with invariant recomputation: at each
ancestor, set `sum = own + left.sum + right.sum` (the literal
definition of G-I1).

- Pro: No subtraction. No trait changes. Cross-type works for free.
  Self-healing under floating-point drift. Directly asserts the
  invariant rather than maintaining it indirectly.
- Con: Reads sibling nodes at each level (2 extra reads vs delta
  propagation). See Performance Analysis below.

### D. Capture delta via helper function

Compute `new_own - old_own` inside a helper that returns both
values. Requires subtraction — reduces to Option A or B.

### E. Add `Observation::delta_v` to the trait

Push the delta computation into the `Observation` trait. Still
requires subtraction internally — reduces to Option A or B.

## Decision

**Option C — Recompute sums from children.**

### New function in `gtree.rs`

```rust
pub fn recompute_g_sums<C: Coordinate, V: Accumulator>(
    gnodes: &mut Arena<GNode<C, V>>,
    start: GNodeId,
) {
    let mut current = Some(start);
    while let Some(id) = current {
        let (left_sum, right_sum) = {
            let g = gnodes.get(id.index());
            let l = g.left.map_or_else(V::zero, |l| gnodes.get(l.index()).sum);
            let r = g.right.map_or_else(V::zero, |r| gnodes.get(r.index()).sum);
            (l, r)
        };
        let g = gnodes.get_mut(id.index());
        g.sum = V::add(g.own, V::add(left_sum, right_sum));
        current = g.parent;
    }
}
```

`observe()` calls `recompute_g_sums` after mutating `receiver.own`
and propagating V-sums. This replaces the delta-based Step 4 in
§IDEA M-8 with invariant recomputation. `decay()` also uses
recomputation (`recompute_g_sums_subtree` for the affected subtree,
then `recompute_g_sums` from the subtree root to the G-root) after
scaling `g.own` values (see [ADR-M-024](024-decay-semantics.md)).

The original `propagate_g_sums` (delta-based `g.sum += Δ` walk) has
been moved to a test helper — no production code path calls it
(see Consequences).

## Performance Analysis

`GNode<u64, u64>` is 48 bytes — fits within one 64-byte cache line.
All fields of a single node are co-located.

### Per-level memory access comparison

| Operation               | Delta propagation   | Recomputation        |
| ----------------------- | ------------------- | -------------------- |
| Current node            | 1 read-modify-write | 1 read + 1 write     |
| Left child              | —                   | 1 read (`sum` field) |
| Right child             | —                   | 1 read (`sum` field) |
| **Cache lines touched** | **1**               | **up to 3**          |

### Cache warmth after routing

`route_to_receiver` descends through one child at each level.
After routing completes:

- **Current node at each ancestor:** warm (visited during descent).
- **Child on the descent path:** warm (visited during descent).
- **Sibling not on the descent path:** cold.

The real extra cost is **1 cold sibling read per ancestor level**.

### Arena locality

The `Arena` is `Vec`-backed. Sibling G-nodes are allocated during
the same split operation, so they occupy adjacent slots in the `Vec`.
Adjacent 48-byte nodes share cache lines or fall within the same
hardware prefetch stride, reducing the effective miss rate.

### Depth bound

G-Tree depth is bounded by $N$ (the domain bit-width): each split
bisects the interval, and at most $N$ bisections can be made before
reaching a unit interval. In practice, trees are much shallower
because the V-Tree depth gate $D_{\text{create}}$ (D-I1, §IDEA M-7)
limits split eligibility — only nodes that have earned a shallow
V-position can refine further. Sustained, concentrated observations
are needed to drive the tree deep.

### Estimated overhead

| Scenario           | Depth | Extra cold reads | Estimated cost |
| ------------------ | ----- | ---------------- | -------------- |
| Typical            | 10–20 | 10–20            | ~100–300 ns    |
| Maximum (`N = 64`) | 64    | 64               | ~600–1300 ns   |

For context, a single `observe()` also performs:

- V-Tree sum propagation: O(V-height), separate arena, pointer-chasing.
- Possible rebalancing: ~10 V-nodes with multiple arena accesses.
- Possible splitting: multiple arena allocations, V-Tree restructuring.

These V-Tree operations dominate by an order of magnitude. The G-sum
recomputation overhead is within the noise floor of the total
observation cost.

### Self-healing property

Delta propagation compounds floating-point errors over repeated
additions. Recomputation reasserts the exact invariant at every
level on every observation. For `Accumulator` types with imprecise
arithmetic (`f32`, `f64`), recomputation is strictly more accurate.

## Consequences

- The observe hot path is subtraction-free. No `Sub` supertrait.
  `Accumulator::sub` was later added for offline reconstruction
  remainder computation ([ADR-M-023](023-pewei-reconstruction.md)),
  but the live observation pipeline never calls it — recomputation
  sidesteps the need entirely.
- Cross-type observations (`f64 → u16`) work without any special
  delta handling. `observe` mutates `own` via `Observation::accumulate`
  and calls `recompute_g_sums`. Done.
- `decay()` uses the same recomputation principle: `recompute_g_sums_subtree`
  reasserts G-I1 bottom-up after scaling `g.own` values in a subtree,
  followed by `recompute_g_sums` from the subtree root to the G-root
  ([ADR-M-024](024-decay-semantics.md)).
- The original `propagate_g_sums` function (delta-based `g.sum += Δ`
  walk) has been moved to a test helper in `src/tests/gtree.rs`.
  Phase 3 analysis ([ADR-M-014](014-value-absorption.md)) showed that
  eviction absorption does **not** require G-sum propagation —
  `p.sum` is invariant under terminal absorption. No production code
  path calls `propagate_g_sums`.
- Custom `Accumulator` implementations (e.g., `Saturating<u64>`)
  must implement `sub` (required by ADR-M-023 for reconstruction),
  but the observe hot path never invokes it.
- `f32`/`f64` accumulators benefit from self-healing sum recomputation.
