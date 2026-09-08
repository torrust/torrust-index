# ADR-S-017: Deferred Cell Warm-Up · `rec:sentinel:background-priority-warmup-off-ingest-path`

**Status:** Implemented (synchronous fallback + background thread) **Date:** 2026-03-11 **Revised:** 2026-03-13 **Spec:** §ALGO S-18 (work variance and timing considerations) **Relates to:** [ADR-S-007](007-automatic-noise-injection.md) (automatic noise injection), [ADR-S-015](015-cell-creation-performance.md) (cell creation performance), [ADR-S-002](002-feed-forward-invariant.md) (feed-forward invariant), [ADR-S-005](005-deterministic-order-and-thread-safety.md) (deterministic order)

## Context · `sec:sentinel:deferredwarmup-context`

The sentinel's `ingest()` call has **variable latency**. Most calls perform only scoring (fast), but calls that trigger analysis set changes also run noise injection synchronously — up to hundreds of milliseconds per new cell (ADR-S-015). A batch-oriented network monitor that budgets $T$ ms per batch cannot tolerate multi-second stalls from cell creation bursts.

## Decision · `sec:sentinel:deferredwarmup-decision`

**Noise injection is moved off the `ingest()` hot path.**

New cells transition through a three-state lifecycle:

```
    created ──→ warming ──→ online ──→ (destroyed)
                  │
                  │  background thread
                  │  highest g.sum first
                  ▼
            noise injection
```

1. **Created.** The analysis selector identifies a new cell. A `SubspaceTracker` is allocated but receives no noise and no real observations. The cell is enqueued into a staging area.

2. **Warming.** A background thread works on whichever warming cell has the highest `g.sum` (accumulated volume in the G-V Graph), injecting noise batches according to the depth-tiered noise schedule (ADR-S-015). If a higher-priority cell arrives, work switches immediately — the previous cell retains partial progress.

3. **Online.** Noise injection is complete. At the start of the next `ingest()` call, the cell is promoted from the staging area into the live `cells` map and begins receiving real observations.

### Observation Routing During Warm-Up · `sec:sentinel:deferredwarmup-observation-routing`

Observations destined for a warming cell are routed to the nearest **online** ancestor (or to the root if no closer ancestor is online). Scoring quality does not degrade — it stays at the coarser resolution until the child goes online.

### Coordination During Warm-Up · `sec:sentinel:deferredwarmup-coordination`

Warming cells do not activate coordination contexts. Coordination activates only at promotion, at which point the cell's baselines are converged. Each newly activated coordination context runs a cheap inline warm-up (§ALGO S-9.8) using Gamma-sampled synthetic score vectors derived from the participating cells' mature baselines.

### No Slot Reservation · `sec:sentinel:deferredwarmup-no-slot-reservation`

Warming cells do not hold competitive slots. The analysis set contains only online cells. If a warming cell's G-node is evicted before warm-up completes, the partial work is discarded — the G-V Graph determined the interval no longer warrants a node.

### Priority: Volume-First · `sec:sentinel:deferredwarmup-volume-first-priority`

The priority rule `g.sum` produces root-first ordering as a consequence — shallow cells accumulate descendant volume, satisfying the dependency constraint (ancestors online before descendants) without encoding depth into the priority key.

### Synchronous Fallback · `sec:sentinel:deferredwarmup-synchronous-fallback`

When `background_warming` is disabled, warm-up runs synchronously within `reconcile_analysis_set()`. This preserves deterministic single-threaded behaviour for testing.

## Work Variance Bound (§ALGO S-18.4) · `sec:sentinel:deferredwarmup-work-variance-bound`

With deferred warm-up, the per-call cost of `ingest()` is bounded by:

$$O\!\Big(n(d_{\text{geo}} + h_V) \;+\; |\mathcal{A}^*| \cdot w_{\max} \cdot (k + b)^2\Big)$$

on every call. Cell creation and noise injection contribute zero cost. The remaining call-to-call variance (batch size, analysis set size, rank) changes slowly relative to call frequency.

## Timing Protection Is Out of Scope (§ALGO S-18.5) · `sec:sentinel:deferredwarmup-timing-protection-out-of-scope`

Deferred warm-up makes `ingest()` operationally predictable — bounded work per call with no structural spikes. It does **not** make `ingest()` constant-time. The remaining variance, though small, is observable to a sufficiently precise adversary. Adaptive timing pads and equalization are explicitly out of scope.

## Consequences · `sec:sentinel:deferredwarmup-consequences`

- `ingest()` has bounded, predictable work per call. Cell creation and noise injection never stall the hot path.

- A background thread (or synchronous fallback) is required for warming. The interaction surface is minimal: a staging map with atomic promotion at the top of each `ingest()` call.

- ADR-S-007 (automatic noise injection) remains correct — injection is still automatic and internal — but the trigger changes from "inject synchronously at creation" to "enqueue for background injection at creation."

- ADR-S-015 (cell creation performance) is complementary. The depth-tiered noise schedule determines how long background warming takes per cell; the hot-path stall concern is resolved by this ADR.
