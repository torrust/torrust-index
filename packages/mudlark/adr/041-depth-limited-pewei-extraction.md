# ADR-M-041: Depth-Limited PEWEI Extraction

**Status:** Decided — implemented  
**Date:** 2026-04-07  
**Phase:** 4 (Output)  
**Relates to:** [ADR-M-021](021-pewei-output-representation.md) (PEWEI output),
[ADR-M-023](023-pewei-reconstruction.md) (reconstruction),
[ADR-M-030](030-graph-module-decomposition.md) (graph modules)  
**Spec:** §PEWEI M-2 (Structure), §PEWEI M-9 (Extraction)  
**API:** [§API M-4.2](../docs/api.md#42-contact-print) (Pewei)  
**Surface:** 1 (Prints)

## Context

`GvGraph::extract()` performs a full BFS over the V-Tree —
every V-node is visited, every layer emitted. The cost is
$O(L + S)$ where $L$ = V-entries and $S$ = V-structural nodes.

Many consumers need only the first $K$ layers (the coarse,
dominant-energy structure) and discard the rest. Today those
consumers must:

1. Pay the full $O(L + S)$ extraction cost.
2. Build the complete `Vec<Layer>` allocation.
3. Either truncate via `reconstruct(K)` or manually slice
   `pewei.layers[..=K]`.

For large V-Trees (deep drilling, high `depth_create`, no budget
cap), the deeper layers account for the vast majority of nodes
— a tree with $D$ depth levels can have up to $2^D - 1$ V-entries,
heavily concentrated in the last few layers. An extraction
limited to layers $0..K$ would skip those nodes entirely.

The `layers()` streaming iterator partially addresses this — callers
can `.take_while(|(layer, _)| *layer <= K)`, but structural nodes
below the cutoff are still traversed (they're scaffolding that must
be entered to discover their children, even if those children are
skipped). A BFS-level cutoff avoids enqueuing sub-cutoff children
at all.

### Motivating use cases

1. **Coarse preview.** Diagnostic or UI tool requests only the
   top 3–5 significance layers for a quick spatial overview.
2. **Progressive refinement.** A streaming protocol sends layer 0
   first, then layer 1, etc. The producer extracts incrementally —
   depth 0, then depth 1, etc. — without redoing the full BFS
   each time (though re-extraction at increasing depths is a
   simpler first step).
3. **Bounded output size.** A caller wants at most $K$ layers
   regardless of how deep the V-Tree has grown. Today this is
   possible post-hoc (`reconstruct(K)`), but the extraction cost
   is still full.

## Questions

### Q1: API shape (DC-041-1)

| Option | Signature                                             | Notes                                                         |
| ------ | ----------------------------------------------------- | --------------------------------------------------------------|
| A      | `extract_to(v_depth_limit: u32) -> Pewei<C, V>`        | Minimal — one new method, zero changes to `extract()`.        |
| B      | `extract_with(opts: ExtractOptions) -> Pewei<C, V>`   | Builder/options struct. Extensible for future extraction knobs.|
| C      | `extract(v_depth_limit: Option<u32>) -> Pewei<C, V>`    | Replaces existing signature. Callers pass `None` for full.    |
| D      | Keep `extract()` unchanged; add `extract_to(u32)`.    | Non-breaking. `extract()` delegates to `extract_to(u32::MAX)`.|

**Considerations:**

- Option C is a breaking change to every existing call site.
  `extract()` is public and documented.
- Option B adds an `ExtractOptions` struct that currently has
  one field. Over-engineering unless other extraction knobs are
  anticipated (none identified today).
- Option D is non-breaking. `extract()` remains the full-extraction
  convenience; `extract_to()` is the depth-limited variant. The
  implementation is shared — `extract()` calls `extract_to(u32::MAX)`.
- Option A is equivalent to D but doesn't refactor `extract()`.

### Q2: Metadata — should `Pewei` record its extraction depth? (DC-041-2)

| Option | Representation                    | Notes                                                            |
| ------ | --------------------------------- | ---------------------------------------------------------------- |
| A      | `Pewei { v_depth_limit: Option<u32>, .. }` | Explicit: `None` = full extraction, `Some(k)` = truncated.  |
| B      | No metadata — layer count implies depth. | Simpler struct. Consumers can't distinguish "full tree has    |
|        |                                   | 3 layers" from "limited to 3 layers".                            |
| C      | `Pewei { truncated: bool, .. }`   | Binary flag. Loses the specific depth value.                     |

**Considerations:**

- With Option B, a consumer receiving a serialised `Pewei` cannot
  tell if deeper layers exist. For diagnostic and progressive-
  refinement use cases, this matters.
- Option A is the richest signal. `v_depth_limit: None` when produced
  by `extract()`, `Some(K)` when produced by `extract_to(K)`.
  Cost: 8 bytes (Option<u32>). Serde-friendly.
- Adding a field to `Pewei` is (mildly) breaking if consumers match
  on struct literals. Since `Pewei` has `pub` fields and is
  constructed in tests, a `#[non_exhaustive]` annotation or a
  future-proofing note may be warranted. However, the struct already
  grew `domain_start`/`domain_end` in ADR-M-023, so precedent
  exists.

### Q3: `layers()` iterator — depth-limited variant? (DC-041-3)

| Option | Approach                                                | Notes                                                 |
| ------ | ------------------------------------------------------- | ----------------------------------------------------- |
| A      | `layers_to(v_depth_limit: usize) -> impl Iterator<..>`   | Parallel to `extract_to()`.                           |
| B      | No new iterator; callers use `.take_while()`.          | Works for skipping nodes but still enqueues structural |
|        |                                                         | children below the cutoff (wasted work).              |
| C      | `layers()` takes an `Option<usize>` parameter.         | Breaking change.                                      |

**Considerations:**

- The streaming `layers()` iterator currently expands structural
  nodes unconditionally. A `take_while` on the consumer side stops
  yielding but the BFS queue still grows. For very deep trees, the
  wasted queue growth is non-trivial.
- Option A fixes the BFS cutoff inside the iterator.

### Q4: Semantics at the cutoff (DC-041-4)

When the BFS encounters a V-structural node at `bfs_depth == v_depth_limit`:

| Option | Behaviour                                                 | Notes                                                 |
| ------ | --------------------------------------------------------- | ----------------------------------------------------- |
| A      | Do not enqueue children. Structural node itself is never  | Clean BFS termination. Layers `> v_depth_limit` are     |
|        | emitted (it's scaffolding), so nothing changes for        | simply absent — energy in those layers is lost from    |
|        | emitted nodes.                                            | the snapshot.                                         |
| B      | Promote invisible children to terminals at `v_depth_limit`. | Preserves energy conservation. Extra complexity.       |

**Considerations:**

- Option A is simple and consistent with how `reconstruct(max_layer)`
  already works — it just sees fewer layers.
  `total_energy()` on a depth-limited `Pewei` returns the energy
  of the visible layers only. This is documented, not a bug.
- Option B would require synthesizing terminal nodes from
  V-structural metadata, which is not possible — structural nodes
  don't carry G-node references (they're pure scaffolding with
  child pointers and aggregated intensity). The "invisible energy"
  belongs to entry nodes at deeper layers that weren't visited.
  There is no clean way to attribute it without walking those nodes,
  which defeats the purpose.
- `reconstruct()` on a depth-limited `Pewei` produces the same
  result as `reconstruct(K)` on a full `Pewei` (assuming
  `v_depth_limit == K`). Energy conservation holds **within** the
  visible layers.

## Decision

**DC-041-1: Option D — `extract_to(v_depth_limit: u32)` alongside unchanged `extract()`.**

```rust
impl<C: Coordinate, V: Accumulator + Inspectable, const N: u32>
    GvGraph<C, V, N>
{
    /// Extract a depth-limited PEWEI snapshot.
    ///
    /// Only V-Tree BFS depths `0..=v_depth_limit` are visited.
    /// Layers beyond `v_depth_limit` are absent from the result.
    /// Pass `u32::MAX` for a full extraction.
    #[must_use]
    pub fn extract_to(&self, v_depth_limit: u32) -> Pewei<C, V> { .. }

    /// Extract a full PEWEI snapshot (all layers).
    ///
    /// Equivalent to `self.extract_to(u32::MAX)`.
    #[must_use]
    pub fn extract(&self) -> Pewei<C, V> {
        self.extract_to(u32::MAX)
    }
}
```

Non-breaking. Existing `extract()` callers are unaffected. The
depth-limited path is a single `if bfs_depth > v_depth_limit { continue }`
guard and a child-enqueue gate on structural nodes:

```rust
VKind::Structural { children, .. } => {
    if bfs_depth + 1 <= v_depth_limit {
        for i in 0..children.len() {
            let (child_id, _) = children.get(i);
            queue.push_back((child_id, bfs_depth + 1));
        }
    }
}
```

**DC-041-2: Option A — `v_depth_limit: Option<u32>` on `Pewei`.**

```rust
pub struct Pewei<C: Coordinate, V: Accumulator> {
    pub domain_start: C,
    pub domain_end: C,
    pub layers: Vec<Layer<C, V>>,
    /// `None` = full extraction (all V-Tree layers present).
    /// `Some(k)` = extraction was limited to V-Tree BFS depth ≤ k.
    pub v_depth_limit: Option<u32>,
}
```

`extract()` sets `v_depth_limit: None`. `extract_to(K)` sets
`v_depth_limit: Some(K)` unless `K >= actual_tree_depth`, in which
case it also sets `None` (the tree was fully captured).

Cost: 8 bytes. Serde-friendly (`Option<u32>` serialises as
`null` / integer). Gives consumers and diagnostics the ability to
distinguish "tree is shallow" from "extraction was capped".

**DC-041-3: Option A — `layers_to(v_depth_limit: usize)` iterator.**

Parallel to `extract_to()`. The iterator's internal BFS queue
respects the depth limit, avoiding wasted structural expansion.
`layers()` delegates to `layers_to(usize::MAX)`.

**DC-041-4: Option A — clean BFS termination, no energy promotion.**

Nodes beyond the cutoff are simply not visited. `total_energy()`
returns the energy of visible layers only. The relationship between
a full and depth-limited PEWEI is:

$$E_{\text{limited}} = E_{\text{full}} - \sum_{d > K} E_d$$

where $E_d$ is the energy contributed by layer $d$. This is
documented on `extract_to()` and `Pewei::total_energy()`.

## Implementation sketch

### `graph_extract.rs` changes

The BFS loop gains a `v_depth_limit` parameter:

```rust
pub fn extract_to(&self, v_depth_limit: u32) -> Pewei<C, V> {
    // ... same setup as extract() ...

    while let Some((vid, bfs_depth)) = queue.pop_front() {
        let vnode = self.vnodes.get(vid.index());

        match &vnode.kind {
            VKind::Entry { gnode, .. } => {
                // Unchanged — emit transition or terminal.
                // (bfs_depth is guaranteed ≤ v_depth_limit by the
                //  structural-node gate below.)
                // ...
            }
            VKind::Structural { children, .. } => {
                // Gate: only enqueue children if they'd be within
                // the depth limit.
                if bfs_depth + 1 <= v_depth_limit {
                    for i in 0..children.len() {
                        let (child_id, _) = children.get(i);
                        queue.push_back((child_id, bfs_depth + 1));
                    }
                }
            }
        }
    }

    let actual_full = /* check if v_depth_limit >= deepest emitted layer */;
    Pewei {
        domain_start,
        domain_end,
        layers,
        v_depth_limit: if actual_full { None } else { Some(v_depth_limit) },
    }
}
```

Entry nodes don't need the depth guard because they're only
reachable if their parent structural node passed the gate. The
root V-entry at depth 0 is always within bounds (unless
`v_depth_limit < 0`, which is impossible for `u32`).

### `pewei.rs` changes

1. Add `v_depth_limit: Option<u32>` field to `Pewei`.
2. Update `reconstruct()` — no logic changes needed; it already
   works on whatever layers are present.
3. Update doc-comments on `total_energy()` to note that a
   depth-limited PEWEI's energy is partial.

### `layers()` changes

```rust
pub fn layers_to(
    &self,
    v_depth_limit: usize,
) -> impl Iterator<Item = (usize, Node<C, V>)> + '_ {
    // BFS with depth gate on structural children.
}

pub fn layers(&self) -> impl Iterator<Item = (usize, Node<C, V>)> + '_ {
    self.layers_to(usize::MAX)
}
```

### Testing

| Test                                  | Validates                                             |
| ------------------------------------- | ----------------------------------------------------- |
| `extract_to(0)` on multi-layer tree   | Only layer 0 emitted; deeper nodes absent.            |
| `extract_to(K)` energy + reconstruct  | `reconstruct(K)` on full == `reconstruct(all)` on depth-limited at K. |
| `extract_to(u32::MAX)` == `extract()` | Full extraction unchanged.                            |
| `layers_to(K)` count                  | Same nodes as `extract_to(K).layers`.                 |
| `v_depth_limit` metadata              | `None` for full, `Some(K)` for limited.               |
| Serde round-trip with `v_depth_limit` | `Option<u32>` serialises correctly.                   |

## Performance analysis

For a balanced V-Tree with $D$ depth levels and $2^D - 1$ total
V-entries:

| Depth limit $K$ | V-nodes visited      | Fraction of full BFS          |
| --------------- | -------------------- | ----------------------------- |
| 0               | $O(1)$               | Constant                      |
| $D/2$           | $O(2^{D/2})$         | $\sqrt{n}$                    |
| $D-1$           | $O(2^{D-1})$         | ~50%                          |
| $D$ (full)      | $O(2^D)$             | 100%                          |

The savings are exponential in $(D - K)$ for balanced trees. Real
V-Trees are typically imbalanced (energy concentrates at a few
deep paths), so the practical speed-up is data-dependent but
never worse than the full BFS.

## Consequences

- **Non-breaking.** `extract()` signature and behavior unchanged.
  `Pewei` gains one field; struct-literal construction in tests
  must add `v_depth_limit: None`.
- **Callers control cost.** "Give me the top 5 layers" is
  $O(L_5 + S_5)$ instead of $O(L + S)$.
- **Energy is partial.** `total_energy()` on a depth-limited PEWEI
  is less than `g.total_sum()`. Documented, not surprising — same
  as `reconstruct(K)` on a full PEWEI.
- **`reconstruct()` unchanged.** It works on whatever layers are
  present; no special-casing for depth-limited PEWEIs.
- **Metadata enables progressive protocols.** A consumer receiving a
  serialised `Pewei` with `v_depth_limit: Some(3)` knows deeper
  layers exist and can request them.
