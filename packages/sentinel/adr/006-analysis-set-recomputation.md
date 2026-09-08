# ADR-S-006: Analysis Set Recomputation Strategy · `rec:sentinel:full-analysis-set-recomputation-until-profiling`

**Status:** Decided **Date:** 2026-03-09 **Spec:** §ALGO S-4.2 (analysis set definition), §ALGO S-4.3 (ancestor closure), §ALGO S-4.4 (multi-scale delivery) **Relates to:** [ADR-S-003](003-mudlark-integration.md) (graph parameterisation), [ADR-S-002](002-feed-forward-invariant.md) (feed-forward invariant)

## Context · `sec:sentinel:recompute-context`

The analysis set ($\mathcal{A}$) is the top-$K$ V-Tree entries by importance with V-Tree depth $\leq L$, closed under G-tree ancestry (§ALGO S-4.2). It determines which cells own `SubspaceTracker`s and therefore controls the sentinel's resource usage.

After each batch of observations mutates the G-V Graph, the analysis set may change — cells split, merge, gain or lose importance. The sentinel must detect these changes, create trackers for new entries, and destroy trackers for evicted entries.

Two broad strategies exist:

1. **Full recomputation:** scan the entire V-Tree via `layers()` after every `ingest()`, rebuild from scratch, diff against the previous set.
2. **Incremental update:** mudlark notifies the sentinel of structural changes via callbacks or a change log; the sentinel patches in place.

## Decision · `sec:sentinel:recompute-decision`

**Start with full recomputation. Defer incremental updates until profiling shows the scan is a bottleneck.**

### Implementation sketch · `sec:sentinel:recompute-implementation-sketch`

```rust
impl<C, V> AnalysisSet<C, V> {
    pub fn recompute(graph: &GvGraph<C, V, N>, k: usize, l: usize) -> Self {
        let mut candidates: Vec<AnalysisEntry<C, V>> = graph
            .layers()
            .filter(|(v_depth, _)| *v_depth <= l)
            .map(|(v_depth, node)| AnalysisEntry {
                gnode: node.gnode_id,
                depth: node.depth,
                v_depth,
                importance: node.own,
                start: node.start,
                end: node.end,
            })
            .collect();

        candidates.sort_by(|a, b| {
            b.importance.cmp(&a.importance)
                .then(a.start.cmp(&b.start))
        });
        candidates.truncate(k);

        // Ancestor closure: walk G-tree parents for each competitive entry.
        // ...
    }
}
```

After recomputation, the sentinel diffs old and new sets:

- **New entries:** create `CellState` + `SubspaceTracker`, noise-inject (ADR-S-007).
- **Removed entries:** destroy the tracker, freeing memory.
- **Retained entries:** keep existing tracker state.

### Why full scan is acceptable initially · `sec:sentinel:recompute-full-scan-rationale`

- `layers()` is $O(n)$ in live G-nodes. With the default budget of 100,000, this is at most 100k iterations per `ingest()`.
- The scan is read-only — no allocation, no mutation.
- The sort is $O(n \log n)$ but operates on a filtered subset (entries with `v_depth ≤ L`), typically much smaller than $n$.
- `ingest()` is called once per batch (not per observation).

### Incremental path (deferred) · `sec:sentinel:recompute-incremental-path`

An incremental approach requires mudlark to expose a change notification mechanism — either callbacks or a `ChangeLog` buffer. Mudlark does not currently provide either. Adding one is non-trivial and should be driven by measured need, not speculation.

## Alternatives Considered · `sec:sentinel:recompute-alternatives`

- **Incremental from the start.** Rejected — premature optimisation. Higher-priority work (suffix analysis, coordination hierarchy) comes first.
- **Periodic recomputation.** Only recompute every $N$ batches. Rejected — stale analysis sets mean trackers may be allocated to cells that no longer exist.
- **Hash-based change detection.** Hash the V-Tree and skip recomputation if unchanged. Rejected — hashing is itself $O(n)$, saving nothing over a full recompute.

## Consequences · `sec:sentinel:recompute-consequences`

- Every `ingest()` pays a scan of the V-Tree limited to depth $\leq L$ via `layers_to(depth_cutoff)` (ADR-M-041). Nodes below the cutoff are never enqueued, so the cost is $O(n_L)$ rather than $O(n)$. With $n = 100{,}000$ and batches arriving at ~1 Hz, this is negligible compared to the SVD work in `SubspaceTracker`.
- Tracker creation/destruction follows analysis set churn. If the contour is stable (most batches), the diff is empty.
- If profiling shows the scan is a bottleneck (unlikely until $n > 10^6$), this ADR should be revisited.
