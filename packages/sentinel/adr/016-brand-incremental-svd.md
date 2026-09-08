# ADR-S-016: Brand's Incremental SVD for Subspace Evolution · `rec:sentinel:brand-incremental-svd-for-subspace-evolution`

**Status:** Implemented **Date:** 2026-03-10 **Spec:** §ALGO S-5.2 Phase 2 (subspace evolution) **Relates to:** [ADR-S-015](015-cell-creation-performance.md) (cell creation performance), [ADR-S-007](007-automatic-noise-injection.md) (automatic noise injection), [ADR-S-013](013-warm-up-convergence-benchmark.md) (warm-up convergence benchmark)

## Context · `sec:sentinel:brandsvd-context`

### The hot loop · `sec:sentinel:brandsvd-hot-loop`

Every call to `SubspaceTracker::observe()` invokes `evolve_subspace()`, which performs Phase 2 of the five-phase core loop (§ALGO S-5.2).  Convergence benchmarks (ADR-S-015) show that **`observe()` accounts for 99.9% of wall-clock time**, and Phase 2 (the SVD) dominates `observe()`.

### Previous implementation · `sec:sentinel:brandsvd-previous-implementation`

`evolve_subspace()` built the composite matrix

$$M = \bigl[\;\sqrt{\lambda}\, U_k \operatorname{diag}(\sigma_{1..k}) \;\big|\; X^\top\;\bigr] \;\in\; \mathbb{R}^{d \times (k+b)}$$

and computed a **full dense thin SVD** of $M$ via `faer::Mat::thin_svd()`.  This produced all $\min(d,\, k{+}b)$ singular triplets, of which only the top $n = \min(k{+}b,\, d,\, \text{cap})$ were retained.

### Cost analysis · `sec:sentinel:brandsvd-cost-analysis`

The thin SVD of a $d \times c$ matrix (where $c = k{+}b$) via bidiagonalisation + divide-and-conquer costs $O(d \, c^2)$ flops, plus $O(d \, c \, n)$ for back-transforming the $n$ left singular vectors from Householder form.

At representative dimensions:

| Config | d | k | b | M shape | SVD cost term | Per-round measured |
|--------|---|---|---|---------|---:|---:|
| bench | 128 | 2 | 4 | 128 × 6 | $128 \cdot 36$ | ~9 ms |
| realistic | 128 | 2 | 16 | 128 × 18 | $128 \cdot 324$ | ~32 ms |
| production (wide) | 128 | 16 | 16 | 128 × 32 | $128 \cdot 1024$ | est. ~90 ms |

With `noise_rounds = 100` per cell and multiple cells created on the hot path (ADR-S-015), the cumulative SVD cost is the dominant throughput bottleneck.

### What faer offers · `sec:sentinel:brandsvd-faer-primitives`

faer 0.24.0 provides:

- **`Mat::thin_svd()`** — dense thin SVD (bidiag + D&C).  No option to compute only the top-$k$ triplets.
- **`matrix_free::eigen::partial_svd()`** — Lanczos-based partial SVD for implicit operators (`&dyn BiLinOp<T>`).  Designed for large sparse matrices; slower than direct decomposition at our dimensions.
- **`linalg::qr::no_pivoting::qr_in_place()`** — dense thin QR.
- Dense matrix multiply via `&A * &B` operator overloads.

faer does **not** provide an incremental/streaming SVD.  However, it provides all the primitives needed to implement one.

## Decision · `sec:sentinel:brandsvd-decision`

Replace the naïve dense thin SVD in `evolve_subspace()` with **Brand's incremental SVD** (Brand 2002, 2006), using faer's existing QR and SVD primitives on a much smaller kernel matrix.

### Algorithm · `sec:sentinel:brandsvd-algorithm`

Given the current rank-$k$ model $(U_k, \sigma_{1..k})$ and a new batch $X \in \mathbb{R}^{b \times d}$:

**Step 1 — Project onto current basis:**

$$P = U_k^\top X^\top \;\in\; \mathbb{R}^{k \times b}$$

**Step 2 — Compute orthogonal residual:**

$$Q = X^\top - U_k\, P \;\in\; \mathbb{R}^{d \times b}$$

**Step 3 — Thin QR of residual:**

$$Q = Q_\perp\, R_\perp, \quad Q_\perp \in \mathbb{R}^{d \times b},\; R_\perp \in \mathbb{R}^{b \times b}$$

**Step 4 — Small-kernel SVD:**

$$K = \begin{bmatrix} \sqrt{\lambda}\,\operatorname{diag}(\sigma_{1..k}) & P \\ 0 & R_\perp \end{bmatrix} \;\in\; \mathbb{R}^{(k+b) \times (k+b)}$$

$$\hat{U}_K,\; \hat{\sigma},\; \hat{V}_K = \operatorname{thin\_svd}(K)$$

This SVD is on a **$(k{+}b) \times (k{+}b)$** matrix — e.g. 18×18 instead of 128×18.

**Step 5 — Back-transform to full basis:**

$$U_\text{new} = \bigl[\, U_k \;\big|\; Q_\perp \,\bigr] \;\hat{U}_K[:,\, :n] \;\in\; \mathbb{R}^{d \times n}$$

where $n = \min(k{+}b,\, d,\, \text{cap})$.

### Cost comparison · `sec:sentinel:brandsvd-cost-comparison`

| Operation | Current (naïve) | Brand's | Ratio |
|-----------|---:|---:|---:|
| Build $M$ | $O(d \cdot c)$ | — | — |
| Projection $P = U_k^\top X^\top$ | — | $O(d \cdot k \cdot b)$ | new |
| Residual $Q$ | — | $O(d \cdot k \cdot b)$ | new |
| Thin QR of $Q$ | — | $O(d \cdot b^2)$ | new |
| SVD kernel | $O(d \cdot c^2)$ | $O(c^3)$ | **$d/c$ ≈ 7×** |
| Back-transform $U_\text{new}$ | $O(d \cdot c \cdot n)$ | $O(d \cdot c \cdot n)$ | same |

### Phase 1 projection reuse · `sec:sentinel:brandsvd-projection-reuse`

Phase 1 of `observe()` already computes:

```
z     = X · U_k        // (b × k) — the latent coordinates
x_hat = z · U_k^T      // (b × d)
residual = X - x_hat    // (b × d)
```

Brand's Step 1 needs $P = U_k^\top X^\top = Z^\top$, and Step 2 needs $Q = X^\top - U_k P = \text{residual}^\top$.  Both are already computed in Phase 1.  Passing `z` and `residual` into `evolve_subspace()` eliminates the redundant matrix builds entirely.

## Implementation · `sec:sentinel:brandsvd-implementation`

### Source layout · `sec:sentinel:brandsvd-source-layout`

| File | Purpose |
|------|---------|
| `maths/mod.rs` | `SvdStrategy` enum, `evolve()` dispatch + oracle |
| `maths/brand_svd.rs` | Brand's incremental SVD (~130 lines) |
| `maths/naive_svd.rs` | Naïve dense thin SVD (reference) |
| `maths/bench_tracing.rs` | `SpanTimingLayer` for benchmark timing |
| `maths/tests/brand_vs_naive.rs` | 14 oracle unit tests |
| ~~`convergence_benchmark.rs`~~ | Removed — timing ported to criterion `warmup_cost_detailed`; diagnostics to `convergence_diagnostics.rs` |

### Strategy pattern · `sec:sentinel:brandsvd-strategy-pattern`

`SvdStrategy` is a runtime-selectable enum (`Naive` | `Brand`, default `Brand`) stored in `SentinelConfig`.  The `evolve()` function dispatches via `run_svd()`, which wraps each strategy call in a named `info_span!` (`"svd_brand"` or `"svd_naive"`) for timing capture.

### Debug oracle · `sec:sentinel:brandsvd-debug-oracle`

When `cfg!(debug_assertions)` is true (debug builds) **or** a `DEBUG`-level tracing subscriber is attached (release with `RUST_LOG=debug`), `evolve()` runs **both** strategies and compares their outputs:

- Singular values: relative tolerance $10^{-8}$
- Basis columns: $|\cos \theta| > 1 - 10^{-6}$ (allowing SVD sign ambiguity)

A **thread-local `ORACLE_FLIP`** toggle alternates execution order on every call, so neither strategy consistently benefits from warmed caches or branch predictors.

### Benchmark timing · `sec:sentinel:brandsvd-benchmark-timing`

`SpanTimingLayer` (in `bench_tracing.rs`) accumulates per-span-name wall-clock durations without console output.  The benchmarks read timing programmatically via `timing.total_ns("svd_brand")`.  An `EnvFilter` (default INFO) controls whether the oracle fires — `RUST_LOG=debug` activates it in release mode.

## Measured Results · `sec:sentinel:brandsvd-measured-results`

All measurements on d=128, 500 rounds, `convergence_matches_theory` benchmark.

### Test config (λ=0.95, b=4) · `sec:sentinel:brandsvd-test-config`

| Run mode | Brand (ms) | Naïve (ms) | Speedup |
|----------|-----------|-----------|---------|
| Debug (opt=3, oracle on) | 2188 | 4203 | **1.92×** |
| Release (oracle off) | 1977 | — | — |
| Release + RUST_LOG=debug | 1944 | 3731 | **1.92×** |

### Production config (λ=0.99, b=16) · `sec:sentinel:brandsvd-production-config`

| Run mode | Brand (ms) | Naïve (ms) | Speedup |
|----------|-----------|-----------|---------|
| Debug (opt=3, oracle on) | 4703 | 13998 | **2.98×** |
| Release (oracle off) | 5538 | — | — |
| Release + RUST_LOG=debug | 5283 | 15149 | **2.87×** |

### Summary · `sec:sentinel:brandsvd-summary`

Brand's incremental SVD delivers a consistent **~1.9× speedup at b=4** and **~2.9× speedup at b=16** (production config), scaling as expected with the $d/c$ ratio in the SVD kernel.  The speedup is stable across debug and release builds.

## Consequences · `sec:sentinel:brandsvd-consequences`

### Performance · `sec:sentinel:brandsvd-performance`

- **SVD kernel shrank from $(d \times c)$ to $(c \times c)$.** For production configs this is 128×18 → 18×18, a measured ~2.9× reduction in `evolve_subspace()` cost.

- **Phase 1 projection is reused**, eliminating redundant $O(d \cdot k \cdot b)$ matrix multiplications from Phase 2.

- **Per-cell noise injection cost (ADR-S-015) drops proportionally.**  At 100 noise rounds and ~2.9× speedup on the dominant step, cell creation time drops significantly.

- **A new thin QR is added** — $O(d \cdot b^2)$ per round.  This is cheaper than the SVD it replaces and has lower constant factors (direct Householder, no iteration).

### Numerical equivalence · `sec:sentinel:brandsvd-numerical-equivalence`

Brand's incremental SVD is mathematically equivalent to the naïve approach — both compute the thin SVD of the same composite matrix $M$.  The difference is purely in how the computation is structured.

The 14 oracle tests in `maths/tests/brand_vs_naive.rs` verify equivalence across a range of configurations:

- Minimal dimensions, identity basis, rank-one, large batch
- Saturated rank, large singular values, near-zero residual
- Multi-step sequences (10 consecutive evolve calls)
- Representative configs matching both benchmark profiles
- Sorted/non-negative singular values, orthonormal output basis

The debug oracle runs on **every** `evolve()` call in test builds, providing continuous regression coverage.

### Complexity · `sec:sentinel:brandsvd-complexity`

- The change is confined to `maths/` and the call site in `observe()` (to pass `z` and `residual` instead of `x`).

- No new dependencies.  Uses `faer::Mat::thin_svd()` for the small kernel, and `faer::Mat::qr()` for the residual QR.

- ~130 lines of new code (`brand_svd.rs`) alongside the existing naïve implementation (~60 lines in `naive_svd.rs`).

### Risks · `sec:sentinel:brandsvd-risks`

- **Rank growth beyond $b$:** When $k > b$, the kernel is $(k{+}b) \times (k{+}b)$ which can grow up to $(\text{cap}{+}b)$. At cap=16, b=16 this is 32×32 — still far smaller than 128×32. The speedup is always at least $d/(k{+}b)$.

- **QR numerical stability:** If the residual $Q$ is nearly zero (new data lies almost entirely in the current subspace), the QR may produce near-zero $R_\perp$ entries.  This is handled naturally — the small singular values from those directions will be discarded by rank adaptation (Phase 5).  No special-casing needed.  The `equivalence_near_zero_residual` test validates this case explicitly.

## References · `sec:sentinel:brandsvd-references`

- Brand, M. (2002). "Incremental Singular Value Decomposition of Uncertain Data with Missing Values." *ECCV 2002.*
- Brand, M. (2006). "Fast low-rank modifications of the thin singular value decomposition." *Linear Algebra and its Applications*, 415(1), 20–30.
- Baker, C.G., Gallivan, K.A., Van Dooren, P. (2012). "Low-Rank Incremental Methods for Computing Dominant Singular Subspaces." *Linear Algebra and its Applications*, 436(8), 2866–2888.
