# ADR-S-010: Linear Routing Over G-Tree Descent · `rec:sentinel:linear-containment-routing-until-profiled`

**Status:** Decided **Date:** 2026-03-10 **Spec:** §ALGO S-8.1 Step 4 (observation delivery) **Relates to:** [ADR-S-003](003-mudlark-integration.md) (mudlark integration), [ADR-S-006](006-analysis-set-recomputation.md) (analysis set recomputation)

## Context · `sec:sentinel:routing-context`

During Step 4 of `ingest()` (§ALGO S-8.1), each observation must be delivered to every tracker on its G-tree ancestor path. Two routing strategies:

- **A)** G-tree descent: use `graph.route(v)` to find the receiving cell, then walk G-tree parents. Requires a mudlark API for parent traversal that returns materialised G-node IDs matching the sentinel's `BTreeMap<GNodeId, CellState>` keys.
- **B)** Linear scan: for each observation, iterate all cells in the analysis set and check interval containment (`cell.start <= v <
  cell.end`). Cost: $O(n \cdot |\mathcal{A}^*|)$ per batch.

## Decision · `sec:sentinel:routing-decision`

**Option B (linear scan).** G-tree descent requires a `route_to_ancestors()` or equivalent API from mudlark that returns the chain of `GNodeId` values for materialised ancestors. This API does not currently exist. The G-tree's intervals are dyadic and nested, so flat interval containment is correct — every ancestor's interval contains the observation by construction.

## Performance · `sec:sentinel:routing-performance`

At typical operating points ($|\mathcal{A}^*| \leq 2K \approx 2{,}048$, batch size $b \leq 64$), the scan is $\sim 130{,}000$ comparisons per batch — negligible relative to the SVD cost that dominates `ingest()`. Profile before optimising.

## Alternatives Considered · `sec:sentinel:routing-alternatives`

| Option | Pros | Cons |
|--------|------|------|
| G-tree descent (A) | $O(d)$ per observation | Requires non-existent mudlark API |
| Binary search on sorted intervals | $O(n \log |\mathcal{A}^*|)$ | Added complexity for marginal gain at current $K$ |
| Linear scan (B) | No external dependency, simple, correct | $O(n \cdot |\mathcal{A}^*|)$ per batch |

## Consequences · `sec:sentinel:routing-consequences`

- No dependency on a mudlark parent-traversal API. The sentinel operates on its own `BTreeMap<GNodeId, CellState>` using interval metadata cached at tracker creation time.
- If profiling identifies routing as a bottleneck at large $K$, the first optimisation step is sorting the analysis set by interval and using binary search ($O(n \log |\mathcal{A}^*|)$), not adding a mudlark API dependency.
- If mudlark later exposes `route_to_ancestors()`, this ADR can be revisited for a constant-factor improvement.
