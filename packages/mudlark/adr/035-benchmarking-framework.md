# ADR-M-035: Benchmarking Framework

**Status:** Decided  
**Date:** 2026-03-04  
**Relates to:** [ADR-M-018](018-hard-budget-guarantee.md) (hard budget
guarantee), [ADR-M-019](019-sampling-semantics.md) (sampling semantics),
[ADR-M-032](032-three-surface-model.md) (three-surface model),
[ADR-M-033](033-v-generic-importance-properties.md) (V generic bounds)  
**Spec:** §IDEA M-8 (observation pipeline), §§IDEA M-9–11 (V-Tree
operations), §IDEA M-12 (eviction), §IDEA M-14 (temporal decay), §IDEA M-18 (Fibonacci
depth bound)  
**API:** [§API M-5.2](../docs/api.md#52-gvgraphc-v-n--the-negative)
(all benchmarked operations)  
**Surface:** 1 + 2 (Prints + Film)

---

## Context

The formal specification (idea.md) and api.md document asymptotic
costs for every public operation:

| Operation     | Claimed cost                     | Source                 |
| ------------- | -------------------------------- | ---------------------- |
| `get()`       | $O(\text{depth})$                | §API M-5.2             |
| `range_sum()` | $O(N)$                           | §API M-5.2             |
| `sample()`    | $O(1.44\,H + 1.67)$              | §API M-5.2, §IDEA M-18 |
| `extract()`   | $O(L + S)$                       | §API M-5.2             |
| `plateaus()`  | $O(1)$ borrowed / $O(G)$ rebuilt | §API M-5.2             |
| `observe()`   | $O(\text{depth})$ amortised      | §IDEA M-8              |
| `decay()`     | $O(G_{\text{subtree}})$          | §IDEA M-14             |

These bounds are proven formally but have never been validated
empirically. Without benchmarks we cannot:

1. **Detect regressions.** A refactor that preserves correctness
   (all tests pass) may silently degrade throughput.
2. **Validate theoretical claims.** The $O(1.44\,H)$ sampling bound
   and the Fibonacci-rate decay bound are central selling points —
   they should be demonstrated, not just asserted.
3. **Characterise pathological workloads.** The specification
   addresses worst-case structure (left-deep spines, adversarial
   zigzag, budget pressure) but we have no empirical data on how
   these affect wall-clock performance.
4. **Measure realistic sustained throughput.** Torrust's actual
   access pattern is random peers hitting random torrents — a
   stationary ergodic process that no existing deterministic test
   plan models.

### Existing infrastructure

The `testing` module ([ADR-M-032](032-three-surface-model.md))
already provides a complete, Surface-1/2-only workload generation
framework:

| Component       | What it provides                                                                                                                  |
| --------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `Plan<C, V>`    | Serializable, replayable observation sequences                                                                                    |
| `Plan` builders | `.spread()`, `.sweep()`, `.skewed()`, `.hotspot()`, `.zigzag()`, `.adversarial_zigzag()`, `.burst()`                              |
| Preset plans    | `plan_left_deep`, `plan_right_deep`, `plan_single_hotspot`, `plan_adversarial`, `plan_budget_burst`, `plan_growth_pressure_relax` |
| Config presets  | `default_config`, `budget_config`, `tight_budget_config`, `deep_config`, `low_threshold_config`, `cascade_config`, etc.           |
| `run()`         | Apply plan to fresh graph (no invariant checks — fast)                                                                            |
| `GraphCreator`  | Fluent builder: config + plan + build in one chain                                                                                |

All of these are ungated (available in release builds) and use only
Surface 1 + 2 types. The only gated component is the RNG stubs
(`TestLcgRng`, `FixedRng`, `SeqRng`), which are
`#[cfg(debug_assertions)]` — unavailable in release-mode benchmarks.

---

## Decision

### D1. Tooling — Criterion.rs

All benchmarks use [Criterion.rs](https://github.com/bheisler/criterion.rs).
Rationale:

- Stable, widely adopted in the Rust ecosystem.
- Statistical rigour: confidence intervals, outlier detection,
  regression comparison.
- `bench_with_input` + `Throughput::Elements` for parameterised
  scaling sweeps.
- `iter_batched` for separating expensive setup from measured
  hot-paths.
- `BenchmarkGroup` + `BenchmarkId` for structured cross-workload
  comparison.
- Auto-generated HTML reports with violin plots, PDF overlays, and
  slope analysis.
- Baseline save/compare (`--save-baseline` / `--baseline`) for CI
  regression detection.

Criterion is added as a `[dev-dependency]` of `torrust-mudlark`.

### D2. Benchmark location — `benches/`

Benchmarks live in `packages/mudlark/benches/`. Each benchmark
family is a separate file registered in `Cargo.toml` via
`[[bench]]` entries with `harness = false`.

```
packages/mudlark/
├── benches/
│   ├── bench_rng.rs        # BenchRng (release-mode LCG)
│   ├── observe.rs          # Family 1: observation throughput
│   ├── query.rs            # Family 2: query latency
│   ├── extract.rs          # Family 3: extraction cost
│   ├── lifecycle.rs        # Family 4: decay + eviction
│   ├── spray.rs            # Family 5: sustained random workload
│   └── pathological.rs     # Family 6: adversarial worst cases
```

### D3. RNG — duplicate, do not re-gate

The `testing` module's RNG stubs are `#[cfg(debug_assertions)]`.
Benchmarks run in release mode. Rather than weakening the gate
(which exists for good reason — test-only code should not appear
in production builds), benchmarks define their own minimal LCG in
`benches/bench_rng.rs`:

```rust
use torrust_mudlark::Rng;

pub struct BenchRng(pub u64);

impl Rng for BenchRng {
    fn next_f64(&mut self) -> f64 {
        self.0 = self.0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}
```

Same constants as `TestLcgRng`. ~10 lines, zero coupling to the
debug gate. Everything else from `testing` (plans, configs, runner,
builder) is ungated and imported directly.

### D4. New `Plan` builder — `random_spray`

A new method is added to `Plan<C, V>` for generating
deterministic pseudo-random observation sequences:

```rust
impl<C: Coordinate, V: Accumulator + Inspectable> Plan<C, V> {
    /// Uniform random spray: `n` observations at pseudo-random
    /// coordinates within `[0, domain)`, all with the same `delta`.
    ///
    /// Deterministic: same `seed` produces identical plans.
    /// Uses an inline LCG — no external RNG dependency.
    pub fn random_spray(
        mut self, seed: u64, domain: u64, delta: V, n: usize,
    ) -> Self {
        let mut state = seed;
        for _ in 0..n {
            state = state
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            let coord = C::from_u64(state % domain);
            self.observations.push((coord, delta));
        }
        self
    }
}
```

Serializable, replayable, seed-controlled. Fits Plan's existing
contract.

### D5. New `Plan` builder — `oscillating_hotspot`

A pathological plan where the hotspot location alternates every
`burst` observations between two distant coordinates:

```rust
impl<C: Coordinate, V: Accumulator + Inspectable> Plan<C, V> {
    /// Oscillating hotspot between two coordinates. Every `burst`
    /// observations the target switches. Forces repeated deep
    /// refinement followed by abandonment and eviction.
    pub fn oscillating_hotspot(
        mut self, a: C, b: C, delta: V, burst: usize, cycles: usize,
    ) -> Self {
        for cycle in 0..cycles {
            let target = if cycle % 2 == 0 { a } else { b };
            self.observations
                .extend(std::iter::repeat((target, delta)).take(burst));
        }
        self
    }
}
```

### D6. Six benchmark families

#### Family 1 — Observe (observation throughput)

Measures the 10-step mutation pipeline: route → accumulate →
propagate → violation check → split → rebalance → depth control →
eviction.

| Benchmark                     | Plan                 | Config               | Sweep parameter                               |
| ----------------------------- | -------------------- | -------------------- | --------------------------------------------- |
| `observe/spread/{n}`          | `.spread(256, 6, n)` | `default_config()`   | $n \in \{100, 1\rm{K}, 10\rm{K}, 100\rm{K}\}$ |
| `observe/spread_budgeted/{n}` | `.spread(256, 6, n)` | `budget_config(500)` | same                                          |
| `observe/skewed/{n}`          | `.skewed(256, n)`    | `default_config()`   | same                                          |
| `observe/deep/{n}`            | `.sweep(256, n)`     | `deep_config()`      | same                                          |

**Criterion features used:**

- `bench_with_input` + `Throughput::Elements(n)` → reports obs/sec.
- Scaling sweep validates amortised $O(\text{depth})$ claim: obs/sec
  should remain roughly constant as $n$ grows (tree depth grows
  logarithmically).

#### Family 2 — Query (point query, range sum, sample)

Uses `run()` as a **setup phase** (not timed) to build a graph to
a known state, then measures the query operation itself.

| Benchmark                 | Setup plan              | Measured operation                  | What it validates             |
| ------------------------- | ----------------------- | ----------------------------------- | ----------------------------- |
| `query/get/{size}`        | `.spread(256, 6, size)` | `get(coord)`                        | $O(\text{depth})$ point query |
| `query/range_sum/{width}` | `.spread(256, 6, 10K)`  | `range_sum(a..b)` at varying widths | $O(N)$ range query            |
| `query/sample/uniform`    | `.spread(256, 6, 5K)`   | `sample(&mut rng)`                  | Baseline sample latency       |
| `query/sample/skewed`     | `.skewed(256, 5K)`      | `sample(&mut rng)`                  | Lower entropy → faster        |
| `query/sample/hotspot`    | `.hotspot(128, 10, 5K)` | `sample(&mut rng)`                  | Minimal entropy → fastest     |

**Criterion features used:**

- `BenchmarkGroup` comparing uniform / skewed / hotspot sampling
  → violin plots directly visualise the entropy-optimal
  $O(1.44\,H)$ claim (hotspot = low $H$ = fast; uniform = high
  $H$ = slower).
- `iter_batched` where fresh graphs are needed per iteration.
- Per-group `measurement_time` / `sample_size` tuning (queries
  are fast — need more samples for stable statistics).

#### Family 3 — Extract (PEWEI extraction, layers, plateaus)

| Benchmark                 | Setup plan              | Measured operation | What it validates                               |
| ------------------------- | ----------------------- | ------------------ | ----------------------------------------------- |
| `extract/full/{size}`     | `.spread(256, 6, size)` | `extract()`        | $O(L + S)$ allocating extraction                |
| `extract/layers/{size}`   | `.spread(256, 6, size)` | `layers().count()` | Streaming extraction                            |
| `extract/plateaus/{size}` | `.spread(256, 6, size)` | `plateaus()`       | $O(1)$ borrow (with `dynamic-contour-tracking`) |

**Criterion features used:**

- `bench_with_input` size sweep $\{100, 1\rm{K}, 10\rm{K}\}$.
- `extract` vs `layers`: same group, showing allocation overhead.

#### Family 4 — Lifecycle (decay + eviction)

Each iteration mutates the graph, so `iter_batched` is mandatory.

| Benchmark                       | Setup plan              | Measured operation              | What it validates                                        |
| ------------------------------- | ----------------------- | ------------------------------- | -------------------------------------------------------- |
| `lifecycle/decay/{size}`        | `.spread(256, 6, size)` | `decay(g_root, 0.95, 0.5)`      | $O(G)$ decay                                             |
| `lifecycle/eviction/{size}`     | `.spread(256, 6, size)` | `check_evictions()`             | Eviction scan cost                                       |
| `lifecycle/decay_evict/{size}`  | `.spread(256, 6, size)` | `decay()` + `check_evictions()` | Combined cycle                                           |
| `lifecycle/decay_selective/{q}` | `.spread(256, 6, 5K)`   | `decay(g_root, 0.95, q)`        | Selectivity sweep: $q \in \{0.0, 0.25, 0.5, 0.75, 1.0\}$ |

**Criterion features used:**

- `iter_batched(setup, routine, BatchSize::SmallInput)` — fresh
  graph per iteration since decay/eviction mutate.
- Selectivity sweep uses `BenchmarkId::new("decay", q)`.

#### Family 5 — Spray (sustained random workload)

The most realistic workload — models random peers hitting random
torrents. Uses the new `random_spray` builder.

| Benchmark                  | Description                                                              | What it validates                                  |
| -------------------------- | ------------------------------------------------------------------------ | -------------------------------------------------- |
| `spray/cold_start/{n}`     | Fresh graph, `n` random observations, measure total throughput           | Growth-phase: splits dominate, no eviction         |
| `spray/steady_state/{n}`   | Pre-load to budget (10K warmup), then `n` more random observations       | Steady-state ops/sec at capacity — eviction active |
| `spray/interleave/{n}`     | Alternating `observe()` + `sample()` at steady state                     | Query latency under mutation pressure              |
| `spray/scaling/budget/{b}` | Fixed spray (5K), varying budget $b \in \{200, 500, 2\rm{K}, 10\rm{K}\}$ | How tree size affects per-observation cost         |

**Criterion features used:**

- `Throughput::Elements` for obs/sec in all spray benchmarks.
- `bench_with_input` for budget sweep.
- `iter_batched` for steady-state (warmup in setup closure).

#### Family 6 — Pathological (adversarial worst cases)

Exercises degenerate tree shapes, maximum eviction pressure, and
rebalance storms. Uses existing preset plans and new builders.

| Benchmark                             | Plan                                        | Config                  | What's pathological                                                                                               |
| ------------------------------------- | ------------------------------------------- | ----------------------- | ----------------------------------------------------------------------------------------------------------------- |
| `pathological/left_deep/{n}`          | `plan_left_deep(6, n)`                      | `default_config()`      | Maximum G-Tree depth on one spine — split cascades, degenerate V-Tree chain                                       |
| `pathological/right_deep/{n}`         | `plan_right_deep(256, 6, n)`                | `default_config()`      | Mirror: exercises right-child code paths                                                                          |
| `pathological/adversarial_zigzag/{n}` | `plan_adversarial(256, n)`                  | `default_config()`      | Escalating intensity at domain extremes — contraction preprocessing, max-uncle violations every other observation |
| `pathological/budget_burst`           | `plan_budget_burst(256, 5K, 1K)`            | `budget_config(500)`    | Gentle growth then sudden spike — eviction under maximum pressure                                                 |
| `pathological/growth_pressure_relax`  | `plan_growth_pressure_relax(256)`           | `default_config()`      | Three-phase cycle: grow → pressure → recovery. Depth-gate oscillation                                             |
| `pathological/single_hotspot/{n}`     | `plan_single_hotspot(128, 6, n)`            | `default_config()`      | All energy at one point — depth ceiling, semi-internal accumulation                                               |
| `pathological/oscillating_hotspot`    | `.oscillating_hotspot(0, 255, 10, 200, 20)` | `budget_config(500)`    | Hotspot jumps — forces repeated deep refinement then abandonment. Maximum eviction churn                          |
| `pathological/tight_budget_spray`     | `.random_spray(42, 256, 6, 10K)`            | `tight_budget_config()` | Minimum headroom (budget=82), random arrivals — every observation fights for scarce slots                         |

**Criterion features used:**

- `bench_with_input` for size sweeps on spine benchmarks.
- Same-group comparison: left-deep vs right-deep vs adversarial
  zigzag → violin plots reveal asymmetries.
- **p99/p50 analysis**: pathological plans may cause occasional
  rebalance storms visible in Criterion's PDF plots. This is the
  primary diagnostic — not just mean throughput.

### D7. Canonical type configuration

All benchmarks use `<u64, u64, 8>` (domain `[0, 256)`) as the
primary configuration. This matches the existing `testing` presets,
keeps node counts manageable, and provides consistent cross-family
comparison.

A secondary `<f64, f64, 52>` configuration is added for one
representative benchmark per family (e.g., `observe/spread_f64`)
to verify that floating-point coordinate/accumulator paths have no
unexpected performance cliffs.

### D8. CI integration

```bash
# Save baseline on main:
cargo bench --package torrust-mudlark -- --save-baseline main

# Compare on feature branch:
cargo bench --package torrust-mudlark -- --baseline main
```

Criterion outputs comparison with confidence intervals. CI
archives the `target/criterion/` directory as a build artifact.

Regression policy (enforced by reviewer, not automated gate in
first iteration):

- **> 5% regression** on any named benchmark: must be acknowledged
  in PR description with justification.
- **> 15% regression**: blocks merge unless explicitly approved.

Automated gating (parsing Criterion JSON output for threshold
violations) is deferred to a follow-up.

### D9. Mirror tests — invariant-checked benchmark workloads

Criterion's built-in test mode (`cargo test` compiling benchmark
binaries) only verifies that each benchmark compiles and survives
one iteration without panicking. It does **not** run invariant
checks, does not benefit from `debug_assertions`, and provides no
violation diagnostics.

Every benchmark workload therefore has a **mirror integration test**
in `tests/` that replays the same `Plan` + `Config` pair in debug
mode with invariant checking enabled.

#### Design

| Aspect                  | Benchmark (`benches/`)                | Mirror test (`tests/`)                                                              |
| ----------------------- | ------------------------------------- | ----------------------------------------------------------------------------------- |
| **Build mode**          | `--release`, no `debug_assertions`    | Debug, `debug_assertions` enabled                                                   |
| **Runner**              | `run()` (no checks — fast)            | `run_checked()` or `run_soft_checked()`                                             |
| **Scale**               | Full (10K–100K observations)          | Reduced (~1/10th) — enough to trigger the same code paths without being slow        |
| **RNG**                 | `BenchRng` (duplicated in `benches/`) | `TestLcgRng` (from `testing`, `#[cfg(debug_assertions)]` — available in debug mode) |
| **Goal**                | Wall-clock performance measurement    | Invariant violation detection under identical workload shapes                       |
| **Invariant frequency** | None                                  | Every 10–100 observations via `run_checked(config, &plan, k)`                       |

#### Naming convention

Mirror tests live in `tests/bench_mirrors/`, mirroring the
`benches/` module structure one-to-one:

```
packages/mudlark/
├── benches/
│   ├── bench_rng.rs
│   ├── observe.rs
│   ├── query.rs
│   ├── extract.rs
│   ├── lifecycle.rs
│   ├── spray.rs
│   └── pathological.rs
└── tests/
    └── bench_mirrors/
        ├── mod.rs              # #[cfg(test)] module root
        ├── observe.rs
        ├── query.rs
        ├── extract.rs
        ├── lifecycle.rs
        ├── spray.rs
        └── pathological.rs
```

Each mirror module contains tests named `mirror_{benchmark}` that
replay the corresponding benchmark's `Plan` + `Config` pair. For
example:

```rust
use torrust_mudlark::testing::*;

/// Mirror of bench: spray/steady_state
#[test]
fn mirror_spray_steady_state() {
    let warmup = Plan::new().random_spray(42, 256, 6, 1_000);
    let spray  = Plan::new().random_spray(99, 256, 6, 200);

    // Build to steady state WITH invariant checking every 50 obs.
    let mut g = run_checked::<u64, u64, 8>(
        budget_config(500), &warmup, 50,
    );

    // Continue under invariant checking.
    for &(coord, delta) in &spray.observations {
        g.observe(coord, delta);
    }
    assert_invariants(&g);
}

/// Mirror of bench: pathological/adversarial_zigzag
#[test]
fn mirror_pathological_adversarial_zigzag() {
    let plan = plan_adversarial(256, 500);
    run_checked::<u64, u64, 8>(default_config(), &plan, 20);
}

/// Mirror of bench: pathological/oscillating_hotspot
#[test]
fn mirror_pathological_oscillating_hotspot() {
    let plan = Plan::new()
        .oscillating_hotspot(0_u64, 255, 10, 50, 6);
    run_checked::<u64, u64, 8>(budget_config(500), &plan, 10);
}

/// Mirror of bench: lifecycle/decay_evict
#[test]
fn mirror_lifecycle_decay_evict() {
    let plan = Plan::new().spread(256, 6_u64, 500);
    let mut g = run_checked::<u64, u64, 8>(
        default_config(), &plan, 50,
    );
    g.decay(g.g_root(), 0.95, 0.5);
    g.check_evictions();
    assert_invariants(&g);
}
```

#### Coverage rule

Every benchmark listed in D6 must have a corresponding mirror test.
The mirror may combine related benchmarks (e.g., a single test
covers `lifecycle/decay`, `lifecycle/eviction`, and
`lifecycle/decay_evict` sequentially). The requirement is that
every **plan shape × config** pair is exercised under invariant
checking, not that the mirror count equals the benchmark count.

#### When mirrors run

Mirror tests are standard `#[test]` functions — they run in
`cargo test --workspace --all-targets --all-features`. They are
always in debug mode, so `debug_assertions` and the `testing`
RNG stubs are available. No special CI configuration is needed.

### D10. What is explicitly deferred

| Topic                                                              | Reason                                                                                                               |
| ------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------- |
| Flamegraph / profiling integration                                 | Complementary but separate concern — ad-hoc, not CI-gated                                                            |
| Surface 3 micro-benchmarks (arena, rebalance, V-Tree in isolation) | Useful for optimisation but not for regression tracking. Requires `pub(crate)` access — violates Surface boundary    |
| Memory benchmarking (node-count to RSS)                            | Needs different tooling (e.g., `dhat`, custom allocator). Worth a follow-up ADR                                      |
| Automated CI gate (fail pipeline on regression)                    | Requires Criterion JSON parser in CI script. Deferred until baseline data is collected                               |
| Feature-flag matrix (`--no-default-features`)                      | First iteration benchmarks only the default feature set. `dynamic-contour-tracking` off/on comparison is a follow-up |

---

## Consequences

- **`Cargo.toml`** gains `criterion` as a dev-dependency and
  `[[bench]]` entries for each family.
- **`Plan`** gains two new builders: `random_spray` and
  `oscillating_hotspot`.
- **`benches/`** directory is created with six benchmark files and
  a shared `bench_rng.rs`.
- The `testing` module's `#[cfg(debug_assertions)]` gate on RNG
  stubs is **not modified** — benchmark RNG is self-contained.
- **`tests/bench_mirrors/`** is created with one module per
  benchmark family, mirroring the `benches/` structure.
  Invariant-checked mirror tests for every benchmark workload
  run in debug mode as part of the standard `cargo test` suite.
- Developers can run individual families:
  `cargo bench --package torrust-mudlark -- spray/` for fast
  iteration.
- HTML reports at `target/criterion/report/index.html` provide
  immediate visual feedback without external tooling.
- Theoretical cost claims in api.md are now empirically verifiable
  and regression-protected.
