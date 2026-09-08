# ADR-S-008: Scoring Geometry Extension · `rec:sentinel:report-scoring-geometry-to-hosts`

**Status:** Implemented **Date:** 2026-03-08 **Spec:** — (implementation extension, not in algorithm.md) **Relates to:** [ADR-S-001](001-measures-not-opinions.md) (measures not opinions)

## Context · `sec:sentinel:geometry-context`

The four scoring axes (§ALGO S-6) have structural preconditions:

- **Novelty** degenerates when `rank == dim` — the subspace spans the full space, so every observation has zero residual. The novelty score is always 0.0.
- **Coherence** is undefined when `rank < 2` — there are fewer than two latent dimensions, so pairwise cross-correlation has no pairs.

These conditions are not bugs — they are natural consequences of the tracker's operating state. But a host consuming novelty z-scores without knowing that novelty is currently degenerate would misinterpret the silence as "everything is normal".

The spec does not define a mechanism for the host to detect these conditions. The host would have to derive them from `rank`, `energy_ratio`, and knowledge of the tracker dimensionality — indirect and error-prone.

## Decision · `sec:sentinel:geometry-decision`

**Add `ScoringGeometry` to every report, tracking the geometric operating state of each tracker.**

```rust
pub struct ScoringGeometry {
    /// Whether novelty is degenerate (rank == dim).
    pub novelty_saturated: bool,
    /// Whether novelty *could* saturate (rank == dim − 1).
    pub novelty_saturable: bool,
    /// Whether coherence is active (rank >= 2).
    pub coherence_active: bool,
}
```

And a `GeometryDistribution` summary in `HealthReport` for fleet-wide monitoring:

```rust
pub struct GeometryDistribution {
    pub novelty_saturated: usize,
    pub novelty_saturable: usize,
    pub coherence_inactive: usize,
}
```

## Alternatives Considered · `sec:sentinel:geometry-alternatives`

| Option | Pros | Cons |
|--------|------|------|
| Let host derive from rank + dim | No new types | Error-prone, requires dim knowledge |
| Boolean flags on `AnomalyScores` | Close to the data | Mixes measurement with metadata |
| Separate `ScoringGeometry` (chosen) | Clean separation, aggregatable | One more type |

## Consequences · `sec:sentinel:geometry-consequences`

- The host can filter out novelty alerts when `novelty_saturated` is true, and monitor `coherence_inactive` counts for health.
- `ScoringGeometry` is purely informational — it does not suppress scores. The sentinel still reports novelty = 0.0 when saturated; the host decides whether to ignore it (ADR-S-001).
- This extension should be back-ported to the spec (§ALGO S-14) as a recommended report field.
