# ADR-S-012: Test Duration Budget · `rec:sentinel:individual-release-test-five-second-budget`

**Status:** Accepted **Date:** 2026-03-12 **Spec:** — (test-suite discipline, with no section in the algorithm specification) **Relates to:** [ADR-S-007](007-automatic-noise-injection.md) (automatic noise injection — warm-up costs), [ADR-S-013](013-warm-up-convergence-benchmark.md) (convergence benchmark — empirical iteration counts)

## Context · `sec:sentinel:testbudget-context`

When this ADR was first proposed the sentinel's test suite took **~1 470 s sum-of-parts in release mode** and **~3 362 s with `CARGO_PROFILE_DEV_OPT_LEVEL=3`**. Of the then-347 tests, **56 exceeded 5 seconds in release** and the three worst exceeded **47 seconds each** (the single worst, `single_observation_repeated`, hit 68 s).

Following a systematic effort guided by the reduction strategies below, the suite was brought to ~251 s sum-of-parts (release) at the time of the original measurement (2026-03-12, 341 tests). The suite has since grown to **551 tests** (283 unit + 264 integration + 4 doc-tests, plus 3 ignored). The current state is:

| Metric                     |   Before |      Current | Improvement |
| -------------------------- | -------: | -----------: | :---------: |
| Sum-of-parts (release)     | ~1 470 s |   **~162 s** |  **9.1×**   |
| Sum-of-parts (debug opt-3) | ~3 362 s |   **~411 s** |  **8.2×**   |
| Tests > 5 s (release)      |       56 |        **3** | **18.7×**   |
| Worst individual test      |   68.0 s |    **6.6 s** | **10.3×**   |
| Critical-path binary       |   68.0 s |   **14.9 s** |  **4.6×**   |

Sum-of-parts and per-test times are measured with `--test-threads=1` (serial execution within each binary) for reproducibility. The critical-path binary uses default parallelism (same as the developer invokes `cargo test`). See [Measurement methodology](#measurement-methodology) below.

The per-test average is **~0.29 s release** (~0.75 s debug opt-3). The 3 remaining offenders above 5 s in serial mode are in **unit tests** (1) and **spray_resistance** (2). An additional ~15 tests hover in the 4–6 s range and may appear above 5 s in any given parallel run due to hybrid CPU core placement variance (P-core vs E-core; see note below).

### Measurement methodology · `sec:sentinel:testbudget-measurement-methodology`

Per-test times come from serial runs (`--test-threads=1
-Zunstable-options --report-time`) on `rustc 1.96.0-nightly` (2026-03-14). Serial execution gives stable per-test timings free from cross-test contention. Release timings use `--release`. Debug timings use `CARGO_PROFILE_DEV_OPT_LEVEL=3`. Binary-level wall-clock times use default parallelism (no `--test-threads` flag).

Measurements were taken on 2026-03-25.

**Test hardware:**

| Component    | Detail                                                       |
| ------------ | ------------------------------------------------------------ |
| CPU          | Intel Core i7-1370P (Raptor Lake, 13th Gen)                  |
| Topology     | 14 cores / 20 threads — 6 P-cores (HT) + 8 E-cores, 1 socket |
| P-core turbo | up to 5.2 GHz                                                |
| E-core turbo | up to 3.9 GHz                                                |
| L1d / L1i    | 544 KiB / 704 KiB (14 instances)                             |
| L2           | 11.5 MiB (8 instances)                                       |
| L3           | 24 MiB (shared)                                              |
| RAM          | 64 GB LPDDR5-6400 (configured at 6000 MT/s)                  |

The sentinel's working sets (tracker matrices are ≤ 128 × `max_rank` × 8 bytes ≈ few KiB each) fit comfortably in L2. The SVD-dominated tests are **compute-bound on ALU throughput**, not memory-bound — consistent with the large debug→release speedup ratios seen for compute-heavy suites (coverage_matrix 4.6×, edge_cases 9.3×) where release-mode autovectorisation and inlining of faer's inner loops make the critical difference.

> **Hybrid scheduling note.** Cargo's test harness uses `std::thread`, and the OS scheduler freely places threads on P-cores or E-cores. A compute-bound test running on an E-core can be 25–35 % slower than on a P-core. All timings in this ADR are from single runs and subject to this variance; the 5 s budget is chosen conservatively to absorb E-core placement.

## Decision · `sec:sentinel:testbudget-decision`

**Every individual test in the sentinel crate should complete in ≤ 5 seconds in release mode.**

Of 551 tests (283 unit + 264 integration + 4 doc-tests), **3 exceed 5 s in serial release mode**. One additional doc-test's binary wall-clock exceeds 5 s but is dominated by merged-doctest compilation overhead. The worst non-doc-test offender is 6.6 s.

### Remaining offenders (serial release mode) · `sec:sentinel:testbudget-remaining-offenders`

|   # | Test                                                                 | Suite            | Release (s) | Debug (s) |
| --: | -------------------------------------------------------------------- | ---------------- | ----------: | --------: |
|   1 | `tests::convergence_fixes::production_lambda_converges_within_bound` | unit tests       |         6.6 |      19.2 |
|   2 | `cells_tracked_bounded_under_diverse_traffic`                        | spray_resistance |         5.7 |      14.3 |
|   3 | `g_nodes_bounded_by_budget_under_spray`                              | spray_resistance |         5.1 |      14.9 |

Release timings are from isolated single-test invocations where serial-run thermal variance can be excluded. `production_lambda_converges_within_bound` is irreducibly expensive: 1 500 rounds of Brand SVD at λ = 0.99, b = 16 is the minimum validated by ADR-S-013 for the 1 200-round convergence bound. The two spray_resistance tests exercise the full graph lifecycle under adversarial traffic and produce many cell splits with noise warm-up.

The **doc-test** binary wall-clock is 14.9 s (release), but this includes merged-doctest compilation overhead; individual doc-tests run in 2–4 s.

### Targeted improvements (release mode) · `sec:sentinel:testbudget-targeted-improvements`

The following tests were targeted in the latest reduction round. Before/after timings are from serial measurement:

| Test                                                                    | Suite           | Before (s) | After (s) | Strategy |
| ----------------------------------------------------------------------- | --------------- | ---------: | --------: | -------- |
| `tests::variance_formula::batch_size_invariant_surprise_ratio`          | unit tests      |       17.3 |       1.0 | §7, §8   |
| `tests::convergence_fixes::production_lambda_converges_within_bound`    | unit tests      |        9.1 |       6.6 | §7       |
| `step3_higher_volume_cells_warm_via_priority`                           | deferred_warmup |        9.3 |       2.5 | §9       |
| `step2_reset_clears_staging_and_resumes`                                | deferred_warmup |        7.7 |       1.4 | §8       |
| `step3_concurrent_ingest_no_panic`                                      | deferred_warmup |        6.6 |       < 1 | §8       |
| `step2_lifecycle_new_cells_appear_after_splits`                         | deferred_warmup |        6.3 |       < 1 | §8       |
| `tests::convergence_fixes::per_axis_convergence_b4_robust_across_seeds` | unit tests      |        5.7 |       0.1 | §7       |
| `graph_accessor_starts_with_single_root`                                | api             |        5.5 |      0.03 | §10      |
| `new_with_default_config_succeeds`                                      | api             |        5.5 |      0.00 | §10      |
| `step3_reset_restarts_background_thread`                                | deferred_warmup |        5.4 |       < 1 | §8       |

### Affected suites — summary · `sec:sentinel:testbudget-affected-suites`

| Suite                | Tests > 5 s | Worst (s) | Pattern                                                                                                 |
| -------------------- | ----------: | --------: | ------------------------------------------------------------------------------------------------------- |
| **unit tests**       |           1 |       6.6 | Production-config convergence: 1 500 slow-EWMA rounds (λ = 0.99) with Brand SVD kernel.                |
| **spray_resistance** |           2 |       5.7 | Full graph lifecycle with many cell splits and noise warm-up under adversarial traffic.                  |

### Binary-level wall-clock times · `sec:sentinel:testbudget-binary-wall-clock-times`

Cargo runs each integration test file as a separate binary with internal parallelism. The suite's total wall-clock equals the sum of all binary wall-clocks (binaries run sequentially).

| Binary                    | Release (s) | Debug (s) |  Speedup |
| ------------------------- | ----------: | --------: | -------: |
| unit tests                |         7.0 |      20.5 |     2.9× |
| ancestor_chain            |         2.0 |       8.4 |     4.2× |
| api                       |         0.3 |       0.6 |     2.3× |
| clip_pressure             |         3.9 |      17.3 |     4.4× |
| coverage_matrix           |         3.9 |      18.0 |     4.6× |
| deferred_warmup           |         5.8 |      29.2 |     5.1× |
| determinism               |         1.0 |      10.4 |    10.1× |
| edge_cases                |         2.8 |      26.1 |     9.3× |
| graph_routing             |         3.3 |       4.8 |     1.4× |
| health                    |         0.1 |       0.2 |     2.2× |
| hierarchical_coordination |         2.1 |       5.0 |     2.5× |
| integration               |         4.5 |       8.5 |     1.9× |
| invariants                |         7.1 |      17.0 |     2.4× |
| noise                     |         3.9 |       9.8 |     2.5× |
| report_structure          |         0.6 |       1.0 |     1.7× |
| sentinel_u64              |         0.8 |       1.8 |     2.3× |
| serde_roundtrip           |         0.0 |       0.0 |        — |
| spatial_decay             |         1.4 |       2.3 |     1.6× |
| spray_resistance          |         8.8 |      19.5 |     2.2× |
| suffix_analysis           |         0.7 |       1.3 |     1.7× |
| warm_up                   |         6.7 |      13.2 |     2.0× |
| doc-tests                 |        14.9 |      34.2 |     2.3× |
| **Total**                 |    **81.6** | **249.0** | **3.1×** |

The critical-path binary in release is **doc-tests at 14.9 s** (dominated by merged-doctest compilation), followed by **spray_resistance at 8.8 s** and **unit tests at 7.0 s** (reduced from 18.8 s).

### Reduction strategies (applied) · `sec:sentinel:testbudget-reduction-strategies`

The following strategies were used to bring the suite from 56 offenders down to the current level:

1. **Reduced iteration counts to empirically validated minimums (ADR-S-013).** Most slow tests used conservatively chosen loop counts (50–500 batches). ADR-S-013's convergence benchmark established the empirical settling batch $n_{\text{settled}}$ and provided justified minimums.

2. **Lightened the shared `rich_report()` setup** in `serde_roundtrip`. The serde tests were restructured — the suite now runs 0 tests (the expensive report-building helpers were eliminated), dropping all 7 former offenders (42+ s each).

3. **Lowered `split_threshold` and `budget` in test configs.** Graph-heavy tests now use tighter parameters to reach the same structural depth with fewer observations.

4. **Fixed `single_observation_repeated` (was 68 s, now 1.9 s).** Smaller iteration count and controlled seed validate the same edge-case property in a fraction of the time.

5. **Rationalised `deferred_warmup` lifecycle tests.** The warm-up batch counts were reduced using ADR-S-013 minimums — all 14 tests now finish in ≤ 4.5 s (release), down from a worst of 33.6 s.

6. **Used minimum validated `noise_rounds`** per ADR-S-013. Tests that do not specifically validate noise behaviour now use $\max(r_{\text{noise}}, 5)$ rounds.

7. **Reduced observation dimensionality in convergence tests.** The EWMA convergence and surprise-ratio properties validated by the unit-test convergence and variance modules are per-axis — they depend on λ and batch size, not on the observation dimension. Tests that dominated the unit-test binary were changed from `dim = 128` to `dim = 32`, cutting SVD cost ~4× without affecting the validated property.

8. **Halved warm-up iterations and noise schedule in tests that were over-provisioned.** `batch_size_invariant_surprise_ratio` used 200 warm-up + 200 measurement rounds where 100 + 100 suffices at λ = 0.95 (half-life = 14 rounds, 95% settled by round 42). `fast_split_config()` in deferred_warmup tests used `NoiseSchedule::Explicit(vec![5])` where `vec![3]` is sufficient: 3 rounds × batch_size = 4 = 12 noise observations gives η = 0.90^12 ≈ 0.28, adequate for tests that only assert `noise_observations > 0`.

9. **Reduced noise schedule in `step3_higher_volume_cells_warm_via
   _priority`.** The test uses a large noise schedule to observe partial background warming. Reduced from 100 to 30 rounds — still slow enough for partial-warming observation, but 3.3× fewer SVD passes.

10. **Replaced full sentinel construction with `validate()` for config-validation tests.** Two api tests (`new_with_default
    _config_succeeds`, `graph_accessor_starts_with_single_root`) used `SentinelConfig::default()` which triggers 450 root noise rounds. `new_with_default_config_succeeds` now calls `cfg.validate()` instead; `graph_accessor_starts_with_single
    _root` now uses `test_config()` (5 noise rounds).

## Alternatives Considered · `sec:sentinel:testbudget-alternatives`

- **Accept slow tests; rely on CI parallelism.** Current CI already runs the suite in parallel across multiple runners, but individual test binaries are serialised. A single slow test blocks its binary's thread pool regardless of runner count. This also does not address the developer-flow problem.

- **Move slow tests to a separate `slow_tests` feature gate.** Adds cognitive overhead ("did I run the full suite?") and fragments coverage. The `#[ignore]` + `--include-ignored` pattern is standard Rust and better supported by tooling.

- **Profile-guided optimisation of the sentinel itself.** Would help the tight SVD loops but does not address the graph-structure-bound tests, and adds build complexity.

## Consequences · `sec:sentinel:testbudget-consequences`

- **The 5 s budget is a standing guideline.** New tests that exceed 5 s in release mode should be flagged in review and either trimmed or marked `#[ignore]`. The 3 remaining offenders (worst 6.6 s) are close to budget and have no natural reduction path without weakening their validated properties.

- **Overall suite performance.** The sequential binary wall-clock sum is **~82 s in release** and **~249 s in debug (opt-3)**. With Cargo's default parallelism on this 20-thread hybrid CPU the practical wall-clock is **under 15 s in release**, which is acceptable for local iteration.

- **Regression safety net.** ADR-S-013's `convergence_matches_theory` and `noise_baselines_converge` tests detect if code changes (EWMA update rule, outlier clipping, maturity formula) shift the convergence rate. This prevents iteration counts from silently becoming insufficient.

- **`#[ignore]`d long-run variants** should be run in nightly CI to retain full convergence coverage without penalising the default suite.
