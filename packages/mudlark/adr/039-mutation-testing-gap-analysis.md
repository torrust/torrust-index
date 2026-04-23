# ADR-M-039: Mutation Testing Gap Analysis

**Status:** Decided  
**Date:** 2026-03-20  
**Relates to:** [ADR-M-032](032-three-surface-model.md) (three-surface model),
[ADR-M-035](035-benchmarking-framework.md) (benchmarking framework),
[ADR-M-037](037-contour-range-queries.md) (contour range queries),
[ADR-M-038](038-decay-factor-table-depth-bound.md) (decay factor table depth bound)  
**Spec:** §IDEA M-5 (invariants), §IDEA M-5.5 (queries),
§IDEA M-5.6 (plateaus), §§CR.1–CR.14 (contour ranges)  
**API:** [§API M-5.2](../docs/api.md#52-gvgraphc-v-n--the-negative) (all
public operations)  
**Surface:** 1 + 2 + 3 (Prints + Film + Emulsion)

---

## Context

### Mutation testing run

A `cargo-mutants` run against the mudlark package produced ~305
surviving mutants. Analysis of the survivors reveals a systematic
pattern: the test suite is heavily oriented toward **invariant-
preservation testing** ("does the structure stay valid?") and
under-oriented toward **output-correctness testing** ("does the
structure produce the right answer?"). This creates a blind spot
where mutations that produce wrong values but maintain valid
structure survive undetected.

### The three blind spots

1. **Value correctness.** Mutations that produce wrong *values* but
   valid *structure* survive. The G-Tree can carry incorrect sums,
   plateaus can have wrong energies, and PEWEI reconstruction can
   produce wrong spans — all without any invariant violation.

2. **Boundary precision.** Invariants verify properties like "basis
   elements tile the range" but not "this specific element is
   included and that one isn't." Off-by-one boundary mutations
   produce slightly different tilings that still satisfy the
   structural invariants.

3. **Type coverage.** The property-testing infrastructure and most
   concrete tests use `u64`, leaving `f32` (and to a lesser extent
   specific `f64` trait methods) as an untested parallel
   implementation.

### Mutant classification

The surviving mutants cluster into seven classes:

| Class | Count | Risk | Root cause |
|-------|-------|------|------------|
| 1. Invariant checker self-consistency | ~50 | Low-Med | No "checker self-tests" |
| 2. Diagnostic / display / tracing | ~25 | None | No functional contract to test against |
| 3. Test infrastructure / plan generators | ~100 | Low | Property-based tests don't verify test data |
| 4. `f32` trait implementations | ~30 | Medium | Type coverage gap |
| 5. Plateau values | ~35 | **High** | Values not asserted, only structure |
| 6. Basis decomposition boundaries | ~40 | **High** | Boundary conditions untested |
| 7. Scattered production logic | ~25 | **High** (7d), Med (others) | Output values not cross-checked |

Classes 2 and 3 are structurally expected and do not warrant
remediation. The remaining classes require targeted test additions.

---

## Decisions

### D1. PEWEI reconstruct value verification (Class 7d — critical)

**Problem.** `reconstruct()` in `pewei.rs` has surviving `+ → *`
mutations on both additive terms (lines 340:66 and 340:104). The
reconstruction output is never checked against known expected values.

**Decision.** Add tests that verify `reconstruct()` spans against
independently computed expected values. Assert the energy
conservation identity:

$$\sum_i \text{span}_i.\text{intensity} \times \text{span}_i.\text{width} = \text{total\_sum}()$$

Tests belong in `src/tests/` (crate-level, since `reconstruct` is
Surface 3).

### D2. Plateau value-assertion tests (Class 5 — high)

**Problem.** Plateau sums and energies are not asserted after known
observation sequences. Mutations that corrupt sums (e.g. `+= → *=`
in `compute_plateau_energy`) survive because the plateau *tiling*
remains valid.

**Decision.** Add value-assertion tests for plateaus:

- After known observation sequences, assert specific plateau sums
  and basis element counts. E.g. "after 10 observations of weight 1
  at coordinate 5, the plateau containing 5 should have sum 10."
- Round-trip conservation: $\sum_p \text{plateau}_p.\text{sum}$
  over all plateaus should equal `total_sum()` (modulo thatching).
- Cross-check: `contour_range` energy should equal
  $\sum_i \text{basis}[i].\text{sum}$ by independent summation.

Tests belong in `tests/` (integration, Surface 1+2 API) and
`src/tests/` (crate-level for internal plateau sums).

### D3. Basis decomposition boundary tests (Class 6 — high)

**Problem.** `decompose_basis` in `graph_query.rs` has ~30 surviving
boundary mutations (`< → <=`, `> → >=`, `> → ==`, etc.). Tests
query ranges that never fall exactly on G-node boundaries, and don't
verify specific basis set membership.

**Decision.** Add boundary-specific decomposition tests:

- Construct minimal graphs with known structure (e.g. exactly 4 leaf
  nodes at known ranges) and verify basis decomposition
  element-by-element for every possible half-open interval.
- Queries whose endpoints fall exactly on G-node midpoints,
  endpoints, and at every possible alignment relative to the dyadic
  grid.
- Semi-internal thatching boundary: construct a graph with
  semi-internal nodes and verify basis decomposition handles the
  uncovered-half boundary correctly.

Tests belong in `src/tests/` (crate-level, since `decompose_basis`
is Surface 3).

### D4. `f32` type coverage (Class 4 — medium)

**Problem.** Trait implementations for `f32` (`Coordinate`,
`Accumulator`, `Attenuatable`, `Weighable`, `Proratable`,
`Inspectable`) have ~30 surviving mutants. The test suite
overwhelmingly uses `u64`.

**Decision.** Add a parallel test suite that replays a
representative subset of existing test scenarios with
`GvGraph<f32, f32, 8>`. The existing `cross_type.rs` integration
test file is the natural home. Key operations to cover:

- Splitting (exercises `midpoint`)
- `range_sum` (exercises `prorate`)
- Sampling (exercises `weight`)
- Decay (exercises `attenuate`)
- Point query (exercises `to_f64_approx`)

### D5. Invariant checker self-tests (Class 1 — low-medium)

**Problem.** The invariant checker is tested only indirectly — as a
verification tool, never as a subject. Mutating a check either
weakens it (still passes on correct data) or strengthens it (would
reject correct data only if the tighter condition is violated in the
test corpus). No test constructs a known-bad graph state and asserts
that the checker catches it.

**Decision.** Create a suite of "checker self-tests" that construct
deliberately invalid graph states and assert that
`check_all_invariants()` reports the specific expected violation.
One test per invariant family:

- G-I1 (summation), V-I3 (intensity ordering), V-I6b (evictable
  flag), P-I1 (contour step keys), P-I3 (basis disjointness),
  P-I4 (basis consolidation).

Tests belong in `src/tests/` (crate-level, since they must
construct invalid internal state).

### D6. Rebalance escalation and decay arithmetic (Class 7a, 7c — medium)

**Problem.** `escalate_after_promote` in `rebalance.rs` has multiple
surviving mutations in the 3-node post-promotion restructuring path.
`decay_selective` has arithmetic mutations (`- → +`, `/ → %`) that
produce wrong decay factors.

**Decision.**

- **Rebalance:** Construct adversarial V-Tree configurations that
  specifically trigger escalation and verify the resulting structure
  post-promote.
- **Decay:** After decay, verify specific node values against
  hand-computed expected results. Compare uniform vs. selective
  paths for consistency on the same graph.

### D7. Config validation boundaries (Class 7e — low)

**Problem.** Config validation formula mutations survive because
tests don't exercise configs at the exact validation boundary.

**Decision.** Add tests for configs at exact validation boundaries:
budget exactly equal to the minimum, `depth_create == depth_evict -
1`, etc.

### D8. No remediation for diagnostic/display code (Class 2)

**Decision.** Diagnostic, `Display`, and tracing-gate mutations are
explicitly excluded from remediation. They have no functional
contract. If snapshot tests are desired in future, they can be added
without an ADR.

### D9. No remediation for test infrastructure (Class 3)

**Decision.** Plan generators and test config mutations are
explicitly excluded. They are means to an end, never the subject
under test. Plan diversity is sufficient for invariant stress-
testing purposes.

---

## Triage Order

The recommended implementation order, by risk and effort:

1. **D1** — PEWEI reconstruct (critical, bounded effort)
2. **D2** — Plateau values (high risk, medium effort)
3. **D3** — Basis boundaries (high risk, medium effort)
4. **D4** — `f32` type coverage (medium risk, medium effort)
5. **D5** — Invariant checker self-tests (low-medium risk, low effort)
6. **D6** — Rebalance + decay (medium risk, varies)
7. **D7** — Config validation (low risk, low effort)

---

## Consequences

- **Positive.** The test suite will transition from pure
  invariant-preservation testing to a hybrid model that also verifies
  output correctness. This closes the systematic blind spot
  identified by mutation testing.
- **Positive.** `f32` becomes a first-class tested type, matching
  its status as a supported coordinate and accumulator.
- **Positive.** The invariant checker gains its own test coverage,
  resolving the "who watches the watchmen?" problem.
- **Negative.** Test volume increases. The new value-assertion tests
  are inherently more brittle than property-based invariant tests —
  they encode specific expected outputs that must be updated if the
  algorithm's numerical behaviour changes intentionally.
- **Neutral.** Classes 2 and 3 (~125 mutants) remain structurally
  unkillable. This is acceptable and expected.
