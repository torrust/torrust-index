# Testing

## Running Tests

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

---

## Test Suite Breakdown

868 tests across unit, integration, and doc-test suites (with `--all-features`).

### Unit tests (`src/`)

| Suite      | Tests |
| ---------- | ----: |
| Unit tests |   346 |

These live alongside the source in `src/**/tests/` modules and cover
internal invariants, arena operations, decay curves, tree balancing,
split logic, and the observation pipeline.

> In `--release` mode, `arena::get_stale_handle_panics_in_debug` is
> excluded (`#[cfg(debug_assertions)]` only), bringing the unit count
> to **345**.

### Integration tests (`tests/`)

| Suite                     |   Tests | Focus                                                      |
| ------------------------- | ------: | ---------------------------------------------------------- |
| `bench_mirrors`           |      43 | Correctness mirrors for every Criterion benchmark scenario |
| `budget`                  |      24 | Budget enforcement, over-budget recovery                   |
| `buffer_oscillation`      |      23 | Oscillating observation patterns and buffer stability      |
| `cascade`                 |      18 | Multi-level split cascades                                 |
| `contour_range`           |      23 | Contour range decomposition (ADR-M-037, revised)           |
| `cross_type`              |      14 | Different coordinate/value type combinations               |
| `decay_infinite`          |      25 | Infinite attenuation, factor-table depth bound (ADR-M-038) |
| `eviction`                |      16 | Eviction ordering and correctness                          |
| `eviction_debug`          |       7 | Debug-only eviction assertions                             |
| `eviction_p_i2`           |       4 | P-I² eviction policy edge cases                            |
| `graph_extract`           |       9 | PEWEI extraction correctness                               |
| `graph_layers`            |      15 | V-Tree layer structure                                     |
| `graph_point`             |      13 | Point queries                                              |
| `graph_range`             |      12 | Range queries                                              |
| `graph_sample`            |       9 | Proportional sampling correctness                          |
| `graph_terminal`          |       7 | Terminal node behaviour                                    |
| `hex_binary_tree_mapping` |      53 | Hex ↔ binary tree coordinate mapping                       |
| `negative_f64`            |       2 | Negative `f64` observations and property-profile violations |
| `plateau`                 |      22 | Plateau projection                                         |
| `sentinel_api`            |      14 | Sentinel integration API (ADR-M-036)                       |
| `stress_patterns`         |      62 | High-volume adversarial and randomised workloads           |
| **Subtotal**              | **415** |                                                            |

### Doc-tests

| Suite     | Tests |
| --------- | ----: |
| Doc-tests |   107 |

106 run normally; 1 is `#[ignore]`d (long-running demonstration).

### Grand total

| Category    |   Tests |
| ----------- | ------: |
| Unit        |     346 |
| Integration |     415 |
| Doc-tests   |     107 |
| **Total**   | **868** |

---

## Feature-gated Test Variation

Some tests are gated behind feature flags or `cfg` attributes:

| Condition                           | Effect                                                                                                                                                       |
| ----------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `--no-default-features`             | Unit tests drop to ~303; `budget`, `contour_range`, `eviction`, `plateau`, and `stress_patterns` suites see fewer or zero tests.                             |
| `--release` (no `debug_assertions`) | `arena::get_stale_handle_panics_in_debug` is excluded (unit: 346 → 345). `eviction_debug` suite still runs its 7 tests but they exercise release-mode paths. |

---

## Benchmarks

Six Criterion benchmark suites live in `benches/`:

| Benchmark      | What it measures                               |
| -------------- | ---------------------------------------------- |
| `observe`      | Observation throughput                         |
| `query`        | Point and range query latency                  |
| `extract`      | PEWEI extraction cost                          |
| `lifecycle`    | Full create → populate → query → extract cycle |
| `spray`        | Bulk random-insert throughput                  |
| `pathological` | Adversarial access patterns                    |

Run them with:

```sh
cargo bench -p torrust-mudlark --all-features
```

Results are written to `target/criterion/` with HTML reports.

See [performance.md](performance.md) for analysis and characterisation.
