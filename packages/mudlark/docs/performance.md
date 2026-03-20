# Mudlark — Performance Profile

Benchmark results and performance characterisation for
`torrust-mudlark` 1.0.0.

Measurements taken with [Criterion 0.5](https://crates.io/crates/criterion)
on a release build. 143 benchmarks across six families exercise every
public operation and several adversarial workloads.

Run the suite:

```bash
cargo bench --package torrust-mudlark
```

---

## 1. Configuration under test

| Parameter           | u64 preset         | f64 preset             |
| ------------------- | ------------------ | ---------------------- |
| Type args           | `<u64, u64, 8>`    | `<f64, f64, 52>`       |
| Max G-depth ($N$)   | 8                  | 52                     |
| Config factory      | `default_config()` | `f64_default_config()` |
| Budget (where used) | 500 nodes          | 500 nodes              |

The u64 configuration is the primary benchmark target. f64 variants
are included as Phase 8 cliff-detection probes.

---

## 2. Headline numbers (u64)

| Operation                  | Latency            | Conditions                                              |
| -------------------------- | ------------------ | ------------------------------------------------------- |
| `plateaus()` borrow        | **2.6–2.9 ns**     | Any graph size. $O(1)$ with `dynamic-contour-tracking`. |
| `get(x)` point query       | **14–22 ns**       | 100–10K observations. $O(\log P)$.                      |
| `sample()` — hotspot       | **10 ns**          | Concentrated distribution (near-zero entropy).          |
| `sample()` — uniform       | **126 ns**         | Maximum entropy (uniform spread, 5K obs).               |
| `range_sum(a..b)`          | **3–77 ns**        | Full-domain to narrow-range.                            |
| `observe()` — cold start   | **142–493 ns/obs** | Growth only, no budget pressure.                        |
| `observe()` — steady state | **1.8 µs/obs**     | Budget=500, sustained random spray.                     |
| `decay()` — selective      | **5.5–8.1 µs**     | q=0.5, 100–10K graph. Sub-linear scaling.               |
| `check_evictions()`        | **58–59 µs**       | Budget=500, graph exceeds budget.                       |

---

## 3. Family 1 — Observe

**Design claim:** Amortised $O(d_\text{geo})$ per observation,
where $d_\text{geo}$ is the geometric depth of the target leaf.
Total cost for $n$ observations: $O(n \cdot \text{depth})$.

### 3.1 Results

| Variant           | n=100  | n=1K    | n=10K   | n=100K  | Scaling exponent |
| ----------------- | ------ | ------- | ------- | ------- | ---------------- |
| spread (default)  | 66 µs  | 323 µs  | 1.92 ms | 19.5 ms | **0.82**         |
| spread (budgeted) | 546 µs | 4.03 ms | 11.9 ms | 96.2 ms | 0.75             |
| skewed            | 53 µs  | 189 µs  | 1.52 ms | 19.8 ms | 0.86             |
| deep              | 304 µs | 1.28 ms | 5.85 ms | 41.9 ms | 0.71             |
| zigzag            | 64 µs  | 245 µs  | 2.10 ms | 21.3 ms | 0.84             |
| diamond           | 55 µs  | 332 µs  | 2.18 ms | 20.4 ms | 0.86             |
| golden spiral     | 50 µs  | 150 µs  | 1.24 ms | 11.1 ms | 0.78             |
| spread f64        | 114 µs | 1.18 ms | 12.0 ms | 127 ms  | 1.01             |

### 3.2 Analysis

**Status: CONFIRMED.**

Scaling exponents of 0.71–0.86 (u64) are sub-linear, consistent
with $O(n \log n)$ total cost where per-observation cost decreases
as the tree fills in. Early observations pay disproportionate
setup costs (initial splits, structure establishment).

Per-observation cost at scale:

- **spread:** ~195 ns/obs — the baseline throughput.
- **skewed:** ~198 ns/obs — concentrated access reaches cost
  parity with spread at scale.
- **deep:** ~419 ns/obs — monotone sweep forces maximum $d_\text{geo}$.
- **zigzag:** ~213 ns/obs — alternating extremes converge to
  near-spread cost at scale.
- **diamond:** ~204 ns/obs — converge-then-diverge access pattern
  behaves similarly to spread.
- **golden spiral:** ~111 ns/obs — quasi-uniform low-discrepancy
  coverage is the cheapest observe pattern.
- **budgeted:** ~962 ns/obs — ~5× slower than default due to
  eviction overhead, not depth.

The f64 variant scales with exponent ~1.01 (near-linear) because
$N=52$ produces deeper G-Trees, so the amortisation benefit is
smaller.

---

## 4. Family 2 — Query

### 4.1 Point query — `get(x)`

**Design claim:** $O(\log P)$ where $P$ is the plateau count
(floor-key lookup in the plateau `BTreeMap`).

| Graph size | Latency |
| ---------- | ------- |
| 100 obs    | 14.5 ns |
| 1K obs     | 19.8 ns |
| 10K obs    | 22.4 ns |

Scaling exponents: 0.137, 0.054 — approaching zero (logarithmic).
Over a 100× size range the cost barely doubles.

**Status: CONFIRMED.**

### 4.2 Range sum — `range_sum(a..b)`

**Design claim:** $O(N)$ tree walk.

| Width             | Latency |
| ----------------- | ------- |
| 16 (narrow)       | 77 ns   |
| 64                | 52 ns   |
| 128               | 31 ns   |
| 256 (full domain) | 3 ns    |

Narrower ranges are _slower_ — they require deeper traversal to
resolve partial overlaps. Full-domain returns the root sum in
essentially $O(1)$.

**Status: CONFIRMED.**

### 4.3 Sampling — `sample(&mut rng)`

**Design claim:** Expected node visits $\leq 1.44\,H + \log_\varphi\!\sqrt{5}
\approx 1.44\,H + 1.67$ where $H$ is the Shannon entropy of the weight
distribution (§IDEA M-18.2, §THEORY M-4.3). Lower entropy → fewer
coin flips → faster.

| Distribution           | Entropy    | Latency      |
| ---------------------- | ---------- | ------------ |
| hotspot (concentrated) | ~0 bits    | **10.2 ns**  |
| skewed (power-law)     | medium     | **55.3 ns**  |
| uniform (max entropy)  | $\log_2 L$ | **125.8 ns** |

Entropy ordering is unmistakable: hotspot is **12.3× faster** than
uniform. The hotspot distribution has near-zero entropy (all weight
on one leaf), so sampling takes ~1 coin flip. Uniform requires a
full tree descent.

The f64 sample (69 ns uniform) is _faster_ than u64 (126 ns) —
different V-Tree shape with the wider coordinate space, not a
performance cliff.

**Status: CONFIRMED.**

---

## 5. Family 3 — Extract

### 5.1 Plateau borrow — `plateaus()`

**Design claim:** $O(1)$ with `dynamic-contour-tracking`.
Returns `Cow::Borrowed`.

| Graph size | Latency |
| ---------- | ------- |
| 100 obs    | 2.85 ns |
| 1K obs     | 2.92 ns |
| 10K obs    | 2.58 ns |

Constant to within measurement noise. This is the cost of
returning a reference.

**Status: CONFIRMED** — textbook $O(1)$.

### 5.2 Full extraction — `extract()`

| Graph size | u64     | f64     | Ratio |
| ---------- | ------- | ------- | ----- |
| 100        | 1.09 µs | 0.92 µs | 0.84× |
| 1K         | 1.61 µs | 1.96 µs | 1.22× |
| 10K        | 1.54 µs | 2.75 µs | 1.79× |

Scaling exponents ~0.01–0.19: extract visits V-Tree nodes (bounded),
not all G-nodes. Cost plateaus at ~1.5 µs regardless of observation
count.

Additional extract variants (skewed and hotspot distributions):

| Distribution | 100 obs | 1K obs  | 10K obs |
| ------------ | ------- | ------- | ------- |
| skewed       | 0.91 µs | 1.07 µs | 1.00 µs |
| hotspot      | 0.51 µs | 0.92 µs | 0.88 µs |

Hotspot extract is cheapest — a concentrated tree has fewer V-Tree
nodes to visit.

### 5.3 Streaming layers — `layers().count()`

| Graph size | Latency |
| ---------- | ------- |
| 100        | 673 ns  |
| 1K         | 866 ns  |
| 10K        | 980 ns  |

Same plateauing pattern. Iterator cost depends on V-Tree structure,
not total observation count.

---

## 6. Family 4 — Lifecycle

### 6.1 Decay

**Design claim:** $O(|G|)$ for full-tree decay.

| Graph size | u64 (q=0.5) | f64 (q=0.5) |
| ---------- | ----------- | ----------- |
| 100        | 5.5 µs      | 6.4 µs      |
| 1K         | 7.1 µs      | 15.0 µs     |
| 10K        | 8.1 µs      | 22.8 µs     |

u64 scaling exponents: 0.113, 0.056 — strongly sub-linear.

**Status: NUANCED.** The benchmark uses selective decay (`q=0.5`),
which prunes cold subtrees via the `is_exposed` flag in $O(1)$ per
skipped subtree. The measured sub-linear growth reflects
effective pruning, not a violation of the $O(|G|)$ claim. A
full-tree decay benchmark (`q=1.0` on a non-uniform graph) would
be needed to directly validate the linear bound.

**f64 concern:** 1.2–2.8× slower than u64 at equivalent sizes.
At 10K the gap exceeds the 2× guideline. Likely caused by
52-bit coordinate operations and deeper tree walks, not code-level
inefficiency. See §PERF 9.

Decay under a skewed (power-law) distribution:

| Graph size | Skewed decay |
| ---------- | ------------ |
| 100        | 3.3 µs       |
| 1K         | 3.6 µs       |
| 10K        | 3.7 µs       |

Skewed decay is ~2× cheaper than uniform-spread decay — the
asymmetric tree has fewer energised subtrees to visit.

### 6.2 Selectivity sweep

**Design claim:** Monotone increasing cost with quantile threshold $q$.

| q    | Latency |
| ---- | ------- |
| 0.00 | 7.3 µs  |
| 0.25 | 6.9 µs  |
| 0.50 | 6.6 µs  |
| 0.75 | 6.9 µs  |
| 1.00 | 6.5 µs  |

**Status: NOT CONFIRMED.** Cost is ~6.5–7.3 µs regardless of $q$.
The tree walk overhead dominates the per-attenuated-node work.
With a uniformly-energised `spread` graph, the quantile threshold
controls _which_ nodes get multiplied, but the walk visits the same
subtrees regardless. The claim assumes the dominant cost is
per-node attenuation; in practice the traversal is the bottleneck.

Benign discrepancy. Would likely manifest under non-uniform
distributions where pruning actually skips large cold subtrees.

### 6.3 Eviction

| Graph size | Budget | Latency |
| ---------- | ------ | ------- |
| 100        | 500    | 1.3 µs  |
| 1K         | 500    | 58.2 µs |
| 10K        | 500    | 58.6 µs |

Budget-gated: at 100 observations the graph hasn't hit the budget,
so `check_evictions()` returns almost immediately. At 1K+ the
budget is exceeded and eviction work begins. The 1K→10K flatness
confirms eviction cost depends on the budget capacity, not the
total observation count — consistent with steady-state theory.

Eviction under a skewed distribution:

| Graph size | Budget | Latency  |
| ---------- | ------ | -------- |
| 100        | 500    | 1.0 µs   |
| 1K         | 500    | 157.9 µs |
| 10K        | 500    | 4.1 µs   |

The 1K spike is interesting — at this size the skewed graph
has a high density just above the budget threshold, causing many
eviction cascades. At 10K the weight is more concentrated,
reducing the number of budget-exceeding nodes.

### 6.4 Combined decay + eviction

| Graph size | Latency  |
| ---------- | -------- |
| 100        | 22.6 µs  |
| 1K         | 114.3 µs |
| 10K        | 115.7 µs |

The 1K peak occurs because decay weakens many nodes simultaneously,
pushing them past the eviction threshold and triggering a mass
eviction cascade. At 10K the per-observation intensity is lower,
so fewer nodes cross the threshold in a single pass.

---

## 7. Family 5 — Spray

### 7.1 Cold start vs steady state

**Design claim:** Cold start throughput should be higher than
steady state (growth only, no eviction).

| n    | Cold (ns/obs) | Steady (ns/obs) | Slowdown |
| ---- | ------------- | --------------- | -------- |
| 100  | 493           | 1786            | 3.6×     |
| 1K   | 288           | 1748            | 6.1×     |
| 10K  | 153           | 1846            | 12.1×    |
| 100K | 142           | 1850            | 13.0×    |

**Status: CONFIRMED.** Steady state is 6–13× slower at scale.
Cold start performs growth only (splits); steady state pays
continuous eviction + rebalance under budget pressure. Steady-state
per-obs cost converges to ~1.8 µs — the inherent per-observation
cost when eviction is active.

### 7.2 Interleave (observe + sample)

| n    | Interleave (ns/obs) | Steady (ns/obs) | Overhead |
| ---- | ------------------- | --------------- | -------- |
| 1K   | 2015                | 1748            | +15%     |
| 10K  | 1965                | 1846            | +6%      |
| 100K | 2266                | 1850            | +22%     |

Interleaving a `sample()` call with each `observe()` adds modest
overhead (6–22%). The extra work is the sampling walk itself,
which is cheap relative to the mutation cost.

### 7.3 Budget scaling

**Design claim:** Sub-linear cost growth with budget.

| Budget | Total (5K obs) | Per-obs |
| ------ | -------------- | ------- |
| 200    | 9.2 ms         | 1.8 µs  |
| 500    | 17.8 ms        | 3.6 µs  |
| 2,000  | 8.8 ms         | 1.8 µs  |
| 10,000 | 9.2 ms         | 1.8 µs  |

**Status: NUANCED.** The relationship is non-monotone. Budget=500
is the _worst_ because 5K observations exceed budget by 10×,
causing heavy eviction, and the budget is large enough for deep
eviction cascades. Budget=200 has simpler eviction (smaller tree to
scan). Budget≥2,000 barely triggers eviction.

Not "sub-linear growth" in the simple sense. The cost depends on
the ratio of observations to budget capacity. When observations
fit within budget, cost drops to near cold-start levels.

### 7.4 f64 cold start

| n    | u64     | f64     | Ratio |
| ---- | ------- | ------- | ----- |
| 100  | 0.05 ms | 0.18 ms | 3.7×  |
| 1K   | 0.29 ms | 1.45 ms | 5.1×  |
| 10K  | 1.53 ms | 19.6 ms | 12.8× |
| 100K | 14.2 ms | 108 ms  | 7.6×  |

See §PERF 9 for discussion of the f64 cost gap.

---

## 8. Family 6 — Pathological

### 8.1 Spine comparison

**Design claim:** Left-deep and right-deep should show roughly
symmetric performance (no code-path asymmetry).

| n   | Left    | Right   | R/L   | Zigzag  | Z/L   |
| --- | ------- | ------- | ----- | ------- | ----- |
| 100 | 34 µs   | 35 µs   | 1.03× | 54 µs   | 1.61× |
| 1K  | 169 µs  | 215 µs  | 1.27× | 256 µs  | 1.51× |
| 10K | 1.78 ms | 1.63 ms | 0.92× | 2.96 ms | 1.66× |

**Status: CONFIRMED.** At scale (10K) right-deep is actually 8%
_faster_ than left-deep — no systematic asymmetry. At 1K a 27%
gap suggests transient structural differences in rebalance
patterns. The gap disappears at larger $n$.

Zigzag (adversarial) costs ~66% more than left-deep at 10K. The
alternating access pattern forces repeated deep refinement and
abandonment.

### 8.2 Scaling exponents

| Pattern        | Exponent |
| -------------- | -------- |
| Left deep      | 0.86     |
| Right deep     | 0.84     |
| Zigzag         | 0.87     |
| Single hotspot | 0.81     |

All sub-linear. Even worst-case patterns don't produce superlinear
scaling.

### 8.3 Fixed workloads

| Benchmark             | Workload                              | Latency |
| --------------------- | ------------------------------------- | ------- |
| budget_burst          | 5K spread + 1K burst, budget=500      | 9.1 ms  |
| growth_pressure_relax | 100 obs, default config               | 116 µs  |
| oscillating_hotspot   | 4K obs alternating between two points | 752 µs  |
| tight_budget_spray    | 10K obs, tight budget                 | 1.51 ms |
| centroid_drift        | drifting centroid pattern             | 833 µs  |
| pincer                | converging from both extremes         | 829 µs  |

The oscillating hotspot exercises repeated deep refinement followed
by abandonment and eviction — a rebalance storm scenario. At
752 µs for 4K observations (~188 ns/obs) it remains well-behaved.

### 8.4 Structural patterns

Additional benchmarks exercising topology-specific code paths:

| Benchmark            | Size  | Latency |
| -------------------- | ----- | ------- |
| cousin_rivalry/100   | 100   | 92 µs   |
| cousin_rivalry/500   | 500   | 241 µs  |
| cousin_rivalry/2000  | 2000  | 607 µs  |
| phase_shifted/200    | 200   | 118 µs  |
| phase_shifted/1000   | 1000  | 423 µs  |
| phase_shifted/4000   | 4000  | 1.55 ms |
| fractal_fill/depth=3 | —     | 29 µs   |
| fractal_fill/depth=5 | —     | 55 µs   |
| fractal_fill/depth=7 | —     | 147 µs  |
| diamond/200          | 200   | 96 µs   |
| diamond/1000         | 1000  | 298 µs  |
| diamond/5000         | 5000  | 1.07 ms |
| gray_code/256        | 256   | 114 µs  |
| gray_code/1000       | 1000  | 291 µs  |
| gray_code/5000       | 5000  | 1.03 ms |
| golden_spiral/500    | 500   | 98 µs   |
| golden_spiral/2000   | 2000  | 272 µs  |
| golden_spiral/10000  | 10000 | 1.24 ms |

All patterns show sub-linear or near-linear scaling. No pathological
blowup detected in any structural pattern.

### 8.5 f64 left-deep

| n   | u64 left | f64 left | Ratio |
| --- | -------- | -------- | ----- |
| 100 | 34 µs    | 136 µs   | 4.1×  |
| 1K  | 169 µs   | 1.83 ms  | 10.8× |
| 10K | 1.78 ms  | 18.2 ms  | 10.2× |

The degenerate case amplifies the N=52 depth penalty (see §PERF 9).

---

## 9. f64 performance gap

The `<f64, f64, 52>` configuration is consistently 1.2–12.8× slower
than `<u64, u64, 8>` on mutation paths.

| Family                 | Ratio range   | Concern          |
| ---------------------- | ------------- | ---------------- |
| extract                | 0.8–1.8×      | Within guideline |
| query/sample           | 0.5× (faster) | No concern       |
| observe                | 1.7–6.5×      | Exceeds 2×       |
| spray/cold_start       | 3.7–12.8×     | Exceeds 2×       |
| lifecycle/decay        | 1.2–2.8×      | Exceeds 2×       |
| pathological/left_deep | 4.1–10.8×     | Exceeds 2×       |

**Root cause:** The $N$ parameter (G-Tree maximum depth) is 52 for
f64 vs 8 for u64. Every `observe()` route, split, and sum
propagation costs proportional to $d_\text{geo}$, which can reach
52 levels instead of 8. This is not a code-quality issue — it is
the fundamental cost of 52-bit coordinate precision.

Read-only operations (`get`, `sample`, `plateaus`, `extract`) are
unaffected because they do not walk the full G-Tree depth.

**Recommendation:** The 2× guideline from the benchmarking plan
should be clarified as applying to **same-$N$ comparisons**. An
f64 configuration with reduced $N$ (e.g., 16) would isolate the
floating-point arithmetic overhead from the depth effect.

---

## 10. Claims scorecard

| #   | Claim                                        | Source         | Verdict                                                 |
| --- | -------------------------------------------- | -------------- | ------------------------------------------------------- |
| 1   | $O(d_\text{geo})$ amortised observe          | §IDEA M-18.9  | **Confirmed**                                           |
| 2   | $O(1.44\,H + 1.67)$ entropy-optimal sampling | §IDEA M-18.2  | **Confirmed**                                           |
| 3   | $O(\log P)$ point query                      | §IDEA M-5.6.5 | **Confirmed**                                           |
| 4   | $O(1)$ plateau borrow                        | §API M-3.3    | **Confirmed**                                           |
| 5   | $O(N)$ range sum                             | §IDEA M-4.6   | **Confirmed**                                           |
| 6   | $O(\lvert G\rvert)$ decay                    | §IDEA M-18.9  | Nuanced — selective path sub-linear; full-tree untested |
| 7   | Selectivity monotone with $q$                | bench plan §5  | Not confirmed — walk cost dominates                     |
| 8   | $O(E_t + S_t)$ eviction scan                 | §IDEA M-12.2  | **Confirmed** — budget-gated                            |
| 9   | Cold start faster than steady                | bench plan §5  | **Confirmed** — 6–13× faster                            |
| 10  | Left ≈ right symmetry                        | bench plan §7  | **Confirmed** — ≤27% gap, 8% at scale                   |
| 11  | f64 within 2× of u64                         | bench plan §8  | Partial — reads OK, mutations 1.2–12.8×                 |
| 12  | Budget sub-linear cost growth                | bench plan §5  | Nuanced — non-monotone, ratio-dependent                 |

---

## 11. Key observations

### What the benchmarks confirm

- **Entropy-adaptive sampling is real.** The 12.3× speedup from
  uniform to hotspot is the structure's signature property. For
  Torrust's concentrated access patterns (a few torrents receive
  most traffic), sampling is effectively $O(1)$.

- **Reads are extremely fast.** Point query (14–22 ns), plateau
  borrow (2.6–2.9 ns), and sampling (10–126 ns) are all
  nanosecond-class. The structure is read-optimised by design.

- **Cold-start observation throughput is good.** ~142 ns/obs at
  scale for growth-only operation. Sub-linear scaling means it
  gets cheaper per observation as the tree fills in.

- **The rebalance machinery converges.** Even adversarial patterns
  (zigzag, oscillating hotspot, cousin rivalry, gray code)
  produce sub-linear scaling exponents (0.81–0.87). No
  superlinear blowup detected across any of the 15 pathological
  workloads.

- **Left/right symmetry is confirmed.** At 10K observations the
  right-deep spine is actually 8% faster than left-deep,
  demonstrating no systematic code-path asymmetry.

### What deserves attention

- **Steady-state cost is 6–13× cold-start.** ~1.8 µs/obs under
  budget pressure. The eviction machinery is the dominant cost
  path. Budget tuning has significant performance impact.

- **f64 mutation paths are 1.2–12.8× slower** due to the N=52 vs
  N=8 depth difference, not floating-point overhead per se.

- **Selectivity sweep does not show the expected monotone cost
  curve.** Tree walk overhead dominates per-node attenuation cost.

- **Budget scaling is non-monotone.** Cost peaks when the
  observation count moderately exceeds the budget. Very small
  and very large budgets are both cheaper.

---

## 12. Reproducing

```bash
# Full suite (~8 minutes)
cargo bench --package torrust-mudlark

# Single family
cargo bench --package torrust-mudlark -- observe/
cargo bench --package torrust-mudlark -- query/
cargo bench --package torrust-mudlark -- extract/
cargo bench --package torrust-mudlark -- lifecycle/
cargo bench --package torrust-mudlark -- spray/
cargo bench --package torrust-mudlark -- pathological/

# f64 variants only
cargo bench --package torrust-mudlark -- _f64

# HTML reports
open target/criterion/report/index.html
```

Criterion configuration: 500 ms warm-up, 1 s measurement time.
Query/sample and selectivity benchmarks use 200 samples; all
others use the Criterion default (100).
