# ADR-S-011: Degenerate Cell Dimension Guard · `rec:sentinel:exclude-and-report-degenerate-tracker-dimensions`

**Status:** Implemented **Date:** 2026-03-10 **Spec:** §ALGO S-4.2 (analysis set closure), §ALGO S-5 (subspace tracker) **Relates to:** [ADR-S-004](004-config-validation-over-panic.md) (config validation over panic), [ADR-S-006](006-analysis-set-recomputation.md) (analysis set recomputation), [ADR-S-007](007-automatic-noise-injection.md) (automatic noise injection)

## Context · `sec:sentinel:dimguard-context`

`SubspaceTracker` operates on suffix bit slices of width $w = N - d$, where $d$ is the G-tree bit depth of the cell. When $d = N$ (e.g. $d = 128$ at $N = 128$), $w = 0$: no suffix bits remain, and the tracker's working space is zero-dimensional.  Values of $w$ that are very small (1–2) are technically representable but yield degenerate subspace models with no practical statistical value.

### The bug · `sec:sentinel:dimguard-bug`

`SubspaceTracker::new(dim, cfg, slow_decay)` unconditionally sets `rank = 1` at construction.  However, the rank capacity is `cap = min(dim, max_rank)`.  When `dim = 0`:

- `cap = 0`, but `rank = 1` — **rank exceeds capacity**.
- `basis` is a $0 \times 0$ matrix.
- The first call to `observe()` attempts `self.basis.subcols(0, 1)` — extracting column 0..1 from a zero-column matrix — which **panics inside faer's SVD**.

This is not a theoretical concern.  Under spray traffic with a low `split_threshold`, the G-tree creates very deep nodes.  The analysis set's ancestor closure (§ALGO S-4.2) can pull these deep nodes into the tracked set, where `reconcile_analysis_set()` creates `SubspaceTracker::new(0, ...)` and the next `observe()` panics.

### Why the existing guards are insufficient · `sec:sentinel:dimguard-existing-guards-insufficient`

1. **Config validation** **([ADR-S-004](004-config-validation-over-panic.md))** mentions "Depth 128 rejection" as a constraint on `analysis_depth_cutoff`, but the cutoff filters on **V-tree depth**, not G-tree bit depth.  The ancestor closure walks the G-tree and can include nodes at any bit depth regardless of the cutoff.

2. **`max_rank >= 1`** is validated at config time, but `cap` is `min(dim, max_rank)`, so a valid `max_rank` does not prevent `cap = 0` at runtime.

3. **No runtime guard** exists in `SubspaceTracker::new()`, `reconcile_analysis_set()`, or the scoring path.  The panic propagates uncaught from faer into the host application.

## Decision · `sec:sentinel:dimguard-decision`

**The sentinel must detect degenerate cell dimensions at runtime and report the condition through its measurement API rather than panicking.**

### 1. Minimum dimension constant · `sec:sentinel:dimguard-minimum-dimension`

Introduce a crate-level constant:

```rust
/// Minimum suffix width for a functional subspace tracker.
///
/// At `dim < MIN_TRACKER_DIM`, the tracker cannot form a
/// meaningful basis or compute residuals.  Cells below this
/// threshold are excluded from the analysis set; a cell at it
/// is kept, which is what makes the value a minimum rather
/// than a floor the analysis set sits above.
pub(crate) const MIN_TRACKER_DIM: usize = 2;
```

The value 2 ensures at least one residual degree of freedom ($d - k \geq 1$ when $k = 1$).  A tracker with `dim = 1` can technically run but produces identically-zero residuals (novelty) since the single basis vector spans the entire space — making it statistically useless.

### 2. Guard in `reconcile_analysis_set()` · `sec:sentinel:dimguard-reconcile-guard`

Before creating a `CellState`, check:

```rust
let width = 128usize.saturating_sub(entry.depth as usize);
if width < MIN_TRACKER_DIM {
    tracing::warn!(
        gnode = %entry.gnode,
        depth = entry.depth,
        width,
        "skipping degenerate cell (width < MIN_TRACKER_DIM)"
    );
    continue;
}
```

Cells that fail the guard are **not tracked** — they receive no `SubspaceTracker`, appear in no reports, and their traffic still flows through the G-V graph normally (observation routing is unaffected).

### 3. Guard in `SubspaceTracker::new()` (defence in depth) · `sec:sentinel:dimguard-constructor-defence`

As a second line of defence, the constructor asserts internally:

```rust
debug_assert!(
    dim >= MIN_TRACKER_DIM,
    "SubspaceTracker::new() called with dim={dim}, \
     expected >= {MIN_TRACKER_DIM} (caller should have filtered)"
);
```

This is a `debug_assert!` (not a runtime error) because the reconciliation guard should make it unreachable.  If it fires in debug builds, it signals a missed filter site.

### 4. Report the condition · `sec:sentinel:dimguard-condition-reporting`

Add a counter to `AnalysisSetSummary`:

```rust
/// Number of G-tree nodes excluded from tracking because their
/// suffix width was below `MIN_TRACKER_DIM`.
pub degenerate_cells_skipped: usize,
```

This follows [ADR-S-001](001-measures-not-opinions.md): the sentinel **measures** the degenerate condition; the host decides whether to log, alert, or ignore it.

## Alternatives Considered · `sec:sentinel:dimguard-alternatives`

- **Cap the rank to `dim` and allow `dim = 0` gracefully.** Setting `rank = 0` when `cap = 0` avoids the panic, but a zero-rank tracker produces no meaningful scores — every axis returns zero.  Silent zeros in the report are worse than an explicit skip, because the host cannot distinguish "no anomaly" from "unable to measure".  The report would lie.

- **Clamp `dim` to a floor of 1 inside the tracker.**  This hides the degeneracy from the caller.  The 1-dimensional tracker produces identically-zero novelty scores (because the sole basis vector captures 100% of variance), misleading the host into thinking the cell is perpetually normal.

- **Reject configs whose parameters *could* produce depth-128 nodes.**  Impractical — whether depth 128 is reached depends on runtime traffic, not solely on config.  A `split_threshold` of 10 with budget 10 000 is a valid configuration that *might* hit depth 128 under unlucky traffic but usually does not.

- **Panic with a clear message.**  Violates [ADR-S-004](004-config-validation-over-panic.md).  The sentinel is a library inside the Torrust Index; panicking on a runtime traffic pattern is unacceptable.

## Consequences · `sec:sentinel:dimguard-consequences`

- **No more panics** from degenerate cell dimensions.  The faer SVD code path is never reached with a zero- or one-dimensional input.

- **`degenerate_cells_skipped`** in the report gives the host visibility into deep-tree pressure.  A persistently non-zero count may indicate the `split_threshold` is too low for the traffic mix.

- **Ancestor chain gaps.**  If a degenerate node is an ancestor of a competitive cell, the ancestor chain has a gap — no `CellReport` is emitted for that depth.  Coordination reports still function (they use the G-tree structure, not the analysis set), so the gap affects only per-cell scoring, not tree-wide propagation.

- **Performance: eliminates wasted SVD churn.**  Even when `dim` is small but positive (e.g. 1–3), the tracker still runs the full SVD pipeline on every `observe()` call: build the $(d \times (k + b))$ augmented matrix, compute thin SVD, update basis, evolve latent statistics.  With so few dimensions the rank adapter oscillates — it trivially meets the energy threshold at $k = 1$, bumps to $k = \text{cap}$ on the next interval, then drops back — rebuilding the basis each flip. The scores produced carry no meaningful statistical signal (novelty is identically zero at $d = 1$ since the single basis vector spans the entire space; at $d = 2$ the residual DOF is 1, giving trivially noisy scores).  Each degenerate cell therefore burns per-batch SVD cost for zero diagnostic value. In adversarial spray scenarios many deep cells can accumulate, multiplying this waste.  Skipping them at `MIN_TRACKER_DIM` eliminates the churn entirely.

- **`MIN_TRACKER_DIM` is a compile-time constant**, not a config field.  It reflects an inherent limitation of the linear algebra (need $\geq 2$ dimensions for a non-trivial subspace), not a tuning knob.  The default value should be **conservative** — set high enough to avoid not only the panic ($w = 0$) but also the SVD churn regime described above.  A value of 3 or 4 would be defensible (guaranteeing $\geq 2$ residual DOF at rank 1, and room for rank adaptation without instant saturation), but 2 is the theoretical minimum that prevents the panic and provides at least one residual degree of freedom.  If profiling reveals measurable SVD overhead from shallow cells in production traffic, raising the constant to 4 is a safe, backward- compatible change — it only removes cells that were contributing noise to the report anyway.
