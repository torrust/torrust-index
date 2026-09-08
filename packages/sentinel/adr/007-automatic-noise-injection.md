# ADR-S-007: Automatic Noise Injection · `rec:sentinel:automatic-internal-noise-injection`

**Status:** Implemented — modified by ADR-S-015 (`noise_rounds` → `noise_schedule`) **Date:** 2026-03-09 **Spec:** §ALGO S-11.1 (noise generation), §ALGO S-11.2 (injection triggers), §ALGO S-11.4 (chained coordination warming) **Relates to:** [ADR-S-001](001-measures-not-opinions.md) (measures not opinions), [ADR-S-005](005-deterministic-order-and-thread-safety.md) (deterministic order), [ADR-S-006](006-analysis-set-recomputation.md) (analysis set lifecycle)

## Context · `sec:sentinel:autonoise-context`

The spec (§ALGO S-11.2) requires noise injection to fire **automatically** on every new tracker creation — analysis set entry, split-induced creation, or legacy promotion. Chained coordination warming (§ALGO S-11.4) follows: synthetic score vectors from the warmed cell flow through the parent coordination tracker.

Without noise injection, new trackers start with placeholder baselines (`mean = 1.0`, `variance = 1.0`). Early z-scores and CUSUM values are meaningless until enough real data has passed.

## Decision · `sec:sentinel:autonoise-decision`

**Noise injection is automatic and internal. No public API for manual injection.**

1. **Auto-inject on tracker creation.** When the analysis selector (ADR-S-006) creates a new `CellState`, the sentinel immediately runs noise injection with rounds determined by `noise_schedule.rounds_for_depth(depth)` (ADR-S-015) and `noise_batch_size`.

2. **Noise config in `SentinelConfig`.** The fields `noise_schedule`, `noise_batch_size`, and `noise_seed` are top-level config fields. `noise_schedule` is a `NoiseSchedule` enum with `Geometric` and `Explicit` variants (ADR-S-015 §1). There is no separate `NoiseParams` struct.

3. **Persistent RNG.** A `SmallRng` is stored on `SpectralSentinel`, seeded from `config.noise_seed` (or system entropy if `None`). This ensures deterministic noise across the sentinel's lifetime.

4. **Chained coordination warming.** After a cell is noise-warmed, its synthetic scores flow through `propagate_coordination()` to warm the parent coordination tracker (§ALGO S-11.4). The coordination tracker's CUSUM is then reset (§ALGO S-7.4).

5. **No manual injection API.** Exposing a public `inject_noise()` method would allow double-injection and create an ordering hazard (host calling it after trackers already received auto-injection). The sentinel is the sole owner of the injection lifecycle.

## Alternatives Considered · `sec:sentinel:autonoise-alternatives`

- **Manual injection only.** Let the host decide when to inject. Rejected — the host cannot know when the sentinel creates trackers internally (analysis set churn is opaque). Manual injection is fundamentally incompatible with automatic tracker lifecycle.

- **Inject lazily on first real observation.** Defer until the tracker receives its first real batch. Rejected — noise injection's purpose is to bootstrap baselines *before* real data, so that early z-scores are meaningful.

- **Re-seed per injection.** Create a fresh RNG for each injection event. Rejected — this either requires the host to supply a seed per cell (impractical) or uses a single global seed per event (loses reproducibility across different analysis set sizes).

## Consequences · `sec:sentinel:autonoise-consequences`

- Every tracker starts warm. The maturity field `noise_influence` reflects genuine noise decay, not a cold-start artefact.
- The persistent RNG means noise sequences depend on creation order, which depends on traffic patterns. This is acceptable — noise need only provide reasonable initial baselines, not be statistically independent across trackers.
- Deterministic order (ADR-S-005) ensures that for a given seed and identical traffic, the noise injection sequence is identical across runs.
