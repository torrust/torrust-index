# ADR-S-021: EWMA-Mean-Centred Latent Variance · `rec:sentinel:ewma-mean-centred-latent-variance`

**Status:** Implemented **Date:** 2026-03-24 **Spec:** §ALGO S-4.2 (Phase 3 — Evolve Latent Distribution), §ALGO S-5.4 (surprise scoring), §ALGO S-11.2 (cold-start latent seeding), §ALGO S-Appendix A (convergence methodology) **Relates to:** [ADR-S-013](013-warm-up-convergence-benchmark.md) (warm-up convergence benchmark — introduced the cold→warm fix this ADR amends)

## Context · `sec:sentinel:latentvar-context`

The algorithm specification (§ALGO S-4.2, Phase 3) has been amended to replace the **within-batch population variance** estimator for $\nu^{(z)}$ with an **EWMA-mean-centred** estimator.  The implementation in `SubspaceTracker::evolve_latent()` still uses the old formula.

### Problem · `sec:sentinel:latentvar-problem`

The within-batch population variance $\operatorname{Var}(Z_{:,j}) = \frac{1}{b}\sum_i (z_{ij} - \bar{Z}_j)^2$ has a systematic negative bias of $\frac{b-1}{b}$ relative to the surprise numerator's expected value:

| Batch size $b$ | Bias          | Effect                                              |
| -------------- | ------------- | --------------------------------------------------- |
| 1              | $-100\%$      | Variance erodes to $\varepsilon$; surprise → $10^5$ |
| 2              | $-50\%$       | Severe underestimate; surprise doubled               |
| 4              | $-25\%$       | Material; surprise inflated by 33%                   |
| 16             | $-6.25\%$     | Significant at precision targets                     |
| 64+            | $< -1.6\%$    | Negligible                                           |

The surprise score divides by $\nu^{(z)}_j + \varepsilon$, so the underestimate inflates surprise scores.  At $b = 1$ (which occurs at per-cell trackers receiving one observation per ingestion cycle), the variance collapses to zero every batch, the EWMA decays toward $\varepsilon$, and per-dimension surprise contributions reach $O(10^5)$.  At $b = 2$ (coordination trackers with two contributing cells), the $-50\%$ bias produces a sustained $2\times$ surprise overestimate.

This is not a hypothetical concern in practice: a host that ingests **one observation per call** ($b = 1$ always at the root tracker) operates at the worst-case batch size for this bias.

### Solution in the spec · `sec:sentinel:latentvar-spec-solution`

The amended formula computes deviations from the **EWMA mean** $\mu^{(z)}_j$ rather than the batch mean $\bar{Z}_j$:

**Subsequent batches ($t > 0$):**

$$\nu^{(z)}_j \leftarrow \lambda\,\nu^{(z)}_j + \alpha \cdot \max\!\left(\frac{1}{b}\sum_{i=1}^{b}(z_{ij} - \mu^{(z)}_j)^2,\;\varepsilon\right)$$

**First batch ($t = 0$):**

$$\nu^{(z)}_j \leftarrow \max\!\left(\frac{1}{b}\sum_{i=1}^{b} z_{ij}^2,\;\varepsilon\right)$$

(using the pre-allocated $\mu^{(z)} = \mathbf{0}$ as centring reference, which makes the $t = 0$ formula a special case of the $t > 0$ formula).

The spec also mandates:

1. **Update order**: $\nu^{(z)}$ first (against pre-update $\mu$), then $\mu^{(z)}$, then $\Gamma$.  Variance sees the same $\mu$ that scoring (Phase 1) used.

2. **Runtime floor**: $\nu^{(z)}_j \leftarrow \max(\nu^{(z)}_j, 10^{-2})$ after every update (including $t = 0$ seeding).  Defence-in-depth against degenerate streams; caps per-dimension surprise at 100.

### Why the EWMA-mean-centred formula eliminates the bias · `sec:sentinel:latentvar-bias-elimination`

The within-batch component contributes $\frac{b-1}{b}\sigma^2$ (same as the old formula).  The batch-mean-vs.-EWMA-mean component contributes $\frac{\sigma^2}{b}$.  They sum to $\sigma^2$ regardless of $b$ — exact cancellation of the $\frac{b-1}{b}$ bias.

The residual bias is $\operatorname{Var}(\mu^{(z)}_j) \approx \frac{\alpha}{b(1+\lambda)}\sigma^2$ — positive (safe direction: dampens surprise), small, worst-case $+0.5\%$ at $b = 1$, $\lambda = 0.99$.

### Trade-offs accepted · `sec:sentinel:latentvar-accepted-tradeoffs`

- **Coupling to $\mu^{(z)}$**: stale $\mu$ inflates $\nu$, dampening surprise during regime transitions.  This is the safe direction.  Displacement is the primary mean-shift detector; surprise's role is distributional shape anomalies (preserved).

- **Basis rotation sensitivity**: after Phase 2 rotates the basis, projections are on the new basis while $\mu$ reflects the old. $(z - \mu)^2$ inflates, $\nu$ inflates, surprise dampens. Transient, $O(t_{1/2})$ batches.  The old formula was invariant (re-centring on $\bar{Z}_j$ subtracted out any rotation-induced mean shift).

## Decision · `sec:sentinel:latentvar-decision`

Implement the amended Phase 3 in `SubspaceTracker::evolve_latent()` and update impacted tests.

### What already works · `sec:sentinel:latentvar-already-works`

- **Surprise scoring formula** (Phase 1): `(z_ij - lat_mean[j])² /
  (lat_var[j] + eps)` — already uses lat_mean as its centring reference.  No change needed.

- **Pre-allocated initial values**: `lat_mean = 0.0`, `lat_var = 1.0`, `cross_corr = 0.0` — unchanged.

- **Cross-correlation ($\Gamma$) update**: uses raw second moments $z_{ij} \cdot z_{il}$, not centred products.  No change needed.

- **Rank-change behaviour**: pre-allocated entries at capacity, EWMA loops iterate `0..k`.  No change needed.

### What needs to change · `sec:sentinel:latentvar-needs-change`

The implementation has **5 gaps** between the current code and the amended spec:

1. **Variance formula (subsequent batches).** In `evolve_latent()`, the `col_var` computation currently centres on the batch mean:

   ```rust
   let mut col_var = 0.0;
   for i in 0..b {
       let d = z[(i, j)] - col_mean;
       col_var += d * d;
   }
   col_var /= b_f;
   ```

   Must change to centre on `self.lat_mean[j]` (the pre-update EWMA mean):

   ```rust
   let mut col_var = 0.0;
   for i in 0..b {
       let d = z[(i, j)] - self.lat_mean[j];
       col_var += d * d;
   }
   col_var /= b_f;
   ```

2. **Variance formula (first batch).** The cold path currently uses within-batch variance centred on batch mean.  Must change to centre on the pre-allocated `lat_mean[j]` (which is 0.0 at $t = 0$):

   ```rust
   // Before (centres on batch mean col_mean):
   self.lat_var[j] = col_var.max(eps);
   // After (col_var already computed against lat_mean[j] = 0.0):
   self.lat_var[j] = col_var.max(eps);
   ```

   The seeded value changes from the within-batch population variance to $\frac{1}{b}\sum z_{ij}^2$.  The code change is in the `col_var` computation (gap 1 handles both paths since the loop uses the same formula).

3. **Update order.** The current code computes mean and variance in a single loop, updating both in the same branch:

   ```rust
   if cold {
       self.lat_mean[j] = col_mean;
       self.lat_var[j] = col_var.max(eps);
   } else {
       self.lat_mean[j] = lam.mul_add(self.lat_mean[j], alpha * col_mean);
       self.lat_var[j] = lam.mul_add(self.lat_var[j], alpha * col_var.max(eps));
   }
   ```

   Must reorder: update variance first (against pre-update mean), then update mean.  Because `col_var` is now computed against `self.lat_mean[j]` (read before any mutation), the natural order is:

   ```rust
   if cold {
       self.lat_var[j] = col_var.max(eps);
       self.lat_mean[j] = col_mean;
   } else {
       self.lat_var[j] = lam.mul_add(self.lat_var[j], alpha * col_var.max(eps));
       self.lat_mean[j] = lam.mul_add(self.lat_mean[j], alpha * col_mean);
   }
   ```

   This is a semantic change: the old order was mean-then-variance; the new order is variance-then-mean. Because `col_var` is now pre-computed against the pre-update `lat_mean[j]`, the reorder ensures the EWMA variance input uses the same reference as the surprise numerator.

4. **Runtime floor.** After the per-dimension loop (both cold and non-cold paths), clamp:

   ```rust
   for j in 0..k {
       self.lat_var[j] = self.lat_var[j].max(1e-2);
   }
   ```

   This is a new addition.  The floor is $25\times$ below the null-hypothesis value of $0.25$ and should never bind under correct operation.  It caps per-dimension surprise at 100.

5. **Test expectations.** Several existing tests assert against the old formula's behaviour:

   - `latent_variance_reaches_steady_state` — steady-state $\nu^{(z)}$ values will change (EWMA-mean-centred is slightly higher than within-batch at $b = 4$).  The assertion `v < 0.5` should still hold, but may need loosening if steady-state is higher.

   - `cold_warm_eliminates_surprise_nonstationarity` — the rise factor should improve (the new formula eliminates the $b$-dependent bias that contributed to the transient).

   - `subspace_evolution_does_not_dominate_latvar_transient` — the tolerance `diff_pct < 50.0` should still hold; verify.

   - `convergence_ewma::ewma_variance_convergence` — tests the `EwmaStats` type directly; unaffected (the EWMA machinery itself is unchanged).

   - `convergence_clipping` tests — EWMA variance assertions on the surprise baseline; should still hold but verify numerically.

### New tests · `sec:sentinel:latentvar-new-tests`

| Test                                                | Validates                                                                   |
| --------------------------------------------------- | --------------------------------------------------------------------------- |
| `b1_surprise_bounded`                               | At $b = 1$, surprise scores stay bounded (no $O(10^5)$ explosion)           |
| `b1_latent_variance_stable`                         | At $b = 1$, `lat_var` converges to $\approx 0.25$, not $\varepsilon$        |
| `b2_no_systematic_surprise_inflation`               | At $b = 2$, surprise ratio near 1.0 (no 2× inflation)                      |
| `runtime_floor_prevents_degenerate_collapse`        | Constant-observation stream: `lat_var` ≥ $10^{-2}$                          |
| `update_order_variance_before_mean`                 | Variance update uses pre-update mean (white-box: instrument `lat_mean` read) |
| `batch_size_invariant_surprise_ratio`               | Surprise ratio $\approx 1.0$ at $b \in \{1, 2, 4, 16, 64\}$ (self-consistency) |

### Impact on hosts · `sec:sentinel:latentvar-host-impact`

The change is transparent to consumers.  Hosts read `BatchReport` fields (z-scores, CUSUM accumulators, maturity) via the public API.  The internal formula change affects the *values* of those fields but not their types or semantics.

Behavioural effects for hosts operating at small batch sizes:

- **$b = 1$:** Surprise scores drop from potentially $O(10^5)$ to $O(1)$.  Any confidence quality factor derived from surprise will be dramatically more stable during warm-up and during production when the distribution is well-behaved.

- **Confidence convergence:** Faster.  The $4×$ surprise inflation at $b = 1$ under the old formula delayed baseline settling.  The new formula's self-consistency property means surprise starts near $1.0$ immediately.

- **CUSUM sensitivity:** Unchanged in steady state (the CUSUM reference tracks the same signal).  During transitions, surprise CUSUM drift is shorter-lived (the $\nu$ co-adaptation described in §ALGO S-4.2 dampens sustained elevation).

No consumer code changes are required.  The surprise formula references $\nu^{(z)}_j + \varepsilon$ in the denominator — unchanged.

### Summary of implementation steps · `sec:sentinel:latentvar-implementation-summary`

1. Modify `evolve_latent()` in [tracker.rs](../src/sentinel/tracker.rs):
   - Change `col_var` computation to centre on `self.lat_mean[j]` instead of `col_mean`.
   - Reorder assignments: variance before mean.
   - Add runtime floor loop after per-dimension updates.

2. Add new tests in a new test module `src/tests/variance_formula.rs` (or extend `convergence_noise.rs`) covering $b = 1$, $b = 2$, runtime floor, update order, and batch-size-invariant surprise ratio.

3. Verify and adjust existing test expectations as needed (tolerance bounds only; no logic changes expected).

4. Run full `--package sentinel` test suite in both debug and release modes.

5. Run `--workspace` tests to confirm no downstream breakage.

## Consequences · `sec:sentinel:latentvar-consequences`

- **Surprise scores become batch-size-invariant.**  The expected surprise ratio is $1 + O(\alpha^2)$ regardless of $b$.  This eliminates a class of false positives at small batch sizes and a class of false negatives at large batch sizes.

- **Hosts at $b = 1$ stabilise.**  The old formula produced catastrophic surprise inflation at this batch size; the new formula produces correctly-scaled scores from the first batch.

- **Regime-transition behaviour changes.**  Surprise spikes after mean shifts are shorter-lived.  The first batch fires at full strength; subsequent batches' $\nu$ co-adapts, progressively dampening the ratio.  Displacement inherits the primary detection role during transitions.

- **Convergence benchmarks (ADR-S-013) may need updating.** Convergence times at $b_{\text{noise}} = 4$ may improve (the $-25\%$ bias that contributed to displacement bimodality is eliminated).  At $b_{\text{noise}} \geq 16$, changes are negligible.

- **Runtime floor adds a hard safety bound.**  Under no circumstances can per-dimension surprise exceed 100.  This replaces the $\varepsilon$-floored worst case of $10^5$.
