# ADR-S-019: Investment-Set Terminology and Reporting Alignment · `rec:sentinel:investment-set-reporting-priority-and-vocabulary`

**Status:** Accepted **Date:** 2026-03-17 **Spec:** §ALGO S-8 (analysis selector), §ALGO S-11.6 (warm-up lifecycle), §ALGO S-14.11–14.12 (reporting) **Supersedes:** [ADR-S-017](017-deferred-cell-warm-up.md) §"No Slot Reservation" (warming cells now hold investment slots) **Relates to:** [ADR-S-017](017-deferred-cell-warm-up.md) (deferred cell warm-up), [ADR-S-006](006-analysis-set-recomputation.md) (analysis set recomputation)

## Context · `sec:sentinel:investment-context`

The algorithm specification (§ALGO S-8) was updated to introduce a three-level naming hierarchy for analysis cells:

| Symbol | Name | Definition |
|--------|------|------------|
| $\mathcal{T}$ | Competitive targets | Top $K$ V-entries by importance within the depth cutoff |
| $\mathcal{I}$ | Investment set | $\mathcal{T}$ closed under G-Tree ancestry; every member has an allocated tracker regardless of online status |
| $\mathcal{A}$ | Producing competitive set | $\mathcal{T} \cap \text{Online}$ — competitive targets that are online and producing scores |
| $\mathcal{A}^*$ | Producing full set | $\mathcal{I} \cap \text{Online}$ — all online members of the investment set |

The key insight is that the **investment set** ($\mathcal{I}$) is the resource-commitment boundary — every member has an allocated tracker and is either online or warming — while the **producing sets** ($\mathcal{A}$, $\mathcal{A}^*$) are the score-generation boundary.

Previously (ADR-S-017 §"No Slot Reservation"), warming cells were described as not holding competitive slots and the analysis set as containing only online cells. The spec now formalises that warming cells hold **investment slots** in $\mathcal{I}$ — they have resources committed — while not holding **production slots** in $\mathcal{A}$.

The spec also introduced the `g.sum` warm-up ordering (§ALGO S-11.6.2) and eager removal (§ALGO S-8.5) as named concepts, and requires three new reporting fields (§ALGO S-14.11–14.12).

### What already works · `sec:sentinel:investment-already-works`

The implementation's behaviour is **functionally correct** for the new spec:

- `AnalysisSet::recompute()` selects competitors and closes under ancestry — computing $\mathcal{T}$ then $\mathcal{I}$.
- The staging area holds warming cells with allocated trackers — this *is* the investment commitment.
- `retain_in_set()` implements eager removal — evicting warming and in-flight cells that left the analysis set.
- The background warming thread prioritises by `volume` (which *is* `g.sum` — `info.sum.to_f64_approx()`).
- Coordination contexts are demand-driven (lazy creation/pruning), which is equivalent to the spec's explicit activate/deactivate calls.

### What needs to change · `sec:sentinel:investment-needs-change`

1. **Reporting fields.** The spec defines three fields that do not exist in the implementation:
   - `HealthReport`: `investment_set_size`, `warming_trackers`, `warming_competitive_targets`.
   - `AnalysisSetSummary`: `investment_set_size`.

2. **Synchronous drain ordering.** `drain_all_synchronous()` processes cells in `BTreeMap` key order (ascending `GNodeId`). The spec requires g.sum order (§ALGO S-11.6.2). In practice this is immaterial — synchronous mode warms all cells to completion before any participate — but the code should match the spec for consistency and to avoid a latent bug if incremental synchronous warm-up is ever added.

3. **Source terminology.** Source comments, doc comments, and `implementation.md` use the pre-update names ("analysis set", "competitive set", "full analysis set"). These should be updated to use the new vocabulary where appropriate.

## Decision · `sec:sentinel:investment-decision`

### 1. Add investment-aware reporting fields · `sec:sentinel:investment-reporting-fields`

**`HealthReport`** gains three fields:

```rust
/// Total cells with allocated trackers (online + warming):
/// $|\mathcal{I}|$.
pub investment_set_size: usize,

/// Members of $\mathcal{I}$ currently in the warm-up pipeline.
pub warming_trackers: usize,

/// Competitive targets ($\mathcal{T}$) not yet promoted to
/// $\mathcal{A}$ — i.e. warming cells that are competitive
/// targets, not ancestors.
pub warming_competitive_targets: usize,
```

**`AnalysisSetSummary`** gains one field:

```rust
/// Total investment set size: $|\mathcal{I}|$ (online + warming).
pub investment_set_size: usize,
```

These are derived from `cells.len()` (online) plus staging area counts. The staging area already tracks per-cell `is_competitive`, so distinguishing warming ancestors from warming competitive targets requires no new state.

### 2. Fix synchronous drain ordering · `sec:sentinel:investment-synchronous-drain-ordering`

`drain_all_synchronous()` will sort cells by cached `volume` (descending) before draining, matching the background thread's g.sum priority rule.

### 3. Align source terminology · `sec:sentinel:investment-source-terminology`

Source comments and `implementation.md` will be updated to use the new terms. No public API names are changed in this ADR — the Rust field names (`competitive_size`, `full_size`, etc.) remain stable. The doc comments on those fields will clarify the spec mapping.

### 4. Accepted deviations · `sec:sentinel:investment-accepted-deviations`

Two implementation patterns differ from the spec's pseudocode but are behaviourally equivalent:

- **Lazy coordination lifecycle.** The spec's Step 3 pseudocode explicitly calls `ActivateCoordinationContexts` / `DeactivateCoordinationContexts`. The implementation creates and prunes coordination contexts on demand during `propagate_coordination_from_root()`. This is equivalent because contexts only affect output when their subtrees have contributing cells.

- **Full recompute vs. incremental reconciliation.** The spec's pseudocode computes `OldInvestment \ NewInvestment` and `NewInvestment \ OldInvestment` incrementally. The implementation recomputes the full analysis set from scratch (ADR-S-006), then reconciles the diff at the orchestrator level. The result is identical.

## Consequences · `sec:sentinel:investment-consequences`

- Hosts gain visibility into the investment/production distinction through three new report fields.

- The synchronous drain path matches the spec's ordering guarantee, eliminating a class of potential bugs if synchronous warm-up is ever made incremental.

- ADR-S-017's "No Slot Reservation" section is superseded: warming cells now formally hold investment slots in $\mathcal{I}$, though not production slots in $\mathcal{A}$.

- No public API breaks. Existing field names are preserved; new fields are additive.
