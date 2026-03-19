# ADR-M-028: Span-Native Tracing for Recursive Tree Operations

**Status:** Decided (Implemented)  
**Date:** 2026-03-02 (updated 2026-03-07)  
**Phase:** 6 (Hardening)  
**Spec:** §IDEA M-8 (observation flow), §IDEA M-10 (contour refinement),
§IDEA M-11 (rebalancing), §IDEA M-12 (contour simplification),
§IDEA M-14 (temporal semantics)  
**Relates to:** [ADR-M-003](003-violation-tracking.md) (violation
tracking), [ADR-M-026](026-point-query-and-plateau-semantics.md)
(plateau semantics), [ADR-M-030](030-graph-module-decomposition.md)
(graph module decomposition — complete)  
**Surface:** 3 (Emulsion)

## Context

The GvGraph is two **recursive trees** sharing nodes. Every mutation
(observe → split → rebalance → evict) is a pipeline of recursive
walks through these trees. The current diagnostic infrastructure uses
a mix of three incompatible mechanisms:

| Mechanism                         | Count | Primary files                                                                               | Problem                                                  |
| --------------------------------- | ----- | ------------------------------------------------------------------------------------------- | -------------------------------------------------------- |
| `eprintln!`                       | ~58   | `evict.rs` (34), `graph_plateau.rs` (17), `graph_budget.rs` (7)                             | No filtering, no structure, no capture                   |
| `#[cfg(debug_assertions)]` blocks | ~39   | 26 standalone + 13 double-gated with `feature = "dynamic-contour-tracking"`, across 8 files | Invisible in release builds; mixes output and assertions |
| `tracing::` macros                | ~69   | mostly `rebalance.rs` (44), plus `evict.rs`, `observe.rs`, `graph_plateau.rs`               | Good but uneven — other modules don't participate        |

### The recursion visibility problem

Flat `eprintln!` output loses the single most important piece of
context: **where in the recursion you are**. When debugging a missed
violation in eviction, knowing that the violation occurred during
"phase 8 plateau maintenance → tiling repair → displaced element
placement" is essential. Flat output shows all three interleaved.

The `rebalance.rs` module already uses `tracing` effectively:

```rust
let _span = tracing::debug_span!("resolve", node = c.index()).entered();
tracing::debug!(ctx = %Ctx(vnodes, c), "begin");
```

This produces hierarchical output when a subscriber is attached:

```
resolve{node=5}
  ├─ contract{p=3}
  │    └─ merged=v7
  └─ standard_promote{c=5}
```

However, `evict.rs` (802 lines) has **zero spans** and relies on
`eprintln!` walls like the 157-line `diagnose_missed_violation`
function (lines 626–782). Similarly, `decay.rs` (272 lines) has
**zero tracing** despite being a top-level mutation entry point.
`graph_plateau.rs` (1530 lines) has 17 `eprintln!` calls in its
`debug_assert_plateau_mirror_consistency` function — a structured
divergence dump that should be structured tracing output.

### Current instrumentation gaps

| Function                                  | Module             | Lines | Spans | Events | Assessment                                          |
| ----------------------------------------- | ------------------ | ----- | ----- | ------ | --------------------------------------------------- |
| `evict_tip`                               | `evict.rs`         | ~270  | **0** | 3      | Worst gap — 8 steps, no nesting                     |
| `plateau_after_evict`                     | `evict.rs`         | ~210  | **0** | 3      | Complex plateau maintenance, `cfg`-gated            |
| `catalytic_split`                         | `split.rs`         | ~80   | **0** | 0      | No instrumentation                                  |
| `bootstrap_split`                         | `split.rs`         | ~70   | **0** | 0      | No instrumentation                                  |
| `decay`                                   | `decay.rs`         | ~210  | **0** | 0      | Top-level mutation, invisible                       |
| `evict_candidates`                        | `graph_budget.rs`  | ~100  | **0** | 0      | Batch loop, invisible                               |
| `check_evictions`                         | `graph_budget.rs`  | ~10   | **0** | 0      | Public entry point, delegates to `evict_candidates` |
| `vtree_remove_leaf`                       | `vtree.rs`         | ~60   | **0** | 0      | 3 control-flow cases                                |
| `debug_assert_plateau_mirror_consistency` | `graph_plateau.rs` | ~90   | **0** | 0      | 17 `eprintln!` calls                                |
| `observe`                                 | `observe.rs`       | ~130  | 1 ✓   | 5 ✓    | Adequate (1 span + `debug`/`warn` events)           |
| `attempt_split`                           | `split.rs`         | ~50   | 1 ✓   | 0      | `split_preprocess` span for 3-node contraction      |
| `rebalance`                               | `rebalance.rs`     | ~90   | 1 ✓   | 8 ✓    | Good                                                |
| `resolve`                                 | `rebalance.rs`     | ~100  | 1 ✓   | 10 ✓   | Good                                                |
| `contract`                                | `rebalance.rs`     | ~70   | 1 ✓   | 1 ✓    | Good                                                |
| `standard_promote`                        | `rebalance.rs`     | ~80   | 1 ✓   | 1 ✓    | Good                                                |
| `skip_promote`                            | `rebalance.rs`     | ~80   | 1 ✓   | 1 ✓    | Good                                                |
| `legacy_promote`                          | `rebalance.rs`     | ~90   | 1 ✓   | 1 ✓    | Good                                                |
| `escalate`                                | `rebalance.rs`     | ~50   | 1 ✓   | 0      | Good                                                |

### Duplicated diagnostic code

Two near-identical functions exist:

| Function                           | Location       | Lines | Extra params                |
| ---------------------------------- | -------------- | ----- | --------------------------- |
| `debug_assert_plateau_consistency` | `evict.rs:514` | ~65   | `parent_id`, `parent_state` |
| `debug_assert_plateau_consistency` | `split.rs:267` | ~20   | None                        |

The `evict.rs` version performs an additional P-I4 spot-check.
Both enforce forward/back consistency and plateau count matching.
This duplication invites divergence and complicates maintenance.

Additionally, `diagnose_missed_violation` in `evict.rs` (lines
626–782) is a 157-line `eprintln!` wall that should be structured
tracing output.

## Decision

**Replace ad-hoc diagnostics with span-native tracing.**

All four decisions below are implemented.

1. ✅ **Create a `diagnostic.rs` module** housing consolidated audit
   functions and structured display helpers.

2. ✅ **Add spans to all recursive functions.** Every function that
   walks a tree gets a span. Point-in-time observations (violation
   found, energy absorbed, plateau placed) are events inside spans.

3. ✅ **Convert `#[cfg(debug_assertions)]` diagnostic blocks** to
   `tracing::enabled!()` guards, making them available in release
   builds when a subscriber is attached. **Retain
   `feature = "dynamic-contour-tracking"` gates** on plateau-specific
   checks — those depend on the plateau mirror data structure, not
   just on debug intent.

4. ✅ **Keep `debug_assert!` and panic paths unchanged.** Hard
   invariant checks (G-sum conservation, arena liveness, bounds)
   remain compile-time gated panics.

### DC-028-1: Span placement — ✅ Implemented

Every recursive entry point gets a span:

| Function              | Module            | Span name             | Key fields                        |
| --------------------- | ----------------- | --------------------- | --------------------------------- |
| `evict_tip`           | `evict.rs`        | `evict_tip`           | `v_id`, `gnode`, `parent`         |
| `plateau_after_evict` | `evict.rs`        | `plateau_after_evict` | `gnode`, `parent`, `parent_state` |
| `catalytic_split`     | `split.rs`        | `catalytic_split`     | `g_id`, `lo`, `hi`, `mid`         |
| `bootstrap_split`     | `split.rs`        | `bootstrap_split`     | `g_id`, `lo`, `hi`                |
| `decay`               | `decay.rs`        | `decay`               | `root`, `attenuation`, `q`        |
| `evict_candidates`    | `graph_budget.rs` | `evict_batch`         | `candidate_count`                 |
| `check_evictions`     | `graph_budget.rs` | `check_evictions`     | `d_evict`                         |
| `vtree_remove_leaf`   | `vtree.rs`        | `vtree_remove_leaf`   | `v_id`, `case` (deferred)         |

Existing spans in `rebalance.rs` (`resolve`, `rebalance`, `contract`,
`standard_promote`, `skip_promote`, `legacy_promote`, `escalate`)
and in `observe.rs` (`observe`) and `split.rs` (`split_preprocess`)
remain unchanged.

#### DC-028-1b: Plateau maintenance spans (complete)

Post-implementation audit identified plateau maintenance functions
as the remaining visibility gap — they nest inside the spans above
but produce no tracing output of their own. All six spans are now
implemented:

| Function                                | Module             | Span name                               | Key fields                         | Level |
| --------------------------------------- | ------------------ | --------------------------------------- | ---------------------------------- | ----- |
| `normalize_plateaus`                    | `graph_plateau.rs` | `normalize_plateaus`                    | —                                  | DEBUG |
| `plateau_after_catalytic_split`         | `graph_plateau.rs` | `plateau_after_catalytic_split`         | `g_id`, `left`                     | DEBUG |
| `plateau_after_bootstrap_split`         | `graph_plateau.rs` | `plateau_after_bootstrap_split`         | `g_id`, `left`                     | DEBUG |
| `plateau_after_legacy_promotes_batched` | `graph_plateau.rs` | `plateau_after_legacy_promotes_batched` | `count`                            | DEBUG |
| `repair_p_i4`                           | `graph_plateau.rs` | `repair_p_i4`                           | `pending`, `candidates` (deferred) | DEBUG |
| `scan_for_candidates`                   | `evict.rs`         | `scan_for_candidates`                   | —                                  | TRACE |

Additionally, `trace!` events on `place_basis_element` match arms
and `debug!` events on `adjust_depth_gates` depth changes complete
the coverage.

### DC-028-2: Diagnostic module structure — ✅ Implemented

```rust
// src/diagnostic.rs

/// Verify no violations exist outside the work queue.
/// O(n) scan — guards behind `tracing::enabled!(Level::DEBUG)`.
pub(crate) fn audit_violations<V: Accumulator>(
    vnodes: &Arena<VNode<V>>,
    violations: &[VNodeId],
    checkpoint: &str,
) -> Vec<VNodeId>;

/// Post-mutation plateau consistency check.
/// Forward/back map agreement, plateau count, P-I4 spot-check.
///
/// Requires `feature = "dynamic-contour-tracking"`.
#[cfg(feature = "dynamic-contour-tracking")]
pub(crate) fn audit_plateau_consistency<C, V, const N: u32>(
    graph: &GvGraph<C, V, N>,
    checkpoint: &str,
    context: Option<&PlateauAuditContext>,
);

/// Structured eviction context for diagnostics.
pub(crate) struct PlateauAuditContext {
    pub parent_id: GNodeId,
    pub parent_state: GState,
}

/// Diagnose a missed violation with structured V-tree ancestry.
/// Emits `tracing::error!` with full context.
pub(crate) fn diagnose_missed_violation<V: Accumulator>(
    vnodes: &Arena<VNode<V>>,
    violated: VNodeId,
    context: &EvictionContext,
);

/// G-tree display helper: `G5(T,[0,128),sum=42)`.
pub(crate) struct Gn<'a, C, V>(...);

/// Plateau display helper: `P([0,128),d=3,sum=42)`.
pub(crate) struct Pl<'a, C, V, const N: u32>(...);
```

The `Nd`, `Ch`, `Ctx` display helpers remain in `rebalance.rs` —
they're V-tree formatters co-located with V-tree operations. The
new `Gn` and `Pl` helpers are G-tree/plateau formatters, co-located
with graph-wide diagnostics.

### DC-028-3: `#[cfg(debug_assertions)]` migration — ✅ Implemented

**Convert to tracing-gated (remove the `cfg`):**

These blocks are diagnostic output that should be available in
release builds when debugging production issues:

- `evict.rs`: Missed violation scan (line 233), plateau case traces
  (lines 425, 466)
- `graph_budget.rs`: Violation audit call in `evict_candidates`
- `observe.rs`: Violation queue audit (line 74), post-observe
  residual check (line 152)
- `rebalance.rs`: `debug_audit_violation_queue` call sites

Pattern:

```rust
// Before
#[cfg(debug_assertions)]
{
    let missed = debug_audit_violation_queue(...);
    // ...
}

// After
if tracing::enabled!(Level::DEBUG) {
    let missed = diagnostic::audit_violations(...);
    // ...
}
```

**Keep as `#[cfg(all(debug_assertions, feature = ...))]`:**

Plateau-specific checks that depend on the `dynamic-contour-tracking`
feature's runtime data structures. These cannot be tracing-gated
alone because the types they reference do not exist without the
feature:

- `evict.rs`: `debug_assert_plateau_consistency` call (line 270)
- `split.rs`: `debug_assert_plateau_consistency` calls (lines 161, 259)
- `observe.rs`: `debug_assert_plateau_mirror_consistency` (line 158),
  `debug_check_plateau_sums` (line 135)
- `decay.rs`: `debug_assert_plateau_mirror_consistency` (lines 158, 264)
- `graph_budget.rs`: Pre/post eviction plateau snapshots in
  `evict_candidates`, final plateau mirror consistency

For these, the feature gate stays but the `debug_assertions` gate
is replaced by `tracing::enabled!()`:

```rust
// Before
#[cfg(all(debug_assertions, feature = "dynamic-contour-tracking"))]
debug_assert_plateau_consistency(graph);

// After
#[cfg(feature = "dynamic-contour-tracking")]
if tracing::enabled!(Level::DEBUG) {
    diagnostic::audit_plateau_consistency(graph, "POST-EVICT", Some(&ctx));
}
```

**Keep as `debug_assert!` or `#[cfg(debug_assertions)]`:**

Hard correctness checks that must panic remain compile-time gated:

- `arena.rs`: Slot liveness on `dealloc`/`get`
- `evict.rs`: `is_evictable` precondition, G-root guard
- `gtree.rs`: Zero-width interval check
- `plateau.rs`: Basis insertion duplicate check
- `rebalance.rs`: Safety-net panic (exceeded max iterations)
- `vnode.rs`: `PackedChildren` bounds check

### DC-028-4: Level discipline — ✅ Implemented

| Level   | Semantics                                                         | Example                                                    |
| ------- | ----------------------------------------------------------------- | ---------------------------------------------------------- |
| `ERROR` | Bug detected at runtime (should never fire)                       | Missed violation, safety-net exceeded                      |
| `WARN`  | Unexpected but recoverable state                                  | Node still violated after resolve, P-I2 spot-check failure |
| `DEBUG` | Tree operations at algorithm level                                | Contract, promote, split, evict, decay                     |
| `TRACE` | Queue bookkeeping, guard skips, audit passes, plateau basis walks | Skip destroyed node, plateau key scan                      |

### DC-028-5: Module target hierarchy — ✅ Implemented

All tracing uses the module's natural `target` (Rust's default):

```bash
RUST_LOG=torrust_mudlark::rebalance=debug      # just rebalancing
RUST_LOG=torrust_mudlark::evict=trace          # eviction + plateau detail
RUST_LOG=torrust_mudlark::graph_budget=debug   # eviction wiring + depth gates
RUST_LOG=torrust_mudlark::graph_plateau=trace  # plateau bookkeeping
RUST_LOG=torrust_mudlark::diagnostic=debug     # all audits
RUST_LOG=torrust_mudlark=trace                 # everything
```

## Rationale

### Why span-native for recursive structures?

The GvGraph's mutation pipelines are deeply recursive:

```
observe{g=42}
  ├─ catalytic_split{g=5, lo=0, hi=128, mid=64}
  ├─ rebalance{queue=2}
  │    └─ resolve{node=5}
  │         └─ standard_promote{c=5}
  └─ evict_batch{candidate_count=3}
       └─ evict_tip{v_id=12, gnode=5, parent=3}
            ├─ plateau_after_evict{parent_state=SemiInternal}
            │    └─ displaced=[g7, g8]
            └─ audit_plateau(checkpoint="post-evict", ok=true)
```

A flat log makes this unrecoverable:

```
catalytic_split: g=5
evict_tip: v_id=12
plateau_maintenance: case=self-was-basis
displaced: [g7, g8]
audit_plateau: post-evict, ok=true
```

Tracing spans preserve hierarchy automatically. When the audit fails,
the span stack tells you exactly which split, in which observe, on
which coordinate.

### Why `tracing::enabled!()` instead of `#[cfg(debug_assertions)]`?

1. **Production debugging.** A release binary with
   `RUST_LOG=torrust_mudlark::evict=debug` can capture diagnostic
   output without recompiling. The `#[cfg]` approach requires a
   debug build.

2. **Zero cost when disabled.** The `tracing` macros compile to
   nothing when no subscriber is installed. The `enabled!()` check
   is a single atomic load in release.

3. **Structured capture.** A subscriber can capture span trees to
   JSON, filter by target, or forward to a distributed tracing
   backend. `eprintln!` goes to stderr and is lost.

### Why retain `feature = "dynamic-contour-tracking"` gates?

The plateau mirror (`self.plateaus`, `self.plateau_basis`) is only
available when the `dynamic-contour-tracking` feature is enabled.
Diagnostic functions that inspect these fields cannot compile without
the feature. Replacing the feature gate with a tracing guard alone
would cause compilation errors. The feature gate controls _data
availability_; the tracing gate controls _diagnostic verbosity_.
Both are needed for plateau-specific checks.

### Why consolidate audit functions?

The duplicated `debug_assert_plateau_consistency` functions will
inevitably diverge. One will gain a check the other lacks. By
consolidating into `diagnostic.rs` with a flexible `context`
parameter, we get:

- Single source of truth for plateau consistency
- Caller-provided context (parent state, checkpoint label)
- Consistent tracing output format

### Cost estimate

| File                      | Lines added | Lines removed | Net     |
| ------------------------- | ----------- | ------------- | ------- |
| **`diagnostic.rs`** (new) | ~280        | 0             | +280    |
| **`lib.rs`**              | 1           | 0             | +1      |
| **`evict.rs`**            | ~40         | ~200          | −160    |
| **`split.rs`**            | ~15         | ~25           | −10     |
| **`graph_budget.rs`**     | ~10         | ~50           | −40     |
| **`graph_plateau.rs`**    | ~10         | ~90           | −80     |
| **`observe.rs`**          | ~5          | ~10           | −5      |
| **`rebalance.rs`**        | ~5          | ~30           | −25     |
| **`decay.rs`**            | ~10         | 0             | +10     |
| **`vtree.rs`**            | ~5          | 0             | +5      |
| **Total**                 | ~381        | ~405          | **−24** |

The net reduction comes from deleting the 157-line
`diagnose_missed_violation` wall, the 90-line
`debug_assert_plateau_mirror_consistency` dump, and the two
`debug_assert_plateau_consistency` duplicates.

## Alternatives Considered

### Alternative 1: `#[tracing::instrument]` proc-macro

The `tracing` crate provides `#[instrument]` for automatic span
creation. We rejected this because:

- We need `field::Empty` + `span.record()` for deferred fields
  (e.g., `case` in `vtree_remove_leaf` is determined mid-function).
- We want explicit control over which fields are captured — the
  proc-macro captures all arguments by default, which can be noisy.
- Manual spans are more readable when learning the codebase.

### Alternative 2: Keep `eprintln!` but add filtering

We could wrap `eprintln!` in a custom `DEBUG_PRINT` macro with
runtime filtering. This would provide filtering but not structure.
The recursive call hierarchy would still be lost.

### Alternative 3: Log-based tracing (`log` crate)

The `log` crate is simpler but lacks span context. It provides
levels and targets but not the hierarchical span trees that make
recursive debugging tractable.

## Non-goals

- **No changes to `invariants.rs`:** The test oracle is correct and
  unconditionally compiled for integration tests.
- **No changes to `rebalance.rs` spans:** Already well-instrumented
  (7 spans, 44 tracing calls). Only the
  `debug_audit_violation_queue` function moves to `diagnostic.rs`.
- **No new dependencies:** `tracing` is already in `[dependencies]`,
  `tracing-subscriber` is already in `[dev-dependencies]`.
- **No functional changes:** Every mutation follows the same code
  path. Only the diagnostic output mechanism changes.

## Implementation Order

The implementation is designed for incremental delivery:

1. **Create `diagnostic.rs`** — audit functions, display helpers.
   Pure addition, no breakage risk.

2. **Register in `lib.rs`** — `pub(crate) mod diagnostic;`

3. **Add span to `decay.rs`** — smallest file (272 lines), lowest
   risk, proves the pattern.

4. **Add span to `evict_tip`** — highest value, converts the
   `eprintln!` wall.

5. **Instrument `split.rs`** — add spans to `catalytic_split` and
   `bootstrap_split`, delete duplicate
   `debug_assert_plateau_consistency`.

6. **Clean up `graph_budget.rs`** — add spans to `check_evictions`
   and `evict_candidates`, convert inline audits.

7. **Clean up `graph_plateau.rs`** — convert
   `debug_assert_plateau_mirror_consistency` from `eprintln!` to
   structured tracing.

8. **Move audit from `rebalance.rs`** — move
   `debug_audit_violation_queue` to `diagnostic::audit_violations`.

9. **Add span/events to `vtree.rs`** — single span for
   `vtree_remove_leaf`, `trace!` for case tracking.

10. **Convert `#[cfg]` blocks** — per-file conversion to tracing
    gates, retaining feature gates where needed.

11. **Compile + test** —
    `cargo check --workspace --all-targets --all-features`,
    then `cargo test -p torrust-mudlark --all-features`.

Steps 3–9 are independent per file and can be reviewed separately.

## Follow-on: Plateau Tracing Coverage (028b) — Complete

A post-implementation audit (2026-03-04) confirmed that all
`eprintln!` calls and diagnostic `#[cfg(debug_assertions)]` blocks
were successfully migrated. The plateau maintenance spans identified
as the remaining visibility gap have since been implemented.

All 6 spans (`normalize_plateaus`, `plateau_after_catalytic_split`,
`plateau_after_bootstrap_split`,
`plateau_after_legacy_promotes_batched`, `repair_p_i4`,
`scan_for_candidates`) plus lightweight `trace!`/`debug!` events
on `place_basis_element` match arms are in place.

**Explicitly excluded:** hot-path functions (`route_to_receiver`,
`recompute_g_sums`, `plateau_after_observe`), already-rich
functions (`consolidate_basis_up`), thin wrappers
(`handle_legacy_promotes`), and `invariants.rs`.

## References

- [ADR-M-003](003-violation-tracking.md) — violation queue design
  (eager tracking with work queue)
- [ADR-M-026](026-point-query-and-plateau-semantics.md) — plateau
  invariants (P-I1 through P-I5)
- [ADR-M-030](030-graph-module-decomposition.md) — graph module
  decomposition (complete — `graph_budget.rs`, `graph_query.rs`,
  `graph_extract.rs`, `graph_plateau.rs`, `graph_traits.rs`)
- §IDEA M-8 — observation flow
- §IDEA M-10 — contour refinement (catalytic and bootstrap splits)
- §IDEA M-11 — rebalance algorithm
- §IDEA M-12 — contour simplification (eviction pipeline)
- §IDEA M-14 — temporal semantics (decay)
