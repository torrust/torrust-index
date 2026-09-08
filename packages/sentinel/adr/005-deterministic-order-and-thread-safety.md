# ADR-S-005: Deterministic Order and Thread Safety · `rec:sentinel:ordered-maps-and-static-send-sync-assertion`

**Status:** Implemented **Date:** 2026-03-08 **Spec:** §ALGO S-11.1 (noise injection), §ALGO S-11.3 (seed parameter) **Relates to:** mudlark [ADR-M-007](../../mudlark/adr/007-thread-safety.md) (mudlark thread safety)

## Context · `sec:sentinel:determinism-context`

Two low-level invariants govern the sentinel's runtime behaviour:

1. **Iteration order must be deterministic** across runs for a given seed, so that noise injection, coordination assembly, and report ordering are reproducible.
2. **`SpectralSentinel` must be `Send + Sync`**, because the host will run it on background threads behind `Arc<Mutex<_>>`.

Both are implementation choices that are not specified in algorithm.md but are required by the sentinel's operating environment.

---

## Part A — Deterministic Iteration Order · `sec:sentinel:determinism-iteration-order`

### Decision · `sec:sentinel:determinism-iteration-decision`

**Use `BTreeMap` (not `HashMap`) for all spatial-key maps.**

```rust
cells: BTreeMap<GNodeId, CellState>,
```

`BTreeMap` iterates in key order (ascending). `GNodeId` is `Copy + Ord` (backed by `NonZeroU32`), providing a stable, cross-platform iteration order.

### Why it matters · `sec:sentinel:determinism-iteration-rationale`

- **Noise injection** — each tracker receives a deterministic RNG sequence seeded from a user-provided seed. If iteration order varies between runs, the same seed produces different per-tracker baselines, making behaviour non-reproducible.
- **Coordination tier** — cell mean-score vectors are assembled into a matrix in iteration order. The meta-tracker's SVD decomposition is sensitive to row ordering when eigenvalues are close.
- **Report ordering** — `cell_reports` appear in the `BatchReport` in map iteration order. Deterministic ordering makes output diffable across runs.

### Alternatives · `sec:sentinel:determinism-iteration-alternatives`

| Option | Pros | Cons |
|--------|------|------|
| `HashMap` + sort-on-iterate | O(1) lookup | Sort cost on every noise/report pass; easy to forget |
| `IndexMap` (insertion order) | Preserves insert order | Order depends on traffic arrival, not spatial position; extra dependency |
| `BTreeMap` (chosen) | Deterministic, no extra sort, no extra dep | O(log n) lookup vs O(1) |

The O(log n) penalty is negligible — cell counts are small relative to the per-tracker SVD cost that dominates runtime.

---

## Part B — Thread Safety via Static Assertion · `sec:sentinel:determinism-thread-safety`

### Decision · `sec:sentinel:determinism-thread-safety-decision`

**Enforce `Send + Sync` with a compile-time static assertion.**

```rust
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SpectralSentinel>();
};
```

This is a zero-cost check — no runtime code. If any field ever violates the constraint, the crate fails to compile with a clear error pointing at the assertion.

### Why a static assertion (not trust-the-derive) · `sec:sentinel:determinism-static-assertion-rationale`

All current fields are `Send + Sync` by construction. But future additions (`GvGraph`, `SmallRng`, etc.) could silently break this. The static assertion catches breakage at `cargo check` time, not at a downstream integration test.

- `GvGraph<C, V, N>` is `Send + Sync` (mudlark ADR-M-007).
- `SmallRng` is `Send + Sync` (no `Rc` or thread-local state).

## Consequences · `sec:sentinel:determinism-consequences`

- Noise injection with a fixed seed produces identical baselines across runs, platforms, and Rust versions.
- Reports are spatially ordered without an explicit sort pass.
- Any future `!Send` field (e.g. a raw pointer cache) will be caught immediately, forcing a conscious design choice.
