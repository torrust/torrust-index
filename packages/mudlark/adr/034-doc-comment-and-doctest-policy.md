# ADR-M-034: Doc-Comment and Doc-Test Policy

**Status:** Decided  
**Date:** 2026-03-04  
**Relates to:** [ADR-M-025](025-public-api-surface.md) (public API
surface), [ADR-M-032](032-three-surface-model.md) (three-surface
model), [ADR-M-033](033-v-generic-importance-properties.md) (V generic bounds)  
**API:** [api.md](../docs/api.md) (entire document)  
**Surface:** 1 + 2 (Prints + Film)

---

## Context

The public API of `torrust-mudlark` is fully specified in
[api.md](../docs/api.md), the three-surface model is documented in
[ADR-M-032](032-three-surface-model.md), and the trait hierarchy is
formalised in [ADR-M-033](033-v-generic-importance-properties.md). What is
missing is a **uniform policy** for translating that specification into
the doc-comments and doc-tests that appear alongside the source code.

Today the crate has:

- Module-level `//!` headers on every file — good.
- `///` doc-comments on most public items — varying in depth and style.
- Doc-tests on a handful of methods (`get`, `layers`) — inconsistent
  presence, no governing standard.
- No doc-tests on any Surface 1 struct, trait definition, or trait
  method.

Without a policy the documentation will remain patchy, and new
contributors will invent ad-hoc styles. We need a standard that:

1. **Guarantees coverage.** Every public item has a doc-test that
   compiles, runs, and demonstrates normal usage.
2. **Keeps doc-tests honest.** They are `cargo test` artifacts — they
   must compile under the same feature matrix the crate ships.
3. **Follows Rust ecosystem conventions.** Mirrors the patterns in
   `std::collections`, `serde`, and `tokio`: one-line summary,
   expanded explanation, then fenced examples.
4. **Scales linearly.** The policy must not be burdensome for a single
   developer to apply and review.

---

## Decision

### D1. Coverage rule — every public item gets a doc-test

Every symbol re-exported from the crate root (see §API M-3) must
carry at least one `/// # Examples` fenced code block that is
**compiled and executed** by `cargo test --doc`. "Compiled and
executed" means the block is plain ` ```rust ` or ` ``` ` — not
` ```ignore `, ` ```no_run `, or ` ```text `.

The coverage mandate applies to:

| Category             | Items                                                                                                                                                                          |
| -------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **Structs**          | `Span`, `Cell`, `Node`, `Pewei`, `Layer`, `Transition`, `Terminal`, `BasisEdge`, `Plateau`, `GNodeId`, `VNodeId`, `Config`, `GvGraph`                                          |
| **Enums**            | `GState`                                                                                                                                                                       |
| **Traits**           | `Coordinate`, `Accumulator`, `Attenuatable`, `Weighable`, `Proratable`, `Inspectable`, `Observation`, `Rng`, `SpatialRead`, `SpatialWrite`, `TemporalDecay`, `WeightedSampler` |
| **Inherent methods** | Every `pub fn` and `pub const fn` on the above types                                                                                                                           |
| **Trait methods**    | Every required and provided method on the above traits                                                                                                                         |

Exception: `#[doc(hidden)]` items (`invariants`, `testing` modules)
are exempt — they are testing affordances, not public API.

### D2. Doc-comment anatomy

Every public item follows the same four-part structure. Parts 2–4 are
optional when the item is trivially self-evident (e.g. a single-field
newtype accessor), but Part 1 is always required.

````text
/// 1. One-line summary — imperative mood, no trailing period.
///
/// 2. Extended description (optional). May span multiple paragraphs.
///    Reference ADRs, api.md sections, or idea.md sections with
///    `[ADR-M-NNN](…)` links or `(ADR-M-NNN)` parentheticals.
///    Use `$…$` for inline math and `$$…$$` for display math where
///    the spec uses formulae (e.g. cost bounds, decay equations).
///
/// # Panics                           ← 3a. If the method can panic.
///
/// Panics if `coord` is NaN.
///
/// # Errors                           ← 3b. If the method returns Result.
///
/// # Safety                           ← 3c. If the method is unsafe.
///
/// # Examples                         ← 4. Always present.
///
/// ```
/// use torrust_mudlark::{GvGraph, Config, Cell};
/// // … minimal self-contained example …
/// ```
````

### D3. Example style rules

1. **Self-contained.** Every example must compile in isolation. It may
   use only `torrust_mudlark` public re-exports and `std`. No
   `extern crate`, no test-only helpers.

2. **Hidden boilerplate with `# `.** Common setup (building a `Config`,
   constructing a `GvGraph`, seeding a few observations) should be
   hidden behind `# ` lines so the rendered documentation highlights
   the API under demonstration, not scaffold.

3. **Canonical config snippet.** To avoid copy-pasting divergent
   configs across dozens of doc-tests, use this hidden preamble
   wherever a live `GvGraph` is needed:

   ````rust
   /// ```
   /// # use torrust_mudlark::{Config, GvGraph};
   /// # let cfg = Config {
   /// #     split_threshold: 5u64,
   /// #     depth_create: 3,
   /// #     depth_evict: 6,
   /// #     budget: None,
   /// #     alpha_relax: 0.75,
   /// #     bounded_eviction: true,
   /// # };
   /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
   /// ```
   ````

   The concrete types `<u64, u64, 8>` keep examples simple — `u64`
   covers integer and float-convertible paths and `N = 8` gives a
   domain of `[0, 256)` that is easy to reason about.

4. **Assert outcomes.** Examples should contain at least one `assert!`
   or `assert_eq!` that verifies observable behaviour. This turns
   doc-tests into lightweight regression tests.

5. **No `unwrap()` on infallible paths.** Methods that never return
   `None` or `Err` should not use `unwrap()` in examples. For methods
   that return `Option` (e.g. `sample`), an `if let` or `.unwrap()` is
   acceptable with a comment explaining when `None` occurs.

6. **One concept per example.** If a method interacts with several
   features, show the primary use first, then add a second `# Examples`
   block (Rust supports multiple) for the secondary use.

### D4. Struct-level versus method-level examples

- **Struct / enum doc-comment:** Show construction and field access.
  For view types (`Cell`, `Span`, `Node`, …) the example should obtain
  the value from a `GvGraph` method (never construct via struct
  literal — those fields are public but the idiomatic path is through
  the graph).

- **Method doc-comment:** Show the method call and its return value.

- **Trait definition doc-comment:** Show a usage pattern that exercises
  the trait bound (e.g. `fn process<V: Accumulator>(…)`), **not** a
  manual implementation. The blanket impls cover all built-in types;
  doc-tests should not encourage users to write their own unless the
  trait is designed for extension (only `Rng` and `Observation<V>` are).

### D5. Module-level (`//!`) doc-comments

Each `pub(crate)` module that hosts re-exported types carries a `//!`
header. These headers:

- Open with a one-line summary.
- List the types defined in the module in a table.
- Reference the governing ADR(s).
- Do NOT carry a `# Examples` section — examples live on the
  individual items, not the module.

### D6. Surface 3 items — no requirement

Surface 3 (`pub(crate)`) items have **no doc-test requirement**. Plain
`///` comments explaining intent and invariants are encouraged but not
mandated. `/// ```text` blocks (ASCII diagrams) are welcome and do not
count as executable doc-tests.

### D7. CI enforcement

The existing `cargo test` invocation already runs doc-tests. To
prevent regressions:

- `#![deny(missing_docs)]` on `lib.rs` ensures every public item has
  a doc-comment. (Does not enforce doc-tests — that is a review-time
  check.)
- `#![doc(test(attr(deny(warnings))))]` on `lib.rs` ensures doc-test
  code compiles warning-free.
- A future `#![deny(rustdoc::missing_doc_code_examples)]` (nightly
  lint) can be enabled once stabilised. Until then, coverage is
  enforced by code review against this ADR's checklist.

### D8. Feature-gated examples

When a doc-test exercises a feature-gated path, annotate the fence:

````text
/// ```
/// # // Requires: `serde` feature.
/// # #[cfg(feature = "serde")]
/// # {
/// let json = serde_json::to_string(&cell).unwrap();
/// # }
/// ```
````

For the default feature set (`dynamic-contour-tracking`, `rand`)
plus the opt-in `serde` feature, no annotation is needed when running
with `--all-features` per the workspace `AGENTS.md`.

### D9. Math in doc-comments

Where the spec uses formulae, reproduce them in doc-comments using
KaTeX-compatible `$…$` (inline) and `$$…$$` (display) notation.
`rustdoc` renders these when `--html-in-header katex.html` is used;
in plain terminal rendering they degrade gracefully to source TeX.

Examples:

```text
/// Expected cost: $O(1.44\,H)$ where $H$ is the Shannon entropy
/// of the intensity distribution (§IDEA M-18.2, §THEORY M-4.3).
```

```text
/// Per-depth decay factor:
///
/// $$\ln\lambda(d) = \ln(\text{att}) \cdot (1 + q \cdot (2d / D - 1))$$
```

### D10. Cross-references

- Link to api.md sections: `[§API M-5.2](../docs/api.md#52-…)`.
- Link to ADRs: `[ADR-M-NNN](../adr/NNN-slug.md)` from source comments,
  or `(ADR-M-NNN)` parenthetical in brief one-liners.
- Link across items: use intra-doc links — `[`Config`]`,
  `[`GvGraph::observe`]`, `[`Cell::to_span`]`.

---

## Implementation checklist

The checklist below catalogues every public item that requires a
doc-test. Items already documented are marked; the rest are the
implementation backlog.

### Surface 1 — Prints

#### Spot readings

| Item                  | Doc-test |
| --------------------- | -------- |
| `Span<C, V>` (struct) | ☑        |
| `Span::width`         | ☑        |
| `Cell<C, V>` (struct) | ☑        |
| `Cell::to_span`       | ☑        |
| `Cell::width`         | ☑        |
| `Cell::is_final`      | ☑        |
| `Node<C, V>` (struct) | ☑        |
| `Node::to_span`       | ☑        |
| `Node::to_cell`       | ☑        |
| `Node::is_terminal`   | ☑        |
| `Node::refinement`    | ☑        |
| `Node::width`         | ☑        |
| `GState` (enum)       | ☑        |

#### Contact print (PEWEI)

| Item                        | Doc-test |
| --------------------------- | -------- |
| `Pewei<C, V>` (struct)      | ☑        |
| `Pewei::layer_count`        | ☑        |
| `Pewei::node_count`         | ☑        |
| `Pewei::total_energy`       | ☑        |
| `Pewei::reconstruct`        | ☑        |
| `Layer<C, V>` (struct)      | ☑        |
| `Transition<C, V>` (struct) | ☑        |
| `Transition::snr`           | ☑        |
| `Transition::width`         | ☑        |
| `Terminal<C, V>` (struct)   | ☑        |
| `Terminal::width`           | ☑        |

#### Contour map

| Item                     | Doc-test |
| ------------------------ | -------- |
| `BasisEdge<C>` (struct)  | ☑        |
| `Plateau<C, V>` (struct) | ☑        |
| `Plateau::width`         | ☑        |
| `Plateau::to_span`       | ☑        |

#### Contour range decomposition (ADR-M-037)

| Item                             | Doc-test |
| -------------------------------- | -------- |
| `BasisElement<C, V>` (struct)    | ☑        |
| `ContourRange<C, V>` (struct)    | ☑        |
| `ContourRangeEnergy<V>` (struct) | ☑        |

#### Handles

| Item                  | Doc-test |
| --------------------- | -------- |
| `GNodeId` (struct)    | ☑        |
| `GNodeId::from_index` | ☑        |
| `GNodeId::index`      | ☑        |
| `VNodeId` (struct)    | ☑        |
| `VNodeId::from_index` | ☑        |
| `VNodeId::index`      | ☑        |

### Surface 2 — Film

#### Configuration and graph

| Item                            | Doc-test |
| ------------------------------- | -------- |
| `Config<V>` (struct)            | ☑        |
| `GvGraph<C, V, N>` (struct)     | ☑        |
| `GvGraph::new`                  | ☑        |
| `GvGraph::from_observations`    | ☑        |
| `GvGraph::observe`              | ☑        |
| `GvGraph::get`                  | ☑        |
| `GvGraph::plateaus`             | ☑        |
| `GvGraph::range_sum`            | ☑        |
| `GvGraph::contour_range`        | ☑        |
| `GvGraph::contour_range_energy` | ☑        |
| `GvGraph::select_plateaus`      | ☑        |
| `GvGraph::sample`               | ☑        |
| `GvGraph::extract`              | ☑        |
| `GvGraph::layers`               | ☑        |
| `GvGraph::decay`                | ☑        |
| `GvGraph::check_evictions`      | ☑        |
| `GvGraph::node_count`           | ☑        |
| `GvGraph::terminal_count`       | ☑        |
| `GvGraph::budget`               | ☑        |
| `GvGraph::total_sum`            | ☑        |
| `GvGraph::config`               | ☑        |
| `GvGraph::g_root`               | ☑        |
| `GvGraph::v_root`               | ☑        |
| `GvGraph::depth_evict`          | ☑        |
| `GvGraph::depth_create`         | ☑        |
| `GvGraph::depth_buffer`         | ☑        |
| `GvGraph::headroom`             | ☑        |
| `GvGraph::soft_limit`           | ☑        |
| `Extend<(C, O)>` impl           | ☑        |

#### Chemistry contracts (traits)

| Item                     | Doc-test |
| ------------------------ | -------- |
| `Coordinate` (trait)     | ☑        |
| `Accumulator` (trait)    | ☑        |
| `Attenuatable` (trait)   | ☑        |
| `Weighable` (trait)      | ☑        |
| `Proratable` (trait)     | ☑        |
| `Inspectable` (trait)    | ☑        |
| `Observation<V>` (trait) | ☑        |
| `Rng` (trait)            | ☑        |

#### Instruments (traits)

| Item                      | Doc-test |
| ------------------------- | -------- |
| `SpatialRead` (trait)     | ☑        |
| `SpatialWrite` (trait)    | ☑        |
| `TemporalDecay` (trait)   | ☑        |
| `WeightedSampler` (trait) | ☑        |

---

## Consequences

- **Every public item will carry a runnable example.** `cargo test
--doc -p torrust-mudlark` becomes an exhaustive smoke-test of the
  public API — any signature change that breaks compatibility will
  fail a doc-test before it reaches review.

- **Uniform style reduces review friction.** The four-part anatomy
  (summary, description, panics, examples) is checkable in seconds.

- **Hidden boilerplate keeps rendered docs clean.** Users see the API
  call, not 10 lines of config construction.

- **The canonical config snippet acts as a living reference.** If
  `Config` fields change, updating one snippet pattern fixes all
  doc-tests — grep for `split_threshold: 5u64` to find them all.

- **`#![deny(missing_docs)]` catches omissions at compile time.**
  Adding a `pub` item without a doc-comment is a hard error.

- **Surface 3 is unaffected.** Internal modules remain lightly
  documented at the author's discretion — the mandate applies only to
  the api.md surface.

- **The implementation checklist is the backlog.** Each ☐ → ☑
  transition is a single reviewable PR (or a batch per module). No
  design decisions remain — this is mechanical work guided by the
  style rules above.
