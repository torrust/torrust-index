# Testing

This document describes the scope, style, and conventions of the
`torrust-mudlark` test suite, and provides an index of every test
file and benchmark.

---

## §1 Scope

### 1.1 What is tested

The test suite validates:

- **Structural invariants** — every invariant defined in idea.md is
  checked as a post-condition after every mutation in every test
  via `assert_invariants()` (or `check_all_invariants()` for
  diagnostic tests that need the error list).

- **Correctness of the public API** — all Surface 1 (Prints) and
  Surface 2 (Film) operations specified in
  [api.md](api.md) and formalised by the three-surface model
  ([ADR-M-032](../adr/032-three-surface-model.md)).

- **Internal machinery** — arena operations, decay curves, tree
  balancing, split logic, V-tree structure, eviction ordering, and
  the observation pipeline, tested via unit tests in `src/tests/`.

- **Feature-gated behaviour** — `--all-features` vs
  `--no-default-features`, and `debug_assertions` vs release-mode
  paths.

- **Regression coverage** — specific bugs captured by ADRs are
  guarded with dedicated regression tests (e.g. the factor-table
  depth bound bug from [ADR-M-038](../adr/038-decay-factor-table-depth-bound.md)).

- **Mutation-testing gaps** — boundary-value tests, invariant-
  checker self-tests, and output-correctness assertions added in
  response to the `cargo-mutants` gap analysis
  ([ADR-M-039](../adr/039-mutation-testing-gap-analysis.md)).

- **Benchmark correctness mirrors** — every Criterion benchmark
  workload is replayed with full invariant checking in debug mode
  ([ADR-M-035](../adr/035-benchmarking-framework.md)).

- **Doc-tests** — every public item carries a compiled-and-executed
  example per the coverage rule in
  [ADR-M-034](../adr/034-doc-comment-and-doctest-policy.md).

### 1.2 What is not tested here

- **Integration with other workspace packages** (e.g.
  `torrust-sentinel`) — those packages carry their own test suites.
- **Performance regression detection** — benchmarks measure wall-clock
  throughput but CI does not gate on timing thresholds. See
  [performance.md](performance.md) for analysis.

---

## §2 Test Levels

Following the workspace-wide convention in `AGENTS.md`:

| Level       | Visibility   | Location            | Purpose                                                  |
| ----------- | ------------ | ------------------- | -------------------------------------------------------- |
| Unit        | `private`    | `src/tests/*.rs`    | Internal invariants, per-module logic                    |
| Crate       | `pub(crate)` | `src/tests/*.rs`    | Cross-module interactions using crate internals          |
| Integration | `pub`        | `tests/*.rs`        | Public API exercised through Surface 1 + 2 only          |
| Doc-test    | `pub`        | inline `///` blocks | API examples compiled and executed by `cargo test --doc` |

Unit and crate tests live side-by-side in `src/tests/` and have
access to `pub(crate)` internals. Integration tests in `tests/`
import only the public API — they must not reach through to
Surface 3 (Emulsion) internals.

---

## §3 Style and Conventions

### 3.1 Invariant checking

Every test that mutates a `GvGraph` calls `assert_invariants(&g)`
at least once (typically at the end). Stress tests use
`run_checked()` with `check_every = 1` to validate invariants
after **every** observation.

Diagnostic tests use `run_diagnostic()` /
`run_diagnostic_budgeted()` to check every invariant after every
observation and dump full G-tree + plateau state on the first
violation.

### 3.2 Test harness — the `testing` module

The `src/testing/` module provides a shared, Surface 1+2-only
infrastructure used by both unit and integration tests:

| Component      | What it provides                                   |
| -------------- | -------------------------------------------------- |
| `Plan<C, V>`   | Serialisable, replayable observation sequences     |
| `GraphCreator` | Fluent builder: config + plan + build in one chain |

#### Plan builders (`Plan` methods)

| Builder                        | Pattern generated                          |
| ------------------------------ | ------------------------------------------ |
| `.observe()`                   | Single observation                         |
| `.observe_n()`                 | Repeated single-coord observation          |
| `.hotspot()`                   | Alias for `.observe_n()`                   |
| `.sweep()`                     | Sequential walk across `[0, domain)`       |
| `.spread()`                    | Uniform spread with constant delta         |
| `.skewed()`                    | Power-of-two weighted distribution         |
| `.zigzag()`                    | Alternating lo/hi coordinates              |
| `.adversarial_zigzag()`        | Zigzag with increasing delta               |
| `.burst()`                     | Spread followed by hotspot burst           |
| `.random_spray()`              | LCG-seeded random coordinates              |
| `.oscillating_hotspot()`       | Alternating bursts between two coordinates |
| `.interleaved_hotspots()`      | Multiple hotspots cycled in round-robin    |
| `.cousin_rivalry()`            | Two adjacent subtrees competing for weight |
| `.adversarial_cousins()`       | Cousin rivalry with escalating deltas      |
| `.phase_shifted_oscillation()` | Two coords with out-of-phase high/low      |
| `.neighborhood_burst()`        | Burst around a centre within a radius      |
| `.fractal_spray()`             | Recursive dyadic subdivision               |
| `.nested_bursts()`             | Multi-level nested burst hierarchy         |
| `.then()`                      | Concatenate two plans                      |

#### Preset plans (`presets.rs`)

| Preset                        | Scenario                                 |
| ----------------------------- | ---------------------------------------- |
| `plan_left_deep`              | All observations at coordinate 0         |
| `plan_right_deep`             | All at maximum coordinate                |
| `plan_single_hotspot`         | All at one chosen coordinate             |
| `plan_adversarial`            | Adversarial zigzag across domain         |
| `plan_budget_burst`           | Spread then hotspot (budget pressure)    |
| `plan_growth_pressure_relax`  | Burst → spread → burst cycle             |
| `plan_range_tree`             | Uniform spread for range-query tests     |
| `plan_evictable`              | 8-coord uniform spread (eviction tests)  |
| `plan_cousin_rivalry`         | Two adjacent subtree regions             |
| `plan_adversarial_cousins_n4` | Adversarial cousins, N=4                 |
| `plan_adversarial_cousins_n8` | Adversarial cousins, N=8                 |
| `plan_fractal_fill`           | Recursive dyadic fill                    |
| `plan_diamond`                | Spread → hotspot → spread (diamond load) |
| `plan_multi_topology_stress`  | Multi-phase mixed topology stress        |
| `plan_phase_shifted`          | Phase-shifted oscillation preset         |
| `plan_gray_code`              | Gray-code coordinate walk                |
| `plan_multi_plateau`          | Multi-region plateau stress              |

#### Config presets (`config.rs`)

| Preset                    | Key characteristics                             |
| ------------------------- | ----------------------------------------------- |
| `default_config`          | θ=5, D_create=3, D_evict=6, no budget           |
| `no_split_config`         | θ=100, single root node                         |
| `budget_config(n)`        | Default + budget of `n`                         |
| `budget_config_unbounded` | Budgeted with unbounded eviction                |
| `tight_budget_config`     | Minimum viable budget (82)                      |
| `small_buffer_config(n)`  | D_create=2, D_evict=4, budget `n`               |
| `low_threshold_config`    | θ=2, aggressive splitting                       |
| `cascade_config`          | θ=3, D_create=2, D_evict=5                      |
| `deep_config`             | D_create=6, D_evict=12                          |
| `aggressive_config`       | θ=3, D_create=2, D_evict=4, budgeted            |
| `wide_shallow_config`     | θ=8, D_create=2, D_evict=4, wider splits        |
| `worked_example_config`   | Matches idea.md worked example                  |
| `decay_config`            | D_create=4, D_evict=8 for decay tests           |
| `range_tree_config`       | θ=3, D_create=4, D_evict=8 for range queries    |
| `contour_range_config`    | θ=3, D_create=3, D_evict=6, budgeted (contours) |
| `evictable_config`        | θ=3, D_create=2, D_evict=4, tight budget        |
| `f64_default_config`      | f64 baseline: θ=5.0, D_create=3, D_evict=8      |
| `f64_no_split_config`     | f64, θ=100.0, no splits                         |
| `f64_deep_config`         | f64, D_create=6, D_evict=12                     |

#### Runners (`runner.rs`)

| Function                    | Behaviour                                                     |
| --------------------------- | ------------------------------------------------------------- |
| `run()`                     | Apply plan — no invariant checks (fast)                       |
| `run_checked()`             | Assert invariants every `check_every` steps                   |
| `run_soft_checked()`        | Collect violations without panicking                          |
| `run_budget_checked()`      | Assert budget convergence after each observation              |
| `run_diagnostic()`          | Check every invariant after every step; dump on first failure |
| `run_diagnostic_budgeted()` | Diagnostic runner with budget convergence                     |

#### RNG stubs (`rng.rs`)

`TestLcgRng`, `FixedRng`, `SeqRng` — gated behind
`#[cfg(any(debug_assertions, test))]`, unavailable in release
benchmarks. Integration tests that need an RNG in release mode
define their own `TestRng` in `tests/support/mod.rs`.

### 3.3 Tracing

Both `src/tests/mod.rs` and `tests/support/mod.rs` provide an
`init_tracing()` helper. Tests bind the result to keep the span
alive:

```rust
let _t = init_tracing();
```

Output is captured by `cargo test` and only shown on failure (or
with `--nocapture`). With `RUST_LOG=info`, each test prints its
wall-clock duration on span close.

### 3.4 Naming

- Test functions use `snake_case` and describe the scenario, not
  the method under test: `budget_converges_under_500_observations`,
  `adr038_finite_selective_f64_does_not_panic`.
- ADR regression tests are prefixed with the ADR number:
  `adr038_…`.
- Test files in `tests/` are named after the subsystem or concern
  they exercise, not after source modules.

### 3.5 Module-level doc-comments

Every test file opens with a `//!` doc-comment summarising what
the suite covers. Integration tests list the ADR decisions or
spec sections being validated when applicable.

### 3.6 Feature and cfg gating

| Condition                           | Effect                                                                                                                                        |
| ----------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------- |
| `--no-default-features`             | Suites gated on `dynamic-contour-tracking` are excluded; unit and integration counts drop.                                                    |
| `--release` (no `debug_assertions`) | `arena::get_stale_handle_panics_in_debug` excluded (unit: 651→648). `negative_f64` tests exercise release-mode paths instead of debug guards. |

### 3.7 Doc-test policy

Governed by [ADR-M-034](../adr/034-doc-comment-and-doctest-policy.md):

- Every public item has at least one `# Examples` block that
  compiles and runs (plain ` ``` `, not `ignore` / `no_run`).
- Examples are self-contained, use only public re-exports + `std`.
- Boilerplate is hidden behind `# ` lines.
- Examples assert observable outcomes.

---

## §4 Relevant ADRs

| ADR                                                       | Title                           | Testing relevance                                                             |
| --------------------------------------------------------- | ------------------------------- | ----------------------------------------------------------------------------- |
| [ADR-M-032](../adr/032-three-surface-model.md)            | Three-Surface Visibility Model  | Defines which API surface integration tests may use (Surface 1+2 only)        |
| [ADR-M-034](../adr/034-doc-comment-and-doctest-policy.md) | Doc-Comment and Doc-Test Policy | Mandates doc-test coverage on every public item                               |
| [ADR-M-035](../adr/035-benchmarking-framework.md)         | Benchmarking Framework          | Establishes Criterion benchmarks and the bench-mirror correctness requirement |
| [ADR-M-036](../adr/036-sentinel-integration-api.md)       | Sentinel Integration API        | Tested by `tests/sentinel_api.rs`                                             |
| [ADR-M-037](../adr/037-contour-range-queries.md)          | Contour Range Queries           | Tested by `tests/contour_range.rs`                                            |
| [ADR-M-038](../adr/038-decay-factor-table-depth-bound.md) | Decay Factor Table Depth Bound  | Regression tests in `tests/decay_infinite.rs`                                 |
| [ADR-M-039](../adr/039-mutation-testing-gap-analysis.md)  | Mutation Testing Gap Analysis   | Drives `config_validation`, `decompose_basis`, `invariant_self` test suites   |

---

## §5 Test File Index

### 5.1 Unit / crate tests — `src/tests/`

| File                       | Focus                                                |
| -------------------------- | ---------------------------------------------------- |
| `arena.rs`                 | Arena alloc/dealloc/reuse cycles                     |
| `config_validation.rs`     | Config boundary-value tests (ADR-M-039 §D7)          |
| `decay.rs`                 | Decay curves, uniform and selective paths            |
| `decay_f64_depth.rs`       | f64-specific depth behaviour (feature-gated)         |
| `decompose_basis.rs`       | Basis decomposition boundary tests (ADR-M-039)       |
| `diagnostic.rs`            | Diagnostic/invariant reporting internals             |
| `evict.rs`                 | Eviction ordering, eligibility, absorption           |
| `gnode.rs`                 | G-node state, children, interval logic               |
| `graph.rs`                 | Graph-level construction, accessors, round-trips     |
| `graph_init.rs`            | Graph initialisation edge cases                      |
| `gtree.rs`                 | G-tree routing, depth computation, sum recomputation |
| `handle.rs`                | Handle (GNodeId, VNodeId) identity and validity      |
| `invariant_self.rs`        | Invariant checker self-tests (ADR-M-039 §D5)         |
| `observe.rs`               | Observation pipeline internals                       |
| `pewei.rs`                 | PEWEI extraction and reconstruction                  |
| `plan_builders.rs`         | Test-plan builder method correctness                 |
| `plateau.rs`               | Plateau projection, basis edges, thatching           |
| `rebalance.rs`             | V-tree rebalancing (2–3 rotations, promotions)       |
| `rebalance_stress.rs`      | Adversarial rebalancing workloads                    |
| `semi_internal_plateau.rs` | Semi-internal plateau tracking (feature-gated)       |
| `spiked_vtree.rs`          | Deliberately unbalanced V-tree construction          |
| `split.rs`                 | Split logic, threshold, guard conditions             |
| `structural.rs`            | Cross-cutting structural properties                  |
| `traits.rs`                | Coordinate, Accumulator, and property-profile tests  |
| `view.rs`                  | Cell, Span, Node view-type accessors                 |
| `vnode.rs`                 | V-node kind, entry/structural fields                 |
| `vtree.rs`                 | V-tree traversal, layer iteration                    |
| `worked_example.rs`        | End-to-end worked example from idea.md               |

### 5.2 Integration tests — `tests/`

| File                         | Focus                                                         | Key ADRs                                                  |
| ---------------------------- | ------------------------------------------------------------- | --------------------------------------------------------- |
| `bench_mirrors.rs`           | Correctness mirrors for every Criterion benchmark scenario    | [ADR-M-035](../adr/035-benchmarking-framework.md)         |
| `budget.rs`                  | Budget enforcement, over-budget recovery, depth-gate dynamics |                                                           |
| `buffer_oscillation.rs`      | Oscillating observation patterns and buffer stability         |                                                           |
| `cascade.rs`                 | Multi-level split cascades                                    |                                                           |
| `contour_range.rs`           | Contour range decomposition                                   | [ADR-M-037](../adr/037-contour-range-queries.md)          |
| `cross_type.rs`              | Different coordinate/value type combinations                  |                                                           |
| `decay_infinite.rs`          | Infinite attenuation, factor-table depth bound regression     | [ADR-M-038](../adr/038-decay-factor-table-depth-bound.md) |
| `eviction.rs`                | Eviction ordering and correctness                             |                                                           |
| `eviction_diagnostic.rs`     | Diagnostic eviction — checks every invariant after every step |                                                           |
| `eviction_plateau.rs`        | Eviction ↔ plateau invariant maintenance (P-I2, P-I4, P-I5)   |                                                           |
| `graph_extract.rs`           | PEWEI extraction correctness                                  |                                                           |
| `graph_layers.rs`            | V-Tree layer structure                                        |                                                           |
| `graph_point.rs`             | Point queries                                                 |                                                           |
| `graph_range.rs`             | Range queries                                                 |                                                           |
| `graph_sample.rs`            | Proportional sampling correctness                             |                                                           |
| `graph_terminal.rs`          | Terminal node behaviour                                       |                                                           |
| `hex_binary_tree_mapping.rs` | Hex ↔ binary tree coordinate bijection                        |                                                           |
| `negative_f64.rs`            | Negative `f64` observations and property-profile violations   |                                                           |
| `plateau.rs`                 | Plateau projection                                            |                                                           |
| `sentinel_api.rs`            | Sentinel integration API (gnode_id, gnode_info, ancestry)     | [ADR-M-036](../adr/036-sentinel-integration-api.md)       |
| `stress_patterns.rs`         | High-volume adversarial and randomised workloads              |                                                           |

### 5.3 Shared test support

| Path                   | Purpose                                                             |
| ---------------------- | ------------------------------------------------------------------- |
| `tests/support/mod.rs` | `init_tracing()`, release-mode `TestRng`                            |
| `src/testing/`         | `Plan`, builders, config presets, runners (§3.2)                    |
| `src/invariants.rs`    | `assert_invariants()`, `check_all_invariants()`, diagnostic dumpers |

### 5.4 Benchmarks — `benches/`

| File              | What it measures                               |
| ----------------- | ---------------------------------------------- |
| `observe.rs`      | Observation throughput                         |
| `query.rs`        | Point and range query latency                  |
| `extract.rs`      | PEWEI extraction cost                          |
| `lifecycle.rs`    | Full create → populate → query → extract cycle |
| `spray.rs`        | Bulk random-insert throughput                  |
| `pathological.rs` | Adversarial access patterns                    |
| `bench_rng.rs`    | Release-mode LCG shared by all benchmarks      |

### 5.5 Bench mirrors — `tests/bench_mirrors/`

Each file mirrors the corresponding benchmark family, replaying
the same plan × config pair with full invariant checking in debug
mode ([ADR-M-035](../adr/035-benchmarking-framework.md)):

`extract.rs`, `lifecycle.rs`, `observe.rs`, `pathological.rs`,
`query.rs`, `spray.rs`.

---

## §6 Running Tests

### All tests (recommended)

```sh
# Unit + integration tests (debug, optimised build):
CARGO_PROFILE_DEV_OPT_LEVEL=3 cargo test -p torrust-mudlark --all-targets --all-features

# Doc-tests (separate cargo invocation required):
cargo test -p torrust-mudlark --all-features --doc

# Release mode (catches cfg(debug_assertions) differences):
cargo test -p torrust-mudlark --all-targets --all-features --release

# No-default-features spot-check:
CARGO_PROFILE_DEV_OPT_LEVEL=3 cargo test -p torrust-mudlark --all-targets --no-default-features
cargo test -p torrust-mudlark --no-default-features --doc
```

> **Tip:** Debug-mode tests without `CARGO_PROFILE_DEV_OPT_LEVEL=3` are
> noticeably slow due to the heavy numeric workloads in several
> integration suites. Always set the env var for a reasonable
> edit-compile-test cycle.

### Clippy & docs

```sh
cargo clippy -p torrust-mudlark --all-targets --all-features
cargo doc -p torrust-mudlark --all-features --no-deps
```

### Benchmarks

```sh
cargo bench -p torrust-mudlark --all-features
```

Results are written to `target/criterion/` with HTML reports.
See [performance.md](performance.md) for analysis and characterisation.
