# ADR-S-014: SubspaceTracker Visibility · `rec:sentinel:private-tracker-with-public-baseline-snapshots`

**Status:** Decided **Date:** 2026-03-10 **Spec:** §ALGO S-5 (subspace tracker), §ALGO S-11.1 (noise injection) **Relates to:** [ADR-S-007](007-automatic-noise-injection.md) (automatic noise injection), [ADR-S-013](013-warm-up-convergence-benchmark.md) (warm-up convergence benchmark), [ADR-S-001](001-measures-not-opinions.md) (measures not opinions)

## Context · `sec:sentinel:trackervis-context`

`SubspaceTracker` is the statistical core of the sentinel — a low-rank online subspace model with four-axis scoring and EWMA baselines.  It is currently declared `pub struct` in a `pub(crate) mod tracker`, making it **visible within the crate but invisible to external consumers** (integration tests under `tests/`, Criterion benchmarks under `benches/`, and downstream crates).

This visibility boundary has been adequate so far: every integration test, benchmark, and host interaction operates exclusively through the `SpectralSentinel` public API (`ingest`, `inspect_cell`, `health`, `decay`, `BatchReport`, etc.).

ADR-S-013 introduces a new testing need that challenges this boundary.  The convergence benchmark's `noise_baselines_converge` test (§3) requires:

1. **Direct tracker construction** — create a bare `SubspaceTracker::new(dim, &cfg, slow_decay)` without sentinel orchestration overhead (no G-V Graph, no analysis set, no coordination).
2. **Repeated `observe()` calls with `is_noise = true`** — feed synthetic noise batches one at a time and inspect the EWMA baselines after each round.
3. **Access to `TrackerReport::scores.*.baseline.mean`** — read the per-axis EWMA mean from the report returned by `observe()`.

The `SpectralSentinel` API is insufficient for this because:

- `inspect_cell()` returns `CellInspection`, which exposes `maturity` (including `noise_influence`) but **not** the per-axis EWMA baseline means/variances.  These are only available in `CellReport::scores.*.baseline` and `TrackerReport::scores.*.baseline`, which come from `ingest()` reports.
- Using `ingest()` for this test adds sentinel orchestration overhead (graph routing, analysis set reconciliation, coordination) that obscures the measurement and slows the test unnecessarily.
- Noise injection is automatic and internal (ADR-S-007). There is no way to call `observe(is_noise = true)` on a tracker through the public API — the sentinel owns the injection lifecycle.

### The deeper question · `sec:sentinel:trackervis-deeper-question`

Should `SubspaceTracker` be part of the crate's public API?

This is not just about one test.  It's about the crate's API philosophy and impacts:

- **Host-side subspace analysis.**  Some hosts may want to run a standalone tracker outside of sentinel orchestration — e.g. for offline analysis, benchmarking specific workloads, or building custom pipelines.
- **Fuzz testing.**  External fuzzer harnesses can exercise the tracker directly without paying graph construction costs.
- **Benchmark granularity.**  Criterion benchmarks can isolate tracker performance (SVD cost, EWMA update cost) without sentinel overhead.
- **Semver surface area.**  Any `pub` type is a commitment. Changes to `SubspaceTracker`'s constructor signature, field layout, or `observe()` return type become breaking changes.
- **Encapsulation.**  ADR-S-007 establishes that the sentinel owns the noise injection lifecycle.  Exposing `observe()` with its `is_noise` parameter hands that control to external callers, creating ordering hazards (double-injection, skipped injection, interleaved noise/real traffic).
- **`AxisBaseline` / `EwmaStats` visibility.**  Making the tracker public is only useful if callers can also construct configs and read results.  `SentinelConfig` and `TrackerReport` are already public.  But `AxisBaseline` is private, and `EwmaStats` is `pub` in `ewma.rs`.

## Decision · `sec:sentinel:trackervis-decision`

**Option E — hybrid: keep `SubspaceTracker` as `pub(crate)`, enrich `CellInspection` with baseline snapshots.**

See [Recommendation](#recommendation) below for rationale.

## Decision Options Considered · `sec:sentinel:trackervis-options`

### Option A: Keep `pub(crate)`, test inside the crate · `sec:sentinel:trackervis-option-private-crate-tests`

Leave `SubspaceTracker` as `pub(crate)`.  Place any test that needs direct tracker access inside the crate boundary:

- **`noise_baselines_converge`** → `#[cfg(test)] mod` inside `src/sentinel/tracker.rs`.
- **Tracker-level benchmarks** → not possible via Criterion (benches are external); would need to be approximated through the sentinel API or use `cargo bench` with the `test` harness.

**Pro:**
- Zero semver surface change.
- Encapsulation preserved: `SubspaceTracker`'s API can evolve freely.
- ADR-S-007's lifecycle invariant ("the sentinel owns injection") is enforced by the type system — external code literally cannot call `observe(is_noise = true)`.
- No risk of misuse by downstream crates.
- Unit tests in `src/` have full `pub(crate)` access with no workarounds needed.

**Con:**
- Tests that need tracker access must live in `src/`, not in the `tests/` folder.  This mixes test code with production code (though `#[cfg(test)]` ensures it is stripped from release builds).
- Criterion benchmarks cannot isolate tracker-level performance without a public API.  Sentinel-level benchmarks are a proxy but include graph and analysis set overhead.
- No path for hosts to use the tracker standalone.

### Option B: Promote to `pub mod tracker` · `sec:sentinel:trackervis-option-public-module`

Change `pub(crate) mod tracker` → `pub mod tracker`, making `SubspaceTracker`, its constructor, `observe()`, `maturity()`, `rank()`, and `reset_cusum()` part of the crate's public API.

**Pro:**
- Integration tests and Criterion benches can construct trackers directly.
- Hosts can build custom pipelines with standalone trackers.
- Benchmark isolation: measure SVD/EWMA costs without sentinel overhead.
- Fuzz testing can target the tracker in external harnesses.

**Con:**
- **Semver commitment.**  `SubspaceTracker::new()` signature, `observe()` parameters, and `TrackerReport` fields all become stable API surface.  Any internal refactor (e.g. changing the basis representation, adding parameters to `observe()`) is a breaking change.
- **Breaks ADR-S-007's lifecycle invariant.**  External callers can call `observe(is_noise = true)` arbitrarily, bypassing the sentinel's controlled injection sequence.  While `SubspaceTracker` is stateless w.r.t. injection ordering (it doesn't panic or corrupt), the *host's interpretation* of maturity becomes unreliable if injection was manual and uncontrolled.
- **AxisBaseline exposure cascade.**  Making the module public invites questions about `AxisBaseline`, `CusumAccumulator`, and the internal scoring pipeline.  These are currently private structs.
- **Documentation burden.**  A public tracker needs doc-comments, usage examples, and safety guidance ("do not mix `is_noise` calls with real observations outside sentinel orchestration").

### Option C: Test-only feature gate · `sec:sentinel:trackervis-option-test-feature`

Add a Cargo feature `test-internals` (default off) that conditionally re-exports internal types:

```rust
#[cfg(feature = "test-internals")]
pub mod test_internals {
    pub use super::tracker::SubspaceTracker;
    pub use super::cusum::CusumAccumulator;
}
```

Integration tests and benches enable it via `[dev-dependencies]` features.

**Pro:**
- Tracker is accessible in tests and benches without being part of the default public API.
- No semver commitment to non-`test-internals` consumers.
- ADR-S-007 lifecycle invariant is preserved for production builds.
- Clean separation: test authors explicitly opt in.

**Con:**
- Feature gates add conditional-compilation complexity.
- `test-internals` is a social contract, not a hard boundary — any downstream crate can enable it.
- Documentation must explain the feature and its warranty limitations.
- `cargo doc` with `--all-features` exposes the internal types, cluttering the generated docs unless `#[doc(hidden)]` is used, which defeats discoverability for test authors.

### Option D: Expose a limited inspection API on `SpectralSentinel` · `sec:sentinel:trackervis-option-inspection-api`

Instead of exposing the tracker, add a method like `inspect_cell_baselines(gnode) -> Option<BaselineInspection>` that returns per-axis EWMA means and variances.  Combined with `inspect_cell()` (maturity) and `ingest()` (reports), this covers the convergence test's needs without exposing the tracker.

```rust
pub struct BaselineInspection {
    pub novelty: BaselineSnapshot,
    pub displacement: BaselineSnapshot,
    pub surprise: BaselineSnapshot,
    pub coherence: BaselineSnapshot,
}
```

**Pro:**
- Narrow API addition — one method, one struct.
- Encapsulation preserved: `SubspaceTracker` remains internal.
- Fulfils the convergence test's data needs (`inspect_cell()` + `inspect_cell_baselines()` after each `ingest()`).
- Useful for hosts too (monitoring baseline drift).

**Con:**
- Does not address the "test tracker in isolation" need.  The `noise_baselines_converge` test (ADR-S-013 §3) specifically wants to measure noise-only convergence without real data. Through the sentinel API, you cannot feed noise without also triggering graph routing / analysis set / coordination.
- Two-step inspection (`inspect_cell` + `inspect_cell_baselines`) is slightly awkward; could be unified into a richer `CellInspection` instead.

### Option E: Hybrid — `pub(crate)` (Option A) + enriched inspection (Option D) · `sec:sentinel:trackervis-option-hybrid`

Keep `SubspaceTracker` as `pub(crate)`.  Place the noise-only convergence test inside the crate (`#[cfg(test)]` in `tracker.rs`).  Additionally, enrich `CellInspection` with baseline snapshots so that the integration-level convergence test (`convergence_matches_theory`) doesn't need to hunt through `BatchReport` arrays:

```rust
pub struct CellInspection {
    // ... existing fields ...
    /// Per-axis EWMA baseline snapshots.
    pub baselines: AxisBaselineSnapshots,
}

pub struct AxisBaselineSnapshots {
    pub novelty: BaselineSnapshot,
    pub displacement: BaselineSnapshot,
    pub surprise: BaselineSnapshot,
    pub coherence: BaselineSnapshot,
}
```

**Pro:**
- The noise-only test lives where it has full access (inside the crate), matching the principle that unit tests belong near the code they test.
- The integration-level convergence test gets clean baseline access through the public API without parsing `BatchReport` cell/ancestor arrays.
- No semver exposure of `SubspaceTracker`.
- Enriched `CellInspection` is independently useful for hosts monitoring baseline drift.
- Clean separation of concerns: noise convergence tested at the unit level, theory + EWMA convergence tested at the integration level.

**Con:**
- Criterion benchmarks still cannot isolate tracker-level performance.  (Mitigated: the existing benchmark suite already measures sentinel-level throughput at various scales, and the convergence benchmark measures batch-over-time cost through the sentinel API.)
- Adds a struct and 4 fields to `CellInspection` (minor API growth).

## Recommendation · `sec:sentinel:trackervis-recommendation`

**Option E — hybrid.**

The rationale:

1. **`SubspaceTracker` is an implementation detail.**  Its constructor takes a `slow_decay: f64` parameter that only makes sense in the context of the sentinel's two-tier EWMA architecture.  Its `observe()` method has an `is_noise: bool` parameter whose correct usage depends on the injection lifecycle described in ADR-S-007.  Exposing these to external callers invites misuse without adding proportional value.

2. **The one test that needs internal access is a unit test.** `noise_baselines_converge` exercises a single tracker's EWMA convergence — a textbook unit test.  Placing it in `#[cfg(test)]` inside `tracker.rs` is idiomatic Rust.

3. **Enriching `CellInspection` is the right public API evolution.**  Baseline means and variances are legitimate observable state.  Hosts monitoring the sentinel in production will benefit from baseline visibility in `inspect_cell()` without needing to parse `BatchReport` arrays.  This is fully aligned with "the sentinel measures; the host decides" (ADR-S-001).

4. **Semver discipline.**  The crate's public surface is covered by semver guarantees from 1.0.0 onwards, and keeping the internal machinery private is what makes that guarantee affordable: a `pub(crate)` tracker can be reshaped in a compatible release, whereas a public `SubspaceTracker` would fix its constructor signature, field layout, and `observe()` return type for the life of the major version.  If a genuine need for standalone trackers emerges (e.g. a host's custom analysis pipeline), a future ADR can promote visibility with an intentional, documented API — an addition rather than a break.

5. **Benchmark isolation is a non-goal for now.**  The existing Criterion suite measures ingest throughput at various scales. The convergence benchmark (ADR-S-013 §1) measures 200-batch convergence wall-clock at the sentinel level, which is the relevant integration point.  Tracker-level micro- benchmarks would be useful but are not blocked by this decision — they can live in `#[cfg(test)]` `mod benches` inside `tracker.rs` using the `test::Bencher` nightly API, or be added later if Option B is adopted.

## Consequences · `sec:sentinel:trackervis-consequences`

1. **`noise_baselines_converge`** is implemented as a `#[cfg(test)]` unit test in `src/sentinel/tracker.rs`.

2. **`CellInspection`** gains a `baselines: AxisBaselineSnapshots` field populated from the tracker's four `AxisBaseline` fast- EWMA snapshots.

3. **`AxisBaselineSnapshots`** is a new public type in `report.rs`, containing four `BaselineSnapshot` fields.

4. **`inspect_cell()`** in `SpectralSentinel` populates the new field from `cell.tracker.axis_baselines()` (already a `pub(crate)` method that returns the necessary data).

5. **`convergence_matches_theory`** (integration test) reads baselines from `inspect_cell().baselines` instead of parsing `BatchReport` arrays.  This simplifies the test and removes the dependency on knowing whether the root is in `cell_reports` or `ancestor_reports`.

6. **No change** to `SubspaceTracker`'s visibility (`pub(crate)`).

7. **Future reconsideration.**  If demand emerges for standalone tracker usage (host custom pipelines, external fuzz harnesses), revisit this ADR.  The migration path is straightforward: change `pub(crate) mod tracker` → `pub mod tracker` and document the `is_noise` lifecycle contract.

## Files Changed · `sec:sentinel:trackervis-files-changed`

| File | Change |
|------|--------|
| `src/report.rs` | Add `AxisBaselineSnapshots` struct; add `baselines` field to `CellInspection` |
| `src/sentinel/mod.rs` | Populate `baselines` in `inspect_cell()` |
| `src/sentinel/tracker.rs` | Add `#[cfg(test)] mod convergence_tests` (noise_baselines_converge) — now in `src/tests/convergence_noise.rs` |

## Cross-References · `sec:sentinel:trackervis-cross-references`

- §ALGO S-5 — Subspace tracker specification
- §ALGO S-7.1 — EWMA baseline update rule
- §ALGO S-11.1–11.4 — Noise injection lifecycle
- ADR-S-001 — Measures not opinions (API philosophy)
- ADR-S-007 — Automatic noise injection (lifecycle ownership)
- ADR-S-013 — Warm-up convergence benchmark (motivating use case)
