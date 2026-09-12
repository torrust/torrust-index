# ADR-S-002: Feed-Forward Invariant · `rec:sentinel:unit-delta-feed-forward-spatial-analysis`

**Status:** Implemented (2026-03-09) **Date:** 2026-03-08 **Spec:** §ALGO S-2.4 (what sentinel owns vs inherits), §ALGO S-8.1 Step 2 (volume accounting), §ALGO S-8.2 (step ordering) **Relates to:** [ADR-S-001](001-measures-not-opinions.md) (measures not opinions), [ADR-S-003](003-mudlark-integration.md) (V = u64)

## Context · `sec:sentinel:feedforward-context`

The sentinel owns a `GvGraph` for adaptive spatial partitioning and a bank of `SubspaceTracker`s for statistical analysis. There is a question of what signal flows between these two subsystems:

- **Feed-forward:** the G-V Graph receives raw observations and shapes the contour; the analysis engine reads the contour's structure. No analysis output flows back.
- **Feedback:** anomaly scores or derived signals could be fed back into the G-V Graph as importance weights, causing regions with high anomaly scores to receive finer resolution.

## Decision · `sec:sentinel:feedforward-decision`

**Strict feed-forward. The G-V Graph receives only `observe(v, 1u64)` per raw input value. Anomaly scores, z-scores, CUSUM values, and any other derived signals are never fed back.**

```rust
// The ONLY permitted G-V Graph mutation during ingest:
for &value in values {
    self.graph.observe(value, 1u64);
}
```

This is a code-review invariant, not mechanically enforced.

## Rationale · `sec:sentinel:feedforward-rationale`

1. **Feedback creates resonance.** If anomaly scores amplify importance, regions currently under scrutiny get more resolution, which generates more data, which changes the anomaly scores. This feedback loop could cause the contour to lock onto artefacts of its own analysis rather than genuinely active traffic.

2. **Separation of concerns.** The G-V Graph (Layer 1) reflects objective traffic volume. The analysis engine (Layer 3) reflects statistical deviation from learned baselines. Keeping them independent means a caller can reason about each in isolation.

3. **Host control.** The host controls temporal policy via `decay()`. If importance weighting is desired, the host can apply it at the decay layer (e.g., selective `decay_subtree()` calls guided by analysis output). This keeps the feedback loop in host code, where policy belongs (ADR-S-001).

## Consequences · `sec:sentinel:feedforward-consequences`

- The G-V Graph's contour is shaped entirely by raw traffic volume and host-initiated decay. The analysis tier has no influence on spatial resolution.
- Δ = 1 means the accumulator type `V` is a pure observation counter (for the default `V = u64`), simplifying reasoning about `split_threshold` (it is simply "how many observations before splitting").
- This invariant must be maintained across all phases of the integration plan. Code review should reject any path that calls `graph.observe()` with a value other than the unit delta.

### Enforcement mechanisms · `sec:sentinel:feedforward-enforcement-mechanisms`

1. **No `graph.observe()` during noise injection.** The noise pathway operates on tracker-space `f64` vectors (synthetic centred bit vectors fed directly to `SubspaceTracker::observe()`), not coordinate-space values. Therefore `graph.observe()` is never called during noise injection. This clarifies how the feed-forward invariant interacts with automatic noise injection (ADR-S-007).

2. **No `graph_mut()` exposure.** The sentinel exposes only a read-only `graph()` accessor (`&GvGraph`) for diagnostics. Mutable graph access is provided exclusively through controlled methods — `decay()` and `decay_subtree()` — which enforce valid parameters. This prevents callers from violating the invariant by calling `graph.observe(v, 100)` directly.

3. **No pre-aggregated observation.** Each value is fed individually via `graph.observe(value, unit_delta)`. Do not pre-aggregate values by cell and call `observe(representative, count)` — this would lose spatial resolution (values may map to different leaf cells), add complexity for no performance gain (routing still happens inside `observe()`), and violate §ALGO S-8.3 which specifies "Δ = 1 per value".
