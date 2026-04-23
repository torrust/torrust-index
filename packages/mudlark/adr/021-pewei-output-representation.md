# ADR-M-021: PEWEI Output Representation

**Status:** Decided  
**Date:** 2026-02-26  
**Phase:** 4 (Output)  
**Relates to:** [ADR-M-005](005-primary-type-name.md) (naming),
[ADR-M-008](008-span-type.md) (Span type),
[ADR-M-022](022-pewei-serialisation.md) (serialisation),
[ADR-M-023](023-pewei-reconstruction.md) (reconstruction),
[ADR-M-032](032-three-surface-model.md) (three-surface model)  
**Spec:** §PEWEI M-2 (Structure), §PEWEI M-9 (Extraction)  
**API:** [§API M-4.2](../docs/api.md#42-contact-print)
(Pewei, Layer, Transition, Terminal)  
**Surface:** 1 (Prints)

## Context

PEWEI extraction (§PEWEI M-9) performs a breadth-first V-Tree
walk producing an ordered sequence of layers. Each layer contains two
populations: **phase-transition nodes** (internal or semi-internal
G-nodes — at least one G-child) and **terminal nodes** (leaf
G-nodes). The `Pewei<C, V>` type is the
owned, serialisable snapshot that `extract()` returns.

Three related sub-decisions:

1. How to structure the output type.
2. Which fields to store vs. derive.
3. How to handle the zero-baseline SNR edge case.

## Questions

### Q1: Output structure (DC-021-1)

| Option | Representation                                                                                          | Notes                                                                  |
| ------ | ------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------- |
| A      | `Vec<Layer<C, V>>` where `Layer { transitions: Vec<Transition<C, V>>, terminals: Vec<Terminal<C, V>> }` | Typed access to the two node populations, directly mirrors §PEWEI M-2. |
| B      | Single flat `Vec<PeweiNode<C, V>>` with a `layer: u32` field and an enum discriminant                   | Simple to iterate; single allocation. Loses typed layer access.        |
| C      | `Vec<Vec<PeweiNode<C, V>>>` — vec of layers, each a vec of nodes (enum)                                 | Grouped by layer. One enum type covers both node kinds.                |

**Considerations:**

- The spec defines two distinct node types with different fields
  (§PEWEI M-2). Option A reflects this directly in the type
  system.
- Consumers often want "all transitions" or "all terminals" within a
  layer — Option A makes this zero-cost.
- Option B is the simplest for sequential processing ("iterate all
  nodes in order") and has a single contiguous allocation.
- Option C is a middle ground but uses an enum, losing the typed
  separation.

### Q2: Stored vs. derived fields (DC-021-2)

Phase-transition nodes have derived fields: `refinement = total - baseline`
and `snr = refinement / baseline`. Terminal nodes carry only `intensity`.

| Option | Approach                                                                     | Notes                                                                                                         |
| ------ | ---------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| A      | Store all fields (precomputed)                                               | Fast access, slightly larger struct. One subtraction and one division per transition node at extraction time. |
| B      | Store only `baseline` and `total`; derive `refinement` and `snr` via methods | Smaller struct, computed on demand. Trivially cheap computation.                                              |
| C      | Store `baseline`, `total`, and `refinement`; derive `snr` via method         | Compromise: subtraction is stored, division is deferred (avoids the zero-baseline question in storage).       |

**Considerations:**

- `refinement` is a single subtraction — cheap either way.
- `snr` involves division by baseline, which has the zero-baseline
  edge case (Q3). Deferring it to a method makes the edge case the
  caller's problem (more explicit).
- Precomputing everything makes the struct self-contained and ready
  for serialisation without recomputation.

### Q3: Zero-baseline SNR (DC-021-3)

When `g.own == 0`, the SNR formula `(g.sum - g.own) / g.own` divides
by zero. This can happen if a node splits immediately on its first
observation (sum > θ but own is the first observation, then
children take over). Under current rules this means `own > 0`
(the triggering observation set `own`), but future changes or
`decay()` could produce `own == 0` with `sum > 0` after children
have accumulated.

| Option | Representation                                 | Notes                                                                    |
| ------ | ---------------------------------------------- | ------------------------------------------------------------------------ |
| A      | `f64::INFINITY`                                | Mathematically correct: infinite refinement relative to zero baseline.   |
| B      | `f64::NAN`                                     | Signals "undefined". Standard "no meaningful value" sentinel for floats. |
| C      | `Option<f64>`                                  | Explicit at the type level. Forces caller to handle.                     |
| D      | Method returning `Option<f64>` (store nothing) | Only relevant if Q2 chooses option B or C.                               |

**Considerations:**

- `INFINITY` is dangerous if the caller does arithmetic with it
  without checking.
- `NAN` propagates silently through arithmetic — a footgun.
- `Option<f64>` is the Rust-idiomatic "might not have a value" but
  adds ergonomic friction for the common case where `own > 0`.
- If Q2 defers SNR to a method, Option D avoids storing a potentially
  surprising value.

Also: should `v_depth` (the V-Tree depth at extraction time) be
stored on each node? It's available during BFS at zero cost and useful
for layer-indexed access, denoising (§PEWEI M-11), and diagnostics.

## Decision

**DC-021-1: Option A — `Vec<Layer>` with typed transition/terminal vecs.**

```rust
pub struct Pewei<C: Coordinate, V: Accumulator> {
    pub domain_start: C,    // C::zero()        — added by ADR-M-023 DC-023-4
    pub domain_end: C,      // C::domain_max(N) — added by ADR-M-023 DC-023-4
    pub layers: Vec<Layer<C, V>>,
}

pub struct Layer<C: Coordinate, V: Accumulator> {
    pub transitions: Vec<Transition<C, V>>,
    pub terminals: Vec<Terminal<C, V>>,
}

/// Phase-transition node: frozen V-entry backed by an internal G-node.
pub struct Transition<C: Coordinate, V: Accumulator> {
    pub start: C,           // region [start, end)
    pub end: C,
    pub baseline: V,        // g.own — energy before structure confirmed
    pub total: V,           // g.sum — total including refinement
    pub refinement: V,      // total - baseline (stored, not derived)
    pub depth: u32,         // G-Tree depth
    pub v_depth: u32,       // V-Tree depth at extraction time
}

/// Terminal node: active V-entry backed by a terminal G-node.
pub struct Terminal<C: Coordinate, V: Accumulator> {
    pub start: C,           // region [start, end)
    pub end: C,
    pub intensity: V,       // g.own = g.sum
    pub depth: u32,         // G-Tree depth
    pub v_depth: u32,       // V-Tree depth at extraction time
}
```

Directly mirrors §PEWEI M-2: two distinct populations with
different field sets. Typed vecs give zero-cost access to either population
within a layer without enum matching.

The PEWEI types are their own structs rather than reusing `Node<C,V>`
or `Cell<C,V>` — PEWEI is a serialisable snapshot with PEWEI-specific
fields (`v_depth`, `refinement`). Conversion from `Node`/`Cell` is
straightforward at extraction time.

**`v_depth` is stored on each node.** Free during BFS (it's the BFS
depth), costs 4 bytes per node. Useful for layer-indexed access,
denoising (§PEWEI M-11), and diagnostics.

**DC-021-2: Option C — store `baseline`, `total`, `refinement`; derive
`snr` via method.**

`refinement = total - baseline` is stored because it is used
frequently (reconstruction, denoising, display) and makes the
serialised form self-documenting.

`snr` is deferred to a method because it crosses type boundaries
(from `V` to `f64`) and has the zero-baseline edge case:

```rust
impl<C: Coordinate, V: Accumulator> Transition<C, V> {
    /// Local signal-to-noise ratio: refinement / baseline.
    ///
    /// Returns `None` when baseline is zero (division undefined).
    pub fn snr(&self) -> Option<f64> {
        let b = self.baseline.to_f64();
        if b == 0.0 { return None; }
        Some(self.refinement.to_f64() / b)
    }
}
```

**DC-021-3: Option D — `fn snr() -> Option<f64>` (method, not stored).**

No `INFINITY`, no `NaN`, no silent footgun. The zero-baseline case
is explicit at the API level. Callers handle it with standard
`Option` combinators. The serialised form contains `baseline` and
`refinement` — `snr` is always derivable.

**DC-021-4: Empty / zero-intensity tree → 1 layer, 1 terminal.**

After `GvGraph::new()` the root V-entry always exists (even with
zero intensity). `extract()` is a faithful snapshot of the V-Tree —
it emits what is physically present. An all-zero tree produces a
single `Layer` containing one `Terminal` with `intensity = V::zero()`.
Returning 0 layers would introduce a special case that consumers
must handle differently; a single zero-intensity terminal is truthful
and uniform.

**DC-021-5: Semi-internal G-nodes are classified as `Transition`.**

A semi-internal G-node (one G-child) has confirmed structure: one
half is covered by a child while the other half accumulates on the
node's own entry. `own != sum` for any non-trivial case, and the
spec's "phase-transition" definition ("further structure has been
confirmed") includes semi-internal by definition. The reconstruction
algorithm (ADR-M-023 DC-023-2) requires `baseline` / `total` /
`refinement` for these nodes — treating them as `Terminal` would
lose the child's energy contribution.

## Consequences

- **Type system mirrors the spec.** Two typed vecs per layer
  (transitions + terminals) directly reflect §PEWEI M-2's two
  node populations. Consumer code accesses either population
  without enum matching.
- **`snr()` is a method, not a stored field.** No `NaN` / `Infinity`
  in the serialised form. The zero-baseline edge case is explicit
  via `Option<f64>` — callers handle it with standard combinators.
- **`v_depth` stored at 4 bytes per node.** Free during BFS
  extraction, available for layer-indexed access, denoising
  (§PEWEI M-11), and diagnostics.
- **`Pewei` is self-contained.** `domain_start` / `domain_end`
  (added by ADR-M-023 DC-023-4) make the snapshot independent of the
  original `GvGraph` and its `N` const generic — reconstruction
  (ADR-M-023) and serialisation (ADR-M-022) need nothing else.
- **Semi-internal ≡ Transition.** No third category; consumers see
  exactly two node kinds. The reconstruction algorithm (ADR-M-023)
  handles semi-internal identically to fully internal.
- **Empty / zero-intensity tree is uniform.** One layer, one
  zero-intensity terminal — no special-case empty-tree handling in
  consumer code.
- **PEWEI types are distinct from graph view types.** `Transition`
  and `Terminal` are separate structs, not aliases for `Node<C,V>`
  or `Cell<C,V>` — they carry PEWEI-specific fields (`v_depth`,
  `refinement`) and belong to the Prints surface (ADR-M-032).
