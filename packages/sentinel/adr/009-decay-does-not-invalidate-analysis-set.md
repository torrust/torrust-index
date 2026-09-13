# ADR-S-009: Decay Does Not Invalidate the Analysis Set · `rec:sentinel:recompute-on-ingest-without-decay-invalidation`

**Status:** Decided **Date:** 2026-03-10 **Spec:** §ALGO S-10 (temporal decay) **Relates to:** [ADR-S-006](006-analysis-set-recomputation.md) (analysis set recomputation)

## Context · `sec:sentinel:decayset-context`

After `decay()` or `decay_subtree()`, cell importance values change. The analysis set — which selects top-$K$ cells by importance — could become stale. Two strategies:

- **A)** Set an invalidation flag; force recomputation before the next scoring operation.
- **B)** Do nothing; rely on `ingest()` always recomputing.

## Decision · `sec:sentinel:decayset-decision`

**Option B.** The analysis set is recomputed from scratch at the start of every `ingest()` call (ADR-S-006). No scoring occurs between a `decay()` call and the next `ingest()` — the sentinel's API does not expose a "score without ingesting" path. Therefore the analysis set is never stale when it matters.

## Alternatives Considered · `sec:sentinel:decayset-alternatives`

| Option | Pros | Cons |
|--------|------|------|
| Invalidation flag (A) | Correct even if API adds a score-only path | Extra state, branching, easy to forget |
| Do nothing (B) | Simpler, zero overhead, no new state | Requires revisiting if score-only API is added |

## Consequences · `sec:sentinel:decayset-consequences`

- `decay()` is a pure spatial operation with no side-effects on the analysis tier. This keeps the temporal and analytical concerns cleanly separated (§ALGO S-10).
- If a future API adds "score against current model without new observations", this ADR must be revisited — the analysis set would need recomputation or validation before scoring.
- The `inspect_cell()` method reads existing tracker state (no recomputation) and is unaffected — it reports from the last `ingest()` cycle's model.
